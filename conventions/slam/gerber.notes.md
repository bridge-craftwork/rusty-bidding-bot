# gerber (`gerber.bid`): notes

Gerber (card `slam.gerber.play`, alias `other_conventions.gerber.play`;
BBA's `Gerber`, on 17 of the 18 corpus cards, and on Rick's card). 4♣
over partner's natural notrump asks for aces: 4♦ 0 or 4, 4♥ 1, 4♠ 2,
4NT 3. 5♣ then asks for kings on the same steps. The asker places the
contract; 4NT after an answer is a sign-off. Cases: `gerber.test`.
Built 2026-09-28.

## Where 4♣ is Gerber

- **Directly over partner's 1NT or 2NT opening**, and over 2♣–2♦–2NT.
- **Over partner's 1NT or 2NT rebid** after 1x–1y, unless the card has
  `gerber.only_over_nt_openings` (BBA's "Gerber only for NT openings",
  set on 21GF-MSS, 21GF-Multi and 21GF-NoInvertedMinor). BBA uses it
  there (Gerber_By_Responder: 1♦–1♠–1NT–4♣, 1♣–1♥–2NT–4♣).
- **Over interference to our 1NT opening** (2026-10-30): after their
  double or a suit overcall at the two level (1NT (2♣) 4♣, 1NT (2♥) 4♣,
  1NT (X) 4♣) the jump to 4♣ is still Gerber, on the same hands as
  without it (`bba` treatment: BBA's 31 HCP). 4♣ is neither a useful
  cue-bid nor natural there. Off with a 4♣ Texas (`texas_4c`, 2-under).
- **After Stayman's 2♦** (1NT–2♣–2♦–4♣, 2026-10-30): no suit agreed and
  clubs not natural, so the jump is Gerber (Bridge Bum: "a jump to 4♣
  after Stayman"). The hands that bid 6NT there (stayman.bid, 32 between
  the hands) ask first. Not with `general.style = bba`: BBA's 4♣ there
  is its minor slam try (stayman.bid, probed). Not after 2♥/2♠: a fit
  may exist and 4♣ stays free for cue-bids/splinters.
- **Gerber by opener over a natural 2NT response** (2026-10-30):
  1♣–2NT–4♣, 1♦–2NT–4♣ (slam values against responder's range, no
  five-card major) and 2♣–2NT–4♣ (balanced, 33 between the hands: the
  hands that bid 6NT there ask first). PBS Gerber_By_Opener defines
  exactly these sequences, citing Bridge Bum. **Decision: 1♣–2NT–4♣ is
  Gerber, not a club slam try.** Opener's 1♣ is not agreement (it may be
  three cards, or two on some cards), and opener with real clubs can
  show them at the three level, still forcing over a natural 2NT.
  Wikipedia's "not when a natural club suit has been bid" would make it
  natural; we read that as "after clubs are bid and supported"
  (1♣–2♣–2NT–4♣ stays off). These and the 1x–1y–NT contexts follow the
  card's "over NT sequence" (`slam.gerber.over_nt_seq`, default true:
  the field is not in the `.bbsa` map, so only a card JSON can switch it
  off; SAYC.json sets it true) and BBA's "Gerber only for NT openings"
  (`only_over_nt_openings`, which turns them off).
- **Brill: over opener's 3NT rebid after 2♣** (2♣–2♦–3NT, 2♣ (2♥) P (P)
  3NT, 2♣ P 2♦ (2♥) 3NT, any overcall), as over Brill's 3NT opening
  (`general.system = brill`). The Brill card needs its JSON
  (`-c Brill`); `cards/bbsa/Brill.bbsa` imports without
  `general.system`, so with it these stay the default.
- **Nowhere else.** Not after a transfer, a Stayman major answer, over a
  minor raise, or when clubs have been bid and supported: 1♣–2♣–2NT–4♣ is
  whatever it was before (27 corpus boards, in minor-raise scenarios).
  Not over our 1NT overcall either: no corpus evidence
  (We_Overcall_1N_then_Gerber has no BBA file). Default (non-Brill):
  after 2♣–2♦–3NT 4♣ stays over-3nt.bid's club slam try (as over the 3NT
  opening), and after 2♣ (overcall) … 3NT there is still no 4♣ rule
  (over-3nt.bid stands aside once they bid); see "Open questions".
