#!/usr/bin/env python3
"""How each auction misses double-dummy par, and whose call it was.

    probes/tools/par_blame.py COMPARE.json [--rules N] [--shapes N] [--which ours|bba|both]
                                             [--category NAME] [--examples N]

Rick, 2026-09-27: the par yardstick only averages the distance from
par. This says *how* a board misses it. The side entitled to par is the
"winners" (W), the other the "losers" (L).

- above par: the auction went past the par contract. The first bid
  (not a double) above it is the error: by W, W overbid; by L, L
  overcompeted or oversacrificed.
- below par, W declaring, W worse off: W stopped short (a lower level)
  or chose a worse strain.
- below par, L declaring, W worse off: W let them play (did not bid on
  or double), or were outbid.
- W better off than par without going above it: L erred, L did not
  compete or sacrifice where par was their contract, or L went down
  more than the par sacrifice.

The blamed call is the first bid above par, or else the last call of
the side that should have acted. For our auctions its rule comes from
`our_rules` in the compare JSON. The cost is the IMP distance from par.
Not actionable board by board, but the running totals point at missing
or wrong bidding logic.
"""
import argparse
import collections
import json
import re

ST = 'CDHSN'


def s(c):
    if isinstance(c, str):
        return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}.get(c, c)
    x = c['Bid']
    return f"{x['level']}{x['strain'][0] if x['strain'] != 'NoTrump' else 'N'}"


def imps(d):
    t = [20, 50, 90, 130, 170, 220, 270, 320, 370, 430, 500, 600, 750, 900, 1100, 1300, 1500,
         1750, 2000, 2250, 2500, 3000, 3500, 4000]
    return sum(1 for v in t if abs(d) >= v)


def score(level, strain, dbl, tricks, vul):
    """Declarer's duplicate score."""
    need = level + 6
    if tricks >= need:
        per = 20 if strain in 'CD' else 30
        mult = {0: 1, 1: 2, 2: 4}[dbl]
        trick_score = (per * level + (10 if strain == 'N' else 0)) * mult
        s_ = trick_score
        s_ += (500 if vul else 300) if trick_score >= 100 else 50
        if level == 6:
            s_ += 750 if vul else 500
        if level == 7:
            s_ += 1500 if vul else 1000
        if dbl:
            s_ += 50 * dbl
        over = tricks - need
        if dbl == 0:
            s_ += over * per
        else:
            s_ += over * (200 if vul else 100) * dbl
        return s_
    down = need - tricks
    if dbl == 0:
        return -down * (100 if vul else 50)
    if vul:
        pen = 200 + 300 * (down - 1)
    else:
        pen = 100 + 200 * min(down - 1, 2) + 300 * max(down - 3, 0)
    return -pen * dbl


def rank(level, strain):
    return level * 5 + ST.index(strain)


def final(calls, dealer):
    """(level, strain, dbl, declarer seat, index of the last bid) or None."""
    dl = 'NESW'.index(dealer[0])
    seat = lambda i: 'NESW'[(dl + i) % 4]
    last = None
    dbl = 0
    for i, c in enumerate(calls):
        if c[0].isdigit():
            last = (i, int(c[0]), c[1], seat(i))
            dbl = 0
        elif c == 'X':
            dbl = 1
        elif c == 'XX':
            dbl = 2
    if not last:
        return None
    i, lv, st, by = last
    side = 'NS' if by in 'NS' else 'EW'
    for j, c in enumerate(calls):
        if c[0].isdigit() and c[1] == st and (seat(j) in 'NS') == (side == 'NS'):
            return lv, st, dbl, seat(j), i
    return None


def vul_of(vul, seat):
    v = {'None': '', 'NorthSouth': 'NS', 'EastWest': 'EW', 'Both': 'NSEW', 'All': 'NSEW'}.get(vul, '')
    return seat in v


