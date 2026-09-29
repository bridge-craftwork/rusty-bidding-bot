Title: Bid a practice table's bot seats: `auction` entry point, meanings for any call, determinism test

Part of the Bridge-Classroom integration plan (issue #3):
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.4, §4, §5).

**Status: in progress.** This is the engine API Bridge-Classroom's solo
table needs to let Rusty bid its bot seats. It may already be merged by
the time this issue is filed; if so, close it with the commit.

## Already done (not part of this issue)

The first draft of this issue asked for a whole WASM API. Most of it
landed in docs/WASM.md (API version 1):

- cards as a stock name (`"21GF-DEFAULT"`), `{bbsa: text}` or
  `{json: card_data}` / the editor's `bridge-classroom/card_data@v1`
  export, with problems as `diagnostics` (c350a6c, 091c8e4);
- `createEngine` handles, reused for the same cards (c350a6c);
- `bid` with `explanation`, `alert` (`{kind: alert|announce, text}`),
  `rule`, `candidates` and their `outcome`; `interpret` for every call of
  an actual auction, human calls included, with `artificial`,
  `knowledge.summary` and the `position` (1e57c2d);
- `bidDeal` from a `prefix`; `coverage` (the card report); `info()` with
  `api`, `version`, `rules_id`;
- calls `1N`/`1NT`, vul `Both`; never throws; `wasm-pack --target web`;
  release builds attach `rbb-wasm.tar.gz` (0f6c2b3, docs/RELEASING.md).

## What this issue adds

1. **An `auction` entry point** (native and WASM) that takes the deal (or
   the bot seats' hands), dealer, vul, scoring, the calls so far and which
   seats are human, and bids the bot seats **until**:
   - the auction ends;
   - a human seat is to call; or
   - a bot seat reaches a position where **no rule applies**. It stops
     there instead of passing (today `bid`/`bidDeal` return `Pass`, "No
     rule applies", `rule: null`), so the client can take BBA's call for
     that seat (Rick's decision 6) and call `auction` again with the
     longer prefix. The engine never calls BBA.

   The response says why it stopped and at which seat, and carries each
   new call's meaning in the step shape `interpret` uses.
2. **The meaning of any call**: what a given call would mean at a given
   point in an auction, whether or not it was made (the mouseover for a
   human's call, the "why" panel), in the same step shape; `null` fields
   when no rule gives it a meaning.
3. **A golden determinism test**: the same inputs (hand, auction, cards)
   give the same call, explanation and meaning natively and in WASM, and
   across runs. Candidate ties break on a stable key, never on `HashMap`
   order.
4. docs/WASM.md documents both entry points with an example of the
   stop-and-resume loop.

## Acceptance criteria

- [ ] `auction` stops at a human seat's turn, at the end of the auction,
      and at a no-rule position, and reports which (tests for each).
- [ ] It resumes from a prefix that contains a call it did not make (a
      human's call, a BBA fallback call) and reads that call like any
      other (test).
- [ ] The meaning of a call no rule would make comes back with `null`
      explanation and rule, not an error (test).
- [ ] The determinism test runs in `cargo test` (native) and against the
      WASM build in CI.
- [ ] docs/WASM.md is updated; `api` stays 1 if only new functions and
      optional fields are added.
- [ ] A tagged release (pre-release is fine) carries it in
      `rbb-wasm.tar.gz`, so Bridge-Classroom can pull it: see C1
      (Bridge-Classroom).

Used by: C1 (Bridge-Classroom), C3 (Bridge-Classroom), C4
(Bridge-Classroom), R5 (rusty-bidding-bot).
