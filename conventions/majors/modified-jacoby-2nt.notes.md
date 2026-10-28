# Modified Jacoby 2NT (`modified-jacoby-2nt.bid`)

Switched on by `major_openings.jacoby_2nt.modified` (no stock card, no
`.bbsa` key: BBA does not play it). It turns on the 2NT response itself
too (jacoby-2nt.bid, `jacoby | modified`), so a card need not also set
`jacoby_2nt.play`, and it turns standard Jacoby's answers off. Cases:
`modified-jacoby-2nt.test`. Asked for by Rick, 2026-10-28.

## What it plays (2026-10-28)

Larry Cohen's schedule, as the article writes it. Opener's rebid over
1M–2NT:

| Rebid | Meaning |
|---|---|
| 4M | minimum, 5-3-3-2, nothing extra (≤12 HCP) |
| 3♣ | any other minimum (≤14) |
| 3♦ | 15+, a singleton or void |
| 3♥ | 15+, 5-4-2-2 |
| 3♠ | 15+, six or more trumps, no singleton or void |
| 3NT | 15+, 5-3-3-2 |
| 4x | 15+, a good five-card side suit |

Responder's relays, only with slam interest (`slam_interest(M)`, the
slam-entry count): over 3♣, 3♦ asks for the minimum's shape (3♥ short,
3♠ 5-4-2-2, 3NT six trumps, 4M 5-3-3-2, 4x a side suit). Over 3♦, 3♥
asks where the shortness is (3♠ a void, then 3NT asks where: 4♣/4♦/4♥ =
clubs/diamonds/the other major; 3NT/4♣/4♦ a singleton in clubs, diamonds,
the other major). Over 3♥, 3♠ asks for the four-card suit (3NT clubs, 4♣
diamonds, 4♦ the other major). After the answers, the slam modules take
over as after standard Jacoby (trump is agreed from the 2NT).

The relays and answers sit at priority 15, above the control bids
(12-14): without that, responder's 3♥ relay was read as a control bid
and opener answered with control bids.

## Decisions (Rick to confirm)

- **Fixed suits over both majors.** Cohen writes one schedule with "our
  MAJOR" as the placeholder, and 3♥/3♠ as suits. We read them as fixed:
  over 1♥, 3♥ is the non-minimum 5-4-2-2 and 3♠ the six trumps, not
  "three of our major". Over 1♥ some answers land on 4♥ (the spade void
  after 3♠–3NT): game in our suit, which responder may pass.
