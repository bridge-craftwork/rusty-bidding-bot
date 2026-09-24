# stayman (`stayman.bid`): notes

2C Stayman over 1NT, opener's answer, and responder's second call.
Cases: `stayman.test`.

## Guidance

- Stayman promises a four-card major and invitational values or better.
  With 5-4 in the majors, Stayman comes first (priority 1), ahead of the
  transfer, as the start of Smolen.
- **4-3-3-3 hands bid notrump and skip Stayman** (Rick, 2026-09-21), though
  BBA uses Stayman with them.
- Responder's second call: raise with a fit (3M invites, 4M is game, 6M is
  slam); without one, 2NT, 3NT or 6NT. Opener corrects 3NT to four of the
  other major with four cards (`choice-of-games.bid`).
- Opener's answer to 2NT uses a 4-4 fit when one is known
  (`one-nt.bid`, `asked nt_invite`).

## Evidence from BBA

Stayman scenario (21GF-DEFAULT, 500 boards): 92.4% of calls agree, 57.2%
identical auctions, 75.0% the same contract.

Choice of games after `1NT P 2C P 2M P 3NT P`: 210 of 211 corpus boards agree.

## Accepted differences from BBA

- **4-3-3-3 with a four-card major**: BBA uses Stayman on 755 corpus deals
  (21GF-DEFAULT) where we bid 2NT, 3NT, 4NT or 6NT. Rick: keep our rule.
- **2NT after Stayman**: BBA's shows about 7–8. With 7 HCP BBA bids 2C and
  then 2NT (for example `J543.964.J32.AJ3`), and BBA's opener passes it
  holding 17 (24 Stayman boards, for example `K64.KJ3.AK85.K32`). Ours
  shows 8–9, as after 1NT–2NT, and opener accepts with 17. Rick: our way is
  fine.

## Gaps (not built yet)

- **5-4 with a minor after 2D**: BBA bids 3C or 3D (for example
  `A82.QJ98.7.KJ973` bids 3C, then 4C; 16 boards). We bid 3NT.
- **Smolen** (5-4 majors, game force after 2D), **Garbage Stayman** (weak
  hands that pass the answer), and Stayman with interference.
- **Slam after a fit**: with 18+ and a fit we jump to 6M. Keycard first
  would be better.

## Questions

- **Weak hands.** Of the corpus deals where BBA bids Stayman and we do not,
  176 are under 8 HCP and not 4-3-3-3: Garbage Stayman, or the lighter
  invitation above. Build Garbage Stayman as its own card option?

## Under interference (2026-09-23)

Two contexts at the foot of `stayman.bid` keep Stayman alive when RHO
acts, gated by the card switches `notrump.transfers.vs_double` and
`notrump.transfers.vs_2c`:

- over a **double** it is still 2♣, from a point below the invitational
  floor (`hcp>=24-partner.hcp.max`, so 7 opposite 15-17: finding the
  4-4 fit is also somewhere to run to);
- over a **2♣ overcall** the call is gone and **the double takes its
  place**, 8+ as usual.

Both **deny a five-card major**, unlike the uncontested 2♣: under
interference BBA transfers with 5-4 rather than looking for the other
major first, so Smolen's priority does not apply here. Opener's answers
are the same three calls, written once per context.

Evidence and numbers: `nt-interference.notes.md`.

## Support points and length points (2026-09-23)

Rick: "our rules should have HCP, total points and support points, and
decide which to use under which circumstances". Once opener's answer
shows the fit, responder also counts **support points**, `tp(x)`: HCP
plus shortness (doubleton 1, singleton 3, void 5, capped by the trumps).

- **6M with 33 support points** opposite opener's minimum
  (`AQ82.K.KQ943.J83`, 15 HCP + 3 = 18 opposite 15–17). This is most of
  the gain: 1NT–2♣–2♥ → 6♥ and 1NT–2♣–2♠ → 6♠ (and the same after
  2NT–3♣) where we bid game.
- **4M with 25 support points** when the total-point count says invite
  (`KJ82.Q7.Q943.J83`: 9 HCP + 1 = 10, game).

These are **extra** rules beside the total-point ones, not replacements.
Replacing the bands with `tp` outright gained less (+1,896) because
partner can no longer read a negative inference from a `tp` rule: after
1NT–2♣–2♥–3NT opener stopped knowing that responder has four spades
(157 boards of 1N lost the 4♠ correction, −392 IMPs). Kept as extras,
corpus par **+2,478 IMPs**, Basic_* +2, uncontested −3 (noise).

Also: opener's answer to the raise (`when asked invite(M)` in
`jacoby-transfers.bid`) accepts with 16 in the fit, see its notes.

## BBA's Stayman, probed (2026-09-24)

Rick asked how BBA treats a light hand with a four-card major. Probes on
Basic_Openers_Rebid board 47 and variants, bba-cli with the Basic-Bridge
card, all four hands fixed (`rbb probe`):

| North (responder) | HCP | tens | BBA |
|---|---|---|---|
| 94.AT83.94.K9876 (the board, South 4-3-4-2) | 7 | 1 | 2♣, then 2NT over 2♠ |
| 94.AT83.964.K987 (2-4-3-4) | 7 | 1 | 2♣; passes 2♥, 2NT over 2♠ |
| 94.A983.964.K987 (♥10 → ♥9) | 7 | 0 | 2♣, then 2NT |
| 942.A983.964.K87 (4-3-3-3) | 7 | 0 | 2♣, then 2NT |
| 942.AT83.964.K87 | 7 | 1 | 2♣, then 2NT |
| T42.AT83.964.K87 | 7 | 2 | 2♣, then 2NT |
| 942.A983.964.Q87 | 6 | 0 | Pass |

With a four-card major BBA bids Stayman on 7 HCP whatever the shape and
tens, passes a fitting answer, and otherwise bids 2NT. Without a major it
passes 1NT on 7 HCP 4-3-3-3 without a ten (10 of 10 corpus hands).

Opener facing that 2NT (North 942.A983.964.K87):

| South | HCP | tens | BBA |
|---|---|---|---|
| AK65.QJ7.AQ3.J32 | 17 | 0 | Pass |
| AK65.Q72.AJ3.QJ2 | 17 | 0 | Pass |
| AKT5.Q72.AJ3.QJ2 | 17 | 1 | Pass |
| AKT5.QT2.AJ3.QJ2 | 17 | 2 | 3NT |
| AKT5.QT2.AJT.QJ2 | 17 | 3 | 3NT |

So 2NT after Stayman is 7-8 to BBA and opener needs 18 counting tens,
where after a direct 1NT-2NT (8-9) it accepts with 17. Both sides move a
point; the target stays 25.

**Measured as our default and rejected on par** (corpus, net IMPs vs BBA):
Stayman from 7 with a major, 2NT 7-9, pass with 7 once the fit is found:
-1,001 (Basic -18). Nearly all of it is opener declining ordinary 8-9
invitations once 2NT can be 7 (1NT-2C-2D-2NT alone: -446 on 232 boards).
Without the pass once the fit is found: -1,078.

Also rejected: passing 2M after the fit is found with any invitational
hand (-113; pass better on 135 boards, the invitation on 123, but the
invitation's wins are games), or with 8 and inviting with 9 (-227).

**Kept as the BBA treatment** (`general.style = bba`): the light Stayman,
2NT 7-9 and the pass with the fit. Our default stays Stayman from 8 total
points with 2NT 8-9, because par prefers it.
