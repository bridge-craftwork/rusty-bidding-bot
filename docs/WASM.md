# rusty-bidding-bot in the browser (WASM API)

`crates/wasm` (crate `rbb-wasm`) builds the engine for the browser. The rules
(`conventions/**/*.bid`), their manifest and card vocabulary
(`conventions/conventions.toml`, `conventions/card/*.toml`) and the stock
convention cards are compiled into the `.wasm` file: nothing is fetched or
read at run time. A caller may also supply a rule set of its own as text
([Rule sets supplied at run time](#rule-sets-supplied-at-run-time)). The API
is JSON in, JSON out, and never throws on bad input: problems come back as
`diagnostics`.

API version: **1** (`info().api`). A change to a request or response shape
that could break a caller bumps it; new optional fields do not.

## Build

```sh
rustup target add wasm32-unknown-unknown   # once
cargo install wasm-pack                    # once
crates/wasm/build.sh                       # release; --dev for a quick build
```

Output: `crates/wasm/pkg/` (gitignored), an ES module package:

| File | What |
|---|---|
| `rbb_wasm.js` | JS glue, `export default init` plus the functions below |
| `rbb_wasm_bg.wasm` | the engine, rules, cards and the double-dummy solver (about 1.4 MB; 430 KB gzipped) |
| `rbb_wasm.d.ts` | TypeScript declarations |
| `package.json` | for `npm install ./pkg` or a bundler |

`RBB_WASM_OUT=/some/dir crates/wasm/build.sh` writes the package elsewhere.
The tool is `wasm-pack build --target web`, as in bridge-rulebot. CI builds
it on every push (`.github/workflows/ci.yml`, artifact `rbb-wasm-pkg`) and
attaches `rbb-wasm.tar.gz` (the package plus `demo.html`) to each release.

Smoke test: `python3 -m http.server -d crates/wasm`, then open
<http://localhost:8000/demo.html>. It bids one hand. (WASM does not load
from `file://`.)

## Embed

```html
<script type="module">
  import init, * as rbb from "./pkg/rbb_wasm.js";
  await init();                       // fetches rbb_wasm_bg.wasm next to the .js

  const engine = JSON.parse(rbb.createEngine(JSON.stringify({
    cards: { ns: "21GF-DEFAULT", ew: "21GF-GIB" },
  }))).engine;

  const res = JSON.parse(rbb.bid(JSON.stringify({
    engine, hand: "AK52.KJ7.Q94.K83", dealer: "N", vul: "None",
    scoring: "IMP", auction: "",
  })));
  if (res.ok) console.log(res.call, res.explanation);   // "1NT", "15-17 HCP, balanced"
  else console.log(res.diagnostics);
</script>
```

With a bundler, `init()` can be given the `.wasm` URL
(`init(new URL("rbb_wasm_bg.wasm", import.meta.url))`), or bytes
(`initSync({ module: bytes })` in a worker or in Node).

All functions are synchronous once `init()` has resolved. `bidDeal` takes
around 100-150 ms for a whole auction on a laptop; a single `bid` a few ms to
tens of ms (the engine caches per position, so repeated work is fast). Run it
in a Web Worker if the page must stay responsive while bidding many deals.

The engine is single-threaded; state (the engines made by `createEngine`)
lives in the module instance. Two instances (e.g. two workers) do not share
engines.

## Conventions used in every call

- Every function takes **one string**: a JSON object (the request) and
  returns **one string**: a JSON object (the response). Only `reference`
  returns plain text. An empty string is an empty request.
- Every JSON response has `ok` (boolean: no diagnostic is an error) and
  `diagnostics` (array, possibly empty). The other fields are always present
  (as `null`, `[]`, or empty) even when `ok` is false, so a UI can render
  either way.
- Nothing throws for bad input. (A panic, which would be a bug, is logged to
  the console by `console_error_panic_hook` and raises a JS `RuntimeError`.)

### Diagnostic

```json
{"severity": "error", "message": "auction: call 2 (1C) is not legal here (E to call)",
 "line": 1, "col": 5,
 "hint": "a bid must outrank the last one; X only over an opponent's bid, XX only over their X"}
```

| Field | Type | |
|---|---|---|
| `severity` | `"error"` \| `"warning"` \| `"info"` | `error`: the request cannot be answered (`ok` is false). `warning`: answered, but something was ignored or doubtful (an unknown card field, a rule problem met while bidding). `info`: a remark (card keys the rules do not use, "no rule applies"). |
| `message` | string | Starts with the request field it concerns: `hand:`, `deal.N:`, `auction:`, `prefix:`, `cards.ns:`, `engine:`, `dealer:`, `vul:`, `scoring:`, `rules:`, `fields:`, `bbsa_map:`, `manifest:`, `request`. Problems in a supplied rule file read `rules: <file>:<line>: ...`. Warnings from the rules name a rule file and line (`conventions/x.bid:12: ...`) or a call (`call 3 (2C): ...`). |
| `file` | string, optional | For a problem in a rule file supplied at run time: its name (the key in `rules`); `line` and `col` are then in that file. |
| `line` | integer, optional | 1-based. For an auction string, always 1. For a `.bbsa` card, the line of the key. For invalid request JSON, the line in the request. For `file`, the line in that file; for `fields` / `bbsa_map`, the line in that text (TOML syntax errors). |
| `col` | integer, optional | 1-based character column: of the offending call in an auction **string** (not given for an auction array), or in the request JSON. |
| `hint` | string, optional | How to fix it (the accepted forms, the list of stock cards). |

This is the bridge-craftwork tool contract's diagnostic shape; `validate`
returns exactly these, so a site's `validate(input, params)` can pass them
through.

### Values

| Name | Form | Notes |
|---|---|---|
| hand | `"AK52.KJ7.Q94.K83"` | PBN order spades.hearts.diamonds.clubs; ranks `AKQJT98765432`; `-` or empty for a void. Exactly 13 distinct cards. |
| deal | `"N:<N> <E> <S> <W>"` or `{"N": hand, "E": hand, "S": hand, "W": hand}` | PBN deal (the first letter is the seat of the first hand, clockwise after it). 52 distinct cards. |
| seat | `"N"`, `"E"`, `"S"`, `"W"` | only the first letter is read (`"North"` works). |
| vul | `"None"`, `"NS"`, `"EW"`, `"All"` | also `"Both"`, `"-"`. Default `"None"`. |
| scoring | `"MP"`, `"IMP"` | Default `"MP"`. |
| auction | `"1NT Pass 2C"` or `["1NT", "Pass", "2C"]` | Calls from the dealer on. `1C`..`7NT` (`1N` also), `Pass`/`P`, `X`, `XX`. Separated by spaces or commas. Checked for legality. |
| call (in responses) | `"1NT"`, `"Pass"`, `"X"`, `"XX"` | Always this spelling. |
| card spec | see below | |

### Cards

`cards` names the convention cards of the two partnerships:

```json
{"ns": <card spec>, "ew": <card spec>, "set": ["general.style=bba"],
 "ns_set": ["notrump.stayman.play=false"], "ew_set": []}
```

- `ns` is required; `ew` defaults to the same card.
- `set` changes apply to both sides, then `ns_set` / `ew_set` to one:
  `path=value` with `true`/`false`, integers, or text. Paths are card fields
  of the rule set's vocabulary (for the embedded rules `rbb card schema`,
  `conventions/card/fields.toml`). An unknown path is an error.

