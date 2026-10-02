# Advancing an overcall (`advances.bid`): notes

Advancer's call after partner's overcall, direct or balancing, at the
one level or the two. Cases: `advances.test`.

## Guidance

Rick has not ruled on the advance yet; the ranges are BBA's Basic-Bridge
meanings, which are the usual ones:

- **raise** 7-11 total points with three-card support;
- **new suit** 8+ with four cards at the one level, 9+ with five at the
  two level;
- **1NT** 8-11 balanced with a stopper in their suit, denying an unbid
  four-card major and three-card support for partner's major;
- **jump raise** weak: four-card support and less than 8
  (`overcalls.responses.jump_raise`);
- an overcall is not forcing, so advancer **passes** with less.

## Evidence

From `bba-cli --all-meanings`, grouped over all four opening suits:

| call | BBA's meaning |
|---|---|
| raise | "calculated bid", 7-12 total points, 3+ support |
| new suit, one level | "bidable suit", 8-21, 4+ |
| new suit, two level | "bidable suit", 9-21, 5+ |
| pass | 0-12 total points (0-16 over a one-level overcall) |

## Advancing a two-level overcall (2026-09-23)

The advance of a two-level overcall was the largest no-rule cluster in
the corpus after the first round (`(1x) 2y (P)` and `(1x) 2y (2x)`
together, about 200 boards of the Basic-Bridge subset alone). It is now
written, and it splits on **what the two-level overcall was**:

- **a simple two-level overcall** (y could not have been shown at the
  one level: 2♣ over anything, 2♦ over 1♥/1♠, 2♥ over 1♠) is 12-17 with
  five cards, so advancer can invite. The raise to three is 7-10 with
  **four**-card support, the cue bid of their suit is a limit raise or
  better (11+ with three), 2NT is 11-12 and 3NT 13+ with a stopper, and
  a new suit is 10+ with five (12+ where it has to go to the three
  level). 4M is 13+ with four-card support, so that a limit raise cues
  rather than blasting;
- **a weak jump overcall** (everything else) is a six-card suit and 4-9,
  so the raise is a further preempt: three-card support in a major or
  four in a minor, under 11 total points, and 4M with four-card support
  and 11+, there being no cue bid worth making opposite a weak hand.

The `when` on each context spells the distinction out in `is` tests
(`x is C | (x is D, y is not C) | (x is H, y is S)` is "y is a jump"),
because the language has no operator for suit rank.

Advancer only bids while RHO has left him room — his pass, his double,
or his simple raise of their own suit. Over RHO's free bid, his notrump
or his jump the corpus says advancer passes, and trying to bid there
cost 130 calls across the corpus, so those auctions get the bare
`(*)` pass rule that covers the seat and nothing else.

**The overcaller answers the cue bid.** The cue `sets ask=cue_raise(y)`
and the answer is written `when asked cue_raise(t)`, once, whatever
level the cue was made at: 13+ total points bids the game, a minimum
signs off in the suit. Without it the cue bid was passed out and the
comparison flagged 30 boards whose contract was an artificial call.

### Evidence

`bba-cli --all-meanings`, grouped by abstracted prefix:

| prefix | call | BBA's meaning | hands |
|---|---|---|---|
| (1x) 2y (P), simple | 3y | "calculated bid", 7-10 tp, 3+ | all had **four** |
| (1x) 2y (P), simple | 2x | "limit raise or better" / "strength cue bid", 9-10+ | 3+ support |
| (1x) 2y (P), simple | new suit, 2 level | "bidable suit", 9-10 tp, 5+ | |
| (1x) 2y (P), simple | new suit, 3 level | "bidable suit", 12-17 tp, 5+ | |
| (1x) 2y (P), jump | 3y | "preemptive", 4-14 tp | 2-4 support |
| (1x) 2y (P), jump | 4y | "calculated bid", 3+ support | |
| (1x) 2y (2x) | 3y | "calculated bid", 6-12 tp, 3+ | |
| (1x) 2y (*) | P | 0-12 tp | 73 of 99 simple, 206 of 264 jump |

