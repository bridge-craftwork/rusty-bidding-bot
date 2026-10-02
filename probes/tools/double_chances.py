#!/usr/bin/env python3
"""Where the doubling errors come from, in one `rbb compare --json` run.

    probes/tools/double_chances.py RUN.json [--by class|level|vul|rule]

Every penalty-double chance (`par.*_class.double_chance` is set: a
contract bid above par that goes down) and every double of a contract
that made (`bad_double`), for our auction and BBA's, grouped by the
position of the defenders:

- class: `silent` (the defenders never bid), `webid` (one or both
  defenders bid; with `(direct, balancing)` what each did: `bid`, `X`
  for doubles only, `-` for passes only, and `fit` when the defenders
  bid one suit twice), `sac` (they bid over our game);
  game or partscore;
- level: the contract's level and strain;
- vul: the defenders' and declarers' vulnerability;
- rule: for our doubles, the rule that doubled and the one that left
  it in (the doubler's partner's next call).

For each engine: chances, doubled, doubles of contracts that made and
what they cost, and the doubling errors (`par.*_errors.doubling` of the
defending side, IMPs); last column BBA's errors minus ours (negative:
we make more). The "direct" seat calls first after the final bid, the
"balancing" seat last. (penalty-doubles.notes.md, 2026-10-02.)
"""
import argparse
import collections
import json


def call(c):
    if isinstance(c, str):
        return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}[c]
    b = c['Bid']
    return f"{b['level']}{'N' if b['strain'] == 'NoTrump' else b['strain'][0]}"


def is_game(c):
    lv, st = int(c[0]), c[1]
    return (st == 'N' and lv >= 3) or (st in 'HS' and lv >= 4) or lv >= 5


def row(b, calls, cls, errs, rules):
    if not cls or (cls.get('double_chance') is None and not cls.get('bad_double')):
        return None
    dl = 'NESW'.index(b['dealer'][0])
    seat = lambda i: 'NESW'[(dl + i) % 4]
    bids = [i for i, c in enumerate(calls) if c[0].isdigit()]
    last = bids[-1]
    decl_ns = seat(last) in 'NS'
    d_seat, b_seat = seat(last + 1), seat(last + 3)
    ours = [(seat(i), calls[i]) for i in range(last + 1) if calls[i] != 'P' and (seat(i) in 'NS') != decl_ns]
    def did(s):
        cs = [c for q, c in ours if q == s]
        return '-' if not cs else ('bid' if any(c[0].isdigit() for c in cs) else 'X')
    suits = [c[1] for _, c in ours if c[0].isdigit() and c[1] != 'N']
    fit = any(suits.count(x) >= 2 for x in set(suits))
    sac = any(is_game(c) for _, c in ours if c[0].isdigit())
    kind = 'sac' if sac else ('webid' if any(c[0].isdigit() or c == 'X' for _, c in ours) else 'silent')
    side = 1 if decl_ns else 0      # the defenders' index in [NS, EW]
    after = calls[last + 1:]
    doubled = 'X' in after
    dbl_rule = None
    if doubled and rules:
        i = last + 1 + after.index('X')
        dbl_rule = (rules[i] or '').split('/')[-1][:60]
        nxt = (rules[i + 2] or '').split('/')[-1][:60] if i + 2 < len(rules) else ''
        dbl_rule = f'{dbl_rule}  <-  {nxt}'
    vul = b['vul']
    dv = {'None': False, 'Both': True, 'NorthSouth': not decl_ns, 'EastWest': decl_ns}.get(vul, False)
    tv = {'None': False, 'Both': True, 'NorthSouth': decl_ns, 'EastWest': not decl_ns}.get(vul, False)
    return dict(
        cls=(kind, '' if kind == 'silent' else (did(d_seat), did(b_seat)) + (('fit',) if fit else ()),
             'game' if is_game(calls[last]) else 'part'),
        level=calls[last][0] + ('N' if calls[last][1] == 'N' else 'M' if calls[last][1] in 'HS' else 'm'),
        vul=('def vul' if dv else 'def nv', 'decl vul' if tv else 'decl nv'),
        chance=cls.get('double_chance') is not None, doubled=bool(cls.get('double_chance')),
        bad=bool(cls.get('bad_double')), err=errs['doubling'][side] if errs else 0, rule=dbl_rule)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('run')
    ap.add_argument('--by', default='class', choices=['class', 'level', 'vul', 'rule'])
    a = ap.parse_args()
    G = collections.defaultdict(collections.Counter)
    for b in json.load(open(a.run))['boards']:
        p = b.get('par') or {}
        for w, ck, cls, errs, rules in (('ours', 'ours', p.get('ours_class'), p.get('ours_errors'), b.get('our_rules')),
                                        ('bba', 'reference', p.get('reference_class'), p.get('reference_errors'), None)):
            r = row(b, [call(c) for c in b[ck]], cls, errs, rules)
            if not r:
                continue
            if a.by == 'rule':
                if w != 'ours' or not r['rule']:
                    continue
                g = G[r['rule']]
                g['ours bad' if r['bad'] else 'ours x'] += 1
                g['ours err'] += r['err']
                continue
            g = G[r["cls" if a.by == "class" else a.by]]
            g[w + ' ch'] += r['chance']
            g[w + ' x'] += r['doubled']
            g[w + ' bad'] += r['bad']
            g[w + ' badimp'] += r['err'] if r['bad'] else 0
            g[w + ' err'] += r['err']
    if a.by == 'rule':
        print(f"{'taken':>6} {'made':>5} {'IMPs':>6}  doubled by  <-  left in by")
        for k in sorted(G, key=lambda k: -(G[k]['ours x'] + G[k]['ours bad'])):
            g = G[k]
            print(f"{g['ours x']:6} {g['ours bad']:5} {g['ours err']:6}  {k}")
        return
    print(f"{'':42} {'ours: chances':>13} {'x':>5} {'made':>5} {'cost':>5} {'errors':>7} | "
          f"{'BBA: chances':>12} {'x':>5} {'made':>5} {'cost':>5} {'errors':>7} | {'BBA-ours':>8}")
    T = collections.Counter()
    for k in sorted(G, key=lambda k: G[k]['bba err'] - G[k]['ours err']):
        g = G[k]
        T.update(g)
        f = lambda w: f"{g[w + ' ch']:13} {g[w + ' x']:5} {g[w + ' bad']:5} {g[w + ' badimp']:5} {g[w + ' err']:7}"
        print(f"{str(k)[:42]:42} {f('ours')} | {f('bba').lstrip():>12} | {g['bba err'] - g['ours err']:8}")
    g = T
    print(f"{'total':42} {g['ours ch']:13} {g['ours x']:5} {g['ours bad']:5} {g['ours badimp']:5} {g['ours err']:7} | "
          f"{g['bba ch']:12} {g['bba x']:5} {g['bba bad']:5} {g['bba badimp']:5} {g['bba err']:7} | {g['bba err'] - g['ours err']:8}")


if __name__ == '__main__':
    main()
