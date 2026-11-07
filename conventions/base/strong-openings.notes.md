# Strong openings (`strong-openings.bid`): notes

2♣, 2NT and 3NT, and what follows them. Cases: `strong-openings.test`.

## Guidance (Rick, 2026-09-23)

**Balanced hands go up a ladder, and every rung is a place to stop:**

| | | | |
|---|---|---|---|
| 2NT | 20-21 | 2♣ then 3NT | 24-25 |
| 2♣ then 2NT | 22-23 | 3NT | 26-27 |
| | | 2♣ then 4NT | 28+ |

So the 22+ HCP branch of 2♣ steps around the two notrump openings: a
balanced 26-27 opens 3NT, not 2♣.

**Unbalanced hands** open 2♣ on 23 total points (HCP plus one for each
card beyond four), or on playing strength — within one trick of game,
which is nine tricks with a major and ten with a minor, since game there
is eleven. Playing tricks are counted as 13 − losers.

**Responder**: 2♦ waits. A suit is a positive, 8+ with a real suit — five
cards with two of the top three honours, or six with three of the top
five. 2NT is the balanced positive. After opener describes, the cheapest
minor is the **second negative**: a bust, less than a jack and a queen.

French 2♦ (`two_level.french_2d.play`, openings/french-2d.bid) caps
every 2♣ opening at 23 and takes the 3NT opening's 26-27 as well.

Card fields: `two_level.two_clubs.2d_response` (waiting/negative/steps),
`parrish_bust` for the treatment where 2♥ shows the bust and 2♦ is game
forcing, `notrump.two_nt.range_min/max`, `notrump.three_nt.range_min/max`.

## Evidence, and one open question

BBA opened 2♣ eighteen times across the Basic_* scenarios, on 17-21 HCP
with big shape — `2.AQJ872.AKQJ83.`, `A2.AQJT7542.A.A3`,
`KQT43.AKQ32..AKT`. Its stated meaning is "19 to 37 total points". Every
one of the eighteen has a six-card suit or 5-5.

Rick's rule reads as two **alternative** triggers: 23 total points **or**
within a trick of game. Implemented that way it opens 2♣ on 17 of BBA's
18, but also on 14 of the 67 strong one-level hands BBA opens at the one
level, and it measures worse — the false positives cost more than the
true ones gain:

| unbalanced trigger | calls | auctions | contracts | IMPs vs par |
|---|---|---|---|---|
| playing strength alone | 84.2% | 39.3% | 50.3% | -8,542 |
| and 20 total points | 84.4% | 40.2% | 51.6% | -8,225 |
| and 21 | 84.7% | 40.9% | 52.7% | -7,821 |
| and 22 | 84.7% | 41.1% | 53.2% | -7,636 |
| **and 23** | **85.1%** | **41.3%** | **53.5%** | **-7,574** |

So the rules currently require **both**: 23 total points *and* within a
trick of game. **This is a question for Rick** — his wording says either
one is enough, and the measurement says both. The cost of following his
wording is about 1,000 IMPs over 11,656 boards.

A suit-quality guard came out of the same measurement: the long suit
needs two of the top three honours, or the losing-trick count flatters a
hand like `Q98765.AQ7.AK6.A` (four losers, a suit not worth bidding
twice). BBA opens that 1♠ and so do we.

## Accepted differences from BBA

- BBA opens 2♣ on hands of 20-22 total points that we open at the one
  level (8 of its 18): `AKQ654.AK9.J5.K5` (22), `2.AQJ872.AKQJ83.` (21).
- BBA opens 1♥/1♠ on some hands within a trick of game that we open 2♣.
- After 2C P 2M P 2NT P: BBA jumps to 4M with some weak seven-card
  suits, bids 4NT rather than a four-card minor with 8-9, and shows
  hearts before rebidding six spades; opener cues a control where BBA
  asks at once (see "After a positive and opener's 2NT" below).

## Gaps and open questions

