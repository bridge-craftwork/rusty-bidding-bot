#!/usr/bin/env python3
"""A reproducible sample of Richard Pavlicek's solved random deals, as PBN.

    probes/tools/rpdd_sample.py --zrd ../rpdd-library/rpdd.zrd \\
        --count 10000 -o SCRATCH/rpdd-10k.pbn [--offset 0] [--event NAME]

The library (10,485,760 random deals, each with its 20-cell double-dummy
table) is (c) 2007 Richard Pavlicek, rpbridge.net, free for noncommercial
use, unmodified, credited to him. Its data is never committed here: the
output belongs in a scratch directory outside the repo. See
docs/random-deals-comparison.md.

`rpdd.zrd` is his file as `rpdd.bat` builds it from `rpdd.zip` (or
`rpdd zrd 0 10485760 --zdd rpdd.zdd` from the rpdd-reader crate). Record
i is 23 bytes at offset 23*i: 13 bytes of deal (two bits a card, seat
00 W 01 N 10 E 11 S, cards SA SK .. S2 then H, D, C, least significant
bits first) and 10 of table (strains NT S H D C, two bytes each, seats
W N E S, a nibble each, low nibble first). Layout as documented in
bridge-encodings/src/zrd.

The selection is evenly spaced: deal k of the sample is library index
floor(k * 10485760 / count) + offset, so it covers the whole library and
the same arguments always give the same deals. Board k+1 gets the
standard board rotation's dealer and vulnerability. Each board carries
`[RpddIndex "i"]` for traceability and the table as an
`[OptimumResultTable]`, which `rbb compare` reads, so par costs nothing.
"""
import argparse
import os
import sys

TOTAL = 10_485_760
RECORD = 23
RANKS = "AKQJT98765432"
SEATS = "WNES"  # the record's two-bit seat codes
STRAINS = ["NT", "S", "H", "D", "C"]  # the table's strain order
DEALERS = "NESW"
# Board 1..16 vulnerability, the standard rotation.
VULS = ["None", "NS", "EW", "All", "NS", "EW", "All", "None",
        "EW", "All", "None", "NS", "All", "None", "NS", "EW"]


def decode(rec):
    """(hands {seat: [suit strings S,H,D,C]}, tricks {(seat, strain): n})."""
    hands = {s: [[], [], [], []] for s in SEATS}
    count = {s: 0 for s in SEATS}
    for j in range(52):
        seat = SEATS[(rec[j >> 2] >> (2 * (j & 3))) & 3]
        hands[seat][j // 13].append(RANKS[j % 13])
        count[seat] += 1
    if any(c != 13 for c in count.values()):
        raise ValueError(f"not a deal: {count}")
    tricks = {}
    for si, strain in enumerate(STRAINS):
        for k, seat in enumerate(SEATS):
            b = rec[13 + si * 2 + k // 2]
            t = b & 0x0F if k % 2 == 0 else b >> 4
            if t > 13:
                raise ValueError(f"{seat} {strain}: {t} tricks")
            tricks[(seat, strain)] = t
    return {s: ["".join(x) for x in h] for s, h in hands.items()}, tricks


def board_text(board, index, hands, tricks, event, scoring):
    deal = "N:" + " ".join(".".join(hands[s]) for s in "NESW")
    lines = [
        f'[Event "{event}"]',
        '[Site "-"]',
        f'[Board "{board}"]',
        f'[Dealer "{DEALERS[(board - 1) % 4]}"]',
        f'[Vulnerable "{VULS[(board - 1) % 16]}"]',
        f'[Deal "{deal}"]',
        f'[RpddIndex "{index}"]',
        f'[Scoring "{scoring}"]',
        '[OptimumResultTable "Declarer;Denomination\\2R;Result\\2R"]',
    ]
    for seat in "NSEW":
        for strain in STRAINS:
            lines.append(f"{seat} {strain:>2} {tricks[(seat, strain)]:>2}")
    return "\n".join(lines) + "\n\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--zrd", required=True, help="Pavlicek's rpdd.zrd")
    ap.add_argument("--count", type=int, default=10_000)
    ap.add_argument("--offset", type=int, default=0,
                    help="shift every index (0 <= offset < 10485760 // count), for a disjoint sample")
    ap.add_argument("--event", default="Random_Pavlicek")
    ap.add_argument("--scoring", default="MP", choices=["MP", "IMP"],
                    help="[Scoring] for every board (the PBS corpus is MP)")
    ap.add_argument("-o", "--output", required=True)
    a = ap.parse_args()
    size = os.path.getsize(a.zrd)
    if size != TOTAL * RECORD:
        sys.exit(f"{a.zrd}: {size} bytes, expected {TOTAL * RECORD} (the full library)")
    with open(a.zrd, "rb") as f, open(a.output, "w") as out:
        out.write("% PBN 2.1\n% Deals and double-dummy tables: Richard Pavlicek's "
                  "10,485,760 random solved deals (c) 2007, rpbridge.net. "
                  "Noncommercial use; do not republish.\n")
        out.write(f"% rpdd_sample.py --count {a.count} --offset {a.offset} --scoring {a.scoring}\n")
        for k in range(a.count):
            index = k * TOTAL // a.count + a.offset
            f.seek(index * RECORD)
            hands, tricks = decode(f.read(RECORD))
            out.write(board_text(k + 1, index, hands, tricks, a.event, a.scoring))


if __name__ == "__main__":
    main()
