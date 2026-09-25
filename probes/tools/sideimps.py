#!/usr/bin/env python3
"""Compare two `rbb compare --json` runs board by board, from the side
that made the first call that differs: IMPs (double dummy) to that side,
and the change in distance from par.

    probes/tools/sideimps.py BASE.json VARIANT.json [TOP]

Distance from par treats both sides alike: a competitive call that
works moves the result away from par and counts as worse. For
competitive decisions judge by the IMPs to the side that acted.
"""
import json,sys,collections
a=json.load(open(sys.argv[1]))['boards']; b=json.load(open(sys.argv[2]))['boards']
key=lambda x:(x['scenario'],x['board']); A={key(x):x for x in a}
def imp(x):
  t=[20,50,90,130,170,220,270,320,370,430,500,600,750,900,1100,1300,1500,1750,2000,2250,2500,3000,3500,4000]
  n=sum(1 for v in t if abs(x)>=v); return n if x>=0 else -n
def s(c):
  if isinstance(c,str): return {'Pass':'P','Double':'X','Redouble':'XX'}.get(c,c)
  x=c['Bid']; return f"{x['level']}{x['strain'][0]}"
def scores(x,y):
  # (our NS score, par NS) for board x, borrowing BBA's score from y when x matched BBA
  p=x['par'] or {}
  if p.get('ours_ns') is not None: return p['ours_ns'], p['par_ns']
  q=y['par'] or {}
  if x['our_contract']==x['reference_contract'] and q.get('reference_ns') is not None: return q['reference_ns'], q['par_ns']
  return None, None
tot=0; n=0; by=collections.Counter(); cnt=collections.Counter(); dist=0; skipped=0
for x in b:
  y=A.get(key(x))
  if not y: continue
  ox=[s(c) for c in x['ours']]; oy=[s(c) for c in y['ours']]
  if ox==oy: continue
  sx,px=scores(x,y); sy,py=scores(y,x)
  if sx is None or sy is None: skipped+=1; continue
  i=0
  while i<min(len(ox),len(oy)) and ox[i]==oy[i]: i+=1
  seat='NESW'[('NESW'.index(x['dealer'][0])+i)%4]
  sg=1 if seat in 'NS' else -1
  d=imp(sg*(sx-sy))
  pre=ox[:i]
  while pre and pre[0]=='P': pre.pop(0)
  k=' '.join(pre[:3]); by[k]+=d; cnt[k]+=1; tot+=d; n+=1
  pn=px if px is not None else py
  dist += imp(abs(sy-pn)) - imp(abs(sx-pn))
print(f'boards changed {n} (skipped {skipped}); IMPs to the side that changed its call: {tot:+}; par-distance change: {dist:+}')
top=int(sys.argv[3]) if len(sys.argv)>3 else 6
for k,v in sorted(by.items(),key=lambda z:z[1])[:top]: print(f'  {v:+5} {cnt[k]:5}  {k}')
for k,v in sorted(by.items(),key=lambda z:-z[1])[:top]: print(f'  {v:+5} {cnt[k]:5}  {k}')
