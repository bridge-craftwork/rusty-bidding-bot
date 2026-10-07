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

- **Without Texas or the super-accepts (2026-10-07).** The convention
  score switches them off, and two of responder's positions after the
  completion had no rule: six-card hands between the bands (over
  `strength=signoff` in total points, under `suit_strength=invite` in
  suit points: J85.KJ9754.6.J52, 6 HCP), and without Texas the slam hands
  with six. The first now pass or invite by HCP (as the 5-5 hands do);
  the second bid 5M (16-17 suit points: opener bids six unless minimum)
  or 6M (18+). No keycard ask: 4NT over the completion is the
  quantitative raise with five. With Texas off the no-rule positions
  on Texas_Transfer/Texas_or_Jacoby went 51 to 1 (see texas-transfers.notes.md).

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
- **Opener accepts with 16 in the fit.** `when asked invite(M)` (in
  rebids.bid since 2026-09-30, so that it works without transfers) now bids
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

## Super-accept needs 16 (2026-09-25)

A screen of every convention module by removal found super-accept.bid
net harmful: removing it gains +2,487 IMPs to our side (+64 by
distance). It super-accepted with any four trumps, minimums included.

| Super-accept with four trumps | Distance | IMPs to our side |
|---|---|---|
| And 16+ (adopted) | +88 | +1,517 |
| And 17 | -58 | +1,489 |

Removal would still do better for our side by about 1,000 more. **For
Rick:** the responder's continuations after a super-accept (4M with
invitational values) may be the rest of it.

The same screen, IMPs to the bidding side when each module is removed:

| Module | IMPs |
|---|---|
| rkcb-1430 | -41,595 |
| stayman | -38,829 |
| jacoby-2nt | -30,434 |
| jacoby-transfers | -29,847 |
| texas-transfers | -3,415 |
| blackwood | -2,241 |
| inverted-minors | -1,946 |
| responsive-doubles | -996 |
| new-minor-forcing | -955 |
| superaccept-doubleton | -738 |
| control-bids | +890 (removal helps; slam/control-bids.notes.md) |

## 5-5 invitational only through length (2026-09-25)

After 1NT-2♦-2♥, 75.QJ952.3.KT974 is 6 HCP but 8 total points: the
invitational band, where 2NT needs a balanced hand, 3M six cards, and
3♣ game. No rule matched (Basic_Openers_Rebid 79 and 395; BBA passes
both). A pass with five of the major, invitational in total points but
no more than a signoff in HCP, is now a fallback below the other calls.

The same shape with a real invitation in HCP (8+, KT432.Q32.2.QJ93)
bids 2NT despite the singleton, as BBA does (Basic_Openers_Rebid 172).
And after 2NT and opener's decline in three of the major responder
passes (it had no rule; Basic_Openers_Rebid 225, BBA passes).

## Opener's answer to the quantitative 4NT (2026-10-03)

1NT-2♦-2♥-4NT (five hearts, 5-3-3-2, 16-17 total points). Rick:
"sometimes we will prefer NT to M when we are 4333 with no ruffing
values. Check the 5H response performance at 4333 vs other 3-card
support patterns, and also vs 4333 with 4 being hearts vs 4-card support
with other patterns." His 5♥ (a fit and a minimum, to play) had measured
−21 contract errors on 33 corpus boards (`base/fit.notes.md`).

Tested his way, by dealing hands to the auction rather than re-running
the corpus (`probes/tools/dd_auction_test.py probes/dd/quant-4nt-hearts.toml`):
North a 15-17 1NT without a five-card major, South five hearts in a
5-3-3-2 with 16-17 total points (our 4NT band), East-West random; every
deal solved double dummy, all contracts by North (he bid 1NT and 2♥
first). IMPs per board against passing 4NT, ± standard error, none
vulnerable (both vulnerable scales the same way, a little larger):

