#!/usr/bin/env python3
"""Rusty in the Brill daily events: a per-board report from the field PBN,
and a running table across days.

    probes/tools/brill_report.py FIELD.pbn... [--out report.md]
        [--history .rbb-cache/brill/history.jsonl] [--us Rusty]
        [--card 21GF-GIB] [--scoring MP|IMP] [--worst 5] [--no-recheck]
    probes/tools/brill_report.py --history H --table-only   # the running table
    probes/tools/brill_report.py --self-test                 # on the fixture

Rusty sits South in Thorvald Aagaard's Brill dailies, with three Brill
robots (2/1 as GIB plays it on BBO), BEN playing our cards. Each morning
the results mail carries the whole field's PBN: every entrant's table on
the same eight boards (rusty-seat-service docs/seat-api.txt, section 0).
Run from the repository root, after `./dev-build.sh build --release -p
rbb-cli` (it calls `target/release/rbb dd`, `explain-auction` and `call`).

**Input.** Standard PBN, one game per table, so a board appears once for
every entrant. What is read (format assumptions to confirm on the first
real file are marked *):

- games are separated by blank lines, or a game starts when a tag repeats;
  `[Tag "#"]` repeats the previous game's value (PBN export format);
  `%` and `;` lines and `{...}` comments are skipped;
- the entrant of a table is its `[South]` player (*: robots sit South, and
  the field PBN names the entrant there, not "Brill"); ours is the one
  matching `--us` (a case-insensitive regular expression, whole name);
  `[Room]`/`[Table]` only label a table;
- an event is (`[Event]`, `[Date]`) (*: one event's eight boards share
  both); a board is its `[Board]` number within the event, and its tables
  must share `[Deal]` (a table with another deal is reported and dropped);
- scoring per event (*): `[Scoring]` (anything with IMP is cross-IMPs,
  else matchpoints), else an IMP/MP word in `[Event]`, else
  `[ScoreIMP]`/`[ScorePercentage]` tags, else `--scoring` (default MP);
- the auction from `[Auction "seat"]` with `=n=` note references,
  `[Note "n:text"]`, `AP`, `!`/`?` suffixes; `Pass/P`, `X/Dbl`, `XX/Rdbl`,
  `1N/1NT`;
- the contract from `[Contract]`+`[Declarer]` (else from the auction), the
  tricks from `[Result]` (declarer's tricks, *: not a +/- figure); a table
  with no result is scored double dummy and marked `dd`;
- `[Score]` ("NS 420", "EW 100", "420") is only checked against the
  computed score; `[ScorePercentage]`/`[ScoreIMP]` of our table are shown
  beside ours as Brill's own figures (*: our MP% and cross-IMPs are
  computed over the tables in the file, all-comparisons, IMPs averaged
  over the other tables; Brill's may differ: humans, adjustments).

**Per board**: our auction and Brill's alerts, contract, result, our score
against the field; **bidding vs play**: every table's contract scored at
its double-dummy result, ours against the field's that way (bidding), and
the rest of our score (play: ours and the field's declarer play and
defence), with our own tricks against double dummy and who played them
(BEN declares when South does; Brill when North does, unless Brill turns
on `declaresForPartner`; on defence BEN plays South, Brill North); our
contract against par (IMPs) and the errors yardstick (`par::table_errors`,
contract and doubling errors, each side charged with its own; Rick,
2026-10-01). **Partnership misunderstandings**: the auction replayed
through `rbb explain-auction -c <card> --deal`, flagging a call our engine
reads with no rule; a call whose reading the caller's hand contradicts
(partner's first, the most telling: we misread Brill's system); Brill
passing a call of ours our engine marks forcing, or our side stopping
below game after a game force; Brill bidding a suit of ours that we bid
artificially; a suit contract in a 6-card or shorter fit. With
`--recheck` (default) every South call is asked of today's engine (`rbb
call`), so a board the rules have since changed says so. On the worst
boards, the other entrants' auctions beside ours.

**History** (`--history`): one JSON line per (date, event), replaced when
the same event is reported again; the running table is per day and
cumulative: boards, MP% (MP events), IMPs a board (IMP events), the
bidding and play split, misunderstandings. Eight boards are very noisy (a
board's MP% has a spread of about 30 points, so a day's four events
carry a standard error of some 5 points): the table prints it.

The DD tables come from `rbb dd` (bridge-solver, cached in
`.rbb-cache/dd.jsonl` with `compare`'s); par as `compare` computes it.
"""

import argparse
import json
import math
import os
import re
import shlex
import subprocess
import sys
import tempfile
from collections import Counter, defaultdict

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
RBB = os.path.join(ROOT, "target", "release", "rbb")
FIXTURE = os.path.join(ROOT, "probes", "fixtures", "brill", "field.pbn")

SEATS = "NESW"
STRAINS = "CDHSN"  # rank order; N = notrump
SUIT_SYMBOL = {"C": "♣", "D": "♦", "H": "♥", "S": "♠", "N": "NT"}
NOISE = ("Eight-board events are very noisy: one board's MP% spreads about "
         "30 points, so an event's average carries some 10 points of luck and "
         "a day's four events about 5. Read the trend over weeks, not a day.")


# ---------------------------------------------------------------- scoring

def imps(diff):
    steps = [20, 50, 90, 130, 170, 220, 270, 320, 370, 430, 500, 600, 750, 900,
             1100, 1300, 1500, 1750, 2000, 2250, 2500, 3000, 3500, 4000]
    n = sum(1 for s in steps if abs(diff) >= s)
    return n if diff >= 0 else -n


