//! Compare the engine's auctions with a reference corpus (BBA's bidding of
//! the Practice-Bidding-Scenarios deals), board by board: replay agreement,
//! first divergence, final contract, and on differing contracts, which one
//! is closer to double-dummy par.

mod board;
pub mod par;
mod report;
mod scenario;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bridge_card::{bbsa, Card};
use rayon::prelude::*;
use rbb_engine::Engine;
use serde::Serialize;

pub use board::{BoardResult, ParComparison};
pub use report::{short, summarize, Agreement, Divergence, ParTally, Stats, Summary};
pub use scenario::{discover, Scenario};

#[derive(Debug, Clone)]
pub struct Options {
    /// A Practice-Bidding-Scenarios checkout.
    pub pbs: PathBuf,
    /// Scenario names; empty for all.
    pub scenarios: Vec<String>,
    /// At most this many boards per scenario.
    pub limit: Option<usize>,
    /// Directory of `.bid` modules.
    pub rules: PathBuf,
    /// Solve double dummy where the final contracts differ.
    pub par: bool,
    /// Where solved double-dummy tables are kept.
    pub dd_cache: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub summary: Summary,
    pub boards: Vec<BoardResult>,
}

fn load_card(pbs: &std::path::Path, name: &str) -> Result<Card, String> {
    let path = pbs.join("bbsa").join(format!("{name}.bbsa"));
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    bbsa::import(&text, Some(name))
        .map(|(c, _)| c)
        .map_err(|e| e.to_string())
}

/// Compiled rules plus one engine per pair of cards, built on first use.
pub struct Engines {
    pbs: PathBuf,
    modules: Vec<bidspec::Module>,
    cache: Mutex<HashMap<(String, String), Arc<Engine>>>,
}

impl Engines {
    /// Compile the rules in `rules`; cards are read from `pbs/bbsa`.
    pub fn new(pbs: &std::path::Path, rules: &std::path::Path) -> Result<Engines, String> {
        let modules = rbb_engine::load_modules(rules).map_err(|d| {
            d.iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        Ok(Engines {
            pbs: pbs.to_path_buf(),
            modules,
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// The engine for North-South playing `ns` and East-West `ew`.
    pub fn get(&self, ns: &str, ew: &str) -> Result<Arc<Engine>, String> {
        let key = (ns.to_string(), ew.to_string());
        if let Some(e) = self.cache.lock().unwrap().get(&key) {
            return Ok(e.clone());
        }
        let engine = Arc::new(Engine::new(
            &load_card(&self.pbs, ns)?,
            &load_card(&self.pbs, ew)?,
            &self.modules,
        ));
        self.cache.lock().unwrap().insert(key, engine.clone());
        Ok(engine)
    }
}

/// Run the comparison. `progress` is called with (boards done, total).
pub fn run(opts: &Options, progress: &(dyn Fn(usize, usize) + Sync)) -> Result<Report, String> {
    let engines = Engines::new(&opts.pbs, &opts.rules)?;
    run_with(opts, &engines, progress)
}

/// `run`, with rules already compiled (so the caller can keep the engines).
pub fn run_with(
    opts: &Options,
    engines: &Engines,
    progress: &(dyn Fn(usize, usize) + Sync),
) -> Result<Report, String> {
    let scenarios = discover(&opts.pbs, &opts.scenarios)?;
    let mut jobs = Vec::new();
    for s in &scenarios {
        let engine = engines.get(&s.ns_card, &s.ew_card)?;
        let mut boards = bridge_encodings::pbn::read_pbn_file(&s.pbn)
            .map_err(|e| format!("{}: {e}", s.pbn.display()))?;
        if let Some(n) = opts.limit {
            boards.truncate(n);
        }
        jobs.extend(boards.into_iter().map(|b| (s, engine.clone(), b)));
    }

    let total = jobs.len();
    let done = AtomicUsize::new(0);
    let cache = par::DdCache::open(opts.dd_cache.clone());
    let mut results: Vec<(usize, BoardResult)> = jobs
        .par_iter()
        .enumerate()
        .filter_map(|(i, (scenario, engine, board))| {
            let mut r = board::compare(engine, &scenario.name, board)?;
            r.ns_card = scenario.ns_card.clone();
            r.ew_card = scenario.ew_card.clone();
            if opts.par && !r.contracts_match() {
                board::add_par(&mut r, board, &cache);
            }
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n.is_multiple_of(250) || n == total {
                progress(n, total);
            }
            Some((i, r))
        })
        .collect();
    results.sort_by_key(|(i, _)| *i);
    let boards: Vec<BoardResult> = results.into_iter().map(|(_, r)| r).collect();
    Ok(Report {
        summary: summarize(&boards),
        boards,
    })
}
