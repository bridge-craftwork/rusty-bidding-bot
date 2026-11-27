# Brill "bidding" boards against the lower-level work (2026-11-27)

All 400 boards that Thorvald's daily mails (2026-10-08 .. 2026-11-26, 4 events a day) flag
"bidding" (including "bidding, and declarer play/defence"), checked against today's engine.
Rules and engine untouched; everything below is read-only analysis.

## Method

- **Data.** Mail flags parsed per event; each board's tables come from the field PBN
  (`brill_report.parse_pbn`/`tables_of`), every table's contract scored at double dummy
  (`rbb dd`, cached). N/E/W are Brill robots, so all tables share the auction up to South's
  first differing call.
- **The costly call.** For each Rusty (South) call, the field tables with the same prefix and
  what they called there, each call's mean DD score. The key decision is the first South
  call where a different call (by at least 10% of those tables) scored better on DD than our
  table and better than the field tables that made our call. Read by hand afterwards; on 14
  boards the real culprit was a later call and I moved the key there (e.g. 11-20 MP-strong b5:
  the takeout double was fine, the 2H rebid with eight hearts was not).
- **Today's engine.** `rbb call --batch` (v0.19.0, built 2026-11-26) at every South decision
  (977 positions), card `cards/json/Brill.json`. That is the card the seat service plays
  (`RBB_CARD` defaults to `Brill` in rusty-seat-service `src/config.rs`; not 21GF-GIB). Two
  rule sets: compiled-in, and the scratch `pack-bottom` (bottom-review plus balancing/fourth
  seat). The seat service does **not** read `meanings` (`query.rs`: "accepted and not
  read"). I asked again with the opponents' Brill meanings passed for every (b)/(c)/known
  position: no call changes. The one exception is the known board, where `pack-bottom`
  already doubles.
- **GIB.** Each position in GIB notation, dealer-relative (as the prefixes file has it),
  looked up in `prefixes-random100k-top20000.json` and counted in all 100k self-play
  auctions (`auctions/r100k-freak-...txt`). The GIB tree at all 100 positions
  (59 archived, 41 fetched now, 1 request a second through `gib_tree.Fetcher`). For 40 of
  them, `gib_tree.py show` with `pack-bottom`. The "GIB on the better call" column is GIB's
  own meaning of the call the field made. "(we have no rule)" means `show` lists the call
  under "GIB has, we have no rule for".
- **Cost.** The DD gain of the better call over ours, in IMPs (MP points passed through the
  IMP scale as a proxy). Recency weight: 1.0 for boards since 2026-11-10 (seat service on
  rbb v0.15.3), 0.5 for 10-25..11-09, 0.25 before that. Ranking = sum of weight x IMPs.

Classes: **(a1)** today's compiled-in rules already bid differently; **(a2)** only
`pack-bottom` fixes it; **(b)** low-level gap at a top-20k self-play position, so the GIB
comparison could have seen it; **(c)** low-level gap at a position our self-play never
produces (at most 6 of 100k boards, none in the top 20k); **(d)** higher level (round 3+,
slam, 5-level); **(e)** no South call that the field beat on DD, or judgment, style or
DD noise.

## Counts

| class | all 400 | since 11-10 (131) | 10-08..10-24 |
|---|---|---|---|
| (a1) fixed on today's main rules | 21 | 0 | 18 |
| (a2) fixed on pack-bottom only | 7 | 3 | 3 |
| (b) gap GIB could see | 58 | 24 | 17 |
| (c) blind spot (not in self-play) | 44 | 17 | 15 |
| (d) higher level | 36 | 15 | 11 |
| (e) judgment / noise / not bidding | 233 | 71 | 84 |
| known (11-26 MP-strong b1, doubler-rebid branch) | 1 | 1 | 0 |

Since 11-10 (the current engine live), 41 of 131 flagged boards are low-level gaps (b+c).
Of those, 17 sit at positions our self-play never reaches.
`pack-bottom` changes only 3 recent boards, and none of the (b)/(c) positions: at every one,
pack-bottom bids what Rusty bid. The bottom review fixed ranges at positions we *produce*.
These boards are mostly about *missing calls* (we pass) at positions we don't produce or
don't weigh.

**How the comparison sees the (b) ones.** 17 of the 58 appear verbatim as examples in
`agreed-20k-final.json`: our pass shows "pts open-top" (e.g. 3C-P P, P-P-1H-D-2H-P-P P,
1H-P-1N P), 1NT rebids "S max-wide", 3NT responses "hcp floor-wide". The rest sit in
the same big groups unlisted. Our pass at a position is compared to GIB's pass, so a
missing stronger call surfaces only as an open top in groups of thousands of boards (the
advances.bid:416 pass group alone holds 24,235). "GIB has, we have no rule for" (from
`show`) names the call directly. That listing is a better work queue than the range diffs
for this kind of gap.

## (a) Already fixed

- **a1 (21, all but 3 from October):** listed in the appendix. Includes the 4th-seat 2D/2H
  openings with 15+ (now 1x), the 1C-P-2C Michaels, takeout doubles of 1C-P-1D and 1H-P-1S,
  and keycard replies.
- **a2 (7):** 10-17 MP-strong b1 (P-1D-D-2H-P-P: 3C), 10-21 IMP b4 (pass, not double, of
  4S), 10-23 MP-strong b1 (power double 1N), 11-01 IMP-strong b2 (2H-D-3H-3S-P: 4S), 11-12
  MP b3 (P-1D-D-1N: 2S), 11-16 MP-strong b4 (P-1C-1H: negative double, not 2H cue), 11-17
  IMP-strong b8 (no short-suit try with 14). Two more boards change on pack-bottom away
  from the field without harm (11-12 MP-strong b2 3H for 4H, 11-26 IMP b3 2N try for 4H).
  I did not count those as fixes.

## (b) and (c): the patterns

Ranked by recency-weighted cost (score; boards; since 11-10; b/c; IMPs). The tables after
this section list every board.

1. **F1 Opener's rebid after partner's negative double and a further bid by them** (31.2; 7
   boards, 4 recent; 1 b / 6 c; 47 IMPs). We pass with four cards in the major partner's
   double promised, or with a six-card suit: 1D (2S) X (3S) with 4 hearts and a void
   gives P, where the field bid 4H 24 of 24. Over (1C) X (1N) with six clubs we pass, and
   over (2C) with 18 and four hearts we bid 3NT. The one (b) goes the other way: 2NT over
   (2S) with a 12-count. *Should:* show the promised major at the cheapest level
   (minimum) or jump to game (15+ or shortness); rebid a six-card suit; pass a minimum
   balanced hand. GIB's tree has every one of these (3S "4+ S, 15-17 TP", 4H "4+ H,
   15+"). *Blind spot because* our self-play opponents rarely raise or bid on over our
   negative double (0-5 boards in 100k at each position).
2. **F13 Responder passes below game with game values** (30.0; 5; 2; 0/5; 44). After a
   free 2C over their 1H we pass partner's 2NT with 16 HCP: we read our 2C as forcing one
   round only. Also: a 1H response with 15 passing partner's 2C rebid in competition; a
   cue raise then passing opener's 2NT with 14; six spades and 12 passing 1NT after
   their double; and 3D not competed over 2H. *Should:* 3NT or a forcing call. The free
   bid plus the opening is game. *Blind spot:* all positions at 0-5 self-play boards.
3. **F14 Opener's rebid after their takeout double and advance** (29.2; 5; 3; 2/3; 39). P
   with 18 and seven spades over (X) P (3C jump); 3D pulling partner's weak 2S over the
   double (Brill: 2S = 6+ spades, 6-9); 4H jump raise on a 13-count; P with 18 and six
   diamonds over (X) 1NT (b: 46 boards in self-play, GIB 3NT 18-21); 3H after 1H-2H-(X)
   read as invitational (b). *Should:* rebid the long suit with extras; pass partner's
   weak jump; raise one level on a minimum.
4. **F11 Advancer's raise of partner's overcall** (29.0; 7; 2; 6/1; 41). We pass with 4
   trumps and a singleton on 2 HCP over (2H): GIB 3S "4+ S, 4-8 TP", field 16/19. We pass
   three-card raises: 2H over their 1NT, 3S over their raise, 3H of a two-level
   overcall. We preempt 3H with 12 HCP and 4 trumps where the cue (limit+) is right, and
   raise to 2S on 3 HCP. *Should:* GIB's ranges (simple raise 6-10 TP with 3+; preemptive
   jump 4+ trumps 4-8; cue 11+). Our pass has an open top. All (b): this is in the
   comparison.
