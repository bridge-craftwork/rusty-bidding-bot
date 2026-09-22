# jacoby-transfers (`jacoby-transfers.bid`): notes

2D/2H transfers over 1NT, the completion, responder's second call, and
opener's answers to invitations and to the mild slam try. The
super-accepts are separate modules. Cases: `jacoby-transfers.test`.

## Guidance

- The transfer is forcing; opener completes (or super-accepts).
- Responder's second call is one `when answered transfer(M)` block, with
  no auction patterns. Notrump decisions (2NT, 3NT) use notrump points (HCP
  + ½ per ten). Decisions to play in the major use **suit points: HCP + ½
  per card beyond four**.
- **8 HCP with a six-card major invites** (transfer, then 3M): 8 HCP + 1 for
  length is 9 suit points.
- 3NT with exactly five offers a choice; opener corrects to 4M with three
  (`choice-of-games.bid`).
- **Mild slam interest** (six-card major, 14–15 suit points): transfer,
  then jump to 4M. Opener bids on (4NT keycard) with 3+ trumps and a
  maximum. Game-only hands and slam hands use Texas (`texas-transfers.bid`).
- Opener accepts an invitation only when game is certain, and plays in the
  major with an eight-card fit: three opposite five, two opposite six.
- After a super-accept (a known fit of nine or more) responder counts suit
  points. Opposite that maximum, invitational values bid game: the
  super-accept exists to reach thin games.

## Evidence from BBA

Probes (21GF-DEFAULT), `--prefix "1NT Pass 2D Pass 2H Pass" --dealer S`:

| Hand (then 1–4 more tens) | BBA | Agree |
|---|---|---|
| `82.KJ973.K94.Q83` (9, five hearts) | 2NT with 0–1 tens, 3NT with 2+: tens count at notrump | 5/5 |
| `82.QJ9732.K94.Q8` (8, six hearts) | 3H however many tens: length counts, tens do not | 5/5 |
| `82.Q97532.K94.Q8` (7, six hearts) | 3H, the same | 5/5 |
| `82.K9732.K94.Q83` (8, five hearts) | 2NT with 0–1 tens, **pass** with 2+ | 2/5 |

Jacoby_Transfer (500 boards): 85.9% of calls agree, 29.2% identical
auctions, 39.2% the same contract. Jacoby_Super-Accept: 81.7%, 30.8%, 33.8%.

## Accepted differences from BBA

- **A ten in the trump suit.** `82.KT973.K94.Q83`: BBA passes. With the ten
  in diamonds instead (`82.K9732.KT9.Q83`), BBA invites. It is the same on
  every layout. That looks like an internal quirk of BBA, not a principle,
  so we do not model it.
- With a six-card major, BBA's choice between Jacoby and Texas differs from
  ours at 8 and at 10–14 HCP (see `texas-transfers.notes.md`).
- After a plain 3M super-accept, which may be a minimum, BBA passes some
  invitational hands where we bid game (8 boards; 24 others go our way).

## Gaps (not built yet)

- **5-4 invitations.** 9 HCP with five spades and four of a minor bids 3m
  for BBA (for example `A6543.7.K43.QT63`), so its new suit is not game
  forcing. We have no rule for these hands and pass the completion (76
  boards in Jacoby_Transfer and Texas_or_Jacoby).
- **Opener after the new suit.** After `1NT P 2M P 2M P 3m P`, BBA's
  opener raises to 4M with three trumps (106 Jacoby_Transfer boards). We
  have no rule for opener here, so base's game-force rule bids 3NT.
- **Five-four with game values after a super-accept.** BBA bids 4M; our
  game-forcing 3m ranks higher (17 boards).
- **Opener after 6NT with five**: no correction to 6M yet.
- **Interference**: a double of the transfer (31 boards), or an overcall.

## Questions

- The 5-4 invitation above means BBA's 3m after a transfer is invitational
  or better. Should our 3m be invitational too?
