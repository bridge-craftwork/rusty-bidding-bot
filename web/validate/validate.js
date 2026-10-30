// The rule checker: `.bid` files opened or typed here, checked by the
// engine's `validate` (the same checks as `rbb bid check`), on their own or
// added to Rusty's rule set (rules/index.json, published by build.sh).
// Nothing is sent anywhere; the files live in this page.

import init, * as rbb from '../pkg/rbb_wasm.js'

await init()

const $ = (id) => document.getElementById(id)
const esc = (s) => String(s ?? '').replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c])

const info = JSON.parse(rbb.info())
$('language').textContent =
  `Engine ${info.version}, rule language ${info.language}. Every term a condition may use: rbb bid terms (docs/CONTRACT.md).`

// Rusty's rule set, as text: { files: [path...], manifest: "conventions.toml" }.
let ours = null
async function oursRules() {
  if (ours) return ours
  const index = await (await fetch('rules/index.json')).json()
  const texts = await Promise.all(index.files.map((p) => fetch('rules/' + p).then((r) => r.text())))
  ours = { files: Object.fromEntries(index.files.map((p, i) => [p, texts[i]])), manifest: await fetch('rules/conventions.toml').then((r) => r.text()) }
  return ours
}

// The files: name → text, one selected.
const files = new Map([['mine/convention.bid', '']])
let selected = 'mine/convention.bid'

// Line numbers, the lines a diagnostic names marked.
let marked = new Map()
let lastDiags = []
function renderGutter() {
  const n = $('source').value.split('\n').length
  let nums = ''
  let rows = ''
  for (let i = 1; i <= n; i++) {
    const sev = marked.get(i)
    nums += (sev ? `<span class="hit ${sev}">${i}</span>` : i) + '\n'
    rows += `<div${sev ? ` class="${sev}"` : ''}></div>`
  }
  $('gutter').innerHTML = nums
  $('backdrop').innerHTML = rows
  syncScroll()
}
function syncScroll() {
  $('gutter').scrollTop = $('source').scrollTop
  $('backdrop').scrollTop = $('source').scrollTop
}
// The worst severity on each line of the selected file.
function markLines() {
  marked = new Map()
  for (const d of lastDiags) {
    if (d.file !== selected || !d.line) continue
    if (d.severity === 'error' || !marked.has(d.line)) marked.set(d.line, d.severity === 'error' ? 'error' : 'warning')
  }
}
$('source').addEventListener('scroll', syncScroll)

function renderTabs() {
  $('tabs').innerHTML = [...files.keys()]
    .map((n) => `<button type="button" role="tab" class="tab${n === selected ? ' on' : ''}" aria-selected="${n === selected}" data-name="${esc(n)}">${esc(n)}</button>`)
    .join('')
  $('name').value = selected
  $('source').value = files.get(selected) ?? ''
  markLines()
  renderGutter()
}

$('tabs').addEventListener('click', (e) => {
  const b = e.target.closest('button[data-name]')
  if (!b) return
  selected = b.dataset.name
  renderTabs()
})

$('source').addEventListener('input', () => {
  files.set(selected, $('source').value)
  renderGutter()
  schedule()
})

$('name').addEventListener('change', () => {
  const name = $('name').value.trim()
  if (!name || name === selected || files.has(name)) {
    $('name').value = selected
    return
  }
  const text = files.get(selected)
  files.delete(selected)
  files.set(name, text)
  selected = name
  renderTabs()
  schedule()
})

$('add').addEventListener('click', () => {
  let n = 1
  while (files.has(`mine/new-${n}.bid`)) n++
  selected = `mine/new-${n}.bid`
  files.set(selected, '')
  renderTabs()
})

$('remove').addEventListener('click', () => {
  files.delete(selected)
  if (!files.size) files.set('mine/convention.bid', '')
  selected = files.keys().next().value
  renderTabs()
  schedule()
})

async function openFiles(list) {
  const chosen = [...list].filter((f) => /\.(bid|toml)$/.test(f.name))
  if (!chosen.length) return
  if (files.size === 1 && !files.get(selected)) files.clear()
  for (const f of chosen) {
    const name = f.webkitRelativePath || f.name
    files.set(name, await f.text())
  }
  selected = chosen[0].webkitRelativePath || chosen[0].name
  renderTabs()
  schedule()
}
$('files').addEventListener('change', (e) => openFiles(e.target.files))
$('folder').addEventListener('change', (e) => openFiles(e.target.files))

