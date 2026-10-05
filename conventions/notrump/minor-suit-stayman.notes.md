# minor-suit-stayman (`minor-suit-stayman.bid`): notes

Minor Suit Stayman over 1NT, on with `notrump.minor_suit_stayman.play`
(BBA "1N-2S Minor Suit Stayman": 21GF-MSS, 21GF-MSTandMSS). Cases:
`minor-suit-stayman.test`.

## What is played (2026-10-05)

GIB's version, as the PBS Minor_Suit_Stayman and MST_or_MSS scenarios
describe it and BBA bids it:

- 2♠: five-four or better in the minors, no four-card major, game
  values (forcing to game). Six-four goes here too (BBA).
- Opener: 3♣ four or more clubs (may hold four diamonds), 3♦ four or
  more diamonds and not four clubs, 2NT no four-card minor.
- Responder: with a fit, six of the minor from 33 total points
  between the hands, five with a singleton, else 3NT; without, 6NT
  from 32, 4NT quantitative, else 3NT.

With 21GF-MSTandMSS the minor transfers are 2NT (clubs) and 3♣
(diamonds), read from `notrump.transfers.two_nt_clubs` and
`three_c_diamonds` because `notrump.minor_transfers` has no value for
that pair (it reads `none` there). Any strength, six or more of the
minor and no four-card side minor (with one, 2♠); opener completes and
responder's next call is minor-transfers.bid's. The natural 2NT
invitation of one-nt.bid steps aside when 2NT is the transfer.

## Deviations

- GIB's opener 3NT "super accept" (17 and four of each minor) and
  BBA's 4♣/4♦ answers (23 and 20 boards in 100) are not played.
- After 2NT BBA shows a short major (3♥/3♠) on the way to slam; we bid
  6NT or 3NT directly.
- Minorwood (MST_or_MSS chat) belongs to slam/ and is not here.

## Scenario check (2026-10-05)

`compare Minor_Suit_Stayman MST_or_MSS We_Overcall_NT_then_MSS --limit
100`, calls after 1NT (and our 1NT overcall) and after 2♠/2NT/3♣:
**30 → 248 of 348**. Left: BBA's short-major continuations after 2NT,
and over our 1NT overcall BBA passes some 9-counts we take to game.

## Open questions

- `notrump.minor_transfers` lacks a value for "2NT clubs, 3♣ diamonds"
  (GIB/BBA's MSTandMSS combination); a field value in convention-card
  would let minor-transfers.bid own it. For now this module reads the
  two transfer switches.

## Sources

- Bridge Bum, "Minor Suit Stayman", https://www.bridgebum.com/minor_suit_stayman.php
  (cited in convention-card `bidding_conventions/minor_suit_stayman`).
- PBS `btn/Minor_Suit_Stayman.btn` and `btn/MST_or_MSS.btn` chat text
  (GIB's answers: 2NT, 3♣, 3♦; 2NT and 3♣ transfers).
