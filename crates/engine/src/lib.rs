//! Selects the active convention modules from a card and interprets them:
//! chooses a call for a hand, and infers what earlier calls showed.
//!
//! See docs/LANGUAGE.md for the model: calls are interpreted one at a time
//! into knowledge about each hand and state for each side; a call is chosen
//! by ranking the candidate rules (priority, descriptiveness, prefer, file
//! order) among those the hand satisfies.

pub mod alerting;
pub mod cases;
mod contract;
mod engine;
mod eval;
pub mod explain;
mod facts;
mod knowledge;
mod macros;
pub mod meanings;
mod position;
mod review;
mod sample;
mod system;

use std::path::Path;

/// How many threads a parallel job of ours may use: `RBB_CPUS` when set,
/// otherwise every core but two, so the machine stays usable for other
/// work while a comparison or test run is going (Rick, 2026-10-05).
pub fn cpu_budget() -> usize {
    std::env::var("RBB_CPUS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map_or(1, |n| n.get())
                .saturating_sub(2)
        })
        .max(1)
}

pub use alerting::{AlertTable, Regulator};
pub use engine::{
    check_sets, CandidateTrace, Choice, DealAuction, DealCall, Decision, Engine, Heard, HeardRead,
    Interpretation, OnNoRule, RuleListing, RuleOption, RuleRejection, Step, StopReason, Table,
    TableAuction, TableStop, SET_KEYS,
};
pub use explain::Audience;
pub use facts::{Facts, Valuation};
pub use knowledge::{Range, SeatKnowledge, Tri};
pub use position::{side, Ask, Forcing, Position, SideState};
pub use review::{
    heard_text, knowledge_full, knowledge_parts, review_text, side_text, strain_symbol, FitView,
    Flags, Narrowed, OwnView, Part, ReviewRow, Role,
};

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
    check_manifest, read_manifest, snapshot_manifest, splice_terms, LANGUAGE_VERSION,
    OLDEST_LANGUAGE_VERSION, TERMS_BEGIN, TERMS_END,
};
pub use eval::{check_terms, terms_reference, Term};
pub use system::{check_card_refs, RuleRef, System};

pub use bidspec::manifest::Manifest;
pub use bridge_card::Vocabulary;
pub use meanings::{AnnouncedNames, Meanings};

/// A rule set: its manifest (`conventions.toml`, when it has one), the card
/// vocabulary its modules were checked against, and the compiled modules.
/// The vocabulary is the standard one (the convention-card repository's
/// `spec/`, pinned by tag in Cargo.toml) unless the rules directory brings
/// its own (`card/fields.toml` and `card/bbsa-map.toml`); cards an engine
/// plays must be read in it (`rules.vocab`).
#[derive(Debug, Clone)]
pub struct RuleSet {
    pub manifest: Option<Manifest>,
    pub vocab: Vocabulary,
    pub modules: Vec<bidspec::Module>,
    /// The pack's announced convention names (`announced-names.toml`;
    /// empty for rules that bring none): what an opponent's announced
    /// name switches on in their card (`Engine::interpret_with`).
    pub announced: AnnouncedNames,
    /// The pack's `alerts.toml` read against these modules: which calls a
    /// regulator alerts or announces (`Engine::with_regulator`). Empty for
    /// rules that bring none: then every call is alerted as its rule's
    /// flag says.
    pub alerts: std::sync::Arc<AlertTable>,
}

impl RuleSet {
    /// This rule set alerting by `alerts` (`alerts.toml`), resolved
    /// against its modules.
    pub fn with_alerts(mut self, alerts: bidspec::alerts::Alerts) -> RuleSet {
        self.alerts = std::sync::Arc::new(AlertTable::new(alerts, &self.modules, &self.vocab));
        self
    }

    /// This rule set reading the opponents' announced names with `names`.
    pub fn with_announced_names(mut self, names: AnnouncedNames) -> RuleSet {
        self.announced = names;
        self
    }
}

