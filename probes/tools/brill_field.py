#!/usr/bin/env python3
"""Every entrant in the Brill dailies, bidding and card play rated apart.

    probes/tools/brill_field.py [FIELD.pbn...] [--min-boards 200] [--since DAY] [--out report.md] [--json out.json]

With no files, reads every `.rbb-cache/brill/<day>/*-field.pbn`. Every entrant
sits South with the same three Brill robots, so the field is a controlled
experiment: what differs between tables is South.

**Bidding** scores each table's contract at its double-dummy result against
every other table's contract at its double-dummy result (MP% in MP events,
cross-IMPs a board in IMP events). **Play** is the rest: the actual score
against the field minus the bidding score.

**Tricks against the same contract** is the cleaner card-play measure: on
each board, a table's tricks minus the mean tricks of the other tables in
the same contract (strain, level, declarer), by role:

- `decl`: South declared (the entrant played both hands);
- `def`: East-West declared (the entrant defended with Brill North);
- `N decl`: North declared, Brill played both hands unless the entrant asked
  to declare for partner; a control, it should sit near zero for everyone.

It is in tricks for North-South (+ good for the entrant) and compares
contracts of equal difficulty, unlike tricks against double dummy, which
also reflects how hard the entrant's contracts were to make. Both are given.

± is one standard error of the mean over boards.
"""

import argparse
import glob
import json
import math
import os
import sys
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brill_report as br  # noqa: E402

ROOT = br.ROOT

# Entrants that play the cards with BEN: Thorvald's BEN entrant, BBO's Ben,
# the author's own account, and Rusty (bids with rbb, plays with our BEN).
BEN_FAMILY = {"BEN", "BBOBen", "lorserker", "Rusty"}


def mean_se(xs):
    n = len(xs)
    if n == 0:
        return None, None, 0
    m = sum(xs) / n
    if n < 2:
        return m, None, n
    v = sum((x - m) ** 2 for x in xs) / (n - 1)
    return m, math.sqrt(v / n), n


def fmt(ms, unit="", digits=1):
    m, se, n = ms
    if m is None:
        return "-"
    s = f"{m:+.{digits}f}{unit}" if unit != "%" else f"{m:.{digits}f}%"
    if se is not None:
        s += f" ± {se:.{digits}f}"
    return s


def boards_of(paths):
    games = []
    for p in paths:
        with open(p, encoding="utf-8", errors="replace") as f:
            games.extend(br.parse_pbn(f.read()))
    tables, bad = br.tables_of(games, "^$")
    return tables, bad


def analyse(paths, args):
    tables, bad = boards_of(paths)
    dd = br.dd_tables(tables, args.rbb, args.dd_cache)
    events = defaultdict(list)
    for t in tables:
        events[(t["event"], t["date"])].append(t)
    per = defaultdict(lambda: defaultdict(list))
    n_events = 0
    for (name, date), ts in events.items():
        scoring = br.event_scoring(ts, "MP")
        n_events += 1
        boards = defaultdict(list)
        for t in ts:
            boards[t["board"]].append(t)
        for bno, bt in boards.items():
            deal = Counter(t["deal"] for t in bt).most_common(1)[0][0]
            bt = [t for t in bt if t["deal"] == deal and t["deal"] in dd]
            if len(bt) < 3:
                continue
            for t in bt:
                c = t["contract"]
                t["dd_tricks"] = br.dd_of(dd[deal], c[3], c[1]) if c else None
                played = t["tricks"] if t["tricks"] is not None else t["dd_tricks"]
                t["ns"] = br.ns_score(c, played, t["vul"]) if c else 0
                t["ns_dd"] = br.ns_score(c, t["dd_tricks"], t["vul"]) if c else 0
            fn = br.mp_percent if scoring == "MP" else br.cross_imps
            same = defaultdict(list)
            for t in bt:
                c = t["contract"]
                if c and t["tricks"] is not None:
                    same[(c[0], c[1], c[3])].append(t)
            for t in bt:
                who = t["south"] or t["label"]
                others = [o for o in bt if o is not t]
                tot = fn(t["ns"], [o["ns"] for o in others])
                bid = fn(t["ns_dd"], [o["ns_dd"] for o in others])
                if tot is None or bid is None:
                    continue
                p = per[who]
                p[f"{scoring}_total"].append(tot)
                p[f"{scoring}_bid"].append(bid)
                p[f"{scoring}_play"].append(tot - bid)
                c = t["contract"]
                if not c or t["tricks"] is None:
                    continue
                role = {"S": "decl", "N": "ndecl"}.get(c[3], "def")
                sign = 1 if c[3] in "NS" else -1
                p[f"dd_{role}"].append(sign * (t["tricks"] - t["dd_tricks"]))
                peers = [o for o in same[(c[0], c[1], c[3])] if o is not t]
                if peers:
                    avg = sum(o["tricks"] for o in peers) / len(peers)
                    p[f"same_{role}"].append(sign * (t["tricks"] - avg))
    return per, n_events, bad


