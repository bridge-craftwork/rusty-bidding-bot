# Range probe: where the robots draw the lines (2026-11-26)

Question (Rick, 2026-11-26): GIB's tree was used to find flaws in our
ranges; how much do the robots themselves vary on common cases, and where
do we sit? Tables of every cell: [TABLES.md](TABLES.md). Scripts and data
in this folder.

**Short answer.** On the plain boundaries the bidding robots agree within
about one HCP:
- pass up to 5 HCP, 1NT from 6;
- a 2/1 from 12 or 13;
- a simple raise up to 8-9, a limit raise at 9-11;
- opener's 1NT rebid at 11/12-14 and 2NT at 18-19;
- a reverse from 16-17.

They split on **treatments**: what to do with 10-11 HCP and three trumps,
how low an invitational jump shift goes, whether 1NT can hold a six-card
minor or three trumps, Bergen raises or not.

GIB's tree sits inside the robots' spread on most counts. Its stated HCP is
often a point above what BBA and Brill actually bid, and its total points
are higher again.

**Rusty is out of line with all four robots in three places, and each is a
units problem.** A `points` or `tp` count that adds length or shortness to
HCP lets the hand make the call 2-3 HCP below everyone else's floor:
1. the invitational jump shift (3♣/3♦) on 7-9 HCP;
2. the 3NT response on 11 HCP;
3. a game-forcing 2♣ on 10-11 HCP with three trumps.

Elsewhere we sit within the robots' spread.

## 1. Test set

`gen_deals.py`, seed 11: dealer North, nobody vulnerable, IMPs.

| position | fixed hand | deciding hand | deals |
|---|---|---|---|
| 1♠ (P) ? | 4 normal 1♠ openers, cycled | South, HCP 3-14 | 240 (5 classes × 12 HCP × 4) |
| 1♥ (P) ? | the same with hearts | South, HCP 3-14 | 240 |
| 1♦ (P) 1♠ (P) ? | 3 normal 1♠ responders (5 spades, 7-10 HCP) | North, HCP 11-21 | 200 (5 classes, no balanced 15-17 or 20-21) |

- **Responder classes:** `nofit` (≤2 in the major opened, ≤3 in the other,
  no six-card minor), `longminor`, `4other` (4+ in the other major),
  `3support`, `4support`.
- **Opener classes:** `bal`, `4support` (four spades), `6diamonds`, `5D4C`,
  `5D4H`.
- **Other hands:** random. East (and West for the rebid) were drawn quiet,
  so that WBridge5 reaches the position.

## 2. How each robot was asked

| robot | how | card or system |
|---|---|---|
| Rusty | `rbb call --batch`, engine 070f438, compiled-in pack; also on the pack's `bottom-review` (`--rules-at`) | 21GF-GIB |
| BBA | `rbb grid` with bba-cli 0.5.0 | 21GF-GIB.bbsa |
| Brill | public `/bid`; each answer also carries Brill's `requires` expression, its stated range | its own |
| WBridge5 | the whole deal at all four seats, its default system | its own |
| GIB | the tree's stated meanings (`gib_tree.py show`, archived) | GIB 2/1 |

- **WBridge5 coverage:** it reached all 480 responding positions but only 97
  of the 200 rebid positions. On the rest it opened 1NT, 2♣ or 2NT, or
  East/West came in.
- **WBridge5's defaults are not 2/1.** Its 2/1 is not game forcing, and it
  plays Bergen 3♣/3♦ raises and a preemptive 3M.
- **Sample size:** four hands a cell, so each boundary is ±1 HCP.
- **Point counts:** no robot's count is known exactly, so all boundaries are
  observed HCP.

## 3. Responder after 1♠ (P) and 1♥ (P)

| boundary (HCP) | GIB stated | BBA | Brill | WBridge5 | Rusty |
|---|---|---|---|---|---|
| pass / 1NT, flat | 5-/6+ | 5/6 | 5/6 | 5/6 | 5/6 |
| … with a 6-card minor | – | 6 often passes | 5 bids 1NT (rule: 5-9 with a 6-card suit) | 5 bids 1NT | 5/6 |
| 1NT top / 2/1 floor | 1NT ≤12 total; 2m 13+ | 1NT through 12; 2/1 from 13 | 1NT ≤11; 2/1 from 12 | 1NT ≤10; non-forcing 2/1 from 10-11 | 1NT ≤12; 2/1 mostly from 12 |
| invitational jump shift 3m | 9-11 | 10-12 | 9-10 | not played | **7-10** |
| 1♠ over 1♥: floor | 6 total | 5-6 | 3-4 | 4-5 | 6 |
| 3 trumps: simple raise | 7-10 total | 5-9 | 5-9 | 5-9 | 7-9; **1NT on 6** |
| 3 trumps at 10-11 | – | 3M, or 4M | 1NT (raise later) | 2NT or 2♣ | 3M at 9-10; **3NT or GF 2♣ at 11** |
| 3NT response | 13-15 total | 13-14 | 12-14 | – | **from 11** |
| 4 trumps: simple raise | 7-10 total | 5-8 | 5-8 | 4-6 (Bergen) | 6-9 |
| 4 trumps: limit raise | 10-12 total | 9-11 | 9-11 | Bergen 3♣ 7-9, 3♦ 8-11 | **8-12** |
| Jacoby 2NT floor | 13 total | 10 | 12 | 11-12 | 10 |
| splinter | 13-16 total | 13-14 | 10-14 | 10-13 | 11-14 (bottom-review 10-12) |

