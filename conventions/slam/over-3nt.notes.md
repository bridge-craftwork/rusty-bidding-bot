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

## Responder after opener's new suit (2026-09-27)

The biggest group short of slam was responder's 3NT after opener's
second suit: 1a–1b–2c–3NT (772 boards), 1a–1b–1c–3NT (489),
1a–2b–2c–3NT (556). BBA reaches slam on well over half of them. The
same 6NT / quantitative 4NT now apply at responder's second call
after 1x–1y–2z, 1x–1y–1z and 1x–2y–2z (new suits).

It counts **points**, not HCP: a jump shift shows 19+ suit points, which
leaves opener's HCP floor at 10, so an HCP count never saw it. The
first try (34/32 HCP, the numbers used over 3NT) changed 10 boards.

| 6NT / 4NT, in points | full corpus (par / side) | boards |
|---|---|---|
| 34 / 32 | +189 / +515 | 48 |
| **33 / 31 (kept)** | **+504 / +1,133** | 108 |
| 32 / 30 | +993 / +1,892 | 196 |
| 31 / 29 | +1,624 / +2,886 | 342 |

The lower thresholds keep winning, and I do not trust it: the scenarios
are dealt for slam (Soloway, Minor_Game_Or_Slam, Fourth_Suit_Forcing).
On the boards where 6NT fires, at 33 it almost always holds 32+ real
HCP between the hands, and makes double dummy every time; at 31 it
fires on 29-31 HCP, where 6NT makes 17-69%. The textbook 33 stays.
**For Rick:** the rest of the gap is that opener's strength after a
simple new-suit rebid is 12-18, too wide to count on. BBA gets there by
exploring (3m, fourth suit, then 4NT); we have no slam try below 3NT
yet in these auctions.

A first attempt put the rule in one context for every uncontested
position below game. It overrode the notrump auctions' own slam
counting (Stayman, transfers, 1NT–4NT): −11,422 par, −14,314 side. The
contexts are now named auctions.

## A standalone major over partner's 3NT (2026-09-27)

Ticket basic-weak-2-b194 (Rick): 1S 2D 2S 3NT with AKQJ63.32.6.T542
should go back to 4S: the suit needs no help to keep trump control, and
the weak heart doubleton is a risk in notrump. Standalone: six to the
AKQJ or seven to the AKQ; plus a doubleton or shorter without a
stopper. +615 par, +969 side IMPs (420 boards). Without the unguarded
condition: +601 / +942 on 603 boards, so it stays.

## With Blackwood or RKCB, 4NT is keycard here (Rick, 2026-10-03)

> A 4NT with no prior NT bids is normally keycard regardless, and if
> there hasn't been suit agreement, it's for the last bid suit.

The quantitative 4NT after 1x-1y-1z, 1x-1y-2z and 1x-2y-2z (no notrump
bid) now needs `no_ask`: on cards with an ace or keycard ask, 4NT there
is the ask in z (rkcb-1430.notes.md, "The keycard suit"); on cards with
neither (bare SAYC) it stays quantitative. The 6NT beside it is
unchanged. Measured with the rest of that change.

## Sources

- **The thresholds:** the textbook combined counts for 6NT and a
  quantitative 4NT (33 HCP for slam) are standard practice, not yet cited
  to a book or article. The numbers in the rules are calibrated on the
  corpus's double-dummy tables ("Calibration"), one point above the
  textbook where partner is counted at his minimum.
- **Rick's rulings:** ticket basic-weak-2-b194 (2026-09-27): a
  standalone major goes back to four of it over partner's 3NT.
- **BBA evidence:** the 14,834 corpus boards where BBA bid slam and we
  did not, and BBA's exploring sequences after a new-suit rebid ("For
  Rick"). No `probes/*.toml` spec yet.
- **Corpus measurements:** the threshold tables (2026-09-27) and the
  rejected single-context version (-11,422 par).
- **Where we differ:** responder's thresholds stay at the textbook 33,
  although lower ones score better on the slam-dealt scenarios.
- **Rick's ruling, 2026-10-03:** 4NT with no notrump bid is keycard
  for the last suit on cards that play an ask.
