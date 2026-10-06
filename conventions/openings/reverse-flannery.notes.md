# Reverse Flannery: notes

## Decisions

- Responses, not openings: 1m–2♥ (5♠ 4-5♥, 5-8 HCP) and 1m–2♠ (5♠ 4♥,
  9-10), as the Dural Bridge page and the PBS scenario define it. The
  convention-card summary calls it "an opening of 2♥ (or 2♦)"; the card
  fields `two_level.reverse_flannery.two_h/.two_s` come from BBA's
  `Reverse Flannery 2H/2S`, which BBA plays as these responses.
- Priority 2: they outrank the one-level response and any jump shift
  (strong or weak) on the same call.
- Over 2♥ opener passes only with longer hearts and at most two spades,
  else 2♠; 2NT asks from 16; game in a major from 18.
- Over 2♠ (BBA, scenario): pass with two spades and a minimum; 3♠ with
  three or more and a minimum (a game try, responder accepts with 10);
  4♠ with 15 support points.

## BBA (compare Reverse_Flannery --limit 50, 2026-10-05: 92% of calls agree)

- BBA bids 2♠ over 2♥ with three-card hearts too: BBA style.
- Thresholds for 3♠ vs 4♠ over 2♠ differ by a point: don't care.
- The 1m–2♥–3♦ divergences are the opponents' (EW) calls.

## Open questions

- The convention-card summary describes Reverse Flannery as an opening;
  should it say "response to 1m"? (convention-card PR, not done here.)

## Sources

- Dural Bridge, "Flannery and Reverse Flannery",
  https://duralbridge.com/flannery.ing.htm: the two responses and the 2NT
  ask (3♣/3♦/3♥/3♠). Opener's ranges for game and the 2♠ continuations
  are ours (from the scenario's BBA auctions).
- Practice-Bidding-Scenarios `btn/Reverse_Flannery.btn` and `bba/`.