BBA's 2NT over a weak jump overcall is artificial (a feature ask); we do
not play it, and our natural 2NT is gated to the simple overcall.

## Advancer's cue-bid raise (2026-09-23)

`(1x) 1y (P) 2x` is a limit raise or better: three-card support and 12+
total points. BBA's meaning is "limit raise or better in !y", 20 calls
in the corpus on 12-18 HCP. 2NT with a stopper is 12-15 there, which is
BBA's 14-15 widened a little to meet the 1NT advance at 11.

The cue bid carries **no `priority`**, so descriptiveness ranks it: a
five-card suit of our own is still bid first, which is what the existing
case `K8.J72.Q93.AQT87 | 1D 1H P | 2C` requires.

## Advancer over RHO's 1NT (2026-09-23)

`after (1x) 1y (1z)` binds a suit, so RHO's 1NT left advancer with no
rule at all. The raise is 9-11 with three-card support (two points more
than over a pass — 1NT may be preparing a penalty double, and BBA passes
these hands out to 16 total points) and the cue bid 12+.

## Advancing a balancing overcall (2026-09-23)

Partner reopened, so he is 8-15 rather than 8-17 and part of his values
are borrowed from us. The direct-seat block no longer carries the
`(1x) P (P) 1y (P)` spelling; there is a separate balancing block with
everything about two points dearer: the raise 9-11, 1NT 10-11, 2NT
13-15, a new suit at the one level 7+ and at the two level 11+, the cue
bid 13+. Those are BBA's own `Balancing` meanings (raise 9-11 HCP, 1NT
10-11, one-level suit 7-9, cue bid 13).

Effect on the three balancing scenarios (Balancing,
Too_Strong_for_Overcall4th, Trap_Pass_Opener): calls +5, contracts
11.7% → 12.1%, par +98 IMPs, identical auctions 8.7% → 8.1%.

## Measured effect of the whole slice (2026-09-23)

Basic-Bridge subset (`compare --min-coverage 50`, 24 scenarios, 11,656
boards) and the whole corpus, before → after the second-call work in
this module and in `after-interference.bid`:

| | subset before | subset after | corpus before | corpus after |
|---|---|---|---|---|
| calls | 85.1% (89437) | 85.1% (89481) | 74.9% (1331086) | 74.9% (1331913) |
| identical auctions | 41.3% | 41.4% | 14.9% | 14.9% |
| same contract | 53.5% | 53.6% | 27.0% | 27.2% |
| par, net IMPs | -7574 | -7431 | -244279 | -239918 |
| no rule in a live auction | 5989 | 5692 | 165221 | 156762 |

## Gaps and open questions

- Advancer's **continuations** after the overcaller rebids, and after
  the cue-bid answer: `when asked cue_raise(t)` is answered once and
  then the auction has no more rules.
- Advancer over RHO's free bid, notrump or jump: we pass by rule. BBA
  passes most of them too, but not all, and a responsive double is the
  proper tool (off on this card).
- The **support cue bid** (`overcalls.responses.support_cuebid`) is
  still unwritten; the cue here is always a limit raise.
- Advancing a **1NT overcall** needs nothing here: systems are on, and
  the notrump modules carry the `(1x) 1N (P)` and `(1x) P (P) 1N (P)`
  spellings of their patterns (see `overcalls.notes.md`).
- **For Rick.** Over a simple two-level overcall the raise to three now
  asks for four-card support, because every BBA hand that raised had
  four even though its stated meaning allows three. With three and 7-10
  we pass. Is that right, or should three-card support raise?
- **For Rick.** 4M by advancer is 13+ with four-card support over a
  simple overcall (so that 11-12 cues first) but 11+ over a weak jump
  (where there is no cue worth making). The split is a judgment call.

## Fallback: the defending side after they bid on (2026-09-25)

