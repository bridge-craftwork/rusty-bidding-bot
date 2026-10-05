#!/usr/bin/env python3
"""Tripwire for the convention fast lane (CLAUDE.md "Pace"): which boards
changed our auction between two `compare --json` runs, and at which call.

    probes/tools/tripwire.py BASE.json VARIANT.json [--expect PATTERN ...] [-v]

Make both runs with `compare --limit 50` over all scenarios (the base:
.rbb-cache/tripwire/ in the main checkout). Prints the changed boards per
scenario, the rule that now makes the first changed call, and a total.
`--expect` takes scenario glob patterns where changes are intended (the
convention's own scenarios); changes elsewhere are listed first, as the
ones to explain. `-v` prints each changed board's two auctions.

No IMP verdict: a change outside the expected scenarios is a question
(did the convention fire where it should not, or take over a call?),
not a score.
"""

import argparse
import fnmatch
import json
from collections import Counter, defaultdict

STRAIN = {"Clubs": "C", "Diamonds": "D", "Hearts": "H", "Spades": "S", "NoTrump": "N"}


def call(c):
    if isinstance(c, dict) and "Bid" in c:
        return f"{c['Bid']['level']}{STRAIN[c['Bid']['strain']]}"
    return {"Pass": "P", "Double": "X", "Redouble": "XX"}.get(c, str(c))


def load(path):
    with open(path) as f:
        return {(b["scenario"], b["board"]): b for b in json.load(f)["boards"]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("base")
    ap.add_argument("variant")
    ap.add_argument("--expect", action="append", default=[])
    ap.add_argument("-v", action="store_true")
    a = ap.parse_args()
    base, var = load(a.base), load(a.variant)

    changed = defaultdict(list)
    for key, v in var.items():
        b = base.get(key)
        if b is None or b["ours"] == v["ours"]:
            continue
        i = next((i for i, (x, y) in enumerate(zip(b["ours"], v["ours"])) if x != y),
                 min(len(b["ours"]), len(v["ours"])))
        rule = v["our_rules"][i] if i < len(v["our_rules"]) else "?"
        changed[key[0]].append((key[1], i, rule, b, v))

    def expected(s):
        return any(fnmatch.fnmatch(s, p) for p in a.expect)

    total = sum(len(x) for x in changed.values())
    print(f"{total} of {len(var)} boards changed, in {len(changed)} scenarios")
    for group, keep in (("UNEXPECTED (explain these)", lambda s: not expected(s)),
                        ("expected", expected)):
        names = sorted((s for s in changed if keep(s)), key=lambda s: -len(changed[s]))
        if not names:
            continue
        print(f"\n{group}: {sum(len(changed[s]) for s in names)} boards")
        for s in names:
            rules = Counter(r for _, _, r, _, _ in changed[s])
            print(f"  {s:40} {len(changed[s]):3}   " +
                  "; ".join(f"{r} x{n}" for r, n in rules.most_common(3)))
            if a.v:
                for bd, i, _, b, v in changed[s]:
                    print(f"      bd {bd:3} was: {' '.join(map(call, b['ours']))}")
                    print(f"             now: {' '.join(map(call, v['ours']))}  (call {i + 1})")


if __name__ == "__main__":
    main()
