//! Scenarios in a Practice-Bidding-Scenarios checkout: `bba/<name>.pbn`
//! bid by BBA, and `btn/<name>.btn` naming the cards each side played.

use std::path::{Path, PathBuf};

/// Cards the PBS build uses when a `.btn` names none.
const DEFAULT_NS: &str = "21GF-DEFAULT";
const DEFAULT_EW: &str = "21GF-GIB";

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    pub pbn: PathBuf,
    /// Card names: files `bbsa/<name>.bbsa`.
    pub ns_card: String,
    pub ew_card: String,
}

/// Every scenario under `pbs/bba`, or just those named in `only`.
pub fn discover(pbs: &Path, only: &[String]) -> Result<Vec<Scenario>, String> {
    let dir = pbs.join("bba");
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "pbn") {
            continue;
        }
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if !only.is_empty() && !only.contains(&name) {
            continue;
        }
        let (ns_card, ew_card) = cards_for(&pbs.join("btn").join(format!("{name}.btn")));
        out.push(Scenario {
            name,
            pbn: path,
            ns_card,
            ew_card,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    if !only.is_empty() {
        for want in only {
            if !out.iter().any(|s| &s.name == want) {
                return Err(format!("no scenario {want:?} in {}", dir.display()));
            }
        }
    }
    Ok(out)
}

/// The `# convention-card-ns:` / `-ew:` lines of a `.btn`.
fn cards_for(btn: &Path) -> (String, String) {
    let text = std::fs::read_to_string(btn).unwrap_or_default();
    let find = |key: &str| {
        text.lines()
            .find_map(|l| l.trim().strip_prefix(key).map(|v| v.trim().to_string()))
            .filter(|v| !v.is_empty())
    };
    (
        find("# convention-card-ns:").unwrap_or_else(|| DEFAULT_NS.into()),
        find("# convention-card-ew:").unwrap_or_else(|| DEFAULT_EW.into()),
    )
}
