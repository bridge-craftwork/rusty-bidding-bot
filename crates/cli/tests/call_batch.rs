//! `rbb call --batch`: JSON lines in, one decision a line out, in order,
//! the same calls as one `rbb call` each.

use std::io::Write;
use std::process::{Command, Stdio};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn rbb(args: &[&str], stdin: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rbb"))
        .args(args)
        .current_dir(ROOT)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

fn call_of(v: &serde_json::Value) -> String {
    match &v["call"] {
        serde_json::Value::String(s) => s.clone(),
        b => format!(
            "{}{}",
            b["Bid"]["level"],
            b["Bid"]["strain"].as_str().unwrap()
        ),
    }
}

#[test]
fn batch_answers_each_line_in_order() {
    let input = [
        r#"{"hand": "AQ5.KJ3.Q984.K32", "id": 1}"#,
        r#"{"hand": "K864.Q752.J3.A54", "auction": "1NT Pass", "id": "two"}"#,
        "",
        "not json",
        r#"{"hand": "AK", "id": 4}"#,
        r#"{"hand": "KT7.KQ6.AK764.T2", "auction": "1H X XX", "dealer": "S", "vul": "All"}"#,
    ]
    .join("\n");
    let out = rbb(
        &["call", "--batch", "-c", "cards/bbsa/21GF-GIB.bbsa"],
        &input,
    );
    let rows: Vec<serde_json::Value> = out
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    // The blank line is skipped; every other line answers.
    assert_eq!(rows.len(), 5, "{out}");
    assert_eq!(rows[0]["id"], 1);
    assert_eq!(call_of(&rows[0]), "1NoTrump");
    assert_eq!(rows[1]["id"], "two");
    assert_eq!(call_of(&rows[1]), "2Clubs");
    assert!(rows[2]["error"].as_str().unwrap().starts_with("not JSON"));
    assert_eq!(rows[3]["id"], 4);
    assert!(rows[3]["error"]
        .as_str()
        .unwrap()
        .starts_with("hand must be"));
    assert!(rows[4].get("id").is_none());

    // The same answer as one call.
    let one = rbb(
        &[
            "call",
            "-c",
            "cards/bbsa/21GF-GIB.bbsa",
            "-a",
            "1H X XX",
            "-d",
            "S",
            "-v",
            "All",
            "--json",
            "KT7.KQ6.AK764.T2",
        ],
        "",
    );
    let one: serde_json::Value = serde_json::from_str(&one).unwrap();
    assert_eq!(call_of(&rows[4]), call_of(&one));
}
