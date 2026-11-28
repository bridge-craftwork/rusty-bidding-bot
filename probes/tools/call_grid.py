#!/usr/bin/env python3
"""The call grid: every legal call at every position of our own uncontested
tree, with or without a meaning; one level of interference described; the
common holes cross-checked against GIB, Brill and WBridge5.

    probes/tools/call_grid.py walk  [--card C] [--depth 4] [--jobs N]
    probes/tools/call_grid.py comp  [--card C] [--jobs N]
    probes/tools/call_grid.py weigh                    # real-auction frequencies
    probes/tools/call_grid.py cross [--top 60]         # GIB / Brill / WBridge5
    probes/tools/call_grid.py covers                   # in-flight pack branches
    probes/tools/call_grid.py report [--out FILE]

Rick (2026-11-27): fill in the grid; all competitive auctions would get out
of hand, so only a general description there; every system has holes, but
cross-check the common ones against Brill, WBridge5 and GIB to see whether
we miss anything basic.

**walk.** Breadth-first from the opening seat (dealer N, auction "") and the
second seat (dealer E, auction "Pass"), opponents passing throughout. At each
of our positions with fewer than --depth non-pass calls, `rbb bid rules
--json` lists the calls some rule can make there (the engine's own
candidates, before the hand is known); every other legal call up to 7NT is
undefined. Children: the defined bids. Our pass ends the auction once anyone
has bid; before that it hands the opening to partner.

**comp.** One level of interference, not a tree: responder's call over each
of our openings and their simple actions (double, a simple or jump overcall,
1NT, 2NT, a cue bid), and our first call over each of their openings, in
the direct and the pass-out seat.

**weigh.** Each grid position and call counted in real auctions
(.rbb-cache/undefined-calls/auctions.jsonl: the Brill field, the BBA and
GIB corpora, from undefined_calls.py collect) and in WBridge5's own auctions
(scratchpad PBNs bid with WBridge5 at all four seats, wbridge5_bid.py).

**cross.** For the common holes and every hole in rounds 1-2: GIB's tree
(the archive, fetched once if missing; one request a second), Brill's
/explain (Thorvald's service, the same courtesy and cache as
brill_probe.py), and whether WBridge5 made the call there.

Read-only on rules. Cache: .rbb-cache/call-grid/ of the main checkout.
Standard library only.
"""

import argparse
import collections
import concurrent.futures as cf
import glob
import hashlib
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
sys.path.insert(0, os.path.dirname(__file__))
import undefined_calls as uc  # noqa: E402

# gib_tree.py lives on the bidding branches (main checkout), not on main yet
sys.path.append(os.path.join(uc.MAIN, "probes", "tools"))

MAIN = uc.MAIN
OUT = os.path.join(MAIN, ".rbb-cache", "call-grid")
RBB = os.path.join(MAIN, "target", "release", "rbb")
SCRATCH = "/private/tmp/claude-501/-Volumes-Express2T-Development-GitHub-rusty-bidding-bot/610ae047-7839-4455-a45b-2f6bfdda781e/scratchpad"
PACK = os.path.join(SCRATCH, "pack-bottom")
WB5 = [os.path.join(SCRATCH, "wb", "pbs", "bba", "Random_Pavlicek_WB5.pbn"),
       os.path.join(SCRATCH, "wb", "slam", "bba", "Brill_Slams_WB5.pbn")]
STRAINS = ["C", "D", "H", "S", "NT"]
BIDS = [f"{l}{s}" for l in range(1, 8) for s in STRAINS]
RANK = {b: i for i, b in enumerate(BIDS)}
SUITS = "CDHS"
CARDS = {"Brill": os.path.join(MAIN, "cards", "json", "Brill.json"), "21GF-GIB": "21GF-GIB"}


def card_arg(name):
    return CARDS.get(name, name)


def last_bid(calls):
    for c in reversed(calls):
        if c in RANK:
            return c
    return None


