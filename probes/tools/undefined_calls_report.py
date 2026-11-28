"""Report for undefined_calls.py (see its docstring)."""

import collections
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(__file__))
import undefined_calls as U  # noqa: E402

STR = "CDHS"
SUITNAME = {"C": "♣", "D": "♦", "H": "♥", "S": "♠"}


def load_answers(card):
    p = os.path.join(U.OUT, f"answers-{card}.jsonl")
    a = {}
    with open(p) as f:
        for l in f:
            r = json.loads(l)
            a[tuple(json.loads(r["key"]))] = r
    return a


def level_strain(c):
    m = re.fullmatch(r"([1-7])(C|D|H|S|NT)", c)
    return (int(m.group(1)), m.group(2)) if m else None


def rank(c):
    ls = level_strain(c)
    return (ls[0] - 1) * 5 + "CDHSNT".index(ls[1] if ls[1] != "NT" else "NT") if ls else None


def rank_of(ls):
    lv, st = ls
    return (lv - 1) * 5 + ["C", "D", "H", "S", "NT"].index(st)


def describe(calls, i, dealer):
    """Role of the caller, the call's kind, contested or not."""
    side = lambda j: j % 2  # 0: dealer's side
    first = next((j for j, c in enumerate(calls[: i + 1]) if c != "Pass"), None)
    opening_side = side(first)
    me = side(i)
    # roles
    bidders = {}
    for j, c in enumerate(calls[:i]):
        if c != "Pass" and j % 4 not in bidders:
            bidders[j % 4] = None
    if me == opening_side:
        opener = first % 4
        role = "opener" if i % 4 == opener else "responder"
    else:
        defs = [j for j, c in enumerate(calls[:i]) if c != "Pass" and side(j) != opening_side]
        if not defs:
            role = "defender (first action)"
        else:
            role = "overcaller" if i % 4 == defs[0] % 4 else "advancer"
    contested = any(c != "Pass" and side(j) != opening_side for j, c in enumerate(calls[: i + 1]))
    # kind of call
    c = calls[i]
    own_suits, their_suits = set(), set()
    for j, cc in enumerate(calls[:i]):
        ls = level_strain(cc)
        if ls and ls[1] != "NT":
            (own_suits if side(j) == me else their_suits).add(ls[1])
    last_bid = None
    for cc in calls[:i]:
        if level_strain(cc):
            last_bid = level_strain(cc)
    if c in ("X", "XX"):
        kind = "double" if c == "X" else "redouble"
    else:
        lv, st = level_strain(c)
        minlv = 1
        if last_bid:
            minlv = last_bid[0] + (1 if rank_of((last_bid[0], st)) <= rank_of(last_bid) else 0)
        jump = lv > minlv
        if st == "NT":
            kind = "NT"
        elif st in their_suits:
            kind = "cue"
        elif st in own_suits:
            kind = "raise/rebid"
        else:
            kind = "new suit"
        if jump:
            kind = "jump " + kind
    return role, kind, contested


