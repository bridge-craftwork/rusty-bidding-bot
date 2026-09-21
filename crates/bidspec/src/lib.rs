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
mod diag;
mod display;
mod expr;
mod lexer;
mod parser;

pub use ast::Module;
pub use check::check;
pub use diag::Diagnostic;

/// Parse one `.bid` file. `file` is used in diagnostics and recorded in the
/// module.
pub fn parse(source: &str, file: &str) -> Result<Module, Vec<Diagnostic>> {
    parser::parse(source, file)
}

/// Parse and check against the card registry.
pub fn compile(source: &str, file: &str) -> Result<Module, Vec<Diagnostic>> {
    let module = parse(source, file)?;
    let diags = check(&module);
    if diags.is_empty() {
        Ok(module)
    } else {
        Err(diags)
    }
}

/// The module as JSON IR.
pub fn to_json(module: &Module) -> String {
    serde_json::to_string_pretty(module).expect("module serializes")
}
