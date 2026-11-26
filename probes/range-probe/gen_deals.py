#!/usr/bin/env python3
"""Deals for the range probe: one deciding hand varied in HCP and shape,
the other hands random but legal.

    probes/range-probe/gen_deals.py [--per 4] [--seed 11]

Positions (dealer North, nobody vulnerable, IMPs):

- `resp1S`: 1S-(P)-?, South responds. North is one of four fixed 1S
  openers (cycled per deal); South varies.
- `resp1H`: 1H-(P)-?, the same with hearts.
- `reb1D1S`: 1D-(P)-1S-(P)-?, North rebids. South is one of three fixed
  1S responders; North varies.

For every position the deciding hand is drawn per (shape class, HCP),
`--per` hands each, from the 39 cards the fixed hand leaves. The seat
after opener (East) and, for the rebid, West too, are drawn as quiet
hands (no suit longer than four, few HCP) so that a robot bidding the
whole deal (WBridge5) reaches the position; robots asked one call with
the auction forced (BBA, Brill, us) never look at them.

Writes, next to this script, `data/<position>.jsonl` (one line a deal:
id, class, the decider's HCP and lengths, the deal) and
`data/<position>.pbn` (for WBridge5 and `rbb bid-pbn`), and the grid
spec `data/<position>.toml` for BBA through `rbb grid`.
"""
import argparse
import json
import os
import random

RANKS = "AKQJT98765432"
HCPV = {"A": 4, "K": 3, "Q": 2, "J": 1}
HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")

# Fixed hands, S.H.D.C. Each opens the suit in every 2/1 or SAYC system.
OPENERS_1S = ["AKJ74.K83.Q62.94", "KQT85.A4.KJ7.632", "AQ976.K5.Q84.J73", "KJ953.AQ6.T8.KJ4"]
OPENERS_1H = ["K83.AKJ74.Q62.94", "A4.KQT85.KJ7.632", "K5.AQ976.Q84.J73", "AQ6.KJ953.T8.KJ4"]
# 1S responders to 1D: a clear 1S in every system (5 spades, 8-10 HCP).
RESP_1D_1S = ["KQ852.Q73.84.J62", "AJ963.T73.5.Q974", "KJ752.K842.J3.84"]

# Responder classes after 1M: (name, test on lengths [S,H,D,C]) for each major.
def resp_classes(major):
    m, o = (0, 1) if major == "S" else (1, 0)  # opened major, other major
    return [
        ("nofit",      lambda L: L[m] <= 2 and L[o] <= 3 and max(L[2], L[3]) <= 5),
        ("longminor",  lambda L: L[m] <= 2 and L[o] <= 3 and max(L[2], L[3]) >= 6),
        ("4other",     lambda L: L[m] <= 2 and L[o] >= 4),
        ("3support",   lambda L: L[m] == 3),
        ("4support",   lambda L: L[m] == 4),
    ]

# Opener classes after 1D-1S (lengths [S,H,D,C]).
def balanced(L):
    s = sorted(L)
    return s in ([3, 3, 3, 4], [2, 3, 4, 4], [2, 3, 3, 5])

REB_CLASSES = [
    ("bal",       lambda L, h: balanced(L) and L[0] <= 3 and L[1] <= 4 and L[2] >= 4 and L[2] >= L[3]
                  and L[1] < 5 and (h <= 14 or 18 <= h <= 19)),
    ("4support",  lambda L, h: L[0] == 4 and L[2] >= 4 and L[2] >= L[3] and L[1] <= 3
                  and not (balanced(L) and (15 <= h <= 17 or h >= 20))),
    ("6diamonds", lambda L, h: L[2] >= 6 and L[0] <= 2 and L[1] <= 3 and L[3] <= 3),
    ("5D4C",      lambda L, h: L[2] >= 5 and L[3] == 4 and L[0] <= 3 and L[1] <= 3),
    ("5D4H",      lambda L, h: L[2] >= 5 and L[1] == 4 and L[0] <= 3),
]

POSITIONS = {
    "resp1S":  dict(fixed=OPENERS_1S, prefix="1S Pass", decider="S", hcp=range(3, 15),
                    classes=[(n, (lambda t: lambda L, h: t(L))(t)) for n, t in resp_classes("S")]),
    "resp1H":  dict(fixed=OPENERS_1H, prefix="1H Pass", decider="S", hcp=range(3, 15),
                    classes=[(n, (lambda t: lambda L, h: t(L))(t)) for n, t in resp_classes("H")]),
    "reb1D1S": dict(fixed=RESP_1D_1S, prefix="1D Pass 1S Pass", decider="N", hcp=range(11, 22),
                    classes=REB_CLASSES),
}


