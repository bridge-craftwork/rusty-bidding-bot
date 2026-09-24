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
