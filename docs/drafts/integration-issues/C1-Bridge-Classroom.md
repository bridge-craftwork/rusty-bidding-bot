Title: Rusty bidder in a Web Worker, pulled from rusty-bidding-bot's release, with the BBA fallback

Part of rusty-bidding-bot's integration plan for the practice tables:
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.1 A, §3.4, §5 items 1–2).

Rusty is a rule-based bidding engine (Rust, native and WASM). This issue
lets it bid the bot seats of the solo practice table, behind the engine
setting of C2 (Bridge-Classroom). BBA stays the default.

## Rick's decisions that apply

- Decision 2: one engine bids every non-human seat.
- Decision 6: where Rusty has no rule, take BBA's call for that seat,
  unflagged.
- Q7 (provisional, 2026-09-28): pull the WASM package from
  rusty-bidding-bot's GitHub release assets; do not hand-vendor it.
- Q11 (provisional): BBA's fallback calls use the scenario's own `.bbsa`
  cards (CC1/CC2), or `21GF-DEFAULT` when the table has none.
- Q2 (provisional): the student is still marked against BBA. This issue
  does not change the reference: LocalEngine keeps asking BBA for its
  predicted auction for marking.

## What exists on the engine side

The WASM API (rusty-bidding-bot docs/WASM.md, API version 1):
`createEngine`, `bid`, `interpret`, `bidDeal`, `coverage`, `info`; card
specs as a stock name (`"21GF-DEFAULT"`, the 18 PBS cards are built in),
`{bbsa: text}`, or `{json: card_data}` (the stored `card_data` or the
editor's `bridge-classroom/card_data@v1` export, loaded as is). JSON
strings in and out; problems come back as `diagnostics`, nothing throws.
R1 (rusty-bidding-bot) adds the `auction` entry point this issue uses: it
bids the bot seats until the auction ends, a human is to call, or a bot
has no rule. The showcase at bridge-craftwork.com/rusty-bidding-bot/
(`web/lib/tool.js` in that repo) is a working client to crib from.

## Scope

1. **Fetch the package from a release.** A script (run by `npm ci` /
   `prebuild` and CI) downloads `rbb-wasm.tar.gz` from a pinned
   rusty-bidding-bot release tag, checks its SHA-256 against the value
   committed beside the tag, and unpacks it where Vite serves it. Upgrading
   is a one-line change of tag and hash. Record the tag, `api` and
   `rules_id` in the app's build info.
2. **`src/utils/rbbClient.js`**, a promise API over a module Web Worker
   that loads the WASM lazily, on the first Rusty table only, and caches
   engines per card pair (`createEngine` handles).
3. **Bot seats bid with `auction`.** LocalEngine, when the engine setting
   is Rusty, calls `auction` with the deal, the calls so far and the human
   seat. At each **no-rule stop** it asks `bbaClient.fetchAuction()` with
   the calls so far as `auctionPrefix` and the Q11 cards, takes BBA's next
   call and meaning for that seat, and calls `auction` again. Because
   `auction` stops at the human's turn, nothing is predicted: divergence,
   toggle and undo send no new bidding request (they may still refresh
   BBA's reference, Q2). This absorbs the "per-seat bidding" half of the
   earlier C4 draft.
4. **The adapter** from Rusty's steps to the fields `AuctionTable`
   already takes: `explanation` → `meaning`, `knowledge.summary` →
   `meaningExtended`, `alert.kind == "alert"` → `isAlert` + `alertText`,
   `alert.kind == "announce"` → `announce`, `artificial`, a no-rule stop
   replaced by BBA → `fallback: true` (kept in data, not shown), `rule` →
   `ruleRef`. Hands go to the engine as `S.H.D.C` strings (convert from
   the app's `{spades: [...]}` form); calls as the app writes them (`1NT`
   and `1N` are both accepted).
5. **Card names from the PBN header**: strip the path and `.bbsa` from
   `% CC1 - …/21GF-DEFAULT.bbsa` (as rusty-bidding-bot's
   `web/lib/pbs.js` does). A name that is not built in is fetched from
   Practice-Bidding-Scenarios `bbsa/` and passed as `{bbsa: text}`; it
   never falls back silently.
6. **Log every fallback** (position, cards, `rules_id`) for C5
   (Bridge-Classroom)'s reports.

## Acceptance criteria

- [ ] `npm ci && npm run build` needs no sibling checkout; a wrong hash
      fails the build with a clear message.
- [ ] The WASM downloads only when Rusty is selected (a network trace
      shows no download otherwise).
- [ ] The main thread is never blocked by an engine call.
- [ ] A position with no Rusty rule is bid by BBA with the Q11 cards, and
      the auction continues with Rusty (fixture test).
- [ ] If BBA is unreachable at a fallback, the call is Pass and the table
      does not hang; an engine failure surfaces as `dealError` with the
      engine's diagnostic message.
- [ ] Divergence, toggle and undo on a Rusty table send no bidding request
      to BBA other than the reference refresh.

Depends on: R1 (rusty-bidding-bot) and a release carrying it. Uses the
checksums of R4 (rusty-bidding-bot) once they exist (until then, hash the
downloaded asset once and commit the value).
