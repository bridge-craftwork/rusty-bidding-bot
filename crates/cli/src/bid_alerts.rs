//! `rbb bid alerts`: each rule's `alert` / `announce` flag checked against
//! the pack's `alerts.toml` (bridge-common-conventions#132).
//!
//! For every entry the card gets the entry's fields turned on, the entry's
//! `after` is written out as concrete auctions, and the engine is asked
//! which rules make the entry's `call` there (`Engine::rules_at`, as `rbb
//! bid rules` asks). Each such rule's flag is compared with what the
//! regulator does. Read-only: it changes neither file.
//!
//! `after` and `call` are written for people (`1NT–2♣`, `(1x)`, `2♦/2♥`,
//! `New suit`). Calls are read; `–` separates them, `( )` marks the
//! opponents', `/` and ` or ` give alternatives, the opponents pass between
//! two calls of ours, `M` is a major, `m` a minor and `x`, `y`, `z` other
//! suits (all different). Prose (`suit agreed`, `Cuebid`) is left
//! unresolved, with the reason.

use std::collections::{BTreeMap, BTreeSet};

use bidspec::alerts::{AlertEntry, AlertRule, Alerts};
use bidspec::ast::{Alert, Context};
use bridge_card::{Card, FieldKind, Value, Vocabulary};
use bridge_types::{Auction, Call, Direction, ScoringMethod, Strain, Vulnerability};
use rbb_engine::{Engine, RuleSet};
use serde_json::{json, Value as Json};

