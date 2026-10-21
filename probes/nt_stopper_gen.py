#!/usr/bin/env python3
"""Write the grid specs of the notrump-stopper probe (probes/nt-stopper-*.toml).

    probes/nt_stopper_gen.py [--card 21GF-DEFAULT] [--suffix ""] [--count 200]
    probes/nt_stopper_gen.py --card 21GF-GIB --suffix -gib --outdir /tmp/x

Each position names the auction before our call (South decides), the
suits the opponents have shown (naturally or by convention), the HCP
range and shape of the hands. Half the hands hold a stopper (A, Kx, Qxx,
Jxxx: `stop(x)` in eval.rs) in every suit they have shown, half miss one.
The same seed gives the same hands for every card, so the cards compare
board for board. Tally with probes/nt_stopper_tally.py.

Rick (2026-10-16): a notrump bid promises a stopper in the opponents'
suit, bid or implied; over 1C (3+) normally not, over 1D (4+) it varies.
"""
import argparse
import random

RANKS = "AKQJT98765432"
HCP = {"A": 4, "K": 3, "Q": 2, "J": 1}
SUITS = "SHDC"

BAL = "sorted(L) in ([3,3,3,4],[2,3,4,4],[2,3,3,5])"
NO5M = "L[0]<=4 and L[1]<=4"

