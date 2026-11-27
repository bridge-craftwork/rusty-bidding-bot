import json, subprocess, math, sys
ROOT="/Volumes/Express2T/Development/GitHub/rusty-bidding-bot"
OUT=sys.argv[1]
cov=json.load(open(f"{ROOT}/.rbb-cache/gib-tree/coverage-random100k.json"))
agr=json.load(open(f"{ROOT}/.rbb-cache/gib-tree/agreed-20k-final.json"))
C1,C2,C3,GR="#2563eb","#ea580c","#16a34a","#9ca3af"
def svg(w,h,body): return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="Helvetica" font-size="11">{body}</svg>'
def legend(items,x,y):
    s=""
    for i,(c,t) in enumerate(items):
        s+=f'<rect x="{x}" y="{y+i*16-9}" width="10" height="10" fill="{c}"/><text x="{x+15}" y="{y+i*16}">{t}</text>'
    return s
def axes(x0,y0,w,h,yt):
    s=f'<line x1="{x0}" y1="{y0}" x2="{x0+w}" y2="{y0}" stroke="#333"/><line x1="{x0}" y1="{y0}" x2="{x0}" y2="{y0-h}" stroke="#333"/>'
    for v in yt:
        y=y0-h*v
        s+=f'<line x1="{x0}" y1="{y}" x2="{x0+w}" y2="{y}" stroke="#e5e7eb"/><text x="{x0-6}" y="{y+4}" text-anchor="end">{int(v*100)}%</text>'
    return s
# chart 1: coverage curve
tops=cov['top']; W,H=620,300; x0,y0,w,h=60,250,420,220
xs=[math.log10(t['n']) for t in tops]; lo,hi=math.log10(400),math.log10(25000)
X=lambda v: x0+w*(v-lo)/(hi-lo)
b=axes(x0,y0,w,h,[0,.25,.5,.75,1])
for t in tops: b+=f'<text x="{X(math.log10(t["n"]))}" y="{y0+16}" text-anchor="middle">{t["n"]:,}</text>'
b+=f'<text x="{x0+w/2}" y="{y0+34}" text-anchor="middle">positions asked of GIB (most frequent first)</text>'
for key,c,lab in [('calls',C1,'all calls'),('uncontested',C3,'uncontested'),('contested',C2,'contested')]:
    pts=" ".join(f'{X(math.log10(t["n"]))},{y0-h*t[key]}' for t in tops)
    b+=f'<polyline points="{pts}" fill="none" stroke="{c}" stroke-width="2.5"/>'
    for t in tops: b+=f'<circle cx="{X(math.log10(t["n"]))}" cy="{y0-h*t[key]}" r="3" fill="{c}"/>'
    last=tops[-1]; b+=f'<text x="{X(math.log10(last["n"]))+6}" y="{y0-h*last[key]+4}" fill="{c}">{last[key]*100:.0f}%</text>'
b+=legend([(C1,'all calls'),(C3,'uncontested calls'),(C2,'contested calls')],500,60)
chart1=svg(W,H,b)
# chart 2: by call number at top 20k
bc=tops[-1]['by_call']; n=14; W,H=620,300; x0,y0,w,h=60,250,500,220
b=axes(x0,y0,w,h,[0,.25,.5,.75,1]); bw=w/n
for i in range(1,n+1):
    d=bc[str(i)]; xi=x0+bw*(i-1)
    for j,(k,c) in enumerate([('uncontested',C3),('contested',C2)]):
        v=d[k]
        if v is None: continue
        b+=f'<rect x="{xi+4+j*(bw-8)/2}" y="{y0-h*v}" width="{(bw-8)/2-1}" height="{h*v}" fill="{c}"/>'
    b+=f'<text x="{xi+bw/2}" y="{y0+16}" text-anchor="middle">{i}</text>'
b+=f'<text x="{x0+w/2}" y="{y0+34}" text-anchor="middle">call number in the auction (top 20,000 positions)</text>'
b+=legend([(C3,'uncontested'),(C2,'contested')],x0+w-110,30)
chart2=svg(W,H,b)
# chart 3: breakdown of all calls
total=cov['calls']; top20=867534
parts=[("no range difference",853587-225477,C3),("range difference",225477,C2),("different meaning / one side only",3885,"#a855f7"),("off an agreed path",10062,"#64748b"),("beyond the top 20,000 positions",total-top20,GR)]
W,H=620,170; x0,w=20,580; b=""; x=x0
for lab,v,c in parts:
    ww=w*v/total; b+=f'<rect x="{x}" y="20" width="{ww}" height="40" fill="{c}"/>'; x+=ww
