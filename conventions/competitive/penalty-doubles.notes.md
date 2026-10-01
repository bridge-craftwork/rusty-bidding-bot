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
- not over partner's notrump opening (one bid, a range of four points
  or less): they came in with shape against a balanced hand;
- not responder's first call over their overcall (the double is
  negative there).

Partner need not have shown values: a condition that he had
(`partner.hcp.min >= 4`) was dropped on 2026-10-01, when the errors
yardstick found it a par artefact (below, "Re-judged by errors").

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

## Re-judged by errors (2026-10-01)

Rick's errors yardstick (`compare`'s "vs BBA, errors" line, PR #29):
each side charged with its own errors, a contract that goes down
undoubled and beats par that way taken as doubled. Every variant below
was measured against main 0717d3f on the vanilla set and split with
`probes/tools/errors_diff.py` (positive: the variant makes fewer
errors). Columns: boards changed, the errors line with its even / odd
halves, per board, its contract and doubling parts, then distance from
par and side IMPs (third opinion only).

Main on vanilla: by distance −29,290; by errors −35,671 (contract
−31,517, doubling −4,154).

| variant (vanilla) | boards | errors (even / odd) | per bd | contract | doubling | par | side |
|---|---|---|---|---|---|---|---|
| the rule off (value of the committed rule, negated) | 358 | −1,002 (−499 / −503) | −2.80 | +92 | −1,094 | −196 | −1,315 |
| good four at 19 | 17 | +9 (+11 / −2) | +0.53 | −15 | +24 | −30 | +48 |
| good four at 21 (= three at 21) | 18 | +19 (+34 / −15) | +1.06 | +19 | 0 | +36 | −25 |
| three at 20 | 148 | −296 (−64 / −232) | −2.00 | −75 | −221 | −330 | −146 |
| three at 22 (= no three-card clause) | 77 | −106 (−45 / −61) | −1.38 | +30 | −136 | −10 | −186 |
| anything at 21 | 116 | −98 (−124 / +26) | −0.84 | −50 | −48 | −155 | +57 |
| anything at 23 | 86 | −152 (−58 / −94) | −1.77 | −8 | −144 | −66 | −201 |
| "anything" at 22 needs two trumps | 106 | −315 (−171 / −144) | −2.97 | −11 | −304 | −117 | −336 |
| good four = two of the top three only | 2 | −12 (−5 / −7) | | 0 | −12 | −1 | −12 |
| any five counts as good | 1 | −8 | | −8 | 0 | −8 | +12 |
| four small at 20 | 35 | −20 (−27 / +7) | −0.57 | −42 | +22 | −74 | +61 |
| **stacks:** five at 19 | 7 | −4 (−8 / +4) | | −24 | +20 | −22 | +51 |
| five at 18 | 25 | −70 (−42 / −28) | −2.80 | −88 | +18 | −98 | +131 |
| five at 16 | 66 | −245 (−105 / −140) | −3.71 | −174 | −71 | −254 | +176 |
| five or good four at 16 | 134 | −349 (−190 / −159) | −2.60 | −255 | −94 | −456 | +268 |
| five at 12, good four at 16 | 320 | −1,046 (−578 / −468) | −3.27 | −520 | −526 | −819 | +289 |
| five or good four at 8 | 443 | −1,495 (−818 / −677) | −3.37 | −672 | −823 | −942 | +320 |
| **notrump** at 17 | 284 | −661 (−406 / −255) | −2.33 | −535 | −126 | −910 | +629 |
| notrump at 19 | 52 | −108 (−52 / −56) | −2.08 | −123 | +15 | −170 | +169 |
| notrump at 20 | 10 | +1 (+4 / −3) | | −19 | +20 | −31 | +47 |
| notrump at 22 | 1 | −5 | | 0 | −5 | −5 | −5 |
| **exclusions:** partner's notrump allowed | 38 | +2 (−9 / +11) | +0.05 | −30 | +32 | −5 | +7 |
| "will stand" dropped | 66 | +10 (+22 / −12) | +0.15 | −90 | +100 | −160 | +290 |
| "will stand" without the fit clause | 84 | −134 (−112 / −22) | −1.60 | +19 | −153 | +27 | −200 |
| "will stand" with a fit of eight | 14 | −49 (−50 / +1) | | −9 | −40 | −15 | −54 |
| **partner need not have shown values (kept)** | 39 | **+94 (+7 / +87)** | +2.41 | −12 | +106 | 0 (−5 / +5) | +112 |
| kept + good four at 19 | 68 | +112 (+24 / +88) | +1.65 | −38 | +150 | −53 | +164 |
| kept + good four at 21 | 52 | +104 (+51 / +53) | +2.00 | +10 | +94 | +42 | +84 |
| kept + partner's notrump allowed | 77 | +96 (−2 / +98) | +1.25 | −42 | +138 | −5 | +119 |
| kept + "will stand" dropped | 118 | +97 (+36 / +61) | +0.82 | −116 | +213 | −199 | +420 |
| kept + anything at 21 | 198 | +65 (−81 / +146) | +0.33 | −53 | +118 | −146 | +260 |
| kept + notrump at 20 | 49 | +95 (+11 / +84) | +1.94 | −31 | +126 | −31 | +159 |
| kept + good four at 21 + notrump at 20 | 62 | +105 (+55 / +50) | +1.69 | −9 | +114 | +11 | +131 |
| kept + stacks at 16 | 334 | −434 (−307 / −127) | −1.30 | −513 | +79 | −864 | +550 |