- **Minimum is 14 HCP or fewer**, the standard module's line. Cohen
  gives no number ("12 points are statistically more likely than 15 or
  19"; 3NT "such as 18-19").
- **4M "dreck" is a 5-3-3-2 with 12 HCP or fewer.** Cohen: "the
  absolute worst dreck imaginable (always 5-3-3-2)". 13-14 5-3-3-2 bids
  3♣ and then 4M over the ask.
- **The direct 4x is non-minimum.** The article lists "4 new suit" in
  both schedules; the minimum one is under the 3♦ ask, so we read the
  direct one as 15+. Over 1♥, 4♠ is not a side-suit bid (above game).
- **Responder relays only with slam interest**; otherwise the slam
  modules decide (game or keycard). The first version also had an
  explicit 4M sign-off at priority 15 without slam interest; it cost
  141 slams the slam layer had reached (self A/B below).
- **Not written:** the further relays after a minimum's 3♥ or 3♠
  answer ("next step asks, same schedule"). Over interference after 1M
  the convention is off, as the 2NT rule already requires (`1M (P)`).
  A passed hand's 2NT is not Jacoby (jacoby-2nt.bid, `!passed_hand`).

## Interference (2026-10-28)

Cohen's interference section, which Rick called the main part
(2026-10-28): "If they double Asks or re-asks: XX = business, Pass =
S1, etc. If they bid after Ask or re-ask: X = Penalty, Pass = S1, etc.
(except when double = short as above)."

**Over a double or bid of an ask or re-ask** (the 3♦, 3♥ and 3♠ asks,
the 3NT void re-ask, and their double of 2NT, the first ask): pass is
the first step, then the bids in order from the cheapest (`step(1)`,
`step(2)`, ...: a call form added for this, LANGUAGE.md "Calls"), with
the same meanings as the uncontested schedule. Over their double, XX is
business; over their bid, X is penalty. Our readings where Cohen is
silent: business and penalty both need four of the suit with the ace
or king; over the doubled 2NT, XX is 15+ balanced (to play 2NT
redoubled); the side-suit answer (4x) has no step and those hands
answer by shape. The steps' pass is marked `artificial`, so the game
force keeps it legal and makes responder bid again.

**Over their bid on 2NT** (Cohen's own schedule; X is the exception
he notes):

| Opener | Meaning |
|---|---|
| X | a singleton or void in their suit |
| 3NT | 15+, balanced, the ace or king of their suit |
| new suit | natural (a good five-card suit) or a control (ace or king) |
| 4M | dead minimum, 6-3-2-2 |
| P | **forcing**: a flat hand, nothing special, can be an awful 5-3-3-2 |

The pass is forcing (Rick, 2026-10-28). Over it responder's X is
penalty (Cohen: "after which, responder's X = penalty"; Rick asked for
it explicitly): four of their suit, or three with the ace or king,
whatever responder's strength, since opener may hold nothing. Otherwise
4M, or the slam modules with slam interest.

Not covered: their cue of our major over 2NT; interference over opener's
answers (as opposed to the asks); the 3NT re-ask after the void was
shown under interference; their double of the 3NT re-ask (no business
XX: opener's void hand is not a notrump hand).

## Self A/B (2026-10-28)

`probes/self-ab.toml`, `modified-jacoby-2nt`: Jacoby_2N,
Jacoby_2N_Leveled, Jacoby_2N_4x_void, Jacoby_2N_4x_void_Leveled, 500
boards each, both sides, field off vs on.

- First version (with responder's 4M sign-off): contract errors −590,
  halves agree, z −4.4, **loses**. 141 boards went from 6M to 4M:
  `1M 2N 3C 4M`, where the base auction's 4M rebid had let responder
  bid keycard.
- Without the sign-off: contract errors −228 (halves +16 / −244,
  differ), distance from par −218, z −2.4, **neutral**. On the two plain
  scenarios alone: −82, halves −11/−71, z −1.4, **leans loss**.
- Where the loss is: a minimum with a good side suit (`P 2N P : 4D ->
  3C` −39 on 33 boards, `4C -> 3C` −27 on 21). Standard Jacoby shows
  the suit at any strength; here it comes out only when responder
  relays. Relaying more often (also at 15+ support points) changed 25
  more boards and no contract: the slam layer reaches the same
  contracts whatever the relays reveal, so the gain the convention
  promises needs the slam layer to read the shape answers. Reverted.
- Gains: `4H -> 3C` +75 on 272 boards (the 13-14 5-3-3-2 that standard
  Jacoby shows as "minimum"), `3N -> 3H` +13.
- The 4x_void scenarios are another Jacoby treatment (4x shows a void),
  which neither of our modules plays.

Tripwire: `compare --limit 50` on all scenarios, main's rules against
these with the field off: 0 of 17,105 boards changed.

## Sources

- Larry Cohen, "Modified Jacoby 2NT",
  https://www.larryco.com/bridge-articles/modified-jacoby-2nt (the
  schedule, the relays, "should not be used by a passed hand", the
  interference notes, quoted in "Interference"). The forcing pass and
  the step answers as the main part of interference: Rick, 2026-10-28. Our deviations: the decisions above.
- convention-card `spec/conventions/bidding_conventions/modified_jacoby_2nt.toml`
  (summary: 3♣ a minimum, responder can then ask).
