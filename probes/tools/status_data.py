#!/usr/bin/env python3
"""Write web/status/status.json, the data of the status page
(web/status.html; docs/WEB.md, "Status page"): where each part of the
system stands, tile by tile.

    target/release/rbb compare --json /tmp/all.json        # every scenario
    probes/tools/self_ab.py --batch probes/self-ab.toml     # .rbb-cache/self-ab
    probes/tools/conv_ab.py                                 # .rbb-cache/conv-ab
    probes/tools/status_data.py --compare /tmp/all.json

Sources:

- the taxonomy: convention-card's spec (conventions.json, fields.toml)
  at the revision Cargo.lock pins (cargo's checkout of it, else
  `git show` in ../convention-card, else raw.githubusercontent.com), so
  the page never runs ahead of the engine's card vocabulary;
- the modules: `rbb bid skills --json` (skills, fields read, rules), the
  `.test` cases and `.notes.md` files beside them;
- probes/status-items.toml: groups (Reverse Drury under Drury), the
  scenarios that measure each tile, partial and extra treatments, and
  the judgment and catch-all tiles;
- performance against BBA: a full `compare --json` (per scenario: calls
  agreeing, errors vs BBA per board), with each scenario's NS card read
  through `rbb card import-bbsa` to see which treatments it plays;
- self A/B: the JSON pairs self_ab.py writes, judged by its own verdict;
- the convention score of a convention BBA plays (Rick, 2026-10-07):
  conv_ab.py's results.json, Rusty's gain from the convention minus
  BBA's on the boards it changed. The tile's colour is that score; the
  scenario score on the same deals ("background") rides beside it as a
  second marker. A tile with no convention score keeps its scenario
  score as its colour, "background (not isolated)".

Status codes (the page's legend says the same):
  gap       not implemented: no module, or no rule reads the field
  tested    implemented, measured only by its .test cases
  partial   implemented in part (status-items.toml says what is missing)
  conv-good / conv-fair / conv-poor  the convention score (conv_ab.THRESHOLDS)
  bba-good / bba-fair / bba-poor   the background: scenarios scored against
                                   BBA (THRESHOLDS)
  ab-gains / ab-neutral / ab-loses self A/B against ourselves
"""
import argparse
import datetime
import glob
import json
import os
import re
import subprocess
import sys
import tomllib
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
sys.path.insert(0, HERE)
import conv_ab  # noqa: E402

REPO_URL = 'https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/'
PBS = os.path.abspath(os.path.join(ROOT, '..', 'Practice-Bidding-Scenarios'))
if not os.path.isdir(PBS):
    PBS = '/Volumes/Express2T/Development/GitHub/Practice-Bidding-Scenarios'

# Against BBA, per tile (its scenarios' boards pooled), par as the
# yardstick ("par decides"): errors vs BBA in IMPs per board (each side
# charged with its own errors; positive: ours fewer) and the NS calls
# agreeing with BBA.
THRESHOLDS = {
    'good': {'errors_per_board': -0.25, 'ns_agree': 0.65},
    'poor': {'errors_per_board': -1.0, 'ns_agree': 0.55},
    # Judgment tiles: our boards in the par class over BBA's.
    'class_ratio': {'good': 1.0, 'fair': 1.25},
    # Catch-alls: per 1,000 boards.
    'per_thousand': {'good': 1.0, 'fair': 5.0},
}

SECTIONS = [
    ('basic', 'Basic bridge'),
    ('judgment', 'Judgment'),
    ('catchall', 'Catch-alls'),
    ('constructive', 'Conventions: constructive'),
    ('competitive', 'Conventions: competitive'),
    ('precision', 'Precision'),
]
CATEGORY_SECTION = {
    'basic_bidding': 'basic',
    'bidding_conventions': 'constructive',
    'competitive_bidding': 'competitive',
    'precision': 'precision',
}
SKIP_OPTIONS = {'none', 'other'}


def sh(*cmd, cwd=ROOT):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=True).stdout


# ── The pinned spec ───────────────────────────────────────────────────

