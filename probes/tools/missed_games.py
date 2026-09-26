#!/usr/bin/env python3
"""Contested boards (the opponents bid in our auction) where the opening
side stops in a partscore although game makes double dummy (`missed`),
or bids a game that fails (`over`), grouped by the auction's shape
(suits renamed a, b, c in order of appearance).

    probes/tools/missed_games.py COMPARE.json [missed|over] [TOP] [MIN_HCP]

MIN_HCP keeps only missed games where the opening side holds that many
HCP between them (25 = games a player should bid).
"""
import json,sys,collections
d=json.load(open(sys.argv[1]))['boards']
want=sys.argv[2] if len(sys.argv)>2 else 'missed'
def s(c):
  if isinstance(c,str): return {'Pass':'P','Double':'X','Redouble':'XX'}.get(c,c)
  x=c['Bid']; return f"{x['level']}{x['strain'][0] if x['strain']!='NoTrump' else 'N'}"
def shape(o):
  m={}; out=[]
  for c in o:
    if c[0].isdigit() and c[1] in 'CDHS':
      if c[1] not in m: m[c[1]]='abcd'[len(m)]
      out.append(c[0]+m[c[1]])
    else: out.append(c)
  return ' '.join(out)
ST='CDHSN'; NEED={'C':11,'D':11,'H':10,'S':10,'N':9}
g=collections.Counter(); n=collections.Counter(); ex={}
for b in d:
  if not b.get('dd'): continue
  o=[s(c) for c in b['ours']]; dl='NESW'.index(b['dealer'][0])
  seats=['NESW'[(dl+i)%4] for i in range(len(o))]
  bids=[(c,st) for c,st in zip(o,seats) if c[0].isdigit()]
  if not bids: continue
  opener=bids[0][1]; side='NS' if opener in 'NS' else 'EW'
  if not any(st not in side for c,st in bids): continue   # uncontested
  c=b['our_contract']
  if not c or c.startswith('P'): continue
  lvl=int(c[0]); strn=c[1]; decl=c.split()[-1]
  if decl not in side: continue
  H={'A':4,'K':3,'Q':2,'J':1}
  hands=dict(zip('NESW',b['deal'][2:].split()))
  hcp=sum(H.get(ch,0) for x in side for ch in hands[x])
  if want=='missed' and hcp<int(sys.argv[4] if len(sys.argv)>4 else 0): continue
  T=b['dd']['tricks']; rows={x:T['NESW'.index(x)] for x in side}
  tricks=max(r[ST.index(strn)] for r in rows.values())
  game=(strn=='N' and lvl>=3) or (strn in 'HS' and lvl>=4) or (strn in 'CD' and lvl>=5)
  best_game=max((rows[x][ST.index(t)]-NEED[t], t) for x in side for t in ST)
  pre=o[:]
  while pre and pre[0]=='P': pre.pop(0)
  while pre and pre[-1]=='P': pre.pop()
  k=shape(pre)
  if want=='missed' and not game and best_game[0]>=0 and lvl<6:
    g[k]+=1; n[k]+=1; ex.setdefault(k,(b['scenario'],b['board'],c,best_game[1]))
  if want=='over' and game and lvl<6 and tricks < lvl+6:
    g[k]+=1; n[k]+=1; ex.setdefault(k,(b['scenario'],b['board'],c))
print(sum(n.values()),'boards')
for k in sorted(n,key=lambda k:-n[k])[:int(sys.argv[3]) if len(sys.argv)>3 else 25]: print(f'{n[k]:5}  {k:36} {ex[k]}')
