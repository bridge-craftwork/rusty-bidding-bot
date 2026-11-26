# The bottom review (2026-11-25/26)

A review of the lower levels of the bidding (openings, responses, rebids,
the first rounds of competition) against GIB's bidding tree, before the
higher levels are rebalanced (docs/BIDDING-MODEL.md 4.3). The rules are on
pack branch `bottom-review` (bridge-common-conventions), the engine on
`freak-patterns` (this repo). Neither is merged: both wait for the
higher-level rebalance.

## How it was measured

`probes/tools/gib_tree.py agreed --card 21GF-GIB`: the positions our engine
reaches in 100,000 random deals, ranked by how often they occur, each asked
of GIB's tree (`gibrest`, archived). Ranges are compared only where both
systems give the call the same kind of meaning on a fully agreed path. The
unit is a **call-board**: one call on one deal whose range differs from
GIB's (floor, top, or a suit length). GIB is a check for flaws, not the
target (Rick, 2026-11-26): a remaining difference may be our style, a least
lie, or a call we don't have yet.

| run | positions | call-boards differing |
|---|---|---|
| before | top 5,000 | 350,431 |
| after the first pass | top 5,000 | 181,255 |
| before the second pass | top 20,000 | 230,784 |
| after the second pass (pack ba99e29) | top 20,000 | 225,477 |

The full reports are in [2026-11-bottom-review/](2026-11-bottom-review/):
`gib-top5000-*.md`, `gib-top20000-*.md`, and the per-rule before/after
lists `per-rule-pass*.txt`. Per-rule lists key rules by `file:line`, so a
rule whose line moved shows as gone and new; pass 1 moved many lines.

## First pass (top 5,000)

- **Engine:** every seat read as a normal hand pattern (freaks bid outside
  the system, BIDDING-MODEL 3.5); exact counts from length vectors; the
  `converts X` clause, so a doubler's own suit or notrump makes it a power
  double (3.6).
- **Points tops** on the catch-all passes, the openings, Drury 4M and the
  splinter (a points top on the pass partner reads).
- One-level overcall from 7 HCP. Takeout, reopening and against-preempt
  doubles read as the unbid suits (3-4 in an unbid major, 3-5 in an unbid
  minor). Negative double from 5 HCP, the redouble from 8.
- Weak twos read with no side void. The 1NT response to a major read to 13
  points; 1NT–3NT read balanced.
- The power double: suit rebids 18+, notrump 19-21 balanced.
- The long-trump hand counts HCP plus length, never support points: about
  60 `tp(x)` uses changed (3.3; the thresholds moved to GIB's numbers are
  provisional, to be settled by par).

## Second pass (top 20,000)

| change | gap before → after |
|---|---|
| Takeout double, length in opener's suit | 6,214 → 6 |
| Opener's pass after partner passed: open top (capped at 17) | 1,429 → 0 |
| Declining a minor invitation: open top | 857 → 0 |
| Competitive raises ("Competes: to play"): floor | 1,229 → 363 |
| Competitive raises opposite a weak two: no top | 909 → 880 |
| After 1♦–1♥–1♠, the 1♠ rebid's heart and diamond limits | 1,208 → 0 each |
| Opener's later pass, diamond floor | 1,240 → 903 |
| One-level negative double | 1,117 → out of the top 40 |
| Opener's sign-off over the cue-bid raise: points top | 588 → 0 |
| Pass after a negative-double auction (after-interference) | 3,586 → 2,170 |

The cap of 17 on opener's pass after partner passed first also caught
1x–P–P–X–P, opener passing a balancing double, where any strength may pass
(GIB 12-22). That made 7,295 call-boards too narrow; the pass over a
double is no longer capped (pack ba99e29).

One call changed: the negative double counted four cards in the opponents'
major as "an unbid major", so 4-3 in the minors could double 1♥ (1♠). It
now passes (Rick, 2026-09-22).

## What is left, and why

- **About 24,000:** the pass in the pass-out seat after their part score.
  It has no top because we never balanced over their fit. Being added
  (work queue, BIDDING-MODEL section 8).
- **About 6,400:** opener's passes after a game bid: higher-level work.
- **About 5,400 and 5,300:** the third-seat four-card major and weak twos
  with a four-card side suit. Our style; a `general.style=gib` will
  measure them separately (BIDDING-MODEL 3.9).
- **Passes whose stronger calls don't exist yet:** fourth seat after
  1M–P–1NT (being added), advancer over their raise, responder over
  interference after our weak two.
- **Licence and style:** Stayman 2♥ with five spades, 5-4-2-2 1NT rebids,
  Walsh 1NT, the redouble with three-card support, 2♦ after
  1♦–1♥–1♠–1NT with 4-3-5-1 (scoring-dependent, 3.6).
- **Least lies** (3.6): not fixed by design.

Pack tests at ba99e29: 3,330 cases, 3,325 pass; the 5 failures are the
known `serious-3nt.test` cases (higher-level work).
