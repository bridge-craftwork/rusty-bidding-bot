#!/usr/bin/env python3
"""Our ranges against GIB's bidding tree, call by call.

    probes/tools/gib_tree.py show P-P-1S-P-2C-P [--card 21GF-GIB]
    probes/tools/gib_tree.py prefixes --compare ALL.json [--top 2000] [--tag NAME]
    probes/tools/gib_tree.py fetch    [--tag NAME] [--max N]
    probes/tools/gib_tree.py report   [--tag NAME] [--card 21GF-GIB] [--rules DIR]
    probes/tools/gib_tree.py run      --compare ALL.json [--top 2000]   # the last three

BBO's GIB robot publishes what each call means in its system: for an
auction so far, every call the next player can make with its meaning
("Drury -- 3+ !S; 11- HCP; 10-12 total points"). The service behind
https://netbridge.dk/gib.html is
`https://gibrest.bridgebase.com/u_bm/u_bm.php?s=P-P-1S-P-*&t=g` (XML, one
`<r b="2c" m="...">` per call). GIB's meanings are cumulative: a call's
line is all the caller has shown so far. They are a check on our own
readings (Rick, 2026-11-25), above all on tops: a game bid we read as
"15+" where GIB's tree says 14-18 because stronger hands have other calls.
See docs/BIDDING-MODEL.md, "Checking ranges against GIB's bidding tree".

**show**: one position: GIB's meaning of every call there beside ours
(`rbb explain-auction` of the auction plus that call), fetched if it is
not archived yet. Auctions as `P-P-1S-P-2C-P` or `"P P 1S P 2C P"`.

**prefixes**: from a compare JSON (`ours`), every position our engine bid
from (an auction so far, with the calls made there), ranked by boards;
the top N go to `prefixes-<tag>.json`.

**fetch**: GIB's tree at each listed position. A guest on BBO's server:
sequential, one request a second at most, a named User-Agent, a stop
after 5 failures in a row. **The archive**: every answer is kept in
`.rbb-cache/gib-tree/xml/<auction>.xml` in the main checkout (shared by
worktrees), with a line in `index.jsonl` (when, the tree version `gv`),
and never fetched again, so a rerun resumes and no position is asked twice.

**report**: for each listed position and each call our engine made there,
the caller's knowledge after it, GIB's against ours:
- HCP: either end more than `--tol` (default 2) apart, or disjoint;
- total points: GIB's against our support points in the agreed suit
  when our side has one (`tp`), else our declarer points (`dp`);
- an open top: GIB states a top and ours is 25+ or more than 3 above it;
- suit length: minimums differ, or maximums where either is 5 or less.
A difference already present at the caller's previous call is not counted
again. Weighted by the boards whose auctions pass through the position,
grouped by our rule. Writes `report-<tag>.md` and `.json`.
GIB plays its own 2/1 card: read the differences, do not copy them; the
default card is our GIB approximation, 21GF-GIB.
"""

import argparse
import ast
import json
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

URL = "https://gibrest.bridgebase.com/u_bm/u_bm.php?s={}&t=g&v3b=web&v3v=6.19.3&v3u=BBO1"
UA = "rusty-bidding-bot gib_tree (personal research; one request a second, archived)"
ROOT = Path(__file__).resolve().parents[2]
SEATS = "NESW"
SUITS = "CDHS"  # our `len` and `tp` order
SYM = {"C": "♣", "D": "♦", "H": "♥", "S": "♠"}
STRAIN = {"Clubs": "C", "Diamonds": "D", "Hearts": "H", "Spades": "S", "NoTrump": "N"}
OPEN = 99  # an open top


def main_checkout():
    try:
        g = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--path-format=absolute",
                            "--git-common-dir"], capture_output=True, text=True, check=True).stdout.strip()
        return Path(g).parent
    except (subprocess.CalledProcessError, OSError):
        return ROOT


CACHE = main_checkout() / ".rbb-cache" / "gib-tree"


# ------------------------------------------------------------- auctions

