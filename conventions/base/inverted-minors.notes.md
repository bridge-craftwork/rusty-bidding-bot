# inverted-minors (`inverted-minors.bid`): notes

## Why (2026-09-25)

The card field `minor_openings.inverted_minors.play` was mapped from
`.bbsa` but read by nothing. On 21GF-DEFAULT (inverted minors on),
responder had no rule with a long fit and game values: about 600
corpus boards after 1C-P and 1D-P where BBA bid.

## BBA's treatment (probes/minor-resp-inv-1C.toml, 1D; inv-opener-1C, 1D)

**Responder:**
- 2m: 10+ HCP, four or more of the minor, no four-card major, forcing.
- 3m: weak, 5-9 with five or more.
- With four diamonds over 1C, 1D comes first.
- 1NT over 1m gives way to the raise with five of the minor.

**Opener after 1m-2m** (BBA's meanings):

| Call | Meaning |
|---|---|
| 2D/2H/2S | "stopper", up the line; the usual call with 15-17 |
| 2NT | balanced, 11-15 (12-14 in practice) |
| 3m | minimum, five or more |
| 3 of a new suit | shortness (splinter), four trumps |
| 3NT | 18-19 balanced |
| 4m | slam try, 16+ |

**Ours:** the same frame, with simpler stopper bids: the cheapest
stopper when opener has 15+ or is unbalanced. Responder then bids 2NT
(10-12 balanced), 3NT (13+) or 3m. Agreement with BBA: responder
65-78%, opener ~42% (BBA's stopper choice is finer).

Corpus, together with the Basic-Bridge fixes below: -163,528 ->
-162,699 (+829).

## Without inverted minors (Basic-Bridge, probes/minor-resp-std-1C.toml)

- **3m limit raise:** now 10-12 HCP. Total points passed six clubs with
  10-12.
- **3NT:** the fallback with 13+, a fit and no major, as BBA. We had no
  call.
