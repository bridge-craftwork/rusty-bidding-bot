#!/usr/bin/env python3
"""Brill's hand-annotated rules as test cases for Rusty.

Many rules in Brill's convention files (the private mirror
bridge-craftwork/brill-data, checked out at ../brill-data) carry the hand
they were written for: `"means": "Board: 530, Hand: KT7.KQ6.AK764.T2"`.
Each such rule set gives an auction, a hand and Brill's call (the
highest-priority rule naming that hand). This asks Rusty the same question
with `rbb call` and reports where the two differ.

    brill_cases.py extract [--only Texas,Michaels*]   cases.jsonl
    brill_cases.py run [--jobs N]                      results (cached)
    brill_cases.py report [--top 15] [--out FILE.md]   per Brill file

Contexts: exact auctions as written; `PX` (pass or double) is read as
Pass; `Ma`/`Mi`/`oMa`/`oMi` are expanded to both suit choices and marked
`expanded` (the board had one of them; we cannot tell which). Contexts
with `*`, `V`/`W`/`R` or `...` are skipped and counted. Dealer and
vulnerability come from a numeric board number (standard rotation);
`T...` boards get dealer N, vulnerability None.

A tag in the means (`[MS=4-sacrifice]`, `[MS=4-missed-penalty-double]`)
marks one of Brill's judgement lessons; the report keeps those apart from
the plain convention cases.

Files live in .rbb-cache/brill-cases/ (results keyed by the rules' git
state, so a rerun after a rule change asks again).
"""
import argparse, collections, concurrent.futures as cf, fnmatch, glob, hashlib
import json, os, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA = os.path.join(os.path.dirname(ROOT), 'brill-data', 'BiddingSystem')
CACHE = os.path.join(ROOT, '.rbb-cache', 'brill-cases')
RBB = os.path.join(ROOT, 'target', 'release', 'rbb')
CARD = os.path.join(ROOT, 'cards', 'json', 'Brill.json')

HAND = re.compile(r'Board: (\S+?),\s*Hand: ([AKQJT2-9]*\.[AKQJT2-9]*\.[AKQJT2-9]*\.[AKQJT2-9]*)')
TAG = re.compile(r'\[MS=([^\]\s(]+)')
CALL = re.compile(r'[1-7][CDHSN]|P|X|XX')
SEATS = 'NESW'
# Standard board rotation: dealer and vulnerability of board n.
VULS = ['None', 'NS', 'EW', 'All', 'NS', 'EW', 'All', 'None',
        'EW', 'All', 'None', 'NS', 'All', 'None', 'NS', 'EW']


def board_seat(board):
    if board.isdigit():
        n = int(board)
        return SEATS[(n - 1) % 4], VULS[(n - 1) % 16]
    return 'N', 'None'


def expand(toks):
    """Concrete call lists for Brill tokens (context plus bid), and whether
    suits were chosen for Ma/Mi."""
    if any(t in ('...', '*', '**') or re.search(r'[*VWR^!]', t) for t in toks):
        return [], False
    toks = ['P' if t in ('PX', 'PXX') else t for t in toks]
    wild = any(re.fullmatch(r'[1-7]o?a?M[ai]', t) for t in toks)
    out = []
    for ma, mi in ([('H', 'C'), ('H', 'D'), ('S', 'C'), ('S', 'D')] if wild else [(None, None)]):
        sub = {'Ma': ma, 'oMa': 'S' if ma == 'H' else 'H', 'Mi': mi, 'oMi': 'D' if mi == 'C' else 'C',
               'aMa': ma, 'aMi': mi}
        res = []
        for t in toks:
            m = re.fullmatch(r'([1-7])(o?a?M[ai])', t)
            res.append(m.group(1) + sub[m.group(2)] if m else t)
        if all(CALL.fullmatch(t) for t in res):
            out.append(res)
    # Expansions that differ only in an unused variable are the same auction.
    uniq = []
    for a in out:
        if a not in uniq:
            uniq.append(a)
    return uniq, wild


def prefixes(ctx, mode, activations):
    """The calls before a context: none for an absolute one; for a relative
    one (it starts with the convention's activating call), the context of
    each activation rule making that call. A mixed file holds both: a
    context that cannot open an auction (it starts with a double), or
    starts with an activating call after which it would not be legal from
    the dealer, is relative."""
    first = ctx.split('-')[0] if ctx else ''
    rel = mode == 'relative' or (mode != 'absolute' and first in ('X', 'XX')) \
        or (mode == 'absolute' and first in ('X', 'XX'))
    if not rel:
        return [[]]
    out = []
    for fc, bid in activations:
        if bid == first:
            p = fc.split('-') if fc else []
            if p not in out:
                out.append(p)
    return out


def to_rbb(calls):
    return ' '.join({'P': 'Pass', 'X': 'X', 'XX': 'XX'}.get(c, c.replace('N', 'NT')) for c in calls)


