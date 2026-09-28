#!/usr/bin/env python3
"""`rbb compare --json` statistics split by who bid in BBA's auction.

    probes/tools/compare_split.py RUN.json [RUN.json ...] [--cards NS/EW]
        [--scenario PATTERN] [--label NAME] [--by contest|par] [--markdown]
        [--write-subset OUT.json] [--check-par]

One row per class of board: all, passed out, uncontested (NS bid alone,
EW bid alone), competitive (both sides bid), the classes `rbb compare
--auctions` uses (BBA's auction defines the board); with `--by par`, by
the size of the par score instead (partscore, game, slam). Columns:

- calls: replay agreement, our call against BBA's at each position of
  BBA's auction.
- auction, contract: identical auctions, same final contract.
- vs BBA: on boards where the contracts differ, BBA's IMP distance from
  double-dummy par minus ours, summed (positive: ours closer), then per
  board of the class; ours closer / BBA closer / equal counts.
- |par| BBA, |par| ours: mean IMP distance from par over every board
  with a table, how far each side's contracts are from par at all (the
  hardness of the deals, which "vs BBA" leaves out); at par: the share
  of boards exactly at par (0 IMPs). The compare JSON has par only where
  the contracts differ, so this script works it out for the rest
  (`--check-par` checks it against the engine's where both exist).
- per bd: "vs BBA" over every board of the class with a table.

`--cards 21GF-DEFAULT/21GF-GIB` keeps the boards bid with that card pair;
`--scenario` takes a glob. `--write-subset` saves the kept boards as a
compare JSON (`{"boards": [...]}`) for par_blame.py and sideimps.py.
"""
import argparse
import fnmatch
import json

IMP_STEPS = [20, 50, 90, 130, 170, 220, 270, 320, 370, 430, 500, 600, 750, 900,
             1100, 1300, 1500, 1750, 2000, 2250, 2500, 3000, 3500, 4000]


def imps(d):
    return sum(1 for v in IMP_STEPS if abs(d) >= v)


ST = 'CDHSN'  # the JSON table's strain order; rows are N, E, S, W


def score(level, strain, dbl, tricks, vul):
    """Declarer's duplicate score (dbl: 0, 1, 2)."""
    need = level + 6
    if tricks >= need:
        per_trick = 20 if strain in 'CD' else 30
        trick_score = (per_trick * level + (10 if strain == 'N' else 0)) * (1, 2, 4)[dbl]
        s = trick_score + ((500 if vul else 300) if trick_score >= 100 else 50)
        s += (750 if vul else 500) if level == 6 else (1500 if vul else 1000) if level == 7 else 0
        over = tricks - need
        return s + 50 * dbl + (over * per_trick if dbl == 0 else over * (200 if vul else 100) * dbl)
    down = need - tricks
    if dbl == 0:
        return -down * (100 if vul else 50)
    pen = 200 + 300 * (down - 1) if vul else 100 + 200 * min(down - 1, 2) + 300 * max(down - 3, 0)
    return -pen * dbl


def vulnerable(vul, side):
    return vul in ('Both', 'All') or vul == ('NorthSouth' if side == 0 else 'EastWest')


def par_ns(tricks, vul):
    """Double-dummy par for North-South, as a game: the sides take turns
    to outbid the contract or let it stand; a contract that fails is
    doubled. Checked against the engine's par on every board the compare
    JSON carries one for (--check-par)."""
    val = {}
    for side in (0, 1):  # 0 NS, 1 EW
        v = vulnerable(vul, side)
        for r in range(35):
            level, st = r // 5 + 1, ST[r % 5]
            t = max(tricks[side][r % 5], tricks[side + 2][r % 5])
            sc = score(level, st, 0 if t >= level + 6 else 1, t, v)
            val[(r, side)] = sc if side == 0 else -sc
    memo = {}

    def game(r, holder):
        # `holder` bid contract r; the other side may outbid it or pass.
        if (r, holder) in memo:
            return memo[(r, holder)]
        other = 1 - holder
        best = val[(r, holder)]
        for r2 in range(r + 1, 35):
            x = game(r2, other)
            best = max(best, x) if other == 0 else min(best, x)
        memo[(r, holder)] = best
        return best

    # North-South may open first; if they pass, East-West may open or
    # pass it out, and either opening is still open to being outbid.
    ns_open = max(game(r, 0) for r in range(35))
    ew_open = min(game(r, 1) for r in range(35))
    return max(ns_open, min(0, ew_open))


