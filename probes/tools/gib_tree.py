#!/usr/bin/env python3
"""Our ranges against GIB's bidding tree, call by call.

    probes/tools/gib_tree.py show P-P-1S-P-2C-P [--card 21GF-GIB]
    probes/tools/gib_tree.py coverage --compare ALL.json [--ns 500,1000,...] [--tag NAME]
    probes/tools/gib_tree.py prefixes --compare ALL.json [--top 2000] [--tag NAME]
    probes/tools/gib_tree.py fetch    [--tag NAME] [--max N]
    probes/tools/gib_tree.py report   [--tag NAME] [--card 21GF-GIB] [--rules DIR]
    probes/tools/gib_tree.py agreed   [--tag NAME] [--card 21GF-GIB] [--rules DIR] [--set F=V] [--fresh]
    probes/tools/gib_tree.py run      --compare ALL.json [--top 2000]   # prefixes, fetch, report

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

**coverage**: how much of all bidding the commonest positions cover. The
positions ranked by boards through them (as `prefixes` keys them); for each
N in `--ns`, the share of all calls made from a position in the top N, by
call number and uncontested/contested (both sides have bid by then), the
call number where that falls below 99% and 90%, and the share of auctions
entirely inside. Writes `coverage-<tag>.json`. The auctions of a compare
JSON are kept in `auctions/` beside the archive, so a big file is read once.

**agreed**: the range comparison only where both systems give a call
basically the same meaning (docs/BIDDING-MODEL.md, 4.2). Each call our
engine made at a listed position is agreed, differing, gib-only (no rule
of ours) or ours-only (GIB's tree states nothing for our convention); see
`classify` and the name tables `GIB_NAMES` / `OUR_NAMES`. Ranges are
compared, as `report` does, only on agreed calls at positions whose whole
path agreed; the rest goes to the GIB-treatments queue, ranked by boards,
with the positions and call-boards dropped below each. Our readings are
kept in `ours/<tag>.jsonl` (`--fresh` after an engine or rules change).
Writes `agreed-<tag>.md` and `.json`, with the share of call-boards on
agreed paths by call number and the names neither table maps.
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


def split(k):
    return [] if k == "_" else k.split("-")


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
        return card  # a stock card the binary knows by name
    return str(Path(card).resolve())


def explain(args, calls):
    """Our rows for an auction given in GIB calls (dealer N, none vul)."""
    a = " ".join(our_call(c) for c in calls)
    cmd = [args.rbb, "explain-auction", "-a", a, "-d", "N", "-c", resolve_card(args.card), "--json"]
    if args.rules:
        cmd += ["--rules", args.rules]
    for kv in getattr(args, "set", None) or []:
        cmd += ["--set", kv]
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


# ---------------------------------------------------------- agreed paths
#
# GIB plays many treatments we don't. A range comparison means something
# only where both systems give a call basically the same meaning, and only
# at positions whose whole path agrees (docs/BIDDING-MODEL.md, 4.2). Each
# call at a position is classified:
#   agreed     both natural, the bid suit's lengths overlapping (or both a
#              natural NT, or a pass), or both artificial with the same
#              convention (an unnamed artificial GIB call opposite an
#              artificial rule of ours is an answer inside the convention
#              the path already agreed on);
#   differing  one artificial and the other natural, different conventions,
#              or natural with disjoint lengths in the bid suit;
#   gib-only   GIB has a meaning, we have no rule;
#   ours-only  we have a rule, GIB's tree has no meaning for the call.
# Below a call that is not agreed, nothing is range-compared; those calls
# go to the GIB-treatments queue, ranked by boards.

# GIB's convention names (the part before " -- "): a regex on the name, the
# convention it is. NATURAL marks GIB's descriptive names for natural calls.
NATURAL = "natural"
GIB_NAMES = [
    # GIB's descriptive names for natural calls (and answers that are natural)
    (r"No suitable call|Opener rebid.*|Opener rebids [CDHS]|Opener two rebid|Opener reverse|Rebid"
     r"|Free bid.*|Overcall|One-level overcall|Two-level overcall|Balancing overcall"
     r"|Balancing (double )?jump overcall|(Aggressive )?weak jump overcall|Jump overcall"
     r"|Preempt|Preemptive jump.*|Preemptive raise|Jump shift|Jump in notrump|Jump in competition"
     r"|Jump raise|Double jump raise|Simple raise|Raise.*|Free raise|Free major raise|Free jump raise"
     r"|Competitive raise|Support to game|Limit major raise|Invitational jump raise"
     r"|Invitational jump rebid|Jump rebid; not forcing|Jump support; not forcing"
     r"|Invitational to 3NT game|Balanced invite|Balanced minimum.*|Invite|Invitational.*"
     r"|Natural, not forcing|Natural.*|New suit.*|Jump new suit|Responder reverse|Two suited.*|Unbalanced"
     r"|One over one|Retreat from notrump|Length|Weak|Weak two .*|Solid suit|Solid [CDHS]; .*"
     r"|Game force|Strong hand.*|Strong [CDHS]; .*|One notrump overcall|Balancing notrump"
     r"|Balancing two notrump|One NT to double|Two NT to double|Penalty|Penalty pass|Opener's suit"
     r"|Support for [CDHS]; no splinter|The Law: .*|Secondary support|Other major"
     r"|No support nor new major|Positive|Quantitative invite to [67]NT"
     r"|Third hand opener passes|Long (clubs|diamonds|hearts|spades|minor)"
     r"|Support; possibly a strong hand|Strong two suiter|Minors|Opener's .*", NATURAL),
    (r"Stayman|No major suits", "stayman"),
    (r"Transfer completed.*|Correct back", NATURAL),
    (r"Smolen", "smolen"),
    (r"Jacoby transfer", "jacoby_transfer"),
    (r"Texas", "texas"),
    (r"Minor transfer", "minor_transfer"),
    (r"Minor Stayman", "minor_suit_stayman"),
    (r"Superaccept.*", "super_accept"),
    (r"Gerber.*", "gerber"),
    (r"Blackwood.*|.*[Kk]ey ?cards?.*", "blackwood"),
    (r"Grand slam force.*", "grand_slam_force"),
    (r"Splinter", "splinter"),
    (r"Drury", "drury"),
    (r"Jacoby 2N.*|Jacoby -> .*", "jacoby_2nt"),
    (r"Truscott.*", "jordan_2nt"),
    (r"Michaels", "michaels"),
    (r"Unusual notrump.*|Unusual 2N.*", "unusual_nt"),
    (r"Show your 2nd suit", "two_suit_ask"),
    (r"Takeout double|Two suit takeout", "takeout_double"),
    (r"Negative double", "negative_double"),
    (r"Penalty double", "penalty_double"),
    (r"Responsive double", "responsive_double"),
    (r"Support (re)?double.*", "support_double"),
    (r"Value-showing redouble", "redouble_values"),
    (r"Lebensohl.*", "lebensohl"),
    (r"Cappelletti.*", "cappelletti"),
    (r"New minor forcing", "new_minor_forcing"),
    (r"Fourth suit forcing", "fourth_suit_forcing"),
    (r"Impossible spade", "impossible_2s"),
    (r"Cue: limit raise or better|Cue bid.*|Good support in [CDHS]", "cue_bid"),
    (r"Mixed raise", "mixed_raise"),
    (r"Strong two club", "strong_2c"),
    (r"2D bid waiting", "waiting_2d"),
    (r"Artificial force|Feature", "weak_two_ask"),
    (r"Game try suit|Unspecified shortness, game try", "game_try"),
]

# Our conventions: a module (full match) or an explanation (search), the
# convention it is; the first entry that matches wins.
OUR_NAMES = [
    (r"minor-suit-stayman", r"^Minor Suit Stayman", "minor_suit_stayman"),
    (r"stayman|puppet-1nt", r"^Stayman|^No (4|four)-card major", "stayman"),
    (r"smolen", r"^Smolen", "smolen"),
    (r"texas-transfers", r"^Texas", "texas"),
    (r"minor-transfers", r"[Tt]ransfer to (clubs|diamonds|♣|♦)|[Mm]inor.suit transfer", "minor_transfer"),
    (r"jacoby-transfers", r"^(Jacoby )?[Tt]ransfer to (hearts|spades|♥|♠)", "jacoby_transfer"),
    (r"super-accept|superaccept-doubleton", r"[Ss]uper.?accept", "super_accept"),
    (r"gerber", r"^Gerber", "gerber"),
    (r"exclusion", r"^Exclusion", "exclusion"),
    (r"blackwood|rkcb-1430|kickback|minorwood", r"Blackwood|[Kk]eycard", "blackwood"),
    (r"", r"[Gg]rand slam force", "grand_slam_force"),
    (r"splinters|nt-splinter", r"[Ss]plinter", "splinter"),
    (r"drury", r"^Drury", "drury"),
    (r"jacoby-2nt|modified-jacoby-2nt", r"^Jacoby 2NT", "jacoby_2nt"),
    (r"", r"Jordan", "jordan_2nt"),
    (r"", r"Michaels", "michaels"),
    (r"", r"Unusual \dNT|[Uu]nusual notrump", "unusual_nt"),
    (r"", r"Asks for your minor|Which is your minor", "two_suit_ask"),
    (r"responsive-doubles", r"[Rr]esponsive double", "responsive_double"),
    (r"support-doubles", r"[Ss]upport (re)?double", "support_double"),
    (r"", r"[Nn]egative double", "negative_double"),
    (r"takeout-double", r"[Tt]akeout|unbid suits|Reopening double", "takeout_double"),
    (r"penalty-doubles", r"[Pp]enalty", "penalty_double"),
    (r"", r"^Redouble", "redouble_values"),
    (r"", r"^Values", "values_double"),
    (r"lebensohl", r"[Ll]ebensohl", "lebensohl"),
    (r"cappelletti", r"^Cappelletti", "cappelletti"),
    (r"new-minor-forcing|nmf-after-2nt|two-way-nmf", r"[Nn]ew [Mm]inor [Ff]orcing", "new_minor_forcing"),
    (r"fourth-suit-forcing", r"[Ff]ourth suit", "fourth_suit_forcing"),
    (r"impossible-2s", r"[Ii]mpossible", "impossible_2s"),
    (r"control-bids", r"[Cc]ue.?bid|[Cc]ontrol bid|limit raise or better", "cue_bid"),
    (r"", r"Mixed raise", "mixed_raise"),
    (r"", r"^Strong 2♣", "strong_2c"),
    (r"", r"^Waiting|[Ss]econd negative", "waiting_2d"),
    (r"", r"^Asks: how good|Ogust|[Ff]eature", "weak_two_ask"),
    (r"game-tries", r"[Gg]ame try", "game_try"),
    (r"namyats", r"^Namyats", "namyats"),
]

# Conventions that are one thing seen from different seats.
FAMILY = {"negative_double": "takeout_double", "responsive_double": "takeout_double",
          "mixed_raise": "cue_bid", "values_double": "penalty_double"}

# GIB names a natural description that our convention calls: GIB's
# "Two suited" over their suit is our Michaels; an answer that completes a
# transfer or relay agrees with whatever completes it on our side; GIB names
# Lebensohl's natural forcing bids "Lebensohl".
TWO_SUITED = (r"Two suited.*|Strong two suiter|Minors", {"michaels", "unusual_nt"})
ANSWERS = r"Transfer completed.*|Correct back"
NATURAL_IN = {"lebensohl"}


def fam(k):
    return FAMILY.get(k, k)


def gib_key(name):
    """GIB's name as a convention, NATURAL (also for no name), or None when the
    table has no entry."""
    if not name:
        return NATURAL
    for rx, k in GIB_NAMES:
        if re.fullmatch(rx, name):
            return k
    return None


def our_key(module, means):
    for mrx, erx, k in OUR_NAMES:
        if mrx and re.fullmatch(mrx, module or "") or re.search(erx, means or ""):
            return k
    return None


def shows_suit(g, s):
    """GIB's meaning shows the bid suit: three or more, or (re)biddable."""
    lo = (g["len"].get(s) or (None, None))[0]
    return (lo or 0) >= 3 or any(re.search(rf"biddable {s}\b", o) for o in g["other"])


