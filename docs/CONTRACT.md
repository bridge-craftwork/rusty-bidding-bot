# The contract between the engine and the conventions

The engine (`rbb-engine`, `bidspec`, the `rbb` tools, the workbench, the
WASM build) and the conventions (`conventions/`: the `.bid` rules with
their `.test` and `.notes.md` files, and the probe specs
in `probes/`) are meant to live in two repositories, so that others can
write convention scripts against a stable engine. This document is what
each side may rely on. The first half is what the engine promises; the
second is what a good convention file does.

## Part 1: what the engine provides

### The language, and its version

The grammar and the model are in [LANGUAGE.md](LANGUAGE.md). This engine
reads **rule language 1**: `rbb --version` prints it
(`rbb 0.4.0 (rule language 1)`), and so does the last line of
`rbb bid check`. In the code it is `rbb_engine::LANGUAGE_VERSION`.

A rules directory says which language it is written in with a manifest at
its root, `conventions.toml`:

```toml
name = "rusty-bidding-bot"
description = "The base system and conventions, switched on by the convention card"
language = 1          # required: the rule language version
engine = "0.4.0"      # optional: the engine it was developed against (information only)
```

- A manifest that asks for a language this engine does not read refuses
  the load, naming the file, the version asked for and the versions read
  ("a newer engine is needed"). A manifest that does not parse, or lacks
  `name` or `language`, refuses the load too.
- **A directory without a manifest loads as the current language.**
  `rbb bid check` warns about it. This keeps a subdirectory
  (`rbb bid check conventions/notrump`) and quick experiments working;
  a rule set meant for others should carry one.
- Unknown manifest keys are reported by `rbb bid check` as warnings and
  otherwise ignored.

`bidspec::manifest` reads the file (the convention layer may parse it);
`rbb_engine::read_manifest` / `check_manifest` enforce the version, and
`rbb_engine::load_modules` calls them. The rules built into the binary
(`rbb-assets`) come from the same tree and build, and are checked by the
tests that load `conventions/` from disk.

### The term vocabulary

