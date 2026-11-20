//! Reading an auction with the opponents' announced meanings
//! (`crate::meanings`): a convention name the pack's table knows switches
//! that field on in their side's card (an engine variant, `variant`), and
//! for a call no rule reads, what the text says becomes what it shows.

use super::*;
use crate::meanings::{self, Meanings};
use crate::HeardRead;

/// What one call's announcement will do, worked out before the replay.
#[derive(Debug, Clone)]
struct Planned {
    text: String,
    /// The name matched and the card changes it makes.
    name: Option<(String, Vec<String>)>,
}

/// The announcements of an auction, call by call, and the card changes
/// they make, by side.
#[derive(Debug, Clone, Default)]
struct Plan {
    calls: Vec<Option<Planned>>,
    changes: [Vec<String>; 2],
}

/// How many variant engines one engine keeps before starting again.
const VARIANT_LIMIT: usize = 64;

/// The engine to read with: this one, or a variant with the opponents'
/// announced conventions on their card.
enum Reader<'a> {
    Own(&'a Engine),
    Variant(Arc<Engine>),
}

impl std::ops::Deref for Reader<'_> {
    type Target = Engine;
    fn deref(&self) -> &Engine {
        match self {
            Reader::Own(e) => e,
            Reader::Variant(e) => e,
        }
    }
}

/// What a seat's knowledge bounds, to tell whether a condition narrowed
/// it.
fn bounds(k: &SeatKnowledge) -> impl PartialEq {
    (k.hcp, k.len, k.balanced, k.pts, k.tp, k.dp)
}

/// Record what an announced text added to call `at` (`Position::heard`),
/// so the caches tell the position from one without it.
fn push_heard(pos: &mut Position, at: usize, what: &str) {
    if !pos.heard.is_empty() {
        pos.heard.push(' ');
    }
    pos.heard.push_str(&format!("{at}:{what}"));
}

fn force_rank(f: Forcing) -> u8 {
    match f {
        Forcing::None => 0,
        Forcing::Round => 1,
        Forcing::Game => 2,
    }
}

/// A condition in the rule language (`hcp=15..17`), as an expression.
fn condition(text: &str) -> Option<Expr> {
    let src = format!("module heard \"heard\"\n\nwhen opening\n  P \"x\" shows {text}\n");
    bidspec::parse(&src, "heard")
        .ok()?
        .contexts
        .first()?
        .rules
        .first()?
        .shows
        .clone()
}

impl Engine {
    /// The table of announced names this engine reads with (the rule
    /// set's: `announced-names.toml`).
    pub fn announced_names(&self) -> &meanings::AnnouncedNames {
        &self.rules.announced
    }

    /// Does `entry`'s context hold for `call` by `caller` at `pos`?
    fn name_applies(
        &self,
        entry: &meanings::NameEntry,
        pos: &Position,
        caller: Direction,
        call: &Call,
    ) -> bool {
        let params = crate::eval::Params::default();
        let ctx = Ctx {
            pos,
            actor: caller,
            hand: None,
            params: &params,
            valuation: self.valuation[side(caller)],
            private: None,
        };
        let mut b = Bindings::default();
        if let Some(alts) = &entry.after {
            if !match_any(alts, &pos.calls, &ctx, &mut b) {
                return false;
            }
        }
        entry.calls.is_empty()
            || entry
                .calls
                .iter()
                .any(|spec| match_call(spec, call, &ctx, &mut b.clone()))
    }

    fn plan(&self, dealer: Direction, calls: &[Call], heard: &Meanings) -> Plan {
        let mut plan = Plan {
            calls: vec![None; calls.len()],
            ..Plan::default()
        };
        let names = &self.rules.announced;
        let mut pos = Position::new(dealer, Vulnerability::None, ScoringMethod::IMP);
        for (i, call) in calls.iter().enumerate() {
            let caller = pos.next_caller();
            if let Some(text) = heard.for_call(i, caller) {
                let name = names
                    .candidates(text)
                    .into_iter()
                    .find(|(e, _)| self.name_applies(e, &pos, caller, call))
                    .map(|(e, n)| (n.to_string(), e.set.clone()));
                if let Some((_, sets)) = &name {
                    let changes = &mut plan.changes[side(caller)];
                    for s in sets {
                        if !changes.contains(s) {
                            changes.push(s.clone());
                        }
                    }
                }
                plan.calls[i] = Some(Planned {
                    text: text.to_string(),
                    name,
                });
            }
            pos.calls.push(call.clone());
        }
        plan
    }

