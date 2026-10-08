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

## Convention score (2026-10-07)

`probes/tools/conv_ab.py --only inverted-minors` (Inverted_Minors, 500
boards): -1.11 IMPs per changed board before (Rusty -323, BBA +208, 479
boards), -0.14 after (Rusty +142, BBA +208; halves +24/-90, differ),
fair. What was wrong:

- **Responder stopped short of game.** Over opener's 2NT (12-14
  balanced) responder bid 3NT only from 13; the 11-12 counts the scenario
  deals passed 2NT, where 3NT makes about half the time with the
  nine-card minor fit (corpus DD: 11 HCP 49%, 12 HCP 54%), and our own
  off run (2NT invitation, opener accepts from 13) reached it. Now 3NT
  from 12, or 11 with a fifth trump; over opener's 3m (minimum,
  unbalanced) 3NT from 12 with both majors held.
- **Opener's 4m slam try was passed**: responder had no rule there and
  played 4m with 26-29 HCP between the hands (5m made on 14 of 15). Now
  5m declines; the keycard rules take a hand with more.
- **Opener bid 3NT over the weak 3m with 14** (rebids.bid's acceptance
  of the standard limit raise). Opposite 5-9 game now needs 18: 3NT
  balanced, 5m otherwise, else pass.
- Opener had no rule after 2NT-3m or a splinter-4m: pass.
- The off run (no inverted minors) had no response with 11-12, four
  diamonds and five clubs over 1♦ on a 2/1 card: a limit raise 3♦ now
  (responses.bid). That made our off run better, so it lowered our gain
  (from +192 to +142) without changing any inverted auction.

Tripwire (`compare --limit 50`, all scenarios): the 3NT-from-12 changes
other inverted-minor scenarios (1m-2N 17 boards -28 errors,
Vics_Bal_Resp_to_1m 7 boards -30): a coin flip on those deals. Kept,
as par on the Inverted_Minors deals favours it; a question for Rick.

## Sources

- **The convention:** inverted minor raises. The structure is modelled on
  BBA's treatment, learned by probing; otherwise standard practice, not
  yet cited to a book or article. The game thresholds after opener's
  rebid (12 opposite 2NT, 18 opposite the weak raise) are par on the
  Inverted_Minors corpus deals (2026-10-07), not a cited source.
- **Rick's rulings:** none recorded for this module.
- **BBA probes:** `probes/minor-resp-inv-1C.toml`,
  `probes/minor-resp-inv-1D.toml`, `probes/inv-opener-1C.toml`,
  `probes/inv-opener-1D.toml`; without inverted minors (Basic-Bridge),
  `probes/minor-resp-std-1C.toml`. Opener's meanings are BBA's alerts.
- **Corpus measurements:** 2026-09-25, with the Basic-Bridge fixes
  (+829).
- **Where we differ:** opener's stopper bids are simpler than BBA's (the
  cheapest stopper with 15+ or an unbalanced hand), so opener agrees
  about 42% of the time.
