# stayman (`stayman.bid`): notes

2C Stayman over 1NT, opener's answer, and responder's second call.
Cases: `stayman.test`.

## Guidance

- Stayman promises a four-card major and invitational values or better.
  With 5-4 in the majors, Stayman comes first (priority 1), ahead of the
  transfer, as the start of Smolen.
- **4-3-3-3 hands bid notrump and skip Stayman** (Rick, 2026-09-21), though
  BBA uses Stayman with them.
- Responder's second call: raise with a fit (3M invites, 4M is game, 6M is
  slam); without one, 2NT, 3NT or 6NT. Opener corrects 3NT to four of the
  other major with four cards (`choice-of-games.bid`).
- Opener's answer to 2NT uses a 4-4 fit when one is known
  (`one-nt.bid`, `asked nt_invite`).

## Evidence from BBA

Stayman scenario (21GF-DEFAULT, 500 boards): 92.4% of calls agree, 57.2%
identical auctions, 75.0% the same contract.

Choice of games after `1NT P 2C P 2M P 3NT P`: 210 of 211 corpus boards agree.

## Accepted differences from BBA

- **4-3-3-3 with a four-card major**: BBA uses Stayman on 755 corpus deals
  (21GF-DEFAULT) where we bid 2NT, 3NT, 4NT or 6NT. Rick: keep our rule.
- **2NT after Stayman**: BBA's shows about 7–8. With 7 HCP BBA bids 2C and
  then 2NT (for example `J543.964.J32.AJ3`), and BBA's opener passes it
  holding 17 (24 Stayman boards, for example `K64.KJ3.AK85.K32`). Ours
  shows 8–9, as after 1NT–2NT, and opener accepts with 17. Rick: our way is
  fine.

## Gaps (not built yet)

- **5-4 with a minor after 2D**: BBA bids 3C or 3D (for example
  `A82.QJ98.7.KJ973` bids 3C, then 4C). The bba style now does too
  ("BBA's 3C/3D after Stayman"); the default bids 3NT, which par prefers.
- **Smolen** (5-4 majors, game force after 2D), **Garbage Stayman** (weak
  hands that pass the answer), and Stayman with interference.
- **Slam after a fit**: with 18+ and a fit we jump to 6M. Keycard first
  would be better.

## Questions

- **Weak hands.** Of the corpus deals where BBA bids Stayman and we do not,
  176 are under 8 HCP and not 4-3-3-3: Garbage Stayman, or the lighter
  invitation above. Build Garbage Stayman as its own card option?

## Under interference (2026-09-23)

Two contexts at the foot of `stayman.bid` keep Stayman alive when RHO
acts, gated by the card switches `notrump.transfers.vs_double` and
`notrump.transfers.vs_2c`:

- over a **double** it is still 2♣, from a point below the invitational
  floor (`hcp>=24-partner.hcp.max`, so 7 opposite 15-17: finding the
  4-4 fit is also somewhere to run to);
- over a **2♣ overcall** the call is gone and **the double takes its
  place**, 8+ as usual.

Both **deny a five-card major**, unlike the uncontested 2♣: under
interference BBA transfers with 5-4 rather than looking for the other
major first, so Smolen's priority does not apply here. Opener's answers
are the same three calls, written once per context.

Evidence and numbers: `nt-interference.notes.md`.

## Support points and length points (2026-09-23)

Rick: "our rules should have HCP, total points and support points, and
decide which to use under which circumstances". Once opener's answer
shows the fit, responder also counts **support points**, `tp(x)`: HCP
plus shortness (doubleton 1, singleton 3, void 5, capped by the trumps).

- **6M with 33 support points** opposite opener's minimum
  (`AQ82.K.KQ943.J83`, 15 HCP + 3 = 18 opposite 15–17). This is most of
  the gain: 1NT–2♣–2♥ → 6♥ and 1NT–2♣–2♠ → 6♠ (and the same after
  2NT–3♣) where we bid game.
