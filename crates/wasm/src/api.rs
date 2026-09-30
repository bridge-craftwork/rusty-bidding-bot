//! The WASM API as plain Rust, so it is tested natively. Every function takes
//! a JSON request (a string) and returns a JSON response (a string), except
//! `reference`, which returns plain text. Nothing panics or throws on bad
//! input: problems come back as `diagnostics`, each
//! `{severity, message, line?, col?, hint?}`, and `ok` is false when one of
//! them is an error. docs/WASM.md is the contract.

use std::cell::RefCell;
use std::sync::{Arc, OnceLock};

use bridge_card::{bbsa, Card, Vocabulary};
use bridge_types::{
    Auction, Call, Contract, Deal, Direction, Doubled, Hand, ScoringMethod, Strain, Vulnerability,
};
use rbb_engine::{
    Ask, CandidateTrace, DealCall, Engine, Forcing, OnNoRule, Position, RuleRef, RuleSet,
    SeatKnowledge, SideState, Step, StopReason, Table, Tri,
};
use serde::Serialize;
use serde_json::{json, Map, Value as Json};

/// Bumped when a response or request shape changes incompatibly.
pub const API_VERSION: u32 = 1;

/// Longest auction `bid_deal` bids before it stops (as `rbb compare`).
const MAX_CALLS: usize = 60;

// ── Diagnostics ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// One problem or remark, in the shape the bridge-craftwork tool contract
/// uses. `line` and `col` are 1-based and point into the request field the
/// message names (the auction string, the `.bbsa` text), or, when `file` is
/// set, into that rule file (a key of the request's `rules`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Default)]
struct Diags(Vec<Diagnostic>);

impl Diags {
    fn push(&mut self, severity: Severity, message: String) -> &mut Diagnostic {
        self.0.push(Diagnostic {
            severity,
            message,
            file: None,
            line: None,
            col: None,
            hint: None,
        });
        self.0.last_mut().unwrap()
    }
    fn error(&mut self, message: impl Into<String>) -> &mut Diagnostic {
        self.push(Severity::Error, message.into())
    }
    fn warning(&mut self, message: impl Into<String>) -> &mut Diagnostic {
        self.push(Severity::Warning, message.into())
    }
    fn info(&mut self, message: impl Into<String>) -> &mut Diagnostic {
        self.push(Severity::Info, message.into())
    }
    fn has_errors(&self) -> bool {
        self.0.iter().any(|d| d.severity == Severity::Error)
    }
}

impl Diagnostic {
    fn hint(&mut self, h: impl Into<String>) -> &mut Self {
        self.hint = Some(h.into());
        self
    }
    fn at(&mut self, line: Option<usize>, col: Option<usize>) -> &mut Self {
        self.line = line;
        self.col = col;
        self
    }
}

/// A response: `ok`, the fields in `body`, and `diagnostics`.
fn respond(body: Json, d: Diags) -> String {
    let mut out = Map::new();
    out.insert("ok".into(), Json::Bool(!d.has_errors()));
    if let Json::Object(m) = body {
        out.extend(m);
    }
    out.insert("diagnostics".into(), serde_json::to_value(&d.0).unwrap());
    Json::Object(out).to_string()
}

fn parse_request(text: &str, d: &mut Diags) -> Json {
    if text.trim().is_empty() {
        return Json::Object(Map::new());
    }
    match serde_json::from_str::<Json>(text) {
        Ok(v @ Json::Object(_)) => v,
        Ok(_) => {
            d.error("request: expected a JSON object")
                .hint("e.g. {\"engine\": 1, \"hand\": \"AK52.KJ7.Q94.K83\"}");
            Json::Object(Map::new())
        }
        Err(e) => {
            d.error(format!("request is not valid JSON: {e}"))
                .at(Some(e.line()), Some(e.column()));
            Json::Object(Map::new())
        }
    }
}

// ── Rules and engines ──────────────────────────────────────────────────────

/// A compiled rule set: the modules and the card vocabulary that cards are
/// read in, with the id responses report (`rules_id`).
struct Rules {
    set: RuleSet,
    id: String,
    files: usize,
}

/// The embedded rules and their vocabulary, compiled once.
fn embedded_rules() -> &'static Result<Arc<Rules>, Vec<bidspec::Diagnostic>> {
    static RULES: OnceLock<Result<Arc<Rules>, Vec<bidspec::Diagnostic>>> = OnceLock::new();
    RULES.get_or_init(|| {
        rbb_engine::compile_rules(
            rbb_assets::MANIFEST,
            rbb_assets::FIELDS,
            rbb_assets::BBSA_MAP,
            rbb_assets::RULE_FILES.iter().copied(),
        )
        .map(|set| {
            Arc::new(Rules {
                set,
                id: rbb_assets::RULES_ID.to_string(),
                files: rbb_assets::RULE_FILES.len(),
            })
        })
    })
}

/// How many rules the modules hold.
fn rule_count(modules: &[bidspec::Module]) -> usize {
    bidspec::reference::entries(modules)
        .iter()
        .map(|m| m.rules.len())
        .sum()
}

fn embedded_or_report(d: &mut Diags) -> Option<Arc<Rules>> {
    match embedded_rules() {
        Ok(r) => Some(r.clone()),
        Err(errors) => {
            for e in errors {
                d.error(format!("embedded rules: {e}"));
            }
            None
        }
    }
}

/// The request fields that supply a rule set at run time.
const RULE_FIELDS: [&str; 4] = ["rules", "fields", "bbsa_map", "manifest"];

