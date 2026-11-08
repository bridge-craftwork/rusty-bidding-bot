//! `--rules-at REF`: the rules directory as it was at a git commit, read
//! from the repository instead of the working tree.

use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        ok.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&ok.stderr)
    );
}

fn module(text: &str) -> String {
    format!(
        "module tiny \"Tiny\"\n  kind system\n\nwhen opening\n  1C  \"{text}\"  shows hcp>=12\n  P   \"No opening\"\n"
    )
}

/// A repository with three commits of `conventions/`: v1, v2, and a
/// manifest in a rule language no engine reads. The working tree holds
/// v3 (uncommitted).
fn repo(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rbb-rules-at-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("conventions")).unwrap();
    git(&dir, &["init", "-q"]);
    let write = |f: &str, t: &str| std::fs::write(dir.join("conventions").join(f), t).unwrap();
    write("tiny.bid", &module("VERSION-ONE"));
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "v1"]);
    git(&dir, &["tag", "v1"]);
    write("tiny.bid", &module("VERSION-TWO"));
    git(&dir, &["commit", "-q", "-am", "v2"]);
    git(&dir, &["tag", "v2"]);
    write(
        "conventions.toml",
        "name = \"future\"\ndescription = \"x\"\nlanguage = 999\n",
    );
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "future"]);
    git(&dir, &["tag", "future"]);
    std::fs::remove_file(dir.join("conventions/conventions.toml")).unwrap();
    write("tiny.bid", &module("VERSION-THREE"));
    dir
}

fn rbb(dir: &Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_rbb"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn rules_come_from_the_commit_not_the_working_tree() {
    let dir = repo("commit");
    let rules = |extra: &[&str]| {
        let mut a = vec!["bid", "rules", "--no-knowledge", "-c", "21GF-DEFAULT"];
        a.extend_from_slice(extra);
        let (ok, out, err) = rbb(&dir, &a);
        assert!(ok, "{err}");
        out
    };
    assert!(rules(&[]).contains("VERSION-THREE"));
    let v1 = rules(&["--rules-at", "v1"]);
    assert!(v1.contains("VERSION-ONE"), "{v1}");
    // Named as the working tree's files are, so runs compare line by line.
    assert!(v1.contains("conventions/tiny.bid:5 (tiny)"), "{v1}");
    assert!(rules(&["--rules-at", "v2"]).contains("VERSION-TWO"));

    // `call` and `explain-auction` take it too.
    let card = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../cards/bbsa/21GF-DEFAULT.bbsa"
    );
    let (ok, out, err) = rbb(
        &dir,
        &["call", "AK52.KJ7.Q94.K83", "-c", card, "--rules-at", "v1"],
    );
    assert!(ok, "{err}");
    assert!(
        out.lines().next().unwrap().ends_with(": VERSION-ONE"),
        "{out}"
    );
    let (ok, out, err) = rbb(
        &dir,
        &[
            "explain-auction",
            "-a",
            "1C",
            "-c",
            "21GF-DEFAULT",
            "--rules-at",
            "v2",
        ],
    );
    assert!(ok, "{err}");
    assert!(out.contains("VERSION-TWO"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_ref_in_an_unreadable_language_or_no_ref_is_refused() {
    let dir = repo("refuse");
    let (ok, _, err) = rbb(
        &dir,
        &["bid", "rules", "-c", "21GF-DEFAULT", "--rules-at", "future"],
    );
    assert!(!ok);
    assert!(err.contains("--rules-at future"), "{err}");
    assert!(err.contains("rule language 999"), "{err}");
    let (ok, _, err) = rbb(
        &dir,
        &[
            "bid",
            "rules",
            "-c",
            "21GF-DEFAULT",
            "--rules-at",
            "no-such-ref",
        ],
    );
    assert!(!ok);
    assert!(err.contains("not a commit"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bid_pbn_takes_it_and_writes_json_lines() {
    let dir = repo("pbn");
    let pbn = dir.join("in.pbn");
    std::fs::write(
        &pbn,
        "[Board \"1\"]\n[Dealer \"N\"]\n[Vulnerable \"None\"]\n\
         [Deal \"N:AK52.KJ7.Q94.K83 QJ3.Q95.KJ3.QJ74 T64.AT832.A2.T62 987.64.T8765.A95\"]\n\n",
    )
    .unwrap();
    let (ok, out, err) = rbb(
        &dir,
        &[
            "bid-pbn",
            "-i",
            "in.pbn",
            "-o",
            "-",
            "--ns-card",
            "21GF-DEFAULT",
            "--rules-at",
            "v1",
            "--json",
            "--all-meanings",
        ],
    );
    assert!(ok, "{err}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1, "{out}");
    let v: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(v["board"], "1");
    assert_eq!(v["dealer"], "N");
    assert_eq!(v["auction"][0], "1C");
    assert_eq!(v["calls"][0]["rule"], "conventions/tiny.bid:5");
    assert_eq!(v["calls"][0]["meaning"], "VERSION-ONE");
    assert_eq!(v["contract"], "1C");
    assert_eq!(v["declarer"], "N");
    // No table in the file and no --par: no `dd`.
    assert!(v.get("dd").is_none(), "{v}");
    let _ = std::fs::remove_dir_all(&dir);
}
