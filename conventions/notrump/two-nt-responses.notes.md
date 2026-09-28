# Responses to a 2NT opening (`two-nt-responses.bid`): notes

Cases: `two-nt-responses.test`.

## Guidance

The 1NT structure one level up, which is what BBA plays on every card in
the corpus: 3♣ Stayman, 3♦ and 3♥ transfers, 3NT to play, 4NT
quantitative, and pass with nothing.

Only the asking calls and opener's answers are written here. Everything
after them was already written against the question — `when answered
majors` in `stayman.bid`, `when answered transfer(M)` in
`jacoby-transfers.bid` — and the strength bands read opener's 20-21, so
responder invites and bids game at the right level without a single new
rule. The calls that would be illegal a level up (the invitational 2NT,
a 3M rebid under the completed transfer) drop out on their own.

## Evidence

`bba-cli --all-meanings` over the Basic_* scenarios, responder's call
after `2N P`:

| call | BBA's meaning | n |
|---|---|---|
| 3♣ | "Stayman" | 37 |
| 3♥ | "transfer" (spades) | 23 |
| 3NT | "calculated bid", 4-10 total points | 16 |
| Pass | 0-4 total points | 16 |
| 3♦ | "transfer" (hearts) | 14 |
| 6NT | 13+ | 4 |
| 4NT | "Quantitative 4NT", 11-12 | 3 |

Before this module responder passed a 2NT opening whatever he held:
4,345 boards across the corpus, 123 of them in Basic_*.

| scenario | calls | auctions | contracts |
|---|---|---|---|
| 2N | 68.8% → **80.2%** | 3.2% → 32.8% | 5.4% → 46.8% |
| 2N_and_Balanced | 69.2% → **81.2%** | 2.0% → 32.2% | 4.0% → 58.4% |
| 2N_and_1_Minor | 69.0% → **75.5%** | 1.2% → 22.4% | 3.2% → 27.2% |
| Basic_* (nine) | 83.1% → **83.6%** | 34.4% → 35.3% | 41.0% → 42.1% |

1N and Basic_NT are unchanged, which is the check that the shared
continuations still behave at the one level.

One ranking fix came with it: in `stayman.bid`, "game, no fit found"
(3NT) and "invitational, no fit found" (2NT) now have `priority -1`, so a
known 4-4 major fit is played in the major. Opposite a 20-21 opening the
game band is wide enough that 3NT was outranking 4♥ on descriptiveness.

## Gaps and open questions

- **Minor-suit transfers over 2NT** (`notrump.two_nt.minor_transfers`,
  on for both 21GF cards) and minor-suit Stayman: not written, so a
  minor one-suiter bids 3NT.
- **Puppet Stayman** (`two_nt.puppet`): played since 2026-09-28, see
  the section below. Puppet over 1NT (`stayman.puppet_1nt`) is not.
- The 3♠ relay, `two_nt.three_s`, and the four-level transfers.
- Texas over 2NT, and slam bidding beyond the quantitative 4NT.

## BBA treatment (2026-09-24)

`general.style = bba` plays a probed model of BBA's responses to 2NT.
Probes: `rbb probe`, Basic-Bridge both sides, dealer S, North after a
forced `2NT Pass`, MP and IMP at love all and IMP all vulnerable (and MP
all vulnerable on the first set). **Scoring and vulnerability changed
nothing here.**

| North | HCP (+tens) | BBA |
|---|---|---|
| 8432.J32.63.8632 | 1 | P |
| K432.7632.96.972 | 3 | P |
| T432.KT32.96.972 | 3+2 | P |
| K432.Q632.96.972 | 5 | 3♣, then 3NT over 3♦ |
| Q432.Q632.96.972 | 4 | P |
| Q432.K632.96.972, K432.K632.96.972 | 5, 6 | 3♣ |
| K32.Q632.962.972 (4-3-3-3) | 5 | 3♣ |
| 5432.Q72.962.972, 5432.K72.962.972 | 2, 3 | P |
| 432.J72.K962.972, T32.J72.KT62.972 | 4, 4+2 | P |
| 432.Q72.K962.972 (and +1, +2 tens) | 5 | 3NT |
| 432.K72.A962.K72 | 10 | 3NT |
| T32.KT2.AT62.K72 | 10+3 | 3NT |
| 432.K72.A962.KJ2, 432.K72.A962.A72 | 11 | 4NT |
| 432.KT2.AT62.A72 | 11+2 | 6NT |
| 432.K72.A962.AJ2 | 12 | 6NT |
| Q32.K72.A962.A72 | 13 | 6NT |
| A32.K72.A962.AQ2 | 17 | 7NT |