def analyse(b, calls, rules):
    dd = b['dd']['tricks']
    p = b['par']
    P = p['par_ns']
    dl = 'NESW'.index(b['dealer'][0])
    seat = lambda i: 'NESW'[(dl + i) % 4]
    side_of = lambda x: 'NS' if x in 'NS' else 'EW'
    f = final(calls, b['dealer'])
    if f:
        lv, st, dbl, decl, lasti = f
        tricks = dd['NESW'.index(decl)][ST.index(st)]
        sc = score(lv, st, dbl, tricks, vul_of(b['vul'], decl))
        R = sc if decl in 'NS' else -sc
    else:
        R = 0
    if R == P:
        return None
    W = 'NS' if P > 0 else 'EW' if P < 0 else None
    if W is None:  # par is a pass-out
        W = side_of(f[3]) if f else 'NS'
        W = 'EW' if W == 'NS' else 'NS'  # whoever bid should not have
    L = 'EW' if W == 'NS' else 'NS'
    sign = 1 if W == 'NS' else -1
    dev = sign * (R - P)  # >0: W did better than par (L erred)
    # The par contract, from the text: "NS 4S=; NS 5CX-2".
    m = re.match(r'\s*(NS|EW)\s+(\d)([CDHSN])', p['par_contract'] or '')
    par_rank = rank(int(m.group(2)), m.group(3)) if m else None
    first_above = None
    if par_rank is not None:
        for i, c in enumerate(calls):
            if c[0].isdigit() and rank(int(c[0]), c[1]) > par_rank:
                first_above = i
                break
    cost = imps(abs(R - P))

    def last_call(side):
        for i in range(len(calls) - 1, -1, -1):
            if side_of(seat(i)) == side:
                return i  # the call that let the contract stand
        return None

    if first_above is not None:
        who = side_of(seat(first_above))
        cat = 'above par: W overbid' if who == W else 'above par: L overcompeted'
        blame = first_above
        later = sum(1 for c in calls[first_above + 1:] if c[0].isdigit())
    elif not f:
        cat = 'passed out: W never bid'
        blame = last_call(W)
        later = 0
    else:
        decl_side = side_of(f[3])
        later = 0
        if dev < 0:
            if decl_side == W:
                if par_rank is not None and rank(f[0], f[1]) < par_rank:
                    pl = int(m.group(2))
                    game = lambda lv, st: lv >= 6 or (st == 'N' and lv >= 3) or (st in 'HS' and lv >= 4) or lv >= 5
                    if pl >= 6 and f[0] < 6:
                        cat = 'below par: W stopped short of slam'
                    elif game(pl, m.group(3)) and not game(f[0], f[1]):
                        cat = 'below par: W stopped short of game'
                    else:
                        cat = 'below par: W stopped short (level)'
                else:
                    cat = 'below par: W wrong strain'
            elif sign * R > 0:
                cat = 'below par: W let them play (no double)' if f[2] == 0 else 'below par: W doubled, par was to bid on'
            else:
                cat = 'below par: W outbid, did not compete'
            blame = last_call(W)
        else:
            if decl_side == W:
                cat = 'L erred: did not compete or sacrifice'
                blame = last_call(L)
            else:
                cat = 'L erred: went down more than par'
                blame = f[4]
    rule = rules[blame] if rules and blame is not None and blame < len(rules) else None
    return {
        'cat': cat, 'cost': cost, 'blame': blame,
        'side': side_of(seat(blame)) if blame is not None else None,
        'role': (('W' if side_of(seat(blame)) == W else 'L') if blame is not None else None),
        'rule': rule, 'later': later,
        'shape': ' '.join(calls[:blame + 1]) if blame is not None else '',
    }


def shape(calls):
    m = {}
    out = []
    for c in calls.split():
        if c[0].isdigit() and c[1] in 'CDHS':
            if c[1] not in m:
                m[c[1]] = 'abcd'[len(m)]
            out.append(c[0] + m[c[1]])
        else:
            out.append(c)
    while out and out[0] == 'P':
        out.pop(0)
    return ' '.join(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('file')
    ap.add_argument('--rules', type=int, default=12)
    ap.add_argument('--shapes', type=int, default=0)
    ap.add_argument('--category')
    ap.add_argument('--examples', type=int, default=0)
    a = ap.parse_args()
    boards = [b for b in json.load(open(a.file))['boards'] if b.get('dd') and b.get('par')]
    tot = {'ours': collections.Counter(), 'bba': collections.Counter()}
    cst = {'ours': collections.Counter(), 'bba': collections.Counter()}
    by_rule = collections.defaultdict(lambda: [0, 0, ''])
    by_shape = collections.defaultdict(lambda: [0, 0])
    later = collections.Counter()
    ex = []
    for b in boards:
        for which, calls, rules in (('ours', [s(c) for c in b['ours']], b.get('our_rules')),
                                    ('bba', [s(c) for c in b['reference']], None)):
            r = analyse(b, calls, rules)
            key = r['cat'] if r else 'at par'
            tot[which][key] += 1
            cst[which][key] += r['cost'] if r else 0
            if which == 'ours' and r:
                if a.category and a.category not in r['cat']:
                    continue
                k = (r['cat'], (r['rule'] or 'no rule').split(' ', 1)[0])
                by_rule[k][0] += 1
                by_rule[k][1] += r['cost']
                by_rule[k][2] = (r['rule'] or '').split(' ', 1)[1] if r['rule'] and ' ' in r['rule'] else ''
                sk = (r['cat'], shape(r['shape']))
                by_shape[sk][0] += 1
                by_shape[sk][1] += r['cost']
                later[r['cat']] += r['later']
                if len(ex) < a.examples:
                    ex.append((b['scenario'], b['board'], ' '.join(calls), r['shape'], r['rule'], r['cost']))
    n = len(boards)
    print(f'{n} boards with a double-dummy table and par\n')
    print(f"{'how the table missed par':44} {'ours':>7} {'IMPs':>7} {'BBA':>7} {'IMPs':>7}")
    cats = sorted(set(tot['ours']) | set(tot['bba']), key=lambda k: -cst['ours'][k])
    for k in cats:
        print(f"{k:44} {tot['ours'][k]:7} {cst['ours'][k]:7} {tot['bba'][k]:7} {cst['bba'][k]:7}")
    print(f"{'total':44} {sum(tot['ours'].values()):7} {sum(cst['ours'].values()):7} "
          f"{sum(tot['bba'].values()):7} {sum(cst['bba'].values()):7}")
    print('\nour blamed calls by rule (IMPs from par):')
    for (cat, rule), (cnt, cost, expl) in sorted(by_rule.items(), key=lambda z: -z[1][1])[:a.rules]:
        print(f'  {cost:6} {cnt:5}  {cat:40} {rule}  {expl[:60]}')
    if a.shapes:
        print('\nour blamed calls by auction (the blamed call last):')
        for (cat, sh), (cnt, cost) in sorted(by_shape.items(), key=lambda z: -z[1][1])[:a.shapes]:
            print(f'  {cost:6} {cnt:5}  {cat:40} {sh}')
    for e in ex:
        print('  e.g.', e)


if __name__ == '__main__':
    main()
