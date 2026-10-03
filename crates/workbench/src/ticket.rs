//! Tickets: what the workbench shows at one moment, with Rick's note, filed
//! as `tickets/YYYY/MM/DD/NN.<status>.<slug>/{ticket.md,context.json}` and
//! optionally as a GitHub issue. `NN` numbers the day's tickets from 01;
//! `<status>` mirrors the frontmatter (`open` when filed; renamed with it
//! by `probes/tools/ticket.py status`).
//!
//! `context.json` is the evidence: everything bridge-relevant the workbench
//! shows, as data, from the same sources the board detail draws from.
//! `ticket.md` is the readable summary, with the commands that reproduce
//! the board without the GUI. Nothing here touches egui, so it can be
//! tested headless.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use bridge_types::{Direction, Strain};
use chrono::{DateTime, Local, SecondsFormat};
use rbb_compare::{short, AuctionFilter, BoardResult, Stats};
use rbb_engine::{RuleRef, SeatKnowledge, SideState};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::detail::{caller, hand_line, knowledge_line, short_path, side_line, Detail, SEATS};

pub const SCHEMA_VERSION: u32 = 1;
/// GitHub refuses an issue body longer than this many characters.
pub const GITHUB_BODY_LIMIT: usize = 65_536;
/// What we aim for, leaving room for GitHub's own accounting.
const BODY_BUDGET: usize = 64_000;
/// Set (non-empty) to print the `gh` commands instead of running them.
pub const DRY_RUN_ENV: &str = "RBB_TICKET_DRY_RUN";
/// Where the last sink choice is remembered.
const SINK_FILE: &str = ".rbb-cache/ticket-sink";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Rule,
    Bug,
    Feature,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Rule, Kind::Bug, Kind::Feature];

    pub fn label(self) -> &'static str {
        match self {
            Kind::Rule => "Rule problem",
            Kind::Bug => "Bug",
            Kind::Feature => "Feature",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Rule => "rule",
            Kind::Bug => "bug",
            Kind::Feature => "feature",
        }
    }

    fn title_prefix(self) -> &'static str {
        match self {
            Kind::Rule => "Rule",
            Kind::Bug => "Bug",
            Kind::Feature => "Feature",
        }
    }

    /// GitHub labels for the issue.
    pub fn labels(self) -> Vec<String> {
        let second = match self {
            Kind::Rule => "rules",
            Kind::Bug => "bug",
            Kind::Feature => "enhancement",
        };
        vec!["workbench-ticket".into(), second.into()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sink {
    Local,
    GitHub,
}

impl Sink {
    pub fn label(self) -> &'static str {
        match self {
            Sink::Local => "Local folder",
            Sink::GitHub => "GitHub issue",
        }
    }

    /// The last choice, from `.rbb-cache/ticket-sink`.
    pub fn load() -> Sink {
        match std::fs::read_to_string(SINK_FILE).as_deref().map(str::trim) {
            Ok("github") => Sink::GitHub,
            _ => Sink::Local,
        }
    }

    pub fn save(self) {
        let text = match self {
            Sink::Local => "local",
            Sink::GitHub => "github",
        };
        let _ = std::fs::create_dir_all(".rbb-cache");
        let _ = std::fs::write(SINK_FILE, text);
    }
}

/// The `--auctions` argument for a filter.
pub fn auctions_arg(f: AuctionFilter) -> &'static str {
    match f {
        AuctionFilter::All => "all",
        AuctionFilter::UncontestedNs => "ns",
        AuctionFilter::UncontestedEw => "ew",
        AuctionFilter::Competitive => "competitive",
    }
}

// ---------------------------------------------------------------------------
// The context: what the workbench shows, as data.

/// The workbench's run settings (toolbar and command line).
#[derive(Debug, Clone, Default, Serialize)]
pub struct Settings {
    /// The scenarios box: names or patterns, space-separated; empty for all.
    pub scenarios: String,
    /// Boards per scenario, as typed; empty for all.
    pub limit: String,
    pub par: bool,
    /// `all`, `ns`, `ew` or `competitive`, as `compare --auctions` takes it.
    pub auctions: String,
    pub auctions_label: String,
    /// `--set` card changes, both sides.
    pub card_changes: Vec<String>,
    pub rules: String,
    pub pbs: String,
    /// Where `.test` files look up card names.
    pub cards: String,
    pub auto_rerun: bool,
}

/// One row of the divergence table.
#[derive(Debug, Clone, Serialize)]
pub struct DivergenceRow {
    /// BBA's auction up to the difference.
    pub auction: String,
    pub bba: String,
    pub ours: String,
    pub boards: usize,
    /// Ours against BBA in IMPs, summed over the row's boards.
    pub imps: i64,
}

/// One row of the problem table.
#[derive(Debug, Clone, Serialize)]
pub struct ProblemRow {
    pub kind: String,
    pub auction: String,
    pub call: String,
    pub boards: usize,
}

/// What is selected and filtered in the lists.
#[derive(Debug, Clone, Default, Serialize)]
pub struct View {
    pub tab: String,
    pub selected_scenario: Option<String>,
    pub divergence_filter: String,
    pub divergences_by_imps: bool,
    pub selected_divergence: Option<DivergenceRow>,
    pub selected_problem: Option<ProblemRow>,
    /// The selected `.test` case, on the Cases tab.
    pub selected_case: Option<rbb_engine::cases::Outcome>,
    pub running: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VsPar {
    /// Boards with differing contracts scored against par.
    pub scored: usize,
    pub ours_closer: usize,
    pub bba_closer: usize,
    pub equal: usize,
    /// Ours against BBA in IMPs, par as the yardstick.
    pub net_imps: i64,
}

impl VsPar {
    fn from(s: &Stats) -> Option<VsPar> {
        (s.par.scored > 0).then_some(VsPar {
            scored: s.par.scored,
            ours_closer: s.par.ours_closer,
            bba_closer: s.par.reference_closer,
            equal: s.par.equal,
            net_imps: s.par.imps_vs_reference,
        })
    }
}

/// The "since last run" line.
#[derive(Debug, Clone, Serialize)]
pub struct SinceLastRun {
    pub calls: i64,
    pub now_identical: usize,
    pub no_longer_identical: usize,
    pub now_same_contract: usize,
    pub lost_same_contract: usize,
}

/// The header's figures, over the boards the auction filter keeps. Rates
/// are fractions (0.5 is 50%).
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    /// The header as shown.
    pub line: String,
    pub boards: usize,
    pub took_secs: f64,
    pub calls_agreeing: f64,
    pub calls_ns: f64,
    pub calls_ew: f64,
    pub identical_auctions: f64,
    pub calls_not_vul: f64,
    pub calls_vul: f64,
    pub same_contract: f64,
    pub vs_par: Option<VsPar>,
    pub since_last_run: Option<SinceLastRun>,
    pub stats: Stats,
}

fn pct(x: f64) -> String {
    format!("{:.1}%", 100.0 * x)
}

impl Summary {
    pub fn new(t: &Stats, took_secs: f64, since_last_run: Option<SinceLastRun>) -> Summary {
        let vs_par = VsPar::from(t);
        let mut line = format!(
            "calls agreeing with BBA {} (NS {}, EW {}); identical auctions {}; \
             calls by caller not vul {} / vul {}; same contract {}",
            pct(t.calls_all().rate()),
            pct(t.calls[0].rate()),
            pct(t.calls[1].rate()),
            pct(t.auction_rate()),
            pct(t.by_caller_vul[0].rate()),
            pct(t.by_caller_vul[1].rate()),
            pct(t.contract_rate()),
        );
        if let Some(p) = &vs_par {
            line += &format!(
                "; vs par: ours closer {}, BBA closer {}, net {:+} IMPs",
                p.ours_closer, p.bba_closer, p.net_imps
            );
        }
        line += &format!("; {} boards in {took_secs:.1}s", t.boards);
        Summary {
            line,
            boards: t.boards,
            took_secs,
            calls_agreeing: t.calls_all().rate(),
            calls_ns: t.calls[0].rate(),
            calls_ew: t.calls[1].rate(),
            identical_auctions: t.auction_rate(),
            calls_not_vul: t.by_caller_vul[0].rate(),
            calls_vul: t.by_caller_vul[1].rate(),
            same_contract: t.contract_rate(),
            vs_par,
            since_last_run,
            stats: t.clone(),
        }
    }
}

