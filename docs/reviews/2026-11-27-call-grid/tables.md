# The call grid (2026-11-27)

Rick (2026-11-27): fill in the grid of which calls have a meaning; all competitive
auctions would get out of hand, so only a general description there; every system has
holes, which is not a problem in itself, but cross-check the common ones against Brill,
WBridge5 and GIB to see whether we miss anything basic. Companion to
[2026-11-27-undefined-calls.md](2026-11-27-undefined-calls.md) (holes in real auctions).

## Method

- **Our tree, uncontested** (`probes/tools/call_grid.py walk`): from the opening seat and
  the second seat (so every seat opens), the opponents passing throughout, every position
  of our own tree up to the fourth non-pass call. At each, `rbb bid rules --json` lists the
  calls some rule can make there (the engine's candidates, before the hand is known); every
  other legal bid up to 5NT is a **hole**. Children are the defined bids, so the tree is the
  one our own rules can reach. Pass is counted apart (a pass ruled out by a force is not a
  hole). Card `Brill` (what the seat service plays); 21GF-GIB beside it. Rules: pack
  `bottom-review` a3bffe8. Vulnerability none.
- **Competitive, one level** (`comp`): responder over our opening and their simple action;
  our first call over their opening, direct and pass-out seat; the advancer over a few common
  overcalls and doubles. A description, not a tree.
- **Weights** (`weigh`): each position and call counted in real auctions, the opening side
  bidding alone: the Brill field (all tables, 50 days), the BBA and GIB corpora
  (Practice-Bidding-Scenarios), and WBridge5 at all four seats (1,000 Pavlicek deals and 199
  slam deals from wbridge5_bid.py). Rules and card differ between corpora, so a position our
  tree reaches may be reached there with other meanings: frequency, not agreement.
- **Cross-check** (`cross`): the most common holes and every hole in rounds 1-2, against
  GIB's tree (the gibrest archive), Brill's `/explain` of the auction (Thorvald's service,
  one request a second, cached) and whether WBridge5 ever made the call there.

## 1. Coverage of our own tree (uncontested)

**Brill**

| call | positions | legal bids to 5NT | with a meaning | through 3NT | 4♣ to 5NT |
|---|---|---|---|---|---|
| 1st (opening) | 4 | 100 | 78 (78%) | 93% | 55% |
| 2nd (response) | 78 | 1,101 | 588 (53%) | 75% | 41% |
| 3rd (opener rebid) | 604 | 5,935 | 3,148 (53%) | 78% | 44% |
| 4th (responder rebid) | 3,722 | 25,268 | 12,668 (50%) | 63% | 47% |

**21GF-GIB**

| call | positions | legal bids to 5NT | with a meaning | through 3NT | 4♣ to 5NT |
|---|---|---|---|---|---|
| 1st (opening) | 4 | 100 | 78 (78%) | 93% | 55% |
| 2nd (response) | 78 | 1,101 | 612 (56%) | 75% | 44% |
| 3rd (opener rebid) | 628 | 6,070 | 3,011 (50%) | 70% | 42% |
| 4th (responder rebid) | 3,589 | 23,521 | 11,442 (49%) | 61% | 46% |

Weighted by what was actually bid at those positions (share of real calls, bids to 5NT, that
have a meaning in our tree):

| call | Brill field | GIB corpus | BBA corpus | WBridge5 |
|---|---|---|---|---|
| 1 | 100% of 31,084 | 100% of 16,626 | 100% of 170,408 | 100% of 1,195 |
| 2 | 99% of 17,257 | 94% of 9,926 | 99% of 104,065 | 98% of 681 |
| 3 | 98% of 12,731 | 97% of 6,503 | 95% of 78,404 | 94% of 475 |
| 4 | 86% of 9,272 | 90% of 4,124 | 80% of 55,700 | 73% of 371 |

### By opening

Holes (bids to 5NT with no meaning) at responder's first call and over the next two
calls, by opening; `reached` = Brill-field auctions through the opening, uncontested.

| opening | response: holes / legal | rounds 3-4: holes / legal | Brill reached | response holes |
|---|---|---|---|---|
| 1♣ | 44/96 | 2630/5724 | 6,435 | 3♦ 3♥ 3♠ 4♣ 4♦ 4♥ 4♠ |
| 1♦ | 42/92 | 2172/4750 | 6,817 | 3♥ 3♠ 4♣ 4♦ 4♥ 4♠ |
| 1♥ | 22/88 | 2313/5084 | 4,509 | 4♠ |
| 1♠ | 20/84 | 1897/4124 | 4,327 |  |
| 1NT | 20/80 | 2044/3576 | 4,579 | 4♠ |
| 2♣ | 32/76 | 1304/2928 | 530 | 3♥ 3♠ 4♣ 4♦ |
| 2♦ | 36/72 | 696/1076 | 679 | 3♣ 3♥ 3♠ 4♣ 4♥ 4♠ |
| 2♥ | 44/68 | 388/666 | 658 | 3♣ 3♦ 3♠ 3NT 4♣ 4♦ 4♠ |
| 2♠ | 44/64 | 248/468 | 949 | 3♣ 3♦ 3♥ 3NT 4♣ 4♦ 4♥ |
| 2NT | 16/60 | 936/1488 | 697 |  |
| 3♣ | 21/42 | 189/293 | 120 | 3♦ 3♥ 3♠ 4♦ |
| 3♦ | 18/39 | 180/287 | 224 | 3♥ 3♠ 4♣ |
| 3♥ | 27/36 | 33/81 | 149 | 3♠ 3NT 4♣ 4♦ 4♠ |
| 3♠ | 24/33 | 27/78 | 100 | 3NT 4♣ 4♦ 4♥ |
| 3NT | 16/40 | 208/320 | 4 |  |
| 4♣ | 21/27 | 21/45 | 53 | 4♦ 4♥ 4♠ |
| 4♦ | 18/24 | 18/45 | 53 | 4♥ 4♠ |
| 4♥ | 15/21 | 15/45 | 73 | 4♠ |
| 4♠ | 12/18 | 12/45 | 104 |  |
| 4NT | 0/20 | 56/80 | 0 |  |
| 5♣ | 12/12 | 0/0 | 23 |  |
| 5♦ | 9/9 | 0/0 | 1 |  |

## 2. Where the holes are, in general terms

Holes by round and kind of call (count of holes at positions of our tree; `weight` =
Brill-field auctions reaching those positions, so a hole at a common position counts more).

**Bids through 3NT**

| kind of call | 2nd call: holes (weight) | 3rd | 4th | bid by Brill at those holes |
|---|---|---|---|---|
| double jump new suit | 20 (17,469) | 8 (1,220) | 0 (0) | 33 |
| jump new suit | 20 (2,002) | 112 (8,525) | 176 (3,990) | 163 |
| simple new suit | 46 (4,799) | 116 (1,509) | 748 (7,222) | 580 |
| jump NT | 8 (912) | 32 (2,656) | 68 (1,345) | 28 |
| cheapest NT | 6 (193) | 66 (2,150) | 230 (1,288) | 98 |
| jump raise of partner's suit | 4 (994) | 0 (0) | 40 (1,780) | 57 |
| simple rebid of own suit | 0 (0) | 20 (378) | 116 (2,324) | 130 |
| simple raise of partner's suit | 0 (0) | 0 (0) | 106 (1,321) | 30 |
| jump fourth suit | 0 (0) | 0 (0) | 58 (1,252) | 1 |
| double jump NT | 0 (0) | 4 (986) | 0 (0) | 19 |
| simple fourth suit | 0 (0) | 0 (0) | 128 (884) | 140 |
| jump rebid of own suit | 0 (0) | 0 (0) | 32 (482) | 13 |
| double jump fourth suit | 0 (0) | 0 (0) | 4 (155) | 0 |

**Bids from 4♣ to 5NT**

| kind of call | 2nd call: holes (weight) | 3rd | 4th | bid by Brill at those holes |
|---|---|---|---|---|
| triple jump new suit | 108 (77,899) | 192 (24,239) | 178 (5,719) | 46 |
| double jump new suit | 82 (15,688) | 404 (16,974) | 862 (9,872) | 9 |
| jump new suit | 82 (3,816) | 560 (10,138) | 1,821 (8,801) | 39 |
| triple jump raise of partner's suit | 16 (13,061) | 24 (4,866) | 112 (4,090) | 0 |
| double jump raise of partner's suit | 16 (8,009) | 72 (3,275) | 504 (7,491) | 6 |
| double jump rebid of own suit | 0 (0) | 112 (6,168) | 416 (5,578) | 27 |
| triple jump rebid of own suit | 0 (0) | 58 (7,739) | 96 (2,965) | 2 |
| jump raise of partner's suit | 10 (669) | 92 (4,264) | 877 (5,209) | 122 |
| 4NT/5NT | 14 (967) | 332 (6,970) | 808 (629) | 15 |
| simple new suit | 69 (1,088) | 319 (1,597) | 2,462 (4,480) | 77 |
| jump rebid of own suit | 0 (0) | 134 (1,435) | 606 (3,952) | 16 |
| simple rebid of own suit | 0 (0) | 72 (572) | 648 (1,683) | 44 |
| simple raise of partner's suit | 12 (156) | 58 (117) | 1,122 (1,615) | 119 |
| triple jump fourth suit | 0 (0) | 0 (0) | 56 (1,376) | 0 |
| double jump fourth suit | 0 (0) | 0 (0) | 102 (703) | 0 |
| jump fourth suit | 0 (0) | 0 (0) | 100 (634) | 0 |
| simple fourth suit | 0 (0) | 0 (0) | 124 (82) | 2 |

Openings with no meaning (first seat, card Brill): 5♥ 5♠ 5NT.

## 3. One level of interference

Share of legal calls through 3NT (with X/XX where legal) that have a meaning, and the
holes through 3NT. `Brill` = Brill-field auctions through the position.

**responder over our opening and their action**: 336/562 (60%) of calls through 3NT have a meaning.

| position | defined | holes through 3NT | Brill |
|---|---|---|---|
| 1♣ 1♦ | 11/14 | 3♦ 3♥ 3♠ | 242 |
| 1♣ 1♥ | 10/13 | 3♦ 3♥ 3♠ | 350 |
| 1♣ 1NT | 6/11 | 2NT 3♣ 3♥ 3♠ 3NT | 64 |
| 1♣ 1♠ | 9/12 | 3♦ 3♥ 3♠ | 288 |
| 1♣ 2♣ | 5/10 | 2♠ 3♦ 3♥ 3♠ X | 20 |
| 1♣ 2♦ | 7/9 | 3♥ 3♠ | 55 |
| 1♣ 2♥ | 7/8 | 3♠ | 86 |
| 1♣ 2NT | 4/6 | 3♥ X | 17 |
| 1♣ 2♠ | 7/7 | — | 5 |
| 1♣ X | 9/15 | 2NT 3♣ 3♦ 3♥ 3♠ 3NT | 130 |
| 1♦ 1♥ | 10/13 | 3♣ 3♥ 3♠ | 239 |
| 1♦ 1NT | 6/11 | 2NT 3♦ 3♥ 3♠ 3NT | 59 |
| 1♦ 1♠ | 9/12 | 3♣ 3♥ 3♠ | 397 |
| 1♦ 2♣ | 7/10 | 3♦ 3♥ 3♠ | 75 |
| 1♦ 2♦ | 5/9 | 2♠ 3♥ 3♠ X | 41 |
| 1♦ 2♥ | 7/8 | 3♠ | 9 |
| 1♦ 2NT | 4/6 | 3♥ X | 0 |
| 1♦ 2♠ | 7/7 | — | 66 |
| 1♦ 3♣ | 5/5 | — | 19 |
| 1♦ X | 8/14 | 2♣ 2NT 3♦ 3♥ 3♠ 3NT | 246 |
| 1♥ 1NT | 8/11 | 2NT 3♠ 3NT | 52 |
| 1♥ 1♠ | 9/12 | 3♣ 3♦ 3♠ | 324 |
| 1♥ 2♣ | 8/10 | 3♦ 3♠ | 93 |
| 1♥ 2♦ | 8/9 | 3♠ | 6 |
| 1♥ 2♥ | 6/8 | 3♠ X | 48 |
| 1♥ 2NT | 5/6 | X | 12 |
| 1♥ 2♠ | 7/7 | — | 82 |
| 1♥ 3♣ | 5/5 | — | 9 |
| 1♥ 3♦ | 4/4 | — | 0 |
| 1♥ X | 9/13 | 2♣ 2♦ 3♠ 3NT | 102 |
| 1NT 2♣ | 5/10 | 2♠ 3♣ 3♦ 3♥ 3♠ | 84 |
| 1NT 2♦ | 4/9 | 2♥ 2♠ 2NT 3♥ 3♠ | 20 |
| 1NT 2♥ | 8/8 | — | 62 |
| 1NT 2♠ | 7/7 | — | 40 |
| 1NT 3♣ | 2/5 | 3♦ 3♥ 3♠ | 0 |
| 1NT 3♦ | 2/4 | 3♥ 3♠ | 0 |
| 1NT X | 4/11 | 2♠ 2NT 3♣ 3♦ 3♥ 3♠ 3NT | 6 |
| 1♠ 1NT | 8/11 | 2NT 3♥ 3NT | 41 |
| 1♠ 2♣ | 8/10 | 3♦ 3♥ | 22 |
| 1♠ 2♦ | 8/9 | 3♥ | 34 |
| 1♠ 2♥ | 8/8 | — | 40 |
| 1♠ 2NT | 4/6 | 3♥ X | 15 |
| 1♠ 2♠ | 6/7 | X | 18 |
| 1♠ 3♣ | 5/5 | — | 19 |
| 1♠ 3♦ | 4/4 | — | 51 |
| 1♠ 3♥ | 3/3 | — | 2 |
| 1♠ X | 8/12 | 2♣ 2♦ 2♥ 3NT | 233 |
| 2♣ 2♦ | 1/9 | 2♥ 2♠ 2NT 3♣ 3♦ 3♥ 3♠ X | 0 |
| 2♣ 2♥ | 1/8 | 2♠ 2NT 3♣ 3♦ 3♥ 3♠ X | 21 |
| 2♣ 2NT | 1/6 | 3♣ 3♦ 3♥ 3♠ X | 0 |
| 2♣ 2♠ | 1/7 | 2NT 3♣ 3♦ 3♥ 3♠ X | 6 |
| 2♣ 3♣ | 1/5 | 3♦ 3♥ 3♠ X | 0 |
| 2♣ 3NT | 0/1 | X | 0 |
| 2♣ X | 1/10 | 2♦ 2♥ 2♠ 2NT 3♣ 3♦ 3♥ 3♠ XX | 0 |
| 2♦ 2♥ | 2/8 | 2♠ 2NT 3♣ 3♥ 3♠ 3NT | 24 |
| 2♦ 2NT | 1/6 | 3♣ 3♥ 3♠ 3NT X | 1 |
| 2♦ 2♠ | 2/7 | 2NT 3♣ 3♥ 3♠ 3NT | 78 |
| 2♦ 3♣ | 2/5 | 3♥ 3♠ 3NT | 27 |
| 2♦ 3♦ | 0/4 | 3♥ 3♠ 3NT X | 0 |
| 2♦ 3NT | 0/1 | X | 8 |
| 2♦ X | 1/9 | 2♥ 2♠ 2NT 3♣ 3♥ 3♠ 3NT XX | 59 |
| 2♥ 2NT | 1/6 | 3♣ 3♦ 3♠ 3NT X | 21 |
| 2♥ 2♠ | 2/7 | 2NT 3♣ 3♦ 3♠ 3NT | 57 |
| 2♥ 3♣ | 2/5 | 3♦ 3♠ 3NT | 28 |
| 2♥ 3♦ | 2/4 | 3♠ 3NT | 2 |
| 2♥ 3♥ | 0/3 | 3♠ 3NT X | 2 |
| 2♥ 3NT | 0/1 | X | 1 |
| 2♥ X | 1/8 | 2♠ 2NT 3♣ 3♦ 3♠ 3NT XX | 39 |
| 2♠ 2NT | 1/6 | 3♣ 3♦ 3♥ 3NT X | 19 |
| 2♠ 3♣ | 2/5 | 3♦ 3♥ 3NT | 4 |
| 2♠ 3♦ | 2/4 | 3♥ 3NT | 16 |
| 2♠ 3♥ | 2/3 | 3NT | 13 |
| 2♠ 3NT | 0/1 | X | 23 |
| 2♠ 3♠ | 0/2 | 3NT X | 12 |
| 2♠ X | 1/7 | 2NT 3♣ 3♦ 3♥ 3NT XX | 87 |

**our first call over their opening, direct seat**: 93/115 (81%) of calls through 3NT have a meaning.

| position | defined | holes through 3NT | Brill |
|---|---|---|---|
| 1♣ | 13/15 | 3♣ 3NT | 2,898 |
| 1♦ | 12/14 | 3♦ 3NT | 3,345 |
| 1♥ | 11/13 | 3♥ 3NT | 2,110 |
| 1NT | 10/11 | 3NT | 2,039 |
| 1♠ | 10/12 | 3♠ 3NT | 1,991 |
| 2♣ | 0/10 | 2♦ 2♥ 2♠ 2NT 3♣ 3♦ 3♥ 3♠ 3NT X | 189 |
| 2♦ | 8/9 | 3NT | 445 |
| 2♥ | 7/8 | 3NT | 433 |
| 2♠ | 6/7 | 3NT | 609 |
| 3♣ | 5/5 | — | 42 |
| 3♦ | 4/4 | — | 145 |
| 3♥ | 3/3 | — | 116 |
| 3♠ | 2/2 | — | 62 |
| 4♥ | 1/1 | — | 21 |
| 4♠ | 1/1 | — | 58 |

**pass-out seat**: 30/65 (46%) of calls through 3NT have a meaning.

| position | defined | holes through 3NT | Brill |
|---|---|---|---|
| 1♣ P P | 6/15 | 2♣ 2♦ 2♥ 2♠ 3♣ 3♦ 3♥ 3♠ 3NT | 85 |
| 1♦ P P | 6/14 | 2♦ 2♥ 2♠ 3♣ 3♦ 3♥ 3♠ 3NT | 274 |
| 1♥ P P | 6/13 | 2♥ 2♠ 3♣ 3♦ 3♥ 3♠ 3NT | 57 |
| 1NT P P | 6/11 | 3♣ 3♦ 3♥ 3♠ 3NT | 455 |
| 1♠ P P | 6/12 | 2♠ 3♣ 3♦ 3♥ 3♠ 3NT | 187 |

**advancer**: 104/129 (81%) of calls through 3NT have a meaning.

| position | defined | holes through 3NT | Brill |
|---|---|---|---|
| 1♣ 1♥ P | 10/12 | 3♠ 3NT | 29 |
| 1♣ X P | 12/14 | 2♦ 3♣ | 21 |
| 1♦ 1♠ P | 10/11 | 3NT | 62 |
| 1♦ X P | 12/13 | 3♦ | 89 |
| 1♥ 1♠ P | 10/11 | 3NT | 107 |
| 1♥ 2♣ P | 6/9 | 3♦ 3♥ 3♠ | 38 |
| 1♥ X P | 11/12 | 3♥ | 13 |
| 1NT 2♥ P | 5/7 | 3♠ 3NT | 37 |
| 1NT X P | 4/10 | 2NT 3♣ 3♦ 3♥ 3♠ 3NT | 6 |
| 1♠ 2♥ P | 6/7 | 3♠ | 18 |
| 1♠ X P | 10/11 | 3♠ | 67 |
| 2♥ 2♠ P | 2/6 | 2NT 3♣ 3♦ 3♥ | 30 |
| 2♠ X P | 6/6 | — | 64 |

## 4. Cross-check of the common holes

114 holes: the 60 most common by real-auction
weight and 54 more in rounds 1-2. GIB = its tree has a
meaning; Brill = `/explain` reads the call by a system rule (not a fallback); WB5 = WBridge5
made the call there in its own auctions (its data is thin: 1,199 deals). GIB's tree
answers almost every call (a natural description when nothing else applies), and Brill
infers a natural meaning for calls no rule set names (`Inferred (...)`), so the test is
narrower than "has a meaning". Classes: **basic** = a named Brill rule set reads it and
GIB has it (both systems carry the call explicitly: likely something we miss);
**convention-dependent** = only one of them names it (a Brill rule set, or a GIB
convention name); **nobody defines it** = Brill only infers it and GIB only describes it
(fine, or a natural meaning nobody bothered to write down).

Counts: basic 86, convention-dependent 2, nobody 26.

### basic (86)

| position | call | kind | Brill reached / bid it | GIB | Brill | WB5 | branch |
|---|---|---|---|---|---|---|---|
| 1♣ P | 3♠ | double jump new suit | 1,610 / 20 | Splinter -- 5+ !C; 3- !H; 1- !S; 14+ total points | Responses to 1Mi-P: Splinter | 2 |  |
| 1♣ P | 3♥ | double jump new suit | 1,610 / 2 | Splinter -- 5+ !C; 1- !H; 3- !S; 14+ total points | Responses to 1Mi-P: Splinter |  |  |
| 1♦ P | 3♥ | double jump new suit | 2,133 / 1 | Splinter -- 5+ !D; 1- !H; 3- !S; 14+ total points | Responses to 1Mi-P: Splinter |  |  |
| 1♦ P | 3♠ | double jump new suit | 2,133 / 2 | Splinter -- 5+ !D; 3- !H; 1- !S; 14+ total points | Responses to 1Mi-P: Splinter |  |  |
| 3♦ P | 3♠ | simple new suit | 100 / 26 | twice rebiddable !S; 16+ total points; forcing | Responses to 3Mi-PX | 1 |  |
| 1♣ P | 3♦ | double jump new suit | 1,610 / 4 | Splinter -- 5+ !C; 1- !D; 3- !H; 3- !S; 14+ total points | Responses to 1C-P: Splinter |  |  |
| 2♦ P | 3♥ | jump new suit | 238 / 0 | strong rebiddable !H; 20+ total points | Responses to 2D-PX: Nat, forcing | 1 |  |
| 1NT P | 4♠ | double jump new suit | 1,823 / 1 |   | Responses to 1N-P |  |  |
| 1♦ P | 4♠ | triple jump new suit | 2,133 / 23 | Preemptive jump. Long suit. -- 6-10 HCP; strong rebiddable !S | Responses to 1Mi-P: Preemptive |  |  |
| 2♦ P | 3♠ | jump new suit | 238 / 0 | strong rebiddable !S; 20+ total points | Responses to 2D-PX: Nat, forcing |  |  |
| 2♠ P | 3♥ | simple new suit | 412 / 1 | rebiddable !H; 17+ total points | Responses to 2S-PX |  |  |
| 4♣ P | 4♥ | simple new suit | 7 / 0 | twice rebiddable !H; 19+ total points | Responses to 4Mi-PX |  |  |
| 2♥ P | 3♦ | simple new suit | 255 / 5 | rebiddable !D; 17+ total points; forcing | Responses to 2Ma-PX |  |  |
| 3♠ P | 3NT | cheapest NT | 59 / 14 | 2-5 !C; 2-5 !D; 2-5 !H; 2- !S; 13+ HCP; stop in !C; stop in !D; stop i | Responses to 3Ma-PX: To play, good hand opposite preempt |  |  |
| 3♠ P | 4♥ | simple new suit | 59 / 9 | 17+ HCP; twice rebiddable !H; 19+ total points | Responses to 3Ma-PX: Board: 134768, Hand: KQJ8742.2.AQJ7.9 |  |  |
| 1♣ P | 4♠ | triple jump new suit | 1,610 / 3 | Preemptive jump. Long suit. -- 6-10 HCP; strong rebiddable !S | Responses to 1Mi-P: Preemptive |  |  |
| 4♦ P | 4♠ | simple new suit | 33 / 0 | twice rebiddable !S; 19+ total points | Responses to 4Mi-PX |  |  |
| 1♣ P | 4♥ | triple jump new suit | 1,610 / 3 | Preemptive jump. Long suit. -- 6-10 HCP; strong rebiddable !H | Responses to 1Mi-P: Preemptive | 1 |  |
| 2♠ P | 4♥ | jump new suit | 412 / 0 | 6+ !H; 17+ HCP; 18+ total points | Responses to 2Ma-PX |  |  |
| 2♠ P | 3♣ | simple new suit | 412 / 0 | rebiddable !C; 17+ total points; forcing to 3S | Responses to 2Ma-PX |  |  |
| 2♥ P | 3♣ | simple new suit | 255 / 0 | rebiddable !C; 17+ total points; forcing to 3H | Responses to 2Ma-PX |  |  |
| 3♣ P | 3♥ | simple new suit | 42 / 0 | twice rebiddable !H; 16+ total points; forcing to 3N | Responses to 3Mi-PX |  |  |
| 2♠ P | 3♦ | simple new suit | 412 / 0 | rebiddable !D; 17+ total points; forcing to 3S | Responses to 2Ma-PX |  |  |
| 3♦ P | 4♣ | simple new suit | 100 / 0 | twice rebiddable !C; 19+ total points; forcing | Responses to 3Mi-PX: Board: 355143, Hand: 8.AKQ42.6.KQT732 [MS=3a-lose |  |  |
| 3♣ P | 3♠ | simple new suit | 42 / 0 | twice rebiddable !S; 16+ total points; forcing | Responses to 3Mi-PX |  |  |
| 1♥ P | 4♠ | triple jump new suit | 1,343 / 3 | 12+ HCP; strong rebiddable !S; 13 total points | Responses to 1Ma-P: preemptive |  |  |
| 2♦ P | 4♥ | double jump new suit | 238 / 0 | 6+ !H; 17+ HCP; 18+ total points | Responses to 2D-PX |  |  |
| 2♠ P | 3NT | jump NT | 412 / 3 | 1- !S; 18-21 HCP; partial stop in !C; partial stop in !D; partial stop | Responses to 2Ma-PX: Nat |  |  |
| 3♣ P | 3♦ | simple new suit | 42 / 0 | strong rebiddable !D; 17+ total points; forcing to 3N | Responses to 3C-P |  |  |
| 4♦ P | 4♥ | simple new suit | 33 / 0 | twice rebiddable !H; 19+ total points | Responses to 4Mi-PX |  |  |
| 3♦ P | 3♥ | simple new suit | 100 / 0 | twice rebiddable !H; 16+ total points; forcing to 3N | Responses to 3Mi-PX |  |  |
| 2♥ P | 3♠ | jump new suit | 255 / 0 | strong rebiddable !S; 20+ total points | Responses to 2H-PX |  |  |
| 2♦ P | 4♠ | double jump new suit | 238 / 0 | 6+ !S; 17+ HCP; 18+ total points | Responses to 2D-PX |  |  |
| 2♥ P | 4♠ | double jump new suit | 255 / 1 | 6+ !S; 17+ HCP; 18+ total points | Responses to 2Ma-PX |  |  |
| 3♥ P | 3NT | cheapest NT | 75 / 1 | 2-5 !C; 2-5 !D; 2- !H; 2-5 !S; 13+ HCP; stop in !C; stop in !D; stop i | Responses to 3Ma-PX: To play, good hand opposite preempt |  |  |
| 3♥ P | 4♠ | jump new suit | 75 / 0 | twice rebiddable !S; 19+ total points | Responses to 3Ma-PX: Board: 134768, Hand: KQJ8742.2.AQJ7.9 |  |  |
| 3♠ P | 4♦ | simple new suit | 59 / 0 | twice rebiddable !D; 19+ total points; forcing to 4S | Responses to 3Ma-PX |  |  |
| 1♦ P | 4♥ | triple jump new suit | 2,133 / 0 | Preemptive jump. Long suit. -- 6-10 HCP; strong rebiddable !H | Responses to 1Mi-P: Preemptive |  |  |
| 2♥ P | 3NT | jump NT | 255 / 0 | 1- !H; 18-21 HCP; partial stop in !C; partial stop in !D; partial stop | Responses to 2Ma-PX: Nat |  |  |
| 3♠ P | 4♣ | simple new suit | 59 / 0 | twice rebiddable !C; 19+ total points; forcing to 4S | Responses to 3Ma-PX |  |  |
| 3♥ P | 4♣ | simple new suit | 75 / 0 | twice rebiddable !C; 19+ total points; forcing to 4H | Responses to 3Ma-PX |  |  |
| 3♥ P | 4♦ | simple new suit | 75 / 0 | twice rebiddable !D; 19+ total points; forcing | Responses to 3Ma-PX |  |  |
| 3♣ P | 4♦ | jump new suit | 42 / 0 | 32- HCP; twice rebiddable !D; 20+ total points | Responses to 3Mi-PX: Board: 355143, Hand: 8.AKQ42.6.KQT732 [MS=3a-lose |  |  |
| 4♣ P | 4♠ | simple new suit | 7 / 0 | twice rebiddable !S; 19+ total points | Responses to 4Mi-PX |  |  |
| 1♣ P 2NT P | 3♣ | simple rebid of own suit | 42 / 1 | 6+ !C; 11-12 HCP; 12+ total points | Responses to 1C-P-2N-P |  |  |
| 2NT P 3♣ P | 3NT | cheapest NT | 52 / 0 | 2-5 !C; 2-5 !D; 2-5 !H; 2-5 !S; 20-21 HCP | Responses to 3C-PX (Stayman2N after 2N-P-3C): Board: DailyMP2026-08-26 |  |  |
| 1♠ P 2♣ P | 3♥ | jump new suit | 119 / 19 | 5+ !H; 5+ !S; 15-21 HCP; !HKQ; !SKQ; 22- total points; forcing to 3N | Responses to 1S-P-2Mi-PX: 5+ H |  |  |
| 1♠ P 2♦ P | 3♥ | jump new suit | 97 / 0 | 5+ !H; 5+ !S; 15-21 HCP; !HKQ; !SKQ; 22- total points; forcing to 3N | Responses to 1S-P-2Mi-PX: 5+ H |  |  |
| 3NT P 4♦ P | 4♥ | simple new suit | 0 / 0 | 2-5 !C; 2-5 !D; 2-4 !H; 2-4 !S; 25-27 HCP | Responses to 4D-PX (Texas after 3N-P-4D): Mandatory |  |  |
| 1♠ P 3♣ P | 3♠ | simple rebid of own suit | 16 / 0 | 6+ !S; 14-15 total points | Responses to 1Ma-P-3Mi-PX |  |  |
| 1NT P 3♣ P 3♦ P | 3♠ | simple new suit | 27 / 0 | Splinter -- 6+ !D; 1- !S; 9+ HCP; forcing | Responses to 3C-PX-3D-P (MinorTransfer after 1N-PX-3C): Splinter |  |  |
| 1NT P 3♣ P 3♦ P | 3♥ | simple new suit | 27 / 0 | Splinter -- 6+ !D; 1- !H; 9+ HCP; forcing to 3N | Responses to 3C-PX-3D-P (MinorTransfer after 1N-PX-3C): Splinter |  |  |
| 1♥ P 1NT P 2♣ P | 2♠ | simple new suit | 50 / 1 | Impossible spade -- 4+ !C; 2- !H; 3- !S; 11-12 total points; forcing t | Responses to 1H-P-1N-PX-2C-P: Impossible 2S, 5+ clubs, max |  |  |
| 1♦ P 2♣ P 2♦ P | 2♥ | simple new suit | 1 / 0 | 5+ !C; 4+ !H; 14+ total points; forcing to 3N | Responses to 1D-P-2C-P-2D-P |  |  |
| 1NT P 2♣ P 2♦ P | 3♣ | simple rebid of own suit | 169 / 27 | 5+ !C; 13+ total points | Responses to 2C-2D-P-P (Stayman after 1N-PX-2C) |  |  |
| 1♦ P 2♣ P 2♦ P | 2♠ | simple new suit | 1 / 1 | 5+ !C; 4+ !S; 14+ total points; forcing to 3N | Responses to 1D-P-2C-P-2D-P |  |  |
| 1NT P 2♣ P 2♥ P | 3♠ | jump new suit | 115 / 0 | forcing H raise -- 2+ !C; 2+ !D; 4+ !H; 15+ total points | Responses to 2C-2D-2Ma-PX (Stayman after 1N-PX-2C): Slam try in the ma |  |  |
| 1NT P 2♣ P 2♥ P | 4♦ | jump new suit | 115 / 0 | Splinter -- 1- !D; 4+ !H; 15+ total points | Splinter: Splinter, 4+ H, 0-1 D |  |  |
| 1♦ P 2♣ P 2♦ P | 3♦ | simple raise of partner's suit | 1 / 0 | 3+ !D; biddable !C; 14+ total points; forcing to 3N | Responses to 1D-P-2C-P-2D-P: Board: 14216, Hand: AQ7.J4.654.AQJT2 [MS= |  |  |
| 1♦ P 2♦ P 2♥ P | 4♦ | jump rebid of own suit | 1 / 0 | 5+ !D; 3- !H; 3- !S; 12+ HCP; 13+ total points | Responses to 2Mi-PX-2Ma-P (InvertedMinor after 1D-P-2D): Nat |  |  |
| 1♠ P 1NT P 2♣ P | 2♦ | simple new suit | 87 / 0 | 5+ !D; 2- !S; 6-9 HCP | Responses to 1S-P-1N-P-2C-P: Nat | 1 |  |
| 1♣ P 2♠ P 2NT P | 3♥ | simple new suit | 5 / 0 | Support for C; splinter in H -- 4+ !C; 1- !H; rebiddable !S; 17+ total | Responses to 1Mi-P-2Ma-P-2N-P: Splinter |  |  |
| 1♣ P 2♠ P 2NT P | 3♦ | simple new suit | 5 / 3 | Support for C; splinter in D -- 4+ !C; 1- !D; rebiddable !S; 17+ total | Responses to 1Mi-P-2Ma-P-2N-P: Splinter |  |  |
| 1NT P 2♣ P 2♠ P | 4♦ | jump new suit | 135 / 13 | Splinter -- 1- !D; 4+ !S; 15+ total points | Splinter: Splinter, 4+ S, 0-1 D | 1 |  |
| 1NT P 2♣ P 2♥ P | 4♣ | jump rebid of own suit | 115 / 0 | Splinter -- 1- !C; 4+ !H; 15+ total points | Splinter: Splinter, 4+ H, 0-1 C |  |  |
| 1♣ P 1♦ P 2NT P | 3♥ | simple new suit | 30 / 15 | 5+ !D; 4+ !H; 8+ total points | Responses to 1C-PX-1D-PX-2N-P | 1 |  |
| 1♣ P 1♦ P 2NT P | 3♣ | simple raise of partner's suit | 30 / 0 | 4+ !C; 4+ !D; 7+ total points | Responses to 1V-PX-1W-PX-2N-P: Nat, Forcing |  |  |
| 1♥ P 1♠ P 2♦ P | 3♣ | simple fourth suit | 49 / 0 | Fourth suit forcing -- 4+ !S; 13+ total points | 4SF: 4SF |  |  |
| 1♣ P 1♥ P 2NT P | 3♣ | simple raise of partner's suit | 2 / 0 | 4+ !C; 4+ !H; 7+ total points | Responses to 1V-PX-1W-PX-2N-P: Nat, Forcing |  |  |
| 1♦ P 1♠ P 2♥ P | 3♣ | simple fourth suit | 4 / 3 | Fourth suit forcing -- 4+ !S; 8+ HCP; forcing to 3N | 4SF: 4SF |  |  |
| 1♥ P 2♦ P 2♥ P | 3♣ | simple new suit | 58 / 19 | 4+ !C; 5+ !D; 2- !H; 13+ HCP; 14+ total points; forcing to 3N | Responses to 1Ma-P-2Mi-PX-2Ma-P: 4+ C |  |  |
| 1NT P 2♥ P 2♠ P | 4♥ | jump rebid of own suit | 60 / 0 | Splinter -- 1- !H; 6+ !S; 14-15 total points | Responses to 2H-PX-2S-P (JacobyTransfer after 1N-PX-2H): Splinter |  |  |
| 1♣ P 1♠ P 1NT P | 3♣ | jump raise of partner's suit | 99 / 12 | Jump support; not forcing -- 5+ !C; 4+ !S; 12 total points | Responses to 1Mi-PX-1Ma-PX-1N-P: Invitational, 5+ in opener's minor, 4 |  |  |
| 1♣ P 1♥ P 2♣ P | 2♠ | simple new suit | 18 / 0 | Responder reverse -- 5+ !H; biddable !S; 13+ total points; forcing to  | Responses to 1Mi-PX-1H-PX-2Mi-P: Nat | 1 |  |
| 1NT P 2♣ P 2♥ P | 3♣ | simple rebid of own suit | 115 / 0 | 5+ !C; 3- !H; 4 !S; 14+ total points | Responses to 2C-2D-2Ma-PX (Stayman after 1N-PX-2C) |  |  |
| 1NT P 2♣ P 2♠ P | 3♣ | simple rebid of own suit | 135 / 0 | 5+ !C; 4 !H; 3- !S; 14+ total points | Responses to 2C-2D-2Ma-PX (Stayman after 1N-PX-2C) |  |  |
| 1♣ P 1♥ P 2NT P | 3♥ | simple rebid of own suit | 2 / 0 | Rebid suit -- 6+ !H; 6+ HCP; 7+ total points | NewMinorForcing2N: 6+ card major, strong -- game forcing, slam interes |  |  |
| 1NT P 2♣ P 2♦ P | 3♦ | simple raise of partner's suit | 169 / 0 | 5+ !D; 13+ total points | Responses to 2C-2D-P-P (Stayman after 1N-PX-2C) |  |  |
| 1NT P 2♣ P 2♠ P | 4♣ | jump rebid of own suit | 135 / 0 | Splinter -- 1- !C; 4+ !S; 15+ total points | Splinter: Splinter, 4+ S, 0-1 C |  |  |
| 2NT P 3♣ P 3♥ P | 3♠ | simple new suit | 13 / 0 | forcing H raise -- 4+ !H; 10+ total points | Responses to 3C-PX-3H-PX (Stayman2N after 2N-P-3C): Board: 733726, Han |  |  |
| 1NT P 2♣ P 2♥ P | 3♦ | simple new suit | 115 / 0 | 5+ !D; 3- !H; 4 !S; 14+ total points | Responses to 2C-2D-2Ma-PX (Stayman after 1N-PX-2C) |  |  |
| 1♣ P 1♠ P 2♥ P | 3♦ | simple fourth suit | 12 / 0 | Fourth suit forcing -- 4+ !S; 8+ HCP; forcing to 3N | 4SF: 4SF |  |  |
| 1♦ P 2♣ P 3♦ P | 3♥ | simple new suit | 0 / 0 | 5+ !C; 4+ !H; 14+ total points; forcing to 3N | Responses to 1D-P-2C-P-3D-P |  |  |
| 1♣ P 1♠ P 2♣ P | 2♦ | simple new suit | 63 / 6 | New suit -- 3+ !D; 4+ !S; 12+ total points; forcing to 3N | Responses to 1C-PX-1Ma-PX-2C-P: Nat |  |  |
| 1NT P 2♣ P 2♠ P | 3♦ | simple new suit | 135 / 1 | 5+ !D; 4 !H; 3- !S; 14+ total points | Responses to 2C-2D-2Ma-PX (Stayman after 1N-PX-2C) |  |  |
| 1♣ P 1♦ P 2NT P | 3♠ | simple new suit | 30 / 14 | 5+ !D; 4+ !S; 8+ total points | Responses to 1C-PX-1D-PX-2N-P |  |  |

### convention-dependent (2)

| position | call | kind | Brill reached / bid it | GIB | Brill | WB5 | branch |
|---|---|---|---|---|---|---|---|
| 1♦ P | 4♣ | double jump new suit | 2,133 / 0 | Splinter -- 1- !C; 5+ !D; 3- !H; 3- !S; 14+ total points | : Inferred ((C >= 6 and competitivevalues) or competitive('C')) |  |  |
| 4♣ P | 4♦ | simple new suit | 7 / 0 | Cue bid -- 19+ total points | : Inferred (fourlevelovercall('D')) |  |  |

### nobody defines it (26)

| position | call | kind | Brill reached / bid it | GIB | Brill | WB5 | branch |
|---|---|---|---|---|---|---|---|
| 2♦ P | 4♣ | jump new suit | 238 / 0 |   | : Inferred (fourlevelovercall('C')) |  |  |
| 2♦ P | 3♣ | simple new suit | 238 / 0 | rebiddable !C; 17+ total points; forcing | : Inferred (twicerebiddable('C') and hcp >= 12) |  |  |
| 2♣ P | 3♠ | jump new suit | 159 / 0 |   | : Inferred (competitivevalues and twicerebiddable('S')) |  |  |
| 2♣ P | 3♥ | jump new suit | 159 / 0 |   | : Inferred (competitivevalues and twicerebiddable('H')) |  |  |
| 3♥ P | 3♠ | simple new suit | 75 / 7 | twice rebiddable !S; 16+ total points; forcing | : Inferred (twicerebiddable('S') and hcp >= 12) |  |  |
| 1♣ P | 4♣ | double jump raise of partner's suit | 1,610 / 0 | biddable !C | : Inferred (C_invite) |  |  |
| 1♦ P | 4♦ | double jump raise of partner's suit | 2,133 / 0 | biddable !D | : Inferred (D_invite) |  |  |
| 2♣ P | 4♣ | jump raise of partner's suit | 159 / 0 |   | : Inferred (C_invite) |  |  |
| 1♣ P | 4♦ | triple jump new suit | 1,610 / 0 | biddable !D | : Inferred ((D >= 6 and competitivevalues) or competitive('D')) |  |  |
| 2♣ P | 4♦ | double jump new suit | 159 / 0 |   | : Inferred ((D >= 6 and competitivevalues) or competitive('D')) |  |  |
| 2♠ P | 4♣ | jump new suit | 412 / 0 |   | : Inferred (fourlevelovercall('C')) |  |  |
| 2♠ P | 4♦ | jump new suit | 412 / 0 |   | : Inferred (fourlevelovercall('D')) |  |  |
| 2♥ P | 4♣ | jump new suit | 255 / 0 |   | : Inferred (fourlevelovercall('C')) |  |  |
| 2♥ P | 4♦ | jump new suit | 255 / 0 |   | : Inferred (fourlevelovercall('D')) |  |  |
| 4♥ P | 4♠ | simple new suit | 21 / 0 | twice rebiddable !S; 19+ total points | : Inferred (S_game or (twicerebiddable('S') and losers <= 3)) |  |  |
| 1NT P 3♣ P | 3NT | cheapest NT | 37 / 4 | 2-5 !C; 2-5 !D; 2-4 !H; 2-4 !S; 17- HCP; 18 total points | : Inferred ((game or NT_trickgame) and not slam and stoppersOK) |  |  |
| 1NT P 2NT P | 3NT | cheapest NT | 15 / 0 | 2-5 !C; 2-5 !D; 2-5 !H; 2-5 !S; 17- HCP; 18 total points | : Inferred ((game or NT_trickgame) and not slam and stoppersOK) |  |  |
| 1♥ P 2♣ P | 3♦ | jump new suit | 143 / 1 | 5+ !D; 5+ !H; 15-21 HCP; !DKQ; !HKQ; 22- total points; forcing to 3N | : Inferred (competitivevalues and twicerebiddable('D')) |  |  |
| 1♠ P 2♣ P | 3♦ | jump new suit | 119 / 0 | 5+ !D; 5+ !S; 15-21 HCP; !DKQ; !SKQ; 22- total points; forcing to 3N | : Inferred (competitivevalues and twicerebiddable('D')) |  |  |
| 1♠ P 3♠ P | 3NT | cheapest NT | 54 / 2 | 4- !H; 5-6 !S; 17-21 HCP; 22- total points; partial stop in !C; partia | : Inferred ((game or NT_trickgame) and not slam and stoppersOK) |  |  |
| 2♦ P 2♥ P | 2♠ | simple new suit | 18 / 0 | 1-4 !C; 6+ !D; 1-3 !H; 3 !S; 10- HCP; 7+ total points; partial stop in | : Inferred (competitivevalues and S >= 4 and D >= 6) | 1 |  |
| 1♥ P 3♠ P | 3NT | cheapest NT | 0 / 0 | 5-6 !H; 4- !S; 14-21 HCP; 22- total points; partial stop in !C; partia | : Inferred ((game or NT_trickgame) and not slam and stoppersOK) |  | gf-after-2nt: game-force:50 |
| 1♠ P 2NT P 3♠ P | 3NT | cheapest NT | 1 / 0 | 4- !H; 4-5 !S; 14- HCP; 13+ total points | : Inferred ((game or NT_trickgame) and not slam and stoppersOK) |  | gf-after-2nt: game-force:50 |
| 1♥ P 2NT P 3♠ P | 3NT | cheapest NT | 31 / 0 | 4-5 !H; 4- !S; 14-21 HCP; partial stop in !C; partial stop in !D; part | : Inferred ((game or NT_trickgame) and not slam and stoppersOK) |  | gf-after-2nt: game-force:50 |
| 1♠ P 2NT P 3♥ P | 3NT | cheapest NT | 2 / 0 | 4- !H; 4-5 !S; 14-21 HCP; partial stop in !C; partial stop in !D; part | : Inferred ((game or NT_trickgame) and not slam and stoppersOK) |  | gf-after-2nt: game-force:50 |
| 1♥ P 1NT P 2♣ P | 3NT | jump NT | 50 / 0 | 2- !H; 3- !S; 12+ HCP; 12- total points; partial stop in !D; partial s | : Inferred ((game or NT_trickgame) and not slam and stoppersOK) |  |  |

### Competitive holes, one level

The 234 holes through 3NT of section 3 at positions the Brill field reached at least
ten times, the same test. Counts: basic 149, nobody defines it 62, convention-dependent 23.

| position | call | Brill reached / bid it | GIB | Brill | class |
|---|---|---|---|---|---|
| 1♦ | 3♦ | 3,345 / 0 | Aggressive weak jump overcall -- 7+ !D; 10- HCP; 3+ total po | Responses to 1Mi: Preemptive | basic |
| 1♣ | 3♣ | 2,898 / 1 | Aggressive weak jump overcall -- 7+ !C; 10- HCP; 3+ total po | Responses to 1Mi: Preemptive | basic |
| 1NT P P | 3♣ | 455 / 0 | strong rebiddable !C; 14-16 total points | Balancing 1N: Strong rebiddable, 14-16 totalpoints | basic |
| 1NT P P | 3♦ | 455 / 1 | strong rebiddable !D; 14-16 total points | Balancing 1N: Strong rebiddable, 14-16 totalpoints | basic |
| 1NT P P | 3♥ | 455 / 13 | strong rebiddable !H; 14-16 total points | Balancing 1N: Strong rebiddable, 14-16 totalpoints | basic |
| 1NT P P | 3♠ | 455 / 1 | strong rebiddable !S; 14-16 total points | Balancing 1N: Strong rebiddable, 14-16 totalpoints | basic |
| 1♦ 1♠ | 3♣ | 397 / 0 | Free bid -- rebiddable !C; 15+ total points | Responses to 1Mi-1Ma: Nat | basic |
| 1♦ 1♠ | 3♥ | 397 / 0 | Free bid -- rebiddable !H; 15+ total points | Responses to 1Mi-1Ma: Nat | basic |
| 1♣ 1♥ | 3♦ | 350 / 0 | Free bid -- rebiddable !D; 15+ total points | Responses to 1Mi-1Ma: Nat | basic |
| 1♣ 1♥ | 3♠ | 350 / 0 | biddable !S | Responses to 1Mi-1Ma: Nat | basic |
| 1♥ 1♠ | 3♠ | 324 / 0 | Splinter -- 4+ !H; 1- !S; 13+ total points | Responses to 1H-1S: Splinter | basic |
| 1♣ 1♠ | 3♦ | 288 / 0 | Free bid -- rebiddable !D; 15+ total points | Responses to 1Mi-1Ma: Nat | basic |
| 1♣ 1♠ | 3♥ | 288 / 0 | Free bid -- rebiddable !H; 15+ total points | Responses to 1Mi-1Ma: Nat | basic |
| 1♦ P P | 2♦ | 274 / 14 | Michaels -- 5+ !H; 5+ !S; 9+ total points; forcing | MichaelsCuebidMinor: 5-5 in Majors | basic |
| 1♦ P P | 2♥ | 274 / 1 | Balancing jump overcall -- twice rebiddable !H; 13+ total po | Balancing 1Mi Opening | basic |
| 1♦ P P | 2♠ | 274 / 2 | Balancing jump overcall -- twice rebiddable !S; 13+ total po | Balancing 1Mi Opening | basic |
| 1♦ P P | 3♥ | 274 / 0 | Balancing double jump overcall -- 7+ !H; strong rebiddable ! | Balancing 1Mi Opening | basic |
| 1♦ P P | 3♠ | 274 / 0 | Balancing double jump overcall -- 7+ !S; strong rebiddable ! | Balancing 1Mi Opening | basic |
| 1♦ P P | 3NT | 274 / 0 | 5- !H; 5- !S; 25-32 HCP; likely stop in !D | Balancing 1Mi Opening | basic |
| 1♦ X | 2♣ | 246 / 0 | Free bid; new suit -- 2- !D; rebiddable !C; 9-11 total point | Responses to 1D-X: Nat | basic |
| 1♦ X | 2NT | 246 / 3 | Truscott (Jordan) -- 5+ !D; 3- !H; 3- !S; 11+ total points | TruscottMinor: TruscottMinor, 10+ hcp, 5+ D | basic |
| 1♦ X | 3♦ | 246 / 0 | 5+ !D; 9- HCP; 8+ total points | Responses to 1Mi-X: Good suit, 7-10 hcp | basic |
| 1♦ X | 3♥ | 246 / 0 | Preempt -- 7+ !H; 7- HCP; !HQ | Responses to 1Mi-X: 7+ H, 6-9 hcp | basic |
| 1♦ X | 3♠ | 246 / 0 | Preempt -- 7+ !S; 7- HCP; !SQ | Responses to 1Mi-X: 7+ S, 6-9 hcp | basic |
| 1♣ 1♦ | 3♥ | 242 / 0 | biddable !H | Responses to 1C-1D: Nat | basic |
| 1♣ 1♦ | 3♠ | 242 / 0 | biddable !S | Responses to 1C-1D: Nat | basic |
| 1♦ 1♥ | 3♣ | 239 / 0 | Free bid -- rebiddable !C; 15+ total points | Responses to 1Mi-1Ma: Nat | basic |
| 1♦ 1♥ | 3♠ | 239 / 0 | biddable !S | Responses to 1Mi-1Ma: Nat | basic |
| 1♠ X | 2♣ | 233 / 14 | Free bid; new suit -- 2- !S; rebiddable !C; 9-11 total point | Responses to 1Ma-X: Free bid | basic |
| 1♠ X | 2♦ | 233 / 10 | Free bid; new suit -- 2- !S; rebiddable !D; 9-11 total point | Responses to 1Ma-X: Free bid | basic |
| 1♠ X | 2♥ | 233 / 13 | Free bid; new suit -- 2- !S; rebiddable !H; 9-11 total point | Responses to 1S-X: Free bid | basic |
| 1♠ X | 3NT | 233 / 1 | 5- !H; 5- !S; 14-21 HCP; likely stop in !C; likely stop in ! | Responses to 1Ma-X: Nat | basic |
| 2♣ | 2♦ | 189 / 0 | twice rebiddable !D; 9+ total points | Overcalling 2C Opener | basic |
| 2♣ | 2♥ | 189 / 21 | twice rebiddable !H; 9+ total points | Responses to 2C | basic |
| 2♣ | 2♠ | 189 / 6 | twice rebiddable !S; 9+ total points | Responses to 2C | basic |
| 2♣ | 3♣ | 189 / 0 | Preempt -- 7+ !C; 5-9 HCP; !CQ; 6+ total points | Overcalling 2C Opener | basic |
| 2♣ | 3♦ | 189 / 3 | Preempt -- 7+ !D; 5-9 HCP; !DQ; 6+ total points | Overcalling 2C Opener | basic |
| 2♣ | 3♥ | 189 / 0 | Preempt -- 7+ !H; 5-9 HCP; !HQ; 6+ total points | Responses to 2C | basic |
| 2♣ | 3♠ | 189 / 0 | Preempt -- 7+ !S; 5-9 HCP; !SQ; 6+ total points | Responses to 2C | basic |
| 2♣ | X | 189 / 0 | 6+ HCP; rebiddable !C; !CKQ | Overcalling 2C Opener: Board: T291810, Hand: KQJ3.T983.5.T76 | basic |
| 1♠ P P | 2♠ | 187 / 0 | Michaels -- 5+ !H; 9+ total points; forcing | MichaelsCuebidMajor: 5-5 in other Major and a minor | basic |
| 1♠ P P | 3♣ | 187 / 21 | Balancing jump overcall -- strong rebiddable !C; 13+ total p | Balancing 1Ma Opening | basic |
| 1♠ P P | 3♦ | 187 / 0 | Balancing jump overcall -- strong rebiddable !D; 13+ total p | Balancing 1Ma Opening | basic |
| 1♠ P P | 3♥ | 187 / 0 | Balancing jump overcall -- strong rebiddable !H; 13+ total p | Balancing 1S Opening | basic |
| 1♠ P P | 3NT | 187 / 0 | 5- !H; 5- !S; 25-32 HCP; likely stop in !S | Balancing 1Ma Opening | basic |
| 1♣ X | 2NT | 130 / 2 | Truscott (Jordan) -- 5+ !C; 3- !H; 3- !S; 11+ total points | TruscottMinor: TruscottMinor, 10+ hcp, 5+ C | basic |
| 1♣ X | 3♣ | 130 / 0 | 5+ !C; 9- HCP; 8+ total points | Responses to 1Mi-X: Good suit, 7-10 hcp | basic |
| 1♣ X | 3♥ | 130 / 0 | Preempt -- 7+ !H; 7- HCP; !HQ | Responses to 1Mi-X: 7+ H, 6-9 hcp | basic |
| 1♣ X | 3♠ | 130 / 0 | Preempt -- 7+ !S; 7- HCP; !SQ | Responses to 1Mi-X: 7+ S, 6-9 hcp | basic |
| 1♥ 1♠ P | 3NT | 107 / 0 | 2- !S; 15-18 HCP; likely stop in !H | Responses to 1H-1S-P | basic |
| 1♥ X | 2♣ | 102 / 0 | Free bid; new suit -- 2- !H; rebiddable !C; 9-11 total point | Responses to 1Ma-X: Free bid | basic |
| 1♥ X | 2♦ | 102 / 0 | Free bid; new suit -- 2- !H; rebiddable !D; 9-11 total point | Responses to 1Ma-X: Free bid | basic |
| 1♥ X | 3♠ | 102 / 6 | Preempt -- 7+ !S; 7- HCP; !SQ | Splinter: Splinter, 4+ H, 0-1 S | basic |
| 1♥ X | 3NT | 102 / 0 | 5- !H; 5- !S; 14-21 HCP; likely stop in !C; likely stop in ! | Responses to 1Ma-X: Nat | basic |
| 1♥ 2♣ | 3♦ | 93 / 0 | Free bid -- rebiddable !D; 15+ total points | Responses to 1Ma-2C | basic |
| 2♠ X | 2NT | 87 / 0 | Artificial force -- 15+ HCP | FeatureAskMajor: FeatureAsk, 15+ hcp | basic |
| 2♠ X | 3♣ | 87 / 0 |   | Responses to 2Ma-PX | basic |
| 2♠ X | 3♦ | 87 / 0 |   | Responses to 2Ma-PX | basic |
| 2♠ X | 3♥ | 87 / 0 | rebiddable !H; 17+ total points | Responses to 2S-PX | basic |
| 2♠ X | 3NT | 87 / 0 | 1- !S; 18-21 HCP; stop in !C; stop in !D; stop in !H | Responses to 2Ma-PX: Nat | basic |
| 1♣ 2♥ | 3♠ | 86 / 0 | Free bid -- rebiddable !S; 15+ total points | Responses to 1Mi-2Ma | basic |
| 1♣ P P | 2♣ | 85 / 0 | Michaels -- 5+ !H; 5+ !S; 9+ total points; forcing | MichaelsCuebidMinor: 5-5 in Majors | basic |
| 1♣ P P | 2♦ | 85 / 0 | Balancing jump overcall -- twice rebiddable !D; 13+ total po | Balancing 1C Opening | basic |
| 1♣ P P | 2♥ | 85 / 0 | Balancing jump overcall -- twice rebiddable !H; 13+ total po | Balancing 1Mi Opening | basic |
| 1♣ P P | 2♠ | 85 / 9 | Balancing jump overcall -- twice rebiddable !S; 13+ total po | Balancing 1Mi Opening | basic |
| 1♣ P P | 3♥ | 85 / 0 | Balancing double jump overcall -- 7+ !H; strong rebiddable ! | Balancing 1Mi Opening | basic |
| 1♣ P P | 3♠ | 85 / 0 | Balancing double jump overcall -- 7+ !S; strong rebiddable ! | Balancing 1Mi Opening | basic |
| 1♣ P P | 3NT | 85 / 0 | 5- !H; 5- !S; 25-32 HCP; likely stop in !C | Balancing 1Mi Opening | basic |
| 1NT 2♣ | 2♠ | 84 / 2 | Minor Stayman -- 4+ !C; 4+ !D; 10+ total points | MinorStayman: MinorStayman -- both minors, no 4-card major | basic |
| 1NT 2♣ | 3♣ | 84 / 0 | Minor transfer -- 6+ !D | MinorTransfer: MinorTransfer | basic |
| 1NT 2♣ | 3♦ | 84 / 0 | Splinter -- 4-5 !C; 1- !D; 4 !H; 4 !S; 12+ total points | SplinterOver1N: Splinter | basic |
| 1NT 2♣ | 3♥ | 84 / 0 | Splinter -- 4-5 !C; 4-5 !D; 1- !H; 4 !S; 12+ total points | SplinterOver1N: Splinter | basic |
| 1NT 2♣ | 3♠ | 84 / 0 | Splinter -- 4-5 !C; 4-5 !D; 4 !H; 1- !S; 12+ total points | SplinterOver1N: Splinter | basic |
| 2♦ 2♠ | 2NT | 78 / 5 | Invitational to 3NT game -- 19 HCP; stop in !S | Responses to 2D-2Ma: Board: Challengebyhwaugh74_3e253394_b4 | basic |
| 2♦ 2♠ | 3♣ | 78 / 0 | 4+ !C; 2- !D; 18+ total points | Responses to 2D-2Ma: Board: T203714, Hand: J32.KQ.7.AKQJ976 | basic |
| 2♦ 2♠ | 3♥ | 78 / 0 | 2- !D; 4+ !H; 18+ total points | Responses to 2D-2Ma: Nat, forcing | basic |
| 2♦ 2♠ | 3♠ | 78 / 0 | 2+ !D; 17+ HCP; 18+ total points; forcing | Responses to 2D-2Ma: Nat, forcing | basic |
| 2♦ 2♠ | 3NT | 78 / 0 | 2- !D; 17-20 HCP; stop in !S | Responses to 2D-2Ma: Nat | basic |
| 1♦ 2♣ | 3♦ | 75 / 6 | Preemptive raise -- 5+ !D; 4-9 total points | Responses to 1D-2C | basic |
| 1♦ 2♣ | 3♥ | 75 / 0 | Free bid -- rebiddable !H; 15+ total points | Responses to 1D-2C | basic |
| 1♦ 2♣ | 3♠ | 75 / 0 | Free bid -- rebiddable !S; 15+ total points | Responses to 1D-2C | basic |
| 1♦ 1♠ P | 3NT | 62 / 2 | 2- !S; 15-18 HCP; likely stop in !D | Responses to 1Mi-1Ma-PX | basic |
| 2♦ X | 2♥ | 59 / 17 | rebiddable !H; 14+ total points; forcing to 3D | Responses to 2D-PX: Nat, forcing | basic |
| 2♦ X | 2♠ | 59 / 0 | rebiddable !S; 14+ total points; forcing to 3D | Responses to 2D-PX: Nat, forcing | basic |
| 2♦ X | 2NT | 59 / 0 | Artificial force -- 15+ HCP | FeatureAskDiamond: FeatureAsk, 15+ hcp | basic |
| 2♦ X | 3♥ | 59 / 0 | strong rebiddable !H; 20+ total points | Responses to 2D-PX: Nat, forcing | basic |
| 2♦ X | 3♠ | 59 / 0 | strong rebiddable !S; 20+ total points | Responses to 2D-PX: Nat, forcing | basic |
| 2♦ X | 3NT | 59 / 0 | 18-21 HCP; stop in !C; stop in !H; stop in !S | Responses to 2D-X: Board: 804882, Hand: AK3.KQ932.52.AKJ [MS | basic |
| 2♦ X | XX | 59 / 2 | 2- !D; 17+ HCP | Responses to 2D-X: Board: 77010, Hand: J75.AKQ65.KQ5.A5 [MS= | basic |
| 1♥ P P | 2♥ | 57 / 0 | Michaels -- 5+ !S; 9+ total points; forcing | MichaelsCuebidMajor: 5-5 in other Major and a minor | basic |
| 1♥ P P | 2♠ | 57 / 0 | Balancing jump overcall -- twice rebiddable !S; 13+ total po | Balancing 1H Opening | basic |
| 1♥ P P | 3♣ | 57 / 0 | Balancing jump overcall -- strong rebiddable !C; 13+ total p | Balancing 1Ma Opening | basic |
| 1♥ P P | 3♦ | 57 / 0 | Balancing jump overcall -- strong rebiddable !D; 13+ total p | Balancing 1Ma Opening | basic |
| 1♥ P P | 3NT | 57 / 0 | 5- !H; 5- !S; 25-32 HCP; likely stop in !H | Balancing 1Ma Opening | basic |
| 2♥ 2♠ | 2NT | 57 / 0 | Invitational to 3NT game -- 19 HCP; stop in !S | Responses to 2H-2S: Nat Inv | basic |
| 2♥ 2♠ | 3♣ | 57 / 0 | 4+ !C; 1- !H; 18+ total points | Responses to 2H-2S | basic |
| 2♥ 2♠ | 3♦ | 57 / 0 | 4+ !D; 1- !H; 18+ total points | Responses to 2H-2S | basic |
| 2♥ 2♠ | 3NT | 57 / 0 | 5- !H; 5- !S; 20-27 HCP; likely stop in !S | Responses to 2H-2S: Nat | basic |
| 1♥ 1NT | 3♠ | 52 / 0 | 7+ !S; 8- HCP | Responses to 1Ma-1N: Board: 20260509_MP_006_WFI-Tournament | basic |
| 1♦ 2♦ | 2♠ | 41 / 5 | Constructive, 4th suit -- 5+ !C; 10+ total points | Responses to 1Mi-2Mi: Board: 387, Hand: K9872.AQJT96.J.6 [MS | basic |
| 1♠ 1NT | 3♥ | 41 / 0 | 7+ !H; 8- HCP | Responses to 1Ma-1N: Board: 20260509_MP_006_WFI-Tournament | basic |
| 2♥ X | 2♠ | 39 / 0 | rebiddable !S; 14+ total points; forcing to 3H | Responses to 2H-PX | basic |
| 2♥ X | 2NT | 39 / 0 | Artificial force -- 15+ HCP | FeatureAskMajor: FeatureAsk, 15+ hcp | basic |
| 2♥ X | 3♣ | 39 / 0 |   | Responses to 2Ma-PX | basic |
| 2♥ X | 3♦ | 39 / 0 |   | Responses to 2Ma-PX | basic |
| 2♥ X | 3♠ | 39 / 0 | 2+ !H; 1- !S; 17+ HCP; 18+ total points; forcing | Responses to 2H-PX | basic |
| 2♥ X | 3NT | 39 / 0 | 1- !H; 18-21 HCP; stop in !C; stop in !D; stop in !S | Responses to 2Ma-PX: Nat | basic |
| 1♥ 2♣ P | 3♠ | 38 / 0 | 4+ !S; 13+ HCP; 14+ total points; forcing | Responses to 1Ma-2Mi-P | basic |
| 1NT 2♥ P | 3NT | 37 / 0 | 5- !H; 5- !S; 16-23 HCP; partial stop in !C; partial stop in | Responses to 2Ma-PX (Cappelletti after 1N-2Ma): Board: 96588 | basic |
| 1♣ 1♥ P | 3NT | 29 / 0 | 2- !H; 15-18 HCP; likely stop in !C | Responses to 1Mi-1Ma-PX | basic |
| 2♥ 3♣ | 3♦ | 28 / 0 | 4+ !D; 1- !H; 18+ total points | Responses to 2Ma-3C: Board: 786012, Hand: KQ973.2.AQJ6432. | basic |
| 2♥ 3♣ | 3♠ | 28 / 0 | 1- !H; 4+ !S; 18+ total points | Responses to 2Ma-3Mi | basic |
| 2♥ 3♣ | 3NT | 28 / 0 | 5- !H; 5- !S; 20-27 HCP; likely stop in !C | Responses to 2Ma-3Mi: Nat | basic |
| 2♦ 3♣ | 3♥ | 27 / 0 | 2- !D; 4+ !H; 18+ total points | Responses to 2D-3C: Nat, forcing | basic |
| 2♦ 3♣ | 3♠ | 27 / 0 | 2- !D; 4+ !S; 18+ total points | Responses to 2D-3C: Nat, forcing | basic |
| 2♦ 3♣ | 3NT | 27 / 0 | 5- !H; 5- !S; 20-27 HCP; likely stop in !C | Responses to 2D-3C: Nat | basic |
| 2♦ 2♥ | 2♠ | 24 / 0 | 2- !D; 4+ !S; 16+ total points | Responses to 2D-2H | basic |
| 2♦ 2♥ | 2NT | 24 / 0 | Invitational to 3NT game -- 19 HCP; stop in !H | Responses to 2D-2Ma: Board: Challengebyhwaugh74_3e253394_b4 | basic |
| 2♦ 2♥ | 3♣ | 24 / 5 | 4+ !C; 2- !D; 18+ total points | Responses to 2D-2Ma: Board: T203714, Hand: J32.KQ.7.AKQJ976 | basic |
| 2♦ 2♥ | 3♥ | 24 / 0 | 2+ !D; 17+ HCP; 18+ total points; forcing to 3N | Responses to 2D-2Ma: Nat, forcing | basic |
| 2♦ 2♥ | 3♠ | 24 / 0 | 5+ !S; 18+ total points; forcing | Responses to 2D-2Ma: Nat, forcing | basic |
| 2♦ 2♥ | 3NT | 24 / 0 | 2- !D; 17-20 HCP; stop in !H | Responses to 2D-2Ma: Nat | basic |
| 2♠ 3NT | X | 23 / 8 |   | Responses to 2Ma-3N | basic |
| 1♠ 2♣ | 3♦ | 22 / 0 | Free bid -- rebiddable !D; 15+ total points | Responses to 1Ma-2C | basic |
| 1♣ X P | 2♦ | 21 / 0 | Jump new suit -- 4+ !D; 9-12 total points | Responses to 1C-X-P | basic |
| 2♣ 2♥ | 2♠ | 21 / 0 | Positive -- 5+ !S; 8+ total points; forcing | Responses to 2C-2H | basic |
| 2♣ 2♥ | 2NT | 21 / 1 | Positive notrump -- 2-5 !C; 2-5 !D; 2-4 !H; 2-4 !S; 8+ HCP | Responses to 2C-2Ma | basic |
| 2♣ 2♥ | 3♣ | 21 / 0 | Positive -- 5+ !C; 8+ total points | Responses to 2C-2Ma | basic |
| 2♣ 2♥ | 3♦ | 21 / 0 | Positive -- 5+ !D; 8+ total points | Responses to 2C-2Ma | basic |
| 2♣ 2♥ | 3♥ | 21 / 0 | Shortness in H -- 1- !H; 8+ total points | Responses to 2C-2Ma | basic |
| 2♣ 2♥ | X | 21 / 0 | Penalty double -- 5+ !H; 8+ total points; forcing to 2N | Responses to 2C-2Ma: Penalty | basic |
| 2♥ 2NT | 3♣ | 21 / 0 | 4+ !C; 1- !H; 18+ total points | Responses to 2Ma-2N: Hand: T.A42.4.AJT76542 (Partscore) | basic |
| 2♥ 2NT | 3♦ | 21 / 0 | 4+ !D; 1- !H; 18+ total points | Responses to 2Ma-2N: Hand: T.A42.4.AJT76542 (Partscore) | basic |
| 2♥ 2NT | 3♠ | 21 / 0 | 1- !H; 4+ !S; 18+ total points | Responses to 2Ma-2N | basic |
| 2♥ 2NT | 3NT | 21 / 0 | 5- !H; 5- !S; 20-27 HCP; partial stop in !C; partial stop in | Responses to 2Ma-2N | basic |
| 2♥ 2NT | X | 21 / 0 |   | Responses to 2Ma-2N | basic |
| 1♣ 2♣ | 2♠ | 20 / 18 | Constructive, 4th suit -- 5+ !D; 10+ total points | Responses to 1Mi-2Mi: Board: 387, Hand: K9872.AQJT96.J.6 [MS | basic |
| 1NT 2♦ | 2NT | 20 / 0 | Lebensohl - Forces 3C by partner --   | Lebensohl1N: Lebensohl1N | basic |
| 1NT 2♦ | 3♥ | 20 / 10 | 10+ HCP; stop in !H; forcing to 3N | Lebensohl1N: Lebensohl1N | basic |
| 1NT 2♦ | 3♠ | 20 / 0 | 10+ HCP; stop in !S; forcing | Lebensohl1N: Lebensohl1N | basic |
| 2♠ 2NT | 3♣ | 19 / 0 | 4+ !C; 1- !S; 18+ total points | Responses to 2Ma-2N: Hand: T.A42.4.AJT76542 (Partscore) | basic |
| 2♠ 2NT | 3♦ | 19 / 0 | 4+ !D; 1- !S; 18+ total points | Responses to 2Ma-2N: Hand: T.A42.4.AJT76542 (Partscore) | basic |
| 2♠ 2NT | 3♥ | 19 / 0 | 4+ !H; 1- !S; 18+ total points | Responses to 2Ma-2N | basic |
| 2♠ 2NT | 3NT | 19 / 0 | 5- !H; 5- !S; 20-27 HCP; partial stop in !C; partial stop in | Responses to 2Ma-2N | basic |
| 2♠ 2NT | X | 19 / 0 |   | Responses to 2Ma-2N | basic |
| 2♠ 3♦ | 3♥ | 16 / 0 | 4+ !H; 1- !S; 18+ total points | Responses to 2Ma-3Mi | basic |
| 2♠ 3♦ | 3NT | 16 / 0 | 5- !H; 5- !S; 20-27 HCP; likely stop in !D | Responses to 2Ma-3Mi: Nat | basic |
| 1♥ X P | 3♥ | 13 / 0 | 11+ total points | Responses to 1Ma-X-P | basic |
| 2♠ 3♥ | 3NT | 13 / 0 | 5- !H; 5- !S; 20-27 HCP; likely stop in !H | Responses to 2S-3H: Nat | basic |
| 1♥ 1♠ | 3♣ | 324 / 1 | Free bid -- rebiddable !C; 15+ total points | (Inferred (competitivevalues and twicerebiddable('C'))) | convention-dependent |
| 1♥ 1♠ | 3♦ | 324 / 0 | Free bid -- rebiddable !D; 15+ total points | (Inferred (competitivevalues and twicerebiddable('D'))) | convention-dependent |
| 1♦ P P | 3♣ | 274 / 0 | Balancing jump overcall -- strong rebiddable !C; 13+ total p | (Inferred (overcall('C'))) | convention-dependent |
| 1♥ 2♣ | 3♠ | 93 / 0 | Free bid -- rebiddable !S; 15+ total points | (Inferred (competitivevalues and twicerebiddable('S'))) | convention-dependent |
| 1♣ P P | 3♦ | 85 / 0 | Balancing double jump overcall -- 7+ !D; strong rebiddable ! | (Inferred (overcall('D'))) | convention-dependent |
| 1♥ P P | 3♠ | 57 / 0 | Balancing double jump overcall -- 7+ !S; strong rebiddable ! | (Inferred (overcall('S'))) | convention-dependent |
| 1♣ 2♦ | 3♥ | 55 / 0 | Free bid -- rebiddable !H; 15+ total points | (Inferred (competitivevalues and twicerebiddable('H'))) | convention-dependent |
| 1♣ 2♦ | 3♠ | 55 / 0 | Free bid -- rebiddable !S; 15+ total points | (Inferred (competitivevalues and twicerebiddable('S'))) | convention-dependent |
| 1♥ 2♥ | 3♠ | 48 / 0 | Limit raise or better in H -- 3+ !H; 1- !S; 12+ HCP; 13+ tot | (Inferred (game and not stopper('S'))) | convention-dependent |
| 1♥ 2♥ | X | 48 / 0 | Penalty double -- 10+ HCP | (Inferred (suitshowing('H'))) | convention-dependent |
| 1♦ 2♦ | 3♥ | 41 / 0 | Limit raise or better in D -- 4+ !D; 10+ total points | (Inferred (game and not stopper('H'))) | convention-dependent |
| 1♦ 2♦ | X | 41 / 2 | Penalty double -- 10+ HCP | (Inferred (suitshowing('D'))) | convention-dependent |
| 1♠ 2♦ | 3♥ | 34 / 0 | Free bid -- rebiddable !H; 15+ total points | (Inferred (competitivevalues and twicerebiddable('H'))) | convention-dependent |
| 1♠ 2♣ | 3♥ | 22 / 0 | Free bid -- rebiddable !H; 15+ total points | (Inferred (competitivevalues and twicerebiddable('H'))) | convention-dependent |
| 1♣ 2♣ | 3♦ | 20 / 0 | Natural, not forcing -- 2- !C; twice rebiddable !D; 7-9 tota | (Inferred (competitivevalues and twicerebiddable('D'))) | convention-dependent |
| 1♣ 2♣ | 3♥ | 20 / 0 | Limit raise or better in C -- 4+ !C; 10+ total points | (Inferred (game and not stopper('H'))) | convention-dependent |
| 1♣ 2♣ | X | 20 / 0 | Penalty double -- 10+ HCP | (Inferred (suitshowing('C'))) | convention-dependent |
| 1♠ 2♠ | X | 18 / 0 | Penalty double -- 10+ HCP | (Inferred (suitshowing('S'))) | convention-dependent |
| 1♣ 2NT | 3♥ | 17 / 2 | Constructive, 4th suit -- 5+ !S; 10+ total points | (Inferred (game and not stopper('H'))) | convention-dependent |
| 1♣ 2NT | X | 17 / 1 | Penalty double -- 10+ HCP | (Inferred (penalty)) | convention-dependent |
| 1♠ 2NT | 3♥ | 15 / 0 | Natural, not forcing -- 2- !S; twice rebiddable !H; 7-9 tota | (Inferred (competitivevalues and twicerebiddable('H'))) | convention-dependent |
| 1♠ 2NT | X | 15 / 0 | Penalty double -- 10+ HCP | (Inferred (penalty)) | convention-dependent |
| 1♥ 2NT | X | 12 / 0 | Penalty double -- 10+ HCP | (Inferred (penalty)) | convention-dependent |
| 1♦ | 3NT | 3,345 / 0 | 5- !H; 5- !S; 25-32 HCP; likely stop in !D | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1♣ | 3NT | 2,898 / 0 | 5- !H; 5- !S; 25-32 HCP; likely stop in !C | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1♥ | 3♥ | 2,110 / 0 | 25+ HCP; forcing to 3N | (Inferred (game and not stopper('H'))) | nobody defines it |
| 1♥ | 3NT | 2,110 / 0 | 5- !H; 5- !S; 25-32 HCP; likely stop in !H | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1NT | 3NT | 2,039 / 0 | 5- !H; 5- !S; 25-32 HCP; partial stop in !C; partial stop in | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1♠ | 3♠ | 1,991 / 0 | 25+ HCP; forcing | (Inferred (game and not stopper('S'))) | nobody defines it |
| 1♠ | 3NT | 1,991 / 0 | 5- !H; 5- !S; 25-32 HCP; likely stop in !S | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 2♠ | 3NT | 609 / 23 | 2-5 !C; 2-5 !D; 2-5 !H; 2-5 !S; 19-24 HCP; stop in !S | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1NT P P | 3NT | 455 / 0 | 5- !H; 5- !S; 25-32 HCP; partial stop in !C; partial stop in | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 2♦ | 3NT | 445 / 8 | 2-5 !C; 2-5 !D; 2-5 !H; 2-5 !S; 19-24 HCP; stop in !D | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 2♥ | 3NT | 433 / 1 | 2-5 !C; 2-5 !D; 2-5 !H; 2-5 !S; 19-24 HCP; stop in !H | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1♦ 1♠ | 3♠ | 397 / 0 | 5+ !D; 13+ total points; forcing to 4D | (Inferred (game and not stopper('S'))) | nobody defines it |
| 1♣ 1♥ | 3♥ | 350 / 0 | 5+ !C; 13+ total points; forcing to 4C | (Inferred (game and not stopper('H'))) | nobody defines it |
| 1♣ 1♠ | 3♠ | 288 / 0 | 5+ !C; 13+ total points; forcing to 4C | (Inferred (game and not stopper('S'))) | nobody defines it |
| 1♦ P P | 3♦ | 274 / 0 | biddable !D | (Inferred (game and not stopper('D'))) | nobody defines it |
| 1♦ X | 3NT | 246 / 0 | 5- !H; 5- !S; 14-21 HCP; likely stop in !C; likely stop in ! | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1♣ 1♦ | 3♦ | 242 / 0 | 5+ !C; 13+ total points; forcing to 4C | (Inferred (game and not stopper('D'))) | nobody defines it |
| 1♦ 1♥ | 3♥ | 239 / 0 | 5+ !D; 13+ total points; forcing to 4D | (Inferred (game and not stopper('H'))) | nobody defines it |
| 2♣ | 2NT | 189 / 0 |   | (Inferred (BestPartscore('N'))) | nobody defines it |
| 2♣ | 3NT | 189 / 0 |   | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1♠ P P | 3♠ | 187 / 0 | 25+ HCP; forcing | (Inferred (game and not stopper('S'))) | nobody defines it |
| 1♣ X | 3♦ | 130 / 0 | biddable !D | (Inferred (BestPartscore('D'))) | nobody defines it |
| 1♣ X | 3NT | 130 / 0 | 5- !H; 5- !S; 14-21 HCP; likely stop in !D; likely stop in ! | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1♦ X P | 3♦ | 89 / 0 | 11+ total points | (Inferred (game and not stopper('D'))) | nobody defines it |
| 2♠ X | XX | 87 / 0 | 2- !S; 17+ HCP | (Inferred (hcp >= 10)) | nobody defines it |
| 1♣ P P | 3♣ | 85 / 0 | biddable !C | (Inferred (game and not stopper('C'))) | nobody defines it |
| 1♠ X P | 3♠ | 67 / 0 | 11+ total points | (Inferred (game and not stopper('S'))) | nobody defines it |
| 1♣ 1NT | 2NT | 64 / 0 | 5+ !C; 10+ total points | (Inferred (BestPartscore('N'))) | nobody defines it |
| 1♣ 1NT | 3♣ | 64 / 0 | 5+ !C; 6-9 total points | (Inferred (C_invite)) | nobody defines it |
| 1♣ 1NT | 3♥ | 64 / 0 | 7+ !H; 8- HCP | (Inferred (competitivevalues and twicerebiddable('H'))) | nobody defines it |
| 1♣ 1NT | 3♠ | 64 / 0 | 7+ !S; 8- HCP | (Inferred (competitivevalues and twicerebiddable('S'))) | nobody defines it |
| 1♣ 1NT | 3NT | 64 / 0 | 5- !H; 5- !S; 14-21 HCP; partial stop in !D; partial stop in | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1♦ 1NT | 2NT | 59 / 0 | 5+ !D; 10+ total points | (Inferred (BestPartscore('N'))) | nobody defines it |
| 1♦ 1NT | 3♦ | 59 / 0 | 5+ !D; 6-9 total points | (Inferred (D_invite)) | nobody defines it |
| 1♦ 1NT | 3♥ | 59 / 0 | 7+ !H; 8- HCP | (Inferred (competitivevalues and twicerebiddable('H'))) | nobody defines it |
| 1♦ 1NT | 3♠ | 59 / 0 | 7+ !S; 8- HCP | (Inferred (competitivevalues and twicerebiddable('S'))) | nobody defines it |
| 1♦ 1NT | 3NT | 59 / 0 | 5- !H; 5- !S; 14-21 HCP; partial stop in !C; partial stop in | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 2♦ X | 3♣ | 59 / 0 |   | (Inferred (BestPartscore('C'))) | nobody defines it |
| 1♥ P P | 3♥ | 57 / 0 | 25+ HCP; forcing to 3N | (Inferred (game and not stopper('H'))) | nobody defines it |
| 2♥ 2♠ | 3♠ | 57 / 0 | 2+ !H; 17+ HCP; 18+ total points; forcing | (Inferred (game and not stopper('S'))) | nobody defines it |
| 1♥ 1NT | 2NT | 52 / 0 | 3+ !H; 11+ total points | (Inferred (BestPartscore('N'))) | nobody defines it |
| 1♥ 1NT | 3NT | 52 / 0 | 5- !H; 5- !S; 14-21 HCP; partial stop in !C; partial stop in | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 1♦ 2♦ | 3♠ | 41 / 1 | 5+ !D; 13+ total points; forcing to 4D | (Inferred (game and not stopper('S'))) | nobody defines it |
| 1♠ 1NT | 2NT | 41 / 0 | 3+ !S; 11+ total points | (Inferred (BestPartscore('N'))) | nobody defines it |
| 1♠ 1NT | 3NT | 41 / 0 | 5- !H; 5- !S; 14-21 HCP; partial stop in !C; partial stop in | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 2♥ X | XX | 39 / 0 | 2- !H; 17+ HCP | (Inferred (hcp >= 10)) | nobody defines it |
| 1♥ 2♣ P | 3♦ | 38 / 0 | 4+ !D; 13+ HCP; 14+ total points; forcing to 3N | (Inferred (safety3level and twicerebiddable('D'))) | nobody defines it |
| 1♥ 2♣ P | 3♥ | 38 / 0 | 3+ !C; 14+ total points; forcing to 4C | (Inferred (game and not stopper('H'))) | nobody defines it |
| 1NT 2♥ P | 3♠ | 37 / 0 | 4+ !S; 13+ HCP; 14+ total points; forcing | (Inferred (twicerebiddable('S') and hcp >= 12)) | nobody defines it |
| 2♥ 2♠ P | 2NT | 30 / 0 | 11-13 HCP; stop in !H | (Inferred (BestPartscore('N') and stopper('H'))) | nobody defines it |
| 2♥ 2♠ P | 3♣ | 30 / 0 | 5+ !C; 2- !S; 12+ total points | (Inferred (advanceovercall('C'))) | nobody defines it |
| 2♥ 2♠ P | 3♦ | 30 / 0 | 5+ !D; 2- !S; 12+ total points | (Inferred (advanceovercall('D'))) | nobody defines it |
| 2♥ 2♠ P | 3♥ | 30 / 0 | 3+ !S; 12+ HCP; 13+ total points; forcing to 3N | (Inferred (game and not stopper('H'))) | nobody defines it |
| 1♣ 1♥ P | 3♠ | 29 / 0 | biddable !S | (Inferred (safety3level and twicerebiddable('S'))) | nobody defines it |
| 1♣ X P | 3♣ | 21 / 0 | 11+ total points | (Inferred (game and not stopper('C'))) | nobody defines it |
| 2♣ 2♥ | 3♠ | 21 / 0 |   | (Inferred (competitivevalues and twicerebiddable('S'))) | nobody defines it |
| 1♣ 2♣ | 3♠ | 20 / 0 | 5+ !C; 13+ total points; forcing to 4C | (Inferred (game and not stopper('S'))) | nobody defines it |
| 1NT 2♦ | 2♥ | 20 / 0 | 10+ HCP; forcing to 3N | (Inferred (game and not stopper('H'))) | nobody defines it |
| 1NT 2♦ | 2♠ | 20 / 0 | 10+ HCP; forcing to 3N | (Inferred (game and not stopper('S'))) | nobody defines it |
| 1♠ 2♥ P | 3♠ | 18 / 0 | 3+ !H; 1- !S; 13+ HCP; 14+ total points; forcing | (Inferred (game and not stopper('S'))) | nobody defines it |
| 2♠ 3♠ | 3NT | 12 / 0 | 5- !H; 5- !S; 20-27 HCP; likely stop in !H | (Inferred ((game or NT_trickgame) and not slam and stoppersO | nobody defines it |
| 2♠ 3♠ | X | 12 / 0 | 2+ !S; 7+ total points | (Inferred (suitshowing('S'))) | nobody defines it |