| opener | HCP | deals | 5♥ | 6♥ | 6NT |
|---|---|---|---|---|---|
| 4-3-3-3, three hearts | 15 | 2,050 | −0.69 ±0.08 | −1.56 ±0.23 | −1.54 ±0.24 |
| | 16 | 1,257 | −0.30 ±0.07 | +2.72 ±0.28 | **+3.25** ±0.29 |
| | 17 | 693 | −0.08 ±0.06 | +5.25 ±0.33 | **+6.28** ±0.34 |
| 4-3-3-3, four hearts | 15 | 2,058 | −0.44 ±0.09 | −3.50 ±0.21 | −3.69 ±0.22 |
| | 16 | 1,211 | −0.23 ±0.07 | +0.34 ±0.30 | **+0.91** ±0.31 |
| | 17 | 731 | −0.07 ±0.04 | +4.76 ±0.34 | **+5.76** ±0.35 |
| three hearts, 4-4-3-2 | 15 | 1,500 | +0.01 ±0.09 | **+1.07** ±0.26 | +0.52 ±0.28 |
| | 16 | 933 | −0.04 ±0.09 | +4.35 ±0.30 | +4.70 ±0.32 |
| | 17 | 567 | −0.04 ±0.09 | +6.16 ±0.34 | **+7.36** ±0.34 |
| three hearts, 5-3-3-2 | 15 | 1,526 | −0.07 ±0.09 | **+1.80** ±0.25 | +1.05 ±0.27 |
| | 16 | 928 | −0.11 ±0.10 | +4.53 ±0.30 | +5.06 ±0.31 |
| | 17 | 546 | −0.12 ±0.10 | +6.58 ±0.33 | **+8.21** ±0.31 |
| four hearts, 4-4-3-2 | 15 | 1,473 | +0.61 ±0.09 | **+1.47** ±0.26 | −1.04 ±0.27 |
| | 16 | 953 | +0.64 ±0.09 | **+5.19** ±0.28 | +3.96 ±0.32 |
| | 17 | 574 | +0.21 ±0.08 | **+6.94** ±0.31 | +7.14 ±0.34 |
| two hearts | 15 | 1,536 | −2.25 ±0.12 | −3.66 ±0.25 | −1.45 ±0.27 |
| | 16 | 980 | −1.50 ±0.12 | −0.37 ±0.33 | **+2.27** ±0.34 |
| | 17 | 484 | −1.10 ±0.14 | +2.00 ±0.47 | **+6.18** ±0.41 |

Scored against 6♥ directly (`--baseline "6H N"`), 6NT − 6♥ is: 4-3-3-3
+0.0 to +0.2 at 16-17 (−0.3/−0.4 at 15, where both lose); three hearts
and a doubleton −0.8 at 15, −0.1/−0.2 at 16 (a tie), +0.25/+0.5 at 17;
four hearts and a doubleton −2.1, −1.3, −0.4 (±0.13-0.15).

What it says:

- **5♥ never wins.** It beats pass only with four hearts and a doubleton
  (+0.6), and there 6♥ beats it by a further 0.9. Over a 4-3-3-3 it
  loses to 4NT (−0.4 to −0.7 at 15): Rick's instinct about 4-3-3-3 is
  right, and it extends to 4-3-3-3 with four hearts, which is the worst
  slam hand of all (no side four-card suit either).
- **The doubleton is a point.** With a fit and a doubleton a 15-count
  plays slam in hearts (+1.1 to +1.8 against pass); a 4-3-3-3 15-count
  passes. That is opener's support points (`tp(H)`: HCP + 1 for the
  doubleton) reaching the 32 the rule already asks of `points`.
- **The strain:** four trumps and a doubleton play hearts; three trumps
  and a doubleton hearts below a maximum, notrump with 17; 4-3-3-3 and
  two hearts notrump. At matchpoints 6NT outscores 6♥ on 54-88 % of the
  boards in every class, so the major is an IMP choice; the 15-count's
  slam on the doubleton stays in hearts (6♥ beats pass on 57-60 %, 6NT
  only 42-52 %).
- **Opposite responder's 16 or 17** (`--by N.h --by S.pts`): 15 + a
  doubleton opposite 16 is close to even (+0.05 to +0.34), opposite 17
  +2.6 to +4.1; 32 combined in `tp` is the right line. 4-3-3-3 with four
  hearts and 16 opposite 16 loses even in 6NT (−1.0 ±0.4); not split
  out, since opener cannot tell 16 from 17.

