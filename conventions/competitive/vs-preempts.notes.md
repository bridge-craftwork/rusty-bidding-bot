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

## Steps 3-4: balancing and three-level preempts (2026-09-27)

BBA, balancing over 2♥ P P (`vs-w2-2H-bal.toml`): a reopening double
with 12+ and two or fewer hearts; new suits from 9 total points at the
two level, 11 at the three; 2NT 16-18; a jump to 3M with six and 15+;
it passes 65%. Direct over 3♣/3♦/3♥/3♠ (`vs-p3-3*.toml`): the double
from 12; new suits at three from 12 total points, at four from 14; 3NT
17-21 with a stopper; 4M with seven; it passes 70-80%. Grid agreement
went from 65-82% (we passed everything) to 84-91%. The replies
(advancer, doubler's rebid, advancing an overcall, their side's pass)
now also cover three-level preempts and the balancing seat: without the
balancing replies advancer left our reopening double in (2♥ P P X P P
P, +670 to them).

| | full corpus (par / side) | Basic_* all |
|---|---|---|
| balancing and 3-level, direct replies only | +2,394 / +10,957 | -45 / +10 |
| with the balancing replies (kept) | **+4,077 / +10,981** | -34 / -13 |

Corpus no-rule positions 4,679 -> 3,277. Basic_* is slightly negative
(50 boards, our E/W balancing against the Basic_Weak_2 opener): single
deals, e.g. a balancing 2♠ where par was their 4♥ making.

## Step 5: balancing over three-level, and four-level preempts (2026-09-27)

BBA (`vs-p3-3H-bal.toml`, `vs-p4-4H.toml`, `vs-p4-4S.toml`,
`vs-p4-4H-bal.toml`): balancing over 3♥ a double with 12+ and at most
one heart, new suits from 11 (three level) / 13 (four level), 3NT
17-20; over 4♥ a takeout double from 14, 4♠ with five from 14, five of a
minor from 16; over 4♠ the double is **penalty, 17+** (values); over
4♥ P P a double with 12+ and no hearts, 4♠ from 13, 5m from 15.
Advancer over our four-level double takes out into five spades over
4♥, else leaves it in. Grid agreement 74-87% -> 82-90%.

Full corpus +781 by par distance, +2,598 by side (930 boards); Basic_*
unchanged (no four-level preempts there).

## Lebensohl after the double: tried, not kept (2026-09-27)

The 21GF cards set `competitive.lebensohl_weak_twos`. BBA on
21GF-DEFAULT (`vs-w2-2{H,S}-X-leb.toml`): 2NT is a relay to 3♣ (weak,
then pass or correct; also game values with their suit stopped, then
3NT: "slow shows"); a direct suit at the three level is constructive,
9-12; the cue bid 12+; a direct 3NT. Written as a variant gated on the
card field, it lost on both yardsticks every time (full corpus, par
distance / side, against the natural rules on the same cards):

| version | full corpus |
|---|---|
| relay, constructive suits, slow-shows 3NT | -653 / -2,256 |
| + the doubler's natural 2NT accept gated off (it answered the relay with 3NT) | -961 / -1,497 |
| + a strong doubler breaks the relay (3NT with 19+) | -264 / -669 |
| + weak 2NT only without a four-card major at two; stopper route from 11 | -736 / -1,013 |

