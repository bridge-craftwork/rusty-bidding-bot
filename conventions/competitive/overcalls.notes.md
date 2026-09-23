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

## Systems on over the 1NT overcall

`nt_overcalls.direct.systems_on` (default on, which is what BBA plays).
Rather than copy the 1NT family into a competitive module, the `after`
line now takes alternatives (`docs/LANGUAGE.md` section 4), so every
notrump context reads

```
after 1N (P) | (1x) 1N (P) | (1x) P (P) 1N (P) when !they.bid | systems_on
```

and Stayman, Jacoby and Texas transfers, minor-suit transfers,
super-accepts and the natural 2NT/3NT responses all apply after our
overcall, direct or balancing. The continuations were already written
against state (`when answered transfer(M)`), so they needed nothing.

BBA's meanings confirm it plays the same way: over `(1x) 1N (P)` the
corpus has 2♣ "Stayman" (7+), 2♦/2♥ "transfer", 3♣ "1N-3C transfer to
diamonds", 4♦ "Texas", 2NT "8-9" and 3NT "9-15".

| scenario | calls | auctions | contracts |
|---|---|---|---|
| We_Overcall_NT_then_Stayman | 75.3% → **87.3%** | 10.6% → 46.6% | 11.4% → 65.8% |
| We_Overcall_NT_then_Texas | 70.5% → **80.8%** | 0.0% → 18.6% | 0.0% → 22.8% |
| We_Overcall_NT_then_Jacoby | 70.0% → **79.5%** | 0.4% → 20.2% | 1.4% → 26.6% |
| We_Overcall_1N | 74.2% → **80.4%** | 15.0% → 26.2% | 15.4% → 33.4% |
| After_1x_1N | 73.3% → **75.7%** | 13.6% → 16.2% | 13.6% → 19.6% |

## Accepted differences from BBA

- BBA passes some 12-14 HCP hands with a five-card minor over a major
  opening that we overcall at the two level, and overcalls some that we
  pass. The line is not a rule in BBA; it evaluates the hand.
- BBA's weak jump overcall runs to 10 total points; ours stops at 9, the
  same as our weak two.
- BBA sometimes preempts to the three level on a six-card suit.

## Gaps and open questions

The balancing seat is `balancing.bid`; responsive doubles are
`responsive-doubles.bid`.

- Overcalls of a 1NT opening, of a weak two, and of two-level openings.
- Two-suited overcalls (Michaels, unusual notrump) are off on this card
  but the fields exist.
- Overcalls of a 1NT opening, of a weak two, and of two-level openings.
