# Random deals against scenario deals

Rick, 2026-09-28: "run our BBA compare against the Pavlicek random solved
deals ... it would be interesting to see if there's a change in relative
performance when we look at random deals vs scenario deals."

The Practice-Bidding-Scenarios corpus is made of selected hands: each
scenario deals only boards where its convention comes up. Random deals are
what a table actually meets: mostly partscores, half of them contested, few
slams. This compares us with BBA on 10,000 random deals and on the corpus
scenarios bid with the same cards.

## Data and terms

The deals and double-dummy tables are **Richard Pavlicek's library of
10,485,760 random solved deals, (c) 2007, [rpbridge.net](https://www.rpbridge.net/)**
([`rpdd.txt`](https://www.rpbridge.net/d/rpdd.txt)). His terms: noncommercial
use, unmodified, credited to him. Nothing of his is in this repository; the
sample and every file made from it live in a scratch directory. The local
copy is `~/Development/GitHub/rpdd-library/rpdd.zrd` (his 241 MB file as
`rpdd.bat` builds it; `rpdd zrd` from rpdd-reader rebuilds it from the
tables).

## Method

1. **Sample.** `probes/tools/rpdd_sample.py` reads `rpdd.zrd` and writes a
   PBN: deal *k* (0-based) of a sample of *n* is library index
   `floor(k * 10485760 / n) + offset`, evenly spaced over the whole
   library, so the same arguments always give the same deals. The pilot is
   `--count 10000 --offset 0` (indices 0, 1048, 2097, ... 10484711).
   Board *k*+1 takes the standard rotation's dealer and vulnerability,
   `[Scoring "MP"]` (as every corpus file), `[RpddIndex "i"]` for
   traceability, and the deal's table as an `[OptimumResultTable]`, which
   `rbb compare` reads, so par is free and nothing is solved. The decoder
   was checked against the bridge-encodings fixture record, and par from
   those tables agrees with the engine's on every board (below).
2. **BBA.** bba-cli bid the same 10,000 deals twice: NS 21GF-DEFAULT /
   EW 21GF-GIB (the pair 198 of the 343 scenarios use) and Basic-Bridge for
   both sides (24 scenarios).
3. **Compare.** The two bba-cli outputs go in a scratch PBS-like directory
   as `bba/Random_Pavlicek_21GF.pbn` and `bba/Random_Pavlicek_Basic.pbn`,
   with `bbsa` a link to the corpus's `bbsa/`. The cards come from the
   `% CC1/CC2` header bba-cli writes. `rbb compare --pbs DIR --json`.
4. **Corpus.** The whole corpus, `--json`, restricted to the boards bid
   with the same card pair (`ns_card`/`ew_card` in the JSON).
5. **Split.** `probes/tools/compare_split.py` gives the table below per
   class of board: by who bid in BBA's auction (the `--auctions` classes)
   or `--by par` by the size of the par score. `rbb compare` fills in par
   only where the contracts differ; for the "|par|" and "at par" columns
   the script works out par and the contract scores itself from the
   table (`--check-par`: it matches the engine on all 78,770 boards where
   both exist). `probes/tools/par_blame.py` for how the auctions miss
   par.

### Reproduce

```sh
S=<scratch dir>; PBS=~/Development/GitHub/Practice-Bidding-Scenarios
python3 probes/tools/rpdd_sample.py --zrd ~/Development/GitHub/rpdd-library/rpdd.zrd \
    --count 10000 -o $S/rpdd-10k.pbn
mkdir -p $S/pbs/bba && ln -s $PBS/bbsa $S/pbs/bbsa
BBA="/Applications/Bridge Utilities/bba-cli"
nice -n 10 "$BBA" -i $S/rpdd-10k.pbn -o $S/pbs/bba/Random_Pavlicek_21GF.pbn \
    --ns-conventions $PBS/bbsa/21GF-DEFAULT.bbsa --ew-conventions $PBS/bbsa/21GF-GIB.bbsa
nice -n 10 "$BBA" -i $S/rpdd-10k.pbn -o $S/pbs/bba/Random_Pavlicek_Basic.pbn \
    --ns-conventions $PBS/bbsa/Basic-Bridge.bbsa --ew-conventions $PBS/bbsa/Basic-Bridge.bbsa
cargo build --release -p rbb-cli
nice -n 10 ./target/release/rbb compare --pbs $S/pbs --json $S/random.json --by-imps
nice -n 10 ./target/release/rbb compare --pbs $PBS --json $S/corpus.json
T=probes/tools
python3 $T/compare_split.py $S/corpus.json --cards 21GF-DEFAULT/21GF-GIB --write-subset $S/corpus-21gf.json
python3 $T/compare_split.py $S/corpus.json --cards Basic-Bridge/Basic-Bridge --write-subset $S/corpus-basic.json
python3 $T/compare_split.py $S/random.json --scenario Random_Pavlicek_21GF --write-subset $S/random-21gf.json
python3 $T/compare_split.py $S/random.json --scenario Random_Pavlicek_Basic --write-subset $S/random-basic.json
python3 $T/compare_split.py $S/random-21gf.json --by par      # and the others
python3 $T/par_blame.py $S/random-21gf.json
```

Rules at commit bf5c9e0 (2026-09-28).

## Throughput

bba-cli, `nice -n 10`, on a machine shared with other corpus runs: 10,000
deals in 113 s with the 21GF pair (about 90 deals a second) and 66 s with
Basic-Bridge (150 a second). `rbb compare` on the 20,000 random boards: 20
s. The sample itself: 0.2 s.

## Results

Columns: *calls*, our call against BBA's at each position of BBA's auction;
*auction*, identical auctions; *contract*, same final contract; *vs BBA*,
on boards where the contracts differ, BBA's IMP distance from par minus
ours (positive: ours closer), summed, then *per bd* over every board of the
class; *ours/BBA closer*, *equal*, those boards counted; *|par|*, mean IMP
distance from par over every board, and *at par*, the share exactly at
par, for BBA's contract and ours.

### 21GF-DEFAULT / 21GF-GIB

Corpus: 198 scenarios, 98,505 boards (94,005 with a table).

| class | boards | share | calls | auction | contract | vs BBA | per bd | ours closer | BBA closer | equal | \|par\| BBA | \|par\| ours | BBA at par | ours at par |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| all | 98505 | 100.0% | 78.0% | 16.6% | 34.7% | -66075 | -0.70 | 19401 | 29218 | 12639 | 4.98 | 5.68 | 34.1% | 28.1% |
| passed out | 625 | 0.6% | 93.4% | 74.9% | 74.9% | +126 | +0.20 | 92 | 55 | 10 | 3.55 | 3.34 | 0.0% | 8.3% |
| NS only | 44633 | 45.3% | 82.1% | 21.9% | 45.7% | -35771 | -0.84 | 6407 | 11021 | 5681 | 4.99 | 5.82 | 41.4% | 35.0% |
| EW only | 4179 | 4.2% | 86.2% | 40.5% | 50.8% | -1239 | -0.30 | 666 | 982 | 398 | 4.34 | 4.63 | 30.3% | 27.3% |
| competitive | 49068 | 49.8% | 73.1% | 9.0% | 22.8% | -29191 | -0.63 | 12236 | 17160 | 6550 | 5.05 | 5.68 | 28.2% | 22.2% |

Random: 10,000 boards.

| class | boards | share | calls | auction | contract | vs BBA | per bd | ours closer | BBA closer | equal | \|par\| BBA | \|par\| ours | BBA at par | ours at par |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| all | 10000 | 100.0% | 79.5% | 18.3% | 34.1% | -4874 | -0.49 | 2079 | 2999 | 1514 | 4.56 | 5.05 | 34.0% | 29.1% |
| passed out | 29 | 0.3% | 96.6% | 86.2% | 86.2% | +4 | +0.14 | 2 | 1 | 1 | 3.59 | 3.45 | 0.0% | 6.9% |
| NS only | 2438 | 24.4% | 85.3% | 29.6% | 45.7% | -1156 | -0.47 | 385 | 587 | 353 | 4.36 | 4.83 | 39.7% | 34.9% |
| EW only | 2507 | 25.1% | 86.1% | 31.3% | 49.4% | -1049 | -0.42 | 356 | 566 | 346 | 4.49 | 4.91 | 38.2% | 33.6% |
| competitive | 5026 | 50.3% | 73.4% | 6.0% | 20.5% | -2673 | -0.53 | 1336 | 1845 | 814 | 4.70 | 5.23 | 29.3% | 24.2% |

### Basic-Bridge, both sides

Corpus: 24 scenarios, 11,656 boards (11,000 with a table).

| class | boards | share | calls | auction | contract | vs BBA | per bd | ours closer | BBA closer | equal | \|par\| BBA | \|par\| ours | BBA at par | ours at par |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| all | 11656 | 100.0% | 85.7% | 39.1% | 58.5% | -2357 | -0.21 | 1572 | 2035 | 909 | 4.57 | 4.79 | 41.5% | 40.0% |
| NS only | 9173 | 78.7% | 88.9% | 46.6% | 66.4% | -1341 | -0.15 | 1005 | 1294 | 587 | 4.53 | 4.68 | 44.3% | 43.3% |
| EW only | 127 | 1.1% | 86.2% | 29.1% | 45.7% | -83 | -0.65 | 24 | 36 | 9 | 3.90 | 4.55 | 34.6% | 30.7% |
| competitive | 2356 | 20.2% | 74.3% | 10.5% | 28.3% | -933 | -0.44 | 543 | 705 | 313 | 4.79 | 5.23 | 30.7% | 26.9% |

Random: 10,000 boards.

| class | boards | share | calls | auction | contract | vs BBA | per bd | ours closer | BBA closer | equal | \|par\| BBA | \|par\| ours | BBA at par | ours at par |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| all | 10000 | 100.0% | 81.6% | 22.4% | 36.0% | -4831 | -0.48 | 2057 | 2923 | 1424 | 4.59 | 5.07 | 33.7% | 29.1% |
| passed out | 29 | 0.3% | 96.6% | 86.2% | 86.2% | +4 | +0.14 | 2 | 1 | 1 | 3.59 | 3.45 | 0.0% | 6.9% |
| NS only | 2543 | 25.4% | 87.6% | 34.5% | 48.5% | -1274 | -0.50 | 380 | 588 | 341 | 4.36 | 4.86 | 39.6% | 34.8% |
| EW only | 2576 | 25.8% | 88.1% | 36.5% | 50.5% | -956 | -0.37 | 388 | 572 | 316 | 4.57 | 4.94 | 37.0% | 33.2% |
| competitive | 4852 | 48.5% | 75.2% | 8.2% | 21.4% | -2605 | -0.54 | 1287 | 1762 | 766 | 4.73 | 5.27 | 29.1% | 24.0% |

Standard errors of "per bd" over the whole set: ±0.04 for each random set
(boards are independent), ±0.016 for corpus 21GF and ±0.04 for corpus Basic
if boards were independent (they are not: boards of a scenario share a
theme, so the real uncertainty of a corpus figure is larger).

### By the size of par

Par under 300 is a partscore (or a cheap sacrifice), 300-910 mostly game,
920 and up a slam. No board has par 0.

| set | band | share | per bd | \|par\| BBA | \|par\| ours | BBA at par | ours at par |
|---|---|---:|---:|---:|---:|---:|---:|
| corpus 21GF | < 300 | 31.6% | -0.07 | 3.08 | 3.16 | 32.8% | 27.7% |
| | 300-910 | 42.7% | -0.32 | 4.59 | 4.91 | 38.6% | 35.4% |
| | 920+ | 21.1% | -2.41 | 8.61 | 11.02 | 27.0% | 14.0% |
| random 21GF | < 300 | 42.7% | -0.21 | 2.67 | 2.88 | 35.3% | 30.2% |
| | 300-910 | 44.3% | -0.35 | 4.61 | 4.95 | 38.9% | 35.6% |
| | 920+ | 12.9% | -1.87 | 10.63 | 12.51 | 12.8% | 3.3% |
| corpus Basic | < 300 | 25.0% | -0.32 | 2.75 | 3.07 | 35.9% | 31.2% |
| | 300-910 | 50.5% | +0.13 | 3.32 | 3.19 | 54.5% | 55.7% |
| | 920+ | 18.9% | -0.99 | 10.33 | 11.32 | 14.2% | 9.6% |
| random Basic | < 300 | 42.7% | -0.22 | 2.65 | 2.87 | 35.7% | 30.7% |
| | 300-910 | 44.3% | -0.32 | 4.66 | 4.98 | 38.1% | 35.3% |
| | 920+ | 12.9% | -1.91 | 10.75 | 12.66 | 12.0% | 2.4% |

### How the auctions miss par (par_blame.py)

Boards whose contracts differ (the JSON carries par only for those), IMPs
from par. Random 21GF, 6,592 boards; the Basic run is within a few percent
of it on every line.

| how the auction missed par | ours | IMPs | BBA | IMPs |
|---|---:|---:|---:|---:|
| below par: W stopped short of slam | 678 | 8717 | 465 | 5736 |
| below par: W stopped short of game | 805 | 6318 | 851 | 6805 |
| below par: W let them play (no double) | 702 | 4598 | 510 | 2737 |
| above par: W overbid | 818 | 4454 | 950 | 4980 |
| below par: W outbid, did not compete | 553 | 4230 | 290 | 2126 |
| below par: W stopped short (level) | 591 | 3209 | 493 | 2667 |
| below par: W wrong strain | 406 | 3127 | 412 | 2817 |
| L erred: did not compete or sacrifice | 282 | 859 | 413 | 1744 |
| above par: L overcompeted | 331 | 787 | 487 | 1507 |
| L erred: went down more than par | 126 | 371 | 85 | 340 |
| passed out: W never bid | 62 | 302 | 4 | 12 |
| below par: W doubled, par was to bid on | 25 | 153 | 117 | 780 |
| at par | 1213 | 0 | 1515 | 0 |
| total | 6592 | 37125 | 6592 | 32251 |

Corpus 21GF (61,258 boards) has the same shape: slam shortfall first
(ours 119,094 IMPs, BBA 66,198), then game, then "let them play" (44,412
against 19,865) and "outbid, did not compete" (33,635 against 17,593).
The rules blamed most, on both sets: `base.bid:76` "Game reached: nothing
more to say" (slam and level shortfalls) and `advances.bid:266`
"Defending: nothing more to say" (let them play, outbid).

### Top divergence points on random deals

`rbb compare --pbs $S/pbs Random_Pavlicek_21GF --by-imps`, IMPs vs BBA:

| count | IMPs | auction so far | BBA | ours |
|---:|---:|---|---|---|
| 37 | -68 | (opening) | 1H | P |
| 34 | -59 | (opening) | 1C | P |
| 24 | -57 | (opening) | P | 1C |
| 14 | -57 | P | 1H | P |
| 18 | -49 | P P P | 1S | 2S |
| 22 | -45 | 1C | 1D | P |
| 20 | -41 | P | 2S | P |
| 16 | -39 | 1C P | 1H | 1D |
| 4 | -37 | 1D P | 1H | 2H |
| 44 | -36 | (opening) | 1D | P |
| 22 | -32 | P | 1D | P |
| 46 | -31 | (opening) | 2S | P |
| 13 | -31 | (opening) | P | 1S |
| 8 | -31 | 1D | 2H | P |
| 22 | -30 | 1D P | 1H | P |

The Basic run has the same list in a different order. The costs are
spread thin: the top twenty points hold under a fifth of the loss.

Openings, all seats (first difference at a call with only passes before
it): 1,082 boards of 10,000, costing -819 of the -4,874. The main pairs
(21GF; HCP and shapes of the opener's hand):

| BBA | ours | boards | IMPs | hands |
|---|---|---:|---:|---|
| 1H | P | 62 | -126 | 10-12 HCP, 5-4 (5431, 5422) |
| 1C | P | 66 | -83 | 10-13 HCP, 4432 / 4441 |
| 1D | P | 80 | -83 | 10-12 HCP, 4432 / 4441 |
| 2S | P | 90 | -76 | 2-10 HCP, six spades or 5332 |
| P | 1C | 43 | -53 | 11 HCP, 5332 |
| 1S | 2S | 18 | -49 | 12-18 HCP, six spades |
| P | 1S | 31 | -40 | 10-11 HCP |
| 2C | 1H | 12 | -30 | 17-21 HCP, six hearts or 7-4 |

## Findings

- **Random deals do not make us look worse.** With the 21GF pair the
  loss to BBA is -0.49 IMPs a board on random deals against -0.70 on the
  scenarios; agreement is a little higher (79.5% of calls against 78.0%,
  18.3% identical auctions against 16.6%). The scenarios select hands for
  the conventions, many of which we play less well than BBA or not at all.
- **With Basic-Bridge it is the other way**: -0.48 on random deals against
  -0.21 on the Basic scenarios, and 22% identical auctions against 39%.
  The Basic scenarios are simple uncontested NS auctions (79% "NS only")
  where our rules are closest to BBA's; random deals bring in the
  competition and the slams the Basic scenarios mostly leave out.
- **On random deals the card hardly matters.** 21GF and Basic score the
  same (-0.49, -0.48) and have the same problem lists. Most random
  auctions never reach a convention that differs between the cards; what
  decides them is the natural core: opening, overcalling, competing,
  slam.
- **Where the auctions are: half competitive.** 50% of random boards are
  competitive in BBA's auction and 25% each uncontested for either side;
  the corpus 21GF set is 45% NS only and 4% EW only. On random deals the
  loss per board is about the same in every class (-0.42 to -0.54), where
  on the corpus the NS-only scenarios cost most (-0.84).
- **Slams are the biggest single loss everywhere.** Boards with par 920+
  are 13% of random deals and cost half the loss (-1.87 a board). We reach
  par on 3.3% of them, BBA on 12.8%. par_blame puts "stopped short of
  slam" first on both sets, blamed on `base.bid:76` ("Game reached:
  nothing more to say").
- **Partscore competition is the second.** We "let them play" or "did not
  compete" far more often than BBA (1,255 boards and 8,828 IMPs against
  800 and 4,863), and pass BBA's light overcalls (1C-(1D), 1D-P-1H-(1S),
  1D-(2H)). BBA's own weak spot is the other side of it: it doubles where
  par was to bid on (117 boards against our 25).
- **Openings show up on random deals as they cannot in scenarios.** About
  11% of random boards first differ at an opening call. BBA opens 10-12
  HCP 4-4-3-2 and 5-4 hands that our 12-total-point rule passes, and
  passes 11 HCP 5-3-3-2 hands we open (preempts.notes.md accepts the
  second difference on the corpus evidence; on random deals BBA's pass is
  closer to par, -53 IMPs over 43 boards). BBA also opens weak twos we
  pass (2-10 HCP), and a strong 2C on 17-21 HCP one-suiters where we open
  one of a suit. These are candidates for `rbb grid` probes, not
  conclusions: 60-90 boards each.
- **Passed out.** BBA passes out 0.3% of random deals (29 of 10,000; it
  opens light); we pass out 62 boards where par is a contract, BBA 4.

Not surprising, but worth saying: random deals are mostly partscores
(43% have par under 300) and the typical board is a low-stakes one; the
corpus has almost twice the slam share (21% against 13%) because its
scenarios pick strong hands.

## A larger sample

- bba-cli: at 90-150 deals a second, 100,000 deals is 11-19 minutes per
  card pair and the full library 20-32 hours. Split the input into files
  of 50,000-100,000 deals (`--offset` gives disjoint evenly spaced
  samples; a contiguous run needs `--count` and a start, easily added).
- `rbb compare`: about a second per 1,000 boards; the JSON is about 1.9
  KB a board, so 1M boards is ~2 GB. Keep each file a scenario and run
  them in batches, or add a summary-only mode.
- `compare_split.py` loads a whole JSON and computes par in Python for
  boards where the contracts match (about 1 ms a board); fine to 100,000,
  slow beyond.
- 100,000 deals would bring the per-board standard error to about ±0.013
  and give each opening or overcall divergence point several hundred
  boards, enough to decide it by par without probing.
