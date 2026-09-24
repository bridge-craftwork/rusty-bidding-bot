# jacoby-transfers (`jacoby-transfers.bid`): notes

2D/2H transfers over 1NT, the completion, responder's second call, and
opener's answers to invitations and to the mild slam try. The
super-accepts are separate modules. Cases: `jacoby-transfers.test`.

## Guidance

- The transfer is forcing; opener completes (or super-accepts).
- Responder's second call is one `when answered transfer(M)` block, with
  no auction patterns. Notrump decisions (2NT, 3NT) use total points (HCP
  + ½ per ten + 1 per card beyond four, since 2026-09-23). Decisions to
  play in the major use **suit points: HCP + ½ per card beyond four**.
- **8 HCP with a six-card major invites** (transfer, then 3M): 8 HCP + 1 for
  length is 9 suit points.
- 3NT with exactly five offers a choice; opener corrects to 4M with three
  (`choice-of-games.bid`).
- **Mild slam interest** (six-card major, 14–17 suit points): transfer,
  then jump to 4M. Opener bids on (4NT keycard) with 3+ trumps and a
  maximum. Game-only hands and slam hands (18+: Texas needs 33 even opposite
  a 15 minimum) use Texas (`texas-transfers.bid`).
- **Responder's new suit (3m) is game forcing** (Rick, 2026-09-21). Opener
  rebids the major with three-card support, which agrees it below game so
  responder can look for slam; otherwise 3NT.
- Opener plays in the major with an eight-card fit: three opposite five,
  two opposite six. Without a fit he accepts only when game is certain
  (17 opposite 8–9); with the fit, 16 is enough (2026-09-23, below).
- After a super-accept (a known fit of nine or more) responder counts suit
  points. Opposite that maximum, invitational values bid game: the
  super-accept exists to reach thin games.

## Evidence from BBA

Probes (21GF-DEFAULT), `--prefix "1NT Pass 2D Pass 2H Pass" --dealer S`:

| Hand (then 1–4 more tens) | BBA | Agree |
|---|---|---|
| `82.KJ973.K94.Q83` (9, five hearts) | 2NT with 0–1 tens, 3NT with 2+: tens count at notrump | 5/5 |
| `82.QJ9732.K94.Q8` (8, six hearts) | 3H however many tens: length counts, tens do not | 5/5 |
| `82.Q97532.K94.Q8` (7, six hearts) | 3H, the same | 5/5 |
| `82.K9732.K94.Q83` (8, five hearts) | 2NT with 0–1 tens, **pass** with 2+ | 2/5 |

Jacoby_Transfer (500 boards): 85.9% of calls agree, 29.2% identical
auctions, 39.2% the same contract. Jacoby_Super-Accept: 81.7%, 30.8%, 33.8%.

## Accepted differences from BBA

- **A ten in the trump suit.** `82.KT973.K94.Q83`: BBA passes. With the ten
  in diamonds instead (`82.K9732.KT9.Q83`), BBA invites. It is the same on
  every layout. That looks like an internal quirk of BBA, not a principle,
  so we do not model it.
- With a six-card major, BBA's choice between Jacoby and Texas differs from
  ours at 8 and at 10–17 (see `texas-transfers.notes.md`).
- **Opener after the new suit**: BBA jumps to 4M with three trumps (106
  Jacoby_Transfer boards); we bid 3M, as Rick plays it. Once 3M agrees the
  suit, BBA's responder sometimes bids 3NT, offering a choice; we bid 4M in the
  known fit.
- After a plain 3M super-accept, which may be a minimum, BBA passes some
  invitational hands where we bid game (8 boards; 24 others go our way).

## Gaps (not built yet)

- **5-4 invitations.** 9 HCP with five spades and four of a minor bids 3m
  for BBA (for example `A6543.7.K43.QT63`). Our 3m is game forcing, so these
  hands need another route (2NT with an unbalanced hand?). For now they
  have no rule and pass the completion (76 boards in Jacoby_Transfer and
  Texas_or_Jacoby).
- **Five-four with game values after a super-accept.** BBA bids 4M; our
  game-forcing 3m ranks higher (17 boards).
- **Opener after 6NT with five**: no correction to 6M yet.
- **Interference**: a double of the transfer (31 boards), or an overcall.
  (RHO's double of the 1NT opening and a 2♣ overcall are now covered —
  see below — but not an overcall of the transfer itself.)

## Questions

- How should responder invite with a five-card major and a four-card minor,
  now that 3m is game forcing?

## Under interference (2026-09-23)

`after 1N (X) when vs_double` and `after 1N (2C) when vs_2c` keep the
transfers on their own calls, 2♦ for hearts and 2♥ for spades, with the
completions written for each. Two differences from the uncontested
rules:

- **the longer major wins, and spades when they are equal**
  (`when S>=H` / `when H>S`). Uncontested there is no such tie-break and
  5-5 hands go through Stayman; under interference BBA transfers, and to
  the higher suit. Note that a `prefer S` / `prefer H` pair did **not**
  break the 5-5 tie — the two calls score the same — so the choice is a
  `when` on each rule.
