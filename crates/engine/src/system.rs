//! Which modules and rules a partnership plays, from its convention card.

use std::collections::{HashMap, HashSet};

use bidspec::ast::{Context, Expr, Literal, Module, PatternCall, Rule};
use bridge_card::{Card, Value};
use serde::Serialize;

use crate::eval::Val;
use crate::knowledge::Range;
use crate::macros::{qualified, Defines};

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
    /// One entry per enclosing `after`; each holds that line's
    /// alternatives, of which any one matching is enough.
    pub patterns: Vec<Vec<Vec<PatternCall>>>,
    pub conditions: Vec<Expr>,
    pub module: usize,
    pub source: RuleRef,
    /// Position in load order, the last tie-breaker.
    pub order: usize,
    /// Its `shows` counts a condition in arithmetic (`+ doubler_four(x)`).
    pub counts_conditions: bool,
}

/// The active rules for one partnership.
#[derive(Debug, Clone, Default)]
pub struct System {
    pub modules: Vec<String>,
    pub params: Vec<crate::eval::Params>,
    pub rules: Vec<RuleEntry>,
    /// The active modules' `force` declarations.
    pub forces: Vec<ForceEntry>,
    /// Modules not active, and why.
    pub inactive: Vec<(String, String)>,
}

/// A `force game` declaration of an active module, its condition
/// expanded (`macros`).
#[derive(Debug, Clone)]
pub struct ForceEntry {
    pub after: Option<Vec<Vec<PatternCall>>>,
    pub when: Option<Expr>,
    pub module: usize,
    pub source: RuleRef,
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
        Value::Text(s) => Val::Sym(s.as_str().into()),
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
        let param_value = |p: &bidspec::ast::Param| {
            let v = card
                .effective(&p.path)
                .map(to_val)
                .or_else(|| p.default.as_ref().map(|d| to_val(&literal_value(d))));
            // A param the card leaves unset, with no default, is Nothing:
            // `style is bba` is then simply false.
            v.unwrap_or(Val::Nothing)
        };
        // Definitions are inlined into the rules that use them, reading
        // their own module's parameters under qualified names, active or
        // not (see `macros`).
        let defines = Defines::new(modules);
        let mut defined = crate::eval::Params::default();
        for m in modules.iter().filter(|m| !m.defines.is_empty()) {
            for p in &m.params {
                defined.insert(qualified(&m.name, &p.name), param_value(p));
            }
        }
        for m in modules.iter().filter(|m| active.contains(m.name.as_str())) {
            let idx = sys.modules.len();
            sys.modules.push(m.name.clone());
            let mut params = defined.clone();
            for p in &m.params {
                params.insert(p.name.clone(), param_value(p));
            }
            sys.params.push(params);
            for f in &m.forces {
                sys.forces.push(ForceEntry {
                    after: f.after.clone(),
                    when: f.when.as_ref().map(|w| defines.expand(w, &mut Vec::new())),
                    module: idx,
                    source: RuleRef {
                        module: m.name.clone(),
                        file: m.file.clone(),
                        line: f.line,
                    },
                });
            }
            for ctx in &m.contexts {
                sys.flatten(ctx, &mut Vec::new(), &mut Vec::new(), idx, m, &defines);
            }
        }
        sys
    }

    fn flatten(
        &mut self,
        ctx: &Context,
        patterns: &mut Vec<Vec<Vec<PatternCall>>>,
        conditions: &mut Vec<Expr>,
        module: usize,
        m: &Module,
        defines: &Defines,
    ) {
        // Problems expanding are reported when the rules are loaded
        // (`check_terms`); here the term is left as written.
        let expand = |e: &Expr| defines.expand(e, &mut Vec::new());
        let (np, nc) = (patterns.len(), conditions.len());
        if let Some(p) = &ctx.after {
            patterns.push(p.clone());
        }
        if let Some(w) = &ctx.when {
            conditions.push(expand(w));
        }
        for rule in &ctx.rules {
            let mut rule = rule.clone();
            for e in [
                &mut rule.shows,
                &mut rule.when,
                &mut rule.denies,
                &mut rule.prefer,
            ]
            .into_iter()
            .flatten()
            {
                *e = expand(e);
            }
            let line = rule.line;
            let counts_conditions = rule
                .shows
                .as_ref()
                .is_some_and(crate::eval::counts_conditions);
            self.rules.push(RuleEntry {
                rule,
                patterns: patterns.clone(),
                conditions: conditions.clone(),
                module,
                source: RuleRef {
                    module: m.name.clone(),
                    file: m.file.clone(),
                    line,
                },
                order: self.rules.len(),
                counts_conditions,
            });
        }
        for child in &ctx.contexts {
            self.flatten(child, patterns, conditions, module, m, defines);
        }
        patterns.truncate(np);
        conditions.truncate(nc);
    }
}

/// Check what modules say about the card against the card fields in
/// `registry` (the rule set's vocabulary): every
/// `card` and `param` path is a field, values fit it, and an enum parameter
/// is only compared with its options (`style is relay`). A misspelled option
/// would otherwise never match, silently.
pub fn check_card_refs(modules: &[Module], registry: &bridge_card::Registry) -> Vec<String> {
    let reg = registry;
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
        for d in &m.defines {
            walk(&d.body, d.line, &options, &mut found);
        }
        errors.extend(
            found
                .into_iter()
                .map(|(line, e)| format!("{}: {e}", at(line))),
        );
    }
    errors
}
