//! `rbb card coverage`: loading the card files. The classification itself
//! is `bidspec::coverage`, shared with the WASM build.

use std::path::Path;

use bridge_card::{bbsa, Card, Vocabulary};

pub use bidspec::coverage::{fields_read, of_card};

/// Load a card in `vocab` (the rules' vocabulary) from a `.bbsa` or a card
/// JSON file, with the `.bbsa` keys that had nowhere to go.
pub fn load(vocab: &Vocabulary, path: &Path) -> Result<(String, Card, Vec<String>), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let name = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if path.extension().is_some_and(|e| e == "bbsa") {
        let (card, report) = bbsa::import(vocab, &text, Some(&name)).map_err(|e| e.to_string())?;
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
        let (card, _) = Card::from_json(vocab, &text).map_err(|e| e.to_string())?;
        Ok((name, card, Vec::new()))
    }
}
