# Snapdragon doubles (`snapdragon.bid`)

Switched on by `competitive.snapdragon.play`. The PBS scenario
(Snapdragon_Double) was bid on 21GF-SPECIALS, which has `Snapdragon
Double = 0` although the scenario asks for them. Cases:
`snapdragon.test`.

## What it plays (2026-10-05)

(1x) 1y (1z) X, also (1x) 1y (2z), (1x) 2y (2z), (1x) 2y (3z), three
different suits: five or more of the fourth suit, two or more of
partner's, 8+ HCP; forcing for one round. The overcaller bids the fourth
suit with three (a jump with 15+ total points), rebids a six-card suit,
bids notrump with RHO's suit stopped, else returns to his suit.

There was no double for advancer in these auctions before (no rule),
so nothing else changes.

## Deviations and open questions

- Bridge Bum treats it as constructive but not forcing; the scenario
  says forcing. We follow the scenario.
- With three-card support and 8-11 the raise and the double both fit;
  ranking picks the double when it is more descriptive.

## Compare (2026-10-05, `--set competitive.snapdragon.play=true`)

Snapdragon_Double, 50 boards, 71.7% call agreement. At the double: BBA
(card off) passes or raises partner with three where we double; BBA
style, not a bug. No forced-pass or contradiction problems.

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only snapdragon`.
IMPs by the errors yardstick, positive when the convention makes fewer;
"actor" is the side that made the first differing call.
Snapdragon_Double: 336 boards changed. First -435 to the actor (z -4.9):
opener's redouble of the Snapdragon double was left to play (1♥XX making
with overtricks, about 30 boards), the overcaller had no rule over XX.
Fixed (the overcaller's context takes the redouble): actor contract
-203, doubling +148; other side contract +230, doubling -257;
double-dummy -23; halves -29/-26 (z -1.0): neutral. What is left is the
overcaller forced to answer: 1NT without a fit on 8-11 opposite a weak
advancer goes down where the natural auction passed.

## Sources

- Bridge Bum, "Snapdragon Double" (https://www.bridgebum.com/snapdragon_double.php).
- PBS `btn/Snapdragon_Double.btn`: 5+ cards, 8+ HCP, tolerance 2-3,
  forcing.
- "25 More Bridge Conventions You Should Know", ch. 24; Todd, Advancing
  in Bridge 568 (cited by convention-card; not read).
