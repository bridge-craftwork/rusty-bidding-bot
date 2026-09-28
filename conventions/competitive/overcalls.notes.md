# Overcalls (`overcalls.bid`): notes

Simple overcalls of a one-level opening, the weak jump overcall, and the
direct 1NT overcall. Cases: `overcalls.test`. Advancer is in
`advances.bid`; the takeout double is in `takeout-double.bid`.

## Guidance (Rick, 2026-09-22)

- **A simple overcall is always a five-card suit.** No four-card
  overcalls on this card (`overcalls.often_4_cards` is off).
- **8-17 total points at the one level, 12-17 at the two level**, taken
  from BBA's Basic-Bridge meanings, which Rick adopted.
- **Too strong to overcall is a double first, then the suit**: 18+ total
  points doubles whatever its shape (`takeout-double.bid`).
- **A jump overcall follows the weak-two rules**: six cards, 4-9 HCP, and
  vulnerable two of the top three or three of the top five honours. A
  seven-card suit jumps to the three level, as it opens there.
- **The direct 1NT overcall is 15-18 balanced with a stopper**, a point
  wider than our 1NT opening.

## The suit has to be worth bidding (Rick, 2026-09-23)

A one-level overcall needs **two of the top three or three of the top
five honours**, or else **an opening hand**: it takes 12 total points to
overcall on a ten-high suit.

Found from board 120 of Basic_Major, where we overcalled 1♠ on
`J8753.QJ8.A63.Q3` — ten HCP and a jack-high suit. BBA passed, we pushed
the auction and they played 2♥ instead of our 3♦.

It is worth far more than that one board: **+4,520 IMPs against par**
across the corpus, identical auctions 15.4% → 15.6%, same contract 30.1%
→ 30.5%. Two `.test` cases had to change, both of which had encoded the
old behaviour on suits like `AT852` (nine HCP, two of the top five) and
`J87654` (seven HCP).

## Evidence

`bba-cli --all-meanings` on `Basic_Overcall.pbn` and
`Basic_Takeout_Double.pbn` with the Basic-Bridge card on both sides
(4,520 and 4,942 calls with their meanings), grouped by auction prefix:

| call | BBA's meaning | hands it made it on |
|---|---|---|
| (1x) 1y | "bidable suit", 8-17 total points, 5+ | 8-16 HCP, five or six cards |
| (1x) 2y | "bidable suit", 12-17 total points, 5+ | 11-16 HCP |
| (1x) 1N | "NT style", 15-17 total points, 2-5 in every suit | 15-17 HCP, never a five-card major |
| (1x) 2M jump | "Weak natural 2M", 4-10 total points, 6-7 cards | 8-10 HCP, six cards |

Two-level overcalls also want a suit: BBA passes plenty of 11-14 HCP
hands with a ragged five-card suit (A7632, K9865, J9543) at the two
level. We ask for **six cards, or two of the top three honours, or four
of the top five**; that gained a little on calls and more on contracts
(auctions matching 7.8% → 8.8% in Basic_Overcall).

**1NT with a five-card major**: every 1NT overcall in the corpus had four
cards or fewer in both majors, so the rule denies a five-card major and
we show the suit instead. BBA does the same (board 1 of Basic_Overcall
overcalls 1♠ on a balanced 16-count with AJ972).

Scenario effect (Basic-Bridge card, 500 boards each):

| | before | after this module | after the whole competitive slice |
|---|---|---|---|
| Basic_Overcall, calls | 66.1% | 72.0% | 74.1% |
| Basic_Takeout_Double, calls | 59.6% | 60.4% | 71.3% |

## Systems on over the 1NT overcall

`nt_overcalls.direct.systems_on` (default on, which is what BBA plays).
Rather than copy the 1NT family into a competitive module, the `after`
line now takes alternatives (`docs/LANGUAGE.md` section 4), so every
notrump context reads

```
after 1N (P) | (1x) 1N (P) | (1x) P (P) 1N (P) when !they.bid | systems_on
```

and Stayman, Jacoby and Texas transfers, minor-suit transfers,
super-accepts and the natural 2NT/3NT responses all apply after our
overcall, direct or balancing. The continuations were already written
against state (`when answered transfer(M)`), so they needed nothing.

BBA's meanings confirm it plays the same way: over `(1x) 1N (P)` the
corpus has 2♣ "Stayman" (7+), 2♦/2♥ "transfer", 3♣ "1N-3C transfer to
diamonds", 4♦ "Texas", 2NT "8-9" and 3NT "9-15".

| scenario | calls | auctions | contracts |
|---|---|---|---|
| We_Overcall_NT_then_Stayman | 75.3% → **87.3%** | 10.6% → 46.6% | 11.4% → 65.8% |
| We_Overcall_NT_then_Texas | 70.5% → **80.8%** | 0.0% → 18.6% | 0.0% → 22.8% |
| We_Overcall_NT_then_Jacoby | 70.0% → **79.5%** | 0.4% → 20.2% | 1.4% → 26.6% |
| We_Overcall_1N | 74.2% → **80.4%** | 15.0% → 26.2% | 15.4% → 33.4% |
| After_1x_1N | 73.3% → **75.7%** | 13.6% → 16.2% | 13.6% → 19.6% |

## Accepted differences from BBA

- BBA passes some 12-14 HCP hands with a five-card minor over a major
  opening that we overcall at the two level, and overcalls some that we
  pass. The line is not a rule in BBA; it evaluates the hand.