Every card, however given, is read in the card vocabulary of the rules it
is used with: the embedded one, or the `fields` and `bbsa_map` of a rule
set supplied at run time. The same `.bbsa` file can therefore give
different cards under different rule sets.

A **card spec** is one of:

| Spec | Meaning |
|---|---|
| `"21GF-DEFAULT"` | a stock card built in: the names are `info().stock_cards` |
| `{"stock": "21GF-DEFAULT"}` | the same |
| `{"bbsa": "<the text of a .bbsa file>", "name": "My card"}` | a BBA card file (`name` optional) |
| `{"json": {...}}` or `{"json": "<string>"}` | Bridge-Classroom card JSON: bare `card_data`, or the editor's export (`{"schema": "bridge-classroom/card_data@v1", "name", "description", "exportedAt", "card_data"}`, whose `name` becomes the card's) |

Loading a card reports: `.bbsa` keys with no card field (one `info`, the
line of the first), unknown JSON card fields (`warning`, ignored), keys
starting with `_` such as `_bbo_raw`, the raw record of a BBO import (one
`info`: not card settings, kept so an export round-trips), old field
names (`info`), invalid values (`warning`, ignored), and a file that is not a
card at all, or an export `schema` other than `@v1` (`error`).

## Functions

Exported names are camelCase in JS; the Rust names in `crates/wasm/src/api.rs`
are snake_case.

