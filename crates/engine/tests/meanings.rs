//! Reading the opponents' announced meanings (`Engine::interpret_with`,
//! `bid_with`): a known convention name switches their card's field on, a
//! text on a call no rule reads shows what it says, partner's text is
//! never read, and the engine's caches keep the two readings apart. Our
//! own announcements (`Step::announcement`) read back as what they say.

use std::path::Path;
use std::sync::OnceLock;

use bridge_card::Card;
use bridge_types::{Call, Direction, Hand, ScoringMethod, Vulnerability};
use rbb_engine::{meanings, AnnouncedNames, Engine, HeardRead, Meanings, RuleSet};

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A few entries in the pack's form (announced-names.toml): the compiled-in
/// snapshot predates the table.
const TABLE: &str = r#"
[[name]]
names = ["Cappelletti", "Hamilton"]
set = ["defence_to_1nt.strong=competitive_bidding/cappelletti", "defence_to_1nt.weak=competitive_bidding/cappelletti"]

[[name]]
names = ["Michaels", "Michaels cue bid"]
call = "2M"
after = "(1M) | (1M) P (P)"
set = ["competitive_bidding/michaels", "competitive_bidding/michaels:over_major"]

[[name]]
names = ["Multi"]
set = ["bidding_conventions/multi_2d", "opening.2d=bidding_conventions/multi_2d"]

[[name]]
names = ["Bogus"]
set = ["no_such/field"]

[[name]]
names = ["Drury", "Reverse Drury"]
set = ["bidding_conventions/drury", "bidding_conventions/drury:reverse"]

[[name]]
names = ["Original Drury", "Standard Drury"]
set = ["bidding_conventions/drury", "bidding_conventions/drury:reverse=false"]
"#;

fn rules() -> &'static RuleSet {
    static RULES: OnceLock<RuleSet> = OnceLock::new();
    RULES.get_or_init(|| {
        rbb_engine::load_rules(&root().join("pack/pack.json"))
            .unwrap()
            .with_announced_names(AnnouncedNames::from_toml(TABLE).unwrap())
    })
}

fn card(name: &str) -> Card {
    let text = std::fs::read_to_string(root().join(format!("cards/json/{name}.json"))).unwrap();
    Card::from_json(&rules().vocab, &text).unwrap().0
}

/// North-South play the Brill card, East-West Natural (no Cappelletti, no
/// Michaels), as an opponent whose card we do not know is read.
fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| Engine::new(&card("Brill"), &card("Natural"), rules()))
}

fn calls(s: &str) -> Vec<Call> {
    s.split_whitespace()
        .map(|c| Call::from_pbn(c).unwrap())
        .collect()
}

