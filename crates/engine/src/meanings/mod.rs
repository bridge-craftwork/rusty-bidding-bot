//! The opponents' announced meanings (rusty-seat-service docs/seat-api.txt
//! sections 3 and 5: `meanings`, the alert text a seat hears for each of
//! the opponents' calls) and how the engine reads them:
//!
//! - a convention name the pack's table knows (`announced-names.toml`,
//!   [`AnnouncedNames`]) switches that card field on in their side's card
//!   for the request, so their call and the rest of their auction are
//!   read by our module for it (`Engine::interpret_with`);
//! - otherwise, for a call no rule reads, [`parse`] picks the lengths, HCP
//!   and flags out of the text, and they become what the call shows.
//!
//! Partner's announcements are never read: they are unauthorised
//! information (`Meanings::listener`).

mod parse;

pub use parse::{parse, Parsed};

use bidspec::ast::CallSpec;
use bridge_types::Direction;
use serde::{Deserialize, Serialize};

use crate::macros::{Alt, Auctions, Defines};

/// The name table's file in a pack checkout.
pub const FILE: &str = "announced-names.toml";
/// Its key in a pack snapshot (`pack.json`).
pub const SNAPSHOT_KEY: &str = "announced_names";

/// What a seat heard the opponents announce: one text per call, aligned
/// with the auction (`""` or missing where nothing was said).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Meanings {
    pub texts: Vec<String>,
    /// The seat hearing them: only its opponents' entries are read (a
    /// partner's announcement is unauthorised information). `None` reads
    /// every entry as given: the caller has already left out what the
    /// reader may not hear.
    pub listener: Option<Direction>,
}

impl Meanings {
    /// What `listener` heard.
    pub fn heard_by(listener: Direction, texts: Vec<String>) -> Meanings {
        Meanings {
            texts,
            listener: Some(listener),
        }
    }

    /// Nothing heard.
    pub fn is_empty(&self) -> bool {
        self.texts.iter().all(|t| t.trim().is_empty())
    }

    /// The text for call `i` made by `caller`, if the listener may read it.
    pub fn for_call(&self, i: usize, caller: Direction) -> Option<&str> {
        if self
            .listener
            .is_some_and(|l| l == caller || l.partner() == caller)
        {
            return None;
        }
        self.texts
            .get(i)
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
    }
}

