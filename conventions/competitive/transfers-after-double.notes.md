# Transfers after 1M (X) (`transfers-after-double.bid`)

Switched on by `vs_to_double.new_suit_forcing_tfr` (no stock card: the
PBS scenario Transfers_after_1M_X / Xfer_after_1M_X was bid on
21GF-DEFAULT, without them). Cases: `transfers-after-double.test`.

## What it plays (2026-10-05)

From the scenario's definition (after the ACBL Bulletin, June 2025):
over 1♥, 1♠ natural, 1NT clubs, 2♣ diamonds, 2♦ hearts; over 1♠, 1NT
clubs, 2♣ diamonds, 2♦ hearts, 2♥ spades. A new-suit transfer is five or
more (six when under 8 HCP), fewer than three trumps. The transfer to
opener's major is a three-card raise, 8-10; 2M three trumps 4-7; the
one-under jump (1♥-3♦, 1♠-3♥) four trumps 8-10; 3M preemptive (as
before); 2NT a fit and 11+ (Jordan, now with three trumps too); the
redouble 10+ with fewer than three trumps.

Opener completes (over the raise transfer: 3M with 15-16, game with
17+); responder passes when weak, invites in his suit with six, bids
3NT with 13+.

Other modules: BROMAD steps aside when this is on
(vs-takeout-double.bid); the natural 1NT response over 1M (X) steps aside
(after-interference.bid). Jordan's 2NT stays, outranked by this module's
three-trump version.

## Deviations and open questions

- Only after a major. 1m (X) transfers (Todd 599) are not written.
- Applied for a passed hand too; with Drury in competition the two
  overlap at 2♣ (Drury wins only if this is off). Question for Rick.
- Opener's super-acceptances of a new-suit transfer are not written.

## Compare (2026-10-05, `--set vs_to_double.new_suit_forcing_tfr=true`)

Transfers_after_1M_X and Xfer_after_1M_X, 100 boards, 72.3% agreement.
At the convention's calls: BBA (card off) raises 2M with four trumps
where we bid 3♦ (four trumps, 8-10), and bids 2♠ where we transfer with
2♥: the convention itself. No bug found.

## Sources

- PBS `btn/Transfers_after_1M_X.btn` (citing the ACBL Bulletin, June
  2025, p. 66): the whole scheme.
- Robert S. Todd, "1-Major (X) Transfers", Advancing in Bridge 598
  (cited by convention-card; not read).