/// FNV-1a, for the ids of rule sets supplied at run time.
fn fnv(h: &mut u64, bytes: &[u8]) {
    for b in bytes {
        *h ^= *b as u64;
        *h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

/// A rule-loading diagnostic as an API diagnostic. Problems in a rule file
/// name it (`file`, and `rules: <file>:<line>:` in the message); problems in
/// `fields`, `bbsa_map` or `manifest` start with that field's name.
fn rule_diagnostic(x: &bidspec::Diagnostic, severity: Severity, d: &mut Diags) {
    let line = (x.line > 0).then_some(x.line);
    let col = (x.col > 0).then_some(x.col);
    // The embedded file standing in for a field the request left out (the
    // embedded `.bbsa` map does not fit the fields it gave, say).
    let embedded = [
        ("fields", Some(rbb_assets::FIELDS.0)),
        ("bbsa_map", Some(rbb_assets::BBSA_MAP.0)),
        ("manifest", rbb_assets::MANIFEST.map(|m| m.0)),
    ]
    .into_iter()
    .find(|(_, name)| *name == Some(x.file.as_str()));
    let diag = if RULE_FIELDS[1..].contains(&x.file.as_str()) {
        d.push(severity, format!("{}: {}", x.file, x.message))
    } else if let Some((field, _)) = embedded {
        let diag = d.push(
            severity,
            format!(
                "{field}: the embedded {} (not given): {}",
                x.file, x.message
            ),
        );
        diag.file = Some(x.file.clone());
        diag
    } else {
        let at = match line {
            Some(l) => format!("{}:{l}", x.file),
            None => x.file.clone(),
        };
        let diag = d.push(severity, format!("rules: {at}: {}", x.message));
        diag.file = Some(x.file.clone());
        diag
    };
    diag.at(line, col);
}

thread_local! {
    /// Rule sets supplied at run time, by id: the last few, so requests
    /// that repeat one (with other cards, or `conventions` after
    /// `createEngine`) do not compile it again.
    static RULE_SETS: RefCell<Vec<Arc<Rules>>> = const { RefCell::new(Vec::new()) };
}
const RULE_SETS_KEPT: usize = 4;

/// The rules a request asks for, when it names no engine: the embedded
/// rules, or a rule set supplied at run time with `rules` (`{"path.bid":
/// source, ...}`), `fields` (the text of `fields.toml`), `bbsa_map`
/// (`bbsa-map.toml`) and `manifest` (`conventions.toml`). What is not
/// supplied comes from the embedded rules, except the manifest when `rules`
/// is given: rules with no manifest are read as the current rule language
/// (an `info` says so), as `rbb` reads a directory without one.
fn rules_for(req: &Json, d: &mut Diags) -> Option<Arc<Rules>> {
    if RULE_FIELDS.iter().all(|f| req.get(f).is_none()) {
        return embedded_or_report(d);
    }
    let text = |field: &str, d: &mut Diags| -> Option<Option<String>> {
        match req.get(field) {
            None | Some(Json::Null) => Some(None),
            Some(Json::String(s)) => Some(Some(s.clone())),
            Some(_) => {
                d.error(format!("{field}: expected the file's text as a string"));
                None
            }
        }
    };
    let fields = text("fields", d)?;
    let bbsa_map = text("bbsa_map", d)?;
    let manifest = text("manifest", d)?;
    let mut files: Vec<(String, String)> = match req.get("rules") {
        None | Some(Json::Null) => rbb_assets::RULE_FILES
            .iter()
            .map(|(n, s)| (n.to_string(), s.to_string()))
            .collect(),
        Some(Json::Object(m)) => {
            let mut files = Vec::new();
            let mut other = Vec::new();
            for (path, v) in m {
                match v.as_str() {
                    Some(_) if !path.ends_with(".bid") => other.push(path.as_str()),
                    Some(src) => files.push((path.clone(), src.to_string())),
                    None => {
                        d.error(format!(
                            "rules.{path}: expected the file's text as a string"
                        ));
                    }
                }
            }
            if !other.is_empty() {
                d.info(format!(
                    "rules: {} files that are not .bid ignored: {}",
                    other.len(),
                    other.join(", ")
                ));
            }
            if files.is_empty() {
                d.warning("rules: no .bid files: every call will be a pass");
            }
            files
        }
        Some(_) => {
            d.error("rules: expected an object {\"path/file.bid\": \"source\", ...}")
                .hint("the keys name the files in diagnostics and set the file order");
            return None;
        }
    };
    if d.has_errors() {
        return None;
    }
    // As `rbb` orders a directory: by path, component by component (file
    // order is the engine's last tie-breaker).
    files.sort_by(|a, b| a.0.split('/').cmp(b.0.split('/')));
    let custom_rules = req.get("rules").is_some_and(|v| !v.is_null());
    let manifest: Option<(String, String)> = match manifest {
        Some(t) => Some(("manifest".into(), t)),
        None if custom_rules => {
            d.info(format!(
                "manifest: none given; the rules are read as rule language {}",
                rbb_engine::LANGUAGE_VERSION
            ))
            .hint("give \"manifest\": the text of the rule set's conventions.toml");
            None
        }
        None => rbb_assets::MANIFEST.map(|(n, t)| (n.to_string(), t.to_string())),
    };
    let fields = fields.map_or(
        (
            rbb_assets::FIELDS.0.to_string(),
            rbb_assets::FIELDS.1.to_string(),
        ),
        |t| ("fields".into(), t),
    );
    let bbsa_map = bbsa_map.map_or(
        (
            rbb_assets::BBSA_MAP.0.to_string(),
            rbb_assets::BBSA_MAP.1.to_string(),
        ),
        |t| ("bbsa_map".into(), t),
    );

    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (name, text) in manifest
        .iter()
        .chain([&fields, &bbsa_map])
        .chain(files.iter())
    {
        fnv(&mut h, name.as_bytes());
        fnv(&mut h, &[0]);
        fnv(&mut h, text.as_bytes());
        fnv(&mut h, &[0]);
    }
    let id = format!("{h:016x}");
    if let Some(found) = RULE_SETS.with(|r| r.borrow().iter().find(|x| x.id == id).cloned()) {
        return Some(found);
    }
    let compiled = rbb_engine::compile_rules(
        manifest.as_ref().map(|(n, t)| (n.as_str(), t.as_str())),
        (&fields.0, &fields.1),
        (&bbsa_map.0, &bbsa_map.1),
        files.iter().map(|(n, s)| (n.as_str(), s.as_str())),
    );
    match compiled {
        Ok(set) => {
            let rules = Arc::new(Rules {
                set,
                id,
                files: files.len(),
            });
            RULE_SETS.with(|r| {
                let mut r = r.borrow_mut();
                r.push(rules.clone());
                if r.len() > RULE_SETS_KEPT {
                    r.remove(0);
                }
            });
            Some(rules)
        }
        Err(errors) => {
            for e in &errors {
                rule_diagnostic(e, Severity::Error, d);
            }
            None
        }
    }
}

/// What `rbb bid check` adds to loading, for `validate`: module names are
/// unique, enum parameters are compared with their options, every `needs`
/// names a module (a warning), and the `.bbsa` map's keys are BBA's (a
/// warning).
fn check_rule_set(rules: &Rules, d: &mut Diags) {
    let modules = &rules.set.modules;
    let mut seen = std::collections::HashMap::new();
    for m in modules {
        if let Some(other) = seen.insert(m.name.as_str(), m.file.as_str()) {
            let x = bidspec::Diagnostic {
                file: m.file.clone(),
                line: 0,
                col: 0,
                message: format!("module `{}` is also defined in {other}", m.name),
            };
            rule_diagnostic(&x, Severity::Error, d);
        }
    }
    for e in rbb_engine::check_card_refs(modules, rules.set.vocab.registry()) {
        // `file:line: message`
        let (at, message) = e.split_once(": ").unwrap_or(("", &e));
        let (file, line) = match at.rsplit_once(':') {
            Some((f, l)) if l.parse::<usize>().is_ok() => (f, l.parse().unwrap()),
            _ => (at, 0),
        };
        let x = bidspec::Diagnostic {
            file: file.to_string(),
            line,
            col: 0,
            message: message.to_string(),
        };
        rule_diagnostic(&x, Severity::Error, d);
    }
    for m in modules {
        for need in &m.needs {
            if !seen.contains_key(need.as_str()) {
                let x = bidspec::Diagnostic {
                    file: m.file.clone(),
                    line: 0,
                    col: 0,
                    message: format!("needs `{need}`, which no module defines"),
                };
                rule_diagnostic(&x, Severity::Warning, d);
            }
        }
    }
    for w in rules.set.vocab.lint() {
        d.warning(w.replacen("bbsa-map.toml: ", "bbsa_map: ", 1));
    }
}

/// A rule set's summary, as `validate` and `info` report it.
fn rule_set_json(rules: &Rules) -> Json {
    json!({
        "rules_id": rules.id,
        "name": rules.set.manifest.as_ref().map(|m| m.name.clone()),
        "language": rules.set.manifest.as_ref().map_or(rbb_engine::LANGUAGE_VERSION, |m| m.language),
        "rule_files": rules.files,
        "modules": rules.set.modules.len(),
        "rules": rule_count(&rules.set.modules),
        "card_fields": rules.set.vocab.registry().fields().len(),
    })
}

/// The rules of the engine a request names, else those it asks for.
fn rules_of(req: &Json, d: &mut Diags) -> Option<Arc<Rules>> {
    if let Some(id) = req.get("engine").and_then(Json::as_u64) {
        if let Some(e) = entry_by_id(id as u32) {
            return Some(e.rules);
        }
    }
    rules_for(req, d)
}

#[derive(Clone)]
struct Entry {
    id: u32,
    key: String,
    engine: Arc<Engine>,
    rules: Arc<Rules>,
    names: [String; 2],
}

thread_local! {
    static ENGINES: RefCell<(u32, Vec<Entry>)> = const { RefCell::new((0, Vec::new())) };
}

fn stock_names() -> Vec<&'static str> {
    rbb_assets::CARDS.iter().map(|(n, _)| *n).collect()
}

/// Line of `key = ...` in `.bbsa` text, 1-based.
fn bbsa_line(text: &str, key: &str) -> Option<usize> {
    text.lines()
        .position(|l| l.split('=').next().is_some_and(|k| k.trim() == key))
        .map(|i| i + 1)
}

/// A card as loaded: the card, its name, and the settings it switches on
/// that have no card field (`.bbsa` keys, or card JSON paths).
type Loaded = (Card, String, Vec<String>);

/// One card from its spec: a stock card's name, `{"stock": name}`,
/// `{"bbsa": text, "name"?: ..}` or `{"json": card_data}` (bare, or in
/// Bridge-Classroom's export wrapper `{schema, name, card_data, ...}`).
fn load_card(spec: &Json, field: &str, vocab: &Vocabulary, d: &mut Diags) -> Option<Loaded> {
    let stock = |name: &str, d: &mut Diags| match rbb_assets::card(name) {
        Some(text) => {
            import_bbsa(vocab, text, Some(name), field, d).map(|(c, u)| (c, name.to_string(), u))
        }
        None => {
            d.error(format!("{field}: no stock card called {name:?}"))
                .hint(format!("stock cards: {}", stock_names().join(", ")));
            None
        }
    };
    match spec {
        Json::String(name) => stock(name, d),
        Json::Object(m) => {
            if let Some(name) = m.get("stock").and_then(Json::as_str) {
                stock(name, d)
            } else if let Some(text) = m.get("bbsa").and_then(Json::as_str) {
                let name = m.get("name").and_then(Json::as_str).unwrap_or("bbsa");
                import_bbsa(vocab, text, Some(name), field, d)
                    .map(|(c, u)| (c, name.to_string(), u))
            } else if let Some(card) = m.get("json") {
                let text = match card {
                    Json::String(s) => s.clone(),
                    other => other.to_string(),
                };
                match Card::from_json(vocab, &text) {
                    Ok((card, report)) => {
                        if !report.ignored.is_empty() {
                            d.info(format!(
                                "{field}: {} ignored (not card settings: raw import records, kept)",
                                report.ignored.join(", ")
                            ));
                        }
                        for (from, to) in &report.aliased {
                            d.info(format!("{field}: {from} is an old name for {to}"));
                        }
                        for path in &report.unknown {
                            d.warning(format!("{field}: unknown card field {path} (ignored)"))
                                .hint("card fields: rbb card schema, or docs/DESIGN.md");
                        }
                        for (path, problem) in &report.invalid {
                            d.warning(format!("{field}: {path}: {problem} (ignored)"));
                        }
                        let name = card
                            .metadata
                            .name
                            .clone()
                            .unwrap_or_else(|| "card".to_string());
                        let unmapped = bidspec::coverage::unmapped_json(&card, &report.unknown);
                        Some((card, name, unmapped))
                    }
                    Err(e) => {
                        d.error(format!("{field}: not a card: {e}"));
                        None
                    }
                }
            } else {
                d.error(format!("{field}: expected \"stock\", \"bbsa\" or \"json\""))
                    .hint("e.g. \"21GF-DEFAULT\", {\"bbsa\": \"<file text>\"}, {\"json\": {...}}");
                None
            }
        }
        _ => {
            d.error(format!("{field}: expected a stock card name or an object"))
                .hint(format!("stock cards: {}", stock_names().join(", ")));
            None
        }
    }
}

fn import_bbsa(
    vocab: &Vocabulary,
    text: &str,
    name: Option<&str>,
    field: &str,
    d: &mut Diags,
) -> Option<(Card, Vec<String>)> {
    match bbsa::import(vocab, text, name) {
        Ok((card, report)) => {
            // One remark for them all: a stock card has dozens.
            if let Some((first, _)) = report.passthrough.first() {
                let keys: Vec<&str> = report.passthrough.iter().map(|(k, _)| k.as_str()).collect();
                d.info(format!(
                    "{field}: {} .bbsa keys have no card field (kept, not used): {}",
                    keys.len(),
                    keys.join(", ")
                ))
                .at(bbsa_line(text, first), None)
                .hint("`rbb card coverage` lists what the rules read of a card");
            }
            for w in &report.warnings {
                d.warning(format!("{field}: {w}"));
            }
            // A key set to 0 says the card does not play it: nothing is
            // lost by having nowhere to put it (as `rbb card coverage`).
            let unmapped = report
                .passthrough
                .iter()
                .filter(|(_, v)| *v != 0)
                .map(|(k, _)| k.clone())
                .collect();
            Some((card, unmapped))
        }
        Err(e) => {
            d.error(format!("{field}: not a .bbsa file: {e}"));
            None
        }
    }
}

fn string_list(v: Option<&Json>, field: &str, d: &mut Diags) -> Vec<String> {
    match v {
        None | Some(Json::Null) => Vec::new(),
        Some(Json::Array(a)) => a
            .iter()
            .filter_map(|x| match x.as_str() {
                Some(s) => Some(s.to_string()),
                None => {
                    d.error(format!(
                        "{field}: expected strings like \"general.style=bba\""
                    ));
                    None
                }
            })
            .collect(),
        Some(Json::String(s)) => vec![s.clone()],
        Some(_) => {
            d.error(format!(
                "{field}: expected a list of \"path=value\" strings"
            ));
            Vec::new()
        }
    }
}

/// Both cards from `{"ns", "ew"?, "set"?, "ns_set"?, "ew_set"?}`.
fn load_cards(cards: &Json, vocab: &Vocabulary, d: &mut Diags) -> Option<([Card; 2], [String; 2])> {
    load_cards_full(cards, vocab, d).map(|[ns, ew]| ([ns.0, ew.0], [ns.1, ew.1]))
}

/// Both cards, with the changes applied and each side's unmapped keys.
fn load_cards_full(cards: &Json, vocab: &Vocabulary, d: &mut Diags) -> Option<[Loaded; 2]> {
    let Json::Object(m) = cards else {
        d.error("cards: expected an object like {\"ns\": \"21GF-DEFAULT\", \"ew\": \"21GF-GIB\"}");
        return None;
    };
    let Some(ns_spec) = m.get("ns") else {
        d.error("cards.ns: missing")
            .hint("the North-South card, e.g. \"21GF-DEFAULT\"");
        return None;
    };
    let ns = load_card(ns_spec, "cards.ns", vocab, d);
    let ew = match m.get("ew") {
        Some(spec) => load_card(spec, "cards.ew", vocab, d),
        None => ns.clone(),
    };
    let both = string_list(m.get("set"), "cards.set", d);
    let mut out = Vec::new();
    for (side, card, own) in [("ns", ns, "ns_set"), ("ew", ew, "ew_set")] {
        let (mut card, name, unmapped) = card?;
        let own = string_list(m.get(own), &format!("cards.{own}"), d);
        for change in both.iter().chain(&own) {
            if let Err(e) = card.apply_change(change) {
                d.error(format!("cards.{side}: {e}"));
            }
        }
        out.push((card, name, unmapped));
    }
    if d.has_errors() {
        return None;
    }
    let (ew, ns) = (out.pop().unwrap(), out.pop().unwrap());
    Some([ns, ew])
}

fn entry_by_id(id: u32) -> Option<Entry> {
    ENGINES.with(|r| r.borrow().1.iter().find(|e| e.id == id).cloned())
}

/// The engine for `cards` under `rules`, made once per distinct pair.
fn engine_for_cards(cards: &Json, rules: Arc<Rules>, d: &mut Diags) -> Option<Entry> {
    // A rule set's id covers its rule files and its vocabulary.
    let key = format!("{} {}", rules.id, cards);
    if let Some(found) = ENGINES.with(|r| r.borrow().1.iter().find(|e| e.key == key).cloned()) {
        return Some(found);
    }
    let (cards, names) = load_cards(cards, &rules.set.vocab, d)?;
    let engine = Arc::new(Engine::new(&cards[0], &cards[1], &rules.set));
    let entry = ENGINES.with(|r| {
        let mut r = r.borrow_mut();
        r.0 += 1;
        let entry = Entry {
            id: r.0,
            key,
            engine,
            rules,
            names,
        };
        r.1.push(entry.clone());
        entry
    });
    Some(entry)
}

/// The engine a request names: `"engine": id`, or `"cards": {...}` (with
/// the rules the request asks for).
fn engine_for(req: &Json, d: &mut Diags) -> Option<Entry> {
    if let Some(v) = req.get("engine") {
        let Some(id) = v.as_u64() else {
            d.error("engine: expected the number create_engine returned");
            return None;
        };
        return match entry_by_id(id as u32) {
            Some(e) => Some(e),
            None => {
                d.error(format!("engine: no engine {id}"))
                    .hint("create one with createEngine (it may have been freed)");
                None
            }
        };
    }
    if let Some(cards) = req.get("cards") {
        let rules = rules_for(req, d)?;
        return engine_for_cards(cards, rules, d);
    }
    d.error("give \"engine\" (from createEngine) or \"cards\"")
        .hint("e.g. {\"cards\": {\"ns\": \"21GF-DEFAULT\"}, ...}");
    None
}

// ── Request fields ─────────────────────────────────────────────────────────

fn field_str<'a>(req: &'a Json, field: &str, d: &mut Diags) -> Option<&'a str> {
    match req.get(field) {
        None | Some(Json::Null) => None,
        Some(Json::String(s)) => Some(s),
        Some(_) => {
            d.error(format!("{field}: expected a string"));
            None
        }
    }
}

