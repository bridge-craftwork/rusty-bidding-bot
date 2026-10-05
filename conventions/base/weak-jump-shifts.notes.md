# weak-jump-shifts (`weak-jump-shifts.bid`): notes

Weak jump shifts by responder over a one-level suit opening. The jumps
are in `responses.bid` (card fields `general.jump_shifts.weak_2` and
`weak_3`); opener's continuations and responder's answers are here.
Cases: `weak-jump-shifts.test`. Scenario: Weak_Jump_Shift (21GF-WJS-MSS).

## Structure

- Two level (1♣-2♦/2♥/2♠, 1♦-2♥/2♠, 1♥-2♠): six cards, 2-5 HCP. Over
  1♥, 2♠ also denies three hearts (BBA passes those hands).
- Three level (1♦-3♣, 1♥-3♣/3♦, 1♠-3♣/3♦/3♥): seven cards, 2-6 HCP.
- Opener after a two-level jump: pass; 4 of a major with 18+ and two or
  more; 3y (16-17, three cards) invites; 2NT asks (16-17 with three, or any
  strong hand with a minor jump): responder bids 3y (minimum, or no
  feature), a feature with 4-5, 3NT with 4-5 and two of the top three;
  3x with seven of opener's suit, not forcing; a new suit 17+, forcing.
- Opener after a three-level jump: pass; 4M with 17+ and two cards; 3NT
  with 18+ and a fit for the minor; 4 of his own major with seven (or six
  good ones) and 16+.
- Card precedence: weak beats strong at two (21GF-WJS-MSS sets both;
  BBA plays weak), and weak beats invitational at three (21GF-MSS,
  -Multi, -NoInvertedMinor set both; not probed).
- `responder-rebids.bid`'s 2/1 continuations and `rebids.bid`'s answers
  to a 2/1 and to an invitational three-level jump are switched off after
  a weak jump shift (`two_level_jump(x, y)` here).

## Corpus (Weak_Jump_Shift, 2026-10-05)

Calls agreeing 79.8% before, 80.1% after (NS 76.5% to 77.0%). BBA's
range is 3-5 HCP at two and up to 6 at three. Remaining divergences at
the convention's calls are mostly EW's (BBA's GIB card competes over the
jump, ours does not), and BBA passing where our opener rebids 3NT with a
long minor (rebids.bid's 18-21 3NT, not this module).

## Open questions

- Weak jump shifts by a passed hand and in competition: the jump is weak
  in both here (Bridge Bum: on in competition). The card has
  `competitive.weak_jump_shifts_in_comp`; competitive rules are elsewhere.
- Weak and invitational both set: weak wins at three. Not probed.

## Sources

- **Bridge Bum, "Weak Jump Shifts"**
  (https://www.bridgebum.com/weak_jump_shifts.php): ranges (2-5 and six,
  2-7 and seven) and opener's options. We take 2-6 at three (BBA's
  corpus maximum is 6).
- **BBA evidence:** the Weak_Jump_Shift corpus (500 boards): 2♠ over 1♥
  denies three hearts; opener rarely rebids a six-card suit.
- **Where we differ:** opener's 3y raise as invitational and the feature
  answers to 2NT are standard practice (as after a weak two), not yet
  cited.
