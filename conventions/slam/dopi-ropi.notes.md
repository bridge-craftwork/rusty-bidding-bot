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
- **Reading 4NT in competition** (`rkcb-1430.bid`, priority -60): the
  asks there are off once they have bid ("in competition it cost"), so
  BBA's contested 4NT read as nothing and we never answered it. A
  read-only copy now makes it keycard for the suit we play in, and for
  the trump after total-tricks.bid's `ask=signoff` jump to game
  (1H-2NT-(3C)-4H-4NT). We still never ask in competition ourselves.

## Corpus (DOPI_ROPI, --limit 50, 21GF-DEFAULT)

At the answer to a 4NT ask: 24/36 agreed with BBA before the reading
rules; the misses were almost all "no rule" for BBA's contested 4NT,
fixed by the reading rules. The asker's next call (7/36) differs mostly
for reasons outside DOPI: BBA's 5NT king ask (now built), BBA passing
a 5♠ answer with two keycards missing where we bid six (see
rkcb-1430.notes.md), and grand slam judgment.

## Open questions

- **DEPO** (`slam.depo`: double even, pass odd, when their suit
  outranks ours) is not built; DOPI applies whatever their suit.
- Interference at the six level is not answered by DOPI (the rules stop
  at 5♠); a double there is whatever the competitive rules say.
- The asker's continuation over an open "1 or 4" double (asker with no
  keycard) passes; a correction structure there is not written.

## Sources

- **The convention:** ACBL Unit 390, "DOPI and ROPI"
  (https://www.acblunit390.org/Simon/dopi-ropi.htm): double/redouble the
  first step, pass the second, then the steps; convention-card
  `spec/conventions/bidding_conventions/dopi_ropi.toml` (Kantar, *25 More
  Bridge Conventions You Should Know*, ch. 15; Todd, Advancing in Bridge
  #536). Where we differ from Unit 390: its 1430 table lists a "3rd
  cheapest = three keycards", which repeats the pass; we use four steps.
- **BBA:** DOPI_ROPI corpus notes ("A=1/5 or 4/5" for a double over 1430)
  agree with the step order.
