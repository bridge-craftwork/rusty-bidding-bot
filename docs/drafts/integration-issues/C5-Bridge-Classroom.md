Title: Play my convention card, both ways, with coverage warnings and bidding reports

Part of rusty-bidding-bot's integration plan for the practice tables:
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.3, §3.6, decision 4, Q11).

On non-scenario solo tables with Rusty, the user's **primary card**
(`useConventionCard`) is played by **both sides** by default (Rick's
decision 4). Scenario tables keep CC1/CC2 from the PBN header (C1
(Bridge-Classroom)).

## What exists on the engine side

- The stored `card_data`, or the editor's export
  (`bridge-classroom/card_data@v1`), loads directly as `{json: ...}`;
  `_bbo_raw` and other `_` keys are reported and ignored; the ACBL-card
  fields are card fields under the editor's own paths
  (rusty-bidding-bot 091c8e4, d7d466a).
- `coverage` reports, per side, the settings Rusty reads, ignores, cannot
  map, and the ones that cannot change a call (carding, notes).
- Each card field names its Bridge-Classroom skill (rusty-bidding-bot
  docs/SKILLS.md; the taxonomy proposals are Bridge-Classroom#422).

## Scope

1. Pass the primary card to `createEngine` as `{json: card_data}` for both
   sides; changing the card takes effect on the next board.
2. Show the `coverage` result: the conventions on the card that Rusty does
   not play yet (`ignored`, `unmapped`), by their catalog names, e.g.
   "Rusty doesn't play Smolen yet; partner will treat 3♥ as natural".
3. The BBA fallback cannot read an editor card: it sends `21GF-DEFAULT`
   (Q11, provisional default approved 2026-09-28).
4. "Report a Problem" bundles for a Rusty table include `{rbb version,
   rules_id, cards (the card JSON or its id and hash), dealer, vul,
   auction, seat, hand, fallback positions}`, and a "bidding problem"
   target files them to rusty-bidding-bot.

## Acceptance criteria

- [ ] Changing the primary card changes both sides' bidding on the next
      board.
- [ ] The coverage warning lists the unsupported conventions by their
      catalog names.
- [ ] A fallback on an editor-card table asks BBA with `21GF-DEFAULT`.
- [ ] A filed bidding report reproduces with `rbb call` from the bundle
      alone.

Depends on: C1 (Bridge-Classroom), R1 (rusty-bidding-bot).