def cards_of(hand):
    return {(s, r) for s, suit in enumerate(hand.split(".")) for r in suit}


def pbn(cards):
    return ".".join("".join(sorted((r for s2, r in cards if s2 == s), key=RANKS.index)) for s in range(4))


def hcp(cards):
    return sum(HCPV.get(r, 0) for _, r in cards)


def lengths(cards):
    return [sum(1 for s2, _ in cards if s2 == s) for s in range(4)]


def quiet(cards, top):
    return max(lengths(cards)) <= 4 and hcp(cards) <= top


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--per", type=int, default=4, help="hands per (class, HCP)")
    ap.add_argument("--seed", type=int, default=11)
    a = ap.parse_args()
    rng = random.Random(a.seed)
    os.makedirs(DATA, exist_ok=True)
    deck = [(s, r) for s in range(4) for r in RANKS]
    for name, P in POSITIONS.items():
        rows, k = [], 0
        for cls, test in P["classes"]:
            for h in P["hcp"]:
                if cls == "bal" and name == "reb1D1S" and (15 <= h <= 17 or h >= 20):
                    continue  # those open 1NT / 2NT
                for _ in range(a.per):
                    fixed = P["fixed"][k % len(P["fixed"])]
                    k += 1
                    rest = [c for c in deck if c not in cards_of(fixed)]
                    for _try in range(2_000_000):
                        d = rng.sample(rest, 13)
                        if hcp(d) == h and test(lengths(d), h):
                            break
                    else:
                        raise SystemExit(f"{name} {cls} {h}: no hand")
                    left = [c for c in rest if c not in set(d)]
                    # East quiet (and West, when West calls before the decision).
                    for top in (8, 9, 10, 11, 12, 40):
                        for _try in range(20000):
                            e = rng.sample(left, 13)
                            w = [c for c in left if c not in set(e)]
                            if quiet(e, top) and (P["decider"] == "S" or quiet(w, top)):
                                break
                        else:
                            continue
                        break
                    hands = {P["decider"]: pbn(d), "E": pbn(e), "W": pbn(w)}
                    hands["N" if P["decider"] == "S" else "S"] = fixed
                    deal = "N:" + " ".join(hands[s] for s in "NESW")
                    L = lengths(d)
                    rows.append({"id": f"{name}-{len(rows) + 1:03d}", "position": name, "class": cls,
                                 "hcp": h, "lengths": L, "hand": pbn(d), "fixed": fixed,
                                 "decider": P["decider"], "prefix": P["prefix"], "deal": deal})
        with open(os.path.join(DATA, f"{name}.jsonl"), "w") as f:
            for r in rows:
                f.write(json.dumps(r) + "\n")
        with open(os.path.join(DATA, f"{name}.pbn"), "w") as f:
            f.write("% PBN 2.1\n% range-probe: " + name + "\n\n")
            for i, r in enumerate(rows, 1):
                f.write(f'[Event "range-probe {name}"]\n[Board "{i}"]\n[Dealer "N"]\n'
                        f'[Vulnerable "None"]\n[Deal "{r["deal"]}"]\n[Scoring "IMP"]\n\n')
        q = lambda xs: "[" + ", ".join(f'"{x}"' for x in xs) + "]"
        with open(os.path.join(DATA, f"{name}.toml"), "w") as f:
            f.write(f"# range-probe {name}: generated by probes/range-probe/gen_deals.py "
                    f"(seed {a.seed}, {a.per} per class and HCP)\n")
            f.write(f'card    = "21GF-GIB"\ndealer  = "N"\nprefix  = "{P["prefix"]}"\n')
            f.write('vuls    = ["None"]\nscoring = ["IMP"]\nour     = []\n')
            f.write(f"partner = {q(P['fixed'])}\n")
            f.write("hands = [\n" + "".join(
                f'  "{r["id"]} | {r["hand"]} | {r["fixed"]}",\n' for r in rows) + "]\n")
        print(f"{name}: {len(rows)} deals")


if __name__ == "__main__":
    main()
