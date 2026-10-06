# Multi 2♦: notes

## Decisions

- Weak options only: a six-card major, 5-10 HCP (vulnerable a good
  suit), seats 1-3. The strong options of Bridge Bum's Multi (4-4-4-1,
  or a 20-21 notrump in Lars Multi) are not written: the card has only
  `two_level.two_diamonds.multi`, and BBA opens those hands naturally
  (probes/multi-openings.toml: 4-4-4-1 18 opens 1♦, 22 opens 2♣).
- With Multi on, the natural weak 2♥/2♠ are off (preempts.bid): BBA
  opens every weak two in a major 2♦ even though 21GF-Multi keeps
  `Weak natural 2M = 1` (probe). Five-card majors pass.
- A ten-count with the ace-king of its suit, or the ace and a side ace,
  opens at the one level, as preempts.bid's default (BBA).
- Responses (Bridge Bum): 2♥ pass-or-correct; 2♠ pass-or-correct with
  three+ hearts and short spades; 2NT asks (from 15); 3♥ / 4♥ preemptive
  pass-or-correct with both majors (eight cards / four-four); 4♦ "bid your
  major" from 15 with three+ in both; 3♣ to play. The same over a double.
- Answers to 2NT (Bridge Bum and BBA): 3♣ max ♥, 3♦ max ♠, 3♥ min ♥,
  3♠ min ♠. The asker bids game opposite a maximum, and opposite a
  minimum from 18 (16 with three trumps).

## BBA (compare Multi_2D --limit 50, 2026-10-05: 79% of calls agree)

- 2♦–P: BBA bids 2♥ where we bid 2♠ (three hearts, short spades): BBA
  style; Bridge Bum's 2♠ is kept.
- BBA's 3♦ response (artificial, an invitation: opener 3M minimum, 4M
  maximum) is not written.
- Most other divergences are the opponents' (EW) calls or hands that
  never reach 2♦.

## Open questions

- Strong options for the Multi (4-4-4-1, 20-21 for Lars Multi): need a
  card field (convention-card) to choose (still none, 2026-10-05); Lars_Multi_2D has no BBA
  corpus (bba-works: false).
- Should 2♥/2♠ stay natural weak twos when the card also says `Weak
  natural 2M` (then Multi would hold only the strong options)? We follow
  BBA: Multi takes them.

## Sources

- Bridge Bum, "Multi 2D", https://www.bridgebum.com/multi_2d.php: the
  weak options, pass-or-correct responses, the 2NT ask and answers.
- probes/multi-openings.toml (BBA's openings with 21GF-Multi), PBS
  `bba/Multi_2D.pbn`.
