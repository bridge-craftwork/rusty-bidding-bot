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
