//! Which modules and rules a partnership plays, from its convention card.

use std::collections::{HashMap, HashSet};

use bidspec::ast::{Context, Expr, Literal, Module, PatternCall, Rule};
use bridge_card::{Card, Value};
use serde::Serialize;

use crate::eval::Val;
use crate::knowledge::Range;

/// Where a rule came from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct RuleRef {
    pub module: String,
    pub file: String,
    pub line: usize,
}

/// A rule with the chain of contexts it sits under.
#[derive(Debug, Clone)]
pub struct RuleEntry {
    pub rule: Rule,
    pub patterns: Vec<Vec<PatternCall>>,
    pub conditions: Vec<Expr>,
    pub module: usize,
    pub source: RuleRef,
    /// Position in load order, the last tie-breaker.
    pub order: usize,
}

/// The active rules for one partnership.
#[derive(Debug, Clone, Default)]
pub struct System {
    pub modules: Vec<String>,
    pub params: Vec<HashMap<String, Val>>,
    pub rules: Vec<RuleEntry>,
    /// Modules not active, and why.
    pub inactive: Vec<(String, String)>,
}

fn literal_value(l: &Literal) -> Value {
    match l {
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Int(i) => Value::Int(*i),
        Literal::Text(s) => Value::Text(s.clone()),
    }
}

fn to_val(v: &Value) -> Val {
    match v {
        Value::Bool(b) => Val::Bool(crate::knowledge::Tri::from_bool(*b)),
        Value::Int(i) => Val::Num(Range::point(*i as i32)),
        Value::Text(s) => Val::Sym(s.clone()),
    }
}

impl System {
    /// Activate `modules` for `card`. A module is active when every `card`
    /// condition holds and every module it `needs` is active. Modules are
    /// taken in the order given (callers sort by file for stable ranking).
    pub fn new(card: &Card, modules: &[Module]) -> System {
        let mut sys = System::default();
        let mut active: HashSet<&str> = HashSet::new();
        for m in modules {
            let failed = m.card.iter().find(|c| match &c.value {
                None => !card.is_on(&c.path),
                Some(v) => card.effective(&c.path) != Some(&literal_value(v)),
            });
            match failed {
                Some(c) => sys
                    .inactive
                    .push((m.name.clone(), format!("card: {} is not set", c.path))),
                None => {
                    active.insert(&m.name);
                }
            }
        }
        // Drop modules whose needs are inactive, until nothing changes.
        loop {
            let before = active.len();
            for m in modules {
                if active.contains(m.name.as_str()) {
                    if let Some(n) = m.needs.iter().find(|n| !active.contains(n.as_str())) {
                        active.remove(m.name.as_str());
                        sys.inactive
                            .push((m.name.clone(), format!("needs {n}, which is not active")));
                    }
                }
            }
            if active.len() == before {
                break;
            }
        }
        for m in modules.iter().filter(|m| active.contains(m.name.as_str())) {
            let idx = sys.modules.len();
            sys.modules.push(m.name.clone());
            let mut params = HashMap::new();
            for p in &m.params {
                let v = card
                    .effective(&p.path)
                    .map(to_val)
                    .or_else(|| p.default.as_ref().map(|d| to_val(&literal_value(d))));
                if let Some(v) = v {
                    params.insert(p.name.clone(), v);
                }
            }
            sys.params.push(params);
            for ctx in &m.contexts {
                sys.flatten(ctx, &mut Vec::new(), &mut Vec::new(), idx, m);
            }
        }
        sys
    }

