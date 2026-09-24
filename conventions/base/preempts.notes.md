# preempts (`preempts.bid`) and the opening rules: notes

Weak two-bids, three- and four-level preempts, and the seat rules in
`base.bid`. Cases: `preempts.test`.

## Guidance (Rick, 2026-09-22)

- **A weak two is 4-9 HCP with a six-card suit.** Vulnerable it needs two
  of the top three or three of the top five honours.
- **Third seat is looser**: a five-card suit will do (we ask for a good one:
  BBA passes those hands, so an unqualified five-card weak two costs about
  200 calls).
- **Fourth seat has no weak two**: two of a suit there shows a six-card
  suit and the values of a minimum opening.
- **With a seven-card suit, open at the three level**, not the two.
- **Third seat may open one of a good four-card major** with 10-11.
- **Fourth seat opens by the rule of 15**: HCP plus spade length, 15+.
- **A one-level opening needs 12 total points** — HCP plus one for each
  card beyond four in every suit — not 12 HCP, so nothing falls between a
  weak two and an opening bid.
- The same guidelines apply to a weak jump overcall (not written yet).

## Evidence

Basic_Weak_2: calls 61.9% → 76.0%, identical auctions 0.2% → 13.2%.
Basic_What_To_Open: 68.4% → 76.6%. Whole corpus: calls agreeing 70.3% →
71.9%, first-call divergences 37,868 → 15,699.

**The rule of 20 was measured** as an extra requirement for hands without a
six-card suit (it rules out 11 HCP with 5-3-3-2). BBA opens those hands far
more often than it passes them: 2,939 calls lost against 225 gained, so the
12 total points stand alone.

**Rick, 2026-09-23: a major weak two goes up to 10 HCP** (preemptive
value), where a 10-count with six cards used to open at the one level (12
total points). BBA does the same (about 70 Basic_Weak_2 boards). Measured
against par it costs: −243 IMPs on the corpus over 1,106 changed boards,
−42 on Basic_*. Most of the first measurement (−515) was a bug it exposed:
"no feature" over the 2NT ask was capped at 9 HCP, so a 10-count passed a
forcing 2NT (fixed in weak-two-responses.bid). What remains is the
preempt's own price: partner passes 2M with hands that would reach game
after 1M, and par does not reward the preemption here. Kept as Rick's
ruling; the trade is his to revisit. Diamonds stay 4-9 (par favoured our
1♦ on those boards, +29).

**A side four-card major** (Rick: modern style allows it, depending on
the relative suit quality) was tried as "the six-card suit holds more of
the top five honours than the major": −51 IMPs on the corpus, −26 on
Basic (−75/−33 with the 2NT fix). Not adopted; **question for Rick**
whether a different quality test is worth trying.

## Accepted differences from BBA

- BBA opens weak twos and three-level preempts on hands outside these
  rules: 10 HCP diamond weak twos, weak twos with a side four-card
  major, and 3♦ on a six-card suit.
- BBA opens 11 HCP with 5-3-3-2 where the rule of 20 says pass; we open
  (12 total points).
- Fourth seat: BBA opens some hands the rule of 15 passes.

## Gaps (not built yet)

- Weak jump overcalls and preempts in competition; defences to preempts.
- Responder's continuations after a preempt above the two level.
- Lebensohl over a weak two (card field exists).

## Treatments (2026-09-24)

`general.style = bba`: every weak two goes to 10 HCP, and a side
four-card major does not stop one (Basic_Weak_2: BBA opened about 110
ten-counts with six cards at the two level, and 15 weak twos held a side
four-card major, junk ones included). Refined by the probes below: a side
major stops the weak two when it holds the jack or better. With the BBA style it adds 0.2
points of identical auctions on Basic_* and costs 0.1 of same contract.
Our default keeps diamonds to 4-9 and no side major, both on par (+29 for
1♦ on the ten-counts; the side major cost 75).

## BBA treatment (2026-09-24): ten-counts, side majors, six-four minors