def gib_call(c):
    """Our call (compare JSON or text) as GIB writes it: P D R 1S 3N."""
    if isinstance(c, dict):
        b = c["Bid"]
        return f"{b['level']}{STRAIN[b['strain']]}"
    t = {"Pass": "P", "Double": "D", "Redouble": "R", "X": "D", "XX": "R"}.get(c, c)
    return t.replace("NT", "N").upper()


def our_call(g):
    """GIB's call as `rbb -a` reads it."""
    return {"P": "P", "D": "X", "R": "XX"}.get(g, g.replace("N", "NT"))


def parse_auction(s):
    s = s.strip().strip("-*").strip()
    if not s:
        return []
    return [gib_call(t) for t in re.split(r"[-\s]+", s) if t and t != "*"]


def key(prefix):
    return "-".join(prefix) or "_"


# ---------------------------------------------------------------- fetch

def xml_path(prefix):
    return CACHE / "xml" / f"{key(prefix)}.xml"


def fetch_one(prefix):
    seq = "-".join(prefix + ["*"])
    req = urllib.request.Request(URL.format(seq), headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=60) as r:
        raw = r.read()
    root = ET.fromstring(raw)
    if root.tag != "sc_bm" or root.get("err") not in (None, "0"):
        raise ValueError(f"unexpected answer: {raw[:200]!r}")
    return raw, root.get("gv")


def archive(prefix, raw, gv):
    p = xml_path(prefix)
    p.parent.mkdir(parents=True, exist_ok=True)
    tmp = p.with_suffix(".tmp")
    tmp.write_bytes(raw)
    tmp.rename(p)
    with open(CACHE / "index.jsonl", "a") as f:
        f.write(json.dumps({"auction": key(prefix), "gv": gv,
                            "fetched": time.strftime("%Y-%m-%dT%H:%M:%S")}) + "\n")


class Fetcher:
    def __init__(self, interval=1.0):
        self.interval = max(1.0, interval)
        self.last = 0.0
        self.fails = 0

    def get(self, prefix):
        """The archived tree at prefix, fetched once if missing; None on failure."""
        p = xml_path(prefix)
        if p.exists():
            return tree(p.read_bytes())
        delay = 2.0
        while True:
            wait = self.last + self.interval - time.time()
            if wait > 0:
                time.sleep(wait)
            self.last = time.time()
            try:
                raw, gv = fetch_one(prefix)
                archive(prefix, raw, gv)
                self.fails = 0
                return tree(raw)
            except (urllib.error.URLError, ValueError, ET.ParseError, TimeoutError, OSError) as e:
                self.fails += 1
                code = getattr(e, "code", None)
                CACHE.mkdir(parents=True, exist_ok=True)
                with open(CACHE / "failures.jsonl", "a") as f:
                    f.write(json.dumps({"auction": key(prefix), "error": str(e)[:300], "code": code,
                                        "time": time.strftime("%Y-%m-%dT%H:%M:%S")}) + "\n")
                print(f"error on {key(prefix)}: {str(e)[:120]}", file=sys.stderr, flush=True)
                if self.fails >= 5:
                    sys.exit("stopping: 5 failures in a row; rerun to resume")
                if code and 400 <= code < 500 and code != 429:
                    return None
                time.sleep(delay)
                delay = min(delay * 4, 300)


def tree(raw):
    """{call: meaning} at a position."""
    root = ET.fromstring(raw)
    return {r.get("b").upper(): r.get("m") or "" for r in root.iter("r")}


# ---------------------------------------------------------------- parse

RNG = r"(\d+)(?:-(\d+)|([+-]))?"


def rng(m):
    lo, hi, sign = m.group(1), m.group(2), m.group(3)
    lo = int(lo)
    if hi:
        return (lo, int(hi))
    if sign == "+":
        return (lo, None)
    if sign == "-":
        return (None, lo)
    return (lo, lo)


