# base (`base.bid`): notes

Natural one-level openings. Cases: `base.test`. The ranges and BBA's
style are explained in the comments of `base.bid` and in
`preempts.notes.md` (the ten-count with a six-card suit).

## Card fields read (2026-10-05)

- `minor_openings.may_hold_5_card_major` (default on): with a five-card
  major and a longer minor, open the minor. Off, a five-card major always
  opens the major (1♦/1♣ deny one). Off on 21GF-WJS-MSS and
  Precision-14-16. The field's own description says "usually a longer
  minor (6+) with reverse strength"; on, we keep the rule we had (the
  longer suit opens, whatever the strength).
- `general.forcing_opening_2c` (default on): the 21 HCP cap on one-level
  openings holds only while 2♣ is the strong opening
  (strong-openings.bid). Off (the Precision cards, whose forcing opening
  is 1♣), there is no strong 2♣, and a strong hand opens at the one level
  with no cap. A card whose forcing opening is 1♣
  (`general.forcing_opening_1c`, the Precision cards) keeps the strong
  2♣ and the cap as a stand-in until a forcing 1♣ is built: switched off
  there, the tripwire showed 22-counts opening 1♦ and being passed out.
  So today "off" bites only on a card with no forcing opening at all
  (no PBS card).

## An unread answer to a quantitative 4NT (Rick, 2026-11-06)

Bridge-Classroom #94 (strong-openings.notes.md has the board): North
asked with a quantitative 4NT, South answered 5♣, which no rule reads,
and North passed (the game-reached pass) in a 4-0 fit. A five-level
suit over a quantitative 4NT is never a place to stop below slam: the
asker now bids 6NT (`answered quant` / `answered quant_major(M)`,
`partner.unread`, 5♣-5♠, priority -40). Keycard answers are read and
have their own continuations in rkcb-1430.bid.

## Open questions

- With the field on, should a minimum 6-5 (minor longer) still open the
  minor? The field's description suggests the major unless strong enough
  to reverse. Not changed.

## Sources

- **Card fields:** convention-card `spec/fields.toml`
  (`minor_openings.may_hold_5_card_major`, `general.forcing_opening_2c`)
  and its `.bbsa` map ("1m opening allows 5M"; the system category's
  derived default for the forcing 2♣).
- **The openings:** standard practice (longest suit, higher of two
  five-card suits), as BBA's Basic-Bridge card plays them; not yet cited
  to a book.
- **Unread answer to a quantitative 4NT:** Rick, 2026-11-06
  (bridge-classroom-bug-artifacts #94); 6NT as the reading of the answer
  is standard practice (a five-level suit accepts and offers a strain),
  not yet cited.
