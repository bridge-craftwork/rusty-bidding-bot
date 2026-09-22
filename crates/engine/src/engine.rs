//! Replaying an auction (interpretation) and choosing a call (selection).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bidspec::ast::{Alert, CallSpec, Expr, PatternCall, StrainSpec};
use bridge_card::Card;
use bridge_types::{Call, Direction, Hand, ScoringMethod, Vulnerability};
use serde::Serialize;

use crate::eval::{strain_of_suit, suit_of_strain, Bindings, Ctx, Val};
use crate::facts::{Facts, Valuation};
use crate::knowledge::{SeatKnowledge, Tri};
use crate::position::{side, Ask, Forcing, Position, SideState};
use crate::sample;
use crate::system::{RuleEntry, RuleRef, System};

/// A candidate: one rule, one concrete call, and the variable bindings that
/// produced it.
#[derive(Debug, Clone)]
struct Cand {
    entry: usize,
    call: Call,
    b: Bindings,
    /// Hand-independent rank: priority, then descriptiveness.
    priority: i64,
    descriptiveness: f64,
}

impl Cand {
    /// Ranks higher than `o` on the hand-independent keys.
    fn outranks(&self, o: &Cand) -> bool {
        (self.priority, self.descriptiveness) > (o.priority, o.descriptiveness)
    }
}

/// One call of an interpreted auction.
#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub caller: Direction,
    pub call: Call,
    /// The rule that gave the call its meaning, if any.
    pub rule: Option<RuleRef>,
    pub explanation: Option<String>,
    pub alert: Option<Alert>,
    /// The rule marked the call `artificial`: not a place to play.
    pub artificial: bool,
    /// What the caller's hand is known to hold after this call.
    pub knowledge: SeatKnowledge,
    /// Problems met while interpreting (unknown terms, contradictions).
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Interpretation {
    pub steps: Vec<Step>,
    pub position: Position,
}

/// How one candidate fared when choosing a call.
#[derive(Debug, Clone, Serialize)]
pub struct CandidateTrace {
    pub call: Call,
    pub rule: RuleRef,
    pub explanation: String,
    pub priority: i64,
    /// Share of hands (consistent with what the caller has shown) that this
    /// call would rule out: higher says more.
    pub descriptiveness: f64,
    pub prefer: Option<f64>,
    /// "chosen", "outranked", or why the hand does not qualify.
    pub outcome: String,
}

/// The engine's call at one position, and how every candidate fared.
#[derive(Debug, Clone, Serialize)]
pub struct Choice {
    pub call: Call,
    pub explanation: String,
    pub alert: Option<Alert>,
    pub rule: Option<RuleRef>,
    /// Every candidate considered, best-ranked first.
    pub candidates: Vec<CandidateTrace>,
    pub warnings: Vec<String>,
}

/// The engine's call, with everything needed to explain it.
#[derive(Debug, Clone, Serialize)]
pub struct Decision {
    pub call: Call,
    pub explanation: String,
    pub alert: Option<Alert>,
    pub rule: Option<RuleRef>,
    /// Every candidate considered, best-ranked first.
    pub candidates: Vec<CandidateTrace>,
    /// The auction as interpreted up to this call.
    pub auction: Interpretation,
    pub warnings: Vec<String>,
}

type IndexCache = HashMap<String, Arc<Vec<u32>>>;

pub struct Engine {
    /// By side: 0 = North-South, 1 = East-West.
    systems: [System; 2],
    valuation: Valuation,
    pool: Vec<Facts>,
    consistent: Mutex<IndexCache>,
    descriptiveness: Mutex<HashMap<String, f64>>,
}

impl Engine {
    /// An engine where North-South play `ns` and East-West play `ew`.
    /// `modules` should be in a stable order (e.g. sorted by file).
    pub fn new(ns: &Card, ew: &Card, modules: &[bidspec::Module]) -> Engine {
        Engine {
            systems: [System::new(ns, modules), System::new(ew, modules)],
            valuation: Valuation::default(),
            pool: sample::pool(),
            consistent: Mutex::new(HashMap::new()),
            descriptiveness: Mutex::new(HashMap::new()),
        }
    }

