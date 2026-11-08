#!/usr/bin/env python3
"""Tally of the Jacoby opener survey (jacoby-2nt.notes.md): BBA's opener
after 1M P 2NT P <answer> P 4M P, from the grids of
probes/jacoby-opener-1{S,H}-<answer>-4{S,H}.toml.

    probes/jacoby_opener_tally.py [GRIDS_DIR]

GRIDS_DIR defaults to .rbb-cache/grids (where `rbb grid` writes). Prints,
per kind of answer (3x shortness, 3M, 3NT, 4x side suit): how often BBA
goes on by HCP, and by HCP x controls; the agreement of candidate lines
with BBA; and how often our engine (the grid's `ours`) agrees.
Points here are HCP plus one a card beyond four; controls A 2, K 1.
"""
import collections
import glob
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GRIDS = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, ".rbb-cache", "grids")
HCPV = {"A": 4, "K": 3, "Q": 2, "J": 1}


def call(c):
    if isinstance(c, str):
        return {"Pass": "P"}.get(c, c)
    b = c["Bid"]
    return f"{b['level']}{'N' if b['strain'] == 'NoTrump' else b['strain'][0]}"


def feats(hand):
    suits = hand.split(".")
    lens = [len(s) for s in suits]
    h = sum(HCPV.get(r, 0) for s in suits for r in s)
    pts = h + sum(max(0, n - 4) for n in lens)
    ctl = sum(2 * ("A" in s) + ("K" in s) for s in suits)
    return h, pts, ctl


kinds = collections.defaultdict(list)
calls = collections.Counter()
for d in sorted(glob.glob(os.path.join(GRIDS, "jacoby-opener-1?-*-4?", "grid.json"))):
    _, _, m, ans, _ = os.path.basename(os.path.dirname(d)).split("-")
    k = "3N" if ans == "3N" else "4x" if ans[0] == "4" else "3M" if ans[1] == m[1] else "3x"
    for r in json.load(open(d))["rows"]:
        c = r["cells"][0]
        bba = call(c["bba"])
        ours = call(c["ours"]) if c.get("ours") else "P"
        calls[bba] += 1
        kinds[k].append((feats(r["hand"]), bba != "P", ours != "P"))
if not kinds:
    sys.exit(f"no jacoby-opener grids under {GRIDS}")

for k in ("3x", "3M", "3N", "4x"):
    data = kinds[k]
    if not data:
        continue
    go = sum(d[1] for d in data)
    print(f"== {k}: BBA goes on {go} of {len(data)}; ours agrees {sum(d[1] == d[2] for d in data)}")
    by_h = collections.defaultdict(lambda: [0, 0])
    grid = collections.defaultdict(lambda: [0, 0])
    for (h, _, ctl), g, _ in data:
        by_h[h][0] += g
        by_h[h][1] += 1
        grid[(h, ctl)][0] += g
        grid[(h, ctl)][1] += 1
    print("  by HCP: " + ", ".join(f"{h}: {a}/{n}" for h, (a, n) in sorted(by_h.items())))
    cs = sorted({c for _, c in grid})
    print("  hcp\\controls " + " ".join(f"{c:>6}" for c in cs))
    for h in sorted(by_h):
        print(f"  {h:12} " + " ".join(f"{grid[(h, c)][0]:>2}/{grid[(h, c)][1]:<3}" if grid[(h, c)][1] else "     -" for c in cs))
    fits = []
    for a in range(14, 30):
        fits.append((sum((d[0][1] >= a) == d[1] for d in data), f"points >= {a}"))
        fits.append((sum((d[0][1] + d[0][2] >= a) == d[1] for d in data), f"points + controls >= {a}"))
    fits.sort(reverse=True)
    print("  best lines: " + "; ".join(f"{name} {n}/{len(data)}" for n, name in fits[:4]))
    for name, line in (("points + controls >= 22", lambda f: f[1] + f[2] >= 22),
                       ("points + controls >= 24", lambda f: f[1] + f[2] >= 24),
                       ("points >= 19", lambda f: f[1] >= 19),
                       ("points >= 20", lambda f: f[1] >= 20)):
        print(f"  {name}: {sum(line(d[0]) == d[1] for d in data)}/{len(data)}")
print("BBA's calls:", calls.most_common())
