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

## Opener passes 4M without 19+ (Rick, 2026-11-07)

Rick, of modified Jacoby, applied here too as the controller asked:

> "modified jacoby 2nt is mostly useful because responder can jump to
> game with no slam interest, after a 3C rebid by opener. the main
> advantage is that it doesn't give the defenders any clues to opener's
> distribution. so we don't want to always ask. but with say 15+ HCP,
> responder should probably ask. normally opener doesn't continue for
> slam exploration after responder chooses not to try, unless opener
> has 19+ points"

After any of opener's answers (3x shortness, 3NT, 3M, 4x) responder's
4M ends the auction unless opener holds 19+ points (`P`, priority 16,
above the keycard ask and the control bids); with 19+ the slam modules
decide as before. Before, opener with 17-18 and a singleton asked for
keycards over the sign-off (rkcb-1430's ask on `slam_values`).

Measured against the version before it (265c925; compare `--par-all`,
the 5,000 Jacoby/splinter boards, standard card; errors_diff): 406
boards changed; the bidding side's contract errors **−887 IMPs**
(halves −307 / −580, agree); distance from par −875. All of it is
opener's pass: `P 4H P : 4N -> P` −675 on 207, `P 4S P : 4N -> P` −212
on 199; "short of slam" 981 → 1,204 boards (12,051 → 14,899 IMPs),
"overbid" 461 → 369. Rick's ruling stands; double dummy cannot see
the defence's side of it, but here the cost is mostly slams opener's
17-18 used to find, so it is a question for Rick (below).

Open: with standard Jacoby opener's 3x says nothing of strength
(10-21), so responder's 4M is not a decline after a limited hand as in
modified Jacoby. Should the 19+ line hold here, or lower (17+)?

## Sources

- Jacoby 2NT: Oswald Jacoby's game-forcing raise; the answers as
  ACBL's SAYC card and the convention-card spec
  (`spec/conventions/bidding_conventions/jacoby_2nt.toml`) give them;
  standard practice, not yet cited beyond that.
- Working values after the shortness: Rick, 2026-11-07, quoted above;
  the threshold is our calibration (slam-entry.notes.md, "Working
  values").
- Opener's pass over 4M without 19+: Rick, 2026-11-07 (quoted above),
  given for modified Jacoby and applied here at the controller's
  request.
