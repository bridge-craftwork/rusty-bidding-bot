//! `Engine::auction`: a practice table where some seats are bots, with a
//! stop where no rule applies so the caller can take another bidder's call
//! there and resume.

use std::path::Path;
use std::sync::OnceLock;

use bridge_card::bbsa;
use bridge_types::{Call, Direction, Hand, ScoringMethod, Vulnerability};
use rbb_engine::{Engine, OnNoRule, StopReason, Table};

fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let text = std::fs::read_to_string(
            root.join("crates/bridge-card/tests/fixtures/bbsa/21GF-DEFAULT.bbsa"),
        )
        .unwrap();
        let rules = rbb_engine::load_rules(&root.join("conventions")).unwrap();
        let (card, _) = bbsa::import(&rules.vocab, &text, None).unwrap();
        Engine::new(&card, &card, &rules)
    })
}

fn calls(s: &str) -> Vec<Call> {
    s.split_whitespace()
        .map(|c| Call::from_pbn(c).unwrap())
        .collect()
}

fn hand(s: &str) -> Option<Hand> {
    Some(Hand::from_pbn(s).unwrap())
}

const N: &str = "987.64.T8765.A95";
const E: &str = "T64.AT832.A2.T62";
const S: &str = "AK52.KJ7.Q94.K83";
const W: &str = "QJ3.Q95.KJ3.QJ74";

/// North deals; North and East are human, South and West bots.
fn table() -> Table {
    Table {
        dealer: Direction::North,
        vul: Vulnerability::None,
        scoring: ScoringMethod::IMP,
        hands: [None, None, hand(S), hand(W)],
        bots: [false, false, true, true],
    }
}

#[test]
fn stops_where_no_rule_applies_and_resumes_after_a_call_from_outside() {
    let e = engine();
    // No rule for South's hand after 1C (4NT): stop before South's call.
    let t = e
        .auction(&table(), &calls("1C 4NT"), OnNoRule::Stop)
        .unwrap();
    assert_eq!(t.stop.reason, StopReason::NoRule);
    assert_eq!(t.stop.seat, Some(Direction::South));
    assert_eq!(t.stop.index, 2);
    assert_eq!(t.calls.len(), 2);
    assert_eq!(t.given, 2);
    let choice = t.stop.choice.as_ref().unwrap();
    assert!(choice.rule.is_none());
    assert_eq!(choice.call, Call::Pass);

    // The fallback bidder's pass is appended; West bids, then North (a
    // human) is to call.
    let t = e
        .auction(&table(), &calls("1C 4NT Pass"), OnNoRule::Stop)
        .unwrap();
    assert_eq!(t.stop.reason, StopReason::HumanToCall);
    assert_eq!(t.stop.seat, Some(Direction::North));
    assert_eq!(t.stop.index, 4);
    assert!(t.stop.choice.is_none());
    assert_eq!(t.given, 3);
    assert!(t.calls[2].choice.is_none(), "the pass was given");
    let west = t.calls[3].choice.as_ref().unwrap();
    assert!(west.rule.is_some());
    assert_eq!(t.calls[3].step.caller, Direction::West);

    // Every call is read as `interpret` reads it, given or made.
    let played: Vec<Call> = t.calls.iter().map(|c| c.step.call.clone()).collect();
    let i = e.interpret(
        Direction::North,
        Vulnerability::None,
        ScoringMethod::IMP,
        &played,
    );
    for (a, b) in t.calls.iter().zip(&i.steps) {
        assert_eq!(a.step.explanation, b.explanation);
        assert_eq!(a.step.rule, b.rule);
        assert_eq!(a.step.knowledge.summary(), b.knowledge.summary());
    }

    // Passing where no rule applies instead: South passes without a rule.
    let t = e
        .auction(&table(), &calls("1C 4NT"), OnNoRule::Pass)
        .unwrap();
    assert_eq!(t.stop.reason, StopReason::HumanToCall);
    assert_eq!(t.stop.index, 4);
    assert!(t.calls[2].choice.as_ref().unwrap().rule.is_none());
}

#[test]
fn all_bots_passing_where_no_rule_applies_is_bid_deal() {
    let e = engine();
    let hands = [N, E, S, W].map(|h| Hand::from_pbn(h).unwrap());
    let table = Table {
        dealer: Direction::North,
        vul: Vulnerability::NorthSouth,
        scoring: ScoringMethod::Matchpoints,
        hands: hands.clone().map(Some),
        bots: [true; 4],
    };
    let t = e.auction(&table, &[], OnNoRule::Pass).unwrap();
    let d = e
        .bid_deal(
            &hands,
            Direction::North,
            Vulnerability::NorthSouth,
            ScoringMethod::Matchpoints,
            &[],
            60,
        )
        .unwrap();
    assert_eq!(t.stop.reason, StopReason::Complete);
    assert_eq!(t.stop.seat, None);
    assert_eq!(t.stop.index, d.calls.len());
    let a: Vec<_> = t.calls.iter().map(|c| c.step.call.clone()).collect();
    let b: Vec<_> = d.calls.iter().map(|c| c.step.call.clone()).collect();
    assert_eq!(a, b);
    assert!(t.position.auction().is_complete());
}

#[test]
fn refuses_an_illegal_call_and_a_bot_without_a_hand() {
    let e = engine();
    assert!(e
        .auction(&table(), &calls("1C 1C"), OnNoRule::Stop)
        .unwrap_err()
        .contains("call 2"));
    let mut t = table();
    t.hands[2] = None;
    assert!(e
        .auction(&t, &[], OnNoRule::Stop)
        .unwrap_err()
        .contains("S is a bot seat"));
    // A human seat needs no hand.
    let t = e.auction(&table(), &[], OnNoRule::Stop).unwrap();
    assert_eq!(t.stop.reason, StopReason::HumanToCall);
    assert_eq!(t.stop.index, 0);
}
