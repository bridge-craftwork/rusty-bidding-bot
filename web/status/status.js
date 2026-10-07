// The status page (status.html): draws status.json as a grid of tiles,
// one per part of the system, with a details dialog per tile. Plain DOM,
// no libraries. State lives in the fragment: #q=drury&status=gap&tile=<id>
// &flat=1 (never a query string, as on the main page).

import { STATUS, BY_CODE, about, band } from './legend.js'

const $ = (id) => document.getElementById(id)
const state = { q: '', status: '', flat: false, tile: '' }
let data = null

function el(tag, attrs = {}, ...kids) {
  const e = document.createElement(tag)
  for (const [k, v] of Object.entries(attrs)) {
    if (v == null || v === false) continue
    if (k === 'class') e.className = v
    else if (k === 'text') e.textContent = v
    else if (k.startsWith('on')) e.addEventListener(k.slice(2), v)
    else e.setAttribute(k, v === true ? '' : v)
  }
  for (const kid of kids.flat()) if (kid != null && kid !== false) e.append(kid.nodeType ? kid : document.createTextNode(String(kid)))
  return e
}

const pct = (x) => `${(x * 100).toFixed(0)}%`
const signed = (x, d = 2) => `${x >= 0 ? '+' : '−'}${Math.abs(x).toFixed(d)}`

function glyph(code, extra = '') {
  const s = BY_CODE[code]
  return el('span', { class: `glyph st-${code} ${extra}`, 'aria-hidden': 'true', text: s?.glyph ?? '?' })
}

function readFragment() {
  const p = new URLSearchParams(location.hash.slice(1))
  state.q = p.get('q') ?? ''
  state.status = BY_CODE[p.get('status')] ? p.get('status') : ''
  state.flat = p.get('flat') === '1'
  state.tile = p.get('tile') ?? ''
}

function writeFragment() {
  const p = new URLSearchParams()
  if (state.q) p.set('q', state.q)
  if (state.status) p.set('status', state.status)
  if (state.flat) p.set('flat', '1')
  if (state.tile) p.set('tile', state.tile)
  const h = p.toString()
  history.replaceState(null, '', h ? `#${h}` : location.pathname + location.search)
}

// ── Summary and legend ────────────────────────────────────────────────

function renderSummary() {
  const h = data.header
  const c = h.corpus
  const box = $('summary')
  box.replaceChildren()
  const stat = (n, label, title) => el('div', { class: 'stat', title }, el('b', { text: n }), el('span', { text: label }))
  const covered = h.cards.filter((x) => x.coverage === 100).length
  box.append(
    el('div', { class: 'stats' },
      stat(data.tiles.length, 'items'),
      stat(h.modules, 'modules', `${h.rules} rules`),
      stat(h.tests.toLocaleString('en'), 'test cases'),
      stat(`${covered}/${h.cards.length}`, 'stock cards fully read', h.cards.map((x) => `${x.card} ${x.coverage}%`).join('\n')),
      c ? stat(pct(c.ns_agree), 'NS calls as BBA', `All calls ${pct(c.calls_agree)}; EW ${pct(c.ew_agree)}; identical auctions ${pct(c.auctions_match)}; same contract ${pct(c.contracts_match)}`) : null,
      c ? stat(signed(c.errors_per_board), 'IMPs/board vs BBA', 'Errors vs BBA per board, each side charged with its own errors (an overbid taken as doubled); negative: more errors than BBA') : null,
    ),
    el('p', { class: 'hint' },
      `Engine ${data.engine}, commit `,
      el('a', { href: `https://github.com/bridge-craftwork/rusty-bidding-bot/commit/${data.commit}`, text: data.commit }),
      ` (${data.commit_date}); data of ${data.generated}`,
      c ? `; ${c.boards.toLocaleString('en')} boards in ${c.scenarios} Practice-Bidding-Scenarios scenarios` : '',
      h.self_ab ? `; ${h.self_ab.runs} self A/B runs (${h.self_ab.date})` : '',
      `; conventions and card fields from convention-card ${data.card_spec.tag}.`),
  )
}

function counts(tiles) {
  const n = {}
  for (const t of tiles) n[t.status] = (n[t.status] ?? 0) + 1
  return n
}

