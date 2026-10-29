# "Bridge robots, one year later": notes and what it means for Rusty

Luc Bellicaud's BridgeWinners article (October 28, 2026),
https://bridgewinners.com/article/view/bridge-robots-one-year-later/, and the
comments under it, read 2026-10-29. Rick agreed with the analysis below.

## The article

Bellicaud is a French former junior international who has built **Lia**,
IntoBridge's robot, mostly on his own for about three years, so he has a stake
in what he says about it. The article follows up his earlier one on why bots
still trail expert players.

- **How robots still work.** Most still use GIB's method from 2001: deal hands
  that fit the auction and the play so far, solve each one double-dummy, and
  choose the call or card that does best on average. He lists four weaknesses:
  - **Bidding:** off its rules, a robot simulates and decides on the spot. It
    never bids to find something out.
  - **Choosing deals:** when none of its imagined deals looks like the real
    one, it defends oddly.
  - **Planning:** a double-dummy solver misses human lines like elimination
    plays, which gain nothing against perfect defence.
  - **Opponents:** it never asks what an opponent's action, or a missing
    action, means.
- **What changed is coding agents.** Coding a full 2/1 system used to take him
  a week. Now he gives an agent the system's PDF. The agent writes the rules
  and plays millions of deals to find problem sequences, and he reviews and
  corrects the important ones. Testing an idea takes hours instead of weeks;
  he puts his capacity at roughly 20 times what it was. Lia went from solving
  about one Bridge Master level-3 problem in two to more than four in five in
  two weeks.
- **LLMs still don't understand bridge.** He shows two confident, wrong LLM
  analyses. "Code has become almost free; understanding bridge has not." He
  still has to supply the bridge judgment.
- **His yardstick** is every auction from the World Championships: "When the
  whole field agrees on a call and the robot does something else, it is always
  the robot that is wrong."
- **Predictions:** a robot has a 50% chance of reaching world-class level
  within a year and 90% within two (he gives no basis). Human bridge will
  change little, as chess did. He suggests a federation enter a team of four
  robots in its top division.

## The comments

Most of the comments are about card play. Six threads matter for a bidding
engine:

- **Learning an unknown system** (Martin Carpenter). Bellicaud: don't get the
  LLM to understand the system. Have it build test protocols: give the robot a
  set of chosen hands and auctions, see what it bids, and turn the results into
  rules. Going the other way, the LLM writes a readable explanation from the
  rules.
- **A swappable bidding-system module** (Christopher Garrard). Bellicaud:
  - Bots bid by rules. Simulation takes over only when the rules run out, and
    a simulated bid carries no meaning partner can read.
  - A system includes the judgment of which allowed bid to choose, so "the
    same logic for every system" is harder than it sounds.
  - Bot-versus-bot comparisons measure how well a bot handles a system, not
    whether the system is good.

  Thorvald Aagaard adds that he is building a bidding language for Brill.
- **A player's complaints** (Steven Allen):
  - Robots open rubbish, count distribution before a fit is found, and
    game-force on poor minimums.
  - They almost never double for penalties.
  - They ignore vulnerability and scoring, for example wild preempts red
    against white at IMPs.

  Bellicaud agrees that overbidding and penalty doubles are what he is working
  on now.
- **Cooperative slam bidding** (Alan Shillitoe). Bots miss what partner's
  choice of bid implies, such as extras or a control. Bellicaud: robots read
  what a bid shows much better than what the choice of it implies.
- **A hybrid with an LLM** (Koen Grauwels). Bellicaud would have an LLM read
  the convention card and turn it into instructions for the program, which
  then plays. The LLM would not decide at the table.
- **An LLM solved a play problem.** Koen's model found the right line in a
  6♥ deal, and found the new one when two heart spots were swapped. That cuts
  against the article's claim that LLMs don't understand bridge.

## What it means for Rusty

### Where we already work this way

- **His method is ours.** Rusty is rule-based, conventions are written from a
  cited source, and a coding agent turns the source into rules. Modified
  Jacoby 2NT (v0.11.0) came straight from Larry Cohen's article. The agent
  checks the rules on many deals (`compare`, self A/B), and Rick rules on the
  judgment calls. Bellicaud's point that the agent writes the code but cannot
  supply the bridge sense matches our experience: the real fixes come from a
  ruling or a bridge reason, not from the code.
- **His "field agrees" test is our BBA and Brill comparisons.** We compare
  against BBA's 500-board corpora and the Brill field. He treats the field as
  right; our rule is "par decides, BBA teaches" (docs/DESIGN.md). That is
  stricter about where a consensus is wrong, but the field is still the right
  place to find problems.
- **Garrard's swappable module is Rusty's design.** We have a rule language
  (docs/LANGUAGE.md), conventions separate from the engine (the planned repo
  split), the contract in docs/CONTRACT.md, and a WASM build that takes a rule
  set as text. Bellicaud's view that bids must carry meaning is why we have no
  simulation fallback: every call comes from a rule that says what it shows.
- **His test protocols are our probes.** `rbb probe` and `rbb grid` treat BBA
  as a black box with chosen hands; that is how we proved what BBA's "Mixed
  raise" key means. `brill_probe` does the same against Brill. His
  "explanation from the rules" is our `explain-auction`, the workbench's
  knowledge view and `rbb bid reference`.
- **Koen's hybrid is how we work now.** Claude reads the card or the cited
  source and writes rules, and the engine bids. `card import-bbsa` and the
  convention-card vocabulary are the card-reading half.

### Where we differ

- **His bidding weakness is what our judgment layer works on.** He says
  robots outside their rules don't bid to find things out. Our relays, keycard
  and the planned judgment layer (placement, competition, slam entry;
  docs/JUDGMENT-LAYER.md) build that in instead of simulating each call. The
  specific-kings result shows the same thing: the asker needs to count tricks,
  not just collect answers.
- **His planning weakness belongs to card play.** Rusty only bids; at the
  Brill tables BEN plays the cards. We use double-dummy only as a yardstick
  (par, the errors line), never to choose calls. That spares us his
  deal-choosing problem, but our par score shares his blind spot: it rewards
  contracts that make only against perfect play.
- **The goal is different.** He aims for world-class strength, and Lia runs on
  simulation. Rusty's goal is a practice partner that plays Rick's card
  faithfully, so readable rules, explanations and convention coverage matter
  more than the last few IMPs. Sticking closely to the cited sources is where
  Rusty can stand out.
- **Macro Xue's open-source double-dummy solver**, mentioned in the article,
  may be worth a look next to `bridge-solver` if speed ever matters for
  `--par-all` or random-deal runs.

### Warnings that apply to us

1. **Bot-versus-bot results measure the bot, not the system.** Our self A/B
   measures how well Rusty handles a convention, not whether the convention is
   good. Specific kings is the example: the loss comes from our grand-slam
   judgment (no trick counting), not from the convention. When a convention
   loses, its notes should say which of the two caused it.
2. **Steven Allen's complaints are worth checking against Rusty.** The errors
   yardstick already charges an overbid as if doubled, Phase 3 of the judgment
   layer is penalty doubles, and we count support points only once a fit is
   known. We have not measured how light we open, game-force or preempt by
   vulnerability. A grid or random-deal probe for each would answer it
   cheaply.
3. **Inferences from the choice of bid are probably a gap.** As far as we
   know, what Rusty knows about a hand comes from what the chosen rule
   `shows`, not from the bids partner could have made and didn't. That belongs
   with the engine-knowledge work and the slam work.
