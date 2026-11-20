//! The rule language on small sources: what parses, and that bad input
//! gives useful diagnostics. The real rule files are a convention pack's;
//! the engine's tests compile the compiled-in copy.

use bidspec::ast::{Alert, CallSpec, Expr};

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
    assert!(matches!(rule.alert, Some(Alert::Alert { text: None, .. })));
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

    let e = errors("module demo \"Demo\"\n  card other_conventions.blackwood.notes\n");
    assert!(e[0].contains("use `slam.blackwood.notes`"), "{e:?}");

    let e = errors("after 1N (P)\n  2C \"x\"\n");
    assert!(e[0].contains("must start with `module"), "{e:?}");

    let e = errors("module demo \"Demo\"\n\nafter 1N (P)\n  2C \"x\"\n  shows hcp>=8\n");
    assert!(e[0].contains("must be indented under a rule"), "{e:?}");

    let e = errors("module demo \"Demo\"\n\nwhen hcp>=\n");
    assert!(e[0].starts_with("t.bid:3:"), "{e:?}");
}

#[test]
fn modules_name_their_teaching_skills() {
    let src = "module demo \"Demo\"\n  convention bidding_conventions/stayman\n  skill bidding_conventions/stayman\n  skill partnership_bidding/stayman_transfers precision/1c_opener\n\nafter 1N (P)\n  2C \"x\"\n";
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
    let e = errors("module demo \"Demo\"\n  kind gadget\n");
    assert!(e[0].contains("not a module kind"), "{e:?}");
    let e = errors("module demo \"Demo\"\n  kind system\n  kind judgement\n");
    assert!(e[0].contains("one `kind` line"), "{e:?}");
    let k = bidspec::parse("module demo \"Demo\"\n  kind defence\n", "t.bid").unwrap();
    assert!(bidspec::to_json(&k).contains("\"kind\": \"defence\""));
    let e = errors("module demo \"Demo\"\nskill bidding_conventions/stayman\n");
    assert!(e[0].contains("must be indented under `module`"), "{e:?}");
    // A `/` is only for skill paths.
    let e = errors("module demo \"Demo\"\n\nwhen hcp / 2 >= 4\n");
    assert!(e[0].starts_with("t.bid:3:"), "{e:?}");
}

#[test]
fn named_auction_patterns_parse() {
    let src = "\
module demo \"Demo\"
  param kokish = bidding_conventions/kokish default false

define auction nt2 = 2N (P) | 2C (P) 2D (P) 2N (P)
  2C (P) 2D (P) 2H (P) 2S (P) 2N (P)   when kokish
define auction nt2_x = nt2 | (1x) 2N (P)

after nt2
  3C  \"Stayman\"
after nt2 3C (P) 3D (P) | 1N (P) 2C (P) 2D (P) when kokish
  P   \"x\"
";
    let m = bidspec::compile(src, "t.bid", vocab().registry()).unwrap();
    assert_eq!(m.auctions.len(), 2);
    let nt2 = &m.auctions[0];
    assert_eq!(nt2.name, "nt2");
    assert_eq!(nt2.alts.len(), 3);
    assert_eq!(
        nt2.alts.iter().map(|a| a.line).collect::<Vec<_>>(),
        [4, 4, 5]
    );
    assert!(nt2.alts[0].when.is_none() && nt2.alts[1].when.is_none());
    assert_eq!(nt2.alts[2].when.as_ref().unwrap().to_string(), "kokish");
    assert!(matches!(
        &m.auctions[1].alts[0].pattern[0].call,
        CallSpec::Named { name } if name == "nt2"
    ));
    let after = m.contexts[1].after.as_ref().unwrap();
    assert_eq!(after.len(), 2);
    assert_eq!(after[0].len(), 5);
    assert!(matches!(&after[0][0].call, CallSpec::Named { name } if name == "nt2"));
    assert!(!after[0][1].theirs && after[0][2].theirs);
    // The IR round-trips.
    let json = serde_json::to_string(&m).unwrap();
    let back: bidspec::Module = serde_json::from_str(&json).unwrap();
    assert_eq!(back, m);
    // The reference prints what was written.
    let r = bidspec::reference::entries(std::slice::from_ref(&m));
    assert_eq!(r[0].rules[0].after, ["nt2"]);
    assert!(
        r[0].defines.iter().any(|d| d
            == "auction nt2 = 2N (P) | 2C (P) 2D (P) 2N (P) | \
                [2C (P) 2D (P) 2H (P) 2S (P) 2N (P) when kokish]"),
        "{:?}",
        r[0].defines
    );
}

