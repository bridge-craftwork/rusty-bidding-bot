// The page: draws a Session (lib/tool.js) and turns clicks into Session
// calls. window.rustyBiddingBot drives the same Session, so a program and a
// person see and do the same things.

import init, * as rbb from './pkg/rbb_wasm.js'
import { Session, PARAMS, GLOBAL, DEFAULT_CARDS } from './lib/tool.js'
import { fetchManifest } from './lib/pbs.js'
import { readFragment, writeFragment } from './lib/fragment.js'
import { SEATS, SEAT_NAMES, SUITS, SUIT_SYMBOLS, hcp, partnership } from './lib/deal.js'

const $ = (id) => document.getElementById(id)
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]))
const SOURCE_URL = 'https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/'
const BC_CARD = 'BC-21-Intermediate'

await init()

async function loadDealer3() {
  const m = await import('./dealer3/dealer3_wasm.js')
  await m.default()
  return m
}

const extraCards = {}
try {
  const r = await fetch('cards/21_intermediate_card.json')
  if (r.ok) extraCards[BC_CARD] = { json: await r.json() }
} catch { /* the card list simply lacks it */ }

const session = new Session(rbb, { loadDealer3, extraCards })
const uploads = new Map() // select value -> card spec
let manifest = null
let source = 'scenario'
let selected = null // index of the call shown in detail
let lastHash = null
let loading = false

$('rules-id').textContent = `${session.info.rules_id} (${session.info.modules} modules, ${session.info.rules} rules; engine ${session.info.version})`

// ── Controls built from PARAMS ─────────────────────────────────────────

const LABELS = {
  rotate: { 0: 'No', 1: '1 seat', 2: '2 seats', 3: '3 seats' },
  view: { all: 'All hands', N: 'North only', E: 'East only', S: 'South only', W: 'West only' },
}
for (const el of document.querySelectorAll('select[data-param]')) {
  const spec = PARAMS.find((p) => p.name === el.dataset.param)
  if (!spec.values) continue
  if (spec.name === 'scoring') el.append(new Option("Board's (MP)", ''))
  for (const v of spec.values) el.append(new Option(LABELS[spec.name]?.[v] ?? String(v), String(v)))
}

function fillCardSelects() {
  for (const side of ['ns', 'ew']) {
    const el = $(`card-${side}`)
    const keep = el.value
    el.textContent = ''
    el.append(new Option("The scenario's card", 'scenario'))
    const stock = document.createElement('optgroup')
    stock.label = 'Practice-Bidding-Scenarios cards'
    for (const n of session.info.stock_cards) stock.append(new Option(n, n))
    el.append(stock)
    const bc = document.createElement('optgroup')
    bc.label = 'Bridge-Classroom cards'
    if (extraCards[BC_CARD]) bc.append(new Option('2/1 Intermediate', BC_CARD))
    el.append(bc)
    if (uploads.size) {
      const up = document.createElement('optgroup')
      up.label = 'Uploaded'
      for (const [v, spec] of uploads) up.append(new Option(spec.name ?? v, v))
      el.append(up)
    }
    const p = session.params[side]
    if (p && typeof p === 'object') {
      const v = `given:${side}`
      if (!uploads.has(v)) uploads.set(v, p)
      el.append(new Option(`${p.name ?? 'Given card'} (given)`, v))
      el.value = v
    } else {
      el.value = p ?? keep ?? 'scenario'
    }
  }
}

function cardValue(v) {
  return uploads.get(v) ?? v
}

function syncControls() {
  const p = session.params
  for (const el of document.querySelectorAll('[data-param]')) {
    const k = el.dataset.param
    if (k === 'ns' || k === 'ew' || k === 'xray') continue
    if (el.type === 'checkbox') el.checked = !!p[k]
    else el.value = p[k] == null ? '' : String(p[k])
  }
  fillCardSelects()
  const b = session.base
  if (b) {
    setSource(b.source === 'deal' ? 'deal' : b.source, { quiet: true })
    if (b.source === 'scenario') {
      if ($('scenario').value !== b.scenario && manifest) $('scenario').value = b.scenario
      $('board').value = b.board
    } else if (b.source === 'random') {
      $('seed').value = b.seed
      $('random-board-no').value = b.board
    } else if (b.source === 'script') {
      if ($('script').value !== b.script) $('script').value = b.script
      $('script-seed').value = b.seed
    }
  }
}

for (const el of document.querySelectorAll('[data-param]')) {
  if (el.dataset.param === 'xray') continue // drawn and wired by drawXray
  el.addEventListener('change', async () => {
    const k = el.dataset.param
    let v = el.type === 'checkbox' ? el.checked : el.value
    if (k === 'ns' || k === 'ew') v = cardValue(v)
    if (k === 'scoring' && v === '') v = null
    if (k === 'rotate') v = +v
    await session.setParams({ [k]: v })
  })
}