def parse_meaning(m):
    """'Drury -- 3+ !S; 11- HCP; 10-12 total points' ->
    {name, hcp, total, len {s: (lo, hi)}, other [...]}."""
    name, _, body = m.rpartition(" -- ") if " -- " in m else ("", "", m)
    out = {"name": name, "hcp": None, "total": None, "len": {}, "other": []}
    for part in [p.strip() for p in body.split(";") if p.strip()]:
        x = re.fullmatch(RNG + r" !([CDHS])", part)
        if x:
            out["len"][x.group(4)] = rng(x)
            continue
        x = re.fullmatch(RNG + r" HCP", part)
        if x:
            out["hcp"] = rng(x)
            continue
        x = re.fullmatch(RNG + r" total points", part)
        if x:
            out["total"] = rng(x)
            continue
        out["other"].append(part.replace("!", ""))
    return out


# ------------------------------------------------------------ our side

def resolve_card(card):
    if "/" not in card and not card.endswith((".json", ".bbsa")):
        for c in (ROOT / "cards/json" / f"{card}.json", ROOT / "cards/bbsa" / f"{card}.bbsa"):
            if c.exists():
                return str(c)
    return str(Path(card).resolve())


def explain(args, calls):
    """Our rows for an auction given in GIB calls (dealer N, none vul)."""
    a = " ".join(our_call(c) for c in calls)
    cmd = [args.rbb, "explain-auction", "-a", a, "-d", "N", "-c", resolve_card(args.card), "--json"]
    if args.rules:
        cmd += ["--rules", args.rules]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        return None
    return json.loads(p.stdout)


def r2(r):
    hi = r["hi"]
    return (r["lo"], OPEN if hi >= 25 else hi)


def our_state(row):
    """The caller's knowledge after his call: hcp, our points, lengths.
    The points are support points in the agreed suit for the hand with
    fewer trumps shown, else declarer points (HCP + length): the long-trump
    hand does not count shortness as dummy (Rick, 2026-11-25)."""
    st = row["step"]
    k = st.get("knowledge")
    if not k:
        return None
    seat = SEATS.index(st["caller"][0])
    side = row["sides"][seat % 2] if row.get("sides") else {}
    trump = side.get("trump")
    short = False
    if trump in STRAIN and STRAIN[trump] in SUITS and row.get("seats"):
        t = SUITS.index(STRAIN[trump])
        partner = row["seats"][(seat + 2) % 4]
        short = k["len"][t]["lo"] < partner["len"][t]["lo"]
    if short:
        pts, unit = r2(k["tp"][SUITS.index(STRAIN[trump])]), f"{SYM[STRAIN[trump]]} sup"
    else:
        pts, unit = r2(k["dp"]), "decl"
    return {"hcp": r2(k["hcp"]), "pts": pts, "unit": unit,
            "len": {s: (k["len"][i]["lo"], k["len"][i]["hi"]) for i, s in enumerate(SUITS)},
            "rule": st.get("rule"), "means": st.get("explanation") or ""}


# -------------------------------------------------------------- compare

def end_diff(ours, gib, tol):
    """Kinds of difference between our range and GIB's (either end open)."""
    glo, ghi = gib
    olo, ohi = ours
    out = []
    if ghi is not None and olo > ghi or glo is not None and ohi < glo:
        return ["conflict"]
    # "-wide": ours says less than GIB's (we know less); "-narrow": more
    if glo is not None and abs(olo - glo) > tol:
        out.append("floor-wide" if olo < glo else "floor-narrow")
    if ghi is not None:
        if ohi >= OPEN or ohi - ghi > 3:
            out.append("open-top")
        elif abs(ohi - ghi) > tol:
            out.append("top-wide" if ohi > ghi else "top-narrow")
    return out


