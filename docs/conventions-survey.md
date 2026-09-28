# Conventions survey: the PBS corpus, BBA and this engine

GitHub issue #4. Survey date 2026-09-28, engine at `fc9dd81`. Rick's
decisions of 2026-09-28 are recorded in section 4 (the re-ranked priority
list) and in "Open questions for Rick" (the answers). The figures are
from the survey run and have not been re-measured since.

This survey cross-checks four sources:

- **BBA's conventions:** the 18 `.bbsa` cards in Practice-Bidding-Scenarios (PBS) `bbsa/`.
- **The scenarios that use them:** 350 `.btn` files in `btn/`, with their `convention-card-ns/ew` and `bba-works` metadata. Each also has a BBA corpus of about 500 boards in `bba/`.
- **This engine:** `fields.toml`, `bbsa-map.toml`, the `.bid` modules, `rbb card coverage` and a full `rbb compare` run.
- **The Bridge-Classroom taxonomy:** `conventionCatalog.js` and `bakerBridgeTaxonomy.js`.

All tables come from `probes/tools/conventions_survey.py`; see "How this was made" at the end.

## Headline numbers

| | |
|---|---|
| BBA convention keys (all 18 cards) | **173**, not counting padding |
| Keys switched on in at least one card that a PBS scenario uses | **117** |
| Keys mapped to a card field (`bbsa-map.toml`) | **145 of 173** (84%). The other 28 are kept verbatim as passthrough; 16 of those are on in a used card |
| Keys that some rule reads | **40 of 173**, or **32 of the 117 in use** (27%). Weighted by boards played with the key on: 36% |
| Card fields (`fields.toml`) / read by rules | **321 / 63** |
| Rule modules | **34** (6,535 lines of `.bid`, 1,833 lines of `.test`) |
| `rbb card coverage` per card | 31% (21GF-PolishTwoSuiters) to 60% (Basic-Bridge); 48% for 21GF-DEFAULT |
| PBS scenarios | **350** (343 with a BBA corpus; 170,633 boards) |
| `bba-works: false` | **24** scenarios: 17 have a BBA corpus (8,500 boards), 7 have none |
| Calls agreeing with BBA (all boards) | **76.6%** (NS 63.6%, EW 89.5%); same final contract 35.4% |
| Bridge-Classroom catalog entries | **67**; the rules read 17 of their fields (18 counting Stayman, which the catalog keys on a different field) |

Two cards carry most of the corpus. **21GF-DEFAULT** is NS in 208 scenarios (101,477 boards). **21GF-GIB** is EW in 316 (154,505 boards). The conventions on 21GF-GIB therefore shape the opponents' bidding in almost every scenario: Cappelletti, Michaels, Unusual 2NT, Jordan 2NT, splinters, DOPI and so on (Appendix A).

## 1. What BBA offers

BBA stores a card as a system preset (`System type`: 2/1, SAYC, Polish Club, Precision, Acol) plus about 170 on/off keys. The table groups the keys the way `bbsa-map.toml` does. "On in a used card" counts keys switched on in at least one card that a PBS scenario uses. Appendix A lists every key, how many cards turn it on, how many scenarios use such a card, and its state in the engine.

| Group | Keys | On in a used card | On in most cards (15+ of 18) | On in fewer cards (how many, or which) |
|---|---:|---:|---|---|
| Notrump openings | 9 | 6 | 1NT 15-17, 5-card major allowed | 13-15 (Precision), 14-16 (Precision-14-16), 4441 (SPECIALS2) |
| Responses to 1NT / 2NT | 28 | 20 | Stayman (implied), Jacoby (implied), Texas, super-accept, Smolen, 2♠→♣ / 3♣→♦ transfers, minor transfers after 2NT, Lebensohl after 1NT, transfers over their 2♣ | Minor Suit Stayman (MSS, MSTandMSS), 1NT–2NT→♣ (MSTandMSS), Puppet over 1NT (Puppet) and over 2NT (MSTandMSS), 1NT–3♦/3M splinters (GIB, GIB-Bergen, Precision), Rubensohl (Precision), MSS after 2NT |
| General / jump shifts | 7 | 5 | strong jump shifts at the 2 level | inviting (8), weak at the 2 or 3 level (5), Soloway (SPECIALS2) |
| Major openings | 13 | 11 | Jacoby 2NT, splinters | forcing 1NT (10), reverse Drury (11), Drury (6), mixed raise (8), Bergen (5), reverse Bergen (3), 1M–3M blocking (4), Gazzilli (Gazzilli, MSTandMSS) |
| Minor openings | 5 | 3 | inverted minors, 1m may hold a 5-card major | Walsh (5) |
| Two-level openings | 15 | 12 | weak 2M | weak 2♦ (13), Multi, Flannery, Ogust, Benjamin, Polish two-suiters (one card each), reverse Flannery (SPECIALS, Precision-14-16), Precision 2♦, Kokish (2) |
| Preempts | 1 | 1 | | Namyats (7) |
| Slam | 17 | 13 | cue bids, DOPI, ROPI, Gerber, King ask by 5NT, quantitative 4NT, 4NT opening | RKCB 1430 (14), 0314 (GIB, GIB-Bergen, Precision), plain Blackwood (Basic-Bridge), Exclusion (7), 5NT pick-a-slam (7), Gerber only over NT (3) |
| Doubles | 3 | 3 | support double/redouble | responsive (14), maximal (3) |
| Competitive | 30 | 17 | Michaels, Unusual 1NT/2NT/4NT, Jordan 2NT, Lebensohl after a double, 1X-(Y)-2Z forcing, 1X-(1Y)-2Z weak | Cappelletti (10), Multi-Landy (7), BROMAD (6), fit-showing jumps (7), leaping Michaels (3), Rubensohl (Precision), direct jump cuebids (Precision) |
| Carding | 6 | 3 | Lavinthal | (outside bidding) |
| Other | 10 | 8 | NMF after a 2NT rebid | NMF (12), FSF game force (11), FSF one round (6), two-way game tries (7), two-way NMF (Precision), Checkback (2), Roudi (SPECIALS2) |
| Not mapped (passthrough) | 28 | 16 | `1NT opening NT style`, `Two suit takeout double`, `Mark on queen/king` | `Imposible 2S` (9), `Lebensohl after 1m` (15), `Shape Bergen structure` (12), `Strength Lawrence structure` (6), `Support 1NT` (6), `Crosswood 0314` (Precision), `Rubensohl after 1m` (Precision), `5431 after 1NT`, `Collante`, `Rodrigue`, `Natural 3N entering style`, `Jordan Truscott 2NT defence` (one card each) |

The 18 cards and how the corpus uses them:

| Card | NS in (scenarios / boards) | EW in | System |
|---|---:|---:|---|
| 21GF-DEFAULT | 208 / 101,477 | 5 / 2,472 | 2/1 |
| 21GF-GIB | 35 / 17,000 | 316 / 154,505 | 2/1 (GIB-like: 0314, Drury, Walsh, 1NT–3♦/3M splinters) |
| Precision | 29 / 14,500 | 2 / 500 | Precision, 1NT 13-15 |
| Basic-Bridge | 24 / 11,656 | 24 / 11,656 | SAYC, plain Blackwood |
| 21GF-SPECIALS2 | 14 / 7,000 | 1 / 500 | 2/1 + Soloway, Roudi, Rodrigue, Collante, Walsh, 4441 1NT |
| 21GF-SPECIALS | 13 / 6,000 | 1 / 500 | 2/1 + reverse Flannery, Kokish, Bergen, `5431 after 1NT` |
| 21GF-MSTandMSS | 7 / 3,500 | | SAYC preset + MSS, 1NT–2NT→♣, Puppet over 2NT |
| 21GF-NoInvertedMinor | 6 / 3,000 | | 2/1 without inverted minors |
| 21GF-MSS, -PolishTwoSuiters, -Puppet, -Multi | 2 each | (Multi: 1 EW) | 2/1 + the named convention |
| 21GF-Flannery, -GIB-Bergen, -Gazzilli, -Ogust, -WJS-MSS, Precision-14-16 | 1 each | | 2/1 (or Precision) + the named convention |

### Keys whose meaning is unclear

Following CLAUDE.md ("do not guess the meaning of an unmapped `.bbsa` key"), the 16 passthrough keys that are on in a used card stay in passthrough. The scenario names suggest meanings for some of them, but that is only a hint, not evidence. Rick's answers of 2026-09-28 are in the last column; every key without one is still **to be investigated**:

