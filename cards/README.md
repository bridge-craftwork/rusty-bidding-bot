# Stock cards

`bbsa/*.bbsa` are the stock cards: embedded by `crates/assets`
(`rbb_assets::card("SAYC")`), listed by the WASM build (`stock_cards`), named
by `rbb ... -c`/`--ns-card` and `.test` `card` lines, and valid BBA cards.
They are read in the convention-card vocabulary (`spec/fields.toml`,
`spec/formats/bbsa-map.toml`). The BBA ones (`21GF-*`, `Basic-Bridge`,
`Precision*`) are copies of `Practice-Bidding-Scenarios/bbsa/`.

`json/*.json` are the sources of the cards made here, in the card vocabulary:
`rbb card check json/X.json`, then `rbb card export-bbsa json/X.json -o
bbsa/X.bbsa`. A JSON card says more than its `.bbsa`: a field with no `.bbsa`
key (weak jump overcalls, the 1NT overcall range, negative doubles through 2♠,
...) does not survive the export, and the stock card falls back on the
rules' default for it. Use the JSON (`-c cards/json/SAYC.json`) where that
matters.

Made 2026-10-10 for Thorvald Aagaard's Brill round robin, whose Seat Robot
API names each side's system (`ourSystem` / `theirSystem`:
`../rusty-seat-service/docs/seat-api.txt`).

## SAYC

