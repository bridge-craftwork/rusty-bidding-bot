# Defences to their 1NT (`vs-1nt.bid`): notes

The shared part of the defences to the opponents' 1NT opening: the
three-level preempts, the pass, and the answers every defence uses
(partner has both majors, a major and a minor, both minors, a suit and
another, one long suit; the penalty double). Cases: `vs-1nt.test`. The
defences themselves: `cappelletti.bid` (Cappelletti and Modified
Cappelletti), `dont.bid`, `meckwell.bid`, `multi-landy.bid` (Landy and
Multi-Landy); the natural defence stays in `overcalls.bid`. How our side
reads their calls: `nt-interference.notes.md`.

## Guidance (Rick, 2026-09-28)

"Implement the common defences against the opponents' 1NT opening, so
our bot is compatible with the cards people play: Cappelletti (a.k.a.
Hamilton), Modified Cappelletti, Meckwell, and DONT. We can do
comparisons to see which appears most effective, but since people play
all of these, we'd like our bot to be compatible with their cards."
Each partnership's card chooses; a card that sets nothing keeps the
natural defence of before.

## The card

`competitive.vs_1nt_strong.system` is the field, the path Bridge-
Classroom's editor writes (`VsNtDefense.vue`: a system name, picked from
`ntDefenses.js` or typed, and one text cell per call). The older
`competitive.defense_vs_strong_nt.convention` loads into it as an alias.
Options: `cappelletti`, `modified_cappelletti`, `landy`, `multi_landy`,
`dont`, `meckwell`, `other` (natural). Since the editor stores display
names, the field has `value_aliases` (a new field attribute, 2026-09-28):
an enum value matches ignoring case, spaces and hyphens ("Meckwell",
"Multi-Landy", "DONT", "Modified Cappelletti"), and "Hamilton" /
"Pottage" load as `cappelletti`, "Penalty" / "Natural" / "Standard" as
`other`. The per-call cells (`.dbl`, `.2c`, `.2d`, `.2h`, `.2s`, `.2nt`,
`.other`) and the whole `vs_1nt_weak` block are text fields: they load
and round-trip, the rules do not read them. `competitive.dont.play`
(the editor's catalog switch for DONT) plays DONT when the system is
unset or `other`.

`.bbsa`: `Cappelletti`, `Landy` and `Multi-Landy` were mapped already
and now write the new path. BBA has no key for DONT, Meckwell or
Modified Cappelletti. No new key was mapped and none was guessed.

## The shared answers

- **Both majors** (Cappelletti 2♦, Landy/Multi-Landy 2♣, DONT 2♥,
  Meckwell X then 2♥): the longer major, hearts when equal (BBA and GIB
  both); a jump with four and 10-12 support points; game from 13; a
  six-card minor with at most two of either major plays there.
  BBA (`probes/vs-1N-capp-2D-adv.toml`): 2♥ 183, 2♠ 191, 3♥/3♠ invites
  90 (11-14 total points), 4♥/4♠ 87, pass with long diamonds 17.
- **A major and a minor** (Cappelletti and Multi-Landy 2♥/2♠, Modified
  2♣ then 2♥/2♠): pass with two or more; 3M invites with three (10-12
  support points), 4M from 13; 2NT asks for the minor with a singleton or
  void; a six-card suit of one's own is to play. BBA
  (`-2H-adv`, `-2S-adv`): pass 328/342, 3M 88/94 (8-14), 4M 99/87
  (from 11), 2NT "ask for minor" 46/55. It raises more than we do.
  The overcaller answers 2NT with his minor (`-2H-2N`: all 300).
- **Both minors** (2NT): the longer minor, diamonds when equal (BBA,
  `-2N-adv`: 98 of 98), 5♣/5♦ with a fit and 14.
- **A suit and another** (DONT 2♣/2♦, Meckwell 2♣/2♦): pass with three,
  else the next step asks, and partner passes or bids his other suit.
- **One long suit shown**: raise a major with two, 11-12 invites, 13+
  game.
- **The penalty double** (Cappelletti): sit with 5+ HCP or no five-card
  suit; run to a five-card suit with 0-4. BBA (`-X-adv`) runs more
  (0-7 total points, to any four-card suit) and passes from about 8;
  running with four-card suits was not tried.
- Every answer needs RHO to pass or double (the relays also allow a
  redouble); when they bid, the fallback is a pass.
- **Preempts**: with a conventional defence a seven-card suit jumps to
  the three level only below the one-suited call (4-8 HCP), as BBA on
  21GF-DEFAULT (6-8 HCP; 2♣ from 9 with seven). The natural defence keeps
  its 4-10 (overcalls.bid).

## Measured (2026-09-28, full corpus, 170,633 boards)

Baseline (this branch before the change): vs BBA **-114,604** IMPs.
After: **-116,953** (-2,349 by par distance), with the cards as they are
(21GF-DEFAULT, NS in 208 scenarios, and 21GF-GIB, EW in 316, both play
Cappelletti; seven cards play Multi-Landy). By the competitive yardstick
(`probes/tools/sideimps.py base.json final.json 3`): **5,106 boards
changed, +6,027 IMPs to the side that changed its call** (the direct
seat +5,454 on 4,309 boards, balancing +579 on 793), par distance -2,349.
The two yardsticks disagree, as they did for the natural overcalls
(overcalls.notes.md, "For Rick: which yardstick"): the defending side
gains, and par distance counts its gains as losses. No-rule positions
in live auctions 2,835 → 2,779; artificial contracts 133 → 133.

By call (base → final, side IMPs / par distance): X +2,186 / -1,286
(1,367 boards); 2♣ +396 / +271 (955); 2♦ +690 / -202 (601); 2♠ +739 /
-92; 2♥ +399 / -287; 2NT +326 / +30; balancing 2♣ -137 / +99 (261),
balancing X +627 / -407. Only the balancing 2♣ loses on the side
yardstick (cappelletti.notes.md, Gaps). By side: EW (mostly 21GF-GIB)
+4,137 / -2,026 over 3,006 boards, NS +1,890 / -323 over 2,100.

Replay agreement with BBA: 76.6% → 76.9% of calls (NS 63.6 → 63.9, EW
89.5 → 89.8); identical auctions 16.4% → 16.9%.

## Comparison of the defences (for Rick: information, not a choice)

Each defence forced on for both sides (`compare --set
competitive.vs_1nt_strong.system=X`), against the natural defence forced
the same way (`=other`). "Side" is sideimps from the natural run to the
defence's run, from the side that made the first differing call; "par"
the change in distance from par over the same boards (positive: closer).
"vs BBA" is the full-corpus total of each run.

| Defence | vs BBA | boards changed | side IMPs | par distance | EW side / par | NS side / par |
|---|---|---|---|---|---|---|
| natural (`other`) | -115,733 | | | | | |
| Cappelletti | -117,155 | 5,141 | **+2,963** | -1,422 | +1,951 / -1,338 | +1,012 / -84 |
| Meckwell | -117,321 | 4,058 | +2,676 | -1,588 | +2,442 / -1,982 | +234 / +394 |
| Modified Cappelletti | -116,607 | 5,014 | +2,401 | -874 | +1,362 / -925 | +1,039 / +51 |
| Multi-Landy | -116,689 | 4,188 | +1,694 | -956 | +1,268 / -1,084 | +426 / +128 |
| DONT | -117,082 | 4,758 | +726 | -1,349 | +798 / -1,497 | -72 / +148 |
| Landy | -115,591 | 1,317 | -58 | +142 | -94 / +18 | +36 / +124 |

Reading it:

- Every defence except Landy gains for the side that uses it, by the
  side yardstick; every one loses by par distance, which counts a
  successful intervention against it. Landy is close to natural (only
  2♣ differs) and is the one par favours.
- Most of Cappelletti's, Modified Cappelletti's and Meckwell's gain is
  the penalty double (Cappelletti: X over a natural pass +1,464 on 908
  boards, balancing X +654) and, for Meckwell, its double (+775 over a
  pass, +707 where natural bid 2♠). DONT has no penalty double and gains
  least; its passes where natural overcalled 2♠/2♦/2♣ lose (-306, -193,
  -124): our DONT has a 15 HCP ceiling, so a 16-17 overcall now passes.
- Cappelletti's one-suited 2♣ where natural bid 2♦ or 2♠ loses (-258
  over 379 boards, -240 over 218): showing the suit a round later costs.
  Modified Cappelletti's 2♣ loses the same way over 2♥ (-125).
