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
- Responses by a passed hand beyond switching off jump shifts (Drury is a
  convention of its own).
- Responses after interference.
- 2/1 game force and forcing 1NT (the 2/1 cards): the same module plays
  them as Standard American for now.

## Questions

- 2026-09-23: two-level new suit on 10 HCP and a five-card suit when a
  one-level major is available (see "Total points count length").

## Treatments (2026-09-24)

The default is Rick's raise structure (above). `general.style = bba` keeps
the structure the rules had before his ruling, which was tuned to agree
with BBA: 2M on 6-9 support points, 3M on 10-12 with three or four trumps
(with three hearts and four spades, 1♠ first), 1♠ over 1♥ with 10+
support points. It is not a probed model of BBA: on Basic_Major boards 45
and 65 BBA raised to 2♥ with 10. On Basic_*, with the BBA style, it adds
0.9 points of identical auctions and 0.2 of same contract against the
default raises. The default stands because it is Rick's ruling; against
par it cost 116 IMPs on the corpus (see above).
