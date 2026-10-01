# A judgment layer: top-down rules

Status: **proposal, 2026-09-29; Phase 0 built 2026-09-30** (see "Phase 0:
what was built" at the end). Rick approved writing it up; his decisions
are listed at the end.

Rick's framing: "we have a lot [of .bid files] that are bottom up, where
we start with opening bids and manage sequences from there, and others
that are top down, that recognize situations and decide part score vs.
game vs. slam, how far to compete, penalty doubles, etc."

The rule set is almost all bottom-up. The few top-down rules we have are
among the most valuable per line, and the biggest losses left (slams,
partscore competition) are the decisions a top-down rule makes. This
note proposes a **judgment layer**: low-priority rules keyed on the
situation (state), not the auction, that decide placement, competition
and slam entry once the sequence rules have nothing more specific to say.
The catch-alls we already have work this way, and the layer would
generalise them.

## 1. Where we are

### What the rules are and what they do

Full corpus, 2026-09-29: 170,633 boards, 1,554,148 of our calls. Each
context is classed as *bottom-up* (`after <pattern>`), *Q&A* (`when
asked/answered X`), *top-down* (any other `when <state>`), or
opening/seat. The defending-side fallback is counted on its own.

| kind | contexts | rules | non-pass calls made | passes |
|---|---:|---:|---:|---:|
| bottom-up (`after …`) | 446 | ~1,577 (75%) | 58.8% | 214,218 |
| openings / seat | 10 | ~49 | 26.2% | (in the openings) |
| Q&A (`asked` / `answered`) | 92 | ~438 (21%) | 11.3% | 39,121 |
| top-down (`when <state>`) | 14 | ~48 (2%) | **3.8% (24,395)** | 132,084 |
| defending-side fallback | 1 | 1 | – | **497,933** |

Non-pass calls are bids and doubles, 646,654 in all. Of the top-down
passes, 82,249 are base.bid's "Game reached: nothing more to say" and
47,063 are after-interference's "Passed last time / Partner passed:
nothing new to say".

About 2% of the rules make about 4% of the bids. The one defending-side
fallback makes a third of all our calls.

### The top-down rules we have

| file | context | what it decides |
|---|---|---|
| `base/base.bid:78` | `when game_reached` | pass once our side has bid game (priority −50) |
| `base/base.bid:84` | `when we.forcing = game, !game_reached` | game in the agreed suit or 3NT (−40); game in a shown eight-card fit (−38/−41) |
| `base/responder-rebids.bid:699` | after partner's 1NT/2NT rebid or opening, no trump agreed | the notrump ladder: captain bids game with 25 between us, invites over 1NT, prefers a known eight-card major fit (−8/−9) |
| `competitive/total-tricks.bid:26` | `when they.bid`, they are still bidding, `!game_reached` | the LoTT: 8/9/10 trumps → the 2/3/4 level, hedged at unfavourable (−4) |
| `slam/slam-catch.bid:26` | `we.trump is suit, we.forcing = game, !asked` | game in the agreed major when nothing else applies (−20) |
| `slam/rkcb-1430.bid:68,76`, `slam/blackwood.bid:64,72` | `we.trump is suit, !asked`, by `we.forcing` | the keycard / ace ask: `we.tp(trump)` 33 against partner's maximum (limited) or floor (forced), no two bare side suits |
| `slam/control-bids.bid:60,94` | `we.trump is suit`, a major, by `we.forcing` | start the control-bid ladder with slam values and a suit unprotected |
| `slam/over-3nt.bid:18` | `partner.last=3N, !they.bid, we.trump is none` | 6NT / quantitative 4NT / minor slam try / back to a standalone major |
| `competitive/after-interference.bid:469,473` | they bid, we opened, I or partner passed last | pass: nothing new to say (−20) |
| `competitive/advances.bid:267` | `lho.opened \| rho.opened` | pass: "Defending: nothing more to say" (−20) |

The most-used top-down rules (calls made on the corpus):

| rule | calls |
|---|---:|
| base.bid "Game in notrump" | 3,470 |
| total-tricks.bid, LoTT at the 4 / 3 / 2 level | 3,241 / 3,009 / 654 |
| responder-rebids ladder "No game: under 25" / "Game: 25+" | 2,772 / 734 |
| rkcb-1430 keycard ask, ♠ / ♥ | 2,059 / 1,819 |
| slam-catch "Game in ♠ / ♥" | 1,822 / 1,658 |
| over-3nt 6NT / quantitative 4NT | 925 / 416 |
| base.bid game in the ♥ / ♠ fit | 651 / 423 |
| blackwood / control bids | 415 / 359 |

### What they were worth when added

| change | corpus par | side IMPs |
|---|---:|---:|
| game in a known fit (base.bid, ticket b455) | +1,392 | +1,698 |
| notrump-ladder captain rule (responder-rebids.notes.md) | +1,231 | +1,203 |
| gating the ladder on a known major fit (NT_Ladder 426, a missed 6♠) | +201 | +285 |
| LoTT, as kept (total-tricks.notes.md) | +2,052 | +12,162 |
| over-3nt thresholds 34/32-33/31 (over-3nt.notes.md) | +2,516 | +5,193 |

### Where the loss is

On 10,000 random deals ([random-deals-comparison.md](random-deals-comparison.md);
21GF pair, `probes/tools/par_blame.py`), boards where the contracts
differ, IMPs from par:

| how the auction missed par | ours | IMPs | BBA | IMPs | gap |
|---|---:|---:|---:|---:|---:|
| W stopped short of slam | 678 | 8,717 | 465 | 5,736 | **−2,981** |
| W outbid, did not compete | 553 | 4,230 | 290 | 2,126 | **−2,104** |
| W let them play (no double) | 702 | 4,598 | 510 | 2,737 | **−1,861** |
| W stopped short (level) | 591 | 3,209 | 493 | 2,667 | −542 |
| W wrong strain | 406 | 3,127 | 412 | 2,817 | −310 |
| W stopped short of game | 805 | 6,318 | 851 | 6,805 | +487 |
| W overbid | 818 | 4,454 | 950 | 4,980 | +526 |
| L did not compete or sacrifice | 282 | 859 | 413 | 1,744 | +885 |
| L overcompeted | 331 | 787 | 487 | 1,507 | +720 |
| W doubled, par was to bid on | 25 | 153 | 117 | 780 | +627 |

- **Slam is the largest gap.** Boards with par 920+ are 13% of random
  deals and half the loss. We reach par on 3.3% of them and BBA on
  12.8%. On the corpus, the keycard ask is missing where BBA asks on
  1,148 boards, for −6,932 IMPs, the largest per-convention figure in
  [conventions-survey.md](conventions-survey.md) §4.
- **Partscore competition is second.** Together, "outbid" and "let them
  play" cost about 4,000 IMPs per 10,000 deals.
- **Game-level placement is at parity.** We stop short of game and
  overbid *less* than BBA does. Strain and level cost a little.
- **BBA's weak spot is doubling** where par was to bid on (117 boards
  against our 25). A penalty-double layer must not buy that problem.

