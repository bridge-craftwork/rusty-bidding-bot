# weak-two-responses (`weak-two-responses.bid`): notes

Responses to a weak two, and opener's answer to the 2NT ask. Cases:
`weak-two-responses.test`.

## Guidance (Rick, 2026-09-22)

Both treatments of the 2NT ask, chosen by the card field
`two_level.weak_two_2nt_response`:

- **feature** (the default): opener bids a side suit with the ace or king
  and a maximum, else three of the trump suit;
- **ogust**: 3♣ minimum/poor suit, 3♦ minimum/good suit, 3♥ maximum/poor,
  3♠ maximum/good, 3NT the top three honours.

BBA's import sets `ogust` when its Ogust switch is on (`[[derived]]`).

## Other choices

Responder: 2NT asks with 15+; four-card support raises (preemptive with a
minimum, game with 14+); with three-card support and a strong hand, ask
first. After the answer responder places the contract and never passes an
artificial one: game with 17+, otherwise three of the suit, 3NT when the
answer was higher, or four of a major.

## Gaps

- A new suit by responder (natural and forcing), and slam tries.
- Continuations after the 3NT Ogust answer beyond signing off.

## BBA treatment (2026-09-24)

`general.style = bba`. Evidence: the Basic_* boards (responder's and
opener's hands against BBA's call) and `rbb probe` with the responder's
hand fixed, `--prefix "2H Pass"`, dealer South, two random layouts each
(both layouts always agreed), at each vulnerability; opener's answers
with both hands fixed, `--prefix "2S Pass 2NT Pass"`. Basic-Bridge card,
where the 2NT answer is the feature treatment.

**Responder over 2♥ depends on vulnerability** (probes; P pass, 2N the
ask, 4H game). "Vul" is responder's side:

| North | HCP | ♥ | love all | we vul, they not | favourable |
|---|---|---|---|---|---|
| AQ52.K95.J932.Q6 | 12 | 3 | P | P | P |
| AQ52.K95.Q932.Q6 | 13 | 3 | P | 2N | P |
| AQ52.K95.K932.Q6 | 14 | 3 | P | 2N | P |
| AQ52.K95.KJ93.Q6 | 15 | 3 | 2N | 4H | 2N |
| AQ52.K95.KJ93.K6 | 16 | 3 | 4H | 4H | 2N |
| AQ52.K95.KQ93.K6 | 17 | 3 | 4H | 4H | 2N |
| AQ52.A95.KQ93.K6 | 18 | 3 | 2N | 4H | 2N |
| AQ5.K952.J932.Q6 | 12 | 4 | P | P | P |
| AQ5.K952.Q932.Q6 | 13 | 4 | 2N | 2N | P |
| AQ5.K952.K932.Q6 | 14 | 4 | 2N | 2N | P |
| AQ5.K952.KJ93.Q6 | 15 | 4 | 4H | 4H | 2N |
| AQ5.K952.KJ93.K6 | 16 | 4 | 4H | 4H | 2N |
| AQ5.K952.KQ93.K6 | 17 | 4 | 4H | 4H | 2N |
| AQ52.K9.KJ932.Q6 | 15 | 2 | 2N | 2N | 2N |
| AQ52.K9.KQ932.K6 | 17 | 2 | 2N | 2N | 2N |
| AQ52.9.KQ932.KJ6 | 15 | 1 | P | P | P |
| AQ52.9.AQ932.KJ6 | 16 | 1 | 2N | 2N | 2N |

The first corpus boards that showed it: AK95.Q83.AK92.76,
AQ52.K95.KJ93.K6 and AJ94.AT6.K95.A62 bid 2NT at favourable and 4♥ at
love all, vulnerable and all vulnerable (10 layouts each). BBA plays the
weak two as lighter the less it risks, so responder needs more for game.

Corpus, responses to 2♥/2♠ (Basic_*): with a singleton, 15-17 passes 11
times and asks 5; four-card support raises to three at 10-11 (3 of 3)
and passes at 12 (5 of 5); five-card support bids game 4 times (13, 13,
16, 19 HCP) and asks 3 times (13, 14, 14). Over 2♦: 5♦ with three-card
support and 18-20 (4 boards); 13-14 with support passes where our 2NT
rule would ask (5 of 5, all with responder vulnerable); a six-card major
with 13+ is bid (18 of 19, the other 5♦); a five-card major with 15+
asks 24 times and is bid 3 times, with 13-14 passes 7 times and is bid
3 times.

