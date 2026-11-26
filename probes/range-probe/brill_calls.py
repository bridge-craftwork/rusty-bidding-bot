#!/usr/bin/env python3
"""Brill's call for each range-probe deal, from Thorvald Aagaard's public
Brill service (`GET /bid`), through brill_replay.py's client: sequential,
one request a second, a named User-Agent, every answer cached under the
main checkout's `.rbb-cache/brill-service/bid/` and never asked again, a
stop after 5 failures in a row (rerun to resume).

    probes/range-probe/brill_calls.py [POSITION...] [--offline]

Writes `data/<position>.brill.jsonl`: id, call, Brill's rule set and the
requirement its rule states.
"""
import argparse
import hashlib
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "tools"))
import brill_replay as brp  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("positions", nargs="*", default=["resp1S", "resp1H", "reb1D1S"])
    ap.add_argument("--offline", action="store_true")
    ap.add_argument("--interval", type=float, default=1.0)
    a = ap.parse_args()
    client = brp.Brill(a)
    for pos in a.positions:
        rows = [json.loads(l) for l in open(os.path.join(HERE, "data", f"{pos}.jsonl"))]
        out = []
        for i, r in enumerate(rows):
            calls = r["prefix"].split()
            try:
                call = client.bid(r["hand"], calls, "N", r["decider"], "None", "IMP")
            except LookupError as e:
                print(f"{r['id']}: {e}")
                continue
            q = {"hand": brp.br.sorted_hand(r["hand"]), "ctx": brp.ctx_of(calls), "dealer": "N",
                 "seat": r["decider"], "vul": "None", "matchtype": "IMP"}
            path = os.path.join(client.dir, client.key(q) + ".json")
            ans = json.load(open(path))["answer"]
            out.append({"id": r["id"], "call": call, "ruleSet": ans.get("ruleSet"),
                        "requires": ans.get("requires"), "alert": ans.get("alert")})
            if (i + 1) % 50 == 0:
                print(f"{pos}: {i + 1}/{len(rows)} (fetched {client.fetched}, cached {client.hits})",
                      flush=True)
        with open(os.path.join(HERE, "data", f"{pos}.brill.jsonl"), "w") as f:
            for o in out:
                f.write(json.dumps(o) + "\n")
        print(f"{pos}: {len(out)} calls")


if __name__ == "__main__":
    main()