In HCP: **pass with 4 or less, a four-card major or not**; from 5
Stayman (4-3-3-3 included) or 3NT; 4NT with 11; 6NT with 12, or 11 and
two tens. Corpus (all three 15-17 cards, 2NT the same structure): with a
major 3–4 HCP mostly pass (the rest are 21GF sequences such as 3♠),
5–6 HCP 3♣ 100% (230 boards); without one 5 HCP 3NT 100%. Basic_What_To_Open: with 5-5 in the majors BBA transfers to spades
(3♥, three boards), and with a five-card major and slam values it
transfers first (`KJT96.A42.K63.T8`, and Basic_Minor 228: 3♥ then 6NT).

`bba-cli --all-meanings` already calls the 4NT "Quantitative 4NT, 11-12":
opposite 20–21 that is 32 with the minimum, where over 1NT BBA invites
with 16 (33 with the maximum).

**What the rules model** (`when style is bba`; the default rules `style
is not bba`): 3♣ `hcp>=25-partner.hcp.min` with a four-card major, any
shape; 3♥ with 5-5; 3NT from 5 HCP; 4NT `hcp>=32-partner.hcp.max,
points<=31-partner.hcp.min`; 6NT `points>=32-partner.hcp.min`; both with
no five-card major; pass with 4 HCP or less. Opener's answer to 4NT over
2NT keeps the default (32 combined): BBA accepted with 21 (two probes;
20 not tried).

**Measured** (whole corpus, bba style, calls agreeing at `2NT P`): 65.7%
→ 73.9% (4,422 decisions; Basic-Bridge 79.7% → 95.7%). The rest is on
the 21GF cards: Gerber 4♣, minor-suit transfers, Puppet (gaps above).

**Tried as our default** (scratch copy, 3♣ Stayman and the pass keyed to
5 HCP): corpus par −66 IMPs, Basic_* 0. Not proposed.

## Responder after the completed transfer (2026-09-25)

The generic transfer rules measure strength bands against opener's
20-21, and left weak or unbalanced hands with no call after 2NT-3D-3H
and 2NT-3H-3S: 408 corpus boards. BBA passes with 0-3 HCP. From 4
(25 combined) it bids 3NT with five of the major, balanced or not, and
4M with six. Those are now fallbacks under the generic rules.
Corpus: -166,629 -> -165,434 (+1,195 IMPs).

## Weak hands with no call (2026-09-25)

Two gaps in the default, both seen in Basic_What_To_Open N/S auctions:

- **A 4-count.** Opposite 20-21 the signoff band ends at 3 total points
  and game starts at 5, so 4 is the invitational band, which has no
  call over 2NT. The pass now covers `strength<=invite`. BBA passes with
  4 too (the bba rule's "4 HCP or less").
- **Stayman with nothing.** The default 3♣ had no floor, so 0-3 counts
  with a four-card major bid it and then had no rule after the answer
  (2NT-3♣-3♦ and 2NT-3♣-3♥, 5 boards). It now needs game values, as
  BBA's does (from 5 HCP).

No-rule positions in Basic_* N/S fell by 10; par effect on Basic_* +1.

## Slam in a minor over 2NT (2026-09-27)

483 boards stopped at 2NT-3NT where BBA bid slam (median 31 HCP between
the hands, half of the slams in a minor). Now 4♣/4♦ is a slam try with
six cards or five unbalanced and slam-invite values; opener bids six
with three-card support, else 4NT; responder then 6NT with slam values.
Full corpus +1,730 by par distance, +3,123 by side (448 boards).

**With Gerber on the card (2026-09-28)** the 4♣ club try is off: 4♣
asks for aces (`slam/gerber.bid`), and the club hands (six clubs, or
five unbalanced, slam-invite values) ask and then bid 6NT with three
aces between the hands (6♣ at IMPs), or sign off in 4NT. The 4♦ diamond
try and its follow-ups stay. This is what BBA does on the 21GF cards
(2NT–4♣ Gerber, then 6♣/6♦ or 6NT: `probes/gerber-resp-2N.toml`).
Measured with the rest of Gerber: `gerber.notes.md`. Texas over 2NT
(4♦/4♥, `two_nt.transfers_4level`, on Rick's card) is still not built;
when it is, the 4♦ diamond try will have to give way to it the same way.

## Puppet Stayman over 2NT (2026-09-28)

Rick: "we want Puppet over 2NT working."

**The switch.** The `.bbsa` key "2N-3C Puppet Stayman" used to write
`notrump.stayman.puppet` while this module read `notrump.two_nt.puppet`
(the path Bridge-Classroom's card editor and PDFs use), so the switch
never reached the rules. There is now one field, `two_nt.puppet`; the
key maps to it and `notrump.stayman.puppet` (the catalog's other path)
is an alias. Merging the fields without Puppet rules was measured first
at −168 par / −354 side IMPs: ordinary Stayman switched off and nothing
replaced it. The merge now comes with the rules.

**What is played** (`when puppet`):

| call | shows |
|---|---|
| 3♣ | game values, a three- or four-card major, no five (5-4 in the majors transfers to the five) |
| opener 3♦ / 3♥ / 3♠ / 3NT | a four-card major and no five / five hearts / five spades / neither |
| after 3♦: 3♥ / 3♠ | four spades, not four hearts / four hearts, not four spades: the major he does *not* hold, so opener declares |
| after 3♦: 4♦ / 4♣ | both majors, no slam interest / slam interest (slam-invite values); opener chooses, spades with both (BBA) |
| after 3♦: 3NT / 6NT | no four-card major; 6NT from 32 total points on opener's minimum |
| opener after 3♥/3♠/4♣/4♦ | 4M with the fit (`sets trump`), else 3NT |
| responder after 4M | 6M with 33 support points on opener's minimum, else pass |
| after opener's 3♥/3♠ | 4M with three, 6M with 33 support points; without: 3NT, 6NT from 32 |
| after 3NT (no major, or no fit) | 6NT from 32, else pass; the minor-suit slam tries of slam/over-3nt.bid behind |

The no-fit counts are those after Stayman (stayman.bid). The ask sets
`ask=puppet`, the second round `ask=puppet_fit(x)` / `ask=puppet_both`,
so only the 3♣ itself names an auction. It is on after 2♣–2♦–2NT too,
because BBA plays it there (below); the strength bands then read the
22-23. The bba style keeps the entry at BBA's count (`hcp>=25-
partner.hcp.min`, 5 HCP opposite 20-21); the rest is shared.

### BBA's treatment (probes, 21GF-MSTandMSS both sides, dealer S, None/MP)

Specs in `probes/puppet-*.toml`:

- `puppet-2N-resp` (500 random responder hands after `2NT P`): 3♣
  with any three- or four-card major and 5+ HCP (3-1, 3-2, 3-3 and
  4-x alike, slam hands too); a five-card major transfers, 4-5 and 5-4
  included; two doubletons or shorter majors 3NT (or 4♣ Gerber with
  slam values); 0-4 pass.
- `puppet-2N-opener` (300 opener hands after `2NT P 3C P`): 3♦ with
  any four-card major, 3♥/3♠ with five, 3NT with neither. No exceptions.
- `puppet-2N-3D-resp` (400 hands after `... 3D P`): 3♥ = four spades,
  3♠ = four hearts at every strength; both majors 4♦ with 6-8 HCP,
  4♣ from 9; no four-card major 3NT to 10, 11-12 split 4NT/6NT, 12+
  6NT (sometimes 6 of a minor).
- `puppet-2N-3D-3H-opener`, `-3S-opener`: over 3♥ (four spades) 4♠
  with 20 HCP but **3♠ with 21** (a maximum, below game, then cue
  bids); over 3♠ (four hearts) 4♥ always; without the fit 3NT.
- `puppet-2N-3D-4D-opener`, `-4C-opener`: with both majors BBA picks
  **spades**; over 4♣ it often bids 4NT keycard itself (more with
  hearts than spades).
- `puppet-2N-3H-resp`, `-3S-resp` (250 each, opener with five): raise
  to game with three, 6M with slam values, 3NT without a fit; with slam
  values and no fit 4♣/4♦ slam tries or 6NT.
- `puppet-2C-2N-resp` (120 hands after `2C P 2D P 2NT P`): the same
  structure after 2♣–2♦–2NT: 3♣ with a three-card major from 3 HCP, 3♦ /
  3♥ transfers, pass with 0-2.

The corpus (Puppet_Stayman_2N, 290 boards through 3♦, 176 through 3NT)
agrees: 3♥/3♠ for the major not held, 4♦/4♣ with both, 3NT/4NT/6NT/7NT
with 3-3, Gerber (4♣) or a quantitative 4NT after 3NT.

**Differences we keep (not modelled):** BBA's 3♠ over 3♥ with a
maximum, its keycard asks after the fit (ours: 6M directly on a count;
it reaches the grand slams we miss, ~20 boards at −11 to −13), 4♣ from
9 HCP with both majors (ours: slam-invite values, 12), Gerber after
3NT, and minor-suit slam tries at the six level.

### Measured (par as the yardstick, 170,633 boards)

| | before | after |
|---|---|---|
| whole corpus, vs BBA | −114,604 | **−114,554** (+50) |
| Puppet_Stayman_2N, vs BBA | −364 | **−314** |
| Puppet_Stayman_2N, calls agreeing | 70.1% | **87.4%** |
| Puppet_Stayman_2N, same contract | 321 | **335** |

`sideimps.py`: +43 IMPs to the side that changed its call (181 boards
changed), all in Puppet_Stayman_2N; par distance +50. Both yardsticks
agree. The other scenarios on 21GF-MSTandMSS (MST_or_MSS,
Minor_Game_Or_Slam, Minor_Suit_Stayman, Minor_Suit_Transfer,
We_Overcall_NT_then_MSS/MST) have no 2NT opening and are unchanged; no
corpus board reaches 2♣–2♦–2NT on this card. With `general.style=bba`
the scenario is −388 (87.4% of calls).

A first version (−257 on the scenario) used the over-3NT counts after
a 3NT answer (6NT from 34 HCP, quantitative 4NT from 32 HCP, which
opener then declined because responder had shown HCP, not points) and
lost the 6NT that plain Stayman reached; the no-fit counts after
Stayman fixed that.

### For Rick

- **Puppet over 1NT** (`notrump.stayman.puppet_1nt`, "1N-3C Puppet
  Stayman", set only on 21GF-Puppet: scenario Puppet_Stayman_1N, −341
  vs BBA): no rule reads it; 1NT–3♣ is whatever the card's other 3♣
  treatment is. Not implemented here, as asked; the 2NT rules above
  would carry over one level down (the ask is written against state).
- The 3♠-with-a-maximum and opener's own keycard ask over 4♣ are BBA's
  refinements; worth a try if the grand slams matter.

### Sources

- Bridgebum, "Puppet Stayman": https://www.bridgebum.com/puppet_stayman.php
  (3♣ with one or two four-card majors, or three-card support; 3♦/3♥/
  3♠/3NT; after 3♦ bid the major you do not hold; 4♦ both without, 4♣
  both with slam interest). The scenario's `.btn` cites it too.
- Wikipedia, "Puppet Stayman": https://en.wikipedia.org/wiki/Puppet_Stayman
  (the same answers and rebids; the strong hand declares).
- ACBL convention card, 2NT section: the Puppet Stayman box that
  Bridge-Classroom's "2NT options: Puppet" (`two_nt.puppet`) mirrors
  (not re-read online for this change).
- GIB: https://netbridge.dk/gib.html (GIB's bidding definitions); the
  21GF-GIB card has "2N-3C Puppet Stayman = 0", so GIB plays plain
  Stayman over 2NT in the corpus. The page renders its definitions with
  script and could not be read here.
- Bridge Guys' page (bridgeguys.com/Conventions/puppet_stayman.html) was
  unreachable on 2026-09-28.
