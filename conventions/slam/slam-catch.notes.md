# slam-catch (`slam-catch.bid`): notes

The common catch for auctions that have agreed a trump suit: stop in game,
or go past it. Rick's framing:

> Where we have a lot of drops it's because we don't have a common catch —
> there should be something that works once we have established a trump
> suit and are in a game force, for example. That would consider game vs
> slam, control bids, keycards.

Nothing here keys on an auction. The context is `we.trump is suit` plus
`we.forcing`, so the same rules serve a simple raise, a limit raise, a
Jacoby 2NT, a splinter, a two-over-one and a transfer.

## Where the rules live

| File | What it holds |
|---|---|
| `slam-catch.bid` | the sign-off: game in the agreed major when nothing else applies |
| `control-bids.bid` | the control-bid dialogue and the stop-in-game rule, card `slam.cue_bids.play` |
| `blackwood.bid` | the ask (`sets ask=aces`) and the ace answers, card `slam.blackwood.standard` |
| `rkcb-1430.bid` | the ask (`sets ask=keycards(trump)`) and the keycard answers, card `slam.blackwood.rkcb_1430` |

The **ask condition is the same text in both ask files**, which is
duplication I would rather not have. It is there because the engine does
not let one module carry both: two rules for the same call at the same
priority, separated only by a card `param` in `when`, generate correctly
but are *interpreted* wrongly — partner reads the first of the two,
whatever the card says, and then the wrong answering module is the
inactive one, so partner passes the ask. That cost 85 passed-out 4NT
contracts in a whole-corpus run before the rules were split back into the
two card-gated modules. See "For Rick" below.

## The decision: when is slam worth investigating

Rick's ruling, which the rules now express directly:

> Slam asks isn't about 18 — what has partner shown? It's more about
> **33+ combined points**, where in a suit contract the player with
> **fewer trumps** gets to switch to short-suit points — 1 for a
> doubleton, 3 for a singleton, 5 for a void, **capped by number of
> trumps held**. When the slam asker has **unprotected suits**, we will
> normally use **control bids before keycard**, and **stop in game if
> there is a suit unprotected in both hands**.

### 1. It is a combined-points decision

`we.tp(t)` is the partnership's support points with `t` as trump: my
exact count plus the range a raise recorded for partner. The engine's
`tp(x)` already counts 1 / 3 / 5 for a doubleton, singleton and void and
caps the total by the trump length held, which is the "player with fewer
trumps" rule as far as one hand can see it (see "For Rick").

This replaced `tp(trump) + partner.hcp.max`, which was a workaround from
before a raise showed support points at all.

### 2. Two branches, because partner's range is read from a different end

**Partner is limited** (a raise, a limit raise) and nothing has forced us
to game. Partner has shown a top as well as a bottom, and the only other
call on offer is game itself, so the ask carries the whole decision:

```
we.tp(trump).max >= 33, losers <= 4, <no two bare side suits>
```

- **33 against partner's maximum**, because this is the hand that
  decides and partner's range here is a narrow one (a raise is 6–9, a
  limit raise 10–12).
- **`losers <= 4`** — the hand plays for twelve tricks opposite a
  minimum, so the aces really are the last question. Never ask a
  question whose answer you cannot use.
- **No two bare side suits** replaces the old `controls >= 7`. A side
  suit is **bare** when I hold two or more cards in it and neither the
  ace nor the king: the first two tricks can go there. Counting controls
  never knew *which* suit was the problem; this does. One bare suit is
  allowed, because a raise facing it usually holds something there; two
  is where partner cannot cover both.

**We are already forced to game.** Partner's range has no top, so
`we.tp(trump).max` says nothing and what is known is partner's floor:

```
tp(trump) + partner.tp(trump).min >= 33 | tp(trump) >= 18
```

33 on partner's floor, with a fallback of my own 18 for the many
game-forcing auctions where the trump agreement recorded no support
points for partner at all (a two-over-one, a splinter, opener's own
raise). A game force is worth about 13, so 18 + 13 = 31 — the number the
old rule used, and the one the corpus still prefers. Here the ask ranks
*behind* the system's descriptive calls: shortness, a second suit and a
control bid all say more than 4NT does.

### 3. Control bids first, when a suit is unprotected

`control-bids.bid`, gated on `slam.cue_bids.play` (BBA's `Cue bid` key,
off on Basic-Bridge, on for 21GF). With one bare suit and the ask's own
values, the hand control-bids instead of asking, and comes back to 4NT
only once partner has shown a control above it. Everything about that —
the ladder, the styles, and the stop-in-game rule — is in
`control-bids.notes.md`.

## Evidence

