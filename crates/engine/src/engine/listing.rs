//! Which rules can make the next call, before anyone knows the hand
//! (`rbb bid rules`): the engine's own candidate list at a position
//! (`Engine::candidates`, the list `choose` ranks a hand against), with
//! the hand-dependent conditions left as conditions.

use std::collections::BTreeMap;

use super::*;

/// A rule (or one call it expanded to) that did not reach the candidate
/// list, and why (`Engine::candidates` with a sink).
#[derive(Debug, Clone)]
pub(crate) struct Rejected {
    pub entry: usize,
    /// The concrete call, when the rule got as far as naming one.
    pub call: Option<Call>,
    /// Where it failed: `after`, `context-when`, `call`, `legal`, `when`.
    pub stage: &'static str,
    pub reason: String,
}

/// One rule that can make the next call: its auction context (`after`
/// patterns, context `when`s, card parameters, asks, what the other seats
/// have shown) holds, the call is legal, and the public parts of its own
/// `when` are not ruled out. Whether it is chosen depends on the hand:
/// `shows` and the hand parts of `when`.
#[derive(Debug, Clone, Serialize)]
pub struct RuleOption {
    /// 1-based place in the engine's ranking (priority, descriptiveness,
    /// file order); `prefer`, which reads the hand, reorders only ties.
    pub rank: usize,
    /// The call in PBN (`2S`, `Pass`, `X`).
    pub call: String,
    pub rule: RuleRef,
    /// The rule's text with `{…}` filled in.
    pub explanation: String,
    pub alert: Option<Alert>,
    pub artificial: bool,
    pub priority: i64,
    /// Share of the hands consistent with the caller's earlier calls that
    /// `shows` rules out (the second ranking key).
    pub descriptiveness: f64,
    /// Load order, the last tie-break.
    pub order: usize,
    /// Pattern variables bound by the auction or the expansion (`x` = `S`).
    pub bindings: BTreeMap<String, String>,
    /// `shows` as written.
    pub shows: Option<String>,
    /// `shows` as the engine records it for partner (`Ctx::resolve`:
    /// definitions written out, variables and other seats' values filled
    /// in, state conditions settled).
    pub shows_resolved: Option<String>,
    /// `when` as written.
    pub when: Option<String>,
    /// `when` with what the auction settles folded in (`Ctx::fold_public`):
    /// what is left depends on the hand, or is not known yet.
    pub when_folded: Option<String>,
    /// The `when` reads the caller's hand (judged only with a hand).
    pub when_reads_hand: bool,
    pub denies: Option<String>,
    pub prefer: Option<String>,
    /// `sets` as written (`forcing=round`, `ask=keycards`, `trump=x`).
    pub sets: Vec<String>,
    /// Partner reads the call by this rule (the first of the call's
    /// top-priority rules, as `Engine::advance` does).
    pub read_as: bool,
    /// The other rules here that make the same call, best-ranked first.
    pub same_call: Vec<RuleRef>,
    /// Ruled out whatever the hand, at choosing time: a pass when
    /// partner's call is forcing.
    pub excluded: Option<String>,
}

/// A rule whose auction context or public `when` failed (`--all`).
#[derive(Debug, Clone, Serialize)]
pub struct RuleRejection {
    /// The call as written in the rule (`2x`, `cheapest(x)`).
    pub spec: String,
    /// The concrete call in PBN, when the rule got as far as one.
    pub call: Option<String>,
    pub rule: RuleRef,
    /// The rule's text, as written.
    pub explanation: String,
    pub priority: i64,
    /// `after`, `context-when`, `call`, `legal`, `when`.
    pub stage: String,
    pub reason: String,
}

/// The rules that can make the next call at a position.
#[derive(Debug, Clone, Serialize)]
pub struct RuleListing {
    /// The seat to call.
    pub seat: Direction,
    /// Partner's call forces this seat to bid.
    pub forced: bool,
    /// Best-ranked first.
    pub candidates: Vec<RuleOption>,
    /// Filled only when asked (`all`), in load order.
    pub rejected: Vec<RuleRejection>,
    /// Modules this seat's card leaves off, and why (with `all`).
    pub inactive: Vec<(String, String)>,
    pub warnings: Vec<String>,
}

