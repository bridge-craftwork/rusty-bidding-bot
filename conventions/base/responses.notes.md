# responses (`responses.bid`): notes

Responses to a one-level suit opening, uncontested. Standard American as BBA's
Basic-Bridge card plays it; the Basic_* scenarios use that card. Cases:
`responses.test` (one corpus hand per response, all agreeing with BBA).

## Guidance (Rick, 2026-09-22)

- **Treatments**, one card field each:
  - `minor_openings.one_club_responses`: `up_the_line` (default, as BBA:
    1♦ with four diamonds and a four-card major) or `walsh` (bypass
    diamonds for a four-card major unless game forcing, 12+).
  - `minor_openings.two_nt_response`: `invitational` (default, as BBA:
    1m-2NT 11-12, 1m-3NT 13-15) or `game_forcing` (1m-2NT 13-15, 1m-3NT
    16-17; with 11-12 bid a new minor, then 2NT).
- **Strong jump shift: 15+**, matching BBA for now.
- **Two-level new suit: as BBA**, 11+, or a six-card suit with 10.
- **Game values with a fit and no four-card side suit: a three-card minor
  first**, as BBA (4M only when there is no three-card minor).
- BBA's own "Walsh style" switch stays on the card as
  `minor_openings.walsh.play`, but the rules read `one_club_responses`:
  with the switch on, BBA still bids up the line.

## How the rules were set

BBA was asked for its meaning of every call: the Basic_* deals re-bid through
`bba-cli --all-meanings` (auctions identical to the corpus, 4000 of 4000).
Its meanings for responder's first call, with the hands that made them:

| Call | BBA's meaning | Hands in the corpus |
|---|---|---|
| New suit at the one level | 6+ total points, 4+ cards | 3-19 HCP, median 9 |
| 1NT | 6-11, no support (over 1♥: no 4 spades) | 6-10 HCP |
| 2M | "7 to 9 total points", 3+ | 6-12 HCP |
| 3M | "1M-3M inviting", 10-12, 3+ | 9-12 HCP |
| 4M | "preemptive", 4-8, 4+ | 4-11 HCP |
| Two-level new suit | "10+ HCP if fit or 6+; 12+ F" | 8-20 HCP, median 12 |
| Jump to 2 of a higher suit | 15+, 5+ cards | 14-22 HCP |
| Jump to 3 of a lower suit | "Inviting jump shifts", 10-12, 6+ | 10-12 HCP |
| 1♣-2♣, 1♦-2♦ | 6-10, 5+, no four-card major | |
| 1♣-3♣, 1♦-3♦ | 9-13, 5+ | |
| 2NT | 11-13, balanced (rare: BBA bids a new suit) | |
| Pass | 0-5 | |

**Raises count support points.** BBA's choice between 2M, 3M and a new suit
(the way to game) follows HCP plus shortness plus a fourth trump. At 9 HCP,
for example, it bids 2M with a flat three-card fit, but 3M with a doubleton
or four trumps. We use `tp(M)`: HCP plus void 5, singleton 3, doubleton 1,
and with only three trumps void 3, singleton 2, doubleton 1. Bands: 2M 6-9,
3M 10-12, and game (a new suit, 3NT, or 4M) 13+. With a fit and limit
values responder raises before bidding a new suit.

**Rick's raise structure over 1M (2026-09-23)**, replacing the bands above:

- 6-10 support points and 3+ trumps: raise to 2M, ahead of any new suit
  (not 1♠ then a heart preference, not 1♠ then notrump).
- 11-12 with four trumps: 3M at once.
- 11-12 with three trumps: a new suit, then a jump raise
  (1♥-1♠-1NT-3♥, 1♥-1♠-2♣-3♥). Read "then jump to 2H" in his message as
  the jump raise to 3M; **to confirm**.
- Where three trumps and 11-12 leave no new suit to bid (over 1♠: 3♠-4♥
  with 3-3 minors; or 11 support points from shortness with a suit too
  weak for the two level), 3M stays as a fallback (`priority -1`). Without
  it 445 hands passed 1♠ (−2,404 IMPs).
- A 2-level new suit counts support points: with three trumps and 11+
  support points it is allowed below 11 notrump points (`932.854.62.AKQJ2`).

Measured against the previous bands, whole corpus: calls agreeing 75.6%
unchanged, same contract 30.5%, −116 IMPs vs par (of −196,305). Basic_*
uncontested NS: roughly flat. The costs are what the ruling asks for:
10-point raises to 2♠ where BBA bids 3♠ (1,120 boards, −223), and the new
suit before a three-card limit raise (about −500 over 1♥ and 1♠). Taken
as Rick's ruling. Follow-up: after 1M-2M, opener with 23+ now blasts
4NT because the raise can hold 10 (130 boards, −140); that belongs to a
slam try, not keycard.

