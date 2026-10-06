//! Named conditions (`define`) and the partnership sums (`we.fit(x)`,
//! `we.points`, `we.hcp`, `we.tp(x)`, `safe_level(x)`), expanded into the
//! terms they stand for before a rule is used.
//!
//! Both are written for the reader. `we.fit(S).min >= 8` is
//! `S + partner.S.min >= 8`, and it has to *be* that: a `shows` must
//! resolve to a condition on the caller's own hand (my length plus
//! partner's known minimum), or partner reads nothing from the call. So the
//! engine rewrites the short form into the long one where the rules are
//! flattened (`System::new`), and everything after that sees only the long
//! form. The modules themselves, and the reference printed from them, keep
//! what was written.

use std::collections::{HashMap, HashSet};

use bidspec::ast::{ArithOp, AuctionAlt, CallSpec, Expr, Module, PatternCall, Segment, StrainSpec};

/// A definition, its body's card parameters already qualified with the
/// defining module's name (`control-bids::style`).
#[derive(Debug, Clone)]
struct Def {
    params: Vec<String>,
    body: Expr,
}

/// Every `define` of a rule set, by name.
#[derive(Debug, Clone, Default)]
pub struct Defines {
    defs: HashMap<String, Def>,
}

/// A card parameter of `module` as a define's body names it once inlined
/// elsewhere: the using module has its own parameters, maybe of the same
/// name (`style` is `general.style` in one module and
/// `slam.cue_bids.style` in another).
pub fn qualified(module: &str, param: &str) -> String {
    format!("{module}::{param}")
}

/// Expansions nest at most this deep: deeper is a definition that uses
/// itself.
const DEPTH: usize = 32;

impl Defines {
    /// The definitions of `modules` (all of them, active or not: a
    /// definition is a condition, not a rule). A name defined twice keeps
    /// the first; `check` reports the second.
    pub fn new(modules: &[Module]) -> Defines {
        let mut defs = HashMap::new();
        for m in modules {
            let params: HashSet<&str> = m.params.iter().map(|p| p.name.as_str()).collect();
            for d in &m.defines {
                let body = qualify(&d.body, &params, &d.params, &m.name);
                defs.entry(d.name.clone()).or_insert(Def {
                    params: d.params.clone(),
                    body,
                });
            }
        }
        Defines { defs }
    }

    /// `e` with every definition inlined and every partnership sum written
    /// out. Problems (a wrong number of arguments, an argument that is not
    /// a suit name, a definition that uses itself) go to `errors`, and the
    /// term is left as written.
    pub fn expand(&self, e: &Expr, errors: &mut Vec<String>) -> Expr {
        self.walk(e, errors, 0)
    }

    fn walk(&self, e: &Expr, errors: &mut Vec<String>, depth: usize) -> Expr {
        let rec = |x: &Expr, errors: &mut Vec<String>| Box::new(self.walk(x, errors, depth));
        match e {
            Expr::And { all } => Expr::And {
                all: all.iter().map(|x| *rec(x, errors)).collect(),
            },
            Expr::Or { any } => Expr::Or {
                any: any.iter().map(|x| *rec(x, errors)).collect(),
            },
            Expr::Not { expr } => Expr::Not {
                expr: rec(expr, errors),
            },
            Expr::Maybe { expr } => Expr::Maybe {
                expr: rec(expr, errors),
            },
            Expr::Neg { expr } => Expr::Neg {
                expr: rec(expr, errors),
            },
            Expr::Cmp { cmp, lhs, rhs } => Expr::Cmp {
                cmp: *cmp,
                lhs: rec(lhs, errors),
                rhs: rec(rhs, errors),
            },
            Expr::Arith { arith, lhs, rhs } => Expr::Arith {
                arith: *arith,
                lhs: rec(lhs, errors),
                rhs: rec(rhs, errors),
            },
            Expr::InRange { expr, lo, hi } => Expr::InRange {
                expr: rec(expr, errors),
                lo: rec(lo, errors),
                hi: rec(hi, errors),
            },
            Expr::InSet { expr, values } => Expr::InSet {
                expr: rec(expr, errors),
                values: values.clone(),
            },
            Expr::Is { expr, what, not } => Expr::Is {
                expr: rec(expr, errors),
                what: what.clone(),
                not: *not,
            },
            Expr::Asked { kind } => Expr::Asked {
                kind: kind.as_ref().map(|k| rec(k, errors)),
            },
            Expr::Answered { kind } => Expr::Answered {
                kind: kind.as_ref().map(|k| rec(k, errors)),
            },
            Expr::Path { path } => self.path(path, errors, depth),
            Expr::Shape { .. } | Expr::Int { .. } | Expr::Call { .. } => e.clone(),
        }
    }

