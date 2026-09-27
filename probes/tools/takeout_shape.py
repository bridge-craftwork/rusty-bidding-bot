#!/usr/bin/env python3
"""How long is a takeout doubler in the unbid suits? From BBA's auctions.

    probes/tools/takeout_shape.py COMPARE.json

Takes every BBA auction that starts (after leading passes) 1x X, the
direct double, or 1x P P X, the balancing double, where the doubler
did not overcall first. For each unbid suit, tallies the doubler's
length, split by the suit opened (minor/major) and the unbid suit
(the other major / a major / a minor), and by HCP (up to 17 vs 18+).
"""
import json, sys, collections

def s(c):
    if isinstance(c, str):
        return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}.get(c, c)
    x = c['Bid']; return f"{x['level']}{x['strain'][0] if x['strain'] != 'NoTrump' else 'N'}"

HCP = {'A': 4, 'K': 3, 'Q': 2, 'J': 1}
SU = 'SHDC'
tab = collections.defaultdict(collections.Counter)
n = collections.Counter()
for b in json.load(open(sys.argv[1]))['boards']:
    r = [s(c) for c in b['reference']]
    i = 0
    while i < len(r) and r[i] == 'P':
        i += 1
    if i >= len(r) or len(r[i]) != 2 or r[i][0] != '1' or r[i][1] not in SU:
        continue
    opened = r[i][1]
    if len(r) > i + 1 and r[i + 1] == 'X':
        j, kind = i + 1, 'direct'
    elif len(r) > i + 3 and r[i + 1:i + 3] == ['P', 'P'] and r[i + 3] == 'X':
        j, kind = i + 3, 'balancing'
    else:
        continue
    dl = 'NESW'.index(b['dealer'][0])
    seat = 'NESW'[(dl + j) % 4]
    hand = dict(zip('NESW', b['deal'][2:].split()))[seat].split('.')
    h = sum(HCP.get(c, 0) for c in ''.join(hand))
    band = '<=17' if h <= 17 else '18+'
    omaj = opened in 'SH'
    for k, suit in enumerate(SU):
        if suit == opened:
            continue
        L = len(hand[k])
        if omaj and suit in 'SH':
            what = 'other major'
        elif suit in 'SH':
            what = 'major (minor opened)'
        else:
            what = 'minor (' + ('major' if omaj else 'minor') + ' opened)'
        key = (kind, what, band)
        tab[key][min(L, 6)] += 1
        n[key] += 1
    n[(kind, 'hands', band)] += 1

print('doublers:', {k: v for k, v in n.items() if k[1] == 'hands'})
print(f"{'seat':10} {'unbid suit':26} {'hcp':5} {'n':>6}   " + '  '.join(f'{L}{"+" if L == 6 else ""}:' + ' ' * 3 for L in range(7)) + '  >=4')
for key in sorted(tab):
    c = tab[key]; t = n[key]
    row = '  '.join(f'{100 * c[L] / t:5.1f}' for L in range(7))
    ge4 = sum(c[L] for L in range(4, 7)) / t * 100
    print(f'{key[0]:10} {key[1]:26} {key[2]:5} {t:6}   {row}  {ge4:5.1f}')
