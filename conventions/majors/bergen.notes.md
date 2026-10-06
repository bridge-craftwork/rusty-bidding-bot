# bergen (`bergen.bid`): notes

Bergen raises over 1♥/1♠, RHO passing: four-card raises at the three
level. Cases: `bergen.test`.

## What the card says

| Field | `.bbsa` key | Meaning here |
|---|---|---|
| `major_openings.bergen_raises.play` | `Bergen` | 3♣ constructive (7-10), 3♦ limit (11-12), 3M preemptive |
| `major_openings.bergen_raises.reverse` | `Reverse Bergen` | 3♣ limit, 3♦ constructive; with both switches, Reverse |

21GF-GIB-Bergen, 21GF-Gazzilli and 21GF-SPECIALS set `Bergen`;
21GF-PolishTwoSuiters, 21GF-SPECIALS2 and the SCS cards set
`Reverse Bergen`.

## The structure (2026-10-05)

- Responder, four or more trumps (support points): 3♣ 7-10, 3♦ 11-12,
  3M about 3-6 (2+ HCP, at most 7 support points). Five trumps and a
  weak hand keep the jump to 4M (responses.bid): the constructive raise
  takes five trumps only from 8 HCP, the preemptive one only up to 3.
  Game values stay with Jacoby 2NT and splinters. A passed hand's limit
  raise is Drury's 2♣ when Drury is played.
- Opener: over the constructive raise game from 17 support points,
  15-16 the next step as a game try (3♦ in Bergen; 3♥ over Reverse
  Bergen's 3♦ when spades are trumps; with hearts there is no step and
  opener bids game from 16), else 3M. Over the limit raise game from 14.
  Opposite the preemptive 3M game from 19 (jump-raises.bid).
- Opener's answers are written against the auction (`after 1M (P) 3x
  (P)`), not as a question: a pending question shuts the slam rules out,
  and the big hands belong to them (Exclusion_After_1M 23 found it).
- With Bergen on, the invitational 1M-3M of responses.bid is off
  (`inv_jump_raise`), so a three-card limit raise goes through 2M, as on a
  card without the limit raise.
- Off in competition (any overcall or double): the natural raises apply.

## BBA

Probed (`probes/bergen-resp-1H.toml`, 21GF-GIB-Bergen) and in the
Bergen_Raises corpus, BBA plays on this card: 3♣ = four trumps, 7-9 HCP;
no 3♦ at all: the limit raise and up goes through 2NT ("inviting", 10+,
the unmapped `Strength Lawrence structure`?); 3M "1M-3M blocking", 2-5;
and it passes many 2-5 counts that we raise to 3M. It counts HCP where we
count support points, so a 7-8 HCP hand with a singleton is constructive
for BBA and a limit raise for us. All BBA style: we play the textbook
Bergen.

## Open questions

- `Shape Bergen structure` and `Strength Lawrence structure` (`.bbsa`,
  unmapped; 21GF-GIB-Bergen sets the second): what do they switch? BBA's
  2NT "inviting" in place of 3♦ may be the Lawrence structure.
- `Support 1NT` (unmapped; 21GF-GIB-Bergen sets it): unknown.
- The ambiguous splinter 3oM (Bergen's 1♥-3♠ / 1♠-3♥ add-on) is not
  played; normal splinters stay.

## Sources

- Bridge Bum, "Bergen Raises", https://www.bridgebum.com/bergen_raises.php
  (3♣ 7-10, 3♦ 10-12, 3M preemptive; Reverse swaps the minors).
- The PBS scenario Bergen_Raises (`btn/Bergen_Raises.btn`): 3♣ 7-10
  TP, 3♦ 11-12, 3M 3-6. We take 11-12 for 3♦, as there.
- Convention-card `bidding_conventions/bergen_raises` and
  `reverse_bergen_raises`.
- Opener's game thresholds (17, 14, 19): standard practice (25 support
  points between the hands with a nine-card fit), not yet cited.
