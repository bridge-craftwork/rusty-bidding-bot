#!/usr/bin/env python3
"""The convention score of a convention BBA plays (Rick, 2026-10-07):
"if we bid the same hands without Bergen, on BBA and Rusty, then again
with it, it seems like that should be the reported delta — ignoring
hands where neither used Bergen."

On each scenario's deals, four auctions per board: BBA and our engine,
each with North-South playing the convention (on) and not (off); East-
West always plays the scenario's own card. Each table is scored against
double-dummy par by the errors yardstick (`par::table_errors`: contract
and doubling errors, IMPs, each side charged with its own), for the side
that used the convention (North-South) with the other side beside it.

    Rusty's gain     = errors(ours, off)  - errors(ours, on)
    BBA's gain       = errors(BBA, off)   - errors(BBA, on)
    convention score = Rusty's gain - BBA's gain

summed over the changed boards: those where either engine's auction
differs between on and off (a board where neither changed adds nothing
to either gain). Positive: the convention does more for us than for BBA.

    probes/tools/conv_ab.py [--batch probes/conv-ab.toml] [--only a,b] [--reuse]

How a run is made, per convention and scenario:

1. The NS card is the one the corpus PBN's `% CC1` header names (as
   `compare` reads it). `rbb card import-bbsa` reads it; the `on`
   changes give the on card, `on` + `off` the off card; `rbb card
   export-bbsa` writes both. The .bbsa keys that differ between the two
   are the switch BBA sees: none means BBA cannot be switched (no .bbsa
   key maps the field) and the convention is reported unmeasured.
2. bba-cli bids the corpus deals with each card for NS (the scenario's
   EW card, its scoring), into a Practice-Bidding-Scenarios-shaped
   directory `<out>/<name>/{on,off}/` (`bba/<scenario>.pbn`, `bbsa/`), so
   the double-dummy tables of the corpus carry over and `compare --pbs`
   reads it as it reads the corpus. With no `on` changes the on run must
   reproduce the corpus: the boards that do not are counted (`repro`).
3. `rbb compare --pbs <dir> --par-all --ns-set ...` replays each run:
   our engine bids NS with the same card (plus the same changes, so a
   field BBA has no key for is still switched for us), every board
   scored against par.

The line per convention: changed boards (ours / BBA / either), Rusty's
gain, BBA's gain, the net per changed board, the even and odd halves of
the net, the other side's net beside it, and the no-rule positions our
off run hit (a convention our rules cannot do without shows up there:
`--max-no-rule` per 100 boards marks it unmeasurable). JSON per
convention in `<out>/<name>.result.json`, all in `<out>/results.json`
(probes/tools/status_data.py reads it).

The batch file has one `[[conv]]` per convention:

    [[conv]]
    name      = "bergen"
    tile      = "bidding_conventions/bergen_raises"   # the status tile
    scenarios = ["Bergen_Raises"]
    off       = ["major_openings.bergen_raises.play=false"]
    on        = []          # optional: changes both runs share
    skip      = "..."       # optional: why it is not measured this way
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
PBS = os.path.abspath(os.path.join(ROOT, '..', 'Practice-Bidding-Scenarios'))
if not os.path.isdir(PBS):
    PBS = '/Volumes/Express2T/Development/GitHub/Practice-Bidding-Scenarios'
BBA_CLI = '/Applications/Bridge Utilities/bba-cli'

# The convention score's colours (status_data.py reads them from here):
# IMPs per changed board, Rusty's gain minus BBA's. Good: the convention
# does at least about as much for us as for BBA (within a tenth of an
# IMP a board it touches). Poor: we lose half an IMP or more a touched
# board to BBA's use of it, and both halves of the boards say so. Fewer
# than MIN_CHANGED changed boards is too few to judge.
THRESHOLDS = {'good': -0.1, 'poor': -0.5, 'min_changed': 10}


def rbb(a):
    return os.path.join(ROOT, 'target', 'release', 'rbb') if not a.rbb else a.rbb


def gate(cmd):
    return [os.path.join(ROOT, 'cpu-gate.sh'), *cmd]


def header_cards(pbn):
    """(NS card, EW card, scoring) from a corpus PBN's header."""
    text = open(pbn, errors='replace').read(4000)
    name = lambda p: re.split(r'[\\/]', p.strip())[-1].removesuffix('.bbsa')
    ns = re.search(r'% CC1 - (.*)', text)
    ew = re.search(r'% CC2 - (.*)', text)
    sc = re.search(r'\[Scoring "(\w+)"\]', text)
    if not (ns and ew):
        return None
    return name(ns.group(1)), name(ew.group(1)), (sc.group(1) if sc else 'MP')


