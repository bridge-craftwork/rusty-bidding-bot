#!/usr/bin/env python3
"""Where the doubling errors are, by the auction around the missed double.

    probes/tools/double_context.py RUN.json [--by context|relation|shape] [--top N]

Reads one `rbb compare --json` run (penalty-doubles.notes.md,
2026-10-09). The defenders are the side that did not bid the final
contract; their doubling error is `par.*_errors.doubling` for that side.

- context: our defenders' doubling errors and BBA's, each on its own
  auctions, by what happened before the final contract (they bid alone,
  we opened and they overcalled, they opened and we competed, ...),
  with chances (a contract above par that went down) and doubles;
- relation: the doubling gap (ours minus BBA's, both sides) on each
  board, by how our final contract compares with BBA's (same contract,
  the same side higher or lower, the other side declaring);
- shape: the auction up to the final bid with suits renamed x, y, z in
  order of appearance, ours against BBA's, competitive auctions only,
  largest gap first.
"""
import argparse
import collections
import json

ST = {'Clubs': 'C', 'Diamonds': 'D', 'Hearts': 'H', 'Spades': 'S', 'NoTrump': 'N'}


def call(c):
    if isinstance(c, str):
        return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}[c]
    b = c['Bid']
    return f"{b['level']}{ST[b['strain']]}"


def is_game(c):
    lv, st = int(c[0]), c[1]
    return (st == 'N' and lv >= 3) or (st in 'HS' and lv >= 4) or lv >= 5


def context(b, calls):
    """(context, defenders' index in [NS, EW], declaring side) or None."""
    dl = 'NESW'.index(b['dealer'][0])
    seat = lambda i: 'NESW'[(dl + i) % 4]
    bids = [i for i, c in enumerate(calls) if c[0].isdigit()]
    if not bids:
        return None
    last = bids[-1]
    decl_ns = seat(last) in 'NS'
    defs = lambda s: (s in 'NS') != decl_ns
    dcalls = [(i, calls[i]) for i in range(last + 1) if defs(seat(i)) and calls[i] != 'P']
    acalls = [(i, calls[i]) for i in range(last + 1) if not defs(seat(i)) and calls[i] != 'P']
    abids = [c for _, c in acalls if c[0].isdigit()]
    dbids = [c for _, c in dcalls if c[0].isdigit()]
    side = 1 if decl_ns else 0
    if not dcalls:
        return 'they bid alone (we never bid)', side
    if any(is_game(c) for c in dbids):
        return 'they bid over our game', side
    if defs(seat(bids[0])):
        first = acalls[0]
        pre = [c for c in calls[:first[0]] if c != 'P']
        if first[1] == 'X':
            return 'we opened, their takeout double, then their bid', side
        if len(pre) >= 2:
            return 'we opened and responded, their (sandwich) overcall', side
        if len(abids) == 1 and calls[last] == abids[0]:
            return 'we opened, their overcall passed out', side
        return 'we opened, their overcall, then more bidding', side
    if not dbids:
        return 'they opened, we only doubled', side
    if [c[1] for c in abids].count(calls[last][1]) >= 2:
        return 'they opened, we bid, they compete in their fit', side
    return 'they opened, we bid, no fit for them', side


def shape(calls):
    i = 0
    while i < len(calls) and calls[i] == 'P':
        i += 1
    last = max(j for j, c in enumerate(calls) if c[0].isdigit())
    out, m = [], {}
    for c in calls[i:last + 1]:
        if c[0].isdigit() and c[1] != 'N':
            if c[1] not in m:
                m[c[1]] = 'xyzw'[len(m)]
            out.append(c[0] + m[c[1]])
        else:
            out.append(c)
    return ' '.join(out)


def key(c):
    c = c or 'PASS'
    return c.split()[0].rstrip('X'), c.split()[-1]


def rank(c):
    s = (c or 'PASS').split()[0].rstrip('X')
    return 0 if s == 'PASS' else int(s[0]) * 5 + 'CDHSN'.index(s[1])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('run')
    ap.add_argument('--by', default='context', choices=['context', 'relation', 'shape'])
    ap.add_argument('--top', type=int, default=30)
    a = ap.parse_args()
    G = collections.defaultdict(collections.Counter)
    for b in json.load(open(a.run))['boards']:
        p = b.get('par') or {}
        if a.by == 'relation':
            if not p.get('ours_errors'):
                continue
            od, rd = sum(p['ours_errors']['doubling']), sum(p['reference_errors']['doubling'])
            if od == rd:
                continue
            o, r = b['our_contract'], b['reference_contract']
            oc = context(b, [call(c) for c in b['ours']])
            rc = context(b, [call(c) for c in b['reference']])
            if key(o) == key(r):
                k = 'the same contract, doubled or not'
            elif oc is None or rc is None:
                k = 'passed out (ours or BBA\'s)'
            else:
                same = oc[1] == rc[1]
                hi = rank(o) > rank(r)
                k = ('same declaring side' if same else 'the other side declares') + (', ours higher' if hi else ', ours lower')
            G[k]['n'] += 1
            G[k]['gap'] += od - rd
            continue
        for eng, w in (('ours', 'o'), ('reference', 'b')):
            cls, errs = p.get(f'{eng}_class'), p.get(f'{eng}_errors')
            if not cls or not errs:
                continue
            calls = [call(c) for c in b[eng]]
            c = context(b, calls)
            if not c:
                continue
            ch = cls.get('double_chance') is not None
            if not ch and not cls.get('bad_double') and not errs['doubling'][c[1]]:
                continue
            if a.by == 'shape':
                if c[0].startswith('they bid alone'):
                    continue
                k = shape(calls)
            else:
                k = c[0]
            g = G[k]
            g[w + 'err'] += errs['doubling'][c[1]]
            g[w + 'ch'] += ch
            g[w + 'x'] += bool(cls.get('double_chance'))
            g[w + 'made'] += bool(cls.get('bad_double'))
    if a.by == 'relation':
        print(f"{'boards':>7} {'gap':>8}  our contract against BBA's (doubling errors, ours minus BBA's)")
        for k in sorted(G, key=lambda k: -G[k]['gap']):
            print(f"{G[k]['n']:7} {G[k]['gap']:+8}  {k}")
        print(f"{sum(g['n'] for g in G.values()):7} {sum(g['gap'] for g in G.values()):+8}  total")
        return
    print(f"{'gap':>7} | {'ours':>7} {'ch':>6} {'x':>5} {'made':>5} | {'BBA':>7} {'ch':>6} {'x':>5} {'made':>5} | {a.by}")
    T = collections.Counter()
    for k in sorted(G, key=lambda k: -(G[k]['oerr'] - G[k]['berr']))[:a.top]:
        g = G[k]
        print(f"{g['oerr'] - g['berr']:7} | {g['oerr']:7} {g['och']:6} {g['ox']:5} {g['omade']:5} | "
              f"{g['berr']:7} {g['bch']:6} {g['bx']:5} {g['bmade']:5} | {k}")
    for g in G.values():
        T.update(g)
    g = T
    print(f"{g['oerr'] - g['berr']:7} | {g['oerr']:7} {g['och']:6} {g['ox']:5} {g['omade']:5} | "
          f"{g['berr']:7} {g['bch']:6} {g['bx']:5} {g['bmade']:5} | total")


if __name__ == '__main__':
    main()