def from_rbb(call):
    if call == 'Pass':
        return 'P'
    if call == 'Double':
        return 'X'
    if call == 'Redouble':
        return 'XX'
    b = call['Bid']
    return f"{b['level']}{'N' if b['strain'].startswith('N') else b['strain'][0]}"


def legal(calls):
    """Bids rise; doubles and redoubles where allowed. Enough to drop typos."""
    last, by, dbl, dbl_by = (0, -1), None, 0, None
    order = 'CDHSN'
    for i, c in enumerate(calls):
        if c == 'P':
            continue
        if c == 'X':
            if by is None or (i - by) % 2 == 0 or dbl:
                return False
            dbl, dbl_by = 1, i
        elif c == 'XX':
            if dbl != 1 or (i - dbl_by) % 2 == 0:
                return False
            dbl = 2
        else:
            v = (int(c[0]), order.index(c[1]))
            if v <= last:
                return False
            last, by, dbl = v, i, 0
    return True


def extract(args):
    only = [p.strip() for p in args.only.split(',')] if args.only else None
    cases, skipped = [], collections.Counter()
    for path in sorted(glob.glob(os.path.join(args.data, 'Conventions', '*.json'))):
        name = os.path.basename(path)[:-5]
        if name == 'manifest' or (only and not any(fnmatch.fnmatch(name, p) for p in only)):
            continue
        d = json.load(open(path))
        mode = d.get('contextMode') or 'absolute'
        activations = [(a.get('forContext', ''), a.get('bid', '')) for a in d.get('ActivationRules', []) or []]
        for rs in d.get('ruleSets', []) or []:
            by_hand = {}
            for r in rs.get('rules', []):
                m = HAND.search(r.get('means', '') or '')
                if not m:
                    continue
                key = m.group(2)
                if key not in by_hand or r.get('priority', 0) > by_hand[key][1].get('priority', 0):
                    by_hand[key] = (m.group(1), r)
            for hand, (board, r) in by_hand.items():
                if sum(len(s) for s in hand.split('.')) != 13:
                    skipped['bad hand'] += 1
                    continue
                tag = TAG.search(r['means'])
                for ctx in rs.get('auctionContext', []):
                    full = []
                    for prefix in prefixes(ctx, mode, activations):
                        got, wild = expand(prefix + ctx.split('-') + [r['bid']])
                        full += [(g[:-1], g[-1], wild or len(prefixes(ctx, mode, activations)) > 1) for g in got]
                    if not full:
                        skipped['wildcard or unplaced context'] += 1
                        continue
                    for a, bid, wild in full:
                        if not legal(a + [bid]):
                            skipped['illegal context'] += 1
                            continue
                        dealer, vul = board_seat(board)
                        cases.append({
                            'file': name, 'context': ctx, 'auction': a, 'hand': hand, 'board': board,
                            'dealer': dealer, 'vul': vul, 'brill': bid, 'means': r['means'],
                            'requires': r.get('requires', ''), 'priority': r.get('priority', 0),
                            'tag': tag.group(1) if tag else None, 'expanded': wild})
    os.makedirs(CACHE, exist_ok=True)
    with open(os.path.join(CACHE, 'cases.jsonl'), 'w') as f:
        for c in cases:
            f.write(json.dumps(c) + '\n')
    print(f'{len(cases)} cases from {len({c["file"] for c in cases})} files; skipped {dict(skipped)}')


def rules_state():
    rev = subprocess.run(['git', 'rev-parse', '--short', 'HEAD'], cwd=ROOT, capture_output=True, text=True).stdout.strip()
    diff = subprocess.run(['git', 'diff', 'HEAD', '--', 'conventions'], cwd=ROOT, capture_output=True, text=True).stdout
    return rev + ('-' + hashlib.sha1(diff.encode()).hexdigest()[:8] if diff else '')


def ask(c, card):
    cmd = [RBB, 'call', '-c', card, '-d', c['dealer'], '-v', c['vul'], '-s', 'IMP',
           '-a', to_rbb(c['auction']), '--json', c['hand']]
    p = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if p.returncode:
        return {'error': p.stderr.strip()[:300]}
    d = json.loads(p.stdout)
    pos = d['auction']['position']
    return {'call': from_rbb(d['call']), 'explanation': d['explanation'],
            'rule': f"{d['rule']['module']}:{d['rule']['line']}" if d['rule'] else None,
            'unread': [i for i, u in enumerate(pos.get('unread', [])) if u],
            'context_rules': [s['rule']['module'] if s.get('rule') else None for s in d['auction']['steps']]}


def run(args):
    cases = [json.loads(l) for l in open(os.path.join(CACHE, 'cases.jsonl'))]
    out = os.path.join(CACHE, f'results-{rules_state()}.jsonl')
    done = {}
    if os.path.exists(out):
        for l in open(out):
            r = json.loads(l)
            done[r['key']] = r
    key = lambda c: f"{c['dealer']}|{c['vul']}|{' '.join(c['auction'])}|{c['hand']}"
    todo = list({key(c): c for c in cases if key(c) not in done}.values())
    print(f'{len(cases)} cases, {len(todo)} positions to ask ({out})', flush=True)
    with open(out, 'a') as f, cf.ThreadPoolExecutor(args.jobs) as ex:
        for i, (c, r) in enumerate(zip(todo, ex.map(lambda c: ask(c, args.card), todo))):
            r['key'] = key(c)
            f.write(json.dumps(r) + '\n')
            if i % 500 == 499:
                print(f'  {i + 1}/{len(todo)}', flush=True)
    print('done')