5. **F16 Direct-seat action** (26.5; 8; 2; 8/0; 54). Two-level overcalls too tight
   (KJT52 and 13, AQJT7 and 10, AKQ982 and 9: all P; GIB 2C 10+ HCP, 11-18 TP). Takeout
   double shape: we need three cards in the unbid minor (AJ32.AQ86.42.Q87 over 1C: P,
   field doubled 12/22). 1D overcall instead of X with 4-3 majors. 1NT overcall preferred
   to a good 5-card major (KQJ97.QT.97.AQJ3 over 1C: field 1S 16/18). No weak jump with
   seven diamonds, a void and four hearts. All at top positions (4,600-10,300 self-play
   boards).
6. **F2 Overcaller's second call after partner passed** (26.0; 6; 3; 3/3; 36). (1D) 1S (2D)
   P (P) with 5-4: P (field 2H). (1C) 1S (2C) P (P) with 5-5: P. (1C) 1D (X) P (1H) with
   AKQT9754: P. Also six-card spades after their bid over a negative double, and 5-4 with
   16 after (X) and 2C. *Should:* the second suit or the long suit, like the doubler-rebid
   fix. The same `!rho.last=P` style stop seems to cut off the overcaller's rebids.
7. **F4 Doubler's rebid beyond the known board** (25.5; 6; 2; 3/3; 45). After their raise
   and two passes: P with 19 and AKJT6 clubs (field 3C), P with seven diamonds (field
   3D), P with six hearts after (2D) X (4D) P (P). After a jump advance, P with 21 and six
   hearts. With 21 and six spades over (2D) X (2H) P (3H), P. Over (1C) X P 2D (jump), 2H
   with eight hearts (field 4H). The raise case has a rule (12 boards were in
   self-play), but it does not show a long suit. *Should:* belong to the doubler-rebid
   branch's scope. Check it there, not separately.
8. **F3 Overcaller's second call after partner bid and they competed** (24.5; 5; 1; 0/5;
   46). Partner raised and they bid on: P with six spades. Partner's 2S over our 1NT
   overcall: P with 4 spades and 17. Partner's 3H forcing: P with nine diamonds. Partner's
   new suit: P with 6 hearts and 2 small. And partner's **3C mixed raise** (Brill: 4+ H,
   7-9 TP, forcing to 3H) after (1C) 1H (1N): P, where the field bid 3H 20/23.
9. **F7 Values double missing over their interference** (21.8; 4; 1; 1/3; 34). (1D) 4S
   and (1D) 4C over partner's opening with 13 HCP: P (no rule; field doubled or bid 5D).
   Partner's weak 2S, their 2NT, we hold 15: P (field X 13/24, +1100; "GIB has, we have
   no rule for: D"). Their 3NT over our weak 2S with 12: P.
10. **F12 Advancing a two-suited or 1NT overcall** (21.5; 4; 2; 1/3; 36). 3D, not 4D/5D,
    over partner's unusual 2NT with 5 diamonds and 9 HCP (GIB 4D: the Law, 10 trumps).
    3H, not 4H, over Michaels with 5 hearts. Over partner's 1NT overcall we transfer and
    partner's **3C super-accept (Brill: doubleton club, 4 hearts)** is passed. Not
    competing 2D with six diamonds over their 2C.
11. **F10 Raising partner's weak two** (20.5; 4; 3; 4/0; 26). Work queue item 8. No
    preemptive 3D with three trumps and a void, nor 3S with four trumps and 12. In
    competition we jump to 4M on the Law with defensive values (QJ5.AKT6.T874.Q3 4D, field
    3D 17/24) or with 7 HCP. GIB: 3M "9 trumps, 4-16/17 TP".
12. **F5 Opener's rebid after their balancing call** (19.2; 5; 3; 5/0; 24). (1x) P P (1y or
    X or 1N): we pass with 5-4 and 15, with 17 after a reopening double, and with 20 and
    six diamonds. "GIB has, we have no rule for" lists 2C/2D/2H/3D at these positions
    (1D-P-P-1S alone: 144 self-play boards, our pass "pts top-narrow" in the agreed
    report).
13. **F9 Responding to partner's three-level preempt with a strong hand** (16.5; 3; 1; 3/0;
    29). P with 18 and AKQJT5 hearts over 3S (field 4H), with 15 and every suit stopped
    over 3C (field 3NT 13/18), and with 16 and a singleton over 3H (field 4H 14/24).
    Our rule says "No game: no fit, or not enough".
