# control-bids (`control-bids.bid`): notes

Control bids: with a trump suit agreed and slam in the air, name a suit
you can control instead of asking for keycards straight away. Cases:
`control-bids.test`. The decision to start the dialogue at all is the
ask's own decision, written up in `slam-catch.notes.md`.

## Rick's rulings (2026-09-23)

- **A singleton is a control and protects the suit.** What a control bid
  guards against is the defenders cashing the ace and king of a side
  suit; keycard makes sure they cannot cash two aces, and controls check
  for the top two in one suit. So "bare" means two or more cards with
  neither the ace nor the king — a singleton or void is not bare.
- **Control-bid when there is room and we have an open suit**, rather
  than asking straight away.
- **The answerer keeps showing controls** for now, with no extra values
  required. Serious and Non-Serious 3NT will divide those hands later.

## Guidance

Rick's ruling, which this module and the ask both implement:

> Slam asks isn't about 18 — what has partner shown? It's more about
> **33+ combined points**, where in a suit contract the player with
> **fewer trumps** gets to switch to short-suit points — 1 for a
> doubleton, 3 for a singleton, 5 for a void, **capped by number of
> trumps held**. When the slam asker has **unprotected suits**, we will
> normally use **control bids before keycard**, and **stop in game if
> there is a suit unprotected in both hands**.

Three things follow, and this file holds the second and the third.

## The card

`slam.cue_bids.play` is BBA's own `Cue bid` key: **0 on Basic-Bridge, 1
on the 21GF cards**. That is why BBA blasts 4NT on the basic card and
cue-bids on the others, and it is why the whole module is `card
slam.cue_bids.play` — on Basic-Bridge nothing here is loaded and the ask
behaves exactly as before.

`slam.cue_bids.style` chooses what a control bid promises:

| Option | Promises |
|---|---|
| `first_round` | the ace or a void |
| `first_or_second_round` (default) | also the king or a singleton |

No `.bbsa` key maps to `style`, so every corpus card gets the default.
That default is right for BBA: of **1024 control bids** in
`Slam_After_Major_Fit`, `Jacoby_2N` and `Splinters` (21GF-DEFAULT,
`bba-cli --all-meanings`), every one held the ace, the king, a singleton
or a void of the suit named. Two held three small cards and **none held
a bare doubleton**, which is why a doubleton is not a control here
either, even under `first_or_second_round`.

## A bare suit

A side suit is **bare** when I hold two or more cards in it and neither
the ace nor the king: the first two tricks can go there. It is the same
test everywhere in `slam/` — it gates the ask (no two bare suits), it
sends a hand here rather than to 4NT (one bare suit), and it is what
"unprotected in both hands" is measured with.

## The ladder

