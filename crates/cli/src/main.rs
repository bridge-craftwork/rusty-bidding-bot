use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use bridge_card::{bbsa, schema, Card};
use clap::{Parser, Subcommand};

mod coverage;

#[derive(Parser)]
#[command(name = "rbb", about = "rusty-bidding-bot command-line tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Convention card tools.
    #[command(subcommand)]
    Card(CardCommand),
    /// Rule file (.bid) tools.
    #[command(subcommand)]
    Bid(BidCommand),
    /// Compare the engine with BBA's auctions in Practice-Bidding-Scenarios.
    Compare {
        /// Scenario names, or patterns with `*` and `?` (e.g. 1N Stayman
        /// 'Basic_*'); all when none are given. Quote a pattern so the
        /// shell does not try to expand it into filenames.
        scenarios: Vec<String>,
        /// Practice-Bidding-Scenarios checkout.
        #[arg(long, default_value = "../Practice-Bidding-Scenarios")]
        pbs: PathBuf,
        /// At most this many boards per scenario.
        #[arg(short, long)]
        limit: Option<usize>,
        /// Directory of .bid modules.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
        /// Score differing contracts against double-dummy par (slow the first
        /// time; results are cached).
        #[arg(long)]
        par: bool,
        /// Double-dummy cache file.
        #[arg(long, default_value = ".rbb-cache/dd.jsonl")]
        dd_cache: PathBuf,
        /// How many divergence points to list.
        #[arg(long, default_value_t = 25)]
        top: usize,
        /// Order the divergence points by what they cost against par
        /// rather than by how often they happen.
        #[arg(long)]
        by_imps: bool,
        /// Only scenarios whose cards our rules cover at least this well,
        /// as a percentage (see `card coverage`). Both sides must pass.
        #[arg(long)]
        min_coverage: Option<f64>,
        /// How many scenarios to list (worst first).
        #[arg(long, default_value_t = 30)]
        worst: usize,
        /// Write the full report (every board) as JSON.
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Ask bba-cli how it bids chosen hands, and compare our engine with it.
    ///
    /// Example: rbb probe --hand S=AK52.KQ73.A95.J8 --vary-tens
    ///          --prefix "1NT Pass 2NT Pass" --dealer S
    Probe {
        /// A fixed holding, SEAT=S.H.D.C (repeatable). Other seats are dealt
        /// at random.
        #[arg(long = "hand")]
        hands: Vec<String>,
        /// Also try the first holding with 1, 2, 3, 4 more tens (same HCP
        /// and shape).
        #[arg(long)]
        vary_tens: bool,
        /// Random layouts of the other hands per holding.
        #[arg(long, default_value_t = 1)]
        layouts: usize,
        /// A dealer3 script to deal from instead of fixed holdings.
        #[arg(long)]
        script: Option<PathBuf>,
        /// Deals from --script, or random deals when neither is given.
        #[arg(short = 'n', long, default_value_t = 50)]
        count: usize,
        /// Calls forced before the decision under study.
        #[arg(short, long, default_value = "")]
        prefix: String,
        #[arg(short, long, default_value = "N")]
        dealer: char,
        #[arg(short, long, default_value = "None")]
        vul: String,
        #[arg(short, long, default_value = "MP")]
        scoring: String,
        /// Card for North-South: a name in PBS bbsa/, a .bbsa path, or
        /// bare:2/1 (also bare:sayc, bare:precision, bare:acol, bare:polish).
        #[arg(long, default_value = "21GF-DEFAULT")]
        ns_card: String,
        #[arg(long, default_value = "21GF-GIB")]
        ew_card: String,
        /// Set a .bbsa key on the North-South card, "Key=value" (repeatable).
        #[arg(long = "set")]
        ns_set: Vec<String>,
        /// Same for East-West.
        #[arg(long = "ew-set")]
        ew_set: Vec<String>,
        #[arg(long, default_value = "../Practice-Bidding-Scenarios")]
        pbs: PathBuf,
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
        #[arg(long, default_value = rbb_compare::probe::DEFAULT_BBA_CLI)]
        bba_cli: PathBuf,
        /// dealer3's binary (for --script).
        #[arg(long, default_value = "dealer")]
        dealer_bin: PathBuf,
        #[arg(long, default_value = ".rbb-cache/probes/last")]
        out: PathBuf,
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },
    /// Choose a call for a hand and show why.
    Call {
        /// The hand in PBN order S.H.D.C, e.g. AK52.KJ7.Q94.K83
        hand: String,
        /// Calls so far, e.g. "1NT Pass"
        #[arg(short, long, default_value = "")]
        auction: String,
        /// Dealer: N, E, S or W.
        #[arg(short, long, default_value = "N")]
        dealer: char,
        /// Vulnerability: None, NS, EW or All.
        #[arg(short, long, default_value = "None")]
        vul: String,
        /// Scoring: MP or IMP.
        #[arg(short, long, default_value = "MP")]
        scoring: String,
        /// Card for both sides (.bbsa or card JSON).
        #[arg(short, long)]
        card: PathBuf,
        /// A different card for East-West.
        #[arg(long)]
        ew_card: Option<PathBuf>,
        /// Directory of .bid modules.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
        /// Print the full decision as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum BidCommand {
    /// Parse and check .bid files (directories are searched recursively).
    Check {
        #[arg(default_value = "conventions")]
        paths: Vec<PathBuf>,
    },
    /// Run the cases in .test files (directories are searched recursively).
    Test {
        #[arg(default_value = "conventions")]
        paths: Vec<PathBuf>,
        /// The rule files the cases run against.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
        /// Where card names (`card 21GF-DEFAULT`) are looked up as .bbsa.
        #[arg(long, default_value = "crates/bridge-card/tests/fixtures/bbsa")]
        cards: PathBuf,
        /// Also list the cases that pass.
        #[arg(short, long)]
        verbose: bool,
    },
    /// Print a .bid file's compiled JSON IR.
    Compile {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum CardCommand {
    /// Convert a BBA .bbsa file to card JSON; reports keys with no card field.
    ImportBbsa {
        file: PathBuf,
        /// Write the card here instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Convert card JSON to a BBA .bbsa file.
    ExportBbsa {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Load card JSON and report aliases, unknown paths and invalid values.
    Check { file: PathBuf },
    /// Print the JSON Schema for card JSON.
    Schema {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// How much of a card our rules read: what they honour, what they
    /// ignore, and which .bbsa keys have no card field at all.
    Coverage {
        /// A .bbsa file or card JSON; several may be given.
        files: Vec<PathBuf>,
        /// Directory of .bid modules.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
        /// List every setting, not just the counts.
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Card(cmd) => card(cmd),
        Command::Bid(cmd) => bid(cmd),
        Command::Compare {
            scenarios,
            pbs,
            limit,
            rules,
            par,
            dd_cache,
            top,
            by_imps,
            min_coverage,
            worst,
            json,
        } => {
            let mut opts = rbb_compare::Options {
                pbs,
                scenarios,
                limit,
                rules,
                par,
                dd_cache,
            };
            if let Some(min) = min_coverage {
                opts.scenarios = covered_scenarios(&opts, min)?;
            }
            compare(&opts, top, by_imps, worst, json.as_deref())
        }
        Command::Probe {
            hands,
            vary_tens,
            layouts,
            script,
            count,
            prefix,
            dealer,
            vul,
            scoring,
            ns_card,
            ew_card,
            ns_set,
            ew_set,
            pbs,
            rules,
            bba_cli,
            dealer_bin,
            out,
            seed,
        } => {
            use bridge_types::{Call, Direction, Hand, ScoringMethod, Vulnerability};
            use rbb_compare::probe::{Deals, ProbeOptions};
            let parse_set = |v: &[String]| -> Result<Vec<(String, i64)>> {
                v.iter()
                    .map(|s| {
                        let (k, n) = s.rsplit_once('=').ok_or("--set takes Key=value")?;
                        Ok((
                            k.trim().to_string(),
                            n.trim().parse().map_err(|_| "value must be a number")?,
                        ))
                    })
                    .collect()
            };
            let fixed = hands
                .iter()
                .map(|h| {
                    let (seat, cards) = h.split_once('=').ok_or("--hand takes SEAT=S.H.D.C")?;
                    let seat = seat
                        .chars()
                        .next()
                        .and_then(|c| Direction::from_char(c.to_ascii_uppercase()));
                    let hand = Hand::from_pbn(cards);
                    match (seat, hand) {
                        (Some(s), Some(h)) => Ok((s, h)),
                        _ => Err(format!("bad --hand {h:?}").into()),
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            let deals = match (&script, fixed.is_empty()) {
                (Some(s), _) => Deals::Script {
                    script: s.clone(),
                    count,
                    dealer_bin: if dealer_bin == Path::new("dealer")
                        && Path::new("../dealer3/target/release/dealer").exists()
                    {
                        PathBuf::from("../dealer3/target/release/dealer")
                    } else {
                        dealer_bin
                    },
                },
                (None, false) => Deals::Fixed {
                    hands: fixed,
                    vary_tens,
                    layouts,
                },
                (None, true) => Deals::Random { count },
            };
            let opts = ProbeOptions {
                deals,
                dealer: Direction::from_char(dealer.to_ascii_uppercase())
                    .ok_or("dealer must be N, E, S or W")?,
                vul: Vulnerability::from_pbn(&vul)
                    .ok_or("vulnerability must be None, NS, EW or All")?,
                scoring: ScoringMethod::from_pbn(&scoring).ok_or("scoring must be MP or IMP")?,
                prefix: prefix
                    .split_whitespace()
                    .map(|c| Call::from_pbn(c).ok_or_else(|| format!("bad call {c:?}")))
                    .collect::<std::result::Result<_, _>>()?,
                ns_card,
                ew_card,
                ns_set: parse_set(&ns_set)?,
                ew_set: parse_set(&ew_set)?,
                pbs,
                rules,
                bba_cli,
                out_dir: out,
                seed,
            };
            probe(&opts)
        }
        Command::Call {
            hand,
            auction,
            dealer,
            vul,
            scoring,
            card,
            ew_card,
            rules,
            json,
        } => call(
            &hand,
            &auction,
            dealer,
            &vul,
            &scoring,
            &card,
            ew_card.as_deref(),
            &rules,
            json,
        ),
    }
}

fn probe(opts: &rbb_compare::probe::ProbeOptions) -> Result<()> {
    let report = rbb_compare::probe::run(opts)?;
    let prefix: Vec<String> = opts.prefix.iter().map(rbb_compare::short).collect();
    println!(
        "probe: {} boards, dealer {}, vul {}, {}; after `{}`",
        report.rows.len(),
        opts.dealer.to_char(),
        opts.vul.to_pbn(),
        if matches!(opts.scoring, bridge_types::ScoringMethod::Matchpoints) {
            "MP"
        } else {
            "IMP"
        },
        prefix.join(" ")
    );
    println!(
        "NS card {} {:?}   EW card {} {:?}",
        opts.ns_card, opts.ns_set, opts.ew_card, opts.ew_set
    );
    println!(
        "\n  seat {:18} {:>3} {:>4} {:>6} {:>6}  {:>5} {:>5}",
        "hand", "HCP", "tens", "NT pts", "suit", "BBA", "ours"
    );
    let call = |c: &Option<bridge_types::Call>| c.as_ref().map_or("-".into(), rbb_compare::short);
    for r in &report.rows {
        let quarters = |q: i32| format!("{}{}", q / 4, ["", "¼", "½", "¾"][(q % 4) as usize]);
        let mark = if r.reference == r.ours { "" } else { "  ≠" };
        println!(
            "  {:4} {:18} {:>3} {:>4} {:>6} {:>6}  {:>5} {:>5}{mark}",
            r.seat.to_char(),
            r.hand,
            r.hcp,
            r.tens,
            quarters(r.points_q),
            quarters(r.suit_points_q),
            call(&r.reference),
            call(&r.ours)
        );
    }
    let (agree, of) = report.agreement();
    println!(
        "\nsame decision as BBA: {agree}/{of}   (files in {})",
        report.out_dir.display()
    );
    Ok(())
}

fn pct(x: f64) -> String {
    format!("{:5.1}%", 100.0 * x)
}

fn compare(
    opts: &rbb_compare::Options,
    top: usize,
    by_imps: bool,
    worst: usize,
    json: Option<&Path>,
) -> Result<()> {
    let started = std::time::Instant::now();
    let report = rbb_compare::run(opts, &|done, total| {
        eprint!("\r{done}/{total} boards");
    })?;
    eprintln!("  ({:.1}s)", started.elapsed().as_secs_f64());
    let s = &report.summary;
    let t = &s.total;
    let calls = t.calls_all();
    println!("{} scenarios, {} boards", s.scenarios.len(), t.boards);
    println!(
        "calls agreeing with BBA (replay): {} ({}/{})   NS {}   EW {}",
        pct(calls.rate()),
        calls.agree,
        calls.total,
        pct(t.calls[0].rate()),
        pct(t.calls[1].rate())
    );
    println!(
        "identical auctions: {}   same final contract: {}",
        pct(t.auction_rate()),
        pct(t.contract_rate())
    );
    if t.runaway > 0 {
        println!(
            "auctions the engine did not finish in 60 calls: {}",
            t.runaway
        );
    }
    if t.par.scored > 0 {
        println!(
            "differing contracts vs par ({} boards): ours closer {}, BBA closer {}, equal {}; net {:+} IMPs to us",
            t.par.scored, t.par.ours_closer, t.par.reference_closer, t.par.equal, t.par.imps_vs_reference
        );
        let solved = t.boards.saturating_sub(t.dd_tables);
        println!(
            "  double-dummy tables: {} read from the corpus files, {solved} boards without one",
            t.dd_tables
        );
    }
    let hist: Vec<String> = t
        .first_divergence
        .iter()
        .take(12)
        .map(|(i, n)| format!("{}:{n}", i + 1))
        .collect();
    println!("first divergence at call #: {}", hist.join("  "));

    // Problems: wrong whatever the convention, and independent of BBA.
    println!(
        "\nproblems in our auctions ({} boards):",
        t.boards_with_problems
    );
    for (kind, n) in &t.problems {
        println!("  {:32} {n}", kind.label());
    }
    for p in s.problems.iter().take(top) {
        let auction = if p.auction.is_empty() {
            "(opening)".to_string()
        } else {
            p.auction.clone()
        };
        let mut names = p
            .scenarios
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        if p.scenarios.len() > 3 {
            names += &format!(" +{}", p.scenarios.len() - 3);
        }
        println!(
            "  {:>5}  {:26} {:28} {:>5}  {names}",
            p.count,
            p.kind.label(),
            auction,
            p.call
        );
    }

    // The conditions the reference auctions were made under.
    let list = |m: &std::collections::BTreeMap<String, usize>| {
        m.iter()
            .map(|(k, v)| format!("{k} {v}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let rates = |m: &std::collections::BTreeMap<String, rbb_compare::Agreement>| {
        m.iter()
            .map(|(k, a)| format!("{k} {}", pct(a.rate()).trim()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!("\nreference conditions (boards):");
    println!("  scoring:    {}", list(&t.boards_by_scoring));
    println!("  generator:  {}", list(&t.boards_by_generator));
    println!("calls agreeing, by condition:");
    println!(
        "  caller not vulnerable {}   vulnerable {}",
        pct(t.by_caller_vul[0].rate()).trim(),
        pct(t.by_caller_vul[1].rate()).trim()
    );
    println!("  scoring:    {}", rates(&t.by_scoring));
    println!("  generator:  {}", rates(&t.by_generator));

    let mut points: Vec<&rbb_compare::Divergence> = s.divergences.iter().collect();
    if by_imps {
        points.sort_by_key(|d| d.imps);
        println!("\nmost expensive divergence points (IMPs against par, ours minus BBA's):");
    } else {
        println!("\nmost common divergence points:");
    }
    println!(
        "  {:>5} {:>7}  {:28} {:>5} {:>5}  scenarios",
        "count", "IMPs", "auction so far", "BBA", "ours"
    );
    for d in points.into_iter().take(top) {
        let auction = if d.auction.is_empty() {
            "(opening)".to_string()
        } else {
            d.auction.clone()
        };
        let mut names = d
            .scenarios
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        if d.scenarios.len() > 3 {
            names += &format!(" +{}", d.scenarios.len() - 3);
        }
        println!(
            "  {:>5} {:>+7}  {:28} {:>5} {:>5}  {names}",
            d.count, d.imps, auction, d.reference, d.ours
        );
    }

    if s.scenarios.len() > 1 {
        let mut by = s.scenarios.clone();
        by.sort_by(|a, b| {
            a.calls_all()
                .rate()
                .partial_cmp(&b.calls_all().rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        println!("\nscenarios, lowest call agreement first:");
        println!(
            "  {:32} {:>6} {:>7} {:>8} {:>9}",
            "scenario", "boards", "calls", "auction", "contract"
        );
        for sc in by.iter().take(worst) {
            println!(
                "  {:32} {:>6} {:>7} {:>8} {:>9}",
                sc.name,
                sc.boards,
                pct(sc.calls_all().rate()),
                pct(sc.auction_rate()),
                pct(sc.contract_rate())
            );
        }
    }
    if let Some(path) = json {
        std::fs::write(path, serde_json::to_string(&report)?)?;
        eprintln!("wrote {}", path.display());
    }
    Ok(())
}

fn load_card(path: &Path) -> Result<Card> {
    let text = read(path)?;
    if path.extension().is_some_and(|e| e == "bbsa") {
        Ok(bbsa::import(&text, None)?.0)
    } else {
        Ok(Card::from_json(&text)?.0)
    }
}

#[allow(clippy::too_many_arguments)]
fn call(
    hand: &str,
    auction: &str,
    dealer: char,
    vul: &str,
    scoring: &str,
    card: &Path,
    ew_card: Option<&Path>,
    rules: &Path,
    json: bool,
) -> Result<()> {
    use bridge_types::{Call, Direction, Hand, ScoringMethod, Vulnerability};
    let scoring = ScoringMethod::from_pbn(scoring).ok_or("scoring must be MP or IMP")?;
    let hand = Hand::from_pbn(hand).ok_or("hand must be PBN S.H.D.C, e.g. AK52.KJ7.Q94.K83")?;
    let calls = auction
        .split_whitespace()
        .map(|c| Call::from_pbn(c).ok_or_else(|| format!("bad call {c:?}")))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let dealer =
        Direction::from_char(dealer.to_ascii_uppercase()).ok_or("dealer must be N, E, S or W")?;
    let vul = Vulnerability::from_pbn(vul).ok_or("vulnerability must be None, NS, EW or All")?;
    let ns = load_card(card)?;
    let ew = match ew_card {
        Some(p) => load_card(p)?,
        None => ns.clone(),
    };
    let modules = rbb_engine::load_modules(rules).map_err(|d| {
        d.iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let engine = rbb_engine::Engine::new(&ns, &ew, &modules);
    let d = engine.bid(&hand, dealer, vul, scoring, &calls);
    if json {
        println!("{}", serde_json::to_string_pretty(&d)?);
        return Ok(());
    }
    println!("{}: {}", d.call, d.explanation);
    if let Some(r) = &d.rule {
        println!("  rule: {}:{} ({})", r.file, r.line, r.module);
    }
    println!("\ncandidates (best-ranked first):");
    for c in &d.candidates {
        println!(
            "  {:5} prio {:>3}  descr {:.3}  {}",
            c.call.to_string(),
            c.priority,
            c.descriptiveness,
            c.outcome
        );
    }
    if !d.auction.steps.is_empty() {
        println!("\nauction so far:");
        for s in &d.auction.steps {
            let k = &s.knowledge;
            println!(
                "  {:?} {:5} {:30} hcp {}  S {} H {} D {} C {}",
                s.caller,
                s.call.to_string(),
                s.explanation.as_deref().unwrap_or("(no rule)"),
                k.hcp,
                k.len[3],
                k.len[2],
                k.len[1],
                k.len[0]
            );
        }
    }
    for w in &d.warnings {
        eprintln!("warning: {w}");
    }
    Ok(())
}

fn bid(cmd: BidCommand) -> Result<()> {
    match cmd {
        BidCommand::Check { paths } => {
            let mut files = Vec::new();
            for p in &paths {
                collect_bid_files(p, &mut files)?;
            }
            files.sort();
            let mut modules = Vec::new();
            let mut errors = 0;
            for file in &files {
                match bidspec::compile(&read(file)?, &file.display().to_string()) {
                    Ok(m) => modules.push(m),
                    Err(diags) => {
                        errors += diags.len();
                        for d in diags {
                            eprintln!("{d}");
                        }
                    }
                }
            }
            // Across modules: unique names, and `needs` that resolve.
            let mut seen = std::collections::HashMap::new();
            for m in &modules {
                if let Some(other) = seen.insert(m.name.as_str(), m.file.as_str()) {
                    eprintln!("{}: module `{}` is also defined in {other}", m.file, m.name);
                    errors += 1;
                }
            }
            for e in rbb_engine::check_card_refs(&modules) {
                eprintln!("{e}");
                errors += 1;
            }
            let mut warnings = 0;
            for m in &modules {
                for need in &m.needs {
                    if !seen.contains_key(need.as_str()) {
                        eprintln!(
                            "{}: warning: needs `{need}`, which is not defined yet",
                            m.file
                        );
                        warnings += 1;
                    }
                }
            }
            let rules: usize = modules.iter().map(count_rules).sum();
            println!(
                "{} files, {} modules, {rules} rules: {errors} errors, {warnings} warnings",
                files.len(),
                modules.len()
            );
            if errors > 0 {
                return Err("rule files have errors".into());
            }
        }
        BidCommand::Test {
            paths,
            rules,
            cards,
            verbose,
        } => {
            if let Some(p) = paths.iter().find(|p| !p.exists()) {
                return Err(format!("{}: no such file or directory", p.display()).into());
            }
            let files: Vec<PathBuf> = paths
                .iter()
                .flat_map(|p| rbb_engine::cases::find(p))
                .collect();
            let outcomes = rbb_engine::cases::run(&files, &rules, &cards)
                .map_err(|errors| errors.join("\n"))?;
            let failed = outcomes.iter().filter(|o| !o.passed).count();
            for o in &outcomes {
                if o.passed && !verbose {
                    continue;
                }
                let mark = if o.passed { "ok  " } else { "FAIL" };
                println!("{mark} {}", o.report());
            }
            println!(
                "{} files, {} cases: {} passed, {failed} failed",
                files.len(),
                outcomes.len(),
                outcomes.len() - failed
            );
            if failed > 0 {
                return Err("some cases failed".into());
            }
        }
        BidCommand::Compile { file, output } => {
            let module =
                bidspec::compile(&read(&file)?, &file.display().to_string()).map_err(|diags| {
                    diags
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                })?;
            write(output.as_deref(), &bidspec::to_json(&module))?;
        }
    }
    Ok(())
}

fn count_rules(m: &bidspec::Module) -> usize {
    fn walk(c: &bidspec::ast::Context) -> usize {
        c.rules.len() + c.contexts.iter().map(walk).sum::<usize>()
    }
    m.contexts.iter().map(walk).sum()
}

fn collect_bid_files(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))? {
            collect_bid_files(&entry?.path(), out)?;
        }
    } else if path.extension().is_some_and(|e| e == "bid") {
        out.push(path.to_path_buf());
    }
    Ok(())
}

fn card(cmd: CardCommand) -> Result<()> {
    match cmd {
        CardCommand::ImportBbsa { file, output } => {
            let text = read(&file)?;
            let name = file.file_stem().and_then(|s| s.to_str());
            let (card, report) = bbsa::import(&text, name)?;
            write(output.as_deref(), &card.to_json_string())?;
            eprintln!("{}: {} keys mapped", file.display(), report.mapped);
            if !report.passthrough.is_empty() {
                eprintln!(
                    "{} keys have no card field (kept in bba_passthrough):",
                    report.passthrough.len()
                );
                for (key, value) in &report.passthrough {
                    eprintln!("  {key} = {value}");
                }
            }
            for w in &report.warnings {
                eprintln!("warning: {w}");
            }
        }
        CardCommand::ExportBbsa { file, output } => {
            let (card, load) = Card::from_json(&read(&file)?)?;
            print_load_report(&load);
            let (text, report) = bbsa::export(&card);
            write(output.as_deref(), &text)?;
            for key in &report.dropped {
                eprintln!("warning: {key} is not in the current .bbsa layout; not written");
            }
        }
        CardCommand::Check { file } => {
            let (card, load) = Card::from_json(&read(&file)?)?;
            println!("{}: {} settings", file.display(), card.values().count());
            print_load_report(&load);
            if !load.is_clean() {
                return Err("card has unknown or invalid fields".into());
            }
        }
        CardCommand::Schema { output } => {
            let text = serde_json::to_string_pretty(&schema::json_schema())?;
            write(output.as_deref(), &text)?;
        }
        CardCommand::Coverage {
            files,
            rules,
            verbose,
        } => card_coverage(&files, &rules, verbose)?,
    }
    Ok(())
}

/// The scenarios whose two cards our rules both cover at least `min`
/// percent of. Reports what it leaves out, since that is the point.
fn covered_scenarios(opts: &rbb_compare::Options, min: f64) -> Result<Vec<String>> {
    use std::collections::HashMap;
    let modules = rbb_engine::load_modules(&opts.rules).map_err(|_| "rule files have errors")?;
    let read = coverage::fields_read(&modules);
    let scenarios = rbb_compare::discover(&opts.pbs, &opts.scenarios)?;
    let mut score: HashMap<String, f64> = HashMap::new();
    let mut of = |name: &str| -> f64 {
        if let Some(s) = score.get(name) {
            return *s;
        }
        let path = opts.pbs.join("bbsa").join(format!("{name}.bbsa"));
        let s = coverage::load(&path)
            .map(|(n, card, unmapped)| coverage::of_card(&n, &card, unmapped, &read).score())
            .unwrap_or(0.0);
        score.insert(name.to_string(), s);
        s
    };
    let mut kept = Vec::new();
    let mut dropped: Vec<(String, f64)> = Vec::new();
    for sc in &scenarios {
        let worst = of(&sc.ns_card).min(of(&sc.ew_card));
        if worst * 100.0 >= min {
            kept.push(sc.name.clone());
        } else {
            dropped.push((sc.name.clone(), worst * 100.0));
        }
    }
    eprintln!(
        "coverage filter: {} of {} scenarios have both cards at {min:.0}% or better ({} left out)",
        kept.len(),
        scenarios.len(),
        dropped.len()
    );
    if kept.is_empty() {
        return Err(format!("no scenario has both cards covered to {min:.0}%").into());
    }
    Ok(kept)
}

/// `card coverage`: what the rules read of each card.
fn card_coverage(files: &[PathBuf], rules: &Path, verbose: bool) -> Result<()> {
    let modules = rbb_engine::load_modules(rules).map_err(|d| {
        for e in &d {
            eprintln!("{e}");
        }
        "rule files have errors"
    })?;
    let read = coverage::fields_read(&modules);
    println!(
        "{} modules read {} card fields\n",
        modules.len(),
        read.len()
    );
    println!(
        "{:24} {:14} {:>6} {:>8} {:>9} {:>9}",
        "card", "system", "read", "ignored", "unmapped", "coverage"
    );
    let mut covs = Vec::new();
    for f in files {
        let (name, card, unmapped) = coverage::load(f)?;
        let cov = coverage::of_card(&name, &card, unmapped, &read);
        println!(
            "{:24} {:14} {:6} {:8} {:9} {:8.0}%",
            cov.name,
            cov.system,
            cov.read.len(),
            cov.ignored.len(),
            cov.unmapped.len(),
            100.0 * cov.score()
        );
        covs.push(cov);
    }
    if verbose {
        for cov in &covs {
            println!("\n{}: settings switched on that no rule reads", cov.name);
            for p in &cov.ignored {
                println!("  {p}");
            }
            if !cov.unmapped.is_empty() {
                println!("{}: .bbsa keys with no card field", cov.name);
                for k in &cov.unmapped {
                    println!("  {k}");
                }
            }
        }
    }
    Ok(())
}

fn print_load_report(load: &bridge_card::LoadReport) {
    for (from, to) in &load.aliased {
        eprintln!("alias: {from} -> {to}");
    }
    for path in &load.unknown {
        eprintln!("unknown: {path}");
    }
    for (path, problem) in &load.invalid {
        eprintln!("invalid: {path}: {problem}");
    }
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()).into())
}

fn write(path: Option<&Path>, text: &str) -> Result<()> {
    match path {
        Some(p) => fs::write(p, text).map_err(|e| format!("{}: {e}", p.display()).into()),
        None => {
            println!("{text}");
            Ok(())
        }
    }
}
