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