The cheapest suit I can control, above the last bid: **3♠** (hearts
agreed, over partner's 3♥), then **4♣, 4♦, 4♥, 4♠**, skipping the trump
suit, whose four level is the sign-off. The priorities (14 down to 10)
put the calls in bidding order, so skipping a suit denies a control in
it, and the engine's ranking does the rest.

That is BBA's structure. From `1S P 2N P 3S P` it cue-bids 4♣ 23, 4♦ 9,
4♥ 6, 4♠ 2 and bids 3NT 9 times; from `1H P 2N P 3H P` it bids 3♠ 36
times; and with hearts agreed it goes on **past game** — `1H P 2N P 3C P
3H P` → 4♦, then → 4♠ — treating 4♥ as the sign-off rather than a step
on the ladder.

`4S` carries `!game_reached`. Without it, hearts agreed and partner
having already bid 4♥, we bid 4♠ as a "control bid" over partner's game:
107 passed-out artificial contracts on the slam scenarios before that
guard went in.

## Stop in game when a suit is unprotected in both hands

The rule at priority 20, above everything else in the dialogue.

Partner's control bid in `t` is partner's *cheapest* control, so partner
has denied every side suit below `t` that was available, and will never
bid one of them again. Two cases, and they are not symmetric:

- **Partner opened the dialogue** (`!answered control`): everything
  below `t` was available. A bare suit of mine below `t` is bare in both
  hands, so game is the limit.
- **Partner is answering my own control bid in `u`**: only the suits
  *between* `u` and `t` were available to partner, and those are the
  ones now known to be bare in both. My own bare suits below `u` are
  **not**: partner bid on knowing I had denied them, which promises
  cover there. Reading it symmetrically would make every dialogue I open
  end in game, since I bid my cheapest control and therefore always have
  a bare suit below it.

When the dialogue has already gone past 4{trump} the sign-off is
5{trump}, at priority 19, so the four level is taken whenever it is
still legal.

`P` at priority 2 closes the other side of it: partner bid game in the
agreed suit over my control bid, which is the sign-off, so I pass.

## What we could not express

- **A suit below the first control bid of the whole dialogue.** Nobody
  can name it — it was never available — so neither the stop rule nor
  anything else can tell whether it is covered. The keycard ask has to
  settle it, and it only counts aces. This is the real hole, and it is
  the same hole a human cue-bidding pair has.
- **Partner's holdings.** `partner.has(A,x)` and `partner.stop(x)` are
  always *unknown*: the knowledge store keeps `has` and `stop` as
  constraints but answers no questions about them (LANGUAGE §12). So
  everything partner knows about controls has to travel as the
  control-bid state (`ask=control(x)`), not as knowledge, and only
  partner's **latest** control bid is remembered, not the whole history.
  A dialogue longer than two rounds loses its earlier steps.
- **"The cheapest control"** is written as five calls with descending
  priorities rather than `cheapest(x)` with a suit variable, because
  `cheapest(x)` needs `x` bound and an unbound suit variable in a call
  expands to four *candidates* whose order is then decided by
  descriptiveness — which for four near-identical `shows` is sample
  noise, not bidding order.
- **Minors.** The module is majors only (`trump is H | trump is S`), like
  the sign-off in `slam-catch.bid`: five of a minor is a different
  conversation and "3NT or five of the minor" was never settled.

## Accepted differences from BBA

- **BBA control-bids on far more hands than we do.** After `1S P 2N P 3S
  P` it cue-bids on 40 of 49 boards — essentially whenever a Jacoby 2NT
  auction reaches the three level, whatever the values. We control-bid
  only with the ask's own values (33 combined support points) *and* a
  bare suit, which is Rick's ruling. On the slam scenarios this is worth
  about 0.3 points of call agreement and nothing measurable in IMPs.
- **BBA continues the ladder where we sign off, and the reverse.** After
  `1H P 2N P 3N P 4C P` BBA bids 4♥ with a minimum balanced hand; we bid
  the next control if we hold one. Gating the *answering* ladder on the
  same values made the whole thing worse (contract agreement 40.0% →
  39.5%, −330 IMPs on the slam scenarios), so it is left alone.
- **BBA does not always read game in the agreed suit as a sign-off.**
  `Jacoby_2N` board 394: 1♥ – 2NT – 3♥ – 3♠ – 4♥, and BBA's next call is
  4NT, reaching 6♥. Our `P` rule treats 4♥ there as the stop. That is
  Rick's rule, and the cost is real but small.

## Open questions for Rick

1. **Does a singleton count as unprotected?** Here it does not: a small
   singleton is one fast loser, and under `first_or_second_round` it is
   a control. So `AKQ952.A.AQ865.5` has no bare suit and goes straight
   to 4NT where BBA cue-bids 4♣.
2. **Is one bare suit facing a limited raise really enough to ask?** The
   ask allows one, and BBA's Basic-Bridge behaviour agrees (272 boards
   asked with none or one, never with two), but it is the loosest of the
   three tests.
3. **Should the answerer need values to continue the ladder?** BBA's
   answerer signs off in game with a minimum; ours shows any control it
   holds. Measuring says leave it, bridge says otherwise.

## For Rick: control bids stop in game too often (2026-09-25)

A removal screen of the convention modules found control-bids.bid net
negative: without it, +427 by distance and +890 IMPs to the bidding side
(451 boards). Almost all of it is game against slam:

| With control bids | Without | Boards | IMPs, for the version without |
|---|---|---|---|
| 4H | 6H | 65 | +559 |
| 4S | 6S | 45 | +307 |
| 5H | 6H | 17 | +164 |

Some of these stops are right, e.g. 1N_5M_and_6m 84, where 6H goes
down on the club lead. The rest stop in game with slam making, which
points at "stop in game if a suit is unprotected in both hands" firing
too often. Suspects: what counts as bare, and whether a control bid by
partner is read as covering the suit. Not changed, as it is your
ruling; worth a look with the examples.

## Fixed: denial by bypassing (2026-09-25, Rick: "that is the actual bridge meaning")

**The cause.** The stop-in-game rule read partner's control bid in `t` as
denying every side suit below `t`. A control bid denies only the suits
it bypassed: those a control bid was available in, between the previous
bid and partner's, on the ladder. Two failures:
- Over our 3H, partner's 3S denies nothing. We read it as denying both
  minors (1NT-2D-2H-3C-3H-3S-4H on 18 boards; responder held the
  minors).
- After 1S-2NT-3C, 4C bypasses nothing: the ladder starts at 4C when
  spades are trumps.

**The engine.** New terms: `partner.bypassed(x)`, optionally with the
ladder's start, `partner.bypassed(x, 3S)`; and `me.last`
(docs/LANGUAGE.md). The stop rule now asks whether I am bare in a suit
partner bypassed. 3S is on the ladder only directly over our 3H.

**Two follow-ons:**
- **Going past game** with a control bid (4S over hearts) needs my own
  cover in each minor partner has not shown. Without it, 4H lets
  partner continue. Test 37 keeps its 4H for this reason.