14. **F17 Opener's 1NT rebid preferred to a major or a raise** (13.5; 4; 3; 4/0; 14).
    1C-P-1D-P: 1NT holding KQJ2 hearts (field 1H 19/20; our 1NT "may hold a 4-card
    major"). 1C (1D) 1H: 1NT, not 1S with four spades. 1D (1H) 1S: 1NT, not the
    three-card raise. 1D (1S) 1N: a 3D jump with 16 and six diamonds, not 3NT.
15. **F15 Range floors from length points** (12.0; 4; 0; 4/0; 36). All October: 3NT over
    2NT with 3 HCP, 3NT after a transfer with 8, 2NT on 9 in a misfit. This is the range
    probe's finding (length counted into points); still bid the same today.
16. **F8 Competing over their preempt and its raise or new suit** (11.2; 4; 3; 3/1; 21).
    2D-P-3D, 2H-P-3H, 2S-P-3S: we pass with six spades and 11, seven diamonds and 13, and
    15 with five hearts (field 3S, 4D, X/4H). "GIB has, we have no rule for" lists 3S/4D.
    Also 3D-P-3S with a 5-5 and a void: P.
17. **F6 Opener reopening after their preempt passes round** (10.0; 3; 1; 0/3; 22). P with
    AKQJT872 diamonds after (4H) P P (field 5D 10/12). P with 5-5 and 14 after (4C) P P.
    X (support for hearts) instead of 3D with 5-5 after (3C) P P.

**Singles** (18 boards). The most telling: 1C (2H) X (P), where we pull to 4C holding
AQ865 hearts instead of passing for penalty (field 11/20; no rule). 1S-P-3S-X-P, where we
**read partner's takeout double of their invitational raise as penalty** and sit. A
help-suit trial bid after their double that we answer as a control bid. After our
penalty double of their 1NT they transfer, and we pass. The balancing double of 1S with
13 and three spades: P. The direct takeout double of their fit (P-1D-P-1H-P-2H) with 14
and a singleton: P. The values double of 1H-P-1N: P (536 of 538 self-play boards pass).
3D over (2H) as penalty with a singleton heart.

**Brill's conventions we misread (cross-cutting, all (c)).** Partner's 3C mixed raise
after (1C) 1H (1N); partner's 3C super-accept after our transfer over his 1NT overcall;
partner's help-suit trial after (X); partner's takeout double of 1S-P-3S. We read
partner by the Brill card. These are Brill treatments our system lacks or reads
otherwise, and GIB's tree cannot show them (GIB's 4D at 3D-P-3S is "27+ HCP", a
different system). Measure them with `brill_probe.py`-style comparisons of Brill's
`/explain`, not with GIB.

## (d) Higher level (36)

Round 3+ and slam boards, appendix. One recurring thing worth a look before the higher-level
rebalance: **9 passes with "No rule applies"** in round 3 of uncontested auctions, some
of them passes of forcing calls. 11-24 MP-strong b2 1D-1S-2D-2H with 6 diamonds and 4
hearts passed 2H (field 3H 17/22). The others: 11-24 MP-strong b5 and b6, 11-08 MP-strong
b3, 11-07 MP b5 and MP-strong b7, 10-18 MP-strong b3, 10-17 MP-strong b2, 10-16
IMP-strong b6. A pass that no rule makes is a hole, not a judgment. The engine could
flag it.

## (e) Not a low-level gap (233)

63 boards where no South call was beaten by the field on DD; the rest judgment, style
or noise. Light openings and 2NT-vs-1m with 20 and a five-card minor. One-board DD swings
where the field made our call too. Fourth-seat 2D with 12-13 HCP (Brill card treatment;
the field opens 1D): 10-19 MP-strong b8, 11-12 IMP-strong b4.

## Recommendations (frequency x cost, recency-weighted)

1. **Second calls in competition when we have more to say: one rule family, four
   seats.** F1 + F2 + F3 + F4 + F14 = 29 boards, 213 IMPs, 13 since 11-10. Opener after
   a negative double, overcaller after their raise or partner's call, doubler, opener
   after their takeout double. The shape is the same: after the opponents bid again, we
   pass with a second suit, a long suit, or the major partner promised. Write them as
   one pass over the "rebid in competition" rules, the way doubler-rebid is being done,
   and let doubler-rebid's design be the template. Top first: F1 (4 recent, 6 of 7
   unseen by self-play).
2. **Make self-play see these positions.** 44 of 102 gaps (17 of 41 recent) are at
   positions our self-play never reaches. Our robots raise, preempt and compete
   differently from Brill/GIB opponents. Run the GIB comparison also on positions taken
   from the Brill field PBNs and from bba/GIB corpora (opponents bidding their own
   system), or self-play with an EW engine on `general.style=gib`. The "GIB has, we have
   no rule for" list per position is the best queue. It named the missing call outright on 14 of
   the 43 boards whose positions went through `show`.
3. **Advancer's raises and direct-seat overcall/double floors** (F11 + F16 + F10 = 19
   boards, 121 IMPs, all (b)). Move toward GIB's ranges where the robots agree:
   preemptive jump raises with 4 trumps from 4 TP, the 3-card raise from 6 TP, the
   two-level overcall from 10 HCP with a good suit, and a takeout double with a doubleton
   in the unbid minor at 12-13. Judge on the bidding side's contract errors (Rick
   2026-10-31). The work-queue item 8 weak-two raises belong here.
4. **Values doubles and game over preempts** (F7 + F9 + F6 + F8 = 14 boards, 106 IMPs).
   Add the double over their 4-level preempt of partner's opening (13+), the penalty
   double of their 2NT/3NT over our weak two, 4M/3NT opposite partner's 3-level preempt
   with a strong hand, opener's reopening after (4x) P P, and competing over 2M-P-3M.
   Several have "no rule" outright.
5. **Responder with game values after a free bid** (F13, 5 boards, 44 IMPs, all (c)). A
   free new suit plus an opening is game: no pass below game.
6. **Opener after their balancing call; opener's 1NT vs a major** (F5 + F17, 9 boards,
   all (b), cheap fixes).
7. **Brill-convention reading** (mixed raise, super-accept form, trial bid after X,
   takeout double of their limit raise) for the Brill card: via the Brill `/explain`
   probe.

Caveat: the cost is DD gain from 1-24 field tables, so single boards are noisy. The
families are the signal, not the IMP totals.

## Boards by pattern

Columns: field = the other entrants' calls at the same prefix (count); self-play = boards through the position in 100k; class in brackets.

### F1 Opener's rebid after partner's negative double and a further bid by them

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-25 MP-strong b5 (c) | .QT87.AKJ72.KQ42 (15) | P-P-1D-2S-D-3S | P | 4H | 4H 24 | 1 | 3+ D; 4+ H; 15+ HCP; 16-21 total points |
| 2026-11-22 IMP b5 (c) | T74.6.AJT.AKJT62 (13) | P-P-1C-1H-D-3H | P | 4C | P 14, 4C 8, D 2, 5C 1 | 3 | 3- S; 21- HCP; twice rebiddable C; 19-22 total points |
| 2026-11-17 IMP-strong b5 (c) | K74.KJ96.AK72.A3 (18) | P-P-1D-1S-D-2C | 3N | 4H | 4H 13, 3H 4, 2S 1, 2H 1 | 0 | 3+ D; 4+ H; 11-21 HCP; 12-22 total points |
| 2026-11-16 IMP b2 (c) | A876.K8.KQ8632.5 (12) | P-1D-1H-D-3D | P | 3S | 3S 13, P 4, D 3, 4S 3 | 3 | 3+ D; 4+ S; 15-17 total points |
| 2026-10-24 MP-strong b1 (c) | 9.A863.AQ2.AQT97 (16) | P-P-1C-2S-D-3S | P | 4H | 4H 12, D 1, P 1 | 5 | 3+ C; 4+ H; 15+ HCP; 16-21 total points |
| 2026-10-13 MP-strong b3 (c) | A942.Q.K7.AJT953 (14) | 1C-1S-D-1N | P | 2C | 2C 10, D 2, P 1 | 0 | 6+ C; 11+ HCP; 12-15 total points |
| 2026-10-10 MP-strong b7 (b) | A74.K53.J752.A97 (12) | 1D-1S-D-2S | 2N | P | P 12, D 1 | 120 | No suitable call -- 3+ D; 11+ HCP; 12-14 total points |

