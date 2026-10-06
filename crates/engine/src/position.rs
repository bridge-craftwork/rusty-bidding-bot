//! The state of an auction at one point: the calls so far, what is known
//! about each hand, and each side's auction state.

use bridge_types::{Call, Direction, ScoringMethod, Strain, Vulnerability};
use serde::Serialize;

use crate::knowledge::SeatKnowledge;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Forcing {
    #[default]
    None,
    /// Partner of the player who set it may not pass at their next turn,
    /// unless the opponent in between (partner's RHO) bids, doubles or
    /// redoubles: then partner has a turn again and may pass.
    Round,
    /// Neither partner may pass below game.
    Game,
}

/// A question one player has asked partner, e.g. `keycards(S)`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ask {
    pub kind: String,
    pub args: Vec<Strain>,
    /// Who asked.
    pub by: Direction,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SideState {
    pub trump: Option<Strain>,
    pub forcing: Forcing,
    pub forcing_by: Option<Direction>,
    /// A question awaiting partner's answer.
    pub ask: Option<Ask>,
    /// A question partner has answered; cleared when the asker calls again.
    pub answered: Option<Ask>,
    /// Control-bid dialogue, by `Direction::to_index`, as suit bitmasks
    /// (bit 0 clubs .. bit 3 spades): the suits each player has denied by
    /// skipping them, and those they have shown a control in. Recorded by
    /// `sets ladder=control`. Control bids start only once a suit is
    /// agreed, so one auction holds one dialogue.
    pub denied: [u8; 4],
    pub cued: [u8; 4],
    /// Where our notrump system came on: the index in the auction of
    /// the natural notrump made where a `systems on` declaration held.
    /// Patterns reach it with `systems 1N` / `systems 2N`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub systems: Option<usize>,
}

impl SideState {
    /// An opponent has bid, doubled or redoubled. A round force obliges
    /// partner to bid only when RHO passes: once the forcer's LHO acts,
    /// partner has room to pass and a pass says he has nothing to add.
    /// A game force stands whatever the opponents do.
    pub fn opponent_acted(&mut self, caller: Direction) {
        if self.forcing == Forcing::Round && self.forcing_by.is_some_and(|f| f.next() == caller) {
            self.forcing = Forcing::None;
            self.forcing_by = None;
        }
    }
}

