# Responses to a 2NT opening (`two-nt-responses.bid`): notes

Cases: `two-nt-responses.test`.

## Guidance

The 1NT structure one level up, which is what BBA plays on every card in
the corpus: 3♣ Stayman, 3♦ and 3♥ transfers, 3NT to play, 4NT
quantitative, and pass with nothing.

Only the asking calls and opener's answers are written here. Everything
after them was already written against the question — `when answered
majors` in `stayman.bid`, `when answered transfer(M)` in
`jacoby-transfers.bid` — and the strength bands read opener's 20-21, so
responder invites and bids game at the right level without a single new
rule. The calls that would be illegal a level up (the invitational 2NT,
a 3M rebid under the completed transfer) drop out on their own.

## Evidence

`bba-cli --all-meanings` over the Basic_* scenarios, responder's call
after `2N P`:

| call | BBA's meaning | n |
|---|---|---|
| 3♣ | "Stayman" | 37 |
| 3♥ | "transfer" (spades) | 23 |
| 3NT | "calculated bid", 4-10 total points | 16 |
| Pass | 0-4 total points | 16 |
| 3♦ | "transfer" (hearts) | 14 |
| 6NT | 13+ | 4 |
| 4NT | "Quantitative 4NT", 11-12 | 3 |

Before this module responder passed a 2NT opening whatever he held:
4,345 boards across the corpus, 123 of them in Basic_*.

| scenario | calls | auctions | contracts |
|---|---|---|---|
| 2N | 68.8% → **80.2%** | 3.2% → 32.8% | 5.4% → 46.8% |
| 2N_and_Balanced | 69.2% → **81.2%** | 2.0% → 32.2% | 4.0% → 58.4% |
| 2N_and_1_Minor | 69.0% → **75.5%** | 1.2% → 22.4% | 3.2% → 27.2% |
| Basic_* (nine) | 83.1% → **83.6%** | 34.4% → 35.3% | 41.0% → 42.1% |

1N and Basic_NT are unchanged, which is the check that the shared
continuations still behave at the one level.

One ranking fix came with it: in `stayman.bid`, "game, no fit found"
(3NT) and "invitational, no fit found" (2NT) now have `priority -1`, so a
known 4-4 major fit is played in the major. Opposite a 20-21 opening the
game band is wide enough that 3NT was outranking 4♥ on descriptiveness.

## Gaps and open questions

- **Minor-suit transfers over 2NT** (`notrump.two_nt.minor_transfers`,
  on for both 21GF cards) and minor-suit Stayman: not written, so a
  minor one-suiter bids 3NT.
- **Puppet Stayman** (`two_nt.puppet`) is only guarded against, not
  played: the 3♣ rule stands down when the card says puppet, and nothing
  replaces it.
- The 3♠ relay, `two_nt.three_s`, and the four-level transfers.
- Texas over 2NT, and slam bidding beyond the quantitative 4NT.
