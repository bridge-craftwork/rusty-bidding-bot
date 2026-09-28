# Landy and Multi-Landy (`multi-landy.bid`): notes

Landy and Multi-Landy over their 1NT, direct and balancing seat. Cases:
`multi-landy.test`. Shared answers: `vs-1nt.bid`; the comparison of the
defences: `vs-1nt.notes.md`.

Not in Rick's list of 2026-09-28 (Cappelletti, Modified Cappelletti,
Meckwell, DONT), but seven BBA cards turn `Multi-Landy` on (21GF-Multi,
21GF-SPECIALS2, 21GF-MSS, 21GF-NoInvertedMinor, 21GF-PolishTwoSuiters,
21GF-GIB-Bergen, Precision), and until now the key only switched our
natural overcalls off, so those cards passed every hand over 1NT. The
module costs little once the shared answers exist. `.bbsa` keys
`Multi-Landy` and `Landy` were already mapped.

## The calls

Multi-Landy as BBA plays it on 21GF-Multi (`probes/vs-1N-mlandy.toml`,
1,000 hands; its alerts in quotes), which is Woolsey's version:

| Call | Shows | BBA | Advancer |
|---|---|---|---|
| X | a four-card major and a longer minor | "4M-5m", 12+ HCP | 2♣ pass or correct to the minor |
| 2♣ | both majors | "both majors", 9+ | preference |
| 2♦ | one major, six or more | "Multi", 9-17 | 2♥ pass or correct |
| 2♥/2♠ | that major and a four-card minor | "5M-4m", 9+ | as Cappelletti's 2♥/2♠ |
| 2NT | both minors | "Unusual 2NT" | the longer minor |

Landy: 2♣ both majors, the rest natural (overcalls.bid).

## Sources

- Bridge Bum, "Multi-Landy (Woolsey)", https://www.bridgebum.com/multi_landy.php
- Wikipedia, "Cappelletti convention", https://en.wikipedia.org/wiki/Cappelletti_convention
  (Multi-Landy as "reverse Cappelletti": 2♣ and 2♦ swapped).
- BBA on 21GF-Multi, `probes/vs-1N-mlandy.toml`.

Bridge-Classroom's `ntDefenses.js` lists Multi-Landy with a penalty
double and natural 2♥/2♠; that is closer to plain Landy. We follow BBA
(and Woolsey) because the BBA cards that carry the key play it that way.

## Gaps

- Woolsey's double also covers a six-card minor and strong hands
  (Bridge Bum); BBA's alert names only the 4M-5m hand, and so do we.
- The advancer's continuations were not probed; the preference and
  pass-or-correct answers are the shared ones.
- Over their Landy/Multi-Landy 2♣ (majors) our side still plays
  systems on (Stayman by double: nt-interference.bid); a cue-bid
  structure would suit it better.
