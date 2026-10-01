#!/usr/bin/env python3
"""BBA's slam count, reconstructed: placement variants and agreement tables.

    probes/tools/slam_count.py variants POSITION [--bases 40] [--seed 1]
    probes/tools/slam_count.py moves
    probes/tools/slam_count.py agree

POSITION is one of the six Phase A grids (probes/slam-*.toml, run first with
`rbb grid` into .rbb-cache/grids/). `variants` writes probes/slam-var-<pos>.toml:
borderline hands of that grid (BBA bids slam on 10-90% of the hands with the
same support points + partner's floor), half with BBA's slam call and half
without, each with every variant that moves one honour to another suit
(same HCP, same shape) or one spot card to another suit (same HCP, new shape).
Run it with `rbb grid probes/slam-var-<pos>.toml`.

`moves` prints how often each kind of move changes BBA's call (variant minus
base: +1 when the variant bids slam and the base did not).

`agree` prints how often a count, compared with one threshold after adding
partner's floor (BBA's own meaning of his last call), agrees with BBA's slam
call: 5-fold on the random hands (threshold chosen on four folds, scored on
the fifth), and on the variant grids (never fitted).
slam-entry.notes.md, "BBA's count, reconstructed" (2026-09-30).
"""
import json
import os
import random
import re
import sys
from collections import defaultdict

GRIDS = ".rbb-cache/grids"
SUIT = {"Clubs": "C", "Diamonds": "D", "Hearts": "H", "Spades": "S", "NoTrump": "N"}
TI = {"S": 0, "H": 1, "D": 2, "C": 3}
NM = "SHDC"
HC = {"A": 4, "K": 3, "Q": 2, "J": 1}
RANKS = "AKQJT98765432"
YES = re.compile(r"4N|5H|5S|5N|6.|7.")
# grid, trump, partner's floor in BBA's total points (its meaning of his last call), partner's suit
POS = [
    ("slam-1N-3S-4S-resp", "S", 15, None),
    ("slam-1S-2S-opener", "S", 7, None),
    ("slam-1D-1H-4H-resp", "H", 16, "D"),
    ("slam-2C-2H-3H-resp", "H", 21, None),
    ("slam-1H-2D-3H-resp", "H", 14, None),
    ("slam-1S-2D-3C-3S-opener", "S", 13, "D"),
]


def call(c):
    if c is None:
        return "-"
    if isinstance(c, str):
        return {"Pass": "P", "Double": "X", "Redouble": "XX"}.get(c, c)
    b = c["Bid"]
    return f"{b['level']}{SUIT[b['strain']]}"


def rows(grid):
    g = json.load(open(os.path.join(GRIDS, grid, "grid.json")))
    return [(r["label"], r["hand"], 1 if YES.fullmatch(call(r["cells"][0]["bba"])) else 0) for r in g["rows"]]


def hcp(h):
    return sum(HC.get(c, 0) for c in h)


def tp(h, tr):
    S = h.split(".")
    t = TI[tr]
    short = sum({0: 5, 1: 3, 2: 1}.get(len(s), 0) for i, s in enumerate(S) if i != t)
    return hcp(h) + min(short, len(S[t]))


def controls(h):
    return 2 * h.count("A") + h.count("K")


def recon_half(h, tr):
    """The reconstructed count in half points: HCP, an ace half a point more,
    a queen or jack outside trumps half a point less, a void 2, a singleton 1
    (a singleton king 0), each card beyond four in a side suit 1."""
    S = h.split(".")
    t = TI[tr]
    v = 2 * hcp(h) + h.count("A")
    for i, s in enumerate(S):
        if i == t:
            continue
        v -= ("Q" in s) + ("J" in s)
        v += 2 * ((len(s) <= 1 and "K" not in s) + (len(s) == 0))
        v += 2 * max(0, len(s) - 4)
    return v


# count (in half points, partner's floor added) or a yes/no rule
COUNTS = {
    "tp (support points)": lambda h, tr, fl: 2 * (tp(h, tr) + fl),
    "hcp": lambda h, tr, fl: 2 * (hcp(h) + fl),
    "reconstructed": lambda h, tr, fl: recon_half(h, tr) + 2 * fl,
}


