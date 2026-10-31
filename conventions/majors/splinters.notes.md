# splinters (`splinters.bid`): notes

Splinters over a major opening, by responder and by opener, and the
slam-or-game decision over them. Cases: `splinters.test`.

## What the card says

`major_openings.splinters.play` (BBA's `Splinter`; on Rick's card, on
21GF-DEFAULT and 21GF-GIB) switches the module on.
`splinters.by_responder` and `splinters.by_opener` default to on: a card
that says only "Splinters" plays both, as BBA does with its one switch.
`mini_splinters.play` (BBA's `Mini Splinter`, set by no corpus card) is
not played.

## The structure

- **Responder's splinter**: a double jump in a new suit over 1♥ (3♠, 4♣,
  4♦) or 1♠ (4♣, 4♦, 4♥): four or more trumps, 12-15 HCP (11 with a void)
  and a singleton or void that is not an honour. It agrees the suit and
  forces to game (`trump=M, forcing=game`), so the slam machinery
  (slam/) runs from there. Above 15, and with a singleton honour, Jacoby
  2NT. A passed hand splinters with 10-11 and four trumps (ahead of
  Drury).
- **Opener's splinter**: after 1x–1M (a one-level major response), a
  double jump in a new suit (1♣/1♦–1♥–3♠; 1x–1M–4y below 4M): four
  trumps, 17-21 HCP (16 with a void), a singleton or void that is not an
  honour.
- **The partner of the splinterer** counts support points less the
  honours wasted in the short suit (K 3, Q 2, J 1; the ace is not
  wasted). Over responder's splinter: 14 or more make a slam try; over
  opener's: 11 or more. The try is the cheapest control bid below game
  when a side suit is bare (two or more cards, neither ace nor king) and
  the card plays control bids, otherwise the keycard ask (Rick's rule in
  control-bids.bid: control bids before keycard when a suit is
  unprotected). With less, game.

Not a jump shift: strong jump shifts are single jumps (1♥–2♠),
invitational ones single jumps to three (1♠–3♣); the double jumps had no
meaning before. `rebids.bid`'s answers to an invitational jump shift
(`after 1x (P) 3y (P)`) now require no agreed trump, so they no longer
read 1♥–3♠ as natural spades (opener used to raise to 4♠).

## Evidence

**BBA's responder** (`probes/splinter-resp-1H.toml`, `-1S.toml`, 1000
hands each with four trumps and shortness, 21GF-DEFAULT): its splinter
is "12 to 20 total points", but it splinters only with 12-15 HCP (11
with a void) and a small singleton; a singleton honour, the ace
included, goes through Jacoby 2NT, as does 16+. 8-9 HCP bid 4M
(preemptive), 10-11 Jacoby 2NT. Agreement 685/1000 and 658/1000.

**Opener over the splinter** (`probes/splinter-opener-1S-4D.toml`,
`-1H-4C.toml`, 800 hands each): BBA's line is about 13 HCP outside
wasted honours (mixed at 11-13, always a try from 14), and its try is
always a control bid. Ours: 355/800 and 380/800; the difference is
mostly that we ask 4NT when no side suit is bare.

**The slam line, on the corpus** (Exclusion_After_1M, Gavin_Strong_Splinter,
Splinters, the four Jacoby_2N scenarios, Gavin_Weak_Splinter,
Slam_After_Major_Fit, Splinters_By_Opener; −3,506 before):

| Opener's slam try from | vs BBA |
|---|---|
| 13 HCP less waste (BBA's line) | −3,860 |
| 12 support points less waste | −3,118 |
| 13 support points less waste | −3,014 |
| **14 support points less waste** | **−3,001** |
| 15 support points less waste | −3,256 |

Counting HCP alone missed the hands whose value is shape: a void and a
seventh trump. The splinter's range: 12-16 −2,984 and 11-15 about equal
(noise); 12-15 kept, BBA's and the books' range.

**Opener's splinter** (`probes/splinter-by-opener-1D-1S.toml`, 800
hands): BBA splinters from 17 HCP (16 with a void), about half the time
(the rest raise to 4♠). Splinters_By_Opener −1,107 → −859; the
responder's threshold (9, 11, 13) changed nothing measurable.

**Corpus, the whole change** (with Drury, same commit): Splinters −33 →
−13, Gavin_Strong_Splinter −23 → +138, Jacoby_2N +33 → +126,
Jacoby_2N_4x_void −57 → +19, its Leveled form −129 → −2,
Splinters_By_Opener −1,107 → −859.

## Accepted differences from BBA

