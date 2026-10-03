#!/usr/bin/env python3
"""Targeted double-dummy test of one decision: deal hands to an auction's
constraints, solve them, and score the candidate contracts per class.

    probes/tools/dd_auction_test.py probes/dd/quant-4nt-hearts.toml
    probes/tools/dd_auction_test.py SPEC.toml --report-only   # re-tally the cache
    probes/tools/dd_auction_test.py SPEC.toml --by N.h --by S.pts
    probes/tools/dd_auction_test.py SPEC.toml --report-only --baseline "6H N"

Rather than re-run the corpus, fix the auction and vary the hands
(Rick, 2026-10-03): which contract should this hand choose, at IMPs and at
matchpoints, both vulnerabilities, double dummy.

The spec (TOML) gives:

  name      = "quant-4nt-hearts"          # cache: .rbb-cache/dd/<name>.jsonl
  dealer    = "hcp(north)>=15 && ..."     # dealer3 condition: a loose prefilter
  north     = "15 <= h <= 17 and nt"      # Python filters, one per seat that
  south     = "L[1] == 5 and 16 <= pts <= 17"   # matters (north/east/south/west)
  where     = "True"                      # optional, over N, E, S, W together
  contracts = ["4NT N", "5H N", "6H N", "6NT N"]   # level, strain, declarer
  baseline  = "4NT N"                     # each candidate is scored against it
  vuls      = ["None", "Both"]            # our side's vulnerability
  quota     = 3000                        # deals wanted in each class
  max_batches = 400
  split     = ["N.h"]                     # default breakdowns (also --by)
  [[class]]                               # first match wins; unmatched: dropped
  label  = "4333, 3 hearts"
  expr   = "N.dist == [4,3,3,3] and N.L[1] == 3"
  dealer = "shape(north, 4333 + 3343 + 3334)"   # optional: extra dealer3 condition
                                               # to fill this class quickly

In the Python expressions a seat (N, E, S, W; or bare names for the seat
being filtered) has: L (lengths, spades first), h (HCP), t (tens), pts (the
engine's total points: HCP + 1/2 a ten + 1 a card beyond four, whole part),
dist (lengths sorted, longest first), nt (balanced: 4333, 4432, 5332),
hand (the PBN string), and has("SA") for a card.

The deals come from dealer3 (../dealer3/target/release/dealer) and are
solved by bridge-solver (../bridge-solver/target/release/bridge-solver); the
tricks are cached, so a new classification or split re-tallies without
solving (--report-only). The report gives, per class and split: deals, the
mean IMPs of each candidate against the baseline with its standard error,
the matchpoint percentage against the baseline (ties half), and the
candidate with the best mean.
"""
import argparse
import json
import math
import os
import re
import subprocess
import sys
import tempfile
import tomllib
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
DEALER = os.environ.get("DEALER3")
SOLVER = os.environ.get("BRIDGE_SOLVER")
HCPV = {"A": 4, "K": 3, "Q": 2, "J": 1}
SEATS = "NESW"
STRAINS = "NSHDC"  # bridge-solver order: NT, S, H, D, C


def sibling(rel):
    # The repo may be a worktree (.claude/worktrees/x): find the GitHub dir.
    d = REPO
    for _ in range(6):
        p = os.path.join(os.path.dirname(d), rel)
        if os.path.exists(p):
            return p
        d = os.path.dirname(d)
    sys.exit(f"cannot find ../{rel}; set DEALER3 / BRIDGE_SOLVER")


class Hand:
    def __init__(self, s):
        self.hand = s
        self.suits = s.split(".")
        self.L = [len(x) for x in self.suits]
        self.h = sum(HCPV.get(c, 0) for c in s)
        self.t = s.count("T")
        self.dist = sorted(self.L, reverse=True)
        self.nt = self.dist in ([4, 3, 3, 3], [4, 4, 3, 2], [5, 3, 3, 2])
        self.pts = (4 * self.h + 2 * self.t + 4 * sum(max(0, l - 4) for l in self.L)) // 4

    def has(self, card):
        return card[1] in self.suits["SHDC".index(card[0])]

    def env(self):
        return {k: getattr(self, k) for k in ("hand", "L", "h", "t", "dist", "nt", "pts")} | {"has": self.has}


def hands_of(deal):
    # "N:a b c d" in seat order from the first seat
    first, rest = deal.split(":")
    hs = rest.split()
    i = SEATS.index(first)
    return {SEATS[(i + k) % 4]: Hand(hs[k]) for k in range(4)}


