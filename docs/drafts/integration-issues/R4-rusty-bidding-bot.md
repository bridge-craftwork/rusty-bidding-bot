Title: Release package for Bridge-Classroom: size and speed budgets, checksums, coverage manifest

Part of the Bridge-Classroom integration plan (issue #3):
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.5, §3.6, §4 item 9, §6).

Bridge-Classroom will pull the WASM package from this repo's GitHub
release assets, pinned by tag and checksum (Q7, provisional default
approved 2026-09-28). This issue makes the release fit for that, and
publishes the evidence Rick needs to gate Rusty per scenario later (Q1).

## Already done (not part of this issue)

- Release workflow: a `v*` tag builds `rbb` for four targets (macOS signed
  and notarized) and attaches `rbb-wasm.tar.gz`; suffixed tags are
  pre-releases; `v0.1.0-rc1` is out (0f6c2b3, fd781b0,
  docs/RELEASING.md).
- CI builds the WASM package on every push (artifact `rbb-wasm-pkg`).
- The size (about 1.4 MB, 430 KB gzipped) and timings (`bidDeal` about
  100–150 ms, one `bid` a few ms) are documented in docs/WASM.md, but not
  checked.
- The native/WASM determinism test moved to R1 (rusty-bidding-bot).
- The stock cards (built-in `.bbsa` names) are done; the freshness check
  below is the one leftover of the dropped "built-in named cards" issue.

## What this issue adds

1. **Budgets in CI.** Measure the gzipped `.wasm` size, `createEngine`,
   and one `bid` and one `auction` in a headless browser (or Node as a
   stand-in, if a browser job is too costly), and fail past agreed
   budgets. Record the budgets in docs/RELEASING.md.
2. **Checksums.** The release attaches `SHA256SUMS` for every asset, and
   the release notes give `info().api`, `version` and `rules_id`, so a
   client can pin tag plus hash and show what it runs.
3. **Coverage manifest.** `rbb compare --manifest out.json` writes, per
   scenario: agreement with BBA, contract agreement, vs-BBA IMPs, card
   coverage, and **the rate of bot calls that would need the BBA
   fallback** (no rule in a live auction, as R1 (rusty-bidding-bot)
   detects it). The release attaches it next to the WASM. Its schema is
   documented, shaped so Bridge-Classroom can use it as it uses PBS's
   `bbaWorks`.
4. **Stock card freshness.** A documented one-command refresh of the
   embedded `.bbsa` cards from Practice-Bidding-Scenarios `bbsa/`, and a
   check (a test when the sibling checkout is present, or a CI job that
   fetches the PBS files) that fails when they differ.

## Acceptance criteria

- [ ] CI fails when the gzipped size or the measured times exceed the
      budgets in docs/RELEASING.md.
- [ ] A release has `SHA256SUMS` covering `rbb-wasm.tar.gz` and the
      binaries, and notes with `api`, `version`, `rules_id`.
- [ ] `rbb compare --manifest` writes the documented schema, including the
      fallback rate per scenario; a release attaches the manifest.
- [ ] The stock-card refresh is one documented command, and a check
      reports a stale embedded card.

Depends on: R1 (rusty-bidding-bot) for the no-rule detection the
manifest counts. Used by: C1 (Bridge-Classroom) (checksums), and later
the per-scenario gating (plan §8, phase 4).