Agreement on the same call over 1♠ (P), by pairs:

| | BBA | Brill | WBridge5 |
|---|---|---|---|
| Rusty | 72% | 76% | 59% |
| BBA | – | 75% | 58% |
| Brill | | – | 69% |

1♥ (P) is about the same.

## 4. Opener's rebid after 1♦ (P) 1♠ (P)

| rebid (HCP) | GIB stated | BBA | Brill | WBridge5 (thin) | Rusty |
|---|---|---|---|---|---|
| 1NT | 12-14 | 11-14 | 11-14 | 12-14 | 11-14 |
| 2NT | 18-19 | 18-19 | 18-19 | 18-19 | 18-19 |
| 2♠ raise | 12-15 total | 11-15 | 11-14 | 11-14 | 11-14 |
| 3♠ jump raise | 16-18 total | 13-18 | 12-17 | 14-17 | 12-16 |
| 4♠ or a splinter | 19-22 total | 16-21 | 18-21 | 16-21 | 16-21 |
| 2♦ (6 diamonds) | 11-15 | 11-14 | 11-15, or 1NT | 11-14 | 11-14 |
| 3♦ jump rebid | 17-20 total | 14-17 | 14-21 | 15-16 | 14-17 |
| 2♣ (5♦ 4♣) | 11-16 | 11-17 | 11-16, or 1NT at 12-14 | 11-16, or 1NT | **11-18** |
| 3♣ jump shift | 18-21 | 18-21, often 3NT/2NT instead | 17-21 | 18-20 | 18-21 |
| 2♥ reverse from | 18 total | 16-17 | 14-17 | 14 | 16-17 |

Agreement: Rusty–BBA 74%, Rusty–Brill 74%, BBA–Brill 62%.

## 5. Ours against all the robots, against GIB only, and the GIB-tuned thresholds

**Out of line with all four robots and GIB (candidates to fix):**
1. **Invitational jump shift on 7-9 HCP.** `responses.bid:250-252` reads
   `points=10..12`, and length counts toward it. Example: 3.T.AT65432.KT94
   bids 3♦ over 1♠. Everyone else starts at 9-10 HCP.
2. **3NT response on 11 HCP.** `responses.bid:323` reads `points=13..15`.
   Example: Q63.A9.T98.KQT86. Everyone else starts at 12-13.
3. **GF 2♣ with three trumps on 10-11 HCP.** `responses.bid:276` reads
   `tp(M)>=13`, so a singleton makes it a game force. Examples:
   AT4.973.9.AQT865 and KJ4.9876.7.AK986. BBA bids 3♠ or 4♠ with these,
   Brill 1NT.
4. **Minor:** the 4-card limit raise from 8 HCP (others from 9).
5. **Minor:** opener's 2♣ up to 18 HCP (others stop at 16-17).

**Out of line with GIB only, inside the robots' spread (not a reason to
move a threshold):**
- Jacoby 2NT from 10 HCP (BBA also bids it from 10).
- The 1NT response top and the 2/1 floor at 12 (Brill matches us).
- Opener's 4♠ raise from 16 HCP (BBA and WBridge5 match us).
- The 3♦ jump rebid from 14 HCP (BBA and Brill match us).
- The splinter: the robots range from 10-14 to 13-14. Rick's 10-12 is
  inside that spread, and no closer to GIB's 13-16 total.

**With GIB, against the bidding robots:** the weak three-card raise through
1NT. We bid 1NT on 6 HCP with three trumps, and 1♠ over 1♥ from 8 HCP with
three hearts and four spades. BBA and Brill raise instead; GIB's 1NT allows
three trumps. Our rule names it as a deliberate treatment (`style is not
bba, system is not brill`).

**The thresholds set to GIB's numbers (`bottom-review`):** rerun on that
branch, our calls changed on 11 of 680 deals, all splinter against Jacoby
2NT. The branch's other GIB-tree changes are readings (what partner reads
from a call), not choices. On these positions they agree with what BBA and
Brill actually bid, so nothing here argues for undoing them. Nothing here
supports moving a choice threshold to GIB's number where GIB differs from
BBA and Brill: their actual bidding usually runs a point below GIB's stated
HCP.

## 6. Questions for Rick

1. Fix the three units problems by stating HCP floors beside `points` /
   `tp`: HCP ≥ 9 for the jump shift, ≥ 12-13 for 3NT and for the
   game-forcing fit bid?
2. The three-card raise at 10-11 HCP has no consensus. Take BBA's 3M,
   Brill's 1NT-then-3M, or keep ours without the 3NT and 2♣ parts?
3. Keep the weak three-card raise through 1NT?

Not probed yet: the rebids after 1♣–1♥ and 1♥–1♠, and competitive
variants.