def contract_score(level, strain, dbl, tricks, vul):
    """Declarer's score: dbl 0, 1 (X) or 2 (XX); vul: declarer vulnerable."""
    over = tricks - (level + 6)
    if over < 0:
        down = -over
        if dbl == 0:
            return -down * (100 if vul else 50)
        if vul:
            pen = 200 + 300 * (down - 1)
        else:
            pen = [100, 300, 500][min(down, 3) - 1] + 300 * max(0, down - 3)
        return -pen * dbl
    per = 20 if strain in "CD" else 30
    base = per * level + (10 if strain == "N" else 0)
    base *= (1, 2, 4)[dbl]
    s = base
    s += (500 if vul else 300) if base >= 100 else 50
    if level == 6:
        s += 750 if vul else 500
    elif level == 7:
        s += 1500 if vul else 1000
    if dbl:
        s += 50 * dbl
        s += over * (200 if vul else 100) * dbl
    else:
        s += over * per
    return s


def vulnerable(vul, seat):
    v = (vul or "").upper().replace("-", "")
    if v in ("ALL", "BOTH", "B"):
        return True
    if v == "NS":
        return seat in "NS"
    if v == "EW":
        return seat in "EW"
    return False


def ns_score(contract, tricks, vul):
    """North-South's score; contract (level, strain, dbl, declarer) or None."""
    if contract is None:
        return 0
    level, strain, dbl, decl = contract
    s = contract_score(level, strain, dbl, tricks, vulnerable(vul, decl))
    return s if decl in "NS" else -s


def rank(level, strain):
    return (level - 1) * 5 + STRAINS.index(strain) + 1


def table_errors(contract, dd_tricks, vul, par_ns, par_rank):
    """A port of `par::table_errors` (crates/compare/src/par.rs): contract
    and doubling errors in IMPs, [NS, EW], each side charged with its own.
    `dd_tricks` is declarer's double-dummy tricks in the contract."""
    actual = ns_score(contract, dd_tricks, vul)
    assumed = actual
    if contract is not None:
        level, strain, dbl, decl = contract
        made = dd_tricks >= level + 6
        mine = (lambda x: x) if decl in "NS" else (lambda x: -x)
        as_ = lambda d: ns_score((level, strain, d, decl), dd_tricks, vul)  # noqa: E731
        if dbl and made:
            assumed = as_(0)
        elif dbl == 2 and not made:
            assumed = as_(1)
        elif dbl == 0 and not made and (rank(level, strain) > par_rank
                                        or mine(actual) > mine(par_ns)):
            assumed = as_(1)
    e = {"contract": [0, 0], "doubling": [0, 0]}

    def charge(slot, result, bench):
        e[slot][0 if result < bench else 1] += imps(abs(result - bench))
    charge("contract", assumed, par_ns)
    charge("doubling", actual, assumed)
    return e


def mp_percent(mine, others):
    if not others:
        return None
    pts = sum(2 if mine > o else 1 if mine == o else 0 for o in others)
    return 100.0 * pts / (2 * len(others))


def cross_imps(mine, others):
    if not others:
        return None
    return sum(imps(mine - o) for o in others) / len(others)


# ------------------------------------------------------------------- PBN

TAG = re.compile(r'^\[(\w+)\s+"(.*)"\]$')
DATA_SECTIONS = ("Auction", "Play")


def parse_pbn(text):
    """Games as dicts: tags, auction tokens, notes (auction notes only),
    play lines. Robust to the variations listed in the docstring."""
    games, cur, section, prev = [], None, None, {}
    in_comment = False

    def close():
        nonlocal cur, prev
        if cur and cur["tags"]:
            games.append(cur)
            prev = dict(cur["tags"])
        cur = None

    for raw in text.splitlines():
        line = raw.strip()
        if in_comment:
            if "}" not in line:
                continue
            line = line[line.index("}") + 1:].strip()
            in_comment = False
            if not line:
                continue
        if line.startswith("%") or line.startswith(";"):
            continue
        if not line:
            close()
            section = None
            continue
        line = re.sub(r"\{[^}]*\}", "", line).strip()
        if "{" in line:
            line, in_comment = line[:line.index("{")].strip(), True
        if not line:
            continue
        m = TAG.match(line)
        if m:
            tag, val = m.groups()
            if tag == "Note":
                if cur is not None and section == "Auction":
                    cur["notes"].append(val)
                continue
            if cur is not None and tag in cur["tags"]:
                close()
            if cur is None:
                cur = {"tags": {}, "auction": [], "notes": [], "play": []}
            if val == "#":
                val = prev.get(tag, "")
            cur["tags"][tag] = val
            if tag in DATA_SECTIONS:
                section = tag
            elif section == "Auction" and tag != "Note":
                section = None if tag not in DATA_SECTIONS else tag
            else:
                section = None
            continue
        if cur is not None and section == "Auction":
            cur["auction"].extend(line.split())
        elif cur is not None and section == "Play":
            cur["play"].append(line)
    close()
    return games


def norm_call(tok):
    t = tok.strip().rstrip("!?").upper()
    if t in ("P", "PASS", "-"):
        return "Pass"
    if t in ("X", "D", "DBL", "DOUBLE"):
        return "X"
    if t in ("XX", "R", "RDBL", "REDOUBLE"):
        return "XX"
    m = re.fullmatch(r"([1-7])(NT|N|S|H|D|C)", t)
    if m:
        return m[1] + ("NT" if m[2] in ("N", "NT") else m[2])
    return None


def auction_complete(calls):
    if len(calls) >= 4 and all(c == "Pass" for c in calls[:4]):
        return True
    if any(c not in ("Pass", "X", "XX") for c in calls):
        return len(calls) >= 4 and calls[-3:] == ["Pass"] * 3
    return False


