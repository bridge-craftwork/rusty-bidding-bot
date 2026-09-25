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
    /// Partner of the player who set it may not pass at their next turn.
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
            knowledge: Default::default(),
            sides: Default::default(),
        }
    }

    pub fn caller(&self, i: usize) -> Direction {
        (0..i).fold(self.dealer, |d, _| d.next())
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

    /// Our side's last bid is below game, so a game force still applies.
    pub fn below_game(&self, d: Direction) -> bool {
        match self.auction().last_bid() {
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
}
