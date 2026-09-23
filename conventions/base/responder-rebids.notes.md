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