- **Asking when everything is covered:** when partner opened the
  dialogue and every side suit is covered, by me or by partner's
  control bid, ask for keycards (blackwood.bid, rkcb-1430.bid). Before,
  that hand signed off in game after Jacoby 2NT.

**Results against the rules before** (146 boards):
- Distance from par: +460.
- IMPs to the bidding side: +739.
- Corpus par: -156,610 -> -156,150.

Against no control bids at all, the gap fell from +427 / +890 to +45 /
+279 (distance / to the side). What is left is the ladder's own limit:
a suit below the first control bid of the dialogue can't be shown, so a
hand that holds it may still stop in game.

## Rewritten to Rick's description (2026-09-25)

Rick: "In a GF and a suit agreed on, new non-jump bids below game are
control bids. You bid the cheapest control you can show; any you skip
over you are denying. Partner then shows the cheapest control they
have. If either player skips (denies) a control in a suit and the other
continues control bidding, they are promising a control in the skipped
suit. Standard control bids are aces and voids; first- and second-round
controls (A, K, singleton, void) are more common now. Control bidding
past game shows first-round control. We don't normally show a singleton
or void in partner's suit." Also: control bids start only once a suit is
agreed.

What the module does now:
- **Every non-jump new suit below game is on the ladder** (`cheapest(x)`,
  `under_game(x)`). It no longer starts above three of the trump suit.
  The cheapest control held is chosen (`prefer 0 - cheapest_rank(x)`).
- **The engine keeps the dialogue.** Each ladder call records what it
  skipped (`denied`) and named (`cued`). Continuing promises a control
  in every suit partner has denied; with no call that keeps the
  promise, the hand signs off in game. That replaces the old "bare in
  both hands" rule.
- **Past game, first-round controls only.**
- **No shortness control in a suit partner has shown** (four or more).
- **`style`:** `first_round` or `first_or_second_round` (default), as
  before.
- **Order:** a control bid below game; the keycard ask once every side
  suit is covered (mine, cued by partner, or denied by me and promised
  by partner); a control bid past game; the sign-off.

Test 25 changed: KJ.AJ864.Q7.AJ72 after 1H-2NT-3H-4D now asks with
4NT. 4S past game would promise first-round control, and every suit is
covered.

**Results:**

| Against | Distance from par | IMPs to the bidding side |
|---|---|---|
| The bypassed fix (earlier today) | +159 | +236 |
| No control bids at all | control bids now ahead by 114 | 41 behind (was 427 / 890 behind this morning) |

What is left is mostly spade Jacoby auctions (1S-2NT, 118 boards).

## Shared conditions named in slam-entry.bid (2026-09-30)

The long conditions of control-bids.bid (up to 1,300 characters a line) are now
named conditions in `slam/slam-entry.bid`: `slam_values`,
`slam_values_limited`, `controlled(x)`, `first_round(x)`,
`side_suit_uncontrolled`, `two_bare_suits`, `denials_promised`,
`all_covered` and their variants. Each is exactly the condition it
replaced; a definition reads `style` from slam-entry.bid, which names the
same card field and default (`slam.cue_bids.style`,
`first_or_second_round`). The full corpus bids identically
(docs/JUDGMENT-LAYER.md, Phase 0). `two_bare_suits` uses the new
`bare(x)` (a side suit of 2+ cards without the ace or king), which, like
`has`, is not recorded when a call is read, so the ask's negative
inference is unchanged; `bare_suits <= 1` would say the same but would
add that denial, a change to measure on its own.

## Sources

- **The convention:** control (cue) bids once a suit is agreed in a game
  force, as Rick described them on 2026-09-25 ("Rewritten to Rick's
  description": the cheapest control first, skipping denies, past game
  shows first-round control). Beyond that description, standard practice,
  not yet cited to a book or article.
- **Rick's rulings:** 2026-09-23 (a singleton is a control; control-bid
  when there is room and an open suit; the answerer keeps showing
  controls; 33+ combined points; stop in game when a suit is unprotected
  in both hands); 2026-09-25 (denial by bypassing is "the actual bridge
  meaning", and the description quoted above).
- **BBA evidence:** 1,024 control bids in Slam_After_Major_Fit, Jacoby_2N
  and Splinters on 21GF-DEFAULT, read with `bba-cli --all-meanings` (what
  a control promises); BBA's ladder after `1S P 2N P 3S P` and
  `1H P 2N P 3H P` ("The ladder"). No `probes/*.toml` spec yet.
- **Corpus measurements:** the removal screen, the bypass fix and the
  rewrite (2026-09-25).
- **Where we differ:** BBA control-bids on far more hands, continues the
  ladder with a minimum, and sometimes bids on over game in the agreed
  suit ("Accepted differences from BBA").
