#!/usr/bin/env python3
"""Survey of 4NT calls: how our rules read each 4NT (BBA's and ours)
against Rick's default (2026-10-03):

  agreed suit (trump set, or shown minimums make 8)  -> keycard in it
                                                        (a major beats a minor)
  else our side has bid notrump                       -> quantitative
  else                                                -> keycard in the last suit our side bid

    probes/tools/keycard_survey.py RUN.json [--set path=value ...] [--rules DIR]
        [--who bba|ours|both] [--out rows.json] [--examples]
    probes/tools/keycard_survey.py --rows rows.json [--who bba] [--examples]

RUN.json is an `rbb compare --json` run. Every 4NT in BBA's auction and
ours is read with `rbb explain-auction` (RBB, default target/release/rbb;
the same `--set` card changes as the compare, e.g. the vanilla set's) and
classed by Rick's default, worked out from the auction and the shown
minimums before the call, against our reading. Limits of the classifier:
any notrump call counts as notrump bid (a 2NT feature ask, Jacoby 2NT),
"competitive" is marked only for the last-suit class, and a 4NT that
answers a question is counted apart. Run from the repository root
(rkcb-1430.notes.md, "The keycard suit", 2026-10-03).
"""
import argparse, collections, json, subprocess, sys, os
from concurrent.futures import ThreadPoolExecutor

RBB = os.environ.get('RBB', os.path.abspath('target/release/rbb'))
CARD_DIRS = ('cards/bbsa', '.rbb-cache/random/pbs/bbsa', '../Practice-Bidding-Scenarios/bbsa')
SUITS = 'CDHS'  # explain-auction len order
SEATS = 'NESW'


def call(c):
    """A call as the compare JSON writes it (object or PBN string), in PBN."""
    if isinstance(c, str):
        return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}.get(c, c)
    b = c['Bid']
    return f"{b['level']}{b['strain'][0]}"


def card_arg(name):
    if name.endswith('.bbsa') or name.endswith('.json'):
        return name
    for d in CARD_DIRS:
        p = f'{d}/{name}.bbsa'
        if os.path.exists(p):
            return p
    return name


def explain(ns, ew, dealer, calls, sets, rules):
    a = ' '.join({'P': 'Pass', 'X': 'X', 'XX': 'XX'}.get(c, c.replace('N', 'NT') if c.endswith('N') else c) for c in calls)
    cmd = [RBB, 'explain-auction', '-c', card_arg(ns), '--ew-card', card_arg(ew), '-d', dealer[0], '-a', a, '--json']
    if rules:
        cmd += ['--rules', rules]
    for x in sets:
        cmd += ['--set', x]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        return None
    return json.loads(r.stdout)


def rick(calls, i, rows, side_idx, seat_of):
    """Rick's default for the 4NT at index i."""
    me = seat_of(i)
    side = me % 2
    ours = [j for j in range(i) if seat_of(j) % 2 == side]
    if i == 0:
        return ('skip', None, 'opening')
    before = rows[i - 1]
    st = before['sides'][side]
    if st.get('ask') or st.get('answered'):
        pass
    # agreement
    agreed = None
    t = st.get('trump')
    shown = {}
    for s_i, s in enumerate(SUITS):
        lo = before['seats'][me]['len'][s_i]['lo'] + before['seats'][(me + 2) % 4]['len'][s_i]['lo']
        shown[s] = lo
    pub = [s for s in 'SHDC' if shown[s] >= 8]
    tr = {'Clubs': 'C', 'Diamonds': 'D', 'Hearts': 'H', 'Spades': 'S', 'NoTrump': 'N'}.get(t) if t else None
    if tr and tr in 'CD' and any(s in 'HS' for s in pub):
        return ('keycard', [s for s in pub if s in 'HS'][0], 'major replaces minor')
    if tr and tr in 'CDHS':
        return ('keycard', tr, 'trump set')
    if pub:
        return ('keycard', pub[0], 'shown 8')
    if any(calls[j] in ('1N', '2N', '3N') for j in ours):
        return ('quant', None, 'NT bid')
    last = [j for j in ours if calls[j][0].isdigit() and calls[j][1] in 'CDHS']
    if not last:
        return ('skip', None, 'no suit bid by us')
    j = last[-1]
    s = calls[j][1]
    if before['seats'][seat_of(j)]['len'][SUITS.index(s)]['lo'] < 2:
        return ('skip', s, 'last suit artificial')
    return ('keycard', s, 'last suit' + (' (they bid)' if any(seat_of(k) % 2 != side and calls[k] not in ('P',) for k in range(i)) else ''))


