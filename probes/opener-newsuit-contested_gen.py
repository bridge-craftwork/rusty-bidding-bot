#!/usr/bin/env python3
"""Write the grid specs of the "opener's new suit once they come in" probe
(probes/opns-*.toml, opns-r-*.toml; Rick 2026-11-07: "yes to 1 and 2").

    probes/opener-newsuit-contested_gen.py [--count 160] [--responder]

South opened one of a minor, partner responded at the one level and they
overcalled; South holds four (a major) or four or more (a minor) of an
unbid suit he could name naturally. South's hands are 11-20 HCP, any
shape consistent with a SAYC minor opening (no five-card major, not a
balanced 15-17 or 20-21). Label `h<HCP> st<0|1> z<length in the new
suit> o<length in the opened minor>` (st: a stopper in their suit, as
`stop(x)` in eval.rs), so probes/opnt_tally.py reads it (use --z).
Each hand gets its own partner: 6-11 HCP with four or more of the
response suit and at most three of theirs.

--strong writes opnj-*: 17-21 unbalanced openers with a new suit
(jump shifts and reverses, Rick 2026-11-07: "5: yes").

--responder writes the specs of responder's next call after opener's
natural new suit (opns-r-*), opener's hand made to fit BBA's meaning.
"""
import argparse
import random

RANKS = "AKQJT98765432"
HCP = {"A": 4, "K": 3, "Q": 2, "J": 1}
SI = {"S": 0, "H": 1, "D": 2, "C": 3}

# name, prefix, opened minor, partner's suit, their suit, opener's new suit
POSITIONS = [
    ("opns-1C-1D-1H", "1C Pass 1D 1H", "C", "D", "H", "S"),
    ("opns-1C-1D-2H", "1C Pass 1D 2H", "C", "D", "H", "S"),
    ("opns-1C-1D-1S", "1C Pass 1D 1S", "C", "D", "S", "H"),
    ("opns-1C-1H-1S", "1C Pass 1H 1S", "C", "H", "S", "D"),
    ("opns-1D-1H-1S", "1D Pass 1H 1S", "D", "H", "S", "C"),
    ("opns-1D-1S-2C", "1D Pass 1S 2C", "D", "S", "C", "H"),
    ("opns-1C-1S-2H", "1C Pass 1S 2H", "C", "S", "H", "D"),
]

# Strong unbalanced openers (--strong; Rick 2026-11-07, "5: yes"): 17-21
# HCP, not balanced, four or more of the new suit (a major: four or five,
# shorter than the opened minor). Same columns as POSITIONS.
STRONG = [
    ("opnj-1C-1D-2H", "1C Pass 1D 2H", "C", "D", "H", "S"),
    ("opnj-1C-1D-1S", "1C Pass 1D 1S", "C", "D", "S", "H"),
    ("opnj-1C-1D-1H", "1C Pass 1D 1H", "C", "D", "H", "S"),
    ("opnj-1C-1H-1S", "1C Pass 1H 1S", "C", "H", "S", "D"),
    ("opnj-1C-1S-2H", "1C Pass 1S 2H", "C", "S", "H", "D"),
    ("opnj-1D-1S-2C", "1D Pass 1S 2C", "D", "S", "C", "H"),
    ("opnj-1D-1H-1S", "1D Pass 1H 1S", "D", "H", "S", "C"),
    ("opnj-1D-1H-2S", "1D Pass 1H 2S", "D", "H", "S", "C"),
    ("opnj-1D-1S-2H", "1D Pass 1S 2H", "D", "S", "H", "C"),
    ("opnj-1C-1H-2S", "1C Pass 1H 2S", "C", "H", "S", "D"),
]

