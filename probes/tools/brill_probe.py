#!/usr/bin/env python3
"""Where our reading of auctions on the 21GF-GIB card differs from Brill's.

    probes/tools/brill_probe.py auctions [--compare ALL.json] [--top 2000]
    probes/tools/brill_probe.py fetch    [--top 2000] [--max N]
    probes/tools/brill_probe.py report   [--top 2000] [--card cards/bbsa/21GF-GIB.bbsa]
    probes/tools/brill_probe.py run      [--top 2000]     # all three

Rusty plays the Brill dailies with three Brill robots (GIB-style 2/1).
Every call of Brill's we misread, or of ours that Brill reads otherwise,
costs us. This tool asks Thorvald Aagaard's public Brill service what
each call of an auction means (`GET /explain?auction=1N P 2D P 3C`) and
compares it, call by call, with `rbb explain-auction --json` on the same
auction (dealer N, vulnerability None, MP; the service's /explain takes no
seat or vulnerability, and dealer N keeps both readings on the same seats).

**auctions**: the distinct auctions our engine bid in a full corpus compare
JSON (`ours`), ranked by how many boards bid them; the top N go to
`.rbb-cache/brill-service/auctions-top<N>.json` (auction, boards, and how
many of those boards had 21GF-GIB on the bidding side), so a run is
reproducible without the 390 MB compare file.

**fetch**: Brill's /explain for each listed auction. Courtesy limits
(a public service run by one person): sequential, at most one request per
second, a descriptive User-Agent, every response cached in
`.rbb-cache/brill-service/explain/<sha1 of the Brill auction>.json` and
never fetched again (a rerun resumes where it stopped), exponential back
off on errors and a stop after 5 consecutive failures. Failures are logged
in `.rbb-cache/brill-service/failures.jsonl`.

**report**: per call, the caller's knowledge after it: Brill's
`extractedConstraints` of its first (winning) explanation against our
`step.knowledge`. A difference is counted at the call that made it (one
side narrowed that dimension at this call), not again at later calls:
- hcp: ranges disjoint (`conflict`), or an end more than `--hcp-tol`
  (default 2) apart; an open top (Brill none, ours >= 25) counts as 37;
- suit length: minimums differ (3+ against 4+), or maximums differ where
  either is 5 or less (a shown limit; both maxima are first tightened by
  the other suits' minimums, which our engine does and Brill does not);
- artificial against natural;
- a call one side has no rule for (ours: no rule; Brill: no system rule,
  only an "Inferred (...)" reading).
Writes `report-top<N>.md` and `.json`: totals, the differences grouped by
(our rule, Brill rule set) weighted by the corpus boards whose auctions
carry them, with examples; and Brill conventions seen, with how we read
the calls that carry them.
"""

import argparse
import ast
import hashlib
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

BASE = "https://brillservice.aalborgdata.dk"
UA = "rusty-bidding-bot brill_probe (rick.wilson.ca@gmail.com)"
ROOT = Path(__file__).resolve().parents[2]


def main_checkout():
    """The main checkout (a worktree shares its .rbb-cache, so the cache outlives it)."""
    try:
        g = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--path-format=absolute",
                            "--git-common-dir"], capture_output=True, text=True, check=True).stdout.strip()
        return Path(g).parent
    except (subprocess.CalledProcessError, OSError):
        return ROOT


MAIN_CACHE = main_checkout() / ".rbb-cache"
DEFAULT_COMPARE = MAIN_CACHE / "all-01f4f27.json"
STRAIN = {"Clubs": "C", "Diamonds": "D", "Hearts": "H", "Spades": "S", "NoTrump": "NT"}
SEATS = "NESW"
SUITS = "CDHS"  # our `len` order
BRILL_LEN = {"C": "clubLength", "D": "diamondLength", "H": "heartLength", "S": "spadeLength"}
SYM = {"C": "♣", "D": "♦", "H": "♥", "S": "♠"}


def cache_dir(args):
    d = Path(args.cache) if args.cache else MAIN_CACHE / "brill-service"
    d.mkdir(parents=True, exist_ok=True)
    return d


# ---------------------------------------------------------------- auctions

