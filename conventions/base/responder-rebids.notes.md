# responder-rebids (`responder-rebids.bid`): notes

Responder's second call after a one-level suit opening, a response and
opener's rebid (Standard American, BBA's Basic-Bridge card). Also:
responder's third call after a 2/1 and after responder's own 1NT. Cases:
`responder-rebids.test`.

## Guidance (Rick, 2026-09-22)

- **Responder invites with 2NT, or with a jump in his own suit, which
  promises six.**
- **A new suit by responder after opener's 1NT is not forcing, unless it is
  a responder's reverse** (game forcing). With or without New Minor
  Forcing; with NMF on, two of the other minor is NMF instead
  (`new-minor-forcing.bid`, a convention of its own).

## How the rest was set

From BBA's meanings and the corpus: opposite a minimum rebid, weak is up to
10, invitational 11-12, game 13+; opposite opener's jumps (16-18) game at
9+, opposite 2NT (18-19) game at 7+. Invitations ask a question, so
opener's existing answers apply: 2NT asks `nt_invite` (accept with a
maximum), a jump in a major asks `invite(M)` (accept in the fit).

Choices from BBA: a weak six-card minor passes 1NT (a weak major rebids
it); preference to opener's major with two cards when opener's second suit
is no longer (false preference), but between minors pass with more of
opener's second suit; a weak hand raises opener's second suit at the two
level up to 10; after a jump rebid in a minor 3NT needs no shortness;
after a 2/1, support means game at 11+ (a 2/1 is already 11+).

## Evidence

Basic_* responder's second calls (1,652 positions): 71.9% agree with BBA.
All Basic_*: calls 69.0% → 78.0%, identical auctions 12.5% → 31.6%.
Whole corpus: calls agreeing 69.9% → 70.3%, same contract 20.8% → 26.0%,
no-rule problems 73,286 → 29,714.

## Accepted differences from BBA

- On the 2/1 cards BBA passes 1m-1M-1NT with 11 HCP (about 350 boards);
  Rick's 2NT invitation stands.
- Invitations over a raise at the boundary (`1♣-1♥-2♥`: BBA passes some
  11-counts that we invite with).

## Gaps (not built yet)

- Fourth suit forcing (the card field exists; the Basic card does not play
  it: 1♦-1♥-1♠-2♣ is natural).
- Slam tries by responder; opener's continuations after responder's
  second call beyond the invitations.
- Passed-hand and competitive versions.

## Answering a reverse (2026-09-22)

Opener's reverse is 17+ and forcing for one round, and responder had no
rules for it: he passed, which broke the force. He now bids game with a
maximum for his 1NT response, raises the second suit with four, and
otherwise gives a preference to opener's first suit.

The context is `after 1x (P) 1N (P) 2y (P) when y is not x, we.forcing =
round`. **The `we.forcing` test is what tells a reverse from an ordinary
second suit**: 1♠-1NT-2♥ is not a reverse and responder may pass it,
while 1♦-1NT-2♥ is. Without that condition the rules fired on every
second suit and bid over hands that should pass, at a cost of about a
dozen calls in the Basic_* scenarios.

## Signing off, and the two-over-one after a minor (2026-09-23)

### Responder's weak calls are a question now

Every weak second call here — 1NT over opener's new suit, a preference,
a weak rebid of responder's own suit, a raise of opener's second suit,
a weak new suit over opener's 1NT rebid — now carries
`sets ask=signoff`. It says "play here unless you have real extras", and
opener's answer is written once in rebids.bid (`when asked signoff`)
instead of a dozen eight-call `after` patterns. Every invitational call
sets `ask=invite(x)` or `ask=nt_invite` for the same reason; two of them
(the invitational raise and the invitational preference after opener's
second suit) were not setting any ask, so opener had nothing to answer
with and passed a live auction.

### The two-over-one block was only written for major openings

`after 1M (P) 2x (P) 2z (P)` left 1♦-2♣-2♥ and its relatives with no
rule at all. The rules that do not name a major are now written for any
opening suit `y`, with `y<=2 | y is C | y is D` where "no fit" was meant
(after a minor opening, notrump is the game to look for anyway), and the
raise to game in opener's major stays in a `1M` block of its own.