/// A strain as written: a suit, or a variable for one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strn {
    Fixed(Strain),
    /// `M`, `m`, `x`, `y`, `z`, and `O` for the other major (`3 of other
    /// major`).
    Var(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pat {
    Pass,
    Double,
    Redouble,
    Bid(u8, Strn),
    /// `overcall`: a suit at the cheapest level.
    Cheapest(Strain),
}

/// One call of `after`: whose, and its alternatives.
#[derive(Debug, Clone, PartialEq)]
struct Seg {
    theirs: bool,
    alts: Vec<Pat>,
}

/// One written alternative as the calls it stands for: a call, or a
/// phrase (`overcall`, `3 of the major`, `4 of their suit`). `their` is
/// the variable of the opponents' suit, when `after` has one.
fn parse_alt(s: &str, their: Option<char>) -> Option<Vec<Pat>> {
    let s = s.trim();
    if s == "overcall" {
        return Some(SUITS.iter().map(|&x| Pat::Cheapest(x)).collect());
    }
    if let Some((level, rest)) = s.split_once(" of ") {
        let level = level.parse::<u8>().ok().filter(|l| (1..=7).contains(l))?;
        let v = match rest {
            "the major" => 'M',
            "other major" => 'O',
            "their suit" => their?,
            _ => return None,
        };
        return Some(vec![Pat::Bid(level, Strn::Var(v))]);
    }
    parse_pat(s).map(|p| vec![p])
}

fn parse_pat(s: &str) -> Option<Pat> {
    let s = s.trim();
    match s {
        "P" | "Pass" => return Some(Pat::Pass),
        "X" => return Some(Pat::Double),
        "XX" => return Some(Pat::Redouble),
        _ => {}
    }
    let mut chars = s.chars();
    let level = chars.next()?.to_digit(10).filter(|l| (1..=7).contains(l))? as u8;
    let strain = match chars.as_str() {
        "NT" | "N" => Strn::Fixed(Strain::NoTrump),
        "♣" | "C" => Strn::Fixed(Strain::Clubs),
        "♦" | "D" => Strn::Fixed(Strain::Diamonds),
        "♥" | "H" => Strn::Fixed(Strain::Hearts),
        "♠" | "S" => Strn::Fixed(Strain::Spades),
        v @ ("M" | "m" | "x" | "y" | "z") => Strn::Var(v.chars().next().unwrap()),
        _ => return None,
    };
    Some(Pat::Bid(level, strain))
}

/// Alternatives: `2♦/2♥`, `1♣ or 1♦`.
fn alternatives(s: &str) -> Vec<&str> {
    s.split('/')
        .flat_map(|p| p.split(" or "))
        .map(str::trim)
        .collect()
}

/// `after` as calls. `opening` is the empty auction; a trailing `(1st/2nd
/// seat)` or `(3rd/4th seat)` puts the auction in that seat.
fn parse_after(s: &str) -> Result<Vec<Seg>, String> {
    let mut s = s.trim();
    let mut segs = Vec::new();
    if let Some(rest) = s.strip_suffix("(3rd/4th seat)") {
        // Partner and the opponent on the right have passed.
        segs.push(Seg {
            theirs: false,
            alts: vec![Pat::Pass],
        });
        s = rest.trim();
    } else if let Some(rest) = s.strip_suffix("(1st/2nd seat)") {
        s = rest.trim();
    }
    if s == "opening" {
        return Ok(segs);
    }
    for part in s.split('–') {
        let part = part.trim();
        let (theirs, inner) = match part.strip_prefix('(').and_then(|p| p.strip_suffix(')')) {
            Some(inner) => (true, inner),
            None => (false, part),
        };
        let mut alts = Vec::new();
        for a in alternatives(inner) {
            alts.extend(parse_alt(a, None).ok_or_else(|| format!("after: {a:?} is not a call"))?);
        }
        segs.push(Seg { theirs, alts });
    }
    Ok(segs)
}

/// The call's alternatives. A qualifier in parentheses at the end
/// (`(natural)`, `(forced)`, `(three cards)`) is dropped.
fn parse_call(s: &str, after: &[Seg]) -> Result<Vec<Pat>, String> {
    let their = after
        .iter()
        .filter(|g| g.theirs)
        .flat_map(|g| &g.alts)
        .find_map(|p| match p {
            Pat::Bid(_, Strn::Var(v)) => Some(*v),
            _ => None,
        });
    let mut s = s.trim();
    if s.ends_with(')') {
        if let Some(i) = s.rfind(" (") {
            s = s[..i].trim();
        }
    }
    let mut out = Vec::new();
    for a in alternatives(s) {
        out.extend(parse_alt(a, their).ok_or_else(|| format!("call: {a:?} is not a call"))?);
    }
    Ok(out)
}

/// One concrete question: the calls so far and the call to look up.
#[derive(Debug, Clone, PartialEq)]
struct Probe {
    calls: Vec<Call>,
    call: Call,
}

const SUITS: [Strain; 4] = [
    Strain::Clubs,
    Strain::Diamonds,
    Strain::Hearts,
    Strain::Spades,
];

fn domain(v: char) -> Vec<Strain> {
    match v {
        'M' | 'O' => vec![Strain::Hearts, Strain::Spades],
        'm' => vec![Strain::Clubs, Strain::Diamonds],
        _ => SUITS.to_vec(),
    }
}

/// Every assignment of suits to `vars`, all different.
fn assignments(vars: &[char]) -> Vec<BTreeMap<char, Strain>> {
    let mut out = vec![BTreeMap::new()];
    for &v in vars {
        let mut next = Vec::new();
        for a in &out {
            for s in domain(v) {
                if !a.values().any(|t| *t == s) {
                    let mut b = a.clone();
                    b.insert(v, s);
                    next.push(b);
                }
            }
        }
        out = next;
    }
    out
}

fn rank(s: Strain) -> u8 {
    match s {
        Strain::Clubs => 0,
        Strain::Diamonds => 1,
        Strain::Hearts => 2,
        Strain::Spades => 3,
        Strain::NoTrump => 4,
    }
}

/// The call `p` stands for after `before`; `None` for an overcall in a
/// suit the other side has bid (a cuebid, not an overcall).
fn concrete(p: Pat, a: &BTreeMap<char, Strain>, before: &[Call]) -> Option<Call> {
    let n = before.len();
    Some(match p {
        Pat::Cheapest(strain) => {
            let theirs = before.iter().enumerate().any(|(j, c)| {
                (n - j) % 2 == 1 && matches!(c, Call::Bid { strain: s, .. } if *s == strain)
            });
            if theirs {
                return None;
            }
            let level = match before.iter().rev().find_map(|c| match c {
                Call::Bid { level, strain } => Some((*level, *strain)),
                _ => None,
            }) {
                None => 1,
                Some((l, s)) if rank(strain) > rank(s) => l,
                Some((l, _)) => l + 1,
            };
            Call::Bid { level, strain }
        }
        Pat::Pass => Call::Pass,
        Pat::Double => Call::Double,
        Pat::Redouble => Call::Redouble,
        Pat::Bid(level, Strn::Fixed(strain)) => Call::Bid { level, strain },
        Pat::Bid(level, Strn::Var(v)) => Call::Bid {
            level,
            strain: a[&v],
        },
    })
}

/// The calls before ours: the opponents pass between two calls of one
/// side, and after the last call of ours.
fn auction_calls(segs: &[Seg], pick: &[Pat], a: &BTreeMap<char, Strain>) -> Option<Vec<Call>> {
    let mut out = Vec::new();
    let mut last: Option<bool> = None;
    for (seg, p) in segs.iter().zip(pick) {
        if last == Some(seg.theirs) {
            out.push(Call::Pass);
        }
        let c = concrete(*p, a, &out)?;
        out.push(c);
        last = Some(seg.theirs);
    }
    if last == Some(false) {
        out.push(Call::Pass);
    }
    Some(out)
}

fn legal(calls: &[Call], call: &Call) -> bool {
    let mut auction = Auction::new(Direction::North);
    for c in calls {
        if auction.is_complete() || !auction.is_legal(c) {
            return false;
        }
        auction.add_call(c.clone());
    }
    !auction.is_complete() && auction.is_legal(call)
}

/// Every legal auction `after` and `call` stand for. When the call has as
/// many alternatives as a call of `after` (the last such), they go in
/// pairs: `1NT–2♦/2♥` then `2♥/2♠` is each transfer and its completion.
fn expand(segs: &[Seg], call: &[Pat]) -> Vec<Probe> {
    let mut vars: Vec<char> = Vec::new();
    for p in segs.iter().flat_map(|s| &s.alts).chain(call) {
        if let Pat::Bid(_, Strn::Var(v)) = p {
            if !vars.contains(v) {
                vars.push(*v);
            }
        }
    }
    let paired = (call.len() > 1)
        .then(|| segs.iter().rposition(|s| s.alts.len() == call.len()))
        .flatten();
    let mut picks: Vec<Vec<usize>> = vec![Vec::new()];
    for seg in segs {
        picks = picks
            .into_iter()
            .flat_map(|p| {
                (0..seg.alts.len()).map(move |i| {
                    let mut q = p.clone();
                    q.push(i);
                    q
                })
            })
            .collect();
    }
    let mut out: BTreeMap<String, Probe> = BTreeMap::new();
    for a in assignments(&vars) {
        for pick in &picks {
            let chosen: Vec<Pat> = segs.iter().zip(pick).map(|(s, &i)| s.alts[i]).collect();
            let Some(calls) = auction_calls(segs, &chosen, &a) else {
                continue;
            };
            let which: Vec<usize> = match paired {
                Some(k) => vec![pick[k]],
                None => (0..call.len()).collect(),
            };
            for i in which {
                let Some(c) = concrete(call[i], &a, &calls) else {
                    continue;
                };
                if legal(&calls, &c) {
                    out.insert(
                        format!("{} {}", pbn(&calls), c.to_pbn()),
                        Probe {
                            calls: calls.clone(),
                            call: c,
                        },
                    );
                }
            }
        }
    }
    out.into_values().collect()
}

/// Who is to call after `calls`: opener, responder, overcaller or
/// advancer.
fn role(calls: &[Call]) -> &'static str {
    let n = calls.len();
    let mine = |j: usize| (n - j) % 4 == 0;
    let ours = |j: usize| (n - j) % 2 == 0;
    match calls.iter().position(|c| matches!(c, Call::Bid { .. })) {
        None => "opener",
        Some(i) if mine(i) => "opener",
        Some(i) if ours(i) => "responder",
        Some(i) => match (i..n).find(|&j| ours(j) && calls[j] != Call::Pass) {
            Some(j) if !mine(j) => "advancer",
            _ => "overcaller",
        },
    }
}

