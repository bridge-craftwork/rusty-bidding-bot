# two-way-nmf (`two-way-nmf.bid`): notes

Two-way New Minor Forcing (xyNT), card field
`other_conventions.two_way_nmf` (on in Precision and Precision-14-16).
Cases: `two-way-nmf.test`.

## Structure

After 1x-1y-1NT (every opening and response, uncontested):

- 2♣ relays to 2♦: every invitational hand (11-12), or a weak hand with
  six diamonds that passes 2♦. After the relay, 2M (five), 3M (six) and
  2NT invite; opener answers in rebids.bid.
- 2♦: artificial game force (13+, with a five-card major, both majors or
  an unbalanced hand; a balanced 13-15 bids 3NT). Opener: three of
  responder's major, four hearts after 1♠, else 2NT. Responder: four of
  the major with a fit, else 3NT.
- 2NT relays to 3♣, to play there with six clubs (weak).
- A jump to three of responder's suit is a slam try (six cards, 16+).
- On, it switches off NMF, the natural 2♣/2♦ and responder's natural
  2NT and three-level invitations (responder-rebids.bid).

## Corpus

The scenario Two-Way_New_Minor_Forcing_aka_xyNT plays 21GF-DEFAULT, which
has plain NMF, so BBA's auctions there are not xyNT. Run with
`--set other_conventions.two_way_nmf=true --set
other_conventions.new_minor_forcing.play=false`: no problems at the
convention's calls (2026-10-05). The Precision cards are the ones that
switch it on.

## Gaps and open questions

- Two-way NMF by a passed hand (`two_way_nmf_by_passed_hand`): not read;
  the relays apply by a passed hand too.
- The 2NT relay's shape-showing continuations (Bridge Winners' 3♦/3♥/
  3♠/3NT after 3♣) are not built; only the club sign-off.
- After 1♥-1♠-1NT, Bridge Winners reads 2♠ as 4-4 invitational; we keep
  our natural calls there.

## Played with XYZ (2026-10-05)

The module has no `card` line any more: its contexts hold when
`other_conventions.two_way_nmf` or `other_conventions.xyz` is set, since
XYZ (xyz.bid) is this structure extended to 1x-1y-1z. responder-rebids.bid
and new-minor-forcing.bid step aside for either switch.

## Sources

- **Bridge Winners, "Two-Way New Minor Forcing aka xyNT"**
  (https://bridgewinners.com/article/view/two-way-new-minor-forcing-aka-xynt):
  2♣ relay for invitations, 2♦ game force, 2NT relay to 3♣, three-level
  slam tries; off in competition. convention-card
  `spec/conventions/bidding_conventions/two_way_nmf.toml` cites it.
- **Where we differ:** opener's answers to 2♦ follow NMF's (standard
  practice, not yet cited); the article leaves them open.
