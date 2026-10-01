# rebids (`rebids.bid`): notes

Opener's rebid after a one-level suit opening and an uncontested response
(Standard American, BBA's Basic-Bridge card). Cases: `rebids.test`.

## Guidance (Rick, 2026-09-22)

- **1NT rebid: 12-14 HCP**, balanced.
- **Jump rebid of opener's suit, or jump raise of responder's suit: 16-18
  total points.**
- **Reverse: 17+ total points, first suit longer than the second.**
- **Jump in a new suit: 19+ total points** (game forcing).

## How the rest was set

BBA's meanings for opener's second call (bba-cli --all-meanings over the
Basic_* deals), with the hands that made them:

| Sequence | BBA |
|---|---|
| New suit at the one level | 11-16 total, 4+ |
| New suit at the two level, lower | 11-17, 4+ |
| Rebid own suit | 11-14/15, 6+ |
| Raise responder's suit | 11-15, 4+ |
| 2NT | 18-19 balanced |
| After 1M-2M: pass / 3M / 4M | 11-14 / 14-17 / 13-20 (median 17) |
| After 1M-3M: pass / 4M | 10-13 / 12-17 |
| After a 2/1: rebid the major | 10-14, 5+ (the default) |

Measures: notrump rebids in HCP, suit rebids in suit points (HCP + ½ per
card beyond four), raises in support points tp(y). Once a fit is found
(responder raised), opener counts support points too: BBA bids game over
2M with 16-18 and a singleton, and passes 14-15 with doubletons. Bands:
pass up to 16, invite 17-18, game 19+; after a limit raise accept at 15+.

Other choices, from BBA: a new suit before rebidding a six-card suit
(6-4 shows the four); four spades bid 1♠ before 1NT, but four-card support
for responder's major raises first; 1NT includes 5-4-2-2; after 1♣-1♦ a
balanced hand rebids 1NT even with one four-card major, and 1♥ with both;
a minimum with no suit to rebid and no lower suit (5-4 with a singleton)
rebids 1NT; after a 2/1 a 5-3-3-2 minimum rebids its major.

## Evidence

Basic_* opener's rebids (1,881 positions): 80.1% agree with BBA (almost
none bid before). Basic_Openers_Rebid: calls 75.7% → 91.2%, identical
auctions 8.6% → 43.4%. Whole corpus: calls agreeing 68.5% → 69.9%, same
contract 14.9% → 20.8%.

## Accepted differences from BBA

- **1NT with 15-16 and a 5-4-2-2** (BBA's range reaches 16): Rick's 12-14.
- **A jump in a new suit with 16-17** (BBA 17-20): Rick's 19+.
- Invitation boundaries over a simple raise (`KQ654.KQ85.A2.J8`: BBA passes,
  we invite with 17 support points).
- 1D-1S with four clubs, balanced: BBA mixes 1NT and 2C.


## Hands with no rebid at all (2026-09-22)

The Problems tab grew a kind of its own, `passed a forcing auction`: a
pass where our own rules said the auction was forcing. The engine will
not *choose* such a pass, but when no rule matches it falls back to one
and the auction dies. Over the nine Basic_* scenarios there were 32, and
every one was opener with no rebid to make:

| hole | what fires now |
|---|---|
| 18-21 with six of a minor (a jump rebid is 16-18) | **3NT**, which is what BBA bids: "calculated bid, 18 to 21 total points, 6+ clubs" |
| 19+ with six of a major | **game in the major** |
| 16-18 with 5-4 and too weak to reverse (a reverse is 17+) | **rebid the five-card suit** at the two level, priority -2 |
| 19+ support for responder's *minor* (the jump raise stopped at 18) | the jump raise, with no upper bound: there is no four-level game in a minor |
| 21 with 4-4-1-4 opening 1♣ | the jump shift, which no longer asks for a fifth club |
| 16-18 after a two-over-one with a five-card major | rebid the major, priority -2 |
| balanced 20-21 after a one-level response | the 2NT rebid, now 18-21: BBA's own 2NT hands run to 21 |

Basic_* now has **no** broken forces at all (32 → 0), with call agreement
unchanged (83.1%) and the contracts a shade closer to par (net -4,145 →
-4,059 IMPs). The whole corpus still has 1,803, all in scenarios on other
cards.

## Gaps (not built yet)

- **Inverted minors** (the 2/1 cards): 1m-3m is weak there, but we read it
  as a limit raise, so opener bids 3NT over it.
- **A passed hand's new suit is not forcing**: BBA's opener may pass it.
- Slam tries by opener (4NT over a limit raise, cuebids).
- Rebids after 2♣ strong openings, preempts and interference.

## Opener's third call (2026-09-23)

The auctions were dying in the middle: on the Basic-Bridge cards
(11,656 boards) there were 5,989 "no rule in a live auction" points, and
**925 of them were in uncontested auctions — every one of them opener**,
with nothing to say at his third call. The clusters were all the same
shape: responder had made a limited second call (a preference, a weak
rebid of his own suit, a raise of opener's second suit, 1NT), and no
rule covered what opener does about it. Listing them as `after` patterns
would have taken a dozen eight-call contexts.

**Responder's weak second calls now set `ask=signoff`** (in
responder-rebids.bid), and opener answers once, against the ask:

```
when asked signoff
  P    priority -5                       # the default: partner is limited
  2x   six or more {x}, minimum          # a better spot, when partner has not supported
  3x   six or more {x}, 16-18            # one more try; sets ask=invite(x)
  3x   raise with 16-18 opposite five
  2y   preference with three, opposite partner's five
  4M   game with 19+
```

This is the §10 pattern: a call that says "play here" is a question with
one answer, and the answer is written once whatever the auction was. It
closed about 600 of the 925 by itself, and the same rules serve
responder when opener signs off (opener's own weak third calls set the
ask in turn, which closed the round after them).

**Invitations in a minor had no answer anywhere.** `ask=invite(M)` is
answered in jacoby-transfers.bid, and because that context is pure state
it was already answering our suit auctions' major invitations. A minor
invitation (1♦-1♥-2♦-3♦) was answered by nobody, so opener passed a live
auction. `when asked invite(m)` now accepts with 3NT (14+ and a shape
for it) or 5 of the minor (18+), and otherwise declines.

**Two bugs found on the way:**

- `after 1m (P) 1y (P) | 1m (P) 1N (P) | 1m (P) 2y (P)` with
  `when y is not m`: after the 1NT alternative `y` is unbound, so the
  condition can never be decided and the rule never fired. A hand with
  19+ and six of a major had no rebid over a 1NT response. The 1NT
  alternatives now have contexts of their own, without the `when`.
- A minimum with a five-card minor and no reverse was rebidding the
  minor ahead of 1NT (the 1NT fallback was priority -3, the minor rebid
  -2). BBA rebids 1NT on those (1♣-1♠ with 1-4-3-5 and 13), and so do
  most players: the minor rebid is now -4. Worth +66 calls on its own.
  The 1NT fallback was tightened to 12-14 at the same time, to keep
  Rick's ruling that the 1NT rebid is 12-14: a 15-16 with 5-4-2-2 still
  rebids the minor. (Leaving it at 15 agreed with BBA 28 calls more
  often; the ruling won.)

Also new: **the fourth suit at the one level** (1♣-1♦-1♥-1♠, natural on
this card) had no third call for opener at all — he raises with four
(jump with 16-18, game with 19+), rebids 1NT or 2NT by strength, or
repeats a six-card suit.

### Evidence

BBA's own meanings for the 64 `1♦-1♥-1♠-1NT` positions in Basic_*:
**56 passes** (12-15 HCP), five 3♦ (6+ diamonds, 14-17), one 5♦ (seven),
one 5♣, one 2NT (17). So passing is the rule and a jump in the long suit
the exception, which is what the block above does — except that we make
the jump at 16-18 and BBA from 14. After responder's preference (51
positions) BBA passes 18 times with 11-15, raises responder's major to
three with 13-14, and bids game with a six-card major and 13-17.

