# Contributing

## Clean room: read this first

This project is an independent implementation. It is **not** derived from
BBA / EPBot.

- **Do not** read, quote, or model on the decompiled EPBot source, or any BBA
  source code. If you have studied it, say so in your pull request and do not
  work on the areas you studied.
- **Fine:** running `bba-cli` or bba-server to produce reference auctions,
  comparing our auctions against them, and reading `.bbsa` files in order to
  import them.
- Rules should come from bridge literature, published convention
  descriptions, the convention card, and observed bidding.

## Adding a convention

1. Add or find the card setting that turns it on (`crates/bridge-card`).
2. Write a `.bid` module in `conventions/` (syntax: docs/DESIGN.md, "The rule
   language"). Every rule needs an explanation and a `shows` clause.
3. Add test auctions, and check the corpus scoreboard for regressions.

## License

Contributions are dual licensed MIT OR Apache-2.0, as described in README.md.