// Examples: our own modules, to start from.
oursRules().then(({ files: rules }) => {
  $('example').innerHTML += Object.keys(rules)
    .map((p) => `<option value="${esc(p)}">${esc(p)}</option>`)
    .join('')
})
$('example').addEventListener('change', async () => {
  const p = $('example').value
  if (!p) return
  const { files: rules } = await oursRules()
  if (files.size === 1 && !files.get(selected)) files.clear()
  files.set(p, rules[p])
  selected = p
  $('example').value = ''
  renderTabs()
  schedule()
})

document.querySelectorAll('input[name=mode]').forEach((r) => r.addEventListener('change', schedule))

// ── Checking ──────────────────────────────────────────────────────────

let timer = 0
function schedule() {
  clearTimeout(timer)
  timer = setTimeout(check, 250)
}

async function check() {
  const mode = document.querySelector('input[name=mode]:checked').value
  const mine = Object.fromEntries([...files].filter(([n, t]) => n.endsWith('.bid') && t.trim()))
  const manifest = [...files].find(([n]) => n.endsWith('conventions.toml'))?.[1]
  if (!Object.keys(mine).length) {
    $('summary').innerHTML = '<p class="hint">Open, choose or type a <code>.bid</code> file.</p>'
    $('diagnostics').innerHTML = ''
    lastDiags = []
    marked = new Map()
    renderGutter()
    return
  }
  const req = {}
  if (mode === 'with') {
    const base = await oursRules()
    req.rules = { ...base.files, ...mine }
    req.manifest = manifest ?? base.manifest
  } else {
    req.rules = mine
    if (manifest) req.manifest = manifest
  }
  let res
  try {
    res = JSON.parse(rbb.validate(JSON.stringify(req)))
  } catch (e) {
    res = { ok: false, diagnostics: [{ severity: 'error', message: String(e?.message ?? e) }] }
  }
  render(res, mine, mode)
}

function render(res, mine, mode) {
  // With our rule set, show what concerns the files here, and count the rest.
  const diags = res.diagnostics ?? []
  const mineDiag = (d) => !d.file || d.file in mine || Object.keys(mine).some((n) => d.message.includes(n))
  const shown = mode === 'with' ? diags.filter(mineDiag) : diags
  const hidden = diags.length - shown.length
  const errors = shown.filter((d) => d.severity === 'error').length
  const warnings = shown.filter((d) => d.severity === 'warning').length
  const rs = res.rule_set
  const verdict = res.ok && !errors
    ? `<p class="verdict ok">✓ ${warnings ? `Compiles, with ${warnings} warning${warnings > 1 ? 's' : ''}.` : 'No problems found.'}</p>`
    : `<p class="verdict bad">✗ ${errors} error${errors === 1 ? '' : 's'}${warnings ? `, ${warnings} warning${warnings > 1 ? 's' : ''}` : ''}.</p>`
  const counts = rs
    ? `<p class="hint">${rs.modules} modules, ${rs.rules} rules in ${rs.rule_files} files${mode === 'with' ? ` (yours: ${Object.keys(mine).length})` : ''}, rule language ${rs.language}.</p>`
    : ''
  const more = hidden ? `<p class="hint">${hidden} more in Rusty's own files, not shown.</p>` : ''
  $('summary').innerHTML = verdict + counts + more
  lastDiags = shown
  markLines()
  renderGutter()
  $('diagnostics').innerHTML = shown
    .map((d) => {
      const where = d.file ? `${d.file}${d.line ? ':' + d.line : ''}${d.col ? ':' + d.col : ''}` : ''
      const jump = d.file && files.has(d.file) && d.line ? ` data-file="${esc(d.file)}" data-line="${d.line}"` : ''
      // The message repeats the place (`rules: <file>:<line>: ...`): once is enough.
      const message = d.file ? d.message.replace(/^rules: [^:]+(:\d+)*: /, '') : d.message
      return `<li class="diag ${esc(d.severity)}"${jump}><span class="sev">${esc(d.severity)}</span> ${where ? `<code>${esc(where)}</code> ` : ''}${esc(message)}${d.hint ? `<div class="hint">${esc(d.hint)}</div>` : ''}</li>`
    })
    .join('')
}

// Click a diagnostic to go to its line.
$('diagnostics').addEventListener('click', (e) => {
  const li = e.target.closest('li[data-file]')
  if (!li) return
  selected = li.dataset.file
  renderTabs()
  const ta = $('source')
  const lines = ta.value.split('\n')
  const n = Math.max(0, Number(li.dataset.line) - 1)
  const start = lines.slice(0, n).reduce((a, l) => a + l.length + 1, 0)
  ta.focus()
  ta.setSelectionRange(start, start + (lines[n]?.length ?? 0))
  ta.scrollTop = Math.max(0, (n - 3) * parseFloat(getComputedStyle(ta).lineHeight || '18'))
})

renderTabs()
check()
