# Lead-directing doubles (`lead-directing-doubles.bid`)

Switched on by `competitive.lead_directing_double.play` (no stock card;
PBS Lead_Directing_Double is bid on 21GF-GIB, and its notes say BBA
does not make them). Cases: `lead-directing-doubles.test`.

## What it plays (2026-10-05)

A double of RHO's artificial bid, when our side has not bid: (1N) P
(2x), (1N) P (3x), (2N) P (3x), (4N) P (5x), and (2C) P (2D).
"Artificial" is the engine's own reading: RHO has not bid the suit
naturally (`!rho.named(x)`), so Stayman, transfers, relays and keycard
answers qualify and a natural weak takeout does not. The hand: five
cards with three of the top five honours, or six with two of the top
three (the scenario's standard).

## Deviations and open questions

- Not written: Lightner (the double of a slam), doubles of their cue
  bids and control bids, and doubles of artificial bids elsewhere.
- Their side after our double (Opps_Double_Stayman, Opps_Double_Jacoby)
  is the notrump modules' business.
- The compare cannot show the gain: the engine does not choose leads.

## Compare (2026-10-05, `--set competitive.lead_directing_double.play=true`)

Lead_Directing_Double, 50 boards, 82.2% agreement. Our doubles of
Stayman (5) and of a transfer (3) where BBA passes: the convention. The
cost is on their side: after 1NT P 2♣ X our opener passes where BBA
bids 2♥ (4 boards, -22 IMPs); opener's answers to Stayman over a double
belong to the Stayman module (notrump/), not changed here.

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only
lead-directing-doubles`. IMPs by the errors yardstick, positive when the
convention makes fewer; "actor" is the side that made the first
differing call. Lead_Directing_Double: 196 boards changed; actor
contract -22, doubling +19; other side contract -214 (their
Stayman/transfer auctions after our double, the opener's pass noted
above); double-dummy +261 to the actor; halves -2/-1: neutral by the
actor's own errors, a gain by the double-dummy and by the other side's
errors.

## Sources

- Bridge Bum, "Lead Directing Double" (https://www.bridgebum.com/lead_directing_double.php).
- PBS `btn/Lead_Directing_Double.btn`: the suit-quality standard.
- "25 Bridge Conventions You Should Know", ch. 24; Todd, Advancing in
  Bridge 467 (cited by convention-card; not read).