BBA, Basic-Bridge card (`Rabbis_Rule`, `To_Finesse_Or_Not_To_Finesse`,
`Endplay_3rd_Round_Strip`, `Basic_Major`, re-bid with `--all-meanings`),
opener's second call, by support points (HCP + length over four +
shortness) and controls (A=2, K=1):

```
1S P 2S P   (raise, "6 to 10 total points")
  tp<=22            4S / 3S / P            — never 4NT
  tp 23 ctrl>=7     4S 89   4NT 87
  tp 24 ctrl>=7     4S 34   4NT 94
  tp 25 ctrl>=7             4NT 38
  any tp, ctrl<=6   never 4NT

1S P 3S P   (limit raise)
  tp<=20            4S / 3NT / P           — 4NT twice in 60
  tp 21 ctrl>=7     4S 3    4NT 5
  tp 22 ctrl>=7             4NT 16
  tp 23 ctrl>=7             4NT 22
  tp 24 ctrl>=7             4NT 15
```

The 4NT itself is alerted "Blackwood 0123, for ♠" and shows "18 to 20
total points" over the simple raise, "15 to 20" over the limit raise.
Shape matters more than HCP: of the 17–19 HCP hands, **every** 6-5-1-1
asked (136 of 136) and 6-4-2-1 split 72 asks to 193 games; no other shape
asked at all. That is exactly what support points measure.

Our thresholds fall out of partner's maximum: 9 after a simple raise, so
the test needs tp ≥ 24; 12 after a limit raise, so tp ≥ 21. Both sit on
BBA's own boundary, the first one point conservative.

**The bare-suit test, same data, by number of bare side suits:**

```
1S P 2S P    4NT: 163 with none bare,  56 with one,  0 with two
             4S : 155 with none bare,  73 with one,  0 with two
1S P 3S P    4NT:  44 with none bare,   9 with one,  0 with two
             4S :   3 with none bare,  12 with one,  1 with two
```

BBA never asks with two bare side suits, and the one hand it holds with
two bare suits bids game. `rbb probe` on the same card confirms the
boundary hand by hand: `AKQJT9.AK.Q54.32` (♦Q54 and ♣32 bare) is 4♠ for
both of us; give it the ♦K instead — `AKQJT9.A.KQ54.32` — and both bid
4NT. The old `controls >= 7` bid game on the second of those.

`rbb probe` on the 21GF-DEFAULT card confirms the point test too
(`1S Pass 3S Pass`): 4NT on `AKQ952.A.AQ865.5` and `AK9874.A.AQJ54.9`,
4♠ on `AKJ63.T5.Q8.AJT9`.

## Measured

Basic-Bridge subset (`compare --min-coverage 50`, 24 scenarios, 11 656
boards) and the whole corpus (342 scenarios, 170 161 boards), before →
after this rework. Both sides of the comparison use the same tree with
only `slam/` swapped, so nothing another module changed in the meantime
is in these numbers.

| | calls | identical | same contract | IMPs vs par | artificial | short fit |
|---|---|---|---|---|---|---|
| Basic subset, before | 85.2% | 41.1% | 54.6% | −5806 | 0 | 127 |
| Basic subset, after | 85.2% | 41.0% | 54.4% | **−5769** | 1 | 128 |
| Whole corpus, before | 75.1% | 15.0% | 28.8% | −213 199 | 41 | 3143 |
| Whole corpus, after | 75.1% | 15.0% | 28.7% | **−213 139** | 48 | 3148 |

And the 16 slam-shaped scenarios (`Slam_After_Major_Fit`, `Jacoby_2N`,
`Splinters`, `Serious`, `Slam_after_Transfer`, `Slam_after_Stayman`,
`Texas_Transfer`, `Grand_Slam_Force_2`, `Gavin_Weak_Splinter`,
`Two_Over_One`, `Minor_Game_Or_Slam`, `Slam_after_Preempt`,
`Splinters_By_Opener`, `Rabbis_Rule`, `To_Finesse_Or_Not_To_Finesse`,
`Basic_Major`; 8000 boards), which is where the control bids live:

| | calls | identical | same contract | IMPs vs par |
|---|---|---|---|---|
| before | 77.6% | 15.9% | 40.2% | −17 420 |
| after | 77.6% | 15.8% | 40.0% | −17 417 |

**It is a wash on the numbers.** The rules now say what Rick said they
should say, at no measurable cost; the 60 IMPs gained on the corpus and
the 0.1–0.2 points of contract agreement lost are both noise at this
size. What is genuinely new and untested by the corpus is the
stop-in-game rule, which fires only inside a control-bid dialogue and so
only on the 21GF cards.

## Accepted differences from BBA

- **We ask less often than BBA over a simple raise.** At 23 support
  points BBA is a coin-flip between 4♠ and 4NT; we bid game. On the 114
  boards where BBA asks and we bid game we are **+41 IMPs against par**.
