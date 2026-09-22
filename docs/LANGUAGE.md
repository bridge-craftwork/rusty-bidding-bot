# The rule language (`.bid` files)

Status: **draft, parsed and running.** The `bidspec` crate parses this syntax
into a JSON IR (`rbb bid check`, `rbb bid compile <file>`), and `rbb-engine`
executes it (`rbb call`). Section 11 lists how the engine currently reads
the vocabulary, including what is not implemented yet. The examples in [`conventions/`](../conventions/) all compile.

## 1. The model: rules describe calls; the engine tracks what is known

Rules cannot be keyed only by auction. There are thousands of auctions that
reach a Keycard 4NT, and nobody can list them. So the engine does not look up
"what do I bid after this auction". Instead it walks the auction **one call at
a time**, building up two things:

- **Knowledge about each hand**: what each seat has shown so far, as ranges
  and sets (partner has 15–17 HCP, 4+ spades, 1 or 4 keycards…).
- **Auction state** for each side: the agreed trump suit, whether we are in a
  forcing or game-forcing auction, any pending question (e.g. a keycard ask
  addressed to me).

Rules are written against that knowledge and state. A Keycard rule says "when
we have agreed a suit and are forcing to game, 4NT asks for keycards". It does
not care how we got there.

Every call goes through the same two steps, whoever makes it:

```
             ┌────────────── for every call already made ───────────────┐
 auction ──► │ INTERPRET: find the rule that gives this call its meaning, │──► knowledge
             │ add its `shows` to the caller's hand, apply its `sets`     │    + state
             └────────────────────────────────────────────────────────────┘
                                                                              │
             ┌──────────────────────── my turn ─────────────────────────┐    │
 my hand ──► │ GENERATE: every rule whose context holds now → candidates │◄───┘
             │ FILTER:   keep candidates my hand satisfies               │
             │ RANK:     score the survivors; best one is my call        │──► call +
             └────────────────────────────────────────────────────────────┘    explanation
                                                                               + losing alternatives
```

A rule is written once and used in both directions. When I make the call, its
`shows` must be true of my hand. When partner makes it, its `shows` becomes
what I know about partner.

## 2. Worked example

South opens 1♠, then North bids a Jacoby 2NT. North later asks for keycards,
and no rule mentions this auction:

| Call | Rule that gives it a meaning | Knowledge added | State change |
|---|---|---|---|
| S: 1♠ | `base` opening | S: hcp 12–21, ♠ ≥ 5 | NS opened |
| W: Pass | `base` | W: denies an opening bid | |
| N: 2NT | `jacoby-2nt` (auction pattern `1M (P)`) | N: ♠ ≥ 4, game-forcing values | NS: **trump = ♠, forcing = game** |
| E: Pass | | | |
| S: 3♣ | `jacoby-2nt` responses | S: ♣ ≤ 1 | |
| W: Pass | | | |
| N: 4NT | `rkcb-1430`: context is only **`we.trump is suit, we.forcing = game`** | N: slam interest | NS: **ask = keycards(♠) to S** |
| E: Pass | | | |
| S: 5♥ | `rkcb-1430` answers: context **`asked keycards`** | S: keycards(♠) = 2, no ♠Q | ask answered |
| W: Pass | | | |
| N: 6♠ | `rkcb-1430` follow-ups: `we.keycards(trump) = 4` | | |

The 4NT rule matched because the Jacoby 2NT rule had set `trump = ♠` and
`forcing = game` two calls earlier. The same rule would fire after
1♥–2♣–2♠–3♠–… or any other auction that agrees a suit in a game force.

The knowledge store also applies **deck constraints**: 40 HCP in the deck, 13
cards in each suit, 5 keycards. So if North holds 2 keycards and South shows
"1 or 4", North's knowledge of South tightens to exactly 1 without any rule
saying so.

## 3. Files and modules

One module per file. The header states when the module is active and what
it depends on.