def built(h, tr, fl):  # direct_slam_values, read on BBA's floor
    return int(tp(h, tr) + fl >= 29 and tp(h, tr) + fl + controls(h) >= 35)


def folds(ys, k=5, seed=0):
    """Stratified fold number per row."""
    rnd = random.Random(seed)
    f = [0] * len(ys)
    for cls in sorted(set(ys)):
        idx = [i for i, y in enumerate(ys) if y == cls]
        rnd.shuffle(idx)
        for j, i in enumerate(idx):
            f[i] = j % k
    return f


def agree():
    data = []  # (pos, hand, y)
    for gi, (g, tr, fl, ps) in enumerate(POS):
        for _, h, y in rows(g):
            data.append((gi, h, y))
    var = []
    for gi, (g, tr, fl, ps) in enumerate(POS):
        name = g.replace("slam-", "slam-var-")
        if os.path.exists(os.path.join(GRIDS, name, "grid.json")):
            for _, h, y in rows(name):
                var.append((gi, h, y))
    F = folds([gi * 2 + y for gi, _, y in data])
    print(f"{'count':22}" + "".join(f"{p[0].replace('slam-', '')[:13]:>14}" for p in POS) + f"{'mean':>8}{'thr':>6}{'variants':>10}")
    for nm, fn in COUNTS.items():
        sc = [fn(h, POS[gi][1], POS[gi][2]) for gi, h, _ in data]
        acc = defaultdict(list)
        for k in range(5):
            tr_i = [i for i in range(len(data)) if F[i] != k]
            cuts = sorted(set(sc[i] for i in tr_i))
            best = max(cuts, key=lambda c: sum((sc[i] >= c) == data[i][2] for i in tr_i))
            for gi in range(len(POS)):
                te = [i for i in range(len(data)) if F[i] == k and data[i][0] == gi]
                acc[gi].append(sum((sc[i] >= best) == data[i][2] for i in te) / len(te))
        per = [sum(acc[g]) / len(acc[g]) for g in range(len(POS))]
        best = max(sorted(set(sc)), key=lambda c: sum((s >= c) == d[2] for s, d in zip(sc, data)))
        vtxt = ""
        if var:
            vs = [(fn(h, POS[gi][1], POS[gi][2]) >= best) == y for gi, h, y in var]
            vtxt = f"{100 * sum(vs) / len(vs):9.1f}%"
        print(f"{nm:22}" + "".join(f"{100 * a:13.1f}%" for a in per) + f"{100 * sum(per) / len(per):7.1f}%{best / 2:6.1f}{vtxt}")
    per = []
    for gi in range(len(POS)):
        d = [(h, y) for g, h, y in data if g == gi]
        per.append(sum(built(h, POS[gi][1], POS[gi][2]) == y for h, y in d) / len(d))
    vtxt = ""
    if var:
        vs = [built(h, POS[gi][1], POS[gi][2]) == y for gi, h, y in var]
        vtxt = f"{100 * sum(vs) / len(vs):9.1f}%"
    print(f"{'direct_slam_values':22}" + "".join(f"{100 * a:13.1f}%" for a in per) + f"{100 * sum(per) / len(per):7.1f}%{'':6}{vtxt}")


def norm(s):
    return "".join(sorted(s, key=RANKS.index))


def spots(s):
    return [c for c in s if c not in "AKQJ"]


def fresh(s):
    return next((c for c in "23456789T" if c not in s), None)


def moves_of(h):
    S = h.split(".")
    out = []
    for a in range(4):
        for X in "AKQJ":
            if X not in S[a]:
                continue
            for b in range(4):
                sp = spots(S[b])
                if b == a or X in S[b] or not sp:
                    continue
                low = sorted(sp, key=RANKS.index)[-1]
                na = S[a].replace(X, "", 1)
                c = fresh(na)
                if c is None:
                    continue
                T = list(S)
                T[a], T[b] = norm(na + c), norm(S[b].replace(low, "", 1) + X)
                out.append((f"{X}{NM[a]}>{NM[b]}", ".".join(T)))
    for a in range(4):
        sp = spots(S[a])
        if not sp:
            continue
        low = sorted(sp, key=RANKS.index)[-1]
        for b in range(4):
            c = fresh(S[b])
            if b == a or c is None:
                continue
            T = list(S)
            T[a], T[b] = norm(S[a].replace(low, "", 1)), norm(S[b] + c)
            out.append((f"x{NM[a]}>{NM[b]}", ".".join(T)))
    return out


