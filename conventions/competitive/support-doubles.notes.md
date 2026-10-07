# Support doubles and redoubles (`support-doubles.bid`): notes

## Guidance (Rick, 2026-09-28)

Play support doubles and support redoubles where the card says so: on
Rick's own card (Bridge-Classroom, `doubles.support.play`,
`doubles.support.rdbl`, `doubles.support.through = "2♥"`) and on the PBS
cards (every BBA card but Basic-Bridge has `Support double redouble = 1`,
which sets both fields). After 1x (P) 1M and an overcall or double by
RHO, opener's double (redouble over their double) shows exactly three
cards in responder's major and a raise shows four; `through` is the
highest call over which it applies. Responder continues knowing the
three-card fit.

## What was built

- **Opener.** `after 1x (*) 1M (1z|2z|3z)`, M a major and z not M, with
  RHO's call no higher than `through`: `X` shows exactly three M, any
  strength, priority 1 so it comes before every other rebid (BBA doubled
  on 490 of 490 hands with three, including six-card minors and 18
  counts). Over `1x (*) 1M (X)`, with `rdbl`, `XX` shows the same. The
  raise with four needed nothing: the contested raise in rebids.bid
  (`1x (*) 1z (*)`) already shows four. Both calls set
  `ask=support(M, x, z)` (`support(M, x)` for the redouble), and the
  later rounds are written against that question, not against auctions.
- **`through`.** A card that sets it is read as a bid ("2♥", "2H"); a card
  that leaves it unset plays them through two of responder's major, where
  BBA stops: over `1C (P) 1H (2S)` and `1D (P) 1H (3C)` BBA's double is
  takeout, 15+ with at most three hearts
  (probes/support-x-1C-1H-2S.toml, support-x-1D-1H-3C.toml), while
  `1D (P) 1S (2H)` and `1C (P) 1S (2H)` are support doubles. This needed
  one engine addition (rule language unchanged, nothing an existing file
  could notice): bids compare in bidding order (`rho.last <= 2{M}`), and a
  text param that holds a bid compares as that bid (LANGUAGE.md §6).
- **Responder, LHO passing** (`when asked support(M, x, z)`): five or more
  M bids 2M up to 9 total points, jumps with 10-11, bids game with 12+;
  with four, 1NT (6-10 balanced, their suit stopped), 2NT (11-12, asks
  `nt_invite`) or 3NT (13+) with a stopper, else a raise of opener's minor
  or a new minor at the two level with 4+ (6-10), a cue bid with 13+ and no
  stopper (game forcing; opener bids 3NT with a stopper, else 4M in the
  4-3 fit), a 4-3 invitation with 11-12, and 2M in the 4-3 fit when
  nothing else fits. A penalty pass with four of their suit and two of the
  top five, 7+ HCP.
- **Responder after the redouble, LHO passing**: pass, playing one of the
  major redoubled with seven trumps (BBA 216 of 250), or 4M with five and
  13+.
- **Responder when LHO bids again**: compete to the cheapest M with five
  (`ask=signoff`: opener goes on with 19+, rebids.bid), game with five and
  12+, else pass.

## Precedence: the 18+ double after their takeout double