**Opener's answer to 2NT** (Basic_*, all three suits, 131 boards): 5-7
rebids the suit (29 of 29, and no 8-10 hand rebids it); 8-10 shows a
side ace, a king that is not singleton, or a queen third (3♣/3♦/3♥/3♠;
55 answers, every one in such a suit), and bids 3NT without one (47
answers, none holding one: queen doubletons and a singleton king do not
count). Probes over 2♠: KQ7543.52.87.K32 3♣,
KQ7543.52.K7.832 3♦, AQ7543.53.87.Q32 3♣, AQ7543.53.Q87.32 3♦,
KQ7543.K.872.832 3NT, AK7543.52.87.832 (7) 3♠, KQ7543.52.87.832 (5) 3♠.

Responder then: facing a maximum in a major, game or a slam try 48
times, pass 5, three of the suit 4 (58 boards without a double); over
2♦-2NT-3NT it bids 4♦ with 15-16 (8 of 9, the ninth passes) and passes
with 17+ (10 of 11, the other 4NT with 22). Over 4♦ opener bids 5♦ with
a singleton or void (3 of 3) and passes 3 of the 5 hands without one
(Q8.98.AK9632.J92 and 93.JT5.AK9754.93 bid 5♦).

**The rules** (weak-two-responses.bid, style bba):
- after 2♦/2♥/2♠: 4M with three-card support and 15-19 vulnerable, 16
  at love all (15 with four), 18 at favourable; 4M with five from 13 (the
  corpus is split there); 5♦
  with three and 18-20; the 2NT ask over a major from 13 with support
  vulnerable, with four at love all, with a doubleton from 14 unless at
  favourable; otherwise 15 with two or more, 18 with a singleton, and
  always 20; the raise to three with four and up to 11; a new major
  natural and forcing (over 2♦ six cards from 13 or five with 14; over
  2♥ five spades from 13 and at most one heart); pass up to 19.
- after a new suit: raise with three (4M with 9+), else rebid the suit.
- the 2NT answer: 3{t} 5-7, a feature 8-10, 3NT 8-10 without one.
- after the answer: game facing a maximum; over 3NT (minor) pass 17+,
  4♦ 15-16; opener's 5♦ with a shortness.

All the probed responder hands agree with the engine except two:
AQ52.A95.KQ93.K6 (18 with three: BBA 2NT at love all, we bid 4♥; the
corpus has 18 with three bidding 4M 7 times of 8) and AQ52.9.AQ932.KJ6
(16 with a singleton: BBA 2NT, we pass, as BBA does on most corpus
singletons).

**Measured**, Basic_Weak_2 under the bba style, openings and responses
together (500 boards): calls agreeing 79.3% → 86.5%, identical auctions
22.2% → 45.0%, same contract 35.2% → 54.8%, par net −497 → −277 IMPs;
with NS bidding alone 82.0% → 89.3%, 26.4% → 53.0%, 40.1% → 61.3%. The
Basic_* and corpus totals are in preempts.notes.md; the responses took
Basic_* identical auctions from 37.9% to 39.8% and same contract from
47.7% to 49.0% after the opening rules.

The default is unchanged (the default contexts gained only `style is not
bba`; the bba rules keep the style in their context, and hand conditions
never share a `when` with it). Board-by-board check in preempts.notes.md.

**Gaps and open questions**
- Opener after a new-suit response: BBA uses the cheapest calls for
  hands with a doubleton (2♦-2♥-2♠ six times, 3♣ three), 3♦ with a
  void or singleton, and raises with three. Our rule only raises or
  rebids the suit; the corpus is too small (35 boards) to pin the
  meaning of 2♠ and 3♣ down.
- Over 2♦-2NT-3♥ (a heart feature) BBA bid 4♦ three times where we bid
  3NT; not modelled.
- Vulnerable, BBA's six-four minor preempts need a better suit
  (preempts.notes.md).
- **Question for Rick**: BBA asks with 13-14 and a fit when vulnerable
  and bids game with 15-16 and three trumps, but at favourable needs 18.
  Our default asks from 15 and bids game only with four. Worth a par
  test of the vulnerability split?