def pinned_card_rev():
    lock = open(os.path.join(ROOT, 'Cargo.lock')).read()
    m = re.search(r'name = "bridge-card"\nversion = "([^"]+)"\nsource = "git\+([^?]+)\?tag=([^#]+)#([0-9a-f]+)"', lock)
    if not m:
        sys.exit('Cargo.lock: no git-pinned bridge-card')
    return {'version': m.group(1), 'repo': m.group(2), 'tag': m.group(3), 'commit': m.group(4)}


def spec_reader(pin, override):
    """A function rel -> text for spec files at the pinned commit."""
    if override:
        return lambda rel: open(os.path.join(override, rel)).read()
    for d in glob.glob(os.path.expanduser('~/.cargo/git/checkouts/convention-card-*/' + pin['commit'][:7])):
        if os.path.isdir(os.path.join(d, 'spec')):
            return lambda rel, d=d: open(os.path.join(d, 'spec', rel)).read()
    sib = os.path.abspath(os.path.join(ROOT, '..', 'convention-card'))
    if not os.path.isdir(sib):
        sib = '/Volumes/Express2T/Development/GitHub/convention-card'
    if os.path.isdir(sib):
        try:
            sh('git', 'cat-file', '-e', pin['commit'], cwd=sib)
            return lambda rel: sh('git', 'show', f"{pin['commit']}:spec/{rel}", cwd=sib)
        except subprocess.CalledProcessError:
            pass
    base = pin['repo'].replace('https://github.com/', 'https://raw.githubusercontent.com/')
    return lambda rel: urllib.request.urlopen(f"{base}/{pin['commit']}/spec/{rel}").read().decode()


def load_fields(text):
    """Every field of fields.toml as {path: attrs} in declaration order."""
    data = tomllib.loads(text)
    out = {}
    for section, entries in data.items():
        for key, attrs in entries.items():
            if isinstance(attrs, dict) and 'kind' in attrs:
                out[f'{section}.{key}'] = attrs
    return out


def skills_of(attrs):
    s = attrs.get('skill')
    return [] if s is None else ([s] if isinstance(s, str) else list(s))


# ── Modules ───────────────────────────────────────────────────────────

def count_tests(path):
    if not os.path.exists(path):
        return 0
    n = 0
    for line in open(path):
        line = line.strip()
        if line and not line.startswith('#') and line.count('|') >= 2:
            n += 1
    return n


def load_modules(rbb):
    inv = json.loads(sh(rbb, 'bid', 'skills', '--json', '--rules', 'conventions'))
    mods = {}
    for m in inv['modules']:
        stem = m['file'][:-4]
        src = open(os.path.join(ROOT, m['file'])).read()
        mods[m['name']] = {
            'name': m['name'],
            'title': m.get('title'),
            'file': m['file'],
            'url': REPO_URL + m['file'],
            'skills': m['skills'],
            'reads': m['reads'],
            'rules': m['rules'],
            'tests': count_tests(stem + '.test'),
            'notes': (REPO_URL + stem + '.notes.md') if os.path.exists(os.path.join(ROOT, stem + '.notes.md')) else None,
            'source': src,
        }
    return mods, set(inv['fields_read'])


def option_read(field, option, attrs, mods):
    """Whether a rule plays this option of an enum field: the default
    counts when the field is read; another option when a module reading
    the field names it (`style is four_way`)."""
    readers = [m for m in mods.values() if field in m['reads']]
    if not readers:
        return False
    if attrs.get('default') == option:
        return True
    pat = re.compile(r'(?<![\w.])' + re.escape(option) + r'(?![\w])')
    return any(pat.search(m['source']) for m in readers)


# ── Cards of the corpus ───────────────────────────────────────────────

def flatten(obj, prefix=''):
    out = {}
    for k, v in obj.items():
        p = f'{prefix}.{k}' if prefix else k
        if isinstance(v, dict):
            out.update(flatten(v, p))
        else:
            out[p] = v
    return out


