# penalty-doubles (`penalty-doubles.bid`): notes

Phase 3 of docs/JUDGMENT-LAYER.md: a penalty double as a judgment rule
keyed on auction state, not on a sequence, so it keeps working as
conventions are added. First version 2026-10-01.

## The rule

When the contract is theirs and undoubled (RHO has just bid, or it has
been passed round to me), at the two level or higher, I double for
penalty when, with `we.hcp.min` my HCP plus the bottom of partner's
shown range:

| my holding in their suit | we.hcp.min |
|---|---|
| four or more with two of the top three or three of the top five (Rick's standard, 2026-09-23) | 20 |
| three or more | 21 |
| anything | 22 |
| their notrump (after they have both bid) | 21 |

Only where it is a contract:
- their suit is natural (`they.fit(x).min >= 4`: not Stayman, a strong
  2♣, a transfer, or an overcall the engine cannot read) and not one we
  have shown (a cue bid);
- it will stand: passed round to me, or they have a fit (seven or more
  shown), or they have bid game. Directly over a new suit the auction
  may be forcing;
- partner has shown values (`partner.hcp.min >= 4`): with partner
  silent, a strong hand with their suit doubles for takeout first (the
  reopening doubles over a preempt read the double so);
- not over partner's notrump opening (one bid, a range of four points
  or less): they came in with shape against a balanced hand;
- not responder's first call over their overcall (the double is
  negative there).

Priority -6: above the silence passes (-10, -20), below every sequence
rule and the LoTT (-4). It sets `ask=penalty`; partner sits (priority
-1, so a sequence pass or bid does not answer it instead).

## Diagnosis (vanilla SAYC random set, main e10a172, 2026-10-01)

Every undoubled final contract by the opponents in our auctions, from
each defender's seat: what a double would have gained (double dummy, no
runout, IMPs). Suit contracts, two level and up, by the actual combined
HCP and the doubler's length in their suit (boards / IMPs / per board):

| combined | 2 trumps | 3 trumps | 4 trumps | 5+ trumps |
|---|---|---|---|---|
| 20 | 2,663 / −13,448 / −5.05 | 2,098 / −8,508 / −4.06 | 889 / −482 / −0.54 | 182 / +741 / +4.07 |
| 22 | 1,598 / −3,956 / −2.48 | 1,205 / −1,173 / −0.97 | 522 / +1,454 / +2.79 | 109 / +774 / +7.10 |
| 23 | 1,036 / −660 / −0.64 | 771 / +703 / +0.91 | 314 / +1,401 / +4.46 | 51 / +430 / +8.43 |
| 24 | 526 / +732 / +1.39 | 381 / +878 / +2.30 | 143 / +774 / +5.41 | 32 / +258 / +8.06 |
| 25 | 257 / +689 / +2.68 | 178 / +766 / +4.30 | 60 / +460 / +7.67 | 11 / +95 / +8.64 |

With four, two of the top three honours were worth about a point
(combined 22+: four small +3.49 a board, good four +5.47).

The combined count is not known at the table. A sample of 15,000 of
these positions (of 45,675) was replayed through `rbb call` to read
what each seat had shown: partner's shown range is wide (a pass of
their opening is 0-30), so the *known* minimum is far below the real
count, and the midpoint of partner's range is worse (it double counts
an unlimited pass: a first try with it lost 14,446 IMPs from par).
Doubles keyed on the known minimum are the ones that survive.

BBA leaves 4,574 final contracts doubled on these boards (we left
684): most are takeout or negative doubles passed for penalty (655 at
`1a X`, 242 at `1a 1b X`), and its own direct penalty doubles cluster at
the three and four level. BBA's issue #448 (github.com/EdwardPiwowar/BBA):
"if we have 13+ HCP and partner opened we either play ourselves or we
double the opponents".

## Measured (2026-10-01)

Both yardsticks against main, plus `probes/tools/penalty_x.py`, which
splits the doubled boards by whether the doubler's side ended above par
and counts the doubler's shortfall from par only ("below par"): distance
from par counts a double that collects more than par (they overbid, we
punish it) as a loss.

Vanilla SAYC random (95,558 deals, both halves):

| version | par vs BBA | side IMPs | doubles | below par |
|---|---|---|---|---|
| min-based, 20/22/24/25 by length, -6 | −78 | +767 | 146 | |
| midpoint of partner's range, -3 | −14,446 | −23,794 | 5,523 | |
| + trump stacks at 8-16 (5 cards, good four) | −2,416 | +2,459 | 1,021 | +103 |
| 4 any at 19, 3 at 21, any at 23 | −97 | +1,440 | 412 | +536 |
| + four small at 20, good four and five at 17 | −454 | +1,664 | 507 | +465 |
| good four at 19, 3 at 21, any at 23 | +85 | +1,269 | 325 | +553 |
| 3 at 20, any at 22 | −212 | +1,337 | 590 | +434 |
| good four at 19, 3 at 21, any at 22 | +137 | +1,450 | 431 | +663 |
| any at 21 | −38 | +1,575 | 596 | +597 |
| **good four at 20, 3 at 21, any at 22** | **+191** | **+1,434** | 409 | +680 |
| the same at priority -5 | +104 | +1,495 | 472 | +673 |
| notrump at 17 instead of 21 | −721 | +2,060 | 694 | |
| + partner has shown values | +191 | +1,322 | 375 | +638 |
| **+ not over partner's notrump opening (committed)** | **+196** | **+1,315** | 348 | +622 |