The rule (`when asked quant_major(M)`): at IMPs 6M with four trumps and a
doubleton, or three and a doubleton below opener's maximum; 6M with a fit,
a doubleton and a minimum that the doubleton lifts to 32 (`tp(M)`), at
either scoring; 6NT on any other acceptance; pass otherwise. No 5M.
Spades: the same test (`probes/dd/quant-4nt-spades.toml`), below.

**Spades** (1NT-2♥-2♠-4NT, 1,000 deals per class at 15, 300-650 at 16
and 17): the same picture. 5♠ never best (+0.61 ±0.10 with four spades
and a doubleton at 15, where 6♠ is +1.76); the 15-count with a fit and a
doubleton +1.55 to +1.76 in 6♠ against pass; 4-3-3-3 passes at 15
(−1.3 / −3.1 in slam) and plays 6NT or 6♠ alike above it (6NT − 6♠ +0.1
/ −0.1 ±0.1-0.2); four spades and a doubleton prefer 6♠ at every
strength (6NT − 6♠ −2.3, −1.8, −1.0); three and a doubleton 6♠ at 15-16,
6NT at 17 (+0.3 / +0.6).

**The sets** (base main 02fc735, 21GF-DEFAULT ranges):

- Corpus (matchpoints throughout): 102 boards, contract errors **+31**
  (+26 / +5), distance from par **+43** (+36 / +7); the defenders'
  doubling −44 (our engine defending does not double the failing slams
  now bid), errors in all −13. By first change: 6♥→6NT +11 (45 boards),
  6♠→6NT 0 (44), pass→6♥ −40 (7), pass→6♠ +16 (6). The 13 slams on a
  doubleton are too few to judge (−24); the dealt deals decide.
- 21GF random set: 7 boards, all 6M→6NT with the same result: 0.
- Vanilla: no transfers, no change (0 boards).

**Re-running or asking the next question this way.** A spec in
`probes/dd/` names the dealer3 prefilter, the Python filters per seat,
the classes (first match wins, each with an optional dealer3 condition
to fill it quickly), the candidate contracts with their declarer and the
baseline; `probes/tools/dd_auction_test.py SPEC` deals until each class
has its `quota`, solves with bridge-solver (both siblings' release
builds), caches the tricks in `.rbb-cache/dd/<name>.jsonl` and prints
the table at both vulnerabilities. `--report-only` re-tallies the cache,
`--by EXPR` splits it (`N.h`, `S.pts`, `N.L[1]`...), `--baseline "6H N"`
scores against another contract, `--json` writes the table. 3,000-4,000
deals per class took about 40 minutes on a loaded machine.

## Sources

- **The convention:** Jacoby transfers over 1NT. Standard practice, not
  yet cited to a book or article.
- **Rick's rulings:** responder's new suit (3m) is game forcing
  (2026-09-21); 8 HCP with a six-card major invites; opener bids 3M, not
  4M, after the new suit ("as Rick plays it"); Texas for slam needs 18
  (`texas-transfers.notes.md`).
- **BBA probes:** `rbb probe` on 21GF-DEFAULT with
  `--prefix "1NT Pass 2D Pass 2H Pass" --vary-tens` (the tens table) and
  on Basic-Bridge for the 5♥-4♠ treatment (2026-09-24), both given as
  command lines and hands; not yet kept as `probes/*.toml` specs.
- **BBA evidence:** the Jacoby_Transfer and Jacoby_Super-Accept corpora;
  the corpus table of opener's answers to the non-forcing 2♠
  (2026-09-24); interference evidence in `nt-interference.notes.md`.
- **Corpus measurements:** length points (2026-09-23), the super-accept
  floor and the module-removal screen (2026-09-25).
- **Double-dummy deal tests:** opener's answer to the quantitative 4NT
  (2026-10-03, Rick's question): `probes/dd/quant-4nt-hearts.toml` and
  `quant-4nt-spades.toml`, run by `probes/tools/dd_auction_test.py`.
  Rick's 5M (a fit and a minimum) is not adopted: it never beat the
  better of pass and slam. His 4-3-3-3-plays-notrump instinct is.
- **Where we differ:** a ten in the trump suit, the Jacoby/Texas choice,
  opener after the new suit, and responder after a plain super-accept
  ("Accepted differences from BBA"); responder's 2NT/3NT move a point
  with length points.
