# gambling-3nt (`gambling-3nt.bid`): notes

Gambling 3NT (`notrump.three_nt.one_suit`): a solid seven-card minor
(AKQ at least, eight allowed) and no outside ace or king, first to third
seat. strong-openings.bid already read the field (the balanced 26-27
opens 2♣ when 3NT is Gambling) but nothing opened the Gambling 3NT. Every
21GF stock card has the field on, so this changes the 21GF scenarios
wherever such a hand turns up (it opened 3♣/3♦ or one of the minor
before). Cases: `gambling-3nt.test`.

## Structure

- Responder: pass with spades, hearts and a minor stopped and 10+ HCP;
  4♣ pass or correct (the default); 5♣ pass or correct with shortness in
  a major and three cards in each minor; 6♣ pass or correct with 16+ and
  four quick tricks; 4♥/4♠ to play with seven, or six and 10+.
- Opener passes clubs or corrects to diamonds at the same level.
- The pass-or-correct calls are alerted but not `artificial`: a pass
  plays them.

## Corpus (Gambling_3N, 50 boards, 2026-10-05)

87.3% of calls agree, no problems in our auctions. Remaining
divergences:

- The opponents' overcalls over 3NT (BBA 4♥/4♠/X, we pass): competitive/,
  not this module. **Don't care here** (reported to the competitive
  owner).
- BBA passes 3NT on some hands where we bid 4♣ (3 boards) and bids 7♦
  once where we bid 6♣: **BBA style**, left.

## Questions for Rick

- 4♦ as an asking bid (shortness, some play it) is not built: 4♦ has no
  meaning. Decision: leave it out, as the SAYC booklet.
- Fourth seat: no Gambling 3NT (seat<=3), as Namyats. Decision taken.

## Sources

- **ACBL SAYC booklet** (Gambling 3NT: a solid seven-card minor, no
  outside ace or king; 4♣/5♣ pass or correct) and **Bridge Bum,
  "Gambling 3NT"** (https://www.bridgebum.com/gambling_3nt.php), which
  the Gambling_3N scenario cites; the scenario's script allows a queen or
  two jacks outside (9-11 HCP).
- **Where we differ:** responder's pass, 5♣ and 6♣ thresholds are ours
  (standard practice, not yet cited).