def call_str(c):
    if isinstance(c, str):
        return {"Pass": "Pass", "Double": "X", "Redouble": "XX"}[c]
    b = c["Bid"]
    return f"{b['level']}{STRAIN[b['strain']]}"


def to_brill(auction):
    return " ".join({"Pass": "P"}.get(t, t.replace("NT", "N")) for t in auction.split())


def sha(brill_auction):
    return hashlib.sha1(brill_auction.encode()).hexdigest()


def cmd_auctions(args):
    out = cache_dir(args) / f"auctions-top{args.top}.json"
    print(f"reading {args.compare} ...", file=sys.stderr)
    data = json.load(open(args.compare))
    boards = Counter()
    gib = Counter()
    for b in data["boards"]:
        calls = b["ours"]
        if isinstance(calls, str):
            calls = ast.literal_eval(calls)
        if not calls:
            continue
        a = " ".join(call_str(c) for c in calls)
        boards[a] += 1
        if "GIB" in (b.get("ns_card") or "") or "GIB" in (b.get("ew_card") or ""):
            gib[a] += 1
    total = sum(boards.values())
    top = boards.most_common(args.top)
    covered = sum(n for _, n in top)
    rows = [{"auction": a, "brill": to_brill(a), "boards": n, "gib_boards": gib[a]} for a, n in top]
    json.dump({"source": str(args.compare), "boards": total, "distinct": len(boards),
               "top": args.top, "covered_boards": covered, "auctions": rows},
              open(out, "w"), indent=1)
    print(f"{len(boards)} distinct auctions on {total} boards; top {args.top} cover "
          f"{covered} boards ({100 * covered / total:.1f}%) -> {out}")


def load_list(args):
    p = cache_dir(args) / f"auctions-top{args.top}.json"
    if not p.exists():
        sys.exit(f"{p} missing: run `auctions` first")
    return json.load(open(p))


# ------------------------------------------------------------------- fetch

def fetch_one(brill_auction):
    url = f"{BASE}/explain?auction={urllib.parse.quote(brill_auction)}"
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read()


def cmd_fetch(args):
    cd = cache_dir(args)
    ed = cd / "explain"
    ed.mkdir(exist_ok=True)
    lst = load_list(args)["auctions"]
    todo = [r for r in lst if not (ed / f"{sha(r['brill'])}.json").exists()]
    if args.max:
        todo = todo[: args.max]
    print(f"{len(lst)} auctions, {len(lst) - len(todo)} cached, fetching {len(todo)}", flush=True)
    fails = 0
    done = 0
    errors = 0
    t0 = time.time()
    last = 0.0
    for r in todo:
        delay = 1.0
        while True:
            wait = last + args.interval - time.time()
            if wait > 0:
                time.sleep(wait)
            last = time.time()
            try:
                raw = fetch_one(r["brill"])
                d = json.loads(raw)
                if not isinstance(d, list):
                    raise ValueError(f"not a list: {raw[:200]!r}")
                tmp = ed / f"{sha(r['brill'])}.json.tmp"
                tmp.write_bytes(raw)
                tmp.rename(ed / f"{sha(r['brill'])}.json")
                fails = 0
                done += 1
                break
            except (urllib.error.URLError, ValueError, TimeoutError, OSError) as e:
                errors += 1
                fails += 1
                code = getattr(e, "code", None)
                with open(cd / "failures.jsonl", "a") as f:
                    f.write(json.dumps({"auction": r["brill"], "error": str(e)[:300], "code": code,
                                        "time": time.strftime("%Y-%m-%dT%H:%M:%S")}) + "\n")
                print(f"error on {r['brill']!r}: {str(e)[:120]}", flush=True)
                if fails >= 5:
                    print(f"stopping: {fails} consecutive failures; rerun to resume", flush=True)
                    report_fetch(done, errors, t0)
                    sys.exit(1)
                if code and 400 <= code < 500 and code != 429:
                    break  # this auction is refused; log it and go on
                time.sleep(delay)
                delay = min(delay * 4, 300)
        if done and done % 100 == 0:
            el = time.time() - t0
            print(f"{done}/{len(todo)} fetched, {errors} errors, {el / 60:.1f} min", flush=True)
    report_fetch(done, errors, t0)


def report_fetch(done, errors, t0):
    print(f"fetched {done}, {errors} errors, {(time.time() - t0) / 60:.1f} min", flush=True)


