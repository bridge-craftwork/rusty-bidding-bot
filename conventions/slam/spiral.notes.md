# spiral (`spiral.bid`): notes

Spiral 2NT (2026-10-05). Gated on `slam.spiral_cuebids.play`. No stock
card plays it and the `.bbsa` format has no key for it.

## What the rules do

- **The ask:** after 1X-1M-2M (opener's single raise of responder's
  major), responder's invitational hand (11-12 support points, the hand
  that would raise to three) bids 2NT instead. Priority 1 over the
  natural raise.
- **Opener's answer:** 3♣ three trumps, minimum; 3♦ three, maximum; 3♥
  four, minimum; 3♠ four, maximum; 3NT 4-3-3-3 with four trumps and a
  maximum. A maximum is 14+ HCP (the raise is 12-15).
- **Responder:** game opposite a maximum (3NT with four trumps opposite
  three and a balanced hand, or a pass of the 3NT answer with a 4-4
  fit and a balanced hand); three of the major opposite a minimum (a
  pass over 3♥ in hearts).

## Deviations, open questions

- **Which Spiral?** The field is `spiral_cuebids`; the convention-card
  spec (`bidding_conventions/spiral`) describes "a relay that asks
  about trump length and quality in steps once a fit is found" and
  cites Todd's 2NT trump-suit game try, which is what is built here.
  The older Spiral cue bid (a repeated relay over keycard answers that
  counts the top trump honours) and Wolpert's Spiral Raises (scenarios
  `Spiral_Raises_Wolpert`/`_Weinstein`: opener raises on three or four
  and the cheapest step asks shape too) are different conventions.
  **Question for Rick:** which one should the field mean? Decision
  taken: the spec's source.
- Our base system raises 1X-1M-2M with four trumps only, so the
  three-card answers come up only on cards that raise with three.
- No repeated ask: the source describes one round.

## Sources

- **The convention:** Robert S. Todd, "Fits and More: 2NT Trump Suit
  Game Try - Spiral", Advancing in Bridge #529
  (https://www.advinbridge.com/this-week-in-bridge/529): 1X-1Y-2Y-2NT,
  answers 3♣/3♦/3♥/3♠ by length and strength, 3NT with 4-3-3-3, four
  trumps and a maximum. Convention-card
  `spec/conventions/bidding_conventions/spiral.toml`.
- **Where we differ:** the maximum is set at 14 HCP, our choice (the
  source says minimum or maximum without a number); the responder's
  hand that asks (11-12 support points) is ours.
