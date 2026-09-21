# The rule language (`.bid` files)

Status: **draft for discussion.** Nothing parses this yet. The examples in
[`conventions/`](../conventions/) are written in this syntax to test it.

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

**What I have already shown** uses the prefix `shown.`, for example
`hcp >= shown.hcp.max` for "I am at the top of my range".

**Auction state:** `opening` (no one has bid yet), `we.trump`, `we.forcing`
(`none | round | game`), `asked <kind>` (partner's pending question to me),
`answered <kind>` (partner answered my question), `partner.last`, `opener`,
`partner.opened`, `they.bid`, `seat`, `passed_hand`, `vul`, `they.vul`.

**Operators:** `!` (not), `|` (or), `,` (and), `a..b` ranges, `in 1|4` sets,
and arithmetic on numbers and `.min` / `.max`. Precedence, tightest first:
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
  `shows`, and later calls can narrow it.
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

## 10. Open questions

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
