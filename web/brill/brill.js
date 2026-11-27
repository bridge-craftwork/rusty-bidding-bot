// The Brill dashboard: draws brill-dd.json (probes/tools/brill_dd_data.py)
// as inline SVG. No libraries.

const $ = (id) => document.getElementById(id)
const SVG = 'http://www.w3.org/2000/svg'
const DAY = 86400000

const state = { events: 'both', window: 7, points: true, actual: true }

function el(name, attrs = {}, parent) {
  const e = document.createElementNS(SVG, name)
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v)
  if (parent) parent.appendChild(e)
  return e
}
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c])
const t = (date) => Date.parse(date + 'T12:00:00Z')
const mean = (xs) => xs.reduce((a, b) => a + b, 0) / xs.length
function stderr(xs) {
  if (xs.length < 2) return null
  const m = mean(xs)
  return Math.sqrt(xs.reduce((a, x) => a + (x - m) ** 2, 0) / (xs.length - 1) / xs.length)
}
function fmt(v, scoring, signed = scoring === 'IMP') {
  if (v == null || Number.isNaN(v)) return '–'
  const s = v.toFixed(1)
  return (signed && v > 0 ? '+' : '') + s + (scoring === 'MP' && !signed ? '%' : '')
}
const fmtDelta = (v) => (v == null ? '–' : (v > 0 ? '+' : '') + v.toFixed(1))

// The versions in force on `date` (each deployment's `in_force`, from
// brill_dd_data.py: Rusty's rbb, seat and card, and BEN's version).
function inForce(data, date) {
  let now = {}
  for (const d of data.deployments) if (d.date <= date) now = d.in_force || now
  return now
}
const rustyName = (v) => (v.rbb ? `rbb ${v.rbb}` : 'rbb ?') + (v.seat ? ` · seat ${v.seat}` : '') + (v.card ? ` · ${v.card}` : '')
const benName = (v) => (v.ben ? `BEN ${v.ben}` : 'BEN ?')
// The BEN versions in force over [start, end): the one at the start and any
// BEN change inside.
function bensOver(data, start, end) {
  const out = [inForce(data, start).ben]
  for (const d of data.deployments) if (d.kind === 'play' && d.date > start && d.date < end) out.push(d.ben || d.in_force?.ben)
  return [...new Set(out.filter(Boolean))]
}

// The sessions of one scoring type, one point per day (the mean of the
// day's chosen events).
function daily(data, scoring) {
  const keep = (r) => r.scoring === scoring && r.dd != null &&
    (state.events === 'both' || (state.events === 'strongest') === r.event.endsWith('-strongest'))
  const byDay = new Map()
  for (const r of data.rows) if (keep(r)) (byDay.get(r.date) || byDay.set(r.date, []).get(r.date)).push(r)
  return [...byDay.entries()].sort(([a], [b]) => (a < b ? -1 : 1)).map(([date, rs]) => ({
    date, rows: rs, dd: mean(rs.map((r) => r.dd)), actual: mean(rs.map((r) => r.actual)),
    cardplay: mean(rs.map((r) => r.cardplay)),
  }))
}

function rolling(days, key, w) {
  return days.map((d, i) => {
    const from = t(d.date) - (w - 1) * DAY
    const win = days.slice(0, i + 1).filter((x) => t(x.date) >= from)
    return mean(win.map((x) => x[key]))
  })
}

// Deployment periods: split by the bidding engine's deployments only.
function periods(data) {
  const first = data.rows.length ? data.rows.map((r) => r.date).sort()[0] : null
  const bids = data.deployments.filter((d) => d.kind === 'bidding')
  const out = []
  if (first && (!bids.length || bids[0].date > first)) out.push({ start: first, label: 'first day(s)', dep: null })
  for (const d of bids) out.push({ start: d.date, label: d.label, dep: d })
  out.forEach((p, i) => { p.end = out[i + 1]?.start ?? '9999-12-31' })
  return out
}