### `info()` → JSON

No argument.

```json
{"ok": true, "api": 1, "version": "0.1.0", "rules_id": "d737c980a4bbdb8c",
 "rule_files": 34, "modules": 34, "rules": 1658, "language": 1,
 "stock_cards": ["21GF-DEFAULT", "21GF-GIB", "..."], "diagnostics": []}
```

`rules_id` is a hash of the embedded rule files, manifest and card
vocabulary: it changes whenever one of them does, so it can key caches and
label results. `language` is the rule language version this engine reads
(docs/CONTRACT.md); a rule set whose manifest asks for another is refused.

### `createEngine(request)` → JSON

Request: `{"cards": {...}}`, plus optionally a rule set (`rules`, `fields`,
`bbsa_map`, `manifest`: [below](#rule-sets-supplied-at-run-time)); without
one, the embedded rules.

```json
{"ok": true, "engine": 1, "rules_id": "d737c980a4bbdb8c",
 "ns": {"name": "21GF-DEFAULT", "modules": ["base", "notrump-base", "stayman", "..."]},
 "ew": {"name": "21GF-GIB", "modules": ["..."]},
 "diagnostics": [{"severity": "info", "message": "cards.ns: 27 .bbsa keys have no card field ...", "line": 12}]}
```

`engine` is a handle (a small integer) for the other calls; `null` when
`ok` is false. `rules_id` identifies its rule set. `modules` lists the
modules each side plays (active under its card). Calling `createEngine`
again with the same `cards` (same JSON text) and the same rule set returns
the same handle without rebuilding. Making an engine takes a few ms (plus
compiling a supplied rule set the first time it is seen).

### `freeEngine(request)` → JSON

Request: `{"engine": 1}`. Response: `{"ok": true, "diagnostics": []}`
(a `warning` if there was no such engine).

### Engine selection in requests

`bid`, `interpret`, `bidDeal`, `conventions` and `reference` take either
`"engine": <handle>` or `"cards": {...}` (which makes or reuses the engine
for those cards, and the rule set the request supplies, as `createEngine`
does). `engine` wins when both are given. `conventions`, `reference`,
`coverage`, `exportCard` and `validate` use the rules (and so the card
vocabulary) of the `engine` they are given, else the rule set the request
supplies, else the embedded one.

### Rule sets supplied at run time

Any request that can make an engine or read cards (`createEngine`,
`validate`, `bid`, `interpret`, `bidDeal` with `cards`, `conventions`,
`reference`, `coverage`, `exportCard`) may carry a rule set as text instead
of using the embedded one:

```json
{"rules": {"demo/one-nt.bid": "module demo \"Demo\"\n  card demo.strong_nt\n...",
           "demo/one-nt.notes.md": "..."},
 "fields": "[demo]\n\"strong_nt\" = { kind = \"bool\", label = \"1NT\", default = true }\n",
 "bbsa_map": "",
 "manifest": "name = \"demo\"\nlanguage = 1\n",
 "cards": {"ns": {"json": {}}}}
```

| Field | | When left out |
|---|---|---|
| `rules` | the rule files: an object from file name to source. Only names ending in `.bid` are compiled (others are ignored, with an `info`), in the order `rbb` reads a directory: by path, component by component (file order is the last tie-breaker between rules). The names appear in `rule.file` and in diagnostics. | the embedded `.bid` files |
| `fields` | the text of the rule set's `card/fields.toml` | the embedded one |
| `bbsa_map` | the text of its `card/bbsa-map.toml` (`""` maps no key: every `.bbsa` key is kept as passthrough) | the embedded one |
| `manifest` | the text of its `conventions.toml` | with `rules`: none (an `info` says the rules are read as the current language, as `rbb` reads a directory without one); without `rules`: the embedded one |

The rule set is compiled as `rbb` loads a rules directory: the manifest
must ask for a rule language this engine reads (`info().language`), the
vocabulary must load (every path `bbsa_map` names is a field of `fields`,
every value fits), and every rule file must compile against that
vocabulary, with every term known to the engine (`check_terms`). Any
failure is an `error` diagnostic, and no engine is made: in a rule file
with `file`, `line` and `col` (`"rules: demo/one-nt.bid:6: unknown term
..."`), in the other texts starting with the field (`"fields: ..."`, with
the `line` of a TOML syntax error). When only some of the texts are given,
the embedded ones fill in and must fit: an error in one of them names it
(`"bbsa_map: the embedded conventions/card/bbsa-map.toml (not given): ..."`).

Cards are read in the supplied vocabulary (stock cards too). A compiled
rule set is identified by a hash of all its texts (`rules_id`); the last
few are kept, so repeating one costs nothing. The engine reads one card
field itself, `general.style` (how it counts points; `bba` is BBA's
valuation); a vocabulary without it gets the default valuation.

### `validate(request)` → JSON

Checks whatever fields the request carries, without bidding. Every field is
optional:

```json
{"hand": "...", "deal": "...", "dealer": "N", "auction": "...", "prefix": "...",
 "vul": "None", "scoring": "MP", "cards": {...}, "engine": 1,
 "rules": {...}, "fields": "...", "bbsa_map": "...", "manifest": "..."}
```

- `auction` and `prefix` are checked for legality from `dealer`, which is
  then required.
- `cards` are loaded (not turned into an engine), with the card diagnostics
  above, in the vocabulary of `engine`, else of the supplied rule set,
  else the embedded one.
- A supplied rule set is compiled as above and also checked as
  `rbb bid check` checks a directory: module names are unique and an enum
  parameter is only compared with its options (errors); a `needs` that
  names no module and a `.bbsa` key that is not one of BBA's are warnings.

Response: `{"ok": bool, "diagnostics": [...]}`, and with a supplied rule
set that compiles, its summary:

```json
{"ok": true,
 "rule_set": {"rules_id": "5c0e...", "name": "demo", "language": 1,
              "rule_files": 1, "modules": 1, "rules": 1, "card_fields": 1},
 "diagnostics": []}
```

### `bid(request)` → JSON

The engine's call for one hand.

Request:

```json
{"engine": 1, "hand": "AK52.KJ7.Q94.K83", "dealer": "N", "vul": "None",
 "scoring": "MP", "auction": "Pass Pass"}
```

The hand belongs to the seat whose turn it is after `auction`. `hand` and
`dealer` are required; an auction that is already over is an error.

Response:

```json
{"ok": true,
 "seat": "S",
 "call": "1NT",
 "explanation": "15-17 HCP, balanced",
 "alert": {"kind": "announce", "text": "15 to 17"},
 "rule": {"module": "notrump-base", "file": "conventions/notrump/one-nt.bid", "line": 19},
 "candidates": [
   {"call": "1NT", "rule": {...}, "explanation": "15-17 HCP, balanced",
    "priority": 0, "descriptiveness": 0.93, "prefer": null, "outcome": "chosen"},
   {"call": "1C", "rule": {...}, "explanation": "...", "priority": 0,
    "descriptiveness": 0.41, "prefer": null, "outcome": "outranked"},
   {"call": "2C", "rule": {...}, "explanation": "...", "priority": 0,
    "descriptiveness": 0.97, "prefer": null, "outcome": "hand fails `shows hcp>=22`"}
 ],
 "auction": [ <step>, <step> ],
 "position": <position>,
 "warnings": [],
 "diagnostics": []}
```

| Field | |
|---|---|
| `seat` | the seat bidding |
| `call` | the call chosen |
| `explanation` | the chosen rule's explanation, filled in for this hand; `"No rule applies"` when no rule matched (the engine then passes, and an `info` diagnostic says so) |
| `alert` | `null`, `{"kind": "alert", "text": string or absent}`, or `{"kind": "announce", "text": string}`; `{name}`s in the text are filled in (`"15 to 17"`) |
| `rule` | `{module, file, line}` of the chosen rule, or `null` |
| `candidates` | every rule considered, best-ranked first: `outcome` is `"chosen"`, `"outranked"`, `"partner's call is forcing"` (a pass not allowed), or why the hand does not qualify (`hand fails ...`) |
| `auction` | the auction so far as the engine read it: one `step` per call (below) |
| `position` | the table as the bidder sees it **before** the call: what every seat has shown and each side's auction state (below) |
| `warnings` | problems met while choosing the call (unknown terms, contradictions), as plain strings; they are also in `diagnostics` as `warning`s |

### `interpret(request)` → JSON

What each call of an auction showed.

Request: `{"engine": 1, "dealer": "N", "vul": "None", "scoring": "MP", "auction": "1NT Pass 2C Pass"}`.

Response:

```json
{"ok": true,
 "steps": [ <step>, ... ],
 "complete": false,
 "next": "N",
 "contract": null,
 "declarer": null,
 "position": <position>,
 "diagnostics": []}
```

`next` is the seat to call (`null` when `complete`). `contract` (`"4S"`,
`"3NX"`, `"Pass"` for a passed-out deal) and `declarer` (a seat, `null` when
passed out) are set once the auction is complete. `position` is the table
after the last call (below).

A **position** (added in API 1; absent from older builds):

```json
{"knowledge": {"N": <knowledge>, "E": <knowledge>, "S": <knowledge>, "W": <knowledge>},
 "sides": {
   "ns": {"trump": "H", "forcing": "round", "forcing_by": "S",
          "ask": {"kind": "keycards", "args": ["H"], "by": "N"}, "answered": null,
          "summary": "trump H, forcing round (set by S), asked keycards by N"},
   "ew": {"trump": null, "forcing": "none", "forcing_by": null,
          "ask": null, "answered": null, "summary": "forcing none"}}}
```

- `knowledge`: what each seat has shown so far, in the step's
  `knowledge` shape (a seat that has not called, or has shown nothing,
  has the full ranges: 0-37 HCP, 0-13 cards a suit, empty `summary`).
- `sides`: each partnership's auction state as the rules see it. `trump`
  is the agreed strain (`C`, `D`, `H`, `S`, `NT`) or `null`; `forcing` is
  `"none"`, `"round"` (the forcer's partner may not pass at the next turn
  unless their right-hand opponent acts) or `"game"` (neither partner may
  pass below game), with `forcing_by` the seat that set it; `ask` a
  question awaiting partner's answer and `answered` one partner has
  answered (`kind` is the rule language's name, e.g. `keycards`; `args`
  its strains); `summary` all of it in one line, as the workbench shows it.

A **step**:

```json
{"seat": "N", "call": "1NT",
 "explanation": "15-17 HCP, balanced",
 "alert": {"kind": "announce", "text": "15 to 17"},
 "rule": {"module": "notrump-base", "file": "conventions/notrump/one-nt.bid", "line": 19},
 "artificial": false,
 "knowledge": {
   "hcp": {"min": 15, "max": 17},
   "lengths": {"S": {"min": 2, "max": 5}, "H": {...}, "D": {...}, "C": {...}},
   "balanced": true,
   "summary": "15-17 HCP, 2-5 S, 2-5 H, 2-5 D, 2-5 C, balanced",
   "shown": ["..."]}}
```

- `explanation`, `alert`, `rule`: the rule that gives the call its meaning
  as the table reads it; all `null` when no rule does (a call outside the
  system).
- `artificial`: the call does not name a place to play (a transfer, an ask).
- `knowledge`: what the caller's hand is known to hold **after** this call,
  from all their calls so far. `balanced` is `true`, `false` or `null`
  (unknown). `summary` is a one-line reading of the narrowed ranges (empty
  when nothing is known); `shown` lists every constraint the seat has shown
  or denied, in rule-language syntax (docs/LANGUAGE.md).

### `bidDeal(request)` → JSON

Bid all four hands.

Request:

```json
{"engine": 1,
 "deal": "N:AK52.KJ7.Q94.K83 QJ3.Q95.KJ3.QJ74 T64.AT832.A2.T62 987.64.T8765.A95",
 "dealer": "N", "vul": "None", "scoring": "MP",
 "prefix": "", "max_calls": 60}
```

`prefix` (optional, auction form) is forced first; the engine bids from
there. `max_calls` (default 60) stops a runaway auction (`complete` false,
with a `warning`).

Response:

```json
{"ok": true,
 "calls": [
   {"seat": "N", "call": "1NT", "forced": false,
    "explanation": "15-17 HCP, balanced", "meaning": "15-17 HCP, balanced",
    "alert": {...}, "rule": {...}, "artificial": false, "knowledge": {...}},
   ...],
 "complete": true,
 "contract": "4H",
 "declarer": "N",
 "diagnostics": []}
```

Each entry is a step plus `forced` (from `prefix`) and `meaning`. For the
engine's own calls `explanation`, `rule` and `alert` are the rule that chose
the call for this hand (as `bid` reports it), and `meaning` is what the call
shows to the table (as `interpret` reports it); they are usually the same
text. For forced calls both are the table's reading.

### `conventions(request)` → JSON

The conventions as data, for a UI: the embedded ones, or those of `engine`
or of a supplied rule set (`rules_id` says which). Request: `{}`, or with `engine`
/ `cards` to learn which modules each side plays.

```json
{"ok": true, "rules_id": "d737c980a4bbdb8c",
 "modules": [
   {"name": "stayman", "title": "Stayman", "file": "conventions/notrump/stayman.bid",
    "card": ["notrump.stayman.play"], "needs": ["notrump-base"],
    "rules": [
      {"after": ["1N (P)", "(1x) 1N (P)"], "context": ["!they.bid | systems_on"],
       "call": "2C", "explanation": "Stayman: asks for a 4-card major",
       "alert": "alert", "shows": "strength>=invite, ...", "when": "!shape 4333 | ...",
       "artificial": true, "line": 12}]}],
 "active": {"ns": ["base", "..."], "ew": ["..."]},
 "diagnostics": []}
```

`active` is present only with `engine` or `cards`. Rule fields are the rule
as written: `after` lists alternative auctions before the call (calls
separated by spaces, the opponents' in parentheses, `P` pass; empty means
"any auction", the rule's own conditions decide); `context` the enclosing
`when` conditions; `call` may be a pattern (`2x`, `jump(x)`, `4{trump}`);
`alert` is `"alert"`, `"alert: <text>"` or `"announce: <text>"`; `shows`
and `when` are in rule-language syntax (docs/LANGUAGE.md); `{name}` in an
explanation is filled in when bidding.

### `reference(request)` → text

The same data as `conventions`, as plain text: what a site serves as
`reference.txt`. Request: `{}`; with `engine` or `cards`, each module is
marked `[on]` or `[off]` for `"side": "ns"` (default) or `"ew"`.

```
# rusty-bidding-bot conventions reference
# rbb 0.1.0 rules d737c980a4bbdb8c
# 34 modules, 1658 rules
#
# Calls: P pass, X double, XX redouble; (..) the opponents' calls.
# x/y any suit, M a major, m a minor. Text in {..} is filled in when bidding.

== stayman: Stayman  [on]
   file: conventions/notrump/stayman.bid
   card: notrump.stayman.play
   needs: notrump-base

   after 1N (P) | (1x) 1N (P) | ...
     when !they.bid | systems_on
     2C     Stayman: asks for a 4-card major  [alert]  [artificial]
            shows strength>=invite, ...
            when  !shape 4333 | ...
```

Errors and warnings in the request are written as `# error: ...` /
`# warning: ...` lines at the top; the rest is still produced. The same text
comes natively from `rbb bid reference` (for generating `reference.txt` at
build time without a browser).

### `coverage(request)` → JSON

How much of each side's card the rules read (the same classification as
`rbb card coverage`): what a UI shows as "conventions and treatments this
engine does not play". Request: `{"cards": {...}}`, with `engine` or a
supplied rule set to measure the cards against those rules (and read them
in their vocabulary).

```json
{"ok": true,
 "ns": {"name": "Precision", "system": "precision",
        "read": ["notrump.stayman.play", "..."],
        "ignored": ["other_conventions.precision.one_c.play", "..."],
        "other": ["carding.upside_down_count"],
        "unmapped": ["Crosswood 1430", "..."],
        "score": 0.62},
 "ew": {...},
 "diagnostics": []}
```

Of the settings the card switches on (not at their default): `read`, a
module names the field (or it feeds one that does); `ignored`, the field
exists and no rule reads it; `unmapped`, settings switched on that have no
card field at all (`.bbsa` keys, or paths of a card JSON); `other`,
carding, leads, notes and the fields marked `note` (write-in lines: they
cannot change a call). `score` is `read / (read + ignored)`.

### `exportCard(request)` → JSON

A card in both formats: the mapping from a BBA `.bbsa` card to
Bridge-Classroom card JSON and back. Request: `{"card": <card spec>,
"set"?: ["path=value"], "engine"?: 1, "exported_at"?: "<ISO 8601>"}`: the
card is read and written in the vocabulary of `engine`, else of a supplied
rule set, else the embedded one.

```json
{"ok": true, "name": "21GF-DEFAULT",
 "json": {"schema_version": "1.0", "format": "bridge_classroom", "...": "..."},
 "bridge_classroom": {"schema": "bridge-classroom/card_data@v1", "name": "21GF-DEFAULT",
                      "description": null, "exportedAt": "...", "card_data": {"...": "..."}},
 "bbsa": "<.bbsa text, CRLF>", "unmapped": ["..."], "diagnostics": []}
```

`json` is Bridge-Classroom `card_data` (it loads back as `{"json": ...}`);
`bridge_classroom` is the same card in the editor's export format (its
"Export content", Bridge Classroom JSON), with `exportedAt` only when the
request gives `exported_at`; it loads back too. Unmapped `.bbsa` keys
travel in `bba_passthrough`, and keys such as `_bbo_raw` stay where they
were, so both formats round-trip.

### `ddTable(request)` → JSON

The double-dummy table of a deal and par, and, given a finished auction,
the contract's double-dummy result. Solved with bridge-solver (a few hundred
ms in the browser). Request: `{"deal": ..., "vul"?: ..., "dealer"?: ...,
"auction"?: ...}` (`dealer` is required with `auction`).