- **The 2♥ bust treatment** (`parrish_bust`, written 2026-10-05): 2♥
  shows 0-2 HCP and lifts the game force (`sets forcing=none`); 2♦ then
  waits with 3+ and the second negatives are off. Opener rebids
  naturally: 2♠ forcing one round (responder 3♠ with three, else 2NT),
  2NT 22-23 and three-level suits passable, 3♥ for hearts, 3NT 24-25.
  The positive-response blocks skip 2♥ on that card. Kokish's relay
  (2♣–2♦–2♥) is unaffected: it follows the waiting 2♦. Bust_Over_Strong_2C
  with the field set: 5 boards, the divergence is our 2♥ where BBA (not
  playing it) waits. Sources: Larry Cohen, "Two Club Opening"
  (larryco.com: 2♥ as the immediate double negative); opener's rebids
  are standard practice, not yet cited.
- `two_clubs.2d_response = steps` is not modelled.
- Opener's second call after the second negative.
- Slam bidding after a positive: covered for opener's raise (responder
  4M, opener asks), for 2C P 2M P 2NT P (below) and for the balanced
  positive; not for a minor second suit (opener bids 3NT over 3C/3D, and
  4NT in the minor is not tried), nor for the grand: after 2C P 3D P 3S
  P BBA reaches 7S with all the keycards where we stop in 6S
  (To_Finesse_Or_Not_To_Finesse 1, 15, 38, 89), because `grand_try`
  counts responder at his 8 HCP floor and there is no king ask.

## Raising opener's major after a positive (2026-09-25)

To_Finesse_Or_Not_To_Finesse was the corpus's worst covered scenario by
par (-2,292 over 500 boards). After 2C-2NT-3S (or 3m-3M), responder
held three trumps and 8-11 and bid 3NT: there was no raise. BBA drives
to slam. Responder now raises to 4M with three or more, agreeing the
suit; opener with the values asks with 4NT.
- Distance from par: +774.
- To the bidding side: +1,556 (165 boards).

BBA's lighter positive responses (any five-card suit with 7+ total
points; 2NT with balanced 7+) are left alone: Rick's guidance (8+ and a
real suit) stands.
- Opening 1M rather than 2NT with 20-21 and a five-card major, as BBA
  sometimes does: -1,717 by distance, -3,432 to the bidding side (626
  boards). Rick's ladder (2NT with 20-21 balanced) stands.

## After a positive and opener's 2NT; the balanced positive (2026-09-28)

NT_Ladder 426 (Rick: "we should be in 6S here like BBA"): 2C P 2S P
2NT P with `AK9876.J4.T9653.` opposite `J4.AKT.AKQ2.AJ75`. BBA:
3S, 4NT, 5H, 6S. We bid 3NT: there were no rules for responder here,
and the notrump-ladder catch-all (responder-rebids.bid) took over.

