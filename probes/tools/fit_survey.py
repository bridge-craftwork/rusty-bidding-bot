#!/usr/bin/env python3
"""Fit survey (fit.notes.md, 2026-10-02): boards where our side played
notrump or a minor although the hands held eight or more in a major,
classed by what the hands knew before each of their calls (`rbb
explain-auction`): agreed (shown minimums make 8), private (the caller's
length and partner's shown minimum make 8), hidden; and our 4NT calls
by the rule's kind, the trump state and public agreement.

    probes/tools/fit_survey.py explain COMPARE.json CARD OUTDIR [--ew-card C] [--set a=b ...] [--rules DIR] [--jobs 8]
    probes/tools/fit_survey.py report  COMPARE.json OUTDIR [--top N] [--dump FILE]

Run from the repository root. `explain` runs `rbb explain-auction` (RBB,
default target/release/rbb) on every candidate board, in parallel, and
caches the rows in OUTDIR/explain.jsonl; `report` prints the tables.
"""
import argparse, json, os, subprocess, sys, collections, concurrent.futures as cf

RBB = os.environ.get('RBB', os.path.abspath('target/release/rbb'))
WT = os.getcwd()
SEATS = 'NESW'
VUL = {'None': 'None', 'NorthSouth': 'NS', 'EastWest': 'EW', 'Both': 'All'}
SC = {'Matchpoints': 'MP', 'IMPs': 'IMP', 'Imps': 'IMP'}
SUITI = {'S': 3, 'H': 2, 'D': 1, 'C': 0}   # knowledge len order C D H S

IMP_STEPS = [20, 50, 90, 130, 170, 220, 270, 320, 370, 430, 500, 600, 750, 900,
             1100, 1300, 1500, 1750, 2000, 2250, 2500, 3000, 3500, 4000]


def call_s(c):
    if isinstance(c, str):
        return {'Pass': 'P', 'Double': 'X', 'Redouble': 'XX'}.get(c, c)
    b = c['Bid']
    return f"{b['level']}{b['strain'][0]}"


def hands(deal):
    first, rest = deal.split(':')
    hs = rest.split()
    i = SEATS.index(first)
    out = {}
    for k, h in enumerate(hs):
        out[SEATS[(i + k) % 4]] = [len(s) for s in h.split('.')]  # S H D C
    return out


def run_explain(args):
    key, auction, dealer, vul, sc, card, sets, rules, ew = args
    cmd = [RBB, 'explain-auction', '-a', auction, '-d', dealer[0], '-v', vul, '-s', sc,
           '-c', card, '--json', '--rules', rules]
    for s in sets:
        cmd += ['--set', s]
    if ew:
        cmd += ['--ew-card', ew]
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=WT)
    if r.returncode != 0:
        return key, None
    rows = json.loads(r.stdout)
    slim = []
    for row in rows:
        st = row['step']
        slim.append({'caller': st['caller'], 'call': call_s(st['call']),
                     'rule': (st['rule'] or {}).get('file', '') + ':' + str((st['rule'] or {}).get('line', '')) if st.get('rule') else None,
                     'expl': st['explanation'], 'art': st['artificial'],
                     'seats': [{'len': [(l['lo'], l['hi']) for l in s['len']], 'hcp': (s['hcp']['lo'], s['hcp']['hi'])} for s in row['seats']],
                     'sides': [{'trump': sd['trump'], 'forcing': sd['forcing'], 'ask': sd['ask']} for sd in row['sides']],
                     'flags': {'ask': row['flags'].get('ask'), 'agrees': row['flags'].get('agrees')}})
    return key, slim


def explain_main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument('compare')
    ap.add_argument('card')
    ap.add_argument('out')
    ap.add_argument('--set', action='append', default=[])
    ap.add_argument('--rules', default=WT + '/conventions')
    ap.add_argument('--jobs', type=int, default=8)
    ap.add_argument('--ew-card')
    a = ap.parse_args(argv)
    a.card = os.path.abspath(a.card) if os.path.exists(a.card) else a.card
    os.makedirs(a.out, exist_ok=True)
    d = json.load(open(a.compare))
    cache_p = os.path.join(a.out, 'explain.jsonl')
    cache = {}
    if os.path.exists(cache_p):
        for line in open(cache_p):
            o = json.loads(line)
            cache[o['key']] = o['rows']
    jobs = []
    cand = {}
    for b in d['boards']:
        oc = b['our_contract']
        calls = [call_s(c) for c in b['ours']]
        key = f"{b['scenario']}:{b['board']}"
        want = False
        if oc and oc != 'Pass' and ' ' in oc:
            con, dec = oc.split()[:2]
            strain = con[1]
            L = hands(b['deal'])
            side = 'NS' if dec in 'NS' else 'EW'
            if strain in 'NCD' and any(L[side[0]][mi] + L[side[1]][mi] >= 8 for mi in (0, 1)):
                want = True
        if '4N' in calls:
            want = True
        if want:
            cand[key] = b
            if key not in cache:
                jobs.append((key, ' '.join(calls), b['dealer'], VUL[b['vul']], SC.get(b['scoring'], 'MP'),
                             a.card, a.set, a.rules, a.ew_card))
    print(f"{len(cand)} candidates, {len(jobs)} to explain")
    with open(cache_p, 'a') as f, cf.ThreadPoolExecutor(a.jobs) as ex:
        for i, (key, rows) in enumerate(ex.map(run_explain, jobs)):
            if rows is None:
                continue
            cache[key] = rows
            f.write(json.dumps({'key': key, 'rows': rows}) + '\n')
    json.dump(list(cand.keys()), open(os.path.join(a.out, 'cand.json'), 'w'))



