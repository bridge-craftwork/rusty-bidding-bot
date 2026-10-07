// reference.txt: the tool described as plain text, for readers without
// JavaScript (people and assistants). Written from the same INPUT and
// PARAMS tables the page builds its controls from, the engine's info(),
// and the engine's own conventions reference, so it cannot describe a
// different tool from the one served next to it.

import { GLOBAL, INPUT, PARAMS, TOOL } from './tool.js'

function wrap(text, width, indent) {
  const words = String(text).split(/\s+/).filter(Boolean)
  const lines = []
  let line = ''
  for (const w of words) {
    if (line && (line + ' ' + w).length > width) {
      lines.push(line)
      line = w
    } else {
      line = line ? line + ' ' + w : w
    }
  }
  if (line) lines.push(line)
  return lines.map((l, i) => (i ? indent : '') + l).join('\n')
}

function table(rows) {
  return rows.map((r) => {
    const head = `  ${r.name}`.padEnd(14)
    const lines = [head + wrap(r.form ?? r.values?.join(' | ') ?? '', 62, ' '.repeat(14))]
    if (r.default !== undefined) lines.push(' '.repeat(14) + `default: ${typeof r.default === 'string' ? r.default : JSON.stringify(r.default)}`)
    lines.push(' '.repeat(14) + wrap(r.about, 62, ' '.repeat(14)))
    return lines.join('\n')
  }).join('\n\n')
}

/**
 * The whole text. `info` is the engine's info(); `conventions` the text of
 * its reference("{}").
 */