- The card's other Gerber fields: `gerber.over_nt_seq` is read (above);
  `gerber.over` is free text (Bridge-Classroom's seed card says
  `1nt_2nt_only`, which is what the opening context does);
  `gerber.directly_over_nt` (covered by `play`) and `non_nt_seq` (4♣
  ace-ask in suit auctions: not played) are read by no rule. Rick's card
  sets only `other_conventions.gerber.play`.

## Decisions

- **Texas.** Gerber is 4♣ only; Texas is 4♦/4♥, so they coexist over
  1NT, and after a Texas completion 4NT stays keycard (texas-transfers).
  With `transfers.texas_4c` (4♣ a transfer to diamonds; no rule reads it
  yet) Gerber is off directly over 1NT.
- **The 4♣ minor slam try over 2NT** (two-nt-responses.bid, 2026-09-27)
  is off when Gerber is on: the same hands (six clubs, or five
  unbalanced, slam-invite values) ask for aces instead, then bid 6NT
  with three aces between the hands (6♣ at IMPs; at matchpoints 6NT
  scored better on par: 19 boards, -50 for 6♣), or sign off in 4NT. The
  4♦ diamond try stays. BBA does the same: 2NT–4♣ then 6♣/6♦/6NT, from 9
  HCP with six clubs (`probes/gerber-resp-2N.toml`). Its follow-ups
  (`after 2N (P) 4x (P)`) are restricted to diamonds when Gerber is on,
  so opener never reads Gerber as the try.
- **Who asks.** The hands that used to bid 6NT directly: balanced, 32
  points between the hands on opener's minimum over 1NT (17 opposite
  15-17), `strength>=slam` over 2NT. 16 opposite 15-17 keeps Rick's
  quantitative 4NT. BBA asks a point lower (31 HCP: 16 over 1NT, 11 over
  2NT, `probes/gerber-resp-1N.toml`, `gerber-resp-2N.toml`) and then bids
  6NT only with three aces between the hands; that is the `bba`
  treatment. A four-card major does not stop the ask (see "Corpus":
  2♣ → 4♣ is +647).