### F13 Responder passes below game with game values after a free bid / in competition

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-25 IMP-strong b4 (c) | AK8.JT2.8.AKJ865 (16) | P-1D-1H-2C-P-2N-P | P | 3N | 3N 14 | 3 | 5+ C; 5- H; 5- S; 14-21 HCP |
| 2026-11-13 MP-strong b5 (c) | KQT972.82.K875.A (12) | 1D-P-1S-D-1N-P | P | 4S | 4S 9, 3S 3, 2C 2, 2S 1 | 0 | 6+ S; 12+ HCP; 13-16 total points |
| 2026-11-06 IMP b8 (c) | 872.AJ.8654.QJ64 (8) | P-1D-1H-1N-P-2C-2H | P | 3D | 3D 5, 3C 3, P 2 | 0 | 3-4 D; 3- S; 11- HCP; 11-12 total points; likely stop in H |
| 2026-11-04 MP-strong b1 (c) | AKQ4.A9865.QT97. (15) | 1C-P-1H-1S-2C-P | P | 2S | 3N 8, 2S 4, 2D 3, 3D 1 | 5 | 4+ H; 13+ total points |
| 2026-10-21 MP-strong b5 (c) | 8.A652.AQJ2.K743 (14) | 1D-1H-2H-P-2N-P | P | 3N | 3N 5, 3D 1, 3C 1 | 0 | 4+ D; 5- H; 5- S; 14-21 HCP |

### F14 Opener's rebid after their takeout double and advance

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-18 MP-strong b2 (b) | K4.AQ7.AKJT42.J7 (18) | P-1D-D-1N-P | P | 3N | 3N 11, 3D 6, 2N 3, 2D 1 | 46 | 3+ D; 5- H; 5- S; 18-21 HCP; 22- total points; likely stop in C; likely stop in H; likely stop in S |
| 2026-11-16 MP-strong b8 (c) | AKJ9653.A2.Q9.A9 (18) | P-P-P-1S-D-P-3C | P | 3S | 3S 8, 4S 7, 3N 2, P 1 | 0 | 6+ S; 21- HCP; 15-22 total points |
| 2026-11-10 MP b3 (c) | 4.AQT9.KJT32.KT4 (13) | 1D-D-1H-2S | 4H | 3H | 3H 16, 4H 2, P 1 | 0 | 3+ D; 4+ H; 13-16 total points |
| 2026-10-18 MP b6 (c) | Q5.8653.AKT43.A3 (13) | P-1D-D-2S-P | 3D | P | P 10, 3D 3, 3S 1 | 2 | No suitable call -- 3+ D; 1+ S; 11+ HCP; 12-21 total points |
| 2026-10-10 IMP-strong b3 (b) | AQT2.AJ543.T4.KJ (15) | 1H-P-2H-D | 3H | P | P 7, R 2, 2N 1 | 34 | No suitable call -- 5+ H; 11-21 HCP; 12-22 total points |

### F11 Advancer's raise of an overcall (simple, preemptive, cue-bid)

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-22 MP b3 (b) | 9864.Q6.T75432.8 (2) | P-1C-1S-2H | P | 3S | 3S 16, 4S 2, P 1, 2S 1 | 83 | 4+ S; 4-8 total points |
| 2026-11-20 IMP b6 (b) | T9842.K85.T.JT98 (4) | P-P-1D-1H-1N | P | 2H | 2H 15, P 11 | 37 | 3+ H; 6-10 total points |
| 2026-11-07 MP b3 (b) | K86.7.87542.KT54 (6) | P-2H-2S-3H | P | 3S | 3S 12, 4S 2, D 1, P 1 | 16 | 3+ S; 11- HCP; 8-12 total points |
| 2026-11-05 IMP-strong b5 (c) | A2.K83.K83.Q9865 (12) | P-1C-P-1D-2H-P | P | 3H | 3H 9, P 8, 4H 1, 3C 1 | 0 | The Law: 9 trump -> 3 level -- 4+ H; 4-16 total points |
| 2026-11-03 MP b8 (b) | A965.962.752.A62 (8) | 1S-2H-P | P | 3H | P 9, 3H 8 | 100 | 3+ H; 7-10 total points |
| 2026-10-25 IMP b7 (b) | 8543.T543.94.KT8 (3) | P-1C-1S-P | 2S | P | P 12, 2S 4, 3S 2 | 137 | No suitable call -- 10- total points |
| 2026-10-19 IMP-strong b8 (b) | QT75.KJT6.2.KQJ9 (12) | 1D-1H-2D | 3H | 3D | 3D 9, 4H 3, 3H 2, 2N 1 | 24 | — (we have no rule) |

### F16 Direct-seat action: overcall floors, takeout-double shape, 1NT vs a major overcall, WJO with 7

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-17 MP-strong b1 (b) | AJ32.AQ86.42.Q87 (13) | P-1C | P | D | D 12, P 8, 1H 2 | 6526 | Takeout double -- 2- C; 3-5 D; 3-4 H; 3-4 S; 12+ total points |
| 2026-11-17 MP b6 (b) | 9432.92.5.AKQ982 (9) | 1H | P | 2C | 2C 12, 3C 8, P 2 | 7211 | Two-level overcall -- 5+ C; 10+ HCP; 11-18 total points |
| 2026-11-01 MP-strong b6 (b) | KQJ97.QT.97.AQJ3 (15) | 1C | 1N | 1S | 1S 16, 1N 1, P 1 | 10298 | One-level overcall -- 5+ S; 8-17 HCP; 9-19 total points |
| 2026-10-21 MP b2 (b) | Q765.KQ5.AKJ32.4 (15) | 1C | 1D | D | D 11, 1D 3 | 10298 | Takeout double -- 2- C; 3-5 D; 3-4 H; 3-4 S; 12+ total points |
| 2026-10-19 IMP b1 (b) | QJ4.954.82.AQJT7 (10) | P-1S | P | 2C | P 11, 2C 8 | 4617 | Two-level overcall -- 5+ C; 10+ HCP; 11-18 total points |
| 2026-10-16 IMP b6 (b) | .Q642.KJT8532.Q3 (8) | 1S | P | 3D | 3D 10, 2D 3, 4D 2, 5D 1, P 1 | 7600 | Aggressive weak jump overcall -- 7+ D; 10- HCP; 3+ total points |
| 2026-10-13 MP-strong b1 (b) | J2.KJ8.A95.KJT52 (13) | P-1D | P | 2C | 2C 10, P 3 | 6958 | Two-level overcall -- 5+ C; 10+ HCP; 11-18 total points |
| 2026-10-09 IMP b2 (b) | J7.KQJ4.KQ72.986 (12) | 1S | P | D | D 10, P 7 | 7600 | Takeout double -- 3-5 C; 3-5 D; 3-4 H; 2- S; 12+ total points |

