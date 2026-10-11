#!/usr/bin/env python3
"""Build the synthetic Brill field PBN (`field.pbn`) that
`probes/tools/brill_report.py --self-test` runs on. Not a real Brill file:
we have none yet (the results mail goes to Rick).

    probes/fixtures/brill/make_fixture.py [--pbs ../Practice-Bidding-Scenarios]

Three eight-board "events" from Practice-Bidding-Scenarios deals, three
entrants each, every table a game of its own (South names the entrant):

- "BBA": the corpus auction (bba-cli, the scenario's cards);
- "Rusty": our engine for all four seats, 21GF-GIB (`rbb bid-pbn`);
- "Alt": our engine with 21GF-DEFAULT.

Results are the double-dummy tricks (`rbb dd`) moved a trick up or down on
about one table in four (a hash of the deal and entrant), so play gains
and losses exist. Two of Rusty's auctions are rewritten by hand to give
the misunderstanding flags something to find (marked with a comment in
the file): an opener passing partner's forcing new-suit response, and a North call our
engine has no rule for.

The events vary the format on purpose: event 1 carries [Scoring "MP"],
[Score "NS n"] and [ScorePercentage]; event 2 says IMP only in its name,
carries no score tags, and labels tables with [Room]/[Table]; event 3 has
[Scoring "MatchPoints"], [Score "EW n"] style scores, `#` repeats for the
deal, dealer and vulnerability of a board's later tables, a `{...}`
comment over two lines and one [Play] section.
"""

import argparse
import hashlib
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "probes", "tools"))
import brill_report as br  # noqa: E402

EVENTS = [
    ("Brill Daily 1 MP", "Basic_Major", "MP"),
    ("Brill Daily 2 IMPs", "Basic_Overcall", "IMP"),
    ("Brill Daily 3", "After_Opp_Overcalls", "MP"),
]
DATE = "2026.10.08"
BOARDS = 8


def sibling(name):
    """../<name> of the checkout, also from a worktree under .claude/."""
    d = ROOT
    while d != os.path.dirname(d):
        if os.path.isdir(os.path.join(os.path.dirname(d), name)):
            return os.path.join(os.path.dirname(d), name)
        d = os.path.dirname(d)
    return os.path.join(ROOT, "..", name)


def h(*xs):
    return int(hashlib.sha1("|".join(xs).encode()).hexdigest(), 16)


def bid(src, card, rbb):
    with tempfile.TemporaryDirectory() as d:
        out = os.path.join(d, "out.pbn")
        subprocess.run([rbb, "bid-pbn", "-i", src, "-o", out, "--ns-card", card],
                       check=True, capture_output=True, cwd=ROOT)
        return {g["tags"]["Board"]: g for g in br.parse_pbn(open(out).read())}


def auction_lines(calls):
    """PBN auction lines with =n= note references and the notes."""
    toks, notes = [], []
    for c, note in calls:
        toks.append(c.replace("NT", "N"))
        if note:
            notes.append(note)
            toks.append(f"={len(notes)}=")
    lines, row, n = [], [], 0
    for tok in toks:
        row.append(tok)
        if not tok.startswith("="):
            n += 1
            if n % 4 == 0:
                lines.append(" ".join(row))
                row = []
    if row:
        lines.append(" ".join(row))
    return lines, [f'[Note "{i + 1}:{t}"]' for i, t in enumerate(notes)]


