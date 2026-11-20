//! Whether a call is alerted, announced or neither: the pack's
//! `alerts.toml` read for a regulator (bridge-common-conventions#132).
//!
//! A rule's `alert` / `announce` flag says what its call is; which calls a
//! table alerts is the regulator's business (the ACBL exempts Stayman, the
//! EBU announces it, the WBF alerts it), and one flag cannot say all
//! three. `alerts.toml` does, per card field. At load each entry is
//! written out as the concrete auctions it stands for (`1NT–2♦/2♥` then
//! `2♥/2♠` is `1NT P 2D P 2H` and `1NT P 2H P 2S`), indexed by auction;
//! at call time [`AlertTable::decide`] finds the entries for the auction
//! so far and the call, keeps those that belong to the rule making it,
//! and gives the regulator's kind. Where none does, the rule's own flag
//! stands (`Engine` falls back).
//!
//! The entries are written for people: `–` separates calls, `( )` marks
//! the opponents', `/` and ` or ` give alternatives, the opponents pass
//! between two calls of ours, `M` is a major, `m` a minor and `x`, `y`, `z`
//! other suits (all different). Prose (`suit agreed`, `Cuebid`) stays
//! unresolved, with the reason. `rbb bid alerts` reports both.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use bidspec::alerts::{AlertEntry, AlertRule, Alerts};
use bridge_card::{Card, FieldKind, Value, Vocabulary};
use bridge_types::{Auction, Call, Direction, Strain};
use serde::{Deserialize, Serialize};

/// Whose alerting rules a table follows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Regulator {
    /// The ACBL Alert Procedure.
    #[default]
    Acbl,
    /// The EBU Blue Book.
    Ebu,
    /// The WBF Alerting Policy.
    Wbf,
}

impl Regulator {
    pub const ALL: [Regulator; 3] = [Regulator::Acbl, Regulator::Ebu, Regulator::Wbf];

    /// Its key in `alerts.toml` (`acbl`).
    pub fn key(self) -> &'static str {
        match self {
            Regulator::Acbl => "acbl",
            Regulator::Ebu => "ebu",
            Regulator::Wbf => "wbf",
        }
    }
}

impl std::fmt::Display for Regulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

impl std::str::FromStr for Regulator {
    type Err = String;
    fn from_str(s: &str) -> Result<Regulator, String> {
        match s.to_ascii_lowercase().as_str() {
            "acbl" => Ok(Regulator::Acbl),
            "ebu" => Ok(Regulator::Ebu),
            "wbf" => Ok(Regulator::Wbf),
            _ => Err(format!("regulator must be acbl, ebu or wbf, not {s:?}")),
        }
    }
}

// ---------------------------------------------------------------------------
// The written auctions as concrete ones.

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

/// The seat an `after` names: `(1st/2nd seat)`, `(3rd/4th seat)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seat {
    Any,
    Early,
    Late,
}

