//! A convention pack's `alerts.toml`: which calls need an alert or an
//! announcement, by card field, and where the regulators differ (the
//! pack's DESIGN.md, "Alerts").
//!
//! The engine does not read it yet: a rule's `alert` / `announce` flag is
//! what it alerts by. `rbb bid alerts` checks the two against each other;
//! this module only reads the file, from a pack checkout or from the
//! snapshot (`pack.json`, whose `alerts` key holds the same tables), so
//! the rule set's loading can share it once alerts are driven by it.
//!
//! ```
//! let a = bidspec::alerts::parse(r#"
//! [regulators.acbl]
//! name = "ACBL"
//! [regulators.ebu]
//! name = "EBU"
//! [[alert]]
//! fields = ["bidding_conventions/stayman"]
//! call = "2♣"
//! after = "1NT"
//! by = "responder"
//! rule = "none"
//! ebu = "announce"
//! "#).unwrap();
//! use bidspec::alerts::AlertRule;
//! assert_eq!(a.entries[0].rule_for("acbl"), AlertRule::None);
//! assert_eq!(a.entries[0].rule_for("ebu"), AlertRule::Announce);
//! ```

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// The file's name at a pack's root.
pub const FILE: &str = "alerts.toml";

/// What a regulator does with a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertRule {
    /// Alerted at the time it is made.
    Alert,
    /// Partner says what it is ("transfer", "15 to 17").
    Announce,
    /// Explained after the auction.
    Delayed,
    /// Not alerted.
    None,
}

impl AlertRule {
    pub fn as_str(self) -> &'static str {
        match self {
            AlertRule::Alert => "alert",
            AlertRule::Announce => "announce",
            AlertRule::Delayed => "delayed",
            AlertRule::None => "none",
        }
    }

    pub fn parse(s: &str) -> Option<AlertRule> {
        Some(match s {
            "alert" => AlertRule::Alert,
            "announce" => AlertRule::Announce,
            "delayed" => AlertRule::Delayed,
            "none" => AlertRule::None,
            _ => return None,
        })
    }
}

/// `[regulators.<key>]`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Regulator {
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// One `[[alert]]` entry.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AlertEntry {
    /// 1-based place in the file, for messages (`alert 12`).
    pub index: usize,
    /// The card fields (addresses) it belongs to.
    pub fields: Vec<String>,
    /// The call, as people write it (`2♦/2♥`, `X`, `New suit`).
    pub call: String,
    /// The auction it comes after (`1NT–2♣`, `(1x)`, `opening`).
    pub after: String,
    /// Who makes it: opener, responder, overcaller, advancer, either.
    pub by: String,
    /// What most regulators do.
    pub rule: AlertRule,
    /// A regulator that differs, by its key (`ebu`).
    pub overrides: BTreeMap<String, AlertRule>,
    /// A variant: the entry applies only if the partnership plays it so.
    pub when: Option<String>,
    pub note: Option<String>,
}

impl AlertEntry {
    /// What `regulator` (`acbl`, `ebu`, `wbf`) does with the call.
    pub fn rule_for(&self, regulator: &str) -> AlertRule {
        self.overrides.get(regulator).copied().unwrap_or(self.rule)
    }
}

/// The whole file.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Alerts {
    pub regulators: BTreeMap<String, Regulator>,
    pub entries: Vec<AlertEntry>,
}

/// The file's shape, as serde reads it from TOML or from the snapshot's
/// JSON.
#[derive(Deserialize)]
struct Doc {
    #[serde(default)]
    regulators: BTreeMap<String, Regulator>,
    #[serde(default)]
    alert: Vec<RawEntry>,
}

#[derive(Deserialize)]
struct RawEntry {
    #[serde(default)]
    fields: Vec<String>,
    call: String,
    after: String,
    by: String,
    rule: String,
    #[serde(default)]
    when: Option<String>,
    #[serde(default)]
    note: Option<String>,
    /// The regulator keys (`ebu = "announce"`).
    #[serde(flatten)]
    rest: BTreeMap<String, serde_json::Value>,
}

