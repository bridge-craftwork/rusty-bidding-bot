# Strong openings (`strong-openings.bid`): notes

2♣, 2NT and 3NT, and what follows them. Cases: `strong-openings.test`.

## Guidance (Rick, 2026-09-23)

**Balanced hands go up a ladder, and every rung is a place to stop:**

| | | | |
|---|---|---|---|
| 2NT | 20-21 | 2♣ then 3NT | 24-25 |
| 2♣ then 2NT | 22-23 | 3NT | 26-27 |
| | | 2♣ then 4NT | 28+ |

So the 22+ HCP branch of 2♣ steps around the two notrump openings: a
balanced 26-27 opens 3NT, not 2♣.

**Unbalanced hands** open 2♣ on 23 total points (HCP plus one for each
card beyond four), or on playing strength — within one trick of game,
which is nine tricks with a major and ten with a minor, since game there
is eleven. Playing tricks are counted as 13 − losers.

**Responder**: 2♦ waits. A suit is a positive, 8+ with a real suit — five
cards with two of the top three honours, or six with three of the top
five. 2NT is the balanced positive. After opener describes, the cheapest
minor is the **second negative**: a bust, less than a jack and a queen.

Card fields: `two_level.two_clubs.2d_response` (waiting/negative/steps),
`parrish_bust` for the treatment where 2♥ shows the bust and 2♦ is game
forcing, `notrump.two_nt.range_min/max`, `notrump.three_nt.range_min/max`.

## Evidence, and one open question

BBA opened 2♣ eighteen times across the Basic_* scenarios, on 17-21 HCP
with big shape — `2.AQJ872.AKQJ83.`, `A2.AQJT7542.A.A3`,
`KQT43.AKQ32..AKT`. Its stated meaning is "19 to 37 total points". Every
one of the eighteen has a six-card suit or 5-5.

Rick's rule reads as two **alternative** triggers: 23 total points **or**
within a trick of game. Implemented that way it opens 2♣ on 17 of BBA's
18, but also on 14 of the 67 strong one-level hands BBA opens at the one
level, and it measures worse — the false positives cost more than the
true ones gain:

| unbalanced trigger | calls | auctions | contracts | IMPs vs par |
|---|---|---|---|---|
| playing strength alone | 84.2% | 39.3% | 50.3% | -8,542 |
| and 20 total points | 84.4% | 40.2% | 51.6% | -8,225 |
| and 21 | 84.7% | 40.9% | 52.7% | -7,821 |
| and 22 | 84.7% | 41.1% | 53.2% | -7,636 |
| **and 23** | **85.1%** | **41.3%** | **53.5%** | **-7,574** |

So the rules currently require **both**: 23 total points *and* within a
trick of game. **This is a question for Rick** — his wording says either
one is enough, and the measurement says both. The cost of following his
wording is about 1,000 IMPs over 11,656 boards.

A suit-quality guard came out of the same measurement: the long suit
needs two of the top three honours, or the losing-trick count flatters a
hand like `Q98765.AQ7.AK6.A` (four losers, a suit not worth bidding
twice). BBA opens that 1♠ and so do we.

## Accepted differences from BBA

- BBA opens 2♣ on hands of 20-22 total points that we open at the one
  level (8 of its 18): `AKQ654.AK9.J5.K5` (22), `2.AQJ872.AKQJ83.` (21).
- BBA opens 1♥/1♠ on some hands within a trick of game that we open 2♣.

## Gaps and open questions

- **The 2♥ bust treatment** (`parrish_bust`) is read but only half
  written: 2♥ shows the bust, but 2♦ is not yet game forcing on that
  card, and there is no rule for opener after it.
- `two_clubs.2d_response = steps` is not modelled.
- Opener's second call after the second negative.
- Slam bidding after a positive: the raise agrees the suit but nothing
  looks for keycards.

## Raising opener's major after a positive (2026-09-25)

To_Finesse_Or_Not_To_Finesse was the corpus's worst covered scenario by
par (-2,292 over 500 boards). After 2C-2NT-3S (or 3m-3M), responder
held three trumps and 8-11 and bid 3NT: there was no raise. BBA drives
to slam. Responder now raises to 4M with three or more, agreeing the
suit; opener with the values asks with 4NT.
- Distance from par: +774.
- To the bidding side: +1,556 (165 boards).

BBA's lighter positive responses (any five-card suit with 7+ total
points; 2NT with balanced 7+) are left alone: Rick's guidance (8+ and a
real suit) stands.
- Opening 1M rather than 2NT with 20-21 and a five-card major, as BBA
  sometimes does: -1,717 by distance, -3,432 to the bidding side (626
  boards). Rick's ladder (2NT with 20-21 balanced) stands.

## Sources

- **Rick's rulings (2026-09-23):** the balanced ladder (2NT 20-21, 2♣
  then 2NT 22-23, 2♣ then 3NT 24-25, 3NT 26-27, 2♣ then 4NT 28+); the
  unbalanced 2♣ on 23 total points or within a trick of game; responder's
  2♦ waiting, positives with a real suit and 8+, the second negative.
  Rick's guidance also stands against BBA's lighter positives and against
  opening 1M with 20-21 and a five-card major (2026-09-25).
- **Book practice:** the ladder, playing tricks as 13 minus losers and the
  second negative are standard practice, not yet cited to a book or
  article.
- **BBA evidence:** BBA's eighteen 2♣ openings in the Basic_* scenarios
  and its stated meaning "19 to 37 total points". No `probes/*.toml` spec
  yet.
- **Corpus measurements:** the unbalanced-trigger table, and raising
  opener's major after a positive (2026-09-25).
- **Where we differ from the source:** Rick's wording makes the two
  unbalanced triggers alternatives; the rules require both, because
  that measures about 1,000 IMPs better (open question for Rick). From
  BBA: see "Accepted differences from BBA".