# ---------------------------------------------------------- our readings

def our_reading(rbb, card, rules, auction):
    p = subprocess.run([rbb, "explain-auction", "-a", auction, "-d", "N", "-c", card,
                        "--rules", rules, "--json"], capture_output=True, text=True)
    if p.returncode != 0:
        return {"error": p.stderr.strip()[:500]}
    return json.loads(p.stdout)


# ---------------------------------------------------------------- compare

UNB = 37


def brill_state(c):
    """Brill's constraints for the caller. hcp: (min, max), None where Brill
    says nothing (an end Brill leaves open is not compared); len {s: (lo, hi,
    max_given)} with hi tightened by the other suits' minimums."""
    if not c:
        return None
    h = c.get("hcp") or {}
    hmin, hmax = h.get("min"), h.get("max")
    if hmin == 0:
        hmin = None
    if hmax is not None and hmax >= 25:
        hmax = None
    ln, given = {}, {}
    for s in SUITS:
        x = c.get(BRILL_LEN[s]) or {}
        ln[s] = [x.get("min") or 0, 13 if x.get("max") is None else x["max"]]
        given[s] = x.get("max") is not None
    tighten(ln)
    return {"hcp": (hmin, hmax), "len": {s: (ln[s][0], ln[s][1], given[s]) for s in SUITS}}


def tighten(ln):
    for s in SUITS:
        others = sum(ln[t][0] for t in SUITS if t != s)
        ln[s][1] = min(ln[s][1], 13 - others)


def rng(r):
    hi = UNB if r["hi"] >= 25 else r["hi"]
    return (r["lo"], hi)


def our_state(k):
    """Ours: hcp (lo, hi) and the point ranges our rules speak in (declarer
    points `dp`, total points per strain `tp`): our rules count points with
    length or shortness, so the hcp floor alone is looser than the rule."""
    if not k:
        return None
    ln = {s: [k["len"][i]["lo"], k["len"][i]["hi"]] for i, s in enumerate(SUITS)}
    tighten(ln)
    strength = [rng(k["hcp"])]
    if k.get("dp"):
        strength.append(rng(k["dp"]))
    strength += [rng(x) for x in k.get("tp") or []]
    return {"hcp": rng(k["hcp"]), "strength": strength,
            "len": {s: (ln[s][0], ln[s][1], True) for s in SUITS}}


UNKNOWN_B = {"hcp": (None, None), "len": {s: (0, 13, False) for s in SUITS}}
UNKNOWN_O = {"hcp": (0, UNB), "strength": [(0, UNB)], "len": {s: (0, 13, True) for s in SUITS}}


def fmt_range(dim, r):
    lo, hi = r[0], r[1]
    if dim == "hcp":
        lo = lo or 0
        if hi is None or hi >= UNB:
            return f"{lo}+"
        return f"{lo}-{hi}"
    if hi >= 13 or hi > 7 and lo == 0:
        return f"{lo}+" if lo else "any"
    if lo == hi:
        return f"={lo}"
    return f"{lo}-{hi}"


def hcp_diff(ours, brill, tol):
    """Brill's hcp against the closest of our strength ranges (hcp, dp, tp)."""
    bmin, bmax = brill
    if bmin is None and bmax is None:
        return None
    lo_b = bmin if bmin is not None else 0
    hi_b = bmax if bmax is not None else UNB
    cands = ours["strength"]
    if all(c[1] < lo_b or hi_b < c[0] for c in cands):
        return "hcp-conflict"
    for lo, hi in cands:
        if (bmin is None or abs(lo - bmin) <= tol) and (bmax is None or abs(hi - bmax) <= tol):
            return None
    return "hcp"


def len_diff(ours, brill):
    (olo, ohi, _), (blo, bhi, bgiven) = ours, brill
    if ohi < blo or bhi < olo:
        return "len-conflict"
    if olo != blo:
        return "len-min"
    if min(ohi, bhi) <= 5 and ohi != bhi:
        # a limit Brill states differently, or one only we infer
        return "len-max" if bgiven else "len-max-ours-only"
    return None


# kinds that do not count as a disagreement (Brill states nothing there)
INFO_KINDS = {"len-max-ours-only", "hcp-ours-only"}


