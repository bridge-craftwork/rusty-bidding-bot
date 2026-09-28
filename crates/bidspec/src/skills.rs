//! The teaching-skill map: each skill path (Bridge-Classroom's SkillPath)
//! with the card fields tagged with it (`skill = ...` in `fields.toml`) and
//! the modules that declare it (`skill <path>` lines), and the gaps between
//! them. `rbb bid skills` prints it; docs/SKILLS.md carries it between
//! [`BEGIN`] and [`END`].

use std::collections::{BTreeMap, BTreeSet};

use bridge_card::{Registry, SkillSource, Skills};

use crate::ast::Module;
use crate::Diagnostic;

/// The markers around the generated map in docs/SKILLS.md.
pub const BEGIN: &str = "<!-- BEGIN GENERATED: rbb bid skills -->";
pub const END: &str = "<!-- END GENERATED: rbb bid skills -->";

/// Categories of the taxonomy that are about bidding (the others are card
/// play and practice sets, which no module or field can name).
const BIDDING: &[&str] = &[
    "basic_bidding",
    "bidding_conventions",
    "competitive_bidding",
    "partnership_bidding",
    "precision",
];

/// One skill path and what names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub path: String,
    /// Display name from `skills.toml`, when listed there.
    pub name: Option<String>,
    /// Where `skills.toml` lists it; `None` when it does not.
    pub source: Option<SkillSource>,
    /// Card fields tagged with it (canonical paths, declaration order).
    pub fields: Vec<String>,
    /// Modules declaring it, by name.
    pub modules: Vec<String>,
}

/// The map and its gaps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillMap {
    /// Every skill a field or a module names, by path.
    pub rows: Vec<Row>,
    /// Modules that declare no skill, by name.
    pub modules_without_skill: Vec<String>,
    /// Bidding skills of the taxonomy that no field and no module names.
    pub unused_taxonomy: Vec<String>,
}

/// Build the map from the rules' modules, card fields and known skills.
pub fn map(modules: &[Module], registry: &Registry, known: &Skills) -> SkillMap {
    let mut fields: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for f in registry.fields() {
        for s in &f.skill {
            fields.entry(s).or_default().push(f.path.clone());
        }
    }
    let mut mods: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for m in modules {
        for s in &m.skills {
            mods.entry(&s.path).or_default().insert(&m.name);
        }
    }
    let paths: BTreeSet<&str> = fields.keys().chain(mods.keys()).copied().collect();
    let rows = paths
        .iter()
        .map(|&p| {
            let skill = known.get(p);
            Row {
                path: p.to_string(),
                name: skill.map(|s| s.name.clone()),
                source: skill.map(|s| s.source),
                fields: fields.get(p).cloned().unwrap_or_default(),
                modules: mods
                    .get(p)
                    .map(|m| m.iter().map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
            }
        })
        .collect();
    let mut modules_without_skill: Vec<String> = modules
        .iter()
        .filter(|m| m.skills.is_empty())
        .map(|m| m.name.clone())
        .collect();
    modules_without_skill.sort();
    let unused_taxonomy = known
        .iter()
        .filter(|s| s.source == SkillSource::Taxonomy)
        .filter(|s| BIDDING.iter().any(|c| s.path.split('/').next() == Some(c)))
        .filter(|s| !paths.contains(s.path.as_str()))
        .map(|s| s.path.clone())
        .collect();
    SkillMap {
        rows,
        modules_without_skill,
        unused_taxonomy,
    }
}

/// Warnings for `rbb bid check`: skill paths that `skills.toml` does not
/// list, on a module's `skill` line or a field's `skill` attribute
/// (`fields_file` names `card/fields.toml` in the message).
pub fn check(
    modules: &[Module],
    registry: &Registry,
    known: &Skills,
    fields_file: &str,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for m in modules {
        for s in &m.skills {
            if known.get(&s.path).is_none() {
                out.push(Diagnostic {
                    file: m.file.clone(),
                    line: s.line,
                    col: 0,
                    message: format!(
                        "skill `{}` is not in {} (a typo, or a new skill to add there)",
                        s.path,
                        Skills::FILE
                    ),
                });
            }
        }
    }
    for f in registry.fields() {
        for s in &f.skill {
            if known.get(s).is_none() {
                out.push(Diagnostic {
                    file: fields_file.to_string(),
                    line: 0,
                    col: 0,
                    message: format!("`{}`: skill `{s}` is not in {}", f.path, Skills::FILE),
                });
            }
        }
    }
    out
}

fn status(r: &Row) -> &'static str {
    match r.source {
        Some(SkillSource::Taxonomy) => "taxonomy",
        Some(SkillSource::Lessons) => "lessons only",
        Some(SkillSource::Proposed) => "proposed",
        None => "unknown",
    }
}