/// A scenario's row of the scenario table.
#[derive(Debug, Clone, Serialize)]
pub struct ScenarioFigures {
    pub name: String,
    /// The row as shown, with the hover details.
    pub line: String,
    pub boards: usize,
    pub calls: f64,
    pub auction: f64,
    pub contract: f64,
    /// "bba/bd": ours against BBA in IMPs per board.
    pub bba_per_board: f64,
    pub vs_par: Option<VsPar>,
    pub boards_with_no_rule: usize,
    pub no_rule_positions: usize,
    pub stats: Stats,
}

impl ScenarioFigures {
    pub fn new(s: &Stats) -> ScenarioFigures {
        let no_rule_positions = s
            .problems
            .get(&rbb_compare::ProblemKind::NoRule)
            .copied()
            .unwrap_or(0);
        let line = format!(
            "{}: calls {}, auction {}, contract {}, bba/bd {:+.2} ({:+} IMPs over {} boards; \
             {} with differing contracts: ours closer {}, BBA closer {}, equal {}), \
             no rule {} ({no_rule_positions} positions)",
            s.name,
            pct(s.calls_all().rate()),
            pct(s.auction_rate()),
            pct(s.contract_rate()),
            s.par_per_board(),
            s.par.imps_vs_reference,
            s.boards,
            s.par.scored,
            s.par.ours_closer,
            s.par.reference_closer,
            s.par.equal,
            s.boards_with_no_rule,
        );
        ScenarioFigures {
            name: s.name.clone(),
            line,
            boards: s.boards,
            calls: s.calls_all().rate(),
            auction: s.auction_rate(),
            contract: s.contract_rate(),
            bba_per_board: s.par_per_board(),
            vs_par: VsPar::from(s),
            boards_with_no_rule: s.boards_with_no_rule,
            no_rule_positions,
            stats: s.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HandInfo {
    pub seat: char,
    /// S.H.D.C, as `rbb call` takes it.
    pub pbn: String,
    pub display: String,
    pub hcp: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParFigures {
    pub par_ns: i32,
    pub par_contract: String,
    /// NS scores, double dummy.
    pub bba_ns: i32,
    pub ours_ns: i32,
    /// "bba/bd" for this board: BBA's IMP distance from par minus ours,
    /// where the contracts differ; 0 where they are the same.
    pub vs_bba_imps: i32,
    /// "ours closer", "BBA closer", "equal", or "same contract".
    pub verdict: String,
}

/// One declarer's row of the double-dummy table.
#[derive(Debug, Clone, Serialize)]
pub struct DdRow {
    pub declarer: char,
    pub nt: u8,
    pub s: u8,
    pub h: u8,
    pub d: u8,
    pub c: u8,
}

/// The board header, hands, contracts and par, as the detail shows them.
#[derive(Debug, Clone, Serialize)]
pub struct BoardFigures {
    pub header: String,
    pub hands: Vec<HandInfo>,
    pub bba_auction: String,
    pub our_auction: String,
    pub bba_contract: String,
    pub our_contract: String,
    pub contracts_match: bool,
    pub par: Option<ParFigures>,
    pub dd_table: Option<Vec<DdRow>>,
    /// Problems in our auction, as shown.
    pub problems: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateRow {
    pub call: String,
    /// The call BBA made here (underlined in the workbench).
    pub is_bba: bool,
    pub priority: i64,
    pub descriptiveness: f64,
    pub prefer: Option<f64>,
    pub outcome: String,
    pub meaning: String,
    pub rule: RuleRef,
    pub rule_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeatShown {
    pub seat: char,
    pub summary: String,
    pub knowledge: SeatKnowledge,
}

#[derive(Debug, Clone, Serialize)]
pub struct SideShown {
    pub side: String,
    pub summary: String,
    pub state: SideState,
}

/// The engine's reasoning at the first difference.
#[derive(Debug, Clone, Serialize)]
pub struct FirstDifference {
    /// Index into the auction (0-based); `call_number` is 1-based.
    pub index: usize,
    pub call_number: usize,
    pub seat: char,
    pub hand: String,
    pub hand_pbn: String,
    pub hcp: u8,
    /// BBA's auction before the difference, short and as PBN calls.
    pub auction_so_far: String,
    pub auction_pbn: String,
    pub bba_call: String,
    pub bba_alert: Option<String>,
    pub our_call: String,
    pub explanation: String,
    pub alert: Value,
    pub rule: Option<RuleRef>,
    /// Whether any rule offered BBA's call.
    pub bba_offered: bool,
    /// Best-ranked first.
    pub candidates: Vec<CandidateRow>,
    pub warnings: Vec<String>,
    /// What each seat had shown at that point, W N E S.
    pub seats: Vec<SeatShown>,
    /// Each side's auction state: trump, forcing, asks.
    pub sides: Vec<SideShown>,
}

/// One row of "How the engine read BBA's auction".
#[derive(Debug, Clone, Serialize)]
pub struct ReadingRow {
    pub n: usize,
    pub seat: char,
    pub bba_call: String,
    pub bba_alert: Option<String>,
    /// The engine's reading; `None`: no rule.
    pub reading: Option<String>,
    pub rule: Option<RuleRef>,
    pub rule_at: Option<String>,
    pub knowledge_summary: String,
    pub knowledge: SeatKnowledge,
    /// What the engine bids in that seat, given BBA's auction so far.
    pub engine_would_bid: String,
    pub agrees: bool,
    pub artificial: bool,
    pub engine_alert: Value,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GitInfo {
    pub commit: String,
    pub branch: String,
    pub dirty: bool,
    /// `git status --porcelain` lines: status code, then path.
    pub modified: Vec<String>,
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

impl GitInfo {
    pub fn collect(root: &Path) -> GitInfo {
        let modified: Vec<String> = git(root, &["status", "--porcelain"])
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect();
        GitInfo {
            commit: git(root, &["rev-parse", "HEAD"]).unwrap_or_default(),
            branch: git(root, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default(),
            dirty: !modified.is_empty(),
            modified,
        }
    }
}

/// The repository's top directory (tickets go there), else the current one.
pub fn repo_root() -> PathBuf {
    git(Path::new("."), &["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Commands that reproduce the ticket without the GUI.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Reproduce {
    /// `rbb call` at the first difference.
    pub call: Option<String>,
    pub compare: String,
    pub notes: Vec<String>,
}

/// The knowledge view's review of one auction, as `rbb explain-auction`
/// prints it: filed from the knowledge view, what Rick was looking at.
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeReview {
    /// `ours` or `bba` (BBA's auction as the engine reads it).
    pub auction: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Context {
    pub schema_version: u32,
    pub created: String,
    pub kind: Kind,
    pub note: String,
    pub git: GitInfo,
    pub workbench: Settings,
    pub view: View,
    pub summary: Option<Summary>,
    /// The scenario-table row of the board's scenario (else the selected
    /// scenario's).
    pub scenario: Option<ScenarioFigures>,
    /// The selected board, as the comparison produced it.
    pub board: Option<BoardResult>,
    pub board_figures: Option<BoardFigures>,
    pub first_difference: Option<FirstDifference>,
    /// "How the engine read BBA's auction", one row per call.
    pub bba_reading: Vec<ReadingRow>,
    /// The knowledge view's reviews, when the ticket was filed from it:
    /// the auction it showed first, then the other (Rick, ticket b400:
    /// both auctions, so ours can be read beside BBA's).
    pub knowledge: Vec<KnowledgeReview>,
    pub detail_error: Option<String>,
    pub reproduce: Reproduce,
    #[serde(skip)]
    pub stamp: DateTime<Local>,
}

/// Everything the app hands over to build a context.
pub struct Capture<'a> {
    pub kind: Kind,
    pub note: String,
    pub created: DateTime<Local>,
    pub git: GitInfo,
    pub settings: Settings,
    pub view: View,
    pub summary: Option<Summary>,
    pub scenario: Option<ScenarioFigures>,
    pub board: Option<(&'a BoardResult, &'a Detail)>,
    /// Filed from the knowledge view: which auction it showed (`true`:
    /// BBA's). Both auctions' reviews go into the ticket, that one first.
    pub knowledge: Option<bool>,
}

fn calls_text(calls: &[bridge_types::Call]) -> String {
    calls.iter().map(short).collect::<Vec<_>>().join(" ")
}

fn rule_at(r: &RuleRef) -> String {
    format!("{}:{}", short_path(&r.file), r.line)
}

const STRAIN_ORDER: [Strain; 5] = [
    Strain::NoTrump,
    Strain::Spades,
    Strain::Hearts,
    Strain::Diamonds,
    Strain::Clubs,
];

fn board_figures(b: &BoardResult, d: &Detail) -> BoardFigures {
    let header = format!(
        "{} — board {}: dealer {}   vul {}   scoring {}   NS card {}   EW card {}   reference: {}",
        b.scenario,
        b.board,
        b.dealer.to_char(),
        b.vul.to_pbn(),
        rbb_compare::scoring_name(b),
        b.ns_card,
        b.ew_card,
        b.generator
    );
    let hands = d
        .deal
        .as_ref()
        .map(|deal| {
            [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ]
            .iter()
            .map(|&s| {
                let h = deal.hand(s);
                HandInfo {
                    seat: s.to_char(),
                    pbn: h.to_pbn(),
                    display: hand_line(h),
                    hcp: h.hcp(),
                }
            })
            .collect()
        })
        .unwrap_or_default();
    let par = b.par.as_ref().map(|p| {
        let ours = rbb_compare::par::imps((p.ours_ns - p.par_ns).abs());
        let reference = rbb_compare::par::imps((p.reference_ns - p.par_ns).abs());
        let (vs_bba_imps, verdict) = if b.contracts_match() {
            (0, "same contract")
        } else {
            let verdict = match (p.ours_ns - p.par_ns)
                .abs()
                .cmp(&(p.reference_ns - p.par_ns).abs())
            {
                std::cmp::Ordering::Less => "ours closer",
                std::cmp::Ordering::Greater => "BBA closer",
                std::cmp::Ordering::Equal => "equal",
            };
            (reference - ours, verdict)
        };
        ParFigures {
            par_ns: p.par_ns,
            par_contract: p.par_contract.clone(),
            bba_ns: p.reference_ns,
            ours_ns: p.ours_ns,
            vs_bba_imps,
            verdict: verdict.into(),
        }
    });
    let dd_table = b.dd.as_ref().map(|dd| {
        [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ]
        .iter()
        .map(|&s| {
            let t = |x| dd.tricks(s, x);
            DdRow {
                declarer: s.to_char(),
                nt: t(Strain::NoTrump),
                s: t(Strain::Spades),
                h: t(Strain::Hearts),
                d: t(Strain::Diamonds),
                c: t(Strain::Clubs),
            }
        })
        .collect()
    });
    BoardFigures {
        header,
        hands,
        bba_auction: calls_text(&b.reference),
        our_auction: calls_text(&b.ours),
        bba_contract: b
            .reference_contract
            .clone()
            .unwrap_or_else(|| "passed out".into()),
        our_contract: b
            .our_contract
            .clone()
            .unwrap_or_else(|| "passed out".into()),
        contracts_match: b.contracts_match(),
        par,
        dd_table,
        problems: b
            .problems
            .iter()
            .map(|p| format!("{} at call {}: {}", p.kind.label(), p.index + 1, p.detail))
            .collect(),
    }
}

fn first_difference(b: &BoardResult, d: &Detail) -> Option<FirstDifference> {
    let (i, dec) = (b.first_divergence?, d.decision.as_ref()?);
    let seat = caller(b.dealer, i);
    let hand = d.deal.as_ref().map(|x| x.hand(seat));
    let bba = &b.reference[i];
    let pos = &dec.auction.position;
    Some(FirstDifference {
        index: i,
        call_number: i + 1,
        seat: seat.to_char(),
        hand: hand.map(hand_line).unwrap_or_default(),
        hand_pbn: hand.map(|h| h.to_pbn()).unwrap_or_default(),
        hcp: hand.map(|h| h.hcp()).unwrap_or(0),
        auction_so_far: calls_text(&b.reference[..i]),
        auction_pbn: b.reference[..i]
            .iter()
            .map(|c| c.to_pbn())
            .collect::<Vec<_>>()
            .join(" "),
        bba_call: short(bba),
        bba_alert: b.reference_alerts.get(i).cloned().flatten(),
        our_call: short(&dec.call),
        explanation: dec.explanation.clone(),
        alert: serde_json::to_value(&dec.alert).unwrap_or(Value::Null),
        rule: dec.rule.clone(),
        bba_offered: dec.candidates.iter().any(|c| &c.call == bba),
        candidates: dec
            .candidates
            .iter()
            .map(|c| CandidateRow {
                call: short(&c.call),
                is_bba: &c.call == bba,
                priority: c.priority,
                descriptiveness: c.descriptiveness,
                prefer: c.prefer,
                outcome: c.outcome.clone(),
                meaning: c.explanation.clone(),
                rule: c.rule.clone(),
                rule_at: rule_at(&c.rule),
            })
            .collect(),
        warnings: dec.warnings.clone(),
        seats: SEATS
            .iter()
            .map(|&s| SeatShown {
                seat: s.to_char(),
                summary: knowledge_line(pos.knowledge(s)),
                knowledge: pos.knowledge(s).clone(),
            })
            .collect(),
        sides: [(0usize, "NS"), (1, "EW")]
            .iter()
            .map(|&(i, name)| SideShown {
                side: name.into(),
                summary: side_line(&pos.sides[i]),
                state: pos.sides[i].clone(),
            })
            .collect(),
    })
}

fn bba_reading(b: &BoardResult, d: &Detail) -> Vec<ReadingRow> {
    let Some(reading) = &d.reading else {
        return vec![];
    };
    reading
        .steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            let ours = b.replay.get(i);
            ReadingRow {
                n: i + 1,
                seat: step.caller.to_char(),
                bba_call: short(&step.call),
                bba_alert: b.reference_alerts.get(i).cloned().flatten(),
                reading: step.explanation.clone(),
                rule: step.rule.clone(),
                rule_at: step.rule.as_ref().map(rule_at),
                knowledge_summary: knowledge_line(&step.knowledge),
                knowledge: step.knowledge.clone(),
                engine_would_bid: ours.map(short).unwrap_or_default(),
                agrees: ours == Some(&step.call),
                artificial: step.artificial,
                engine_alert: serde_json::to_value(&step.alert).unwrap_or(Value::Null),
                warnings: step.warnings.clone(),
            }
        })
        .collect()
}

/// Quote for a POSIX shell when needed.
fn sh(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./=:,+@%".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

fn absolute(p: &str) -> String {
    std::fs::canonicalize(p)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| p.to_string())
}

fn reproduce(s: &Settings, b: Option<&BoardResult>, fd: Option<&FirstDifference>) -> Reproduce {
    let pbs = absolute(&s.pbs);
    let rules = (s.rules != "conventions").then(|| absolute(&s.rules));
    let mut notes = vec![];
    let call = match (b, fd) {
        (Some(b), Some(fd)) => {
            let card = |name: &str| format!("{pbs}/bbsa/{name}.bbsa");
            let scoring = match b.scoring {
                Some(bridge_types::ScoringMethod::IMP) => "IMP",
                _ => "MP",
            };
            let mut c = format!(
                "cargo run -q --release -p rbb-cli -- call {} -a {} -d {} -v {} -s {scoring} -c {}",
                sh(&fd.hand_pbn),
                sh(&fd.auction_pbn),
                b.dealer.to_char(),
                b.vul.to_pbn(),
                sh(&card(&b.ns_card)),
            );
            if b.ew_card != b.ns_card {
                c += &format!(" --ew-card {}", sh(&card(&b.ew_card)));
            }
            if let Some(r) = &rules {
                c += &format!(" --rules {}", sh(r));
            }
            if !s.card_changes.is_empty() {
                notes.push(format!(
                    "`rbb call` has no --set: the workbench ran with {}, which the call line \
                     does not apply.",
                    s.card_changes
                        .iter()
                        .map(|c| format!("--set {c}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                ));
            }
            Some(c)
        }
        _ => None,
    };
    let scenario = b
        .map(|b| b.scenario.clone())
        .unwrap_or_else(|| s.scenarios.clone());
    let mut compare = String::from("cargo run -q --release -p rbb-cli -- compare");
    for pat in scenario.split_whitespace() {
        compare += &format!(" '{pat}'");
    }
    compare += &format!(" --pbs {}", sh(&pbs));
    if let Some(r) = &rules {
        compare += &format!(" --rules {}", sh(r));
    }
    if !s.limit.trim().is_empty() {
        compare += &format!(" --limit {}", s.limit.trim());
    }
    if s.par {
        compare += " --par";
    }
    for c in &s.card_changes {
        compare += &format!(" --set {}", sh(c));
    }
    if s.auctions != "all" && !s.auctions.is_empty() {
        compare += &format!(" --auctions {}", s.auctions);
    }
    compare += " --json /tmp/rbb-compare.json";
    if let Some(b) = b {
        notes.push(format!(
            "In the JSON report, the board is the one with scenario {} and board {}.",
            b.scenario, b.board
        ));
    }
    Reproduce {
        call,
        compare,
        notes,
    }
}

impl Context {
    pub fn build(c: Capture) -> Context {
        let (board, board_figures, first, reading, detail_error) = match c.board {
            Some((b, d)) => (
                Some(b.clone()),
                Some(board_figures(b, d)),
                first_difference(b, d),
                bba_reading(b, d),
                d.error.clone(),
            ),
            None => (None, None, None, vec![], None),
        };
        let knowledge = match (c.board, c.knowledge) {
            (Some((_, d)), Some(shown)) => [shown, !shown]
                .into_iter()
                .map(|bba| KnowledgeReview {
                    auction: if bba { "bba" } else { "ours" }.into(),
                    text: rbb_engine::review_text(&d.review[usize::from(bba)]),
                })
                .collect(),
            _ => Vec::new(),
        };
        let reproduce = reproduce(&c.settings, board.as_ref(), first.as_ref());
        Context {
            schema_version: SCHEMA_VERSION,
            created: c.created.to_rfc3339_opts(SecondsFormat::Secs, false),
            kind: c.kind,
            note: c.note,
            git: c.git,
            workbench: c.settings,
            view: c.view,
            summary: c.summary,
            scenario: c.scenario,
            board,
            board_figures,
            first_difference: first,
            bba_reading: reading,
            knowledge,
            detail_error,
            reproduce,
            stamp: c.created,
        }
    }

    pub fn scenario_name(&self) -> Option<&str> {
        self.board
            .as_ref()
            .map(|b| b.scenario.as_str())
            .or(self.view.selected_scenario.as_deref())
    }

    pub fn board_number(&self) -> Option<&str> {
        self.board.as_ref().map(|b| b.board.as_str())
    }
}

// ---------------------------------------------------------------------------
// ticket.md

/// A YAML scalar: bare when safe, else double-quoted.
fn yaml(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let safe = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_./:+".contains(c))
        && !s.starts_with(|c: char| "-:".contains(c));
    if safe {
        s.to_string()
    } else {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = format!("| {} |\n", headers.join(" | "));
    out += &format!("|{}\n", "---|".repeat(headers.len()));
    for r in rows {
        let cells: Vec<String> = r.iter().map(|c| cell(c)).collect();
        out += &format!("| {} |\n", cells.join(" | "));
    }
    out
}

/// The note's first non-empty line.
fn first_line(note: &str) -> &str {
    note.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", t.trim_end())
    }
}

/// `Rule: …`, `Bug: …` or `Feature: …`.
pub fn title(c: &Context) -> String {
    let line = first_line(&c.note);
    let text = if line.is_empty() {
        match (c.scenario_name(), c.board_number()) {
            (Some(s), Some(b)) => format!("{s} board {b}"),
            (Some(s), None) => s.to_string(),
            _ => "workbench".into(),
        }
    } else {
        line.to_string()
    };
    format!("{}: {}", c.kind.title_prefix(), truncate(&text, 80))
}

pub fn render_markdown(c: &Context) -> String {
    let mut md = String::new();
    md += "---\n";
    md += "status: open\n";
    md += &format!("kind: {}\n", c.kind.name());
    md += &format!("created: {}\n", yaml(&c.created));
    md += &format!("commit: {}\n", yaml(&c.git.commit));
    md += &format!("scenario: {}\n", yaml(c.scenario_name().unwrap_or("")));
    md += &format!("board: {}\n", yaml(c.board_number().unwrap_or("")));
    md += "issue: \n";
    md += "resolution: \n";
    md += "refs: []\n";
    md += "---\n\n";
    md += &format!("# {}\n\n", title(c));

    md += "## Note\n\n";
    if c.note.trim().is_empty() {
        md += "_(no note)_\n\n";
    } else {
        md += c.note.trim_end();
        md += "\n\n";
    }

    if let (Some(b), Some(f)) = (&c.board, &c.board_figures) {
        md += "## Board\n\n";
        md += &format!("**{}**\n\n", f.header);
        if let Some(e) = &c.detail_error {
            md += &format!("Detail error: {e}\n\n");
        }
        if !f.hands.is_empty() {
            let rows: Vec<Vec<String>> = f
                .hands
                .iter()
                .map(|h| {
                    vec![
                        h.seat.to_string(),
                        format!("`{}`", h.display),
                        h.hcp.to_string(),
                    ]
                })
                .collect();
            md += &table(&["seat", "hand", "HCP"], &rows);
            md += "\n";
        }

        md += "### Auctions\n\n";
        let n = b.reference.len().max(b.ours.len());
        let rows: Vec<Vec<String>> = (0..n)
            .map(|i| {
                let mark = |s: String| {
                    if Some(i) == b.first_divergence && !s.is_empty() {
                        format!("**{s}**")
                    } else {
                        s
                    }
                };
                vec![
                    (i + 1).to_string(),
                    caller(b.dealer, i).to_char().to_string(),
                    mark(b.reference.get(i).map(short).unwrap_or_default()),
                    mark(b.ours.get(i).map(short).unwrap_or_default()),
                    if Some(i) == b.first_divergence {
                        "first difference".into()
                    } else {
                        String::new()
                    },
                ]
            })
            .collect();
        md += &table(&["#", "seat", "BBA", "ours", ""], &rows);
        md += &format!(
            "\nBBA: `{}`  \nours: `{}`\n\n",
            f.bba_auction, f.our_auction
        );

        md += "### Contracts and par\n\n";
        md += &format!("- BBA's contract: {}\n", f.bba_contract);
        md += &format!(
            "- our contract: {}{}\n",
            f.our_contract,
            if f.contracts_match { " (same)" } else { "" }
        );
        match &f.par {
            Some(p) => {
                md += &format!("- par: {:+} ({})\n", p.par_ns, p.par_contract);
                md += &format!(
                    "- NS scores, double dummy: BBA {:+}, ours {:+}\n",
                    p.bba_ns, p.ours_ns
                );
                md += &format!(
                    "- vs BBA (bba/bd): {:+} IMPs ({})\n",
                    p.vs_bba_imps, p.verdict
                );
            }
            None => md += "- par: not solved\n",
        }
        md += "\n";
        if let Some(dd) = &f.dd_table {
            md += "Double dummy (tricks):\n\n";
            let rows: Vec<Vec<String>> = dd
                .iter()
                .map(|r| {
                    vec![
                        r.declarer.to_string(),
                        r.nt.to_string(),
                        r.s.to_string(),
                        r.h.to_string(),
                        r.d.to_string(),
                        r.c.to_string(),
                    ]
                })
                .collect();
            let heads: Vec<&str> = std::iter::once("")
                .chain(STRAIN_ORDER.iter().map(|s| s.symbol()))
                .collect();
            md += &table(&heads, &rows);
            md += "\n";
        }

        md += "### Problems\n\n";
        if f.problems.is_empty() {
            md += "none\n\n";
        } else {
            for p in &f.problems {
                md += &format!("- {p}\n");
            }
            md += "\n";
        }
    }

    if let Some(fd) = &c.first_difference {
        md += "## First difference\n\n";
        md += &format!(
            "Call {}: {} holds `{}` ({} HCP), after `{}`. BBA bid **{}**, the engine chose **{}** — {}\n\n",
            fd.call_number,
            fd.seat,
            fd.hand,
            fd.hcp,
            if fd.auction_so_far.is_empty() {
                "(nothing)"
            } else {
                &fd.auction_so_far
            },
            fd.bba_call,
            fd.our_call,
            fd.explanation
        );
        if let Some(a) = &fd.bba_alert {
            md += &format!("BBA's alert for {}: {a}\n\n", fd.bba_call);
        }
        let rows: Vec<Vec<String>> = fd
            .candidates
            .iter()
            .map(|c| {
                vec![
                    if c.is_bba {
                        format!("**{}** (BBA)", c.call)
                    } else {
                        c.call.clone()
                    },
                    c.priority.to_string(),
                    format!("{:.3}", c.descriptiveness),
                    c.outcome.clone(),
                    c.meaning.clone(),
                    format!("`{}`", c.rule_at),
                ]
            })
            .collect();
        md += "Candidates, best-ranked first:\n\n";
        md += &table(
            &["call", "prio", "descr", "outcome", "meaning", "rule"],
            &rows,
        );
        md += "\n";
        if !fd.bba_offered {
            md += &format!("No rule offers BBA's {} here.\n\n", fd.bba_call);
        }
        for w in &fd.warnings {
            md += &format!("- warning: {w}\n");
        }
        md += "What each seat had shown at that point:\n\n";
        let rows: Vec<Vec<String>> = fd
            .seats
            .iter()
            .map(|s| vec![s.seat.to_string(), format!("`{}`", s.summary)])
            .collect();
        md += &table(&["seat", "shown"], &rows);
        md += "\n";
        for s in &fd.sides {
            md += &format!("- {}: {}\n", s.side, s.summary);
        }
        md += "\n";
    }

    if !c.bba_reading.is_empty() {
        md += "## How the engine read BBA's auction\n\n";
        let rows: Vec<Vec<String>> = c
            .bba_reading
            .iter()
            .map(|r| {
                vec![
                    r.n.to_string(),
                    r.seat.to_string(),
                    r.bba_call.clone(),
                    r.bba_alert.clone().unwrap_or_default(),
                    r.reading.clone().unwrap_or_else(|| "_no rule_".into()),
                    r.rule_at
                        .as_ref()
                        .map(|x| format!("`{x}`"))
                        .unwrap_or_default(),
                    format!("`{}`", r.knowledge_summary),
                    if r.agrees {
                        r.engine_would_bid.clone()
                    } else {
                        format!("**{}**", r.engine_would_bid)
                    },
                ]
            })
            .collect();
        md += &table(
            &[
                "#",
                "seat",
                "BBA",
                "BBA alert",
                "engine's reading",
                "rule",
                "knowledge",
                "engine would bid",
            ],
            &rows,
        );
        md += "\n";
    }

    for k in &c.knowledge {
        md += &format!(
            "## Knowledge view ({})\n\n```text\n{}```\n\n",
            if k.auction == "bba" {
                "BBA's auction, as the engine reads it"
            } else {
                "our auction"
            },
            k.text
        );
    }

    md += "## Workbench\n\n";
    let s = &c.workbench;
    md += &format!(
        "- scenarios: `{}`; boards per scenario: {}; par: {}; auctions: {}\n",
        if s.scenarios.is_empty() {
            "all"
        } else {
            &s.scenarios
        },
        if s.limit.trim().is_empty() {
            "all"
        } else {
            s.limit.trim()
        },
        if s.par { "on" } else { "off" },
        s.auctions_label
    );
    if !s.card_changes.is_empty() {
        md += &format!(
            "- card changes: `--set {}`\n",
            s.card_changes.join("` `--set ")
        );
    }
    md += &format!("- rules: `{}`; PBS: `{}`\n", s.rules, s.pbs);
    let v = &c.view;
    md += &format!(
        "- tab: {}; selected scenario: {}",
        v.tab,
        v.selected_scenario.as_deref().unwrap_or("all")
    );
    if !v.divergence_filter.is_empty() {
        md += &format!("; divergence filter `{}`", v.divergence_filter);
    }
    if v.divergences_by_imps {
        md += "; divergences by IMPs";
    }
    md += "\n";
    if let Some(d) = &v.selected_divergence {
        md +=
            &format!(
            "- selected divergence: after `{}`, BBA {} / ours {} — {} boards, {:+} IMPs vs BBA\n",
            if d.auction.is_empty() { "(opening)" } else { &d.auction },
            d.bba,
            d.ours,
            d.boards,
            d.imps
        );
    }
    if let Some(p) = &v.selected_problem {
        md += &format!(
            "- selected problem: {} after `{}` at {} — {} boards\n",
            p.kind, p.auction, p.call, p.boards
        );
    }
    if let Some(o) = &v.selected_case {
        md += &format!("- selected case: {}\n", o.summary());
    }
    if let Some(sum) = &c.summary {
        md += &format!("- summary: {}\n", sum.line);
        if let Some(d) = &sum.since_last_run {
            md += &format!(
                "- since last run: {:+} calls agree, +{} / -{} identical auctions, +{} / -{} same contract\n",
                d.calls,
                d.now_identical,
                d.no_longer_identical,
                d.now_same_contract,
                d.lost_same_contract
            );
        }
    }
    if let Some(sc) = &c.scenario {
        md += &format!("- scenario: {}\n", sc.line);
    }
    if let Some(e) = &v.error {
        md += &format!("- error shown: {e}\n");
    }
    md += "\n";

    md += "## Reproduce\n\n```sh\n";
    if let Some(call) = &c.reproduce.call {
        md += call;
        md += "\n";
    }
    md += &c.reproduce.compare;
    md += "\n```\n\n";
    for n in &c.reproduce.notes {
        md += &format!("- {n}\n");
    }
    if !c.reproduce.notes.is_empty() {
        md += "\n";
    }

    md += "## Source\n\n";
    md += &format!(
        "- commit `{}` on `{}`{}\n",
        c.git.commit,
        c.git.branch,
        if c.git.dirty {
            ", with local changes:"
        } else {
            ", clean"
        }
    );
    for m in &c.git.modified {
        md += &format!("  - `{}`\n", m.trim());
    }
    md += "- full context: `context.json` (schema version ";
    md += &format!("{})\n", c.schema_version);
    md
}

/// The markdown without its frontmatter.
pub fn strip_frontmatter(md: &str) -> &str {
    if let Some(rest) = md.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            return rest[end + 5..].trim_start_matches('\n');
        }
    }
    md
}

/// Set `key: value` in the frontmatter.
pub fn set_frontmatter(md: &str, key: &str, value: &str) -> String {
    let Some(rest) = md.strip_prefix("---\n") else {
        return md.to_string();
    };
    let Some(end) = rest.find("\n---\n") else {
        return md.to_string();
    };
    let (front, body) = rest.split_at(end);
    let prefix = format!("{key}:");
    let mut found = false;
    let mut lines: Vec<String> = front
        .lines()
        .map(|l| {
            if l.starts_with(&prefix) {
                found = true;
                format!("{key}: {}", yaml(value))
            } else {
                l.to_string()
            }
        })
        .collect();
    if !found {
        lines.push(format!("{key}: {}", yaml(value)));
    }
    format!("---\n{}{}", lines.join("\n"), body)
}

// ---------------------------------------------------------------------------
// Local sink

/// Lowercase words joined by `-`: the scenario, the board, and the note's
/// first few words.
pub fn slug(scenario: Option<&str>, board: Option<&str>, note: &str) -> String {
    let clean = |s: &str| -> Vec<String> {
        s.split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(|w| w.to_ascii_lowercase())
            .collect()
    };
    let mut parts: Vec<String> = vec![];
    if let Some(s) = scenario {
        parts.extend(clean(s));
    }
    if let Some(b) = board {
        parts.push(format!("b{}", clean(b).join("")));
    }
    parts.extend(clean(first_line(note)).into_iter().take(5));
    let mut out = String::new();
    for p in parts {
        if !out.is_empty() && out.len() + p.len() + 1 > 60 {
            break;
        }
        if !out.is_empty() {
            out.push('-');
        }
        out += &p;
    }
    if out.is_empty() {
        "ticket".into()
    } else {
        out.chars().take(60).collect()
    }
}

/// Write `ticket.md` and `context.json` under
/// `root/tickets/YYYY/MM/DD/NN.open.<slug>/`, `NN` the next number that
/// day. Returns the directory.
pub fn write_local(root: &Path, c: &Context) -> Result<PathBuf, String> {
    let base = root
        .join("tickets")
        .join(c.stamp.format("%Y").to_string())
        .join(c.stamp.format("%m").to_string())
        .join(c.stamp.format("%d").to_string());
    let dir = base.join(format!(
        "{:02}.open.{}",
        next_index(&base),
        slug(c.scenario_name(), c.board_number(), &c.note)
    ));
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let json = serde_json::to_string_pretty(c).map_err(|e| e.to_string())?;
    let write = |file: &str, text: &str| {
        let p = dir.join(file);
        std::fs::write(&p, text).map_err(|e| format!("{}: {e}", p.display()))
    };
    write("context.json", &json)?;
    write("ticket.md", &render_markdown(c))?;
    Ok(dir)
}

/// One more than the highest `NN.` among the day's ticket folders.
fn next_index(day: &Path) -> u32 {
    std::fs::read_dir(day)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.split_once('.')?.0.parse::<u32>().ok()
        })
        .max()
        .unwrap_or(0)
        + 1
}

/// Record the issue URL in a written ticket's frontmatter.
pub fn record_issue(dir: &Path, url: &str) -> Result<(), String> {
    let p = dir.join("ticket.md");
    let md = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    std::fs::write(&p, set_frontmatter(&md, "issue", url))
        .map_err(|e| format!("{}: {e}", p.display()))
}

/// The prompt to paste into Claude Code.
pub fn claude_prompt(dir: &Path, c: &Context) -> String {
    let repro = c
        .reproduce
        .call
        .as_deref()
        .map(|r| {
            format!(
                " The ticket and context.json are usually enough to diagnose; run the \
                 `rbb call` line in ticket.md ({r}) only if the rules have changed since \
                 the ticket's commit, or to test a fix."
            )
        })
        .unwrap_or_else(|| {
            " The ticket and context.json are usually enough to diagnose; run the \
             commands in ticket.md only if the rules have changed since the ticket's \
             commit, or to test a fix."
                .into()
        });
    format!(
        "Work the ticket at {}. Read ticket.md first, then context.json.{repro} \
         Diagnose before changing anything, and judge any change by the house rules in \
         CLAUDE.md. When done, record the outcome in ticket.md's frontmatter: status \
         (resolved, wontfix or duplicate), resolution, and refs to the commits. Do not \
         edit context.json.",
        dir.display()
    )
}

// ---------------------------------------------------------------------------
// GitHub sink

/// `owner/repo` from a GitHub remote URL.
pub fn parse_repo(url: &str) -> Option<String> {
    let url = url.trim();
    let path = url
        .strip_prefix("git@github.com:")
        .or_else(|| url.strip_prefix("ssh://git@github.com/"))
        .or_else(|| url.strip_prefix("https://github.com/"))
        .or_else(|| url.strip_prefix("http://github.com/"))?;
    let path = path.trim_end_matches('/').trim_end_matches(".git");
    let mut it = path.split('/');
    let (owner, repo) = (it.next()?, it.next()?);
    (!owner.is_empty() && !repo.is_empty() && it.next().is_none())
        .then(|| format!("{owner}/{repo}"))
}

/// The repository of `git remote get-url origin`.
pub fn origin_repo(root: &Path) -> Option<String> {
    parse_repo(&git(root, &["remote", "get-url", "origin"])?)
}

type Trim = (&'static str, fn(&mut Value) -> bool);

fn drop_key(v: &mut Value, path: &[&str]) -> bool {
    let (last, parents) = path.split_last().unwrap();
    let mut cur = v;
    for p in parents {
        match cur.get_mut(*p) {
            Some(x) => cur = x,
            None => return false,
        }
    }
    cur.as_object_mut()
        .and_then(|o| o.remove(*last))
        .is_some_and(|x| match x {
            Value::Null => false,
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
            _ => true,
        })
}

fn drop_in_each(v: &mut Value, array: &[&str], key: &str) -> bool {
    let mut cur = v;
    for p in array {
        match cur.get_mut(*p) {
            Some(x) => cur = x,
            None => return false,
        }
    }
    let mut any = false;
    if let Some(items) = cur.as_array_mut() {
        for it in items {
            if let Some(o) = it.as_object_mut() {
                any |= o.remove(key).is_some();
            }
        }
    }
    any
}

/// What goes first when the context is too big for an issue body: the
/// big arrays, then whole sections. Each entry says what was dropped.
const TRIMS: [Trim; 13] = [
    (
        "bba_reading[].knowledge (the one-line knowledge_summary is kept)",
        |v| drop_in_each(v, &["bba_reading"], "knowledge"),
    ),
    (
        "first_difference.seats[].knowledge (the one-line summary is kept)",
        |v| drop_in_each(v, &["first_difference", "seats"], "knowledge"),
    ),
    ("summary.stats (the header figures are kept)", |v| {
        drop_key(v, &["summary", "stats"])
    }),
    (
        "scenario.stats (the scenario-table figures are kept)",
        |v| drop_key(v, &["scenario", "stats"]),
    ),
    ("view.selected_case", |v| {
        drop_key(v, &["view", "selected_case"])
    }),
    (
        "first_difference.sides[].state (the summaries are kept)",
        |v| drop_in_each(v, &["first_difference", "sides"], "state"),
    ),
    (
        "first_difference.candidates beyond the first 12",
        |v| match v
            .get_mut("first_difference")
            .and_then(|f| f.get_mut("candidates"))
            .and_then(Value::as_array_mut)
        {
            Some(a) if a.len() > 12 => {
                a.truncate(12);
                true
            }
            _ => false,
        },
    ),
    (
        "board (board_figures and bba_reading carry the same)",
        |v| drop_key(v, &["board"]),
    ),
    ("bba_reading", |v| drop_key(v, &["bba_reading"])),
    ("knowledge[].text (ticket.md has it)", |v| {
        drop_in_each(v, &["knowledge"], "text")
    }),
    ("first_difference", |v| drop_key(v, &["first_difference"])),
    ("summary", |v| drop_key(v, &["summary"])),
    ("git.modified", |v| drop_key(v, &["git", "modified"])),
];

/// The context as pretty JSON of at most `budget` characters, dropping the
/// big arrays first. Returns the JSON and what was dropped.
pub fn trim_context(ctx: &Value, budget: usize) -> (String, Vec<String>) {
    let render = |v: &Value| serde_json::to_string_pretty(v).unwrap_or_default();
    let mut v = ctx.clone();
    let mut text = render(&v);
    let mut trimmed: Vec<String> = vec![];
    for (what, f) in TRIMS {
        if text.chars().count() <= budget {
            break;
        }
        if f(&mut v) {
            trimmed.push(what.to_string());
            v["_trimmed"] = json!(trimmed);
            text = render(&v);
        }
    }
    if text.chars().count() > budget {
        trimmed.push("everything (still too large)".into());
        text = render(&json!({ "_trimmed": trimmed }));
    }
    (text, trimmed)
}

/// An issue ready to file.
#[derive(Debug, Clone)]
pub struct IssuePlan {
    pub repo: String,
    pub title: String,
    pub labels: Vec<String>,
    pub body: String,
    /// What was dropped from context.json to fit.
    pub trimmed: Vec<String>,
}

/// The issue for a written ticket: ticket.md without frontmatter, then
/// context.json in a collapsed block, trimmed to fit GitHub's limit.
pub fn plan_issue(c: &Context, md: &str, repo: &str, local: &Path) -> IssuePlan {
    let text = strip_frontmatter(md).trim_end();
    let text = if text.chars().count() > BODY_BUDGET / 2 {
        format!(
            "{}\n\n_(ticket.md truncated; the full text is in the local copy)_",
            truncate(text, BODY_BUDGET / 2)
        )
    } else {
        text.to_string()
    };
    let text = format!("{text}\n\nLocal copy: `{}`", local.display());
    const OPEN: &str = "\n\n<details><summary>context.json</summary>\n\n```json\n";
    const CLOSE: &str = "\n```\n\n</details>\n";
    // Leave room for the note on what was trimmed.
    let budget = BODY_BUDGET.saturating_sub(text.chars().count() + OPEN.len() + CLOSE.len() + 1000);
    let value = serde_json::to_value(c).unwrap_or(Value::Null);
    let (json, trimmed) = trim_context(&value, budget);
    let note = if trimmed.is_empty() {
        String::new()
    } else {
        format!(
            "\n\n_context.json trimmed to fit GitHub's {GITHUB_BODY_LIMIT}-character limit; \
             dropped: {}. The local copy is complete._",
            trimmed.join("; ")
        )
    };
    let body = format!("{text}{note}{OPEN}{json}{CLOSE}");
    IssuePlan {
        repo: repo.to_string(),
        title: title(c),
        labels: c.kind.labels(),
        body,
        trimmed,
    }
}

/// `gh` arguments for the issue; the body goes on stdin.
pub fn issue_args(p: &IssuePlan) -> Vec<String> {
    let mut a: Vec<String> = ["issue", "create", "--repo", &p.repo, "--title", &p.title]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for l in &p.labels {
        a.push("--label".into());
        a.push(l.clone());
    }
    a.push("--body-file".into());
    a.push("-".into());
    a
}

fn run_gh(args: &[String], stdin: Option<&str>) -> Result<String, String> {
    let mut child = Command::new("gh")
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("gh: {e}"))?;
    if let Some(text) = stdin {
        if let Some(mut s) = child.stdin.take() {
            s.write_all(text.as_bytes())
                .map_err(|e| format!("gh stdin: {e}"))?;
        }
    }
    let out = child.wait_with_output().map_err(|e| format!("gh: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(format!(
            "gh {}: {}",
            args.first().map(String::as_str).unwrap_or(""),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn label_style(name: &str) -> (&'static str, &'static str) {
    match name {
        "workbench-ticket" => ("5319e7", "Filed from the rbb workbench"),
        "rules" => (
            "0e8a16",
            "A rule in conventions/ bids or reads a call wrongly",
        ),
        "bug" => ("d73a4a", "Something isn't working"),
        _ => ("a2eeef", "New feature or request"),
    }
}

/// Create the labels the repository lacks.
fn ensure_labels(repo: &str, labels: &[String]) -> Result<(), String> {
    let have = run_gh(
        &[
            "label", "list", "--repo", repo, "--limit", "1000", "--json", "name", "--jq",
            ".[].name",
        ]
        .map(String::from),
        None,
    )?;
    let have: Vec<&str> = have.lines().map(str::trim).collect();
    for l in labels {
        if have.iter().any(|h| h.eq_ignore_ascii_case(l)) {
            continue;
        }
        let (color, desc) = label_style(l);
        run_gh(
            &[
                "label",
                "create",
                l,
                "--repo",
                repo,
                "--color",
                color,
                "--description",
                desc,
            ]
            .map(String::from),
            None,
        )?;
    }
    Ok(())
}

pub enum Filed {
    Url(String),
    /// Nothing was filed; the command that would have run.
    DryRun(String),
}

pub fn dry_run_from_env() -> bool {
    std::env::var(DRY_RUN_ENV).is_ok_and(|v| !v.is_empty() && v != "0")
}

/// File the issue with `gh` (blocking: call it off the UI thread).
pub fn file_issue(p: &IssuePlan, dry_run: bool) -> Result<Filed, String> {
    let args = issue_args(p);
    let shown = format!(
        "gh {}",
        args.iter().map(|a| sh(a)).collect::<Vec<_>>().join(" ")
    );
    if dry_run {
        eprintln!(
            "[ticket dry run] would ensure labels {:?} on {}\n[ticket dry run] {shown} \
             <<< body ({} chars{})",
            p.labels,
            p.repo,
            p.body.chars().count(),
            if p.trimmed.is_empty() {
                String::new()
            } else {
                format!("; trimmed: {}", p.trimmed.join("; "))
            }
        );
        return Ok(Filed::DryRun(shown));
    }
    ensure_labels(&p.repo, &p.labels)?;
    let out = run_gh(&args, Some(&p.body))?;
    let url = out
        .lines()
        .rev()
        .find(|l| l.starts_with("http"))
        .unwrap_or(out.as_str())
        .to_string();
    Ok(Filed::Url(url))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_time() -> DateTime<Local> {
        use chrono::TimeZone;
        Local.with_ymd_and_hms(2026, 9, 27, 14, 3, 11).unwrap()
    }

    pub(super) fn settings(pbs: &str, rules: &str) -> Settings {
        Settings {
            scenarios: "Basic_Takeout_Double".into(),
            limit: "193".into(),
            par: false,
            auctions: "all".into(),
            auctions_label: AuctionFilter::All.label().into(),
            card_changes: vec![],
            rules: rules.into(),
            pbs: pbs.into(),
            cards: "cards/bbsa".into(),
            auto_rerun: true,
        }
    }

    fn bare(note: &str) -> Context {
        Context::build(Capture {
            kind: Kind::Bug,
            note: note.into(),
            created: fixed_time(),
            git: GitInfo {
                commit: "abc123".into(),
                branch: "main".into(),
                dirty: true,
                modified: vec![" M conventions/overcalls.bid".into()],
            },
            settings: settings("../Practice-Bidding-Scenarios", "conventions"),
            view: View {
                tab: "Divergences".into(),
                ..Default::default()
            },
            summary: None,
            scenario: None,
            board: None,
            knowledge: None,
        })
    }

    #[test]
    fn day_folders_are_numbered_in_order() {
        let day = std::env::temp_dir().join(format!("rbb-ticket-index-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&day);
        assert_eq!(next_index(&day), 1);
        std::fs::create_dir_all(day.join("01.resolved.a")).unwrap();
        std::fs::create_dir_all(day.join("02.open.b")).unwrap();
        std::fs::create_dir_all(day.join("notes")).unwrap();
        assert_eq!(next_index(&day), 3);
        let _ = std::fs::remove_dir_all(&day);
    }

    #[test]
    fn slug_takes_scenario_board_and_first_words() {
        assert_eq!(
            slug(
                Some("Basic_Takeout_Double"),
                Some("193"),
                "Why not double?\nMore text here"
            ),
            "basic-takeout-double-b193-why-not-double"
        );
        assert_eq!(slug(None, None, ""), "ticket");
        assert_eq!(
            slug(
                None,
                None,
                "  The engine's 2NT: wrong! And more words beyond five"
            ),
            "the-engine-s-2nt-wrong"
        );
        let long = slug(
            Some("A_Very_Long_Scenario_Name_That_Goes_On_And_On_Forever"),
            Some("1"),
            "note",
        );
        assert!(long.len() <= 60, "{long}");
        assert!(!long.ends_with('-'));
    }

    #[test]
    fn title_is_prefixed_and_truncated() {
        let c = bare(&"x".repeat(200));
        let t = title(&c);
        assert!(t.starts_with("Bug: "));
        assert_eq!(t.chars().count(), "Bug: ".len() + 80);
        assert!(t.ends_with('…'));
        assert_eq!(title(&bare("")), "Bug: workbench");
    }

    #[test]
    fn markdown_has_frontmatter_note_and_repro() {
        let c = bare("Line one | with a pipe\n\nSecond paragraph.");
        let md = render_markdown(&c);
        assert!(md.starts_with("---\nstatus: open\nkind: bug\n"));
        assert!(md.contains("\ncommit: abc123\n"));
        assert!(md.contains("\nissue: \nresolution: \nrefs: []\n---\n"));
        // The note is verbatim.
        assert!(md.contains("## Note\n\nLine one | with a pipe\n\nSecond paragraph.\n"));
        assert!(md.contains("rbb-cli -- compare 'Basic_Takeout_Double'"));
        assert!(md.contains("--limit 193"));
        assert!(md.contains("`conventions/overcalls.bid`") || md.contains("M conventions"));
        let body = strip_frontmatter(&md);
        assert!(body.starts_with("# Bug: Line one"));
        let with = set_frontmatter(&md, "issue", "https://github.com/o/r/issues/7");
        assert!(with.contains("\nissue: https://github.com/o/r/issues/7\n"));
        assert_eq!(strip_frontmatter(&with), body);
    }

    #[test]
    fn table_cells_escape_pipes() {
        let t = table(&["a", "b"], &[vec!["x|y".into(), "1\n2".into()]]);
        assert_eq!(t, "| a | b |\n|---|---|\n| x\\|y | 1 2 |\n");
    }

    #[test]
    fn repo_from_remote_urls() {
        assert_eq!(
            parse_repo("git@github.com:bridge-craftwork/rusty-bidding-bot.git").as_deref(),
            Some("bridge-craftwork/rusty-bidding-bot")
        );
        assert_eq!(
            parse_repo("https://github.com/bridge-craftwork/rusty-bidding-bot").as_deref(),
            Some("bridge-craftwork/rusty-bidding-bot")
        );
        assert_eq!(parse_repo("https://gitlab.com/a/b"), None);
    }

    #[test]
    fn small_context_is_not_trimmed() {
        let v = json!({"board": {"x": 1}, "bba_reading": [{"knowledge": {"a": 1}}]});
        let (text, trimmed) = trim_context(&v, 10_000);
        assert!(trimmed.is_empty());
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), v);
    }

    #[test]
    fn trimming_drops_big_arrays_first_and_fits() {
        let big = "k".repeat(2000);
        let rows: Vec<Value> = (0..40)
            .map(|i| json!({"n": i, "knowledge_summary": "12-14 HCP", "knowledge": {"shown": [big]}}))
            .collect();
        let v = json!({
            "schema_version": 1,
            "board": {"deal": "N:..."},
            "bba_reading": rows,
            "first_difference": {"seats": [{"seat": "N", "knowledge": {"shown": [big]}}]},
        });
        let (text, trimmed) = trim_context(&v, 20_000);
        assert!(text.chars().count() <= 20_000);
        assert_eq!(
            trimmed,
            vec!["bba_reading[].knowledge (the one-line knowledge_summary is kept)"]
        );
        let back: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(back["bba_reading"][3]["knowledge_summary"], "12-14 HCP");
        assert!(back["bba_reading"][3].get("knowledge").is_none());
        assert_eq!(back["_trimmed"][0], trimmed[0].as_str());
        // A tiny budget drops sections, and still returns JSON.
        let (text, trimmed) = trim_context(&v, 300);
        assert!(text.chars().count() <= 300, "{text}");
        assert!(trimmed.len() > 2);
        serde_json::from_str::<Value>(&text).unwrap();
    }

    #[test]
    fn issue_body_fits_and_args_read_stdin() {
        let mut c = bare("Some bug");
        c.view.divergence_filter = "q".repeat(100_000);
        let md = render_markdown(&c);
        let p = plan_issue(&c, &md, "o/r", Path::new("/tmp/t"));
        assert!(p.body.chars().count() < GITHUB_BODY_LIMIT);
        assert!(!p.body.starts_with("---"));
        assert!(p.body.contains("<details><summary>context.json</summary>"));
        assert!(!p.trimmed.is_empty());
        assert_eq!(p.labels, vec!["workbench-ticket", "bug"]);
        let args = issue_args(&p);
        assert_eq!(
            &args[..6],
            [
                "issue",
                "create",
                "--repo",
                "o/r",
                "--title",
                "Bug: Some bug"
            ]
        );
        assert_eq!(&args[args.len() - 2..], ["--body-file", "-"]);
        // The dry run files nothing.
        assert!(matches!(file_issue(&p, true), Ok(Filed::DryRun(_))));
    }
}

/// The local sink end to end, on a real board: compare one scenario, pick
/// Basic_Takeout_Double board 193, and write its ticket. Skipped when the
/// Practice-Bidding-Scenarios checkout is not there (`RBB_PBS` overrides
/// where to look). Set `RBB_TICKET_SAMPLE_DIR` to keep the ticket
/// somewhere to read.
#[cfg(test)]
mod end_to_end {
    use super::tests::settings;
    use super::*;
    use rbb_compare::Engines;

    #[test]
    fn local_ticket_for_basic_takeout_double_193() {
        // The Practice-Bidding-Scenarios checkout beside this repo, or RBB_PBS.
        let pbs = std::env::var("RBB_PBS")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../Practice-Bidding-Scenarios")
            });
        if !pbs.join("bba").is_dir() {
            eprintln!(
                "skipped: no Practice-Bidding-Scenarios at {}",
                pbs.display()
            );
            return;
        }
        let here = Path::new(env!("CARGO_MANIFEST_DIR"));
        let rules = here.join("../../conventions");
        let opts = rbb_compare::Options {
            pbs: pbs.clone(),
            scenarios: vec!["Basic_Takeout_Double".into()],
            limit: Some(193),
            rules: rules.clone(),
            par: false,
            dd_cache: std::env::temp_dir().join("rbb-workbench-ticket-dd.jsonl"),
            card_changes: vec![],
        };
        let engines = Engines::for_options(&opts).unwrap();
        let report = rbb_compare::run_with(&opts, &engines, &|_, _| {}).unwrap();
        let b = report
            .boards
            .iter()
            .find(|b| b.board == "193")
            .expect("board 193");
        let detail = Detail::compute(b, &engines);
        let (total, scenarios) = rbb_compare::tally(report.boards.iter());
        let c = Context::build(Capture {
            kind: Kind::Rule,
            note: "West's double: should it be takeout here?\nSecond line of the note.".into(),
            created: Local::now(),
            git: GitInfo::collect(here),
            settings: settings(&pbs.display().to_string(), &rules.display().to_string()),
            view: View {
                tab: "Boards".into(),
                selected_scenario: Some("Basic_Takeout_Double".into()),
                ..Default::default()
            },
            summary: Some(Summary::new(&total, 1.0, None)),
            scenario: scenarios.first().map(ScenarioFigures::new),
            board: Some((b, &detail)),
            knowledge: Some(false),
        });
        let root = std::env::var("RBB_TICKET_SAMPLE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("rbb-ticket-e2e"));
        let dir = write_local(&root, &c).unwrap();
        let md = std::fs::read_to_string(dir.join("ticket.md")).unwrap();
        let json: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("context.json")).unwrap())
                .unwrap();
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let (index, rest) = name.split_once('.').unwrap();
        assert!(index.len() == 2 && index.parse::<u32>().is_ok(), "{name}");
        assert!(
            rest.starts_with("open.basic-takeout-double-b193-west-s-double"),
            "{name}"
        );
        assert!(md.contains("scenario: Basic_Takeout_Double\nboard: 193\n"));
        assert!(md.contains("## How the engine read BBA's auction"));
        assert!(md.contains("## Knowledge view (our auction)\n\n```text\n 1. "));
        assert!(md.contains(
            "## Knowledge view (BBA's auction, as the engine reads it)\n\n```text\n 1. "
        ));
        assert_eq!(json["knowledge"][0]["auction"], "ours");
        assert_eq!(json["knowledge"][1]["auction"], "bba");
        assert!(json["knowledge"][0]["text"]
            .as_str()
            .unwrap()
            .contains(" 1. "));
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["board"]["board"], "193");
        let steps = json["bba_reading"].as_array().unwrap();
        assert_eq!(steps.len(), b.reference.len());
        assert_eq!(json["board_figures"]["hands"].as_array().unwrap().len(), 4);
        if let Some(i) = b.first_divergence {
            assert!(md.contains("rbb-cli -- call "));
            assert!(!json["first_difference"]["candidates"]
                .as_array()
                .unwrap()
                .is_empty());
            assert_eq!(json["first_difference"]["our_call"], short(&b.replay[i]));
        }
        let plan = plan_issue(&c, &md, "bridge-craftwork/rusty-bidding-bot", &dir);
        assert!(plan.body.chars().count() < GITHUB_BODY_LIMIT);
        assert!(plan.title.starts_with("Rule: West's double"));
        assert!(matches!(file_issue(&plan, true), Ok(Filed::DryRun(_))));
        eprintln!("ticket written to {}", dir.display());
    }
}