def card_values(rbb, name, cache):
    if name in cache:
        return cache[name]
    vals = {}
    for d in (os.path.join(PBS, 'bbsa'), os.path.join(ROOT, 'cards', 'bbsa')):
        p = os.path.join(d, name + '.bbsa')
        if os.path.exists(p):
            try:
                j = json.loads(subprocess.run([rbb, 'card', 'import-bbsa', p], cwd=ROOT, capture_output=True,
                                              text=True, check=True).stdout)
                j = {k: v for k, v in j.items() if k not in ('schema_version', 'format', 'metadata')}
                vals = flatten(j)
            except (subprocess.CalledProcessError, json.JSONDecodeError):
                pass
            break
    cache[name] = vals
    return vals


# ── Measures ──────────────────────────────────────────────────────────

def bba_status(boards, err, ns_agree):
    t = THRESHOLDS
    epb = err / boards
    if epb < t['poor']['errors_per_board'] or ns_agree < t['poor']['ns_agree']:
        return 'bba-poor'
    if epb >= t['good']['errors_per_board'] and ns_agree >= t['good']['ns_agree']:
        return 'bba-good'
    return 'bba-fair'


def pool(scen_rows):
    b = sum(r['boards'] for r in scen_rows)
    if not b:
        return None
    agree = sum(r['ns_calls_agree'] for r in scen_rows)
    total = sum(r['ns_calls'] for r in scen_rows)
    err = sum(r['errors'] for r in scen_rows)
    ns = agree / total if total else 0
    return {'boards': b, 'ns_agree': round(ns, 4), 'errors_per_board': round(err / b, 3),
            'status': bba_status(b, err, ns)}


def scenario_rows(compare):
    rows = {}
    if not compare:
        return rows
    cards = {}
    for b in compare['boards']:
        cards.setdefault(b['scenario'], b.get('ns_card'))
    for s in compare['summary']['scenarios']:
        rows[s['name']] = {
            'name': s['name'],
            'boards': s['boards'],
            'ns_calls_agree': s['calls'][0]['agree'],
            'ns_calls': s['calls'][0]['total'],
            'ns_agree': round(s['calls'][0]['agree'] / max(1, s['calls'][0]['total']), 4),
            'errors': s['par'].get('errors_vs_reference', 0),
            'errors_per_board': round(s['par'].get('errors_vs_reference', 0) / max(1, s['boards']), 3),
            'auctions_match': round(s['auctions_match'] / max(1, s['boards']), 4),
            'ns_card': cards.get(s['name']),
        }
    return rows


def conv_results(path):
    """{tile: [result, ...]} from conv_ab.py's results.json."""
    out = {}
    if not path or not os.path.exists(path):
        return out
    for r in json.load(open(path))['results'].values():
        if r.get('tile'):
            out.setdefault(r['tile'], []).append(r)
    return out


CONV_KEYS = ('changed', 'changed_ours', 'changed_bba', 'rusty_gain', 'bba_gain', 'net', 'net_other',
             'rusty_contract', 'rusty_doubling', 'bba_contract', 'bba_doubling')


def conv_pool(results):
    """The tile's convention score: its measured runs pooled (Texas with
    Jacoby, Unusual 2NT with Michaels)."""
    ms = [r for r in results if r.get('score') and not r.get('unmeasured')]
    if not ms:
        return None
    s = {k: sum(r['score'][k] for r in ms) for k in CONV_KEYS}
    s['net_halves'] = [sum(r['score']['net_halves'][h] for r in ms) for h in (0, 1)]
    n = s['changed']
    s['net_per_changed'] = round(s['net'] / n, 3) if n else None
    s['rusty_per_changed'] = round(s['rusty_gain'] / n, 3) if n else None
    s['bba_per_changed'] = round(s['bba_gain'] / n, 3) if n else None
    e, o = s['net_halves']
    s['halves_agree'] = (e > 0 and o > 0) or (e < 0 and o < 0) or (e == 0 and o == 0)
    s['runs'] = [r['name'] for r in ms]
    s['scenarios'] = sorted({x for r in ms for x in r['used']})
    st = conv_ab.status_of(s)
    s['status'] = f'conv-{st}' if st else None
    return s


