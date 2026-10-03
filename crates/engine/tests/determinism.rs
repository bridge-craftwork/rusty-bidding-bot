//! The same deal gets the same auction however the engine got there: two
//! engines bid the same random deals in opposite orders (so their caches
//! fill differently, and their hash maps are seeded differently) and must
//! agree on every call, rule, explanation and reading. The WASM build is
//! held to the same auctions by web/scripts/determinism.mjs (CI, web job).

use std::path::Path;

use bridge_card::bbsa;
use bridge_types::{Card, Direction, Hand, ScoringMethod, Vulnerability};
use rbb_engine::{DealAuction, Engine};

const DEALS: usize = 24;

fn engine() -> Engine {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let card =
        |name: &str| std::fs::read_to_string(root.join(format!("cards/bbsa/{name}.bbsa"))).unwrap();
    let rules = rbb_engine::load_rules(&root.join("conventions")).unwrap();
    let (ns, _) = bbsa::import(&rules.vocab, &card("21GF-DEFAULT"), None).unwrap();
    let (ew, _) = bbsa::import(&rules.vocab, &card("21GF-GIB"), None).unwrap();
    Engine::new(&ns, &ew, &rules)
}

/// Deal `n` of a fixed sequence (xorshift64*), with the board's dealer and
/// vulnerability.
fn deal(n: usize) -> ([Hand; 4], Direction, Vulnerability, ScoringMethod) {
    let mut s = 0x2545_F491_4F6C_DD1Du64 ^ (n as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut next = || {
        s ^= s >> 12;
        s ^= s << 25;
        s ^= s >> 27;
        s.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let mut deck: Vec<Card> = (0..52).filter_map(Card::from_index).collect();
    for i in (1..52).rev() {
        deck.swap(i, (next() % (i as u64 + 1)) as usize);
    }
    let hands = std::array::from_fn(|h| Hand::from_cards(deck[h * 13..h * 13 + 13].to_vec()));
    let dealer = Direction::from_index(n % 4).unwrap();
    let vul = [
        Vulnerability::None,
        Vulnerability::NorthSouth,
        Vulnerability::EastWest,
        Vulnerability::Both,
    ][(n / 4 + n) % 4];
    let scoring = if n.is_multiple_of(2) {
        ScoringMethod::IMP
    } else {
        ScoringMethod::Matchpoints
    };
    (hands, dealer, vul, scoring)
}

fn bid(e: &Engine, n: usize) -> DealAuction {
    let (hands, dealer, vul, scoring) = deal(n);
    e.bid_deal(&hands, dealer, vul, scoring, &[], 60).unwrap()
}

/// Everything the auction shows, as text.
fn digest(a: &DealAuction) -> Vec<String> {
    a.calls
        .iter()
        .map(|c| {
            let choice = c.choice.as_ref().unwrap();
            format!(
                "{} {:?} {} | {:?} {:?} {} | {:?}",
                c.step.call,
                choice.rule,
                choice.explanation,
                c.step.rule,
                c.step.explanation,
                c.step.knowledge.summary(),
                choice
                    .candidates
                    .iter()
                    .map(|k| (k.call.to_string(), k.outcome.clone()))
                    .collect::<Vec<_>>()
            )
        })
        .collect()
}

#[test]
fn the_same_deal_gets_the_same_auction_in_any_order() {
    let (forward, mut backward) = std::thread::scope(|s| {
        let f = s.spawn(|| {
            let a = engine();
            (0..DEALS).map(|n| digest(&bid(&a, n))).collect::<Vec<_>>()
        });
        let b = engine();
        let back: Vec<_> = (0..DEALS).rev().map(|n| digest(&bid(&b, n))).collect();
        (f.join().unwrap(), back)
    });
    backward.reverse();
    let mut bid_something = 0;
    for (n, (x, y)) in forward.iter().zip(&backward).enumerate() {
        assert_eq!(x, y, "deal {n}: {:?}", deal(n).0);
        bid_something += usize::from(x.len() > 4);
    }
    assert!(bid_something > DEALS / 2, "most deals are not passed out");
}

/// The same hands under each vulnerability, the four bid in opposite
/// orders by two engines: the first calls (passes, mostly) repeat across
/// the four boards, so a cache keyed on the calls but not on everything
/// their meaning depends on hands the board bid second whatever the first
/// one stored (the descriptiveness cache did until 2026-10-03, which made
/// `rbb compare` differ from run to run).
#[test]
fn the_vulnerability_never_leaks_through_a_cache() {
    const VULS: [Vulnerability; 4] = [
        Vulnerability::None,
        Vulnerability::NorthSouth,
        Vulnerability::EastWest,
        Vulnerability::Both,
    ];
    let run = |order: [usize; 4]| {
        let e = engine();
        let mut out = vec![Vec::new(); DEALS * 4];
        for n in 0..DEALS {
            let (hands, dealer, _, scoring) = deal(n);
            for v in order {
                let a = e
                    .bid_deal(&hands, dealer, VULS[v], scoring, &[], 60)
                    .unwrap();
                out[n * 4 + v] = digest(&a);
            }
        }
        out
    };
    let (forward, backward) = std::thread::scope(|s| {
        let f = s.spawn(|| run([0, 1, 2, 3]));
        let b = run([3, 2, 1, 0]);
        (f.join().unwrap(), b)
    });
    for (i, (x, y)) in forward.iter().zip(&backward).enumerate() {
        assert_eq!(x, y, "deal {} at {:?}", i / 4, VULS[i % 4]);
    }
}
