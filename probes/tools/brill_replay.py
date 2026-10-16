#!/usr/bin/env python3
"""Replay our Brill daily tables with today's engine: how would we have
scored, bidding only, with the rules as they are now?

    probes/tools/brill_replay.py [FIELD.pbn ...] [--card 21GF-GIB] [--ew-card Brill]
        [--check 10] [--offline] [--out-dir .rbb-cache/brill]

With no files, every `.rbb-cache/brill/<date>/daily-*-field.pbn` of the
main checkout. For each of our tables (South = `--us`, Rusty):

1. Walk the actual auction; at each South turn ask today's engine (`rbb
   call`, the card, the event's scoring). Up to the first South call that
   differs, the actual auction stands.
2. From there the auction goes on: South from our engine; North, East and
   West from Thorvald Aagaard's public Brill service (`GET /bid`, the seat
   API's encodings: rusty-seat-service docs/seat-api.txt §3), until it ends.
   No `meanings` are sent (Brill reads our calls by its own system).
3. Determinism check: on `--check` boards (default 10) whose auction does
   not change at South, every Brill call of the actual auction is asked of
   the service again and compared with the PBN. A low match rate means the
   replay also measures today's Brill, not only today's Rusty.
4. Score: the replayed and the actual contract each at double dummy (`rbb
   dd`), against every other table's contract at double dummy on the same
   board (brill_report's "bidding" column): MP% in MP events, cross-IMPs
   in IMP events.
5. Report: `<out-dir>/replay-<card>.md` and `.json`.

Courtesy to the service (one person runs it): sequential, at most one
request a second, a descriptive User-Agent, every answer cached by (hand,
ctx, dealer, seat, vul, matchtype) under the main checkout's
`.rbb-cache/brill-service/bid/` and never fetched again, so a rerun
resumes; a stop after 5 consecutive failures (rerun to resume).
`--offline` uses the cache only (a board that needs a fetch is skipped).

`--card` is a stock name (`cards/json/<name>.json` where there is one, as
`rbb` plays it, else `cards/bbsa/<name>.bbsa`) or a file; a card that does
not exist is reported and the run skipped. It is North-South's (our engine
gives a side one card, so South and partner share it); `--ew-card` reads
the opponents with another (default: the same). The plan for the dailies is
`--card Brill`: Rusty plays Brill's card.
Build first: `./dev-build.sh --ci build --release -p rbb-cli`.
"""

import argparse
import glob
import hashlib
import json
import math
import os
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brill_report as br  # noqa: E402

ROOT = br.ROOT
BASE = "https://brillservice.aalborgdata.dk"
UA = "rusty-bidding-bot brill_replay (rick.wilson.ca@gmail.com)"
SEATS = "NESW"


def main_checkout():
    """The main checkout: a worktree shares its caches, so they outlive it."""
    try:
        g = subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"],
                           capture_output=True, text=True, check=True, cwd=ROOT).stdout.strip()
        return os.path.dirname(g)
    except (subprocess.CalledProcessError, OSError):
        return ROOT


MAIN = main_checkout()
CACHE = os.path.join(MAIN, ".rbb-cache")


# ------------------------------------------------------------------ Brill

TOKEN = {"Pass": "--", "X": "Db", "XX": "Rd"}


def ctx_of(calls):
    return "".join(TOKEN.get(c) or (c[0] + ("N" if c.endswith("NT") else c[1])) for c in calls)


def from_brill(bid):
    c = br.norm_call(bid.replace("--", "P").replace("Db", "X").replace("Rd", "XX"))
    if c is None:
        raise ValueError(f"unreadable call {bid!r}")
    return c


def vul_brill(v):
    return br.vul_arg(v)  # None, NS, EW, All: the seat API's spelling


