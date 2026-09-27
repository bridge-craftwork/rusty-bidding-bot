# Responses to a 2NT opening (`two-nt-responses.bid`): notes

Cases: `two-nt-responses.test`.

## Guidance

The 1NT structure one level up, which is what BBA plays on every card in
the corpus: 3♣ Stayman, 3♦ and 3♥ transfers, 3NT to play, 4NT
quantitative, and pass with nothing.

Only the asking calls and opener's answers are written here. Everything
after them was already written against the question — `when answered
majors` in `stayman.bid`, `when answered transfer(M)` in
`jacoby-transfers.bid` — and the strength bands read opener's 20-21, so
responder invites and bids game at the right level without a single new
rule. The calls that would be illegal a level up (the invitational 2NT,
a 3M rebid under the completed transfer) drop out on their own.

## Evidence

`bba-cli --all-meanings` over the Basic_* scenarios, responder's call
after `2N P`:

| call | BBA's meaning | n |
|---|---|---|
| 3♣ | "Stayman" | 37 |
| 3♥ | "transfer" (spades) | 23 |
| 3NT | "calculated bid", 4-10 total points | 16 |
| Pass | 0-4 total points | 16 |
| 3♦ | "transfer" (hearts) | 14 |
| 6NT | 13+ | 4 |
| 4NT | "Quantitative 4NT", 11-12 | 3 |

Before this module responder passed a 2NT opening whatever he held:
4,345 boards across the corpus, 123 of them in Basic_*.

| scenario | calls | auctions | contracts |
|---|---|---|---|
| 2N | 68.8% → **80.2%** | 3.2% → 32.8% | 5.4% → 46.8% |
| 2N_and_Balanced | 69.2% → **81.2%** | 2.0% → 32.2% | 4.0% → 58.4% |
| 2N_and_1_Minor | 69.0% → **75.5%** | 1.2% → 22.4% | 3.2% → 27.2% |
| Basic_* (nine) | 83.1% → **83.6%** | 34.4% → 35.3% | 41.0% → 42.1% |

1N and Basic_NT are unchanged, which is the check that the shared
continuations still behave at the one level.

One ranking fix came with it: in `stayman.bid`, "game, no fit found"
(3NT) and "invitational, no fit found" (2NT) now have `priority -1`, so a
known 4-4 major fit is played in the major. Opposite a 20-21 opening the
game band is wide enough that 3NT was outranking 4♥ on descriptiveness.

## Gaps and open questions

- **Minor-suit transfers over 2NT** (`notrump.two_nt.minor_transfers`,
  on for both 21GF cards) and minor-suit Stayman: not written, so a
  minor one-suiter bids 3NT.
- **Puppet Stayman** (`two_nt.puppet`) is only guarded against, not
  played: the 3♣ rule stands down when the card says puppet, and nothing
  replaces it.
- The 3♠ relay, `two_nt.three_s`, and the four-level transfers.
- Texas over 2NT, and slam bidding beyond the quantitative 4NT.

## BBA treatment (2026-09-24)

`general.style = bba` plays a probed model of BBA's responses to 2NT.
Probes: `rbb probe`, Basic-Bridge both sides, dealer S, North after a
forced `2NT Pass`, MP and IMP at love all and IMP all vulnerable (and MP
all vulnerable on the first set). **Scoring and vulnerability changed
nothing here.**

| North | HCP (+tens) | BBA |
|---|---|---|
| 8432.J32.63.8632 | 1 | P |
| K432.7632.96.972 | 3 | P |
| T432.KT32.96.972 | 3+2 | P |
| K432.Q632.96.972 | 5 | 3♣, then 3NT over 3♦ |
| Q432.Q632.96.972 | 4 | P |
| Q432.K632.96.972, K432.K632.96.972 | 5, 6 | 3♣ |
| K32.Q632.962.972 (4-3-3-3) | 5 | 3♣ |
| 5432.Q72.962.972, 5432.K72.962.972 | 2, 3 | P |
| 432.J72.K962.972, T32.J72.KT62.972 | 4, 4+2 | P |
| 432.Q72.K962.972 (and +1, +2 tens) | 5 | 3NT |
| 432.K72.A962.K72 | 10 | 3NT |
| T32.KT2.AT62.K72 | 10+3 | 3NT |
| 432.K72.A962.KJ2, 432.K72.A962.A72 | 11 | 4NT |
| 432.KT2.AT62.A72 | 11+2 | 6NT |
| 432.K72.A962.AJ2 | 12 | 6NT |
| Q32.K72.A962.A72 | 13 | 6NT |
| A32.K72.A962.AQ2 | 17 | 7NT |

