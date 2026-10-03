# The rule language (`.bid` files)

Status: **draft, parsed and running.** The `bidspec` crate parses this syntax
into a JSON IR (`rbb bid check`, `rbb bid compile <file>`), and `rbb-engine`
executes it (`rbb call`). Section 11 lists how the engine currently reads
the vocabulary, including what is not implemented yet. The examples in [`conventions/`](../conventions/) all compile.

This is **rule language 1**. [CONTRACT.md](CONTRACT.md) says what that
version promises, how a rules directory declares it (`conventions.toml`),
and lists every term a condition may use (`rbb bid terms`).

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
  skill  bidding_conventions/roman_keycard
  param  nt_min = notrump.one_nt.range_min default 15
```

- `card <path> [= value]`: activation condition, read from the convention
  card. Several `card` lines must all hold. No `card` line means the module is
  always active once something `needs` it (base systems).
- `param <name> = <card path> [default v]`: a card value the rules can use by
  name.
- `needs <module>`: load order and dependencies.
- `skill <category>/<name>`: a teaching skill the module implements, as
  Bridge-Classroom tags its lessons (`skill bidding_conventions/stayman`).
  Repeatable, and a line may name several. It records what the module
  teaches and changes nothing about how it loads or bids; `rbb bid skills`
  maps skills to card fields and modules (docs/SKILLS.md).

**Treatments.** When players agree on a convention but not on its details,
the card holds one enum field for the choice (`notrump.minor_transfers`:
`relay`, `four_way`, `four_way_reversed`, `bba`, `none`). The module reads
it as a param and tests it with `is`:

```
module minor-transfers "Minor-suit responses to 1NT"
  param  style = notrump.minor_transfers default relay

after 1N (P) when style is relay
  2S  "Relay to 3♣"  ...
after 1N (P) when style is four_way | style is four_way_reversed
  2S  "Transfer to clubs"  ...  sets ask=transfer(C)
when answered transfer(m)          # shared by every treatment
  ...
```

Only the rules that give a call its meaning differ between treatments. What
follows is written once against state (`answered transfer(m)`), whichever
call made the transfer. `rbb bid check` rejects an option the field does not
have (`style is four-way`), which would otherwise never match. Another
module can read the same field: `one-nt.bid` turns off its natural 2NT when
2NT is a transfer.

**Named conditions.** `define` at the left margin names a condition (or a
value) that any rule of the rule set may use, in any module:

```
define controlled(x) = has(A,x) | (x<=0, partner.x.min<=3) | (style is first_or_second_round, has(K,x) | (x<=1, partner.x.min<=3))
define slam_values = we.tp(trump).min >= 33 | tp(trump) >= 18

when we.trump is suit, !asked, we.forcing = game
  4N  "Keycard ask in {trump}"
      when  slam_values, controlled(C), controlled(D)