- **4M with 25 support points** when the total-point count says invite
  (`KJ82.Q7.Q943.J83`: 9 HCP + 1 = 10, game).

These are **extra** rules beside the total-point ones, not replacements.
Replacing the bands with `tp` outright gained less (+1,896) because
partner can no longer read a negative inference from a `tp` rule: after
1NT–2♣–2♥–3NT opener stopped knowing that responder has four spades
(157 boards of 1N lost the 4♠ correction, −392 IMPs). Kept as extras,
corpus par **+2,478 IMPs**, Basic_* +2, uncontested −3 (noise).

Also: opener's answer to the raise (`when asked invite(M)` in
`jacoby-transfers.bid`) accepts with 16 in the fit, see its notes.

## BBA's Stayman, probed (2026-09-24)

Rick asked how BBA treats a light hand with a four-card major. Probes on
Basic_Openers_Rebid board 47 and variants, bba-cli with the Basic-Bridge
card, all four hands fixed (`rbb probe`):

| North (responder) | HCP | tens | BBA |
|---|---|---|---|
| 94.AT83.94.K9876 (the board, South 4-3-4-2) | 7 | 1 | 2♣, then 2NT over 2♠ |
| 94.AT83.964.K987 (2-4-3-4) | 7 | 1 | 2♣; passes 2♥, 2NT over 2♠ |
| 94.A983.964.K987 (♥10 → ♥9) | 7 | 0 | 2♣, then 2NT |
| 942.A983.964.K87 (4-3-3-3) | 7 | 0 | 2♣, then 2NT |
| 942.AT83.964.K87 | 7 | 1 | 2♣, then 2NT |
| T42.AT83.964.K87 | 7 | 2 | 2♣, then 2NT |
| 942.A983.964.Q87 | 6 | 0 | Pass |

With a four-card major BBA bids Stayman on 7 HCP whatever the shape and
tens, passes a fitting answer, and otherwise bids 2NT. Without a major it
passes 1NT on 7 HCP 4-3-3-3 without a ten (10 of 10 corpus hands).

Opener facing that 2NT (North 942.A983.964.K87):

| South | HCP | tens | BBA |
|---|---|---|---|
| AK65.QJ7.AQ3.J32 | 17 | 0 | Pass |
| AK65.Q72.AJ3.QJ2 | 17 | 0 | Pass |
| AKT5.Q72.AJ3.QJ2 | 17 | 1 | Pass |
| AKT5.QT2.AJ3.QJ2 | 17 | 2 | 3NT |
| AKT5.QT2.AJT.QJ2 | 17 | 3 | 3NT |

So 2NT after Stayman is 7-8 to BBA and opener needs 18 counting tens,
where after a direct 1NT-2NT (8-9) it accepts with 17. Both sides move a
point; the target stays 25.

**Measured as our default and rejected on par** (corpus, net IMPs vs BBA):
Stayman from 7 with a major, 2NT 7-9, pass with 7 once the fit is found:
-1,001 (Basic -18). Nearly all of it is opener declining ordinary 8-9
invitations once 2NT can be 7 (1NT-2C-2D-2NT alone: -446 on 232 boards).
Without the pass once the fit is found: -1,078.