function renderLegend() {
  const n = counts(data.tiles)
  const list = $('legend')
  list.replaceChildren(...STATUS.map((s) => el('li', {},
    el('button', {
      class: `legend-item${state.status === s.code ? ' on' : ''}`, 'data-status': s.code,
      'aria-pressed': state.status === s.code ? 'true' : 'false',
      title: about(s.code, data.thresholds),
      onclick: () => { state.status = state.status === s.code ? '' : s.code; update() },
    }, glyph(s.code), el('span', { class: 'lab', text: s.label }), el('span', { class: 'n', text: n[s.code] ?? 0 })))))
  const sel = $('status-filter')
  if (sel.options.length === 1) {
    for (const s of STATUS) sel.append(el('option', { value: s.code, text: `${s.glyph} ${s.label}` }))
  }
  sel.value = state.status
  const t = data.thresholds
  $('thresholds').textContent =
    `vs BBA, per item over its scenarios with par as the yardstick: good at errors ≥ ${t.good.errors_per_board} IMPs/board and ≥ ${pct(t.good.ns_agree)} of North-South calls agreeing; ` +
    `poor below ${t.poor.errors_per_board} IMPs/board or ${pct(t.poor.ns_agree)}; fair between. Judgment items: our boards in their par class over BBA's (good ≤ ${t.class_ratio.good}, fair ≤ ${t.class_ratio.fair}). ` +
    `Catch-alls: per 1,000 boards (good under ${t.per_thousand.good}, fair under ${t.per_thousand.fair}). ` +
    'A/B: our engine with the convention against without it, by errors. A tile shows its convention\'s status; each dot below is one treatment (a card setting).'
}

// ── The grid ──────────────────────────────────────────────────────────

function matches(t) {
  if (state.status && t.status !== state.status && !(t.treatments ?? []).some((x) => x.status === state.status)) return false
  if (!state.q) return true
  const q = state.q.toLowerCase()
  const hay = [t.name, t.id, t.summary, ...(t.treatments ?? []).map((x) => `${x.label} ${x.field ?? ''}`),
    ...(t.variants ?? []).map((v) => v.name), ...(t.modules ?? []).map((m) => m.name)].join(' ').toLowerCase()
  return hay.includes(q)
}

function tileTitle(t) {
  const s = BY_CODE[t.status]
  return [`${t.name}: ${s.label}`, t.summary, t.why].filter(Boolean).join('\n')
}

function tileButton(t) {
  const trs = t.treatments ?? []
  const b = el('button', {
    class: `tile st-${t.status}`, 'data-id': t.id, title: tileTitle(t),
    'aria-label': `${t.name}, ${BY_CODE[t.status].label}${t.level ? `, level ${t.level}` : ''}${trs.length ? `, ${trs.length} treatments` : ''}`,
    onclick: () => openDetail(t.id),
  },
  el('span', { class: 'row1' }, glyph(t.status), el('span', { class: 'name', text: t.name }),
    t.level ? el('span', { class: 'lvl', text: t.level, title: `Level ${t.level}` }) : null),
  trs.length ? el('span', { class: 'dots', 'aria-hidden': 'true' },
    trs.slice(0, 18).map((x) => el('span', { class: `dot st-${x.status}`, title: `${x.label}: ${BY_CODE[x.status].label}` })),
    trs.length > 18 ? el('span', { class: 'more', text: `+${trs.length - 18}` }) : null) : null)
  return el('li', {}, b)
}

function bar(tiles) {
  const n = counts(tiles)
  const total = tiles.length || 1
  return el('span', { class: 'bar', role: 'img', 'aria-label': STATUS.filter((s) => n[s.code]).map((s) => `${n[s.code]} ${s.label}`).join(', ') },
    STATUS.filter((s) => n[s.code]).map((s) => el('span', { class: `seg st-${s.code}`, style: `flex-grow:${n[s.code] / total}`, title: `${n[s.code]} ${s.label}` })))
}