- These boards were dealt for other scenarios, with our engine on both
  sides; the corpus has no BBA reference for DONT, Meckwell or Modified
  Cappelletti, so only the two double-dummy yardsticks judge them.

Runs: `/Volumes/X10 Pro/rusty-bidding-bot/compare/ntd/` (`base.json`,
`final.json`, `f-<defence>.json`).

## Sources

- Bridge Bum: "Cappelletti (Hamilton)", https://www.bridgebum.com/cappelletti.php
  (with its Modified Cappelletti section); "DONT",
  https://www.bridgebum.com/dont.php; "Meckwell Defense to 1NT",
  https://www.bridgebum.com/meckwell_defense_to_1nt.php; "Multi-Landy
  (Woolsey)", https://www.bridgebum.com/multi_landy.php.
- Wikipedia: "Cappelletti convention",
  https://en.wikipedia.org/wiki/Cappelletti_convention; "DONT",
  https://en.wikipedia.org/wiki/DONT; "Meckwell convention",
  https://en.wikipedia.org/wiki/Meckwell_convention.
- ACBL Unit 390 (Simon): "Modified Cappelletti",
  https://www.acblunit390.org/Simon/modcapp.htm; "Meckwell defense to
  1NT", https://www.acblunit390.org/Simon/meckwellnt.htm.