    fn path(&self, path: &[Segment], errors: &mut Vec<String>, depth: usize) -> Expr {
        // Arguments first: `tp(x)` inside a definition's argument list.
        let path: Vec<Segment> = path
            .iter()
            .map(|s| Segment {
                name: s.name.clone(),
                args: s
                    .args
                    .as_ref()
                    .map(|a| a.iter().map(|x| self.walk(x, errors, depth)).collect()),
            })
            .collect();
        if let [seg] = path.as_slice() {
            if let Some(def) = self.defs.get(&seg.name) {
                if depth >= DEPTH {
                    errors.push(format!("`{}` is defined in terms of itself", seg.name));
                    return Expr::Path { path };
                }
                let args = seg.args.as_deref().unwrap_or(&[]);
                if args.len() != def.params.len() {
                    errors.push(format!(
                        "`{}` takes {} argument{} ({})",
                        seg.name,
                        def.params.len(),
                        if def.params.len() == 1 { "" } else { "s" },
                        def.params.join(", ")
                    ));
                    return Expr::Path { path };
                }
                let mut map = HashMap::new();
                for (p, a) in def.params.iter().zip(args) {
                    match simple_name(a) {
                        Some(n) => {
                            map.insert(p.clone(), n.to_string());
                        }
                        None => {
                            errors.push(format!(
                                "`{}`: the argument `{a}` is not a suit, a suit variable or `trump`",
                                seg.name
                            ));
                            return Expr::Path { path };
                        }
                    }
                }
                return self.walk(&subst(&def.body, &map), errors, depth + 1);
            }
        }
        match sugar(&path) {
            Some(Ok(e)) => e,
            Some(Err(msg)) => {
                errors.push(msg);
                Expr::Path { path }
            }
            None => Expr::Path { path },
        }
    }
}

/// One alternative of an `after` line as the engine matches it: the
/// calls, with any named pattern (`define auction`) written out, and the
/// condition its definition's line adds (`when kokish`), which must hold
/// too. Bindings work as for any alternative: the first that matches
/// supplies them, and the condition sees them.
#[derive(Debug, Clone, PartialEq)]
pub struct Alt {
    pub calls: Vec<PatternCall>,
    pub when: Option<Expr>,
}

/// Every `define auction` of a rule set, by name, each line's condition
/// reading its own module's card parameters (`qualified`).
#[derive(Debug, Clone, Default)]
pub struct Auctions {
    defs: HashMap<String, Vec<AuctionAlt>>,
}

impl Auctions {
    /// The named patterns of `modules`, active or not. A name defined
    /// twice keeps the first; `check_terms` reports the second.
    pub fn new(modules: &[Module]) -> Auctions {
        let mut defs = HashMap::new();
        for m in modules {
            let params: HashSet<&str> = m.params.iter().map(|p| p.name.as_str()).collect();
            for a in &m.auctions {
                let alts = a
                    .alts
                    .iter()
                    .map(|alt| AuctionAlt {
                        pattern: alt.pattern.clone(),
                        when: alt.when.as_ref().map(|w| qualify(w, &params, &[], &m.name)),
                        line: alt.line,
                    })
                    .collect();
                defs.entry(a.name.clone()).or_insert(alts);
            }
        }
        Auctions { defs }
    }

