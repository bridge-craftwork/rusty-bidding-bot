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

Partner judges the side suit: with an honour or three cards there and
14+ HCP (12+ in a major with the fit there) he bids game (3NT with a
balanced minor opening, five of the minor with 16+), otherwise the
cheapest bid in the trump suit, which the jumper passes.

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
2NT. For Rick: opener's continuation after a passed hand's fit jump
bids game too readily; not changed here.

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
