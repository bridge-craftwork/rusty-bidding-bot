# Takeout doubles (`takeout-double.bid`): notes

The direct double of a one-level opening, advancer's replies, and the
doubler's rebid. Cases: `takeout-double.test`.

## Guidance (Rick, 2026-09-22)

- **12 total points with three or more cards in every unbid suit.**
- **18+ total points doubles whatever the shape**, including with a long
  strong suit: too strong to overcall is double first, then bid the suit.
  That is why the double's second branch starts one point above the
  overcall maximum (`overcalls.one_level_max` + 1).
- **Advancer answers on the standard ladder**: the cheapest suit is 0-8,
  a jump 9-11, and the cue bid of their suit 12+ and forcing to game.
  (BBA's own ladder is 0-10 / 7-12 / 10+, which we did not copy.)
- Advancer bids his longest suit, majors first, and the cheaper of two
  equal suits.
- **The doubler passes a minimum** and shows his extra values afterwards:
  18-19 raises one level, 20-21 two, 22+ bids the major game, and a
  five-card suit of his own is bid at the cheapest level.

## Evidence

From `bba-cli --all-meanings` on the two Basic scenarios:

- **(1x) X** — "takeout double", 12-37 total points, 3+ in each unbid
  suit and at most four of theirs. The hands were 10-18 HCP, median 13,
  and typically 4-4-3 or 4-3-3 in the unbid suits.
- **(1x) X (P)** — advancer: a minimum suit bid 0-10 total points with
  4+; a jump 7-12; the cue bid "strength cue bid", 10+.

The double is what BBA does with most 12-17 balanced-ish hands short in
their suit: it is the single largest divergence in these two scenarios
(424 of the 1,000 boards diverged at call 2 with BBA doubling and us
passing). Adding it took call-2 agreement from 46.6% to 87.5%.

## Accepted differences from BBA

- Our ladder starts the jump at 9 and the cue bid at 12; BBA's bands
  overlap (7-12 for a jump, 10+ for the cue bid), so boundary hands
  differ.
- BBA doubles some 10-11 HCP hands with shape that we pass, and passes
  some 12-counts with a five-card suit that we double.
- We require three cards in every unbid suit; BBA's meaning allows a
  doubleton in an unbid minor on stronger hands.

Advancer when RHO does not pass is written too: a free bid needs 6+ at
the one level and 8+ higher, the cue bid is still 12+, and over a
redouble advancer names his cheapest four-card suit however weak. That
alone took Basic_Takeout_Double from 71.3% to 72.5% of calls and its
contracts from 5.6% to 7.4%.

## Gaps and open questions

- **Responsive doubles** are off on this card (`doubles.responsive.play`),
  so a double by advancer over their raise has no rule.
- The **penalty pass** of a takeout double with length in their suit.
- Doubles of anything but a one-level suit opening: weak twos, preempts,
  1NT, and the **balancing double**.
- Advancer's rebid after the cue bid (the cue sets `forcing=game`, so the
  base rule bids game in the agreed suit or 3NT).
