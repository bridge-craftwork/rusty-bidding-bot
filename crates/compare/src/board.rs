//! Comparing one board: replay the reference auction through the engine, then
//! let the engine finish the auction on its own from the first difference.

use bridge_types::{
    Auction, Board, Call, DdTable, Direction, FinalContract, ScoringMethod, Vulnerability,
};
use rbb_engine::Engine;
use serde::Serialize;

/// Longest auction the engine may produce before we stop it.
const MAX_CALLS: usize = 60;

/// Something that is wrong whatever the convention: not a matter of
/// judgment, and independent of BBA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemKind {
    /// The engine had no rule, in an auction its side had already entered.
    NoRule,
    /// The auction ended in an artificial call (a keycard answer, a transfer).
    ArtificialContract,
    /// The contract is in a suit where the declaring side has fewer than
    /// seven cards between them.
    ShortFit,
    /// A call contradicted what the same player had shown.
    Contradiction,
    /// A pass where our own rules said the auction was forcing: partner's
    /// call was forcing for one round, or we were in a game force below
    /// game. The engine only does this when no rule matched at all.
    BrokenForce,
    /// The engine did not finish the auction.
    Runaway,
}

impl ProblemKind {
    pub fn label(self) -> &'static str {
        match self {
            ProblemKind::NoRule => "no rule in a live auction",
            ProblemKind::ArtificialContract => "contract is an artificial call",
            ProblemKind::ShortFit => "trump fit under 7 cards",
            ProblemKind::Contradiction => "contradicts earlier calls",
            ProblemKind::BrokenForce => "passed a forcing auction",
            ProblemKind::Runaway => "auction not finished",
        }
    }
}

/// One problem in the engine's own auction.
#[derive(Debug, Clone, Serialize)]
pub struct Problem {
    pub kind: ProblemKind,
    /// The call it concerns, as an index into `BoardResult::ours`.
    pub index: usize,
    pub detail: String,
}

/// One of our own calls that partner reads by a higher-priority rule than
/// the one that chose it: the engine reads a call as the highest-priority
/// rules for it whose public conditions hold, so a lower-priority
/// (judgment, fallback) rule choosing a call a higher rule also offers is
/// read as the higher rule (docs/JUDGMENT-LAYER.md, "Reading partner's judgment call").
#[derive(Debug, Clone, Serialize)]
pub struct ReadAs {
    /// The call, as an index into `BoardResult::ours`.
    pub index: usize,
    pub call: String,
    /// The rule that chose the call, `file:line`, and its priority.
    pub chosen: String,
    pub chosen_priority: Option<i64>,
    /// The rule it was read as, `file:line` (None: no rule read it), and
    /// its priority.
    pub read: Option<String>,
    pub read_priority: Option<i64>,
}

