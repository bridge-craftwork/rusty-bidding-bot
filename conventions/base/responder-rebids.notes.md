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
