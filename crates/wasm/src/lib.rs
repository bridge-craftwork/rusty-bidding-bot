//! Browser (WASM) boundary for rusty-bidding-bot.
//!
//! Each export takes a JSON request string and returns a JSON response
//! string (`reference` returns plain text). The logic is in `api`, plain
//! Rust tested natively; this file only binds it. The contract, with every
//! request and response shape, is docs/WASM.md.

pub mod api;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// `{ok, api, version, rules_id, rule_files, modules, rules, stock_cards, diagnostics}`
#[wasm_bindgen]
pub fn info() -> String {
    api::info()
}

/// `{"cards": {...}}` → `{ok, engine, ns, ew, diagnostics}`
#[wasm_bindgen(js_name = createEngine)]
pub fn create_engine(request: &str) -> String {
    api::create_engine(request)
}

/// `{"engine": id}` → `{ok, diagnostics}`
#[wasm_bindgen(js_name = freeEngine)]
pub fn free_engine(request: &str) -> String {
    api::free_engine(request)
}

/// Check a request's fields without bidding → `{ok, diagnostics}`
#[wasm_bindgen]
pub fn validate(request: &str) -> String {
    api::validate(request)
}

/// The engine's call for one hand → `{ok, seat, call, explanation, alert, rule, candidates, auction, diagnostics}`
#[wasm_bindgen]
pub fn bid(request: &str) -> String {
    api::bid(request)
}

/// What each call of an auction showed → `{ok, steps, complete, next, contract, declarer, diagnostics}`
#[wasm_bindgen]
pub fn interpret(request: &str) -> String {
    api::interpret(request)
}

/// Bid all four hands → `{ok, calls, complete, contract, declarer, diagnostics}`
#[wasm_bindgen(js_name = bidDeal)]
pub fn bid_deal(request: &str) -> String {
    api::bid_deal(request)
}

/// The embedded conventions as data → `{ok, rules_id, modules, active?, diagnostics}`
#[wasm_bindgen]
pub fn conventions(request: &str) -> String {
    api::conventions(request)
}

/// The embedded conventions as plain text (reference.txt).
#[wasm_bindgen]
pub fn reference(request: &str) -> String {
    api::reference(request)
}

/// How much of each side's card the rules read → `{ok, ns, ew, diagnostics}`
#[wasm_bindgen]
pub fn coverage(request: &str) -> String {
    api::coverage(request)
}

/// A card as Bridge-Classroom JSON and `.bbsa` → `{ok, name, json, bbsa, unmapped, diagnostics}`
#[wasm_bindgen(js_name = exportCard)]
pub fn export_card(request: &str) -> String {
    api::export_card(request)
}

/// Double-dummy table, par and the contract's result → `{ok, tricks, par, result, diagnostics}`
#[wasm_bindgen(js_name = ddTable)]
pub fn dd_table(request: &str) -> String {
    api::dd_table(request)
}
