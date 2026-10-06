# Meckwell (`meckwell.bid`): notes

Meckwell (Meckstroth-Rodwell) over their 1NT, direct and balancing
seat. Cases: `meckwell.test`. Shared answers: `vs-1nt.bid`; the
comparison of the defences: `vs-1nt.notes.md`.

## Guidance (Rick, 2026-09-28)

Implement it so that cards that play it are played as written. The card
turns it on with `competitive.vs_1nt_strong.system =
meckwell` (Bridge-Classroom's seed card, `21_intermediate_card.json`,
does). No `.bbsa` key: BBA has no Meckwell, and played Multi-Landy in the
scenario `Meckwell` (304 notes).

## The calls (the scheme against a strong notrump)

| Call | Shows | Advancer |
|---|---|---|
| X | a long minor (six or more, no four-card side suit), or both majors (5-4 or longer) | 2♣ relay: doubler passes with clubs, bids 2♦ with diamonds, 2♥ with the majors (then preference) |
| 2♣ | clubs and a major, 5-4 either way | pass with three clubs, else 2♦ asks for the major |
| 2♦ | diamonds and a major, 5-4 either way | pass with three diamonds, else 2♥ pass or correct |
| 2♥/2♠ | natural: six or more, or a good five with no four-card minor | pass, or raise |
| 2NT | both minors, 5-5 or longer | the longer minor |

Strength as in Cappelletti: HCP plus a point a card beyond four, 12 for
a one-suiter, 10 for a two-suiter, 8 for the minors; 11 for the natural
2♥/2♠.

Against a **weak** notrump Meckwell players double for penalties and
shift the other calls (Bridge Bum). Our card has one field, for the
strong notrump, so the strong-notrump scheme is played against every
range; the sources say a passed hand and the balancing seat play it
anyway ("a passed hand uses the defensive scheme vs. a strong 1NT
instead, since a penalty double isn't needed anymore").

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only meckwell`.
IMPs by the errors yardstick, positive when the convention makes fewer;
"actor" is the side that made the first differing call. Against the
card's default defence, Meckwell: 367 boards changed; actor contract
-79, doubling +76; other side contract +97, doubling +47; double-dummy
-145; halves +21/-24 (z -0.1): neutral.

## Sources

- Wikipedia, "Meckwell convention", https://en.wikipedia.org/wiki/Meckwell_convention
  (X "a single minor or both majors; advancer bids 2♣, after which the
  intervener corrects to his actual suit if a minor, or hearts if holding
  both majors"; 2♣/2♦ that suit and a major; 2♥/2♠ natural; 2NT minors;
  direct and pass-out seats).
- Bridge Bum, "Meckwell Defense to 1NT",
  https://www.bridgebum.com/meckwell_defense_to_1nt.php (the strong- and
  weak-notrump schemes; "Advancer can pass or correct" over 2♣/2♦; the
  passed-hand rule above).
- ACBL Unit 390 (Simon), "Meckwell defense to 1NT",
  https://www.acblunit390.org/Simon/meckwellnt.htm (the same two schemes;
  its strong-NT double also allows "a good 2♠ overcall or other good
  hand", which we leave out).
- Bridge-Classroom `src/utils/ntDefenses.js` (the editor's Meckwell: "♣
  or ♦, or both majors", "♣ + a major", "♦ + a major", natural ♥/♠,
  "Minors").

## BBA

No reference: BBA does not play Meckwell. Judged by par and the side-IMPs
yardstick only (vs-1nt.notes.md, forced on for both sides).

## Gaps and open questions

- The weak-notrump scheme (penalty double) is not written: should the
  card get a `defense_vs_weak_nt` field? Bridge-Classroom's editor keeps
  two columns (`competitive.vs_1nt_strong` / `vs_1nt_weak`) as free text.
- Over 2♣/2♦ we ask with the next step (2♦ over 2♣, 2♥ over 2♦); some
  pairs bid 2♥ pass-or-correct over both.
- The strong variant of the double ("a good hand") is not written.
