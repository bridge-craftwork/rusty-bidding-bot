# western-cue (`western-cue.bid`): notes

Western cue bid (2026-10-05). Gated on `slam.western_cuebid.play` (the
card files it under slam; the module lives in competitive/ because it
is a competitive tool, in a file of its own). No stock card plays it,
the `.bbsa` format has no key for it, and BBA does not play it (the
Western_Cue_Bid scenario says so).

## What the rules do

- **The ask:** both of us have bid, they have bid a suit (their last
  bid), no suit of ours agreed but a minor, game values between us (25
  on partner's floor), no major fit known, and no stopper of mine in
  their suit: the cheapest bid in their suit at the two or three level
  asks partner for a stopper. It forces to game. Priority 2.
  1♦ (1♠) 2♣ (P) 2♠; 1♥ (2♣) 2♦ (P) 3♣; 1♦ (1♠) 2♣ (2♠) 3♠.
- **The answer:** 3NT with a stopper; without one, the natural
  descriptive bid (a five-card suit, or support making seven cards with
  partner's shown length, the longer fit first); 3NT anyway when there
  is nothing to show.
- **The asker:** passes 3NT; over a suit, game in a major fit or five of
  the best minor fit.

## Deviations, open questions

- The two-level cue (1♦ (1♠) 2♣ (P) 2♠) is included: once both of us
  have bid it is never the first-round cue raise of
  after-interference.bid, and asking for the stopper is the usual
  meaning. The scenario speaks of "3 of opponent's suit" only.
- No partial-stopper answers (Qx, Jxx: bid 3NT if partner has half),
  and no Western cue in uncontested auctions (a cue of a suit we bid,
  "directional asking bid"). **Question for Rick:** wanted? Decision
  taken: competitive only, as the source and the scenario.

## Corpus

Western_Cue_Bid `--limit 50 --set slam.western_cuebid.play=true`: the
cue came up on 2 boards (a spade and a diamond stopper ask). BBA does
not play it, so agreement there is not a measure.

## Sources

- **The convention:** Robert S. Todd, "Thinking and Responding:
  Western Cuebids", Advancing in Bridge #510
  (https://www.advinbridge.com/this-week-in-bridge/510): a cue bid of
  their suit asks partner to bid notrump with a stopper, otherwise to
  make a natural descriptive bid (1♠ (2♣) 2♦ (P) 3♣). Convention-card
  `spec/conventions/bidding_conventions/western_cuebid.toml`. The PBS
  scenario `Western_Cue_Bid` (after 1x (y) z (P or raise), three of
  their suit asks for 3NT with a stopper, else scramble).
- **Where we differ:** the game-values test (25 on partner's floor) and
  the answer without a stopper are ours.
