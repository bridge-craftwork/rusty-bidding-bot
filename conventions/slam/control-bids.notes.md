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
