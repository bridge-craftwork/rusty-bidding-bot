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
