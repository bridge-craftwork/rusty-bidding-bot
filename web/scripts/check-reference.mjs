// The reference.txt drift check (CI): the served <dist>/reference.txt must
// describe what is served beside it.
//
//   node web/scripts/check-reference.mjs web/dist native-reference.txt
//
// 1. Its CONVENTIONS section is byte for byte what `rbb bid reference`
//    prints from the .bid files in the repository: the rules compiled into
//    the WASM are the rules in the tree.
// 2. It is what lib/reference.js writes from the shipped engine now (a
//    stale file left from an older build fails).
// 3. It names every INPUT and PARAMS field, every API method, every
//    built-in card, and every module the engine's conventions() lists.
// 4. index.html and app.js use no control the vocabulary does not have:
//    every data-param in the page is a PARAMS name.

import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { pathToFileURL } from 'node:url'

const dist = path.resolve(process.argv[2] ?? 'web/dist')
const nativeFile = process.argv[3]
const served = await readFile(path.join(dist, 'reference.txt'), 'utf8')
const rbb = await import(pathToFileURL(path.join(dist, 'pkg', 'rbb_wasm.js')).href)
await rbb.default({ module_or_path: await readFile(path.join(dist, 'pkg', 'rbb_wasm_bg.wasm')) })
const { renderReference } = await import(pathToFileURL(path.join(dist, 'lib', 'reference.js')).href)
const { INPUT, PARAMS, GLOBAL } = await import(pathToFileURL(path.join(dist, 'lib', 'tool.js')).href)

const problems = []
const info = JSON.parse(rbb.info())
const conventions = rbb.reference('{}')

if (nativeFile) {
  const native = await readFile(nativeFile, 'utf8')
  if (native.trimEnd() !== conventions.trimEnd()) {
    const a = native.split('\n')
    const b = conventions.split('\n')
    const i = a.findIndex((l, n) => l !== b[n])
    problems.push(`the WASM rules differ from \`rbb bid reference\` (first at line ${i + 1}:\n  native: ${a[i]}\n  wasm:   ${b[i]})`)
  }
}
if (renderReference({ info, conventions }) !== served) {
  problems.push('reference.txt is not what lib/reference.js writes from this engine: re-run web/build.sh')
}
const must = [
  ...INPUT.map((f) => f.name), ...PARAMS.map((f) => f.name),
  'run(input, params)', 'validate(input, params)', 'getInput()', 'setInput(input)', 'getOutput()',
  `window.${GLOBAL}`, ...info.stock_cards,
]
const mods = JSON.parse(rbb.conventions('{}')).modules
must.push(...mods.map((m) => `== ${m.name}:`))
for (const word of must) if (!served.includes(word)) problems.push(`reference.txt does not mention ${word}`)

const names = new Set(PARAMS.map((p) => p.name))
for (const f of ['index.html', 'app.js']) {
  const text = await readFile(path.join(dist, f), 'utf8')
  for (const m of text.matchAll(/data-param="([^"]+)"/g)) {
    if (!names.has(m[1])) problems.push(`${f}: data-param="${m[1]}" is not a PARAMS field`)
  }
}

if (problems.length) {
  console.error('reference.txt drift:\n- ' + problems.join('\n- '))
  process.exit(1)
}
console.log(`reference.txt: up to date (${mods.length} modules, ${must.length} names checked)`)
