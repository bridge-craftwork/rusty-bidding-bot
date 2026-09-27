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

- **Responsive doubles** are in `responsive-doubles.bid`, switched on by
  `doubles.responsive.play` — the 21GF cards play them, Basic-Bridge does
  not, so on this card a double by advancer over their raise still has no
  rule.
- The **penalty pass** of a takeout double with length in their suit.
- Doubles of anything but a one-level suit opening: weak twos, preempts,
  1NT, and the **balancing double**.
- Advancer's rebid after the cue bid (the cue sets `forcing=game`, so the
  base rule bids game in the agreed suit or 3NT).

## Advancer, probed (2026-09-25, `probes/advance-1S-X.toml`)

2,000 random advancer hands after 1S X P (Basic-Bridge). Agreement
before: 45%; after: 55%. Corpus: -162,699 -> -162,394 (+305).
- **3NT with 12+ and their suit stopped** (no four-card unbid major),
  where we cue-bid. Worth +72 alone. **For Rick:** this overrides "the
  cue bid is 12+" for hands with a stopper. Test 19 now expects 3NT.
- **No four-card suit to name** (four of their suit, say): BBA bids the
  cheapest three-card suit. We had no call.
- **1NT is 7-10 HCP** with a stopper. Total points put 5-3-3-2 hands
  with nine or ten out of range, leaving them no call.
- **Jumps:** BBA does not jump with four cards and 9 HCP. It jumps with
  five from about 8, or four with 10-11. Four cards and 9 now bid at the
  cheapest level.

Not done: BBA's penalty pass with long strong trumps; the rest of the
disagreement is mostly the level with 9-11 (2S cue vs 3H...).
- **Their suit is not an unbid major** (2026-09-25). The advances in a
  minor said S<=3, H<=3, meant to prefer an unbid major. That left four
  of their suit (spades over 1S) with no call: 177 corpus boards. The
  guards now read `S<=3 | x is S`. The heart advance says the same about
  spades. Corpus: -162,394 -> -162,157.

## The double itself, probed over 1S (2026-09-25, `probes/tod-1S.toml`)

2,500 hands with three or fewer spades and 9-19 HCP over 1S: 80%
agreement. BBA's threshold is HCP by shortness. With a singleton in
their suit it doubles from 10-11 HCP (and prefers the double to a
two-level minor overcall at 11-14). With a doubleton it doubles from
12 HCP; 11 with a five-card suit passes, where our total points double.
Tried against par:

| Change | IMPs |
|---|---|
| 12 HCP, or 10 with a singleton | -314 |
| Our 12 total points, plus 10 HCP with a singleton | -380 |
| Two-level overcalls on three of the top five honours instead of four | -285 |

Ours stays in all three.

## The power double in HCP (2026-09-27)

Step a of the Law of Total Tricks work (Basic_Takeout_Double 193). The
double's second meaning was "18+ total points, any shape". Length
points can add nine to an unknown hand, so no HCP limit could ever rule
the power hand out, and the double never collapsed to "three or more
in the unbid suits". BBA (`probes/tools/takeout_shape.py` and a scan
of 913 auctions where the doubler, free to pass, then bids a new suit
or notrump): its power double steps up at 17 **HCP** (12-14: 103
hands, the competitive minimum; 15: 9; 16: 5; 17: 129; 18: 268), and
counted as HCP plus length the same hands show no step. Rick: treat it
as a likelihood, accepting the rare distributional power double below
17 that the inference will misread.

| power branch | full corpus (par / side) | Basic_* competitive |
|---|---|---|
| 18+ HCP, overcall to 17 | -937 / -2,176 | -3 / -26 |
| 17+ HCP, overcall to 17 (overlap) | +13 / -171 | -11 / -11 |
| 17+ HCP, overcall to 16 | +1 / +154 | -11 / -11 |

Kept: 17+ HCP with the overcall to 16 (`one_level_max` default 16),
neutral on its own; its value is the inferences it allows. **For
Rick:** this lowers the overcall ceiling from 17 to 16 HCP; the
fallback is the overlap version (overcall to 17), -171 by side.