fn seat(req: &Json, field: &str, default: Option<Direction>, d: &mut Diags) -> Option<Direction> {
    match field_str(req, field, d) {
        None => {
            if default.is_none() && !d.has_errors() {
                d.error(format!("{field}: missing")).hint("N, E, S or W");
            }
            default
        }
        Some(s) => {
            let dir = s
                .trim()
                .chars()
                .next()
                .and_then(|c| Direction::from_char(c.to_ascii_uppercase()));
            if dir.is_none() {
                d.error(format!("{field}: {s:?} is not a seat"))
                    .hint("N, E, S or W");
            }
            dir
        }
    }
}

fn vulnerability(req: &Json, d: &mut Diags) -> Vulnerability {
    match field_str(req, "vul", d) {
        None => Vulnerability::None,
        Some(s) => Vulnerability::from_pbn(s.trim()).unwrap_or_else(|| {
            d.error(format!("vul: {s:?} is not a vulnerability"))
                .hint("None, NS, EW or All");
            Vulnerability::None
        }),
    }
}

fn scoring(req: &Json, d: &mut Diags) -> ScoringMethod {
    match field_str(req, "scoring", d) {
        None => ScoringMethod::Matchpoints,
        Some(s) => match ScoringMethod::from_pbn(s.trim()) {
            Some(m @ (ScoringMethod::Matchpoints | ScoringMethod::IMP)) => m,
            _ => {
                d.error(format!("scoring: {s:?} is not MP or IMP"))
                    .hint("MP or IMP");
                ScoringMethod::Matchpoints
            }
        },
    }
}

const HAND_HINT: &str =
    "spades.hearts.diamonds.clubs, e.g. AK52.KJ7.Q94.K83 (T for ten, - for a void)";

/// A 13-card hand, S.H.D.C.
fn parse_hand(text: &str, field: &str, d: &mut Diags) -> Option<Hand> {
    let Some(hand) = Hand::from_pbn(text) else {
        d.error(format!("{field}: {text:?} is not a hand"))
            .hint(HAND_HINT);
        return None;
    };
    let cards = hand.cards();
    for (i, c) in cards.iter().enumerate() {
        if cards[..i].contains(c) {
            d.error(format!("{field}: {c} appears twice"))
                .hint(HAND_HINT);
            return None;
        }
    }
    if hand.len() != 13 {
        d.error(format!("{field}: {} cards, not 13", hand.len()))
            .hint(HAND_HINT);
        return None;
    }
    Some(hand)
}

/// Four hands: a PBN deal string (`N:<N> <E> <S> <W>`) or
/// `{"N": .., "E": .., "S": .., "W": ..}`.
fn parse_deal(v: &Json, d: &mut Diags) -> Option<[Hand; 4]> {
    let mut hands: [Option<Hand>; 4] = Default::default();
    match v {
        Json::String(s) => {
            let Some(deal) = Deal::from_pbn(s) else {
                d.error(format!("deal: {s:?} is not a PBN deal"))
                    .hint("N:<north> <east> <south> <west>, each S.H.D.C");
                return None;
            };
            for dir in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                let h = deal.hand(dir);
                hands[dir.to_index()] =
                    parse_hand(&h.to_pbn(), &format!("deal.{}", dir.to_char()), d);
            }
        }
        Json::Object(m) => {
            for dir in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                let key = dir.to_char().to_string();
                match m.get(&key).and_then(Json::as_str) {
                    Some(t) => hands[dir.to_index()] = parse_hand(t, &format!("deal.{key}"), d),
                    None => {
                        d.error(format!("deal.{key}: missing")).hint(HAND_HINT);
                    }
                }
            }
        }
        _ => {
            d.error("deal: expected a PBN deal string or {\"N\":..,\"E\":..,\"S\":..,\"W\":..}");
            return None;
        }
    }
    if d.has_errors() {
        return None;
    }
    let hands = hands.map(Option::unwrap);
    let mut seen = Vec::with_capacity(52);
    for (i, h) in hands.iter().enumerate() {
        for c in h.cards() {
            if seen.contains(c) {
                let who = Direction::from_index(i).map_or('?', |x| x.to_char());
                d.error(format!("deal: {c} is in two hands (again in {who})"));
                return None;
            }
            seen.push(*c);
        }
    }
    Some(hands)
}

/// Calls from a string (`"1NT Pass 2C"`) or a list (`["1NT", "Pass"]`),
/// checked for legality from `dealer`. A position in a string is reported
/// as `col` (1-based).
fn parse_auction(
    v: Option<&Json>,
    field: &str,
    dealer: Direction,
    d: &mut Diags,
) -> Option<Vec<Call>> {
    let tokens: Vec<(String, Option<usize>)> = match v {
        None | Some(Json::Null) => Vec::new(),
        Some(Json::String(s)) => {
            let mut out = Vec::new();
            let mut start = None;
            for (i, c) in s.chars().chain(std::iter::once(' ')).enumerate() {
                if c.is_whitespace() || c == ',' {
                    if let Some(st) = start.take() {
                        let tok: String = s.chars().skip(st).take(i - st).collect();
                        out.push((tok, Some(st + 1)));
                    }
                } else if start.is_none() {
                    start = Some(i);
                }
            }
            out
        }
        Some(Json::Array(a)) => {
            let mut out = Vec::new();
            for x in a {
                match x.as_str() {
                    Some(s) => out.push((s.to_string(), None)),
                    None => {
                        d.error(format!("{field}: expected calls as strings"));
                        return None;
                    }
                }
            }
            out
        }
        Some(_) => {
            d.error(format!(
                "{field}: expected a string like \"1NT Pass\" or a list of calls"
            ));
            return None;
        }
    };
    let mut auction = Auction::new(dealer);
    let mut calls = Vec::new();
    for (i, (tok, col)) in tokens.iter().enumerate() {
        let n = i + 1;
        let call = match Call::from_pbn(tok) {
            Some(c @ (Call::Pass | Call::Double | Call::Redouble | Call::Bid { .. })) => c,
            _ => {
                d.error(format!("{field}: call {n}: {tok:?} is not a call"))
                    .at(col.map(|_| 1), *col)
                    .hint("1C..7NT, Pass (or P), X, XX");
                return None;
            }
        };
        if auction.is_complete() {
            d.error(format!(
                "{field}: call {n} ({tok}): the auction is already over"
            ))
            .at(col.map(|_| 1), *col);
            return None;
        }
        if !auction.is_legal(&call) {
            d.error(format!(
                "{field}: call {n} ({tok}) is not legal here ({} to call)",
                auction.next_caller().to_char()
            ))
            .at(col.map(|_| 1), *col)
            .hint("a bid must outrank the last one; X only over an opponent's bid, XX only over their X");
            return None;
        }
        auction.add_call(call.clone());
        calls.push(call);
    }
    Some(calls)
}

// ── Output shapes ──────────────────────────────────────────────────────────

fn seat_str(d: Direction) -> String {
    d.to_char().to_string()
}

fn range(r: rbb_engine::Range) -> Json {
    json!({"min": r.lo, "max": r.hi})
}

fn knowledge(k: &SeatKnowledge) -> Json {
    json!({
        "hcp": range(k.hcp),
        "lengths": {
            "S": range(k.len[3]),
            "H": range(k.len[2]),
            "D": range(k.len[1]),
            "C": range(k.len[0]),
        },
        "balanced": match k.balanced {
            Tri::True => Json::Bool(true),
            Tri::False => Json::Bool(false),
            Tri::Unknown => Json::Null,
        },
        "summary": k.summary(),
        "shown": k.shown,
    })
}

fn rule(r: &Option<RuleRef>) -> Json {
    match r {
        Some(r) => json!({"module": r.module, "file": r.file, "line": r.line}),
        None => Json::Null,
    }
}

fn alert(a: &Option<bidspec::ast::Alert>) -> Json {
    serde_json::to_value(a).unwrap_or(Json::Null)
}

fn step_json(s: &Step, index: usize, d: &mut Diags) -> Json {
    for w in &s.warnings {
        d.warning(format!("call {} ({}): {w}", index + 1, s.call.to_pbn()));
    }
    json!({
        "seat": seat_str(s.caller),
        "call": s.call.to_pbn(),
        "explanation": s.explanation,
        "alert": alert(&s.alert),
        "rule": rule(&s.rule),
        "artificial": s.artificial,
        "knowledge": knowledge(&s.knowledge),
    })
}

/// A call of an auction the engine bid: its step, plus `forced` and
/// `meaning`; for the engine's own calls `explanation`, `rule` and `alert`
/// are the rule that chose it for the hand.
fn deal_call_json(c: &DealCall, index: usize, d: &mut Diags) -> Json {
    let mut j = step_json(&c.step, index, d);
    let m = j.as_object_mut().unwrap();
    // "explanation" is why this hand made the call (the rule that chose
    // it); "meaning" is what the call shows to the table (the step).
    m.insert("meaning".into(), json!(c.step.explanation));
    m.insert("forced".into(), Json::Bool(c.choice.is_none()));
    if let Some(choice) = &c.choice {
        m.insert("explanation".into(), json!(choice.explanation));
        m.insert("rule".into(), rule(&choice.rule));
        m.insert("alert".into(), alert(&choice.alert));
        for w in &choice.warnings {
            d.warning(format!(
                "call {} ({}): {w}",
                index + 1,
                choice.call.to_pbn()
            ));
        }
    }
    j
}

fn strain_str(s: Strain) -> &'static str {
    STRAIN_KEYS
        .iter()
        .find(|(k, _)| *k == s)
        .map_or("?", |(_, v)| v)
}

fn ask_json(a: &Option<Ask>) -> Json {
    match a {
        Some(a) => json!({
            "kind": a.kind,
            "args": a.args.iter().map(|s| strain_str(*s)).collect::<Vec<_>>(),
            "by": seat_str(a.by),
        }),
        None => Json::Null,
    }
}