def real_diffs(c):
    return [d for d in c["diffs"] if d["kind"] not in INFO_KINDS]


def brill_entry(e):
    ex = e.get("explanation") or []
    x = ex[0] if ex else {}
    ri = x.get("explainingRuleInstance") or {}
    conv = e.get("convention") or ri.get("convention") or ri.get("resolvedConvention")
    rs = x.get("explainingRuleSetName")
    means = ri.get("resolvedMeans") or ""
    norule = not ex or not rs or not x.get("isSystemExplained", True) or means.startswith("Inferred")
    return {
        "set": rs or ("(inferred)" if ex else "(none)"),
        "means": means,
        "requires": ri.get("resolvedRequiresString") or "",
        "artificial": bool(ri.get("artificial")),
        "fallback": bool(ri.get("isFallbackOnly")),
        "convention": conv,
        "norule": norule,
        "constraints": x.get("extractedConstraints"),
        "alternatives": len(ex),
    }


def compare_auction(row, brill, ours, tol):
    """Per-call records."""
    out = []
    calls = row["auction"].split()
    if not isinstance(ours, list):
        return None
    if len(brill) != len(calls) or len(ours) != len(calls):
        return None
    prev_b = {p: UNKNOWN_B for p in SEATS}
    prev_o = {p: UNKNOWN_O for p in SEATS}
    for i, call in enumerate(calls):
        seat = SEATS[i % 4]
        be = brill_entry(brill[i])
        st = ours[i]["step"]
        rule = st.get("rule")
        rule_id = f"{rule['file'].split('conventions/')[-1]}:{rule['line']}" if rule else "(no rule)"
        bs = brill_state(be["constraints"]) or prev_b[seat]
        os_ = our_state(st.get("knowledge")) or prev_o[seat]
        pb, po = prev_b[seat], prev_o[seat]
        diffs = []
        # a dimension is compared at the call where either side narrowed it
        if bs["hcp"] != pb["hcp"] or os_["strength"] != po["strength"]:
            k = hcp_diff(os_, bs["hcp"], tol)
            if k == "hcp" and bs["hcp"] == pb["hcp"]:
                k = "hcp-ours-only"  # Brill's rule says nothing new on strength here
            if k:
                best = min(os_["strength"], key=lambda c: abs(c[0] - (bs["hcp"][0] or 0))
                           + abs(c[1] - (bs["hcp"][1] or UNB)))
                diffs.append({"kind": k, "dim": "hcp", "ours": os_["hcp"], "ours_pts": best,
                              "brill": bs["hcp"]})
        for s in SUITS:
            if bs["len"][s] == pb["len"][s] and os_["len"][s] == po["len"][s]:
                continue
            k = len_diff(os_["len"][s], bs["len"][s])
            if k == "len-max" and bs["len"][s] == pb["len"][s]:
                k = "len-max-ours-only"  # a limit only we draw (a denial by priority, say)
            if k:
                diffs.append({"kind": k, "dim": s, "ours": os_["len"][s][:2], "brill": bs["len"][s][:2]})
        art_o = bool(st.get("artificial"))
        if art_o != be["artificial"] and not be["norule"] and rule:
            diffs.append({"kind": "artificial", "ours": art_o, "brill": be["artificial"]})
        if not rule:
            diffs.append({"kind": "ours-no-rule"})
        if be["norule"]:
            diffs.append({"kind": "brill-no-rule"})
        out.append({
            "i": i, "seat": seat, "call": call, "prefix": " ".join(calls[:i]),
            "our_rule": rule_id, "our_module": rule["module"] if rule else None,
            "our_means": st.get("explanation") or "", "our_art": art_o,
            "brill": {k: v for k, v in be.items() if k != "constraints"},
            "diffs": diffs,
        })
        prev_b[seat], prev_o[seat] = bs, os_
    return out


def describe(d):
    k = d["kind"]
    if k.startswith("hcp"):
        pts = "" if tuple(d["ours_pts"]) == tuple(d["ours"]) else f" (pts {fmt_range('hcp', d['ours_pts'])})"
        return (f"HCP ours {fmt_range('hcp', d['ours'])}{pts} / Brill {fmt_range('hcp', d['brill'])}"
                f"{' CONFLICT' if k == 'hcp-conflict' else ''}")
    if k.startswith("len"):
        s = SYM[d["dim"]]
        return f"{s} ours {fmt_range('len', d['ours'])} / Brill {fmt_range('len', d['brill'])}"
    if k == "artificial":
        return f"artificial: ours {'yes' if d['ours'] else 'no'}, Brill {'yes' if d['brill'] else 'no'}"
    return k


