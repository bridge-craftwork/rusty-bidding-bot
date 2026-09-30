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

## Sources

- Rick's rulings, as recorded in rkcb-1430.bid ("The ask", the 33-point
  test and no two bare suits, 2026-09-23) and control-bids.bid (the
  control treatment and first-or-second-round controls, 2026-09-25);
  slam-catch.notes.md for the 33 support-point threshold.
- The 18-point fallback facing a game force: rkcb-1430.bid, "The ask".
- Combined 33 points for a small slam: standard practice, not yet cited.
