# soloway (`soloway.bid`): notes

Soloway jump shifts (`general.jump_shifts.soloway`): responder's jump to
two of a higher suit over 1♣, 1♦ or 1♥ is game forcing and shows one of
GIB's four hand types. The jumps are in `responses.bid` (`soloway_js`);
the continuations here. Cases: `soloway.test`. Scenarios:
Soloway_Jump_Shift and Soloway_Jump_Shift_Type-1..4 (21GF-SPECIALS2).

## Structure

- Types (4+ controls each): (1) six cards with four of the top five, 17+
  total points, no side four-card suit; (2) a solid suit (AKQJ), 17+;
  (3) five cards with three of the top five, 18+ HCP, 5-3-3-2 or 6-3-2-2;
  (4) five cards with three of the top five, four-card support for
  opener, 17+ support points. Suit tests as in the scenario's own script.
- Opener: raises with four, or three with an honour (priority); rebids
  his suit with five or more; shows a side suit with two of the top three
  honours; else 2NT.
- Responder: raises opener's suit (type 4), rebids his suit (types 1-2),
  or bids 3NT (type 3). From there the agreed suit leads to slam/.
- On a card with Soloway, strong jump shifts are off; weak ones win if
  both are set (no PBS card does).

## Corpus (Soloway_Jump_Shift*, 2026-10-05)

Same final contract as BBA on 38 of the first 250 boards (50 a
scenario) before (strong jump shifts), 76 after. With the jumps but
without the continuations calls agreed 73.1% (all 2,500 boards), with
them 78.3% (NS 48.1% to 58.4%). Remaining divergences at the jump, all
**BBA style**, not changed (GIB's definition is the spec):

- BBA never jumps 1♣-2♦ with 17 HCP (all 2♦ jumps have 18+), where GIB's
  "17+ total points" does.
- BBA does not jump with a balanced hand and a five-card major (type 3):
  1♦-1♥, then NMF/Roudi.
- BBA does not jump with a side four-card suit even with a solid suit
  (type 2), nor with a five-six two-suiter.
- With four clubs over 1♣ BBA raises (inverted 2♣) instead of type 4.
- Opener: BBA rebids a five-card minor (we do too; GIB says six) and
  raises three cards only with an honour (we follow BBA).

## Gaps

- After opener's 3♣ rebid and responder's 4♣, opener's keycard 4NT and a
  5♦ answer leave no sign-off below 5♦; we pass 5♦ (about 30 boards in
  these scenarios; a slam/ issue, rkcb-1430.bid).
- Soloway extended (`soloway_extended`) not built.

## Sources

- **GIB system notes, "Soloway Jump Shifts"**
  (https://www.bridgebase.com/doc/gib_system_notes.php#Soloway_Jump_Shift):
  the four types and both players' rebids. convention-card
  `spec/conventions/bidding_conventions/soloway.toml` cites the same page.
- **BBA evidence:** the Soloway_Jump_Shift* corpus (2,500 boards): the
  BBA-style differences above; opener's raise needs an honour with three.
