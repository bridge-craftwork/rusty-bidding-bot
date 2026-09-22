# superaccept-doubleton (`superaccept-doubleton.bid`): notes

BBA's "Extended acceptance after NT" (card field
`notrump.transfers.super_accept_doubleton`). Opener super-accepts with
four trumps, a maximum and a doubleton. Cases: in `jacoby-transfers.test`
(responder's side).

## Guidance

- Over 2D: **2S with 4+ hearts, 16+ and a doubleton spade; otherwise 2NT**
  (a doubleton elsewhere). **With the queen in the spade doubleton (Qx, KQ,
  AQ), 2NT** as BBA plays it (Rick, 2026-09-21).
- Over 2H: 2NT with 4+ spades, 16+ and a doubleton.
- Both are artificial: never a place to play.

## Evidence from BBA

Jacoby_Super-Accept (500 boards): 81.7% of calls agree, 30.8% identical
auctions, 33.8% the same contract. When this module was added (fa9e091)
those were 73.0% and 0% identical.

Over `1NT P 2D P`, opener with four hearts and two spades (21GF-DEFAULT
corpus):

| Spade doubleton | BBA 2S | BBA 2NT |
|---|---|---|
| xx | 15 | 0 |
| Jx, Kx, Ax, KJ, AJ, AK | 71 | 0 |
| **Qx, KQ, AQ** | 0 | **22** |

## Gaps (not built yet)

- **Cuebids after the super-accept.** After 2NT, BBA's responder bids 3C:
  "Cue bid, a ♣ stopper" by EPBot's own meanings, not a relay. Opener cuebids
  in turn (3D, "a ♦ stopper"), or returns to 3M, "denies stoppers" in the
  suits skipped; then 4NT or game (263 corpus deals). We bid game or sign
  off instead.
- Opener's rebids after responder's 3m over the super-accept.

## Fixed (2026-09-21)

- Responder could pass the artificial 2NT: after the super-accept, the
  sign-off measured notrump points, and a five-card suit with invitational
  values had no rule. With a known fit, the sign-off now uses suit points,
  and invitational values bid game. Corpus: artificial contracts 87 → 18.

## Questions

- Cuebidding after a super-accept: build it (with controls, not
  stoppers?), or keep bidding game directly?
