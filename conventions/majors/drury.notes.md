# drury (`drury.bid`): notes

Drury after partner's third- or fourth-seat 1♥/1♠: a passed hand's 2♣
is artificial, a fit and limit-raise values. Cases: `drury.test`.

## What the card says

| Field | `.bbsa` key | Meaning here |
|---|---|---|
| `major_openings.drury.play` | `Drury` | the original: opener's 2♦ is the sub-minimum, 2M a full opening |
| `major_openings.drury.reverse` | `Reverse drury` | Reverse: 2M is the sub-minimum, 2♦ a full opening |
| `major_openings.drury.two_d` | none | two-way: 2♣ exactly three trumps, 2♦ four or more |
| `major_openings.drury.in_comp` | none | 2♣ is still Drury over a double or a one-level overcall |

Either of the first two switches the convention on (21GF-DEFAULT sets
only `Reverse drury`, 21GF-GIB only `Drury`); with both, Reverse. Rick's
card (Bridge-Classroom) sets `drury.play` and `drury.reverse`: Reverse
Drury. Bridge-Classroom's catalog describes Reverse Drury as "3-step
responses with reverse strength"; we play the two-step form below.

**Which is which.** In the original Drury opener's 2♦ is the negative
(a light opening) and 2M the normal one; Reverse Drury swaps them
(Bridgebum, Wikipedia). BBA plays exactly that: on 21GF-DEFAULT (Reverse)
its 2♥ is "minimum, 11-12 total points" and 2♦ "artificial, 12-14"; on
21GF-GIB (original) 2♦ carries the 10-11 counts and 2♥ the 12-14 ones
(`probes/drury-opener-21GF-DEFAULT.toml`, `-GIB.toml`).

## The structure

- **Responder** (a passed hand): 2♣ with three or more trumps and 10-12
  support points. It replaces the passed hand's jump to 3M
  (`responses.bid` no longer offers the limit raise, the natural 2♣, or
  the game-values 2♣/2♦ to a passed hand when Drury is on). 2♣ sets the
  trump suit and asks (`ask=drury(M)`), and is forcing for a round.
- **Opener**, counting support points (`tp`, HCP and shortness):
  12 or fewer, the sub-minimum (2M Reverse, 2♦ original); 13-14, a full
  opening (2♦ Reverse, 2M original); 15+, game.
- **Responder again**: passes the sub-minimum (or converts the original's
  2♦ to 2M); facing a full opening bids game with 12, or 11 and four
  trumps, else signs off.
- **Two-way**: 2♣ promises exactly three, 2♦ four or more; after 2♦
  opener bids game from 13 (the ninth trump), else 2M.

## Evidence

**BBA's responder** (`probes/drury-resp-1H.toml`, `-1S.toml`, 800 hands
each, 21GF-DEFAULT): 2♣ is "9 to 11 total points, 3+ trumps". It starts at
9 HCP with shortness, 9-10 flat; with 5+ trumps and 6-8 it jumps to 4M
(preemptive); over 1♥ with four spades and 10-11 it sometimes bids 1♠;
with 11 and shortness it splinters (a passed hand's splinter, three or
four trumps). Our agreement: 583/800 (1♥), 572/800 (1♠).

**Where 2♣ starts.** The first version took the limit raise's own
range (11-12, or 10 with four trumps and shortness). Starting at 10
support points gained, 9 lost (the Drury corpus subset: Drury,
Gavin_Passed_Hand_Response_Structure, After_2/3_Passes,
Open_In_Fourth_*, Bergen_Raises):

| 2♣ from | vs BBA |
|---|---|
| 11 (the limit raise) | −1,845 |
| **10** | **−1,719** |
| 9 | −1,753 |

**Opener's bands.** Tried on the same subset: one point lower (game from
14) −1,809; one point higher (game from 16) −1,749; the same bands on
total points instead of support points −1,746, and a point lower −1,769.
The support-point bands (12 / 13-14 / 15) stay: −1,719.

BBA's opener counts no shortness, so our agreement on its opener probe
is low (252/1200 on 21GF-DEFAULT): a 10-count with a singleton is 13
support points and a full opening here, and 15+ go straight to game
where BBA bids 2NT, new suits or its own splinters. Par prefers ours.

**Corpus, the whole change** (with splinters, same commit): Drury
−100 → +153, Gavin_Passed_Hand_Response_Structure −36 → +140,
Open_In_Fourth_13/14/16 +49/+33/+40, After_2_Passes −15.

## Accepted differences from BBA

- 2♣ from 10 support points (BBA 9-11 total points, counting no
  shortness); with 9 we raise to 2M.
- Opener's answers count shortness; BBA's do not.
- Opener with 15+ bids game; BBA shows a second suit, a splinter, 2NT
  (14-16) or 3NT (16-18) first. Slam opposite a passed hand's limit
  raise is rare; the control bids still run over 4M when the values are
  there.
- A passed hand's splinter needs four trumps (splinters.bid); BBA
  splinters with three at 11.

## Gaps

- Drury after 2♣ is doubled or overcalled (Bridgebum: off; "stolen bid"
  optional): opener's answers need RHO to pass, so over interference
  opener has only natural calls.
- `in_comp` and `two_d` have no `.bbsa` key and no corpus card sets them:
  they are unmeasured, tested only by `drury.test`.
- Opener's 2♠ over 1♥–2♣ (four spades, which BBA shows with 15+) is not
  a Drury answer here.

## Sources

- **Convention descriptions**: Bridgebum, "Drury"
  (bridgebum.com/drury.php: 2♣ = 3+ support, 10-12 support points;
  original 2♦ sub-minimum, 2M normal; Reverse swaps them; applies over a
  double or overcall when 2♣ is available); Wikipedia, "Drury convention"
  (two-way: 2♣ three trumps, 2♦ four; opener may jump to game with 15+).
  We differ from both in counting opener's support points (with
  shortness) for the bands.
- **Bridge-Classroom** `conventionCatalog.js`: the fields `drury.play`
  (2♣), `drury.two_d` (2♦), `drury.in_comp`, `drury.reverse`.
- **BBA probes**: `probes/drury-resp-1H.toml`, `probes/drury-resp-1S.toml`
  (2026-09-28), `probes/drury-opener-21GF-DEFAULT.toml`,
  `probes/drury-opener-21GF-GIB.toml` (2026-09-25, rerun 2026-09-28).
- **Corpus measurements** (2026-09-28): where 2♣ starts and opener's
  bands, the tables above. The earlier trial (responses.notes.md,
  "Drury, tried", 2026-09-25) lost to the passed-hand limit raise; it
  had no answers above 12-14 and no responder follow-ups.
- **Rick's rulings**: none specific to Drury yet; the support-point
  count is his (responses.notes.md, 2026-09-23).
