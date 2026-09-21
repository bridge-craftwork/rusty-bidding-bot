//! End to end: the 1NT slice (opening, Stayman, Jacoby transfers) with the
//! 21GF-DEFAULT card imported from BBA's .bbsa.

use std::path::Path;
use std::sync::OnceLock;

use bridge_card::bbsa;
use bridge_types::{Call, Direction, Hand, Strain, Vulnerability};
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
fn opens_1nt_with_15_to_17_balanced() {
    // 16 HCP, 4-3-3-3
    assert_call(&bid("AK52.KJ7.Q94.K83", ""), "1NT");
}

#[test]
fn opens_a_suit_or_passes_otherwise() {
    assert_call(&bid("AKJ52.K73.94.Q83", ""), "1S"); // 13, five spades
    assert_call(&bid("K52.A73.Q94.KJ83", ""), "1C"); // 13 balanced: 3-3 minors... 4 clubs
    assert_call(&bid("K52.A73.QJ94.K83", ""), "1D"); // 13, four diamonds
    assert_call(&bid("Q52.J73.Q94.K832", ""), "Pass"); // 8 HCP
}

#[test]
fn stayman_with_a_four_card_major_and_invitational_values() {
    // North, after 1NT (South) - Pass: 10 HCP, four spades, 4-2-4-3.
    assert_call(&bid("KJ82.Q7.K943.J83", "1NT Pass"), "2C");
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
fn transfers_with_a_five_card_major() {
    assert_call(&bid("82.QJ973.K94.J83", "1NT Pass"), "2D"); // weak, five hearts
    assert_call(&bid("QJ973.82.K94.J83", "1NT Pass"), "2H"); // five spades
}

#[test]
fn opener_answers_stayman() {
    assert_call(&bid("AK5.KJ72.Q94.K83", "1NT Pass 2C Pass"), "2H");
    assert_call(&bid("AK52.KJ7.Q94.K83", "1NT Pass 2C Pass"), "2S");
    assert_call(&bid("AK5.KJ7.Q942.K83", "1NT Pass 2C Pass"), "2D");
}

#[test]
fn opener_completes_or_super_accepts_a_transfer() {
    assert_call(&bid("AK5.KJ7.Q942.K83", "1NT Pass 2D Pass"), "2H"); // 3 hearts
    assert_call(&bid("AK5.KJ72.Q94.KQ3", "1NT Pass 2D Pass"), "3H"); // 4 hearts, 17
}

#[test]
fn weak_hand_passes_1nt() {
    assert_call(&bid("Q82.J73.Q942.J83", "1NT Pass"), "Pass");
}

#[test]
fn interpretation_tracks_what_each_call_showed() {
    let i = engine().interpret(
        Direction::South,
        Vulnerability::None,
        &calls("1NT Pass 2NT Pass"),
    );
    let south = &i.steps[0].knowledge;
    assert_eq!(south.hcp, Range::new(15, 17));
    assert_eq!(south.balanced, rbb_engine::Tri::True);
    let north = &i.steps[2].knowledge;
    assert_eq!(north.hcp, Range::new(8, 9), "{:#?}", i.steps[2]);
    assert_eq!(i.steps[2].rule.as_ref().unwrap().module, "notrump-base");
}

#[test]
fn negative_inference_from_the_opening_pass() {
    // West passes as dealer... here South deals and passes: denies an opening.
    let i = engine().interpret(Direction::South, Vulnerability::None, &calls("Pass"));
    let south = &i.steps[0].knowledge;
    assert!(south.hcp.hi <= 11, "{south:#?}");
}

#[test]
fn transfer_is_forcing() {
    // Opener may not pass partner's transfer, even with a minimum.
    let d = bid("AK5.Q72.Q942.K83", "1NT Pass 2D Pass");
    assert_ne!(d.call, Call::Pass, "\n{}", explain(&d));
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