```

A definition takes suit parameters (one lower-case letter, or `M`); the
arguments are a suit, a suit variable or `trump`. Its condition may
continue on indented lines, each one more part (joined like `,`). The
engine inlines it where it is used (so a rule reads, is read and resolves
exactly as if the condition were written out), with the definition's card
parameters read from its **own** module (`style` above is
`slam-entry.bid`'s `slam.cue_bids.style`, whatever `style` means in the
module that uses it). Definitions are shared across the rule set whether
or not their module is active; a name may be defined once, may not be a
term, and may not use itself. The reference (`rbb bid reference`) prints
rules as written. Engine 0.3.0.

**Forces from the shape of the auction.** `force game [after <auction>]
[when <condition>]` at the left margin puts the caller's side in a game
force (`we.forcing = game`) whenever a call is made where the pattern and
condition hold, whichever rule made or explains the call. The pattern
matches the auction before the call, as a context's does; `call` is the
call being made. The condition is public (no terms of the caller's hand).
A rule whose own meaning forces says so with `sets forcing=game`; `force`
is for what belongs to the auction (base/game-force.bid):

```
force game when partner.opened, !they.bid, me.bids = 2, call >= 3C, call <= 3S
force game after 1x (P) 1y (P) 2N (P) when !call = P, !(wolff, call = 3C)
```

Engine 0.4.0.

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
- **Alternatives.** `after A | B` holds when either pattern matches, and the
  first that matches supplies the bindings. This is how "systems on" is
  written: the auction after our 1NT overcall is the auction after a 1NT
  opening, one call later.

  ```
  after 1N (P) | (1x) 1N (P) | (1x) P (P) 1N (P) when !they.bid | systems_on
    2C  "Stayman" ...
  ```

  Rules under such a context must not rely on a variable that only one
  alternative binds, unless a `when` rules the others out.
- Leading passes are skipped unless written. Use the state `seat` or
  `passed_hand` when position matters.
- `(*)` means any opponent call. Suit variables: `M` = a major, `m` = a minor,
  `x` / `y` / `z` = any suit, `N` = notrump. A variable binds on first use and can be
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
| `hcp`, `tp(x)`, `controls`, `losers`, `tens` | point counts; `tp` = total points with `x` as trump; `tens` = tens held |
| `S H D C`, `M`, `x` | length of that suit |
| `balanced`, `semibalanced`, `shape 5-4-x-x`, `shape 4333`, `shortest`, `longest` | shape |
| `stop(x)`, `quality(x) >= good`, `has(Q, x)`, `bare(x)`, `bare_suits` | suit holdings; `bare(x)`: a side suit of 2+ cards without the ace or king |
| `quick_tricks` | A-K 2, A-Q 1½, A 1, K-Q 1, K-x ½ (whole part) |
| `keycards(x)` | aces + trump king, with `x` as trump |

**Other seats** use the same terms with a prefix: `partner.`, `lho.`, `rho.`.
Their values are **ranges**, so a comparison means "known to be true":
`partner.S >= 3` holds only if partner has *shown* 3+ spades. Use
`maybe partner.S >= 3` for "not ruled out", and `.min` / `.max` for arithmetic.

**Partnership totals** add my exact hand to partner's range, and are
written out that way before a rule is used: `we.fit(x)` is
`x + partner.x`, `we.points` is `points + partner.points`, `we.hcp` is
`hcp + partner.hcp`, `we.tp(x)` is `tp(x) + partner.tp(x)`, and `.min` /
`.max` take partner's end (`we.fit(S).min >= 8` is
`S + partner.S.min >= 8`: a known eight-card fit). So a `shows` with them
is still a condition on my own hand, and partner reads it.
`safe_level(x)` is `we.fit(x).min - 6`, the level the Law of Total Tricks
makes safe. `we.keycards(x)` is my keycards plus partner's answer, within
the deck's five. Remember that a range compares as known: `we.fit(S) <= 7`
holds only once partner's length is known, and "no known fit" is
`we.fit(S).min <= 7`.

**Opponents' totals and last bid:** `they.hcp` and `they.fit(x)` (both
opponents' ranges added), `they.level` and `they.strain` (their last bid;
the strain is usable as a suit), `they.still_bidding` (one of their last
calls is not a pass) and `they.game_reached`.

**Conditions as numbers.** In arithmetic a condition counts 1 when it holds
and 0 when it does not (a range 0..1 while it is unknown), so a count can
carry its adjustments: `we.fit(x).min + doubler_four(x) - unfavourable = 9`
(total-tricks.bid). Engine 0.3.0.

**Points.** Kept in quarter points and compared by their whole part
(`points=8..9` means 8 up to 9¾). Rick's model is three measures, with each
rule choosing the one that fits its decision: HCP, total points, and
support points.

- `hcp`: pure HCP. The 1NT opening uses it (15-17 HCP exactly, whatever
  the tens or length), and so can any rule where length should not count.
- `points`, total points: HCP plus ½ for each ten plus 1 for each card
  beyond four. `strength` bands use it.
- `tp(x)`, support points once a fit in `x` is known (shortness, capped by
  the trumps held).
- `suit_points` (HCP plus ½ for each card beyond four) and
  `hcp+length_points` (HCP plus 1 a card beyond four) remain from earlier
  rules.

Probing BBA hand by hand (`rbb probe`: the same hand with 0-4 tens, with
and without a long suit) showed that it counts tens and not length at
notrump. We count length anyway (Rick, 2026-09-23: "total points should
include length points"). Against double-dummy par, over the whole corpus:
½ a point a card gained 6,526 IMPs, 1 a point gained 9,718; counting a full
point in `suit_points` as well gained another 2,957 but needs its bands
reset first. With `general.style = bba` a side counts as BBA does: tens,
and no length. Two variants were tried and rejected on par: a ten only with a
neighbouring honour (KT, QT, JT; −1,647), and ½ or 1 point off a 4-3-3-3
(−2,543 / −5,670). The weights are engine settings (`Valuation`).

**Strength** compares with a band: `strength=invite`, `strength>=game`
(total points), or `suit_strength=invite` (suit points). Bands,
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
`answered <kind>` (partner answered my question), `partner.last`,
`partner.opened`, `lho.opened`, `rho.opened` (there is no form for my own
opening: write `!lho.opened, !rho.opened` with `they.bid`), `they.bid`,
`seat`, `passed_hand`, `vul`, `they.vul`, `favourable` (we are not
vulnerable, they are), `unfavourable`, `captain` (I place the contract:
partner has limited his hand to a range of four points or less, 12-15, and
mine is wider; or partner has answered my question),
`we.keycards(t)` (my keycards plus partner's answer, within the deck limit),
`imps` (IMPs and other total-point scoring), `matchpoints` (matchpoints and
board-a-match), `me.last` (my own last call). A last call compares with a
call: `partner.last=3N`, `partner.last=3{t}`, and `partner.last=P`, `X` or
`XX` for pass, double and redouble. Bids are also ordered, in the
bidding order: `rho.last <= 2{M}` holds when RHO's last bid was no higher
than two of M (a pass or double compares only with `=`). A text card
field holding a bid (`"2♥"`, `"2H"`, `"3NT"`) compares as that bid, so a
card's limit can be read with a `param`: `rho.last <= through`
(support-doubles.bid, `doubles.support.through`). Text that is not a bid
makes the comparison an error, reported as a warning, and the condition
does not hold.

**Skipping a suit.** `partner.bypassed(x)` is true when partner's last call
was a bid in another strain and a bid in `x` was available between the
previous bid (anyone's) and it: partner went past `x`. That is how a ladder
(control bids, stoppers, up the line) denies something, and only suits that
could have been bid are denied: over our 3H, partner's 3S bypasses nothing,
4D bypasses spades and clubs. A second argument, a call, is where the
ladder starts when it starts higher than the previous bid:
`partner.bypassed(C, 3S)` for control bids that begin above 3S.

**Jumps.** `partner.jumped` (and `me.jumped`, `lho.`, `rho.`) is true when
that seat's last call was a bid at least one level above the cheapest bid
in the same strain: 1♦ (1♥) X (P) 2♠ is a jump, 1♦ (1♠) X (P) 2♥ is not.

**Control-bid dialogue.** A ladder call says so with `sets ladder=control`;
the engine then records, for the caller, the suits the call skipped
(`partner.denied(x)`, `me.denied(x)`) and the suit it names
(`partner.cued(x)`, `me.cued(x)`). Control bids start only once a suit is
agreed, so an auction holds one dialogue. For writing the ladder:
`cheapest(x)` is the non-jump bid in `x`; `under_game(x)` says whether it
is still below game in the agreed suit; `cheapest_rank(x)` is its place in
the bidding order (level x 5 + C0 D1 H2 S3), so `prefer 0 - cheapest_rank(x)`
picks the cheapest of several calls.

**Operators:** `!` (not), `|` (or), `,` (and), `a..b` ranges, `in 1|4` sets,
and arithmetic on numbers and `.min` / `.max`.

**Suits.** A suit in a comparison is its length (`S>=H`, `M>=4`, `x<=1`).
`is` asks what something *is*: `we.trump is suit` (or `notrump`, `none`), and
`x is M` / `x is not M` for "the same suit" or "a different suit", and
`style is relay` for a card option (section 3). Precedence, tightest first:
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

This section first proposed judgment in Rust: named evaluators registered
by the engine and used like terms (`when slam_try`). Rick decided
otherwise (2026-09-30, docs/JUDGMENT-LAYER.md): **the engine measures,
the rules judge**. Combined ranges, fits and holdings are terms; the
thresholds (25 for game, 33 for slam, no two bare suits, the vulnerability
hedge) are written in the rules as named conditions (`define`, section 3),
where they can be read, cited and tuned. `slam_try` and `grand_try` remain
as placeholders (combined HCP 31 and 35) until the judgment layer replaces
them.

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
- A rule's `when` rules it out if its public parts are not known true: a
  conjunction fails when any public part does, a disjunction when every
  branch does. (`when partner.M>=4` cannot be what the caller relied on
  before partner has shown four; `when style is bba, C>=4` is ruled out
  under another style.) Parts about the caller's own hand (`slam_try`,
  `shape 4333`, `we.keycards(t)`) are not judged and stay possible.
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

  The summary keeps HCP, the four suit lengths, balance, total points
  (`points`, `suit_points`), support points per trump suit (`tp(x)`) and
  declarer points (`hcp + length_points`, written together). The point
  counts are tied to the HCP both ways through what the lengths allow
  (since 2026-10-03): length points lie between each suit's sure excess
  over four and what the most lopsided hand the lengths and the deck
  allow holds (a suit known 6–8, the others at most four, gives 2–4), and
  shortness likewise, capped by the trumps. So "12+ total points" with
  six hearts and nothing else beyond four is 10+ HCP; a known HCP floor
  raises the points; "at most 16" in total points rules out a "17+ HCP"
  branch of an earlier disjunction. Each seat's lengths are also capped
  by the deck: 13 less what the other three have shown.

  The narrowing is **sound**: a hand the calls allow is never excluded
  (tested on random hands, `eval.rs`, `knowledge_soundness`). A denial
  whose terms cannot all be judged for another hand (`has(A,x)`,
  `stop(x)`) keeps only its sound part: an unjudgeable term counts as
  false inside the negation, so "not (A or B)" with B unknown still
  denies A, and "not (A and B)" denies nothing.
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
  end with an opponent's call: the one just before my turn. Several
  alternatives may be written on one `after` line, separated by `|`; each is
  checked on its own and any match is enough.
- `shows`, `when` and `denies` may be repeated, on the rule line or on
  continuation lines; the repeats are combined with AND. `prefer`,
  `priority`, `replaces`, `as` and `alert`/`announce` may appear once.
- A `skill` path is lower-case letters, digits and `_` on both sides of
  one `/` (`precision/1c_opener`); `/` appears nowhere else in the
  language. `rbb bid check` warns about a path that the rules'
  is not a standard convention or skill (convention-card's `spec/conventions/`).
- `card` and `param` paths must exist in the rules' card vocabulary
  (convention-card's `spec/fields.toml`); an old alias is reported with the
  current name.
- `rbb bid check` also checks across files: module names are unique, and a
  `needs` that names no module is a warning.
- Every name in a condition must be one the engine knows (a term such as
  `hcp`, `partner.jumped`, `we.forcing`, a function such as `stop(x)`, a
  suit, a variable or a module `param`). The engine checks this whenever
  it loads the rules (`rbb bid check`, `rbb call`, `rbb compare`, the
  workbench, `cargo test`) and refuses to load on an unknown name. Before
  this check an unknown name made its condition false every time, so a
  typo, or a rule newer than the running engine, silently switched the
  rule off.
- A context's `when` must not depend on the chooser's hand (`hcp`,
  `points`, `stop(x)`, `shape …`, a suit or suit variable compared as a
  length, `me.hcp`, `we.hcp`): contexts are checked before the hand is
  known, so such a condition is never true and the rules under it never
  fire. The same check reports it; put it in the rules' `when` or
  `shows`. (`x is not C`, `me.last`, `partner.…` are public and fine.)

## 12. The engine today

How `rbb-engine` implements the model, and its current limits:

- **Suit comparisons** are about length: `S>=H` means at least as many
  spades as hearts, and `x=M` means the same length. To ask whether two
  names are the *same suit*, write `x is M` or `x is not M`.
- A `when` that is false whatever the hand (such as `x is not M` with x bound
  to M) removes the candidate before ranking. It does not appear in the
  trace and is never used for negative inference.
- **Knowledge** holds ranges for HCP and each suit length, and whether the
  hand is balanced. The deck has 40 HCP: after every call each seat's
  maximum is capped at 40 minus the other three's minimums, and while
  choosing a call a rule reading another seat also uses the chooser's
  own HCP. Either cap re-narrows that seat's earlier disjunctions, so a
  takeout double collapses to its shape once the doubler cannot hold
  the 17 HCP of the power branch. The deck constraint applies: lengths sum to 13, and a
  balanced hand has 2 to 5 cards in every suit. All other terms (`has`,
  `keycards`, `shape`, `quality`) are kept as constraints but do not narrow
  the ranges. Partner's keycards are not tracked yet, so
  `we.keycards(t)` is only known from my own count.
- **Descriptiveness** is measured on a fixed sample of 20,000 random hands.
  It is the share of hands that the call's `shows` rules out, among the
  hands consistent with what the caller has already shown. It depends on
  the calls and on the board's conditions (the vulnerability seen from the
  caller's side, the scoring: earlier calls may mean different things
  under them), and is computed once for each.
- **Negative inference** is automatic. A call denies every candidate that
  outranks it on priority and descriptiveness. It is skipped when the
  denied rule has a term that cannot be resolved (for example `slam_try` or
  `has(Q,t)` about another player's hand), because negating a partial
  condition would claim more than is known.
- **Forcing.** After `sets forcing=round`, partner may not pass at their next
  turn if RHO passes. If RHO bids, doubles or redoubles, the round force
  ends: partner may pass, and the pass says he has nothing to add. After
  `sets forcing=game`, neither partner may pass below game, whatever the
  opponents do. The compare tool's "passed a forcing auction" check uses
  the same rule (`Position::must_bid`). Takeout, negative, responsive and
  reopening doubles do not set `forcing=round`. Their rule blocks leave
  out the pass when RHO passes, so they are forcing then, and offer a pass
  once RHO bids. They can still offer an explicit penalty pass, which a
  round force would remove.
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
   they can be precomputed per rule and context. Today: the fixed sample
   (20,000 hands), one pass per candidate at each new position, cached
   per auction prefix. Options: exact fractions for rules that constrain
   only HCP and lengths (the common case); precomputing per rule and
   context; a smaller sample, at the cost of arbitrary order between
   rules closer than the sampling noise. Rick, 2026-09-28: decide once
   more conventions are built, for a richer basis.
3. **How much of "natural bidding" is rules vs. built in.** Proposal: a
   `natural.bid` module for everything expressible, plus Rust judgment hooks.
4. **Competitive defaults.** For example, "after interference, systems on
   over a double, off over a suit bid". Card-driven state rules, or explicit
   patterns in each module?
