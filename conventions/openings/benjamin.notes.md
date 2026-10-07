# Benjamin twos: notes

## Decisions

- 2♣ = 22-23 (any shape), or unbalanced 20-23 with a good six-card suit
  or a 5-5/6-5 two-suiter and at most four losers. 2♦ = 24+, or 23
  without a five-card suit. These are BBA's ranges (Benjamin_2D corpus);
  Wikipedia and the PBS chat put game-in-hand hands (fewer HCP, a long
  solid suit) in 2♦, BBA opens them 2♣. We follow BBA here because the
  source gives no number for "game in hand".
- The strong 2♣ openings of strong-openings.bid are off (`!benjamin`).
- 2♣ keeps the strong 2♣'s machinery: game forcing, 2♦ waiting, the
  positives and the second negative. Benjamin's 2♣ is really forcing for
  one round and passable after opener's rebid; written that way, half of
  the continuations had no rule (no generic non-forcing structure after
  2♣–2♦–2M). Open: write the passable version.
- Responses to 2♦ (PBS chat, BBA): 2NT negative (0-6, no five-card
  major or six-card minor), 2♥/2♠ five+ and 3♣/3♦ six+ at any strength,
  3NT 7+ without them. After 2♦–2NT opener bids naturally (3NT
  balanced, 4NT 28+ quantitative).
- 2♦ also opens 22-23 with at most one loser (game in hand; BBA 4 of 4).
- Weak 2♦ off (preempts.bid).

## BBA (compare Benjamin_2D --limit 50, 2026-10-05: 76% of calls agree)

- Opening 1♠/1♣ vs our 2♣ (5 boards): BBA opens some 20-21 two-suiters
  at the one level; don't care.
- 2♦–P: BBA's thresholds for the positives differ a little: don't care.
- The scenario's card (21GF-PolishTwoSuiters) also plays Polish
  two-suiters and Namyats.

## Convention score (2026-10-07)

`probes/tools/conv_ab.py --only benjamin` (Benjamin_2D, NS with Benjamin
on and off, BBA and us): poor before, Rusty's gain -215, BBA's +52, net
-0.68 per changed board (393 boards). Our 2♦ auctions lost the slams
(and some games) that the strong 2♣ structure finds:

- **No rules after a positive.** Opener had nothing after 2♦–2♥/2♠/3♣/3♦
  and the base fallback bid 3NT, passed (2♦-2♥-3NT on boards with a
  grand slam). Now opener raises with three (agreeing the suit, so the
  keycard ask follows), shows a five-card suit, or bids notrump;
  responder raises opener's suit with three, rebids a six-card major or
  a seven-card minor, else notrump.
- **2♦–3NT was passed** with 24+ facing 7+. BBA bids slam on all 41
  corpus boards, mostly 5NT ("6NT, or 7NT with a maximum"). Now: 7NT
  with 37 between us, else 5NT; responder bids 7NT with 12+.
- **The positives had a 5 HCP floor** read from the chat's 2♣
  responses. The chat gives none for 2♦ ("2♥/♠ 5+♥/♠", "2NT 5- HCP, no
  5-card major"), and BBA bids every five-card major from 0 HCP. So a
  weak five-card major bid 2NT and the fit was lost.
- **Weak raises.** After 2♦–2M–3M responder with 0-5 bids 4M (fast
  arrival); opener over 4M (and over 2♦–2NT–3M–4M) passes with three
  losers or more (BBA: asked on 4 of 4 with two or fewer, passed 5 of 6
  with three or more), else the keycard ask.
- Game-in-hand 22-23s (one loser) open 2♦, as BBA does.

| | Rusty's gain | BBA's gain | net / changed board | halves |
|---|---|---|---|---|
| before | -215 | +52 | -0.68 (393) | agree |
| after | +109 | +52 | +0.14 (393) | agree (+47/+10) |

Background (Benjamin_2D, vs BBA by par): -726 -> -402 IMPs (-1.45 ->
-0.80 a board). What is left is slam bidding after the 2♦ positives
(responder's cue-bids on weak hands, keycard answers with five
keycards) and the opening split between 2♣ and 2♦.

## Open questions

- Should 2♣ be passable after 2♣–2♦–2M / 2NT (true Benjamin)? We keep
  the game force for now.
- Game-in-hand hands with fewer than 24 HCP: 2♦ (Wikipedia) or 2♣ (BBA)?
  We follow BBA: 2♣, except 22-23 with one loser or none, which BBA
  opens 2♦ too (2026-10-07).

## Sources

- Wikipedia, "Benjamin Twos", https://en.wikipedia.org/wiki/Benjamin_Twos:
  the meanings of 2♣ and 2♦.
- Practice-Bidding-Scenarios `btn/Benjamin_2D.btn` (the responses to
  2♦: no strength floor for the suit positives) and
  `bba/Benjamin_2D.pbn` (ranges; BBA's 3NT positive from 7, the 5NT
  slam after it, the loser count over a weak game raise: corpus counts
  above, 2026-10-07).
- Where we differ: the 2♦ continuations after a positive are our own
  (natural, the slam modules decide); BBA cue-bids more.