### F2 Overcaller's second call (partner passed): a second suit or a six-card suit

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-19 IMP-strong b4 (b) | KQJ932.K2.Q4.KJ7 (15) | P-P-1C-1S-D-P-2H | P | 2S | 2S 18, P 2 | 11 | — (we have no rule) |
| 2026-11-18 IMP-strong b4 (c) | KQ953.Q9.AKJ42.8 (15) | P-P-1C-1S-2C-P-P | P | 2D | 2D 15, P 3, 3D 1, D 1, 2S 1 | 3 | 4+ D; 5+ S; 17- HCP; 10-19 total points |
| 2026-11-13 IMP b2 (c) | T3.AQ.AKQT9754.7 (15) | 1C-1D-D-P-1H | P | 3D | 2D 6, 3D 2, 1N 1, 2C 1 | 3 | 6+ D; 17 HCP; 19 total points |
| 2026-11-08 MP b2 (b) | AJ9732.KQ6.9.KT6 (13) | 2D-2S-3D-P-P | P | 3S | 3S 8, P 6, D 2 | 13 | — (we have no rule) |
| 2026-11-08 IMP-strong b2 (c) | KQJ32.AQ63.72.Q7 (14) | 1D-1S-2D-P-P | P | 2H | 2H 9, P 4, D 2 | 1 | 4+ H; 5+ S; 17- HCP; 10-19 total points |
| 2026-10-09 IMP-strong b1 (b) | KJ543.A963.A.KJ7 (16) | P-1C-1S-D-P-2C | P | 2H | P 7, 2H 4 | 33 | 5+ S; 8-17 HCP; biddable H; 9-19 total points |

### F4 Doubler's rebid (after their raise, a jump advance, or their competing)

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-22 MP-strong b2 (c) | AQJT93.K5.KJ4.AK (21) | 2D-D-2H-P-3H | P | 3S | 3S 11, 4S 5 | 0 | rebiddable S; 19+ total points |
| 2026-11-20 MP-strong b5 (c) | A.AKQ98765.K.J87 (17) | P-1C-D-P-2D-P | 2H | 4H | 4H 5, 2H 2, 4N 1 | 0 | rebiddable H; 17+ total points |
| 2026-11-07 MP-strong b8 (b) | KJ3.QT.AKQ8432.Q (17) | P-P-1H-D-2H-P-P | P | 3D | 3D 7 | 40 | — (we have no rule) |
| 2026-11-05 IMP-strong b1 (b) | AQ6.T84.AJ.AKJT6 (19) | P-1H-D-2H-P-P | P | 3C | 3C 9, P 3, D 3 | 24 | — (we have no rule) |
| 2026-10-17 IMP-strong b4 (c) | KJ9.AKT532..AQ53 (17) | P-P-2D-D-4D-P-P | P | 4H | 4H 4 | 1 | 5+ H; 16+ HCP; 17+ total points |
| 2026-10-08 MP-strong b4 (b) | AJ3.AKQ542.J6.AQ (21) | 1D-P-P-D-P-3C-P | P | 3H | 3H 7, 4H 2, P 1, 3D 1 | 15 | 4 C; 4+ S; biddable H; 14+ total points |

### F3 Overcaller's second call after partner bid and they competed (incl. Brill's mixed raise)

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-25 MP b2 (c) | J7.AKJ65.Q42.976 (11) | 1C-1H-1N-3C-P | P | 3H | 3H 20, 4H 3 | 0 | 5+ H; 8+ HCP; 9-17 total points |
| 2026-11-04 IMP-strong b2 (c) | KJ6.3.AKQT87653. (13) | 1C-1D-3C-3H-4C | P | 5D | 5D 6, 6D 1, P 1 | 0 | 17- HCP; strong rebiddable D; 17-18 total points |
| 2026-10-30 IMP b5 (c) | AKJ8.J97.A32.KJ9 (17) | P-1C-1N-2H-2S-3H | P | 3S | 3S 9, P 3, 4S 3 | 0 | 2-5 C; 2-5 D; 2-5 H; 3-5 S; 17-18 total points; stop in C |
| 2026-10-22 MP-strong b6 (c) | KJ95.KT8652.74.A (11) | 1C-1H-1N-2D-P | P | 2H | 2H 7, P 5, 2S 3 | 0 | 6+ H; 11-15 total points |
| 2026-10-16 IMP b2 (c) | AQJ974.A3.T.J632 (12) | 1D-1S-2D-2S-3D | P | 3S | 3S 10, 4S 6, D 1 | 1 | 5+ S; 17- HCP; 12-19 total points |

### F7 Penalty/values double missing over their interference (4-level preempt over our 1x; overcall over our weak two)

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-25 MP-strong b4 (b) | Q.AKQ52.A62.9852 (15) | P-2S-2N | P | D | D 13, P 10, 3H 1 | 31 | — (we have no rule) |
| 2026-11-07 IMP b8 (c) | J6.98.AK42.AJT32 (13) | P-1D-4S | P | D | 5D 10, D 6, 4N 2, P 2 | 0 | 13+ HCP; 14+ total points |
| 2026-11-02 IMP b1 (c) | T5.AQT6.KJT6.Q32 (12) | 2S-3N | P | D | P 13, D 8 | 0 |   |
| 2026-10-23 IMP-strong b5 (c) | T987.AQ4.QJ75.A2 (13) | 1D-4C | P | D | D 12, P 2, 4D 2 | 0 | 13+ HCP; 14+ total points |

### F12 Advancing a two-suited overcall or a 1NT overcall

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-24 IMP b7 (b) | K8.53.Q9854.A863 (9) | P-1C-2N-P | 3D | 4D | 4D 12, 5D 6, 3D 6, 3C 2, P 1 | 34 | The Law: 10 trump -> game support -- 5+ D; 11- HCP; 4-12 total points |
| 2026-11-13 IMP b6 (c) | 874.43.QT7652.93 (2) | P-P-1H-1N-2C | P | 2D | 2D 9, P 8, 2N 1 | 4 | 5+ D; 6-9 total points |
| 2026-10-25 MP b4 (c) | J4.98632.A52.JT3 (6) | 1C-1N-P-2D-P-3C-P | P | 3H | 3N 3, 3D 2, 4H 2, 3H 2, 4C 1 | 0 | 5+ H |
| 2026-10-23 MP b8 (c) | J5.AQ762.T8.K765 (10) | 1D-2D-2N | 3H | 4H | 4H 8, 3H 5, P 2 | 6 | 4+ H; 7-12 total points |