```json
{"ok": true,
 "tricks": {"N": {"C": 8, "D": 6, "H": 10, "S": 10, "NT": 10}, "E": {...}, "S": {...}, "W": {...}},
 "par": {"score_ns": 430, "contracts": ["N 3N+1"]},
 "result": {"contract": "3N", "declarer": "N", "tricks": 10, "score_ns": 430},
 "diagnostics": []}
```

`tricks` are for each declarer and strain. `par.contracts` lists every
contract tied at par (empty when par is a pass-out). `result` is `null`
without a finished auction; for a passed-out deal its `contract` is
`"Pass"` and `score_ns` 0.

## Mapping to the bridge-craftwork tool contract

The site's tools expose `window.<tool> = { run(input, params), validate(input,
params), getInput(), setInput(), getOutput() }` with diagnostics
`{severity, message, line?, col?, hint?}` and a `/<tool>/reference.txt`
(bridge-craftwork-site issue #3). This API is shaped for that:

| Contract | Here |
|---|---|
| `run(input, params)` | build a request from `input` (hand or deal, auction) and `params` (cards, vul, scoring), call `bid` / `interpret` / `bidDeal`, parse the JSON |
| `validate(input, params)` | `validate` with the same request: returns `{ok, diagnostics}` without bidding |
| diagnostics | every response's `diagnostics`, already in the contract's shape; pass them through |
| `reference.txt` | `reference("{}")` in the browser, or `rbb bid reference -o reference.txt` at build time (same text, from the same compiled rules); `conventions` gives the same data as JSON for the UI |
| deep links | requests are plain JSON: a fragment can carry the request fields (`#hand=...&auction=...&card=...`) and replay them |

## Stability

- The function names, request fields and response fields above are the
  contract; `api` is bumped when one changes incompatibly.
- Explanations, alerts, `shown` constraints, rule files and line numbers,
  candidate lists and `rules_id` change whenever the rules do: display them,
  do not parse them.
- New fields may be added to responses; ignore what you do not know.
