# The balancing seat (`balancing.bid`): notes

They opened, partner passed, RHO passed. Cases: `balancing.test`.

## Guidance (Rick, 2026-09-22)

Follow BBA's meanings, which are the usual lighter balancing ranges:

- **reopening double** 8+ total points with three or more cards in every
  unbid suit (or 18+ with any shape, as in the direct seat);
- **a suit at the one level** 8-15 total points with five cards, **at the
  two level** 10-15;
- **1NT** 12-15 balanced with a stopper — `nt_overcalls.balance.range_min`
  and `_max`, not the 15-18 of the direct seat — and **2NT** 19-21.

A balanced 16+ is too strong for the balancing notrump and doubles first.

## Evidence

`bba-cli --all-meanings` on the `Balancing` scenario (21GF cards, 5,197
calls). Grouped by the opening they reopened over:

| call | BBA's meaning |
|---|---|
| X | "reopening double", 8-37 total points, 3+ in each unbid suit |
| 1NT | "NT style", 12-15 total points, 2-5 in every suit |
| 2NT | "NT style", 19-21 |
| suit, one level | "bidable suit", 8-15, 5+ |
| suit, two level | "bidable suit", 10-15, 5+ |
| cue bid | Michaels (not on our card) |

Advancer needs no new rules: the advance contexts in `takeout-double.bid`
and `advances.bid` now carry the balancing spelling of their patterns
(`(1x) P (P) X (P)` beside `(1x) X (P)`), and the notrump modules carry
`(1x) P (P) 1N (P)`, so systems are on over the balancing notrump too.

Effect: the `Balancing` scenario gained 6.7 points of call agreement,
11.0 of identical auctions and 13.4 of contracts; Too_Strong_for_Overcall4th
+4.1 calls; Trap_Pass_Opener +1.9 calls and +6.2 auctions. Summed over
the 46 scenarios that moved: calls +29.8, contracts +11.0, auctions -6.8
(a handful of weak-notrump scenarios where BBA passes the hand out).

## Gaps and open questions

- **Jump overcalls in the balancing seat** are not written: a jump there
  is natural and intermediate, not the weak jump of the direct seat.
- Reopening over a **two-level or preemptive opening**, and the balancing
  double of 1NT.
- Michaels and the unusual notrump in balancing (`competitive.michaels`
  is off on our cards).
- Partner should read a reopening double as lighter than a direct one;
  the advance rules use the direct-seat ranges for both.
