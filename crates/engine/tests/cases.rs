//! Runs every `conventions/**/*.test` file (see `rbb_engine::cases`).
//! `rbb bid test` runs the same cases with a fuller report.

use std::path::Path;

#[test]
fn convention_cases() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let files = rbb_engine::cases::find(&root.join("conventions"));
    assert!(!files.is_empty(), "no .test files under conventions/");
    let outcomes = rbb_engine::cases::run(
        &files,
        &root.join("conventions"),
        &root.join("crates/bridge-card/tests/fixtures/bbsa"),
    )
    .unwrap_or_else(|errors| panic!("\n{}", errors.join("\n")));
    let failures: Vec<String> = outcomes
        .iter()
        .filter(|o| !o.passed)
        .map(|o| o.report())
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n{}",
        failures.len(),
        outcomes.len(),
        failures.join("\n")
    );
}

#[test]
fn card_references_are_valid() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let modules = rbb_engine::load_modules(&root.join("conventions")).unwrap();
    let errors = rbb_engine::check_card_refs(&modules);
    assert!(errors.is_empty(), "\n{}", errors.join("\n"));
}
