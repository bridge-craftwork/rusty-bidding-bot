# puppet-1nt (`puppet-1nt.bid`): notes

Puppet Stayman over 1NT, on with `notrump.stayman.puppet_1nt` (BBA
"1N-3C Puppet Stayman": 21GF-Puppet). Cases: `puppet-1nt.test`.

## What is played (2026-10-05)

1NT–3♣ with game values, no five-card major (that transfers) and a
three- or four-card major, as the PBS Puppet_Stayman_1N scenario
defines it ("any 4-card major, or 3-card support for one or both
majors"). It outranks 2♣ Stayman for these hands; 2♣ stays for the
invitational ones. Everything after the ask is the Puppet structure
over 2NT (two-nt-responses.bid, `when asked puppet`): 3♥/3♠ five
cards, 3♦ a four-card major, 3NT neither; after 3♦ responder bids the
major he does not hold, 4♦ both, 4♣ both with slam interest. The slam
counts there read partner's minimum, so they follow the 1NT range.

With the switch on, the natural 1NT–3♣ slam try of one-nt.bid is off.

## Scenario check (2026-10-05)

`compare Puppet_Stayman_1N --limit 100`, calls after 1NT up to
responder's second call: **26 → 301 of 366**. Left: over 3NT BBA bids
4♣ (Gerber or a club slam try) where we jump to 6NT, and its keycard
auctions (slam/).

## Deviations

- The scenario's "after 3♥/3♠, the other major is a slam try" is not
  written: responder bids 4M or 6M on the total-point count.
- Puppet "on in competition" (scenario text) is not written.

## Sources

- Bridge Bum, "Puppet Stayman", https://www.bridgebum.com/puppet_stayman.php
  (cited in convention-card `bidding_conventions/puppet_stayman`).
- PBS `btn/Puppet_Stayman_1N.btn` chat text (when to bid it; the
  answers and responder's follow-ups).