## Rick's rulings and more probes (2026-09-24)

**Feature answer, standard (now the default).** With a maximum (8-10 of
the 4-10 range) show a side ace or king; without one a queen; a jack only
with AK of the suit. 3NT is kept for AKQ in the suit. A minimum, or a
maximum with nothing to show, rebids the suit. BBA's "3NT = maximum with
no side honour" stays as the `bba` treatment. Ogust is the card's other
choice (`ask2nt`), to be filled in later.

**Vulnerability and scoring**, probed over 2H (South 73.KQJ964.853.72,
West's pass forced), responder 3-3-4-3 with three hearts:

| North | HCP | love all MP | love all IMP | we vul (MP = IMP) | favourable (MP = IMP) |
|---|---|---|---|---|---|
| K94.A85.KQ64.Q93 | 14 | P | P | P | P |
| K94.A85.KQ64.K93 | 15 | P | 2NT | 2NT | P |
| A94.A85.KQ64.K93 | 16 | 2NT | 2NT | 4H | 2NT |
| AQ4.A85.KQ64.K93 | 18 | 2NT | 4H | 4H | 2NT |
| AQ4.A85.AQ64.K93 | 19 | 4H | 4H | 4H | 4H |

Both the 2NT ask and the raise to game move with vulnerability; at love
all IMPs make BBA about a point bolder, otherwise MP and IMP agree. The
corpus is all matchpoints.

**Preemptive raises exist**, and move with vulnerability too (scoring
matters only on the last hand):

| North | HCP | love all | we vul | favourable |
|---|---|---|---|---|
| 9852.T85.KQ64.J9 (3 trumps) | 6 | P | P | 3H |
| 985.T875.J642.98 (4, flat) | 1 | 3H | P | P |
| 9.T8752.K642.985 (5, singleton) | 3 | 3H | P | 3H |
| 9.T875.KQ64.J985 (4, singleton) | 6 | 4H | 3H | 4H |
| 9.AT875.9642.985 (5, singleton) | 4 | 4H | 3H | 4H |
| A9.T875.KQ64.J98 (4) | 10 | 3H | 3H | 4H MP / 3H IMP |

With four trumps and a singleton BBA preempts to game unless we are
vulnerable. The flat 1-count raising at love all and passing at
favourable is one probe and odd; to be checked with more hands before
the bba rules use it.

## Weak preemptive raises, more probes; BBA's responses become the default (2026-09-24)

Rick found the flat 1-count strange; nine more flat four-trump hands, 0-4
HCP, over the same 2H:

| North (four hearts, flat) | HCP | love all | we vul | favourable | both vul |
|---|---|---|---|---|---|
| 985.T875.J642.98 | 1 | 3H | P | P | P |
| 986.T875.9642.98 | 0 | 3H | P | P | P |
| 9865.T875.964.98 | 0 | 3H | P | P | P |
| 986.T875.J64.985 (4-3-3-3) | 1 | P | P | P | P |
| Q86.T875.964.985 (4-3-3-3) | 2 | P | P | P | P |
| 986.T875.Q964.J8 | 3 | 3H | P | P | P |
| K86.T875.964.985 (4-3-3-3) | 3 | 3H | P | P | P |
| 9865.T875.J64.98 | 1 | 3H | P | P | P |
| 986.A875.964.985 (4-3-3-3) | 4 | 3H | P | P | P |

and with shortness:

| North | HCP | love all | we vul | favourable | both vul |
|---|---|---|---|---|---|
| 9.T875.KQ64.J985 (4, singleton) | 6 | 4H | 3H | 4H | 4H |
| 9.AT875.9642.985 (5, singleton) | 4 | 4H | 3H | 4H | 3H |
| 9.T8752.K642.985 (5, singleton) | 3 | 3H | P | 3H | P |
| 9.T875.9642.J985 (4, singleton) | 1 | 4H | 3H | 3H | 3H |
| 98.T875.9642.J98 (4, doubleton) | 1 | P | P | P | P |

It is real: flat four-trump weak hands raise only at love all (4-3-3-3
with 1-2 HCP excepted), and pass at every other vulnerability, even
favourable. A singleton with four trumps always raises; the level is
not a clean function of HCP and vulnerability. Modelled simply: flat
0-4 raises at love all; a singleton and four trumps, up to 9 HCP, goes
to game unless vulnerable; 5+ HCP with four trumps raises to three as
before. Few corpus hands are affected (+1 IMP).

**Par test of BBA's responses as the default** (the 2NT ask, raises and
game by vulnerability, forcing new suits; opener's answers stay the
standard feature ones): corpus -168,198 -> -167,745 (+453), Basic_* +78,
uncontested +75, Basic_Weak_2 -439 -> -361. Adopted; the previous
responses (2NT from 15, raises with four) are the `standard` treatment.

