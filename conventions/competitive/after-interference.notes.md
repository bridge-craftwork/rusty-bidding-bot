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

## Opener's answer to the negative double, completed (2026-09-23)

The block only bid majors, notrump with a stopper, and a six-card suit
of its own, so 1♥-(1♠)-X — where the double shows the **minors** — and
every minimum with no major and no stopper died on the spot: `1x 1y X P`
was 665 boards of "no rule in a live auction" across the corpus and the
largest second-call gap in the module. Opener now has, below the
four-card major and the notrump rebid and in this order:

1. an **unbid four-card minor** (`cheapest(D)` with four, `cheapest(C)`
   with four and at least as many as diamonds, so 4-4 bids the cheaper);
2. **three cards in the major partner promised** — an eight-card fit,
   which is what BBA does (`1D (1S) X (P) 2H` on KQ6.832.KT84.A64);
3. **five cards in his own suit** (the old rule needed six);
4. `cheapest(x)` with nothing shown at priority -5, so the forcing
   double always gets an answer.

Two more holes at the top of the range: with **19+** and a four-card
major opener bids the game (`1C (1S) X (P) 4H` on A63.AT92.A.AQJ43), and
the jump in a major stops at 18 as before.

Effect on the corpus: `1x 1y X P` gained 255 calls, and the whole
module's slice took no-rule from 165,221 to 156,762 boards.

## Opener when they compete over the negative double (2026-09-23)

`1x (1y) X (2y)` was the second-largest second-call gap (225 boards
across the corpus). They have bid again, so the double is no longer
forcing and **pass is BBA's commonest answer** (it passes 11-18 there).
Opener competes only with something to say: a four-card major at the
cheapest level, a jump in it with 16-18, notrump with a stopper, or a
six-card suit with 13+. Everything else passes, by rule rather than by
having no rule.

Over a **jump** by them (`1x (1y) X (3y)` and higher) even that is too
much: BBA passes almost everything, and bidding there cost 43 calls.
That context keeps only a four-card major or a stopper **with 13+**,
and passes otherwise.

`when y is not x` on both contexts: their bid in our suit is a cue bid,
not an overcall. Without it these rules fired over Michaels and cost
about 20 calls in the `1x (2x) X (4y)` auctions alone.

## Measured effect (2026-09-23)

Before → after, for the whole second-call slice in this module and
`advances.bid` together:

| | subset before | subset after | corpus before | corpus after |
|---|---|---|---|---|
| calls | 85.1% (89437) | 85.1% (89481) | 74.9% (1331086) | 74.9% (1331913) |
| identical auctions | 41.3% | 41.4% | 14.9% | 14.9% |
| same contract | 53.5% | 53.6% | 27.0% | 27.2% |
| par, net IMPs | -7574 | -7431 | -244279 | -239918 |
| no rule in a live auction | 5989 | 5692 | 165221 | 156762 |

252 scenarios moved: +827 calls, +358 boards with the same contract,
-8,459 no-rule points. The biggest movers are Double_double (+82 calls),
Maximal_Double (+55), McCabe_after_WJO (+49 calls, -560 no-rule),
After_Partner_Overcalls (+41 calls, +26 contracts) and
After_Opp_Overcalls (+35). Two scenarios gain no-rule points because
their auctions now live longer than they used to
(After_Partner_Overcalls +174, Double_by_Advancer +115); both gained 26
contracts in exchange.

Accepted cost: 13 more boards flagged **trump fit under 7 cards** on the
Basic-Bridge subset (176 → 189), nearly all opener naming a minor for a
negative double whose `shows` is a disjunction, so the engine cannot
know partner really holds it.

## Gaps and open questions

- **Their 1NT overcall**: `after 1x (1y)` binds a suit, so nothing fires
  after `1x (1N)` and responder passes. It is the largest single no-rule
  point left in the corpus — about 3,700 boards (`1D 1NT`, `1C 1NT`) —
  and the `Opps_Overcall_1NT` scenario is 500 of them.
- **Opener's rebid in a contested auction generally** is now the biggest
  remaining cluster of dead auctions, and it is not written anywhere:
  `1x (1y) 2z (P)` 68 boards, `1x (X) 1y (P)` 65, `1x (X) P (2y)` 60,
  `1x (1y) 1z (2y)` 55, `1x (X) P (2x)` 51, `1x (X) XX (1y)` 50,
  `1x (1y) 1z (P)` 48 on the Basic-Bridge subset alone, over 500 boards
  in all. It sits between this module and `base/rebids.bid`; a decision
  on where it belongs would unblock it.
- **Responder's second call in a contested auction**: `1x (1y) X (P) 2x`
  is 54 boards on the subset, and it is the next turn after the rules
  added here.
- **Jump shifts** (`competitive.jump_shift_after_overcall` is strong on
  this card) and Lebensohl over an overcall of 1m (BBA plays it on the
  21GF card).
- **Trap passes**: BBA passes with length and strength behind the
  overcaller where we double or cue. Trap_Pass loses 3.0 points of call
  agreement, and its contracts gain 2.6.
- Opener's rebid after our side has been doubled
  (`competitive.jordan_2nt`, support redoubles).
- **For Rick.** Opener's answer to a negative double now names an unbid
  four-card minor before rebidding a four-card suit of his own, and bids
  a three-card major for partner's promised four before a five-card
  minor. BBA is not consistent about this (it bids 2♦ on one 4-5 minor
  hand and 2♣ on another identical one); the order above is what
  measured best.
