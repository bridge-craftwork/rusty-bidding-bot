Title: Bidding engine setting (BBA or Rusty), remembered; labels name the bidder and the reference

Part of rusty-bidding-bot's integration plan for the practice tables:
https://github.com/bridge-craftwork/rusty-bidding-bot/blob/main/docs/integration-bridge-classroom.md
(§3.5, decisions 1 and 2).

## Scope

Add `bp.biddingEngine = 'bba' | 'rusty'` to the table settings, next to
the cardplay bot:

- **default `bba`, persisted like the other `bp.*` settings**, so the
  choice is remembered once changed (Rick's decision 1); `?bidder=` for
  testing;
- the chosen engine bids **all** non-human seats (decision 2);
- LocalEngine's bot-seat bidding dispatches to `bbaClient` or `rbbClient`
  (C1 (Bridge-Classroom)).

**The reference stays BBA** (Q2, provisional default approved
2026-09-28): whichever engine bids, the student is marked against BBA's
auction, as today. So the hardcoded labels become parameters that name
their source, and today they name:

- the bidder, on seat labels and anywhere a bot call is attributed: BBA
  or Rusty;
- the reference, on the stacked diverged cell (`AuctionTable.vue:37`) and
  the summary ("You matched the BBA all the way through",
  `localEngine.js:131-136`): BBA.

`getExpectedAuction()` keeps returning BBA's auction (it is the
reference).

## Acceptance criteria

- [ ] A full board can be played with Rusty for all three bot seats,
      including divergence, toggle, undo and restart.
- [ ] The setting defaults to BBA for a new user, and a changed choice
      survives a reload and a new session.
- [ ] Switching the engine takes effect on the next board.
- [ ] No table mixes engines across bot seats (the per-call BBA fallback
      aside).
- [ ] No label calls a Rusty call "BBA"; the reference is labelled BBA on
      both kinds of table.
- [ ] Embedded (iframe) mode keeps BBA unless the host passes `bidder`.

Depends on: C1 (Bridge-Classroom).
