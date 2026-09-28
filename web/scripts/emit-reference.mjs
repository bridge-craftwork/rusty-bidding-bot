// Write <dist>/reference.txt from the engine in <dist>/pkg (the one the page
// loads) and lib/reference.js (the page's own vocabulary). Run by build.sh.
//
//   node web/scripts/emit-reference.mjs web/dist

import { readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { pathToFileURL } from 'node:url'

const dist = path.resolve(process.argv[2] ?? 'web/dist')
const rbb = await import(pathToFileURL(path.join(dist, 'pkg', 'rbb_wasm.js')).href)
await rbb.default({ module_or_path: await readFile(path.join(dist, 'pkg', 'rbb_wasm_bg.wasm')) })
const { renderReference } = await import(pathToFileURL(path.join(dist, 'lib', 'reference.js')).href)

const info = JSON.parse(rbb.info())
if (!info.ok) {
  console.error('emit-reference: the engine reports errors:', info.diagnostics)
  process.exit(1)
}
const text = renderReference({ info, conventions: rbb.reference('{}') })
await writeFile(path.join(dist, 'reference.txt'), text)
console.log(`reference.txt: ${text.length} bytes, ${info.modules} modules, rules ${info.rules_id}`)