#[test]
fn systems_on_parses() {
    let src = "\
module demo \"Demo\"

systems on when call = 1N | call = 2N, maybe shown.balanced
  !me.named(C)
define auction nt2 = systems 2N (P)
after systems 1N (P) 2C (P) | nt2
  P   \"x\"
";
    let m = bidspec::compile(src, "t.bid", vocab().registry()).unwrap();
    assert_eq!(m.systems.len(), 1);
    assert_eq!(m.systems[0].line, 3);
    assert!(matches!(&m.systems[0].when, Expr::And { all } if all.len() == 2));
    assert!(matches!(
        m.auctions[0].alts[0].pattern[0].call,
        CallSpec::Systems { level: 2 }
    ));
    let after = m.contexts[0].after.as_ref().unwrap();
    assert!(matches!(after[0][0].call, CallSpec::Systems { level: 1 }));
    assert!(after[0][1].theirs && !after[0][2].theirs);
    let r = bidspec::reference::entries(std::slice::from_ref(&m));
    assert_eq!(r[0].rules[0].after, ["systems 1N (P) 2C (P)", "nt2"]);
    assert!(
        r[0].forces[0].starts_with("systems on when "),
        "{:?}",
        r[0].forces
    );
    let json = serde_json::to_string(&m).unwrap();
    let back: bidspec::Module = serde_json::from_str(&json).unwrap();
    assert_eq!(back, m);

    let head = "module demo \"Demo\"\n";
    let e = errors(&format!("{head}after systems 2N\n  P \"x\"\n"));
    assert!(e[0].contains("RHO"), "{e:?}");
    let e = errors(&format!("{head}after systems 2C (P)\n  P \"x\"\n"));
    assert!(e[0].contains("notrump bid after `systems`"), "{e:?}");
    let e = errors(&format!("{head}after 1C (P) systems 1N (P)\n  P \"x\"\n"));
    assert!(e[0].contains("expected a call"), "{e:?}");
    let e = errors(&format!("{head}systems when call = 1N\n"));
    assert!(e[0].contains("systems on when"), "{e:?}");
    let e = errors(&format!("{head}define auction systems = 1N (P)\n"));
    assert!(e[0].contains("name uses a-z"), "{e:?}");
}

#[test]
fn named_auction_pattern_errors() {
    let head = "module demo \"Demo\"\n";
    // After the name, our call comes next: the pattern ends with theirs.
    let e = errors(&format!("{head}after nt2 (P)\n  P \"x\"\n"));
    assert!(e[0].contains("alternate"), "{e:?}");
    // A name only starts an alternative.
    let e = errors(&format!("{head}after 1N (P) nt2\n  P \"x\"\n"));
    assert!(e[0].contains("expected a call"), "{e:?}");
    // A suffix ends with the call just before my turn.
    let e = errors(&format!("{head}after nt2 3C\n  P \"x\"\n"));
    assert!(e[0].contains("RHO"), "{e:?}");
    let e = errors(&format!("{head}define auction nt2 =\n"));
    assert!(e[0].contains("at least one pattern"), "{e:?}");
    let e = errors(&format!("{head}define auction X = 2N (P)\n"));
    assert!(e[0].contains("name uses a-z"), "{e:?}");
    let e = errors(&format!(
        "{head}define auction nt2 = 2N (P)\n  2C (P) 2D (P) 2N\n"
    ));
    assert!(e[0].starts_with("t.bid:3:"), "{e:?}");
}
