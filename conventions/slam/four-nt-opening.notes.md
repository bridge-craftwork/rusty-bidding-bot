# four-nt-opening (`four-nt-opening.bid`): notes

The 4NT opening asking for specific aces (2026-10-05), gated on
`slam.four_nt_opening.play` (on in most 21GF cards; off in Basic-Bridge
and Precision).

## What the rules do

- **The opening:** a hand whose only losers are aces: no void, each suit
  without the ace a singleton, K-x or headed by the K-Q, the suits with
  the ace losing nothing (`losers` = aces missing), one to three aces
  missing, and a suit of six or more to play in (the longest). Priority
  2, over the strong 2♣. The opening sets the long suit as trumps.
- **Answers:** 5♣ no ace, 5♦/5♥/5♠ that ace, 6♣ the club ace, 5NT two
  aces (three, which cannot happen often, the same).
- **Opener:** each ace partner shows covers one loser (all my losers
  are aces), so seven with none left, six with one, else five of the
  suit, or pass when partner's answer was five of it. An answer past
  five of the suit (6♣, or 5NT over 5♠-and-up) forces six.

Rare: no PBS scenario deals it; checked by the `.test` cases only.

## Open questions

- The opener does not play 6NT/7NT, even when the long suit is solid and
  the aces make the tricks.
- Three aces opposite is answered 5NT (two), as there is no step for it.

## Sources

- **The convention:** the 4NT opening for specific aces, standard
  practice (Culbertson's), as Bridge Guys' "Four Notrump Opening" and
  ACBL's convention charts describe it: 5♣ none, 5♦/5♥/5♠ that ace, 6♣
  the club ace, 5NT two aces. Not yet checked against a book; the
  opening condition (every loser an ace) is ours.
