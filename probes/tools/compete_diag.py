#!/usr/bin/env python3
"""Where a met-par class comes from (2026-10-02, the competing diagnosis).

    probes/tools/compete_diag.py COMPARE.json [--class did_not_compete] [--top N]

For the boards of one "how each table met par" class (`par.ours_class`
and `par.reference_class` in a `rbb compare --json`), for our table and
BBA's:

1. boards and the table's errors (IMPs) by whether the side charged with
   the errors ever bid (or only doubled, or never came in), who opened,
   and the level of the opponents' last bid;
2. for our boards: the first call where our auction left BBA's, by stage
   (the opening; our first entry over their opening; their first entry
   over ours; both sides in), with our errors minus BBA's on those
   boards (positive: BBA's auction did better);
3. the decisive passes: our passes by the charged side after the
   opponents' last bid, whether BBA's auction reached the same position,
   and what BBA called there, by the rule that passed.

The side charged is the one with the larger errors on our table.
"""
import argparse
import collections
import json
import re

SEATS = 'NESW'


def call(c):
    if isinstance(c, str):
        return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}.get(c, c)
    b = c['Bid']
    return f"{b['level']}{b['strain'][0] if b['strain'] != 'NoTrump' else 'N'}"


def tot(e):
    return sum(e['contract']) + sum(e['doubling'])


def side(seat):
    return 'NS' if seat in 'NS' else 'EW'


def charged(errors):
    c = [errors['contract'][i] + errors['doubling'][i] for i in (0, 1)]
    return 'NS' if c[0] >= c[1] else 'EW'


def state(b, auction, who):
    p = b['par']
    ps = charged(p[f'{who}_errors'])
    seats = [SEATS[(SEATS.index(b['dealer'][0]) + i) % 4] for i in range(len(auction))]
    bids = [i for i, c in enumerate(auction) if c not in ('P', 'X', 'XX')]
    if not bids:
        return None
    li = bids[-1]
    opener = 'charged side opened' if side(seats[bids[0]]) == ps else 'they opened'
    if any(side(seats[i]) == ps for i in bids):
        came = 'bid'
    elif any(side(seats[i]) == ps and c != 'P' for i, c in enumerate(auction)):
        came = 'doubled only'
    else:
        came = 'never came in'
    return ps, seats, li, opener, came, auction[li][0]


def stage(prefix):
    """prefix from the caller's view: own side's calls bare, theirs in ()."""
    ours = any(not t.startswith('(') and t != 'P' for t in prefix)
    theirs = any(t.startswith('(') and t != '(P)' for t in prefix)
    if not ours and not theirs:
        return 'the opening'
    if theirs and not ours:
        return 'our first entry over their opening'
    if ours and not theirs:
        return 'their entry / uncontested'
    return 'both sides in'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('json')
    ap.add_argument('--class', dest='cls', default='did_not_compete')
    ap.add_argument('--top', type=int, default=12)
    a = ap.parse_args()
    boards = json.load(open(a.json))['boards']

    print(f'class {a.cls}: boards / errors of that table')
    for who in ('ours', 'reference'):
        t = collections.defaultdict(lambda: [0, 0])
        for b in boards:
            p = b.get('par')
            if not p or p[f'{who}_class']['class'] != a.cls:
                continue
            auc = [call(c) for c in (b['ours'] if who == 'ours' else b['reference'])]
            s = state(b, auc, who)
            if not s:
                continue
            e = tot(p[f'{who}_errors'])
            for k in ((s[4], ''), (s[4], s[3]), (s[4], f'their last bid at {s[5]}')):
                t[k][0] += 1
                t[k][1] += e
        print(f'  {"BBA" if who == "reference" else "ours"}:')
        for k, v in sorted(t.items()):
            print(f'    {k[0]:14} {k[1]:28} {v[0]:6} {v[1]:7}')

    st = collections.defaultdict(lambda: [0, 0])
    first = collections.defaultdict(lambda: [0, 0])
    reach = collections.Counter()
    byrule = collections.defaultdict(collections.Counter)
    for b in boards:
        p = b.get('par')
        if not p or p['ours_class']['class'] != a.cls:
            continue
        o = [call(c) for c in b['ours']]
        r = [call(c) for c in b['reference']]
        d = tot(p['ours_errors']) - tot(p['reference_errors'])
        fd = b.get('first_divergence')
        if fd is not None and fd < len(o) and fd < len(r):
            k = 0
            while k < fd and r[k] == 'P':
                k += 1
            m = {}
            toks = []
            for j in range(k, fd):
                c = r[j]
                if c[-1] in 'CDHS':
                    if c[-1] not in m:
                        m[c[-1]] = 'abcd'[len(m)]
                    c = c[:-1] + m[c[-1]]
                toks.append(c if (fd - j) % 2 == 0 else f'({c})')
            s = stage(toks)
            st[s][0] += 1
            st[s][1] += d
            ab = lambda c: c[:-1] + m.get(c[-1], 'e') if c[-1] in 'CDHS' else c
            if s == 'both sides in':
                f = first[(' '.join(toks), ab(o[fd]), ab(r[fd]))]
                f[0] += 1
                f[1] += d
        s = state(b, o, 'ours')
        if not s:
            continue
        ps, seats, li = s[0], s[1], s[2]
        passes = [i for i in range(li + 1, len(o)) if side(seats[i]) == ps]
        same = [i for i in passes if o[:i] == r[:i] and i < len(r)]
        reach['BBA reached a decisive pass' if same else 'BBA never reached it'] += 1
        rules = b.get('our_rules') or []
        for i in same:
            rule = re.sub(r'^conventions/', '', (rules[i] if i < len(rules) else None) or 'no rule')
            rc = r[i]
            byrule[rule]['P' if rc == 'P' else ('X' if rc in ('X', 'XX') else 'bid')] += 1

    print('\nour boards: the first call that left BBA\'s auction (our errors - BBA\'s)')
    for k, v in sorted(st.items(), key=lambda kv: -kv[1][1]):
        print(f'  {k:40} {v[0]:6} {v[1]:+7}')
    print('  both sides in, the largest (prefix from the caller, ours -> BBA):')
    for k, v in sorted(first.items(), key=lambda kv: -kv[1][1])[:a.top]:
        print(f'    {k[0]:36} {k[1]:>4} -> {k[2]:<4} {v[0]:5} {v[1]:+6}')
    print(f'\nour decisive passes: {dict(reach)}')
    for k, v in sorted(byrule.items(), key=lambda kv: -sum(kv[1].values()))[:a.top]:
        print(f'  {sum(v.values()):6}  BBA {dict(v)}  {k}')


if __name__ == '__main__':
    main()