def variants(pos, n, seed):
    g, tr, fl, ps = next(p for p in POS if p[0] == pos)
    band = defaultdict(list)
    for _, h, y in rows(g):
        band[tp(h, tr) + fl].append((h, y))
    cands = []
    for k in sorted(band):
        L = band[k]
        if 0.1 <= sum(y for _, y in L) / len(L) <= 0.9:
            cands += L
    random.Random(seed).shuffle(cands)
    base = [h for h, y in cands if y][: n // 2]
    base += [h for h, y in cands if not y][: n - len(base)]
    spec = open(f"probes/{g}.toml").read()
    m = re.search(r"--partner-hcp (\d+) (\d+)", spec)
    head = [f"# Placement variants of {len(base)} borderline hands from probes/{g}.toml (BBA bare SAYC, matchpoints).",
            "# slam-entry.notes.md, \"BBA's count, reconstructed\" (2026-09-30).",
            f"# Made with: probes/tools/slam_count.py variants {g} --bases {n} --seed {seed}"]
    for line in spec.splitlines():
        if line.startswith("hands"):
            break
        if line.startswith("#"):
            continue
        head.append('scoring = ["MP"]' if line.startswith("scoring") else line)
    head.append(f"partner_auto = [{m.group(1)}, {m.group(2)}]")
    lines = []
    for i, h in enumerate(base):
        lines.append(f'  "b{i}:base | {h}",')
        lines += [f'  "b{i}:{lab} | {v}",' for lab, v in moves_of(h)]
    out = f"probes/{g.replace('slam-', 'slam-var-')}.toml"
    with open(out, "w") as f:
        f.write("\n".join(head) + "\nhands = [\n" + "\n".join(lines) + "\n]\n")
    print(f"{out}: {len(base)} bases, {len(lines)} hands")


def role(i, t, ps):
    if i == t:
        return "T"
    if ps is not None and i == TI[ps]:
        return "P"
    return "s"


def moves():
    res = defaultdict(lambda: defaultdict(list))
    for g, tr, fl, ps in POS:
        name = g.replace("slam-", "slam-var-")
        if not os.path.exists(os.path.join(GRIDS, name, "grid.json")):
            continue
        R = rows(name)
        base = {lab.split(":")[0]: (h, y) for lab, h, y in R if lab.endswith(":base")}
        t = TI[tr]
        for lab, h, y in R:
            bi, mv = lab.split(":", 1)
            if mv == "base":
                continue
            bh, by = base[bi]
            S = bh.split(".")
            a, b = NM.index(mv[1]), NM.index(mv[3])
            if mv[0] in "AKQJ":
                k = f"{mv[0]} {role(a, t, ps)}>{role(b, t, ps)}"
            else:
                k = f"x {role(a, t, ps)}{len(S[a])}>{role(b, t, ps)}{len(S[b])}"
            res[k][g].append(y - by)
    print("T trumps, P partner's suit, s another side suit; x a spot card, with the suit lengths before the move")
    print(f"{'move':12}" + "".join(f"{p[0].replace('slam-', '')[:13]:>15}" for p in POS) + f"{'all':>14}")
    for k in sorted(res, key=lambda k: -sum(len(v) for v in res[k].values())):
        allv = [x for v in res[k].values() for x in v]
        if len(allv) < 20:
            continue
        cells = [(f"{sum(v) / len(v):+.2f} ({len(v):3})" if v else "") for v in (res[k].get(p[0], []) for p in POS)]
        print(f"{k:12}" + "".join(f"{c:>15}" for c in cells) + f"{sum(allv) / len(allv):+7.2f} ({len(allv):4})")


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "agree"
    if cmd == "variants":
        a = sys.argv[2:]
        n = int(a[a.index("--bases") + 1]) if "--bases" in a else 40
        s = int(a[a.index("--seed") + 1]) if "--seed" in a else 1
        variants(a[0], n, s)
    elif cmd == "moves":
        moves()
    else:
        agree()