## Responder after 2x-2M-3x (2026-09-25)

The most frequent "no rule" position in Basic_* N/S auctions (27 of 203
positions): opener rebid the suit over responder's forcing major, and
responder had nothing to say. Probed with 300 random responders each
(5+ in the major, 13-21 HCP; opener six of its suit, at most two of the
major), at love all and all vulnerable: `probes/w2-2H-2S-3H.toml`,
`w2-2D-2H-3D.toml`, `w2-2D-2S-3D.toml`.

BBA passes most hands up to 18-19 HCP. It bids game with a seven-card
major from 13 (and a six-card suit with four of the top five honours
from about 16), three of the major with six and 19+ (forcing: opener
bids 4M with a doubleton, 3NT without), and 3NT or four of opener's
major with 20+. Over 3♦ it sometimes raises to 4♦ with 16-20 and a
doubleton, which opener mostly passes; not modelled.

Our rules follow that. Agreement with BBA on the grids: 92-95%, the rest
mostly BBA passing a few 19-counts where we bid. Basic_* N/S: no-rule
positions 203 -> 176, vs BBA +21 IMPs, +47 to the side that changed its
call (7 boards).

## Opener after responder signs off following the answer (2026-09-25)

2x-2NT-3y-3x (y above x's three-level) is a sign-off, and opener now
passes it explicitly (it passed before by default, with no rule). BBA
bid on over 2♦-2NT-3♣-3♦ in four Basic_Weak_2 boards (73: 5♦, 109 and
431: 3NT, 282: 4♦); against par, passing 3♦ was better on 109, 282
and 431 (+370, +210, +330 NS) and worse on 73 (150 against 400). Par
decides: kept as a sign-off. No-rule positions 176 -> 143.

## 3NT before 5♦ over a weak 2♦ (Rick, 2026-09-27; ticket basic-weak-2-b104)

Basic_Weak_2 board 104: 2♦ P with AK.AK985.JT2.A74 bid 5♦ (the
BBA-derived "three-card support, 18-20") and went down; 3NT makes ten
tricks. Rick: over a weak 2♦ strongly prefer 3NT to 5m, with something
in diamonds and the unbid suits stopped. Now 3NT with 16+, two or more
diamonds, every side suit stopped and no six-card major, ranked above
5♦.

| variant | full corpus (par / side) | Basic_* (par / side) |
|---|---|---|
| 14+ | +79 / +58 | |
| 15+ | +138 / +167 | -13 / -10 |
| 15+, no six-card major | +139 / +181 | +2 / +11 |
| 16+, no six-card major | +126 / +202 | +10 / +26 |
| 17+, no six-card major | +59 / +119 | +2 / +18 |

16 kept: better on three of the four figures, 13 IMPs behind 15 by par
on the corpus. The Basic_* losses at 15 were hands where the 2NT ask
would have found opener's minimum and stopped in 3♦, and hands with a
six-card heart suit.

On the same board BBA answered the 2NT ask with 3NT on KQ8764 and Q3:
BBA's maximum with no side honour (the `bba` treatment), which our
default reads as AKQ. Our default answer is 3♥, the heart queen with a
maximum, and responder then bids 3NT. No change needed there.

## Slam after the 2NT ask (2026-09-27)

Ticket basic-weak-2-b201: 2H 2NT 3S (maximum), North AKJT86.AJ.AK8.AQ
bid 4H. Responder now agrees the weak-two suit and asks with 4NT
(keycards on 1430/0314 cards, aces on standard-Blackwood cards) when
his HCP plus partner's floor reach 33: +308 par, +832 side IMPs (70
boards). The board now reaches 6H.

