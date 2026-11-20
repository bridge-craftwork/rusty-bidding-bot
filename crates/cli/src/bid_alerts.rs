//! `rbb bid alerts`: the pack's `alerts.toml` against the rules
//! (bridge-common-conventions#132), as the engine alerts by both.
//!
//! For every entry the card gets the entry's fields turned on, the entry's
//! `after` is written out as concrete auctions (`rbb_engine::alerting`,
//! the same reading the engine alerts by), and the engine is asked which
//! rules make the entry's `call` there (`Engine::rules_at`, as `rbb bid
//! rules` asks). For each such rule the report gives what the engine tells
//! the table under the regulator (`Engine::alert_at`: the entry's kind
//! where the entry decides, the rule's own flag where it falls back, and
//! why) beside the rule's own flag, so both "does the engine do what the
//! entry says" and "does the `.bid` flag agree" can be read. Read-only: it
//! changes neither file.

use std::collections::{BTreeMap, BTreeSet};

use bidspec::alerts::{AlertEntry, AlertRule, Alerts};
use bidspec::ast::{Alert, Context};
use bridge_card::{Card, FieldKind, Value, Vocabulary};
use bridge_types::{Direction, ScoringMethod, Vulnerability};
use rbb_engine::alerting::{entry_modules, entry_probes, pbn, Probe};
use rbb_engine::{Engine, Regulator, RuleRef, RuleSet};
use serde_json::{json, Value as Json};

fn kind(alert: &Option<Alert>) -> &'static str {
    match alert {
        Some(Alert::Alert { delayed: true, .. }) => "delayed",
        Some(Alert::Alert { .. }) => "alert",
        Some(Alert::Announce { .. }) => "announce",
        None => "none",
    }
}

