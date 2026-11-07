# Kokish relay: notes

## Decisions

- 2♣–2♦–2♥ = five or more hearts (unbalanced), or 24-25 balanced: the
  hand our ladder (strong-openings.bid) rebids 3NT. Bridge Bum's relay
  2NT is "25+"; on our ladder 26-27 opens 3NT, or with a Gambling 3NT
  opens 2♣ and rebids 3NT directly, and 28+ rebids 4NT. BBA (Kokish
  scenario): relay-2NT is exactly 25, direct 3NT 26-27, which matches.
- Balanced with five hearts: 2NT (22-23) or the relay (24-25).
- Responder's 2♠ is forced (priority 2, above the second-negative 3♣).
- After the two-suiter responder agrees hearts at the three level with
  three, raises the second suit with four, else 3NT; over 3♥ (six or
  more), 4♥ with two, else 3NT.

## Fixed on the way (strong-openings.bid)

- With a Gambling 3NT on the card, a balanced 26-27 had no opening at
  all (the 22+ 2♣ excluded it for the 3NT opening) and passed: 10 of the
  first 50 Kokish_Relay boards. It now opens 2♣ and rebids 3NT (BBA does
  the same). Affects every card with `Gambling = 1`.

## BBA (compare Kokish_Relay --limit 50, 2026-10-05: 87.8% of calls agree)

- After the relay's 2NT BBA plays Stayman (3♣) and transfers; we have no
  system over it and responder bids 3NT. Open: systems on over 2♣–2♦–2NT
  and the Kokish 2NT belong to the notrump modules.
  Checked 2026-10-05 (batch 2): not a gate's worth. Only Puppet
  (two-nt-responses.bid) is written over 2♣–2♦–2NT; Stayman and the
  transfers after a 2NT opening are `after 2N (P)` contexts in
  notrump/two-nt-responses.bid and two-nt-minors.bid (19 blocks). Adding
  the Kokish 2NT means extending those contexts, together with
  2♣–2♦–2NT: left to the notrump work.
  Done 2026-10-05 (Rick: "all the same responses as after 2C/2NT, just
  with modified HCP range for opener"): systems are on over the Kokish
  2NT (one-nt.notes.md, "Systems on"), and the 2NT system counts against
  its 24-25. Kokish_Relay: 5 of 50 boards changed, each to Stayman.
- Divergences at 2♣–P (positive responses) and in the opponents'
  overcalls are not Kokish's.

## Bridge-Classroom #92 (Rick, 2026-11-06)

NT Ladder (Lev) board 31: Rick (South, `A86.AQ6.AK4.AQJT`, 24 balanced)
bid 2♣–2♦–2♥ meaning Kokish, and Rusty (North) bid 3NT from base.bid's
"Game in notrump". Rick: "Card should have Kokish relay here but maybe
it isn't set up yet."

- **With Kokish on the relay works.** On the stand-in card (21GF-GIB
  with `two_level.kokish.play=true`) North relays 2♠ on every hand tried,
  six good spades and a balanced 11 included (priority 2, no condition);
  opener's 2NT (24-25) then plays the 2NT system: Stayman, transfers,
  Gerber (one-nt.notes.md, "Systems on"). `.test` cases added.
- **Exceptions to the 2♠:** none taken. Bridge Bum shows 2♠ only as
  "completing the relay" and names no exception (checked 2026-11-07);
  treating it as forced is standard practice, not yet cited. A hand
  with a long strong suit made a positive at its first turn
  (2♣–2♥/2♠/3m), so it is not waiting here.
- **So Kokish was most likely off on Rick's card.** The screenshot shows
  2♣ alerted and 2♥ not, and the Kokish 2♥ is alerted: on Rick's card
  2♥ was natural (five or more hearts, which Rick's hand does not have),
  and North had no rule for 2♣–2♦–2♥ but the second negative, so the
  base fallback bid 3NT. **For Rick to check:** is "Kokish Relay" on in
  "Dan Bergmann and Rick Wilson"?
- **Kokish off (strong-openings.bid, 2026-11-07):** responder over a
  natural 2♣–2♦–2M now shows a five-card suit of his own without a fit
  (2♠ over 2♥, 3♦), waits with 2NT with no fit, and keeps the second
  negative; opener over the 2NT rebids a six-card major, shows a second
  suit or bids 3NT. These are off when Kokish is on (`kokish` param).

## Sources

- Bridge Bum, "Kokish Relay", https://www.bridgebum.com/kokish_relay.php:
  the relay, the forced 2♠, the rebids. Deviation: the balanced range
  (24-25 instead of 25+) to fit our ladder.
- Practice-Bidding-Scenarios `btn/Kokish_Relay.btn` and `bba/`.
- Rick, 2026-11-06 (bridge-classroom-bug-artifacts #92): the board and
  his note; the card question is open for him.
