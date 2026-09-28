# DONT (`dont.bid`): notes

DONT (Disturb Opponents' No Trump, Marty Bergen) over their 1NT, direct
and balancing seat. Cases: `dont.test`. Shared answers: `vs-1nt.bid`;
the comparison of the defences: `vs-1nt.notes.md`.

## Guidance (Rick, 2026-09-28)

Implement it so that cards that play it are played as written. The card
turns it on with `competitive.vs_1nt_strong.system = dont`,
or with Bridge-Classroom's own switch `competitive.dont.play` (its
catalog entry `dont`) when the convention field is unset. No `.bbsa`
key: BBA has no DONT, and the corpus scenario `DONT` is marked
`bba-works: false` (BBA played Cappelletti there: 379 notes).

## The calls

| Call | Shows | Advancer |
|---|---|---|
| X | one long suit, six or more, not spades, no four-card side suit | 2♣ relay (pass with clubs, else 2♦/2♥); a six-card suit of his own to play |
| 2♣ | clubs and a higher suit, 5-4 either way | pass with three clubs, else 2♦ asks for the other suit |
| 2♦ | diamonds and a major, 5-4 either way | pass with three diamonds, else 2♥ pass or correct |
| 2♥ | both majors, 5-4 either way | preference (vs-1nt.bid) |
| 2♠ | six or more spades | pass, or raise with support |

Strength: DONT is a nuisance defence and is meant to be light. We count
HCP plus a point a card beyond four in each suit and ask for 10 (8 HCP
with a six-card suit or 5-5, 9 with 5-4), with a ceiling of 15 HCP:
there is no penalty double, and a stronger hand passes and waits. The
same in the balancing seat (the sources allow even lighter there).

A 6-4 hand is shown as a two-suiter (2♣ or 2♦), not by the double:
Wikipedia's "any single suit" and the pass-or-correct structure assume
the double is one-suited.

## Sources

- Bridge Bum, "DONT", https://www.bridgebum.com/dont.php (X one suit,
  relay 2♣; 2♣/2♦ that suit and a higher one; 2♥ the majors; 2♠ spades;
  advancer passes with support or bids the next suit to ask; "8 points
  might qualify … at favorable vulnerability", lighter when balancing).
- Wikipedia, "DONT", https://en.wikipedia.org/wiki/DONT (X "any single
  suit (six or more cards)"; 2♣, 2♦, 2♥ "the bid suit and any
  higher-ranking suit"; 2♠ six spades; the next suit asks, a new suit is
  natural).
- Bridge-Classroom `src/utils/ntDefenses.js` (the editor's DONT: "Single-
  suited (pass to play)", "♣ + higher", "♦ + higher", "♥ + ♠", "♠ (long,
  by inference)").

## BBA

No reference: BBA does not play DONT. Judged by par and the side-IMPs
yardstick only (vs-1nt.notes.md, forced on for both sides).

## Gaps and open questions

- The advancer's 2NT (strong, asks for the second suit) and invitational
  raises of the major are not written.
- Some play the double as including spades and 2♠ as weaker; we follow
  the common "X = a minor or hearts, 2♠ = spades".
- After a redouble the relay is still 2♣; pass-to-play agreements over
  the redouble are not written.
