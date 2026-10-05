# SOS redouble (`sos-redouble.bid`)

Switched on by `other_conventions.sos_redouble.play` (no stock card, no
PBS scenario). Cases: `sos-redouble.test`.

## What it plays (2026-10-05)

Only 1x (X) P (P): our one-level opening, their takeout double passed
for penalty. Opener redoubles with three cards or fewer in his suit
(SOS: pick another suit), passes with four or more. Responder bids his
longest other suit, the cheapest of equals.

## Deviations and open questions

- The source's other spots (a natural overcall doubled for penalty and
  passed round, our 1NT doubled) are not written: the engine cannot
  tell a penalty double from a takeout or negative one except where the
  auction makes it so, and the 1NT runouts (nt-interference.bid) already
  own 1NT (X).
- Without the switch the redouble there is not written (opener passes).

## Sources

- "25 More Bridge Conventions You Should Know", ch. 22 (cited by
  convention-card; not read).
- convention-card `spec/conventions/competitive_bidding/sos_redouble.toml`:
  "when a low-level contract is doubled for penalty, a redouble asks
  partner to choose another suit".