**Open, for Rick: counting tricks for the grand.** BBA reached 7H via
5NT; Rick counts 14 top tricks for 7NT. `grand_try` is still the
placeholder "35 HCP between us, partner at his floor" (34 here). A
real evaluator would count sure tricks: my suits' top cards with
partner's known honours and length, plus the aces and kings the asks
have located. That wants a design (what partner's answers pin down,
how a long suit with a known fit counts), not a one-off rule.

## Game opposite a maximum answer (Rick, 2026-09-28; ticket basic-weak-2-b278)

Basic_Weak_2 board 278: 2H P 2NT P 3C P, North AJ94.AT6.K95.A62 (16)
signed off in 3H; BBA bid 4H. Rick: with 16 and opener showing a
maximum, responder should normally bid game. The only game rule was
`hcp>=17` whatever the answer, which is right opposite a minimum (4-10)
and too much opposite a feature (8-10).

Now responder counts partner at the floor of his answer: game with
`hcp + partner.hcp.min >= 21` (four of the major, 3NT over a minor),
ranked above the sign-offs. Facing a feature (8+) every 2NT asker (13+)
bids game; facing an Ogust maximum (7+) from 14; facing a minimum (4)
the old 17 is unchanged.

| combined threshold | full corpus vs BBA | side IMPs | par-distance (sideimps) | boards |
|---|---|---|---|---|
| baseline | -115,452 | | | |
| 20 (also 16 facing a minimum) | -114,591 (+861) | +1051 | +861 | 330 |
| **21** | **-114,604 (+848)** | **+1054** | **+848** | 284 |
| 22 | -114,659 (+793) | +975 | +793 | 261 |
| 23 | -114,976 (+476) | +550 | +476 | 128 |

21 adopted: 20 is 13 IMPs better by par and 3 worse by side IMPs, so
the yardsticks disagree on lowering the minimum case to 16; not taken.
The gains are over 2H (+472 side, 119 boards) and 2S (+559, 158).

**BBA probes** (`probes/w2-2H-2N-3C.toml`, `probes/w2-2H-2N-3H.toml`:
200 responders each, 13-17 HCP with two or three hearts, opener 6-card
hearts, 8-10 or 4-7; love all and NS vulnerable, MP and IMP). Over the
3C feature BBA bids 4H with 15+ almost always, with 14 about half the
time and with 13 about a quarter (MP 12 of 63, IMP 19 of 63); heart
length does not split it. So BBA wants about 14 opposite a feature;
the corpus par says game from 13, and par decides. Over the 3H minimum
BBA passes up to 14, and with 15-17 bids game about a quarter of the
time at love all and half or more with NS vulnerable, where our rule
waits for 17: left alone here (not part of the ticket; a candidate for
a later par test).

## Sources

- **The conventions:** the 2NT feature ask and Ogust, with the usual
  answers. Standard practice, not yet cited to a book or article.
- **Rick's rulings:** both treatments of the 2NT ask, chosen by the card
  (2026-09-22); the standard feature answer as the default (2026-09-24);
  BBA's responses become the default after a par test, the old ones kept
  as the `standard` treatment (2026-09-24); 3NT before 5♦ over a weak 2♦
  (2026-09-27, ticket basic-weak-2-b104); game opposite a maximum answer
  (2026-09-28, ticket basic-weak-2-b278). Ticket basic-weak-2-b201
  (slam after the ask, 2026-09-27).
- **BBA probes:** `rbb probe` with responder's hand fixed over 2♥ and with
  both hands fixed over `2S Pass 2NT Pass`, at each vulnerability
  (2026-09-24, hands in the tables); `probes/w2-2H-2S-3H.toml`,
  `probes/w2-2D-2H-3D.toml`, `probes/w2-2D-2S-3D.toml` (2026-09-25);
  `probes/w2-2H-2N-3C.toml`, `probes/w2-2H-2N-3H.toml` (2026-09-28).
- **BBA evidence:** the Basic_* boards, Basic_Weak_2 in particular.
- **Corpus measurements:** the par test of BBA's responses (+453), the
  sign-off pass, slam after the ask (+308), the game threshold table
  (21 adopted), judged by par and by side IMPs.
- **Where we differ:** we pass 2x-2NT-3y-3x, where BBA sometimes bids on
  (par decides); game opposite a feature from 13 where BBA wants about 14;
  the vulnerability split for asking is a question for Rick.
