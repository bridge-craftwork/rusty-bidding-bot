# Our side once they come in (`after-interference.bid`): notes

Responder's call when RHO has overcalled or doubled partner's one-level
opening, and opener's answer to a negative double. Cases:
`after-interference.test`.

## Guidance (Rick, 2026-09-22)

- **Negative doubles through 3♠**, plus natural bids.
- The double shows **6+ total points with four or more cards in each
  unbid major**, so a suit bid in the only unbid major promises five.
  Where both majors have been bid (1♥-(1♠)) it shows the minors.
- **Redouble is 10+** and says we own the hand.
- Raises are 6-9 with support; over a double, four-card support with less
  than 8 jumps as a preempt.
- 1NT denies an unbid four-card major (partner would hear about it from
  the double) and denies three-card support for partner's major.

## Evidence

From `bba-cli --all-meanings` on the two Basic scenarios:

- **(1x 1y) X** — "negative double", 6-21 total points, 4-6 cards in the
  unbid major. A one-level suit bid in the only unbid major showed 5+
  (1♣-(1♥)-1♠), but only 4+ when both majors were still unbid
  (1♣-(1♦)-1♠ and 1♣-(1♦)-1♥), which is what the rules do.
- **(1x X)** — responder over a double: pass 0-5; a new suit at the one
  level 6+ with 4+; a raise "calculated bid" 7-9 with 3+; redouble
  "penalty" 10+; a jump raise of a minor "preemptive" 3-7 with five.

Effect: call-2 agreement over the two scenarios went from 49.5% to 62.8%,
and Basic_Takeout_Double from 60.4% to 71.3% of calls.

Two rules were wrong until the `.test` cases caught them: the negative
double outranked a five-card major (it now denies one), and 1NT outranked
a raise with three-card support for partner's major.

## Gaps and open questions

- **Two-level and jump overcalls**: `after 1x (2y)` has no rules, so
  responder passes over a weak jump overcall.
- The **cue-bid raise** (limit raise or better with support) and
  **jump shifts** (`competitive.jump_shift_after_overcall` is strong on
  this card).
- Opener's rebid after anything but a negative double, and after our side
  has been doubled (`competitive.jordan_2nt`, support redoubles).
- Responder's second call in a contested auction.
