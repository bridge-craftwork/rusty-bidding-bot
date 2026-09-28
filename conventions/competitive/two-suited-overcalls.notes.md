# Michaels cue bids and the Unusual 2NT (`two-suited-overcalls.bid`): notes

Two-suited overcalls of a one-level suit opening on our right, advancer's
answers, the two-suiter's rebid, and the opening side's answer when
they make one. Cases: `two-suited-overcalls.test`.

## Guidance (Rick, 2026-09-28)

- **Michaels**: (1m) 2m shows both majors; (1M) 2M the other major and a
  minor. Strength conventions (for example weak or strong, not
  intermediate) and vulnerability.
- **Unusual 2NT**: the two lowest unbid suits (over a major, both minors).
- **Advancer**: preference, jump preference (invitational), asking for
  the minor after a major-minor Michaels, cue bids with game interest,
  passing.
- **The two-suiter's rebids** (the strong variety bids again), and the
  opening side's actions over them.
- Card fields under Bridge-Classroom's paths.

## Card fields

| Field | Meaning | Source |
|---|---|---|
| `direct_cuebids.nat_minors_michaels` | our cue of a natural 1♣/1♦ is Michaels | Bridge-Classroom's DirectCuebidsMatrix (new here) |
| `direct_cuebids.quasi_michaels` | the same over a 1♣/1♦ that may be short | the matrix (new) |
| `direct_cuebids.nat_majors_michaels` | our cue of 1♥/1♠ is Michaels | the matrix (new) |
| `direct_cuebids.{art,quasi,nat_minors,nat_majors}_{michaels,natural,other}`, `direct_cuebids.description` | the rest of the matrix and its notes | recorded, not read |
| `competitive.michaels.play` | Bridge-Classroom's catalog switch: Michaels over both | existing |
| `competitive.michaels.strength` | `wide_range` (default) or `weak_or_strong` | was free text; now an enum. `weak_or_strong` is the value Bridge-Classroom's seed card writes |
| `competitive.unusual_2nt.play`, `nt_overcalls.jump_2nt_lowest_unbid` | the Unusual 2NT (catalog switch, and the card's NT-overcall box) | existing; either turns the rules on |
| `competitive.unusual_vs_unusual.play` | the opening side's cue bids over their 2NT, Bridgebum's way | existing |

The matrix's columns are the kind of opening (artificial ♣/♦, quasi-
natural ♣/♦, natural ♣/♦, natural ♥/♠) and its rows what our cue bid
means (Michaels, natural, other). The rules read the Michaels cells of
the natural columns and of quasi (a possibly short minor is treated like
a natural one: our rules cannot see the opponents' minimum length when
we choose). The artificial column is recorded only; there are no
strong-club rules yet.

**`.bbsa`**: `Michaels Cuebid` now sets the matrix
(`nat_minors_michaels` and `nat_majors_michaels`) instead of
`competitive.michaels.play`. The meaning is certain from the probes
below: over 1♣/1♦ BBA's alert is "5+ hearts, 5+ spades", over 1♥/1♠
"5+ in the other major, 0-3 in theirs", and BBA's minor openings are
natural. `Unusual 2NT` keeps its mapping (`competitive.unusual_2nt.play`),
whose meaning the probes confirm (the two lowest unbid suits, five-five).
Nothing about short or artificial minors is mapped.

Rick's own card (a Bridge-Classroom export) sets `nat_minors_michaels`,
`nat_majors_michaels` and `nt_overcalls.jump_2nt_lowest_unbid`, which the
rules now read, and says "minimax Mich, Leaping Michaels" in
`direct_cuebids.description`. Free text is not parsed: **for Rick**, set
`competitive.michaels.strength = weak_or_strong` on the card for
mini-maxi. Leaping Michaels is not written.

## What BBA does (probes, 2026-09-28, 21GF-DEFAULT, IMPs)

Specs in `probes/`, made with `probes/gen_hands.py`, dealer E opening,
columns None / NS (we vulnerable) / EW.

**The overcall** (`michaels-1C`, `-1D`, `-1H`, `-1S`, `unusual-2n-1C`,
`-1D`, `-1H`, `-1S`: 200-300 five-five hands each, 5-19 HCP):

- Michaels: BBA's alert is "8 to 29 total points" not vulnerable, "12 to
  29" vulnerable. In HCP: not vulnerable about half the 7-counts bid it,
  almost all from 8 (singletons and voids in their suit count); up to 17
  HCP; 18-19 double. Vulnerable, 8-10 overcalls the higher suit, 11-12
  split, 13+ Michaels. Same over all four openings.
- Unusual 2NT: "9 to 29 total points" at every vulnerability, from about
  7 HCP, and still 2NT on 19. Over 1♣ diamonds and hearts, over 1♦ clubs
  and hearts. With six-six in the minors BBA sometimes bids 4NT (Unusual
  4NT: not written).

**Advancer** (`michaels-adv-1C`, `michaels-adv-1H`, `unusual-2n-adv-1H`):

