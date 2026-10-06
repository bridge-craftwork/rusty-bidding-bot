# exclusion (`exclusion.bid`): notes

Exclusion Blackwood (2026-10-05), gated on `slam.exclusion_blackwood.play`
(on in 21GF-GIB, -GIB-Bergen, -SPECIALS2, -WJS-MSS, -PolishTwoSuiters,
Precision).

## What the rules do

- **The ask:** trumps agreed, a void in v (not trumps), the keycard ask's
  slam values (`slam_values | slam_values_limited`), and a **jump** to
  five in v (the cheapest bid in v was at the four level or lower).
  Priority 12: it outranks the 4NT ask and the splinter's own 4NT, and
  ties control bids, where it wins as the more descriptive call.
- **Room:** three keycards of my own, or two when the answer "none" is
  still at or below five of our suit, so a bad answer leaves a sign-off.
- **Answers:** 1430 steps excluding the void ace (four keycards: three
  aces and the trump king): 1 or 4, 0 or 3, 2 without the queen, 2 with.
  Over interference, DOPI/ROPI's first two steps (double/redouble,
  pass); with two keycards, bid six.
- **The asker:** seven with all four keycards and the queen, six with
  three, else five of our suit (pass when that was the answer); a
  double or a pass of partner's double when they interfered. An open
  count ("1 or 4" facing none, "0 or 3" facing one) is read low.
- **`general.style=bba`:** BBA's treatment, steps counting 0, 1, 2, 3
  ("1 out of 4" in its notes) and any five-level bid in the void suit
  asks, jump or not.

## Corpus (Exclusion_After_1M, Exclusion_After_Sta_Jac, --limit 50)

BBA asks 66 times on these 100 boards; we make the same call 9 times.
Almost all the misses are **BBA style**: BBA's ask is a non-jump five-level
bid over a game in our suit (1S-2NT-3C-4S-5C, 1H-(3S)-4H-5D) or over a
splinter (1S-4D-5C), where by the cited definition 5♣ is a control bid,
not Exclusion; we ask 4NT. A few are our room rule (two keycards, the
"none" answer past five of our suit: we ask 4NT). Where we do ask, the
answers and the slam level agree with BBA's reading; the grand is
sometimes missed (we need the queen shown).

## Open questions (for Rick)

- **Jump or not:** we follow the sources (an unusual jump). BBA, and the
  scenarios, treat 5 of a new suit over our game as Exclusion. Should
  the default follow BBA here? (Then control bids past game lose 5♣/5♦.)
- **Responses:** 1430 steps by default (Bridge Bum, Todd's first
  option); Todd prefers 0-1-1-2-2 for the four keycards, BBA 0-1-2-3.
- No queen ask or king ask after an Exclusion answer.

## Sources

- **The convention:** Bridge Bum, "Exclusion Blackwood"
  (https://www.bridgebum.com/exclusion_blackwood.php): a jump to the 4-
  or 5-level in a new suit shows a void, 1430 step answers excluding
  that ace. Robert S. Todd, "Exclusion Keycard", Advancing in Bridge
  #584 (an unusual jump at the five level; 1430 or 0-1-1-2-2 answers).
  Convention-card `bidding_conventions/exclusion_blackwood.toml` (Kantar,
  *25 More Bridge Conventions You Should Know*, ch. 10).
- **Where we differ:** we leave the four-level jump out (it is a
  splinter in our system), and use only five-level jumps.
- **BBA:** its corpus notes ("Exclusion, for !S", "1 out of 4") give the
  `bba` treatment.
