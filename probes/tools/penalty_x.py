#!/usr/bin/env python3
"""Judge penalty doubles between two `rbb compare --json` runs.

    probes/tools/penalty_x.py BASE.json VARIANT.json [RULE_SUBSTRING]

For every board where the variant's own auction has a double chosen by a
rule whose text contains RULE_SUBSTRING (default "penalty-doubles.bid"),
against the base run's auction on the same board (double dummy, no
runout beyond what the engine bid):

- side: IMPs to the doubler's side, variant against base;
- par: the change in distance from par (positive: closer);
- below par: the change in the doubler's side's shortfall from par
  only, counting nothing when it ends above par.

Distance from par counts a double that collects more than par (the
opponents overbid and we punish it) as a loss; "below par" does not.
The boards are split by whether the doubler's side ended above par.
(penalty-doubles.notes.md, 2026-10-01.)
"""
import collections
import json
import sys

ST = 'CDHSN'
T = [20, 50, 90, 130, 170, 220, 270, 320, 370, 430, 500, 600, 750, 900, 1100, 1300, 1500,
     1750, 2000, 2250, 2500, 3000, 3500, 4000]


def imp(x):
    n = sum(1 for v in T if abs(x) >= v)
    return n if x >= 0 else -n


def s(c):
    if isinstance(c, str):
        return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}[c]
    x = c['Bid']
    return f"{x['level']}{x['strain'][0] if x['strain'] != 'NoTrump' else 'N'}"


def score(level, strain, dbl, tricks, vul):
    need = level + 6
    if tricks >= need:
        per = 20 if strain in 'CD' else 30
        ts = (per * level + (10 if strain == 'N' else 0)) * {0: 1, 1: 2, 2: 4}[dbl]
        r = ts + ((500 if vul else 300) if ts >= 100 else 50)
        r += (750 if vul else 500) if level == 6 else (1500 if vul else 1000) if level == 7 else 0
        r += 50 * dbl
        over = tricks - need
        return r + (over * per if dbl == 0 else over * (200 if vul else 100) * dbl)
    down = need - tricks
    if dbl == 0:
        return -down * (100 if vul else 50)
    pen = 200 + 300 * (down - 1) if vul else 100 + 200 * min(down - 1, 2) + 300 * max(down - 3, 0)
    return -pen * dbl


def ns_score(b, calls):
    dl = 'NESW'.index(b['dealer'][0])
    seat = lambda i: 'NESW'[(dl + i) % 4]
    last, dbl = None, 0
    for i, c in enumerate(calls):
        if c[0].isdigit():
            last, dbl = i, 0
        elif c in ('X', 'XX'):
            dbl = len(c)
    if last is None:
        return 0
    lv, st = int(calls[last][0]), calls[last][1]
    ns = seat(last) in 'NS'
    decl = next(seat(j) for j, c in enumerate(calls) if c[0].isdigit() and c[1] == st and (seat(j) in 'NS') == ns)
    tr = b['dd']['tricks']['NESW'.index(decl)][ST.index(st)]
    vul = decl in {'None': '', 'NorthSouth': 'NS', 'EastWest': 'EW', 'Both': 'NSEW', 'All': 'NSEW'}.get(b['vul'], '')
    r = score(lv, st, dbl, tr, vul)
    return r if decl in 'NS' else -r


def main():
    base = {(b['scenario'], b['board']): b for b in json.load(open(sys.argv[1]))['boards']}
    var = json.load(open(sys.argv[2]))['boards']
    pat = sys.argv[3] if len(sys.argv) > 3 else 'penalty-doubles.bid'
    g = collections.defaultdict(lambda: [0, 0, 0, 0])
    for v in var:
        b = base.get((v['scenario'], v['board']))
        if not b or not v.get('dd'):
            continue
        calls = [s(c) for c in v['ours']]
        rules = v.get('our_rules') or []
        i = next((j for j, (c, r) in enumerate(zip(calls, rules)) if c == 'X' and r and pat in r), None)
        par = ((v.get('par') or b.get('par')) or {}).get('par_ns')
        if i is None or par is None:
            continue
        sg = 1 if 'NESW'[('NESW'.index(v['dealer'][0]) + i) % 4] in 'NS' else -1
        sv = ns_score(v, calls)
        sb = ns_score(b, [s(c) for c in b['ours']])
        short = lambda x: imp(max(0, sg * (par - x)))
        k = 'doubler above par' if sg * (sv - par) > 0 else 'doubler at or below par'
        g[k][0] += 1
        g[k][1] += imp(sg * (sv - sb))
        g[k][2] += imp(abs(sb - par)) - imp(abs(sv - par))
        g[k][3] += short(sb) - short(sv)
    print(f"{'':28} {'boards':>7} {'side':>7} {'par':>7} {'below par':>10}")
    for k in sorted(g):
        n, a, p, q = g[k]
        print(f'{k:28} {n:7} {a:+7} {p:+7} {q:+10}')
    t = [sum(x[j] for x in g.values()) for j in range(4)]
    print(f"{'total':28} {t[0]:7} {t[1]:+7} {t[2]:+7} {t[3]:+10}")


if __name__ == '__main__':
    main()
