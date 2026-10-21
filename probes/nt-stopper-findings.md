# Notrump stoppers after the opponents have shown a suit

Measurement only; no rules changed. Date 2026-10-16.

Rick (2026-10-16): "When you bid NT you are promising a stopper in the
opponent's suit, if it has been bid or implied. For opening bids of 1♣
(3+) this is normally not required, and for opening bids of 1♦ (4+) some
play as required, others desired, others non-required. We might probe
BBA to see how it bids 1NT after opponents have shown a suit."

## Method

- `probes/nt_stopper_gen.py` writes 40 grid specs, `probes/nt-stopper-<position>.toml`
  (card 21GF-DEFAULT, IMPs, none vulnerable, South decides).
  Each spec has 200 random hands in the HCP range of the position, mostly
  balanced (4333/4432/5332) with the obvious alternatives filtered out:
  no five-card major for an overcall, no four-card unbid major for a responder.
  100 hands stop every suit the opponents showed and 100 do not.
  `stop` is eval.rs's: A, Kx, Qxx or Jxxx.
  The same seed gives the same hands on every card, so
  `--card 21GF-GIB --suffix=-gib --outdir <tmp>` and
  `--card Basic-Bridge --suffix=-basic` repeat the probe board for board.
- `rbb grid --pbs ../Practice-Bidding-Scenarios probes/nt-stopper-<name>.toml`
  runs each spec, and `probes/nt_stopper_tally.py [--suffix=-gib] [--detail NAME]` tallies it.
  Each hand is classed **stop** (stops all their suits), **half**
  (an honour but no stopper: Qx, Jxx, Jx, a bare K or Q) or **none**
  (no honour: xxx, xxxx...).
  The tally gives BBA's natural-NT count per class, ours beside it
  (`general.style=bba`), BBA's calls without a stopper, and whether BBA's
  meaning of the NT call (`bba_alert`) names a stopper.
- 21GF-GIB gave the same answers as 21GF-DEFAULT on every position, with one
  exception: adv-1H-2C with a stopper, 49 against 43. Basic-Bridge: see the end.

## BBA's behaviour (21GF-DEFAULT; 21GF-GIB identical)

NT = BBA's natural notrump calls in that class, out of the hands in it.
"Ours" = our engine's NT calls on the same hands.

