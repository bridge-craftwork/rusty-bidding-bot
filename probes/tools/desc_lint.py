#!/usr/bin/env python3
"""List rule explanations that break the description style (docs/CONTRACT.md,
Part 2, "The explanation is what the table is told").

    probes/tools/desc_lint.py [PATH...] [--summary] [--max-len 60]

Each finding: file:line, the call, the text, and what is wrong with it.
The checks are heuristics: a finding is a text to look at, not
necessarily one to change ("Pass" with a parenthesis a player would say
is fine). Exit status 1 when anything is found, so it can gate a run.
"""

import argparse
import collections
import pathlib
import re
import sys

RULE = re.compile(r'^\s+(\S+)\s+"([^"]*)"')

CHECKS = [
    ("parenthesis", re.compile(r"\(")),
    ("reasoning", re.compile(
        r"\b(because|since|instead of|rather than|between us|our count|"
        r"the last suit we|replaces|we have|we hold|are ours|is ours|"
        r"on partner's floor|partner at h\w+ minimum)\b", re.I)),
    ("engine ran out", re.compile(r"nothing (more|new|else) to (say|show)|no rule", re.I)),
    ("card note", re.compile(r"\bwhen (no )?[\w -]+ is (not )?played\b|\bon the card\b", re.I)),
    ("two colons", re.compile(r":[^:]*:")),
]


def lint(text, max_len):
    found = [name for name, rx in CHECKS if rx.search(text)]
    # The name before a colon, and what follows: long either way.
    if len(re.sub(r"\{[^}]*\}", "x", text)) > max_len:
        found.append(f"over {max_len} characters")
    return found


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("paths", nargs="*", default=["conventions"])
    ap.add_argument("--summary", action="store_true", help="counts per check and directory only")
    ap.add_argument("--max-len", type=int, default=60)
    args = ap.parse_args()

    files = []
    for p in map(pathlib.Path, args.paths):
        files += sorted(p.rglob("*.bid")) if p.is_dir() else [p]

    by_check = collections.Counter()
    by_dir = collections.Counter()
    total = 0
    for f in files:
        for n, line in enumerate(f.read_text().splitlines(), 1):
            m = RULE.match(line)
            if not m:
                continue
            call, text = m.groups()
            issues = lint(text, args.max_len)
            if not issues:
                continue
            total += 1
            by_check.update(issues)
            by_dir[f.parent.name] += 1
            if not args.summary:
                print(f"{f}:{n}  {call}  \"{text}\"  [{', '.join(issues)}]")
    if args.summary or total:
        print(f"\n{total} texts to look at", file=sys.stderr)
        for k, v in by_check.most_common():
            print(f"  {k:24} {v}", file=sys.stderr)
        for k, v in by_dir.most_common():
            print(f"  {k + '/':24} {v}", file=sys.stderr)
    return 1 if total else 0


if __name__ == "__main__":
    sys.exit(main())
