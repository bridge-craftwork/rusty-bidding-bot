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
- Raises are 6-9 with support over an overcall. **Over a double the raise
  starts at 4 total points**: BBA raises on 5 HCP with three trumps, and
  taking the level away from a doubler is worth it. The jump raise is
  five-card support for a major with less than 8; a minor raises to two,
  which is what BBA does with the same hands.
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

## Over a two-level or jump overcall (Rick, 2026-09-22)

- **a new suit is 10+ total points with five cards, and forcing**
  (`competitive.new_suit_after_overcall_forcing`, default on; Basic-Bridge
  turns it off, and then the same bids are made without the force);
- **the negative double is 8+ with four cards** in each unbid major;
- **the cue bid of their suit is a limit raise or better**: 11+ with
  three-card support (`competitive.cue_bid_raise.play`, default on).
  Opener accepts with 16+ and signs off in the suit otherwise.

Their bid **in our suit** is a cue bid, not an overcall (Michaels and the
like), so these rules are gated on `y is not x`. Without that they fired
over a Michaels cue bid and cost about a point of agreement in each of
the six Michaels scenarios.

Opener's answer to the negative double is written once with relative
calls (`cheapest(S)`, `jump(N)`, `cheapest(x)`), so it is right at every
level the opponents push us to. Before that it only covered a one-level
overcall, and a two-level negative double was left to die: the corpus
lost 12 points of identical auctions in McCabe_after_WJO and 11 in
Opps_Preemptive_Overcall until opener could answer.

Across the corpus this slice is worth +36.6 points of call agreement and
+75.6 points of contract agreement summed over the 90 scenarios it moved.

## Gaps and open questions

- **Jump shifts** (`competitive.jump_shift_after_overcall` is strong on
  this card) and Lebensohl over an overcall of 1m (BBA plays it on the
  21GF card).
- **Trap passes**: BBA passes with length and strength behind the
  overcaller where we double or cue. Trap_Pass loses 3.0 points of call
  agreement, and its contracts gain 2.6.
- Opener's rebid after anything but a negative double, and after our side
  has been doubled (`competitive.jordan_2nt`, support redoubles).
- Responder's second call in a contested auction.
