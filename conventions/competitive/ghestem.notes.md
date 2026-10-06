# Ghestem (`ghestem.bid`)

Switched on by `competitive.ghestem.play` (no stock card, no PBS
scenario). Cases: `ghestem.test`.

## What it plays (2026-10-05)

Over their one-level opening on our right: 2NT the two lowest unbid
suits, 3♣ the two highest, the cue bid the highest and the lowest;
five-five or longer, 9+ total points (11+ vulnerable), up to 17 HCP.
Michaels over a one-level opening steps aside (two-suited-overcalls.bid,
`!ghestem`); the Unusual 2NT rules are reused for Ghestem's 2NT (same
pairs). Michaels and Leaping Michaels over preempts are untouched.

Advancer: the longer of partner's suits at the cheapest level, a jump
in a major with four and 10-11, four of a major with three and 12+.
After the cue bid of a major (clubs and the other major) the Michaels
advances apply, since they read only partner's shown major: their 2NT
"which minor?" is then a wasted step (partner's minor is clubs).

## Deviations and open questions

- The field's summary says "a jump cue bid"; Wikipedia's standard
  Ghestem uses 3♣, which is the jump cue bid only over 1♣. We follow
  Wikipedia (3♣ throughout). Question for Rick: or the variant over 1♦
  (2♦ blacks, 3♣ natural, 3♦ majors)?
- Strength copied from our wide-range Michaels; Wikipedia asks for
  "opening values" and notes many play lower or weak-or-strong.
- No advancer cue bid for slam interest yet (Wikipedia: the cue bid
  asks, the Ghestem bidder relays).

## Compare (2026-10-05, `--set competitive.ghestem.play=true`)

Michaels_Cuebid and Michaels_and_Unusual, 100 boards: every top
divergence is Ghestem itself (3♣ for the majors over a minor where BBA
cue-bids, and so on). No bug found.

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only
ghestem-vs-michaels`. IMPs by the errors yardstick, positive when the
convention makes fewer; "actor" is the side that made the first
differing call. Against Michaels (the cards' default), Michaels_Cuebid
and Michaels_and_Unusual: 516 boards changed; actor contract -81,
doubling +17, double-dummy -50, halves -49/-15 (z -0.7): neutral. The
losses are the majors over 1♣ (3♣ for Ghestem against Michaels' 2♣, -118
on 107 boards), the gains the red suits over 1♠ (+65 on 111): the
convention's own trade, no bug.

## Sources

- Wikipedia, "Ghestem" (https://en.wikipedia.org/wiki/Ghestem): the
  three bids and the pairs, the 5-5 requirement, the responses.
- convention-card `spec/conventions/competitive_bidding/ghestem.toml`.
