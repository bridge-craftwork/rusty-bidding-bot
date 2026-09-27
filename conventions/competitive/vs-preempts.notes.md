# vs-preempts (`vs-preempts.bid`): notes

## Why (2026-09-27)

`probes/tools/par_blame.py` over the corpus: "the side entitled to par
let them play" cost us 98,160 IMPs from par (BBA 32,191), and the
largest shapes were their weak two passed out (3,223 boards) and their
three-level preempt passed out (1,187). Our overcall and takeout-double
rules covered only one-level openings; over 2x and 3x the only call was
pass. Rick: start on the defence against preempts.

## BBA on the Basic-Bridge card (probes, 400 random hands each)

Direct seat over 2♦/2♥/2♠ (`probes/vs-w2-2{D,H,S}.toml`; love all and
all vulnerable nearly the same):

| call | BBA's meaning | HCP seen |
|---|---|---|
| pass | up to 16 total points | median 10 |
| X | takeout, 12+, 3+ in each unbid suit (or strong) | median 17 |
| new suit at two | 5+, 12-17 total points | median 13 |
| new suit at three | 5+, 12+ (13+ vulnerable) | median 13-14 |
| jump to 3M | 6+, 15+ | 14-16 |
| 2NT | 15-17 balanced, stopper, no five-card major | 15-17 |
| 4M | 7+, game | 10-17 |

Advancer after 2♥/2♠ X P (`vs-w2-2{H,S}-X-adv.toml`): the cheapest
four-card suit 0-12 (majors first); pass with 3+ of theirs (a flat
four-card holding with values is a penalty pass); 3NT 12+ with a
stopper; the cue bid 13+ forcing to game; a jump in a major 8-11 (only
where it stays below game); 4M game; 5m game.

## The rules (step 1: direct seat; step 2: the replies)

As above, with our overcall discipline: the suit must be worth bidding
(two of the top three, three of the top five, six cards, or 13 total
points), the takeout double two or fewer of theirs, the power double
17+ HCP with three or fewer of theirs. A weak two is told from a strong
2♣ by the opener's shown maximum (12 HCP). Also the doubler's rebid (a
pass denies 17+), advancing our overcall and our 2NT, and a pass for
the weak-two side once we have acted (their competitive raises come
from total-tricks.bid).

Grid agreement with BBA: direct seat 84-89%; advancer 60-64% (the
rest mostly 3NT against a cue bid, and BBA's flat passes with three of
their suit).

## Measured (full corpus, par distance / side)

| step | full corpus | Basic_* all | Basic_* competitive |
|---|---|---|---|
| direct seat only | -905 / +6,891 | -16 / +58 | -18 / +18 |
| with the replies (kept) | **+3,291 / +11,442** | +3 / +81 | -5 / +36 |

The direct seat alone split the yardsticks because with no replies
advancer passed our double, leaving their weak two doubled; with the
replies both agree. "Let them play" fell from 98,160 to about 83,000
IMPs after step 1 alone.

## Next

Balancing over (2x) P (P); three-level preempts; Lebensohl after the
double on the cards that play it (`competitive.lebensohl_weak_twos`,
on in the 21GF cards); the weak-two side's own continuations over our
double (redouble, new suits).