```
module rkcb-1430 "Roman Keycard Blackwood (1430)"
  card   slam.blackwood.rkcb_1430          # active when the card says so
  needs  base
  param  nt_min = notrump.one_nt.range_min default 15
```

- `card <path> [= value]`: activation condition, read from the convention
  card. Several `card` lines must all hold. No `card` line means the module is
  always active once something `needs` it (base systems).
- `param <name> = <card path> [default v]`: a card value the rules can use by
  name.
- `needs <module>`: load order and dependencies.

## 4. Contexts

A **context** says when a group of rules applies. Rules are indented under
it. Contexts can nest, and nested conditions are combined with AND.

### Auction patterns: `after`

For the early, well-mapped part of an auction.

```
after 1N (P)                   # partner opened 1NT, RHO passed, my turn
after 1M (P) 2N (P)            # M binds to hearts or spades
after (1x) X (P)               # they opened, partner doubled, RHO passed
```

- Our side's calls are bare; the opponents' calls are in parentheses. Calls
  alternate, and the pattern always ends with the call made just before my
  turn.
- Leading passes are skipped unless written. Use the state `seat` or
  `passed_hand` when position matters.
- `(*)` means any opponent call. Suit variables: `M` = a major, `m` = a minor,
  `x` / `y` = any suit, `N` = notrump. A variable binds on first use and can be
  used in the rules below (`shows M>=4`).

### State conditions: `when`

For everything else. These are the rules that scale.

```
when we.trump is suit, we.forcing = game, !asked
when asked keycards(t)            # partner asked me; binds t to the suit
when they.bid, partner.opened     # competitive: we opened, they came in
```

A context can combine both: `after 1N (P) when !passed_hand`.

## 5. Rules

```
<call>  "<explanation>"  [alert ["<text>"]] [announce "<text>"]
    shows    <condition>      # true of the caller's hand; becomes knowledge
    when     <condition>      # selection-only condition, not promised to partner
    sets     <assignments>    # auction state changes
    denies   <condition>      # explicit negative inference
    prefer   <expression>     # score used when ranking (section 7)
    priority <n>
    replaces <module>[.<rule-id>]
    artificial                # not a place to play: passing it out is a mistake
    as       <rule-id>
```

Short clauses can go on the rule's own line:

```
after 1N (P)
  2D  "Transfer to hearts"  alert  shows H>=5  sets forcing=round
```

### Calls

- Literal calls: `1N`, `2D`, `P`, `X`, `XX`.
- Variables: `2M`, `3x`.
- State interpolation: `6{trump}`, `5{trump}`.
- Relative calls: `cheapest(x)`, `jump(x)`, `raise`, `new_suit`.

The explanation string interpolates the same way: `"Keycard ask in {trump}"`.

## 6. Conditions and expressions

**My hand** (exact values, when I am the one choosing a call):

| Term | Meaning |
|---|---|
| `hcp`, `tp(x)`, `controls`, `losers` | point counts; `tp` = total points with `x` as trump |
| `S H D C`, `M`, `x` | length of that suit |
| `balanced`, `semibalanced`, `shape 5-4-x-x`, `shape 4333`, `shortest`, `longest` | shape |
| `stop(x)`, `quality(x) >= good`, `has(Q, x)` | suit holdings |
| `keycards(x)` | aces + trump king, with `x` as trump |

**Other seats** use the same terms with a prefix: `partner.`, `lho.`, `rho.`.
Their values are **ranges**, so a comparison means "known to be true":
`partner.S >= 3` holds only if partner has *shown* 3+ spades. Use
`maybe partner.S >= 3` for "not ruled out", and `.min` / `.max` for arithmetic.

**Partnership totals**, `we.hcp` and `we.keycards(x)`, combine my exact hand
with partner's range. Example: `we.hcp.min >= 25`.

**Points.** Two measures, kept in quarter points and compared by their
whole part (`points=8..9` means 8 up to 9¾):

