#!/usr/bin/env python3
"""Write the grid specs of the "opener's notrump once they come in" probe
(probes/opnt-*.toml; bridge-classroom-bug-artifacts #93, Rick 2026-11-07:
"we can follow BBA's lead").

    probes/opener-nt-contested_gen.py [--count 160] [--card 21GF-GIB]

South opened one of a minor, partner responded (or passed) and they came
in. South's hands are balanced (4-3-3-3, 4-4-3-2, 5-3-3-2 with five in the
opened minor), 12-21 HCP, consistent with the opening (1C: clubs at least
as long as diamonds and not 4-4; 1D: four diamonds, or three with two
clubs), at most three of partner's major. The label carries what the
decision may turn on: `h<HCP> st<0|1> z<len in partner's suit>`
(st: a stopper in their suit, A, Kx, Qxx or Jxxx as `stop(x)` in eval.rs).
Each hand gets its own partner: 6-11 HCP with four or more of the
response suit (for the reopening positions 0-5 with nothing to say).
Tally with probes/grid_tally.py or the one-liner in rebids.notes.md.
"""
import argparse
import random

RANKS = "AKQJT98765432"
HCP = {"A": 4, "K": 3, "Q": 2, "J": 1}
SI = {"S": 0, "H": 1, "D": 2, "C": 3}

# name, prefix, opened minor, partner's suit (None: partner passed), their suit
POSITIONS = [
    ("opnt-1D-1H-2S", "1D Pass 1H 2S", "D", "H", "S"),   # #93: weak jump overcall
    ("opnt-1C-1H-2S", "1C Pass 1H 2S", "C", "H", "S"),
    ("opnt-1D-1S-2H", "1D Pass 1S 2H", "D", "S", "H"),   # two-level, support X applies
    ("opnt-1C-1H-2D", "1C Pass 1H 2D", "C", "H", "D"),
    ("opnt-1D-1H-2C", "1D Pass 1H 2C", "D", "H", "C"),
    ("opnt-1C-1H-1S", "1C Pass 1H 1S", "C", "H", "S"),   # one-level
    ("opnt-1D-1S-2C", "1D Pass 1S 2C", "D", "S", "C"),
    ("opnt-1C-1D-1S", "1C Pass 1D 1S", "C", "D", "S"),   # minor response
    ("opnt-1D-P-P-1H", "1D Pass Pass 1H", "D", None, "H"),  # reopening (#90)
    ("opnt-1C-P-P-2H", "1C Pass Pass 2H", "C", None, "H"),
    ("opnt-1C-1D-2H", "1C Pass 1D 2H", "C", "D", "H"),   # jump over a minor response
    ("opnt-1C-1D-1H", "1C Pass 1D 1H", "C", "D", "H"),   # minor response, the double (Rick 2026-11-07)
    ("opnt-1C-1D-2S", "1C Pass 1D 2S", "C", "D", "S"),
]


# Responder's next call: name, prefix, responder's suit, their suit,
# opener's hand test (lengths L spades first, HCP h), responder's HCP.
RESP = [
    ("opnt-r-1D-1H-2S-X", "1D Pass 1H 2S X Pass", "H", "S",
     "15 <= h <= 18 and L[1] <= 3 and L[0] <= 3 and L[2] >= 4", (5, 12)),
    ("opnt-r-1C-1H-2S-X", "1C Pass 1H 2S X Pass", "H", "S",
     "15 <= h <= 18 and L[1] <= 3 and L[0] <= 3 and L[3] >= 4", (5, 12)),
    ("opnt-r-1D-1H-2S-2N", "1D Pass 1H 2S 2NT Pass", "H", "S",
     "17 <= h <= 18 and L[1] <= 3 and L[0] == 4 and L[2] >= 4", (5, 12)),
    ("opnt-r-1D-1S-2H-2N", "1D Pass 1S 2H 2NT Pass", "S", "H",
     "17 <= h <= 18 and L[0] <= 2 and L[1] >= 3 and L[2] >= 4", (5, 12)),
    ("opnt-r-1C-1H-1S-1N", "1C Pass 1H 1S 1NT Pass", "H", "S",
     "13 <= h <= 16 and L[1] <= 2 and L[0] >= 3 and L[3] >= 4", (5, 14)),
    ("opnt-r-1C-1D-1S-1N", "1C Pass 1D 1S 1NT Pass", "D", "S",
     "13 <= h <= 16 and L[0] >= 3 and L[3] >= 4 and L[1] <= 3", (5, 14)),
    # Over opener's double of their overcall of 1C-1D (Rick 2026-11-07)
    ("opnt-r-1C-1D-1S-X", "1C Pass 1D 1S X Pass", "D", "S",
     "14 <= h <= 18 and L[1] >= 3 and L[0] <= 3 and L[3] >= 3 and L[2] <= 3", (5, 12)),
    ("opnt-r-1C-1D-2H-X", "1C Pass 1D 2H X Pass", "D", "H",
     "15 <= h <= 18 and L[0] >= 3 and L[1] <= 3 and L[3] >= 3 and L[2] <= 3", (5, 12)),
]

