# jump-raises (`jump-raises.bid`): notes

1M-3M when it is not the limit raise: weak (preemptive) or mixed.
Cases: `jump-raises.test`.

## What the card says

| Field | `.bbsa` key | Meaning here |
|---|---|---|
| `major_openings.jump_raise.weak` | `1M-3M blocking` | 3M four trumps, about 3-6 |
| `major_openings.jump_raise.mixed` | `Mixed raise` | 3M four trumps, 7-10 support points |
| `major_openings.jump_raise.inv` | `1M-3M inviting` | the limit raise (responses.bid) |

The three are one choice on the card, but BBA's cards set several:
21GF-GIB has `Mixed raise` and `1M-3M inviting`, 21GF-GIB-Bergen `Mixed
raise` and `1M-3M blocking`. Decision (2026-10-05): the invitational
raise wins over both, Bergen wins over everything (bergen.bid), weak wins
over mixed. BBA on 21GF-GIB plays 1M-3M inviting with `Mixed raise` set
(`probes/mixed-resp-1H-GIB.toml`: "1M-3M inviting", 10-12), so the
mixed raise never fires on a PBS card; it is there for cards that choose
it alone.

- Opener over the weak raise: game from 19 support points (shared with
  Bergen's 3M); over the mixed raise game from 16.
- Uncontested only. Gavin_Mixed_Raise plays 21GF-DEFAULT (no mixed raise),
  so its 3M are BBA's limit raises with four trumps and 9-10 HCP.

## Open questions

- What does BBA's `Mixed raise` switch? Not 1M-3M (probe above). Perhaps
  the jump raise after an overcall (Mixed_Raise_In_Comp plays 21GF-SPECIALS
  "to include Mixed Raise"); if so, its mapping to
  `major_openings.jump_raise.mixed` is wrong and it belongs with
  `jump_raise_after_overcall.mixed` (competitive, another module).

## Sources

- Bridge Bum, "Weak Jump Raise" and "Mixed Raise" (4 trumps, about 7-10);
  the PBS scenario Gavin_Mixed_Raise ("Mixed 4-card Raise (8-10 TP) 3M").
- Opener's thresholds: standard practice, not yet cited.