def gib_double(g):
    """An unnamed GIB double: takeout when it shows three or more in two suits."""
    return "takeout_double" if sum((lo or 0) >= 3 for lo, _ in g["len"].values()) >= 2 \
        else "penalty_double"


def classify(call, gib_m, ours):
    """(class, reason, gib name, gib key) for one call our engine made.
    The keys are conventions; natural calls agree when the bid suit's lengths
    overlap; an unnamed GIB call silent on the suit it bids (an answer, a
    relay, a preference) agrees with whatever we bid there."""
    if ours is None or ours.get("rule") is None:
        g = parse_meaning(gib_m or "")
        return "gib-only", "no rule of ours", g["name"], gib_key(g["name"])
    art = bool(ours.get("artificial"))
    ok = our_key(ours.get("module"), ours.get("means"))
    if call == "P":
        return "agreed", "pass", "", NATURAL
    if gib_m is None or not gib_m.strip():  # GIB states nothing
        if art and ok:
            return "ours-only", f"ours {ok}; GIB's tree states nothing", "", None
        return "agreed", "GIB states nothing", "", NATURAL
    g = parse_meaning(gib_m)
    name = g["name"]
    gk = gib_key(name)
    if call in ("D", "R"):
        if gk is None:
            return "differing", f"GIB {name} (unmapped) / ours {ok or 'unmapped'}", name, gk
        if gk == NATURAL:
            gk = gib_double(g) if call == "D" else "redouble_values"
        if ok and fam(ok) == fam(gk):
            return "agreed", fam(gk), name, gk
        return "differing", f"GIB {gk} / ours {ok or 'unmapped'}", name, gk
    strain = call[1]
    if gk not in (None, NATURAL) and ok and fam(ok) == fam(gk):
        return "agreed", fam(gk), name, gk
    if gk is None:  # an unmapped name: natural when it shows the suit (or is NT)
        gk = NATURAL if (strain == "N" or shows_suit(g, strain)) else "artificial?"
    elif gk == NATURAL and not name and strain != "N" and not shows_suit(g, strain):
        gk = "artificial?"
    if gk == "artificial?":
        return "agreed", "answer or relay (GIB unnamed, silent on the suit)", name, gk
    if re.fullmatch(ANSWERS, name):
        return "agreed", "completes a transfer or relay", name, gk
    if re.fullmatch(TWO_SUITED[0], name) and ok in TWO_SUITED[1]:
        return "agreed", ok, name, ok
    if gk in NATURAL_IN and not art and (strain == "N" or shows_suit(g, strain)):
        return "agreed", f"natural inside {gk}", name, gk
    if gk == NATURAL:
        if art:
            return "differing", f"GIB natural / ours artificial ({ok or 'unmapped'})", name, gk
        if strain != "N":
            glo, ghi = g["len"].get(strain) or (None, None)
            olo, ohi = ours["state"]["len"][strain]
            if glo is not None and ohi < glo or ghi is not None and olo > ghi:
                return "differing", "natural, disjoint lengths in the suit", name, gk
        return "agreed", "natural", name, gk
    if ok:
        return "differing", f"GIB {gk} / ours {ok}", name, gk
    if not art:
        return "differing", f"GIB {gk} / ours natural", name, gk
    return "differing", f"GIB {gk} / ours unmapped", name, gk


