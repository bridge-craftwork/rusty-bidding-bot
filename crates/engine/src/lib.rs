//! Selects the active convention modules from a card and interprets them:
//! chooses a call for a hand, and infers what earlier calls showed.
//!
//! See docs/LANGUAGE.md for the model: calls are interpreted one at a time
//! into knowledge about each hand and state for each side; a call is chosen
//! by ranking the candidate rules (priority, descriptiveness, prefer, file
//! order) among those the hand satisfies.

pub mod cases;
mod contract;
mod engine;
mod eval;
mod facts;
mod knowledge;
mod position;
mod sample;
mod system;

use std::path::Path;

pub use engine::{
    check_sets, CandidateTrace, Choice, DealAuction, DealCall, Decision, Engine, Interpretation,
    OnNoRule, Step, StopReason, Table, TableAuction, TableStop, SET_KEYS,
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
pub use contract::{
    check_manifest, read_manifest, splice_terms, LANGUAGE_VERSION, OLDEST_LANGUAGE_VERSION,
    TERMS_BEGIN, TERMS_END,
};
pub use eval::{check_terms, terms_reference, Term};
pub use system::{check_card_refs, RuleRef, System};

pub use bidspec::manifest::Manifest;
pub use bridge_card::Vocabulary;

/// A rule set: its manifest (`conventions.toml`, when it has one), the card
/// vocabulary its modules were checked against (`card/fields.toml` and
/// `card/bbsa-map.toml` in the rules directory), and the compiled modules.
/// Every rules directory brings its own vocabulary: cards an engine plays
/// must be read in it (`rules.vocab`).
#[derive(Debug, Clone)]
pub struct RuleSet {
    pub manifest: Option<Manifest>,
    pub vocab: Vocabulary,
    pub modules: Vec<bidspec::Module>,
}

/// A vocabulary problem as a rule diagnostic (file and line when known).
fn vocab_diagnostic(e: bridge_card::Error) -> bidspec::Diagnostic {
    bidspec::Diagnostic {
        file: e.file.unwrap_or_default(),
        line: e.line.unwrap_or(0),
        col: 0,
        message: e.message,
    }
}

/// Load the rule set in `dir`: its manifest (a language version this
/// engine does not read refuses the load; a directory without one is read
/// as the current language), its card vocabulary (`card/`), then every
/// `.bid` file under it, compiled against that vocabulary and sorted by
/// path (which fixes the file-order tie-breaker).
pub fn load_rules(dir: &Path) -> Result<RuleSet, Vec<bidspec::Diagnostic>> {
    let manifest = read_manifest(dir).map_err(|d| vec![d])?;
    let vocab = Vocabulary::load(dir).map_err(|e| vec![vocab_diagnostic(e)])?;
    compile_dir(dir, &vocab).map(|modules| RuleSet {
        manifest,
        vocab,
        modules,
    })
}

/// The rule set given as text, for callers without a filesystem (the WASM
/// build, the rules embedded by `rbb-assets`): what `load_rules` does once
/// it has read the files. Each file is `(name, text)`; the names are used
/// in diagnostics. The rule files are compiled in the order given (which
/// fixes the file-order tie-breaker). With no manifest the rules are read
/// as the current language.
pub fn compile_rules<'a>(
    manifest: Option<(&str, &str)>,
    (fields_name, fields): (&str, &str),
    (bbsa_map_name, bbsa_map): (&str, &str),
    sources: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<RuleSet, Vec<bidspec::Diagnostic>> {
    let manifest = manifest
        .map(|(name, text)| check_manifest(text, name))
        .transpose()
        .map_err(|d| vec![d])?;
    let vocab = Vocabulary::parse(fields, bbsa_map).map_err(|mut e| {
        e.file = Some(match e.file.as_deref() {
            Some("bbsa-map.toml") => bbsa_map_name.to_string(),
            _ => fields_name.to_string(),
        });
        vec![vocab_diagnostic(e)]
    })?;
    compile_modules(&vocab, sources).map(|modules| RuleSet {
        manifest,
        vocab,
        modules,
    })
}

/// Compile every `.bid` file under `dir` against `vocab`, sorted by path
/// (which fixes the file-order tie-breaker). A manifest (`conventions.toml`)
/// in `dir` that asks for a rule language this engine does not read
/// refuses the load; a directory without one is read as the current
/// language. `load_rules` does this and loads the vocabulary too.
pub fn load_modules(
    dir: &Path,
    vocab: &Vocabulary,
) -> Result<Vec<bidspec::Module>, Vec<bidspec::Diagnostic>> {
    read_manifest(dir).map_err(|d| vec![d])?;
    compile_dir(dir, vocab)
}

fn compile_dir(
    dir: &Path,
    vocab: &Vocabulary,
) -> Result<Vec<bidspec::Module>, Vec<bidspec::Diagnostic>> {
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
    match compile_modules(vocab, sources.iter().map(|(n, s)| (n.as_str(), s.as_str()))) {
        Ok(modules) if diags.is_empty() => Ok(modules),
        Ok(_) => Err(diags),
        Err(d) => {
            diags.extend(d);
            Err(diags)
        }
    }
}

/// Compile rule files given as `(name, source)` against `vocab`, in the
/// order given (which fixes the file-order tie-breaker): what
/// `load_modules` does once it has read the files.
pub fn compile_modules<'a>(
    vocab: &bridge_card::Vocabulary,
    sources: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<Vec<bidspec::Module>, Vec<bidspec::Diagnostic>> {
    let mut modules = Vec::new();
    let mut diags = Vec::new();
    for (name, src) in sources {
        match bidspec::compile(src, name, vocab.registry()) {
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
