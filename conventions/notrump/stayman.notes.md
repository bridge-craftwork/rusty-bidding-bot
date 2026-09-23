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
