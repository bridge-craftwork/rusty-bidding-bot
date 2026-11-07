//! `rbb bid rules`: which rules can make the next call after an auction,
//! asked of the engine (`Engine::rules_at`) rather than worked out by
//! reading .bid files.

use std::fmt::Write as _;
use std::path::Path;

use bridge_types::{Call, Direction, ScoringMethod, Strain, Vulnerability};
use rbb_engine::{RuleListing, RuleOption, RuleRejection};

use crate::Result;

pub struct BidRulesArgs<'a> {
    pub auction: &'a str,
    pub dealer: char,
    pub vul: &'a str,
    pub scoring: &'a str,
    pub cards: [&'a str; 2],
    pub card_changes: &'a [String],
    pub rules: &'a Path,
    pub call: Option<&'a str>,
    /// `--rule` specs: `file:line`, `module`, `module:line`.
    pub rule_specs: &'a [String],
    pub all: bool,
    pub knowledge: bool,
    pub json: bool,
}

pub fn bid_rules(a: &BidRulesArgs) -> Result<()> {
    let calls = a
        .auction
        .split_whitespace()
        .map(|c| Call::from_pbn(c).ok_or_else(|| format!("bad call {c:?}")))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let dealer =
        Direction::from_char(a.dealer.to_ascii_uppercase()).ok_or("dealer must be N, E, S or W")?;
    let vul = Vulnerability::from_pbn(a.vul).ok_or("vulnerability must be None, NS, EW or All")?;
    let scoring = ScoringMethod::from_pbn(a.scoring).ok_or("scoring must be MP or IMP")?;
    let only = a
        .call
        .map(|c| Call::from_pbn(c).ok_or_else(|| format!("bad --call {c:?}")))
        .transpose()?;
    let mut auction = bridge_types::Auction::new(dealer);
    for (i, c) in calls.iter().enumerate() {
        if auction.is_complete() || !auction.is_legal(c) {
            return Err(format!("call {} ({c}) is not legal here", i + 1).into());
        }
        auction.add_call(c.clone());
    }
    if auction.is_complete() {
        return Err("the auction is over: no call to make".into());
    }
    let rules = crate::rules_from(Some(a.rules))?;
    let mut cards = [
        crate::card_or_stock(&rules.vocab, a.cards[0])?,
        crate::card_or_stock(&rules.vocab, a.cards[1])?,
    ];
    for card in &mut cards {
        for change in a.card_changes {
            card.apply_change(change)?;
        }
    }
    let selected = select(&rules.modules, a.rule_specs)?;
    let engine = rbb_engine::Engine::new(&cards[0], &cards[1], &rules);
    let interp = engine.interpret(dealer, vul, scoring, &calls);
    // A selection always reports why a rule fails, so it needs the
    // rejections whether or not --all asked for them.
    let mut listing = engine.rules_at(&interp.position, a.all || !selected.is_empty());
    if let Some(c) = &only {
        filter(&mut listing, c);
    }
    let selection = (!selected.is_empty()).then(|| verdicts(&selected, &mut listing));
    let rows = a
        .knowledge
        .then(|| engine.review(dealer, vul, scoring, &calls, &Default::default()));
    if a.json {
        let mut v = serde_json::to_value(&listing)?;
        v["auction"] = serde_json::json!(calls.iter().map(|c| c.to_pbn()).collect::<Vec<_>>());
        v["dealer"] = serde_json::json!(dealer);
        if let Some(sel) = &selection {
            v["selection"] = sel
                .iter()
                .map(Verdict::to_json)
                .collect::<serde_json::Result<Vec<_>>>()?
                .into();
        }
        if let Some(rows) = rows {
            v["knowledge"] = serde_json::to_value(rows)?;
        }
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(());
    }
    if let Some(rows) = &rows {
        if !rows.is_empty() {
            print!("{}", rbb_engine::review_text(rows));
            println!();
        }
    }
    match &selection {
        Some(sel) => print!("{}", selection_text(&listing, sel, only.as_ref())),
        None => print!("{}", text(&listing, only.as_ref())),
    }
    for w in &listing.warnings {
        eprintln!("warning: {w}");
    }
    Ok(())
}

/// Keep only the rules for `call`. A rejected rule that never got as far
/// as a concrete call is kept when its written call could be `call`
/// (`2x` for any two-level suit bid; `cheapest(x)` and the like always).
fn filter(l: &mut RuleListing, call: &Call) {
    let pbn = call.to_pbn();
    l.candidates.retain(|c| c.call == pbn);
    l.rejected.retain(|r| match &r.call {
        Some(c) => *c == pbn,
        None => spec_may_be(&r.spec, call),
    });
}

