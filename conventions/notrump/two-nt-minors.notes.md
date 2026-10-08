# two-nt-minors (`two-nt-minors.bid`): notes

Two treatments of 2NT–3♠, each on its own card switch. Cases:
`two-nt-minors.test`.

## Minor-suit transfer (`notrump.two_nt.minor_transfers`)

BBA's "Minor Suit Transfers after 2NT" (on for 21GF-DEFAULT and most
21GF cards) is one relay for both minors: 2NT–3♠–4♣ (alerted "Minor
Suit Transfers after 2NT", then "automat"), then responder passes,
bids 4♦, five or six of his minor. The separate
`two_nt.transfer_3s_clubs` / `transfer_4c_diamonds` switches (the
Bridge Bum two-suit scheme) are off on every corpus card and not read
here.

We use it for **weak hands only** (six or more of a minor, no four-card
major, 0-3 HCP): pass 4♣ or bid 4♦. BBA also relays with some 7-9
counts and then bids slam, but its choices between 3♠, 3NT and Gerber
4♣ at 4-13 HCP showed no rule we could write on 100 boards
(2N_and_1_Minor), so game hands bid 3NT and slam hands keep the 4♣/4♦
natural tries (or Gerber) of two-nt-responses.bid.

## Minor Suit Stayman (`notrump.two_nt.minor_stayman`)

3♠ asks for a four-card minor: five-four or better in the minors, no
four-card major, slam interest (game hands bid 3NT, as BBA
does). Opener 4♣ with four clubs, 4♦ with four diamonds and not four
clubs, 3NT with neither. Responder bids six in the fit from 33 total
points between the hands, else five; over 3NT 6NT from 32, 4NT
quantitative with a slam invitation, else pass. If a card has both
switches on, 3♠ is Minor Suit Stayman.

## Slam hands (2026-10-07)

From the Minor Suit Stayman convention score
(minor-suit-stayman.notes.md, "Convention score"): over the fit
(2NT–3♠–4m) responder with slam values asks for keycards in the minor
when the card plays it (BBA: "Blackwood 1430, for ♣/♦"), else bids
six; over 3NT (no four-card minor) slam hands ask with Gerber first
when the card plays it, as they do directly over 2NT without the
convention.

## Scenario check (2026-10-05)

`compare 2N_and_MSS 2N_and_1_Minor --limit 100`, calls after 2NT and
after 2NT–3♠: **80 → 155 of 238**. What is left is BBA style: over
3NT it bids 4♣ (a natural slam try) where we bid 6NT, over 4m it uses
keycard (slam/ module), and its own choices with long minors.

## Every 2NT, and the weak relay (2026-10-05)

The module now answers every 2NT where systems are on (`after nt2`;
two-nt-responses.notes.md, "One system for every 2NT"). The relay's
"0-3 HCP" was written for 20-21; it is now "too weak for game opposite
opener's maximum" (`hcp<=24-partner.hcp.max`, game `hcp>=25-partner.
hcp.max`), the same 0-3 opposite 20-21. In a game force (2♣–2♦–2NT,
Kokish's 2NT) there is no weak hand to play a partscore with: the relay
is off there and the weak long minor bids 3NT (two-nt-minors.test).

## Open questions

- 2NT–3♠ for game hands with a long minor and shortness (5m rather
  than 3NT)? Decided: no, 3NT (BBA mostly does too).

## Sources

- Bridge Bum, "Minor Suit Stayman", https://www.bridgebum.com/minor_suit_stayman.php
  (cited in convention-card `bidding_conventions/minor_suit_stayman`):
  3♠ over 2NT with both minors.
- PBS `btn/2N_and_MSS.btn` chat: responder at least 5-4 in the minors,
  game or slam.
- The single 3♠ relay for either minor: BBA's own auctions in
  `bba/2N_and_1_Minor.pbn` (black box). Standard schemes use 3♠ for
  clubs and 3NT or 4♣ for diamonds (Bridge Bum, "Minor Suit Transfers");
  we follow BBA because the card field names BBA's switch.
