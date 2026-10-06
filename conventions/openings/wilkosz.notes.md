# Wilkosz 2♦: notes

## Decisions

- Five or more in two suits, at least one a major (not both minors),
  5-10 HCP, seats 1-3. BBA opens it from 4 (alert: 4-10 total points);
  we start at 5, as the weak twos. The weak 2♦ is off (preempts.bid);
  the weak 2♥/2♠ stay.
- Responses follow BBA (probes/b2-Wilkosz-resp.toml) where it bids, and
  Matula's scheme (BBO forum) elsewhere:
  - 2♥ pass or correct (opener 2♠ without hearts), 2♠ pass or correct
    with spades longer (opener bids his minor without spades);
  - 3♣ six clubs to play; 3♦ invitational with three in both majors
    (opener 3M minimum, 4M with 9-10); 4♦ "bid your major" (12-13);
  - 2NT 14+ asks: 3♣ clubs and a major (3♦ then asks which),
    3♦ diamonds and hearts, 3♥ both majors, 3♠ spades and diamonds.
  - After 2♥–2♠ responder passes with two spades or bids 3♣ pass or
    correct.

## Open questions (decision taken)

- 3♥/3♠ preemptive pass-or-correct (Matula) and 3NT to play are not
  written; BBA never chose them in the probe.
- Maximum/minimum in the 2NT answers: not shown.
- No PBS scenario; checked with `.test` cases and the probes only.

## Sources

- BBO Forums, "Wilkosz responses" (Matula's scheme),
  https://www.bridgebase.com/forums/topic/53084-wilkosz-responses/.
- BBO Forums, "Responses to Polish two bids" (the definition: 5+/5+, any
  two suits but the minors, about 7-10).
- BBA as a black box: probes/b2-Wilkosz-openings.toml,
  probes/b2-Wilkosz-resp.toml (2026-10-05).