| Key | On in | Hint from the corpus (not a definition) | Rick, 2026-09-28 |
|---|---|---|---|
| `Imposible 2S` (sic) | 9 cards, incl. 21GF-DEFAULT | Scenario `Impossible_2S`. BBA's notes there show no alert with that name (forcing 1NT 364, limit raise 154) | **Named, with sources.** The impossible 2♠ in auctions like 1♥ 1NT 2♣ 2♠, showing a good club raise. Sources: Robert Todd's write-up, ["The Impossible Spade Bid"](https://static1.squarespace.com/static/5127d3d2e4b0b304f0b6db24/t/5c8e551f104c7b066eb43ac8/1552831776048/3+%28269%29+The+Impossilbe+Spade+Bid+.pdf), and the older GIB reference, <https://netbridge.dk/gib.html>, which may treat it differently. **In progress** (another piece of work) |
| `Lebensohl after 1m` | 15 cards | 227 boards carry the note "Lebensohl after 1m", in 30 scenarios | Rick does not know what it is: to be investigated |
| `Rubensohl after 1m` | Precision, Precision-14-16 | 90 boards carry the note | Rick does not know what it is: to be investigated |
| `5431 after 1NT` | 21GF-SPECIALS | Scenario `5431_After_NT`. 409 boards carry the note, and we first differ from BBA on 408 of them (-319 IMPs) | to be investigated |
| `Two suit takeout double` | 17 cards | 75 boards carry the note, in 36 scenarios | to be investigated |
| `Support 1NT` | 6 cards | 32 boards | to be investigated |
| `Collante`, `Rodrigue` | 21GF-SPECIALS2 | Scenarios `WB5_Collante` and `WB5_Rodrigue`; 108 and 352 boards | to be investigated |
| `Crosswood 0314` | Precision cards | 57 boards | to be investigated |
| `Shape Bergen structure`, `Strength Lawrence structure` | 12 and 6 cards; never both on in one card | No note. They look like two alternatives to one setting | to be investigated |
| `Natural 3N entering style` | Precision | No note | to be investigated |
| `1NT opening NT style` | all 18 | No note. `1NT opening natural` is off in all 18 | to be investigated |
| `Mark on queen`, `Mark on king` | 16 cards | No note; probably carding (they sit next to Lavinthal) | to be investigated |
| `Jordan Truscott 2NT defence` | Precision | 1 board | to be investigated |

## 2. The PBS scenarios: where BBA covers and where it does not

### By family (the release layout's sections)

"NS calls = BBA" is the share of NS's calls where our engine, replaying BBA's auction, makes BBA's call. "Contract = BBA" is the share of boards that reach the same final contract. "vs BBA" is our IMPs against BBA's, with double-dummy par as the yardstick; negative means BBA was closer. Appendix B gives the same figures for every scenario.

| section | scenarios | BBA works: no | boards | NS cards | NS calls = BBA | contract = BBA | vs BBA (IMPs) |
|---|---:|---:|---:|---|---:|---:|---:|
| We Compete in Their Auctions | 65 | 4 | 31500 | 21GF-DEFAULT (46), 21GF-GIB (9), 21GF-SPECIALS (4), 21GF-SPECIALS2 (2), 21GF-NoInvertedMinor (2), 21GF-MSTandMSS (2) | 69% | 27% | -14918 |
| Notrump Sequences | 46 | 1 | 23000 | 21GF-DEFAULT (30), 21GF-GIB (7), 21GF-MSTandMSS (4), 21GF-SPECIALS2 (2), Basic-Bridge (1), 21GF-MSS (1), 21GF-Puppet (1) | 67% | 48% | -14297 |
| They Compete in Our Auctions | 40 | 5 | 19500 | 21GF-DEFAULT (32), 21GF-GIB (5), 21GF-SPECIALS (2), 21GF-SPECIALS2 (1) | 68% | 25% | -10672 |
| Minor/Major Sequences | 34 | 1 | 16972 | 21GF-DEFAULT (29), 21GF-GIB (2), 21GF-SPECIALS2 (1), 21GF-SPECIALS (1), 21GF-WJS-MSS (1) | 66% | 34% | -12379 |
| Preempts | 21 | 2 | 10000 | 21GF-DEFAULT (16), 21GF-Multi (2), 21GF-SPECIALS (1), 21GF-PolishTwoSuiters (1), 21GF-Ogust (1) | 67% | 28% | -12657 |
| Beyond BBO Robots | 20 | 1 | 10000 | 21GF-DEFAULT (12), 21GF-SPECIALS (4), 21GF-SPECIALS2 (2), 21GF-Gazzilli (1), 21GF-MSS (1) | 65% | 34% | -6927 |
| Game Forcing Sequences | 20 | 3 | 9500 | 21GF-DEFAULT (12), 21GF-SPECIALS2 (5), 21GF-GIB (2), 21GF-PolishTwoSuiters (1) | 52% | 36% | -16133 |
| Major Suit Sequences | 18 | 0 | 9000 | 21GF-DEFAULT (12), 21GF-GIB (3), 21GF-GIB-Bergen (1), 21GF-SPECIALS2 (1), 21GF-Flannery (1) | 60% | 43% | -5674 |
| Strong Club | 18 | 5 | 9000 | Precision (17), Precision-14-16 (1) | 55% | 31% | -4682 |
| Strong Club 16+ | 12 | 0 | 6000 | Precision | 38% | 21% | -7406 |
| Beginners Bidding | 11 | 0 | 5500 | Basic-Bridge (10), 21GF-GIB (1) | 79% | 48% | -1455 |
| Minor Suit Sequences | 11 | 1 | 5000 | 21GF-GIB (6), 21GF-DEFAULT (4), 21GF-MSTandMSS (1) | 69% | 34% | -4745 |
| Gavin's Major Suit Response Structure | 8 | 0 | 4000 | 21GF-DEFAULT | 70% | 45% | -1405 |
| Finesses | 5 | 0 | 2500 | Basic-Bridge | 75% | 63% | -1402 |
| (not in layout) | 7 | 1 | 2161 | 21GF-DEFAULT (3), Basic-Bridge (2), 21GF-SPECIALS (1), 21GF-Puppet (1) | 63% | 44% | -182 |
| Josh Donn's preferred Minor Suit Response Structure | 4 | 0 | 2000 | 21GF-NoInvertedMinor | 65% | 38% | -201 |
| Suit Contract Play | 3 | 0 | 1500 | Basic-Bridge | 77% | 69% | 181 |
| Gavin's Transfer After 2N Rebid | 3 | 0 | 1500 | 21GF-DEFAULT | 70% | 50% | -123 |
| Notrump Play | 3 | 0 | 1500 | Basic-Bridge | 88% | 81% | 57 |
| Bidding with BBO Robots | 1 | 0 | 500 | 21GF-DEFAULT | 68% | 25% | -432 |

The families that agree least with BBA:

- **Strong Club 16+ (38%)** and **Strong Club (55%)**: no Precision rules.
- **Game Forcing Sequences (52%)**: the five Soloway scenarios (28-46%), Benjamin_2D (36%) and Serious (45%).
- **Major Suit Sequences (60%)**: Flannery (28%), Impossible_2S (44%), Exclusion_After_1M (46%), Grand_Slam_Force (49%) and Drury (53%).

The low-agreement scenarios in the competitive sections have mostly two-suited overcalls behind them: Michaels, Unusual 2NT, and their Opps_ versions.

### The 24 `bba-works: false` scenarios

The chat text in each `.btn` names the convention, so each gap below is named after it. "Field" says whether our card can already express it.

| Scenario | BBA corpus | Proposed name for the missing convention | Evidence (btn chat / notes) | Field in our card | Classroom catalog |
|---|---|---|---|---|---|
| 1M-3N_Picture_Bid | 500 | **3NT picture response to 1M** (12-15, 2 cards in opener's major, 4-4 minors, 3 in the other major) | chat; BBA treats 1M–3NT otherwise (its notes: forcing 1NT, cue bids) | none (`three_nt_raise` is the 4-3-3-3 raise, a different call) | `major_openings.art_raises_other` (free text) |
| 1m-2x | none | **Todd minor-suit responses** (2♠ constructive raise, 2♥ 10-12 balanced, 2NT mini-maxi GF, 3NT 15-17) | chat, "Robert Todd" | none | none |
| Bergen_Thrump_X_after_Preempt | none | **Thrump double** (Bergen) over their three-level preempt | chat, bridgebum link | none | none |
| CRASH | none | **CRASH** defence to a strong 1♣ | chat | none | none |
| DONT | 500 (BBA played Cappelletti: 379 notes) | **DONT** vs 1NT (in progress, 2026-09-28) | chat | yes: `competitive.defense_vs_strong_nt = dont`, `competitive.dont.play` | `dont` (skill `competitive_bidding/dont`) |
| Gerber_By_Opener | 500 | **Gerber by opener** after 1m–2NT or 2♣…2NT | chat; BBA's notes show no Gerber here | partly: `slam.gerber.over` (text), `gerber.over_nt_seq` | `gerber` |
| Good_Bad_2N | 500 | **Good-bad 2NT** | chat | none | none |
| Lars_Multi_2D | none | **Multi 2♦ with a 20-21 NT option** ("Lars Multi") | chat | none (`two_diamonds.multi` is plain Multi) | `multi_2d` (a variant of it) |
| Mini-Maxi_Roman_2D | 500 | **Mini-Roman / Maxi-Roman 2♦** (4-4-4-1 or 5-4-4-0, 11-15 or 19+) | chat | yes: `two_level.two_diamonds.mini_roman` | `mini_roman_2d` |
| Mitchell_Stayman | 500 | **Mitchell Stayman**: after 1m (1NT), 2♣ shows 5-4 in the majors and 6-9 | chat | none | none |
| Preempt_Keycard | 500 (BBA: RKCB 0314 by 4NT, 314 notes) | **4♣ keycard ask over preempts** (4♦ over 3♣) | chat | none | none (sits under `roman_keycard`) |
| Reversed_Splinters_By_Opener | none | **Reversed splinters by opener** (Maas) | chat | none (`splinters.by_opener` is the normal kind) | none |
| Rosenkranz_Double | 500 | **Rosenkranz double / redouble** | chat: "BBO and BBA does not play Rosenkranz doubles" | none | none |
| Six_Key_RKC | none | **Six-key RKCB** (two agreed suits) | chat | none | none (sits under `roman_keycard`) |
| Size_Asking_Minor_Suit_Stayman | 500 | **1NT–2♠ range ask with a conditional 2NT diamond transfer** ("size-asking") | chat | close: `notrump.minor_transfers` could take a new option next to `four_way`. **Unsure** whether it counts as a four-way variant or its own convention | none |
| Tislevoll_after_Opps_Preempt | 500 | **Tislevoll slam try** over their preempt (4♣ control, 4♦ no control) | chat | none | none |
| Transfer_Advances | 500 | **Transfer advances** of partner's overcall | chat, bridgewinners link | yes: `overcalls.responses.new_suit = transfer` | `overcalls.responses.new_suit` (structured field) |
| We_Overcall_1N_then_Gerber | none | **Gerber over our 1NT overcall** | chat | partly: `nt_overcalls.direct.systems_on` + `slam.gerber.*` | `gerber` |
| Weak_NT_09-12, _09-15, _10-12, _10-13, _11-16 (5) | 500 each | **1NT ranges BBA cannot set**: its presets are 12-14, 13-15, 14-16 and 15-17 only | the chats give the ranges; BBA bid them with the Precision card's 13-15 | yes: `notrump.one_nt.range_min/max` take any range, and one-nt.bid reads them | 1NT range (structured field) |
| Western_Cue_Bid | 500 | **Western cue bid** (asks for a stopper for 3NT) | chat: "BBA does not play Western Cue Bid" | yes: `slam.western_cuebid.play` | `western_cuebid` |

For the 17 of these with a BBA corpus, the corpus is **not a reference** for the named convention: BBA bid them with its own methods. Appendix B shows 0 IMPs vs BBA for every one of them, so `rbb compare` appears not to score them. Only the dealing is usable.

### `bba-works: true`, but BBA plays a stand-in

Some scenarios are marked `bba-works: true`, yet BBA has no key for the convention they are named after, and its notes show another convention.

**Rick, 2026-09-28: `bba-works` means BBA is designed to support the given convention.** It does not mean "BBA's auctions are acceptable practice here". So for the scenarios below the flag and the corpus disagree: BBA is meant to play the named convention, but its notes show a stand-in. Possible reasons, none checked yet: the card the scenario uses does not switch the convention on, BBA plays it under an alert with another name, or the flag is wrong for that scenario. Until each is checked, treat their corpora as evidence of BBA's stand-in, not of the named convention. For us they are gaps for teaching, not for matching BBA:

| Scenario | Named convention | What BBA's notes show instead |
|---|---|---|
| Meckwell | Meckwell vs 1NT (in progress in our rules, 2026-09-28) | Multi-Landy (304) |
| Suction | Suction vs 1NT | Cappelletti (370) |
| Spear | Spear (after 1m (P) 1NT) | no Spear note; Michaels 18 |
| Help_Suit_Game_Try | help-suit game tries | two-way game tries (89) |
| Muiderberg_Two_Bids | Muiderberg | Polish two-suiters (171) |
| Transfer_Walsh, Vics_2C_Relay, Vics_Bal_Resp_to_1m, Ned_Weak_Two, Ned_2S, Ned_3-Level_Resp_to_1N, Gavin_* (12), Last_Train_*, Serious, Spiral_Raises_*, BART, 3_Under_Invitational_Jump, Impossible_2S, Namyats, XYZ, Double_Showing_2_Suits, Power_Double_*, Equal_Level_Conversion, Exit_Transfers, Mathe, Lead_Directing_Double | the named convention | no BBA note with the convention's name. BBA bids naturally or with its nearest key (e.g. Impossible_2S: forcing 1NT; 3_Under: Inviting Jump Shifts; Namyats: none although the SPECIALS card turns Namyats on) |

I did not read all 326 chats. This list comes from matching scenario names against BBA's notes, so it is a lower bound.

## 3. State of each convention in this engine, with its Classroom entry

States:

- **rules:** a module reads the card field.
- **partly:** rules cover part of it, or the field only switches natural bids off.
- **field only:** the field and its `.bbsa` mapping exist, but no rule reads the field.
- **passthrough:** the `.bbsa` key has no field.
- **none:** neither a field nor a BBA key.

"**In progress**" marks the conventions being implemented as of 2026-09-28 (section 4); their state is the one the survey measured, before that work.

"BBA boards" counts the boards whose BBA auction carries the convention's alert. "We differ" counts the boards where our replay first departs from BBA's auction at that alert. "IMPs" is the net vs BBA on those boards, with par as the yardstick; for competitive calls par is a poor judge (CLAUDE.md, "Judging a change").

"Classroom" gives the `conventionCatalog.js` id, and the Baker skill in brackets where there is one.

### Notrump

| Convention | BBA key(s) (cards on) | Engine | Module | BBA boards / we differ / IMPs | Classroom |
|---|---|---|---|---|---|
| 1NT opening range, shape | range keys (18); 5422 (14), 5M (16), 4441 (1) | rules for the range; shape: field only | notrump-base | | 1NT range fields (`notrump_openings`) |
| Stayman | implied | rules | stayman | 12,065 / 1,715 / +138 | `stayman` (stayman). The catalog keys it on `notrump.stayman.forcing`, the engine on `notrump.stayman.play` |
| Jacoby transfers | implied | rules | jacoby-transfers | | `jacoby_transfers` (jacoby_transfers) |
| Texas | 17 | rules | texas-transfers | 1,763 / 774 / -1,549 | `texas_transfers` |
| Super-accept; doubleton super-accept | 17; 10 | rules | super-accept, superaccept-doubleton | 469 / 167 / -183; 963 / 239 / -475 | none |
| Smolen | 17 | rules (2026-09-28) | smolen | 1,592 / 1,409 / -1,486 | `smolen` (stayman) |
| Garbage Stayman | 0 | field only | | | `garbage_stayman` |
| Minor-suit responses: 2♠→♣, 3♣→♦ | 15 | rules (the `bba` combination is derived) | minor-transfers | 380 / 101 / -198; 749 / 500 / -532 | none |
| Minor Suit Stayman; 1NT–2NT→♣ | 2; 1 | partly (the other combinations fall to `none`) | minor-transfers | 1,065 / 1,046 / -1,118; 413 / 238 / -786 | none |
| Puppet Stayman over 1NT (3♣) | 1 | field only | | 977 / 970 / -333 | none |
| Puppet Stayman over 2NT (**in progress**) | 1 | field only. The key maps to `notrump.stayman.puppet`, but two-nt-responses reads `notrump.two_nt.puppet` | two-nt-responses (Stayman and transfers) | 495 / 170 / -113 | `puppet_stayman` (on `stayman.puppet`) |
| 2NT: minor transfers, MSS, minor slam try | 15, 2, 0 | field only | | 79 / 48 / -28; 200 / 193 / -97 | 2NT options (structured) |
| 1NT–3♦ meaning; 1NT–3M splinters | 17; 2 | rules; field only | notrump-base | 163 / 151 / -17; 347 / 339 / -135 | responses (text fields) |
| Transfers on over X or 2♣ | 3; 18 | rules | jacoby-transfers, stayman, nt-interference | | `sys_on_vs` (text) |
| Lebensohl over interference to 1NT | 15 | field only (nt-interference plays natural) | | 147 / 0 / 0 | `lebensohl_interference` (lebensohl) |
| Rubensohl after 1NT | 2 | field only | | | none |
| Gambling 3NT | 13 | partly (the field only turns natural 3NT off; no Gambling rules) | strong-openings | 1,004 / 1,003 / -794 | 3NT one-suit (structured) |
| 4NT opening | 16 | field only | | 21 / 21 / -64 | none |
| `5431 after 1NT`, `Imposible 2S` (**in progress**; a major-opening convention, 1♥ 1NT 2♣ 2♠, listed here by its key), `1NT opening NT style` | 1, 9, 18 | passthrough | | 409 / 408 / -319 (5431) | none |

### Major openings and jump shifts

| Convention | BBA key(s) (cards on) | Engine | Module | BBA boards / we differ / IMPs | Classroom |
|---|---|---|---|---|---|
| Jacoby 2NT | 17 | rules | jacoby-2nt | 4,186 / 210 / -78 | `jacoby_2nt` (jacoby_2nt_splinters) |
| Splinters (responder, opener) | 17 | field only (splinters exist only after inverted minors) | | 2,697 / 1,822 / -1,738 | `splinters` (jacoby_2nt_splinters) |
| Forcing / semi-forcing 1NT | 10 / 0 | field only | responses bids 1NT, but nothing reads the field | 3,420 / 2,470 / -212 | `forcing_1nt`, `semi_forcing_1nt` |
| Jump raise: inviting / blocking / mixed | 14 / 4 / 8 | rules (inviting); field only | responses | 428 / 233 / -132 (mixed) | `mixed_jump_raise` |
| Bergen, reverse Bergen | 5, 3 | field only | | 199 / 190 / -193; 29 / 22 / -84 | `bergen_raises` |
| `Shape Bergen` / `Strength Lawrence structure` | 12 / 6 | passthrough | | | none |
| Drury, reverse Drury | 6, 11 | field only | | 144 / 56 / +9; 854 / 626 / -48 | `drury`, `reverse_drury` (reverse_drury) |
| Two-way game tries | 7 | field only | | 375 / 311 / +425 | none (`help_suit_game_tries` is the other kind) |
| Gazzilli | 2 | field only | | 209 / 126 / -63 | none |
| Strong jump shifts (2 level); inviting jump shifts | 16; 8 | rules | responses | 1,347 / 1,128 / -534 (inviting) | none |
| Weak jump shifts (2 / 3 level) | 5 / 5 | field only | | 283 / 281 / -314; 90 / 86 / -76 | none |
| Soloway jump shifts (few players use them: low priority, Rick 2026-09-28) | 1 (SPECIALS2) | field only | | **1,514 / 1,497 / -7,103** | none |
| 2/1 game force | preset | field only (derived; the rules assume 2/1) | | | `two_over_one` (two_over_one) |
| 3NT picture response | none | none | | | none |

### Minor openings

| Convention | BBA key(s) (cards on) | Engine | Module | BBA boards / we differ / IMPs | Classroom |
|---|---|---|---|---|---|
| Inverted minors | 16 | rules | inverted-minors | 1,948 / 24 / -115 | `inverted_minors` |
| Walsh | 5 | partly. `walsh.play` is field only; the rules read their own `minor_openings.one_club_responses` | responses, rebids | | `walsh` |
| 1m may hold a 5-card major; 1♦ promises 4 or 5 | 16; 0 | field only | | 18 / 10 / +2 | 1♦ min length (structured) |
| 1m–2NT meaning | none (our own field) | rules | responses | | none |
| Todd responses (1m-2x) | none | none | | | none |

### Two-level openings and preempts

| Convention | BBA key(s) (cards on) | Engine | Module | BBA boards / we differ / IMPs | Classroom |
|---|---|---|---|---|---|
| Weak twos (2♦, 2M) | 13, 16 | rules | preempts, weak-two-responses | | `weak_2d/2h/2s` (weak_2s) |
| Strong 2♣ | preset | rules | strong-openings | | `strong_2c` (strong_2c), `parrish_2h_bust` |
| Ogust | 1 | rules | weak-two-responses | 318 / 94 / +38 | `ogust` (ogust) |
| Kokish relay | 2 | field only | | 264 / 102 / -76 | `kokish` |
| Multi 2♦ | 1 | field only | | 696 / 696 / -528 | `multi_2d` |
| Flannery; reverse Flannery 2♥/2♠ | 1; 2/1 | field only | | 499 / 499 / -15; 499 / 498 / -235 | `reverse_flannery_2d` (a 2♦ version; no Flannery row) |
| Benjamin 2♦ | 1 | field only | | 265 / 265 / -719 | none |
| Polish two-suiters | 1 | field only | | 171 / 171 / +2 | none |
| Precision 2♦ | 2 | field only | | 68 / 68 / +11 | none |
| Mini-Roman 2♦ | none | field only (no BBA key) | | | `mini_roman_2d` |
| Namyats | 7 | field only | | no BBA note seen | none (`preempts.transfer_4_minor` is similar) |
| Keycard over preempts | none | none | | | none |

### Slam

| Convention | BBA key(s) (cards on) | Engine | Module | BBA boards / we differ / IMPs | Classroom |
|---|---|---|---|---|---|
| RKCB 1430 / 0314 | 14 / 3 | rules | rkcb-1430 | 13,058 / 1,148 / **-6,932**; 2,891 / 408 / -346 | `rkcb_1430`, `rkcb_0314` (roman_keycard) |
| Plain Blackwood | 1 (Basic-Bridge) | rules | blackwood | 782 / 183 / -665 | `standard_blackwood` (blackwood) |
| Control (cue) bids | 17 | rules | control-bids | 8,103 / 1,114 / -986 | `control_bids` (text) |
| Quantitative 4NT | 18 | partly (rules in over-3nt and one-nt, not gated by the field) | over-3nt, notrump-base | 2,130 / 569 / -938 | none |
| Gerber | 17 (+3 only over NT) | rules (2026-09-28; the only-over-openings switch too) | gerber | 1,403 / 892 / -900 | `gerber` (blackwood) |
| King ask by 5NT / next step | 18 / 0 | field only (a TODO in rkcb-1430) | | 1,387 / 17 / -126 | `queen_ask` is the nearest |
| Exclusion | 7 | field only | | 1,163 / 182 / -707 | `exclusion_blackwood` |
| 5NT pick-a-slam | 7 | field only | | 444 / 178 / -455 | `pick_a_slam_5nt` |
| DOPI / ROPI / DEPO | 17 / 18 / 0 | field only | | | DOPI/DEPO/ROPI (structured) |
| Kickback, Crosswood | 0, 2 | field only; passthrough | | 57 / 3 / -24 (Crosswood) | `kickback` |
| Grand slam force | none (BBA plays it; scenarios GSF*) | none | | | none |
| Six-key RKCB, Minorwood, spiral cue bids, non-serious 3NT | none | none (fields for the last three) | | | `minorwood`, `spiral_cuebids`, `non_serious_3nt` |

### Competitive: our overcalls, doubles and defences

| Convention | BBA key(s) (cards on) | Engine | Module | BBA boards / we differ / IMPs | Classroom |
|---|---|---|---|---|---|
| Simple and jump overcalls, 1NT overcall | implied | rules | overcalls, advances, balancing | | overcall fields (overcalls) |
| Takeout double, power double | implied | rules | takeout-double, vs-preempts | | `takeout_doubles` (takeout_doubles) |
| Negative double | implied | rules | after-interference | | `negative_doubles` (negative_doubles) |
| Responsive double | 14 | rules | responsive-doubles | 381 / 41 / -14 | `responsive_doubles` |
| Cue-bid raise (limit raise or better) | implied | rules | advances, after-interference | 5,374 / 828 / -1,299 | `cue_bid_raise` (support_cuebids) |
| 1X-(Y)-2Z forcing | 15 | rules | after-interference, rebids | 1,809 / 457 / -474 | none |
| Support double / redouble | 17 | field only | | 1,181 / 65 / +2 | `support_doubles` |
| Maximal doubles | 3 | field only | | 7 / 2 / 0 | `maximal_doubles` |
| Michaels cuebid | 17 | rules (2026-09-28; the survey figures are from before) | two-suited-overcalls | **3,790 / 3,559 / -3,452** | `michaels` (michaels_unusual) |
| Unusual 2NT / 1NT / 4NT | 17 each | 2NT: rules (2026-09-28); 1NT, 4NT: field only | two-suited-overcalls | 2,102 / 1,875 / -772; 30 / 18 / -11; 278 / 190 / -180 | `unusual_2nt` (michaels_unusual) |
| Leaping / non-leaping Michaels | 3 / 0 | field only | | no separate BBA note | `leaping_michaels` |
| Cappelletti / Multi-Landy / Landy vs 1NT | 10 / 7 / 0 | rules (2026-09-28; field `competitive.vs_1nt_strong.system`) | cappelletti, multi-landy, vs-1nt | **4,612 / 4,409 / +462**; 495 / 479 / +109 | VsNtDefense panel (structured); `dont` |
| DONT, Meckwell, Modified Cappelletti; Suction, Spear, CRASH | none | rules for the first three (2026-09-28, options of `vs_1nt_strong.system`); none for the others | dont, meckwell, cappelletti | | `dont` (dont) |
| Lebensohl after doubling a weak two; Rubensohl | 15; 2 | field only | | 1,138 / 806 / +121 | `lebensohl_weak_twos` (lebensohl) |
| `Lebensohl after 1m`, `Rubensohl after 1m` (meaning unknown to Rick: to be investigated) | 15; 2 | passthrough | | 227 / 119 / -251; 90 / 15 / -62 | none |
| Jordan (Truscott) 2NT | 17 | field only | | 859 / 649 / -1,476 | none |
| Fit-showing jumps | 7 | field only | | 361 / 320 / -694 | none |
| 1X-(1Y)-2Z weak | 17 | field only | | 63 / 51 / -36 | none |
| Direct jump cuebids | 1 | field only | | | DirectCuebidsMatrix (structured) |
| BROMAD, Snapdragon, scrambling 2NT, Ghestem, Raptor, Unusual vs Unusual | 6, 0, 0, 0, 0, 0 | field only | | no BBA note | `snapdragon`, `unusual_vs_unusual` |
| `Two suit takeout double`, `Support 1NT`, `Collante`, `Rodrigue`, `Jordan Truscott 2NT defence` | 17, 6, 1, 1, 1 | passthrough | | 75 / 38 / -6; 32 / 21 / +2; 108 / 33 / -50; 352 / 351 / -192 | none |
| Roudi | 1 | field only | | 251 / 238 / -750 | none |
| Rosenkranz, good-bad 2NT, Western cue, Mitchell Stayman, Tislevoll, Thrump X, transfer advances, lead-directing X, sandwich NT, SOS redouble | none | none (fields exist for Western cue, transfer advances, lead-directing X, sandwich NT, SOS) | | | `western_cuebid`, `lead_directing_double`, `sandwich_nt`, `sos_redouble` |

### Other

| Convention | BBA key(s) (cards on) | Engine | Module | BBA boards / we differ / IMPs | Classroom |
|---|---|---|---|---|---|
| New minor forcing | 12 | rules | new-minor-forcing, responder-rebids | 1,076 / 44 / -39 | `nmf` (new_minor_forcing) |
| NMF after a 2NT rebid; by a passed hand | 16; 0 | field only | | 235 / 151 / -90 | none |
| Two-way NMF | 2 | field only | | 49 / 27 / -66 | `two_way_nmf` |
| Checkback | 2 | field only | | | none |
| Fourth suit forcing: game force / one round | 11 / 6 | field only (rebids treats a one-level fourth suit as natural) | | 1,006 / 609 / -981; 480 / 311 / -616 | `fourth_suit_forcing_gf/_1rnd` (fourth_suit_forcing) |
| Reverses | preset | rules | rebids, responder-rebids | | `reverse_bids` (reverse_bids) |
| XYZ, Ingberman | none | none (fields exist) | | | `xyz`, `ingberman_2nt` |
| Precision system (strong 1♣, 1♦, 2♣, 2♦); in scope (Rick, 2026-09-28) | `System type` = Precision (2 cards) | field only (the preset derives structural fields; there are no strong-club rules) | | "16+ HCP": **8,149 / 5,698 / -7,106** | `transfer_responses_1c` is the nearest; none for strong club |

### Taxonomy notes

- The Classroom catalog has **67** entries. The rules read the fields of 17 of them; Stayman makes 18, although the catalog keys it on another field.
- 40 catalog entries have a Baker skill.
- Of the 25 Baker bidding skills (3 basic, 15 conventions, 7 competitive), the engine covers the three basic ones and, partly or wholly these: stayman, jacoby_transfers, jacoby_2nt_splinters (Jacoby 2NT yes, splinters no), NMF, reverse_bids, roman_keycard, blackwood, strong_2c, two_over_one, weak_2s, ogust, preemptive_bids, negative_doubles, overcalls, takeout_doubles, support_cuebids.
- Missing: **michaels_unusual, dont, lebensohl, fourth_suit_forcing, help_suit_game_try** and reverse_drury.
- Catalog paths that disagree with the engine:
  - Stayman: the catalog keys it on `notrump.stayman.forcing`; the engine gates on `notrump.stayman.play`.
  - Puppet over 2NT: the `.bbsa` import writes `notrump.stayman.puppet`, while two-nt-responses reads `notrump.two_nt.puppet`.
  - Walsh: `walsh.play` sits beside the engine's own `one_club_responses`.
  - Blackwood and Gerber: the catalog uses the `other_conventions.*` paths, which `fields.toml` loads through aliases, so these are fine.
- BBA keys with no catalog entry include: Soloway, Jordan 2NT, fit-showing jumps, Cappelletti and Multi-Landy (only the VsNtDefense text panel), super-accept, Flannery, Namyats, Benjamin, Polish two-suiters, two-way game tries and Gazzilli. Adding catalog rows for the high-priority ones below would keep the editor and the engine in step.

## 4. Gap table and recommended priority

Each convention is weighed by what it would unlock against its effort:

- **Unlocks:**
  - boards where our replay first departs from BBA at the convention's alert;
  - IMPs lost there, with par as the yardstick;
  - scenarios built around the convention.
- **Effort:** S is up to about 100 lines of `.bid`, like super-accept, Jacoby 2NT, NMF or responsive doubles. M is 100-300 lines, like RKCB, minor transfers, overcalls or advances. L is 300-600 lines, like Stayman, the takeout double or vs-preempts. XL is a system of several modules.

The ranking takes both unlock figures per unit of effort (S=1, M=2, L=3, XL=5), with judgement where they disagree. Competitive entries also count their "we differ" boards, since par misjudges competitive calls.

**Rick's decisions (2026-09-28)** change the ranking the figures alone would give:

- **Soloway jump shifts: few players use them.** Pushed down the list, although they are the largest IMP loss per line in the corpus (one card, 21GF-SPECIALS2, plays them).
- **Precision is in scope**, large and complex. Pushed down, but "not next but not too far": of everything here it is the convention most likely to force engine changes, and those are cheaper found early.
- **Defences to 1NT: Cappelletti, Modified Cappelletti, Meckwell and DONT**, for compatibility with the cards people play. Comparisons between them (par, side IMPs) are for information; they do not decide which to build.
- **Puppet Stayman over 2NT** is wanted working.
- **Impossible 2♠** has named sources (section 1).

### In progress (2026-09-28)

Being implemented now, in other pieces of work; the figures are the survey's, from before that work:

| Convention | We differ / IMPs | Scenarios built on it | State at survey | Effort | Notes |
|---|---|---|---|---|---|
| **Defences to 1NT:** Cappelletti, Modified Cappelletti, Meckwell, DONT (ours, and reading EW's) | Cappelletti 4,409 / +462 | Cappelletti, Cappelletti_in_4th, Meckwell, DONT (bba-works: false), and every 1NT scenario, where EW (21GF-GIB) plays Cappelletti: 74 scenarios carry the note | partly (Cappelletti), none (the others) | M | Rick: for compatibility. Replay fidelity: EW's calls over our 1NT that we do not make (1NT X, 2♣, 2♦, 2♠, 2♥) are five of the top 17 divergence points, 4,372 boards. Judge with par and sideimps.py, for information |
| **Puppet Stayman over 2NT** | 170 / -113 | 2N_and_MSS and the 21GF-MSTandMSS scenarios | field only (and the `two_nt.puppet` / `stayman.puppet` path mismatch) | S | Rick: wanted working |
| **Impossible 2♠** (1♥ 1NT 2♣ 2♠, a good club raise) | not attributable: BBA shows no alert with the name | Impossible_2S (44% call agreement) | passthrough (`Imposible 2S`, on in 9 cards) | S | Sources: Robert Todd's write-up and the GIB reference (section 1), which may differ |

### Remaining, ranked

| # | Convention | We differ / IMPs | Scenarios built on it | State | Effort | Why this rank |
|---:|---|---|---|---|---|---|
| 1 | **Michaels cuebid + Unusual 2NT** (making them, and reading them for EW, whose 21GF-GIB card plays both) | 5,434 / -4,224 | Michaels_Cuebid, Unusual_2N, Michaels_and_Unusual, Michaels_after_1m, Opps_Michaels_Cuebid, Opps_Michaels_and_Unusual, Two-Suited_Overcalls, Opps_2-Suited_Overcalls, Leaping_Michaels, Non_Leaping_* (11), all at 58-63% call agreement | **done 2026-09-28** (two-suited-overcalls; Leaping and non-leaping Michaels still field only) | M-L | Most divergence boards of any missing convention; Baker skill michaels_unusual. Now first, with Soloway pushed down |
| 2 | **Smolen** (built 2026-09-28: smolen.bid) | 1,409 / -1,486 | Smolen, Smolen_Invitational, Smolen_after_2N, We_Overcall_NT_then_Smolen (28 carry the note) | field only | S | Cheap; fits on stayman.bid |
| 3 | **Jordan 2NT** | 649 / -1,476 | Jordan_2N, Xfer_after_1M_X (43 carry the note) | field only | S | Cheap; after-interference already handles 1M (X) |
| 4 | **Gerber** (over 1NT/2NT; by opener later; built 2026-09-28 over openings and rebids: gerber.bid) | 892 / -900 | Gerber, Gerber_By_Responder, Gerber_By_Opener, Slam_after_NT… (56 carry the note) | field only | S | Cheap; the rkcb answer machinery exists |
| 5 | Gambling 3NT | 1,003 / -794 | Gambling_3N, Opps_Gambling_3N (both at about 57%) | partly | S | Cheap |
| 6 | **Precision strong club**, stage 1: 1♣ 16+, the responses, 1♦, 2♣ | 5,698 / -7,106 at "16+ HCP", plus the opening divergences (1♣ vs 1♦/1♥/1♠: 4,830 boards, about -6,300) | Strong Club (18) + Strong Club 16+ (12) + Mathe: 15,500 boards | field only (preset) | XL | Rick: in scope, "not next but not too far". The largest family we cannot bid at all (38-55% agreement), and the likeliest to need engine changes. Split it into stages |
| 7 | **Splinters** (responder's and opener's) | 1,822 / -1,738 | Splinters, Splinters_By_Opener, Splinters_after_Minor, Gavin_*_Splinter (183 carry the note) | field only | M | Spread over many scenarios; Baker skill jacoby_2nt_splinters |
| 8 | **Minor Suit Stayman family** (1NT–2♠ MSS, 1NT–2NT→♣, MSS after 2NT) | 1,477 / -2,001 | Minor_Suit_Stayman, MST_or_MSS (-4.6 IMPs a board, among the eight worst scenarios), Minor_Suit_Transfer, 2N_and_MSS, We_Overcall_NT_then_MSS/MST | partly | M | Needs a new `minor_transfers` combination for the MSS cards; shares 2N_and_MSS with the Puppet work |
| 9 | **Fourth suit forcing** (game force and one round) | 920 / -1,597 | Fourth_Suit_Forcing (74 carry the note) | field only | M | Baker skill fourth_suit_forcing; closes a hole in responder's rebids |
| 10 | Multi-Landy (ours and theirs) | 479 / +109 | Multi_Landy; BBA's stand-in in Meckwell | field only | S | Not in Rick's list of defences to 1NT; cheap once that work lands |
| 11 | Forcing 1NT (opener's rebids after it) | 2,470 / -212 | Forcing_NT, BART, Last_Train… (77) | field only | M | Many divergences, few IMPs |
| 12 | Lebensohl after a double of a weak two; Lebensohl over 1NT interference | 806 / +121 | Lebensohl_vs_Opps_W2_*, Better_Minor_Lebensohl, McCabe_After_Weak_2, Lebensohl | field only | M | Baker skill lebensohl. The weak-two version was tried on 2026-09-27 and lost on both yardsticks (vs-preempts.notes.md) |
| 13 | Exclusion | 182 / -707 | Exclusion_After_1M, Exclusion_After_Sta_Jac | field only | S | |
| 14 | Fit-showing jumps | 320 / -694 | Fit_Showing_Jumps, Fit_Jumps_after_1M_Double | field only | S | |
| 15 | Multi 2♦ (ours and theirs) | 696 / -528 | Multi_2D, Opps_Multi_2D (both at 60%) | field only | M | |
| 16 | **Soloway jump shifts** | 1,497 / -7,103 | Soloway_Jump_Shift + Type-1..4 (5) | field only | M | Rick: few players use them. By the figures alone this would be first |
| 17 | Roudi, Benjamin 2♦, reverse Flannery, weak jump shifts, 5NT pick-a-slam, Puppet over 1NT, Rodrigue | each 170-500 / -100 to -750 | one card, one or two scenarios each | field only (Rodrigue passthrough) | S each | Card-specific; do them with their scenarios |
| 18 | Western cue bid, transfer advances, 1NT ranges outside BBA's presets | no BBA reference | 7 of the bba-works: false scenarios | fields exist | S-M | Teaching value, but no corpus to test against: judge by par and `.test` cases only |
| 19 | Other bba-works: false gaps (picture 3NT, Todd, Mitchell, Rosenkranz, good-bad 2NT, Tislevoll, Thrump X, CRASH, keycard over preempts, six-key RKCB, reversed splinters, Lars Multi) | no BBA reference | one scenario each; 7 have no corpus at all | none: a new field each | S-M each | After the above |

Recommended order:

1. **Finish the work in progress:** the four defences to 1NT, Puppet Stayman over 2NT, and Impossible 2♠.
2. **The big conventional gap:** Michaels/Unusual 2NT.
3. **Quick wins:** Smolen, Jordan 2NT, Gerber and Gambling 3NT. All are S; together they account for 3,953 divergence boards and about -4,650 IMPs.
4. **Precision:** a project of its own, in stages, started here rather than last so that the engine changes it forces come early. Stage 1 is the strong 1♣ and its responses, which covers the 12 SCS16 and 11 SCS scenarios.
5. **Medium modules:** splinters, the MSS family and FSF.
6. **The rest of the table**, with Soloway near the end on Rick's ruling.

### Implemented, but still costly

These are not gaps; they are tuning work on modules we have:

- **Slam exploration.** Three of the four worst scenarios per board are grand-slam scenarios, and a fourth is eighth:
  - GSF_after_Preempt: -11.3 IMPs a board;
  - Grand_Slam_Invite: -6.6;
  - Grand_Slam_Force: -6.2;
  - Grand_Slam_Force_2: -4.2.

  Together they lose about -14,100 IMPs. At their first divergences BBA's call was either unalerted (-6,447) or its RKCB 4NT (-4,178). By call, the largest losses are BBA's 4NT where we bid game (4♠ -1,554, 4♥ -1,278). The grand slam force is not the cause.
- **RKCB:** corpus-wide, we fail to ask where BBA does on 1,148 boards, worth -6,932 IMPs. This is the single largest per-convention figure in the survey.
- **Texas:** -1,549 IMPs.
- **Control bids:** -986.
- **Quantitative 4NT:** -938.
- **Stayman:** 1,715 divergences, but +138 IMPs.

If tuning counts as priority work, slam entry (when to use keycard) belongs above everything in the gap table. It is not a missing convention, though.

## 5. Work done vs work remaining on the convention config files

| File | Done | Remaining | How estimated |
|---|---|---|---|
| `bbsa-map.toml` | 145 of 173 keys mapped (84%) | 28 passthrough keys; 16 are on in a used card. `Imposible 2S` now has named sources (in progress); the other 15 are to be investigated, since Rick does not know `Lebensohl after 1m` or `Rubensohl after 1m` either (section 1) | key count |
| `fields.toml` | 321 fields. They express every BBA key except the 28 above, 5 of the 20 named bba-works: false gaps fully (DONT, Mini-Roman, Western cue, transfer advances, 1NT range), and 3 partly (Gerber by opener, Gerber after a 1NT overcall, size-asking) | About 25 new fields: ~15 for passthrough keys once named, ~12 for gap conventions with no field. Plus 3 path fixes (Stayman catalog path, `two_nt.puppet` vs `stayman.puppet`, `walsh.play` vs `one_club_responses`). **About 90% done** | field count |
| `.bid` modules | 34 modules, 6,535 lines, reading 63 fields. They cover 32 of the 117 keys in use (27%; 36% weighted by boards), and all the unkeyed basics (openings, responses, rebids, overcalls, takeout and negative doubles, advances, vs preempts) | By the effort classes above: about 42 S units (~60 lines each, ~2,500), about 16 M units (~200 each, ~3,200, Michaels/Unusual counted as L), Precision (~2,000), and ~20 gap conventions with no BBA key (~100 each, ~2,000). **About 9,700 lines**, plus about 2,700 lines of `.test` at today's ratio | S/M/L sizes taken from the lines in today's comparable modules |

Put together, by lines of rules the `.bid` work is roughly **40% done and 60% to go** (6.5k of about 16k). By conventions switched on in the PBS cards it is **27-36% done**. The two figures differ because the done part includes the natural base system, which carries more lines per card key than a single convention does.

The remaining share also understates the tuning that follows each module. Today's modules still lose on RKCB entry, Texas and control bids (above), and every new module will need the same probe-and-tune rounds that the existing notes record.

## Open questions for Rick

Answered on 2026-09-28:

1. **Passthrough keys** (partly answered). `Imposible 2S` is the impossible 2♠ (1♥ 1NT 2♣ 2♠, a good club raise); sources: Robert Todd's write-up (<https://static1.squarespace.com/static/5127d3d2e4b0b304f0b6db24/t/5c8e551f104c7b066eb43ac8/1552831776048/3+%28269%29+The+Impossilbe+Spade+Bid+.pdf>) and the older GIB reference (<https://netbridge.dk/gib.html>), which may treat it differently; in progress. Rick does not know what `Lebensohl after 1m` and `Rubensohl after 1m` are. **Still open:** those two and the other in-use passthrough keys stay to be investigated: `Shape Bergen structure` vs `Strength Lawrence structure`, `Two suit takeout double`, `Support 1NT`, `5431 after 1NT`, `1NT opening NT style`, `Collante`, `Rodrigue`, `Crosswood 0314`, `Natural 3N entering style`, `Mark on queen/king`, `Jordan Truscott 2NT defence`. Each can be probed with `rbb grid` once there is a candidate meaning to test; this survey does not guess.
2. **What `bba-works` means.** Answered: BBA is designed to support the given convention. The scenarios where the flag is true but BBA's notes show a stand-in (section 2) are therefore to be checked one by one.
3. **Cappelletti.** Answered: implement Cappelletti, Modified Cappelletti, Meckwell and DONT, for compatibility with the cards people play; comparisons between them are for information. In progress.
4. **Precision.** Answered: in scope, large and complex; pushed down the list but not too far, since it is the convention most likely to force engine changes. **Still open:** the stages (the proposal is stage 1 = the strong 1♣, its responses, 1♦ and 2♣).
5. **Soloway** (not asked, ruled): few players use it; pushed down the list.
6. **Puppet Stayman over 2NT** (not asked, ruled): wanted working; in progress.

Still open:

7. **Size-asking MSS.** Is it a new option of `notrump.minor_transfers`, or its own convention?
8. **Classroom catalog.** Should the catalog gain rows for the priority conventions that it lacks: Jordan 2NT, Cappelletti/Multi-Landy (and Modified Cappelletti, Meckwell), fit-showing jumps, super-accept, Flannery, two-way game tries, Soloway?
9. **The `bba-works` stand-ins.** With `bba-works` meaning "BBA is designed to support it", should the stand-in scenarios (Meckwell, Suction, Spear, Muiderberg, Help_Suit_Game_Try, Namyats, Impossible_2S…) be reported to the PBS maintainers, or re-run with a card that switches the convention on?

## How this was made

```
cargo build --release -p rbb-cli
target/release/rbb card coverage -v ../Practice-Bidding-Scenarios/bbsa/*.bbsa
target/release/rbb compare --pbs <PBS> --json compare.json          # all 343 scenarios, ~90 s
python3 probes/tools/conventions_survey.py --compare compare.json --json survey.json > survey.md
```

`conventions_survey.py` gathers its figures as follows:

- **Cards:** it reads the `.bbsa` cards.
- **Scenarios:** it reads the `.btn` headers and chat, and takes each scenario's cards from the first `% CC1/CC2` line of its BBA PBN, as `rbb compare` does, falling back to the `.btn`.
- **BBA notes:** it counts the `[Note]` alerts in `bba/*.pbn`, normalised by stripping suits and counts.
- **Engine state:** it reads `bbsa-map.toml` and `fields.toml`. A key counts as read when a `.bid` `card` or `param` line names its field, or a field derived from it.
- **Divergences:** it attributes each first divergence in `compare.json` to BBA's alert on that call. The IMPs are per board, imp(|BBA − par|) − imp(|ours − par|), as `rbb compare` scores them.
- **Classroom:** it reads the catalog with a regex over `conventionCatalog.js`.

The appendices below are its output, lightly trimmed.

## Appendix A: every BBA key

"cards on" counts the cards (of 18) that switch the key on. "Scenarios, NS / EW" counts the scenarios that use such a card on that side, taking each scenario's cards from its corpus header (or its `.btn` when there is no corpus). "Engine" gives the key's state:

- **rules:** a `.bid` module reads the field, or a field derived from it.
- **field only:** the key is mapped to a field that nothing reads.
- **passthrough:** the key is kept verbatim, with no field.

"rules" can overstate: Cappelletti and Gambling are read only to switch natural calls off (section 3).

#### General

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| System type | 4 | 61 / 26 | field only | `general.system_category` |  |

#### Notrump openings

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| 1NT opening allows less 1HCP | 0 | 0 / 0 | rules | `notrump.one_nt.allow_one_less` | notrump-base |
| 1NT opening range 12-14 | 0 | 0 / 0 | rules | `notrump.one_nt.range_min`, `notrump.one_nt.range_max` | notrump-base |
| 1NT opening range 13-15 | 1 | 29 / 2 | rules | `notrump.one_nt.range_min`, `notrump.one_nt.range_max` | notrump-base |
| 1NT opening range 14-16 | 1 | 1 / 0 | rules | `notrump.one_nt.range_min`, `notrump.one_nt.range_max` | notrump-base |
| 1NT opening range 15-17 | 16 | 320 / 348 | rules | `notrump.one_nt.range_min`, `notrump.one_nt.range_max` | notrump-base |
| 1NT opening shape 4441 | 1 | 14 / 1 | field only | `notrump.one_nt.allow_4441` |  |
| 1NT opening shape 5422 | 14 | 295 / 324 | field only | `notrump.one_nt.allow_5422` |  |
| 1NT opening shape 5 major | 16 | 320 / 348 | field only | `notrump.one_nt.five_card_major` |  |
| 1NT opening shape 6 minor | 0 | 0 / 0 | field only | `notrump.one_nt.allow_6_minor` |  |

#### Responses to 1NT / 2NT

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| 1N-2S Minor Suit Stayman | 2 | 9 / 0 | rules | `notrump.minor_suit_stayman.play` | minor-transfers, notrump-base, stayman |
| 1N-2S transfer to clubs | 15 | 317 / 326 | rules | `notrump.transfers.two_s_clubs` | minor-transfers, notrump-base, stayman |
| 1N-2N transfer to clubs | 1 | 7 / 0 | rules | `notrump.transfers.two_nt_clubs` | minor-transfers, notrump-base, stayman |
| 1N-2N transfer to diamonds | 0 | 0 / 0 | rules | `notrump.transfers.two_nt_diamonds` | minor-transfers, notrump-base, stayman |
| 1N-3C transfer to diamonds | 15 | 322 / 326 | rules | `notrump.transfers.three_c_diamonds` | minor-transfers, notrump-base, stayman |
| 1N-3C Puppet Stayman | 1 | 2 / 0 | field only | `notrump.stayman.puppet_1nt` |  |
| 1N-3D majors | 0 | 0 / 0 | rules | `notrump.three_d_response` | notrump-base |
| 1N-3D minors | 0 | 0 / 0 | rules | `notrump.three_d_response` | notrump-base |
| 1N-3D natural | 14 | 284 / 32 | rules | `notrump.three_d_response` | notrump-base |
| 1N-3D splinter | 3 | 65 / 318 | rules | `notrump.three_d_response` | notrump-base |
| 1N-3M splinter | 2 | 36 / 316 | field only | `notrump.three_major_splinter` |  |
| 2N-3C-3N both majors | 0 | 0 / 0 | field only | `notrump.two_nt.three_c_3nt_both_majors` |  |
| 2N-3C Puppet Stayman | 1 | 7 / 0 | field only | `notrump.stayman.puppet` |  |
| 2N-3S transfer to clubs | 0 | 0 / 0 | field only | `notrump.two_nt.transfer_3s_clubs` |  |
| 2N-4C transfer to diamonds | 0 | 0 / 0 | field only | `notrump.two_nt.transfer_4c_diamonds` |  |
| Extended acceptance after NT | 10 | 243 / 7 | rules | `notrump.transfers.super_accept_doubleton` | superaccept-doubleton |
| Gambling | 13 | 274 / 9 | rules | `notrump.three_nt.one_suit` | strong-openings |
| Garbage Stayman | 0 | 0 / 0 | field only | `notrump.stayman.garbage` |  |
| Lebensohl after 1NT | 15 | 296 / 324 | field only | `notrump.lebensohl.over_interference` |  |
| Minor Suit Slam Try after 2NT | 0 | 0 / 0 | field only | `notrump.two_nt.minor_slam_try` |  |
| Minor Suit Stayman after 2NT | 2 | 3 / 0 | field only | `notrump.two_nt.minor_stayman` |  |
| Minor Suit Transfers after 2NT | 15 | 323 / 326 | field only | `notrump.two_nt.minor_transfers` |  |
| Rubensohl after 1NT | 2 | 30 / 2 | field only | `notrump.rubensohl.over_interference` |  |
| SMOLEN | 17 | 326 / 326 | rules | `notrump.smolen.play` |  |
| Super acceptance after NT | 17 | 326 / 326 | rules | `notrump.transfers.super_accept` | super-accept |
| Texas | 17 | 326 / 326 | rules | `notrump.transfers.texas` | jacoby-transfers, texas-transfers |
| Transfers if RHO doubles | 3 | 60 / 340 | rules | `notrump.transfers.vs_double` | jacoby-transfers, nt-interference, stayman |
| Transfers if RHO bids clubs | 18 | 350 / 350 | rules | `notrump.transfers.vs_2c` | jacoby-transfers, minor-transfers, notrump-base, stayman, texas-transfers |

#### General / jump shifts

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| Inviting Jump Shifts | 8 | 280 / 346 | rules | `general.jump_shifts.invitational` | responses |
| Soloway Jump Shifts | 1 | 14 / 1 | field only | `general.jump_shifts.soloway` |  |
| Soloway Jump Shifts Extended | 0 | 0 / 0 | field only | `general.jump_shifts.soloway_extended` |  |
| Strong jump shifts 2 | 16 | 323 / 348 | rules | `general.jump_shifts.strong_2` | responses |
| Strong jump shifts 3 | 0 | 0 / 0 | field only | `general.jump_shifts.strong_3` |  |
| Weak Jump Shifts 2 | 5 | 24 / 2 | field only | `general.jump_shifts.weak_2` |  |
| Weak Jump Shifts 3 | 5 | 12 / 1 | field only | `general.jump_shifts.weak_3` |  |

#### Major openings

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| 1M-3M blocking | 4 | 32 / 2 | field only | `major_openings.jump_raise.weak` |  |
| 1M-3M inviting | 14 | 318 / 348 | rules | `major_openings.jump_raise.inv` | responses |
| Bergen | 5 | 23 / 1 | field only | `major_openings.bergen_raises.play` |  |
| Drury | 6 | 82 / 319 | field only | `major_openings.drury.play` |  |
| Forcing 1NT | 10 | 239 / 7 | field only | `major_openings.one_nt_response.forcing` |  |
| Gazzilli | 2 | 8 / 0 | field only | `major_openings.gazzilli.play` |  |
| Jacoby 2NT | 17 | 326 / 326 | rules | `major_openings.jacoby_2nt.play` | jacoby-2nt |
| Mini Splinter | 0 | 0 / 0 | field only | `major_openings.mini_splinters.play` |  |
| Mixed raise | 8 | 96 / 320 | field only | `major_openings.jump_raise.mixed` |  |
| Reverse Bergen | 3 | 45 / 3 | field only | `major_openings.bergen_raises.reverse` |  |
| Reverse drury | 11 | 244 / 7 | field only | `major_openings.drury.reverse` |  |
| Semi forcing 1NT | 0 | 0 / 0 | field only | `major_openings.one_nt_response.semi_forcing` |  |
| Splinter | 17 | 326 / 326 | field only | `major_openings.splinters.play` |  |

#### Minor openings

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| 1D opening with 4 cards | 0 | 0 / 0 | field only | `minor_openings.one_diamond.min_length` |  |
| 1D opening with 5 cards | 0 | 0 / 0 | field only | `minor_openings.one_diamond.min_length` |  |
| 1m opening allows 5M | 16 | 348 / 350 | field only | `minor_openings.may_hold_5_card_major` |  |
| Inverted minors | 16 | 320 / 326 | rules | `minor_openings.inverted_minors.play` | inverted-minors, responses |
| Walsh style | 5 | 115 / 344 | field only | `minor_openings.walsh.play` |  |

#### Two-level openings

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| Benjamin 2D | 1 | 2 / 0 | field only | `two_level.benjamin.play` |  |
| Flannery | 1 | 1 / 0 | field only | `two_level.flannery.play` |  |
| French 2D | 0 | 0 / 0 | field only | `two_level.french_2d.play` |  |
| Kokish Relay | 2 | 42 / 3 | field only | `two_level.two_clubs.kokish` |  |
| Multi | 1 | 2 / 1 | field only | `two_level.two_diamonds.multi` |  |
| Ogust | 1 | 1 / 0 | rules | `two_level.ogust.play` | weak-two-responses |
| Polish two suiters | 1 | 2 / 0 | field only | `two_level.polish_two_suiters.play` |  |
| Precision 2D | 2 | 30 / 2 | field only | `two_level.precision_2d.play` |  |
| Reverse Flannery 2H | 2 | 14 / 1 | field only | `two_level.reverse_flannery.two_h` |  |
| Reverse Flannery 2S | 1 | 13 / 1 | field only | `two_level.reverse_flannery.two_s` |  |
| Strong natural 2D | 0 | 0 / 0 | rules | `two_level.two_diamonds.meaning` | preempts |
| Strong natural 2M | 0 | 0 / 0 | rules | `two_level.two_hearts.meaning`, `two_level.two_spades.meaning` | preempts |
| Weak natural 2D | 13 | 315 / 347 | rules | `two_level.two_diamonds.meaning` | preempts |
| Weak natural 2M | 16 | 346 / 349 | rules | `two_level.two_hearts.meaning`, `two_level.two_spades.meaning` | preempts |
| Wilkosz | 0 | 0 / 0 | field only | `two_level.wilkosz.play` |  |

#### Preempts

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| Namyats | 7 | 83 / 319 | field only | `preempts.namyats.play` |  |

#### Slam

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| 4NT opening | 16 | 297 / 324 | field only | `slam.four_nt_opening.play` |  |
| 5NT pick a slam | 7 | 83 / 319 | field only | `slam.pick_a_slam_5nt.play` |  |
| Blackwood 0123 | 1 | 24 / 24 | rules | `slam.blackwood.standard` | blackwood, weak-two-responses |
| Blackwood 0314 | 3 | 65 / 318 | rules | `slam.blackwood.rkcb_0314` | rkcb-1430, weak-two-responses |
| Blackwood 1430 | 14 | 261 / 8 | rules | `slam.blackwood.rkcb_1430` | rkcb-1430, weak-two-responses |
| Cue bid | 17 | 326 / 326 | rules | `slam.cue_bids.play` | control-bids |
| DEPO | 0 | 0 / 0 | field only | `slam.depo` |  |
| DOPI | 17 | 326 / 326 | field only | `slam.dopi` |  |
| Exclusion | 7 | 83 / 319 | field only | `slam.exclusion_blackwood.play` |  |
| Gerber | 17 | 326 / 326 | rules | `slam.gerber.play` |  |
| Gerber only for NT openings | 3 | 10 / 1 | rules | `slam.gerber.only_over_nt_openings` |  |
| Kickback 0314 | 0 | 0 / 0 | field only | `slam.kickback.rkcb_0314` |  |
| Kickback 1430 | 0 | 0 / 0 | field only | `slam.kickback.rkcb_1430` |  |
| King ask by 5NT | 18 | 350 / 350 | field only | `slam.king_ask.five_nt` |  |
| King ask by available bid | 0 | 0 / 0 | field only | `slam.king_ask.next_step` |  |
| Quantitative 4NT | 18 | 350 / 350 | field only | `slam.quantitative_4nt.play` |  |
| ROPI | 18 | 350 / 350 | field only | `slam.ropi` |  |

#### Doubles

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| Maximal Doubles | 3 | 31 / 2 | field only | `doubles.maximal` |  |
| Responsive double | 14 | 309 / 325 | rules | `doubles.responsive.play` | responsive-doubles |
| Support double redouble | 17 | 326 / 326 | field only | `doubles.support.play`, `doubles.support.rdbl` |  |

#### Competitive

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| (1X)-1Y-(1Z)-2Z natural | 0 | 0 / 0 | field only | `overcalls.responses.natural_in_responders_suit` |  |
| 1X-(Y)-2Z forcing | 15 | 296 / 324 | rules | `competitive.new_suit_after_overcall_forcing` | after-interference, rebids |
| 1X-(1Y)-2Z strong | 1 | 24 / 24 | field only | `competitive.jump_shift_after_overcall` |  |
| 1X-(1Y)-2Z weak | 17 | 326 / 326 | field only | `competitive.jump_shift_after_overcall` |  |
| BROMAD | 6 | 48 / 3 | field only | `competitive.bromad.play` |  |
| Cappelletti | 10 | 270 / 322 | rules | `competitive.defense_vs_strong_nt.convention` | overcalls |
| Fit showing jumps | 7 | 83 / 319 | field only | `competitive.fit_showing_jumps.play` |  |
| Ghestem | 0 | 0 / 0 | field only | `competitive.ghestem.play` |  |
| Jordan Truscott 2NT | 17 | 326 / 326 | field only | `competitive.jordan_2nt.play` |  |
| Landy | 0 | 0 / 0 | rules | `competitive.defense_vs_strong_nt.convention` | overcalls |
| Leaping Michaels | 3 | 31 / 2 | field only | `competitive.leaping_michaels.play` |  |
| Lebensohl after double | 15 | 296 / 324 | field only | `competitive.lebensohl_weak_twos.play` |  |
| Major Direct Jump Cuebid Gambling | 1 | 29 / 2 | field only | `competitive.direct_jump_cuebid_major` |  |
| Major Direct Jump Cuebid Minor | 0 | 0 / 0 | field only | `competitive.direct_jump_cuebid_major` |  |
| Major Direct Jump Cuebid Strong | 0 | 0 / 0 | field only | `competitive.direct_jump_cuebid_major` |  |
| Minor Direct Jump Cuebid Gambling | 1 | 29 / 2 | field only | `competitive.direct_jump_cuebid_minor` |  |
| Minor Direct Jump Cuebid Majors | 0 | 0 / 0 | field only | `competitive.direct_jump_cuebid_minor` |  |
| Minor Direct Jump Cuebid Preempt | 0 | 0 / 0 | field only | `competitive.direct_jump_cuebid_minor` |  |
| Michaels Cuebid | 17 | 326 / 326 | rules | `direct_cuebids.nat_minors_michaels`, `direct_cuebids.nat_majors_michaels` |  |
| Multi-Landy | 7 | 56 / 4 | rules | `competitive.defense_vs_strong_nt.convention` | overcalls |
| Non-Leaping Michaels | 0 | 0 / 0 | field only | `competitive.non_leaping_michaels.play` |  |
| Raptor 1NT | 0 | 0 / 0 | field only | `competitive.raptor_1nt.play` |  |
| Rubensohl after double | 2 | 30 / 2 | field only | `competitive.rubensohl_after_double.play` |  |
| Scrambling 2NT | 0 | 0 / 0 | field only | `competitive.scrambling_2nt.play` |  |
| Snapdragon Double | 0 | 0 / 0 | field only | `competitive.snapdragon.play` |  |
| Unusual 1NT | 17 | 326 / 326 | field only | `competitive.unusual_1nt.play` |  |
| Unusual 2NT | 17 | 326 / 326 | rules | `competitive.unusual_2nt.play` |  |
| Unusual 3NT | 0 | 0 / 0 | field only | `competitive.unusual_3nt.play` |  |
| Unusual 4NT | 17 | 326 / 326 | field only | `competitive.unusual_4nt.play` |  |
| Unusual vs. Unusual | 0 | 0 / 0 | field only | `competitive.unusual_vs_unusual.play` |  |

#### Carding

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| Inverted count signals | 0 | 0 / 0 | field only | `carding.inverted_count` |  |
| Lavinthal from void | 16 | 297 / 324 | field only | `carding.lavinthal.from_void` |  |
| Lavinthal on ace | 16 | 297 / 324 | field only | `carding.lavinthal.on_ace` |  |
| Lavinthal on trump | 0 | 0 / 0 | field only | `carding.lavinthal.on_trump` |  |
| Lavinthal to void | 16 | 297 / 324 | field only | `carding.lavinthal.to_void` |  |
| Smith Echo | 0 | 0 / 0 | field only | `carding.smith_echo` |  |

#### Other

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| Checkback | 2 | 3 / 0 | field only | `other_conventions.checkback.play` |  |
| Fourth suit | 6 | 82 / 319 | field only | `other_conventions.fourth_suit_forcing.one_round` |  |
| Fourth suit game force | 11 | 244 / 7 | field only | `other_conventions.fourth_suit_forcing.game_force` |  |
| New Minor Forcing | 12 | 279 / 323 | rules | `other_conventions.new_minor_forcing.play` | new-minor-forcing, responder-rebids |
| NMF after 2NT rebid | 16 | 297 / 324 | field only | `other_conventions.new_minor_forcing.after_2nt_rebid` |  |
| NMF by passed hand | 0 | 0 / 0 | field only | `other_conventions.new_minor_forcing.by_passed_hand` |  |
| Roudi | 1 | 14 / 1 | field only | `other_conventions.roudi.play` |  |
| Two way game tries | 7 | 83 / 319 | field only | `other_conventions.two_way_game_tries.play` |  |
| Two Way New Minor Forcing | 2 | 30 / 2 | field only | `other_conventions.two_way_nmf` |  |
| TWNMF by passed hand | 0 | 0 / 0 | field only | `other_conventions.two_way_nmf_by_passed_hand` |  |

#### Not mapped

| .bbsa key | cards on (of 18) | scenarios, NS / EW | engine | card field | modules reading it |
|---|---:|---:|---|---|---|
| 1NT opening natural | 0 | 0 / 0 | **passthrough** |  |  |
| 1NT opening NT style | 18 | 350 / 350 | **passthrough** |  |  |
| 5431 after 1NT | 1 | 13 / 1 | **passthrough** |  |  |
| Blackwood without K and Q | 0 | 0 / 0 | **passthrough** |  |  |
| Collante | 1 | 14 / 1 | **passthrough** |  |  |
| Crosswood 0123 | 0 | 0 / 0 | **passthrough** |  |  |
| Crosswood 0314 | 2 | 30 / 2 | **passthrough** |  |  |
| Crosswood 1430 | 0 | 0 / 0 | **passthrough** |  |  |
| Extended Stayman | 0 | 0 / 0 | **passthrough** |  |  |
| Fitted 2NT | 0 | 0 / 0 | **passthrough** |  |  |
| Imposible 2S | 9 | 242 / 7 | **passthrough** |  |  |
| Jordan Truscott 2NT defence | 1 | 29 / 2 | **passthrough** |  |  |
| Kickback 0123 | 0 | 0 / 0 | **passthrough** |  |  |
| King ask by 5NT inviting | 0 | 0 / 0 | **passthrough** |  |  |
| Lebensohl after 1m | 15 | 296 / 324 | **passthrough** |  |  |
| Mark on queen | 16 | 297 / 324 | **passthrough** |  |  |
| Mark on king | 16 | 297 / 324 | **passthrough** |  |  |
| Natural 3N entering style | 1 | 29 / 2 | **passthrough** |  |  |
| Niemeijer | 0 | 0 / 0 | **passthrough** |  |  |
| Rodrigue | 1 | 14 / 1 | **passthrough** |  |  |
| Rubensohl after 1m | 2 | 30 / 2 | **passthrough** |  |  |
| Shape Bergen structure | 12 | 268 / 31 | **passthrough** |  |  |
| Strength Lawrence structure | 6 | 82 / 319 | **passthrough** |  |  |
| Support 1NT | 6 | 48 / 3 | **passthrough** |  |  |
| Surplus pass | 0 | 0 / 0 | **passthrough** |  |  |
| Two suit takeout double | 17 | 326 / 326 | **passthrough** |  |  |
| Opponent type | 0 | 0 / 0 | **passthrough** |  |  |
| Transfers if RHO passes | 0 | 0 / 0 | **passthrough** |  |  |

## Appendix B: every scenario

"NS calls = BBA" and "contract = BBA" come from `rbb compare` over the full corpus at `fc9dd81`. "vs BBA" is IMPs against BBA with par as the yardstick; it is 0 where the corpus has no double-dummy data or the scenario is not scored. Boards 0 means no BBA corpus exists.

| scenario | section | NS card | EW card | BBA works | boards | NS calls = BBA | contract = BBA | vs BBA (IMPs) |
|---|---|---|---|---|---:|---:|---:|---:|
| Basic_Major | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 73% | 48% | -173 |
| Basic_Minor | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 73% | 40% | -136 |
| Basic_NT | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 96% | 89% | 22 |
| Basic_Openers_Rebid | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 86% | 57% | 14 |
| Basic_Overcall | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 75% | 23% | -164 |
| Basic_Responses | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 81% | 57% | -12 |
| Basic_Takeout_Double | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 75% | 24% | -251 |
| Basic_Weak_2 | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 74% | 47% | -288 |
| Basic_What_To_Open | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 77% | 48% | -188 |
| GIB_1N_Basic | Beginners Bidding | 21GF-GIB | 21GF-GIB | yes | 500 | 83% | 66% | -141 |
| Open_and_Rebid | Beginners Bidding | Basic-Bridge | Basic-Bridge | yes | 500 | 74% | 29% | -138 |
| 5431_After_NT | Beyond BBO Robots | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 51% | 67% | -309 |
| Any_5422_with_15-16 | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 67% | 32% | -124 |
| Any_5422_with_15-17 | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 67% | 28% | -194 |
| Gazzilli | Beyond BBO Robots | 21GF-Gazzilli | 21GF-GIB | yes | 500 | 57% | 42% | -211 |
| Kokish_Relay | Beyond BBO Robots | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 51% | 34% | -1115 |
| Losing_Trick_Count | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 77% | 34% | -294 |
| Mini-Maxi_Roman_2D | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | **no** | 500 | 68% | 27% | 0 |
| Namyats | Beyond BBO Robots | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 69% | 30% | -446 |
| Ned_2S | Beyond BBO Robots | 21GF-MSS | 21GF-GIB | yes | 500 | 60% | 61% | 30 |
| Ned_3-Level_Resp_to_1N | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 58% | 23% | -1585 |
| Ned_Weak_Two | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 72% | 37% | -80 |
| Ned_Weak_Two_Leveled | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 77% | 27% | -351 |
| Negative_Free_Bid | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 74% | 23% | -268 |
| Reverse_Flannery | Beyond BBO Robots | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 49% | 15% | -233 |
| Spear | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 80% | 27% | -170 |
| Transfer_Walsh | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 69% | 30% | -239 |
| Vics_2C_Relay | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 54% | 44% | -662 |
| Vics_Bal_Resp_to_1m | Beyond BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 63% | 33% | -355 |
| WB5_Rodrigue | Beyond BBO Robots | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 59% | 19% | -232 |
| WB5_Roudi | Beyond BBO Robots | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 69% | 41% | -89 |
| Robot_Free_Bid | Bidding with BBO Robots | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 68% | 25% | -432 |
| Choice_Of_Finesses | Finesses | Basic-Bridge | Basic-Bridge | yes | 500 | 95% | 96% | 27 |
| Finesse_Simple | Finesses | Basic-Bridge | Basic-Bridge | yes | 500 | 78% | 63% | 57 |
| Rabbis_Rule | Finesses | Basic-Bridge | Basic-Bridge | yes | 500 | 69% | 64% | 379 |
| To_Finesse_Or_Not_To_Finesse | Finesses | Basic-Bridge | Basic-Bridge | yes | 500 | 55% | 29% | -1768 |
| Two_Way_Finesse | Finesses | Basic-Bridge | Basic-Bridge | yes | 500 | 78% | 62% | -97 |
| 1M-3N_Picture_Bid | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | **no** | 500 | 63% | 42% | 0 |
| Benjamin_2D | Game Forcing Sequences | 21GF-PolishTwoSuiters | 21GF-GIB | yes | 500 | 36% | 14% | -983 |
| Game_Forcing_2C | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 57% | 22% | -872 |
| Gerber_By_Opener | Game Forcing Sequences | 21GF-GIB | 21GF-GIB | **no** | 500 | 55% | 28% | 0 |
| Gerber_By_Responder | Game Forcing Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 53% | 32% | -1599 |
| Jacoby_2N | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 66% | 66% | 33 |
| Jacoby_2N_4x_void | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 66% | 68% | -57 |
| Jacoby_2N_4x_void_Leveled | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 59% | -129 |
| Jacoby_2N_Leveled | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 64% | 60% | -16 |
| Last_Train_GT2 | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 67% | 26% | 125 |
| Last_Train_Game_Try | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 59% | 62% | 334 |
| Serious | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 45% | 36% | -1364 |
| Six_Key_RKC | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | **no** | 0 |  |  |  |
| Soloway_Jump_Shift | Game Forcing Sequences | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 35% | 13% | -1952 |
| Soloway_Jump_Shift_Type-1 | Game Forcing Sequences | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 28% | 12% | -2133 |
| Soloway_Jump_Shift_Type-2 | Game Forcing Sequences | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 31% | 10% | -2299 |
| Soloway_Jump_Shift_Type-3 | Game Forcing Sequences | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 39% | 8% | -2764 |
| Soloway_Jump_Shift_Type-4 | Game Forcing Sequences | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 46% | 27% | -1599 |
| Splinters | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 60% | 64% | -33 |
| Two_Over_One | Game Forcing Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 50% | 28% | -825 |
| Gavin_3_Under_Invitational_Jump | Gavin's Major Suit Response Structure | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 55% | 30% | -596 |
| Gavin_4-Card_Limit_Raise | Gavin's Major Suit Response Structure | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 69% | 46% | -324 |
| Gavin_Mixed_Raise | Gavin's Major Suit Response Structure | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 77% | 27% | -231 |
| Gavin_Passed_Hand_Response_Structure | Gavin's Major Suit Response Structure | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 64% | 55% | -36 |
| Gavin_Semi-Constructive_Raise | Gavin's Major Suit Response Structure | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 80% | 35% | 68 |
| Gavin_Semi-Forcing_NT_with_Fit | Gavin's Major Suit Response Structure | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 78% | 26% | -345 |
| Gavin_Strong_Splinter | Gavin's Major Suit Response Structure | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 56% | 66% | -23 |
| Gavin_Weak_Splinter | Gavin's Major Suit Response Structure | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 78% | 72% | 82 |
| Gavin_Transfers_after_2N_Rebid_Bal | Gavin's Transfer After 2N Rebid | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 77% | 63% | -109 |
| Gavin_Transfers_after_2N_Rebid_Unb | Gavin's Transfer After 2N Rebid | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 64% | 46% | -141 |
| Gavin_Transfers_after_2N_Rebid_Weak | Gavin's Transfer After 2N Rebid | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 70% | 40% | 127 |
| Minor_Suit_Opener_Balanced_Response | Josh Donn's preferred Minor Suit Response Structure | 21GF-NoInvertedMinor | 21GF-GIB | yes | 500 | 65% | 52% | -389 |
| Minor_Suit_Opener_Inv_Raise | Josh Donn's preferred Minor Suit Response Structure | 21GF-NoInvertedMinor | 21GF-GIB | yes | 500 | 69% | 29% | 35 |
| Minor_Suit_Opener_Mixed_Raise | Josh Donn's preferred Minor Suit Response Structure | 21GF-NoInvertedMinor | 21GF-GIB | yes | 500 | 59% | 25% | 184 |
| Minor_Suit_Opener_Resp_Structure | Josh Donn's preferred Minor Suit Response Structure | 21GF-NoInvertedMinor | 21GF-GIB | yes | 500 | 68% | 44% | -31 |
| After_1M_2M | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 76% | 49% | 410 |
| BART | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 57% | 53% | 197 |
| Bergen_Raises | Major Suit Sequences | 21GF-GIB-Bergen | 21GF-GIB | yes | 500 | 61% | 42% | -397 |
| Drury | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 53% | 59% | -100 |
| Exclusion_After_1M | Major Suit Sequences | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 46% | 56% | -1379 |
| Flannery | Major Suit Sequences | 21GF-Flannery | 21GF-GIB | yes | 500 | 28% | 14% | -11 |
| Forcing_NT | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 64% | 32% | -447 |
| GIB_1M-P-Resp | Major Suit Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 90% | 73% | 8 |
| Gavin_3-Card_Limit_Raise | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 60% | 39% | -45 |
| Grand_Slam_Force | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 49% | 24% | -3095 |
| Help_Suit_Game_Try | Major Suit Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 74% | 62% | 266 |
| Impossible_2S | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 44% | 45% | 449 |
| Major_Opener | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 69% | 34% | -396 |
| Major_Suit_Fit | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 70% | 40% | -83 |
| Preemptive_Major_Raise | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 72% | 29% | -454 |
| Slam_After_Major_Fit | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 50% | 35% | -877 |
| Two-Way_Game_Try | Major Suit Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 62% | 55% | 332 |
| Xfer_after_1M_X | Major Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 27% | -52 |
| 1m-1N | Minor Suit Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 84% | 47% | -42 |
| 1m-1_2_or_3_NT | Minor Suit Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 78% | 42% | -63 |
| 1m-2N | Minor Suit Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 66% | 39% | -350 |
| 1m-2x | Minor Suit Sequences | 21GF-GIB | 21GF-GIB | **no** | 0 |  |  |  |
| 1m-3N | Minor Suit Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 79% | 46% | -164 |
| 1m_1x_2m | Minor Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 78% | 45% | -60 |
| Fit_Showing_Jumps | Minor Suit Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 68% | 34% | -175 |
| Inverted_Minors | Minor Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 28% | -549 |
| Minor_Game_Or_Slam | Minor Suit Sequences | 21GF-MSTandMSS | 21GF-GIB | yes | 500 | 47% | 17% | -1846 |
| Minor_Suit_Opener | Minor Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 72% | 30% | -254 |
| Splinters_after_Minor | Minor Suit Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 53% | 11% | -1242 |
| 1C_WalshStyle | Minor/Major Sequences | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 68% | 45% | -344 |
| 3N_Rebid_by_Opener | Minor/Major Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 58% | 39% | -268 |
| 3_Under_Invitational_Jump | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 55% | 28% | -374 |
| After_2_Passes | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 84% | 31% | -186 |
| After_3_Passes | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 74% | 30% | -426 |
| DOPI_ROPI | Minor/Major Sequences | 21GF-DEFAULT | 21GF-DEFAULT | yes | 472 | 55% | 43% | -879 |
| Fourth_Bid_Inviting | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 70% | 39% | 179 |
| Fourth_Suit_Forcing | Minor/Major Sequences | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 64% | 34% | -44 |
| GIB_1C-P-Resp | Minor/Major Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 68% | 46% | -203 |
| Goulash | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 68% | 12% | -22 |
| Grand_Slam_Force_2 | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 51% | 45% | -2079 |
| Misfit | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 66% | 37% | -265 |
| Misfit6-5 | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 46% | 10% | -1449 |
| Misfit_06-10 | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 72% | 38% | -60 |
| Misfit_11-12 | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 62% | 32% | -134 |
| Misfit_13-Plus | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 55% | 31% | -846 |
| New_Minor_Forcing | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 76% | 39% | 114 |
| Notrump_18-19 | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 67% | 37% | -229 |
| Open_In_Fourth_13 | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 84% | 43% | -270 |
| Open_In_Fourth_14 | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 73% | 31% | -384 |
| Open_In_Fourth_15 | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 79% | 33% | -151 |
| Open_In_Fourth_16 | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 78% | 31% | -125 |
| Open_In_Fourth_Seat | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 77% | 34% | -199 |
| Preemptive_Raise | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 71% | 24% | -448 |
| Reverse_After_Two_Over_One | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 44% | 29% | -993 |
| Reverse_By_Opener | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 27% | -333 |
| Reverse_By_Responder | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 61% | 48% | -407 |
| Spiral_Raises_Weinstein | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 73% | 48% | 229 |
| Spiral_Raises_Wolpert | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 71% | 49% | 129 |
| Splinters_By_Opener | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 56% | 39% | -1107 |
| Transfer_Advances | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | **no** | 500 | 68% | 28% | 0 |
| Two-Way_New_Minor_Forcing_aka_xyNT | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 64% | 46% | -305 |
| Weak_Jump_Shift | Minor/Major Sequences | 21GF-WJS-MSS | 21GF-GIB | yes | 500 | 48% | 9% | -507 |
| XYZ | Minor/Major Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 73% | 33% | 7 |
| Bust_Over_Strong_2C | (not in layout) | 21GF-DEFAULT | 21GF-GIB | yes | 5 | 48% | 0% | -8 |
| Found_Endplay | (not in layout) | Basic-Bridge | Basic-Bridge | yes | 500 | 59% | 46% | 0 |
| Found_Rabbis_Rule | (not in layout) | Basic-Bridge | Basic-Bridge | yes | 156 | 67% | 68% | 0 |
| Lebensohl2 | (not in layout) | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 76% | 47% | 72 |
| Mathe | (not in layout) | 21GF-DEFAULT | Precision | yes | 500 | 86% | 18% | -271 |
| Reversed_Splinters_By_Opener | (not in layout) | 21GF-DEFAULT | 21GF-GIB | **no** | 0 |  |  |  |
| TEST | (not in layout) | 21GF-Puppet | 21GF-SPECIALS2 | yes | 500 | 38% | 84% | 25 |
| Hold_Up_3N | Notrump Play | Basic-Bridge | Basic-Bridge | yes | 500 | 94% | 89% | 89 |
| Play_Top_Tricks_NT | Notrump Play | Basic-Bridge | Basic-Bridge | yes | 500 | 87% | 82% | 28 |
| Suit_Promotion | Notrump Play | Basic-Bridge | Basic-Bridge | yes | 500 | 83% | 71% | -60 |
| 1N | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 80% | 55% | 41 |
| 1N_5M_and_6m | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 69% | 44% | -516 |
| 1N_Balanced_Raise | Notrump Sequences | Basic-Bridge | Basic-Bridge | yes | 500 | 87% | 69% | 6 |
| 1N_with_Singleton | Notrump Sequences | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 66% | 25% | -233 |
| 2N | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 64% | 51% | -292 |
| 2N_and_1_Minor | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 58% | 35% | -712 |
| 2N_and_3C_Response | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 71% | 67% | -119 |
| 2N_and_Balanced | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 66% | 68% | -258 |
| 2N_and_MSS | Notrump Sequences | 21GF-MSS | 21GF-GIB | yes | 500 | 48% | 36% | -239 |
| 2N_and_Transfers | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 51% | 28% | -942 |
| 3N | Notrump Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 39% | 10% | -1267 |
| DropDead_Crawling | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 79% | 54% | 87 |
| Exclusion_After_Sta_Jac | Notrump Sequences | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 61% | 49% | -612 |
| GIB_1N | Notrump Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 80% | 59% | 49 |
| GIB_1N-P-2C | Notrump Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 78% | 63% | 145 |
| GIB_1N5422 | Notrump Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 64% | 17% | -210 |
| Gerber | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 43% | 86% | 25 |
| Grand_Slam_Invite | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 51% | 23% | -3275 |
| Jacoby_Super-Accept | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 74% | 42% | -108 |
| Jacoby_Transfer | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 78% | 64% | 27 |
| MST_or_MSS | Notrump Sequences | 21GF-MSTandMSS | 21GF-GIB | yes | 500 | 29% | 11% | -2299 |
| Minor_Suit_Stayman | Notrump Sequences | 21GF-MSTandMSS | 21GF-GIB | yes | 500 | 39% | 61% | -190 |
| Minor_Suit_Transfer | Notrump Sequences | 21GF-MSTandMSS | 21GF-GIB | yes | 500 | 59% | 0% | -55 |
| NT_Ladder | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 70% | 42% | -360 |
| NT_Splinter | Notrump Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 58% | 79% | -232 |
| Puppet_Stayman_1N | Notrump Sequences | 21GF-Puppet | 21GF-GIB | yes | 500 | 31% | 67% | -341 |
| Puppet_Stayman_2N | Notrump Sequences | 21GF-MSTandMSS | 21GF-GIB | yes | 500 | 42% | 64% | -364 |
| Rule_of_16 | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 81% | 54% | 345 |
| Rule_of_16-15 | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 86% | 63% | 105 |
| Rule_of_16-16 | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 84% | 58% | 126 |
| Rule_of_16-17 | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 84% | 56% | -32 |
| Rule_of_16-18 | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 84% | 58% | 213 |
| Runout_after_1N_X | Notrump Sequences | 21GF-GIB | 21GF-DEFAULT | yes | 500 | 81% | 17% | 46 |
| Size_Asking_Minor_Suit_Stayman | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | **no** | 500 | 75% | 53% | 0 |
| Slam_after_NT | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 69% | 56% | -144 |
| Slam_after_Stayman | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 73% | 51% | -107 |
| Slam_after_Stayman_or_Jacoby_w30plus | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 67% | 37% | -162 |
| Slam_after_Stayman_or_Jacoby_w31plus | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 68% | 48% | -293 |
| Slam_after_Transfer | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 68% | 46% | -598 |
| Smolen | Notrump Sequences | 21GF-GIB | 21GF-GIB | yes | 500 | 59% | 32% | -608 |
| Smolen_Invitational | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 73% | 28% | -313 |
| Smolen_after_2N | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 52% | 25% | -647 |
| Stayman | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 85% | 72% | 208 |
| Texas_Transfer | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 78% | 63% | -86 |
| Texas_or_Jacoby | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 80% | 56% | 113 |
| Two_NT_in_Fourth_Seat | Notrump Sequences | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 81% | 63% | -219 |
| 3N_over_LHO_3x | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 70% | 34% | -701 |
| 3N_over_RHO_3x | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 62% | 33% | -392 |
| GSF_after_Preempt | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 48% | 2% | -5651 |
| Gambling_3N | Preempts | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 30% | 5% | -414 |
| Lars_Multi_2D | Preempts | 21GF-Multi | 21GF-GIB | **no** | 0 |  |  |  |
| McCabe_After_Weak_2 | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 76% | 26% | -439 |
| Muiderberg_Two_Bids | Preempts | 21GF-PolishTwoSuiters | 21GF-GIB | yes | 500 | 70% | 22% | -132 |
| Multi_2D | Preempts | 21GF-Multi | 21GF-GIB | yes | 500 | 45% | 22% | -534 |
| Ogust | Preempts | 21GF-Ogust | 21GF-GIB | yes | 500 | 71% | 38% | -208 |
| Opps_Gambling_3N | Preempts | 21GF-DEFAULT | 21GF-SPECIALS | yes | 500 | 82% | 6% | -307 |
| Opps_Multi_2D | Preempts | 21GF-DEFAULT | 21GF-Multi | yes | 500 | 75% | 25% | -304 |
| Opps_Preempt | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 75% | 29% | -388 |
| Opps_Preempt_4M | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 76% | 51% | -171 |
| Opps_Weak_Two | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 72% | 34% | -359 |
| Preempt_Keycard | Preempts | 21GF-DEFAULT | 21GF-GIB | **no** | 500 | 44% | 21% | 0 |
| Preempts | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 76% | 29% | -261 |
| Slam_after_Preempt | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 56% | 35% | -913 |
| Weak_2_Bids | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 84% | 34% | -541 |
| Weak_2_Bids_Lax | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 78% | 37% | -346 |
| Weak_2_Bids_Lax_Leveled | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 78% | 34% | -288 |
| Weak_2_Bids_Leveled | Preempts | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 77% | 39% | -308 |
| SCS_1C_3-Suit_Resp | Strong Club | Precision | 21GF-GIB | yes | 500 | 37% | 30% | -703 |
| SCS_1C_3-Suit_Resp_5-7 | Strong Club | Precision | 21GF-GIB | yes | 500 | 46% | 26% | -161 |
| SCS_1C_54_Resp | Strong Club | Precision | 21GF-GIB | yes | 500 | 40% | 28% | -569 |
| SCS_1C_55_Resp | Strong Club | Precision | 21GF-GIB | yes | 500 | 38% | 26% | -956 |
| SCS_1C_any_0-4_Resp | Strong Club | Precision | 21GF-GIB | yes | 500 | 53% | 13% | -217 |
| SCS_1C_any_5-7_Resp | Strong Club | Precision | 21GF-GIB | yes | 500 | 44% | 26% | -83 |
| SCS_1C_any_8plus_Resp | Strong Club | Precision | 21GF-GIB | yes | 500 | 40% | 34% | -496 |
| SCS_Impossible_Negative | Strong Club | Precision | 21GF-GIB | yes | 500 | 25% | 24% | -1020 |
| SCS_Major_Open_2-Suit_Resp | Strong Club | Precision | 21GF-GIB | yes | 500 | 63% | 31% | -469 |
| SCS_Major_with_2nd_Suit | Strong Club | Precision | 21GF-GIB | yes | 500 | 60% | 41% | -220 |
| SCS_Two_Clubs | Strong Club | Precision | 21GF-GIB | yes | 500 | 39% | 17% | -139 |
| Weak_NT_09-12 | Strong Club | Precision | 21GF-GIB | **no** | 500 | 73% | 33% | 0 |
| Weak_NT_09-15 | Strong Club | Precision | 21GF-GIB | **no** | 500 | 74% | 35% | 0 |
| Weak_NT_10-12 | Strong Club | Precision | 21GF-GIB | **no** | 500 | 72% | 32% | 0 |
| Weak_NT_10-13 | Strong Club | Precision | 21GF-GIB | **no** | 500 | 72% | 34% | 0 |
| Weak_NT_11-16 | Strong Club | Precision | 21GF-GIB | **no** | 500 | 68% | 36% | 0 |
| Weak_NT_13-15 | Strong Club | Precision | 21GF-GIB | yes | 500 | 81% | 50% | 256 |
| Weak_NT_14-15 | Strong Club | Precision-14-16 | 21GF-GIB | yes | 500 | 73% | 45% | 95 |
| SCS16_1C_1SuitedOpnr | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 46% | 10% | -239 |
| SCS16_1C_1SuitedResp | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 28% | 14% | -1422 |
| SCS16_1C_3-Suit_Resp | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 27% | 22% | -913 |
| SCS16_1C_3-Suit_Resp_5-7 | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 37% | 22% | -505 |
| SCS16_1C_54_Resp | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 28% | 25% | -829 |
| SCS16_1C_55_Resp | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 28% | 14% | -757 |
| SCS16_1C_any_0-4_Resp | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 45% | 13% | -312 |
| SCS16_1C_any_5-7_Resp | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 35% | 22% | -245 |
| SCS16_1C_any_8plus_Resp | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 29% | 23% | -932 |
| SCS16_1or2C | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 31% | 18% | -469 |
| SCS16_Major_Open_2-Suit_Resp | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 63% | 33% | -330 |
| SCS16_Major_with_2nd_Suit | Strong Club 16+ | Precision | 21GF-GIB | yes | 500 | 62% | 38% | -453 |
| Endplay_3rd_Round_Strip | Suit Contract Play | Basic-Bridge | Basic-Bridge | yes | 500 | 77% | 66% | 151 |
| Play_Top_Tricks | Suit Contract Play | Basic-Bridge | Basic-Bridge | yes | 500 | 79% | 58% | -35 |
| Side_Suit_Ruff_Before_Trump | Suit Contract Play | Basic-Bridge | Basic-Bridge | yes | 500 | 76% | 81% | 65 |
| After_1x_1N | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 80% | 29% | -30 |
| After_Opp_Overcalls | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 72% | 26% | -200 |
| Bergen_Thrump_X_after_Preempt | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | **no** | 0 |  |  |  |
| Dealing_with_Overcalls_Strong | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 57% | 22% | -524 |
| Dealing_with_Overcalls_Weak | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 60% | 25% | -414 |
| Double_double | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 68% | 21% | -404 |
| Equal_Level_Conversion | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 70% | 23% | -316 |
| Exit_Transfers | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-DEFAULT | yes | 500 | 81% | 7% | -48 |
| Fit_Jumps_after_1M_Double | They Compete in Our Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 47% | 13% | -1907 |
| Forcing_Pass | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 53% | 24% | -387 |
| GIB_1N-P-2C-BID | They Compete in Our Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 70% | 31% | 233 |
| GIB_Overcall_with_4 | They Compete in Our Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 78% | 34% | -163 |
| Going_for_Blood | They Compete in Our Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 65% | 19% | -485 |
| Good_Bad_2N | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | **no** | 500 | 65% | 18% | 0 |
| Jordan_2N | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 51% | 33% | -716 |
| Lebensohl | They Compete in Our Auctions | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 71% | 62% | 271 |
| Maximal_Double | They Compete in Our Auctions | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 72% | 26% | -176 |
| Michaels_after_1m | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 11% | -598 |
| Mitchell_Stayman | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | **no** | 500 | 81% | 34% | 0 |
| Negative_Double | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 67% | 34% | -361 |
| Opps_2-Suited_Overcalls | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 16% | -363 |
| Opps_Bid_Over_GF_2C | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 49% | 20% | -1171 |
| Opps_Double_1_NT | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-DEFAULT | yes | 500 | 82% | 5% | -125 |
| Opps_Double_Jacoby | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 73% | 36% | 35 |
| Opps_Double_Stayman | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 59% | 60% | 145 |
| Opps_Michaels_Cuebid | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 69% | 11% | -429 |
| Opps_Michaels_and_Unusual | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 62% | 19% | -307 |
| Opps_Overcall_1NT | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 88% | 37% | 91 |
| Opps_Overcall_Stayman_or_Jacoby | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 79% | 23% | -68 |
| Opps_Preemptive_Overcall | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 62% | 18% | -294 |
| Opps_Takeout_X | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 70% | 20% | -167 |
| Opps_Takeout_X_We_XX | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 68% | 20% | -489 |
| Opps_Weak_Jump_Overcall | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 21% | -290 |
| Rosenkranz_Double | They Compete in Our Auctions | 21GF-GIB | 21GF-GIB | **no** | 500 | 70% | 21% | 0 |
| Support_Double | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 64% | 17% | -265 |
| Tislevoll_after_Opps_Preempt | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-DEFAULT | **no** | 500 | 62% | 35% | 0 |
| Transfers_after_1M_X | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 71% | 22% | -182 |
| Trap_Pass | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 32% | 33 |
| Trap_Pass_Maybe | They Compete in Our Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 78% | 27% | 40 |
| WB5_Collante | They Compete in Our Auctions | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 64% | 23% | -641 |
| After_Partner_Overcalls | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 59% | 16% | -708 |
| After_Partner_Takeout_Double | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 63% | 21% | -263 |
| Balancing | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 67% | 26% | -517 |
| Better_Minor_Lebensohl | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 55% | 32% | -142 |
| CRASH | We Compete in Their Auctions | 21GF-DEFAULT | Precision | **no** | 0 |  |  |  |
| Cappelletti | We Compete in Their Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 66% | 29% | 62 |
| Cappelletti_in_4th | We Compete in Their Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 67% | 28% | 36 |
| Competitive_Doubles | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 81% | 36% | 162 |
| DONT | We Compete in Their Auctions | 21GF-GIB | 21GF-GIB | **no** | 500 | 57% | 16% | 0 |
| Double_Showing_2_Suits | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 70% | 26% | -88 |
| Double_by_Advancer | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 62% | 24% | -123 |
| GIB_Sandwich_NT_BPH | We Compete in Their Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 78% | 28% | -187 |
| Game_Overcalls | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 63% | 6% | -476 |
| Jump_Cuebid_Strong | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 6% | -1053 |
| Jump_Cuebid_Weak | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 72% | 18% | 46 |
| Jump_Overcalls | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 84% | 21% | -435 |
| Lead_Directing_Double | We Compete in Their Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 90% | 58% | -121 |
| Leaping_Michaels | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 40% | 11% | -1191 |
| Lebensohl_vs_Opps_W2_Bal | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 66% | 35% | 69 |
| Lebensohl_vs_Opps_W2_Bal_or_Dir | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 61% | 35% | -88 |
| Lebensohl_vs_Opps_W2_Dir | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 58% | 30% | -295 |
| Maximal_After_Overcall | We Compete in Their Auctions | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 72% | 18% | -159 |
| McCabe_after_WJO | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 84% | 22% | -219 |
| Meckwell | We Compete in Their Auctions | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 62% | 26% | 32 |
| Michaels_Cuebid | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 57% | 17% | -410 |
| Michaels_and_Unusual | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 59% | 19% | -204 |
| Mixed_Raise_In_Comp | We Compete in Their Auctions | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 63% | 25% | -127 |
| Multi_Landy | We Compete in Their Auctions | 21GF-SPECIALS2 | 21GF-GIB | yes | 500 | 73% | 41% | 155 |
| Non_Leaping_Michaels_After_2-Bid | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 66% | 23% | -555 |
| Non_Leaping_Michaels_After_3-Bid | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 57% | 22% | -448 |
| OBAR_BIDS | We Compete in Their Auctions | 21GF-NoInvertedMinor | 21GF-GIB | yes | 500 | 91% | 48% | -25 |
| OBAR_BIDS11 | We Compete in Their Auctions | 21GF-NoInvertedMinor | 21GF-GIB | yes | 500 | 80% | 32% | 36 |
| Opp_Redoubles | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 92% | 23% | -531 |
| Opps_Open_1N_15-17 | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 91% | 55% | 45 |
| Power_Double_Balanced | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 67% | 17% | -333 |
| Power_Double_Unbalanced | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 16% | -652 |
| Preempt_X_XX | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 81% | 31% | -834 |
| Responsive_Double | We Compete in Their Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 80% | 31% | -57 |
| Responsive_Double_after_Overcall | We Compete in Their Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 73% | 32% | -64 |
| Rule_of_2 | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 76% | 46% | 81 |
| Scrambling_2NT | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 87% | 55% | 56 |
| Snapdragon_Double | We Compete in Their Auctions | 21GF-SPECIALS | 21GF-GIB | yes | 500 | 79% | 22% | -398 |
| Suction | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 62% | 22% | -111 |
| SupportX_by_Advancer | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 21% | -119 |
| Takeout_Double | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 72% | 25% | -260 |
| Too_Strong_for_Overcall | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 14% | -618 |
| Too_Strong_for_Overcall4th | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 60% | 12% | -374 |
| Trap_Pass_Opener | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 64% | 24% | -95 |
| Trap_Pass_Opener_Maybe | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 66% | 22% | -366 |
| Two-Suited_Overcalls | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 65% | 20% | -48 |
| Unusual_2N | We Compete in Their Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 49% | 9% | -280 |
| W2_X_XX | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 80% | 10% | -1115 |
| We_Overcall | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 73% | 21% | -416 |
| We_Overcall_1N | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 74% | 40% | 150 |
| We_Overcall_1N_then_Gerber | We Compete in Their Auctions | 21GF-SPECIALS | 21GF-GIB | **no** | 0 |  |  |  |
| We_Overcall_NT_then_Jacoby | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 74% | 28% | -74 |
| We_Overcall_NT_then_MSS | We Compete in Their Auctions | 21GF-MSTandMSS | 21GF-GIB | yes | 500 | 50% | 36% | 282 |
| We_Overcall_NT_then_MST | We Compete in Their Auctions | 21GF-MSTandMSS | 21GF-GIB | yes | 500 | 55% | 55% | 268 |
| We_Overcall_NT_then_Smolen | We Compete in Their Auctions | 21GF-GIB | 21GF-GIB | yes | 500 | 58% | 36% | -45 |
| We_Overcall_NT_then_Stayman | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 76% | 62% | 105 |
| We_Overcall_NT_then_Texas | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 69% | 26% | -456 |
| We_Overcall_Their_2C | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 82% | 14% | -780 |
| We_X_Opps_Preempt | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 68% | 33% | -621 |
| We_X_Opps_Weak_2 | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | yes | 500 | 54% | 28% | -22 |
| Western_Cue_Bid | We Compete in Their Auctions | 21GF-DEFAULT | 21GF-GIB | **no** | 500 | 63% | 30% | 0 |

## Appendix C: BBA's alerts in the corpus (top 150)

The table counts BBA's `[Note]` texts across `bba/*.pbn`, with suits and counts normalised: "x" stands for a suit, "N" for a number.

- **times:** how often the note occurs.
- **scenarios:** how many scenarios it occurs in.
- **boards:** the boards whose BBA auction carries it.
- **we first differ at it:** the boards where our replay first departs from BBA at a call BBA alerted with it.
- **vs BBA there:** our net IMPs on those boards, with par as the yardstick.

Answers to asks ("A=1/5 or 4/5", "K=0") are listed too; we rarely differ at them, because the difference comes earlier.

| BBA's note | times | scenarios | boards | we first differ at it | vs BBA there (IMPs) |
|---|---:|---:|---:|---:|---:|
| Cue bid | 13843 | 281 | 8103 | 1114 | -986 |
| Blackwood 1430 | 13058 | 197 | 13058 | 1148 | -6932 |
| Stayman | 12065 | 166 | 12065 | 1715 | 138 |
| artificial | 11886 | 186 | 10718 | 1597 | -2258 |
| N HCP | 8149 | 29 | 8149 | 5698 | -7106 |
| A=2/5 or 5/5 | 6489 | 249 | 6489 | 27 | -89 |
| limit raise or better in x | 5390 | 257 | 5374 | 828 | -1299 |
| A=1/5 or 4/5 | 5228 | 252 | 5228 | 9 | 0 |
| Cappelletti | 4612 | 74 | 4612 | 4409 | 462 |
| A=0/5 or 3/5 | 4407 | 239 | 4407 | 3 | 7 |
| Jacoby 2NT | 4186 | 80 | 4186 | 210 | -78 |
| Michaels Cuebid | 3790 | 146 | 3790 | 3559 | -3452 |
| strong | 3544 | 80 | 3544 | 603 | -1844 |
| forcing 1NT | 3420 | 77 | 3420 | 2470 | -212 |
| Blackwood 0314 | 2891 | 112 | 2891 | 408 | -346 |
| surplus | 2751 | 195 | 2732 | 393 | -914 |
| Splinter | 2697 | 183 | 2697 | 1822 | -1738 |
| Quantitative 4NT | 2130 | 141 | 2130 | 569 | -938 |
| Unusual 2NT | 2102 | 158 | 2102 | 1875 | -772 |
| x queen and x king | 2096 | 178 | 2096 | 0 | 0 |
| automat | 2070 | 91 | 2070 | 165 | -5 |
| Inverted minors | 1948 | 71 | 1948 | 24 | -115 |
| 1X-(Y)-2Z forcing | 1809 | 86 | 1809 | 457 | -474 |
| Texas | 1763 | 90 | 1763 | 774 | -1549 |
| shortness  x | 1667 | 47 | 1667 | 182 | -393 |
| SMOLEN | 1592 | 28 | 1592 | 1409 | -1486 |
| Soloway Jump Shifts | 1514 | 8 | 1514 | 1497 | -7103 |
| Gerber | 1403 | 56 | 1403 | 892 | -900 |
| King ask by 5NT | 1387 | 129 | 1387 | 17 | -126 |
| Inviting Jump Shifts | 1347 | 71 | 1347 | 1128 | -534 |
| waiting | 1303 | 45 | 1303 | 8 | -13 |
| Support double redouble | 1181 | 93 | 1181 | 65 | 2 |
| Exclusion | 1163 | 92 | 1163 | 182 | -707 |
| N HCP if fit or 6+; 12+ F | 1157 | 33 | 1157 | 426 | -398 |
| Lebensohl after double | 1138 | 27 | 1138 | 806 | 121 |
| New Minor Forcing | 1076 | 63 | 1076 | 44 | -39 |
| 1N-2S Minor Suit Stayman | 1065 | 5 | 1065 | 1046 | -1118 |
| strength cue bid | 1044 | 157 | 1044 | 209 | -331 |
| artificial ask | 1023 | 18 | 1023 | 0 | 0 |
| no x queen | 1022 | 180 | 1022 | 0 | 0 |
| Fourth suit game force | 1006 | 74 | 1006 | 609 | -981 |
| Gambling | 1004 | 20 | 1004 | 1003 | -794 |
| bidable suit | 984 | 64 | 984 | 0 | 0 |
| 1N-3C Puppet Stayman | 977 | 2 | 977 | 970 | -333 |
| Extended acceptance after NT | 963 | 43 | 963 | 239 | -475 |
| to the partner's longer | 920 | 66 | 920 | 0 | 0 |
| 0 or 1 trump keys | 880 | 150 | 880 | 0 | 0 |
| Jordan Truscott 2NT | 859 | 43 | 859 | 649 | -1476 |
| Reverse drury | 854 | 20 | 854 | 626 | -48 |
| any 4441 | 798 | 8 | 798 | 236 | -523 |
| Blackwood 0123 | 782 | 18 | 782 | 183 | -665 |
| 1N-3C transfer to diamonds | 749 | 54 | 749 | 500 | -532 |
| 3 x | 737 | 50 | 737 | 334 | -53 |
| K=1 | 708 | 106 | 708 | 0 | 0 |
| Multi | 696 | 2 | 696 | 696 | -528 |
| A=2 out of 4 | 633 | 46 | 633 | 30 | -122 |
| K=0 | 610 | 106 | 610 | 0 | 0 |
| 1 out of 4 | 562 | 62 | 562 | 0 | 0 |
| x queen no x king no x king no x king | 504 | 133 | 504 | 0 | 0 |
| Flannery | 499 | 1 | 499 | 499 | -15 |
| Multi-Landy | 495 | 5 | 495 | 479 | 109 |
| 2N-3C Puppet Stayman | 495 | 1 | 495 | 170 | -113 |
| Fourth suit | 480 | 44 | 480 | 311 | -616 |
| Two way game tries | 475 | 45 | 375 | 311 | 425 |
| Super acceptance after NT | 469 | 63 | 469 | 167 | -183 |
| 5NT pick a slam | 444 | 32 | 444 | 178 | -455 |
| Mixed raise | 428 | 71 | 428 | 233 | -132 |
| 1N-2N transfer to clubs | 413 | 4 | 413 | 238 | -786 |
| 5431 after 1NT | 409 | 3 | 409 | 408 | -319 |
| A=0/5 or 4/5 | 391 | 10 | 391 | 0 | 0 |
| Responsive double | 381 | 32 | 381 | 41 | -14 |
| 1N-2S transfer to clubs | 380 | 49 | 380 | 101 | -198 |
| K=2 | 374 | 59 | 374 | 0 | 0 |
| max 2 x | 369 | 78 | 369 | 5 | -1 |
| inviting | 363 | 14 | 363 | 93 | -263 |
| Fit showing jumps | 361 | 27 | 361 | 320 | -694 |
| 2 out of 4 | 358 | 56 | 358 | 2 | 9 |
| Rodrigue | 352 | 3 | 352 | 351 | -192 |
| x queen no x king no x king | 350 | 114 | 350 | 0 | 0 |
| A=1/4 | 347 | 32 | 347 | 4 | -37 |
| 1N-3M splinter | 347 | 13 | 347 | 339 | -135 |
| A=3 out of 4 | 336 | 19 | 336 | 29 | -69 |
| Ogust | 318 | 1 | 318 | 94 | 38 |
| King ask by available bid | 294 | 68 | 294 | 0 | 0 |
| Weak Jump Shifts 2 | 283 | 3 | 283 | 281 | -314 |
| Unusual 4NT | 278 | 96 | 278 | 190 | -180 |
| A=1/5 or 5/5 | 275 | 18 | 275 | 0 | 0 |
| Benjamin 2D | 265 | 1 | 265 | 265 | -719 |
| Kokish Relay | 264 | 1 | 264 | 102 | -76 |
| Completing the relay | 261 | 1 | 261 | 155 | -129 |
| Reverse Flannery 2H | 254 | 3 | 254 | 253 | -173 |
| Roudi | 251 | 8 | 251 | 238 | -750 |
| Reverse Flannery 2S | 245 | 1 | 245 | 245 | -62 |
| NMF after 2NT rebid | 235 | 25 | 235 | 151 | -90 |
| Lebensohl after 1m | 227 | 30 | 227 | 119 | -251 |
| Gazzilli | 209 | 1 | 209 | 126 | -63 |
| Minor Suit Stayman after 2NT | 200 | 1 | 200 | 193 | -97 |
| Bergen | 199 | 9 | 199 | 190 | -193 |
| 0 out of 4 | 196 | 53 | 196 | 0 | 0 |
| Polish two suiters | 171 | 1 | 171 | 171 | 2 |
| 1N-3D splinter | 163 | 14 | 163 | 151 | -17 |
| 5+x 4+x | 158 | 1 | 158 | 0 | 0 |
| 1+ or 2 trump keys | 156 | 56 | 156 | 0 | 0 |
| Lebensohl after 1NT | 147 | 20 | 147 | 0 | 0 |
| Drury | 144 | 24 | 144 | 56 | 9 |
| 2 x | 134 | 5 | 134 | 3 | 8 |
| shortness | 123 | 19 | 123 | 48 | -211 |
| A=0/4 or 4/4 | 117 | 24 | 117 | 13 | -72 |
| Minors | 112 | 31 | 112 | 83 | 32 |
| Collante | 108 | 2 | 108 | 33 | -50 |
| weak | 101 | 1 | 101 | 0 | 0 |
| Rubensohl after 1m | 90 | 17 | 90 | 15 | -62 |
| Weak Jump Shifts 3 | 90 | 2 | 90 | 86 | -76 |
| A=2 out of 5 | 81 | 13 | 81 | 0 | 0 |
| SOS | 80 | 21 | 80 | 6 | -25 |
| Minor Suit Transfers after 2NT | 79 | 21 | 79 | 48 | -28 |
| Two suit takeout double | 75 | 36 | 75 | 38 | -6 |
| denies a x stopper | 70 | 37 | 70 | 0 | 0 |
| Precision 2D | 68 | 10 | 68 | 68 | 11 |
| 1X-(1Y)-2Z weak | 63 | 27 | 63 | 51 | -36 |
| 3 out of 4 | 60 | 26 | 60 | 0 | 0 |
| Crosswood 0314 | 57 | 15 | 57 | 3 | -24 |
| King ask | 51 | 15 | 51 | 0 | 0 |
| Two Way New Minor Forcing | 49 | 9 | 49 | 27 | -66 |
| x queen no x king | 47 | 29 | 47 | 0 | 0 |
| both majors | 45 | 29 | 45 | 4 | -5 |
| K=3 | 38 | 21 | 38 | 0 | 0 |
| A=3 out of 5 | 34 | 12 | 34 | 0 | 0 |
| show your better suit | 33 | 15 | 33 | 0 | 0 |
| Support 1NT | 32 | 9 | 32 | 21 | 2 |
| Unusual 1NT | 30 | 14 | 30 | 18 | -11 |
| relay | 30 | 15 | 30 | 1 | -4 |
| Reverse Bergen | 29 | 6 | 29 | 22 | -84 |
| A=1/4 or 4/4 | 27 | 17 | 27 | 0 | 0 |
| 1M-3M blocking | 25 | 5 | 25 | 21 | -35 |
| x queen | 23 | 10 | 23 | 0 | 0 |
| 4432 or 4441 | 22 | 6 | 22 | 0 | 0 |
| 4NT opening | 21 | 4 | 21 | 21 | -64 |
| 1m opening allows 5M | 18 | 10 | 18 | 10 | 2 |
| asks a stopper | 17 | 15 | 17 | 1 | 0 |
| solo suit | 16 | 10 | 16 | 1 | 0 |
| 1/5 or 3/5 | 13 | 9 | 13 | 0 | 0 |
| support | 12 | 4 | 12 | 0 | 0 |
| A=0/4 or 3/4 | 11 | 8 | 11 | 0 | 0 |
| 0/5 or 2/5 | 10 | 7 | 10 | 0 | 0 |
| Maximal Doubles | 7 | 6 | 7 | 2 | 0 |
| Rubensohl after 1NT | 6 | 2 | 6 | 0 | 0 |
| Rubensohl after double | 5 | 5 | 5 | 1 | 0 |
| 4 out of 4 | 3 | 3 | 3 | 0 | 0 |
| K=4 | 2 | 2 | 2 | 0 | 0 |
