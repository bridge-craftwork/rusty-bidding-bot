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

Which suit to bid first at the one level: the longest; with two four-card
suits the cheaper, up the line (1♦ over 1♣ with four diamonds and a
four-card major: BBA always does, 76 of 76); with two five-card suits the
higher. Over 1♥ a weak hand with three hearts raises rather than bid 1♠.
With 11+ and a longer minor, 2m comes before a four-card major.

## Evidence

Basic_* responses (2,069 positions after `1x P`): 89.3% agree with BBA.
Before this module we passed every one. Whole corpus: calls agreeing 66.4%
→ 68.5%, same contract 11.7% → 14.9%.

## Accepted differences from BBA

- Where exactly 2M becomes 3M (18 boards like `J52.632.KQ98.A64`: BBA 2♠
  with a flat 10-count, we 3♠).
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

None open.
