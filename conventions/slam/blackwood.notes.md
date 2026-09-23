# blackwood (`blackwood.bid`): notes

Standard Blackwood (card `slam.blackwood.standard`, the Basic-Bridge card):
4NT asks for aces once a trump suit is agreed; 5♣ 0 or 4, 5♦ 1, 5♥ 2, 5♠ 3.
The asker bids six with at most one ace missing, else signs off in five
(passing an answer in the trump suit). Cases: `blackwood.test`.

Added because BBA asks with 4NT on the Basic card and our side used to pass
the ask. Aces are `keycards(N)`: keycards with no trump king, four in the
deck, so "0 or 4" resolves against the asker's own aces.

## When to ask

The two-branch test in the `# The ask` block is shared with
`rkcb-1430.bid` and is written up once in **`slam-catch.notes.md`**, with
the BBA evidence behind the boundaries. It replaced `when slam_try`, the
31-HCP placeholder, which never fired over a raise: BBA asks there with
24-30 combined *support* points, and the hands are 6-5 and 6-4 shapes
whose HCP total never reaches 31.

## Gaps

- The 5NT king ask (card `slam.king_ask.five_nt`) and grand slams.
- **The queen ask.** After 4NT-5C ("0 or 4") BBA asks for the trump queen
  with the next step (5D, alerted "spades queen ask"), and the answer
  names a king as well ("!S queen and !C king"); it then places the
  contract. We bid six directly on the ace count. 55 boards, -30 IMPs
  against par. `slam.blackwood.queen_ask` exists in the field registry but
  no `.bbsa` key is mapped to it, so it cannot be switched on from a
  corpus card yet.