def compact(row):
    """What the agreed-path report keeps of one of our rows."""
    if row is None:
        return None
    st = row["step"]
    o = our_state(row)
    rule = st.get("rule")
    if rule is None or o is None:
        return {"rule": None, "module": None, "means": st.get("explanation") or "",
                "artificial": st.get("artificial"), "state": o}
    f = rule["file"]
    rel = f.split("conventions/", 1)[-1] if "/conventions/" in f else f.split("rules/", 1)[-1]
    return {"rule": f"{rel}:{rule['line']}", "module": rule.get("module"),
            "means": st.get("explanation") or "", "artificial": st.get("artificial"),
            "state": {"hcp": o["hcp"], "pts": o["pts"], "unit": o["unit"], "len": o["len"]}}


def ours_cache_path(args):
    return CACHE / "ours" / f"{args.tag}.jsonl"


def read_ours(args, jobs):
    """{auction key (prefix + call): (row, the caller's previous row)} for each
    job, explained once and kept in ours/<tag>.jsonl (--fresh redoes it: after
    an engine or rules change)."""
    p = ours_cache_path(args)
    have = {}
    if p.exists() and not args.fresh:
        for line in open(p):
            d = json.loads(line)
            have[d["a"]] = (d["row"], d["prev"])
    todo = [j for j in jobs if key(j) not in have]
    if todo:
        print(f"reading ours for {len(todo)} calls ({len(have)} kept) ...", file=sys.stderr, flush=True)
        p.parent.mkdir(parents=True, exist_ok=True)

        def work(calls):
            rows = explain(args, calls)
            i = len(calls) - 1
            if rows is None or len(rows) <= i:
                return key(calls), None, None
            return key(calls), compact(rows[i]), compact(rows[i - 4]) if i >= 4 else None
        with open(p, "w" if args.fresh else "a") as f, ThreadPoolExecutor(args.jobs) as ex:
            for n, (k, row, prev) in enumerate(ex.map(work, todo), 1):
                have[k] = (row, prev)
                f.write(json.dumps({"a": k, "row": row, "prev": prev}, ensure_ascii=False) + "\n")
                if n % 2000 == 0:
                    print(f"  {n}/{len(todo)}", file=sys.stderr, flush=True)
    return have


