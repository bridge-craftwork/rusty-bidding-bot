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
- Splinters after a 2/1 or a jump (1♠–2♣–2♦–4♥), and Rick's card's
  `slam.control_bids` text is not read.
- Splinters in competition: the patterns need the opponents to pass.

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
- **Rick's rulings**: control bids before keycard when a suit is
  unprotected, and the 33-support-point slam count (control-bids.bid,
  rkcb-1430.bid, 2026-09-25/27); the wasted-honour count here is a
  refinement of that count, not a ruling.
