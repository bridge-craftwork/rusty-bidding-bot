# rusty-bidding-bot

Rule-based bridge bidding engine in Rust (native + WASM). Design: docs/DESIGN.md.

## Hard rules

- **Clean room.** Never read the decompiled EPBot/BBA source
  (`bba-mac-private/decompile/` and similar) or model code on it. BBA is used
  only as a black box via `bba-cli` outputs and `.bbsa` imports. See
  CONTRIBUTING.md.
- **Convention layer is engine-independent.** `crates/bridge-card`,
  `crates/bidspec`, and `conventions/` must never depend on `rbb-engine` or
  `rbb-cli`; they are expected to move to their own repo with the card editor.

## Related repos (siblings under ~/Development/GitHub)

- `bridge-types`: Hand, Call, Auction (git dependency).
- `bridge-rulebot`: cardplay bot; reference for the WASM wrapper and the
  local-dev `[patch]` + dev-build.sh pattern.
- `Bridge-Classroom`: convention card editor (`src/utils/conventionCatalog.js`,
  `src/components/conventionCard/`), seed card in
  `bridge-classroom-api/seed_data/21_intermediate_card.json`.
- `Practice-Bidding-Scenarios`: reference corpora `bba/` (BBA, 500 boards per
  scenario), `GIB/`, card settings `bbsa/`, scenario definitions `btn/`.
- `BBA-tools`: `bba-cli` for producing reference auctions.

## Commands

- `cargo test --workspace`
- `cargo run -q -p rbb-cli -- card import-bbsa <file.bbsa>`: card JSON on stdout,
  passthrough report on stderr. Also `export-bbsa`, `check`, `schema`.
- `cargo run -q -p rbb-cli -- card coverage <file.bbsa>...`: how much of each
  card the rules read — settings honoured, settings ignored, `.bbsa` keys with
  no field (`-v` lists them). `compare --min-coverage 50` restricts the
  comparison to scenarios whose cards we cover that well.
- `cargo run -q -p rbb-cli -- bid check`: parse and check every `.bid` file in
  `conventions/`; `bid compile <file>` prints the JSON IR.
- docs/CONTRACT.md is the contract between engine and conventions (they
  will split into two repos): what the engine promises, what a good
  convention file does. `conventions/conventions.toml` is the rule set's
  manifest; its `language` must be one the engine reads
  (`rbb_engine::LANGUAGE_VERSION`, shown by `rbb --version`), or loading
  refuses. A change that alters how existing rule files load or read
  needs a language version bump. `rbb bid terms` prints every term a
  condition may use; the meanings live in the term tables in
  `crates/engine/src/eval.rs`, and after changing them run
  `rbb bid terms --doc docs/CONTRACT.md` (a test checks the doc).
- `cargo run -q -p rbb-cli -- bid test [paths]`: run the `<module>.test` cases
  (`seat hand | auction | expect | why`) next to the modules; `-v` lists passes.
  `cargo test` runs them too. Put call expectations there, not in Rust tests.
- **Par decides, BBA teaches**: where agreement with BBA and distance from
  double-dummy par disagree, par wins, and the trade goes in the notes and
  the commit message (docs/DESIGN.md).
- `<module>.notes.md` next to each module: Rick's guidance, probe and corpus
  evidence, accepted differences from BBA, gaps and open questions. Update it
  when a decision is made or a probe settles something.
- `cargo run -q -p rbb-cli -- call <S.H.D.C> -a "1NT Pass" -d S -c <card.bbsa>`:
  the engine's call with the candidate trace (`--json` for everything).
- `cargo run -q --release -p rbb-cli -- compare [SCENARIO...] [--limit N] [--par]`:
  compare with BBA's auctions in `../Practice-Bidding-Scenarios` (all 342
  scenarios in ~45 s in release). The top divergence points are the work
  queue. Par comes free for boards whose corpus file carries an
  `OptimumResultTable`; `--par` solves the deals that have none.
  A scenario argument may be a pattern: `compare 'Basic_*'`, quoted so the
  shell leaves it alone. The workbench takes the same arguments. The cards
  each side played come from the PBN's own `% CC1/CC2` header, falling back
  to the `.btn`.
  `--by-imps` orders the divergence points by what they cost against BBA
  instead of by how often they happen.
  **"vs BBA" / "bba/bd"** (the workbench column too) is our score against
  BBA's in IMPs, with double-dummy par as the yardstick: on each board
  where the contracts differ, BBA's IMP distance from par minus ours.
  Positive means ours was closer. It is not our distance from par. Notes
  written before 2026-09-25 call it "par".
  `--set general.style=bba` plays BBA's treatments where we have them
  (docs/DESIGN.md, "Treatments"), for A/B tests; `--set path=value` works
  for any card field, on both sides.
  `--auctions ns|ew|competitive` keeps only the boards where, in BBA's
  auction, one side bid alone or both sides bid (the workbench has the same
  filter in its toolbar).