/// A side's auction state: trump, forcing, a question outstanding.
fn side_json(st: &SideState) -> Json {
    let forcing = match st.forcing {
        Forcing::None => "none",
        Forcing::Round => "round",
        Forcing::Game => "game",
    };
    let mut summary = vec![];
    if let Some(t) = st.trump {
        summary.push(format!("trump {}", strain_str(t)));
    }
    summary.push(match st.forcing_by {
        Some(by) if st.forcing != Forcing::None => {
            format!("forcing {forcing} (set by {})", by.to_char())
        }
        _ => format!("forcing {forcing}"),
    });
    if let Some(a) = &st.ask {
        summary.push(format!("asked {} by {}", a.kind, a.by.to_char()));
    }
    if let Some(a) = &st.answered {
        summary.push(format!("answered {} (asked by {})", a.kind, a.by.to_char()));
    }
    json!({
        "trump": st.trump.map(strain_str),
        "forcing": forcing,
        "forcing_by": st.forcing_by.map(seat_str),
        "ask": ask_json(&st.ask),
        "answered": ask_json(&st.answered),
        "summary": summary.join(", "),
    })
}

/// What every seat has shown, and each side's state, at a point of the
/// auction.
fn position_json(p: &Position) -> Json {
    let mut seats = Map::new();
    for s in SEATS {
        seats.insert(seat_str(s), knowledge(p.knowledge(s)));
    }
    json!({
        "knowledge": seats,
        "sides": {"ns": side_json(&p.sides[0]), "ew": side_json(&p.sides[1])},
    })
}

fn candidate_json(c: &CandidateTrace) -> Json {
    json!({
        "call": c.call.to_pbn(),
        "rule": rule(&Some(c.rule.clone())),
        "explanation": c.explanation,
        "priority": c.priority,
        "descriptiveness": c.descriptiveness,
        "prefer": c.prefer,
        "outcome": c.outcome,
    })
}

fn contract_json(dealer: Direction, calls: &[Call]) -> (Json, Json) {
    let mut a = Auction::new(dealer);
    for c in calls {
        a.add_call(c.clone());
    }
    if !a.is_complete() {
        return (Json::Null, Json::Null);
    }
    match a.final_contract() {
        Some(fc) => (
            Json::String(fc.to_pbn()),
            Json::String(seat_str(fc.declarer)),
        ),
        None => (Json::String("Pass".into()), Json::Null),
    }
}

// ── The API ────────────────────────────────────────────────────────────────

/// What this build is: API and crate versions, the rules, the stock cards.
pub fn info() -> String {
    let mut d = Diags::default();
    let (files, modules, rules) = match embedded_or_report(&mut d) {
        Some(r) => (r.files, r.set.modules.len(), rule_count(&r.set.modules)),
        None => (0, 0, 0),
    };
    respond(
        json!({
            "api": API_VERSION,
            "version": env!("CARGO_PKG_VERSION"),
            "rules_id": rbb_assets::RULES_ID,
            "rule_files": files,
            "modules": modules,
            "rules": rules,
            "language": rbb_engine::LANGUAGE_VERSION,
            "stock_cards": stock_names(),
        }),
        d,
    )
}

/// Make (or find) the engine for a pair of cards.
pub fn create_engine(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let mut body = json!({"engine": Json::Null});
    if !d.has_errors() {
        match req.get("cards") {
            None => {
                d.error("cards: missing")
                    .hint("{\"cards\": {\"ns\": \"21GF-DEFAULT\", \"ew\": \"21GF-GIB\"}}");
            }
            Some(cards) => {
                if let Some(e) =
                    rules_for(&req, &mut d).and_then(|rules| engine_for_cards(cards, rules, &mut d))
                {
                    let side = |seat: Direction, name: &str| json!({"name": name, "modules": e.engine.system(seat).modules});
                    body = json!({
                        "engine": e.id,
                        "rules_id": e.rules.id,
                        "ns": side(Direction::North, &e.names[0]),
                        "ew": side(Direction::East, &e.names[1]),
                    });
                }
            }
        }
    }
    respond(body, d)
}

/// Forget an engine.
pub fn free_engine(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    match req.get("engine").and_then(Json::as_u64) {
        Some(id) => {
            let removed = ENGINES.with(|r| {
                let mut r = r.borrow_mut();
                let before = r.1.len();
                r.1.retain(|e| e.id != id as u32);
                before != r.1.len()
            });
            if !removed {
                d.warning(format!("engine: no engine {id}"));
            }
        }
        None => {
            d.error("engine: expected the number createEngine returned");
        }
    }
    respond(json!({}), d)
}

/// Check the fields a request carries (hand, deal, auction with dealer,
/// cards, vul, scoring) without bidding.
pub fn validate(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    if let Some(h) = field_str(&req, "hand", &mut d) {
        parse_hand(h, "hand", &mut d);
    }
    if let Some(v) = req.get("deal") {
        parse_deal(v, &mut d);
    }
    let has_auction = req.get("auction").is_some() || req.get("prefix").is_some();
    let dealer = seat(
        &req,
        "dealer",
        if has_auction {
            None
        } else {
            Some(Direction::North)
        },
        &mut d,
    );
    if let Some(dealer) = dealer {
        parse_auction(req.get("auction"), "auction", dealer, &mut d);
        parse_auction(req.get("prefix"), "prefix", dealer, &mut d);
    }
    if req.get("vul").is_some() {
        vulnerability(&req, &mut d);
    }
    if req.get("scoring").is_some() {
        scoring(&req, &mut d);
    }
    if let Some(id) = req.get("engine") {
        if id.as_u64().and_then(|i| entry_by_id(i as u32)).is_none() {
            d.error(format!("engine: no engine {id}"));
        }
    }
    let mut body = json!({});
    let supplied = RULE_FIELDS.iter().any(|f| req.get(f).is_some());
    let rules = if supplied || req.get("cards").is_some() {
        rules_of(&req, &mut d)
    } else {
        None
    };
    if let Some(rules) = &rules {
        if supplied && req.get("engine").is_none() {
            check_rule_set(rules, &mut d);
            body["rule_set"] = rule_set_json(rules);
        }
        if let Some(cards) = req.get("cards") {
            load_cards(cards, &rules.set.vocab, &mut d);
        }
    }
    respond(body, d)
}

/// The engine's call for a hand after an auction, with the reason, the
/// candidates, and how it read the auction so far.
pub fn bid(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let empty = json!({"call": Json::Null});
    if d.has_errors() {
        return respond(empty, d);
    }
    let engine = engine_for(&req, &mut d);
    let hand = match field_str(&req, "hand", &mut d) {
        Some(h) => parse_hand(h, "hand", &mut d),
        None => {
            d.error("hand: missing").hint(HAND_HINT);
            None
        }
    };
    let dealer = seat(&req, "dealer", None, &mut d);
    let vul = vulnerability(&req, &mut d);
    let scoring = scoring(&req, &mut d);
    let calls = dealer.and_then(|dl| parse_auction(req.get("auction"), "auction", dl, &mut d));
    let (Some(Entry { engine, .. }), Some(hand), Some(dealer), Some(calls)) =
        (engine, hand, dealer, calls)
    else {
        return respond(empty, d);
    };
    if d.has_errors() {
        return respond(empty, d);
    }
    {
        let mut a = Auction::new(dealer);
        for c in &calls {
            a.add_call(c.clone());
        }
        if a.is_complete() {
            d.error("auction: the auction is already over");
            return respond(empty, d);
        }
    }
    let decision = engine.bid(&hand, dealer, vul, scoring, &calls);
    let steps: Vec<Json> = decision
        .auction
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| step_json(s, i, &mut d))
        .collect();
    for w in &decision.warnings {
        d.warning(w.clone());
    }
    if decision.rule.is_none() {
        d.info("no rule applies: the engine passes");
    }
    respond(
        json!({
            "seat": seat_str(decision.auction.position.next_caller()),
            "call": decision.call.to_pbn(),
            "explanation": decision.explanation,
            "alert": alert(&decision.alert),
            "rule": rule(&decision.rule),
            "candidates": decision.candidates.iter().map(candidate_json).collect::<Vec<_>>(),
            "auction": steps,
            "position": position_json(&decision.auction.position),
            "warnings": decision.warnings,
        }),
        d,
    )
}

/// What each call of an auction showed, as the engine reads it.
pub fn interpret(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let empty = json!({"steps": []});
    if d.has_errors() {
        return respond(empty, d);
    }
    let engine = engine_for(&req, &mut d);
    let dealer = seat(&req, "dealer", None, &mut d);
    let vul = vulnerability(&req, &mut d);
    let scoring = scoring(&req, &mut d);
    let calls = dealer.and_then(|dl| parse_auction(req.get("auction"), "auction", dl, &mut d));
    let (Some(Entry { engine, .. }), Some(dealer), Some(calls)) = (engine, dealer, calls) else {
        return respond(empty, d);
    };
    if d.has_errors() {
        return respond(empty, d);
    }
    let interp = engine.interpret(dealer, vul, scoring, &calls);
    let steps: Vec<Json> = interp
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| step_json(s, i, &mut d))
        .collect();
    let (contract, declarer) = contract_json(dealer, &calls);
    let complete = interp.position.auction().is_complete();
    respond(
        json!({
            "steps": steps,
            "complete": complete,
            "next": if complete { Json::Null } else { Json::String(seat_str(interp.position.next_caller())) },
            "contract": contract,
            "declarer": declarer,
            "position": position_json(&interp.position),
        }),
        d,
    )
}

/// Bid all four hands of a deal: forced `prefix` calls first, then the
/// engine to the end.
pub fn bid_deal(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let empty = json!({"calls": []});
    if d.has_errors() {
        return respond(empty, d);
    }
    let engine = engine_for(&req, &mut d);
    let hands = match req.get("deal") {
        Some(v) => parse_deal(v, &mut d),
        None => {
            d.error("deal: missing")
                .hint("N:<north> <east> <south> <west>, each S.H.D.C");
            None
        }
    };
    let dealer = seat(&req, "dealer", None, &mut d);
    let vul = vulnerability(&req, &mut d);
    let scoring = scoring(&req, &mut d);
    let prefix = dealer.and_then(|dl| parse_auction(req.get("prefix"), "prefix", dl, &mut d));
    let max_calls = req
        .get("max_calls")
        .and_then(Json::as_u64)
        .map_or(MAX_CALLS, |n| n as usize);
    let (Some(Entry { engine, .. }), Some(hands), Some(dealer), Some(prefix)) =
        (engine, hands, dealer, prefix)
    else {
        return respond(empty, d);
    };
    if d.has_errors() {
        return respond(empty, d);
    }
    let auction = match engine.bid_deal(&hands, dealer, vul, scoring, &prefix, max_calls) {
        Ok(a) => a,
        Err(e) => {
            d.error(format!("prefix: {e}"));
            return respond(empty, d);
        }
    };
    if !auction.complete {
        d.warning(format!(
            "the auction did not end within {max_calls} calls; stopped there"
        ));
    }
    let mut calls = Vec::new();
    let mut plain = Vec::new();
    for (i, c) in auction.calls.iter().enumerate() {
        calls.push(deal_call_json(c, i, &mut d));
        plain.push(c.step.call.clone());
    }
    let (contract, declarer) = contract_json(dealer, &plain);
    respond(
        json!({
            "calls": calls,
            "complete": auction.complete,
            "contract": contract,
            "declarer": declarer,
        }),
        d,
    )
}