/// The announced-names table of a pack checkout (`announced-names.toml`),
/// empty when it has none. A file that does not parse is an error; an
/// entry whose context does not parse is left out (`problems`).
pub fn read_announced_names(dir: &Path) -> Result<AnnouncedNames, String> {
    let f = dir.join(meanings::FILE);
    match std::fs::read_to_string(&f) {
        Ok(text) => AnnouncedNames::from_toml(&text).map_err(|e| format!("{}: {e}", f.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AnnouncedNames::default()),
        Err(e) => Err(format!("{}: {e}", f.display())),
    }
}

/// The announced-names table of a pack snapshot (`pack.json`'s
/// `announced_names`), empty when it has none: older snapshots do not.
pub fn snapshot_announced_names(text: &str) -> Result<AnnouncedNames, String> {
    let doc: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    match doc.get(meanings::SNAPSHOT_KEY) {
        Some(v) if !v.is_null() => AnnouncedNames::from_json(v),
        _ => Ok(AnnouncedNames::default()),
    }
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

/// The card vocabulary for the rules in `dir`: the directory's own
/// (`card/fields.toml`, `card/bbsa-map.toml`) when it has one, otherwise
/// the standard vocabulary built into `bridge_card` (convention-card's
/// `spec/`, at the tag Cargo.toml pins).
pub fn rules_vocabulary(dir: &Path) -> Result<Vocabulary, bridge_card::Error> {
    if dir.is_file() {
        // A pack's published snapshot (`pack.json`).
        let name = dir.display().to_string();
        let text = std::fs::read_to_string(dir).map_err(|e| bridge_card::Error {
            message: e.to_string(),
            file: Some(name),
            line: None,
        })?;
        Vocabulary::from_snapshot(&text, bridge_card::standard::BBSA_MAP)
    } else if bridge_card::Pack::is_pack(dir) {
        // A convention pack (bridge-common-conventions): conventions,
        // treatments and settings named by address, with card schema 2's
        // paths as aliases. The `.bbsa` map is still convention-card's.
        Vocabulary::load_pack(dir, bridge_card::standard::BBSA_MAP)
    } else if dir.join(Vocabulary::FIELDS).exists() {
        Vocabulary::load(dir)
    } else {
        bridge_card::standard::vocabulary()
    }
}

/// The definitions a rule module's `skill` lines may name, for the rules in
/// `dir`: a snapshot's own (`pack.json`), a pack checkout's own
/// (`conventions/` and `skills/`), otherwise the ones built into
/// `bridge_card`. As [`rules_vocabulary`] chooses the card vocabulary.
pub fn rules_definitions(dir: &Path) -> Result<bridge_card::Skills, bridge_card::Error> {
    if dir.is_file() {
        let name = dir.display().to_string();
        let text = std::fs::read_to_string(dir).map_err(|e| bridge_card::Error {
            message: e.to_string(),
            file: Some(name),
            line: None,
        })?;
        bridge_card::Skills::from_snapshot(&text)
    } else if bridge_card::Pack::is_pack(dir) {
        bridge_card::Skills::load_pack(dir)
    } else {
        bridge_card::standard::conventions()
    }
}

/// Load the rule set in `dir`: its manifest (a language version this
/// engine does not read refuses the load; a directory without one is read
/// as the current language), its card vocabulary ([`rules_vocabulary`]), then every
/// `.bid` file under it, compiled against that vocabulary and sorted by
/// path (which fixes the file-order tie-breaker).
pub fn load_rules(dir: &Path) -> Result<RuleSet, Vec<bidspec::Diagnostic>> {
    if dir.is_file() {
        let name = dir.display().to_string();
        let text = std::fs::read_to_string(dir).map_err(|e| {
            vec![bidspec::Diagnostic {
                file: name.clone(),
                line: 0,
                col: 0,
                message: e.to_string(),
            }]
        })?;
        return compile_snapshot(&text, &name);
    }
    let manifest = read_manifest(dir).map_err(|d| vec![d])?;
    let vocab = rules_vocabulary(dir).map_err(|e| vec![vocab_diagnostic(e)])?;
    let announced = read_announced_names(dir).map_err(|message| {
        vec![bidspec::Diagnostic {
            file: dir.join(meanings::FILE).display().to_string(),
            line: 0,
            col: 0,
            message,
        }]
    })?;
    let alerts = bidspec::alerts::load(dir).map_err(|message| {
        vec![bidspec::Diagnostic {
            file: dir.join(bidspec::alerts::FILE).display().to_string(),
            line: 0,
            col: 0,
            message,
        }]
    })?;
    compile_dir(dir, &vocab).map(|modules| {
        let set = RuleSet {
            manifest,
            vocab,
            modules,
            announced,
            alerts: Default::default(),
        };
        match alerts {
            Some(a) => set.with_alerts(a),
            None => set,
        }
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
        announced: AnnouncedNames::default(),
        alerts: Default::default(),
    })
}

/// The rule files in a pack's published snapshot, `(name, source)` in the
/// order a directory of them is read: by path, component by component.
pub fn snapshot_rule_files(doc: &serde_json::Value) -> Vec<(&str, &str)> {
    let mut files: Vec<(&str, &str)> = doc
        .get("files")
        .and_then(|f| f.as_object())
        .into_iter()
        .flatten()
        .filter(|(name, _)| name.ends_with(".bid"))
        .filter_map(|(name, text)| Some((name.as_str(), text.as_str()?)))
        .collect();
    files.sort_by(|a, b| a.0.split('/').cmp(b.0.split('/')));
    files
}

/// The rule set in a pack's published snapshot (`pack.json`, as
/// bridge-common-conventions' `published` branch carries it): its manifest,
/// its vocabulary and every rule file, as [`load_rules`] gives them for the
/// pack's source tree. `name` is the snapshot's, for diagnostics. Needs no
/// filesystem: what the WASM build and a host that fetched the snapshot use.
pub fn compile_snapshot(text: &str, name: &str) -> Result<RuleSet, Vec<bidspec::Diagnostic>> {
    let manifest = snapshot_manifest(text, name).map_err(|d| vec![d])?;
    let vocab =
        Vocabulary::from_snapshot(text, bridge_card::standard::BBSA_MAP).map_err(|mut e| {
            e.file = Some(name.to_string());
            vec![vocab_diagnostic(e)]
        })?;
    let doc: serde_json::Value = serde_json::from_str(text).map_err(|e| {
        vec![bidspec::Diagnostic {
            file: name.to_string(),
            line: 0,
            col: 0,
            message: e.to_string(),
        }]
    })?;
    // Optional: a snapshot without the table (built before it existed)
    // reads no names, and one whose table does not parse is refused.
    let announced = match doc.get(meanings::SNAPSHOT_KEY) {
        Some(v) if !v.is_null() => AnnouncedNames::from_json(v).map_err(|message| {
            vec![bidspec::Diagnostic {
                file: name.to_string(),
                line: 0,
                col: 0,
                message,
            }]
        })?,
        _ => AnnouncedNames::default(),
    };
    // Optional too: the snapshot's `alerts` (alerts.toml as JSON).
    let alerts = bidspec::alerts::from_snapshot(&doc).map_err(|message| {
        vec![bidspec::Diagnostic {
            file: name.to_string(),
            line: 0,
            col: 0,
            message: format!("alerts: {message}"),
        }]
    })?;
    compile_modules(&vocab, snapshot_rule_files(&doc)).map(|modules| {
        let set = RuleSet {
            manifest: Some(manifest),
            vocab,
            modules,
            announced,
            alerts: Default::default(),
        };
        match alerts {
            Some(a) => set.with_alerts(a),
            None => set,
        }
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
    if dir.is_file() {
        let name = dir.display().to_string();
        let fail = |message: String| {
            vec![bidspec::Diagnostic {
                file: name.clone(),
                line: 0,
                col: 0,
                message,
            }]
        };
        let text = std::fs::read_to_string(dir).map_err(|e| fail(e.to_string()))?;
        let doc: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| fail(e.to_string()))?;
        return compile_modules(vocab, snapshot_rule_files(&doc));
    }
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
        // Rule files are named with `/` on every platform, so rule
        // references (`conventions/base/rebids.bid:58`), JSON and `--rule`
        // specs read the same on Windows.
        let mut name = f.display().to_string();
        if cfg!(windows) {
            name = name.replace('\\', "/");
        }
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
