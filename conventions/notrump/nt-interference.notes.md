# nt-interference (`nt-interference.bid`): notes

Responder when RHO doubles our 1NT or overcalls 2♣. Cases:
`nt-interference.test`.

This module holds only the calls that exist *because* they bid — the
redouble, the natural escapes, the pass. Stayman, the transfers, Texas
and the natural notrump calls keep their own calls and live in their own
modules, in `after 1N (X)` and `after 1N (2C)` contexts next to the
uncontested ones (`stayman.bid`, `jacoby-transfers.bid`,
`minor-transfers.bid`, `texas-transfers.bid`, `one-nt.bid`).

## The two card switches

`notrump.transfers.vs_double` ("Transfers if RHO doubles") and
`notrump.transfers.vs_2c` ("Transfers if RHO bids clubs") were both in
the coverage report's *ignored* list: every notrump pattern we had was
`after 1N (P)` or a competitive spelling of it, so a double or a 2♣
overcall switched the whole structure off and responder passed. On the
corpus that was 1,331 decisions after a double and 1,243 after 2♣.

The switches really do change the system, and the corpus lets both
halves be measured: **21GF-DEFAULT** (199 of 342 scenarios) has
`vs_double = 0` and `vs_2c = 1`; **21GF-GIB** and **Basic-Bridge** have
both on.

Note that `after A | B` takes one `when` for the whole line, so the
double and the 2♣ overcall are written as two contexts rather than two
alternatives. That is also what keeps the uncontested
`after 1N (P) | (1x) 1N (P) | …` line — with its
`when !they.bid | systems_on` — untouched.

## Over a double

**Transfers on** (`vs_double`). The 1NT structure is unchanged and
responder is very active. From a 150-board `rbb probe --prefix "1NT X"`
on Basic-Bridge: 2♣ Stayman 42, redouble 48, 2♦/2♥ transfer 35, pass 12.

- 2♣ is **Stayman one point below the uncontested floor** — 7 opposite
  15-17, written `hcp>=24-partner.hcp.max` so it follows the card.
  Finding the 4-4 fit is also somewhere to run to.
- Stayman **denies a five-card major** here, unlike the uncontested 2♣:
  with 5-4 BBA transfers rather than looking for the other major first,
  and with 5-5 it transfers to **spades**. (Uncontested, Stayman keeps
  priority with 5-4 for Smolen.)
- **XX is 6+ HCP with nothing else to say** — partner has 15-17 and they
  have doubled into it, so the hand is ours. BBA passes 0-5 and
  redoubles from 6.

Agreement on the probe: **142/150**.

**Transfers off.** A different game: BBA passes 170 of 252 decisions in
`Opps_Double_1_NT` (21GF-DEFAULT). The two-level calls are natural
escapes — 2♥/2♠ with five, 2♣/2♦ with six — and the cut is **four HCP**:

| escape cap | decisions matched (of 252) |
|---|---|
| 3 | 193 |
| **4** | **205** |
| 5 | 186 |
| 6 | 168 |
| pass everything (before) | 170 |

**There is no redouble in this branch.** BBA redoubles 10 times in 252,
on 6-9 counts that are indistinguishable from the fifty 6-9 counts it
passes, and a `hcp>=6` redouble cost 77 boards on its own (205 → 138).
Passing them is both closer to BBA and the safer bid.

## Over a 2♣ overcall

One structure on every corpus card that sets `vs_2c`, and it is simply
"systems on with the double standing in for 2♣":

- **X is Stayman**, 8+ with a four-card major and no five-card one.
- **2♦ / 2♥ are the transfers**, with a **five-HCP floor**: below that
  BBA leaves their 2♣ alone rather than push to the two level with
  nothing. (Over a double there is no floor — running is the point.)
- **2♠ / 3♣ are the minor transfers**, same floor, same treatment field.
- **2NT is the natural invitation and 3NT the natural game**; 2♣ takes
  neither away.
- **Texas is on**, and over interference it doubles as a preempt: with
  **seven** cards in a major and no values BBA transfers straight to
  game rather than to the two level (`A9.JT87432.2.T65` → 4♦).

Agreement at `1N (2C)`: **45.8% → 87.1%** (571 → 1,086 of 1,247).

## Accepted differences from BBA

- **The 2♣ escape with a long minor over a double, transfers on.** With
  2♣ Stayman and 2♦ a transfer there is nowhere to run, and BBA passes.
  So do we. A card with minor transfers on would want `2♠`/`3♣` here;
  not written.
- **Stayman over 2♣ with game values.** BBA's X is 8-9 only: with 10+
  and a four-card major it bids 3NT and never looks for the 4-4 fit.
  We keep `strength>=invite`, so we double with game values too. It
  costs 11 boards of agreement and looks like better bridge; flagged
  for Rick.