rebids.bid had one meaning for opener's double in these auctions: after
`1x (X) 1z (1y)`, `X` = 18+ (ticket b6). With support doubles on, **the
support double wins** where it applies (z a major, their call within
`through`): the 18+ rule now carries `when !support_x | z is C | z is D |
!(within through)`, and its follow-ups (opener's third call in
rebids.bid, responder's answers in responder-rebids.bid) are switched off
by `!answered support` / `!asked support`. Over a minor response, or
above `through`, or on a card without support doubles (Basic-Bridge), the
18+ double is as before. That is BBA's order: over `1C (X) 1H (1S)` it
doubled with three hearts on all 72 hands (11-17) and never doubled
otherwise (probes/support-x-1C-X-1H-1S.toml).

The strong hands that lose the 18+ double still need a call there, so the
support context gives them one (only after their takeout double, where
the 18+ double was): 2NT 18-19 balanced with a stopper, 3NT 20+, and a
cue bid of their suit with 18+, forcing to game. BBA: 2NT or 3NT with 18.

After `1x (P) 1M (1z)` opener's double had no meaning before: the engine
passed. Nothing else competes for it.

## Evidence (BBA probes, 21GF-DEFAULT, IMPs)

| spec | position | BBA | ours after |
|---|---|---|---|
| support-x-1C-1H-1S | opener, 3 hearts | X 103/103 (11-18) | 103/103 |
| support-x-1D-1S-2H | opener, 3 spades | X 84/84 | 84/84 |
| support-x-1C-1S-2D | opener, 3 spades | X 70/70 | 70/70 |
| support-x-1C-1S-2H | opener, 3 spades | X 78/78 | 78/78 |
| support-x-1C-1D-1H-2D | opener, 3 hearts | X 84/84 | 84/84 |
| support-x-1C-X-1H-1S | opener, 3 hearts | X 72/72 | 72/72 |
| support-xx-1C-1H-X | opener, 3 hearts | XX 69/69 | 69/69 |
| support-x-1C-1H-2S | above through | no support X; X is takeout 15+ | pass/raise as before |
| support-x-1D-1H-3C | above through | no support X | as before |
| support-x-resp-1C-1H-1S | responder after X (P) | see below | 174/300 (0 before) |
| support-x-resp-1D-1S-2C | responder after X (P) | see below | 157/250 (30 before) |
| support-xx-resp-1C-1H-X | responder after XX (P) | pass 216/250 | 184/250 (as before: the pass was already ours) |
| support-raise-resp-1C-1H-1S | responder after the raise | 2H raise: pass 6-10, 3H/4H | unchanged (not this module) |

The meaning BBA gives the double (`bba_alert`): "Support double
redouble: 11 to 21 total points, exactly 3 in responder's major".

Responder after the double, as BBA bids it: with five hearts 2H (6-7),
3H (mostly 8-10), 4H (11+); with four, **a minor before the 4-3 fit**
(2C with four clubs, 2D with four diamonds, 1NT with a stopper: 100 of
100 minimum hands), 2NT/3NT with a stopper when stronger, a cue bid or a
splinter with 12+; over a two-level overcall, a penalty pass with four of
their suit (30 of 184 hands with four of the major).

## Accepted differences from BBA

- **Responder with four of the major and 6-10 bids 2M in the 4-3 fit
  when he has no stopper and no four-card minor at the two level**; BBA
  finds a minor almost always (it bids 2D with four diamonds even after
  their diamond-free auction, and raises clubs on three). A 4-3 fit at
  the two level is textbook (Bridgebum: "rebid 2♥" among responder's weak
  continuations).
- **BBA raises opener's minor ahead of 1NT**; we let descriptiveness
  choose (11 hands).
- **No splinters** after the support double (BBA: 3S/3D, 12-20, 4+ hearts,
  shortness); with 13+ and four we bid 3NT with a stopper or cue bid.
- **Four-card major, 11-12, no stopper:** we invite in the 4-3 fit (3M);
  BBA bids a minor, 2NT or a splinter.
- **After the redouble** BBA sometimes bids 2C with four clubs; we always
  pass (BBA passes 86%).
- **Default `through`** is two of responder's major (BBA). Bridgebum
  gives "through 2♠" as the common agreement, "some partnerships limit it
  to 2♥"; a card that says so gets it.

## Measured effect (whole corpus, 342 scenarios, 170,633 boards)

Baseline is the rebased tree (01610eb). After:

| | before | after |
|---|---|---|
| calls agreeing with BBA | 1,366,343 (76.7%) | **1,368,008 (76.7%)**, +1,665 |
| vs BBA, par as the yardstick | -114,531 IMPs | **-114,037**, +494 |
| no rule in a live auction | 2,835 | 2,843 |
| passed a forcing auction | 92 | 100 |
| trump fit under 7 cards | 2,039 | 2,036 |
| contradicts earlier calls | 245 | 246 |

`probes/tools/sideimps.py base new 3`: 338 boards changed (68 skipped),
**+821 IMPs to the side that changed its call**, par distance +494. The
three largest groups: `1C 1D 1H` +174 (76 boards), `1C 1D 1S` +195 (70),
`1C 1H 1S` +190 (35); the smallest, `1C X 1S` -1 (18). Both yardsticks
agree, so it is adopted.

A first version bid a new minor at the three level and in LHO's suit
(`1C (1D) 1S (2H) X (P) 3D`: a 5- and 6-card fit); the minor bids are
now two-level only and not in a suit LHO has shown (+29 par, 3 fewer
short fits), and the competitive 3M sets `ask=signoff` so opener has an
answer (20 fewer no-rule positions than the first version).

## Convention score (2026-10-07)

The first run found our auctions identical with the card field on and
off (0 of 500 boards on Support_Double). The module was gated; the
position never arose: our fourth hand had no rule after (1x) P (1y)
and always passed, so nobody overcalled partner's response. With the
sandwich-seat overcalls and double written (overcalls.bid, 2026-10-07)
the switch changes 485 of 500 boards (BBA's 420): Rusty +589, BBA +151,
net +438 = +0.88 a changed board, both halves positive ("good"). The
other side's net is -1,452: the defenders who overcall now get
support-doubled and competed against.

## Gaps and open questions

- The +8 no-rule and +8 forcing passes are the defenders' side: after
  `1x (X) 1S (2x) X (3N) 4S` the opponents had cue-bid our suit (their
  game force) and have no rule over our 4S (4 boards; advances.bid's
  area). Before, opener's double did not exist and they never got there.
- Responder's splinters and the 4-3 choices above.
- Opener's strong hands with two cards in the major after `1x (P) 1M (1z)`
  (not after their double): BBA rebids 1NT 13-17, 2NT/3NT with 18; we
  still have no notrump rebid when they have bid over partner
  (rebids.bid, `1x (*) 1z (*)`), a gap older than this module.
- **For Rick.** Rick's card says "through 2♥". Over 1C (P) 1S (2H) that
  still applies; over 1C (P) 1S (2S) (their cue bid) it never does (z is
  M is excluded whatever `through` says). Is that the intended reading?

## Sources

- Bridgebum, "Support Double" (https://www.bridgebum.com/support_double.php,
  read 2026-09-28): opener and responder bid different suits at the one
  level, responder a major; exactly three for the double, four for the
  raise, "making a support double is opener's first priority"; the
  redouble over their double; "through 2♠, some limit it to 2♥";
  responder's weak (2M, 1NT, raise, new suit), invitational (jumps, 2NT)
  and game-forcing (cue bid) continuations. We follow it except the
  default `through` (BBA's two of the major) and responder's minor bids,
  which are two-level only.
- Wikipedia, "Support double" (https://en.wikipedia.org/wiki/Support_double,
  read 2026-09-28): Eric Rodwell, 1974; double or redouble three, raise
  four, other calls fewer than three; "through 2♥ or 2♠" by agreement.
- The Bridge Guys page could not be read (the site returned an error,
  2026-09-28); ACBL material not consulted.
- Rick's ruling of 2026-09-28 (the task above) and his card's values.
- BBA probes, all on 21GF-DEFAULT at IMPs: probes/support-x-1C-1H-1S.toml,
  support-x-1D-1S-2H.toml, support-x-1C-1H-2S.toml,
  support-x-1C-1S-2D.toml, support-x-1C-1S-2H.toml,
  support-x-1D-1H-3C.toml, support-xx-1C-1H-X.toml,
  support-x-1C-X-1H-1S.toml, support-x-1C-1D-1H-2D.toml,
  support-x-resp-1C-1H-1S.toml, support-x-resp-1D-1S-2C.toml,
  support-xx-resp-1C-1H-X.toml, support-raise-resp-1C-1H-1S.toml.
- Responder's point bands (2M to 9, jump 10-11, game 12+) follow BBA's
  meanings (3H 9-11, 4H 11+ total points) moved to our total-point count;
  the 1NT/2NT/3NT bands are the ones after-interference.bid uses.
- Corpus measurement above (both yardsticks).
