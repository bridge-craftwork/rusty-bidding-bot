# roudi (`roudi.bid`): notes

Roudi, card field `other_conventions.roudi.play` (21GF-SPECIALS2).
Cases: `roudi.test`. Scenario: WB5_Roudi.

## Structure

- After 1m-1M-1NT, 2♣ is artificial, invitational or better (11+ total
  points), with five of the major or both majors; ahead of a natural
  2NT/3NT.
- Opener (12-13 minimum, 14 maximum): 2♦ minimum, at most two of
  responder's major; 2♥ minimum with three (whichever major); 2♠ maximum
  with three; 2NT maximum, at most two; 3m maximum with three and five of
  the minor.
- Responder: after a minimum, invite (2NT, or 3M with a fit or six) or bid
  game; after a maximum, game (4M with a fit, else 3NT).
- On, it switches off NMF and the natural 2♣ (responder-rebids.bid).
- Only after a minor opening: on 21GF-SPECIALS2 BBA's 2♣ after
  1♥-1♠-1NT is natural (no alert), so ours is too.

## Corpus (WB5_Roudi, 2026-10-05)

NS calls agreeing 76.2% (all calls 88.1%). The opener's answers match
BBA's alerts on the boards checked. Divergences at the convention's
calls:

- **BBA style:** BBA passes 1NT with 11 total points where we invite
  (2NT or Roudi), and bids Roudi on some 10-counts; it is the existing
  invitation threshold, not Roudi.
- **BBA style:** after 2♥ (minimum, three) BBA's responder bids 2♠, a
  further try, where we invite with 3M or bid game. Not modelled.
- BBA sometimes passes 2♦ with a weak Roudi bid; ours always invites.

## Sources

- **WB5_Roudi scenario** (Practice-Bidding-Scenarios `btn/WB5_Roudi.btn`,
  its chat text): opener's five answers. The BBA corpus agrees with it.
- convention-card `spec/conventions/bidding_conventions/roudi.toml` (no
  structure given).
- **Where we differ:** responder's continuations are standard practice,
  not yet cited.
