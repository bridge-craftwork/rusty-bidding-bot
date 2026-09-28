# new-minor-forcing (`new-minor-forcing.bid`): notes

After 1m-1M-1NT, two of the other minor asks: invitational or better,
responder's major (usually five) or both majors. Card field
`other_conventions.new_minor_forcing.play` (on in 21GF-DEFAULT, off in
Basic-Bridge). Cases: `new-minor-forcing.test`.

## Guidance

Rick (2026-09-22): New Minor Forcing is its own convention. Without it, the
same bid is natural and not forcing.

## Structure

Opener: three cards in responder's major first; after a 1♠ response four
hearts; else 2NT with 12-13 and 3NT with 14. Responder: with a fit 3M
invites, 4M is game; without one 2NT invites, 3NT is game; pass opener's
minimum 2NT with an invitation. NMF outranks a natural 2NT or 3NT.

## Gaps

- Opener's jump answers with a maximum and a fit (3M), and 2 of opener's
  minor as a minimum with five.
- Two-way NMF and NMF after a 2NT rebid (their own card fields).

## BBA treatment (2026-09-24)

No `bba` rules. On Basic_* uncontested NS under `--set general.style=bba`
no divergence after `1x P 1y P 1NT P` involves the new-minor ask (the
largest there are 1♦-1♠-1NT: BBA 2♠ where we pass, 3 of 44; BBA pass or
2♣ where we bid 2♣ or 2NT, 2 each), so there was nothing to model.

## For Rick: our NMF loses to no NMF by double dummy (2026-09-25)

With the module removed on the NMF cards, 2,415 boards change:
- distance from par +1,644;
- IMPs to the bidding side +1,891.

Where the difference comes from:
- **The fit found through NMF, then 4M, against a direct 3NT**
  (1D-1S-1NT-2C-2S-4S vs 3NT, and the heart versions): 3NT is better
  double dummy on these deals. Double dummy tends to favour 3NT over a
  5-3 or 4-4 major fit less than the table does, so part of this may
  be the yardstick.
- **Invitations:** NMF with 10-11, then opener's 2NT (minimum), and we
  pass. The natural 2NT invitation reaches 3NT more often, and makes it.
  Opener's NMF answer treats 12-13 as a minimum; accepting with 13
  might recover much of this.

Not changed: NMF is on these cards, and dropping a standard convention
on a double-dummy count is your call. Worth trying first: opener
accepts the NMF invitation with 13; responder with a 5-3 fit and a
flat hand bids 3NT rather than 4M (a known modern choice).

**Found (same day): a bug, not NMF.** After NMF, responder bid game or
invited "in the fit" whenever opener showed three cards. NMF promises
only four in responder's major, so four spades opposite three went to
4S on a 4-3 fit. A fit now needs five cards opposite three (or 4-4
where opener showed four hearts): +2,574 by distance, +2,850 to the
bidding side (1,468 boards). With the fix, NMF beats no NMF by 930
IMPs by distance (955 to the bidding side): the finding above was the
bug.

## Sources

- **The convention:** New Minor Forcing after 1m-1M-1NT. Standard
  practice, not yet cited to a book or article.
- **Rick's rulings:** NMF is its own convention; without it the same bid
  is natural and not forcing (2026-09-22).
- **BBA evidence:** Basic_* under `--set general.style=bba` showed no
  divergence involving the ask, so there are no `bba` rules (2026-09-24).
  No `probes/*.toml` spec yet.
- **Corpus measurements:** the removal screen and the 4-3 fit bug
  (2026-09-25): with the fix, NMF beats no NMF by 930 IMPs by distance.
- **Where we differ:** none recorded from BBA. Suggestions not yet tried
  (opener accepting with 13, 3NT with a flat 5-3) are in "For Rick".