- **Super-accepts are not written for these auctions.** Opener simply
  completes the transfer.
- **BBA's 2NT over a double** (8-9 balanced, `vs_double` off) is bid on
  two boards and passed on fifty like it; not written.

## Measured effect (2026-09-23)

At the two decisions: `1N (X)` 59.8% → 76.3% (798 → 1,018 of 1,334),
`1N (2C)` 45.8% → 87.1% (571 → 1,086 of 1,247). The scenarios that
moved most are Lebensohl (+212 calls), Suction (+155), Cappelletti
(+148), Mitchell_Stayman (+118 calls, +71 contracts), Opps_Double_1_NT
(+105) and Exit_Transfers (+105) — all of them scenarios built around
an action over 1NT.

## Gaps and open questions

- **Opener after the interference.** `1N (X) XX (2y)` and the rest of
  the auctions where they bid again have no rules: opener passes.
- **Lebensohl** (`notrump.lebensohl.over_interference`) is on for
  21GF-DEFAULT and is not built. That is the structure over a 2♦ or
  higher overcall of our 1NT, which is untouched here: only 2♣ and the
  double are covered, because those are the two the card switches name.
- **Runout / exit transfers over the double.** The remaining
  disagreements at `1N (X)` are dominated by cards that play something
  else again: "BBA 2♠, ours P" 56 boards and "BBA 3♣, ours P" 46 come
  from the Exit_Transfers and Runout_after_1N_X scenarios, which are
  treatments we have no field for.
- **Their double is not interpreted.** We have no rule for a double of
  a 1NT opening, so the engine does not know it shows 15+ (BBA calls it
  "Cappelletti, strong" on these cards). The redouble's 6-HCP floor is
  therefore a flat number rather than a count of the deck.

## Length points (2026-09-23)

- `AT9.J6.K2.976532` over their 2♣ (Cappelletti, any six-card suit;
  Opps_Double_1_NT) is now 8 HCP + ½ + 2 = 10½, game, and bids 3NT; BBA
  transfers to clubs (2♠). Accepted: by Rick's count it is a game hand.
  On that one board 3NT takes 7 tricks double-dummy, and BBA's auction
  ended in 3♦X by West, one down — a single board, not evidence either
  way. The minor transfer case moved to
  `AT9.J6.Q2.976532` (9½).
- The Texas preempt over 2♣ (`A9.JT87432.2.T65`) now reads
  `suit_strength=signoff`, see `texas-transfers.notes.md`.

## Their natural 2♦/2♥/2♠ overcall (2026-09-25)

