# Raises over their takeout double (`vs-takeout-double.bid`): notes

Jordan (Truscott) 2NT and BROMAD, written on the fast lane (CLAUDE.md
"Pace", 2026-10-05). The rest of responder's calls over the double are in
after-interference.bid; opener's answer to the limit raise is the shared
`when asked limit_raise(t)` in two-suited-overcalls.bid (game in a major
with 14+ support points, else the cheapest bid in the suit).

## What we play

- **Jordan 2NT** (`competitive.jordan_2nt.play`, on in every PBS 21GF
  card): 1M (X) 2NT is a limit raise or better with four or more trumps
  (10+ support points), above the redouble. A passed hand that plays
  Drury over the double (`major_openings.drury.in_comp`) bids Drury
  instead. Responder bids game after opener's sign-off with 13+.
- **The jump to 3M** is preemptive with four trumps and **up to 7**
  support points. Bridge Bum says 5-9; BBA (all 1M (X) auctions in the
  corpus) raises to 2M with four trumps and 6-9 HCP (450 boards) and
  jumps or passes with less, and our first version with 9 took ~35 of
  BBA's 2M raises. 7 is the compromise; four trumps and 8-9 raise to 2M.
- **BROMAD** (`competitive.bromad.play`; 21GF-GIB-Bergen, SPECIALS2,
  PolishTwoSuiters, WJS-MSS, Precision): Bridge Bum's table, with the
  constructive raises 7-9 (Bridge Bum: 7-10, overlapping the limit
  raise's 10). Every card with BROMAD also plays Jordan, and BBA bid
  Jordan's 2NT on the one such board in the corpus (Exclusion_After_1M,
  21GF-GIB-Bergen), so with both on 2NT stays Jordan (four-card limit
  raise or better) and BROMAD's 3♦ and weak-minors 2NT are dropped.
  Opener over 2♣: 2M, 3M inviting with 15-16, 4M with 17+; over 3♣ 4M
  with 15+. 2♦ and 3♦ are answered as limit raises.

## Corpus (compare --limit 50, 2026-10-05)

- Jordan_2N: call agreement 86.8% after (it was the redouble at the
  first call on 28 of 50 boards before). Remaining at our calls: opener
  accepts with 14 support points where BBA signs off in 3M on some
  (3 boards), and BBA jumps to 4M with five trumps and 6-9 HCP where our
  support points reach 10 and we bid 2NT (4 boards).
- Elsewhere the tripwire moved Fit_Jumps_after_1M_Double +50 agreeing
  calls, Xfer_after_1M_X +10, Gavin_4-Card_Limit_Raise +9,
  Responsive_Double -5, Transfers_after_1M_X -2.

## Gaps and open questions

- No preemptive 4M with five trumps (BBA: 157 boards with 6-9 HCP);
  after-interference.bid has no rule for it either. For Rick: add one
  (five trumps, under 10 HCP) above Jordan?
- Opener's splinters and game tries after 2NT (BBA bids 4♣/4♦ splinters
  and 3♣ as a try) are not written.
- BROMAD's 2NT (both minors) is unreachable with Jordan on, which is
  every PBS card; for Rick: is that the intended combination?
- The convention-card field `vs_to_double.two_nt_raise_majors.*` (limit
  or weak) is not read; Jordan's field is.

## Sources

- **Bridge Bum**, "Jordan 2NT" (<https://www.bridgebum.com/jordan_2nt.php>):
  2NT limit raise or better with four trumps, the redouble for a
  balanced 10-12, the jump to 3M weak (5-9), opener's sign-off in 3M.
  We differ: the jump stops at 7 (BBA).
- **Bridge Bum**, "BROMAD" (<https://www.bridgebum.com/bromad.php>): the
  responses table. We differ: constructive raises 7-9, and with Jordan
  on, 2NT is Jordan.
- **Practice-Bidding-Scenarios** `btn/Jordan_2N.btn`: "After 1M (X), 2N
  shows 4+ card support and 10+ TP".
- **BBA corpus** (2026-10-05): responder's calls after 1M (X) by trump
  length and HCP, all scenarios; "Jordan Truscott 2NT" notes.