### Numbers (Basic-Bridge cards, 24 scenarios, 11,656 boards)

| | before | after |
|---|---|---|
| calls agreeing | 89,437 (85.1%) | 89,506 (85.1%) |
| identical auctions | 41.3% | 41.4% |
| same contract | 53.5% | **54.3%** |
| par, net IMPs vs BBA | -7,574 | **-6,969** |
| no rule in a live auction | 5,989 | **5,238** |
| uncontested no-rule points | 925 | **10** |
| trump fit under 7 cards | 176 | 119 |

Whole corpus (342 scenarios, 170,161 boards), with responder-rebids.bid:
calls 1,331,086 (74.9%) → 1,332,454 (75.0%); identical auctions 14.9% →
15.0%; same contract 27.0% → **27.7%**; no rule 165,221 → **156,108**;
short fits 3,617 → 2,786; par -244,279 → **-232,422** IMPs. Two counts
went the wrong way, both small and both outside these auctions:
"passed a forcing auction" 1,809 → 1,836 and "contradicts earlier calls"
20 → 28, on cards the comparison does not cover.

### Accepted differences from BBA

- **The one-more-try jump is 16-18, BBA's is 14+.** After
  1♦-1♥-1♠-1NT with six diamonds and 14-15 BBA bids 3♦; we bid 2♦ when
  partner has not shown support, and pass otherwise. Rick's ladder puts
  a jump rebid at 16-18 and this is the same call one round later.
- **Game opposite a signoff needs 19.** BBA bids 4♥ with a six-card
  major and 13-17 opposite a weak preference; we pass or try.

### Tried and rejected

- Lowering the bands after a simple raise (pass ≤15, invite 16-17, game
  18+, against the present ≤16 / 17-18 / 19+): -41 calls, contracts
  54.2% → 54.0%, par +43 IMPs. Not worth it; the present bands stay.

### Open questions

- ~~`when asked invite(M)` lives in jacoby-transfers.bid~~: moved here
  2026-09-30 ("The major invitation without transfers").
- Opener's 2NT at the third call: BBA bids it with 17 and 4-4-1-4 after
  1♦-1♥-1♠-1NT. We have no rule for it (a 17-count that has already
  shown two suits). Is 2NT right, or is pass?

## Opener's rebid once they come in (2026-09-23)

Everything above is written as `after 1x (P) 1y (P)` and its relatives,
so none of it fired once the opponents acted: **opener had no second
call in any contested auction**. On the Basic-Bridge cards that was 911
dead boards, and across the corpus about 46,000 — the largest cluster
of dead auctions left anywhere. It was also the largest source of
`passed a forcing auction`: 1,438 of the 1,838 were opener with no
answer to partner's forcing free bid at the two level.

### The shape of the solution