    /// One `after` line's alternatives with every named pattern written
    /// out: `nt2 3C (P)` becomes one alternative per alternative of
    /// `nt2`, each followed by `3C (P)`, in the definition's order.
    /// Conditions are expanded (`Defines`). An unknown name or a pattern
    /// defined in terms of itself goes to `errors` and contributes no
    /// alternative.
    pub fn resolve(
        &self,
        alts: &[Vec<PatternCall>],
        defines: &Defines,
        errors: &mut Vec<String>,
    ) -> Vec<Alt> {
        let mut out = Vec::new();
        for a in alts {
            self.one(a, defines, errors, &mut Vec::new(), &mut out);
        }
        out
    }

    /// The alternatives of the definition of `name` written out, for the
    /// checks: a use of `name` among them is a cycle.
    pub fn resolve_def(
        &self,
        name: &str,
        alts: &[Vec<PatternCall>],
        defines: &Defines,
        errors: &mut Vec<String>,
    ) -> Vec<Alt> {
        let mut out = Vec::new();
        for a in alts {
            self.one(a, defines, errors, &mut vec![name.to_string()], &mut out);
        }
        out
    }

    fn one(
        &self,
        a: &[PatternCall],
        defines: &Defines,
        errors: &mut Vec<String>,
        stack: &mut Vec<String>,
        out: &mut Vec<Alt>,
    ) {
        let Some((
            PatternCall {
                call: CallSpec::Named { name },
                ..
            },
            suffix,
        )) = a.split_first()
        else {
            out.push(Alt {
                calls: a.to_vec(),
                when: None,
            });
            return;
        };
        if stack.contains(name) {
            let mut chain = stack.clone();
            chain.push(name.clone());
            errors.push(format!(
                "the auction pattern `{name}` is defined in terms of itself ({})",
                chain.join(" -> ")
            ));
            return;
        }
        let Some(def) = self.defs.get(name) else {
            errors.push(format!(
                "no auction pattern `{name}` (`define auction {name} = ...`)"
            ));
            return;
        };
        stack.push(name.clone());
        for d in def {
            let mut inner = Vec::new();
            self.one(&d.pattern, defines, errors, stack, &mut inner);
            let when = d.when.as_ref().map(|w| defines.expand(w, errors));
            for mut alt in inner {
                alt.calls.extend_from_slice(suffix);
                alt.when = match (alt.when.take(), when.clone()) {
                    (None, w) | (w, None) => w,
                    (Some(a), Some(b)) => Some(Expr::And { all: vec![a, b] }),
                };
                out.push(alt);
            }
        }
        stack.pop();
    }
}

/// A name standing alone (`S`, `x`, `trump`): what a definition's argument
/// and `we.fit`'s must be.
fn simple_name(e: &Expr) -> Option<&str> {
    match e {
        Expr::Path { path } if path.len() == 1 && path[0].args.is_none() => {
            Some(path[0].name.as_str())
        }
        _ => None,
    }
}

fn name_path(names: &[&str]) -> Expr {
    Expr::Path {
        path: names
            .iter()
            .map(|n| Segment {
                name: n.to_string(),
                args: None,
            })
            .collect(),
    }
}