def parse_auction(tokens, notes):
    """[(call, note text or None)]; AP expands to the passes that end it."""
    note_text = {}
    for n in notes:
        k, _, v = n.partition(":")
        note_text[k.strip()] = v.strip()
    calls = []
    for tok in tokens:
        if tok.startswith("=") and tok.endswith("=") and len(tok) > 2:
            if calls:
                calls[-1] = (calls[-1][0], note_text.get(tok.strip("="), "?"))
            continue
        if tok.upper() == "AP":
            names = [c for c, _ in calls]
            while not auction_complete(names):
                calls.append(("Pass", None))
                names.append("Pass")
            continue
        if tok.startswith("$") or tok in ("*", "+", "-"):
            continue
        c = norm_call(tok)
        if c:
            calls.append((c, None))
    return calls


def final_contract(calls, first):
    """(level, strain, dbl, declarer) or None from the calls, first caller
    `first`; strain letter as in STRAINS."""
    last, dbl, who = None, 0, None
    seats = [SEATS[(SEATS.index(first) + i) % 4] for i in range(len(calls))]
    for (c, _), s in zip(calls, seats):
        if c == "X":
            dbl = 1
        elif c == "XX":
            dbl = 2
        elif c != "Pass":
            last, dbl, who = c, 0, s
    if last is None:
        return None
    level, strain = int(last[0]), last[1:][:1]
    strain = "N" if last.endswith("NT") else strain
    side = "NS" if who in "NS" else "EW"
    for (c, _), s in zip(calls, seats):
        if c[0].isdigit() and (c.endswith("NT") and strain == "N" or c[1:] == strain) and s in side:
            return (level, strain, dbl, s)
    return (level, strain, dbl, who)


def parse_contract(text, decl):
    t = (text or "").strip().upper()
    if not t or t in ("PASS", "AP", "-"):
        return None if t in ("PASS", "AP") else "?"
    m = re.fullmatch(r"([1-7])(NT|N|S|H|D|C)(XX|X)?", t)
    if not m or (decl or "").upper()[:1] not in SEATS:
        return "?"
    return (int(m[1]), "N" if m[2] in ("N", "NT") else m[2],
            {None: 0, "X": 1, "XX": 2}[m[3]], decl.upper()[:1])


def deal_hands(deal):
    """{seat: 'S.H.D.C'} from a PBN deal string."""
    first, _, rest = deal.strip().partition(":")
    hands = rest.split()
    if len(hands) != 4 or first.upper() not in SEATS:
        return None
    i = SEATS.index(first.upper())
    return {SEATS[(i + k) % 4]: hands[k] for k in range(4)}


def deal_north(hands):
    return "N:" + " ".join(hands[s] for s in SEATS)


def parse_score(text):
    t = (text or "").strip().upper()
    m = re.fullmatch(r"(NS|EW)?\s*(-?\d+)", t)
    if not m:
        return None
    v = int(m[2])
    return -v if m[1] == "EW" else v


def tables_of(games, us):
    """Normalised tables; the ones whose South matches `us` are ours."""
    out, bad = [], []
    usre = re.compile(us, re.I)
    for g in games:
        t = g["tags"]
        hands = deal_hands(t.get("Deal", ""))
        if not hands:
            continue
        first = (t.get("Auction") or t.get("Dealer") or "N").upper()[:1]
        calls = parse_auction(g["auction"], g["notes"])
        contract = parse_contract(t.get("Contract"), t.get("Declarer"))
        if contract == "?":
            contract = final_contract(calls, first) if calls else "?"
        if contract == "?":
            bad.append(f"board {t.get('Board')}: no contract for {t.get('South')}")
            continue
        res = t.get("Result", "").strip()
        tricks = int(res) if res.isdigit() and int(res) <= 13 else None
        south = t.get("South", "").strip()
        out.append({
            "event": t.get("Event", "").strip() or "?",
            "date": t.get("Date", "").strip(),
            "board": t.get("Board", "").strip(),
            "dealer": (t.get("Dealer") or first).upper()[:1],
            "first": first,
            "vul": t.get("Vulnerable", "None"),
            "deal": deal_north(hands),
            "hands": hands,
            "south": south,
            "north": t.get("North", "").strip(),
            "label": south + "".join(f" {t[k]}" for k in ("Room", "Table") if t.get(k)),
            "ours": bool(usre.fullmatch(south)),
            "calls": calls,
            "contract": contract,
            "tricks": tricks,
            "score_tag": parse_score(t.get("Score")),
            "scoring_tag": t.get("Scoring", ""),
            "official": {k: t[k] for k in ("ScorePercentage", "ScoreIMP", "ScoreMP")
                         if t.get(k) and t[k] != "-"},
            "play": g["play"],
        })
    return out, bad


def event_scoring(tables, default):
    for t in tables:
        s = t["scoring_tag"].upper()
        if s:
            return "IMP" if "IMP" in s else "MP"
    name = tables[0]["event"].upper()
    if re.search(r"\bIMPS?\b|CROSS.?IMP|BUTLER", name):
        return "IMP"
    if re.search(r"\bMPS?\b|MATCH.?POINT|PAIRS", name):
        return "MP"
    for t in tables:
        if "ScoreIMP" in t["official"]:
            return "IMP"
        if "ScorePercentage" in t["official"] or "ScoreMP" in t["official"]:
            return "MP"
    return default


# ---------------------------------------------------------------- engine

def run_json(args, lines=False):
    p = subprocess.run(args, capture_output=True, text=True, cwd=ROOT)
    if p.returncode != 0:
        raise RuntimeError(f"{shlex.join(args)}: {p.stderr.strip()[:400]}")
    if lines:
        return [json.loads(x) for x in p.stdout.splitlines() if x.strip()]
    return json.loads(p.stdout)


