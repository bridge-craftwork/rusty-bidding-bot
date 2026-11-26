#!/usr/bin/env python3
"""Our engine's call for each range-probe deal: `rbb call --batch` on
21GF-GIB (the compiled-in rules), the auction forced to the position.

    probes/range-probe/ours_calls.py [POSITION...] [--rbb target/release/rbb] [--card 21GF-GIB]

Writes `data/<position>.ours.jsonl`: id, call, the rule that chose it.
"""
import argparse
import json
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))


STRAIN = {"Clubs": "C", "Diamonds": "D", "Hearts": "H", "Spades": "S", "NoTrump": "N"}


def call_text(c):
    if isinstance(c, dict) and "Bid" in c:
        return f"{c['Bid']['level']}{STRAIN[c['Bid']['strain']]}"
    return {"Pass": "P", "Double": "X", "Redouble": "XX"}.get(c, str(c))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("positions", nargs="*", default=["resp1S", "resp1H", "reb1D1S"])
    ap.add_argument("--rbb", default=os.path.join(ROOT, "target", "release", "rbb"))
    ap.add_argument("--card", default=os.path.join(ROOT, "cards", "bbsa", "21GF-GIB.bbsa"))
    ap.add_argument("--rules", help="a pack checkout (default: the compiled-in snapshot)")
    ap.add_argument("--rules-at", help="the pack as at this git ref (needs --rules)")
    ap.add_argument("--tag", help="write data/<position>.ours-<tag>.jsonl instead")
    a = ap.parse_args()
    extra = (["--rules", a.rules] if a.rules else []) + (["--rules-at", a.rules_at] if a.rules_at else [])
    for pos in a.positions:
        rows = [json.loads(l) for l in open(os.path.join(HERE, "data", f"{pos}.jsonl"))]
        inp = "".join(json.dumps({"hand": r["hand"], "auction": r["prefix"], "dealer": "N",
                                  "vul": "None", "scoring": "IMP", "id": r["id"]}) + "\n"
                      for r in rows)
        p = subprocess.run([a.rbb, "call", "--batch", "-c", a.card] + extra, input=inp,
                           capture_output=True, text=True, check=True)
        out = []
        for line in p.stdout.splitlines():
            d = json.loads(line)
            rule = d.get("rule") or {}
            out.append({"id": d.get("id"), "call": call_text(d.get("call")),
                        "rule": f"{rule.get('file')}:{rule.get('line')}" if rule else None,
                        "explanation": d.get("explanation")})
        with open(os.path.join(HERE, "data", f"{pos}.ours{'-' + a.tag if a.tag else ''}.jsonl"), "w") as f:
            for o in out:
                f.write(json.dumps(o) + "\n")
        print(f"{pos}: {len(out)} calls")


if __name__ == "__main__":
    main()
