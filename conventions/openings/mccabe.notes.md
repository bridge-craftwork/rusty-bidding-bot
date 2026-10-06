# McCabe adjunct: notes

## Decisions

- After our weak two (2♦, 2♥ or 2♠) is doubled: a new suit at the three
  level below opener's suit (over 2♦ only 3♣) is a raise to three that
  asks for that lead, with two of its top three honours; opener bids
  three of his suit. Redouble shows a six-card suit of responder's own
  and at most a singleton in opener's: opener bids the next step
  (2♦→2♥, 2♥→2♠, 2♠→2NT), responder passes it or bids his suit.
  2NT (15+) asks as without the double. The plain raises and pass are
  those of total-tricks.bid / vs-preempts.bid.
- 2♥ (X) 2♠ stays natural (Todd's article names three-level suits only).

## PBS (compare McCabe_After_Weak_2 --limit 50 --set two_level.mccabe.play=true, 2026-10-05)

- 76.4% of calls agree (75.9% without McCabe); errors line +18 IMPs to
  us (-65 without). BBA does not play McCabe (no `.bbsa` key), so its
  3♣/XX/pass at these points is its own style.
- After a McCabe call (2♠ X 3♥ P 3♠ P, 2♦ X 3♣ P 3♦ P) and after the
  redouble's relay the doubler's side has no rule and passes: the
  defence to it belongs to the competitive area (vs-preempts.bid).

## Open questions (decision taken)

- Transfer McCabe (Todd's second article) is not written.
- With a fit and a long side suit, the lead-directing raise wins over a
  redouble (it shows the fit).

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only mccabe`.
IMPs by the errors yardstick, positive when the convention makes fewer;
"actor" is the side that made the first differing call.
McCabe_After_Weak_2 and McCabe_after_WJO: 168 boards changed; actor
contract -20, doubling +181; other side contract +161, doubling -167;
double-dummy +155; halves +76/+85 (z +2.4): gains.

## Sources

- Robert S. Todd, "McCabe Responses to Preempts", Advancing in Bridge,
  https://www.advinbridge.com/this-week-in-bridge/468: the new-suit
  raises, the redouble, 2NT as without the double.
- PBS `btn/McCabe_After_Weak_2.btn` (responder's hand types).
