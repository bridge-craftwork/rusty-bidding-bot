//! Selects the active convention modules from a card and interprets them:
//! chooses a call for a hand, and infers what earlier calls showed.
//!
//! See docs/LANGUAGE.md for the model: calls are interpreted one at a time
//! into knowledge about each hand and state for each side; a call is chosen
//! by ranking the candidate rules (priority, descriptiveness, prefer, file
//! order) among those the hand satisfies.

pub mod cases;
mod engine;
mod eval;
mod facts;
mod knowledge;
mod position;
mod sample;
mod system;

use std::path::Path;

pub use engine::{
    CandidateTrace, Choice, DealAuction, DealCall, Decision, Engine, Interpretation, Step,
};
pub use facts::{Facts, Valuation};
pub use knowledge::{Range, SeatKnowledge, Tri};
pub use position::{side, Ask, Forcing, Position, SideState};

/// The playing suit of a strain (`None` for notrump).
pub fn suit_of(strain: bridge_types::Strain) -> Option<bridge_types::Suit> {
    use bridge_types::{Strain, Suit};
    match strain {
        Strain::Clubs => Some(Suit::Clubs),
        Strain::Diamonds => Some(Suit::Diamonds),
        Strain::Hearts => Some(Suit::Hearts),
        Strain::Spades => Some(Suit::Spades),
        Strain::NoTrump => None,
    }
}
pub use eval::check_terms;
pub use system::{check_card_refs, RuleRef, System};

/// Compile every `.bid` file under `dir`, sorted by path (which fixes the
/// file-order tie-breaker).
pub fn load_modules(dir: &Path) -> Result<Vec<bidspec::Module>, Vec<bidspec::Diagnostic>> {
    fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    collect(&p, out);
                } else if p.extension().is_some_and(|x| x == "bid") {
                    out.push(p);
                }
            }
        }
    }
    let mut files = Vec::new();
    collect(dir, &mut files);
    files.sort();
    let mut sources = Vec::new();
    let mut diags = Vec::new();
    for f in files {
        let name = f.display().to_string();
        match std::fs::read_to_string(&f) {
            Ok(src) => sources.push((name, src)),
            Err(e) => diags.push(bidspec::Diagnostic {
                file: name,
                line: 0,
                col: 0,
                message: e.to_string(),
            }),
        }
    }
    match compile_modules(sources.iter().map(|(n, s)| (n.as_str(), s.as_str()))) {
        Ok(modules) if diags.is_empty() => Ok(modules),
        Ok(_) => Err(diags),
        Err(d) => {
            diags.extend(d);
            Err(diags)
        }
    }
}

/// Compile rule files given as `(name, source)`, in the order given (which
/// fixes the file-order tie-breaker): what `load_modules` does once it has
/// read the files, for callers without a filesystem (the WASM build, the
/// rules embedded by `rbb-assets`).
pub fn compile_modules<'a>(
    sources: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<Vec<bidspec::Module>, Vec<bidspec::Diagnostic>> {
    let mut modules = Vec::new();
    let mut diags = Vec::new();
    for (name, src) in sources {
        match bidspec::compile(src, name) {
            Ok(m) => modules.push(m),
            Err(d) => diags.extend(d),
        }
    }
    // A name the engine does not know would otherwise make its condition
    // false every time it is tried: refuse to load instead.
    if diags.is_empty() {
        diags = check_terms(&modules);
    }
    if diags.is_empty() {
        Ok(modules)
    } else {
        Err(diags)
    }
}