- `cargo run -q --release -p rbb-cli -- probe --hand S=<S.H.D.C> --vary-tens --prefix "1NT Pass 2NT Pass" --dealer S`:
  ask bba-cli how it bids chosen hands and compare (`--ns-card bare:2/1`,
  `--set Texas=0`, `--script file.dlr`, `--scoring IMP`).
  **Find what a decision turns on by making hands, not by searching the
  corpus** (Rick, 2026-09-24), with `rbb grid probes/<name>.toml` (half a
  second): a spec gives the card, dealer, the forced `prefix`, `vuls`,
  `scoring`, a list of `partner` hands (the first that shares no card
  with a variant is used: partner's cards do not change the decision),
  `our = ["general.style=bba"]`, and one of `hands = ["label | S.H.D.C",
  ...]`, `survey = "S.H.D.C"` (every single-card exchange with the
  opponents; prints a summary and the changes) or `morph = [from, to]`.
  It prints one row per hand and one column per vulnerability/scoring
  (BBA's call, and ours after ≠ when it differs), and writes grid.tsv and
  grid.json; each cell also keeps BBA's auction from there (`bba_rest`)
  and its meaning of the call (`bba_alert`). Keep the specs in probes/
  and cite them from the notes. For bulk random hands,
  `probes/gen_hands.py OUT.toml --hcp 9 14 --where "<python on L, h>"
  --partner-where ... --prefix ...` writes a spec, and
  `probes/grid_tally.py <name> [--by hcp] [--diff]` tallies the run;
  `probes/grid_diff.py <name> --key "L[0]" --key h` groups the
  disagreements.
- **Judging a change:** the corpus par figure is distance from par, which
  suits uncontested auctions but counts a successful competitive call
  as a loss. For competitive changes also run
  `probes/tools/sideimps.py BASE.json VARIANT.json` on two
  `compare --json` runs: double-dummy IMPs to the side that made the
  first differing call. Report both; act where they agree
  (overcalls.notes.md, "For Rick: which yardstick").
  Corpus tables mix cards (1NT ranges, transfer structures): restrict
  them to one card.
- `cargo run -q --release -p rbb-cli -- bid-pbn -i in.pbn -o out.pbn --ns-card X.bbsa --ew-card Y.bbsa`:
  bid every deal of a PBN file, written in bba-cli's layout (`[Auction]`,
  `[Note]`s for alerts, `--all-meanings` for every call). Uses the rules
  compiled into the binary unless `--rules`; cards may be stock names
  (`21GF-DEFAULT`). docs/RELEASING.md covers it and the release builds.
- `crates/wasm/build.sh`: the WASM package (`crates/wasm/pkg`); API in
  docs/WASM.md, which a website builds on: change it compatibly or bump
  `API_VERSION`. `rbb bid reference` prints the conventions reference
  the WASM build serves. It bids with the embedded rules, or with a rule
  set a request supplies as text (`rules`, `fields`, `bbsa_map`,
  `manifest`); `validate` checks one without bidding.
- `web/build.sh`: the web site in `web/dist` (docs/WEB.md): engine,
  dealer3, `reference.txt`; `node web/scripts/test.mjs web/dist` tests
  its `window.rustyBiddingBot` contract. Serve with
  `python3 -m http.server -d web/dist`.
- `cargo run --release -p rbb-workbench [SCENARIO...] [--limit N]`: the GUI over
  the same comparison; re-runs when a `.bid` file is saved. `--editor` sets how
  rule links open (default `code -g {file}:{line}`).

## Tickets

Rick files tickets from the workbench's **Report…** button: his note plus
everything the workbench showed, in
`tickets/YYYY/MM/DD/NN.<status>.<slug>/` (gitignored; `NN` numbers the
day's tickets from 01, `<status>` mirrors the frontmatter), and optionally as a GitHub
issue labelled `workbench-ticket` (`--ticket-repo owner/repo` overrides the
origin remote; `--ticket-dry-run` or `RBB_TICKET_DRY_RUN=1` prints the `gh`
commands instead of running them).

- **Work one:** read `ticket.md` (the note, board, auctions, first
  difference, the engine's reading of BBA's auction, par), then
  `context.json` for anything the summary leaves out; together they are
  usually enough to diagnose (the candidates, their rules and why each
  lost, and the engine's reading of every BBA call are all there). Run
  the `rbb call` / `compare` lines only when the rules have changed
  since the ticket's commit (check `git` in the context: rules
  hot-reload, so uncommitted `.bid` edits may have shaped what Rick
  saw), or to test a fix and positions the ticket does not show.
  Diagnose before changing anything, then fix or explain, judging the
  change by the rules above (par decides, the competitive yardstick,
  `.test` cases, notes).
- **Record the verdict** with `probes/tools/ticket.py status DIR
  resolved|wontfix|duplicate --resolution "..." --refs a,b`: it sets the
  frontmatter (`status`, `resolution`: one line on what was done or why
  not, `refs`: the commits and any ticket it duplicates) and renames the
  folder to the new status, so the two never disagree. A ticket left
  for Rick stays `open` with the question in `resolution`.
  `ticket.py list [--status open]` lists them. Close the GitHub issue,
  if `issue:` names one, with the same words.
- `context.json` is evidence: never edit it.

## Card fields

The card vocabulary belongs to the rules: add or change card fields in
`conventions/card/fields.toml`, and `.bbsa` mappings in
`conventions/card/bbsa-map.toml`. Every consumer reads them from the rules
directory it was given (`--rules DIR`, default `conventions`; the embedded
copy in `rbb-assets` for the release `rbb` and the WASM), and a card is
always read in its rule set's vocabulary (`rbb_engine::RuleSet`), so a new
field needs no Rust change. `rbb bid check` and `cargo test` validate both
files. Do not guess the meaning of an unmapped `.bbsa` key: leave it in
passthrough and ask.