- GIB (what the GIB corpus plays): its bidding database, as browsed at
  https://netbridge.dk/gib.html (sequences `1N-*`, `1N-P-P-*`,
  `1N-D-P-*`, `1N-2C-P-*`, `1N-2C-P-2D-P-*`, `1N-2D-P-*`, `1N-2H-P-*`,
  `1N-2N-P-*`): Cappelletti, X penalty 16+, the same in the balancing
  seat.
- Bridge-Classroom `src/utils/ntDefenses.js` and
  `src/components/conventionCard/VsNtDefense.vue` (the card paths and
  the editor's meanings).
- BBA, as a black box: the probes listed under each module.

## Probes (all `our = []`, 21GF-DEFAULT unless named)

`probes/vs-1N-capp.toml` (direct, 1,500 hands), `vs-1N-capp-bal.toml`
(balancing, 800), `vs-1N-capp-2C-adv`, `-2D-adv`, `-2H-adv`, `-2S-adv`,
`-2N-adv`, `-X-adv` (advancer, 600 each), `-2C-2D` and `-2H-2N` (the
overcaller's answers), `vs-1N-mlandy.toml` (21GF-Multi, 1,000).

## Gaps and open questions

- **Which yardstick** decides the thresholds (the penalty double at 15,
  the minimum for the two-suiters)? The card decides the defence; the
  thresholds follow BBA and the sources where they agree.
- **Their weak notrump**: `vs_1nt_weak.system` loads but is not read;
  the strong-notrump scheme is played against every range. Reading it
  needs the opener's range in the context (their 1NT's `hcp.max`).
- Competitive continuations when the opener's side bids over our call
  are not written (the fallback is pass).
- Systems (Stayman, transfers) by the advancer of a 1NT *overcall* are
  unrelated and unchanged.
- Suction and Hello (in the editor's list) are not options: a card
  naming them loads with the field reported invalid and plays natural.
