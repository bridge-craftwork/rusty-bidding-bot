# suction (`suction.bid`): notes

Suction over their 1NT (`competitive.vs_1nt_strong.system = suction`,
the option existed but no rules played it: it fell to the pass). Each
two-level suit bid shows the next suit up or the two above it; X clubs
and hearts, 2NT diamonds and spades. Cases: `suction.test`. Scenario:
Suction (BBA on 21GF-DEFAULT, so natural/Cappelletti there: with `--set
competitive.vs_1nt_strong.system=suction` only our problems count; 50
boards, 2026-10-05: two in competition after their double, none at
Suction's own calls).

## Structure

- One-suiter: six or more, or a good five, with no second four-card suit
  beside a five. Two-suiters 5-4 or longer at the two level, 5-5 where
  they may play at three (2♥ minors, 2♠ diamonds-hearts, 2NT). Strength
  as cappelletti.bid: HCP plus a point a card beyond four from 10, HCP at
  most 17. Preempts and the pass are vs-1nt.bid's.
- Advancer bids the next suit (the one-suiter); the overcaller passes,
  or shows the two-suiter: after 2♣ the majors (vs-1nt.bid's
  `nt_majors`), after 2♥ the minors (`nt_minors`), after 2♦ / 2♠ the
  first of the pair, which advancer keeps or corrects. Over X and 2NT
  advancer takes the longer suit, the higher when equal.

## Gaps and questions for Rick

- Advancer never raises or jumps with a fit or values; game is found only
  through the generic rules. Decision: the relay first, as Bridge Bum
  ("advancer usually assumes a one-suited hand, and bids that suit").
- Our side after their double or redouble of a Suction call: not built
  (the problems seen in the scenario).
- `vs_1nt_weak.system` is still not read (vs-1nt.bid).

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only suction`.
IMPs by the errors yardstick, positive when the convention makes fewer;
"actor" is the side that made the first differing call. Against the
card's default defence, Suction: 401 boards changed. First -90 to the
actor (z -1.2), two bugs: the overcaller did not correct the relay once
they had doubled it (1NT 2♦ X 2♥ X left in with spades and clubs,
-1400), and advancer passed their redouble of the Suction double. Both
continue now over X and XX. After: 400 changed; actor contract -32,
doubling +84; other side doubling -238; double-dummy +238; halves +60/-8
(z +0.8): neutral.

## Sources

- **Bridge Bum, "Suction"** (https://www.bridgebum.com/suction.php): the
  meanings, "a good 5+ card suit at the 2-level, or a 6+ card suit if
  forcing the bidding to the 3-level", 5-5 for three-level two-suiters,
  advancer bids the one-suiter. convention-card
  `spec/conventions/competitive_bidding/suction.toml` cites it.
- **Where we differ:** strength thresholds follow our other 1NT
  defences (cappelletti.bid), not Bridge Bum (which gives none).