/// One entry of the table as written (`announced-names.toml`, `[[name]]`).
#[derive(Debug, Clone, Deserialize)]
struct RawEntry {
    names: Vec<String>,
    set: Vec<String>,
    #[serde(default)]
    call: Option<OneOrMany>,
    #[serde(default)]
    after: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
struct RawTable {
    #[serde(default)]
    name: Vec<RawEntry>,
}

/// One entry, ready to match.
#[derive(Debug, Clone)]
pub(crate) struct NameEntry {
    /// Its names, normalised (`key`).
    pub keys: Vec<String>,
    /// Its names as written, beside `keys`.
    pub names: Vec<String>,
    /// Card changes, `path=value` (`=true` added to a bare address).
    pub set: Vec<String>,
    /// The calls it applies to (empty: any).
    pub calls: Vec<CallSpec>,
    /// The auction before the call (none: anywhere).
    pub after: Option<Vec<Alt>>,
}

impl NameEntry {
    pub fn has_context(&self) -> bool {
        !self.calls.is_empty() || self.after.is_some()
    }
}

/// The pack's table of announced convention names (`announced-names.toml`).
#[derive(Debug, Clone, Default)]
pub struct AnnouncedNames {
    pub(crate) entries: Vec<NameEntry>,
    /// Entries left out, and why (a call or pattern that does not parse).
    pub problems: Vec<String>,
}

/// A name as it is matched: case, punctuation, spacing and CamelCase
/// ignored, suit symbols as letters, "notrump", "NT" and "N" after a
/// level as one (`2N` is `2nt`). The pack's checker (scripts/check.py,
/// `announced_key`) normalises the same way.
pub fn key(text: &str) -> String {
    let mut split = String::with_capacity(text.len() + 8);
    let mut prev: Option<char> = None;
    for c in text.chars() {
        if let Some(p) = prev {
            if (p.is_ascii_lowercase() && c.is_ascii_uppercase())
                || (p.is_ascii_alphabetic() && c.is_ascii_digit())
            {
                split.push(' ');
            }
        }
        split.push(c);
        prev = Some(c);
    }
    let mut s = split.to_lowercase();
    for (sym, letter) in [("♠", "s"), ("♥", "h"), ("♦", "d"), ("♣", "c")] {
        s = s.replace(sym, letter);
    }
    for nt in [
        "no-trumps",
        "no trumps",
        "notrumps",
        "no-trump",
        "no trump",
        "notrump",
    ] {
        s = s.replace(nt, "nt");
    }
    let s: String = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect();
    let toks: Vec<&str> = s.split_whitespace().collect();
    let level = |t: &str| t.len() == 1 && matches!(t.as_bytes()[0], b'1'..=b'7');
    let mut out: Vec<String> = Vec::with_capacity(toks.len());
    let mut i = 0;
    while i < toks.len() {
        let t = toks[i];
        if level(t) {
            if let Some(&n) = toks.get(i + 1) {
                if matches!(n, "c" | "d" | "h" | "s" | "nt" | "n") {
                    out.push(format!("{t}{}", if n == "n" { "nt" } else { n }));
                    i += 2;
                    continue;
                }
            }
        }
        if t.len() == 2 && level(&t[..1]) && t.ends_with('n') {
            out.push(format!("{t}t"));
        } else {
            out.push(t.to_string());
        }
        i += 1;
    }
    out.join(" ")
}

/// A synthetic rule file holding an entry's context, parsed with the rule
/// language's own parser: `after <pattern>` and one rule per call.
fn context(
    after: Option<&str>,
    calls: &[String],
) -> Result<(Vec<CallSpec>, Option<Vec<Alt>>), String> {
    let calls: Vec<String> = if calls.is_empty() {
        vec!["P".into()]
    } else {
        calls.to_vec()
    };
    let mut src = String::from("module announced-name \"announced name\"\n\n");
    match after {
        Some(a) => src.push_str(&format!("after {a}\n")),
        None => src.push_str("when opening | !opening\n"),
    }
    for c in &calls {
        src.push_str(&format!("  {c} \"announced\"\n"));
    }
    let module = bidspec::parse(&src, FILE).map_err(|d| {
        d.iter()
            .map(|d| d.message.clone())
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    let ctx = module
        .contexts
        .first()
        .ok_or_else(|| "no context parsed".to_string())?;
    let specs: Vec<CallSpec> = ctx.rules.iter().map(|r| r.call.clone()).collect();
    let alts = match &ctx.after {
        Some(a) => {
            let mut errors = Vec::new();
            let modules = [module.clone()];
            let alts = Auctions::new(&modules).resolve(a, &Defines::new(&modules), &mut errors);
            if !errors.is_empty() {
                return Err(errors.join("; "));
            }
            Some(alts)
        }
        None => None,
    };
    Ok((specs, alts))
}

impl AnnouncedNames {
    /// The table from `announced-names.toml`'s text. Card fields are not
    /// checked here (the engine applies them to a card and reports a
    /// change the card refuses); a call or pattern that does not parse
    /// leaves its entry out, in `problems`.
    pub fn from_toml(text: &str) -> Result<AnnouncedNames, String> {
        let raw: RawTable = toml::from_str(text).map_err(|e| format!("{FILE}: {e}"))?;
        Ok(Self::from_raw(raw))
    }

    /// The table from a snapshot's `announced_names` value (the TOML file
    /// as JSON).
    pub fn from_json(value: &serde_json::Value) -> Result<AnnouncedNames, String> {
        let raw: RawTable =
            serde_json::from_value(value.clone()).map_err(|e| format!("{SNAPSHOT_KEY}: {e}"))?;
        Ok(Self::from_raw(raw))
    }

    fn from_raw(raw: RawTable) -> AnnouncedNames {
        let mut out = AnnouncedNames::default();
        for (i, e) in raw.name.into_iter().enumerate() {
            let calls = match e.call {
                None => Vec::new(),
                Some(OneOrMany::One(c)) => vec![c],
                Some(OneOrMany::Many(v)) => v,
            };
            let (specs, after) = if calls.is_empty() && e.after.is_none() {
                (Vec::new(), None)
            } else {
                match context(e.after.as_deref(), &calls) {
                    Ok((specs, after)) => {
                        (if calls.is_empty() { Vec::new() } else { specs }, after)
                    }
                    Err(err) => {
                        out.problems.push(format!("{FILE}: entry {}: {err}", i + 1));
                        continue;
                    }
                }
            };
            let set = e
                .set
                .iter()
                .map(|s| {
                    if s.contains('=') {
                        s.trim().to_string()
                    } else {
                        format!("{}=true", s.trim())
                    }
                })
                .collect();
            out.entries.push(NameEntry {
                keys: e.names.iter().map(|n| key(n)).collect(),
                names: e.names,
                set,
                calls: specs,
                after,
            });
        }
        out
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries one of whose names starts `text`, best first: the
    /// longer name, then an entry with a context before one without. Each
    /// with the name as written that matched.
    pub(crate) fn candidates(&self, text: &str) -> Vec<(&NameEntry, &str)> {
        let t = key(text);
        let mut hits: Vec<(&NameEntry, &str, usize)> = self
            .entries
            .iter()
            .filter_map(|e| {
                e.keys
                    .iter()
                    .zip(&e.names)
                    .filter(|(k, _)| {
                        !k.is_empty()
                            && (t == **k
                                || (t.starts_with(k.as_str())
                                    && t.as_bytes().get(k.len()) == Some(&b' ')))
                    })
                    .max_by_key(|(k, _)| k.len())
                    .map(|(k, n)| (e, n.as_str(), k.len()))
            })
            .collect();
        hits.sort_by(|a, b| (b.2, b.0.has_context()).cmp(&(a.2, a.0.has_context())));
        hits.into_iter().map(|(e, n, _)| (e, n)).collect()
    }

    /// Every card change the table names, for checking against a
    /// vocabulary.
    pub fn changes(&self) -> impl Iterator<Item = (&[String], &str)> {
        self.entries
            .iter()
            .flat_map(|e| e.set.iter().map(move |s| (e.names.as_slice(), s.as_str())))
    }

    /// The table's problems against `vocab`: entries left out at load
    /// (`problems`), and changes a card in `vocab` refuses (a field it
    /// lacks, a value it does not take). `rbb bid check` reports them.
    pub fn check(&self, vocab: &bridge_card::Vocabulary) -> Vec<String> {
        let mut out = self.problems.clone();
        for (names, change) in self.changes() {
            let mut card = vocab.new_card();
            if let Err(e) = card.apply_change(change) {
                out.push(format!(
                    "{FILE}: {}: {e}",
                    names.first().map_or("", |n| n.as_str())
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys() {
        assert_eq!(key("Multi-Landy"), "multi landy");
        assert_eq!(key("MultiLandy"), "multi landy");
        assert_eq!(key("Unusual 2NT"), "unusual 2nt");
        assert_eq!(key("Unusual 2N"), "unusual 2nt");
        assert_eq!(key("unusual 2 nt"), "unusual 2nt");
        assert_eq!(key("Jacoby2NT"), "jacoby 2nt");
        assert_eq!(key("Multi 2♦"), "multi 2d");
        assert_eq!(key("Unusual notrump"), "unusual nt");
        assert_eq!(key("D.O.N.T."), "d o n t");
        assert_eq!(key("4SF"), "4sf");
        assert_eq!(
            key("Cappelletti: one long suit"),
            "cappelletti one long suit"
        );
    }

    const TABLE: &str = r#"
[[name]]
names = ["Cappelletti", "Hamilton"]
set = ["defence_to_1nt.strong=competitive_bidding/cappelletti"]

[[name]]
names = ["Multi", "Multi 2♦"]
set = ["bidding_conventions/multi_2d"]

[[name]]
names = ["Multi"]
call = "2D"
after = "(1N) | (1N) P (P)"
set = ["defence_to_1nt.strong=competitive_bidding/multi_landy"]

[[name]]
names = ["Multi-Landy"]
set = ["defence_to_1nt.strong=competitive_bidding/multi_landy"]

[[name]]
names = ["Michaels"]
call = "2m"
after = "(1m)"
set = ["competitive_bidding/michaels"]
"#;

    #[test]
    fn table_loads_and_matches() {
        let t = AnnouncedNames::from_toml(TABLE).unwrap();
        assert!(t.problems.is_empty(), "{:?}", t.problems);
        assert_eq!(t.len(), 5);
        let c = t.candidates("Cappelletti: one long suit");
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].1, "Cappelletti");
        assert_eq!(
            c[0].0.set,
            vec!["defence_to_1nt.strong=competitive_bidding/cappelletti"]
        );
        // "Hamilton" too; "Hamiltonian" is not it.
        assert_eq!(t.candidates("hamilton").len(), 1);
        assert!(t.candidates("Hamiltonian").is_empty());
        // The longer name, and the entry with a context first.
        let c = t.candidates("Multi-Landy: 2D one major");
        assert_eq!(c[0].1, "Multi-Landy");
        let c = t.candidates("Multi");
        assert_eq!(c.len(), 2);
        assert!(c[0].0.has_context());
        // A bare address is switched on.
        assert_eq!(
            t.candidates("Multi")[1].0.set,
            vec!["bidding_conventions/multi_2d=true"]
        );
    }

    #[test]
    fn a_bad_pattern_is_a_problem_not_an_error() {
        let t = AnnouncedNames::from_toml(
            "[[name]]\nnames = [\"X\"]\ncall = \"2D\"\nafter = \"(((\"\nset = [\"a\"]\n",
        )
        .unwrap();
        assert_eq!(t.len(), 0);
        assert_eq!(t.problems.len(), 1);
    }

    #[test]
    fn partner_is_never_heard() {
        use Direction::*;
        let m = Meanings::heard_by(
            South,
            vec!["15-17".into(), "Cappelletti".into(), "Stayman".into()],
        );
        // North dealt: N, E, S.
        assert_eq!(m.for_call(0, North), None);
        assert_eq!(m.for_call(1, East), Some("Cappelletti"));
        assert_eq!(m.for_call(2, South), None);
        assert_eq!(m.for_call(3, West), None);
    }
}
