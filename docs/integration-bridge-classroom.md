# Integration with Bridge-Classroom's bidding tables

Status: **plan, for review** (issue #3). Rick answered most of the open
questions on 2026-09-28; his decisions are listed below and folded into the
recommendation, the phased plan and the draft issues. The questions still
open are at the end. The issues are drafts, not yet filed.

**Rick's decisions (2026-09-28):**

1. **A settings option chooses the bidding engine, BBA or Rusty.** It
   defaults to BBA, and the choice is remembered once changed.
2. **Initially the same bot plays at all non-human seats.** No mixing of
   BBA and Rusty seats at one table.
3. **Mouseover meanings from Rusty.** Hovering a human's call shows what
   it would mean to Rusty; hovering Rusty's own calls shows what they
   mean. Neither exists today; both should be added.
4. **By default the same card is played in both directions.**
5. **"Distributed table"** means the table service with humans on
   different computers at the same table (§3.2 with B), not peer-to-peer
   or offline tables.
6. **When no rule applies, fall back to BBA for that call**, initially,
   and do not flag it at this level.

Goal (issue #3): offer this engine (called "Rusty" below) as a configurable
alternative to BBA at Bridge-Classroom's practice tables. On a single-user
table it runs in the browser for each bot seat. On a multiplayer table each
table bids on its own. It may need to go into the table service too.

File references are to the sibling checkouts under `../` (checkouts beside this repo) as
of 2026-09-28. Line numbers will drift.

## 1. How bidding works today

### 1.1 The BBA service

BBA runs as `bba-server` (repo `BBA-tools/bba-server`, host
`bba.harmonicsystems.com`, port 5000 behind the shared Caddy). It is a Rust
Axum wrapper around `epbot-core`.

- One endpoint matters here: `POST /api/auction/generate`
  (`bba-server/src/main.rs:129`). It returns the **complete** auction for a
  whole deal, from the first call. It does not return one call at a time.
- The request carries the full deal (all four hands, "N:" PBN), dealer,
  vulnerability, scoring, an optional `auctionPrefix` to continue from, and
  either a scenario name or explicit card ids `{ns, ew}`
  (`routes/api.rs:77-87`).
- Cards are **`.bbsa` names**, such as `21GF-DEFAULT`. The server fetches them
  from the Practice-Bidding-Scenarios repo on GitHub. A scenario name is
  resolved through `pbs-release/<scenario>.pbs` to its CC1/CC2 cards. The
  defaults are `21GF-DEFAULT` for NS and `21GF-GIB` for EW
  (`config.rs:26-33`, `services/convention_service.rs:56-66`).
- The response is `{auction[], meanings[], conventionsUsed}`. Each meaning is
  `{position, bid, meaning, meaningExtended, isAlert}` (camelCase;
  `models.rs:106-118`, built at `routes/api.rs:146-157`).
- BBA emits its own CORS headers, so Bridge-Classroom's origins are
  allow-listed in `bba-server/src/main.rs:97-114`.

### 1.2 The solo practice table (LocalEngine, in the browser)

- `src/utils/bbaClient.js:27-58` `fetchAuction()` is the only client. Scoring
  is always `MP` (`:34`).
- `src/composables/engines/localEngine.js`:
  - `DEFAULT_CARD = '21GF-DEFAULT'` for sources that are not scenarios
    (`:34`). `generateAuction()` sends a scenario name, the embedded cards, or
    the default card for both sides (`:160-166`).
  - `loadDeal()` asks BBA for the whole expected auction, then
    `playToHumanTurn()` replays BBA's calls for the bot seats, one every
    300 ms, until it is the human's turn (`:170-179`, `:194-231`).
  - **Bots do not bid on their own.** They replay BBA's prediction. When the
    human bids something different, `onUserBid()` records the divergence and
    asks BBA again with the actual auction as `auctionPrefix`, and the bots
    continue from the new prediction (`:235-257`). Toggling a diverged call
    and undo also send the request again (`:260-284`, `:313-338`).
  - The same BBA auction is the **reference** the student is marked against:
    `divergedBids` and the summary text "You matched the BBA all the way
    through" (`:131-136`).
- `src/views/BiddingPracticeView.vue`:
  - It builds the LocalEngine at `:1082-1087`. Embedded mode (an iframe host)
    passes explicit card ids from the URL (`:903-909`, `:1808-1821`) and posts
    `meanings` back to the host when the auction ends (`:1827-1838`).
  - `bp.cardplayShowBbaCompare` turns the comparison off (`:1063-1075`). The
    cardplay bot choice is `bp.cardplayBot` (`:1011-1022`). **There is no
    choice of bidding engine.**
  - It passes `meanings` to `AuctionTable` (`:562`, `:575`).
- `src/components/AuctionTable.vue`:
  - Hovering a call shows `meaningExtended || meaning` in a tooltip, looked up
    by **position in BBA's predicted auction** (`:260-273`). Suit codes `!C !D
    !H !S` become symbols, and "natural"/"pass"/"double" are suppressed
    (`:96-116`). `isAlert` is not used.
  - A diverged cell stacks "You" and **"BBA"**. The label is hardcoded
    (`:37`).
- `src/composables/engines/tableEngine.js` defines the TableEngine contract.
  `getExpectedAuction(deal) → {auction, meanings}` (`:93-96`) and the
  `bbaExpectedAuction` capability (`:38-49`, `:60-71`) are part of it.

### 1.3 The served (multiplayer) table: bridge-table-service

- The server has authority. Bot seats bid **server-side**, in Rust. The
  driver is `src/bots.rs`: the bid branch is at `:341-365`, and
  `choose_call()` is at `:503-538`.
- `src/bots/bba.rs` is the BBA client and uses the same endpoint. It sends
  **one hardcoded card, `21GF-DEFAULT`, for both sides** (`:28-29`, body
  `:59-74`) and has an 8 s timeout (`:31-32`). A per-room *prefix cache*
  (`rooms.rs:211-215`) serves bot calls from the prediction while the actual
  auction is still a prefix of it (`bba.rs:76-86`). When it is not, the
  client asks again. Any failure or illegal call falls back to Pass.
- `BotMode` (`rooms.rs:79-100`) chooses only the **cardplay** bot. "Bidding
  is ALWAYS real (BBA)" (`bots.rs:353-361`). PassBot sides
  (`rooms.rs:~245`) skip BBA. In PlayOnly mode BBA bids every seat
  (`rooms.rs:137-139`).
- Bot calls reach the clients as plain calls. **No explanation or alert goes
  over the wire**, and the served `AuctionTable` gets no `meanings`
  (`BiddingPracticeView.vue:181-187`).
- The service already links a bot as a **native Rust crate**:
  `bridge-rulebot` (`Cargo.toml:35-37`, pinned git rev in `Cargo.lock`).
- Sessions are created by the Bridge-Classroom API
  (`bridge-classroom-api/src/routes/table_sessions.rs:93-122`,
  `POST /admin/sessions`). The payload is `{id, kind, boards_pbn,
  table_count, seat_policy, owner_sub}` and carries no cards.
- On the client (`src/composables/engines/serverEngine.js`), the BBA
  reference auction for the "you vs BBA" overlay is computed **in the
  browser** at review, from the full deal, with the default card (`:108-116`,
  `:165-237`). Bot seats are labelled `"BBA+RulesBot"`, hardcoded in
  `botLabelFor()` (`:398-406`). The host chooses the cardplay bot through
  `set_bot_mode` (`useTeacherConsole.js:134`, `sessions.rs:536`).

### 1.4 The rulebot precedent (cardplay, already in the browser)

- `bridge-rulebot/wasm/src/lib.rs` has two stateless `wasm_bindgen` functions
  with a JSON string on each side (`choose_opening_lead_json`,
  `choose_card_json`). It deliberately does not use serde-wasm-bindgen.
- It is vendored into Bridge-Classroom at `src/vendor/bridge-rulebot-wasm/`
  (a 162 KB wasm file), with a README that records how it was built. It is
  vendored rather than an npm dependency "so `npm ci` in CI needs no sibling
  checkout".
- `src/utils/cardplayBots.js:238-336` imports it lazily on first use, runs it
  on the main thread, and on any error falls back to a legal card. The table
  service uses the same core crate natively, so "solo and shared tables share
  one implementation".

### 1.5 Convention cards in Bridge-Classroom

- Cards are stored in `convention_cards` (`card_data` is JSON text, format
  `bridge_classroom`, schema `1.0`) and linked to users through
  `user_convention_cards` with `is_primary`
  (`bridge-classroom-api/src/db.rs:515-572`,
  `src/models/convention_card.rs`).
- The front end loads the user's **primary card**, falling back to the public
  system card (`src/composables/useConventionCard.js:44-76`). The editor
  renders against `src/utils/conventionCatalog.js`.
- **No card from the database ever reaches BBA.** BBA only sees `.bbsa`
  names.

### 1.6 What this repo offers

- `rbb_engine::Engine::new(ns: &Card, ew: &Card, modules)`
  (`crates/engine/src/engine.rs:115`), `bid(hand, dealer, vul, scoring,
  calls) -> Decision` (`:345`) and `interpret(dealer, vul, scoring, calls) ->
  Interpretation` (`:329`).
  - `Decision` has `call`, `explanation`, `alert`, `rule` (file:line), the
    ranked `candidates` with the reason each lost, and the interpreted
    auction.
  - `Step` (`:39-52`) has `explanation`, `alert`, `artificial`, and
    `knowledge`: HCP and suit-length ranges and the constraints shown.
- `Alert` is `Alert{text?}` or `Announce{text}` (`crates/bidspec/src/ast.rs:149-157`).
- The engine bids **one seat from that seat's own hand**. It never needs the
  other three hands.
- It is deterministic by design. The sample pool uses a fixed seed
  (`crates/engine/src/sample.rs`). There is no clock and no randomness.
- `bridge_card::Card::from_json` (`crates/bridge-card/src/card.rs:107`) reads
  the editor's nested `card_data` directly.
  - Aliases cover the places where the seed card and the catalog disagree
    (docs/DESIGN.md, "The convention card"). Unknown leaves are kept.
  - `.bbsa` import is lossless. All 18 PBS `.bbsa` files load.
- Cost: one cold native `rbb call` is 0.1–0.2 s of wall time. That includes
  parsing every `.bid` file from disk and building the sample pool. The WASM
  cost is not measured yet.
- Coverage gap: when no rule applies, the call is `Pass` with explanation "No
  rule applies" and `rule: null` (seen on `Pass 1H 1S 2H 2S 3H`). A practice
  table must not show that as if it were a system bid. Rick's decision:
  the table asks BBA for that one call instead (decision 6, §3.4).

## 2. Integration points

| # | Where | Today | With Rusty |
|---|---|---|---|
| P1 | `bbaClient.js` / LocalEngine `generateAuction()` | whole BBA auction | Rusty in the browser: a call per bot seat, or a whole auction in BBA's shape |
| P2 | LocalEngine reference and divergence | BBA's prediction | the reference engine's call for the human's seat, with candidates ("why") |
| P3 | `AuctionTable` tooltips | BBA meanings by predicted position | `interpret()` of the **actual** auction, every call, human calls included (decision 3) |
| P4 | Engine choice | none (bidding is always BBA) | a remembered per-user setting on the solo table, default BBA; a per-session setting on served tables; one bot for every non-human seat (decisions 1, 2) |
| P5 | Cards | `.bbsa` names only | the user's own `card_data`; built-in named cards for scenarios and defaults; the same card both ways by default (decision 4) |
| P6 | Table service `choose_call()` | BBA over HTTP plus prefix cache | native `rbb-engine` call, no cache needed; BBA for a call with no rule (decision 6) |
| P6a | No-rule calls | n/a | Rusty's `fallback` call replaced by BBA's call at that point, not flagged (decision 6) |
| P7 | Table service wire | calls only | bot calls also carry explanation and alert |
| P8 | Session creation (API → service) | no cards | cards (or card ids) in the payload |
| P9 | Bug reports | app build only | engine version, cards, auction, so `rbb call` reproduces it |

## 3. Design alternatives

### 3.1 Where the engine runs

**A. In the browser (WASM), per seat.** The solo table gets its own engine.
When a bot seat is to call, it asks `bid()` with that seat's hand.

- For: no service, and the latency is local. It does exactly what issue #3
  asks for, and it follows the rulebot pattern. With the BBA fallback
  (decision 6) a call with no rule still needs BBA over the network, so
  the table works offline only while Rusty has a rule for every bot call.
- For: prefix caching and resending requests go away. The "expected auction"
  becomes one `bid()` for the human's seat, which also says *why*.
- Against: the WASM download, and the compute cost on slow devices (the
  sample pool and descriptiveness). This needs a Web Worker and measuring.
  Rules update only when the vendored WASM is rebuilt.

**B. Native, in bridge-table-service.** `choose_call()` calls `rbb-engine`
directly, the way the service already calls `bridge-rulebot`.

- For: the server keeps authority, so every client sees the same call. There
  is no HTTP hop and no 8 s timeout. It is deterministic and replayable, and
  all tables of a class session bid independently in the same way.
- Against: the service takes a dependency on this repo and on its
  `bridge-types` pin. The rules are fixed at the service's build.

**C. A Rusty service behind BBA's interface.** A small HTTP service
implements `POST /api/auction/generate` with the same request and response.
Trying it means changing `VITE_BBA_URL` / `BBA_URL`.

- For: the fastest A/B test, with no client or table-service code change.
- Against: another service to host (droplet, Caddy, CORS). It gives up the
  local/offline benefit, and the interface cannot carry candidates or the
  user's own card unless it is extended. It must never proxy BBA (clean
  room).
- **Not recommended** as the product path. It might be worth doing later if
  some other consumer, such as the BBOAlert extension, wants it.

**D. Clients compute bot calls for a served table** (the host, or every
client). This is rejected. The table service has authority, and a client
that computes bot calls:

- stalls the table when it disconnects;
- can differ from other clients if their app versions differ;
- could cheat, since every client already receives all four hands
  (`serverEngine.js:281-307`, where redaction is client-side).

If every client computed calls independently, the server would still have to
pick one, so this is B done badly.

**Recommendation: A for the solo table, B for served tables, both from the
same crate, and not C or D.** This is the pattern `bridge-rulebot` already
uses.

### 3.2 Do independent bidders diverge?

Rick's "distributed table" (decision 5) is the served table below: the
table service, with humans on different computers at the same table and
the bot seats bid on the server. Tables that run without the service
(peer-to-peer or offline) are not wanted.

- **Solo table:** there is one bidder, so nothing can diverge.
- **Served table with B:** there is one bidder per room, on the server.
  Each table in a class session bids on its own, and the same position gives
  the same call at every table because the engine is deterministic. The only
  thing that can diverge is the *client-side analysis overlay*, when the
  browser's WASM is a different version from the service's crate. The fix:
  - the service sends each bot call's explanation (P7), so the client does
    not recompute it;
  - the welcome frame carries `bidder` and `bidder_version`;
  - the overlay says which version it used, or hides itself when the versions
    differ.
- **Determinism is a claim to test, not assume.** Rust's `HashMap` hashes
  with a random seed. Any tie-break that depends on map iteration would make
  two processes disagree. The WASM and native builds need a golden test: the
  same inputs give the same call and explanation across runs and across
  targets. The candidate sort must break ties on a stable key.

### 3.3 Passing cards

The options:

1. **The editor's card JSON, as is.** `new(ns_json, ew_json)` accepts
   `card_data`, and the engine loads it with `Card::from_json`. This is what
   makes Rusty worth having in a classroom: partner bids *your* card.
2. **Built-in named cards.** The 18 PBS `.bbsa` files are imported to card
   JSON at build time and embedded by name (`"21GF-DEFAULT"`, …). Scenario
   tables use them.
   - The scenario PBN files the client already downloads carry `% CC1 - …/21GF-DEFAULT.bbsa`
     and `% CC2` headers. The client reads CC1/CC2 there, as `rbb compare`
     does, so it needs no `.pbs` fetch.
   - A name that is not built in fails loudly. It never falls back silently.
3. **Raw `.bbsa` text**, imported in WASM. This is useful for teachers'
   custom `.bbsa` files, and it is cheap to add because `bridge-card`
   already does the import.

**Which card each side plays.** Rick's decision 4: by default the same card
is played in both directions. So:

- Solo, non-scenario table: the user's side plays the user's **primary card**
  (`useConventionCard.js:70-76`), and so do the opponents. A different card
  for the opponents is a later option, not a default.
- Scenario tables: CC1/CC2 from the PBN, like BBA. The scenario names its
  cards, so the default does not apply there.
- Served tables: the API adds cards to the session payload (P8). By default
  one card for both sides: the owner's primary card, or the teacher's
  choice (which of the two is still open, Q3).
- When the table falls back to BBA for a call (decision 6), BBA can only be
  given `.bbsa` names. With an editor card, the fallback call is bid on
  BBA's nearest card (the default `21GF-DEFAULT`, or a built-in name that
  the editor card was imported from). The draft issues record this; it is
  a known inconsistency of the fallback, not a new decision.
- The card is needed for **interpreting** too, not only for bidding. Its
  interpretation of a human's call ("your 2♣ was read as Stayman") uses the
  human's side's card.

**Coverage warnings:** a user's card can switch on conventions Rusty has no
rules for yet. WASM should expose a card report: fields honoured, ignored and
unknown, as `rbb card coverage` does for `.bbsa`. The table can then say
"Rusty doesn't play Smolen yet; partner will treat 3♥ as natural."

### 3.4 Alerts and explanations → the UI

Proposed mapping into the shape `AuctionTable` already takes, so the
component changes little:

| Rusty | UI field | Notes |
|---|---|---|
| `explanation` | `meaning` | short text, e.g. "Stayman: asks for a 4-card major" |
| `knowledge` (HCP, lengths) | `meaningExtended` | rendered by the engine, e.g. "8+ HCP, 4+ ♥"; one wording, owned by the engine |
| `alert: Alert` | `isAlert: true` (+ `alertText`) | first use of `isAlert` in the UI |
| `alert: Announce{text}` | `announce: text` | e.g. "Transfer", "15–17"; shown inline, not only on hover |
| `artificial` | `artificial` | may style the cell |
| `rule == null` | `fallback: true` | not shown: the table replaces the call with BBA's call at that point and shows BBA's meaning for it, unflagged (decision 6); the flag stays in the data for logs and bug reports |
| `rule` (file:line) | `ruleRef` | goes into bug reports, not shown to students |
| `candidates` | `why` | the teaching panel: why the reference call won and why the student's call lost |

**Mouseover (decision 3).** Every call in the auction grid gets a tooltip
from Rusty: for a human's call, what it would mean to Rusty (the
interpretation of that call with the human's side's card); for Rusty's own
calls, what they mean. Today only BBA's predicted meanings are shown, and
human calls after a divergence have none.

**The BBA fallback (decision 6).** When Rusty's `bid()` returns
`fallback: true` for a bot seat, the table asks BBA for the auction from
that point (`auctionPrefix` = the calls so far) and takes BBA's next call
for the seat, with BBA's meaning for it. It is not flagged to the player.
Rusty then reads BBA's call like any other call in the auction; if Rusty
has no reading for it, its meaning for later calls is wider than usual,
which is one more reason to log fallbacks (P9) even though the UI does not
show them.

Differences from BBA that the UI must allow for:

1. Meanings come from `interpret()` over the **actual** auction, so tooltips
   stay correct after a divergence. The BBA meanings describe BBA's line, and
   they no longer line up with the actual calls once the auction leaves it.
2. The suit notation must be settled once. Either the engine emits `!S`
   tokens, which `formatMeaningHtml` already converts, or it emits Unicode
   symbols. HTML is never passed through.
3. Hidden information: in real bridge you do not see explanations of your
   partner's calls, and alerts go to the opponents. For teaching, the solo
   table can show everything. A served table should probably follow table
   rules (Q6).

### 3.5 Choosing the engine and the reference

- Solo: add a `bp.biddingEngine = 'bba' | 'rusty'` setting next to the
  cardplay bot, **default `bba`, remembered once changed** (decision 1),
  and `?bidder=` for testing. The chosen engine bids for **every**
  non-human seat (decision 2).
- Served: add a host setting. This is a new `set_bidder` frame, or a field on
  session create. It is separate from `BotMode`, which is about cardplay. The
  seat label becomes `"Rusty+RulesBot"`. Again one bidder for all bot seats.
- The **reference** the student is marked against is either the table's
  engine (the simplest), or a separate choice that can show both BBA and
  Rusty (Q2). Every hardcoded "BBA" label becomes a parameter: the stacked
  cell label (`AuctionTable.vue:37`), the summary text
  (`localEngine.js:134-135`) and the seat label (`serverEngine.js:404`).
- Per-scenario gating: PBS already publishes `bbaWorks` per scenario
  (`pbsScenarios.js:182-212`). Rusty can publish a matching coverage manifest,
  built from `rbb compare` (agreement and card coverage per scenario). The
  picker can then show, or enforce, where Rusty is ready.

### 3.6 Versioning rules and engine

- The rules are embedded at build time. The WASM and the native crate both
  expose `version()`, which returns the crate version, the git commit, a
  **content hash of the compiled rules**, and the card schema version.
- Bridge-Classroom vendors the WASM, as it does the rulebot, and the vendor
  README records the commit. The table service pins a git rev in its
  `Cargo.lock`. Both report their version in the welcome frame and in bug
  reports.
- Upgrades are explicit. Rebuild and re-vendor, or bump the pin. A table
  never changes rules in the middle of a board.
- Loading the compiled rules (the JSON IR) from a URL at run time would let
  rules ship without an app deploy. It costs reproducibility and adds a
  compatibility contract between the IR and the engine. **Defer it** (Q8).
- Card JSON: `schema_version` is "1.0" on both sides. `Card::from_json` is
  already tolerant: aliases work, unknown leaves are kept, and a `LoadReport`
  describes what happened. The open question in DESIGN.md ("Versioning")
  becomes real once the stored cards drive bidding. A card saved by a newer
  editor must still load.
- Because the engine is deterministic, a report with {version, cards, dealer,
  vul, auction, hand} reproduces exactly with `rbb call`. Bridge-Classroom's
  "Report a Problem" can then file bidding reports that Rusty can act on
  (P9), in the same shape as the workbench tickets.

## 4. What the integration needs from the WASM crate (issue #1)

Assumed base API: `new(ns_card, ew_card)`, `bid(...)`, `interpret(...)`. The
integration also needs:

1. **Cards** accepted as editor JSON, as a built-in name (the PBS set,
   embedded), or as `.bbsa` text. Load warnings are returned, not thrown.
2. **Handle reuse.** `new()` is where the cost goes (compiled rules, the
   sample pool). The object has to stay alive across calls and boards, and be
   cached per card pair.
3. **`auction(hands, dealer, vul, scoring, prefix)`** runs `bid()` for each
   seat until the auction ends. It returns `{auction[], meanings[]}` in BBA's
   shape. That makes it a drop-in for `fetchAuction()`, and it is the
   cheapest first step for LocalEngine. Because of the BBA fallback
   (decision 6), it **stops at the first call with no rule** and says so
   (`fallbackAt: <index>`), so the caller can take BBA's call there and
   call `auction()` again with the longer prefix. The WASM never calls
   BBA itself.
4. **Display fields** as in §3.4: `meaning`, `meaningExtended`, `isAlert`,
   `alertText`, `announce`, `artificial`, `fallback`, `ruleRef`. The
   `candidates` come with human-readable reasons.
5. **`card_report(card)`**: fields honoured, ignored and unknown.
6. **`version()`**: crate version, commit, rules hash, schema version.
7. **Errors** as `{error}` JSON or a thrown string, never a panic.
   `console_error_panic_hook` is on, as in the rulebot.
8. **Wire formats that match the app**:
   - calls in PBN (`"1N"`, `"Pass"`, `"X"`), with the app's `"1NT"` accepted
     too;
   - hands as `{spades:[…],…}` or `S.H.D.C`;
   - vul as `None|NS|EW|All` (with `Both` accepted);
   - scoring as `MP|IMP`.
9. **Budgets, measured in CI:**
   - the wasm size (gzipped);
   - the time for `new()` and for one `bid()` in a headless browser;
   - a determinism test that runs the same golden cases natively and in
     WASM.
10. **A web target** (`wasm-pack --target web`) that also runs in a module
    Web Worker.

## 5. Recommendation

Revised for Rick's decisions of 2026-09-28.

1. **Solo table first, in the browser (A).**
   - Rusty is a bidding engine chosen in the table settings: **BBA by
     default, the choice remembered** (decision 1). The chosen engine bids
     every non-human seat (decision 2).
   - Step one is the smallest possible change: an `rbbClient.fetchAuction()`
     with BBA's return shape, selected by the setting. It runs Rusty seat by
     seat and, where Rusty has no rule, **takes BBA's call for that seat**
     (decision 6). That puts Rusty behind a switch with almost no change to
     LocalEngine.
   - **Mouseover meanings come with step one** (decision 3): with Rusty
     bidding there are no BBA meanings to show, so the tooltips must come
     from Rusty's `interpret()` over the actual auction, human calls
     included.
   - Step two moves to calls per seat, which removes the re-requests on
     divergence and gives the "why" panel.
2. **Run it in a Web Worker** from the start. The worker exposes a
   promise-based API. The engine is not free, bots already pace at 300 ms,
   and a main-thread stall would show.
3. **Served tables next, natively in bridge-table-service (B).** This is
   Rick's "distributed table" (decision 5): humans on different computers
   at one table, the bot seats bid on the server.
   - Add a per-session bidder choice. `choose_call()` calls `rbb-engine`
     directly, with an engine cached per session and card pair, and falls
     back to its existing BBA client for a call with no rule.
   - Bot-call events carry the explanation. The API passes cards at session
     create.
4. **No BBA-compatible Rusty service (C)**, and no bot calls computed by
   clients (D).
5. **Cards:**
   - by default one card for both sides (decision 4): the user's primary
     card on solo non-scenario tables;
   - scenario tables use CC1/CC2 as built-in names;
   - coverage warnings are visible.
6. **The fallback is not shown, but it is counted.** A BBA call in place of
   a Rusty no-rule call is not flagged to the player (decision 6); it is
   logged with the position, so the gaps reach this repo as work.
7. **Gate by coverage** later (Q1): publish a per-scenario manifest, so the
   picker can show where Rusty is ready once Rick sets the threshold.

## 6. Risks

| Risk | Mitigation |
|---|---|
| Coverage gaps: partner bids badly, and students learn wrong things | Opt-in (default BBA), BBA fallback for no-rule calls, coverage manifest, fallback log and bidding reports routed to this repo |
| The fallback hides the gaps: nobody sees how often BBA stepped in | Log every fallback (position, cards, versions); count them in the coverage manifest and in "Report a Problem" bundles |
| A fallback call mixes two systems: BBA bids on a `.bbsa` card, Rusty may be playing an editor card, and Rusty then has to read BBA's call | Accepted for now (decision 6). Use the built-in name the card came from where there is one; log the cases where Rusty has no reading of BBA's call |
| The fallback needs the network | Only at no-rule calls; if BBA is unreachable, Pass as the table service does today, and log it |
| WASM compute on slow devices (sample pool, descriptiveness over 20k hands) | Web Worker, reuse the handle, measure in CI; shrink or precompute the pool if needed |
| WASM size (the rules are ~240 KB of `.bid` source; the rulebot is 162 KB) | Embed the compiled IR, not the source; wasm-opt; lazy-load only when Rusty is chosen |
| Non-determinism (HashMap order, float ties) splits native from WASM | Golden determinism tests; stable tie-break key |
| `bridge-types` version skew in the table service (two copies of `Call`) | Keep the git revs pinned together, or keep the service's boundary in PBN strings |
| The editor's catalog and the card registry drift apart (seed/catalog path differences) | Registry aliases; a fixture test over real DB cards; later, the editor consumes `rbb card schema` |
| Confusing BBA and Rusty feedback | Every surface names its engine; one reference per table unless Q2 says both |
| Mixed versions (the WASM overlay vs the service crate) | The service sends explanations; versions in the welcome frame; the overlay is hidden on mismatch |
| Clean room | Unchanged: BBA stays a black box behind HTTP, the fallback included; nothing proxies it or models it |

## 7. Open questions for Rick

The original numbers are kept, so references elsewhere stay valid. Rick
answered some on 2026-09-28 (the decisions at the top); the rest stay open.

1. **Default and gating.** *Partly answered:* a settings option, default
   BBA, remembered once changed (decision 1). **Still open:** at what
   coverage (agreement or card coverage per scenario) may a scenario offer
   Rusty, or default to it?
2. **Reference.** *Open.* On a Rusty table, is the student marked against
   Rusty, against BBA, or can they see both?
3. **Whose card.** *Partly answered:* by default the same card in both
   directions (decision 4), so on a solo table both sides play the user's
   primary card. **Still open:** on served tables, does the teacher pick
   the card at session create, or does the owner's primary card apply?
   And should the opponents' card be selectable later?
4. **"Distributed table."** *Answered:* the table service with humans on
   different computers at the same table (decision 5).
5. **No rule.** *Answered:* fall back to BBA for that call, initially, and
   do not flag it at this level (decision 6).
6. **Hidden information.** *Open.* Should served tables follow table rules
   (you do not see explanations of your partner's calls, and alerts go to
   the opponents), or teaching rules (everyone sees everything)? This
   decides who sees the mouseover meanings (decision 3) on a served table.
7. **Release path.** *Open.* Hand-vendored WASM, like the rulebot, or a
   build product from issue #1 (CI artifact or npm package) that
   Bridge-Classroom pulls?
8. **Rules without a deploy.** *Open.* Load the rules IR from a URL at run
   time, or keep them pinned inside the WASM (the recommendation)?
9. **Card editor.** *Open.* Should the editor start reading
   `rbb card schema` and the coverage report now, ahead of the convention
   layer's move to its own repo with the editor?
10. **Scoring.** *Open.* Bridge-Classroom always sends MP. Should tables
    offer IMPs? The engine takes scoring as an input.
11. **Fallback card** (new, follows from decisions 4 and 6). When the table
    plays an editor card and falls back to BBA, which `.bbsa` card should
    BBA use: `21GF-DEFAULT`, or the built-in card nearest to the editor
    card?
12. **When to flag the fallback** (new). "Not at this level" leaves a later
    level open: should a teacher's view or a debug setting show which calls
    BBA made?

Decided without a question: one bot for all non-human seats (decision 2),
and mouseover meanings from Rusty for human and bot calls (decision 3).

## 8. Phased plan

| Phase | Scope | Issues (below) |
|---|---|---|
| 0. Engine ready | WASM API additions (including `auction()` that stops at a no-rule call), built-in cards, display contract, determinism and performance budgets, coverage manifest with fallback counts | R1–R4 (and #1) |
| 1. Solo, opt-in | Vendor the WASM in a worker; `fetchAuction`-compatible adapter with the BBA fallback; the remembered engine setting (default BBA), one engine for all bot seats; parameterized labels; mouseover meanings from Rusty for every call | C1, C2, C3 |
| 2. Solo, teaching | Calls per seat; alerts and announcements; "why" panel; the user's card for both sides and coverage warnings; bidding reports with the fallback log | C4, C5 |
| 3. Served ("distributed") tables | Native bidder in the table service with the BBA fallback; explanations on the wire; one card for both sides at session create; client labels and tooltips | R5, T1, T2, C6 |
| 4. Gating | Per-scenario offer or default from the manifest, decided by Rick with evidence (Q1) | none yet |

## Proposed issues

Drafts for review. **Not filed.** R = rusty-bidding-bot, C = Bridge-Classroom,
T = bridge-table-service. Dependencies are given in each body. Revised on
2026-09-28 for Rick's decisions (numbered at the top of this document).

### R1 · rusty-bidding-bot · WASM API for Bridge-Classroom's practice tables

Issue #1 builds the WASM. This issue gives what Bridge-Classroom needs from
it (docs/integration-bridge-classroom.md §4).

- `new(ns, ew)` takes each card as editor `card_data` JSON, a built-in name
  (R2), or `{bbsa: "<text>"}`. Load warnings are returned. Passing one card
  for both sides is the common case (decision 4).
- `bid(hand, dealer, vul, scoring, auction)` returns `{call, meaning,
  meaningExtended, isAlert, alertText, announce, artificial, fallback,
  ruleRef, candidates[]}`. `fallback: true` tells the caller to take BBA's
  call instead (decision 6).
- `interpret(dealer, vul, scoring, auction)` returns one entry per call with
  the same display fields, for human calls as well as bot calls: the
  mouseover meanings (decision 3).
- `auction(hands, dealer, vul, scoring, prefix)` returns `{auction[],
  meanings[], fallbackAt?}` in the shape of bba-server's
  `/api/auction/generate`. It stops at the first call with no rule and
  gives its index, so the caller can insert BBA's call and continue. The
  WASM never calls BBA.
- `card_report(card)` returns the fields honoured, ignored and unknown.
- `version()` returns `{crate, commit, rulesHash, cardSchema}`.
- Calls may be PBN or app tokens (`1N`/`1NT`, `Pass`, `X`, `XX`). Vul may be
  `None|NS|EW|All|Both`.

Acceptance:
- [ ] Every function takes and returns JSON strings; errors are returned, and
      nothing panics.
- [ ] The handle is reusable: a second `bid()` does not rebuild rules or the
      sample pool.
- [ ] `auction()` output, completed with the fallback calls, is accepted
      unchanged by Bridge-Classroom's `AuctionTable` / LocalEngine.
- [ ] `auction()` stops at a no-rule call and resumes from a prefix that
      includes a call it did not make (test).
- [ ] The build works as `wasm-pack --target web` and loads inside a module
      Web Worker (smoke test).
- [ ] Documented in the crate README with a JS example.

### R2 · rusty-bidding-bot · Built-in named cards (the PBS `.bbsa` set)

Scenario tables name their cards the way BBA does (`21GF-DEFAULT`,
`21GF-GIB`, … from the PBN `% CC1/CC2` headers). Embed the 18
Practice-Bidding-Scenarios `.bbsa` files, imported to card JSON at build
time, so the WASM and native builds resolve these names without network
access. The same names are what the BBA fallback sends to BBA.

Acceptance:
- [ ] `cards()` lists the built-in names. `new("21GF-DEFAULT", "21GF-GIB")`
      works.
- [ ] An unknown name is an error. It never falls back silently.
- [ ] A build script or test fails when a PBS `.bbsa` changes and the
      embedded copy has not been refreshed. At minimum, the refresh is one
      documented command.
- [ ] Names parsed from a CC1/CC2 header path (`…/bbsa/21GF-DEFAULT.bbsa`)
      resolve.

### R3 · rusty-bidding-bot · Display contract for explanations and alerts

Define and implement how a call's explanation reaches a UI:

- `meaning` comes from the rule text.
- `meaningExtended` is rendered from `knowledge`: HCP and suit-length ranges,
  one wording owned by the engine.
- `Alert` maps to `isAlert` plus `alertText`, and `Announce` to `announce`.
- `artificial`.
- `fallback: true` when no rule applied. It replaces the "No rule applies"
  text as a meaning. The table does not show it (decision 6): it takes
  BBA's call and meaning instead, and logs the flag.
- The meaning of a call the engine did not make (a human's call, or a BBA
  fallback call) comes from `interpret()` in the same shape, so the
  mouseover is uniform (decision 3).

Choose one suit notation (`!S` tokens, which Bridge-Classroom already
renders, or Unicode) and never HTML.

Acceptance:
- [ ] The contract is documented in docs/ (fields, notation, examples).
- [ ] Native `Decision`/`Step` and the WASM output share it (one serializer).
- [ ] `.test` cases or unit tests cover an alert, an announcement, an
      artificial call, a fallback, and the interpretation of a call no rule
      would make.

### R4 · rusty-bidding-bot · Determinism, performance budget, and coverage manifest

- Determinism: golden cases (hand, auction, cards → call and explanation)
  give identical results natively and in WASM, across runs. Candidate ties
  break on a stable key, never on HashMap order.
- Performance: CI measures the wasm size (gzipped), `new()`, and one `bid()`
  in a headless browser, and fails on regressions past the agreed budgets.
- Coverage manifest: `rbb compare --manifest out.json` writes, per scenario,
  the agreement with BBA, the contract agreement, the vs-BBA IMPs, the card
  coverage, and **how often a bot call would need the BBA fallback** (no
  rule in a live auction). Bridge-Classroom can later gate Rusty the way it
  uses `bbaWorks` (Q1).

Acceptance:
- [ ] The determinism test runs in `cargo test` (native) and in the WASM
      test job.
- [ ] Budgets are recorded in docs and checked in CI.
- [ ] The manifest schema is documented. A release build publishes it next to
      the WASM.

### R5 · rusty-bidding-bot · Native embedding for services

bridge-table-service needs the engine with the rules embedded, as the WASM
has them, without reading `conventions/` from disk. Provide
`rbb_engine::builtin::engine(ns, ew)` (or an `rbb-embedded` crate) that
shares the embedding with the WASM crate. Document the `bridge-types` pin
policy so that the service does not link two copies.

Acceptance:
- [ ] One embedding mechanism serves both WASM and native.
- [ ] Native and WASM report the same `version().rulesHash` for the same
      commit.
- [ ] Documented: how a downstream service pins the crate and `bridge-types`
      together.

### C1 · Bridge-Classroom · Vendor rusty-bidding-bot WASM with a worker-based client and the BBA fallback

Depends on R1 and R2. Add `src/vendor/rbb-wasm/`, vendored the same way as
`bridge-rulebot-wasm` (README with the build command and commit). Add
`src/utils/rbbClient.js`, which runs the engine in a module Web Worker and
exposes a promise API:

- `fetchAuction()`, with the same signature and return value as
  `bbaClient.fetchAuction()`. It runs Rusty's `auction()`; at each
  `fallbackAt` it asks `bbaClient.fetchAuction()` with the calls so far as
  `auctionPrefix`, takes BBA's next call and its meaning, and resumes
  Rusty (decision 6). The fallback is not marked in the returned meanings;
  it is logged (position, cards, versions).
- `bid()`, `interpret()`, `cardReport()` and `version()`.

It loads lazily on first use and caches engines per card pair.

Acceptance:
- [ ] The WASM loads only when Rusty is selected (a network trace shows no
      download otherwise).
- [ ] The main thread is never blocked by an engine call.
- [ ] A position with no Rusty rule is bid by BBA, and the auction
      continues with Rusty (fixture test).
- [ ] If BBA is unreachable at a fallback, the call is Pass and the table
      does not hang; a failure in the WASM surfaces as `dealError` with a
      clear message.
- [ ] `npm ci && npm run build` needs no sibling checkout.

### C2 · Bridge-Classroom · Bidding engine setting (BBA or Rusty), remembered

Depends on C1. Add a `bp.biddingEngine = bba | rusty` setting in the table
settings, **default `bba`, persisted like the other `bp.*` settings** so the
choice is remembered once changed (decision 1), plus `?bidder=` for testing.
The chosen engine bids **all** non-human seats (decision 2).

- LocalEngine's `generateAuction()` dispatches to `bbaClient` or `rbbClient`.
- The hardcoded "BBA" text becomes the selected engine's name: the stacked
  cell label (`AuctionTable.vue:37`) and the summary
  (`localEngine.js:131-136`).
- `getExpectedAuction()` follows the setting.

Acceptance:
- [ ] A full board can be played with Rusty for all three bot seats,
      including divergence, toggle, undo and restart.
- [ ] The setting defaults to BBA for a new user, and a changed choice
      survives a reload and a new session.
- [ ] Switching the engine takes effect on the next board.
- [ ] No table mixes engines across bot seats (the per-call BBA fallback
      aside).
- [ ] The UI never says "BBA" about a Rusty result.
- [ ] Embedded (iframe) mode keeps BBA unless the host passes `bidder`.

### C3 · Bridge-Classroom · Mouseover meanings from Rusty for every call

Depends on C1 and R3. Today the tooltips show BBA's meanings by position
in BBA's predicted auction, and a human's call after a divergence has none.
On a Rusty table, every call in the auction grid gets its meaning from
Rusty's `interpret()` over the **actual** auction (decision 3):

- a human's call: what it would mean to Rusty, read with the human's side's
  card;
- Rusty's calls: what they mean;
- a BBA fallback call: BBA's meaning, unflagged (decision 6).

Also show `announce` inline and use `isAlert`.

Acceptance:
- [ ] Every call in the auction grid, including the human's, has a tooltip
      when the engine has a meaning for it.
- [ ] After a divergence, the tooltips match the calls actually made
      (regression test with a fixture).
- [ ] Announcements are visible without hover. Alerts are marked.
- [ ] Who sees which meanings on a served table follows Rick's ruling on
      hidden information (Q6); on the solo table everything is shown.

### C4 · Bridge-Classroom · Per-seat Rusty bidding and the "why" panel

Depends on C3. With Rusty, bot seats call `bid()` for their own hand at
their turn, instead of replaying a predicted auction; there is no
prediction to request again on divergence or undo. A `fallback` result is
replaced by BBA's call, as in C1. The reference call for the human's seat
is one `bid()`, measured against the reference Rick chooses (Q2).

When the student's call differs from the reference, show the reference
call, its meaning, and why the student's call lost. The losing reasons come
from `candidates[].outcome`, such as "hand fails `shows
strength=invite`", rewritten for students where the engine gives a readable
reason.

Acceptance:
- [ ] Bot calls come from `bid()` per seat; divergence, toggle and undo send
      no new auction request.
- [ ] Clicking or tapping a diverged cell opens the panel. Its content comes
      from the engine, not from hardcoded text.
- [ ] It works in review and during the auction, and is hidden when the
      comparison is off (`bp.cardplayShowBbaCompare`, renamed if needed).

### C5 · Bridge-Classroom · Play my convention card, both ways, with coverage warnings

Depends on C1 and R1. On non-scenario solo tables with Rusty, the user's
**primary card** (`useConventionCard`) is played by **both sides** by
default (decision 4). Scenario tables use CC1/CC2 read from the PBN header.
Show `cardReport()`: conventions on the card that Rusty does not honour yet.

- The BBA fallback sends BBA a `.bbsa` name, since BBA cannot read an
  editor card; which one is Q11 (until then `21GF-DEFAULT`).
- Include `{rbb version, cards (ids or hash), dealer, vul, auction, seat,
  hand, fallback positions}` in "Report a Problem" bundles.
- Offer a "bidding problem" target that files to rusty-bidding-bot.

Acceptance:
- [ ] Changing the primary card changes both sides' bidding on the next
      board.
- [ ] The coverage warning lists the unsupported conventions by their catalog
      names.
- [ ] A filed bidding report reproduces with `rbb call` from the bundle
      alone.

### C6 · Bridge-Classroom · Served ("distributed") table: bidder choice, labels, meanings

Depends on T1 (and T2 for cards). The served table is Rick's "distributed
table" (decision 5): humans on different computers, bot seats bid by the
table service.

- Add a host control for the session bidder (BBA or Rusty), one bidder for
  all bot seats (decision 2).
- `botLabelFor()` (`serverEngine.js:401-406`) shows the actual bidder.
- `AuctionTable` on the served table gets `meanings` from the explanations on
  bot-call events, and the mouseover meaning of human calls from the
  client's `interpret()` (decision 3), within Rick's ruling on hidden
  information (Q6).
- The "you vs reference" overlay uses the table's bidder. It is hidden when
  the client's WASM version differs from the service's `bidder_version`.
- Pass the chosen card at session create (bridge-classroom-api
  `table_sessions.rs`), one card for both sides by default (decision 4).

Acceptance:
- [ ] A host can switch the bidder between boards. Seat labels follow.
- [ ] Bot calls on a Rusty table show explanations for every viewer allowed to
      see them.
- [ ] No "BBA" label appears on a Rusty table (a fallback call is not
      labelled either, decision 6).

### T1 · bridge-table-service · Native Rusty bidder with the BBA fallback

Depends on R5. Add a per-session and per-room `BidderMode { Bba, Rusty }`
(default `Bba`), set by a `{"t":"set_bidder","bidder":"rusty"}` host frame and
by an optional `bidder` on `POST /admin/sessions`. The bidder plays every bot
seat (decision 2).

- For Rusty, `choose_call()` calls `rbb-engine` directly with the seat's hand
  and the calls so far. It needs no prefix cache and no HTTP.
- **When Rusty has no rule** (`fallback`), `choose_call()` asks the existing
  BBA client (`src/bots/bba.rs`) with the calls so far and uses BBA's call
  for that seat (decision 6). The event is not flagged to clients; it is
  recorded with `record_event()` and counted in the metrics.
- Engines are cached per card pair.
- Illegal or failed calls, including a failed fallback, fall back to Pass,
  as today, and are logged with `record_event()`.
- Bot-call events add `explanation`, `alert` and `announce` (BBA's meaning
  for a fallback call).
- The welcome frame carries `bidder` and `bidder_version`.
- PlayOnly and PassBot behave as they do with BBA.

Acceptance:
- [ ] A Rusty table plays a board end to end; BBA traffic appears only for
      no-rule positions (checked in the metrics and logs).
- [ ] Same board, same auction, same seat: the same call at every table in a
      session (test), fallback calls included while BBA answers.
- [ ] Undo and changes during a bot's turn stay safe (the existing
      seq-recheck pattern).
- [ ] The CI-parity build (`./dev-build.sh --ci test`) passes with the pinned
      git rev. The committed `Cargo.lock` has no local paths.

### T2 · bridge-table-service · Convention cards per session

Accept `cards: {ns, ew}` on `POST /admin/sessions`, or a single `card` that
both sides play (the default, decision 4). Each is either a built-in name or
editor `card_data` JSON. Use them for Rusty, and for BBA (and the BBA
fallback) when they are names BBA knows, which replaces the hardcoded
`21GF-DEFAULT` (`src/bots/bba.rs:28-29`). The companion change in
bridge-classroom-api (`table_sessions.rs` `service_create_payload`) sends
the owner's primary card or the teacher's choice (Q3 decides which).

Acceptance:
- [ ] A session created with one card bids with it on both sides; one
      created with two cards bids each side with its own; one created
      without cards keeps today's default.
- [ ] Invalid card JSON is rejected at session create with a clear error, not
      at the first bot call.
- [ ] Cards are visible in the dashboard or session info for debugging.

### Deferred (not proposed for now)

- **Rusty behind BBA's HTTP interface (alternative C).** File it only if a
  consumer other than Bridge-Classroom needs it.
- **Rules IR loaded at run time** (Q8).
- **Showing BBA and Rusty together** as two references (Q2).
- **Mixing engines across bot seats** (decision 2 says "initially" one bot
  for all).
- **Flagging fallback calls in the UI** (Q12).