- Over a splinter with every side suit protected we ask for keycards;
  BBA cue-bids (Rick's control-bid rule, control-bids.bid).
- The slam line is on support points (shape counts); BBA's is on HCP.
- Opener splinters every time the hand qualifies; BBA sometimes raises
  to game instead.
- A passed hand's splinter needs four trumps; BBA's takes three at 11.

## Gaps

- **Minor-suit splinters**: responder's splinters in support of a minor
  live in `inverted-minors.bid` (13+ HCP, only with inverted minors) and
  are not gated on this field. Splinters_after_Minor is −1,242; BBA
  splinters there with less (1♦–3♠ and 1♦–3♥ are its two costliest
  divergences). Left for a separate change.
- Mini-splinters (the field exists, no corpus card sets it).
- Splinters after a jump or a 2/1 rebid (1♠–2♣–2♦–4♥), and Rick's
  card's `slam.control_bids` text is not read. Opener's splinter over the
  2/1 itself and responder's over opener's raise or second suit are
  written ("Later splinters", below).
- Splinters in competition: only responder's in their suit over a
  two-level overcall (below); the rest need the opponents to pass.
- **Transfer Walsh** (2026-10-05): opener's splinters stand aside after
  1♣-1♦/1♥ when the card plays transfer responses
  (`minor_openings.one_club.transfer_resp`): they read the transfer as
  the natural suit and agreed the wrong major (Transfer Walsh self A/B,
  transfer-walsh.notes.md). Opener's splinters in the transfer's major
  are not written.

## Their double of the splinter (2026-10-15)

Brill daily 2026-10-11 board 1 (MP): 1♠ (P) 4♦ (X) 4♠ (P), Rusty South
the splinterer with AKT87.KQT62.3.64 (12 HCP). On the day it bid 4NT and
reached 6♠ (12 tricks double dummy, one off at the table); the replay
passed 4♠ under rebids.bid's "Nothing to add: partner's rebid was a
minimum". Not a regression in the slam rules: on the day opener's 4♠
over the double was read by rebids.bid as "Game: six or more spades,
19+" (no rule here covered the doubled splinter), which made South's
keycard ask look right; since 2026-10-09 it was read as total-tricks'
"ten trumps, 10-12". Both are misreadings: opener's 4M is the sign-off
whether or not they double the artificial splinter (it takes no room).
The opener's block now covers `1M (P) 4x (X)` and `1H (P) 3S (X)`, so 4♠
reads "Game: no slam interest", and the splinterer with no extras passes
it, as he does uncontested (`1S P 4D P 4S P` passes too). 12 HCP
opposite a sign-off is no slam try by any count here; the slam on the
day was the misreading's luck. Opener's redouble or pass over the
double (control-showing) is not written.

## Later splinters (2026-10-30)

Gaps found by comparing Brill's convention files with our rules (calls
we had no rule for; brill_probe/brill_cases, the "Splinter" and
"Overcall1HSplinter" groups). Brill's definitions were the hint, not the
spec; the ranges are ours.

- **In their suit over a two-level overcall**: 1♥ (2♣) 4♣, 1♥ (2♦) 4♦,
  1♠ (2♣/2♦/2♥) 4 of their suit: the responder's splinter (4+ trumps,
  12-15 HCP or 11 with a void, a small singleton or void in their suit),
  not by a passed hand. The simple cue bid stays the limit raise or
  better. Opener answers as over the uncontested splinter (the opener's
  block takes `1M (2x) 4x (P)` too). BBA plays it: the corpus boards
  where we now bid it (After_Opp_Overcalls 176, Dealing_with_Overcalls_Weak
  240, Transfer_Advances 23) are BBA's own call.
- **Opener over a two-over-one** (seat 1 or 2 only: a passed hand's 2♣ is
  Drury): 1♠–2♥–4♣/4♦, four hearts, a small singleton or void, 14+ HCP:
  the fast 4♥ (1♠–2♥–4♥, rebids.bid 14-17) is the other hand, so the
  splinter carries slam interest. Responder counts support points less
  waste from 16 (4♥ or a control bid / keycard, as above).
