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

## Brill (general.system = brill, 2026-10-15)

Brill's treatment, played on the Brill card only (`cards/json/Brill.json`, `general.system = brill`: a stand-in until convention-card's `general.style` takes a `brill` value; cards/README.md "Brill"). Rusty plays the Brill dailies as South with three Brill robots, so it should read and bid what they do. Every other card is unchanged (a `.test` line on 21GF-GIB beside each Brill line; `compare --limit 50` tripwire against main: no board changed).

A maximum 4-3-3-3 bids 2NT, since the jump to three of the major is the
minimum. Brill "Superaccept, no doubleton": `max and ExplicitShape ==
'3=4=3=3'`; over 2H "Superaccept, max": `max and ExplicitShape == '4=3=3=3'`;
"Superaccept, doubleton": `max and S <= 2 and hearts > 3` (we keep 2NT with
Qx of spades, as for BBA). Not seen in the cache: what Brill bids with a
maximum, four trumps and a minor doubleton (we bid 2NT).

Measured with `probes/tools/brill_probe.py report --card Brill` over the top 2000 corpus auctions (calls of this group, board-weighted, our reading against Brill's): see jacoby-transfers.notes.md.

## Sources

- Brill /explain, 2026-10-15 (Thorvald Aagaard's public Brill service, answers cached in `.rbb-cache/brill-service/explain/`): the rule text quoted in the `brill` section and in the `.bid` comments. Applies to the Brill card only (`general.system = brill`, cards/README.md "Brill").
- **The convention:** BBA's "Extended acceptance after NT" `.bbsa` key,
  as BBA plays it; learned from BBA's auctions, not from a published
  description. Not yet cited to a book or article.
- **Rick's rulings:** with the queen in the spade doubleton, 2NT as BBA
  plays it (2026-09-21).
- **BBA evidence:** the Jacoby_Super-Accept corpus, and the 21GF-DEFAULT
  corpus table of opener's call by spade doubleton ("Evidence from BBA").
  BBA's cue-bid meanings after the super-accept come from EPBot's own
  alerts. No `probes/*.toml` spec yet.
- **Corpus measurements:** agreement when the module was added (fa9e091)
  and the 2026-09-21 fix (artificial contracts 87 to 18).
- **Where we differ:** no cue bids after the super-accept; we bid game or
  sign off (Gaps).