fn read_as(index: usize, choice: &rbb_engine::Choice, step: &rbb_engine::Step) -> Option<ReadAs> {
    let chosen = choice.rule.as_ref()?;
    if step.rule.as_ref() == Some(chosen) {
        return None;
    }
    let priority = |r: &rbb_engine::RuleRef| {
        choice
            .candidates
            .iter()
            .find(|c| &c.rule == r && c.call == choice.call)
            .map(|c| c.priority)
    };
    let chosen_priority = priority(chosen);
    let read_priority = step.rule.as_ref().and_then(priority);
    // A rule of the same priority is read together with the chosen one
    // (the call shows their union), so only a higher rule misreads it.
    if let (Some(c), Some(r)) = (chosen_priority, read_priority) {
        if r <= c {
            return None;
        }
    }
    let at = |r: &rbb_engine::RuleRef| format!("{}:{}", r.file, r.line);
    Some(ReadAs {
        index,
        call: crate::report::short(&choice.call),
        chosen: at(chosen),
        chosen_priority,
        read: step.rule.as_ref().map(at),
        read_priority,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ParComparison {
    pub par_ns: i32,
    pub par_contract: String,
    pub reference_ns: i32,
    pub ours_ns: i32,
    /// Each table's errors by side, with an overbid taken as doubled
    /// (`par::table_errors`).
    pub reference_errors: crate::par::TableErrors,
    pub ours_errors: crate::par::TableErrors,
}

/// Everything about one board, for statistics and for the A/B view.
#[derive(Debug, Clone, Serialize)]
pub struct BoardResult {
    pub scenario: String,
    /// The cards each side played (names of `bbsa/<name>.bbsa`).
    pub ns_card: String,
    pub ew_card: String,
    pub board: String,
    pub dealer: Direction,
    pub vul: Vulnerability,
    /// The board's `[Scoring]` tag; `None` when the file does not record it
    /// (the engine then assumes matchpoints, bba-cli's default).
    pub scoring: Option<ScoringMethod>,
    /// What produced the reference auction (see `Scenario::generator`).
    pub generator: String,
    /// PBN deal, North first.
    pub deal: String,
    /// The reference auction and its alert texts (`[Note]`s).
    pub reference: Vec<Call>,
    pub reference_alerts: Vec<Option<String>>,
    /// The engine's call at each position of the reference auction.
    pub replay: Vec<Call>,
    /// The engine's own auction: the reference up to the first difference,
    /// then the engine's calls to the end.
    pub ours: Vec<Call>,
    /// For each of our calls, the rule that gave it its meaning, as
    /// `file:line explanation` (None where no rule did). Our own calls
    /// are read by the rule that chose them. For par attribution
    /// (`probes/tools/par_blame.py`).
    #[serde(default)]
    pub our_rules: Vec<Option<String>>,
    /// Our own calls read by a higher rule than the one that chose them
    /// (see `ReadAs`).
    #[serde(default)]
    pub read_as: Vec<ReadAs>,
    pub first_divergence: Option<usize>,
    pub reference_contract: Option<String>,
    pub our_contract: Option<String>,
    /// The deal's double-dummy table, when the reference file carried one
    /// (PBN `OptimumResultTable`). Par then costs nothing: no solving, and
    /// every board can be scored against it, not just the slow `--par` run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dd: Option<DdTable>,
    pub par: Option<ParComparison>,
    /// The engine stopped its auction at `MAX_CALLS`.
    pub runaway: bool,
    /// Problems in the engine's own auction (see `ProblemKind`).
    pub problems: Vec<Problem>,
}

impl BoardResult {
    pub fn contracts_match(&self) -> bool {
        self.reference_contract == self.our_contract
    }
}

pub fn contract_text(c: Option<&FinalContract>) -> Option<String> {
    c.map(|c| format!("{} {}", c.to_pbn(), c.declarer.to_char()))
}

fn alert_text(auction: &Auction, i: usize) -> Option<String> {
    let ann = auction.calls[i].annotation.as_deref()?;
    let n: u8 = ann.trim_matches('=').parse().ok()?;
    auction.get_note(n).map(str::to_string)
}

/// Compare the engine with the reference auction on `board`. `None` when the
/// board has no auction or dealer.
pub fn compare(engine: &Engine, scenario: &str, board: &Board) -> Option<BoardResult> {
    let auction = board.auction.as_ref()?;
    let dealer = board.dealer.unwrap_or(auction.dealer);
    let reference: Vec<Call> = auction.calls.iter().map(|c| c.call.clone()).collect();
    if reference
        .iter()
        .any(|c| matches!(c, Call::Continue | Call::Blank))
    {
        return None;
    }
    let hand = |d: Direction| board.deal.hand(d);

    let scoring = board.extra_tag("Scoring").and_then(ScoringMethod::from_pbn);
    let mut pos = engine.start(dealer, board.vulnerable, scoring.unwrap_or_default());
    let mut replay = Vec::with_capacity(reference.len());
    // For our own auction: at each call, whether a rule chose it, and how the
    // engine read it. Before the first difference our auction is BBA's.
    let mut decided: Vec<bool> = Vec::new();
    let mut steps: Vec<rbb_engine::Step> = Vec::new();
    let mut acted = [false; 2];
    let mut live: Vec<bool> = Vec::new();
    // Whether the caller's side was forced to bid at each call of our
    // auction: partner's call forcing for a round, or a game force below
    // game. The engine will not choose a pass there, so a pass means no
    // rule matched and it fell back to one.
    let mut forced: Vec<bool> = Vec::new();
    let mut at_divergence = None;
    let mut read_as_list: Vec<ReadAs> = Vec::new();
    for (i, call) in reference.iter().enumerate() {
        let before = pos.clone();
        let seat = before.next_caller();
        let was_forced = is_forced(&before, seat);
        let (choice, step) = engine.step(&mut pos, hand(seat), call);
        if at_divergence.is_none() && &choice.call != call {
            at_divergence = Some((i, before));
        }
        if at_divergence.is_none() {
            let side = rbb_engine::side(seat);
            live.push(acted[side]);
            acted[side] |= !call.is_pass();
            decided.push(choice.rule.is_some());
            forced.push(was_forced);
            // Our engine chose this call itself (it agrees with BBA's).
            read_as_list.extend(read_as(i, &choice, &step));
            steps.push(step);
        }
        replay.push(choice.call);
    }

    let mut ours = reference.clone();
    let mut runaway = false;
    let first_divergence = at_divergence.as_ref().map(|(i, _)| *i);
    if let Some((i, mut p)) = at_divergence {
        ours.truncate(i);
        while !p.auction().is_complete() {
            if ours.len() >= MAX_CALLS {
                runaway = true;
                break;
            }
            let seat = p.next_caller();
            let choice = engine.choose(&p, hand(seat));
            let side = rbb_engine::side(seat);
            live.push(acted[side]);
            acted[side] |= !choice.call.is_pass();
            decided.push(choice.rule.is_some());
            forced.push(is_forced(&p, seat));
            let step = engine.advance(&mut p, &choice.call);
            read_as_list.extend(read_as(ours.len(), &choice, &step));
            steps.push(step);
            ours.push(choice.call);
        }
    }
    let problems = find_problems(
        board, dealer, &ours, &decided, &live, &forced, &steps, runaway,
    );
    let our_rules = steps
        .iter()
        .map(|s| {
            s.rule.as_ref().map(|r| {
                format!(
                    "{}:{} {}",
                    r.file,
                    r.line,
                    s.explanation.as_deref().unwrap_or("")
                )
            })
        })
        .collect();

    let contract_of = |calls: &[Call]| {
        let mut a = Auction::new(dealer);
        for c in calls {
            a.add_call(c.clone());
        }
        a.final_contract()
    };
    Some(BoardResult {
        scenario: scenario.to_string(),
        ns_card: String::new(),
        ew_card: String::new(),
        board: board
            .board_id
            .clone()
            .or(board.number.map(|n| n.to_string()))
            .unwrap_or_default(),
        dealer,
        vul: board.vulnerable,
        scoring,
        generator: String::new(),
        deal: board.deal.to_pbn(Direction::North),
        reference_alerts: (0..auction.calls.len())
            .map(|i| alert_text(auction, i))
            .collect(),
        reference_contract: contract_text(contract_of(&reference).as_ref()),
        our_contract: contract_text(contract_of(&ours).as_ref()),
        dd: board.double_dummy_tricks.filter(|t| !t.is_null()),
        reference,
        replay,
        ours,
        our_rules,
        read_as: read_as_list,
        first_divergence,
        par: None,
        runaway,
        problems,
    })
}

/// Must this seat bid? The engine's own rule (`Position::must_bid`):
/// partner's call was forcing for a round and RHO passed, or the side is in
/// a game force and the auction is below game.
fn is_forced(pos: &rbb_engine::Position, seat: Direction) -> bool {
    pos.must_bid(seat)
}

#[allow(clippy::too_many_arguments)]
fn find_problems(
    board: &Board,
    dealer: Direction,
    ours: &[Call],
    decided: &[bool],
    live: &[bool],
    forced: &[bool],
    steps: &[rbb_engine::Step],
    runaway: bool,
) -> Vec<Problem> {
    let mut out = Vec::new();
    let seat_of = |i: usize| (0..i).fold(dealer, |d, _| d.next());
    for i in 0..ours.len().min(decided.len()).min(live.len()) {
        if !decided[i] && live[i] {
            out.push(Problem {
                kind: ProblemKind::NoRule,
                index: i,
                detail: format!("{} had no rule", seat_of(i).to_char()),
            });
        }
        if ours[i].is_pass() && forced.get(i).copied().unwrap_or(false) {
            out.push(Problem {
                kind: ProblemKind::BrokenForce,
                index: i,
                detail: format!("{} passed a forcing auction", seat_of(i).to_char()),
            });
        }
    }
    for (i, s) in steps.iter().enumerate() {
        for w in s.warnings.iter().filter(|w| w.contains("contradicts")) {
            out.push(Problem {
                kind: ProblemKind::Contradiction,
                index: i,
                detail: w.clone(),
            });
        }
    }
    // The call that set the contract.
    if let Some(j) = ours.iter().rposition(Call::is_bid) {
        if let (Some(step), true) = (steps.get(j), !runaway) {
            let fit8 = match ours[j] {
                Call::Bid { strain, .. } => rbb_engine::suit_of(strain).is_some_and(|suit| {
                    let s = rbb_engine::side(seat_of(j));
                    [
                        Direction::North,
                        Direction::East,
                        Direction::South,
                        Direction::West,
                    ]
                    .into_iter()
                    .filter(|d| rbb_engine::side(*d) == s)
                    .map(|d| board.deal.hand(d).suit_length(suit))
                    .sum::<usize>()
                        >= 8
                }),
                _ => false,
            };
            // An artificial call that happens to name our real fit (a keycard
            // answer in the trump suit) is a fine place to stop.
            if step.artificial && !fit8 {
                out.push(Problem {
                    kind: ProblemKind::ArtificialContract,
                    index: j,
                    detail: format!(
                        "{} ({}) was passed out",
                        ours[j].to_pbn(),
                        step.explanation.as_deref().unwrap_or("artificial")
                    ),
                });
            }
        }
        // A one-level contract is an opening or response passed out: normal.
        if let Call::Bid { level: 2.., strain } = ours[j] {
            if let Some(suit) = rbb_engine::suit_of(strain) {
                let declarer_side = rbb_engine::side(seat_of(j));
                let fit: usize = [
                    Direction::North,
                    Direction::East,
                    Direction::South,
                    Direction::West,
                ]
                .into_iter()
                .filter(|d| rbb_engine::side(*d) == declarer_side)
                .map(|d| board.deal.hand(d).suit_length(suit))
                .sum();
                if fit < 7 {
                    out.push(Problem {
                        kind: ProblemKind::ShortFit,
                        index: j,
                        detail: format!(
                            "{}: {fit} cards between the declaring pair",
                            ours[j].to_pbn()
                        ),
                    });
                }
            }
        }
    }
    if runaway {
        out.push(Problem {
            kind: ProblemKind::Runaway,
            index: ours.len().saturating_sub(1),
            detail: "no end".into(),
        });
    }
    out
}

/// Fill in `result.par` from double-dummy analysis: from the board's own
/// `OptimumResultTable` when it has one, else by solving the deal.
pub fn add_par(result: &mut BoardResult, board: &Board, cache: &crate::par::DdCache) {
    result.par = Some(par_of(result, &board.deal, cache));
}

/// Par for a result on its own, from the deal it records (the workbench
/// solves a board on demand when the run skipped par), with the table it
/// used so the caller can keep it. None if the deal does not parse.
pub fn par_for(
    result: &BoardResult,
    cache: &crate::par::DdCache,
) -> Option<(ParComparison, DdTable)> {
    let deal = bridge_types::Deal::from_pbn(&result.deal)?;
    let dd = result.dd.unwrap_or_else(|| cache.table(&deal));
    Some((par_of(result, &deal, cache), dd))
}

fn par_of(
    result: &BoardResult,
    deal: &bridge_types::Deal,
    cache: &crate::par::DdCache,
) -> ParComparison {
    // The file's own table if it has one; otherwise solve (and cache) it.
    let dd = result.dd.unwrap_or_else(|| cache.table(deal));
    let contract = |calls: &[Call]| {
        let mut a = Auction::new(result.dealer);
        for c in calls {
            a.add_call(c.clone());
        }
        a.final_contract()
    };
    let (par_ns, par_contract) = crate::par::par_ns(&dd, result.vul);
    let (reference, ours) = (contract(&result.reference), contract(&result.ours));
    ParComparison {
        par_ns,
        par_contract,
        reference_ns: crate::par::score_ns(reference.as_ref(), &dd, result.vul),
        ours_ns: crate::par::score_ns(ours.as_ref(), &dd, result.vul),
        reference_errors: crate::par::table_errors(reference.as_ref(), &dd, result.vul, par_ns),
        ours_errors: crate::par::table_errors(ours.as_ref(), &dd, result.vul, par_ns),
    }
}
