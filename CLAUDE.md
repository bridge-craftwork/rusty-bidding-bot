# rusty-bidding-bot

Rule-based bridge bidding engine in Rust (native + WASM). Design: docs/DESIGN.md;
the proposed top-down judgment layer (placement, competition, slam entry): docs/JUDGMENT-LAYER.md.

## Hard rules

- **Clean room.** Never read the decompiled EPBot/BBA source
  (`bba-mac-private/decompile/` and similar) or model code on it. BBA is used
  only as a black box via `bba-cli` outputs and `.bbsa` imports. See
  CONTRIBUTING.md.
- **Convention layer is engine-independent.** `bridge-card` (now in the
  convention-card repo), `crates/bidspec`, and `conventions/` must never
  depend on `rbb-engine` or `rbb-cli`.

## Related repos (sibling checkouts, `../<repo>`)

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

**Use `./dev-build.sh` for local builds, not bare cargo** (as in the other
bridge-craftwork Rust repos). This repo depends on sibling crates
(`bridge-types`, `bridge-encodings`, `bridge-solver`, and `bridge-card` from
`convention-card`) as git dependencies, with gitignored `[patch]` overrides in
`.cargo/config.toml` redirecting them to the local checkouts in `../`. Bare
cargo with those patches either silently builds the GitHub revisions or, when
the patch takes effect, rewrites `Cargo.lock` with local-path entries that must
never be committed (CI has no sibling checkouts). The script keeps a separate
local lock (`.cargo/dev.lock`), verifies each patched crate resolved to its
checkout, and leaves the committed `Cargo.lock` untouched. For CI parity
(patches off, the committed lock's git pins) use `./dev-build.sh --ci <args>`.
`cargo fmt` resolves nothing, so bare cargo is fine for it.
`crates/wasm/build.sh` restores `Cargo.lock` after its `wasm-pack` run for the
same reason. To get a sibling change into CI: push and tag it there, move the
pin here, and commit the re-resolved lock (docs/RELEASING.md, "Building against
local checkouts").

- `./dev-build.sh test --workspace`
- `./dev-build.sh run -q -p rbb-cli -- card import-bbsa <file.bbsa>`: card JSON on stdout,
  passthrough report on stderr. Also `export-bbsa`, `check`, `schema`.
- `./dev-build.sh run -q -p rbb-cli -- card coverage <file.bbsa>...`: how much of each
  card the rules read — settings honoured, settings ignored, `.bbsa` keys with
  no field (`-v` lists them). `compare --min-coverage 50` restricts the
  comparison to scenarios whose cards we cover that well.
- `./dev-build.sh run -q -p rbb-cli -- bid check`: parse and check every `.bid` file in
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
- Teaching skills (docs/SKILLS.md): each module names the Bridge-Classroom
  skills it implements (`skill bidding_conventions/stayman` header lines),
  each convention's card field its skill (`skill = ...` in fields.toml);
  the card field is the canonical convention ID. Known paths: the standard
  conventions and skills in convention-card's `spec/conventions/` (a new one
  is a pull request there). After
  changing any, run `rbb bid skills --doc docs/SKILLS.md` (a test checks
  the doc).
- `./dev-build.sh run -q -p rbb-cli -- bid test [paths]`: run the `<module>.test` cases
  (`seat hand | auction | expect | why`) next to the modules; `-v` lists passes.
  `./dev-build.sh test` runs them too. Put call expectations there, not in Rust tests.
- **Par decides, BBA teaches**: where agreement with BBA and distance from
  double-dummy par disagree, par wins, and the trade goes in the notes and
  the commit message (docs/DESIGN.md).
- `<module>.notes.md` next to each module: Rick's guidance, probe and corpus
  evidence, accepted differences from BBA, gaps and open questions. Update it
  when a decision is made or a probe settles something. Each ends with a
  **Sources** section: where the rules come from (books, articles, URLs,
  Rick's dated rulings, probe specs, corpus measurements) and where they
  differ from that source; "standard practice, not yet cited" when that
  is the truth (docs/CONTRACT.md, Part 2).
- `./dev-build.sh run -q -p rbb-cli -- call <S.H.D.C> -a "1NT Pass" -d S -c <card.bbsa>`:
  the engine's call with the candidate trace (`--json` for everything).
- `./dev-build.sh run -q --release -p rbb-cli -- compare [SCENARIO...] [--limit N] [--par]`:
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
  **"vs BBA, errors"** (Rick, 2026-10-01; `par::table_errors`) charges
  each side with its own errors instead: a contract that goes down
  undoubled and beats par that way is taken as doubled, the **contract
  error** (assumed result against par) goes to the side it leaves worse
  off, the **doubling error** (actual against assumed: not doubling, a
  double of a making contract, a redouble into a set) to the side the
  actual result leaves worse off. Two errors at a table add instead of
  cancelling, and a penalty double of an overbid is no longer a loss.
  The JSON has both tables' errors per board (`reference_errors`,
  `ours_errors`: `contract` and `doubling`, each `[NS, EW]`). A contract
  counts as above par when it outranks every par contract (or beats par
  undoubled): DD fact, not a judgment of whether anyone would double.
  **"how each table met par"** (`parclass`) puts every board with a DD
  table in one class per engine: at par, short of slam / game / level,
  wrong strain or declarer, overbid, did not compete, the other side's
  overbid or missing sacrifice, passed out, a doubling error only; with
  boards and the table's errors. **"penalty doubles"** counts the chances
  (a contract above par that goes down), how many each engine doubled,
  doubles of contracts that made, and bail-outs (bidding on over a failing
  overbid instead of doubling). JSON: `reference_class`, `ours_class`.
  `--set general.style=bba` plays BBA's treatments where we have them
  (docs/DESIGN.md, "Treatments"), for A/B tests; `--set path=value` works
  for any card field, on both sides.
  `--auctions ns|ew|competitive` keeps only the boards where, in BBA's
  auction, one side bid alone or both sides bid (the workbench has the same
  filter in its toolbar).
  "calls read as a higher rule than chose them" counts our own calls
  that partner reads by a higher-priority rule than the one that bid
  them (a judgment or fallback rule claiming a call a higher rule
  offers; docs/JUDGMENT-LAYER.md §4), with the top (chosen -> read as)
  pairs; `read_as` in `--json`.
- `./dev-build.sh run -q --release -p rbb-cli -- probe --hand S=<S.H.D.C> --vary-tens --prefix "1NT Pass 2NT Pass" --dealer S`:
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
- **Judging a change:** distance from par suits uncontested auctions
  but counts a successful competitive call or a penalty double of an
  overbid as a loss. For competitive and doubling changes judge by the
  **errors** line (each side charged with its own errors, an overbid taken
  as doubled; Rick, 2026-10-01) and report distance from par beside it.
  Our engine plays both sides, so our weak doubling also judges our
  bidding. Rick (2026-10-02): judge a **bidding** change (competing,
  raising, overbidding) on its **contract** errors and distance from
  par; judge a **doubling** change on its **doubling** errors. Report
  the other half beside it.
  `probes/tools/sideimps.py BASE.json VARIANT.json` (IMPs to the side
  that made the first differing call) predates it and is flattered by
  weak runouts; use it only as a third opinion.
  Corpus tables mix cards (1NT ranges, transfer structures): restrict
  them to one card.
- `./dev-build.sh run -q --release -p rbb-cli -- bid-pbn -i in.pbn -o out.pbn --ns-card X.bbsa --ew-card Y.bbsa`:
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
- `./dev-build.sh run --release -p rbb-workbench [SCENARIO...] [--limit N]`: the GUI over
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

The card vocabulary is the standard one in the [convention-card](https://github.com/bridge-craftwork/convention-card) repo
(`spec/fields.toml`, `spec/formats/bbsa-map.toml`, `spec/conventions/`),
which the `bridge-card` crate carries at the tag `Cargo.toml` pins
(`bridge_card::standard`). Add or change a card field, a `.bbsa` mapping or a
convention there, by pull request; then cut a tag there and move the pin here.
A conventions directory may still bring its own `card/fields.toml` and
`card/bbsa-map.toml` (`rbb_engine::rules_vocabulary`), and a card is always
read in its rule set's vocabulary (`rbb_engine::RuleSet`). Do not guess the
meaning of an unmapped `.bbsa` key: leave it in passthrough and ask. The stock
`.bbsa` cards are in `cards/bbsa/`.
