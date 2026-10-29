# texas-transfers (`texas-transfers.bid`): notes

4D/4H transfers over 1NT with a six-card major, and responder's call after
the completion. Cases: `texas-transfers.test`.

## Guidance

- **Texas means game only, or slam values followed by 4NT keycard.**
  Game only is 10–13 suit points opposite 15–17. **Slam values are 33 total
  points even opposite opener's minimum: 18+** (Rick, 2026-09-21).
- **Mild slam interest** (14–17 suit points) does not use Texas. Responder
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
| 15–16 | 4D 97% | 2D (mild slam) | 1 of 32 |
| 17–18 | 4D 100% | 4D | 12 of 12 |

HCP here; a six-card suit adds one suit point, so 15–16 HCP is 16–17 suit
points: mild slam interest by Rick's rule, where BBA uses Texas. Overall
276 of 600 agree.

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
- **13–17**: BBA uses Texas, and keycards after it with 16–17; we show mild
  slam interest through 2D (Rick: Texas slam needs 18). Moving the slam line
  from 16 to 18 cost about 390 agreeing calls; that is expected.
- **After a 5D answer ("0 or 3")**: BBA bids six with 13–16 (9 boards),
  reading it as 3. When the answer is ambiguous we sign off in five, and
  opener corrects with the higher count (Rick's rule).

## Gaps (not built yet)

- After Texas BBA also keycards with 11–14 HCP and a long suit or shortness
  (46 boards). Rick's rule is 33 total points, so we pass the completion.
- **Queen ask** (5D over 5C, 4 boards) and the **grand slam** (7S after 5S
  with 15–20, 5 boards): see `rkcb-1430.bid`.
- Interference over Texas (an overcall of the 4♦/4♥ transfer). A 2♣
  overcall of the 1NT opening is covered; see below.

## Convention score: the off switch (2026-10-07)

With `notrump.transfers.texas=false` our off run had 51 no-rule
positions (12 on): responder after the Jacoby completion, six-card
hands between the strength bands, and the slam hands that would have
used Texas then keycards. Both are now written in jacoby-transfers.bid
(jacoby-transfers.notes.md, Gaps). The score itself is in the
convention-score table (status page).

## Unexplained

- After `1NT P 4D P 4H P 4NT P 5H P`, BBA passes 5H (6 boards) even with
  four or five keycards between the hands (for example
  `A4.QJT932.A.AT42` opposite a 5H answer). We bid 6H. It is not clear
  what BBA's 5H means there.

## Under interference (2026-09-23)

`after 1N (2C) when vs_2c` repeats the four Texas rules and the two
completions, and adds one that only exists under interference:

- **Texas as a preempt.** With **seven** cards in a major and no values
  BBA transfers straight to game rather than to the two level, taking
  the auction away from them: `A9.JT87432.2.T65` (5 HCP) bids 4♦ over
  their 2♣. Written as `shows H>=7, hcp>=4, suit_strength=signoff`
  (it was `strength=signoff`; with length counted in total points that
  hand became 8 and lost the preempt, 2026-09-23).

Nothing is written over a double: 4♦ and 4♥ are still available there,
but BBA does not use them, and with `vs_double` off the two-level
escapes are the whole structure.

Evidence and numbers: `nt-interference.notes.md`.

## Support points (2026-09-23)

With six trumps the fit is certain, so **slam values** may also be
counted in support points: Texas and the keycard ask after it now also
fire with `tp(M)>=33-partner.points.min` (HCP + shortness), beside the
suit-point rule. `8.AQ9732.K94.AQ8` (15 HCP + 3 for the singleton = 18)
now Texases and asks for keycards; without the singleton
(`82.AQ9732.K94.AQ`, 17 support points) it is still mild slam interest
via Jacoby. Corpus par **+985 IMPs**, Basic_* 0.

## Everywhere the system is on, and over interference (Rick, 2026-10-29)

Texas was on only over the 1NT opening and its doubled overcall (`nt1`),
plus over their 2♣ when `notrump.transfers.vs_2c` was set. That field is
BBA's "Transfers if RHO bids clubs": 2♦/2♥ transfers over their 2♣, not
Texas. Rick: Texas is on after every notrump of ours where the system is
on: 2NT, 2♣–2♦–2NT, Kokish's 2NT (2♣–2♦–2♥–2♠–2NT), 2♣ and a positive
then 2NT, our notrump overcall as the card says. Over interference it
stays on whenever it is a jump: a double, or any bid that leaves three of
the transfer suit available. It never transfers into their suit, nor
into a major responder has already bid. The seven-card preempt applies only
over a bid, not over a double. Opposite 2NT, 4♦ is Texas, so the diamond
slam try (two-nt-responses.bid) gives way when the card plays Texas, and the
natural 4♥/4♠ over interference (nt-interference.bid) give way where Texas
uses the call.

Game values count HCP and length points (Rick: "8 HCP + 2 length points
= 10"): KQJ974 and 8 HCP is a Texas game opposite 15-17. Before, the
count used suit points (9).

Compare --limit 50 against main: 97 boards change, errors contract +100,
doubling −35 IMPs to us (2N_and_Transfers 16, We_Overcall_NT_then_Texas 10,
2N 9). Tests: texas-transfers.test (2NT, 2♣–2♦–2NT, Kokish, over 2♦, X,
3♣); two old expectations changed from a 3-level transfer to Texas.

Compared with Brill's Texas (Thorvald's Texas.json, 2026-10-29): Brill
plays it over 1NT and 2NT openings, our notrump overcalls and balancing
notrumps, over 1NT (2♣) only among interference, after 2♣–2♦–2NT and
2♣ (2M) P P 2NT, and also after 3NT (opening, 2♣–2♦–3NT, 2♣ (2M) P P
3NT), which we do not. Its game values are 10+ total points over 1NT;
after Texas it bids 4NT (RKCB 0314) with "slammish" hands or combined
values, controls and losers, or 6M/7M directly.

## Sources

- **The convention:** Texas transfers (4♦/4♥ over 1NT). Standard
  practice, not yet cited to a book or article.
- **Rick's rulings:** Texas is game only or slam values followed by
  keycard; slam values are 33 total points even opposite opener's
  minimum, so 18+; 8 HCP with a six-card major invites (2026-09-21); the
  sign-off on an ambiguous keycard answer (`rkcb-1430.notes.md`).
- **BBA probes:** the dealer3 script `six-hearts.dlr` run through
  `rbb probe --script` (600 deals), given in full in "Evidence from BBA".
  It is not yet kept as a `probes/*.toml` spec.
- **BBA evidence:** the Texas_Transfer and Texas_or_Jacoby corpora; Texas
  as a preempt over their 2♣ ("Under interference", evidence in
  `nt-interference.notes.md`).
- **Corpus measurements:** support points for slam values (2026-09-23,
  +985 IMPs).
- **Where we differ:** with 8 HCP, 10-12 HCP and 13-17 HCP, and after a
  5♦ answer ("Accepted differences from BBA").