Every name a condition may use is listed in the [generated
reference](#term-reference) at the end of this document, with its meaning.
`rbb bid terms` prints the same list; it comes from the tables in
`crates/engine/src/eval.rs`, which are also what the load-time check
accepts.

### Loading

A rules directory holds:

| Path | |
|---|---|
| `conventions.toml` | the manifest: name, rule language version |
| `card/fields.toml`, `card/bbsa-map.toml` | optional: the rules' own card vocabulary. Without them the rules use the standard one, convention-card's `spec/` at the tag `Cargo.toml` pins |
| `**/*.bid` | the rules (with their `.test` and `.notes.md` files) |

- **Each rules directory has a card vocabulary**: its own `card/fields.toml`
  and `card/bbsa-map.toml` when it brings them, otherwise the standard one
  (convention-card's `spec/`, carried by the `bridge-card` crate; the
  teaching skills are its `spec/conventions/`). The engine uses the one for
  the directory it is given (`--rules DIR`; the copy embedded with the
  rules in the release `rbb` and the WASM), checks the rules against it,
  and reads every card (`.bbsa`, card JSON, `--set path=value`) in it, so
  a field's default, aliases and `.bbsa` mapping are the ones the rules
  were written for. A rule set with its own fields needs no engine change.
  Both files must load: a missing file, a path the map names that is not a
  field, or a value that does not fit its field refuses the rule set.
- Every `.bid` file under the rules directory is compiled, **sorted by
  path**. That order is the last tie-breaker between candidates, so moving
  a file can change a call.
- A module is active when all its `card` conditions hold on the card and
  every module it `needs` is active (repeated until nothing changes).
  `param` reads a card field, with the given default when the card leaves
  it unset (no default: the value is nothing, and `style is x` is false).
- `card` and `param` paths must exist in the card vocabulary
  (the card fields); an alias is reported with its current name, and an
  enum option the field does not have is an error.

### Card fields the engine reads by name

Everything else on a card reaches the engine only through a module's
`card` and `param` lines. These few the Rust code reads by path, so a
vocabulary should keep them (a vocabulary without them still loads):

| Field | Read by | What it does | Without it |
|---|---|---|---|
| `general.style` | engine (`Valuation::for_card`) | `bba` counts total `points` as BBA does, without the point per card beyond four (LANGUAGE.md, "Points"). Modules also read it through `param` for treatments | the default count, length included; `--set general.style=...` is refused |
| `general.system_category` | `rbb card coverage` | the system shown beside each card | shown as `two_over_one` |
| `general.system_category` | `rbb probe` / `rbb grid` `bare:2/1` cards | the only field a `bare:` card sets (`two_over_one`, `sayc`, `polish_club`, `precision`, `acol`) | a `bare:` card is refused (unknown field); other cards are unaffected |
| sections `carding`, `leads`, `notes`, `metadata` | `rbb card coverage` | settings under these prefixes count as play, not bidding (neither honoured nor ignored) | nothing: they are only prefixes |
| attribute `note = true` | `rbb card coverage` | a free-text field inside a bidding section (a card's write-in line, `two_level.two_clubs.notes`) counts with carding, leads and notes | nothing: the field counts as bidding |

### Failures refuse to load

`rbb bid check`, `rbb bid test`, `rbb call`, `rbb compare`, the workbench
and `cargo test` all load through the same checks, and refuse the whole
rule set, with `file:line` diagnostics, when:

- a file does not parse (LANGUAGE.md §11);
- the card vocabulary (the rules' own `card/` files, when they bring them) is
  invalid, or a card path or enum option is unknown;
- a condition names a term, attribute or function the engine does not
  know (`check_terms`): an unknown name would otherwise make its
  condition false every time and silently switch the rule off;
- a `sets` names a state the engine does not keep, or gives it a value
  of the wrong form (`forcing=gam`, `trump=Q`, `ladder=cue`,
  `ask=invite(hcp)`): the forms are in the [reference](#state-set-by-sets);
  otherwise the state would silently not be set;
- a context's `when` depends on the chooser's hand (`hcp`, a suit length,
  `stop(x)`, `shape`, `we.hcp`...): contexts are tried before the hand is
  known, so such a context never holds;
- the manifest asks for a language this engine does not read.

Two module names that clash are an error in `rbb bid check`; a `needs`
naming no module is a warning.

### How a call is chosen

LANGUAGE.md §7, §8 and §12 are the detail; the promises are:

1. **Candidates**: every active rule whose `after` pattern and `when`
   context hold now, with its call made concrete (variables, `{trump}`).
   Illegal calls are dropped.
2. **Filter**: the hand must satisfy the rule's `shows` and `when`.
3. **Rank** the survivors by `priority` (default 0), then
   **descriptiveness** (the share of hands consistent with what I have
   shown that the `shows` rules out, measured on a fixed sample of 20,000
   hands: the same for every hand), then `prefer`, then file order.
4. **No rule**: when no candidate survives, the engine passes and says
   so in the trace. `compare` counts these in live auctions.
5. **Forcing**: after `sets forcing=round` partner may not pass if RHO
   passes (an intervening call ends the force); after `sets forcing=game`
   neither partner may pass below game. A rule's pass that the force
   forbids is ruled out; only when no rule applies at all does the engine
   pass anyway (`compare` reports it as a forcing auction passed).

**Reading a call** (partner's or an opponent's, with the caller's card)
takes the highest-priority rules for that call whose contexts hold; the
caller's hand becomes the union of their `shows`. **Negative inference**
is automatic: the call also denies every candidate that outranks it on
priority and descriptiveness, except where the outranking rule has a term
that cannot be judged for another hand. `denies` adds one by hand.

### State: `sets` and questions

`sets` changes the calling side's state when the call is made (chosen or
read): `forcing=round|game|none`, `trump=<strain>`, `ladder=control|stopper`
(records the suits a ladder call skipped and named), and
`ask=<kind>(<args>)`. An
`ask` is a question to partner: at partner's next turn `asked <kind>(...)`
holds for partner, and binds its arguments; once partner has called,
`answered <kind>(...)` holds for the asker, whatever partner bid, until
the asker calls again. The kind is a free name (`keycards`, `transfer`,
`majors`), agreed only between the rules that set and read it. A round
force never weakens a game force.

These four are the only states, and each value must have the form listed
in the [reference](#state-set-by-sets) (`SET_KEYS` beside `apply_sets` in
`crates/engine/src/engine.rs`); anything else refuses the load
(`check_sets`, run with `check_terms`).

### Determinism

The same rules, card, hand, auction, dealer, vulnerability and scoring
always give the same call, trace and explanations, on every platform and in
the WASM build. There is no randomness at bidding time: the
descriptiveness sample is fixed (seeded), and ties end in file order.

### Tools for a conventions author

| Command | What it does |
|---|---|
| `rbb bid check [DIR\|FILE...] [--rules DIR]` | parse and check every rule file against the card vocabulary of `--rules`; reports the manifest and the language read |
| `rbb bid test [PATHS] [--rules DIR] [-v]` | run the `.test` cases next to the modules |
| `rbb bid terms` | print the term vocabulary (this document's generated section) |
| `rbb bid skills [--rules DIR] [--doc F]` | the teaching-skill map: each skill, the card fields and the modules that name it, and the gaps (docs/SKILLS.md) |
| `rbb bid reference [--rules DIR]` | every module and what each call means after each auction |
| `rbb bid compile FILE [--rules DIR]` | a file's compiled JSON IR |
| `rbb call S.H.D.C -a "1NT Pass" -d S -c CARD [--rules DIR] [--json] [--knowledge]` | the engine's call with every candidate and why it lost |
| `rbb explain-auction -a "1NT Pass 2C" -d N -c CARD [--deal PBN] [--set path=value] [--json]` | the auction reviewed call by call: what is known about every seat after each call, the flags, each side's state, each hand's own view |
| `rbb compare [SCENARIO...] --rules DIR [--json F]` | compare with BBA's auctions in Practice-Bidding-Scenarios: agreement, distance from par, "no rule" in live auctions |
| `rbb-workbench --rules DIR` | the same comparison as a GUI; re-runs when a `.bid`, `.test` or `card/*.toml` file is saved |
| `rbb card coverage CARD... [--rules DIR]` | which card settings the rules read, ignore, or have no field for |
| `rbb grid probes/NAME.toml`, `rbb probe` | ask BBA how it bids hands you make |

The commands and their options are in CLAUDE.md.

### Stability

Within one language version, the engine keeps these, and a change to any
of them needs a new language version (and a manifest that asks for it):

- the grammar: every file that parses keeps parsing to the same meaning;
- every term in the vocabulary, with its meaning; the state kinds
  (`forcing`, `trump`, `ladder`, `ask`) and the matching of `asked` and
  `answered`;
- the ranking order (priority, descriptiveness, `prefer`, file order),
  negative inference, the forcing rules and pass when no rule applies;
- module loading: file order by path, `card` / `needs` activation,
  `param` defaults;
- the load-time checks may be made stricter only for code that could never
  have worked (a condition that is always false, say); anything else
  stricter is a new version.

Additions that no existing file can notice may come without a new version:
a new term, a new function, a new tool or option. A rule set that uses one
does not load on an older engine (the unknown-term check refuses it), so
bump `engine` in the manifest when you start relying on one.

The `skill` header line (LANGUAGE.md §3, engine 0.2.0) is such an
addition. No existing file can notice it: it only records which teaching
skills a module implements, and changes nothing about loading or bidding.
It is still grammar an older engine does not read (its parser refuses a
header line other than `card`, `needs` and `param`), so it came without a
new language version, and this repository's manifest names `engine =
"0.2.0"`. A skill path is not checked against Bridge-Classroom when the
rules load; `rbb bid check` warns about one that is not a standard
convention or skill (convention-card's `spec/conventions/`). The same goes for the `skill` attribute of a card field.

Engine 0.3.0 adds, the same way: named conditions (`define`, LANGUAGE.md
§3), conditions counted as 1 or 0 in arithmetic, and the terms of the
judgment layer's Phase 0 (`we.fit(x)`, `we.points`, the opponents'
totals and last bid, `favourable` / `unfavourable`, `captain`,
`safe_level(x)`, `quick_tricks`, `bare(x)`, `bare_suits`). `we.hcp` and
`we.tp(x)` are now written out as `hcp + partner.hcp` and
`tp(x) + partner.tp(x)` before a rule is used, which is what they meant
while choosing; how an unchosen call's `we.tp` resolves could differ, and
on the corpus nothing changed (170,633 boards, 2026-09-30). Engine
0.4.0 adds the `force game` declaration (LANGUAGE.md §3) and the `bids`
term. This repository's manifest names `engine = "0.4.0"`.

What the
engine *knows* may also get sharper within a version (the deck limits,
how a range narrows): rules read it through the same terms, but a call can
change. Such changes are measured on the corpus like a rule change.

## Part 2: good behaviour for convention files

These are the habits the rule set in this repository follows
(CLAUDE.md is the working detail).

- **Every module has a `.test` file** next to it, with a case for each
  rule a hand can reach: `seat hand | auction | expect | why`, under
  `card` / `dealer` / `vul` / `scoring` headers (DESIGN.md, "Rule-level
  tests"). Call expectations go there, not in Rust tests. `rbb bid test`
  and `cargo test` must pass.
- **Every module has a `.notes.md`**: the agreed guidance (who ruled, and
  when), the probe and corpus evidence behind each decision, the accepted
  differences from BBA and why, gaps and open questions. Update it when a
  decision is made or a probe settles something, and cite the probe specs
  (`probes/NAME.toml`) it relies on.
- **Every `.notes.md` has a "Sources" section** saying where the module's
  rules come from (Rick, 2026-09-28: "We should document in our config
  files where we are sourcing the bidding rules"). Name each kind that
  applies:
  - books, articles and URLs, and published convention descriptions;
  - Rick's rulings, with their dates (and the ticket, if one prompted it);
  - BBA probes, by their spec (`probes/NAME.toml`, or the `rbb probe`
    command line when there is no spec), and `bba-cli --all-meanings`
    readings;
  - corpus measurements that set a threshold or decided a choice.

  A rule adopted from a source says where it **differs** from that source
  (a book's range, a convention description, BBA's treatment, Rick's
  wording), and why. Where the source is simply common practice, say
  "standard practice, not yet cited" rather than inventing a citation;
  add the reference when one is found.
- **No "no rule" in live auctions.** When the engine has no rule after its
  side has entered the auction it passes, which is usually a missing
  continuation. `rbb compare` reports these ("no rule in a live auction");
  keep the count at zero for the auctions a module opens.
- **Measure every change on the corpus** with `rbb compare --json` before
  and after, and judge it with both yardsticks: distance from double-dummy
  par (the compare report's "vs BBA") and, for competitive calls,
  `probes/tools/sideimps.py BASE.json VARIANT.json` (IMPs to the side that
  made the first differing call). Adopt it where they agree; where they do
  not, write the trade in the notes and ask. **Par decides, BBA teaches**
  (DESIGN.md): where agreeing with BBA and par disagree, par wins. Restrict
  corpus tables to one card when cards differ.
- **Find what a decision turns on by making hands** (`rbb grid` specs in
  `probes/`), not by searching the corpus.
- **Name the teaching skills.** A module lists the skills it implements
  with `skill` lines, and a field the skill its convention is taught under
  with its `skill` attribute (Bridge-Classroom's SkillPath strings; the
  known ones are convention-card's `spec/conventions/`). A convention Bridge-Classroom has no
  skill for gets a proposed path there. Regenerate docs/SKILLS.md with
  `rbb bid skills --doc docs/SKILLS.md`.
- **Card fields go in the vocabulary.** A module reads only fields of
  convention-card's `spec/fields.toml`; a new treatment is a new field or enum option there (and a
  `.bbsa` mapping in its `spec/formats/bbsa-map.toml` when BBA has one), never a guessed
  meaning. An unmapped `.bbsa` key stays in passthrough until its meaning
  is known. Treatments are enum fields tested with `is`
  (LANGUAGE.md §3); the BBA treatment is the `bba` option.
- **Clean room.** BBA is a black box: its auctions, `bba-cli` probes and
  `.bbsa` files. Never read or model rules on decompiled BBA/EPBot code
  (CONTRIBUTING.md).
- **Style.**
  - One module per file; the file is named after the module
    (`jacoby-transfers.bid` holds `module jacoby-transfers`), in the
    directory of its family (`base/`, `notrump/`, `majors/`, `competitive/`,
    `slam/`). Module and param names are lower-case with hyphens (modules)
    or underscores (params).
  - Every rule has an explanation a player would say at the table
    ("Stayman: asks for a 4-card major"), `alert` when it is alertable, and
    a `shows` for what the hand promises; `when` only for what is not
    promised to partner.
  - Write later rounds against state (`asked`, `answered`, `we.trump`,
    `strength` bands), not against listed auctions (LANGUAGE.md §10).
  - Use `priority` sparingly, and say why in a trailing comment.
  - Name a condition that recurs (`define`, LANGUAGE.md §3) rather than
    copying it; judgment thresholds belong in named conditions, where
    they can be cited and tuned.
- **Priority bands.** Priority ranks first, so a band says which layer a
  rule belongs to (docs/JUDGMENT-LAYER.md §4):

  | band | layer | today |
  |---|---|---|
  | +1 … +20 | overrides inside a dialogue | control-bid stop-in-game (20), keycard ask in a control dialogue (11-15), control bids (10-12), keycard ask over a limit raise (1) |
  | 0 … −3 | sequence rules and question-and-answer | the default |
  | −4 … −15 | **judgment**: competition, placement, slam entry | Law of Total Tricks −4, notrump ladder −8/−9, over-3NT −10; sequence modules' own closing passes (−5 … −10) sit here too |
  | −16 … −45 | keep-alive sign-offs in a force | slam-catch game −20, base.bid game in a known fit −38 … −41, game −40 |
  | −20 now, ≤ −50 proposed | silence ("nothing more to say") | the defending side and after-interference passes at −20; base.bid "Game reached" at −50 |

  A judgment rule sits below every sequence rule that describes the hand,
  and above silence. Two cautions. The silence passes at −20 share a
  number with slam-catch's sign-off, so a judgment rule written below −20
  loses to them; they are not yet moved to −50, because there they meet
  base.bid's "Game reached" pass (tried 2026-09-30: 6,942 passes read by
  the other rule, 20 calls changed off our own auctions), so the move is a
  measured rule change of its own. And a sequence module's closing pass is
  either a judgment ("minimum: no game", which should outrank the layer)
  or silence (which should sit below it): say which in its comment.
- **A judgment call must not be read as another rule.** Partner reads a
  call as the highest-priority rules for it whose contexts hold
  (LANGUAGE.md §8). A fallback rule that bids a call a higher rule also
  offers in the same position is read as the higher rule, which is what
  its hand did *not* have. Keep a judgment rule off calls that sequence
  rules give a meaning in the same position (Rick, 2026-09-30: discipline
  and a count, no engine change). `rbb compare` counts our calls read as
  a higher-priority rule than the one that chose them, with the top
  pairs, so a new judgment rule that adds to the count shows up.
  - Comments say why, with the source: a ruling (`Rick, 2026-09-24`), a
    probe, a corpus figure. Longer evidence belongs in the notes.

## Term reference

Generated by `rbb bid terms` from `crates/engine/src/eval.rs`; `cargo test`
fails when it is out of date. Regenerate with
`cargo run -q -p rbb-cli -- bid terms --doc docs/CONTRACT.md`. The
grammar's own words (`after`, `when`, `asked`, `answered`, `maybe`, `is`,
`in`, `shape`, `cheapest(x)`, `jump(x)`) are in LANGUAGE.md.

<!-- BEGIN GENERATED: rbb bid terms -->

### My own hand

Exact while I choose a call; what I have shown when my call is being read. Bare or with `me.` (`me.hcp`). They may not appear in a context's `when`.

| Term | Meaning |
|---|---|
| `hcp` | high-card points (A 4, K 3, Q 2, J 1) |
| `tens` | tens held |
| `points` | total points: HCP + 1/2 a ten + 1 a card beyond four (whole part) |
| `suit_points` | suit points: HCP + 1/2 a card beyond four (whole part) |
| `balanced` | 4-3-3-3, 4-4-3-2 or 5-3-3-2 |
| `semibalanced` | no singleton or void, no suit longer than six |
| `shortest` | length of the shortest suit |
| `longest` | length of the longest suit |
| `second_longest` | length of the second-longest suit |
| `length_points` | one for each card beyond four in every suit |
| `bba_nt_points` | BBA's count opposite 1NT, scaled so it invites from 8 (matchpoints) |
| `bba_nt_game_points` | BBA's count opposite 1NT, scaled so it bids game from 10 (matchpoints) |
| `bba_nt_imp_points` | `bba_nt_points` at IMPs |
| `bba_nt_imp_game_points` | `bba_nt_game_points` at IMPs |
| `bba_stay_nt_points` | BBA's count after Stayman, no fit: 3NT rather than 2NT from 10 (matchpoints) |
| `bba_stay_nt_imp_points` | `bba_stay_nt_points` at IMPs |
| `bba_stay_raise_points` | BBA's count after Stayman, heart fit: invite rather than pass from 8 (matchpoints) |
| `bba_stay_raise_imp_points` | `bba_stay_raise_points` at IMPs |
| `bba_stay_game_points` | BBA's count after Stayman, heart fit: game rather than invite from 10 (matchpoints) |
| `bba_stay_game_imp_points` | `bba_stay_game_points` at IMPs |
| `bba_stay_sraise_points` | BBA's count after Stayman, spade fit: invite rather than pass from 8 (matchpoints) |
| `bba_stay_sraise_imp_points` | `bba_stay_sraise_points` at IMPs |
| `bba_stay_sgame_points` | BBA's count after Stayman, spade fit: game rather than invite from 10 (matchpoints) |
| `bba_stay_sgame_imp_points` | `bba_stay_sgame_points` at IMPs |
| `controls` | controls: ace 2, king 1 |
| `losers` | losing-trick count |
| `quick_tricks` | quick tricks, whole part: A-K 2, A-Q 1½, A 1, K-Q 1, K-x ½ (Culbertson) |
| `bare_suits` | how many side suits are `bare(x)`: 2+ cards without the ace or king |

### Functions of my own hand

Bare or with `me.`. `x` is a suit, a suit variable or `trump`.

| Term | Meaning |
|---|---|
| `tp(x)` | total points with x as trump: support points, shortness capped by trumps held |
| `keycards(x)` | aces plus the king of x (`keycards(N)`: the four aces) |
| `has(rank, x)` | holds that card (A K Q J T) in x |
| `quality(x)` | top honours (A, K, Q) in x: poor 0, fair 1, good 2, excellent 3 |
| `top5(x)` | how many of A K Q J T are held in x |
| `stop(x)` | a stopper in x: A, Kx, Qxx or Jxxx |
| `bare(x)` | x is a side suit (not the agreed trump) of 2+ cards without the ace or king |
| `safe_level(x)` | the level our trumps make safe, the Law of Total Tricks: `we.fit(x).min - 6` |

### Auction state

Bare or with `me.`. Public, so fine in a context's `when`, except the judgment hooks `slam_try` and `grand_try`.

| Term | Meaning |
|---|---|
| `opening` | no one has bid yet |
| `last` | my own last call (`me.last`); compare with a call: `me.last=1N` |
| `passed_hand` | I have called, and only passed |
| `bids` | how many bids I have made, not counting passes and doubles (`me.bids`) |
| `has_bid` | I have made a bid, not only passes or doubles (`me.has_bid`) |
| `seat` | my seat from the dealer, 1 to 4 |
| `vul` | my side is vulnerable |
| `game_reached` | our side's last bid is game or higher and it is our contract |
| `imps` | IMPs or other total-point scoring |
| `matchpoints` | matchpoints or board-a-match |
| `favourable` | we are not vulnerable and they are |
| `unfavourable` | we are vulnerable and they are not |
| `captain` | I place the contract: partner's shown points span 4 or fewer (12-15) and mine more, or partner has answered my question |
| `trump` | `we.trump` |
| `slam_try` | judgment hook: slam is worth trying (placeholder: combined HCP >= 31) |
| `grand_try` | judgment hook: grand slam is worth trying (placeholder: combined HCP >= 35) |

### My position in the auction

Bare or with `me.`.

| Term | Meaning |
|---|---|
| `denied(x)` | in the control-bid dialogue I skipped x (`me.denied(x)`) |
| `cued(x)` | in the control-bid dialogue I have shown a control in x |
| `under_game(x)` | the cheapest bid in x is below game in our agreed suit (3NT if none) |
| `cheapest_rank(x)` | the cheapest bid in x as level x 5 + C0 D1 H2 S3 (`prefer 0 - cheapest_rank(x)`) |

### Other seats

With `partner.`, `lho.`, `rho.`, or `shown.` (what I have shown), besides a suit or suit variable for its length (`partner.S`, `partner.M`). Values are ranges: a comparison holds when it is known; `maybe` asks whether it is still possible.

| Term | Meaning |
|---|---|
| `partner.hcp` | HCP shown |
| `partner.tens` | tens (never tracked: 0..4) |
| `partner.points` | total points shown (HCP when no call showed points) |
| `partner.suit_points` | suit points shown |
| `partner.balanced` | shown balanced |
| `partner.shortest` | length of the shortest suit, as far as shown |
| `partner.longest` | length of the longest suit, as far as shown |
| `partner.second_longest` | length of the second-longest suit, as far as shown |
| `partner.length_points` | cards beyond four, as far as shown |
| `partner.last` | that seat's last call (`partner.last=3N`, `=P`, `=X`, `=XX`) |
| `partner.opened` | that seat made the opening bid |
| `partner.bids` | how many bids that seat has made |
| `partner.has_bid` | that seat has made a bid, not only passes or doubles |
| `partner.jumped` | that seat's last bid was at least a level above the cheapest in its strain |
| `partner.denied(x)` | in the control-bid dialogue that seat skipped x |
| `partner.cued(x)` | in the control-bid dialogue that seat has shown a control in x |
| `partner.bypassed(x[, call])` | that seat's last bid went past an available bid in x (above `call` when given) |
| `partner.named(x)` | that seat has made a natural bid in x (a suit or N) at any point; calls a rule marks artificial do not count |
| `partner.has(rank, x)` | not tracked: unknown |
| `partner.stop(x)` | not tracked: unknown |
| `partner.semibalanced` | not tracked: unknown |
| `partner.tp(x)` | support points with x as trump, when a raise showed them; else HCP |
| `partner.keycards(x)` | not tracked: 0..40 (see `we.keycards`) |
| `partner.controls` | not tracked: 0..40 |
| `partner.losers` | not tracked: 0..40 |
| `partner.quick_tricks` | not tracked: 0..40 |
| `partner.bare_suits` | not tracked: 0..4 |
| `partner.bare(x)` | not tracked: unknown |
| `partner.trump` | length in our agreed suit (`partner.trump.min`) |
| `partner.quality(x)` | not tracked: 0..40 |
| `partner.top5(x)` | not tracked: 0..40 |

### Our partnership

| Term | Meaning |
|---|---|
| `we.trump` | the agreed strain (`is suit`, `is notrump`, `is none`), usable as a suit |
| `we.forcing` | `none`, `round` or `game` |
| `we.named(x)` | either of us has made a natural bid in x (a suit or N) at any point; artificial calls do not count |
| `we.gf` | we are in a game force (`we.forcing = game`) |
| `we.hcp` | my HCP plus partner's range: `hcp + partner.hcp` (`.min`, `.max` are partner's ends) |
| `we.points` | my points plus partner's range: `points + partner.points` |
| `we.fit(x)` | my length in x plus partner's range: `x + partner.x`; `.min` is the known fit |
| `we.tp(x)` | my support points with x as trump plus partner's: `tp(x) + partner.tp(x)` |
| `we.keycards(x)` | my keycards plus partner's answer, within the deck's five |

### The opponents

| Term | Meaning |
|---|---|
| `they.bid` | the opponents have bid or doubled |
| `they.vul` | the opponents are vulnerable |
| `they.hcp` | LHO's and RHO's HCP ranges added |
| `they.fit(x)` | LHO's and RHO's lengths in x added |
| `they.level` | the level of the opponents' last bid (none before they bid) |
| `they.strain` | the strain of the opponents' last bid, usable as a suit |
| `they.still_bidding` | one of the opponents' last calls is not a pass (or one has not called yet) |
| `they.game_reached` | the opponents' last bid is game or higher and it is their contract |

### Other names

| Term | Meaning |
|---|---|
| `N` | notrump, as a strain (`NT` too) |
| `NT` | notrump, as a strain |
| `strength` | my total points as a band: `strength=invite`, `strength>=game` |
| `suit_strength` | the same with suit points |

### Words

Values compared with `=` or `is`.

| Term | Meaning |
|---|---|
| `game` | a forcing level (`we.forcing = game`), and a strength band |
| `round` | a forcing level: partner may not pass at the next turn |
| `none` | no forcing level; with `is`, no trump agreed (`we.trump is none`) |
| `suit` | with `is`: the agreed trump is a suit (`we.trump is suit`) |
| `notrump` | with `is`: the agreed strain is notrump |
| `poor` | suit quality 0: none of A, K, Q |
| `fair` | suit quality 1: one of A, K, Q |
| `good` | suit quality 2: two of A, K, Q |
| `excellent` | suit quality 3: A, K and Q |
| `signoff` | strength band: too weak to invite (`strength=signoff`) |
| `invite` | strength band: invitational opposite partner's range |
| `slam_invite` | strength band: enough to invite slam |
| `slam` | strength band: enough for slam opposite partner's minimum |
| `A` | the ace, in `has(A, x)` |
| `K` | the king, in `has(K, x)` |
| `Q` | the queen, in `has(Q, x)` |
| `J` | the jack, in `has(J, x)` |
| `T` | the ten, in `has(T, x)` |

### Names accepted by form

Checked by shape rather than listed.

| Term | Meaning |
|---|---|
| `S H D C` | a suit: its length in a comparison (`S>=5`), else the suit (`stop(S)`) |
| `M, m, x, y, z, t, ...` | suit variables, bound by a pattern (`after 1M (P)`) or a question; `M` a major, `m` a minor |
| `<param>` | a module `param`, the card value it names |
| `partner.bba_*` | any `bba_` count of another seat: unknown (0..40) |
| `.min / .max` | the ends of a range: `partner.hcp.min` |

### State set by `sets`

Not conditions: what a rule's `sets` clause may assign (`sets forcing=game, trump=x`). Another name, or a value of another form, refuses the load.

| Term | Meaning |
|---|---|
| `sets forcing=round/game/none` | `round`: partner may not pass if RHO passes; `game`: neither partner may pass below game (a round force never weakens it); `none` ends a force |
| `sets trump=x` | the agreed strain: a suit, `N`/`NT`, a suit variable or `trump` |
| `sets ladder=control/stopper` | a ladder call (control or stopper bids, recorded alike): the suits it skipped, other than trump, become `denied(x)` for the caller, the suit it names `cued(x)` |
| `sets ask=kind(x, ...)` | a question to partner, read by `asked kind(...)` and `answered kind(...)`; the kind is a free name, the arguments (optional) each a suit, `N`/`NT`, a suit variable or `trump` |

<!-- END GENERATED: rbb bid terms -->
