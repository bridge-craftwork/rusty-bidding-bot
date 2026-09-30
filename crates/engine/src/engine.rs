//! Replaying an auction (interpretation) and choosing a call (selection).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bidspec::ast::{Alert, CallSpec, Expr, PatternCall, StrainSpec};
use bridge_card::Card;
use bridge_types::{Call, Direction, Hand, ScoringMethod, Vulnerability};
use serde::Serialize;

use crate::eval::{strain_of_suit, suit_of_strain, Bindings, Ctx, PrivateCache, Val};
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

/// The candidates at a position, with the warnings met finding them.
type CandCache = HashMap<String, Arc<(Vec<Cand>, Vec<String>)>>;

/// Entries kept before the candidate cache is emptied and refilled, so a
/// run over the whole corpus stays within memory.
const CACHE_LIMIT: usize = 400_000;

/// What a position is made of: the calls so far and the board's
/// conditions. The candidates at a position depend on nothing else (never
/// on a hand), so they are found once per auction prefix and shared by the
/// boards that reach it: many share the first few calls. (Caching the
/// position after each call as well cost 18 GB over the corpus for a
/// small gain: deeper auctions rarely repeat.)
fn pos_key(pos: &Position) -> String {
    let mut k = format!("{:?} {:?} {:?}", pos.dealer, pos.vul, pos.scoring);
    for c in &pos.calls {
        k.push(' ');
        k.push_str(&c.to_pbn());
    }
    k
}

pub struct Engine {
    /// By side: 0 = North-South, 1 = East-West.
    systems: [System; 2],
    /// By side: how each side counts points (its card's `general.style`).
    valuation: [Valuation; 2],
    pool: Vec<Facts>,
    consistent: Mutex<IndexCache>,
    descriptiveness: Mutex<HashMap<String, f64>>,
    cands: Mutex<CandCache>,
}

impl Engine {
    /// An engine where North-South play `ns` and East-West play `ew` with
    /// `rules` (modules in a stable order, e.g. sorted by file).
    ///
    /// Both cards must be in the rules' vocabulary (loaded with
    /// `rules.vocab`): a card read through another vocabulary could resolve
    /// a field, an alias or a default differently from the one the rules
    /// were checked against, so this panics rather than bid with it.
    pub fn new(ns: &Card, ew: &Card, rules: &crate::RuleSet) -> Engine {
        for (side, card) in [("North-South", ns), ("East-West", ew)] {
            assert!(
                *card.vocabulary() == rules.vocab,
                "{side}'s card is in card vocabulary {} but the rules use {}: \
                 load cards with the rule set's vocabulary",
                card.vocabulary().id(),
                rules.vocab.id()
            );
        }
        let modules = &rules.modules;
        Engine {
            systems: [System::new(ns, modules), System::new(ew, modules)],
            valuation: [Valuation::for_card(ns), Valuation::for_card(ew)],
            pool: sample::pool(),
            consistent: Mutex::new(HashMap::new()),
            descriptiveness: Mutex::new(HashMap::new()),
            cands: Mutex::new(HashMap::new()),
        }
    }

    /// Count total points differently (weights in quarter points).
    pub fn with_valuation(mut self, valuation: Valuation) -> Engine {
        self.valuation = [valuation; 2];
        self.descriptiveness.lock().unwrap().clear();
        self.consistent.lock().unwrap().clear();
        self.cands.lock().unwrap().clear();
        self
    }