    /// Count total points differently (weights in quarter points).
    pub fn with_valuation(mut self, valuation: Valuation) -> Engine {
        self.valuation = valuation;
        self.descriptiveness.lock().unwrap().clear();
        self.consistent.lock().unwrap().clear();
        self
    }

    pub fn system(&self, seat: Direction) -> &System {
        &self.systems[side(seat)]
    }

    fn ctx<'a>(
        &'a self,
        pos: &'a Position,
        actor: Direction,
        hand: Option<&'a Facts>,
        entry: &RuleEntry,
    ) -> Ctx<'a> {
        Ctx {
            pos,
            actor,
            hand,
            params: &self.systems[side(actor)].params[entry.module],
            valuation: self.valuation,
        }
    }

    /// Every rule of `actor`'s side whose context holds, expanded to legal
    /// calls, ranked best first on the hand-independent keys.
    fn candidates(
        &self,
        pos: &Position,
        actor: Direction,
        warnings: &mut Vec<String>,
    ) -> Vec<Cand> {
        let sys = &self.systems[side(actor)];
        let auction = pos.auction();
        let mut out = Vec::new();
        for (i, entry) in sys.rules.iter().enumerate() {
            let ctx = self.ctx(pos, actor, None, entry);
            let mut b = Bindings::new();
            if !entry
                .patterns
                .iter()
                .all(|p| match_pattern(p, &pos.calls, &ctx, &mut b))
            {
                continue;
            }
            let mut holds = true;
            for c in &entry.conditions {
                match ctx.cond(c, &mut b) {
                    Ok(Tri::True) => {}
                    Ok(_) => holds = false,
                    Err(e) => {
                        warnings.push(format!("{}:{}: {e}", entry.source.file, entry.source.line));
                        holds = false;
                    }
                }
                if !holds {
                    break;
                }
            }
            if !holds {
                continue;
            }
            for (call, b) in expand(&entry.rule.call, &ctx, b, &auction) {
                // A `when` that cannot hold for this caller rules the rule out:
                // known false, or not known true while depending only on public
                // knowledge (`x is not M` with x = M; `partner.M>=4` before
                // partner has shown four).
                // A condition on the caller's own hand is never judged here: the
                // hand is not known yet (and `.max` of an unknown count means
                // nothing).
                let impossible = entry.rule.when.as_ref().is_some_and(|w| {
                    !crate::eval::hand_dependent(w, &b)
                        && ctx.cond(w, &mut b.clone()) != Ok(Tri::True)
                });
                if auction.is_legal(&call) && !impossible {
                    out.push(Cand {
                        entry: i,
                        call,
                        b,
                        priority: entry.rule.priority.unwrap_or(0),
                        descriptiveness: 0.0,
                    });
                }
            }
        }
        for c in &mut out {
            c.descriptiveness = self.descriptiveness_of(pos, actor, c);
        }
        out.sort_by(|a, b| {
            (b.priority, b.descriptiveness)
                .partial_cmp(&(a.priority, a.descriptiveness))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(sys.rules[a.entry].order.cmp(&sys.rules[b.entry].order))
        });
        out
    }

    /// Indices of sample hands consistent with what `actor` has shown.
    fn consistent_hands(&self, pos: &Position, actor: Direction) -> Arc<Vec<u32>> {
        let key = format!("{:?}|{:?}", actor, pos.knowledge(actor).constraints);
        if let Some(v) = self.consistent.lock().unwrap().get(&key) {
            return v.clone();
        }
        let k = pos.knowledge(actor);
        let params = HashMap::new();
        let ids: Vec<u32> = self
            .pool
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                let q = [f.points_q(self.valuation), f.suit_points_q(self.valuation)];
                if !(k.hcp.lo <= f.hcp && f.hcp <= k.hcp.hi)
                    || (0..2).any(|i| !(k.pts[i].lo <= q[i] && q[i] <= k.pts[i].hi))
                    || (0..4).any(|s| !(k.len[s].lo <= f.len[s] && f.len[s] <= k.len[s].hi))
                {
                    return false;
                }
                let ctx = Ctx {
                    pos,
                    actor,
                    hand: Some(f),
                    params: &params,
                    valuation: self.valuation,
                };
                k.constraints
                    .iter()
                    .all(|c| ctx.cond(c, &mut Bindings::new()) != Ok(Tri::False))
            })
            .map(|(i, _)| i as u32)
            .collect();
        let ids = Arc::new(ids);
        self.consistent.lock().unwrap().insert(key, ids.clone());
        ids
    }

    /// Share of the hands consistent with the actor's earlier calls that the
    /// candidate's `shows` rules out.
    fn descriptiveness_of(&self, pos: &Position, actor: Direction, c: &Cand) -> f64 {
        let entry = &self.systems[side(actor)].rules[c.entry];
        let Some(shows) = &entry.rule.shows else {
            return 0.0;
        };
        let key = format!(
            "{}|{}|{}|{:?}|{:?}",
            side(actor),
            c.entry,
            c.call,
            sorted(&c.b),
            pos.calls
        );
        if let Some(v) = self.descriptiveness.lock().unwrap().get(&key) {
            return *v;
        }
        let mut ids = self.consistent_hands(pos, actor);
        if ids.is_empty() {
            ids = Arc::new((0..self.pool.len() as u32).collect());
        }
        let mut pass = 0usize;
        for &i in ids.iter() {
            let ctx = self.ctx(pos, actor, Some(&self.pool[i as usize]), entry);
            if ctx.cond(shows, &mut c.b.clone()) == Ok(Tri::True) {
                pass += 1;
            }
        }
        let d = 1.0 - pass as f64 / ids.len() as f64;
        self.descriptiveness.lock().unwrap().insert(key, d);
        d
    }

    /// An empty auction.
    pub fn start(&self, dealer: Direction, vul: Vulnerability, scoring: ScoringMethod) -> Position {
        Position::new(dealer, vul, scoring)
    }

    /// Interpret one more call: what it shows, and its effect on the state.
    pub fn advance(&self, pos: &mut Position, call: &Call) -> Step {
        let mut warnings = Vec::new();
        let cands = self.candidates(pos, pos.next_caller(), &mut warnings);
        self.advance_from(pos, call, &cands, warnings)
    }

    /// Choose a call for `hand` at the current position.
    pub fn choose(&self, pos: &Position, hand: &Hand) -> Choice {
        let mut warnings = Vec::new();
        let cands = self.candidates(pos, pos.next_caller(), &mut warnings);
        self.choose_from(pos, hand, &cands, warnings)
    }

    /// Both at once, sharing the candidate list: what would the engine call
    /// with `hand` here, and what does the `actual` call show? This is the
    /// step used to replay a reference auction.
    pub fn step(&self, pos: &mut Position, hand: &Hand, actual: &Call) -> (Choice, Step) {
        let mut warnings = Vec::new();
        let cands = self.candidates(pos, pos.next_caller(), &mut warnings);
        let choice = self.choose_from(pos, hand, &cands, warnings.clone());
        let step = self.advance_from(pos, actual, &cands, warnings);
        (choice, step)
    }

    /// Replay `calls`, working out what each call showed.
    pub fn interpret(
        &self,
        dealer: Direction,
        vul: Vulnerability,
        scoring: ScoringMethod,
        calls: &[Call],
    ) -> Interpretation {
        let mut pos = self.start(dealer, vul, scoring);
        let steps = calls.iter().map(|c| self.advance(&mut pos, c)).collect();
        Interpretation {
            steps,
            position: pos,
        }
    }

    /// Choose a call for `hand` after `calls`.
    pub fn bid(
        &self,
        hand: &Hand,
        dealer: Direction,
        vul: Vulnerability,
        scoring: ScoringMethod,
        calls: &[Call],
    ) -> Decision {
        let auction = self.interpret(dealer, vul, scoring, calls);
        let c = self.choose(&auction.position, hand);
        Decision {
            call: c.call,
            explanation: c.explanation,
            alert: c.alert,
            rule: c.rule,
            candidates: c.candidates,
            auction,
            warnings: c.warnings,
        }
    }

    fn advance_from(
        &self,
        pos: &mut Position,
        call: &Call,
        cands: &[Cand],
        mut warnings: Vec<String>,
    ) -> Step {
        let caller = pos.next_caller();
        let sys = &self.systems[side(caller)];
        // Rules that could have produced this call. A `when` known to be
        // false for this caller rules the rule out.
        let matching: Vec<&Cand> = cands
            .iter()
            .filter(|c| &c.call == call)
            .filter(|c| {
                let e = &sys.rules[c.entry];
                let ctx = self.ctx(pos, caller, None, e);
                // Conditions on the caller's hand stay possible; public ones
                // were already required to hold (see `candidates`).
                e.rule.when.as_ref().is_none_or(|w| {
                    crate::eval::hand_dependent(w, &c.b)
                        || ctx.cond(w, &mut c.b.clone()) == Ok(Tri::True)
                })
            })
            .collect();
        // Lower-priority rules are fallbacks ("only if nothing better"): they
        // do not widen what the call means to partner.
        let top = matching.iter().map(|c| c.priority).max();
        let matching: Vec<&Cand> = matching
            .into_iter()
            .filter(|c| Some(c.priority) == top)
            .collect();
        let mut k = pos.knowledge(caller).clone();
        let mut step = Step {
            caller,
            call: call.clone(),
            rule: None,
            explanation: None,
            alert: None,
            artificial: false,
            knowledge: k.clone(),
            warnings: Vec::new(),
        };
        let best = matching.first().copied();
        if let Some(best) = best {
            let entry = &sys.rules[best.entry];
            let ctx = self.ctx(pos, caller, None, entry);
            let mut b = best.b.clone();
            step.rule = Some(entry.source.clone());
            step.explanation = Some(ctx.interpolate(&entry.rule.explanation, &mut b));
            step.alert = entry.rule.alert.clone();
            step.artificial = entry.rule.artificial;
            // What the call shows: the union over every rule that makes it.
            let meanings: Vec<Expr> = matching
                .iter()
                .filter_map(|c| {
                    let e = &sys.rules[c.entry];
                    let ctx = self.ctx(pos, caller, None, e);
                    e.rule
                        .shows
                        .as_ref()
                        .map(|s| ctx.resolve(s, &mut c.b.clone()))
                })
                .collect();
            if meanings.len() == matching.len() && !meanings.is_empty() {
                let meaning = if meanings.len() == 1 {
                    meanings.into_iter().next().unwrap()
                } else {
                    Expr::Or { any: meanings }
                };
                if !k.add(meaning) {
                    warnings.push(format!("{call} contradicts what {caller:?} showed before"));
                }
            }
            if let Some(d) = &entry.rule.denies {
                k.add(Expr::Not {
                    expr: Box::new(ctx.resolve(d, &mut b)),
                });
            }
            // Negative inference: the caller would have made any call that
            // outranks this one, had the hand qualified.
            for other in cands.iter().filter(|c| &c.call != call && c.outranks(best)) {
                let e = &sys.rules[other.entry];
                let ctx = self.ctx(pos, caller, None, e);
                let mut ob = other.b.clone();
                let parts: Vec<&Expr> = e.rule.shows.iter().chain(e.rule.when.iter()).collect();
                if parts.is_empty() {
                    continue;
                }
                let whole = Expr::And {
                    all: parts.into_iter().cloned().collect(),
                };
                // Only when every term resolves: a dropped term would make
                // the denial claim more than we know.
                if let Some(resolved) = ctx.resolve_exact(&whole, &mut ob) {
                    if !is_const(&resolved) {
                        k.add(Expr::Not {
                            expr: Box::new(resolved),
                        });
                    }
                }
            }
        }
        // State: questions answered, forcing satisfied, then this call's effects.
        advance_state(&mut pos.sides[side(caller)], caller);
        if let Some(best) = best {
            let entry = &sys.rules[best.entry];
            let ctx = self.ctx(pos, caller, None, entry);
            let mut b = best.b.clone();
            let mut st = pos.sides[side(caller)].clone();
            apply_sets(
                &mut st,
                &entry.rule.sets,
                &ctx,
                &mut b,
                caller,
                &mut warnings,
            );
            pos.sides[side(caller)] = st;
        }
        step.knowledge = k.clone();
        step.warnings = warnings;
        pos.knowledge[caller.to_index()] = k;
        pos.calls.push(call.clone());
        step
    }

    fn choose_from(
        &self,
        pos: &Position,
        hand: &Hand,
        cands: &[Cand],
        mut warnings: Vec<String>,
    ) -> Choice {
        let actor = pos.next_caller();
        let facts = Facts::new(hand);
        let sys = &self.systems[side(actor)];
        let state = pos.side_state(actor);
        let forced = (state.forcing == Forcing::Round && state.forcing_by == Some(actor.partner()))
            || (state.forcing == Forcing::Game && pos.below_game(actor));

        struct Eligible {
            idx: usize,
            prefer: f64,
        }
        let mut traces = Vec::new();
        let mut eligible: Vec<Eligible> = Vec::new();
        for (idx, c) in cands.iter().enumerate() {
            let entry = &sys.rules[c.entry];
            let ctx = self.ctx(pos, actor, Some(&facts), entry);
            let mut b = c.b.clone();
            let mut outcome = String::new();
            for (label, e) in [("shows", &entry.rule.shows), ("when", &entry.rule.when)] {
                let Some(e) = e else { continue };
                match ctx.cond(e, &mut b) {
                    Ok(Tri::True) => {}
                    Ok(_) => {
                        outcome = format!(
                            "hand fails `{label} {}`",
                            first_failing(&ctx, e, &mut b.clone())
                        );
                        break;
                    }
                    Err(err) => {
                        warnings.push(format!(
                            "{}:{}: {err}",
                            entry.source.file, entry.source.line
                        ));
                        outcome = format!("error: {err}");
                        break;
                    }
                }
            }
            if outcome.is_empty() && forced && c.call == Call::Pass {
                outcome = "partner's call is forcing".into();
            }
            let prefer = entry
                .rule
                .prefer
                .as_ref()
                .and_then(|p| ctx.eval(p, &mut b).ok())
                .and_then(|v| match v {
                    Val::Num(r) => Some(r.lo as f64),
                    _ => None,
                });
            if outcome.is_empty() {
                eligible.push(Eligible {
                    idx,
                    prefer: prefer.unwrap_or(0.0),
                });
            }
            traces.push(CandidateTrace {
                call: c.call.clone(),
                rule: entry.source.clone(),
                explanation: ctx.interpolate(&entry.rule.explanation, &mut b),
                priority: c.priority,
                descriptiveness: c.descriptiveness,
                prefer,
                outcome,
            });
        }
        // `cands` is already in priority/descriptiveness/file order; `prefer`
        // only reorders candidates equal on the first two keys.
        eligible.sort_by(|a, b| {
            let (ca, cb) = (&cands[a.idx], &cands[b.idx]);
            (cb.priority, cb.descriptiveness, b.prefer)
                .partial_cmp(&(ca.priority, ca.descriptiveness, a.prefer))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.idx.cmp(&b.idx))
        });
        let chosen = eligible.first().map(|e| e.idx);
        for (idx, t) in traces.iter_mut().enumerate() {
            if Some(idx) == chosen {
                t.outcome = "chosen".into();
            } else if t.outcome.is_empty() {
                t.outcome = "outranked".into();
            }
        }
        let (call, explanation, alert, rule) = match chosen {
            Some(idx) => {
                let t = &traces[idx];
                let entry = &sys.rules[cands[idx].entry];
                (
                    t.call.clone(),
                    t.explanation.clone(),
                    entry.rule.alert.clone(),
                    Some(t.rule.clone()),
                )
            }
            None => (Call::Pass, "No rule applies".into(), None, None),
        };
        Choice {
            call,
            explanation,
            alert,
            rule,
            candidates: traces,
            warnings,
        }
    }
}