# Responder after opener's new suit: name, prefix, responder's suit,
# their suit, opener's hand test (lengths L spades first, HCP h), responder's HCP.
RESP = [
    ("opns-r-1C-1D-1H-1S", "1C Pass 1D 1H 1S Pass", "D", "H",
     "12 <= h <= 16 and L[0] == 4 and L[1] <= 3 and L[3] >= 3 and L[3] >= L[2] and not (bal and 15 <= h <= 17)", (4, 14)),
    ("opns-r-1C-1D-1H-2S", "1C Pass 1D 1H 2S Pass", "D", "H",
     "16 <= h <= 19 and L[0] == 4 and L[1] <= 2 and L[3] >= 4 and L[3] >= L[2]", (4, 14)),
    # Rick 2026-11-07, "6: yes": 1D P 1H (1S) 2C with five-five in the minors
    ("opns-r-1D-1H-1S-2C", "1D Pass 1H 1S 2C Pass", "H", "S",
     "13 <= h <= 17 and L[2] >= 5 and L[3] >= 5 and L[2] >= L[3]", (4, 14)),
    # Rick 2026-11-07, "5: yes": after BBA's strong new suit, and its 13-19 3C
    ("opns-r-1C-1D-2H-2S", "1C Pass 1D 2H 2S Pass", "D", "H",
     "19 <= h <= 21 and 4 <= L[0] <= 5 and L[3] >= 5 and L[3] > L[0] and L[1] <= 2", (4, 14)),
    ("opns-r-1C-1D-1S-2H", "1C Pass 1D 1S 2H Pass", "D", "S",
     "19 <= h <= 21 and 4 <= L[1] <= 5 and L[3] >= 5 and L[3] > L[1] and L[0] <= 2", (4, 14)),
    ("opns-r-1D-1S-2C-2H", "1D Pass 1S 2C 2H Pass", "S", "C",
     "19 <= h <= 21 and 4 <= L[1] <= 5 and L[2] >= 5 and L[2] > L[1] and L[0] <= 3", (4, 14)),
    ("opns-r-1C-1S-2H-3D", "1C Pass 1S 2H 3D Pass", "S", "H",
     "18 <= h <= 21 and L[2] >= 4 and L[3] >= 5 and L[0] <= 2", (4, 14)),
    ("opns-r-1D-1H-2S-3C", "1D Pass 1H 2S 3C Pass", "H", "S",
     "13 <= h <= 19 and L[3] >= 4 and L[2] >= 4 and L[2] >= L[3] and L[1] <= 3", (4, 14)),
    ("opns-r-1D-1S-2H-3C", "1D Pass 1S 2H 3C Pass", "S", "H",
     "13 <= h <= 19 and L[3] >= 4 and L[2] >= 4 and L[2] >= L[3] and L[0] <= 3", (4, 14)),
]


def deal13(rng, deck):
    return rng.sample(deck, 13)


def pbn(cards):
    return ".".join(
        "".join(sorted((r for s2, r in cards if s2 == s), key=RANKS.index)) for s in range(4)
    )


def hcp(cards):
    return sum(HCP.get(r, 0) for _, r in cards)


def lengths(cards):
    return [sum(1 for s2, _ in cards if s2 == s) for s in range(4)]


def balanced(L):
    return sorted(L) in ([3, 3, 3, 4], [2, 3, 4, 4], [2, 3, 3, 5])


def stop(cards, s):
    rs = [r for s2, r in cards if s2 == s]
    n = len(rs)
    return "A" in rs or ("K" in rs and n >= 2) or ("Q" in rs and n >= 3) or ("J" in rs and n >= 4)


def opener_ok(c, opened, resp, theirs, new, strong=False):
    L = lengths(c)
    h = hcp(c)
    if strong:
        if balanced(L) or L[SI[new]] < 4 or max(L) > 7:
            return False
        if L[0] >= 5 or L[1] >= 5:
            if not (L[SI[new]] in (4, 5) and L[SI[opened]] > L[SI[new]] and max(L[0], L[1]) == L[SI[new]]):
                return False
        if opened == "C" and not (L[3] >= L[2] and L[3] >= 4):
            return False
        if opened == "D" and not (L[2] >= L[3] and L[2] >= 4):
            return False
        if resp in ("H", "S") and L[SI[resp]] >= 4:
            return False
        return True
    if L[0] >= 5 or L[1] >= 5:
        return False
    if balanced(L) and (15 <= h <= 17 or h >= 20):
        return False
    if opened == "C" and not (L[3] >= 3 and L[3] >= L[2] and not (L[3] == 4 and L[2] == 4 and L[3] == 4)):
        return False
    if opened == "D" and not (L[2] >= 4 and L[2] >= L[3] or (L[2] == 3 and L[3] <= 2 and L[0] == 4 and L[1] == 4)):
        return False
    n = L[SI[new]]
    if new in ("H", "S") and n != 4:
        return False
    if new in ("C", "D") and n < 4:
        return False
    if resp in ("H", "S") and L[SI[resp]] >= 4:
        return False
    if max(L) > 7:
        return False
    return True


