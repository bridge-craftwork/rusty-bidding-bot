#!/usr/bin/env python3
"""Self A/B of a convention BBA does not play (Rick, 2026-10-05: "for
conventions BBA doesn't have, we should compare against ourself, to see
if the convention aids or not"): the same deals bid twice by our engine,
once with the base card changes and once with the variant's, judged by
the errors yardstick (errors_diff.py: each side charged with its own
errors, IMPs).

One convention:

    probes/tools/self_ab.py --name kickback \\
        --scenarios Minor_Game_Or_Slam Slam_After_Major_Fit \\
        --base slam.kickback.play=false --variant slam.kickback.play=true

A sweep (one line per convention at the end):

    probes/tools/self_ab.py --batch probes/self-ab.toml [--only a,b] [--reuse]

where the TOML has one table per convention:

    [[ab]]
    name      = "kickback"
    scenarios = ["Minor_Game_Or_Slam", "Slam_After_Major_Fit"]
    base      = ["slam.kickback.play=false"]
    variant   = ["slam.kickback.play=true"]
    side      = "ns"      # optional: "ns" or "ew" plays the variant alone
                          # (compare --ns-set / --ew-set); default both

`side` matters for competitive conventions: with `--set` both pairs
play the convention, so a board can change because the opponents used
it. The actor split (the side that made the first differing call)
attributes most of that, but the other pair's errors then mix their
reply to the convention with their own use of it. With `side = "ns"`
only one pair has it, which is the "practice my card" question.

Each run is `compare` on every board of the scenarios (500 each; IMP
verdicts need them), through ./cpu-gate.sh, with --par, writing
`<out>/<name>.base.json` and `<out>/<name>.variant.json` (default out:
.rbb-cache/self-ab). `--reuse` keeps JSON files that are already there.

The verdict line: boards changed; the change in the actor's errors
(contract, doubling; positive: the variant makes fewer) and the other
side's; double-dummy IMPs to the actor; whether the even and odd board
halves agree in sign on the actor's errors; z, the actor's total over
its standard error; and the verdict: gains / loses when the halves
agree, the total is 3 IMPs or more and |z| >= 2, "leans" for
1 <= |z| < 2, else neutral.
"""
import argparse
import json
import os
import subprocess
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
sys.path.insert(0, HERE)
import errors_diff  # noqa: E402

PBS = os.path.abspath(os.path.join(ROOT, '..', 'Practice-Bidding-Scenarios'))
if not os.path.isdir(PBS):
    # A worktree under .claude/worktrees: the sibling checkouts are beside
    # the main checkout.
    PBS = '/Volumes/Express2T/Development/GitHub/Practice-Bidding-Scenarios'


def set_args(changes, side):
    flag = {'ns': '--ns-set', 'ew': '--ew-set'}.get(side or 'both', '--set')
    out = []
    for c in changes:
        out += [flag, c]
    return out


