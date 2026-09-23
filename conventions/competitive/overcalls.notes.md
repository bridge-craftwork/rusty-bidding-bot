# Overcalls (`overcalls.bid`): notes

Simple overcalls of a one-level opening, the weak jump overcall, and the
direct 1NT overcall. Cases: `overcalls.test`. Advancer is in
`advances.bid`; the takeout double is in `takeout-double.bid`.

## Guidance (Rick, 2026-09-22)

- **A simple overcall is always a five-card suit.** No four-card
  overcalls on this card (`overcalls.often_4_cards` is off).
- **8-17 total points at the one level, 12-17 at the two level**, taken
  from BBA's Basic-Bridge meanings, which Rick adopted.
- **Too strong to overcall is a double first, then the suit**: 18+ total
  points doubles whatever its shape (`takeout-double.bid`).
- **A jump overcall follows the weak-two rules**: six cards, 4-9 HCP, and
  vulnerable two of the top three or three of the top five honours. A
  seven-card suit jumps to the three level, as it opens there.
- **The direct 1NT overcall is 15-18 balanced with a stopper**, a point
  wider than our 1NT opening.

## Evidence

`bba-cli --all-meanings` on `Basic_Overcall.pbn` and
`Basic_Takeout_Double.pbn` with the Basic-Bridge card on both sides
(4,520 and 4,942 calls with their meanings), grouped by auction prefix:

| call | BBA's meaning | hands it made it on |
|---|---|---|
| (1x) 1y | "bidable suit", 8-17 total points, 5+ | 8-16 HCP, five or six cards |
| (1x) 2y | "bidable suit", 12-17 total points, 5+ | 11-16 HCP |
| (1x) 1N | "NT style", 15-17 total points, 2-5 in every suit | 15-17 HCP, never a five-card major |
| (1x) 2M jump | "Weak natural 2M", 4-10 total points, 6-7 cards | 8-10 HCP, six cards |

Two-level overcalls also want a suit: BBA passes plenty of 11-14 HCP
hands with a ragged five-card suit (A7632, K9865, J9543) at the two
level. We ask for **six cards, or two of the top three honours, or four
of the top five**; that gained a little on calls and more on contracts
(auctions matching 7.8% → 8.8% in Basic_Overcall).

**1NT with a five-card major**: every 1NT overcall in the corpus had four
cards or fewer in both majors, so the rule denies a five-card major and
we show the suit instead. BBA does the same (board 1 of Basic_Overcall
overcalls 1♠ on a balanced 16-count with AJ972).

Scenario effect (Basic-Bridge card, 500 boards each):

| | before | after this module | after the whole competitive slice |
|---|---|---|---|
| Basic_Overcall, calls | 66.1% | 72.0% | 74.1% |
| Basic_Takeout_Double, calls | 59.6% | 60.4% | 71.3% |

## Accepted differences from BBA

- BBA passes some 12-14 HCP hands with a five-card minor over a major
  opening that we overcall at the two level, and overcalls some that we
  pass. The line is not a rule in BBA; it evaluates the hand.
- BBA's weak jump overcall runs to 10 total points; ours stops at 9, the
  same as our weak two.
- BBA sometimes preempts to the three level on a six-card suit.

## Gaps and open questions

- The balancing seat: `(1x) P (P)` has no rules at all, and the card has
  separate `nt_overcalls.balance.*` ranges for it.
- Overcalls of a 1NT opening, of a weak two, and of two-level openings.
- Two-suited overcalls (Michaels, unusual notrump) are off on this card
  but the fields exist.
- Systems on over our 1NT overcall (BBA plays Stayman and transfers over
  it): `nt_overcalls.direct.systems_on`. The notrump modules key off
  `after 1N (P)`, so nothing fires after `(1x) 1N (P)` yet.