Which suit to bid first at the one level: the longest; with two four-card
suits the cheaper, up the line (1♦ over 1♣ with four diamonds and a
four-card major: BBA always does, 76 of 76); with two five-card suits the
higher. Over 1♥ a weak hand with three hearts raises rather than bid 1♠.
With 11+ and a longer minor, 2m comes before a four-card major.

## Total points count length (2026-09-23)

`points` now adds 1 for each card beyond four (Rick: "total points should
include length points"). Each band here was re-chosen for the measure that
fits its decision, and measured against par (net IMPs vs BBA; less negative
is better). Baseline after the engine change, before these rules moved:
corpus −182,498, Basic_* −2,153, Basic_* uncontested NS −1,030.

- **1NT response: 6-10 HCP** (was total points). A five-card minor is not a
  reason to leave 1NT: `K3.Q84.KJ852.J62` bids 1♠-1NT. With total points,
  10 HCP and a five-card suit could not bid 1NT at all.
- **1♦-1M "11+ with longer clubs: 2♣ first"** and **1♥-1♠ "11+ with a
  longer minor: 2m first"**: the threshold is HCP. With 10 HCP and 5-4 the
  major comes first (previously some such hands passed 1♦).
- **Walsh 1♣-1♦ "game forcing"**: 12+ HCP (Rick's "12+"). With total
  points 11 HCP and 5-4 (`KQ2.JT63.KQ742.8`) became game forcing and bid
  1♦ over the major. Not in the corpus (no card plays Walsh), so unmeasured.
- **Kept on total points**: the two-level new suit (11+) and the
  invitational jump shift (10-12). Moving the two-level new suit to HCP
  cost 930 IMPs over the corpus on its own; moving the jump shift to HCP
  cost 701 (11-12 HCP six-card suits are worth forcing, and 9-10 HCP ones
  inviting, which total points gets right on both sides). So four `.test`
  cases now differ from BBA, taken on par: `A8.94.532.AK8632` 1♦-2♣ (BBA
  3♣), `AK85.8.QT9862.K3` 1♥-2♦ (BBA 3♦), `T.AKT542.87.A953` 1♠-2♥ (BBA
  3♥), `.KT9763.AQ98.432` 1♠-3♥ (BBA 2♥).

| Variant | corpus | Basic_* | Basic_* NS |
|---|---|---|---|
| baseline (all bands on total points) | −182,498 | −2,153 | −1,030 |
| jump shifts on HCP | −183,199 | −2,204 | −1,081 |
| two-level new suit on HCP | −183,428 | −2,224 | −1,087 |
| 1NT and two-level on HCP, 1♦-1M guard on HCP | −181,766 | −2,138 | −1,014 |
| 1NT on HCP | −182,051 | −2,097 | −983 |
| 1NT and 1♦-1M guard on HCP | −181,903 | −2,073 | −959 |
| **+ 1♥-1♠ guard on HCP (kept)** | **−181,873** | **−2,066** | **−952** |

Question for Rick: with 10 HCP and a five-card suit both 1NT (6-10 HCP) and
a two-level new suit (11+ total) are allowed; 1NT wins over 1M, but over 1♦
`K843.Q3.J6.KJ852` bids 2♣ rather than 1♠. Should a two-level new suit
need 11 HCP when a one-level major or 1NT is available? Moving the whole
two-level band to HCP measured worse (−930), so it stays on total points
until you rule.

## Evidence

Basic_* responses (2,069 positions after `1x P`): 89.3% agree with BBA.
Before this module we passed every one. Whole corpus: calls agreeing 66.4%
→ 68.5%, same contract 11.7% → 14.9%.

## Accepted differences from BBA

- Where exactly 2M becomes 3M: BBA makes a limit raise with some 10
  support points; we raise to 2M (Rick: 6-10).
- BBA's preemptive 4M with 8-10 HCP, four trumps and a singleton (it counts
  these as game hands); we make a limit raise.
- Some 6-counts BBA passes (for example `65.K8654.KT43.86`).

## Gaps (not built yet)

- **Opener's rebid**: the largest gap in the whole corpus now (1♣-1♦,
  1♦-1♥, 1♦-1♠ each pass on thousands of boards).
- Responses by a passed hand beyond switching off jump shifts. Drury is
  its own module (`majors/drury.bid`, 2026-09-28): with it on, a passed
  hand's limit raise, natural 2♣ and game-values 2♣/2♦ are switched off
  here.