| Position | stop: BBA (ours) | half: BBA (ours) | none: BBA (ours) | BBA's call without a stopper |
|---|---|---|---|---|
| **Direct overcall** (1♣) 1NT, 15-18 | 66/100 (100) | 0/32 (0) | **3/68** (0) | X 69, P 20, 1♦ 8 |
| (1♦) 1NT | 64/100 (99) | 1/53 (0) | 2/47 (0) | X 73, P 13, 2♣ 11 |
| (1♥) 1NT | 70/100 (84) | 0/35 | 0/65 | X 61, P 18, 2♦/2♣ |
| (1♠) 1NT | 63/100 (89) | 0/44 | 0/56 | X 65, P 17, 2♣/2♦ |
| (2♥) 2NT, 15-19 | 60/100 (65) | 0/51 | 0/49 | X 60, P 17, 3♣ 14 |
| (2♠) 2NT | 57/100 (65) | 0/47 | 0/53 | X 56, P 17, 3♦ 17 |
| (3♥) 3NT, 16-21 | 70/100 (60) | 0/48 | 0/52 | X 66, P 14, 4m 20 |
| **Balancing** (1♣) P (P) 1NT, 11-17 | 36/100 (67) | 0/36 | **0/64** | X 68, P 26, 1♦ 6 |
| (1♦) P (P) 1NT | 55/100 (68) | 0/30 | 0/70 | X 62, P 27, 2♣ 11 |
| (1♥) P (P) 1NT | 20/100 (54) | 0/33 | 0/67 | X 72, P 23 |
| (1♠) P (P) 1NT | 26/100 (54) | 0/42 | 0/58 | X 68, P 20 |
| **Responder** 1♣ (1♥), 6-15 | 61/100 (67) | 0/25 | 0/75 | P 43, 2♣ 38, 2♥ 12 |
| 1♣ (1♠) | 58/100 (73) | 0/21 | 0/79 | P 48, 2♣ 33 |
| 1♦ (1♠) | 43/100 (89) | 0/21 | 0/79 | P 80, 2♣ 20 |
| 1♦ (2♣) | 19/100 (5) | | 0/100 | P 100 (sic: even 15 HCP) |
| 1♥ (1♠) | 38/100 (26) | 0/27 | 0/73 | X 45, P 37 |
| 1♥ (2♣) | 13/100 (18) | 0/21 | 0/79 | P 55, X 29, 2♦ 16 |
| 1♠ (2♦) | 11/100 (18) | 0/24 | 0/76 | P 77, X 20 |
| 1♥ (2♥ Michaels ♠+m) | 54/100 (54) | 0/27 | 0/73 | P 92 |
| 1♠ (2♠ Michaels ♥+m) | 48/100 (43) | 0/33 | 0/67 | P 89 |
| 1♥ (2NT minors), both stopped | 1/100 (36) | 0/19 | 0/81 | P 50, X 42 (BBA doubles even with both stopped) |
| 1♣ (2♣ majors), both stopped | 39/100 (40) | 0/32 | 0/68 | P 39, 2♠ 23, 2♥ 16, X 11 |
| 1♦ (2♦ majors), both stopped | 33/100 (71) | 0/40 | 0/60 | P 60, 2♠ 40 |
| 1♦ (2♠ weak jump) | 31/100 (7) | 0/28 | 0/72 | P 80, 3♠ 20 (30 of the 31 "NT" are BBA's **Lebensohl 2NT**) |
| 1♥ (2♠) | 30/100 (6) | 0/23 | 0/77 | P 59, X 34 |
| 1♠ (3♥) 3NT, 10-17 | 54/100 (**0**: we double) | 0/29 | 0/71 | X 56, P 44 |
| **1♠ (3♥) X (P) 3♠ (P)**, Brill board 8 | **0/100 (91)** | **0/31 (31)** | **0/69 (57)** | 4♠ 48, P 29, 4♥ 23 |
| **Advancer** (1♦) 1♠ (P), 6-15 | 62/100 (60) | 0/18 | **12/82** (0) | P 83; 1NT 4, 2NT 6, 3NT 2 |
| (1♣) 1♥ (P) | 57/100 (76) | 0/15 | **12/85** (0) | P 84; 1NT 8, 2NT 4 |
| (1♥) 2♣ (P) | 43/100 (38) | 0/17 | 4/83 (0) | P 70, 2♦ 16, 3♦ 7 |
| (1♠) 2♥ (P) | 60/100 (42) | 0/26 | 3/74 (0) | P 88, 3♣ 6 |
| (1♦) X (P) | 63/100 (68) | 2/17 | **16/83** (0) | 2♦ (cue) 39, 2♣ 36, 1NT 18 |
| (1♥) X (P) | 69/100 (49) | 1/26 | **13/74** (0) | 2♥ (cue) 47, 2♣ 25, 2♦ 13 |
| (1♠) X (P) | 74/100 (41) | 3/24 | **14/76** (0) | 2♠ (cue) 34, 2♣ 32, 2♦ 17 |
| **Opener** 1♣ (1♠) P (P), 12-14/18-19 | 11/100 (11) | 0/27 | 0/73 | P 55, X 43 |
| 1♦ (1♥) P (P) | 16/100 (18) | 0/42 | 0/58 | X 58, P 35 |
| 1♣ (1♥) 1♠ (P), ≤3♠ | 25/100 (100) | 0/28 | **22/72** (0) | 2♠ 61, 1NT 22 |
| 1♦ (1♥) 1♠ (2♥) | 5/100 (0) | 0/34 | 0/66 | 2♠ 75, P 19 |
| 1♦ (1♠) X (P) | 58/100 (100) | 0/34 | 0/66 | 2♥ 56, 2♦ 23, 2♣ 19 |
| 1♣ (1♠) X (P) | 40/100 (100) | 0/28 | 0/72 | 2♥ 64, 2♣ 32 |

The "stop" column is low in some positions for reasons that have nothing to
do with stoppers: the hand is too weak, BBA raises (a 3-card raise of a free
1♠), or it doubles for penalty. The no-stopper columns are the measurement.

### What BBA does

1. **It never bids NT with a half stopper.** Qx, Jxx and a bare K or Q gave
   7 NT calls out of about 1000 hands, all of them in advancer's 1NT after
   partner's double.
2. **NT overcalls, balancing NT and responder's NT require a stopper, in every
   suit, 1♣ and 1♦ included.**
   - Direct 1NT over 1♣: 3/68 without one, all three holding xxx. Over 1♦:
     3/100 (xxx, xxx, Jxx).
   - Balancing 1NT over 1♣ or 1♦: 0/134.
   - Responder after an overcall, or after a Michaels or Unusual 2NT
     two-suiter (both suits checked where both are known): 0 without a stopper.
   - Without one it doubles, passes or bids a minor. After their Michaels
     with the minor unknown, it checks only the known major (resp-1H-2H:
     54/100 with ♠ stopped, 0/73 without).
3. **The one exception is the least lie.** In a few low-level positions, a
   balanced hand that has nothing else to say bids NT with **length but no
   honour** (xxx, xxxx, xxxxx), never with Qx:
   - advancer's 1NT (6-10) after partner's double: 13-16 of 74-83;
   - advancer after partner's overcall: 3-12 of 74-85, and in the
     (1♣) 1♥ case with xxxx clubs;
   - opener's 1NT rebid 1♣ (1♥) 1♠ (P) 1NT with 2 spades and nothing else:
     22/72, xxx 15 and xxxx 7.

   The exception is not specific to minors: it happens over their 1♥ and 1♠
   as often as over 1♣ and 1♦.
4. **Its meanings say so where the call is a response.** The NT meanings
   read "stopper !x" in every responder, advancer and opener position
   (e.g. "stopper !D | 1N … 6 to 10 total points").
   - The direct and balancing overcalls read "NT style" with no word on
     stoppers, but BBA plays them as if the meaning said so.
   - The 3NT overcall of 3♥ reads "stopper".
   - The 2NT after 1♦ (2♠) reads "Lebensohl after 1m": artificial.
5. **The Brill daily board 8 pattern, 1♠ (3♥) X (P) 3♠ (P): BBA never bids
   3NT**, even with ♥ stopped (0/100). It bids 4♠ with a doubleton, passes,
   or cue-bids 4♥.
   **We bid 3NT on 91/100 hands with a stopper, 31/31 with a half stopper
   and 57/69 with none** (on both cards). The rule is
   base/responder-rebids.bid:459, `cheapest(N) "Invitational: 11-12
   balanced"`. Over 3♠ the cheapest NT is 3NT, a game bid, and the rule has
   no stopper check. Rule :463 (3NT 13+) has no check either.
   `rbb call 52.T842.KQ96.AQ9 -a "1S 3H X Pass 3S Pass" -d N -c cards/bbsa/21GF-DEFAULT.bbsa`
   shows it.
   - Side finding: we read the negative double of 3♥ as "four or more in
     each unbid major". After 1♠ (3♥) the only unbid major is their suit.

### How BBA plays their 1♣ and 1♦

- BBA's 1♦ (21GF-DEFAULT and 21GF-GIB) is alerted "3+ !D". It opens 1♦ on
  three cards only with 4-4-3-2 (4♠ 4♥ 3♦ 2♣). Probed by hand: 4432 opens
  1♦, 4423/3433/4333 open 1♣, and 3343 opens 1♦.
  convention-card has `minor_openings.one_diamond.min_length` (2/3/4/5,
  default 3); the stock cards leave it unset.
- **Over both minors BBA plays "required" in every position probed.** The
  1♣ overcall exception that Rick describes is not in BBA: 3/68 without a
  stopper, against 0/65 over 1♥. It doubles instead (69 of the 100
  no-stopper hands).
  Its only notrump without a stopper is the least-lie exception (3 above),
  and that applies to every suit.

## Our rules (audit of `conventions/**/*.bid`)

Natural NT calls that can be chosen after the opponents have bid a suit:
**204**. Of these:

| | Rules |
|---|---|
| Check `stop()` of every suit shown | 128 |
| Partial: check some of their suits | 11 |
| No check, and none implied | 49 |
| Inherited: rely on partner's earlier NT, which cannot be checked because `partner.stop(x)` is not tracked | 16 |

Seven more come after a takeout double, which shows no suit. Systems-on
NT continuations, after our 1NT overcall, rest on the overcaller's
stopper.

In the first-call positions the probe measured, our engine bid NT without a
stopper only in the 1♠ (3♥) X (P) 3♠ (P) case. The gaps are in later rounds,
in wildcard (`*`) patterns, and in ask-answer rules, which cannot name
their suits.

**No stopper check** (file:line of the call; the context condition in short):

- Overcalls and balancing:
  - openings/vs-multi.bid:33, 52, 63: 2NT over their Multi. The major is unknown.
- Responder, first and later rounds:
  - base/responder-rebids.bid:459 (`cheapest(N)` 11-12) and :463 (3N 13+):
    after 1x (\*) X (\*) …. **The Brill board 8 rule.** :463 at priority -1
    outranks :472 (3N with stop(y), priority -2).
  - base/responder-rebids.bid:568, 577, 590, 592: 1x (\*) 1y (\*) 1z/2z (\*)
    → 1N / jump(N) / cheapest(N).
  - base/base.bid:134: the generic `3N "Game in notrump"` (game force, no
    trump, priority -40). It matches any contested game-forcing auction.
  - competitive/two-suited-overcalls.bid:695: `asked limit_raise(t)` →
    3N "Accepts: 14+" after their two-suiter. Neither of their suits is checked.
- Opener's rebids:
  - base/rebids.bid:840: 1x (\*) 2x (\*) → 3N "19+ balanced".
  - base/rebids.bid:890, 899, 904: 1x (\*) 1N (\*) → jump(N) / cheapest(N).
    In 1x (P) 1N (2y), partner's 1NT promised nothing in y.
  - base/rebids.bid:528, 556: `asked invite(M|m)` → 3N. Reachable after
    contested invitations (responder-rebids.bid:708-807, rebids.bid:763/835/888,
    support-doubles.bid:75).
  - majors/fit-showing-jumps.bid:153: 3N "16+ with {z} covered" checks
    partner's suit, not theirs.
- Overcaller and doubler:
  - competitive/advances.bid:228: `asked cue_raise(t)` → 3N "Accepts: 13+".
  - competitive/vs-preempts.bid:485: (2x) X (P) 3x (P) → 3N "No four-card
    major" at priority 1, with no stop(x). It outranks the "no stopper" 3♦ at :487.
- Our 1NT and they bid (notrump/nt-interference.bid unless noted):
  - :122: 1N (2y) → 3N 10+ when Lebensohl is off.
  - :311: 3N over their artificial 2y.
  - :369: 1N (2N) → 3N. Their two minors are not checked.
  - :151, :158: responder's 3N after the double.
  - :166, 226, 335, 374: opener's game in NT after a new suit at the three level.
  - :239, 243, 245: the 3♣-cue sequence.
  - :142, 406: "Nothing better" fallbacks.
  - lebensohl.bid:152 (opener accepts the invitation).
  - one-nt.bid:310, 312, 316: 1N (2C) with `notrump.transfers.vs_2c`.
  - rubensohl.bid:139, 146, 188, 193, 200, 207, 225, 234.
- By design: lebensohl.bid:65 (fast 3N denies), vs-preempts.bid:409 (fast
  3N), western-cue.bid:75 ("3NT anyway", priority -5).

**Partial** (some of their suits not checked):

- two-suited-overcalls.bid:552, 553, 571, 572: 2N/3N after their Michaels.
  Only the known major is checked; the minor is unknown, and BBA does the same.
- advances.bid:125: (1x) 1y (1z) 1N checks x, not z.
- takeout-double.bid:291: (1x) X (2y) 2N checks x, not y.
- snapdragon.bid:59: checks z, not x.
- after-interference.bid:475, 478, 496: 1x (1y) X (2z/3z/4z) checks y, not z.
- nt-interference.bid:355: checks z, not 2y.

**Inherited:**

- nt-interference.bid:171, 219, 255, 282
- lebensohl.bid:166
- vs-preempts.bid:376, 500
- two-suited-overcalls.bid:672
- one-nt.bid:276
- responder-rebids.bid:622, 623, 696, 730, 732, 823, 826. The context of
  823 and 826 allows `!they.bid | lho.last=P`.

Side finding: our advancer's 1NT after their takeout double turns systems
on. In 1♣ X P 1NT P, the doubler gets 2♣ Stayman, a cue bid of their own
suit (notrump/stayman.bid:17, through `nt_overcalls.direct.systems_on`).

**How the engine sees the opponents' suits.** Their calls are read by our own rules with
their card (`Engine::interpret`), so conventional calls fill their
knowledge (explain-auction):

- after 1♥ (2♥), RHO is ♠5-8, ♥0-3, and the minor is unknown;
- after 1♥ (2NT), ♦5-6 ♣5-6;
- after 1♣ (2♣), ♠5+ ♥5+;
- after a natural overcall, 5+ in the suit;
- after their 1♣ or 1♦ (3+), clubs or diamonds 3+;
- a takeout double sets no lengths.

So `lho.x >= 4 | rho.x >= 4` is "they have shown x", by a natural bid or by
convention, and it leaves out their 3+ minor openings. No single term
lists every suit they have shown, and `partner.stop(x)` is not tracked.

## Proposed design: one shared condition

In base/ (say base/stoppers.bid, or at the top of rebids.bid), with the
card parameters read from that module:

```
param  stop_1c = competitive.nt_stopper.one_club    default required
param  stop_1d = competitive.nt_stopper.one_diamond default required

# A suit they have shown: four or more known in one opponent (a natural
# overcall, a Michaels or Unusual 2NT suit, a preempt, a rebid), or
# their natural 1C/1D opening when the card requires a stopper there.
define their_suit(x) = lho.x >= 4 | rho.x >= 4
    | (x is C, lho.named(C) | rho.named(C), stop_1c is required)
    | (x is D, lho.named(D) | rho.named(D), stop_1d is required)

define held(x) = !their_suit(x) | stop(x)
define stops_their_suits = held(S), held(H), held(D), held(C)

# BBA's least lie: length (3+) without an honour, where no other call fits.
define nt_least_lie(x) = x >= 3
```

- **Every natural NT rule in a contested auction** gets
  `when stops_their_suits`. That includes the generic and wildcard rules
  (base.bid:134, rebids.bid:840/890-904, responder-rebids.bid:459-592) and
  the ask-answer rules (`asked invite`, `cue_raise`, `limit_raise`), which
  today cannot name their suits.
  - Pattern-variable rules that already write `stop(y)` can keep it or
    switch over. The define also covers the suit that the pattern does not
    name (the partial rules above).
  - The minor of a Michaels stays unknown, so it stays unchecked, as in BBA.
- **`desired`**: the NT call is allowed without the stopper, at a priority
  below the descriptive alternatives, through a second copy of the rule
  with `!stops_their_suits` and a lower priority. **`not_required`** drops
  the suit from `their_suit`.
- **The least lie** stays out of the define. BBA's exception (point 3
  above) appears only in advancer's 1NT after a double, advancer's NT after
  an overcall, and opener's minimum 1NT rebid. Model it as a fallback NT
  rule in those three positions, `x >= 3` in their suit, priority below
  every suit call.
