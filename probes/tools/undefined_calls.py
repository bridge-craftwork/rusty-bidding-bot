#!/usr/bin/env python3
"""Which low-level calls does our engine have no reading for?

    probes/tools/undefined_calls.py collect          # auctions -> positions (cache)
    probes/tools/undefined_calls.py ask [--card C]   # the engine's readings (rbb call --batch)
    probes/tools/undefined_calls.py report [--out FILE]

Rick (2026-11-27): most low-level auctions have a meaning for every bid; probe
which ones Rusty cannot read. Thorvald's example, 1S-1NT-2NT-3H, was the
fourth non-pass call.

Positions come from real auctions, not our self-play:
- the Brill field PBNs (.rbb-cache/brill/<day>/*field.pbn of the main
  checkout, every table; North is Rusty's partner, a Brill robot; East and
  West are Brill robots; South is the entrant), with each call's Brill note;
- the BBA corpus (../Practice-Bidding-Scenarios/bba/*.pbn, bba-cli);
- the GIB corpus (../Practice-Bidding-Scenarios/GIB/*);
- the calls GIB's tree defines at those positions, where the tree is archived
  (.rbb-cache/gib-tree/xml/, gib_tree.py; nothing new is fetched).

Every call up to and including the fourth non-pass call is checked. The
engine is asked once per distinct auction (`rbb call --batch`, the hand to
act next): `auction.position.unread` says which calls no rule reads. The
decision itself, for South in the Brill field, says where our engine has no
rule at all for the hand ("No rule applies": a pass made by no rule).

Read-only: rules from the pack at a git ref (default origin/bottom-review of
../bridge-common-conventions). Standard library only.
"""