// ── Deal sources ───────────────────────────────────────────────────────

function setSource(name, { quiet = false } = {}) {
  source = name
  for (const b of document.querySelectorAll('[data-source]')) b.setAttribute('aria-selected', String(b.dataset.source === name))
  for (const s of document.querySelectorAll('.source')) s.hidden = s.dataset.for !== name
  if (!quiet && name === 'random' && session.base?.source !== 'random') dealRandom()
}
for (const b of document.querySelectorAll('[data-source]')) b.addEventListener('click', () => setSource(b.dataset.source))

async function load(input, params = {}) {
  loading = true
  selected = null
  render()
  try {
    await session.run(input, { stop: 0, ...params })
  } finally {
    loading = false
    syncControls()
    render()
  }
}

function dealRandom(seed) {
  seed ??= Math.floor(Math.random() * 2 ** 31)
  const board = Math.floor(Math.random() * 16) + 1
  return load({ random: true, seed, board })
}

$('deal-random').addEventListener('click', () => dealRandom())
$('scenario').addEventListener('change', () => {
  const name = $('scenario').value
  if (name) load({ scenario: name, board: 1 })
})
const loadBoard = (n) => {
  const name = $('scenario').value || session.base?.scenario
  if (!name) return
  const count = session.base?.count ?? 500
  n = ((n - 1) % count + count) % count + 1
  load({ scenario: name, board: n })
}
$('board').addEventListener('change', () => loadBoard(+$('board').value || 1))
$('prev-board').addEventListener('click', () => loadBoard((session.base?.board ?? 1) - 1))
$('next-board').addEventListener('click', () => loadBoard((session.base?.board ?? 0) + 1))
$('random-board').addEventListener('click', () => loadBoard(Math.floor(Math.random() * (session.base?.count ?? 500)) + 1))
$('scenario-script').addEventListener('click', async () => {
  const name = $('scenario').value || session.base?.scenario
  if (!name) return
  setSource('script', { quiet: true })
  await load({ scenarioScript: name, seed: 1 })
})
// The script as typed, or (unchanged) the scenario's by name, which keeps
// the link short.
function scriptInput(seed) {
  const b = session.base
  const text = $('script').value
  if (b?.scenarioScript && b.script === text) return { scenarioScript: b.scenarioScript, seed }
  return { script: text, seed }
}
$('deal-script').addEventListener('click', () => load(scriptInput(+$('script-seed').value || 1)))
$('next-script').addEventListener('click', () => {
  $('script-seed').value = (+$('script-seed').value || 1) + 1
  load(scriptInput(+$('script-seed').value))
})
$('deal-own').addEventListener('click', () => {
  const auction = $('deal-auction').value.trim()
  load({ deal: $('deal-text').value.trim(), dealer: $('deal-dealer').value, vul: $('deal-vul').value, ...(auction ? { auction } : {}) })
})

