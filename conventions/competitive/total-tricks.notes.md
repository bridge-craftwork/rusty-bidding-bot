# total-tricks (`total-tricks.bid`): notes

## Guidance (Rick, 2026-09-27)

From Basic_Takeout_Double 193 (1♥ (X) P (2♥) 3♥ P P, North
T3.J.AJ8763.KJ87; BBA 4♦, our engine had no rule):

- When both sides are actively bidding and the HCP look roughly equal,
  compete to the level where the tricks needed equal our total trumps:
  South's double nominally shows four diamonds, North has six, so 4♦.
- The double can be counted because South cannot hold the power
  double: after North's cue bid a power double would have acted, and
  the deck (North 10, East about 16) leaves South too little. These
  are likelihoods, not certainties: play for the 19 hands in 20.
- Treat the double's three-card minimum as more likely four: a double
  of a major nearly always has the other major, of a minor less so.
- Vulnerability tempers it: at unfavourable, hedge by a trick.

## How it is built (steps a-d, each measured)

- a. The double's power branch in HCP (17+), the overcall to 16
  (takeout-double.notes.md): BBA's power doubles step up at 17 HCP.
- b. Engine: the 40-HCP deck caps every seat, publicly and from the
  chooser's own hand; the cap collapses the double to its shape.
- c. The doubler's pass denies the power double (takeout-double.bid).
- d. This module: my length plus partner's shown minimum, plus one
  when partner's three is a takeout double of their major (four is the
  norm there: other major 68%, a minor 73%;
  `probes/tools/takeout_shape.py`), bid exactly that level less six, at
  most the four level, one lower vulnerable against not. Needs 5+ HCP
  of my own and not clearly our hand (my HCP plus partner's minimum at
  most 24). A sign-off (`ask=signoff`): partner passes. Priority -4,
  below every rule that describes a hand.

## Measured (full corpus, par distance / side; Basic_* competitive)

| variant | full corpus | Basic_* comp. |
|---|---|---|
| any time they have bid, no floor | +1,723 / +16,098 | +98 / +280 |
| still bidding, no floor | +468 / +17,438 | +84 / +346 |
| still bidding, 5+ HCP | +1,996 / +12,084 | +79 / +237 |
| still bidding, 7+ HCP | +1,673 / +8,818 | +103 / +224 |
| any time, 5+ HCP, sign-off | +3,486 / +11,559 | +101 / +182 |
| **still bidding, 5+ HCP, sign-off (kept)** | **+2,052 / +12,162** | **+79 / +237** |

"Still bidding" = one of the opponents' last calls was not a pass,
Rick's "both sides actively bidding". Without it the rule scores more
par (+3,486) but fires once they have gone quiet: a minimum doubler
raised advancer's 0-8 1♠ to 2♠, and responder overrode opener's
decline of the limit raise. Those are wrong bridge, so the restriction
stays. Basic_* N/S: -1 / +6. Failed games in contested auctions rose
(2,133 -> about 2,488 in the unrestricted run), expected from a
competitive rule; par still gains.

Two cases in the test files that expected a pass now compete, as the
rule says: 1♠ (2♦) X (3♦) 3♥ with six diamonds, 4♦; 1♠ (X) 2♠ (3♣) P P
with four spades, 3♠.

## Open

- **For Rick:** the unrestricted version's extra par (+1,434) comes
  from contested partscore auctions where the opponents paused for a
  round (raising partner's balancing bid, the doubler raising
  advancer). Some of that may be worth a narrower rule.
- The hedge is one level at unfavourable only; favourable is the plain
  count.

## Not after our game (2026-09-27)

Ticket basic-takeout-double-b100: `1H X P 2H 3H 3NT P 4C`, the LoTT
pulling partner's 3NT to four of a six-card minor. Rick: we do not pull
partner's 3NT to a minor unless looking for slam; the six clubs are
tricks in notrump and 3NT promised the heart stopper. The context now
requires `!game_reached`. Tiny on the corpus (5 boards: +20 par, −12
side IMPs), adopted on Rick's ruling. Keeping the LoTT out of the whole
game force was worse: it sent a hand with a singleton heart jack to 3NT
(the 4♦ case in total-tricks.test).