- `points`, for notrump: HCP plus ½ for each ten.
- `suit_points`, for a suit contract: HCP plus ½ for each card beyond four.

This is what BBA does, found by probing it hand by hand with `rbb probe`
(the same hand with 0-4 tens; with and without a long suit): tens count at
notrump and not in a suit contract; length counts in a suit contract and not
at notrump. Nines and eights showed no effect. The weights are engine
settings (`Valuation`). `hcp` stays pure HCP: BBA's 1NT is 15-17 HCP
exactly, whatever the tens or length.

**Strength** compares with a band: `strength=invite`, `strength>=game`
(notrump points), or `suit_strength=invite` (suit points). Bands,
weakest first: `signoff`, `invite`, `game`, `slam_invite`, `slam`. Each is my
total points given partner's range p in whole points (partner's points when
a call showed points, else partner's HCP), for 25 combined for game and 33
for slam: signoff ≤ 24−p.max; invite 25−p.max..24−p.min; game
25−p.min..32−p.max; slam_invite 33−p.max..32−p.min; slam ≥ 33−p.min. A band
can be empty: after a super-accept (p known exactly) there is no invite.
Because it becomes a points range, partner infers my strength from it too.
Explanations can quote a band: `"Invitational: {invite} total points"`
prints "Invitational: 8-9 total points" after a 15-17 1NT.

**What I have already shown** uses the prefix `shown.`, for example
`hcp >= shown.hcp.max` for "I am at the top of my range".

