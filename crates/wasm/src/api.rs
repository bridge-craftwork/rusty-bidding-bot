//! The WASM API as plain Rust, so it is tested natively. Every function takes
//! a JSON request (a string) and returns a JSON response (a string), except
//! `reference`, which returns plain text. Nothing panics or throws on bad
//! input: problems come back as `diagnostics`, each
//! `{severity, message, line?, col?, hint?}`, and `ok` is false when one of
//! them is an error. docs/WASM.md is the contract.

use std::cell::RefCell;
use std::sync::{Arc, OnceLock};

use bridge_card::{bbsa, Card};
use bridge_types::{Auction, Call, Deal, Direction, Hand, ScoringMethod, Vulnerability};
use rbb_engine::{CandidateTrace, Engine, RuleRef, SeatKnowledge, Step, Tri};
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
/// message names (the auction string, the `.bbsa` text).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
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

/// The embedded rules, compiled once.
fn modules() -> &'static Result<Vec<bidspec::Module>, Vec<bidspec::Diagnostic>> {
    static MODULES: OnceLock<Result<Vec<bidspec::Module>, Vec<bidspec::Diagnostic>>> =
        OnceLock::new();
    MODULES.get_or_init(|| rbb_engine::compile_modules(rbb_assets::RULE_FILES.iter().copied()))
}

fn modules_or_report(d: &mut Diags) -> Option<&'static [bidspec::Module]> {
    match modules() {
        Ok(m) => Some(m),
        Err(errors) => {
            for e in errors {
                d.error(format!("embedded rules: {e}"));
            }
            None
        }
    }
}

struct Entry {
    id: u32,
    key: String,
    engine: Arc<Engine>,
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

/// One card from its spec: a stock card's name, `{"stock": name}`,
/// `{"bbsa": text, "name"?: ..}` or `{"json": card_data}`.
fn load_card(spec: &Json, field: &str, d: &mut Diags) -> Option<(Card, String)> {
    let stock = |name: &str, d: &mut Diags| match rbb_assets::card(name) {
        Some(text) => import_bbsa(text, Some(name), field, d).map(|c| (c, name.to_string())),
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
                import_bbsa(text, Some(name), field, d).map(|c| (c, name.to_string()))
            } else if let Some(card) = m.get("json") {
                let text = match card {
                    Json::String(s) => s.clone(),
                    other => other.to_string(),
                };
                match Card::from_json(&text) {
                    Ok((card, report)) => {
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
                        Some((card, name))
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

fn import_bbsa(text: &str, name: Option<&str>, field: &str, d: &mut Diags) -> Option<Card> {
    match bbsa::import(text, name) {
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
            Some(card)
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
                    d.error(format!("{field}: expected strings like \"general.style=bba\""));
                    None
                }
            })
            .collect(),
        Some(Json::String(s)) => vec![s.clone()],
        Some(_) => {
            d.error(format!("{field}: expected a list of \"path=value\" strings"));
            Vec::new()
        }
    }
}

/// Both cards from `{"ns", "ew"?, "set"?, "ns_set"?, "ew_set"?}`.
fn load_cards(cards: &Json, d: &mut Diags) -> Option<([Card; 2], [String; 2])> {
    let Json::Object(m) = cards else {
        d.error("cards: expected an object like {\"ns\": \"21GF-DEFAULT\", \"ew\": \"21GF-GIB\"}");
        return None;
    };
    let Some(ns_spec) = m.get("ns") else {
        d.error("cards.ns: missing")
            .hint("the North-South card, e.g. \"21GF-DEFAULT\"");
        return None;
    };
    let ns = load_card(ns_spec, "cards.ns", d);
    let ew = match m.get("ew") {
        Some(spec) => load_card(spec, "cards.ew", d),
        None => ns.clone(),
    };
    let both = string_list(m.get("set"), "cards.set", d);
    let mut out = Vec::new();
    for (side, card, own) in [("ns", ns, "ns_set"), ("ew", ew, "ew_set")] {
        let (mut card, name) = card?;
        let own = string_list(m.get(own), &format!("cards.{own}"), d);
        for change in both.iter().chain(&own) {
            if let Err(e) = card.apply_change(change) {
                d.error(format!("cards.{side}: {e}"));
            }
        }
        out.push((card, name));
    }
    if d.has_errors() {
        return None;
    }
    let (ew, ns) = (out.pop().unwrap(), out.pop().unwrap());
    Some(([ns.0, ew.0], [ns.1, ew.1]))
}

fn engine_by_id(id: u32) -> Option<(Arc<Engine>, [String; 2])> {
    ENGINES.with(|r| {
        r.borrow()
            .1
            .iter()
            .find(|e| e.id == id)
            .map(|e| (e.engine.clone(), e.names.clone()))
    })
}

/// The engine for `cards`, made once per distinct request.
fn engine_for_cards(cards: &Json, d: &mut Diags) -> Option<(u32, Arc<Engine>, [String; 2])> {
    let key = cards.to_string();
    if let Some(found) = ENGINES.with(|r| {
        r.borrow()
            .1
            .iter()
            .find(|e| e.key == key)
            .map(|e| (e.id, e.engine.clone(), e.names.clone()))
    }) {
        return Some(found);
    }
    let modules = modules_or_report(d)?;
    let (cards, names) = load_cards(cards, d)?;
    let engine = Arc::new(Engine::new(&cards[0], &cards[1], modules));
    let id = ENGINES.with(|r| {
        let mut r = r.borrow_mut();
        r.0 += 1;
        let id = r.0;
        r.1.push(Entry {
            id,
            key,
            engine: engine.clone(),
            names: names.clone(),
        });
        id
    });
    Some((id, engine, names))
}

/// The engine a request names: `"engine": id`, or `"cards": {...}`.
fn engine_for(req: &Json, d: &mut Diags) -> Option<(u32, Arc<Engine>)> {
    if let Some(v) = req.get("engine") {
        let Some(id) = v.as_u64() else {
            d.error("engine: expected the number create_engine returned");
            return None;
        };
        return match engine_by_id(id as u32) {
            Some((e, _)) => Some((id as u32, e)),
            None => {
                d.error(format!("engine: no engine {id}"))
                    .hint("create one with createEngine (it may have been freed)");
                None
            }
        };
    }
    if let Some(cards) = req.get("cards") {
        return engine_for_cards(cards, d).map(|(id, e, _)| (id, e));
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
                d.error(format!("{field}: {s:?} is not a seat")).hint("N, E, S or W");
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
                d.error(format!("scoring: {s:?} is not MP or IMP")).hint("MP or IMP");
                ScoringMethod::Matchpoints
            }
        },
    }
}