(Superseded 2026-10-05: see "Lebensohl after the double: played from
the card" below.) Reverted: the natural structure is used on every card, including those
that list Lebensohl. **For Rick:** the card says we play Lebensohl
there, and par says the natural replies (a weak three-level suit
0-10, 3NT with a stopper, the cue bid) do better in our engine. Keep
natural, or play the card's convention and accept the cost while the
Lebensohl continuations mature?

## Lebensohl after the double: played from the card (2026-10-05)

Rick's pace ruling (2026-10-05, CLAUDE.md "Pace"): the card decides and
the cited definition is the spec, so the cards that list Lebensohl (all
21GF cards) now play it; Basic-Bridge and Precision keep the natural
replies. **The trade:** the 2026-09-27 table above says the natural
replies did better on par in our engine; we accept that cost for
playing the partnership's card, and the big par runs come back to it
in the periodic batch.

Over 2D/2H/2S doubled (direct or balancing), advancer:

| call | meaning |
|---|---|
| 2-level suit | natural, 0-10 (the natural rule, unchanged) |
| 2NT | relay to 3C: 0-8 with a suit to play at three (no four-card major biddable at two), or 13+ with their suit stopped and no five-card unbid major |
| 3-level suit directly | constructive, 9-12 HCP, four or more |
| cue directly | 13+, exactly four of an unbid major, no stopper ("fast denies") |
| 3NT directly | 13+, no four-card major, no stopper |
| 4M | five or more of an unbid major and 11+ |
| pass | penalty (as before), or five or more of theirs and 0-8 |

After 2NT the doubler bids 3C (3NT with 19+ and their suit stopped);
advancer passes with clubs, corrects to a lower suit (weak), or shows
game values with a stopper: the slow cue (four of an unbid major) or
3NT. The doubler answers either cue with an unbid four-card major,
else 3NT.

Fast-lane compare (`Lebensohl_vs_Opps_W2_*`, 150 boards): call agreement
83.5% (natural) -> 89.2% (NS 65.0% -> 77.7%). Remaining divergences at
the convention's calls, classified:

- BBA style: BBA's doubler breaks the relay with an artificial 3 of
  their suit on strong hands (we bid 3C, or 3NT with 19+ and a stop);
  BBA bids a direct 3NT *with* a stopper on several balanced 13-15s,
  against its own scenario text and the slow-shows rule we follow; a
  flat 10+ with four of theirs we pass for penalty where BBA relays.
- BBA's constructive range starts at 9 HCP: its 8-counts relay (our
  ranges were moved to match: weak 0-8, constructive 9-12, game 13+).
- Don't care: single boards of judgment (BBA 2S with four spades and 12
  where we bid 4S).

**For Rick:** the doubler's strong relay break (BBA's artificial cue)
is not written; tell us if you want it.

## Next

The weak-two side's own continuations over our double (redouble, new
suits).

## Sources

- **Rick's rulings:** start on the defence against preempts (2026-09-27).
  The overcall and takeout-double discipline comes from the rulings in
  `overcalls.notes.md` and `takeout-double.notes.md`.
- **BBA probes (Basic-Bridge unless noted, 400 random hands each):**
  `probes/vs-w2-2D.toml`, `vs-w2-2H.toml`, `vs-w2-2S.toml`;
  `vs-w2-2H-X-adv.toml`, `vs-w2-2S-X-adv.toml`; `vs-w2-2H-bal.toml`;
  `vs-p3-3C.toml`, `vs-p3-3D.toml`, `vs-p3-3H.toml`, `vs-p3-3S.toml`;
  `vs-p3-3H-bal.toml`; `vs-p4-4H.toml`, `vs-p4-4S.toml`,
  `vs-p4-4H-bal.toml`; Lebensohl on 21GF-DEFAULT, `vs-w2-2H-X-leb.toml`
  and `vs-w2-2S-X-leb.toml`.
- **Corpus measurements:** `probes/tools/par_blame.py` for the motive,
  and each step judged by par distance and by side IMPs.
- **Lebensohl after the double** (2026-10-05): Bridge Bum, "Lebensohl
  over weak two bids" (https://www.bridgebum.com/lebensohl_over_weak_two.php):
  2NT relay for weak hands and slow-shows game hands, direct three-level
  suits constructive, the cue four of the unbid major, fast denies;
  Larry Cohen, "Lebensohl" (https://www.larryco.com/bridge-articles/lebensohl)
  for fast denies / slow shows. Practice-Bidding-Scenarios
  `btn/Lebensohl_vs_Opps_W2_*.btn`. Deviations: ranges in HCP from
  BBA's corpus (weak 0-8, constructive 9-12, game 13+) rather than the
  source's 0-7 / 8-11; a five-card unbid major with game values bids
  game rather than going through the cue; the cue promises exactly four.
  Played on the cards that list it, against the earlier par result
  (the section above); Rick's pace ruling, 2026-10-05.