def dd_tables(tables, rbb, cache):
    pairs = {(t["deal"], t["vul"]) for t in tables}
    with tempfile.TemporaryDirectory() as d:
        src = os.path.join(d, "deals.pbn")
        with open(src, "w") as f:
            for i, (deal, vul) in enumerate(sorted(pairs)):
                f.write(f'[Board "{i + 1}"]\n[Vulnerable "{vul}"]\n[Deal "{deal}"]\n\n')
        rows = run_json([rbb, "dd", src, "--dd-cache", cache], lines=True)
    out = {}
    for r in rows:
        out[r["deal"]] = r
    return out


def dd_of(row, decl, strain):
    return row["tricks"][decl]["CDHSN".index(strain)]


def call_name(c):
    """explain-auction / call JSON call to our text form."""
    if isinstance(c, str):
        return norm_call(c) or c
    if c in ("Pass", "Double", "Redouble"):
        return {"Pass": "Pass", "Double": "X", "Redouble": "XX"}[c]
    if isinstance(c, dict) and "Bid" in c:
        b = c["Bid"]
        st = {"Clubs": "C", "Diamonds": "D", "Hearts": "H", "Spades": "S",
              "NoTrump": "NT"}[b["strain"]]
        return f"{b['level']}{st}"
    return str(c)


def card_path(card):
    p = os.path.join(ROOT, "cards", "bbsa", card + ".bbsa")
    return p if os.path.exists(p) else card


def hand_shape(hand):
    """(hcp, lengths in C D H S order) of an S.H.D.C hand."""
    suits = hand.split(".")
    hcp = sum({"A": 4, "K": 3, "Q": 2, "J": 1}.get(ch, 0) for ch in hand.upper())
    return hcp, [len(suits[3]), len(suits[2]), len(suits[1]), len(suits[0])]


def contradiction(know, hand):
    """What the caller's hand contradicts in our reading of it, or None."""
    hcp, lens = hand_shape(hand)
    bad = []
    h = know.get("hcp") or {}
    if hcp < h.get("lo", 0) or hcp > h.get("hi", 40):
        bad.append(f"{hcp} HCP, read as {h.get('lo')}-{h.get('hi')}")
    for i, s in enumerate("CDHS"):
        r = know["len"][i]
        if lens[i] < r["lo"] or lens[i] > r["hi"]:
            rng = f"{r['lo']}+" if r["hi"] >= 13 else f"{r['lo']}-{r['hi']}"
            bad.append(f"{lens[i]} {SUIT_SYMBOL[s]}, read as {rng}")
    return "; ".join(bad) or None


GAME = {"C": 5, "D": 5, "H": 4, "S": 4, "N": 3}


def misunderstandings(t, rows, card, recheck, rbb, scoring):
    """Flags for our table from the explain-auction rows."""
    flags = []
    seat_of = lambda i: SEATS[(SEATS.index(t["first"]) + i) % 4]  # noqa: E731
    who = {"N": "partner (Brill N)", "S": "we (S)", "E": "opponent E", "W": "opponent W"}
    contradicted = set()
    calls = [c for c, _ in t["calls"]]
    ns_gf = False
    artificial = {}  # strain -> our artificial call index
    for i, r in enumerate(rows):
        s = r["step"]
        seat = seat_of(i)
        call = call_name(s["call"])
        if s["rule"] is None:
            flags.append(("no-rule", seat, i, f"{who[seat]} {call}: our engine has no rule for it"))
        elif seat not in contradicted:
            sidx = SEATS.index(seat)
            c = contradiction(r["seats"][sidx], t["hands"][seat])
            if c:
                contradicted.add(seat)
                kind = {"N": "partner-misread", "S": "own-misdescribed"}.get(seat, "opponent-misread")
                flags.append((kind, seat, i, f"{who[seat]} {call} read as \"{s['explanation']}\": "
                              f"the hand has {c}"))
        f = r["flags"]
        if seat == "S" and (f.get("forcing") or f.get("game_force")):
            if i + 2 < len(calls) and calls[i + 1] == "Pass" and calls[i + 2] == "Pass":
                flags.append(("forcing-passed", "N", i + 2,
                              f"Brill passed our forcing {call} (\"{s['explanation']}\")"))
        if seat == "N" and (f.get("forcing") or f.get("game_force")):
            if i + 2 < len(calls) and calls[i + 1] == "Pass" and calls[i + 2] == "Pass":
                flags.append(("forcing-passed", "S", i + 2,
                              f"we passed partner's {call}, which our engine reads as forcing"))
        if r["sides"][0].get("forcing") == "game":
            ns_gf = True
        if seat == "S" and call[0].isdigit() and not call.endswith("NT") and (
                s.get("artificial") or f.get("artificial")):
            artificial[call[-1]] = i
        if seat == "N" and call[0].isdigit() and call[-1] in artificial and not call.endswith("NT"):
            st = call[-1]
            if i > 0 and rows[i - 1]["seats"][2]["len"]["CDHS".index(st)]["lo"] < 3:
                flags.append(("raised-artificial", "N", i,
                              f"Brill bid {call} in a suit our {calls[artificial[st]]} showed "
                              f"artificially"))
    c = t["contract"]
    if c and c[3] in "NS":
        if ns_gf and c[0] < GAME[c[1]] and not c[2]:
            flags.append(("gf-dropped", c[3], len(calls) - 1,
                          f"stopped in {c[0]}{SUIT_SYMBOL[c[1]]} after a game force"))
        if c[1] != "N":
            _, ln = hand_shape(t["hands"]["N"])
            _, ls = hand_shape(t["hands"]["S"])
            k = "CDHS".index(c[1])
            if ln[k] + ls[k] <= 6:
                flags.append(("misfit", c[3], len(calls) - 1,
                              f"played in a {ln[k]}-{ls[k]} {SUIT_SYMBOL[c[1]]} fit"))
    now = []
    if recheck:
        for i in range(len(calls)):
            if seat_of(i) != "S":
                continue
            args = [rbb, "call", t["hands"]["S"], "-a", " ".join(calls[:i]), "-d", t["first"],
                    "-v", vul_arg(t["vul"]), "-s", scoring, "-c", card_path(card),
                    "--rules", os.path.join(ROOT, "conventions"), "--json"]
            try:
                d = run_json(args)
            except RuntimeError as e:
                now.append((i, calls[i], f"error: {e}"))
                continue
            got = call_name(d["call"])
            if got != calls[i]:
                now.append((i, calls[i], f"{got} ({d.get('explanation') or 'no rule'})"))
    return flags, now