fn sorted(b: &Bindings) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = b
        .iter()
        .map(|(k, v)| (k.clone(), format!("{v:?}")))
        .collect();
    v.sort();
    v
}

fn is_const(e: &Expr) -> bool {
    matches!(e, Expr::And { all } if all.is_empty())
        || matches!(e, Expr::Or { any } if any.is_empty())
}

/// The first conjunct of `e` that is not known true, for explanations.
fn first_failing(ctx: &Ctx, e: &Expr, b: &mut Bindings) -> String {
    if let Expr::And { all } = e {
        for sub in all {
            if ctx.cond(sub, b) != Ok(Tri::True) {
                return sub.to_string();
            }
        }
    }
    e.to_string()
}

/// Questions and forcing move on when a player calls.
fn advance_state(st: &mut SideState, caller: Direction) {
    if st.answered.as_ref().is_some_and(|a| a.by == caller) {
        st.answered = None;
    }
    if let Some(ask) = st.ask.take() {
        if ask.by == caller.partner() {
            st.answered = Some(ask);
        } else {
            st.ask = Some(ask);
        }
    }
    if st.forcing == Forcing::Round && st.forcing_by == Some(caller.partner()) {
        st.forcing = Forcing::None;
        st.forcing_by = None;
    }
}

fn apply_sets(
    st: &mut SideState,
    sets: &[bidspec::ast::Assign],
    ctx: &Ctx,
    b: &mut Bindings,
    caller: Direction,
    warnings: &mut Vec<String>,
) {
    for a in sets {
        match a.name.as_str() {
            "trump" => match ctx.eval(&a.value, b) {
                Ok(Val::Suit(s)) => st.trump = Some(strain_of_suit(s)),
                Ok(Val::Strain(s)) => st.trump = Some(s),
                other => warnings.push(format!("sets trump: {other:?} is not a strain")),
            },
            "forcing" => {
                let f = match &a.value {
                    Expr::Path { path } => match path[0].name.as_str() {
                        "round" => Some(Forcing::Round),
                        "game" => Some(Forcing::Game),
                        "none" => Some(Forcing::None),
                        _ => None,
                    },
                    _ => None,
                };
                match f {
                    Some(f) => {
                        // A round force never weakens a game force.
                        if !(f == Forcing::Round && st.forcing == Forcing::Game) {
                            st.forcing = f;
                            st.forcing_by = Some(caller);
                        }
                    }
                    None => warnings.push(format!(
                        "sets forcing: `{}` is not round, game or none",
                        a.value
                    )),
                }
            }
            "ask" => match &a.value {
                Expr::Path { path } if path.len() == 1 => {
                    let seg = &path[0];
                    let mut args = Vec::new();
                    for arg in seg.args.iter().flatten() {
                        match ctx.eval(arg, b) {
                            Ok(Val::Suit(s)) => args.push(strain_of_suit(s)),
                            Ok(Val::Strain(s)) => args.push(s),
                            other => {
                                warnings.push(format!("sets ask: argument `{arg}` is {other:?}"))
                            }
                        }
                    }
                    st.ask = Some(Ask {
                        kind: seg.name.clone(),
                        args,
                        by: caller,
                    });
                }
                other => warnings.push(format!("sets ask: `{other}` is not a question")),
            },
            n => warnings.push(format!("sets: unknown state `{n}`")),
        }
    }
}

