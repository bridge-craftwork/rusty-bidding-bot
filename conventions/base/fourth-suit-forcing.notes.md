# fourth-suit-forcing: Fourth suit forcing

Built 2026-09-30 (Rick: the game-force state is triggered by "2/1, or 4th
suit forcing, or xyz 2d"). Until then the fourth suit was natural
whatever the card said.

- **When:** after 1x-1y-1z or 1x-1y-2z (three suits), responder's cheapest
  bid in the fourth suit at the two level; not by a passed hand. The
  fourth suit at the one level (1C 1D 1H 1S) stays natural.
- **Card:** on if any of `other_conventions.fourth_suit_forcing.play`
  (Bridge-Classroom), `.one_round` or `.game_force` (the .bbsa keys);
  game forcing with `game_force`, or `play` without `one_round`
  (21GF-DEFAULT), else one round (21GF-GIB, 12+).
- **Opener answers, in order:** three cards in responder's suit; a
  stopper in the fourth suit (notrump); five of his first suit; five of
  his second; four of the fourth suit; else his first suit again.
- **Responder** then bids game in a major fit, 3NT, or (one-round only)
  passes a minimum.

BBA (probe 2026-09-30, 1D P 1H P 1S P, responder 4-4 in hearts and
clubs): on 21GF-DEFAULT 12 HCP bids 2NT and 14-15 bid 2C with or without
a club stopper; on 21GF-GIB 2C from 12.

## Measured (IMPs vs BBA, par as the yardstick; 2026-09-30)

| version | random (20,000) | corpus |
|---|---:|---:|
| **ranked with the natural calls (kept)** | **+56** | **+180** |
| ranked above them (as BBA: the fourth suit before 3NT) | -23 | -506 |
| natural 3NT promises the fourth suit stopped | +34 | +75, and new "no rule" boards (+12 / +88) |

Accepted difference from BBA: with game values and a stopper we bid 3NT
directly; BBA goes through the fourth suit.

## Open

- Responder's later calls after the answers are the basic ones (game in a
  fit, 3NT); slam tries go through the general machinery.
- The fourth suit at the three level (1D 1S 2H 3C) is not covered.

## Sources

- Rick, 2026-09-30.
- Fourth suit forcing (answer order): standard practice, not yet cited.
- BBA probe above (`rbb probe --prefix "1D P 1H P 1S P"`, 21GF-DEFAULT and
  21GF-GIB).