const SUITS: [Strain; 4] = [
    Strain::Clubs,
    Strain::Diamonds,
    Strain::Hearts,
    Strain::Spades,
];

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
fn parse_after(s: &str) -> Result<(Vec<Seg>, Seat), String> {
    let mut s = s.trim();
    let mut segs = Vec::new();
    let mut seat = Seat::Any;
    if let Some(rest) = s.strip_suffix("(3rd/4th seat)") {
        // Partner and the opponent on the right have passed.
        segs.push(Seg {
            theirs: false,
            alts: vec![Pat::Pass],
        });
        s = rest.trim();
        seat = Seat::Late;
    } else if let Some(rest) = s.strip_suffix("(1st/2nd seat)") {
        s = rest.trim();
        seat = Seat::Early;
    }
    if s == "opening" {
        return Ok((segs, seat));
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
    Ok((segs, seat))
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

/// One concrete auction an entry stands for: the calls so far (from the
/// dealer) and the call it is about.
#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    pub calls: Vec<Call>,
    pub call: Call,
}

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
pub fn role(calls: &[Call]) -> &'static str {
    let n = calls.len();
    let mine = |j: usize| (n - j).is_multiple_of(4);
    let ours = |j: usize| (n - j).is_multiple_of(2);
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

/// The calls in PBN, `(opening)` for none.
pub fn pbn(calls: &[Call]) -> String {
    if calls.is_empty() {
        return "(opening)".into();
    }
    calls.iter().map(Call::to_pbn).collect::<Vec<_>>().join(" ")
}

/// The concrete auctions `e` stands for, its `by` applied; `Err` says why
/// there are none (prose, no legal auction, the wrong caller).
pub fn entry_probes(e: &AlertEntry) -> Result<Vec<Probe>, String> {
    entry_auctions(e).map(|(p, _)| p)
}

fn entry_auctions(e: &AlertEntry) -> Result<(Vec<Probe>, Seat), String> {
    let (segs, seat) = parse_after(&e.after)?;
    let call = parse_call(&e.call, &segs)?;
    let all = expand(&segs, &call);
    if all.is_empty() {
        return Err("no legal auction".into());
    }
    let probes: Vec<Probe> = all
        .iter()
        .filter(|p| e.by == "either" || role(&p.calls) == e.by)
        .cloned()
        .collect();
    if probes.is_empty() {
        return Err(format!(
            "by = {:?}, but {} puts the {} to call",
            e.by,
            pbn(&all[0].calls),
            role(&all[0].calls)
        ));
    }
    Ok((probes, seat))
}

// ---------------------------------------------------------------------------
// Which rules an entry is about.

/// The modules an entry's fields belong to: the convention a module is
/// for (`convention` line, a `skill` line), or a field a `card` line or a
/// `param` reads (or an enum whose option plays the convention).
pub fn entry_modules(
    modules: &[bidspec::Module],
    vocab: &Vocabulary,
    fields: &[String],
) -> BTreeSet<String> {
    let reg = vocab.registry();
    let canonical = |p: &str| reg.get(p).map_or_else(|| p.to_string(), |d| d.path.clone());
    let fields: Vec<String> = fields.iter().map(|f| canonical(f)).collect();
    let ids: Vec<&str> = fields
        .iter()
        .map(|f| f.split_once(':').map_or(f.as_str(), |(id, _)| id))
        .collect();
    modules
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

/// How an entry's fields are told to be on a card. Where entries of more
/// than one kind apply to a call, the first kind wins: a convention the
/// card switches on says more about the call than a setting does (Two
/// Hearts Bust's 2♥ over the 2♣ entry's natural responses).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FieldsAre {
    /// Yes/no fields (or enum options playing a convention): the entry
    /// applies when the card plays one.
    Played,
    /// Enum fields with named options (`two_level.two_clubs.meaning`): the
    /// entry is for the field's calls whatever its value.
    Chosen,
    /// Numbers, text and enums of numbers (`one_club.min_length`, `2`
    /// to `5`): the entry's condition on the value is prose ("may be
    /// short"), so it decides how a call the rule flags is alerted, never
    /// whether an unflagged one is.
    Setting,
}

/// Whether `card` plays `path` (canonical): a yes/no field that is on, or
/// an enum holding an option that plays it.
fn plays(card: &Card, path: &str) -> bool {
    if card.is_on(path) {
        return true;
    }
    card.vocabulary().registry().fields().iter().any(|f| {
        matches!(card.effective(&f.path), Some(Value::Text(v))
            if f.plays.get(v).is_some_and(|p| p == path))
    })
}

/// One entry as the engine reads it.
#[derive(Debug, Clone)]
struct Resolved {
    /// Index into `Alerts::entries`.
    entry: usize,
    /// Canonical paths of its fields that are card fields.
    fields: Vec<String>,
    are: FieldsAre,
    modules: BTreeSet<String>,
    /// How many concrete auctions it stands for: of two entries that
    /// disagree about a call, the one written for fewer is the more
    /// specific (`1♣/1♦–(1♥/1♠)` before `1x–(overcall)`).
    breadth: usize,
}

/// An index key's seat condition: the passes before the first bid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lead {
    Any,
    /// Opened in first or second seat (at most one pass before).
    Early,
    /// At least two passes before the first bid (third or fourth seat, a
    /// passed hand's response).
    Late,
}

impl Lead {
    fn holds(self, passes: usize) -> bool {
        match self {
            Lead::Any => true,
            Lead::Early => passes <= 1,
            Lead::Late => passes >= 2,
        }
    }
}

/// The calls before the first bid, double or redouble, and the key of
/// the rest with `call`.
fn key(calls: &[Call], call: &Call) -> (usize, String) {
    let lead = calls.iter().take_while(|c| **c == Call::Pass).count();
    let mut k = String::new();
    for c in &calls[lead..] {
        k.push_str(&c.to_pbn());
        k.push(' ');
    }
    k.push_str("-> ");
    k.push_str(&call.to_pbn());
    (lead, k)
}

/// Why no entry decided a call: the rule's own flag stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum Fallback {
    /// No entry is written for this auction and call.
    NoEntry,
    /// Only variants (`when`): whether the partnership plays it so is not
    /// on the card.
    Variant,
    /// The card plays none of the entries' fields.
    NotPlayed,
    /// The entries belong to other modules, one of which makes the call
    /// here too.
    OtherModule,
    /// A setting's entry (`FieldsAre::Setting`) and the rule does not flag
    /// the call.
    Setting,
    /// Entries that apply disagree for this regulator (their indices).
    Ambiguous { entries: Vec<usize> },
}