/// The hands of a practice table: `deal` (all four) or `hands` (an object
/// with the seats given, at least the bots'), no card in two hands.
fn table_hands(req: &Json, d: &mut Diags) -> Option<[Option<Hand>; 4]> {
    match (req.get("deal"), req.get("hands")) {
        (Some(_), Some(_)) => {
            d.error("hands: give \"deal\" or \"hands\", not both");
            None
        }
        (Some(v), None) => parse_deal(v, d).map(|h| h.map(Some)),
        (None, Some(Json::Object(m))) => {
            let mut hands: [Option<Hand>; 4] = Default::default();
            for (key, v) in m {
                let Some(dir) = key
                    .chars()
                    .next()
                    .and_then(|c| Direction::from_char(c.to_ascii_uppercase()))
                else {
                    d.error(format!("hands: {key:?} is not a seat"))
                        .hint("N, E, S or W");
                    continue;
                };
                let field = format!("hands.{}", dir.to_char());
                match v.as_str() {
                    Some(t) => hands[dir.to_index()] = parse_hand(t, &field, d),
                    None => {
                        d.error(format!("{field}: expected a hand as a string"))
                            .hint(HAND_HINT);
                    }
                }
            }
            if d.has_errors() {
                return None;
            }
            let mut seen = Vec::with_capacity(52);
            for (i, h) in hands.iter().enumerate() {
                for c in h.iter().flat_map(|h| h.cards()) {
                    if seen.contains(c) {
                        let who = Direction::from_index(i).map_or('?', |x| x.to_char());
                        d.error(format!("hands: {c} is in two hands (again in {who})"));
                        return None;
                    }
                    seen.push(*c);
                }
            }
            Some(hands)
        }
        (None, Some(_)) => {
            d.error("hands: expected {\"S\": hand, ...} with the bot seats' hands");
            None
        }
        (None, None) => {
            d.error("hands: missing")
                .hint("\"hands\": {\"E\": .., \"W\": ..} (the bot seats), or a whole \"deal\"");
            None
        }
    }
}

/// The seats the engine bids for: `"bots": ["N", "E", "W"]` or `"NEW"`.
fn bot_seats(req: &Json, d: &mut Diags) -> Option<[bool; 4]> {
    let mut bots = [false; 4];
    let mut mark = |s: &str, d: &mut Diags| {
        let dir = s
            .trim()
            .chars()
            .next()
            .and_then(|c| Direction::from_char(c.to_ascii_uppercase()));
        match dir {
            Some(dir) => bots[dir.to_index()] = true,
            None => {
                d.error(format!("bots: {s:?} is not a seat"))
                    .hint("a list of seats, e.g. [\"N\", \"E\", \"W\"], or \"NEW\"");
            }
        }
    };
    match req.get("bots") {
        Some(Json::String(s)) => {
            for c in s.chars().filter(|c| !c.is_whitespace() && *c != ',') {
                mark(&c.to_string(), d);
            }
        }
        Some(Json::Array(a)) => {
            for x in a {
                match x.as_str() {
                    Some(s) => mark(s, d),
                    None => {
                        d.error("bots: expected seats as strings");
                    }
                }
            }
        }
        None | Some(Json::Null) => {
            d.error("bots: missing")
                .hint("the seats the engine bids for, e.g. [\"N\", \"E\", \"W\"]");
        }
        Some(_) => {
            d.error("bots: expected a list of seats");
        }
    }
    (!d.has_errors()).then_some(bots)
}

/// Bid the bot seats of a practice table from the calls so far until the
/// auction ends, a human is to call, or a bot seat has no rule (where the
/// caller takes another bidder's call and calls again).
pub fn auction(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let empty = json!({"calls": [], "steps": [], "stop": Json::Null});
    if d.has_errors() {
        return respond(empty, d);
    }
    let entry = engine_for(&req, &mut d);
    let hands = table_hands(&req, &mut d);
    let bots = bot_seats(&req, &mut d);
    let dealer = seat(&req, "dealer", None, &mut d);
    let vul = vulnerability(&req, &mut d);
    let scoring = scoring(&req, &mut d);
    let calls = dealer.and_then(|dl| parse_auction(req.get("auction"), "auction", dl, &mut d));
    let on_no_rule = match field_str(&req, "no_rule", &mut d) {
        None | Some("stop") => OnNoRule::Stop,
        Some("pass") => OnNoRule::Pass,
        Some(s) => {
            d.error(format!("no_rule: {s:?} is not \"stop\" or \"pass\""));
            OnNoRule::Stop
        }
    };
    let (Some(entry), Some(hands), Some(bots), Some(dealer), Some(calls)) =
        (entry, hands, bots, dealer, calls)
    else {
        return respond(empty, d);
    };
    if d.has_errors() {
        return respond(empty, d);
    }
    let table = Table {
        dealer,
        vul,
        scoring,
        hands,
        bots,
    };
    let t = match entry.engine.auction(&table, &calls, on_no_rule) {
        Ok(t) => t,
        Err(e) => {
            let field = if e.contains("bot seat") {
                "hands"
            } else {
                "auction"
            };
            d.error(format!("{field}: {e}"));
            return respond(empty, d);
        }
    };
    let mut steps = Vec::new();
    let mut made = Vec::new();
    let mut plain = Vec::new();
    for (i, c) in t.calls.iter().enumerate() {
        let by = if c.choice.is_some() { "bot" } else { "given" };
        let mut s = step_json(&c.step, i, &mut Diags::default());
        s["index"] = json!(i);
        s["by"] = json!(by);
        steps.push(s);
        if let Some(choice) = &c.choice {
            let mut j = deal_call_json(c, i, &mut d);
            j["index"] = json!(i);
            j["no_rule"] = json!(choice.rule.is_none());
            made.push(j);
        }
        plain.push(c.step.call.clone());
    }
    // The warnings of the given calls' readings, once.
    for (i, c) in t.calls.iter().enumerate().take(t.given) {
        for w in &c.step.warnings {
            d.warning(format!("call {} ({}): {w}", i + 1, c.step.call.to_pbn()));
        }
    }
    let stop = &t.stop;
    let mut stop_json = json!({
        "reason": stop.reason,
        "seat": stop.seat.map(seat_str),
        "index": stop.index,
        "auction": plain.iter().map(Call::to_pbn).collect::<Vec<_>>(),
    });
    if let Some(choice) = &stop.choice {
        stop_json["call"] = json!(choice.call.to_pbn());
        stop_json["candidates"] = choice.candidates.iter().map(candidate_json).collect();
        for w in &choice.warnings {
            d.warning(format!("call {} (no rule): {w}", stop.index + 1));
        }
    }
    if stop.reason == StopReason::NoRule {
        d.info(format!(
            "no rule applies for {} at call {}: supply that call and call auction again",
            stop.seat.map_or('?', |s| s.to_char()),
            stop.index + 1
        ));
    }
    let (contract, declarer) = contract_json(dealer, &plain);
    let complete = stop.reason == StopReason::Complete;
    respond(
        json!({
            "rules_id": entry.rules.id,
            "cards": {"ns": entry.names[0], "ew": entry.names[1]},
            "calls": made,
            "steps": steps,
            "stop": stop_json,
            "complete": complete,
            "next": stop.seat.map(seat_str),
            "contract": contract,
            "declarer": declarer,
            "position": position_json(&t.position),
        }),
        d,
    )
}

/// What one call of an auction means to the engine (a mouseover): the
/// auction is read up to and including call `index` (default the last).
pub fn meaning(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let empty = json!({"index": Json::Null, "known": false, "step": Json::Null});
    if d.has_errors() {
        return respond(empty, d);
    }
    let engine = engine_for(&req, &mut d);
    let dealer = seat(&req, "dealer", None, &mut d);
    let vul = vulnerability(&req, &mut d);
    let scoring = scoring(&req, &mut d);
    let calls = dealer.and_then(|dl| parse_auction(req.get("auction"), "auction", dl, &mut d));
    let (Some(Entry { engine, .. }), Some(dealer), Some(calls)) = (engine, dealer, calls) else {
        return respond(empty, d);
    };
    if d.has_errors() {
        return respond(empty, d);
    }
    let index = match req.get("index") {
        None | Some(Json::Null) if !calls.is_empty() => calls.len() - 1,
        Some(v) if v.as_u64().is_some_and(|i| (i as usize) < calls.len()) => {
            v.as_u64().unwrap() as usize
        }
        _ => {
            d.error(format!(
                "index: expected the index of a call of the auction (0 to {})",
                calls.len() as i64 - 1
            ))
            .hint("0 is the dealer's first call; leave it out for the last call");
            return respond(empty, d);
        }
    };
    let interp = engine.interpret(dealer, vul, scoring, &calls[..=index]);
    let step = &interp.steps[index];
    let mut s = step_json(step, index, &mut d);
    s["index"] = json!(index);
    respond(
        json!({
            "index": index,
            "known": step.rule.is_some(),
            "step": s,
        }),
        d,
    )
}

fn active_sets(req: &Json, d: &mut Diags) -> Option<[Vec<String>; 2]> {
    if req.get("engine").is_none() && req.get("cards").is_none() {
        return None;
    }
    let engine = engine_for(req, d)?.engine;
    Some([
        engine.system(Direction::North).modules.clone(),
        engine.system(Direction::East).modules.clone(),
    ])
}

/// The embedded conventions as data: modules, their card conditions, and
/// each rule's auction, call and meaning. With `engine` or `cards`, also
/// which modules each side plays.
pub fn conventions(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let Some(rules) = rules_of(&req, &mut d) else {
        return respond(json!({"modules": []}), d);
    };
    let active = active_sets(&req, &mut d);
    let mut body = json!({
        "rules_id": rules.id,
        "modules": bidspec::reference::entries(&rules.set.modules),
    });
    if let Some([ns, ew]) = active {
        body["active"] = json!({"ns": ns, "ew": ew});
    }
    respond(body, d)
}

/// The embedded conventions as plain text (for `reference.txt`), from the
/// same data as `conventions`. With `engine` or `cards`, each module is
/// marked `[on]` or `[off]` for `side` (`"ns"`, the default, or `"ew"`).
/// Problems with the request are written as `# error:` lines at the top.
pub fn reference(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let Some(rules) = rules_of(&req, &mut d) else {
        return d
            .0
            .iter()
            .map(|x| format!("# error: {}\n", x.message))
            .collect();
    };
    let active = active_sets(&req, &mut d);
    let side = match req.get("side").and_then(Json::as_str) {
        None | Some("ns") | Some("NS") => 0,
        Some("ew") | Some("EW") => 1,
        Some(other) => {
            d.error(format!("side: {other:?} is not ns or ew"));
            0
        }
    };
    let header = format!(
        "rusty-bidding-bot conventions reference\nrbb {} rules {}",
        env!("CARGO_PKG_VERSION"),
        rules.id
    );
    let entries = bidspec::reference::entries(&rules.set.modules);
    let on = active.as_ref().map(|a| a[side].clone());
    let marker = on
        .as_ref()
        .map(|list| move |name: &str| list.iter().any(|m| m == name));
    let mut out: String =
        d.0.iter()
            .filter(|x| x.severity != Severity::Info)
            .map(|x| format!("# {}: {}\n", severity_word(x.severity), x.message))
            .collect();
    out.push_str(&bidspec::reference::text(
        &entries,
        &header,
        marker.as_ref().map(|f| f as &dyn Fn(&str) -> bool),
    ));
    out
}