- **We control-bid much less often than BBA.** BBA cue-bids on nearly
  every Jacoby 2NT auction that reaches the three level, whatever the
  values; we need the ask's values and a bare suit. See
  `control-bids.notes.md`.
- **No queen ask.** After 4NT–5♣ ("0 or 4 aces") BBA asks for the trump
  queen with the next step and then bids the slam; we go straight to 6 of
  the trump suit when the ace count allows. 55 boards, −30 IMPs. Adding
  it needs a card field (below).

## Gaps and open questions

- The sign-off rule and the control-bid dialogue cover **majors only**:
  `4{trump}` is game in a major but not in a minor, and "3NT or five of
  the minor" is a choice-of-game question this piece did not settle.
- **Grand slams** are untouched; `grand_try` is still the 35-HCP
  placeholder.
- A game force that arrives *without* a trump suit (2♣ – 2NT – 3♠, where
  responder has three-card support and never raises) is outside the
  catch, because nothing agrees the suit. See "For Rick".
- **Splinters are not in the rules at all.** `1S P 4C P` is "no rule in
  a live auction", so no trump suit is agreed and the whole catch — the
  ask, the control bids, the sign-off — never runs. BBA cue-bids 4♦ there
  and reaches slam; we pass out 4♣. That is `majors/`, not `slam/`.

## For Rick (changes outside `slam/`)

1. **`we.tp(t)` is dropped before the hand is looked at.**
   `crates/engine/src/eval.rs`, `hand_dependent`, treats `we.` as
   depending on my hand only for `hcp | keycards | points`. `we.tp` is
   not in that list, so a rule `when we.tp(trump).min >= 33` is judged
   with no hand at all (`tp` then returns 0–40), comes out false, and the
   candidate is removed before ranking — silently. `we.tp(trump).max`
   happens to survive because 40 + partner's floor always clears the bar.
   Adding `"tp"` to that list would let the game-force branch be written
   as `we.tp(trump).min >= 33` instead of
   `tp(trump) + partner.tp(trump).min >= 33`.
2. **Hand-dependent conditions in a *context* are silently dropped too.**
   `candidates()` evaluates a context's conditions with no hand and skips
   the whole entry unless they are known true, so `when ... has(A,C)` in
   a context matches nothing, with no error. It cost an afternoon. A
   `bid check` warning — "this context condition depends on the hand;
   put it on the rule" — would pay for itself.
3. **A rule `when` that mixes a hand term with an auction term is not
   filtered.** `when trump is not S, has(A,S) | S<=1` is judged
   hand-dependent as a whole, so the `trump is not S` half is never used
   to rule the candidate out, and on *interpretation* the rule can claim
   a call it could never have made. That is why every `trump is not x` in
   `control-bids.bid` is a nested context rather than part of the rule's
   `when`. LANGUAGE §12 says such a `when` "removes the candidate before
   ranking"; it only does when the whole condition is hand-independent.
4. **The trumps cap is applied to both hands.** `tp(x)` caps shortness by
   the trump length *of the hand counting*, so opener with six trumps and
   a singleton counts 3 and responder with three trumps and a singleton
   counts 3 as well, and `we.tp` adds both. Rick's rule is that only the
   hand with **fewer trumps** switches to short-suit points. Doing that
   needs the two counts compared, which is an engine change.
5. **Continuation-line syntax.** A context condition and a rule clause
   must each fit on one line: wrapping either is a parse error
   (`unexpected indentation`). The stop-in-game condition and the
   bare-suit gates are therefore 300-character lines that no one can
   read. A trailing continuation marker, or simply allowing a line that
   begins with `|` or `,` to continue the previous one, would fix it.
6. **Responder never raises opener's suit after 2♣ – 2NT – 3M.** 38
   boards at −280 IMPs on the Basic-Bridge subset, the largest
   slam-shaped divergence left. A raise in `base/strong-openings.bid`
   would agree the trump suit and hand the auction to this catch.
7. **The 1NT quantitative ladder** (46 boards, −230 IMPs, BBA 6NT where we
   bid 4NT) is a band question, not a trump-fit question: BBA's 6NT starts
   at 17 total points opposite 15–17, ours at 18, because the `slam` band
   is defined at 33 combined and BBA plays 32. That is in the engine.
8. **`slam.blackwood.queen_ask` has no `.bbsa` mapping.** BBA's key looks
   like `Blackwood without K and Q` (0 on the Basic-Bridge card, i.e. the
   queen ask is on). Worth 55 boards and −30 IMPs.
9. **`suit_strength` ignores shortness.** The bands use `suit_points`
   (HCP + ½ per card over four), so a 6-5-1-1 with 19 HCP counts 20½ and
   never reaches `slam_invite`. Every slam decision with a fit wants
   `tp(trump)`, which is why no rule here uses the bands.
