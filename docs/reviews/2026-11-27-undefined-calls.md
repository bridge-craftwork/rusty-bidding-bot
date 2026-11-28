# Calls our engine cannot read, up to the fourth non-pass call (2026-11-27)

Rick (2026-11-27): most low-level auctions have a meaning for every bid; which ones does
Rusty have none for? Thorvald's example (1♠-1NT-2NT-3♥, Brill daily 2026-11-24 MP-strong
board 6) was the fourth non-pass call. Read-only probe: `probes/tools/undefined_calls.py`.

## Summary

**One Brill auction in five has a call we cannot read by the fourth non-pass call:**
6,335 of 31,227 field auctions (card Brill, rules `bottom-review`). The rate climbs with each
call:

| non-pass call | uncontested: first unread | contested: first unread |
|---|---|---|
| 1st (openings) | 15 of 31,084 (0.05%) | – |
| 2nd | 1.4% | 5.0% |
| 3rd | 2.6% | 9.5% |
| 4th | 13.7% | 23.5% |

- **Partner's calls:** 5.3% of North's calls (Brill robot partner) are the first call we can't
  read in their auction; 6.7% are unread at all. The opponents' rates are about the same
  (East 5.4%, West 6.5%).
- **21GF-GIB:** gives nearly the same numbers, so these are gaps in the rules, not in the
  Brill card.
- **The other corpora agree on the shape.** In the GIB corpus 9.9% of uncontested 4th calls
  are unread, and 19.7% of contested 3rd and 4th calls. In the BBA corpus the figures are
  18.8% and 19.4%.
- **Holes:** our engine has no rule at all for the South hand ("No rule applies") at 447 of
  53,909 South positions (0.8%). Of these, 192 follow calls that were all read: true holes in
  the rules.

**Families, by Brill-field count of the first unread call** (the tables below have the positions):

