// The auction explorer: a card for each side, an auction clicked through
// call by call, and for the seat to call every legal call with what it
// means to the engine (WASM `meaning`, its `announcement`). Like GIB's
// bidding definitions browser, for our system and any card.
//
// State lives in the URL fragment (ns, ew, dealer, vul, scoring, auction),
// as on the main page: an uploaded card stays out of it.

import init, * as rbb from '../pkg/rbb_wasm.js'
import { SEATS, SEAT_NAMES, SUIT_SYMBOLS, normCall, auctionComplete } from '../lib/deal.js'
import { fetchBbsa } from '../lib/pbs.js'

await init()

const $ = (id) => document.getElementById(id)
const esc = (s) => String(s ?? '').replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c])

function call(fn, req) {
  try {
    return JSON.parse(rbb[fn](JSON.stringify(req ?? {})))
  } catch (e) {
    return { ok: false, diagnostics: [{ severity: 'error', message: `${fn}: ${e?.message ?? e}` }] }
  }
}

const info = JSON.parse(rbb.info())
const STOCK = info.stock_cards ?? []
const DEFAULTS = { ns: '21GF-DEFAULT', ew: '21GF-GIB', dealer: 'N', vul: 'None', scoring: 'MP' }
const STRAINS = ['C', 'D', 'H', 'S', 'NT']
const uploads = new Map()

const state = { ...DEFAULTS, calls: [] }
let engine = null
let engineKey = ''
let diagnostics = []

// ── The fragment ──────────────────────────────────────────────────────

