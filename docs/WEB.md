# The web site (issue #2)

`web/` is the engine's showcase page, modelled on dealer3's: pick a
Practice-Bidding-Scenarios (PBS) scenario, a random deal, a dealer3 script
or a deal of your own, choose the two convention cards, and watch the engine
bid seat by seat, every call explained. It is a bridge-craftwork tool, mounted
at **`bridge-craftwork.com/rusty-bidding-bot/`**, and follows the site's
LLM-friendly tool contract (bridge-craftwork-site issue #3).

**Tool path: `rusty-bidding-bot`.** Every tool on the site is mounted at its
repository's name (`pbn-to-pdf`, `dealer3`, `pdf-handouts`,
`bridge-solver`), with a Cloudflare Pages project of the same name, so the
router's table stays mechanical: `'/rusty-bidding-bot':
'https://rusty-bidding-bot.pages.dev'`. The JavaScript namespace follows the
contract's camelCase rule: `window.rustyBiddingBot`.

## Build and run

```sh
web/build.sh                 # web/dist: page, engine, dealer3, reference.txt
python3 -m http.server -d web/dist 8000   # http://localhost:8000/
node web/scripts/test.mjs web/dist        # the tool contract, in Node
PBS_DIR=../Practice-Bidding-Scenarios node web/scripts/test.mjs web/dist  # + scenarios, offline
```

`build.sh` runs `crates/wasm/build.sh` into `dist/pkg`, builds dealer3's
single-threaded WASM into `dist/dealer3` (from `DEALER3_DIR`, else a clone
of bridge-craftwork/Dealer3 at the pinned `DEALER3_REF` in `web/.cache`;
`--no-dealer3` skips it and the script input then says so), copies the
static files, and writes `dist/reference.txt`. No bundler and no npm
dependencies: plain ES modules with relative URLs, so the same build works
at `/` (pages.dev) and under `/rusty-bidding-bot/`. `wrangler.jsonc` points
Pages at `web/dist`; `npx wrangler pages dev` serves it locally with the real
`_headers` and 404 behaviour.

## Files

| File | |
|---|---|
| `index.html`, `app.js`, `app.css` | the page; `app.js` only draws and dispatches |
| `lib/tool.js` | the tool: `Session` (deal, cards, calls, engine), and `INPUT` / `PARAMS`, the vocabulary the page's controls and `reference.txt` are built from |
| `lib/deal.js` | deals, rotation, boards, PBN reading (no DOM, no WASM) |
| `lib/pbs.js` | PBS manifest, corpus (`bba/<name>.pbn`, with BBA's auctions), scripts, cards: read from raw.githubusercontent.com (CORS `*`), as dealer3 and Bridge-Classroom do |
| `lib/fragment.js` | deep links |
| `lib/reference.js` | `reference.txt` |
| `styles.css` | the bridge-craftwork design tokens, copied from Bridge-Classroom (not fetched) |
| `cards/21_intermediate_card.json` | Bridge-Classroom's 2/1 Intermediate card (seed data), offered as `BC-21-Intermediate` |
| `_headers`, `404.html` | Pages headers; the real 404 |
| `scripts/` | `emit-reference.mjs`, `check-reference.mjs` (drift), `test.mjs` |

## The contract (bridge-craftwork-site #3)

| Item | Here |
|---|---|
| `/<tool>/reference.txt`, text/plain, no JS | written at build time by `lib/reference.js` from `INPUT`/`PARAMS` (the page's controls come from the same tables), the shipped engine's `info()` and its `reference()` (the conventions, module by module); `_headers` sets `text/plain; charset=utf-8` |
| CI drift check | `.github/workflows/ci.yml`, job *Web site*: `check-reference.mjs` fails if the conventions section differs from native `rbb bid reference` (the rules compiled into the WASM are the tree's), if the file is not what the shipped engine writes, if it misses an input field, parameter, API method, card or module, or if the page has a `data-param` control that is not a `PARAMS` field |
| `window.rustyBiddingBot = {run, validate, getInput, setInput, getOutput}` | `app.js`, over `Session`; shapes in `reference.txt` |
| structured diagnostics | `{severity, message, line?, col?, hint?}`, the WASM API's own, plus the page's (params, script); listed under the table and returned by `validate` and in every output |
| fragment deep links | `#scenario=Stayman&board=12&stop=3&rotate=1&view=S...`; the page rewrites its fragment as you go (`replaceState`), `hashchange` reloads; never a query string |
| real 404s | `404.html` at the root: Pages answers unknown paths with it and status 404 (checked with `wrangler pages dev`) |
| relative asset URLs | all of them; tested served under `/rusty-bidding-bot/` |
| `_headers` | nosniff, no-referrer, revalidation for the unhashed assets; no COOP/COEP (nothing threaded) |
| own Pages project | `wrangler.jsonc` (`name: rusty-bidding-bot`, output `web/dist`) |

## Features

- **Deals**: PBS scenario boards from the corpus (500 each, with BBA's
  auction shown for comparison at the end), fresh boards from a scenario's
  own dealer script, any dealer3 script (with `# convention-card-ns:` lines
  choosing cards), random deals (seeded, so linkable), or a PBN deal with
  calls already made.
- **Bidding**: *Bid next* (the engine's call for the seat to bid, with why,
  the rule and its file and line, and the candidates it weighed), *Bid to
  end*, *Undo*, *Restart*, and a bidding box for a call of your own, from
  which the engine carries on.
- **Rotate** the deal 0-3 seats (dealer and vulnerability turn; the
  scenario's cards follow their hands). **Show** all hands or one seat's.
- **Alerts** (`!`) and announcements (`A`) marked; optionally every call's
  meaning; calls no rule explains flagged (`?`): a gap in the engine's system.
- **End of auction**: all hands, the double-dummy table, par and the
  contract's double-dummy result (the WASM `ddTable`, bridge-solver), and
  for a scenario BBA's auction and contract.
- **Cards**: the PBS cards built in (the scenario's by default), the
  Bridge-Classroom 2/1 card, or an uploaded `.bbsa` / Bridge-Classroom JSON;
  any of them downloads as Bridge-Classroom JSON or `.bbsa` (the WASM
  `exportCard`: the mapping between the two).
- **Missing conventions**: for each card, the settings it switches on that no
  rule reads (WASM `coverage`, the same buckets as `rbb card coverage`), and
  the `.bbsa` keys with no card field.

## Later

- Deploy: a Pages project and a deploy workflow (dealer3's `pages.yml` is the
  model), then the router entry in bridge-craftwork-site and an `llms.txt`
  line.
- Run the engine and dealer3 in a Web Worker (a hard dealer script or a
  slow double-dummy solve blocks the page briefly today).
- The scenario's chat text and a per-scenario "what this scenario teaches"
  panel; a comparison over many boards (the workbench's view) on the page.
- Treatments (`general.style=bba` and friends) as controls, not only `set`.
