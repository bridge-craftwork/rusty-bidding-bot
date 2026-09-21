# rusty-bidding-bot

An open-source bridge bidding engine in Rust, for native platforms and WASM.

Bidding systems and conventions are written as small rule files in
[`conventions/`](conventions/), switched on by a convention card, so new
conventions can be added without touching the engine. Every call comes with an
explanation of what it shows.

Status: early design. Start with [docs/DESIGN.md](docs/DESIGN.md).

## Layout

- `crates/bridge-card`: convention card schema, `.bbsa` import
- `crates/bidspec`: the rule language
- `conventions/`: system and convention rule files (`.bid`)
- `crates/engine`: the bidding engine
- `crates/compare`: comparison with reference auctions (BBA)
- `crates/cli`: the `rbb` command-line tool
- `crates/workbench`: `rbb-workbench`, a desktop GUI for the comparison

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
