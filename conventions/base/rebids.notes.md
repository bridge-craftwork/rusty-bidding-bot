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
