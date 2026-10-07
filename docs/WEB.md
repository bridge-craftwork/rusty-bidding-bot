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
| `status.html`, `status/` | the status page (below): `status.js` draws, `legend.js` the status vocabulary, `status.css`, and `status.json`, its data (committed) |
| `_headers`, `404.html` | Pages headers; the real 404 |
| `scripts/` | `emit-reference.mjs`, `check-reference.mjs` (drift), `test.mjs` |

## The contract (bridge-craftwork-site #3)

| Item | Here |
|---|---|
| `/<tool>/reference.txt`, text/plain, no JS | written at build time by `lib/reference.js` from `INPUT`/`PARAMS` (the page's controls come from the same tables), the shipped engine's `info()` and its `reference()` (the conventions, module by module); `_headers` sets `text/plain; charset=utf-8` |
| CI drift check | `.github/workflows/ci.yml`, job *Web site*: `check-reference.mjs` fails if the conventions section differs from native `rbb bid reference` (the rules compiled into the WASM are the tree's), if the file is not what the shipped engine writes, if it misses an input field, parameter, API method, card or module, or if the page has a `data-param` control that is not a `PARAMS` field |
| `window.rustyBiddingBot = {run, validate, getInput, setInput, getOutput}` | `app.js`, over `Session`; shapes in `reference.txt` |
| structured diagnostics | `{severity, message, line?, col?, hint?}`, the WASM API's own, plus the page's (params, script); listed under the table and returned by `validate` and in every output |
| fragment deep links | `#scenario=Stayman&board=12&stop=3&rotate=1&view=S&xray=1...`; the page rewrites its fragment as you go (`replaceState`), `hashchange` reloads; never a query string |
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
- **X-ray** (button in the auction head, key `x`, parameter `xray`, deep
  link `xray=1`): the workbench's view of a call, directly under the
  auction. For the selected call (the last one when none is selected), or
  for every call with *Every call* (`xray=all`): the caller's hand, every
  rule the engine weighed for it best-ranked first (call, priority,
  descriptiveness, outcome: chosen / outranked / why the hand fails,
  meaning, rule file:line linked to the source), for a call the engine did
  not make what it would have bid and whether any rule offers the call,
  warnings, then what each seat had shown before the call (HCP and suit
  ranges, balance, the constraints behind them) and each side's state
  (trump, forcing none/round/game and who set it, asks). At the end of a
  scenario it adds how the engine reads BBA's auction: each BBA call with
  BBA's note, the engine's reading, rule and what it shows, and the call
  the engine would make there. When *Show* hides a seat, that seat's hand
  and candidates stay hidden until the auction ends. The same data is
  `output.xray` (reference.txt, "output"): `{calls, now, bba}`, built on
  the WASM `bid` / `interpret` responses' `position` and `warnings`
  (docs/WASM.md).
- **Rotate** the deal 0-3 seats (dealer and vulnerability turn; the
  scenario's cards follow their hands). **Show** all hands or one seat's.
- **Alerts** (`!`) and announcements (`A`) marked; optionally every call's
  meaning; calls no rule explains flagged (`?`): a gap in the engine's system.
- **End of auction**: all hands, the double-dummy table, par and the
  contract's double-dummy result (the WASM `ddTable`, bridge-solver), and
  for a scenario BBA's auction and contract.
- **Cards**: the PBS cards built in (the scenario's by default), the
  Bridge-Classroom 2/1 card, or an uploaded `.bbsa` / Bridge-Classroom JSON
  (bare `card_data` or the editor's export, named by its `name`);
  any of them downloads as Bridge-Classroom JSON (the editor's export
  format) or `.bbsa` (the WASM `exportCard`: the mapping between the two).
- **Missing conventions**: for each card, the settings it switches on that no
  rule reads (WASM `coverage`, the same buckets as `rbb card coverage`), and
  the settings with no card field (`.bbsa` keys, card JSON paths).

## Status page

`status.html` (linked from the page's header, the 404 page and
`reference.txt`; Rick, 2026-10-06): where the system stands, as a compact
grid of tiles. Sections: Basic bridge, Judgment, Catch-alls, Conventions:
constructive, Conventions: competitive, Precision. Within the convention
sections tiles are grouped by convention-card's level bands (basic 1–3,
intermediate 4–6, advanced 7–8, expert 9–10) and ordered by level, then
name; a checkbox turns the grouping off (alphabetical). A tile is a
convention with its variants folded in (Reverse and Two-way Drury under
Drury; `[groups]` in `probes/status-items.toml`), its colour and glyph the
convention's status, and one dot per treatment: each card field carrying
the convention's skill, each option of an enum field. Hover gives the
summary and the reason for the colour; a click (or Enter) opens a dialog
with the spec summary, the treatments and their card settings, the
scenarios measured against BBA, the self A/B runs, the modules with rule
and test counts and links to the source and notes, and citations.
Escape closes it. Search and a status filter narrow the grid; state is
in the fragment (`#q=drury&status=gap&tile=bidding_conventions/drury&flat=1`).

Statuses (the page's legend, `status/legend.js`): **not implemented**
(no module, or no rule reads the setting), **implemented, tests only**,
**partial** (curated in `status-items.toml`), **convention good / fair /
poor** (▲ ◆ ▼; Rick, 2026-10-07: a convention BBA plays, bid by BBA and
by us on the same deals with North-South playing it and not; what it
gains us minus what it gains BBA, by errors, over the boards where
either auction changed: good at ≥ −0.1 IMPs per changed board, poor at
≤ −0.5 with both board halves agreeing, fewer than 10 changed boards
unjudged; `probes/tools/conv_ab.py`), **background good / fair / poor**
(✓ ≈ ✗; the tile's scenarios pooled against BBA, par as the yardstick:
good at errors ≥ −0.25 IMPs/board and ≥ 65% of NS calls agreeing, poor
below −1.0 or 55%), **A/B gains / neutral / loses** (`self_ab.py`'s
verdict; "leans" counts with gains and loses). A tile takes, in order:
gap, partial, its convention score, a self A/B of its own convention
(BBA does not play it), its background, tests only. A tile coloured by
its convention score also carries the background on the same deals as a
small square beside the name: a good convention with a poor background
says the judgment around it needs work, a poor convention with a good
background that the convention itself does. A tile with no convention
score (BBA cannot switch it: no .bbsa key; our rules have no fallback
without it; the scenario card does not play it) takes its background as
its colour, "background (not isolated)". The status filter and the
legend pick by either signal: a background status matches the tiles
coloured by it and those carrying it as the square. The details dialog
shows the convention score beside the background, then each `conv_ab`
run (switched off, BBA's keys, changed boards, both gains, halves).

The convention score's thresholds are per changed board, so they are
not the background's: −0.1 lets the convention do a tenth of an IMP
less for us than for BBA on each board it touches before it stops being
good (about the noise of a few hundred boards), and −0.5, half an IMP a
touched board with both halves agreeing, is a loss nobody would accept
from a convention. A treatment chip is measured only on scenarios whose NS card
switches it on. Judgment tiles compare our boards in a par class with
BBA's (good at most BBA's count, fair up to 25% more); catch-alls count
per 1,000 boards (no-rule positions, calls read as a higher rule).

**Regenerating** (not in CI: the measurements take minutes). After a
measurement run, from the repository root:

```sh
./dev-build.sh build --release -p rbb-cli
./cpu-gate.sh target/release/rbb compare --json /tmp/all.json       # every scenario, 500 boards
probes/tools/self_ab.py --batch probes/self-ab.toml                 # .rbb-cache/self-ab
probes/tools/conv_ab.py                                             # .rbb-cache/conv-ab (bba-cli twice per scenario)
probes/tools/status_data.py --compare /tmp/all.json                 # web/status/status.json
```

and commit `web/status/status.json`. `status_data.py` reads the
convention-card spec at the revision `Cargo.lock` pins (cargo's
checkout, else `git show` in `../convention-card`, else GitHub), the
modules through `rbb bid skills --json`, each corpus card through
`rbb card import-bbsa`, and warns about curated module or scenario names
that match nothing. Without `--compare` the BBA columns are empty and
measured tiles fall back to tests only. `conv_ab.py` (driven by
`probes/conv-ab.toml`: per convention its tile, scenarios and `off`
changes) needs the installed bba-cli; it writes the cards, both BBA
runs and both replays under `.rbb-cache/conv-ab/<name>/` and the scores
in `.rbb-cache/conv-ab/results.json`, which `status_data.py --conv-ab`
reads (the default path). `node web/scripts/test.mjs`
checks that the shipped `status.json` parses and every tile has a known
status.

## Later

- Deploy: `.github/workflows/pages.yml` builds, checks and deploys on a push
  to main that touches the site, engine or rules (and on demand). It needs
  the Pages project (`npx wrangler pages project create rusty-bidding-bot`)
  and two org secrets: `CLOUDFLARE_PAGES_DEPLOY_TOKEN` (a token with
  Account -> Cloudflare Pages -> Edit; named for its use, mapped to
  wrangler's `CLOUDFLARE_API_TOKEN`) and `CLOUDFLARE_ACCOUNT_ID`; until
  they exist it builds and skips the deploy with a notice. Then the router
  entry in bridge-craftwork-site and an `llms.txt` line.
- Run the engine and dealer3 in a Web Worker (a hard dealer script or a
  slow double-dummy solve blocks the page briefly today).
- The scenario's chat text and a per-scenario "what this scenario teaches"
  panel; a comparison over many boards (the workbench's view) on the page.
- Treatments (`general.style=bba` and friends) as controls, not only `set`.
- Custom rules. The WASM already takes a rule set as text (docs/WASM.md,
  "Rule sets supplied at run time"), and the page already reads cards,
  coverage and card exports through its engine, so they follow whatever
  rules that engine plays. What is missing is on the page: a way to load a
  rules directory (a folder picker or a `.zip` of `*.bid`,
  `conventions.toml` and `card/*.toml`, kept per viewer), a `rules`
  parameter for `Session.prepareEngine` passed to `createEngine` with the
  cards, `validate` on load to list compile errors by file and line, the
  stock and uploaded cards re-read in that vocabulary, `reference` from
  that engine instead of the build-time `reference.txt`, and a deep link
  that says which rule set bid (its `rules_id`; the texts are too big for a
  fragment). The `.bid` editor itself would be a separate tool.
