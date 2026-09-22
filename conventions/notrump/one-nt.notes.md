# notrump-base (`one-nt.bid`): notes

The 1NT opening, responder's natural notrump raises, and opener's answer to
2NT. Cases: `one-nt.test`.

## Guidance

- 1NT is **15–17 HCP exactly**, balanced, as BBA plays it. Tens and length
  do not move the range. The card option "1NT may be 1 HCP light"
  (`notrump.one_nt.allow_one_less`) opens a 14 whose extras bring it to 15
  total points.
- Responder's strength is in **notrump points: HCP + ½ per ten**. A fifth
  card in a suit does not count at notrump.
- Bands are measured against opener's range. Opposite 15–17: sign off 0–7,
  invite 8–9, game 10–15, slam invite (4NT) 16–17, slam (6NT) 18+.
- Opener accepts 2NT **only when game is certain**: 17 opposite 8–9. That is
  17 notrump points, so 15 HCP with four tens accepts.
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

- **Six-card minor.** 21GF-DEFAULT sets `1N-2S transfer to clubs` and
  `1N-3C transfer to diamonds` (card fields
  `notrump.transfers.two_s_clubs` and `.three_c_diamonds`); 2NT stays a
  natural invitation, and 3D is natural. In the corpus BBA's opener always
  completes (3C or 3D, no super-accept), and responder with six passes when
  weak (43 and 56 deals), bids 3NT when invitational or better (22, 23), or
  4NT or a new suit with slam interest. We have no rules for these yet: with
  `5.K73.QJ9542.J83` we pass or bid 2NT.
  Rick's notes on other methods, for when a card asks for them: the common
  standard is 2S for either minor (opener 3C, responder corrects to 3D).
  With four-suit transfers, 2S shows clubs and 2NT diamonds. With a 2S range
  ask, 2S is weak with clubs or a notrump invitation: opener bids 3C with a
  maximum, 2NT otherwise, and responder can then bid 3C with clubs. Rick's
  own: 2NT transfers to diamonds, and opener bids 3C as a super-accept (Kxx
  or better in diamonds), else 3D.
- **Grand slam**: nothing above the slam band. BBA bids 7NT with 21–23.
- **Interference** over 1NT (the `Opps_*` scenarios).

## Fixed (2026-09-21)

- 18+ balanced had no rule and passed 1NT (7 Basic_NT boards). Added 6NT;
  with a five-card major, responder transfers first. Stayman and the
  transfer continuations got slam rules too. Corpus: +365 calls agree,
  same contract 10.6% → 11.1%.

