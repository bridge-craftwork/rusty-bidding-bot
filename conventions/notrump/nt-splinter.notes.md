# nt-splinter (`nt-splinter.bid`): notes

Splinters over 1NT: 1NT–3♦ with `notrump.three_d_response = splinter`
(BBA "1N-3D splinter"), 1NT–3♥/3♠ with `notrump.three_major_splinter`
("1N-3M splinter"); both on for 21GF-GIB. Cases: `nt-splinter.test`.

## What is played (2026-10-05)

The splinter suit is a singleton or void, the other three suits are
four or more each with no five-card major (4-4-4-1, 4-4-5-0, ...), and
game values. It outranks Stayman. Opener bids four of a major he holds
four of (spades first over 3♦), else 3NT. Nothing is written for slam:
responder passes the game (slam/ may add a try later).

**Not with a singleton A, K or Q.** The PBS NT_Splinter text says GIB
splinters with one; BBA does not (it bids 2♣ Stayman: 20 boards in 100),
and the standard advice is the same (an honour singleton is a stopper
for notrump, not shortness to show). We follow BBA.

## Scenario check (2026-10-05)

`compare NT_Splinter --limit 100`, responder's call over 1NT and
opener's answer: **24 → 162 of 166**.

## Sources

- PBS `btn/NT_Splinter.btn` chat text (shape and strength: a stiff or
  void, four in each other suit, 12+ total points).
- BBA's own auctions in `bba/NT_Splinter.pbn` (black box): the
  singleton honour exception and opener's answers.
- Standard practice for the stiff-honour exception, not yet cited.
