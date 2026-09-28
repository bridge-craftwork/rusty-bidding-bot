# gerber (`gerber.bid`): notes

Gerber (card `slam.gerber.play`, alias `other_conventions.gerber.play`;
BBA's `Gerber`, on 17 of the 18 corpus cards, and on Rick's card). 4♣
over partner's natural notrump asks for aces: 4♦ 0 or 4, 4♥ 1, 4♠ 2,
4NT 3. 5♣ then asks for kings on the same steps. The asker places the
contract; 4NT after an answer is a sign-off. Cases: `gerber.test`.
Built 2026-09-28.

## Where 4♣ is Gerber

- **Directly over partner's 1NT or 2NT opening**, and over 2♣–2♦–2NT.
- **Over partner's 1NT or 2NT rebid** after 1x–1y, unless the card has
  `gerber.only_over_nt_openings` (BBA's "Gerber only for NT openings",
  set on 21GF-MSS, 21GF-Multi and 21GF-NoInvertedMinor). BBA uses it
  there (Gerber_By_Responder: 1♦–1♠–1NT–4♣, 1♣–1♥–2NT–4♣).
- **Nowhere else.** Not after a transfer, a Stayman answer, over a
  minor raise, or when clubs have been bid naturally: 1♣–2♣–2NT–4♣ is
  whatever it was before (27 corpus boards, in minor-raise scenarios).
  Not over our 1NT overcall either: no corpus evidence
  (We_Overcall_1N_then_Gerber has no BBA file).
- The card's other Gerber fields are left alone: `gerber.over` is free
  text (Bridge-Classroom's seed card says `1nt_2nt_only`, which is what
  the opening context does), and `gerber.directly_over_nt`,
  `over_nt_seq`, `non_nt_seq` (the ACBL card's three boxes) are read by
  no rule yet. Rick's card sets only `other_conventions.gerber.play`.

## Decisions

- **Texas.** Gerber is 4♣ only; Texas is 4♦/4♥, so they coexist over
  1NT, and after a Texas completion 4NT stays keycard (texas-transfers).
  With `transfers.texas_4c` (4♣ a transfer to diamonds; no rule reads it
  yet) Gerber is off directly over 1NT.
- **The 4♣ minor slam try over 2NT** (two-nt-responses.bid, 2026-09-27)
  is off when Gerber is on: the same hands (six clubs, or five
  unbalanced, slam-invite values) ask for aces instead, then bid 6NT
  with three aces between the hands (6♣ at IMPs; at matchpoints 6NT
  scored better on par: 19 boards, -50 for 6♣), or sign off in 4NT. The
  4♦ diamond try stays. BBA does the same: 2NT–4♣ then 6♣/6♦/6NT, from 9
  HCP with six clubs (`probes/gerber-resp-2N.toml`). Its follow-ups
  (`after 2N (P) 4x (P)`) are restricted to diamonds when Gerber is on,
  so opener never reads Gerber as the try.
- **Who asks.** The hands that used to bid 6NT directly: balanced, 32
  points between the hands on opener's minimum over 1NT (17 opposite
  15-17), `strength>=slam` over 2NT. 16 opposite 15-17 keeps Rick's
  quantitative 4NT. BBA asks a point lower (31 HCP: 16 over 1NT, 11 over
  2NT, `probes/gerber-resp-1N.toml`, `gerber-resp-2N.toml`) and then bids
  6NT only with three aces between the hands; that is the `bba`
  treatment. A four-card major does not stop the ask (see "Corpus":
  2♣ → 4♣ is +647).