class Seat:
    def __init__(self, h):
        self.__dict__.update(h.env())


def ctx(hs):
    return {s: Seat(hs[s]) for s in SEATS}


def parse_contract(c):
    m = re.fullmatch(r"([1-7])(NT|N|S|H|D|C)\s+([NESW])", c.strip())
    if not m:
        sys.exit(f"bad contract {c!r}: want e.g. '4NT N'")
    return int(m[1]), m[2][0], m[3]


def score(level, strain, tricks, vul):
    need = level + 6
    if tricks < need:
        return -(100 if vul else 50) * (need - tricks)
    per = 20 if strain in "CD" else 30
    trick_pts = per * level + (10 if strain == "N" else 0)
    s = trick_pts + 50
    if trick_pts >= 100:
        s += 450 if vul else 250
    if level == 6:
        s += 750 if vul else 500
    if level == 7:
        s += 1500 if vul else 1000
    return s + per * (tricks - need)


IMP_T = [20, 50, 90, 130, 170, 220, 270, 320, 370, 430, 500, 600, 750, 900,
         1100, 1300, 1500, 1750, 2000, 2250, 2500, 3000, 3500, 4000]


def imps(d):
    n = sum(1 for v in IMP_T if abs(d) >= v)
    return n if d >= 0 else -n


def load_spec(path):
    with open(path, "rb") as f:
        spec = tomllib.load(f)
    spec.setdefault("where", "True")
    spec.setdefault("vuls", ["None", "Both"])
    spec.setdefault("quota", 2000)
    spec.setdefault("max_batches", 200)
    spec.setdefault("batch", 4000)
    spec.setdefault("split", [])
    spec["classes"] = spec.pop("class")
    return spec


def accept(spec, hs):
    for s, name in zip(SEATS, ("north", "east", "south", "west")):
        if name in spec and not eval(spec[name], {}, hs[s].env()):
            return False
    return bool(eval(spec["where"], {}, ctx(hs)))


def classify(spec, hs):
    c = ctx(hs)
    for k in spec["classes"]:
        if eval(k["expr"], {}, c):
            return k["label"]
    return None


def run_dealer(cond, n, seed):
    script = f"condition\n  {cond}\naction printpbn\n"
    out = subprocess.run([DEALER, "-p", str(n), "-g", str(10**9), "-s", str(seed)], input=script,
                         capture_output=True, text=True, check=True).stdout
    return re.findall(r'\[Deal "([^"]+)"\]', out)


def solve(deals):
    with tempfile.TemporaryDirectory() as d:
        src = os.path.join(d, "in.pbn")
        with open(src, "w") as f:
            for i, x in enumerate(deals):
                f.write(f'[Board "{i + 1}"]\n[Dealer "N"]\n[Vulnerable "None"]\n[Deal "{x}"]\n\n')
        out = subprocess.run([SOLVER, "-i", src], capture_output=True, text=True, check=True).stdout
    res = {}
    for blk in out.split("\n\n"):
        m = re.search(r'\[Deal "([^"]+)"\]', blk)
        t = re.search(r'\[DoubleDummyTricks "([0-9a-dA-D]{20})"\]', blk)
        if m and t:
            res[m[1]] = [int(c, 16) for c in t[1]]
    return res


def tricks(tab, strain, decl):
    return tab["NSEW".index(decl) * 5 + STRAINS.index(strain)]


def generate(spec, cache_path):
    have = defaultdict(int)
    seen = set()
    if os.path.exists(cache_path):
        for line in open(cache_path):
            r = json.loads(line)
            seen.add(r["deal"])
            hs = hands_of(r["deal"])
            if accept(spec, hs):
                c = classify(spec, hs)
                if c:
                    have[c] += 1
    q = spec["quota"]
    seed = spec.get("seed", 1) * 100003 + len(seen)
    for b in range(spec["max_batches"]):
        short = [k for k in spec["classes"] if have[k["label"]] < q]
        if not short:
            break
        k = short[0]
        cond = spec["dealer"] + (f" && ({k['dealer']})" if k.get("dealer") else "")
        todo = []
        for x in run_dealer(cond, spec["batch"], seed + b):
            if x in seen:
                continue
            hs = hands_of(x)
            if not accept(spec, hs):
                continue
            c = classify(spec, hs)
            if c and have[c] < q:
                have[c] += 1
                seen.add(x)
                todo.append(x)
        print(f"batch {b}: {k['label']}: +{len(todo)} to solve; "
              + ", ".join(f"{kk['label']} {have[kk['label']]}" for kk in spec["classes"]),
              file=sys.stderr)
        if not todo:
            continue
        sol = solve(todo)
        with open(cache_path, "a") as f:
            for x in todo:
                if x in sol:
                    f.write(json.dumps({"deal": x, "dd": sol[x]}) + "\n")


