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