/// Does the auction so far end with this pattern (after leading passes)?
fn match_pattern(p: &[PatternCall], calls: &[Call], ctx: &Ctx, b: &mut Bindings) -> bool {
    let (k, n) = (p.len(), calls.len());
    if k > n || !calls[..n - k].iter().all(Call::is_pass) {
        return false;
    }
    p.iter()
        .zip(&calls[n - k..])
        .all(|(pc, call)| match_call(&pc.call, call, ctx, b))
}

/// Suit variable classes: `M` is a major, `m` a minor, others any suit.
pub(crate) fn var_allows(var: &str, suit: usize) -> bool {
    match var {
        "M" => suit >= 2,
        "m" => suit < 2,
        _ => true,
    }
}

fn match_call(spec: &CallSpec, call: &Call, ctx: &Ctx, b: &mut Bindings) -> bool {
    match (spec, call) {
        (CallSpec::Any, _) => true,
        (CallSpec::Pass, Call::Pass)
        | (CallSpec::Double, Call::Double)
        | (CallSpec::Redouble, Call::Redouble) => true,
        (
            CallSpec::Bid { level, strain },
            Call::Bid {
                level: l,
                strain: s,
            },
        ) if level == l => match strain {
            StrainSpec::Lit(x) => crate::eval::strain_from_spec(*x) == *s,
            StrainSpec::Var(v) => match (b.get(v), suit_of_strain(*s)) {
                (Some(Val::Suit(bound)), Some(actual)) => *bound == actual,
                (None, Some(actual)) if var_allows(v, actual) => {
                    b.insert(v.clone(), Val::Suit(actual));
                    true
                }
                _ => false,
            },
            StrainSpec::Interp(n) => match ctx.eval(&crate::eval::path_expr(n), b) {
                Ok(Val::Strain(x)) => x == *s,
                Ok(Val::Suit(x)) => Some(x) == suit_of_strain(*s),
                _ => false,
            },
        },
        _ => false,
    }
}

