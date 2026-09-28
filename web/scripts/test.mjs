// Smoke tests of the tool contract (lib/tool.js) against the built WASM, in
// Node: run, validate, getInput/getOutput round trips, rotation, undo, the
// dd table. Offline by default; with PBS_DIR=<a Practice-Bidding-Scenarios
// checkout> it also deals a scenario from the local corpus.
//
//   node web/scripts/test.mjs [web/dist]

import { readFile } from 'node:fs/promises'
import assert from 'node:assert/strict'
import { fileURLToPath, pathToFileURL } from 'node:url'
import path from 'node:path'

const here = path.dirname(fileURLToPath(import.meta.url))
const dist = path.resolve(process.argv[2] ?? path.join(here, '..', 'dist'))
const rbb = await import(pathToFileURL(path.join(dist, 'pkg', 'rbb_wasm.js')).href)
await rbb.default({ module_or_path: await readFile(path.join(dist, 'pkg', 'rbb_wasm_bg.wasm')) })
const { Session } = await import(pathToFileURL(path.join(dist, 'lib', 'tool.js')).href)
const { PBS_RAW } = await import(pathToFileURL(path.join(dist, 'lib', 'pbs.js')).href)

let loadDealer3 = null
try {
  const js = path.join(dist, 'dealer3', 'dealer3_wasm.js')
  await readFile(js)
  globalThis.self ??= globalThis
  loadDealer3 = async () => {
    const m = await import(pathToFileURL(js).href)
    await m.default({ module_or_path: await readFile(path.join(dist, 'dealer3', 'dealer3_wasm_bg.wasm')) })
    return m
  }
} catch { /* built without dealer3 */ }

const pbsDir = process.env.PBS_DIR
const fetchImpl = async (url) => {
  if (pbsDir && url.startsWith(PBS_RAW)) {
    try {
      const text = await readFile(path.join(pbsDir, url.slice(PBS_RAW.length)), 'utf8')
      return { ok: true, status: 200, text: async () => text, json: async () => JSON.parse(text) }
    } catch { return { ok: false, status: 404 } }
  }
  return { ok: false, status: 599 }
}

let failed = 0
async function test(name, f) {
  try {
    await f()
    console.log(`ok    ${name}`)
  } catch (e) {
    failed++
    console.log(`FAIL  ${name}\n${e.stack}`)
  }
}

const DEAL = 'N:AK52.KJ7.Q94.K83 QJ3.Q95.KJ3.QJ74 T64.AT832.A2.T62 987.64.T8765.A95'

await test('run bids a deal to the end and solves it', async () => {
  const s = new Session(rbb, { fetchImpl, loadDealer3 })
  const out = await s.run({ deal: DEAL, dealer: 'N', vul: 'None' }, { scoring: 'IMP' })
  assert.equal(out.ok, true, JSON.stringify(out.diagnostics))
  assert.equal(out.complete, true)
  assert.equal(out.calls[0].call, '1NT')
  assert.equal(out.calls[0].by, 'engine')
  assert.ok(out.calls[0].why)
  assert.ok(out.contract)
  assert.equal(out.dd.tricks.N.NT + out.dd.tricks.E.NT, 13)
  assert.equal(out.dd.result.contract.replace('NT', 'N'), out.contract.replace('NT', 'N'))
  assert.equal(out.cards.ns, '21GF-DEFAULT')
  assert.ok(out.coverage.ns.read.length > 0)
})

await test('stop, undo, your own call and getInput', async () => {
  const s = new Session(rbb, { fetchImpl })
  let out = await s.run({ deal: DEAL, dealer: 'N' }, { stop: 1 })
  assert.equal(out.calls.length, 1)
  assert.equal(out.complete, false)
  assert.equal(out.next, 'E')
  const r = s.play('2C')
  assert.equal(r.ok, true, JSON.stringify(r))
  assert.equal(s.getInput().auction, '1NT 2C')
  const bad = s.play('1C')
  assert.equal(bad.ok, false)
  assert.ok(bad.diagnostics[0].message.includes('not legal'))
  s.undo()
  assert.equal(s.calls.length, 1)
  assert.equal(s.getInput().auction, undefined)
  s.bidTo('end')
  out = s.getOutput()
  assert.equal(out.complete, true)
})