for i,(lab,v,c) in enumerate(parts):
    yy=85+(i%3)*22; xx=x0+(i//3)*300
    b+=f'<rect x="{xx}" y="{yy-9}" width="10" height="10" fill="{c}"/><text x="{xx+15}" y="{yy}">{lab}: {v:,} ({100*v/total:.1f}%)</text>'
chart3=svg(W,H,b)
# chart 4: progress
runs=[("top 5,000: before",350431,GR),("top 5,000: after pass 1",181255,C1),("top 20,000: before pass 2",230784,GR),("top 20,000: after pass 2",225477,C1),("top 20,000: gib style on",214315,C3)]
W,H=620,190; x0,w=190,380; b=""; mx=360000
for i,(lab,v,c) in enumerate(runs):
    y=15+i*34
    b+=f'<text x="{x0-8}" y="{y+16}" text-anchor="end">{lab}</text><rect x="{x0}" y="{y}" width="{w*v/mx}" height="22" fill="{c}"/><text x="{x0+w*v/mx+6}" y="{y+16}">{v:,}</text>'
chart4=svg(W,H,b)
# chart 5: agreed share by call number (of all calls at top 20k)
ab=agr['by_call']; n=14; W,H=620,300; x0,y0,w,h=60,250,500,220
b=axes(x0,y0,w,h,[0,.25,.5,.75,1]); bw=w/n
for i in range(1,n+1):
    d=ab[str(i)]; v=d['agreed']/d['all']; xi=x0+bw*(i-1)
    b+=f'<rect x="{xi+6}" y="{y0-h*v}" width="{bw-12}" height="{h*v}" fill="{C1}"/><text x="{xi+bw/2}" y="{y0+16}" text-anchor="middle">{i}</text><text x="{xi+bw/2}" y="{y0-h*v-4}" text-anchor="middle" font-size="9">{v*100:.0f}</text>'
b+=f'<text x="{x0+w/2}" y="{y0+34}" text-anchor="middle">call number: share of calls compared (agreed path and meaning)</text>'
chart5=svg(W,H,b)
body=subprocess.run(["pandoc",f"{ROOT}/docs/reviews/2026-11-27-lower-levels.md","-t","html"],capture_output=True,text=True,check=True).stdout
charts=f'''<h2>Charts: coverage of the GIB comparison</h2>
<figure>{chart1}<figcaption>Figure 1. Share of the 971,231 calls in 100,000 random deals that fall at the N most frequent positions.</figcaption></figure>
<figure>{chart2}<figcaption>Figure 2. Coverage by call number at the top 20,000 positions. Early calls are almost all covered; contested auctions thin out from the fifth call.</figcaption></figure>
<figure>{chart5}<figcaption>Figure 3. Of the calls at the top 20,000 positions, the share compared (fully agreed path, same kind of meaning), by call number.</figcaption></figure>
<figure>{chart3}<figcaption>Figure 4. Every call in the 100,000 deals, by what the comparison could say about it (after pass 2, default style).</figcaption></figure>
<figure>{chart4}<figcaption>Figure 5. Call-boards whose range differs from GIB's, by run.</figcaption></figure>'''
# put charts after section 1
i=body.find('<h2 id="what-the-bottom-review-changed">')
body=body[:i]+charts+body[i:] if i>0 else body+charts
html=f'''<!doctype html><html><head><meta charset="utf-8"><title>Lower levels: status</title><style>
@page {{ size: A4; margin: 18mm 16mm; }}
body {{ font-family: Helvetica, Arial, sans-serif; font-size: 10.5pt; line-height: 1.4; color:#111; }}
h1 {{ font-size: 18pt; }} h2 {{ font-size: 13.5pt; margin-top: 18pt; border-bottom: 1px solid #ddd; }}
table {{ border-collapse: collapse; margin: 6pt 0; }} td,th {{ border: 1px solid #ccc; padding: 3pt 6pt; }} th {{ background:#f3f4f6; }}
td:last-child {{ text-align: right; }} figure {{ margin: 10pt 0; page-break-inside: avoid; }} figcaption {{ font-size: 9pt; color:#444; }}
code {{ font-size: 9pt; }}
</style></head><body>{body}</body></html>'''
open(OUT.replace(".pdf",".html"),"w").write(html)
subprocess.run(["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome","--headless","--disable-gpu","--no-pdf-header-footer","--print-to-pdf="+OUT,"file://"+OUT.replace(".pdf",".html")],check=True,capture_output=True)
print("ok",OUT)