fn code_list(items: &[String]) -> String {
    if items.is_empty() {
        "—".to_string()
    } else {
        items
            .iter()
            .map(|s| format!("`{s}`"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The map as Markdown (docs/SKILLS.md's generated section).
pub fn markdown(map: &SkillMap) -> String {
    let mut out = String::new();
    let count = |src: Option<SkillSource>| map.rows.iter().filter(|r| r.source == src).count();
    out.push_str(&format!(
        "{} skills named: {} in Bridge-Classroom's taxonomy, {} used only by lessons, \
         {} proposed, {} unknown. {} modules declare a skill; {} card fields carry one.\n\n",
        map.rows.len(),
        count(Some(SkillSource::Taxonomy)),
        count(Some(SkillSource::Lessons)),
        count(Some(SkillSource::Proposed)),
        count(None),
        map.rows
            .iter()
            .flat_map(|r| &r.modules)
            .collect::<BTreeSet<_>>()
            .len(),
        map.rows
            .iter()
            .flat_map(|r| &r.fields)
            .collect::<BTreeSet<_>>()
            .len(),
    ));
    out.push_str("### The map\n\n");
    out.push_str("| Skill | Status | Card fields | Modules |\n|---|---|---|---|\n");
    for r in &map.rows {
        let name = r
            .name
            .as_deref()
            .map(|n| format!(" ({n})"))
            .unwrap_or_default();
        out.push_str(&format!(
            "| `{}`{name} | {} | {} | {} |\n",
            r.path,
            status(r),
            code_list(&r.fields),
            code_list(&r.modules),
        ));
    }

    out.push_str("\n### Gaps\n\n");
    out.push_str("**Skills with a card field but no module** (the card can say we play it; no rules bid it):\n\n");
    let no_module: Vec<&Row> = map
        .rows
        .iter()
        .filter(|r| !r.fields.is_empty() && r.modules.is_empty())
        .collect();
    list(
        &mut out,
        no_module
            .iter()
            .map(|r| format!("`{}`: {}", r.path, code_list(&r.fields))),
    );
    out.push_str(
        "\n**Conventions a module declares but no card field carries** (always on, or \
         switched by a field not yet tagged; `bidding_conventions` and \
         `competitive_bidding` only, since the course-level skills have no field):\n\n",
    );
    list(
        &mut out,
        map.rows
            .iter()
            .filter(|r| r.fields.is_empty() && !r.modules.is_empty())
            .filter(|r| {
                r.path.starts_with("bidding_conventions/")
                    || r.path.starts_with("competitive_bidding/")
            })
            .map(|r| format!("`{}`: {}", r.path, code_list(&r.modules))),
    );
    out.push_str("\n**Modules with no skill:**\n\n");
    list(
        &mut out,
        map.modules_without_skill.iter().map(|m| format!("`{m}`")),
    );
    out.push_str(
        "\n**Card fields tagged with a skill not in Bridge-Classroom's taxonomy** \
         (proposed, used only by lessons, or unknown):\n\n",
    );
    list(
        &mut out,
        map.rows
            .iter()
            .filter(|r| r.source != Some(SkillSource::Taxonomy) && !r.fields.is_empty())
            .map(|r| format!("`{}` ({}): {}", r.path, status(r), code_list(&r.fields))),
    );
    out.push_str("\n**Bidding skills of the taxonomy that nothing here names:**\n\n");
    list(
        &mut out,
        map.unused_taxonomy.iter().map(|s| format!("`{s}`")),
    );
    out
}

fn list(out: &mut String, items: impl Iterator<Item = String>) {
    let mut any = false;
    for i in items {
        out.push_str(&format!("- {i}\n"));
        any = true;
    }
    if !any {
        out.push_str("- none\n");
    }
}

/// `doc` with the text between [`BEGIN`] and [`END`] replaced by `text`;
/// `None` when the markers are missing.
pub fn splice(doc: &str, text: &str) -> Option<String> {
    let start = doc.find(BEGIN)? + BEGIN.len();
    let end = start + doc[start..].find(END)?;
    Some(format!("{}\n\n{}\n{}", &doc[..start], text, &doc[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_fields_and_modules_to_skills() {
        let registry = Registry::parse(concat!(
            "[notrump]\n",
            r#""stayman.play" = { kind = "bool", label = "Stayman", skill = "bidding_conventions/stayman" }"#,
            "\n",
            r#""smolen.play" = { kind = "bool", label = "Smolen", skill = "bidding_conventions/smolen" }"#,
            "\n",
            r#""x.play" = { kind = "bool", label = "X", skill = "bidding_conventions/xyz" }"#,
            "\n",
        ))
        .unwrap();
        let known = Skills::parse(concat!(
            "[taxonomy]\n",
            "\"bidding_conventions/stayman\" = \"Stayman\"\n",
            "\"bidding_conventions/ogust\" = \"Ogust\"\n",
            "\"declarer_play/finessing\" = \"Finessing\"\n",
            "[proposed]\n",
            "\"bidding_conventions/smolen\" = \"Smolen\"\n",
        ))
        .unwrap();
        let parse = |src: &str| crate::parse(src, "t.bid").unwrap();
        let modules = [
            parse("module stayman \"S\"\n  skill bidding_conventions/stayman\n"),
            parse("module base \"B\"\n"),
        ];
        let m = map(&modules, &registry, &known);
        assert_eq!(m.rows.len(), 3);
        assert_eq!(m.rows[0].path, "bidding_conventions/smolen");
        assert_eq!(m.rows[0].source, Some(SkillSource::Proposed));
        assert!(m.rows[0].modules.is_empty());
        assert_eq!(m.rows[1].fields, ["notrump.stayman.play"]);
        assert_eq!(m.rows[1].modules, ["stayman"]);
        assert_eq!(m.rows[2].source, None);
        assert_eq!(m.modules_without_skill, ["base"]);
        assert_eq!(m.unused_taxonomy, ["bidding_conventions/ogust"]);
        let md = markdown(&m);
        assert!(
            md.contains("`bidding_conventions/smolen`: `notrump.smolen.play`"),
            "{md}"
        );
        assert!(
            md.contains("| `bidding_conventions/xyz` | unknown |"),
            "{md}"
        );
        let warnings = check(&modules, &registry, &known, "fields.toml");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("bidding_conventions/xyz"));
    }

    #[test]
    fn splices_between_the_markers() {
        let doc = format!("a\n{BEGIN}\nold\n{END}\nb\n");
        let new = splice(&doc, "new\n").unwrap();
        assert_eq!(new, format!("a\n{BEGIN}\n\nnew\n\n{END}\nb\n"));
        assert_eq!(splice(&new, "new\n").unwrap(), new);
        assert!(splice("no markers", "x").is_none());
    }
}
