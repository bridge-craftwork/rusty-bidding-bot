# slam-entry: the conditions the slam modules share

A module of named conditions (`define`) and no rules yet. It was made in
the judgment layer's Phase 0 (docs/JUDGMENT-LAYER.md, 2026-09-30) to name
what `rkcb-1430.bid`, `blackwood.bid` and `control-bids.bid` spelled out
in full, and it is where Phase 1 (slam entry: *whether* slam is worth
looking for, one definition read by every module that decides *how*) will
put its rules.

## The conditions

| name | meaning | used by |
|---|---|---|
| `slam_values` | facing a game force: 33 support points between us on partner's floor, or 18 of my own | keycard asks (rkcb, blackwood), opening control bids |
| `slam_values_limited` | facing a limited hand: 33 on partner's maximum, four losers or fewer | the asks and control bids over a limit raise |
| `controlled(x)` | ace or void; with `first_or_second_round`, also king or singleton; shortness only when partner has not shown four of x | control bids, the ask in a control dialogue |
| `first_round(x)` | ace or void (past game) | control bids past game |
| `side_suit_uncontrolled` | a side suit I cannot control on my own | what sends a hand to control bids |
| `two_bare_suits` | two side suits of 2+ cards without ace or king | Rick's rule: no keycard ask with two |
| `denials_promised`, `denials_promised_first` | every suit partner denied is trump, not denied, or controlled by me | going on in the dialogue |
| `all_covered`, `all_covered_by_us` | every side suit covered for the ask | the ask after a control dialogue |

Each replaced its written-out form exactly: on the full corpus our calls
did not change (170,633 boards, 2026-09-30).

`style` in `controlled(x)` is this module's own parameter
(`slam.cue_bids.style`, default `first_or_second_round`): a definition
reads its own module's card parameters wherever it is used.

## Open