def mean_se(v):
    n = len(v)
    if n == 0:
        return 0.0, 0.0
    m = sum(v) / n
    if n < 2:
        return m, 0.0
    var = sum((x - m) ** 2 for x in v) / (n - 1)
    return m, math.sqrt(var / n)


def report(spec, cache_path, splits, as_json=None):
    cons = [(c, parse_contract(c)) for c in spec["contracts"]]
    base = parse_contract(spec["baseline"])
    rows = defaultdict(list)  # (class, split values) -> list of per-deal dicts
    for line in open(cache_path):
        r = json.loads(line)
        hs = hands_of(r["deal"])
        if not accept(spec, hs):
            continue
        c = classify(spec, hs)
        if not c:
            continue
        cx = ctx(hs)
        key = (c,) + tuple(eval(s, {}, cx) for s in splits)
        rows[key].append(r["dd"])
    order = {k["label"]: i for i, k in enumerate(spec["classes"])}
    out = []
    for vul in spec["vuls"]:
        v = vul in ("Both", "All", "NS")
        print(f"\n== vulnerable: {vul}; IMPs and MP% of each contract against {spec['baseline']} ==")
        hdr = f"{'class':28} {' '.join(f'{s:>9}' for s in splits)} {'deals':>6}"
        for c, _ in cons:
            if parse_contract(c) != base:
                hdr += f" | {c:>16} {'MP%':>4}"
        print(hdr + " | best")
        for key in sorted(rows, key=lambda k: (order[k[0]],) + tuple(k[1:])):
            dds = rows[key]
            line = f"{key[0]:28} {' '.join(f'{str(x):>9}' for x in key[1:])} {len(dds):>6}"
            means = {spec["baseline"]: 0.0}
            rec = {"vul": vul, "class": key[0], "split": dict(zip(splits, key[1:])), "deals": len(dds), "vs_baseline": {}}
            for c, (lv, st, de) in cons:
                if (lv, st, de) == base:
                    continue
                diffs, mp = [], 0.0
                for dd in dds:
                    a = score(lv, st, tricks(dd, st, de), v)
                    b = score(base[0], base[1], tricks(dd, base[1], base[2]), v)
                    diffs.append(imps(a - b))
                    mp += 1.0 if a > b else 0.5 if a == b else 0.0
                m, se = mean_se(diffs)
                means[c] = m
                pct = 100 * mp / len(dds)
                rec["vs_baseline"][c] = {"imps": round(m, 3), "se": round(se, 3), "mp_pct": round(pct, 1)}
                line += f" | {m:+7.2f} ±{se:4.2f}    {pct:4.0f}"
            best = max(means, key=means.get)
            rec["best"] = best
            out.append(rec)
            print(line + f" | {best}")
    if as_json:
        with open(as_json, "w") as f:
            json.dump(out, f, indent=1)


def main():
    global DEALER, SOLVER
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("spec")
    ap.add_argument("--report-only", action="store_true", help="tally the cache, solve nothing")
    ap.add_argument("--by", action="append", help="split expression (replaces the spec's split)")
    ap.add_argument("--quota", type=int, help="override the spec's quota")
    ap.add_argument("--cache", help="tricks cache (default .rbb-cache/dd/<name>.jsonl)")
    ap.add_argument("--json", help="write the table as JSON")
    ap.add_argument("--baseline", help="score against this contract instead (e.g. '6H N')")
    a = ap.parse_args()
    spec = load_spec(a.spec)
    if a.baseline:
        spec["baseline"] = a.baseline
        if a.baseline not in spec["contracts"]:
            spec["contracts"].append(a.baseline)
    if a.quota:
        spec["quota"] = a.quota
    cache = a.cache or os.path.join(REPO, ".rbb-cache", "dd", spec["name"] + ".jsonl")
    os.makedirs(os.path.dirname(cache), exist_ok=True)
    DEALER = DEALER or sibling("dealer3/target/release/dealer")
    SOLVER = SOLVER or sibling("bridge-solver/target/release/bridge-solver")
    if not a.report_only:
        generate(spec, cache)
    report(spec, cache, a.by if a.by is not None else spec["split"], a.json)


if __name__ == "__main__":
    main()
