# pick-a-slam (`pick-a-slam.bid`): notes

Pick-a-slam 5NT (2026-10-05), gated on `slam.pick_a_slam_5nt.play`.

## What the rules do

- **The ask:** a jump to 5NT (partner's and RHO's last bids below 4NT),
  no ask pending (a stopper ask aside), 33 total points and 30 HCP on
  partner's floor, and two eight-card fits known: a real choice of
  strains. (Without the HCP floor a 10-count with an eight-card suit
  asked opposite a strong raise: Michaels_and_Unusual 21.)
- **The answer:** six of the suit with the most cards between us (spades
  first on a tie), or 6NT with no known eight-card fit. The asker
  passes.

## Gaps and open questions

- Our system rarely shows two fits before the five level (1♠-2♣-3♣-3♠ is
  a stopper ask here, not Todd's preference), so the ask is rare.
- Not built: 5NT pick-a-slam after their preempt (1♥-(4♠)-5NT) or over a
  1NT opening (where 5NT is the invitation to seven).
- The answerer does not see the asker's fits (his `shows` is a
  disjunction), so he picks from what he knows himself.

## Sources

- **The convention:** Robert S. Todd, "Slam Bidding: 5NT Choice of
  Slams", Advancing in Bridge #535 (a jump to 5NT, not a keycard
  follow-up, asks partner to choose; six of his preferred suit or 6NT).
  Convention-card `bidding_conventions/pick_a_slam_5nt.toml` (Kantar, *25
  More Bridge Conventions You Should Know*, ch. 23).
- **Where we differ:** we require two eight-card fits before asking.
