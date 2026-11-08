#!/usr/bin/env python3
"""Opener hands for standard Jacoby 2NT, for the survey of where BBA's
opener goes on over responder's 4M (jacoby-2nt.notes.md).

    probes/jacoby_opener_gen.py OUT.toml MAJOR N SEED [PREFIX]

N hands, 12-21 HCP evenly, five to seven of MAJOR (S or H), half with a
singleton or void; each hand carries its own responder (four trumps,
12-14 HCP, balanced: a 4M sign-off). The specs in probes/ were made with
    probes/jacoby_opener_gen.py probes/jacoby-opener-1S-answer.toml S 1000 11
    probes/jacoby_opener_gen.py probes/jacoby-opener-1H-answer.toml H 1000 12
then `rbb grid` on each, then probes/jacoby_opener_stage_b.py S and H.
"""
import random
import sys

RANKS = "AKQJT98765432"
HCP = {"A": 4, "K": 3, "Q": 2, "J": 1}
out, major, n, seed = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
prefix = sys.argv[5] if len(sys.argv) > 5 else f"1{major} Pass 2NT Pass"
mi = 0 if major == "S" else 1
rng = random.Random(seed)


def pbn(cards):
    return ".".join("".join(sorted((r for s2, r in cards if s2 == s), key=RANKS.index)) for s in range(4))


deck = [(s, r) for s in range(4) for r in RANKS]
hands = []
per = {h: 0 for h in range(12, 22)}
want = n // 10
while min(per.values()) < want:
    c = rng.sample(deck, 13)
    L = [sum(1 for s2, _ in c if s2 == s) for s in range(4)]
    h = sum(HCP.get(r, 0) for _, r in c)
    if h not in per or per[h] >= want:
        continue
    if L[mi] < 5 or L[mi] > 7 or max(L[s] for s in range(4) if s != mi) > 5:
        continue
    if mi == 1 and L[0] >= 5:
        continue
    # half the hands with a singleton or void (any suit but M)
    short = min(L[s] for s in range(4) if s != mi) <= 1
    if (per[h] % 2 == 0) != short:
        continue
    rest = [x for x in deck if x not in c]
    for _ in range(20000):
        q = rng.sample(rest, 13)
        Lq = [sum(1 for s2, _ in q if s2 == s) for s in range(4)]
        hq = sum(HCP.get(r, 0) for _, r in q)
        if Lq[mi] == 4 and 12 <= hq <= 14 and min(Lq) >= 2:
            break
    else:
        continue
    per[h] += 1
    hands.append((pbn(c), pbn(q)))
# responder: 13-14 HCP, four trumps, balanced (a 4M sign-off)
partners = []
while len(partners) < 40:
    c = rng.sample(deck, 13)
    L = [sum(1 for s2, _ in c if s2 == s) for s in range(4)]
    h = sum(HCP.get(r, 0) for _, r in c)
    if L[mi] == 4 and 12 <= h <= 14 and min(L) >= 2:
        partners.append(pbn(c))
with open(out, "w") as f:
    f.write(f"# Standard Jacoby 2NT, opener over 1{major}: {len(hands)} hands, 12-21 HCP, seed {seed}\n")
    f.write("# (probes/jacoby-opener-*; jacoby-2nt.notes.md)\n")
    f.write('card    = "21GF-GIB"\ndealer  = "S"\n')
    f.write(f'prefix  = "{prefix}"\nvuls    = ["None"]\nscoring = ["IMP"]\n')
    f.write("partner = [" + ", ".join(f'"{p}"' for p in partners) + "]\n")
    f.write("our     = []\nhands = [\n")
    for i, (hd, q) in enumerate(hands):
        f.write(f'  "h{i} | {hd} | {q}",\n')
    f.write("]\n")