function periodStats(data, scoring, ps) {
  return ps.map((p) => {
    const rs = data.rows.filter((r) => r.scoring === scoring && r.dd != null && r.date >= p.start && r.date < p.end &&
      (state.events === 'both' || (state.events === 'strongest') === r.event.endsWith('-strongest')))
    if (!rs.length) return { ...p, n: 0 }
    const dd = rs.map((r) => r.dd)
    const days = new Set(rs.map((r) => r.date))
    return {
      ...p, n: rs.length, days: days.size, last: [...days].sort().pop(),
      dd: mean(dd), se: stderr(dd), actual: mean(rs.map((r) => r.actual)), cardplay: mean(rs.map((r) => r.cardplay)),
    }
  })
}

// One chart. kind 'score' (dd and actual) or 'cardplay' (bars).
function draw(box, data, scoring, kind, range, ps) {
  const days = daily(data, scoring)
  box.textContent = ''
  if (!days.length) { box.textContent = 'No sessions.'; return }
  const W = Math.max(320, box.clientWidth || 800)
  const H = kind === 'score' ? (W < 560 ? 240 : 300) : 130
  const m = { l: 40, r: 12, t: kind === 'score' ? 46 : 8, b: 22 }
  const svg = el('svg', { viewBox: `0 0 ${W} ${H}`, role: 'img',
    'aria-label': `${scoring} ${kind === 'score' ? 'double-dummy and actual scores' : 'cardplay contribution'} by day` }, box)
  const x0 = t(range[0]) - DAY / 2, x1 = t(range[1]) + DAY / 2
  const X = (d) => m.l + ((t(d) - x0) / (x1 - x0)) * (W - m.l - m.r)
  const base = scoring === 'MP' && kind === 'score' ? 50 : 0
  const keys = kind === 'score' ? (state.actual ? ['dd', 'actual'] : ['dd']) : ['cardplay']
  let vals = days.flatMap((d) => keys.map((k) => d[k]))
  if (!state.points && kind === 'score') vals = keys.flatMap((k) => rolling(days, k, state.window))
  let lo = Math.min(base, ...vals), hi = Math.max(base, ...vals)
  const pad = (hi - lo) * 0.06 || 1
  lo -= pad; hi += pad
  const Y = (v) => m.t + (1 - (v - lo) / (hi - lo)) * (H - m.t - m.b)

  // Grid and axes.
  const g = el('g', { class: 'grid' }, svg)
  const span = hi - lo
  const step = [1, 2, 5, 10, 20, 25, 50].map((s) => s * (span > 100 ? 10 : 1)).find((s) => span / s <= (kind === 'score' ? 6 : 4)) || 100
  for (let v = Math.ceil(lo / step) * step; v <= hi; v += step) {
    el('line', { x1: m.l, x2: W - m.r, y1: Y(v), y2: Y(v) }, g)
    const tx = el('text', { x: m.l - 6, y: Y(v) + 3.5, 'text-anchor': 'end' }, svg)
    tx.textContent = scoring === 'MP' && kind === 'score' ? v + '%' : (v > 0 ? '+' : '') + v
  }
  el('line', { class: 'zero', x1: m.l, x2: W - m.r, y1: Y(base), y2: Y(base) }, svg)
  // Weekly date ticks.
  for (let d = t(range[0]); d <= t(range[1]); d += 7 * DAY) {
    const iso = new Date(d).toISOString().slice(0, 10)
    const tx = el('text', { x: X(iso), y: H - 6, 'text-anchor': 'middle' }, svg)
    tx.textContent = iso.slice(5)
  }

  // Deployments. Markers of one day sit side by side (a BEN change and a
  // bidding change on the same day both show); labels take the first of
  // three rows with room.
  const rowEnd = [-Infinity, -Infinity, -Infinity]
  const sameDay = new Map()
  for (const dep of data.deployments) {
    const k = sameDay.get(dep.date) ?? 0
    sameDay.set(dep.date, k + 1)
    const gx = el('g', { class: 'deploy ' + dep.kind }, svg)
    const x = X(dep.date) - (W - m.l - m.r) / ((x1 - x0) / DAY) / 2 + k * 4
    el('line', { x1: x, x2: x, y1: m.t - (kind === 'score' ? 4 : 0), y2: H - m.b }, gx)
    // Labels need room; on a narrow screen the hover names the deployment.
    if (kind === 'score' && W >= 700) {
      const text = dep.kind === 'play' ? dep.label : dep.label.replace(/^seat v[\d.]+\+? \/ /, '')
      const row = rowEnd.findIndex((end) => x + 3 > end + 6)
      if (row < 0) continue
      const lab = el('text', { x: x + 3, y: m.t - 6 - row * 12 }, gx)
      lab.textContent = text
      rowEnd[row] = x + 3 + text.length * 5.6
    }
  }

  if (kind === 'score') {
    // Period means (dashed steps).
    const stats = periodStats(data, scoring, ps)
    for (const k of keys) {
      for (const p of stats) {
        if (!p.n) continue
        const a = X(p.start) - 4, b = X(p.last) + 4
        el('line', { class: 'mean ' + k, x1: a, x2: b, y1: Y(p[k]), y2: Y(p[k]) }, svg)
      }
    }
    if (state.points) {
      for (const k of keys) for (const d of days) el('circle', { class: 'pt ' + k, cx: X(d.date), cy: Y(d[k]), r: 4 }, svg)
    }
    if (state.window > 1 || !state.points) {
      for (const k of keys) {
        const r = rolling(days, k, state.window)
        el('path', { class: 'roll ' + k, d: days.map((d, i) => `${i ? 'L' : 'M'}${X(d.date).toFixed(1)},${Y(r[i]).toFixed(1)}`).join('') }, svg)
      }
    }
  } else {
    const bw = Math.max(2, Math.min(10, ((W - m.l - m.r) / ((x1 - x0) / DAY)) - 3))
    for (const d of days) {
      const y = Y(d.cardplay), y0 = Y(0)
      el('rect', { class: 'bar', x: X(d.date) - bw / 2, y: Math.min(y, y0), width: bw, height: Math.max(1, Math.abs(y - y0)), rx: 2 }, svg)
    }
    if (state.window > 1) {
      const r = rolling(days, 'cardplay', state.window)
      el('path', { class: 'roll cp', d: days.map((d, i) => `${i ? 'L' : 'M'}${X(d.date).toFixed(1)},${Y(r[i]).toFixed(1)}`).join('') }, svg)
    }
  }

  // Hover: a crosshair and the day's sessions.
  const cross = el('line', { class: 'cross', y1: m.t, y2: H - m.b, visibility: 'hidden' }, svg)
  const hit = el('rect', { class: 'hit', x: m.l, y: 0, width: W - m.l - m.r, height: H }, svg)
  const rolls = Object.fromEntries(['dd', 'actual', 'cardplay'].map((k) => [k, rolling(days, k, state.window)]))
  const tip = $('tip')
  const show = (ev) => {
    const r = svg.getBoundingClientRect()
    const px = ((ev.clientX - r.left) / r.width) * W
    let best = 0
    days.forEach((d, i) => { if (Math.abs(X(d.date) - px) < Math.abs(X(days[best].date) - px)) best = i })
    const d = days[best]
    cross.setAttribute('x1', X(d.date)); cross.setAttribute('x2', X(d.date)); cross.setAttribute('visibility', 'visible')
    const p = ps.find((q) => d.date >= q.start && d.date < q.end)
    const deps = data.deployments.filter((x) => x.date === d.date)
    const name = (e) => data.events.find((x) => x.id === e)?.name ?? e
    const v = inForce(data, d.date)
    tip.innerHTML = `<b>${d.date}</b> <span class="muted">${esc(p?.label ?? '')}</span><br>` +
      `<span class="muted">${esc(rustyName(v))}; ${esc(benName(v))}</span><br>` +
      d.rows.map((r) => `${esc(name(r.event))}: <span class="sw" style="background:var(--series-dd)"></span>DD ${fmt(r.dd, scoring)} · ` +
        `<span class="sw" style="background:var(--series-actual)"></span>actual ${fmt(r.actual, scoring)} ` +
        `<span class="muted">(cardplay ${fmtDelta(r.cardplay)}; ${r.rank} of ${r.field}${r.dd_source === 'field' ? '; DD from the field PBN' : ''})</span>`).join('<br>') +
      (state.window > 1 ? `<br><span class="muted">${state.window}-day mean: DD ${fmt(rolls.dd[best], scoring)}, actual ${fmt(rolls.actual[best], scoring)}, cardplay ${fmtDelta(rolls.cardplay[best])}</span>` : '') +
      deps.map((x) => `<br>Deployed: <b>${esc(x.label)}</b> — ${esc(x.what)}`).join('')
    tip.hidden = false
    const tw = tip.offsetWidth, th = tip.offsetHeight
    tip.style.left = Math.min(window.innerWidth - tw - 8, ev.clientX + 14) + 'px'
    tip.style.top = Math.max(8, ev.clientY - th - 12) + 'px'
  }
  hit.addEventListener('pointermove', show)
  hit.addEventListener('pointerdown', show)
  hit.addEventListener('pointerleave', () => { tip.hidden = true; cross.setAttribute('visibility', 'hidden') })
}

