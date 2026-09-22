# minor-transfers (`minor-transfers.bid`): notes

Responses to 1NT with a long minor. One card field,
`notrump.minor_transfers`, chooses the treatment. Cases:
`minor-transfers.test` (a section per treatment).

## Guidance (Rick, 2026-09-21)

- **relay** is the default: 2S relays to 3C, and responder passes or
  corrects to 3D.
- **four_way**: 2S shows clubs and 2NT diamonds. Opener shows good support
  (Kxx or better) by bidding the step below the suit (2NT over 2S, 3C over
  2NT); bidding the suit denies it. This is how Rick plays it.
- **four_way_reversed**: the same, with the two answers swapped (the suit
  shows good support).
- **bba**: BBA's own treatment (below).
- Treatments are card choices, not separate conventions; only the calls
  that differ are written per treatment (LANGUAGE.md, "Treatments").
- Another variant Rick mentioned, not built: a 2S range ask, where 2S is
  weak with clubs or a notrump invitation, and opener bids 3C with a maximum
  and 2NT otherwise.

What we build on top:

- The transfers are for weak and invitational hands with no five-card
  major. With a five-card major, transfer to the major first; with game
  values, bid 3NT.
- After a minor transfer, responder plays the minor at the three level, or
  bids 3NT: with game values, or with invitational values opposite good
  support (the minor should run).
- With four-way transfers 2NT is not natural. Invitational hands without a
  major go through Stayman and bid 2NT next (`stayman.bid`), and the natural
  2NT (`one-nt.bid`) is off.

## Evidence from BBA

BBA's cards set the treatment through several switches: 21GF-DEFAULT
and 13 other PBS cards have `1N-2S transfer to clubs` and
`1N-3C transfer to diamonds`, which import as `bba`. In the corpus BBA's
opener always completes (3C or 3D, no super-accept). Responder passes when
weak (43 and 56 deals) and bids 3NT when invitational or better (22, 23).
With a five-card major and a six-card minor BBA transfers to the major
(616 calls).

Other switch combinations (Minor Suit Stayman over 1NT, 3C Puppet
Stayman, 2S clubs alone) import as `none`: we do not know how BBA plays
them.

Corpus, when this module was added: +752 calls agree with BBA, 1 fewer.

## Gaps (not built yet)

- Slam tries and game-forcing hands with a long minor.
- The 2S range ask.
- Interference over the transfers.
