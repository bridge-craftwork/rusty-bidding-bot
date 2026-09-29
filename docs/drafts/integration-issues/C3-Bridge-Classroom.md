Title: Mouseover meanings from Rusty for every call in the auction

Part of rusty-bidding-bot's integration plan for the practice tables:
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.4, decision 3).

Today the auction grid's tooltips show BBA's meanings by position in
BBA's **predicted** auction, so after a divergence they no longer match
the calls made, and a human's call has none. On a Rusty table, every call
gets its meaning from Rusty over the **actual** auction (Rick's decision
3):

- a human's call: what it would mean to Rusty, read with the human's
  side's card;
- Rusty's calls: what they mean;
- a BBA fallback call: BBA's meaning, unflagged (decision 6).

The engine side exists: `interpret` returns one step per call of an
actual auction (`explanation`, `alert` as `{kind: alert|announce, text}`,
`artificial`, `knowledge.summary`), human calls included, and R1
(rusty-bidding-bot) adds the meaning of any call at any point. C1
(Bridge-Classroom)'s adapter maps them to `meaning`, `meaningExtended`,
`isAlert`, `alertText`, `announce`.

Also:

- show `announce` inline in the cell, and mark alerted calls (the first
  use of `isAlert`);
- render the engine's plain text: calls in its spelling (`2S`, `1NT`) and
  suit letters in ranges (`4+ H`, `2-5 S`) as suit symbols, as
  `formatMeaningHtml` does for BBA's `!S` tokens; the engine never sends
  HTML.

## Acceptance criteria

- [ ] Every call in the auction grid, including the human's, has a tooltip
      when Rusty has a meaning for it.
- [ ] After a divergence, the tooltips match the calls actually made
      (regression test with a fixture).
- [ ] Announcements are visible without hover. Alerts are marked.
- [ ] Suit letters and call names show as symbols; no raw engine tokens
      and no HTML injection.
- [ ] On the solo table everything is shown; who sees which meanings on a
      served table follows Rick's ruling on hidden information (Q6), in
      C6 (Bridge-Classroom).

Depends on: C1 (Bridge-Classroom), R1 (rusty-bidding-bot).