def legal(calls):
    """Legal calls after `calls` (dealer-relative), up to 7NT."""
    lb = last_bid(calls)
    out = ["Pass"] + [b for b in BIDS if lb is None or RANK[b] > RANK[lb]]
    # X / XX: the last non-pass call by an opponent of the caller
    n = len(calls)
    for i in range(n - 1, -1, -1):
        c = calls[i]
        if c == "Pass":
            continue
        opp = (n - i) % 2 == 1
        if opp and c in RANK:
            out.append("X")
        if opp and c == "X":
            out.append("XX")
        break
    return out


def ask(card, rules, dealer, calls):
    a = " ".join(calls)
    p = subprocess.run([RBB, "bid", "rules", "-a", a, "-d", dealer, "-c", card_arg(card),
                        "--rules", rules, "--json"], capture_output=True, text=True)
    if p.returncode != 0:
        return {"error": p.stderr.strip()[:300]}
    d = json.loads(p.stdout)
    defined = {}
    for r in d.get("candidates", []):
        c = r["call"]
        c = {"P": "Pass"}.get(c, c)
        if c.endswith("N") and not c.endswith("NT"):
            c = c + "T"
        loc = f"{r['rule']['module']}:{r['rule']['line']}"
        if c not in defined:
            defined[c] = [(r.get("explanation") or "")[:90], loc, bool(r.get("artificial"))]
    return {"forced": d.get("forced"), "defined": defined}


def nonpass(calls):
    return sum(1 for c in calls if c != "Pass")


# ------------------------------------------------------------------ walk

def walk(args):
    os.makedirs(OUT, exist_ok=True)
    path = os.path.join(OUT, f"grid-{args.card}.jsonl")
    done = {}
    if os.path.exists(path):
        for l in open(path):
            r = json.loads(l)
            done[(r["dealer"], tuple(r["calls"]))] = r
    frontier = [("N", ()), ("E", ("Pass",))]
    level = 0
    fo = open(path, "a")
    while frontier:
        todo = [p for p in frontier if p not in done]
        t0 = time.time()
        with cf.ThreadPoolExecutor(args.jobs) as ex:
            futs = {ex.submit(ask, args.card, args.rules, d, list(c)): (d, c) for d, c in todo}
            for f in cf.as_completed(futs):
                d, c = futs[f]
                res = f.result()
                r = {"dealer": d, "calls": list(c), "n": nonpass(c), "legal": legal(list(c)), **res}
                done[(d, c)] = r
                fo.write(json.dumps(r) + "\n")
        fo.flush()
        print(f"level {level}: {len(frontier)} positions ({len(todo)} asked, {time.time() - t0:.0f}s)",
              file=sys.stderr, flush=True)
        nxt = []
        for d, c in frontier:
            r = done[(d, c)]
            if "defined" not in r:
                continue
            for call in r["defined"]:
                if call == "Pass":
                    if nonpass(c) == 0 and len(c) < 2:
                        # pass hands the opening to partner (opponent passes too)
                        nxt.append((d, c + ("Pass", "Pass")))
                    continue
                if call not in RANK:
                    continue
                cc = c + (call, "Pass")
                if nonpass(cc) < args.depth:
                    nxt.append((d, cc))
        frontier = sorted(set(nxt))
        level += 1
    fo.close()


# ------------------------------------------------------------------ comp