The notrump exclusion was found on the 21GF set (below: 69 boards after
our 1NT, −285 par, −42 side) and confirmed on vanilla and the corpus.
With the partner-shown condition, 375 vanilla doubles: 315 went down
(117 two down, 67 three), 60 made; the doubled side ran on 20 boards.

The committed rules on all three sets (against main; par vs BBA and
side IMPs, each with its even / odd halves; "below par" and the split
from `penalty_x.py`):

| set | par vs BBA | side IMPs | doubles | par: above / at-or-below | below par |
|---|---|---|---|---|---|
| vanilla SAYC random | +196 (+83 / +113) | +1,315 (+687 / +628) | 348 | −101 / +319 | +622 |
| 21GF random (100,000) | +152 (+26 / +126) | +1,929 (+973 / +956) | 493 | −239 / +413 | +885 |
| full corpus (161,477) | +157 (+105 / +52) | +4,256 (+1,923 / +2,333) | 900 | −600 / +831 | +1,883 |

par_blame, main → committed (IMPs from par; BBA in brackets):

| set | W let them play (no double) | W doubled, par was to bid on |
|---|---|---|
| vanilla | 6,614 / 41,325 → 6,378 / 39,144 (4,373 / 23,481) | 185 / 1,174 → 355 / 2,457 (1,011 / 6,341) |
| 21GF | 7,439 / 49,343 → 7,120 / 46,522 (4,732 / 25,643) | 222 / 1,423 → 443 / 3,003 (1,082 / 6,834) |
| corpus | 9,139 / 65,220 → 8,530 / 59,678 (5,633 / 32,825) | 509 / 3,480 → 924 / 6,640 (2,147 / 14,911) |

The guard roughly doubles but stays under half of BBA's. Most of the new
guard boards were "let them play" before: we hold the balance, they are
in a partscore or a sacrifice, par is our game or slam, and the double
collects part of it instead of nothing (the "let them play" drop is
larger than the guard's rise on every set).

"Calls read as a higher rule": corpus 20,029 → 20,078, 21GF 14,719 →
14,729. Most are two-suited-overcalls.bid's own penalty double (-19)
read as this one (-6), the same meaning.

## For Rick

1. **Which yardstick for penalty doubles.** Distance from par cannot
   reward a double of an overbid: when the opponents go past par, our
   pass already beats par and the double "moves away" from it. Side IMPs
   play our engine on both sides, and the doubled side rarely runs.
   `penalty_x.py`'s "below par" counts only the doubler's shortfall.
   With the committed rules all three agree on every set; on the way,
   variants split exactly on the "doubler above par" boards (par loses
   there by construction). Should penalty-double work be judged by side
   IMPs plus "below par", with par distance only as a guard?
2. **Trump stacks at lower strength.** Five trumps or a good four with
   8-16 known between us gain double dummy (+2,459 side) but lose par
   (−2,416), much of it the artefact above; left out.
3. **Four small.** With 22+ between us four small gains (+3.49 a board,
   oracle); the rule counts it as "anything at 22". Your 2026-09-23
   ruling was about responder's double after our redouble.
4. **Partner's shown range.** The known minimum is the binding limit:
   a pass of their opening shows 0-30 in the engine. Narrowing passes
   (no overcall, no double: about 0-11 or so) would let the rule fire
   far more often; that is engine knowledge, not this module.
5. **Penalty-double style** (JUDGMENT-LAYER.md question 6) is still
   open: this rule doubles only where no other double is defined, so it
   does not need the hierarchy yet.

## Sources

- **Standard practice, not yet cited:** double when they are too high
  and the balance of strength is ours; trump length and honours behind
  or in front of the declarer; the combined-strength thresholds are
  calibrated here, not quoted.
- **Rick's rulings:** the penalty-double holding, four or more with two
  of the top three or three of the top five (2026-09-23,
  after-interference.bid trap pass, responder-rebids.bid); thresholds in
  the rules as named conditions (JUDGMENT-LAYER.md decisions, 2026-09-30).
- **BBA evidence:** its doubled final contracts on the vanilla random
  set (above); issue #448 on not letting them play undoubled.
- **Measurements:** the tables above, from `rbb compare` on
  Random_Pavlicek_SAYCvanilla with the vanilla `--set` list, the
  `rbb call` replay of 15,000 positions, `probes/tools/sideimps.py`,
  `probes/tools/par_blame.py` and `probes/tools/penalty_x.py`.
