//! Statistics over compared boards.

use std::collections::{BTreeMap, HashMap};

use bridge_types::{Call, Direction};
use serde::Serialize;

use crate::board::BoardResult;
use crate::par::imps;

/// Agreement counts for one side's calls.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Agreement {
    pub agree: usize,
    pub total: usize,
}

impl Agreement {
    pub fn rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.agree as f64 / self.total as f64
        }
    }
}

/// On boards where the final contracts differ: who got closer to par.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ParTally {
    pub scored: usize,
    pub ours_closer: usize,
    pub reference_closer: usize,
    pub equal: usize,
    /// Sum over boards of (reference's IMP distance from par) minus (ours).
    /// Positive: our contracts were closer to par.
    pub imps_vs_reference: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Stats {
    pub name: String,
    pub boards: usize,
    /// Replay agreement: North-South calls, East-West calls.
    pub calls: [Agreement; 2],
    pub auctions_match: usize,
    pub contracts_match: usize,
    /// First divergence by call index (0 = the first call).
    pub first_divergence: BTreeMap<usize, usize>,
    pub par: ParTally,
    pub runaway: usize,
}

impl Stats {
    fn add(&mut self, b: &BoardResult) {
        self.boards += 1;
        let mut seat = b.dealer;
        for (r, e) in b.reference.iter().zip(&b.replay) {
            let side = matches!(seat, Direction::East | Direction::West) as usize;
            self.calls[side].total += 1;
            self.calls[side].agree += (r == e) as usize;
            seat = seat.next();
        }
        match b.first_divergence {
            None => self.auctions_match += 1,
            Some(i) => *self.first_divergence.entry(i).or_default() += 1,
        }
        self.contracts_match += b.contracts_match() as usize;
        self.runaway += b.runaway as usize;
        if let Some(p) = &b.par {
            self.par.scored += 1;
            let ours = imps((p.ours_ns - p.par_ns).abs());
            let reference = imps((p.reference_ns - p.par_ns).abs());
            match ours.cmp(&reference) {
                std::cmp::Ordering::Less => self.par.ours_closer += 1,
                std::cmp::Ordering::Greater => self.par.reference_closer += 1,
                std::cmp::Ordering::Equal => self.par.equal += 1,
            }
            self.par.imps_vs_reference += (reference - ours) as i64;
        }
    }

    pub fn calls_all(&self) -> Agreement {
        Agreement {
            agree: self.calls[0].agree + self.calls[1].agree,
            total: self.calls[0].total + self.calls[1].total,
        }
    }

    pub fn auction_rate(&self) -> f64 {
        ratio(self.auctions_match, self.boards)
    }

    pub fn contract_rate(&self) -> f64 {
        ratio(self.contracts_match, self.boards)
    }
}

fn ratio(a: usize, b: usize) -> f64 {
    if b == 0 {
        0.0
    } else {
        a as f64 / b as f64
    }
}

/// A place where the engine first departs from the reference, with how often.
#[derive(Debug, Clone, Serialize)]
pub struct Divergence {
    /// The calls before the difference, e.g. `1NT P 2C P`.
    pub auction: String,
    pub reference: String,
    pub ours: String,
    pub count: usize,
    /// Up to five `(board index in Report::boards)`, for the A/B view.
    pub examples: Vec<usize>,
    pub scenarios: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub total: Stats,
    pub scenarios: Vec<Stats>,
    /// Most frequent first.
    pub divergences: Vec<Divergence>,
}

/// Short call text: `P` for pass, `1NT`, `X`.
pub fn short(c: &Call) -> String {
    match c {
        Call::Pass => "P".into(),
        other => other.to_pbn(),
    }
}

pub fn summarize(boards: &[BoardResult]) -> Summary {
    let mut total = Stats {
        name: "ALL".into(),
        ..Stats::default()
    };
    let mut by_scenario: BTreeMap<&str, Stats> = BTreeMap::new();
    let mut points: HashMap<(String, String, String), Divergence> = HashMap::new();
    for (i, b) in boards.iter().enumerate() {
        total.add(b);
        by_scenario
            .entry(&b.scenario)
            .or_insert_with(|| Stats {
                name: b.scenario.clone(),
                ..Stats::default()
            })
            .add(b);
        if let Some(d) = b.first_divergence {
            let auction = b.reference[..d]
                .iter()
                .map(short)
                .collect::<Vec<_>>()
                .join(" ");
            let key = (auction.clone(), short(&b.reference[d]), short(&b.replay[d]));
            let e = points.entry(key.clone()).or_insert_with(|| Divergence {
                auction,
                reference: key.1.clone(),
                ours: key.2.clone(),
                count: 0,
                examples: Vec::new(),
                scenarios: Vec::new(),
            });
            e.count += 1;
            if e.examples.len() < 5 {
                e.examples.push(i);
            }
            if !e.scenarios.contains(&b.scenario) {
                e.scenarios.push(b.scenario.clone());
            }
        }
    }
    let mut divergences: Vec<Divergence> = points.into_values().collect();
    divergences.sort_by(|a, b| b.count.cmp(&a.count).then(a.auction.cmp(&b.auction)));
    Summary {
        total,
        scenarios: by_scenario.into_values().collect(),
        divergences,
    }
}
