//! Double-dummy results, par, and scores, for judging contracts that differ.
//! Tables are cached on disk by deal, since solving is the slow part.

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use bridge_types::{
    Contract, DdTable, Deal, Direction, Doubled, FinalContract, Vulnerability, DECLARERS, STRAINS,
};
#[cfg(test)]
use bridge_types::Strain;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CachedTable {
    deal: String,
    /// Tricks by declarer (N E S W) then strain (C D H S NT).
    tricks: Vec<u8>,
}

pub struct DdCache {
    path: PathBuf,
    tables: Mutex<HashMap<String, DdTable>>,
}

impl DdCache {
    /// Open (or start) the cache file at `path`: one JSON table per line.
    pub fn open(path: PathBuf) -> DdCache {
        let mut tables = HashMap::new();
        if let Ok(text) = std::fs::read_to_string(&path) {
            for line in text.lines() {
                if let Ok(c) = serde_json::from_str::<CachedTable>(line) {
                    if c.tricks.len() == 20 {
                        let mut table = DdTable::new();
                        for (i, d) in DECLARERS.iter().enumerate() {
                            for (j, s) in STRAINS.iter().enumerate() {
                                table.set(*d, *s, c.tricks[i * 5 + j]);
                            }
                        }
                        tables.insert(c.deal, table);
                    }
                }
            }
        }
        DdCache {
            path,
            tables: Mutex::new(tables),
        }
    }

    pub fn len(&self) -> usize {
        self.tables.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The table for `deal`, solving and caching it if new.
    pub fn table(&self, deal: &Deal) -> DdTable {
        let key = deal.to_pbn(Direction::North);
        if let Some(t) = self.tables.lock().unwrap().get(&key) {
            return *t;
        }
        let table = bridge_solver::par::solve_dd_table(deal);
        let tricks: Vec<u8> = DECLARERS
            .iter()
            .flat_map(|d| STRAINS.iter().map(move |s| table.tricks(*d, *s)))
            .collect();
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let line = serde_json::to_string(&CachedTable {
                deal: key.clone(),
                tricks,
            })
            .unwrap_or_default();
            let _ = writeln!(f, "{line}");
        }
        self.tables.lock().unwrap().insert(key, table);
        table
    }
}

/// North-South's score for `contract` played double dummy (0 if passed out).
pub fn score_ns(contract: Option<&FinalContract>, dd: &DdTable, vul: Vulnerability) -> i32 {
    let Some(c) = contract else { return 0 };
    score_ns_as(c, doubling_of(c), dd, vul)
}

fn doubling_of(c: &FinalContract) -> Doubled {
    if c.redoubled {
        Doubled::Redoubled
    } else if c.doubled {
        Doubled::Doubled
    } else {
        Doubled::None
    }
}

/// North-South's score for `c` played double dummy, doubled as `doubled`.
fn score_ns_as(c: &FinalContract, doubled: Doubled, dd: &DdTable, vul: Vulnerability) -> i32 {
    let tricks = dd.tricks(c.declarer, c.strain) as i32;
    let score = Contract::new(c.level, c.strain, doubled, c.declarer.to_char())
        .score(tricks - (c.level as i32 + 6), vul.is_vulnerable(c.declarer));
    match c.declarer {
        Direction::North | Direction::South => score,
        Direction::East | Direction::West => -score,
    }
}

/// One table's errors against par, in IMPs, charged to the side that made
/// them: index 0 North-South, 1 East-West (Rick, 2026-10-01).
///
/// Distance from par is one absolute number per table, so two errors can
/// cancel, and a penalty double of an overbid moves the result past par and
/// counts as a loss. Instead, as in double-dummy par, a contract that goes
/// down undoubled and would beat par that way is taken as doubled (the
/// assumed result). The **contract error** is the IMPs between the assumed
/// result and par, charged to the side it leaves worse off than par: the
/// overbidder, or the side that should have bid on or sacrificed. The
/// **doubling error** is the IMPs between the actual and the assumed result,
/// charged to the side the actual result leaves worse off: the defenders
/// for not doubling, or for doubling a contract that makes; declarer for
/// redoubling one that fails.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableErrors {
    pub contract: [i32; 2],
    pub doubling: [i32; 2],
}

impl TableErrors {
    pub fn total(&self) -> i32 {
        self.contract.iter().chain(&self.doubling).sum()
    }
    pub fn contract_total(&self) -> i32 {
        self.contract.iter().sum()
    }
    pub fn doubling_total(&self) -> i32 {
        self.doubling.iter().sum()
    }
}

