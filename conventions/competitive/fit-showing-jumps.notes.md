# Fit-showing jumps (`fit-showing-jumps.bid`): notes

Switched on by `competitive.fit_showing_jumps.play` (21GF-GIB,
21GF-GIB-Bergen, 21GF-WJS-MSS, 21GF-SPECIALS2, 21GF-PolishTwoSuiters,
Precision). Cases: `fit-showing-jumps.test`.

## What it plays (2026-10-05)

A jump shift shows four or more of partner's suit, five or more of the
side suit with the ace, king or queen, and about a limit raise:

- a passed hand over partner's opening (also over a double or an
  overcall on the right): 10-12 total points, at most 11 HCP;
- responder over their takeout double: 10+;
- advancer after partner's one-level overcall: 10+ (a passed advancer
  at most 11 HCP), forcing one round.

Partner judges the side suit (revised 2026-10-09, "Continuations"
below). In a major: game with 14+ total points (as over Jordan 2NT), or
12+ HCP with an honour or three cards in the side suit. In a minor:
four of a major side suit with three or more there and 14+ total
points; 3NT with 16+ balanced and the side suit covered, or 18+; five
of the minor with 16+ unbalanced and the side suit fitting. Otherwise
the cheapest bid in the trump suit, which the jumper passes.

## BBA (fast-lane compare, 2026-10-05)

`compare Fit_Showing_Jumps Fit_Jumps_after_1M_Double --limit 50`: BBA
alerts fit jumps in exactly these places (passed hand over 1m, 1M (X)
jumps, advancer's jumps), but makes them on fewer hands. On a passed
hand with one top honour and a singleton it often bids the simple
response (1D, 1H, 1S), and over 1M (X) it bids the Jordan 2NT on most of
the scenario's hands. Taken as BBA style: the cited definition is the
spec. Its opener also bids game more often after the jump (BBA 4M where
we sign off), which is judgment inside the convention.

## Open questions

- **For Rick.** Is a passed hand's jump over partner's *major* fit-showing
  too? We play it over any opening; on the 21GF cards Drury takes the
  passed hand's limit raise with three trumps, so the fit jump only
  arises with four trumps and the side suit. BBA's scenario note says
  BBA uses Drury instead there.
- An unpassed responder's jump shift over their overcall is left to
  `competitive.jump_shift_after_overcall` (weak on the 21GF cards); some
  partnerships play those as fit jumps as well.

## Convention score (2026-10-07)

The off run had 54 no-rule positions (1 on): without fit jumps a
passed hand's fit raises go through inverted minors, where opener had
no rule over 1m-2m-2NT-3m and responder none over 4m. Both are now
written (inverted-minors.bid, the same day's inverted-minors fixes);
with them the off run had 5 against 1, and the score is measured:
changed 855 (ours 850, BBA 307), Rusty -1,778, BBA -133, net -1,645 =
-1.92 a changed board, both halves negative ("poor"). The worst boards
are the passed hand's jump followed by opener's 3NT where the off
auction stops in a partscore (P P 1D P 3C P 3NT; P P 1C P 2S P 3C P
3NT), and over 1M X the fit jump where the off auction uses Jordan
2NT. Fixed 2026-10-09 (Continuations).

## Continuations (2026-10-09)

The convention score's worst boards had three causes, all in the
continuations, none in the jumps themselves:

1. **Opener over the jump after 1M (X) counted HCP only** (14+, or 12+
   with the side suit fitting). With a 5-5 or 6-4 minimum and shortness
   in partner's side suit he signed off in 3M where the off auction
   (1M (X) 2NT, Jordan) reached a making game: opener accepts Jordan
   with 14+ total points. Fix: 4M with `tp(M)>=14`, or 12+ HCP with the
   side suit fitting. This was the "fit jump where the off auction plays
   Jordan" item: the jump itself was not the loss (with the fix our
   1M (X) boards are -42 for the jump against our Jordan auctions, BBA's
   -75). vs-takeout-double.bid is unchanged.
2. **The passed jumper bid 3NT over opener's sign-off**: P P 1D P 2H P
   3D is also the shape of a two-over-one followed by opener's rebid of
   his suit, and responder-rebids.bid's "3NT: game, no fit" (priority 0)
   outranked our pass (-5). Fix: the pass at priority 3 (the 13+ game
   bid for an unlimited jumper at 4).
3. **Opener's 3NT over a passed hand's minor-suit jump with 14
   balanced** (8-11 HCP opposite: 22-25). Fix: 3NT needs 16+ balanced
   (in practice 18-19, since 15-17 opens 1NT) or 18+ with any shape,
   and a major side suit with three or more cards and 14+ total points
   plays four of the major (P P 1D P 2S P 3NT missed 4S).

Convention score (conv_ab, BBA's runs reused): changed 855 (ours 850,
BBA 307); Rusty -1,667 -> +167, BBA -133; net -1,534 (-1.79 a changed
board) -> +300 (+0.35), halves -722/-812 -> +144/+156; contract +151,
doubling +16. By scenario: Fit_Showing_Jumps Rusty +209 against BBA's
-58, Fit_Jumps_after_1M_Double -42 against -75. Tripwire (`compare
--limit 50`, all scenarios): 68 boards changed, 59 in the two scenarios;
the other 9 are this module's own continuations in scenarios whose cards
play fit jumps (Muiderberg, Weak_NT, GIB_1N5422, Mathe, ...).

Left: opener's 3NT over P P 1C P 2D and P P 1D P 3C still loses a
little (-10 and +3 against BBA's +7, +19); the over-double jump is still
a little worse than our Jordan auction on the same hands.

## Sources

- Robert S. Todd, "Fit-Showing Jumps", Advancing in Bridge 574
  (https://www.advinbridge.com/this-week-in-bridge/574): passed hand and
  advancer, 4+ support, a side suit with honours, a limit raise.
- Robinson, *25 More Bridge Conventions You Should Know*, ch. 14 (cited
  by convention-card `spec/conventions/competitive_bidding/fit_showing_jumps.toml`;
  not read here).
- Practice-Bidding-Scenarios `btn/Fit_Showing_Jumps.btn`,
  `btn/Fit_Jumps_after_1M_Double.btn` (the 1M (X) case, 10-12 support
  points, a five-card side suit with a top-three honour).
- Deviation from Todd: the side-suit honour is any of A, K, Q (the
  scenario's definition); responder over their double and advancer are
  unlimited (10+), not exactly a limit raise.
- Deviation (2026-10-09): Todd has opener discount shortness in the
  side suit; in a major we still accept with 14+ total points, which
  count that shortness, because with four trumps opposite the jump the
  10-card fit made game on the corpus deals (convention score above).
  Question for Rick: keep it, or discount the side-suit shortness (and
  give back some of the gain)?
