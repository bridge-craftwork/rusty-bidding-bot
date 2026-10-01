//! The rules and stock cards, compiled into the binary.
//!
//! `RULE_FILES` holds every `conventions/**/*.bid`, named and ordered as
//! `rbb_engine::load_rules("conventions")` names and orders them, `MANIFEST`
//! the manifest (`conventions/conventions.toml`), and
//! `FIELDS` / `BBSA_MAP` the card vocabulary (the standard one, convention-card's
//! `spec/` via `bridge_card::standard`, unless the rules bring their own), so
//! `rbb_engine::compile_rules(MANIFEST, FIELDS, BBSA_MAP, RULE_FILES)` gives the same
//! rule set as loading the directory. `CARDS` holds the stock `.bbsa` cards
//! (`cards/bbsa`), which are read in that
//! vocabulary. Nothing here needs a filesystem at run time: this is what
//! the release `rbb` and the WASM build bid with.

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
        assert!(RULE_FILES
            .iter()
            .all(|(n, _)| n.starts_with("conventions/")));
        assert_eq!(MANIFEST.unwrap().0, "conventions/conventions.toml");
        assert_eq!(FIELDS.0, "convention-card/spec/fields.toml");
        assert_eq!(BBSA_MAP.0, "convention-card/spec/formats/bbsa-map.toml");
        assert!(FIELDS.1.contains("[notrump"));
        assert!(card("21GF-DEFAULT").is_some());
        assert_eq!(RULES_ID.len(), 16);
    }
}