# name, dealer, prefix, their suits, hcp range, extra condition on L (lengths
# S H D C) and h, partner hcp, partner condition, description.
POSITIONS = [
    # Our direct 1NT overcall
    ("oc1n-1C", "E", "1C", "C", (15, 18), f"{BAL} and {NO5M}", None, "Direct 1NT overcall of 1C"),
    ("oc1n-1D", "E", "1D", "D", (15, 18), f"{BAL} and {NO5M}", None, "Direct 1NT overcall of 1D"),
    ("oc1n-1H", "E", "1H", "H", (15, 18), f"{BAL} and L[0]<=4", None, "Direct 1NT overcall of 1H"),
    ("oc1n-1S", "E", "1S", "S", (15, 18), f"{BAL} and L[1]<=4", None, "Direct 1NT overcall of 1S"),
    # Balancing 1NT
    ("bal1n-1C", "E", "1C Pass Pass", "C", (11, 17), f"{BAL} and {NO5M}", None, "Balancing 1NT over 1C"),
    ("bal1n-1D", "E", "1D Pass Pass", "D", (11, 17), f"{BAL} and {NO5M}", None, "Balancing 1NT over 1D"),
    ("bal1n-1H", "E", "1H Pass Pass", "H", (11, 17), f"{BAL} and L[0]<=4", None, "Balancing 1NT over 1H"),
    ("bal1n-1S", "E", "1S Pass Pass", "S", (11, 17), f"{BAL} and L[1]<=4", None, "Balancing 1NT over 1S"),
    # Natural NT overcalls of preempts
    ("oc2n-2H", "E", "2H", "H", (15, 19), f"{BAL} and L[0]<=4", None, "2NT overcall of a weak 2H"),
    ("oc2n-2S", "E", "2S", "S", (15, 19), f"{BAL} and L[1]<=4", None, "2NT overcall of a weak 2S"),
    ("oc3n-3H", "E", "3H", "H", (16, 21), f"{BAL} and L[0]<=4", None, "3NT overcall of 3H"),
    # Responder after their overcall (partner N opened)
    ("resp-1C-1H", "N", "1C 1H", "H", (6, 15), f"{BAL} and L[0]<=3", None, "Responder after 1C (1H)"),
    ("resp-1C-1S", "N", "1C 1S", "S", (6, 15), f"{BAL} and L[1]<=3", None, "Responder after 1C (1S)"),
    ("resp-1D-1S", "N", "1D 1S", "S", (6, 15), f"{BAL} and L[1]<=3 and L[2]<=3", None, "Responder after 1D (1S)"),
    ("resp-1D-2C", "N", "1D 2C", "C", (6, 15), f"{BAL} and {NO5M} and L[0]<=3 and L[1]<=3 and L[2]<=3", None, "Responder after 1D (2C)"),
    ("resp-1H-1S", "N", "1H 1S", "S", (6, 15), f"{BAL} and L[1]<=2", None, "Responder after 1H (1S)"),
    ("resp-1H-2C", "N", "1H 2C", "C", (6, 15), f"{BAL} and L[1]<=2 and L[0]<=4", None, "Responder after 1H (2C)"),
    ("resp-1S-2D", "N", "1S 2D", "D", (6, 15), f"{BAL} and L[0]<=2 and L[1]<=4", None, "Responder after 1S (2D)"),
    # Responder after their two-suited bid (suits implied)
    ("resp-1H-2H", "N", "1H 2H", "S", (8, 16), f"{BAL} and L[1]<=2", None, "Responder after 1H (2H Michaels: spades + a minor)"),
    ("resp-1S-2S", "N", "1S 2S", "H", (8, 16), f"{BAL} and L[0]<=2", None, "Responder after 1S (2S Michaels: hearts + a minor)"),
    ("resp-1H-2N", "N", "1H 2NT", "CD", (8, 16), f"{BAL} and L[1]<=2", None, "Responder after 1H (2NT unusual: minors)"),
    ("resp-1C-2C", "N", "1C 2C", "HS", (8, 16), f"{BAL}", None, "Responder after 1C (2C Michaels: majors)"),
    ("resp-1D-2D", "N", "1D 2D", "HS", (8, 16), f"{BAL} and L[2]<=3", None, "Responder after 1D (2D Michaels: majors)"),
    # Responder over their jump overcall / preempt
    ("resp-1D-2S", "N", "1D 2S", "S", (8, 16), f"{BAL} and L[1]<=3 and L[2]<=3", None, "Responder after 1D (2S weak jump overcall)"),
    ("resp-1H-2S", "N", "1H 2S", "S", (8, 16), f"{BAL} and L[1]<=2", None, "Responder after 1H (2S weak jump overcall)"),
    ("resp-1S-3H", "N", "1S 3H", "H", (10, 17), f"{BAL} and L[0]<=2", None, "Responder after 1S (3H preempt)"),
    ("resp-1S-3H-X-3S", "N", "1S 3H X Pass 3S Pass", "H", (10, 17), f"{BAL} and L[0]<=2", None,
     "Responder after 1S (3H) X (P) 3S (P): the Brill daily board 8 pattern"),
    # Advancer after partner's overcall or double (dealer W opens, N acts)
    ("adv-1D-1S", "W", "1D 1S Pass", "D", (6, 15), f"{BAL} and L[0]<=2 and L[1]<=4", None, "Advancer after (1D) 1S (P)"),
    ("adv-1C-1H", "W", "1C 1H Pass", "C", (6, 15), f"{BAL} and L[1]<=2 and L[0]<=3", None, "Advancer after (1C) 1H (P)"),
    ("adv-1H-2C", "W", "1H 2C Pass", "H", (6, 15), f"{BAL} and L[3]<=2 and L[0]<=3", None, "Advancer after (1H) 2C (P)"),
    ("adv-1S-2H", "W", "1S 2H Pass", "S", (6, 15), f"{BAL} and L[1]<=2", None, "Advancer after (1S) 2H (P)"),
    ("adv-1D-X", "W", "1D X Pass", "D", (6, 15), f"{BAL} and L[0]<=3 and L[1]<=3", None, "Advancer after (1D) X (P)"),
    ("adv-1H-X", "W", "1H X Pass", "H", (6, 15), f"{BAL} and L[0]<=3", None, "Advancer after (1H) X (P)"),
    ("adv-1S-X", "W", "1S X Pass", "S", (6, 15), f"{BAL} and L[1]<=3", None, "Advancer after (1S) X (P)"),
    # Opener's notrump rebid after their overcall (we opened)
    ("op-1C-1S-P-P", "S", "1C 1S Pass Pass", "S", (12, 19),
     f"{BAL} and {NO5M} and L[3]>=3 and (L[3]>L[2] or L[2]==3) and not 15<=h<=17", None, "Opener after 1C (1S) P (P)"),
    ("op-1D-1H-P-P", "S", "1D 1H Pass Pass", "H", (12, 19),
     f"{BAL} and {NO5M} and L[2]>=4 and L[2]>=L[3] and not 15<=h<=17", None, "Opener after 1D (1H) P (P)"),
    ("op-1C-1H-1S-P", "S", "1C 1H 1S Pass", "H", (12, 19),
     f"{BAL} and {NO5M} and L[0]<=3 and L[3]>=3 and (L[3]>L[2] or L[2]==3) and not 15<=h<=17", None, "Opener after 1C (1H) 1S (P)"),
    ("op-1D-1H-1S-2H", "S", "1D 1H 1S 2H", "H", (12, 19),
     f"{BAL} and {NO5M} and L[0]<=3 and L[2]>=4 and L[2]>=L[3] and not 15<=h<=17", None, "Opener after 1D (1H) 1S (2H)"),
    ("op-1D-1S-X-P", "S", "1D 1S X Pass", "S", (12, 19),
     f"{BAL} and {NO5M} and L[1]<=3 and L[2]>=4 and L[2]>=L[3] and not 15<=h<=17", None, "Opener after 1D (1S) X (P), negative double"),
    ("op-1C-1S-X-P", "S", "1C 1S X Pass", "S", (12, 19),
     f"{BAL} and {NO5M} and L[1]<=3 and L[3]>=3 and (L[3]>L[2] or L[2]==3) and not 15<=h<=17", None, "Opener after 1C (1S) X (P), negative double"),
]