def state_diffs(o, g_m, prev_o, prev_g_m, tol):
    """diffs() on compact states, less those already at the caller's previous call."""
    def norm(st):
        return {"hcp": tuple(st["hcp"]), "pts": tuple(st["pts"]),
                "len": {s: tuple(v) for s, v in st["len"].items()}}
    ds = diffs(norm(o), parse_meaning(g_m), tol)
    if prev_o and prev_g_m and prev_g_m.strip():
        sig = lambda d: (d["dim"], d["kind"], tuple(d["ours"]),
                         tuple(x if x is not None else -1 for x in d["gib"]))
        old = {sig(d) for d in diffs(norm(prev_o), parse_meaning(prev_g_m), tol)}
        ds = [d for d in ds if sig(d) not in old]
    return ds


def cmd_agreed(args):
    rows = load_prefixes(args)
    boards_at = {r["auction"]: r["boards"] for r in rows}
    trees = {}
    for r in rows:
        p = xml_path(split(r["auction"]))
        if p.exists():
            trees[r["auction"]] = tree(p.read_bytes())
    jobs = [split(r["auction"]) + [c] for r in rows if r["auction"] in trees for c in r["calls"]]
    print(f"{len(rows)} positions, {len(trees)} archived; {len(jobs)} calls", file=sys.stderr)
    ours = read_ours(args, jobs)

    # classify every call at every archived position
    cls = {}
    for r in rows:
        a = r["auction"]
        if a not in trees:
            continue
        for c, n in r["calls"].items():
            row, prev = ours.get(key(split(a) + [c]), (None, None))
            k, why, name, gk = classify(c, trees[a].get(c), row)
            cls[(a, c)] = {"class": k, "why": why, "gib_name": name, "gib_key": gk, "boards": n,
                           "row": row, "prev": prev}

    # each position's path: agreed when every call on it was; else the first
    # call that was not (or "unfetched": a position on it is not archived)
    path = {}
    for r in rows:  # ranked, so a prefix comes before its extensions
        a = r["auction"]
        pre = split(a)
        if not pre:
            path[a] = ("agreed", None)
            continue
        up = key(pre[:-1])
        st = path.get(up, ("unfetched", None))
        if st[0] != "agreed":
            path[a] = st
            continue
        c = cls.get((up, pre[-1]))
        if c is None:
            path[a] = ("unfetched", None)
        elif c["class"] == "agreed":
            path[a] = ("agreed", None)
        else:
            path[a] = ("dropped", (up, pre[-1]))

    # positions and call-boards dropped below each first disagreeing call
    below = defaultdict(lambda: [0, 0])
    for r in rows:
        st = path.get(r["auction"])
        if st and st[0] == "dropped":
            below[st[1]][0] += 1
            below[st[1]][1] += r["boards"]

    by_call = defaultdict(lambda: Counter())
    kinds = Counter()
    groups = defaultdict(lambda: {"boards": 0, "calls": 0, "kinds": Counter(), "examples": []})
    queue = []
    unmapped_gib = Counter()
    unmapped_ours = Counter()
    agreed_why = Counter()
    for (a, c), x in cls.items():
        depth = len(split(a)) + 1
        n = x["boards"]
        on_path = path.get(a, ("unfetched",))[0] == "agreed"
        by_call[depth]["all"] += n
        if x["gib_key"] is None and x["gib_name"]:
            unmapped_gib[x["gib_name"]] += n
        if x["row"] and x["row"].get("artificial") and x["row"].get("rule") \
                and our_key(x["row"]["module"], x["row"]["means"]) is None:
            unmapped_ours[f"{x['row']['module']}: {x['row']['means']}"] += n
        if not on_path:
            by_call[depth][path.get(a, ("unfetched",))[0]] += n
            continue
        by_call[depth]["path"] += n
        by_call[depth][x["class"]] += n
        if x["class"] == "agreed":
            agreed_why[x["why"]] += n
        if x["class"] != "agreed":
            row = x["row"] or {}
            queue.append({"auction": a, "call": c, "boards": n, "class": x["class"], "why": x["why"],
                          "gib": trees[a].get(c) or "", "gib_name": x["gib_name"],
                          "our_rule": row.get("rule"), "our_means": row.get("means", ""),
                          "dropped_positions": below[(a, c)][0], "dropped_boards": below[(a, c)][1]})
            continue
        row, prev = x["row"], x["prev"]
        pre = split(a)
        prev_g = None
        if prev and len(pre) >= 4 and key(pre[:-4]) in trees:
            prev_g = trees[key(pre[:-4])].get(pre[-4])
        ds = state_diffs(row["state"], trees[a][c], (prev or {}).get("state"), prev_g, args.tol)
        for d in ds:
            kinds[f"{d['dim'] if d['dim'] in ('hcp', 'pts') else 'len'} {d['kind']}"] += n
        if not ds:
            continue
        g = groups[(row["rule"], row["means"])]
        g["boards"] += n
        g["calls"] += 1
        for d in ds:
            g["kinds"][f"{d['dim']} {d['kind']}"] += n
        g["examples"].append({"auction": a, "call": c, "boards": n, "gib": trees[a][c],
                              "ours": row["state"], "diffs": ds})
    queue.sort(key=lambda q: -q["boards"])
    tq = defaultdict(lambda: {"boards": 0, "positions": 0, "dropped_boards": 0, "examples": []})
    for q in queue:
        t = tq[(q["class"], q["gib_name"] or q["why"])]
        t["boards"] += q["boards"]
        t["positions"] += 1
        t["dropped_boards"] += q["dropped_boards"]
        if len(t["examples"]) < 3:
            t["examples"].append(q)
    out = {"card": args.card, "tol": args.tol, "positions": len(rows), "archived": len(trees),
           "calls": len(cls),
           "by_call": {d: dict(v) for d, v in sorted(by_call.items())},
           "kinds": dict(kinds.most_common()),
           "groups": [{"rule": k[0], "means": k[1], "boards": g["boards"], "calls": g["calls"],
                       "kinds": dict(g["kinds"].most_common()),
                       "examples": sorted(g["examples"], key=lambda c: -c["boards"])[:5]}
                      for k, g in sorted(groups.items(), key=lambda kv: -kv[1]["boards"])],
           "queue": queue,
           "treatments": [{"class": k[0], "gib_name": k[1], **v}
                          for k, v in sorted(tq.items(), key=lambda kv: -kv[1]["boards"])],
           "unmapped_gib": dict(unmapped_gib.most_common()),
           "unmapped_ours": dict(unmapped_ours.most_common()),
           "agreed_why": dict(agreed_why.most_common())}
    base = CACHE / f"agreed-{args.tag}"
    json.dump(out, open(f"{base}.json", "w"), indent=1, ensure_ascii=False)
    open(f"{base}.md", "w").write(agreed_markdown(out, args.groups))
    print(agreed_summary(out))
    print(f"-> {base}.md, {base}.json")


