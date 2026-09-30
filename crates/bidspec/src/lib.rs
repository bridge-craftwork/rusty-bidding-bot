//! Parser for `.bid` rule files, compiling to a JSON IR.
//!
//! ```
//! let src = "module demo \"Demo\"\n\nafter 1N (P)\n  2C \"Stayman\" shows hcp>=8\n";
//! let module = bidspec::parse(src, "demo.bid").unwrap();
//! assert_eq!(module.contexts[0].rules.len(), 1);
//! ```
//!
//! See docs/LANGUAGE.md for the language.

pub mod ast;
mod check;
pub mod coverage;
mod diag;
mod display;
mod expr;
mod lexer;
pub mod manifest;
mod parser;
pub mod reference;
pub mod skills;

pub use ast::Module;
pub use check::check;
pub use diag::Diagnostic;

/// Parse one `.bid` file. `file` is used in diagnostics and recorded in the
/// module.
pub fn parse(source: &str, file: &str) -> Result<Module, Vec<Diagnostic>> {
    parser::parse(source, file)
}

/// Parse and check against the card fields in `registry` (the rule set's
/// vocabulary, [`bridge_card::Vocabulary::registry`]).
pub fn compile(
    source: &str,
    file: &str,
    registry: &bridge_card::Registry,
) -> Result<Module, Vec<Diagnostic>> {
    let module = parse(source, file)?;
    let diags = check(&module, registry);
    if diags.is_empty() {
        Ok(module)
    } else {
        Err(diags)
    }
}

/// The standard card vocabulary (convention-card's `spec/`), for the unit
/// tests.
#[cfg(test)]
pub(crate) fn test_vocabulary() -> &'static bridge_card::Vocabulary {
    static V: std::sync::OnceLock<bridge_card::Vocabulary> = std::sync::OnceLock::new();
    V.get_or_init(|| bridge_card::standard::vocabulary().expect("the standard vocabulary loads"))
}

/// The module as JSON IR.
pub fn to_json(module: &Module) -> String {
    serde_json::to_string_pretty(module).expect("module serializes")
}
