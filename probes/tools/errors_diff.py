#!/usr/bin/env python3
"""Compare two `rbb compare --json` runs by the errors yardstick (Rick,
2026-10-01): each side charged with its own errors, a contract that
goes down undoubled and beats par that way taken as doubled
(`par.ours_errors` / `par.reference_errors`, IMPs, `[NS, EW]`).

    probes/tools/errors_diff.py BASE.json VARIANT.json [--top N]

On every board where our auction changed: the change in our errors
(positive: the variant makes fewer), split by error type (contract,
doubling) and by side, the side that made the first differing call
("actor") against the other one. Beside it, the change in distance from
par and the double-dummy IMPs to the actor (sideimps.py), each with the
even / odd board halves; then the first differing calls (base ->
variant) that gained and lost most, and the whole-set doubling errors
against BBA's before and after.

The compare JSON has par only where our contract differs from BBA's;
where one run matched BBA, its errors are BBA's from the other run.
"""
import argparse
import collections
import json

IMP_STEPS = [20, 50, 90, 130, 170, 220, 270, 320, 370, 430, 500, 600, 750, 900,
             1100, 1300, 1500, 1750, 2000, 2250, 2500, 3000, 3500, 4000]


def imp(d):
    n = sum(1 for v in IMP_STEPS if abs(d) >= v)
    return n if d >= 0 else -n


def call(c):
    if isinstance(c, str):
        return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}.get(c, c)
    b = c['Bid']
    return f"{b['level']}{b['strain'][0]}"


def errors(x, y):
    """(errors {contract, doubling}, our NS score, par NS) for board x;
    BBA's from y where x matched BBA."""
    p = x.get('par') or {}
    if p.get('ours_errors') is not None:
        return p['ours_errors'], p.get('ours_ns'), p.get('par_ns')
    q = y.get('par') or {}
    if x['our_contract'] == x['reference_contract'] and q.get('reference_errors') is not None:
        return q['reference_errors'], q.get('reference_ns'), q.get('par_ns')
    return None, None, None


def whole(boards):
    """Whole-set totals against BBA: (contract, doubling), BBA's minus ours."""
    c = d = 0
    for x in boards:
        p = x.get('par') or {}
        if p.get('ours_errors') is None:
            continue
        r, o = p['reference_errors'], p['ours_errors']
        c += sum(r['contract']) - sum(o['contract'])
        d += sum(r['doubling']) - sum(o['doubling'])
    return c, d


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('base')
    ap.add_argument('variant')
    ap.add_argument('--top', type=int, default=6)
    a = ap.parse_args()
    base = json.load(open(a.base))['boards']
    var = json.load(open(a.variant))['boards']
    key = lambda x: (x['scenario'], x['board'])
    B = {key(x): x for x in base}

    # cells[name][half] = IMPs; half 0 even boards, 1 odd
    cells = collections.defaultdict(lambda: [0, 0])
    n = [0, 0]
    skipped = 0
    by = collections.Counter()
    cnt = collections.Counter()
    for v in var:
        b = B.get(key(v))
        if not b:
            continue
        ov = [call(c) for c in v['ours']]
        ob = [call(c) for c in b['ours']]
        if ov == ob:
            continue
        eb, sb, pb = errors(b, v)
        ev, sv, pv = errors(v, b)
        if eb is None or ev is None:
            skipped += 1
            continue
        try:
            h = int(v['board']) % 2
        except ValueError:
            h = 0
        n[h] += 1
        i = 0
        while i < min(len(ov), len(ob)) and ov[i] == ob[i]:
            i += 1
        seat = 'NESW'[('NESW'.index(v['dealer'][0]) + i) % 4]
        actor = 0 if seat in 'NS' else 1
        tot = 0
        for kind in ('contract', 'doubling'):
            for side in (0, 1):
                g = eb[kind][side] - ev[kind][side]
                who = 'actor' if side == actor else 'other'
                cells[f'{kind} {who}'][h] += g
                tot += g
        cells['errors'][h] += tot
        par = pv if pv is not None else pb
        if None not in (sb, sv, par):
            cells['distance from par'][h] += imp(abs(sb - par)) - imp(abs(sv - par))
            sg = 1 if actor == 0 else -1
            cells['side IMPs (actor)'][h] += imp(sg * (sv - sb))
        pre = ob[:i]
        while pre and pre[0] == 'P':
            pre.pop(0)
        k = f"{' '.join(pre[-3:])} : {ob[i] if i < len(ob) else '-'} -> {ov[i] if i < len(ov) else '-'}"
        by[k] += tot
        cnt[k] += 1

    N = n[0] + n[1]
    print(f'boards changed {N} (even {n[0]}, odd {n[1]}; skipped {skipped})')
    order = ['errors', 'contract actor', 'contract other', 'doubling actor',
             'doubling other', 'distance from par', 'side IMPs (actor)']
    for name in order:
        e, o = cells[name]
        per = (e + o) / N if N else 0
        print(f'  {name:20} {e + o:+7}  ({e:+} / {o:+})  {per:+.2f}/bd')
    c = cells['contract actor'][0] + cells['contract actor'][1] + cells['contract other'][0] + cells['contract other'][1]
    d = cells['doubling actor'][0] + cells['doubling actor'][1] + cells['doubling other'][0] + cells['doubling other'][1]
    print(f'  by type: contract {c:+}, doubling {d:+}')
    cb, db = whole(base)
    cv, dv = whole(var)
    print(f'whole set vs BBA: contract {cb:+} -> {cv:+}, doubling {db:+} -> {dv:+} '
          f'(net {cb + db:+} -> {cv + dv:+})')
    print('first differing calls (context : base -> variant), IMPs by errors:')
    for k, g in sorted(by.items(), key=lambda z: z[1])[:a.top]:
        print(f'  {g:+6} {cnt[k]:5}  {k}')
    for k, g in sorted(by.items(), key=lambda z: -z[1])[:a.top]:
        print(f'  {g:+6} {cnt[k]:5}  {k}')


if __name__ == '__main__':
    main()