- **Grand slam and the king ask** (thresholds by corpus A/B, below):
  7NT with all four aces and 35 between the hands (my points plus
  opener's minimum); with 33-34, ask for kings and bid 7NT when all four
  are there, else 6NT. A long club suit does not ask for kings. Corpus
  A/B (all scenarios, vs BBA with par as the yardstick, both modules in):

  | 7NT from | king ask | vs BBA |
  |---:|---|---:|
  | 37 | none | -108,591 |
  | 37 | 34-36 | -106,518 |
  | 36 | 33-35 | -106,262 |
  | 36 | 32-35 | -106,572 |
  | 35 | none | -106,396 |
  | 35 | 34 | -106,004 |
  | **35** | **33-34** | **-105,913** |

  The textbook grand needs 37. Caution for Rick: the slam scenarios that
  carry most of this (Gerber_By_Responder, Grand_Slam_Invite) deal
  slam-rich hands by design, so the corpus may favour bold grands; the
  king ask at 33 alone lost (6NT → 5♣: -78 on 155 boards after 4♠) and
  the gain is in 7NT at 35-36.

## Evidence from BBA

- `probes/gerber-resp-1N.toml` (21GF-DEFAULT, 300 hands, 14-22 HCP, no
  four-card major, after 1NT): 4♣ from 16 HCP with a balanced hand,
  3NT below; with a five-card minor often 3♦ (its minor slam try). After
  the answer: 6NT, or 4NT with two aces missing, 7NT at 20+ with all the
  aces. It never asks for kings.
- `probes/gerber-resp-2N.toml` (300 hands, 9-17 HCP, after 2NT): 4♣ from
  11 HCP balanced, from 9 with six clubs; 7NT with all the aces from 13-14
  HCP.
- Corpus, all scenarios: 1NT–4♣–4♠–6NT 227, –4♥–6NT 152, 2NT–4♣–4NT–6NT
  99; 4NT sign-offs after an answer 65; 7NT 119; no 5♣ king ask anywhere.

## Corpus

All 343 scenarios, 2026-09-28, Smolen and Gerber together, against the
tree they were built on (`rbb compare --json`, then
`probes/tools/sideimps.py`):

- **vs BBA, par as the yardstick: -114,531 → -105,913 (+8,618)** on
  5,072 changed boards; calls agreeing with BBA 76.7% → 76.9%,
  identical auctions 16.5% → 17.1%; "no rule in a live auction" 2,835
  → 2,795.
- **IMPs to the side that changed its call: +8,930** (3,527 boards
  scored). Both yardsticks agree. The only negative openings: our 1NT
  overcall of 1♥ (-58 on 23 boards) and of 1♠ (-12), both Smolen
  (smolen.notes.md).

The Gerber part, by the first call that changed:

| Where | Boards | vs BBA |
|---|---:|---:|
| 1NT: 6NT → 4♣ | 913 | +562 |
| 1NT: 2♣ → 4♣ (a four-card major, slam values) | 474 | +807 |
| 2NT: 3♣ → 4♣ (the same over 2NT) | 411 | +828 |
| 2NT: 6NT → 4♣ | 213 | +346 |
| opener over 2NT–4♣ (was the club try) | 217 | +408 |
| 1x–1y–1NT/2NT: 3NT or 2♣ → 4♣ | 346 | +2,793 |

Scenarios: Gerber_By_Responder +1,416, Soloway_Jump_Shift_Type-3
+1,109, Grand_Slam_Invite +1,086, Puppet_Stayman_2N +417, 2N_and_MSS
+385; the worst -16 (SCS_1C_any_8plus_Resp, 6 boards).

**A four-card major does not stop the ask.** Over 1NT and 2NT the
Gerber hands with a four-card major used to bid Stayman and then 6M or
6NT; now 4♣ outranks 2♣ (equal priority, 4♣ is more descriptive) and
the 4-4 fit is not looked for. Par likes it (+807 and +828 above),
probably because nothing after Stayman checks aces. Standard practice
would be Stayman first; for Rick.

## Differences from BBA

- We ask from 17 over 1NT (BBA 16) and keep the 4NT invitation at 16.
- We ask for kings with 34 between the hands; BBA never does, and goes
  straight to 6NT or 7NT.
- BBA's 3♦ minor slam try with 16-18 and a five-card minor (we have ours
  at 3♣/3♦ with six, or five unbalanced).

## Gaps

- Gerber over our 1NT overcall; over 2NT after Stayman (2NT–3♣–3♦–4♣:
  3♣ is Puppet on some cards, so left alone).
- Interference over our 1NT at the three level (1NT (3x) 4♣): no rule.
- The card's `gerber.directly_over_nt` / `non_nt_seq` boxes;
  `gerber.over` as an enum once its values are known; a `.bbsa` mapping
  for `over_nt_seq` if BBA has one (none known: BBA's only switch is
  "Gerber only for NT openings").
- Interference over 4♣ (DOPI/ROPI) is not handled here: after a double
  the answers are as without it.

## Gaps filled 2026-10-30 (Brill convention files)

`brill_probe` / Brill's convention files listed 4♣ = Gerber where we had
no rule: 1NT (2♣) 4♣; 1♣/1♦/2♣ P 2NT P 4♣; 1NT P 2♣ P 2♦ P 4♣;
2♣ (2♥/2♠/3♥/3♠) P P 3NT P 4♣; 2♣ P 2♦ (2♥/2♠/3♥/3♠) 3NT P 4♣. All are
Gerber now on the Brill card (the 3NT ones on Brill only, above), the
first five on every card that plays Gerber. Brill's definitions were
hints; the standard sources decide (Sources).

