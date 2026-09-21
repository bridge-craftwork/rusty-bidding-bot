# rusty-bidding-bot — Design

Status: **draft, open for discussion.** Decisions that are settled are marked
**Decided**; everything else is a proposal.

## Goals

- An open-source (MIT OR Apache-2.0) bridge bidding engine in Rust.
- Cross-platform: Linux, macOS, Windows, and **WASM** in the browser.
- Bidding systems and conventions are **data, not code**: a collaborator adds a
  convention by writing a small rule file, not by editing the engine.
- Conventions are switched on and parameterized by a **convention card** that
  maps to the Bridge-Classroom convention card editor.
- Every call the engine makes carries an explanation (what it shows, alert
  text), so it can be used for teaching.
- Import BBA `.bbsa` files into our card format, for compatibility and for
  cross-testing against BBA.

## Non-goals

- Not a port, wrapper, or derivative of BBA/EPBot. See [Clean room](#clean-room).
- Not a double-dummy or simulation-based bidder (like BEN or GIB's Monte Carlo
  bidding). That could be layered on later, but the core is rule-based and
  deterministic.

## Clean room

**Decided.** BBA is a black box to this project.

- Allowed: running `bba-cli` / bba-server to produce reference auctions;
  reading `.bbsa` files (a list of convention names and on/off flags) in order
  to import them; comparing our auctions against BBA's.
- Not allowed: reading, quoting, or modeling on the decompiled EPBot source,
  or any BBA source code. Our module structure, names, and rules come from
  bridge literature, our convention card, and observed bidding.

## Architecture

```
          ┌──────────────────── convention layer (engine-independent) ───────────────────┐
          │                                                                               │
 .bbsa ──►│  bridge-card          conventions/*.bid          bidspec                      │
          │  card schema,   ──►   rule files, one per   ──►  parser for the rule language,│
          │  .bbsa import         convention/system          compiles to a JSON IR         │
          └───────────────────────────────────────────────────────────────────────────────┘
                                             │
                                             ▼
                              engine (rbb-engine)             bridge-types
                              selects the active modules  ◄── Hand, Call, Auction
                              from a card, then interprets
                              them: choose a call, infer
                              what calls show
                                             │
                          ┌──────────────────┼───────────────────┐
                          ▼                  ▼                   ▼
                      rbb (CLI)          rbb-wasm            (future) server
                      bid PBN files,     browser build for
                      corpus scoreboard  Bridge-Classroom
```

**Decided.** The convention layer (`bridge-card`, `bidspec`, `conventions/`)
never depends on the engine. It lives in this repo for now to simplify
development, and may later move to its own repo together with the card editor.

### Crates

| Path | Crate | Role |
|---|---|---|
| `crates/bridge-card` | `bridge-card` | Convention card schema (serde + generated JSON Schema), `.bbsa` importer |
| `crates/bidspec` | `bidspec` | Rule language: lexer, parser, AST, JSON IR, validation |
| `conventions/` | — | `.bid` files: base systems and conventions |
| `crates/engine` | `rbb-engine` | Rule interpreter, auction state, inference |
| `crates/cli` | `rbb-cli` (binary `rbb`) | Bid PBN files; score against reference corpora |
| `crates/wasm` (later) | `rbb-wasm` | wasm-bindgen wrapper with a JSON boundary, same pattern as `bridge-rulebot/wasm` |

Hands, calls, and auctions come from
[`bridge-types`](https://github.com/bridge-craftwork/bridge-types). It needs two
additions, made upstream: serde support and bid legality (sufficiency,
double/redouble rules).

## The convention card

**Decided.** The card schema is defined here, in Rust (`bridge-card`), and a
JSON Schema is generated from it for the Vue editor to consume.

Starting point: the shape of Bridge-Classroom's existing `card_data`
(`schema_version`, `metadata`, and one section per card area: `general`,
`notrump`, `major_openings`, `minor_openings`, `two_level`, `slam`,
`preempts`, `overcalls`, `nt_overcalls`, `doubles`, `competitive`, `carding`,
`leads`, `other_conventions`). Keeping that shape means existing saved cards
load unchanged. Settings are addressed by dotted paths
(`notrump.transfers.jacoby`, `notrump.one_nt.range_min`).

The card records **agreements**, not bid meanings. It says "we play Smolen" and
"our 1NT is 15–17"; what Smolen bids mean lives in `conventions/`. The card
switches modules on and supplies their parameters.

The schema will need fields the editor does not have yet. The `.bbsa` import
is a good checklist: BBA has toggles for Kokish, Rodrigue, Collante, Roudi,
Lavinthal, and others that have no card field today.

### `.bbsa` import

A `.bbsa` file is ~258 lines of `Key = value`: a `System type` integer (0 = 2/1,
1 = SAYC, 2 = Polish Club, 3 = Precision, 4 = Acol), about 170 on/off toggles,
an `Opponent type`, and `Not defined` padding lines. Some toggles form a
one-of group (the four `1NT opening range` keys; `Blackwood 0314` / `1430` /
`0123`).

The importer is a mapping table from each `.bbsa` key to a card path and
value, plus the group rules. Every import produces a report of unmapped keys,
so gaps in our schema are visible. Round-trip test: import all 18 files in
`Practice-Bidding-Scenarios/bbsa/`.

## The rule language (`bidspec`)

**Decided:** a compact custom language that compiles to a JSON IR. The syntax
below is a **first draft** to argue about.

```
# conventions/notrump/jacoby-transfers.bid

module jacoby-transfers
  title  "Jacoby Transfers"
  card   notrump.transfers.jacoby          # module is active when this is true
  needs  notrump-base

# Responder, after partner opens 1NT and RHO passes.
after 1N (P)
  2D  "Transfer to hearts"   alert  shows H>=5
  2H  "Transfer to spades"   alert  shows S>=5

# Opener: super-accept with 4-card support and a maximum, else complete.
after 1N (P) 2D (P)
  3H  "Super-accept"         shows H>=4, hcp=max
  2H  "Completes transfer"
```

Key ideas:

- **Auction patterns** are written from the point of view of the side that
  owns the module. Our calls are bare; opponents' calls are in parentheses.
  `(*)` matches any opponent call, `(P)` only a pass.
- **`shows` is used both ways.** When choosing a call, the engine checks
  whether our hand satisfies `shows`. When partner makes the call, the engine
  records `shows` as what partner's hand now holds. That shared record is what
  lets later rules refer to partner's hand (`partner.H>=3`,
  `ours.hcp>=25`). It is also what the old single-function design could not do
  cleanly.
- **Order is priority.** Within an `after` block, the first rule whose
  conditions fit the hand wins. `priority N` can override this across modules.
- **`when`** adds a selection condition that is *not* promised to partner (for
  tactical choices). Rare; `shows` should carry the meaning.
- **Card parameters** are bound by name:
  `param nt_min = notrump.one_nt.range_min default 15`, then
  `shows hcp=nt_min..nt_max`. Relative terms like `hcp=max` resolve against
  the range the auction has already shown.
- **Overrides are explicit.** A module that replaces a base-system rule says
  `replaces notrump-base` on that rule. If two active modules claim the same
  call at the same auction point and neither says `replaces`, loading fails
  and names both files.
- **Hand-condition vocabulary** (initial): `hcp`, suit lengths `S H D C`,
  `balanced`, `semibalanced`, `stop(S)`, `controls`, `losers`, `tp` (total
  points), `quality(S)`, shape patterns (`5-4-x-x`, `4333`), boolean `,` / `|`
  / `!`, ranges `a..b`.

The IR is plain JSON (serde), so the WASM build, the card editor, and future
tools (for example a rule browser, or an editor for the rules themselves) can
read the same compiled form.

## The engine

For a given hand and auction:

1. Build the **active module set** from the card: base system + every module
   whose `card` path is true, with parameters filled in. Validate conflicts.
2. Replay the auction, and for each call made so far, record what it showed
   (the `shows` of the matching rule). Unmatched calls fall back to generic
   natural inference.
3. Collect the rules whose `after` pattern matches the current auction, in
   priority order, and return the first whose conditions fit the hand.
4. If nothing matches, fall back to a small built-in **natural bidding**
   layer (for example: pass with nothing to say, raise with a fit, bid a long
   suit), so the engine never refuses to bid.

Output: the call, the rule that produced it (module + source line), the
explanation, and the alert text. This matches `bridge-rulebot`'s reason-code
style.

## Testing against the corpora

Reference data is in `Practice-Bidding-Scenarios`:

- `bba/`: 342 PBN files × 500 boards, all four seats bid by BBA. The file
  header names the `.bbsa` cards used; each scenario's `btn/*.btn` names them
  as `convention-card-ns/ew`. Alerts appear as `=n=` with `[Note "n:Stayman"]`.
- `GIB/`: ~56 PBN files, ~30–50 boards each, BBO "BasicGIB 2/1", no alerts.

The `rbb corpus` command replays each reference auction. At each turn of the
side under test it asks the engine for a call, given that hand and the
auction so far, and reports per scenario:

- agreement rate of our call with the reference call;
- position of the first difference;
- whether our explanation matches the reference alert label.

The output is a scoreboard of which scenarios pass. Differences that GIB and
BBA also disagree on are informational, not failures.

Reference outputs are pinned to a specific BBA library **by sha256**: builds
that all report version 8740 bid differently.

## Milestones

1. **Scaffold** (done): workspace, licenses, this document.
2. **Card**: `bridge-card` schema covering the existing Bridge-Classroom seed
   card; `.bbsa` import with an unmapped-key report for all 18 PBS files.
3. **Language**: `bidspec` parser and JSON IR for the draft syntax, with good
   error messages (file, line, what was expected).
4. **First slice**: 1NT opening and uncontested responses (Stayman, Jacoby,
   Texas, Smolen, 4-way transfers), end to end through the engine.
5. **Scoreboard**: `rbb corpus` against the 1NT scenarios in `bba/`.
6. **WASM**: browser build, wired into Bridge-Classroom beside `bbaClient.js`.
7. Then widen: major openings, minor openings, two-level openings, slam
   bidding, and finally competitive bidding. Competitive bidding is the
   hardest part and should wait until the rule format has held up.

## Open questions

- **Auction anchoring.** How patterns handle leading passes and seat
  position (`after 1N (P)` should match whether 1NT was opened first or third
  seat, but third/fourth-seat openings differ). Proposal: leading passes are
  ignored unless written, with a `seat` condition when it matters.
- **Natural fallback.** How much of general bidding is built-in Rust versus a
  `natural.bid` module.
- **Hand evaluation.** Where upgrades/downgrades and judgment live (a
  pluggable evaluation function referenced from rules?).
- **Opponent modeling.** Whether we read the opponents' calls through their
  own card (as BBA does with CC1/CC2), which needs both cards loaded.
- **Versioning.** How card `schema_version` and IR versions evolve once cards
  are stored in Bridge-Classroom's database.
