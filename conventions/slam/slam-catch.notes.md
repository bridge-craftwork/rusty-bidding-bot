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
| `blackwood.bid` | the ask (`sets ask=aces`) and the ace answers, card `slam.blackwood.standard` |
| `rkcb-1430.bid` | the ask (`sets ask=keycards(trump)`) and the keycard answers, card `slam.blackwood.rkcb_1430` |

The **ask condition is the same text in both files**, which is duplication
I would rather not have. It is there because the engine does not let one
module carry both: two rules for the same call at the same priority,
separated only by a card `param` in `when`, generate correctly but are
*interpreted* wrongly — partner reads the first of the two, whatever the
card says, and then the wrong answering module is the inactive one, so
partner passes the ask. That cost 85 passed-out 4NT contracts in a
whole-corpus run before the rules were split back into the two card-gated
modules. See "For Rick" below.

## The decision: when is slam worth investigating

Two branches, because the two cases are genuinely different questions.

### 1. Partner's hand is limited, and nothing has forced us to game

A raise, a limit raise, a preemptive raise. Partner has shown a top as
well as a bottom, and the only other call on offer is game itself, so the
ask has to outrank game and its own test carries the whole decision:

```
tp(trump) + partner.hcp.max >= 33, controls >= 7, losers <= 4
```

- **`tp(trump) + partner.hcp.max >= 33`** — 33 support points for the
  agreed suit if partner is at the top of what they have shown. Support
  points and not HCP, because the length of the fit and the shortness
  beside it are what produce twelve tricks. Partner's *maximum* rather
  than minimum because this is the hand that decides, and here partner's
  range is a narrow one (a raise is 6–9, a limit raise 10–12).
- **`controls >= 7`** — I hold the top cards myself. Facing a hand of
  queens and jacks the answer settles nothing.
- **`losers <= 4`** — the hand plays for twelve tricks opposite a
  minimum, so the aces really are the last question. This is the old
  discipline: never ask a question whose answer you cannot use. It is
  also what keeps five of the trump suit safe when the answer is bad, and
  therefore what keeps the sign-off reliable — no hand short of this ever
  leaves game.

### 2. We are already forced to game

A Jacoby 2NT, a splinter, a two-over-one, a strong 2♣. Partner has shown
game values with **no top to the range**, so `partner.hcp.max` is the rest
of the deck and the test above degenerates to "always". What is known
instead is partner's floor: a game force promises about 13 support points.
So the test is my own:

```
tp(trump) >= 18
```

18 + 13 = 31, the same number the `slam_try` hook uses, but counted in
support points for the agreed suit rather than in bare HCP. And here the
ask ranks *behind* the system's descriptive calls (priority −1 / 0, with
no `shows`, so descriptiveness 0): shortness, a second suit and a control
bid all say more than 4NT does. The ask is what is left when they are
used up.

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

Our thresholds fall out of `partner.hcp.max`: 9 after a simple raise, so
the test needs tp ≥ 24; 12 after a limit raise, so tp ≥ 21. Both sit on
BBA's own boundary, the first one point conservative.

`rbb probe` on the 21GF-DEFAULT card confirms the same hands (`1S Pass 3S
Pass`): 4NT on `AKQ952.A.AQ865.5` (4/4 boards) and `AK9874.A.AQJ54.9`
(2/2), 4♠ on `AKJ63.T5.Q8.AJT9` (2/2).

## Control bids

**Decided: not on a basic card.** BBA's card has a `Cue bid` switch and
the Basic-Bridge card has it **off** (`bba_passthrough."Cue bid" = 0`),
which is why BBA goes straight to 4NT there. The 21GF cards have it on,
and `rbb probe` shows BBA control-bidding on exactly the hands it
Blackwoods with on the basic card: after `1S Pass 2S Pass`,
`AKQ952.A.AQ865.5` bids **4♣** on 21GF-DEFAULT and **4NT** on
Basic-Bridge.

So control bids belong in this catch, as the branch that runs *before* the
ask whenever the card plays them, and on a basic card a control bid should
promise first-round control (an ace or a void) of the suit bid, cheapest
first, made only with slam values and a known fit. They are **not
written** here because there is no card field to switch them on:
`slam.control_bids` is a free-text field, and BBA's `Cue bid` key is
unmapped passthrough. See "For Rick".

## Accepted differences from BBA

- **We ask less often than BBA over a simple raise.** At 23 support points
  BBA is a coin-flip between 4♠ and 4NT; we bid game. On the 114 boards
  where BBA asks and we bid game we are **+41 IMPs against par**, so the
  conservative line is not costing anything measurable.
- **We do not control-bid** (above).
- **No queen ask.** After 4NT–5♣ ("0 or 4 aces") BBA asks for the trump
  queen with the next step and then bids the slam; we go straight to 6 of
  the trump suit when the ace count allows. 55 boards, −30 IMPs. Adding it
  needs a card field (below).

## Gaps and open questions

- The sign-off rule covers **majors only**: `4{trump}` is game in a major
  but not in a minor, and "3NT or five of the minor" is a choice-of-game
  question this piece did not settle.
- **Grand slams** are untouched; `grand_try` is still the 35-HCP
  placeholder.
- A game force that arrives *without* a trump suit (2♣ – 2NT – 3♠, where
  responder has three-card support and never raises) is outside the catch,
  because nothing agrees the suit. See "For Rick".

## For Rick (changes outside `slam/`)

1. **A card field for control bids.** `slam.control_bids` is text; a bool
   `slam.cue_bids.play` in `fields.toml`, with BBA's `Cue bid` key mapped
   to it in `bbsa-map.toml`, would let the control-bid branch be written
   and measured. Same for `slam.blackwood.queen_ask`, which exists in the
   registry but has no `.bbsa` mapping — BBA's key looks like `Blackwood
   without K and Q` (0 on the Basic-Bridge card, i.e. the queen ask is on).
2. **`shows tp(x)=10..12` records no points.** The limit raise in
   `base/responses.bid` shows `tp(S)=10..12, hcp<=12`, and the knowledge
   store comes out as `hcp 0-12`, points `0..14`: there is no floor at
   all. Same for the Jacoby 2NT (`tp(M)>=13` → `hcp 0-37`). That is why
   the test above has to use `partner.hcp.max`, which is the wrong end of
   the range for a five-level commitment. Either `tp` shows should feed
   the points ranges, or the raises should also show an HCP floor.
3. **Two rules for the same call in one module, separated only by a card
   `param` in `when`, are interpreted wrongly** (see "Where the rules
   live"). Generation honours the param; interpretation does not, and
   takes the first rule.
4. **`suit_strength` ignores shortness.** The bands use `suit_points`
   (HCP + ½ per card over four), so a 6-5-1-1 with 19 HCP counts 20½ and
   never reaches `slam_invite`. Every slam decision with a fit wants
   `tp(trump)`, which is why no rule here uses the bands.
5. **Responder never raises opener's suit after 2♣ – 2NT – 3M.** 38 boards
   at −280 IMPs on the Basic-Bridge subset, the largest slam-shaped
   divergence left. A raise in `base/strong-openings.bid` would agree the
   trump suit and hand the auction to this catch.
6. **The 1NT quantitative ladder** (46 boards, −230 IMPs, BBA 6NT where we
   bid 4NT) is a band question, not a trump-fit question: BBA's 6NT starts
   at 17 total points opposite 15–17, ours at 18, because the `slam` band
   is defined at 33 combined and BBA plays 32. That is in the engine.