class Brill:
    def __init__(self, args):
        self.dir = os.path.join(CACHE, "brill-service", "bid")
        os.makedirs(self.dir, exist_ok=True)
        self.offline = args.offline
        self.interval = max(1.0, args.interval)
        self.last = 0.0
        self.fails = 0
        self.fetched = 0
        self.hits = 0
        self.errors = 0
        self.log = os.path.join(CACHE, "brill-service", "replay-failures.jsonl")

    def key(self, q):
        return hashlib.sha1(json.dumps(q, sort_keys=True).encode()).hexdigest()

    def bid(self, hand, calls, dealer, seat, vul, scoring):
        q = {"hand": br.sorted_hand(hand), "ctx": ctx_of(calls), "dealer": dealer, "seat": seat,
             "vul": vul_brill(vul), "matchtype": scoring}
        path = os.path.join(self.dir, self.key(q) + ".json")
        if os.path.exists(path):
            self.hits += 1
            with open(path) as f:
                return from_brill(json.load(f)["answer"]["bid"])
        if self.offline:
            raise LookupError("not cached (offline)")
        url = f"{BASE}/bid?" + urllib.parse.urlencode({**q, "details": "false"})
        delay = 2.0
        while True:
            wait = self.last + self.interval - time.time()
            if wait > 0:
                time.sleep(wait)
            self.last = time.time()
            try:
                req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
                with urllib.request.urlopen(req, timeout=60) as r:
                    raw = r.read()
                ans = json.loads(raw)
                call = from_brill(ans["bid"])
                tmp = path + ".tmp"
                with open(tmp, "w") as f:
                    json.dump({"query": q, "answer": ans,
                               "time": time.strftime("%Y-%m-%dT%H:%M:%S")}, f)
                os.replace(tmp, path)
                self.fails = 0
                self.fetched += 1
                return call
            except (urllib.error.URLError, ValueError, KeyError, TimeoutError, OSError) as e:
                self.errors += 1
                self.fails += 1
                code = getattr(e, "code", None)
                with open(self.log, "a") as f:
                    f.write(json.dumps({"query": q, "error": str(e)[:300], "code": code,
                                        "time": time.strftime("%Y-%m-%dT%H:%M:%S")}) + "\n")
                print(f"  Brill error {q['seat']} ctx={q['ctx']!r}: {str(e)[:120]}", flush=True)
                if self.fails >= 5:
                    raise SystemExit("stopping: 5 consecutive Brill failures; rerun to resume")
                if code and 400 <= code < 500 and code != 429:
                    raise LookupError(f"refused ({code})")
                time.sleep(delay)
                delay = min(delay * 4, 300)


# ----------------------------------------------------------------- engine

def find_card(card):
    for p in (os.path.join(ROOT, "cards", "json", card + ".json"),
              os.path.join(ROOT, "cards", "bbsa", card + ".bbsa"),
              os.path.join(ROOT, "cards", card + ".json"), card):
        if os.path.isfile(p):
            return os.path.abspath(p)
    return None


def rule_id(rule):
    if not rule:
        return "(no rule)"
    return f"{rule['file'].split('conventions/')[-1]}:{rule['line']}"


class Engine:
    def __init__(self, args, card, ew_card=None):
        self.rbb, self.card = args.rbb, card
        self.ew = ["--ew-card", ew_card] if ew_card else []
        self.rules = os.path.join(ROOT, "conventions")
        self.memo = {}

    def call(self, hand, calls, dealer, vul, scoring):
        k = (hand, tuple(calls), dealer, vul, scoring)
        if k not in self.memo:
            d = br.run_json([self.rbb, "call", br.sorted_hand(hand), "-a", " ".join(calls),
                             "-d", dealer, "-v", br.vul_arg(vul), "-s", scoring, "-c", self.card,
                             *self.ew, "--rules", self.rules, "--json"])
            self.memo[k] = {"call": br.call_name(d["call"]), "rule": rule_id(d.get("rule")),
                            "module": (d.get("rule") or {}).get("module") or "(no rule)",
                            "explanation": d.get("explanation") or ""}
        return self.memo[k]


# ----------------------------------------------------------------- replay

def seat_at(first, i):
    return SEATS[(SEATS.index(first) + i) % 4]


def replay(t, scoring, eng, brill):
    """The board replayed: (calls, divergence index or None, first changed
    call info). Raises LookupError when a needed Brill answer is missing."""
    actual = [c for c, _ in t["calls"]]
    first = t["first"]
    calls, div, change = [], None, None
    for i, c in enumerate(actual):
        if seat_at(first, i) == "S":
            e = eng.call(t["hands"]["S"], calls, first, t["vul"], scoring)
            if e["call"] != c:
                div, change = i, {**e, "was": c, "index": i, "prefix": " ".join(calls)}
                calls.append(e["call"])
                break
        calls.append(c)
    if div is None:
        return calls, None, None
    while not br.auction_complete(calls):
        if len(calls) > 80:
            raise RuntimeError("auction did not end")
        s = seat_at(first, len(calls))
        if s == "S":
            calls.append(eng.call(t["hands"]["S"], calls, first, t["vul"], scoring)["call"])
        else:
            calls.append(brill.bid(t["hands"][s], calls, first, s, t["vul"], scoring))
    return calls, div, change


