# Precision 2♦: notes

## Decisions

- 11-15 HCP; 4-4-1-4 or 4-4-0-5; 4-3-1-5 or 3-4-1-5 when the clubs hold
  fewer than two of the top three honours (Bridge Bum: "too weak for a
  Precision 2♣"; we have no Precision 2♣ yet, so this stands in).
  Written as an opening of its own: it does not depend on the forcing
  1♣, and any card with `Precision 2D` turns it on (Precision.bbsa,
  Precision-14-16.bbsa).
- Responses as Bridge Bum: pass with six diamonds; 2♥/2♠/3♣ sign-offs
  (2♥ with equal majors, opener's 3-4-1-5 is as likely as 4-3-1-5); 2NT
  game-forcing ask with Bridge Bum's answers; 3♦ six diamonds inviting
  3NT (opener accepts with 14-15); 3♥/3♠ preemptive.
- After the answer responder places the contract: a known 4-4 major fit
  (or five opposite three), 3NT with diamonds stopped, else 5♣.

## BBA (probes/b2-Precision-openings.toml, 2026-10-05)

- With the Precision card BBA opens 2♦ on 4-4-1-4 and 4-4-0-5 14, as we
  do. With `Precision 2D = 1` set on 21GF-DEFAULT it ignores the key and
  opens 1♣; we honour the field on any card.
- Tripwire (batch-1 baseline): 4 boards in SCS_* now open 2♦ (expected).
  On the Precision card the weak 2♦ is now off (`Weak natural 2D = 0`
  there; before, the default kept it on): five third-seat boards in
  Weak_NT_* that opened a weak 2♦ on a six-card ten-count now pass, where
  BBA opens 3♦. Our three-level preempt wants seven cards (preempts.bid).

## Open questions (decision taken)

- Invitational hands with a major (10-ish and four cards) have no
  natural invite in Bridge Bum's scheme; they sign off at two or use 2NT.
- No defence to interference beyond opener passing.

## Sources

- Bridge Bum, "Precision 2D", https://www.bridgebum.com/precision_2d.php:
  shapes, range, every response and the 2NT answers.