def partner_for(rng, rest, resp, theirs, lo=6, hi=11):
    for _ in range(20000):
        p = deal13(rng, rest)
        L = lengths(p)
        if lo <= hcp(p) <= hi and L[SI[resp]] >= 4 and L[SI[theirs]] <= 3 and (
            resp in ("H", "S") or (L[0] <= 3 and L[1] <= 3)
        ) and (resp != "H" or L[0] <= L[1]):
            return p
    return None


def header(a, name, prefix, what):
    return (
        f"# {what}\n"
        f"# Generated by probes/opener-newsuit-contested_gen.py (seed {a.seed}); Rick 2026-11-07, \"yes to 1 and 2\".\n"
        f'card    = "{a.card}"\ndealer  = "S"\nprefix  = "{prefix}"\n'
        'vuls    = ["None"]\nscoring = ["IMP"]\npartner = ["2.T9876.AKJ.8643"]\nhands = [\n'
    )


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=160)
    ap.add_argument("--card", default="21GF-GIB")
    ap.add_argument("--seed", type=int, default=1108)
    ap.add_argument("--outdir", default="probes")
    ap.add_argument("--responder", action="store_true")
    ap.add_argument("--strong", action="store_true", help="17-21 unbalanced openers (opnj-*)")
    a = ap.parse_args()
    rng = random.Random(a.seed)
    full = [(s, r) for s in range(4) for r in RANKS]
    if a.responder:
        responder(a, rng, full)
        return
    for name, prefix, opened, resp, theirs, new in (STRONG if a.strong else POSITIONS):
        rows = []
        while len(rows) < a.count:
            want = rng.randint(17, 21) if a.strong else rng.randint(11, 20)
            c = deal13(rng, full)
            if hcp(c) != want or not opener_ok(c, opened, resp, theirs, new, a.strong):
                continue
            p = partner_for(rng, [x for x in full if x not in c], resp, theirs)
            if p is None:
                continue
            L = lengths(c)
            st = int(stop(c, SI[theirs]))
            rows.append(f'  "h{hcp(c)} st{st} z{L[SI[new]]} o{L[SI[opened]]} | {pbn(c)} | {pbn(p)}",')
        with open(f"{a.outdir}/{name}.toml", "w") as f:
            f.write(
                header(a, name, prefix,
                       f"Opener after {prefix} with {new} (their suit {theirs}); label h<HCP> st<stopper> z<len {new}> o<len {opened}>.")
                + "\n".join(rows) + "\n]\n"
            )


def responder(a, rng, full):
    for name, prefix, mine, theirs, opener_where, (lo, hi) in RESP:
        rows = []
        while len(rows) < a.count:
            want = rng.randint(lo, hi)
            c = deal13(rng, full)
            L = lengths(c)
            if hcp(c) != want or L[SI[mine]] < 4 or L[SI[theirs]] > 4 or max(L) > 6:
                continue
            if mine == "D" and (L[0] >= L[2] and L[0] >= 4 or L[1] >= L[2] and L[1] >= 4):
                continue  # with a longer or equal major he would have bid it
            rest = [x for x in full if x not in c]
            for _ in range(20000):
                p = deal13(rng, rest)
                Lp = lengths(p)
                if Lp[0] <= 4 and Lp[1] <= 4 and eval(
                    opener_where, {}, {"L": Lp, "h": hcp(p), "bal": balanced(Lp)}
                ):
                    break
            else:
                continue
            st = int(stop(c, SI[theirs]))
            rows.append(f'  "h{hcp(c)} st{st} z{L[0]} o{L[SI[mine]]} | {pbn(c)} | {pbn(p)}",')
        with open(f"{a.outdir}/{name}.toml", "w") as f:
            f.write(
                header(a, name, prefix,
                       f"Responder after {prefix} (their suit {theirs}); label h<HCP> st<stopper> z<spades> o<len {mine}>.\n"
                       f"# Opener: {opener_where}.")
                + "\n".join(rows) + "\n]\n"
            )


if __name__ == "__main__":
    main()
