# Jacoby 2NT (`jacoby-2nt.bid`)

The game-forcing raise of a major (`major_openings.jacoby_2nt.play`),
and opener's standard answers: 3x a singleton or void, 4x a good
five-card side suit, 3M 18+, 3NT 15-17, 4M a minimum, all without
shortness. Modified Jacoby has its own answers
(modified-jacoby-2nt.bid, `modified`). Cases: `jacoby-2nt.test`.

## Responder after the shortness (2026-11-07)

Rick, 2026-11-07 (bridge-classroom #83/#84, of modified Jacoby, "and
the standard jacoby 2nt, and splinter bids"):

> "keycard should be the stronger hand. the idea of the relays is it
> will allow us to find low HCP slams when there is a good fit. one of
> the hands had 4-5 weak spades in responder's hand, and opener had a
> spade void, i think, which means all the combined HCP are 'working',
> which is the goal of this system (and the standard jacoby 2nt, and
> splinter bids). this may be a concept we haven't introduced yet."

Over 3x, responder counts his working values (slam-entry.bid,
`working_slam(M)`: support points less his K, Q, J opposite the
singleton, plus opener's floor; 27 opposite a singleton, 23 opposite a
void, though standard Jacoby's 3x does not say which) and with them
asks for keycards (RKCB, else Blackwood). Without them the slam modules
decide as before (control bids, game). Responder is the one who asks:
opener's strength is still open (10-21), responder's is too, and
responder is the one who knows where the waste is.

**No explicit sign-off.** A 4M "game: honours wasted or too few" below
the ask lost opener's extras: opener, who has not limited his hand,
did not go on over it. Measured (compare `--par-all`, the 5,000-board
Jacoby and splinter set, standard card; errors_diff, the bidding side's
contract errors): with the sign-off −118 (26/28) and −726 (30/31);
without it **+99 IMPs** (halves +46 / +53, agree; distance from par
+99, 209 boards changed). "Short of slam" 1,021 → 981 boards
(12,559 → 12,051 IMPs); "overbid" 433 → 461 (6,111 → 6,524).

## Opener after responder's 4M: not the 19+ pass (2026-11-07)

Rick's ruling of 2026-11-07 is about modified Jacoby (quoted in
modified-jacoby-2nt.notes.md): "normally opener doesn't continue for
slam exploration after responder chooses not to try, unless opener has
19+ points". It is **not applied to standard Jacoby**: here opener's
shortness answer (3x) is unlimited, 10-21, so responder's 4M is not a
decline opposite a limited hand, and opener with extras still goes on
through the slam modules (rkcb-1430's ask on `slam_values`) as before.
Tried and reverted the same day: the 19+ pass here cost −887 IMPs on
the 5,000 Jacoby/splinter boards (contract errors, halves −307 / −580;
opener's 17-18 slams, `P 4H P : 4N -> P` −675 on 207 boards).

Rick's answer (2026-11-07): "for standard jacoby, survey bba to see
where it decides to go on."

### BBA's line over responder's 4M (survey, 2026-11-07)

Grids: `probes/jacoby-opener-1S-answer.toml` and
`probes/jacoby-opener-1H-answer.toml` (1,000 opener hands each,
12-21 HCP evenly, half with a singleton or void, each with its own
responder: four trumps, 12-14 HCP balanced; 21GF-GIB, IMPs) give BBA's
own answer to 2NT; `probes/jacoby-opener-1{S,H}-<answer>-4{S,H}.toml`
then force 1M P 2NT P <answer> P 4M P with the same hands and ask
BBA's opener again (1,800 hands). To rerun, from the repo root:
`probes/jacoby_opener_gen.py probes/jacoby-opener-1S-answer.toml S 1000 11`
(H 1000 12 for hearts), `rbb grid` on each, `probes/jacoby_opener_stage_b.py S`
(and H) for the second-stage specs, `rbb grid` on those, then
`probes/jacoby_opener_tally.py [GRIDS_DIR]` for the tables below (our
engine's agreement after the change: the second-stage grids rerun with
`--out DIR` and the tally pointed at DIR). BBA's answers:
3x a singleton or void at any strength, 3NT 14-15 HCP, 3M 16+, 4x a
good side suit; 4M (a minimum) was left out.

| Answer | BBA goes on | Rate by HCP | Line that fits BBA best |
|---|---|---|---|
| 3x shortness | 463 of 863 | 12: 0%, 14: 21%, 16: 55%, 18: 83%, 20+: 100% | points + controls (A 2, K 1) ≥ 24: 84% agreement |
| 3M | 360 of 590 | 16: 5%, 17: 19%, 18: 57%, 19: 89%, 20+: 100% | points ≥ 20: 88% |
| 3NT | 9 of 199 | 14-15: 3-6% | pass |
| 4x side suit | 144 of 148 | 15+: 100% | go on (as we did) |

BBA's going-on call is 4NT (870 of 903), else a five-level cue. Our
opener before: 3x 76% agreement (went on with 15+ always), 3M 52%
(rarely went on), 3NT 89%, 4x 97%.

**Ours** (jacoby-2nt.bid, `jac_on_short`, `jac_on_3m`): over 3x and
3NT, pass with points + controls under 22; over 3M, under 19 points;
else the slam modules' call, with the keycard ask as a fallback
(priority −10, below exclusion and the control bids). Agreement with
BBA on the grids: 3x 76% → 84% (BBA's own 24 line in the rule: 84%,
22: 83%), 3M 52% → 87%, 3NT 89% → 95%.

**Par decides, BBA teaches** (CLAUDE.md; docs/DESIGN.md): the trade.
BBA's own line (points + controls ≥ 24 over 3x and 3NT, points ≥ 20
over 3M) costs **−526 IMPs** against double-dummy par on the 5,000
Jacoby/splinter boards (compare `--par-all`, standard card, 01177e6
against these; errors_diff, the bidding side's contract errors; halves
−175 / −351, agree; distance from par −512), all of it opener's passes
of slams that make. Ours sits two points lower: 22 over 3x and 3NT,
19 over 3M (our 3M shows 18+): **+60** (halves +118 / −58, differ;
distance from par +67; 101 boards changed), i.e. neutral against the
version before. In between, 23 cost −243 (agree); 21 was +12 (differ).
What we give up in agreement with BBA on the grids is small: 3x 83%
with 22 against 84% with BBA's 24 (76% before), 3M 87% with 19 against
88% with 20 (52% before), 3NT 95% (89% before). The going-on half
alone changed no corpus board. So we keep BBA's shape of the line
(points and controls) and par's threshold.

## Sources

- Jacoby 2NT: Oswald Jacoby's game-forcing raise; the answers as
  ACBL's SAYC card and the convention-card spec
  (`spec/conventions/bidding_conventions/jacoby_2nt.toml`) give them;
  standard practice, not yet cited beyond that.
- Working values after the shortness: Rick, 2026-11-07, quoted above;
  the threshold is our calibration (slam-entry.notes.md, "Working
  values").
- Opener over 4M: Rick's 19+ ruling (2026-11-07) is modified Jacoby's
  and not applied here (above); for standard Jacoby his answer "survey
  bba to see where it decides to go on" (2026-11-07). BBA's line from
  the grids `probes/jacoby-opener-*.toml`, made and tallied with
  `probes/jacoby_opener_gen.py`, `probes/jacoby_opener_stage_b.py` and
  `probes/jacoby_opener_tally.py` (behaviour only, bba-cli as a black
  box); our threshold set by par (the trade above).