- BBA never passes the cue. Preference "to the partner's longer" with
  two or more; with equal length it prefers spades over hearts and
  diamonds over clubs.
- The jump preference is **preemptive** in BBA ("4 to 9 total points,
  3+"), the jump to game "preemptive" 4-11 with four trumps (and with
  more it bids game too).
- 2NT after a major Michaels: "to the partner's longer", asking for the
  minor.
- The cue bid is a "strength cue bid" with 17+ (16+ after 2NT).
- 3NT with their suit held, 15-16+.
- After 2NT: preference at the three level, 4m "preemptive" 4-8 with
  four, 5m to play.

**The opening side** (`vs-michaels-1H`, `vs-michaels-1C`,
`vs-unusual-2n-1H`; BBA's cards play no Unusual vs Unusual):

- 1♥ (2♥): 2♠ limit raise or better (10+, three hearts); X "support"
  6-9 with three; 3♥/4♥ preemptive; 2NT/3NT natural with spades held;
  3m natural 13+.
- 1♣ (2♣): 2♥ limit raise or better in clubs (12+, four); 2♠ "strength
  cue" 11+; X "support" 6-9 with four clubs; 2♦ natural, forcing;
  3NT with both majors held.
- 1♥ (2NT): 3♣ limit raise or better; 3♦ "strength cue" 14+; 3♥ 7-10
  with three; 4♥ preemptive; 3♠ natural; X penalty, 10+.

## What we play, and where we differ from BBA

- **The overcall**: as BBA, in our count: `hcp+length_points` 9+ not
  vulnerable (7 HCP five-five), 11+ vulnerable (9 HCP), up to 17 HCP
  (the power double above). BBA waits for about 11 HCP vulnerable;
  **11 beat 13 on both yardsticks** (+93 IMPs par distance, +231 to the
  overcalling side, 334 boards). Not vulnerable, 10 and 8 were tried:
  the yardsticks disagreed both times (10: par +81, side −146; 8: par
  −51, side +94), so 9 stays.
- **weak_or_strong** (mini-maxi, Bridgebum and Wikipedia): weak 9-11
  HCP-ish (`hcp+length_points` 9+, 11+ vulnerable, HCP 11 at most), or
  16+ and bids again. Played on both sides of the whole corpus it lost
  on both yardsticks (par −603, side −430, 1,283 boards), which only
  says it is not BBA's method: the corpus auctions were bid with a wide
  range. It stays an option for cards that ask for it.
- **The jump preference is invitational** (Rick; Wikipedia: "preference
  bids or jump preferences"); BBA's is preemptive, and Bridgebum's jump
  is preemptive too. BBA's way measured +3 par, +65 side IMPs (312
  boards): a wash. Rick's ruling stands; **for Rick** if he wants BBA's.
- **The cue bid** by advancer is 16+, game or slam interest, forcing
  (BBA 17+, Bridgebum "game or slam interest"); over 1♠ (2♠), where the
  hearts can only be shown at the three level, the cue is 12+ (game
  interest), since there is no room for an invitational jump.
- **The opening side**: the cue of their known suit (after Michaels) or
  of their lower suit (after 2NT) is the limit raise, as BBA; the
  raise itself is competitive (BBA uses a double for the 6-9 raise and
  keeps 3♥ preemptive); four of our major with four trumps is
  preemptive. **Not written**: BBA's doubles (the 6-9 raise double after
  Michaels, the penalty double after 2NT), because opener would need
  answers to them. With Unusual vs Unusual on we play Bridgebum's
  version (3♣ game force in the other major, 3♦ the limit raise, 3♠
  natural 7-10).
- **Opener** accepts the limit raise with 14+ support points (a major)
  or 14+ HCP (3NT; five of the minor from 16); after the competitive
  raise bids game with 17+ support points; after responder's 2NT bids
  3NT with 14+; once responder has raised or bid notrump and they bid on,
  passes, or doubles with 15+.
- `after-interference.bid`: two contexts that read our raise over their
  cue bid as a cue-bid raise now say `y is not x` (1♦ (2♦) 3♦ was taken
  for "1x (2y) 3y": opener accepted a "limit raise" that was a 6-9
  raise).

## Corpus (2026-09-28)

Before and after, whole corpus and the boards where both sides bid in
BBA's auction (`--auctions competitive`). "vs BBA" is distance from par
against BBA; "side" is `probes/tools/sideimps.py BASE NEW`.

| | calls agreeing | vs BBA (par distance) | side IMPs | no rule in a live auction |
|---|---|---|---|---|
| whole corpus | 76.7% → 77.0% | −114,531 → −113,454 (**+1,077**) | **+3,480** (4,351 boards) | 2,835 → 2,800 |
| competitive auctions | 71.1% → 71.9% | −40,830 → −39,732 (**+1,098**) | **+3,268** (4,181 boards) | |

The two yardsticks agree. By opening (side IMPs): over 1♣ +1,084, 1♥
+941, 1♦ +776, 1♠ +680.

