#!/usr/bin/env bash
#
# Build the web site into web/dist/: the static files, the engine
# (crates/wasm/build.sh into dist/pkg), dealer3's WASM for dealer scripts
# (dist/dealer3) and reference.txt, generated from the engine that ships.
# The output is what the Cloudflare Pages project serves (wrangler.jsonc).
#
#   web/build.sh                 # release build
#   web/build.sh --dev           # faster engine build
#   web/build.sh --no-dealer3    # skip dealer3: the script input says it is missing
#
# dealer3 comes from DEALER3_DIR (a checkout, e.g. ../dealer3) if set, else
# from a clone of bridge-craftwork/Dealer3 at DEALER3_REF into web/.cache/.
# Its build uses its own target directory under web/.cache, so a sibling
# checkout is only read. The single-threaded `web` build: no COOP/COEP
# needed (dealing one deal at a time does not need threads).
#
# Then: python3 -m http.server -d web/dist 8000, and open
# http://localhost:8000/ (WASM does not load from file://).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/.." && pwd)
dist="$here/dist"
cache="$here/.cache"

# The Dealer3 revision the site is built with. Bump it deliberately: the
# run_json envelope (v: 1) is what lib/tool.js speaks.
DEALER3_REPO=${DEALER3_REPO:-https://github.com/bridge-craftwork/Dealer3}
DEALER3_REF=${DEALER3_REF:-e2963c831e630bdb20b16841dc1272fe4463d21c}

dev=
dealer3=1
for a in "$@"; do
    case $a in
        --dev) dev=--dev ;;
        --no-dealer3) dealer3= ;;
        *) echo "build.sh: unknown option $a" >&2; exit 2 ;;
    esac
done

rm -rf "$dist"
mkdir -p "$dist" "$cache"

# The page: copied as it is (no bundler; relative URLs throughout, so it
# works at / and under /rusty-bidding-bot/).
cp "$here"/index.html "$here"/404.html "$here"/_headers "$here"/favicon.svg \
   "$here"/app.js "$here"/app.css "$here"/styles.css "$dist"/
cp -R "$here"/lib "$here"/cards "$dist"/

# The engine.
RBB_WASM_OUT="$dist/pkg" "$root/crates/wasm/build.sh" $dev
rm -f "$dist"/pkg/.gitignore "$dist"/pkg/package.json "$dist"/pkg/README.md "$dist"/pkg/*.d.ts

# dealer3, for dealer scripts.
if [[ -n $dealer3 ]]; then
    src=${DEALER3_DIR:-}
    if [[ -z $src ]]; then
        src="$cache/dealer3"
        if [[ ! -d $src/.git ]]; then
            git clone --quiet --filter=blob:none "$DEALER3_REPO" "$src"
        fi
        git -C "$src" fetch --quiet origin "$DEALER3_REF" 2>/dev/null || true
        git -C "$src" checkout --quiet --detach "$DEALER3_REF"
    fi
    CARGO_TARGET_DIR="$cache/dealer3-target" wasm-pack build "$src/wasm" \
        --release --target web --no-typescript --out-dir "$dist/dealer3" --out-name dealer3_wasm
    rm -f "$dist"/dealer3/.gitignore "$dist"/dealer3/package.json "$dist"/dealer3/README.md
fi

# reference.txt, from the engine just built and the same vocabulary the page uses.
node "$here/scripts/emit-reference.mjs" "$dist"

du -sh "$dist"