fn kind(alert: &Option<Alert>) -> &'static str {
    match alert {
        Some(Alert::Alert { .. }) => "alert",
        Some(Alert::Announce { .. }) => "announce",
        None => "none",
    }
}

/// Whether a rule flagged `kind` does what the regulator asks. The rule
/// language has no delayed alert: an alerted rule stands for one.
fn agrees(expected: AlertRule, kind: &str) -> bool {
    match expected {
        AlertRule::Delayed => kind == "alert",
        e => e.as_str() == kind,
    }
}

/// `base` with the entry's fields on: a yes/no field set, a treatment's
/// convention with it, and what cannot be played beside it switched off
/// (the other fields of its choice group, other conventions filling the
/// same part of a function). Other fields keep the card's value. Returns
/// the changes made, `path=value`.
fn card_for(
    base: &Card,
    vocab: &Vocabulary,
    fields: &[String],
) -> Result<(Card, Vec<String>), String> {
    let reg = vocab.registry();
    let mut card = base.clone();
    let mut changes = Vec::new();
    let set = |card: &mut Card, path: &str, on: bool, changes: &mut Vec<String>| {
        if card.is_on(path) != on {
            card.set(path, Value::Bool(on)).map_err(|e| e.to_string())?;
            changes.push(format!("{path}={on}"));
        }
        Ok::<(), String>(())
    };
    for f in fields {
        let def = reg
            .get(f)
            .ok_or_else(|| format!("{f} is not a card field"))?;
        // A convention an enum's option plays (Ogust as the 2NT response
        // to a weak two): that option.
        for other in reg.fields() {
            if let Some((option, _)) = other.plays.iter().find(|(_, id)| *id == &def.path) {
                if card.effective(&other.path) != Some(&Value::Text(option.clone())) {
                    card.set(&other.path, Value::Text(option.clone()))
                        .map_err(|e| e.to_string())?;
                    changes.push(format!("{}={option}", other.path));
                }
            }
        }
        if def.kind != FieldKind::Bool {
            continue;
        }
        let mut on = vec![def.path.clone()];
        if let Some((id, _)) = def.path.split_once(':') {
            if reg.get(id).is_some_and(|d| d.kind == FieldKind::Bool) {
                on.push(id.to_string());
            }
        }
        for path in &on {
            if let Some(group) = &reg.get(path).and_then(|d| d.choice.clone()) {
                for other in reg
                    .fields()
                    .iter()
                    .filter(|o| o.choice.as_ref() == Some(group))
                {
                    if &other.path != path {
                        set(&mut card, &other.path, false, &mut changes)?;
                    }
                }
            }
        }
        if let Some(pack) = vocab.pack() {
            let id = def
                .path
                .split_once(':')
                .map_or(def.path.as_str(), |(i, _)| i);
            for func in &pack.functions {
                let Some((_, mine)) = func.implementers.iter().find(|(i, _)| i == id) else {
                    continue;
                };
                for (other, theirs) in &func.implementers {
                    let overlaps = mine.is_empty()
                        || theirs.is_empty()
                        || mine.iter().any(|p| theirs.contains(p));
                    if other != id && overlaps && reg.get(other).is_some() && card.is_on(other) {
                        set(&mut card, other, false, &mut changes)?;
                    }
                }
            }
        }
        for path in on.iter().rev() {
            set(&mut card, path, true, &mut changes)?;
        }
    }
    Ok((card, changes))
}