def deal13(rng, avoid=()):
    deck = [(s, r) for s in range(4) for r in RANKS if (s, r) not in avoid]
    return rng.sample(deck, 13)


def pbn(cards):
    return ".".join(
        "".join(sorted((r for s2, r in cards if s2 == s), key=RANKS.index)) for s in range(4))


def hcp(cards):
    return sum(HCP.get(r, 0) for _, r in cards)


def lengths(cards):
    return [sum(1 for s2, _ in cards if s2 == s) for s in range(4)]


def stop(holding):
    """A, Kx, Qxx or Jxxx: eval.rs's stop(x)."""
    n = len(holding)
    return "A" in holding or ("K" in holding and n >= 2) or ("Q" in holding and n >= 3) or ("J" in holding and n >= 4)


def holdings(cards):
    return pbn(cards).split(".")


def stops_all(cards, suits):
    hs = holdings(cards)
    return all(stop(hs[SUITS.index(s)]) for s in suits)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--card", default="21GF-DEFAULT")
    ap.add_argument("--suffix", default="")
    ap.add_argument("--outdir", default="probes")
    ap.add_argument("--count", type=int, default=200)
    ap.add_argument("--seed", type=int, default=1016)
    ap.add_argument("--only", nargs="*")
    a = ap.parse_args()
    for i, (name, dealer, prefix, theirs, (lo, hi), where, _p, desc) in enumerate(POSITIONS):
        if a.only and name not in a.only:
            continue
        rng = random.Random(a.seed * 1000 + i)
        want = {True: a.count // 2, False: a.count - a.count // 2}
        hands, seen, tries = [], set(), 0
        while (want[True] or want[False]) and tries < 20_000_000:
            tries += 1
            h = deal13(rng)
            n = hcp(h)
            if not lo <= n <= hi:
                continue
            L = lengths(h)
            if not eval(where, {}, {"L": L, "h": n}):
                continue
            s = stops_all(h, theirs)
            if not want[s]:
                continue
            p = pbn(h)
            if p in seen:
                continue
            seen.add(p)
            want[s] -= 1
            # Partner: any hand from the other 39 cards (BBA decides on its own hand).
            hands.append((p, pbn(deal13(rng, avoid=set(h)))))
        out = f"{a.outdir}/nt-stopper-{name}{a.suffix}.toml"
        with open(out, "w") as f:
            f.write(f"# {desc}. Their suit(s): {theirs}. HCP {lo}-{hi}; condition: {where}\n")
            f.write("# Generated by probes/nt_stopper_gen.py (seed "
                    f"{a.seed}): half the hands stop {theirs}, half do not. Tally: probes/nt_stopper_tally.py\n")
            f.write(f'card    = "{a.card}"\ndealer  = "{dealer}"\nprefix  = "{prefix}"\n')
            f.write('vuls    = ["None"]\nscoring = ["IMP"]\n')
            f.write('partner = ["2.T9876.AKJ.8643"]\n')
            f.write('our     = ["general.style=bba"]\n')
            f.write("hands = [\n" + "".join(f'  "{h} | {h} | {p}",\n' for h, p in hands) + "]\n")
        print(f"{out}: {len(hands)} hands")


if __name__ == "__main__":
    main()