// Cards: upload and download.
$('card-file').addEventListener('change', async () => {
  const f = $('card-file').files[0]
  if (!f) return
  const text = await f.text()
  const name = f.name.replace(/\.(bbsa|json)$/i, '')
  let spec
  if (/\.json$/i.test(f.name) || /^\s*\{/.test(text)) {
    try {
      const j = JSON.parse(text)
      // Bridge-Classroom's export ({schema, name, card_data, ...}) or bare
      // card_data: the engine reads both, and ignores `_bbo_raw` and the
      // like with an info. The card's own name when it has one.
      spec = { json: j, name: j.name || j.card_data?.metadata?.name || j.metadata?.name || name }
    } catch (e) {
      session.diagnostics = [{ severity: 'error', message: `card file: ${f.name} is not JSON: ${e.message}` }]
      render()
      return
    }
  } else {
    spec = { bbsa: text, name }
  }
  const v = `upload:${name}`
  uploads.set(v, spec)
  const v2 = await session.validate(null, { ns: spec })
  session.diagnostics = v2.diagnostics.filter((d) => d.message.startsWith('cards.'))
  fillCardSelects()
  // Give it to the side whose select was last used: North-South by default.
  $('card-ns').value = v
  await session.setParams({ ns: spec })
  $('card-file').value = ''
})

function download(name, text, type) {
  const a = document.createElement('a')
  a.href = URL.createObjectURL(new Blob([text], { type }))
  a.download = name
  a.click()
  setTimeout(() => URL.revokeObjectURL(a.href), 1000)
}

async function exportCard(side, format) {
  const specs = await session.cardSpecs()
  // Read and written in the vocabulary of the rules the engine plays.
  const r = JSON.parse(rbb.exportCard(JSON.stringify({
    card: specs[side], engine: session.engine ?? undefined, exported_at: new Date().toISOString(),
  })))
  if (!r.ok) {
    session.addDiagnostics(r.diagnostics)
    render()
    return
  }
  const base = String(r.name).replace(/[^\w.-]+/g, '_')
  if (format === 'json') {
    // Bridge-Classroom's own export format: {schema, name, description, exportedAt, card_data}.
    download(`${base}.json`, JSON.stringify(r.bridge_classroom, null, 2) + '\n', 'application/json')
  } else {
    download(`${base}.bbsa`, r.bbsa, 'text/plain')
  }
}

// ── The auction controls ───────────────────────────────────────────────

$('bid-next').addEventListener('click', () => { session.bidNext(); selected = session.calls.length - 1; render() })
$('bid-end').addEventListener('click', () => session.bidTo('end'))
$('undo').addEventListener('click', () => { session.undo(); selected = null })
$('restart').addEventListener('click', () => { session.restart(); selected = null })
document.addEventListener('keydown', (e) => {
  if (e.target.closest('input, textarea, select') || e.metaKey || e.ctrlKey || e.altKey) return
  if (e.key === 'n') $('bid-next').click()
  else if (e.key === 'e') $('bid-end').click()
  else if (e.key === 'u') $('undo').click()
  else if (e.key === 'x') $('xray-toggle').click()
})
$('xray-toggle').addEventListener('click', () => session.setParams({ xray: session.params.xray ? 0 : 1 }))

const STRAINS = ['C', 'D', 'H', 'S', 'NT']
function legal(calls, dealerIdx) {
  let lastBid = -1
  let lastNonPass = null
  calls.forEach((c, i) => {
    if (/^[1-7]/.test(c)) lastBid = (+c[0]) * 5 + STRAINS.indexOf(c.slice(1))
    if (c !== 'Pass') lastNonPass = { c, i }
  })
  const opp = lastNonPass && (calls.length - lastNonPass.i) % 2 === 1
  return {
    bid: (level, s) => level * 5 + STRAINS.indexOf(s) > lastBid,
    X: !!(opp && /^[1-7]/.test(lastNonPass.c)),
    XX: !!(opp && lastNonPass.c === 'X'),
  }
}

function buildBidbox() {
  const box = $('bidbox')
  box.textContent = ''
  for (let level = 1; level <= 7; level++) {
    for (const s of STRAINS) {
      const b = document.createElement('button')
      b.dataset.call = `${level}${s}`
      b.innerHTML = fmtCall(`${level}${s}`)
      box.append(b)
    }
  }
  const other = document.createElement('div')
  other.className = 'other'
  for (const c of ['Pass', 'X', 'XX']) {
    const b = document.createElement('button')
    b.dataset.call = c
    b.textContent = c
    other.append(b)
  }
  box.append(other)
  box.addEventListener('click', (e) => {
    const c = e.target.closest('button')?.dataset.call
    if (!c) return
    const r = session.play(c)
    if (!r.ok) {
      session.addDiagnostics(r.diagnostics)
      render()
    } else {
      selected = session.calls.length - 1
      render()
    }
  })
}
buildBidbox()

// ── Drawing ────────────────────────────────────────────────────────────

function fmtCall(c) {
  if (!c) return ''
  const m = /^([1-7])(C|D|H|S|NT?)$/.exec(c)
  if (!m) return esc(c)
  const s = m[2].startsWith('N') ? 'N' : m[2]
  return s === 'N' ? `${m[1]}NT` : `${m[1]}<span class="s-${s}">${SUIT_SYMBOLS[s]}</span>`
}

function fmtContract(c) {
  if (!c || c === 'Pass') return 'Passed out'
  const m = /^([1-7])(NT|N|S|H|D|C)(X{0,2})$/.exec(c)
  return m ? fmtCall(m[1] + (m[2] === 'N' ? 'NT' : m[2])) + m[3] : esc(c)
}

function vulFor(seat, vul) {
  return vul === 'All' || (vul === 'NS' && partnership(seat) === 'ns') || (vul === 'EW' && partnership(seat) === 'ew')
}

function drawHands(out) {
  const view = session.params.view
  for (const el of document.querySelectorAll('.hand')) {
    const seat = el.dataset.seat
    const hand = out.hands?.[seat]
    const shown = hand && (view === 'all' || view === seat || out.complete)
    el.classList.toggle('hidden', !shown)
    const who = `${SEAT_NAMES[seat]}${hand && shown ? ` · ${hcp(hand)} HCP` : ''}`
    if (!hand) {
      el.innerHTML = ''
    } else if (!shown) {
      el.innerHTML = `<div class="who">${who}</div><div class="suits">13 cards</div>`
    } else {
      const suits = hand.split('.')
      el.innerHTML = `<div class="who">${who}</div><div class="suits">${SUITS.map((s, i) =>
        `<div class="suit"><span class="s-${s}">${SUIT_SYMBOLS[s]}</span> ${esc(suits[i] || '—').replace(/T/g, '10')}</div>`).join('')}</div>`
    }
  }
  const c = $('centre')
  if (!out.hands) {
    c.innerHTML = loading ? 'Dealing…' : ''
    return
  }
  c.innerHTML = SEATS.map((s) => `<span class="v ${s}${vulFor(s, out.vul) ? ' vul' : ''}${s === out.dealer ? ' dealer' : ''}${s === out.next ? ' next' : ''}">${s}</span>`).join('') +
    `<span>${out.board ? `Board ${out.board}<br>` : ''}Dealer ${out.dealer}<br>${out.vul === 'None' ? 'None vul' : out.vul === 'All' ? 'All vul' : out.vul + ' vul'}</span>`
}

function drawHead(out) {
  const h = $('deal-head')
  if (loading) {
    h.innerHTML = '<span class="title">Dealing…</span>'
    return
  }
  if (!out.hands) {
    h.innerHTML = '<span class="sub">Choose a deal.</span>'
    return
  }
  let title = 'Your deal'
  if (out.source === 'scenario') title = manifest?.scenarios[out.scenario]?.label ?? out.scenario
  else if (out.source === 'random') title = 'Random deal'
  else if (out.source === 'script') title = 'Dealer script'
  const sub = out.source === 'scenario' ? (manifest?.scenarios[out.scenario]?.title ?? '') : ''
  h.innerHTML = `<span class="title">${esc(title)}</span>` +
    (sub ? `<span class="sub">${esc(sub)}</span>` : '') +
    `<span class="sub">N-S: ${esc(out.cards.ns)} · E-W: ${esc(out.cards.ew)} · ${esc(out.scoring)}${out.rotate ? ` · rotated ${out.rotate}` : ''}</span>`
}

function drawAuction(out) {
  const body = $('auction').querySelector('tbody')
  const heads = $('auction').querySelectorAll('th')
  const order = ['W', 'N', 'E', 'S']
  heads.forEach((th, i) => th.classList.toggle('vul', !!out.vul && vulFor(order[i], out.vul)))
  body.textContent = ''
  if (!out.hands) return
  const cells = Array(order.indexOf(out.dealer)).fill(null)
  out.calls.forEach((c, i) => cells.push({ c, i }))
  if (!out.complete) cells.push({ next: true })
  for (let r = 0; r < cells.length; r += 4) {
    const tr = document.createElement('tr')
    for (let k = 0; k < 4; k++) {
      const td = document.createElement('td')
      const cell = cells[r + k]
      if (cell?.next) {
        td.innerHTML = '<span class="qmark">?</span>'
      } else if (cell) {
        const { c, i } = cell
        const b = document.createElement('button')
        b.className = `callbtn by-${c.by}${c.alert ? ' alerted' : ''}${c.noRule && c.call !== 'Pass' ? ' norule' : ''}${selected === i ? ' selected' : ''}`
        b.innerHTML = fmtCall(c.call) +
          (c.alert ? `<span class="mark">${c.alert.kind === 'announce' ? 'A' : '!'}</span>` : '') +
          (c.noRule && c.call !== 'Pass' ? '<span class="mark gap">?</span>' : '')
        b.title = [c.alert ? `${c.alert.kind === 'announce' ? 'Announced' : 'Alert'}${c.alert.text ? ': ' + c.alert.text : ''}` : '', c.meaning ?? c.why ?? ''].filter(Boolean).join('\n')
        b.addEventListener('click', () => { selected = selected === i ? null : i; render() })
        td.append(b)
      }
      tr.append(td)
    }
    body.append(tr)
  }
}

function drawDetail(out) {
  const el = $('call-detail')
  const c = selected != null ? out.calls[selected] : null
  if (!c) {
    el.hidden = true
    return
  }
  const raw = session.calls[selected]
  if (out.xray) {
    // The x-ray under the auction says all of this and more.
    el.hidden = true
    return
  }
  const rows = detailRows(c)
  let cands = ''
  if (raw?.candidates?.length) {
    cands = '<details class="cands"><summary>Calls the engine considered</summary><ul>' +
      raw.candidates.slice(0, 20).map((k) => `<li class="${k.outcome === 'chosen' ? 'chosen' : ''}">${fmtCall(k.call)}: ${esc(k.explanation)} <em>(${esc(k.outcome)})</em></li>`).join('') +
      '</ul></details>'
  }
  el.innerHTML = `<dl>${rows.map(([k, v]) => `<dt>${k}</dt><dd>${v}</dd>`).join('')}</dl>${cands}`
  el.hidden = false
}

const byText = (by) => by === 'engine' ? "the engine's call" : by === 'you' ? 'your call' : 'given'

function ruleLink(r) {
  if (!r) return ''
  const short = String(r.file).replace(/^.*?conventions\//, '')
  return `<a href="${SOURCE_URL}${esc(r.file)}#L${r.line}" target="_blank" rel="noopener" title="${esc(r.module)} · ${esc(r.file)}:${r.line}">${esc(short)}:${r.line}</a>`
}

/** The rows of a call's detail: what it says, what it shows, its rule. */
function detailRows(c) {
  const rows = [
    ['Call', `${SEAT_NAMES[c.seat]}: ${fmtCall(c.call)} (${byText(c.by)})`],
  ]
  if (c.alert) rows.push([c.alert.kind === 'announce' ? 'Announced' : 'Alerted', esc(c.alert.text ?? '(alert)')])
  if (c.why) rows.push(['Why', esc(c.why)])
  if (c.meaning && c.meaning !== c.why) rows.push(['Shows the table', esc(c.meaning)])
  if (c.shows) rows.push(['Hand known', esc(c.shows)])
  if (c.noRule && c.call !== 'Pass') rows.push(['Gap', 'No rule explains this call: outside the engine\'s system here.'])
  else if (c.noRule && c.by === 'engine') rows.push(['Note', 'No rule applied, so the engine passed.'])
  if (c.rule) rows.push(['Rule', `<a href="${SOURCE_URL}${esc(c.rule.file)}#L${c.rule.line}" target="_blank" rel="noopener">${esc(c.rule.module)} · ${esc(c.rule.file)}:${c.rule.line}</a>`])
  return rows
}

// ── The x-ray: the workbench's view of a call ─────────────────────────

const FULL = { hcp: 37, len: 13 }
function fmtRange(r, full) {
  if (!r) return ''
  if (r.min <= 0 && r.max >= full) return '<span class="unk">·</span>'
  return r.min === r.max ? String(r.min) : `${r.min}–${r.max}`
}

function handLine(hand) {
  return hand.split('.').map((cards, i) => `<span class="s-${SUITS[i]}">${SUIT_SYMBOLS[SUITS[i]]}</span>${esc(cards || '—').replace(/T/g, '10')}`).join(' ')
}

/** What every seat had shown, one row a seat (West first, as the auction); `mark` is the caller. */
function knowledgeTable(pos, mark) {
  if (!pos?.knowledge) return ''
  const order = ['W', 'N', 'E', 'S']
  const rows = order.map((s) => {
    const k = pos.knowledge[s]
    if (!k) return ''
    const bal = k.balanced === true ? 'yes' : k.balanced === false ? 'no' : '<span class="unk">·</span>'
    const shown = k.shown?.length
      ? `<details><summary>${k.shown.length}</summary><ul>${k.shown.map((x) => `<li><code>${esc(x)}</code></li>`).join('')}</ul></details>` : ''
    return `<tr${s === mark ? ' class="caller"' : ''}><th>${s}</th><td>${fmtRange(k.hcp, FULL.hcp)}</td>${SUITS.map((x) => `<td>${fmtRange(k.lengths?.[x], FULL.len)}</td>`).join('')}<td>${bal}</td><td class="shown">${shown}</td></tr>`
  }).join('')
  return `<div class="xscroll"><table class="xt know"><thead><tr><th>Seat</th><th>HCP</th>${SUITS.map((x) => `<th><span class="s-${x}">${SUIT_SYMBOLS[x]}</span></th>`).join('')}<th>Bal.</th><th>Shown</th></tr></thead><tbody>${rows}</tbody></table></div>`
}

function sidesLine(pos) {
  if (!pos?.sides) return ''
  return `<p class="sides"><span><strong>N-S</strong> ${esc(pos.sides.ns.summary)}</span><span><strong>E-W</strong> ${esc(pos.sides.ew.summary)}</span></p>`
}

function outcomeClass(o) {
  return o === 'chosen' ? 'chosen' : o === 'outranked' ? 'outranked' : 'fails'
}

/** One call under the x-ray. */
function xrayCall(out, x) {
  const c = out.calls[x.index]
  const view = session.params.view
  const secret = view !== 'all' && view !== x.seat && !out.complete
  const parts = []
  const engine = x.by === 'engine'
    ? ''
    : x.engineCall
      ? ` · the engine would bid ${fmtCall(x.engineCall)}${x.engineCall === x.call ? ' too' : ''}`
      : ''
  // The head names the call: its rows from "why" on.
  parts.push(`<dl>${detailRows(c).slice(1).map(([k, v]) => `<dt>${k}</dt><dd>${v}</dd>`).join('')}</dl>`)
  if (secret) {
    parts.push(`<p class="hint">${SEAT_NAMES[x.seat]}'s hand is hidden (Show: ${SEAT_NAMES[view]} only), and so are the rules weighed for it. What the table knows is below.</p>`)
  } else {
    parts.push(`<p class="holds">${SEAT_NAMES[x.seat]} holds ${handLine(x.hand)} · ${hcp(x.hand)} HCP${engine}</p>`)
    if (x.candidates.length) {
      parts.push('<h4>Candidates, best-ranked first</h4>')
      parts.push(`<div class="xscroll"><table class="xt cands"><thead><tr><th>Call</th><th title="Priority: a higher priority wins outright">Prio</th><th title="Descriptiveness: the share of the hands the caller could hold that the call rules out; higher says more">Descr</th><th>Outcome</th><th>Meaning</th><th>Rule</th></tr></thead><tbody>` +
        x.candidates.map((k) => `<tr class="${outcomeClass(k.outcome)}${k.call === x.call ? ' made' : ''}"><td class="call">${fmtCall(k.call)}</td><td class="num">${k.priority}</td><td class="num">${Number(k.descriptiveness).toFixed(3)}</td><td class="outcome">${esc(k.outcome)}</td><td class="meaning">${esc(k.explanation)}</td><td class="rule">${ruleLink(k.rule)}</td></tr>`).join('') +
        '</tbody></table></div>')
    } else {
      parts.push('<p class="hint">No rule applies here for this hand.</p>')
    }
    if (!x.offered && x.call !== 'Pass') parts.push(`<p class="gap">No rule offers ${fmtCall(x.call)} here.</p>`)
    for (const w of x.warnings) parts.push(`<p class="warn">warning: ${esc(w)}</p>`)
  }
  parts.push('<h4>What each seat had shown before the call</h4>')
  parts.push(knowledgeTable(x.before, x.seat))
  parts.push(sidesLine(x.before))
  return parts.join('')
}

function xrayHead(x) {
  return `Call ${x.index + 1} · ${SEAT_NAMES[x.seat]} ${fmtCall(x.call)} <span class="by">(${byText(x.by)})</span>`
}

function drawXray(out) {
  const el = $('xray')
  const mode = session.params.xray
  $('xray-toggle').setAttribute('aria-pressed', String(!!mode))
  el.hidden = !mode || !out.hands
  if (el.hidden) return
  const all = mode === 'all'
  const list = out.xray?.calls ?? []
  const bar = `<div class="xbar"><h3>X-ray</h3><label class="check"><input type="checkbox" data-param="xray" ${all ? 'checked' : ''}> Every call</label></div>`
  let body
  if (!list.length) {
    body = '<p class="hint">No calls yet. Once a call is made, the x-ray shows every rule the engine weighed for it, what each seat had shown and the forcing state.</p>'
  } else if (all) {
    body = list.map((x) => `<details class="xcall" open><summary>${xrayHead(x)}</summary>${xrayCall(out, x)}</details>`).join('')
  } else {
    const x = list[selected ?? list.length - 1] ?? list[list.length - 1]
    body = `<div class="xcall"><p class="xhead">${xrayHead(x)}${selected == null ? ' <span class="hint">the last call; click a call to x-ray it</span>' : ''}</p>${xrayCall(out, x)}</div>`
  }
  el.innerHTML = bar + body
  el.querySelector('input[data-param=xray]').addEventListener('change', (e) => session.setParams({ xray: e.target.checked ? 'all' : 1 }))
}

/** BBA's auction as the engine reads it: the workbench's table. */
function bbaReading(out) {
  const rows = out.xray?.bba
  if (!rows?.length) return ''
  return '<h4>How the engine reads BBA\'s auction</h4><div class="xscroll"><table class="xt reading"><thead><tr><th>#</th><th>Seat</th><th>BBA</th><th>BBA\'s note</th><th>The engine\'s reading</th><th>Engine would bid</th></tr></thead><tbody>' +
    rows.map((r, i) => {
      const read = r.reading?.explanation ? esc(r.reading.explanation) : '<span class="unk">no rule</span>'
      const known = r.reading?.knowledge?.summary ? `<span class="known">${esc(r.reading.knowledge.summary)}</span>` : ''
      const same = r.engineCall === r.call
      return `<tr><td class="num">${i + 1}</td><th>${r.seat}</th><td class="call">${fmtCall(r.call)}</td><td>${esc(r.note ?? '')}</td><td class="meaning">${read} ${ruleLink(r.reading?.rule)}${known}</td><td class="call ${same ? 'same' : 'differs'}">${r.engineCall ? fmtCall(r.engineCall) : ''}</td></tr>`
    }).join('') + '</tbody></table></div>'
}

function drawMeanings(out) {
  const el = $('meanings')
  el.hidden = !session.params.meanings || !out.calls.length
  if (el.hidden) return
  el.innerHTML = out.calls.map((c) => {
    const text = c.meaning ?? c.why ?? ''
    const tag = c.alert ? `<span class="alert-tag">${c.alert.kind === 'announce' ? 'Announced' : 'Alert'}${c.alert.text ? ': ' + esc(c.alert.text) : ''}</span>` : ''
    const gap = c.noRule && c.call !== 'Pass' ? '<span class="gap">no rule explains this call</span>' : ''
    return `<li><span class="seat">${c.seat}</span><span>${fmtCall(c.call)}</span><span>${esc(text)}${tag}${gap}</span></li>`
  }).join('')
}

function drawBidbox(out) {
  const wrap = $('bidbox-wrap')
  const open = !!out.hands && !out.complete
  wrap.classList.toggle('busy', !open)
  if (!open) return
  const l = legal(out.calls.map((c) => c.call), SEATS.indexOf(out.dealer))
  for (const b of $('bidbox').querySelectorAll('button')) {
    const c = b.dataset.call
    b.disabled = c === 'X' ? !l.X : c === 'XX' ? !l.XX : c === 'Pass' ? false : !l.bid(+c[0], c.slice(1))
  }
}

function scoreText(ns) {
  if (ns === 0) return '0'
  return ns > 0 ? `N-S +${ns}` : `E-W +${-ns}`
}

function drawResult(out) {
  const el = $('result')
  el.hidden = !out.complete
  if (!out.complete) return
  const parts = []
  const by = out.declarer ? ` by ${SEAT_NAMES[out.declarer]}` : ''
  let line = `<span class="contract">${fmtContract(out.contract)}${by}</span>`
  if (out.dd?.result?.tricks != null) {
    const level = parseInt(out.contract, 10)
    const diff = out.dd.result.tricks - (level + 6)
    line += `<span>Double dummy: ${out.dd.result.tricks} tricks (${diff === 0 ? 'made' : diff > 0 ? `+${diff}` : diff}), ${scoreText(out.dd.result.score_ns)}</span>`
  }
  if (out.dd?.par) line += `<span>Par ${scoreText(out.dd.par.score_ns)}${out.dd.par.contracts.length ? ` (${esc(out.dd.par.contracts.join('; '))})` : ''}</span>`
  parts.push(`<div class="result-head">${line}</div>`)
  if (out.dd?.tricks) {
    const strains = ['NT', 'S', 'H', 'D', 'C']
    const cs = /^([1-7])(NT|N|S|H|D|C)/.exec(out.contract ?? '')
    const cStrain = cs ? (cs[2] === 'N' ? 'NT' : cs[2]) : null
    parts.push(`<table class="dd"><thead><tr><th></th>${strains.map((s) => `<th>${s === 'NT' ? 'NT' : `<span class="s-${s}">${SUIT_SYMBOLS[s]}</span>`}</th>`).join('')}</tr></thead><tbody>` +
      ['N', 'S', 'E', 'W'].map((seat) => `<tr><th>${seat}</th>${strains.map((s) => {
        const t = out.dd.tricks[seat][s]
        const mark = seat === out.declarer && s === cStrain ? ' class="made"' : ''
        return `<td${mark}>${t > 6 ? t - 6 : '–'}</td>`
      }).join('')}</tr>`).join('') + '</tbody></table><p class="hint">Levels each declarer makes double dummy (tricks − 6).</p>')
  } else if (session.params.dd) {
    parts.push('<p class="hint">Solving…</p>')
  }
  if (out.bba) {
    const calls = out.bba.auction.map((c) => fmtCall(c.call) + (c.note ? `<span class="note" title="${esc(c.note)}">*</span>` : '')).join(' ')
    let bbaLine = `<p><strong>BBA</strong> bid this deal: <span class="auction-line">${calls}</span></p>`
    if (out.bba.contract) {
      const cm = /^([1-7])(NT|N|S|H|D|C)/.exec(out.bba.contract)
      const t = cm && out.bba.declarer && out.dd?.tricks ? out.dd.tricks[out.bba.declarer][cm[2] === 'N' ? 'NT' : cm[2]] : null
      bbaLine += `<p>BBA's contract: ${fmtContract(out.bba.contract)}${out.bba.declarer ? ` by ${SEAT_NAMES[out.bba.declarer]}` : ''}${t != null ? `, ${t} tricks double dummy` : ''}.</p>`
    }
    const notes = out.bba.auction.filter((c) => c.note)
    if (notes.length) bbaLine += `<p class="hint">${notes.map((c) => `${fmtCall(c.call)}: ${esc(c.note)}`).join(' · ')}</p>`
    parts.push(`<div class="cmp">${bbaLine}${bbaReading(out)}</div>`)
  }
  el.innerHTML = parts.join('')
}

function drawCoverage(out) {
  const el = $('coverage')
  const cov = out.coverage
  if (!cov) {
    el.innerHTML = ''
    return
  }
  const side = (k, label) => {
    const c = cov[k]
    if (!c) return ''
    const on = c.read.length + c.ignored.length
    const pct = Math.round(100 * c.score)
    const flag = c.ignored.length
      ? `<span class="flag">${c.ignored.length} setting${c.ignored.length === 1 ? '' : 's'} not played</span>`
      : '<span class="ok">every setting played</span>'
    const list = (items, title) => items.length
      ? `<details><summary>${title} (${items.length})</summary><ul>${items.map((x) => `<li>${esc(x)}</li>`).join('')}</ul></details>` : ''
    return `<div class="cov"><div><span class="name">${label}: ${esc(c.name)}</span></div>
      <div>The rules read ${c.read.length} of the ${on} bidding settings this card switches on (${pct}%): ${flag}.</div>
      ${list(c.ignored, 'Conventions and treatments the engine does not play')}
      ${list(c.unmapped, 'BBA settings with no card field')}
      ${list(c.read, 'Settings the rules read')}
      <div class="dl"><button data-export="${k}:json" title="Download as a Bridge-Classroom convention card">Card JSON</button><button data-export="${k}:bbsa" title="Download as a BBA .bbsa file">.bbsa</button></div></div>`
  }
  el.innerHTML = side('ns', 'N-S') + side('ew', 'E-W')
  for (const b of el.querySelectorAll('[data-export]')) {
    const [k, fmt] = b.dataset.export.split(':')
    b.addEventListener('click', () => exportCard(k, fmt))
  }
}

function drawDiagnostics(out) {
  const el = $('diagnostics')
  const list = out.diagnostics.filter((d) => d.severity !== 'info' || !/bbsa keys have no card field/.test(d.message))
  el.hidden = !list.length
  el.innerHTML = '<h2>Messages</h2><ul>' + list.map((d) =>
    `<li class="${d.severity}"><strong>${d.severity}</strong>${d.line ? ` (line ${d.line}${d.col ? `, col ${d.col}` : ''})` : d.col ? ` (col ${d.col})` : ''}: ${esc(d.message)}${d.hint ? `<span class="hint">${esc(d.hint)}</span>` : ''}</li>`).join('') + '</ul>'
}

function writeHash(out) {
  if (!out.hands || loading) return
  const params = { ...session.params, stop: out.complete ? 'end' : String(out.calls.length) }
  const hash = writeFragment(session.getInput(), params, {
    ns: 'scenario', ew: 'scenario', rotate: 0, stop: 'end', dd: true, view: 'all', meanings: false, xray: 0,
  })
  if (hash !== location.hash) {
    lastHash = hash
    history.replaceState(null, '', location.pathname + location.search + hash)
  }
}

function render() {
  const out = session.getOutput()
  drawHead(out)
  drawHands(out)
  drawAuction(out)
  drawXray(out)
  drawDetail(out)
  drawMeanings(out)
  drawBidbox(out)
  drawResult(out)
  drawCoverage(out)
  drawDiagnostics(out)
  const has = !!out.hands && !loading
  $('bid-next').disabled = !has || out.complete
  $('bid-end').disabled = !has || out.complete
  $('undo').disabled = !has || !out.calls.length
  $('restart').disabled = !has || !out.calls.some((c) => c.by !== 'forced')
  writeHash(out)
}
session.onChange(() => render())

// ── The tool contract: window.rustyBiddingBot ─────────────────────────

window[GLOBAL] = {
  /** Load input (null: keep the deal), apply params, bid to params.stop. */
  async run(input, params) {
    loading = true
    render()
    try {
      return await session.run(input ?? null, params)
    } finally {
      loading = false
      syncControls()
      render()
    }
  },
  /** {ok, diagnostics} without bidding. */
  validate: (input, params) => session.validate(input, params),
  getInput: () => session.getInput(),
  /** Load input into the page; the engine does not bid. */
  async setInput(input) {
    await window[GLOBAL].run(input, { stop: 0 })
    return session.getInput()
  },
  getOutput: () => session.getOutput(),
  info: session.info,
  defaults: DEFAULT_CARDS,
}

// ── Start ──────────────────────────────────────────────────────────────

async function fromHash() {
  const f = readFragment(location.hash)
  if (!f) return false
  const { input } = f
  // A link says everything: what it leaves out is the default, not
  // whatever the page had before.
  const params = { ...session.defaultParams(), ...f.params }
  const hasSource = ['scenario', 'scenarioScript', 'script', 'random', 'pbn', 'deal'].some((k) => input[k] != null)
  if (!hasSource) return false
  if (params.rotate != null) params.rotate = +params.rotate
  params.stop ??= 'end'
  await window[GLOBAL].run(input, params)
  return true
}

window.addEventListener('hashchange', () => {
  if (location.hash !== lastHash) fromHash()
})

fetchManifest().then((m) => {
  manifest = m
  const sel = $('scenario')
  sel.textContent = ''
  sel.append(new Option('Choose a scenario…', ''))
  for (const s of m.sections) {
    const g = document.createElement('optgroup')
    g.label = s.label
    for (const it of s.items) g.append(new Option(it.label, it.name))
    sel.append(g)
  }
  if (session.base?.scenario) sel.value = session.base.scenario
  render()
}).catch((e) => {
  $('scenario').innerHTML = '<option value="">Scenarios did not load</option>'
  session.addDiagnostics([{ severity: 'warning', message: `scenarios: ${e.message}`, hint: 'Practice-Bidding-Scenarios is read from raw.githubusercontent.com; random deals, scripts and your own deals still work' }])
  render()
})

syncControls()
if (!(await fromHash())) {
  setSource('random', { quiet: true })
  await dealRandom()
}
