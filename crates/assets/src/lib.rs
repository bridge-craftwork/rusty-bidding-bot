//! The rules and stock cards, compiled into the binary.
//!
//! `RULE_FILES` holds every `conventions/**/*.bid`, named and ordered as
//! `rbb_engine::load_modules("conventions")` names and orders them, so
//! `rbb_engine::compile_modules(RULE_FILES)` gives the same modules as
//! loading the directory. `CARDS` holds the stock `.bbsa` cards
//! (`crates/bridge-card/tests/fixtures/bbsa`). Nothing here needs a
//! filesystem at run time: this is what the release `rbb` and the WASM
//! build bid with.

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// The stock card called `name` (e.g. `21GF-DEFAULT`), as `.bbsa` text.
pub fn card(name: &str) -> Option<&'static str> {
    CARDS.iter().find(|(n, _)| *n == name).map(|(_, t)| *t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_and_cards_are_embedded() {
        assert!(RULE_FILES.len() > 10);
        assert!(RULE_FILES.iter().all(|(n, _)| n.starts_with("conventions/")));
        assert!(card("21GF-DEFAULT").is_some());
        assert_eq!(RULES_ID.len(), 16);
    }
}
