# Cappelletti (`cappelletti.bid`): notes

Cappelletti (Hamilton) and Modified Cappelletti over their 1NT, direct
and balancing seat. Cases: `cappelletti.test`. The answers every defence
shares (the majors, a major and a minor, the minors, the penalty double)
are in `vs-1nt.bid`; the comparison of the defences, the corpus figures
and how the opponents' calls are read are in `vs-1nt.notes.md`.

## Guidance (Rick, 2026-09-28)

"People play all of these, we'd like our bot to be compatible with their
cards." The card's `competitive.vs_1nt_strong.system` decides
(`cappelletti` from the `.bbsa` key `Cappelletti`, on for 21GF-DEFAULT,
21GF-GIB and eight other cards; `modified_cappelletti` is a new option
with no `.bbsa` key).

## The calls

| Call | Cappelletti | Modified Cappelletti |
|---|---|---|
| X | penalty, 15+ HCP | the same |
| 2♣ | one long suit, six or more (advancer relays 2♦) | a long minor, or 5-4 in a major and a minor (advancer relays 2♦) |
| 2♦ | both majors, 5-4 or longer | the same |
| 2♥/2♠ | that major (5+) and a four-card minor | that major, six or more, one-suited |
| 2NT | both minors, 5-5 or longer | the same |

After the Cappelletti relay the overcaller passes with diamonds and bids
2♥, 2♠ or 3♣; after the Modified relay he passes with diamonds, bids
2♥/2♠ with that major and a minor (then the 2♥/2♠ answers of vs-1nt.bid
apply) or 3♣ with clubs. The Cappelletti advancer may pass 2♣ with six
clubs or bid a six-card major of his own (BBA and GIB both do); the
Modified advancer always relays, since 2♣ may be diamonds or a major.

## BBA (probes, 21GF-DEFAULT, 2026-09-28)

`probes/vs-1N-capp.toml` (1,500 hands, direct seat): BBA's meanings are
exactly the table above: "Cappelletti, strong" X from 15 HCP, "any 6+
suit" 2♣, "both majors" 2♦ (4-4 or longer in its alert, 5-4 in practice),
"5M-4m" 2♥/2♠, "Unusual 2NT" 5-5 minors. The strength it needs:

- 2♣ from 9 HCP (half the hands at 9, 60% at 10, nearly all from 11);
- the two-suiters from 9 (5-5 at 9-10 always, 5-4 majors half the time
  at 9);
- 2♥/2♠ with 5-4 only about a third of the time at 9-14: it passes when
  the four-card minor has no ace, king or queen (AKQ73.KQ5.9.8732,
  72.AJT32.K4.J743 pass; K3.QT874.KJT8.54 bids). We ask for a top honour
  in the minor;
- 2NT from 6 HCP with 6-5.

We count HCP plus a point a card beyond four in each suit: 12 for the
one-suited 2♣, 10 for the two-suiters, 8 for 2NT. After the rules,
**1,302 of the 1,500 hands agree with BBA** (87%). The differences: BBA
passes 159 hands we bid (a 15-HCP double 42 times, where BBA passes 16 of
51 at 15; 2♣ 51 times at the bottom of the range), and we pass 18 it bids.

`probes/vs-1N-capp-bal.toml` (800 hands, balancing seat): the same
meanings, a point or two stronger (2♣ from 11 HCP). We use the direct
thresholds; balancing 2♣ lost 137 IMPs to the side that bid it (par
distance +99): see Gaps.

Advancer (`probes/vs-1N-capp-2C-adv.toml`, `-2D-adv`, `-2H-adv`,
`-2S-adv`, `-2N-adv`, `-X-adv`) and the overcaller's rebids
(`-2C-2D`, `-2H-2N`): recorded in vs-1nt.notes.md, where those rules
live.

## Accepted differences from BBA

- X at 15 HCP: BBA passes a third of the 15-counts. We double all of
  them (the side that doubled gained 2,186 IMPs over 1,367 boards on the
  corpus, par distance -1,286: vs-1nt.notes.md).
- A 5-4 major-minor hand with a poor minor passes, as BBA; with 15-17
  and no call it passes too (the double wants 15, so 15-17 doubles).
- The advancer after a one-suited answer raises a major with two cards
  (11-12 invites, 13+ bids game); BBA's continuations there were not
  probed.

## Sources

- Bridge Bum, "Cappelletti (Hamilton)", https://www.bridgebum.com/cappelletti.php
  (the calls; advancer's 2♦ relay, pass with long clubs, 2♥/2♠ natural;
  and its "Modified Cappelletti" section: 2♣ "a minor one-suiter, or at
  least 5-4 or 4-5 in a major and a minor", relay 2♦, then pass with
  diamonds, 2♥/2♠ with that major and a minor, 3♣ with clubs; 2♥/2♠
  "6+").
- Wikipedia, "Cappelletti convention", https://en.wikipedia.org/wiki/Cappelletti_convention
  (penalty double 15+; 2♦ majors "traditionally 5-5 but often 5-4").
- ACBL Unit 390 (Simon), "Modified Cappelletti",
  https://www.acblunit390.org/Simon/modcapp.htm (a variant where 2♣ is a
  *diamond* one-suiter or a major-minor two-suiter and 3♣ is natural; we
  follow Bridge Bum's "either minor", which is what the relay answers
  3♣ with clubs imply in both).
- GIB (BBO's robot, what the GIB corpus in Practice-Bidding-Scenarios
  plays), queried through https://netbridge.dk/gib.html (its bidding
  database, sequences `1N-*`, `1N-2C-P-*`, `1N-2C-P-2D-P-*`, `1N-2D-P-*`,
  `1N-2H-P-*`, `1N-2N-P-*`, `1N-D-P-*`, `1N-P-P-*`): X "Penalty double --
  16+ HCP"; 2♣ "Cappelletti - single suited -- 14- HCP; 10+ total
  points"; 2♦ "majors -- 5+ ♥; 5+ ♠"; 2♥/2♠ "hearts/spades and a minor";
  2NT "minors -- 5+ ♣ 5+ ♦"; the same in the balancing seat; advancer
  passes 2♣ with long clubs, 2♦ relay, 2♥/2♠ six-card suits, 2NT over
  2♥/2♠ "Show me your other suit".

## Gaps and open questions

- **Balancing 2♣** lost on both yardsticks (vs-1nt.notes.md). BBA wants
  11+ HCP there; we use the direct 12-with-length. Worth a probe-driven
  threshold of its own.
- The advancer's invitational 2NT over 2♣ (GIB: 11-14 balanced) is not
  written: a strong advancer relays and then raises.
- Nothing after the opponents compete over our call (the answers need
  RHO to pass or double); the fallback is pass.
- Cappelletti over a weak notrump: played the same (the card's field is
  "vs strong NT"; BBA and GIB do not change it either).