def agreed_shares(out):
    L = ["call #  call-boards  on agreed path  agreed call  (of the call-boards at fetched positions)"]
    for d, v in out["by_call"].items():
        a = v.get("all", 0)
        if not a:
            continue
        L.append(f"{d:>6}  {a:>11}  {100 * v.get('path', 0) / a:13.0f}%  {100 * v.get('agreed', 0) / a:10.0f}%")
    return "\n".join(L)


def agreed_summary(out):
    tot = sum(v.get("all", 0) for v in out["by_call"].values())
    agr = sum(v.get("agreed", 0) for v in out["by_call"].values())
    return (f"{out['calls']} calls at {out['archived']} archived positions; {agr}/{tot} call-boards "
            f"agreed on agreed paths\n" + agreed_shares(out))


def agreed_markdown(o, ngroups):
    L = [f"# GIB's tree against ours on agreed paths ({o['card']})", "",
         f"{o['calls']} calls at {o['archived']} of {o['positions']} positions (archived).", "",
         "## Call-boards on agreed paths, by call number", "", "```", agreed_shares(o), "```", "",
         "Agreed, by why (call-boards): " + ", ".join(f"{k} {v}" for k, v in o["agreed_why"].items()), "",
         "## Range differences on agreed calls", "", "| difference | call-boards |", "|---|---|"]
    L += [f"| {k} | {v} |" for k, v in o["kinds"].items()]
    L += ["", f"### The top {ngroups} rules by boards", ""]
    for g in o["groups"][:ngroups]:
        L.append(f"#### {g['rule']}: {g['means']}  ({g['boards']} boards, {g['calls']} positions)")
        L.append("")
        L.append(", ".join(f"{k} {v}" for k, v in g["kinds"].items()))
        L.append("")
        for c in g["examples"][:3]:
            L.append(f"- `{c['auction']}` **{c['call']}** ({c['boards']} boards). GIB: {c['gib']}")
            L.append(f"  - ours: {fr(c['ours']['hcp'])} HCP, {fr(c['ours']['pts'])} {c['ours']['unit']}; "
                     + "; ".join(describe(d) for d in c["diffs"]))
        L.append("")
    L += ["## GIB treatments queue", "",
          "Calls on agreed paths where the meanings part, grouped by GIB's name, by boards. "
          "Below each, nothing was range-compared.", "",
          "| class | GIB name | call-boards | positions | call-boards dropped below | example |",
          "|---|---|---|---|---|---|"]
    for t in o["treatments"][:60]:
        e = t["examples"][0]
        L.append(f"| {t['class']} | {t['gib_name'] or '(unnamed)'} | {t['boards']} | {t['positions']} | "
                 f"{t['dropped_boards']} | `{e['auction']}` {e['call']}: GIB {e['gib']}; ours "
                 f"{e['our_rule'] or '-'} {e['our_means']} |")
    L += ["", "### The top 60 calls", "",
          "| position | call | boards | class | why | GIB | ours | dropped below (positions / call-boards) |",
          "|---|---|---|---|---|---|---|---|"]
    for q in o["queue"][:60]:
        L.append(f"| `{q['auction']}` | {q['call']} | {q['boards']} | {q['class']} | {q['why']} | {q['gib']} | "
                 f"{q['our_rule'] or '-'} {q['our_means']} | {q['dropped_positions']} / {q['dropped_boards']} |")
    L += ["", "## Unmapped names", "", "GIB's (call-boards): "
          + ", ".join(f"{k} {v}" for k, v in list(o["unmapped_gib"].items())[:40]), "",
          "Our artificial rules with no convention entry (call-boards): "
          + "; ".join(f"{k} {v}" for k, v in list(o["unmapped_ours"].items())[:40])]
    return "\n".join(L) + "\n"


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
        k, why, _, _ = classify(call, m, compact(rows[len(prefix)]))
        print(f"{call:>3}  GIB  {m}")
        print(f"     {k}: {why}")
        o = c["ours"]
        lens = " ".join(f"{SYM[s]}{fr(o['len'][s])}" for s in "SHDC" if o["len"][s] != (0, 13))
        print(f"     ours {c['our_means']} [{c['our_rule']}]: {fr(o['hcp'])} HCP, "
              f"{fr(o['pts'])} {o['unit']}  {lens}")
        for d in c["diffs"]:
            print(f"       ≠ {describe(d)}")
        print()
    if missing:
        print(f"GIB has, we have no rule for: {' '.join(missing)}")


