//! Checks that need more than the syntax: card paths and values must exist
//! in the convention card registry.

use bridge_card::{registry, Value};

use crate::ast::{Literal, Module};
use crate::Diagnostic;

fn to_value(lit: &Literal) -> Value {
    match lit {
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Int(i) => Value::Int(*i),
        Literal::Text(s) => Value::Text(s.clone()),
    }
}

/// Check a parsed module against the card registry.
pub fn check(module: &Module) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut report = |line: usize, message: String| {
        diags.push(Diagnostic {
            file: module.file.clone(),
            line,
            col: 0,
            message,
        })
    };
    let mut check_path =
        |line: usize, path: &str, value: Option<&Literal>| match registry().get(path) {
            None => report(
                line,
                format!("unknown card field `{path}` (see crates/bridge-card/data/fields.toml)"),
            ),
            Some(field) => {
                if field.path != path {
                    report(
                        line,
                        format!("`{path}` is an old name; use `{}`", field.path),
                    );
                }
                if let Some(v) = value {
                    if let Err(e) = field.normalize(to_value(v)) {
                        report(line, format!("`{path}`: {e}"));
                    }
                }
            }
        };
    for cond in &module.card {
        check_path(cond.line, &cond.path, cond.value.as_ref());
    }
    for param in &module.params {
        check_path(param.line, &param.path, param.default.as_ref());
    }
    diags
}
