// Practice-Bidding-Scenarios (PBS): the scenario menu, each scenario's deals
// with BBA's auctions (the bba/ corpus), its dealer script and its cards.
//
// The menu is PBS's pre-built manifest (one fetch, as dealer3 and
// Bridge-Classroom read it). raw.githubusercontent.com sends
// `Access-Control-Allow-Origin: *`, so the browser reads it directly: no
// backend and no copy at build time.

import { parsePbn } from './deal.js'

export const PBS_RAW =
  'https://raw.githubusercontent.com/bridge-craftwork/Practice-Bidding-Scenarios/main'

/** `!S` and friends in PBS text as suit symbols. */
export function suitSymbols(s) {
  return String(s ?? '').replace(/!C/g, '♣').replace(/!D/g, '♦')
    .replace(/!H/g, '♥').replace(/!S/g, '♠')
}

function firstChatLine(chat) {
  const line = String(chat ?? '').split(/\\n|\n/).map((l) => l.trim())
    .find((l) => l && !/^---\s*$/.test(l))
  return suitSymbols((line ?? '').replace(/^---\s*/, ''))
}

/**
 * The release menu: `{sections: [{label, items: [{name, label}]}],
 * scenarios: {name: {label, title, chat, ns, ew, dlr}}}`.
 */
let manifestPromise = null
export function fetchManifest(fetchImpl = fetch) {
  manifestPromise ??= readManifest(fetchImpl)
  manifestPromise.catch(() => { manifestPromise = null })
  return manifestPromise
}

async function readManifest(fetchImpl) {
  const resp = await fetchImpl(`${PBS_RAW}/manifest/manifest-release.json`)
  if (!resp.ok) throw new Error(`the PBS scenario list did not load (HTTP ${resp.status})`)
  const m = await resp.json()
  const scenarios = {}
  for (const [name, sc] of Object.entries(m.scenarios ?? {})) {
    if (sc.missing) continue
    scenarios[name] = {
      label: suitSymbols(sc.buttonText || name.replace(/[_-]/g, ' ')),
      title: firstChatLine(sc.chat),
      chat: suitSymbols(String(sc.chat ?? '').replace(/\\n/g, '\n')).trim(),
      ns: sc.conventionCardNS || null,
      ew: sc.conventionCardEW || null,
      dlr: /^[A-Za-z0-9_-]+\/[A-Za-z0-9_.-]+\.dlr$/.test(sc.dlr ?? '') ? sc.dlr : `dlr/${name}.dlr`,
    }
  }
  const sections = []
  let cur = null
  for (const node of m.layout ?? []) {
    if (node.type === 'section') {
      cur = { label: node.title, items: [] }
      sections.push(cur)
    } else if (node.type === 'row' && cur) {
      // A row's later buttons are often variants labelled only by what
      // differs ("(Lev)", "13", "then Stayman"): name them with the first.
      let head = null
      for (const b of node.buttons ?? []) {
        if (!b.name || b.name === '---' || !scenarios[b.name]) continue
        let label = scenarios[b.name].label
        if (head && (label.length <= 6 || /^[(=]|^then /.test(label))) label = `${head} ${label}`
        else head ??= label
        scenarios[b.name].label = label
        cur.items.push({ name: b.name, label })
      }
    }
  }
  return { sections: sections.filter((s) => s.items.length), scenarios }
}

const corpusCache = new Map()

/** A scenario's 500 corpus boards, with BBA's auctions and notes. */
export async function fetchCorpus(name, fetchImpl = fetch) {
  if (!/^[A-Za-z0-9_.-]+$/.test(name)) throw new Error(`not a scenario name: ${name}`)
  if (!corpusCache.has(name)) {
    const p = (async () => {
      const resp = await fetchImpl(`${PBS_RAW}/bba/${name}.pbn`)
      if (!resp.ok) throw new Error(`scenario ${name}: no deals (HTTP ${resp.status})`)
      const text = await resp.text()
      const cc = (n) => (new RegExp(`^%\\s*CC${n}\\s*-\\s*(.+)$`, 'm').exec(text)?.[1] ?? '')
        .split(/[\\/]/).pop().replace(/\.bbsa\s*$/i, '').trim() || null
      return { boards: parsePbn(text), cards: { ns: cc(1), ew: cc(2) } }
    })()
    corpusCache.set(name, p)
    p.catch(() => corpusCache.delete(name))
  }
  return corpusCache.get(name)
}

/** A scenario's dealer script. */
export async function fetchScript(path, fetchImpl = fetch) {
  const resp = await fetchImpl(`${PBS_RAW}/${path}`)
  if (!resp.ok) throw new Error(`${path}: HTTP ${resp.status}`)
  return resp.text()
}

/** A PBS card that is not built in, as `.bbsa` text. */
export async function fetchBbsa(name, fetchImpl = fetch) {
  if (!/^[A-Za-z0-9_.-]+$/.test(name)) throw new Error(`not a card name: ${name}`)
  const resp = await fetchImpl(`${PBS_RAW}/bbsa/${name}.bbsa`)
  if (!resp.ok) throw new Error(`card ${name}: HTTP ${resp.status}`)
  return resp.text()
}

/** The cards a dealer script names in its `# convention-card-ns:` lines. */
export function scriptCards(script) {
  const get = (side) =>
    new RegExp(`^#\\s*convention-card-${side}\\s*:\\s*(\\S+)`, 'mi').exec(script ?? '')?.[1] ?? null
  return { ns: get('ns'), ew: get('ew') }
}
