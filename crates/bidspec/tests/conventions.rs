//! Every module in `conventions/` must compile, and bad input must give
//! useful diagnostics.

use std::fs;
use std::path::Path;

use bidspec::ast::{Alert, CallSpec, Expr, StrainSpec};

fn bid_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            bid_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "bid") {
            out.push(path);
        }
    }
}

#[test]
fn all_convention_modules_compile() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conventions");
    let mut files = Vec::new();
    bid_files(&root, &mut files);
    assert!(files.len() >= 5);
    for path in files {
        let src = fs::read_to_string(&path).unwrap();
        let name = path.display().to_string();
        match bidspec::compile(&src, &name) {
            Ok(module) => {
                // The IR round-trips through JSON.
                let json = bidspec::to_json(&module);
                let back: bidspec::Module = serde_json::from_str(&json).unwrap();
                assert_eq!(back, module, "{name}: JSON round trip");
            }
            Err(diags) => {
                let msgs: Vec<String> = diags.iter().map(|d| d.to_string()).collect();
                panic!("{}", msgs.join("\n"));
            }
        }
    }
}

#[test]
fn rkcb_structure() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conventions/slam/rkcb-1430.bid");
    let m = bidspec::compile(&fs::read_to_string(path).unwrap(), "rkcb").unwrap();
    assert_eq!(m.name, "rkcb-1430");
    assert_eq!(m.card[0].path, "slam.blackwood.rkcb_1430");
    assert_eq!(m.contexts.len(), 4);
    assert!(m.contexts.iter().all(|c| c.after.is_none()));
    let signoff = &m.contexts[2].rules;
    assert!(matches!(
        &signoff[0].call,
        CallSpec::Bid { level: 7, strain: StrainSpec::Interp(t) } if t == "t"
    ));
    assert_eq!(signoff.len(), 5);
    assert!(matches!(signoff[2].call, CallSpec::Pass));
}

#[test]
fn multi_line_rules_merge_clauses() {
    let src = "\
module demo \"Demo\"

after 1N (P)
  2C  \"Stayman\"  alert
      shows hcp>=8
      shows H=4 | S=4
      sets  forcing=round, ask=majors
      priority -1
";
    let m = bidspec::parse(src, "demo.bid").unwrap();
    let rule = &m.contexts[0].rules[0];
    assert!(matches!(rule.alert, Some(Alert::Alert { text: None })));
    assert!(matches!(&rule.shows, Some(Expr::And { all }) if all.len() == 2));
    assert_eq!(rule.sets.len(), 2);
    assert_eq!(rule.priority, Some(-1));
    assert_eq!(rule.line, 4);
}

fn errors(src: &str) -> Vec<String> {
    match bidspec::compile(src, "t.bid") {
        Ok(_) => vec![],
        Err(d) => d.iter().map(|d| d.to_string()).collect(),
    }
}

#[test]
fn diagnostics_have_locations() {
    let e = errors("module demo \"Demo\"\n\nafter 1N (P)\n  2C shows hcp>=8\n");
    assert_eq!(
        e,
        vec!["t.bid:4:5: a rule needs an explanation string after the call"]
    );

    let e = errors("module demo \"Demo\"\n\nafter 1N\n  2C \"x\"\n");
    assert!(e[0].starts_with("t.bid:3:"), "{e:?}");
    assert!(e[0].contains("RHO"), "{e:?}");

    let e = errors("module demo \"Demo\"\n\nafter 1N (P)\n  2Q \"x\"\n");
    assert_eq!(
        e,
        vec!["t.bid:4:3: 2Q: strain must be C D H S N or a variable M m x y z"]
    );

    let e = errors("module demo \"Demo\"\n  card notrump.no_such_thing\n");
    assert_eq!(e.len(), 1);
    assert!(e[0].contains("unknown card field"), "{e:?}");

    let e = errors("module demo \"Demo\"\n  card other_conventions.blackwood.rkcb_1430\n");
    assert!(e[0].contains("use `slam.blackwood.rkcb_1430`"), "{e:?}");

    let e = errors("after 1N (P)\n  2C \"x\"\n");
    assert!(e[0].contains("must start with `module"), "{e:?}");

    let e = errors("module demo \"Demo\"\n\nafter 1N (P)\n  2C \"x\"\n  shows hcp>=8\n");
    assert!(e[0].contains("must be indented under a rule"), "{e:?}");

    let e = errors("module demo \"Demo\"\n\nwhen hcp>=\n");
    assert!(e[0].starts_with("t.bid:3:"), "{e:?}");
}