**Auction state:** `opening` (no one has bid yet), `we.trump`, `we.forcing`
(`none | round | game`), `asked <kind>` (partner's pending question to me),
`answered <kind>` (partner answered my question), `partner.last`, `opener`,
`partner.opened`, `they.bid`, `seat`, `passed_hand`, `vul`, `they.vul`,
`we.keycards(t)` (my keycards plus partner's answer, within the deck limit),
`imps` (IMPs and other total-point scoring), `matchpoints` (matchpoints and
board-a-match).

**Operators:** `!` (not), `|` (or), `,` (and), `a..b` ranges, `in 1|4` sets,
and arithmetic on numbers and `.min` / `.max`.

**Suits.** A suit in a comparison is its length (`S>=H`, `M>=4`, `x<=1`).
`is` asks what something *is*: `we.trump is suit` (or `notrump`, `none`), and
`x is M` / `x is not M` for "the same suit" or "a different suit". Precedence, tightest first:
`!`, then `|`, then `,`. So `hcp>=8, H=4 | S=4` reads as "8+ HCP, and a
4-card heart or spade suit". Parentheses override.

**`shows` values are fixed when the call is made.** A rule such as
`shows hcp >= 25 - partner.hcp.min` is evaluated against the auction at that
moment and stored as a plain range.

## 7. Choosing between candidates

The engine asks every rule whose context holds for its call. It then:

1. **Filters** out illegal calls and those whose `shows` and `when` my hand
   does not satisfy.
2. **Ranks** the survivors by, in order:
   1. `priority` (default 0). Use it sparingly, for cases where the
      principle below gives the wrong answer (a waiting or forcing bid that
      deliberately shows little).
   2. **Descriptiveness.** *Decided:* the call that says more about my hand
      wins. With 6♠-4♦, after 1♠ – 1NT, a 2♦ rebid shows ten cards (♠5+ and
      ♦4+) and a 2♠ rebid shows six, so the call that shows more wins unless
      a rule says otherwise. The measure is how much the call's `shows`
      narrows my hand beyond what I have already shown: the fraction of the
      hands still consistent with my earlier calls that would **not** satisfy
      it. That fraction is computed from the hand statistics, not from my
      actual hand, so it is the same for every hand. That matters for
      negative inference (section 8). A Stayman 2♣ therefore outranks a
      natural 2♣ without anyone giving it a priority number.
   3. **`prefer` score**, a numeric expression for judgment between otherwise
      equal calls. Example: `prefer x` chooses the longer suit, and
      `prefer quality(x)` chooses the better one.
   4. order in the file.
3. Returns the winner **and the ranked losers with the reason each lost**, so
   every call can be explained ("2♥ not chosen: needs 5+ hearts").

This ranking step is the scoring you suggested. It is deterministic and
explainable, and it can be replaced with other scorers later. Because the
knowledge store gives constraints for every hand, a future scorer could deal
hands consistent with what has been shown (dealer3) and score
candidate contracts double-dummy (bridge-solver). That would add simulation
without changing the rule files.

### Judgment hooks

Some decisions are better written in Rust than as conditions: "is slam
worth trying", "invite or bid game", hand upgrades. These are **named
evaluators** registered by the engine and used like terms:
`when slam_try`, `prefer upgrade(hcp)`. The language stays small, and the
hard judgment code is in one place that can be tested.

## 8. Interpreting other players' calls

To interpret a call, the engine runs the same rules from the caller's point
of view, using **the caller's card** (so the opponents' conventions are read
with their card). It then takes the highest-ranked rule for that call whose
context holds. It does not check the caller's hand, which it cannot see.

- If several rules give the same call different meanings (a Multi 2♦, or a
  2NT that is natural or a relay), the knowledge becomes the **union** of their
  `shows`, and later calls can narrow it. Only the rules of the highest
  `priority` count: a lower-priority rule for the same call is a fallback
  ("only if nothing better") and does not widen the call's meaning.
- A rule's `when` rules it out if it depends only on public knowledge and is
  not known true. (`when partner.M>=4` cannot be what the caller relied on
  before partner has shown four.) A `when` that depends on the caller's own
  hand (`slam_try`, `shape 4333`, `we.keycards(t)`) stays possible.
- **Negative inference.** *Decided:* this is automatic, and the knowledge
  model must support it. The caller would have made any higher-ranked
  candidate its hand satisfied, so a call also says the caller's hand fails
  every candidate that outranks it. For example, "did not open 1NT" means
  "not (15–17 and balanced)", and "rebid 2♠ rather than 2♦" means "not 4+
  diamonds". This only works because the ranking order does not depend on the
  hand (section 7). Where `prefer` breaks a tie using the hand, the inference
  is skipped for that group. `denies` can still add an inference by hand.

  **Knowledge representation.** Each seat's knowledge is a set of
  constraints, which may include negations of conjunctions ("not (A and
  B)"). A range summary kept alongside answers the common questions
  quickly (`partner.hcp.min`). When later constraints settle a
  disjunction, it collapses into the ranges. Example: "not (15–17 and
  balanced)" plus a later "balanced" becomes "hcp not 15–17".
- If no rule matches, a built-in natural interpretation applies. An
  unmatched call is reported; it usually means a convention is missing.

## 9. Layering and conflicts

- Base systems (`base-2over1`, `base-sayc`, `base-precision`) are ordinary
  modules; conventions `needs` them.
