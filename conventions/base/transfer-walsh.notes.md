# transfer-walsh (`transfer-walsh.bid`): notes

Transfer Walsh (`minor_openings.one_club.transfer_resp`): over 1♣, 1♦
shows hearts, 1♥ spades, 1♠ no four-card major (four diamonds, or a game
force with longer diamonds); 1NT stays natural. Cases:
`transfer-walsh.test`. Scenario: Transfer_Walsh (BBA plays it on
21GF-DEFAULT without the switch, so its auctions there are natural: only
our problems count, not agreement).

## Structure

- Responder bids hearts first with four of each major; spades first with
  five spades. Off over interference (the natural/negative-double rules).
- Opener: accepts with three of the major or four and a minimum (1M);
  raises to 2M with four and 16-18, 4M with 19+; else natural (1♠ over
  1♦, 1NT 12-14, 2♣ six clubs, 2NT 18-19, a reverse 17+), each denying
  three of the major.
- Responder's second call: after an acceptance, pass weak, 2M with five
  and 8-10, 3M invites with five, 4M game; 2NT/3NT with four. After a
  denial, natural (2M/3M/4M with six, 2NT/3NT). After a reverse: 2NT
  minimum, 3♣/3 of opener's suit game forcing, 3NT, or the five-card
  major.
- The natural rules that read the response as a suit (rebids.bid,
  responder-rebids.bid, checkback, NMF, Roudi, two-way NMF, XYZ, FSF,
  Wolff, NMF after 2NT) stand aside after 1♣ under this card: `define
  tw(x)` in rebids.bid.

## Corpus (Transfer_Walsh with the switch on, 50 boards, 2026-10-05)

Divergences at 1♣ P are all the transfer itself (BBA natural). Problems
in our auctions fixed: the reverse over a transfer and responder's
four spades after an acceptance had no rule. One remains in competition
(1♣ 2♦ 2♥ P 2♠, not ours).

## Gaps and questions for Rick

- The 1♠ response: Wikipedia has it as diamonds; convention-card's
  summary as "no four-card major". We take both: four diamonds and no
  major, or a game force with longer diamonds. A balanced 6-10 without a
  major bids 1NT.
- Opener's 1M acceptance with four and a minimum (as 1♣-1♥-2♥ would be
  for many): we chose 1M with any minimum, 2M with 16-18. Some play 2M
  as the minimum four-card raise.
- Slam auctions and the modules under slam/ and majors/ that read
  `after 1x (P) 1M (P)` literally (splinters by opener) are not gated.

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only
transfer-walsh`. IMPs by the errors yardstick, positive when the
convention makes fewer; "actor" is the side that made the first
differing call. Against natural Walsh, Transfer_Walsh and 1C_WalshStyle:
711 boards changed. First -338 to the actor (z -3.2), three gaps: (1)
after 1♣-1x-1NT responder had only pass/2NT/3NT, so the game force with
longer diamonds and a major (1♠ first) never showed the major, and 19-21
balanced stopped in 3NT (-93 on 75 boards); now the responder's reverses
as in natural Walsh, opener raising with four, and a quantitative 4NT /
6NT (also after 1♣-1♦-1♠); (2) responder had no rule over opener's 3♣
(six clubs, 16-18) and passed it with 13-14; (3) opener's splinters
(splinters.bid) read 1♣-1♦/1♥ as natural and agreed the wrong major;
they now stand aside under Transfer Walsh. After: 703 changed; actor
contract -137, doubling 0; double-dummy -184; halves -172/+35 (z -1.4):
neutral. Left: no opener's splinter under Transfer Walsh (natural Walsh
found two slams by 1♣-1♥-4♦), and the game raise 4M with 19+ that
responder cannot judge for slam.

## Sources

- **Wikipedia, "Transfer Walsh"** (https://en.wikipedia.org/wiki/Transfer_Walsh):
  1♦ hearts, 1♥ spades, 1♠ diamonds, 1NT 6-9 balanced; opener accepts
  with three, jumps with extras. convention-card
  `spec/conventions/bidding_conventions/transfer_walsh.toml` cites it.
- **Where we differ:** ranges (accept up to 15 support points, 2M 16-18)
  and responder's continuations follow our natural responder rebids
  (standard practice, not yet cited).