- Responses after interference.
- 2/1 game force and forcing 1NT: built 2026-09-30 (below).

## Questions

- 2026-09-23: two-level new suit on 10 HCP and a five-card suit when a
  one-level major is available (see "Total points count length").

## BBA treatment (2026-09-24)

The default is Rick's raise structure (above) and stays. `general.style =
bba` now plays a probed model of BBA's responses to 1M; it replaces the
first `bba` treatment, which was the pre-ruling structure tuned for
agreement and never probed (BBA raised to 2♥ with 10 on Basic_Major 45 and
65, which that model bid 1♠).

**Probes** (`rbb probe`, Basic-Bridge both sides, dealer S, None, MP;
responder North after `1S Pass` or `1H Pass`; honours placed as A♣, K♥,
Q♦, K♠, ... so each row adds one honour; no tens):

| Shape (♠-♥-♦-♣) over 1♠ | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|
| 3-3-3-4 (3 trumps, flat) | 2♠ | 2♠ | 2♠ | 2♠ | 2♠ | 3♠ | 3♠ |
| 3-4-2-4 (doubleton Qx) | Pass | 2♠ | 2♠ | 2♠ | 3♠ | 3♠ | 2♣ |
| 3-4-1-5 | 2♠ | 2♠ | 2♠ | 3♠ | 3♠ | 3♠ | 2♣ |
| 4-3-3-3 | 2♠ | 2♠ | 2♠ | 3♠ | 3♠ | 3♠ | 2♣ |
| 4-3-2-4 | 2♠ | 2♠ | 2♠ | 3♠ | 3♠ | 3♠ | 2♣ |
| 4-4-1-4 | 2♠ | 2♠ | 4♠ | 4♠ | 4♠ | 4♠ | 2♣ |
| 5-3-3-2 | 2♠ | 2♠ | 2♠ | 3♠ | 3♠ | 2♦ | |
| 5-3-4-1 | 4♠ | 4♠ | 4♠ | 4♠ | 4♠ | 2♦ | |
| 5-4-4-0 (from 4) | 4♠ | 4♠ | 4♠ | 4♠ | 4♠ | 2♦ | |

Over 1♥ the same shapes give the same answers (3-4-1-5: 4♥ from 7 with
the five-card club suit). With three hearts and four spades: 2♥ up to 9-10,
3♥ on 10 with a doubleton, 1♠ from 11 (from 10 with five spades); with
four hearts BBA never bids 1♠, and with 12 it bids a three-card minor.

- **Tens count for nothing** in a raise: `432.K32.QJ2.AT32` and the same
  hand with three tens both bid 2♠; 8-counts with one to three tens all 2♠.
- **Honour placement matters** and is not modelled: a queen or jack in
  trumps upgrades (`J432.K32.Q2.A432` 10 → 4♠, `Q432.K32.Q2.A432` 11 →
  2♣, but `K432.Q32.Q2.A432` 11 → 3♠); an ace in a short side suit
  upgrades (`K972.86.A65.K874` 4♠, `K972.86.K65.A874` 3♠); an honour in
  the doubleton does not count as shortness (`432.K432.Q2.A432` 9 → 2♠,
  where the corpus hands with a small doubleton and 9 bid 3♠).
- BBA's 4♠ is labelled "preemptive, 4 to 8 total points" by
  `--all-meanings`, but it is bid on 8-10 HCP with four trumps and a
  singleton: a hand worth game that is short of the HCP to force.

**Corpus** (Basic_*, raises with 3+ trumps, 10-12 HCP): with four trumps
and 11 HCP BBA bids a new suit (5 of 5 over 1♠, 7 of 12 over 1♥), not the
limit raise the probes gave for honour-light hands; with three trumps and
11 it is half and half (7 new suit, 7 3♠).

**The model** (`responses.bid`, rules under `when style is bba`):

- 2M: three trumps and 6-10 HCP flat, or 6-8 with a doubleton or
  shorter; four trumps and 5-8 HCP (without the shortness that makes 4M).
- 3M: three trumps and 11-12 flat, or 9-11 with a doubleton or shorter;
  four trumps and 9-10 (not the 4M hands below).
- 4M: four trumps and a singleton or void with at most 10 HCP (`tp(M) +
  M >= 15`: 8 HCP with four trumps, 7 with five); or 9-10 with a doubleton
  and four controls or a five-card suit.
- A new suit, forcing: 11 HCP with four trumps, 12 with three and a
  doubleton or shorter, else 13 (the three-card minor as before). Over
  1♥ with three hearts and four spades: 1♠ from 11, from 10 with five
  spades or a doubleton, and with six spades; 3♥ below that.
- 1♠-2♥ on 10 HCP and five hearts (probes: 10 HCP five hearts bids 2♥
  over 1♠ whatever the shape; a minor needs 11, `2.KJ2.K5432.K432` 1NT).
- The default's preemptive 4M (5+ trumps, 5-9) is off under `bba`.

**Measured** (`--set general.style=bba`; before = the old `bba`
treatment, after = this one plus the rebid treatments in rebids.bid and
responder-rebids.bid):

| | calls | identical auctions | same contract | par (IMPs vs BBA) |
|---|---|---|---|---|
| corpus, before | 75.7% | 16.0% | 31.2% | −185,676 |
| corpus, after | 75.8% | 16.8% | 31.8% | −187,508 |
| Basic_*, before | 84.1% | 37.3% | 46.8% | −2,298 |
| Basic_*, after | 84.6% | 40.4% | 48.6% | −2,193 |
| Basic_* NS, before | 90.5% | 52.6% | 61.6% | −1,243 |
| Basic_* NS, after | 91.2% | 57.1% | 64.4% | −1,162 |

Responder's first call on Basic_* uncontested NS: after 1♠ 462 → 496 of
538 agree (85.9% → 92.2%), after 1♥ 334 → 356 of 396 (84.3% → 89.9%).
The divergences named for this work: 1♠-2♠/3♠ 14 → 2, 1♠-4♠/3♠ 11 → 1,
1♥-2♥/1♠ 8 → 0, 1♥-4♥/3♥ 7 → 2, 1♠-2♥/1NT 5 → 0. The default is
unchanged: every one of the 170,161 corpus boards bids the same with and
without these rules (corpus −173,271, Basic_* −1,926, Basic_* NS −838).

**Where BBA still differs under `bba`** (each a handful of boards):
three trumps and 9 with an honour doubleton (`AJT94.K62.863.J6`, Basic_Major
45: BBA 2♥, the model 3♥); the honour-placement cases above; 4♠ on
three trumps and a seven-card side suit (`A75.AT98643.73.2`); 11 HCP
three-trump hands that BBA sends through a new suit.

**Where our default differs and why**: Rick's ruling of 2026-09-23 (2M
6-10 support points, a limit raise only with four trumps) is the default.
BBA's 3♠ with a flat 9 and four trumps, and its 4♠ on 8-10 with a
singleton, are not in it.

## Strong jump shifts count HCP (2026-09-25)

The strong jump shifts said `points>=15`. Total points count half a
point a ten and a point a card beyond four, so 12 HCP with four tens and
five spades jumped (AJT74.JT4.QT.AT5 over 1H). BBA
(`probes/sjs-1H.toml`, 800 hands with 5+ spades over 1H, Basic-Bridge):

| HCP | 12 | 13 | 14 | 15+ |
|---|---|---|---|---|
| Jump to 2S | never | ~1% | ~25% | ~94% |

The rules now say `hcp>=15`. Corpus: par -167,558 -> -167,342 (+216 IMPs);
the 1H P 1S / 2S divergence alone cost 453 IMPs over 261 boards.

## The fourth trump with shortness makes a limit raise (2026-09-25)

Corpus, Basic-Bridge, 1S P: BBA's 3S against our 2S cost 276 IMPs (184
boards); our 3S against BBA's 2S won 435 (138). By responder's hand:

| Trumps | HCP | Shortness | Boards | BBA's call | Ours | IMPs to us |
|---|---|---|---|---|---|---|
| 4 | 9 | doubleton | 59 | 3S | 2S | -179 |
| 3 | 9 | doubleton | 99 | 3S | 2S | -98 |
| 3 | 8 | singleton | 106 | 2S | 3S | +316 |
| 4 | 8 | singleton | 25 | 2S | 3S | +115 |

So our shortness count is right, and a fourth trump is worth a point
when there is shortness to ruff. With four trumps, 10 support points and
a doubleton or shorter, we now make the limit raise.
- Corpus: -167,342 -> -167,152.
- Flat 10s still raise to 2M (Rick: 2M is 6-10; the corpus agrees, +4
  over 11 boards).
- The same for three trumps with a doubleton lost 636 IMPs overall: not
  adopted.

## Invitational jump shifts: BBA's 10-12 HCP loses to ours (2026-09-25)

BBA's invitational jump shift over 1S (Basic-Bridge, `probes/ijs-1S.toml`,
800 hands with a six-card minor and no spade fit) is 10-12 HCP. With
7-9 it bids 1NT; with 13+ it bids 2m. Ours counts total points (10-12,
so 8-10 HCP with six cards) and bids 2m with 11-12: 26% agreement.
- BBA's HCP range: 90% agreement but -483 IMPs against par.
- Widening ours to 10+ points and up to 12 HCP: -409.

Par decides: ours stays. Tests 24, 41, 48 and 52 are Rick's cases for it.

## Drury, tried (2026-09-25)

21GF-DEFAULT plays Reverse Drury and 21GF-GIB plays Drury; we play
neither. BBA's opener answers (`probes/drury-opener-*.toml`): minimum
2M/2D, 12-14 the other, and 2NT, 3NT, new suits, splinters or game
above that. A simple version was tried: 2C with three trumps and 10-12
support points; opener's minimum, 12-14 or game; responder signs off.
It lost to our passed-hand limit raise on both measures: -237 by
distance, -354 to the side (536 boards). Not adopted. A fuller version
(opener's strong answers, responder's game tries) would be needed to
beat it.

Built 2026-09-28 as `majors/drury.bid`, with opener's game answer and
responder's follow-ups and 2♣ from 10 support points: adopted
(drury.notes.md).

## Over 1H, 1S before a five-card minor with 11-12 (2026-09-25)

With four spades, a five-card minor and 11-12, BBA responds 1S; we bid
the minor at the two level. Now 1S, and 2m with four spades needs 13 or
a six-card minor (test 41 keeps 2D with six). +52 by distance, +71 to
the bidding side.

## 2/1 game force and the forcing 1NT (Rick, 2026-09-30: "definitely fix 2/1")

On cards with `major_openings.two_over_one.game_force` (the 21GF cards;
derived from the system type) the two-over-one is **game forcing**
(`sets forcing=game`) with opening values: 13+ total points with 12+ HCP
by default (length counts, Rick 2026-09-23), 13+ HCP under
`general.style = bba`. BBA (probe 2026-09-30, 21GF-DEFAULT, over 1S, 1H
and 1D): 2/1 from 13 HCP; 12 HCP with a five-card suit bids 1NT over a
major and the invitational 2NT over 1D. The 1NT response over a major is
6-12, forcing (`sets forcing=round`) when the card plays
`one_nt_response.forcing` (21GF-DEFAULT; not 21GF-GIB). Standard American
cards keep the 11+ one-round force.

Around it: over 1D, 11-12 with clubs bids 2NT and 10-12 with six clubs
3C (short of a game force); the "11+ with a longer minor: 2m first" gate
on the one-level majors lets 11-12 bid the major on 2/1 cards. Opener's
rebids over a forcing 1NT and responder's continuations: rebids.notes.md,
responder-rebids.notes.md.

Measured (IMPs vs BBA, par as the yardstick): random deals (20,000,
21GF) +96, corpus +115; calls agreeing with BBA 80.4% -> 80.6% random,
78.1% -> 78.3% corpus; "no rule in a live auction" 324 -> 316 random,
2,669 -> 2,605 corpus; broken forces unchanged. Measuring responder's new
11-12 minor raise in points instead of HCP scored +375 more on the corpus
but overrode impossible-2s.bid's tested choices, so it counts HCP.

Not measured yet: BBA's plain 13 HCP threshold as the default.

## Sources

- **The system:** Standard American responses as BBA's Basic-Bridge card
  plays them. The general framework is standard practice, not yet cited
  to a book or article.
- **BBA's meanings:** every Basic_* deal re-bid through
  `bba-cli --all-meanings` ("How the rules were set"), which is where the
  first bands came from.
- **Rick's rulings:** the treatments `one_club_responses` and
  `two_nt_response`, strong jump shift 15+, two-level new suit and the
  three-card minor first, as BBA (2026-09-22); the raise structure over 1M
  (2026-09-23, one detail still to confirm); total points count length
  (2026-09-23); Walsh's game force at 12+.
- **BBA probes:** `rbb probe` on Basic-Bridge for the `bba` raise model
  (2026-09-24, the tables in "BBA treatment"); `probes/sjs-1H.toml`,
  `probes/ijs-1S.toml`, `probes/drury-opener-21GF-DEFAULT.toml` and
  `probes/drury-opener-21GF-GIB.toml` (2026-09-25).
- **Corpus measurements:** strong jump shifts on HCP, the fourth trump
  with shortness, over 1♥ 1♠ before a five-card minor, and the Drury
  trial (not adopted), all 2026-09-25.
- **Where we differ:** "Accepted differences from BBA"; the invitational
  jump shift, where par beat BBA's 10-12 HCP range and ours stays
  (2026-09-25).
