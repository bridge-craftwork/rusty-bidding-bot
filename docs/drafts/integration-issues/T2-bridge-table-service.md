Title: Convention cards per session

Part of rusty-bidding-bot's integration plan for the practice tables:
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.3, decision 4, Q3, Q11).

Today `src/bots/bba.rs` sends one hardcoded card, `21GF-DEFAULT`, for both
sides, and session create carries no cards.

## Scope

- Accept `cards: {ns, ew}` on `POST /admin/sessions`, or a single `card`
  that both sides play (the default, Rick's decision 4). Each is a card
  spec in the engine's form (R5 (rusty-bidding-bot)): a built-in name
  (`"21GF-DEFAULT"`, the 18 PBS cards), `{bbsa: text}`, or
  `{json: card_data}` (the stored card or the editor's export).
- A session made from scenario boards may take its names from the PBN's
  `% CC1` / `% CC2` headers.
- Rusty bids with the session's cards.
- BBA, and BBA's fallback calls for Rusty (T1 (bridge-table-service)), get
  the session's cards when they are `.bbsa` names, else `21GF-DEFAULT`
  (Q11, provisional default approved 2026-09-28). This replaces the
  hardcoded card in `src/bots/bba.rs`.
- The companion change in bridge-classroom-api (`table_sessions.rs`,
  `service_create_payload`) is tracked in C6 (Bridge-Classroom); which
  card it sends (owner's primary or teacher's choice) is Q3, open.

## Acceptance criteria

- [ ] A session created with one card bids with it on both sides; one
      created with two cards bids each side with its own; one created
      without cards keeps today's default.
- [ ] Invalid card specs are rejected at session create with the engine's
      diagnostics, not at the first bot call.
- [ ] BBA requests carry the session's names, or `21GF-DEFAULT` for an
      editor card.
- [ ] Cards are visible in the dashboard or session info for debugging.

Depends on: R5 (rusty-bidding-bot), T1 (bridge-table-service). Used by:
C6 (Bridge-Classroom).
