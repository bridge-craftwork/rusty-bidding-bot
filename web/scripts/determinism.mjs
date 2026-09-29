// Determinism across builds (CI, web job): the WASM engine must bid every
// deal exactly as the native `rbb` does, call for call, with the same
// meanings and readings.
//
//   node web/scripts/determinism.mjs deals deals.pbn [count]
//   rbb bid-pbn -i deals.pbn -o native.pbn --ns-card 21GF-DEFAULT \
//       --ew-card 21GF-GIB --all-meanings
//   node web/scripts/determinism.mjs check web/dist native.pbn
//
// `deals` writes `count` (default 300) random deals from a fixed seed, with
// each board's dealer, vulnerability and scoring. `check` bids the deals of
// the native output in the WASM build, twice (bidDeal, and auction() with
// four bots), and compares each call and its [Note] (the meaning and what
// the call showed, as `--all-meanings` writes it) with the native file.

import { readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { pathToFileURL } from 'node:url'

const [mode, a, b] = process.argv.slice(2)

const SEATS = ['N', 'E', 'S', 'W']
const RANKS = 'AKQJT98765432'
const VULS = ['None', 'NS', 'EW', 'All', 'NS', 'EW', 'All', 'None', 'EW', 'All', 'None', 'NS', 'All', 'None', 'NS', 'EW']

function deals(count) {
  // xorshift32, seeded: the same file every time.
  let s = 0x9e3779b9
  const next = () => {
    s ^= s << 13; s >>>= 0
    s ^= s >>> 17
    s ^= s << 5; s >>>= 0
    return s
  }
  const out = ['% PBN 2.1', '% Random deals for web/scripts/determinism.mjs', '']
  for (let n = 1; n <= count; n++) {
    const deck = [...Array(52).keys()]
    for (let i = 51; i > 0; i--) {
      const j = next() % (i + 1)
      ;[deck[i], deck[j]] = [deck[j], deck[i]]
    }
    const hands = SEATS.map((_, h) => {
      const cards = deck.slice(h * 13, h * 13 + 13)
      return [0, 1, 2, 3].map((suit) => cards.filter((c) => Math.floor(c / 13) === suit)
        .sort((x, y) => x - y).map((c) => RANKS[c % 13]).join('')).join('.')
    })
    out.push(`[Board "${n}"]`, `[Dealer "${SEATS[(n - 1) % 4]}"]`, `[Vulnerable "${VULS[(n - 1) % 16]}"]`,
      `[Scoring "${n % 2 ? 'IMP' : 'MP'}"]`, `[Deal "N:${hands.join(' ')}"]`, '')
  }
  return out.join('\n')
}

/// Boards of a PBN file: tags, and the [Auction] section's calls and notes.
function parsePbn(text) {
  const boards = []
  let cur = null
  let inAuction = false
  for (const line of text.split(/\r?\n/)) {
    if (line.startsWith('%')) continue
    const tag = line.match(/^\[(\w+) "(.*)"\]$/)
    if (tag) {
      const [, name, value] = tag
      if (name === 'Board' || !cur) {
        cur = { tags: {}, calls: [], refs: [], notes: {} }
        boards.push(cur)
      }
      if (name === 'Note') {
        const i = value.indexOf(':')
        cur.notes[value.slice(0, i)] = value.slice(i + 1)
      } else {
        cur.tags[name] = value
        inAuction = name === 'Auction'
      }
      continue
    }
    if (line.startsWith('[')) { inAuction = false; continue }
    if (inAuction && cur) {
      for (const tok of line.trim().split(/\s+/).filter(Boolean)) {
        const ref = tok.match(/^=(\d+)=$/)
        if (ref) cur.refs[cur.calls.length - 1] = ref[1]
        else cur.calls.push(tok)
      }
    }
  }
  return boards
}

/// A call's note as `rbb bid-pbn --all-meanings` writes it.
function note(step) {
  if (!step.rule) return null
  const explanation = step.explanation ?? ''
  const shown = step.knowledge.summary
  const t = (!shown || shown === explanation ? explanation : `${explanation} | ${shown}`)
    .replaceAll('"', "'").replace(/[\r\n]/g, ' ').trim()
  return t || null
}

const pbn = (call) => call.replace('NT', 'N')

if (mode === 'deals') {
  await writeFile(a, deals(Number(b ?? 300)))
  console.log(`wrote ${b ?? 300} deals to ${a}`)
} else if (mode === 'check') {
  const dist = path.resolve(a)
  const rbb = await import(pathToFileURL(path.join(dist, 'pkg', 'rbb_wasm.js')).href)
  await rbb.default({ module_or_path: await readFile(path.join(dist, 'pkg', 'rbb_wasm_bg.wasm')) })
  const J = (name, req) => JSON.parse(rbb[name](JSON.stringify(req)))
  const engine = J('createEngine', { cards: { ns: '21GF-DEFAULT', ew: '21GF-GIB' } }).engine
  const boards = parsePbn(await readFile(b, 'utf8'))
  const problems = []
  let calls = 0
  const t0 = Date.now()
  for (const bd of boards) {
    const { Board: n, Deal: deal, Dealer: dealer, Vulnerable: vul, Scoring: scoring } = bd.tags
    const req = { engine, deal, dealer, vul, scoring }
    const whole = J('bidDeal', req)
    const table = J('auction', { ...req, bots: SEATS, no_rule: 'pass' })
    if (!whole.ok || !table.ok) {
      problems.push(`board ${n}: ${JSON.stringify([...whole.diagnostics, ...table.diagnostics])}`)
      continue
    }
    const native = bd.calls.filter((c) => c !== 'AP')
    const wasm = whole.calls.map((c) => pbn(c.call))
    const viaTable = table.steps.map((s) => pbn(s.call))
    calls += native.length
    if (native.join(' ') !== wasm.join(' ')) {
      problems.push(`board ${n}: native ${native.join(' ')}\n            wasm   ${wasm.join(' ')}`)
      continue
    }
    if (viaTable.join(' ') !== wasm.join(' ')) {
      problems.push(`board ${n}: auction() ${viaTable.join(' ')} but bidDeal ${wasm.join(' ')}`)
      continue
    }
    table.steps.forEach((s, i) => {
      const want = bd.refs[i] ? bd.notes[bd.refs[i]] : null
      const got = note(s)
      if (want !== got) problems.push(`board ${n}, call ${i + 1} (${s.call}): native ${JSON.stringify(want)}\n            wasm   ${JSON.stringify(got)}`)
    })
  }
  if (boards.length === 0) problems.push(`no boards in ${b}`)
  if (problems.length) {
    console.error(`determinism: ${problems.length} differences between native and WASM:\n- ${problems.slice(0, 20).join('\n- ')}`)
    process.exit(1)
  }
  console.log(`determinism: ${boards.length} deals, ${calls} calls, native and WASM agree (${Date.now() - t0} ms)`)
} else {
  console.error('usage: determinism.mjs deals <out.pbn> [count] | check <dist> <native.pbn>')
  process.exit(2)
}
