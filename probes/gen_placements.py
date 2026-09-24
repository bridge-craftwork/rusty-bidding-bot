#!/usr/bin/env python3
"""Write a grid spec placing a fixed set of honours in every arrangement
across suits of a fixed shape, low spots filling the rest: which
combinations does BBA value?

    probes/gen_placements.py OUT.toml --shape 3 3 4 3 --cards A Q J T T \
        --prefix "1NT Pass" --partner-auto 15 17
"""
import argparse, itertools

ap = argparse.ArgumentParser()
ap.add_argument("out")
ap.add_argument("--shape", type=int, nargs=4, default=[3, 3, 4, 3])
ap.add_argument("--cards", nargs="+", default=["A", "Q", "J", "T", "T"])
ap.add_argument("--prefix", default="1NT Pass")
ap.add_argument("--dealer", default="S")
ap.add_argument("--card", default="Basic-Bridge")
ap.add_argument("--scoring", nargs="+", default=["MP"])
ap.add_argument("--partner-auto", type=int, nargs=2, default=[15, 17])
a = ap.parse_args()

RANKS = "AKQJT98765432"
SPOTS = "765432"   # never 9 or 8, so only the placed cards vary
hands, seen = [], set()
for place in itertools.product(range(4), repeat=len(a.cards)):
    suits = [[] for _ in range(4)]
    ok = True
    for card, s in zip(a.cards, place):
        if card in suits[s]:
            ok = False  # the same rank twice in one suit
            break
        suits[s].append(card)
    if not ok or any(len(suits[s]) > a.shape[s] for s in range(4)):
        continue
    for s in range(4):
        suits[s] += list(SPOTS[: a.shape[s] - len(suits[s])])
    h = ".".join("".join(sorted(x, key=RANKS.index)) for x in suits)
    if h not in seen:
        seen.add(h)
        hands.append(h)

q = lambda xs: "[" + ", ".join(f'"{x}"' for x in xs) + "]"
with open(a.out, "w") as f:
    f.write(f"# probes/gen_placements.py: {' '.join(a.cards)} in every arrangement, shape "
            f"{'-'.join(map(str, a.shape))}: {len(hands)} hands\n")
    f.write(f'card    = "{a.card}"\ndealer  = "{a.dealer}"\nprefix  = "{a.prefix}"\n')
    f.write(f"scoring = {q(a.scoring)}\npartner_auto = {a.partner_auto}\n")
    f.write('our     = ["general.style=bba"]\n')
    f.write("hands = [\n" + "".join(f'  "{h}",\n' for h in hands) + "]\n")
print(f"{a.out}: {len(hands)} hands")