The ACBL Standard American Yellow Card as its booklet publishes it ("ACBL
Standard Yellow Card System Booklet"; text used:
<https://www.sao.ru/hq/dim/bridge/biddings/natural/bridgebsysyellowcard.html>).
Built from `Basic-Bridge` (BBA's SAYC preset, `System type = 1`). Each setting
and the booklet line behind it:

| Setting | Booklet |
|---|---|
| `System type` sayc: 5-card majors, 2/1 forcing one round, 2♣ strong | "1♥ and 1♠ openings show a five-card or longer suit"; "2♣, 2♦ = 11 points or more"; "A 2♣ opening shows at least 22+ points" |
| 1NT 15-17, may hold a five-card major; 2NT 20-21; 3NT 25-27 | "1NT = 15-17", "2NT = 20-21", "3NT = 25-27", "can be made with a five-card major or minor suit" |
| Stayman, non-forcing (JSON) | "2♣ is 'non-forcing' Stayman" |
| Jacoby transfers (implied by every .bbsa) | "2♦ is a transfer to hearts, 2♥ is a transfer to spades" |
| `Super acceptance after NT` | "3♥ = 17 points and four-card heart support" |
| `Transfers if RHO doubles` on, `... bids clubs` off, no Lebensohl | "If the opponents double, all conventional responses are 'on'"; "If the opponents bid over your 1NT opener, Stayman and transfers are 'off'" |
| `minor_transfers = relay` (JSON only, see gaps) | "A 2♠ response requires the 1NT bidder to rebid 3♣" |
| `1N-3D natural`; Texas off | "3♣, 3♦ = invitational to 3NT with a six-card or longer suit"; 4♦/4♥ transfers only over 3NT |
| 2NT: 3-level transfers (JSON) | "3♦, 3♥ = Jacoby transfers" |
| `Gerber`, not only over openings | "4♣ IS GERBER OVER ANY 1NT OR 2NT BID BY PARTNER INCLUDING A REBID" |
| `Quantitative 4NT` | "A direct raise of 1NT to 4NT is natural and invites 6NT" |
| `Blackwood 0123` (plain), `King ask by 5NT` | "Blackwood 4NT ... 5♣ = 0 or 4 aces ..."; "5NT is then used to ask for kings" |
| `Jacoby 2NT` | "2NT = Game-forcing raise ('Jacoby 2NT'), 13+ dummy points" |
| `1M-3M inviting` | "3♥ = limit raise (10-12 dummy points...)" |
| `Strong jump shifts 2` and `3`, `Inviting Jump Shifts` off | "2♠, 3♣, 3♦ = strong jump shifts" |
| Forcing 1NT off | "1NT = 6-10 points ... NOT forcing" |
| Inverted minors off; 1m–2NT game forcing 13-15, 3NT 16-17 (JSON) | "There is no forcing minor-suit raise"; "1♣ -- 2NT = 13-15, game forcing"; "3NT = 16-17" |
| Weak 2♦/2♥/2♠, 2NT feature ask, Ogust off, new suit forcing | "Weak two-bids show a six-card suit ... 5-11 HCP"; "bids another suit to show a 'feature'"; "A new suit response is forcing one round" |
| `Fourth suit` (one round) | "If it is a fourth suit in the auction, it may be artificial" (a one-round force) |
| `Michaels Cuebid`, `Unusual 2NT` | "(1♦) -- 2♦ = at least 5-5 in the majors"; "(1♠) -- 2♠ = at least 5-5 in hearts and a minor"; "A jump overcall of 2NT shows at least 5-5 in the lower two unbid suits" |
| Overcalls 8-16, weak jump overcalls (JSON) | "Overcalls show 8-16 points"; "Jump overcalls are preemptive" |
| 1NT overcall 15-18, balancing 10-15 (JSON) | "A 1NT overcall shows 15-18 points"; "A reopening 1NT ... shows 10-15 points" |
| Negative doubles through 2♠ (JSON) | "Negative doubles are used through 2♠" |
| Takeout doubles of preempts through 4♦ (JSON) | "Doubles are for takeout over opening partscore bids (4♦ or lower)" |
| `Jordan Truscott 2NT`; redouble 10+, 1-level new suit forcing, 2-level not, jump shifts weak (JSON) | "2NT = limit raise (at least 10 points) -- or better"; "Redouble = 10 points or more"; "1♥, 1♠ = forcing"; "2♣ = non-forcing"; "2♥, 2♠, 3♣ = six-plus-card suit, like a weak two-bid" |
| `1X-(Y)-2Z forcing`, `1X-(1Y)-2Z strong` | "Bids mean the same thing they meant without the intervening bid" |
| SOS redouble (JSON) | "SOS, requesting a different suit, if your side is doubled for penalty ... at the three level or lower" |
| Off, as the booklet does not have them | Responsive and support doubles, splinters, Drury, Bergen, NMF, Checkback, Smolen, Texas, Lebensohl, Cappelletti (natural vs 1NT), DOPI/ROPI, Walsh (up the line), `Cue bid` (control bids) |
| Carding: standard attitude and count | "high encourages, low discourages" |

Coverage (`rbb card coverage`): 96% of the `.bbsa` (25 read, 1 ignored:
`Strong jump shifts 3`; 2 keys with no field), 53% of the JSON (the JSON
records the booklet's ranges and lengths, which no rule reads yet).

## Natural

The plainest card: SAYC's openings (5-card majors, 15-17 1NT, 2NT 20-21, strong
2♣, weak twos with the 2NT feature ask), Stayman, plain Blackwood with the
5NT king ask, quantitative 4NT, negative and takeout doubles, natural
overcalls (weak jump overcalls), strong jump shifts, limit raises, and every
other convention off: no Jacoby 2NT, Michaels, Unusual 2NT, Jordan 2NT,
Gerber, inverted minors, fourth suit forcing, NMF, splinters, Drury, Texas,
super-accepts, Lebensohl, control bids, responsive or support doubles. The
JSON also turns the Jacoby transfers off (1NT–2♥ to play), and the rules
then fall back on natural responses to 1NT.

Its `.bbsa` cannot: **BBA plays Stayman and Jacoby transfers on every card**
(`bbsa-map.toml` `[implied]`), so `Natural.bbsa` transfers over 1NT and 2NT
(that is what BBA plays with it). Our engine plays the JSON whenever a stock
card has one (`rbb_assets::card_json`, the `.test` runner, `rbb` and the
WASM build, 2026-10-10), so the stock `Natural` is transfer-free everywhere
but in BBA. Coverage 94% (`.bbsa`), 54% (JSON).

## Natural-4CM

`Natural` with four-card majors (`System type = 4`, acol: 1♥/1♠ may be four
cards in every seat), for natural four-card-major systems (Vienna, Pidgin's
natural bot). **Our rules do not read the major's length yet**
(`major_openings.min_length_*` are "switched on that no rule reads"), so we
bid it exactly as `Natural`; it is right for BBA and as a description of the
opponents' system only. Coverage 84% (`.bbsa`).

## Brill

What the Brill robots play, for the Brill dailies, where Rusty sits South
with three Brill robots (partner and both opponents). Built from `21GF-GIB`
(Brill's system is a GIB-style 2/1) with the card-level differences that
`probes/tools/brill_probe.py` found against Brill's own explanations
(Thorvald Aagaard's public Brill service, `/explain`, answers cached in
`.rbb-cache/brill-service/explain/`, 2026-10-15). Each setting and Brill's
rule text behind it:

| Setting | Brill /explain |
|---|---|
| `notrump.minor_transfers = two_nt_three_c` (2NT clubs, 3♣ diamonds, always completed; `transfers.two_nt_clubs`, `three_c_diamonds` on, `two_s_clubs` off) | 1NT–2NT "MinorTransfer" `C >= 6 and (hcp >= 13 or hcp <= 7 or shouldinvestigateslam('C'))`; 1NT–3♣ the same with `D >= 6`; the completions 3♣ / 3♦ `true`, no super-accept |
| `notrump.minor_suit_stayman.play` | 1NT–2♠ "MinorStayman -- both minors, no 4-card major" `C >= 5 and D >= 4 and H <= 3 and S <= 3 and totalpoints >= 10`; answers 2NT `min and D <= 3 and C <= 3`, 3♣/3♦ "Shows 4+ minor", 3NT `max`, 4♣/4♦ "Shows 5+" |
| `notrump.two_nt.minor_stayman` on, `two_nt.minor_transfers` off | 2NT–3♠ "MinorStayman" `C >= 4 and D >= 4 and CombinedPointsMin >= 31` or `C >= 5 and D >= 5 and CombinedPointsMin >= 25`; no 3♠ relay |
| `notrump.transfers.super_accept_doubleton` (and `super_accept`) | 1NT–2♦–2♠ "Superaccept, doubleton" `max and S <= 2 and hearts > 3`; 2NT "Superaccept, no doubleton" `max and ExplicitShape == '3=4=3=3'` |
| `notrump.three_nt.range_min/max = 25/27` | 3NT opening "Balanced hand, 25-27 HCP" |
| `nt_overcalls.direct.range 15-17`, `balance.range 11-15` | "Overcalling 1Mi with 1N" "15-17" `hcp>= 15 and hcp<= 17`; "Balancing 1Mi Opening" `hcp >= 11 and hcp<= 15 and balish` |
| `general.jump_shifts.soloway` | 1m–2♥ "Soloway" `hcp >= 16 and (solid('H') or semisolid('H') or H >= 7) and singlesuited and controls >= 4` (and the 5-4 and 6-card balanced types) |
| `general.system = brill` | Turns on Brill's rule-level treatments (below) |

As 21GF-GIB and checked against Brill: 15-17 1NT, Stayman and Smolen,
Jacoby transfers, Texas, 1NT–3♦/3♥/3♠ splinters ("SplinterOver1N"),
Cappelletti and a penalty double of 1NT, Gerber, quantitative 4NT,
inverted minors, Jacoby 2NT, splinters, two-way game tries
("TwoWayGameTrySpades": 2NT shortness, 3x help suit), Drury, Jordan 2NT,
Lebensohl over their overcall of 1NT and after doubling a weak two, NMF,
fourth suit, support doubles, Michaels and the Unusual 2NT, weak twos with
the 2NT feature ask, inviting jump shifts to three of a minor (1M–3m
"Nat, Inv, 10-12 hcp"), no Gambling 3NT.

Kept from 21GF-GIB against Brill's label: 1M–1NT is not forcing on the
card. Brill calls it "Forcing" and its opener bids a three-card minor over
it, but `one_nt_response.forcing = true` measured slightly worse in the
probe (bids agreeing 66.0% -> 65.8%); a question for Rick.

**Rule-level treatments** (`general.system = brill`): what no card field
says. `general.style` has no value for them (convention-card's options are
`standard` and `bba`), so until it takes a `brill` value the rules key on
the text field `general.system`, which only this card sets. Each is in its
module's notes ("Brill"), with Brill's rule text:

- weak twos: no five-card side major, a four-card one allowed (preempts);
  three-level preempts likewise;
- the transfer completion denies four trumps (but a minimum 4-3-3-3); 3M
  is four trumps and a minimum, 2NT a maximum 4-3-3-3 (jacoby-transfers,
  super-accept, superaccept-doubleton);
- the limit raise 1M–3M has four trumps; three-card limit raises bid 1NT
  first (responses);
- Soloway has 16+ HCP (responses);
- the 1NT overcall may be semi-balanced with a five-card major (overcalls);
- Cappelletti: 10+ HCP, 2♦ five-five in the majors (cappelletti);
- 3NT opening without a five-card major (strong-openings);
- Jordan 2NT with three trumps (vs-takeout-double);
- Gerber over a 3NT opening (gerber).

Measured (`brill_probe.py report --card Brill`, the top 2000 corpus
auctions, our reading against Brill's): see the commit messages and the
modules' notes. The `.bbsa` (`bbsa/Brill.bbsa`, exported from the JSON)
loses the ranges, the 1NT overcall and `general.system`, which have no
`.bbsa` key; our engine plays the JSON.

## Known gaps (fields or mappings missing)

For a convention-card pull request:

- No `.bbsa` key turns Stayman or Jacoby transfers off (`[implied]`), so no
  stock card can be transfer-free. A non-BBA key, or JSON stock cards, would
  do it.
- `notrump.minor_transfers = relay` (SAYC's 2♠ to 3♣) has no `.bbsa` form:
  the `[[derived]]` rule turns every `.bbsa` without BBA's 2♠/3♣ transfer
  pair into `none`. The stock SAYC therefore has no 2♠ relay (the JSON has).
- SAYC items with no field at all: the Grand Slam Force (jump to 5NT), the
  cue bid of their overcall over our 1NT as Stayman, 3♣/3♦ over 1NT
  invitational with six, 1M–3NT 15-17 with two-card support, 1M–4M weak with
  a singleton, Gerber's 5♣ king ask, the overcaller's cue-bid response asking
  about quality, "a double of their convention shows 10+".
- Fields with no `.bbsa` key (lost in the export; the rules' defaults apply):
  `overcalls.jump`, `nt_overcalls.*`, `doubles.negative.*`,
  `vs_to_double.*`, `vs_preempts.*`, `minor_openings.two_nt_response`,
  `notrump.stayman.forcing`, `notrump.two_nt.transfers_3level`,
  `other_conventions.sos_redouble.play`, the weak-two ranges.

## Questions for Rick (decisions taken)

1. Natural without transfers: the stock `.bbsa` cannot say it. Taken: the
   stock Natural transfers (as BBA's would); the JSON does not. Should stock
   cards also come as JSON (embedding `cards/json/`), or should
   convention-card map a key that turns the implied fields off?
2. Control bids (`Cue bid`) are off on SAYC and Natural: the booklet slams
   by Blackwood and Gerber only. GIB's SAYC on BBO does cue-bid. Keep off?
3. Texas off on SAYC: the booklet has 4♦/4♥ transfers only over 3NT.
4. Natural-4CM is only a label for us until the rules open four-card majors.
5. Seat API names: `SAYC` and `Natural` are these cards; `21GF` would be
   `21GF-DEFAULT` and `21GF-GIB` is the stock card of that name. Aliases
   belong in the seat service, not here.