/// 0 for North-South, 1 for East-West.
pub fn side(d: Direction) -> usize {
    match d {
        Direction::North | Direction::South => 0,
        Direction::East | Direction::West => 1,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Position {
    pub dealer: Direction,
    pub vul: Vulnerability,
    /// How the board is scored. Rules see it as `imps` / `matchpoints`.
    pub scoring: ScoringMethod,
    pub calls: Vec<Call>,
    /// Beside `calls`: whether the rule that read each call marked it
    /// `artificial` (missing entries count as natural).
    pub artificial: Vec<bool>,
    /// By `Direction::to_index`.
    pub knowledge: [SeatKnowledge; 4],
    /// By `side`.
    pub sides: [SideState; 2],
}

impl Position {
    pub fn new(dealer: Direction, vul: Vulnerability, scoring: ScoringMethod) -> Position {
        Position {
            dealer,
            vul,
            scoring,
            calls: Vec::new(),
            artificial: Vec::new(),
            knowledge: Default::default(),
            sides: Default::default(),
        }
    }

    pub fn caller(&self, i: usize) -> Direction {
        // `i` steps clockwise from the dealer (`Direction::next` goes N E S
        // W, the index order): arithmetic, as scans over the calls ask for
        // every call's seat.
        Direction::ALL[(self.dealer.to_index() + i) % 4]
    }

    pub fn next_caller(&self) -> Direction {
        self.caller(self.calls.len())
    }

    pub fn knowledge(&self, d: Direction) -> &SeatKnowledge {
        &self.knowledge[d.to_index()]
    }

    pub fn side_state(&self, d: Direction) -> &SideState {
        &self.sides[side(d)]
    }

    /// The auction so far as a `bridge_types::Auction`, for legality checks.
    pub fn auction(&self) -> bridge_types::Auction {
        let mut a = bridge_types::Auction::new(self.dealer);
        for c in &self.calls {
            a.add_call(c.clone());
        }
        a
    }

    /// A seat's most recent call.
    pub fn last_call_of(&self, d: Direction) -> Option<&Call> {
        (0..self.calls.len())
            .rev()
            .find(|&i| self.caller(i) == d)
            .map(|i| &self.calls[i])
    }

    /// How many bids `d` has made (not passes, doubles or redoubles).
    pub fn bids_of(&self, d: Direction) -> i32 {
        (0..self.calls.len())
            .filter(|&i| self.caller(i) == d && self.calls[i].is_bid())
            .count() as i32
    }

    /// Has `d` made a bid (not only passes, doubles or redoubles)?
    pub fn has_bid(&self, d: Direction) -> bool {
        (0..self.calls.len()).any(|i| self.caller(i) == d && self.calls[i].is_bid())
    }

    /// No bid yet (only passes so far).
    pub fn is_opening(&self) -> bool {
        self.calls.iter().all(|c| c.is_pass())
    }

    /// The seat that made the first bid.
    pub fn opener(&self) -> Option<Direction> {
        self.calls
            .iter()
            .position(|c| c.is_bid())
            .map(|i| self.caller(i))
    }

    /// Has `d` called before, and only passed?
    pub fn passed_hand(&self, d: Direction) -> bool {
        let mine: Vec<&Call> = (0..self.calls.len())
            .filter(|&i| self.caller(i) == d)
            .map(|i| &self.calls[i])
            .collect();
        !mine.is_empty() && mine.iter().all(|c| c.is_pass())
    }

    /// Has the side of `d` made a bid, double or redouble?
    /// Has `d` made a natural bid in `strain`, at any point in the auction?
    /// Calls a rule marked artificial (a transfer, a relay) do not count.
    pub fn named(&self, d: Direction, strain: Strain) -> bool {
        (0..self.calls.len()).any(|i| {
            self.caller(i) == d
                && matches!(self.calls[i], Call::Bid { strain: s, .. } if s == strain)
                && !self.artificial.get(i).copied().unwrap_or(false)
        })
    }

    pub fn side_acted(&self, d: Direction) -> bool {
        (0..self.calls.len()).any(|i| side(self.caller(i)) == side(d) && !self.calls[i].is_pass())
    }

    /// Scored by total points, where a game bonus is worth stretching for:
    /// IMPs and the like. Matchpoints and board-a-match are not.
    pub fn is_imps(&self) -> bool {
        !matches!(
            self.scoring,
            ScoringMethod::Matchpoints | ScoringMethod::BAM
        )
    }

    pub fn is_vulnerable(&self, d: Direction) -> bool {
        self.vul.is_vulnerable(d)
    }

    /// Would `call`, made next, skip a bid in `suit` that was available
    /// after the last bid? (The rule behind `bypassed`, for a call not yet
    /// in the auction.)
    pub fn would_skip(&self, call: &Call, suit: usize) -> bool {
        let rank = |c: &Call| match c {
            Call::Bid { level, strain } => Some(*level as i32 * 5 + strain_rank(*strain)),
            _ => None,
        };
        let Some(mine) = rank(call) else {
            return false;
        };
        if mine % 5 == suit as i32 {
            return false;
        }
        let before = self.calls.iter().rev().find_map(rank).unwrap_or(0);
        (1..=7)
            .find(|l| l * 5 + suit as i32 > before)
            .is_some_and(|l| l * 5 + (suit as i32) < mine)
    }

    /// The cheapest legal bid in `suit` now, as level * 5 + strain rank.
    pub fn cheapest_rank(&self, suit: usize) -> Option<i32> {
        let before = self
            .calls
            .iter()
            .rev()
            .find_map(|c| match c {
                Call::Bid { level, strain } => Some(*level as i32 * 5 + strain_rank(*strain)),
                _ => None,
            })
            .unwrap_or(0);
        (1..=7).map(|l| l * 5 + suit as i32).find(|&r| r > before)
    }

    /// Did `d`'s last call bypass a bid in `suit` (0 clubs .. 3 spades)?
    /// True when that call was a bid in another strain and a bid in
    /// `suit` was available between the previous bid (anyone's) and it:
    /// the bridge meaning of skipping a suit on a ladder (control bids,
    /// stoppers, up the line). A suit below the previous bid's level was
    /// never available at that level, so it is not bypassed there.
    pub fn bypassed(&self, d: Direction, suit: usize) -> bool {
        self.bypassed_above(d, suit, None)
    }

    /// As `bypassed`, but the ladder starts above `floor` too: a bid in
    /// `suit` at or below `floor` was never on it (a control-bid ladder
    /// begins above three of the agreed suit).
    pub fn bypassed_above(&self, d: Direction, suit: usize, floor: Option<&Call>) -> bool {
        let rank = |c: &Call| match c {
            Call::Bid { level, strain } => Some(*level as i32 * 5 + strain_rank(*strain)),
            _ => None,
        };
        let Some(i) = (0..self.calls.len()).rev().find(|&i| self.caller(i) == d) else {
            return false;
        };
        let Some(mine) = rank(&self.calls[i]) else {
            return false;
        };
        if mine % 5 == suit as i32 {
            return false;
        }
        let before = self.calls[..i].iter().rev().find_map(rank).unwrap_or(0);
        let before = before.max(floor.and_then(rank).unwrap_or(0));
        // The cheapest bid in `suit` above the previous bid.
        let level = (1..=7).find(|l| l * 5 + suit as i32 > before);
        level.is_some_and(|l| l * 5 + (suit as i32) < mine)
    }

    /// `jumped`: this seat's last call was a bid one or more levels higher
    /// than the cheapest bid in the same strain after the previous bid
    /// (anyone's). 1D (1H) X (P) 2S is a jump; 1D (1S) X (P) 2H is not.
    pub fn jumped(&self, d: Direction) -> bool {
        let rank = |c: &Call| match c {
            Call::Bid { level, strain } => Some(*level as i32 * 5 + strain_rank(*strain)),
            _ => None,
        };
        let Some(i) = (0..self.calls.len()).rev().find(|&i| self.caller(i) == d) else {
            return false;
        };
        let Some(mine) = rank(&self.calls[i]) else {
            return false;
        };
        let before = self.calls[..i].iter().rev().find_map(rank).unwrap_or(0);
        mine - 5 > before
    }

    /// May `d` not pass here? Partner's last call was forcing for a round
    /// (and RHO passed over it), or the side is in a game force below game.
    pub fn must_bid(&self, d: Direction) -> bool {
        let st = self.side_state(d);
        (st.forcing == Forcing::Round && st.forcing_by == Some(d.partner()))
            || (st.forcing == Forcing::Game && self.below_game(d))
    }

    /// The deck has 40 HCP: no seat holds more than 40 minus what the other
    /// three have shown at least. Added as a constraint, so earlier
    /// disjunctions are narrowed again: a takeout double's "17+ HCP, any
    /// shape" branch drops out once the doubler cannot hold 17, leaving
    /// the shape. Repeated until nothing moves (a collapse can raise a
    /// minimum, which lowers the others' maxima).
    ///
    /// The same for each suit's 13 cards: a 1♥ opener facing a raise to 2♥
    /// holds at most ten hearts, which bounds his length points, and so
    /// what his total points say about his high cards.
    pub fn apply_deck_hcp(&mut self) {
        for _ in 0..4 {
            let mut moved = false;
            for s in 0..4 {
                let others: i32 = (0..4)
                    .filter(|&t| t != s)
                    .map(|t| self.knowledge[t].hcp.lo)
                    .sum();
                let bound = 40 - others;
                if bound < self.knowledge[s].hcp.hi {
                    let before = self.knowledge[s].hcp;
                    self.knowledge[s].add(crate::eval::hcp_at_most(bound));
                    moved |= self.knowledge[s].hcp != before;
                }
                for suit in 0..4 {
                    let others: i32 = (0..4)
                        .filter(|&t| t != s)
                        .map(|t| self.knowledge[t].len[suit].lo)
                        .sum();
                    let bound = 13 - others;
                    if bound < self.knowledge[s].len[suit].hi {
                        let before = self.knowledge[s].len[suit];
                        self.knowledge[s].narrow_by_deck(&crate::eval::length_at_most(suit, bound));
                        moved |= self.knowledge[s].len[suit] != before;
                    }
                }
            }
            if !moved {
                break;
            }
        }
    }

    /// Our side's last bid is below game, so a game force still applies.
    pub fn below_game(&self, d: Direction) -> bool {
        // The last bid and who made it, read off the calls: building an
        // `Auction` for it cost a seventh of a comparison run (`we.` and
        // `they.game_reached` are read for every sample hand).
        let last = (0..self.calls.len())
            .rev()
            .find_map(|i| match self.calls[i] {
                Call::Bid { level, strain } => Some((level, strain, self.caller(i))),
                _ => None,
            });
        match last {
            Some((level, strain, by)) if side(by) == side(d) => match strain {
                Strain::NoTrump => level < 3,
                Strain::Hearts | Strain::Spades => level < 4,
                Strain::Clubs | Strain::Diamonds => level < 5,
            },
            // The opponents hold the contract: we are not yet at our game.
            _ => true,
        }
    }
}

/// Clubs 0, diamonds 1, hearts 2, spades 3, notrump 4: bidding order.
fn strain_rank(s: Strain) -> i32 {
    match s {
        Strain::Clubs => 0,
        Strain::Diamonds => 1,
        Strain::Hearts => 2,
        Strain::Spades => 3,
        Strain::NoTrump => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_steps_clockwise_from_the_dealer() {
        for dealer in Direction::ALL {
            let pos = Position::new(
                dealer,
                Vulnerability::None,
                ScoringMethod::from_pbn("IMP").unwrap(),
            );
            let mut d = dealer;
            for i in 0..12 {
                assert_eq!(pos.caller(i), d);
                d = d.next();
            }
        }
    }

    #[test]
    fn named_counts_natural_bids_only() {
        let mut pos = Position::new(
            Direction::North,
            Vulnerability::None,
            ScoringMethod::from_pbn("IMP").unwrap(),
        );
        // N 1NT, E P, S 2D (a transfer: artificial), W P, N 2H.
        let bid = |level, strain| Call::Bid { level, strain };
        for (c, art) in [
            (bid(1, Strain::NoTrump), false),
            (Call::Pass, false),
            (bid(2, Strain::Diamonds), true),
            (Call::Pass, false),
            (bid(2, Strain::Hearts), false),
        ] {
            pos.calls.push(c);
            pos.artificial.push(art);
        }
        assert!(pos.named(Direction::North, Strain::NoTrump));
        assert!(pos.named(Direction::North, Strain::Hearts));
        assert!(!pos.named(Direction::South, Strain::Diamonds));
        assert!(!pos.named(Direction::South, Strain::NoTrump));
    }

    fn pos(calls: &str) -> Position {
        let mut p = Position::new(
            Direction::South,
            Vulnerability::None,
            ScoringMethod::Matchpoints,
        );
        for c in calls.split_whitespace() {
            p.calls.push(Call::from_pbn(c).unwrap());
        }
        p
    }

    #[test]
    fn bypassed_counts_only_the_suits_that_were_available() {
        // 1NT P 2D P 2H P 3C P 3H P 3S: North's 3S over 3H skips nothing.
        let p = pos("1NT Pass 2D Pass 2H Pass 3C Pass 3H Pass 3S");
        let n = Direction::North;
        assert!(!p.bypassed(n, 0) && !p.bypassed(n, 1) && !p.bypassed(n, 2));
        // 4D over 3H skips 3S and 4C, not hearts.
        let p = pos("1NT Pass 2D Pass 2H Pass 3C Pass 3H Pass 4D");
        assert!(p.bypassed(n, 3) && p.bypassed(n, 0));
        assert!(!p.bypassed(n, 2) && !p.bypassed(n, 1));
        // 4S over 4D skips 4H only.
        let p = pos("1H Pass 2NT Pass 4D Pass 4S");
        assert!(p.bypassed(n, 2) && !p.bypassed(n, 0) && !p.bypassed(n, 1));
        // 1S 2NT 3C 4C: with the ladder starting above 3S, 4C skips nothing.
        let p = pos("1S Pass 2NT Pass 3C Pass 4C");
        let floor = Call::from_pbn("3S");
        assert!(p.bypassed(n, 1) && !p.bypassed_above(n, 1, floor.as_ref()));
        assert!(!p.bypassed_above(n, 2, floor.as_ref()));
    }

    #[test]
    fn jumped_means_a_cheaper_bid_in_the_same_strain_was_available() {
        let s = Direction::South;
        assert!(pos("1D 1H X Pass 2S").jumped(s));
        assert!(!pos("1D 1S X Pass 2H").jumped(s));
        assert!(pos("1D 2C X Pass 3H").jumped(s));
        assert!(!pos("1D 2C X Pass 2H").jumped(s));
        assert!(!pos("1D 1H X Pass Pass").jumped(Direction::West));
        assert!(!pos("1D 1H X Pass 1S").jumped(s));
    }

    /// South deals; North's last call set `f`.
    fn forced_by_north(calls: &str, f: Forcing) -> Position {
        let mut p = pos(calls);
        p.sides[0].forcing = f;
        p.sides[0].forcing_by = Some(Direction::North);
        p
    }

    #[test]
    fn a_round_force_holds_when_rho_passes() {
        let p = forced_by_north("1D 1H 2H Pass", Forcing::Round);
        assert!(p.must_bid(Direction::South));
        assert!(!p.must_bid(Direction::North));
    }

    #[test]
    fn a_round_force_ends_when_rho_bids_doubles_or_redoubles() {
        for rho in ["3H", "X", "XX"] {
            let mut p = forced_by_north("1D 1H 2H", Forcing::Round);
            p.calls.push(Call::from_pbn(rho).unwrap());
            p.sides[0].opponent_acted(Direction::East);
            assert!(!p.must_bid(Direction::South), "after {rho}");
            assert_eq!(p.sides[0].forcing, Forcing::None);
        }
    }

    #[test]
    fn only_the_forcers_lho_releases_a_round_force() {
        // West calls after South: the force North set is not his to end.
        let mut p = forced_by_north("1D 1H 2H", Forcing::Round);
        p.sides[0].opponent_acted(Direction::West);
        assert_eq!(p.sides[0].forcing, Forcing::Round);
    }

    #[test]
    fn a_game_force_holds_when_rho_bids() {
        let mut p = forced_by_north("1D 1H 2H 3H", Forcing::Game);
        p.sides[0].opponent_acted(Direction::East);
        assert!(p.must_bid(Direction::South));
        // At game the force is satisfied.
        let p = forced_by_north("1D 1H 2H 3H 5D Pass", Forcing::Game);
        assert!(!p.must_bid(Direction::North));
    }
}
