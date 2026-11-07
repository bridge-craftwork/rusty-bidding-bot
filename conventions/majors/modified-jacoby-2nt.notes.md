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
  141 slams the slam layer had reached (self A/B below). Over the
  minimum 3♣ the gate is wider since 2026-11-06: 16 support points
  also ask ("Relays after a minimum", below).
- **Not written:** the relays after the minimum's 4x and 3NT answers.
  Over interference after 1M
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

## Relays after a minimum, then keycard (Rick, 2026-11-06)

Bridge-Classroom reports #83 and #84 (bridge-classroom-bug-artifacts,
boards 5 and 7; Rick South, Rusty the other seats, on Rick's card: 2/1,
modified Jacoby, Kokish):

- #83: N QJ54.KQJ7.K98.KJ (16 HCP), S -.AT542.AQ764.A96.
  1♥–2NT–3♣–4♥ (slam-catch's sign-off); 13 tricks in hearts.
  Rick: "Rusty north should ask about south shape and learn of spade
  void then try keycard."
- #84: N J8732.AT63.A.KJ5 (13), S -.KQ872.KJT98.A32. The same auction;
  13 tricks. Rick: "Same as prior issue."

Responder's slam count opposite a minimum is 17 + 12 = 29 and 16 + 12,
below the 31 that `slam_interest` asks for, so he signed off. A
minimum with the right shortness opposite no wasted values makes slam
that no count on its floor sees; Cohen's guidance is only "If
responder still has slam interest, he can ask further". Decisions:

- **The gate over 3♣:** `slam_interest(M)` or 16 support points
  (`mj_try`; since 2026-11-07 every responder relays, below). 15 or fewer sign off (the 14-count with a doubleton in
  the cases still bids 4M).
- **The relays after a minimum** (Cohen: "next step asks, using same
  schedule"): one step above the non-minimum schedule. Over 3♥ (short),
  3♠ asks: 3NT a void (4♣ asks where: 4♦ clubs, 4♥ diamonds, 4♠ the
  other major), 4♣/4♦/4♥ a singleton club, diamond, other major. Over
  3♠ (5-4-2-2), 3NT would ask: 4♣/4♦/4♥ four clubs, diamonds, other
  major. Opener answers it, but our responder does not ask: nothing
  reads the four-card suit, and relaying lost 53 IMPs on 34 boards
  against going on at once. Over 1♥ the spade void lands on 4♠, past
  game; without keycard values responder goes back to 5♥.
- **The relay asks are `mj_relay`**, no longer `relay`:
  minor-transfers.bid's `when answered relay` ("Clubs: play here")
  read our relays, and responder passed opener's artificial 4♥.
- **A void goes ahead of the good side suit** in the answer to 3♦ (the
  4x answer now needs `shortest>=1`): a void is the slam news. #83's
  South (a spade void and AQ764 of diamonds) answered 4♦ before.
- **Keycard once the shape is placed** (the count superseded on
  2026-11-07, below) (`answered mj_relay`, any of
  the modified-Jacoby relays, minimum or not): responder asks (RKCB on
  the card, else Blackwood) with 13 HCP outside a king or queen
  opposite opener's singleton (`mj_working`: K 3, Q 2 taken off; the
  ace takes its trick, a jack is ignored), the ace taken off too
  opposite a void; or 14 HCP with the ace, king or queen of opener's
  good five-card side suit (the 4x answer). Less is left to the slam
  modules. Rick's words have keycard after the void; the 13 is our line
  (#84 is exactly 13). Not after the 5-4-2-2's four-card suit: that
  branch lost 223 IMPs on 46 boards in the first run (slams on a
  4-4 side fit with no ruffing value, half of them down).

Replays (bid-pbn, all four seats Rusty, 21GF-GIB with modified Jacoby
and Kokish, the stand-in for Rick's card):

- #83: 1♥–2NT–3♣–3♦–3♥–3♠–3NT–4♣–4♠ (spade void)–4NT–5♦–5♥–6♥.
- #84: the same to 4♠, then 4NT–5♠ (two and the queen)–6♥.

**Measured 2026-11-06** (compare --par-all on Jacoby_2N,
Jacoby_2N_Leveled, Jacoby_2N_4x_void, Jacoby_2N_4x_void_Leveled with
`major_openings.jacoby_2nt.modified=true`, main's rules against these;
2,000 boards; errors_diff): 526 boards changed; the bidding side's
contract errors **+76 IMPs** (halves +3 / +73, agree); distance from
par +66; "short of slam" 370 → 331 boards (4,495 → 3,993 IMPs). By first difference:
`P 3C P : 4H -> 3D` +49 on 152, `4S -> 3D` +39 on 149 (the wider
relay); `4N -> 3D` −11 on 50 (relaying where keycard came at once).
By the shortness found: voids gained most, singletons about even. The
other side's doubling errors (−65) are our defenders not doubling and
say nothing about the call (CLAUDE.md, "Only the side bidding").
The first version (four-card side suit to keycard, the 3NT re-ask,
`relay` as the ask) scored −85 there.

Open: RKCB's own 4NT is read as this rule's 4NT 223 times (`read_as`),
the same ask with a shape claim opener's answers do not use.

## Working HCP, and the stronger hand asks (Rick, 2026-11-07)

Rick, 2026-11-07, on #83/#84:

> "keycard should be the stronger hand. the idea of the relays is it
> will allow us to find low HCP slams when there is a good fit. one of
> the hands had 4-5 weak spades in responder's hand, and opener had a
> spade void, i think, which means all the combined HCP are 'working',
> which is the goal of this system (and the standard jacoby 2nt, and
> splinter bids). this may be a concept we haven't introduced yet."

And over 1♥: keep the minimum's spade-void answer at 4♠ past game
("responder is looking for something, and still has 4NT for keycard,
and 5H for bail out"). Kept.

What changed:

- **Working HCP is an engine concept** (`wasted_hcp`, `working_hcp`,
  `we.working_hcp`; slam-entry.notes.md, "Working values"). The
  rule-local `mj_working`, `mj_void_waste` and `mj_short_fits` are
  gone; the jack now counts as wasted too.
- **The decision is the working count**: `working_slam(trump)`,
  support points less the waste plus opener's floor, 23 opposite a
  void, 27 opposite a singleton (calibrated below).
- **The relay gate**: responder relays whenever slam is possible with
  everything working, his support points plus opener's top HCP 27 or
  more (`mj_relay_floor`). Every Jacoby 2NT hand has 13 support points
  and the minimum's top is 14, so over 3♣ responder always relays; over
  3♦ (15+, short) and the void re-ask the same gate replaces
  `slam_interest`. 13 or 15 support points facing a minimum now relay
  (cases). Floors measured with 23/27: 27 +301, 28 +287, 30 +247.
- **The stronger hand asks.** Our reading of Rick's rule: the stronger
  hand by what has been shown, responder's HCP against opener's floor
  (`stronger_hand`: 10 opposite a minimum, so responder, unlimited, is
  the captain there; 15 opposite the non-minimum answers). With slam on
  the count, the stronger responder bids 4NT; a weaker one bids the
  cheapest suit below game ("slam values opposite your shape: you
  ask", `ask=mj_captain`) and opener, the stronger hand, bids 4NT.
  With no room below game (over 1♥ the singleton or void spade lands on
  4♥/4♠) responder asks himself. The signal's public conditions (a
  shortness placed, below game, not the trump suit) sit in `when`: in
  `shows` the reading took responder's 4♠ sign-off as the signal and
  opener asked over it (−2,191 in the first run).
- **The side-suit answer** (4x, a good five-card suit) asks with 16 HCP
  and an honour in it (was 14): with the wider relay the 14-15 hands
  reached it and went down.

**Measured** (compare `--par-all`, 5,000 boards: Jacoby_2N,
Jacoby_2N_Leveled, Jacoby_2N_4x_void(_Leveled), Splinters,
Splinters_By_Opener, Splinters_after_Minor, Slam_After_Major_Fit,
Gavin_Strong/Weak_Splinter; `--set major_openings.jacoby_2nt.modified=true`;
PR #81's rules against these; errors_diff): 1,035 boards changed; the
bidding side's contract errors **+301 IMPs** (halves +127 / +174,
agree); distance from par +270; "short of slam" 1,041 → 1,000 boards
(12,803 → 12,323 IMPs), "overbid" 440 → 449. By first difference:
`P 4S P : 5H -> 4N` +66 on 15 (keycard after the spade void instead of
5♥), `P 3C P : 4S -> 3D` +48 on 335 (the wider relay), `P 4D P : 4N ->
4H` +34 on 22, `P 4D P : 4H -> 4N` +32 on 16. Losses small:
`P 4D P : 4N -> 4S` −8 on 7.

#83 and #84 still reach 6♥ (the auctions above), now on the general
count: #83's North 17 support points less Q, J of spades (3) plus 12 =
26; #84's 16 less the jack plus 12 = 27, both opposite the void.
The neighbour with KQ54 of spades opposite #83's South now asks too
(17 − 5 + 12 = 24; 13 tricks double dummy), so the case changed; the
sign-off case is now AKQ4 opposite the void (21: 5♥, 11 tricks DD
opposite -.AT642.AQ763.963). The low-HCP slam case: 8765.KJ87.KJ8.A2
opposite -.AQ642.AQ763.963, 24 HCP between them, 13 tricks: relay,
then keycard.

## Ask over 3♣ only with 15+; opener passes 4M without 19+ (Rick, 2026-11-07)

Rick, later the same day:

> "modified jacoby 2nt is mostly useful because responder can jump to
> game with no slam interest, after a 3C rebid by opener. the main
> advantage is that it doesn't give the defenders any clues to opener's
> distribution. so we don't want to always ask. but with say 15+ HCP,
> responder should probably ask. normally opener doesn't continue for
> slam exploration after responder chooses not to try, unless opener
> has 19+ points"

- **Over 3♣ responder relays 3♦ with 15+ HCP, else jumps to 4M**
  (priority 15, so the slam modules do not ask straight from 3♣
  either). This replaces the always-relay gate over 3♣ above
  (`mj_relay_floor` now gates only the relays over the non-minimum 3♦
  and the void re-ask). #84's North (13 HCP) now bids 1♥–2NT–3♣–4♥;
  #83's North (16) still relays and reaches 6♥.
- **Opener passes responder's 4M without 19+ points**, after any of
  his answers or relay answers (the `after` block at the end of the
  file, priority 16, above the keycard ask and the control bids). Not in
  standard Jacoby, whose shortness answer is unlimited
  (jacoby-2nt.notes.md; an open question for Rick).
- **The ruling overrides the double-dummy measurement.** What the relays
  tell the defenders (opener's shape, for the lead and the defence) is
  invisible to DD par, which scores every contract as if the defenders
  saw all four hands.

Measured against the always-relay version (PR #81 branch at 265c925;
compare `--par-all`, the same 5,000 boards, modified Jacoby on;
errors_diff): 988 boards changed; the bidding side's contract errors
**−267 IMPs** (halves −17 / −250, agree); distance from par −228.
"Short of slam" 1,000 → 1,108 boards (12,323 → 13,607 IMPs), "overbid"
449 → 386 (6,324 → 5,388). By first difference: `P 3C P : 3D -> 4H`
−176 on 441 and `3D -> 4S` −104 on 434 (the relay kept for 15+);
opener's pass `P 4H P : 4N -> P` −116 on 63, `P 4S P : 4N -> P` +129 on
50. It is Rick's ruling either way.

## Sources

- Larry Cohen, "Modified Jacoby 2NT",
  https://www.larryco.com/bridge-articles/modified-jacoby-2nt (the
  schedule, the relays, "should not be used by a passed hand", the
  interference notes, quoted in "Interference"). The forcing pass and
  the step answers as the main part of interference: Rick, 2026-10-28. Our deviations: the decisions above.
- convention-card `spec/conventions/bidding_conventions/modified_jacoby_2nt.toml`
  (summary: 3♣ a minimum, responder can then ask).
- Rick, 2026-11-06 (bridge-classroom-bug-artifacts #83/#84): relay over
  the minimum and go to keycard once the void is known. The 16-point
  gate, the working-HCP test and the void-first answer are ours.
- Rick, 2026-11-07 (same reports): "keycard should be the stronger
  hand", working HCP as the goal of the relays, the 4♠ void answer kept
  past game (quoted above). The reading of "stronger" (HCP against the
  shown floor), the "you ask" signal, the thresholds and the relay
  floor are ours.
- Rick, 2026-11-07, later (quoted in "Ask over 3♣ only with 15+"): the
  15+ HCP ask over 3♣, game otherwise, opener's pass without 19+.