def parse_value(v):
    if v in ('true', 'false'):
        return v == 'true'
    try:
        return int(v)
    except ValueError:
        return v


def get_path(card, path):
    cur = card
    for k in path.split('.'):
        if not isinstance(cur, dict) or k not in cur:
            return None
        cur = cur[k]
    return cur


def set_path(card, path, value):
    """Set `path` in the nested card JSON. A field's key may itself hold
    dots (`vs_1nt_strong.system` is nested, `two_nt.puppet` too): the
    card JSON nests at every dot."""
    cur = card
    ks = path.split('.')
    for k in ks[:-1]:
        cur = cur.setdefault(k, {})
    cur[ks[-1]] = value


def bbsa_keys(path):
    d = {}
    for line in open(path, errors='replace').read().replace('\r', '').splitlines():
        if '=' in line:
            k, v = line.rsplit('=', 1)
            d[k.strip()] = v.strip()
    return d


def auctions(pbn):
    """{board: auction text} of a PBN, notes and alerts stripped."""
    text = open(pbn, errors='replace').read().replace('\r', '')
    out = {}
    for blk in text.split('\n\n'):
        m = re.search(r'\[Board "(\d+)"\]', blk)
        a = re.search(r'\[Auction "\w"\]\n(.*?)(?=\n\[|$)', blk, re.S)
        if m and a:
            out[m.group(1)] = ' '.join(t for t in a.group(1).split() if not t.startswith(('=', '$')))
    return out