import argparse
import collections
import glob
import json
import os
import re
import subprocess
import sys
import threading

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def main_checkout():
    try:
        g = subprocess.run(["git", "-C", ROOT, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                           capture_output=True, text=True, check=True).stdout.strip()
        return os.path.dirname(g)
    except (subprocess.CalledProcessError, OSError):
        return ROOT


MAIN = main_checkout()
PBS = os.path.join(os.path.dirname(MAIN), "Practice-Bidding-Scenarios")
PACK = os.path.join(os.path.dirname(MAIN), "bridge-common-conventions")
OUT = os.path.join(MAIN, ".rbb-cache", "undefined-calls")
GIBXML = os.path.join(MAIN, ".rbb-cache", "gib-tree", "xml")
SEATS = "NESW"
MAXN = 4

# ------------------------------------------------------------------ PBN


def norm_call(t):
    t = t.strip()
    u = t.upper()
    if u in ("PASS", "P"):
        return "Pass"
    if u in ("X", "D", "DBL"):
        return "X"
    if u in ("XX", "R", "RDBL"):
        return "XX"
    m = re.fullmatch(r"([1-7])(C|D|H|S|N|NT)", u)
    if m:
        return m.group(1) + ("NT" if m.group(2) in ("N", "NT") else m.group(2))
    return None


def parse_pbn(path):
    """Games: {dealer, vul, deal {seat: hand}, calls [..], notes [..]}."""
    text = open(path, encoding="utf-8", errors="replace").read()
    games, cur, last_deal = [], None, None
    in_auction = False
    note_map = {}
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("["):
            in_auction = False
            m = re.match(r'\[(\w+) "(.*)"\]', s)
            if not m:
                continue
            tag, val = m.groups()
            if tag in ("Event", "Board") and cur is not None and cur.get("calls") is not None:
                games.append(cur)
                cur = None
            if cur is None:
                cur = {"notes_raw": {}}
                note_map = cur["notes_raw"]
            if tag == "Dealer":
                cur["dealer"] = val
            elif tag == "Vulnerable":
                cur["vul"] = {"Love": "None", "-": "None", "Both": "All"}.get(val, val)
            elif tag == "Deal":
                if val == "#":
                    val = last_deal
                last_deal = val
                cur["deal_raw"] = val
            elif tag == "Auction":
                cur["calls"], cur["callnotes"] = [], []
                in_auction = True
            elif tag == "Note":
                mm = re.match(r"(\d+):(.*)", val)
                if mm:
                    note_map[mm.group(1)] = mm.group(2)
            continue
        if in_auction and s and not s.startswith("%") and not s.startswith("{"):
            for tok in s.split():
                mm = re.fullmatch(r"=(\d+)=", tok)
                if mm:
                    if cur["callnotes"]:
                        cur["callnotes"][-1] = mm.group(1)
                    continue
                if tok.upper() == "AP":
                    cur["calls"] += ["Pass"] * 3
                    cur["callnotes"] += [None] * 3
                    continue
                c = norm_call(tok)
                if c is None:
                    continue
                cur["calls"].append(c)
                cur["callnotes"].append(None)
    if cur is not None and cur.get("calls") is not None:
        games.append(cur)
    out = []
    for g in games:
        if not g.get("calls") or "deal_raw" not in g or "dealer" not in g:
            continue
        first, rest = g["deal_raw"].split(":")
        hs = rest.split()
        if len(hs) != 4:
            continue
        i = SEATS.index(first)
        deal = {SEATS[(i + k) % 4]: hs[k] for k in range(4)}
        notes = [g["notes_raw"].get(n) if n else None for n in g["callnotes"]]
        out.append({"dealer": g["dealer"], "vul": g.get("vul", "None"), "deal": deal,
                    "calls": g["calls"], "notes": notes})
    return out


def sources():
    srcs = {}
    srcs["brill"] = sorted(glob.glob(os.path.join(MAIN, ".rbb-cache", "brill", "2026-*", "*field.pbn")))
    srcs["bba"] = sorted(glob.glob(os.path.join(PBS, "bba", "*.pbn")))
    srcs["gib"] = sorted(p for p in glob.glob(os.path.join(PBS, "GIB", "*")) if os.path.isfile(p))
    return srcs


def gib_key(calls):
    t = {"Pass": "P", "X": "D", "XX": "R"}
    return "-".join(t.get(c, c.replace("NT", "N")) for c in calls) or "_"


def gib_tree(calls):
    p = os.path.join(GIBXML, gib_key(calls) + ".xml")
    if not os.path.exists(p):
        return None
    import xml.etree.ElementTree as ET
    try:
        root = ET.parse(p).getroot()
    except ET.ParseError:
        return None
    return {r.get("b").upper(): (r.get("m") or "") for r in root.iter("r")}


def our_from_gib(g):
    return {"P": "Pass", "D": "X", "R": "XX"}.get(g, g.replace("N", "NT"))


def seat_of(dealer, i):
    return SEATS[(SEATS.index(dealer) + i) % 4]


# -------------------------------------------------------------- collect


def collect(args):
    os.makedirs(OUT, exist_ok=True)
    recs = []
    for src, files in sources().items():
        n = 0
        for f in files:
            for g in parse_pbn(f):
                g["src"] = src
                g["file"] = os.path.relpath(f, os.path.dirname(MAIN))
                recs.append(g)
                n += 1
        print(f"{src}: {len(files)} files, {n} auctions", file=sys.stderr)
    with open(os.path.join(OUT, "auctions.jsonl"), "w") as fo:
        for r in recs:
            fo.write(json.dumps(r) + "\n")


def load_auctions():
    with open(os.path.join(OUT, "auctions.jsonl")) as f:
        return [json.loads(l) for l in f]


def cut(calls):
    """The calls up to and including the MAXN-th non-pass call."""
    n = 0
    for i, c in enumerate(calls):
        if c != "Pass":
            n += 1
            if n == MAXN:
                return calls[: i + 1]
    return calls


DUMMY = ["AKQJ.T98.765.432", "T98.AKQJ.432.765", "765.432.AKQJ.T98", "432.765.T98.AKQJ"]


def queries(auctions):
    """Distinct (dealer, vul, auction) to ask: each real auction's cut, each
    South position in the Brill field (with the hand), and each GIB-defined
    call at a real position with fewer than MAXN non-pass calls."""
    q = {}
    positions = collections.Counter()
    for a in auctions:
        cc = cut(a["calls"])
        while cc and cc[-1] == "Pass":
            cc = cc[:-1]
        nxt = len(cc)
        hand = a["deal"][seat_of(a["dealer"], nxt)] if nxt < 4 * 40 else DUMMY[0]
        k = (a["dealer"], a["vul"], " ".join(cc))
        q.setdefault(k, {"hand": hand, "auction": " ".join(cc), "dealer": a["dealer"], "vul": a["vul"]})
        n = 0
        for i, c in enumerate(a["calls"]):
            if n >= MAXN:
                break
            positions[(a["dealer"], a["vul"], tuple(a["calls"][:i]))] += 1
            if a["src"] == "brill" and seat_of(a["dealer"], i) == "S":
                kk = (a["dealer"], a["vul"], " ".join(a["calls"][:i]), a["deal"]["S"])
                q.setdefault(kk, {"hand": a["deal"]["S"], "auction": " ".join(a["calls"][:i]),
                                  "dealer": a["dealer"], "vul": a["vul"]})
            if c != "Pass":
                n += 1
    # GIB-defined calls at real positions
    seen = set()
    for (d, v, pre) in positions:
        npass = sum(1 for c in pre if c != "Pass")
        if npass >= MAXN:
            continue
        key = gib_key(list(pre))
        if key in seen:
            continue
        seen.add(key)
        t = gib_tree(list(pre))
        if not t:
            continue
        for gc, m in t.items():
            c = our_from_gib(gc)
            if c == "Pass" or not m:
                continue
            calls = list(pre) + [c]
            # GIB's tree is dealer-relative: ask with dealer N, vul None
            kk = ("N", "None", " ".join(calls))
            q.setdefault(kk, {"hand": DUMMY[len(calls) % 4], "auction": " ".join(calls), "dealer": "N", "vul": "None"})
    return q


def ask(args):
    auctions = load_auctions()
    q = queries(auctions)
    items = list(q.items())
    print(f"{len(items)} queries, card {args.card}", file=sys.stderr)
    out_path = os.path.join(OUT, f"answers-{os.path.basename(args.card).split('.')[0]}.jsonl")
    done = set()
    if os.path.exists(out_path) and not args.fresh:
        with open(out_path) as f:
            for l in f:
                try:
                    done.add(json.loads(l)["key"])
                except (ValueError, KeyError):
                    pass
    todo = [(k, v) for k, v in items if json.dumps(k) not in done]
    print(f"{len(todo)} to ask", file=sys.stderr)
    rbb = os.path.join(MAIN, "target", "release", "rbb")
    lock = threading.Lock()
    fo = open(out_path, "a")

    def worker(chunk):
        cmd = [rbb, "call", "--batch", "-c", args.card, "--rules", args.rules, "--rules-at", args.rules_at]
        p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)

        def feed():
            for i, (k, v) in enumerate(chunk):
                p.stdin.write(json.dumps({**v, "id": i}) + "\n")
            p.stdin.close()
        th = threading.Thread(target=feed)
        th.start()
        for line in p.stdout:
            d = json.loads(line)
            k, v = chunk[d["id"]]
            if "error" in d:
                rec = {"key": json.dumps(k), "error": d["error"][:200]}
            else:
                pos = d["auction"]["position"]
                steps = d["auction"]["steps"]
                rec = {"key": json.dumps(k), "unread": pos["unread"],
                       "art": pos["artificial"],
                       "expl": [s.get("explanation") for s in steps],
                       "rule": [(s["rule"] or {}).get("file", "") + (":%d" % s["rule"]["line"] if s.get("rule") else "") for s in steps],
                       "decision": {"call": d["call"] if isinstance(d["call"], str) else
                                    f"{d['call']['Bid']['level']}{ {'Clubs':'C','Diamonds':'D','Hearts':'H','Spades':'S','NoTrump':'NT'}[d['call']['Bid']['strain']] }",
                                    "explanation": d["explanation"], "rule": bool(d["rule"])}}
            with lock:
                fo.write(json.dumps(rec) + "\n")
        p.wait()
        th.join()

    jobs = args.jobs
    chunks = [todo[i::jobs] for i in range(jobs)]
    ts = [threading.Thread(target=worker, args=(c,)) for c in chunks if c]
    for t in ts:
        t.start()
    for t in ts:
        t.join()
    fo.close()
    print(f"-> {out_path}", file=sys.stderr)


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("collect")
    a = sub.add_parser("ask")
    a.add_argument("--card", default=os.path.join(MAIN, "cards", "json", "Brill.json"))
    a.add_argument("--rules", default=PACK)
    a.add_argument("--rules-at", default="origin/bottom-review")
    a.add_argument("--jobs", type=int, default=8)
    a.add_argument("--fresh", action="store_true")
    r = sub.add_parser("report")
    r.add_argument("--out", default=os.path.join(ROOT, "docs", "reviews", "2026-11-27-undefined-calls.md"))
    r.add_argument("--card", default="Brill")
    args = ap.parse_args()
    if args.cmd == "collect":
        collect(args)
    elif args.cmd == "ask":
        ask(args)
    else:
        from undefined_calls_report import report  # noqa: E402
        report(args)
