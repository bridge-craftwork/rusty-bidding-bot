//! `--rules-at REF`: the rule set as it was at a git commit, read from the
//! repository in memory (`git ls-tree`, `git cat-file --batch`), so a
//! before/after comparison needs no copied rules directory.
//!
//! The files are named as `load_rules` names them from the working tree
//! (`conventions/base/base.bid`), and compiled in the same order (sorted by
//! path), so the rule references in a run's output match a run on the
//! working tree's rules.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rbb_engine::RuleSet;

use crate::Result;

fn git(args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr)
            .trim()
            .to_string()
            .into());
    }
    Ok(String::from_utf8(out.stdout)?)
}

/// The blobs `specs` (`<commit>:<path>`) name, in order, read with one
/// `git cat-file --batch`.
fn blobs(specs: &[String]) -> Result<Vec<Vec<u8>>> {
    let mut child = Command::new("git")
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git cat-file: {e}"))?;
    let mut stdin = child.stdin.take().unwrap();
    let input: String = specs.iter().map(|s| format!("{s}\n")).collect();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let mut raw = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut raw)?;
    writer.join().unwrap()?;
    child.wait()?;
    let mut out = Vec::with_capacity(specs.len());
    let mut at = 0;
    for spec in specs {
        let nl = raw[at..]
            .iter()
            .position(|&b| b == b'\n')
            .ok_or("git cat-file: short output")?;
        let header = String::from_utf8_lossy(&raw[at..at + nl]).to_string();
        at += nl + 1;
        let size: usize = match header.rsplit_once(' ') {
            Some((_, n)) if header.contains(" blob ") => n.parse()?,
            _ => return Err(format!("git cat-file {spec}: {header}").into()),
        };
        out.push(raw[at..at + size].to_vec());
        at += size + 1;
    }
    Ok(out)
}

/// The rules directory `dir` (relative to the current directory, inside a
/// git repository) as it was at `git_ref`. Refuses a ref that is not a
/// commit, a directory with no rule files there, and a manifest whose rule
/// language this engine does not read.
pub fn load(git_ref: &str, dir: &Path) -> Result<RuleSet> {
    let fail =
        |m: String| -> Box<dyn std::error::Error> { format!("--rules-at {git_ref}: {m}").into() };
    let commit = git(&[
        "rev-parse",
        "--verify",
        "--quiet",
        &format!("{git_ref}^{{commit}}"),
    ])
    .map_err(|_| fail("not a commit in this repository".into()))?;
    let commit = commit.trim().to_string();
    let prefix = git(&["rev-parse", "--show-prefix"]).map_err(|e| fail(e.to_string()))?;
    let dir_text = dir.to_string_lossy().trim_end_matches('/').to_string();
    let listed = git(&[
        "ls-tree",
        "-r",
        "--full-name",
        "--name-only",
        "-z",
        &commit,
        "--",
        &format!("{dir_text}/"),
    ])
    .map_err(|e| fail(e.to_string()))?;
    // In-repo path (from the root) -> path relative to `dir`.
    let root_dir = format!("{}{dir_text}/", prefix.trim());
    let files: Vec<(String, PathBuf)> = listed
        .split('\0')
        .filter(|p| !p.is_empty())
        .filter_map(|p| {
            p.strip_prefix(&root_dir)
                .map(|rel| (p.to_string(), PathBuf::from(rel)))
        })
        .collect();
    let wanted = |rel: &Path| {
        rel.extension().is_some_and(|x| x == "bid")
            || rel == Path::new(bidspec::manifest::FILE)
            || rel == Path::new(rbb_engine::Vocabulary::FIELDS)
            || rel == Path::new(rbb_engine::Vocabulary::BBSA_MAP)
    };
    let mut files: Vec<(String, PathBuf)> = files.into_iter().filter(|(_, r)| wanted(r)).collect();
    // The order `load_rules` compiles in: sorted by path.
    files.sort_by_key(|f| dir.join(&f.1));
    if !files
        .iter()
        .any(|(_, r)| r.extension().is_some_and(|x| x == "bid"))
    {
        return Err(fail(format!(
            "no .bid files under {dir_text}/ at {commit:.12}"
        )));
    }
    let specs: Vec<String> = files.iter().map(|(p, _)| format!("{commit}:{p}")).collect();
    let texts = blobs(&specs).map_err(|e| fail(e.to_string()))?;
    let mut sources: Vec<(String, String)> = Vec::new();
    let (mut manifest, mut fields, mut bbsa_map) = (None, None, None);
    for ((_, rel), bytes) in files.iter().zip(texts) {
        // Named with `/` as `load_modules` names the working tree's files.
        let mut name = dir.join(rel).display().to_string();
        if cfg!(windows) {
            name = name.replace('\\', "/");
        }
        let text = String::from_utf8(bytes).map_err(|_| fail(format!("{name}: not UTF-8")))?;
        if rel == Path::new(bidspec::manifest::FILE) {
            manifest = Some((name, text));
        } else if rel == Path::new(rbb_engine::Vocabulary::FIELDS) {
            fields = Some(text);
        } else if rel == Path::new(rbb_engine::Vocabulary::BBSA_MAP) {
            bbsa_map = Some(text);
        } else {
            sources.push((name, text));
        }
    }
    let manifest = manifest
        .map(|(name, text)| rbb_engine::check_manifest(&text, &name))
        .transpose()
        .map_err(|d| fail(d.to_string()))?;
    let vocab = match fields {
        Some(f) => rbb_engine::Vocabulary::parse(&f, bbsa_map.as_deref().unwrap_or(""))
            .map_err(|e| fail(e.to_string()))?,
        None => bridge_card::standard::vocabulary().map_err(|e| fail(e.to_string()))?,
    };
    let modules = rbb_engine::compile_modules(
        &vocab,
        sources.iter().map(|(n, s)| (n.as_str(), s.as_str())),
    )
    .map_err(|d| fail(crate::diagnostics(d)))?;
    Ok(RuleSet {
        manifest,
        vocab,
        modules,
    })
}
