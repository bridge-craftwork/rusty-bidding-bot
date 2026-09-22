# rebids (`rebids.bid`): notes

Opener's rebid after a one-level suit opening and an uncontested response
(Standard American, BBA's Basic-Bridge card). Cases: `rebids.test`.

## Guidance (Rick, 2026-09-22)

- **1NT rebid: 12-14 HCP**, balanced.
- **Jump rebid of opener's suit, or jump raise of responder's suit: 16-18
  total points.**
- **Reverse: 17+ total points, first suit longer than the second.**
- **Jump in a new suit: 19+ total points** (game forcing).

## How the rest was set

BBA's meanings for opener's second call (bba-cli --all-meanings over the
Basic_* deals), with the hands that made them:

| Sequence | BBA |
|---|---|
| New suit at the one level | 11-16 total, 4+ |
| New suit at the two level, lower | 11-17, 4+ |
| Rebid own suit | 11-14/15, 6+ |
| Raise responder's suit | 11-15, 4+ |
| 2NT | 18-19 balanced |
| After 1M-2M: pass / 3M / 4M | 11-14 / 14-17 / 13-20 (median 17) |
| After 1M-3M: pass / 4M | 10-13 / 12-17 |
| After a 2/1: rebid the major | 10-14, 5+ (the default) |

Measures: notrump rebids in HCP, suit rebids in suit points (HCP + ½ per
card beyond four), raises in support points tp(y). Once a fit is found
(responder raised), opener counts support points too: BBA bids game over
2M with 16-18 and a singleton, and passes 14-15 with doubletons. Bands:
pass up to 16, invite 17-18, game 19+; after a limit raise accept at 15+.

Other choices, from BBA: a new suit before rebidding a six-card suit
(6-4 shows the four); four spades bid 1♠ before 1NT, but four-card support
for responder's major raises first; 1NT includes 5-4-2-2; after 1♣-1♦ a
balanced hand rebids 1NT even with one four-card major, and 1♥ with both;
a minimum with no suit to rebid and no lower suit (5-4 with a singleton)
rebids 1NT; after a 2/1 a 5-3-3-2 minimum rebids its major.

## Evidence

Basic_* opener's rebids (1,881 positions): 80.1% agree with BBA (almost
none bid before). Basic_Openers_Rebid: calls 75.7% → 91.2%, identical
auctions 8.6% → 43.4%. Whole corpus: calls agreeing 68.5% → 69.9%, same
contract 14.9% → 20.8%.

## Accepted differences from BBA

- **1NT with 15-16 and a 5-4-2-2** (BBA's range reaches 16): Rick's 12-14.
- **A jump in a new suit with 16-17** (BBA 17-20): Rick's 19+.
- Invitation boundaries over a simple raise (`KQ654.KQ85.A2.J8`: BBA passes,
  we invite with 17 support points).
- 1D-1S with four clubs, balanced: BBA mixes 1NT and 2C.

## Gaps (not built yet)

- **Inverted minors** (the 2/1 cards): 1m-3m is weak there, but we read it
  as a limit raise, so opener bids 3NT over it.
- **A passed hand's new suit is not forcing**: BBA's opener may pass it.
- Slam tries by opener (4NT over a limit raise, cuebids).
- Rebids after 2♣ strong openings, preempts and interference.
