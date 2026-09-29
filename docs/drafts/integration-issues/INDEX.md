# Integration issues: filing order

Drafts of the issues in
[docs/integration-bridge-classroom.md](../../integration-bridge-classroom.md)
("Proposed issues"), revised 2026-09-28 to describe only the remaining
work. Each file starts with `Title: <title>`, a blank line, then the body.
Cross-references are written as `R1 (rusty-bidding-bot)` and the like;
replace them with issue links once the issues are filed, filing in this
order so each dependency exists first.

| # | ID | Repo | File | Title | Depends on |
|---|---|---|---|---|---|
| 1 | R1 | rusty-bidding-bot | [R1-rusty-bidding-bot.md](R1-rusty-bidding-bot.md) | Bid a practice table's bot seats: `auction` entry point, meanings for any call, determinism test (in progress) | none |
| 2 | R4 | rusty-bidding-bot | [R4-rusty-bidding-bot.md](R4-rusty-bidding-bot.md) | Release package for Bridge-Classroom: size and speed budgets, checksums, coverage manifest | R1 |
| 3 | R5 | rusty-bidding-bot | [R5-rusty-bidding-bot.md](R5-rusty-bidding-bot.md) | Native embedding for bridge-table-service: one constructor from card specs, JSON output, version, pin policy | R1 |
| 4 | C1 | Bridge-Classroom | [C1-Bridge-Classroom.md](C1-Bridge-Classroom.md) | Rusty bidder in a Web Worker, pulled from rusty-bidding-bot's release, with the BBA fallback | R1 (and a release carrying it); R4 checksums when available |
| 5 | C2 | Bridge-Classroom | [C2-Bridge-Classroom.md](C2-Bridge-Classroom.md) | Bidding engine setting (BBA or Rusty), remembered; labels name the bidder and the reference | C1 |
| 6 | C3 | Bridge-Classroom | [C3-Bridge-Classroom.md](C3-Bridge-Classroom.md) | Mouseover meanings from Rusty for every call in the auction | C1, R1 |
| 7 | C4 | Bridge-Classroom | [C4-Bridge-Classroom.md](C4-Bridge-Classroom.md) | The "why" panel: BBA's reference call and Rusty's reading of the student's call | C3, R1 |
| 8 | C5 | Bridge-Classroom | [C5-Bridge-Classroom.md](C5-Bridge-Classroom.md) | Play my convention card, both ways, with coverage warnings and bidding reports | C1, R1 |
| 9 | T1 | bridge-table-service | [T1-bridge-table-service.md](T1-bridge-table-service.md) | Native Rusty bidder with the BBA fallback | R5 |
| 10 | T2 | bridge-table-service | [T2-bridge-table-service.md](T2-bridge-table-service.md) | Convention cards per session | R5, T1 |
| 11 | C6 | Bridge-Classroom | [C6-Bridge-Classroom.md](C6-Bridge-Classroom.md) | Served ("distributed") table: bidder choice, labels and meanings | T1, T2, C3 |

Not filed (see the plan, "Done, merged or dropped"):

- **R2** (built-in named cards): done; its one leftover (a freshness check
  of the stock cards against Practice-Bidding-Scenarios) is in R4.
- **R3** (display contract): done on the engine side; the no-rule signal
  and meanings for any call are in R1, the field mapping is in C1, the
  suit notation is settled in the plan (§3.4) and rendered in C3.
