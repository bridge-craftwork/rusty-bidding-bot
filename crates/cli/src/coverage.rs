//! How much of a convention card our rules actually read.
//!
//! A card is a few hundred switches; the rules read a few dozen. The gap is
//! what the engine silently ignores when it bids that card, which is the
//! difference between "we disagree with BBA" and "we were never playing the
//! same system". Three buckets per setting that is switched on:
//!
//! - **read**: a module names it in a `card` line or a `param`, or a
//!   `[[derived]]` rule turns it into one that does;
//! - **ignored**: the field exists and nothing reads it;
//! - **unmapped**: the `.bbsa` key has no card field at all, so the setting
//!   never reaches our model (it is kept in `bba_passthrough` so an export
//!   round-trips). Only keys the card switches *on* are counted, to match
//!   the other two buckets.
//!
//! Carding, opening leads and free-text notes cannot change a call, so they
//! are counted apart from the conventions.

use std::collections::BTreeSet;
use std::path::Path;

use bridge_card::{bbsa, registry, Card, Value};

/// Sections that describe the play or the card itself, not the bidding.
const NOT_BIDDING: [&str; 4] = ["carding", "leads", "notes", "metadata"];

pub struct Coverage {
    pub name: String,
    pub system: String,
    pub read: Vec<String>,
    pub ignored: Vec<String>,
    pub other: Vec<String>,
    pub unmapped: Vec<String>,
}

impl Coverage {
    /// Of the bidding settings this card switches on, the share our rules
    /// read.
    pub fn score(&self) -> f64 {
        let on = self.read.len() + self.ignored.len();
        if on == 0 {
            1.0
        } else {
            self.read.len() as f64 / on as f64
        }
    }
}

/// Every card path the rules name, directly or through a derivation.
pub fn fields_read(modules: &[bidspec::Module]) -> BTreeSet<String> {
    let mut read: BTreeSet<String> = BTreeSet::new();
    for m in modules {
        read.extend(m.card.iter().map(|c| c.path.clone()));
        read.extend(m.params.iter().map(|p| p.path.clone()));
    }
    // A field feeding a derivation counts as read when the derived field is.
    for (inputs, outputs) in bbsa::derivations() {
        if outputs.iter().any(|o| read.contains(o)) {
            read.extend(inputs);
        }
    }
    read
}

/// Classify what `card` switches on against what `read` names.
pub fn of_card(
    name: &str,
    card: &Card,
    unmapped: Vec<String>,
    read: &BTreeSet<String>,
) -> Coverage {
    let reg = registry();
    let mut cov = Coverage {
        name: name.to_string(),
        system: match card.effective("general.system_category") {
            Some(Value::Text(s)) => s.clone(),
            _ => "two_over_one".into(),
        },
        read: Vec::new(),
        ignored: Vec::new(),
        other: Vec::new(),
        unmapped,
    };
    for (path, value) in card.values() {
        if !is_on(value) {
            continue;
        }
        // A field at its default says nothing about this card.
        if reg.get(path).and_then(|f| f.default.as_ref()) == Some(value) {
            continue;
        }
        let bucket = if NOT_BIDDING.iter().any(|s| path.starts_with(s)) {
            &mut cov.other
        } else if read.contains(path) {
            &mut cov.read
        } else {
            &mut cov.ignored
        };
        bucket.push(path.to_string());
    }
    cov
}

/// Is this setting saying anything? `false`, an empty string and an unset
/// value are all "not switched on".
fn is_on(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Text(s) => !s.is_empty() && s != "none",
        Value::Int(_) => true,
    }
}

/// Load a card from a `.bbsa` or a card JSON file, with the `.bbsa` keys
/// that had nowhere to go.
pub fn load(path: &Path) -> Result<(String, Card, Vec<String>), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let name = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if path.extension().is_some_and(|e| e == "bbsa") {
        let (card, report) = bbsa::import(&text, Some(&name)).map_err(|e| e.to_string())?;
        // A key set to 0 says the card does not play it: nothing is lost by
        // having nowhere to put it.
        let unmapped = report
            .passthrough
            .iter()
            .filter(|(_, v)| *v != 0)
            .map(|(k, _)| k.clone())
            .collect();
        Ok((name, card, unmapped))
    } else {
        let (card, _) = Card::from_json(&text).map_err(|e| e.to_string())?;
        Ok((name, card, Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modules(src: &str) -> Vec<bidspec::Module> {
        vec![bidspec::parse(src, "test.bid").expect("parses")]
    }

    #[test]
    fn a_field_is_read_directly_or_through_a_derivation() {
        let read = fields_read(&modules(concat!(
            "module t \"t\"\n",
            "  card   notrump.stayman.play\n",
            "  param  minors = notrump.minor_transfers default relay\n",
            "\nafter 1N (P)\n  2C \"Stayman\" shows hcp>=8\n"
        )));
        assert!(read.contains("notrump.stayman.play"));
        assert!(read.contains("notrump.minor_transfers"));
        // `minor_transfers` is derived from BBA's four switches, so those
        // count as read even though no rule names them.
        assert!(
            read.contains("notrump.transfers.two_s_clubs"),
            "derivation inputs count as read: {read:?}"
        );
        assert!(!read.contains("competitive.michaels.play"));
    }

    #[test]
    fn settings_split_into_read_ignored_and_play_only() {
        let read = fields_read(&modules(concat!(
            "module t \"t\"\n",
            "  card   notrump.stayman.play\n",
            "\nafter 1N (P)\n  2C \"Stayman\" shows hcp>=8\n"
        )));
        let mut card = Card::new();
        card.set("notrump.stayman.play", Value::Bool(true)).unwrap();
        card.set("competitive.michaels.play", Value::Bool(true))
            .unwrap();
        card.set("carding.smith_echo", Value::Bool(true)).unwrap();
        card.set("competitive.ghestem.play", Value::Bool(false))
            .unwrap();
        let cov = of_card("t", &card, vec!["Some Key".into()], &read);
        assert_eq!(cov.read, ["notrump.stayman.play"]);
        assert_eq!(cov.ignored, ["competitive.michaels.play"]);
        assert_eq!(cov.other, ["carding.smith_echo"], "play, not bidding");
        assert_eq!(cov.unmapped.len(), 1);
        assert_eq!(cov.score(), 0.5);
    }
}
