# weak-two-responses (`weak-two-responses.bid`): notes

Responses to a weak two, and opener's answer to the 2NT ask. Cases:
`weak-two-responses.test`.

## Guidance (Rick, 2026-09-22)

Both treatments of the 2NT ask, chosen by the card field
`two_level.weak_two_2nt_response`:

- **feature** (the default): opener bids a side suit with the ace or king
  and a maximum, else three of the trump suit;
- **ogust**: 3♣ minimum/poor suit, 3♦ minimum/good suit, 3♥ maximum/poor,
  3♠ maximum/good, 3NT the top three honours.

BBA's import sets `ogust` when its Ogust switch is on (`[[derived]]`).

## Other choices

Responder: 2NT asks with 15+; four-card support raises (preemptive with a
minimum, game with 14+); with three-card support and a strong hand, ask
first. After the answer responder places the contract and never passes an
artificial one: game with 17+, otherwise three of the suit, 3NT when the
answer was higher, or four of a major.

## Gaps

- A new suit by responder (natural and forcing), and slam tries.
- Continuations after the 3NT Ogust answer beyond signing off.