### F10 Raising partner's weak two (alone and in competition)

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-26 MP b4 (b) | QJ5.AKT6.T874.Q3 (12) | P-2D-2S | 4D | 3D | 3D 17, 4D 3, 5D 1, P 1, D 1, 2N 1 | 50 | The Law: 9 trump -> 3 level -- 2+ D; 4-17 total points |
| 2026-11-16 MP b4 (b) | A32.J864.QT4.975 (7) | P-2H-2S | 4H | 3H | 3H 15, P 5, 4H 1 | 71 | The Law: 9 trump -> 3 level -- 2+ H; 4-17 total points |
| 2026-11-13 MP-strong b1 (b) | QT62.QT85.A7.KJ5 (12) | 2S-P | P | 3S | P 8, 3S 6, 4S 4, 2N 2 | 751 | The Law: 9 trump -> 3 level -- 3+ S; 4-16 total points |
| 2026-11-04 MP b5 (b) | .AT53.Q82.QT8652 (8) | 2D-P | P | 3D | 3D 11, P 2, 5D 2, 4D 2, 2N 1 | 641 | The Law: 9 trump -> 3 level -- 3+ D; 4-16 total points |

### F5 Opener's rebid after their balancing call ((1x) P P action)

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-18 IMP-strong b7 (b) | AQJ86.AK.K974.T6 (17) | 1S-P-P-D | P | 2D | 2D 12, P 4, R 2, 1N 1 | 249 | — (we have no rule) |
| 2026-11-16 IMP b3 (b) | AQJ95.AJ43.82.K6 (15) | 1S-P-P-1N | P | 2H | 2H 10, P 9, D 1 | 65 | — (we have no rule) |
| 2026-11-10 MP-strong b3 (b) | 75.A9.KJ963.AK62 (15) | 1D-P-P-1S | P | 2C | 2C 3, P 3, 1N 1 | 144 | — (we have no rule) |
| 2026-10-30 IMP b4 (b) | A9.J5.AKQ6.AQ654 (20) | P-P-P-1C-P-P-1S | P | 1N | 1N 2, 3D 1 | 11 | 3+ C; 18-19 HCP; 22- total points; stop in S |
| 2026-10-18 MP-strong b7 (b) | A9.QT.AK8732.AK9 (20) | 1D-P-P-1S | P | 3D | 3D 2, 2D 1 | 144 | — (we have no rule) |

### F9 Responding to partner's 3-level preempt with a strong hand

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-20 IMP-strong b4 (b) | KT93.3.KQJ62.AK9 (16) | P-3H-P | P | 4H | 4H 14, P 6, 3N 4 | 42 | 1+ H |
| 2026-11-08 MP-strong b1 (b) | A4.QJT6.AQ52.Q76 (15) | 3C-P | P | 3N | 3N 13, P 4, 4C 1 | 108 | 3- C; 2-5 D; 2-5 H; 2-5 S; 13+ HCP; stop in D; stop in H; stop in S |
| 2026-10-13 IMP b5 (b) | 2.AKQJT5.AQT72.Q (18) | 3S-P | P | 4H | 4H 9, 4S 5, 4N 1 | 185 | 17+ HCP; twice rebiddable H; 19+ total points |

### F17 Opener's 1NT rebid preferred to a major or a raise

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-25 IMP b3 (b) | A65.KQJ2.Q5.9742 (12) | 1C-P-1D-P | 1N | 1H | 1H 19, 1N 1 | 1191 | 3+ C; 4+ H; 11+ HCP; 12-18 total points |
| 2026-11-24 IMP-strong b1 (b) | 64.92.AKQT87.AQJ (16) | P-P-1D-1S-1N-P | 3D | 3N | 3N 9, 3D 4, 2N 1, P 1 | 33 | 3+ D; 5- H; 5- S; 17-21 HCP; 22- total points |
| 2026-11-20 MP-strong b1 (b) | Q863.43.AT7.KQJ5 (12) | P-P-1C-1D-1H-P | 1N | 1S | 1S 19, 1N 4, P 1 | 47 | 3+ C; 3- H; 4+ S; 11+ HCP; 12-18 total points |
| 2026-11-03 IMP-strong b1 (b) | J93.AJ98.A863.A2 (14) | P-P-1D-1H-1S-P | 1N | 2S | 2S 10, P 4, 1N 3 | 34 | Simple raise -- 3+ D; 3-4 S; 11+ HCP; 12-15 total points |

### F15 Range floors from length points (3NT/2NT with too few HCP)

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-01 IMP b1 (b) | A95.T8763.A7.T82 (8) | 1N-P-2D-P-2H-P | 3N | 2N | 2N 8, 3N 5, P 4 | 437 | Invitational to either game -- 5 H; 9 HCP |
| 2026-10-15 MP b8 (b) | A5432.82.Q6432.K (9) | P-1C-P-1S-P-2C-P | 2N | P | P 7, 2D 5, 3D 1 | 95 | No suitable call -- 4+ S; 6+ total points |
| 2026-10-11 MP b4 (b) | T3.52.T7642.K653 (3) | P-2N-P | 3N | P | P 11, 3N 1 | 475 | No suitable call -- 3- HCP |
| 2026-10-09 IMP b5 (b) | T863.Q.AT6.Q9754 (8) | 1D-P-1S-P-3D-P | 3N | 5D | 5D 5, P 5, 4D 4, 3N 2 | 54 | 2+ D; 4+ S; 12-13 total points |

### F8 Competing over their preempt and its raise or new suit (2M-P-3M, 3x-P-3y)

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-23 MP-strong b8 (b) | J765.T.AKQ9532.K (13) | 2H-P-3H | P | 4D | 4D 16, P 3, D 2, 5D 2 | 31 | — (we have no rule) |
| 2026-11-23 IMP-strong b8 (b) | QT.AQJ73.K3.QJ92 (15) | 2S-P-3S | P | D | D 9, 4H 8, P 6 | 38 | 3-5 C; 3-5 D; 3-4 H; 2- S; 14+ total points |
| 2026-11-20 MP b4 (b) | AQ8642.T754.J.A8 (11) | 2D-P-3D | P | 3S | 3S 16, P 4, 4S 1 | 30 | — (we have no rule) |
| 2026-10-09 IMP-strong b8 (c) | K52.AQ962..AQ987 (15) | 3D-P-3S | P | 4D | 4H 5, P 4, 4D 4, D 1 | 0 | 27+ HCP; forcing to 4N |