par_blame blames the rule that made the last call of the side that
should have acted. `base.bid` "Game reached" and `advances.bid`
"Defending: nothing more to say" top its list. That tells us where the
auction *ended*, not necessarily where it went wrong: the missing
decision is often a round earlier (no slam try, no overcall). Phase 2
starts by separating the two (§6).

## 2. What the layer decides

Four families. Each one keys on state, sits below the sequence rules,
and hands over to the Q&A layer when its decision is to ask something.

Examples use today's language. Terms in *italics* in the comments are
the proposed ones from §3.

### (a) Placement: part score, game or slam, and the strain

**State keys:** combined strength (`we.points`, or `we.tp(trump)` with a
fit), whether a fit is known (`we.fit(x)`), who is captain, stoppers in
their suits or the unbid ones, `we.forcing`, and `game_reached`.

**Game or not.** The notrump ladder is the model. One player has put a
range on his hand, the other is captain, and 25 between us is game
(counting partner at his minimum), with an invitation when only his
maximum reaches it. The same decision recurs after every limit bid: a
raise, a limit raise, a 1NT response, a minimum rebid, a preference.
Each is written in its own sequence module today, and some positions
are missed ("no rule in a live auction"). Generalised:

```
# conventions/judgment/placement.bid (proposed)
when captain, !asked, !game_reached, !they.bid | lho.last=P   # *captain*
  4M  "Game in the {M} fit: 25+ between us"
      shows we.fit(M).min >= 8, we.points.min >= 25           # *we.fit*, *we.points*
      priority -8
  3N  "Game in notrump: 25+, no major fit, stoppers"
      shows we.points.min >= 25, we.fit(S).min <= 7, we.fit(H).min <= 7
      when  we.stopped_theirs                                 # *new knowledge*
      priority -8
  P   "No game: under 25 between us"
      shows we.points.max <= 24
      priority -9
```

Today the ladder writes this as `points + partner.points.min >= 25,
S + partner.S.min <= 7`. `we.fit` and `we.points` shorten it but add no
knowledge. What does add knowledge is `captain` and the stoppers.

**Strain.**
- A known eight-card major fit is preferred to 3NT (the ladder's
  2026-09-28 gate).
- 3NT needs their suits stopped. Uncontested, it needs the unbid suits
  stopped or asked about, and base.bid's fallback 3NT checks neither.
- A minor fit prefers 3NT to 5m when the stoppers are there, and plays
  5m (or 4m as a try) when they are not. slam-catch.notes.md leaves
  "3NT or five of the minor" unsettled. The sign-off and the control-bid
  ladder cover majors only.
- The standalone major over partner's 3NT (over-3nt.bid, ticket b194) is
  a strain rule of this family that currently lives in a sequence file.

**Slam as a placement.** Placement also bids slam directly when
combined values are clear and nothing needs asking (over-3nt.bid's 6NT
at 34). Once slam is *possible* rather than clear, the decision goes to
family (c).

### (b) Competition: how high, double, or sacrifice

**State keys:** `they.bid`, whether they are still bidding, our fit and
theirs (`we.fit(x)`, *`they.fit(y)`*), the strength split (`we.hcp`,
*`they.hcp`*), vulnerability (*`favourable` / `unfavourable`*), the level
and strain of their last bid (*`they.level`, `they.strain`*), my holding
in their suit, scoring (`imps` / `matchpoints`), and which side opened.

