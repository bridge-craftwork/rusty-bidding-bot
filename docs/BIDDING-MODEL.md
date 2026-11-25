# The bidding model: decisions and their basis

What the engine and the rules believe about a partnership's hands and
roles beyond what single conventions say: who decides, what a call's
range is, how slam values are counted, what controls are known. Each
entry gives the decision, who made it and when, and the evidence, so a
later change can tell a settled point from a guess.

Status words: **decided** (Rick ruled), **measured** (an experiment
settled it), **proposed** (not yet ruled or built), **open** (a question).
Related: [JUDGMENT-LAYER.md](JUDGMENT-LAYER.md) (the top-down layer and
its first decisions, 2026-09-30), [DESIGN.md](DESIGN.md) ("par decides,
BBA teaches").

## 1. Why this document

Slams are Rusty's costliest weakness: on 1,000 random deals bid by
WBridge5, Rusty lost 2.52 IMPs a board on slam deals against WBridge5's
auctions (memory: WBridge5 probes, 2026-11-24). Rick (2026-11-25): the
conventions are in decent shape; what is missing is the fundamentals
for *using* what the calls convey: who is captain, what each call's
range really is, and what is known about controls. Decisions on those
fundamentals live here.

## 2. Captaincy

**Decided (Rick, 2026-11-25).** The captain is normally the partner of
the player who limited their hand first. The captain sets the strain and
the level, sometimes by asking partner a question. The limited hand
describes and answers; it does not decide, and **it never makes a slam
move on its own**.

- A pass limits a hand (GIB: "No suitable call -- 11- HCP; 12- total
  points"), and so do a limit raise, a notrump bid with a range, a
  preempt, a weak two and a sign-off.
- Example: P–P–1♠–P–2♣ (Drury)–P–4♠. Responder limited their hand
  twice, by passing and then by Drury (10-12 support points). Opener is
  captain; responder must not bid 4NT, whatever the count opposite
  opener's top says.

**Earlier decision (Rick, 2026-09-30, JUDGMENT-LAYER.md "Decisions"
2):** define `captain` from the width of each partner's shown range
(about four points or less), not from a per-call attribute; the asker is
captain after partner answers.

**What exists** (`crates/engine/src/eval.rs`, `"captain"`,
`CAPTAIN_WIDTH = 3`): I am captain when partner's whole-point range is
four points wide or less and mine is wider, or partner has answered my
question. It is recomputed at every call. Only two rules read it,
both in the notrump game-or-not logic (responder-rebids); no keycard
ask or control bid does.

**Gaps against the 2026-11-25 ruling:**
1. A top alone does not count as a limit: a passed hand (0-11 HCP) is
   eleven points wide, so it never makes partner captain.
2. It is not sticky. "Who limited first" is history; width is
   recomputed and can swap when a later call narrows the other hand.
3. The slam rules ignore it. In the Drury example the width rule would
   have made opener captain (responder's support points 10-12), but
   `rkcb-1430.bid:69` asked from responder's seat anyway.

**Proposed:** captaincy as side state, set by the first call that
limits a hand: a pass, a call whose range has a top within reach of the
rest of the auction, or the answer to a question. The partner of that
hand is captain until an ask and its answer hand the role over. Slam
moves (keycard asks, control bids, quantitative raises, grand-slam tries)
are made by the captain; the limited hand only answers and accepts or
declines invitations. To rule on: does a passed hand stay limited after a
fit jump or Drury shows a maximum pass? Is opener limited by a 1NT
rebid but not by a new suit (2026-09-30 says opener's new suit,
normally 12-18, does not limit yet)?

## 3. Ranges: floors, tops, and where tops come from

### 3.1 Counting partner's floor, not top

**Measured (2026-11-24/25).** The keycard ask facing a non-forcing
partner (`rkcb-1430.bid:69`, `slam_values_limited`) counted the
partnership's points with partner at their *maximum* and needed the
own hand to have four losers or fewer. Rick: the losers test "doesn't
make any sense" (2026-11-24). Removing it, on 100,000 random deals (the
BBA 21GF set), scored against par:

| variant | auctions changed | IMPs a board | slams bid that make | slams that fail |
|---|---|---|---|---|
| base (losers ≤ 4) | – | – | 2,712 | 797 |
| losers test removed | 6,218 | −0.217 ± 0.007 | 3,800 | 3,496 |
| width ≤ 4 instead of losers | 329 | −0.005 | | |
| losers or width ≤ 4 | 168 | −0.003 | | |
| floor, or top when partner's range is narrow (≤ 5), no losers test | 1,261 | −0.012 ± 0.003 | 3,145 | 1,198 |
| same, and the fit ask (`fit_slam_values`) on the floor only | 836 | +0.002 ± 0.003 | 2,983 | 966 |
| same, and the ask needs an eight-card fit | 749 | **+0.004 ± 0.003** | 2,944 | 922 |

Without the losers test, 2,323 of the bad asks came from that one rule,
and most went wrong for want of values (combined 22-28 HCP). Only 17% of
the extra failing slams had a side suit neither hand controlled; 19%
were missing two keycards.

Rick's reading of the two typical failures (2026-11-25):
- P–P–1♠–P–2♣(Drury)–P–4♠, asked by ♠J742 ♥2 ♦Q52 ♣KJ543 (7 HCP):
  opener's 4♠ shows game values and no interest; the Drury hand is
  limited and never asks (section 2).
- 1♦–1♥–4♥, asked by ♠K75 ♥9753 ♦6 ♣KJT64: opener shows 19-21, and 7
  HCP opposite that is not enough for slam.

**Principle (proposed, from the measurement):** count partner on their
floor. Count partner's top only when their range is narrow, and only
when the call that set the top was a real limit (not a catch-all, 3.2).
The losers test goes. The variant on the last line of the table is
in a local worktree, not committed; captaincy (section 2) may make most
of it unnecessary, and is to be measured first.

### 3.2 Tops come from stronger alternatives

**Finding (2026-11-25).** A call's top is inferred from the calls a
stronger hand would have made instead. Where a module offers no
stronger alternative, the catch-all call keeps an open top. After
P–P–1♠–P–2♣ (Drury), our opener has 2♠ (12 or less), 2♦ (13-14) and 4♠
(15+), plus natural rebids, and no slam try. So our 4♠ reads 15-31
support points. GIB's tree has 3♠ (19-22, forcing), three splinters
(18-22), 4♦/4♥ voids (18-22) and 3♣ (13-14), so its 4♠ is 14-18.

Consequences:
- partner counts a maximum that is not there (the bad keycard asks);
- a strong hand has no way to show its extras, so slams are missed
  from the other side too.

**Rick (2026-11-25), the tops these calls should have:**
- **Drury 4♠** does not go up to 28: with that much opposite a Drury
  bidder's 10 or more, opener looks for slam. Its top is where opener's
  slam try starts (GIB: 4♠ 14-18, 3♠ 19-22 forcing).
- **Splinter by responder** (1♠–4♣) is normally 10-12 HCP, 13-15
  support points: a limited bid. With more, responder uses Jacoby 2NT,
  or without it starts with a 2/1 and supports at the three level (no
  fast arrival). Ours reads 11-15 HCP.

**Proposed:** every module that ends in a game bid should either give it
a real top, or the engine should read a catch-all game bid on its floor.
Then the system needs the stronger calls (slam tries) that make the top
true. The GIB comparison (section 4) lists the open tops.

### 3.3 Which count each hand uses

**Decided (Rick, 2026-11-25).** The hand with the long trumps counts its
own points (HCP plus length), never support points; support points
(shortness counted as dummy) belong to the hand with fewer trumps.

**Finding:** our rules break this in many places. `tp(x)` is one
measure, "support points, shortness capped by trumps held", whichever
hand holds it. The pack uses it about 170 times, across Drury, Bergen,
Jacoby 2NT, jump raises, game tries, the base rebids and slam entry.
After P–P–1♠–P–2♣ (Drury), opener's rebids are written as
`tp(M)<=12`, `13..14` and `>=15`: the long-trump hand counting
shortness. Slam entry already tells the two apart (`count_long`: HCP
plus length for the longer hand). The calls do not, so what partner
reads is in the wrong unit, and our declarer points stay unnarrowed:
after opener's 4♠ the engine knows 10-27 declarer points, where GIB
says 14-18 total points.

**Proposed:** go through the `tp(x)` uses by who holds the long trumps.
A rebid by the hand that showed the suit first (opener after a raise,
responder after opener raises a major) counts HCP plus length. The raiser
keeps support points. Where a rule can be either hand, the engine could
give the term both meanings (`tp(x)` as declarer points when the hand
has shown more trumps than partner). That is a language question for
CONTRACT.md before it is built.

### 3.4 Openings have tops in points, not only HCP

**Decided (Rick, 2026-11-25).** A one-level opening cannot hold 31
points: with 22 or more the hand opens 2♣. GIB: 1♠ opening 11-21 HCP,
12-22 total points.

**Finding:** our 1♠ opening reads 10-21 HCP, with declarer points up to
27 and support points up to 31. The strong 2♣ shows `hcp>=22`, or
`hcp>=17, losers<=4, hcp+length_points>=23` for an unbalanced hand
(strong-openings.bid). The engine keeps ranges, not disjunctions, so
it cannot draw a points top from "not that" (the losers term blocks it).
Every count opposite an opening (partner's top in 3.1, the slam counts)
inherits the open top.

How the 31 arises after P–P–1♠–P–2♣–P–4♠ (`knowledge.rs`,
`support_bonus` and the `tp` narrowing): support points are bounded by
the HCP top plus the most shortness the lengths allow, capped at the
trumps held. Opener's HCP top is 21, because 2♣ takes 22+. Spades are
known 4-10 and the side suits 0-9, so two voids are allowed: 5 + 5 = 10
shortness points, capped at 10 trumps, for 21 + 10 = 31. Four things
stack up:
1. HCP and lengths are kept as separate ranges, so the HCP top and the
   shortness top are added although no hand holds both: a hand with ten
   spades and two voids holds at most 19 HCP.
2. Nothing caps opener's length (a 1♠ opening "could" hold ten spades).
3. The long-trump hand is counted in support points at all (3.3).
4. 4♠ itself says only "15+" (3.2), and the opening has no points top
   (above).

**Rick (2026-11-25):** the 1♠ opening is normally 12-21 HCP, and not
enough for a 2♣ opening by tricks. With five spades and 25 declarer
points the hand would almost always open 2♣.

**Tried (2026-11-25, pack worktree `points-tops`, not merged), on the
100,000 random deals with the freak-pattern engine:**
- the one-level openings show `hcp+length_points<=22 | hcp<=18`;
- a new 2♣ "23+ declarer points" (19+ HCP, unbalanced);
- Drury 4M becomes 15-18, with a new 3M slam try at 19+;
- responder's splinter becomes 10-12 HCP.

Together: 881 auctions changed, -931 IMPs from par, and the bidding
side's contract errors -1,306. Grouped by the first call that changed:
- the Drury 3M slam try, -666: after 3M–4M opener still asks for
  keycards, through `slam_values`' "18 support points of my own"
  branch, which counts the long hand in support points (3.3);
- the quantitative 4NT over 1m–2♠ (strong jump shift)–3NT, -350:
  opener's new 20 top lets more responders pass
  `hcp + partner.hcp.max <= 35`, and 4NT goes down;
- the splinter, -18; Jacoby 2NT +11; the new 2♣ +60.

The tops themselves read right (1♠: decl 12-22; Drury 4♠: 15-18;
splinter: 10-12 HCP). The losses are in the rules downstream, which
were tuned on the open tops. Next: give each part its own measurement,
and before the Drury slam try, settle its follow-up (opener as the
captain, counting declarer points).

**Rick (2026-11-25): the bottom first, then rebalance the top.** The
higher bids (slam thresholds, quantitative 4NT, the Drury follow-ups)
must be rebalanced for the narrower ranges at the bottom. Before that,
review the early rounds more thoroughly against GIB (section 4.2).
GIB's own Drury continuation is a candidate to adopt. After
P–P–1♠–P–2♣–P–3♠ (3♠: 19-22, forcing), responder bids:
- 4♠ with 10-12 total points;
- 3NT: 3-4 spades, 9-11 HCP, partial stoppers;
- a cue bid of an ace (4♣/4♦/4♥) with 11-12 total points;
- 4NT Blackwood with 12.

**Proposed:** the one-level openings state their points top outright
(`hcp+length_points<=22` or similar, checked against the strong 2♣
rule so no hand is left without an opening), measured on random deals.

### 3.5 Ranges cover the normal hands; freaks bid outside the system

**Decided (Rick, 2026-11-25).** A call's ranges should cover 99% or
99.9% of the hands that make it, not every hand that could. Bidding
systems are not built for super-freaks, and conventions don't work with
them, so freak hands must not widen what partner reads. A freak bids
outside the system: a high preempt (4M, 5m), or 2♣ when it is strong,
whatever its points. Rick's example: 5=8=0=0 with 5 HCP might open 4♥,
but 4♥'s meaning must not include that pattern; it may be worth far
more if partner bids spades.

Background: a ten-card suit is 1 hand in 243,000 (1 in 61,000 in any
suit), and 10-3-0-0, the shape behind the 31 in 3.4, is 1 in 2.6
million. Yet the engine sets a top from any length it cannot rule out.

**What defines a freak (decided for a first version, Rick, 2026-11-25):** by how common the shape is, with the line at 99.9% as the card's default.
Rank the 39 hand patterns by frequency; the patterns past the
coverage line are freaks:

| coverage | the last normal patterns | freaks (share of hands) |
|---|---|---|
| 99% | … 7-4-1-1, 7-4-2-0 | 7-3-3-0, every 8+ suit, 7-5-1-0, 6-6-1-0 and rarer (0.96%) |
| 99.9% | … 7-5-1-0, 6-6-1-0 | 8-4-1-0, every 9+ suit, 7-6-0-0, 8-5-0-0 and rarer (0.09%) |

At 99.9%, no suit is longer than eight and there is at most one void
unless the hand is a freak. Rick's 5=8=0=0 (8-5-0-0, 0.003%) is a freak
either way.

At 99%, 8-2-2-1 and 7-3-3-0 would be freaks, although they preempt
normally; 99.9% looks the better default.

A second test may be needed for hands whose playing strength is far
from what their HCP say (Rick's 5-HCP 5=8=0=0 has about four losers).
That could be losers against points, or the count of tricks.

**Proposed:**
1. **Engine:** a `freak` term (the pattern is outside the coverage set,
   a card setting defaulting to 99.9%). A seat that has not shown a freak
   is read as not holding one: the shortness and length tops come only
   from normal patterns (`vertex_max` skips freak patterns), and so do
   the points tops that follow from them.
2. ~~**Rules:** a freak module whose calls show `freak`.~~ **Ruled
   out (Rick, 2026-11-25):** no call is known to show a freak. An
   8-5-0-0 may open 4♠, but that is advertised as a preempt, which
   could as easily be 8-3-1-1; partner reads it as a preempt. Freak
   hands borrow normal calls and the meanings stay normal. So there is
   no `freak` term for what a call shows. A hand term for choosing the
   call may come later, if a rule needs one.

   Rick, restated: with a freak the bidder chooses a call whose
   description won't match the hand but is the best bid anyhow, and
   partner and the opponents take it in its normal meaning. Gambling
   3NT is the one convention built for a freak, because that hand is
   common enough to earn one; other freaks find nothing that matches
   or describes them as well.
3. **Measure:** how often freaks occur in our random sets (about 0.1% of
   hands at 99.9%), and the effect on the ranges partner reads. The GIB
   comparison (section 4) should then show fewer open tops.

**Built, engine part (2026-11-25; branch `freak-patterns`, not yet
merged):** `knowledge.rs` holds the 25 normal patterns
(`NORMAL_PATTERNS`, through 6-6-1-0) and `is_freak`. A seat's lengths
and the counts they allow (length points, support-point bonus) come from
the length vectors of normal patterns only, each counted exactly rather
than at the corners of the length ranges. A seat is read as holding a
freak only when its calls together leave no normal pattern (seven
spades, then six hearts): its constraints are then read again with every
pattern allowed. No call shows a freak (Rick's ruling under 2).

What partner reads now: no suit longer than eight and at most four
length points until shown otherwise. The tops below are only what the
pattern arithmetic leaves; they are not the system's tops, which come
from the stronger alternatives (3.2, 3.4 and Rick's tops there). 1♠ opener reads decl 12-25 (was
12-30); the Drury 4♠ reads ♠sup 15-28 (was 15-31); the strong 2♣
(`hcp+length_points>=23`) reads 19+ HCP (was 17+); a splinter 15-22
support points (was 15-23).

Measured on the 100,000 random deals (against the balanced fix): 358
auctions changed. Distance from par +256 IMPs (+0.72 ± 0.25 a changed
board). For the side that bid the change: contract errors +144 and
doubling errors +608 (both halves agree). Most of the doubling gain is
penalty doubles of the opponents' overcalls and competitive bids, now
that their tops are known. Slams bid with a slam par went from 2,712 to
2,721; failed slams went from 797 to 798. In the GIB comparison,
call-boards with any difference fell from 441,390 to 310,162, and "open
top" in points fell from 326,897 to 180,961.

**Side effect on the slam thresholds.** The exact counts also tighten
deductions that have nothing to do with freaks. Every hand with four
spades and four clubs has a doubleton, so after 1♠–2♣ (which denies
four spades with 13+ support points) responder is read as holding at
most three spades. In 1♠–2♣–2♠–3♠ (Lawrence style) responder then
reads as 13+ HCP and 16+ support points (was 12+ and 12+). The long
hand's slam count, `slam_count` at 30 on partner's floor, was
calibrated (2026-10-01) when that floor was understated. Five
`slam/serious-3nt.test` cases now fail:
- a 13-14 HCP opener asks for keycards instead of bidding the
  non-serious 3NT;
- the 3NT reading itself contradicts the narrower ranges, so it shows
  nothing, and responder then signs off where the test expects a
  control bid.

The pack's slam thresholds need recalibrating against the new floors
before this engine change is released.

### 3.6 A call's description is narrower than its licence

**Decided (Rick, 2026-11-25).** ACBL lets a 1NT opening hold a
singleton A, K or Q, and players open 1NT with a six-card minor
(usually 6-3-3-2). Those hands may make the call, but the call is still
described, and partner still plans, as at least two cards in every suit
and at most five (2-5 each). This is 3.5's principle again: the rare
hand that bids within the licence does not widen what partner reads.
Our 1NT rules show `balanced` or 5-4-2-2 with honours in the short
suits, and partner reads 2-5 each (4.1, "Built").

### 3.7 What "total points" means in the GIB comparison

GIB states HCP and total points: HCP plus length for the long hand,
shortness for the hand with support. Ours has hcp, declarer points
(`dp`, "decl"), support points per strain (`tp`, "sup") and `points`.
The comparison in section 4 pairs GIB's total points with our support
points in the agreed suit, for the hand that has shown fewer trumps than
partner, and with our declarer points otherwise (3.3).

## 4. Checking ranges against GIB's bidding tree

**Decided (Rick, 2026-11-25):** use GIB's bidding tree to check our
readings for large differences. One request a second is fine. Archive
the answers, so the same position is never asked twice.

- **Source:** BBO's GIB robot,
  `https://gibrest.bridgebase.com/u_bm/u_bm.php?s=P-P-1S-P-*&t=g`
  (the service behind `https://netbridge.dk/gib.html?bid=P-P-1S-P-*`).
  For an auction so far it returns every call the next player can make,
  with its meaning, as XML. The meanings are cumulative: a call's
  line is all the caller has shown.
- **Tool:** `probes/tools/gib_tree.py`.
  - `show P-P-1S-P-2C-P`: one position, GIB's meaning beside ours for
    every call, and the calls GIB has where we have no rule.
  - `run --compare ALL.json --top N --tag NAME`: the positions our
    engine bid from, ranked by boards. It fetches the missing ones,
    then reports the differences grouped by our rule.
- **Archive:** `.rbb-cache/gib-tree/xml/<auction>.xml` in the main
  checkout, shared by worktrees, with `index.jsonl` recording when each
  was fetched and the tree version (`gv`). A rerun resumes; nothing is
  fetched twice.
- **Reading a difference:** "wide" means ours says less than GIB's (an
  open top, a lower floor, a looser length). That is where our model is
  missing knowledge. "Narrow" means ours says more. That is often fine:
  we infer from what partner did not bid, or our rule really is
  narrower. GIB plays its own 2/1 card (5-card majors in third seat, a
  forcing 3♠ after Drury), so judge each difference; don't copy GIB's
  ranges. Our readings use 21GF-GIB, our GIB approximation.

### 4.1 First run (2026-11-25)

The 2,000 positions our engine bid from most often on the 100,000 random
deals: 1,995 fetched in 34 minutes, no failures; 7,657 calls compared
(21GF-GIB). Weighted by boards, the differences where ours says less:

- **Points tops missing almost everywhere.** All four one-level openings
  (about 70,000 boards) read 12+ with no top, where GIB says 12-22 (3.4).
  Our passes are worse:
  - "No opening bid" allows up to 19 declarer points against GIB's 12-;
  - "No overcall" and "No further bid" have no top at all (GIB: 16-
    over a one-level opening, 20- after a response).

  Passes are the commonest limits (section 2), so captaincy needs these
  tops.
- **A passed hand's later calls forget the pass.**
  - P–P–1♦–P–1♥: ours 6-15, GIB 6-12;
  - P–P–1NT–P–2♣ (Stayman): ours 6-15, GIB 12-.
- **"Balanced" does not narrow the suit lengths.** After 1♣–1♠–1NT
  (12-14 balanced) we read clubs 3-13, hearts and diamonds 0-6; the
  1NT opening narrows no length at all. Balanced means no suit under two
  and none over five (six where the card allows 6-3-2-2). Without
  it, every count opposite a notrump bidder allows voids and singletons,
  so shortness and support-point tops are open. This is a cheap engine
  fix with a wide effect.

  **Built (2026-11-25):** the cause was `shape 5-4-2-2` (and the other
  patterns) narrowing nothing. Every notrump rule reads
  `balanced | shape 5-4-2-2`, and the hull of "2-5 each" with
  "anything" is anything. A positive `shape` pattern now narrows each
  suit to the hull of the length vectors it allows within the known
  ranges (`knowledge.rs`, `shape_hull`). After 1NT the opener reads
  2-5 in every suit; after 1♣–1♠–1NT, ♠2-3 ♥2-4 ♦2-4 ♣3-5.

  Measured on the 100,000 random deals: 164 auctions changed,
  +144 IMPs (+0.88 ± 0.42 a changed board, +0.001 a board overall).
  Almost all of the change is responder bidding game in a now-known
  eight-card major fit after opener's 1NT rebid. The GIB comparison's
  "length floor wide" fell from 109,476 to 97,639 call-boards.

  One pack test now fails for that reason, `base/fit.test:19`
  (KQ84 AJ9873 A2 4 after 1♣–1♥–1NT: expected 2♠, now 4♥, game in the
  known heart fit). The measurement favours the new call. **Rick
  (2026-11-25):** with slam in view, look for a 4-4 spade fit before
  settling in hearts, since a 4-4 fit allows ruffs in either hand and
  can add a trick. After the 1NT rebid, slam doesn't look likely, so the
  jump to 4♥ is probably best. The test changes to 4♥ with the engine
  release, with that reason. The 2♠ search belongs to slam-going hands.
- **Shape lost in either-or meanings.** Our takeout double ("the unbid
  suits and 12+, or any strong hand") narrows no length, because a range
  cannot hold the "or". GIB's double shows 3-5 in each unbid suit and 2-
  in theirs.
- **Smaller:**
  - weak twos allow side voids (GIB: one or more in each side suit);
  - one-level overcalls state total points but leave the HCP floor at 0;
  - 1♦–1♥–1♠ reads up to 21, GIB up to 18.

The report and its JSON are in `.rbb-cache/gib-tree/report-random100k.*`.

### 4.2 The early rounds as a base (Rick, 2026-11-25)

Find the most common auction positions (1,000, maybe 5,000), measure
what share of all calls they cover by call number, and check how deep
they reach. Then compare GIB's ranges with ours there, and get those
early rounds solid. Probing GIB over time to grow the archive is fine.

GIB plays many treatments we don't, some of them uncommon. The
comparison keeps only calls with basically the same meaning, at
positions whose whole path agrees. The rest go to a GIB-treatments
queue, ranked by frequency, from which rules may be added later to see
how well we track GIB. Being built by an agent: `gib_tree.py coverage`
and the agreed-path filter.

## 5. Controls

**Decided (Rick, 2026-11-24):**
- Track controls per seat, for both sides, the defenders included:
  - from first- and second-round control bids;
  - from shortness shown by splinters, Jacoby 2NT shortness answers
    and the like;
  - from keycard answers.
- Track what each call promises and what it denies. A defender can see
  which suit is unprotected and lead it.
- When neither partner controls a suit, slam is off.
- Show controls when there is room; high in the auction not all can be
  shown.
- Control bids above game normally show first-round control.
- Three styles, a card setting:
  - first-round only;
  - first or second round (the common modern style);
  - traditional: a suit's first control bid shows first-round control,
    a second one in the same suit shows second-round.
- Control-bid on slam *interest*, especially while partner's strength is
  unknown. After Jacoby 2NT and opener's shortness answer, responder may
  cue in case opener has 21.
- Not every slam needs control bids. Slam-likely hands with an
  uncontrolled side suit are rare: Rick's dealer scripts found few, and
  17% of the failing slams in 3.1 had one. Cueing is good practice,
  not a gate.

**Serious and Non-serious 3NT (decided, 2026-11-24):**
- Serious 3NT: 3NT shows serious slam interest; bypassing it to cue
  shows mild interest.
- Non-serious 3NT, which Rick plays, reverses this: 3NT shows mild
  interest, and bypassing it to cue is serious.
- Either way, control bids below 3NT are free and promise no extras.
- Rusty must play both, and neither. The pack has
  `serious_3nt/convention.toml` (proposed) with no rules yet.

**Splinters (decided, 2026-11-24):**
- The singleton treatment is a setting with three options:
  - any singleton, the ace included (common);
  - no singleton ace;
  - no singleton ace or king.
- Nobody bars a singleton queen or jack. The pack's `spl_short` (no A,
  K, Q or J) is wrong on every option.
- A later control bid in the splinter suit shows a void.

**Priority (Rick, 2026-11-25):** control bids matter, but they are a
small part of the slam loss. Valuation and captaincy come first.

**What exists:** `cued(x)` and `denied(x)` bitmasks from
`sets ladder=control` (position.rs), and `control-bids.bid` (majors only,
entered with slam values and an uncontrolled side suit). Not tracked yet:
`partner.keycards`, `partner.controls`, `partner.losers`,
`partner.quick_tricks`. `slam_try` and `grand_try` are placeholders
(combined HCP 31 and 35).

## 6. Evidence and yardsticks

- **Random deals, scored against par**, are the verdict for valuation
  and slam changes. Use the 100,000-deal BBA 21GF set
  (`.rbb-cache/random/pbs`) and the 1,000 Pavlicek deals bid by WBridge5.
  Count slams bid that make and that fail, by par band.
- **WBridge5** (memory: WBridge5 probes): its behaviour can be probed,
  not its internals. It cues whenever it holds a control below 4NT and
  bids 4NT only without one.
- **Brill:**
  - its slam set (199 deals) is biased toward deals where slam makes,
    so it flatters any change that bids more slams;
  - the example hands in its bid files (`Board: N, Hand: ...
    [MS=...]`) reference 29,710 boards, none of them in our dailies;
  - its slam primitives (combined ranges per strain, keycard state,
    loser cover, trick estimation) are a list of ideas, not a
    specification.
- **GIB's tree** (section 4) is for ranges, not judgment.

## 7. Open questions for Rick

1. Captaincy details (section 2): passed hand after a maximum-showing
   call; opener's new suit; the hand-over after an ask.
2. Catch-all game bids (3.2): give each a top in its module, or have
   the engine read any game bid without a stated top on its floor?
3. Which control style is the default for 21GF-DEFAULT, and does the
   splinter default stay "any singleton"?

## Sources

- Rick's rulings, 2026-11-24 and 2026-11-25 (this session's
  conversation; memory files `controls-and-serious-3nt` and
  `captaincy-and-gib-tree`), and 2026-09-30 (JUDGMENT-LAYER.md).
- Measurements: the compare runs of 2026-11-24/25 (pack branches
  `slam-no-losers-gate`, `slam-gate-w4`, `slam-gate-w4or`,
  `slam-limited-floor`..`floor4`, local only), on the deal sets in
  section 6.
- GIB: BBO's published bidding tree via `gib_tree.py`, archived.
- Bridge practice: captaincy, control bids and Serious 3NT are
  standard practice, not yet cited to a book.
