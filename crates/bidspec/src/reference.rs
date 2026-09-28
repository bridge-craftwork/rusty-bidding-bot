//! A reference to compiled modules: which conventions exist, when a card
//! switches them on, and what each call means after each auction. One
//! structure (`entries`), two renderings: JSON for a UI (it serializes) and
//! plain text (`text`), so both always say the same thing.

use serde::Serialize;

use crate::ast::{Alert, Context, Expr, Module, PatternCall};

/// One module, flattened for reading.
#[derive(Debug, Clone, Serialize)]
pub struct ModuleRef {
    pub name: String,
    pub title: Option<String>,
    pub file: String,
    /// Card conditions that switch it on, as written (`notrump.stayman.play`,
    /// `notrump.range = 15-17`).
    pub card: Vec<String>,
    pub needs: Vec<String>,
    pub rules: Vec<RuleRef>,
}

/// One rule: the auctions it applies after, and what its call means.
#[derive(Debug, Clone, Serialize)]
pub struct RuleRef {
    /// Alternative auctions before the call, each as calls separated by
    /// spaces, the opponents' in parentheses (`1N (P)`). Empty for an
    /// opening-position rule written outside any `after`.
    pub after: Vec<String>,
    /// Conditions from the enclosing contexts' `when`s.
    pub context: Vec<String>,
    /// The call as written (`2C`, `2x`, `jump(x)`).
    pub call: String,
    pub explanation: String,
    /// `alert` or `announce`, with its text if any.
    pub alert: Option<String>,
    pub shows: Option<String>,
    pub when: Option<String>,
    pub artificial: bool,
    pub line: usize,
}

fn pattern(calls: &[PatternCall]) -> String {
    calls
        .iter()
        .map(|p| {
            if p.theirs {
                format!("({})", p.call)
            } else {
                p.call.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn alert_text(a: &Alert) -> String {
    match a {
        Alert::Alert { text: Some(t) } => format!("alert: {t}"),
        Alert::Alert { text: None } => "alert".into(),
        Alert::Announce { text } => format!("announce: {text}"),
    }
}

fn walk(c: &Context, after: &[String], when: &[String], out: &mut Vec<RuleRef>) {
    // A nested `after` continues the enclosing one.
    let after: Vec<String> = match &c.after {
        None => after.to_vec(),
        Some(alts) => {
            let mine: Vec<String> = alts.iter().map(|a| pattern(a)).collect();
            if after.is_empty() {
                mine
            } else {
                after
                    .iter()
                    .flat_map(|a| mine.iter().map(move |m| format!("{a} {m}")))
                    .collect()
            }
        }
    };
    let mut when = when.to_vec();
    if let Some(w) = &c.when {
        when.push(w.to_string());
    }
    for r in &c.rules {
        out.push(RuleRef {
            after: after.clone(),
            context: when.clone(),
            call: r.call.to_string(),
            explanation: r.explanation.clone(),
            alert: r.alert.as_ref().map(alert_text),
            shows: r.shows.as_ref().map(Expr::to_string),
            when: r.when.as_ref().map(Expr::to_string),
            artificial: r.artificial,
            line: r.line,
        });
    }
    for sub in &c.contexts {
        walk(sub, &after, &when, out);
    }
}

/// Every module, flattened: one entry per rule.
pub fn entries(modules: &[Module]) -> Vec<ModuleRef> {
    modules
        .iter()
        .map(|m| {
            let mut rules = Vec::new();
            for c in &m.contexts {
                walk(c, &[], &[], &mut rules);
            }
            ModuleRef {
                name: m.name.clone(),
                title: m.title.clone(),
                file: m.file.clone(),
                card: m
                    .card
                    .iter()
                    .map(|c| match &c.value {
                        None => c.path.clone(),
                        Some(v) => format!("{} = {}", c.path, literal(v)),
                    })
                    .collect(),
                needs: m.needs.clone(),
                rules,
            }
        })
        .collect()
}

fn literal(l: &crate::ast::Literal) -> String {
    match l {
        crate::ast::Literal::Bool(b) => b.to_string(),
        crate::ast::Literal::Int(i) => i.to_string(),
        crate::ast::Literal::Text(s) => s.clone(),
    }
}

/// The reference as plain text: a header, then per module its card
/// conditions and, grouped by auction, each call and what it means.
/// `active`, when given, marks each module on or off for a card.
pub fn text(modules: &[ModuleRef], header: &str, active: Option<&dyn Fn(&str) -> bool>) -> String {
    let mut out = String::new();
    for line in header.lines() {
        out.push_str("# ");
        out.push_str(line);
        out.push('\n');
    }
    let rules: usize = modules.iter().map(|m| m.rules.len()).sum();
    out.push_str(&format!("# {} modules, {rules} rules\n", modules.len()));
    out.push_str("#\n# Calls: P pass, X double, XX redouble; (..) the opponents' calls.\n");
    out.push_str("# x/y any suit, M a major, m a minor. Text in {..} is filled in when bidding.\n");
    for m in modules {
        out.push('\n');
        out.push_str(&format!(
            "== {}{}",
            m.name,
            m.title
                .as_deref()
                .map(|t| format!(": {t}"))
                .unwrap_or_default()
        ));
        if let Some(on) = active {
            out.push_str(if on(&m.name) { "  [on]" } else { "  [off]" });
        }
        out.push('\n');
        out.push_str(&format!("   file: {}\n", m.file));
        if !m.card.is_empty() {
            out.push_str(&format!("   card: {}\n", m.card.join(", ")));
        }
        if !m.needs.is_empty() {
            out.push_str(&format!("   needs: {}\n", m.needs.join(", ")));
        }
        let mut last: Option<(&Vec<String>, &Vec<String>)> = None;
        for r in &m.rules {
            if last != Some((&r.after, &r.context)) {
                let after = if r.after.is_empty() {
                    "(any auction)".to_string()
                } else {
                    r.after.join(" | ")
                };
                out.push_str(&format!("\n   after {after}\n"));
                for w in &r.context {
                    out.push_str(&format!("     when {w}\n"));
                }
                last = Some((&r.after, &r.context));
            }
            let mut line = format!("     {:6} {}", r.call, r.explanation);
            if let Some(a) = &r.alert {
                line.push_str(&format!("  [{a}]"));
            }
            if r.artificial {
                line.push_str("  [artificial]");
            }
            out.push_str(line.trim_end());
            out.push('\n');
            if let Some(s) = &r.shows {
                out.push_str(&format!("            shows {s}\n"));
            }
            if let Some(w) = &r.when {
                out.push_str(&format!("            when  {w}\n"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_are_listed_under_their_auction() {
        let src = "module demo \"Demo\"\n  card notrump.stayman.play\n\nafter 1N (P)\n  2C \"Stayman\" alert\n      shows hcp>=8\n";
        let m = crate::parse(src, "demo.bid").unwrap();
        let e = entries(&[m]);
        assert_eq!(e[0].rules[0].after, vec!["1N (P)".to_string()]);
        assert_eq!(e[0].rules[0].alert.as_deref(), Some("alert"));
        let t = text(&e, "test", None);
        assert!(t.contains("after 1N (P)"), "{t}");
        assert!(t.contains("2C     Stayman  [alert]"), "{t}");
        assert!(t.contains("shows hcp>=8"), "{t}");
    }
}
