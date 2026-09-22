//! End to end: the 1NT slice (opening, Stayman, Jacoby transfers) with the
//! 21GF-DEFAULT card imported from BBA's .bbsa.

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

#[test]
fn responder_rebids_after_a_completed_transfer() {
    // One `when answered transfer(M)` block, no auction patterns. Strength is
    // in total points: a six-card suit adds a point (½ per card beyond four).
    let after = "1NT Pass 2D Pass 2H Pass";
    assert_call(&bid("82.QJ973.K94.J83", after), "Pass"); // 4 HCP: sign off
    assert_call(&bid("82.KJ973.K94.Q83", after), "2NT"); // 9, five hearts
    assert_call(&bid("82.QJ9732.K94.Q8", after), "3H"); // 8 HCP + 1 length = 9 points
    assert_call(&bid("82.KJ973.K94.AQ3", after), "3NT"); // 13, five hearts
    assert_call(&bid("82.KJ9732.K94.A8", after), "4H"); // 13, six hearts
    assert_call(&bid("8.KJ973.K4.AQ832", after), "3C"); // second suit, GF
                                                        // The same block over spades.
    assert_call(&bid("KJ973.82.K94.Q83", "1NT Pass 2H Pass 2S Pass"), "2NT");
}

#[test]
fn strength_bands_follow_what_partner_showed() {
    // After a maximum super-accept (2NT: 16-17 with four hearts) the
    // invitational band shrinks to exactly 8, so 9 HCP is a game hand, in
    // the known fit.
    let d = bid("82.KJ973.K94.Q83", "1NT Pass 2D Pass 2NT Pass");
    assert_call(&d, "4H");
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
fn opener_answers_invitations() {
    // 1NT-2NT: accept only with a maximum (Basic_NT board 51: 17 HCP);
    // BBA declines with 16.
    assert_call(&bid("KQJ.KJ73.AK5.T52", "1NT Pass 2NT Pass"), "3NT"); // 17
    assert_call(&bid("KQJ.KJ73.AQ5.T52", "1NT Pass 2NT Pass"), "Pass"); // 16
                                                                        // After a transfer and 2NT (five hearts): fit decides the strain.
    let inv = "1NT Pass 2D Pass 2H Pass 2NT Pass";
    assert_call(&bid("KQ5.KJ7.AK53.J52", inv), "4H"); // 17, three hearts
    assert_call(&bid("KQ5.KJ7.AQ53.T52", inv), "3H"); // 15, three hearts
    assert_call(&bid("KQ53.KJ.AK53.J52", inv), "3NT"); // 17, two hearts
    assert_call(&bid("KQ53.KJ.AQ53.T52", inv), "Pass"); // 15, two hearts
                                                        // After 3H (six hearts) two-card support is a fit.
    assert_call(
        &bid("KQ53.KJ.AK53.J52", "1NT Pass 2D Pass 2H Pass 3H Pass"),
        "4H",
    );
}

#[test]
fn total_points_count_tens_and_length() {
    // 9 HCP and a six-card suit is 10 total points: game, not an invite.
    assert_call(&bid("82.KJ9732.K94.Q8", "1NT Pass 2D Pass 2H Pass"), "4H");
    // 7 HCP, 4-3-3-3 with four tens (7 + 1 = 8 points): invite.
    assert_call(&bid("T82.QT7.KT9.QT83", "1NT Pass"), "2NT");
    // Same shape and HCP without the tens: pass.
    assert_call(&bid("982.Q87.K95.Q853", "1NT Pass"), "Pass");
}

#[test]
fn opener_corrects_3nt_to_a_known_major_fit() {
    // Stayman, 2H, 3NT: responder has four spades (no heart raise), so
    // opener with four spades too plays 4S.
    assert_call(
        &bid("AK52.KJ73.Q94.K8", "1NT Pass 2C Pass 2H Pass 3NT Pass"),
        "4S",
    );
    assert_call(
        &bid("AK5.KJ73.Q942.K8", "1NT Pass 2C Pass 2H Pass 3NT Pass"),
        "Pass",
    );
    // Transfer, then 3NT with five hearts: opener with three hearts plays 4H.
    assert_call(
        &bid("AK5.KJ7.Q942.K83", "1NT Pass 2D Pass 2H Pass 3NT Pass"),
        "4H",
    );
    assert_call(
        &bid("AK52.KJ.Q942.K83", "1NT Pass 2D Pass 2H Pass 3NT Pass"),
        "Pass",
    );
}

#[test]
fn responder_raises_after_stayman() {
    let after = "1NT Pass 2C Pass 2S Pass";
    assert_call(&bid("KJ82.Q7.K943.J83", after), "4S"); // 10: game in the fit
    assert_call(&bid("KJ82.Q7.Q943.J83", after), "3S"); // 9: invite in the fit
                                                        // After 2H with four spades and no heart fit: 2NT / 3NT.
    assert_call(&bid("KJ82.Q7.K943.J83", "1NT Pass 2C Pass 2H Pass"), "3NT");
}

#[test]
fn texas_or_jacoby_by_slam_interest() {
    // Game only (12 suit points): Texas, then pass the completion.
    assert_call(&bid("8.KQ9732.K94.Q83", "1NT Pass"), "4D");
    assert_call(&bid("8.KQ9732.K94.Q83", "1NT Pass 4D Pass 4H Pass"), "Pass");
    // Mild slam interest (15 suit points): 2D, then jump to game.
    assert_call(&bid("82.KQ9732.A94.KQ", "1NT Pass"), "2D");
    assert_call(&bid("82.KQ9732.A94.KQ", "1NT Pass 2D Pass 2H Pass"), "4H");
    // Opener bids on with a good fit and a maximum, else passes.
    assert_call(&bid("AK5.Q74.KQ82.K93", "1NT Pass 2D Pass 2H Pass 4H Pass"), "4NT");
    assert_call(&bid("AJ5.J74.KQ82.K93", "1NT Pass 2D Pass 2H Pass 4H Pass"), "Pass");
    // Slam values (17 suit points): Texas, then keycard.
    assert_call(&bid("8.AQ9732.K94.AQ8", "1NT Pass"), "4D");
    assert_call(&bid("8.AQ9732.K94.AQ8", "1NT Pass 4D Pass 4H Pass"), "4NT");
}

#[test]
fn keycard_answers_resolve_with_the_deck_limit() {
    let asked = "1NT Pass 4D Pass 4H Pass 4NT Pass";
    // "1 or 4" facing my two: only 1 fits, so two are missing: sign off.
    assert_call(&bid("8.AQ9732.K94.AQ8", &format!("{asked} 5C Pass")), "5H");
    // "0 or 3" facing my two: 2 or 5 in all, unresolved: sign off in five,
    // and partner corrects holding the higher count.
    assert_call(&bid("8.AQ9732.K94.AQ8", &format!("{asked} 5D Pass")), "5H");
    let signed_off = format!("{asked} 5D Pass 5H Pass");
    assert_call(&bid("AK5.K74.KQ82.A93", &signed_off), "6H"); // 3 keycards
    assert_call(&bid("KJ5.K74.KQ82.Q93", &signed_off), "Pass"); // 0 keycards
}
