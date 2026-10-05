# kickback (`kickback.bid`): notes

Kickback (2026-10-05). Gated on `slam.kickback.play`, `.rkcb_1430` or
`.rkcb_0314` (any of the three; the `.bbsa` keys "Kickback 1430" and
"Kickback 0314" set the last two). No stock card plays it.

## What the rules do

- **The ask:** with a suit agreed, the step above four of the trump
  suit: 4♦ for clubs, 4♥ for diamonds, 4♠ for hearts; spades keep 4NT
  (rkcb-1430.bid). The values are rkcb-1430.bid's (33 on partner's
  maximum when he is limited, with no two bare side suits; 33 on his
  floor in a game force). After a heart control-bid exchange, as the 4NT
  ask there. Available while the auction is below the ask; once past
  it, 4NT is keycard again.
- **Priority 11:** the engine reads a call by its highest-ranked rule,
  so the ask must outrank anything else that could claim 4♦/4♥/4♠. It
  stays below a control bid (12): a hand with an uncontrolled suit
  control-bids first, as with 4NT.
- **Answers:** four steps, 1430 (0314 with `rkcb_0314`): first step "1
  or 4", second "0 or 3", third two without the queen, fourth two with
  it. The fourth step is always five of the trump suit.
- **The asker:** as after 4NT: grand with every keycard and the queen,
  six with at most one missing, sign off in five otherwise (a pass when
  the answer is five of our suit), partner correcting to six with the
  higher count of an open answer (`ask=kb_correct`).
- rkcb-1430.bid's trump-agreed asks and control-bids.bid's past-game 4♠
  control bid (hearts trumps) step aside when Kickback applies
  (`kb_asks`, `kb_spade_ask`).
- This is also the cure for the club problem in rkcb-1430.notes.md (an
  answer past 5♣ leaves no sign-off): with Kickback every answer leaves
  five of the minor.

## Gaps and open questions

- No queen ask (the next step over a "1 or 4"/"0 or 3" answer) and no
  king ask; DOPI/ROPI over interference with the Kickback ask is not
  written (the ask is simply lost).
- A fit known but not agreed (rkcb-1430.bid's `kc_open`, `shown_fit`)
  still asks with 4NT, also in a minor. **Question for Rick:** should
  Kickback extend there? Decision taken: no, to keep 4NT unambiguous
  where nothing set the trump suit.
- Kickback for hearts as well as the minors follows the convention-card
  summary and Todd. Some partnerships play it in the minors only
  ("Redwood"). **Question for Rick:** minors only? Decision taken: all
  three, as the card's source.

## Corpus

No PBS scenario plays Kickback, and BBA's 21GF cards have it off. Checked
with `--set slam.kickback.play=true` on Minor_Game_Or_Slam and
Slam_After_Major_Fit (`--limit 50`): the ask came up 3 times in the
minors and 8 times in hearts, answered and placed as written; no final
contract changed (same par scores as without Kickback).

## Sources

- **The convention:** Robert S. Todd, "Kickback Keycard Ask",
  Advancing in Bridge #582
  (https://www.advinbridge.com/this-week-in-bridge/582): 4♦ asks in
  clubs, 4♥ in diamonds, 4♠ in hearts; 1430 step answers. Convention-card
  `spec/conventions/bidding_conventions/kickback.toml`.
- **Where we differ:** no queen or king ask yet.