def engine_class(expl):
    e = expl or ''
    if e.startswith('Keycard ask in ') or e.startswith('Keycard ask, agreeing'):
        s = {'♠': 'S', '♥': 'H', '♦': 'D', '♣': 'C'}.get(e[-1])
        return ('keycard', s)
    if e.startswith('Blackwood'):
        return ('aces', None)
    if 'uantitative' in e or 'No fit' in e or 'invites 6NT' in e:
        return ('quant', None)
    if e == '' or e == '(no rule)':
        return ('none', None)
    return ('other', e)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('run', nargs='?')
    ap.add_argument('--rows', help='summarise rows written by --out instead of running')
    ap.add_argument('--set', action='append', default=[])
    ap.add_argument('--rules')
    ap.add_argument('--out')
    ap.add_argument('--who', default='both', choices=['both', 'bba', 'ours'])
    ap.add_argument('--examples', action='store_true')
    a = ap.parse_args()
    if a.rows:
        out = json.load(open(a.rows))
    else:
        out = survey(a)
        if a.out:
            json.dump(out, open(a.out, 'w'))
    for who in ('bba', 'ours'):
        if a.who in (who, 'both'):
            print(f'== {who}')
            summarize(out, who, a.examples)


def survey(a):
    jobs = []
    for b in json.load(open(a.run))['boards']:
        for who, key in (('bba', 'reference'), ('ours', 'ours')):
            if a.who not in ('both', who):
                continue
            cs = [call(c) for c in b.get(key) or []]
            jobs += [(b, who, cs, i) for i, c in enumerate(cs) if c == '4N']
    print(f'{len(jobs)} 4NT calls', file=sys.stderr)

    def work(job):
        b, who, cs, i = job
        return job, explain(b['ns_card'], b['ew_card'], b['dealer'], cs[:i + 1], a.set, a.rules)

    out = []
    with ThreadPoolExecutor(3) as ex:
        for (b, who, cs, i), rows in ex.map(work, jobs):
            if not rows or len(rows) <= i:
                continue
            d0 = SEATS.index(b['dealer'][0])
            seat_of = lambda j: (d0 + j) % 4
            step = rows[i]['step']
            side = seat_of(i) % 2
            prev = rows[i - 1]['sides'][side] if i else {}
            try:
                r = rick(cs, i, rows, side, seat_of)
            except Exception as e:  # a malformed row: keep the call, unclassed
                r = ('error', None, str(e))
            out.append(dict(scenario=b['scenario'], board=b['board'], who=who, auction=' '.join(cs[:i + 1]),
                            rick=r, engine=engine_class(step.get('explanation')), expl=step.get('explanation'),
                            rule=step.get('rule'), ask_before=prev.get('ask'), ns=b['ns_card']))
    return out


SYM = {'♠': 'S', '♥': 'H', '♦': 'D', '♣': 'C'}


def summarize(rows, who, examples):
    """Rick's default (left) against our reading (after the arrow)."""
    m = collections.Counter()
    exs = collections.defaultdict(list)
    for r in rows:
        if r['who'] != who or r['ask_before']:
            continue
        k, s, why = r['rick']
        comp = 'they bid' in (why or '')
        if k == 'keycard':
            rc = f"keycard: {(why or '').replace(' (they bid)', '')}"
        else:
            rc = f'{k}: {why}' if k == 'skip' else k
        e = r['engine'][0]
        if e == 'keycard':
            es = next((SYM[c] for c in r['expl'] or '' if c in SYM), None)
            ec = 'keycard, same suit' if (k == 'keycard' and es == s) else 'keycard, other suit'
        else:
            ec = e
        key = ('competitive' if comp else 'uncontested', rc, ec)
        m[key] += 1
        if len(exs[key]) < 3:
            exs[key].append(f"{r['scenario']}#{r['board']}: {r['auction']} [{r['expl']}]")
    for k, n in sorted(m.items()):
        print(f'{n:6}  {k[0]:12} {k[1]:36} -> {k[2]}')
        if examples:
            for e in exs[k]:
                print('            ', e)


if __name__ == '__main__':
    main()