def determinism(t, scoring, brill):
    """Brill's calls along the actual auction against the PBN's."""
    actual = [c for c, _ in t["calls"]]
    out = []
    for i, c in enumerate(actual):
        s = seat_at(t["first"], i)
        if s == "S":
            continue
        got = brill.bid(t["hands"][s], actual[:i], t["first"], s, t["vul"], scoring)
        out.append({"i": i, "seat": s, "pbn": c, "now": got, "prefix": " ".join(actual[:i])})
    return out


# ---------------------------------------------------------------- scoring

def contract_of(calls, first):
    return br.final_contract([(c, None) for c in calls], first)


def score_dd(contract, row, vul):
    if contract is None:
        return 0, None
    tr = br.dd_of(row, contract[3], contract[1])
    return br.ns_score(contract, tr, vul), tr


def mean_se(xs):
    if not xs:
        return None, None
    m = sum(xs) / len(xs)
    if len(xs) < 2:
        return m, None
    sd = math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))
    return m, sd / math.sqrt(len(xs))


def load_tables(paths, us):
    games = []
    for p in paths:
        with open(p, encoding="utf-8", errors="replace") as f:
            games.extend(br.parse_pbn(f.read()))
    tables, bad = br.tables_of(games, us)
    return tables, bad


def run(args):
    card = find_card(args.card)
    if card is None:
        print(f"card {args.card!r} not found (cards/bbsa, cards/json, or a path): skipped",
              file=sys.stderr)
        return 2
    paths = args.pbn or sorted(glob.glob(os.path.join(CACHE, "brill", "*", "daily-*-field.pbn")))
    if not paths:
        raise SystemExit("no field PBN files")
    tables, warnings = load_tables(paths, args.us)
    dd = br.dd_tables(tables, args.rbb, args.dd_cache)
    ew_card = None
    if args.ew_card:
        ew_card = find_card(args.ew_card)
        if ew_card is None:
            print(f"card {args.ew_card!r} not found (cards/json, cards/bbsa, or a path): skipped",
                  file=sys.stderr)
            return 2
    eng = Engine(args, card, ew_card)
    brill = Brill(args)

    # boards: (event, date, board) -> tables with the deal of ours
    boards = defaultdict(list)
    for t in tables:
        boards[(t["event"], t["date"], t["board"])].append(t)
    ev_scoring = {}
    for (ev, date, _), ts in boards.items():
        ev_scoring.setdefault((ev, date), [])
        ev_scoring[(ev, date)].extend(ts)
    ev_scoring = {k: br.event_scoring(v, "MP") for k, v in ev_scoring.items()}

    ours = []
    for key in sorted(boards, key=lambda k: (k[1], k[0], int(k[2]) if k[2].isdigit() else 0)):
        bt = boards[key]
        us = [t for t in bt if t["ours"]]
        if not us:
            continue
        t = us[0]
        field = [o for o in bt if o is not t and o["deal"] == t["deal"]]
        ours.append((key, t, field))
    print(f"{len(paths)} files, {len(ours)} of our boards, card {os.path.relpath(card, ROOT)}",
          flush=True)

    records, missing = [], []
    for n, ((ev, date, bno), t, field) in enumerate(ours, 1):
        scoring = ev_scoring[(ev, date)]
        try:
            calls, div, change = replay(t, scoring, eng, brill)
        except LookupError as e:
            missing.append(f"{ev} board {bno}: {e}")
            continue
        except RuntimeError as e:
            missing.append(f"{ev} board {bno}: engine: {e}")
            continue
        row = dd[t["deal"]]
        actual = [c for c, _ in t["calls"]]
        ca = t["contract"]
        cr = contract_of(calls, t["first"]) if div is not None else ca
        a_ns, a_tr = score_dd(ca, row, t["vul"])
        r_ns, r_tr = score_dd(cr, row, t["vul"])
        others = [score_dd(o["contract"], row, o["vul"])[0] for o in field]
        fn = br.mp_percent if scoring == "MP" else br.cross_imps
        rec = {
            "event": ev, "date": date, "board": bno, "scoring": scoring, "dealer": t["first"],
            "vul": t["vul"], "hands": t["hands"], "deal": t["deal"],
            "actual": actual, "replay": calls, "diverge": div, "change": change,
            "actual_contract": ca, "replay_contract": cr, "actual_dd_tricks": a_tr,
            "replay_dd_tricks": r_tr, "actual_ns_dd": a_ns, "replay_ns_dd": r_ns,
            "field_n": len(field), "field_contracts": [o["contract"] for o in field],
            "actual_score": fn(a_ns, others), "replay_score": fn(r_ns, others),
            "par_ns": row["par_ns"], "par": row["par"],
        }
        if rec["actual_score"] is not None:
            rec["delta"] = rec["replay_score"] - rec["actual_score"]
        records.append(rec)
        if div is not None:
            print(f"[{n}/{len(ours)}] {date} {ev} b{bno}: S {change['was']} -> {change['call']} "
                  f"({change['rule']}); {br.fmt_contract(ca)} -> {br.fmt_contract(cr)}; "
                  f"{scoring} {rec.get('delta', 0):+.1f}", flush=True)

    # determinism check on unchanged boards, spread over the week
    unchanged = {(r["event"], r["date"], r["board"]) for r in records if r["diverge"] is None}
    same = [(t, ev_scoring[k[:2]]) for (k, t, _) in ours if k in unchanged]
    step = max(1, len(same) // args.check) if args.check else 0
    picked = same[::step][:args.check] if step else []
    det = []
    for t, scoring in picked:
        try:
            det.append({"event": t["event"], "board": t["board"],
                        "calls": determinism(t, scoring, brill)})
        except LookupError as e:
            missing.append(f"determinism {t['event']} board {t['board']}: {e}")

    report = summarize(records, det, args, card, missing, warnings, brill)
    os.makedirs(args.out_dir, exist_ok=True)
    stem = os.path.join(args.out_dir, f"replay-{os.path.splitext(os.path.basename(card))[0]}")
    with open(stem + ".json", "w") as f:
        json.dump(report, f, ensure_ascii=False, indent=1, default=list)
    with open(stem + ".md", "w") as f:
        f.write(render(report))
    print(f"Brill: {brill.fetched} fetched, {brill.hits} cached, {brill.errors} errors\n"
          f"-> {stem}.md\n-> {stem}.json", flush=True)
    return 0


# ---------------------------------------------------------------- report

def agg(recs):
    out = {"boards": len(recs), "changed": sum(1 for r in recs if r["diverge"] is not None),
           "contract_changed": sum(1 for r in recs if r["replay_contract"] != r["actual_contract"])}
    sc = [r for r in recs if r.get("delta") is not None]
    if sc:
        out["scored"] = len(sc)
        out["actual"], out["actual_se"] = mean_se([r["actual_score"] for r in sc])
        out["replay"], out["replay_se"] = mean_se([r["replay_score"] for r in sc])
        out["delta"], out["delta_se"] = mean_se([r["delta"] for r in sc])
    return out


def summarize(records, det, args, card, missing, warnings, brill):
    by_event = defaultdict(list)
    by_day = defaultdict(lambda: defaultdict(list))
    by_type = defaultdict(list)
    for r in records:
        by_event[(r["date"], r["event"])].append(r)
        by_day[r["date"]][r["scoring"]].append(r)
        by_type[r["scoring"]].append(r)
    dcalls = [c for d in det for c in d["calls"]]
    origin = Counter(r["change"]["rule"] for r in records if r["change"])
    origin_mod = Counter(r["change"]["module"] for r in records if r["change"])
    return {
        "card": os.path.relpath(card, ROOT), "commit": git_head(), "generated": time.strftime("%Y-%m-%d %H:%M"),
        "events": [{"date": d, "event": e, "scoring": rs[0]["scoring"], **agg(rs)}
                   for (d, e), rs in sorted(by_event.items())],
        "days": [{"date": d, "scoring": s, **agg(rs)} for d in sorted(by_day)
                 for s, rs in sorted(by_day[d].items())],
        "totals": {s: agg(rs) for s, rs in sorted(by_type.items())},
        "all": agg(records),
        "origin_rule": origin.most_common(), "origin_module": origin_mod.most_common(),
        "determinism": {"boards": len(det), "calls": len(dcalls),
                        "match": sum(1 for c in dcalls if c["pbn"] == c["now"]),
                        "mismatches": [{"event": d["event"], "board": d["board"], **c}
                                       for d in det for c in d["calls"] if c["pbn"] != c["now"]]},
        "brill_requests": {"fetched": brill.fetched, "cached": brill.hits, "errors": brill.errors},
        "missing": missing, "warnings": warnings, "boards": records,
    }


def git_head():
    try:
        return subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True,
                              text=True, cwd=ROOT).stdout.strip()
    except OSError:
        return "?"


