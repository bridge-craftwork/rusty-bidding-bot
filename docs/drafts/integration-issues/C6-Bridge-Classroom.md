Title: Served ("distributed") table: bidder choice, labels and meanings

Part of rusty-bidding-bot's integration plan for the practice tables:
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.2, §3.5, decisions 2, 3, 4 and 5).

The served table is Rick's "distributed table" (decision 5): humans on
different computers at one table, the bot seats bid by
bridge-table-service (T1 (bridge-table-service)).

## Scope

- A host control for the session bidder (BBA or Rusty), one bidder for all
  bot seats (decision 2), sent as the `set_bidder` frame of T1.
- `botLabelFor()` (`serverEngine.js:398-406`) shows the actual bidder
  instead of the hardcoded `"BBA+RulesBot"`.
- `AuctionTable` on the served table gets `meanings` from the
  explanations on bot-call events (T1), and the mouseover meaning of human
  calls from the client's Rusty (C3 (Bridge-Classroom)), within Rick's
  ruling on hidden information (Q6, open).
- The client's own Rusty readings are shown only when the client's
  `rules_id` matches the service's `bidder_version` from the welcome
  frame; otherwise they are hidden.
- The "you vs reference" overlay stays **BBA's**, computed as today (Q2,
  provisional default approved 2026-09-28).
- bridge-classroom-api (`table_sessions.rs`, `service_create_payload`)
  passes the chosen card at session create, one card for both sides by
  default (decision 4); whether it is the owner's primary card or the
  teacher's choice is Q3 (open). The service side is T2
  (bridge-table-service).

## Acceptance criteria

- [ ] A host can switch the bidder between boards. Seat labels follow.
- [ ] Bot calls on a Rusty table show explanations for every viewer allowed
      to see them.
- [ ] No Rusty call is labelled "BBA" (a fallback call is not labelled
      either, decision 6); the reference overlay is labelled BBA.
- [ ] With a mismatched client version, the client-side Rusty readings are
      hidden and nothing else breaks.

Depends on: T1 (bridge-table-service), T2 (bridge-table-service), C3
(Bridge-Classroom).
