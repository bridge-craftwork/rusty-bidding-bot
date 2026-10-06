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
- Over their overcall (2026-10-05, written from the self A/B, standard
  practice not yet cited): responder 3NT with a stopper, 4M with five
  (four over their other major), else a values double; opener defends
  with three of their suit, else bids his cheapest four-card suit.
- Left, for Rick (self A/B, below): a slam try after the 2NT ask (16+
  and a fit signs off in game now), responder's pass of 2♦ with five
  diamonds and 9-10, and whether the scenario's deals (all 11-15
  three-suiters) are the fair field for Mini-Roman against the weak 2♦.

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only
mini-roman-vs-weak-2d`. IMPs by the errors yardstick, positive when the
convention makes fewer; "actor" is the side that made the first
differing call. Against the card's weak 2♦, Mini-Maxi_Roman_2D: 416
boards changed. First -359 to the actor (z -3.3), part of it responder
passing every overcall of the 2♦ (13 HCP and a fit included). Fixed:
responder's game calls and a values double over their overcall, opener's
answer to the double, responder's game. After: 415 changed (56 without
par); actor contract -282, doubling -6; other side contract -59,
doubling -250; double-dummy +20; halves -146/-142 (z -2.7): loses. What
is left is the structure itself on these deals (11-15 three-suiters
opened at two instead of one): the weak 2♥ pass-or-correct (-71 on 67),
the pass of 2♦ with five diamonds up to 10 (-44 on 40), and no slam try
after the 2NT ask (responder signs off in 4M with 16 and a fit: 6♠
missed three times). Questions for Rick above.

## Sources

- ACBL Unit 390 (Simon), "Mini-Roman",
  https://www.acblunit390.org/Simon/mini-roman.htm: range, shapes, pass,
  2♥/2♠ weak, 2NT asks the singleton.
- The up-the-line correction from a singleton, the invitational 3M and
  the placement after the ask: standard practice, not yet cited.
- PBS `btn/Mini-Maxi_Roman_2D.btn`.
