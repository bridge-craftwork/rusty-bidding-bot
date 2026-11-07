//! `rbb bid rules`: the rules that can make the next call after an
//! auction, asked of the engine.

use std::process::Command;

use serde_json::Value;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn rbb(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_rbb"))
        .args(args)
        .current_dir(ROOT)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "rbb {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn rules_json(args: &[&str]) -> Value {
    let mut all = vec![
        "bid",
        "rules",
        "--json",
        "-c",
        "cards/bbsa/21GF-DEFAULT.bbsa",
    ];
    all.extend_from_slice(args);
    serde_json::from_str(&rbb(&all)).unwrap()
}

/// (call, module) of each candidate, best-ranked first.
fn calls(v: &Value) -> Vec<(String, String)> {
    v["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["call"].as_str().unwrap().to_string(),
                c["rule"]["module"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn has(v: &Value, call: &str, module: &str) -> bool {
    calls(v).iter().any(|(c, m)| c == call && m == module)
}

#[test]
fn after_1nt_pass_lists_stayman_and_transfers_by_the_card() {
    let v = rules_json(&["-a", "1N P"]);
    assert_eq!(v["seat"], "South");
    assert!(has(&v, "2C", "stayman"), "{:?}", calls(&v));
    assert!(has(&v, "2D", "jacoby-transfers"), "{:?}", calls(&v));
    assert!(has(&v, "2H", "jacoby-transfers"), "{:?}", calls(&v));
    // Ranks are the engine's order: 1, 2, 3, ...
    for (i, c) in v["candidates"].as_array().unwrap().iter().enumerate() {
        assert_eq!(c["rank"], i + 1);
    }
    // The card decides: without Jacoby transfers, 2D is not a transfer.
    let off = rules_json(&["-a", "1N P", "--set", "notrump.jacoby_transfers.play=false"]);
    assert!(!has(&off, "2D", "jacoby-transfers"), "{:?}", calls(&off));
    assert!(has(&off, "2C", "stayman"));
    // The hand conditions stay conditions, as written and as read.
    let stayman = v["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["call"] == "2C" && c["rule"]["module"] == "stayman")
        .unwrap();
    assert!(stayman["shows"]
        .as_str()
        .unwrap()
        .contains("strength>=invite"));
    assert!(stayman["shows_resolved"]
        .as_str()
        .unwrap()
        .contains("points>="));
    assert_eq!(stayman["artificial"], true);
    assert_eq!(stayman["read_as"], true);
    assert!(stayman["sets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s == "forcing=round"));
}

#[test]
fn contested_auction_binds_pattern_variables() {
    let v = rules_json(&["-a", "1C P 1D 1H"]);
    assert_eq!(v["seat"], "North");
    // Opener's raise of partner's diamonds: `2z` bound to 2D.
    let raise = v["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["call"] == "2D" && c["rule"]["module"] == "rebids")
        .expect("2D raise");
    assert_eq!(raise["bindings"]["z"], "D");
    assert_eq!(raise["shows_resolved"].as_str().unwrap(), "D>=4, tp(D)<=15");
    // The same call from another module is marked, and only one of them is
    // how partner reads it.
    let others = raise["same_call"].as_array().unwrap();
    assert!(others.iter().any(|r| r["module"] == "total-tricks"));
    let twod: Vec<&Value> = v["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["call"] == "2D")
        .collect();
    assert_eq!(twod.iter().filter(|c| c["read_as"] == true).count(), 1);
    // The knowledge so far: one row per call.
    assert_eq!(v["knowledge"].as_array().unwrap().len(), 4);
}

#[test]
fn read_as_is_the_rule_the_engine_reads_the_call_by() {
    let v = rules_json(&["-a", "1N P"]);
    let review: Value = serde_json::from_str(&rbb(&[
        "explain-auction",
        "--json",
        "-c",
        "cards/bbsa/21GF-DEFAULT.bbsa",
        "-a",
        "1N P 2C",
    ]))
    .unwrap();
    let read = &review.as_array().unwrap()[2]["step"]["rule"];
    let listed = v["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["call"] == "2C" && c["read_as"] == true)
        .unwrap();
    assert_eq!(&listed["rule"], read);
}

#[test]
fn call_filter_keeps_one_call() {
    let v = rules_json(&["-a", "1N P", "--call", "2C"]);
    let c = calls(&v);
    assert!(!c.is_empty());
    assert!(c.iter().all(|(call, _)| call == "2C"), "{c:?}");
    // Ranks stay the engine's, not renumbered.
    let all = rules_json(&["-a", "1N P"]);
    let rank = all["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["call"] == "2C")
        .unwrap()["rank"]
        .clone();
    assert_eq!(v["candidates"][0]["rank"], rank);
}

#[test]
fn all_shows_a_failed_when_with_its_reason() {
    // Under BBA's style the standard Stayman rule's `style is not bba`
    // fails; the BBA one applies.
    let v = rules_json(&[
        "-a",
        "1N P",
        "--call",
        "2C",
        "--all",
        "--set",
        "general.style=bba",
    ]);
    let failed = v["rejected"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["stage"] == "when" && r["rule"]["module"] == "stayman")
        .expect("a stayman rule rejected by its when");
    assert_eq!(failed["call"], "2C");
    assert!(
        failed["reason"]
            .as_str()
            .unwrap()
            .contains("`style is not bba` is false"),
        "{failed}"
    );
    // A rule whose `after` does not match says which line.
    let none = rules_json(&["-a", "1N P", "--all"]);
    assert!(none["rejected"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["stage"] == "after"
            && r["reason"]
                .as_str()
                .unwrap()
                .contains("no alternative matches")));
    // Without --all there are none.
    assert!(rules_json(&["-a", "1N P"])["rejected"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn forced_pass_is_marked() {
    // Stayman forces opener: a natural pass is out whatever the hand.
    let v = rules_json(&["-a", "1N P 2C P"]);
    assert_eq!(v["forced"], true);
    for c in v["candidates"].as_array().unwrap() {
        if c["call"] == "Pass" && c["artificial"] == false {
            assert_eq!(c["excluded"], "partner's call is forcing");
        }
    }
}

#[test]
fn json_shape() {
    let v = rules_json(&["-a", "1C P 1D 1H"]);
    for key in [
        "seat",
        "forced",
        "auction",
        "dealer",
        "candidates",
        "rejected",
        "inactive",
        "warnings",
        "knowledge",
    ] {
        assert!(v.get(key).is_some(), "no {key}");
    }
    assert_eq!(v["auction"], serde_json::json!(["1C", "Pass", "1D", "1H"]));
    let c = &v["candidates"][0];
    for key in [
        "rank",
        "call",
        "rule",
        "explanation",
        "alert",
        "artificial",
        "priority",
        "descriptiveness",
        "order",
        "bindings",
        "shows",
        "shows_resolved",
        "when",
        "when_folded",
        "when_reads_hand",
        "denies",
        "prefer",
        "sets",
        "read_as",
        "same_call",
        "excluded",
    ] {
        assert!(c.get(key).is_some(), "candidate has no {key}: {c}");
    }
    for key in ["module", "file", "line"] {
        assert!(c["rule"].get(key).is_some());
    }
    // --no-knowledge leaves the review out.
    let bare = rules_json(&["-a", "1C P 1D 1H", "--no-knowledge"]);
    assert!(bare.get("knowledge").is_none());
}

#[test]
fn text_form_prints_the_review_and_the_rules() {
    let out = rbb(&[
        "bid",
        "rules",
        "-c",
        "cards/bbsa/21GF-DEFAULT.bbsa",
        "-a",
        "1N P",
        "--call",
        "2C",
    ]);
    assert!(out.contains("South to call"), "{out}");
    assert!(out.contains("stayman.bid"), "{out}");
    assert!(out.contains("shows:"), "{out}");
    // The review of the auction so far comes first.
    assert!(out.find("1NT").unwrap() < out.find("South to call").unwrap());
}

fn selection(v: &Value) -> &Vec<Value> {
    v["selection"].as_array().expect("a selection")
}

#[test]
fn rule_that_applies_reports_its_entry() {
    let v = rules_json(&["-a", "1C P 1D 1H", "--rule", "rebids.bid:725"]);
    let s = selection(&v);
    assert_eq!(s.len(), 1);
    assert_eq!(s[0]["verdict"], "applies");
    assert_eq!(s[0]["rule"]["line"], 725);
    let c = &s[0]["candidates"][0];
    assert_eq!(c["call"], "2D");
    assert_eq!(c["bindings"]["z"], "D");
    assert!(c["rank"].as_u64().unwrap() >= 1);
    // The listing keeps its shape, narrowed to the selection.
    assert!(v["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["rule"]["line"] == 725));
}

#[test]
fn rule_rejected_by_after_says_which_line() {
    let v = rules_json(&["-a", "1C P 1D 1H", "--rule", "base/rebids.bid:57"]);
    let s = &selection(&v)[0];
    assert_eq!(s["verdict"], "fails");
    assert_eq!(s["rejected"][0]["stage"], "after");
    assert!(s["reasons"][0]
        .as_str()
        .unwrap()
        .contains("`1x (P) 1y (P)`: no alternative matches"));
}

#[test]
fn rule_rejected_by_when_names_the_conjunct() {
    let v = rules_json(&[
        "-a",
        "1N P",
        "--set",
        "general.style=bba",
        "--rule",
        "stayman.bid:19",
    ]);
    let s = &selection(&v)[0];
    assert_eq!(s["verdict"], "fails");
    assert_eq!(s["rejected"][0]["stage"], "when");
    assert!(s["reasons"][0]
        .as_str()
        .unwrap()
        .contains("`style is not bba` is false"));
    // Without --all, the text form still reports it.
    let text = rbb(&[
        "bid",
        "rules",
        "-c",
        "cards/bbsa/21GF-DEFAULT.bbsa",
        "-a",
        "1N P",
        "--set",
        "general.style=bba",
        "--no-knowledge",
        "--rule",
        "stayman.bid:19",
    ]);
    assert!(text.contains("DOES NOT APPLY"), "{text}");
}

#[test]
fn module_selection_and_inactive_module() {
    let v = rules_json(&["-a", "1N P", "--rule", "jacoby-transfers", "--call", "2D"]);
    let s = selection(&v);
    assert!(s.len() > 2);
    assert!(s.iter().all(|x| x["rule"]["module"] == "jacoby-transfers"));
    assert!(s
        .iter()
        .any(|x| x["verdict"] == "applies" && x["candidates"][0]["call"] == "2D"));
    let off = rules_json(&[
        "-a",
        "1N P",
        "--set",
        "notrump.jacoby_transfers.play=false",
        "--rule",
        "jacoby-transfers:19",
    ]);
    let s = &selection(&off)[0];
    assert_eq!(s["verdict"], "inactive");
    assert!(s["reasons"][0].as_str().unwrap().contains("is off"));
}

#[test]
fn bad_rule_spec_lists_near_matches() {
    for (spec, near) in [("rebids.bid:726", "rebids.bid:725"), ("stayma", "stayman")] {
        let out = Command::new(env!("CARGO_BIN_EXE_rbb"))
            .args(["bid", "rules", "-a", "1N P", "--rule", spec])
            .current_dir(ROOT)
            .output()
            .unwrap();
        assert!(!out.status.success());
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains(near), "{spec}: {err}");
    }
}
