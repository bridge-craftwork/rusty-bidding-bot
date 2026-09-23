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
- **Mild slam interest** (six-card major, 14–17 suit points): transfer,
  then jump to 4M. Opener bids on (4NT keycard) with 3+ trumps and a
  maximum. Game-only hands and slam hands (18+: Texas needs 33 even opposite
  a 15 minimum) use Texas (`texas-transfers.bid`).
- **Responder's new suit (3m) is game forcing** (Rick, 2026-09-21). Opener
  rebids the major with three-card support, which agrees it below game so
  responder can look for slam; otherwise 3NT.
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
  ours at 8 and at 10–17 (see `texas-transfers.notes.md`).
- **Opener after the new suit**: BBA jumps to 4M with three trumps (106
  Jacoby_Transfer boards); we bid 3M, as Rick plays it. Once 3M agrees the
  suit, BBA's responder sometimes bids 3NT, offering a choice; we bid 4M in the
  known fit.
- After a plain 3M super-accept, which may be a minimum, BBA passes some
  invitational hands where we bid game (8 boards; 24 others go our way).

## Gaps (not built yet)

- **5-4 invitations.** 9 HCP with five spades and four of a minor bids 3m
  for BBA (for example `A6543.7.K43.QT63`). Our 3m is game forcing, so these
  hands need another route (2NT with an unbalanced hand?). For now they
  have no rule and pass the completion (76 boards in Jacoby_Transfer and
  Texas_or_Jacoby).
- **Five-four with game values after a super-accept.** BBA bids 4M; our
  game-forcing 3m ranks higher (17 boards).
- **Opener after 6NT with five**: no correction to 6M yet.
- **Interference**: a double of the transfer (31 boards), or an overcall.
  (RHO's double of the 1NT opening and a 2♣ overcall are now covered —
  see below — but not an overcall of the transfer itself.)

## Questions

- How should responder invite with a five-card major and a four-card minor,
  now that 3m is game forcing?

## Under interference (2026-09-23)

`after 1N (X) when vs_double` and `after 1N (2C) when vs_2c` keep the
transfers on their own calls, 2♦ for hearts and 2♥ for spades, with the
completions written for each. Two differences from the uncontested
rules:

- **the longer major wins, and spades when they are equal**
  (`when S>=H` / `when H>S`). Uncontested there is no such tie-break and
  5-5 hands go through Stayman; under interference BBA transfers, and to
  the higher suit. Note that a `prefer S` / `prefer H` pair did **not**
  break the 5-5 tie — the two calls score the same — so the choice is a
  `when` on each rule.
- **over a 2♣ overcall the transfer has a five-HCP floor.** Below that
  BBA leaves their 2♣ alone rather than push to the two level with
  nothing; over a double there is no floor, because running is the
  point.

The super-accepts are not written for these auctions: opener simply
completes. Evidence and numbers: `nt-interference.notes.md`.