/// How much of each side's card the rules read: the settings the card
/// switches on, split into `read` (a module names them), `ignored` (the
/// field exists but no rule reads it: a convention or treatment the engine
/// does not play) and `unmapped` (`.bbsa` keys with no card field at all).
/// `other` holds carding, leads and notes, which cannot change a call.
pub fn coverage(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let empty = json!({"ns": Json::Null, "ew": Json::Null});
    if d.has_errors() {
        return respond(empty, d);
    }
    let Some(rules) = rules_of(&req, &mut d) else {
        return respond(empty, d);
    };
    let Some(cards) = req.get("cards") else {
        d.error("cards: missing")
            .hint("{\"cards\": {\"ns\": \"21GF-DEFAULT\", \"ew\": \"21GF-GIB\"}}");
        return respond(empty, d);
    };
    let Some([ns, ew]) = load_cards_full(cards, &rules.set.vocab, &mut d) else {
        return respond(empty, d);
    };
    let read = bidspec::coverage::fields_read(&rules.set.modules, &rules.set.vocab);
    let side = |(card, name, unmapped): Loaded| {
        let cov = bidspec::coverage::of_card(&name, &card, unmapped, &read);
        let mut j = serde_json::to_value(&cov).unwrap_or(Json::Null);
        j["score"] = json!(cov.score());
        j
    };
    respond(json!({"ns": side(ns), "ew": side(ew)}), d)
}

/// A card as Bridge-Classroom card JSON (`card_data`, and the editor's
/// export wrapper around it) and as `.bbsa` text: the mapping between the
/// two formats. Request: `{"card": <card spec>, "set"?: ["path=value"],
/// "exported_at"?: "<ISO 8601 time>"}`.
pub fn export_card(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let empty = json!({"name": Json::Null, "json": Json::Null, "bridge_classroom": Json::Null,
                       "bbsa": Json::Null, "unmapped": []});
    if d.has_errors() {
        return respond(empty, d);
    }
    let Some(spec) = req.get("card") else {
        d.error("card: missing")
            .hint("a stock card name, {\"bbsa\": text} or {\"json\": card_data}");
        return respond(empty, d);
    };
    let Some(rules) = rules_of(&req, &mut d) else {
        return respond(empty, d);
    };
    let Some((mut card, name, unmapped)) = load_card(spec, "card", &rules.set.vocab, &mut d) else {
        return respond(empty, d);
    };
    for change in string_list(req.get("set"), "set", &mut d) {
        if let Err(e) = card.apply_change(&change) {
            d.error(format!("set: {e}"));
        }
    }
    if d.has_errors() {
        return respond(empty, d);
    }
    let (bbsa_text, report) = bbsa::export(&card);
    if !report.dropped.is_empty() {
        d.info(format!(
            "bbsa: {} keys not in the current .bbsa layout were not written: {}",
            report.dropped.len(),
            report.dropped.join(", ")
        ));
    }
    respond(
        json!({
            "name": name,
            "json": card.to_json(),
            "bridge_classroom": card.to_export_json(
                req.get("exported_at").and_then(Json::as_str)
            ),
            "bbsa": bbsa_text,
            "unmapped": unmapped,
        }),
        d,
    )
}

const SEATS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];
const STRAIN_KEYS: [(Strain, &str); 5] = [
    (Strain::Clubs, "C"),
    (Strain::Diamonds, "D"),
    (Strain::Hearts, "H"),
    (Strain::Spades, "S"),
    (Strain::NoTrump, "NT"),
];

/// The double-dummy table of a deal, par, and (with a finished `auction`
/// and its `dealer`) the tricks and score of the contract reached.
pub fn dd_table(request: &str) -> String {
    let mut d = Diags::default();
    let req = parse_request(request, &mut d);
    let empty = json!({"tricks": Json::Null, "par": Json::Null, "result": Json::Null});
    if d.has_errors() {
        return respond(empty, d);
    }
    let hands = match req.get("deal") {
        Some(v) => parse_deal(v, &mut d),
        None => {
            d.error("deal: missing")
                .hint("N:<north> <east> <south> <west>, each S.H.D.C");
            None
        }
    };
    let vul = vulnerability(&req, &mut d);
    let has_auction = req.get("auction").is_some();
    let dealer = seat(
        &req,
        "dealer",
        if has_auction {
            None
        } else {
            Some(Direction::North)
        },
        &mut d,
    );
    let calls = match (has_auction, dealer) {
        (true, Some(dl)) => parse_auction(req.get("auction"), "auction", dl, &mut d),
        _ => None,
    };
    let Some(hands) = hands else {
        return respond(empty, d);
    };
    if d.has_errors() {
        return respond(empty, d);
    }
    let mut deal = Deal::new();
    for (i, h) in hands.iter().enumerate() {
        if let Some(dir) = Direction::from_index(i) {
            for c in h.cards() {
                deal.hand_mut(dir).add_card(*c);
            }
        }
    }
    let table = bridge_solver::par::solve_dd_table(&deal);
    let mut tricks = Map::new();
    for seat in SEATS {
        let row: Map<String, Json> = STRAIN_KEYS
            .iter()
            .map(|(s, k)| (k.to_string(), json!(table.tricks(seat, *s))))
            .collect();
        tricks.insert(seat_str(seat), Json::Object(row));
    }
    let p = bridge_solver::par::par(
        &table,
        vul.is_vulnerable(Direction::North),
        vul.is_vulnerable(Direction::East),
    );
    let par = json!({
        "score_ns": p.score_ns,
        "contracts": p.contracts.iter().map(|c| c.describe()).collect::<Vec<_>>(),
    });
    let mut result = Json::Null;
    if let (Some(calls), Some(dealer)) = (calls, dealer) {
        let mut a = Auction::new(dealer);
        for c in &calls {
            a.add_call(c.clone());
        }
        if !a.is_complete() {
            d.info("auction: not finished, so no result");
        } else {
            result = match a.final_contract() {
                None => {
                    json!({"contract": "Pass", "declarer": Json::Null, "tricks": Json::Null, "score_ns": 0})
                }
                Some(fc) => {
                    let taken = table.tricks(fc.declarer, fc.strain) as i32;
                    let doubled = if fc.redoubled {
                        Doubled::Redoubled
                    } else if fc.doubled {
                        Doubled::Doubled
                    } else {
                        Doubled::None
                    };
                    let score = Contract::new(fc.level, fc.strain, doubled, fc.declarer.to_char())
                        .score(
                            taken - (fc.level as i32 + 6),
                            vul.is_vulnerable(fc.declarer),
                        );
                    let score_ns = match fc.declarer {
                        Direction::North | Direction::South => score,
                        _ => -score,
                    };
                    json!({
                        "contract": fc.to_pbn(),
                        "declarer": seat_str(fc.declarer),
                        "tricks": taken,
                        "score_ns": score_ns,
                    })
                }
            };
        }
    }
    respond(json!({"tricks": tricks, "par": par, "result": result}), d)
}