def comp_positions():
    """(dealer, calls, label): one level of interference."""
    pos = []
    ours = ["1C", "1D", "1H", "1S", "1NT", "2C", "2D", "2H", "2S"]
    for o in ours:
        actions = ["X"]
        lo = RANK[o]
        if o in ("1C", "1D", "1H", "1S"):
            suit = o[1]
            for s in SUITS:
                if s == suit:
                    continue
                simple = next(b for b in BIDS if b[-1] == s and RANK[b] > lo)
                actions.append(simple)
                jump = f"{int(simple[0]) + 1}{s}"
                if jump in RANK and int(jump[0]) <= 3:
                    actions.append(jump)
            actions += ["1NT" if RANK["1NT"] > lo else None, "2NT", f"2{suit}"]
        elif o == "1NT":
            actions += ["2C", "2D", "2H", "2S", "3C", "3D"]
        else:  # strong 2C, weak twos
            for s in SUITS:
                b = next(x for x in BIDS if x[-1] == s and RANK[x] > lo)
                actions.append(b)
            actions += ["2NT", "3NT"]
        for a in [x for x in actions if x]:
            pos.append(("N", [o, a], f"our {o}, their {a}"))
    theirs = ["1C", "1D", "1H", "1S", "1NT", "2C", "2D", "2H", "2S", "3C", "3D", "3H", "3S", "4H", "4S"]
    for t in theirs:
        pos.append(("W", [t], f"their {t}, direct seat"))
        if t[0] == "1":
            pos.append(("W", [t, "Pass", "Pass"], f"their {t}, pass-out seat"))
    # advancer's first call over the most common: partner overcalls / doubles their 1x
    for t, a in [("1C", "1H"), ("1D", "1S"), ("1H", "1S"), ("1H", "2C"), ("1S", "2H"), ("1C", "X"),
                 ("1D", "X"), ("1H", "X"), ("1S", "X"), ("1NT", "X"), ("1NT", "2H"), ("2H", "2S"), ("2S", "X")]:
        pos.append(("W", [t, a, "Pass"], f"their {t}, partner {a}, advancer"))
    return pos


def comp(args):
    os.makedirs(OUT, exist_ok=True)
    pos = comp_positions()
    rows = []
    with cf.ThreadPoolExecutor(args.jobs) as ex:
        futs = {ex.submit(ask, args.card, args.rules, d, c): (d, c, l) for d, c, l in pos}
        for f in cf.as_completed(futs):
            d, c, l = futs[f]
            rows.append({"dealer": d, "calls": c, "label": l, "legal": legal(c), **f.result()})
    rows.sort(key=lambda r: [r["dealer"]] + r["calls"])
    with open(os.path.join(OUT, f"comp-{args.card}.json"), "w") as fo:
        json.dump(rows, fo, indent=0)
    print(f"{len(rows)} competitive positions", file=sys.stderr)


# ------------------------------------------------------------------ weigh

def opening_side_uncontested(calls):
    """Length of the longest prefix of `calls` in which the side that opened
    is the only one bidding (all of the other side's calls are passes)."""
    first = next((i for i, c in enumerate(calls) if c != "Pass"), None)
    if first is None:
        return len(calls)
    side = first % 2
    for i, c in enumerate(calls):
        if i % 2 != side and c != "Pass":
            return i
    return len(calls)


def weigh(args):
    pos = collections.defaultdict(collections.Counter)   # calls tuple -> src -> reached
    made = collections.defaultdict(collections.Counter)  # (calls tuple, call) -> src
    srcs = []
    for a in uc.load_auctions():
        srcs.append((a["src"], a["calls"]))
    for f in WB5:
        if os.path.exists(f):
            for g in uc.parse_pbn(f):
                srcs.append(("wb5", g["calls"]))
    for src, calls in srcs:
        k = opening_side_uncontested(calls)
        n = 0
        for i in range(min(k, len(calls)) + 1):
            pre = tuple(calls[:i])
            if nonpass(pre) >= 5:
                break
            pos[pre][src] += 1
            if i < len(calls) and i < k:
                made[(pre, calls[i])][src] += 1
        # competitive: just the first two calls after an opening (one level)
        first = next((i for i, c in enumerate(calls) if c != "Pass"), None)
        if first is not None:
            for j in range(first + 1, min(first + 4, len(calls))):
                pre = tuple(calls[:j])
                pos[("comp",) + pre][src] += 1
                made[(("comp",) + pre, calls[j])][src] += 1
    with open(os.path.join(OUT, "weights.json"), "w") as fo:
        json.dump({"pos": {" ".join(k): v for k, v in pos.items()},
                   "made": {" ".join(k[0]) + " | " + k[1]: v for k, v in made.items()}}, fo)
    print(f"{len(pos)} positions, {len(made)} (position, call) pairs", file=sys.stderr)