1. **Responder's second call, uncontested (668 new suits, 277 raises/rebids, 234 jump
   raises/rebids).** The biggest family. Examples:
   - 1♠-1NT-2♥-**4♥**;
   - 1NT-2♣-2♦-**3♣**, and 1NT-2♣-2♥-**2♠** (Brill's Stayman follow-ups);
   - 1♥-2♣-2♦-**2♠** and 1♠-2♥-3♣-**3♦** (fourth suit);
   - 1♦-1♠-2♦-**2♥** and 1♦-1♠-2♦-**4♠**;
   - 1♠-1NT-3♦-**4♦**;
   - 1M-1NT-2NT-**3x** (Thorvald's board, 22).

   We have rules for some of these sequences but not for every call in them.
2. **Direct-seat jump overcalls and preempts at the four level (436).** Examples: (1♦) **4♥**,
   (1♥) **4♣**, (1♣) **4♦**, (1♦) **4♠**, (1NT) P P **4♠**. GIB's tree has a meaning for every
   jump overcall up to 4♠ over any one-bid; we have none at the four level. Fourth-seat
   openings at 3♠ or 4♥ (P P P 3♠) are unread too: the openings' 15.
3. **Advancer's calls after (1x) overcall (X), and the mixed raise (226 cues, 164 NT, 119
   jump cues, 280 new suits).** Examples:
   - 1♦-1♠-X-**2♦** and 1♣-1♠-X-**2♣** (Brill's cue: 10+ with 3+ spades);
   - 1♣-1♠-X-**1NT**;
   - 1♣-1♥-1NT-**3♣** (the mixed raise);
   - P-1♣-X-P-**2♦** (the jump advance of a takeout double);
   - 1H-1NT-X-**2♥** (transfers after their double of our 1NT overcall).
4. **Opener's rebid in competition (285 new suits, 216 NT, 71 doubles).** Examples:
   - 1♦-P-1♠-X-**1NT**;
   - 1♦-1♥-1NT-**2♣** and 1♠-1NT-X-**2♣**;
   - 1♣-P-P-X-**1NT** (18-19) and 1♣-P-P-X-**1♠**;
   - 1♣-1♠-P-2♣-**X**.
5. **Their first action after our 1NT and a transfer or Stayman (200).** Examples:
   1NT-P-2♦-**2♥** (Brill: takeout of hearts), and over 1NT-2♣ the overcalls 2♦, 2♥, 2♠ and
   3x. These matter for defence (reading East/West), and for our systems-on when partner
   bids them.
6. **Responder in competition (164 new suits, 100 jump raises).** Examples:
   - P-P-1♣-X-**3♣** (Brill: good suit, 7-10);
   - 1NT-2♣-**3♣** (Brill: a minor transfer over their 2♣; systems on);
   - 1♥-2♣-**4♠**;
   - 1♦-P-**3♠**, a splinter over a minor that GIB's tree also defines.
7. **Opener's jump rebids, uncontested (137).** Examples: 1NT-2♣-**3♥**, and 1♠-2♣-**3♦**.

**Holes with every call read:**
- Opener's call after partner's 2NT over their overcall: P-P-1♦-2♠-2NT-P (22), P-1♠-2♥-2NT-P
  (19) and P-P-P-1♦-2♣-2NT-P (17). That is 2NT in competition, natural or Lebensohl, with no
  continuation rules.
- 2♥-4NT (21) and 2♠-3NT (22, after an unread 3NT).
- Responder to partner's 1NT with a weak hand (58). The pass there comes from no rule, though
  pass is the right call.

**Overlap with the work in flight** (Brill-field first unread, top 60 pairs: 1,451 calls):
- `second-calls-opener` (F1/F14): 180, family 4;
- `second-calls-defence` (F2/F3/F4): 65, plus 343 more in family 3 that the advancer-side
  work may cover. The mixed raise over their 1NT is already read there;
- `gf-after-2nt`: 22;
- not covered by any branch: 841, mainly families 1, 2, 5 and 6.

None of those branches is merged into `bottom-review` yet, so these counts are before them.

## Method

- **Auctions (real, not our self-play):** the Brill field PBNs, every table, 31,227
  auctions over 50 days (North = Rusty's partner, a Brill robot; East/West Brill robots; South
  the entrant); the BBA corpus of Practice-Bidding-Scenarios, 171,133 (bba-cli, both
  sides on 2/1 cards; scenario deals, so skewed to each scenario's auction); the GIB corpus
  there, 16,863. Plus every call GIB's tree defines at those positions where the
  tree is archived (no new gibrest requests).
- **Readings:** `rbb call --batch` once per distinct auction cut after its fourth non-pass
  call; `auction.position.unread` marks each call no rule reads (as explain-auction's
  "(no rule)"). Rules: the pack at `origin/bottom-review` (a3bffe8); engine v0.19.0 (main
  checkout binary). Card Brill (what the seat service plays); 21GF-GIB beside it.
- **First unread:** after one unread call the later ones are often unread too (the rules
  cannot follow). Tables count every unread call; the work queue counts only the first
  one in each auction.
- **Holes:** at every South position in the Brill field, the entrant's real hand asked of
  our engine: "No rule applies" is a pass that no rule makes.

## Calls not read, card Brill

| source | call | uncontested: calls | unread | first unread | contested: calls | unread | first unread |
|---|---|---|---|---|---|---|---|
| brill | 1 | 31,084 | 15 (0.0%) | 15 (0.0%) | 0 | 0 (–) | 0 (–) |
| brill | 2 | 17,291 | 235 (1.4%) | 235 (1.4%) | 11,955 | 596 (5.0%) | 596 (5.0%) |
| brill | 3 | 12,824 | 379 (3.0%) | 335 (2.6%) | 13,445 | 1,586 (11.8%) | 1,271 (9.5%) |
| brill | 4 | 9,623 | 1,458 (15.2%) | 1,322 (13.7%) | 10,913 | 3,249 (29.8%) | 2,561 (23.5%) |
| gib | 1 | 16,626 | 0 (0.0%) | 0 (0.0%) | 0 | 0 (–) | 0 (–) |
| gib | 2 | 9,937 | 583 (5.9%) | 583 (5.9%) | 6,031 | 46 (0.8%) | 46 (0.8%) |
| gib | 3 | 7,134 | 568 (8.0%) | 298 (4.2%) | 7,464 | 1,551 (20.8%) | 1,468 (19.7%) |
| gib | 4 | 4,736 | 877 (18.5%) | 471 (9.9%) | 6,077 | 2,090 (34.4%) | 1,198 (19.7%) |
| bba | 1 | 170,408 | 1 (0.0%) | 1 (0.0%) | 0 | 0 (–) | 0 (–) |
| bba | 2 | 104,366 | 1,132 (1.1%) | 1,132 (1.1%) | 61,513 | 2,504 (4.1%) | 2,504 (4.1%) |
| bba | 3 | 79,315 | 4,410 (5.6%) | 3,852 (4.9%) | 69,590 | 15,085 (21.7%) | 13,495 (19.4%) |
| bba | 4 | 61,805 | 13,439 (21.7%) | 11,637 (18.8%) | 58,479 | 18,903 (32.3%) | 11,358 (19.4%) |

Brill field by seat (North is Rusty's partner; East/West the opponents; South the entrants):

| seat | calls | unread | first unread |
|---|---|---|---|
| N | 29,290 | 1,977 (6.7%) | 1,560 (5.3%) |
| E | 21,289 | 1,335 (6.3%) | 1,146 (5.4%) |
| S | 35,864 | 2,724 (7.6%) | 2,292 (6.4%) |
| W | 20,692 | 1,482 (7.2%) | 1,337 (6.5%) |

## Calls not read, card 21GF-GIB

| source | call | uncontested: calls | unread | first unread | contested: calls | unread | first unread |
|---|---|---|---|---|---|---|---|
| brill | 1 | 31,084 | 15 (0.0%) | 15 (0.0%) | 0 | 0 (–) | 0 (–) |
| brill | 2 | 17,291 | 217 (1.3%) | 217 (1.3%) | 11,955 | 639 (5.3%) | 639 (5.3%) |
| brill | 3 | 12,824 | 596 (4.6%) | 553 (4.3%) | 13,445 | 1,562 (11.6%) | 1,236 (9.2%) |
| brill | 4 | 9,623 | 1,547 (16.1%) | 1,325 (13.8%) | 10,913 | 3,264 (29.9%) | 2,586 (23.7%) |
| gib | 1 | 16,626 | 0 (0.0%) | 0 (0.0%) | 0 | 0 (–) | 0 (–) |
| gib | 2 | 9,937 | 583 (5.9%) | 583 (5.9%) | 6,031 | 46 (0.8%) | 46 (0.8%) |
| gib | 3 | 7,134 | 823 (11.5%) | 553 (7.8%) | 7,464 | 1,534 (20.6%) | 1,451 (19.4%) |
| gib | 4 | 4,736 | 900 (19.0%) | 466 (9.8%) | 6,077 | 2,081 (34.2%) | 1,204 (19.8%) |
| bba | 1 | 170,408 | 1 (0.0%) | 1 (0.0%) | 0 | 0 (–) | 0 (–) |
| bba | 2 | 104,366 | 1,065 (1.0%) | 1,065 (1.0%) | 61,513 | 2,552 (4.1%) | 2,552 (4.1%) |
| bba | 3 | 79,315 | 8,542 (10.8%) | 7,993 (10.1%) | 69,590 | 14,894 (21.4%) | 13,298 (19.1%) |
| bba | 4 | 61,805 | 15,634 (25.3%) | 11,330 (18.3%) | 58,479 | 19,035 (32.6%) | 11,556 (19.8%) |

Brill field by seat (North is Rusty's partner; East/West the opponents; South the entrants):

| seat | calls | unread | first unread |
|---|---|---|---|
| N | 29,290 | 2,111 (7.2%) | 1,667 (5.7%) |
| E | 21,289 | 1,387 (6.5%) | 1,181 (5.5%) |
| S | 35,864 | 2,847 (7.9%) | 2,384 (6.6%) |
| W | 20,692 | 1,495 (7.2%) | 1,339 (6.5%) |

## Families (first unread call, card Brill)

Grouped by the caller's role, which non-pass call it is, the kind of call (relative to the
auction: new suit, raise or rebid of our suit, cue of theirs, NT, double; jump), and contested
or not. Ranked by Brill-field count, then GIB and BBA.

| role | call # | kind | contested | Brill | GIB | BBA |
|---|---|---|---|---|---|---|
| responder | 4 | new suit | no | 668 | 210 | 4331 |
| defender (first action) | 2 | jump new suit | yes | 436 | 15 | 1100 |
| opener | 4 | new suit | yes | 285 | 261 | 1703 |
| advancer | 4 | new suit | yes | 280 | 81 | 513 |
| responder | 4 | raise/rebid | no | 277 | 74 | 2251 |
| responder | 4 | jump raise/rebid | no | 234 | 76 | 2584 |
| advancer | 4 | cue | yes | 226 | 52 | 287 |
| opener | 4 | NT | yes | 216 | 46 | 504 |
| defender (first action) | 3 | new suit | yes | 200 | 340 | 1720 |
| responder | 3 | new suit | yes | 164 | 131 | 787 |
| advancer | 4 | NT | yes | 164 | 31 | 124 |
| overcaller | 4 | new suit | yes | 152 | 106 | 1008 |
| overcaller | 4 | raise/rebid | yes | 144 | 80 | 909 |
| opener | 3 | jump new suit | no | 137 | 26 | 1065 |
| defender (first action) | 4 | double | yes | 124 | 7 | 364 |
| advancer | 4 | jump cue | yes | 119 | 36 | 101 |
| overcaller | 4 | jump raise/rebid | yes | 102 | 15 | 259 |
| responder | 3 | jump raise/rebid | yes | 100 | 2 | 392 |
| opener | 3 | new suit | yes | 93 | 22 | 1228 |
| responder | 2 | jump new suit | no | 84 | 567 | 731 |
| responder | 2 | new suit | no | 83 | 9 | 271 |
| responder | 4 | jump new suit | no | 82 | 69 | 1026 |
| responder | 3 | jump new suit | yes | 81 | 152 | 693 |
| opener | 4 | double | yes | 71 | 160 | 810 |
| advancer | 3 | jump new suit | yes | 68 | 10 | 276 |
| defender (first action) | 2 | jump NT | yes | 62 | 1 | 124 |
| defender (first action) | 4 | new suit | yes | 56 | 35 | 118 |
| defender (first action) | 3 | jump NT | yes | 52 | 0 | 92 |
| defender (first action) | 2 | new suit | yes | 50 | 28 | 642 |
| opener | 3 | new suit | no | 48 | 95 | 820 |
| opener | 3 | NT | yes | 47 | 0 | 4 |
| defender (first action) | 3 | double | yes | 46 | 310 | 2781 |
| overcaller | 4 | cue | yes | 46 | 17 | 352 |
| opener | 4 | jump raise/rebid | yes | 46 | 3 | 317 |
| opener | 3 | NT | no | 45 | 0 | 1111 |
| opener | 3 | jump NT | no | 44 | 21 | 77 |
| advancer | 3 | new suit | yes | 43 | 34 | 292 |
| responder | 4 | NT | no | 42 | 16 | 899 |
| overcaller | 4 | NT | yes | 41 | 8 | 137 |
| responder | 4 | new suit | yes | 40 | 49 | 368 |

## Top unread (position, call) in the Brill field (first unread; card Brill)

| position (dealer first) | call | Brill (N/E/W/S) | GIB | BBA | role, call #, kind | Brill's note | GIB's meaning | in flight |
|---|---|---|---|---|---|---|---|---|
| 1D-P-1S-D | 1NT | 44 (16/0/14/14) | 0 | 5 | opener, 4, NT, contested |  | 2-5 !C; 3-5 !D; 2-4 !H; 2-3 !S; 12-14 HCP; stop in !C; stop in !H | second-calls-opener (F1/F14) |
| 1N-P-2D | 2H | 41 (17/20/1/3) | 3 | 47 | defender (first action), 3, new suit, contested | Takeout of hearts (the suit the transfer shows): spades and the minors [takeout('H') and S | Takeout double -- 3+ !C; 3+ !D; 2- !H; 3+ !S; 16+ total points |  |
| P-1D-1S-D | 2D | 36 (0/36/0/0) | 0 | 0 | advancer, 4, cue, contested | Artificial - 10+ hcp, 3+ S :: (HCP >= 9, S >= 4, SpadePoints >= 12) or (HCP >= 10, S >= 3) | Good support in S -- 3+ !S; 11- HCP; 10-12 total points | second-calls-defence (F2/F3/F4)? |
| P-1S-P-1N-P-2H-P | 4H | 33 (17/0/15/1) | 1 | 0 | responder, 4, jump raise/rebid | Nat :: (H >= 4, HeartPoints >= 12) or (H >= 5, HCP >= 10) | 4+ !H; 3- !S; 11-12 total points |  |
| 1N-P-2C-P | 3H | 32 (0/0/22/10) | 1 | 0 | opener, 3, jump new suit | natural | 2-5 !C; 2-5 !D; 5- !H; 2-5 !S; 17 HCP; rebiddable !H; 18 total points; forcing to 3N |  |
| 1D | 4H | 32 (24/0/0/8) | 0 | 73 | defender (first action), 2, jump new suit, contested | natural | Aggressive weak jump overcall -- 7+ !H; 11- HCP; 3+ total points |  |
| 1C-1S-D | 1NT | 31 (0/16/15/0) | 1 | 3 | advancer, 4, NT, contested | (HCP <= 11, HCP >= 9, Balanced) or (HCP <= 11, HCP >= 9, C >= 4) or (HCP <= 11, HCP >= 9,  | 2- !S; 9-11 HCP | second-calls-defence (F2/F3/F4)? |
| P-P-1N-P-2C-P-2H-P | 2S | 29 (27/0/0/2) | 0 | 0 | responder, 4, new suit | 4 Spades, NF :: S >= 4, NOT (CombinedHcpMin >= 25 OR IsGameForcing OR TwoOpenersGame) | 3- !H; 4 !S; 9 HCP |  |
| P-P-1H | 4C | 28 (0/0/28/0) | 0 | 0 | defender (first action), 2, jump new suit, contested | Preemptive [C >= 7 and hcp< 12 and loserlevel >= 3 and HasTopHonors('C', 2, 4) and S < 5 a | Aggressive weak jump overcall -- 7+ !C; 11- HCP; 3+ total points |  |
| P-1D-1H-1N-P | 2C | 28 (11/17/0/0) | 0 | 0 | opener, 4, new suit, contested | Board: 1 :: C >= 4, D >= 5 | New suit -- 4+ !C; 4+ !D; 11+ HCP; 3-card !D; 12-16 total points | second-calls-opener (F1/F14) |
| P-1C-D-P | 2D | 28 (28/0/0/0) | 0 | 0 | advancer, 3, jump new suit, contested | D >= 4, HCP >= 9, HCP <= 12 | Jump new suit -- 4+ !D; 11- HCP; 9-12 total points | second-calls-defence (F2/F3/F4)? |
| 1N-P-2C-P-2D-P | 3C | 27 (27/0/0/0) | 5 | 111 | responder, 4, raise/rebid | C >= 5, Game in C [C >= 5 and C_game] | 5+ !C; 13+ total points |  |
| P-1S-P-1N-P-3D-P | 4D | 27 (0/0/27/0) | 0 | 0 | responder, 4, raise/rebid | D >= 4 or (D = 3, HCP >= 10) | 4+ !D; 2- !S; 7+ HCP; 12- total points |  |
| 3D-P | 3S | 26 (0/15/0/11) | 0 | 15 | responder, 2, new suit | (TwiceRebiddable(S), HCP >= 12, BestMajor(S)) or (Rebiddable(S), BestMajor(S), HCP >= 15)  | twice rebiddable !S; 16+ total points; forcing |  |
| P-1C-1S-D | 2C | 26 (0/19/7/0) | 0 | 1 | advancer, 4, cue, contested | Artificial - 10+ hcp, 3+ S :: (HCP >= 9, S >= 4, SpadePoints >= 12) or (HCP >= 10, S >= 3) | Good support in S -- 3+ !S; 11- HCP; 10-12 total points | second-calls-defence (F2/F3/F4)? |
| P-P-1D | 4H | 26 (0/0/26/0) | 0 | 2 | defender (first action), 2, jump new suit, contested | Strong Preemptive Overcall, Preemptive with 8+ card suit, Preemptive :: (H >= 7, LoserLeve | Aggressive weak jump overcall -- 7+ !H; 11- HCP; 3+ total points |  |
| 2S-P-3S-D-P | 4H | 25 (25/0/0/0) | 1 | 8 | advancer, 4, new suit, contested | H >= 4 |  | second-calls-defence (F2/F3/F4)? |
| P-1H-P-2C-P-2D-P | 2S | 25 (0/0/24/1) | 0 | 0 | responder, 4, new suit | Natural fourth suit, 4+ cards :: S >= 4 | 5+ !C; 4+ !S; 13+ HCP; 14+ total points; forcing to 3N |  |
| P-1C-P-P-D | 1NT | 25 (0/25/0/0) | 0 | 0 | opener, 3, NT, contested | HCP >= 18, Balanced [hcp >= 18 and balanced] | 3+ !C; 18-19 HCP; 22- total points; stop in !D; stop in !H; stop in !S | second-calls-opener (F1/F14) |
| P-P-P-1H-1N-D | 2H | 25 (0/25/0/0) | 0 | 0 | advancer, 4, cue, contested | Jacoby, Transfer to Spades (5+ Spades) [S >= 5 and not opponentsuit('S')] | Jacoby transfer -- 5+ !S; 11- HCP; 12- total points | second-calls-defence (F2/F3/F4)? |
| P-P-1C | 4D | 24 (24/0/0/0) | 0 | 0 | defender (first action), 2, jump new suit, contested | Preemptive :: D >= 7, HCP < 12, LoserLevel >= 4, 2 of Top 4 Honors in D, H < 5, S < 5 | Aggressive weak jump overcall -- 7+ !D; 11- HCP; 3+ total points |  |
| 1C-1H-1N | 3C | 24 (24/0/0/0) | 0 | 1 | advancer, 4, jump cue, contested | Mixed raise -- 4+ !H; 7-9 total points; forcing to 3H [H >= 4 and totalpoints >= 7 and tot | Mixed raise -- 4+ !H; 7-9 total points; forcing to 3H | second-calls-defence (F2/F3/F4)? |
| 1D-P | 4S | 23 (1/22/0/0) | 0 | 0 | responder, 2, jump new suit | Preemptive [S >= 7 and hcp<= 10 and hcp>= 6] | Preemptive jump. Long suit. -- 6-10 HCP; strong rebiddable !S |  |
| 2H-2S-3H | 4C | 23 (12/0/11/0) | 0 | 0 | advancer, 4, new suit, contested | (BestMinor(C), C >= 5, HCP >= 12) or (BestMinor(C), C >= 6, ClubPoints >= 12) | 2- !S; 32- HCP; twice rebiddable !C; 17+ total points | second-calls-defence (F2/F3/F4)? |
| P-1C-1S-D | 1NT | 23 (4/19/0/0) | 0 | 1 | advancer, 4, NT, contested | (HCP <= 11, HCP >= 9, Balanced) or (HCP <= 11, HCP >= 9, C >= 4) or (HCP <= 11, HCP >= 9,  | 2- !S; 9-11 HCP; 12- total points | second-calls-defence (F2/F3/F4)? |
| 2S | 3NT | 23 (0/22/0/1) | 0 | 23 | defender (first action), 2, jump NT, contested | Board: 306126, Hand: AK.AJ965.AQJ6.AK [MS=?] :: (HCP >= 19, HCP <= 24, balish, Stopper in  | 2-5 !C; 2-5 !D; 2-5 !H; 2-5 !S; 19-24 HCP; stop in !S |  |
| P-1N-D-P-2D-P | 2H | 23 (0/20/3/0) | 0 | 0 | overcaller, 4, new suit, contested | Board: 146701, Hand: AT72.AKQT95..KQ3 :: Biddable(H), NOT (CombinedHcpMin >= 25 OR IsGameF | 4+ !H; 19+ total points | second-calls-defence (F2/F3/F4) |
| P-3S-P | 4H | 23 (0/0/23/0) | 0 | 0 | responder, 2, new suit | Board: 134768, Hand: KQJ8742.2.AQJ7.9, Board: 759309, Hand: AQJ8643..A7632.Q, Board: 88178 | 17+ HCP; twice rebiddable !H; 19+ total points |  |
| 1H | 4C | 23 (0/0/23/0) | 0 | 11 | defender (first action), 2, jump new suit, contested | Preemptive [C >= 7 and hcp< 12 and loserlevel >= 3 and HasTopHonors('C', 2, 4) and S < 5 a | Aggressive weak jump overcall -- 7+ !C; 11- HCP; 3+ total points |  |
| P-1D-P-1S-P-2D-P | 2H | 23 (23/0/0/0) | 0 | 0 | responder, 4, new suit | Nat :: H >= 4, HCP >= 10, S >= 5 | New suit -- 3+ !H; 4+ !S; 12+ total points; forcing to 3N |  |
| P-1N-2C | 3C | 22 (22/0/0/0) | 0 | 0 | responder, 3, cue, contested | MinorTransfer [D >= 6 and (hcp >= 13 or (hcp >= 4 and hcp<= 7) or shouldinvestigateslam('D | Minor transfer -- 6+ !D |  |
| P-1C-P-1D-P-2C-P | 2S | 22 (0/0/22/0) | 0 | 0 | responder, 4, new suit | Nat [(S >= 4 and hcp>= 12 and D >= 5) / (S >= 4 and hcp>= 18)] | Responder reverse -- 5+ !D; biddable !S; 13+ total points; forcing to 3N |  |
| P-1H-P-1N-P-2N-P | 3D | 22 (0/0/22/0) | 0 | 0 | responder, 4, new suit | [H == 3 and hcp >= 10 and hcp <= 12] | 3 !H; 3- !S; 10-12 total points | gf-after-2nt |
| P-P-1C-D | 3C | 22 (0/0/22/0) | 0 | 0 | responder, 3, jump raise/rebid, contested | Good suit, 7-10 hcp [hcp >= 7 and hcp<= 10 and C >= 5] | 5+ !C; 11- HCP; 8-12 total points |  |
| 1H-2C | 4S | 22 (0/0/22/0) | 0 | 2 | responder, 3, jump new suit, contested | [S >= 7 and losers <= 7 and H <= 3] | 12+ HCP; strong rebiddable !S; 13 total points |  |
| P-1H-1S-P-2H-P | 3S | 22 (0/0/22/0) | 0 | 0 | overcaller, 4, jump raise/rebid, contested | SpadePoints >= 13, NOT Game in S |  | second-calls-defence (F2/F3/F4) |
| 1S-1N-D-P | 2C | 22 (0/22/0/0) | 0 | 1 | opener, 4, new suit, contested | C >= 4, HCP <= 13 | 4+ !C; 5+ !S; 11-21 HCP; 12-22 total points | second-calls-opener (F1/F14) |
| P-P-P-1H-P-1N-P-2D-P-P | 2S | 22 (0/22/0/0) | 0 | 0 | defender (first action), 4, new suit, contested | 4+ hearts; 10-16 total points [(overcall('S')) / (totalpoints >= 10 and totalpoints <= 16  |  |  |
| 2C | 2H | 21 (0/0/21/0) | 1 | 95 | defender (first action), 2, new suit, contested | H >= 6, LoserLevel >= 2 [H >= 6 and loserlevel >= 2] | twice rebiddable !H; 9+ total points |  |
| 1S-P-P | 3C | 21 (21/0/0/0) | 1 | 3 | defender (first action), 2, jump new suit, contested | C >= 6, HCP >= 16, HCP <= 19 [C >= 6 and hcp>= 16 and hcp<= 19] | Balancing jump overcall -- strong rebiddable !C; 13+ total points |  |
| 1H-P-1S-P-1N-P | 3D | 21 (14/0/0/7) | 0 | 2 | responder, 4, jump new suit | Brill: BAR bid :: HCP <= 8, D >= 6 | Bar bid; long 0 -- 6+ !D; 4+ !S; 6 total points |  |
| P-P-2N-P-3C-P-3H-P | 4C | 21 (0/0/21/0) | 0 | 0 | responder, 4, raise/rebid | C >= 5, NOT balish, HCP >= 10 | 5+ !C; 3- !H; 11- HCP; 11-12 total points |  |
| P-1D | 4S | 21 (0/21/0/0) | 0 | 1 | defender (first action), 2, jump new suit, contested | Strong Preemptive Overcall, Preemptive with 8+ card suit, Preemptive :: (S >= 7, LoserLeve | Aggressive weak jump overcall -- 7+ !S; 11- HCP; 3+ total points |  |
| P-P-P-1D-P-P-D | 1NT | 21 (0/21/0/0) | 0 | 0 | opener, 3, NT, contested | HCP >= 18, Balanced | 3+ !D; 18-19 HCP; 22- total points; stop in !C; stop in !H; stop in !S | second-calls-opener (F1/F14) |
| P-P-P-1D-3C | 4D | 21 (21/0/0/0) | 0 | 0 | responder, 3, jump raise/rebid, contested | D >= 4, HCP >= 10, HCP <= 13 |  |  |
| P-P-1D-1H-1S | 3D | 21 (21/0/0/0) | 0 | 0 | advancer, 4, jump cue, contested | Mixed raise -- 4+ !H; 7-9 total points; forcing to 3H [H >= 4 and totalpoints >= 7 and tot | Mixed raise -- 4+ !H; 7-9 total points; forcing | second-calls-defence (F2/F3/F4)? |
| 1D-P-P-2C-P | 2D | 21 (0/0/21/0) | 0 | 1 | advancer, 3, cue, contested | Artificial - C >= 2, HCP >= 10 | 3+ !C; 11-16 total points | second-calls-defence (F2/F3/F4)? |
| 1C-P | 3S | 20 (0/17/0/3) | 113 | 70 | responder, 2, jump new suit | Splinter — max 1 !S and 14+ total pts and 5+ !C and max 3 !H | Splinter -- 5+ !C; 3- !H; 1- !S; 14+ total points |  |
| 1D-1S-D | 3D | 20 (0/20/0/0) | 4 | 2 | advancer, 4, jump cue, contested | Mixed raise -- 4+ !S; 7-9 total points; forcing to 3S [S >= 4 and totalpoints >= 7 and tot | Mixed raise -- 4+ !S; 7-9 total points; forcing to 3S | second-calls-defence (F2/F3/F4)? |
| 1C-1S-P-2C | X | 20 (0/0/20/0) | 3 | 6 | opener, 4, double, contested | IsGoodSuit(C), C >= 6 | 11-21 HCP; rebiddable !C; !CKQ; 12-22 total points | second-calls-opener (F1/F14) |
| 1D-1S-2D | 3D | 20 (0/20/0/0) | 2 | 0 | advancer, 4, cue, contested | Limit+ [hcp >= 9 and S >= 3] | 3+ !S; 11+ total points | second-calls-defence (F2/F3/F4)? |
| P-P-P-1N-P-2H-P-3D-P | 3H | 20 (1/19/0/0) | 0 | 0 | responder, 4, raise/rebid | Retransfer |  |  |
| 1S-P-2H-P-3C-P | 3D | 20 (20/0/0/0) | 0 | 9 | responder, 4, new suit | Natural fourth suit, 4+ cards [D >= 4] | 4+ !D; 5+ !H; 12+ HCP; 13+ total points; forcing to 3N |  |
| 1D-P-P-D-P-2H-P | 4H | 20 (0/20/0/0) | 0 | 0 | overcaller, 4, jump raise/rebid, contested | Game in H (competitive), H >= 4 [H_compgame and H >= 4] | 3+ !C; 4+ !H; 3+ !S; 16-19 total points | second-calls-defence (F2/F3/F4) |
| 1N-P-P | 4S | 20 (0/0/20/0) | 0 | 0 | defender (first action), 2, jump new suit, contested | (Rebiddable(S), S >= 7, HCP <= 10, LoserLevel >= 4) or (Rebiddable(S), S >= 7, Losers <= 3 | 24+ HCP; strong rebiddable !S; 25 total points |  |
| P-2S | 4S | 20 (20/0/0/0) | 0 | 0 | defender (first action), 2, jump cue, contested | 5-5 in minors [diamonds >= 5 and clubs >= 5 and losers <= 4 and hcp>= 17] | Minors -- 5+ !C; 5+ !D; 20+ total points; forcing to 5C |  |
| P-1S | 4C | 20 (0/0/20/0) | 0 | 1 | defender (first action), 2, jump new suit, contested | Preemptive :: C >= 7, HCP < 12, LoserLevel >= 3, 2 of Top 4 Honors in C, H < 5, HCP >= 5 | Aggressive weak jump overcall -- 7+ !C; 11- HCP; 3+ total points |  |
| P-P-1D-1S-D | 2C | 20 (0/20/0/0) | 0 | 0 | advancer, 4, new suit, contested | 10+ hcp, 5+ C, Board: 67213, Hand: 5.T98.8.AKJT7642 :: (HCP >= 10, C >= 5) or AdvanceOverc | 5+ !C; 2- !S; 11- HCP; 9-12 total points | second-calls-defence (F2/F3/F4)? |
| P-1D-P-1S-P-2D-P | 4S | 20 (0/20/0/0) | 0 | 0 | responder, 4, jump raise/rebid | To play [S >= 6 and hcp> 11 and hcp<= 17] | 12+ HCP; strong rebiddable !S; 13-16 total points |  |
| 1C-P-P-D | 1S | 20 (0/0/20/0) | 0 | 40 | opener, 3, new suit, contested | S >= 4, C >= 5 | 5+ !C; 4+ !S; 21- HCP; 15-22 total points | second-calls-opener (F1/F14) |

## Top unread (position, call) in the GIB and BBA corpora, not in the Brill field

| position (dealer first) | call | Brill (N/E/W/S) | GIB | BBA | role, call #, kind | Brill's note | GIB's meaning | in flight |
|---|---|---|---|---|---|---|---|---|
| 1N-D-R-P | 2C | 0 (0/0/0/0) | 161 | 0 | opener, 4, new suit, contested |  |  | second-calls-opener (F1/F14) |
| 1N-P-2C | 2D | 0 (0/0/0/0) | 91 | 96 | defender (first action), 3, new suit, contested |  | Overcall -- twice rebiddable !D; 11+ total points |  |
| 1N-P-2C | 3D | 0 (0/0/0/0) | 89 | 89 | defender (first action), 3, jump new suit, contested |  | Overcall -- 7+ !D; 11+ total points |  |
| 1N-P-2C | 2S | 0 (0/0/0/0) | 81 | 49 | defender (first action), 3, new suit, contested |  | Overcall -- twice rebiddable !S; 11+ total points |  |
| 1N-P-2C | 2H | 0 (0/0/0/0) | 78 | 42 | defender (first action), 3, new suit, contested |  | Overcall -- twice rebiddable !H; 11+ total points |  |
| 1N-2D | 2NT | 0 (0/0/0/0) | 70 | 52 | responder, 3, NT, contested |  | Lebensohl - Forces 3C by partner --   |  |
| 1N-P-2C | 3C | 0 (0/0/0/0) | 37 | 185 | defender (first action), 3, cue, contested |  | Overcall -- 7+ !C; 11+ total points |  |
| 1N-P-2C | 4D | 0 (0/0/0/0) | 46 | 33 | defender (first action), 3, jump new suit, contested |  | Preempt -- 7+ !D; 3-20 total points |  |
| 2C | X | 0 (0/0/0/0) | 0 | 262 | defender (first action), 2, double, contested |  | 6+ HCP; rebiddable !C; !CKQ |  |
| 1N-P-3C-P-3D-P | 3S | 0 (0/0/0/0) | 4 | 227 | responder, 4, new suit |  | Splinter -- 6+ !D; 1- !S; 9+ HCP; forcing |  |
| 1H-2H | X | 0 (0/0/0/0) | 15 | 161 | responder, 3, double, contested |  | Penalty double -- 10+ HCP |  |
| 1N-P-3C-P-3D-P | 3H | 0 (0/0/0/0) | 2 | 213 | responder, 4, new suit |  | Splinter -- 6+ !D; 1- !H; 9+ HCP; forcing to 3N |  |
| 1H-P-1S-P-2D-P | 3C | 0 (0/0/0/0) | 29 | 74 | responder, 4, new suit |  | Fourth suit forcing -- 4+ !S; 13+ total points |  |
| 1S-2S | X | 0 (0/0/0/0) | 12 | 150 | responder, 3, double, contested |  | Penalty double -- 10+ HCP |  |
| 1C-2C | X | 0 (0/0/0/0) | 9 | 165 | responder, 3, double, contested |  | Penalty double -- 10+ HCP |  |
| 1N-P-2H | X | 0 (0/0/0/0) | 0 | 197 | defender (first action), 3, double, contested |  | 6+ HCP; rebiddable !H; !HKQ |  |
| 1N-P-2C-P-2H-P | 3S | 0 (0/0/0/0) | 11 | 141 | responder, 4, jump new suit |  | forcing H raise -- 2+ !C; 2+ !D; 4+ !H; 15+ total points |  |
| 1N-P-2C | 3S | 0 (0/0/0/0) | 31 | 31 | defender (first action), 3, jump new suit, contested |  | Overcall -- 7+ !S; 11+ total points |  |
| 1N-P-2N-P | 3NT | 0 (0/0/0/0) | 0 | 184 | opener, 3, NT |  | 2-5 !C; 2-5 !D; 2-5 !H; 2-5 !S; 17- HCP; 18 total points |  |
| 2S-D | XX | 0 (0/0/0/0) | 0 | 176 | responder, 3, redouble, contested |  | 2- !S; 17+ HCP |  |
| 2N-P-3C-P | 3NT | 0 (0/0/0/0) | 0 | 176 | opener, 3, NT |  | 2-5 !C; 2-5 !D; 2-5 !H; 2-5 !S; 20-21 HCP |  |
| 1D-P-2C-P-2D-P | 2H | 0 (0/0/0/0) | 0 | 175 | responder, 4, new suit |  | 5+ !C; 4+ !H; 14+ total points; forcing to 3N |  |
| P-1D-P-1S-1N | X | 0 (0/0/0/0) | 34 | 1 | opener, 4, double, contested |  | Support double - 3 S -- 4+ !D; 3 !S; 11-21 HCP; 12-22 total points | second-calls-opener (F1/F14) |
| 1S-P-2N-P-3S-P | 3NT | 0 (0/0/0/0) | 0 | 170 | responder, 4, NT |  | 4- !H; 4-5 !S; 14- HCP; 13+ total points |  |
| 1N-P-2C | 3H | 0 (0/0/0/0) | 27 | 33 | defender (first action), 3, jump new suit, contested |  | Overcall -- 7+ !H; 11+ total points |  |
| 1N-2D | 3S | 0 (0/0/0/0) | 33 | 1 | responder, 3, jump new suit, contested |  | 10+ HCP; stop in !S; forcing |  |
| 1S-P-2C-P | 3D | 0 (0/0/0/0) | 0 | 161 | opener, 3, jump new suit |  | 5+ !D; 5+ !S; 15-21 HCP; !DKQ; !SKQ; 22- total points; forcing to 3N |  |
| 1D-2N | X | 0 (0/0/0/0) | 2 | 148 | responder, 3, double, contested |  | Penalty double -- 10+ HCP |  |
| 1C-1N-P-2C | X | 0 (0/0/0/0) | 4 | 136 | opener, 4, double, contested |  | 11-21 HCP; rebiddable !C; 12-22 total points | second-calls-opener (F1/F14) |
| 1D-P-1S-P-2D-P | 2H | 0 (0/0/0/0) | 23 | 29 | responder, 4, new suit |  | New suit -- 3+ !H; 4+ !S; 12+ total points; forcing to 3N |  |
| 1N-P-2C-P-2H-P | 4D | 0 (0/0/0/0) | 3 | 125 | responder, 4, jump new suit |  | Splinter -- 1- !D; 4+ !H; 15+ total points |  |
| 1N-D | 2S | 0 (0/0/0/0) | 0 | 134 | responder, 3, new suit, contested |  | Minor Stayman --   |  |
| 1N-P-2C-P-2H-P | 4C | 0 (0/0/0/0) | 4 | 112 | responder, 4, jump raise/rebid |  | Splinter -- 1- !C; 4+ !H; 15+ total points |  |
| 1C-P-1H-P-2C-P | 2D | 0 (0/0/0/0) | 16 | 48 | responder, 4, new suit |  | New suit -- 3+ !D; 4+ !H; 12+ total points; forcing to 3N |  |
| 1D-P-2C-P-2D-P | 3D | 0 (0/0/0/0) | 0 | 128 | responder, 4, raise/rebid |  | 3+ !D; biddable !C; 14+ total points; forcing to 3N |  |
| 1D-P-2D-P-2H-P | 4D | 0 (0/0/0/0) | 0 | 128 | responder, 4, jump raise/rebid |  | 5+ !D; 3- !H; 3- !S; 12+ HCP; 13+ total points |  |
| 1N-P-2C-P-2D-P | 3D | 0 (0/0/0/0) | 10 | 76 | responder, 4, raise/rebid |  | 5+ !D; 13+ total points |  |
| 1N-P-2C-P-2H-P | 2S | 0 (0/0/0/0) | 25 | 0 | responder, 4, new suit |  | 3- !H; 4 !S; 9 HCP |  |
| 1H-P-4D | X | 0 (0/0/0/0) | 17 | 39 | defender (first action), 3, double, contested |  | 6+ HCP; rebiddable !D; !DKQ |  |
| 1N-2H | 4S | 0 (0/0/0/0) | 22 | 14 | responder, 3, jump new suit, contested |  | 6+ !S; 9+ HCP; 10-14 total points |  |

## Calls GIB's tree defines that we cannot read (card Brill)

At real positions (fewer than four non-pass calls so far, every call before read), each
call up to the four level (and doubles) that GIB's tree gives a meaning and no rule of
ours reads, weighted by how often the
position occurs. Many are calls nobody makes; the frequent positions matter.

| position | call | position seen: Brill | GIB | BBA | GIB's meaning |
|---|---|---|---|---|---|
| 1D | 4S | 3345 | 2827 | 25810 | Aggressive weak jump overcall -- 7+ !S; 11- HCP; 3+ total points |
| 1D | 4H | 3345 | 2827 | 25810 | Aggressive weak jump overcall -- 7+ !H; 11- HCP; 3+ total points |
| 1D | 4D | 3345 | 2827 | 25810 | Preempt -- 7+ !D; 11- HCP; 3+ total points |
| 1D | 4C | 3345 | 2827 | 25810 | Aggressive weak jump overcall -- 7+ !C; 11- HCP; 3+ total points |
| 1D | 3NT | 3345 | 2827 | 25810 | 5- !H; 5- !S; 25-32 HCP; likely stop in !D |
| 1D | 3D | 3345 | 2827 | 25810 | Aggressive weak jump overcall -- 7+ !D; 10- HCP; 3+ total points |
| 1C | 4S | 2898 | 3515 | 32105 | Aggressive weak jump overcall -- 7+ !S; 11- HCP; 3+ total points |
| 1C | 4H | 2898 | 3515 | 32105 | Aggressive weak jump overcall -- 7+ !H; 11- HCP; 3+ total points |
| 1C | 4D | 2898 | 3515 | 32105 | Aggressive weak jump overcall -- 7+ !D; 11- HCP; 3+ total points |
| 1C | 4C | 2898 | 3515 | 32105 | Preempt -- 7+ !C; 11- HCP; 3+ total points |
| 1C | 3NT | 2898 | 3515 | 32105 | 5- !H; 5- !S; 25-32 HCP; likely stop in !C |
| 1C | 3C | 2898 | 3515 | 32105 | Aggressive weak jump overcall -- 7+ !C; 10- HCP; 3+ total points |
| P-P-P | 4S | 2779 | 539 | 3031 | Preempt -- 8+ !S; 6-9 HCP |
| P-P-P | 4H | 2779 | 539 | 3031 | Preempt -- 8+ !H; 6-9 HCP |
| P-P-P | 4D | 2779 | 539 | 3031 | Preempt -- 8+ !D; 6-9 HCP |
| P-P-P | 4C | 2779 | 539 | 3031 | Preempt -- 8+ !C; 6-9 HCP |
| P-P-P | 3S | 2779 | 539 | 3031 | 4th seat preempt -- 7+ !S; 9-12 HCP; !SQ |
| P-P-P | 3H | 2779 | 539 | 3031 | 4th seat preempt -- 7+ !H; 9-12 HCP; !HQ |
| P-P-P | 3D | 2779 | 539 | 3031 | 4th seat preempt -- 7+ !D; 9-12 HCP; !DQ |
| P-P-P | 3C | 2779 | 539 | 3031 | 4th seat preempt -- 7+ !C; 9-12 HCP; !CQ |
| P-1C | 4S | 2144 | 164 | 627 | Aggressive weak jump overcall -- 7+ !S; 11- HCP; 3+ total points |
| P-1C | 4H | 2144 | 164 | 627 | Aggressive weak jump overcall -- 7+ !H; 11- HCP; 3+ total points |
| P-1C | 4D | 2144 | 164 | 627 | Aggressive weak jump overcall -- 7+ !D; 11- HCP; 3+ total points |
| P-1C | 4C | 2144 | 164 | 627 | Preempt -- 7+ !C; 3-20 total points |
| P-1C | 3NT | 2144 | 164 | 627 | 5- !H; 5- !S; 25-32 HCP; likely stop in !C |
| P-1C | 3C | 2144 | 164 | 627 | Aggressive weak jump overcall -- 7+ !C; 10- HCP; 3+ total points |
| 1D-P | 4S | 2133 | 1770 | 14998 | Preemptive jump. Long suit. -- 6-10 HCP; strong rebiddable !S |
| 1D-P | 4H | 2133 | 1770 | 14998 | Preemptive jump. Long suit. -- 6-10 HCP; strong rebiddable !H |
| 1D-P | 4D | 2133 | 1770 | 14998 | biddable !D |
| 1D-P | 4C | 2133 | 1770 | 14998 | Splinter -- 1- !C; 5+ !D; 3- !H; 3- !S; 14+ total points |
| 1D-P | 3S | 2133 | 1770 | 14998 | Splinter -- 5+ !D; 3- !H; 1- !S; 14+ total points |
| 1D-P | 3H | 2133 | 1770 | 14998 | Splinter -- 5+ !D; 1- !H; 3- !S; 14+ total points |
| 1H | 4S | 2110 | 1287 | 22202 | Aggressive weak jump overcall -- 7+ !S; 11- HCP; 3+ total points |
| 1H | 4H | 2110 | 1287 | 22202 | 27+ HCP; forcing to 4N |
| 1H | 4D | 2110 | 1287 | 22202 | Aggressive weak jump overcall -- 7+ !D; 11- HCP; 3+ total points |
| 1H | 4C | 2110 | 1287 | 22202 | Aggressive weak jump overcall -- 7+ !C; 11- HCP; 3+ total points |
| 1H | 3NT | 2110 | 1287 | 22202 | 5- !H; 5- !S; 25-32 HCP; likely stop in !H |
| 1H | 3H | 2110 | 1287 | 22202 | 25+ HCP; forcing to 3N |
| 1N | 4NT | 2039 | 5854 | 32905 |   |
| 1N | 4S | 2039 | 5854 | 32905 | Preempt -- 7+ !S; 11- HCP; 3+ total points |
| 1N | 4H | 2039 | 5854 | 32905 | Preempt -- 7+ !H; 11- HCP; 3+ total points |
| 1N | 4D | 2039 | 5854 | 32905 | Preempt -- 7+ !D; 11- HCP; 3+ total points |
| 1N | 4C | 2039 | 5854 | 32905 | Preempt -- 7+ !C; 11- HCP; 3+ total points |
| 1N | 3NT | 2039 | 5854 | 32905 | 5- !H; 5- !S; 25-32 HCP; partial stop in !C; partial stop in !D; partial stop in !H; partial stop in |
| 1S | 4S | 1991 | 1331 | 23443 | 27+ HCP; forcing |
| 1S | 4H | 1991 | 1331 | 23443 | Aggressive weak jump overcall -- 7+ !H; 11- HCP; 3+ total points |
| 1S | 4D | 1991 | 1331 | 23443 | Aggressive weak jump overcall -- 7+ !D; 11- HCP; 3+ total points |
| 1S | 4C | 1991 | 1331 | 23443 | Aggressive weak jump overcall -- 7+ !C; 11- HCP; 3+ total points |
| 1S | 3NT | 1991 | 1331 | 23443 | 5- !H; 5- !S; 25-32 HCP; likely stop in !S |
| 1S | 3S | 1991 | 1331 | 23443 | 25+ HCP; forcing |

In all, 42,578 (position, call) pairs GIB defines are unread by us; 25492 of them at positions seen in the Brill field.

## Holes: South positions where no rule applies (Brill field, card Brill)

53,909 South positions (entrants' real hands, up to the fourth non-pass call). "No rule applies": 447 (0.8%); 192 with every call before read (a hole in our rules for that hand), 255 after a call we could not read.

| position | times | after an unread call | what the entrants bid there |
|---|---|---|---|
| P-P-P-1N-P | 40 | 0 | Pass 22, 2NT 6, 2C 4, 2S 3 |
| P-P-1D-4H-D-P | 26 | 26 | Pass 24, 4S 2 |
| P-1N-2C-3C-P | 22 | 22 | 3D 13, 3NT 4, Pass 3, 4C 2 |
| 2S-3N | 22 | 22 | Pass 14, X 8 |
| P-P-1D-2S-2N-P | 22 | 0 | 3NT 9, 3H 5, Pass 5, 5D 1 |
| 2H-4N | 21 | 0 | Pass 20, X 1 |
| P-1D-4S | 21 | 21 | 5D 10, X 6, Pass 3, 4NT 2 |
| 1N-P-2D-2H | 20 | 20 | Pass 18, X 1, 4D 1 |
| P-1S-2H-2N-P | 19 | 0 | 3D 8, 3NT 2, 3H 2, 4D 2 |
| P-1D-5C | 18 | 18 | Pass 15, 5H 3 |
| 1N-P | 18 | 0 | 2S 7, 2NT 5, 3C 2, Pass 2 |
| 1D-4C | 17 | 17 | X 12, Pass 3, 4D 2 |
| P-1N-P-2C-D | 17 | 17 | 2S 14, Pass 3 |
| 1N-P-4D-4S | 17 | 17 | X 13, Pass 4 |
| P-P-P-1D-2C-2N-P | 17 | 0 | 3NT 10, 3H 6, Pass 1 |
| 2N-P-3C-3S | 15 | 15 | Pass 6, 3NT 3, 4H 2, X 2 |
| P-1N-P | 14 | 0 | Pass 5, 2C 5, 2NT 4 |
| P-P-1C-4S-D-P | 14 | 14 | Pass 14 |
| P-2D-P-2S-P-3S-P | 13 | 0 | Pass 7, 4S 6 |
| 2H-P-3D-P-3H-P | 5 | 5 | 3S 4, Pass 1 |
| P-2D-P-3C-P-3D-P | 5 | 5 | Pass 2, 3NT 2, 4D 1 |
| 2D-2S-2N-P | 5 | 5 | 3C 3, 3NT 1, Pass 1 |
| 1C-P-1N-P-2C-P | 4 | 0 | 2H 2, 2NT 1, 3S 1 |
| P-2D-2H-2N-P | 4 | 4 | 3D 3, Pass 1 |
| P-1H-P-1N-P-2C-P | 3 | 0 | 4H 1, 2H 1, 2D 1 |
| 1C-P-3D-P-4C-P | 3 | 3 | 4D 1, 4H 1, 5C 1 |
| P-1D-P-3C-P-3D-P | 3 | 3 | 4C 2, 3NT 1 |
| P-P-P-1N-P-2D-2H | 3 | 3 | Pass 2, 3H 1 |
| 1H-P-1N-P-2H-P | 2 | 0 | 3H 1, 3D 1 |
| P-P-1C-P-1S-P | 2 | 0 | 2NT 1, 2H 1 |
| 2D-P-3D-D | 2 | 0 | Pass 2 |
| 1N-P-2H-P-2S-P | 2 | 0 | 2NT 2 |
| P-3C-P-3D-P-4C-P | 2 | 2 | 5C 1, Pass 1 |
| 1N-P-2D-2S | 2 | 2 | X 1, 3H 1 |
| 2H-P-2S-D | 2 | 2 | 3S 2 |
| P-1N-P-2D-2H | 2 | 2 | X 2 |
| P-P-2D-P-4D-P | 2 | 2 | Pass 1, 4NT 1 |
| P-1S-P-1N-P-2S-P | 1 | 0 | 2NT 1 |
| P-P-P-1S-P | 1 | 0 | 2C 1 |
| 1D-P-1N-P-2D-P | 1 | 0 | 2NT 1 |