Once our E/W overcalled in boards BBA bid N/S alone, the overcaller and
advancer had no rule whenever the opening side bid again (63 of the
remaining no-rule positions in Basic_* N/S). A pass at priority -20
under `when lho.opened | rho.opened` now closes them: each of us has
described his hand once. It changes no call (the engine passed anyway)
but it hides the defending side from the no-rule count, which falls
from 95,896 to 28,594 over the corpus. **A real gap on the defending
side (a big fit worth competing on, a second suit) now shows only as a
divergence from BBA, not as a problem.** A narrower form (partner
passed last) closed about two thirds as many; say if you want that one
instead.

Advancer over our E/W's overcall of their 1NT also passes (see
nt-interference.notes.md for the raises that were tried).

## Systems on after they double our 1NT overcall (2026-09-27)

Rick: systems are on after a 1NT overcall, and by default also after a
penalty double of it (some play escapes there, as over 1NT (X)). Every
notrump context spelled `(1x) 1N (P)` now also takes `(1x) 1N (X)`,
still gated by `nt_overcalls.direct.systems_on`. Ticket
basic-takeout-double-b160: North, Q9.JT942.932.J84, transfers with 2♦
(BBA too) instead of passing 1NT doubled.

## Raising a one-level overcall: trumps and points (2026-10-02)

Over a one-level overcall (RHO passing, doubling or bidding at the one
level) the raises now count trumps the way BBA does:

- **three-card support** raises to two with 8-11 total points (was
  7-11);
- **four-card support** raises to two at any strength up to 11;
- the **weak jump** (under 8) needs five trumps, or four at favourable
  vulnerability (`weak_jump_raise(y)` in `total-tricks.bid`); it took
  four at any vulnerability.

Unchanged over a two-level bid by RHO (`(1x) 1y (2z)`): letting the weak
four-card hand raise to two there lost (−34 and −20 IMPs on the two
commonest auctions), so that block keeps 7-11 with three.

**Evidence** (vanilla SAYC random set, BBA on bare SAYC, the advancer of
a one-level major overcall, any RHO action up to 2♦): with three hearts
BBA raises to 2♥ from about 8 HCP and passes 5-7 (we raised 495 such
hands); with four it raises to 2♥ with 2-11 HCP (169 hands where we
jumped) and jumps mostly with five or at favourable. On the boards
where this was the first difference, BBA's pass beat our raise by 729
IMPs and its 2M beat our 3M by 362 (errors yardstick).

**Measured** against the two previous changes (errors yardstick, IMPs;
even / odd boards):

| set | boards | errors | contract / doubling | distance from par | side IMPs |
|---|---|---|---|---|---|
| vanilla SAYC random | 690 | +582 (+328 / +254) | +88 / +494 | −100 | +215 |
| corpus | 844 | +342 (+152 / +190) | +18 / +324 | −156 | +265 |
| 21GF random | 630 | +715 (+377 / +338) | +178 / +537 | +56 | −42 |

Most of the gain is in the doubling errors (the opponents, also us,
missing a penalty double of the light raise), but the contract errors
gain on all three sets as well, so the change is kept. Distance from
par is mixed (for Rick, below).

## Sources

- **The ranges:** BBA's Basic-Bridge meanings, which are the usual ones,
  from `bba-cli --all-meanings` ("Evidence"). Rick has not ruled on the
  advance; the usual ranges are standard practice, not yet cited to a
  book or article.
- **Rick's rulings:** systems on after a 1NT overcall, and by default
  after a penalty double of it (2026-09-27, ticket
  basic-takeout-double-b160).
- **BBA evidence:** BBA's advances of two-level and balancing overcalls
  and its cue-bid raise (2026-09-23 sections). No `probes/*.toml` spec
  yet.
- **Corpus measurements:** the whole slice (2026-09-23) and the
  defending-side fallback (2026-09-25).
- **Where we differ from BBA's stated meanings:** the raise of a
  two-level overcall asks for four trumps, as every BBA hand that raised
  had, although its meaning allows three (for Rick).
- **Raising a one-level overcall** (2026-10-02): BBA's raises on the
  vanilla SAYC random set (95,558 deals, bare SAYC) and the measurements
  in that section; the Law of Total Tricks (Cohen, *To Bid or Not to
  Bid*, 1992) hedged a trump for a weak hand, which is BBA's practice,
  not the book's.