def vul_arg(v):
    u = (v or "").upper().replace("-", "")
    return {"ALL": "All", "BOTH": "All", "NS": "NS", "EW": "EW"}.get(u, "None")


def explain(t, card, rbb, scoring):
    calls = " ".join(c for c, _ in t["calls"])
    args = [rbb, "explain-auction", "-a", calls, "-d", t["first"], "-v", vul_arg(t["vul"]),
            "-s", scoring, "-c", card, "--deal", t["deal"],
            "--rules", os.path.join(ROOT, "conventions")]
    return args


# ---------------------------------------------------------------- report

def fmt_contract(c, tricks=None):
    if c is None:
        return "passed out"
    level, strain, dbl, decl = c
    s = f"{level}{SUIT_SYMBOL[strain]}{['', 'X', 'XX'][dbl]} {decl}"
    if tricks is not None:
        d = tricks - (level + 6)
        s += " =" if d == 0 else f" {d:+d}"
    return s


def fmt_auction(t):
    """W N E S table rows, from the dealer, with Brill's alerts as notes."""
    cols = "WNES"
    out = ["| W | N | E | S |", "|---|---|---|---|"]
    row = [""] * 4
    pos = cols.index(t["first"])
    notes = []
    for c, note in t["calls"]:
        txt = c
        if note:
            notes.append(note)
            txt += f" ({len(notes)})"
        row[pos] = txt
        pos += 1
        if pos == 4:
            out.append("| " + " | ".join(row) + " |")
            row, pos = [""] * 4, 0
    if any(row):
        out.append("| " + " | ".join(row) + " |")
    lines = "\n".join(out)
    if notes:
        lines += "\n\n" + "; ".join(f"({i + 1}) {n}" for i, n in enumerate(notes))
    return lines


def fmt_inline(t):
    return " ".join(c for c, _ in t["calls"]) or "-"


def fmt_unit(v, scoring):
    if v is None:
        return "-"
    return f"{v:.0f}%" if scoring == "MP" else f"{v:+.1f}"


def fmt_delta(v, scoring):
    if v is None:
        return "-"
    return f"{v:+.0f}%" if scoring == "MP" else f"{v:+.1f}"


def analyse(paths, args):
    games = []
    for p in paths:
        with open(p, encoding="utf-8", errors="replace") as f:
            games.extend(parse_pbn(f.read()))
    tables, bad = tables_of(games, args.us)
    if not tables:
        raise SystemExit("no tables with a deal in the input")
    dd = dd_tables(tables, args.rbb, args.dd_cache)
    events = defaultdict(list)
    for t in tables:
        events[(t["event"], t["date"])].append(t)
    report = {"events": [], "warnings": bad}
    for (name, date), ts in events.items():
        scoring = event_scoring(ts, args.scoring)
        boards = defaultdict(list)
        for t in ts:
            boards[t["board"]].append(t)
        ev = {"event": name, "date": date, "scoring": scoring, "boards": [], "entrants": {}}
        totals = defaultdict(list)
        for bno in sorted(boards, key=lambda b: (len(b), b)):
            bt = boards[bno]
            ours = [t for t in bt if t["ours"]]
            deal = Counter(t["deal"] for t in (ours or bt)).most_common(1)[0][0]
            dropped = [t for t in bt if t["deal"] != deal]
            for t in dropped:
                report["warnings"].append(f"{name} board {bno}: {t['label']} has another deal; dropped")
            bt = [t for t in bt if t["deal"] == deal]
            row = dd[deal] if deal in dd else None
            for t in bt:
                c = t["contract"]
                t["dd_tricks"] = dd_of(dd[t["deal"]], c[3], c[1]) if c else None
                played = t["tricks"] if t["tricks"] is not None else t["dd_tricks"]
                t["ns"] = ns_score(c, played, t["vul"]) if c else 0
                t["ns_dd"] = ns_score(c, t["dd_tricks"], t["vul"]) if c else 0
                if t["score_tag"] is not None and t["tricks"] is not None and t["score_tag"] != t["ns"]:
                    report["warnings"].append(
                        f"{name} board {bno} {t['label']}: [Score] {t['score_tag']} but "
                        f"{fmt_contract(c, t['tricks'])} scores {t['ns']}")
            fn = mp_percent if scoring == "MP" else cross_imps
            for t in bt:
                others = [o for o in bt if o is not t]
                t["field"] = fn(t["ns"], [o["ns"] for o in others])
                t["field_dd"] = fn(t["ns_dd"], [o["ns_dd"] for o in others])
                if t["field"] is not None:
                    totals[t["south"] or t["label"]].append(t["field"])
            if not ours:
                continue
            if len(ours) > 1:
                report["warnings"].append(f"{name} board {bno}: {len(ours)} tables match --us; "
                                          "the first is ours")
            t = ours[0]
            b = {"board": bno, "us": t, "field": [o for o in bt if o is not t], "dd": row}
            if t["field"] is not None:
                b["total"] = t["field"]
                b["bidding"] = t["field_dd"]
                b["play"] = t["field"] - t["field_dd"]
            c = t["contract"]
            if c and t["tricks"] is not None:
                d = t["tricks"] - t["dd_tricks"]
                b["trick_gain"] = d if c[3] in "NS" else -d
                b["player"] = {"S": "BEN declared", "N": "Brill (N) declared"}.get(
                    c[3], "defence: BEN S, Brill N")
            b["par_ns"] = row["par_ns"]
            b["par"] = row["par"]
            b["vs_par"] = imps(t["ns_dd"] - row["par_ns"])
            b["errors"] = table_errors(c, t["dd_tricks"] or 0, t["vul"], row["par_ns"],
                                       row["par_rank"])
            try:
                rows = run_json(explain(t, args.card, args.rbb, scoring) + ["--json"])
                b["flags"], b["now"] = misunderstandings(t, rows, args.card, args.recheck,
                                                         args.rbb, scoring)
            except RuntimeError as e:
                b["flags"], b["now"] = [("error", "-", 0, str(e))], []
            b["explain"] = shlex.join(["target/release/rbb"] + explain(t, args.card, "x", scoring)[1:-2])
            ev["boards"].append(b)
        places = sorted(((sum(v) / len(v) if scoring == "MP" else sum(v), k)
                         for k, v in totals.items()), reverse=True)
        ev["standings"] = places
        ours_label = next((b["us"]["south"] or b["us"]["label"] for b in ev["boards"]), None)
        ev["place"] = next((i + 1 for i, (_, k) in enumerate(places) if k == ours_label), None)
        ev["n_entrants"] = len(places)
        if ev["boards"]:
            report["events"].append(ev)
    return report