await test('input auction is forced; rotation turns the deal', async () => {
  const s = new Session(rbb, { fetchImpl })
  let out = await s.run({ deal: DEAL, dealer: 'N', vul: 'NS', auction: '1C' }, { rotate: 0 })
  assert.equal(out.calls[0].by, 'forced')
  assert.equal(out.calls[0].call, '1C')
  out = await s.run(null, { rotate: 1 })
  assert.equal(out.dealer, 'E')
  assert.equal(out.vul, 'EW')
  assert.equal(out.hands.E, 'AK52.KJ7.Q94.K83')
  assert.equal(out.calls[0].call, '1C')
  assert.equal(out.calls[0].seat, 'E')
})

await test('validate reports problems in the contract shape', async () => {
  const s = new Session(rbb, { fetchImpl })
  const v = await s.validate({ deal: 'N:AK52.KJ7.Q94 x y z', auction: '1NT 1C' }, { scoring: 'rubber', bogus: 1 })
  assert.equal(v.ok, false)
  for (const x of v.diagnostics) {
    assert.ok(['error', 'warning', 'info'].includes(x.severity))
    assert.equal(typeof x.message, 'string')
  }
  assert.ok(v.diagnostics.some((x) => x.message.startsWith('params.scoring')))
  assert.ok(v.diagnostics.some((x) => x.message.startsWith('params.bogus')))
  const good = await s.validate({ deal: DEAL, dealer: 'S', auction: '1NT Pass 2C' }, { scoring: 'IMP' })
  assert.equal(good.ok, true, JSON.stringify(good.diagnostics))
})

await test('random deals are reproducible from the seed', async () => {
  const a = await new Session(rbb, { fetchImpl }).run({ random: true, seed: 7, board: 3 }, { stop: 0 })
  const b = await new Session(rbb, { fetchImpl }).run({ random: true, seed: 7, board: 3 }, { stop: 0 })
  assert.equal(a.deal, b.deal)
  assert.equal(a.dealer, 'S')
  assert.equal(a.vul, 'EW')
})

await test('a Bridge-Classroom card and an uploaded .bbsa', async () => {
  const card = JSON.parse(await readFile(path.join(dist, 'cards', '21_intermediate_card.json'), 'utf8'))
  const bbsa = JSON.parse(rbb.exportCard(JSON.stringify({ card: 'Precision' }))).bbsa
  const s = new Session(rbb, { fetchImpl })
  const out = await s.run({ deal: DEAL, dealer: 'N' }, { ns: { json: card }, ew: { bbsa, name: 'My Precision' } })
  assert.equal(out.ok, true, JSON.stringify(out.diagnostics))
  assert.equal(out.cards.ns, '2/1 Intermediate')
  assert.equal(out.cards.ew, 'My Precision')
  assert.ok(Array.isArray(out.coverage.ew.ignored))
})

if (loadDealer3) {
  await test('a dealer3 script deals', async () => {
    const s = new Session(rbb, { fetchImpl, loadDealer3 })
    const script = '# convention-card-ns: Precision\ndealer south\ncondition hcp(south) >= 16 && hcp(south) <= 21\n'
    const out = await s.run({ script, seed: 3 }, { stop: 1 })
    assert.equal(out.ok, true, JSON.stringify(out.diagnostics))
    assert.equal(out.dealer, 'S')
    assert.equal(out.cards.ns, 'Precision')
    const v = await s.validate({ script: 'condition hcp(north) >=' })
    assert.equal(v.ok, false)
  })
}

