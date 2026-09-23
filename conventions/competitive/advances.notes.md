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

- Advancing a **two-level or jump overcall**. Advancing when RHO doubles
  or bids is written, but only the raises, 1NT and a five-card suit at
  the one level; there is no free bid at the three level.
- Advancer's **cue-bid raise** of their suit. Responder's is written
  (`after-interference.bid`); advancer's is not, nor the support cue bid
  (`overcalls.responses.support_cuebid`).
- Advancing a **balancing** overcall uses the direct-seat ranges, though
  partner is 8-15 there rather than 8-17.

Advancing our 1NT overcall needs nothing here: systems are on, and the
notrump modules carry the `(1x) 1N (P)` and `(1x) P (P) 1N (P)`
spellings of their patterns (see `overcalls.notes.md`).
- Advancer's continuations after the overcaller rebids.