Opener's problem after interference is the same problem as before it.
What he does with his own hand — raise partner, rebid his suit, bid the
game — does not depend on what they bid, so the new blocks are written
with `(*)` for the opponents' calls and with **relative calls**
(`cheapest(x)`, `jump(x)`, `4{z}`) so the level looks after itself.
Seven contexts replace what would otherwise have been a few dozen
eight-call patterns, one per (their call × partner's call × level):

| block | when | what opener does |
|---|---|---|
| `1x (*) 1z (*)` | partner's one-level free bid | raise, rebid, second suit, notrump |
| `1x (*) 2z (*)`, `3z`, `4z` | partner's two-level-or-higher free bid | raise, rebid, pass a minimum |
| `1x (*) 2x (*)` / `3x (*)` | partner raised | the uncontested ladder |
| `1x (*) 1N (*)` | partner bid notrump | pass, rebid, jump |
| `1x (X) XX (*)` | partner redoubled | rebid or leave it to him |
| `1x (*) P (*)` | partner passed, they keep bidding | a six-card suit, else pass |
| `1x (1y) P (P)` … | they bought it, partner passed | reopen: double, second suit, 2NT |

Two things do depend on their call, and neither needed a pattern
variable for it:

- **Never bid a suit they have shown.** `maybe lho.S<=4, maybe rho.S<=4`
  reads "neither of them has shown five spades". It is false after a
  spade overcall (the overcall showed 5+) and true after a takeout
  double (which showed 3+), so one rule is right over a double, an
  overcall, a cue bid or a jump alike. Knowledge does the work a `y is
  not S` guard would have done, and it does it in a `(*)` context where
  there is no `y` to name.
- **Never compete against a strong notrump.** `maybe lho.hcp<=14, maybe
  rho.hcp<=14` is "they have not shown the balance of the pack". Without
  it, opener kept rebidding his six-card suit over a 1NT overcall and
  its Stayman, and that alone cost 157 calls in the five
  `We_Overcall_NT_*` scenarios.

The only place a pattern still names their suit is **notrump**, which
needs `stop(y)`; those rules sit in small blocks of their own, with a
copy for the takeout double where there is no suit to stop.

Rick's ladder is unchanged, because partner's free bid is worth about
what an uncontested response is worth: 1NT 12-14, a jump rebid or jump
raise 16-18, a reverse 17+ with the first suit longer, a jump in a new
suit 19+, and over a raise pass to 16 support points, invite with 17-18,
game with 19.

Every minimum rebid sets `ask=signoff` and every one-more-try sets
`ask=invite(suit)`, so the state blocks written in the last wave answer
them without a single new pattern.

### Evidence (bba-cli --all-meanings, Basic-Bridge scenarios)

Twenty-two scenarios were re-bid with meanings; the positions are thin
one auction at a time, so the rules follow the shape rather than the
exact boundaries.

| position | BBA |
|---|---|
| `1x (1y) 1z (P)` | **39 calls, no pass**: raise partner's four 12-14, 1NT 12-14 with a stopper, rebid six 12-15, 2NT 18-19, jump raise 14+ |
| `1x (X) 1y (P)` | **49 calls, no pass**: a new suit at the one level 11-15, 1NT 12-16, jump rebid 15-17, raise partner's four 12-14 |
| `1x (1y) 2z (P)` | pass 8, rebid six 10-15, raise partner's five 14, game 18-20 |
| `1x (1y) 2x (P)` | pass 23 (12-15), invite 6 (12-17), game 5 (14-17) |
| `1x (2y) 2x (P)` | pass 22, game 11 (14-18), invite 6 (15-17) |
| `1x (X) 2x (P)` | pass 32, game 3 (17-19) |
| `1x (1y) P (P)` | **reopen**: double 11 (a singleton or doubleton in their suit), a five-card suit 11, rebid six 8, pass only 6 |
| `1x (2y) P (P)` | rebid six 13, a five-card suit 10, reopening double 10, pass 19 |
| `1x (X) P (2y)` | pass 48, rebid six 18, jump with six 16-17 |
| `1x (X) XX (1y/2y)` | rebids and second suits; **never a penalty double** on these cards |
| `1x (X) P (P)` | pass, 8 of 8 |

So: opener always answers a one-level free bid, may pass a two-level
one, passes most of what partner's pass leaves him, and reopens more
often than he passes when they have bought it cheaply.

### Numbers

Basic-Bridge cards (24 scenarios, 11,656 boards), before → after, with
`responder-rebids.bid`:

| | before | after |
|---|---|---|
| calls agreeing | 89,583 (85.2%) | **89,841 (85.4%)** |
| identical auctions | 41.1% | **41.5%** |
| same final contract | 54.6% | **56.2%** |
| par, net IMPs vs BBA | -5,806 | **-4,971** |
| no rule in a live auction | 4,941 | **3,863** |
| passed a forcing auction | 1 | 5 |
| trump fit under 7 cards | 127 | 128 |
| contradicts earlier calls | 0 | 1 |

Whole corpus (342 scenarios, 170,161 boards):

| | before | after |
|---|---|---|
| calls agreeing | 1,334,882 (75.1%) | **1,340,690 (75.4%)** |
| identical auctions | 15.0% | **15.4%** |
| same final contract | 28.8% | **30.1%** |
| par, net IMPs vs BBA | -213,199 | **-200,197** |
| no rule in a live auction | 147,686 | **112,025** |
| passed a forcing auction | 1,838 | **102** |
| trump fit under 7 cards | 3,143 | **2,949** |
| contradicts earlier calls | 28 | 43 |

The two tables above are measured against the tree these files were
written from (78cb267). Re-measured on top of the notrump-under-
interference work that landed while they were being written (dbe10ec),
the same six files give the same picture: subset calls 89,635 (85.3%) →
89,892 (85.5%), same contract 54.6% → 56.2%, no rule 4,890 → 3,825;
corpus calls 1,336,873 (75.2%) → 1,342,617 (75.5%), identical auctions
15.1% → 15.5%, same contract 28.9% → 30.2%, par -214,172 → -201,138
IMPs, no rule 143,210 → 108,381, passed a forcing auction 1,838 → 102,
short fits 3,239 → 3,020.

258 scenarios moved, +5,808 calls and +2,152 contracts in all. Biggest
movers: Competitive_Doubles +303 calls, Trap_Pass +254,
Snapdragon_Double +176, Trap_Pass_Maybe +157, Double_Showing_2_Suits
+149, Robot_Free_Bid +136, Opps_Takeout_X +131, WB5_Collante +125.

Every contested opener-rebid cluster on the subset went to zero: the
seven the competitive notes listed (68, 65, 60, 55, 51, 50, 48 boards)
and thirteen more, 911 points in all. Corpus-wide the `1x (…)` clusters
of 200 boards or more fell from 46,309 to 7,100, and what is left there
is `1x (1N)` (5,584, their 1NT overcall — not written anywhere) and
`1x (P)` (782, responder with no call at all).

No divergence point in these contexts appears in the twenty most
expensive by IMPs, on the subset or corpus-wide.

### Accepted differences from BBA

- **Opener invites over a contested raise where BBA passes.** The ladder
  is the uncontested one (pass to 16 support points, invite 17-18), and
  partner's contested raise shows a point or two less than an
  uncontested one, so we invite on a handful of hands BBA passes: +11
  new disagreements at `1x (2y) 2x (P)` and `1x (X) 2x (P)`. In exchange
  the same two auctions lost 14 where BBA bid game and we had no rule.
  Keeping one ladder is worth more than the eleven calls.
- **We pass a two-level notrump rebid that BBA also passes, but we rank
  it differently.** Over partner's two-level free bid a 12-15 balanced
  hand with a stopper is ranked *below* the pass (priority -5), so the
  2NT rebid only appears when our own rules have made the auction
  forcing. That is what BBA does in practice and it is worth 8 calls.
- **Opener never doubles them for penalty after partner's redouble.**
  BBA does not either on these cards, and the redoubler is better placed
  to judge; opener passes and leaves it to him (`responder-rebids.bid`
  then doubles with four of their suit).
- **A 15-17 balanced hand with no fit, no six-card suit and no reverse
  rebids 2NT** (priority -4, so only when nothing else fits). Under
  Rick's 12-14 ruling 1NT would be a lie. On a card whose 1NT opening is
  15-17 the hand never arises; it appears in the strong-club scenarios,
  where it was 279 of the corpus's broken forces.

### Tried and rejected

- **New suits by opener when they have bid again.** The second-suit
  rules were first written in the `1x (*) 1z (*)` context, where they
  also fired after `1x (1y) 1z (2y)` and produced three-level bids BBA
  passes. They now live in the trailing-`(P)` context only: opener shows
  a second suit when partner's free bid is still unanswered, and
  competes with a fit or a big hand when they have bid again.

### Gaps and open questions

- **Their 1NT overcall** was the largest single hole left after this
  work (5,584 boards: `after 1x (1y)` binds a suit, so nothing fired
  after `1x (1N)`). It was closed independently by
  `competitive/their-1nt-overcall.bid` (dbe10ec) while these blocks were
  being written, which is why the merged corpus numbers above are better
  than either change on its own.
- **For Rick.** Opener's reopening double (`1x (2y) P (P) X`) is written
  as 12+ with a doubleton or shorter in their suit and three cards in
  each other suit, or 16+ with any shape. BBA reopens with 12-17 and
  0-2 in their suit. Is the three-card requirement in every other suit
  right for an opening bidder, or should a long suit of his own reopen
  with the double as well?
- **For Rick.** Over partner's two-level free bid opener raises with
  three cards when partner has promised five (`1x (1y) 2z (P) 3z`). BBA
  does the same with 14, but on the cards where the free bid shows only
  8 total points this gets high quickly. Is three-card support enough at
  the three level?
- Opener's third call in a contested auction is answered only through
  `ask=signoff` and `ask=invite`. The calls that set neither — the
  reopening double, the second suit at the one level — are answered by
  the new blocks in `responder-rebids.bid`, but only for the patterns
  that occur often; the long tail is still open.
- `contradicts earlier calls` went 28 → 43 corpus-wide. About eleven of
  the new ones are in these contexts: `when asked signoff`'s "raise with
  the fit" fires for a player who has already denied the length. The
  rule needs a `maybe` on its own shown length, which is
  `rebids.bid`'s to fix in the next pass.

## Opener's jump to 4M (Rick, 2026-09-23)

With six or more of the major and 19-21, opener jumps to game after a new
suit (1♥-1♠-4♥) or 1NT. The 19 counts a full point for each card beyond
four, as the opening does (`hcp+length_points`), not the half point of
`suit_points`: BBA's jumps in Basic_* are 13-17 HCP with six or seven
hearts. Measured: +239 IMPs vs par on the corpus, +15 on Basic_*. One
case changed: `T32.AKQ732.A7.A4` after 1♥-2♣ now bids 4♥, not 3♥.

## BBA treatment (2026-09-24)

Rules under `when style is bba` (module param `style = general.style`);
the default is unchanged (every corpus board bids the same with and
without them). Probes: `rbb probe`, Basic-Bridge both sides, dealer S,
None, MP, opener South with the auction forced up to his rebid; honours
added one at a time, no tens.

**Raising responder's major** (`1♦-1♥`, four hearts):

| Opener (♠-♥-♦-♣) | 13 | 14 | 15 | 16 | 17 | 18 |
|---|---|---|---|---|---|---|
| 1-4-5-3 (small singleton ♠) | 2♥ | 2♥ | 4♥ | 4♥ | 4♥ | 4♥ |
| 2-4-5-2 | 2♥ | 2♥ | 3♥ | 3♥ | 4♥ | 4♥ |
| 3-4-5-1 (singleton ♣K) | 2♥ | 3♥ | 3♥ | 3♥ | 3♥ | 4♥ |
| 3-4-4-2 | 2♥ | 2♥ | 3♥ | 3♥ | 3♥ | 4♥ |
| 1-4-4-4 (small singleton ♠) | 2♥ | 2♥ | 4♥ | 4♥ | 4♥ | 4♥ |

In the corpus (Basic_*, 1♦-1♥, 1♣-1♥, 1♣-1♠, 1♦-1♠, 1♥-1♠) BBA bids 3M
with 16-17 and a singleton honour and with 16 and a small singleton about
as often as 4M (`AJ74.A742.4.AKT8` 3♥, `AK94.KQ94.A983.7` 4♥), and 2♥
with 13-14 and 4-4-4-1. What fits the corpus best: 2y up to 14 HCP plus a
point a card beyond four (`hcp+length_points`, no shortness); 3y 15-17 on
the same count; 4M with 18 HCP, or 17 and 19 support points. The probes'
jump to game with 15 and a small singleton in the unbid major is not
modelled (it would gain 3 corpus calls and lose 2).

**Rebidding a six- or seven-card major** (`1♥-1♠`): six hearts 12-14 2♥,
15-18 3♥; seven hearts 4♥ from 13 (2-7-2-2) or 15 (1-7-3-2). Corpus: six
hearts, no four-card side suit and 16-18 HCP bids 3♥ 15 times of 15 (we
bid 4♥ on 17-18, `hcp+length_points >= 19`); seven and 13-15 bids 4♥.
After 1M-1NT a six-card major and 16-18 bids 3M 19 times of 19. Rules: 3x
6+ and 15-18 HCP; 4M with seven and 13+, or six and 19+ (after a
one-level response, a 2/1 and 1NT).

**1♣-1♦** (BBA's 1♦ is "Walsh style", and `--all-meanings` gives opener's
1NT as "balanced, 11 to 16 total points, 2-4 cards in each major"):

| Opener | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 |
|---|---|---|---|---|---|---|---|---|
| 4-3-1-5, 3-4-1-5, 4-2-2-5, 4-1-3-5 | 1NT | 1NT | 1NT | 1NT | 1NT | 1NT | 2M | 2M |
| 4-3-0-6, 4-1-2-6, 4-2-1-6 | 2♣ | 2♣ | 2♣ | 2♣/3♣ | 3♣ | 3♣ | 2M | 2M |
| 4-1-4-4 | 1NT | 2♦ | 1NT | 1NT | 3♦ | 3♦ | 2♠ | 2♠ |
| 1-4-4-4 | 2♦ | 2♦ | 2♦ | 2♦ | 3♦ | 3♦ | 2♥ | 2♥ |

In the corpus BBA's 1♥ here always has both majors (13 of 13), and 1NT
covers 12-17 with one four-card major (13 hands where we bid the major).
Rules: 1NT 12-17 with one four-card major, clubs no longer than five and
no singleton unless clubs are five; 1♥ only with both majors; 1♠ never;
2M jump shift from 18 HCP.

**1♦-1M with 4-4 in the minors** (balanced, no four-card major): 1NT
with 12-13 (15 of 15 corpus hands), 2♣ with 14 (20 of 20, with or
without tens). Rule: 1NT 12-13 balanced with
four clubs, priority 1 over 2♣.

**After 1M-1NT**: BBA's jump shift into hearts (1♠-1NT-3♥) is 17 HCP with
5-4 (4 of 4) or 15-17 with 6-4 (4 of 4); with 15-16 and 5-4 it bids 2♥
(3 of 3). Into a minor it bids 2m with 5-4 and 15-17 (27 of 27 after
1♠-1NT and 1♥-1NT) and jumps with 18 or more (8 of 8). Rules (priority 1): 3♥ with 17+ suit points, 3♣/3♦ with 18+;
4♥ with 5-5 in the majors and 16+ (2 hands).

**Opposite a single raise** (1♠-2♠, probes):

| Opener | 13 | 14 | 15 | 16 | 17 | 18 |
|---|---|---|---|---|---|---|
| 5-3-3-2 | P | P | P | 3♠ | 2NT | 4♠ |
| 5-4-2-2 | P | P | P | 3♠ | 4♠ | 4♠ |
| 5-4-3-1 (singleton ♣K) | P | P | P | 3♠ | 4♠ | 4♠ |
| 5-3-4-1 (singleton ♣K) | P | P | P | 3♠ | 4♠ | 4♠ |
| 6-3-2-2 | P | 3♠ | 3♠ | 4♠ | 4♠ | 4♠ |

Corpus: BBA passes every 15-count with five trumps (16 of 16, singletons
and 5-5 included), where our tp-based try asked with 17-18 support points;
16 with a small singleton or six trumps bids game. Rules: pass up to 15
(13 with six trumps), 3M on 16 (14-15 with six), 4M on 17, or 16 with six
trumps or 19 support points. The 1m-2m rules are unchanged.

**Measured** (Basic_* uncontested NS, `--set general.style=bba`, calls
agreeing at opener's rebid, before → after):

| Auction | positions | before | after |
|---|---|---|---|
| 1♣-1♦ | 143 | 115 (80.4%) | 128 (89.5%) |
| 1♣-1♥ | 109 | 98 (89.9%) | 100 (91.7%) |
| 1♣-1♠ | 98 | 87 (88.8%) | 89 (90.8%) |
| 1♦-1♥ | 241 | 217 (90.0%) | 230 (95.4%) |
| 1♦-1♠ | 143 | 123 (86.0%) | 131 (91.6%) |
| 1♥-1♠ | 127 | 100 (78.7%) | 116 (91.3%) |
| 1♠-1NT | 157 | 132 (84.1%) | 153 (97.5%) |
| 1♥-1NT | 51 | 43 (84.3%) | 48 (94.1%) |
| 1♠-2♠ | 80 | 55 (68.8%) | 71 (88.8%) |
| 1♥-2♥ | 59 | 42 (71.2%) | 53 (89.8%) |

All of opener's rebids after an uncontested one-level response: 79.1% →
85.5% (1,747 positions). Totals with the response treatment are in
responses.notes.md.

**Our default against BBA's, by par** (each BBA rule made the default in
a scratch copy, corpus net IMPs vs BBA, default −173,271; Basic_* −1,926):

- 1♦-1M 1NT with 4-4 minors and 12-13: corpus −172,937 (**+334**),
  Basic_* −1,904 (+22). **Question for Rick**: adopt it as the default?
- 1♣-1♦ as BBA (1NT 12-17 with one major, jump shift 18): corpus −173,839
  (−568), Basic_* −1,901 (+25). Not better.
- 1M-2M as BBA (pass to 15, 3M on 16): corpus −175,070 (−1,799), Basic_*
  −1,943 (−17). Our default is better.

## Rick, 2026-09-24: Walsh after 1C-1D, 1NT after 1D-1M

- **1C-1D: the Walsh treatment is the (modern) default.** A balanced
  12-14 rebids 1NT even with one or both four-card majors (responder
  bypasses diamonds with a major and a weak hand, so opener's majors can
  wait). `standard`: a four-card major up the line, balanced or not.
- **1D-1M with a balanced 12-14: 1NT, not 2C**, even with four clubs.
  BBA does this on 12-13 and bids 2C on 14; measured as a default before
  the ruling: +334 IMPs on the corpus.

Together with the weak-two opening and the standard feature answers,
the default went from -173,271 to -172,679 IMPs against par on the
corpus (+592), Basic_* -1,926 -> -1,893, uncontested -838 -> -820. The
BBA style is unchanged.

## Balanced 16-17 passes 1NT (2026-09-25)

After 1M-1NT (not forcing), BBA passes a 5-3-3-2 with 16-17
(`probes/major-rebid-1S-1N.toml`, 1,500 hands, 87% agreement); we
rebid the five-card suit. Opposite 6-10 there is no game, so balanced
hands up to 17 HCP now pass. Corpus: -167,152 -> -167,116 (+36).

## 2NT and the game raise before a jump shift (2026-09-25)

Probed after 1D-1S and 1C-1H (`probes/rebid-1D-1S.toml`, `1C-1H`;
2,000 hands each; 86-87% agreement). BBA rebids 2NT with 18-19
balanced and raises to game with four trumps, where we made a jump shift
or reverse on descriptiveness. Both now outrank the jump shift. 5-4-2-2
hands still jump, as Rick's tests have them.
- Distance from par: +220.
- To the side that bids: +373.
- Rick's support-point raise ranges (2y up to 15, 3y 16-18, 4M 19+)
  against BBA's HCP-and-length ranges: BBA's lose 325 by distance and
  333 to the bidding side (224 boards). Ours stay.

## No-rule gaps: after a jump shift and after a reverse (2026-09-25)

Found by the no-rule scan of Basic_* N/S auctions.

- **1M-3m (the invitational jump shift on Basic-Bridge).** 3NT needed a
  doubleton or less in responder's suit, so a 14+ opener with a fit for
  the minor had no call (Basic_Major 44, 121, 241). 3NT now allows a
  minor fit, and opener bids game in a seven-card major, or six with
  four of the top five honours.
- **1x-1NT-2z (reverse)-3x.** Opener had no third call. 4M with 19+
  suit points in a major; with a minor 3NT from 19 HCP, 5m with six and
  20; otherwise pass.

Basic_* N/S: vs BBA +30, +44 by side (7 boards). Full corpus: +267 by
par distance, +327 by side (174 boards).

More gaps closed the same day: 1m-3m with an unbalanced hand
now bids 5m from 14, as 3NT does balanced (it had no call; Basic_Minor 35, 5-5 with 14, where BBA
passed), and after 1x-1NT a 4-4-4-1 18+ with nothing to reverse into
bids 3NT (Basic_Minor 78, BBA 3NT). Measured with the responder and
transfer fixes below: full corpus +807 by par distance, +1,168 by side.

## Strong rebids after an overcall (2026-09-25)

In the overcall quality A/B, the side yardstick's "gain" for light
overcalls was mostly our opening side stopping short after them. Two
contested rebid blocks had no strong call: after a one-level free bid
opener with 19+ and no fit repeated a five-card suit, and after a
two-level free bid 17-18 with their suit held did too. Now: 3NT with a
stopper (19+ over a one-level response, 17+ over a two-level one),
else a cue bid of their suit, forcing to game. Also responder raises
opener's reopening notrump (18-19) to 3NT with 7+
(responder-rebids.bid). Full corpus +257 by par distance, +346 by side
(131 boards); Basic_* competitive -12 / -18 (13 boards).

## The simple raise at the three level (2026-09-26)

Over a two-level overcall 1♦ (2♠) 3♦ is the cheapest raise (7-10), but
the contested raise block read every three-level raise as preemptive,
so opener passed with 16-21. For a non-jump raise (`!partner.jumped`)
opener now bids 3NT with 16+ and their suit held, 4M with 17 support
points, 5m with 18. Full corpus +127 by par distance, +303 by side.

Over partner's contested 1NT (6-10, their suit stopped) opener now
invites with 16-17 (2NT) and bids 3NT with 18+, any shape; not after a
takeout double, where 1NT has no suit to stop (that lost). Full corpus
+95 by par distance, +89 by side.

## No jump when opener is not forced (Rick, 2026-09-27)

Basic_Takeout_Double 193: 1♥ (X) P (2♥), opener KQ84.AKQ973.Q.94 jumped
to 4♥ ("six or more, 16-18"); BBA bid 3♥. Rick: the 16-18 jump rebid
is for when partner's response forces opener to bid (1♥-1♠-3♥). When
partner has passed and they keep bidding, opener is free to pass, so
bidding again at all shows extras, and there is no need to jump; with
16 and a singleton queen, 3♥ is the least lie. The block "partner
passed and they kept bidding" now has one rebid of a six-card suit,
13-18 suit points, and no jump.

| free rebid | full corpus (par / side) | Basic_* (par / side) |
|---|---|---|
| up to 18, jump removed | -35 / +746 | -25 / +57 |
| 13-18 | +191 / +119 | -16 / +40 |
| 14-18 | +529 / -1,504 | -12 / +11 |

13-18 kept: the only one where both yardsticks gain on the corpus. Four
rebids.test cases that expected the jump now expect the free bid. It
also changes what opener's rebid tells the others: E's 3♥ now reads
13-18, which is what lets North place the doubler below 18 by deck
arithmetic (the Law of Total Tricks work that follows).

## No jump to game over a game force (2026-09-30)

"19+ with six or more M: game" also matched 1M-2y where 2y is a two-over-
one or a strong jump shift, both game forcing: 1H 2S 4H with 20 HCP
facing 15+, and responder passed. Over a game force the jump to game is
the minimum (fast arrival), so the rule now needs `!we.gf`; opener raises
or rebids and the slam machinery takes over (slam-entry.notes.md).

**For Rick:** the minor twin, "18-21 with six or more m: 3NT", was
changed the same way and measured both ways. Without it, random deals
gain +112 and the corpus loses -1,027 (465 boards of 1m-2y-3NT where
responder's 6NT counted opener's 18-21; without the rule opener has only
base.bid's bare 3NT after a jump shift over a minor). Kept for now. The
proper fix is opener's natural rebids after a minor-suit jump shift
(repeat the suit, raise, 2NT), which do not exist yet.

## Over a forcing 1NT (2026-09-30, 2/1 game force)

A forcing 1NT (6-12) may not be passed: a balanced minimum bids its longer
minor, three cards if need be; 18-19 balanced bids 2NT; 2M is six, or five
with no three-card minor (5-4-2-2 with the other major), one rule so that
responder reads either (reading it as six contradicted the five).

## The major invitation without transfers (2026-09-30)

On the vanilla SAYC random set (`.rbb-cache/random/README.md`: BBA's bare
SAYC card, transfers off on our side) opener had no rule after
1♦-1♥-2♥-3♥, 1♠-1NT-2♠-3♠, 1♣-1♠-2♠-3♠ and every other invitation in
a major: about 3,000 of the 4,675 no-rule boards. Responder's invitations
set `ask=invite(M)`, and the only answer to that question was in
jacoby-transfers.bid, a module a card without transfers switches off. The
two state rules (`when asked invite(M)` and responder's pass after the
decline) now live here, unchanged.

Two fixes came with it, because the transfer answer had never seen a suit
auction:

- **A raise shows support points.** Responder's 3M raise shows
  `tp(M)=11..12`, not `points`, so `points>=24-partner.points.min` read
  responder as the 6 of the one-level response and opener never accepted.
  New rules ahead of it, for invitations that showed 10+ support points
  (`partner.tp(M).min>=10`): game from 25 support points between us, 14
  opposite 11-12. BBA's decisions on the vanilla set agree at that line:
  it declines AJ2.KT94.Q543.Q2, AQ52.Q83.KJ9.JT4, AQ76.Q54.A76.J74 (12-13,
  no shortness) and accepts KJ73.KQ75.A642.8, KQ98.K5.985.AQ42,
  Q8.KQ97.AJ652.J3 (14+ counting shortness).
- **1M-1NT-2M-3M** was read by the "3+ {x}, maximum for the 1NT response"
  rule, which a 1M-1NT responder (two at most) cannot hold, so opener
  saw no fit. That rule is now for minors (responder-rebids.bid).

Measured (vs BBA, double-dummy par as the yardstick):

| set | before | after | change |
|---|---|---|---|
| vanilla SAYC, 95,558 boards | −45,508 | −42,209 | **+3,299** |
| corpus (342 scenarios) | −95,102 | −93,855 | **+1,247** |
| 21GF random, 100,000 boards | −44,476 | −43,948 | **+528** |

The move alone changed nothing on the corpus or 21GF (both play
transfers); the gains there are the two fixes. No-rule boards on the
vanilla set 4,675 → 1,641.

## Up the line without Walsh (2026-09-30)

Rick's Walsh ruling (2026-09-24) made the 1NT rebid after 1♣-1♦ hide a
four-card major on every card that was not `standard`. BBA reads its own
switch, `minor_openings.walsh.play`, as exactly this choice for opener:

- **Switch on** (Basic-Bridge, 21GF-GIB, Precision; probe
  `probes/rebid-1C-1D-walsh.toml`): balanced hands with one major rebid
  1NT, 1♥ only with both majors (as the BBA treatment above).
- **Switch off** (bare SAYC, 21GF-DEFAULT and most 21GF cards; probe
  `probes/rebid-1C-1D-21GF.toml`; the vanilla SAYC set): a four-card
  major up the line, balanced or not. On the vanilla set BBA's 1NT never
  holds a four-card major (0 of 363); 1♥ has 1,037 hands (12-14 HCP 797),
  1♠ 667, none of them with four hearts.

Rule: the Walsh 1NT applies when the card plays Walsh
(`walsh = minor_openings.walsh.play`, default on, so a card that does not
say keeps Rick's default); with it off every style bids the major up the
line and 1NT denies one, which is how responder reads it (`S 0-3 H 0-3`).
Rick confirmed this reading of his ruling (2026-09-30): a card that
turns Walsh off, 21GF-DEFAULT included, bids up the line.
The `standard` style is unchanged (always up the line). The `bba` style
follows the switch too: its 1NT with one major only when Walsh is on.

Responder had no game after **1♣-1♦-1♥-1♠** and opener's minimum: 1NT
and 2♠ set `ask=signoff`, so a 16-count passed 1NT. Rare while 1♥
promised both majors; common once it does not. Responder now bids 3NT
(13+) or 2NT (11-12) over 1NT, 4♠ or 3♠ over the raise, and 3NT over 2♣
(responder-rebids.bid). BBA's own answer over 1NT on the vanilla set:
pass 30, 3NT 15 (13-17 HCP), 2NT 6.

Measured (vs BBA, double-dummy par as the yardstick; main f2954c7 →
this change):

| set | before | after | change |
|---|---|---|---|
| vanilla SAYC, 95,558 boards | −38,490 | −37,952 | **+538** |
| corpus (342 scenarios) | −93,830 | −92,854 | **+976** |
| 21GF random, 100,000 boards | −43,892 | −43,589 | **+303** |

The up-the-line rebid alone (measured on the previous main, before the
responder fix) was −55 on the vanilla set (+163 on 1♠, −218 on 1♥,
mostly 1♣-1♦-1♥-1♠-1NT passed out with game values), +209 on the corpus
and +28 on 21GF; the responder fix makes up the rest. Calls agreeing on
the vanilla set 82.4% → 82.5%.

Not this change: the "calls read as a higher rule" pair 1NT
`rebids.bid:55 (-3) -> :51 (0)` (735 on the vanilla set, unchanged;
also `-> :133` 274, `-> :122` 124) is mostly openers with 11 HCP (992 of
about 1,570 calls) and a few 12-14 hands with a singleton, rebidding
1NT through the "Minimum, no better rebid" fallback, which partner reads
as the 12-14 balanced rebid. After 1♣-1♦ the same 102 calls are now read
by the no-major 1NT on a non-Walsh card instead of the Walsh one.

## Opposite a raise to 12 (2026-09-30)

On a card without the limit raise 1M-2M is 6-12 support points
(responses.notes.md, "A card without the limit raise"). Opener reads it
from `partner.tp(M).max` and invites from 15 instead of 17: 3M 15-18,
pass up to 14 (majors only; the minors' raise is unchanged). Of the
invitation ranges tried on the vanilla set, 15-18 with responder still
accepting from 9 scored best (+359 over no change); measured with the
widened raise there.

## Sources

- **Rick's rulings**, dated in the sections above.
- **BBA evidence:** the Basic_* and 21GF corpora, `bba-cli --all-meanings`
  and the probes named in each section; the vanilla SAYC random set for
  the major invitation (2026-09-30); for 1♣-1♦ without Walsh the vanilla
  set and `probes/rebid-1C-1D-21GF.toml` / `rebid-1C-1D-walsh.toml`
  (2026-09-30), which show BBA's `walsh.play` switch deciding opener's
  rebid, not responder's. The invitation opposite a raise to 12 has no
  BBA evidence (BBA does not widen its raise): it is measured on the
  vanilla set (2026-09-30).
- **Book practice:** 1♣-1♦ then a four-card major up the line is
  standard SAYC practice, not yet cited to a book; the Walsh 1NT that may hide a major
  is Rick's modern default (2026-09-24).
- **Corpus measurements:** the par figures given with each change.
- **Book practice:** opener's rebids (Standard American: 1NT 12-14, the
  jump rebid and jump raise 16-18, reverses 17+, accepting a limit raise
  with 14 or more) are standard practice, not yet cited to a book.