def diffs(o, g, tol):
    out = []
    for dim, ours, gib in (("hcp", o["hcp"], g["hcp"]), ("pts", o["pts"], g["total"])):
        if gib:
            for k in end_diff(ours, gib, tol):
                out.append({"dim": dim, "kind": k, "ours": ours, "gib": gib})
    for s, (glo, ghi) in g["len"].items():
        olo, ohi = o["len"][s]
        glo = glo or 0
        ghi = 13 if ghi is None else ghi
        if olo > ghi or ohi < glo:
            out.append({"dim": s, "kind": "conflict", "ours": (olo, ohi), "gib": (glo, ghi)})
        elif olo != glo:
            out.append({"dim": s, "kind": "min-wide" if olo < glo else "min-narrow",
                        "ours": (olo, ohi), "gib": (glo, ghi)})
        elif min(ohi, ghi) <= 5 and ohi != ghi:
            out.append({"dim": s, "kind": "max-wide" if ohi > ghi else "max-narrow",
                        "ours": (olo, ohi), "gib": (glo, ghi)})
    return out


def fr(r, unit=""):
    lo, hi = r
    lo = lo or 0
    if hi is None or hi >= OPEN:
        return f"{lo}+{unit}"
    return f"{lo}{unit}" if lo == hi else f"{lo}-{hi}{unit}"


def describe(d):
    if d["dim"] in SUITS:
        return f"{SYM[d['dim']]} ours {fr(d['ours'])} / GIB {fr(d['gib'])}"
    return f"{d['dim']} {d['kind']}: ours {fr(d['ours'])} / GIB {fr(d['gib'])}"


def compare_call(args, prefix, call, gib_here, gib_prev, rows):
    """One call: GIB's meaning against ours; None when either is missing."""
    i = len(prefix)
    if rows is None or len(rows) <= i or call not in gib_here:
        return None
    o = our_state(rows[i])
    if o is None:
        return None
    g = parse_meaning(gib_here[call])
    if o["rule"] is None:  # no rule of ours makes this call: nothing to compare
        return {"auction": key(prefix), "call": call, "seat": SEATS[i % 4], "our_rule": "(no rule)",
                "our_means": "", "ours": None, "gib": gib_here[call], "gib_parsed": g, "diffs": [],
                "no_rule": True}
    ds = diffs(o, g, args.tol)
    # not again at the caller's previous call
    if i >= 4 and gib_prev and prefix[i - 4] in gib_prev:
        po = our_state(rows[i - 4])
        if po:
            old = {(d["dim"], d["kind"], tuple(d["ours"]), tuple(x if x is not None else -1 for x in d["gib"]))
                   for d in diffs(po, parse_meaning(gib_prev[prefix[i - 4]]), args.tol)}
            ds = [d for d in ds if (d["dim"], d["kind"], tuple(d["ours"]),
                                    tuple(x if x is not None else -1 for x in d["gib"])) not in old]
    rule = o["rule"]
    return {"auction": key(prefix), "call": call, "seat": SEATS[i % 4],
            "our_rule": f"{rule['file'].split('rules/')[-1]}:{rule['line']}" if rule else "(no rule)",
            "our_means": o["means"], "ours": {"hcp": o["hcp"], "pts": o["pts"], "unit": o["unit"],
                                              "len": o["len"]},
            "gib": gib_here[call], "gib_parsed": g, "diffs": ds}


# ------------------------------------------------------------- commands

def cmd_show(args):
    prefix = parse_auction(args.auction)
    f = Fetcher(args.interval)
    here = f.get(prefix)
    if here is None:
        sys.exit("no answer")
    prev = f.get(prefix[:-4]) if len(prefix) >= 4 else None
    seat = SEATS[len(prefix) % 4]
    print(f"{key(prefix)}-*  ({seat} to call, dealer N; GIB tree, ours on {args.card})\n")
    missing = []
    for call, m in here.items():
        rows = explain(args, prefix + [call])
        c = compare_call(args, prefix, call, here, prev, rows)
        if c is None or c.get("no_rule"):
            missing.append(call)
            continue
        print(f"{call:>3}  GIB  {m}")
        o = c["ours"]
        lens = " ".join(f"{SYM[s]}{fr(o['len'][s])}" for s in "SHDC" if o["len"][s] != (0, 13))
        print(f"     ours {c['our_means']} [{c['our_rule']}]: {fr(o['hcp'])} HCP, "
              f"{fr(o['pts'])} {o['unit']}  {lens}")
        for d in c["diffs"]:
            print(f"       ≠ {describe(d)}")
        print()
    if missing:
        print(f"GIB has, we have no rule for: {' '.join(missing)}")