- **Grand slam and the king ask** (thresholds by corpus A/B, below):
  7NT with all four aces and 35 between the hands (my points plus
  opener's minimum); with 33-34, ask for kings and bid 7NT when all four
  are there, else 6NT. A long club suit does not ask for kings. Corpus
  A/B (all scenarios, vs BBA with par as the yardstick, both modules in):

  | 7NT from | king ask | vs BBA |
  |---:|---|---:|
  | 37 | none | -108,591 |
  | 37 | 34-36 | -106,518 |
  | 36 | 33-35 | -106,262 |
  | 36 | 32-35 | -106,572 |
  | 35 | none | -106,396 |
  | 35 | 34 | -106,004 |
  | **35** | **33-34** | **-105,913** |

  The textbook grand needs 37. Caution for Rick: the slam scenarios that
  carry most of this (Gerber_By_Responder, Grand_Slam_Invite) deal
  slam-rich hands by design, so the corpus may favour bold grands; the
  king ask at 33 alone lost (6NT → 5♣: -78 on 155 boards after 4♠) and
  the gain is in 7NT at 35-36.

## Evidence from BBA

- `probes/gerber-resp-1N.toml` (21GF-DEFAULT, 300 hands, 14-22 HCP, no
  four-card major, after 1NT): 4♣ from 16 HCP with a balanced hand,
  3NT below; with a five-card minor often 3♦ (its minor slam try). After
  the answer: 6NT, or 4NT with two aces missing, 7NT at 20+ with all the
  aces. It never asks for kings.
- `probes/gerber-resp-2N.toml` (300 hands, 9-17 HCP, after 2NT): 4♣ from
  11 HCP balanced, from 9 with six clubs; 7NT with all the aces from 13-14
  HCP.
- Corpus, all scenarios: 1NT–4♣–4♠–6NT 227, –4♥–6NT 152, 2NT–4♣–4NT–6NT
  99; 4NT sign-offs after an answer 65; 7NT 119; no 5♣ king ask anywhere.

## Corpus

All 343 scenarios, 2026-09-28, Smolen and Gerber together, against the
tree they were built on (`rbb compare --json`, then
`probes/tools/sideimps.py`):

- **vs BBA, par as the yardstick: -114,531 → -105,913 (+8,618)** on
  5,072 changed boards; calls agreeing with BBA 76.7% → 76.9%,
  identical auctions 16.5% → 17.1%; "no rule in a live auction" 2,835
  → 2,795.
- **IMPs to the side that changed its call: +8,930** (3,527 boards
  scored). Both yardsticks agree. The only negative openings: our 1NT
  overcall of 1♥ (-58 on 23 boards) and of 1♠ (-12), both Smolen
  (smolen.notes.md).

The Gerber part, by the first call that changed:

| Where | Boards | vs BBA |
|---|---:|---:|
| 1NT: 6NT → 4♣ | 913 | +562 |
| 1NT: 2♣ → 4♣ (a four-card major, slam values) | 474 | +807 |
| 2NT: 3♣ → 4♣ (the same over 2NT) | 411 | +828 |
| 2NT: 6NT → 4♣ | 213 | +346 |
| opener over 2NT–4♣ (was the club try) | 217 | +408 |
| 1x–1y–1NT/2NT: 3NT or 2♣ → 4♣ | 346 | +2,793 |

Scenarios: Gerber_By_Responder +1,416, Soloway_Jump_Shift_Type-3
+1,109, Grand_Slam_Invite +1,086, Puppet_Stayman_2N +417, 2N_and_MSS
+385; the worst -16 (SCS_1C_any_8plus_Resp, 6 boards).

**A four-card major does not stop the ask.** Over 1NT and 2NT the
Gerber hands with a four-card major used to bid Stayman and then 6M or
6NT; now 4♣ outranks 2♣ (equal priority, 4♣ is more descriptive) and
the 4-4 fit is not looked for. Par likes it (+807 and +828 above),
probably because nothing after Stayman checks aces. Standard practice
would be Stayman first; for Rick.

## Differences from BBA

- We ask from 17 over 1NT (BBA 16) and keep the 4NT invitation at 16.
- We ask for kings with 34 between the hands; BBA never does, and goes
  straight to 6NT or 7NT.
- BBA's 3♦ minor slam try with 16-18 and a five-card minor (we have ours
  at 3♣/3♦ with six, or five unbalanced).

## Gaps

- Gerber by opener (1m–2NT–4♣, 2♣–2NT–4♣: Gerber_By_Opener, which BBA
  cannot bid) and over our 1NT overcall.
- The card's `gerber.directly_over_nt` / `over_nt_seq` / `non_nt_seq`
  boxes; `gerber.over` as an enum once its values are known.
- Interference over 4♣ (DOPI/ROPI) is not handled here: after a double
  the answers are as without it.

## Sources

- **The convention:** Bridge Bum, "Gerber"
  (https://www.bridgebum.com/gerber.php), read 2026-09-28: "An
  immediate 4♣ response to any no-trumps bid (or overcall) is Gerber";
  the step answers 4♦ 0 or 4, 4♥ 1, 4♠ 2, 4NT 3; 5♣ asks for kings on the
  same steps. Wikipedia, "Gerber convention"
  (https://en.wikipedia.org/wiki/Gerber_convention), read 2026-09-28:
  the same answers, not "when a natural club suit bid has been made".
  Where we differ: Bridge Bum also plays it as a jump after Stayman and
  over NT overcalls; we do neither yet. Neither page gives the 4NT
  sign-off or point counts: the sign-off is standard practice, not yet
  cited; the counts are ours (below).
- **Rick's rulings:** 17 opposite 15-17 bids slam, 16 invites
  (one-nt.bid, 2026-09-24); Gerber is on his card.
- **BBA probes:** `probes/gerber-resp-1N.toml`,
  `probes/gerber-resp-2N.toml` (21GF-DEFAULT, matchpoints).
- **Corpus measurements:** the 6♣-at-IMPs choice and the grand and
  king-ask thresholds (above).
- **Card fields:** Bridge-Classroom `conventionCatalog.js` (4♣ Gerber:
  directly over NT / over NT sequence / non-NT sequence) and the seed
  card's `slam.gerber.over = "1nt_2nt_only"`.