/// The partnership sums, written out: mine plus partner's, with `.min` /
/// `.max` applying to partner's range (mine is exact while I choose).
/// `None` for any other path.
pub fn sugar(path: &[Segment]) -> Option<Result<Expr, String>> {
    let add = |a: Expr, b: Expr| Expr::Arith {
        arith: ArithOp::Add,
        lhs: Box::new(a),
        rhs: Box::new(b),
    };
    let suffix: Vec<&str> = match path.first().map(|s| s.name.as_str()) {
        Some("we") if path.len() >= 2 => path[2..].iter().map(|s| s.name.as_str()).collect(),
        _ => Vec::new(),
    };
    // `safe_level(x)`: the level our trumps make safe, eight trumps the two
    // level (the Law of Total Tricks): `we.fit(x).min - 6`.
    if let [seg] = path {
        if seg.name == "safe_level" {
            let Some([x]) = seg.args.as_deref() else {
                return Some(Err("`safe_level(x)` takes one suit".into()));
            };
            let Some(x) = simple_name(x) else {
                return Some(Err(format!(
                    "`safe_level({x})`: not a suit, a suit variable or `trump`"
                )));
            };
            return Some(Ok(Expr::Arith {
                arith: ArithOp::Sub,
                lhs: Box::new(add(name_path(&[x]), name_path(&["partner", x, "min"]))),
                rhs: Box::new(Expr::Int { value: 6 }),
            }));
        }
        return None;
    }
    if path.first()?.name != "we" || path.len() < 2 {
        return None;
    }
    let seg = &path[1];
    let partner = |mid: Segment| {
        let mut p = vec![
            Segment {
                name: "partner".into(),
                args: None,
            },
            mid,
        ];
        p.extend(suffix.iter().map(|n| Segment {
            name: n.to_string(),
            args: None,
        }));
        Expr::Path { path: p }
    };
    let plain = |n: &str| Segment {
        name: n.into(),
        args: None,
    };
    Some(Ok(match seg.name.as_str() {
        "fit" => {
            let Some([x]) = seg.args.as_deref() else {
                return Some(Err("`we.fit(x)` takes one suit".into()));
            };
            let Some(x) = simple_name(x) else {
                return Some(Err(format!(
                    "`we.fit({x})`: not a suit, a suit variable or `trump`"
                )));
            };
            add(name_path(&[x]), partner(plain(x)))
        }
        "points" | "hcp" if seg.args.is_none() => {
            add(name_path(&[&seg.name]), partner(plain(&seg.name)))
        }
        "tp" => {
            let mine = Expr::Path {
                path: vec![seg.clone()],
            };
            add(mine, partner(seg.clone()))
        }
        _ => return None,
    }))
}

/// Name the defining module's card parameters by their qualified names,
/// leaving the definition's own parameters alone.
fn qualify(e: &Expr, params: &HashSet<&str>, formals: &[String], module: &str) -> Expr {
    map_names(e, &|n: &str, first: bool| {
        (first && params.contains(n) && !formals.iter().any(|f| f == n))
            .then(|| qualified(module, n))
    })
}

/// Replace a definition's parameters by the arguments' names.
fn subst(e: &Expr, map: &HashMap<String, String>) -> Expr {
    map_names(e, &|n: &str, _| map.get(n).cloned())
}

