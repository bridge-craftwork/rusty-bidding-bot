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
