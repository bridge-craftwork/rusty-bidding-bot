# Advancing an overcall (`advances.bid`): notes

Advancer's call after partner's one-level overcall and a pass by RHO.
Cases: `advances.test`.

## Guidance

Rick has not ruled on the advance yet; the ranges are BBA's Basic-Bridge
meanings, which are the usual ones:

- **raise** 7-11 total points with three-card support;
- **new suit** 8+ with four cards at the one level, 9+ with five at the
  two level;
- **1NT** 8-11 balanced with a stopper in their suit, denying an unbid
  four-card major and three-card support for partner's major;
- **jump raise** weak: four-card support and less than 8
  (`overcalls.responses.jump_raise`);
- an overcall is not forcing, so advancer **passes** with less.

## Evidence

From `bba-cli --all-meanings`, grouped over all four opening suits:

| call | BBA's meaning |
|---|---|
| raise | "calculated bid", 7-12 total points, 3+ support |
| new suit, one level | "bidable suit", 8-21, 4+ |
| new suit, two level | "bidable suit", 9-21, 5+ |
| pass | 0-12 total points (0-16 over a one-level overcall) |

## Gaps and open questions

- **Advancing a 1NT overcall**: BBA plays systems on (Stayman 2♣ and
  transfers 2♦/2♥ appear in the corpus). Nothing fires after
  `(1x) 1N (P)` yet, which is about 3,600 corpus boards.
- Advancing a **two-level or jump overcall**, and advancing when RHO has
  bid or doubled.
- The **cue-bid raise** of their suit (limit raise or better), and the
  support cue bid (`overcalls.responses.support_cuebid`).
- Advancer's continuations after the overcaller rebids.