**The measure was wrong too.** A two-over-one is bid on suit points —
`KJ74.Q95.9.KJ762` is a ten-count at notrump and bids 2♣ — so the
follow-ups that choose a suit now use `suit_points`, and the raise to
game uses support points `tp(M)`. With `points>=11` those hands failed
every rule and passed. Added at the same time: a pass for a minimum
two-over-one when opener rebids his own suit, 3NT over opener's 2NT or
jump rebid, and a raise to three when responder is a maximum for his
1NT response and opener rebids his suit.

### Numbers

On the Basic-Bridge cards (11,656 boards), together with rebids.bid:
calls 89,437 → 89,506, identical auctions 41.3% → 41.4%, same contract
53.5% → 54.3%, no rule in a live auction 5,989 → 5,238 (uncontested
925 → 10), par -7,574 → -6,969 IMPs. Whole corpus: same contract 27.0%
→ 27.7%, no rule 165,221 → 156,108, par -244,279 → -232,422.

### Accepted differences from BBA

- Opposite opener's jump rebid of a six-card major after a two-over-one
  we raise to game with a doubleton (an eight-card fit); 3NT now needs
  a singleton or void in opener's suit. BBA bids 4M there too.
- A ten-count with shape that bid a two-over-one raises opener's major
  to game on support points. BBA opens those auctions differently
  (it jumps to 4M over 1M), so the corpus cannot settle it.

### Tried and rejected

- Accepting opener's invitation after a raise (1♠-2♠-3♠) only with 10+
  support points instead of 9+: +11 calls, but par -7,002 → -7,529 IMPs
  and contracts 54.3% → 54.1%. Declining more costs real matchpoints;
  the 9+ boundary stays.

### Open questions

- Responder's fourth call after opener's "one more try" 3x: he is
  limited to 10 and the try showed 16-18, so the strength bands make a
  9-10 count bid game. BBA passes. Is accepting right at 9, or should
  the try show more before responder can accept?
- Weak two-over-one hands (ten-counts with shape) reach positions where
  nothing but a preference fits. `responses.bid` decides which hands bid
  a two-over-one at all; if it required 11 notrump points or a real
  suit, several of these would not arise.

## Responder's second call once they have come in (2026-09-23)

Written alongside the contested opener rebids (`rebids.notes.md`),
because opener's new calls need an answer and because responder's own
second call was the next dead turn in every one of those auctions.

Opener's contested rebids set `ask=signoff` or `ask=invite(suit)`
wherever they are limited, and the state blocks in `rebids.bid` answer
those whatever the auction was. Four things set no ask, and each has a
block here:

- **opener's answer to our negative double** (`1x (*) X (*) 1z/2z/3z`,
  and a separate one for his notrump answer). The double showed 6+ and
  four cards in each unbid major, so responder is usually done: BBA
  passes 16 of the 24 positions in the Basic-Bridge scenarios with 6-10.
  We pass to 10, raise his suit with four and 11-12, bid the game with
  13, and put notrump below the fit (`priority -1`) because opener's
  answer may be a three-card major — `partner.z>=3`, not `>=4`, is what
  lets the raise fire at all.
- **opener's rebids after our redouble**, and the auctions where he
  passes them back: responder doubles with four of their suit, goes back
  to opener's suit with three, or bids his own five-card major. BBA bids
  2x/3x on these with 10-11 and never doubles; we keep the penalty
  double, because the redouble said we own the hand and somebody has to
  be able to collect.
- **opener's reopening double** (`1x (2y) P (P) X`): responder takes it
  out in his longest suit, passes it with four of theirs, or returns to
  opener's suit.
- **opener's second suit over their call** (`1x (*) 1y (*) 1z/2z`), the
  mirror of the uncontested blocks above. Our own rules make a contested
  reverse forcing, and 58 corpus broken forces were responder with no
  answer to one.

Two blocks are pass-by-rule rather than pass-by-accident: when both
partners have passed and they have stopped (`1x (*) P (*) P (P)`, which
BBA passes 6 of 6 with 3-9), and when they bid over our negative double
and opener passed it out — there responder needs 11+ to bid again.

### Numbers

Included in the totals in `rebids.notes.md`. On their own these blocks
were worth +26 calls and about -390 no-rule points on the Basic-Bridge
subset,
and corpus-wide they closed `1x (1y) X (P) 2x (P)` (1,235 boards),
`1x (1y) X (2y) P (P)` (958), `1x (1y) X (P) 2z (P)` (738),
`1x (1y) X (3y) P (P)` (724), `1x (X) XX (1y/2y) P (P)` (1,471),
`1x (2y) P (P) X (P)` (646) and `1x (1y) X (P) 1N (P)` (404).

