#!/usr/bin/env python3
"""Which hand measure carries BBA's slam decision, over several `rbb grid` runs.

    probes/tools/slam_measures.py GRID:TRUMP:FLOOR ... [--yes REGEX] [--col N]

GRID is a run in .rbb-cache/grids/, TRUMP the agreed suit (S H D C), FLOOR
partner's minimum in BBA's own total points (from BBA's meaning of
partner's last call). A decision is "yes" when BBA's call matches --yes
(default: 4NT, 5M, 5NT or any six or seven).

Prints, for each candidate measure, the log-likelihood per decision of a
logistic fit with a separate intercept per position (higher is better),
then BBA's slam rate by the combined count (my support points plus
partner's floor) pooled over the positions, by controls at each count,
and the card values of a free fit (in HCP, a king = 3).
slam-entry.notes.md, "Phase A" (2026-09-30).
"""
import argparse
import collections
import json
import math
import re

SUIT = {"Clubs": "C", "Diamonds": "D", "Hearts": "H", "Spades": "S", "NoTrump": "N"}
TI = {"S": 0, "H": 1, "D": 2, "C": 3}
HCP = {"A": 4, "K": 3, "Q": 2, "J": 1}


def call(c):
    if c is None:
        return "-"
    if isinstance(c, str):
        return {"Pass": "P", "Double": "X", "Redouble": "XX"}.get(c, c)
    b = c["Bid"]
    return f"{b['level']}{SUIT[b['strain']]}"


def losers(suits):
    n = 0
    for s in suits:
        m = min(len(s), 3)
        n += m - ("A" in s[:m]) - (m >= 2 and "K" in s[:m]) - (m >= 3 and "Q" in s[:m])
    return n


def feats(hand, trump):
    S = hand.split(".")
    L = [len(s) for s in S]
    t = TI[trump]
    hcp = sum(HCP.get(c, 0) for c in hand)
    short = sum({0: 5, 1: 3, 2: 1}.get(x, 0) for i, x in enumerate(L) if i != t)
    return dict(
        hcp=hcp,
        tp=hcp + min(short, L[t]),  # the engine's tp(x): shortness capped by trumps held
        points=hcp + hand.count("T") // 2 + sum(max(0, x - 4) for x in L),
        controls=sum(2 * s.count("A") + s.count("K") for s in S),
        losers=losers(S),
        aces=hand.count("A"), kings=hand.count("K"), queens=hand.count("Q"), jacks=hand.count("J"),
        short=min(short, L[t]), lenpts=sum(max(0, x - 4) for x in L), trumps=L[t],
        bare=sum(1 for i, s in enumerate(S) if i != t and len(s) >= 2 and "A" not in s and "K" not in s),
    )


def solve(A, b):
    n = len(b)
    M = [row[:] + [b[i]] for i, row in enumerate(A)]
    for c in range(n):
        p = max(range(c, n), key=lambda r: abs(M[r][c]))
        M[c], M[p] = M[p], M[c]
        if abs(M[c][c]) < 1e-12:
            continue
        for r in range(n):
            if r != c:
                f = M[r][c] / M[c][c]
                for k in range(c, n + 1):
                    M[r][k] -= f * M[c][k]
    return [M[i][n] / M[i][i] if abs(M[i][i]) > 1e-12 else 0 for i in range(n)]


def logit(X, Y, G):
    """Newton's method; separate intercept per group. Returns (LL per decision, weights per unit)."""
    n, d, ng = len(Y), len(X[0]), max(G) + 1
    mu = [sum(x[j] for x in X) / n for j in range(d)]
    sd = [(sum((x[j] - mu[j]) ** 2 for x in X) / n) ** 0.5 or 1 for j in range(d)]
    R = []
    for x, g in zip(X, G):
        z = [(x[j] - mu[j]) / sd[j] for j in range(d)] + [0.0] * ng
        z[d + g] = 1.0
        R.append(z)
    D = d + ng
    th = [0.0] * D

    def prob(z):
        s = sum(t * v for t, v in zip(th, z))
        return 1 / (1 + math.exp(-max(-30, min(30, s))))

    for _ in range(40):
        g = [1e-4 * t for t in th]
        H = [[(1e-4 if a == b else 0.0) for b in range(D)] for a in range(D)]
        for z, y in zip(R, Y):
            p = prob(z)
            e, v = p - y, p * (1 - p)
            for a in range(D):
                if z[a]:
                    g[a] += e * z[a]
                    for b in range(D):
                        if z[b]:
                            H[a][b] += v * z[a] * z[b]
        step = solve(H, g)
        th = [t - s for t, s in zip(th, step)]
        if max(abs(s) for s in step) < 1e-7:
            break
    ll = sum(math.log(max(1e-12, prob(z) if y else 1 - prob(z))) for z, y in zip(R, Y)) / n
    return ll, [th[j] / sd[j] for j in range(d)]