/// The concrete calls a rule's call stands for.
fn expand(
    spec: &CallSpec,
    ctx: &Ctx,
    b: Bindings,
    auction: &bridge_types::Auction,
) -> Vec<(Call, Bindings)> {
    match spec {
        CallSpec::Bid {
            level,
            strain: StrainSpec::Var(v),
        } if !b.contains_key(v) => (0..4)
            .filter(|&s| var_allows(v, s))
            .map(|s| {
                let mut nb = b.clone();
                nb.insert(v.clone(), Val::Suit(s));
                (
                    Call::Bid {
                        level: *level,
                        strain: strain_of_suit(s),
                    },
                    nb,
                )
            })
            .collect(),
        CallSpec::Relative {
            func,
            arg: Some(arg),
        } if func == "cheapest" || func == "jump" => {
            let mut b = b;
            let strain = match ctx.eval(&crate::eval::path_expr(arg), &mut b) {
                Ok(Val::Suit(s)) => strain_of_suit(s),
                Ok(Val::Strain(s)) => s,
                _ => return vec![],
            };
            let cheapest = (1..=7).find(|&l| auction.is_legal(&Call::Bid { level: l, strain }));
            let level = match (cheapest, func.as_str()) {
                (Some(l), "cheapest") => l,
                (Some(l), _) if l < 7 => l + 1,
                _ => return vec![],
            };
            vec![(Call::Bid { level, strain }, b)]
        }
        _ => {
            let mut b = b;
            match ctx.concrete_call(spec, &mut b) {
                Ok(Some(c)) => vec![(c, b)],
                _ => vec![],
            }
        }
    }
}