def contract_tags(c):
    if c is None:
        return '[Declarer ""]\n[Contract "Pass"]\n[Result ""]\n'
    level, strain, dbl, decl = c
    s = "NT" if strain == "N" else strain
    return f'[Declarer "{decl}"]\n[Contract "{level}{s}{["", "X", "XX"][dbl]}"]\n'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pbs", default=sibling("Practice-Bidding-Scenarios"))
    ap.add_argument("--rbb", default=br.RBB)
    ap.add_argument("--out", default=os.path.join(HERE, "field.pbn"))
    args = ap.parse_args()
    out = ["% PBN 2.1",
           "% Synthetic Brill field PBN for brill_report.py --self-test (make_fixture.py).",
           "% Deals from Practice-Bidding-Scenarios; tables: BBA (corpus), Rusty and Alt (rbb).", ""]
    with tempfile.TemporaryDirectory() as tmp:
        for ei, (event, scen, scoring) in enumerate(EVENTS):
            games = br.parse_pbn(open(os.path.join(args.pbs, "bba", scen + ".pbn")).read())[:BOARDS]
            src = os.path.join(tmp, f"{scen}.pbn")
            with open(src, "w") as f:
                for i, g in enumerate(games):
                    t = g["tags"]
                    f.write(f'[Event "{event}"]\n[Board "{i + 1}"]\n[Dealer "{t["Dealer"]}"]\n'
                            f'[Vulnerable "{t["Vulnerable"]}"]\n[Deal "{t["Deal"]}"]\n'
                            f'[Scoring "{scoring}"]\n\n')
            rusty = bid(src, "21GF-GIB", args.rbb)
            alt = bid(src, "21GF-DEFAULT", args.rbb)
            dd = {r["deal"]: r for r in br.run_json([args.rbb, "dd", src, "--dd-cache",
                                                     os.path.join(ROOT, ".rbb-cache", "dd.jsonl")],
                                                    lines=True)}
            boards = []
            for i, g in enumerate(games):
                bno = str(i + 1)
                t = g["tags"]
                hands = br.deal_hands(t["Deal"])
                deal = br.deal_north(hands)
                tables = []
                for name, src_game, first in (
                        ("BBA", g, t.get("Auction", t["Dealer"])),
                        ("Rusty", rusty[bno], rusty[bno]["tags"].get("Auction", t["Dealer"])),
                        ("Alt", alt[bno], alt[bno]["tags"].get("Auction", t["Dealer"]))):
                    calls = br.parse_auction(src_game["auction"], src_game["notes"])
                    tables.append([name, calls, first, None])
                boards.append((bno, t, deal, tables))
            # Hand-made misunderstandings at Rusty's tables (event 1 and 2).
            if ei == 0:
                inject_forcing_passed(boards)
            if ei == 1:
                inject_no_rule(boards)
            # Scores.
            for bno, t, deal, tables in boards:
                for tb in tables:
                    calls, first = tb[1], tb[2]
                    c = br.final_contract(calls, first)
                    tricks = None
                    if c:
                        tricks = br.dd_of(dd[deal], c[3], c[1])
                        r = h(deal, tb[0]) % 8
                        tricks = max(0, min(13, tricks + (-1 if r == 0 else 1 if r == 1 else 0)))
                    tb[3] = (c, tricks, br.ns_score(c, tricks, t["Vulnerable"]) if c else 0)
                if scoring == "MP":
                    for tb in tables:
                        others = [o[3][2] for o in tables if o is not tb]
                        tb.append(br.mp_percent(tb[3][2], others))
            for bno, t, deal, tables in boards:
                for k, (name, calls, first, (c, tricks, ns), *mp) in enumerate(tables):
                    later = ei == 2 and k > 0
                    o = [f'[Event "{event}"]', '[Site "brill.aalborgdata.dk (synthetic)"]',
                         f'[Date "{DATE}"]', f'[Board "{bno}"]',
                         '[West "Brill"]', '[North "Brill"]', '[East "Brill"]', f'[South "{name}"]',
                         f'[Dealer "{"#" if later else t["Dealer"]}"]',
                         f'[Vulnerable "{"#" if later else t["Vulnerable"]}"]',
                         f'[Deal "{"#" if later else deal}"]']
                    if ei == 0:
                        o.append('[Scoring "MP"]')
                    if ei == 1:
                        o += [f'[Room "Open"]', f'[Table "{k + 1}"]']
                    if ei == 2:
                        o.append('[Scoring "MatchPoints"]')
                    o.append(contract_tags(c).rstrip("\n"))
                    if c:
                        o.append(f'[Result "{tricks}"]')
                    if ei == 0:
                        o.append(f'[Score "NS {ns}"]')
                        o.append(f'[ScorePercentage "{mp[0]:.2f}"]')
                    if ei == 2:
                        o.append(f'[Score "{"NS" if ns >= 0 else "EW"} {abs(ns)}"]')
                    if name == "Rusty" and getattr(calls, "comment", None):
                        o.append("{" + calls.comment + "}")
                    if ei == 2 and bno == "1" and name == "Rusty":
                        o.append("{A comment over two lines,\n as PBN allows.}")
                    lines, notes = auction_lines(calls)
                    o.append(f'[Auction "{first}"]')
                    o += lines
                    o += notes
                    if ei == 2 and bno == "1" and name == "BBA" and c:
                        o.append(f'[Play "{"NESW"[("NESW".index(c[3]) + 1) % 4]}"]')
                        o.append("- - - -")
                        o.append("*")
                    out += o
                    out.append("")
    with open(args.out, "w") as f:
        f.write("\n".join(out))
    print(f"wrote {args.out}", file=sys.stderr)


class Calls(list):
    comment = None


def inject_forcing_passed(boards):
    """Rusty's first auction where one of us opens one of a suit and
    partner responds in a new suit (forcing): the opener passes it."""
    for bno, t, deal, tables in boards:
        tb = tables[1]
        calls, first = tb[1], tb[2]
        seats = ["NESW"[("NESW".index(first) + i) % 4] for i in range(len(calls))]
        bids = [(i, c, s) for i, ((c, _), s) in enumerate(zip(calls, seats)) if c[0].isdigit()]
        if len(bids) < 2:
            continue
        (i0, c0, s0), (i1, c1, s1) = bids[0], bids[1]
        rank = "CDHS".find
        new_suit = c1[1:] != "NT" and (c1[0] == "1" and rank(c1[1:]) > rank(c0[1:])
                                       or c1[0] == "2" and rank(c1[1:]) < rank(c0[1:]))
        if s0 in "NS" and c0[0] == "1" and c0 != "1NT" and s1 == "NESW"[("NESW".index(s0) + 2) % 4] \
                and new_suit and i1 + 2 < len(calls):
            new = Calls(calls[:i1 + 1] + [("Pass", None)] * 3)
            new.comment = f"synthetic: the opener passes the forcing {c1}"
            tb[1] = new
            return
    raise SystemExit("no board for the forcing-pass injection")


def inject_no_rule(boards):
    """Rusty's first auction where North calls after South bid: North's
    call becomes 7C, which no rule reads."""
    for bno, t, deal, tables in boards:
        tb = tables[1]
        calls, first = tb[1], tb[2]
        seats = ["NESW"[("NESW".index(first) + i) % 4] for i in range(len(calls))]
        s_bid = [i for i, ((c, _), s) in enumerate(zip(calls, seats)) if s == "S" and c[0].isdigit()]
        n_after = [i for i, s in enumerate(seats) if s == "N" and s_bid and i > s_bid[0]]
        if n_after and calls[n_after[0]][0][0].isdigit() and int(calls[n_after[0]][0][0]) < 7:
            j = n_after[0]
            new = Calls(calls[:j] + [("7C", None)] + [("Pass", None)] * 3)
            new.comment = "synthetic: Brill (N) bids 7C, which no rule of ours reads"
            tb[1] = new
            return
    raise SystemExit("no board for the no-rule injection")


if __name__ == "__main__":
    main()
