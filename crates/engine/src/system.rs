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
