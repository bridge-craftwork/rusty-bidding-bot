# Benjamin twos: notes

## Decisions

- 2♣ = 22-23 (any shape), or unbalanced 20-23 with a good six-card suit
  or a 5-5/6-5 two-suiter and at most four losers. 2♦ = 24+, or 23
  without a five-card suit. These are BBA's ranges (Benjamin_2D corpus);
  Wikipedia and the PBS chat put game-in-hand hands (fewer HCP, a long
  solid suit) in 2♦, BBA opens them 2♣. We follow BBA here because the
  source gives no number for "game in hand".
- The strong 2♣ openings of strong-openings.bid are off (`!benjamin`).
- 2♣ keeps the strong 2♣'s machinery: game forcing, 2♦ waiting, the
  positives and the second negative. Benjamin's 2♣ is really forcing for
  one round and passable after opener's rebid; written that way, half of
  the continuations had no rule (no generic non-forcing structure after
  2♣–2♦–2M). Open: write the passable version.
- Responses to 2♦ (PBS chat, BBA): 2NT negative, 2♥/2♠ five+ and 5+ HCP,
  3♣/3♦ six+, 3NT balanced 8+. After 2♦–2NT opener bids naturally (3NT
  balanced, 4NT 28+ quantitative).
- Weak 2♦ off (preempts.bid).

## BBA (compare Benjamin_2D --limit 50, 2026-10-05: 76% of calls agree)

- Opening 1♠/1♣ vs our 2♣ (5 boards): BBA opens some 20-21 two-suiters
  at the one level; don't care.
- 2♦–P: BBA's thresholds for the positives differ a little: don't care.
- The scenario's card (21GF-PolishTwoSuiters) also plays Polish
  two-suiters and Namyats.

## Open questions

- Should 2♣ be passable after 2♣–2♦–2M / 2NT (true Benjamin)? We keep
  the game force for now.
- Game-in-hand hands with fewer than 24 HCP: 2♦ (Wikipedia) or 2♣ (BBA)?
  We follow BBA.

## Sources

- Wikipedia, "Benjamin Twos", https://en.wikipedia.org/wiki/Benjamin_Twos:
  the meanings of 2♣ and 2♦.
- Practice-Bidding-Scenarios `btn/Benjamin_2D.btn` (the responses) and
  `bba/Benjamin_2D.pbn` (ranges).
