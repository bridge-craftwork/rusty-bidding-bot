//! Alerts by regulator (`Engine::with_regulator`, the pack's
//! `alerts.toml`): where an entry is written for the call, its kind for
//! the regulator is what the table is told; elsewhere the rule's flag. The
//! bidding never depends on it.

use std::path::Path;
use std::sync::OnceLock;

use bidspec::ast::Alert;
use bridge_card::Card;
use bridge_types::{Call, Deal, Direction, ScoringMethod, Vulnerability};
use rbb_engine::{Engine, Regulator, RuleSet};

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rules() -> &'static RuleSet {
    static RULES: OnceLock<RuleSet> = OnceLock::new();
    RULES.get_or_init(|| rbb_engine::load_rules(&root().join("pack/pack.json")).unwrap())
}

/// SAYC, with Stayman marked forcing (the pack's Stayman entries named that
/// treatment until bridge-common-conventions#136; now any Stayman).
fn card() -> Card {
    let mut c = plain_card();
    c.apply_change("bidding_conventions/stayman:forcing=true")
        .unwrap();
    c
}

fn plain_card() -> Card {
    let text = std::fs::read_to_string(root().join("cards/json/SAYC.json")).unwrap();
    Card::from_json(&rules().vocab, &text).unwrap().0
}

fn calls(s: &str) -> Vec<Call> {
    s.split_whitespace()
        .map(|c| Call::from_pbn(c).unwrap())
        .collect()
}

/// The alert of the last call of `auction`, as `regulator`'s table hears it.
fn alert(regulator: Option<Regulator>, dealer: Direction, auction: &str) -> Option<Alert> {
    let c = card();
    let mut e = Engine::new(&c, &c, rules());
    if let Some(r) = regulator {
        e = e.with_regulator(r);
    }
    let interp = e.interpret(
        dealer,
        Vulnerability::None,
        ScoringMethod::IMP,
        &calls(auction),
    );
    interp.steps.last().unwrap().alert.clone()
}

fn kind(a: &Option<Alert>) -> &'static str {
    match a {
        None => "none",
        Some(Alert::Alert { delayed: true, .. }) => "delayed",
        Some(Alert::Alert { .. }) => "alert",
        Some(Alert::Announce { .. }) => "announce",
    }
}

#[test]
fn the_snapshot_brings_its_alerts() {
    assert!(!rules().alerts.is_empty());
    assert_eq!(
        Engine::new(&card(), &card(), rules()).regulator(),
        Regulator::Acbl
    );
}

#[test]
fn stayman_is_exempt_announced_or_alerted_by_regulator() {
    let n = Direction::North;
    // The ACBL exempts Stayman; the EBU announces it; the WBF alerts it.
    assert_eq!(kind(&alert(None, n, "1NT Pass 2C")), "none");
    assert_eq!(
        kind(&alert(Some(Regulator::Acbl), n, "1NT Pass 2C")),
        "none"
    );
    assert_eq!(
        kind(&alert(Some(Regulator::Ebu), n, "1NT Pass 2C")),
        "announce"
    );
    assert_eq!(
        kind(&alert(Some(Regulator::Wbf), n, "1NT Pass 2C")),
        "alert"
    );
    // Passes before the opening do not matter.
    assert_eq!(
        kind(&alert(Some(Regulator::Wbf), n, "Pass Pass 1NT Pass 2C")),
        "alert"
    );
    assert_eq!(
        kind(&alert(Some(Regulator::Acbl), n, "Pass 1NT Pass 2C")),
        "none"
    );
}

/// The Stayman entries name the convention (bridge-common-conventions#136,
/// Rick 2026-11-20), so they apply to a card that plays Stayman without
/// marking it forcing: exempt at an ACBL table.
#[test]
fn stayman_entries_apply_to_any_stayman() {
    let c = plain_card();
    let e = Engine::new(&c, &c, rules()).with_regulator(Regulator::Acbl);
    let steps = e
        .interpret(
            Direction::North,
            Vulnerability::None,
            ScoringMethod::IMP,
            &calls("1NT Pass 2C"),
        )
        .steps;
    assert_eq!(kind(&steps[2].alert), "none");
}

#[test]
fn transfers_are_announced_with_the_rules_text() {
    let a = alert(Some(Regulator::Acbl), Direction::North, "1NT Pass 2D");
    let Some(Alert::Announce { text }) = &a else {
        panic!("{a:?}")
    };
    assert!(!text.is_empty());
    assert_eq!(
        kind(&alert(
            Some(Regulator::Wbf),
            Direction::North,
            "1NT Pass 2D"
        )),
        "alert"
    );
}

#[test]
fn an_auction_no_entry_names_keeps_the_rules_flag() {
    // After their double no entry is written for 2♣: the rule's flag.
    let c = card();
    let flags = Engine::new(&c, &c, rules());
    let interp = flags.interpret(
        Direction::North,
        Vulnerability::None,
        ScoringMethod::IMP,
        &calls("1NT X 2C"),
    );
    let rule_flag = interp.steps.last().unwrap().alert.clone();
    for r in Regulator::ALL {
        assert_eq!(
            alert(Some(r), Direction::North, "1NT X 2C"),
            rule_flag,
            "{r}"
        );
    }
}

#[test]
fn the_bidding_does_not_depend_on_the_regulator() {
    let deals = [
        "N:AK52.KJ7.Q94.K83 QJ3.T98.KJ5.QJ92 T984.AQ52.A62.A4 76.643.T873.T765",
        "N:KQ7.AJ5.KQ84.Q93 J632.K874.95.J52 A984.Q932.AJ7.A4 T5.T6.T632.KT876",
        "N:A2.KQJ84.A93.K82 KJT9.62.QJ85.QT6 Q854.A975.K4.A73 763.T3.T762.J954",
    ];
    let c = card();
    let auctions: Vec<Vec<Vec<String>>> = Regulator::ALL
        .iter()
        .map(|&r| {
            let e = Engine::new(&c, &c, rules()).with_regulator(r);
            deals
                .iter()
                .map(|d| {
                    let deal = Deal::from_pbn(d).unwrap();
                    let hands = Direction::ALL.map(|s| deal.hand(s).clone());
                    e.bid_deal(
                        &hands,
                        Direction::North,
                        Vulnerability::None,
                        ScoringMethod::IMP,
                        &[],
                        40,
                    )
                    .unwrap()
                    .calls
                    .iter()
                    .map(|c| c.step.call.to_pbn())
                    .collect()
                })
                .collect()
        })
        .collect();
    assert_eq!(auctions[0], auctions[1]);
    assert_eq!(auctions[0], auctions[2]);
}
