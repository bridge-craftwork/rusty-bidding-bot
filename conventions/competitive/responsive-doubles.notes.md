# Responsive doubles (`responsive-doubles.bid`): notes

Partner doubled or overcalled, RHO raised, and our double is for takeout.
Card: `doubles.responsive.play` — on for the 21GF cards, off for
Basic-Bridge. Cases: `responsive-doubles.test`.

## Guidance (Rick, 2026-09-22)

- after partner's **takeout double** the double shows **4-4 in the
  majors** when they have bid a minor, and **4-4 in the minors** when
  they have bid a major;
- after partner's **overcall** it shows **the other two suits**, with
  tolerance (usually Hx) for partner's suit;
- advancer needs **6+ total points at the two level, 8+ at the three,
  10+ at the four**.

Partner answers by naming the suit he is longer in. The double `sets
ask=responsive` and the answer is written `when asked responsive`, so it
is one set of rules whatever the level.

## Evidence, and where we did not follow Rick

After partner's **overcall** BBA's meaning is "Responsive double, 11 to
37 total points, 4 to 5 cards in each unbid suit, 0 to 2 cards in
partner's suit". Rick's 6+ at the two level doubles about three times as
often as BBA does, and it measured badly: Double_by_Advancer lost 2.4
points of call agreement, Maximal_After_Overcall 1.5.

So this half uses **10+ at the two level and 11+ at the three**, with
exactly a doubleton in partner's suit and at most three of theirs, and
Rick's 6/8/10 ladder stands for the double after partner's **takeout
double**, where partner has promised all three unbid suits and advancer
can afford to be lighter. **This is the one place where the rules do not
follow a ruling of Rick's, and it should be settled.**

The double also denies three-card support for partner's overcall: with
three we raise. Without that it outranked the raise and cost 2.4 points
in Double_by_Advancer alone.

| scenario | before | after |
|---|---|---|
| Responsive_Double | 72.2% | **74.1%** |
| Responsive_Double_after_Overcall | 76.9% | **79.1%** |
| Double_by_Advancer | 73.5% | 71.6% |
| Maximal_After_Overcall | 69.8% | 69.6% |

Double_by_Advancer is the one that still loses: BBA passes many 10-11
counts with 4-4 in the unbid suits that our rule doubles. Its own stated
meaning says it should double them, so this is BBA's judgment, not its
system.

## Gaps and open questions

- The responsive double after they bid a **new suit** rather than
  raising, and over a preemptive raise to the four level.
- **Answering** the double is a plain "bid your longer suit": no strength
  ladder, so a strong advancer cannot invite or force.
- `doubles.responsive.through` (a text field) is not read; the rules
  cover two- and three-level raises and stop there.