Opener's 2NT after a major positive is balanced, 22+, with exactly two
cards in responder's major. Responder now (priority order):
- **3M with six or more**: the eight-card fit is known, so it agrees the
  suit (`sets trump`), and the keycard or control-bid machinery in
  slam/ takes over (opener's 22 is always 18 of its own).
- **The other major with four**, then **a minor with four**: natural.
  Opener with four in responder's other major agrees it and asks at once
  (4NT, `trump=z, ask=keycards(z)`; game in it without RKCB on the
  card). Over a minor opener bids 3NT (the base fallback).
- **5-3-3-2**: a quantitative 4NT with 30-32 between us (opener at 22),
  6NT with 33+.

Opener's notrump after 2C P 2NT P now splits the ladder as over 2D:
3NT is 22, a quantitative 4NT 23-24, 6NT 25+; and both of opener's
balanced rebids after a positive show `hcp>=22` (the 2C opening's own
reading starts at 17 because of the playing-strength openings), so
responder's slam arithmetic over them counts opener at 22. That is what
found 6NT on Benjamin_2D 25 and 151.

BBA, probed:
- `probes/strong2c-2S-2N-resp.toml`, `strong2c-2H-2N-resp.toml`
  (responder, 8-13 with five or more in the major, opener 22-24 with two):
  six-card suits rebid (we always do; BBA jumps to 4M with some weak
  7-card suits and 8-9), four-card side suits are shown (BBA, with a
  minor and 8-9, bids 4NT about half the time; we show the minor),
  5-3-3-2 bids 4NT with 8-9 and 6NT with 10+ (we bid 4NT with 10: 22+10
  is 32, one short). BBA shows hearts before rebidding six spades on
  6-4; we rebid the spades.
- `probes/strong2c-2S-2N-3S-opener.toml` (opener, 22-24 with two spades,
  after 3S): BBA asks with 4NT on 289 of 300 and bids 4S on 11; we agree
  on 256. The rest are control bids: with a side suit holding neither
  ace nor king we cue first (control-bids.bid, Rick 2026-09-25), where
  BBA asks straight away.
- `probes/strong2c-2N-opener.toml` (opener balanced 22-25 after
  2C P 2NT P): BBA 3NT with 22, 4NT with 23-24, 6NT with 25; we agree on
  295 of 300.

Measured (after the catch-all change in responder-rebids.bid):
- Responder's rebid over 2NT and the 4NT ask: distance from par +48,
  to the side +135 (22 boards).
- Opener after 2C P 2NT P (3NT 22, 4NT 23-24) and `hcp>=22` on the
  balanced rebids: distance from par +452, to the side +951 (132 boards).
- The 2C-opened boards over all scenarios (either auction opened 2C,
  5,401 boards): vs BBA -11,726 → -11,226.

## Opener's raise of a positive shows 22+ (2026-10-01)

Rick (2026-10-01): after 2C-2H (a natural positive) opener's 3H is the
balanced 22-24 hand that was going to rebid 2NT; with a long suit and
fewer HCP opener shows his own suit. BBA raises on every 22+ hand with
three hearts, even 75.AKQ.AKQ95.AT9 with five diamonds (272 of 272 in
`probes/slam-2C-2H-opener.toml`, alert "21+ total points"); with 18-21
it bids 3H or 4H ("19-20", fast arrival), never its own suit. We read
the raise at the 2C floor (17) and bid our own suit on a third of the
22+ hands. The raise now shows 22+ and outranks the own suit, for every
positive (`2C 2y` / `2C 3y`). Vanilla +60 IMPs vs BBA, corpus +97, 21GF
random +40, both halves positive; responder's slams after 2C-2H-3H are
slam-entry's (slam-entry.notes.md, "Declarer points and support
points").

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only 2h-bust`.
IMPs by the errors yardstick, positive when the convention makes fewer;
"actor" is the side that made the first differing call. 2♥ bust
(`parrish_bust`), Bust_Over_Strong_2C (5 boards), Game_Forcing_2C and
Opps_Bid_Over_GF_2C: 174 boards changed; actor contract +180, other side
doubling +193, double-dummy +171, halves +94/+86 (z +2.6): gains.

## Systems on over opener's notrump (Rick, 2026-10-05)

Rick: 2♣–2♠–2NT and 2♣–2♥(positive)–2NT are systems on, like
2♣–2♦–2NT and 2♣ (2♠) P (P) 2NT (one-nt.notes.md, "Systems on"). So by
default responder answers opener's 2NT after a positive with the 2NT
system (Stayman, transfers: the six-card suit transfers and opener
declares). The natural rebids probed from BBA (2026-09-28, "After a
positive and opener's 2NT") are now BBA's treatment, `general.style =
bba` (strong-openings.test keeps their cases there). Measured on the
tripwire only: Grand_Slam_Invite 23 now stops in 4♥ after the transfer
where the natural 3♥ and keycard reached 7♥; slam continuations after a
transfer over a 22+ 2NT are the jacoby-transfers.bid ones.

**After their overcall and partner's pass** (2♣ (2x) P (P)) opener had
no rule at all. Added the balanced rebids only: 2NT with 22-23 and a
stopper (systems on over it), 3NT with 24-25 and a stopper. The suit
rebids and doubles there are still missing.

## Bridge-Classroom #94 and #92 (Rick, 2026-11-06; rules 2026-11-07)

**#94**, NT Ladder (Lev) board 38, dealer S, EW vul: North
`K973.AQT43.J942.-`, South (Rick) `AQJ.K8.AKT65.AK2`. Auction 2♣ P 2♥
(positive) P 2NT P 3♦ (transfer) P 4♦ (Rick) P 4NT (North:
"Quantitative, 5 ♥", alerted) P 5♣ (no rule) P P P: 5♣ in a 4-0 fit,
thirteen tricks in ♦, ♥, ♠ and NT. Rick: "Broken Rusty auction.
Strange 4NT alert."

1. **Systems stay on over 2♣–2♥(positive)–2NT.** Rick ruled this on
   2026-10-05 ("Systems on over opener's notrump" above), so the 3♦
   transfer is right: hearts are known, but the transfer makes the 22+
   hand declarer. Not reopened.
2. **Opener's 4♦ over the transfer is natural**: five or more diamonds,
   slam interest. Opener's 2NT denied three hearts (it would have
   raised), so a super-accept is impossible. It was unread (base.bid's
   runout fallback); responder still took it as "partner answered my
   transfer" and jacoby-transfers.bid's quantitative 4NT fired. Now
   read (`4C`/`4D` after 2♣–2M–2NT–transfer, priority -1: opener's own
   choice stays the completion).
3. **Responder over that 4m:** with three or more of the minor the fit
   is found and 4NT asks for keycards in it (`trump=m`), the card's RKCB;
   without RKCB, six of the minor; no fit, four of his major.
4. **The asker never passes an unread five-level answer to a
   quantitative 4NT** (base.bid, priority -40, above the game-reached
   pass): 6NT, taking the unread suit as an acceptance. The keycard
   answers already had their continuations (rkcb-1430.bid passes a
   5-of-trumps answer only with at most three keycards between us).
5. **Rusty's own auction on the board** went 2♣ 2♥ 3♦ (five diamonds)
   3NT: responder had no raise of opener's minor after a major
   positive. Responder now raises to 4m with four (agrees it), opener
   asks: 2♣ 2♥ 3♦ 4♦ 4NT 5♣ 6♦.

Calls on the board, before → after (stand-in card 21GF-GIB, modified
Jacoby 2NT, Kokish on): North over Rick's 4♦: Pass/4NT quantitative →
4NT keycard in ♦; over 5♣: Pass → 6♦. All-Rusty: 3NT → 6♦.

**#92** (kokish.notes.md has the diagnosis): with Kokish off, 2♣–2♦–2♥
is natural and responder had only the second negative, so the base
fallback bid 3NT on everything else. Responder now shows a five-card
suit without a fit (2♠ over 2♥, 3♦), waits with 2NT with no fit, and
opener over the 2NT rebids six of his major, shows a second suit or
bids 3NT. A natural 4♥ raise with few values was tried and dropped:
opener's 18+ asked for keycards over it (-81 IMPs on the actor's
contract errors over the strong-2♣ scenarios).

Measured (compare --limit 50 --par-all, Kokish_Relay, NT_Ladder,
Game_Forcing_2C, To_Finesse_Or_Not_To_Finesse, Bust_Over_Strong_2C,
Opps_Bid_Over_GF_2C; 255 boards): 13 changed, all at responder's call
over 2♣–2♦–2M (3NT → 2♠/2NT/3♦). `errors_diff.py` verdict, the actor's
contract errors: +10 IMPs (halves -5 / +15, they differ: small and
mixed), distance from par +13; vs BBA by par -185 → -172, by errors
-173 → -167. The 4m raise and the reading of opener's 4m over the
transfer did not occur in those 50-board samples (`.test` cases only).
All-scenario tripwire (`--limit 50`, 17,105 boards): 28 changed, every
one at responder's call over 2♣–2♦–2M or the new 4♦ raise after
2♣–2M–3♦ (+12 twice); nothing else moved. Actor's contract errors +22
(halves -5 / +27), distance from par +25; vs BBA by par -6,223 → -6,198,
by errors -7,734 → -7,716.

## Brill (general.system = brill, 2026-10-15)

Brill's treatment, played on the Brill card only (`cards/json/Brill.json`, `general.system = brill`: a stand-in until convention-card's `general.style` takes a `brill` value; cards/README.md "Brill"). Rusty plays the Brill dailies as South with three Brill robots, so it should read and bid what they do. Every other card is unchanged (a `.test` line on 21GF-GIB beside each Brill line; `compare --limit 50` tripwire against main: no board changed).

3NT is 25-27 (the card) with no five-card major ("Balanced hand, 25-27
HCP": `(balanced or semibalanced) and hcp>= 25 and hcp<= 27 and H<= 4 and S
<= 4`); a balanced 25-27 with a five-card major opens 2C. The 2C rule's
`hcp<=25` stays for every other card (their 3NT is 26-27 or the card's).

Measured with `probes/tools/brill_probe.py report --card Brill` over the top 2000 corpus auctions (calls of this group, board-weighted, our reading against Brill's): 0% -> 100% of 1,170 boards.

## Sources

- Brill /explain, 2026-10-15 (Thorvald Aagaard's public Brill service, answers cached in `.rbb-cache/brill-service/explain/`): the rule text quoted in the `brill` section and in the `.bid` comments. Applies to the Brill card only (`general.system = brill`, cards/README.md "Brill").
- **Rick's rulings (2026-09-23):** the balanced ladder (2NT 20-21, 2♣
  then 2NT 22-23, 2♣ then 3NT 24-25, 3NT 26-27, 2♣ then 4NT 28+); the
  unbalanced 2♣ on 23 total points or within a trick of game; responder's
  2♦ waiting, positives with a real suit and 8+, the second negative.
  Rick's guidance also stands against BBA's lighter positives and against
  opening 1M with 20-21 and a five-card major (2026-09-25).
- **Book practice:** the ladder, playing tricks as 13 minus losers, the
  second negative, and after 2C–positive–2NT rebidding a six-card suit,
  showing a four-card side suit and the quantitative 4NT are standard
  practice, not yet cited to a book or article.
- **BBA evidence:** BBA's eighteen 2♣ openings in the Basic_* scenarios
  and its stated meaning "19 to 37 total points" (no probe spec for the
  opening itself).
- **BBA probes (2026-09-28):** `probes/strong2c-2S-2N-resp.toml`,
  `strong2c-2H-2N-resp.toml`, `strong2c-2S-2N-3S-opener.toml`,
  `strong2c-2N-opener.toml` (responder's rebid over 2NT, opener after
  the six-card rebid, opener after the balanced positive).
- **Rick's ticket (2026-09-28):** NT_Ladder 426, "we should be in 6S
  here like BBA".
- **Corpus measurements:** the unbalanced-trigger table, raising
  opener's major after a positive (2026-09-25), and the 2NT
  continuations (2026-09-28).
- **Where we differ from the source:** Rick's wording makes the two
  unbalanced triggers alternatives; the rules require both, because
  that measures about 1,000 IMPs better (open question for Rick). From
  BBA: see "Accepted differences from BBA".
- **Rick's guidance (2026-10-01):** the raise of a positive is the 22+
  hand; `probes/slam-2C-2H-opener.toml` (BBA bare SAYC) for BBA's raise.
- Rick, 2026-10-05: systems on over 2♣–positive–2NT and over 2♣ (2x) P
  (P) 2NT; the balanced rebids after interference are standard practice,
  not yet cited.
- Rick, 2026-11-06 (bridge-classroom-bug-artifacts #92, #94): the boards
  and notes. Opener's natural minor over a transfer that cannot be a
  super-accept, the raise of opener's minor after a positive, the
  keycard ask in it, 2NT waiting over 2♣–2♦–2M and showing one's own
  suit there: standard practice, not yet cited.