def make_cards(spec, ns_card, d, a):
    """Write the on and off NS cards into d/{on,off}/bbsa; return
    (on name, off name, BBA keys switched, on value per off field)."""
    src = os.path.join(PBS, 'bbsa', ns_card + '.bbsa')
    r = subprocess.run([rbb(a), 'card', 'import-bbsa', src], cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        sys.exit(f'import-bbsa {src}: {r.stderr}')
    card = json.loads(r.stdout)
    for c in spec.get('on', []):
        k, v = c.split('=', 1)
        set_path(card, k, parse_value(v))
    before = {c.split('=', 1)[0]: get_path(card, c.split('=', 1)[0]) for c in spec['off']}
    off = json.loads(json.dumps(card))
    for c in spec['off']:
        k, v = c.split('=', 1)
        set_path(off, k, parse_value(v))
    names = {}
    for which, cj in (('on', card), ('off', off)):
        name = f"{ns_card}__{spec['name']}-{which}"
        os.makedirs(os.path.join(d, which, 'bbsa'), exist_ok=True)
        jp = os.path.join(d, which, 'bbsa', name + '.json')
        json.dump(cj, open(jp, 'w'), indent=1)
        out = os.path.join(d, which, 'bbsa', name + '.bbsa')
        r = subprocess.run([rbb(a), 'card', 'export-bbsa', jp, '-o', out], cwd=ROOT, capture_output=True, text=True)
        if r.returncode:
            sys.exit(f'export-bbsa {jp}: {r.stderr}')
        names[which] = name
    ka = bbsa_keys(os.path.join(d, 'on', 'bbsa', names['on'] + '.bbsa'))
    kb = bbsa_keys(os.path.join(d, 'off', 'bbsa', names['off'] + '.bbsa'))
    switched = {k: [ka.get(k), kb.get(k)] for k in sorted(set(ka) | set(kb)) if ka.get(k) != kb.get(k)}
    return names['on'], names['off'], switched, before


def bba_run(scen, ns_name, ew_card, scoring, which_dir, a):
    out = os.path.join(which_dir, 'bba', scen + '.pbn')
    if a.reuse and os.path.exists(out):
        return out
    os.makedirs(os.path.dirname(out), exist_ok=True)
    ew_src = os.path.join(PBS, 'bbsa', ew_card + '.bbsa')
    ew_dst = os.path.join(which_dir, 'bbsa', ew_card + '.bbsa')
    if not os.path.exists(ew_dst):
        shutil.copyfile(ew_src, ew_dst)
    cmd = gate([BBA_CLI, '-i', os.path.join(PBS, 'bba', scen + '.pbn'), '-o', out,
                '--ns-conventions', os.path.join(which_dir, 'bbsa', ns_name + '.bbsa'),
                '--ew-conventions', ew_dst, '--event', scen, '--scoring', scoring])
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        sys.exit(f'bba-cli {scen}: {r.stderr[-2000:]}')
    return out


def compare_run(scens, which_dir, changes, out, a):
    if a.reuse and os.path.exists(out):
        return
    cmd = gate([rbb(a), 'compare', *scens, '--pbs', which_dir, '--rules', os.path.join(ROOT, 'conventions'),
                '--par-all', '--dd-cache', a.dd_cache, '--json', out])
    for c in changes:
        cmd += ['--ns-set', c]
    r = subprocess.run(cmd, cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    if r.returncode:
        sys.exit(f'compare {scens}: {r.stderr[-2000:]}')


def calls(b, key):
    def one(c):
        if isinstance(c, str):
            return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}.get(c, c)
        s = c['Bid']['strain']
        return f"{c['Bid']['level']}{'NT' if s == 'NoTrump' else s[0]}"
    return [one(c) for c in b[key]]


def side_err(e, side):
    return e['contract'][side] + e['doubling'][side]


def score(on_json, off_json):
    """The four-way comparison of two compare runs on the same deals."""
    key = lambda b: (b['scenario'], b['board'])
    on = {key(b): b for b in json.load(open(on_json))['boards']}
    offr = json.load(open(off_json))
    off = {key(b): b for b in offr['boards']}
    onr = json.load(open(on_json))
    s = {'boards': 0, 'changed': 0, 'changed_ours': 0, 'changed_bba': 0, 'changed_both': 0, 'unscored': 0,
         'rusty_gain': 0, 'bba_gain': 0, 'rusty_gain_other': 0, 'bba_gain_other': 0,
         'net_halves': [0, 0], 'rusty_contract': 0, 'rusty_doubling': 0, 'bba_contract': 0, 'bba_doubling': 0}
    swings = []
    for k, b_on in on.items():
        b_off = off.get(k)
        if not b_off:
            continue
        s['boards'] += 1
        rc = calls(b_on, 'ours') != calls(b_off, 'ours')
        bc = calls(b_on, 'reference') != calls(b_off, 'reference')
        if not (rc or bc):
            continue
        p_on, p_off = b_on.get('par'), b_off.get('par')
        if not p_on or not p_off:
            s['unscored'] += 1
            continue
        s['changed'] += 1
        s['changed_ours'] += rc
        s['changed_bba'] += bc
        s['changed_both'] += rc and bc
        g = {}
        for who, fld in (('rusty', 'ours_errors'), ('bba', 'reference_errors')):
            eon, eoff = p_on[fld], p_off[fld]
            g[who] = side_err(eoff, 0) - side_err(eon, 0)
            s[f'{who}_gain'] += g[who]
            s[f'{who}_gain_other'] += side_err(eoff, 1) - side_err(eon, 1)
            s[f'{who}_contract'] += eoff['contract'][0] - eon['contract'][0]
            s[f'{who}_doubling'] += eoff['doubling'][0] - eon['doubling'][0]
        net = g['rusty'] - g['bba']
        try:
            h = int(k[1]) % 2
        except ValueError:
            h = 0
        s['net_halves'][h] += net
        swings.append((net, k[0], k[1], ' '.join(calls(b_on, 'ours')), ' '.join(calls(b_off, 'ours')),
                       ' '.join(calls(b_on, 'reference')), ' '.join(calls(b_off, 'reference'))))
    s['net'] = s['rusty_gain'] - s['bba_gain']
    s['net_other'] = s['rusty_gain_other'] - s['bba_gain_other']
    n = s['changed']
    s['net_per_changed'] = round(s['net'] / n, 3) if n else None
    s['rusty_per_changed'] = round(s['rusty_gain'] / n, 3) if n else None
    s['bba_per_changed'] = round(s['bba_gain'] / n, 3) if n else None
    e, o = s['net_halves']
    s['halves_agree'] = (e > 0 and o > 0) or (e < 0 and o < 0) or (e == 0 and o == 0)
    s['no_rule_on'] = onr['summary']['total']['problems'].get('no_rule', 0)
    s['no_rule_off'] = offr['summary']['total']['problems'].get('no_rule', 0)
    s['worst'] = [list(x) for x in sorted(swings)[:5]]
    s['best'] = [list(x) for x in sorted(swings, reverse=True)[:3]]
    return s


def status_of(s):
    t = THRESHOLDS
    if not s or s['changed'] < t['min_changed']:
        return None
    x = s['net_per_changed']
    if x >= t['good']:
        return 'good'
    e, o = s['net_halves']
    if x <= t['poor'] and e < 0 and o < 0:
        return 'poor'
    return 'fair'


def one(spec, a):
    name = spec['name']
    d = os.path.join(a.out, name)
    res = {'name': name, 'tile': spec.get('tile'), 'scenarios': spec['scenarios'], 'off': spec['off'],
           'on': spec.get('on', []), 'skip': spec.get('skip')}
    if spec.get('skip'):
        res['unmeasured'] = spec['skip']
        return res
    os.makedirs(d, exist_ok=True)
    used, notes, switched_all = [], [], {}
    repro = [0, 0]
    for scen in spec['scenarios']:
        pbn = os.path.join(PBS, 'bba', scen + '.pbn')
        if not os.path.exists(pbn):
            notes.append(f'{scen}: no corpus file')
            continue
        hc = header_cards(pbn)
        if not hc:
            notes.append(f'{scen}: no CC1/CC2 header')
            continue
        ns, ew, scoring = hc
        on_name, off_name, switched, before = make_cards(spec, ns, d, a)
        offv = {c.split('=', 1)[0]: parse_value(c.split('=', 1)[1]) for c in spec['off']}
        if all(before[k] == v or (before[k] is None and v is False) for k, v in offv.items()):
            notes.append(f'{scen}: {ns} does not play it or leaves it unset ({", ".join(f"{k}={before[k]}" for k in offv)})')
            continue
        if not switched:
            notes.append(f'{scen}: no .bbsa key changes ({", ".join(spec["off"])}): BBA cannot be switched')
            continue
        switched_all.update(switched)
        on_pbn = bba_run(scen, on_name, ew, scoring, os.path.join(d, 'on'), a)
        bba_run(scen, off_name, ew, scoring, os.path.join(d, 'off'), a)
        if not spec.get('on'):
            A, B = auctions(pbn), auctions(on_pbn)
            repro[0] += sum(1 for k in A if A[k] != B.get(k))
            repro[1] += len(A)
        used.append(scen)
    res.update({'used': used, 'notes': notes, 'bba_keys': switched_all,
                'repro': {'differ': repro[0], 'boards': repro[1]} if repro[1] else None})
    if not used:
        res['unmeasured'] = '; '.join(notes) or 'no scenario'
        return res
    on_json, off_json = os.path.join(d, 'on.json'), os.path.join(d, 'off.json')
    compare_run(used, os.path.join(d, 'on'), spec.get('on', []), on_json, a)
    compare_run(used, os.path.join(d, 'off'), spec.get('on', []) + spec['off'], off_json, a)
    s = score(on_json, off_json)
    res['score'] = s
    nr = (s['no_rule_off'] - s['no_rule_on']) / max(1, s['boards']) * 100
    if nr > a.max_no_rule:
        res['unmeasured'] = (f'our off run hits {s["no_rule_off"]} no-rule positions ({s["no_rule_on"]} on): '
                             f'our rules have no fallback without it')
    # One engine that ignores the switch makes the score that engine's
    # gain against nothing: not a comparison.
    elif s['changed_ours'] < THRESHOLDS['min_changed']:
        res['unmeasured'] = (f'our auctions changed on {s["changed_ours"]} boards: our rules do not switch it off '
                             f'with {", ".join(spec["off"])}')
    elif s['changed_bba'] < THRESHOLDS['min_changed']:
        res['unmeasured'] = (f"BBA's auctions changed on {s['changed_bba']} boards: its key "
                             f"({', '.join(res['bba_keys'])}) does not switch it")
    res['status'] = None if res.get('unmeasured') else status_of(s)
    if s['changed'] < THRESHOLDS['min_changed'] and not res.get('unmeasured'):
        res['unmeasured'] = f'only {s["changed"]} boards changed'
    return res


def line(r):
    if r.get('unmeasured') and not r.get('score'):
        return f"{r['name']:22} unmeasured: {r['unmeasured']}"
    s = r['score']
    pc = lambda x: f'{x:+.2f}' if x is not None else '  -  '
    e, o = s['net_halves']
    return (f"{r['name']:22} {','.join(r['used'])[:44]:44} changed {s['changed']:4} "
            f"(ours {s['changed_ours']}, BBA {s['changed_bba']})  Rusty {s['rusty_gain']:+5} BBA {s['bba_gain']:+5} "
            f"net {s['net']:+5} = {pc(s['net_per_changed'])}/bd  halves {e:+}/{o:+} "
            f"{'agree' if s['halves_agree'] else 'differ'}  other {s['net_other']:+}  "
            f"no-rule {s['no_rule_on']}->{s['no_rule_off']}  {r.get('status') or 'unmeasured: ' + r.get('unmeasured', '')}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--batch', default=os.path.join(ROOT, 'probes', 'conv-ab.toml'))
    ap.add_argument('--only', help='comma-separated names from the batch')
    ap.add_argument('--out', default=os.path.join(ROOT, '.rbb-cache', 'conv-ab'))
    ap.add_argument('--dd-cache', default=os.path.join(ROOT, '.rbb-cache', 'dd.jsonl'))
    ap.add_argument('--rbb')
    ap.add_argument('--reuse', action='store_true', help='keep PBN and JSON files already written')
    ap.add_argument('--max-no-rule', type=float, default=2.0,
                    help='extra no-rule positions per 100 boards in our off run that make it unmeasurable')
    a = ap.parse_args()
    specs = tomllib.load(open(a.batch, 'rb'))['conv']
    if a.only:
        keep = set(a.only.split(','))
        specs = [s for s in specs if s['name'] in keep]
    os.makedirs(a.out, exist_ok=True)
    allp = os.path.join(a.out, 'results.json')
    results = json.load(open(allp)).get('results', {}) if os.path.exists(allp) else {}
    for spec in specs:
        r = one(spec, a)
        results[spec['name']] = r
        json.dump(r, open(os.path.join(a.out, spec['name'] + '.result.json'), 'w'), indent=1)
        print(line(r), flush=True)
        if r.get('score'):
            for w in r['score']['worst'][:3]:
                if w[0] < 0:
                    print(f'    {w[0]:+} {w[1]} {w[2]}: ours {w[3]} / off {w[4]} | BBA {w[5]} / off {w[6]}')
    # Only the batch's conventions stay in the results.
    names = {s['name'] for s in tomllib.load(open(a.batch, 'rb'))['conv']}
    results = {k: v for k, v in results.items() if k in names}
    json.dump({'thresholds': THRESHOLDS, 'results': results}, open(allp, 'w'), indent=1)
    print(f'\n== {allp}')
    for r in results.values():
        print(line(r))


if __name__ == '__main__':
    main()
