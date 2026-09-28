# smolen (`smolen.bid`): notes

Smolen (card `notrump.smolen.play`; BBA's `SMOLEN`, on 17 of the 18
corpus cards, and on Rick's card). With 5-4 in the majors and game
values responder bids Stayman (`stayman.bid` gives those hands priority
1 for it); over opener's denial he bids three of his **four**-card
major, showing five of the other and forcing to game. Opener bids four
of that major with three, 3NT with two. Opener declares either way.
Cases: `smolen.test`. Built 2026-09-28.

## What the module does

- **Written against the question, not the auction.** The jumps sit
  under `when answered majors, partner.H<=3, partner.S<=3`, so the same
  rules serve 1NT–2♣–2♦, 2NT–3♣–3♦ (Smolen_after_2N) and the same
  auctions with systems on over our 1NT overcall or their double, as BBA
  plays them. The jump sets `forcing=game, ask=smolen(M)` with M the
  five-card major.
- **Opener** (`when asked smolen(M)`): 4M with three, 3NT with two.
  After 3♥ (five spades) there is room for **3♠: three spades and a
  maximum** (`hcp>=shown.hcp.max`), a slam try that agrees spades below
  game. BBA does exactly this (below).
- **Responder** after 3NT passes, or bids 6NT with 32 between the hands
  (the no-fit count after Stayman). After a fit the agreed suit and the
  game force are enough: keycard (`rkcb-1430.bid`), control bids and
  the base game fallback read state, and nothing here names them.
- **Strength.** The default counts total points (`strength>=game`: 10
  opposite 15-17, the fifth card of the major counting), as everywhere
  after 1NT (Rick, 2026-09-23). The `bba` treatment uses HCP, 10
  opposite 15-17, which is what BBA does.
- **Five spades, four hearts, invitational** now bids 2♠ over the 2♦
  denial (`stayman.bid`, `when asked spades_invite`): opener bids game
  with 16-17 (4♠ with three, 3NT with two) and passes a minimum. Not
  Smolen, but the invitational half of the same structure
  (Smolen_Invitational); it used to bid 2NT. With five hearts and four
  spades an invitation still transfers first (jacoby-transfers.bid).

## Evidence from BBA

- `probes/smolen-resp-1N.toml` (21GF-DEFAULT, 200 hands 5-4 in the
  majors, 8-18 HCP, after 1NT–2♣–2♦): BBA bids Smolen from **10 HCP**
  up to 18 (never a direct slam), and with 8-9 HCP two of the five-card
  major.
- `probes/smolen-opener-3H.toml`, `smolen-opener-3S.toml` (opener 15-17,
  no four-card major): with two cards in responder's suit 3NT, with
  three 4M, every time, except that after 3♥ with three spades and **17**
  BBA bids **3♠** (26 of 26 hands; 15-16 bid 4♠). After 3♠ (five hearts)
  there is no such room: 4♥ (a few 6♥ with 17).
- Corpus (Smolen, Smolen_after_2N): 1NT–2♣–2♦–3♠–4♥ 133, –3♥–4♠ 122,
  –3♠–3NT 88, –3♥–3NT 82, –3♥–3♠ 38; the same shapes after 2NT.

## Corpus (2026-09-28, all 343 scenarios, with Gerber)

The Smolen part of the change, by the first call that changed:

| Where | Boards | vs BBA (par) |
|---|---:|---:|
| 1NT–2♣–2♦: 3NT → 3♠ | 446 | +608 |
| 1NT–2♣–2♦: 3NT → 3♥ | 443 | +251 |
| 2NT–3♣–3♦: 3NT → 3♥ / 3♠ | 409 | +398 |
| 1NT–2♣–2♦: 2NT → 2♠ (invitational) | 117 | +167 |
| 1NT–2♣–2♦: pass → 3♠ / 3♥ | 40 | +263 |
| 6NT → Smolen (1NT and 2NT) | 306 | +18 |
| after our 1NT overcall (1♣/1♦/1♥/1♠ 1NT) | 461 | +34 (over 1♥: -47) |

Scenario totals: Smolen +542 (500 boards changed), Smolen_Invitational
+420, Smolen_after_2N +384. The whole change (Smolen and Gerber) is in
`gerber.notes.md`, "Corpus".

## Differences from BBA

- **Invitational hands with a fifth card.** 9 HCP with 5-4 in the
  majors is 10 total points to us, so it bids Smolen where BBA bids 2♠
  (117 boards in Smolen_Invitational). The `bba` treatment agrees with
  BBA. The par total for the scenario is positive either way.
- **After a fit** BBA cue-bids or asks for keycards more often (4♣ over
  opener's 3♠; 4NT over 4M with 11+ extra); we sign off in 4M unless the
  keycard modules' 33-point test fires. Not examined further.
- **After 2NT–3♣–3♦–3♠–3NT BBA sometimes bids 4♥** (23 boards):
  apparently with a strong or long heart suit. We pass; not yet probed.
- Over our 1NT overcall of 1♥, Smolen's 3♥ names their suit (-47 on 24
  boards). Left alone for now: too few boards to judge.

## Gaps

- 6-4 in the majors: some play 4♦/4♥ over 2♦ as a transfer showing six
  (Texas-style Smolen). Not built; 6-4 hands take their existing route.
- Responder's slam tries after 3NT other than 6NT (a quantitative 4NT).

## Sources

- **The convention:** Bridge Bum, "Smolen"
  (https://www.bridgebum.com/smolen.php), read 2026-09-28: game-forcing
  5-4 in the majors; after 2♦ "responder now jumps to the three-level in
  his shorter major, thereby showing five cards in the other major";
  opener bids 3NT with a doubleton, four of the major with three. Where we
  differ: the page says nothing about 2NT, opener's 3♠ slam try or 6-4
  hands; the 2NT version and the 3♠ try come from BBA (above).
- **Rick's rulings:** Smolen is on his card (`notrump.smolen.play`); total
  points include length (2026-09-23); 5-4 hands go through Stayman first
  (stayman.notes.md, 2026-09-24).
- **BBA probes:** `probes/smolen-resp-1N.toml`,
  `probes/smolen-opener-3H.toml`, `probes/smolen-opener-3S.toml`
  (21GF-DEFAULT, matchpoints, none vulnerable).
- **Corpus:** Smolen, Smolen_after_2N, Smolen_Invitational (BBA's
  auctions), and the compare runs above.
