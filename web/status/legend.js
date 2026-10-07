// The status page's vocabulary (no DOM): every status a tile or a
// treatment can have, in legend order, with its glyph and what it means.
// probes/tools/status_data.py writes the codes; web/scripts/test.mjs
// checks that status.json uses no other.

export const STATUS = [
  { code: 'conv-good', glyph: '▲', label: 'Convention: good', kind: 'conv' },
  { code: 'conv-fair', glyph: '◆', label: 'Convention: fair', kind: 'conv' },
  { code: 'conv-poor', glyph: '▼', label: 'Convention: poor', kind: 'conv' },
  { code: 'bba-good', glyph: '✓', label: 'Background: good', kind: 'bba' },
  { code: 'bba-fair', glyph: '≈', label: 'Background: fair', kind: 'bba' },
  { code: 'bba-poor', glyph: '✗', label: 'Background: poor', kind: 'bba' },
  { code: 'ab-gains', glyph: '↑', label: 'A/B: gains', kind: 'ab' },
  { code: 'ab-neutral', glyph: '=', label: 'A/B: neutral', kind: 'ab' },
  { code: 'ab-loses', glyph: '↓', label: 'A/B: loses', kind: 'ab' },
  { code: 'tested', glyph: '◇', label: 'Implemented, tests only', kind: 'impl' },
  { code: 'partial', glyph: '◐', label: 'Partial', kind: 'impl' },
  { code: 'gap', glyph: '○', label: 'Not implemented', kind: 'gap' },
]

export const BY_CODE = Object.fromEntries(STATUS.map((s) => [s.code, s]))

export const ABOUT = {
  'conv-good': 'The convention score: the same deals bid by BBA and by us with the convention and without it; what it gains us minus what it gains BBA, on the boards it changed, is no worse than {conv_good} IMPs a changed board.',
  'conv-fair': 'The convention score: between good and poor.',
  'conv-poor': 'The convention score: it gains us at least {conv_poor_abs} IMPs a changed board less than it gains BBA, and both halves of the boards agree.',
  'bba-good': 'The background: the scenarios that test it scored against BBA, with par as the yardstick: errors no worse than {good_epb} IMPs a board and {good_agree}% of North-South calls agreeing. It measures the bidding around a convention as much as the convention; the small square on a tile with a convention score.',
  'bba-fair': 'The background against BBA: between good and poor.',
  'bba-poor': 'The background against BBA: worse than {poor_epb} IMPs a board, or under {poor_agree}% of North-South calls agreeing.',
  'ab-gains': 'BBA does not play it: our engine with it against our engine without it, on the same deals. Fewer errors with it (halves agree, |z| at least 1; "leans" under 2).',
  'ab-neutral': 'A/B against ourselves: no clear difference.',
  'ab-loses': 'A/B against ourselves: more errors with it.',
  tested: 'Rules written and covered by test cases; nothing measures it on deals yet.',
  partial: 'Rules for part of it; the details say what is missing.',
  gap: 'Not implemented: no module, or no rule reads the card setting.',
}

/** Level bands, as convention-card names them (derived, never stored). */
export function band(level) {
  if (!level) return { id: 'none', name: 'No level' }
  if (level <= 3) return { id: 'basic', name: 'Basic (levels 1–3)' }
  if (level <= 6) return { id: 'intermediate', name: 'Intermediate (4–6)' }
  if (level <= 8) return { id: 'advanced', name: 'Advanced (7–8)' }
  return { id: 'expert', name: 'Expert (9–10)' }
}

/** The about text with the thresholds filled in. */
export function about(code, thresholds) {
  const t = thresholds ?? {}
  const pct = (x) => Math.round((x ?? 0) * 100)
  return (ABOUT[code] ?? '')
    .replace('{conv_good}', String(t.conv?.good ?? ''))
    .replace('{conv_poor_abs}', String(Math.abs(t.conv?.poor ?? 0)))
    .replace('{good_epb}', String(t.good?.errors_per_board ?? ''))
    .replace('{good_agree}', String(pct(t.good?.ns_agree)))
    .replace('{poor_epb}', String(t.poor?.errors_per_board ?? ''))
    .replace('{poor_agree}', String(pct(t.poor?.ns_agree)))
}

/** Whether tile `t` shows status `code`: as its own status, or (a
 *  background code) as the background marker beside a convention score. */
export function shows(t, code) {
  return t.status === code || t.background === code
}

/** Problems with a status.json, for the tests: [] when it is sound. */
export function checkData(data) {
  const problems = []
  if (data?.schema !== 'rbb-status/1') problems.push(`schema ${data?.schema}`)
  const sections = new Set((data?.sections ?? []).map((s) => s.id))
  if (!Array.isArray(data?.tiles) || !data.tiles.length) problems.push('no tiles')
  const ids = new Set()
  for (const t of data?.tiles ?? []) {
    if (!t.id || ids.has(t.id)) problems.push(`tile id ${t.id} missing or repeated`)
    ids.add(t.id)
    if (!BY_CODE[t.status]) problems.push(`${t.id}: status ${t.status}`)
    // The second marker: the background beside a convention score.
    if (t.background != null && (BY_CODE[t.background]?.kind !== 'bba' || BY_CODE[t.status]?.kind !== 'conv')) {
      problems.push(`${t.id}: background ${t.background} beside ${t.status}`)
    }
    if (!sections.has(t.section)) problems.push(`${t.id}: section ${t.section}`)
    if (!t.name) problems.push(`${t.id}: no name`)
    for (const tr of t.treatments ?? []) {
      if (!BY_CODE[tr.status]) problems.push(`${t.id}: treatment ${tr.label}: status ${tr.status}`)
    }
  }
  return problems
}