def summary_row(ev):
    bs = [b for b in ev["boards"] if "total" in b]
    n = len(bs)
    agg = (lambda xs: sum(xs) / n) if ev["scoring"] == "MP" else (lambda xs: sum(xs))
    flags = [f for b in ev["boards"] for f in b["flags"]]
    mis = [f for f in flags if f[0] not in ("no-rule", "opponent-misread", "error")]
    return {
        "date": ev["date"], "event": ev["event"], "scoring": ev["scoring"], "boards": n,
        "place": ev["place"], "entrants": ev["n_entrants"],
        "score": agg([b["total"] for b in bs]) if n else None,
        "bidding": agg([b["bidding"] for b in bs]) if n else None,
        "play": agg([b["play"] for b in bs]) if n else None,
        "board_scores": [b["total"] for b in bs],
        "vs_par": sum(b["vs_par"] for b in ev["boards"]),
        "trick_gain": {k: sum(b.get("trick_gain", 0) for b in ev["boards"] if b.get("player", "").startswith(p))
                       for k, p in (("ben", "BEN"), ("brill", "Brill"), ("defence", "defence"))},
        "misunderstandings": len(mis),
        "no_rule": sum(1 for f in flags if f[0] == "no-rule"),
    }


def render(report, args, history_rows):
    out = []
    dates = sorted({e["date"] for e in report["events"]})
    out.append(f"# Rusty in the Brill dailies: {', '.join(d or '?' for d in dates)}\n")
    out.append(f"> {NOISE}\n")
    out.append("MP events: score, bidding and play are MP% (bidding: every table's contract "
               "at its double-dummy result; play: the rest, ours and the field's). IMP events: "
               "cross-IMPs averaged over the other tables, summed over the boards. "
               "Tricks: ours against double dummy (+ good for us).\n")
    out.append("## Summary\n")
    out.append("| event | scoring | boards | place | score | bidding | play | vs par (IMPs) "
               "| tricks BEN / Brill / defence | misunderstandings | no rule |")
    out.append("|---|---|---|---|---|---|---|---|---|---|---|")
    for ev in report["events"]:
        r = summary_row(ev)
        sc = ev["scoring"]
        tg = r["trick_gain"]
        out.append(f"| {ev['event']} {ev['date']} | {sc} | {r['boards']} | "
                   f"{r['place']}/{r['entrants']} | {fmt_unit(r['score'], sc)} | "
                   f"{fmt_delta(r['bidding'] - (50 if sc == 'MP' else 0), sc) if r['bidding'] is not None else '-'} | "
                   f"{fmt_delta(r['play'], sc)} | {r['vs_par']:+d} | "
                   f"{tg['ben']:+d} / {tg['brill']:+d} / {tg['defence']:+d} | "
                   f"{r['misunderstandings']} | {r['no_rule']} |")
    out.append("\nBidding for MP is shown against an average 50%; play is the MP% the cards "
               "added to (or took from) it.\n")
    # Worst boards.
    allb = [(ev, b) for ev in report["events"] for b in ev["boards"] if "total" in b]

    def badness(eb):
        ev, b = eb
        return (b["total"] - 50) / 50 * 6 if ev["scoring"] == "MP" else b["total"]
    worst = sorted(allb, key=badness)[:args.worst]
    out.append(f"## Worst boards\n")
    for ev, b in worst:
        t = b["us"]
        out.append(f"- **{ev['event']} board {b['board']}**: {fmt_unit(b['total'], ev['scoring'])} "
                   f"(bidding {fmt_unit(b['bidding'], ev['scoring'])}), "
                   f"{fmt_contract(t['contract'], t['tricks'])}, {len(b['flags'])} flag(s)\n"
                   f"  `{b['explain']}`")
    if report["warnings"]:
        out.append("\n## Input warnings\n")
        out.extend(f"- {w}" for w in report["warnings"])
    worst_ids = {(id(ev), b["board"]) for ev, b in worst}
    for ev in report["events"]:
        sc = ev["scoring"]
        out.append(f"\n## {ev['event']} {ev['date']} ({sc})\n")
        out.append("Standings (computed from the file): " + ", ".join(
            f"{i + 1}. {k} {fmt_unit(v, sc)}" for i, (v, k) in enumerate(ev["standings"])) + "\n")
        for b in ev["boards"]:
            t = b["us"]
            c = t["contract"]
            out.append(f"### Board {b['board']}: {fmt_unit(b.get('total'), sc)}"
                       f" (bidding {fmt_unit(b.get('bidding'), sc)}, play {fmt_delta(b.get('play'), sc)})\n")
            out.append(f"Dealer {t['dealer']}, vul {t['vul']}. "
                       + " ".join(f"{s}: {t['hands'][s]}" for s in SEATS) + "\n")
            out.append(fmt_auction(t) + "\n")
            res = "dd" if t["tricks"] is None else t["tricks"]
            line = (f"**{fmt_contract(c, t['tricks'] if t['tricks'] is not None else t['dd_tricks'])}**"
                    f" ({res} tricks, DD {t['dd_tricks'] if c else '-'}), NS {t['ns']:+d}")
            if "trick_gain" in b:
                line += f"; {b['player']}: {b['trick_gain']:+d} trick(s) against double dummy"
            if t["official"]:
                line += "; Brill: " + ", ".join(f"{k} {v}" for k, v in t["official"].items())
            out.append(line + ".  ")
            e = b["errors"]
            out.append(f"Par NS {b['par_ns']:+d} ({b['par'] or 'pass'}); our contract at DD "
                       f"{b['vs_par']:+d} IMPs from par; errors NS/EW: contract "
                       f"{e['contract'][0]}/{e['contract'][1]}, doubling "
                       f"{e['doubling'][0]}/{e['doubling'][1]}.  ")
            fc = Counter(fmt_contract(o["contract"]) for o in b["field"])
            out.append("Field: " + ", ".join(f"{k} ×{v}" if v > 1 else k for k, v in fc.most_common())
                       + "\n")
            for kind, seat, i, text in b["flags"]:
                out.append(f"- `{kind}` (call {i + 1}): {text}")
            for i, was, now in b["now"]:
                out.append(f"- `engine-now` (call {i + 1}): today's engine bids {now} instead of {was}")
            if b["flags"] or b["now"]:
                out.append("")
            out.append(f"`{b['explain']}`\n")
            if (id(ev), b["board"]) in worst_ids:
                out.append("Every table:\n")
                out.append(f"| entrant | auction | contract | NS | {sc} |")
                out.append("|---|---|---|---|---|")
                for o in sorted([t] + b["field"], key=lambda o: -(o["field"] or 0)):
                    tr = o["tricks"] if o["tricks"] is not None else o["dd_tricks"]
                    name = f"**{o['label']}**" if o is t else o["label"]
                    out.append(f"| {name} | {fmt_inline(o)} | {fmt_contract(o['contract'], tr)} | "
                               f"{o['ns']:+d} | {fmt_unit(o['field'], sc)} |")
                out.append("")
    if history_rows:
        out.append(render_history(history_rows))
    return "\n".join(out) + "\n"