def cmd_prefixes(args):
    print(f"reading {args.compare} ...", file=sys.stderr)
    data = json.load(open(args.compare))
    pos = defaultdict(Counter)
    total = 0
    for b in data["boards"]:
        calls = b.get("ours")
        if isinstance(calls, str):
            calls = ast.literal_eval(calls)
        if not calls:
            continue
        total += 1
        # GIB's tree starts at the dealer; we read every auction as dealt by N
        calls = [gib_call(c) for c in calls]
        for i, c in enumerate(calls):
            pos[key(calls[:i])][c] += 1
    ranked = sorted(pos.items(), key=lambda kv: -sum(kv[1].values()))[: args.top]
    rows = [{"auction": k, "boards": sum(c.values()), "calls": dict(c.most_common())} for k, c in ranked]
    out = CACHE / f"prefixes-{args.tag}.json"
    CACHE.mkdir(parents=True, exist_ok=True)
    json.dump({"source": str(args.compare), "boards": total, "positions": len(pos), "top": args.top,
               "rows": rows}, open(out, "w"), indent=1)
    print(f"{len(pos)} positions on {total} boards; top {len(rows)} -> {out}")


def load_prefixes(args):
    p = CACHE / f"prefixes-{args.tag}.json"
    if not p.exists():
        sys.exit(f"{p} missing: run `prefixes` first")
    return json.load(open(p))["rows"]


def split(k):
    return [] if k == "_" else k.split("-")


def cmd_fetch(args):
    rows = load_prefixes(args)
    todo = [r for r in rows if not xml_path(split(r["auction"])).exists()]
    if args.max:
        todo = todo[: args.max]
    print(f"{len(rows)} positions, {len(rows) - len(todo)} archived, fetching {len(todo)}", flush=True)
    f = Fetcher(args.interval)
    t0 = time.time()
    for n, r in enumerate(todo, 1):
        f.get(split(r["auction"]))
        if n % 100 == 0:
            print(f"{n}/{len(todo)} in {(time.time() - t0) / 60:.1f} min", flush=True)
    print(f"done: {len(todo)} in {(time.time() - t0) / 60:.1f} min")


def cmd_report(args):
    rows = load_prefixes(args)
    jobs = []
    for r in rows:
        prefix = split(r["auction"])
        p = xml_path(prefix)
        if not p.exists():
            continue
        here = tree(p.read_bytes())
        pp = xml_path(prefix[:-4]) if len(prefix) >= 4 else None
        prev = tree(pp.read_bytes()) if pp and pp.exists() else None
        for call, n in r["calls"].items():
            jobs.append((prefix, call, n, here, prev))
    print(f"{len(jobs)} calls at {len(rows)} positions; reading ours ...", file=sys.stderr)

    def work(j):
        prefix, call, n, here, prev = j
        c = compare_call(args, prefix, call, here, prev, explain(args, prefix + [call]))
        if c:
            c["boards"] = n
        return c

    with ThreadPoolExecutor(args.jobs) as ex:
        res = [c for c in ex.map(work, jobs) if c]
    groups = defaultdict(lambda: {"boards": 0, "calls": 0, "kinds": Counter(), "examples": []})
    kinds = Counter()
    no_rule = Counter()
    for c in res:
        if c.get("no_rule"):
            no_rule[f"{c['auction']} {c['call']}"] += c["boards"]
            continue
        for d in c["diffs"]:
            kinds[f"{d['dim'] if d['dim'] in ('hcp', 'pts') else 'len'} {d['kind']}"] += c["boards"]
        if not c["diffs"]:
            continue
        g = groups[(c["our_rule"], c["our_means"])]
        g["boards"] += c["boards"]
        g["calls"] += 1
        for d in c["diffs"]:
            g["kinds"][f"{d['dim']} {d['kind']}"] += c["boards"]
        g["examples"].append(c)
    out = {"card": args.card, "tol": args.tol, "calls": len(res),
           "boards_with_diff": sum(c["boards"] for c in res if c["diffs"]),
           "boards": sum(c["boards"] for c in res), "kinds": dict(kinds.most_common()),
           "no_rule": dict(no_rule.most_common()),
           "groups": [{"rule": k[0], "means": k[1], **{x: v for x, v in g.items() if x != "kinds"},
                       "kinds": dict(g["kinds"].most_common()),
                       "examples": sorted(g["examples"], key=lambda c: -c["boards"])[:5]}
                      for k, g in sorted(groups.items(), key=lambda kv: -kv[1]["boards"])]}
    base = CACHE / f"report-{args.tag}"
    json.dump(out, open(f"{base}.json", "w"), indent=1, ensure_ascii=False)
    open(f"{base}.md", "w").write(markdown(out, args.groups))
    print(f"{len(res)} calls compared; {out['boards_with_diff']}/{out['boards']} call-boards differ "
          f"-> {base}.md")


