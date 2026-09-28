// The tool: a deal, two convention cards, and the engine bidding it seat by
// seat. No DOM here: the page (app.js) draws a Session, and `window.
// rustyBiddingBot` (the bridge-craftwork tool contract) drives the same
// Session, so the UI and the programmatic surface cannot disagree. Node runs
// it too (web/scripts/test.mjs), with the WASM module passed in.
//
// INPUT and PARAMS below are the vocabulary: the page builds its controls
// from them and reference.txt is written from them.

import {
  SEATS, auctionComplete, dealToPbn, normSeat, normVul, parseCalls, parseDeal,
  parsePbn, partnership, randomDeal, rotate, seeded, boardDealer, boardVul,
} from './deal.js'
import { fetchBbsa, fetchCorpus, fetchManifest, fetchScript, scriptCards } from './pbs.js'

export const TOOL = 'rusty-bidding-bot'
export const GLOBAL = 'rustyBiddingBot'
export const DEFAULT_CARDS = { ns: '21GF-DEFAULT', ew: '21GF-GIB' }

/** What `input` may carry. One of scenario, script, random, pbn or deal gives the deal. */
export const INPUT = [
  { name: 'scenario', form: 'PBS scenario name, e.g. "Stayman"',
    about: 'Deal a board from the Practice-Bidding-Scenarios corpus for this scenario (500 boards each, with BBA\'s own auction for comparison). Its convention cards become the default cards.' },
  { name: 'board', form: 'integer 1-500', about: 'Which corpus board (scenario), or the board number for dealer and vulnerability (random, script). Default 1.' },
  { name: 'scenarioScript', form: 'PBS scenario name', about: 'Deal fresh boards with that scenario\'s own dealer script (dealer3), from `seed`, instead of its corpus; its cards as for scenario.' },
  { name: 'script', form: 'dealer3 script text', about: 'Deal with dealer3 (the dealer.exe language): the first deal the script produces from `seed`. `# convention-card-ns:` / `-ew:` lines in the script choose the cards.' },
  { name: 'random', form: 'true', about: 'A random deal from `seed`.' },
  { name: 'seed', form: 'integer', about: 'Seed for random and script deals, so a link reproduces the deal. Default 1.' },
  { name: 'pbn', form: 'PBN text of one board', about: 'Its [Deal], [Dealer], [Vulnerable] and [Board] are read; its [Auction] is not.' },
  { name: 'deal', form: 'PBN deal "N:<N> <E> <S> <W>" (hands S.H.D.C) or {"N": hand, ...}', about: 'The deal itself.' },
  { name: 'dealer', form: 'N | E | S | W', about: 'Overrides the source\'s dealer.' },
  { name: 'vul', form: 'None | NS | EW | All', about: 'Overrides the source\'s vulnerability.' },
  { name: 'auction', form: 'calls from the dealer, e.g. "1NT Pass 2C" (P, X, XX; 1N or 1NT)', about: 'Calls already made: kept as they are (forced); the engine bids from there.' },
]

/** What `params` may carry. */
export const PARAMS = [
  { name: 'ns', form: 'card', default: 'the scenario\'s card, else 21GF-DEFAULT', about: 'North-South\'s convention card: a built-in card name (see "Cards"), "scenario", {"bbsa": "<.bbsa file text>"} or {"json": <Bridge-Classroom card JSON>}.' },
  { name: 'ew', form: 'card', default: 'the scenario\'s card, else 21GF-GIB', about: 'East-West\'s card, the same forms.' },
  { name: 'set', form: '["path=value", ...]', default: '[]', about: 'Card changes for both sides, e.g. "general.style=bba" or "notrump.stayman.play=false" (field paths: the conventions reference, `card:` lines).' },
  { name: 'scoring', values: ['MP', 'IMP'], default: 'the board\'s, else MP', about: 'Matchpoints or IMPs; some rules judge by it.' },
  { name: 'rotate', values: [0, 1, 2, 3], default: 0, about: 'Turn the deal this many seats clockwise (dealer and vulnerability turn with it). With odd turns the scenario\'s cards change sides too, so each hand keeps its system.' },
  { name: 'stop', form: '"end" or an integer', default: '"end"', about: 'How many calls to have on the table when run() returns: "end" bids the auction out.' },
  { name: 'dd', values: [true, false], default: true, about: 'Solve the double-dummy table, par and the contract\'s result when the auction ends.' },
  { name: 'view', values: ['all', 'N', 'E', 'S', 'W'], default: 'all', about: 'The page: show all four hands, or only this seat\'s until the auction ends.' },
  { name: 'meanings', values: [true, false], default: false, about: 'The page: show what every call means, not only alerts.' },
]

