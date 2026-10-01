# game-force: when the auction forces to game

Rick, 2026-09-30: "we should have a state 'in game force' that 2/1
triggers, or 4th suit forcing, or xyz 2d, etc. there are also some
standard bidding scenarios - i'm not sure there is 100% agreement, but
like a 3rd bid by responder at the 3 level or higher is usually
considered a GF. Also 1x - 1y - 2NT, anything but pass is usually a GF.
There is a convention where 3C there is to stop in part score (Wolff?)."

The state is `we.forcing = game` (it existed). What was missing is what
creates it:

| trigger | where | status |
|---|---|---|
| a call whose meaning forces (2/1 on a 2/1 card, strong jump shift, reverse ...) | the rule's `sets forcing=game` | 2/1 built 2026-09-30 (responses.notes.md) |
| fourth suit forcing, game-forcing variant | fourth-suit-forcing.bid, the card's `fourth_suit_forcing.game_force` | built 2026-09-30 |
| XYZ 2D | a new module | not built |
| responder's third bid at the three level, below game (3C-3S), uncontested | `force game` here | built |
| after 1x-1y-2NT, anything but pass (Wolff's 3C excepted) | `force game` here | built |

`force game` (engine 0.4.0, LANGUAGE.md §3) is a declaration, not a rule:
it applies whichever rule made or explains the call, because these forces
belong to the shape of the auction.

## Measured (2026-09-30)

Random deals (20,000 Pavlicek, 21GF both sides) +23, corpus +7 IMPs vs
BBA, double-dummy par as the yardstick; no new "no rule" or broken-force
boards. Small because few auctions reach these shapes without a rule that
already forces; the declarations matter for the rules written against
`we.forcing = game` (control bids, the keycard ask, fast arrival).

## Open

- "Usually considered a GF": exceptions some partnerships make
  (responder's third-bid preference to opener's minor at the three
  level) are not carved out; say if they should be.

## Sources

- Rick's rulings, 2026-09-30 (quoted above).
- Responder's third bid at the three level and the 2NT-rebid game force:
  standard practice, not yet cited.