function legend(box) {
  const key = (cls, text, line) => `<span class="key"><svg width="22" height="10" aria-hidden="true">` +
    (line === 'dash' ? `<line x1="1" x2="21" y1="5" y2="5" stroke="var(${cls})" stroke-width="2" stroke-dasharray="6 4"/>`
      : line ? `<line x1="1" x2="21" y1="5" y2="5" stroke="var(${cls})" stroke-width="2"/>`
        : `<circle cx="11" cy="5" r="4" fill="var(${cls})"/>`) + `</svg>${text}</span>`
  const parts = [key('--series-dd', 'double dummy (bidding)'), key('--series-dd', `${state.window}-day mean`, true), key('--series-dd', 'period mean', 'dash')]
  if (state.actual) parts.push(key('--series-actual', 'actual'), key('--series-actual', `${state.window}-day mean`, true))
  parts.push(`<span class="key"><svg width="22" height="10" aria-hidden="true"><line x1="11" x2="11" y1="0" y2="10" stroke="var(--deploy)"/></svg>bidding deployment</span>`,
    `<span class="key"><svg width="22" height="10" aria-hidden="true"><line x1="11" x2="11" y1="0" y2="10" stroke="var(--deploy)" stroke-dasharray="2 3"/></svg>BEN (card play)</span>`)
  box.innerHTML = parts.join('')
}