def load_auctions(compare):
    """Our auctions in a compare JSON, in GIB calls, each read as dealt by N
    (GIB's tree starts at the dealer). The list is kept beside the archive,
    keyed by the file's size and time, so a big JSON is read once."""
    src = Path(compare).resolve()
    st = src.stat()
    cached = CACHE / "auctions" / f"{src.stem}-{st.st_size}-{int(st.st_mtime)}.txt"
    if cached.exists():
        return [split(line.strip()) for line in open(cached) if line.strip()]
    print(f"reading {compare} ...", file=sys.stderr, flush=True)
    data = json.load(open(src))
    out = []
    for b in data["boards"]:
        calls = b.get("ours")
        if isinstance(calls, str):
            calls = ast.literal_eval(calls)
        if calls:
            out.append([gib_call(c) for c in calls])
    del data
    cached.parent.mkdir(parents=True, exist_ok=True)
    tmp = cached.with_suffix(".tmp")
    with open(tmp, "w") as f:
        for a in out:
            f.write(key(a) + "\n")
    tmp.rename(cached)
    return out


def positions(auctions):
    """{position key: Counter of the calls made there}."""
    pos = defaultdict(Counter)
    for calls in auctions:
        for i, c in enumerate(calls):
            pos[key(calls[:i])][c] += 1
    return pos