### F6 Opener reopening after their preempt overcall passes round

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-11 IMP-strong b2 (c) | A.873.AKQJT872.K (17) | P-1D-4H-P-P | P | 5D | 5D 10, P 2 | 0 | 21 HCP; strong rebiddable D; 22 total points |
| 2026-10-15 IMP-strong b5 (c) | 94.AKQJ5.KJ432.7 (14) | P-P-1H-4C-P-P | P | 4D | P 7, 4D 4, D 2 | 0 | 4+ D; 5+ H; 20-21 HCP; 22 total points |
| 2026-10-12 IMP-strong b1 (c) | T7542.AQ.AKQJ4.J (17) | P-P-1S-3C-P-P | D | 3D | 3D 8, D 2, 4D 1 | 0 | 4+ D; 5+ S; 21- HCP; 18-22 total points |

### F18 Singles

| board | South | position | ours | better | field | self-play | GIB on the better call |
|---|---|---|---|---|---|---|---|
| 2026-11-26 IMP-strong b8 (b) | AT85.A3.Q3.KT654 (13) | 2S-P-P | 3C | P | P 15, 3C 8, 2N 2 | 509 | No suitable call -- 18- total points |
| 2026-11-25 IMP-strong b7 (b) | 4.AQ865.K.AQJ532 (16) | 1C-2H-D-P | 4C | P | P 11, 3N 4, 3C 4, 4C 1 | 22 | — (we have no rule) |
| 2026-11-25 IMP-strong b8 (c) | KQ.KQ984.QJT74.Q (15) | P-P-P-1H-1N-D-2H | P | 3D | 3D 9, P 8, D 6, 3C 1 | 0 | 4+ D; 5+ H; 11-21 HCP; 12-22 total points |
| 2026-11-18 IMP-strong b5 (c) | QJ75.AJT9.985.AJ (13) | P-1H-P-2H-D-P | 4S | 3S | P 6, 2S 5, 4S 3, 3S 3, 2N 1, 3H 1 | 0 | 4+ S; 12+ HCP; 13-16 total points; forcing |
| 2026-11-16 IMP-strong b8 (b) | K3.AK95.K6.JT987 (14) | 1D-P-P | 2C | 1N | 1N 12, 2C 5, D 3, 1H 1 | 1035 | Balancing notrump -- 11-14 HCP |
| 2026-11-06 IMP-strong b5 (b) | AK75.6.Q972.AJ74 (14) | P-1D-P-1H-P-2H | P | D | D 9, P 3 | 128 | Takeout double -- 4+ C; 5- D; 5- H; 4+ S; 14-16 total points |
| 2026-11-05 MP b8 (b) | 873.QJ72.K5.AKT8 (13) | 1S-P-P | P | D | D 11, 1N 5, P 1, 2H 1 | 775 | 3+ C; 3+ D; 3+ H; 9+ HCP |
| 2026-11-02 MP b5 (c) | J83.9764.A632.Q6 (7) | 1S-P-2S-D-3C-P | 3D | 3S | 3S 9, 4S 3, P 2, 3D 1 | 0 | 3+ S; 7-8 total points |
| 2026-11-01 IMP-strong b3 (b) | K7.A5.K73.K98763 (13) | 1C-1H-1S-3H | 4C | P | P 17, 3N 1 | 10 | No suitable call -- 3+ C; 11-21 HCP; 12-22 total points |
| 2026-10-30 IMP-strong b5 (c) | 65.AKQ985.QT4.K5 (14) | 1D-D-R-1H-1S-P | P | 3N | 3N 5, 2N 1 | 0 | 4- D; 5- H; 5- S; 14-21 HCP; likely stop in C; likely stop in H; partial stop in S |
| 2026-10-28 MP b7 (b) | 95.KT4.Q.KJ86542 (9) | _ | P | 3C | 3C 11, P 6, 2C 1 | 100000 | — (we have no rule) |
| 2026-10-28 IMP-strong b6 (c) | A.J976.K92.AQJ94 (15) | P-1C-1D-1S-3C | P | D | D 9, P 3, 3H 2 | 3 | 11-21 HCP; rebiddable C; CKQ; 12-22 total points |
| 2026-10-27 MP b4 (b) | AJ9.Q6.A98.A9653 (15) | 1H-P-1N | P | D | P 8, D 8, 2C 3 | 538 | Takeout double -- 3+ C; 3+ D; 2- H; 4+ S; 13+ total points |
| 2026-10-18 IMP b7 (c) | 9.A7.J9652.AKJ86 (13) | 1D-P-1N-3S | P | 4C | 4C 9, P 7 | 0 | 4+ D; 19- HCP; rebiddable C; 18-21 total points |
| 2026-10-15 MP-strong b6 (c) | AQ.7.AQ96432.AT6 (16) | P-1D-2H-2N-3H | D | 3N | 3N 4, 6D 2, 5D 2, 4H 1, 4N 1, 4D 1 | 0 | 3+ D; 5- H; 5- S; 14-21 HCP; 22- total points |
| 2026-10-14 MP-strong b2 (b) | AJ8.AKJ97.AQ.A52 (23) | 4S | 5H | D | D 10, 4N 1, 5H 1, P 1 | 57 | 2+ S |
| 2026-10-11 IMP b8 (b) | J732.KQ.52.KT764 (9) | P-1S-2D | 3C | 2S | 3S 5, 2S 5, 3D 3, 4S 2, 2N 1 | 244 | Free major raise -- 3+ S; 6-10 total points |
| 2026-10-08 MP b6 (c) | Q87.63.J974.9632 (3) | 1S-P-3S-D-P | P | 4C | 4C 8, P 3, 4D 2 | 0 | 4+ C; 16- total points |

## Appendix: (a1) and (d) boards


**a1**

| board | position | Rusty | today (main / bottom) |
|---|---|---|---|
| 2026-10-08 IMP b1 | P-1C-P-P-1N-P | 4N | P / P (No game) |
| 2026-10-08 IMP b2 | 1C-P-1D-1H-1S | P | 2H / 2H (Raise: three-card support, 7+ total points) |
| 2026-10-08 IMP-strong b2 | P-1D-P-2C-P-2S-P-2N-P | 6N | 3N / 3N (Game: no extras) |
| 2026-10-08 MP b5 | 1D-P-1S-3H-P-P | P | D / D (Penalty double of their hearts) |
| 2026-10-09 MP b7 | _ | P | 2H / 2H (Weak two: six hearts, 5-10, no five spades) |
| 2026-10-09 MP b4 | 1H-P-1S | P | D / D (Takeout: both minors, 12+) |
| 2026-10-10 IMP-strong b4 | P-P-P-1S-P-3C-P | 3S | 4S / 4S (Game: 14+ total points, or 12+ with a fit in ) |
| 2026-10-12 IMP-strong b8 | 1S-P-2S | P | 3D / 3D (Overcall over their raise: five or more diamo) |
| 2026-10-12 MP-strong b4 | 1C-P-1D | P | D / D (Takeout: hearts and spades, 12+) |
| 2026-10-13 IMP-strong b7 | 1N-P-4H-P-4S-P-4N-P | 5D | 5C / 5C (0 or 3 keycards) |
| 2026-10-14 IMP-strong b4 | 2D-P-5D | D | 5H / 5H (Six or more hearts, 16+) |
| 2026-10-14 IMP-strong b5 | P-4C | P | D / D (Takeout: 14+, short in ♣, or 16+) |
| 2026-10-14 MP b7 | 1N-P-2S-P | 3C | 2N / 2N (No four-card minor) |
| 2026-10-14 MP-strong b4 | P-P-P | 2D | 1D / 1D (3+ diamonds) |
| 2026-10-15 IMP-strong b8 | P-1S-3H-D-P-3S-P | 3N | 4S / 4S (Game in ♠: 13+ with an eight-card fit) |
| 2026-10-15 MP-strong b4 | P-P-P | 2H | 1H / 1H (Major opening: 5+ hearts) |
| 2026-10-20 IMP b4 | 1C-P-2C | P | 3C / 3C (Michaels: five or more in each major) |
| 2026-10-23 MP-strong b4 | P-P-P | 2D | 1D / 1D (3+ diamonds) |
| 2026-10-27 MP-strong b4 | P-1S-P-2C-P-2D-P | 2S | 3N / 3N (Game, no fit: 15-17) |
| 2026-11-07 IMP b5 | 1C-P-1D-1H-D-P | P | 2D / 2D (Five or more diamonds, up to 9) |
| 2026-11-09 MP b8 | P-1S-P-2D-P-2N-P-3S-P-4N-P | 5D | 5C / 5C (0 or 3 keycards) |