fn spec_may_be(spec: &str, call: &Call) -> bool {
    if let Some(c) = Call::from_pbn(spec) {
        return c == *call;
    }
    let mut chars = spec.chars();
    let (Some(d), Some(v)) = (chars.next(), chars.next()) else {
        return true;
    };
    let Call::Bid { level: l, strain } = call else {
        return false; // the rest stand for bids
    };
    let Some(level) = d.to_digit(10) else {
        // cheapest(x), jump(x), step(n), raise, new_suit: any bid, or
        // only bids in the strain a literal argument names.
        let arg = spec
            .split_once('(')
            .and_then(|(_, a)| a.strip_suffix(')'))
            .unwrap_or("");
        let named = match arg {
            "C" => Some(Strain::Clubs),
            "D" => Some(Strain::Diamonds),
            "H" => Some(Strain::Hearts),
            "S" => Some(Strain::Spades),
            "N" | "NT" => Some(Strain::NoTrump),
            _ => None,
        };
        return named.is_none_or(|s| s == *strain);
    };
    if *l as u32 != level {
        return false;
    }
    match v {
        '{' => true,
        'M' => matches!(strain, Strain::Hearts | Strain::Spades),
        'm' => matches!(strain, Strain::Clubs | Strain::Diamonds),
        _ => *strain != Strain::NoTrump,
    }
}

fn flags(c: &RuleOption) -> String {
    let mut f = Vec::new();
    match &c.alert {
        Some(bidspec::ast::Alert::Alert { text: Some(t) }) => f.push(format!("alert: {t}")),
        Some(bidspec::ast::Alert::Alert { text: None }) => f.push("alert".into()),
        Some(bidspec::ast::Alert::Announce { text }) => f.push(format!("announce: {text}")),
        None => {}
    }
    if c.artificial {
        f.push("artificial".into());
    }
    f.extend(c.sets.iter().map(|s| format!("sets {s}")));
    f.join("; ")
}

fn rule_ref(r: &rbb_engine::RuleRef) -> String {
    format!("{}:{} ({})", r.file, r.line, r.module)
}

/// One candidate as text.
fn option_text(c: &RuleOption) -> String {
    let mut out = String::new();
    let call = Call::from_pbn(&c.call).map_or(c.call.clone(), |x| x.to_string());
    let _ = writeln!(
        out,
        "\n{:>3}. {:5} prio {:>3}  descr {:.3}  {}{}",
        c.rank,
        call,
        c.priority,
        c.descriptiveness,
        rule_ref(&c.rule),
        if c.read_as {
            ""
        } else {
            "  [not how partner reads it]"
        },
    );
    let _ = writeln!(out, "     \"{}\"", c.explanation);
    let fl = flags(c);
    if !fl.is_empty() {
        let _ = writeln!(out, "     flags: {fl}");
    }
    if !c.bindings.is_empty() {
        let b: Vec<String> = c.bindings.iter().map(|(k, v)| format!("{k}={v}")).collect();
        let _ = writeln!(out, "     bound: {}", b.join(", "));
    }
    if let Some(s) = &c.shows {
        let _ = writeln!(out, "     shows: {s}");
        if let Some(r) = c.shows_resolved.as_ref().filter(|r| *r != s) {
            let _ = writeln!(out, "       as read: {r}");
        }
    }
    if let Some(w) = &c.when {
        let _ = writeln!(
            out,
            "     when: {w}{}",
            if c.when_reads_hand { "" } else { "  (public)" }
        );
        if let Some(f) = c.when_folded.as_ref().filter(|f| *f != w) {
            let _ = writeln!(out, "       left after the auction: {f}");
        }
    }
    if let Some(p) = &c.prefer {
        let _ = writeln!(out, "     prefer: {p}");
    }
    if let Some(d) = &c.denies {
        let _ = writeln!(out, "     denies: {d}");
    }
    if let Some(x) = &c.excluded {
        let _ = writeln!(out, "     EXCLUDED: {x}");
    }
    if !c.same_call.is_empty() {
        let others: Vec<String> = c
            .same_call
            .iter()
            .map(|r| {
                if r.module != c.rule.module {
                    format!("{} [other module]", rule_ref(r))
                } else {
                    rule_ref(r)
                }
            })
            .collect();
        let _ = writeln!(out, "     same call: {}", others.join(", "));
    }
    out
}