def report(args):
    auctions = U.load_auctions()
    cards = {}
    for card in ("Brill", "21GF-GIB"):
        try:
            cards[card] = load_answers(card)
        except OSError:
            pass
    main = cards[args.card]

    def unread_list(a, ans):
        cc = U.cut(a["calls"])
        while cc and cc[-1] == "Pass":
            cc = cc[:-1]
        r = ans.get((a["dealer"], a["vul"], " ".join(cc)))
        if not r or "unread" not in r:
            return None, cc
        return r, cc

    stats = {card: collections.Counter() for card in cards}
    pairs = collections.defaultdict(lambda: {"n": collections.Counter(), "ex": None, "brill_note": None,
                                             "gib": None, "role": None, "kind": None, "contested": None,
                                             "seat": collections.Counter()})
    fam = collections.defaultdict(collections.Counter)
    missing = 0
    for card, ans in cards.items():
        for a in auctions:
            r, cc = unread_list(a, ans)
            if r is None:
                missing += card == args.card
                continue
            n = 0
            seen_unread = False
            for i, c in enumerate(cc):
                if c == "Pass":
                    continue
                n += 1
                role, kind, contested = describe(cc, i, a["dealer"])
                seat = U.seat_of(a["dealer"], i)
                un = r["unread"][i]
                base = (a["src"], n, contested)
                stats[card][("calls",) + base] += 1
                if a["src"] == "brill":
                    stats[card][("brillseat", seat, "calls")] += 1
                if un:
                    stats[card][("unread",) + base] += 1
                    if not seen_unread:
                        stats[card][("first-unread",) + base] += 1
                    if a["src"] == "brill":
                        stats[card][("brillseat", seat, "unread")] += 1
                        if not seen_unread:
                            stats[card][("brillseat", seat, "first-unread")] += 1
                    if card == args.card and not seen_unread:
                        rel = [x for x in cc[:i]]
                        k = (U.gib_key(rel), c)
                        p = pairs[k]
                        p["n"][a["src"]] += 1
                        p["seat"][seat if a["src"] == "brill" else "-"] += 1
                        p["role"], p["kind"], p["contested"] = role, kind, contested
                        if p["ex"] is None or (a["src"] == "brill" and p["ex"][0] != "brill"):
                            p["ex"] = (a["src"], a["dealer"], " ".join(cc[: i + 1]), a["file"])
                            if a["src"] == "brill":
                                p["brill_note"] = a["notes"][i] if i < len(a["notes"]) else None
                        if p["gib"] is None:
                            t = U.gib_tree(rel)
                            if t is not None:
                                gc = {"X": "D", "XX": "R"}.get(c, c.replace("NT", "N"))
                                p["gib"] = t.get(gc, "(not in GIB's tree)") or "(GIB: no meaning text)"
                        fam[(role, n, kind, contested)][a["src"]] += 1
                    seen_unread = True

    # GIB-defined calls at real positions (dealer-relative, asked with dealer N)
    posfreq = collections.Counter()
    for a in auctions:
        n = 0
        for i, c in enumerate(a["calls"]):
            if n >= U.MAXN:
                break
            posfreq[(a["src"], tuple(a["calls"][:i]))] += 1
            if c != "Pass":
                n += 1
    gibdef = collections.defaultdict(lambda: collections.Counter())
    gibmean = {}
    seen = set()
    for (src, pre), w in posfreq.items():
        if sum(1 for c in pre if c != "Pass") >= U.MAXN:
            continue
        t = U.gib_tree(list(pre))
        if not t:
            continue
        for gc, m in t.items():
            c = U.our_from_gib(gc)
            if c == "Pass" or not m:
                continue
            ls = level_strain(c)
            if ls and ls[0] >= 5:  # low levels only
                continue
            calls = list(pre) + [c]
            r = main.get(("N", "None", " ".join(calls)))
            if not r or "unread" not in r:
                continue
            # unread here, and everything before read
            if r["unread"][len(pre)] and not any(r["unread"][: len(pre)]):
                gibdef[(U.gib_key(list(pre)), c)][src] += w
                gibmean[(U.gib_key(list(pre)), c)] = m

    # holes: Brill South positions where no rule applies
    holes = collections.defaultdict(lambda: {"n": 0, "unread_before": 0, "ex": None, "calls": collections.Counter()})
    hole_total = collections.Counter()
    for a in auctions:
        if a["src"] != "brill":
            continue
        n = 0
        for i, c in enumerate(a["calls"]):
            if n >= U.MAXN:
                break
            if U.seat_of(a["dealer"], i) == "S":
                r = main.get((a["dealer"], a["vul"], " ".join(a["calls"][:i]), a["deal"]["S"]))
                if r and "decision" in r:
                    hole_total["positions"] += 1
                    d = r["decision"]
                    if not d["rule"] and (d["explanation"] or "").startswith("No rule"):
                        hole_total["no rule"] += 1
                        ub = any(r["unread"])
                        hole_total["no rule, a call before unread" if ub else "no rule, every call read"] += 1
                        h = holes[U.gib_key(a["calls"][:i])]
                        h["n"] += 1
                        h["unread_before"] += ub
                        h["calls"][c] += 1
                        if h["ex"] is None:
                            h["ex"] = (a["file"], a["deal"]["S"])
            if c != "Pass":
                n += 1

    write(args, auctions, cards, stats, pairs, fam, gibdef, gibmean, holes, hole_total, missing)


def pct(a, b):
    return f"{100 * a / b:.1f}%" if b else "–"


