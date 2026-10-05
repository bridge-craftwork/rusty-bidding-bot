# Raptor 1NT (`raptor.bid`)

Switched on by `competitive.raptor_1nt.play` (no stock card, no PBS
scenario). Cases: `raptor.test`.

## What it plays (2026-10-05)

The direct 1NT overcall of a one-level suit opening: over 1♣ five+
diamonds and four of a major, over 1♦ five+ clubs and a four-card major,
over 1♥ four spades and a five+ minor, over 1♠ four hearts and a five+
minor. 8-15 HCP (10+ vulnerable). The natural 15-18 1NT overcall is
off (overcalls.bid, `!raptor`): a strong balanced hand doubles (17+ as
the power double) or passes.

Advancer: over a minor, the known minor at the cheapest level, a weak
five-card major to play, three of the minor to invite, the cue bid
(11+) asks for the major. Over a major, two of partner's major with
three, 3M to invite with four, 2♣ pass-or-correct for the minor, the
cue bid (11+) asks for the minor.

## Deviations and open questions

- Strength is not fixed by the sources (Wikipedia: "a matter for
  partnership agreement"). We chose an overcall's range.
- A 15-16 balanced hand without the shape for a takeout double now
  passes over their opening (no natural 1NT). Question for Rick: let a
  balanced 15-18 double regardless of shape when Raptor is on?
- The Unusual 1NT advances (two-suited-overcalls.bid, priority 2) also
  read partner's known suits after a Raptor 1NT; the Raptor advances sit
  at priority 3 above them.

## Compare (2026-10-05, `--set competitive.raptor_1nt.play=true`)

We_Overcall_1N, 50 boards: the strong balanced hands that BBA overcalls
1NT with now double (17+) or pass (15-16): the cost of the convention,
recorded under the open question above.

## Sources

- Wikipedia, "Raptor (bridge)": the suits shown over each opening.
- convention-card `spec/conventions/competitive_bidding/raptor.toml`.
- Advances: standard practice, not yet cited.
