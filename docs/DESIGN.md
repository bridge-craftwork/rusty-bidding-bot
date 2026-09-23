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
| `crates/compare` | `rbb-compare` | Headless comparison against reference corpora: statistics, divergences, par |
| `crates/cli` | `rbb-cli` (binary `rbb`) | Bid PBN files; `rbb compare` report |
| `crates/workbench` | `rbb-workbench` | Desktop GUI over `rbb-compare` (see [Comparison workbench](#comparison-workbench)) |
| `crates/wasm` (later) | `rbb-wasm` | wasm-bindgen wrapper with a JSON boundary, same pattern as `bridge-rulebot/wasm` |

Hands, calls, and auctions come from
[`bridge-types`](https://github.com/bridge-craftwork/bridge-types). It needs two
additions, made upstream: serde support and bid legality (sufficiency,
double/redouble rules).

## The convention card

**Decided.** The card schema is defined here (`crates/bridge-card`). It may
later move to its own repo together with the card editor.

**Implemented (milestone 2)** as a data-driven registry rather than Rust
structs:

- [`data/fields.toml`](../crates/bridge-card/data/fields.toml) declares every
  field once: dotted path, kind (`bool | int | enum | text`), label, options,
  bounds, default, and **aliases** (older paths that still load). A card is a
  set of `path = value` pairs. Adding a convention means adding a field there
  and a `.bid` module that names it; no Rust changes are needed. This is the
  same direction `conventionCatalog.js` describes for its "Phase 2".
- Cards are read and written as Bridge-Classroom's nested `card_data` JSON, so
  existing saved cards load. Leaves the registry does not know about are
  kept and written back unchanged. So is the editor's own `skill_path`.
- The seed card and the editor catalog disagree on some paths. For example,
  RKCB is at `slam.blackwood.*` in the seed and `other_conventions.blackwood.*`
  in the catalog, and the seed's `fourth_suit_forcing.game_forcing` is
  `game_force` in the catalog. The registry picks one canonical path and
  lists the others as aliases.
- `rbb card schema` generates a JSON Schema (draft 2020-12) from the registry
  for the editor.

The card records **agreements**, not bid meanings. It says "we play Smolen"
and "our 1NT is 15–17"; what Smolen bids mean lives in `conventions/`. The
card switches modules on and supplies their parameters. The engine reads
`effective` values: the card's value, or the registry default when unset.

### `.bbsa` import and export

A `.bbsa` file is 258 CRLF lines of `Key = value`: a `System type` integer
(0 = 2/1, 1 = SAYC, 2 = Polish Club, 3 = Precision, 4 = Acol), about 170
on/off toggles, an `Opponent type`, and `Not defined` padding.
[`data/bbsa-map.toml`](../crates/bridge-card/data/bbsa-map.toml) maps each key
to card fields, in one of three forms:

- a toggle (`"SMOLEN" = "notrump.smolen.play"`);
- one member of a one-of group, which sets values when on
  (`"1NT opening range 12-14"` sets the range);
- an integer that selects an enum value (`System type`).

Keys whose meaning needs a bridge decision are **not guessed**. They are
kept verbatim in the card's `bba_passthrough` and written back on export, so
`.bbsa → card → .bbsa` is lossless. Every current-layout file in
`Practice-Bidding-Scenarios/bbsa/` exports back byte for byte; older-layout
files keep every setting (tested). A treatment field that BBA spreads over
several switches is derived on import (`[[derived]]` in `bbsa-map.toml`):
BBA's switches stay on the card for export, and the rules read the field.
`rbb card import-bbsa` lists the passthrough keys. Giving them card fields is a task for a bridge expert.

Commands: `rbb card import-bbsa`, `export-bbsa`, `check`, `schema`.

## The rule language (`bidspec`)

**Decided:** a compact custom language that compiles to a JSON IR.

> **Superseded.** Rules cannot be keyed only by auction patterns (no one can
> list every auction that reaches a Keycard 4NT). The current draft,
> [LANGUAGE.md](LANGUAGE.md), builds knowledge about each hand and state for
> each side call by call. Rules key off that knowledge and state, and the
> engine ranks the candidate calls. The text below is the original sketch,
> kept for history.

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

> Updated model: see [LANGUAGE.md §1 and §7](LANGUAGE.md). Interpret each
> call to build knowledge and state, then generate, filter, and rank the
> candidate calls. The steps below are the original outline.

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

## Variables that affect a call

A call depends on more than the hand and the auction. Each variable below
is recorded for every compared board, and `rbb compare` breaks agreement
down by it, so we can see when one matters.

| Variable | In the BBA reference corpus | In our engine |
|---|---|---|
| Convention card, per side | from each scenario's `.btn` (`convention-card-ns/ew`) | modelled: activates modules, supplies parameters |
| Vulnerability | varies by board, evenly (None/NS/EW/All ≈ 42k each) | rules can test `vul`, `they.vul`; no rule uses them yet |
| Scoring | **matchpoints**: the PBS pipeline does not pass `--scoring`, so bba-cli's default `MP` applies, and bba-cli passes it to BBA's engine for every player (`epbot_set_scoring`). 18 older files (4,000 boards) do not record it | input to the engine (from `[Scoring]`, default MP); rules can test `imps` / `matchpoints`; no rule uses them yet |
| Dealer and seat | varies by board | `seat`, `passed_hand`, `opening` |
| Reference engine build | varies by file (`% Generated by`): bba-cli 324 files, BBA 8738 GUI 8, bba-cli-mac 7, none 3. The library build (sha256) is not recorded per file | reported per generator; not a factor in our bidding |
| BBA `Opponent type` | 0 in every card; meaning unknown | kept verbatim (`bba_passthrough`) |

**Not modelled yet: stretching for games.** The strength bands use fixed
thresholds (game at 25 combined, slam at 33). "Bid a close vulnerable game
at IMPs" belongs there: for example a lower game threshold when `imps` and
`vul`, which moves the invite/game boundaries for every rule written with
`strength`. Since the reference is all matchpoints, checking such a change
against BBA needs an IMP reference set (bba-cli `--scoring IMP`).

## Comparison workbench

Getting the engine right means many small, fast rounds: find where we differ
from BBA, see why, fix a rule or an evaluator, and re-run. The workbench is
built for that loop, and it is the milestone that comes right after the
engine core.

### Engine trace (a requirement from day one)

For every call it makes, the engine returns a **trace** as well as the call:

- the candidate calls, ranked, with the rule behind each (module, file, line);
- for each losing candidate, why it lost (a `shows` or `when` clause failed,
  or it ranked lower);
- a snapshot of the knowledge about all four hands, and each side's auction
  state (trump, forcing, pending ask), as they were **before** the call.

The workbench shows the trace at the point of divergence, and the CLI and
unit tests print it. It is serde data, so any tool can read it.

### Reference data

In `Practice-Bidding-Scenarios`:

- `bba/`: 342 PBN files × 500 boards, all four seats bid by BBA. Each
  scenario's `btn/*.btn` names the cards used, as `convention-card-ns/ew`;
  we load the matching `.bbsa` files through our importer, so both engines
  bid with the same agreements. Alerts appear as `=n=` with
  `[Note "n:Stayman"]`.
- `GIB/`: 65 scenarios, ~50 boards each, BBO "BasicGIB 2/1", no alerts
  (none yet for 1NT, Stayman or transfers). A third party for deciding
  conflicts between us and BBA, consulted by hand; it is not part of
  `rbb compare`.

**BBA's meanings.** The PBS files carry only alerts. EPBot describes every
call through its public info API: a short meaning ("Cue bid, a ♣ stopper",
"denies stoppers: ♦") and an extended one with ranges ("7 to 10 total
points, 5 to 13 cards in spades"). `bba-cli --all-meanings` (BBA-Tools PR #24) writes both as a `[Note]` on every
call, so a scenario can be re-bid to see what BBA meant.

Reference outputs are pinned to a specific BBA library **by sha256**: builds
that all report version 8740 bid differently. New reference sets are made by
dealing with **dealer3** (reusing the PBS dealer scripts) and bidding with
`bba-cli`.

### Comparison modes

- **Full auction.** Our engine bids all four seats, then the auction and
  final contract are compared with BBA's. This is the headline number.
- **Replay.** At each turn of the side under test, give the engine BBA's
  auction so far and compare its single call. A single early difference then
  does not hide everything after it. This measures each call on its own and
  works before the engine can bid every seat.

### Statistics

For each scenario and in total:

- auction match %, final-contract match %;
- **first divergence**: a histogram by call number, and, most useful for
  deciding what to work on, a table of the **most common divergence
  points**, grouped by the auction so far:

  | Auction so far | BBA | Ours | Boards |
  |---|---|---|---|
  | `1N P 2C P` | 2H | 2S | 41 |
  | `1S P 2N P` | 3C | 4S | 17 |

- **Par comparison**, using bridge-solver in the same process
  (`solve_dd_table` and `par()`): for each board, the double-dummy result of
  our contract and of BBA's, each scored against par. Totals give an IMP
  difference: **when we differ from BBA, who got closer to par?** A
  divergence where we reach par more often may be an improvement, not a bug.

  The table comes from the corpus file when it has one: the reference PBNs
  carry a PBN 2.1 `OptimumResultTable` section, which `bridge-encodings`
  reads into `Board::double_dummy_tricks`. Those boards are scored against
  par on every run, `--par` or not, and nothing is solved. `--par` is for
  the rest: it solves the deals that have no table, and those are cached by
  deal in `.rbb-cache/dd.jsonl` so they are computed once. A board's table
  is kept on its `BoardResult`, so the workbench can work out par for a
  selected board without solving it.

### The GUI

A desktop app in Rust. **Decided: egui/eframe**, the simplest choice for a
data-heavy tool (tables, histograms, inspectors), and it can also build for
the web. Bridge-Classroom's Vue components (hands, DD tables) are the path if this later becomes a web interface.

- **Scenario list**: match %, contract match %, par IMPs vs BBA; sortable and
  filterable.
- **Divergence view** for the selected scenario, or all of them: histogram,
  plus the divergence-point table above. Clicking a row lists its boards.
- **Board detail (A/B)**:
  - the four hands;
  - both auctions side by side, with the first divergence highlighted;
  - the engine trace at that call: ranked candidates, rules with
    `file:line` (click to open in the editor), why each alternative lost;
  - the knowledge table for all four seats, and each side's state;
  - double-dummy table, par, and the result of each contract.
- **Hot reload**: the app watches `conventions/` and re-runs automatically
  when a `.bid` or `.test` file is saved, then shows what changed ("+23 boards now match,
  4 now differ"). Editing a rule and seeing the effect a few seconds later is
  the point of the tool.

A headless library does all the work. The GUI and a CLI report
(`rbb compare`, for CI and quick checks) are thin layers over it.

### Problems: wrong whatever the convention

BBA agreement mixes judgment (where a point boundary falls) with mistakes.
Problems are the mistakes, found in our own auctions independently of BBA,
and shown in `rbb compare` and the workbench's Problems tab:

- **no rule in a live auction**: the engine had nothing to say once its side
  had bid (entering over the opponents is not counted yet);
- **contract is an artificial call**: the auction ended in a call its rule
  marks `artificial` (a transfer, a keycard answer), unless it happens to
  be an eight-card fit;
- **trump fit under 7 cards**, from the two level up;
- **contradicts earlier calls**: a call the engine read as impossible given
  what the same player had shown;
- **auction not finished**.

The target is zero. Differences from BBA that are judgment are recorded
in each module's notes as accepted.

### Probes: asking BBA directly

`rbb probe` makes a small reference set on purpose, to isolate one
decision. Deals come from fixed holdings (with `--vary-tens`, the same hand
with 0-4 tens), a dealer3 script, or at random. bba-cli (default
`/Applications/Bridge Utilities/bba-cli`) bids them with `--auction-prefix`
forcing the auction up to the decision, with any card: a PBS card, a
`.bbsa` file, or `bare:2/1` (every toggle off), plus `--set Key=value`
edits. Our engine replays the result, and the report shows the decision
hand by hand. Files go to `.rbb-cache/probes/`. Example:

```
rbb probe --hand S=AQ52.KJ73.A95.Q8 --vary-tens --prefix "1NT Pass 2NT Pass" --dealer S
```

### Rule-level tests

Next to each module, a `<module>.test` file holds hands whose call is
agreed, one per line: `seat hand | auction | expect | why`. Header lines
(`card`, `dealer`, `vul`, `scoring`) apply to the lines below them. `card`
names a `.bbsa` in the cards directory (default
`crates/bridge-card/tests/fixtures/bbsa`), or a path; both sides play it.
Changes can follow, so one file can test each treatment:
`card 21GF-DEFAULT notrump.minor_transfers=four_way`.
`expect` is a call, `!call` (anything but), or alternatives `a/b`.

```
card    21GF-DEFAULT
dealer  S
N 82.QJ973.K94.J83  | 1NT P      | 2D  | weak, five hearts: transfer
S AK5.Q72.Q942.K83  | 1NT P 2D P | !P  | the transfer is forcing
```

`rbb bid test [paths]` runs them and prints each failure with its
candidates. `cargo test` runs every `conventions/**/*.test`
(`crates/engine/tests/cases.rs`), and the workbench's Cases tab lists
failing cases, re-running when a `.bid` or `.test` file is saved. Tests of
what the engine *knows* (interpretation, knowledge, traces) stay in Rust
(`crates/engine/tests/interpretation.rs`).

Next to each module, a `<module>.notes.md` records the agreed guidance, the
probe and corpus evidence behind it, the accepted differences from BBA, the
gaps, and questions still open.

Later, **generated tests**: turn a rule's context and `shows` into a dealer3
script, deal hands that fit, and check that the engine chooses that rule, or
knowingly chooses a higher-ranked one.

## Milestones

1. **Scaffold** (done): workspace, licenses, design documents.
2. **Card** (done): field registry, card JSON load/save compatible with
   Bridge-Classroom, JSON Schema, lossless `.bbsa` import/export.
3. **Language** (done): `bidspec` parser and JSON IR, with file:line:column
   errors; `rbb bid check` / `rbb bid compile`.
4. **Engine core with trace** (done): knowledge store, auction state,
   ranking, negative inference, trace; `rbb call`. The 1NT slice (opening,
   Stayman, Jacoby transfers, natural responses) bids end to end.
5. **Compare library + CLI** (done): `rbb compare` replays every BBA board
   (the whole corpus in about 8 seconds), finishes the auction from the
   first difference, reports agreement and the most common divergence
   points, and with `--par` scores differing contracts against double-dummy
   par (tables cached in `.rbb-cache/`).
6. **Workbench GUI** (done): `rbb-workbench [SCENARIO...]`, where a
   scenario argument may be a pattern (`'Basic_*'`). Scenario list, divergence
   table, board list, and board detail: hands, both auctions, the engine's
   reading of BBA's calls, and at the first difference the ranked candidates
   (click a rule to open it in the editor) with each seat's known holding.
   It re-runs when a `.bid` file is saved and shows what changed.
7. **Iterate**: widen coverage in the workbench. Major openings, minor
   openings, two-level openings, slam bidding, then competitive bidding.
8. **WASM**: browser build, wired into Bridge-Classroom beside `bbaClient.js`.

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