### Open questions

- Responder's double of their escape after our redouble is written as
  four cards in their suit and nothing else. BBA never doubles there, so
  the corpus cannot settle whether that is too free; it needs a probe of
  its own.
- `1x (1y) X (P) 2x (P)` is written against `z` bound to opener's
  answer, which may be his own suit repeated. A repeat shows six, so
  raising it with four is an eleven-card fit and the rule is generous
  about the level. BBA's sample here is three hands.

## Total points count length (2026-09-23)

Rick: "our rules should have HCP, total points and support points, and
decide which to use under which circumstances, but total points should
include length points." `points` is now HCP + ½ per ten + 1 per card
beyond four, so every band here moved: a six-card suit adds two, and the
bands (weak to 10, invitational 11-12, game 13+) now read as total
points. Measured against the old count on this module alone (every
`points` written as `points-length_points`), the new count is worth
**+9,968 IMPs** over the corpus: responder's games after 1m-1x-1NT and
over opener's 2m rebid and new suit, with length counted, are where par
gains most. So the bands stay in total points and the expectations moved
with them.

### Rule changes

- **Gaps the new count opened.** With 13+ total points and a long suit
  some hands matched no rule and passed:
  - after opener's 1NT, `4M` with six was measured in `suit_points>=13`
    (HCP + ½ per extra card), so 11 HCP with six hearts (13 total) was
    too strong for the invitation and too weak for game. It is now
    `points>=13`, the same measure as the band below it.
  - 3NT after opener's 1NT, 2m rebid, or second suit at the two level
    required shortness in a minor (`y<=5`, `x<=2`, `z<=3`). With a minor
    that is where the game is anyway, so the shape limit now applies only
    to majors (`| y is C | y is D` and the like).
  Together +2,040 IMPs (741 boards changed, none of the groups worse by
  more than 5).
- **1M-1y-2M: 2NT needs a singleton in opener's suit.** With a doubleton
  facing six, the raise to three (10-12) is the invitation: an eight-card
  fit is safer than notrump. +47 IMPs (240 boards).

### Expectations changed (the new call is sound, par agrees)

- `J65.KJ9643.A5.Q7`, 1♣-1♥-1NT: 4♥ (13 total), was 3♥.
- `AKJ73.Q942.Q.T72`, 1♣-1♠-1NT: 3NT (13½), was 2♥. 2♥ is not forcing;
  with 13 facing 12-14 game has to be bid. (No forcing way to show the
  hearts on this card.)
- `KJT865.J3.J52.A4`, 1♣-1♠-1NT: 3♠ (12½), was 2♠.
- `A742.KQ9765.73.T`, 1♣-1♥-2♣: 3♥ (11), was 2♥. Measured: keeping this
  block's own-suit bands in the old count costs 208 IMPs.
- `87.KQT763.QJ8.A2`, 1♣-1♥-1♠: 3NT (14½), was 3♥.
- `A85.QJ975..QJ542`, 1♦-1♥-1♠: 2NT (12), was 2♣. See below.
- `Q843.J6.KT.A9532`, 1♥-1♠-2♦: 2NT (11½), was 2♥.
- `3.85.QT97.AQ9642`, 1♠-1NT-3♠: 3NT (10½), was pass. Measured: the old
  count for this 3NT costs 48 IMPs.
- `T4.AT943.K5.KJ76`, 1♠-2♥-2♠: 3NT (13), was 2NT.
- `KQ83.K9.Q8763.95`, 1♥-1♠-2♥: still 3♥, now because of the 2NT change.

New cases for the gaps: 1♣-1♦-1NT with six diamonds, 1♣-1♠-2♣ with three
clubs, 1♦-1♥-2♣ with four clubs, all 13 total: 3NT.

### Tried and rejected

- 2NT over opener's one-level second suit only without a singleton or
  void (`shortest>=2`), with 2♣ after 1♦-1♥-1♠ widened to 8-12: -326
  IMPs. Most such hands then passed, and par prefers the invitation.
- Only a void in opener's first suit barred from that 2NT, 2♣ widened to
  8-12: -26 IMPs on 16 boards. So `A85.QJ975..QJ542` bids 2NT.

