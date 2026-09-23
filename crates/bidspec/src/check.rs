//! Checks that need more than the syntax: card paths and values must exist
//! in the convention card registry.

use bridge_card::{registry, Value};

use crate::ast::{Context, Literal, Module};
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
    for ctx in &module.contexts {
        empty_contexts(ctx, &mut diags, &module.file);
    }
    diags
}

/// A context with nothing under it does nothing, and the usual cause is a
/// `when` written on its own line under an `after`: structure follows
/// indentation, so the rules below it at a shallower indent belong to the
/// `after` and the condition is silently dropped.
fn empty_contexts(ctx: &Context, diags: &mut Vec<Diagnostic>, file: &str) {
    if ctx.rules.is_empty() && ctx.contexts.is_empty() {
        diags.push(Diagnostic {
            file: file.to_string(),
            line: ctx.line,
            col: 0,
            message: "this context has no rules under it, so it does nothing. \
                      A `when` on its own line needs the rules it governs \
                      indented under it."
                .into(),
        });
    }
    for child in &ctx.contexts {
        empty_contexts(child, diags, file);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_context_with_no_rules_is_an_error() {
        // `when` on its own line: the rules below it, at a shallower
        // indent, belong to the `after` and the condition is dropped.
        let src = concat!(
            "module t \"t\"\n\nafter 1N (P)\n",
            "    when hcp>=40\n",
            "  2C \"fires anyway\" shows hcp>=0\n"
        );
        let module = crate::parse(src, "t.bid").expect("parses");
        let diags = crate::check(&module);
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert!(diags[0].message.contains("no rules under it"));
        // Indented under it, the same condition governs the rule.
        let ok = concat!(
            "module t \"t\"\n\nafter 1N (P)\n",
            "  when hcp>=40\n",
            "    2C \"needs 40 HCP\" shows hcp>=0\n"
        );
        let module = crate::parse(ok, "t.bid").expect("parses");
        assert!(crate::check(&module).is_empty());
    }
}
