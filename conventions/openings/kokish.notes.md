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

## Transfers over every 2NT, and 2♣–2♦–3NT (Rick, 2026-11-07)

Rick: "people playing jacoby transfers normally play them over 1NT, 2NT,
2C/2NT, and kokish relay, and sometimes 2C/3NT".

- **Over the relay's 2NT the transfers were already on** (systems on,
  2026-10-05), but a five-card major with a slam invitation bid 4NT: the
  2NT system's quantitative 4NT and 6NT had no "with five, transfer
  first" condition, and opposite 24-25 a 7-count (84.KJ863.Q72.J54,
  KJ863.84.Q72.J54) is a slam invitation, where opposite 20-23 the same
  hand is game only and transferred. Now 4NT and 6NT over any `nt2` need
  no five-card major when the card plays Jacoby (two-nt-responses.bid):
  transfer, then jacoby-transfers.bid's "Quantitative, 5 M" 4NT. The 2NT
  opening changes the same way (12 with five hearts: 3♦ then 4NT, not
  4NT), as over 1NT already.
- **2♣–2♦–3NT with Kokish on is 26-27** (strong-openings.bid): the
  24-25 goes through the relay, so the direct 3NT is the next rung.
  Rusty read it as 24-25 and passed with 7 (31-32 on that floor); now 7
  bids a quantitative 4NT (33-34, over-3nt.bid). BBA on Kokish_Relay
  (Kokish and Gambling): 2♣–2♦–3NT 85 boards, opener 26 HCP on 59 and 27
  on 26; the relay's 2NT 93 boards, all 25. Bridge Bum gives no Kokish
  meaning for the direct 3NT. Without a Gambling 3NT the 26-27 opens 3NT,
  so we never rebid it; the reading still matters when a human does.
- **Transfers over 2♣–2♦–3NT** ("sometimes"): no card field; they follow
  the card's Texas setting (texas-transfers.bid, since 2026-10-29). A
  six-card major with 4-5 HCP fell between Texas's game-only and slam
  bands opposite 26-27 and passed 3NT; a fallback Texas to game now takes
  it. **For Rick:** a card field "transfers over 2♣–2♦–3NT" (a
  convention-card pull request) if the "sometimes" should be switchable.
- **Measured** (compare --par-all, all 500 boards of Kokish_Relay,
  Game_Forcing_2C, NT_Ladder, SCS_Two_Clubs, Bust_Over_Strong_2C, 2N,
  2N_and_Balanced, 2N_and_Transfers, Smolen_after_2N, Puppet_Stayman_2N,
  Texas_Transfer, Texas_or_Jacoby, Gerber, Gerber_By_Responder,
  Slam_after_Transfer; `errors_diff.py`): 51 boards changed
  (2N_and_Transfers 42, 2N 5, Kokish_Relay 2, Game_Forcing_2C 1,
  NT_Ladder 1). VERDICT, the actor's contract errors: +13 IMPs (halves
  +0 / +13), distance from par +13; vs BBA contract −628 → −615. The
  transfer-first boards (2NT–4NT/6NT → 3♦/3♥ then 4NT/6NT) score the
  same: the same contract, the suit shown on the way. The +13 is two
  2♣–2♦–3NT boards where Texas now reaches 4♥ instead of passing.
  Tripwire, all scenarios --limit 50: 9 boards changed (2NT openings
  transferring first, one 2♣–2♦–3NT Texas), 0 IMPs either way.
  Kokish_Relay itself moves little: its card (21GF-SPECIALS) has a
  Gambling 3NT, so the old reading was already 24-27 there.

## Sources

- Bridge Bum, "Kokish Relay", https://www.bridgebum.com/kokish_relay.php:
  the relay, the forced 2♠, the rebids. Deviation: the balanced range
  (24-25 instead of 25+) to fit our ladder.
- Practice-Bidding-Scenarios `btn/Kokish_Relay.btn` and `bba/`.
- Rick, 2026-11-06 (bridge-classroom-bug-artifacts #92): the board and
  his note; the card question is open for him.
- Bridge Bum, "Kokish Relay" (re-read 2026-11-07): the relay's 2NT "25+
  points, game-forcing", footnote "2NT can be eased to 24+ points"; the
  worked example has responder's 3♦ over it as a "Jacoby transfer". The
  page gives no meaning for 2♣–2♦–3NT with Kokish; its non-Kokish table
  has 2♣–2♦–3NT 25-27. Our 26-27 is the ladder's next rung and BBA's.
- Rick, 2026-11-07: transfers over every 2NT and "sometimes" 2♣–2♦–3NT
  (quoted above).