impl Engine {
    /// Every rule that can make the next call at `pos`, ranked as the
    /// engine ranks them before it looks at a hand: the candidate list
    /// `choose` uses, read without a hand. With `all`, also every rule
    /// that failed, with the reason.
    pub fn rules_at(&self, pos: &Position, all: bool) -> RuleListing {
        let actor = pos.next_caller();
        let sys = &self.systems[side(actor)];
        let mut rejected = Vec::new();
        let (cands, warnings) = if all {
            let mut warnings = Vec::new();
            let c = self.candidates(pos, actor, &mut warnings, Some(&mut rejected));
            (c, warnings)
        } else {
            let c = self.cands_at(pos, &pos_key(pos));
            (c.0.clone(), c.1.clone())
        };
        let forced = pos.must_bid(actor);
        let text = |e: &Option<Expr>| e.as_ref().map(|e| e.to_string());
        let mut out: Vec<RuleOption> = Vec::with_capacity(cands.len());
        for (idx, c) in cands.iter().enumerate() {
            let entry = &sys.rules[c.entry];
            let ctx = self.ctx(pos, actor, None, entry);
            let r = &entry.rule;
            let same_call: Vec<RuleRef> = cands
                .iter()
                .enumerate()
                .filter(|(j, o)| *j != idx && o.call == c.call)
                .map(|(_, o)| sys.rules[o.entry].source.clone())
                .collect();
            let read_as = cands.iter().position(|o| o.call == c.call) == Some(idx);
            out.push(RuleOption {
                rank: idx + 1,
                call: c.call.to_pbn(),
                rule: entry.source.clone(),
                explanation: ctx.interpolate(&r.explanation, &mut c.b.clone()),
                alert: fill_alert(&r.alert, &ctx, &c.b),
                artificial: r.artificial,
                priority: c.priority,
                descriptiveness: c.descriptiveness,
                order: entry.order,
                bindings: c.b.iter().map(|(k, v)| (k.clone(), val_text(v))).collect(),
                shows: text(&entry.written.0),
                shows_resolved: r
                    .shows
                    .as_ref()
                    .map(|s| expr_text(&ctx.resolve(s, &mut c.b.clone()))),
                when: text(&entry.written.1),
                when_folded: r.when.as_ref().map(|w| {
                    expr_text(&crate::eval::simplify(ctx.fold_public(w, &mut c.b.clone())))
                }),
                when_reads_hand: r
                    .when
                    .as_ref()
                    .is_some_and(|w| crate::eval::hand_dependent(w, &c.b)),
                denies: text(&r.denies),
                prefer: text(&r.prefer),
                sets: r
                    .sets
                    .iter()
                    .map(|a| format!("{}={}", a.name, a.value))
                    .collect(),
                read_as,
                same_call,
                excluded: (forced && c.call == Call::Pass && !r.artificial)
                    .then(|| "partner's call is forcing".to_string()),
            });
        }
        let rejected = rejected
            .into_iter()
            .map(|x| {
                let entry = &sys.rules[x.entry];
                RuleRejection {
                    spec: entry.rule.call.to_string(),
                    call: x.call.map(|c| c.to_pbn()),
                    rule: entry.source.clone(),
                    explanation: entry.rule.explanation.clone(),
                    priority: entry.rule.priority.unwrap_or(0),
                    stage: x.stage.to_string(),
                    reason: x.reason,
                }
            })
            .collect();
        RuleListing {
            seat: actor,
            forced,
            candidates: out,
            rejected,
            inactive: if all {
                sys.inactive.clone()
            } else {
                Vec::new()
            },
            warnings,
        }
    }
}

/// An expression as a rule would write it; the constants `true` and
/// `false` (an empty conjunction or disjunction) by name.
fn expr_text(e: &Expr) -> String {
    match e {
        Expr::And { all } if all.is_empty() => "true".into(),
        Expr::Or { any } if any.is_empty() => "false".into(),
        _ => e.to_string(),
    }
}

/// A bound value as a rule would write it (`S`, `NT`, `3`).
fn val_text(v: &Val) -> String {
    match v {
        Val::Suit(s) => ["C", "D", "H", "S"][*s].to_string(),
        other => format!("{other:?}"),
    }
}

/// One `after` line's alternatives as written: `1C (P) 1D (1H) | ...`.
pub(super) fn alts_text(alts: &[Alt]) -> String {
    let shown: Vec<String> = alts
        .iter()
        .take(4)
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
                Some(w) => format!("`{}` when `{w}`", calls.join(" ")),
                None => format!("`{}`", calls.join(" ")),
            }
        })
        .collect();
    let more = alts.len().saturating_sub(4);
    if more > 0 {
        format!("{} (+{more} more)", shown.join(" | "))
    } else {
        shown.join(" | ")
    }
}

pub(super) fn tri_word(t: Tri) -> &'static str {
    match t {
        Tri::True => "true",
        Tri::False => "false",
        _ => "not known",
    }
}

/// The part of `when` that makes `publicly_impossible` hold: a public
/// conjunct not known true, or every branch of a disjunction failing.
/// Mirrors `eval::publicly_impossible`; `None` when it does not hold.
pub(super) fn public_failure(ctx: &Ctx, e: &Expr, b: &Bindings) -> Option<String> {
    match e {
        Expr::And { all } => all.iter().find_map(|x| public_failure(ctx, x, b)),
        Expr::Or { any } => {
            let parts: Option<Vec<String>> =
                any.iter().map(|x| public_failure(ctx, x, b)).collect();
            parts.map(|p| p.join(" | "))
        }
        _ if crate::eval::hand_dependent(e, b) => None,
        _ => match ctx.cond(e, &mut b.clone()) {
            Ok(Tri::True) => None,
            Ok(t) => Some(format!("`{e}` is {}", tri_word(t))),
            Err(err) => Some(format!("`{e}`: error: {err}")),
        },
    }
}
