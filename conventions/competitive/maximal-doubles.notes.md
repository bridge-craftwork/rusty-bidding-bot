# Maximal doubles (`maximal-doubles.bid`): notes

Switched on by `doubles.maximal` (21GF-WJS-MSS, Precision,
Precision-14-16). Cases: `maximal-doubles.test`.

## What it plays (2026-10-05)

After 1M (any) 2M or (1x) 1M (any) 2M, when they bid three of the suit
directly below ours (3D under hearts, 3H under spades):

- X: the game try, 17-18 support points (the opener's usual invitation
  over a single raise in rebids.bid), alerted;
- 3M: competitive, six trumps or five with an unbalanced hand, 16 or
  less;
- 4M: 19+;
- partner answers the double with 4M on a maximum raise (9+ support
  points), else 3M; after the competitive 3M he passes.

Over any other three-level bid the double keeps its old meaning.

## BBA (fast-lane compare, 2026-10-05)

The scenarios' corpus (Maximal_Double, Maximal_After_Overcall) was bid
on 21GF-SPECIALS, which has `Maximal Doubles = 0` although the scenario
asks for them, so BBA's 3M there is invitational and its doubles are
penalty. With `--set doubles.maximal=true` call agreement 73.3% ->
74.1% (100 boards); the one remaining divergence at the maximal call is
the convention itself (our X game try where BBA bids 3M).

## Open questions

- Partner's penalty pass of the maximal double (a trump stack in their
  suit) is not written; the sources say partner signs off or bids game.
- Not applied when the responder, rather than the opener, is the one
  to act over their 3x (1M P 2M P P 3x), nor to minor-suit raises.

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only
maximal-doubles`. IMPs by the errors yardstick, positive when the
convention makes fewer; "actor" is the side that made the first
differing call. Maximal_Double and Maximal_After_Overcall: 85 boards
changed; actor contract +10, doubling +39; other side contract +42,
doubling -54; double-dummy +63 to the actor; halves +4/+45 (z +1.3):
leans gain.

## Sources

- Bridge Bum, "Maximal Double" (https://www.bridgebum.com/maximal_double.php):
  when it applies, X a game try, 3M competitive, partner signs off or
  bids game; alertable. Bridge Bum lists it after the opponents bid
  *and raise* the suit below ours; we also apply it when they introduce
  it at the three level (as the scenarios' filters do), where the lack
  of room is the same.
- Robert S. Todd, "Non-Takeout Doubles: Card-Showing, Maximal, Penalty,
  Lead-Directing", Advancing in Bridge 357 (cited by convention-card;
  not read here).
- Practice-Bidding-Scenarios `btn/Maximal_Double.btn`,
  `btn/Maximal_After_Overcall.btn`.