    /// This engine with `changes` made to each side's card, made once and
    /// kept. A change the card refuses (a field this vocabulary lacks) is
    /// left out, with a warning.
    fn variant(&self, changes: &[Vec<String>; 2]) -> (Arc<Engine>, Vec<String>) {
        let key = format!("{changes:?}");
        let mut warnings = Vec::new();
        let mut cards = self.cards.clone();
        for (s, list) in changes.iter().enumerate() {
            for change in list {
                if let Err(e) = cards[s].apply_change(change) {
                    warnings.push(format!("announced name: {e}"));
                }
            }
        }
        if let Some(v) = self.variants.lock().unwrap().get(&key) {
            return (v.clone(), warnings);
        }
        let modules = &self.rules.modules;
        let system = |s: usize| {
            if changes[s].is_empty() {
                self.systems[s].clone()
            } else {
                System::new(&cards[s], modules)
            }
        };
        let v = Arc::new(Engine {
            systems: [system(0), system(1)],
            valuation: self.valuation,
            audience: self.audience,
            cards,
            rules: self.rules.clone(),
            variants: Mutex::default(),
            pool: self.pool.clone(),
            consistent: Mutex::default(),
            interned: Mutex::default(),
            descriptiveness: Mutex::default(),
            cands: Mutex::default(),
        });
        let mut variants = self.variants.lock().unwrap();
        if variants.len() >= VARIANT_LIMIT {
            variants.clear();
        }
        variants.insert(key, v.clone());
        (v, warnings)
    }

    fn reader(
        &self,
        dealer: Direction,
        calls: &[Call],
        heard: &Meanings,
    ) -> (Reader<'_>, Plan, Vec<String>) {
        let plan = self.plan(dealer, calls, heard);
        if plan.changes.iter().all(Vec::is_empty) {
            return (Reader::Own(self), plan, Vec::new());
        }
        let (v, warnings) = self.variant(&plan.changes);
        (Reader::Variant(v), plan, warnings)
    }

    /// `advance`, then what the call's announcement adds: for a call no
    /// rule read, what the text says it shows.
    fn advance_heard(&self, pos: &mut Position, call: &Call, planned: Option<&Planned>) -> Step {
        let mut step = self.advance(pos, call);
        let Some(planned) = planned else {
            return step;
        };
        let mut heard = Heard {
            text: planned.text.clone(),
            ..Heard::default()
        };
        if let Some((name, sets)) = &planned.name {
            heard.name = Some(name.clone());
            heard.sets = sets.clone();
        }
        let parsed = meanings::parse(&planned.text, Some(call));
        let caller = step.caller;
        let at = pos.calls.len() - 1;
        if step.rule.is_none() {
            // No rule read the call: what the text says is what it shows.
            let mut k = pos.knowledge(caller).clone();
            for c in parsed.conditions() {
                match condition(&c) {
                    Some(e) => {
                        if k.add(e) {
                            heard.shows.push(c);
                        } else {
                            step.warnings.push(format!(
                                "announced `{c}` contradicts what {caller:?} showed before"
                            ));
                        }
                    }
                    None => step
                        .warnings
                        .push(format!("announced `{c}`: not a condition")),
                }
            }
            if !heard.shows.is_empty() {
                pos.knowledge[caller.to_index()] = k.clone();
                step.knowledge = k;
                pos.apply_deck_hcp();
                // Read now: terms and fallbacks for a call no rule read
                // (`unread`) leave it alone (Rick, 2026-11-20).
                if let Some(u) = pos.unread.get_mut(at) {
                    *u = false;
                }
            }
            if parsed.artificial {
                heard.artificial = true;
                step.artificial = true;
                if let Some(a) = pos.artificial.get_mut(at) {
                    *a = true;
                }
            }
            if parsed.forcing != Forcing::None {
                heard.forcing = parsed.forcing;
                let st = &mut pos.sides[side(caller)];
                if force_rank(parsed.forcing) > force_rank(st.forcing) {
                    st.forcing = parsed.forcing;
                    st.forcing_by = Some(caller);
                }
            }
            if !parsed.is_empty() {
                heard.read = HeardRead::Unread;
                push_heard(pos, at, &parsed.summary());
                if step.announcement.is_empty() {
                    step.announcement = planned.text.clone();
                }
            }
        } else {
            // A rule read the call (Rick, 2026-11-20): the text narrows
            // that reading where the two agree; where any part contradicts
            // it, the rule's reading stands and the text is ignored (it is
            // likely describing another agreement). Its flags are the
            // rule's.
            let conds = parsed.conditions();
            let mut k = pos.knowledge(caller).clone();
            let mut narrowed = Vec::new();
            let mut contradicts = false;
            for c in conds.iter() {
                let Some(e) = condition(c) else {
                    step.warnings
                        .push(format!("announced `{c}`: not a condition"));
                    continue;
                };
                let mut t = k.clone();
                if !t.add(e) {
                    contradicts = true;
                    break;
                }
                if bounds(&t) != bounds(&k) {
                    k = t;
                    narrowed.push(c.clone());
                }
            }
            if contradicts {
                heard.read = HeardRead::Contradicts;
                step.warnings.push(format!(
                    "announced {:?} contradicts our reading of {call}: text ignored",
                    planned.text
                ));
            } else if !narrowed.is_empty() {
                heard.read = HeardRead::Narrowed;
                push_heard(pos, at, &narrowed.join(", "));
                pos.knowledge[caller.to_index()] = k.clone();
                step.knowledge = k;
                pos.apply_deck_hcp();
                heard.shows = narrowed;
            } else if !conds.is_empty() {
                heard.read = HeardRead::Agrees;
            }
        }
        step.heard = Some(heard);
        step
    }

