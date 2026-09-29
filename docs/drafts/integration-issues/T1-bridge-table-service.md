Title: Native Rusty bidder with the BBA fallback

Part of rusty-bidding-bot's integration plan for the practice tables:
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§1.3, §3.1 B, §3.2, §5 item 3).

Rusty is a rule-based bidding engine in Rust. Like `bridge-rulebot` for
cardplay, the service links it as a native crate, with the rules built in
(R5 (rusty-bidding-bot)).

## Scope

Add a per-session and per-room `BidderMode { Bba, Rusty }` (default
`Bba`), set by a `{"t":"set_bidder","bidder":"rusty"}` host frame and by
an optional `bidder` on `POST /admin/sessions`. The bidder plays every bot
seat (Rick's decision 2).

- For Rusty, `choose_call()` calls the engine directly with the seat's
  hand and the calls so far. It needs no prefix cache and no HTTP.
- **When Rusty has no rule**, `choose_call()` asks the existing BBA client
  (`src/bots/bba.rs`) with the calls so far and uses BBA's call for that
  seat (decision 6), with the session's named `.bbsa` cards, or
  `21GF-DEFAULT` when it has none (Q11, provisional default approved
  2026-09-28). The event is not flagged to clients; it is recorded with
  `record_event()` and counted in the metrics.
- Engines are cached per session and card pair.
- Illegal or failed calls, including a failed fallback, fall back to Pass,
  as today, and are logged with `record_event()`.
- Bot-call events add `explanation`, `alert` and `announce` in the
  engine's JSON shape (BBA's meaning for a fallback call).
- The welcome frame carries `bidder` and `bidder_version` (the engine's
  `version`, `rules_id` and commit).
- PlayOnly and PassBot behave as they do with BBA.

## Acceptance criteria

- [ ] A Rusty table plays a board end to end; BBA traffic appears only for
      no-rule positions (checked in the metrics and logs).
- [ ] Same board, same auction, same seat: the same call at every table in
      a session (test), fallback calls included while BBA answers.
- [ ] Undo and changes during a bot's turn stay safe (the existing
      seq-recheck pattern).
- [ ] The CI-parity build (`./dev-build.sh --ci test`) passes with the
      pinned rev. The committed `Cargo.lock` has no local paths.

Depends on: R5 (rusty-bidding-bot). Used by: C6 (Bridge-Classroom).
