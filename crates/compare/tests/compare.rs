//! The comparison on eight real boards from Practice-Bidding-Scenarios/bba/1N.

use std::path::Path;

use rbb_compare::{run, Options};

fn options() -> Options {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    Options {
        pbs: here.join("tests/fixtures/pbs"),
        scenarios: vec![],
        limit: None,
        rules: here.join("../../conventions"),
        par: false,
        dd_cache: std::env::temp_dir().join("rbb-compare-test-dd.jsonl"),
    }
}

#[test]
fn replay_and_divergence_are_consistent() {
    let report = run(&options(), &|_, _| {}).unwrap();
    assert_eq!(report.boards.len(), 8);
    assert_eq!(report.summary.total.boards, 8);
    for b in &report.boards {
        assert_eq!(b.replay.len(), b.reference.len());
        assert_eq!(b.reference_alerts.len(), b.reference.len());
        match b.first_divergence {
            None => {
                assert_eq!(b.replay, b.reference);
                assert_eq!(b.ours, b.reference);
                assert!(b.contracts_match());
            }
            Some(d) => {
                assert_eq!(b.replay[..d], b.reference[..d]);
                assert_ne!(b.replay[d], b.reference[d]);
                // Our auction follows BBA up to the difference, then is ours.
                assert_eq!(b.ours[..d], b.reference[..d]);
                assert_eq!(b.ours[d], b.replay[d]);
            }
        }
    }
    let s = &report.summary;
    let calls = s.total.calls_all();
    assert_eq!(
        calls.total,
        report
            .boards
            .iter()
            .map(|b| b.reference.len())
            .sum::<usize>()
    );
    let diverged: usize = s.total.first_divergence.values().sum();
    assert_eq!(diverged + s.total.auctions_match, 8);
    assert_eq!(
        s.divergences.iter().map(|d| d.count).sum::<usize>(),
        diverged
    );
}

#[test]
fn bba_alerts_are_read_from_notes() {
    let report = run(&options(), &|_, _| {}).unwrap();
    // 1N.pbn alerts transfers and Stayman with [Note]s.
    let alerts: Vec<&str> = report
        .boards
        .iter()
        .flat_map(|b| b.reference_alerts.iter().flatten())
        .map(String::as_str)
        .collect();
    assert!(!alerts.is_empty());
}