await test('the WASM takes a rule set and its card vocabulary at run time', async () => {
  const J = (name, req) => JSON.parse(rbb[name](JSON.stringify(req)))
  const ruleSet = (rule) => ({
    manifest: 'name = "demo"\nlanguage = 1\n',
    fields: '[demo]\n"strong_nt" = { kind = "bool", label = "1NT", default = true }\n' +
      '"nt_min" = { kind = "int", label = "1NT minimum", min = 10, max = 20, default = 15 }\n',
    bbsa_map: '',
    rules: {
      'demo/one-nt.bid': 'module demo "Demo"\n  card   demo.strong_nt\n  param  lo = demo.nt_min default 15\n\n' +
        `when opening\n${rule}\n`,
    },
  })
  const RULE = '  1N  "Demo 1NT"  shows hcp=lo..17, balanced'

  const v = J('validate', ruleSet(RULE))
  assert.equal(v.ok, true, JSON.stringify(v.diagnostics))
  assert.equal(v.rule_set.modules, 1)
  assert.equal(v.rule_set.name, 'demo')

  const e = J('createEngine', { ...ruleSet(RULE), cards: { ns: { json: { demo: { nt_min: 15 } } } } })
  assert.equal(e.ok, true, JSON.stringify(e.diagnostics))
  assert.deepEqual(e.ns.modules, ['demo'])
  const r = J('bid', { engine: e.engine, hand: 'AK52.KJ7.Q94.K83', dealer: 'N', auction: '' })
  assert.equal(r.call, '1NT')
  assert.equal(r.rule.file, 'demo/one-nt.bid')
  // The card's own field, in the rule set's vocabulary.
  const high = J('createEngine', { ...ruleSet(RULE), cards: { ns: { json: {} }, set: ['demo.nt_min=17'] } })
  assert.equal(J('bid', { engine: high.engine, hand: 'AK52.KJ7.Q94.K83', dealer: 'N' }).call, 'Pass')
  // The embedded rules still bid as before.
  const std = J('bid', { cards: { ns: '21GF-DEFAULT' }, hand: 'AK52.KJ7.Q94.K83', dealer: 'N' })
  assert.equal(std.rule.file, 'conventions/notrump/one-nt.bid')

  // Compile errors and unknown terms come back with file and line.
  const bad = J('validate', ruleSet('  1N  "Demo"  shows hcpx>=15'))
  assert.equal(bad.ok, false)
  const err = bad.diagnostics.find((x) => x.severity === 'error')
  assert.equal(err.file, 'demo/one-nt.bid')
  assert.equal(err.line, 6)
  assert.match(err.message, /^rules: demo\/one-nt\.bid:6: /)
  const noEngine = J('createEngine', { ...ruleSet('  1N  "Demo"  shows hcpx>=15'), cards: { ns: { json: {} } } })
  assert.equal(noEngine.engine, null)
})

if (pbsDir) {
  await test('a PBS scenario from the corpus, with its cards and BBA\'s auction', async () => {
    const s = new Session(rbb, { fetchImpl })
    const out = await s.run({ scenario: 'Stayman', board: 2 }, {})
    assert.equal(out.ok, true, JSON.stringify(out.diagnostics))
    assert.equal(out.cards.ns, '21GF-DEFAULT')
    assert.equal(out.cards.ew, '21GF-GIB')
    assert.ok(out.bba.auction.length > 3)
    assert.deepEqual(s.getInput(), { scenario: 'Stayman', board: 2 })
    const turned = await s.run(null, { rotate: 1 })
    assert.equal(turned.cards.ns, '21GF-GIB', 'odd rotation swaps the scenario cards')
  })
  if (loadDealer3) {
    await test('a PBS scenario\'s own script deals fresh boards', async () => {
      const s = new Session(rbb, { fetchImpl, loadDealer3 })
      const out = await s.run({ scenarioScript: 'Stayman', seed: 4 }, { stop: 1 })
      assert.equal(out.ok, true, JSON.stringify(out.diagnostics))
      assert.equal(out.source, 'script')
      assert.equal(out.calls[0].call, '1NT')
      assert.deepEqual(s.getInput(), { scenarioScript: 'Stayman', seed: 4, board: 1 })
    })
  }
}

if (failed) {
  console.log(`\n${failed} failed`)
  process.exit(1)
}
console.log('\nall passed')