def cmd_report(args):
    cd = cache_dir(args)
    lst = load_list(args)
    rows = lst["auctions"]
    ed = cd / "explain"
    have = [r for r in rows if (ed / f"{sha(r['brill'])}.json").exists()]
    print(f"{len(have)}/{len(rows)} auctions have Brill's reading; computing ours ...", file=sys.stderr)
    rbb = str(ROOT / "target/release/rbb")
    card = str((ROOT / args.card).resolve())
    rules = str(ROOT / "conventions")
    with ThreadPoolExecutor(args.jobs) as ex:
        ours = list(ex.map(lambda r: our_reading(rbb, card, rules, r["auction"]), have))

    calls = []
    skipped = []
    for r, o in zip(have, ours):
        b = json.load(open(ed / f"{sha(r['brill'])}.json"))
        recs = compare_auction(r, b, o, args.hcp_tol)
        if recs is None:
            skipped.append({"auction": r["auction"], "why": o.get("error") if isinstance(o, dict) else "length mismatch"})
            continue
        for c in recs:
            c["auction"] = r["auction"]
            c["boards"] = r["boards"]
            calls.append(c)

    # totals
    t = Counter()
    wt = Counter()
    for c in calls:
        kinds = {d["kind"] for d in c["diffs"]}
        real = kinds - INFO_KINDS
        t["calls"] += 1
        wt["calls"] += c["boards"]
        if "ours-no-rule" in kinds:
            t["ours_no_rule"] += 1; wt["ours_no_rule"] += c["boards"]
        if "brill-no-rule" in kinds:
            t["brill_no_rule"] += 1; wt["brill_no_rule"] += c["boards"]
        if kinds & {"ours-no-rule", "brill-no-rule"}:
            continue
        t["compared"] += 1
        wt["compared"] += c["boards"]
        if not real:
            t["agree"] += 1; wt["agree"] += c["boards"]
        for k in kinds:
            t["kind:" + k] += 1; wt["kind:" + k] += c["boards"]
        if c["call"] != "Pass":
            t["compared_bids"] += 1; wt["compared_bids"] += c["boards"]
            if not real:
                t["agree_bids"] += 1; wt["agree_bids"] += c["boards"]

    # groups: (our rule, Brill rule set) over calls with a difference
    groups = defaultdict(lambda: {"calls": 0, "boards": 0, "kinds": Counter(), "examples": [],
                                  "brill_means": Counter(), "our_means": Counter(), "calls_seen": Counter()})
    for c in calls:
        if not real_diffs(c):
            continue
        key = (c["our_rule"], c["brill"]["set"])
        g = groups[key]
        g["calls"] += 1
        g["boards"] += c["boards"]
        for d in c["diffs"]:
            g["kinds"][d["kind"]] += c["boards"]
        g["brill_means"][c["brill"]["means"] or c["brill"]["requires"]] += c["boards"]
        g["our_means"][c["our_means"]] += c["boards"]
        g["calls_seen"][c["call"]] += c["boards"]
        if len(g["examples"]) < 3:
            g["examples"].append({
                "auction": c["auction"], "at": c["i"], "call": c["call"], "seat": c["seat"],
                "boards": c["boards"], "ours": c["our_means"], "our_art": c["our_art"],
                "brill_means": c["brill"]["means"], "brill_requires": c["brill"]["requires"],
                "brill_art": c["brill"]["artificial"], "convention": c["brill"]["convention"],
                "diffs": [describe(d) for d in real_diffs(c)]
                + [f"({describe(d)}, ours only)" for d in c["diffs"] if d["kind"] in INFO_KINDS],
            })
    # agreement per group key too, for the share of that pairing that differs
    pair_total = Counter()
    for c in calls:
        pair_total[(c["our_rule"], c["brill"]["set"])] += c["boards"]
    glist = sorted(groups.items(), key=lambda kv: -kv[1]["boards"])

    # Brill conventions
    convs = defaultdict(lambda: {"calls": 0, "boards": 0, "our_modules": Counter(), "our_art": 0,
                                 "brill_art": 0, "agree": 0, "examples": []})
    for c in calls:
        cv = c["brill"]["convention"]
        if not cv:
            continue
        g = convs[cv]
        g["calls"] += 1
        g["boards"] += c["boards"]
        g["our_modules"][c["our_module"] or "(no rule)"] += c["boards"]
        g["our_art"] += c["boards"] if c["our_art"] else 0
        g["brill_art"] += c["boards"] if c["brill"]["artificial"] else 0
        g["agree"] += c["boards"] if not real_diffs(c) else 0
        if len(g["examples"]) < 2:
            g["examples"].append(f"{c['prefix']} **{c['call']}**: Brill {c['brill']['means']!r}; ours {c['our_means']!r}")

    out = {
        "brill_version": brill_version_cached(cd),
        "list": {k: v for k, v in lst.items() if k != "auctions"},
        "auctions": len(have), "skipped": skipped, "totals": dict(t), "board_weighted": dict(wt),
        "hcp_tol": args.hcp_tol,
        "groups": [{"our_rule": k[0], "brill_set": k[1], "calls": g["calls"], "boards": g["boards"],
                    "pair_boards": pair_total[k], "kinds": dict(g["kinds"]),
                    "our_means": g["our_means"].most_common(3), "brill_means": g["brill_means"].most_common(3),
                    "calls_seen": g["calls_seen"].most_common(5), "examples": g["examples"]}
                   for k, g in glist],
        "conventions": [{"convention": k, "calls": g["calls"], "boards": g["boards"],
                         "agree_boards": g["agree"], "brill_artificial_boards": g["brill_art"],
                         "our_artificial_boards": g["our_art"],
                         "our_modules": g["our_modules"].most_common(4), "examples": g["examples"]}
                        for k, g in sorted(convs.items(), key=lambda kv: -kv[1]["boards"])],
    }
    if args.calls_out:
        with open(cd / f"calls-top{args.top}.jsonl", "w") as f:
            for c in calls:
                f.write(json.dumps(c) + "\n")
    jp = cd / f"report-top{args.top}.json"
    json.dump(out, open(jp, "w"), indent=1, default=list)
    mp = cd / f"report-top{args.top}.md"
    mp.write_text(markdown(out, args.groups))
    print(f"-> {mp}\n-> {jp}", file=sys.stderr)