def load_weights():
    w = json.load(open(os.path.join(OUT, "weights.json")))
    return w["pos"], w["made"]


# ------------------------------------------------------------- classify

def bidder_suits(calls, seat_side):
    """Suits bid by each partner of the side whose turn it is: (mine, partner's)."""
    n = len(calls)
    mine, pard = [], []
    for i, c in enumerate(calls):
        if c not in RANK or c.endswith("NT"):
            continue
        rel = (n - i) % 4
        if rel == 0:
            mine.append(c[-1])
        elif rel == 2:
            pard.append(c[-1])
    return mine, pard


def call_type(calls, call):
    """General description of a call in an uncontested auction."""
    if call == "Pass":
        return "pass"
    lb = last_bid(calls)
    if call in ("4NT", "5NT"):
        return "4NT/5NT"
    if int(call[0]) >= 6:
        return "slam level"
    mine, pard = bidder_suits(calls, None)
    if lb is None:
        lvl = int(call[0])
        return f"opening {lvl}-level" if lvl <= 3 else "opening 4-level+"
    # how far above the cheapest bid in that strain
    strain = call[1:] if call.endswith("NT") else call[-1]
    cheapest = next(b for b in BIDS if (b[1:] if b.endswith("NT") else b[-1]) == strain and RANK[b] > RANK[lb])
    jump = int(call[0]) - int(cheapest[0])
    jl = ["", "jump ", "double jump ", "triple jump "][min(jump, 3)]
    if strain == "NT":
        return f"{jl}NT".strip() if jump else "cheapest NT"
    used = set(mine) | set(pard)
    if strain in pard and strain not in mine:
        kind = "raise of partner's suit"
    elif strain in mine:
        kind = "rebid of own suit"
    elif len(used) == 3:
        kind = "fourth suit"
    else:
        kind = "new suit"
    return f"{jl}{kind}" if jump else f"simple {kind}"


# ----------------------------------------------------------------- cross

def brill_explain(auction_calls, fetcher_state):
    """Brill's reading of the last call; cached like brill_probe.py."""
    import brill_probe as bp
    ba = bp.to_brill(" ".join(auction_calls))
    ed = os.path.join(MAIN, ".rbb-cache", "brill-service", "explain")
    os.makedirs(ed, exist_ok=True)
    p = os.path.join(ed, bp.sha(ba) + ".json")
    if os.path.exists(p):
        data = json.load(open(p))
    else:
        if fetcher_state["fails"] >= 5 or fetcher_state["count"] >= fetcher_state["limit"]:
            return None
        wait = fetcher_state["last"] + 1.0 - time.time()
        if wait > 0:
            time.sleep(wait)
        fetcher_state["last"] = time.time()
        try:
            raw = bp.fetch_one(ba)
            data = json.loads(raw)
            with open(p + ".tmp", "wb") as f:
                f.write(raw)
            os.rename(p + ".tmp", p)
            fetcher_state["fails"] = 0
            fetcher_state["count"] += 1
        except (urllib.error.URLError, ValueError, TimeoutError, OSError) as e:
            fetcher_state["fails"] += 1
            print(f"brill error {ba}: {str(e)[:100]}", file=sys.stderr)
            return None
    if not isinstance(data, list) or not data:
        return None
    last = data[-1]
    ex = (last.get("explanation") or [{}])
    e = ex[0] if ex else {}
    inst = e.get("explainingRuleInstance") or {}
    return {"system": bool(e.get("isSystemExplained")), "set": e.get("explainingRuleSetName"),
            "means": inst.get("resolvedMeans") or "", "fallback": bool(inst.get("isFallbackOnly")),
            "requires": (inst.get("resolvedRequiresString") or "")[:160]}