def ranked_positions(pos):
    """Position keys by boards through them, most first (ties by length, then
    auction, so the order is stable and a prefix comes before its extensions)."""
    return sorted(pos, key=lambda k: (-sum(pos[k].values()), len(split(k)), k))


def contested(prefix):
    """Both sides have made a call other than pass by then."""
    bid = [False, False]
    for i, c in enumerate(prefix):
        if c != "P":
            bid[i % 2] = True
    return all(bid)


def cmd_coverage(args):
    """Share of all calls made from a position within the top N, by call number
    and by uncontested/contested, for several N; how deep the top N reach; the
    share of whole auctions entirely inside."""
    auctions = load_auctions(args.compare)
    pos = positions(auctions)
    order = ranked_positions(pos)
    rank = {k: i for i, k in enumerate(order)}
    ns = [int(x) for x in args.ns.split(",")]
    tot = Counter()
    tot_c = Counter()
    hit = {n: Counter() for n in ns}
    hit_c = {n: Counter() for n in ns}
    worst = []  # per auction, the worst rank among its positions
    for calls in auctions:
        w = 0
        for i in range(len(calls)):
            r = rank[key(calls[:i])]
            w = max(w, r)
            c = contested(calls[:i])
            tot[i + 1] += 1
            if c:
                tot_c[i + 1] += 1
            for n in ns:
                if r < n:
                    hit[n][i + 1] += 1
                    if c:
                        hit_c[n][i + 1] += 1
        worst.append(w)
    ncalls = sum(tot.values())
    res = {"source": str(args.compare), "boards": len(auctions), "positions": len(pos),
           "calls": ncalls, "by_call": {i: {"calls": tot[i], "contested": tot_c[i]} for i in sorted(tot)},
           "top": []}
    for n in ns:
        by = {}
        for i in sorted(tot):
            u, uc = tot[i] - tot_c[i], hit[n][i] - hit_c[n][i]
            by[i] = {"all": hit[n][i] / tot[i],
                     "uncontested": uc / u if u else None,
                     "contested": hit_c[n][i] / tot_c[i] if tot_c[i] else None}

        def below(th):
            return next((i for i in sorted(tot) if by[i]["all"] < th), None)
        u_all = sum(tot.values()) - sum(tot_c.values())
        res["top"].append({"n": n, "calls": sum(hit[n].values()) / ncalls,
                           "uncontested": (sum(hit[n].values()) - sum(hit_c[n].values())) / u_all,
                           "contested": sum(hit_c[n].values()) / sum(tot_c.values()),
                           "auctions": sum(1 for w in worst if w < n) / len(auctions),
                           "below_99": below(0.99), "below_90": below(0.90), "by_call": by})
    out = CACHE / f"coverage-{args.tag}.json"
    CACHE.mkdir(parents=True, exist_ok=True)
    json.dump(res, open(out, "w"), indent=1)
    print(coverage_table(res, args.depth))
    print(f"\n-> {out}")