### Numbers

`measure.sh` (corpus / Basic_* / Basic_* uncontested NS), par net IMPs
against BBA: -182,498 / -2,153 / -1,030 → -180,411 / -2,104 / -993.
Now: calls 75.3% / 83.3% / 89.1%, identical auctions 14.4% / 32.8% /
45.8%, same contract 30.5% / 44.1% / 57.4%.

### Open questions

- 2NT with a void in opener's first suit (1♦-1♥-1♠-2NT holding
  3-5-0-5): par slightly prefers it to a natural 2♣ over 16 boards. Is
  that a sample accident, or should the 5-5 hand have a forcing call?
- 5-4 majors with game values after 1♣-1♠-1NT: 3NT now, because 2♥ is
  not forcing and the Basic card has no checkback. Should a jump to 3♥
  be natural and forcing there?

## BBA treatment (2026-09-24)

Two rules under `when style is bba` (module param `style =
general.style`); the default is unchanged.

- **Accepting opener's 3M game try** (1M-2M-3M): BBA accepts with 10 HCP
  or with four trumps and a singleton, and declines a flat 9 (Basic_*
  uncontested: `A84.K98.T32.K853` 4♠, `875.QJ8.Q64.A752` 4♠,
  `J72.A52.J86.K876` and `953.986.J86.AKJ2` pass, `T865.4.QT4.A9765` and
  `9732.KJ7.J.QJ952` 4♠, `Q65.2.K943.KT632` three trumps pass). Rule: 4M
  with `hcp>=10 | (M>=4, shortest<=1)`, else pass. 1♠-2♠-3♠: 8 → 12 of 13
  agree.
- **Raising opener's rebid minor** (1♦-1♥-2♦): BBA raises to 3♦ with 7-9
  and three or more diamonds (4 of 7 such hands; the passes had two
  diamonds or 6 HCP). Rule: 3x with 3+ and 7-9 points, `ask=signoff`.

Responder's second call overall (Basic_* uncontested NS): 72.2% → 72.6%
(1,560 positions). What is left is spread thin (no divergence above 5
boards): 1♠-2♥-2♠ BBA 3♠ with 13-15 where we bid 4♠ (4 of 20; BBA's 3♠
is forcing, and opener has no rule after it, so it is not modelled);
1♥-1♠-3♥ BBA pass where we bid 3NT (3 of 17); 1♥-1♠-2♦ BBA 4♥ where we
bid 3♥ (3 of 24).

## A minor fit: stoppers, not control bids (Rick, 2026-09-25)

Rick: "We don't usually use control bids for minors. In an auction
1S-2D-3D, for example, new suits are usually stoppers, looking for 3NT,
with 5D as a backup contract."

After 1M-2m-3m (or 1m-2m-3m), responder with game values:
- **3NT** when every suit is covered: my stopper, one partner has shown,
  or a suit either of us bid naturally.
