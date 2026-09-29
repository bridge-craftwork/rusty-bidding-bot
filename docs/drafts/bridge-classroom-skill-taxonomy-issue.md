# Issue for Bridge-Classroom (filed as bridge-craftwork/Bridge-Classroom#422)

Drafted 2026-09-28 from rusty-bidding-bot's work on tying teaching skills
to card fields and rules (docs/SKILLS.md). Every claim below was checked
against the files in ../Bridge-Classroom, ../Bridge-Lessons-* on that date;
line numbers are from then. To file it, copy the title and body into a new
issue in the Bridge-Classroom repository.

---

**Title:** Skill taxonomy: card paths the mapping reads but no card writes, duplicate skill names, lesson tags missing from skillPaths.json

**Body:**

`src/utils/cardToTaxonomyMapping.js` decides which skills a convention card
lights up, and `public/data/skillPaths.json` is the list lessons are tagged
from. Checking both against what the card editor, the importers and the
lesson files actually write turned up the problems below. rusty-bidding-bot
(bridge-craftwork/rusty-bidding-bot) now tags its card fields and bidding
modules with these same skill paths and generates a cross-check,
[docs/SKILLS.md](https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/SKILLS.md),
which is where most of this came from.

### (a) Card paths in `cardToTaxonomyMapping.js` that cards do not write

These entries read a path that nothing in the editor, the importers or the
seed card writes (or that only one of several writers uses), so the skill
never lights up for most cards:

| Mapping entry (line) | What cards actually write |
|---|---|
| `competitive.support_doubles.play` (206) | `doubles.support.play`: the editor catalog (`conventionCatalog.js` 692), `bridgeodexImport.js` 385, `acblCardPdf.js` 530, `acblClassicFillPdf.js` 326. Nothing writes `competitive.support_doubles.*`. |
| `notrump.lebensohl.over_weak_twos` (70) | Nothing writes it. The editor's vs-preempts panel (`VsPreemptsPanel.vue` 41-43), `bridgeodexImport.js` 440, `bboImport.js` 157 and both PDF fillers use `vs_preempts.lebensohl_response`; the catalog row uses a third path, `competitive.lebensohl_weak_twos.play` (`conventionCatalog.js` 727). |
| `competitive.michaels.play` (188) | Only the catalog row (`conventionCatalog.js` 698) and the seed card write it. The editor's `DirectCuebidsMatrix.vue`, `bridgeodexImport.js` 446-450, `bboImport.js` 143-144 record Michaels in the direct-cuebid matrix, `direct_cuebids.{art,quasi,nat_minors,nat_majors}_michaels`, and both PDF exporters (`acblCardPdf.js` 651, `acblClassicFillPdf.js` 364-366) read it from there. An imported card never gets `competitive_bidding/michaels_unusual`. |

The same check found four more of the same kind:

| Mapping entry (line) | What cards actually write |
|---|---|
| `other_conventions.fourth_suit_forcing.play` (174) | Nothing. The catalog rows and `bridgeodexImport.js` 494-495 write `.one_round` and `.game_force`. |
| `competitive.negative_doubles.through` (182) | Only the seed card. The catalog (`conventionCatalog.js` 681) and `bridgeodexImport.js` 380-381 write `doubles.negative.play` / `doubles.negative.through`. |
| `competitive.dont.play` (200) | Only the catalog row (709). The editor's `VsNtDefense.vue` and both importers record the defence as text in `competitive.vs_1nt_strong.system` / `vs_1nt_weak.system` ("DONT"). |
| `other_conventions.blackwood.*`, `other_conventions.gerber.play` (142-160) | The editor and importers write these, but the seed card `bridge-classroom-api/seed_data/21_intermediate_card.json` still has `slam.blackwood.*` and `slam.gerber.*` (the catalog's comment at 596 says the seed "previously" used them), so the seed card lights up no Blackwood, RKCB or Gerber skill. |

`documentation/CARD_TAXONOMY_MAPPING.md` repeats the same paths.

### (b) Duplicate or inconsistent skill names in `skillPaths.json`

- **One topic, two paths, spelled differently.** The `partnership_bidding/*`
  paths are the Baker Bridge partnership deal sets for topics that also
  have a `bidding_conventions/*` or `competitive_bidding/*` path:
  `bidding_conventions/blackwood` / `partnership_bidding/blackwood`,
  `bidding_conventions/roman_keycard` / `partnership_bidding/roman_key_card`,
  `bidding_conventions/weak_2s` / `partnership_bidding/weak_twos`,
  `bidding_conventions/strong_2c` / `partnership_bidding/two_club`,
  `bidding_conventions/jacoby_2nt_splinters` / `partnership_bidding/jacoby_2nt`,
  `competitive_bidding/negative_doubles` / `partnership_bidding/negative_doubles`,
  `competitive_bidding/overcalls` / `partnership_bidding/overcalls`,
  `bidding_conventions/stayman` + `bidding_conventions/jacoby_transfers` /
  `partnership_bidding/stayman_transfers`. The path mixes the topic with
  the deal format, the spellings drift (`roman_keycard` vs
  `roman_key_card`, `weak_2s` vs `weak_twos`), and the mapping never names
  a `partnership_bidding/*` path, so a card that plays RKCB does not pull
  in the partnership RKCB deals.
- **Support doubles feed `competitive_bidding/support_cuebids`**
  (mapping line 206, `conventionCatalog.js` 692). A support double
  (opener's double showing three-card support) is not a support cue bid
  (the cue-bid raise, which the catalog also maps there, at 715). They need
  their own skill.
- **Gerber is filed under `bidding_conventions/blackwood`** (mapping 160,
  catalog 626): a different convention (4♣ over notrump), taught
  separately.
- Smaller cases of the same: original Drury (`major_openings.drury.play`)
  maps to `bidding_conventions/reverse_drury` (catalog 485); Smolen and
  Garbage Stayman map to `bidding_conventions/stayman` (catalog 424, 442),
  so a lesson on Smolen cannot be told from one on Stayman.

### (c) SkillPath tags in lesson files that `skillPaths.json` lacks

`grep -rhoE '\[SkillPath "[^"]+"\]'` over Bridge-Classroom and the
Bridge-Lessons-* repositories finds 35 paths used in lessons but missing
from `skillPaths.json` (and from `BAKER_BRIDGE_TAXONOMY`, which lists the
same 50). Occurrences in parentheses.

Bridge-Lessons-by-Andrew-Rowberg (`Lock-Step Lessons`):
`precision/1c_mixed` (43), `precision/1c_opener` (43), `precision/1c_responder` (43).

Bridge-Lessons-by-Eddie-Kantar (`Curated/` and `Packaged/`), bidding:
`basic_bidding/hand_evaluation` (170),
`competitive_bidding/advancing_overcalls` (153),
`competitive_bidding/balancing` (45),
`partnership_bidding/miscellaneous` (221),
`partnership_bidding/passed_hand_bidding` (170),
`partnership_bidding/rebids` (391),
`partnership_bidding/responders_rebid` (204),
`partnership_bidding/slam_bidding` (357).

Bridge-Lessons-by-Eddie-Kantar, declarer play:
`declarer_play/card_combinations` (588), `declarer_play/combining_chances` (119),
`declarer_play/counting_distribution` (119), `declarer_play/counting_the_hand` (45),
`declarer_play/counting_their_tricks` (136), `declarer_play/counting_your_tricks` (504),
`declarer_play/deception` (204), `declarer_play/loser_on_loser` (136),
`declarer_play/miscellaneous` (340), `declarer_play/notrump_play` (45),
`declarer_play/overtaking` (78), `declarer_play/placing_honors` (525),
`declarer_play/playing_to_make` (45).

Bridge-Lessons-by-Eddie-Kantar, defence:
`defense/card_combinations` (170), `defense/counting_declarers_tricks` (289),
`defense/discarding` (272), `defense/dummy_recognition` (289),
`defense/inferences` (20), `defense/miscellaneous` (221), `defense/overtaking` (20),
`defense/ruffs` (187), `defense/signals_vs_suit_contracts` (168),
`defense/taking_charge` (78), `defense/trump_promotion` (357).

(Template strings in documentation and scripts, such as `category/skill`,
`${board.skillPath}` and `{skill_path}`, are left out.)

### (d) Proposed new skill paths

rusty-bidding-bot has rules for these conventions, and the card has a field
for most of them, but the taxonomy has no skill, so neither a lesson nor a
card can name them. Proposed in the existing style (the card field each one
belongs to in brackets):

- `bidding_conventions/smolen` [`notrump.smolen.play`]
- `bidding_conventions/drury` [`major_openings.drury.play`, `drury.two_d`]: original and two-way Drury; `reverse_drury` stays for Reverse
- `bidding_conventions/gerber` [`slam.gerber.play`, i.e. `other_conventions.gerber.play`]
- `bidding_conventions/inverted_minors` [`minor_openings.inverted_minors.play`]
- `bidding_conventions/impossible_2s` [`major_openings.impossible_2s.play`]
- `bidding_conventions/super_accept` [`notrump.transfers.super_accept`]
- `bidding_conventions/minor_suit_transfers` [the 1NT minor-suit responses: relay, four-way transfers]
- `bidding_conventions/control_bids` [`slam.cue_bids.play`]
- `competitive_bidding/support_doubles` [`doubles.support.play`, `doubles.support.rdbl`]
- `competitive_bidding/responsive_doubles` [`doubles.responsive.play`]
- `competitive_bidding/defense_vs_1nt` [`competitive.vs_1nt_strong.system`, `vs_1nt_weak.system`], with one per defence:
  `competitive_bidding/cappelletti`, `competitive_bidding/multi_landy` (Landy and Multi-Landy), `competitive_bidding/meckwell` (DONT already has `competitive_bidding/dont`)
- `competitive_bidding/unusual_vs_unusual` [`competitive.unusual_vs_unusual.play`]
- `competitive_bidding/defense_vs_preempts` [the vs-preempts section]
- `competitive_bidding/law_of_total_tricks` [no field]

The lesson-only tags in (c) that rusty-bidding-bot's modules also use
(`competitive_bidding/advancing_overcalls`, `competitive_bidding/balancing`,
`partnership_bidding/rebids`, `partnership_bidding/responders_rebid`,
`partnership_bidding/slam_bidding`) would be enough as they are, once they
are in `skillPaths.json`.

### (e) The cross-check

rusty-bidding-bot keeps a copy of `skillPaths.json` in
`conventions/card/skills.toml` (with the lesson-only and proposed paths
marked as such), tags each card field with its skill (`skill = ...` in
`conventions/card/fields.toml`, the field being the convention's canonical
ID) and each bidding module with the skills it implements, and generates
the map `skill -> card fields -> modules`, with its gaps, into
[docs/SKILLS.md](https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/SKILLS.md)
(`rbb bid skills`; a test fails when it drifts). Its "Card fields tagged
with a skill not in Bridge-Classroom's taxonomy" list is this issue's (d),
and it will shrink as paths are adopted here.

### Acceptance criteria

- [ ] Every `card_path` in `cardToTaxonomyMapping.js` is a path the editor
      (`conventionCatalog.js` and the `conventionCard` components), the
      Bridgeodex and BBO importers and the seed card write, or the entry
      lists each of the paths they use: support doubles
      (`doubles.support.play`), Lebensohl over weak twos
      (`vs_preempts.lebensohl_response`, and the catalog's
      `competitive.lebensohl_weak_twos.play` unified with it), Michaels
      (the `direct_cuebids.*_michaels` cells as well as
      `competitive.michaels.play`), fourth suit forcing (`.one_round` /
      `.game_force`), negative doubles (`doubles.negative.*`), DONT (the
      `vs_1nt_*.system` value), Blackwood/RKCB/Gerber (the seed card moved
      to `other_conventions.*`, or both read).
- [ ] `CARD_TAXONOMY_MAPPING.md` matches the code.
- [ ] A test fails when a mapping entry names a path no card writer uses.
- [ ] `skillPaths.json` lists every SkillPath used in the lesson repositories
      (the 35 in (c)), or those lessons are retagged; a check (script or CI)
      fails on a tag not in the list.
- [ ] The duplicate names in (b) are resolved: one spelling per topic (or an
      explicit alias table), and support doubles and Gerber no longer map
      to support cue bids and Blackwood.
- [ ] The proposed paths in (d) are added (or rejected with a reason), and
      the catalog and mapping use them for the listed card fields.
