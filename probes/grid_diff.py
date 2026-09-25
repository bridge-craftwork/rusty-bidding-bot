#!/usr/bin/env python3
"""Where a grid run disagrees with BBA, grouped: (key..., BBA, ours) -> count.

    probes/grid_diff.py NAME [--col 0] [--key "L[0]"] [--key hcp] [--top 40]

Keys are Python expressions over L (lengths, spades first), h (HCP),
t (tens) and hand (S.H.D.C).
"""
import argparse
import collections
import json

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from grid_tally import call  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name")
    ap.add_argument("--col", type=int, default=0)
    ap.add_argument("--key", action="append", default=[])
    ap.add_argument("--top", type=int, default=40)
    ap.add_argument("--all", action="store_true", help="count agreements too")
    a = ap.parse_args()
    g = json.load(open(f".rbb-cache/grids/{a.name}/grid.json"))
    c = collections.Counter()
    ex = {}
    for r in g["rows"]:
        cell = r["cells"][a.col]
        b, o = call(cell["bba"]), call(cell["ours"])
        if b == o and not a.all:
            continue
        env = {"L": r["shape"], "h": r["hcp"], "t": r["tens"], "hand": r["hand"]}
        k = tuple(eval(e, {}, env) for e in a.key) + (b, o)
        c[k] += 1
        ex.setdefault(k, r["hand"])
    n = len(g["rows"])
    print(f"{sum(c.values())} of {n}")
    for k, v in c.most_common(a.top):
        print(f"{v:5}  {k}  e.g. {ex[k]}")


if __name__ == "__main__":
    main()
