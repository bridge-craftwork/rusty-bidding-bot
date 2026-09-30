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

## Sources

- Rick's rulings, as recorded in rkcb-1430.bid ("The ask", the 33-point
  test and no two bare suits, 2026-09-23) and control-bids.bid (the
  control treatment and first-or-second-round controls, 2026-09-25);
  slam-catch.notes.md for the 33 support-point threshold.
- The 18-point fallback facing a game force: rkcb-1430.bid, "The ask".
- Combined 33 points for a small slam: standard practice, not yet cited.