    /// The candidates at `pos`, computed once per position (`pos_key`).
    fn cands_at(&self, pos: &Position, key: &str) -> Arc<(Vec<Cand>, Vec<String>)> {
        if let Some(c) = self.cands.lock().unwrap().get(key) {
            return c.clone();
        }
        let mut warnings = Vec::new();
        let cands = self.candidates(pos, pos.next_caller(), &mut warnings);
        let c = Arc::new((cands, warnings));
        let mut cache = self.cands.lock().unwrap();
        if cache.len() >= CACHE_LIMIT {
            cache.clear();
        }
        cache.insert(key.to_string(), c.clone());
        c
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
            valuation: self.valuation[side(actor)],
            private: None,
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
                .all(|alts| match_any(alts, &pos.calls, &ctx, &mut b))
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
                // A `when` whose public parts cannot hold rules the rule out
                // (`x is not M` with x = M; `partner.M>=4` before partner has
                // shown four; `style is bba` under another style), whatever
                // it also says about the caller's hand, which is never judged
                // here: it is not known yet (see `publicly_impossible`).
                let impossible = entry
                    .rule
                    .when
                    .as_ref()
                    .is_some_and(|w| crate::eval::publicly_impossible(&ctx, w, &b));
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
                let v = self.valuation[side(actor)];
                let q = [f.points_q(v), f.suit_points_q(v)];
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
                    valuation: v,
                    private: None,
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
        // The calls fix everything a `shows` can read except the board's
        // conditions; a rule that reads those (`unfavourable`, `imps`) is
        // keyed on their values too, or boards bid in parallel would share
        // whichever came first.
        let board: Vec<String> = {
            let ctx = self.ctx(pos, actor, None, entry);
            entry
                .board_terms
                .iter()
                .map(|t| format!("{:?}", ctx.eval(t, &mut c.b.clone())))
                .collect()
        };
        let key = format!(
            "{}|{}|{}|{:?}|{:?}|{board:?}",
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
        // A condition counted in arithmetic (`+ doubler_four(x)`) would be
        // judged again for every hand: what the auction already settles is
        // settled once.
        let folded;
        let shows = if entry.counts_conditions {
            folded = self
                .ctx(pos, actor, None, entry)
                .fold_public(shows, &mut c.b.clone());
            &folded
        } else {
            shows
        };
        let mut pass = 0usize;
        let private = PrivateCache::default();
        // One copy of the bindings for the whole pool, restored only when
        // an evaluation bound something new (cloning per hand was a sixth
        // of the run).
        let mut b = c.b.clone();
        for &i in ids.iter() {
            let mut ctx = self.ctx(pos, actor, Some(&self.pool[i as usize]), entry);
            ctx.private = Some(&private);
            if ctx.cond(shows, &mut b) == Ok(Tri::True) {
                pass += 1;
            }
            if b.len() != c.b.len() {
                b = c.b.clone();
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
        let cands = self.cands_at(pos, &pos_key(pos));
        self.advance_from(pos, call, &cands.0, cands.1.clone())
    }

    /// Choose a call for `hand` at the current position.
    pub fn choose(&self, pos: &Position, hand: &Hand) -> Choice {
        let cands = self.cands_at(pos, &pos_key(pos));
        self.choose_from(pos, hand, &cands.0, cands.1.clone())
    }

    /// Both at once, sharing the candidate list: what would the engine call
    /// with `hand` here, and what does the `actual` call show? This is the
    /// step used to replay a reference auction.
    pub fn step(&self, pos: &mut Position, hand: &Hand, actual: &Call) -> (Choice, Step) {
        let cands = self.cands_at(pos, &pos_key(pos));
        let choice = self.choose_from(pos, hand, &cands.0, cands.1.clone());
        let step = self.advance_from(pos, actual, &cands.0, cands.1.clone());
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
                // Conditions on the caller's hand stay possible; the public
                // parts must hold (see `publicly_impossible`).
                e.rule
                    .when
                    .as_ref()
                    .is_none_or(|w| !crate::eval::publicly_impossible(&ctx, w, &c.b))
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
            // On a copy of the bindings: filling in the text must not
            // change what the rest of this step resolves.
            step.alert = fill_alert(&entry.rule.alert, &ctx, &b);
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
                // Only when every term resolves: a dropped term would make
                // the denial claim more than we know. Resolved part by part,
                // which is what resolving their conjunction does, without
                // copying the rule's trees first.
                let resolved: Option<Vec<Expr>> = parts
                    .iter()
                    .map(|p| ctx.resolve_exact(p, &mut ob))
                    .collect();
                if let Some(resolved) = resolved.map(|all| Expr::And { all }) {
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
        if !call.is_pass() {
            pos.sides[1 - side(caller)].opponent_acted(caller);
        }
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
                (pos, call),
                &mut warnings,
            );
            pos.sides[side(caller)] = st;
        }
        // The auction's own game forces (`force game after ... when ...`),
        // whichever rule made or explains the call.
        if sys.forces.iter().any(|f| {
            let ctx = Ctx {
                pos,
                actor: caller,
                hand: None,
                params: &sys.params[f.module],
                valuation: self.valuation[side(caller)],
                private: None,
            };
            let mut b = Bindings::new();
            if let Some(alts) = &f.after {
                if !match_any(alts, &pos.calls, &ctx, &mut b) {
                    return false;
                }
            }
            b.insert("call".into(), Val::Call(call.clone()));
            f.when.as_ref().is_none_or(|w| ctx.cond(w, &mut b) == Ok(Tri::True))
        }) {
            let st = &mut pos.sides[side(caller)];
            if st.forcing != Forcing::Game {
                st.forcing = Forcing::Game;
                st.forcing_by = Some(caller);
            }
        }
        step.knowledge = k.clone();
        step.warnings = warnings;
        pos.knowledge[caller.to_index()] = k;
        pos.apply_deck_hcp();
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
        let forced = pos.must_bid(actor);

        struct Eligible {
            idx: usize,
            prefer: f64,
        }
        let mut traces = Vec::new();
        let mut eligible: Vec<Eligible> = Vec::new();
        let mut alerts = Vec::new();
        let private = PrivateCache::default();
        for (idx, c) in cands.iter().enumerate() {
            let entry = &sys.rules[c.entry];
            let mut ctx = self.ctx(pos, actor, Some(&facts), entry);
            ctx.private = Some(&private);
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
            alerts.push(fill_alert(&entry.rule.alert, &ctx, &b));
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
                (
                    t.call.clone(),
                    t.explanation.clone(),
                    alerts[idx].take(),
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

    /// Bid a whole deal: `prefix` first (forced calls), then the engine's
    /// call for each seat's hand until the auction ends or reaches
    /// `max_calls`. `hands` is indexed by `Direction::to_index`.
    ///
    /// Each call keeps how it was read (`step`: what it shows, its alert,
    /// the rule that gave it its meaning, as partner and the opponents read
    /// it) and, for the engine's own calls, the choice with the hand.
    /// `Err` names the first prefix call that is not legal.
    pub fn bid_deal(
        &self,
        hands: &[Hand; 4],
        dealer: Direction,
        vul: Vulnerability,
        scoring: ScoringMethod,
        prefix: &[Call],
        max_calls: usize,
    ) -> Result<DealAuction, String> {
        let mut pos = self.start(dealer, vul, scoring);
        let mut calls = Vec::new();
        for (i, call) in prefix.iter().enumerate() {
            let auction = pos.auction();
            if auction.is_complete() {
                return Err(format!(
                    "call {} ({call}): the auction is already over",
                    i + 1
                ));
            }
            if !auction.is_legal(call) {
                return Err(format!("call {} ({call}) is not legal here", i + 1));
            }
            let step = self.advance(&mut pos, call);
            calls.push(DealCall { step, choice: None });
        }
        let mut complete = pos.auction().is_complete();
        while !complete && calls.len() < max_calls {
            let seat = pos.next_caller();
            let choice = self.choose(&pos, &hands[seat.to_index()]);
            let step = self.advance(&mut pos, &choice.call);
            calls.push(DealCall {
                step,
                choice: Some(choice),
            });
            complete = pos.auction().is_complete();
        }
        Ok(DealAuction { calls, complete })
    }

    /// Bid the bot seats of a practice table in turn: the `given` calls
    /// first (anyone's: a human's, or a call another bidder made where this
    /// engine had no rule), then the engine's call for each bot seat until
    /// the auction ends, a seat that is not a bot is to call, or, with
    /// `OnNoRule::Stop`, a bot seat reaches a position where no rule
    /// applies. The caller then supplies that call itself (the table's
    /// fallback bidder), appends it to `given` and calls again.
    ///
    /// Every call, given or made, is read the same way (`step`), so the
    /// meanings of a human's call and of a fallback call come out as
    /// `interpret` gives them. `Err` names the first given call that is not
    /// legal, or a bot seat without a hand.
    pub fn auction(
        &self,
        table: &Table,
        given: &[Call],
        on_no_rule: OnNoRule,
    ) -> Result<TableAuction, String> {
        for seat in SEATS {
            if table.bots[seat.to_index()] && table.hands[seat.to_index()].is_none() {
                return Err(format!("{} is a bot seat without a hand", seat.to_char()));
            }
        }
        let mut pos = self.start(table.dealer, table.vul, table.scoring);
        let mut calls = Vec::new();
        for (i, call) in given.iter().enumerate() {
            let auction = pos.auction();
            if auction.is_complete() {
                return Err(format!(
                    "call {} ({call}): the auction is already over",
                    i + 1
                ));
            }
            if !auction.is_legal(call) {
                return Err(format!("call {} ({call}) is not legal here", i + 1));
            }
            let step = self.advance(&mut pos, call);
            calls.push(DealCall { step, choice: None });
        }
        let stop = loop {
            let index = calls.len();
            if pos.auction().is_complete() {
                break TableStop {
                    reason: StopReason::Complete,
                    seat: None,
                    index,
                    choice: None,
                };
            }
            let seat = pos.next_caller();
            let Some(hand) = table.hands[seat.to_index()]
                .as_ref()
                .filter(|_| table.bots[seat.to_index()])
            else {
                break TableStop {
                    reason: StopReason::HumanToCall,
                    seat: Some(seat),
                    index,
                    choice: None,
                };
            };
            let choice = self.choose(&pos, hand);
            if choice.rule.is_none() && on_no_rule == OnNoRule::Stop {
                break TableStop {
                    reason: StopReason::NoRule,
                    seat: Some(seat),
                    index,
                    choice: Some(choice),
                };
            }
            let step = self.advance(&mut pos, &choice.call);
            calls.push(DealCall {
                step,
                choice: Some(choice),
            });
        };
        Ok(TableAuction {
            given: given.len(),
            calls,
            stop,
            position: pos,
        })
    }
}

const SEATS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

/// A practice table for `Engine::auction`: the board, and which seats the
/// engine bids for.
#[derive(Debug, Clone)]
pub struct Table {
    pub dealer: Direction,
    pub vul: Vulnerability,
    pub scoring: ScoringMethod,
    /// By `Direction::to_index`. Every bot seat needs its hand; the others
    /// may be left out (they are not looked at).
    pub hands: [Option<Hand>; 4],
    /// By `Direction::to_index`: the seats the engine bids for.
    pub bots: [bool; 4],
}

/// What `Engine::auction` does at a bot seat where no rule applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnNoRule {
    /// Stop there, so the caller can take another bidder's call.
    Stop,
    /// Pass (what `bid` and `bid_deal` do) and go on.
    Pass,
}

/// Why `Engine::auction` stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// The auction is over.
    Complete,
    /// A seat the engine does not bid for is to call.
    HumanToCall,
    /// A bot seat is to call and no rule applies (with `OnNoRule::Stop`).
    NoRule,
}

/// Where `Engine::auction` stopped.
#[derive(Debug, Clone, Serialize)]
pub struct TableStop {
    pub reason: StopReason,
    /// The seat to call; `None` when the auction is complete.
    pub seat: Option<Direction>,
    /// The index (0-based, from the dealer's first call) of the next call:
    /// the number of calls so far.
    pub index: usize,
    /// At a `NoRule` stop, the engine's choice there: a pass with no rule,
    /// and every candidate with why it failed.
    pub choice: Option<Choice>,
}

/// A practice-table auction bid by `Engine::auction`.
#[derive(Debug, Clone, Serialize)]
pub struct TableAuction {
    /// Every call so far: the given ones first (`choice` is `None`), then
    /// the engine's (`choice` is `Some`).
    pub calls: Vec<DealCall>,
    /// How many of `calls` were given.
    pub given: usize,
    pub stop: TableStop,
    /// The table after the last call.
    pub position: Position,
}

/// One call of an auction bid by `Engine::bid_deal`.
#[derive(Debug, Clone, Serialize)]
pub struct DealCall {
    /// How the call was read (`step.caller`, `step.call`, what it shows).
    pub step: Step,
    /// The engine's choice with the caller's hand; `None` for a call forced
    /// by the prefix.
    pub choice: Option<Choice>,
}

/// An auction bid by `Engine::bid_deal`.
#[derive(Debug, Clone, Serialize)]
pub struct DealAuction {
    pub calls: Vec<DealCall>,
    /// False when the auction was stopped at `max_calls`.
    pub complete: bool,
}

/// An alert with `{name}`s in its text filled in, as explanations are
/// (`{nt_min} to {nt_max}` is announced as `15 to 17`). Works on a copy of
/// the bindings, so it cannot change anything else the caller resolves.
fn fill_alert(alert: &Option<Alert>, ctx: &Ctx, b: &Bindings) -> Option<Alert> {
    let fill = |t: &str| {
        if t.contains('{') {
            ctx.interpolate(t, &mut b.clone())
        } else {
            t.to_string()
        }
    };
    alert.as_ref().map(|a| match a {
        Alert::Alert { text } => Alert::Alert {
            text: text.as_deref().map(fill),
        },
        Alert::Announce { text } => Alert::Announce { text: fill(text) },
    })
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

/// The state names `sets` may assign, with the form of their value
/// (`args`), as `rbb bid terms` prints them. `check_sets` accepts exactly
/// these and `apply_sets` applies them; `sets_match_apply_sets` keeps the
/// two in step.
pub const SET_KEYS: &[crate::eval::Term] = &[
    crate::eval::Term {
        name: "forcing",
        args: "=round/game/none",
        meaning: "`round`: partner may not pass if RHO passes; `game`: neither \
                  partner may pass below game (a round force never weakens it); \
                  `none` ends a force",
    },
    crate::eval::Term {
        name: "trump",
        args: "=x",
        meaning: "the agreed strain: a suit, `N`/`NT`, a suit variable or `trump`",
    },
    crate::eval::Term {
        name: "ladder",
        args: "=control/stopper",
        meaning: "a ladder call (control or stopper bids, recorded alike): the \
                  suits it skipped, other than trump, become `denied(x)` for the \
                  caller, the suit it names `cued(x)`",
    },
    crate::eval::Term {
        name: "ask",
        args: "=kind(x, ...)",
        meaning: "a question to partner, read by `asked kind(...)` and \
                  `answered kind(...)`; the kind is a free name, the arguments \
                  (optional) each a suit, `N`/`NT`, a suit variable or `trump`",
    },
];

/// The values `sets ladder=` accepts (the engine records them alike).
const LADDER_KINDS: &[&str] = &["control", "stopper"];

/// A bare word (`round`, `control`, `x`), if `e` is one.
fn bare_word(e: &Expr) -> Option<&str> {
    match e {
        Expr::Path { path } if path.len() == 1 && path[0].args.is_none() => {
            Some(path[0].name.as_str())
        }
        _ => None,
    }
}

/// A strain as `sets trump=` and an `ask` argument take it.
fn strain_form(e: &Expr) -> bool {
    bare_word(e).is_some_and(|n| {
        crate::knowledge::suit_index(n).is_some()
            || matches!(n, "N" | "NT" | "trump")
            || crate::eval::is_variable(n)
    })
}

/// What is wrong with one `sets` assignment, if anything.
fn check_assign(a: &bidspec::ast::Assign) -> Option<String> {
    let v = &a.value;
    match a.name.as_str() {
        "forcing" => match bare_word(v) {
            Some("round" | "game" | "none") => None,
            _ => Some(format!("sets forcing: `{v}` is not round, game or none")),
        },
        "trump" if strain_form(v) => None,
        "trump" => Some(format!(
            "sets trump: `{v}` is not a strain (a suit, N, a suit variable or trump)"
        )),
        "ladder" => match bare_word(v) {
            Some(w) if LADDER_KINDS.contains(&w) => None,
            _ => Some(format!(
                "sets ladder: `{v}` is not {}",
                LADDER_KINDS.join(" or ")
            )),
        },
        "ask" => match v {
            Expr::Path { path } if path.len() == 1 => {
                let bad: Vec<String> = path[0]
                    .args
                    .iter()
                    .flatten()
                    .filter(|x| !strain_form(x))
                    .map(|x| format!("`{x}`"))
                    .collect();
                (!bad.is_empty()).then(|| {
                    format!(
                        "sets ask: {} is not a strain (a suit, N, a suit variable or trump)",
                        bad.join(", ")
                    )
                })
            }
            _ => Some(format!(
                "sets ask: `{v}` is not a question (`kind` or `kind(x)`)"
            )),
        },
        n => Some(format!(
            "sets: unknown state `{n}` (known: {})",
            SET_KEYS
                .iter()
                .map(|k| k.name)
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Every `sets` in every rule names a state `apply_sets` knows, with a
/// value of the right form. Errors name the file and the rule's line:
/// otherwise a misspelt state is only a warning in the trace when the
/// rule is used, and the state is silently not set.
pub fn check_sets(modules: &[bidspec::Module]) -> Vec<bidspec::Diagnostic> {
    fn walk(m: &bidspec::Module, c: &bidspec::ast::Context, out: &mut Vec<bidspec::Diagnostic>) {
        for r in &c.rules {
            for a in &r.sets {
                if let Some(message) = check_assign(a) {
                    out.push(bidspec::Diagnostic {
                        file: m.file.clone(),
                        line: r.line,
                        col: 0,
                        message,
                    });
                }
            }
        }
        for inner in &c.contexts {
            walk(m, inner, out);
        }
    }
    let mut out = Vec::new();
    for m in modules {
        for c in &m.contexts {
            walk(m, c, &mut out);
        }
    }
    out
}

fn apply_sets(
    st: &mut SideState,
    sets: &[bidspec::ast::Assign],
    ctx: &Ctx,
    b: &mut Bindings,
    caller: Direction,
    (pos, call): (&Position, &Call),
    warnings: &mut Vec<String>,
) {
    for a in sets {
        match a.name.as_str() {
            "trump" => {
                let t = match ctx.eval(&a.value, b) {
                    Ok(Val::Suit(s)) => Some(strain_of_suit(s)),
                    Ok(Val::Strain(s)) => Some(s),
                    other => {
                        warnings.push(format!("sets trump: {other:?} is not a strain"));
                        None
                    }
                };
                if let Some(t) = t {
                    st.trump = Some(t);
                }
            }
            // `sets ladder=control`: a control-bid ladder call. Record the
            // suits it skipped (not the trump suit) as denied by the caller,
            // and the suit it names as cued.
            "ladder" => {
                let trump = st.trump.and_then(suit_of_strain);
                let who = caller.to_index();
                for s in 0..4 {
                    if Some(s) != trump && pos.would_skip(call, s) {
                        st.denied[who] |= 1 << s;
                    }
                }
                if let Call::Bid { strain, .. } = call {
                    if let Some(s) = suit_of_strain(*strain) {
                        if Some(s) != trump {
                            st.cued[who] |= 1 << s;
                        }
                    }
                }
            }
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

/// Does the auction end with any of one `after` line's alternatives?
/// The first that matches keeps its bindings; the others leave none.
fn match_any(alts: &[Vec<PatternCall>], calls: &[Call], ctx: &Ctx, b: &mut Bindings) -> bool {
    for p in alts {
        let mut trial = b.clone();
        if match_pattern(p, calls, ctx, &mut trial) {
            *b = trial;
            return true;
        }
    }
    false
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
        // `cheapest(w)` with `w` not yet bound: one candidate per suit, as
        // `2x` expands (before, no call at all, so the rule never applied).
        CallSpec::Relative {
            func,
            arg: Some(arg),
        } if (func == "cheapest" || func == "jump")
            && crate::eval::is_variable(arg)
            && !b.contains_key(arg)
            && !ctx.params.contains_key(arg) =>
        {
            (0..4)
                .filter(|&s| var_allows(arg, s))
                .flat_map(|s| {
                    let mut nb = b.clone();
                    nb.insert(arg.clone(), Val::Suit(s));
                    expand(spec, ctx, nb, auction)
                })
                .collect()
        }
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

#[cfg(test)]
mod sets_tests {
    use super::*;
    use bridge_types::Strain;

    fn compile(src: &str) -> bidspec::Module {
        bidspec::compile(src, "t.bid", &bridge_card::Registry::parse("").unwrap()).unwrap()
    }

    fn rule_sets(sets: &str) -> Vec<bidspec::ast::Assign> {
        let m = compile(&format!(
            "module t \"t\"\nafter 1S (P)\n  2S  \"x\"  sets {sets}\n"
        ));
        m.contexts[0].rules[0].sets.clone()
    }

    /// `apply_sets` on one assignment, spades agreed and `x` and `M`
    /// bound: its warnings.
    fn apply(a: &bidspec::ast::Assign) -> Vec<String> {
        let mut pos = Position::new(
            Direction::South,
            Vulnerability::None,
            ScoringMethod::Matchpoints,
        );
        for s in &mut pos.sides {
            s.trump = Some(Strain::Spades);
        }
        let params = HashMap::new();
        let ctx = Ctx {
            pos: &pos,
            actor: Direction::South,
            hand: None,
            params: &params,
            valuation: Valuation::default(),
            private: None,
        };
        let mut st = SideState {
            trump: Some(Strain::Spades),
            ..Default::default()
        };
        let mut b = Bindings::new();
        b.insert("x".into(), Val::Suit(2));
        b.insert("M".into(), Val::Suit(3));
        let call = Call::Bid {
            level: 2,
            strain: Strain::Spades,
        };
        let mut warnings = Vec::new();
        apply_sets(
            &mut st,
            std::slice::from_ref(a),
            &ctx,
            &mut b,
            Direction::South,
            (&pos, &call),
            &mut warnings,
        );
        warnings
    }

    /// Every value the check accepts, `apply_sets` applies without a
    /// warning, and the keys are those of `SET_KEYS`; every value it
    /// refuses, `apply_sets` warns about (`ladder` ignores its value).
    #[test]
    fn sets_match_apply_sets() {
        let good = [
            "forcing=round",
            "forcing=game",
            "forcing=none",
            "trump=H",
            "trump=N",
            "trump=NT",
            "trump=x",
            "trump=M",
            "trump=trump",
            "ladder=control",
            "ladder=stopper",
            "ask=signoff",
            "ask=invite(x)",
            "ask=keycards(trump)",
            "ask=control(S), forcing=game",
        ];
        let mut keys = std::collections::BTreeSet::new();
        for s in good {
            for a in rule_sets(s) {
                assert_eq!(check_assign(&a), None, "{s}");
                assert!(apply(&a).is_empty(), "{s}: {:?}", apply(&a));
                keys.insert(a.name.clone());
            }
        }
        let known: std::collections::BTreeSet<String> =
            SET_KEYS.iter().map(|k| k.name.to_string()).collect();
        assert_eq!(keys, known);
        for s in [
            "forcing=gam",
            "trump=Q",
            "trump=hcp",
            "trump=partner.S",
            "ask=invite(Q)",
            "ask=keycards(hcp)",
            "ask=partner.invite",
            "force=game",
        ] {
            let a = &rule_sets(s)[0];
            assert!(check_assign(a).is_some(), "{s} passed the check");
            assert!(!apply(a).is_empty(), "{s}: apply_sets did not warn");
        }
        // Refused although `apply_sets` would read them.
        for s in ["ladder=cue", "forcing=round(S)"] {
            assert!(check_assign(&rule_sets(s)[0]).is_some(), "{s}");
        }
    }

    #[test]
    fn check_sets_names_file_and_line() {
        let m = compile(
            "module t \"t\"\nafter 1S (P)\n  2S  \"x\"  sets forcing=round\n  \
             3S  \"y\"  sets trump=Q\n",
        );
        let d = check_sets(&[m]);
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!((d[0].file.as_str(), d[0].line), ("t.bid", 4));
    }
}
