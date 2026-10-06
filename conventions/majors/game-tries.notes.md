# game-tries (`game-tries.bid`): notes

Game tries after 1M-2M, uncontested. Cases: `game-tries.test`.

## What the card says

| Field | `.bbsa` key | Meaning here |
|---|---|---|
| `other_conventions.two_way_game_tries.play` | `Two way game tries` | GIB's two-way tries |
| `other_conventions.help_suit_game_tries.play` | none | a new suit is a help-suit try |

21GF-GIB (the EW card of nearly every scenario) and 21GF-PolishTwoSuiters
set `Two way game tries`, so the tries fire across the corpus.

## The structure (2026-10-05)

- Range 17-18 support points with 14+ HCP: the 3M invitation's range in
  rebids.bid. BBA passes 15 HCP with a singleton opposite a raise
  (Two-Way_Game_Try), and a singleton honour counts twice in tp.
- Two-way: the step (1♥-2♥-2♠, 1♠-2♠-2NT) shows a void or a small
  singleton (not K, Q or J) in a side suit. Responder declines (3M) with
  6 or less, accepts with 10, and asks with the next step otherwise;
  opener names the short suit, or bids 3M when it is a step suit (spades
  over 1♥, clubs over 1♠); responder bids game without a K, Q or J
  there. Any other suit below 3M is a long-suit try: three or more with
  an honour (GIB: "at least a 3-card suit with some honors"); over 1♥,
  2NT is the spade try and needs four spades with the ace or K-Q. BBA
  bids 2NT with such spades ahead of the short-suit try (AK62.KJT42.KT4.2,
  AJ98.AQT82.AT3.6) and the short-suit try with QJ86 or KT32.
- Help-suit (without two-way): a new suit below 3M with three or more
  cards and two or three losers.
- Responder accepts a long- or help-suit try with at most one loser in
  the suit or four of them, or the top of the raise (10), or 8+ with the
  ace or king there.
- Priorities 13-15, above the control bids (slam/control-bids.bid, 12):
  over a single raise a new suit is a game try and partner must read it
  so. The other way round, opener with 20+ who makes a control bid in the
  step suit (2♠ over 1♥-2♥) is read as the short-suit try (Benjamin_2D
  49): rare, left.
- base/game-force.bid: responder's third bid at the three level forced
  to game; three of the agreed suit is now excluded (the sign-off after
  the shortness exchange, 1♠-2♠-2NT-3♣-3♦-3♠).

## BBA

In Two-Way_Game_Try and Help_Suit_Game_Try (21GF-GIB both sides) BBA
always asks after the short-suit try, and invites only from 16 HCP. It
tries less often than we do with 17-18 support points and a singleton.

## Sources

- BBO, "GIB system notes", Two-way Game Try,
  https://www.bridgebase.com/doc/gib_system_notes.php#Two-way_Game_Try
  (the step, the ask, "bids his major if the short suit is one of the
  step suits", long-suit tries, 3M general).
- Bridge Bum, "Help Suit Game Try",
  https://www.bridgebum.com/help_suit_game_try.php.
- Responder's thresholds (6 / 10, wasted K-Q-J): standard practice, not
  yet cited.