**How high: the LoTT, already there.** `total-tricks.bid` is good work,
and it covers only part of the Law. What it lacks:

1. **Their fit.** It counts our trumps only. The Law is about the
   *total*: with nine trumps facing their eight, the 3-level is right
   only when the total is 17 or more. Knowledge already tracks `lho.x` /
   `rho.x`, so `they.fit(y)` is a convenience.
2. **Whose hand it is.** "Not clearly our hand" is `hcp +
   partner.hcp.min <= 24`. That bounds only our strength, not theirs, and
   there is no branch for "our hand, they are sacrificing" (double) or
   "their hand, we are sacrificing" (bid on, family (b) sacrifice).
3. **The double.** When the total trumps are low and the strength is
   ours, the Law's answer is to double, not bid (Cohen). There is no
   such rule.
4. **The five level.** The Law is capped at four. Five over their four
   is the sacrifice decision.
5. **Vulnerability** is a one-trick hedge at unfavourable. There is no
   push at favourable, and no matchpoint adjustment ("the two level
   belongs to us").
6. **Twelve rules for three decisions.** The vulnerability hedge and
   "count the takeout double as four" are spelled out as text
   (total-tricks.bid is 12 near-identical rules). `we.fit(x)` with the
   doubler's bonus folded in, plus a vulnerability relation, makes
   three.
7. **Silence ends it.** Once they pause, it stops (ticket and notes:
   the unrestricted version scored +1,434 par more but fired in wrong
   spots). The pause case needs its own rule: balancing when they stop
   at the two level with a fit.

Rewritten with the proposed terms:

```
when they.bid, they.still_bidding, !game_reached, !asked     # *they.still_bidding*
  3x  "Total tricks: nine trumps, the three level"
      shows we.fit(x).min >= 9, x >= 3                        # doubler's bonus inside we.fit
      when  !unfavourable | we.fit(x).min >= 10
      when  we.hcp.max <= 24 | they.hcp.min >= 16            # neither side clearly
      sets  ask=signoff
      priority -4
```

**Penalty doubles.** Today they exist only inside sequences (Cappelletti,
their 1NT overcall, nt-interference, vs-preempts, responder-rebids.bid:373).
A situational rule:

```
when they.bid, rho.last is a bid, !asked, we.fit(x).min <= 7 for every x   # misfit
  X   "Penalty: they are too high"
      shows they.strain >= 4, quality(they.strain) >= fair    # length and honours in their suit
      when  we.hcp.min >= 20, they.level >= 2
      when  they.vul | we.hcp.min >= 23
      priority -6
```

(Pseudocode: "for every x" and `they.strain` as a suit are both new; see
§3.) Three things make this family harder than it looks:
- **What a double means.** Most doubles already have a meaning from a
  sequence rule (negative, responsive, takeout, support). A judgment X
  must never claim a call that a higher rule in the same position gives
  another meaning, because partner reads the highest-priority rule for
  the call (§4). The classic teaching hierarchy settles most cases: a
  double is for penalty once partner has made a limit bid or a penalty
  pass, once we have found a fit and they compete, after our redouble,
  or at the 4 level and above over our game. It is a rule-writing
  guideline, and a `bid check` lint could enforce it.
- **Partner's answer.** Sit, or pull with a void in their suit or a
  freak. That is a Q&A pair (`sets ask=penalty`, `when asked penalty`).
- **The guard.** "W doubled, par was to bid on" is 25 boards for us and
  117 for BBA. Every penalty-double change is measured on that line too.

**Sacrifice.** Five over their four when the total trumps are 19-20+, at
favourable or equal, with little defence (*`quick_tricks`*). par_blame
says we already out-judge BBA here (fewer missed and fewer overcompeting
sacrifices), so it is last in the plan.

**Replacing the defending-side pass.** "Defending: nothing more to say"
makes 497,933 passes. Most are right. Where they are wrong:
- The overcaller with a sixth card or a second suit, and the advancer
  with a fit, when they raise. The LoTT fires only if the fit is already
  *known*, and an overcall shows five: an advancer with three makes eight
  and is covered. A 6-card overcall facing a silent partner is not.
- Balancing when they stop low with a fit (2♥-P-P: do we let them play
  2♥?). balancing.bid covers the one-level reopening only.
- The takeout doubler with extras after they raise.

These are judgment rules in the defending side's own band, above the
pass. Which one comes first is decided by the diagnosis in phase 2.

### (c) Slam entry: one decision, several tools

Today the question "is slam worth looking for?" is answered in 13 files
(stayman, two-nt-responses, one-nt, jacoby-transfers, texas-transfers,
smolen, strong-openings, weak-two-responses, splinters, over-3nt,
gerber, rkcb-1430, blackwood). Some of the answers are the same text
repeated: the keycard condition in rkcb-1430.bid and blackwood.bid, and
the bare-suit test, which runs to 300-character lines in control-bids
too. Others use different measures: HCP over 3NT, points after a new
suit, `we.tp` with a fit, the `slam` strength band at 33 where BBA
plays 32 (slam-catch.notes.md, For Rick 7).

The proposal separates **whether** from **how**:

1. **Whether** is one definition, in one module, keyed on state:
   - *slam values*: 33+ combined on partner's floor. That is
     `we.tp(trump)` with a fit, `we.points` or `we.hcp` without one (the
     thresholds that over-3nt.notes.md calibrated on the double-dummy
     tables).
   - *slam interest*: 31+ on partner's maximum, or on my own count with
     a game force and no top to partner's range.
   - *controls*: no two bare side suits in my hand (Rick's rule), and no
     suit bare in both hands (control-bids' stop-in-game rule).
2. **How** stays in the card-gated modules, reading that definition:
   - trump agreed, no bare suit: keycard (rkcb-1430) or Blackwood;
   - trump agreed, one bare suit, cue bids on: control bids;
   - no trump, partner balanced and limited: quantitative 4NT, or 6NT
     with slam values;
   - partner's natural NT opening, Gerber on: 4♣;
   - a fit known but not agreed (2♣–2NT–3♠ with three spades, slam-catch
     Gaps): agree it on the way (`sets trump=x, ask=keycards(x)`), which
     the catch cannot do today.

Why the call rules stay per module: two rules for the same call at the
same priority, told apart only by a card `param`, are *read* wrongly.
Partner takes the first, and the answering module is inactive. That
cost 85 passed-out 4NT contracts (slam-catch.notes.md). So the sharing
has to happen in the *condition*, not the call. That needs named
conditions (§3) or an engine hook.

A caution from over-3nt.notes.md: the first attempt at one context "for
every uncontested position below game" overrode the notrump modules' own
slam counting and cost −11,422 par. The unified entry has to sit
**below** every module that already counts slam for its own auction,
and it replaces only the duplicated conditions, not those modules.

Target positions: the 1,148 corpus boards where BBA asks for keycards
and we bid game, and the "stopped short of slam" boards that pass via
base.bid's "Game reached" rule.

### (d) Grand-slam judgment

`grand_try` is a placeholder: combined HCP ≥ 35 (LANGUAGE.md §12,
`crates/engine/src/eval.rs`). Its only use is the keycard answerer's
correction to seven (`rkcb-1430.bid:135`). There is no king ask (the
5NT field exists, a TODO in rkcb-1430), no grand slam force and no
trick count. Grand-slam scenarios are three of the four worst per board
on the corpus (about −14,100 IMPs; conventions-survey.md §4).

What a grand decision needs, in order of value:
- all five keycards and the trump queen (`we.keycards(t)=5`, the queen
  answer: tracked);
- kings, via the 5NT king ask (card field `slam.king_ask.five_nt`,
  field only today; 1,387 BBA boards);
- a trick source: a long side suit or a crossruff, meaning shortness
  facing length (`partner.x.max <= 1` from a splinter);
- combined strength well above slam: 37+ with a fit, as a start, to be
  calibrated from the double-dummy tables like over-3nt's small-slam
  table, and at IMPs only with odds of about two to one.

## 3. Terms the language needs

Checked against `rbb bid terms` (docs/CONTRACT.md, Term reference). What
exists and matters here:
- `we.hcp`: my HCP plus partner's shown range, hand-dependent. No rule
  uses it yet. Rules write `hcp + partner.hcp.min` (44 places in 6 files
  count HCP or points this way).
- `we.tp(x)`: my support points plus partner's recorded ones. Now in
  `hand_dependent`, so slam-catch.notes.md For Rick 1 is fixed.
- `we.keycards(x)`, `we.trump`, `we.forcing`, `we.gf`.
- `partner.points`, `partner.tp(x)`, and the lengths of every seat
  (`lho.H` is used 143 times).
- `vul`, `they.vul`, `imps`, `matchpoints`, `game_reached`,
  `cued`/`denied`.
- The strength bands (`strength=game`, 25 / 33 combined).
- `slam_try`, `grand_try`: placeholders.

Proposed. *Convenience* means it can be computed from what knowledge
already holds, so it only makes rules shorter and consistent. *New*
means the engine must track or define something it does not have today.

| term | meaning | kind | replaces / used by |
|---|---|---|---|
| `we.points` | my points + partner's points range | convenience (the `hand_dependent` list already names it; `we_attr` does not, so it errors) | ladder, over-3nt, placement |
| `we.fit(x)` | my length in x + partner's shown range; `.min` is the known fit | convenience | `x + partner.x.min` (24 places), LoTT, ladder, base.bid fit games |
| `they.hcp`, `they.fit(x)` | LHO + RHO ranges | convenience | LoTT total, penalty doubles |
| `they.level`, `they.strain` | level and strain of the opponents' last bid; strain usable as a suit | convenience (from the auction) | penalty doubles, sacrifice |
| `they.still_bidding`, `they.game_reached` | one of their last calls is not a pass; their contract is game or higher | convenience | LoTT context, sacrifice |
| `favourable`, `unfavourable` | vulnerability relations | convenience | LoTT, sacrifice, penalty doubles |
| `bare(x)`, `bare_suits` | my 2+ cards in a side suit with neither A nor K; how many | convenience (my hand) | rkcb-1430, blackwood, control-bids (the 300-character lines) |
| `quick_tricks` | A-K 2, A-Q 1½, A 1, K-Q 1, K-x ½ | new hand term (easy) | penalty doubles, sacrifice |
| `safe_level(x)` | `we.fit(x).min − 6`, the LoTT level | convenience | competition |
| `partner.stop(x)`, `we.stopped(x)`, `we.stopped_theirs` | partner has shown a stopper (a 3NT over their suit, a stopper-ladder bid); a stopper in either hand; every suit they have shown is stopped | **new knowledge**: today `partner.stop(x)` is "not tracked" and `ladder=stopper` is recorded as `cued` | 3NT placement, minor games |
| `captain` | I know partner's range well enough to place the contract, and partner does not know mine | **new**: needs a notion of a limited call (a range narrow enough, or a `limit` attribute on rules) | placement, slam entry |
| `we.tp(x)` for the hand with fewer trumps only | Rick's short-suit rule (slam-catch For Rick 4) | **new**: compares both hands' trump lengths | slam entry |
| named conditions (`define slam_values = …`) | a module-level named condition other modules can use | **new grammar**, additive | slam entry shared by rkcb / blackwood / control bids / Gerber |
| `slam_try`, `grand_try` as real evaluators | slam / grand worth trying | **changed meaning** of existing terms (see §6, contract) | slam entry, grand |

Two engine behaviours matter as much as the terms:
- **`we.*` inside `shows`.** A judgment call's `shows` must resolve to a
  condition on my own hand, as `points + partner.points.min >= 25` does
  today (fixed when the call is made, LANGUAGE.md §6). Otherwise partner
  reads nothing from it. This needs a check for each new `we.` term.
- **Comparing ranges.** `we.fit(S) <= 7` means "known at most seven", so
  it is false while partner's length is open. "No known fit" is
  `we.fit(S).min <= 7`. The ladder's gate reads that way today, and the
  terms keep LANGUAGE.md §10's known-vs-possible rule.

**Rules or Rust for the thresholds?** LANGUAGE.md §7 suggests judgment
hooks in Rust. I recommend the other split: **the engine measures, the
rules judge.** Combined ranges, fits and stoppers are engine terms. The
thresholds (25, 33, "no two bare suits", the vulnerability hedge) are
bridge judgment and belong in the convention repo when it splits
(CLAUDE.md), where they can be read, cited and tuned. Named conditions
make that practical. Rust hooks would put the judgment in the engine
repo.

## 4. How it fits the engine

**Priority bands.** The ranking goes by priority first, so a band says
where each layer sits. What the priorities are today, and a proposal:

| band | layer | today |
|---|---|---|
| +1 … +20 | overrides inside a dialogue | control-bid stop-in-game (20), keycard in a control dialogue (11-15), keycard over a limit raise (1) |
| 0 … −3 | sequence rules and Q&A | the default |
| −4 … −15 | **judgment**: competition, placement, slam entry | LoTT −4, ladder −8/−9, over-3nt −10; sequence-level "minimum, no game" passes −5 … −10 are mixed in |
| −16 … −45 | keep-alive sign-offs in a force | slam-catch 4{trump} −20, base.bid fit games −38 … −41, game −40 |
| ≤ −50 | silence ("nothing more to say") | base.bid game reached −50; after-interference and the defending side at −20 |

Two things follow:
- **Silence passes move to −50.** The defending-side and
  after-interference passes at −20 share a number with slam-catch's
  sign-off. They cannot meet in the same position today, but any
  judgment rule written below −20 would lose to them. Moving them
  changes nothing now. It is still a rule change, so it gets measured.
- **Judged passes vs silent passes.** Sequence modules end with passes
  at −5 … −10, and they come in two kinds. "Minimum: no game" is a
  judgment and should outrank the layer. "Nothing more to say" is
  silence and should sit below it. Today the LoTT at −4 outranks both.
  That was right for the jump case in after-interference.bid, but it
  happened by accident. Each module's pass should say which kind it is.

The keycard ask over a limit raise sits at +1, *above* the sequence
rules, while the game-forcing ask sits at 0/−1. Unification puts both in
one place.

**Negative inference.** It is automatic: a call denies every candidate
that outranks it. For judgment rules this is mostly what we want:
- a judgment pass ("No game: under 25") denies the game hand. That is
  the ladder's point;
- a silence pass denies every judgment call above it: no LoTT raise, no
  penalty double. That is also right, and it makes the silence pass
  informative;
- a judgment call denies all the sequence rules above it. Descriptive
  calls say more, and a judgment call says "none of those fitted".

It goes wrong in two cases:
- a judgment rule that ties on priority with another and is separated by
  `prefer` loses its inference (LANGUAGE.md §8). So judgment rules
  should use priority and descriptiveness, not `prefer`;
- the reading problem below.

**Reading partner's judgment call.** The engine reads a call as the
*highest-priority* rules for that call whose contexts hold
(`advance_from` in `crates/engine/src/engine.rs`). Lower-priority rules
"do not widen the meaning". So if partner bids a judgment 3NT at −8 in a
position where a sequence rule also offers 3NT at 0, I read the
sequence 3NT, even though partner bid it because the sequence rule did
*not* fit his hand. Today this happens rarely, because the catch-alls
sit where nothing else bids the same call. A broad judgment layer will
make it common. The options:
1. **Rule discipline plus a lint.** A judgment rule may not claim a call
   that a higher rule offers in the same position. `compare` can count
   the positions where that happens ("judgment call read as …").
   Cheap, and it can come first. `compare` now prints it: "calls read
   as a higher rule than chose them", with the top (chosen rule ->
   read-as rule, call) pairs, and `read_as` in `--json`.
2. **Engine: read a fallback as the union** of the top rule and each
   lower rule for the same call, each branch carrying the denial of what
   outranks it. This is correct in principle. It changes how existing
   files read, so it is a language version bump (§6).

**Descriptiveness** within the band works as it does for everything
else. A judgment call's `shows`, once resolved, is a range on my hand
(`points >= 25 − p.min`), so 4♥ "fit and 25" outranks 3NT "25 and no
fit" without a priority number, as the ladder already does.

**Forcing.** Judgment rules never pass a force (the engine removes a
forbidden pass). Placement calls end the auction's exploration: they
`sets ask=signoff` as the LoTT does, so partner answers with the
existing `when asked signoff` rules (pass, or bid on with extras).
Judgment calls create a force only when they ask something (keycard,
quantitative, a slam try).

**The Q&A layer.** Judgment never answers a question: every context
carries `!asked`, as slam-catch, the ladder and the ask rules already
do. Partner's pending question is answered by its protocol. After an
answer, the asker is the natural captain (`answered transfer(M)`,
`answered stayman`), and placement is the asker's next call. Judgment
calls that ask (4NT keycard, quantitative 4NT, a slam try) hand over to
the Q&A layer through `sets ask=…`. So the judgment layer routes between
the bottom-up descriptions and the protocols. It never replaces either.

## 5. How to measure it

- **Both yardsticks, every change** (CLAUDE.md, overcalls.notes.md "For
  Rick: which yardstick"): distance from par, and
  `probes/tools/sideimps.py BASE.json VARIANT.json` (IMPs to the side
  that made the first differing call). Act where they agree. Until
  penalty doubles exist, side IMPs flatter competitive bidding, because
  our opponents (ourselves) rarely punish a light call.
- **Random deals as well as the corpus.** The scenario corpus is dealt
  for conventions and has twice the slam share (21% against 13%). Random
  deals show the natural core. Every family is judged on
  `Random_Pavlicek_21GF` (docs/random-deals-comparison.md, "Reproduce")
  as well as the corpus. For slam thresholds, use a 100,000-deal sample
  (standard error about ±0.013 a board). 10,000 deals hold only about
  1,300 slam boards.
- **par_blame categories are the per-family targets.** Placement: "stopped
  short (level)" and "wrong strain". Competition: "outbid, did not
  compete" and "let them play". Penalty doubles: "let them play", with
  "W doubled, par was to bid on" as the guard. Slam: "stopped short of
  slam". Sacrifice: "L erred: did not compete or sacrifice", with "L
  overcompeted" as the guard.
- **Overfitting.** over-3nt.notes.md is the warning: after a new-suit
  rebid, every lower 6NT threshold scored better on the corpus (31/29:
  +1,624 par), but at 31 the rule fired on 29-31 real HCP, where 6NT
  makes 17-69%. The textbook 33 was kept. For judgment thresholds:
  - calibrate from the double-dummy tables (P(make) by combined strength
    and fit, as over-3nt's table does), not by searching the corpus for
    the best number;
  - tune on one set, confirm on another: corpus and random deals, or
    two disjoint random samples (`rpdd_sample.py --offset`);
  - report the real combined HCP on the boards where a rule fires. A
    threshold that fires well below its stated number is being carried
    by the deal selection;
  - prefer the textbook number unless both sets agree it is wrong.
- **Coverage.** "No rule in a live auction" and "passed a forcing
  auction" stay at zero. The top-down share of calls (3.8% today) is a
  progress indicator, not a goal.

## 6. A phased plan

Smallest high-value first. Each phase is its own commits, with `.test`
cases, a `.notes.md` with Sources, and both yardsticks.

| phase | what | target (evidence) | contract |
|---|---|---|---|
| **0. Groundwork** | `we.points`, `we.fit(x)`, `they.fit`/`they.hcp`, `they.level`/`they.strain`, `favourable`/`unfavourable`, `bare(x)`; named conditions; the priority bands in CONTRACT Part 2; the "judgment call read as" count in `compare`. Rewrite the LoTT, the ladder and the ask conditions with them. | **No call changes**: identical compare output is the test. total-tricks.bid from 12 rules to 3; the 300-character lines gone | new terms: eval.rs term tables, `rbb bid terms --doc docs/CONTRACT.md`, manifest `engine` bump. Named conditions are additive grammar (like `skill`): no language bump |
| **1. Slam entry** | one slam-values / slam-interest definition (`slam/slam-entry.bid` or `judgment/`), read by rkcb-1430, blackwood, control-bids, gerber and the quantitative rules; fit known but not agreed; minor fits | random: "stopped short of slam" 678 bd / 8,717 IMPs toward BBA's 465 / 5,736 (gap 2,981 per 10k deals, about 0.30 a board); corpus: the 1,148 missing keycard asks (−6,932) | none beyond phase 0, unless `slam_try` itself becomes the definition (below) |
| **2. The defending side's pass** | first a diagnosis: of the boards par_blame pins on "Defending: nothing more to say", how many went wrong earlier (no overcall, no balance) and how many at the pass; then the missing judgment: rebidding a six-card overcall, balancing when they stop low with a fit, the doubler with extras, the LoTT with their fit counted | random: "outbid, did not compete" 553 / 4,230 against BBA's 290 / 2,126; part of "let them play" 702 / 4,598 against 510 / 2,737. Guards: "L overcompeted" (331) and "W overbid" (818) must not rise | terms only |
| **3. Penalty doubles** | the too-high double (misfit, trumps, balance of strength, low total trumps), partner's sit or pull as a Q&A pair, the "when is a double for penalty" hierarchy as a rule-writing guideline and a lint | random: the rest of "let them play" (gap 1,861 with phase 2). Guard: "W doubled, par was to bid on" stays near 25. Then re-run phase 2 and the open overcall questions (overcalls.notes.md), since side IMPs become fair | `quick_tricks`; terms only |
| **4. Grand slams** | a real grand decision; the 5NT king ask (`slam.king_ask.five_nt`, field only); calibration from double-dummy tables | corpus grand-slam scenarios (about −14,100 IMPs); random 920+ band "at par" 3.3% toward BBA's 12.8% | `grand_try` changes meaning: a **language version bump** (it is read by rkcb-1430.bid:135), or a new term and the placeholder retired |
| **5. Placement consolidation** | `captain`; game/no-game after any limit bid; stoppers; 3NT vs 5m; fold base.bid's fallbacks and sequence copies into the layer | "stopped short (level)" 591 / 3,209 against 493 / 2,667; "wrong strain" 406 / 3,127. Game shortfall and overbidding are already at parity, so this phase is for coverage and simplicity more than IMPs | `captain`, stopper knowledge: new engine knowledge (terms only for the contract) |
| **6. Sacrifice** | five over their four by total trumps, vulnerability, defence | small: we already beat BBA on both sacrifice lines | none |

Contract notes that apply throughout:
- A new term is an addition no existing file can notice. It needs no
  language version, but the manifest's `engine` must name the engine
  that has it (docs/CONTRACT.md, Stability), and `rbb bid terms --doc
  docs/CONTRACT.md` must be re-run (a test checks it).
- Changing what an existing term means (`slam_try`, `grand_try`,
  `we.tp` counting shortness for one hand only), or how a call is read
  (the fallback union), makes existing files read differently. That
  needs a **language version bump** and a manifest that asks for it.
- Moving rules between priority bands is a rule change: measure it, no
  version.
- New modules get `skill` lines (`competitive_bidding/law_of_total_tricks`,
  `partnership_bidding/slam_bidding`; a new path under `[proposed]` in
  `card/skills.toml` for penalty doubles if Bridge-Classroom has none)
  and `rbb bid skills --doc docs/SKILLS.md`.

## 7. Open questions for Rick

1. **Where the judgment lives.** Thresholds as named conditions in the
   rules (my recommendation: the engine measures, the rules judge), or
   Rust judgment hooks as LANGUAGE.md §7 has it?
2. **Captain.** Derived from how narrow partner's range is (say ≤ 3
   points), or marked on the rules that limit a hand (a `limit`
   attribute)? And after a question has been answered, is the asker
   always captain?
3. **Reading judgment calls.** Rule discipline plus a lint, or the
   engine change (read a fallback as the union with its denials), which
   is language 2?
4. **Order of phases 2 and 3.** Competition first (the larger gap), or
   penalty doubles first so that side IMPs judge competitive calls
   fairly?
5. **Priority bands.** Adopt them, move the silence passes to −50, and
   have each sequence module's closing pass say whether it is judgment
   or silence?
6. **Penalty-double style.** Which "double is for penalty when…"
   hierarchy do you teach? That becomes the guideline and the lint.
7. **Slam thresholds and scoring.** The corpus and the random sample
   are matchpoint-scored, while our yardsticks are IMPs. Should the
   slam and grand thresholds differ by scoring (`imps` / `matchpoints`)
   from the start? And is the textbook 33 the default wherever the
   calibration and the corpus disagree?
8. **A fit known but not agreed.** Should the slam layer treat a known
   eight-card fit as agreed (2♣–2NT–3♠ with three spades), or should the
   sequence modules always raise explicitly?
9. **`we.tp` for the hand with fewer trumps** (slam-catch.notes.md For
   Rick 4). Worth the engine change and the language bump, or are the
   current counts good enough for entry?
10. **Where the files go.** A new `conventions/judgment/` family, which
    makes the top-down layer visible, or keep each family with its
    subject (`slam/`, `competitive/`)?

## Sources

- **Rick's framing** (2026-09-29, quoted above) and his earlier rulings
  that these rules already follow: the common catch once a trump suit is
  agreed and the 33-point slam test (slam-catch.notes.md, 2026-09-23);
  the notrump ladder and captain (responder-rebids.notes.md,
  2026-09-27/28); the LoTT guidance (total-tricks.notes.md, 2026-09-27).
- **Measurements:** the corpus classification of rules and calls
  (2026-09-29, controller run on the full corpus); the random-deal
  comparison and par_blame tables
  ([random-deals-comparison.md](random-deals-comparison.md)); the
  keycard and grand-slam figures
  ([conventions-survey.md](conventions-survey.md) §3-4); the per-change
  figures in the module notes cited in §1.
- **Bridge sources:** the Law of Total Tricks and its use for
  competitive doubles: Larry Cohen, *To Bid or Not to Bid: The Law of
  Total Tricks* (1992). Combined-point targets (25 for game, 33 for a
  small slam) and the "when is a double for penalty" conventions:
  standard practice, not yet cited. The quick-trick table is the
  standard (Culbertson) scale.
- **Engine facts:** `rbb bid terms` (2026-09-29) and
  `crates/engine/src/eval.rs` (`we_attr`, `hand_dependent`, the
  placeholder hooks), `crates/engine/src/engine.rs` (`advance_from`: how
  a call is read).

## Decisions (Rick, 2026-09-30) and where to start

Answers to §7's first five questions:

1. **Thresholds live in the rules**, as named conditions, not Rust hooks.
2. **Captain:** when a player has limited his range to 3-4 points, his
   partner usually becomes captain. The notrump ladder always does this;
   a limit bid often does; opener's rebid usually does, but opener's
   non-jump new suit (normally 12-18) does not limit him yet. Define
   `captain` from the width of each partner's shown range (about 4
   points or less), not from a per-call attribute; the asker is captain
   after an answer to his question.
3. **Reading judgment calls:** a lint and rule discipline for now; no
   language-2 engine change.
4. **Order:** penalty doubles (Phase 3) before the defending-side pass
   (Phase 2), so the side-IMP yardstick is fair when the pass work is
   measured.
5. **Scoring:** slam and grand thresholds may differ by scoring (IMPs vs
   matchpoints); when the evidence conflicts, textbook 33 is the default.

Questions 6-10 stay open.

**Start here (a new session):** Phase 0 exactly as §6 describes: the
convenience terms (`we.fit(x)`, `we.points` fixed or dropped, the
opponents' fit and level, favourable/unfavourable, `safe_level`,
`quick_tricks`), named conditions (`define`), `captain` per decision 2,
the priority bands, and the lint for decision 3; compare output must be
unchanged (byte-identical boards) and `rbb bid terms --doc
docs/CONTRACT.md` regenerated. Then Phase 1 (slam entry), then Phase 3
(penalty doubles), then Phase 2. Work pattern: CLAUDE.md, the
controller/agent memory, compare JSONs in the SSD scratchpad (the X10
drive is corrupt), both yardsticks, random deals alongside the corpus.

## Phase 0: what was built (2026-09-30)

Engine 0.3.0, rule language 1 unchanged. **Our calls did not change**:
the full corpus (170,633 boards) gives identical auctions and identical
replays of BBA's; only 577 LoTT explanations changed wording.

- **Terms** (docs/CONTRACT.md, Term reference): `we.fit(x)`,
  `we.points`, and `we.hcp` / `we.tp(x)` as *sums written out*: the
  engine rewrites `we.fit(S).min` into `S + partner.S.min` before a rule
  is used, so a `shows` still resolves to a condition on my own hand;
  `safe_level(x)`; `they.hcp`, `they.fit(x)`, `they.level`,
  `they.strain`, `they.still_bidding`, `they.game_reached`;
  `favourable`, `unfavourable`; `captain` (decision 2: partner's shown
  points span 4 or fewer and mine more, or partner answered my
  question); `quick_tricks`, `bare(x)`, `bare_suits`; `partner.trump`.
  Not built: `we.stopped(x)` and stopper knowledge (Phase 5), `we.tp` for
  the hand with fewer trumps (question 9).
- **Named conditions**: `define name(x) = condition` (LANGUAGE.md §3),
  shared across the rule set, reading their own module's card
  parameters, inlined by the engine (`crates/engine/src/macros.rs`), and
  checked at load (duplicates, clashes with terms, arity, cycles). A
  condition counts 1 or 0 in arithmetic.
- **Rewrites**: total-tricks.bid from 12 rules to 3 (`lott_trumps(x) =
  we.fit(x).min + doubler_four(x) - unfavourable`); the notrump ladder
  with `we.points` / `we.fit`; the keycard, Blackwood and control-bid
  conditions as named conditions in the new `slam/slam-entry.bid` (every
  line over 150 characters is gone). The other 24 hand-written
  `x + partner.x.min` sums are left for when their modules are next
  touched.
- **Priority bands** are in docs/CONTRACT.md Part 2, with the rule that a
  judgment call must not be read as another rule. **The silence passes
  were not moved**: at −50 they tie with base.bid's "Game reached" pass,
  which then reads 6,942 of them, and 20 calls off our own auctions
  change. §4's "moving them changes nothing now" is wrong; the move is a
  measured rule change for a later phase.
- **The lint** (decision 3): `compare` counts our own calls that partner
  reads as a higher-priority rule than the one that chose them, with the
  top pairs (`read_as` in `--json`).
- **A bug found on the way**: descriptiveness was cached by rule and calls
  only, so a `shows` that reads the vulnerability took the value of
  whichever board filled the cache first (boards are bid in parallel).
  The merged LoTT reads `unfavourable`, and six replay calls flipped
  between runs. The key now carries the values of such terms, and a
  `shows` that counts conditions has its publicly settled parts folded
  once instead of per sample hand (compare time within a few percent of
  before).

