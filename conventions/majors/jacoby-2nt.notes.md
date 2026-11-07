# Jacoby 2NT (`jacoby-2nt.bid`)

The game-forcing raise of a major (`major_openings.jacoby_2nt.play`),
and opener's standard answers: 3x a singleton or void, 4x a good
five-card side suit, 3M 18+, 3NT 15-17, 4M a minimum, all without
shortness. Modified Jacoby has its own answers
(modified-jacoby-2nt.bid, `modified`). Cases: `jacoby-2nt.test`.

## Responder after the shortness (2026-11-07)

Rick, 2026-11-07 (bridge-classroom #83/#84, of modified Jacoby, "and
the standard jacoby 2nt, and splinter bids"):

> "keycard should be the stronger hand. the idea of the relays is it
> will allow us to find low HCP slams when there is a good fit. one of
> the hands had 4-5 weak spades in responder's hand, and opener had a
> spade void, i think, which means all the combined HCP are 'working',
> which is the goal of this system (and the standard jacoby 2nt, and
> splinter bids). this may be a concept we haven't introduced yet."

Over 3x, responder counts his working values (slam-entry.bid,
`working_slam(M)`: support points less his K, Q, J opposite the
singleton, plus opener's floor; 27 opposite a singleton, 23 opposite a
void, though standard Jacoby's 3x does not say which) and with them
asks for keycards (RKCB, else Blackwood). Without them the slam modules
decide as before (control bids, game). Responder is the one who asks:
opener's strength is still open (10-21), responder's is too, and
responder is the one who knows where the waste is.

**No explicit sign-off.** A 4M "game: honours wasted or too few" below
the ask lost opener's extras: opener, who has not limited his hand,
did not go on over it. Measured (compare `--par-all`, the 5,000-board
Jacoby and splinter set, standard card; errors_diff, the bidding side's
contract errors): with the sign-off −118 (26/28) and −726 (30/31);
without it **+99 IMPs** (halves +46 / +53, agree; distance from par
+99, 209 boards changed). "Short of slam" 1,021 → 981 boards
(12,559 → 12,051 IMPs); "overbid" 433 → 461 (6,111 → 6,524).

## Opener after responder's 4M: not the 19+ pass (2026-11-07)

Rick's ruling of 2026-11-07 is about modified Jacoby (quoted in
modified-jacoby-2nt.notes.md): "normally opener doesn't continue for
slam exploration after responder chooses not to try, unless opener has
19+ points". It is **not applied to standard Jacoby**: here opener's
shortness answer (3x) is unlimited, 10-21, so responder's 4M is not a
decline opposite a limited hand, and opener with extras still goes on
through the slam modules (rkcb-1430's ask on `slam_values`) as before.
Tried and reverted the same day: the 19+ pass here cost −887 IMPs on
the 5,000 Jacoby/splinter boards (contract errors, halves −307 / −580;
opener's 17-18 slams, `P 4H P : 4N -> P` −675 on 207 boards).

Open question for Rick: should standard Jacoby have a line of its own
(say 17+ to go on over 4M), or stay as it is?

## Sources

- Jacoby 2NT: Oswald Jacoby's game-forcing raise; the answers as
  ACBL's SAYC card and the convention-card spec
  (`spec/conventions/bidding_conventions/jacoby_2nt.toml`) give them;
  standard practice, not yet cited beyond that.
- Working values after the shortness: Rick, 2026-11-07, quoted above;
  the threshold is our calibration (slam-entry.notes.md, "Working
  values").
- Opener over 4M: Rick's 19+ ruling (2026-11-07) is modified Jacoby's
  and not applied here (above).