Since our East-West overcall 1NT naturally (overcalls.bid, "Over their
1NT, natural"), boards that BBA bid N/S alone now reach 1NT-(2x), and
three seats had no rule: responder, the advancer, and opener in the
pass-out seat (20 of the 110 remaining no-rule positions in Basic_* N/S).

**Decision: add simple natural rules** rather than leave the seats
empty. BBA on the Basic card (`probes/nt-2D-resp.toml`,
`nt-2H-resp.toml`, `nt-2S-resp.toml`, 250 random responders each):

| call | BBA's meaning (its own `--all-meanings`) |
|---|---|
| X | negative: 5+, four cards in an unbid major (over 2♠ 4-6 hearts) |
| 2♥/2♠ | natural, to play, 4-9 |
| 3♣/3♦/3♥ (non-jump) | natural, 5+, 9-13, forcing |
| 2NT | 8-9 |
| 3NT | 9-15, no stopper required (3NT on a diamond void over 2♦) |
| 4♥/4♠ | six cards, game |
| Pass | 0-8 |

Our rules follow that, in HCP where BBA's boundaries are HCP-like; with
a five-card major and game values over 2♦/2♥ we bid 3NT as BBA does
rather than jump. Agreement on the grids: 83-87%; most of the rest is
2NT or 3NT with 9. Opener answers the double with a four-card major,
2NT with a stopper, a pass with four of their suit, or a four-card
minor; responder then bids game with 10+, invites with 8-9. Opener
raises a three-level major with three, else 3NT; passes a two-level
suit; accepts 2NT with 16-17.

The advancer (partner of the 1NT overcaller) passes. Competitive raises
(three-card support, 6-10) were tried: -26 IMPs to the advancing side
over the corpus, par distance neutral, so they were dropped.

**Measured**, both yardsticks, as the competitive-yardstick rule asks:
full corpus +188 IMPs by par distance and +170 to the side that changed
its call (74 boards); Basic_* N/S flat (8 boards, -4 / -6). No-rule
positions in Basic_* N/S 110 -> 87.

## Their preempt over our 1NT (2026-09-25)

1NT (3x) had no rules (Basic_NT and Basic_What_To_Open, 7 positions
with the follow-ups). Now: 3NT with 9+ and a stopper, four of a five-
card major with 9+, a takeout double with 9+ short in their suit (opener
defends with four of theirs, else 3NT with a stopper or a four-card
suit), otherwise pass. BBA's calls in the corpus: pass, 3NT and one
double. Full corpus: 1NT (3♦) +105, (3♣) +90, (3♥) +80 by side, part of
a +189 / +262 change with the fallbacks below.

## Their conventional defences (2026-09-28)

The opponents' calls over our 1NT are now read with their card's defence
(`competitive.vs_1nt_strong.system`: cappelletti.bid, dont.bid,
meckwell.bid, multi-landy.bid; vs-1nt.notes.md). 21GF-GIB, the EW card
of 316 scenarios, plays Cappelletti, so from now on EW double 1NT with
15+, bid 2♣ with a one-suiter, 2♦ with the majors and so on, where
before they passed every hand. What changed here:

- **The natural-overcall rules** (`after 1N (2y)` and their
  continuations) now apply only when the overcaller has shown four or
  more of the suit bid and not both majors: `rho.y>=4, (maybe
  rho.H<=3) | (maybe rho.S<=3)` (`lho.` from opener's seat). Cappelletti
  2♥/2♠ (five of the major and a minor), Meckwell 2♥/2♠, DONT 2♦ and 2♠
  pass; Cappelletti 2♦ and DONT 2♥ (both majors) do not.
- **Both majors shown** (Cappelletti 2♦, DONT 2♥): X shows 8+ HCP and
  says we can defend; 3♣/3♦ natural and forcing to game (opener bids
  3NT); 3NT with both majors held. When they run, a double is for
  penalties (four of the suit), and opener leaves a penalty double in.
- **An unknown major** (Multi-Landy 2♦): X values, 3NT game.
- **Their 2NT** (minors): X values; 3♥/3♠ natural and forcing, four of
  a major with six; 3NT with 10+; opener raises a major with three.
- **Their 2♣ relay over our Stayman double** (Cappelletti 2♣ X 2♦):
  opener still shows a major, doubles 2♦ with four and no major.
- **Their penalty double**: opener passes partner's escape
  (`vs_double` off), passes a redouble left in, and after our redouble
  and their run leaves it to responder, who doubles with four of their
  suit or 9+.
- After a transfer they interrupted, a responder with no continuation
  (5-4 in the majors with a singleton, which goes through Stayman
  uncontested) passes: a fallback, not a treatment.

These took the no-rule positions they had caused (821 more at the first
run) back below the baseline (2,835 before, 2,795 after). Still open:
Lebensohl (on for 21GF-DEFAULT) over their two-level calls; a cue-bid
structure over Landy/Multi-Landy 2♣ (we keep Stayman by double);
opener's rebids when their advancer bids over our double (a pass).

## Their natural 2♣, transfers off (2026-09-30)

With `notrump.transfers.vs_2c` off (the vanilla SAYC card; no stock
corpus card) nothing applied after a natural 2♣: the 2♦/2♥/2♠ rules are
gated `y is not C` and the 2♣ systems-on rules need the switch. On the
95,558 vanilla SAYC deals that was 156 no-rule boards at `1NT (2C)` and
54 more at `1NT (2C) 2NT (P)`.

The rules apply only when the 2♣ showed five or more clubs
(`rho.C>=5`): DONT and Meckwell 2♣ promise only four, Cappelletti's 2♣
an unknown suit, Landy the majors, so their 2♣ is untouched (a test
holds this for Cappelletti on 21GF-DEFAULT).

**What BBA plays** (bare SAYC, `probes/nt-2C-nat-resp.toml`, 400 random
responders, with BBA's own meanings from the grid's `bba_alert`):

| call | BBA |
|---|---|
| X | never bid (0 of 400; 0 of 270 in the vanilla corpus): no negative double |
| 2♦/2♥/2♠ | five or more, "4 to 9 total points", to play |
| 2NT | 8-9, a club stopper |
| 3NT | 9-15, a club stopper |
| 3♦ | five or more diamonds, 9-13 |
| 4♥/4♠ | six or more, "6 to 13 total points" |
| 3♣ | artificial, two 16-counts |
| Pass | 0-9, but also 10-16 counts with no club stopper (QT.Q862.AKQ8.Q95, 15, passes) and long clubs |

Opener (vanilla corpus): after responder's pass he **reopens with a
double holding two clubs** (32 of 32 two-club hands; never with three),
and responder passes it with four or more clubs or names a four-card
suit. After 2NT he bids 3NT with 16 (2 of 2) and passes 15 (4 of 4).
After a two-level suit he passes; after 3♦ he bids 3NT.

**Ours.** BBA's low end, as it is: two-level suits from 4 HCP and at
most 9 points, 2NT 8-9 with a club stopper, 3NT from 10 with one, a weak
hand with a four-card major passes and relies on the reopening double,
and the reopening double itself. BBA's high end we do not copy: it has
no forcing call short of game, so it passes or bids a non-forcing 2♠
with 13-16 counts. Ours, standard practice rather than BBA:

- a five-card suit at the three level forces to game (3♦ non-jump, 3♥/3♠
  jumps); opener raises a major with three, else 3NT;
- **3♣ cue-bid: Stayman, forcing to game**, with a four-card major or
  without a club stopper. Opener shows a four-card major (hearts first),
  else 3NT with a club stopper, else 3♦; responder raises a fit, shows
  four spades over 3♥, else 3NT;
- four of a major with six cards and 9+ points (BBA from about 6-7 HCP;
  our cut is a judgment call, between BBA's and the natural block's 10);
- when they raise clubs over a two-level call opener competes in a major
  with four and accepts 2NT with 16-17; otherwise passes.

**Measured** (baseline main 16e3f68, same binary):

| set | vs BBA before | after | no-rule before | after |
|---|---|---|---|---|
| vanilla SAYC (95,558) | -42,209 | -42,106 (+103) | 1,641 | 1,425 |
| PBS corpus | -93,855 | -93,855 | 2,607 | 2,607 |
| 21GF random | -43,948 | -43,948 | 1,378 | 1,378 |

Calls agreeing on vanilla 798,209 → 798,399. The corpus and the 21GF set
do not move: every card there keeps `vs_2c` on or plays an artificial
2♣. `sideimps.py` on the vanilla runs: 266 boards changed, **+548 IMPs
to the side that changed its call**, +103 by par distance. By call:

| change | boards | side IMPs | par distance |
|---|---|---|---|
| opener's reopening double (P → X) | 53 | +172 | -90 |
| responder's 2♦/2♥/2♠ (from pass, 2NT or a 3-level preempt) | 136 | +190 | +12 |
| 2NT / 3NT / 3♣ / three-level suits / 4M | 66 | +182 | +168 |
| opener accepts 2NT (P → 3NT) | 11 | +4 | +13 |

The yardsticks disagree on the **reopening double** (and, mildly, on
the two-level diamonds: +58 side, -45 par): it often ends in 2♣ doubled
beaten by more than our par contract, or pushes them, which par distance
counts as a loss. It is the competitive call the side yardstick exists
for, and BBA's call. **Rick, 2026-09-30: keep it.**

Still open: a weak six-card major (5-6 HCP) bids 2♥/2♠ where BBA jumps
to game; a five-card major with 9 HCP and 10 points forces with 3♥ where
BBA bids 3NT (5 boards, -9).


## The answer to 2NT outranks one-nt.bid's (2026-10-03)

After 1NT (2♣/2♦/...) 2NT, opener's "Accepts: a maximum" (16+) and
"Declines: a minimum" (15-) here and one-nt.bid's general answer to
`asked nt_invite` (by strength) are candidates for the same calls at
the same priority, so descriptiveness chose between them. With the
engine's narrower knowledge of opener (2026-10-03) the general Pass
measured 0.445 against this 3NT's 0.435 and 16 HCP declined (the test
at 1NT (2C) 2NT). These pairs are now priority 1: the specific answer
to the invitation over their interference comes first.

## Sources

- **The structure over interference:** standard practice, not yet cited
  (Stayman by double and transfers over 2♣ follow the card's switches,
  BBA's play on every corpus card that sets them). Over a natural 2♣
  with the switch off: natural two-level suits to play, the cue-bid as
  game-forcing Stayman, three-level suits forcing: standard practice,
  not yet cited; the reopening double with shortness in their suit is
  BBA's (vanilla corpus).
- **BBA probes:** `rbb probe --prefix "1NT X"` on Basic-Bridge (not kept);
  `probes/nt-2D-resp.toml`, `nt-2H-resp.toml`, `nt-2S-resp.toml` (natural
  2♦/2♥/2♠, Basic-Bridge); `probes/nt-2C-nat-resp.toml` (natural 2♣,
  bare SAYC), meanings from the grid's `bba_alert`.
- **Corpus measurements:** Opps_Double_1_NT (escape cut), the full corpus
  (2026-09-23, -25, -28), the vanilla SAYC random set (2026-09-30).
- **Where we differ from BBA:** Stayman over 2♣ with game values; game
  forcing calls over a natural 2♣ where BBA passes or bids 2♠; 4♥/4♠ with
  six from 9 points ("Accepted differences", the 2026-09-30 section).
- **2026-10-03:** priority of the 2NT answers, nt-interference.test
  (1NT 2C 2NT P, transfers off).
