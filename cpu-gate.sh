#!/bin/bash
#
# cpu-gate.sh - run a command when no other heavy job of ours is running.
#
# Rick (2026-10-05): parallel agents kept all 12 cores busy for hours; keep
# two free for other work. Every build, test, comparison or probe in any
# checkout of this repo (main or an agent's worktree) takes this one gate,
# so at most one runs at a time, and each runs on RBB_CPUS threads (every
# core but two) at a lower priority. Others wait their turn here.
#
#   ./cpu-gate.sh <command> [args...]    # e.g. a Python probe that runs bba-cli
#
# dev-build.sh goes through it by itself. RBB_CPU_GATE=off skips the gate
# (for a quick run while you are sure nothing else is going), RBB_NICE sets
# the priority (default 5), RBB_CPUS the thread budget.

set -euo pipefail

if [[ $# -eq 0 ]]; then
    echo "usage: cpu-gate.sh <command> [args...]" >&2
    exit 2
fi

ncpu=$(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 4)
export RBB_CPUS=${RBB_CPUS:-$((ncpu > 3 ? ncpu - 2 : 1))}
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-$RBB_CPUS}
export RUST_TEST_THREADS=${RUST_TEST_THREADS:-$RBB_CPUS}
export RAYON_NUM_THREADS=${RAYON_NUM_THREADS:-$RBB_CPUS}

# Already inside the gate (a gated script calling dev-build.sh), or asked
# to skip it: just run.
if [[ -n ${RBB_CPU_GATE_HELD:-} || ${RBB_CPU_GATE:-on} == off ]]; then
    exec "$@"
fi

gate_dir=${RBB_CPU_GATE_DIR:-$HOME/.cache/rbb}
mkdir -p "$gate_dir"
gate=$gate_dir/cpu.gate
export RBB_CPU_GATE_HELD=1

if ! lockf -s -k -t 0 "$gate" true; then
    echo "cpu-gate: waiting for the CPU; held by: $(cat "$gate.who" 2>/dev/null || echo '?')" >&2
fi
# The holder writes who it is (pid, directory, command) for those waiting.
exec nice -n "${RBB_NICE:-5}" lockf -k "$gate" \
    /bin/sh -c 'echo "$$ $(date +%H:%M:%S) $PWD: $*" > "$0.who"; exec "$@"' "$gate" "$@"
