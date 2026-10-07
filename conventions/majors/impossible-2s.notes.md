# Impossible 2♠ — notes

After 1♥–1NT responder has denied four spades, so 2♠ over opener's 2♣ or
2♦ cannot be natural: it is a good raise of opener's minor. Card field
`major_openings.impossible_2s.play`; BBA's `.bbsa` key `Imposible 2S`
(sic) maps to it (on in 9 of the PBS cards, 21GF-DEFAULT among them).

## Sources

Rick (2026-09-28) pointed to two sources; the key was mapped on their
authority.

- **Robert S. Todd, "The Impossible ♠ Bid"**, Adventures in Bridge, This
  Week in Bridge (269), level 3
  (<https://static1.squarespace.com/static/5127d3d2e4b0b304f0b6db24/t/5c8e551f104c7b066eb43ac8/1552831776048/3+%28269%29+The+Impossilbe+Spade+Bid+.pdf>).
  After 1♥–1NT (forcing or semi-forcing) – 2♣: "3♣ Courtesy Raise, 4+card
  ♣, 8-9 pts"; "2♠ Limit Raise, 4+card ♣, 10-11 pts (Impossible ♠)". The
  same over 2♦. Over opener's 2♥ (11-14, six hearts) the two raises
  describe the fit instead: 3♥ invitational with two hearts, 2♠ a limit
  raise with three, both 10-11. "In all of these auctions the impossible
  ♠ shows the best raise possible, given the context of the auction!"
  No follow-ups are given, and nothing after a 1♠ opening.
- **GIB** (BBO's bidding database, read through
  <https://netbridge.dk/gib.html>, which queries
  `gibrest.bridgebase.com/u_bm/u_bm.php`; fetched 2026-09-28):
  - 1♥–1NT–2♣–2♠: "Impossible spade -- 4+ ♣; 2- ♥; 3- ♠; 11-12 total
    points; forcing to 3C". The same over 2♦ ("forcing to 3D"). 3m there
    is "5+, 8-12 total points"; 2NT a balanced invitation, 10+ HCP.
  - Opener's answers: 3m minimum (12-13 total points); 3♥ six hearts,
    12-13; 2NT 15-17 HCP, partial stoppers, forcing to 3m; 3NT 16+ HCP;
    the other minor natural and forcing; 3♠ "4+ ♠".
  - Responder then: pass 3m; over 2NT, 3NT with 10+ HCP, 3m with five.
  - Not over 1♠ (1♠–1NT–2m–2♠ is "2-3 ♠, 6-9 HCP") and not over 1♥–1NT–2♥
    (2♠ there is natural-ish: "3 ♠, 8-12 total points").

**Where they differ.** Todd's 2♠ is 10-11 with four trumps and his 3m the
8-9 courtesy raise; GIB's 2♠ is 11-12 total points and its 3m is a
five-card raise over the whole 8-12 range. Todd also plays 2♠ over 2♥
(three-card limit raise); GIB does not. GIB defines opener's answers,
Todd gives none. Neither uses it after 1♠.

The PBS scenario (`btn/Impossible_2S.btn`) cites a third source, a Kiva
Bridge Club PDF, whose 2♠ over 1♥–1NT–2♥ shows 6-5 or better in the
minors. Not implemented.

## What we play

- Responder, after 1♥–1NT–2♣/2♦: 2♠ with 4+ of opener's minor and 11+
  total points (our 1NT is 6-10 HCP, so this is 10 HCP and a fifth trump,
  or 9 with a sixth). Forcing one round, the minor agreed. The raise to 3m
  stays at 9-10: Todd's courtesy raise, one point up because our points
  count length. Before this, 11+ hands had no call and passed 2m.
- Opener: 3m with 14 or fewer support points (HCP plus shortness);
  with 15+, 3NT holding both unbid suits, else the stopper he has (up the
  line, `ask=stoppers`), else five of the minor. Responder answers a
  stopper bid in responder-rebids.bid (`when asked stoppers`), with
  3NT, the next stopper or 5m.
- Responder passes 3m, 3NT and 5m.
- Only after 1♥, only over a minor rebid. Todd's 2♠ over 2♥ has no hand
  here: our 1NT over 1♥ denies three hearts (the three-card raises go
  through 2♥ or 3♥).
- When the opponents play it, their 2♠ is read with their card: over
  1♥–1NT–2♣ it shows 4+ clubs and ≤2 hearts, not spades (checked with
  `rbb call ... --ew-card 21GF-DEFAULT`; with Basic-Bridge for EW the
  same 2♠ has no rule).

## BBA (probes, 21GF-DEFAULT, 2026-09-28)

BBA's note on the call is "limit raise or better in !C" (142 corpus
boards in Impossible_2S) or "!D" (12).

- `probes/imp2s-resp-1H-1N-2C.toml` (200 responders, 6-12 HCP, ≤2 hearts,
  4+ clubs): BBA bids 2♠ from about 11 points with five clubs, and from 12
  with four; with four clubs and two hearts it prefers 2♥ up to 11; 3♣ is
  five clubs and 10-11. Every one of BBA's 79 2♠ calls is ours too;
  agreement 24/200 before, 100/200 after. The rest are the existing
  raise/preference rules (BBA 2♥ or pass where we raise to 3♣ with four
  clubs), untouched here.
- `probes/imp2s-opener-1H-1N-2C-2S.toml` (200 openers, 11-18, 5♥ 4+♣):
  BBA treats 2♠ as nearly forcing to game. 3♣ only with a poor 11; then
  3♥ ("biddable suit", 87), 4NT keycard (39), 3♠/3♦ ("en passant"), 5♣,
  3NT. Agreement 54/200. **Accepted difference**: we sign off in 3♣ with
  up to 14 support points (39 hands where BBA goes on), following GIB's
  12-13 minimum; responder has at most 10 HCP. For Rick: the corpus cannot
  settle this (see below); a par study on dealt hands could.

## Corpus (2026-09-28)

Full corpus, `--pbs` Practice-Bidding-Scenarios: vs BBA −114,604 →
−114,581 (+23 IMPs); 18 boards change, all at 1♥–1NT; sideimps.py
+12 IMPs to the side that changed its call. Calls agreeing +337.
Impossible_2S itself: calls agreeing 72.1% → 75.8% (NS 44.2% → 51.6%),
contracts unchanged: the scenario's responder has 11-12 HCP and BBA's
forcing 1NT, where ours bids 2♣/2♦ at once (the 1NT response is 6-10), so
our auctions seldom reach the position.

## Convention score (2026-10-07)

The first run (`probes/tools/conv_ab.py --only impossible-2s`) found our
auctions identical with the card field on and off: on the 2/1 cards the
6-12 1NT had brought a natural "3m: 4+, 11-12 HCP" raise into
responder-rebids.bid (2026-09-30) that outranked 2♠ on description and
took all its hands. 2♠ now has priority 1. Without the convention the
11+ hands raise to 3m (responder-rebids.bid: "4+, 11+ points", for any
card, so the old gap where they passed is closed). After the fix the
switch changes 463 of 500 boards (BBA's 154): Rusty +250, BBA +6, net
+244 = +0.53 a changed board, both halves positive ("good").

## Gaps

- On the 2/1 cards the 1NT is 6-12 and the scenario's 11-12 responders
  now reach 2♠; the "6-10" in "What we play" is the standard card's 1NT.
- Todd's 2♥ variant and Kiva's 6-5 variant: not built.
