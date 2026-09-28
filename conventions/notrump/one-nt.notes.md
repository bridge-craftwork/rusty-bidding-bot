# notrump-base (`one-nt.bid`): notes

The 1NT opening, responder's natural notrump raises, and opener's answer to
2NT. Cases: `one-nt.test`.

## Guidance

- 1NT is **15–17 HCP exactly**, balanced, as BBA plays it. Tens and length
  do not move the range. The card option "1NT may be 1 HCP light"
  (`notrump.one_nt.allow_one_less`) opens a 14 whose extras bring it to 15
  total points.
- Responder's strength is in **total points: HCP + ½ per ten + 1 per card
  beyond four** (Rick, 2026-09-23: "usually the NT opener doesn't count
  length points, but responder does"). Until 2026-09-23 it was HCP + ½
  per ten, and a fifth card did not count.
- Bands are measured against opener's range. Opposite 15–17: sign off 0–7,
  invite 8–9, game 10–15, slam invite (4NT) 16–17, slam (6NT) 18+.
- Opener accepts 2NT **only when game is certain**: 17 opposite 8–9, in
  total points (so 15 HCP with four tens accepts, and so does 16 with a
  five-card suit; see "Length points" below for why opener counts length
  too).
- Differences in where a point boundary falls are acceptable; mistakes are
  not. **We do not model BBA's finer valuation: the aim is defensible bids**
  (Rick, 2026-09-21).

## Evidence from BBA

Probes (21GF-DEFAULT). Reproduce with, for example,
`rbb probe --hand S=AQ52.KJ73.A95.J8 --vary-tens --prefix "1NT Pass 2NT Pass" --dealer S`.

| Decision | Hand (then 1–4 more tens) | BBA | Agree |
|---|---|---|---|
| Opener over 2NT | `AQ52.KJ73.A95.J8` (15) | pass with 0–3 tens, 3NT with 4 | 5/5 |
| Opener over 2NT | `AQ52.KJ73.A95.Q8` (16) | pass with 0–1, 3NT with 2+ | 5/5 |
| Opener over 2NT | `AQ952.KJ7.A95.Q8` (16, 5-3-3-2) | the same: the fifth card does not count | 5/5 |
| Responder | `Q82.K73.J83.K954` (9) | 2NT with 0–1, 3NT with 2+ | 5/5 |
| Responder | `Q82.K73.J832.K95` (9) | the same | 5/5 |

Corpus: BBA's call over `1NT P` with balanced hands and no four-card major,
by HCP and tens (all scenarios, one row per deal):

| HCP | 0 tens | 1 ten | 2 tens | 3 tens |
|---|---|---|---|---|
| 7 | P 100% | P 100% | 2NT 66% | 2NT 71% |
| 8 | P 65%, 2NT 32% | 2NT 71% | 2NT 61%, 3NT 23% | 2NT 48%, 3NT 27% |
| 9 | 2NT 94% | 2NT 79% | 3NT 60% | 3NT 68% |
| 10 | 3NT 71% | 3NT 83% | 3NT 80% | 3NT 65% |

½ per ten is BBA's central tendency, not its whole rule (see below).

Basic_NT (500 boards, played with BBA's **Basic-Bridge** card, not
21GF-DEFAULT): 97.2% of calls agree, 85.4% identical auctions, 88.2% the
same contract.

## Accepted differences from BBA

- **BBA's valuation has more in it than HCP and tens.** These probes are the
  same on every random layout, and they are the same with Basic-Bridge and
  21GF-DEFAULT, so this is not noise. Rick: not modeled.
  `984.95.8654.AKQJ` (10) and `75.AK6.Q753.J752` (10): BBA invites;
  `T82.QT7.A82.QT65` (8, three tens) and `T64.AT8.QJ9.J432` (8, two tens):
  BBA bids game; `72.A97.K432.J983` (8): BBA passes.
- **8 HCP flat with no tens**: we invite, and BBA passes 65% of the time.
- **Five-card minor**: BBA does not count tens. `Q5.K73.J83.K9542` with two
  to four tens: BBA 2NT, we 3NT. We count them.
- **16–17 opposite 15–17**: BBA often bids 6NT directly (13 Basic_NT
  boards). We invite with 4NT, as most players would.

## Gaps (not built yet)

- (Six-card minors are now `minor-transfers.bid`; see its notes.)
- **Grand slam**: nothing above the slam band. BBA bids 7NT with 21–23.
- **Interference** over 1NT: a double and a 2♣ overcall are now covered
  (`nt-interference.bid` and the `after 1N (X)` / `after 1N (2C)`
  contexts in `stayman.bid`, `jacoby-transfers.bid`,
  `minor-transfers.bid`, `texas-transfers.bid` and the natural calls at
  the foot of this file). **Anything higher than 2♣ is still not
  covered**, and that is where Lebensohl belongs
  (`notrump.lebensohl.over_interference`, on for 21GF-DEFAULT).

## Fixed (2026-09-21)

- 18+ balanced had no rule and passed 1NT (7 Basic_NT boards). Added 6NT;
  with a five-card major, responder transfers first. Stayman and the
  transfer continuations got slam rules too. Corpus: +365 calls agree,
  same contract 10.6% → 11.1%.

## A strong responder never passes 1NT (2026-09-22)

The par-priced work queue put `1NT P — BBA 2♠, we pass` fifth in the
corpus, 322 boards and -2,669 IMPs. Two separate things were going on,
and only one of them is ours.

**Not a transfer floor.** Jacoby transfers have no strength condition —
`shows H>=5` and nothing else — and a 0-count with five hearts transfers.
That was the first guess and it was wrong.

**BBA's 2♠ was Minor Suit Stayman**, on `21GF-MSTandMSS`: the card sets
`1N-2S Minor Suit Stayman = 1`, `1N-2N transfer to clubs = 1` and
`1N-3C transfer to diamonds = 1`. Our card derivation has no option for
that combination, so `notrump.minor_transfers` falls through to `none`
and no rule claims 2♠ (see the gaps below).

**What was ours**: responder held 16-21 with a long minor, and every
natural response we had wanted either a five-card major or a balanced
hand — 6NT, 4NT and 3NT all ask for `balanced` or an exact band — so
nothing matched and he *passed* a 1NT opening with 17 HCP. Now:

- **3♣ / 3♦** with six or more and too strong for 3NT
  (`strength>=slam_invite`), when the card leaves those calls natural
  (`transfers.three_c_diamonds`, `three_d_response`, and not the `bba`
  minor style);
- **3NT at `priority -5`** as the catch-all for `strength>=game`, so a
  strong hand always has a call.

The 3♣/3♦ rules were `strength>=game` at first, which bid the minor on
ordinary game hands where BBA bids 3NT: -104 calls in Ned_2S alone. The
band matters.

| scenario | before | after |
|---|---|---|
| Ned_2S contracts | 60.2% | 61.0% |
| Minor_Suit_Stayman contracts | 60.2% | 62.2% |
| MST_or_MSS contracts | 1.8% | 3.4% |

Corpus par improves by 623 IMPs and the divergence point drops from
-2,669 to -2,198, where it stays as a *treatment* gap: we now bid 3NT
where BBA starts a minor-suit sequence with 2♠.

## Under interference (2026-09-23)

`notrump.transfers.vs_double` and `notrump.transfers.vs_2c` were both on
for Basic-Bridge and read by no rule: every notrump pattern was
`after 1N (P)` or a competitive spelling of it, so a double or a 2♣
overcall turned Stayman and the transfers off and responder passed
whatever he held — 1,331 and 1,243 corpus decisions.

Responder's natural notrump calls over a 2♣ overcall are written at the
foot of `one-nt.bid` as their own `after 1N (2C) when vs_2c` context:
2♣ takes none of them away, so 2NT is still the invitation and 3NT the
game. Over a *double* there is no such block — BBA redoubles with the
balanced 6+ hands instead of bidding notrump, and that rule is in
`nt-interference.bid`.

The full evidence, the two treatments of the double, and the measured
effect are in `nt-interference.notes.md`.

## Slam jump (Rick, 2026-09-23)

17 opposite 15-17 jumps to 6NT instead of making the quantitative 4NT:
6NT needs 32 total points combined with opener's minimum, 4NT is left
with 16. BBA does this on 10 of its 12 17-counts in Basic_*; at 16 it
invites 7 times of 11. Rick asked whether BBA judges 17 by controls:
at 16-17 its choice does not follow controls (6NT with 4 controls, 4NT
with 6), so the rule counts points only. Measured: +2,543 IMPs vs par on
the corpus, +77 on Basic_*.

## 4-3-3-3 is worth less? (Rick, 2026-09-23) — tried, not adopted

Rick: at the 2NT/3NT boundary, 4-3-3-3 is worse than 4-4-3-2 with the
same HCP and tens. Tried as a deduction from the notrump point count of
every 4-3-3-3 hand (engine `Valuation`): −½ point cost 2,543 IMPs vs par
on the corpus and 35 on Basic_*; −1 point cost 5,670 and 137. BBA does not
downgrade it either: after 1NT, with no four-card major, 10 HCP and no ten
bids 3NT on 43 of 44 4-3-3-3 hands but 19 of 33 4-4-3-2; at 8 HCP and one
ten it invites on 29 of 34 4-3-3-3 but 19 of 31 4-4-3-2 (at 8 HCP and no
ten the other way: 6 of 26 against 10 of 16). **Question for Rick**: a
narrower version (only responder's invite decision, only without tens) is
possible if he wants it tried.

## Length points (2026-09-23)

`points` now counts length (+1 a card beyond four). What changed here:

- **A long minor with invitational values** (7 HCP and six diamonds is 9
  total points) had no call once the natural 2NT required `C<=5, D<=5`:
  the relay is for weak hands only and the `none` treatment has no
  minor route. With `relay` or `none`, such a hand now bids the natural
  2NT (`5.K73.QJ9542.J83`). Par cannot choose here — the corpus has no
  `relay` card, and on the `none` cards 2NT, 3NT and signing off all
  measured within 10 IMPs of each other — so this is the old design (the
  natural 2NT) restored for the styles that have nothing else.
- **Opener answers the quantitative 4NT** (there was no rule: opener
  passed every 4NT). 4NT shows exactly 16 opposite 15–17, and opener bids
  6NT with **32 combined**, the same line Rick drew for the direct 6NT: 16
  or more accepts. The same answer serves the 4NT over our 1NT after a 2♣
  overcall and over a 2NT opening (`sets ask=quant`). 4NT now also says
  `H<=4, S<=4`: with five in a major, transfer first and then 4NT
  (`jacoby-transfers.bid`). Corpus par **+1,303 IMPs**, Basic_* +43;
  33 combined instead gained 52 less on the corpus and 19 less on Basic_*.

**Opener's count.** Rick expected the 1NT opener not to count length in
his later decisions. Measured on the invitation answers (1NT–2NT, and
`asked invite(M)` after a transfer or a Stayman raise), against keeping
total points:

| opener's count when accepting | corpus par | Basic_* |
|---|---|---|
| total points (kept) | — | — |
| HCP + ½ per ten, no length | −1,269 | −97 |
| plain HCP | −2,881 | −166 |

So opener keeps total points. **Question for Rick**: is that acceptable,
or does he want the no-length count for opener regardless?

Also measured, not adopted: accepting 1NT–2NT with **16** (+7 IMPs on the
corpus, +29 on Basic_*, but BBA declines, Basic_NT board 51, and the gain
is within noise). With a *fit* opener does accept with 16
(`jacoby-transfers.notes.md`).

## BBA treatment (2026-09-24)

`general.style = bba` plays a probed model of BBA's natural responses to
1NT and its answer to 4NT. The default is unchanged (checked board by
board over the whole corpus).

**The corpus first.** Every `1NT P` decision with a balanced hand and no
four-card major (Basic-Bridge, 21GF-DEFAULT and 21GF-GIB, 15-17 each).
A logistic fit of BBA's call on the honours, spot cards and shape gives,
per honour relative to a king at 3: A 3.9, Q 1.9–2.0, J 0.9–1.0, ten
0.4, nine and eight 0; 4-3-3-3 +0.2 to +0.5 against 4-4-3-2 (BBA does
not downgrade it); thresholds 8.0 for an invitation and 10.0 for game.
So HCP + ½ per ten (the `bba` valuation) is BBA's centre. The rest is
not in any feature we have: the same HCP and tens split both ways, and
changing a king to an ace turned 3NT into 2NT once (below). There is no
vulnerability effect (every corpus board is love all; probes at all four
vulnerabilities agree), and the corpus is all matchpoints.

By shape, % of BBA's calls, all three cards:

| points | 4-3-3-3 | 4-4-3-2 (4-4 minors) | 5-3-3-2 |
|---|---|---|---|
| 7½ | P 100% | P 100% | P 95% |
| 8 | P 62% | 2NT 59% | 2NT 63% |
| 8½ | 2NT 83% | 2NT 61% | 2NT 79% |
| 9½ | 2NT 75% | 2NT 80% | 2NT 72% |
| 10 | 3NT 83% | 2NT 55% | 3NT 67% |

Two exceptions stand out and are modelled:

- **8 HCP 4-3-3-3 without a ten passes** (P 81% of 52; with one ten 2NT
  83% of 83; 7 HCP and two tens 2NT 82%).
- **With 4-4 in the minors the tens do not count**: 9 HCP and two tens
  2NT 64% (3NT 36%), three tens 2NT 61%; 10 HCP 3NT 59–88%; 8 HCP 2NT
  about 60% with none to two tens.

**Probes** (`rbb probe`, Basic-Bridge both sides, dealer S, responder
North after a forced `1NT Pass`; MP and IMP at love all, and IMP all
vulnerable; `probes/ntb`, `probes/nt1`):

| North | HCP+tens | MP | IMP (None = All) |
|---|---|---|---|
| 542.Q76.K82.Q862 (4-3-3-3) | 7 | P | P |
| T42.Q76.K82.Q862 | 7+1 | P | P |
| T42.QT6.K82.Q862 | 7+2 | 2NT | 2NT |
| 542.Q76.K82.K862 | 8 | **P** | P |
| T42.Q76.K82.K862 | 8+1 | 2NT | 2NT |
| T42.QT6.K82.K862 | 8+2 | 2NT | **3NT** |
| T42.QT6.KT8.K862 | 8+3 | 3NT | 3NT |
| 542.K76.K82.K862 | 9 | 2NT | 2NT |
| T42.K76.K82.K862 | 9+1 | 2NT | 2NT |
| T42.KT6.K82.K862 | 9+2 | 3NT | 3NT |
| 542.K76.K82.A862 | 10 | 3NT | 3NT |
| 54.Q76.K862.K862 (4-4 minors) | 8, 8+1..3 | P | P |
| 54.K76.K862.K862 | 9 | 2NT | **3NT** |
| T4.K76.K862.K862 | 9+1 | 2NT | **3NT** |
| T4.KT6.K862.K862 | 9+2 | 3NT | 3NT |
| 54.K76.K862.A862 | 10 | 2NT | 2NT |
| T4.KT6.K862.A862 | 10+2 | 2NT | **3NT** |
| 542.Q76.K8.Q8652 (5-3-3-2) | 7, 7+1 | P | P |
| T42.QT6.K8.Q8652 | 7+2 | 2NT | 2NT |
| 542.Q76.K8.K8652 | 8, 8+1 | 2NT | 2NT |
| T42.QT6.K8.K8652 | 8+2 | 2NT | **3NT** |
| 542.K76.K8.K8652 | 9, 9+1 | 2NT | **3NT** |
| 542.K76.A8.K8652 | 10 | 3NT | 3NT |
| 542.J7.AK82.8642 (4-4 minors) | 8 | P | **2NT** |
| JT3.T8.KJ73.A864 (4-4 minors) | 9+2 | 2NT | **3NT** |

Vulnerability changed nothing (all four tried on the first set).
**At IMPs BBA is bolder by about a point**: game from 9 in most shapes
(not 9 or 9+1 with 4-3-3-3). The 4-4-minor rows show the noise: 8 HCP
with three tens passes, 10 HCP with an ace (for a king) invites.

The slam zone (Basic-Bridge corpus, balanced, no major): 15 HCP with any
tens 3NT (22 of 22), 16 4NT 96% without a ten, 71% with one, 6NT 67%
with two; 17 6NT ~72%; 18+ 6NT. Opener facing 4NT: 15 passes (36 of
36), 16 passes 10 of 13 whatever the tens, 17 with a ten bids 6NT (4 of
4), 17 flat passed twice.

Opener facing 1NT–2NT (all three cards): 16 HCP P 95% with 0–1 tens,
3NT 51% with two; 17 3NT 83–90%. No change by scoring or vulnerability
(probe `AQ5.KJ72.A95.J83` with 0–2 tens, 15–17: accepts from 16+2 tens,
MP = IMP, None = All). Already what `bba` valuation gives, so not
touched.

**What the rules model** (`one-nt.bid`, rules under `when style is
bba`, the default ones now `when style is not bba`):

- 2NT: 8–9 total points (HCP + ½ per ten), 4-4 minors 8–9 HCP; not 8
  HCP 4-3-3-3 without a ten (`has(T,x)` in each suit).
- 3NT: 10–15 total points, 4-4 minors 10+ HCP. At IMPs one point lower
  (a matchpoints and an IMPs pair of rules).
- Pass: the complement, including the flat 8.
- 4NT needs 16 HCP as well as 16 points: 15 and three tens bids 3NT
  (Basic_NT 105, 400).
- Opener's answer to 4NT over his 1NT: **17 HCP** accepts, tens or not
  (`hcp>=33-partner.hcp.min`); over 2NT the default answer stays. A
  points version (17 counting tens) agreed on 48 of 57 corpus boards, the
  HCP version on 52.

**Measured** (whole corpus, calls agreeing at the decision, bba style,
this module with the Stayman changes that share `1NT P`): `1NT P` 73.1%
→ 79.8% (27,704 decisions; Basic-Bridge 87.5% → 92.1%), most of it
Stayman with 4-3-3-3 (`stayman.notes.md`); the 4-4-minors exception alone
+1 call corpus-wide, +5 on Basic-Bridge; `1NT P 4NT P` 66.7% → 91.2%.
Totals for all the notrump work are in `stayman.notes.md`.

**Where our default differs, and why.** The default counts length
(Rick, 2026-09-23) and invites flat 8-counts: par preferred it. The flat-8
pass and the minors exception are BBA's judgement, not measured as
defaults. The default accepts 4NT with 16 (Rick: 32 combined).

**Open question.** BBA's residual (same features, different calls) looks
like its own evaluation, possibly simulation. More features will not
close it.

## How BBA values a 9-count opposite 1NT: probing to the bottom (2026-09-24)

Auction: 1NT (15-17, Basic-Bridge) - Pass - ? Responder balanced or
semi-balanced, no four-card major, 9 HCP: Pass, 2NT or 3NT. Started from
Basic_NT board 2 (A5.K43.T764.QT65: BBA 3NT, our BBA style 2NT). Rick:
track it down to 100%, since it shows how BBA values hands generally.

Settled:
- **Deterministic in responder's 13 cards.** 400 hands, each bid with
  three different partners and opponents: the same call every time
  (probes/gen_hands.py, .rbb-cache). Partner's cards, the opponents' cards
  and vulnerability do not matter.
- **Low spot cards (7-2) do not matter**: a model with all 52 cards is no
  better than one with A-8 and lengths.
- **Suit identity matters.** 2,000 hands bid beside their mirror (diamonds
  and clubs exchanged): 198 of 2,000 pairs differ at MP, 361 at IMPs.
  E.g. CQ432 with D9875 bids 3NT, DQ432 with C9875 2NT
  (probes/nt-9count-queen-spots.toml).
- **Scoring matters.** At IMPs every 9-count 4-4-3-2 in the small grids
  bid 3NT and every 4-3-3-3 2NT; at MP the tens decide
  (probes/nt-9count-tens.toml).
- On 25,000 random 9-counts at MP (26% game): no ten 7% game, one ten
  15-16% (bare or honoured alike), two tens 55-79%.

What BBA's rule is not:
- not HCP + 1/2 per ten with a threshold (the corpus-best simple rule);
- not additive over suits: a model giving every suit holding (length and
  A-9, suit by suit) its own value fits 85%, and adding the exact shape
  does not help;
- not losing tricks, quick tricks, stoppers, short honours or spot
  intermediates (none adds anything to the fit).

A flexible model (gradient boosting on each suit's length and A-8) fits
94.5-95% and rises with data, so the rule is in those features but
combines suits non-additively: it behaves like several conditions
together (e.g. enough value *and* the suits the defence will attack held).
Readable fragments from trees: with two tens, at most seven minor cards,
a heart honour and at most one jack, 1,049 of 1,055 bid game; nines count
up; four-four in the minors counts down; honours in the majors count up.

Open, for Rick: which quantities to vary next. The grid tool makes each
question a spec file and half a second (probes/*.toml); gen_hands.py
makes bulk sets.

### Why a heart honour and a spade honour differ: what 1NT contains (2026-09-24)

The pass/2NT line (7 HCP = AQJ + two tens, every placement,
probes/nt-7count-placements-AQJTT.toml) depends on where the honours sit,
across suits, and the pattern follows suit *length*: in 3-3-4-3 the pass
cells involve the three-card clubs, in 3-3-3-4 the same cells with clubs
and diamonds exchanged. It is identical under Basic-Bridge, 21GF-DEFAULT,
bare 2/1 and bare SAYC cards and with the 1NT shape switches flipped: if
BBA values responder's honours against what 1NT is likely to hold, the
model is fixed, not rebuilt from the card.

Rick's explanation: responder's honours are worth what they are likely to
mesh with, and what opener holds depends on which hands open 1NT. Tested
on the opening itself (probes/open-15-5422-major-mirror.toml, 15 HCP
5-4-2-2 with a five-card minor and a four-card major, each hand beside its
spade/heart mirror, "1NT opening shape 5422" on): BBA opens 1NT with four
hearts when both doubletons are headed by the ace or king (65 of 65), with
one such doubleton a third of the time (61 of 183), never with neither --
and never with four spades (0 of 300): with spades opener rebids 1S over
1H; with hearts he has no good rebid over 1S. With the switch off every
hand opens its minor. So a 1NT opener holds four hearts more often than
four spades, and responder's heart and spade honours are not worth the
same.

Rick's reading (2026-09-24): BBA supports switching 5-4-2-2 1NT openings
off, but its responses were written for a 1NT that includes them --
otherwise there would be no suit priority at all. So the bba treatment
should value responder's honours against a 1NT that may hold 5-4-2-2 with
four hearts (not spades), whatever the card says.

### Are BBA's pass cells statistically right? (2026-09-24)

`rbb simulate probes/nt-7count-placements-AQJTT.toml --pool
probes/pools/bba-1nt-15-17-with-5422.txt --samples 60`: each responder hand
against 60 openers drawn from the hands BBA itself opens 1NT (5-4-2-2
included, in their natural frequency), opponents random, double dummy.
The 7 placements BBA passes average 7.61 notrump tricks (8+ 55.0%, 9+
18.6%); the 57 it raises 7.63 (56.7%, 19.8%). A difference of 0.02
tricks, inside the noise (about 0.07 a placement), and the ranking of
placements does not follow BBA's passes (HAQ... HCH, a pass cell, is
among the best). So measured double dummy against what its 1NT contains,
BBA's suit-dependent passes are not better bridge: a fixed heuristic, not
a statistical optimum. Not modelled in the bba treatment for now.

### BBA's count for pass vs 2NT (2026-09-24): implemented

`bba_nt_points` (engine, Facts): per shape, A/K/Q/J/ten weights, a charge
for a doubleton major without the ace or king, and a threshold, fitted
by logistic regression to 37,000 random 7-9 HCP responder hands BBA bid
(probes/gen_hands.py; Basic-Bridge, MP), and scaled so BBA invites from
8. The bba style's matchpoint pass and 2NT use it; IMPs, 3NT and the
default are unchanged.

What BBA counts, across shapes: A about 4, K 3, Q 1.8-2.0, J 0.7-0.95; a
ten 0.75 in 4-3-3-3, 0.3-0.4 with 3-2-4-4 or five clubs, about 0 with five
diamonds or 2-3-4-4; a doubleton spade without A or K costs about a point
in 2-3-4-4 (1/2 in 2-3-5-3), a doubleton heart without them about 1/2 in
3-2-3-5 and 0.7 in 2-2-4-5; the invitation starts at 6.9-8.25 by shape.

A first count fitted to the placement frames (a few honour sets each)
did not transfer to real hands and was dropped: frames are for finding
what matters, random hands for fitting values.

Measured, Basic-Bridge corpus: BBA's own pass/2NT choices predicted
94.6% (722/763) against 89.8% for HCP + 1/2 a ten from 8; in replay,
decisions after 1NT P involving pass or 2NT 72.3% -> 74.9% (the rest are
other boundaries: Stayman, 3NT). BBA style overall: Basic_* identical
auctions 44.5% -> 44.6%, uncontested same contract 69.2% -> 69.4%.

### BBA's count for 2NT vs 3NT (2026-09-24)

Same method: 20,000 more random 10-11 HCP responders (with the 7-9 HCP
sets, 57,000 hands), a count fitted per shape for 2NT vs 3NT, tested on
the Basic-Bridge corpus: 93.6% of BBA's 2NT/3NT choices against 92.3% for
HCP + 1/2 a ten from 10. The flat shapes add nothing (a ten ~0.47, game
from ~9.85: the simple rule); the gain is in the short-major shapes, with
the same charges as for the invitation -- a doubleton spade without A or
K about 3/4 of a point in 2-3-4-4, a doubleton heart about 0.8 in 3-2-3-5
and 2-2-4-5. At 10 HCP about 7% of hands bid 3D, a third option not
modelled. `bba_nt_game_points`, scaled so BBA bids game from 10, drives
the bba style's matchpoint 3NT and caps its 2NT.

Replay, Basic-Bridge boards, all responses after 1NT P: 92.1% before
either count, 92.7% with the invitation count, 93.6% with both;
decisions involving pass, 2NT or 3NT 89.4% -> 90.3% -> 91.8%. BBA style:
corpus identical auctions 18.2% -> 18.3%, Basic_* uncontested 63.4% ->
63.6%. Default unchanged.

### The counts at IMPs (2026-09-24)

The same 57,000 hands bid at IMPs, fitted per shape and tested on a
held-out fifth (the corpus is all matchpoints). Pass vs 2NT hardly moves
(95.5% held out, against 91.2% for the best simple threshold); 2NT vs 3NT
does: flat 4-3-3-3 hands still bid game from ~9.8, but with a five-card
minor or 4-4 minors game comes a point or more earlier (from ~7.5-8.7),
unless a doubleton major lacks the ace or king, which costs more than at
matchpoints (1.5 in 2-3-4-4, 1.65 in 3-2-3-5). 88.7% held out, against
83.8%. `bba_nt_imp_points` and `bba_nt_imp_game_points` drive the bba
style's IMP pass, 2NT and 3NT.

Checked end to end on 4,000 fresh 7-11 HCP responders (the engine's bba
style against BBA): MP 82.9% -> 89.0%, IMP 76.9% -> 87.9%. Known miss:
54.K76.K862.K862 (2-3-4-4) bids 3NT at IMPs, the count 2NT. Also fixed:
the matchpoint pass rule had lost its `priority -10` in 89d5f9b (no
effect on the corpus).

## After the invitation is declined in the fit (2026-09-25)

1NT-2♣-2♥-2NT-3♠: opener declines in the spade fit, and responder had
no rule in the default (only the bba style had one, which bids game with
the top of the invitation). The default now passes three of the major
after a decline. BBA passed on 2 of the 3 Basic_* boards and bid 4♠ on
Basic_Openers_Rebid 218 (8 HCP).

## Opener after 1NT-3m, the natural slam try (2026-09-25)

1NT-3♣/3♦ (six of the minor, slam-invite values, when the card plays
them natural) had no answer. Opener now bids six with three-card support
and 17, else 3NT. Basic_What_To_Open 320 (the only Basic_* board) +1;
full corpus +660 by par distance and +2,375 to the side that changed,
over 248 boards, almost all 1NT-3♦ on cards outside Basic_*.

## Slam in a minor over 1NT (2026-09-27)

First step of the slam work. Over the corpus BBA bids 22,939 slams and
we bid 9,905; on the 14,834 boards where only BBA bids one it makes 78%
of the time double dummy. The largest group of those (544 boards) was
1NT-3NT: responder with 15-18 HCP and a five-card minor, mostly on
21GF-MSTandMSS (Minor Suit Stayman, which we do not play), where the
only slam route was the natural 3m, and that needed six cards.
Calibration from every double-dummy table in the corpus: a small slam
makes 72% of the time with a nine-card fit at 29 HCP between the hands,
71% with an eight-card fit at 30, and 6NT 59% at 31 with no fit.

Now: 3♣/3♦ is a slam try with six cards or five in an unbalanced hand
and slam-invite values; opener bids six with a fit and a maximum, four
of the minor with a fit and a minimum (forcing; responder bids six from
17 total points, else five), 3NT without a fit (responder bids 6NT with
slam values, else passes). Full corpus +1,943 by par distance, +3,630 by
side (481 boards).

## Sources

- **The system:** a 15-17 1NT opening and natural responses (2NT
  invites, 3NT, quantitative 4NT, 6NT), as BBA plays them on 21GF-DEFAULT
  and Basic-Bridge. The point bands are standard practice, not yet cited
  to a book or article.
- **Rick's rulings:** "we do not model BBA's finer valuation: the aim is
  defensible bids" (2026-09-21); responder counts length points and the
  NT opener usually does not (2026-09-23); the 6NT jump with 17 (32
  combined, 2026-09-23); 4-3-3-3 downgrade tried at his request and not
  adopted (2026-09-23); track BBA's 9-count valuation "down to 100%", and
  his reading that BBA's responses value honours against a 1NT that may
  hold 5-4-2-2 with four hearts (2026-09-24).
- **BBA probes:** `rbb probe --vary-tens` command lines in "Evidence from
  BBA"; the 2026-09-24 runs `probes/ntb` and `probes/nt1` (sandbox
  outputs); `rbb grid` specs `probes/nt-9count-*.toml`,
  `probes/nt-7count-placements-AQJTT.toml` and
  `probes/open-15-5422-major-mirror.toml`; random hands from
  `probes/gen_hands.py`; `rbb simulate` with
  `probes/pools/bba-1nt-15-17-with-5422.txt`.
- **Corpus measurements:** the slam jump (+2,543), length points, opener's
  count, the minor slam tries (2026-09-25, 2026-09-27) and the
  double-dummy slam calibration.
- **Where we differ:** BBA's finer valuation, flat 8-counts, tens with a
  five-card minor, and 16-17 opposite 15-17 ("Accepted differences from
  BBA").
