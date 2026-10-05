# Scrambling 2NT (`scrambling-2nt.bid`)

Switched on by `competitive.scrambling_2nt.play` (no stock card plays
it: the scenario's 21GF-DEFAULT has it off). Cases:
`scrambling-2nt.test`.

## What it plays (2026-10-05)

(1M) P (2M) X (P) 2NT, and the same after a balancing double: both
minors, four or more each, no five-card suit, no four-card unbid
major, 0-10 HCP; the doubler bids his lower four-card minor, advancer
passes or corrects.

## The takeout double it starts from

There was no takeout double of their raise: (1x) P (2x) X was read as a
penalty double (penalty-doubles.bid) and advancer sat. takeout-double.bid
now has it, whatever the card: a singleton or void in their suit, three
or more in each unbid suit, 13+ HCP (the scenario's own standard); and
advancer's forced reply (his longest unbid suit, majors
first; game in a major or 3NT with 11+; a pass with five good trumps).
This is a base change. BBA passes most of these hands, even the
scenario's (Scrambling_2NT: 17 of the first divergences are BBA's pass
against our double). A first try at 12+ total points with a doubleton
doubled on 95 tripwire boards, BBA agreeing on 14: our contract errors
-8 IMPs, doubling errors +148 (contracts of ours going down undoubled,
charged to the defenders). The 13+ singleton version doubles on 32 of
those boards (BBA agrees on 8): contract errors -4, doubling +29. Kept
as the cited definition; question for Rick whether to follow BBA's
silence instead.

## Compare (2026-10-05, `--set competitive.scrambling_2nt.play=true`)

Scrambling_2NT, 50 boards: the divergences are at the double (above);
at the 2NT itself one board (BBA 3♣, card off). No bug found.

## Deviations and open questions

- Scrambling also applies, by most definitions, when the double was a
  reopening double after (1x) P (2x) P (P); the double itself is not
  written for that seat yet.
- Over a minor raise, 2NT stays undefined.
- Good/bad 2NT (the free-bid version) is not written.

## Sources

- Robert S. Todd, "Competitive Auctions: Scrambling 2NT", Advancing in
  Bridge 555 (cited by convention-card; not read).
- PBS `btn/Scrambling_2NT.btn` and the linked "Scrambling vs Good/Bad
  2N": the auction and "2NT asks partner to pick".
- The doubler's "lower four-card suit": standard practice, not yet cited.
