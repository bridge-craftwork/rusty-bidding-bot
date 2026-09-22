# blackwood (`blackwood.bid`): notes

Standard Blackwood (card `slam.blackwood.standard`, the Basic-Bridge card):
4NT asks for aces once a trump suit is agreed; 5♣ 0 or 4, 5♦ 1, 5♥ 2, 5♠ 3.
The asker bids six with at most one ace missing, else signs off in five
(passing an answer in the trump suit). Cases: `blackwood.test`.

Added because BBA asks with 4NT on the Basic card and our side used to pass
the ask. Aces are `keycards(N)`: keycards with no trump king, four in the
deck, so "0 or 4" resolves against the asker's own aces.

## Gaps

- The 5NT king ask (card `slam.king_ask.five_nt`) and grand slams.
- `slam_try` is still the 31-HCP placeholder.
