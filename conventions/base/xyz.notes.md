# xyz (`xyz.bid`): notes

XYZ (`other_conventions.xyz`): after three one-level bids responder's
2♣ relays to 2♦ (invitations, or weak with diamonds), 2♦ is an
artificial game force, 2NT relays to 3♣ (weak with clubs). After
1x-1y-1NT the same structure is two-way NMF: `two-way-nmf.bid` now has
no `card` line and plays when either `other_conventions.two_way_nmf` or
`other_conventions.xyz` is set (NMF, its natural 2♣/2♦ and the direct
invitations stand aside for both). Cases: `xyz.test`. Scenario: XYZ
(BBA on 21GF-DEFAULT without XYZ; with `--set other_conventions.xyz=true`
no problems in our auctions at the convention's calls, 50 boards,
2026-10-05).

## Structure

- 2♣ relay: 11-12, or weak with six diamonds (or four diamonds and a
  doubleton in opener's second suit after 1♦-1♥-1♠). Opener always
  answers 2♦ (two-way-nmf.bid's `xy_relay`); responder passes, or
  invites: 2y with five (a major), 3y with six, 3z with four of opener's
  second suit, 2NT balanced, 3 of opener's minor with four.
- 2♦ game force: answered as two-way NMF's (`xy_gf`): three of
  responder's major, else 2NT; responder places the contract.
- 2NT: weak with six clubs (or five and a doubleton in opener's second
  suit after 1♣); responder passes 3♣.
- Natural calls kept: 1NT, the raises of opener's second suit (2z weak,
  3z invitational), 2y with a weak six-card major, 1♠ over 1♣-1♦-1♥.
  Taken by XYZ: 2♣ and 2♦ (preference, the weak diamonds, the natural 2♣
  after 1♦-1♥-1♠), 2NT, the direct 3y invitation.
- Priority 2: ahead of fourth suit forcing (fourth-suit-forcing.bid),
  which still applies after a two-level rebid.

## Gaps and questions for Rick

- Todd lets opener break the 2♣ relay with a good 15-17; we always
  answer 2♦ (as two-way NMF). Decision: keep the relay forced.
- Every game-forcing hand goes through 2♦, even with four of opener's
  second major (4z directly would be natural); the answers find the fit.
- Some play XYZ's 2NT as natural invitational (Todd); we use the 3♣
  relay, matching two-way NMF and the convention-card summary
  ("extending two-way checkback").

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only xyz-vs-nmf`.
IMPs by the errors yardstick, positive when the convention makes fewer;
"actor" is the side that made the first differing call. Against New
Minor Forcing (on in both runs), XYZ: 248 boards changed; actor contract
-28, double-dummy -15, halves +15/-43 (z -0.7): neutral.

## Sources

- **Robert S. Todd, "XYZ", Advancing in Bridge #587**
  (https://www.advinbridge.com/this-week-in-bridge/587): the four
  auctions, 2♣ relay (invitational), 2♦ game forcing, continuations after
  the relay. convention-card `spec/conventions/bidding_conventions/xyz.toml`
  cites it.
- **Where we differ:** 2NT is the clubs relay (as two-way NMF, Bridge
  Winners "xyNT") rather than Todd's natural 2NT; the relay cannot be
  broken.
