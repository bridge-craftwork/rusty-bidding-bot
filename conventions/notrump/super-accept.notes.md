# super-accept (`super-accept.bid`): notes

Opener jumps to 3M over a Jacoby transfer with four-card support (card
field `notrump.jacoby_transfers.super_accept`). Cases: `super-accept.test`.

## Guidance

- 3M with 4+ trumps. With `superaccept-doubleton.bid` on as well, maximums
  with a doubleton take the doubleton bids (more descriptive), so 3M is
  left for minimums and 4-3-3-3 hands.

## Evidence from BBA

Jacoby_Super-Accept (500 boards): 81.7% of calls agree, 30.8% identical
auctions. See `superaccept-doubleton.notes.md`.

## Accepted differences from BBA

- After 3M, which may be a minimum, responder with invitational values now
  bids game (the known-fit rule in `jacoby-transfers.bid`). BBA passes some
  of these (8 boards; 24 others go our way).

## Gaps (not built yet)

- Responder's slam tries over 3M other than keycard.

## Convention score (2026-10-07)

Unmeasurable as BBA's convention: its key (`Super acceptance after NT`)
does not change BBA's auctions on Jacoby_Super-Accept (0 boards). Our
own off run had 13 no-rule positions (0 on): with no super-accept the
plain completion is the whole 1NT range, and six-card responders
between the strength bands had no call; jacoby-transfers.bid now passes
or invites them by HCP. Judge it against ourselves (self A/B) instead.

## Brill (general.system = brill, 2026-10-15)

Brill's treatment, played on the Brill card only (`cards/json/Brill.json`, `general.system = brill`: a stand-in until convention-card's `general.style` takes a `brill` value; cards/README.md "Brill"). Rusty plays the Brill dailies as South with three Brill robots, so it should read and bid what they do. Every other card is unchanged (a `.test` line on 21GF-GIB beside each Brill line; `compare --limit 50` tripwire against main: no board changed).

The jump to 3M is the minimum with four trumps (not 4-3-3-3, which
completes); maximums go through 2NT or the doubleton (superaccept-doubleton.bid).
Brill "4+ hearts, min": `hearts > 3 and not max`; "4+ spades, min": `spades >
3 and not max`.

Measured with `probes/tools/brill_probe.py report --card Brill` over the top 2000 corpus auctions (calls of this group, board-weighted, our reading against Brill's): see jacoby-transfers.notes.md.

## Sources

- Brill /explain, 2026-10-15 (Thorvald Aagaard's public Brill service, answers cached in `.rbb-cache/brill-service/explain/`): the rule text quoted in the `brill` section and in the `.bid` comments. Applies to the Brill card only (`general.system = brill`, cards/README.md "Brill").
- **The convention:** the Jacoby super-accept (3M with four-card support
  over a transfer). Standard practice, not yet cited to a book or
  article.
- **Rick's rulings:** none specific to this module; the doubleton
  super-accept rulings are in `superaccept-doubleton.notes.md`.
- **BBA evidence:** the Jacoby_Super-Accept corpus (500 boards).
- **Corpus measurements:** the 2026-09-25 removal screen that set the
  16-point floor (`jacoby-transfers.notes.md`, "Super-accept needs 16").
- **Where we differ:** after a 3M super-accept, responder with
  invitational values bids game where BBA sometimes passes ("Accepted
  differences from BBA").
