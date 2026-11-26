#!/bin/bash
# The range probe's CPU-heavy robot runs, one after the other: BBA through
# `rbb grid` (bba-cli, the auction forced to the position), then WBridge5
# on the whole deals (probes/tools/wbridge5_bid.py, the Windows VM).
# Run it through the gate: ./cpu-gate.sh probes/range-probe/run_robots.sh [bba|wb5]...
# Brill (network, one request a second) is probes/range-probe/brill_calls.py.
set -euo pipefail
cd "$(dirname "$0")/../.."
D=probes/range-probe/data
what=${*:-bba wb5}
for w in $what; do
  for pos in resp1S resp1H reb1D1S; do
    case $w in
      bba)
        target/release/rbb grid "$D/$pos.toml" --pbs /Volumes/Express2T/Development/GitHub/Practice-Bidding-Scenarios --out ".rbb-cache/grids/range-$pos" > "$D/$pos.bba.txt"
        cp ".rbb-cache/grids/range-$pos/grid.json" "$D/$pos.bba.json"
        ;;
      wb5)
        mkdir -p "$D/wb5"
        python3 "${WB5_TOOL:-/Volumes/Express2T/Development/GitHub/rusty-bidding-bot/probes/tools/wbridge5_bid.py}" -i "$D/$pos.pbn" -o "$D/wb5/$pos.pbn" --tables 3 --resume
        python3 probes/range-probe/wb5_calls.py "$pos" "$D/wb5/$pos.pbn"
        ;;
    esac
  done
done