/// Rename bare names: path segments without arguments (`first` says the
/// segment starts its path), `is` targets and the strains of calls. A
/// segment after the first is a seat's attribute (`partner.x.min`), which
/// a suit parameter renames but a card parameter never is.
fn map_names(e: &Expr, f: &dyn Fn(&str, bool) -> Option<String>) -> Expr {
    let rec = |x: &Expr| Box::new(map_names(x, f));
    match e {
        Expr::And { all } => Expr::And {
            all: all.iter().map(|x| map_names(x, f)).collect(),
        },
        Expr::Or { any } => Expr::Or {
            any: any.iter().map(|x| map_names(x, f)).collect(),
        },
        Expr::Not { expr } => Expr::Not { expr: rec(expr) },
        Expr::Maybe { expr } => Expr::Maybe { expr: rec(expr) },
        Expr::Neg { expr } => Expr::Neg { expr: rec(expr) },
        Expr::Cmp { cmp, lhs, rhs } => Expr::Cmp {
            cmp: *cmp,
            lhs: rec(lhs),
            rhs: rec(rhs),
        },
        Expr::Arith { arith, lhs, rhs } => Expr::Arith {
            arith: *arith,
            lhs: rec(lhs),
            rhs: rec(rhs),
        },
        Expr::InRange { expr, lo, hi } => Expr::InRange {
            expr: rec(expr),
            lo: rec(lo),
            hi: rec(hi),
        },
        Expr::InSet { expr, values } => Expr::InSet {
            expr: rec(expr),
            values: values.clone(),
        },
        Expr::Is { expr, what, not } => Expr::Is {
            expr: rec(expr),
            what: f(what, true).unwrap_or_else(|| what.clone()),
            not: *not,
        },
        Expr::Asked { kind } => Expr::Asked {
            kind: kind.as_ref().map(|k| rec(k)),
        },
        Expr::Answered { kind } => Expr::Answered {
            kind: kind.as_ref().map(|k| rec(k)),
        },
        Expr::Path { path } => Expr::Path {
            path: path
                .iter()
                .enumerate()
                .map(|(i, s)| Segment {
                    name: match &s.args {
                        None => f(&s.name, i == 0).unwrap_or_else(|| s.name.clone()),
                        Some(_) => s.name.clone(),
                    },
                    args: s
                        .args
                        .as_ref()
                        .map(|a| a.iter().map(|x| map_names(x, f)).collect()),
                })
                .collect(),
        },
        Expr::Call { call } => Expr::Call {
            call: match call {
                CallSpec::Bid { level, strain } => CallSpec::Bid {
                    level: *level,
                    strain: match strain {
                        StrainSpec::Var(v) | StrainSpec::Interp(v) => match f(v, true) {
                            Some(n) => StrainSpec::Interp(n),
                            None => strain.clone(),
                        },
                        s => s.clone(),
                    },
                },
                c => c.clone(),
            },
        },
        Expr::Shape { .. } | Expr::Int { .. } => e.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(src: &str) -> Module {
        bidspec::parse(src, "t.bid").unwrap()
    }

    fn when(m: &Module) -> &Expr {
        m.contexts[0].when.as_ref().unwrap()
    }

    fn expanded(defs: &str, cond: &str) -> (String, Vec<String>) {
        let d = module(&format!(
            "module d \"d\"\n  param style = general.style\n{defs}\n"
        ));
        let u = module(&format!("module u \"u\"\nwhen {cond}\n  P \"x\"\n"));
        let defines = Defines::new(&[d]);
        let mut errors = Vec::new();
        let e = defines.expand(when(&u), &mut errors);
        (e.to_string(), errors)
    }

    #[test]
    fn partnership_sums_are_written_out() {
        let (e, err) = expanded(
            "",
            "we.fit(S).min >= 8, we.points.max <= 24, we.fit(x) >= 9",
        );
        assert!(err.is_empty(), "{err:?}");
        assert_eq!(
            e,
            "S+partner.S.min>=8, points+partner.points.max<=24, x+partner.x>=9"
        );
        let (e, _) = expanded("", "we.tp(trump).min >= 33, safe_level(x) >= 3");
        assert_eq!(
            e,
            "tp(trump)+partner.tp(trump).min>=33, x+partner.x.min-6>=3"
        );
    }

    #[test]
    fn definitions_are_inlined_with_their_arguments() {
        let (e, err) = expanded(
            "define held(x) = has(A,x) | (x<=0, partner.x.min<=3), trump is not x\n\
             define both = held(C), held(D)",
            "both, held(y)",
        );
        assert!(err.is_empty(), "{err:?}");
        assert_eq!(
            e,
            "has(A,C) | (C<=0, partner.C.min<=3), trump is not C, \
             has(A,D) | (D<=0, partner.D.min<=3), trump is not D, \
             has(A,y) | (y<=0, partner.y.min<=3), trump is not y"
        );
    }

    #[test]
    fn a_definition_reads_its_own_modules_parameters() {
        let (e, _) = expanded("define styled = style is bba", "styled");
        assert_eq!(e, "d::style is bba");
    }

    /// Resolve the first context's `after` of `uses` against the named
    /// patterns of `defs`: each alternative as text, its condition after
    /// ` ? `.
    fn resolved(defs: &str, uses: &str) -> (Vec<String>, Vec<String>) {
        let d = module(&format!(
            "module d \"d\"\n  param kokish = two_level.two_clubs.kokish\ndefine strong = kokish\n{defs}\n"
        ));
        let u = module(&format!("module u \"u\"\nafter {uses}\n  P \"x\"\n"));
        let defines = Defines::new(std::slice::from_ref(&d));
        let auctions = Auctions::new(&[d]);
        let mut errors = Vec::new();
        let alts = auctions.resolve(u.contexts[0].after.as_ref().unwrap(), &defines, &mut errors);
        let text = alts
            .iter()
            .map(|a| {
                let calls: Vec<String> = a
                    .calls
                    .iter()
                    .map(|p| {
                        if p.theirs {
                            format!("({})", p.call)
                        } else {
                            p.call.to_string()
                        }
                    })
                    .collect();
                match &a.when {
                    None => calls.join(" "),
                    Some(w) => format!("{} ? {w}", calls.join(" ")),
                }
            })
            .collect();
        (text, errors)
    }

    const NT2: &str = "define auction nt2 = 2N (P) | 2C (P) 2D (P) 2N (P)\n  2C (P) 2D (P) 2H (P) 2S (P) 2N (P)  when kokish";

    #[test]
    fn named_auctions_are_written_out_in_order() {
        let (alts, err) = resolved(NT2, "nt2");
        assert!(err.is_empty(), "{err:?}");
        assert_eq!(
            alts,
            [
                "2N (P)",
                "2C (P) 2D (P) 2N (P)",
                "2C (P) 2D (P) 2H (P) 2S (P) 2N (P) ? d::kokish",
            ]
        );
    }

    #[test]
    fn a_suffix_extends_every_alternative() {
        let (alts, err) = resolved(NT2, "nt2 3C (P) 3D (P) | 1N (P) 3C (P) 3D (P)");
        assert!(err.is_empty(), "{err:?}");
        assert_eq!(
            alts,
            [
                "2N (P) 3C (P) 3D (P)",
                "2C (P) 2D (P) 2N (P) 3C (P) 3D (P)",
                "2C (P) 2D (P) 2H (P) 2S (P) 2N (P) 3C (P) 3D (P) ? d::kokish",
                "1N (P) 3C (P) 3D (P)",
            ]
        );
    }

    #[test]
    fn named_auctions_nest_and_their_conditions_combine() {
        let defs = "define auction nt1 = 1N (P) | (1x) 1N (P)\n\
                    define auction nt1x = nt1 | (1x) 1N (X)\n\
                    define auction strong_nt1 = nt1x 2C (P) 2D (P) when strong";
        let (alts, err) = resolved(defs, "strong_nt1 2H (P)");
        assert!(err.is_empty(), "{err:?}");
        assert_eq!(
            alts,
            [
                "1N (P) 2C (P) 2D (P) 2H (P) ? d::kokish",
                "(1x) 1N (P) 2C (P) 2D (P) 2H (P) ? d::kokish",
                "(1x) 1N (X) 2C (P) 2D (P) 2H (P) ? d::kokish",
            ]
        );
    }

    #[test]
    fn unknown_and_cyclic_auctions_are_reported() {
        let (alts, err) = resolved(NT2, "nt3 3C (P) | 2N (P)");
        assert_eq!(alts, ["2N (P)"]);
        assert_eq!(
            err,
            ["no auction pattern `nt3` (`define auction nt3 = ...`)"]
        );
        let defs = "define auction aa = bb 2C (P)\ndefine auction bb = 1N (P) | aa 2D (P)";
        let (_, err) = resolved(defs, "aa");
        assert_eq!(
            err,
            ["the auction pattern `aa` is defined in terms of itself (aa -> bb -> aa)"]
        );
    }

    #[test]
    fn problems_are_reported() {
        let (_, err) = expanded("define held(x) = has(A,x)", "held");
        assert_eq!(err, ["`held` takes 1 argument (x)"]);
        let (_, err) = expanded("define held(x) = has(A,x)", "held(hcp + 1)");
        assert_eq!(err.len(), 1, "{err:?}");
        let (_, err) = expanded("define loop = loop", "loop");
        assert_eq!(err, ["`loop` is defined in terms of itself"]);
    }
}