function readFragment() {
  const q = new URLSearchParams(location.hash.replace(/^#/, ''))
  for (const k of ['ns', 'ew', 'dealer', 'vul', 'scoring']) {
    if (q.get(k)) state[k] = q.get(k)
  }
  state.calls = (q.get('auction') ?? '').split(/[\s,-]+/).filter(Boolean).map(normCall)
}

function writeFragment() {
  const q = new URLSearchParams()
  for (const k of ['ns', 'ew', 'dealer', 'vul', 'scoring']) {
    if (!state[k].startsWith('upload:') && state[k] !== DEFAULTS[k]) q.set(k, state[k])
  }
  if (state.calls.length) q.set('auction', state.calls.map(short).join('-'))
  const h = q.toString()
  history.replaceState(null, '', h ? '#' + h : location.pathname + location.search)
}

/** `Pass` → `P`, `1NT` → `1N`: GIB's short calls, for the link. */
const short = (c) => (c === 'Pass' ? 'P' : c.replace('NT', 'N'))

// ── Cards and the engine ──────────────────────────────────────────────

async function cardSpec(name) {
  if (uploads.has(name)) return uploads.get(name)
  if (STOCK.includes(name)) return name
  try {
    return { bbsa: await fetchBbsa(name), name }
  } catch (e) {
    diagnostics.push({ severity: 'warning', message: `${name} is not a built-in card and did not load (${e.message})` })
    return null
  }
}

async function prepareEngine() {
  const key = `${state.ns}|${state.ew}`
  if (engine != null && key === engineKey) return
  diagnostics = []
  const ns = (await cardSpec(state.ns)) ?? DEFAULTS.ns
  const ew = (await cardSpec(state.ew)) ?? DEFAULTS.ew
  const r = call('createEngine', { cards: { ns, ew } })
  diagnostics.push(...(r.diagnostics ?? []).filter((d) => d.severity !== 'info'))
  engine = r.ok ? r.engine : null
  engineKey = key
}

function fillCardSelects() {
  for (const side of ['ns', 'ew']) {
    const sel = $(`card-${side}`)
    const names = [...STOCK, ...uploads.keys()]
    if (!names.includes(state[side])) names.push(state[side])
    sel.innerHTML = names.map((n) => `<option value="${esc(n)}">${esc(n.replace(/^upload:/, 'Uploaded: '))}</option>`).join('')
    sel.value = state[side]
  }
}

// ── The auction ───────────────────────────────────────────────────────

const seatOf = (i) => SEATS[(SEATS.indexOf(state.dealer) + i) % 4]
const sideOf = (seat) => (seat === 'N' || seat === 'S' ? 'ns' : 'ew')

/** Every legal call after `calls`: Pass, X or XX when allowed, every higher bid. */
function legalCalls(calls) {
  const next = sideOf(seatOf(calls.length))
  let lastBid = -1
  let lastAct = null
  for (let i = calls.length - 1; i >= 0; i--) {
    if (calls[i] !== 'Pass' && lastAct == null) lastAct = { call: calls[i], side: sideOf(seatOf(i)) }
    if (/^[1-7]/.test(calls[i])) {
      lastBid = (+calls[i][0] - 1) * 5 + STRAINS.indexOf(calls[i].slice(1))
      break
    }
  }
  const out = ['Pass']
  if (lastAct && lastAct.side !== next && /^[1-7]/.test(lastAct.call)) out.push('X')
  if (lastAct && lastAct.side !== next && lastAct.call === 'X') out.push('XX')
  for (let r = lastBid + 1; r < 35; r++) out.push(`${Math.floor(r / 5) + 1}${STRAINS[r % 5]}`)
  return out
}

function callHtml(c) {
  if (c === 'Pass') return 'Pass'
  if (c === 'X' || c === 'XX') return c
  const m = /^([1-7])(C|D|H|S|NT)$/.exec(c)
  if (!m) return esc(c)
  return m[2] === 'NT' ? `${m[1]}NT` : `${m[1]}<span class="s-${m[2]}">${SUIT_SYMBOLS[m[2]]}</span>`
}

/** Suit names in a meaning shown with their symbols ("5+ spades" → "5+ ♠"). */
function suitsHtml(text) {
  return esc(text).replace(/\b(spade|heart|diamond|club)s?\b/gi, (w, s) => {
    const k = { spade: 'S', heart: 'H', diamond: 'D', club: 'C' }[s.toLowerCase()]
    return `<span class="s-${k}">${SUIT_SYMBOLS[k]}</span>`
  })
}

const request = (calls) => ({ engine, dealer: state.dealer, vul: state.vul, scoring: state.scoring, auction: calls.join(' ') })

// ── Drawing ───────────────────────────────────────────────────────────

function renderAuction(steps) {
  const order = ['W', 'N', 'E', 'S']
  const lead = order.indexOf(state.dealer)
  const cells = Array(lead).fill('<td></td>')
  state.calls.forEach((c, i) => {
    const s = steps?.[i]
    const tip = s ? (s.announcement || s.explanation || 'Rusty has no meaning for this call') : ''
    cells.push(`<td><button type="button" class="xc${s && !s.rule ? ' unknown' : ''}" data-at="${i}" title="${esc(tip)}">${callHtml(c)}</button></td>`)
  })
  const done = auctionComplete(state.calls)
  if (!done) cells.push('<td class="next">?</td>')
  const rows = []
  for (let i = 0; i < cells.length; i += 4) rows.push(`<tr>${cells.slice(i, i + 4).join('')}</tr>`)
  $('auction').innerHTML = `<thead><tr>${order.map((s) => `<th>${s} <span class="side">${esc(state[sideOf(s)].replace(/^upload:/, ''))}</span></th>`).join('')}</tr></thead><tbody>${rows.join('')}</tbody>`
  $('back').disabled = !state.calls.length
  $('restart').disabled = !state.calls.length
}

function renderCalls(rows, complete, contract) {
  if (complete) {
    $('next-title').textContent = 'Auction over'
    $('calls').innerHTML = `<tbody><tr><td>${contract ? `Contract: ${esc(contract)}` : 'Passed out'}</td></tr></tbody>`
    return
  }
  const seat = seatOf(state.calls.length)
  $('next-title').textContent = `${SEAT_NAMES[seat]} to call (${state[sideOf(seat)].replace(/^upload:/, '')})`
  const showUnknown = $('show-unknown').checked
  const showReason = $('show-reason').checked
  const body = rows
    .filter((r) => showUnknown || r.known)
    .map((r) => {
      const text = r.known ? (r.step.announcement || r.step.explanation || '') : 'No meaning in this system'
      const flags = []
      const a = r.step?.alert
      if (a?.kind === 'alert') flags.push('<span class="flag alert">Alert</span>')
      if (a?.kind === 'announce') flags.push(`<span class="flag announce">${esc(a.text || 'Announce')}</span>`)
      const reason = showReason && r.known && r.step.explanation && r.step.explanation !== r.step.announcement
        ? `<div class="reason">${suitsHtml(r.step.explanation)}${r.step.rule ? ` <code>${esc(r.step.rule.file.replace(/^.*conventions\//, ''))}:${r.step.rule.line}</code>` : ''}</div>`
        : ''
      return `<tr class="${r.known ? '' : 'unknown'}"><td class="call"><button type="button" class="xc" data-call="${esc(r.call)}">${callHtml(r.call)}</button></td>` +
        `<td class="meaning">${suitsHtml(text)} ${flags.join(' ')}${reason}</td></tr>`
    })
  $('calls').innerHTML = `<thead><tr><th>Call</th><th>Meaning</th></tr></thead><tbody>${body.join('')}</tbody>`
}

function renderDiagnostics() {
  $('diagnostics').innerHTML = diagnostics.map((d) => `<div class="diag ${esc(d.severity)}">${esc(d.message)}</div>`).join('')
}

let drawing = 0
async function draw() {
  const ticket = ++drawing
  writeFragment()
  fillCardSelects()
  for (const k of ['dealer', 'vul', 'scoring']) $(k).value = state[k]
  await prepareEngine()
  if (ticket !== drawing) return
  renderDiagnostics()
  if (engine == null) return
  const read = call('interpret', request(state.calls))
  if (!read.ok) {
    diagnostics.push(...(read.diagnostics ?? []))
    renderDiagnostics()
    return
  }
  renderAuction(read.steps)
  $('position').textContent = [read.position?.sides?.ns?.summary, read.position?.sides?.ew?.summary]
    .map((s, i) => (s && s !== 'forcing none' ? `${i ? 'EW' : 'NS'}: ${s}` : ''))
    .filter(Boolean).join(' · ')
  if (read.complete) {
    renderCalls([], true, read.contract && read.contract !== 'Pass' ? `${read.contract} by ${read.declarer}` : null)
    return
  }
  const rows = legalCalls(state.calls).map((c) => {
    const r = call('meaning', request([...state.calls, c]))
    return { call: c, known: !!r.ok && !!r.known, step: r.step ?? null }
  })
  if (ticket !== drawing) return
  renderCalls(rows, false)
}

// ── Events ────────────────────────────────────────────────────────────

document.addEventListener('click', (e) => {
  const b = e.target.closest('button.xc')
  if (!b) return
  if (b.dataset.call) state.calls = [...state.calls, b.dataset.call]
  else if (b.dataset.at != null) state.calls = state.calls.slice(0, +b.dataset.at)
  draw()
})
$('back').addEventListener('click', () => { state.calls = state.calls.slice(0, -1); draw() })
$('restart').addEventListener('click', () => { state.calls = []; draw() })
for (const side of ['ns', 'ew']) {
  $(`card-${side}`).addEventListener('change', (e) => { state[side] = e.target.value; draw() })
}
for (const k of ['dealer', 'vul', 'scoring']) {
  $(k).addEventListener('change', (e) => {
    state[k] = e.target.value
    // The calls belong to seats: a new dealer starts a new auction.
    if (k === 'dealer') state.calls = []
    draw()
  })
}
for (const id of ['show-unknown', 'show-reason']) $(id).addEventListener('change', draw)
$('card-file').addEventListener('change', async () => {
  const f = $('card-file').files[0]
  if (!f) return
  const text = await f.text()
  const name = f.name.replace(/\.(bbsa|json)$/i, '')
  let spec
  if (/\.json$/i.test(f.name) || /^\s*\{/.test(text)) {
    try {
      const j = JSON.parse(text)
      spec = { json: j, name: j.name || j.card_data?.metadata?.name || j.metadata?.name || name }
    } catch (err) {
      diagnostics = [{ severity: 'error', message: `card file: ${f.name} is not JSON: ${err.message}` }]
      renderDiagnostics()
      return
    }
  } else {
    spec = { bbsa: text, name }
  }
  const key = `upload:${spec.name}`
  uploads.set(key, spec)
  state.ns = key
  $('card-file').value = ''
  draw()
})
window.addEventListener('hashchange', () => { readFragment(); draw() })

readFragment()
draw()