def run_compare(scenarios, changes, side, out, a):
    if a.reuse and os.path.exists(out):
        return
    cmd = [os.path.join(ROOT, 'cpu-gate.sh'), os.path.join(ROOT, 'target/release/rbb'),
           'compare', *scenarios, '--pbs', a.pbs, '--rules', os.path.join(ROOT, 'conventions'),
           '--par', '--dd-cache', a.dd_cache, '--json', out, *set_args(changes, side)]
    print('$', ' '.join(cmd), file=sys.stderr)
    r = subprocess.run(cmd, cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    if r.returncode != 0:
        sys.exit(f'compare failed:\n{r.stderr[-2000:]}')


def verdict(name, spec, base_path, var_path, a):
    base = json.load(open(base_path))['boards']
    var = json.load(open(var_path))['boards']
    cells, n, skipped, by, cnt, changed = errors_diff.diff(base, var)
    N = n[0] + n[1]
    tot = lambda k: sum(cells[k])
    act = [cells['contract actor'][h] + cells['doubling actor'][h] for h in (0, 1)]
    a_tot = act[0] + act[1]
    agree = (act[0] > 0 and act[1] > 0) or (act[0] < 0 and act[1] < 0)
    # z: the actor's total over its standard error (board swings taken
    # as independent).
    g = [c[2] for c in changed]
    sd = (sum(x * x for x in g) / len(g) - (sum(g) / len(g)) ** 2) ** 0.5 if g else 0
    z = a_tot / (sd * len(g) ** 0.5) if sd else 0.0
    if N == 0 or abs(a_tot) < 3 or not agree or abs(z) < 1:
        word = 'neutral'
    elif abs(z) < 2:
        word = 'leans ' + ('gain' if a_tot > 0 else 'loss')
    else:
        word = 'gains' if a_tot > 0 else 'loses'
    side = spec.get('side', 'both')
    line = (f"{name:24} {','.join(spec['scenarios'])[:60]:60} side={side:4} "
            f"changed {N:4} (skip {skipped})  actor: contract {tot('contract actor'):+5} "
            f"doubling {tot('doubling actor'):+5}  other: contract {tot('contract other'):+5} "
            f"doubling {tot('doubling other'):+5}  DD-to-actor {tot('side IMPs (actor)'):+5}  "
            f"halves {act[0]:+}/{act[1]:+} {'agree' if agree else 'differ'}  z {z:+.1f}  {word}")
    worst = sorted(changed, key=lambda z: z[2])[:a.worst]
    return line, worst


def one(name, spec, a):
    os.makedirs(a.out, exist_ok=True)
    bp = os.path.join(a.out, f'{name}.base.json')
    vp = os.path.join(a.out, f'{name}.variant.json')
    side = spec.get('side')
    # The base gets the side treatment too, so the two runs differ only
    # in the convention.
    run_compare(spec['scenarios'], spec.get('base', []), side, bp, a)
    run_compare(spec['scenarios'], spec.get('variant', []), side, vp, a)
    print(f'== {name}: base {spec.get("base", [])} -> variant {spec.get("variant", [])} '
          f'side {side or "both"}', flush=True)
    sys.stdout.flush()
    subprocess.run([sys.executable, os.path.join(HERE, 'errors_diff.py'), bp, vp, '--top', str(a.top)])
    line, worst = verdict(name, spec, bp, vp, a)
    if worst:
        print('worst boards for the actor (scenario board actor IMPs side : first difference):')
        for s, b, g, sd, k in worst:
            if g < 0:
                print(f'  {s} {b} {g:+} {sd} : {k}')
    print(line, flush=True)
    return line


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--batch', help='TOML with [[ab]] tables')
    ap.add_argument('--only', help='comma-separated names from the batch')
    ap.add_argument('--name', default='ab')
    ap.add_argument('--scenarios', nargs='*', default=[])
    ap.add_argument('--base', nargs='*', default=[])
    ap.add_argument('--variant', nargs='*', default=[])
    ap.add_argument('--side', choices=['ns', 'ew', 'both'])
    ap.add_argument('--out', default=os.path.join(ROOT, '.rbb-cache', 'self-ab'))
    ap.add_argument('--pbs', default=PBS)
    ap.add_argument('--dd-cache', default=os.path.join(ROOT, '.rbb-cache', 'dd.jsonl'))
    ap.add_argument('--reuse', action='store_true', help='keep JSON files already written')
    ap.add_argument('--top', type=int, default=4)
    ap.add_argument('--worst', type=int, default=5)
    a = ap.parse_args()
    if a.batch:
        specs = tomllib.load(open(a.batch, 'rb'))['ab']
        if a.only:
            keep = set(a.only.split(','))
            specs = [s for s in specs if s['name'] in keep]
        if a.side:
            # The whole sweep with one pair playing the variant (or both).
            specs = [{**s, 'side': a.side, 'name': f"{s['name']}.{a.side}"} for s in specs]
    else:
        specs = [{'name': a.name, 'scenarios': a.scenarios, 'base': a.base,
                  'variant': a.variant, **({'side': a.side} if a.side else {})}]
    lines = [one(s['name'], s, a) for s in specs]
    print('\n== verdicts (actor errors: positive = the convention makes fewer)')
    for line in lines:
        print(line)


if __name__ == '__main__':
    main()