The two-suited scenarios (calls agreeing with BBA; vs BBA in IMPs):

| scenario | calls | vs BBA |
|---|---|---|
| Michaels_Cuebid | 62.5% → 72.9% | −410 → −391 |
| Michaels_after_1m | 58.5% → 69.2% | −598 → −352 |
| Michaels_and_Unusual | 60.5% → 74.1% | −204 → −126 |
| Unusual_2N | 62.6% → 77.9% | −280 → −195 |
| Two-Suited_Overcalls | 63.0% → 73.7% | −48 → −83 |
| Opps_Michaels_Cuebid | 60.3% → 75.2% | −429 → −20 |
| Opps_Michaels_and_Unusual | 60.4% → 73.0% | −307 → −206 |
| Opps_2-Suited_Overcalls | 63.7% → 72.8% | −363 → −350 |
| Leaping_Michaels, Non_Leaping_* | unchanged (not written) | |

Probe agreement with BBA after the change (None / we vulnerable):
Michaels over 1♣ 260/300 and 205/300 (we bid it lighter vulnerable, by
design), over 1♥ 273/300 and 193/300; Unusual 2NT over 1♥ 256/300 and
262/300; advancer after 1♣ (2♣) 151/300, after 1♥ (2♥) 157/300, after
1♥ (2NT) 177/300 (the differences are the invitational jump, our
preemptive four-level raise with four trumps and 9 HCP or less where
BBA bids game from 12, and our 4m preempt after 2NT); the opening side
216/300 over Michaels of 1♥, 171/300 over Michaels of 1♣, 209/300 over
2NT (mostly BBA's doubles).

## Gaps and open questions

- Michaels and the Unusual 2NT in the **balancing seat**
  (`(1x) P (P) 2x`), by a passed hand over a second-seat opening only
  through the same rules; after two suits bid (`1x P 1y 2NT`).
- **Leaping Michaels** (Rick's card) and non-leaping Michaels over weak
  twos; Unusual 1NT and 4NT (BBA bids 4NT with six-six).
- The opening side's **doubles** (above), and "unusual over unusual"
  over a minor opening and over Michaels (we play BBA's natural-ish
  structure there whatever the card says).
- Advancer when opener's partner bids more than a preference allows
  (a jump, notrump): pass unless a simple competitive preference fits.
- A two-suiter that is 6-5: no special treatment.

## Sources

- **Rick's rulings** (2026-09-28): the task above, and the jump
  preference as invitational.
- **Bridgebum**, "Michaels Cuebid"
  (<https://www.bridgebum.com/michaels_cuebid.php>): what the cue shows,
  "no point minimum" with vulnerability considered, the mini-maxi option
  (0-10 or 16+), preference with the cheaper suit on equal length, jump
  raises **preemptive**, 2NT asking for the minor, the raise of the cue
  "game or slam interest", the Michaels bidder's minimum rebid after it.
  We differ: the jump is invitational (Rick), equal length prefers
  spades (BBA).
- **Bridgebum**, "Unusual 2NT" (<https://www.bridgebum.com/unusual_2nt.php>):
  the two lowest unbid suits, preference, preemptive jumps, the cue with
  a good hand. We differ: equal minors prefer diamonds (BBA; Bridgebum
  says the cheapest).
- **Bridgebum**, "Unusual vs. Unusual"
  (<https://www.bridgebum.com/unusual_vs_unusual.php>): over 1♥ (2NT),
  3♣ a game force in the other major, 3♦ a limit raise, 3♥ 7-10, 3♠
  natural 7-10, double for penalty. Played when the card turns the
  convention on; the double is not written.
- **Wikipedia**, "Michaels cuebid"
  (<https://en.wikipedia.org/wiki/Michaels_cuebid>): 8+ points, the
  mini-maxi variant (weak 8-12, strong 16+), jump preferences, the cue as
  a game or slam try, 2NT for the minor.
- Bridge Guys' page could not be fetched (server error, 2026-09-28): not
  used.
- **BBA probes**: `probes/michaels-1C.toml`, `-1D`, `-1H`, `-1S`,
  `probes/unusual-2n-1C.toml`, `-1D`, `-1H`, `-1S`,
  `probes/michaels-adv-1C.toml`, `michaels-adv-1H.toml`,
  `unusual-2n-adv-1H.toml`, `probes/vs-michaels-1H.toml`,
  `vs-michaels-1C.toml`, `vs-unusual-2n-1H.toml` (with the alerts they
  record).
- **Bridge-Classroom**: `DirectCuebidsMatrix.vue` (the matrix fields),
  `conventionCatalog.js` (`competitive.michaels.play`,
  `competitive.unusual_2nt.play`, `nt_overcalls.jump_2nt_lowest_unbid`),
  the seed card (`competitive.michaels.strength = weak_or_strong`).
- **Corpus measurements**: the thresholds and the treatments above
  (2026-09-28).
