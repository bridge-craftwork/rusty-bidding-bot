Title: Native embedding for bridge-table-service: one constructor from card specs, JSON output, version, pin policy

Part of the Bridge-Classroom integration plan (issue #3):
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.1 B, §3.6, §5 item 3).

bridge-table-service will bid Rusty's bot seats natively, as it already
links `bridge-rulebot` (T1 (bridge-table-service)). It needs the engine
with the rules built in, the same card specs and output as the WASM, and a
version to report.

## Already done (not part of this issue)

- **One embedding for native and WASM**: `rbb-assets` compiles the rules,
  manifest, card vocabulary and stock `.bbsa` cards into the binary, with
  `RULES_ID`; `rbb_engine::compile_rules(MANIFEST, FIELDS, BBSA_MAP,
  RULE_FILES)` builds the same rule set as loading `conventions/`
  (c350a6c, 3a67650). The release `rbb` and the WASM both use it, so they
  report the same `rules_id` for the same commit.

## What this issue adds

1. **One native entry point** a service can call without reading the
   disk: build the embedded rule set once, and make an engine from two
   card specs in the WASM's form (stock name, `{bbsa}`, `{json}`),
   returning the load diagnostics. Today the card-spec loader lives in
   `crates/wasm/src/api.rs` (`load_card`); move it where both the WASM
   and native callers use it, keeping the convention layer
   (`bridge-card`, `bidspec`, `conventions/`) free of engine
   dependencies.
2. **The same output natively**: R1 (rusty-bidding-bot)'s `auction` and
   the meaning of a call, available natively with the same serializer as
   the WASM, so the service can put a bot call's explanation, alert and
   announcement on the wire (T1) in the shape the client already reads.
3. **The source commit** in `info()` (WASM) and in the native version
   call, beside `version` and `rules_id`.
4. **Pin policy**, documented: how a downstream service pins this repo
   and `bridge-types` to matching revs so it does not link two copies of
   `Call`, or keeps its boundary in PBN strings.

## Acceptance criteria

- [ ] A test builds an engine natively from `"21GF-DEFAULT"`, a `{bbsa}`
      and a `{json}` card spec with no filesystem access.
- [ ] Native and WASM give byte-identical JSON for the same `auction`
      request (extends R1's determinism test).
- [ ] Native and WASM report the same `version`, `rules_id` and commit for
      the same build.
- [ ] The pin policy is in docs (RELEASING.md or a new section), with the
      `Cargo.toml` lines a service uses.

Depends on: R1 (rusty-bidding-bot). Used by: T1 (bridge-table-service),
T2 (bridge-table-service).