def gib_meaning(calls, call, fetcher):
    import gib_tree as gt
    t = fetcher.get([gt.gib_call(c) for c in calls])
    if t is None:
        return None
    g = {"Pass": "P", "X": "D", "XX": "R"}.get(call, call.replace("NT", "N"))
    return t.get(g)


def holes(card):
    rows = [json.loads(l) for l in open(os.path.join(OUT, f"grid-{card}.jsonl"))]
    out = []
    for r in rows:
        if "defined" not in r:
            continue
        for c in r["legal"]:
            if c in r["defined"]:
                continue
            if c == "Pass" and r.get("forced"):
                continue
            out.append((r, c))
    return rows, out


def score(pos, made, calls, call):
    k = " ".join(calls)
    m = made.get(k + " | " + call, {})
    p = pos.get(k, {})
    return (2 * m.get("brill", 0) + m.get("gib", 0) + m.get("bba", 0) + m.get("wb5", 0),
            2 * p.get("brill", 0) + p.get("gib", 0) + p.get("bba", 0) + p.get("wb5", 0))


def comp_holes(card, pos):
    """Competitive holes through 3NT (with X/XX) at positions the Brill field reaches."""
    rows = json.load(open(os.path.join(OUT, f"comp-{card}.json")))
    out = []
    for r in rows:
        if "defined" not in r:
            continue
        if pos.get("comp " + " ".join(r["calls"]), {}).get("brill", 0) < 10:
            continue
        for c in r["legal"]:
            if c == "Pass" or c in r["defined"]:
                continue
            if c in ("X", "XX") or RANK[c] <= RANK["3NT"]:
                rr = dict(r, n=nonpass(r["calls"]))
                out.append((rr, c))
    return out


def cross(args):
    import gib_tree as gt
    pos, made = load_weights()
    if args.comp:
        return cross_comp(args, pos, made, gt)
    _, hs = holes(args.card)
    hs = [(r, c) for r, c in hs if c != "Pass" and (RANK.get(c, 99) <= RANK["5NT"])]
    ranked = sorted(hs, key=lambda rc: score(pos, made, rc[0]["calls"], rc[1]), reverse=True)
    pick = []
    seen = set()
    for r, c in ranked[: args.top]:
        pick.append((r, c, "top"))
        seen.add((tuple(r["calls"]), c))
    for r, c in hs:
        first_seat = not r["calls"] or r["calls"][0] != "Pass"
        if (r["n"] <= 1 and first_seat and (tuple(r["calls"]), c) not in seen
                and RANK.get(c, 99) <= RANK[args.round12_max]):
            pick.append((r, c, "round 1-2"))
            seen.add((tuple(r["calls"]), c))
    print(f"{len(pick)} holes to cross-check", file=sys.stderr)
    gf = gt.Fetcher()
    bstate = {"last": 0.0, "fails": 0, "count": 0, "limit": args.brill_limit}
    res = []
    for i, (r, c, why) in enumerate(pick):
        calls = r["calls"]
        gm = gib_meaning(calls, c, gf)
        bm = brill_explain(calls + [c], bstate)
        k = " ".join(calls)
        m = made.get(k + " | " + c, {})
        res.append({"dealer": r["dealer"], "calls": calls, "call": c, "why": why, "n": r["n"], "type": call_type(calls, c),
                    "score": score(pos, made, calls, c), "gib": gm, "brill": bm,
                    "wb5_made": m.get("wb5", 0), "made": m, "reached": pos.get(k, {})})
        if i % 25 == 0:
            print(f"  {i}/{len(pick)} (brill fetched {bstate['count']})", file=sys.stderr, flush=True)
    with open(os.path.join(OUT, f"cross-{args.card}.json"), "w") as fo:
        json.dump(res, fo, indent=0)


