//! Comparing one board: replay the reference auction through the engine, then
//! let the engine finish the auction on its own from the first difference.

use bridge_types::{Auction, Board, Call, Direction, FinalContract, Vulnerability};
use rbb_engine::Engine;
use serde::Serialize;

/// Longest auction the engine may produce before we stop it.
const MAX_CALLS: usize = 60;

#[derive(Debug, Clone, Serialize)]
pub struct ParComparison {
    pub par_ns: i32,
    pub par_contract: String,
    pub reference_ns: i32,
    pub ours_ns: i32,
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
    pub first_divergence: Option<usize>,
    pub reference_contract: Option<String>,
    pub our_contract: Option<String>,
    pub par: Option<ParComparison>,
    /// The engine stopped its auction at `MAX_CALLS`.
    pub runaway: bool,
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

    let mut pos = engine.start(dealer, board.vulnerable);
    let mut replay = Vec::with_capacity(reference.len());
    let mut at_divergence = None;
    for (i, call) in reference.iter().enumerate() {
        let before = pos.clone();
        let (choice, _) = engine.step(&mut pos, hand(before.next_caller()), call);
        if at_divergence.is_none() && &choice.call != call {
            at_divergence = Some((i, before));
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
            let choice = engine.choose(&p, hand(p.next_caller()));
            engine.advance(&mut p, &choice.call);
            ours.push(choice.call);
        }
    }

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
        deal: board.deal.to_pbn(Direction::North),
        reference_alerts: (0..auction.calls.len())
            .map(|i| alert_text(auction, i))
            .collect(),
        reference_contract: contract_text(contract_of(&reference).as_ref()),
        our_contract: contract_text(contract_of(&ours).as_ref()),
        reference,
        replay,
        ours,
        first_divergence,
        par: None,
        runaway,
    })
}

/// Fill in `result.par` from double-dummy analysis.
pub fn add_par(result: &mut BoardResult, board: &Board, cache: &crate::par::DdCache) {
    let dd = cache.table(&board.deal);
    let contract = |calls: &[Call]| {
        let mut a = Auction::new(result.dealer);
        for c in calls {
            a.add_call(c.clone());
        }
        a.final_contract()
    };
    let (par_ns, par_contract) = crate::par::par_ns(&dd, result.vul);
    result.par = Some(ParComparison {
        par_ns,
        par_contract,
        reference_ns: crate::par::score_ns(contract(&result.reference).as_ref(), &dd, result.vul),
        ours_ns: crate::par::score_ns(contract(&result.ours).as_ref(), &dd, result.vul),
    });
}