# Opener's third call after his double of 1C P 1D (overcall) and
# responder's answer (Rick 2026-11-07, "yes to 1 and 2"): name, prefix,
# the other major M, their suit, responder's hand test (L spades first,
# HCP h, st: a stopper in their suit). Opener: balanced, 14 or 18 HCP
# (15-17 opened 1NT), three or four M, at most three diamonds, a 1C opening.
THIRD = [
    ("opnt-o-1C-1D-1S-X-2C", "1C Pass 1D 1S X Pass 2C Pass", "H", "S",
     "4 <= h <= 9 and L[1] <= 3 and L[2] <= 4 and not (st and 8 <= h)"),
    ("opnt-o-1C-1D-1S-X-2D", "1C Pass 1D 1S X Pass 2D Pass", "H", "S",
     "4 <= h <= 9 and L[1] <= 3 and L[2] >= 5"),
    ("opnt-o-1C-1D-1S-X-2H", "1C Pass 1D 1S X Pass 2H Pass", "H", "S",
     "4 <= h <= 11 and L[1] >= 4"),
    ("opnt-o-1C-1D-1S-X-2N", "1C Pass 1D 1S X Pass 2NT Pass", "H", "S",
     "8 <= h <= 9 and L[1] <= 3 and st"),
    ("opnt-o-1C-1D-1H-X-1S", "1C Pass 1D 1H X Pass 1S Pass", "S", "H",
     "4 <= h <= 11 and L[0] >= 4"),
    ("opnt-o-1C-1D-1H-X-2C", "1C Pass 1D 1H X Pass 2C Pass", "S", "H",
     "4 <= h <= 9 and L[0] <= 3 and L[2] <= 4 and not (st and 8 <= h)"),
    ("opnt-o-1C-1D-2H-X-3C", "1C Pass 1D 2H X Pass 3C Pass", "S", "H",
     "4 <= h <= 9 and L[0] <= 3 and L[2] <= 4 and not (st and 8 <= h)"),
    ("opnt-o-1C-1D-2H-X-2S", "1C Pass 1D 2H X Pass 2S Pass", "S", "H",
     "4 <= h <= 11 and L[0] >= 4"),
]


def third(a, rng, full):
    for name, prefix, M, theirs, resp_where in THIRD:
        rows = []
        while len(rows) < a.count:
            want = rng.choice([14, 18])
            c = deal13(rng, full)
            L = lengths(c)
            if hcp(c) != want or sorted(L) not in ([3, 3, 3, 4], [2, 3, 4, 4], [2, 3, 3, 5]):
                continue
            if not (3 <= L[SI[M]] <= 4 and L[SI[theirs]] <= 3 and L[2] <= 3 and L[3] >= 3):
                continue
            rest = [x for x in full if x not in c]
            for _ in range(20000):
                p = deal13(rng, rest)
                Lp = lengths(p)
                if Lp[2] >= 4 and max(Lp) <= 6 and Lp[SI[theirs]] <= 3 and eval(
                    resp_where, {}, {"L": Lp, "h": hcp(p), "st": stop(p, SI[theirs])}
                ):
                    break
            else:
                continue
            st = int(stop(c, SI[theirs]))
            rows.append(f'  "h{hcp(c)} st{st} z{L[SI[M]]} | {pbn(c)} | {pbn(p)}",')
        with open(f"{a.outdir}/{name}.toml", "w") as f:
            f.write(
                f"# Opener's third call after {prefix} (their suit {theirs}); label h<HCP> st<stopper> z<len {M}>.\n"
                f"# Responder: {resp_where}.\n"
                f"# Generated by probes/opener-nt-contested_gen.py --third (seed {a.seed}); Rick 2026-11-07, \"yes to 1 and 2\".\n"
                f'card    = "{a.card}"\ndealer  = "S"\nprefix  = "{prefix}"\n'
                'vuls    = ["None"]\nscoring = ["IMP"]\npartner = ["2.T9876.AKJ.8643"]\nhands = [\n'
                + "\n".join(rows)
                + "\n]\n"
            )


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


def stop(cards, s):
    rs = [r for s2, r in cards if s2 == s]
    n = len(rs)
    return "A" in rs or ("K" in rs and n >= 2) or ("Q" in rs and n >= 3) or ("J" in rs and n >= 4)


