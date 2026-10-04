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
| four or more with two of the top three or three of the top five (Rick's standard, 2026-09-23) | 17 (20 until 2026-10-03) |
| four or more | 20 (2026-10-03) |
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

Two more doubles (2026-10-02, below "Doubling more"):
- **their game, my 17**: once they have bid game, 17 HCP in my own hand
  is enough on its own (any holding in their suit, and their notrump
  game too): they have 23 at most between them (`big_hand_vs_game`);
- **they bid over our game** (`our_game_bid`: my or partner's last call
  was 3NT, 4M or five of a suit, and they bid on at the four level or
  higher): 12+ in my hand or 20 known between us, and two or more of
  their trumps (`over_our_game(x)`). Priority -5, and also when partner's
  game was a signoff (`asked signoff`): the signoff's "nothing to add"
  pass (rebids.bid, -5) otherwise sold out to them.

And one penalty pass in responder-rebids.bid: responder leaves opener's
reopening double in with four of their suit only when they are good
(two of the top three, three of the top five), five long, or he has
8 HCP.

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

## Doubling more: what limits it (2026-10-02)

The task: we doubled 790 of 23,685 chances on the vanilla set (3.3%;
BBA 3,271 of 23,759, 13.8%) and made 10,550 IMPs more doubling errors
than BBA (main 8224d84). A chance is a contract above par that goes
down; the defenders' doubling error is what the undoubled result
leaves on the table. `probes/tools/double_chances.py RUN.json` makes
these tables (`--by level|vul|rule` for the others).

### Where the doubling errors are (vanilla, main)

By what the defenders did before the final contract: chances, doubled,
doubles of contracts that made, doubling errors (IMPs), and IMPs per
undoubled chance:

| defenders | ours: chances | x | made | errors | per undoubled | BBA: chances | x | made | errors | BBA - ours |
|---|---|---|---|---|---|---|---|---|---|---|
| silent, they bid game | 7,407 | 19 | 6 | 30,563 | 4.13 | 4,646 | 76 | 18 | 16,753 | -13,810 |
| one bid, partscore | 5,205 | 438 | 205 | 22,854 | 4.54 | 4,078 | 464 | 194 | 16,515 | -6,339 |
| silent, partscore | 5,340 | 79 | 39 | 22,117 | 4.15 | 4,501 | 146 | 71 | 18,130 | -3,987 |
| a defender doubled | 1,780 | 133 | 26 | 7,641 | 4.51 | 2,079 | 413 | 170 | 8,078 | +437 |
| they bid over our game | 190 | 23 | 6 | 920 | 5.31 | 1,106 | 758 | 211 | 2,619 | +1,699 |
| one bid, they bid game | 1,692 | 4 | 0 | 7,628 | 4.52 | 2,509 | 211 | 84 | 9,644 | +2,016 |
| both bid, no fit | 733 | 64 | 13 | 3,078 | 4.45 | 1,732 | 443 | 148 | 6,256 | +3,178 |
| both bid, a fit | 1,338 | 30 | 5 | 5,723 | 4.34 | 3,108 | 760 | 354 | 11,896 | +6,173 |
| total | 23,685 | 790 | 300 | 100,524 | | 23,759 | 3,271 | 1,250 | 89,891 | -10,633 |

What it says:
- **Most of the gap is not doubling.** Where the defenders never bid,
  neither engine doubles much (BBA 2.4% of those chances), but ours
  face 12,747 such chances to BBA's 9,147: our declaring side bids more
  failing games and partscores in uncontested auctions (7,407 failing
  games against 4,646). That is 17,797 IMPs of "doubling errors" that
  are our bidding's contract errors seen from the other side; the
  defenders know only that each other passed.
- **Where both sides compete, we are ahead,** because our declarers
  compete less: fewer chances, not better doubling. BBA doubles 24%
  (both bid, a fit), 26% (no fit) and 69% (over our game) of those
  chances, and 713 of its 1,250 doubles of contracts that made are
  there.
- **Vulnerability:** an undoubled chance costs 4.9 IMPs when the
  declarers are vulnerable and 3.4 when not (BBA 4.5 and 3.3).
- **Seat:** in the undoubled chances where the defenders held 20+ HCP,
  both decisive passes were replayed. Half of them are "Defending:
  nothing more to say" (advances.bid), then after-interference.bid's
  "Passed last time" and "Partner passed", rebids.bid's "Partner has
  nothing", responder-rebids.bid's "Partner had no reopening bid
  either".

### What the defenders know

The decisive passes of the undoubled chances where the defenders held
20+ HCP were replayed through `rbb call --json` (19,632 positions),
then a sample of 15,000 contested final contracts, made or not (30,000
positions), and 8,000 uncontested ones (16,000), to cost a double
(double dummy, no runout: an upper bound, see below).
- **Partner's shown minimum is 0 in most positions.** A pass is a true
  0, but so is most of what partner bids: in 53% of the positions where
  partner had made a bid his `hcp.min` was 0, and in 93% after a
  two-level bid. The rules describe overcalls, raises and new suits in
  total points (`hcp+length_points>=8`), and the knowledge ranges do
  not narrow `hcp` (or `points`) from a sum: a simple raise "6-9 total
  points" reads as 0-30 HCP, an overcall "8-16 total points" as 0-16.
  So `we.hcp.min` is my own HCP in most contested positions, and
  `they.hcp.max` is near 40 (their raises are 0-30 too): the deck bound
  (40 minus their maximum) adds to it in only a fifth to a quarter of
  contested positions. This is the knowledge limit, and it is in the
  engine (For Rick 7).
- **The inference from a pass is lost twice.** "No overcall" over their
  one-level opening should deny 17+ (the power double takes every hand
  of 17+), but reads 0-30: the takeout double's third `shows` line uses
  `has(A,x)` about the doubler's own hand, which cannot be resolved for
  another seat, so the whole denial is skipped (engine.rs, negative
  inference); and without that line the denial is kept as a constraint
  but the range summary cannot see that `hcp>=17` implies the rest of
  it. A lower maximum would not help a penalty double anyway: the
  doubles read minimums.
- **Their suit is known**: their length was read in 99.7% of contested
  positions. Unread bids (no rule: the balancing 2S of 1H P 2H 2S)
  still block `natural_theirs` there.
- **Doubling what is not known loses.** The contested sample loses
  63,444 IMPs doubled whole; with the actual 22+ HCP and three trumps
  it would gain about 2 a board. With what is known (`we.hcp.min` 20,
  three trumps) a double gains 1.8 a board on 275 positions; lower
  thresholds and trump stacks look good statically (five trumps and 10
  HCP, +4.6 a board) but lost when measured on 2026-10-01 (stacks,
  above): they run, and the static cost ignores it. "Our fit, they
  compete at the three level", BBA's largest class, loses statically at
  every HCP threshold tried (-1.6 to -3.1 a board): with a fit on both
  sides the three level often makes.
- **Uncontested**: with 15+ HCP against their game +1.6 a board
  statically (85 of 8,000 sampled contracts), 17+ against 3NT more;
  against partscores everything lost. Hence the game double below.

### Doubles of a sacrifice over our game

A small block in self-play: our declarers bid over our game on 369
final contracts of the vanilla set (BBA's opponents 1,884), and 74 of
163 sampled made: they are competitive bids with a fit, not
sacrifices. A forcing-pass treatment is not needed for these numbers;
the rule above lets the side that bid game double with values.

### The reopening double left in

Responder's penalty pass of opener's reopening double was our worst
doubling rule: 304 taken, 178 made (1,020 IMPs); every other pair of
doubling rule and the partner's rule that left it in nets positive
double dummy. Four poor trumps lose at every HCP below 7 at the one and
two level; with good trumps or five, the pass gains 3.9 to 8.5 IMPs a
board.

### Measured (errors_diff against main 8224d84)

Each change alone; doubling errors with even / odd halves, then
contract errors and distance from par. Positive: fewer errors.

| change | set | doubling (even / odd) | contract | par |
|---|---|---|---|---|
| reopening double: four poor trumps need 6 HCP | vanilla | +243 | -70 | +241 |
| ... 7 | vanilla | +307 (+152 / +155) | -113 | +277 |
| **... 8 (kept)** | vanilla | **+317 (+162 / +155)** | -95 | +326 |
| | corpus | +217 (+81 / +136) | -87 | +229 |
| | 21GF | +282 (+145 / +137) | -93 | +303 |
| ... 9 | vanilla | +320 | -96 | +356 |
| ... 10 | vanilla | +357 | -123 | +337 |
| their game, my 15 | vanilla | +721 (+381 / +340) | -217 | -1,850 |
| ... 16 | vanilla | +860 (+431 / +429) | -116 | -721 |
| ... 16, suits with two trumps | vanilla | +510 | -116 | -729 |
| ... 16, suits only (notrump stays at 21 known) | vanilla | +705 | -39 | -387 |
| **... 17 (kept)** | vanilla | **+674 (+317 / +357)** | -54 | -101 |
| | corpus | +1,106 (+544 / +562) | +28 | -237 |
| | 21GF | +1,055 (+487 / +568) | -72 | -73 |
| ... 17, suits with two trumps | vanilla | +386 | -57 | -205 |
| ... 18 | vanilla | +498 (+199 / +299) | -36 | +7 |
| | corpus | +949 (+421 / +528) | +50 | +100 |
| | 21GF | +851 (+378 / +473) | -40 | +45 |
| over our game: 10 or 19 known | vanilla | +212 | -6 | -32 |
| ... 12 or 20 | vanilla | +215 (+135 / +80) | -1 | +29 |
| ... 14 or 21 | vanilla | +181 (+81 / +100) | +5 | +62 |
| **... 12 or 20, two trumps (kept)** | vanilla | **+194 (+122 / +72)** | +1 | +42 |
| | corpus | +662 (+453 / +209) | -12 | -327 |
| | 21GF | +286 (+136 / +150) | -7 | -64 |

9 and 10 HCP gain a few more doubling IMPs and lose more contract
errors; 8 is where the two meet. The three together (the branch
against main):

| set | doubling errors vs BBA | change (even / odd) | contract | par | errors line | chances doubled | doubled, made | bailed out |
|---|---|---|---|---|---|---|---|---|
| vanilla SAYC random | -10,550 -> -9,375 | +1,175 (+602 / +573) | -148 | +260 | +1,027 (+475 / +552) | 790 -> 946 | 300 -> 284 | 1,075 -> 1,076 |
| full corpus | -14,861 -> -12,982 | +1,879 (+1,016 / +863) | -69 | -352 | +1,810 (+935 / +875) | 2,519 -> 3,116 | 683 -> 925 | 1,785 -> 1,785 |
| 21GF random | -16,597 -> -14,996 | +1,601 (+761 / +840) | -172 | +183 | +1,429 (+634 / +795) | 1,104 -> 1,381 | 378 -> 423 | 1,381 -> 1,382 |

The contract errors that fall are mostly the reopening double's: when
responder no longer leaves it in he bids, and some of those contracts
are worse than the doubled one would have been. Par counts a double
that collects more than par as a loss, which is why it goes the other
way from the doubling errors on the corpus.

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
7. **Partner's strength from total points** (2026-10-02, engine). The
   rules describe most bids in total points (`hcp+length_points`), and
   the knowledge ranges do not narrow `hcp` or `points` from such a sum:
   a simple raise reads 0-30 HCP, an overcall 0-16, so `we.hcp.min` is
   my own HCP in most contested positions and `they.hcp.max` near 40.
   Two engine changes would let the doubles (and every rule that reads
   a range) see more: narrow `hcp.max` from `hcp + lp <= N` (sound:
   lp >= 0), and keep a total-points range that `hcp + lp >= N` raises,
   for a `we.points.min` the doubles could read. Not done: it needs your
   approval, and it changes how existing rules read (a language
   version question).
8. **The has() denial.** The takeout double's `has(A,x)` line stops the
   negative inference from "No overcall" altogether (it is about the
   caller's own hand, unresolvable for another seat). Dropping only the
   unresolvable branch of a denial is an engine change; a `denies
   hcp>=17` on the pass is a rules change. Neither helps a penalty
   double (it lowers a maximum).
9. **Doubles of contracts that made.** The kept rules lower the count
   on vanilla (300 -> 284) but raise it on the corpus (683 -> 925) and
   21GF (378 -> 423): the game double adds them (+252 on the corpus),
   the over-our-game double some (+100), the reopening pass takes some
   away (-107). The 18-HCP game double adds a third to two fifths as
   many and gains 74-86% as much. Kept at 17 by the yardstick; 18 if you
   would rather double fewer games that make.
10. **Most of the doubling gap is our declarers.** 17,797 IMPs of the
    vanilla gap are in auctions where the defenders never bid: we bid
    more failing games and partscores uncontested than BBA (7,407
    failing games against 4,646). That is placement, not a double
    anyone could find; the errors yardstick books it to the defenders.
11. **"Will stand" with the lower floors** (2026-10-03). Dropping the
    condition that the contract will stand (passed round, a fit, or
    game) now gains doubling errors on vanilla alone (+246, both halves)
    and with the kept change (+920 together), but the contract errors
    grow to -321 and par to -876: directly over a new suit that may be
    forcing, the double takes away their runout and ours. Not measured
    on 21GF and the corpus. Worth a ruling on whether a contract error of
    that size is "clearly worse".
12. **How far down the good four goes** (2026-10-03). Each point below
    20 gains doubling errors on both halves down to 15 (+590); contract
    errors and par worsen at each step. Kept at 17, where the trade is
    best; 16 and 15 are there if the doubling yardstick alone decides.
    Their game at 16 likewise gains doubling errors (+204) at -605 par
    and 123 more doubles of contracts that made: not kept.


## Higher floors, more doubles (2026-10-03)

The engine now ties total points to the HCP (LANGUAGE.md §8): a 1♦
opening of 12+ declarer points with no suit beyond five is 11+ HCP,
not 10+. `we.hcp.min` is a point higher in many positions, so the
thresholds of `pen_suit` (calibrated 2026-10-01 on the old floors)
double a little more. The test of four small trumps with 20 known now
had 21 known (and doubled); the hand is now 9 HCP so it keeps testing
20. Across the three sets ours doubled 945 -> 999 (vanilla), 1,380 ->
1,452 (21GF), 3,116 -> 3,174 (corpus), with doubles of making contracts
287 -> 296, 425 -> 439, 926 -> 954. The doubling errors of the run
moved -1,099 / -298 / -656, but almost all of that is "doubling other"
(the side that did not make the first differing call failing to double
our bolder contracts); the actor's own doubling errors improved
(+412 / +445 / +281). The thresholds were then retuned on these floors
(next section): none rose, the good-four floor fell to 17.

## Retuned on the higher floors (2026-10-03)

After PR #44 (partner's HCP floor known in 87% of positions, was about
70%; a 1♦ opener reads 11+, was 10) and PR #45 (compare is
deterministic: two runs of the same rules are identical, so small
differences are real), every threshold was moved one point each way,
two where the first step gained, one variant at a time, on the vanilla
set against main 53f1d3a (`probes/tools/errors_diff.py`). Main:
errors -43,131 (contract -32,577, doubling -10,554), distance from par
-25,486. Judged on **doubling errors** (CLAUDE.md, "Judging a change"),
contract errors and par beside them. Positive: fewer errors.

| variant (vanilla) | boards | errors (even / odd) | doubling (even / odd) | contract | par (even / odd) | doubled / made |
|---|---|---|---|---|---|---|
| main | | | | | | 999 / 296 |
| good four at 21 | 21 | +10 (+17 / -7) | +1 (+7 / -6) | +9 | +28 | 988 / 290 |
| good four at 19 | 35 | +44 (+25 / +19) | +61 (+32 / +29) | -17 | -92 (-45 / -47) | 1,019 / 302 |
| good four at 18 | 93 | +103 (+64 / +39) | +168 (+98 / +70) | -65 | -236 (-136 / -100) | 1,049 / 312 |
| good four at 17 | 165 | +261 (+121 / +140) | +347 (+180 / +167) | -86 | -382 (-257 / -125) | 1,098 / 322 |
| good four at 16 | 264 | +275 (+154 / +121) | +428 (+239 / +189) | -153 | -676 (-422 / -254) | 1,152 / 349 |
| good four at 15 | 387 | +388 (+211 / +177) | +590 (+307 / +283) | -202 | -949 (-559 / -390) | 1,226 / 375 |
| any four at 20 (four small) | 50 | +83 (+53 / +30) | +102 (+77 / +25) | -19 | -77 (-56 / -21) | 1,030 / 305 |
| three at 20 | 230 | -143 (+35 / -178) | -138 (+29 / -167) | -5 | -396 | 1,115 / 374 |
| three at 22 | 93 | -178 (-98 / -80) | -166 (-91 / -75) | -12 | +62 | 932 / 277 |
| anything at 21 | 187 | -34 (-134 / +100) | -57 (-146 / +89) | +23 | -246 | 1,122 / 356 |
| anything at 23 | 100 | -226 (-110 / -116) | -205 (-97 / -108) | -21 | -59 | 921 / 278 |
| notrump at 20 | 11 | -8 (+4 / -12) | +20 (+15 / +5) | -28 | -40 | 1,004 / 297 |
| notrump at 22 | 1 | -5 | -5 | 0 | -5 | 998 / 296 |
| their game, my 16 | 303 | +144 (+101 / +43) | +204 (+134 / +70) | -60 | -605 (-292 / -313) | 1,160 / 419 |
| their game, my 16, suits only | 238 | +114 (+125 / -11) | +131 (+131 / 0) | -17 | -381 | 1,132 / 396 |
| their game, my 18 | 145 | -155 (-114 / -41) | -174 (-124 / -50) | +19 | +125 (+31 / +94) | 918 / 246 |
| over our game: 11 in my hand | 29 | -3 (-9 / +6) | -1 (-7 / +6) | -2 | +5 | 1,010 / 306 |
| over our game: 13 in my hand | 14 | -38 (-34 / -4) | -38 (-34 / -4) | 0 | +6 | 989 / 293 |
| over our game: 19 known | 10 | -1 (-11 / +10) | -1 (-11 / +10) | 0 | +13 | 1,001 / 298 |
| over our game: 21 known | 0 | 0 | 0 | 0 | 0 | 999 / 296 |
| over our game: 14 or 21 | 32 | -83 (-61 / -22) | -89 (-62 / -27) | +6 | -11 | 976 / 290 |
| partner's notrump allowed | 39 | -3 (+1 / -4) | +30 (+20 / +10) | -33 | -7 | 1,023 / 301 |
| "will stand" dropped | 111 | +184 (+87 / +97) | +246 (+142 / +104) | -62 | -203 (-89 / -114) | 1,058 / 310 |
| **good four at 17, any four at 20 (kept)** | 215 | **+344 (+174 / +170)** | **+449 (+257 / +192)** | -105 | -459 (-313 / -146) | 1,129 / 331 |
| good four at 17 + "will stand" dropped | 353 | +421 (+157 / +264) | +728 (+382 / +346) | -307 | -769 (-475 / -294) | 1,181 / 341 |
| kept + "will stand" dropped | 438 | +599 (+266 / +333) | +920 (+503 / +417) | -321 | -876 (-558 / -318) | 1,215 / 354 |

What it says:
- **Nothing should rise.** Every raise (three at 22, anything at 23,
  their game at 18, over our game at 13 or 21) loses doubling errors on
  both halves or changes nothing. The higher floors did not make the
  doubles too loose.
- **The good-four floor falls.** On the old floors good four at 19
  split the halves (+9) and five-or-good-four at 16 lost; now each
  step down gains doubling errors on both halves, to 15 and beyond.
  Contract errors and par worsen with each step: the doubling gain per
  point of contract loss is best at 17 (+347 / -86; 18 is +168 / -65,
  16 +428 / -153), so it stops there.
- **Four small at 20** gains (+102, both halves; it split on the old
  floors), and adds to the good four at 17 almost unchanged.
- **The rest holds**: three at 21, anything at 22, notrump at 21,
  their game at 17 (16 gains doubling errors but par loses 605 and the
  doubles of making contracts rise 296 -> 419; 18 loses), over our game
  at 12 / 20 known.
- **"Will stand" dropped** now gains alone (+246, both halves; it split
  on the old floors), but with the lower good-four floor the contract
  errors grow to -307 / -321: doubling a new suit that may be forcing
  costs the doubled side's runout. Left out (For Rick 11).

The kept change on the three sets (main 53f1d3a -> kept):

| set | doubling errors vs BBA | change (even / odd) | contract (even / odd) | par | errors line (even / odd) | chances doubled | doubled, made | bailed out |
|---|---|---|---|---|---|---|---|---|
| vanilla SAYC random | -10,554 -> -10,105 | +449 (+257 / +192) | -105 (-83 / -22) | -25,486 -> -25,945 (-459) | -43,131 -> -42,787: +344 (+174 / +170) | 999 / 24,033 -> 1,129 / 24,028 | 296 -> 331 | 1,080 -> 1,080 |
| 21GF random | -15,422 -> -14,717 | +705 (+355 / +350) | -166 (-107 / -59) | -39,441 -> -40,013 (-572) | -63,093 -> -62,554: +539 (+248 / +291) | 1,452 / 26,416 -> 1,611 / 26,408 | 439 -> 474 | 1,387 -> 1,387 |
| full corpus | -13,498 -> -12,546 | +952 (+542 / +410) | -200 (-85 / -115) | -84,142 -> -84,765 (-623) | -107,211 -> -106,459: +752 (+457 / +295) | 3,174 / 35,417 -> 3,368 / 35,400 | 954 -> 1,001 | 1,789 -> 1,789 |

The par loss is the known artefact: on the corpus, `penalty_x.py`
(the penalty-doubles.bid doubles) gives -532 par but +939 side IMPs on
the boards where the doubler's side ended above par, and -28 par / -49
side / -80 below par on the rest. Side IMPs to the doubler: +464
vanilla, +704 21GF, +864 corpus, both halves each. The contract loss
is about two thirds the doubler's side ("actor": -70 / -117 / -207),
where a double now takes the place of our own contract.

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
- **BBA's issues** on its own penalty doubles (github.com/EdwardPiwowar/BBA):
  #83 "Another terrible penalty double", #86 "Yet another wrong penalty
  double", #731, #797, #578 "This is not a penalty double - just a
  strong hand", #1092 "Mad doubles are still with us": its count of
  doubles that made is a known cost, not one to copy.
- **2026-10-02:** the game and over-our-game doubles and the reopening
  pass are standard practice, not yet cited (a game bid against 17 in
  my hand; the side that bid game doubles their bid over it; leaving a
  takeout double in needs trump tricks); thresholds calibrated here on
  the three sets (tables above); `probes/tools/double_chances.py`.
- **Measurements:** the tables above, from `rbb compare` on
  Random_Pavlicek_SAYCvanilla with the vanilla `--set` list, the
  `rbb call` replay of 15,000 positions, `probes/tools/sideimps.py`,
  `probes/tools/par_blame.py`, `probes/tools/penalty_x.py`, and
  `probes/tools/errors_diff.py` for the re-judgment by errors
  (Rick's yardstick, 2026-10-01).
- **2026-10-03:** the test hand and the counts above, from
  `rbb compare` on the three sets and `probes/tools/errors_diff.py`.
- **2026-10-03, retune:** the variant tables and the three-set table,
  from `rbb compare` on main 53f1d3a (after PRs #44 and #45) and
  `probes/tools/errors_diff.py`, `probes/tools/penalty_x.py`; the
  thresholds are calibrated here, not quoted.