def fmt_ms(m, se, mp, delta=False):
    """MP% (or an MP% change) or IMPs a board, with its standard error."""
    if m is None:
        return "-"
    if mp:
        s = f"{m:+.1f}" if delta else f"{m:.1f}%"
        return s + (f" ± {se:.1f}" if se is not None else "")
    return f"{m:+.2f}" + (f" ± {se:.2f}" if se is not None else "")


def fmt_row(label, a, mp):
    if "scored" not in a:
        return f"| {label} | {a['boards']} | {a['changed']} | {a['contract_changed']} | - | - | - |"
    return (f"| {label} | {a['boards']} | {a['changed']} | {a['contract_changed']} | "
            f"{fmt_ms(a['actual'], a['actual_se'], mp)} | {fmt_ms(a['replay'], a['replay_se'], mp)} | "
            f"{fmt_ms(a['delta'], a['delta_se'], mp, delta=True)} |")


def auction_line(calls, first, div=None):
    out = []
    for i, c in enumerate(calls):
        s = seat_at(first, i)
        txt = f"{s}:{c}"
        if div is not None and i == div:
            txt = f"**{txt}**"
        out.append(txt)
    return " ".join(out)


def render(o):
    L = [f"# Brill dailies replayed with today's engine ({o['card']}, {o['commit']})\n",
         f"Generated {o['generated']}. Every one of our tables: the actual auction up to the first "
         "South call today's engine makes differently, then South from our engine and the other "
         "three seats from Brill's public `/bid` service, to the end. Both contracts scored at "
         "double dummy against every other table's contract at double dummy (bidding only, no "
         "play): MP% in MP events, cross-IMPs a board in IMP events. ± is one standard error "
         "over boards.\n"]
    d = o["determinism"]
    rate = d["match"] / d["calls"] if d["calls"] else None
    L.append("## Determinism check\n")
    if rate is None:
        L.append("Not run.\n")
    else:
        L.append(f"Brill's calls along {d['boards']} unchanged auctions, asked again today: "
                 f"**{d['match']}/{d['calls']} ({100 * rate:.0f}%)** match the PBN.")
        if rate < 0.9:
            L.append(f"\n**WARNING: Brill today bids differently from Brill on the day in "
                     f"{100 * (1 - rate):.0f}% of calls; the replay measures today's Brill as "
                     "well as today's Rusty.**")
        for m in d["mismatches"][:20]:
            L.append(f"- {m['event']} b{m['board']} `{m['prefix']}` {m['seat']}: PBN {m['pbn']}, now {m['now']}")
        L.append("")
    br_ = o["brill_requests"]
    L.append(f"Brill requests this run: {br_['fetched']} fetched, {br_['cached']} from cache, "
             f"{br_['errors']} errors.\n")
    L.append("## Week\n")
    L.append("| | boards | auction changed | contract changed | actual (DD) | replayed (DD) | change |")
    L.append("|---|---:|---:|---:|---:|---:|---:|")
    for s, a in o["totals"].items():
        L.append(fmt_row(f"**{s}**", a, s == "MP"))
    L.append(f"\nAll boards: {o['all']['boards']}, auctions changed {o['all']['changed']}, "
             f"contracts changed {o['all']['contract_changed']}.\n")
    L.append("## Per day\n")
    L.append("| day | boards | auction changed | contract changed | actual (DD) | replayed (DD) | change |")
    L.append("|---|---:|---:|---:|---:|---:|---:|")
    for r in o["days"]:
        L.append(fmt_row(f"{r['date']} {r['scoring']}", r, r["scoring"] == "MP"))
    L.append("\n## Per event\n")
    L.append("| event | boards | auction changed | contract changed | actual (DD) | replayed (DD) | change |")
    L.append("|---|---:|---:|---:|---:|---:|---:|")
    for r in o["events"]:
        L.append(fmt_row(f"{r['event']} ({r['scoring']})", r, r["scoring"] == "MP"))
    L.append("\n## Where the first changed call came from\n")
    L.append("| rule (today's) | boards |\n|---|---:|")
    for k, v in o["origin_rule"]:
        L.append(f"| `{k}` | {v} |")
    L.append("\n| module | boards |\n|---|---:|")
    for k, v in o["origin_module"]:
        L.append(f"| {k} | {v} |")

    def norm(r):  # comparable units: MP% points / 100 * 12 ~ IMPs, for ranking only
        return r["delta"] / 100 * 12 if r["scoring"] == "MP" else r["delta"]
    ch = [r for r in o["boards"] if r.get("delta") is not None and r["diverge"] is not None]
    gains = sorted((r for r in ch if norm(r) > 0), key=norm, reverse=True)
    losses = sorted((r for r in ch if norm(r) < 0), key=norm)
    L.append(f"\n{len(gains)} changed boards gained, {len(losses)} lost, "
             f"{len(ch) - len(gains) - len(losses)} scored the same.")
    for title, rs in (("gained most", gains[:10]), ("lost most", losses[:10])):
        L.append(f"\n## Boards that {title}\n")
        for r in rs:
            mp = r["scoring"] == "MP"
            u = (lambda v: f"{v:.0f}%") if mp else (lambda v: f"{v:+.1f}")
            c = r["change"]
            L.append(f"### {r['date']} {r['event']} board {r['board']}: {u(r['actual_score'])} -> "
                     f"{u(r['replay_score'])} ({r['delta']:+.1f})\n")
            L.append(f"Dealer {r['dealer']}, vul {r['vul']}. "
                     + " ".join(f"{s}: {br.sorted_hand(r['hands'][s])}" for s in SEATS)
                     + f". Par {r['par_ns']:+d} ({r['par'] or 'pass'}).\n")
            L.append(f"- actual: {auction_line(r['actual'], r['dealer'])} -> "
                     f"**{br.fmt_contract(r['actual_contract'])}** (DD NS {r['actual_ns_dd']:+d})")
            L.append(f"- replay: {auction_line(r['replay'], r['dealer'], r['diverge'])} -> "
                     f"**{br.fmt_contract(r['replay_contract'])}** (DD NS {r['replay_ns_dd']:+d})")
            L.append(f"- first change: S {c['was']} -> {c['call']}, `{c['rule']}` \"{c['explanation']}\"")
            fc = Counter(br.fmt_contract(x) for x in r["field_contracts"])
            L.append("- field: " + ", ".join(f"{k} ×{v}" if v > 1 else k for k, v in fc.most_common()) + "\n")
    if o["missing"]:
        L.append("\n## Not replayed\n")
        L.extend(f"- {m}" for m in o["missing"])
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("pbn", nargs="*", help="field PBN files (default: the main checkout's "
                    ".rbb-cache/brill/*/daily-*-field.pbn)")
    ap.add_argument("--card", default="21GF-GIB", help="North-South's card: stock name or file")
    ap.add_argument("--ew-card", help="East-West's card (default: --card)")
    ap.add_argument("--us", default="Rusty")
    ap.add_argument("--check", type=int, default=10, help="boards for the determinism check")
    ap.add_argument("--interval", type=float, default=1.0, help="seconds between requests (>= 1)")
    ap.add_argument("--offline", action="store_true", help="Brill cache only, no requests")
    ap.add_argument("--out-dir", default=os.path.join(CACHE, "brill"))
    ap.add_argument("--rbb", default=br.RBB)
    ap.add_argument("--dd-cache", default=os.path.join(CACHE, "dd.jsonl"))
    args = ap.parse_args()
    sys.exit(run(args))


if __name__ == "__main__":
    main()
