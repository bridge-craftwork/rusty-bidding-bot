// Deep links: the input and the params in the URL fragment, never the query
// string (a fragment never reaches a server, a log or a cache key; a query
// string carrying a deal can be refused at the edge). key=value pairs
// joined by &, URI-encoded, as URLSearchParams writes them.

import { INPUT, PARAMS } from './tool.js'

const INPUT_KEYS = INPUT.map((f) => f.name)
const PARAM_KEYS = PARAMS.map((f) => f.name)

/** `#a=1&b=2` → {input, params}, or null when there is nothing to read. */
export function readFragment(hash) {
  const text = String(hash ?? '').replace(/^#/, '')
  if (!text) return null
  const q = new URLSearchParams(text)
  const input = {}
  const params = {}
  for (const [k, v] of q) {
    if (INPUT_KEYS.includes(k)) input[k] = k === 'random' ? v !== 'false' : v
    else if (PARAM_KEYS.includes(k)) params[k] = k === 'set' ? v.split(';').filter(Boolean) : v
  }
  if (!Object.keys(input).length && !Object.keys(params).length) return null
  return { input, params }
}

/** {input, params} → `#...`: plain values only (an uploaded card stays out). */
export function writeFragment(input, params, defaults = {}) {
  const q = new URLSearchParams()
  for (const k of INPUT_KEYS) {
    const v = input?.[k]
    if (v == null || v === '' || v === false) continue
    q.set(k, String(v))
  }
  for (const k of PARAM_KEYS) {
    let v = params?.[k]
    if (v == null || (typeof v === 'object' && !Array.isArray(v))) continue
    if (Array.isArray(v)) {
      if (!v.length) continue
      v = v.join(';')
    }
    if (String(v) === String(defaults[k] ?? '')) continue
    q.set(k, String(v))
  }
  const s = q.toString()
  return s ? '#' + s : ''
}
