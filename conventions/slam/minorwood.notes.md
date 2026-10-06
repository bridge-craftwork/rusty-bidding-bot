# minorwood (`minorwood.bid`): notes

Minorwood (2026-10-05). Gated on `slam.minorwood.play`; off when the card
also plays Kickback (kickback.bid is then the ask). No stock card plays
it and the `.bbsa` format has no key for it.

## What the rules do

- **The ask:** with a minor agreed, four of it while the auction is
  below that: in a game force with rkcb-1430.bid's values (33 on
  partner's floor), or uncontested in a forcing or limited auction (33
  on partner's maximum, no two bare suits). The source: "if there is any
  question about what is going on, 4m is forcing and we should usually
  treat it as Minorwood". Priority 11, as Kickback, so 4m reads as the
  ask whatever else could claim it.
- **Answers:** four steps above the ask, 1430 or 0314 as the card's
  keycard ask has them in a minor (`kc_first_14`, dopi-ropi.bid: the
  `slam.blackwood.minor_0314` treatment, or `general.style=bba` with
  0314). Over 4♣: 4♦ 4♥ 4♠ 4NT; over 4♦: 4♥ 4♠ 4NT 5♣.
- **The asker:** grand, six, or five of the minor (partner corrects to
  six with the higher count of an open answer, `ask=mw_correct`).
- rkcb-1430.bid's trump-agreed 4NT asks step aside while Minorwood
  applies (`mw_asks`).

## Gaps and open questions

- With Minorwood on, any uncontested 4m in a minor-agreed auction reads
  as the ask, so the natural 4m calls other modules make (a raise to
  four, "game try in the minor") become unavailable in meaning. That is
  the convention's price; the source accepts it.
- Not in competition below a game force (the source: "not in
  non-forcing competitive auctions").
- No queen ask, and no 5m+1 king ask (the source's follow-ups).

## Corpus

Minor_Game_Or_Slam `--limit 50 --set slam.minorwood.play=true`: 7 asks,
7 boards changed, +26 IMPs against BBA by par (-243 against -269
without). Mostly 1m-2m(inverted)-4m, which our base bid as a natural
non-forcing raise and partner passed; as Minorwood it reached slams that
make.

## Self A/B (2026-10-05)

BBA does not play it, so it is judged against ourselves (CLAUDE.md,
"Pace"): the same deals with the card field off and on, every board,
`probes/tools/self_ab.py --batch probes/self-ab.toml --only minorwood`.
IMPs by the errors yardstick, positive when the convention makes fewer;
"actor" is the side that made the first differing call.
Minor_Game_Or_Slam: 96 boards changed; actor contract +492 (with the 5NT
king ask added, +478 before), double-dummy +641, halves +236/+256 (z
+8.5): gains. Mostly the 1m-2m-4m auctions our base passed out in four
(Corpus, above).

## Sources

- **The convention:** Robert S. Todd, "Slam Bidding: Minorwood",
  Advancing in Bridge #405
  (https://www.advinbridge.com/this-week-in-bridge/405): when 4m is
  Minorwood (game forcing with a minor fit and no major fit; jumps that
  are clearly not weak), the 1430 step answers over 4♣ and 4♦, the queen
  and king asks. Convention-card
  `spec/conventions/bidding_conventions/minorwood.toml`.
- **Where we differ:** no queen or king ask.
