#!/usr/bin/env python3
"""Tally the notrump-stopper grids (probes/nt_stopper_gen.py, then rbb grid).

    probes/nt_stopper_tally.py [--suffix ""] [--suffix -gib] [--detail NAME]

Per position and card: how often BBA (and we) bid natural notrump with a
stopper (A, Kx, Qxx, Jxxx) in every suit they have shown, with only a half
stopper (Qx, Jxx, Jx, K singleton... anything with an honour short of a
stopper), and with none; what BBA bids instead without one; and whether
BBA's meaning of its notrump call mentions a stopper. `--detail NAME`
prints the holdings in their suit behind each no-stopper NT call.
"""
import argparse
import collections
import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from nt_stopper_gen import POSITIONS, SUITS, stop  # noqa: E402

GRIDS = ".rbb-cache/grids"


def call_str(c):
    if c is None:
        return "?"
    if isinstance(c, str):
        return {"Pass": "P", "Double": "X", "Redouble": "XX"}.get(c, c)
    b = c.get("Bid")
    if not b:
        return str(c)
    st = {"NoTrump": "NT", "Spades": "S", "Hearts": "H", "Diamonds": "D", "Clubs": "C"}[b["strain"]]
    return f"{b['level']}{st}"


def stop_class(hand, suits):
    hs = hand.split(".")
    worst = "stop"
    for s in suits:
        h = hs[SUITS.index(s)]
        if stop(h):
            continue
        if any(r in h for r in "KQJ"):
            worst = "half" if worst == "stop" else worst
        else:
            worst = "none"
    return worst


def shape_of(holding):
    return "".join(r if r in "AKQJ" else "x" for r in holding) or "-"


def is_nt(call):
    return call.endswith("NT")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--suffix", action="append")
    ap.add_argument("--detail")
    a = ap.parse_args()
    suffixes = a.suffix or [""]
    for name, _d, prefix, theirs, (lo, hi), _w, _p, desc in POSITIONS:
        if a.detail and name != a.detail:
            continue
        for suf in suffixes:
            path = f"{GRIDS}/nt-stopper-{name}{suf}/grid.json"
            if not os.path.exists(path):
                continue
            rows = json.load(open(path))["rows"]
            by = collections.defaultdict(list)
            for r in rows:
                cell = r["cells"][0]
                by[stop_class(r["hand"], theirs)].append(
                    (r, call_str(cell.get("bba")), call_str(cell.get("ours")), cell.get("bba_alert") or ""))
            parts = []
            for cls in ("stop", "half", "none"):
                xs = by.get(cls, [])
                if not xs:
                    continue
                bnt = sum(is_nt(b) for _, b, _, _ in xs)
                ont = sum(is_nt(o) for _, _, o, _ in xs)
                parts.append(f"{cls} {bnt}/{len(xs)} (ours {ont})")
            # What BBA bids without a full stopper, NT alerts mentioning a stopper.
            alt = collections.Counter(b for cls in ("half", "none") for _, b, _, _ in by.get(cls, []))
            alerts = [al for xs in by.values() for _, b, _, al in xs if is_nt(b)]
            stopmention = sum("stop" in al.lower() for al in alerts)
            print(f"{name}{suf:6} [{prefix}] theirs {theirs} hcp {lo}-{hi}: " + "; ".join(parts))
            print(f"    BBA without a stopper: {dict(alt.most_common(6))}; "
                  f"NT meanings naming a stopper {stopmention}/{len(alerts)}")
            if a.detail:
                hold = collections.Counter()
                for cls in ("half", "none"):
                    for r, b, _, _ in by.get(cls, []):
                        if is_nt(b):
                            hs = r["hand"].split(".")
                            hold[" ".join(shape_of(hs[SUITS.index(s)]) for s in theirs) + f" {b}"] += 1
                print("    no-stopper NT holdings:", dict(hold.most_common(20)))
                for al, n in collections.Counter(alerts).most_common(4):
                    print(f"    alert x{n}: {al[:200]}")
                # NT by HCP band and class
                band = collections.defaultdict(lambda: [0, 0])
                for cls, xs in by.items():
                    for r, b, _, _ in xs:
                        k = (cls, r["hcp"])
                        band[k][1] += 1
                        band[k][0] += is_nt(b)
                print("    by hcp:", " ".join(f"{c[0]}{h}:{v[0]}/{v[1]}" for (c, h), v in sorted(band.items(), key=lambda kv: (kv[0][1], kv[0][0]))))


if __name__ == "__main__":
    main()