impl Doc {
    fn finish(self) -> Result<Alerts, String> {
        let mut entries = Vec::with_capacity(self.alert.len());
        for (i, raw) in self.alert.into_iter().enumerate() {
            let at = |m: String| format!("alert {}: {m}", i + 1);
            let rule = AlertRule::parse(&raw.rule).ok_or_else(|| {
                at(format!(
                    "rule {:?} is not alert, announce, delayed or none",
                    raw.rule
                ))
            })?;
            let mut overrides = BTreeMap::new();
            for (key, value) in raw.rest {
                if !self.regulators.contains_key(&key) {
                    return Err(at(format!("unknown key {key:?}")));
                }
                let r = value.as_str().and_then(AlertRule::parse).ok_or_else(|| {
                    at(format!(
                        "{key} = {value}: not alert, announce, delayed or none"
                    ))
                })?;
                overrides.insert(key, r);
            }
            entries.push(AlertEntry {
                index: i + 1,
                fields: raw.fields,
                call: raw.call,
                after: raw.after,
                by: raw.by,
                rule,
                overrides,
                when: raw.when,
                note: raw.note,
            });
        }
        Ok(Alerts {
            regulators: self.regulators,
            entries,
        })
    }
}

/// Read `alerts.toml`'s text.
pub fn parse(source: &str) -> Result<Alerts, String> {
    let doc: Doc = toml::from_str(source).map_err(|e| e.message().to_string())?;
    doc.finish()
}

/// Read the `alerts` key of a pack's snapshot (`pack.json`), the file
/// parsed into JSON. `None` when the snapshot has none.
pub fn from_snapshot(doc: &serde_json::Value) -> Result<Option<Alerts>, String> {
    let Some(v) = doc.get("alerts") else {
        return Ok(None);
    };
    let doc: Doc = serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
    doc.finish().map(Some)
}

/// The alerts of the rules at `path`: a pack checkout's `alerts.toml`, or
/// a snapshot file's `alerts`. `None` when there are none (a plain rules
/// directory, a snapshot without them).
pub fn load(path: &Path) -> Result<Option<Alerts>, String> {
    let fail = |e: String| format!("{}: {e}", path.display());
    if path.is_file() {
        let text = std::fs::read_to_string(path).map_err(|e| fail(e.to_string()))?;
        let doc: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| fail(e.to_string()))?;
        return from_snapshot(&doc).map_err(fail);
    }
    let file = path.join(FILE);
    if !file.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&file).map_err(|e| fail(e.to_string()))?;
    parse(&text)
        .map(Some)
        .map_err(|e| format!("{}: {e}", file.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = r#"
[regulators.acbl]
name = "ACBL"
[regulators.ebu]
name = "EBU"

[[alert]]
fields = ["a"]
call = "2♣"
after = "1NT"
by = "responder"
rule = "none"
ebu = "announce"
note = "n"

[[alert]]
fields = ["b"]
call = "X"
after = "(1x)"
by = "overcaller"
rule = "alert"
when = "sometimes"
"#;

    #[test]
    fn reads_rules_overrides_and_variants() {
        let a = parse(TEXT).unwrap();
        assert_eq!(a.regulators.len(), 2);
        assert_eq!(a.entries.len(), 2);
        let e = &a.entries[0];
        assert_eq!(
            (e.index, e.rule_for("acbl"), e.rule_for("ebu")),
            (1, AlertRule::None, AlertRule::Announce)
        );
        assert_eq!(e.rule_for("wbf"), AlertRule::None);
        assert_eq!(a.entries[1].when.as_deref(), Some("sometimes"));
    }

    #[test]
    fn the_snapshot_reads_as_the_file() {
        let table: toml::Table = toml::from_str(TEXT).unwrap();
        let json = serde_json::json!({ "alerts": serde_json::to_value(table).unwrap() });
        assert_eq!(from_snapshot(&json).unwrap().unwrap(), parse(TEXT).unwrap());
        assert_eq!(from_snapshot(&serde_json::json!({})).unwrap(), None);
    }

    #[test]
    fn refuses_unknown_keys_and_rules() {
        let bad = TEXT.replace("ebu = \"announce\"", "fia = \"announce\"");
        assert!(parse(&bad).unwrap_err().contains("unknown key \"fia\""));
        let bad = TEXT.replace("rule = \"none\"", "rule = \"maybe\"");
        assert!(parse(&bad).unwrap_err().contains("maybe"));
    }
}
