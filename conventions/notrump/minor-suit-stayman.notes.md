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
  more diamonds and not four clubs, 2NT no four-card minor; with a
  maximum 4♣ / 4♦ / 3NT (2026-10-07, below).
- Responder: with a fit and 33 total points between the hands, the
  keycard ask in the minor (else six); with game values and a
  singleton or void in a major, three of that major (shortness, asks
  for a stopper: opener 3NT with one, else four of the minor, then
  responder five); else 3NT (five over opener's jump). Without a fit,
  on HCP: 6NT from 33, 4NT quantitative with 31-32, else 3NT.

With 21GF-MSTandMSS the minor transfers are 2NT (clubs) and 3♣
(diamonds), read from `notrump.transfers.two_nt_clubs` and
`three_c_diamonds` because `notrump.minor_transfers` has no value for
that pair (it reads `none` there). Any strength, six or more of the
minor and no four-card side minor (with one, 2♠); opener completes and
responder's next call is minor-transfers.bid's. The natural 2NT
invitation of one-nt.bid steps aside when 2NT is the transfer.

## Convention score (2026-10-07)

`conv_ab.py --only minor-suit-stayman` (Minor_Suit_Stayman and
2N_and_MSS, NS with and without both MSS switches): **−0.80 → −0.32
IMPs per changed board** (Rusty's gain −201 → +126, BBA's +343; 677
changed boards; halves −152/−65, agree). Background good. What was
wrong, against BBA's auctions on the same boards:

- **Responder with a fit and a singleton jumped to five of the
  minor** (−133 over 1NT on 364 boards where without MSS we bid 3NT).
  BBA bids the short major ("artificial") and opener 3NT with the
  stopper (BBA's auctions, black box; not in the cited Bridge Bum
  page, so a BBA-sourced treatment). Now played.
- **Slam hands went straight to six** where BBA asks for keycards
  ("Blackwood 1430, for ♣/♦") and finds the grand: now 4NT keycard
  over 3m/4m (1NT) and over 4m (2NT, two-nt-minors.bid).
- **The no-fit counts read total points**, so a 13-count with two
  long minors bid 6NT opposite 15-17. Now HCP.
- **Opener's maximum**: BBA answers 4♣/4♦ (with the minor) or 3NT
  (without) with every 17-count of 15-17 and with no other hand
  (corpus, 2026-10-07: 23 + 20 + 23 boards, all 17). Now played
  (`hcp>=shown.hcp.max`); it leaves room for keycard below five.
- Over 2NT, after 3NT (no four-card minor) slam hands ask with
  Gerber first (two-nt-minors.bid).

Left: over 2NT, keycard answers that should reach a grand stop in six
(slam/rkcb-1430.bid's grand logic, outside this module); BBA's short
major after 2NT (no fit) where we bid 3NT directly.

## Deviations

- GIB's opener 3NT "super accept" (17 and four of each minor) is not
  played; BBA's 3NT with a maximum shows no four-card minor, and that
  is what we play.
- After 2NT (no fit) BBA shows a short major (3♥/3♠); we bid 3NT, 4NT
  or 6NT directly.
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
- BBA's own auctions (black box: `bba/Minor_Suit_Stayman.pbn` and the
  conv-ab runs, 2026-10-07): opener's maximum jumps (4♣/4♦/3NT with
  17), responder's short major after the fit, keycard for the minor.
  Not in the cited Bridge Bum page.
