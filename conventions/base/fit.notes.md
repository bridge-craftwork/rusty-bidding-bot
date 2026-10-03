# fit (`fit.bid`): notes

A major-suit fit, whatever the path: suit agreement, which both hands
know, and a fit only one hand can see yet. Cases: `fit.test`.

## Rick's ruling (2026-10-02)

> There should be some middle-ground rule, maybe judgement, that flags
> when we have found a major-suit fit, whatever the path.

His examples: 1M-1y-2M with a doubleton (done locally, PR #39); 1m-1H
(1S) X as a support double, after which responder knows the heart fit;
1m-1H-1NT with six hearts, where responder should "click 8-card major
fit" and use it for the strain. Then:

> The major suit fit may need to be more specific. In cases like the
> support double, or the 6-2 fit where opener has shown 6, responder
> knows about the fit before opener does. There is a separate concept of
> **suit agreement**, which most often applies to majors, but can be for
> minors in some situations. An example: 1NT-4D! (Texas). We already have
> an agreement that hearts will be trump, because opener promised at
> least 2 and responder promised at least 6. A subsequent 4NT by
> responder will be keycard for hearts. Compare with 1NT-2D!: we have not
> set hearts as trump, because opener might have 2 and responder might
> have 5. But if responder has 6, responder knows there is a fit. If the
> auction goes 1NT-2D! 2H-4NT, this is quantitative with 5 hearts. Opener
> will then decide if there is a fit, and can bid 5H to play (with 3-5
> H), pass to play (2H), or 6H or 6NT to accept the invite.

## The two concepts, as named conditions

| name | condition | meaning |
|---|---|---|
| `agreed_fit(x)` | `trump is x \| shown.x.min + partner.x.min >= 8` | suit agreement: public, both hands know it; x is trump for 4NT and control bids |
| `private_fit(x)` | `x + partner.x.min >= 8, shown.x.min + partner.x.min <= 7` | I know the fit, partner does not yet: it tells me where to play, and changes nothing about 4NT |

`shown_fit(x)` (slam-entry.bid), which the keycard and Blackwood asks
use to agree a suit on the way, is now `agreed_fit(x) | partner.x.min >=
6`: agreement, or partner's own six-card suit (a weak two, a jump
rebid), which a keycard ask can agree because the ask names it. No call
changes (the asks that read it run with no trump set).

**Where agreement already works** (checked 2026-10-02, 21GF random set,
1,000+ 4NT calls classified by the trump state and the shown minimums
before the call):

- Texas: `when answered texas(M)` asks for keycards in M (1NT-4D-4H-4NT).
  No trump is set, but the ask names the suit; `fit.test` has it.
- 1NT-2D-2H-4NT with five hearts is jacoby-transfers.bid's quantitative
  4NT (`quant_major`); the keycard ask that agrees a suit on the way
  needs `shown_fit`, which 5 + 2 does not make. `fit.test` has it.
- The support double: responder with five sets `trump=M`
  (support-doubles.bid), so 4NT later is keycard in M.
- Every keycard 4NT with no trump set was made with the suit agreed
  publicly (200 spades, 158 hearts) or over partner's shown six (62 over
  a weak two, 46 over a jump rebid); every quantitative 4NT (589) had no
  agreement. Consistent.
- **Inconsistent:** 12 boards in 100,000 where a minor was set as trump
  and a major was later agreed publicly (1S-2C-3C-4S-4NT asks in clubs).
  A candidate for minor-suit agreement giving way to a later major one;
  not changed (open question 2).
- **A bug, fixed:** after a negative double and opener's three-level
  answer, responder-rebids.bid's `jump(N)` "Game: 13+ balanced" was 4NT
  (1H (2S) X (P) 3H (P) 4NT); 3NT was meant (responder-rebids.notes.md,
  2026-10-02).

## The survey (2026-10-02)

Boards where our side played notrump or a minor although the two hands
held eight or more in a major, classed by what the hands knew before
each of their calls (`probes/tools/fit_survey.py`, knowledge from `rbb
explain-auction`). "Errors": our contract errors on those boards, IMPs
(par::table_errors). "DD gain": IMPs the major would have scored double
dummy instead, at game when we played game, else at the matching
partscore level.

| set | class | boards | errors | /bd | DD gain | /bd |
|---|---|---:|---:|---:|---:|---:|
| vanilla (95,558) | agreed (shown mins 8+) | 64 | 505 | 7.9 | 199 | 3.1 |
| | **private** (one hand knew) | **1,232** | 8,724 | 7.1 | **4,103** | 3.3 |
| | hidden (neither knew) | 8,430 | 52,029 | 6.2 | 19,941 | 2.4 |
| 21GF (100,000) | agreed | 64 | 437 | 6.8 | 144 | 2.3 |
| | **private** | **1,338** | 9,913 | 7.4 | **4,230** | 3.2 |
| | hidden | 8,546 | 51,848 | 6.1 | 20,284 | 2.4 |