function renderGrid() {
  const grid = $('grid')
  grid.replaceChildren()
  let shown = 0
  for (const sec of data.sections) {
    const all = data.tiles.filter((t) => t.section === sec.id)
    if (!all.length) continue
    const tiles = all.filter(matches)
    shown += tiles.length
    if (!tiles.length) continue
    const box = el('section', { class: 'sec', 'aria-labelledby': `h-${sec.id}` },
      el('div', { class: 'sechead' },
        el('h2', { id: `h-${sec.id}`, text: sec.title }),
        el('span', { class: 'count', text: tiles.length === all.length ? `${all.length}` : `${tiles.length} of ${all.length}` }),
        bar(all)))
    const leveled = !state.flat && ['constructive', 'competitive', 'precision'].includes(sec.id)
    if (leveled) {
      const groups = new Map()
      for (const t of tiles) {
        const b = band(t.level)
        if (!groups.has(b.id)) groups.set(b.id, { b, tiles: [] })
        groups.get(b.id).tiles.push(t)
      }
      for (const { b, tiles: ts } of groups.values()) {
        box.append(el('h3', { class: 'band', text: b.name }), el('ul', { class: 'tiles' }, ts.map(tileButton)))
      }
    } else {
      const ts = state.flat ? [...tiles].sort((a, b) => a.name.localeCompare(b.name)) : tiles
      box.append(el('ul', { class: 'tiles' }, ts.map(tileButton)))
    }
    grid.append(box)
  }
  if (!shown) grid.append(el('p', { class: 'panel empty', text: 'Nothing matches.' }))
}

// ── Details ───────────────────────────────────────────────────────────

function table(head, rows) {
  return el('div', { class: 'tscroll' }, el('table', { class: 'dt' },
    el('thead', {}, el('tr', {}, head.map((h) => el('th', { text: h })))),
    el('tbody', {}, rows.map((r) => el('tr', {}, r.map((c) => el('td', {}, c)))))))
}

/** A dotted path with a line-break chance after each dot. */
function breakable(text) {
  return text.split('.').flatMap((part, i, all) => (i < all.length - 1 ? [`${part}.`, el('wbr')] : [part]))
}

