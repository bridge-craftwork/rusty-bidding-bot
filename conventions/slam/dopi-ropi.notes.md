# dopi-ropi (`dopi-ropi.bid`): notes

Answering an ace or keycard ask after the opponents interfere
(2026-10-05). Gated on `slam.dopi` (they bid a suit) and `slam.ropi`
(they double 4NT). Every PBS card plays both except Basic-Bridge (ROPI
only).

## What the rules do

- **DOPI** (their suit bid over 4NT, up to 5♠): double = the first
  step, pass = the second, the next two bids = steps three and four.
  With 1430 a double is "1 or 4", a pass "0 or 3"; the card's 0314
  treatments swap them as `rkcb-1430.bid` does. Standard Blackwood:
  double 0 or 4 aces, pass 1, then 2 and 3.
- **ROPI** (they double 4NT): redouble = first step, pass = second, 5♣
  and 5♦ steps three and four.
- **The asker** with two keycards (aces) missing defends: passes
  partner's double, or doubles over partner's pass. An open count
  ("0 or 3" facing my two) is read low and the double sets
  `keycard_correct`, so partner bids six with the higher count (as the
  5-level sign-off does). Over ROPI's redouble the asker always bids.
- **Reading 4NT in competition** (`rkcb-1430.bid`, `read_only`): the
  asks there are off once they have bid ("in competition it cost"), so
  BBA's contested 4NT read as nothing and we never answered it. A
  read-only copy now makes it keycard for the suit we play in, and for
  the trump after total-tricks.bid's `ask=signoff` jump to game
  (1H-2NT-(3C)-4H-4NT). We still never ask in competition ourselves.

## Corpus (DOPI_ROPI, Exclusion_*, --limit 50, 21GF cards)

At the answer to BBA's 4NT: 24/36 agreed on DOPI_ROPI before the reading
rules (most misses "no rule" for BBA's contested 4NT), 34/44 over the
three scenarios after. What is left is reading: 4NT after a strong 2♣
and their overcall, and 5♥/5♠ answers where BBA counts the queen
differently. The asker's next call differs mostly outside DOPI: BBA's
5NT king ask (now built), BBA passing a 5♠ answer with a keycard
missing, and grand slam judgment.

The reading rules are `read_only` (rkcb-1430.bid): a first version
ranked them at -60 only, below "every pass", but in a game force no pass
is offered and we bid them (25 boards in the tripwire, contract errors
136 -> 291). Now never chosen.

**Engine change (2026-10-05):** a pass a rule marks `artificial` is
allowed in a force (engine.rs `choose_from`). Without it DOPI's pass was
impossible in any game-forcing auction (Jacoby 2NT, a splinter): the
engine forbids a pass while the opponents hold the contract below our
game.

## Open questions

- **DEPO** (`slam.depo`, 2026-10-05): double an even number of
  keycards (aces), pass an odd number, over their suit bid. The spec
  says "played instead of DOPI", so with `slam.depo` on it replaces DOPI
  at every level (some pairs play DOPI low and DEPO high; the card has
  no field for that split). ROPI is unchanged. The asker's DOPI rules
  apply as they are: over an even double facing two of mine the count
  stays open (2 or 4) and we sign off, and partner's correction to six
  needs three keycards, so the slam with two and two is missed.
  **Question for Rick:** worth a DEPO-specific correction? Decision
  taken: no, rare.
- Interference at the six level is not answered by DOPI (the rules stop
  at 5♠); a double there is whatever the competitive rules say.
- The asker's continuation over an open "1 or 4" double (asker with no
  keycard) passes; a correction structure there is not written.

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only
depo-vs-dopi`. IMPs by the errors yardstick, positive when the
convention makes fewer; "actor" is the side that made the first
differing call. DEPO against DOPI (`slam.depo` on, `slam.dopi` on in
both runs), DOPI_ROPI: 0 boards changed. Our 4NT came up on 394 boards
and the opponents never bid over it: our defenders do not interfere over
keycard, so the scenario's interference exists only in BBA's auctions.
No self verdict possible; the `.test` cases are the check.

## Sources

- **The convention:** ACBL Unit 390, "DOPI and ROPI"
  (https://www.acblunit390.org/Simon/dopi-ropi.htm): double/redouble the
  first step, pass the second, then the steps; convention-card
  `spec/conventions/bidding_conventions/dopi_ropi.toml` (Kantar, *25 More
  Bridge Conventions You Should Know*, ch. 15; Todd, Advancing in Bridge
  #536). Where we differ from Unit 390: its 1430 table lists a "3rd
  cheapest = three keycards", which repeats the pass; we use four steps.
- **DEPO:** convention-card
  `spec/conventions/bidding_conventions/depo.toml` (Kantar, *25 More
  Bridge Conventions You Should Know*, ch. 15): double even, pass odd,
  instead of DOPI.
- **BBA:** DOPI_ROPI corpus notes ("A=1/5 or 4/5" for a double over 1430)
  agree with the step order.
