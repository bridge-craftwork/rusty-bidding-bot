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
  with no cap. There is no Precision 1♣ module yet, so this is the honest
  reading of "off", not Precision.

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