fn texts(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn heard_by_south(v: &[&str]) -> Meanings {
    Meanings::heard_by(Direction::South, texts(v))
}

const NV: Vulnerability = Vulnerability::None;
const IMP: ScoringMethod = ScoringMethod::IMP;

#[test]
fn a_known_name_switches_their_card() {
    let a = calls("1NT 2D");
    let plain = engine().interpret(Direction::North, NV, IMP, &a);
    let m = heard_by_south(&["", "Cappelletti: both majors"]);
    let read = engine().interpret_with(Direction::North, NV, IMP, &a, &m);
    let (p, r) = (&plain.steps[1], &read.steps[1]);
    assert_ne!(
        p.rule.as_ref().map(|r| r.module.as_str()),
        Some("cappelletti")
    );
    assert_eq!(
        r.rule.as_ref().map(|r| r.module.as_str()),
        Some("cappelletti")
    );
    let h = r.heard.as_ref().unwrap();
    assert_eq!(h.name.as_deref(), Some("Cappelletti"));
    assert!(h.sets[0].starts_with("defence_to_1nt.strong="));
    // Both majors: four or more in each.
    assert!(
        r.knowledge.len[2].lo >= 4 && r.knowledge.len[3].lo >= 4,
        "{:?}",
        r.knowledge
    );
    // And South answers it as Cappelletti: no natural 2H over it.
    let hand = Hand::from_pbn("KJT98.82.Q65.T98").unwrap();
    let d0 = engine().bid(&hand, Direction::North, NV, IMP, &a);
    let d1 = engine().bid_with(&hand, Direction::North, NV, IMP, &a, &m.texts);
    assert_eq!(d0.call, Call::from_pbn("2S").unwrap());
    assert_ne!(d1.call, d0.call);
}

#[test]
fn a_context_picks_the_meaning() {
    // "Michaels" applies to 2M over 1M only.
    let a = calls("1H 2H");
    let m = heard_by_south(&["", "Michaels: spades and a minor"]);
    let r = engine().interpret_with(Direction::North, NV, IMP, &a, &m);
    let h = r.steps[1].heard.as_ref().unwrap();
    assert_eq!(h.name.as_deref(), Some("Michaels"));
    assert_eq!(
        r.steps[1].rule.as_ref().map(|r| r.module.as_str()),
        Some("two-suited-overcalls")
    );
    let a = calls("1C 2C");
    let r = engine().interpret_with(Direction::North, NV, IMP, &a, &m);
    assert_eq!(r.steps[1].heard.as_ref().unwrap().name, None);
}

#[test]
fn an_unread_call_shows_what_its_text_says() {
    let a = calls("1H 2H");
    let plain = engine().interpret(Direction::North, NV, IMP, &a);
    assert!(
        plain.steps[1].rule.is_none(),
        "2H is unread on the Natural card"
    );
    let m = heard_by_south(&["", "5+ spades and a 5+ minor, 8-15 HCP, artificial"]);
    let r = engine().interpret_with(Direction::North, NV, IMP, &a, &m);
    let s = &r.steps[1];
    assert!(s.rule.is_none());
    assert_eq!(s.knowledge.len[3].lo, 5);
    assert_eq!((s.knowledge.hcp.lo, s.knowledge.hcp.hi), (8, 15));
    assert!(s.artificial && r.position.artificial[1]);
    let h = s.heard.as_ref().unwrap();
    assert_eq!(h.shows, texts(&["hcp=8..15", "S>=5"]));
    assert_eq!(
        s.announcement,
        "5+ spades and a 5+ minor, 8-15 HCP, artificial"
    );
    // South's rules see it: a limit raise or better is a cue bid now.
    let hand = Hand::from_pbn("A5.K82.Q6532.T98").unwrap();
    let d0 = engine().bid(&hand, Direction::North, NV, IMP, &a);
    let d1 = engine().bid_with(&hand, Direction::North, NV, IMP, &a, &m.texts);
    assert_ne!(d0.call, d1.call, "{} / {}", d0.explanation, d1.explanation);
}

#[test]
fn a_text_that_says_nothing_changes_nothing() {
    let a = calls("1H 2H");
    let plain = engine().interpret(Direction::North, NV, IMP, &a);
    let m = heard_by_south(&["", "see our card"]);
    let r = engine().interpret_with(Direction::North, NV, IMP, &a, &m);
    assert_eq!(r.position.knowledge, plain.position.knowledge);
    assert_eq!(r.position.heard, "");
    assert!(r.steps[1].heard.as_ref().unwrap().shows.is_empty());
}

#[test]
fn partner_is_never_heard() {
    // North's own announcement reaches South only as partner's: ignored.
    let a = calls("1H 2H");
    let hand = Hand::from_pbn("A5.K82.Q6532.T98").unwrap();
    let d = engine().bid_with(
        &hand,
        Direction::East,
        NV,
        IMP,
        &calls("P 1H 2H"),
        &texts(&["", "Michaels: spades and a minor", ""]),
    );
    // Dealer East: E P, S 1H, W 2H; North to call, West is an opponent.
    assert!(d.auction.steps[2].heard.is_none());
    assert!(
        d.auction.steps[1].heard.is_none(),
        "South is North's partner"
    );
    let _ = a;
}

#[test]
fn caches_keep_the_readings_apart() {
    // The same calls read with and without a text: each answer is what a
    // fresh engine gives, whichever came first.
    let a = calls("1H 2H");
    let hand = Hand::from_pbn("J4.A97.KQ8.QJ862").unwrap();
    let m = texts(&["", "5+ spades and a 5+ minor, artificial"]);
    let fresh = || Engine::new(&card("Brill"), &card("Natural"), rules());
    let e = fresh();
    let with_first = e.bid_with(&hand, Direction::North, NV, IMP, &a, &m);
    let without_after = e.bid(&hand, Direction::North, NV, IMP, &a);
    let e = fresh();
    let without_first = e.bid(&hand, Direction::North, NV, IMP, &a);
    let with_after = e.bid_with(&hand, Direction::North, NV, IMP, &a, &m);
    assert_eq!(with_first.call, with_after.call);
    assert_eq!(without_first.call, without_after.call);
    let descr = |d: &rbb_engine::Decision| {
        d.candidates
            .iter()
            .map(|c| (c.call.to_string(), (c.descriptiveness * 1e6) as i64))
            .collect::<Vec<_>>()
    };
    assert_eq!(descr(&with_first), descr(&with_after));
    assert_eq!(descr(&without_first), descr(&without_after));
}

#[test]
fn a_field_the_card_lacks_is_a_warning() {
    let a = calls("2D");
    let m = Meanings::heard_by(Direction::South, texts(&["Bogus 2D"]));
    let r = engine().interpret_with(Direction::East, NV, IMP, &a, &m);
    assert!(
        r.steps[0]
            .warnings
            .iter()
            .any(|w| w.contains("no_such/field")),
        "{:?}",
        r.steps[0].warnings
    );
    assert!(!rules().announced.check(&rules().vocab).is_empty());
}

#[test]
fn a_snapshot_without_the_table_reads_no_names() {
    let text = std::fs::read_to_string(root().join("pack/pack.json")).unwrap();
    let names = rbb_engine::snapshot_announced_names(&text).unwrap();
    // The compiled-in snapshot may or may not carry it yet; either loads.
    let _ = names.len();
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    doc.as_object_mut().unwrap().remove(meanings::SNAPSHOT_KEY);
    let rules = rbb_engine::compile_snapshot(&doc.to_string(), "pack.json").unwrap();
    assert!(rules.announced.is_empty());
    // And one that has it loads it.
    let table: toml::Value = toml::from_str(TABLE).unwrap();
    doc[meanings::SNAPSHOT_KEY] = serde_json::to_value(table).unwrap();
    let rules = rbb_engine::compile_snapshot(&doc.to_string(), "pack.json").unwrap();
    assert_eq!(rules.announced.len(), 6);
}

/// Our own announcements, for people and for Brill (`Audience`), read
/// back by the parser, recover what they state: every HCP range and suit length read holds the caller's known
/// range (the text states only bounds the call narrowed).
#[test]
fn our_announcements_read_back() {
    let e = Engine::new(&card("Brill"), &card("Brill"), rules());
    let auctions = [
        "1S",
        "1H",
        "1C",
        "1D",
        "1NT",
        "2NT",
        "2C",
        "2D",
        "2H",
        "3S",
        "1S P 2S",
        "1S P 3S",
        "1S P 2NT",
        "1S P 4D",
        "1H P 1S",
        "1H P 1NT",
        "1C P 1H P 1S",
        "1C P 1H P 1NT",
        "1C P 1D P 1H P 2D",
        "1D P 1S P 2S",
        "1NT P 2C",
        "1NT P 2C P 2H",
        "1NT P 2C P 2D",
        "1NT P 2D",
        "1NT P 2D P 2H",
        "1NT P 2H",
        "1NT P 3NT",
        "2C P 2D P 2H",
        "1H 1S",
        "1H 2C",
        "1H X",
        "1H 1S 2H",
        "1H 1S X",
        "1D 1H 1S",
        "1C 1NT",
        "1NT 2D",
        "1NT 2C",
        "1NT X",
        "1H 2H",
        "1S 2NT",
        "2S X",
        "1S P 2C P 2D",
        "1H P 2H P 3H",
    ];
    let mut read = 0;
    for a in auctions {
        let calls = calls(a);
        let i = e.interpret(Direction::North, NV, IMP, &calls);
        for (s, audience) in i
            .steps
            .iter()
            .flat_map(|s| rbb_engine::explain::Audience::ALL.map(|a| (s, a)))
        {
            let text = s.announcement_for(audience);
            if text.is_empty() {
                continue;
            }
            let p = meanings::parse(&text, Some(&s.call));
            let k = &s.knowledge;
            let ctx = format!(
                "{a} ({audience:?}): {} {text:?} -> {p:?}; known {k:?}",
                s.call
            );
            if let Some(r) = p.hcp {
                assert!(r.lo <= k.hcp.lo && k.hcp.hi <= r.hi, "HCP: {ctx}");
                read += 1;
            } else if text.contains(" HCP") {
                panic!("HCP stated but not read: {ctx}");
            }
            for suit in 0..4 {
                if let Some(r) = p.len[suit] {
                    let l = k.len[suit];
                    assert!(r.lo <= l.lo && l.hi <= r.hi, "suit {suit}: {ctx}");
                    read += 1;
                }
            }
            if p.balanced == Some(true) {
                assert_ne!(k.balanced, rbb_engine::Tri::False, "balanced: {ctx}");
            }
            if s.artificial {
                assert!(p.artificial || !text.contains("artificial"), "{ctx}");
            }
        }
    }
    assert!(read > 30, "only {read} bounds read back");
}

// Rick's rulings of 2026-11-20 on reading their texts.

#[test]
fn a_name_reads_their_earlier_calls_too() {
    // Cappelletti named on advancer's 2D only: overcaller's 2C before it
    // is read by our Cappelletti module as well.
    let a = calls("1NT 2C P 2D");
    let plain = engine().interpret(Direction::North, NV, IMP, &a);
    let m = heard_by_south(&["", "", "", "Cappelletti: relay"]);
    let read = engine().interpret_with(Direction::North, NV, IMP, &a, &m);
    let module = |s: &rbb_engine::Step| s.rule.as_ref().map(|r| r.module.clone());
    assert_ne!(module(&plain.steps[1]).as_deref(), Some("cappelletti"));
    assert_eq!(module(&read.steps[1]).as_deref(), Some("cappelletti"));
    assert!(read.steps[1].heard.is_none(), "nothing was said on 2C");
}

#[test]
fn a_text_narrows_the_rules_reading() {
    // East opens 1H on the Natural card: a rule reads it. "11-15 points"
    // (plain points are HCP) narrows its range.
    let a = calls("1H");
    let plain = engine().interpret(Direction::East, NV, IMP, &a);
    let k0 = &plain.steps[0].knowledge;
    assert!(plain.steps[0].rule.is_some());
    assert!(k0.hcp.hi > 15, "{k0:?}");
    let m = heard_by_south(&["5+ hearts, 11-15 points"]);
    let read = engine().interpret_with(Direction::East, NV, IMP, &a, &m);
    let s = &read.steps[0];
    assert_eq!(s.rule, plain.steps[0].rule, "the rule still reads it");
    let h = s.heard.as_ref().unwrap();
    assert_eq!(h.read, HeardRead::Narrowed);
    assert_eq!(h.shows, texts(&["hcp=11..15"]), "5+ hearts adds nothing");
    assert_eq!(
        (s.knowledge.hcp.lo, s.knowledge.hcp.hi),
        (k0.hcp.lo.max(11), 15)
    );
    assert_eq!(s.knowledge.len, k0.len);
    assert_ne!(read.position.heard, "", "the caches tell it apart");
}

#[test]
fn a_text_that_agrees_changes_nothing() {
    let a = calls("1H");
    let plain = engine().interpret(Direction::East, NV, IMP, &a);
    let m = heard_by_south(&["5+ hearts"]);
    let read = engine().interpret_with(Direction::East, NV, IMP, &a, &m);
    assert_eq!(
        read.steps[0].heard.as_ref().unwrap().read,
        HeardRead::Agrees
    );
    assert_eq!(read.position.knowledge, plain.position.knowledge);
    assert_eq!(read.position.heard, "");
}

#[test]
fn a_text_that_contradicts_the_rule_is_ignored() {
    // 1H is five or more hearts; a text of two to three hearts and some
    // HCP that would fit is not read at all: the rule's reading stands.
    let a = calls("1H");
    let plain = engine().interpret(Direction::East, NV, IMP, &a);
    let m = heard_by_south(&["2-3 hearts, 12-14 HCP"]);
    let read = engine().interpret_with(Direction::East, NV, IMP, &a, &m);
    let s = &read.steps[0];
    let h = s.heard.as_ref().unwrap();
    assert_eq!(h.read, HeardRead::Contradicts);
    assert!(h.shows.is_empty());
    assert_eq!(read.position.knowledge, plain.position.knowledge);
    assert_eq!(read.position.heard, "");
    assert!(
        s.warnings.iter().any(|w| w.contains("text ignored")),
        "{:?}",
        s.warnings
    );
}

#[test]
fn an_unread_call_with_a_text_counts_as_read() {
    // 1H 2H is unread on the Natural card; a text that says what it shows
    // makes it read (`unread` false), one that says nothing does not.
    let a = calls("1H 2H");
    let plain = engine().interpret(Direction::North, NV, IMP, &a);
    assert!(plain.position.unread(Direction::East));
    let m = heard_by_south(&["", "5+ spades and a 5+ minor, 8-15 HCP"]);
    let read = engine().interpret_with(Direction::North, NV, IMP, &a, &m);
    assert!(!read.position.unread(Direction::East));
    assert_eq!(
        read.steps[1].heard.as_ref().unwrap().read,
        HeardRead::Unread
    );
    let m = heard_by_south(&["", "artificial"]);
    let read = engine().interpret_with(Direction::North, NV, IMP, &a, &m);
    assert!(
        read.position.unread(Direction::East),
        "a flag shows nothing"
    );
    let m = heard_by_south(&["", "see our card"]);
    let read = engine().interpret_with(Direction::North, NV, IMP, &a, &m);
    assert!(read.position.unread(Direction::East));
}

#[test]
fn drury_is_reverse_unless_original() {
    // Dealer East: E P, S P, W 1H, N P, E 2C (Drury); North and South hear.
    let a = calls("P P 1H P 2C P 2D");
    let with = |t: &str| {
        let m = heard_by_south(&["", "", "", "", t, "", ""]);
        engine().interpret_with(Direction::East, NV, IMP, &a, &m)
    };
    let plain = with("Drury");
    let h = plain.steps[4].heard.as_ref().unwrap();
    assert_eq!(h.name.as_deref(), Some("Drury"));
    assert!(
        h.sets
            .iter()
            .any(|s| s == "bidding_conventions/drury:reverse=true"),
        "{h:?}"
    );
    let original = with("Original Drury");
    let h = original.steps[4].heard.as_ref().unwrap();
    assert!(
        h.sets
            .iter()
            .any(|s| s == "bidding_conventions/drury:reverse=false"),
        "{h:?}"
    );
    let module = |s: &rbb_engine::Step| s.rule.as_ref().map(|r| r.module.clone());
    assert_eq!(module(&plain.steps[4]).as_deref(), Some("drury"));
    assert_eq!(module(&original.steps[4]).as_deref(), Some("drury"));
    // Opener's 2D: a full opening in Reverse Drury, a light one in the
    // original.
    let (r, o) = (
        &plain.steps[6].knowledge.hcp,
        &original.steps[6].knowledge.hcp,
    );
    assert!(r.lo > o.lo || r.hi > o.hi, "reverse {r:?} original {o:?}");
}