- **Card fields.** convention-card has none today: `rg -i stop
  spec/fields.toml` finds none. For the batched convention-card PR:
  - `competitive.nt_stopper.one_club = required | desired | not_required`
  - `competitive.nt_stopper.one_diamond = required | desired | not_required`
  - label: "NT promises a stopper in their 1♣ / 1♦ opening"
  - skill: none
  - .bbsa: no key that we know of, so it stays unmapped.
- **Default: `required` for both**, from BBA: 3/68 and 3/100 without a
  stopper over 1♣ and 1♦, the same rate as over the majors. This is the
  decision taken; see question 1.

## Basic-Bridge

Basic-Bridge gives the same answers as 21GF-DEFAULT, give or take a few
hands, in every overcall, balancing, advancer and opener position, and for
responder after a natural overcall. 1♣ and 1♦ are required here too: 3/68
and 3/100 without a stopper.

The exception is responder after their cue bid or 2NT. Here BBA's
responder calls are "calculated bid", with no agreement behind them, and
some NT calls come without a stopper:

| Auction | NT without a stopper |
|---|---|
| 1♥ (2♥) | 9/73 |
| 1♠ (2♠) | 7/67 |
| 1♣ (2♣) | 10/68 |
| 1♥ (2NT) | 5/81 |

We bid no NT in those positions on that card. Nothing in this changes the
default.

## Questions for Rick (decisions taken meanwhile)

1. **1♣ default.** You say a stopper is normally not required over their
   1♣. BBA requires it over 1♣ and 1♦ alike in every position probed.
   *Decision: default `required` (BBA), with the card field to switch it
   to `not_required`. If you want `not_required` as our default for 1♣,
   say so and only the default changes.*
2. **`desired` semantics.** *Decision: NT is allowed without the stopper
   only as a fallback, at a priority below every descriptive call.* Is that
   what "desired" means to you?
3. **Least lie.** BBA bids 1NT with xxx or xxxx (never Qx) as advancer
   after a double, advancer after an overcall, and opener's minimum rebid,
   when nothing else fits. *Decision: copy it as a fallback in those three
   positions only, not in the shared define.*
4. **1♠ (3♥) X (P) 3♠ (P).** BBA never bids 3NT here, not even with ♥
   stopped. *Decision: our fix requires the stopper, keeps 3NT with one,
   and otherwise takes 4♠ or a 4♥ cue as BBA does.* That leaves the 3NT
   with a stopper to be judged by par, not by BBA.