def coverage_table(res, depth):
    def pc(x):
        if x is None:
            return "   -"
        return " 99+" if 0.995 <= x < 1 else f"{100 * x:4.0f}"
    L = [f"{res['boards']} auctions, {res['calls']} calls, {res['positions']} positions",
         "", "share of the calls made at each call number from a position in the top N",
         "(all / uncontested / contested; contested: both sides have bid by then)", ""]
    calls = sorted(res["by_call"], key=int)[:depth]
    L.append("call #      " + "".join(f"{c:>5}" for c in calls))
    L.append("calls (k)   " + "".join(f"{res['by_call'][c]['calls'] / 1000:5.0f}" for c in calls))
    L.append("contested % " + "".join(
        f"{100 * res['by_call'][c]['contested'] / res['by_call'][c]['calls']:5.0f}" for c in calls))
    for t in res["top"]:
        L.append("")
        L.append(f"top {t['n']}: {100 * t['calls']:.1f}% of calls ({100 * t['uncontested']:.1f}% "
                 f"uncontested, {100 * t['contested']:.1f}% contested), {100 * t['auctions']:.1f}% of "
                 f"auctions entirely inside; below 99% from call {t['below_99']}, "
                 f"below 90% from call {t['below_90']}")
        for part in ("all", "uncontested", "contested"):
            L.append(f"  {part:<10}" + "".join(f" {pc(t['by_call'][c][part])}" for c in calls))
    return "\n".join(L)


def cmd_prefixes(args):
    auctions = load_auctions(args.compare)
    total = len(auctions)
    pos = positions(auctions)
    ranked = [(k, pos[k]) for k in ranked_positions(pos)[: args.top]]
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
    ap.add_argument("cmd", choices=["show", "coverage", "prefixes", "fetch", "report", "agreed", "run"])
    ap.add_argument("auction", nargs="?", help="show: the auction so far (P-P-1S-P)")
    ap.add_argument("--compare", help="a compare JSON (prefixes, run)")
    ap.add_argument("--top", type=int, default=2000, help="positions to keep (prefixes)")
    ap.add_argument("--tag", default="main", help="names the position list and report")
    ap.add_argument("--max", type=int, default=0, help="fetch at most N this run")
    ap.add_argument("--interval", type=float, default=1.0, help="seconds between requests (>= 1)")
    ap.add_argument("--card", default="21GF-GIB", help="a stock card's name or a path")
    ap.add_argument("--rules", help="rules for our readings (default: compiled in)")
    ap.add_argument("--set", action="append", default=[], metavar="PATH=VALUE",
                    help="a card field for our readings (repeatable), e.g. general.style=gib")
    ap.add_argument("--rbb", default=str(ROOT / "target/release/rbb"))
    ap.add_argument("--tol", type=int, default=2)
    ap.add_argument("--jobs", type=int, default=6)
    ap.add_argument("--groups", type=int, default=40, help="rules listed in the markdown")
    ap.add_argument("--fresh", action="store_true", help="agreed: read ours again (engine or rules changed)")
    ap.add_argument("--ns", default="500,1000,2000,5000,10000,20000", help="coverage: the top Ns")
    ap.add_argument("--depth", type=int, default=12, help="coverage: call numbers in the table")
    args = ap.parse_args()
    if args.cmd == "show":
        if args.auction is None:
            ap.error("show needs an auction")
        cmd_show(args)
    elif args.cmd == "coverage":
        if not args.compare:
            ap.error("coverage needs --compare")
        cmd_coverage(args)
    elif args.cmd == "prefixes":
        if not args.compare:
            ap.error("prefixes needs --compare")
        cmd_prefixes(args)
    elif args.cmd == "fetch":
        cmd_fetch(args)
    elif args.cmd == "report":
        cmd_report(args)
    elif args.cmd == "agreed":
        cmd_agreed(args)
    else:
        if not args.compare:
            ap.error("run needs --compare")
        cmd_prefixes(args)
        cmd_fetch(args)
        cmd_report(args)


if __name__ == "__main__":
    main()
