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