def brill_version_cached(cd):
    p = cd / "version.json"
    if not p.exists():
        try:
            req = urllib.request.Request(f"{BASE}/version", headers={"User-Agent": UA})
            p.write_bytes(urllib.request.urlopen(req, timeout=30).read())
        except Exception as e:  # noqa: BLE001
            return {"error": str(e)}
    return json.load(open(p))


def pct(a, b):
    return f"{100 * a / b:.1f}%" if b else "-"


def markdown(o, ngroups):
    t, w = o["totals"], o["board_weighted"]
    L = []
    v = o["brill_version"]
    L.append(f"# Brill vs Rusty (21GF-GIB): how each reads the top {o['list']['top']} auctions\n")
    L.append(f"Brill service {v.get('version', '?')} ({v.get('ruleCount', '?')} rules); corpus "
             f"{o['list']['boards']} boards, {o['list']['distinct']} distinct auctions; these "
             f"{o['auctions']} cover {o['list']['covered_boards']} boards. Dealer N, vul None, MP. "
             f"HCP tolerance ±{o['hcp_tol']}. A difference is counted at the call that made it.\n")
    L.append("## Totals\n")
    L.append("| | calls | board-weighted |\n|---|---:|---:|")
    L.append(f"| calls | {t.get('calls', 0)} | {w.get('calls', 0)} |")
    L.append(f"| no rule, ours | {t.get('ours_no_rule', 0)} | {w.get('ours_no_rule', 0)} |")
    L.append(f"| no system rule, Brill (inferred) | {t.get('brill_no_rule', 0)} | {w.get('brill_no_rule', 0)} |")
    L.append(f"| compared (both have a rule) | {t.get('compared', 0)} | {w.get('compared', 0)} |")
    L.append(f"| agree within tolerance | {t.get('agree', 0)} ({pct(t.get('agree', 0), t.get('compared', 0))}) "
             f"| {w.get('agree', 0)} ({pct(w.get('agree', 0), w.get('compared', 0))}) |")
    L.append(f"| bids (not Pass) compared | {t.get('compared_bids', 0)} | {w.get('compared_bids', 0)} |")
    L.append(f"| bids agreeing | {t.get('agree_bids', 0)} ({pct(t.get('agree_bids', 0), t.get('compared_bids', 0))}) "
             f"| {w.get('agree_bids', 0)} ({pct(w.get('agree_bids', 0), w.get('compared_bids', 0))}) |")
    for k in sorted(k for k in t if k.startswith("kind:")):
        L.append(f"| differ: {k[5:]} | {t[k]} | {w[k]} |")
    if o["skipped"]:
        L.append(f"\n{len(o['skipped'])} auctions skipped (no reading on one side).")
    L.append("\n## Differences by (our rule, Brill rule set), by corpus boards\n")
    L.append("`boards` counts each board once per differing call; `of` is the boards of all calls "
             "with that pairing (agreeing or not).\n")
    for n, g in enumerate(o["groups"][:ngroups], 1):
        kinds = ", ".join(f"{k} {v}" for k, v in sorted(g["kinds"].items(), key=lambda kv: -kv[1]))
        calls = ", ".join(f"{c} {n_}" for c, n_ in g["calls_seen"])
        L.append(f"### {n}. `{g['our_rule']}` / Brill \"{g['brill_set']}\": {g['boards']} boards "
                 f"(of {g['pair_boards']}), {g['calls']} calls\n")
        L.append(f"- calls: {calls}; kinds: {kinds}")
        L.append(f"- ours: {'; '.join(repr(m) for m, _ in g['our_means'])}")
        L.append(f"- Brill: {'; '.join(repr(m) for m, _ in g['brill_means'])}")
        for e in g["examples"]:
            L.append(f"- `{e['auction']}` call {e['at'] + 1} ({e['seat']} {e['call']}, {e['boards']} boards): "
                     f"{'; '.join(e['diffs'])}. Brill: {e['brill_means']!r} [{e['brill_requires']}]"
                     f"{' conv ' + e['convention'] if e['convention'] else ''}")
        L.append("")
    L.append("## Brill conventions seen\n")
    L.append("| convention | calls | boards | agree | Brill art. | our art. | our modules |\n|---|---:|---:|---:|---:|---:|---|")
    for c in o["conventions"]:
        mods = ", ".join(f"{m} {n}" for m, n in c["our_modules"])
        L.append(f"| {c['convention']} | {c['calls']} | {c['boards']} | {pct(c['agree_boards'], c['boards'])} | "
                 f"{c['brill_artificial_boards']} | {c['our_artificial_boards']} | {mods} |")
    L.append("")
    for c in o["conventions"]:
        for e in c["examples"][:1]:
            L.append(f"- {c['convention']}: {e}")
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cmd", choices=["auctions", "fetch", "report", "run"])
    ap.add_argument("--compare", default=str(DEFAULT_COMPARE))
    ap.add_argument("--top", type=int, default=2000)
    ap.add_argument("--cache", help="cache dir (default: the main checkout's .rbb-cache/brill-service)")
    ap.add_argument("--interval", type=float, default=1.0, help="seconds between requests (>= 1)")
    ap.add_argument("--max", type=int, default=0, help="fetch at most N this run")
    ap.add_argument("--card", default="cards/bbsa/21GF-GIB.bbsa")
    ap.add_argument("--hcp-tol", type=int, default=2)
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--groups", type=int, default=60, help="groups in the markdown")
    ap.add_argument("--calls-out", action="store_true", help="also write calls-top<N>.jsonl")
    args = ap.parse_args()
    args.interval = max(args.interval, 1.0)
    if args.cmd in ("auctions", "run"):
        if args.cmd == "auctions" or not (cache_dir(args) / f"auctions-top{args.top}.json").exists():
            cmd_auctions(args)
    if args.cmd in ("fetch", "run"):
        cmd_fetch(args)
    if args.cmd in ("report", "run"):
        cmd_report(args)


if __name__ == "__main__":
    main()