- BBA's weak jump overcall runs to 10 total points; ours stops at 9, the
  same as our weak two.
- BBA sometimes preempts to the three level on a six-card suit.

## Gaps and open questions

The balancing seat is `balancing.bid`; responsive doubles are
`responsive-doubles.bid`.

- Overcalls of a 1NT opening, of a weak two, and of two-level openings.
- Two-suited overcalls (Michaels, unusual notrump) are off on this card
  but the fields exist.
- Overcalls of a 1NT opening, of a weak two, and of two-level openings.

## Probed over 1C (2026-09-25, `probes/overcall-1C.toml`)

2,500 hands with a five-card or longer suit and 5-17 HCP over 1C
(Basic-Bridge, default style): 78% agreement before, 82% after.
- **Light overcalls.** BBA overcalls a five-card suit with 8-10 HCP
  whatever its quality (KT853, T976 in hearts...). Dropping our quality
  test, Rick's 2026-09-23 rule, costs **-4,835 IMPs** on the corpus.
  Rick's rule stands, strongly.
- **Weak jump overcalls:** BBA jumps with 10 HCP too. 4-10 instead of
  4-9: +258 (-167,116 -> -166,858).
- **The ceiling:** a six-card suit with 16 HCP has 18 total points, so
  we doubled; BBA overcalls. Capping the overcall at 17 HCP instead of
  17 total points: +229 (-166,858 -> -166,629). The strong any-shape
  double keeps 18 total points: moving it to HCP too cost 971 IMPs.

## Over their 1NT, natural; and judging competitive calls (2026-09-25)

We had no rules over an opposing 1NT. BBA on Basic-Bridge
(`probes/vs-1N.toml`, 2,500 hands) overcalls naturally:
- a five-card suit at the two level with 12+ total points;
- a seven-card preempt at the three level with 4-10.

Our rules now do the same when the card names no conventional defence:
91% agreement.

**Distance from par is the wrong yardstick here.** The corpus par figure
treats both sides alike: a competitive call that works (an overcall that
buys the contract, a preempt that pushes them too high) moves the
result away from par and counts as a loss. On the 237 boards this
change touches:

| Measure | IMPs |
|---|---|
| Distance from par | -825 |
| Double-dummy result to the side that overcalled | +1,007 |

`probes/tools/sideimps.py BASE.json VARIANT.json` computes the second.
Use it for competitive decisions; distance from par stays right for
uncontested auctions.

## For Rick: which yardstick for competitive calls? (2026-09-25)

Today's competitive A/B tests, re-run with both measures. Each row is a
variant against the rules as committed:

| Variant | Distance from par | IMPs to the side that acted |
|---|---|---|
| Reopening takeout double, (1x) X (2x) P P X | -1,741 | -2,478 |
| Advancer's takeout double after (1x) 1N (2x) | -449 | -555 |
| Overcaller's takeout double after (1x) 1y (2x) P P | -203 | -497 |
| Takeout double on 12 HCP (10 with a singleton) | -314 | +232 |
| Ours plus 10 HCP with a singleton | -380 | +163 |
| Two-level overcalls on three of the top five honours | -285 | +768 |
| **One-level overcalls without the quality test (your rule)** | **-4,924** | **+7,429** |
| Weak jump overcalls back to 4-9 (from 4-10) | -229 | +312 |
| One-level overcall cap back to 17 total points | -60 | -620 |
| Natural overcalls of 1NT (committed) | -825 | +1,007 |
| Undo "five-card major before the redouble" after (X) | -452 | +654 |
| Undo "support first over a major" after (X) | -338 | -541 |
| Undo the doubler's rebid when opener competes | -405 | -860 |
| Undo the preempt responses' vulnerability split | -57 | -204 |

Neither measure is clean:
- **Distance from par** counts any move away from par against us,
  including a competitive call that gains.
- **IMPs to the side that acted** plays our engine on both sides. Our
  engine rarely doubles for penalty, so a light overcall is seldom
  punished here as it would be at the table. That flatters bidding.

Kept where both measures agree: the three doubles are rejected, and the
17-HCP cap stays. The rest are for you, above all the quality test on
one-level overcalls: +7,429 by the second measure, -4,924 by the first.
A fair test probably needs penalty doubles on the defending side first.

## Two-level overcalls: the suit test at 15 HCP? (2026-09-27, for Rick)

Ticket basic-weak-2-b51: P 1S, KJ87.QJT76.K7.AJ (15 HCP) passes because
QJT76 fails the two-level suit test (six cards, two of the top three,
or four of the top five). BBA bids 2H; Rick: 2H is more descriptive
than pass. `probes/oc-2H-over-1S.toml` (400 hands with five hearts,
10-17 HCP, at None and EW): with a good suit BBA overcalls from 12;
with a weak one it splits at 12-14 (13 of 32 at 12) and always bids
from 15. Vulnerability makes no difference.

Waiving the suit test from N HCP, all three two-level overcalls:

| waived from | boards | par | side IMPs |
|---|---|---|---|
| 14 | 486 | −203 | −144 |
| **15** | 133 | **−37** | **+31** |
| 16 | 35 | −37 | +4 |

At 15 the yardsticks disagree, both near zero. Not adopted: they do
not agree, and par decides. **For Rick:** adopt at 15 on your judgment
and BBA's evidence, or keep the suit test. The ticket's alternative,
1NT with 5-4-2-2, runs against the 1NT overcall's evidence (no BBA 1NT
overcall held a five-card major), so it was not tried.