- **Natural for now** (Rick, 2026-10-31: "advanced splinters deserve
  research as a whole"): opener's splinters raising a two-over-one minor
  (1♥–2♣/2♦–3♠, 1♠–2♣–4♦) and 1♠–2m–3♥. The 1♥–2m–3♠ splinter was
  written and taken out again; BBA bids it (Two_Over_One 380, Serious
  15, Slam_After_Major_Fit 445), so those boards go back to differing.
- **Responder over opener's single raise**: 1m–1♥–2♥–3♠ and
  1x–1M–2M–4y (y a new suit below 4M): 4+ trumps, a small singleton or
  void, 18+ support points (slam interest: the raise is 12-15 and plain
  game values bid 4M). Opener tries for slam from 15 less waste.
- **Responder over opener's second suit**: 1♣–1♥–1♠–4♦ and
  1♦–1♥–1♠–4♣ agree spades as a splinter over 1♠ would (12-15, 11 with
  a void); opener counts from 14 as over the splinter of his opening.

The counts are named once (`spl_netN(t, s)`, `spl_short(s)`,
`spl_bare(t, s)`, defined in splinters.bid) for the new blocks; the
older blocks still spell the eight branches out.

Measured: `compare` on Splinters, Splinters_By_Opener, the Gavin
splinters, Slam_After_Major_Fit, Two_Over_One, Reverse_After_Two_Over_One,
After_Opp_Overcalls, Dealing_with_Overcalls_Strong/Weak and
Mixed_Raise_In_Comp (500 boards each): 6 boards changed, every new
splinter BBA's own call; errors +11 on one board (Slam_After_Major_Fit
86, 7♥ for 6♥), 0 on the rest. The all-scenario tripwire (`--limit 50`,
17,100 boards): 2 boards changed (Serious 15, Transfer_Advances 23), both
to BBA's call; totals unchanged.

Accepted difference: Dealing_with_Overcalls_Weak 203, 1♠ (2♥) with a
heart void and 13 HCP: we splinter 4♥, BBA cue-bids 3♥; both reach 5♠.
Slam_After_Major_Fit 86: opener with seven hearts and four clubs
splinters 1♥–2♣–3♠ (void); showing the hearts first may be better, but
it reached the grand that par wants.

**Left out**:
- **Advancer's splinter** (Brill's Overcall1HSplinter: (1♣) P (1♦) 1♥
  (1NT/P/1♠/2♣/2♦) 3♣/3♦/3♠/4♣/4♦, a passed hand's jump in their suit
  raising the overcall). No card field covers it, so it cannot be self
  A/B'd, and the position (both opponents bid, advancer passed) is rare
  enough that 500 random boards would not show it. Our rules have no
  meaning for those jumps, so nothing conflicts (fit-showing jumps are
  in new suits; the mixed-raise jump cue is direct-seat `(1x) 1y (P)`).
  Rick, 2026-10-31: not now.
- **After Stayman** (1NT–2♣–2♥–4♣/4♦): 4♣ there is Gerber on many cards;
  left to the notrump modules.
- 1♥ 1♠ 2♠ (P) 4♣ (the task excluded it) and the splinters over 1NT
  interference.

## Sources

- **Convention description**: Wikipedia, "Splinter bid" (double jump in
  a side suit; four-card support, a singleton or void, game values,
  "typically 10-15 HCP"; opener re-values, "Axxx is ideal whereas KJ9x is
  almost worthless" in the short suit; opener's splinter with 15-18). We
  take the lower end from BBA (12, or 11 with a void) and opener's range
  from BBA (17-21), and we exclude singleton honours as BBA does.
- **Bridge-Classroom** `conventionCatalog.js`: `splinters.play` ("double
  jump shows shortness").
- **BBA probes** (2026-09-28): `probes/splinter-resp-1H.toml`,
  `probes/splinter-resp-1S.toml`, `probes/splinter-opener-1S-4D.toml`,
  `probes/splinter-opener-1H-4C.toml`,
  `probes/splinter-by-opener-1D-1S.toml`.
- **Corpus measurements** (2026-09-28): the slam line and the ranges,
  the tables above.
- **Later splinters** (2026-10-30): the splinter in the overcaller's
  suit, opener's over a two-over-one and responder's over a raise or
  opener's second suit follow the Wikipedia definition (double jump, or
  a jump past a reverse, in a side suit; four trumps; shortness) applied
  one round later; the ranges are ours (above), standard practice, not
  yet cited beyond that. BBA bids the same calls on the corpus boards
  named above. Brill's convention files pointed at the gaps (no text
  taken from them).
- **Rick's rulings**: control bids before keycard when a suit is
  unprotected, and the 33-support-point slam count (control-bids.bid,
  rkcb-1430.bid, 2026-09-25/27); the wasted-honour count here is a
  refinement of that count, not a ruling.
- **The doubled splinter** (2026-10-15): opener's game bid stays the
  sign-off over a lead-directing double, standard practice, not yet
  cited; the Brill daily 2026-10-11 board 1 for the motive.
