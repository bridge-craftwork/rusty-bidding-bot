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
    let engine = rbb_engine::Engine::new(&cards[0], &cards[1], &rules);
    let interp = engine.interpret(dealer, vul, scoring, &calls);
    let mut listing = engine.rules_at(&interp.position, a.all);
    if let Some(c) = &only {
        filter(&mut listing, c);
    }
    let rows = a
        .knowledge
        .then(|| engine.review(dealer, vul, scoring, &calls, &Default::default()));
    if a.json {
        let mut v = serde_json::to_value(&listing)?;
        v["auction"] = serde_json::json!(calls.iter().map(|c| c.to_pbn()).collect::<Vec<_>>());
        v["dealer"] = serde_json::json!(dealer);
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
    print!("{}", text(&listing, only.as_ref()));
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