impl Fallback {
    pub fn as_str(&self) -> &'static str {
        match self {
            Fallback::NoEntry => "no entry",
            Fallback::Variant => "only variants (`when`)",
            Fallback::NotPlayed => "the card plays none of the entry's fields",
            Fallback::OtherModule => "the entry belongs to another module that makes the call",
            Fallback::Setting => "a setting's entry, and the rule does not flag the call",
            Fallback::Ambiguous { .. } => "entries disagree",
        }
    }
}

/// What an entry decided: its 1-based index in `alerts.toml` and the
/// regulator's kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Decided {
    pub entry: usize,
    pub rule: AlertRule,
}

/// `alerts.toml` as the engine reads it: the entries, and for each
/// concrete auction the entries written for it.
#[derive(Debug, Clone, Default)]
pub struct AlertTable {
    pub alerts: Alerts,
    resolved: Vec<Resolved>,
    /// Auction key (`key`) to (index into `resolved`, seat condition).
    index: HashMap<String, Vec<(usize, Lead)>>,
    /// Entries (1-based) that stand for no concrete auction, and why.
    pub unresolved: Vec<(usize, String)>,
}

impl AlertTable {
    /// Read `alerts` against a rule set's modules and vocabulary.
    pub fn new(alerts: Alerts, modules: &[bidspec::Module], vocab: &Vocabulary) -> AlertTable {
        let reg = vocab.registry();
        let mut t = AlertTable::default();
        for (i, e) in alerts.entries.iter().enumerate() {
            let defs: Vec<_> = e.fields.iter().filter_map(|f| reg.get(f)).collect();
            if defs.is_empty() {
                t.unresolved.push((
                    e.index,
                    format!("{} is not a card field", e.fields.join(", ")),
                ));
                continue;
            }
            let (probes, seat) = match entry_auctions(e) {
                Ok(p) => p,
                Err(m) => {
                    t.unresolved.push((e.index, m));
                    continue;
                }
            };
            let played = |d: &&bridge_card::FieldDef| {
                d.kind == FieldKind::Bool
                    || reg
                        .fields()
                        .iter()
                        .any(|o| o.plays.values().any(|id| *id == d.path))
            };
            let are = if defs.iter().any(played) {
                FieldsAre::Played
            } else if defs.iter().any(|d| {
                // A function's part (`opening.2d`: the convention the
                // card plays for it) reads as an enum of conventions.
                d.derived.is_some()
                    || (d.kind == FieldKind::Enum
                        && !d.options.iter().all(|o| o.parse::<i64>().is_ok()))
            }) {
                FieldsAre::Chosen
            } else {
                FieldsAre::Setting
            };
            let r = t.resolved.len();
            t.resolved.push(Resolved {
                entry: i,
                fields: defs.iter().map(|d| d.path.clone()).collect(),
                are,
                modules: entry_modules(modules, vocab, &e.fields),
                breadth: probes.len(),
            });
            for p in probes {
                let (passes, k) = key(&p.calls, &p.call);
                let lead = match seat {
                    Seat::Early => Lead::Early,
                    Seat::Late => Lead::Late,
                    Seat::Any if passes >= 2 => Lead::Late,
                    Seat::Any => Lead::Any,
                };
                let list = t.index.entry(k).or_default();
                if !list.contains(&(r, lead)) {
                    list.push((r, lead));
                }
            }
        }
        t.alerts = alerts;
        t
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// The entry that decides how `call` after `calls` is alerted under
    /// `regulator`, when the rule of `module` makes it on `card`.
    /// `call_modules` are the modules of every rule that makes the same
    /// call here; `flagged`, whether the rule alerts or announces it.
    #[allow(clippy::too_many_arguments)]
    pub fn decide<'a>(
        &self,
        regulator: Regulator,
        card: &Card,
        calls: &[Call],
        call: &Call,
        module: &str,
        call_modules: impl IntoIterator<Item = &'a str> + Clone,
        flagged: bool,
    ) -> Result<Decided, Fallback> {
        if self.index.is_empty() {
            return Err(Fallback::NoEntry);
        }
        let (passes, k) = key(calls, call);
        let Some(list) = self.index.get(&k) else {
            return Err(Fallback::NoEntry);
        };
        let here: Vec<&Resolved> = list
            .iter()
            .filter(|(_, lead)| lead.holds(passes))
            .map(|(r, _)| &self.resolved[*r])
            .collect();
        if here.is_empty() {
            return Err(Fallback::NoEntry);
        }
        let entry = |r: &Resolved| &self.alerts.entries[r.entry];
        let firm: Vec<&Resolved> = here
            .into_iter()
            .filter(|r| entry(r).when.is_none())
            .collect();
        if firm.is_empty() {
            return Err(Fallback::Variant);
        }
        let on: Vec<&Resolved> = firm
            .into_iter()
            .filter(|r| match r.are {
                FieldsAre::Played => r.fields.iter().any(|f| plays(card, f)),
                FieldsAre::Chosen | FieldsAre::Setting => {
                    r.fields.iter().any(|f| card.effective(f).is_some())
                }
            })
            .collect();
        if on.is_empty() {
            return Err(Fallback::NotPlayed);
        }
        // The entries of the rule's own module; failing those, an entry
        // none of whose modules makes the call here, as `rbb bid alerts`
        // judges one: a setting no module reads (`1♣` that may be short),
        // a convention's call another module makes (the super-accept of a
        // Jacoby transfer, the forcing 1NT's 2♣ rebid). It can also take
        // a natural call where the convention does not apply (a natural
        // 2NT over 1NT–(2♣) when Lebensohl starts over 2♦).
        let own: Vec<&Resolved> = on
            .iter()
            .copied()
            .filter(|r| r.modules.contains(module))
            .collect();
        let chosen = if own.is_empty() {
            on.into_iter()
                .filter(|r| {
                    !call_modules
                        .clone()
                        .into_iter()
                        .any(|m| r.modules.contains(m))
                })
                .collect()
        } else {
            own
        };
        if chosen.is_empty() {
            return Err(Fallback::OtherModule);
        }
        let chosen: Vec<&Resolved> = chosen
            .into_iter()
            .filter(|r| flagged || r.are != FieldsAre::Setting)
            .collect();
        if chosen.is_empty() {
            return Err(Fallback::Setting);
        }
        let tier = chosen.iter().map(|r| r.are).min().unwrap();
        let mut chosen: Vec<&Resolved> = chosen.into_iter().filter(|r| r.are == tier).collect();
        let key = regulator.key();
        let disagree = |c: &[&Resolved]| {
            let first = entry(c[0]).rule_for(key);
            c.iter().any(|r| entry(r).rule_for(key) != first)
        };
        if disagree(&chosen) {
            let narrowest = chosen.iter().map(|r| r.breadth).min().unwrap();
            chosen.retain(|r| r.breadth == narrowest);
        }
        let first = entry(chosen[0]).rule_for(key);
        if disagree(&chosen) {
            return Err(Fallback::Ambiguous {
                entries: chosen.iter().map(|r| entry(r).index).collect(),
            });
        }
        Ok(Decided {
            entry: entry(chosen[0]).index,
            rule: first,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_written_auctions() {
        let segs = |s: &str| parse_after(s).unwrap().0;
        assert_eq!(segs("1NT–(2x)").len(), 2);
        assert!(segs("1NT–(2x)")[1].theirs);
        assert_eq!(segs("opening"), Vec::new());
        assert_eq!(parse_after("1♥/1♠ (3rd/4th seat)").unwrap().1, Seat::Late);
        assert_eq!(segs("1♥/1♠ (3rd/4th seat)").len(), 2);
        assert_eq!(parse_call("2♣/2♦ (three cards)", &[]).unwrap().len(), 2);
        assert_eq!(parse_call("1♣ or 1♦", &[]).unwrap().len(), 2);
        assert!(parse_after("suit agreed").is_err());
        assert!(parse_call("New suit", &[]).is_err());
        assert_eq!(segs("1x–(X or overcall)")[1].alts.len(), 5);
        assert!(parse_call("3 of their suit", &[]).is_err());
    }

    #[test]
    fn expands_with_passes_variables_and_pairs() {
        let p = |a: &str, c: &str| {
            let segs = parse_after(a).unwrap().0;
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
    fn keys_ignore_the_passes_before_the_opening() {
        let c = |s: &str| -> Vec<Call> {
            s.split_whitespace()
                .map(|t| Call::from_pbn(t).unwrap())
                .collect()
        };
        let two_c = Call::from_pbn("2C").unwrap();
        assert_eq!(key(&c("1NT Pass"), &two_c), (0, "1NT Pass -> 2C".into()));
        assert_eq!(
            key(&c("Pass Pass 1NT Pass"), &two_c),
            (2, "1NT Pass -> 2C".into())
        );
        assert!(Lead::Late.holds(3) && !Lead::Late.holds(1));
        assert!(Lead::Early.holds(1) && !Lead::Early.holds(2));
    }

    #[test]
    fn regulators_read_and_print() {
        for r in Regulator::ALL {
            assert_eq!(r.key().parse::<Regulator>().unwrap(), r);
        }
        assert_eq!("EBU".parse::<Regulator>().unwrap(), Regulator::Ebu);
        assert!("fia".parse::<Regulator>().is_err());
        assert_eq!(Regulator::default(), Regulator::Acbl);
    }
}