Also rejected: passing 2M after the fit is found with any invitational
hand (-113; pass better on 135 boards, the invitation on 123, but the
invitation's wins are games), or with 8 and inviting with 9 (-227).

**Kept as the BBA treatment** (`general.style = bba`): the light Stayman,
2NT 7-9 and the pass with the fit. Our default stays Stayman from 8 total
points with 2NT 8-9, because par prefers it.

## BBA treatment (2026-09-24)

Extends "BBA's Stayman, probed" above. Evidence: the corpus boards of
Basic-Bridge, 21GF-DEFAULT and 21GF-GIB (15-17 1NT, Stayman, Jacoby; all
love all, matchpoints), and `rbb probe` with Basic-Bridge both sides,
dealer S, the auction forced up to the decision, MP and IMP at love all
and all vulnerable (NS vulnerable, EW vulnerable on some). Probe output is
under `probes/` in the work sandbox (`fit`, `fitS`, `acc`, `nf`, `st`,
`t54`).

**Who bids Stayman.** Corpus, responder with a four-card major and no
six-card one, % 2♣:

| HCP | 4-3-3-3 | 4-4-3-2 | 4-4-4-1 | with a long minor |
|---|---|---|---|---|
| 4–6 | 0% | 0% | 0% | P 66–86% |
| 7 | 100% (42) | 100% (139) | 100% (165) | 100% (97) |
| 8–17 | 96–100% | 98–100% | 2♣ or a splinter (21GF cards) | 68–100% |

Tens do not move it: 6 HCP with tens passes, 7 HCP flat bids 2♣ (probe
`942.A983.964.K87` 2♣ at MP and IMP, love all and all vulnerable;
`942.A983.964.Q87` and `T42.AT83.T64.Q87` pass). **4-3-3-3 uses Stayman**
(12 Basic_Openers_Rebid boards and one Basic_What_To_Open where the
earlier bba rule bid 2NT or 3NT).

With **5-4 in the majors**: 5♠-4♥ bids 2♣ from 7 HCP (transfer below);
5♥-4♠ transfers with up to 9 HCP (2♦ 100% at 7, 8 and 9, 2♣ from 10),
then bids 2♠ (`jacoby-transfers.notes.md`). A six-card major transfers
(Basic-Bridge; the 21GF cards use Texas). 5-5 transfers to spades.

**The raise once the fit is found** (corpus, responder with four of
opener's major; % of BBA's calls):

| HCP | 4-3-3-3 | 4-4-3-2 | 4-4-4-1 | 5-4-3-1 (5 minor) |
|---|---|---|---|---|
| 7 | P 100% | P 100% | invite 76% | invite 72% |
| 8 | P 100% | invite 89% | invite 93% | invite 94% |
| 9 | P 44%, invite 56% | invite 94% | invite 54%, game 46% | invite 59% |
| 10 | invite 82% | game 84% | game 100% | game 92% |
| 11 | game 80% | game 97% | game 94% | game 100% |

Tens: 7 HCP with two tens passes 70%, 8 HCP passes 13–28% whatever the
tens: **tens do not count in the fit**. A logistic fit (907 and 1,170
boards)
gives, relative to a king at 3: A 4.0–4.2, Q 2.0, J 0.7–0.9, ten 0,
singleton +0.6–0.7, void +1.7–1.8, 4-3-3-3 −0.7 to −1.5, spades against
hearts +0.1; thresholds 7.5 (invite) and 9.7 (game).

Probes over 2♥ (the same over 2♠ where tried):

| North | HCP | MP | IMP |
|---|---|---|---|
| 542.Q876.K82.Q62 (4-3-3-3) | 7 | P | P |
| 542.K876.K82.Q62 | 8 | P | P |
| 542.K876.K82.K62 | 9 | 3♥ | 3♥ |
| 542.K876.A82.K62 | 10 | 3♥ | **4♥** |
| 542.A876.A82.K62 | 11 | 4♥ | 4♥ |
| 54.Q876.K862.Q62 (4-4-3-2) | 7 | P | P |
| 54.K876.K862.Q62 | 8 | 3♥ | 3♥ |
| 54.K876.K862.K62 | 9 | 3♥ | **4♥** |
| 54.K876.A862.K62 | 10 | 4♥ | 4♥ |
| 5.Q876.K862.Q652 (4-4-4-1) | 7 | 3♥ | 3♥ |
| 5.K876.K862.Q652 | 8 | 3♥ | **4♥** |
| 5.K876.K862.K652 | 9 | 4♥ | 4♥ |
| 5.K876.K82.Q8652 (1-4-3-5) | 8 | 3♥ | **4♥** |
| 5.K876.K82.K8652 | 9 | 4♥ | 4♥ |
| T4.QT76.K862.Q62 | 7+2 tens | P | P |
| T4.KT76.KT62.Q62 | 8+3 tens | 3♥ | 3♥ |
| K876.5.K862.Q652, over 2♠ | 8 | 3♠ | **4♠** |
| K876.5.K862.K652, over 2♠ | 9 | 4♠ | 4♠ |
| QJ85.6.Q982.KJ64, over 2♠ | 9 | 3♠ | **4♠** |
| 6.QJ85.Q982.KJ64, over 2♥ | 9 | 4♥ | 4♥ |
| K876.542.K82.Q62, over 2♠ | 8 | P | **3♠** |

Over 2♥ all vulnerable gave the same as love all at both scorings (over
2♠ only love all was tried). So: HCP, not
tens; 4-3-3-3 a point less; a singleton about half a point (it makes 7
an invitation, and 9 a game only sometimes: the same 9-count bid 4♥ with
hearts and 3♠ with spades); **at IMPs game a point earlier**, the
singleton then counting a full point.

**Opener's answer to the raise.** Corpus, by trump length: with four,
15 passes (99–100%), 16 accepts about half the time, 17 61–80%; with
five, 15 accepts 46–50%, 16 62–73%, 17 95–100%. Probe (South
AQ5.KJ72.A95.J83 family over `1NT P 2C P 2H P 3H P`): 15 and 16 with 0–2
tens pass, 17 bids 4♥; MP = IMP, love all = all vulnerable.

**After 2♥–2NT opener shows four spades** (corpus): with four spades
3♠ on 52 of 56 boards (15–17), pass on the rest, 4♠ never; without
them pass, except three 17-counts with two or three tens (3NT). Our bba 2NT used to leave responder's
spades unknown (opener passed: 8 Basic boards). Responder over the 3♠:
9 HCP 4♠ 62–73%, 8 about half, 7 pass.

**No fit after 2♦**, probes (North K542 with three hearts): 7, 8, 9,
8+2 tens and 9+1 ten bid 2NT at MP; 9+2 tens 3NT. At IMPs 8+2 tens
(4-3-3-3) and one of two 9-counts with a doubleton (4-3-2-4) bid 3NT;
the flat 9 and 9+1 ten still 2NT. Not modelled at IMPs (the
corpus has none, and one rule per answer and scoring would be needed).
Over 2♥ BBA sometimes passes with three hearts and 8 HCP (probe
`K542.Q76.K82.862`, and 20% of 8-count 4-3-3-3 in the corpus): not
modelled.

**What the rules model** (`stayman.bid`, rules under `when style is
bba`, the default ones `style is not bba`):

- 2♣: 7+ HCP (`hcp>=24-partner.hcp.max`) and a four-card major, any
  shape; not with a six-card major; with 5♥-4♠ only from 10 HCP.
- In the fit, fit points in HCP by shape against opener's range: game 10
  (9 with a void; 11 with 4-3-3-3), invite from 8 (7 with a singleton, 9
  with 4-3-3-3), pass below. At IMPs game from 9, the singleton counting
  1. Opener answers with a maximum (points ≥ `shown.hcp.max`, 17), or
  16 with a fifth trump (`ask=fit_invite(M)`).
- No fit: 2NT 7–9, 3NT game, **each rule for one answer** so that it
  denies the major it did not raise (the knowledge the default gets from
  negative inference). 4NT 16 HCP, 6NT 17 (before, 16–17 had no rule
  after 2♦ and passed: `K842.KQ5.AQ.QJ82`, Basic_What_To_Open 204; our
  default has the same gap).
- `1NT P 2C P 2H P 2NT P 3S P`: 4♠ with 9, else pass.

**Engine note: a `when` with a hand term leaks, on its own line too.**
The engine ANDs every `when` line of a rule into one condition, and in
interpretation keeps the rule possible whenever any part of it depends on
the caller's hand (`hand_dependent` in `eval.rs`). So `when !shape 4333
| ...` on one line and `when style is bba` on the next still made the
old bba 2♣ part of what 2♣ means under the default style: the default's
2♣ read as `(points>=8, ...) | (points>=8 | (hcp>=7, H=4 | S=4), ...)`.
It changed no call (the whole corpus is identical with and without the
rule), but the knowledge was wrong. The new rule has only `when style is
bba`, and the default 2♣ now reads as before. The reverse still holds:
default rules with hand terms in `when` (2♣'s `!shape 4333`, the 3♣ over
2NT, the support-point 4M's `strength<=invite`) widen what those calls
mean under `bba`: after 2♣ the bba side does not know responder has 7+
HCP. The fit answers are therefore keyed to opener's own maximum, not to
responder's range.

**Measured** (bba style, calls agreeing at the decision over the whole
corpus / Basic-Bridge boards only):

| decision | before | after |
|---|---|---|
| `1NT P` | 73.1% / 87.5% | 79.8% / 92.1% |
| `1NT P 2C P 2H P` | 59.5% / 70.6% | 73.0% / 88.2% |
| `1NT P 2C P 2S P` | 62.1% / 70.6% | 73.6% / 81.0% |
| `1NT P 2C P 2D P` | 50.2% / 76.2% | 55.6% / 77.8% |
| `1NT P 2C P 2H P 2NT P` | 81.0% / 52.4% | 98.4% / 100% |
| `1NT P 2C P 2H P 3H P` | 72.9% / 65.9% | 85.4% / 68.2% |
| `1NT P 2C P 2S P 3S P` | 71.8% / 61.8% | 78.0% / 65.8% |

Totals, all the notrump bba work (`measure2.sh --set general.style=bba`;
calls / identical auctions / same contract, par net IMPs):

| | before | after |
|---|---|---|
| corpus | 75.9% / 17.1% / 32.1% | 76.1% / 18.2% / 33.0%, par −181,088 |
| Basic_* | 85.4% / 43.0% / 50.7% | 85.7% / 44.5% / 51.9%, par −1,965 |
| Basic_* NS uncontested | 92.2% / 61.0% / 67.4% | 92.6% / 63.3% / 69.2%, par −967 |

Default (no `--set`): corpus 75.5% / 14.7% / 31.0%, par −172,679;
Basic_* −1,893; Basic_* NS −820, and every one of the 170,161 corpus
boards bids the same auction as before (ours and the replay).

**Where the default differs.** The default keeps 4-3-3-3 out of Stayman
(Rick, 2026-09-21), counts support points (shortness) in the fit, and
reads the no-fit 2NT/3NT denials from negative inference. It also has
the 16–17 gap after 2♦ noted above: a question for Rick (4NT with 16,
6NT with 17, as over 1NT?).

## Rick, 2026-09-24: 16-17 with no fit; 5-4 majors transfer

- After 1NT-2C-2D (or a major that does not fit) a balanced 16 bids a
  quantitative 4NT and 17 bids 6NT, as directly over 1NT (32 combined
  opposite a minimum). Before, the default had only 6NT at 18 total
  points, and 16-17 passed (Basic_What_To_Open 204).
- Five hearts and four spades below game values transfer and then bid a
  non-forcing 2S (BBA's treatment, now the default; jacoby-transfers.bid).
  Stayman keeps the other 5-4 hands.
- Passing 2NT with 4 HCP or less (BBA) is not adopted (-66 IMPs).

Together: -172,716 -> -168,198 IMPs against par on the corpus (+4,518),
Basic_* +13, uncontested +18.

## BBA's counts after Stayman (2026-09-24)

The `bba` style's continuations after 1NT-2C now use counts fitted to BBA's
answers on random hands, as with pass/2NT/3NT directly over 1NT
(one-nt.notes.md):

- **After 2D (no major fit):** pass vs 2NT, then 2NT vs 3NT. The measures are
  `bba_stay_nt_points` and `bba_stay_nt_imp_points`.
- **After 2H, responder with four hearts:** pass vs 3H, then 3H vs 4H. The
  measures are `bba_stay_raise_points`, `bba_stay_game_points`, and their
  `_imp` versions.
- **After 2S, responder with four spades:** the same, with the
  `bba_stay_sraise_*` and `bba_stay_sgame_*` measures. Spades need their own
  weights: the ♠/♥ asymmetry in BBA's 1NT (5-4-2-2 with four hearts, never
  four spades) carries through.

Each measure is linear, fitted by logistic regression on random hands that
include singletons and voids. The features are the honour counts, tens, a
charge for a doubleton ♠ or ♥ without A/K, shape class (4333, 4432, 5332,
5422), a five-card minor, and doubletons, singletons and voids. The counts
are scaled so the higher call starts at 8 (invite) or 10 (game).
Agreement on held-out hands:

| Decision | Agreement |
|---|---|
| No fit after 2D | about 91-92% |
| Heart fit | about 93-94% |
| Spade fit | 88-95% |

**Known miss:** K876.5.K862.Q652 after 2S at IMPs. BBA bids 4S; the count
says 3S (stayman.test).

**Gated to 15-17 openers.** The counts were fitted opposite BBA's 15-17
1NT. Applied elsewhere, they lost boards: after 2NT openings, after our 1NT
overcall, and after Precision's weaker 1NT. The count rules are therefore
gated `partner.hcp.min>=15, partner.hcp.max<=17`, and the earlier rules
stay in force for any other range (`partner.hcp.min<=14 |
partner.hcp.max>=18`). The first-response counts in one-nt.bid get the
same gate. No-fit continuations after 2H/2S keep the earlier rules, because
the counts were fitted after 2D only.

**Corpus (bba style):**
- Corpus: calls 76.1%, auction/contract 18.3% / 33.0% (unchanged), par
  -181,410 -> -181,330.
- Basic_*: 44.7% / 52.0%.
- Basic_* NS: 63.6% / 69.4%.
- Replay against the rules before: better on 127 boards, worse on 114.
- The default is unchanged (-167,558).

**Not modelled yet:** the losses after 2D on the other cards (21GF).
3C/3D: next section.

## BBA's 3C/3D after Stayman (2026-09-24)

Probed with random hands (`probes/stay-3m-*.toml`, 400-2,500 hands each;
`probes/grid_tally.py <name>` tallies a run). The grid now keeps BBA's
continuation and the meaning of its call (bba-cli `--all-meanings`), which
is how the meanings below were read.

**Responder, after 2D.** With a five-card minor and a four-card major,
BBA's 3C/3D ("9 to 25 total points, 5+ cards") is the game-going call
unless the hand wants 3NT:
- **Below 12 HCP:** 3NT with a diamond stopper (A, Kx, Qxx, Jxxx), 3m
  without one, from 9 HCP. It is the diamond stopper for both minors: five
  diamonds without an honour bid 3D, and a spade stopper changes nothing.
  Clubs: 99% of 1,050 hands; diamonds: 96% of 946.
- **From 12 HCP:** 3m whatever the stoppers (a slam try in effect).
- **Level:** below the 3m line, pass, 2NT or 3NT come from the counts
  already fitted (`bba_stay_nt_points`).
- **Six-card minors:** 4m from about 12 ("slam try"). Not modelled.

**Responder, after 2H/2S without the fit.** 3m from 10 HCP whatever the
stoppers; 2NT at 8, 9 split. 3NT almost never with a five-card minor.

**Opener over 3m.** A suit is *open* when:
- the other minor is unstopped; or
- after 2D only, a major doubleton is unstopped (responder's major is
  unknown).

An open three-card major is no reason. Opener:
- raises to 4m with a fit (four clubs, or three at 16+; three diamonds)
  and a suit open, or with a flat 4-3-3-3 minimum;
- bids 5m instead with 17;
- otherwise bids 3NT.

A minimum is 15 counting half a point per ten (two tens make it 3NT).
With a minimum:
- after 2D, bid 3D over 3C without four clubs, and 4C over 3D with five
  clubs;
- after 2H/2S, bid 3D over 3C with four diamonds;
- after 2S, bid 3H with a flat hand and three hearts, the 4-3 fit, since
  responder has four.

Agreement: 83% after 2D, 80-84% after 2H/2S.

**Responder again.**
- After the raise: 5m from 11 HCP (half the 11s pass), 4NT from about 15.
  Our rule is 5m from 11, with no Blackwood.
- After a minimum: 3NT up to 14, and a quantitative 4NT with 15.
- After 3D-4C (opener's five clubs): BBA bids its four-card major at the
  four level, into a 4-3 fit at best. Modelled as it stands.

**Agreement on the random hands, bba style, before -> after:**

| Probe | Before | After |
|---|---|---|
| Responder after 2D (MP) | 36% | 89% |
| Responder after 2H (MP) | — | 94% |
| Responder after 2S (MP) | — | 93% |
| Opener over 3C/3D | 0% | 83% |

"0%" means we passed 3m. The IMP columns are lower after 2H/2S: at IMPs
BBA bids 3NT with 8-9 where the earlier no-fit rules bid 2NT. That is a
gap in the no-fit level line, not in 3m.

**Corpus (bba style):** replayed calls closer to BBA on 399 boards and
further on 1. 127 auctions became identical to BBA's and none stopped
being identical. Par: -181,330 -> -181,678 (-348 IMPs). The loss is on
boards where our 3NT used to beat BBA's 3m auction and now copies it.

**Default: keeps 3NT (par decides).** On the corpus, where BBA bid 3m at
game level and we bid 3NT (99 boards on all cards), 3NT came out +72 IMPs
ahead. BBA's gains came on slam hands, where the default passes 2D or
bids 2NT on the 21GF cards. That is worth a look separately: a 3m slam
try from 12+ opposite 15-17 may pay.

## Six-card minors and the no-fit line after 2H/2S (2026-09-25)

**Six-card minors** (`probes/stay-4m-*.toml`, 1,200 hands per answer):
- **4m, BBA's slam try:** from 13 HCP at matchpoints, 12 at IMPs, after
  all three answers. Some 10-12 after 2D bid 4m as well; nothing I tried
  separates them.
- **3m below that:** from 9 HCP, even with a diamond stopper after 2D.
- **Weak, 7-8:** BBA plays the answer. It passes 2D with six diamonds,
  and passes 2H/2S with three of the major.
- **Opener over 4m:** 5m, or the other minor with five of it and two of
  responder's minor (4D over 4C, 5C over 4D). Some 16-17s with a
  four-card fit ask with 4NT; the 4NT is not modelled.
- **Responder over 5m:** 6m from 14. The decision fits only ~70% on
  HCP, shape and controls.

Agreement on the probes, before -> after:

| Answer | Matchpoints | IMPs |
|---|---|---|
| 2D | 32% -> 83% | 26% -> 81% |
| 2H | 53% -> 91% | 46% -> 88% |
| 2S | 54% -> 90% | 47% -> 86% |

**No fit after 2H/2S opposite 15-17** (`probes/stay-nofit-2H.toml`,
2S; 1,500 hands each, no five-card minor). Fitted with the same 15
features as the counts above, the lines come out as HCP with a ten worth
about half:

| Decision | Matchpoints | IMPs |
|---|---|---|
| 3NT | 10 HCP, or 9 with two tens | 9 with a ten or a singleton |
| Pass with three of the major | 7 without a ten | 7 with at most one ten |

Held-out agreement: 3NT line 97% (MP) and 91% (IMPs); pass line 92-93%.
IMP agreement overall: 77% -> 85% after 2H, 75% -> 86% after 2S. The
old rules stay for other opening ranges. A new hand measure, `tens`,
makes the lines expressible.

**Known miss:** 942.A983.964.K87 after 2S. BBA bids 2NT at matchpoints;
the count, and its fit, pass.

**Corpus (bba style):**
- Par: -181,678 -> -181,243.
- Replay: 133 boards closer to BBA, 8 further.
- Identical auctions: +45, -5.
- The default is unchanged.