def markdown(o, ngroups):
    L = [f"# Our ranges against GIB's bidding tree ({o['card']})", "",
         f"{o['calls']} calls compared, weighted by boards: {o['boards_with_diff']} of {o['boards']} "
         f"call-boards differ (tolerance {o['tol']}).", "", "| difference | call-boards |", "|---|---|"]
    L += [f"| {k} | {v} |" for k, v in o["kinds"].items()]
    if o["no_rule"]:
        L += ["", "Calls our engine made with no rule (GIB's tree has them): "
              + ", ".join(f"`{k}` {v}" for k, v in list(o["no_rule"].items())[:20])]
    L += ["", f"## The top {ngroups} rules by boards", ""]
    for g in o["groups"][:ngroups]:
        L.append(f"### {g['rule']}: {g['means']}  ({g['boards']} boards, {g['calls']} positions)")
        L.append("")
        L.append(", ".join(f"{k} {v}" for k, v in g["kinds"].items()))
        L.append("")
        for c in g["examples"][:3]:
            L.append(f"- `{c['auction']}` **{c['call']}** ({c['boards']} boards). GIB: {c['gib']}")
            L.append(f"  - ours: {fr(c['ours']['hcp'])} HCP, {fr(c['ours']['pts'])} {c['ours']['unit']}; "
                     + "; ".join(describe(d) for d in c["diffs"]))
        L.append("")
    return "\n".join(L)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cmd", choices=["show", "prefixes", "fetch", "report", "run"])
    ap.add_argument("auction", nargs="?", help="show: the auction so far (P-P-1S-P)")
    ap.add_argument("--compare", help="a compare JSON (prefixes, run)")
    ap.add_argument("--top", type=int, default=2000, help="positions to keep (prefixes)")
    ap.add_argument("--tag", default="main", help="names the position list and report")
    ap.add_argument("--max", type=int, default=0, help="fetch at most N this run")
    ap.add_argument("--interval", type=float, default=1.0, help="seconds between requests (>= 1)")
    ap.add_argument("--card", default="21GF-GIB", help="a stock card's name or a path")
    ap.add_argument("--rules", help="rules for our readings (default: compiled in)")
    ap.add_argument("--rbb", default=str(ROOT / "target/release/rbb"))
    ap.add_argument("--tol", type=int, default=2)
    ap.add_argument("--jobs", type=int, default=6)
    ap.add_argument("--groups", type=int, default=40, help="rules listed in the markdown")
    args = ap.parse_args()
    if args.cmd == "show":
        if args.auction is None:
            ap.error("show needs an auction")
        cmd_show(args)
    elif args.cmd == "prefixes":
        if not args.compare:
            ap.error("prefixes needs --compare")
        cmd_prefixes(args)
    elif args.cmd == "fetch":
        cmd_fetch(args)
    elif args.cmd == "report":
        cmd_report(args)
    else:
        if not args.compare:
            ap.error("run needs --compare")
        cmd_prefixes(args)
        cmd_fetch(args)
        cmd_report(args)


if __name__ == "__main__":
    main()
