# Integration with Bridge-Classroom's bidding tables

Status: **plan, for review** (issue #3). Rick answered most of the open
questions on 2026-09-28; his decisions are listed below and folded into the
recommendation, the phased plan and the draft issues. Three more (Q2, Q7,
Q11) were settled the same day as provisional defaults (§7). The questions
still open are at the end. The issues were revised on 2026-09-28 for the
engine work that has landed since the first draft (§1.7) and now describe
only the remaining work; their bodies, ready to file, are in
[drafts/integration-issues/](drafts/integration-issues/INDEX.md).

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

Provisional defaults, approved 2026-09-28 (§7):

- **Q2:** the student is marked against BBA for now, whichever engine bids.
- **Q7:** Bridge-Classroom pulls the WASM package from rusty-bidding-bot's
  GitHub release assets; it is not hand-vendored.
- **Q11:** BBA's fallback calls use the scenario's own `.bbsa` card, or
  `21GF-DEFAULT` when the table has none.

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

- `rbb_engine::Engine::new(ns: &Card, ew: &Card, rules: &RuleSet)`
  (`crates/engine/src/engine.rs:144`), `bid(...) -> Decision` (`:410`),
  `interpret(...)` (`:394`) and `bid_deal(...)` (`:691`).
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
- Cards are read from the editor's nested `card_data`, or from its export
  file (`bridge-classroom/card_data@v1`, whose `name` becomes the card's;
  keys such as `_bbo_raw` are reported and ignored), everywhere a card JSON
  is read: CLI, WASM, workbench (commit 091c8e4). Bridge-Classroom's
  ACBL-card fields are card fields under its own paths (d7d466a), and its
  seed 2/1 Intermediate card loads cleanly (f5b4932).
  - Aliases cover the places where the seed card and the catalog disagree
    (docs/DESIGN.md, "The convention card"). Unknown leaves are kept.
  - `.bbsa` import is lossless. All 18 PBS `.bbsa` files load.
- Cost: `bidDeal` takes about 100–150 ms for a whole auction in the
  browser, and one `bid` a few ms to tens of ms (docs/WASM.md, "Embed").
  The package is about 1.4 MB, 430 KB gzipped.
- Coverage gap: when no rule applies, the call is `Pass` with explanation "No
  rule applies", `rule: null` and an `info` diagnostic (seen on `Pass 1H 1S
  2H 2S 3H`). A practice table must not show that as if it were a system
  bid. Rick's decision: the table asks BBA for that one call instead
  (decision 6, §3.4).

### 1.7 What has landed since the first draft

The plan was first drafted in bb64ed0 and revised for Rick's decisions in
0db54f3. Since then (engine version 0.2.0):

- **The WASM API** (docs/WASM.md, API version 1): `info`,
  `createEngine`/`freeEngine` (a handle, reused for the same cards),
  `bid`, `interpret` (every call of an actual auction, human calls
  included, with `explanation`, `alert` as `{kind: alert|announce, text}`,
  `rule`, `artificial`, `knowledge` with a one-line `summary`, and the
  `position`), `bidDeal` (a whole auction from an optional `prefix`),
  `validate`, `conventions`, `reference`, `coverage` (the card report),
  `exportCard` and `ddTable`. JSON string in and out; nothing throws;
  problems come back as structured `diagnostics` `{severity, message,
  line?, col?, hint?}`. Calls accept `1N`/`1NT`, `P`/`Pass`; vul accepts
  `Both`. Commits c350a6c, 7499107, 1e57c2d.
- **Rule sets supplied at run time** (062b0e1): a request may carry rules,
  card vocabulary and manifest as text; `rules_id` (a hash of the texts)
  identifies the rule set of every engine and of the embedded rules.
- **Built-in cards** (`rbb-assets`, c350a6c): the rules and the 18 PBS
  `.bbsa` cards are compiled into the release `rbb` and the WASM package.
  A card spec is a stock name (`"21GF-DEFAULT"`), `{bbsa: text}` or
  `{json: card_data | export}`; an unknown stock name is an error whose
  hint lists the names. The showcase also offers Bridge-Classroom's 2/1
  Intermediate card, as `BC-21-Intermediate` from `web/cards/` (it is
  passed as `{json}`, not a stock name).
- **Release builds** (0f6c2b3, fd781b0, docs/RELEASING.md): a tag `v*`
  builds `rbb` for Linux, Windows and macOS (signed and notarized) and
  attaches `rbb-wasm.tar.gz` (the package plus `demo.html`); tags with a
  suffix publish as pre-releases. `v0.1.0-rc1` is out. CI builds the
  package on every push (artifact `rbb-wasm-pkg`).
- **The showcase** at bridge-craftwork.com/rusty-bidding-bot/ (docs/WEB.md):
  the WASM bidding PBS scenarios, random deals and dealer3 scripts seat by
  seat, every call explained, with the cards read from the PBN's CC1/CC2
  headers (`web/lib/pbs.js`). `web/lib/tool.js` is a working client of the
  API that Bridge-Classroom can crib from.
- **Teaching skills** (docs/SKILLS.md, d91f4f8): modules and card fields
  name Bridge-Classroom's skill paths; the taxonomy proposals are filed as
  bridge-craftwork/Bridge-Classroom#422.

**In progress:** R1, an `auction` entry point that bids the bot seats until
the auction ends, a human is to call, or a bot reaches a position with no
rule (it stops there so the client can take BBA's call and resume), the
meaning of any call, and a native/WASM determinism test.

**Not done yet:** a no-rule stop in `bidDeal` (R1), size and speed budgets
in CI and a coverage manifest (R4), the build's git commit in `info()` and
a documented native embedding for a service (R5).

## 2. Integration points

| # | Where | Today | With Rusty |
|---|---|---|---|
| P1 | `bbaClient.js` / LocalEngine `generateAuction()` | whole BBA auction | Rusty in the browser: R1's `auction` bids the bot seats up to the human's turn, with BBA's call at a no-rule stop |
| P2 | LocalEngine reference and divergence | BBA's prediction | still BBA's prediction for now (Q2); Rusty's reading of the student's call and its candidates ("why") alongside |
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
- Against: the WASM download (430 KB gzipped), and the compute cost on
  slow devices (the sample pool and descriptiveness; about 100–150 ms a
  whole auction on a laptop). This needs a Web Worker and budgets in CI
  (R4). Rules update only when Bridge-Classroom moves to a newer release
  (Q7).

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
  (The reference the student is marked against is BBA's auction for now,
  Q2, so it is a second source but not a second bidder.)
- **Served table with B:** there is one bidder per room, on the server.
  Each table in a class session bids on its own, and the same position gives
  the same call at every table because the engine is deterministic. The only
  thing that can diverge is the *client-side analysis overlay*, when the
  browser's WASM is a different version from the service's crate. The fix:
  - the service sends each bot call's explanation (P7), so the client does
    not recompute it;
  - the welcome frame carries `bidder` and `bidder_version` (the crate
    version and `rules_id`);
  - the client's own Rusty readings (the mouseover meaning of a human's
    call) say which version they used, or are hidden when the versions
    differ. The "you vs reference" overlay stays BBA's for now (Q2).
- **Determinism is a claim to test, not assume.** Rust's `HashMap` hashes
  with a random seed. Any tie-break that depends on map iteration would make
  two processes disagree. The WASM and native builds need a golden test: the
  same inputs give the same call and explanation across runs and across
  targets. The candidate sort must break ties on a stable key. This test
  is being written as part of R1.

### 3.3 Passing cards

The options:

1. **The editor's card JSON, as is.** `new(ns_json, ew_json)` accepts
   `card_data`, and the engine loads it with `Card::from_json`. This is what
   makes Rusty worth having in a classroom: partner bids *your* card.
2. **Built-in named cards.** The 18 PBS `.bbsa` files are embedded by name
   (`"21GF-DEFAULT"`, …) and imported when an engine is made. Scenario
   tables use them. *Done* (§1.7).
   - The scenario PBN files the client already downloads carry `% CC1 - …/21GF-DEFAULT.bbsa`
     and `% CC2` headers. The client reads CC1/CC2 there, as `rbb compare`
     does, so it needs no `.pbs` fetch.
   - A name that is not built in fails loudly. It never falls back silently.
     The client strips the path and `.bbsa` from the header, as
     `web/lib/pbs.js` does; a PBS card that is not built in can be fetched
     and passed as `{bbsa: text}`.
3. **Raw `.bbsa` text**, imported in WASM (`{bbsa: text}`). *Done.* Option
   1 is done too (`{json: card_data}` or the editor's export).

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
  given `.bbsa` names. **Q11 (provisional):** a scenario table sends the
  scenario's own `.bbsa` cards (CC1/CC2); any other table, including one
  playing an editor card, sends `21GF-DEFAULT`. With an editor card the
  fallback call is therefore bid on a different card from Rusty's: a known
  inconsistency of the fallback, accepted for now.
- The card is needed for **interpreting** too, not only for bidding. Its
  interpretation of a human's call ("your 2♣ was read as Stayman") uses the
  human's side's card.

**Coverage warnings:** a user's card can switch on conventions Rusty has no
rules for yet. WASM's `coverage` is the card report (*done*): the settings
read, ignored, unmapped and other, as `rbb card coverage` does for `.bbsa`.
The table can then say "Rusty doesn't play Smolen yet; partner will treat
3♥ as natural." Card fields name their Bridge-Classroom skill
(docs/SKILLS.md), which gives the warning a name the student has seen in
lessons.

### 3.4 Alerts and explanations → the UI

The engine's side of this exists: every step of `interpret`, `bid` and
`bidDeal` carries the fields in the left column (docs/WASM.md). The mapping
into the shape `AuctionTable` already takes is the client's adapter (C1),
so the component changes little:

| Rusty (WASM step) | UI field | Notes |
|---|---|---|
| `explanation` (`meaning` in `bidDeal`) | `meaning` | short text, e.g. "Stayman: asks for a 4-card major" |
| `knowledge.summary` | `meaningExtended` | rendered by the engine, e.g. "15-17 HCP, 2-5 S, ..., balanced"; one wording, owned by the engine |
| `alert: {kind: "alert", text?}` | `isAlert: true` (+ `alertText`) | first use of `isAlert` in the UI |
| `alert: {kind: "announce", text}` | `announce: text` | e.g. "Transfer", "15 to 17"; shown inline, not only on hover |
| `artificial` | `artificial` | may style the cell |
| `rule == null` on the engine's own call | `fallback: true` | R1 stops there instead of passing. Not shown: the table replaces the call with BBA's call at that point and shows BBA's meaning for it, unflagged (decision 6); the flag stays in the data for logs and bug reports |
| `rule` (`{module, file, line}`) | `ruleRef` | goes into bug reports, not shown to students |
| `candidates` (`outcome` per rule) | `why` | the teaching panel: why Rusty rejected the student's call |

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
2. Suit notation: the engine's text is plain, never HTML, with calls in
   the engine's spelling (`2S`, `1NT`) and suits as letters (`4+ H`). The
   client turns them into symbols when it renders (C3), as
   `formatMeaningHtml` does for BBA's `!S` tokens.
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
- The **reference** the student is marked against stays **BBA** for now,
  whichever engine bids (Q2, provisional). So a Rusty table still asks BBA
  for its predicted auction, as today, for marking only. Every hardcoded
  "BBA" label becomes a parameter naming its source: the stacked cell label
  (`AuctionTable.vue:37`) and the summary text (`localEngine.js:134-135`)
  name the reference (BBA), and the seat label (`serverEngine.js:404`)
  names the bidder.
- Per-scenario gating: PBS already publishes `bbaWorks` per scenario
  (`pbsScenarios.js:182-212`). Rusty can publish a matching coverage manifest,
  built from `rbb compare` (agreement and card coverage per scenario). The
  picker can then show, or enforce, where Rusty is ready.

### 3.6 Versioning rules and engine

- The rules are embedded at build time (`rbb-assets`). WASM `info()`
  returns `api`, the crate `version`, `rules_id` (a content hash of the
  embedded rules, manifest and card vocabulary), the rule `language`
  version and the stock card names. The source commit is not in it yet (R5).
- **Bridge-Classroom pulls the WASM package from rusty-bidding-bot's
  GitHub release assets** (`rbb-wasm.tar.gz`), pinned to a tag and a
  checksum, instead of hand-vendoring it (Q7, provisional). The table
  service pins a rev in its `Cargo.lock`. Both report their version in
  the welcome frame and in bug reports.
- Upgrades are explicit: bump the release tag, or bump the pin. A table
  never changes rules in the middle of a board.
- Rules without a deploy: the WASM now accepts a rule set as text at run
  time (`rules`, `fields`, `bbsa_map`, `manifest`; 062b0e1), identified by
  its `rules_id`, so the mechanism exists. Using it from Bridge-Classroom
  costs reproducibility (a report must then carry the rule set or its id
  and a way to fetch it). **Still deferred** (Q8): the tables play the
  embedded rules.
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

The first draft assumed a base API of `new`, `bid` and `interpret` and
listed ten additions. Their state on 2026-09-28 (docs/WASM.md, §1.7):

1. **Cards** as editor JSON, as a built-in name (the PBS set), or as
   `.bbsa` text; load warnings returned, not thrown. *Done*: card specs
   `"21GF-DEFAULT"`, `{bbsa}`, `{json}` (bare `card_data` or the editor's
   `@v1` export); problems are `diagnostics`.
2. **Handle reuse.** *Done*: `createEngine` returns a handle and reuses it
   for the same cards; the engine caches per position.
3. **`auction(...)`** bids the bot seats and **stops at the first call
   with no rule** so the caller can take BBA's call and resume. *In
   progress* (R1): it also stops when a human is to call. `bidDeal` (a
   whole auction from a `prefix`) exists but passes at a no-rule call.
4. **Display fields** (§3.4). *Done* on the engine's side: `explanation`,
   `alert` (`alert`/`announce`), `artificial`, `rule`, `knowledge.summary`,
   `candidates` with their `outcome`. The mapping into `AuctionTable`'s
   field names is the client's (C1). The meaning of *any* call, not only
   one in the auction, is part of R1.
5. **Card report.** *Done*: `coverage`.
6. **Version.** *Mostly done*: `info()` gives `api`, `version`, `rules_id`,
   `language`. The source commit is missing (R5).
7. **Errors** never thrown. *Done*: `{ok, diagnostics}` on every response;
   `console_error_panic_hook` is on.
8. **Wire formats.** *Done*, except hands: calls `1N`/`1NT`, `P`/`Pass`,
   `X`, `XX`; vul `None|NS|EW|All|Both`; scoring `MP|IMP`. Hands are
   `S.H.D.C` strings only; the client converts the app's `{spades: [...]}`
   form (C1).
9. **Budgets measured in CI** (size, `createEngine`, one `bid` in a
   headless browser). *Not done* (R4). The size is known (1.4 MB, 430 KB
   gzipped) and timings are documented, but nothing fails on a
   regression. The native/WASM **determinism test** is part of R1.
10. **A web target** that runs in a module Web Worker. *Done* for the
    target (`wasm-pack --target web`; `initSync` works in a worker). A
    worker smoke test is left to the client (C1).

## 5. Recommendation

Revised for Rick's decisions of 2026-09-28 and the provisional defaults
Q2, Q7 and Q11.

1. **Solo table first, in the browser (A).**
   - Rusty is a bidding engine chosen in the table settings: **BBA by
     default, the choice remembered** (decision 1). The chosen engine bids
     every non-human seat (decision 2).
   - The WASM package comes from rusty-bidding-bot's **GitHub release
     assets**, pinned by tag and checksum (Q7).
   - Rusty bids the bot seats with R1's `auction` entry point: it runs until
     a human is to call, and where Rusty has no rule the client **takes
     BBA's call for that seat** and resumes (decision 6), asking BBA with
     the scenario's own cards, or `21GF-DEFAULT` (Q11). Because R1 stops at
     the human's turn, the earlier plan's two steps (a `fetchAuction`
     look-alike first, calls per seat later) collapse into one: nothing is
     predicted, so there is nothing to request again on divergence.
   - The student is **still marked against BBA** (Q2): the table keeps
     asking BBA for its predicted auction, for the reference only.
   - **Mouseover meanings come with Rusty** (decision 3): the tooltips come
     from Rusty's `interpret()` over the actual auction, human calls
     included.
2. **Run it in a Web Worker** from the start. The worker exposes a
   promise-based API. The engine is not free, bots already pace at 300 ms,
   and a main-thread stall would show.
3. **Served tables next, natively in bridge-table-service (B).** This is
   Rick's "distributed table" (decision 5): humans on different computers
   at one table, the bot seats bid on the server.
   - Add a per-session bidder choice. `choose_call()` calls `rbb-engine`
     directly, with an engine cached per session and card pair, and falls
     back to its existing BBA client for a call with no rule, with the
     session's named cards or `21GF-DEFAULT` (Q11).
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
7. **Gate by coverage** later (Q1): publish a per-scenario manifest with
   each release, so the picker can show where Rusty is ready once Rick sets
   the threshold.

## 6. Risks

| Risk | Mitigation |
|---|---|
| Coverage gaps: partner bids badly, and students learn wrong things | Opt-in (default BBA), BBA fallback for no-rule calls, marked against BBA for now (Q2), coverage manifest, fallback log and bidding reports routed to this repo |
| The fallback hides the gaps: nobody sees how often BBA stepped in | Log every fallback (position, cards, versions); count them in the coverage manifest and in "Report a Problem" bundles |
| A fallback call mixes two systems: BBA bids on a `.bbsa` card, Rusty may be playing an editor card, and Rusty then has to read BBA's call | Accepted for now (decision 6, Q11: the scenario's own cards, else `21GF-DEFAULT`); log the cases where Rusty has no reading of BBA's call |
| The fallback needs the network | Only at no-rule calls; if BBA is unreachable, Pass as the table service does today, and log it. (Marking against BBA, Q2, needs the network anyway.) |
| WASM compute on slow devices (sample pool, descriptiveness over 20k hands) | Web Worker, reuse the handle, budgets in CI (R4); shrink or precompute the pool if needed |
| WASM size (1.4 MB, 430 KB gzipped; the rulebot is 162 KB) | Lazy-load only when Rusty is chosen; a size budget in CI (R4) |
| A release asset changes or disappears under a pinned tag | Pin tag and SHA-256; the fetch fails loudly on a mismatch (C1, R4) |
| Non-determinism (HashMap order, float ties) splits native from WASM | Golden determinism tests (R1); stable tie-break key |
| `bridge-types` version skew in the table service (two copies of `Call`) | Keep the revs pinned together, or keep the service's boundary in PBN strings (R5) |
| The editor's catalog and the card registry drift apart (seed/catalog path differences) | Bridge-Classroom's fields are card fields under its paths (d7d466a); aliases; a fixture test over real DB cards; later, the editor consumes `rbb card schema` |
| Confusing BBA and Rusty feedback | Every surface names its source: the bidder on seats, the reference (BBA, Q2) on marking |
| Mixed versions (the client's WASM vs the service crate) | The service sends explanations; versions (`rules_id`) in the welcome frame; client-side Rusty readings hidden on mismatch |
| Clean room | Unchanged: BBA stays a black box behind HTTP, the fallback included; nothing proxies it or models it |

## 7. Open questions for Rick

The original numbers are kept, so references elsewhere stay valid. Rick
answered some on 2026-09-28 (the decisions at the top) and approved
provisional defaults for Q2, Q7 and Q11 the same day; the rest stay open.

1. **Default and gating.** *Partly answered:* a settings option, default
   BBA, remembered once changed (decision 1). **Still open:** at what
   coverage (agreement or card coverage per scenario) may a scenario offer
   Rusty, or default to it?
2. **Reference.** *Answered (provisional, 2026-09-28):* mark the student
   against **BBA** for now, whichever engine bids. Rusty's reading of the
   student's call may be shown beside it, labelled as Rusty's (C4).
   Showing BBA and Rusty as two references stays deferred.
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
7. **Release path.** *Answered (provisional, 2026-09-28):* Bridge-Classroom
   pulls the WASM package from rusty-bidding-bot's GitHub release assets
   (`rbb-wasm.tar.gz`), pinned by tag and checksum; it is not hand-vendored.
8. **Rules without a deploy.** *Open.* The WASM can now take a rule set as
   text at run time (062b0e1). Should Bridge-Classroom ever use that, or
   keep the rules pinned inside the release it pulls (the recommendation)?
9. **Card editor.** *Open.* Should the editor start reading
   `rbb card schema` and the coverage report (WASM `coverage`) now, ahead
   of the convention layer's move to its own repo with the editor? The
   skill taxonomy work (Bridge-Classroom#422) is a first step of this kind.
10. **Scoring.** *Open.* Bridge-Classroom always sends MP. Should tables
    offer IMPs? The engine takes scoring as an input.
11. **Fallback card.** *Answered (provisional, 2026-09-28):* BBA's fallback
    calls use the scenario's own `.bbsa` cards (CC1/CC2), or `21GF-DEFAULT`
    when the table has none (an editor card, a non-scenario table).
12. **When to flag the fallback** (new). "Not at this level" leaves a later
    level open: should a teacher's view or a debug setting show which calls
    BBA made?

Decided without a question: one bot for all non-human seats (decision 2),
and mouseover meanings from Rusty for human and bot calls (decision 3).

## 8. Phased plan

| Phase | Scope | Issues (below) |
|---|---|---|
| 0. Engine ready | *Done:* WASM API, built-in cards, display fields, card report, release builds with the WASM package. *Remaining:* the `auction` entry point that stops at a human's turn or a no-rule call, meanings for any call, determinism (R1); budgets, checksums and the coverage manifest in the release (R4); native embedding for the service (R5) | R1, R4, R5 |
| 1. Solo, opt-in | The WASM from the release in a worker; Rusty bids the bot seats with the BBA fallback (Q11 cards); the remembered engine setting (default BBA), one engine for all bot seats; labels naming bidder and reference (BBA, Q2); mouseover meanings from Rusty for every call | C1, C2, C3 |
| 2. Solo, teaching | "Why" panel beside the BBA reference; the user's card for both sides and coverage warnings; bidding reports with the fallback log | C4, C5 |
| 3. Served ("distributed") tables | Native bidder in the table service with the BBA fallback; explanations on the wire; one card for both sides at session create; client labels and tooltips | T1, T2, C6 |
| 4. Gating | Per-scenario offer or default from the manifest, decided by Rick with evidence (Q1) | none yet |

## Proposed issues

Revised 2026-09-28 for the work that has landed (§1.7) and the
provisional defaults Q2, Q7, Q11. **Not filed.** R = rusty-bidding-bot,
C = Bridge-Classroom, T = bridge-table-service. The full bodies, ready to
file, are one file per issue in
[drafts/integration-issues/](drafts/integration-issues/INDEX.md), which
also gives the filing order. Below: what each issue now covers, and what
was done, merged or dropped since the first draft.

### Done, merged or dropped

- **R2 · Built-in named cards: dropped, done.** The 18 PBS `.bbsa` cards
  are compiled in (`rbb-assets`, c350a6c), listed by `info().stock_cards`;
  `createEngine({cards: {ns: "21GF-DEFAULT", ew: "21GF-GIB"}})` works; an
  unknown name is an error with the list as its hint. Reading the name
  from a CC1/CC2 header path is done client-side, as the showcase does
  (`web/lib/pbs.js`); C1 carries that. The one leftover, a check that the
  embedded copies match Practice-Bidding-Scenarios' `bbsa/`, moves to R4.
- **R3 · Display contract: dropped, merged.** The fields exist and are
  documented (docs/WASM.md: step `explanation`, `alert` with `kind`
  alert/announce, `artificial`, `rule`, `knowledge.summary`, `candidates`
  with `outcome`). What was left is split: the no-rule signal and the
  meaning of any call go to R1; the mapping into `AuctionTable`'s field
  names goes to C1 (it is Bridge-Classroom's shape); the suit notation is
  settled in §3.4 (plain text, letters, the client renders symbols; C3).
- **R1** keeps its ID but not its scope: `createEngine`, `bid`,
  `interpret`, card specs, handle reuse, card report (`coverage`),
  diagnostics and the web target are done. What remains is what is being
  built now.
- **R4** loses the determinism test (to R1) and gains the release items
  Q7 needs (checksums) and R2's leftover.
- **C1** absorbs C4's first half: with R1 stopping at the human's turn,
  per-seat bidding is C1's design from the start, not a later step. It
  also pulls the release instead of vendoring (Q7).
- **C4** keeps only the "why" panel, beside BBA's reference (Q2).

### R1 · rusty-bidding-bot · Bid a practice table's bot seats: `auction`, meanings for any call, determinism

In progress. The `auction` entry point (native and WASM) bids from a
prefix until the auction ends, a human is to call, or a bot seat reaches a
position with no rule, and says which; the meaning of any call at any
point; a golden native/WASM determinism test. Done parts listed in the
body. File: `R1-rusty-bidding-bot.md`.

### R4 · rusty-bidding-bot · Release package for Bridge-Classroom: budgets, checksums, coverage manifest

Size and speed budgets checked in CI (headless browser); SHA-256 checksums
for the release assets; `rbb compare --manifest` per scenario with the
no-rule (fallback) rate, published with the release; a check that the
stock cards match PBS. File: `R4-rusty-bidding-bot.md`.

### R5 · rusty-bidding-bot · Native embedding for bridge-table-service

`rbb-assets` already embeds the rules for native and WASM with one
`RULES_ID`. Remaining: one native constructor from card specs (the WASM's
`load_card` moved where a service can call it), the native `auction` /
meaning output in the WASM's JSON shape, the source commit in `info()` and
natively, and the `bridge-types` pin policy. File:
`R5-rusty-bidding-bot.md`.

### C1 · Bridge-Classroom · Rusty bidder in a Web Worker, from the release, with the BBA fallback

Fetch `rbb-wasm.tar.gz` from a pinned release (Q7); a worker client; bot
seats bid with R1's `auction`, BBA's call at each no-rule stop (cards per
Q11); the adapter from Rusty's steps to `AuctionTable`'s fields. File:
`C1-Bridge-Classroom.md`.

### C2 · Bridge-Classroom · Bidding engine setting (BBA or Rusty), remembered; labels name their source

Unchanged in scope, except that the reference stays BBA (Q2), so the
labels name the bidder and the reference separately. File:
`C2-Bridge-Classroom.md`.

### C3 · Bridge-Classroom · Mouseover meanings from Rusty for every call

Unchanged in scope; the engine side exists (`interpret`) plus R1's meaning
of any call; suit letters rendered as symbols. File:
`C3-Bridge-Classroom.md`.

### C4 · Bridge-Classroom · The "why" panel beside BBA's reference

Per-seat bidding moved to C1. The panel shows BBA's reference call (Q2)
and, labelled as Rusty's, its reading of the student's call and why its
own rules rejected it. File: `C4-Bridge-Classroom.md`.

### C5 · Bridge-Classroom · Play my convention card, both ways, with coverage warnings

The card goes in as stored (`{json: card_data}`); `coverage` gives the
warnings; the BBA fallback uses `21GF-DEFAULT` for an editor card (Q11);
bidding reports. File: `C5-Bridge-Classroom.md`.

### C6 · Bridge-Classroom · Served ("distributed") table: bidder choice, labels, meanings

The "you vs reference" overlay stays BBA's (Q2); the version check now
guards the client's Rusty readings. File: `C6-Bridge-Classroom.md`.

### T1 · bridge-table-service · Native Rusty bidder with the BBA fallback

Unchanged in scope; builds on R5; fallback cards per Q11. File:
`T1-bridge-table-service.md`.

### T2 · bridge-table-service · Convention cards per session

Card specs in the WASM's form; BBA and its fallback get the session's
named cards or `21GF-DEFAULT` (Q11). File: `T2-bridge-table-service.md`.

### Deferred (not proposed for now)

- **Rusty behind BBA's HTTP interface (alternative C).** File it only if a
  consumer other than Bridge-Classroom needs it.
- **Rule sets loaded at run time by Bridge-Classroom** (Q8); the engine
  side exists.
- **Showing BBA and Rusty together** as two references (Q2).
- **Mixing engines across bot seats** (decision 2 says "initially" one bot
  for all).
- **Flagging fallback calls in the UI** (Q12).
