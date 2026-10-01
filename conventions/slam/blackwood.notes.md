# blackwood (`blackwood.bid`): notes

Standard Blackwood (card `slam.blackwood.standard`, the Basic-Bridge card):
4NT asks for aces once a trump suit is agreed; 5♣ 0 or 4, 5♦ 1, 5♥ 2, 5♠ 3.
The asker bids six with at most one ace missing, else signs off in five
(passing an answer in the trump suit). Cases: `blackwood.test`.

Added because BBA asks with 4NT on the Basic card and our side used to pass
the ask. Aces are `keycards(N)`: keycards with no trump king, four in the
deck, so "0 or 4" resolves against the asker's own aces.

## When to ask

The two-branch test in the `# The ask` block is shared with
`rkcb-1430.bid` and is written up once in **`slam-catch.notes.md`**, with
the BBA evidence behind the boundaries. It is Rick's rule: **33 combined
support points** with the agreed suit as trump, read against partner's
maximum when partner is limited and against partner's floor in a game
force, plus at most four losers and **no two bare side suits**.

`we.tp(trump)` is what makes the combined count sayable at all; before a
raise recorded support points the rule had to use `partner.hcp.max`. The
bare-suit test replaced `controls >= 7`, which counted top cards without
knowing which suit could lose the first two tricks; on the Basic-Bridge
card BBA draws the same line (272 boards asked with none or one bare
suit, none with two).

Basic-Bridge has `Cue bid = 0`, so `control-bids.bid` is not loaded and
4NT is still the whole of the slam machinery on this card. The
`asked control(t), answered control(u)` rule is there for a card that
plays standard Blackwood *and* control bids; no corpus card does.

## Gaps

- The 5NT king ask (card `slam.king_ask.five_nt`) and grand slams.
- **The queen ask.** After 4NT-5C ("0 or 4") BBA asks for the trump queen
  with the next step (5D, alerted "spades queen ask"), and the answer
  names a king as well ("!S queen and !C king"); it then places the
  contract. We bid six directly on the ace count. 55 boards, -30 IMPs
  against par. `slam.blackwood.queen_ask` exists in the field registry but
  no `.bbsa` key is mapped to it, so it cannot be switched on from a
  corpus card yet.

## Shared conditions named in slam-entry.bid (2026-09-30)

The long conditions of blackwood.bid (up to 1,300 characters a line) are now
named conditions in `slam/slam-entry.bid`: `slam_values`,
`slam_values_limited`, `controlled(x)`, `first_round(x)`,
`side_suit_uncontrolled`, `two_bare_suits`, `denials_promised`,
`all_covered` and their variants. Each is exactly the condition it
replaced; a definition reads `style` from slam-entry.bid, which names the
same card field and default (`slam.cue_bids.style`,
`first_or_second_round`). The full corpus bids identically
(docs/JUDGMENT-LAYER.md, Phase 0). `two_bare_suits` uses the new
`bare(x)` (a side suit of 2+ cards without the ace or king), which, like
`has`, is not recorded when a call is read, so the ask's negative
inference is unchanged; `bare_suits <= 1` would say the same but would
add that denial, a change to measure on its own.

## A fit known but not agreed (2026-09-30)

With no trump suit set, a public fit in a major (eight shown between us,
or partner's six-card suit) and slam values on partner's floor (or his
top when he has limited his hand, as a preempt does), 4NT agrees the
major and asks (`sets trump=x`). Uncontested only. Evidence and figures:
slam-entry.notes.md, "Phase 1".

## Sources

- **The convention:** standard (plain) Blackwood with the usual ace
  answers, as the Basic-Bridge card plays it. Standard practice, not yet
  cited to a book or article.
- **Rick's rulings:** when to ask (33 combined support points, the
  two-branch test, no two bare side suits), 2026-09-23; written up in
  `slam-catch.notes.md` and quoted in `control-bids.notes.md`.
- **BBA evidence:** BBA's asks on the Basic-Bridge card (272 boards asked
  with none or one bare side suit, none with two), re-bid with
  `bba-cli --all-meanings`, and the `rbb probe` boundary hands, all in
  `slam-catch.notes.md` "Evidence". No `probes/*.toml` spec yet.
- **Corpus measurements:** the queen-ask gap (55 boards, -30 IMPs).
- **Where we differ:** no queen ask after 4NT-5♣, where BBA asks for the
  trump queen with the next step (Gaps).
