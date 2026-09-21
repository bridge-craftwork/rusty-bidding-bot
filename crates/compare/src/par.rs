//! Double-dummy results, par, and scores, for judging contracts that differ.
//! Tables are cached on disk by deal, since solving is the slow part.

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use bridge_types::{
    Contract, DdTable, Deal, Direction, Doubled, FinalContract, Vulnerability, DECLARERS, STRAINS,
};
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
    let tricks = dd.tricks(c.declarer, c.strain) as i32;
    let doubled = if c.redoubled {
        Doubled::Redoubled
    } else if c.doubled {
        Doubled::Doubled
    } else {
        Doubled::None
    };
    let score = Contract::new(c.level, c.strain, doubled, c.declarer.to_char())
        .score(tricks - (c.level as i32 + 6), vul.is_vulnerable(c.declarer));
    match c.declarer {
        Direction::North | Direction::South => score,
        Direction::East | Direction::West => -score,
    }
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
}
