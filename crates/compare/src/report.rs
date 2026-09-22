//! Statistics over compared boards.

use std::collections::{BTreeMap, HashMap};

use bridge_types::{Call, Direction};
use serde::Serialize;

use crate::board::{BoardResult, ProblemKind};
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
    /// Replay agreement split by the conditions each call was made under,
    /// to see whether they matter: caller not vulnerable / vulnerable, the
    /// board's scoring, and the reference file's generator.
    pub by_caller_vul: [Agreement; 2],
    pub by_scoring: BTreeMap<String, Agreement>,
    pub by_generator: BTreeMap<String, Agreement>,
    /// Problems in our own auctions, by kind, and boards with any.
    pub problems: BTreeMap<ProblemKind, usize>,
    pub boards_with_problems: usize,
    /// Boards by the reference's scoring and generator.
    pub boards_by_scoring: BTreeMap<String, usize>,
    pub boards_by_generator: BTreeMap<String, usize>,
}

impl Stats {
    fn add(&mut self, b: &BoardResult) {
        self.boards += 1;
        for p in &b.problems {
            *self.problems.entry(p.kind).or_default() += 1;
        }
        self.boards_with_problems += (!b.problems.is_empty()) as usize;
        let scoring = scoring_name(b);
        *self.boards_by_scoring.entry(scoring.clone()).or_default() += 1;
        *self
            .boards_by_generator
            .entry(b.generator.clone())
            .or_default() += 1;
        let mut seat = b.dealer;
        for (r, e) in b.reference.iter().zip(&b.replay) {
            let side = matches!(seat, Direction::East | Direction::West) as usize;
            let agree = (r == e) as usize;
            self.calls[side].total += 1;
            self.calls[side].agree += agree;
            let vul = b.vul.is_vulnerable(seat) as usize;
            self.by_caller_vul[vul].total += 1;
            self.by_caller_vul[vul].agree += agree;
            for a in [
                self.by_scoring.entry(scoring.clone()).or_default(),
                self.by_generator.entry(b.generator.clone()).or_default(),
            ] {
                a.total += 1;
                a.agree += agree;
            }
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

/// "MP", "IMP", ... or "not recorded".
pub fn scoring_name(b: &BoardResult) -> String {
    match b.scoring {
        Some(bridge_types::ScoringMethod::Matchpoints) => "MP".into(),
        Some(bridge_types::ScoringMethod::IMP) => "IMP".into(),
        Some(other) => format!("{other:?}"),
        None => "not recorded (assumed MP)".into(),
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
    /// Most frequent first.
    pub problems: Vec<ProblemPoint>,
}

/// A place in our auctions where a problem of one kind occurs.
#[derive(Debug, Clone, Serialize)]
pub struct ProblemPoint {
    pub kind: ProblemKind,
    /// The calls before the one at fault.
    pub auction: String,
    /// The call at fault.
    pub call: String,
    pub count: usize,
    /// Up to five board indices, for the A/B view.
    pub examples: Vec<usize>,
    pub scenarios: Vec<String>,
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
    let mut probs: HashMap<(ProblemKind, String, String), ProblemPoint> = HashMap::new();
    for (i, b) in boards.iter().enumerate() {
        for p in &b.problems {
            let auction = b.ours[..p.index.min(b.ours.len())]
                .iter()
                .map(short)
                .collect::<Vec<_>>()
                .join(" ");
            let call = b.ours.get(p.index).map(short).unwrap_or_default();
            let e = probs
                .entry((p.kind, auction.clone(), call.clone()))
                .or_insert_with(|| ProblemPoint {
                    kind: p.kind,
                    auction,
                    call,
                    count: 0,
                    examples: Vec::new(),
                    scenarios: Vec::new(),
                });
            e.count += 1;
            if e.examples.len() < 5 && !e.examples.contains(&i) {
                e.examples.push(i);
            }
            if !e.scenarios.contains(&b.scenario) {
                e.scenarios.push(b.scenario.clone());
            }
        }
    }
    let mut problems: Vec<ProblemPoint> = probs.into_values().collect();
    problems.sort_by(|a, b| b.count.cmp(&a.count).then(a.auction.cmp(&b.auction)));
    let mut divergences: Vec<Divergence> = points.into_values().collect();
    divergences.sort_by(|a, b| b.count.cmp(&a.count).then(a.auction.cmp(&b.auction)));
    Summary {
        total,
        scenarios: by_scenario.into_values().collect(),
        divergences,
        problems,
    }
}