def report(args):
    cases = [json.loads(l) for l in open(os.path.join(CACHE, 'cases.jsonl'))]
    out = os.path.join(CACHE, f'results-{rules_state()}.jsonl')
    res = {}
    for l in open(out):
        r = json.loads(l)
        res[r['key']] = r
    key = lambda c: f"{c['dealer']}|{c['vul']}|{' '.join(c['auction'])}|{c['hand']}"
    per = collections.defaultdict(lambda: collections.Counter())
    diffs = collections.defaultdict(collections.Counter)
    examples = {}
    for c in cases:
        r = res.get(key(c))
        if not r or 'error' in r:
            per[c['file']]['error' if r else 'unasked'] += 1
            continue
        kind = 'lesson' if c['tag'] else ('wild' if c['expanded'] else 'plain')
        bucket = per[(c['file'])]
        bucket[kind] += 1
        same = r['call'] == c['brill']
        bucket[kind + '_same'] += same
        if r['rule'] is None:
            bucket[kind + '_norule'] += 1
        if r['unread']:
            bucket[kind + '_unread'] += 1
        if not same and kind == 'plain':
            # Group by the auction shape: the context with its last call, Brill's call, ours.
            g = (c['file'], kind, c['context'], c['brill'], r['call'])
            diffs[c['file']][g] += 1
            examples.setdefault(g, (c, r))
    lines = ['# Brill hand cases against Rusty', '',
             f'Rules state `{rules_state()}`, card `{os.path.relpath(args.card, ROOT)}`.',
             '"plain": convention cases; "lesson": Brill\'s tagged judgement lessons ([MS=...]).',
             '"no rule": Rusty passed because no rule applied. "unread": some call of the context is read by no Rusty rule.', '',
             '"wild": plain cases whose context had Ma/Mi wildcards or several activations, expanded to every choice (only one was the real board).', '',
             '| Brill file | plain | agree | no rule | unread | wild | agree | lessons | agree |',
             '|---|---:|---:|---:|---:|---:|---:|---:|---:|']
    tot = collections.Counter()
    for f in sorted(per, key=lambda f: -(per[f]['plain'] + per[f]['lesson'])):
        b = per[f]
        tot.update(b)
        pct = lambda a, n: f'{a} ({100 * a // n}%)' if n else '—'
        lines.append(f"| {f} | {b['plain']} | {pct(b['plain_same'], b['plain'])} | {b['plain_norule']} | {b['plain_unread']} | {b['wild']} | {pct(b['wild_same'], b['wild'])} | {b['lesson']} | {pct(b['lesson_same'], b['lesson'])} |")
    pct = lambda a, n: f'{a} ({100 * a // n}%)' if n else '—'
    lines.append(f"| **all** | {tot['plain']} | {pct(tot['plain_same'], tot['plain'])} | {tot['plain_norule']} | {tot['plain_unread']} | {tot['wild']} | {pct(tot['wild_same'], tot['wild'])} | {tot['lesson']} | {pct(tot['lesson_same'], tot['lesson'])} |")
    lines.append('')
    for f in sorted(diffs, key=lambda f: -sum(diffs[f].values())):
        lines += [f'## {f}', '', '| n | kind | Brill context | Brill | Rusty | example hand | Brill means | Rusty |', '|---:|---|---|---|---|---|---|---|']
        for g, n in diffs[f].most_common(args.top):
            c, r = examples[g]
            ours = f"{r['explanation']} ({r['rule'] or 'no rule'})"
            lines.append(f"| {n} | {g[1]} | `{g[2]}` | {g[3]} | {g[4]} | {c['hand']} | {c['means'][:70]} | {ours[:80]} |")
        lines.append('')
    text = '\n'.join(lines) + '\n'
    if args.out:
        open(args.out, 'w').write(text)
    print(text if not args.out else f"wrote {args.out}: {tot['plain']} plain cases, {tot['plain_same']} agree")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--data', default=DATA)
    ap.add_argument('--card', default=CARD)
    sub = ap.add_subparsers(dest='cmd', required=True)
    e = sub.add_parser('extract'); e.add_argument('--only')
    r = sub.add_parser('run'); r.add_argument('--jobs', type=int, default=int(os.environ.get('RBB_CPUS', max(1, (os.cpu_count() or 4) - 2))))
    p = sub.add_parser('report'); p.add_argument('--top', type=int, default=15); p.add_argument('--out')
    args = ap.parse_args()
    {'extract': extract, 'run': run, 'report': report}[args.cmd](args)


if __name__ == '__main__':
    main()
