Title: The "why" panel: BBA's reference call and Rusty's reading of the student's call

Part of rusty-bidding-bot's integration plan for the practice tables:
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.4, §3.5, Q2).

The first draft of this issue also moved Rusty's bot seats to per-seat
bidding. That is now part of C1 (Bridge-Classroom), since R1
(rusty-bidding-bot)'s `auction` stops at the human's turn. What is left
is the teaching panel.

The student is marked against **BBA** for now (Q2, provisional default
approved 2026-09-28). When the student's call differs from BBA's, clicking
or tapping the diverged cell opens a panel with:

- BBA's reference call and BBA's meaning for it;
- labelled as Rusty's: what the student's call means to Rusty (its
  meaning of that call at that point, R1), and Rusty's own call for the
  student's hand (`bid`);
- when Rusty's call agrees with BBA's, why Rusty's rules rejected the
  student's call, from `candidates[].outcome` (for example "hand fails
  `shows strength>=invite`"), rewritten for students where the engine
  gives a readable reason. When Rusty and BBA disagree, the panel shows
  BBA's reference and Rusty's reading of the student's call only; showing
  both engines as two references is deferred (Q2).

## Acceptance criteria

- [ ] Clicking or tapping a diverged cell opens the panel; its content
      comes from BBA's meanings and Rusty's responses, not from hardcoded
      text.
- [ ] Every Rusty item is labelled as Rusty's; the reference is labelled
      BBA.
- [ ] It works in review and during the auction, and is hidden when the
      comparison is off (`bp.cardplayShowBbaCompare`, renamed if needed).
- [ ] It works on a BBA table too (Rusty's reading loads lazily when the
      panel opens), or is shown only on Rusty tables: decide and document.

Depends on: C3 (Bridge-Classroom), R1 (rusty-bidding-bot).