/// The listing as text: one block a rule, best-ranked first.
pub fn text(l: &RuleListing, only: Option<&Call>) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:?} to call{}: {} rule{} can apply{} (best-ranked first: priority, descriptiveness, file order; `prefer` reorders ties by the hand)",
        l.seat,
        if l.forced { ", forced to bid" } else { "" },
        l.candidates.len(),
        if l.candidates.len() == 1 { "" } else { "s" },
        only.map(|c| format!(" for {c}")).unwrap_or_default(),
    );
    for c in &l.candidates {
        out.push_str(&option_text(c));
    }
    if !l.rejected.is_empty() {
        // Nearest misses first: a public `when` or legality, then the
        // rule's call, the context's `when`, and last the `after` lines.
        let _ = writeln!(
            out,
            "\nrules that fail here ({}), nearest misses first:",
            l.rejected.len()
        );
        for stage in ["when", "legal", "call", "context-when", "after"] {
            for r in l.rejected.iter().filter(|r| r.stage == stage) {
                out.push_str(&rejection_line(r));
            }
        }
    }
    if !l.inactive.is_empty() {
        let _ = writeln!(out, "\nmodules off for this side:");
        for (m, why) in &l.inactive {
            let _ = writeln!(out, "  {m}: {why}");
        }
    }
    out
}

fn rejection_line(r: &RuleRejection) -> String {
    format!(
        "  {:12} {:6} {}  \"{}\"\n      {}\n",
        r.stage,
        r.call.as_deref().unwrap_or(&r.spec),
        rule_ref(&r.rule),
        r.explanation,
        r.reason
    )
}

/// A rule of the rule set, active or not.
type RuleId = rbb_engine::RuleRef;

/// Every rule of `modules`, in load order.
fn all_rules(modules: &[bidspec::Module]) -> Vec<RuleId> {
    fn walk(m: &bidspec::Module, c: &bidspec::ast::Context, out: &mut Vec<RuleId>) {
        for r in &c.rules {
            out.push(RuleId {
                module: m.name.clone(),
                file: m.file.clone(),
                line: r.line,
            });
        }
        for child in &c.contexts {
            walk(m, child, out);
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

/// `file` ends with `suffix` at a path boundary.
fn file_matches(file: &str, suffix: &str) -> bool {
    file == suffix || file.ends_with(&format!("/{suffix}"))
}

/// The rules `--rule` specs name, each with the spec that named it first.
/// A spec is `file:line` (any path suffix: `rebids.bid:725`), a file
/// (`rebids.bid`), a module (`rebids`) or `module:line`. A spec that names
/// no rule is an error listing the near matches.
fn select(modules: &[bidspec::Module], specs: &[String]) -> Result<Vec<(String, RuleId)>> {
    let rules = all_rules(modules);
    let mut out: Vec<(String, RuleId)> = Vec::new();
    for spec in specs {
        let (place, line) = match spec.rsplit_once(':') {
            Some((p, l)) if l.parse::<usize>().is_ok() => (p, l.parse::<usize>().ok()),
            _ => (spec.as_str(), None),
        };
        let by_file: Vec<&RuleId> = rules
            .iter()
            .filter(|r| file_matches(&r.file, place))
            .collect();
        let in_place: Vec<&RuleId> = if by_file.is_empty() {
            rules.iter().filter(|r| r.module == place).collect()
        } else {
            by_file
        };
        if in_place.is_empty() {
            let mut near: Vec<String> = modules
                .iter()
                .filter(|m| {
                    let stem = place.trim_end_matches(".bid");
                    let stem = stem.rsplit('/').next().unwrap_or(stem);
                    m.name.contains(stem) || m.file.contains(stem)
                })
                .map(|m| format!("{} ({})", m.name, m.file))
                .collect();
            near.truncate(10);
            return Err(format!(
                "--rule {spec}: no module or rule file {place:?}{}",
                if near.is_empty() {
                    String::new()
                } else {
                    format!("; near: {}", near.join(", "))
                }
            )
            .into());
        }
        let hits: Vec<&RuleId> = match line {
            Some(l) => in_place.iter().copied().filter(|r| r.line == l).collect(),
            None => in_place.clone(),
        };
        if hits.is_empty() {
            let l = line.unwrap_or(0);
            let mut near = in_place.clone();
            near.sort_by_key(|r| (r.line as i64 - l as i64).abs());
            let near: Vec<String> = near
                .iter()
                .take(6)
                .map(|r| format!("{}:{} ({})", r.file, r.line, r.module))
                .collect();
            return Err(format!(
                "--rule {spec}: no rule starts on line {l} there; nearest rules: {}",
                near.join(", ")
            )
            .into());
        }
        for r in hits {
            if !out.iter().any(|(_, o)| o == r) {
                out.push((spec.clone(), r.clone()));
            }
        }
    }
    Ok(out)
}

/// What became of one selected rule at this position.
#[derive(Debug, Clone)]
pub struct Verdict {
    /// The `--rule` spec that selected it.
    pub spec: String,
    pub rule: RuleId,
    /// `applies` (at least one of its calls is a candidate here), `fails`
    /// (none is; `rejected` says why), `inactive` (its module is off for
    /// this side; `reasons` says why) or `filtered` (with `--call`: it has
    /// neither a candidate nor a rejection for that call).
    pub verdict: String,
    /// Its candidates here, as in `candidates` (rank among all of them).
    pub candidates: Vec<RuleOption>,
    /// Its rejections here (each binding or call that failed), as in
    /// `rejected`.
    pub rejected: Vec<RuleRejection>,
    /// The reasons in one line each: the rejections' reasons, or why the
    /// module is off.
    pub reasons: Vec<String>,
}

impl Verdict {
    fn to_json(&self) -> serde_json::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "spec": self.spec,
            "rule": serde_json::to_value(&self.rule)?,
            "verdict": self.verdict,
            "candidates": serde_json::to_value(&self.candidates)?,
            "rejected": serde_json::to_value(&self.rejected)?,
            "reasons": self.reasons,
        }))
    }
}

