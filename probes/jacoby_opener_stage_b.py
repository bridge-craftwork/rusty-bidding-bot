#!/usr/bin/env python3
"""Second stage of the Jacoby opener survey (jacoby-2nt.notes.md).

    probes/jacoby_opener_stage_b.py MAJOR

Reads BBA's answers to 2NT from .rbb-cache/grids/jacoby-opener-1M-answer/
(after `rbb grid probes/jacoby-opener-1M-answer.toml`) and writes one
spec per answer, probes/jacoby-opener-1M-<answer>-4M.toml, with the
prefix 1M P 2NT P <answer> P 4M P: the same hands and responders, opener
to call again. Run `rbb grid` on each, then probes/jacoby_opener_tally.py.
"""
import collections
import json
import os
import re
import sys

M = sys.argv[1]
W = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
spec = open(f"{W}/probes/jacoby-opener-1{M}-answer.toml").read()
partner = dict(re.findall(r'"(h\d+) \| [^|]+ \| ([^"]+)"', spec))
d = json.load(open(f"{W}/.rbb-cache/grids/jacoby-opener-1{M}-answer/grid.json"))


def call(c):
    if isinstance(c, str):
        return {"Pass": "P"}.get(c, c)
    b = c["Bid"]
    return f"{b['level']}{'N' if b['strain'] == 'NoTrump' else b['strain'][0]}"


groups = collections.defaultdict(list)
for r in d["rows"]:
    a = call(r["cells"][0]["bba"])
    if a == f"4{M}" or a[0] not in "34":
        continue
    groups[a].append((r["label"], r["hand"], partner[r["label"]]))
for a, rows in sorted(groups.items()):
    name = f"jacoby-opener-1{M}-{a}-4{M}"
    with open(f"{W}/probes/{name}.toml", "w") as f:
        f.write(f"# Standard Jacoby 2NT: opener after 1{M} P 2NT P {a} P 4{M} P (responder's sign-off).\n")
        f.write(f"# Hands: BBA's own {a} answers in probes/jacoby-opener-1{M}-answer.toml (jacoby_opener_stage_b.py).\n")
        f.write('card    = "21GF-GIB"\ndealer  = "S"\n')
        f.write(f'prefix  = "1{M} Pass 2NT Pass {a} Pass 4{M} Pass"\nvuls    = ["None"]\nscoring = ["IMP"]\n')
        f.write("our     = []\nhands = [\n")
        for lab, h, p in rows:
            f.write(f'  "{lab} | {h} | {p}",\n')
        f.write("]\n")
    print(name, len(rows))
