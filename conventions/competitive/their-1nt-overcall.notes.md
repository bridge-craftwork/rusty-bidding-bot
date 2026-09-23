# their-1nt-overcall (`their-1nt-overcall.bid`): notes

Responder when RHO overcalls partner's one-level opening with 1NT.
Cases: `their-1nt-overcall.test`.

## Why it is here and not in `competitive/`

`after-interference.bid` writes every one-level context as
`after 1x (1y)`, and `1y` binds a *suit*, so no rule in it matched a
notrump overcall: responder passed whatever he held. `1D (1N)` and
`1C (1N)` were the two largest no-rule points in the whole corpus
(1,862 and 1,790 boards), and the four `1x (1N)` points together were
5,464 boards of dead auction, 20,461 with everything downstream.

The module sits in `notrump/` because that is the slice this piece of
work owned. **It arguably belongs beside the other contested-auction
rules in `competitive/after-interference.bid`** — it is responder's
first call over interference, which is exactly that module's subject.
Moving it is a copy of one `after` block; nothing here reads a notrump
card field. (For Rick.)

## Guidance

Their 1NT is 15-18 balanced with a stopper. Partner opened, so the deck
is accounted for: with 10+ total points we own the balance of the
strength and double for penalty. Below that it is an ordinary
competitive auction, one level higher than usual.

- **X is penalty, 10+ total points**, and denies a five-card major or a
  six-card minor — with one of those, bid it.
- **A raise of partner's suit is 6-9 support points** with three-card
  support for a major, five for a minor. It comes before naming a suit
  of our own.
- **A new suit at the two level is 9+ total points**, five cards in a
  major, six in a minor.
- **3♣ / 3♦ with seven cards and 5-9** total points: a preempt.
- Everything else passes.

## Evidence

`bba-cli --all-meanings` on `Opps_Overcall_1NT` (NS play 21GF-DEFAULT,
which is the card on 199 of the 342 corpus scenarios), 406 decisions at
`1x (1N)`:

| BBA's call | boards | its stated meaning |
|---|---|---|
| Pass | 287 | |
| 2♠ / 2♥ | 75 | "bidable suit", 7-12 total points, 5+ cards |
| 2♠ / 2♥ in partner's major | (of those) | "calculated bid", 6-9 total points, 3+ cards |
| X | 21 | "penalty", 10-37 total points |
| 2♣ / 2♦ | 12 | "bidable suit", 7-12 total points, **6+** cards |
| 3♣ / 3♦ | 5 | "bidable suit", 5-7 total points, 7+ cards |
| 4♠ / 4♥ / 3♥ / 3♠ | 6 | preemptive |

**The declared range is not the range BBA bids.** The note on the suit
bid says 7-12 total points, but BBA passes most 7s and 8s: a grid search
over the 406 decisions puts the floor at **9**.

| new-suit floor | decisions matched (of 406) |
|---|---|
| 7 (BBA's stated range) | 344 |
| 8 | 363 |
| **9** | **366** |
| 10 | 363 |

The raise band and the double threshold were searched the same way:
raise 6-9 and X at 10+ give 366; a wider raise (4-11) gives 371, two of
which are boards where BBA raises on four points. 6-9 is what BBA
declares and what the rule promises, so that is what is written.
Passing everything, the state before this module, matched 287.

## Accepted differences from BBA

- **BBA sometimes bids a poor five-card suit on 7-8 total points and
  sometimes passes an eleven-count with five spades** (`AQ753.K732.Q.932`
  over 1♦: BBA passes). Nothing in shape, suit quality (top honours) or
  vulnerability separated the two groups in the 406 decisions. The 9-point
  floor is the best single cut; it costs about 12 boards where BBA bids
  and we pass.
- **No preemptive jump raise.** BBA's 4♠ over their 1NT was a shapely
  11-count, not a weak five-card raise, so there is no `4{x}` rule. Three
  boards.

## Measured effect (2026-09-23, whole corpus)

At the `1x (1N)` decision itself: **72.8% → 86.5%** of calls agree
(3,240 → 3,850 of 4,452). Corpus-wide this module plus the 1NT
interference work is +1,991 calls, +186 boards with the same contract,
+209 identical auctions and −4,476 no-rule points; `1x (1N)` alone
accounts for the 5,464 dead second calls, and the whole `1x 1N…`
subtree drops from 20,461 no-rule points to 16,092.

## Gaps and open questions

- **Opener's rebid is now the dead end.** Responder bidding creates a
  new no-rule point one call later: `1H (1N) 2H` +168 boards,
  `1S (1N) 2S` +148, `1D (1N) 2S` +140, `1C (1N) 2S` +126,
  `1D (1N) 2H` +110, `1C (1N) 2H` +109 — about 800 boards in all,
  every one of them opener with nothing written for him. This is the
  same "opener's rebid in a contested auction" gap
  `after-interference.notes.md` already lists as the biggest remaining
  cluster, and it is not ours to write.
- **Their 1NT overcall is not interpreted as 15-18 with a stopper.** We
  have no rule for the opponents' 1NT overcall either, so the engine
  reads it as an unknown call and `partner`/`rho` knowledge stays wide.
  The point counts here are ours alone; a rule for their overcall (in
  `competitive/overcalls.bid`) would let responder count the deck
  properly.
- **A balancing 1NT overcall** (`1x P P 1N`) is a different animal and is
  not covered: the pattern here is only the direct seat.
- **Their 1NT over a two-level or higher opening** is not covered.