What it says:
- **The rule survives the new yardstick.** Switching it off costs
  1,002 IMPs on vanilla (both halves), 1,493 on 21GF and 2,796 on the
  corpus, almost all doubling errors (the opponents' sacrifices and
  overbids left undoubled).
- **The thresholds hold** (20 / 21 / 22, notrump 21): every move of
  one point either way loses or splits the halves. With "partner has
  shown values" dropped, good four at 21 instead of 20 gains a little
  on vanilla (+10) but loses on 21GF (−29) and the corpus (−145).
- **The trump conditions hold**: Rick's standard as it is; narrowing
  it to two of the top three, or widening it to any five, changes a
  handful of boards and loses.
- **"Partner has shown values" was a par artefact**: dropping it is
  level by distance on vanilla (0) but +94 by errors, +106 of it
  doubling errors, and it gains on 21GF and the corpus: dropped. The new doubles are big hands, 20+ HCP in my own
  hand, doubling their preempt or game directly or doubling again after
  my takeout double was run from; partner sits.
- The other two exclusions stay: allowing partner's notrump, or
  dropping "the contract will stand", does not gain on both halves.

A caution on reading the split: our engine plays both sides, so when a
double collects more than the undoubled contract, the doubled side's
contract error grows by the same move (an overbid that was already
worse than par undoubled is not taken as doubled). That is the
"contract" loss in the stack and notrump rows. The doubler's own errors
alone tell the same story there: five or good four at 8 adds 823 IMPs
of doubling errors to the doubler; notrump at 17, 126.

Kept on the three sets (main → kept; boards where the auction changed,
the errors line with halves, per board, contract / doubling):

| set | by distance | by errors (contract, doubling) | change by errors | boards | per bd | contract / doubling | par / side |
|---|---|---|---|---|---|---|---|
| vanilla SAYC random | −29,290 → −29,290 | −35,671 (−31,517, −4,154) → −35,577 (−31,529, −4,048) | +94 (+7 / +87) | 39 | +2.41 | −12 / +106 | 0 / +112 |
| 21GF random | −43,000 → −42,940 | −51,356 (−43,857, −7,499) → −51,082 (−43,861, −7,221) | +274 (+110 / +164) | 63 | +4.35 | −4 / +278 | +60 / +247 |
| full corpus | −90,676 → −90,583 | −100,458 (−93,859, −6,599) → −99,999 (−93,853, −6,146) | +459 (+249 / +210) | 120 | +3.83 | +6 / +453 | +93 / +429 |

Our doubling errors move toward BBA's, not past them: we still make
4,048 (vanilla), 7,221 (21GF) and 6,146 (corpus) IMPs more doubling
errors than BBA (with the rule off: 5,248, 9,135, 9,945).

## For Rick

1. **Which yardstick for penalty doubles.** Answered by the errors
   yardstick (2026-10-01): penalty-double work is now judged by it.
   It and distance from par agree on every kept change above; they
   split on the stacks and notrump at 17, where both say no.
   One thing to look at: in our self-play a double of an overbid that
   was already worse than par undoubled moves the doubled side's
   contract error by the same amount it gains (see the caution above).
   Should the assumed result take *every* failing undoubled contract
   as doubled, not only those that beat par that way?
2. **Trump stacks at lower strength** (+2,459 side, −2,416 par before):
   by errors they lose at every strength tried, five at 19 down to
   five-or-good-four at 8 (−4 to −1,495 IMPs, both halves); the
   doubler's own doubling errors grow too. Left out.
3. **Four small.** Four small at 20 loses (−20, halves split); the rule
   still counts four small as "three or more" at 21.
4. **Partner's shown range.** Still the binding limit (a pass of their
   opening shows 0-30 in the engine); engine knowledge, not this module.
   Dropping "partner has shown values" (kept) is the module's side of it.
5. **Their notrump from 17** (−910 par, +667 side before): −661 by
   errors (−406 / −255), 535 of it contract errors; 19 loses too, 20 is
   level (+1), 22 loses. It stays at 21.
6. **Penalty-double style** (JUDGMENT-LAYER.md question 6) is still
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
  `probes/tools/par_blame.py`, `probes/tools/penalty_x.py`, and
  `probes/tools/errors_diff.py` for the re-judgment by errors
  (Rick's yardstick, 2026-10-01).
