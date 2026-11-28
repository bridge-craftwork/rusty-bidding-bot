"""The call-grid report (call_grid.py report): numbers and tables from the
cached grid, competitive probe, weights and cross-check."""

import collections
import json
import os

import call_grid as g

LOW = g.RANK["3NT"]
TOP = g.RANK["5NT"]
SUIT = {"C": "♣", "D": "♦", "H": "♥", "S": "♠"}


def pretty(calls):
    out = []
    for c in calls:
        if c == "Pass":
            out.append("P")
        elif c in ("X", "XX"):
            out.append(c)
        elif c.endswith("NT"):
            out.append(c)
        else:
            out.append(c[0] + SUIT[c[1]])
    return " ".join(out) or "(opening)"


def pc(c):
    return pretty([c])


def opening_of(calls):
    return next((c for c in calls if c != "Pass"), None)


def seat_label(r):
    lead = 0
    for c in r["calls"]:
        if c != "Pass":
            break
        lead += 1
    return lead


def load(card):
    rows = [json.loads(l) for l in open(os.path.join(g.OUT, f"grid-{card}.jsonl"))]
    return [r for r in rows if "defined" in r]


def legal_bids(r, top=TOP):
    return [c for c in r["legal"] if c != "Pass" and c in g.RANK and g.RANK[c] <= top]


def coverage(rows):
    """By round: positions, legal bids (to 5NT), defined, through 3NT share, 4C-5NT share."""
    out = {}
    for rnd in range(1, 5):
        rs = [r for r in rows if r["n"] + 1 == rnd]
        tot = dfn = lo_t = lo_d = hi_t = hi_d = 0
        for r in rs:
            for c in legal_bids(r):
                d = c in r["defined"]
                tot += 1
                dfn += d
                if g.RANK[c] <= LOW:
                    lo_t += 1
                    lo_d += d
                else:
                    hi_t += 1
                    hi_d += d
        out[rnd] = (len(rs), tot, dfn, lo_t, lo_d, hi_t, hi_d)
    return out


def pct(a, b):
    return f"{100 * a / b:.0f}%" if b else "–"


def weighted(rows, pos, made):
    """Real calls made at grid positions (bids to 5NT): share defined, by source."""
    res = collections.defaultdict(lambda: [0, 0])
    for r in rows:
        k = " ".join(r["calls"])
        for c in legal_bids(r):
            m = made.get(k + " | " + c)
            if not m:
                continue
            for s, n in m.items():
                res[(s, r["n"] + 1)][0] += n
                res[(s, r["n"] + 1)][1] += n * (c in r["defined"])
    return res


def type_table(w, rows, pos, made, lo, hi, title):
    bytype = collections.defaultdict(lambda: collections.Counter())
    for r in rows:
        k = " ".join(r["calls"])
        reach = pos.get(k, {}).get("brill", 0)
        for c in legal_bids(r):
            if c in r["defined"] or not (lo <= g.RANK[c] <= hi):
                continue
            t = g.call_type(r["calls"], c)
            bytype[t][("n", r["n"] + 1)] += 1
            bytype[t][("w", r["n"] + 1)] += reach
            bytype[t]["madeb"] += made.get(k + " | " + c, {}).get("brill", 0)
    w(f"**{title}**")
    w("")
    w("| kind of call | 2nd call: holes (weight) | 3rd | 4th | bid by Brill at those holes |")
    w("|---|---|---|---|---|")
    order = sorted(bytype, key=lambda t: -sum(bytype[t][("w", i)] for i in (2, 3, 4)))
    for t in order:
        if t.startswith("opening"):
            continue
        cells = [f"{bytype[t][('n', i)]:,} ({bytype[t][('w', i)]:,})" for i in (2, 3, 4)]
        w(f"| {t} | " + " | ".join(cells) + f" | {bytype[t]['madeb']:,} |")
    w("")


