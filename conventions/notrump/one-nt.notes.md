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

- (Six-card minors are now `minor-transfers.bid`; see its notes.)
- **Grand slam**: nothing above the slam band. BBA bids 7NT with 21–23.
- **Interference** over 1NT (the `Opps_*` scenarios).

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
