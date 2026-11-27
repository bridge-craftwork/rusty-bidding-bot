# The lower levels: where they stand (2026-11-27)

Status of the lower-level work (BIDDING-MODEL 4.3): the bottom review, the
four follow-ups Rick asked for on 2026-11-26, and the open questions. The
first part of the review, with its full GIB tables, is
[2026-11-bottom-review.md](2026-11-bottom-review.md). The rules are on pack
branch `bottom-review` (00eed39), the engine on `freak-patterns`; both are
pushed, neither is merged, and nothing reaches `main` until the higher
levels are rebalanced.

## 1. Coverage of the GIB comparison

The positions come from our own engine bidding 100,000 random deals on
21GF-GIB: 82,790 distinct positions and 971,231 calls. The 20,000 most
frequent positions, all asked of GIB's tree, hold 89% of the calls (95% of
uncontested calls, 74% of contested ones). Within them:

| | call-boards |
|---|---|
| calls at the top 20,000 positions | 867,534 |
| off a fully agreed path | 10,062 |
| a different kind of meaning, or a meaning on one side only | 3,885 |
| compared | 853,587 |
| … with no range difference | ≈628,100 (74%) |
| … with a range difference | 225,477 (26%) |
| … with a range difference, `general.style=gib` | 214,315 (25%) |

**A blind spot.** The positions are the ones our own engine produces, so a
position that arises only when the opponents play a different system is
never charted. Brill daily 2026-11-26 board 1 is one: P (1♣) X (2♥ weak)
P (P) ?. Our responders treat a jump after a double as fit-showing and
never pass it out, so the position is absent from the 82,790, and the
doubler had no rule there (section 4).

## 2. What the bottom review changed

| run | positions | call-boards with a difference |
|---|---|---|
| before | top 5,000 | 350,431 |
| after the first pass | top 5,000 | 181,255 |
| before the second pass | top 20,000 | 230,784 |
| after the second pass | top 20,000 | 225,477 |
| … with `general.style=gib` | top 20,000 | 214,315 |

The details of both passes are in the bottom-review report. What is left is
mostly not missed limits: passes whose stronger calls did not exist yet
(now partly added, section 3), opener's passes after game (higher-level
work), our style (now measurable with the gib style), and least lies, which
are not widened by design (BIDDING-MODEL 3.6).

## 3. The four follow-ups (2026-11-26), all merged into `bottom-review`

**Balancing over their fit, and borrowing a king** (BIDDING-MODEL 3.8).
Over (1x) P (2x) P P and (1x) P (1y) P (2y) P P: a double (singleton or void
in their suit, 8+ HCP, the unbid suits), a natural suit (five cards; at the
three level also shortness), 2NT for the two lowest unbid suits. Advancing
a reopening double now runs three points higher than in the direct seat
(cheapest suit 0-11, jump 12-14, 1NT 10-13, 2NT 14-15, 3NT or cue 15+), and
the reopening doubler's rebids three lower. The any-shape reopening double
starts at 16. On 100,000 random deals the balancing side's contract errors
improve by 1,170 IMPs (+0.70 a board over 1,682 boards; both halves agree).
With a doubleton in their suit the double lost, so the over-fit calls need
shortness and fire on only 239 boards.

**Fourth seat after (1x) P (1NT).** Two-level overcalls (six cards, or five
with 16-18), weak three-level jumps, a takeout double with shortness in
their suit, Michaels 12-17, and advances. The lighter values of GIB and the
books lost 407 IMPs on the acting side's contract errors (the light calls
gained on double dummy only because nobody doubled them); the tightened
version gains 27.

**`general.style=gib`.** Five-card majors in every seat; weak twos with no
side four-card suit except clubs. The 10,700 call-boards these two caused
leave the difference list; nothing else moved by more than 110. GIB's
power-double suit is 18+ total points, like ours. The style's items, with
our value, GIB's, the source and a future setting name, are in the pack's
`settings/styles.md`: the map of what will need its own card setting.

**Help-suit game tries.** Opener tries the cheapest suit that needs help,
and partner reads that cheaper suits did not; responder without help there
can show help in a higher suit below 3M. A precedence bug in `helps(x)`
(only "four cards with an honour" counted) was fixed.

## 4. The range probe: GIB as a check, not a target

680 deals: responder after 1♠ (P) and 1♥ (P) with 3-14 HCP in five shapes,
and opener's rebid after 1♦ (P) 1♠ (P), bid by BBA, Brill, WBridge5 and us,
with GIB's stated ranges beside them (`probes/range-probe/REPORT.md` on
branch `range-probe`). The bidding robots agree within about one HCP on the
plain boundaries (pass/1NT 5/6, 2/1 from 12-13, simple raise to 8-9, limit
raise 9-11, 1NT rebid 11/12-14, 2NT rebid 18-19, reverse from 16-17); they
split on treatments. GIB's stated HCP is usually a point above what BBA and
Brill bid.

We are below every robot's floor in three places, each from length counted
into points: the invitational jump shift from 7 HCP, the 3NT response from
11, and a game-forcing 2♣ with three trumps on 10-11. Where we differ only
from GIB and sit inside the robots' spread (Jacoby 2NT from 10, the 1NT top
at 12, opener's 4♠ from 16, the 3♦ jump rebid from 14, the 10-12 splinter),
nothing needs to move. The thresholds set to GIB's numbers on
`bottom-review` changed 11 of the 680 calls.

## 5. The Brill board (2026-11-26, board 1)

South QJ86.QJ.AKQJ5.K9 (19 HCP): P (1♣) X (2♥, Brill: 6+ hearts, 6-9) P
(P) ?. We passed; a second double (takeout: spades and diamonds, extra
values) is right, and GIB's tree reads it so (4 spades, 4-5 diamonds, 17+
total points). Not covered: the doubler's rebid rules stop at partner's pass
in the pass-out seat (`!rho.last=P`), and only their raise, (1x) X (2x) P
(P), has a rule. On the compiled-in rules the double would even be read as
penalty. Being added (pack branch `doubler-rebid`).

## 6. Open questions for Rick

From the range probe:
1. HCP floors beside the point counts: 9+ for the invitational jump shift,
   12-13+ for 3NT and for the game-forcing fit bid?
2. The three-card raise with 10-11 HCP: BBA's 3M, Brill's 1NT then raise,
   or ours without the 3NT and 2♣ parts?
3. Keep 1NT with three trumps and 6 HCP (GIB allows it; BBA and Brill
   raise)?

From the gib style:
4. A GIB position with the power double at 17+? The two looked up say 18+.
5. GIB's 11 HCP floor for one of a suit under the gib style?
6. Responder's counter-try (8-9 support points) and opener's game with 17:
   settle by par?

From the fourth seat:
7. Keep our calls tighter than GIB and the books, since the light ones lose?
8. Tighten the weak three-level jumps (−19 IMPs over 34 boards)?
9. The opening side's answers to these new calls (penalty doubles, answers
   to the Michaels cue)?

From balancing:
10. The balancing suit is 8-15, only two below the direct 8-17. Drop its
    floor?
11. The singleton-or-void rule for the balancing double: it balances far
    less often than the 24,000 passes there suggest.
12. The reopening doubler with 14-16, no fit and no five-card suit has no
    rebid and passes.

Already ruled (2026-11-26): par decides Drury opener's 4♠ floor and the 4♥
over the cue-bid raise; the role-aware `tp` stays a to-do; help-suit tries
prefer the lowest suit. The work queue is BIDDING-MODEL section 8.
