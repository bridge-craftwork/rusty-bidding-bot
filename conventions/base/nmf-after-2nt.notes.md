# nmf-after-2nt (`nmf-after-2nt.bid`): notes

New Minor Forcing after opener's 2NT rebid (18-19), card field
`other_conventions.new_minor_forcing.after_2nt_rebid` (BBA's "NMF after
2NT rebid"; most 21GF cards). Cases: `nmf-after-2nt.test`.

## Structure

- 1♣-1M-2NT-3♦ and 1♦-1M-2NT-3♣: five of the major (or both majors), any
  game-going values; 1♥-1♠-2NT-3♣: five spades.
- Opener: three of responder's major, else four of the other major, else
  3NT. Responder: four of the major with a fit (5-3, 4-4), else 3NT.
- Independent of `new_minor_forcing.play`: 21GF-SPECIALS2 plays NMF after
  2NT without NMF after 1NT, and BBA's corpus on it shows the 3♣ ask.
- With Wolff on (`wolff_signoff.play`), 3♣ after 1♦-1M-2NT stays Wolff's,
  and there is no ask there.

## BBA (corpus, 2026-10-05)

On 21GF-DEFAULT BBA alerts 3 of the unbid minor after 1m-1M-2NT, and
after 1♥-1♠-2NT both 3♣ and 3♦ ("NMF after 2NT rebid"). We take 3♣ there
and leave 3♦ natural. After a 1♦ response (1♣-1♦-2NT) 3♦ is natural, as
in BBA. Opener's jump to 4M with three and a maximum (seen once) is not
modelled.

## Open questions

- 1♥-1♠-2NT: which minor asks? We use 3♣; BBA labels both.

## Sources

- **The convention:** New Minor Forcing as extended to the 2NT rebid;
  standard practice (Bridge Bum, "New Minor Forcing", mentions the 2NT
  rebid variant), not yet cited further.
- **BBA evidence:** the PBS corpus, auctions 1x-1M-2NT-3m with the alert
  (about 100 boards on 21GF-DEFAULT, a few on 21GF-SPECIALS2).