def conv_row(r):
    """One conv_ab run as the page shows it."""
    row = {'name': r['name'], 'off': r['off'], 'scenarios': r.get('used') or r['scenarios'],
           'bba_keys': sorted((r.get('bba_keys') or {}).keys()), 'notes': r.get('notes', []),
           'unmeasured': r.get('unmeasured'), 'repro': r.get('repro'), 'status': None}
    if r.get('score'):
        s = r['score']
        row.update({k: s[k] for k in CONV_KEYS + ('net_halves', 'net_per_changed', 'rusty_per_changed',
                                                  'bba_per_changed', 'halves_agree', 'no_rule_on', 'no_rule_off')})
        row['status'] = f"conv-{r['status']}" if r.get('status') else None
    return row


def self_ab_results(dirpath, batch):
    """{name: result} for every batch entry whose two JSON files exist."""
    out = {}
    if not dirpath or not os.path.isdir(dirpath):
        return out
    import self_ab  # noqa: E402
    specs = tomllib.load(open(batch, 'rb'))['ab']

    class A:
        worst = 0
    for spec in specs:
        bp = os.path.join(dirpath, f"{spec['name']}.base.json")
        vp = os.path.join(dirpath, f"{spec['name']}.variant.json")
        if not (os.path.exists(bp) and os.path.exists(vp)):
            continue
        line, _ = self_ab.verdict(spec['name'], spec, bp, vp, A)
        changed = int(re.search(r'changed\s+(\d+)', line).group(1))
        word = re.search(r'z [+-][\d.]+\s+(.*)$', line).group(1).strip()
        z = float(re.search(r'z ([+-][\d.]+)', line).group(1))
        contract = int(re.search(r'actor: contract\s+([+-]\d+)', line).group(1))
        doubling = int(re.search(r'actor: contract\s+[+-]\d+ doubling\s+([+-]\d+)', line).group(1))
        if changed == 0:
            status = None
        elif word in ('gains', 'leans gain'):
            status = 'ab-gains'
        elif word in ('loses', 'leans loss'):
            status = 'ab-loses'
        else:
            status = 'ab-neutral'
        base = dict(c.split('=', 1) for c in spec.get('base', []))
        var = dict(c.split('=', 1) for c in spec.get('variant', []))
        changes = {k: v for k, v in var.items() if base.get(k) != v}
        out[spec['name']] = {
            'name': spec['name'], 'scenarios': spec['scenarios'], 'side': spec.get('side', 'both'),
            'changed': changed, 'verdict': word if changed else 'no boards changed', 'z': z,
            'actor_contract': contract, 'actor_doubling': doubling, 'status': status,
            'changes': changes, 'mtime': os.path.getmtime(vp),
        }
    return out