SI = {'S': 3, 'H': 2}            # knowledge len index (C D H S)
HI = {'S': 0, 'H': 1}            # hand string index (S H D C)
DDS = {'C': 0, 'D': 1, 'H': 2, 'S': 3, 'N': 4}


def imp(d):
    n = sum(1 for v in IMP_STEPS if abs(d) >= v)
    return n if d >= 0 else -n


def hand_lengths(deal):
    first, rest = deal.split(':')
    hs = rest.split()
    i = SEATS.index(first)
    return {SEATS[(i + k) % 4]: [len(s) for s in h.split('.')] for k, h in enumerate(hs)}


def score(level, strain, tricks, vul, dbl=0):
    need = level + 6
    if tricks >= need:
        per = 20 if strain in 'CD' else 30
        base = per * level + (10 if strain == 'N' else 0)
        if dbl:
            base *= 2
        s = base
        game = base >= 100
        s += (500 if vul else 300) if game else 50
        if level == 6:
            s += 750 if vul else 500
        if level == 7:
            s += 1500 if vul else 1000
        if dbl:
            s += 50
            s += (tricks - need) * (200 if vul else 100)
        else:
            s += (tricks - need) * per
        return s
    down = need - tricks
    if not dbl:
        return -(100 if vul else 50) * down
    if vul:
        return -(200 + 300 * (down - 1))
    return -(100 + 200 * min(down - 1, 2) + 300 * max(down - 3, 0))


def side_vul(vul, side):
    return vul == 'Both' or (vul == 'NorthSouth' and side == 'NS') or (vul == 'EastWest' and side == 'EW')