    fn interpret_planned(
        &self,
        dealer: Direction,
        vul: Vulnerability,
        scoring: ScoringMethod,
        calls: &[Call],
        plan: &Plan,
    ) -> Interpretation {
        let mut pos = self.start(dealer, vul, scoring);
        let steps = calls
            .iter()
            .enumerate()
            .map(|(i, c)| {
                self.advance_heard(&mut pos, c, plan.calls.get(i).and_then(Option::as_ref))
            })
            .collect();
        Interpretation {
            steps,
            position: pos,
        }
    }

    /// `interpret`, with the opponents' announced meanings: a convention
    /// name the pack's table knows switches that field on in their card
    /// (their calls, all of them, are then read by our module for it),
    /// and a call no rule reads shows what its text says (lengths, HCP,
    /// balanced; artificial and forcing). Entries `heard` may not read
    /// (the listener's own and partner's) are ignored. A card change the
    /// card refuses is reported in the first step's warnings.
    pub fn interpret_with(
        &self,
        dealer: Direction,
        vul: Vulnerability,
        scoring: ScoringMethod,
        calls: &[Call],
        heard: &Meanings,
    ) -> Interpretation {
        if heard.is_empty() {
            return self.interpret(dealer, vul, scoring, calls);
        }
        let (reader, plan, warnings) = self.reader(dealer, calls, heard);
        let mut i = reader.interpret_planned(dealer, vul, scoring, calls, &plan);
        if let Some(first) = i.steps.first_mut() {
            first.warnings.extend(warnings);
        }
        i
    }

    /// `bid`, having heard the opponents' announced meanings (`texts`,
    /// aligned with `calls`; `""` where nothing was said): read as
    /// `interpret_with` reads them for the seat to call. Partner's entries
    /// are never read, whatever they hold.
    pub fn bid_with(
        &self,
        hand: &Hand,
        dealer: Direction,
        vul: Vulnerability,
        scoring: ScoringMethod,
        calls: &[Call],
        texts: &[String],
    ) -> Decision {
        let seat = Position::new(dealer, vul, scoring).caller(calls.len());
        let heard = Meanings::heard_by(seat, texts.to_vec());
        if heard.is_empty() {
            return self.bid(hand, dealer, vul, scoring, calls);
        }
        let (reader, plan, mut warnings) = self.reader(dealer, calls, &heard);
        let auction = reader.interpret_planned(dealer, vul, scoring, calls, &plan);
        let c = reader.choose(&auction.position, hand);
        warnings.extend(c.warnings);
        Decision {
            call: c.call,
            explanation: c.explanation,
            alert: c.alert,
            rule: c.rule,
            candidates: c.candidates,
            auction,
            warnings,
        }
    }

    /// `review` with the opponents' announced meanings, read as
    /// `interpret_with` reads them.
    pub fn review_with(
        &self,
        dealer: Direction,
        vul: Vulnerability,
        scoring: ScoringMethod,
        calls: &[Call],
        hands: &[Option<Hand>; 4],
        heard: &Meanings,
    ) -> Vec<crate::ReviewRow> {
        let (reader, plan, _) = self.reader(dealer, calls, heard);
        reader.review_by(
            dealer,
            vul,
            scoring,
            calls,
            hands,
            &mut |e, pos, i, call| {
                e.advance_heard(pos, call, plan.calls.get(i).and_then(Option::as_ref))
            },
        )
    }

    /// The rules that can make the next call after `calls` (`rules_at`),
    /// with the opponents' announced meanings read as `interpret_with`
    /// reads them.
    pub fn rules_at_with(
        &self,
        dealer: Direction,
        vul: Vulnerability,
        scoring: ScoringMethod,
        calls: &[Call],
        heard: &Meanings,
        all: bool,
    ) -> RuleListing {
        let (reader, plan, _) = self.reader(dealer, calls, heard);
        let i = reader.interpret_planned(dealer, vul, scoring, calls, &plan);
        reader.rules_at(&i.position, all)
    }
}