MEASURES = {
    "hcp": lambda f: [f["hcp"]],
    "points (HCP + length)": lambda f: [f["points"]],
    "losers": lambda f: [f["losers"]],
    "controls": lambda f: [f["controls"]],
    "hcp, losers": lambda f: [f["hcp"], f["losers"]],
    "hcp, controls": lambda f: [f["hcp"], f["controls"]],
    "tp (HCP + shortness)": lambda f: [f["tp"]],
    "tp + controls": lambda f: [f["tp"] + f["controls"]],
    "tp, controls, losers": lambda f: [f["tp"], f["controls"], f["losers"]],
}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("specs", nargs="+")
    ap.add_argument("--yes", default=r"4N|5H|5S|5N|6.|7.")
    ap.add_argument("--col", type=int, default=0)
    a = ap.parse_args()
    yes = re.compile(a.yes)
    data = []  # (feats, yes, group, floor)
    for gi, spec in enumerate(a.specs):
        name, trump, floor = spec.split(":")
        g = json.load(open(f".rbb-cache/grids/{name}/grid.json"))
        for r in g["rows"]:
            data.append((feats(r["hand"], trump), 1 if yes.fullmatch(call(r["cells"][a.col]["bba"])) else 0, gi, int(floor)))
    Y = [y for _, y, _, _ in data]
    G = [g for _, _, g, _ in data]
    print(f"{len(data)} decisions over {len(a.specs)} positions, BBA slam {sum(Y)}")
    print("\nlog-likelihood per decision (separate intercept per position; null "
          f"{sum(math.log(sum(1 for y2, g2 in zip(Y, G) if g2 == g and y2 == y) / G.count(g)) for y, g in zip(Y, G)) / len(Y):.3f}):")
    for name, fn in MEASURES.items():
        ll, w = logit([fn(f) for f, _, _, _ in data], Y, G)
        print(f"  {name:24} {ll:.3f}")
    comb = collections.defaultdict(lambda: [0, 0])
    byc = collections.defaultdict(lambda: [0, 0])
    for f, y, _, fl in data:
        c = f["tp"] + fl
        comb[c][0] += y
        comb[c][1] += 1
        k = (c, "0-3" if f["controls"] <= 3 else "4-5" if f["controls"] <= 5 else "6+")
        byc[k][0] += y
        byc[k][1] += 1
    print("\nBBA's slam rate by my tp + partner's floor (pooled):")
    print("  " + "  ".join(f"{c}:{100 * v[0] // v[1]}%" for c, v in sorted(comb.items()) if v[1] >= 20))
    print("  by controls:   " + "".join(f"{k:>12}" for k in ("0-3", "4-5", "6+")))
    for c in range(28, 36):
        print(f"  {c:13}  " + "".join(
            f"{(str(100 * byc[(c, k)][0] // byc[(c, k)][1]) + '% (' + str(byc[(c, k)][1]) + ')') if byc[(c, k)][1] >= 5 else '':>12}"
            for k in ("0-3", "4-5", "6+")))
    keys = ["aces", "kings", "queens", "jacks", "short", "lenpts", "bare"]
    ll, w = logit([[f[k] for k in keys] for f, _, _, _ in data], Y, G)
    unit = w[1] / 3
    print(f"\nfree card values (LL {ll:.3f}), in HCP with a king = 3:")
    print("  " + ", ".join(f"{k} {wi / unit:+.1f}" for k, wi in zip(keys, w)))


if __name__ == "__main__":
    main()