function openDetail(id) {
  const t = data.tiles.find((x) => x.id === id)
  if (!t) return
  state.tile = id
  writeFragment()
  $('detail-title').textContent = t.name
  const body = $('detail-body')
  body.replaceChildren()
  const s = BY_CODE[t.status]
  body.append(el('p', { class: `status-line st-${t.status}` }, glyph(t.status), el('b', { text: s.label }), t.why ? ` · ${t.why}` : ''))
  const meta = [t.level ? `Level ${t.level}` : null, t.id.includes('/') && t.source !== 'curated' ? t.id : null].filter(Boolean)
  if (meta.length) body.append(el('p', { class: 'hint' }, meta.join(' · ')))
  if (t.summary) body.append(el('p', { text: t.summary }))
  if (t.partial) body.append(el('p', { class: 'note' }, el('b', { text: 'Not yet: ' }), t.partial))
  if (t.variants?.length) {
    body.append(el('h3', { text: 'Shown with it' }), el('ul', { class: 'plain' },
      t.variants.map((v) => el('li', {}, el('b', { text: v.name }), v.level ? ` (level ${v.level})` : '', v.summary ? `: ${v.summary}` : ''))))
  }
  if (t.treatments?.length) {
    body.append(el('h3', { text: `Treatments (${t.treatments.length})` }), table(['', 'Treatment', 'Card setting', 'Status'],
      t.treatments.map((x) => [glyph(x.status), el('span', { text: x.label, title: x.why ?? '' }),
        x.field ? el('code', {}, breakable(x.option ? `${x.field} = ${x.option}` : x.field)) : '—',
        el('span', { title: x.why ?? '' }, BY_CODE[x.status]?.label ?? x.status, x.why && !x.why.startsWith('a rule') ? el('span', { class: 'why', text: ` · ${x.why}` }) : null)])))
  }
  if (t.measure) {
    const m = t.measure
    body.append(el('h3', { text: 'Measure' }))
    if (m.kind === 'par class') {
      body.append(el('p', {}, `Boards in ${m.classes.map((c) => c.replace(/_/g, ' ')).join(', ')}: ours ${m.ours.toLocaleString('en')}, BBA ${m.bba.toLocaleString('en')} (ratio ${m.ratio}) over ${m.boards.toLocaleString('en')} corpus boards.`))
    } else {
      body.append(el('p', {}, `${m.count.toLocaleString('en')} in ${m.boards.toLocaleString('en')} corpus boards: ${m.per_thousand} per 1,000.`))
    }
    if (t.examples?.length) body.append(table(['Count', 'Auction', 'Call'], t.examples.map((e) => [String(e.count), el('code', { text: e.auction }), e.call])))
  }
  if (t.scenarios?.length) {
    const b = t.bba
    body.append(el('h3', { text: 'Against BBA' }))
    if (b) body.append(el('p', {}, `Pooled: ${b.boards} boards, errors ${signed(b.errors_per_board)} IMPs/board, NS calls agreeing ${pct(b.ns_agree)}.`))
    body.append(table(['', 'Scenario', 'Boards', 'NS calls', 'IMPs/bd', 'NS card'],
      t.scenarios.map((x) => [glyph(x.status), x.name, String(x.boards), pct(x.ns_agree), signed(x.errors_per_board), x.ns_card ?? ''])))
  }
  if (t.ab?.length) {
    body.append(el('h3', { text: 'Self A/B' }), table(['', 'Run', 'Switches on', 'Boards changed', 'Verdict', 'z'],
      t.ab.map((r) => [r.status ? glyph(r.status) : '', r.name, el('code', { text: Object.entries(r.changes).map(([k, v]) => `${k}=${v}`).join(' ') }),
        String(r.changed), r.verdict, r.changed ? (r.z >= 0 ? '+' : '') + r.z.toFixed(1) : '—'])))
  }
  if (t.modules?.length) {
    body.append(el('h3', { text: 'Rules' }), table(['Module', 'Rules', 'Tests', 'Notes'],
      t.modules.map((m) => [el('a', { href: m.url, title: m.title ?? '', text: m.name }), String(m.rules), String(m.tests),
        m.notes ? el('a', { href: m.notes, text: 'notes' }) : '—'])))
  }
  if (t.see?.length) {
    body.append(el('h3', { text: 'Read more' }), el('ul', { class: 'plain' }, t.see.map((r) => el('li', {},
      r.url ? el('a', { href: r.url, text: r.title }) : el('i', { text: r.title }), r.by ? `, ${r.by}` : '', r.site ? ` (${r.site})` : ''))))
  }
  const d = $('detail')
  if (!d.open) d.showModal()
  $('detail-close').focus()
}

function closeDetail() {
  const id = state.tile
  state.tile = ''
  writeFragment()
  if ($('detail').open) $('detail').close()
  document.querySelector(`.tile[data-id="${CSS.escape(id)}"]`)?.focus()
}

// ── Wiring ────────────────────────────────────────────────────────────

function update() {
  writeFragment()
  $('q').value = state.q
  $('by-level').checked = !state.flat
  renderLegend()
  renderGrid()
}

async function main() {
  readFragment()
  try {
    const r = await fetch(new URL('./status.json', import.meta.url))
    if (!r.ok) throw new Error(`status.json: HTTP ${r.status}`)
    data = await r.json()
  } catch (e) {
    $('grid').replaceChildren(el('p', { class: 'panel empty', text: `Could not load the status data (${e.message}).` }))
    return
  }
  renderSummary()
  update()
  $('q').addEventListener('input', (e) => { state.q = e.target.value.trim(); writeFragment(); renderGrid() })
  $('status-filter').addEventListener('change', (e) => { state.status = e.target.value; update() })
  $('by-level').addEventListener('change', (e) => { state.flat = !e.target.checked; update() })
  $('detail-close').addEventListener('click', closeDetail)
  // Escape closes the dialog natively; tidy the fragment and focus after
  // it, unless another tile has opened since (the event is queued).
  $('detail').addEventListener('close', () => { if (state.tile && !$('detail').open) closeDetail() })
  $('detail').addEventListener('click', (e) => { if (e.target === $('detail')) closeDetail() })
  window.addEventListener('hashchange', () => {
    readFragment(); update()
    if (state.tile) openDetail(state.tile); else if ($('detail').open) $('detail').close()
  })
  if (state.tile) openDetail(state.tile)
  window.statusData = data
}

main()