In HCP: **pass with 4 or less, a four-card major or not**; from 5
Stayman (4-3-3-3 included) or 3NT; 4NT with 11; 6NT with 12, or 11 and
two tens. Corpus (all three 15-17 cards, 2NT the same structure): with a
major 3–4 HCP mostly pass (the rest are 21GF sequences such as 3♠),
5–6 HCP 3♣ 100% (230 boards); without one 5 HCP 3NT 100%. Basic_What_To_Open: with 5-5 in the majors BBA transfers to spades
(3♥, three boards), and with a five-card major and slam values it
transfers first (`KJT96.A42.K63.T8`, and Basic_Minor 228: 3♥ then 6NT).

`bba-cli --all-meanings` already calls the 4NT "Quantitative 4NT, 11-12":
opposite 20–21 that is 32 with the minimum, where over 1NT BBA invites
with 16 (33 with the maximum).

**What the rules model** (`when style is bba`; the default rules `style
is not bba`): 3♣ `hcp>=25-partner.hcp.min` with a four-card major, any
shape; 3♥ with 5-5; 3NT from 5 HCP; 4NT `hcp>=32-partner.hcp.max,
points<=31-partner.hcp.min`; 6NT `points>=32-partner.hcp.min`; both with
no five-card major; pass with 4 HCP or less. Opener's answer to 4NT over
2NT keeps the default (32 combined): BBA accepted with 21 (two probes;
20 not tried).

**Measured** (whole corpus, bba style, calls agreeing at `2NT P`): 65.7%
→ 73.9% (4,422 decisions; Basic-Bridge 79.7% → 95.7%). The rest is on
the 21GF cards: Gerber 4♣, minor-suit transfers, Puppet (gaps above).

**Tried as our default** (scratch copy, 3♣ Stayman and the pass keyed to
5 HCP): corpus par −66 IMPs, Basic_* 0. Not proposed.

## Responder after the completed transfer (2026-09-25)

The generic transfer rules measure strength bands against opener's
20-21, and left weak or unbalanced hands with no call after 2NT-3D-3H
and 2NT-3H-3S: 408 corpus boards. BBA passes with 0-3 HCP. From 4
(25 combined) it bids 3NT with five of the major, balanced or not, and
4M with six. Those are now fallbacks under the generic rules.
Corpus: -166,629 -> -165,434 (+1,195 IMPs).

## Weak hands with no call (2026-09-25)

Two gaps in the default, both seen in Basic_What_To_Open N/S auctions:

- **A 4-count.** Opposite 20-21 the signoff band ends at 3 total points
  and game starts at 5, so 4 is the invitational band, which has no
  call over 2NT. The pass now covers `strength<=invite`. BBA passes with
  4 too (the bba rule's "4 HCP or less").
- **Stayman with nothing.** The default 3♣ had no floor, so 0-3 counts
  with a four-card major bid it and then had no rule after the answer
  (2NT-3♣-3♦ and 2NT-3♣-3♥, 5 boards). It now needs game values, as
  BBA's does (from 5 HCP).

No-rule positions in Basic_* N/S fell by 10; par effect on Basic_* +1.

## Slam in a minor over 2NT (2026-09-27)

483 boards stopped at 2NT-3NT where BBA bid slam (median 31 HCP between
the hands, half of the slams in a minor). Now 4♣/4♦ is a slam try with
six cards or five unbalanced and slam-invite values; opener bids six
with three-card support, else 4NT; responder then 6NT with slam values.
Full corpus +1,730 by par distance, +3,123 by side (448 boards).