function summary(data, ps) {
  const days = new Set(data.rows.map((r) => r.date))
  const cur = ps[ps.length - 1]
  const tile = (v, label) => `<div class="stat"><b>${v}</b><span>${label}</span></div>`
  const parts = [tile(days.size, `days, ${[...days].sort()[0]} to ${data.through}`), tile(data.rows.length, 'sessions of 8 boards'),
    tile(data.deployments.filter((d) => d.kind === 'bidding').length, 'bidding deployments')]
  for (const sc of ['IMP', 'MP']) {
    const st = periodStats(data, sc, ps).filter((p) => p.n)
    const now = st[st.length - 1], before = st[st.length - 2]
    if (!now) continue
    const delta = before ? now.dd - before.dd : null
    parts.push(tile(`${fmt(now.dd, sc)} <small class="delta ${delta > 0 ? 'up' : delta < 0 ? 'down' : ''}">${before ? fmtDelta(delta) : ''}</small>`,
      `${sc} double dummy, mean since ${esc(cur.start)} (${now.n} sessions)${before ? '; change from the period before' : ''}`))
  }
  $('summary').innerHTML = `<div class="stats">${parts.join('')}</div>`
}

function periodTable(data, ps) {
  const head = `<tr><th>Period</th><th>From</th><th>Rusty</th><th>BEN</th><th class="num">Days</th><th class="num">Sessions</th><th class="num">Double dummy</th>` +
    `<th class="num">± s.e.</th><th class="num">Δ DD</th><th class="num">Actual</th><th class="num">Cardplay</th></tr>`
  let html = ''
  for (const sc of ['IMP', 'MP']) {
    html += `<tr class="group"><th colspan="11">${sc} events ${sc === 'IMP' ? '(IMPs per session)' : '(percent)'}</th></tr>` + head
    let prev = null
    for (const p of periodStats(data, sc, ps)) {
      if (!p.n) continue
      html += `<tr><td>${esc(p.label)}</td><td>${p.start}</td>` +
        `<td>${esc(rustyName(inForce(data, p.start)))}</td><td>${esc(bensOver(data, p.start, p.end).join(', ') || '?')}</td>` +
        `<td class="num">${p.days}</td><td class="num">${p.n}</td>` +
        `<td class="num">${fmt(p.dd, sc)}</td><td class="num">${p.se == null ? '–' : p.se.toFixed(1)}</td>` +
        `<td class="num">${prev ? fmtDelta(p.dd - prev.dd) : ''}</td><td class="num">${fmt(p.actual, sc)}</td><td class="num">${fmtDelta(p.cardplay)}</td></tr>`
      prev = p
    }
  }
  $('periods').innerHTML = html
}

