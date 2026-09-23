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

- `when asked invite(M)` lives in jacoby-transfers.bid, so a card with
  transfers switched off has no answer to a major invitation in a suit
  auction. It is a state rule and belongs in a base file (here, or
  responses). Moving it needs an owner for that file.
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
