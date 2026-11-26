#!/usr/bin/env python3
"""WBridge5's call at each range-probe position, read from the PBN that
probes/tools/wbridge5_bid.py wrote (WBridge5 bids the whole deal at all
four seats, its own default system, so the position is reached only when
it opens and the others pass as the prefix says).

    probes/range-probe/wb5_calls.py POSITION WB5.pbn

Writes `data/<position>.wb5.jsonl`: id, call (or null with `reached`:
false and the auction it bid instead).
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def games(text):
    out, cur = [], []
    for line in text.splitlines():
        if line.strip() == "":
            if cur:
                out.append(cur)
                cur = []
        else:
            cur.append(line)
    if cur:
        out.append(cur)
    return out


def norm(c):
    c = c.strip().upper()
    if c in ("PASS", "P", "AP"):
        return "Pass"
    if c in ("X", "XX"):
        return c
    c = re.sub(r"N$", "NT", c)
    return c


def main():
    pos, path = sys.argv[1], sys.argv[2]
    rows = [json.loads(l) for l in open(os.path.join(HERE, "data", f"{pos}.jsonl"))]
    by_deal = {}
    for g in games(open(path).read()):
        deal = auction = None
        calls = []
        in_auc = False
        for l in g:
            m = re.match(r'\[(\w+)\s+"(.*)"\]', l)
            if m:
                in_auc = m.group(1) == "Auction"
                if m.group(1) == "Deal":
                    deal = m.group(2)
                continue
            if in_auc and not l.startswith("%"):
                for tok in l.split():
                    if tok.startswith("="):
                        continue
                    if tok == "AP":
                        calls += ["Pass"] * 3
                    else:
                        calls.append(norm(tok))
        if deal:
            by_deal[deal] = calls
    out = []
    for r in rows:
        calls = by_deal.get(r["deal"])
        if calls is None:
            continue
        pre = r["prefix"].split()
        k = len(pre)
        reached = calls[:k] == pre and len(calls) > k
        out.append({"id": r["id"], "call": calls[k] if reached else None, "reached": reached,
                    "auction": " ".join(calls)})
    with open(os.path.join(HERE, "data", f"{pos}.wb5.jsonl"), "w") as f:
        for o in out:
            f.write(json.dumps(o) + "\n")
    print(f"{pos}: {len(out)} boards, {sum(o['reached'] for o in out)} reached the position")


if __name__ == "__main__":
    main()