## Three rules instead of twelve (2026-09-30)

The judgment layer's Phase 0 (docs/JUDGMENT-LAYER.md) rewrote the file
with named conditions: `doubler_four(x)` (partner's three counts four
after a shape takeout double of their major) and `lott_trumps(x)`
(`we.fit(x).min + doubler_four(x) - unfavourable`, a condition counting 1
or 0). One rule per level, with a guard, `doubler_four(x) |
!doubler_four(x)`, that keeps the old behaviour of firing only once the
double's shape is known either way. On the full corpus our calls did not
change (170,633 boards, identical auctions and replays); only the
explanations did: the "a trick hedged vulnerable" and "partner's double
counted as four" wordings are gone from 577 calls, which now read as the
plain level. The context uses `they.still_bidding` for `!lho.last=P |
!rho.last=P` (the same test).

Measuring it exposed a cache bug in the engine: descriptiveness was
cached by rule and calls only, so a `shows` reading the vulnerability
(`unfavourable`) took whichever board filled the cache first when boards
are bid in parallel. The key now carries the values of such terms.

## The weak raise to two only against a bid (2026-10-02)

"Still bidding" (one of their last calls is not a pass) let the
two-level rule raise partner's one-level overcall on 5-6 HCP and three
trumps after `(1D) 1S (P)`: opener had bid once and responder passed,
so nobody was competing. BBA on bare SAYC passes these (raises from
about 9 HCP with three trumps, 7 with four). The two-level rule now also
needs `they_compete`: the last bid or double at the table is theirs,
made over our side's last bid (RHO's call, or LHO's when partner and RHO
have passed since).

Measured on top of the minor-raise change (errors yardstick, IMPs; even
/ odd boards):

| set | boards | errors | contract / doubling | distance from par | side IMPs |
|---|---|---|---|---|---|
| vanilla SAYC random | 218 | +152 (+114 / +38) | +19 / +133 | −16 | +200 |
| corpus | 272 | +126 (+45 / +81) | +101 / +25 | +94 | +60 |
| 21GF random | 189 | +152 (+121 / +31) | +29 / +123 | −5 | +175 |

**The same restriction on all three levels was tried and not kept.** It
gained on the errors line (+393 vanilla, +383 corpus, +426 21GF, both
halves) but only through the doubling errors (+1,482 doubling against
−1,089 contract on vanilla), and distance from par lost 753 / 916 / 679.
By level: the two level gained on every count; the three and four levels
lost on contract errors and par everywhere and gained only because our
defenders, who double 3.4% of the chances against BBA's 13.8%, were no
longer given a chance to miss a penalty double. A gain that depends on
our weak doubling is not one to bank (penalty-doubles.notes.md), so the
three and four levels stay as they were (e.g. (2♠) 3♥ (P) 4♥ with ten
trumps, a raise they have not bid over yet, gained under both).

## Sources

- **The principle:** the Law of Total Tricks (LoTT), compete to the level
  of the total trumps. Standard practice, not yet cited to a book or
  article in these notes.
- **Rick's rulings:** the guidance from Basic_Takeout_Double 193
  (2026-09-27): compete when both sides are bidding, count the double as
  four in the other major, play for the 19 hands in 20, hedge a trick at
  unfavourable; do not pull partner's 3NT to a minor unless looking for
  slam (2026-09-27, ticket basic-takeout-double-b100).
- **BBA evidence:** BBA's 4♦ on the ticket board; the takeout double's
  shape from `probes/tools/takeout_shape.py`; BBA's power doubles from 17
  HCP (`takeout-double.notes.md`).
- **Corpus measurements:** the variant table, judged by par distance and
  by side IMPs; "still bidding" is kept although the unrestricted rule
  scores more par, because the extra cases are wrong bridge.
- **The two-level restriction** (2026-10-02): BBA's raises of a one-level
  overcall on the vanilla SAYC random set (`(1x) 1S (P)`: 2♠ 196 hands
  with three spades from about 9 HCP, pass 198 with 5-8), and the
  measurements in its section.