# ── Tiles ─────────────────────────────────────────────────────────────

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--compare', help='a full `rbb compare --json` (all scenarios)')
    ap.add_argument('--self-ab', default=os.path.join(ROOT, '.rbb-cache', 'self-ab'))
    ap.add_argument('--batch', default=os.path.join(ROOT, 'probes', 'self-ab.toml'))
    ap.add_argument('--conv-ab', default=os.path.join(ROOT, '.rbb-cache', 'conv-ab', 'results.json'),
                    help="conv_ab.py's results (the convention score of conventions BBA plays)")
    ap.add_argument('--items', default=os.path.join(ROOT, 'probes', 'status-items.toml'))
    ap.add_argument('--rbb', default=os.path.join(ROOT, 'target', 'release', 'rbb'))
    ap.add_argument('--spec', help="a spec directory to read instead of the pinned revision's")
    ap.add_argument('--out', default=os.path.join(ROOT, 'web', 'status', 'status.json'))
    a = ap.parse_args()

    pin = pinned_card_rev()
    read_spec = spec_reader(pin, a.spec)
    conventions = json.loads(read_spec('conventions.json'))['conventions']
    fields = load_fields(read_spec('fields.toml'))
    items = tomllib.load(open(a.items, 'rb'))
    mods, fields_read = load_modules(a.rbb)
    compare = json.load(open(a.compare)) if a.compare else None
    scen = scenario_rows(compare)
    ab = self_ab_results(a.self_ab, a.batch)
    conv = conv_results(a.conv_ab)
    card_cache = {}

    groups = items.get('groups', {})
    parent = {c: p for p, cs in groups.items() for c in cs}
    tile_of = lambda skill: parent.get(skill, skill)
    curated = items.get('items', {})

    def field_on(card, path, option):
        v = card.get(path)
        if option is None:
            return v is True
        return v == option

    def ab_touches(r, path, option):
        """Whether self A/B run `r` switches this treatment on."""
        for k, v in r['changes'].items():
            if k == path or k.endswith('.' + path) or path.endswith('.' + k):
                if (option is None and v == 'true') or (option is not None and v == option):
                    return True
        return False

    def treatment_ab(path, option):
        return next((r for r in ab.values() if ab_touches(r, path, option)), None)

    def measured_treatment(t, tile_scen):
        r = treatment_ab(t['field'], t.get('option'))
        if r and r['status']:
            t['status'] = r['status']
            t['why'] = f"self A/B {r['name']}: {r['verdict']} ({r['changed']} boards changed)"
            return
        if not scen or not tile_scen:
            return
        on = [scen[s] for s in tile_scen if s in scen and scen[s]['ns_card']
              and field_on(card_values(a.rbb, scen[s]['ns_card'], card_cache), t['field'], t.get('option'))]
        p = pool(on)
        if p:
            t['status'] = p['status']
            t['why'] = (f"vs BBA in {', '.join(r['name'] for r in on)}: {p['errors_per_board']:+.2f} IMPs/board, "
                        f"NS calls agreeing {p['ns_agree'] * 100:.0f}%")

    def module_list(names):
        return [{k: mods[n][k] for k in ('name', 'title', 'file', 'url', 'rules', 'tests', 'notes')}
                for n in names if n in mods]

    tiles = {}
    # Spec conventions.
    for cid, c in conventions.items():
        cat = cid.split('/')[0]
        if cat not in CATEGORY_SECTION or cid in parent:
            continue
        tiles[cid] = {
            'id': cid, 'name': c['name'], 'section': CATEGORY_SECTION[cat], 'level': c.get('level'),
            'summary': ' '.join(c.get('summary', '').split()), 'source': c.get('source'),
            'members': [cid] + groups.get(cid, []), 'treatments': [], 'modules': [],
            'see': c.get('see', []),
        }
        for child in groups.get(cid, []):
            if child in conventions:
                tiles[cid].setdefault('variants', []).append(
                    {'id': child, 'name': conventions[child]['name'], 'level': conventions[child].get('level'),
                     'summary': ' '.join(conventions[child].get('summary', '').split())})

    # Treatments: card fields by their (first) skill; enum options by the
    # skill named like the option when the field lists one.
    for path, f in fields.items():
        sk = skills_of(f)
        if not sk or f.get('note') or f['kind'] == 'text':
            continue
        read = path in fields_read
        if f['kind'] == 'enum':
            for opt in f.get('options', []):
                if opt in SKIP_OPTIONS:
                    continue
                own = next((s for s in sk if s.split('/')[-1] == opt), sk[0])
                tid = tile_of(own)
                if tid not in tiles:
                    continue
                ok = option_read(path, opt, f, mods)
                tiles[tid]['treatments'].append({
                    'label': f"{f['label']}: {opt.replace('_', ' ')}", 'field': path, 'option': opt, 'skill': own,
                    'level': f.get('level'), 'status': 'tested' if ok else 'gap',
                    'why': 'a rule plays this option' if ok else 'no rule plays this option'})
        else:
            tid = tile_of(sk[0])
            if tid not in tiles:
                continue
            tiles[tid]['treatments'].append({
                'label': f['label'], 'field': path, 'skill': sk[0], 'level': f.get('level'),
                'status': 'tested' if read else 'gap',
                'why': 'a rule reads this field' if read else 'no rule reads this field'})
    for tid, cur in curated.items():
        if tid not in tiles:
            continue
        for path in cur.get('fields', []):
            sec, key = path.split('.', 1)
            f = fields.get(path)
            if not f:
                continue
            for opt in (f.get('options') or [None]):
                if opt in SKIP_OPTIONS:
                    continue
                ok = option_read(path, opt, f, mods) if opt else path in fields_read
                tiles[tid]['treatments'].append({
                    'label': f"{f['label']}{': ' + opt.replace('_', ' ') if opt else ''}", 'field': path,
                    'option': opt, 'level': f.get('level'), 'status': 'tested' if ok else 'gap',
                    'why': 'a rule plays this' if ok else 'no rule plays this'})
        for t in cur.get('treatments', []):
            tiles[tid]['treatments'].append({'label': t['label'], 'field': None, 'status': t['status'],
                                             'why': t.get('note', '')})

    # Modules by skill.
    for m in mods.values():
        for s in m['skills']:
            tid = tile_of(s)
            if tid in tiles and m['name'] not in tiles[tid]['modules']:
                tiles[tid]['modules'].append(m['name'])

    # Curated tiles.
    for kind, section in (('basic', 'basic'), ('judgment', 'judgment'), ('catchall', 'catchall')):
        for it in items.get(kind, []):
            tiles[it['id']] = {
                'id': it['id'], 'name': it['name'], 'section': section, 'level': it.get('level'),
                'summary': it.get('summary', ''), 'source': 'curated', 'members': [it['id']],
                'treatments': [], 'modules': list(it.get('modules', [])), 'measure': it.get('measure'),
                'scenarios_curated': it.get('scenarios', []), 'see': [],
            }

    # Curated names that match nothing: a typo here would silently drop a
    # module or a scenario from the page.
    for tid, t in tiles.items():
        for n in t['modules'] + curated.get(tid, {}).get('modules', []):
            if n not in mods:
                print(f'warning: {tid}: no module {n}', file=sys.stderr)
        for s in t.get('scenarios_curated') or curated.get(tid, {}).get('scenarios', []):
            if scen and s not in scen:
                print(f'warning: {tid}: no scenario {s} in the compare', file=sys.stderr)
    for tid in curated:
        if tid not in tiles:
            print(f'warning: status-items.toml: [items."{tid}"] is no tile (grouped, or not in the spec)', file=sys.stderr)

    total_boards = compare['summary']['total']['boards'] if compare else 0
    tot = compare['summary']['total'] if compare else None

    out_tiles = []
    for tid, t in tiles.items():
        cur = curated.get(tid, {})
        for n in cur.get('modules', []):
            if n not in t['modules']:
                t['modules'].append(n)
        scen_names = t.pop('scenarios_curated', None) or cur.get('scenarios', [])
        t['scenarios'] = [{k: scen[s][k] for k in ('name', 'boards', 'ns_agree', 'errors_per_board', 'auctions_match', 'ns_card')}
                          | {'status': bba_status(scen[s]['boards'], scen[s]['errors'], scen[s]['ns_agree'])}
                          for s in scen_names if s in scen]
        t['scenarios_missing'] = [s for s in scen_names if s not in scen] if scen else []
        t['bba'] = pool([scen[s] for s in scen_names if s in scen])
        for tr in t['treatments']:
            if tr['status'] == 'tested' and tr.get('field'):
                measured_treatment(tr, scen_names)
        # Self A/B runs that touch this tile's fields.
        t['ab'] = [{k: r[k] for k in ('name', 'scenarios', 'side', 'changed', 'verdict', 'z', 'actor_contract',
                                       'actor_doubling', 'status', 'changes')}
                   for r in ab.values()
                   if any(tr.get('field') and ab_touches(r, tr['field'], tr.get('option')) for tr in t['treatments'])]
        # The convention score (BBA plays it): Rusty's gain against BBA's
        # on the boards it changed, with the scenario score on the same
        # deals beside it as the background.
        t['conv'] = [conv_row(r) for r in conv.get(tid, [])]
        t['conv_score'] = conv_pool(conv.get(tid, []))
        if t['conv_score']:
            t['conv_score']['background'] = pool([scen[s] for s in t['conv_score']['scenarios'] if s in scen])
        t['partial'] = cur.get('partial')
        t['modules'] = module_list(t['modules'])
        t['tests'] = sum(m['tests'] for m in t['modules'])
        t['rules'] = sum(m['rules'] for m in t['modules'])
        t['notes'] = [m['notes'] for m in t['modules'] if m['notes']]

        # The tile's status and why.
        measure = t.pop('measure', None)
        # A self A/B decides the tile when it switches the tile's own
        # convention (BBA does not play it); one that switches a variant
        # (Rubensohl under Lebensohl) only colours that treatment's chip.
        own = [tr for tr in t['treatments'] if tr.get('field') and tr.get('skill') == tid]
        ab_main = next((r for r in t['ab'] if r['status']
                        and any(ab_touches(ab[r['name']], tr['field'], tr.get('option')) for tr in own)), None)
        if ab_main is None and not t['bba']:
            ab_main = next((r for r in t['ab'] if r['status']), None)
        implemented = bool(t['modules']) or any(tr['status'] != 'gap' and tr.get('field') for tr in t['treatments'])
        if measure and compare and 'class' in measure:
            pc = tot['par_classes']
            ref = sum(pc.get(k, [{'boards': 0}, {'boards': 0}])[0]['boards'] for k in measure['class'])
            ours = sum(pc.get(k, [{'boards': 0}, {'boards': 0}])[1]['boards'] for k in measure['class'])
            ratio = ours / ref if ref else 0
            th = THRESHOLDS['class_ratio']
            t['status'] = 'bba-good' if ratio <= th['good'] else 'bba-fair' if ratio <= th['fair'] else 'bba-poor'
            t['measure'] = {'kind': 'par class', 'classes': measure['class'], 'ours': ours, 'bba': ref,
                            'ratio': round(ratio, 3), 'boards': total_boards}
            t['why'] = (f"{', '.join(c.replace('_', ' ') for c in measure['class'])}: {ours} boards for us, "
                        f"{ref} for BBA, all {total_boards} corpus boards")
        elif measure and compare and (measure.get('no_rule') or measure.get('read_as')):
            n = tot['problems']['no_rule'] if measure.get('no_rule') else tot['read_as']
            per = n / total_boards * 1000
            th = THRESHOLDS['per_thousand']
            t['status'] = 'bba-good' if per < th['good'] else 'bba-fair' if per < th['fair'] else 'bba-poor'
            t['measure'] = {'kind': 'no_rule' if measure.get('no_rule') else 'read_as', 'count': n,
                            'per_thousand': round(per, 2), 'boards': total_boards}
            if measure.get('no_rule'):
                t['examples'] = [{'auction': p['auction'], 'call': p['call'], 'count': p['count']}
                                 for p in compare['summary']['problems'] if p['kind'] == 'no_rule'][:8]
            else:
                t['examples'] = [{'auction': f"{os.path.basename(r['chosen'])} -> {os.path.basename(r['read'])}",
                                  'call': r['call'], 'count': r['count']} for r in compare['summary']['read_as'][:8]]
            t['why'] = f"{n} in {total_boards} corpus boards ({per:.1f} per 1,000)"
        elif not implemented:
            t['status'], t['why'] = 'gap', 'no module implements it'
        elif t['partial']:
            t['status'], t['why'] = 'partial', t['partial']
        elif t['conv_score'] and t['conv_score']['status']:
            # BBA plays it: the convention score decides, ahead of a self
            # A/B (which is for conventions BBA does not play).
            c = t['conv_score']
            t['status'] = c['status']
            bg = c.get('background')
            if bg:
                t['background'] = bg['status']
            t['why'] = (f"convention score {c['net_per_changed']:+.2f} IMPs per changed board "
                        f"(our gain {c['rusty_gain']:+}, BBA's {c['bba_gain']:+}, {c['changed']} boards)"
                        + (f"; background on these deals {bg['errors_per_board']:+.2f} IMPs/board, "
                           f"NS calls agreeing {bg['ns_agree'] * 100:.0f}%" if bg else ''))
        elif ab_main:
            t['status'] = ab_main['status']
            t['why'] = f"self A/B {ab_main['name']}: {ab_main['verdict']} ({ab_main['changed']} boards changed)"
        elif t['bba']:
            t['status'] = t['bba']['status']
            t['why'] = (('background (not isolated): ' if t['section'] in ('constructive', 'competitive', 'precision') else '')
                        + f"errors {t['bba']['errors_per_board']:+.2f} IMPs/board against BBA's, "
                        f"NS calls agreeing {t['bba']['ns_agree'] * 100:.0f}% ({t['bba']['boards']} boards)")
        else:
            t['status'] = 'tested'
            t['why'] = f"{t['tests']} test cases; no corpus or A/B measure"
        t['treatments'].sort(key=lambda x: (x.get('level') or 0, x['label']))
        out_tiles.append(t)

    section_order = {s: i for i, (s, _) in enumerate(SECTIONS)}
    out_tiles.sort(key=lambda t: (section_order[t['section']], t['level'] or 0, t['name'].lower()))

    # Header numbers.
    tests = sum(m['tests'] for m in mods.values())
    rules = sum(m['rules'] for m in mods.values())
    cov = []
    try:
        txt = sh(a.rbb, 'card', 'coverage', *sorted(glob.glob(os.path.join(ROOT, 'cards', 'bbsa', '*.bbsa'))))
        for line in txt.splitlines():
            m = re.match(r'(\S+)\s+(\S+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)%', line)
            if m:
                cov.append({'card': m.group(1), 'system': m.group(2), 'coverage': int(m.group(6))})
    except subprocess.CalledProcessError:
        pass
    corpus = None
    if tot:
        c0, c1 = tot['calls']
        corpus = {
            'scenarios': len(compare['summary']['scenarios']), 'boards': tot['boards'],
            'calls_agree': round((c0['agree'] + c1['agree']) / (c0['total'] + c1['total']), 4),
            'ns_agree': round(c0['agree'] / c0['total'], 4), 'ew_agree': round(c1['agree'] / c1['total'], 4),
            'auctions_match': round(tot['auctions_match'] / tot['boards'], 4),
            'contracts_match': round(tot['contracts_match'] / tot['boards'], 4),
            'errors_per_board': round(tot['par']['errors_vs_reference'] / tot['boards'], 3),
            'no_rule': tot['problems']['no_rule'],
        }
    try:
        commit = sh('git', 'rev-parse', '--short', 'HEAD').strip()
        date = sh('git', 'log', '-1', '--format=%cs').strip()
    except subprocess.CalledProcessError:
        commit, date = None, None
    version = sh(a.rbb, '--version').strip()
    counts = {}
    for t in out_tiles:
        counts[t['status']] = counts.get(t['status'], 0) + 1
    data = {
        'schema': 'rbb-status/1',
        'generated': datetime.date.today().isoformat(),
        'engine': version,
        'commit': commit,
        'commit_date': date,
        'card_spec': pin,
        'repo': REPO_URL,
        'thresholds': THRESHOLDS | {'conv': conv_ab.THRESHOLDS},
        'sections': [{'id': s, 'title': n} for s, n in SECTIONS],
        'header': {
            'modules': len(mods), 'rules': rules, 'tests': tests, 'fields_read': len(fields_read),
            'cards': cov, 'corpus': corpus,
            'self_ab': {'runs': len(ab), 'date': datetime.date.fromtimestamp(max(r['mtime'] for r in ab.values())).isoformat()} if ab else None,
            'counts': counts,
        },
        'tiles': out_tiles,
    }
    os.makedirs(os.path.dirname(a.out), exist_ok=True)
    with open(a.out, 'w') as f:
        json.dump(data, f, ensure_ascii=False, indent=1)
        f.write('\n')
    print(f"{a.out}: {len(out_tiles)} tiles; " + ', '.join(f'{k} {v}' for k, v in sorted(counts.items())), file=sys.stderr)


if __name__ == '__main__':
    main()