The private boards by the last call made by a hand that knew (top
positions; the caller's length in brackets):

| set | position → call | boards | errors | DD gain |
|---|---|---:|---:|---:|
| vanilla | 1S → 3NT (1M-3NT with three spades) | 27 | 121 | 30 |
| | 1H (1S) → 1NT (three or four hearts) | 25 | 89 | 65 |
| | 1S (2H) X 3D → pass | 18 | 145 | 95 |
| | 1S 2C 2NT → 3NT (three spades) | 13 | 50 | 19 |
| | 1S 3NT → pass (six spades) | 13 | 99 | 22 |
| | 2C 2D 3NT → pass (six or seven hearts) | 9 | 93 | 3 |
| 21GF | 1S → 3NT (three spades) | 38 | 202 | 76 |
| | 1D 1S 1NT 2C(NMF) 2H → 3NT (six spades) | 28 | 169 | 57 |
| | 1H (1S) → 1NT | 24 | 88 | 64 |
| | 1NT 2C 2D → 3NT (six hearts) | 19 | 111 | 27 |
| | 1D 1H 1NT 2C 2NT → 3NT (six hearts) | 19 | 138 | 103 |
| | 1S 3NT → pass (six spades) | 18 | 132 | 29 |

The loss is spread thin: no position holds more than 40 boards, and a
third of the private boards are competitive. The positions after
partner's notrump (his rebid, or his answer to new minor forcing or
Stayman) are the largest family that a single state-keyed rule can
reach without claiming other modules' calls.

## The rule: game in a fit only I can see

At my second call or later, nothing asked of me, no suit agreed,
uncontested, partner's last call notrump (or partner has answered my
new minor forcing or Stayman): with `private_fit(M)` and 25 between us
counted by role (Rick, 2026-10-01: the long-trump hand declarer points,
the short hand support points, on partner's floor), bid 4M, priority 1,
above the sequence rules' 3NT and pass. Exclusions, each found on the
sets: partner's last bid in M (the sequence raises say more), the
strong 2NT (transfers), a slam call or slam exploration that applies,
four cards in the other major partner may hold (1D-1S-1NT-3H with 6-4:
−18 IMPs on 17 vanilla boards before it).

| set | boards changed | contract errors (even / odd) | distance from par (even / odd) | doubling |
|---|---:|---:|---:|---:|
| vanilla | 141 | **+204** (actor +132 / +65) | **+208** (+128 / +80) | −10 |
| corpus | 310 | **+395** (actor +258 / +156) | **+401** (+250 / +151) | −37 |
| 21GF | 220 | **+255** (actor +126 / +145) | **+272** (+127 / +145) | −27 |

**In competition** (2026-10-02, a second commit): the same rule once
the opponents have passed since (`!they.bid | rho.last=P`, as the
notrump ladder): 1S X P 2S P 3NT P 4H with six hearts facing the
doubler. Against the uncontested rule:

| set | boards changed | contract errors (even / odd) | distance from par (even / odd) | doubling |
|---|---:|---:|---:|---:|
| vanilla | 45 | **+113** (actor +83 / +31) | **+113** (+77 / +36) | −8 |
| corpus | 78 | **+275** (actor +166 / +138) | **+251** (+135 / +116) | +41 |
| 21GF | 49 | **+140** (actor +63 / +84) | **+132** (+59 / +73) | +1 |

**Over partner's notrump answer** (a third commit): the rule had kept
out of every position where a question of mine had been answered, so
partner's 3NT over my stopper ask or fourth suit (1D-2C-2S-3C-3NT with
four spades facing the reverse) was passed. Now any notrump call of
partner's counts; a suit answer still only to new minor forcing or
Stayman. Against the rule before it:

| set | boards changed | contract errors (even / odd) | distance from par (even / odd) | doubling |
|---|---:|---:|---:|---:|
| vanilla | 42 | **+141** (+33 / +108) | **+141** (+33 / +108) | −10 |
| corpus | 37 | **+98** (+102 / −4) | **+99** (+99 / +0) | −11 |
| 21GF | 26 | **+147** (+58 / +89) | **+130** (+49 / +81) | +13 |

The corpus odd half is level (−4, par 0), the rest gain. It adds
"contradicts earlier calls" boards: vanilla 185 → 199, corpus
392 → 402, 21GF 320 → 331. They are hands that described themselves
wrongly earlier (Stayman with six hearts, then 2NT) and now correct to
the major; the flag is right about the description, and the contract is
better.