# --------------------------------------------------------------- history

def update_history(path, report):
    rows = load_history(path)
    keep = {(r["date"], r["event"]): r for r in rows}
    for ev in report["events"]:
        r = summary_row(ev)
        keep[(r["date"], r["event"])] = r
    rows = sorted(keep.values(), key=lambda r: (r["date"], r["event"]))
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "w") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    return rows


def load_history(path):
    if not path or not os.path.exists(path):
        return []
    with open(path) as f:
        return [json.loads(x) for x in f if x.strip()]


def render_history(rows):
    out = ["\n## Running table\n", f"> {NOISE}\n",
           "| day | boards | MP% (MP boards) | IMPs/board (IMP boards) | bidding MP% / IMPs | "
           "play MP% / IMPs | misunderstandings/board | places |",
           "|---|---|---|---|---|---|---|---|"]

    def agg(rs, label):
        mp = [x for r in rs if r["scoring"] == "MP" for x in r["board_scores"]]
        imp = [x for r in rs if r["scoring"] == "IMP" for x in r["board_scores"]]
        mpb = sum(r["boards"] for r in rs if r["scoring"] == "MP")
        impb = sum(r["boards"] for r in rs if r["scoring"] == "IMP")

        def mean_se(xs):
            if not xs:
                return "-"
            m = sum(xs) / len(xs)
            if len(xs) < 2:
                return f"{m:.1f}"
            sd = math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))
            return f"{m:.1f} ± {sd / math.sqrt(len(xs)):.1f}"
        bid_mp = sum(r["bidding"] * r["boards"] for r in rs if r["scoring"] == "MP" and r["bidding"] is not None)
        play_mp = sum(r["play"] * r["boards"] for r in rs if r["scoring"] == "MP" and r["play"] is not None)
        bid_imp = sum(r["bidding"] for r in rs if r["scoring"] == "IMP" and r["bidding"] is not None)
        play_imp = sum(r["play"] for r in rs if r["scoring"] == "IMP" and r["play"] is not None)
        boards = mpb + impb
        mis = sum(r["misunderstandings"] for r in rs)
        bidding = (f"{bid_mp / mpb:.1f}" if mpb else "-") + " / " + (f"{bid_imp / impb:+.2f}" if impb else "-")
        play = (f"{play_mp / mpb:+.1f}" if mpb else "-") + " / " + (f"{play_imp / impb:+.2f}" if impb else "-")
        ranked = [(r["place"] - 1) / (r["entrants"] - 1) for r in rs
                  if r["place"] and r["entrants"] > 1]
        places = ", ".join(f"{r['place']}/{r['entrants']}" for r in rs) if label != "all" else \
            (f"mean rank {sum(ranked) / len(ranked):.2f} (0 first, 1 last)" if ranked else "-")
        return (f"| {label} | {boards} | {mean_se(mp)} | {mean_se(imp)} | {bidding} | {play} | "
                f"{mis / boards if boards else 0:.2f} | {places} |")
    by_day = defaultdict(list)
    for r in rows:
        by_day[r["date"]].append(r)
    for d in sorted(by_day):
        out.append(agg(by_day[d], d or "?"))
    out.append(agg(rows, "all"))
    out.append("\n± is one standard error of the mean over boards. Places: our place over the "
               "number of entrants in the file, per event.")
    return "\n".join(out) + "\n"


