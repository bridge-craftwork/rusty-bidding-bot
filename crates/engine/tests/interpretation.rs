//! What the engine knows and why it chose: interpretation, knowledge and
//! candidate traces, with the 21GF-DEFAULT card imported from BBA's .bbsa.
//! Tests that only check the call belong in the `.test` file next to the
//! module (run by tests/cases.rs).

use std::path::Path;
use std::sync::OnceLock;

use bridge_card::bbsa;
use bridge_types::{Call, Direction, Hand, ScoringMethod, Strain, Vulnerability};
use rbb_engine::{Decision, Engine, Range};

fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let text = std::fs::read_to_string(
            root.join("crates/bridge-card/tests/fixtures/bbsa/21GF-DEFAULT.bbsa"),
        )
        .unwrap();
        let (card, _) = bbsa::import(&text, None).unwrap();
        let modules = rbb_engine::load_modules(&root.join("conventions")).unwrap();
        Engine::new(&card, &card, &modules)
    })
}

fn calls(s: &str) -> Vec<Call> {
    s.split_whitespace()
        .map(|c| Call::from_pbn(c).unwrap())
        .collect()
}

/// South deals; `auction` is the calls before South's (or North's) turn.
fn bid(hand: &str, auction: &str) -> Decision {
    engine().bid(
        &Hand::from_pbn(hand).unwrap(),
        Direction::South,
        Vulnerability::None,
        ScoringMethod::Matchpoints,
        &calls(auction),
    )
}

fn explain(d: &Decision) -> String {
    let mut s = format!("chose {} ({})\n", d.call, d.explanation);
    for c in &d.candidates {
        s += &format!(
            "  {:5} p{} d{:.3} {}\n",
            c.call.to_string(),
            c.priority,
            c.descriptiveness,
            c.outcome
        );
    }
    for w in &d.warnings {
        s += &format!("  warning: {w}\n");
    }
    s
}

fn assert_call(d: &Decision, expected: &str) {
    assert_eq!(
        d.call,
        Call::from_pbn(expected).unwrap(),
        "\n{}",
        explain(d)
    );
}

#[test]
fn flat_hand_skips_stayman() {
    // 4-3-3-3 with a four-card major and 10 HCP: 3NT, not Stayman
    // (`when !shape 4333`); the trace says why Stayman was passed over.
    let d = bid("KJ82.Q73.K94.J83", "1NT Pass");
    assert_call(&d, "3NT");
    let stayman = d
        .candidates
        .iter()
        .find(|c| c.call == Call::bid(2, Strain::Clubs))
        .unwrap();
    assert_eq!(stayman.outcome, "hand fails `when !shape 4333`");
}

#[test]
fn interpretation_tracks_what_each_call_showed() {
    let i = engine().interpret(
        Direction::South,
        Vulnerability::None,
        ScoringMethod::Matchpoints,
        &calls("1NT Pass 2NT Pass"),
    );
    let south = &i.steps[0].knowledge;
    assert_eq!(south.hcp, Range::new(15, 17));
    assert_eq!(south.balanced, rbb_engine::Tri::True);
    // 2NT shows 8-9 total points (a 7-count with a five-card suit
    // qualifies), so HCP are only capped at 9.
    let north = &i.steps[2].knowledge;
    assert_eq!(north.whole_points(0), Range::new(8, 9), "{:#?}", i.steps[2]);
    assert_eq!(north.hcp.hi, 9);
    assert_eq!(i.steps[2].rule.as_ref().unwrap().module, "notrump-base");
}

#[test]
fn negative_inference_from_the_opening_pass() {
    // West passes as dealer... here South deals and passes: denies an opening.
    let i = engine().interpret(
        Direction::South,
        Vulnerability::None,
        ScoringMethod::Matchpoints,
        &calls("Pass"),
    );
    let south = &i.steps[0].knowledge;
    assert!(south.hcp.hi <= 11, "{south:#?}");
}

#[test]
fn jacoby_2nt_shortness_is_never_the_trump_suit() {
    // South opens 1S; North bids Jacoby 2NT. South, 13 HCP with a singleton
    // club, shows the shortness with 3C. `when x is not M` keeps 3S out of
    // the shortness rule (3S is "18+, no shortness").
    let d = bid("AKJ52.K73.Q943.8", "1S Pass 2NT Pass");
    assert_call(&d, "3C");
    let shortness_in_trumps = d
        .candidates
        .iter()
        .any(|c| c.call == Call::bid(3, Strain::Spades) && c.explanation.starts_with("Singleton"));
    assert!(!shortness_in_trumps, "\n{}", explain(&d));
    // Interpretation: 2NT agreed spades and set a game force.
    let state = &d.auction.position.sides[0];
    assert_eq!(state.trump, Some(Strain::Spades));
    assert_eq!(state.forcing, rbb_engine::Forcing::Game);
}

#[test]
fn strength_bands_follow_what_partner_showed() {
    // What an invitational 2NT shows, as partner sees it: 8-9 HCP, 5 hearts.
    let i = engine().interpret(
        Direction::South,
        Vulnerability::None,
        ScoringMethod::Matchpoints,
        &calls("1NT Pass 2D Pass 2H Pass 2NT"),
    );
    let north = &i.steps[6].knowledge;
    assert_eq!(north.whole_points(0), Range::new(8, 9));
    assert_eq!(north.len[2], Range::new(5, 5));
}

#[test]
fn keycard_answer_does_not_contradict_what_opener_showed() {
    // "No queen" cannot be checked for another player; dropping it inside a
    // negation used to make opener's knowledge contradictory.
    let i = engine().interpret(
        Direction::South,
        Vulnerability::None,
        ScoringMethod::Matchpoints,
        &calls("1NT Pass 4D Pass 4H Pass 4NT Pass 5H"),
    );
    let answer = &i.steps[8];
    assert!(answer.warnings.is_empty(), "{:?}", answer.warnings);
    assert_eq!(answer.knowledge.hcp, Range::new(15, 17));
}