/// A rule that makes the entry's call in one of its auctions.
#[derive(Debug, Clone)]
pub struct Found {
    pub module: String,
    pub file: String,
    pub line: usize,
    pub call: String,
    pub explanation: String,
    pub kind: &'static str,
    pub artificial: bool,
    /// Partner reads the call by it (the call's top rule).
    pub read_as: bool,
    /// The best-ranked judged rule for the call in one of its auctions.
    pub decides: bool,
    pub agrees: bool,
    /// The auctions it was found after (PBN, space-separated).
    pub auctions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// Every rule found agrees.
    Agree,
    /// The rule partner reads the call by agrees, another rule for the
    /// same call does not.
    Mixed,
    /// The rule partner reads the call by does not agree.
    Differ,
    /// No rule to compare: the entry could not be read as auctions, or no
    /// rule makes the call there.
    Unresolved,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Agree => "agree",
            Status::Mixed => "mixed",
            Status::Differ => "differ",
            Status::Unresolved => "unresolved",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntryResult {
    pub entry: AlertEntry,
    pub expected: AlertRule,
    pub status: Status,
    /// Why it is unresolved; for one resolved, auctions where no rule
    /// makes the call.
    pub reason: Option<String>,
    pub card_changes: Vec<String>,
    pub probes: usize,
    /// A rule of the entry's own modules makes the call: only those were
    /// judged. Otherwise every rule that makes it was.
    pub own_rules: bool,
    pub found: Vec<Found>,
}

/// A rule that is alerted or artificial in a module no entry names.
#[derive(Debug, Clone)]
pub struct Unlisted {
    pub module: String,
    pub kind: Option<String>,
    pub file: String,
    pub line: usize,
    pub call: String,
    pub explanation: String,
    pub alert: &'static str,
    pub artificial: bool,
}

pub struct Report {
    pub regulator: String,
    pub entries: Vec<EntryResult>,
    pub unlisted: Vec<Unlisted>,
}

fn pbn(calls: &[Call]) -> String {
    if calls.is_empty() {
        return "(opening)".into();
    }
    calls.iter().map(Call::to_pbn).collect::<Vec<_>>().join(" ")
}

fn check_entry(rules: &RuleSet, base: &Card, e: &AlertEntry, regulator: &str) -> EntryResult {
    let expected = e.rule_for(regulator);
    let mut r = EntryResult {
        entry: e.clone(),
        expected,
        status: Status::Unresolved,
        reason: None,
        card_changes: Vec::new(),
        probes: 0,
        own_rules: false,
        found: Vec::new(),
    };
    let parsed = parse_after(&e.after).and_then(|a| {
        let c = parse_call(&e.call, &a)?;
        Ok((a, c))
    });
    let (segs, call) = match parsed {
        Ok(p) => p,
        Err(m) => {
            r.reason = Some(m);
            return r;
        }
    };
    let all = expand(&segs, &call);
    if all.is_empty() {
        r.reason = Some("no legal auction".into());
        return r;
    }
    let probes: Vec<Probe> = all
        .iter()
        .filter(|p| e.by == "either" || role(&p.calls) == e.by)
        .cloned()
        .collect();
    if probes.is_empty() {
        r.reason = Some(format!(
            "by = {:?}, but {} puts the {} to call",
            e.by,
            pbn(&all[0].calls),
            role(&all[0].calls)
        ));
        return r;
    }
    r.probes = probes.len();
    let (card, changes) = match card_for(base, &rules.vocab, &e.fields) {
        Ok(c) => c,
        Err(m) => {
            r.reason = Some(m);
            return r;
        }
    };
    r.card_changes = changes;
    let mine = entry_modules(rules, &e.fields);
    let engine = Engine::new(&card, &card, rules);
    // Each auction's rules for the call, best-ranked first.
    let mut asked: Vec<(String, Vec<Candidate>)> = Vec::new();
    for p in &probes {
        let interp = engine.interpret(
            Direction::North,
            Vulnerability::None,
            ScoringMethod::Matchpoints,
            &p.calls,
        );
        let listing = engine.rules_at(&interp.position, false);
        let pc = p.call.to_pbn();
        let list = listing
            .candidates
            .iter()
            .filter(|o| o.call == pc)
            .map(|o| Candidate {
                module: o.rule.module.clone(),
                file: o.rule.file.clone(),
                line: o.rule.line,
                call: pc.clone(),
                explanation: o.explanation.clone(),
                kind: kind(&o.alert),
                artificial: o.artificial,
                read_as: o.read_as,
                mine: mine.contains(&o.rule.module),
            })
            .collect();
        asked.push((format!("{} -> {pc}", pbn(&p.calls)), list));
    }
    // Where a rule of the entry's own modules makes the call, those rules
    // are the ones judged, and only in those auctions (a written
    // alternative may stand for auctions the convention does not have).
    r.own_rules = asked.iter().any(|(_, l)| l.iter().any(|o| o.mine));
    let mut found: Vec<Found> = Vec::new();
    let mut missing = Vec::new();
    for (auction, list) in &asked {
        let judged: Vec<&Candidate> = list.iter().filter(|o| !r.own_rules || o.mine).collect();
        if judged.is_empty() {
            if !r.own_rules {
                missing.push(auction.clone());
            }
            continue;
        }
        for (i, o) in judged.into_iter().enumerate() {
            match found
                .iter_mut()
                .find(|f| f.file == o.file && f.line == o.line)
            {
                Some(f) => {
                    f.read_as |= o.read_as;
                    f.decides |= i == 0;
                    f.auctions.push(auction.clone());
                }
                None => found.push(Found {
                    module: o.module.clone(),
                    file: o.file.clone(),
                    line: o.line,
                    call: o.call.clone(),
                    explanation: o.explanation.clone(),
                    kind: o.kind,
                    artificial: o.artificial,
                    read_as: o.read_as,
                    decides: i == 0,
                    agrees: agrees(expected, o.kind),
                    auctions: vec![auction.clone()],
                }),
            }
        }
    }
    if found.is_empty() {
        r.reason = Some(format!("no rule makes the call: {}", missing.join("; ")));
        return r;
    }
    r.status = if found.iter().filter(|f| f.decides).any(|f| !f.agrees) {
        Status::Differ
    } else if found.iter().any(|f| !f.agrees) {
        Status::Mixed
    } else {
        Status::Agree
    };
    if !missing.is_empty() {
        r.reason = Some(format!("no rule makes the call: {}", missing.join("; ")));
    }
    r.found = found;
    r
}

/// A rule for the call in one auction.
struct Candidate {
    module: String,
    file: String,
    line: usize,
    call: String,
    explanation: String,
    kind: &'static str,
    artificial: bool,
    read_as: bool,
    /// Its module belongs to the entry's fields.
    mine: bool,
}

/// The modules an entry's fields belong to: the convention a module is
/// for (`convention` line, a `skill` line), or a field a `card` line or a
/// `param` reads (or an enum whose option plays the convention).
fn entry_modules(rules: &RuleSet, fields: &[String]) -> BTreeSet<String> {
    let reg = rules.vocab.registry();
    let canonical = |p: &str| reg.get(p).map_or_else(|| p.to_string(), |d| d.path.clone());
    let fields: Vec<String> = fields.iter().map(|f| canonical(f)).collect();
    let ids: Vec<&str> = fields
        .iter()
        .map(|f| f.split_once(':').map_or(f.as_str(), |(id, _)| id))
        .collect();
    rules
        .modules
        .iter()
        .filter(|m| {
            m.convention
                .as_deref()
                .is_some_and(|c| ids.contains(&canonical(c).as_str()))
                || m.card.iter().any(|c| {
                    let p = canonical(&c.path);
                    fields.contains(&p) || ids.contains(&p.as_str())
                })
                || m.skills.iter().any(|k| ids.contains(&k.path.as_str()))
                || m.params.iter().any(|p| {
                    fields.contains(&canonical(&p.path))
                        || reg
                            .get(&p.path)
                            .is_some_and(|d| d.plays.values().any(|id| ids.contains(&id.as_str())))
                })
        })
        .map(|m| m.name.clone())
        .collect()
}

/// The canonical paths and convention IDs `alerts` names.
fn covered(alerts: &Alerts, vocab: &Vocabulary) -> BTreeSet<String> {
    let reg = vocab.registry();
    let mut out = BTreeSet::new();
    for f in alerts.entries.iter().flat_map(|e| &e.fields) {
        let path = reg.get(f).map_or_else(|| f.clone(), |d| d.path.clone());
        if let Some((id, _)) = path.split_once(':') {
            out.insert(id.to_string());
        }
        out.insert(path);
    }
    out
}

/// Alerted or artificial rules in modules none of whose fields
/// (`convention` line, `card` lines, `skill` lines) an entry names.
fn unlisted(rules: &RuleSet, alerts: &Alerts) -> Vec<Unlisted> {
    let reg = rules.vocab.registry();
    let covered = covered(alerts, &rules.vocab);
    let canonical = |p: &str| reg.get(p).map_or_else(|| p.to_string(), |d| d.path.clone());
    let mut out = Vec::new();
    for m in &rules.modules {
        let fields: Vec<String> = m
            .convention
            .iter()
            .map(|c| canonical(c))
            .chain(m.card.iter().map(|c| canonical(&c.path)))
            .chain(m.skills.iter().map(|k| k.path.clone()))
            .collect();
        let named = fields.iter().any(|f| {
            covered.contains(f)
                || f.split_once(':')
                    .is_some_and(|(id, _)| covered.contains(id))
        });
        if named {
            continue;
        }
        fn walk<'a>(c: &'a Context, out: &mut Vec<&'a bidspec::ast::Rule>) {
            out.extend(c.rules.iter());
            for sub in &c.contexts {
                walk(sub, out);
            }
        }
        let mut list = Vec::new();
        for c in &m.contexts {
            walk(c, &mut list);
        }
        for rule in list {
            if rule.alert.is_some() || rule.artificial {
                out.push(Unlisted {
                    module: m.name.clone(),
                    kind: m.kind.map(|k| k.as_str().to_string()),
                    file: m.file.clone(),
                    line: rule.line,
                    call: rule.call.to_string(),
                    explanation: rule.explanation.clone(),
                    alert: kind(&rule.alert),
                    artificial: rule.artificial,
                });
            }
        }
    }
    out
}

