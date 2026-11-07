#!/usr/bin/env python3
"""Tally the opnt-* grids (probes/opener-nt-contested_gen.py): BBA's call
by HCP, stopper and length in partner's suit, ours beside it, and BBA's
meaning of each call.

    probes/opnt_tally.py opnt-1D-1H-2S [--meanings] [--rest CALL]
"""
import argparse
import collections
import json
import re

SUIT = {"Clubs": "C", "Diamonds": "D", "Hearts": "H", "Spades": "S", "NoTrump": "N"}


def call(c):
    if c is None:
        return "-"
    if isinstance(c, str):
        return {"Pass": "P", "Double": "X", "Redouble": "XX"}.get(c, c)
    b = c["Bid"]
    return f"{b['level']}{SUIT[b['strain']]}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name")
    ap.add_argument("--meanings", action="store_true")
    ap.add_argument("--z", action="store_true", help="do not cap the length at 3")
    ap.add_argument("--rest", help="list BBA's continuations after this call")
    a = ap.parse_args()
    g = json.load(open(f".rbb-cache/grids/{a.name}/grid.json"))
    t = collections.defaultdict(collections.Counter)
    ours = collections.defaultdict(collections.Counter)
    meanings = collections.defaultdict(collections.Counter)
    agree = 0
    for r in g["rows"]:
        m = re.match(r"h(\d+) st(\d) z(\d)", r["label"])
        h, st, z = int(m[1]), int(m[2]), int(m[3])
        c = r["cells"][0]
        b, o = call(c["bba"]), call(c["ours"])
        agree += b == o
        key = (h, st, z if a.z else min(z, 3))
        t[key][b] += 1
        ours[key][o] += 1
        meanings[b][re.sub(r"^\S+ (ALERT\. )?", "", c.get("bba_alert") or "")[:160]] += 1
        if a.rest and b == a.rest:
            print(f"  {r['label']:12} {r['hand']:18} {c.get('bba_rest')}")
    print(f"{a.name}: {len(g['rows'])} hands, ours agrees {agree}")
    print("hcp st z3 | BBA | ours")
    for key in sorted(t):
        bb = " ".join(f"{k}:{v}" for k, v in t[key].most_common())
        oo = " ".join(f"{k}:{v}" for k, v in ours[key].most_common())
        mark = "" if t[key] == ours[key] else "   <>"
        print(f"{key[0]:3} {key[1]:2} {key[2]:2} | {bb:24} | {oo}{mark}")
    if a.meanings:
        for b, ms in meanings.items():
            print(f"== {b}")
            for k, v in ms.most_common(4):
                print(f"   {v:3} {k}")


if __name__ == "__main__":
    main()