export function renderReference({ info, conventions }) {
  const title = `${TOOL} reference: engine ${info.version}, API ${info.api}, rules ${info.rules_id}`
  return `${title}
${'='.repeat(title.length)}

A rule-based bridge bidding engine (Rust, compiled to WebAssembly) bidding
all four hands of a deal in the browser, seat by seat, with the convention
cards you choose. Generated from the page's own vocabulary and the engine it
ships, so it describes exactly the tool served beside it.

  Page:    https://bridge-craftwork.com/${TOOL}/
  Status:  https://bridge-craftwork.com/${TOOL}/status.html (data: status/status.json)
  Source:  https://github.com/bridge-craftwork/rusty-bidding-bot
  Engine:  ${info.modules} convention modules, ${info.rules} rules

PAGE
----

Pick a deal (a Practice-Bidding-Scenarios scenario, a random deal, a
dealer3 script or a deal of your own), the two partnerships' convention
cards, and let the engine bid: one call at a time or to the end. Undo takes
back the last call; the bidding box makes a call of your own, and the
engine carries on from it. Alerted and announced calls are marked; the
meanings of all calls can be shown. The x-ray shows, under the auction,
every rule the engine weighed for a call (its priority, descriptiveness and
why it lost), what each seat had shown and each side's forcing state
before it, and how the engine reads BBA's auction. When the auction ends every hand is
shown with the double-dummy table, par and the contract's result. For a
scenario, BBA's auction of the same deal is shown beside ours. Settings a
card switches on that no rule reads (conventions and treatments the engine
does not play) are flagged.

DEEP LINKS
----------

The page's state is in the URL fragment (never the query string), as
key=value pairs joined by &, values URI-encoded: every INPUT field and
every PARAMS field below whose value is text, a number or true/false.
Examples:

  #scenario=Stayman&board=12
  #random=true&seed=42&board=3&view=S
  #deal=N%3AAK52.KJ7.Q94.K83%20QJ3.Q95.KJ3.QJ74%20T64.AT832.A2.T62%20987.64.T8765.A95&dealer=N&vul=None&auction=1NT%20Pass&ns=21GF-DEFAULT&ew=Precision
  #scenario=Jacoby_2N&board=4&rotate=2&stop=3
  #scenarioScript=Stayman&seed=7
  #scenario=Stayman&board=12&xray=1

stop=N shows the position after N calls (the engine is deterministic, so
the link reproduces them). xray=1 opens the x-ray under the auction (the
rules the engine weighed for a call, what each seat had shown, the forcing
state); xray=all shows it for every call. set is written path=value;path=value. An
uploaded card cannot travel in a link.

JAVASCRIPT API
--------------

window.${GLOBAL} = { run, validate, getInput, setInput, getOutput }

  run(input, params)       Promise of the output. Loads input (null keeps
                           the current deal), applies params, bids to
                           params.stop and solves the deal when it ends.
                           The page shows the result.
  validate(input, params)  Promise of {ok, diagnostics}: checks without
                           bidding (the deal, dealer, auction legality,
                           cards, params; a dealer3 script's syntax).
  getInput()               The input that reproduces the page's position.
  setInput(input)          Promise: load input into the page (the calls
                           in input.auction; the engine does not bid).
  getOutput()              The page's state as data (below).

input: an object with the fields below, or a string (a PBN deal
"N:<N> <E> <S> <W>", or PBN text of one board).

${table(INPUT)}

params:

${table(PARAMS)}

output:

  ok            false when a diagnostic is an error
  source        "scenario" | "random" | "script" | "deal"
  scenario, board
  deal          PBN deal, after rotation; hands {N, E, S, W}
  dealer, vul, scoring, rotate
  cards         {ns, ew}: the names of the cards in play
  calls         [{seat, call, by, why, meaning, alert, rule, noRule, shows}]
                by: "engine" | "you" | "forced" (from input.auction);
                why: the engine's reason for its call; meaning: what the
                call shows to the table; alert: null or {kind: "alert" |
                "announce", text}; noRule: no rule explains the call (a gap
                in the engine's system); shows: the caller's hand as known
                after the call
  complete, next, contract, declarer
  dd            null, or {tricks: {N: {C, D, H, S, NT}, E, S, W},
                par: {score_ns, contracts}, result: {contract, declarer,
                tricks, score_ns}}
  bba           for a scenario, BBA's auction of the deal:
                {auction: [{seat, call, note}]}
  coverage      {ns, ew}: of the settings each card switches on, read (the
                rules use them), ignored (no rule reads them: not played),
                unmapped (settings with no card field: .bbsa keys, card
                JSON paths), other (carding, leads), score = read /
                (read + ignored)
  xray          null unless params.xray is 1 or "all"; then
                {calls: [{index, seat, call, by, hand, engineCall,
                engineWhy, offered, candidates, warnings, before,
                reading}], now, bba}. For each call: candidates are every
                rule the engine weighed for the caller's hand there,
                best-ranked first: {call, explanation, priority,
                descriptiveness, prefer, outcome, rule: {module, file,
                line}}, outcome "chosen", "outranked", "partner's call is
                forcing" or why the hand fails ("hand fails \`shows
                hcp>=17\`"); engineCall is what the engine bids with that
                hand there (for a call it did not make, too) and offered
                whether any rule offers the call actually made; before is
                the table before the call: {knowledge: {N, E, S, W: {hcp:
                {min, max}, lengths: {S, H, D, C}, balanced, summary,
                shown}}, sides: {ns, ew: {trump, forcing: "none" |
                "round" | "game", forcing_by, ask, answered, summary}}};
                reading is how the table reads the call: {explanation,
                alert, rule, artificial, knowledge (the caller's, after
                it)}. now is the table after the last call, in the shape
                of before. bba (a scenario) is BBA's auction as the engine
                reads it: [{seat, call, note, reading, engineCall}].
  diagnostics   [{severity, message, line?, col?, hint?}]

DIAGNOSTICS
-----------

{severity: "error" | "warning" | "info", message, line?, col?, hint?}.
message starts with the field it concerns (deal.N:, auction:, cards.ns:,
params.scoring:, script:). line and col are 1-based: the column of a call
in an auction string, the line of a key in a .bbsa card, the line and
column in a dealer script. The page lists them under the table.

CARDS
-----

Built in (the Practice-Bidding-Scenarios .bbsa cards):
${wrap(info.stock_cards.join(', '), 74, '')}

Also: a BBA .bbsa file ({"bbsa": text}), or a Bridge-Classroom convention
card ({"json": card_data}, or the editor's export {"json": {schema, name,
card_data}}: keys such as _bbo_raw are ignored); BC-21-Intermediate is
Bridge-Classroom's 2/1 Intermediate card, built in. The page converts any
card to either format (its JSON download is Bridge-Classroom's export).

CONVENTIONS
-----------

The engine's rules, module by module: the card fields that switch each
module on, and every call it knows with what the call shows.

${conventions.trimEnd()}
`
}