What BBA opens with a six-card suit in first or second seat, Basic-Bridge
card both sides. Corpus first (Basic_* boards, the opener's hand against
BBA's call), then `rbb probe` with the South hand fixed, dealer South,
one feature changed at a time. BBA's opening does not depend on the
other hands, so one layout per hand is enough; every probe was repeated
at love all and vulnerable where vulnerability could matter.

**Corpus, ten HCP and a six-card suit** (seats 1-2): diamonds 1♦ 24,
2♦ 42, pass 3; hearts 1♥ 45, 2♥ 35, pass 1; spades 1♠ 36, 2♠ 35, pass
3; clubs pass 3.
At 9 HCP it is a weak two (116 of 129, the rest passes), at 11 always
one of a suit (19 of 19).

**Majors, 10 HCP, 2-6-3-2 or 2-6-2-3** (probes, same result at love all
and all vulnerable):

| South | feature | BBA |
|---|---|---|
| Q2.AKJ862.T95.93 | ♥AK | 1♥ |
| J2.AKQ862.T95.93 | ♥AKQ | 1♥ |
| Q2.AK8762.J95.93 | ♥AK | 1♥ |
| K2.AK8762.T95.93 | ♥AK | 1♥ |
| 92.AK8762.QJ5.93 | ♥AK | 1♥ |
| 92.AK8762.K95.93 | ♥AK | 1♥ |
| A2.AQ8762.T95.93 | two aces | 1♥ |
| 92.AQ8762.A95.93 | two aces | 1♥ |
| 92.AJ8762.AJ5.93 | two aces | 1♥ |
| K2.AQJ862.T95.93 | ♥AQ, side K | 2♥ |
| A2.KQJ862.T95.93 | side A, ♥KQ | 2♥ |
| A2.K98762.K95.93 | A K K | 2♥ |
| 92.A98762.K95.K3 | A K K | 2♥ |
| Q2.K98762.A95.J3 | one ace | 2♥ |
| AK2.QJ8762.95.93 | side AK | 2♥ |
| 92.QJ8762.AK5.93 | side AK | 2♥ |
| 92.AK8762.Q95.93 (9) | ♥AK, 9 HCP | 2♥ |
| AKQ762.92.95.932 (9) | ♠AKQ, 9 HCP | 2♠ |

A ten-count opens one of a major when the suit holds the ace-king or the
hand holds two aces; otherwise it is a weak two. Controls do not decide
it (A K K is 2♥), nor does a side ace-king.

**Diamonds, 10 HCP**: 2♦ with ♦AKJ, ♦AQJ, ♦AKQ, ♦AKQJ, ♦AK and a side
king, A K K (92.Q2.AKJ862.T93, 92.K2.AQJ862.T93, 92.J2.AKQ862.T93,
92.K2.AK8762.T93, 92.A2.K98762.K93, K92.2.AK8762.T93: all 2♦); 1♦ only
with two aces (92.A2.AQ8762.T93, A2.92.AQ8762.T93, A2.J2.AJ8762.T93). The
ace-king does not lift a minor.

**Clubs, 10 HCP**: pass (92.K2.T93.AK8762, 92.A2.T93.AQ8762,
9.KJ82.93.AQ8762, 9.K982.93.AK8762, K9.J982.9.AQ8762, 9.93.KJ82.AQ8762:
all pass). There is no weak 2♣ on this card.

**A side four-card major** (corpus, six-card diamonds, hearts or spades
with 4-10 HCP and four cards in a major): with no honour above the ten,
a weak two 24 times, pass once, one of a suit 4 (ten-counts with the
ace-king); with the jack or better, never a weak two: pass 26 (7 of them
at 10 HCP), one of a suit 36 (all at 10 HCP). Probes:
9.9832.AQ8762.KJ 2♦; 9.93.AQ8762.KJ82 (four clubs) 2♦;
9.J982.AQ8762.Q3 (9) pass; 9.Q982.KQ8762.Q3 (9) pass.

Between one of a suit and pass, at 10 HCP with the honoured side major:

| South | aces + kings | BBA |
|---|---|---|
| 9.KJ82.AQ8762.93 | A K | 1♦ |
| KJ82.9.AQ8762.93 | A K | 1♦ |
| 9.K982.AQ8762.J3 | A K | 1♦ |
| 9.J982.AQ8762.K3 | A K | 1♦ |
| 9.QJ82.AK8762.93 | A K | 1♦ |
| 9.Q982.AK8762.J3 | A K | 1♦ |
| 9.Q982.AJ8762.K3 | A K | 1♦ |
| 9.A982.KQ8762.J3 | A K | 1♦ |
| 9.Q982.AQ8762.Q3 | A | pass |
| 9.QJ82.AQ8762.J3 | A | pass |
| 8.QJ82.AQ8762.J3 | A | pass |
| QJ82.9.AQ8762.J3 | A | pass |
| J8.QJ82.AQ8762.5 | A | pass |
| 9.QJ82.AQJ762.93 | A | pass |
| 9.QJ82.KQJ762.J3 | K | pass |

In the corpus the same split holds for the majors (1♠ with KQ9643.KJ74,
KQT732.AJ83, AQ6542.JT65.97.K; pass with KQ7643.Q983, AQJ863.QT92,
AQJT86.JT85, Q653.KQ7653) and for 72.KJ73.KQJ542.5 and
K743.7.KQ7532.Q5 (two kings: 1♦). So the rule is two of the aces and
kings, queens and jacks not counting: in the rules, controls 3+ or two
kings.

**Six-four in the minors**, 4-8 HCP:

| South | vul | BBA |
|---|---|---|
| 9.84.AQT654.8642 (6) | none | 3♦ |
| 9.J4.KQT654.8642 (6) | none | 3♦ |
| 98.4.AQT654.8642 (6) | none | 3♦ |
| 9.84.AQT654.K642 (9) | none | 2♦ |
| 9.84.AQJ654.K642 (10) | none | 2♦ |
| 9.842.AQT654.642 (6-3) | none | 2♦ |
| 9.84.8642.AQT654 (6) | none | 3♣ |
| 9.84.8642.AQJ654 (7) | all | 3♣ |
| 9.84.8642.AQT654 / 98.4.8642.AQT654 | all | pass |
| 9.84.AQT654.8642 | all | 2♦ |

Corpus: 3♦ on 6♦-4♣ with 5-8 HCP 9 times (8 not vulnerable; the
vulnerable one AKJT52), 2♦ once vulnerable with 8 (AQ9762), and 2♦ with
9-10 (12 of 14; the other two hold two aces and open 1♦). Modelled not
vulnerable only; vulnerable BBA wants a better suit (3♣ with AQJ, pass
with AQT) and that is left open.

**The rules** (`general.style = bba`, seats 1-2):
- preempts.bid, bba weak twos (one rule not vulnerable, one vulnerable
  with the suit-quality test, all hand conditions in `shows`): 4-10 HCP
  as before; no weak two with a
  four-card other major (for 2♦ either major) holding the jack or better;
  at 10 HCP no weak two in a major with the ace-king or with two aces,
  none in diamonds with two aces.
- preempts.bid: 3♦ with 6♦-4♣ and 3♣ with 6♣-4♦, 4-8 HCP, not vulnerable.
- base.bid: the one-level openings are written twice, `when style is not
  bba` and `when style is bba`; the bba ones add to `shows` that at 10 HCP
  with a six-card suit the opening needs two aces or kings (controls 3+,
  or controls 2 without an ace) when the other major has four cards, and
  that 1♣ needs 11. A pass rule with the same
  style says "10 HCP and a six-card suit" so that the pass has a meaning.
- The probed hands agree with the engine (`rbb call` with the bba card):
  60 of 60 at love all, 3 of 4 vulnerable (not the vulnerable 3♣ on
  AQJ654).

**Measured** (`measure2.sh`, `--set general.style=bba`; calls agreeing /
identical auctions / same contract, par):

| | before | openings | with the responses |
|---|---|---|---|
| corpus | 75.7% / 16.0% / 31.2%, −185,676 | 75.7% / 16.1% / 31.3%, −184,932 | 75.8% / 16.3% / 31.5%, −183,355 |
| Basic_* | 84.1% / 37.3% / 46.8%, −2,298 | 84.5% / 37.9% / 47.7%, −2,247 | 84.9% / 39.8% / 49.0%, −2,068 |
| Basic_* NS alone | 90.5% / 52.6% / 61.6%, −1,243 | 90.9% / 53.5% / 62.8%, −1,200 | 91.5% / 56.5% / 64.7%, −1,045 |

"Before" and the last column were measured back to back on two copies of
the sandbox rules, one with these three modules reverted, so other
modules' changes do not enter the difference.

The opening divergences left on Basic_Weak_2 under the bba style: 7
boards (two 5-HCP vulnerable weak twos BBA passes, two ten-counts with
seven clubs in second seat that BBA opens 3♣, one vulnerable 6-4, an
eight-card 4♦ where we bid 5♦, and K76.QJ8542.T5.K5 vulnerable in second
seat, which BBA opens 2♥ and our quality rule passes).

**Our default is unchanged**: `measure2.sh` without `--set` still gives
corpus 75.4% / 14.7% / 30.9%, par −173,271; Basic_* 83.4% / 33.4% /
44.9%, −1,926; Basic_* NS alone 89.2% / 46.6% / 58.5%, −838. Board by
board over the whole corpus (170,161 boards) our auctions are identical;
one replayed call differs (Muiderberg_Two_Bids 279, opener's answer to
2NT after a 2♥ opening: 3♠ now, 3♣ before, BBA 3♣; 1,340,079 → 1,340,078
calls agreeing). The cause is the earlier bba 2♦/2♥ rules: their `when`
mixed `style is bba` with hand terms, and the engine keeps such a rule
possible when reading a call, so they widened what a default 2♥ shows.
Putting the hand conditions in `shows` and leaving `when` public fixed
that; restoring only the old bba 2♥ rule brings the 3♣ back. Rick's
rulings above stand: diamonds 4-9, 12 total points for an opening, no
side four-card major in 2♦ or 2♥ (2♠ allows one). Where the default
differs from BBA: we open 2M on a 10-HCP major with the ace-king or two
aces (Rick's 10-HCP ruling of 2026-09-23 was about preemptive value), 1♦
on every ten-count with six diamonds, 1♣ on ten with six clubs, and 2♦
on 6-4 in the minors.

**Questions for Rick** (not measured against par):
- BBA's line for a ten-count major is "ace-king or two aces opens one,
  otherwise a weak two". Worth trying as our default beside the 10-HCP
  weak two?
- Should a side four-card major with an honour (J or better) stop a weak
  two in our default too (BBA never opens one)? The earlier trial was a
  quality comparison, not this test.
