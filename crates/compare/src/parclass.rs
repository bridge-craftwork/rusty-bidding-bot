//! How a table met par, and its penalty doubles (Rick, 2026-10-01).
//!
//! `par::table_errors` prices a table's errors; this says what kind they
//! were. The class follows the contract error: whose it is, and how the
//! contract stands against par (short of game or slam, too high, theirs).
//! The doubling flags count the chances for a good penalty double (a
//! contract bid above par that goes down), how many were taken, the doubles
//! of contracts that made, and the side that bid on over the other side's
//! failing overbid instead of doubling it ("bailed out").

use bridge_types::{Call, DdTable, Direction, FinalContract, Strain, Vulnerability};
use serde::{Deserialize, Serialize};

use crate::par::{rank, TableErrors};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParClass {
    /// No error at the table.
    AtPar,
    /// The contract was right; the only error was a double (or its absence).
    DoublingOnly,
    /// Nobody bid; par was a contract.
    PassedOut,
    /// The side with par stopped below the slam par is.
    ShortOfSlam,
    /// The side with par stopped in a partscore; par is a game.
    ShortOfGame,
    /// The side with par stopped too low in a par strain.
    ShortLevel,
    /// The side with par played another strain or from the wrong side.
    WrongStrain,
    /// The side with par bid above par.
    Overbid,
    /// The side with par let the other side play a contract better than par
    /// for them: it did not compete, or did not bid on over a sacrifice.
    DidNotCompete,
    /// The other side bid above par: a sacrifice that cost more than their
    /// contract, or an overcompetition.
    TheirOverbid,
    /// The other side let the side with par score more than par: it did not
    /// compete or sacrifice.
    NoSacrifice,
}

impl ParClass {
    pub fn describe(self) -> &'static str {
        match self {
            ParClass::AtPar => "at par",
            ParClass::DoublingOnly => "right contract, a doubling error",
            ParClass::PassedOut => "passed out, par is a contract",
            ParClass::ShortOfSlam => "short of slam",
            ParClass::ShortOfGame => "short of game",
            ParClass::ShortLevel => "too low in a par strain",
            ParClass::WrongStrain => "wrong strain or declarer",
            ParClass::Overbid => "overbid (the side with par)",
            ParClass::DidNotCompete => "did not compete or bid on (the side with par)",
            ParClass::TheirOverbid => "overbid or sacrificed too much (the other side)",
            ParClass::NoSacrifice => "did not compete or sacrifice (the other side)",
        }
    }
}

/// One table's class and doubling flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableClass {
    pub class: ParClass,
    /// The final contract was a chance for a good penalty double (bid above
    /// par and down): `Some(true)` if it was doubled.
    pub double_chance: Option<bool>,
    /// A contract that made was doubled (or redoubled).
    pub bad_double: bool,
    /// A side bid on over the other side's failing overbid instead of
    /// doubling it.
    pub bailed_out: bool,
}

/// The tier of a contract: 0 partscore, 1 game, 2 slam.
fn tier(level: u8, strain: Strain) -> u8 {
    if level >= 6 {
        2
    } else if level >= 5
        || (level >= 4 && matches!(strain, Strain::Hearts | Strain::Spades))
        || (level >= 3 && strain == Strain::NoTrump)
    {
        1
    } else {
        0
    }
}

fn ns(d: Direction) -> bool {
    matches!(d, Direction::North | Direction::South)
}

/// Whether `c` played double dummy goes down.
fn fails(dd: &DdTable, declarer: Direction, level: u8, strain: Strain) -> bool {
    (dd.tricks(declarer, strain) as i32) < level as i32 + 6
}

/// Classify a table. `par` is the par contracts as (level, strain), empty
/// for a pass-out; `par_ns` its score; `assumed_doubled` whether
/// `table_errors` took the contract as doubled.
#[allow(clippy::too_many_arguments)]
pub fn classify(
    calls: &[Call],
    dealer: Direction,
    contract: Option<&FinalContract>,
    dd: &DdTable,
    vul: Vulnerability,
    par: &[(u8, Strain)],
    par_ns: i32,
    errors: &TableErrors,
) -> TableClass {
    let par_rank = par.iter().map(|&(l, s)| rank(l, s)).max().unwrap_or(0);
    let made = contract.is_some_and(|c| !fails(dd, c.declarer, c.level, c.strain));
    let doubled = contract.is_some_and(|c| c.doubled || c.redoubled);
    let double_chance = contract.and_then(|c| {
        if made {
            return None;
        }
        let mine = |x: i32| if ns(c.declarer) { x } else { -x };
        let undoubled = {
            let mut u = c.clone();
            u.doubled = false;
            u.redoubled = false;
            crate::par::score_ns(Some(&u), dd, vul)
        };
        (rank(c.level, c.strain) > par_rank || mine(undoubled) > mine(par_ns)).then_some(doubled)
    });
    let class = class_of(contract, par, par_ns, par_rank, errors);
    TableClass {
        class,
        double_chance,
        bad_double: made && doubled,
        bailed_out: bailed_out(calls, dealer, dd, par_rank),
    }
}