const HAND_HINT: &str = "spades.hearts.diamonds.clubs, e.g. AK52.KJ7.Q94.K83 (T for ten, - for a void)";

/// A 13-card hand, S.H.D.C.
fn parse_hand(text: &str, field: &str, d: &mut Diags) -> Option<Hand> {
    let Some(hand) = Hand::from_pbn(text) else {
        d.error(format!("{field}: {text:?} is not a hand")).hint(HAND_HINT);
        return None;
    };
    let cards = hand.cards();
    for (i, c) in cards.iter().enumerate() {
        if cards[..i].contains(c) {
            d.error(format!("{field}: {c} appears twice")).hint(HAND_HINT);
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
            for dir in [Direction::North, Direction::East, Direction::South, Direction::West] {
                let h = deal.hand(dir);
                hands[dir.to_index()] = parse_hand(&h.to_pbn(), &format!("deal.{}", dir.to_char()), d);
            }
        }
        Json::Object(m) => {
            for dir in [Direction::North, Direction::East, Direction::South, Direction::West] {
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
fn parse_auction(v: Option<&Json>, field: &str, dealer: Direction, d: &mut Diags) -> Option<Vec<Call>> {
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
            d.error(format!("{field}: expected a string like \"1NT Pass\" or a list of calls"));
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
            d.error(format!("{field}: call {n} ({tok}): the auction is already over"))
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
        Some(fc) => (Json::String(fc.to_pbn()), Json::String(seat_str(fc.declarer))),
        None => (Json::String("Pass".into()), Json::Null),
    }
}

// ── The API ────────────────────────────────────────────────────────────────

/// What this build is: API and crate versions, the rules, the stock cards.
pub fn info() -> String {
    let mut d = Diags::default();
    let (modules, rules) = match modules_or_report(&mut d) {
        Some(m) => (
            m.len(),
            bidspec::reference::entries(m)
                .iter()
                .map(|m| m.rules.len())
                .sum::<usize>(),
        ),
        None => (0, 0),
    };
    respond(
        json!({
            "api": API_VERSION,
            "version": env!("CARGO_PKG_VERSION"),
            "rules_id": rbb_assets::RULES_ID,
            "rule_files": rbb_assets::RULE_FILES.len(),
            "modules": modules,
            "rules": rules,
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
                if let Some((id, engine, names)) = engine_for_cards(cards, &mut d) {
                    let side = |seat: Direction, name: &str| {
                        json!({"name": name, "modules": engine.system(seat).modules})
                    };
                    body = json!({
                        "engine": id,
                        "ns": side(Direction::North, &names[0]),
                        "ew": side(Direction::East, &names[1]),
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
        if has_auction { None } else { Some(Direction::North) },
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
    if let Some(cards) = req.get("cards") {
        load_cards(cards, &mut d);
    }
    if let Some(id) = req.get("engine") {
        if id.as_u64().and_then(|i| engine_by_id(i as u32)).is_none() {
            d.error(format!("engine: no engine {id}"));
        }
    }
    respond(json!({}), d)
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
    let (Some((_, engine)), Some(hand), Some(dealer), Some(calls)) = (engine, hand, dealer, calls)
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
    let (Some((_, engine)), Some(dealer), Some(calls)) = (engine, dealer, calls) else {
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
    let (Some((_, engine)), Some(hands), Some(dealer), Some(prefix)) = (engine, hands, dealer, prefix)
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
        let mut j = step_json(&c.step, i, &mut d);
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
                d.warning(format!("call {} ({}): {w}", i + 1, choice.call.to_pbn()));
            }
        }
        calls.push(j);
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

fn active_sets(req: &Json, d: &mut Diags) -> Option<[Vec<String>; 2]> {
    if req.get("engine").is_none() && req.get("cards").is_none() {
        return None;
    }
    let (_, engine) = engine_for(req, d)?;
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
    let Some(modules) = modules_or_report(&mut d) else {
        return respond(json!({"modules": []}), d);
    };
    let active = active_sets(&req, &mut d);
    let mut body = json!({
        "rules_id": rbb_assets::RULES_ID,
        "modules": bidspec::reference::entries(modules),
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
    let Some(modules) = modules_or_report(&mut d) else {
        return d.0.iter().map(|x| format!("# error: {}\n", x.message)).collect();
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
        rbb_assets::RULES_ID
    );
    let entries = bidspec::reference::entries(modules);
    let on = active.as_ref().map(|a| a[side].clone());
    let marker = on.as_ref().map(|list| move |name: &str| list.iter().any(|m| m == name));
    let mut out: String = d
        .0
        .iter()
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
        let r = parse(&bid(
            r#"{"cards": {"ns": "21GF-DEFAULT", "ew": "21GF-GIB"},
                "hand": "AK52.KJ7.Q94.K83", "dealer": "N", "vul": "None",
                "scoring": "IMP", "auction": ""}"#,
        ));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["call"], "1NT", "{r}");
        assert_eq!(r["seat"], "N");
        assert!(r["candidates"].as_array().unwrap().len() > 1);

        // Same answer as the engine on the files, through rbb-assets.
        let modules = rbb_engine::load_modules(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../conventions"
        )))
        .unwrap();
        let card = bbsa::import(rbb_assets::card("21GF-DEFAULT").unwrap(), None)
            .unwrap()
            .0;
        let ew = bbsa::import(rbb_assets::card("21GF-GIB").unwrap(), None)
            .unwrap()
            .0;
        let e = Engine::new(&card, &ew, &modules);
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
        assert!(diags.iter().any(|x| x["message"].as_str().unwrap().starts_with("hand:")));
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

        let v = parse(&validate(r#"{"hand": "AK52.KJ7.Q94.K83", "dealer": "S", "auction": "1NT X XX"}"#));
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

    #[test]
    fn reference_text_matches_the_data() {
        let t = reference(r#"{"cards": {"ns": "21GF-DEFAULT"}}"#);
        assert!(t.starts_with("# rusty-bidding-bot conventions reference"), "{t}");
        assert!(t.contains("== stayman"), "{}", &t[..500]);
        assert!(t.contains("[on]") && t.contains("[off]"));
        let c = parse(&conventions("{}"));
        assert_eq!(c["ok"], true);
        let n = c["modules"].as_array().unwrap().len();
        assert_eq!(t.matches("\n== ").count(), n);
    }
}