function deployTable(data) {
  $('deploys').innerHTML = `<tr><th>First day</th><th>Kind</th><th>Deployment</th><th>In force from that day</th><th>What changed</th><th>Date</th></tr>` +
    data.deployments.map((d) => `<tr><td>${d.date}</td><td>${d.kind === 'play' ? 'BEN (play)' : 'bidding'}</td><td>${esc(d.label)}</td>` +
      `<td>${esc(rustyName(d.in_force || {}))}<br>${esc(benName(d.in_force || {}))}</td>` +
      `<td class="wrap">${esc(d.what)}<br><span class="hint">${esc(d.evidence)}</span></td><td>${esc(d.confidence)}` +
      `${d.started_at ? `<br><span class="hint">${esc(d.started_at)}</span>` : ''}</td></tr>`).join('')
}

function sessionTable(data) {
  const name = Object.fromEntries(data.events.map((e) => [e.id, e.name]))
  $('sessions').innerHTML = `<tr><th>Date</th><th>Event</th><th class="num">Actual</th><th class="num">Double dummy</th>` +
    `<th class="num">Cardplay</th><th class="num">Rank</th><th>Declaring</th><th>Defending</th></tr>` +
    [...data.rows].reverse().map((r) => `<tr><td>${r.date}</td><td>${r.url ? `<a href="${esc(r.url)}">${esc(name[r.event])}</a>` : esc(name[r.event])}</td>` +
      `<td class="num">${fmt(r.actual, r.scoring)}</td><td class="num">${fmt(r.dd, r.scoring)}${r.dd_source === 'field' ? '*' : ''}</td>` +
      `<td class="num">${fmtDelta(r.cardplay)}</td><td class="num">${r.rank} of ${r.field}</td>` +
      `<td>${r.declaring ? `${fmtDelta(r.declaring.tricks)} over ${r.declaring.boards}` : ''}</td>` +
      `<td>${r.defending ? `${fmtDelta(r.defending.tricks)} over ${r.defending.boards}` : ''}</td></tr>`).join('') +
    `<tr><td colspan="8" class="hint">* the mail had no double-dummy line; scored from the field PBN by brill_report.py. ` +
    `Declaring and defending: tricks over double dummy, over that many boards.</td></tr>`
}

async function main() {
  let data
  try {
    const r = await fetch('brill/brill-dd.json', { cache: 'no-cache' })
    data = await r.json()
  } catch (e) {
    $('summary').textContent = 'Could not load brill/brill-dd.json: ' + e.message
    return
  }
  const ps = periods(data)
  const dates = data.rows.map((r) => r.date).sort()
  const range = [dates[0], data.through]
  const render = () => {
    summary(data, ps)
    for (const sc of ['IMP', 'MP']) {
      const id = sc.toLowerCase()
      legend($('legend-' + id))
      draw($('chart-' + id), data, sc, 'score', range, ps)
      draw($('cardplay-' + id), data, sc, 'cardplay', range, ps)
    }
    periodTable(data, ps)
  }
  for (const [id, key, conv] of [['events', 'events', String], ['window', 'window', Number]]) {
    $(id).addEventListener('change', (e) => { state[key] = conv(e.target.value); render() })
  }
  for (const id of ['points', 'actual']) $(id).addEventListener('change', (e) => { state[id] = e.target.checked; render() })
  deployTable(data)
  sessionTable(data)
  render()
  let pending
  window.addEventListener('resize', () => { clearTimeout(pending); pending = setTimeout(render, 120) })
  window.brillDashboard = { data, periods: ps, periodStats: (sc) => periodStats(data, sc, ps) }
}

main()