- A rule that intentionally replaces another says `replaces <module>.<rule>`
  (or the whole module's rule for the same call and context).
- If two active rules claim the same call in overlapping contexts, with equal
  priority and descriptiveness and no `replaces`, **loading fails** and names both
  files and lines.

## 10. Writing continuations without listing auctions

`after` patterns are for the opening of an auction, where the calls really
are the context (`after 1N (P)`). Later rounds should not name auctions: there
are too many ways to reach the same point. Write them against **state** and
**what has been shown**:

- **Questions and answers.** A call that asks partner to do something (a
  transfer, Stayman, keycard) says `sets ask=transfer(H)`. Partner's next call
  answers it, whatever it is. The asker's second call is then written once:

  ```
  when answered transfer(M)          # after 1NT, 2NT, or with interference
    P   "Weak: play here"            shows strength=signoff
    2N  "Invitational, exactly 5 {M}" shows M=5, balanced, strength=invite
    4M  "Game in the known fit"      shows strength=game   when partner.M>=3
  ```

- **Strength bands** (`strength`, section 6) turn "invite / game / slam" into
  HCP ranges relative to partner's shown range, so one rule reads correctly
  after 1NT, 2NT or a super-accept.

- **Knowledge, known vs possible.** `partner.M>=3` is true only when partner
  has *shown* three. "The fit is not known yet" is `maybe partner.M<=2`, not
  `!partner.M>=3` (with partner's length unknown, both of those are unknown).

## 11. Parser rules

These are enforced by `bidspec`, with file:line:column errors:

- Indent with spaces; tabs are an error. Structure follows indentation:
  header lines under `module`, rules under a context, clause lines under a
  rule. A context may contain nested contexts.
- Every rule needs an explanation string straight after its call.
- An `after` pattern must alternate our calls with (their calls), and must
  end with an opponent's call: the one just before my turn.
- `shows`, `when` and `denies` may be repeated, on the rule line or on
  continuation lines; the repeats are combined with AND. `prefer`,
  `priority`, `replaces`, `as` and `alert`/`announce` may appear once.
- `card` and `param` paths must exist in the card registry
  (`crates/bridge-card/data/fields.toml`); an old alias is reported with the
  current name.
- `rbb bid check` also checks across files: module names are unique, and a
  `needs` that names no module is a warning.

## 12. The engine today

How `rbb-engine` implements the model, and its current limits:

- **Suit comparisons** are about length: `S>=H` means at least as many
  spades as hearts, and `x=M` means the same length. To ask whether two
  names are the *same suit*, write `x is M` or `x is not M`.
- A `when` that is false whatever the hand (such as `x is not M` with x bound
  to M) removes the candidate before ranking. It does not appear in the
  trace and is never used for negative inference.
- **Knowledge** holds ranges for HCP and each suit length, and whether the
  hand is balanced. The deck constraint applies: lengths sum to 13, and a
  balanced hand has 2 to 5 cards in every suit. All other terms (`has`,
  `keycards`, `shape`, `quality`) are kept as constraints but do not narrow
  the ranges. Partner's keycards are not tracked yet, so
  `we.keycards(t)` is only known from my own count.
- **Descriptiveness** is measured on a fixed sample of 20,000 random hands.
  It is the share of hands that the call's `shows` rules out, among the
  hands consistent with what the caller has already shown.
- **Negative inference** is automatic. A call denies every candidate that
  outranks it on priority and descriptiveness. It is skipped when the
  denied rule has a term that cannot be resolved (for example `slam_try` or
  `has(Q,t)` about another player's hand), because negating a partial
  condition would claim more than is known.
- **Forcing.** After `sets forcing=round`, partner may not pass at their next
  turn. After `sets forcing=game`, neither partner may pass below game.
- **Judgment hooks** `slam_try` and `grand_try` are placeholders (combined
  HCP of at least 31 or 35) until real evaluators are written.
- **No rule applies:** the engine passes and says so in the trace.
- **Keycards.** Partner's keycard answer is combined with my own count and
  the deck limit (five in all): "1 or 4" facing my 2 is 1. When it stays
  ambiguous the asker signs off in five of the suit, and the answerer
  corrects to six holding the higher count.
- **Not yet implemented:** `replaces`, `raise` and `new_suit`, and the
  conflict check between modules.

## 13. Open questions

1. **Relative call notation.** Are `cheapest(x)` / `jump(x)` enough, or do we
   need step notation (`step 1`, `step 2`) for relay systems and Kickback?
2. **Measuring descriptiveness.** Exact fractions from hand-distribution
   tables, or estimates from a fixed sample of dealt hands? Either way,
   they can be precomputed per rule and context.
3. **How much of "natural bidding" is rules vs. built in.** Proposal: a
   `natural.bid` module for everything expressible, plus Rust judgment hooks.
4. **Competitive defaults.** For example, "after interference, systems on
   over a double, off over a suit bid". Card-driven state rules, or explicit
   patterns in each module?