/// Check every entry of `alerts` against `rules`, on `base` with each
/// entry's fields turned on, for `regulator` (`acbl`, `ebu`, `wbf`).
pub fn check(rules: &RuleSet, base: &Card, alerts: &Alerts, regulator: &str) -> Report {
    Report {
        regulator: regulator.to_string(),
        entries: alerts
            .entries
            .iter()
            .map(|e| check_entry(rules, base, e, regulator))
            .collect(),
        unlisted: unlisted(rules, alerts),
    }
}

impl Report {
    /// Rule files named from `root` (the `--rules` directory) on.
    pub fn relative_to(&mut self, root: &str) {
        let root = format!("{}/", root.trim_end_matches('/'));
        let strip = |f: &mut String| {
            if let Some(rest) = f.strip_prefix(&root) {
                *f = rest.to_string();
            }
        };
        for r in &mut self.entries {
            r.found.iter_mut().for_each(|f| strip(&mut f.file));
        }
        self.unlisted.iter_mut().for_each(|u| strip(&mut u.file));
    }

    /// Entries that are not variants (`when`), by status.
    pub fn count(&self, s: Status, variants: bool) -> usize {
        self.entries
            .iter()
            .filter(|r| r.status == s && r.entry.when.is_some() == variants)
            .count()
    }