fn severity_word(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Json {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn info_lists_rules_and_cards() {
        let r = parse(&info());
        assert_eq!(r["ok"], true, "{r}");
        assert!(r["modules"].as_u64().unwrap() > 10);
        assert!(r["stock_cards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "21GF-DEFAULT"));
    }

    #[test]
    fn bids_a_hand_like_the_engine_does() {
        let r = parse(&bid(r#"{"cards": {"ns": "21GF-DEFAULT", "ew": "21GF-GIB"},
                "hand": "AK52.KJ7.Q94.K83", "dealer": "N", "vul": "None",
                "scoring": "IMP", "auction": ""}"#));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["call"], "1NT", "{r}");
        assert_eq!(r["seat"], "N");
        assert!(r["candidates"].as_array().unwrap().len() > 1);

        // Same answer as the engine on the files, through rbb-assets.
        let rules = rbb_engine::load_rules(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../conventions"
        )))
        .unwrap();
        let stock = |name: &str| {
            bbsa::import(&rules.vocab, rbb_assets::card(name).unwrap(), None)
                .unwrap()
                .0
        };
        let (card, ew) = (stock("21GF-DEFAULT"), stock("21GF-GIB"));
        let e = Engine::new(&card, &ew, &rules);
        let hand = Hand::from_pbn("AK52.KJ7.Q94.K83").unwrap();
        let dec = e.bid(
            &hand,
            Direction::North,
            Vulnerability::None,
            ScoringMethod::IMP,
            &[],
        );
        assert_eq!(r["call"], dec.call.to_pbn());
        assert_eq!(r["explanation"], dec.explanation);
    }

    #[test]
    fn bid_and_interpret_give_the_position() {
        // South to call after 1NT: North's range is known, East's is not.
        let r = parse(&bid(r#"{"cards": {"ns": "21GF-DEFAULT", "ew": "21GF-GIB"},
                "hand": "T64.AT832.A2.T62", "dealer": "N", "auction": "1NT Pass"}"#));
        assert_eq!(r["ok"], true, "{r}");
        let k = &r["position"]["knowledge"];
        assert_eq!(k["N"]["hcp"]["min"], 15, "{r}");
        assert_eq!(k["N"]["balanced"], true);
        assert_eq!(k["S"]["hcp"]["min"], 0);
        assert!(r["position"]["sides"]["ns"]["forcing"].is_string());
        assert!(r["position"]["sides"]["ew"]["summary"]
            .as_str()
            .unwrap()
            .contains("forcing none"));
        assert!(r["warnings"].is_array());

        // After a new suit by responder the side is forced for a round.
        let r = parse(&interpret(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "auction": "1C Pass 1H"}"#,
        ));
        assert_eq!(r["ok"], true, "{r}");
        let ns = &r["position"]["sides"]["ns"];
        assert_eq!(ns["forcing"], "round", "{r}");
        assert_eq!(ns["forcing_by"], "S", "{r}");
        assert!(
            r["position"]["knowledge"]["S"]["hcp"]["min"]
                .as_i64()
                .unwrap()
                >= 5
        );
    }

    #[test]
    fn engines_are_made_once_and_reused() {
        let a = parse(&create_engine(r#"{"cards": {"ns": "21GF-DEFAULT"}}"#));
        let b = parse(&create_engine(r#"{"cards": {"ns": "21GF-DEFAULT"}}"#));
        assert_eq!(a["ok"], true, "{a}");
        assert_eq!(a["engine"], b["engine"]);
        assert!(a["ns"]["modules"].as_array().unwrap().len() > 5);
        let id = a["engine"].as_u64().unwrap();
        let r = parse(&interpret(&format!(
            r#"{{"engine": {id}, "dealer": "N", "auction": "1NT Pass 2C Pass"}}"#
        )));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["steps"].as_array().unwrap().len(), 4);
        assert_eq!(r["steps"][2]["call"], "2C");
        assert_eq!(r["next"], "N");
        assert!(r["steps"][0]["knowledge"]["hcp"]["min"].as_i64().unwrap() >= 14);
        let f = parse(&free_engine(&format!(r#"{{"engine": {id}}}"#)));
        assert_eq!(f["ok"], true);
        let gone = parse(&interpret(&format!(r#"{{"engine": {id}, "dealer": "N"}}"#)));
        assert_eq!(gone["ok"], false);
    }

    #[test]
    fn bad_input_comes_back_as_diagnostics() {
        let r = parse(&bid(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "hand": "AK52.KJ7.Q94", "dealer": "N",
                "auction": "1NT 1C"}"#,
        ));
        assert_eq!(r["ok"], false);
        let diags = r["diagnostics"].as_array().unwrap();
        assert!(diags
            .iter()
            .any(|x| x["message"].as_str().unwrap().starts_with("hand:")));
        let illegal = diags
            .iter()
            .find(|x| x["message"].as_str().unwrap().contains("not legal"))
            .expect("illegal call reported");
        assert_eq!(illegal["severity"], "error");
        assert_eq!(illegal["col"], 5);
        assert!(illegal["hint"].is_string());

        let v = parse(&validate(r#"{"cards": {"ns": "NoSuchCard"}}"#));
        assert_eq!(v["ok"], false);
        assert!(v["diagnostics"][0]["hint"]
            .as_str()
            .unwrap()
            .contains("21GF-DEFAULT"));

        let v = parse(&validate(
            r#"{"cards": {"ns": {"json": {"notrump": {"no_such_field": true}}}}}"#,
        ));
        assert_eq!(v["ok"], true, "{v}");
        assert_eq!(v["diagnostics"][0]["severity"], "warning", "{v}");

        let v = parse(&validate("{not json"));
        assert_eq!(v["ok"], false);
        assert_eq!(v["diagnostics"][0]["line"], 1);

        let v = parse(&validate(
            r#"{"hand": "AK52.KJ7.Q94.K83", "dealer": "S", "auction": "1NT X XX"}"#,
        ));
        assert_eq!(v["ok"], true, "{v}");
    }

    #[test]
    fn bids_a_whole_deal() {
        let r = parse(&bid_deal(
            r#"{"cards": {"ns": "21GF-DEFAULT", "ew": "21GF-GIB"},
                "deal": "N:AK52.KJ7.Q94.K83 QJ3.Q95.KJ3.QJ74 T64.AT832.A2.T62 987.64.T8765.A95",
                "dealer": "N", "vul": "None"}"#,
        ));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["complete"], true);
        assert_eq!(r["calls"][0]["call"], "1NT");
        assert_eq!(r["calls"][0]["forced"], false);
        assert!(r["contract"].is_string());

        let forced = parse(&bid_deal(
            r#"{"cards": {"ns": "21GF-DEFAULT"},
                "deal": {"N": "AK52.KJ7.Q94.K83", "E": "QJ3.Q95.KJ3.QJ74",
                         "S": "T64.AT832.A2.T62", "W": "987.64.T8765.A95"},
                "dealer": "N", "prefix": ["1C"]}"#,
        ));
        assert_eq!(forced["ok"], true, "{forced}");
        assert_eq!(forced["calls"][0]["call"], "1C");
        assert_eq!(forced["calls"][0]["forced"], true);
    }

    /// North and East human, South and West bots: after 1C (4NT) no rule
    /// covers South's hand.
    const TABLE: &str = r#""cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "vul": "None",
        "scoring": "IMP", "bots": ["S", "W"],
        "hands": {"S": "AK52.KJ7.Q94.K83", "W": "QJ3.Q95.KJ3.QJ74"}"#;

    #[test]
    fn auction_stops_at_no_rule_and_resumes_after_a_call_from_outside() {
        let r = parse(&auction(&format!(r#"{{{TABLE}, "auction": "1C 4NT"}}"#)));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["stop"]["reason"], "no_rule", "{r}");
        assert_eq!(r["stop"]["seat"], "S");
        assert_eq!(r["stop"]["index"], 2);
        assert_eq!(r["stop"]["auction"], json!(["1C", "4NT"]));
        assert_eq!(r["stop"]["call"], "Pass");
        assert!(r["stop"]["candidates"].is_array());
        assert_eq!(r["calls"], json!([]));
        assert_eq!(r["steps"].as_array().unwrap().len(), 2);
        assert_eq!(r["steps"][1]["by"], "given");
        assert_eq!(r["next"], "S");
        assert_eq!(r["complete"], false);
        assert_eq!(r["cards"]["ns"], "21GF-DEFAULT");
        assert_eq!(r["rules_id"], rbb_assets::RULES_ID);
        assert!(r["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["severity"] == "info"));

        // The fallback's pass appended: West bids, North (human) is next.
        let r = parse(&auction(&format!(
            r#"{{{TABLE}, "auction": "1C 4NT Pass"}}"#
        )));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["stop"]["reason"], "human_to_call", "{r}");
        assert_eq!(r["stop"]["seat"], "N");
        assert_eq!(r["stop"]["index"], 4);
        assert!(r["stop"].get("candidates").is_none());
        let made = r["calls"].as_array().unwrap();
        assert_eq!(made.len(), 1);
        assert_eq!(made[0]["seat"], "W");
        assert_eq!(made[0]["index"], 3);
        assert_eq!(made[0]["no_rule"], false);
        assert_eq!(made[0]["forced"], false);
        assert!(made[0]["rule"].is_object());
        let steps = r["steps"].as_array().unwrap();
        assert_eq!(steps.len(), 4);
        assert_eq!(steps[2]["by"], "given");
        assert_eq!(steps[3]["by"], "bot");

        // Every call reads as `interpret` reads it.
        let i = parse(&interpret(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "vul": "None", "scoring": "IMP",
                "auction": "1C 4NT Pass Pass"}"#,
        ));
        for (a, b) in steps.iter().zip(i["steps"].as_array().unwrap()) {
            for k in ["call", "explanation", "alert", "rule", "knowledge"] {
                assert_eq!(a[k], b[k], "{k}");
            }
        }
        assert_eq!(r["position"], i["position"]);

        // Passing without a rule instead of stopping.
        let r = parse(&auction(&format!(
            r#"{{{TABLE}, "auction": "1C 4NT", "no_rule": "pass"}}"#
        )));
        assert_eq!(r["stop"]["reason"], "human_to_call", "{r}");
        assert_eq!(r["calls"][0]["no_rule"], true);
        assert_eq!(r["calls"][0]["rule"], Json::Null);
        assert_eq!(r["calls"][0]["explanation"], "No rule applies");
    }

    #[test]
    fn auction_bids_to_the_end_with_four_bots() {
        let deal = "N:AK52.KJ7.Q94.K83 QJ3.Q95.KJ3.QJ74 T64.AT832.A2.T62 987.64.T8765.A95";
        let r = parse(&auction(&format!(
            r#"{{"cards": {{"ns": "21GF-DEFAULT", "ew": "21GF-GIB"}}, "deal": "{deal}",
                "dealer": "N", "bots": "NESW", "no_rule": "pass"}}"#
        )));
        let b = parse(&bid_deal(&format!(
            r#"{{"cards": {{"ns": "21GF-DEFAULT", "ew": "21GF-GIB"}}, "deal": "{deal}",
                "dealer": "N"}}"#
        )));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["stop"]["reason"], "complete");
        assert_eq!(r["stop"]["seat"], Json::Null);
        assert_eq!(r["complete"], true);
        assert_eq!(r["contract"], b["contract"]);
        let calls = |x: &Json| -> Vec<Json> {
            x.as_array()
                .unwrap()
                .iter()
                .map(|c| c["call"].clone())
                .collect()
        };
        assert_eq!(calls(&r["calls"]), calls(&b["calls"]));
        // A human at the dealer's seat: nothing to bid yet.
        let r = parse(&auction(&format!(
            r#"{{"cards": {{"ns": "21GF-DEFAULT"}}, "deal": "{deal}", "dealer": "N",
                "bots": ["E", "S", "W"]}}"#
        )));
        assert_eq!(r["stop"]["reason"], "human_to_call", "{r}");
        assert_eq!(r["stop"]["index"], 0);
    }

    #[test]
    fn auction_reports_bad_tables() {
        let bad = |req: &str, starts: &str| {
            let r = parse(&auction(req));
            assert_eq!(r["ok"], false, "{r}");
            assert!(
                r["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|x| x["message"].as_str().unwrap().starts_with(starts)),
                "{starts}: {r}"
            );
            assert!(r["stop"].is_null());
        };
        bad(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "bots": ["S"], "hands": {"W": "AK52.KJ7.Q94.K83"}}"#,
            "hands: S is a bot seat",
        );
        bad(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "hands": {"S": "AK52.KJ7.Q94.K83"}}"#,
            "bots: missing",
        );
        bad(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "bots": "S",
                "hands": {"S": "AK52.KJ7.Q94.K83", "W": "AK52.KJ7.Q94.K83"}}"#,
            "hands: ",
        );
        bad(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "bots": "S", "no_rule": "guess",
                "hands": {"S": "AK52.KJ7.Q94.K83"}}"#,
            "no_rule:",
        );
    }

    #[test]
    fn meaning_of_one_call() {
        let r = parse(&meaning(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "auction": "1NT Pass 2C Pass", "index": 2}"#,
        ));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["index"], 2);
        assert_eq!(r["known"], true);
        assert_eq!(r["step"]["call"], "2C");
        assert_eq!(r["step"]["seat"], "S");
        let i = parse(&interpret(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "auction": "1NT Pass 2C Pass"}"#,
        ));
        for k in ["explanation", "alert", "rule", "knowledge", "artificial"] {
            assert_eq!(r["step"][k], i["steps"][2][k], "{k}");
        }
        // The last call by default; a call outside the system is unknown.
        let r = parse(&meaning(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "auction": "1C 4NT"}"#,
        ));
        assert_eq!(r["index"], 1, "{r}");
        let unknown = parse(&meaning(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "auction": "2C 3NT Pass 7C"}"#,
        ));
        assert_eq!(unknown["ok"], true, "{unknown}");
        assert_eq!(unknown["known"], false, "{unknown}");
        assert_eq!(unknown["step"]["explanation"], Json::Null);
        let r = parse(&meaning(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N", "auction": "1C", "index": 1}"#,
        ));
        assert_eq!(r["ok"], false);
        let r = parse(&meaning(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "dealer": "N"}"#,
        ));
        assert_eq!(r["ok"], false);
    }

    #[test]
    fn coverage_splits_what_the_rules_read() {
        let r = parse(&coverage(
            r#"{"cards": {"ns": "21GF-DEFAULT", "ew": "Precision"}}"#,
        ));
        assert_eq!(r["ok"], true, "{r}");
        assert!(!r["ns"]["read"].as_array().unwrap().is_empty(), "{r}");
        assert!(r["ns"]["score"].as_f64().unwrap() > 0.0);
        assert!(r["ew"]["ignored"].is_array());
        assert!(r["ew"]["unmapped"].is_array());
        let bad = parse(&coverage("{}"));
        assert_eq!(bad["ok"], false);
    }

    #[test]
    fn exports_a_card_both_ways() {
        let r = parse(&export_card(r#"{"card": "21GF-DEFAULT"}"#));
        assert_eq!(r["ok"], true, "{r}");
        assert!(r["json"].is_object());
        let bbsa = r["bbsa"].as_str().unwrap();
        assert!(!bbsa.is_empty());
        // The exported JSON loads back as a card.
        let back = parse(&validate(
            &json!({"cards": {"ns": {"json": r["json"]}}}).to_string(),
        ));
        assert_eq!(back["ok"], true, "{back}");
        // So does Bridge-Classroom's export wrapper, with its name.
        let wrapped = parse(&export_card(
            r#"{"card": "21GF-DEFAULT", "exported_at": "2026-09-28T00:00:00.000Z"}"#,
        ));
        let bc = &wrapped["bridge_classroom"];
        assert_eq!(bc["schema"], "bridge-classroom/card_data@v1");
        assert_eq!(bc["exportedAt"], "2026-09-28T00:00:00.000Z");
        assert_eq!(bc["card_data"], r["json"]);
        let again = parse(&export_card(&json!({"card": {"json": bc}}).to_string()));
        assert_eq!(again["ok"], true, "{again}");
        assert_eq!(again["json"], r["json"]);
    }

    #[test]
    fn reads_bridge_classroom_exports() {
        // A synthetic export, shaped like the editor's: the wrapper, a raw
        // BBO record, and a setting the vocabulary does not have.
        let card = json!({
            "schema": "bridge-classroom/card_data@v1",
            "name": "Pat and Sam", "description": "Imported from BBO",
            "exportedAt": "2026-09-28T20:04:58.311Z",
            "card_data": {
                "schema_version": "1.0", "format": "bridge_classroom",
                "metadata": {"name": "Pat and Sam", "source": "bbo"},
                "_bbo_raw": {"conventions": {"1NStayman": "y"}},
                "notrump": {"stayman": {"play": true}, "mystery": {"on": true}}
            }
        });
        let v = parse(&validate(
            &json!({"cards": {"ns": {"json": card}}}).to_string(),
        ));
        assert_eq!(v["ok"], true, "{v}");
        let raw: Vec<&Json> = v["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["message"].as_str().unwrap().contains("_bbo_raw"))
            .collect();
        assert_eq!(raw.len(), 1, "{v}");
        assert_eq!(raw[0]["severity"], "info");
        let c = parse(&coverage(
            &json!({"cards": {"ns": {"json": card}}}).to_string(),
        ));
        assert_eq!(c["ok"], true, "{c}");
        assert_eq!(c["ns"]["name"], "Pat and Sam");
        assert!(
            c["ns"]["read"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p == "notrump.stayman.play"),
            "{c}"
        );
        assert_eq!(c["ns"]["unmapped"], json!(["notrump.mystery.on"]));
        // Card JSON as a string, the way a page reads an uploaded file.
        let e = parse(&create_engine(
            &json!({"cards": {"ns": {"json": card.to_string()}}}).to_string(),
        ));
        assert_eq!(e["ok"], true, "{e}");
        assert_eq!(e["ns"]["name"], "Pat and Sam");
    }

    #[test]
    fn dd_table_par_and_result() {
        let r = parse(&dd_table(
            r#"{"deal": "N:AK52.KJ7.Q94.K83 QJ3.Q95.KJ3.QJ74 T64.AT832.A2.T62 987.64.T8765.A95",
                "vul": "None", "dealer": "N", "auction": "1NT Pass 3NT Pass Pass Pass"}"#,
        ));
        assert_eq!(r["ok"], true, "{r}");
        for seat in ["N", "E", "S", "W"] {
            for s in ["C", "D", "H", "S", "NT"] {
                let t = r["tricks"][seat][s].as_u64().unwrap();
                assert!(t <= 13);
            }
        }
        let n = r["tricks"]["N"]["NT"].as_u64().unwrap();
        let e = r["tricks"]["E"]["NT"].as_u64().unwrap();
        assert_eq!(n + e, 13, "{r}");
        assert_eq!(r["result"]["contract"], "3N");
        assert_eq!(r["result"]["declarer"], "N");
        assert_eq!(r["result"]["tricks"], n);
        assert!(r["par"]["score_ns"].is_i64());
        let bad = parse(&dd_table(r#"{"deal": "N:AK52"}"#));
        assert_eq!(bad["ok"], false);
    }

    #[test]
    fn reference_text_matches_the_data() {
        let t = reference(r#"{"cards": {"ns": "21GF-DEFAULT"}}"#);
        assert!(
            t.starts_with("# rusty-bidding-bot conventions reference"),
            "{t}"
        );
        assert!(t.contains("== stayman"), "{}", &t[..500]);
        assert!(t.contains("[on]") && t.contains("[off]"));
        let c = parse(&conventions("{}"));
        assert_eq!(c["ok"], true);
        let n = c["modules"].as_array().unwrap().len();
        assert_eq!(t.matches("\n== ").count(), n);
    }

    /// A rule set of its own: one field, one module, one rule.
    fn demo_rules(rule: &str) -> Json {
        json!({
            "manifest": "name = \"demo\"\nlanguage = 1\n",
            "fields": "[demo]\n\"strong_nt\" = { kind = \"bool\", label = \"1NT\", default = true }\n\"nt_min\" = { kind = \"int\", label = \"1NT minimum\", min = 10, max = 20, default = 15 }\n",
            "bbsa_map": "",
            "rules": {
                "demo/one-nt.bid": format!(
                    "module demo \"Demo\"\n  card   demo.strong_nt\n  param  lo = demo.nt_min default 15\n\nwhen opening\n{rule}\n"
                ),
                "demo/one-nt.notes.md": "not a rule file",
            },
        })
    }

    const RULE: &str = "  1N  \"Demo 1NT\"  shows hcp=lo..17, balanced";

    /// The first error of a response.
    fn error(r: &Json) -> &Json {
        r["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["severity"] == "error")
            .unwrap_or(&Json::Null)
    }

    #[test]
    fn a_rule_set_supplied_at_run_time_bids() {
        let mut req = demo_rules(RULE);
        req["cards"] = json!({"ns": {"json": {"demo": {"nt_min": 15}}}});
        let e = parse(&create_engine(&req.to_string()));
        assert_eq!(e["ok"], true, "{e}");
        assert_eq!(e["ns"]["modules"], json!(["demo"]), "{e}");
        assert_ne!(e["rules_id"], rbb_assets::RULES_ID);
        let id = e["engine"].as_u64().unwrap();
        let bid_with = |engine: u64| {
            parse(&bid(&json!({"engine": engine, "hand": "AK52.KJ7.Q94.K83",
                                "dealer": "N", "auction": ""})
            .to_string()))
        };
        let r = bid_with(id);
        assert_eq!(r["call"], "1NT", "{r}");
        assert_eq!(r["rule"]["file"], "demo/one-nt.bid", "{r}");

        // The card is read in the rule set's vocabulary: its own field,
        // through `set` too.
        let mut strict = demo_rules(RULE);
        strict["cards"] = json!({"ns": {"json": {}}, "set": ["demo.nt_min=17"]});
        let e17 = parse(&create_engine(&strict.to_string()));
        assert_eq!(e17["ok"], true, "{e17}");
        assert_ne!(e17["engine"], e["engine"]);
        assert_eq!(bid_with(e17["engine"].as_u64().unwrap())["call"], "Pass");
        // ...and not in the embedded one.
        let bad = parse(&create_engine(
            r#"{"cards": {"ns": "21GF-DEFAULT", "set": ["demo.nt_min=17"]}}"#,
        ));
        assert_eq!(bad["ok"], false, "{bad}");

        // The same rule set again is the same engine; the embedded rules
        // are untouched.
        let again = parse(&create_engine(&req.to_string()));
        assert_eq!(again["engine"], e["engine"]);
        let std = parse(&bid(
            r#"{"cards": {"ns": "21GF-DEFAULT"}, "hand": "AK52.KJ7.Q94.K83",
                "dealer": "N", "auction": ""}"#,
        ));
        assert_eq!(
            std["rule"]["file"], "conventions/notrump/one-nt.bid",
            "{std}"
        );

        // conventions, coverage and exportCard follow the engine's rules.
        let c = parse(&conventions(&format!(r#"{{"engine": {id}}}"#)));
        assert_eq!(c["modules"].as_array().unwrap().len(), 1, "{c}");
        assert_eq!(c["rules_id"], e["rules_id"]);
        let x = parse(&export_card(&format!(
            r#"{{"engine": {id}, "card": {{"json": {{}}}}, "set": ["demo.nt_min=16"]}}"#
        )));
        assert_eq!(x["ok"], true, "{x}");
        assert_eq!(x["json"]["demo"]["nt_min"], 16, "{x}");
        let mut cov = demo_rules(RULE);
        cov["cards"] = json!({"ns": {"json": {"demo": {"strong_nt": true}}}});
        let cov = parse(&coverage(&cov.to_string()));
        assert_eq!(cov["ns"]["read"], json!([]), "at its default: {cov}");
    }

    #[test]
    fn validate_reports_rule_set_problems_by_file_and_line() {
        let ok = parse(&validate(&demo_rules(RULE).to_string()));
        assert_eq!(ok["ok"], true, "{ok}");
        assert_eq!(ok["rule_set"]["modules"], 1);
        assert_eq!(ok["rule_set"]["name"], "demo");
        assert_eq!(ok["rule_set"]["card_fields"], 2);
        assert!(ok["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["message"].as_str().unwrap().contains("not .bid")));

        // A card field the vocabulary does not have.
        let mut bad = demo_rules(RULE);
        bad["rules"]["demo/one-nt.bid"] = json!("module demo \"Demo\"\n  card   demo.nope\n");
        let v = parse(&validate(&bad.to_string()));
        assert_eq!(v["ok"], false, "{v}");
        let x = &error(&v);
        assert_eq!(x["file"], "demo/one-nt.bid", "{v}");
        assert_eq!(x["line"], 2, "{v}");
        assert!(x["message"]
            .as_str()
            .unwrap()
            .starts_with("rules: demo/one-nt.bid:2: unknown card field"));

        // A term the engine does not know (check_terms).
        let v = parse(&validate(
            &demo_rules("  1N  \"Demo\"  shows hcpx>=15").to_string(),
        ));
        assert_eq!(v["ok"], false, "{v}");
        assert_eq!(error(&v)["file"], "demo/one-nt.bid", "{v}");
        assert_eq!(error(&v)["line"], 6, "{v}");

        // The same through createEngine: no engine.
        let mut req = demo_rules("  1N  \"Demo\"  shows hcpx>=15");
        req["cards"] = json!({"ns": {"json": {}}});
        let e = parse(&create_engine(&req.to_string()));
        assert_eq!(e["ok"], false);
        assert_eq!(e["engine"], Json::Null);

        // A vocabulary that does not parse, with its line.
        let mut bad = demo_rules(RULE);
        bad["fields"] = json!("[demo]\nstrong_nt = {");
        let v = parse(&validate(&bad.to_string()));
        assert_eq!(v["ok"], false, "{v}");
        assert!(error(&v)["message"]
            .as_str()
            .unwrap()
            .starts_with("fields: "));
        assert_eq!(error(&v)["line"], 2, "{v}");

        // A manifest in a language this engine does not read.
        let mut bad = demo_rules(RULE);
        bad["manifest"] = json!("name = \"demo\"\nlanguage = 99\n");
        let v = parse(&validate(&bad.to_string()));
        assert_eq!(v["ok"], false, "{v}");
        assert!(error(&v)["message"]
            .as_str()
            .unwrap()
            .starts_with("manifest: "));

        // No manifest: read as the current language, and said so.
        let mut none = demo_rules(RULE);
        none.as_object_mut().unwrap().remove("manifest");
        let v = parse(&validate(&none.to_string()));
        assert_eq!(v["ok"], true, "{v}");
        assert!(v["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["message"]
                .as_str()
                .unwrap()
                .starts_with("manifest: none given")));
    }

    #[test]
    fn a_vocabulary_alone_keeps_the_embedded_rules() {
        // The embedded rules against a copy of their own vocabulary: the
        // same rule set, so the same calls.
        let req = json!({"fields": rbb_assets::FIELDS.1, "bbsa_map": rbb_assets::BBSA_MAP.1});
        let v = parse(&validate(&req.to_string()));
        assert_eq!(v["ok"], true, "{v}");
        assert_eq!(
            v["rule_set"]["rule_files"].as_u64().unwrap() as usize,
            rbb_assets::RULE_FILES.len()
        );
        // A vocabulary without the fields the embedded rules read refuses.
        let v = parse(&validate(r#"{"fields": ""}"#));
        assert_eq!(v["ok"], false);
        assert!(error(&v)["message"]
            .as_str()
            .unwrap()
            .starts_with("bbsa_map: the embedded convention-card/spec/formats/bbsa-map.toml"));
    }
}