def opener_ok(c, opened, resp):
    L = lengths(c)
    S = sorted(L)
    if S not in ([3, 3, 3, 4], [2, 3, 4, 4], [2, 3, 3, 5]):
        return False
    if L[0] >= 5 or L[1] >= 5:
        return False
    if opened == "C" and not (L[3] >= 3 and L[3] >= L[2] and not (L[3] == 4 and L[2] == 4)):
        return False
    if opened == "D" and not (L[2] >= 4 and L[2] >= L[3] or (L[2] == 3 and L[3] <= 2)):
        return False
    if resp in ("H", "S") and L[SI[resp]] >= 4:
        return False
    return True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=160)
    ap.add_argument("--card", default="21GF-GIB")
    ap.add_argument("--seed", type=int, default=1107)
    ap.add_argument("--outdir", default="probes")
    ap.add_argument("--responder", action="store_true", help="write the responder specs (opnt-r-*)")
    ap.add_argument("--third", action="store_true", help="write opener's third-call specs (opnt-o-*)")
    a = ap.parse_args()
    rng = random.Random(a.seed)
    full = [(s, r) for s in range(4) for r in RANKS]
    if a.third:
        third(a, rng, full)
        return
    if a.responder:
        responder(a, rng, full)
        return
    for name, prefix, opened, resp, theirs in POSITIONS:
        rows = []
        while len(rows) < a.count:
            want = rng.randint(12, 21)
            c = deal13(rng, full)
            if hcp(c) != want or not opener_ok(c, opened, resp):
                continue
            rest = [x for x in full if x not in c]
            for _ in range(20000):
                p = deal13(rng, rest)
                L = lengths(p)
                if resp is None:
                    if hcp(p) <= 5 and max(L) <= 5:
                        break
                elif 6 <= hcp(p) <= 11 and L[SI[resp]] >= 4 and L[SI[theirs]] <= 3 and (
                    resp in ("H", "S") or (L[0] <= 3 and L[1] <= 3)
                ) and (resp != "H" or L[0] <= L[1]):
                    break
            else:
                continue
            z = lengths(c)[SI[resp]] if resp else 0
            st = int(stop(c, SI[theirs]))
            rows.append(f'  "h{hcp(c)} st{st} z{z} | {pbn(c)} | {pbn(p)}",')
        with open(f"{a.outdir}/{name}.toml", "w") as f:
            f.write(
                f"# Opener after {prefix} (their suit {theirs}); balanced 12-21, label h<HCP> st<stopper> z<partner's suit>.\n"
                f"# Generated by probes/opener-nt-contested_gen.py (seed {a.seed}); Rick 2026-11-07, #93.\n"
                f'card    = "{a.card}"\ndealer  = "S"\nprefix  = "{prefix}"\n'
                'vuls    = ["None"]\nscoring = ["IMP"]\npartner = ["2.T9876.AKJ.8643"]\nhands = [\n'
                + "\n".join(rows)
                + "\n]\n"
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
            relaxed = name.startswith("opnt-r-1C-1D-") and name.endswith("-X")
            if relaxed and mine == "D" and (L[0] >= L[2] or L[1] >= L[2]):
                continue
            if not relaxed and (mine == "H" and L[0] > L[1] or mine == "D" and (L[0] >= 4 or L[1] >= 4)):
                continue
            rest = [x for x in full if x not in c]
            for _ in range(20000):
                p = deal13(rng, rest)
                Lp = lengths(p)
                if sorted(Lp) in ([3, 3, 3, 4], [2, 3, 4, 4], [2, 3, 3, 5]) and eval(
                    opener_where, {}, {"L": Lp, "h": hcp(p)}
                ) and stop(p, SI[theirs]):
                    break
            else:
                continue
            st = int(stop(c, SI[theirs]))
            rows.append(f'  "h{hcp(c)} st{st} z{L[SI[mine]]} | {pbn(c)} | {pbn(p)}",')
        with open(f"{a.outdir}/{name}.toml", "w") as f:
            f.write(
                f"# Responder after {prefix} (their suit {theirs}); label h<HCP> st<stopper> z<length in my suit>.\n"
                f"# Opener: {opener_where}, their suit stopped.\n"
                f"# Generated by probes/opener-nt-contested_gen.py --responder (seed {a.seed}); Rick 2026-11-07, #93.\n"
                f'card    = "{a.card}"\ndealer  = "S"\nprefix  = "{prefix}"\n'
                'vuls    = ["None"]\nscoring = ["IMP"]\npartner = ["2.T9876.AKJ.8643"]\nhands = [\n'
                + "\n".join(rows)
                + "\n]\n"
            )


if __name__ == "__main__":
    main()