const PARAM_NAMES = new Set(PARAMS.map((p) => p.name))
const INPUT_NAMES = new Set(INPUT.map((p) => p.name))

/** Parse an rbb WASM JSON response, never throwing. */
function call(rbb, fn, req) {
  try {
    return JSON.parse(rbb[fn](typeof req === 'string' ? req : JSON.stringify(req ?? {})))
  } catch (e) {
    return { ok: false, diagnostics: [{ severity: 'error', message: `${fn}: ${e?.message ?? e}` }] }
  }
}

function diag(severity, message, hint) {
  return hint ? { severity, message, hint } : { severity, message }
}

/**
 * Read `input` loosely: a string is PBN (a deal string or a board's tags),
 * an object is INPUT fields.
 */
export function normalizeInput(input) {
  if (input == null) return {}
  if (typeof input === 'string') {
    const s = input.trim()
    if (/^\[/.test(s)) return { pbn: s }
    return { deal: s }
  }
  return { ...input }
}

/**
 * A session: the deal, the cards, the calls so far and what the engine
 * made of them. `rbb` is the initialised rbb WASM module; `loadDealer3` an
 * async function returning the initialised dealer3 module (or throwing).
 */
export class Session {
  constructor(rbb, { loadDealer3 = null, fetchImpl = (...a) => fetch(...a), extraCards = {} } = {}) {
    this.extraCards = extraCards // name -> card spec, e.g. a Bridge-Classroom card shipped with the page
    this.rbb = rbb
    this.loadDealer3 = loadDealer3
    this.fetch = fetchImpl
    this.info = call(rbb, 'info', '')
    this.stock = new Set(this.info.stock_cards ?? [])
    this.params = this.defaultParams()
    this.base = null // the deal as its source gave it, before rotation
    this.input = {}
    this.calls = [] // [{call, by: 'forced'|'engine'|'you', why, alert, rule, candidates}]
    this.steps = [] // interpret's reading of the auction
    this.dd = null
    this.diagnostics = []
    this.engine = null
    this.cardNames = { ns: null, ew: null }
    this.coverage = null
    this.listeners = new Set()
    this.busy = false
  }

  defaultParams() {
    return { ns: 'scenario', ew: 'scenario', set: [], scoring: null, rotate: 0, stop: 'end', dd: true, view: 'all', meanings: false }
  }

  onChange(f) { this.listeners.add(f); return () => this.listeners.delete(f) }
  changed() { for (const f of this.listeners) f(this) }

  // ── Input ────────────────────────────────────────────────────────────

  /** Check input and params without bidding: `{ok, diagnostics}`. */
  async validate(input, params) {
    const d = []
    const inp = normalizeInput(input ?? this.input)
    for (const k of Object.keys(inp)) {
      if (!INPUT_NAMES.has(k)) d.push(diag('warning', `input.${k}: not an input field (ignored)`, `input fields: ${[...INPUT_NAMES].join(', ')}`))
    }
    const p = { ...this.params, ...(params ?? {}) }
    for (const k of Object.keys(params ?? {})) {
      if (!PARAM_NAMES.has(k)) d.push(diag('warning', `params.${k}: not a parameter (ignored)`, `parameters: ${[...PARAM_NAMES].join(', ')}`))
    }
    d.push(...this.checkParams(p))
    const sources = ['scenario', 'scenarioScript', 'script', 'random', 'pbn', 'deal'].filter((k) => inp[k] != null && inp[k] !== false)
    if (sources.length > 1) d.push(diag('warning', `input: ${sources.join(', ')} each give a deal; ${sources[0]} is used`))
    if (!sources.length && !this.base) d.push(diag('error', 'input: no deal', 'give scenario, scenarioScript, script, random, pbn or deal'))
    const req = {}
    if (inp.deal != null) req.deal = inp.deal
    if (inp.pbn != null) {
      const b = parsePbn(inp.pbn)[0]
      if (!b?.deal) d.push(diag('error', 'pbn: no [Deal] tag with a deal in it', '[Deal "N:<N> <E> <S> <W>"]'))
      else req.deal = dealToPbn(b.deal)
    }
    if (inp.dealer != null) req.dealer = inp.dealer
    if (inp.vul != null) req.vul = inp.vul
    if (inp.auction != null) {
      req.auction = inp.auction
      req.dealer ??= this.base?.dealer ?? (inp.board ? boardDealer(+inp.board) : 'N')
    }
    if (p.scoring && ['MP', 'IMP'].includes(p.scoring)) req.scoring = p.scoring
    const cards = await this.cardSpecs(p, d, { dryRun: true })
    if (cards) req.cards = cards
    const v = call(this.rbb, 'validate', req)
    d.push(...(v.diagnostics ?? []))
    if (inp.script != null) {
      try {
        const d3 = await this.dealer3()
        const c = JSON.parse(d3.check_script(String(inp.script), []))
        if (!c.ok) d.push({ severity: 'error', message: `script: ${c.error}`, ...(c.line ? { line: c.line } : {}), ...(c.column ? { col: c.column } : {}) })
      } catch (e) {
        d.push(diag('error', `script: ${e.message ?? e}`))
      }
    }
    if (inp.board != null && !(Number.isInteger(+inp.board) && +inp.board >= 1)) {
      d.push(diag('error', `board: ${JSON.stringify(inp.board)} is not a board number`, '1, 2, 3, ...'))
    }
    return { ok: !d.some((x) => x.severity === 'error'), diagnostics: d }
  }

  checkParams(p) {
    const d = []
    for (const spec of PARAMS) {
      if (spec.values && p[spec.name] != null && !spec.values.some((v) => String(v) === String(p[spec.name]))) {
        d.push(diag('error', `params.${spec.name}: ${JSON.stringify(p[spec.name])} is not one of ${spec.values.join(', ')}`))
      }
    }
    if (p.stop !== 'end' && !(Number.isInteger(+p.stop) && +p.stop >= 0)) {
      d.push(diag('error', `params.stop: ${JSON.stringify(p.stop)} is not "end" or a number of calls`))
    }
    return d
  }

  /** Load a deal from `input` (see INPUT). Resets the auction to input.auction. */
  async load(input, params) {
    const inp = normalizeInput(input)
    if (params) this.setParams(params, { quiet: true })
    const d = []
    let base = null
    const board = inp.board != null ? Math.max(1, parseInt(inp.board, 10) || 1) : null
    const seed = inp.seed != null ? parseInt(inp.seed, 10) || 1 : 1
    try {
      if (inp.scenario) {
        const corpus = await fetchCorpus(inp.scenario, this.fetch)
        if (!corpus.boards.length) throw new Error(`scenario ${inp.scenario}: no deals`)
        const n = board ?? 1
        const b = corpus.boards[(n - 1) % corpus.boards.length]
        base = {
          source: 'scenario', scenario: inp.scenario, board: n, count: corpus.boards.length,
          deal: b.deal, dealer: b.dealer, vul: b.vul, scoring: b.scoring,
          cards: corpus.cards, bba: b.auction,
          bbaContract: b.tags.Contract ? { contract: b.tags.Contract, declarer: b.tags.Declarer || null } : null,
        }
      } else if (inp.script != null || inp.scenarioScript) {
        const d3 = await this.dealer3()
        let script = inp.script
        if (script == null) {
          const m = await fetchManifest(this.fetch)
          const sc = m.scenarios[inp.scenarioScript]
          if (!sc) throw new Error(`scenarioScript: no scenario called ${inp.scenarioScript}`)
          script = await fetchScript(sc.dlr, this.fetch)
        }
        script = String(script)
        const env = { v: 1, script, settings: { seed, produce: 1, maxGenerate: 10000000, format: 'pbn', autoLevel: false, roundRobin: false } }
        const r = JSON.parse(d3.run_json(JSON.stringify(env), undefined, undefined))
        const b = parsePbn(r.deals?.[0] ?? '')[0]
        if (!b?.deal) throw new Error(r.hit_limit ? 'script: no deal matched within 10,000,000 tries' : 'script: produced no deal')
        const n = board ?? 1
        base = {
          source: 'script', script, seed, board: n, scenarioScript: inp.script == null ? inp.scenarioScript : null,
          deal: b.deal, dealer: /dealer\s+[nesw]/i.test(script) ? b.dealer : boardDealer(n),
          vul: /vulnerable\s+\w/i.test(script) ? b.vul : boardVul(n),
          cards: scriptCards(script), bba: null,
        }
      } else if (inp.random) {
        const n = board ?? 1
        base = { source: 'random', seed, board: n, deal: randomDeal(seeded(seed)), dealer: boardDealer(n), vul: boardVul(n), cards: {}, bba: null }
      } else if (inp.pbn != null) {
        const b = parsePbn(inp.pbn)[0]
        if (!b?.deal) throw new Error('pbn: no [Deal] tag with a deal in it')
        base = { source: 'deal', board: b.board, deal: b.deal, dealer: b.dealer, vul: b.vul, scoring: b.scoring, cards: {}, bba: null }
      } else if (inp.deal != null) {
        const deal = typeof inp.deal === 'string' ? parseDeal(inp.deal) : inp.deal
        if (!deal || SEATS.some((s) => typeof deal[s] !== 'string')) throw new Error(`deal: ${JSON.stringify(inp.deal)} is not a PBN deal`)
        const n = board ?? 1
        base = { source: 'deal', board, deal, dealer: boardDealer(n), vul: boardVul(n), cards: {}, bba: null }
      } else if (this.base) {
        base = { ...this.base }
      } else {
        throw new Error('input: no deal (give scenario, scenarioScript, script, random, pbn or deal)')
      }
    } catch (e) {
      d.push(diag('error', String(e.message ?? e).replace(/^Error: /, '')))
    }
    if (!base) {
      this.diagnostics = d
      this.changed()
      return false
    }
    if (inp.dealer != null) base.dealer = normSeat(inp.dealer) ?? base.dealer
    if (inp.vul != null) base.vul = normVul(inp.vul) ?? base.vul
    const v = call(this.rbb, 'validate', { deal: dealToPbn(base.deal), dealer: base.dealer, auction: parseCalls(inp.auction ?? []) })
    d.push(...v.diagnostics)
    if (!v.ok) {
      this.diagnostics = d
      this.changed()
      return false
    }
    this.base = base
    this.input = { ...inp }
    this.calls = parseCalls(inp.auction ?? []).map((c) => ({ call: c, by: 'forced' }))
    this.diagnostics = d
    this.dd = null
    await this.prepareEngine()
    this.refresh()
    return true
  }

  async dealer3() {
    if (!this.loadDealer3) throw new Error('dealer3 is not available here')
    if (!this.d3) {
      this.d3 = this.loadDealer3().catch((e) => {
        this.d3 = null
        throw new Error(`the dealer3 engine did not load: ${e.message ?? e}`)
      })
    }
    return this.d3
  }

  // ── Cards and the engine ────────────────────────────────────────────

  setParams(params, { quiet = false } = {}) {
    const p = { ...this.params, ...params }
    if (p.rotate != null) p.rotate = ((parseInt(p.rotate, 10) || 0) % 4 + 4) % 4
    if (typeof p.dd === 'string') p.dd = p.dd !== 'false'
    if (typeof p.meanings === 'string') p.meanings = p.meanings === 'true'
    if (typeof p.set === 'string') p.set = p.set.split(/[;\n]/).map((s) => s.trim()).filter(Boolean)
    const engineChange = ['ns', 'ew', 'set', 'rotate', 'scoring'].some((k) => JSON.stringify(p[k]) !== JSON.stringify(this.params[k]))
    this.params = p
    if (engineChange) {
      // What the engine bid under the old cards (or seats) is not its
      // call any more: keep the calls up to the last one it did not make.
      let last = -1
      this.calls.forEach((c, i) => { if (c.by !== 'engine') last = i })
      this.calls = this.calls.slice(0, last + 1)
      this.dd = null
    }
    if (!quiet && engineChange && this.base) {
      return this.prepareEngine().then(() => this.refresh())
    }
    if (!quiet) this.changed()
    return Promise.resolve()
  }

  /** The card specs for the engine, `{ns, ew, set}`, resolving "scenario". */
  async cardSpecs(p = this.params, d = [], { dryRun = false } = {}) {
    const base = this.base
    const odd = (p.rotate ?? 0) % 2 === 1
    const out = {}
    for (const side of ['ns', 'ew']) {
      let spec = p[side]
      if (spec == null || spec === '' || spec === 'scenario') {
        // The hands the source dealt to this side: after an odd rotation
        // they are the other side's.
        const from = odd ? (side === 'ns' ? 'ew' : 'ns') : side
        spec = base?.cards?.[from] ?? DEFAULT_CARDS[from]
      }
      if (typeof spec === 'string' && this.extraCards[spec]) spec = this.extraCards[spec]
      if (typeof spec === 'string' && !this.stock.has(spec)) {
        if (dryRun) continue
        try {
          spec = { bbsa: await fetchBbsa(spec, this.fetch), name: spec }
        } catch (e) {
          d.push(diag('warning', `cards.${side}: ${spec} is not a built-in card and did not load (${e.message}); using ${DEFAULT_CARDS[side]}`))
          spec = DEFAULT_CARDS[side]
        }
      }
      out[side] = spec
    }
    if (dryRun && (!out.ns || !out.ew)) return null
    if (p.set?.length) out.set = p.set
    return out
  }

  async prepareEngine() {
    const d = []
    const cards = await this.cardSpecs(this.params, d)
    const r = call(this.rbb, 'createEngine', { cards })
    d.push(...r.diagnostics)
    if (r.ok) {
      this.engine = r.engine
      this.cardNames = { ns: r.ns.name, ew: r.ew.name }
      const cov = call(this.rbb, 'coverage', { cards })
      this.coverage = cov.ok ? { ns: cov.ns, ew: cov.ew } : null
    } else {
      this.engine = null
    }
    this.addDiagnostics(d)
  }

  addDiagnostics(list) {
    for (const x of list ?? []) {
      if (!this.diagnostics.some((y) => y.message === x.message && y.severity === x.severity)) this.diagnostics.push(x)
    }
  }

  // ── The position ────────────────────────────────────────────────────

  get view() {
    if (!this.base) return null
    return rotate(this.base, this.params.rotate)
  }

  get scoring() {
    return this.params.scoring ?? this.base?.scoring ?? 'MP'
  }

  get complete() {
    return auctionComplete(this.calls.map((c) => c.call))
  }

  get next() {
    const v = this.view
    if (!v || this.complete) return null
    return SEATS[(SEATS.indexOf(v.dealer) + this.calls.length) % 4]
  }

  /** Re-read the auction (meanings, alerts) and, once over, solve it. */
  refresh() {
    const v = this.view
    if (!v || this.engine == null) {
      this.changed()
      return
    }
    const r = call(this.rbb, 'interpret', {
      engine: this.engine, dealer: v.dealer, vul: v.vul, scoring: this.scoring,
      auction: this.calls.map((c) => c.call),
    })
    this.steps = r.steps ?? []
    this.contract = r.contract
    this.declarer = r.declarer
    this.addDiagnostics(r.diagnostics.filter((x) => x.severity !== 'info'))
    if (this.complete && this.params.dd && !this.dd) {
      const t = call(this.rbb, 'ddTable', {
        deal: dealToPbn(v.deal), vul: v.vul, dealer: v.dealer, auction: this.calls.map((c) => c.call),
      })
      this.addDiagnostics(t.diagnostics)
      this.dd = t.ok ? { tricks: t.tricks, par: t.par, result: t.result } : null
    }
    this.changed()
  }

  /** The engine's call for the seat to bid; false when it cannot. */
  bidNext({ refresh = true } = {}) {
    const v = this.view
    const seat = this.next
    if (!seat || this.engine == null) return false
    const r = call(this.rbb, 'bid', {
      engine: this.engine, hand: v.deal[seat], dealer: v.dealer, vul: v.vul,
      scoring: this.scoring, auction: this.calls.map((c) => c.call),
    })
    this.addDiagnostics(r.diagnostics.filter((x) => x.severity !== 'info'))
    if (!r.ok) {
      if (refresh) this.changed()
      return false
    }
    this.calls.push({
      call: r.call, by: 'engine', why: r.explanation, alert: r.alert, rule: r.rule,
      candidates: r.candidates, noRule: r.rule == null,
    })
    if (refresh) this.refresh()
    return true
  }

  /** Bid until `stop` calls are on the table ("end": the auction is over). */
  bidTo(stop = 'end', max = 80) {
    let n = 0
    while (!this.complete && n < max && (stop === 'end' || this.calls.length < +stop)) {
      if (!this.bidNext({ refresh: false })) break
      n++
    }
    this.refresh()
  }

  /** A call of your own for the seat to bid. */
  play(callText) {
    const v = this.view
    if (!v || this.complete) return { ok: false, diagnostics: [diag('error', 'the auction is over')] }
    const c = parseCalls([callText])[0]
    const auction = [...this.calls.map((x) => x.call), c]
    const r = call(this.rbb, 'validate', { dealer: v.dealer, auction })
    if (!r.ok) return r
    this.calls.push({ call: c, by: 'you' })
    this.dd = null
    this.refresh()
    return r
  }

  undo() {
    if (!this.calls.length) return false
    this.calls.pop()
    this.dd = null
    this.refresh()
    return true
  }

  restart() {
    this.calls = this.calls.filter((c, i) => c.by === 'forced' && this.calls.slice(0, i).every((x) => x.by === 'forced'))
    this.dd = null
    this.refresh()
  }

  // ── The contract surface ────────────────────────────────────────────

  /** The input that reproduces this position: the source, and every call up to the last one the engine did not make. */
  getInput() {
    const inp = {}
    const b = this.base
    if (!b) return inp
    if (b.source === 'scenario') Object.assign(inp, { scenario: b.scenario, board: b.board })
    else if (b.source === 'script') Object.assign(inp, { ...(b.scenarioScript ? { scenarioScript: b.scenarioScript } : { script: b.script }), seed: b.seed, board: b.board })
    else if (b.source === 'random') Object.assign(inp, { random: true, seed: b.seed, board: b.board })
    else Object.assign(inp, { deal: dealToPbn(b.deal), ...(b.board ? { board: b.board } : {}) })
    if (this.input.dealer != null) inp.dealer = b.dealer
    if (this.input.vul != null) inp.vul = b.vul
    if (b.source === 'deal') Object.assign(inp, { dealer: b.dealer, vul: b.vul })
    let last = -1
    this.calls.forEach((c, i) => { if (c.by !== 'engine') last = i })
    if (last >= 0) inp.auction = this.calls.slice(0, last + 1).map((c) => c.call).join(' ')
    return inp
  }

  /** Everything the page shows, as data. */
  getOutput() {
    const v = this.view
    if (!v) return { ok: !this.diagnostics.some((x) => x.severity === 'error'), diagnostics: this.diagnostics, deal: null, calls: [] }
    const calls = this.calls.map((c, i) => {
      const s = this.steps[i] ?? {}
      return {
        seat: SEATS[(SEATS.indexOf(v.dealer) + i) % 4],
        call: c.call,
        by: c.by,
        why: c.by === 'engine' ? c.why : null,
        meaning: s.explanation ?? null,
        alert: (c.by === 'engine' ? c.alert : null) ?? s.alert ?? null,
        rule: c.rule ?? s.rule ?? null,
        noRule: c.by === 'engine' ? !!c.noRule : s.rule == null,
        shows: s.knowledge?.summary ?? '',
      }
    })
    const bba = this.base.bba ? {
      auction: this.base.bba.map((c, i) => ({ seat: SEATS[(SEATS.indexOf(v.dealer) + i) % 4], call: c.call, note: c.note ?? null })),
      contract: this.base.bbaContract?.contract ?? null,
      // The file's declarer is for the deal as dealt: turn it with the deal.
      declarer: this.base.bbaContract?.declarer
        ? SEATS[(SEATS.indexOf(this.base.bbaContract.declarer) + this.params.rotate) % 4] : null,
    } : null
    return {
      ok: !this.diagnostics.some((x) => x.severity === 'error'),
      tool: TOOL,
      source: this.base.source,
      scenario: this.base.scenario ?? null,
      board: this.base.board ?? null,
      deal: dealToPbn(v.deal),
      hands: v.deal,
      dealer: v.dealer,
      vul: v.vul,
      scoring: this.scoring,
      rotate: this.params.rotate,
      cards: { ...this.cardNames },
      calls,
      complete: this.complete,
      next: this.next,
      contract: this.complete ? (this.contract ?? null) : null,
      declarer: this.complete ? (this.declarer ?? null) : null,
      dd: this.dd,
      bba,
      coverage: this.coverage,
      diagnostics: this.diagnostics,
    }
  }

  /** run(input, params): load (unless input is null), bid to params.stop, return the output. */
  async run(input, params) {
    this.diagnostics = []
    if (params) await this.setParams(params, { quiet: true })
    const bad = this.checkParams(this.params)
    if (bad.some((x) => x.severity === 'error')) {
      this.diagnostics = bad
      this.changed()
      return this.getOutput()
    }
    if (input != null) {
      if (!(await this.load(input))) return this.getOutput()
    } else if (!this.base) {
      this.diagnostics = [diag('error', 'input: no deal', 'give scenario, scenarioScript, script, random, pbn or deal')]
      return this.getOutput()
    } else {
      await this.prepareEngine()
    }
    // stop is for this run only: the next run() bids to the end unless told.
    this.bidTo(params?.stop ?? 'end')
    this.params.stop = 'end'
    return this.getOutput()
  }
}

export { partnership }
