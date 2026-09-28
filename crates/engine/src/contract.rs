//! The engine's side of the contract with a rules directory
//! (docs/CONTRACT.md): the rule language version it reads, the check of a
//! directory's manifest, and the generated term reference in the doc.

use std::path::Path;

use bidspec::manifest::{self, Manifest};
use bidspec::Diagnostic;

/// The rule language version this engine reads. Raise it when a change to
/// the language or the term vocabulary would make existing rule files
/// load differently or not at all (docs/CONTRACT.md, "Stability").
pub const LANGUAGE_VERSION: u32 = 1;

/// The oldest language version this engine still reads.
pub const OLDEST_LANGUAGE_VERSION: u32 = 1;

/// Check a manifest's text: it parses, and asks for a language version
/// this engine reads. `file` names it in the diagnostic.
pub fn check_manifest(source: &str, file: &str) -> Result<Manifest, Diagnostic> {
    let fail = |message: String| Diagnostic {
        file: file.to_string(),
        line: 0,
        col: 0,
        message,
    };
    let m = manifest::parse(source).map_err(fail)?;
    if !(OLDEST_LANGUAGE_VERSION..=LANGUAGE_VERSION).contains(&m.language) {
        let supported = if OLDEST_LANGUAGE_VERSION == LANGUAGE_VERSION {
            format!("language {LANGUAGE_VERSION}")
        } else {
            format!("languages {OLDEST_LANGUAGE_VERSION} to {LANGUAGE_VERSION}")
        };
        let hint = if m.language > LANGUAGE_VERSION {
            "a newer engine is needed"
        } else {
            "the rules need updating to a newer language"
        };
        return Err(fail(format!(
            "`{}` is written in rule language {}, but this engine (rbb {}) reads {supported}: {hint}",
            m.name,
            m.language,
            env!("CARGO_PKG_VERSION"),
        )));
    }
    Ok(m)
}

/// The manifest of the rules directory `dir`, checked; `None` when it has
/// none. A directory without a manifest is read as the current language
/// version (`rbb bid check` warns about it).
pub fn read_manifest(dir: &Path) -> Result<Option<Manifest>, Diagnostic> {
    let path = dir.join(manifest::FILE);
    let name = path.display().to_string();
    match std::fs::read_to_string(&path) {
        Ok(src) => check_manifest(&src, &name).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Diagnostic {
            file: name,
            line: 0,
            col: 0,
            message: e.to_string(),
        }),
    }
}

/// The markers around the generated term reference in docs/CONTRACT.md.
pub const TERMS_BEGIN: &str = "<!-- BEGIN GENERATED: rbb bid terms -->";
pub const TERMS_END: &str = "<!-- END GENERATED: rbb bid terms -->";

/// `doc` with the text between the term-reference markers replaced by
/// `terms`; `None` when the markers are missing.
pub fn splice_terms(doc: &str, terms: &str) -> Option<String> {
    let start = doc.find(TERMS_BEGIN)? + TERMS_BEGIN.len();
    let end = start + doc[start..].find(TERMS_END)?;
    Some(format!("{}\n\n{}\n{}", &doc[..start], terms, &doc[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_other_language_versions() {
        let ok = check_manifest("name = \"x\"\nlanguage = 1\n", "m.toml").unwrap();
        assert_eq!(ok.language, 1);
        let newer = check_manifest(
            &format!("name = \"x\"\nlanguage = {}\n", LANGUAGE_VERSION + 1),
            "m.toml",
        )
        .unwrap_err();
        assert!(
            newer.message.contains("a newer engine is needed"),
            "{newer}"
        );
        assert_eq!(newer.file, "m.toml");
    }

    #[test]
    fn load_rules_enforces_the_manifest() {
        let dir = std::env::temp_dir().join(format!("rbb-manifest-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("card")).unwrap();
        std::fs::write(
            dir.join("demo.bid"),
            "module demo \"Demo\"\n\nafter 1N (P)\n  2C \"Stayman\" shows hcp>=8\n",
        )
        .unwrap();
        // Every rules directory brings its own card vocabulary: none, no load.
        let err = crate::load_rules(&dir).unwrap_err();
        assert!(err[0].file.ends_with("fields.toml"), "{}", err[0]);
        // An empty one is enough for rules that read no card field.
        std::fs::write(dir.join("card/fields.toml"), "").unwrap();
        std::fs::write(dir.join("card/bbsa-map.toml"), "").unwrap();
        // No manifest: read as the current language.
        assert!(crate::load_rules(&dir).unwrap().manifest.is_none());
        let manifest = dir.join(manifest::FILE);
        std::fs::write(&manifest, "name = \"demo\"\nlanguage = 1\n").unwrap();
        assert_eq!(
            crate::load_rules(&dir).unwrap().manifest.unwrap().name,
            "demo"
        );
        std::fs::write(&manifest, "name = \"demo\"\nlanguage = 99\n").unwrap();
        let err = crate::load_rules(&dir).unwrap_err();
        let vocab = crate::Vocabulary::parse("", "").unwrap();
        let also = crate::load_modules(&dir, &vocab).unwrap_err();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(err[0].message.contains("rule language 99"), "{}", err[0]);
        assert!(also[0].message.contains("rule language 99"), "{}", also[0]);
    }

    #[test]
    fn splices_between_the_markers() {
        let doc = format!("a\n{TERMS_BEGIN}\nold\n{TERMS_END}\nb\n");
        let new = splice_terms(&doc, "new\n").unwrap();
        assert_eq!(new, format!("a\n{TERMS_BEGIN}\n\nnew\n\n{TERMS_END}\nb\n"));
        assert_eq!(splice_terms(&new, "new\n").unwrap(), new);
        assert!(splice_terms("no markers", "x").is_none());
    }

    /// docs/CONTRACT.md carries the output of `rbb bid terms`. Regenerate
    /// it with `cargo run -q -p rbb-cli -- bid terms --doc docs/CONTRACT.md`.
    #[test]
    fn contract_doc_lists_the_terms() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/CONTRACT.md");
        let doc = std::fs::read_to_string(&path).expect("docs/CONTRACT.md");
        let want = splice_terms(&doc, &crate::terms_reference()).expect("markers");
        assert!(
            doc == want,
            "docs/CONTRACT.md's term reference is out of date: run \
             `cargo run -q -p rbb-cli -- bid terms --doc docs/CONTRACT.md`"
        );
    }
}