def contract_ns(calls, dealer, tricks, vul):
    """North-South's double-dummy score for the auction's final contract."""
    dl = 'NESW'.index(dealer[0])
    last, dbl = None, 0
    for i, c in enumerate(calls):
        if isinstance(c, dict):
            last, dbl = (i, c['Bid']['level'], c['Bid']['strain']), 0
        elif c == 'Double':
            dbl = 1
        elif c == 'Redouble':
            dbl = 2
    if not last:
        return 0
    i, level, strain = last
    side = (dl + i) % 2
    first = next(j for j, c in enumerate(calls) if isinstance(c, dict)
                 and c['Bid']['strain'] == strain and (dl + j) % 2 == side)
    seat = (dl + first) % 4
    st = 'N' if strain == 'NoTrump' else strain[0]
    sc = score(level, st, dbl, tricks[seat][ST.index(st)], vulnerable(vul, side))
    return sc if side == 0 else -sc


def contest(dealer, calls):
    seat = 'NESW'.index(dealer[0])
    bid = [False, False]
    for c in calls:
        if c != 'Pass':
            bid[seat % 2] = True
        seat = (seat + 1) % 4
    return {(False, False): 'passed out', (True, False): 'NS only',
            (False, True): 'EW only', (True, True): 'competitive'}[tuple(bid)]


def stats(boards):
    s = dict(boards=len(boards), agree=0, calls=0, auctions=0, contracts=0,
             scored=0, ours=0, bba=0, equal=0, net=0, with_dd=0, d_bba=0, d_ours=0,
             bba_at_par=0, ours_at_par=0)
    for b in boards:
        s['calls'] += len(b['reference'])
        s['agree'] += sum(r == e for r, e in zip(b['reference'], b['replay']))
        s['auctions'] += b['first_divergence'] is None
        same = b['reference_contract'] == b['our_contract']
        s['contracts'] += same
        if not b.get('dd'):
            continue
        t, p = b['dd']['tricks'], b.get('par')
        if p:
            par, ref, ours = p['par_ns'], p['reference_ns'], p['ours_ns']
        else:
            # rbb compare fills par in only where the contracts differ.
            par = par_ns(t, b['vul'])
            ref = contract_ns(b['reference'], b['dealer'], t, b['vul'])
            ours = ref if same else contract_ns(b['ours'], b['dealer'], t, b['vul'])
        o, r = imps(ours - par), imps(ref - par)
        s['with_dd'] += 1
        s['d_bba'] += r
        s['d_ours'] += o
        s['bba_at_par'] += r == 0
        s['ours_at_par'] += o == 0
        if same:
            continue
        s['scored'] += 1
        s['net'] += r - o
        s['ours' if o < r else 'bba' if r < o else 'equal'] += 1
    return s


PAR_BANDS = ['par 0', 'par < 300', 'par 300-910', 'par 920+', 'no table']


def klass(b):
    """The board's band by the size of its par score: under 300 is a
    partscore (or a cheap sacrifice), 920 and up a slam."""
    if not b.get('dd'):
        return 'no table'
    p = b.get('par')
    v = abs(p['par_ns'] if p else par_ns(b['dd']['tricks'], b['vul']))
    return 'par 0' if v == 0 else 'par < 300' if v < 300 else 'par 300-910' if v < 920 else 'par 920+'


def pct(a, b):
    return f"{100 * a / b:.1f}%" if b else "-"


def per(a, b):
    return f"{a / b:+.2f}" if b else "-"


def mean(a, b):
    return f"{a / b:.2f}" if b else "-"