- Phase 1 replaces these thresholds with one slam-entry definition
  (slam values 33+ on partner's floor, slam interest 31+ on his maximum),
  calibrated from the double-dummy tables (docs/JUDGMENT-LAYER.md §2(c)).
- `bare_suits <= 1` could replace `!two_bare_suits`; it adds a denial
  when the ask is not made, so it is a measured change.

## Phase 1: where slams are missed (2026-09-30)

Measured on **random deals** (20,000 Pavlicek deals bid by BBA with the
21GF cards, `probes/tools/rpdd_sample.py`), because the scenario corpus
is dealt for its conventions and overweights rare positions (Rick,
2026-09-30: "slam after partner preempts is pretty rare"). Among the
richer side's boards with an eight-card fit, par is a slam 85% of the
time at 31 HCP, 93% at 32 and 100% from 33; BBA bid those slams on
69/118, 67/81, 55/61 and 30/30 boards (31-34 HCP), we on 26, 21, 17 and
**4 of 30**. The missing slams did not come from missed keycard asks
(BBA's 4NT at our first divergence is worth about 400 IMPs there) but
from sequence rules that bid game with no top: opener's jump to 4M over
a game force, responder's "game with support" / "game, no fit" after a
two-over-one, 3NT after a jump shift or a reverse, the 2NT ladder over
a strong 2C.

### Built

- **A fit known but not agreed** (rkcb-1430.bid, blackwood.bid): with no
  trump set and the fit public (`shown_fit(x)`: eight shown between us,
  or partner's six-card suit), 4NT agrees the major and asks, on
  `fit_slam_values(x)`: 33 on partner's floor, or 33 on his top with four
  losers or fewer when he has limited his hand to seven points (a
  preempt). Uncontested and majors only: in competition the first
  version asked on the wrong side and cost; the minor sign-offs after an
  answer are not ready.
- **Fast arrival**: in a game force with an eight-card fit in partner's
  major and no trump set, the cheapest bid in it agrees it and shows
  `slam_interest(x)` (31 on partner's floor); the jump to game is then
  the minimum. The public guard `partner.x.min >= 3` matters: without it
  the rule matched any cheapest heart bid publicly and 2C-2D-2H was read
  as agreeing hearts (-102 on random deals).
- rebids.bid: no jump to 4M with 19+ over a game-forcing response.

| step | random (20k) | corpus |
|---|---:|---:|
| knowledge: partnership sums narrow ranges | +28 | +2,120 |
| fit ask + fast arrival + no 4M jump over a game force | +179 | +3,364 |

(IMPs vs BBA, double-dummy par as the yardstick.)

### Next

- Responder's game bids after a two-over-one ("Game with support",
  "Game, no fit": no top), and 3NT with a minor fit after a reverse or
  a jump rebid.
- 2C-2D-2NT: the ladder bids 3NT with 33 between us; no quantitative
  4NT or 6NT over 2NT.
- Minor fits: the ask and the sign-offs.
- Calibrate `slam_interest` (31) and the fit ask on a second, disjoint
  sample (`rpdd_sample.py --offset`), 100,000 deals.

## Calibration of the slam thresholds (2026-09-30)

Rick approved calibrating on two random samples. **Tuning set**: 100,000
Pavlicek deals (`rpdd_sample.py --count 100000 --offset 1`), bid by BBA
with 21GF-DEFAULT / 21GF-GIB. **Confirmation set**: the earlier 20,000
(offset 0, disjoint). The corpus as a third. Matchpoint-scored deals,
IMPs vs BBA with double-dummy par as the yardstick.

Double dummy, random deals (20k), P(small slam makes) by the side's HCP:
with an eight-card fit 68% at 30, 80% at 31, 92% at 32, 100% at 33;
without one, 6NT 55% at 31, 73% at 32, 93% at 33. In total points
(length counted) about two points higher (8-card fit: 51% at 31, 67% at
32, 86% at 33): the textbook 33 is right when it is the real total, but
our rules count it on partner's *floor*.

| variant (vs base) | tuning 100k | corpus | confirmation 20k |
|---|---:|---:|---:|
| slam values 33 -> 31 on partner's floor | +687 | +1,408 | -45 |
| slam values 33 -> 32 | +408 | +1,118 | |
| slam values 33 -> 30 | +232 | | |
| slam interest 31 -> 30 | +211 | -14 | +2 |
| over 3NT one point lower (6NT 33, 4NT 31) | -1,797 | | |
| responder after opener's new suit one lower | +52 | | |
| responder's 2/1 slam-interest splits one lower | +15 | | |
| midpoint of partner's range (values / interest) | +38 / +2 | | |

Where the change bites (tuning set, new slams by the side's real HCP):
at 31, 218 new slams at 29 HCP or less make 59% double dummy (likely
below break-even at the table), 75 at 30-31 make 85%; at 32, 74 at 29 or
less make 65%, 40 at 30-31 make 83%.

**Adopted: slam values 32 on partner's floor** (`slam_values`,
`fit_slam_values`): both larger sets agree, and it keeps most new slams
in sound territory; 31 scores more but is carried by double-dummy-lucky
slams, and the 20k set (noise about +-260) does not support it. Slam
interest stays 31, over-3NT stays (lower is clearly worse), the rest are
within noise.

Two control-bid gaps surfaced at 32 (control-bids.test "partner stopped
in game over my control bid"): after I bypassed a suit and partner
signed off in game, neither a control bid past game nor the general
game-force keycard ask may go on. Both now stop.

Final (32 plus the two fixes): tuning +412, corpus +1,121,
confirmation -13; no new "no rule" or broken-force boards.

## Phase A: what BBA's slam decision turns on (2026-09-30)

Rick (2026-09-30): "for slam bidding, probe BBA to see what it's using —
this may be more than HCP or total points." Measured on the **vanilla
SAYC random set** (95,558 Pavlicek deals bid by BBA with a bare SAYC
card that plays no Blackwood, so 4NT is natural), where boards with a
slam par cost -21,800 of our -36,678 IMPs vs BBA.

### Where BBA goes past game (the vanilla set)

BBA's first slam-going call (4NT, 5M, 5NT, a six or a seven), on the
5,214 boards that have one, and what those boards cost us:

| BBA's call | boards | IMPs vs BBA | we reached slam |
|---|---:|---:|---:|
| a direct six of a suit | 3,329 | -9,611 | 185 |
| 6NT | 581 | -1,674 | 182 |
| 5M | 696 | -761 | 7 |
| 4NT | 548 | -429 | 17 |
| a seven | 55 | -385 | 12 |

So BBA **jumps to slam** ("calculated bid" in its meanings); it almost
never asks. 3,449 of the boards are uncontested (-9,934). By opening:
a one-level suit opening, opener's side -9,418 (3,477 boards); 2♣
-1,834 (524); responder's side after a suit opening -911; 2NT -286;
1NT -41 (our notrump ladders already agree). The single positions are
small (the largest: `1S P 2D P 3C P 3S P -> 6S` 16 boards, -133;
`1S P 2D P 2S P 3S P 4S P -> 6S` 21, -116; `2C P 2H P 3H P -> 6H` 28,
-94; `2N P 3D P 3H P -> 6H` 22, -79): the loss is spread over hundreds
of positions, so it needs a general rule, not patterns.

Where *our* auction stopped, on the boards where BBA bid a slam and our
uncontested auction did not: **base.bid's "Game reached: nothing more
to say" passed on 2,917 boards (-10,007 IMPs)**, everything else under
60 boards. The commonest of our auctions there: partner's 3NT after a
new-suit rebid (`1H P 1S P 2C P 3N`, `1D P 1S P 2C P 3N`), partner's
4M in an agreed fit (`1H P 2D P 2H P 3H P 4H`, `1S P 2D P 2S P 3S P
4S`, `1S P 2C P 4S`, `1D P 1S P 4S`, `1D P 1H P 4H`), 2♣ auctions
ending in 3NT, and responder's 3NT over opener's 2NT rebid
(`1C P 1D P 2N P 3N`).

### What decides it: made hands

Six positions with a fit, 800 random hands each for the player who
decides (probes below), MP and IMP. Partner's floor is BBA's own
meaning of partner's last call (from `bba_alert` one round earlier, in
BBA's "total points"):

| position (decider) | partner's call per BBA | BBA slam | ours |
|---|---|---:|---:|
| `1NT 3S 4S` (responder) | 15-17, three spades | 29% | 27% |
| `1S 2S` (opener) | 7-9 | 4% | 0 |
| `1D 1H 4H` (responder) | 16-21, four hearts | 25% | 0 |
| `2C 2H 3H` (responder, 2H a positive) | 21-37 | 82% | 0 |
| `1H 2D 3H` (responder) | 14-20, six hearts | 22% | 0 |
| `1S 2D 3C 3S` (opener) | 13-29, three spades | 31% | 0 (we pass 3S) |

Which measure of the decider's hand carries BBA's decision
(`probes/tools/slam_measures.py`, logistic fit with one intercept per
position, log-likelihood per decision, null -0.493; higher is better):

| measure | LL |
|---|---:|
| losers (LTC) | -0.369 |
| controls (A 2, K 1) | -0.358 |
| HCP | -0.335 |
| HCP and losers | -0.304 |
| points (HCP + length) | -0.311 |
| **tp(trump)** (HCP + shortness 5/3/1, capped by trumps) | **-0.288** |
| tp + controls | -0.277 |
| tp, controls and losers (three weights) | -0.272 |
| free card values (A, K, Q, J, shortness, length, bare suits) | -0.269 |

**BBA counts support points, not HCP.** Shortness is most of the gain
over HCP (-0.335 to -0.288); controls add a little (-0.277), losers on
top of that almost nothing. The free fit values the cards, in HCP with a
king = 3, at A 4.5, K 3.0, Q 1.6, J 0.6, shortness +0.7 a point, length
+0.6 a card, a bare side suit -0.5: aces and kings count a little more
than 4-3-2-1, queens and jacks less (one ace is worth about a point
more than four HCP of queens and jacks).

**The threshold is the one we already use.** Pooled over the six
positions, BBA's slam rate by my tp plus partner's floor:

| tp + partner's floor | 28 | 29 | 30 | 31 | 32 | 33 | 34 | 35 | 36 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| BBA bids slam | 3% | 12% | 20% | 41% | 57% | 75% | 84% | 90% | 96% |

The 50% point is 31-32 in every position (my tp at BBA's switch,
partner's floor: 17+15, 24+7, 15+16, 10+21, 17+14, 19+13). Our
`slam_values` is 32 on partner's floor. Counted in HCP instead, the curve
is wider (50% at 29, LL -0.341 against -0.311 pooled without
intercepts). Controls at a fixed count move the rate by about one point
of count per five or six controls, which is within the noise of a rule.

Scoring and vulnerability: IMPs bid slightly more slams than matchpoints
(23 more of 800 hands in `1NT 3S 4S`, 17 in `1D 1H 4H`, 7 or fewer
elsewhere); vulnerability changed nothing (`1D 1H 4H` at IMPs, none vs
both vulnerable: identical).

**Without a fit** (6NT): HCP is the measure. Over `1C 1D 2NT` (opener
18-19) BBA bids 6NT from 15 HCP (30/37) and often at 13-14 (47/117),
i.e. 31-33 combined, and bids a suit rather than 6NT with an unbalanced
hand; **we bid 3NT on all 800 hands**. Over a 2NT opening (20-21) we
already agree (6NT at 12-17 on balanced hands, a point more eager than
BBA).

So what we lack is not the count but **a place where the count is
asked**: with no ask convention on the card, nothing past game reads
`slam_values` (the keycard asks are card-gated), and responder's 3NT
over a 2NT rebid never looks at slam.

### Other findings

- **4NT on this card** is natural: over `2C P 2D P` "balanced, 28-30
  total points" (then 6NT or pass), over `1D P 1H P 3N P 4D P` "calculated
  bid, 18-21" (a sign-off below 6D). It is never an ask.
- **`1S 2D 3C 3S`**: BBA reads responder's 3S as **13-29 with three
  spades** (forcing, slam-interested; 4S would be 11-12) and opener never
  passes it (4S 447, 6S 228, 3NT 106 of 800). We pass it on every hand.
- **`1H 3H`**: BBA's bare SAYC never makes this jump raise on the vanilla
  set, and over a forced 1H-3H it passed with 13-21 (658 of 800): it
  reads 3H as weak, unlike our vanilla card (Rick's ruling: limit raise).
  No slam probe there.
- **The 2♣ opening** (`probes/slam-open-2c.toml`, 600 hands of 18-24
  HCP with a five-card suit): BBA opens 2♣ on every 22+; with 18-21 it
  opens 2♣ mostly with a six-card suit, on half the hands with 22 total
  points (HCP + length; 11 of 23, 10 of 22 with four losers or fewer)
  and on 9 of 11 with 23, rarely with 5-4 or 5-5 below 23. We require 23
  total points and four losers (strong-openings.bid), so BBA opens 2♣ a
  point lighter with a six-card suit: `KQT875.A.AKQ.Q82` (20 HCP, 22
  total) is one of those. strong-openings.notes.md measured 23 as better
  on the corpus; not changed here.

Probes (card `bare:sayc` for BBA; `our` = the vanilla settings), all
made with `probes/gen_hands.py ... --card bare:sayc --max-major 13
--min-len 0 --max-len 13 --scoring MP IMP --partner-any-shape` and the
arguments in each header: `probes/slam-1N-3S-4S-resp.toml`,
`slam-1S-2S-opener.toml`, `slam-1D-1H-4H-resp.toml`,
`slam-2C-2H-3H-resp.toml`, `slam-1H-2D-3H-resp.toml`,
`slam-1S-2D-3C-3S-opener.toml`, `slam-2N-resp.toml`,
`slam-1C-1D-2N-resp.toml`, `slam-open-2c.toml`. The measures:
`probes/tools/slam_measures.py slam-1N-3S-4S-resp:S:15
slam-1S-2S-opener:S:7 slam-1D-1H-4H-resp:H:16 slam-2C-2H-3H-resp:H:21
slam-1H-2D-3H-resp:H:14 slam-1S-2D-3C-3S-opener:S:13`.

## Phase B: slam without an ask (2026-09-30)

**Direct slam in an agreed suit** (`direct_slam_values`, slam-entry.bid):
on a card with no Blackwood and no RKCB (`no_ask`), with a suit agreed
and the opponents silent, bid six when our support points on partner's
floor plus my controls make 35, with 29 or more in support points. It
ranks below every descriptive call and above base.bid's "Game reached"
pass. Cards that ask are untouched: the corpus (-91,712) and the 21GF
random set (-43,518) are identical before and after.

Calibration on the vanilla set (IMPs vs BBA, double-dummy par; base
-36,678; even/odd boards as a check):

| variant | IMPs | even / odd |
|---|---:|---:|
| tp on floor >= 31 | +1,413 | |
| tp >= 32 | +1,494 | +794 / +700 |
| tp >= 33 | +1,203 | |
| tp >= 32 and no two bare suits | +1,459 | |
| tp >= 31, tp + controls >= 38 | +1,541 | |
| tp >= 30, tp + controls >= 37 | +1,962 | |
| tp >= 30, + controls >= 36 | +2,124 | |
| tp >= 30, + controls >= 35 | +2,101 | |
| tp >= 29, + controls >= 36 | +2,240 | |
| **tp >= 29, + controls >= 35** | **+2,325** | +1,454 / +871 |
| tp >= 28, + controls >= 35 | +2,340 | |
| tp >= 29, + controls >= 34 | +1,652 | |

Controls carry more in our rule than in BBA's choice (Phase A), because
the count is on partner's *floor*: a hand with aces and kings is the one
that can afford to count partner at his minimum. The bare-suit guard
cost a little.

| class (vanilla) | before | after |
|---|---:|---:|
| all boards | -36,678 (-0.38/bd) | -34,353 (-0.36/bd) |
| uncontested, slam par (4,7xx boards) | -15,456 (-3.26/bd) | -11,398 (-2.42/bd) |
| competitive, slam par (2,6xx) | -6,367 (-2.41/bd) | -5,890 (-2.21/bd) |
| our slams (make double dummy) | 593 (81%) | 1,396 (75%) |

BBA bids 4,228 slams (71%). The biggest gains: `1S 2C 2D 3S -> 6S` (36
boards, +204), `2C 2N 3H 4H -> 6H` (+154), `1D 1S 4S -> 6S` (+137),
`1H 2D 2H 3H -> 6H` (+124), `1D 1H 4H -> 6H` (+120); the worst,
`1S 2H 2S 3S -> 6S` (30, -15). No new problems, no new misread calls.

## How well a count predicts BBA (2026-09-30)

Rick asked how often support points plus controls predict BBA's slam
call. On the six Phase A grids (4,800 made hands), scoring each threshold
on hands it was not fitted to (5-fold):

| predictor | agrees with BBA |
|---|---:|
| BBA's usual call in the position, no count | 78.4% |
| `direct_slam_values` as built (tp 29+, +controls 35) | 82.4% |
| tp + partner's floor >= 32, one threshold | 86.2% |
| best threshold per position: HCP / tp / tp + controls | 85.0 / 87.0 / 87.7% |
| Zar points / 5-4-3-2-1 / losing tricks | 87.2 / 84.3 / 82.9% |
| logistic / boosted trees over every card | 88.1 / 89.0% |

No count gets past about 89%. The rule as built agrees less than the
plain 32 threshold, but par prefers it (+2,325 against +1,494): it was
tuned on par, not on agreement.

The rest is not noise. Over 1NT-3S-4S, 77 borderline hands got the same
call from BBA twice, again with a different partner hand, and again after
a spot-card change (0 of 77 changed). Single-card surveys show placement
mattering at the same HCP and shape: AKT765.74.A6.K97 passes, but moving
the ♠A to clubs or the ♠K to diamonds bids 6♠ (a top honour counts for
more outside trumps); a singleton heart bids 6♠ too; AJT54.A4.AJ42.95
bids 6♠ but passes without its fifth spade or any spade honour. A
"controls outside trumps" term did not help across the 4,800 hands, which
points to a suit-by-suit trick count against partner's shown holding
rather than a points formula. Next (Rick, option 3): reconstruct that
valuation with surveys, and test it against our count on par.

## BBA's count, reconstructed (option 3, 2026-09-30)

### What the BBA issues say

The 167 slam issues on github.com/EdwardPiwowar/BBA (public discussion of
behaviour; 128 opened by Thorvald Aagaard, the author Edward Piwowar
answering). Most reports are screenshots, so the text carries the
complaints and Edward's answers. **Edward's statements:**

- **A trick estimate, not a point count**: "BBA conducts a single dummy
  analysis before each bid. For this purpose, it simulates the cards of
  all players and checks likely tricks. Tricks are slightly adjusted
  according to additional criteria. Traditional losers are not counted.
  Keycards are only needed for rules." (#706, 2024-07-16; the opponents'
  bidding shapes their hands too). "Generally, bots count tricks" (#245,
  2024-02-26); "We have 9 HCP in hearts and only 2 tricks" (#638,
  2024-09-12).
- **Deterministic**: "BBA is purely rule-based and does not sample"
  (#297, 2024-03-27); no multiple deals or double dummy, for speed (#608,
  2024-08-23). The "simulation" is a constructed layout: a bug report
  shows it placing honours and lengths in the other hands ("the BOT added
  the king to void. He forgot to lengthen the suit", #178, 2024-01-09).
  Matches our test: partner's cards and spot cards never change the call.
- **A threshold on that estimate**: with a suit agreed the slam is a
  "calculated bid" (#416, 2024-04-28); "S has 11 tricks but he estimates
  that he will earn more, so he bids 6" (#363, 2024-04-17); "BBA will
  calculate 6 for a slightly different or slightly stronger hand" (#644,
  2024-06-25); "E needs 1 small point to slam try" (#839, 2024-09-10).
  Edward accepts borderline errors both ways ("It's impossible to avoid
  missed slams", #363).
- **Other counts**: an internal adjusted HCP and total points exist
  ("AdjustedHCP are not available to the manager", #1226; total points
  "each program counts them differently", #1240, both 2025-04-11); voids
  are counted "in a rather complicated way" (#656, 2024-07-01); honours
  in partner's suits are hard to discount (#1300, 2025-05-09).
- **Notrump and grand slams**: 6NT needs 33 HCP between the hands, on
  partner's maximum ("11HCP+21HCP<33HCP", #1000, 2024-11-12); a grand
  "usually when counts 13 tricks" (#1236, 2025-04-10), or on the odds of
  a missing king (#1163, 2025-03-10).
- **Asking**: the weak hand asks only with a clear slam (#55,
  2023-11-29); no 4NT invitation when the count already says slam (#446,
  2024-05-05). Matchpoints are BBA's default scoring (#225).

**Testers' requests** (not BBA's rules): brakes on minimum hands and on
the limited hand driving to slam (Thorvald #799, #723, #739, #363, #1291);
wasted values opposite a singleton (#1041, after which Edward made BBA
"less optimistic in this situation"); a grand only on 13 counted tricks
(#89); a splinter count with the ace of the short suit at 0.8, its king
0.3, its queen and jack 0 (ThePokerDude, #644).

### Placement surveys

`probes/slam-var-*.toml` (`probes/tools/slam_count.py variants`): in each
of the six Phase A positions, 40 borderline hands (BBA bids slam on
10-90% of the hands with the same count), half bidding slam and half
not, each with every variant that moves one honour to another suit (same
HCP and shape) or one spot card to another suit: 4,982 hands.
`slam_count.py moves` gives how often each move changes BBA's call (+1:
the variant bids slam and the base did not):

| move | effect |
|---|---:|
| an ace or king into or out of trumps, or between side suits | -0.04 to +0.01 |
| the trump queen to a side suit / a side queen into trumps | **-0.21 / +0.08** |
| the trump jack to a side suit | -0.09 |
| a trump to a side suit, leaving four trumps | **-0.19 to -0.43** |
| a side card into five trumps (a doubleton becomes a singleton) | +0.38 |
| a side suit's spot to a singleton (singleton gone) | -0.21 to -0.42 |

So an ace or a king is worth the same in trumps as outside (the
AKT765.74.A6.K97 case above is local, not a rule); the trump queen and
jack are worth more than side ones; trump length and shortness count.

### The count

Fitted on the 4,800 random hands (a logistic on card features plus
partner's floor), in points of partner's floor: an ace 3.9, a king 2.7,
a queen 1.3 and a jack 0.4 outside trumps (in trumps 1.75 and 0.85; a
point of HCP is 0.9), a void 2.1, a singleton 1.3 and a doubleton 0.2 (a
singleton king nothing extra), each side card beyond four 0.8. In the two positions with a
partner's suit, an ace there is worth about 1.8 and a king 1.2, a
singleton there 1.6 (3.9 elsewhere). Rounded to half points:

> HCP, plus half a point for each ace, minus half a point for each queen
> and jack outside trumps; plus 2 for a void, 1 for a singleton other than
> the king, 1 for each card beyond four in a side suit. Slam from 30 with
> partner's floor.

It agrees with BBA a little more than support points do, and the rest is
not reachable by any count (`slam_count.py agree`; 5-fold, one threshold
on count + BBA's floor; the variants never fitted):

| count | random hands | placement variants |
|---|---:|---:|
| `direct_slam_values` (tp 29+, + controls 35) | 82.4% | 65.6% |
| HCP | 85.0% | 67.6% |
| support points (tp) | 86.2% | 71.5% |
| **reconstructed count** | **87.7%** | **72.8%** |
| a free logistic / boosted trees over every card | 88.1-88.5 / 89.0% | |

The ceiling is the construction Edward describes: how my honours combine
with the cards it gives partner is an interaction no per-card count holds.

### On par: the built rule stays

Written in the engine's terms (`keycards(N)` for the aces, conditions as
numbers for the side queens and jacks, shortness and length, in half
points) as an alternative to `direct_slam_values`, on our floor
`partner.tp(trump).min`. Vanilla set, IMPs vs BBA against the built rule
(base -33,895; even / odd boards):

| variant | IMPs | even / odd |
|---|---:|---:|
| reconstructed count, 28 / 29 / 30 / 31 | -3,035 / -946 / **-570** / -808 | (30) -271 / -299 |
| reconstructed 28 or 29, plus controls 35 | -240 / -297 | |
| tp with the card corrections (A +½, side Q/J -½), + controls 35 | -203 | |
| the same, + controls 35½ | -43 | +13 / -56 |
| built, minus a side suit of three small | -4 | |
| built, plus side length | -54 | |
| built, plus the trump queen | -285 | |
| built, shortness in partner's four-card suit not counted | -49 | |

Corpus (-91,400) and the 21GF random set (-43,426): unchanged by the
reconstructed count, as by the rule (no ask on those cards).

**Par prefers the built rule**, so it stays: BBA's count values shortness
less than support points do and controls less than our rule does, and
both cost on par. Agreement with BBA is not what limits us here: on the
grids, our own calls agree with BBA's on 76.0% of the hands with the
built rule and 74.8% with the reconstructed count, because what differs
most is **partner's floor as we read it** (after `1D 1H 4H` we count
opener at 19, BBA at 16; after `2C 2H 3H` 17 against 21) and **where no
suit is agreed** (`1H 2D 3H`, `1S 2D 3C 3S`: the direct slam never
applies in our auction).

### Open (for Rick)

- Partner's floor: should `partner.tp` after a jump to game over a
  one-level response (19) and after 2C-2H-3H (17) move toward BBA's 16
  and 21? It moves every slam decision in those auctions, not only this
  rule.
- A direct slam where the fit is shown but not agreed (`1H 2D 3H`, opener's
  six-card suit): `shown_fit(x)` as the 4NT ask already uses.
- The construction itself (partner's expected cards from his shown
  ranges, then a trick count) is beyond the rule language; worth it only
  if a measured position shows the count is what loses.

## Sources

- Rick's rulings, as recorded in rkcb-1430.bid ("The ask", the 33-point
  test and no two bare suits, 2026-09-23) and control-bids.bid (the
  control treatment and first-or-second-round controls, 2026-09-25);
  slam-catch.notes.md for the 33 support-point threshold.
- The 18-point fallback facing a game force: rkcb-1430.bid, "The ask".
- Combined 33 points for a small slam: standard practice, not yet cited.
- BBA's slam decision, measured by made hands on the bare SAYC card
  (2026-09-30, "Phase A" above, the `probes/slam-*.toml` specs and
  `probes/tools/slam_measures.py`): support points on partner's floor,
  50% at 31-32; the vanilla random set (95,558 deals) for where it
  matters.
- BBA's method, in its author's words (Edward Piwowar, public issues on
  github.com/EdwardPiwowar/BBA, read 2026-09-30): a deterministic single
  dummy trick estimate over constructed hands (#706, #297, #608, #178),
  slam as a calculated bid on a threshold (#416, #363, #644, #839), 33 HCP
  for 6NT (#1000), a grand on 13 counted tricks (#1236). Testers'
  requests from the same issues (Thorvald Aagaard, ThePokerDude: #799,
  #723, #1041, #89, #644) are marked as theirs above. Behaviour only; no
  BBA code was read.
- The reconstructed count and its agreement with BBA: placement surveys
  `probes/slam-var-*.toml` and `probes/tools/slam_count.py` (2026-09-30);
  par on the vanilla set, the corpus and the 21GF random set (same date).
  Par kept the built rule.