    fn flatten(
        &mut self,
        ctx: &Context,
        patterns: &mut Vec<Vec<PatternCall>>,
        conditions: &mut Vec<Expr>,
        module: usize,
        m: &Module,
    ) {
        let (np, nc) = (patterns.len(), conditions.len());
        if let Some(p) = &ctx.after {
            patterns.push(p.clone());
        }
        if let Some(w) = &ctx.when {
            conditions.push(w.clone());
        }
        for rule in &ctx.rules {
            self.rules.push(RuleEntry {
                rule: rule.clone(),
                patterns: patterns.clone(),
                conditions: conditions.clone(),
                module,
                source: RuleRef {
                    module: m.name.clone(),
                    file: m.file.clone(),
                    line: rule.line,
                },
                order: self.rules.len(),
            });
        }
        for child in &ctx.contexts {
            self.flatten(child, patterns, conditions, module, m);
        }
        patterns.truncate(np);
        conditions.truncate(nc);
    }
}

/// Check what modules say about the card against the card registry: every
/// `card` and `param` path is a field, values fit it, and an enum parameter
/// is only compared with its options (`style is relay`). A misspelled option
/// would otherwise never match, silently.
pub fn check_card_refs(modules: &[Module]) -> Vec<String> {
    let reg = bridge_card::registry();
    let mut errors = Vec::new();
    for m in modules {
        let at = |line: usize| format!("{}:{line}", m.file);
        for c in &m.card {
            match reg.get(&c.path) {
                None => errors.push(format!("{}: card: no field {}", at(c.line), c.path)),
                Some(f) => {
                    if let Some(v) = &c.value {
                        if let Err(e) = f.normalize(literal_value(v)) {
                            errors.push(format!("{}: card {}: {e}", at(c.line), c.path));
                        }
                    }
                }
            }
        }
        let mut options: HashMap<&str, &[String]> = HashMap::new();
        for p in &m.params {
            match reg.get(&p.path) {
                None => errors.push(format!(
                    "{}: param {}: no field {}",
                    at(p.line),
                    p.name,
                    p.path
                )),
                Some(f) => {
                    if let Some(d) = &p.default {
                        if let Err(e) = f.normalize(literal_value(d)) {
                            errors.push(format!("{}: param {} default: {e}", at(p.line), p.name));
                        }
                    }
                    if !f.options.is_empty() {
                        options.insert(p.name.as_str(), &f.options);
                    }
                }
            }
        }
        if options.is_empty() {
            continue;
        }
        fn walk(
            e: &Expr,
            line: usize,
            options: &HashMap<&str, &[String]>,
            out: &mut Vec<(usize, String)>,
        ) {
            match e {
                Expr::Is { expr, what, .. } => {
                    if let Expr::Path { path } = expr.as_ref() {
                        if let [seg] = path.as_slice() {
                            if let Some(opts) = options.get(seg.name.as_str()) {
                                if !opts.iter().any(|o| o == what) {
                                    out.push((
                                        line,
                                        format!(
                                            "`{} is {what}`: options are {}",
                                            seg.name,
                                            opts.join(", ")
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
                Expr::And { all } => all.iter().for_each(|x| walk(x, line, options, out)),
                Expr::Or { any } => any.iter().for_each(|x| walk(x, line, options, out)),
                Expr::Not { expr } | Expr::Maybe { expr } | Expr::Neg { expr } => {
                    walk(expr, line, options, out)
                }
                Expr::Cmp { lhs, rhs, .. } | Expr::Arith { lhs, rhs, .. } => {
                    walk(lhs, line, options, out);
                    walk(rhs, line, options, out);
                }
                _ => {}
            }
        }
        fn ctx(c: &Context, options: &HashMap<&str, &[String]>, out: &mut Vec<(usize, String)>) {
            if let Some(w) = &c.when {
                walk(w, c.line, options, out);
            }
            for r in &c.rules {
                for e in [&r.shows, &r.when, &r.denies, &r.prefer]
                    .into_iter()
                    .flatten()
                {
                    walk(e, r.line, options, out);
                }
            }
            c.contexts.iter().for_each(|x| ctx(x, options, out));
        }
        let mut found = Vec::new();
        m.contexts.iter().for_each(|c| ctx(c, &options, &mut found));
        errors.extend(
            found
                .into_iter()
                .map(|(line, e)| format!("{}: {e}", at(line))),
        );
    }
    errors
}