COLUMNS = ["class", "boards", "share", "calls", "auction", "contract", "vs BBA",
           "per bd", "ours closer", "BBA closer", "equal", "|par| BBA", "|par| ours",
           "BBA at par", "ours at par"]


def row(name, s, total):
    return [name, str(s['boards']), pct(s['boards'], total), pct(s['agree'], s['calls']),
            pct(s['auctions'], s['boards']), pct(s['contracts'], s['boards']),
            f"{s['net']:+d}", per(s['net'], s['with_dd']), str(s['ours']), str(s['bba']),
            str(s['equal']), mean(s['d_bba'], s['with_dd']), mean(s['d_ours'], s['with_dd']),
            pct(s['bba_at_par'], s['with_dd']), pct(s['ours_at_par'], s['with_dd'])]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('files', nargs='+')
    ap.add_argument('--cards', help='NS/EW card pair, e.g. 21GF-DEFAULT/21GF-GIB')
    ap.add_argument('--scenario', help='glob on the scenario name')
    ap.add_argument('--label', default='')
    ap.add_argument('--markdown', action='store_true')
    ap.add_argument('--write-subset')
    ap.add_argument('--by', choices=['contest', 'par'], default='contest',
                    help="rows by who bid in BBA's auction (default) or by the par score's size")
    ap.add_argument('--check-par', action='store_true',
                    help="check this script's par and contract scores against the engine's")
    a = ap.parse_args()
    boards = []
    for f in a.files:
        boards += json.load(open(f))['boards']
    if a.cards:
        ns, ew = a.cards.split('/')
        boards = [b for b in boards if b['ns_card'] == ns and b['ew_card'] == ew]
    if a.scenario:
        boards = [b for b in boards if fnmatch.fnmatchcase(b['scenario'], a.scenario)]
    if a.write_subset:
        json.dump({'boards': boards}, open(a.write_subset, 'w'))
    if a.check_par:
        n = bad = 0
        for b in boards:
            p = b.get('par')
            if not (p and b.get('dd')):
                continue
            t = b['dd']['tricks']
            got = (par_ns(t, b['vul']), contract_ns(b['reference'], b['dealer'], t, b['vul']),
                   contract_ns(b['ours'], b['dealer'], t, b['vul']))
            n += 1
            if got != (p['par_ns'], p['reference_ns'], p['ours_ns']):
                bad += 1
                if bad <= 5:
                    print('mismatch', b['scenario'], b['board'], got, p)
        print(f'par check: {n - bad}/{n} boards agree with the engine')
    classes = {}
    for b in boards:
        classes.setdefault(klass(b) if a.by == 'par' else contest(b['dealer'], b['reference']),
                           []).append(b)
    rows = [row('all', stats(boards), len(boards))]
    order = PAR_BANDS if a.by == 'par' else ['passed out', 'NS only', 'EW only', 'competitive']
    for c in order:
        rows.append(row(c, stats(classes.get(c, [])), len(boards)))
    scen = sorted({b['scenario'] for b in boards})
    no_par = sum(1 for b in boards if not b.get('dd'))
    head = (f"{a.label or ', '.join(a.files)}: {len(boards)} boards, {len(scen)} scenarios"
            + (f", {no_par} without a double-dummy table" if no_par else ""))
    if a.markdown:
        print(f"**{head}**\n")
        print("| " + " | ".join(COLUMNS) + " |")
        print("|" + "|".join(["---"] + ["---:"] * (len(COLUMNS) - 1)) + "|")
        for r in rows:
            print("| " + " | ".join(r) + " |")
        print()
    else:
        print(head)
        w = [max(len(c), *(len(r[i]) for r in rows)) for i, c in enumerate(COLUMNS)]
        print("  ".join(c.rjust(w[i]) if i else c.ljust(w[i]) for i, c in enumerate(COLUMNS)))
        for r in rows:
            print("  ".join(x.rjust(w[i]) if i else x.ljust(w[i]) for i, x in enumerate(r)))


if __name__ == '__main__':
    main()
