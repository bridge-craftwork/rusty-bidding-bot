//! The comparison on eight real boards from Practice-Bidding-Scenarios/bba/1N.

use std::path::Path;

use rbb_compare::{par, run, Options};

fn options() -> Options {
    options_for("1N")
}

fn options_for(scenario: &str) -> Options {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    Options {
        pbs: here.join("tests/fixtures/pbs"),
        scenarios: vec![scenario.to_string()],
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

/// The corpus files carry `OptimumResultTable` sections. Par must come from
/// them: no solving, on every run rather than only under `--par`.
#[test]
fn par_comes_from_the_files_double_dummy_table() {
    use bridge_types::{Direction, Strain};

    let mut opts = options_for("Basic_NT");
    // A cache path that does not exist and must not be written to: any
    // solving here would be a bug.
    opts.dd_cache = std::env::temp_dir().join("rbb-compare-test-no-solve.jsonl");
    let _ = std::fs::remove_file(&opts.dd_cache);
    assert!(!opts.par, "par is off: the tables come from the file");

    let report = run(&opts, &|_, _| {}).unwrap();
    assert_eq!(report.summary.total.dd_tables, report.boards.len());
    for b in &report.boards {
        let dd = b.dd.expect("the file's double-dummy table");
        assert!(!dd.is_null());
        assert_eq!(
            b.par.is_some(),
            !b.contracts_match(),
            "par is scored where the contracts differ"
        );
        if b.board == "1" {
            // North-South make ten tricks in diamonds and nine in notrump.
            assert_eq!(dd.tricks(Direction::North, Strain::Diamonds), 10);
            assert_eq!(dd.tricks(Direction::North, Strain::NoTrump), 9);
            // 3NT by North makes, not vulnerable: par is +400 to us.
            let (par, table) = rbb_compare::par_for(b, &par::DdCache::open(opts.dd_cache.clone()))
                .expect("par from the board's own table");
            assert_eq!(table, dd);
            assert_eq!(par.par_ns, 400);
        }
    }
    assert!(!opts.dd_cache.exists(), "nothing was solved");
}

/// Scenario arguments may be patterns, so the workbench can be pointed at
/// a family of scenarios without listing them.
#[test]
fn scenario_patterns_select_a_family() {
    let report = run(&options_for("Basic_*"), &|_, _| {}).unwrap();
    assert!(report
        .summary
        .scenarios
        .iter()
        .all(|s| s.name.starts_with("Basic_")));
    assert_eq!(report.summary.scenarios.len(), 1, "one Basic_ fixture");

    let all = run(&options_for("*"), &|_, _| {}).unwrap();
    assert_eq!(all.summary.scenarios.len(), 2, "1N and Basic_NT");

    let err = run(&options_for("Basik_*"), &|_, _| {}).unwrap_err();
    assert!(err.contains("no scenario matching"), "{err}");
}
