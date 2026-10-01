#!/usr/bin/env bash
#
# Build the browser package: crates/wasm/pkg/ (rbb_wasm.js, rbb_wasm_bg.wasm,
# .d.ts and package.json), an ES module for `<script type="module">` or a
# bundler. The rules in conventions/ and the stock cards are compiled in.
#
#   crates/wasm/build.sh            # release build
#   crates/wasm/build.sh --dev      # faster build, bigger and slower wasm
#   RBB_WASM_OUT=/path crates/wasm/build.sh   # write the package elsewhere
#
# Needs: rustup target add wasm32-unknown-unknown; cargo install wasm-pack.
# Then serve crates/wasm/ over HTTP (e.g. `python3 -m http.server -d crates/wasm`)
# and open http://localhost:8000/demo.html: wasm will not load from file://.
#
# Same tool and flags as bridge-rulebot/wasm (wasm-pack --target web). This
# crate is a workspace member, so it builds against the committed Cargo.lock;
# the local-dev [patch] pattern (bridge-rulebot's dev-build.sh) is not needed
# here, and nothing local leaks into CI.
set -euo pipefail
cd "$(dirname "$0")"

profile=--release
if [[ ${1:-} == --dev ]]; then
    profile=--dev
fi
out=${RBB_WASM_OUT:-pkg}

command -v wasm-pack >/dev/null || {
    echo "build.sh: wasm-pack not found: cargo install wasm-pack" >&2
    exit 1
}
rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown || {
    echo "build.sh: installing the wasm32-unknown-unknown target" >&2
    rustup target add wasm32-unknown-unknown
}

# wasm-pack runs cargo itself, so with the gitignored [patch] overrides in
# .cargo/config.toml (local sibling checkouts; dev-build.sh) cargo would
# rewrite the committed Cargo.lock with local-path entries. Keep the lock
# as it was before the build.
lock=../../Cargo.lock
saved=$(mktemp)
cp "$lock" "$saved"
trap 'cp "$saved" "$lock"; rm -f "$saved"' EXIT

wasm-pack build --target web "$profile" --out-dir "$out" --out-name rbb_wasm
ls -l "$out"/rbb_wasm_bg.wasm
