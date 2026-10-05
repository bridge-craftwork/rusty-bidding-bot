# serious-3nt (`serious-3nt.bid`): notes

Non-serious 3NT (2026-10-05). Gated on `slam.non_serious_3nt.play` and
on control bids (`slam.cue_bids.play`): without control bids there is
nothing for the 3NT to ask for. No `.bbsa` key; scenario `Serious`
(21GF cards) is the one that teaches it.

## What the rules do

- **Where:** a game force, hearts or spades agreed, partner's last call
  three of the major, neither of us shown balanced (`our_notrump`). The
  first hand to speak after the 3M:
  - **3NT, non-serious:** within two points of the slam values that
    make a control bid serious (`ns_count`: 28 for the long-trump hand,
    29 short, 30 equal, on partner's floor), short of the values
    themselves. Shown, so partner counts on it.
  - **A control bid:** serious, control-bids.bid's own (slam values and
    a side suit uncontrolled). With every suit controlled, the keycard
    ask as before.
  - **4M:** no interest.
- **Partner over 3NT:** with extras on the raised floor
  (`slam_interest`, 31/29), the cheapest control below game, which
  opens control-bids.bid's dialogue; with slam values and every side
  suit controlled, the keycard ask (RKCB cards); else four of the major.

## Deviations, open questions

- **Thresholds:** our slam values are lower than the source's (it puts
  the non-serious 3NT about 14-15 opposite a 2/1 and the serious control
  bid 16+); with Rick's 33-point count, 14 HCP with controls is already
  serious after 1♠-2♣-2♠-3♠, and the 3NT lands on 13-14. Decision taken:
  keep the engine's slam values and put 3NT just below them.
  **Question for Rick:** OK, or should serious move up to the source's
  16+?
- **Hearts:** some play 3♠ as the non-serious bid when hearts are
  trumps and 3NT as a spade control (the source mentions it). We use
  3NT in both majors; 3♠ stays a control bid.
- Only the non-serious version: the card has no field for "serious
  3NT" (the reverse meaning).
- The Jacoby 2NT auctions (1M-2NT-3M) qualify too; there our 3M shows
  extras, so responder is usually serious.

## Corpus (Serious, --limit 50, 21GF cards, `--set slam.non_serious_3nt.play=true`)

BBA's 3NT there is "surplus" (its alert): a **serious** 3NT, the
reverse of ours. So where BBA bids 3NT we usually control-bid (6
boards: BBA style, not a bug), and our 3NT came up twice. Two boards
changed contract: after 3NT, a control bid and partner's first-round
control past game (5♣), the dialogue stalled in 5♠ with every suit
covered (boards 17, 47). Fixed in rkcb-1430.bid: with every suit
covered and the values, once partner's control bid has passed 4NT, bid
the slam. That fix also changed 7 boards elsewhere in the tripwire (the
same stall without the 3NT): six slams that make, one 6♠ one down
(SCS16_Major_Open_2-Suit_Resp 10).

## Sources

- **The convention:** Robert S. Todd, "Slam Bidding: Non-Serious 3NT",
  Advancing in Bridge #539
  (https://www.advinbridge.com/this-week-in-bridge/539): when it applies
  (a bid and supported major at the three level in a game force, an
  8+ card fit, neither hand balanced), what 3NT, control bids, 4M and
  4NT show, and partner's continuations. Convention-card
  `spec/conventions/bidding_conventions/serious_3nt.toml`.
- **Where we differ:** the point ranges, as above.