def report(args):
    card = args.card
    rows = load(card)
    pos, made = g.load_weights()
    L = []
    w = L.append
    w("# The call grid (2026-11-27)")
    w("")
    w("Rick (2026-11-27): fill in the grid of which calls have a meaning; all competitive")
    w("auctions would get out of hand, so only a general description there; every system has")
    w("holes, which is not a problem in itself, but cross-check the common ones against Brill,")
    w("WBridge5 and GIB to see whether we miss anything basic. Companion to")
    w("[2026-11-27-undefined-calls.md](2026-11-27-undefined-calls.md) (holes in real auctions).")
    w("")
    w("## Method")
    w("")
    w(f"- **Our tree, uncontested** (`probes/tools/call_grid.py walk`): from the opening seat and")
    w("  the second seat (so every seat opens), the opponents passing throughout, every position")
    w("  of our own tree up to the fourth non-pass call. At each, `rbb bid rules --json` lists the")
    w("  calls some rule can make there (the engine's candidates, before the hand is known); every")
    w("  other legal bid up to 5NT is a **hole**. Children are the defined bids, so the tree is the")
    w("  one our own rules can reach. Pass is counted apart (a pass ruled out by a force is not a")
    w("  hole). Card `Brill` (what the seat service plays); 21GF-GIB beside it. Rules: pack")
    w("  `bottom-review` a3bffe8. Vulnerability none.")
    w("- **Competitive, one level** (`comp`): responder over our opening and their simple action;")
    w("  our first call over their opening, direct and pass-out seat; the advancer over a few common")
    w("  overcalls and doubles. A description, not a tree.")
    w("- **Weights** (`weigh`): each position and call counted in real auctions, the opening side")
    w("  bidding alone: the Brill field (all tables, 50 days), the BBA and GIB corpora")
    w("  (Practice-Bidding-Scenarios), and WBridge5 at all four seats (1,000 Pavlicek deals and 199")
    w("  slam deals from wbridge5_bid.py). Rules and card differ between corpora, so a position our")
    w("  tree reaches may be reached there with other meanings: frequency, not agreement.")
    w("- **Cross-check** (`cross`): the most common holes and every hole in rounds 1-2, against")
    w("  GIB's tree (the gibrest archive), Brill's `/explain` of the auction (Thorvald's service,")
    w("  one request a second, cached) and whether WBridge5 ever made the call there.")
    w("")

    # ---- coverage
    w("## 1. Coverage of our own tree (uncontested)")
    w("")
    for cname in ([card] + [c for c in ("21GF-GIB",) if c != card and os.path.exists(os.path.join(g.OUT, f"grid-{c}.jsonl"))]):
        rr = rows if cname == card else load(cname)
        cv = coverage(rr)
        w(f"**{cname}**")
        w("")
        w("| call | positions | legal bids to 5NT | with a meaning | through 3NT | 4♣ to 5NT |")
        w("|---|---|---|---|---|---|")
        for rnd, (n, tot, dfn, lt, ld, ht, hd) in cv.items():
            w(f"| {['1st (opening)', '2nd (response)', '3rd (opener rebid)', '4th (responder rebid)'][rnd - 1]} | {n:,} | {tot:,} | {dfn:,} ({pct(dfn, tot)}) | {pct(ld, lt)} | {pct(hd, ht)} |")
        w("")
    wt = weighted(rows, pos, made)
    w("Weighted by what was actually bid at those positions (share of real calls, bids to 5NT, that")
    w("have a meaning in our tree):")
    w("")
    w("| call | Brill field | GIB corpus | BBA corpus | WBridge5 |")
    w("|---|---|---|---|---|")
    for rnd in range(1, 5):
        cells = []
        for s in ("brill", "gib", "bba", "wb5"):
            t, d = wt.get((s, rnd), [0, 0])
            cells.append(f"{pct(d, t)} of {t:,}" if t else "–")
        w(f"| {rnd} | " + " | ".join(cells) + " |")
    w("")

    # ---- by opening
    w("### By opening")
    w("")
    w("Holes (bids to 5NT with no meaning) at responder's first call and over the next two")
    w("calls, by opening; `reached` = Brill-field auctions through the opening, uncontested.")
    w("")
    w("| opening | response: holes / legal | rounds 3-4: holes / legal | Brill reached | response holes |")
    w("|---|---|---|---|---|")
    byop = collections.defaultdict(lambda: [0, 0, 0, 0, []])
    for r in rows:
        if r["n"] == 0:
            continue
        o = opening_of(r["calls"])
        lb = legal_bids(r)
        hs = [c for c in lb if c not in r["defined"]]
        if r["n"] == 1:
            byop[o][0] += len(hs)
            byop[o][1] += len(lb)
            if seat_label(r) == 0:
                byop[o][4] = hs
        else:
            byop[o][2] += len(hs)
            byop[o][3] += len(lb)
    for o in sorted(byop, key=lambda c: g.RANK[c]):
        a, b, c3, d3, hs = byop[o]
        reached = pos.get(o, {}).get("brill", 0) + pos.get("Pass " + o, {}).get("brill", 0) \
            + pos.get("Pass Pass " + o, {}).get("brill", 0) + pos.get("Pass Pass Pass " + o, {}).get("brill", 0)
        lohs = [x for x in hs if g.RANK[x] <= g.RANK["4S"]]
        w(f"| {pc(o)} | {a}/{b} | {c3}/{d3} | {reached:,} | {' '.join(pc(x) for x in lohs[:14])}{' …' if len(lohs) > 14 else ''} |")
    w("")

    # ---- by type
    w("## 2. Where the holes are, in general terms")
    w("")
    w("Holes by round and kind of call (count of holes at positions of our tree; `weight` =")
    w("Brill-field auctions reaching those positions, so a hole at a common position counts more).")
    w("")
    for lo, hi, title in ((0, LOW, "Bids through 3NT"), (LOW + 1, TOP, "Bids from 4♣ to 5NT")):
        type_table(w, rows, pos, made, lo, hi, title)
    op = [r for r in rows if r["n"] == 0 and seat_label(r) == 0]
    if op:
        r = op[0]
        hs = [c for c in legal_bids(r) if c not in r["defined"]]
        w(f"Openings with no meaning (first seat, card {card}): {' '.join(pc(c) for c in hs) or 'none'}.")
        w("")

    # ---- comp
    cp = os.path.join(g.OUT, f"comp-{card}.json")
    if os.path.exists(cp):
        comp = json.load(open(cp))
        w("## 3. One level of interference")
        w("")
        w("Share of legal calls through 3NT (with X/XX where legal) that have a meaning, and the")
        w("holes through 3NT. `Brill` = Brill-field auctions through the position.")
        w("")
        fams = collections.OrderedDict([("responder over our opening and their action", []),
                                        ("our first call over their opening, direct seat", []),
                                        ("pass-out seat", []), ("advancer", [])])
        for r in comp:
            if "defined" not in r:
                continue
            lab = r["label"]
            f = ("advancer" if "advancer" in lab else "pass-out seat" if "pass-out" in lab
                 else "our first call over their opening, direct seat" if lab.startswith("their")
                 else "responder over our opening and their action")
            fams[f].append(r)
        for f, rs in fams.items():
            tot = dfn = 0
            for r in rs:
                lb = [c for c in r["legal"] if c != "Pass" and (c in ("X", "XX") or g.RANK[c] <= LOW)]
                tot += len(lb)
                dfn += sum(c in r["defined"] for c in lb)
            w(f"**{f}**: {dfn}/{tot} ({pct(dfn, tot)}) of calls through 3NT have a meaning.")
            w("")
            w("| position | defined | holes through 3NT | Brill |")
            w("|---|---|---|---|")
            for r in rs:
                lb = [c for c in r["legal"] if c != "Pass" and (c in ("X", "XX") or g.RANK[c] <= LOW)]
                hs = [c for c in lb if c not in r["defined"]]
                k = " ".join(r["calls"])
                b = pos.get("comp " + k, {}).get("brill", 0)
                lead = {"N": "", "W": ""}[r["dealer"]]
                w(f"| {pretty(r['calls'])} | {len(lb) - len(hs)}/{len(lb)} | {' '.join(pc(c) for c in hs) or '—'} | {b:,} |")
            w("")

    # ---- cross
    xp = os.path.join(g.OUT, f"cross-{card}.json")
    if os.path.exists(xp):
        xs = json.load(open(xp))
        cov = {}
        cvp = os.path.join(g.OUT, f"covers-{card}.json")
        if os.path.exists(cvp):
            cov = json.load(open(cvp))
        for x in xs:
            b = x["brill"] or {}
            # Brill: an explicit rule set (not its inference of a natural meaning)
            bd = bool(b.get("system") and b.get("set") and not b.get("fallback"))
            gd = bool(x["gib"])
            gnamed = gd and " -- " in x["gib"]
            if bd and gd:
                x["cls"] = "basic"
            elif bd or gnamed:
                x["cls"] = "convention-dependent"
            else:
                x["cls"] = "nobody defines it"
            x["cov"] = cov.get(" ".join(x["calls"]) + " | " + x["call"], {})
        cnt = collections.Counter(x["cls"] for x in xs)
        w("## 4. Cross-check of the common holes")
        w("")
        w(f"{len(xs)} holes: the {sum(1 for x in xs if x['why'] == 'top')} most common by real-auction")
        w(f"weight and {sum(1 for x in xs if x['why'] != 'top')} more in rounds 1-2. GIB = its tree has a")
        w("meaning; Brill = `/explain` reads the call by a system rule (not a fallback); WB5 = WBridge5")
        w("made the call there in its own auctions (its data is thin: 1,199 deals). GIB's tree")
        w("answers almost every call (a natural description when nothing else applies), and Brill")
        w("infers a natural meaning for calls no rule set names (`Inferred (...)`), so the test is")
        w("narrower than \"has a meaning\". Classes: **basic** = a named Brill rule set reads it and")
        w("GIB has it (both systems carry the call explicitly: likely something we miss);")
        w("**convention-dependent** = only one of them names it (a Brill rule set, or a GIB")
        w("convention name); **nobody defines it** = Brill only infers it and GIB only describes it")
        w("(fine, or a natural meaning nobody bothered to write down).")
        w("")
        w(f"Counts: basic {cnt['basic']}, convention-dependent {cnt['convention-dependent']}, nobody {cnt['nobody defines it']}.")
        w("")
        GIB_GENERIC = ("strong rebiddable", "rebiddable", "No suitable call")
        for cls in ("basic", "convention-dependent", "nobody defines it"):
            sel = [x for x in xs if x["cls"] == cls]
            if not sel:
                continue
            sel.sort(key=lambda x: (x["n"], -x["score"][0], -x["score"][1]))
            w(f"### {cls} ({len(sel)})")
            w("")
            w("| position | call | kind | Brill reached / bid it | GIB | Brill | WB5 | branch |")
            w("|---|---|---|---|---|---|---|---|")
            for x in sel:
                bm = x["brill"] or {}
                bmeans = (bm.get("set") or "") + (f": {bm['means']}" if bm.get("means") else "")
                if not (bm.get("system") and not bm.get("fallback")):
                    bmeans = "—" if not bm else f"({'fallback: ' if bm.get('fallback') else 'no system rule'}{bm.get('set') or ''})"
                cv = ", ".join(f"{b}: {v[0]}" for b, v in x["cov"].items()) or ""
                reach = x["reached"].get("brill", 0)
                madeb = x["made"].get("brill", 0)
                w(f"| {pretty(x['calls'])} | {pc(x['call'])} | {x['type']} | {reach:,} / {madeb:,} | "
                  f"{(x['gib'] or '—')[:70]} | {bmeans[:70]} | {x['wb5_made'] or ''} | {cv} |")
            w("")

    xcp = os.path.join(g.OUT, f"cross-comp-{card}.json")
    if os.path.exists(xcp):
        xs = json.load(open(xcp))
        for x in xs:
            b = x["brill"] or {}
            bd = bool(b.get("system") and b.get("set") and not b.get("fallback"))
            gd = bool(x["gib"])
            x["cls"] = ("basic" if bd and gd else
                        "convention-dependent" if bd or (gd and " -- " in x["gib"]) else "nobody defines it")
        cnt = collections.Counter(x["cls"] for x in xs)
        w("### Competitive holes, one level")
        w("")
        w(f"The {len(xs)} holes through 3NT of section 3 at positions the Brill field reached at least")
        w("ten times, the same test. Counts: " + ", ".join(f"{k} {v}" for k, v in cnt.most_common()) + ".")
        w("")
        w("| position | call | Brill reached / bid it | GIB | Brill | class |")
        w("|---|---|---|---|---|---|")
        order = {"basic": 0, "convention-dependent": 1, "nobody defines it": 2}
        xs.sort(key=lambda x: (order[x["cls"]], -x["reached"].get("brill", 0), x["calls"], g.RANK.get(x["call"], 99)))
        for x in xs:
            b = x["brill"] or {}
            bm = (b.get("set") or "") + (f": {b['means']}" if b.get("means") else "")
            if not b.get("set"):
                bm = f"({b.get('means') or '—'})"
            w(f"| {pretty(x['calls'])} | {pc(x['call'])} | {x['reached'].get('brill', 0):,} / {x['made'].get('brill', 0):,} | "
              f"{(x['gib'] or '—')[:60]} | {bm[:60]} | {x['cls']} |")
        w("")

    with open(args.out, "w") as f:
        f.write("\n".join(L) + "\n")
    print(f"wrote {args.out}")
