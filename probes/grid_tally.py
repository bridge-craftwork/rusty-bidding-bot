#!/usr/bin/env python3
"""Tally a grid run: BBA's calls per column, where ours differs, and what
BBA's auction went on to (bba_rest).

    probes/grid_tally.py NAME [--by hcp|shape] [--rest] [--diff]
"""
import argparse
import collections
import json

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
    ap.add_argument("--by", choices=["hcp", "shape"])
    ap.add_argument("--rest", action="store_true", help="tally BBA's continuations")
    ap.add_argument("--diff", action="store_true", help="list hands where ours differs")
    a = ap.parse_args()
    g = json.load(open(f".rbb-cache/grids/{a.name}/grid.json"))
    cols = [f"{c['vul']}/{'MP' if c['scoring'] == 'Matchpoints' else 'IMP'}" for c in g["columns"]]
    for i, col in enumerate(cols):
        t = collections.Counter()
        agree = 0
        for r in g["rows"]:
            cell = r["cells"][i]
            key = {"hcp": r["hcp"], "shape": "".join(map(str, r["shape"]))}.get(a.by)
            b = call(cell["bba"])
            t[(key, b) if a.by else b] += 1
            agree += b == call(cell["ours"])
        print(f"== {col}: agree {agree}/{len(g['rows'])}")
        for k, v in sorted(t.items(), key=str):
            print(f"  {k}: {v}")
        if a.rest:
            rt = collections.Counter(r["cells"][i]["bba_rest"] for r in g["rows"])
            for k, v in rt.most_common(40):
                print(f"  {v:4}  {k}")
        if a.diff:
            for r in g["rows"]:
                cell = r["cells"][i]
                if call(cell["bba"]) != call(cell["ours"]):
                    print(f"  {r['hand']:22} {r['hcp']:2}  bba {call(cell['bba']):3} ours {call(cell['ours']):3}  {cell['bba_rest']}")


if __name__ == "__main__":
    main()
