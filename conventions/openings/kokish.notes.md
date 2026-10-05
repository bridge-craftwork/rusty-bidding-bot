# Kokish relay: notes

## Decisions

- 2♣–2♦–2♥ = five or more hearts (unbalanced), or 24-25 balanced: the
  hand our ladder (strong-openings.bid) rebids 3NT. Bridge Bum's relay
  2NT is "25+"; on our ladder 26-27 opens 3NT, or with a Gambling 3NT
  opens 2♣ and rebids 3NT directly, and 28+ rebids 4NT. BBA (Kokish
  scenario): relay-2NT is exactly 25, direct 3NT 26-27, which matches.
- Balanced with five hearts: 2NT (22-23) or the relay (24-25).
- Responder's 2♠ is forced (priority 2, above the second-negative 3♣).
- After the two-suiter responder agrees hearts at the three level with
  three, raises the second suit with four, else 3NT; over 3♥ (six or
  more), 4♥ with two, else 3NT.

## Fixed on the way (strong-openings.bid)

- With a Gambling 3NT on the card, a balanced 26-27 had no opening at
  all (the 22+ 2♣ excluded it for the 3NT opening) and passed: 10 of the
  first 50 Kokish_Relay boards. It now opens 2♣ and rebids 3NT (BBA does
  the same). Affects every card with `Gambling = 1`.

## BBA (compare Kokish_Relay --limit 50, 2026-10-05: 87.8% of calls agree)

- After the relay's 2NT BBA plays Stayman (3♣) and transfers; we have no
  system over it and responder bids 3NT. Open: systems on over 2♣–2♦–2NT
  and the Kokish 2NT belong to the notrump modules.
- Divergences at 2♣–P (positive responses) and in the opponents'
  overcalls are not Kokish's.

## Sources

- Bridge Bum, "Kokish Relay", https://www.bridgebum.com/kokish_relay.php:
  the relay, the forced 2♠, the rebids. Deviation: the balanced range
  (24-25 instead of 25+) to fit our ladder.
- Practice-Bidding-Scenarios `btn/Kokish_Relay.btn` and `bba/`.