def render(per, n_events, args):
    rows = []
    for who, p in per.items():
        n = len(p["MP_total"]) + len(p["IMP_total"])
        if n >= args.min_boards:
            rows.append((who, p, n))
    rows.sort(key=lambda r: -(mean_se(r[1]["MP_bid"])[0] or 0))
    out = [f"# The Brill dailies field: bidding and card play by entrant",
           "",
           f"{n_events} events; entrants with at least {args.min_boards} boards. "
           "Bidding: each contract at double dummy against the field's at double dummy. "
           "Play: the actual score against the field minus that. "
           "Same contract: tricks for NS against the other tables in the same contract, "
           "by role (decl = the entrant declared, def = East-West declared, "
           "N decl = Brill North declared: a control). ± one standard error.",
           "",
           "## Scores",
           "",
           "| entrant | boards | MP total | MP bidding | MP play | IMP total | IMP bidding | IMP play |",
           "|---|---|---|---|---|---|---|---|"]
    for who, p, n in rows:
        out.append(f"| {who} | {n} | {fmt(mean_se(p['MP_total']), '%')} | "
                   f"{fmt(mean_se(p['MP_bid']), '%')} | {fmt(mean_se(p['MP_play']))} | "
                   f"{fmt(mean_se(p['IMP_total']), '', 2)} | {fmt(mean_se(p['IMP_bid']), '', 2)} | "
                   f"{fmt(mean_se(p['IMP_play']), '', 2)} |")
    out += ["", "## Card play in tricks",
            "",
            "| entrant | decl vs same contract | def vs same contract | N decl vs same (control) "
            "| decl vs DD | def vs DD | N decl vs DD |",
            "|---|---|---|---|---|---|---|"]
    rows.sort(key=lambda r: -(mean_se(r[1]["same_decl"])[0] or 0))
    for who, p, n in rows:
        cells = []
        for k in ("same_decl", "same_def", "same_ndecl", "dd_decl", "dd_def", "dd_ndecl"):
            m = mean_se(p[k])
            cells.append(fmt(m, "", 2) + f" ({m[2]})" if m[0] is not None else "-")
        mark = " (BEN)" if who in BEN_FAMILY else ""
        out.append(f"| {who}{mark} | " + " | ".join(cells) + " |")
    out += ["", "(n) is the number of boards in that role. Rusty plays with our own BEN "
            "on the droplet; BEN and BBOBen bid and play with BEN."]
    return "\n".join(out) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("pbn", nargs="*")
    ap.add_argument("--min-boards", type=int, default=200)
    ap.add_argument("--since", help="only days from this one on (YYYY-MM-DD), e.g. a deploy date")
    ap.add_argument("--out")
    ap.add_argument("--json")
    ap.add_argument("--rbb", default=br.RBB)
    ap.add_argument("--dd-cache", default=os.path.join(ROOT, ".rbb-cache", "dd.jsonl"))
    args = ap.parse_args()
    paths = args.pbn or sorted(glob.glob(os.path.join(ROOT, ".rbb-cache", "brill", "*", "*-field.pbn")))
    if args.since:
        paths = [p for p in paths if os.path.basename(os.path.dirname(p)) >= args.since]
    per, n_events, bad = analyse(paths, args)
    text = render(per, n_events, args)
    if args.out:
        with open(args.out, "w") as f:
            f.write(text)
        print(f"wrote {args.out}", file=sys.stderr)
    else:
        print(text)
    if args.json:
        with open(args.json, "w") as f:
            json.dump({k: dict(v) for k, v in per.items()}, f)
    if bad:
        print(f"{len(bad)} tables skipped (no contract)", file=sys.stderr)


if __name__ == "__main__":
    main()