**d**

| board | position | Rusty | today (main / bottom) |
|---|---|---|---|
| 2026-10-08 IMP-strong b6 | P-P-P-1S-P-1N-P-2C-P | 3C | 3C / 3C (Raise: 4+ ♣, 11-12 HCP) |
| 2026-10-09 MP b2 | 2C-P-2D-P-2H-P-2S-P-3H-P-4H-P-4N-P-5C-P-6H | P | P / P (No further bid) |
| 2026-10-12 IMP b1 | 1S-P-2N-3C-P-P-3S-P-4S-P | 4N | 4N / 4N (Keycard ask in ♠) |
| 2026-10-13 MP-strong b6 | 1D-P-1S-P-1N-P-2S-P-P | P | P / P (No further bid) |
| 2026-10-14 IMP-strong b6 | P-1H-P-2H-D-P-3D-3H-P | 4H | 4H / 4H (Competes: to play) |
| 2026-10-14 MP-strong b3 | 1D-P-1S-P-1N-P-2C-P-2S-P-3S-P | 4S | 4S / 4S (Accepts, in the fit) |
| 2026-10-16 IMP-strong b6 | P-1C-D-1H-P-2D-P-2N-P | P | P / P (No rule applies) |
| 2026-10-17 MP b5 | P-P-1H-D-2H-2S-3H-3S-P-P | P | P / P (No further bid) |
| 2026-10-17 MP-strong b2 | P-1D-P-1H-P-1S-P-3D-P | P | P / P (No rule applies) |
| 2026-10-18 IMP-strong b6 | 1D-P-P-1S-P-2D-P-2S-P | P | P / P (No further bid) |
| 2026-10-18 MP-strong b3 | 1N-P-2C-P-2S-P-4D-P | P | P / P (No rule applies) |
| 2026-10-25 IMP b3 | P-P-1D-1S-P-P-1N-P | P | P / P (No game) |
| 2026-10-25 MP b3 | 1C-1D-1H-P-2C-2D-P-P | P | P / P (No further bid) |
| 2026-10-27 IMP-strong b1 | 1D-P-1S-D-R-P-P-2C-P-P | D | D / D (Penalty double of their clubs) |
| 2026-10-30 IMP b1 | 1N-P-2D-P-2H-P-3N-P-4D-P | 6D | 6D / 6D (Small slam: a fit) |
| 2026-10-31 MP b2 | P-P-1C-1N-P-2H-P-2S-P | 2N | 2N / 2N (Invitational: 8-9 total points, exactly 5 ♠) |
| 2026-11-01 MP-strong b2 | P-1C-P-1S-P-2N-P-3D-P-3S-P-4N-P | P | P / P (Declines: not enough for slam) |
| 2026-11-02 MP b2 | P-P-P-1D-1S-2H-P-3H-P | P | P / P (No further bid) |
| 2026-11-07 MP b5 | P-P-1N-P-2C-P-2H-P-2S-P | P | P / P (No rule applies) |
| 2026-11-07 MP-strong b7 | 1N-P-2C-P-2D-P-3C-P | P | P / P (No rule applies) |
| 2026-11-08 MP-strong b3 | 1C-P-1D-1H-2C-P-2S-P | P | P / P (No rule applies) |
| 2026-11-11 MP-strong b7 | 1D-P-1H-P-1S-P-3H-P | P | P / P (Declines) |
| 2026-11-13 MP-strong b7 | 1S-P-2H-P-3C-P-3D-P | 3N | 3N / 3N (Game in notrump) |
| 2026-11-14 IMP b3 | 1H-P-1S-P-2S-P-3C-P | 3D | 3D / 3D (Control bid in ♦) |
| 2026-11-14 MP b6 | 1S-P-P-2D-2S-3D-3S-P-P | P | P / P (No further bid) |
| 2026-11-15 MP-strong b7 | 1N-P-2D-2H-P-2S-P-P | P | P / P (No further bid) |
| 2026-11-20 IMP b7 | P-1C-1N-P-2H-P-2S-P | 3N | 3N / 3N (Choice of games: 5 ♠, 10-15 total points) |
| 2026-11-22 MP-strong b3 | 1N-P-2C-P-2H-P-3H-P | 4H | 4H / 4H (Accepts, in the fit) |
| 2026-11-24 MP b6 | P-1H-P-2N-P-3C-P-3D-P | 4C | 4C / 4C (Control bid in ♣) |
| 2026-11-24 MP-strong b5 | P-P-2C-P-2D-P-3C-P-4C-P | P | P / P (No rule applies) |
| 2026-11-24 MP-strong b2 | P-1D-P-1S-P-2D-P-2H-P | P | P / P (No rule applies) |
| 2026-11-24 MP-strong b6 | P-1S-P-1N-P-2N-P-3H-P | P | P / P (No rule applies) |
| 2026-11-25 IMP b6 | P-P-P-1N-P-2D-P-2N-P | 4H | 4H / 4H (Game: in the fit, opposite a super-accept) |
| 2026-11-25 MP b8 | P-P-1D-1N-P-2D-P-2H-P-2N-3D | P | P / P (Declines) |
| 2026-11-25 MP b3 | 1C-1H-D-2H-3C-3H-P-P | P | P / P (No further bid) |
| 2026-11-25 MP-strong b6 | P-1S-P-1N-P-2H-P-3D-P | P | P / P (No further bid) |

## Files

Scratch: `bg/boards2.json` (all 400 boards, every South decision with field, today's calls, self-play counts), `bg/cls.py` (the hand classification), `bg/final.json`, `bg/rows.json`, `bg/gibtrees.json`, `bg/show/` (gib_tree show output).
