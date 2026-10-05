# Mini-Roman 2♦: notes

## Decisions

- 11-15 HCP, 4-4-4-1 (any singleton) or 5-4-4-0 with the five in a
  minor; seats 1-4. A five-card major opens 1M. The Maxi half (19+,
  PBS `Mini-Maxi_Roman_2D`) is not written: the card has only
  `two_level.two_diamonds.mini_roman`, and Maxi changes every response
  (responder may not pass 2♦).
- Spades are not guaranteed (the source lists that as a partnership
  choice; the PBS scenario deals only hands with four spades).
- Responder: pass with five diamonds and up to 10; 2♥ / 2♠ / 3♣ to play
  with a weak hand and four (clubs five); opener passes or, short there,
  bids his next four-card suit up the line. 2NT (11+, game forcing) asks
  for the short suit; responder then bids 4M with four, 3NT with the short
  suit stopped, else five of a minor. 3♥/3♠ invitational with five,
  4♥/4♠ to play with six.

## PBS (compare Mini-Maxi_Roman_2D --limit 50 --set ...mini_roman=true, 2026-10-05)

- BBA does not play Mini-Roman (`bba-works: false`; it opens these hands
  1♣/1♦), so the divergences at the opening (42 of 50) are expected.
- Passing 2♦ with five diamonds can leave a 5-0/5-1 fit when opener is
  short in diamonds (one board); the source's choice is kept.
- Weak responses can end in a 4-3 or 4-2 fit after opener's correction
  (inherent in the convention).
- Over 2♦ (X) 2♥ P 2♠ the doubler's side has no rule (competitive area).

## Open questions (decision taken)

- Maxi-Roman (19+) as part of the same 2♦: needs a card field. Not written.
- Four spades required? We take any 4-4-4-1.

## Sources

- ACBL Unit 390 (Simon), "Mini-Roman",
  https://www.acblunit390.org/Simon/mini-roman.htm: range, shapes, pass,
  2♥/2♠ weak, 2NT asks the singleton.
- The up-the-line correction from a singleton, the invitational 3M and
  the placement after the ask: standard practice, not yet cited.
- PBS `btn/Mini-Maxi_Roman_2D.btn`.