def report_main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument('compare')
    ap.add_argument('out')
    ap.add_argument('--top', type=int, default=25)
    ap.add_argument('--dump')
    a = ap.parse_args(argv)
    d = json.load(open(a.compare))
    boards = {f"{b['scenario']}:{b['board']}": b for b in d['boards']}
    rows = {}
    for line in open(a.out + '/explain.jsonl'):
        o = json.loads(line)
        rows[o['key']] = o['rows']

    cls = collections.Counter()
    cost = collections.Counter()     # our contract errors, side X
    pot = collections.Counter()      # DD gain of the major contract
    vsb = collections.Counter()
    pos = collections.defaultdict(lambda: [0, 0, 0])   # private positions: boards, errors, pot
    posc = collections.defaultdict(collections.Counter)
    dump = []
    q4 = collections.Counter()
    q4ex = collections.defaultdict(list)
    for key, R in rows.items():
        b = boards.get(key)
        if b is None:
            continue
        oc = b['our_contract']
        calls = [r['call'] for r in R]
        L = hand_lengths(b['deal'])
        # ---- 4NT readings
        for i, r in enumerate(R):
            if r['call'] != '4N':
                continue
            me = SEATS.index(r['caller'][0])
            pa = (me + 2) % 4
            before = R[i - 1]['seats'] if i else None
            side = 0 if r['caller'][0] in 'NS' else 1
            trump_before = R[i - 1]['sides'][side]['trump'] if i else None
            ag = []
            pf = []
            for M in 'SH':
                if before:
                    if before[me]['len'][SI[M]][0] + before[pa]['len'][SI[M]][0] >= 8:
                        ag.append(M)
                    if before[pa]['len'][SI[M]][0] >= 6:
                        pf.append(M)
            ask = (r['flags'] or {}).get('ask') or ''
            kind = 'keycard' if 'keycard' in str(ask).lower() or 'aces' in str(ask).lower() else ('quant' if 'quant' in str(ask).lower() else f'other:{ask}')
            tag = (kind, 'trump=' + str(trump_before), 'agreedM=' + ''.join(ag), 'p6=' + ''.join(pf))
            q4[tag] += 1
            if len(q4ex[tag]) < 4:
                q4ex[tag].append((key, ' '.join(calls[:i + 1]), r['rule'].split('conventions/')[-1] if r['rule'] else None, r['expl']))
        # ---- strain
        if not oc or ' ' not in oc:
            continue
        con, dec = oc.split()[:2]
        level, strain = int(con[0]), con[1]
        if strain not in 'NCD':
            continue
        side = 'NS' if dec in 'NS' else 'EW'
        si = 0 if side == 'NS' else 1
        fitM = [M for M in 'SH' if L[side[0]][HI[M]] + L[side[1]][HI[M]] >= 8]
        if not fitM:
            continue
        status = 'hidden'
        first = None
        last = None
        for i, r in enumerate(R):
            c = r['caller'][0]
            if c not in side:
                continue
            me = SEATS.index(c)
            pa = (me + 2) % 4
            if i == 0:
                continue
            before = R[i - 1]['seats']
            for M in fitM:
                if before[me]['len'][SI[M]][0] + before[pa]['len'][SI[M]][0] >= 8:
                    status = 'agreed'
                    first = first or (i, M)
                elif L[c][HI[M]] + before[pa]['len'][SI[M]][0] >= 8:
                    last = (i, M)
                    if status == 'hidden':
                        status = 'private'
                        first = (i, M)
            if status == 'agreed':
                break
        p = b.get('par') or {}
        err = None
        if p.get('ours_errors'):
            err = p['ours_errors']['contract'][si]
        elif b['our_contract'] == b['reference_contract'] and p.get('reference_errors'):
            err = p['reference_errors']['contract'][si]
        # DD potential: M contract by the longer hand at the "equivalent" level
        dd = b['dd']['tricks']
        tl = 4 if (strain == 'N' and level >= 3) or (strain in 'CD' and level >= 4) else (3 if (strain == 'N' and level == 2) or (strain in 'CD' and level == 3) else 2)
        if level >= 6:
            tl = level
        best = None
        vul = side_vul(b['vul'], side)
        for M in fitM:
            dh = side[0] if L[side[0]][HI[M]] >= L[side[1]][HI[M]] else side[1]
            t = dd[SEATS.index(dh)][DDS[M]]
            s = score(tl, M, t, vul)
            best = s if best is None else max(best, s)
        ours_t = dd[SEATS.index(dec)][DDS[strain]]
        dbl = 1 if oc.endswith('X') or 'X' in con[2:] else 0
        ours_s = score(level, strain, ours_t, vul, dbl)
        g = imp(best - ours_s)
        cls[status] += 1
        cost[status] += err or 0
        pot[status] += g
        if status == 'private' and last:
            i, M = last
            r = R[i]
            # position: auction prefix before the call, our side bare, theirs in ()
            pre = []
            for j in range(i):
                cj = R[j]['caller'][0]
                pre.append(R[j]['call'] if cj in side else '(' + R[j]['call'] + ')')
            while pre and pre[0] == '(P)':
                pre.pop(0)
            k = ' '.join(pre) + ' -> ' + r['call']
            pos[k][0] += 1
            pos[k][1] += err or 0
            pos[k][2] += g
            posc[k][f"{M}{L[r['caller'][0]][HI[M]]}"] += 1
            dump.append({'key': key, 'deal': b['deal'], 'dealer': b['dealer'], 'vul': b['vul'], 'ours': ' '.join(calls),
                         'bba': b['reference_contract'], 'oc': oc, 'pos': k, 'M': M, 'err': err, 'pot': g,
                         'rule': r['rule'].split('conventions/')[-1] if r['rule'] else None, 'expl': r['expl']})
    n = sum(cls.values())
    print(f"NT/minor contracts with an 8+ major fit between the hands: {n}")
    print(f"{'status':10} {'boards':>7} {'our contract errors':>20} {'per bd':>7} {'DD gain of M':>13} {'per bd':>7}")
    for s in ('agreed', 'private', 'hidden'):
        c = cls[s]
        print(f"{s:10} {c:7} {cost[s]:20} {cost[s]/max(c,1):7.2f} {pot[s]:13} {pot[s]/max(c,1):7.2f}")
    print()
    print("private: the last call made by a hand that knew (top by boards)")
    print(f"{'boards':>6} {'errors':>7} {'DDgain':>7}  position -> the call made  [M and caller's length]")
    for k, v in sorted(pos.items(), key=lambda kv: -kv[1][0])[:a.top]:
        print(f"{v[0]:6} {v[1]:7} {v[2]:7}  {k}   {dict(posc[k].most_common(3))}")
    print()
    print("4NT calls by kind / trump state / public agreement (shown mins >= 8) / partner showed 6")
    for k, v in q4.most_common():
        print(f"{v:6}  {k}")
        for ex in q4ex[k][:2]:
            print(f"          {ex}")
    if a.dump:
        json.dump(dump, open(a.dump, 'w'), indent=0)



if __name__ == '__main__':
    if len(sys.argv) < 2 or sys.argv[1] not in ('explain', 'report'):
        sys.exit(__doc__)
    {'explain': explain_main, 'report': report_main}[sys.argv[1]](sys.argv[2:])