def in_flight(key, call, role, n, contested):
    calls = key.split("-") if key != "_" else []
    s = "-".join(c for c in calls)
    tags = []
    nonpass = [c for c in calls if c != "P"]
    if re.search(r"(^|-)1[HS]-P-1N-P-2N-P$", s) and not contested:
        tags.append("gf-after-2nt")
    if role == "opener" and contested and n >= 3:
        tags.append("second-calls-opener (F1/F14)")
    if role in ("overcaller",) and contested:
        tags.append("second-calls-defence (F2/F3/F4)")
    if role == "advancer" and contested and n >= 3:
        tags.append("second-calls-defence (F2/F3/F4)?")
    return ", ".join(tags)


def write(args, auctions, cards, stats, pairs, fam, gibdef, gibmean, holes, hole_total, missing):
    L = []
    w = L.append
    src_n = collections.Counter(a["src"] for a in auctions)
    w("# Calls our engine cannot read, up to the fourth non-pass call (2026-11-27)")
    w("")
    w("Rick (2026-11-27): most low-level auctions have a meaning for every bid; which ones does")
    w("Rusty have none for? Thorvald's example (1♠-1NT-2NT-3♥, Brill daily 2026-11-24 MP-strong")
    w("board 6) was the fourth non-pass call. Read-only probe: `probes/tools/undefined_calls.py`.")
    w("")
    w("## Method")
    w("")
    w(f"- **Auctions (real, not our self-play):** the Brill field PBNs, every table, {src_n['brill']:,}")
    w(f"  auctions over 50 days (North = Rusty's partner, a Brill robot; East/West Brill robots; South")
    w(f"  the entrant); the BBA corpus of Practice-Bidding-Scenarios, {src_n['bba']:,} (bba-cli, both")
    w(f"  sides on 2/1 cards; scenario deals, so skewed to each scenario's auction); the GIB corpus")
    w(f"  there, {src_n['gib']:,}. Plus every call GIB's tree defines at those positions where the")
    w("  tree is archived (no new gibrest requests).")
    w("- **Readings:** `rbb call --batch` once per distinct auction cut after its fourth non-pass")
    w("  call; `auction.position.unread` marks each call no rule reads (as explain-auction's")
    w("  \"(no rule)\"). Rules: the pack at `origin/bottom-review` (a3bffe8); engine v0.19.0 (main")
    w(f"  checkout binary). Card {args.card} (what the seat service plays); 21GF-GIB beside it.")
    w("- **First unread:** after one unread call the later ones are often unread too (the rules")
    w("  cannot follow). Tables count every unread call; the work queue counts only the first")
    w("  one in each auction.")
    w("- **Holes:** at every South position in the Brill field, the entrant's real hand asked of")
    w("  our engine: \"No rule applies\" is a pass that no rule makes.")
    if missing:
        w(f"- {missing} auctions could not be read by the engine (errors) and are left out.")
    w("")

    for card, st in stats.items():
        w(f"## Calls not read, card {card}")
        w("")
        w("| source | call | uncontested: calls | unread | first unread | contested: calls | unread | first unread |")
        w("|---|---|---|---|---|---|---|---|")
        for src in ("brill", "gib", "bba"):
            for n in range(1, U.MAXN + 1):
                row = [src, f"{n}"]
                for cont in (False, True):
                    c = st[("calls", src, n, cont)]
                    u = st[("unread", src, n, cont)]
                    f = st[("first-unread", src, n, cont)]
                    row += [f"{c:,}", f"{u:,} ({pct(u, c)})", f"{f:,} ({pct(f, c)})"]
                w("| " + " | ".join(row) + " |")
        w("")
        w("Brill field by seat (North is Rusty's partner; East/West the opponents; South the entrants):")
        w("")
        w("| seat | calls | unread | first unread |")
        w("|---|---|---|---|")
        for seat in "NESW":
            c = st[("brillseat", seat, "calls")]
            u = st[("brillseat", seat, "unread")]
            f = st[("brillseat", seat, "first-unread")]
            w(f"| {seat} | {c:,} | {u:,} ({pct(u, c)}) | {f:,} ({pct(f, c)}) |")
        w("")

    # families
    w(f"## Families (first unread call, card {args.card})")
    w("")
    w("Grouped by the caller's role, which non-pass call it is, the kind of call (relative to the")
    w("auction: new suit, raise or rebid of our suit, cue of theirs, NT, double; jump), and contested")
    w("or not. Ranked by Brill-field count, then GIB and BBA.")
    w("")
    w("| role | call # | kind | contested | Brill | GIB | BBA |")
    w("|---|---|---|---|---|---|---|")
    for (role, n, kind, cont), c in sorted(fam.items(), key=lambda kv: (-kv[1]["brill"], -kv[1]["gib"], -kv[1]["bba"]))[:40]:
        w(f"| {role} | {n} | {kind} | {'yes' if cont else 'no'} | {c['brill']} | {c['gib']} | {c['bba']} |")
    w("")

    # top pairs
    def table(title, order, limit):
        w(title)
        w("")
        w("| position (dealer first) | call | Brill (N/E/W/S) | GIB | BBA | role, call #, kind | Brill's note | GIB's meaning | in flight |")
        w("|---|---|---|---|---|---|---|---|---|")
        for (k, c), p in sorted(pairs.items(), key=order)[:limit]:
            n_calls = sum(1 for x in (k.split("-") if k != "_" else []) if x != "P") + 1
            seats = "/".join(str(p["seat"][s]) for s in "NEWS")
            note = (p["brill_note"] or "").replace("|", "/")[:90]
            gib = (p["gib"] or "").replace("|", "/")[:90]
            tag = in_flight(k, c, p["role"], n_calls, p["contested"])
            w(f"| {k} | {c} | {p['n']['brill']} ({seats}) | {p['n']['gib']} | {p['n']['bba']} | "
              f"{p['role']}, {n_calls}, {p['kind']}{', contested' if p['contested'] else ''} | {note} | {gib} | {tag} |")
        w("")

    table(f"## Top unread (position, call) in the Brill field (first unread; card {args.card})",
          lambda kv: (-kv[1]["n"]["brill"], -kv[1]["n"]["gib"]), 60)
    table("## Top unread (position, call) in the GIB and BBA corpora, not in the Brill field",
          lambda kv: (kv[1]["n"]["brill"] > 0, -(kv[1]["n"]["gib"] * 5 + kv[1]["n"]["bba"])), 40)

    w(f"## Calls GIB's tree defines that we cannot read (card {args.card})")
    w("")
    w("At real positions (fewer than four non-pass calls so far, every call before read), each")
    w("call up to the four level (and doubles) that GIB's tree gives a meaning and no rule of")
    w("ours reads, weighted by how often the")
    w("position occurs. Many are calls nobody makes; the frequent positions matter.")
    w("")
    w("| position | call | position seen: Brill | GIB | BBA | GIB's meaning |")
    w("|---|---|---|---|---|---|")
    for (k, c), cnt in sorted(gibdef.items(), key=lambda kv: (-kv[1]["brill"], -kv[1]["gib"], -kv[1]["bba"]))[:50]:
        w(f"| {k} | {c} | {cnt['brill']} | {cnt['gib']} | {cnt['bba']} | {gibmean[(k, c)].replace('|', '/')[:100]} |")
    w("")
    w(f"In all, {len(gibdef):,} (position, call) pairs GIB defines are unread by us; "
      f"{sum(1 for v in gibdef.values() if v['brill'])} of them at positions seen in the Brill field.")
    w("")

    w(f"## Holes: South positions where no rule applies (Brill field, card {args.card})")
    w("")
    tp = hole_total["positions"]
    w(f"{tp:,} South positions (entrants' real hands, up to the fourth non-pass call). "
      f"\"No rule applies\": {hole_total['no rule']:,} ({pct(hole_total['no rule'], tp)}); "
      f"{hole_total['no rule, every call read']:,} with every call before read (a hole in our rules for "
      f"that hand), {hole_total['no rule, a call before unread']:,} after a call we could not read.")
    w("")
    w("| position | times | after an unread call | what the entrants bid there |")
    w("|---|---|---|---|")
    for k, h in sorted(holes.items(), key=lambda kv: -kv[1]["n"])[:40]:
        calls = ", ".join(f"{c} {n}" for c, n in h["calls"].most_common(4))
        w(f"| {k} | {h['n']} | {h['unread_before']} | {calls} |")
    w("")
    with open(args.out, "w") as f:
        f.write("\n".join(L) + "\n")
    print(f"-> {args.out}", file=sys.stderr)