Positive means fewer errors (`probes/tools/errors_diff.py`). No new
"no rule" or "passed a forcing" boards; contradictions 391 → 392 on the
corpus. "Calls read as a higher rule" +90 on vanilla: the sequence
modules' own "Game: 6+ M" over partner's 1NT/2NT (responder-rebids.bid
lines 36, 176), which now reads as this rule's 4M. The two say the same
(six of M, game values).

### Tried and dropped

- **Broad context** (any position after my first call, not only over
  partner's notrump): it claimed the 4M of the jump raise (1D-1H-4H read
  as this rule, so the raiser's range was lost and the slam with it:
  slam-entry.test) and the forcing 3S preference after 1S-2D-3C
  (responder-rebids.test). A priority-1 rule decides how partner reads
  the call wherever its context holds (docs/JUDGMENT-LAYER.md §4).
- **The invitation (3M with a private fit and partner limited).** With
  the broad context, vanilla contract errors −173 and par −322: it
  claimed partner's reading of responder's game-forcing 3H (1D-1S-1NT-3H,
  4H-5S) and of opener's jump shift (1S-1NT-3H), which then read as an
  invitation and were passed. Restricted to the long hand over a
  notrump rebid, +110 / +97 against +182 / +192 without it: it still
  claimed the sequence's own 3M invitations with a different count
  (opener stopped accepting). The sequence modules already invite in the
  standard positions (1x-1M-1NT-3M); dropped.
- **No slam bound at all:** slam-interest hands bid 4M and stopped
  (1D-1H-1NT with 16 and six hearts, a lost 7H). `fit_slam_room` sends
  them to the forcing calls below 3NT; over 3NT only the notrump slam
  values (32 HCP, over-3nt.bid) go elsewhere, so a weak hand with seven
  hearts still corrects 2C-2D-3NT to 4H. Corpus +215 → +273 with it.
- **Two sequence fixes the survey pointed to**, measured together with
  the negative-double 3NT (kept, responder-rebids.notes.md): 1M-3NT
  only with two cards in the major (`responses.bid`, so three bid a
  game raise or a new suit), first calls 1S-3NT → 2C/3S: vanilla −20,
  corpus +157, 21GF +57; and 1x (1y) 1NT not with three cards in
  opener's major below 10 points (`after-interference.bid`, raise
  instead): vanilla −29, corpus +54, 21GF −2. Neither gains on all
  three sets; not kept. On vanilla the 1M-3NT loss is slams: 1S-2C-...-4S
  where 1S-3NT-6S had found them.
- **A minor already agreed** (`we.trump is C | D` as well as none:
  1S-2C-3C-3D-3NT with three spades): corpus −22 contract errors, −22
  par on 8 boards. Not kept; minor agreement stays as it is (open
  question 2).
- **Rick's 5H answer to the quantitative 4NT** (1NT-2D-2H-4NT, opener
  minimum with three or more hearts): corpus −21 contract errors, −19
  par, on 33 boards, both halves negative; 21GF not run. With a minimum
  the 4NT contract makes where five of the major fails one trick in
  three. Not kept: par decides; opener passes 4NT with any minimum.

## Open

1. **Rick: the quantitative 4NT answer.** 5M with three or more hearts
   and a minimum is your ruling and lost on the corpus (above). Keep
   pass, or take 5M as the teaching answer at a small cost?
2. **Minor-suit agreement.** Candidates, not changed: a minor raise that
   set trump followed by a public major fit (12 boards per 100,000 on
   21GF: 4NT asks in the minor); the inverted-minor and 1NT-3m auctions
   already set trump in the minor. Should a later public major fit
   replace a minor trump?
3. **Positions this rule does not reach**, each a sequence question:
   1M-3NT with three cards in the major (27 vanilla, 38 21GF boards;
   the fix above lost on vanilla); 1x (1y) 1NT with three-card support
   (likewise); the competitive positions where partner's last call was
   a suit (1S (2H) X (P) 3D, responder passing with three spades).
4. **The invitation in a private fit** needs either a three-level call
   no module uses or the engine reading a fallback as the union of
   meanings (JUDGMENT-LAYER.md §4, option 2, language 2).

## Sources

- **Rick's rulings**, 2026-10-02 (quoted above), and his counting rule
  (2026-10-01, slam-entry.bid: declarer points for the long hand,
  support points for the short).
- **Measurements:** the survey (`probes/tools/fit_survey.py` over `rbb
  compare --json` and `rbb explain-auction`), vanilla SAYC random set
  (95,558 boards), 21GF random set (100,000), the corpus (170,633),
  errors and par by `probes/tools/errors_diff.py`, 2026-10-02.
- **Book practice:** a known eight-card major fit is played in the major
  rather than notrump, and a six-card suit facing a balanced partner is
  a fit: standard practice, not yet cited.
