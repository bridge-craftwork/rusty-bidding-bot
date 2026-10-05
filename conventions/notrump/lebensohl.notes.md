# lebensohl (`lebensohl.bid`): notes

Lebensohl after a natural 2♦/2♥/2♠ overcall of our 1NT, on with
`notrump.lebensohl.over_interference` (BBA "Lebensohl after 1NT": on
for every 21GF card, off on Basic-Bridge). Cases: `lebensohl.test`.

## What is played (2026-10-05)

Larry Cohen's "slow shows" Lebensohl, which is also how the PBS
`Lebensohl` scenario describes it:

| responder | meaning |
|---|---|
| 2♥/2♠ | natural, to play (nt-interference.bid, unchanged) |
| 2NT | relay to 3♣: weak with a long suit below theirs, invitational with a five-card major above theirs, or game with a stopper (3NT or Stayman next) |
| 2NT–3♣–P / lower suit | to play |
| 2NT–3♣–higher major | invitational, five cards; opener 4M with three and 16-17, 3NT with two and 16-17 |
| 2NT–3♣–their suit | Stayman with a stopper, forcing to game |
| 2NT–3♣–3NT | game with a stopper |
| direct 3-level new suit | natural, forcing to game (the jumps to 3♥/3♠ too) |
| direct cue-bid | Stayman without a stopper, forcing to game |
| direct 3NT | game without a stopper |
| X | unchanged: the negative double of nt-interference.bid |

Opener answers Stayman with the unbid four-card major, else 3NT (after
the fast cue only with their suit stopped), else four of his longer
minor; responder then bids five of it.

Only a natural overcall (`rho.y>=4`, not both majors) gets Lebensohl.
Their 2♣, their double and the conventional two-suiters keep the
calls in nt-interference.bid.

Without the switch the natural structure stays (2NT invitational, 3NT
any game): it was fitted to BBA on Basic-Bridge.

## Deviations

- **The negative double and Stayman.** With game values and four of an
  unbid major we cue-bid (fast or slow) instead of doubling: the cue is
  given priority over the negative double. BBA doubles with these hands
  (1N 2♠ X then 3♥/4♥, 14+ boards in the Lebensohl scenario) and uses its
  direct cue for "game values, no stopper" without a major. We follow
  the cited source; a question for Rick.
- **Invitational balanced hands** have no call (2NT is the relay): they
  double with a four-card major, else pass. Cohen: the price of Lebensohl.

## Scenario check (2026-10-05, `compare Lebensohl Lebensohl2 --limit 100`)

At the calls after `1N 2x` agreement went 90 → 79 of 164: the loss is
the double/cue difference above (BBA X, ours 3♠/2NT) and BBA's 2♠
then 3♠ with an invitational five-card spade suit where we relay. The
rest of the divergences are in the Cappelletti 2♦ branch (both majors),
which is not Lebensohl.

## Open questions

- Should the negative double take game-forcing hands with a four-card
  major (BBA), leaving the cue-bid for "no stopper"? Decided: no, the
  source's Stayman cue.
- `Lebensohl after 1m` (a .bbsa key with no field) is not mapped and
  not guessed at.
- Over their 2♣ with `vs_2c` off, Lebensohl is not applied: their 2♣
  keeps the natural structure.

## Sources

- Larry Cohen, "Lebensohl", https://www.larryco.com/bridge-articles/lebensohl
  (slow shows, fast denies; the structure above).
- PBS `btn/Lebensohl.btn` chat text (same structure).
- Robert S. Todd, "Responding to 1NT in Competition - Lebensohl",
  https://www.advinbridge.com/this-week-in-bridge/545 (cited in
  convention-card `competitive_bidding/lebensohl`; not re-read here).
