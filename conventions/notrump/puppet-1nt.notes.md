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

## Convention score (2026-10-07)

`conv_ab.py --only puppet-stayman` (Puppet_Stayman_1N and _2N, NS
with and without both Puppet switches): **−0.88 → −0.27 IMPs per
changed board** (Rusty's gain −235 → +375, BBA's +640; 994 changed
boards; halves −231/−34, agree). Background (the scenarios' score
against BBA) good, so the convention was the problem. What was wrong:

- **The slam hands lost their ace check.** Without Puppet our slam
  hands ask with Gerber directly over 1NT/2NT and reach the grands;
  with it they jumped to 6NT over opener's 3NT (and over a 3NT that
  denies the 4-4 fit), where BBA asks with Gerber (−291 on 191
  boards). Now 4♣ Gerber there (two-nt-responses.bid), when the
  card plays it.
- **The 4-4 fit went straight to six.** After 3♦–3M–4M responder bid
  6M ahead of the keycard ask; it now ranks behind it (BBA: "Blackwood
  1430").
- **Over 2NT, Gerber outranked Puppet** (equal priority, file order):
  194 boards on Puppet_Stayman_2N where BBA puppets first. Puppet
  now ranks ahead of Gerber, as it does over 1NT.
- 7NT after 3♦ with no four-card major and 37 between the hands
  (BBA bids it directly).

Left: the keycard answers stop in six with every keycard and the
grand there (slam/rkcb-1430.bid, outside this module), and the
reverse, a grand bid with a king missing (Puppet_Stayman_1N 29, 159,
221); BBA's quantitative 4NT over 2NT–3♣–3NT where we ask aces;
2NT–3♣–3♦ with no major, too weak for 7NT, bids 6NT where without
Puppet Gerber finds the grand (BBA the same).

## Deviations

- The scenario's "after 3♥/3♠, the other major is a slam try" is not
  written: responder bids 4M or 6M on the total-point count.
- Puppet "on in competition" (scenario text) is not written.

## Sources

- Bridge Bum, "Puppet Stayman", https://www.bridgebum.com/puppet_stayman.php
  (cited in convention-card `bidding_conventions/puppet_stayman`).
- PBS `btn/Puppet_Stayman_1N.btn` chat text (when to bid it; the
  answers and responder's follow-ups).