    /// Entries that are not variants and do not agree (`--strict`).
    pub fn failures(&self) -> usize {
        self.count(Status::Differ, false) + self.count(Status::Mixed, false)
    }

    pub fn to_json(&self) -> Json {
        let entries: Vec<Json> = self
            .entries
            .iter()
            .map(|r| {
                json!({
                    "index": r.entry.index,
                    "fields": r.entry.fields,
                    "call": r.entry.call,
                    "after": r.entry.after,
                    "by": r.entry.by,
                    "when": r.entry.when,
                    "expected": r.expected.as_str(),
                    "status": r.status.as_str(),
                    "reason": r.reason,
                    "card_changes": r.card_changes,
                    "auctions": r.probes,
                    "own_rules": r.own_rules,
                    "rules": r.found.iter().map(|f| json!({
                        "module": f.module,
                        "file": f.file,
                        "line": f.line,
                        "call": f.call,
                        "explanation": f.explanation,
                        "alert": f.kind,
                        "artificial": f.artificial,
                        "read_as": f.read_as,
                        "decides": f.decides,
                        "agrees": f.agrees,
                        "auctions": f.auctions,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let summary = |variants: bool| {
            json!({
                "agree": self.count(Status::Agree, variants),
                "mixed": self.count(Status::Mixed, variants),
                "differ": self.count(Status::Differ, variants),
                "unresolved": self.count(Status::Unresolved, variants),
            })
        };
        json!({
            "regulator": self.regulator,
            "summary": summary(false),
            "variants": summary(true),
            "entries": entries,
            "unlisted": self.unlisted.iter().map(|u| json!({
                "module": u.module,
                "kind": u.kind,
                "file": u.file,
                "line": u.line,
                "call": u.call,
                "explanation": u.explanation,
                "alert": u.alert,
                "artificial": u.artificial,
            })).collect::<Vec<_>>(),
        })
    }

    pub fn to_text(&self, verbose: bool) -> String {
        use std::fmt::Write as _;
        let mut s = String::new();
        let reg = self.regulator.to_uppercase();
        let _ = writeln!(
            s,
            "alerts.toml against the rules' alert flags ({reg}): {} entries",
            self.entries.len()
        );
        let line = |v: bool| {
            format!(
                "agree {}, mixed {}, differ {}, unresolved {}",
                self.count(Status::Agree, v),
                self.count(Status::Mixed, v),
                self.count(Status::Differ, v),
                self.count(Status::Unresolved, v)
            )
        };
        let _ = writeln!(s, "  entries: {}", line(false));
        let _ = writeln!(s, "  variants (`when`, informational): {}", line(true));
        let head = |r: &EntryResult| {
            format!(
                "#{} {} {} after {} by {}: {reg} {}{}{}",
                r.entry.index,
                r.entry.fields.join(", "),
                r.entry.call,
                r.entry.after,
                r.entry.by,
                r.expected.as_str(),
                r.entry
                    .when
                    .as_ref()
                    .map(|w| format!(" (when {w})"))
                    .unwrap_or_default(),
                if r.own_rules || r.found.is_empty() {
                    ""
                } else {
                    " [no rule of the entry's modules makes it: judged on those that do]"
                }
            )
        };
        let rule_line = |f: &Found| {
            format!(
                "      {} {:8} {}:{} {} \"{}\"{}{}  [{}]",
                if f.agrees { "ok" } else { "!=" },
                f.kind,
                f.file,
                f.line,
                f.call,
                f.explanation,
                if f.artificial { " artificial" } else { "" },
                if f.read_as {
                    ""
                } else {
                    " (partner reads the call by another rule)"
                },
                f.auctions.first().map(String::as_str).unwrap_or_default()
            )
        };
        for status in [Status::Differ, Status::Mixed] {
            // Grouped by what the entry asks and what the rules say.
            let mut groups: BTreeMap<String, Vec<&EntryResult>> = BTreeMap::new();
            for r in self.entries.iter().filter(|r| r.status == status) {
                let got: BTreeSet<&str> = r
                    .found
                    .iter()
                    .filter(|f| !f.agrees && (f.decides || status == Status::Mixed))
                    .map(|f| f.kind)
                    .collect();
                let got: Vec<&str> = got.into_iter().collect();
                groups
                    .entry(format!(
                        "{reg} {}, rule {}",
                        r.expected.as_str(),
                        got.join("/")
                    ))
                    .or_default()
                    .push(r);
            }
            if groups.is_empty() {
                continue;
            }
            let _ = writeln!(s, "\n{}:", match status {
                Status::Differ => "Differ (the best-ranked rule for the call, in at least one auction)",
                _ => "Mixed (the best-ranked rules agree, a lower one for the same call does not)",
            });
            for (g, list) in groups {
                let _ = writeln!(s, "  {g} ({}):", list.len());
                for r in list {
                    let _ = writeln!(s, "    {}", head(r));
                    for f in &r.found {
                        if verbose || !f.agrees || f.decides {
                            let _ = writeln!(s, "{}", rule_line(f));
                        }
                    }
                }
            }
        }
        let unresolved: Vec<&EntryResult> = self
            .entries
            .iter()
            .filter(|r| r.status == Status::Unresolved)
            .collect();
        if !unresolved.is_empty() {
            let _ = writeln!(s, "\nUnresolved ({}):", unresolved.len());
            for r in unresolved {
                let _ = writeln!(
                    s,
                    "    {}\n      {}",
                    head(r),
                    r.reason.as_deref().unwrap_or_default()
                );
            }
        }
        let _ = writeln!(
            s,
            "\nAgree ({}):",
            self.count(Status::Agree, false) + self.count(Status::Agree, true)
        );
        for r in self.entries.iter().filter(|r| r.status == Status::Agree) {
            let _ = writeln!(s, "    {}", head(r));
            if verbose {
                for f in &r.found {
                    let _ = writeln!(s, "{}", rule_line(f));
                }
            }
        }
        let mut by_module: BTreeMap<&str, Vec<&Unlisted>> = BTreeMap::new();
        for u in &self.unlisted {
            by_module.entry(&u.module).or_default().push(u);
        }
        let _ = writeln!(
            s,
            "\nAlerted or artificial rules in modules no entry names: {} rules in {} modules",
            self.unlisted.len(),
            by_module.len()
        );
        for (m, list) in by_module {
            let alerted = list.iter().filter(|u| u.alert != "none").count();
            let _ = writeln!(
                s,
                "  {m} ({}, {}): {} rules, {alerted} alerted or announced",
                list[0].kind.as_deref().unwrap_or("no kind"),
                list[0].file,
                list.len()
            );
            if verbose {
                for u in list {
                    let _ = writeln!(
                        s,
                        "      {:8} {}:{} {} \"{}\"{}",
                        u.alert,
                        u.file,
                        u.line,
                        u.call,
                        u.explanation,
                        if u.artificial { " artificial" } else { "" }
                    );
                }
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELDS: &str = r#"
[stayman]
"play" = { kind = "bool", label = "Stayman" }
[transfers]
"play" = { kind = "bool", label = "Transfers" }
[other]
"play" = { kind = "bool", label = "Other" }
"#;

    const BASE: &str = r#"
module base "Base"
  kind system

when opening
  1N "15-17"  announce "15 to 17"  shows hcp>=15, hcp<=17, balanced
  1H "Hearts"  shows H>=5, hcp>=12
  1S "Spades"  shows S>=5, hcp>=12
  P  "Pass"

after 1N (P)
  P  "Pass"
  2C "Clubs"  shows C>=6

after (1x)
  X  "Takeout"  alert  shows hcp>=12  artificial
  P  "Pass"
"#;

    const STAYMAN: &str = r#"
module stayman "Stayman"
  kind convention
  card stayman.play

after 1N (P)
  2C  "Stayman"  alert  shows hcp>=8  artificial
"#;

    const TRANSFERS: &str = r#"
module transfers "Transfers"
  kind convention
  card transfers.play

after 1N (P)
  2D  "Transfer"  announce "Transfer"  shows H>=5  artificial
  2H  "Transfer"  alert  shows S>=5  artificial

after 1N (P) 2D (P)
  2H  "Completes"
after 1N (P) 2H (P)
  2S  "Completes"
"#;

    const OTHER: &str = r#"
module other "Other"
  kind convention
  card other.play

after 1N (P)
  3C  "Puppet"  alert  artificial
"#;

    fn rules() -> RuleSet {
        rbb_engine::compile_rules(
            None,
            ("fields.toml", FIELDS),
            ("bbsa-map.toml", ""),
            [
                ("base.bid", BASE),
                ("stayman.bid", STAYMAN),
                ("transfers.bid", TRANSFERS),
                ("other.bid", OTHER),
            ],
        )
        .unwrap()
    }

    const ALERTS: &str = r#"
[regulators.acbl]
name = "ACBL"
[regulators.wbf]
name = "WBF"

[[alert]]
fields = ["stayman.play"]
call = "2♣"
after = "1NT"
by = "responder"
rule = "none"
wbf = "alert"

[[alert]]
fields = ["transfers.play"]
call = "2♦/2♥"
after = "1NT"
by = "responder"
rule = "announce"

[[alert]]
fields = ["transfers.play"]
call = "2♥/2♠"
after = "1NT–2♦/2♥"
by = "opener"
rule = "none"

[[alert]]
fields = ["stayman.play"]
call = "Cuebid"
after = "1NT"
by = "responder"
rule = "alert"

[[alert]]
fields = ["stayman.play"]
call = "3♠"
after = "1NT"
by = "responder"
rule = "alert"

[[alert]]
fields = ["stayman.play"]
call = "2♣"
after = "1NT"
by = "opener"
rule = "none"

[[alert]]
fields = ["no.such"]
call = "2♣"
after = "1NT"
by = "responder"
rule = "none"
"#;

    fn report(regulator: &str) -> Report {
        let rules = rules();
        let base = Card::new(&rules.vocab);
        let alerts = bidspec::alerts::parse(ALERTS).unwrap();
        check(&rules, &base, &alerts, regulator)
    }

    #[test]
    fn reads_the_written_auctions() {
        let segs = parse_after("1NT–(2x)").unwrap();
        assert_eq!(segs.len(), 2);
        assert!(segs[1].theirs);
        assert_eq!(parse_after("opening").unwrap(), Vec::new());
        assert_eq!(parse_after("1♥/1♠ (3rd/4th seat)").unwrap().len(), 2);
        assert_eq!(parse_call("2♣/2♦ (three cards)", &[]).unwrap().len(), 2);
        assert_eq!(parse_call("1♣ or 1♦", &[]).unwrap().len(), 2);
        assert!(parse_after("suit agreed").is_err());
        assert!(parse_call("New suit", &[]).is_err());
        assert_eq!(parse_after("1x–(X or overcall)").unwrap()[1].alts.len(), 5);
        assert!(parse_call("3 of their suit", &[]).is_err());
    }

    #[test]
    fn expands_with_passes_variables_and_pairs() {
        let p = |a: &str, c: &str| {
            let segs = parse_after(a).unwrap();
            expand(&segs, &parse_call(c, &segs).unwrap())
        };
        let t = |calls: &[Call]| pbn(calls);
        let probes = p("1NT–2♦/2♥", "2♥/2♠");
        let got: Vec<String> = probes
            .iter()
            .map(|p| format!("{} {}", t(&p.calls), p.call.to_pbn()))
            .collect();
        assert_eq!(got, ["1NT Pass 2D Pass 2H", "1NT Pass 2H Pass 2S"]);
        // The opponents' calls go in as written: no pass before ours.
        let probes = p("(1x)", "X");
        assert_eq!(probes.len(), 4);
        assert!(probes
            .iter()
            .all(|p| p.calls.len() == 1 && role(&p.calls) == "overcaller"));
        // Different suits, and only legal auctions.
        let probes = p("1x–1y", "1NT");
        assert!(probes.iter().all(|p| p.calls[0] != p.calls[2]));
        assert_eq!(probes.len(), 6);
        // Third seat.
        let probes = p("1♥/1♠ (3rd/4th seat)", "2♣");
        assert_eq!(t(&probes[0].calls), "Pass Pass 1H Pass");
        assert_eq!(role(&probes[0].calls), "responder");
        assert_eq!(role(&p("(2x)–X–(P)", "2NT")[0].calls), "advancer");
        // Phrases: the major, the other major, their suit, an overcall.
        let got: Vec<String> = p("1M–2NT", "3 of other major")
            .iter()
            .map(|p| format!("{} {}", t(&p.calls), p.call.to_pbn()))
            .collect();
        assert_eq!(got, ["1H Pass 2NT Pass 3S", "1S Pass 2NT Pass 3H"]);
        let got: Vec<String> = p("1NT–(2x)", "3 of their suit")
            .iter()
            .map(|p| p.call.to_pbn())
            .collect();
        assert_eq!(got, ["3C", "3D", "3H", "3S"]);
        let got: Vec<String> = p("1♥–(overcall)", "X")
            .iter()
            .map(|p| t(&p.calls))
            .collect();
        assert_eq!(got, ["1H 1S", "1H 2C", "1H 2D"]);
    }

    #[test]
    fn compares_each_entry_with_the_rules() {
        let r = report("acbl");
        let status: Vec<Status> = r.entries.iter().map(|e| e.status).collect();
        assert_eq!(
            status,
            [
                // Stayman is alerted in the rule; the ACBL exempts it.
                Status::Differ,
                // 2♦ announced (agrees), 2♥ alerted (does not).
                Status::Differ,
                Status::Agree,
                Status::Unresolved,
                Status::Unresolved,
                Status::Unresolved,
                Status::Unresolved,
            ]
        );
        // Judged on the rules of the entry's own module only.
        let modules: Vec<&str> = r.entries[0]
            .found
            .iter()
            .map(|f| f.module.as_str())
            .collect();
        assert_eq!(modules, ["stayman"]);
        assert!(r.entries[0].own_rules);
        let transfers = &r.entries[1];
        assert_eq!(transfers.card_changes, ["transfers.play=true"]);
        let by_call: BTreeMap<&str, bool> = transfers
            .found
            .iter()
            .map(|f| (f.call.as_str(), f.agrees))
            .collect();
        assert_eq!(by_call, BTreeMap::from([("2D", true), ("2H", false)]));
        assert!(r.entries[3]
            .reason
            .as_deref()
            .unwrap()
            .contains("\"Cuebid\""));
        assert!(r.entries[4]
            .reason
            .as_deref()
            .unwrap()
            .contains("no rule makes the call"));
        assert!(r.entries[5]
            .reason
            .as_deref()
            .unwrap()
            .contains("puts the responder"));
        assert!(r.entries[6]
            .reason
            .as_deref()
            .unwrap()
            .contains("not a card field"));
        // The WBF alerts Stayman, as the rule does.
        assert_eq!(report("wbf").entries[0].status, Status::Agree);
        assert_eq!(r.failures(), 2);
    }

    #[test]
    fn lists_alerted_rules_in_modules_no_entry_names() {
        let r = report("acbl");
        let got: Vec<(&str, &str, &str)> = r
            .unlisted
            .iter()
            .map(|u| (u.module.as_str(), u.call.as_str(), u.alert))
            .collect();
        // Base has no field; other's field has no entry.
        assert_eq!(
            got,
            [
                ("base", "1N", "announce"),
                ("base", "X", "alert"),
                ("other", "3C", "alert"),
            ]
        );
        let text = r.to_text(false);
        assert!(text.contains("differ 2, unresolved 4"), "{text}");
        assert_eq!(r.to_json()["summary"]["agree"], 1);
    }
}
