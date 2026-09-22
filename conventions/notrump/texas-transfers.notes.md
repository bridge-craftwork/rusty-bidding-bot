# texas-transfers (`texas-transfers.bid`): notes

4D/4H transfers over 1NT with a six-card major, and responder's call after
the completion. Cases: `texas-transfers.test`.

## Guidance

- **Texas means game only, or slam values followed by 4NT keycard.**
  Game only is 10–13 suit points opposite 15–17. Slam values are the slam
  invitation band or better.
- **Mild slam interest** (14–15 suit points) does not use Texas. Responder
  transfers at the two level and jumps to 4M, and opener bids on with 3+
  trumps and a maximum (`jacoby-transfers.bid`).
- Suit points are HCP + ½ per card beyond four, so a six-card suit adds 1.

## Evidence from BBA

A dealer3 probe: 600 deals where North has six hearts (at most three
spades, 8–18 HCP) and South a balanced 15–17 with 2+ hearts. Reproduce with
`rbb probe --script six-hearts.dlr -n 600 --prefix "1NT Pass" --dealer S`,
where `six-hearts.dlr` is:

```
generate 5000000
produce 600
dealer south
condition hearts(north) == 6 && spades(north) <= 3 && hcp(north) >= 8 && hcp(north) <= 18
  && hcp(south) >= 15 && hcp(south) <= 17
  && shape(south, any 4333 + any 4432 + any 5332) && hearts(south) >= 2
action printpbn
```

BBA's first call, by North's HCP ("short" is a singleton or void):

| HCP | BBA | us | agree |
|---|---|---|---|
| 8 | 4D 68% | 2D (then 3H) | 43 of 133 |
| 9 | 4D 97% | 4D | 91 of 94 |
| 10–12, no shortness | 2D 76% | 4D | 24 of 98 |
| 10–12, short | 4D 56%, 2D 44% | 4D | 92 of 164 |
| 13–14 | 4D 81% | 2D (mild slam) | 13 of 67 |
| 15–18 | 4D 98% | 4D | 43 of 44 |

With 10–12 BBA's usual route through 2D is 2D, 2H, 4H (84 deals), which is
our route for mild slam interest.

Texas_Transfer (500 boards): 86.9% of calls agree, 32.0% identical
auctions, 59.0% the same contract. Texas_or_Jacoby: 86.6%, 32.8%, 44.8%.
The top four divergences in Texas_Transfer are this choice (234 boards).

## Accepted differences from BBA

- **8 HCP with six**: BBA mostly bids game through Texas. We invite
  (Rick: 8 HCP with a six-card major invites).
- **10–12 HCP**: BBA's choice depends on shape. Flat hands usually go
  through 2D and then 4H. We use Texas for all game-only hands.
- **13–14 HCP**: BBA uses Texas; we show mild slam interest through 2D.
- **After a 5D answer ("0 or 3")**: BBA bids six with 13–16 (9 boards),
  reading it as 3. When the answer is ambiguous we sign off in five, and
  opener corrects with the higher count (Rick's rule).

## Gaps (not built yet)

- **When to keycard after Texas.** BBA asks with 11–14 HCP and a long suit
  or shortness (22 boards over spades, 24 over hearts); we pass the
  completion. The slam values band may be too high for shapely hands.
- **Queen ask** (5D over 5C, 4 boards) and the **grand slam** (7S after 5S
  with 15–20, 5 boards): see `rkcb-1430.bid`.
- Interference over Texas.

## Unexplained

- After `1NT P 4D P 4H P 4NT P 5H P`, BBA passes 5H (6 boards) even with
  four or five keycards between the hands (for example
  `A4.QJT932.A.AT42` opposite a 5H answer). We bid 6H. It is not clear
  what BBA's 5H means there.
