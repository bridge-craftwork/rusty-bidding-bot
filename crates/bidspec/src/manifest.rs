//! The manifest at the root of a rules directory (`conventions.toml`):
//! what the rule set is called and which version of the rule language it
//! is written in. The engine decides which language versions it reads
//! (docs/CONTRACT.md); this module only reads the file.
//!
//! ```
//! let m = bidspec::manifest::parse("name = \"demo\"\nlanguage = 1\n").unwrap();
//! assert_eq!(m.language, 1);
//! ```

use serde::Serialize;

/// The manifest's file name, at the root of a rules directory.
pub const FILE: &str = "conventions.toml";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Manifest {
    pub name: String,
    pub description: Option<String>,
    /// The rule language version the files are written in.
    pub language: u32,
    /// The engine version the rules were developed against (information
    /// only).
    pub engine: Option<String>,
    /// Keys this version of the manifest does not know.
    pub unknown_keys: Vec<String>,
}

/// Read a manifest. Errors name the key at fault.
pub fn parse(source: &str) -> Result<Manifest, String> {
    let table: toml::Table = toml::from_str(source).map_err(|e| e.message().to_string())?;
    let text = |key: &str| -> Result<Option<String>, String> {
        match table.get(key) {
            None => Ok(None),
            Some(toml::Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(format!("`{key}` must be a string")),
        }
    };
    let name = text("name")?.ok_or("`name` is missing")?;
    let language = match table.get("language") {
        None => return Err("`language` is missing (the rule language version, e.g. 1)".into()),
        Some(toml::Value::Integer(n)) if *n >= 1 && *n <= u32::MAX as i64 => *n as u32,
        Some(_) => return Err("`language` must be a whole number from 1".into()),
    };
    Ok(Manifest {
        name,
        description: text("description")?,
        language,
        engine: text("engine")?,
        unknown_keys: table
            .keys()
            .filter(|k| !matches!(k.as_str(), "name" | "description" | "language" | "engine"))
            .cloned()
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_fields() {
        let m = parse(
            "name = \"x\"\ndescription = \"d\"\nlanguage = 2\nengine = \"0.1.0\"\nextra = 1\n",
        )
        .unwrap();
        assert_eq!(m.name, "x");
        assert_eq!(m.description.as_deref(), Some("d"));
        assert_eq!(m.language, 2);
        assert_eq!(m.engine.as_deref(), Some("0.1.0"));
        assert_eq!(m.unknown_keys, vec!["extra".to_string()]);
    }

    #[test]
    fn refuses_a_bad_manifest() {
        assert!(parse("language = 1").unwrap_err().contains("name"));
        assert!(parse("name = \"x\"").unwrap_err().contains("language"));
        assert!(parse("name = \"x\"\nlanguage = \"1\"").is_err());
        assert!(parse("name = \"x\"\nlanguage = 0").is_err());
        assert!(parse("name = ").is_err());
    }
}
