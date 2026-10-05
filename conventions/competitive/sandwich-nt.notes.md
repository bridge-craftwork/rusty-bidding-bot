# Sandwich 1NT (`sandwich-nt.bid`)

Switched on by `competitive.sandwich_nt.play` (no stock card). Cases:
`sandwich-nt.test`.

## What it plays (2026-10-05)

(1x) P (1y) 1NT, they have bid two suits: the two unbid suits,
five-four or better, 8-15 HCP. The passed hand's version is already the
Unusual 1NT (`competitive.unusual_1nt.play`, two-suited-overcalls.bid,
on in every 21GF card); with both switches on, the Unusual 1NT keeps the
passed hand and Sandwich 1NT adds the hand that has not passed. With
only Sandwich on it covers both. Advances: the Unusual 1NT's (the
longer suit, lower with equal length; a jump with four and 11+), whose
context now also reads this switch.

## Deviations and open questions

- Todd (Advancing in Bridge 194) introduces it as a passed hand's bid
  ("or even direct seat by agreement"); the field has no seat option.
  We play it by any hand in the sandwich seat. Question for Rick: limit
  it to passed hands (then it is the Unusual 1NT)?
- With 16+ the hand passes: there are no other sandwich-seat actions for
  an unpassed hand yet ((1x) P (1y) X or overcalls). That is a gap in the
  base competitive rules, not in this convention.
- GIB_Sandwich_NT_BPH (passed hand) is the Unusual 1NT's scenario;
  BBA hardly bids it (two-suited-overcalls.notes.md).

## Compare (2026-10-05, `--set competitive.sandwich_nt.play=true`)

GIB_Sandwich_NT_BPH is a passed-hand scenario, so it shows the Unusual
1NT as before (BBA overcalls 1♠ or passes); nothing new.

## Sources

- Robert S. Todd, "Competitive Bidding: Sandwich NT", Advancing in
  Bridge 194 (https://www.advinbridge.com/this-week-in-bridge/194):
  the idea; details are in the lesson files, not read.
- Wikipedia, "Unusual notrump": the sandwich 1NT shows the two unbid
  suits.
- Lengths and strength: standard practice, not yet cited.