def cross_comp(args, pos, made, gt):
    hs = comp_holes(args.card, pos)
    print(f"{len(hs)} competitive holes to cross-check", file=sys.stderr)
    gf = gt.Fetcher()
    bstate = {"last": 0.0, "fails": 0, "count": 0, "limit": args.brill_limit}
    res = []
    for i, (r, c) in enumerate(hs):
        calls = r["calls"]
        k = "comp " + " ".join(calls)
        res.append({"dealer": r["dealer"], "calls": calls, "call": c, "label": r["label"],
                    "gib": gib_meaning(calls, c, gf), "brill": brill_explain(calls + [c], bstate),
                    "made": made.get(k + " | " + c, {}), "reached": pos.get(k, {})})
        if i % 25 == 0:
            print(f"  {i}/{len(hs)} (brill fetched {bstate['count']})", file=sys.stderr, flush=True)
    with open(os.path.join(OUT, f"cross-comp-{args.card}.json"), "w") as fo:
        json.dump(res, fo, indent=0)


# ---------------------------------------------------------------- covers

def covers(args):
    """Ask the in-flight pack branches about the cross-checked holes."""
    pack = os.path.join(os.path.dirname(MAIN), "bridge-common-conventions")
    res = json.load(open(os.path.join(OUT, f"cross-{args.card}.json")))
    branches = args.branches.split(",")
    out = {}

    def one(b, dealer, calls, call):
        p = subprocess.run([RBB, "bid", "rules", "-a", " ".join(calls), "-d", dealer,
                            "-c", card_arg(args.card), "--rules", pack, "--rules-at", f"origin/{b}",
                            "--call", call, "--json"], capture_output=True, text=True)
        if p.returncode != 0:
            return None
        d = json.loads(p.stdout)
        return [f"{x['rule']['module']}:{x['rule']['line']}" for x in d.get("candidates", [])]

    jobs = []
    for r in res:
        for b in branches:
            jobs.append((b, r["dealer"], tuple(r["calls"]), r["call"]))
    with cf.ThreadPoolExecutor(args.jobs) as ex:
        futs = {ex.submit(one, b, d, list(c), call): (b, c, call) for b, d, c, call in jobs}
        for f in cf.as_completed(futs):
            b, c, call = futs[f]
            v = f.result()
            if v:
                out.setdefault(" ".join(c) + " | " + call, {})[b] = v
    with open(os.path.join(OUT, f"covers-{args.card}.json"), "w") as fo:
        json.dump(out, fo, indent=0)
    print(f"{len(out)} holes covered by a branch", file=sys.stderr)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("walk", "comp", "cross", "covers", "report"):
        s = sub.add_parser(name)
        s.add_argument("--card", default="Brill")
        s.add_argument("--rules", default=PACK)
        s.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 4) - 2))
        if name == "walk":
            s.add_argument("--depth", type=int, default=4)
        if name == "cross":
            s.add_argument("--top", type=int, default=60)
            s.add_argument("--round12-max", default="4S")
            s.add_argument("--brill-limit", type=int, default=400)
            s.add_argument("--comp", action="store_true", help="the one-level competitive holes instead")
        if name == "covers":
            s.add_argument("--branches", default="gf-after-2nt,second-calls-defence,second-calls-opener")
        if name == "report":
            s.add_argument("--out", default=os.path.join(ROOT, "docs", "reviews", "2026-11-27-call-grid.md"))
    sub.add_parser("weigh")
    a = ap.parse_args()
    if a.cmd == "report":
        import call_grid_report
        call_grid_report.report(a)
    else:
        {"walk": walk, "comp": comp, "weigh": weigh, "cross": cross, "covers": covers}[a.cmd](a)


if __name__ == "__main__":
    main()
