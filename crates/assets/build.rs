//! Generates `$OUT_DIR/assets.rs`: every `conventions/**/*.bid` file and every
//! stock card `crates/bridge-card/tests/fixtures/bbsa/*.bbsa`, as
//! `include_str!`s, so a binary (the release `rbb`, the WASM build) carries
//! its rules and needs no filesystem to load them.
//!
//! Rule files are named and ordered exactly as `rbb_engine::load_modules`
//! names and orders them when given the directory `conventions`: the path
//! `conventions/<dir>/<file>.bid`, sorted by path components. File order is
//! the engine's last tie-breaker, so the embedded rules bid exactly like the
//! files on disk.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, ext, out);
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
}

/// `path` relative to `root`, with `/` separators on every platform.
fn relative(path: &Path, root: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn slashed(p: &Path) -> String {
    p.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// FNV-1a over the names and contents: identifies a build of the rules.
fn fnv(h: &mut u64, bytes: &[u8]) {
    for b in bytes {
        *h ^= *b as u64;
        *h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.join("../..").canonicalize().expect("workspace root");
    let rules_dir = root.join("conventions");
    let cards_dir = root.join("crates/bridge-card/tests/fixtures/bbsa");
    // A directory is scanned recursively for changes.
    println!("cargo:rerun-if-changed={}", rules_dir.display());
    println!("cargo:rerun-if-changed={}", cards_dir.display());

    let mut rules = Vec::new();
    collect(&rules_dir, "bid", &mut rules);
    // As load_modules sorts: by path, component by component.
    let mut rules: Vec<(PathBuf, PathBuf)> = rules
        .into_iter()
        .map(|p| (relative(&p, &root), p))
        .collect();
    rules.sort();
    assert!(!rules.is_empty(), "no .bid files under {}", rules_dir.display());

    let mut cards = Vec::new();
    collect(&cards_dir, "bbsa", &mut cards);
    cards.sort();

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut out = String::new();
    out.push_str("/// Every rule file: (name as `rbb_engine::load_modules` gives it, source).\n");
    out.push_str("pub static RULE_FILES: &[(&str, &str)] = &[\n");
    for (rel, abs) in &rules {
        let name = slashed(rel);
        fnv(&mut hash, name.as_bytes());
        fnv(&mut hash, &std::fs::read(abs).expect("read rule file"));
        println!("cargo:rerun-if-changed={}", abs.display());
        writeln!(out, "    ({name:?}, include_str!({:?})),", abs.display().to_string()).unwrap();
    }
    out.push_str("];\n\n");
    out.push_str("/// Stock cards: (name, .bbsa text).\n");
    out.push_str("pub static CARDS: &[(&str, &str)] = &[\n");
    for abs in &cards {
        let name = abs.file_stem().unwrap().to_string_lossy().into_owned();
        println!("cargo:rerun-if-changed={}", abs.display());
        writeln!(out, "    ({name:?}, include_str!({:?})),", abs.display().to_string()).unwrap();
    }
    out.push_str("];\n\n");
    writeln!(
        out,
        "/// Identifies this build of the rules (FNV-1a of the rule files' names and text).\npub const RULES_ID: &str = \"{hash:016x}\";"
    )
    .unwrap();

    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("assets.rs");
    std::fs::write(dest, out).unwrap();
}