fn same(r: &rbb_engine::RuleRef, id: &RuleId) -> bool {
    r.file == id.file && r.line == id.line
}

/// The verdict on each selected rule; `listing` keeps only the selected
/// rules' candidates and rejections, and only the inactive modules they
/// belong to.
fn verdicts(selected: &[(String, RuleId)], listing: &mut RuleListing) -> Vec<Verdict> {
    let out = selected
        .iter()
        .map(|(spec, id)| {
            let candidates: Vec<RuleOption> = listing
                .candidates
                .iter()
                .filter(|c| same(&c.rule, id))
                .cloned()
                .collect();
            let rejected: Vec<RuleRejection> = listing
                .rejected
                .iter()
                .filter(|r| same(&r.rule, id))
                .cloned()
                .collect();
            let off = listing.inactive.iter().find(|(m, _)| *m == id.module);
            let (verdict, reasons) = if !candidates.is_empty() {
                (
                    "applies",
                    rejected.iter().map(|r| r.reason.clone()).collect(),
                )
            } else if let Some((m, why)) = off {
                ("inactive", vec![format!("module {m} is off: {why}")])
            } else if rejected.is_empty() {
                (
                    "filtered",
                    vec!["neither a candidate nor a rejection for the --call call".into()],
                )
            } else {
                ("fails", rejected.iter().map(|r| r.reason.clone()).collect())
            };
            Verdict {
                spec: spec.clone(),
                rule: id.clone(),
                verdict: verdict.into(),
                candidates,
                rejected,
                reasons,
            }
        })
        .collect();
    let picked = |r: &rbb_engine::RuleRef| selected.iter().any(|(_, id)| same(r, id));
    listing.candidates.retain(|c| picked(&c.rule));
    listing.rejected.retain(|r| picked(&r.rule));
    listing
        .inactive
        .retain(|(m, _)| selected.iter().any(|(_, id)| id.module == *m));
    out
}

/// The verdicts as text: one block a selected rule.
fn selection_text(l: &RuleListing, sel: &[Verdict], only: Option<&Call>) -> String {
    let mut out = format!(
        "{:?} to call{}{}: {} rule{} selected\n",
        l.seat,
        if l.forced { ", forced to bid" } else { "" },
        only.map(|c| format!(", only {c}")).unwrap_or_default(),
        sel.len(),
        if sel.len() == 1 { "" } else { "s" },
    );
    let filtered: Vec<String> = sel
        .iter()
        .filter(|v| v.verdict == "filtered")
        .map(|v| format!("{}:{}", v.rule.file, v.rule.line))
        .collect();
    for v in sel.iter().filter(|v| v.verdict != "filtered") {
        let _ = writeln!(
            out,
            "\n== {}:{} ({}): {}",
            v.rule.file,
            v.rule.line,
            v.rule.module,
            match v.verdict.as_str() {
                "applies" => "CAN APPLY",
                "fails" => "DOES NOT APPLY",
                "inactive" => "MODULE OFF",
                _ => "NOT THIS CALL",
            }
        );
        for c in &v.candidates {
            out.push_str(&option_text(c));
        }
        if v.verdict == "inactive" {
            for r in &v.reasons {
                let _ = writeln!(out, "     {r}");
            }
        }
        for r in &v.rejected {
            if !v.candidates.is_empty() {
                let _ = writeln!(out, "\n     another binding fails:");
            }
            out.push_str(&rejection_line(r));
        }
    }
    if !filtered.is_empty() {
        let _ = writeln!(
            out,
            "\n{} selected rule{} ha{} no {} here (neither a candidate nor a rejection): {}",
            filtered.len(),
            if filtered.len() == 1 { "" } else { "s" },
            if filtered.len() == 1 { "s" } else { "ve" },
            only.map(|c| c.to_string()).unwrap_or_default(),
            filtered.join(", ")
        );
    }
    out
}