# ------------------------------------------------------------- self-test

def self_test(args):
    """The whole report on the synthetic fixture, with checks."""
    assert imps(420) == 9 and imps(-20) == -1 and imps(10) == 0
    assert contract_score(4, "S", 0, 10, False) == 420
    assert contract_score(3, "N", 0, 10, True) == 630
    assert contract_score(4, "H", 1, 8, True) == -500
    assert contract_score(1, "N", 2, 7, False) == 160 + 300 + 100
    assert contract_score(2, "C", 1, 8, False) == 80 + 50 + 50
    assert contract_score(3, "C", 1, 5, False) == -800
    assert contract_score(6, "S", 0, 12, True) == 1430
    assert contract_score(7, "N", 0, 13, False) == 1520
    assert mp_percent(420, [420, 450, 400]) == 50.0
    toks = "1S =1= Pass 2C! Pass AP".split()
    au = parse_auction(toks, ["1:5+ spades"])
    assert [c for c, _ in au] == ["1S", "Pass", "2C", "Pass", "Pass", "Pass"] and au[0][1] == "5+ spades"
    assert final_contract([(c, None) for c in "1S Pass 2S X Pass Pass Pass".split()], "N") == (2, "S", 1, "N")
    # 4S by N one down undoubled, par 2S= (+110): taken as doubled (-100),
    # the contract error NS's, the missing double EW's.
    e = table_errors((4, "S", 0, "N"), 9, "None", 110, rank(2, "S"))
    assert e == {"contract": [imps(210), 0], "doubling": [0, imps(50)]}, e
    games = parse_pbn(open(FIXTURE).read())
    tables, bad = tables_of(games, args.us)
    assert not bad, bad
    assert len(tables) == 72, len(tables)
    assert any(t["ours"] for t in tables)
    report = analyse([FIXTURE], args)
    assert len(report["events"]) == 3, [e["event"] for e in report["events"]]
    sc = {e["event"]: e["scoring"] for e in report["events"]}
    assert sorted(sc.values()) == ["IMP", "MP", "MP"], sc
    for ev in report["events"]:
        assert len(ev["boards"]) == 8, (ev["event"], len(ev["boards"]))
        assert ev["n_entrants"] == 3, ev["standings"]
    flags = Counter(f[0] for ev in report["events"] for b in ev["boards"] for f in b["flags"])
    assert flags["forcing-passed"] >= 1 and flags["no-rule"] >= 1, flags
    with tempfile.TemporaryDirectory() as d:
        h = os.path.join(d, "history.jsonl")
        rows = update_history(h, report)
        rows = update_history(h, report)  # idempotent per (date, event)
        assert len(rows) == 3, len(rows)
        text = render(report, args, rows)
    assert "## Running table" in text and "Worst boards" in text
    print(text)
    print(f"self-test passed: {len(tables)} tables, flags {dict(flags)}", file=sys.stderr)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("pbn", nargs="*", help="field PBN file(s)")
    ap.add_argument("--us", default="Rusty", help="our South player name (regex, case-insensitive)")
    ap.add_argument("--card", default="21GF-GIB", help="the card all four seats play (stock name or file)")
    ap.add_argument("--scoring", default="MP", choices=["MP", "IMP"],
                    help="scoring when the file does not say")
    ap.add_argument("--history", default=os.path.join(ROOT, ".rbb-cache", "brill", "history.jsonl"),
                    help="running table (JSON lines); '' to skip")
    ap.add_argument("--table-only", action="store_true", help="print the running table and stop")
    ap.add_argument("--out", help="write the report here (default stdout)")
    ap.add_argument("--json", help="also write the analysis as JSON here")
    ap.add_argument("--worst", type=int, default=5, help="how many worst boards to list")
    ap.add_argument("--no-recheck", dest="recheck", action="store_false",
                    help="do not ask today's engine for our calls again")
    ap.add_argument("--rbb", default=RBB)
    ap.add_argument("--dd-cache", default=os.path.join(ROOT, ".rbb-cache", "dd.jsonl"))
    ap.add_argument("--self-test", action="store_true", help="run on the synthetic fixture")
    args = ap.parse_args()
    if args.self_test:
        self_test(args)
        return
    if args.table_only:
        print(render_history(load_history(args.history)))
        return
    if not args.pbn:
        ap.error("give the field PBN file(s)")
    report = analyse(args.pbn, args)
    rows = update_history(args.history, report) if args.history else []
    text = render(report, args, rows)
    if args.out:
        with open(args.out, "w") as f:
            f.write(text)
        print(f"wrote {args.out}", file=sys.stderr)
    else:
        print(text)
    if args.json:
        with open(args.json, "w") as f:
            json.dump(report, f, default=str, ensure_ascii=False, indent=1)


if __name__ == "__main__":
    main()