- **over a 2♣ overcall the transfer has a five-HCP floor.** Below that
  BBA leaves their 2♣ alone rather than push to the two level with
  nothing; over a double there is no floor, because running is the
  point.

The super-accepts are not written for these auctions: opener simply
completes. Evidence and numbers: `nt-interference.notes.md`.

## Length points (2026-09-23)

`points` counts length now, so the fifth card of the major is worth a
point at notrump.

- **Responder's 2NT / 3NT after the completion** move one point:
  `82.QJ973.K94.J83` (7 HCP) invites, `82.KJ973.K94.Q83` (9 HCP) bids
  3NT. BBA invites with the 9 (probe table above), so this departs from
  it. Counting the old way (the fifth card not counted for these two
  calls only) cost 71 IMPs on the corpus and 14 on Basic_*: kept.
- **Opener accepts with 16 in the fit.** `when asked invite(M)` now bids
  4M with an eight-card fit and 16 opposite 8–9 (`points>=24-...`), and
  3NT without a fit still needs 17. The rule also answers the Stayman
  raise (`1NT 2C 2H 3H`) and other modules' suit invitations. Corpus par
  **+1,309 IMPs**, Basic_* +62. Accepting 3NT at 16 without a fit as well
  measured −18 on its own, so it stays at 17.
- **No pass of an artificial super-accept.** `P "Weak: play here"` could
  pass opener's 2♠ or 2NT super-accept (`Q8542.T86.43.A65` passed 2NT in
  1N board 284); it now needs the fit to be unknown, or opener's call to
  be 3M. Corpus par **+432 IMPs**.
- **Opener answers the quantitative 4NT with five** (`sets
  ask=quant_major(M)`): 6M with three trumps, 6NT with two, at 32
  combined, as over 1NT.

**Tried, not adopted** — question for Rick:

- The six-card major's invite / game (3M, 4M, and Texas for game) on
  **total points** (8 HCP + 2 = 10: game) instead of suit points: corpus
  +225 (with Texas as well, +510), but Basic_* −13 and it contradicts
  "8 HCP with a six-card major invites". Should the six-card decisions
  count the full length point, or stay at ½?
- Support points (shortness) for the six-card game decision: corpus
  +251, Basic_* 0, uncontested −6, and the extra 4M rule took over the
  reading of 4M as the mild slam try. Not kept.

## BBA treatment (2026-09-24)

**Five hearts and four spades, 7–9 HCP.** BBA transfers (all three 15-17
cards: 2♦ on every such hand from 0 to 9 HCP; 2♣ from 10), then bids 2♠
with 7–9 (7: 9 of 9; 8: 132 of 132; 9: 88%) and passes with less. The
2♠ is **not forcing**. Opener, corpus:

| opener ♠-♥ | 15 | 16 | 17 |
|---|---|---|---|
| 3-2, 2-2 | P | P | P 96% |
| 2-3, 3-3 | 3♥ | 3♥ 93–96% | 4♥ 76–83% |
| x-4 | 4♥ | 4♥ | 4♥ (one 3♥) |
| 4-2 | 3♠ / 4♠ | 3♠ / 4♠ | |
| 4-3 | 3♥ 80% | 4♥ 67% | 4♥ |

Responder over 3♥: 9 HCP 4♥ 64–100%, 8 HCP mostly pass.

Probes (Basic-Bridge, dealer S; responder at MP and IMP, love all and
all vulnerable; opener at MP and IMP, love all):
`Q653.KJ874.K2.84` (9) 2♦ then 2♠; with 8 and 7 HCP 2♦ as well; with 11
(`...K2.Q4`) and 13 2♣, then 3♥ over 2♦. Opener after 2♠: `AK7.Q52.AQ53.J92`
(16) 3♥, `AK7.Q52.AQ53.K92` (17) 4♥, `AK7.Q2.AQ53.J952` pass. MP = IMP.
The mirror, `KJ874.Q653.K2.84` (5♠-4♥, 9), bids 2♣ and over 2♥ 3♥ at MP,
4♥ at IMPs (`stayman.notes.md`).

With **5-5 in the majors** BBA transfers to spades (2♥), 100% at every
strength on all three cards.

**What the rules model** (`jacoby-transfers.bid`, contexts `after 1N (P)
2D (P) 2H (P) ... when style is bba`): 2♠ `H=5, S=4`, 7–9 HCP; opener 4♥
with four hearts or three and 17, 3♥ with three, 4♠ with four spades (and
two hearts) and 16, 3♠ less, pass otherwise; responder over 3♥/3♠ game
with 9. 2♥ with 5-5. Stayman's side of it (no 2♣ with 5♥-4♠ below 10) is
in `stayman.bid`.

**Measured** (whole corpus, bba style): `1NT P 2D P 2H P` 70.6% → 82.6%
(2,238 decisions), `1NT P 2D P 2H P 2S P` 30.1% → 89.0% (272).

**Tried as our default** (scratch copy: 5♥-4♠ invitational transfers and
bids 2♠, 8–9 total points, the same answers): corpus par +156 IMPs,
Basic_* −5, Basic_* uncontested 0. **Question for Rick**: within noise;
is the non-forcing 2♠ worth adopting for its own sake (it finds the 4-4
spade fit and the 5-3 heart fit at the two or three level)?
