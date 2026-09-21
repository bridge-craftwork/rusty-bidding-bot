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
- `cargo run -q -p rbb-cli -- bid check`: parse and check every `.bid` file in
  `conventions/`; `bid compile <file>` prints the JSON IR.
- `cargo run -q -p rbb-cli -- call <S.H.D.C> -a "1NT Pass" -d S -c <card.bbsa>`:
  the engine's call with the candidate trace (`--json` for everything).
- `cargo run -q --release -p rbb-cli -- compare [SCENARIO...] [--limit N] [--par]`:
  compare with BBA's auctions in `../Practice-Bidding-Scenarios` (all 342
  scenarios in ~8 s in release). The top divergence points are the work queue.
- `cargo run --release -p rbb-workbench [SCENARIO...] [--limit N]`: the GUI over
  the same comparison; re-runs when a `.bid` file is saved. `--editor` sets how
  rule links open (default `code -g {file}:{line}`).

## Card fields

Add or change card fields in `crates/bridge-card/data/fields.toml`, and `.bbsa`
mappings in `data/bbsa-map.toml`; tests validate both files. Do not guess the
meaning of an unmapped `.bbsa` key: leave it in passthrough and ask.