/// `TableErrors` for `contract` (None: passed out) against par `par_ns`.
pub fn table_errors(
    contract: Option<&FinalContract>,
    dd: &DdTable,
    vul: Vulnerability,
    par_ns: i32,
) -> TableErrors {
    let actual = score_ns(contract, dd, vul);
    let assumed = match contract {
        None => actual,
        Some(c) => {
            let made = dd.tricks(c.declarer, c.strain) as i32 >= c.level as i32 + 6;
            let ns = matches!(c.declarer, Direction::North | Direction::South);
            // Declarer's side's view of a North-South score.
            let mine = |x: i32| if ns { x } else { -x };
            match (doubling_of(c), made) {
                // A double of a making contract was a mistake: undoubled.
                (Doubled::Doubled | Doubled::Redoubled, true) => {
                    score_ns_as(c, Doubled::None, dd, vul)
                }
                // A redouble of a failing one: doubled.
                (Doubled::Redoubled, false) => score_ns_as(c, Doubled::Doubled, dd, vul),
                // Down undoubled, and better than par that way: doubled.
                (Doubled::None, false) if mine(actual) > mine(par_ns) => {
                    score_ns_as(c, Doubled::Doubled, dd, vul)
                }
                _ => actual,
            }
        }
    };
    let mut e = TableErrors::default();
    // The side a result leaves worse off than its benchmark made the error.
    let charge = |slot: &mut [i32; 2], result: i32, benchmark: i32| {
        let side = if result < benchmark { 0 } else { 1 };
        slot[side] += imps((result - benchmark).abs());
    };
    charge(&mut e.contract, assumed, par_ns);
    charge(&mut e.doubling, actual, assumed);
    e
}

/// Par for the deal, from North-South's side, and its contract(s) described.
pub fn par_ns(dd: &DdTable, vul: Vulnerability) -> (i32, String) {
    let p = bridge_solver::par::par(
        dd,
        vul.is_vulnerable(Direction::North),
        vul.is_vulnerable(Direction::East),
    );
    let text = p
        .contracts
        .iter()
        .map(|c| c.describe())
        .collect::<Vec<_>>()
        .join("; ");
    (p.score_ns, text)
}

/// The standard IMP scale.
pub fn imps(diff: i32) -> i32 {
    const STEPS: [i32; 24] = [
        20, 50, 90, 130, 170, 220, 270, 320, 370, 430, 500, 600, 750, 900, 1100, 1300, 1500, 1750,
        2000, 2250, 2500, 3000, 3500, 4000,
    ];
    let n = STEPS.iter().take_while(|&&s| diff.abs() >= s).count() as i32;
    n * diff.signum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imp_scale() {
        assert_eq!(imps(0), 0);
        assert_eq!(imps(10), 0);
        assert_eq!(imps(20), 1);
        assert_eq!(imps(-420), -9);
        assert_eq!(imps(620), 12);
        assert_eq!(imps(5000), 24);
    }

    fn contract(level: u8, strain: Strain, declarer: Direction, doubled: bool) -> FinalContract {
        let mut c = FinalContract::new(level, strain, declarer);
        c.doubled = doubled;
        c
    }

    #[test]
    fn an_overbid_is_charged_as_doubled_and_the_defenders_for_not_doubling() {
        // Par: North-South's 4♠ +420. East-West bid 5♥ and take seven tricks.
        let mut dd = DdTable::new();
        dd.set(Direction::East, Strain::Hearts, 7);
        let five_h = contract(5, Strain::Hearts, Direction::East, false);
        let e = table_errors(Some(&five_h), &dd, Vulnerability::None, 420);
        // Assumed doubled, -800 for them: 380 better than par for North-South,
        // East-West's error (9 IMPs). Undoubled, +200: North-South's (12).
        assert_eq!(e.contract, [0, 9]);
        assert_eq!(e.doubling, [12, 0]);
        let doubled = contract(5, Strain::Hearts, Direction::East, true);
        let e = table_errors(Some(&doubled), &dd, Vulnerability::None, 420);
        assert_eq!(e.contract, [0, 9]);
        assert_eq!(e.doubling, [0, 0]);
    }

    #[test]
    fn a_double_of_a_making_contract_is_the_defenders_error() {
        // North makes 2♠ doubled; par is 2♠ +110.
        let mut dd = DdTable::new();
        dd.set(Direction::North, Strain::Spades, 8);
        let c = contract(2, Strain::Spades, Direction::North, true);
        let e = table_errors(Some(&c), &dd, Vulnerability::None, 110);
        assert_eq!(e.contract, [0, 0]);
        // +470 against +110: East-West's double cost them 8 IMPs.
        assert_eq!(e.doubling, [0, 8]);
    }

    #[test]
    fn going_down_below_par_is_not_assumed_doubled() {
        // Par: North-South 4♠ +420; they stop in 4♠ by North and go one down.
        let mut dd = DdTable::new();
        dd.set(Direction::North, Strain::Spades, 9);
        let c = contract(4, Strain::Spades, Direction::North, false);
        let e = table_errors(Some(&c), &dd, Vulnerability::None, 420);
        assert_eq!(e.contract, [10, 0]);
        assert_eq!(e.doubling, [0, 0]);
    }
}