Measured (fast lane, `--limit 50`, against the tree before):
- Gerber, Gerber_By_Opener, Gerber_By_Responder: no board changed
  (BBA's auctions never reach 1m–2NT, and our responders bid otherwise).
- All scenarios (tripwire): only Gerber calls changed. 1NT–2♣–2♦: 6NT →
  4♣ on 23 replayed boards (Slam_after_Stayman 13, Slam_after_NT 5,
  ...), 2♣–2NT: 6NT/4NT → 4♣ on 20 boards of our own auctions (3N 9,
  Grand_Slam_Invite 8, Kokish_Relay 2, NT_Ladder 1), and their
  follow-ups. vs BBA, par: -6339 → -6230 (+109); errors: -7814 → -7710
  (+104, contract +109, doubling -5); calls agreeing 144,469 →
  144,447 (BBA bids 6NT directly where we ask first).

## Open questions (for Rick)

- 1♣–2NT–4♣: Gerber (our choice, as PBS Gerber_By_Opener and Brill) or a
  natural club slam try?
- Default card, 2♣–2♦–3NT–4♣: our club slam try (as over the 3NT
  opening) or Gerber (Brill, and many partnerships)? And after their
  overcall (2♣ (2♥) P P 3NT) the default has no 4♣ at all.
- Gerber over interference: we kept it on over a double and any
  two-level overcall; some play 4♣ as a cue-bid after (2♣). Fine?

## Brill (general.system = brill, 2026-10-15)

Brill's treatment, played on the Brill card only (`cards/json/Brill.json`, `general.system = brill`: a stand-in until convention-card's `general.style` takes a `brill` value; cards/README.md "Brill"). Rusty plays the Brill dailies as South with three Brill robots, so it should read and bid what they do. Every other card is unchanged (a `.test` line on 21GF-GIB beside each Brill line; `compare --limit 50` tripwire against main: no board changed).

Gerber over a 3NT opening too (Brill, after 3N P: "Gerber" `slammish`),
from 31 HCP between the hands as over-3nt.bid counts; our default reads 4C
there as a club slam try. Brill's derived floor is lower (3+ opposite 25), so
the probe still counts an HCP difference at 4C; the answer (4D...) now
agrees.

Measured with `probes/tools/brill_probe.py report --card Brill` over the top 2000 corpus auctions (calls of this group, board-weighted, our reading against Brill's): 4C's meaning now matches; the answers 0% -> 100% of 284 boards.

## Sources

- Brill /explain, 2026-10-15 (Thorvald Aagaard's public Brill service, answers cached in `.rbb-cache/brill-service/explain/`): the rule text quoted in the `brill` section and in the `.bid` comments. Applies to the Brill card only (`general.system = brill`, cards/README.md "Brill").
- **The convention:** Bridge Bum, "Gerber"
  (https://www.bridgebum.com/gerber.php), read 2026-09-28: "An
  immediate 4♣ response to any no-trumps bid (or overcall) is Gerber";
  the step answers 4♦ 0 or 4, 4♥ 1, 4♠ 2, 4NT 3; 5♣ asks for kings on the
  same steps. Wikipedia, "Gerber convention"
  (https://en.wikipedia.org/wiki/Gerber_convention), read 2026-09-28:
  the same answers, not "when a natural club suit bid has been made".
  Where we differ: Bridge Bum also plays it over NT overcalls; we do
  not yet. The jump after Stayman's 2♦ we now play (2026-10-30); after
  a major answer we do not.
- **Gerber by opener:** PBS `btn/Gerber_By_Opener.btn` (1m–2NT–4♣ and
  2♣–2NT–4♣, citing Bridge Bum). Interference over our 1NT: standard
  practice, not yet cited (4♣ stays Gerber when it is a jump).
- **Brill's convention files** (2026-10-30 gap list from brill_probe):
  hints for where 4♣ is Gerber, not the spec. Neither page gives the 4NT
  sign-off or point counts: the sign-off is standard practice, not yet
  cited; the counts are ours (below).
- **Rick's rulings:** 17 opposite 15-17 bids slam, 16 invites
  (one-nt.bid, 2026-09-24); Gerber is on his card.
- **BBA probes:** `probes/gerber-resp-1N.toml`,
  `probes/gerber-resp-2N.toml` (21GF-DEFAULT, matchpoints).
- **Corpus measurements:** the 6♣-at-IMPs choice and the grand and
  king-ask thresholds (above).
- **Card fields:** Bridge-Classroom `conventionCatalog.js` (4♣ Gerber:
  directly over NT / over NT sequence / non-NT sequence) and the seed
  card's `slam.gerber.over = "1nt_2nt_only"`; convention-card
  `spec/fields.toml` `slam.gerber.over_nt_seq` (read, default true; not
  in `bbsa-map.toml`).