- **Otherwise the cheapest stopper below 3NT.**
- **Otherwise the cheapest call below 3NT** (often opener's suit), which
  denies what it skips.
- **Otherwise five of the minor.**

The same `ladder` record as control bids keeps what each hand showed and
skipped. Opener answers the same way: 3NT with everything covered, the
next stopper up the line, else 5m. Before, responder bid 3NT on any
game-going hand.

Results: +157 by distance, +331 to the bidding side (290 boards).
Every auction family gained.

**For Rick:** the card field `major_openings.two_over_one.game_force` is
mapped from `.bbsa` but no rule reads it. Our 2/1 responses and
continuations are the SAYC ones, one round forcing, even on the 21GF
cards (e.g. "P: minimum for the two-over-one" after 1y-2x-3x). The
stopper dialogue sets the game force itself; the 2/1 structure as a
whole is a bigger job.

## No-rule gaps after 1NT and after opener's major rebid (2026-09-25)

Found by the no-rule scan of Basic_* N/S auctions:

- **1x-1NT-2x.** The pass read `points<=10`, and a 1NT response is 6-10
  HCP, so with length points 11-12 and a doubleton there was no call.
  The pass is now in HCP. BBA raises with a doubleton from 8 HCP
  (`probes/resp-1H-1N-2H.toml`: 60 of 79 hands with 8-10 and two
  hearts), an eight-card fit opposite six, so 3M with two and 8+ is
  now an invitation. Over a minor it lost by the side yardstick
  (1♦-1NT-2♦, -81 IMPs to the raising side over the corpus though +47 by
  par distance), so it is majors only.
- **1x-1NT-2z** with one card in each of opener's suits: 3m with six,
  to play.
- **1M-1y-2M**: 4M was measured in `suit_points>=13` and the invitation
  below it in `points`, so 12 HCP with a fifth card (13 points, 12.5
  suit points) had no call. 4M is now `points>=13`, the fix made after
  1NT earlier. BBA bids 4♥ on Basic_Weak_2 420.

Measured together with the 2NT and transfer fixes: Basic_* N/S vs BBA
+18 (+11 by side), full corpus +58 by par distance, +15 by side.

Also (2026-09-25): over opener's 2NT rebid, 3NT with a six-card minor
(it needed five or fewer in responder's suit, so Q8.QJT4.JT8754.Q had no
call; BBA bid 3♥ there, a four-card suit we do not show); and after
1m-2m-3m (opener's invitation, 17-18 support points) 3NT with 9, pass
otherwise (9 measured +5 / +18 against 8).

## Contested: responder after a one-level response and opener's minimum (2026-09-25)

Opener's contested minimum rebids set `ask=signoff`, and responder
answered every sign-off with a pass, so 1♣ (1♦) 1♠ 2♣ was passed with
13 where 1♣-1♠-2♣ goes on to 3NT. Contested versions of the uncontested
continuations now outrank that pass: after opener's rebid of his suit,
a raise of responder's, or 1NT, invite with 11-12 and bid game with
13+; notrump needs their suit stopped, and without it a cue bid forces
to game. Full corpus +401 by par distance, +280 by side (269 boards).

## Two-level free bids: opener's forced rebid and responder's second call (2026-09-25)

Once the forcing free bids forced (after-interference.notes.md), the
auction died in opener's rebid: opener had no call with a four-card
major or extras (rebids.bid), and responder none after it. Added:
opener's four-card major, a raise with extras forcing to game, a
forced fallback (only when the auction is forcing); responder game with
13+ (3NT with their suit held, else a cue bid), an invitation with
11-12, a pass with a minimum; after opener raises a three-level minor
to four, 5m with 13+.

**A misread found on the way.** Opener's contested minimum rebids set
`ask=signoff`, and `when asked signoff` (rebids.bid) holds opener's
answers to responder's sign-offs. Read as responder's, its 3x ("six or
more, 16-18, one more try") took over responder's cue bid of their
suit, and opener raised it: 1♦ (1♠) 2♣ 3♣ 3♠ 4♠. That block is now
opener's only (`!partner.opened`), with a plain pass for responder.
The doubler had been bidding on through the same misread; it now has
its own rebid after their responder bids and opener rebids
(takeout-double.bid). Together: full corpus +974 by par distance,
+2,252 by side (2,088 boards); Basic_* competitive +1 / +18, N/S
unchanged.

Follow-up the same day: responder's contested invitations now set
`ask=invite(...)`, so opener answers them with the existing rules
(before, opener had no rule and passed), and every contested cue bid
requires `cheapest_rank(<their suit>) <= 18`, below 3NT (a cue at 4♣
over 3♥ had appeared). Full corpus +384 by par distance, +625 by side;
corpus no-rule positions 4,938 -> 4,661.

## After our redouble, their runout and opener's rebid (2026-09-26)

Second group of the missed-games queue (`probes/tools/missed_games.py`):
1x (X) XX (runout) 2x, where responder, having shown 10+ with the
redouble, read opener's rebid as a sign-off and passed with 10-13.
Now: game with 12+ (4M with a doubleton, 3NT with their suit stopped,
5m from 13), 3x inviting with 10-11. Full corpus +2,025 by par
distance, +2,568 by side (545 boards, mostly after 1♥/1♠ (X) XX).
Same day: when opener passes their runout back (a forcing pass),
responder with 12+ now bids game (3NT with a stopper, 4M with three,
else a cue bid) instead of going back to opener's suit, which is now
10-11. Full corpus +979 by par distance, +913 by side.
And when they jump over our negative double and opener passes, responder
with 13+ bids 3NT (their suit held) or game in opener's suit; the
preference is now 11-12. Full corpus +262 by par distance, +515 by
side.
