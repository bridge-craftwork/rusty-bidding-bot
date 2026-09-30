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

/// The standard card vocabulary (convention-card's `spec/`), which these
/// rules are written against.
fn vocab() -> &'static bridge_card::Vocabulary {
    static V: std::sync::OnceLock<bridge_card::Vocabulary> = std::sync::OnceLock::new();
    V.get_or_init(|| bridge_card::standard::vocabulary().unwrap_or_else(|e| panic!("{e}")))
}

/// The standard `fields.toml` and `bbsa-map.toml` parse, every mapped
/// path is a field and every value fits it (checked when they load), and
/// every mapped key is one of BBA's.
#[test]
fn the_card_vocabulary_is_valid() {
    let v = vocab();
    assert!(v.registry().fields().len() > 250);
    assert!(v.bbsa_mapping().len() > 130);
    assert_eq!(v.lint(), Vec::<String>::new());
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
        match bidspec::compile(&src, &name, vocab().registry()) {
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
    let m = bidspec::compile(
        &fs::read_to_string(path).unwrap(),
        "rkcb",
        vocab().registry(),
    )
    .unwrap();
    assert_eq!(m.name, "rkcb-1430");
    // It serves 1430 and 0314, so the card fields are params, not gates.
    assert!(m.card.is_empty());
    assert!(m.params.iter().any(|p| p.name == "k0314"));
    // Every context is keyed on state, not on an auction: that is the
    // point of the module. The blocks themselves are found by what they
    // contain, so restructuring the file does not break this test.
    assert!(m.contexts.iter().all(|c| c.after.is_none()));
    let signoff = m
        .contexts
        .iter()
        .map(|c| &c.rules)
        .find(|rules| {
            matches!(
                &rules[0].call,
                CallSpec::Bid { level: 7, strain: StrainSpec::Interp(t) } if t == "t"
            )
        })
        .expect("the block that answers our keycard ask, grand slam first");
    assert!(
        signoff.iter().any(|r| matches!(r.call, CallSpec::Pass)),
        "and it can stop"
    );
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
    match bidspec::compile(src, "t.bid", vocab().registry()) {
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

/// docs/SKILLS.md carries the output of `rbb bid skills`, and every skill
/// a module or field names is a standard convention or skill
/// (convention-card's `spec/conventions/`).
/// Regenerate with `cargo run -q -p rbb-cli -- bid skills --doc docs/SKILLS.md`.
#[test]
fn skills_doc_carries_the_map() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conventions");
    let known = bridge_card::standard::conventions().unwrap_or_else(|e| panic!("{e}"));
    let mut files = Vec::new();
    bid_files(&root, &mut files);
    files.sort();
    let modules: Vec<bidspec::Module> = files
        .iter()
        .map(|p| {
            let src = fs::read_to_string(p).unwrap();
            bidspec::compile(&src, &p.display().to_string(), vocab().registry()).unwrap()
        })
        .collect();
    let unknown = bidspec::skills::check(&modules, vocab().registry(), &known, "fields.toml");
    assert!(unknown.is_empty(), "{unknown:?}");
    let map = bidspec::skills::map(&modules, vocab().registry(), &known);
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/SKILLS.md");
    // A Windows checkout without .gitattributes has CRLF line ends.
    let doc = fs::read_to_string(&path)
        .expect("docs/SKILLS.md")
        .replace("\r\n", "\n");
    let want = bidspec::skills::splice(&doc, &bidspec::skills::markdown(&map)).expect("markers");
    assert!(
        doc == want,
        "docs/SKILLS.md's skill map is out of date: run \
         `cargo run -q -p rbb-cli -- bid skills --doc docs/SKILLS.md`"
    );
}

#[test]
fn modules_name_their_teaching_skills() {
    let src = "module demo \"Demo\"\n  card notrump.stayman.play\n  skill bidding_conventions/stayman\n  skill partnership_bidding/stayman_transfers precision/1c_opener\n\nafter 1N (P)\n  2C \"x\"\n";
    let m = bidspec::compile(src, "t.bid", vocab().registry()).unwrap();
    let paths: Vec<(&str, usize)> = m.skills.iter().map(|s| (s.path.as_str(), s.line)).collect();
    assert_eq!(
        paths,
        [
            ("bidding_conventions/stayman", 3),
            ("partnership_bidding/stayman_transfers", 4),
            ("precision/1c_opener", 4)
        ]
    );
    let json = bidspec::to_json(&m);
    assert!(json.contains("\"skills\""), "{json}");

    let e = errors("module demo \"Demo\"\n  skill Stayman\n");
    assert!(e[0].starts_with("t.bid:2:"), "{e:?}");
    assert!(e[0].contains("not a skill path"), "{e:?}");
    let e = errors("module demo \"Demo\"\n  skill\n");
    assert!(e[0].contains("expected `skill"), "{e:?}");
    let e = errors("module demo \"Demo\"\nskill bidding_conventions/stayman\n");
    assert!(e[0].contains("must be indented under `module`"), "{e:?}");
    // A `/` is only for skill paths.
    let e = errors("module demo \"Demo\"\n\nwhen hcp / 2 >= 4\n");
    assert!(e[0].starts_with("t.bid:3:"), "{e:?}");
}