/// Whether a call alerted as `kind` is what the regulator asks. A rule's
/// own flag (`flag`) has no delayed alert: an alerted rule stands for one.
fn agrees(expected: AlertRule, kind: &str, flag: bool) -> bool {
    match expected {
        AlertRule::Delayed if flag => kind == "alert",
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
    /// What the engine tells the table under the regulator (in the first
    /// auction it was found after).
    pub kind: &'static str,
    /// The rule's own flag.
    pub flag: &'static str,
    pub artificial: bool,
    /// Partner reads the call by it (the call's top rule).
    pub read_as: bool,
    /// The best-ranked judged rule for the call in one of its auctions.
    pub decides: bool,
    /// The engine does what the entry asks, in every auction.
    pub agrees: bool,
    /// The rule's flag is what the entry asks.
    pub flag_agrees: bool,
    /// The `alerts.toml` entries that decided the engine's alert.
    pub entries: BTreeSet<usize>,
    /// Where none did: why the rule's flag stood.
    pub fallbacks: BTreeSet<String>,
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

    /// From the judged rules, by `agrees` (the engine's alert) or
    /// `flag_agrees` (the rules' flags).
    fn of(found: &[Found], ok: impl Fn(&Found) -> bool) -> Status {
        if found.is_empty() {
            Status::Unresolved
        } else if found.iter().filter(|f| f.decides).any(|f| !ok(f)) {
            Status::Differ
        } else if found.iter().any(|f| !ok(f)) {
            Status::Mixed
        } else {
            Status::Agree
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntryResult {
    pub entry: AlertEntry,
    pub expected: AlertRule,
    /// What the engine does against the entry.
    pub status: Status,
    /// The rules' own flags against the entry (the pack's work list).
    pub flag_status: Status,
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
    pub regulator: Regulator,
    pub entries: Vec<EntryResult>,
    pub unlisted: Vec<Unlisted>,
}

fn check_entry(rules: &RuleSet, base: &Card, e: &AlertEntry, regulator: Regulator) -> EntryResult {
    let expected = e.rule_for(regulator.key());
    let mut r = EntryResult {
        entry: e.clone(),
        expected,
        status: Status::Unresolved,
        flag_status: Status::Unresolved,
        reason: None,
        card_changes: Vec::new(),
        probes: 0,
        own_rules: false,
        found: Vec::new(),
    };
    let probes: Vec<Probe> = match entry_probes(e) {
        Ok(p) => p,
        Err(m) => {
            r.reason = Some(m);
            return r;
        }
    };
    r.probes = probes.len();
    let (card, changes) = match card_for(base, &rules.vocab, &e.fields) {
        Ok(c) => c,
        Err(m) => {
            r.reason = Some(m);
            return r;
        }
    };
    r.card_changes = changes;
    let mine = entry_modules(&rules.modules, &rules.vocab, &e.fields);
    let engine = Engine::new(&card, &card, rules).with_regulator(regulator);
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
            .map(|o| {
                let (alert, decided) = engine
                    .alert_at(&interp.position, &p.call, &o.rule)
                    .unwrap_or((
                        o.alert.clone(),
                        Err(rbb_engine::alerting::Fallback::NoEntry),
                    ));
                Candidate {
                    rule: o.rule.clone(),
                    call: pc.clone(),
                    explanation: o.explanation.clone(),
                    kind: kind(&alert),
                    flag: kind(&o.alert),
                    decided: decided.map(|d| d.entry).map_err(|f| match f {
                        rbb_engine::alerting::Fallback::Ambiguous { entries } => format!(
                            "entries {} disagree",
                            entries
                                .iter()
                                .map(|i| format!("#{i}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        f => f.as_str().to_string(),
                    }),
                    artificial: o.artificial,
                    read_as: o.read_as,
                    mine: mine.contains(&o.rule.module),
                }
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
            let at = match found
                .iter()
                .position(|f| f.file == o.rule.file && f.line == o.rule.line && f.call == o.call)
            {
                Some(at) => at,
                None => {
                    found.push(Found {
                        module: o.rule.module.clone(),
                        file: o.rule.file.clone(),
                        line: o.rule.line,
                        call: o.call.clone(),
                        explanation: o.explanation.clone(),
                        kind: o.kind,
                        flag: o.flag,
                        artificial: o.artificial,
                        read_as: false,
                        decides: false,
                        agrees: true,
                        flag_agrees: agrees(expected, o.flag, true),
                        entries: BTreeSet::new(),
                        fallbacks: BTreeSet::new(),
                        auctions: Vec::new(),
                    });
                    found.len() - 1
                }
            };
            let f = &mut found[at];
            f.read_as |= o.read_as;
            f.decides |= i == 0;
            // A more specific entry for the same call (`1♣/1♦–(1♥/1♠)`
            // within `1x–(overcall)`) decides it there: alerts.toml's word.
            let by_other = matches!(o.decided, Ok(n) if n != e.index);
            f.agrees &= by_other || agrees(expected, o.kind, false);
            match &o.decided {
                Ok(n) => {
                    f.entries.insert(*n);
                }
                Err(why) => {
                    f.fallbacks.insert(why.clone());
                }
            }
            f.auctions.push(auction.clone());
        }
    }
    if found.is_empty() {
        r.reason = Some(format!("no rule makes the call: {}", missing.join("; ")));
        return r;
    }
    r.status = Status::of(&found, |f| f.agrees);
    r.flag_status = Status::of(&found, |f| f.flag_agrees);
    if !missing.is_empty() {
        r.reason = Some(format!("no rule makes the call: {}", missing.join("; ")));
    }
    r.found = found;
    r
}

/// A rule for the call in one auction.
struct Candidate {
    rule: RuleRef,
    call: String,
    explanation: String,
    /// The engine's alert under the regulator.
    kind: &'static str,
    /// The rule's own flag.
    flag: &'static str,
    /// The entry that decided the engine's alert, or why none did.
    decided: Result<usize, String>,
    artificial: bool,
    read_as: bool,
    /// Its module belongs to the entry's fields.
    mine: bool,
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
/// entry's fields turned on, for `regulator`.
pub fn check(rules: &RuleSet, base: &Card, alerts: &Alerts, regulator: Regulator) -> Report {
    Report {
        regulator,
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

    /// Entries that are variants (`when`) or not, by what the engine does.
    pub fn count(&self, s: Status, variants: bool) -> usize {
        self.entries
            .iter()
            .filter(|r| r.status == s && r.entry.when.is_some() == variants)
            .count()
    }

    /// The same by the rules' own flags.
    pub fn count_flags(&self, s: Status, variants: bool) -> usize {
        self.entries
            .iter()
            .filter(|r| r.flag_status == s && r.entry.when.is_some() == variants)
            .count()
    }

    /// Entries that are not variants where the engine does not do what
    /// the entry asks (`--strict`).
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
                    "flag_status": r.flag_status.as_str(),
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
                        "flag": f.flag,
                        "artificial": f.artificial,
                        "read_as": f.read_as,
                        "decides": f.decides,
                        "agrees": f.agrees,
                        "flag_agrees": f.flag_agrees,
                        "entries": f.entries,
                        "fallbacks": f.fallbacks,
                        "auctions": f.auctions,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let summary = |variants: bool, flags: bool| {
            let n = |s| {
                if flags {
                    self.count_flags(s, variants)
                } else {
                    self.count(s, variants)
                }
            };
            json!({
                "agree": n(Status::Agree),
                "mixed": n(Status::Mixed),
                "differ": n(Status::Differ),
                "unresolved": n(Status::Unresolved),
            })
        };
        json!({
            "regulator": self.regulator.key(),
            "summary": summary(false, false),
            "variants": summary(true, false),
            "flag_summary": summary(false, true),
            "flag_variants": summary(true, true),
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
        let reg = self.regulator.key().to_uppercase();
        let _ = writeln!(
            s,
            "alerts.toml against the engine's alerts and the rules' flags ({reg}): {} entries",
            self.entries.len()
        );
        let line = |v: bool, flags: bool| {
            let n = |st| {
                if flags {
                    self.count_flags(st, v)
                } else {
                    self.count(st, v)
                }
            };
            format!(
                "agree {}, mixed {}, differ {}, unresolved {}",
                n(Status::Agree),
                n(Status::Mixed),
                n(Status::Differ),
                n(Status::Unresolved)
            )
        };
        let _ = writeln!(s, "  the engine's alerts: {}", line(false, false));
        let _ = writeln!(
            s,
            "    variants (`when`, never applied): {}",
            line(true, false)
        );
        let _ = writeln!(s, "  the rules' own flags: {}", line(false, true));
        let _ = writeln!(s, "    variants: {}", line(true, true));
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
        let rule_line = |f: &Found, ok: bool| {
            let source = if f.entries.is_empty() {
                format!(
                    "flag ({})",
                    f.fallbacks.iter().cloned().collect::<Vec<_>>().join("; ")
                )
            } else {
                let e: Vec<String> = f.entries.iter().map(|i| format!("#{i}")).collect();
                let also = if f.fallbacks.is_empty() {
                    String::new()
                } else {
                    format!(
                        ", elsewhere flag ({})",
                        f.fallbacks.iter().cloned().collect::<Vec<_>>().join("; ")
                    )
                };
                format!("entry {}{also}", e.join(", "))
            };
            format!(
                "      {} {:8} flag {:8} {}:{} {} \"{}\"{}{}  by {source}  [{}]",
                if ok { "ok" } else { "!=" },
                f.kind,
                f.flag,
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
        // What the engine does that the entry does not ask: where it fell
        // back on a rule's flag that disagrees.
        for status in [Status::Differ, Status::Mixed] {
            let list: Vec<&EntryResult> =
                self.entries.iter().filter(|r| r.status == status).collect();
            if list.is_empty() {
                continue;
            }
            let _ = writeln!(
                s,
                "\nThe engine {} ({}):",
                match status {
                    Status::Differ =>
                        "differs (the best-ranked rule for the call, in at least one auction)",
                    _ => "is mixed (the best-ranked rules agree, a lower one does not)",
                },
                list.len()
            );
            for r in list {
                let _ = writeln!(s, "    {}", head(r));
                for f in &r.found {
                    if verbose || !f.agrees || f.decides {
                        let _ = writeln!(s, "{}", rule_line(f, f.agrees));
                    }
                }
            }
        }
        // Where the engine alerted by a rule's flag although an entry was
        // read for the call.
        let fell: Vec<&EntryResult> = self
            .entries
            .iter()
            .filter(|r| r.entry.when.is_none() && r.found.iter().any(|f| !f.fallbacks.is_empty()))
            .collect();
        if !fell.is_empty() {
            let _ = writeln!(
                s,
                "\nThe rule's own flag stood, in at least one auction ({}):",
                fell.len()
            );
            for r in fell {
                let _ = writeln!(s, "    {}", head(r));
                for f in r.found.iter().filter(|f| !f.fallbacks.is_empty()) {
                    let _ = writeln!(s, "{}", rule_line(f, f.agrees));
                }
            }
        }
        // The pack's work list: flags the entry now overrides.
        for status in [Status::Differ, Status::Mixed] {
            let list: Vec<&EntryResult> = self
                .entries
                .iter()
                .filter(|r| r.flag_status == status)
                .collect();
            if list.is_empty() {
                continue;
            }
            let _ = writeln!(
                s,
                "\nRule flags that {} the entry ({}):",
                match status {
                    Status::Differ => "differ from",
                    _ => "partly differ from",
                },
                list.len()
            );
            for r in list {
                let _ = writeln!(s, "    {}", head(r));
                for f in &r.found {
                    if verbose || !f.flag_agrees {
                        let _ = writeln!(s, "{}", rule_line(f, f.flag_agrees));
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
            "\nThe engine agrees ({}):",
            self.count(Status::Agree, false) + self.count(Status::Agree, true)
        );
        for r in self.entries.iter().filter(|r| r.status == Status::Agree) {
            let _ = writeln!(s, "    {}", head(r));
            if verbose {
                for f in &r.found {
                    let _ = writeln!(s, "{}", rule_line(f, f.agrees));
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

    fn report(regulator: Regulator, engine_reads_alerts: bool) -> Report {
        let alerts = bidspec::alerts::parse(ALERTS).unwrap();
        let rules = if engine_reads_alerts {
            rules().with_alerts(alerts.clone())
        } else {
            rules()
        };
        let base = Card::new(&rules.vocab);
        check(&rules, &base, &alerts, regulator)
    }

    #[test]
    fn compares_each_entry_with_the_rules() {
        let r = report(Regulator::Acbl, true);
        let status: Vec<(Status, Status)> = r
            .entries
            .iter()
            .map(|e| (e.status, e.flag_status))
            .collect();
        assert_eq!(
            status,
            [
                // Stayman is alerted in the rule; the ACBL exempts it, and
                // the engine follows the entry.
                (Status::Agree, Status::Differ),
                // 2♦ announced (agrees), 2♥ alerted (does not).
                (Status::Agree, Status::Differ),
                (Status::Agree, Status::Agree),
                (Status::Unresolved, Status::Unresolved),
                (Status::Unresolved, Status::Unresolved),
                (Status::Unresolved, Status::Unresolved),
                (Status::Unresolved, Status::Unresolved),
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
        let stayman = &r.entries[0].found[0];
        assert_eq!(stayman.entries, BTreeSet::from([1]));
        assert_eq!((stayman.kind, stayman.flag), ("none", "alert"));
        let transfers = &r.entries[1];
        assert_eq!(transfers.card_changes, ["transfers.play=true"]);
        let by_call: BTreeMap<&str, (bool, bool, &str)> = transfers
            .found
            .iter()
            .map(|f| (f.call.as_str(), (f.agrees, f.flag_agrees, f.kind)))
            .collect();
        assert_eq!(
            by_call,
            BTreeMap::from([
                ("2D", (true, true, "announce")),
                ("2H", (true, false, "announce"))
            ])
        );
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
        assert_eq!(r.failures(), 0);
        // The WBF alerts Stayman, as the rule does.
        let wbf = report(Regulator::Wbf, true);
        assert_eq!(wbf.entries[0].status, Status::Agree);
        assert_eq!(wbf.entries[0].found[0].kind, "alert");
        // Rules that bring no alerts.toml: the engine alerts by the flags.
        let flags = report(Regulator::Acbl, false);
        assert_eq!(flags.entries[0].status, Status::Differ);
        assert!(flags.entries[0].found[0].fallbacks.contains("no entry"));
        assert_eq!(flags.failures(), 2);
    }

    #[test]
    fn lists_alerted_rules_in_modules_no_entry_names() {
        let r = report(Regulator::Acbl, false);
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
