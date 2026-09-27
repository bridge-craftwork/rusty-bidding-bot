# over-3nt (`over-3nt.bid`): notes

## Why (2026-09-27)

Slam work, third group. Over the corpus BBA bid slam on 14,834 boards
where we did not (78% make double dummy). After the minor slam tries
over 1NT and 2NT, the largest remaining shapes were suit openings that
reached 3NT: 1a-2b-3NT (876 boards), 1a-1b-2c-3NT (557) and others,
where one partner bid 3NT with 15-18 balanced and the other passed with
15-17 and often a long suit. Nothing looked past partner's 3NT.

## Calibration (all double-dummy tables in the corpus)

P(a small slam makes), by HCP between the hands and longest fit:

| HCP | fit 7 (suit / 6NT) | fit 8 | fit 9 | fit 10+ |
|---|---|---|---|---|
| 29 | 32% / 19% | 55% / 28% | 72% / 37% | 83% / 39% |
| 30 | 53% / 39% | 71% / 47% | 83% / 56% | 89% / 59% |
| 31 | 65% / 59% | 80% / 66% | 90% / 76% | 95% / 80% |
| 32 | 79% / 75% | 89% / 83% | 94% / 89% | 99% / 95% |
| 33 | 91% / 92% | 94% / 92% | 98% / 98% | 99% / 97% |

Double dummy flatters slams; treat these as upper bounds.

## Rules and thresholds

When partner has just bid 3NT, no suit is agreed and they are silent:
6NT, a quantitative 4NT (answered by `when asked quant`), or 4♣/4♦
with six as a slam try (partner bids six with three-card support, else
4NT; then 6NT with the values). My HCP plus partner's shown minimum:

| 6NT / 4NT / minor try | full corpus (par / side) | ours-only slams make |
|---|---|---|
| 33 / 31-32 / 30 | +2,298 / +5,352 | 59% |
| **34 / 32-33 / 31 (kept)** | **+2,516 / +5,193** | 64% |
| 35 / 33-34 / 32 | +2,137 / +4,432 | |

Counting partner at his minimum, one point above the textbook numbers
is best.