fn class_of(
    contract: Option<&FinalContract>,
    par: &[(u8, Strain)],
    par_ns: i32,
    par_rank: i32,
    errors: &TableErrors,
) -> ParClass {
    if errors.total() == 0 {
        return ParClass::AtPar;
    }
    if errors.contract_total() == 0 {
        return ParClass::DoublingOnly;
    }
    let Some(c) = contract else {
        return ParClass::PassedOut;
    };
    // The side charged with the contract error: 0 North-South, 1 East-West.
    let erred_ns = errors.contract[0] > 0;
    let declarer_ns = ns(c.declarer);
    // The side par belongs to; a pass-out par belongs to nobody, and then
    // any contract that went wrong is the declarer's overbid.
    let owner_ns = match par_ns.signum() {
        1 => Some(true),
        -1 => Some(false),
        _ => None,
    };
    let Some(owner_ns) = owner_ns else {
        return ParClass::Overbid;
    };
    match (declarer_ns == owner_ns, erred_ns == owner_ns) {
        // The side with par declared and did worse than par.
        (true, true) => {
            if rank(c.level, c.strain) > par_rank {
                return ParClass::Overbid;
            }
            let top = par.iter().map(|&(l, s)| tier(l, s)).max().unwrap_or(0);
            let mine = tier(c.level, c.strain);
            if top == 2 && mine < 2 {
                ParClass::ShortOfSlam
            } else if top >= 1 && mine == 0 {
                ParClass::ShortOfGame
            } else if par.iter().any(|&(l, s)| s == c.strain && c.level < l) {
                ParClass::ShortLevel
            } else {
                ParClass::WrongStrain
            }
        }
        // The side with par declared and did better: the others erred.
        (true, false) => ParClass::NoSacrifice,
        // The other side declared and did better than par for them.
        (false, true) => ParClass::DidNotCompete,
        // The other side declared and did worse.
        (false, false) => ParClass::TheirOverbid,
    }
}

/// Did a side bid on over the other side's failing bid above par, instead of
/// doubling it? A bid counts when it outranks every par contract and its
/// declarer (the first of that side to name the strain) goes down double
/// dummy; the other side bailed out if its next call other than a pass,
/// before that side bids again, is a bid.
fn bailed_out(calls: &[Call], dealer: Direction, dd: &DdTable, par_rank: i32) -> bool {
    let seats: Vec<Direction> = std::iter::successors(Some(dealer), |d| Some(d.next()))
        .take(calls.len())
        .collect();
    let first_to_name = |side_ns: bool, strain: Strain, before: usize| {
        (0..=before).find_map(|i| match calls[i] {
            Call::Bid { strain: s, .. } if s == strain && ns(seats[i]) == side_ns => Some(seats[i]),
            _ => None,
        })
    };
    for (i, call) in calls.iter().enumerate() {
        let Call::Bid { level, strain } = *call else {
            continue;
        };
        let side = ns(seats[i]);
        if rank(level, strain) <= par_rank {
            continue;
        }
        let Some(declarer) = first_to_name(side, strain, i) else {
            continue;
        };
        if !fails(dd, declarer, level, strain) {
            continue;
        }
        for (j, later) in calls.iter().enumerate().skip(i + 1) {
            let theirs = ns(seats[j]) != side;
            match (theirs, later) {
                (true, Call::Pass) => {}
                (true, Call::Bid { .. }) => return true,
                (true, _) => break,
                (false, Call::Bid { .. }) => break,
                (false, _) => {}
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bid(level: u8, strain: Strain) -> Call {
        Call::Bid { level, strain }
    }

    #[test]
    fn bidding_on_over_a_failing_sacrifice_is_bailing_out() {
        // N 4♠ (par), E 5♥ (seven tricks), S 5♠ instead of doubling.
        let mut dd = DdTable::new();
        dd.set(Direction::East, Strain::Hearts, 7);
        let calls = [
            bid(4, Strain::Spades),
            bid(5, Strain::Hearts),
            bid(5, Strain::Spades),
        ];
        assert!(bailed_out(
            &calls,
            Direction::North,
            &dd,
            rank(4, Strain::Spades)
        ));
        // Doubled instead: no.
        let calls = [bid(4, Strain::Spades), bid(5, Strain::Hearts), Call::Double];
        assert!(!bailed_out(
            &calls,
            Direction::North,
            &dd,
            rank(4, Strain::Spades)
        ));
        // If 5♥ made, bidding on is competing, not bailing out.
        dd.set(Direction::East, Strain::Hearts, 11);
        let calls = [
            bid(4, Strain::Spades),
            bid(5, Strain::Hearts),
            bid(5, Strain::Spades),
        ];
        assert!(!bailed_out(
            &calls,
            Direction::North,
            &dd,
            rank(4, Strain::Spades)
        ));
    }

    #[test]
    fn tiers() {
        assert_eq!(tier(3, Strain::NoTrump), 1);
        assert_eq!(tier(4, Strain::Diamonds), 0);
        assert_eq!(tier(4, Strain::Spades), 1);
        assert_eq!(tier(6, Strain::Clubs), 2);
    }
}
