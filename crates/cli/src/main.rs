use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use bridge_card::{bbsa, schema, Card, Vocabulary};
use clap::{Parser, Subcommand};
use rbb_engine::RuleSet;

mod bid_pbn;
mod coverage;

#[derive(Parser)]
#[command(name = "rbb", about = "rusty-bidding-bot command-line tools", version = version())]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
// Parsed once at startup; the probe command's many options make it large.
#[allow(clippy::large_enum_variant)]
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
        /// Order the divergence points by what they cost against BBA (par
        /// as the yardstick) rather than by how often they happen.
        #[arg(long)]
        by_imps: bool,
        /// Only scenarios whose cards our rules cover at least this well,
        /// as a percentage (see `card coverage`). Both sides must pass.
        #[arg(long)]
        min_coverage: Option<f64>,
        /// A card change for both sides, `path=value` (repeatable), e.g.
        /// `--set general.style=bba` to run BBA's treatments for an A/B test.
        #[arg(long = "set")]
        card_changes: Vec<String>,
        /// Only boards where, in BBA's auction, one side bid alone
        /// (`ns`, `ew`) or both sides bid (`competitive`).
        #[arg(long, value_parser = ["all", "ns", "ew", "competitive"], default_value = "all")]
        auctions: String,
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
        /// Every single-card exchange between the first holding and its two
        /// opponents (partner's hand stays), one deal: which changes flip
        /// BBA's call?
        #[arg(long, conflicts_with_all = ["vary_tens", "morph_to"])]
        survey: bool,
        /// Walk from the first holding to this one (S.H.D.C) a card at a
        /// time, then try each exchange alone: where does BBA's call flip?
        #[arg(long, conflicts_with = "vary_tens")]
        morph_to: Option<String>,
        /// A card change for our engine only, `path=value` (repeatable),
        /// e.g. `general.style=bba`.
        #[arg(long = "our-set")]
        our_set: Vec<String>,
        /// Probe these holdings (S.H.D.C) in place of the first --hand, each
        /// as its own board (repeatable). Other --hand seats stay fixed.
        #[arg(long = "variant", conflicts_with_all = ["vary_tens", "survey", "morph_to"])]
        variants: Vec<String>,
        /// A file of holdings for the first seat, one per line: `S.H.D.C`
        /// or `label | S.H.D.C` (# comments).
        #[arg(long = "variants", conflicts_with_all = ["vary_tens", "survey", "morph_to"])]
        variants_file: Option<PathBuf>,
        /// Bid every deal at each of these vulnerabilities, e.g.
        /// None,NS,EW,All (overrides --vul).
        #[arg(long, value_delimiter = ',')]
        vuls: Vec<String>,
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
        /// Card for North-South: a name in PBS bbsa/, a .bbsa path, a card
        /// JSON path (Bridge-Classroom's export too; BBA gets the .bbsa it
        /// maps to), or bare:2/1 (also bare:sayc, bare:precision, bare:acol,
        /// bare:polish).
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
    /// Probe one decision over a list of hands, a survey or a morph, from
    /// a spec file (see probes/*.toml), and print BBA's calls as a table:
    /// one row per hand, one column per vulnerability and scoring.
    Grid {
        /// The spec (TOML): card, dealer, prefix, vuls, scoring, partner,
        /// our, and one of hands / survey / morph.
        spec: PathBuf,
        /// A survey prints only the exchanges that change BBA's call; show
        /// every row.
        #[arg(long)]
        all: bool,
        #[arg(long, default_value = "../Practice-Bidding-Scenarios")]
        pbs: PathBuf,
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
        #[arg(long, default_value = rbb_compare::probe::DEFAULT_BBA_CLI)]
        bba_cli: PathBuf,
        /// Output directory (default .rbb-cache/grids/<spec name>).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Double-dummy yardstick for a grid spec's hands: deal partner from a
    /// pool of hands that made partner's call, the opponents at random, and
    /// report how often the partnership makes 8 and 9 tricks in notrump.
    Simulate {
        /// A grid spec with a `hands` list.
        spec: PathBuf,
        /// Partner hands, one S.H.D.C per line (e.g. the hands BBA opens 1NT).
        #[arg(long)]
        pool: PathBuf,
        #[arg(long, default_value_t = 100)]
        samples: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Also write the rows as TSV here.
        #[arg(long)]
        tsv: Option<PathBuf>,
    },
    /// Bid every deal in a PBN file and write it with our auctions, like
    /// bba-cli: [Auction] with alerts as [Note]s, [Declarer], [Contract].
    /// Every other tag of the input is kept.
    ///
    /// Example: rbb bid-pbn -i deals.pbn -o bid.pbn --ns-card 21GF-DEFAULT.bbsa
    ///          --ew-card 21GF-GIB.bbsa
    #[command(name = "bid-pbn")]
    BidPbn {
        /// Input PBN file.
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,
        /// Output PBN file (`-` for stdout).
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,
        /// North-South card: a .bbsa file, card JSON (bare card_data or
        /// Bridge-Classroom's export), or the name of a stock card built
        /// into rbb (e.g. 21GF-DEFAULT).
        #[arg(long, alias = "ns-conventions", value_name = "CARD")]
        ns_card: String,
        /// East-West card (default: the North-South card).
        #[arg(long, alias = "ew-conventions", value_name = "CARD")]
        ew_card: Option<String>,
        /// A card change for both sides, `path=value` (repeatable).
        #[arg(long = "set")]
        card_changes: Vec<String>,
        /// Scoring, MP or IMP, for every board (also written as [Scoring]).
        /// Default: each board's [Scoring] tag, else MP.
        #[arg(long)]
        scoring: Option<String>,
        /// A note for every call a rule explains, with what it showed, not
        /// only for alerted calls.
        #[arg(long)]
        all_meanings: bool,
        /// Force these calls at the start of every auction, e.g. "1C Pass 1H".
        #[arg(long, value_name = "CALLS")]
        auction_prefix: Option<String>,
        /// Set [Event] on every board.
        #[arg(long)]
        event: Option<String>,
        /// Stop an auction after this many calls (it is then finished with AP).
        #[arg(long, default_value_t = 60)]
        max_calls: usize,
        /// Directory of .bid modules; default: the rules built into rbb.
        #[arg(long)]
        rules: Option<PathBuf>,
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
        /// Also review the auction so far, call by call: what is known
        /// about every seat after each call, the flags, and this hand's own
        /// view (as `explain-auction`; with --json, a `knowledge` array).
        #[arg(long)]
        knowledge: bool,
    },
    /// Review an auction call by call: what each call shows, what is known
    /// about every seat after it (`*` marks what the call changed), the
    /// flags (F1, GF, inv, ask, alert, art), each side's state, and, with
    /// --deal, each hand's own view of itself (HCP, points, and with a fit
    /// the count by role: declarer or support points).
    #[command(name = "explain-auction")]
    ExplainAuction {
        /// The calls, e.g. "1NT Pass 2C Pass 2H".
        #[arg(short, long)]
        auction: String,
        /// The deal in PBN, e.g. "N:AK52.KJ7.Q94.K83 ...", for the hands'
        /// own view.
        #[arg(long)]
        deal: Option<String>,
        /// Dealer: N, E, S or W.
        #[arg(short, long, default_value = "N")]
        dealer: char,
        /// Vulnerability: None, NS, EW or All.
        #[arg(short, long, default_value = "None")]
        vul: String,
        /// Scoring: MP or IMP.
        #[arg(short, long, default_value = "MP")]
        scoring: String,
        /// Card for both sides: a .bbsa or card JSON file, or a stock card
        /// name (21GF-DEFAULT).
        #[arg(short, long)]
        card: String,
        /// A different card for East-West.
        #[arg(long)]
        ew_card: Option<String>,
        /// A card change for both sides, `path=value` (repeatable), as in
        /// `compare --set`.
        #[arg(long = "set")]
        card_changes: Vec<String>,
        /// Directory of .bid modules.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
        /// Print the rows as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum BidCommand {
    /// Parse and check .bid files (directories are searched recursively),
    /// and the card vocabulary they are checked against.
    Check {
        #[arg(default_value = "conventions")]
        paths: Vec<PathBuf>,
        /// The rules directory; the files are checked against its card
        /// vocabulary (its own card/ files, or the standard one).
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
    },
    /// Run the cases in .test files (directories are searched recursively).
    Test {
        #[arg(default_value = "conventions")]
        paths: Vec<PathBuf>,
        /// The rule files the cases run against.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
        /// Where card names (`card 21GF-DEFAULT`) are looked up as .bbsa.
        #[arg(long, default_value = "cards/bbsa")]
        cards: PathBuf,
        /// Also list the cases that pass.
        #[arg(short, long)]
        verbose: bool,
    },
    /// Print the conventions reference (the WASM build's `reference`, the
    /// text a website serves as reference.txt): every module, when a card
    /// switches it on, and what each call means after each auction.
    Reference {
        /// Directory of .bid modules; default: the rules built into rbb.
        #[arg(long)]
        rules: Option<PathBuf>,
        /// Also write it as JSON (the WASM build's `conventions`) here.
        #[arg(long)]
        json: Option<PathBuf>,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Print every name a rule condition may use, with its meaning: the
    /// term vocabulary of this engine's rule language (Markdown).
    Terms {
        /// Instead of printing, rewrite the generated section of this
        /// Markdown file (docs/CONTRACT.md) between its markers.
        #[arg(long)]
        doc: Option<PathBuf>,
    },
    /// Print the teaching-skill map (Markdown): each skill path, the card
    /// fields tagged with it (`skill` in the card fields), the modules
    /// declaring it (`skill` lines), and the gaps.
    Skills {
        /// The rules directory: its .bid files (the card fields and skills
        /// are the standard ones, from the bridge-card crate).
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
        /// Instead of printing, rewrite the generated section of this
        /// Markdown file (docs/SKILLS.md) between its markers.
        #[arg(long)]
        doc: Option<PathBuf>,
    },
    /// Print a .bid file's compiled JSON IR.
    Compile {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// The rules directory whose card vocabulary the file is checked
        /// against.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
    },
}

/// Every `card` command reads cards in the card vocabulary of a rules
/// directory: its own `card/` files, or the standard vocabulary.
#[derive(Subcommand)]
enum CardCommand {
    /// Convert a BBA .bbsa file to card JSON; reports keys with no card field.
    ImportBbsa {
        file: PathBuf,
        /// Write the card here instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Write Bridge-Classroom's export format ({schema, name,
        /// description, exportedAt, card_data}) instead of bare card_data.
        #[arg(long)]
        wrap: bool,
        /// The rules directory whose card vocabulary to use.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
    },
    /// Convert card JSON to a BBA .bbsa file.
    ExportBbsa {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// The rules directory whose card vocabulary to use.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
    },
    /// Load card JSON (bare card_data or Bridge-Classroom's export) and
    /// report aliases, unknown paths and invalid values.
    Check {
        file: PathBuf,
        /// The rules directory whose card vocabulary to use.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
    },
    /// Print the JSON Schema for card JSON.
    Schema {
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// The rules directory whose card vocabulary to describe.
        #[arg(long, default_value = "conventions")]
        rules: PathBuf,
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

/// `rbb --version`: the engine version and the rule language it reads.
fn version() -> &'static str {
    static V: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    V.get_or_init(|| {
        format!(
            "{} (rule language {})",
            env!("CARGO_PKG_VERSION"),
            rbb_engine::LANGUAGE_VERSION
        )
    })
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
            card_changes,
            auctions,
            worst,
            json,
        } => {
            let filter = match auctions.as_str() {
                "ns" => rbb_compare::AuctionFilter::UncontestedNs,
                "ew" => rbb_compare::AuctionFilter::UncontestedEw,
                "competitive" => rbb_compare::AuctionFilter::Competitive,
                _ => rbb_compare::AuctionFilter::All,
            };
            let mut opts = rbb_compare::Options {
                pbs,
                scenarios,
                limit,
                rules,
                par,
                dd_cache,
                card_changes,
            };
            if let Some(min) = min_coverage {
                opts.scenarios = covered_scenarios(&opts, min)?;
            }
            compare(&opts, filter, top, by_imps, worst, json.as_deref())
        }
        Command::Probe {
            hands,
            vary_tens,
            survey,
            morph_to,
            our_set,
            variants,
            variants_file,
            vuls,
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
                    variation: if !variants.is_empty() || variants_file.is_some() {
                        let mut list = Vec::new();
                        for v in &variants {
                            list.push((v.clone(), v.clone()));
                        }
                        if let Some(f) = &variants_file {
                            for line in read(f)?.lines() {
                                let line = line.split('#').next().unwrap_or("").trim();
                                if line.is_empty() {
                                    continue;
                                }
                                let (label, hand) = match line.split_once('|') {
                                    Some((l, h)) => (l.trim().to_string(), h.trim().to_string()),
                                    None => (line.to_string(), line.to_string()),
                                };
                                list.push((label, hand));
                            }
                        }
                        rbb_compare::probe::Variation::List(
                            list.into_iter()
                                .map(|(l, h)| {
                                    Hand::from_pbn(&h)
                                        .filter(|x| x.len() == 13)
                                        .map(|x| (l.clone(), x))
                                        .ok_or_else(|| format!("variant {l:?}: bad hand {h:?}"))
                                })
                                .collect::<std::result::Result<_, _>>()?,
                        )
                    } else {
                        match (&morph_to, survey, vary_tens) {
                            (Some(h), _, _) => rbb_compare::probe::Variation::MorphTo(
                                Hand::from_pbn(h).ok_or_else(|| format!("bad --morph-to {h:?}"))?,
                            ),
                            (None, true, _) => rbb_compare::probe::Variation::Survey,
                            (None, false, true) => rbb_compare::probe::Variation::Tens,
                            _ => rbb_compare::probe::Variation::None,
                        }
                    },
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
                our_changes: our_set,
                vuls: vuls
                    .iter()
                    .map(|v| {
                        Vulnerability::from_pbn(v)
                            .ok_or_else(|| format!("bad vulnerability {v:?}: None, NS, EW or All"))
                    })
                    .collect::<std::result::Result<_, _>>()?,
            };
            probe(&opts)
        }
        Command::Grid {
            spec,
            all,
            pbs,
            rules,
            bba_cli,
            out,
        } => {
            use rbb_compare::grid;
            let s = grid::read_spec(&spec)?;
            let name = spec
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "grid".into());
            let env = grid::Env {
                pbs,
                rules,
                bba_cli,
                out_dir: out.unwrap_or_else(|| PathBuf::from(".rbb-cache/grids").join(&name)),
            };
            let g = grid::run(&s, &env)?;
            println!(
                "{}: {} {:?}, {} decides after `{}`; card {}{}",
                spec.display(),
                g.rows.len(),
                g.mode,
                g.seat.to_char(),
                s.prefix,
                s.card,
                if s.our.is_empty() {
                    String::new()
                } else {
                    format!("; ours with {}", s.our.join(" "))
                }
            );
            if g.mode == grid::Mode::Survey && !all {
                let changed = g.changes_from_first();
                print!("\n{}", g.survey_summary());
                println!(
                    "\n{} of {} exchanges change BBA's call:",
                    changed.len(),
                    g.rows.len() - 1
                );
                let mut shown = vec![&g.rows[0]];
                shown.extend(changed);
                print!("{}", g.table(&shown));
            } else {
                println!();
                print!("{}", g.table(&g.rows.iter().collect::<Vec<_>>()));
            }
            println!("\n(table in {}/grid.tsv)", g.out_dir.display());
            Ok(())
        }
        Command::Simulate {
            spec,
            pool,
            samples,
            seed,
            tsv,
        } => {
            let s = rbb_compare::grid::read_spec(&spec)?;
            let pool = rbb_compare::simulate::read_pool(&pool)?;
            let rows = rbb_compare::simulate::run(&s, &pool, samples, seed)?;
            let lw = rows.iter().map(|r| r.label.len()).max().unwrap_or(5).max(5);
            println!(
                "{:lw$}  {:18} {:>5} {:>6} {:>6} {:>6}",
                "", "hand", "n", "tricks", "8+", "9+"
            );
            let mut out = String::from("label\thand\tn\ttricks\tmake8\tmake9\n");
            for r in &rows {
                println!(
                    "{:lw$}  {:18} {:>5} {:>6.2} {:>6.2} {:>6.2}",
                    r.label, r.hand, r.samples, r.mean_tricks, r.make8, r.make9
                );
                out += &format!(
                    "{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\n",
                    r.label, r.hand, r.samples, r.mean_tricks, r.make8, r.make9
                );
            }
            if let Some(p) = tsv {
                std::fs::write(&p, out)?;
            }
            Ok(())
        }
        Command::BidPbn {
            input,
            output,
            ns_card,
            ew_card,
            card_changes,
            scoring,
            all_meanings,
            auction_prefix,
            event,
            max_calls,
            rules,
        } => {
            use bridge_types::{Call, ScoringMethod};
            let rules = rules_from(rules.as_deref())?;
            let ew_card = ew_card.unwrap_or_else(|| ns_card.clone());
            let mut cards = [
                card_or_stock(&rules.vocab, &ns_card)?,
                card_or_stock(&rules.vocab, &ew_card)?,
            ];
            for card in &mut cards {
                for change in &card_changes {
                    card.apply_change(change)?;
                }
            }
            let engine = rbb_engine::Engine::new(&cards[0], &cards[1], &rules);
            let opts = bid_pbn::Options {
                prefix: auction_prefix
                    .as_deref()
                    .unwrap_or("")
                    .split_whitespace()
                    .map(|c| Call::from_pbn(c).ok_or_else(|| format!("bad call {c:?}")))
                    .collect::<std::result::Result<_, _>>()?,
                scoring: scoring
                    .map(|s| match ScoringMethod::from_pbn(&s) {
                        Some(m @ (ScoringMethod::Matchpoints | ScoringMethod::IMP)) => Ok(m),
                        _ => Err("scoring must be MP or IMP"),
                    })
                    .transpose()?,
                all_meanings,
                event,
                max_calls,
                ns_card,
                ew_card,
            };
            let started = std::time::Instant::now();
            let stats = bid_pbn::run_files(&input, &output, &engine, &opts)?;
            eprintln!(
                "{} deals, {} auctions, {} errors{} ({:.1}s)",
                stats.deals,
                stats.auctions,
                stats.errors,
                if stats.runaway > 0 {
                    format!(", {} stopped at {max_calls} calls", stats.runaway)
                } else {
                    String::new()
                },
                started.elapsed().as_secs_f64()
            );
            if stats.auctions == 0 {
                return Err("no auctions generated".into());
            }
            Ok(())
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
            knowledge,
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
            knowledge,
        ),
        Command::ExplainAuction {
            auction,
            deal,
            dealer,
            vul,
            scoring,
            card,
            ew_card,
            card_changes,
            rules,
            json,
        } => explain_auction(
            &auction,
            deal.as_deref(),
            dealer,
            &vul,
            &scoring,
            [&card, ew_card.as_deref().unwrap_or(&card)],
            &card_changes,
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
    if !opts.our_changes.is_empty() {
        println!("our engine with {:?}", opts.our_changes);
    }
    let labelled = report.rows.iter().any(|r| r.label.is_some());
    let survey = report.rows.first().and_then(|r| r.label.as_deref()) == Some("base");
    let by_vul = !opts.vuls.is_empty();
    println!(
        "\n  {}{}seat {:18} {:>3} {:>4} {:>6} {:>6}  {:>5} {:>5}",
        if labelled {
            format!("{:22} ", "change")
        } else {
            String::new()
        },
        if by_vul {
            format!("{:5} ", "vul")
        } else {
            String::new()
        },
        "hand",
        "HCP",
        "tens",
        "NT pts",
        "suit",
        "BBA",
        "ours"
    );
    let call = |c: &Option<bridge_types::Call>| c.as_ref().map_or("-".into(), rbb_compare::short);
    let base_call = report.rows.first().and_then(|r| r.reference.clone());
    let mut flips = 0;
    for (i, r) in report.rows.iter().enumerate() {
        // A survey lists the base and the changes that flip BBA's call.
        let flipped = r.reference != base_call;
        flips += (i > 0 && flipped) as usize;
        if survey && i > 0 && !flipped {
            continue;
        }
        let quarters = |q: i32| format!("{}{}", q / 4, ["", "¼", "½", "¾"][(q % 4) as usize]);
        let mark = if r.reference == r.ours { "" } else { "  ≠" };
        let label = if labelled {
            format!("{:22} ", r.label.as_deref().unwrap_or(""))
        } else {
            String::new()
        };
        let vul = if by_vul {
            format!("{:5} ", r.board.vul.to_pbn())
        } else {
            String::new()
        };
        println!(
            "  {label}{vul}{:4} {:18} {:>3} {:>4} {:>6} {:>6}  {:>5} {:>5}{mark}",
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
    if survey {
        println!(
            "\n{flips} of {} single-card exchanges change BBA's call from {}",
            report.rows.len() - 1,
            call(&base_call)
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
    filter: rbb_compare::AuctionFilter,
    top: usize,
    by_imps: bool,
    worst: usize,
    json: Option<&Path>,
) -> Result<()> {
    let started = std::time::Instant::now();
    let mut report = rbb_compare::run(opts, &|done, total| {
        eprint!("\r{done}/{total} boards");
    })?;
    if filter != rbb_compare::AuctionFilter::All {
        report.retain(filter);
        println!("{} (in BBA's auction)", filter.label());
    }
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
            "vs BBA, par as the yardstick ({} boards with differing contracts): ours closer {}, BBA closer {}, equal {}; net {:+} IMPs to us, {:+.2} per board",
            t.par.scored,
            t.par.ours_closer,
            t.par.reference_closer,
            t.par.equal,
            t.par.imps_vs_reference,
            t.par.imps_vs_reference as f64 / t.boards.max(1) as f64
        );
        println!(
            "vs BBA, errors (an overbid taken as doubled, each side charged with its own): net {:+} IMPs to us, {:+.2} per board; contract {:+}, doubling {:+}",
            t.par.errors_vs_reference,
            t.par.errors_vs_reference as f64 / t.boards.max(1) as f64,
            t.par.contract_errors_vs_reference,
            t.par.doubling_errors_vs_reference
        );
        let with_par: usize = t.par_classes.values().map(|c| c[0].boards).sum();
        println!(
            "how each table met par ({with_par} boards; IMPs are the table's errors):\n  {:48} {:>14} {:>8}   {:>14} {:>8}",
            "", "BBA boards", "IMPs", "ours boards", "IMPs"
        );
        for (class, [r, o]) in &t.par_classes {
            let pc = |n: usize| 100.0 * n as f64 / with_par.max(1) as f64;
            println!(
                "  {:48} {:>7} {:>5.1}% {:>8}   {:>7} {:>5.1}% {:>8}",
                class.describe(),
                r.boards,
                pc(r.boards),
                r.imps,
                o.boards,
                pc(o.boards),
                o.imps
            );
        }
        let [rd, od] = t.doubles;
        let rate = |d: &rbb_compare::DoubleTally| 100.0 * d.taken as f64 / d.chances.max(1) as f64;
        println!(
            "penalty doubles (a contract bid above par that goes down is a chance):\n  BBA  doubled {} of {} chances ({:.1}%), doubled {} that made, bailed out {} times\n  ours doubled {} of {} chances ({:.1}%), doubled {} that made, bailed out {} times",
            rd.taken, rd.chances, rate(&rd), rd.bad, rd.bailed_out,
            od.taken, od.chances, rate(&od), od.bad, od.bailed_out
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

    // Our calls partner reads by a higher rule than the one that chose
    // them (docs/JUDGMENT-LAYER.md, "Reading partner's judgment call").
    println!(
        "\ncalls read as a higher rule than chose them: {}",
        t.read_as
    );
    let prio = |p: Option<i64>| p.map(|p| format!(" ({p})")).unwrap_or_default();
    for r in s.read_as.iter().take(top) {
        println!(
            "  {:>5}  {:>5}  {}{} -> {}{}   {}",
            r.count,
            r.call,
            r.chosen,
            prio(r.chosen_priority),
            r.read.as_deref().unwrap_or("(no rule)"),
            prio(r.read_priority),
            r.scenarios.first().cloned().unwrap_or_default()
                + &if r.scenarios.len() > 1 {
                    format!(" +{}", r.scenarios.len() - 1)
                } else {
                    String::new()
                }
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
        println!("\nmost expensive divergence points (IMPs vs BBA, par as the yardstick; + means ours was closer):");
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
            "  {:32} {:>6} {:>7} {:>8} {:>9} {:>8} {:>7}",
            "scenario", "boards", "calls", "auction", "contract", "vs BBA", "bba/bd"
        );
        for sc in by.iter().take(worst) {
            println!(
                "  {:32} {:>6} {:>7} {:>8} {:>9} {:>+8} {:>+7.2}",
                sc.name,
                sc.boards,
                pct(sc.calls_all().rate()),
                pct(sc.auction_rate()),
                pct(sc.contract_rate()),
                sc.par.imps_vs_reference,
                sc.par_per_board()
            );
        }
    }
    if let Some(path) = json {
        std::fs::write(path, serde_json::to_string(&report)?)?;
        eprintln!("wrote {}", path.display());
    }
    Ok(())
}

fn diagnostics(d: Vec<bidspec::Diagnostic>) -> String {
    d.iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The rule set built into rbb: its rules and their card vocabulary.
fn embedded_rules() -> std::result::Result<RuleSet, Vec<bidspec::Diagnostic>> {
    rbb_engine::compile_rules(
        rbb_assets::MANIFEST,
        rbb_assets::FIELDS,
        rbb_assets::BBSA_MAP,
        rbb_assets::RULE_FILES.iter().copied(),
    )
}

/// The rule set in `dir` (its `.bid` files and `card/` vocabulary), or with
/// no directory the one built into rbb.
fn rules_from(dir: Option<&Path>) -> Result<RuleSet> {
    match dir {
        Some(d) => rbb_engine::load_rules(d),
        None => embedded_rules(),
    }
    .map_err(|d| diagnostics(d).into())
}

/// The card vocabulary of the rules in `dir`: its own `card/` files, or
/// the standard one (`rbb_engine::rules_vocabulary`).
fn vocab_from(dir: &Path) -> Result<Vocabulary> {
    Ok(rbb_engine::rules_vocabulary(dir)?)
}

fn load_card(vocab: &Vocabulary, path: &Path) -> Result<Card> {
    let text = read(path)?;
    if path.extension().is_some_and(|e| e == "bbsa") {
        Ok(bbsa::import(vocab, &text, None)?.0)
    } else {
        let (card, report) = Card::from_json(vocab, &text)?;
        for key in &report.ignored {
            eprintln!(
                "info: {}: {key} ignored (not a card setting)",
                path.display()
            );
        }
        Ok(card)
    }
}

/// A card from a file (.bbsa or card JSON) or, when no such file exists,
/// the stock card of that name built into rbb.
fn card_or_stock(vocab: &Vocabulary, spec: &str) -> Result<Card> {
    let path = Path::new(spec);
    if path.exists() {
        return load_card(vocab, path);
    }
    match rbb_assets::card(spec) {
        Some(text) => Ok(bbsa::import(vocab, text, Some(spec))?.0),
        None => Err(format!(
            "{spec}: no such file, and no stock card of that name ({})",
            rbb_assets::CARDS
                .iter()
                .map(|(n, _)| *n)
                .collect::<Vec<_>>()
                .join(", ")
        )
        .into()),
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
    knowledge: bool,
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
    let rules = rules_from(Some(rules))?;
    let ns = load_card(&rules.vocab, card)?;
    let ew = match ew_card {
        Some(p) => load_card(&rules.vocab, p)?,
        None => ns.clone(),
    };
    let engine = rbb_engine::Engine::new(&ns, &ew, &rules);
    let d = engine.bid(&hand, dealer, vul, scoring, &calls);
    // The review, with this hand's own view at its seat.
    let rows = knowledge.then(|| {
        let mut hands: [Option<Hand>; 4] = Default::default();
        hands[d.auction.position.next_caller().to_index()] = Some(hand.clone());
        engine.review(dealer, vul, scoring, &calls, &hands)
    });
    if json {
        match rows {
            Some(rows) => println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "decision": d,
                    "knowledge": rows,
                }))?
            ),
            None => println!("{}", serde_json::to_string_pretty(&d)?),
        }
        return Ok(());
    }
    if let Some(rows) = &rows {
        print!("{}", rbb_engine::review_text(rows));
        println!();
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

#[allow(clippy::too_many_arguments)]
fn explain_auction(
    auction: &str,
    deal: Option<&str>,
    dealer: char,
    vul: &str,
    scoring: &str,
    cards: [&str; 2],
    card_changes: &[String],
    rules: &Path,
    json: bool,
) -> Result<()> {
    use bridge_types::{Call, Deal, Direction, Hand, ScoringMethod, Vulnerability};
    let scoring = ScoringMethod::from_pbn(scoring).ok_or("scoring must be MP or IMP")?;
    let calls = auction
        .split_whitespace()
        .map(|c| Call::from_pbn(c).ok_or_else(|| format!("bad call {c:?}")))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let dealer =
        Direction::from_char(dealer.to_ascii_uppercase()).ok_or("dealer must be N, E, S or W")?;
    let vul = Vulnerability::from_pbn(vul).ok_or("vulnerability must be None, NS, EW or All")?;
    let mut a = bridge_types::Auction::new(dealer);
    for (i, c) in calls.iter().enumerate() {
        if a.is_complete() || !a.is_legal(c) {
            return Err(format!("call {} ({c}) is not legal here", i + 1).into());
        }
        a.add_call(c.clone());
    }
    let mut hands: [Option<Hand>; 4] = Default::default();
    if let Some(d) = deal {
        let deal = Deal::from_pbn(d).ok_or("deal must be PBN, e.g. \"N:AK52.KJ7.Q94.K83 ...\"")?;
        for seat in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            hands[seat.to_index()] = Some(deal.hand(seat).clone());
        }
    }
    let rules = rules_from(Some(rules))?;
    let mut cards = [
        card_or_stock(&rules.vocab, cards[0])?,
        card_or_stock(&rules.vocab, cards[1])?,
    ];
    for card in &mut cards {
        for change in card_changes {
            card.apply_change(change)?;
        }
    }
    let engine = rbb_engine::Engine::new(&cards[0], &cards[1], &rules);
    let rows = engine.review(dealer, vul, scoring, &calls, &hands);
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        print!("{}", rbb_engine::review_text(&rows));
    }
    Ok(())
}

fn bid(cmd: BidCommand) -> Result<()> {
    match cmd {
        BidCommand::Check { paths, rules } => {
            let vocab = vocab_from(&rules)?;
            let mut warnings = 0;
            for w in vocab.lint() {
                eprintln!("{}: warning: {w}", rules.join("card").display());
                warnings += 1;
            }
            let mut files = Vec::new();
            for p in &paths {
                collect_bid_files(p, &mut files)?;
            }
            files.sort();
            let mut modules = Vec::new();
            let mut errors = 0;
            // Each directory's manifest: the language version it asks for.
            for dir in paths.iter().filter(|p| p.is_dir()) {
                match rbb_engine::read_manifest(dir) {
                    Ok(Some(m)) => {
                        for k in &m.unknown_keys {
                            eprintln!(
                                "{}: warning: unknown key `{k}`",
                                dir.join(bidspec::manifest::FILE).display()
                            );
                            warnings += 1;
                        }
                        println!(
                            "{}: {} (rule language {})",
                            dir.display(),
                            m.name,
                            m.language
                        );
                    }
                    Ok(None) => {
                        eprintln!(
                            "{}: warning: no {}; read as rule language {}",
                            dir.display(),
                            bidspec::manifest::FILE,
                            rbb_engine::LANGUAGE_VERSION
                        );
                        warnings += 1;
                    }
                    Err(d) => {
                        eprintln!("{d}");
                        errors += 1;
                    }
                }
            }
            for file in &files {
                match bidspec::compile(&read(file)?, &file.display().to_string(), vocab.registry())
                {
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
            for e in rbb_engine::check_card_refs(&modules, vocab.registry()) {
                eprintln!("{e}");
                errors += 1;
            }
            for d in rbb_engine::check_terms(&modules) {
                eprintln!("{d}");
                errors += 1;
            }
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
            // Teaching skills: every path named is a standard convention or
            // skill (convention-card's spec/conventions/).
            match bridge_card::standard::conventions() {
                Ok(known) => {
                    let fields_file = "fields.toml (convention-card spec/)".to_string();
                    for d in
                        bidspec::skills::check(&modules, vocab.registry(), &known, &fields_file)
                    {
                        if d.line == 0 {
                            eprintln!("{}: warning: {}", d.file, d.message);
                        } else {
                            eprintln!("{}:{}: warning: {}", d.file, d.line, d.message);
                        }
                        warnings += 1;
                    }
                }
                Err(e) => {
                    eprintln!("{e}");
                    errors += 1;
                }
            }
            let rules: usize = modules.iter().map(count_rules).sum();
            println!(
                "{} files, {} modules, {rules} rules: {errors} errors, {warnings} warnings \
                 (engine reads rule language {})",
                files.len(),
                modules.len(),
                rbb_engine::LANGUAGE_VERSION
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
        BidCommand::Reference {
            rules,
            json,
            output,
        } => {
            let rule_set = rules_from(rules.as_deref())?;
            let entries = bidspec::reference::entries(&rule_set.modules);
            let rules_id = if rules.is_some() {
                "from files".to_string()
            } else {
                rbb_assets::RULES_ID.to_string()
            };
            let header = format!(
                "rusty-bidding-bot conventions reference\nrbb {} rules {rules_id}",
                env!("CARGO_PKG_VERSION")
            );
            let text = bidspec::reference::text(&entries, &header, None);
            if let Some(p) = json {
                let v = serde_json::json!({"rules_id": rules_id, "modules": entries});
                fs::write(&p, serde_json::to_string_pretty(&v)?)
                    .map_err(|e| format!("{}: {e}", p.display()))?;
            }
            match output {
                Some(p) => fs::write(&p, text).map_err(|e| format!("{}: {e}", p.display()))?,
                None => print!("{text}"),
            }
        }
        BidCommand::Terms { doc } => {
            let terms = rbb_engine::terms_reference();
            match doc {
                None => print!("{terms}"),
                Some(p) => {
                    let text = read(&p)?;
                    let new = rbb_engine::splice_terms(&text, &terms).ok_or_else(|| {
                        format!(
                            "{}: no `{}` ... `{}` markers",
                            p.display(),
                            rbb_engine::TERMS_BEGIN,
                            rbb_engine::TERMS_END
                        )
                    })?;
                    if new != text {
                        fs::write(&p, new).map_err(|e| format!("{}: {e}", p.display()))?;
                        eprintln!("{}: term reference updated", p.display());
                    }
                }
            }
        }
        BidCommand::Skills { rules, doc } => {
            let vocab = vocab_from(&rules)?;
            let known = bridge_card::standard::conventions()?;
            let mut files = Vec::new();
            collect_bid_files(&rules, &mut files)?;
            files.sort();
            let mut modules = Vec::new();
            for file in &files {
                let m =
                    bidspec::compile(&read(file)?, &file.display().to_string(), vocab.registry())
                        .map_err(|diags| {
                        diags
                            .iter()
                            .map(|d| d.to_string())
                            .collect::<Vec<_>>()
                            .join("\n")
                    })?;
                modules.push(m);
            }
            let map = bidspec::skills::map(&modules, vocab.registry(), &known);
            let text = bidspec::skills::markdown(&map);
            match doc {
                None => print!("{text}"),
                Some(p) => {
                    let old = read(&p)?;
                    let new = bidspec::skills::splice(&old, &text).ok_or_else(|| {
                        format!(
                            "{}: no `{}` ... `{}` markers",
                            p.display(),
                            bidspec::skills::BEGIN,
                            bidspec::skills::END
                        )
                    })?;
                    if new != old {
                        fs::write(&p, new).map_err(|e| format!("{}: {e}", p.display()))?;
                        eprintln!("{}: skill map updated", p.display());
                    }
                }
            }
        }
        BidCommand::Compile {
            file,
            output,
            rules,
        } => {
            let vocab = vocab_from(&rules)?;
            let module =
                bidspec::compile(&read(&file)?, &file.display().to_string(), vocab.registry())
                    .map_err(diagnostics)?;
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
        CardCommand::ImportBbsa {
            file,
            output,
            wrap,
            rules,
        } => {
            let vocab = vocab_from(&rules)?;
            let text = read(&file)?;
            let name = file.file_stem().and_then(|s| s.to_str());
            let (card, report) = bbsa::import(&vocab, &text, name)?;
            let json = if wrap {
                card.to_export_json_string(Some(&now_iso8601()))
            } else {
                card.to_json_string()
            };
            write(output.as_deref(), &json)?;
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
        CardCommand::ExportBbsa {
            file,
            output,
            rules,
        } => {
            let vocab = vocab_from(&rules)?;
            let (card, load) = Card::from_json(&vocab, &read(&file)?)?;
            print_load_report(&load);
            let (text, report) = bbsa::export(&card);
            write(output.as_deref(), &text)?;
            for key in &report.dropped {
                eprintln!("warning: {key} is not in the current .bbsa layout; not written");
            }
        }
        CardCommand::Check { file, rules } => {
            let vocab = vocab_from(&rules)?;
            let (card, load) = Card::from_json(&vocab, &read(&file)?)?;
            println!("{}: {} settings", file.display(), card.values().count());
            print_load_report(&load);
            if !load.is_clean() {
                return Err("card has unknown or invalid fields".into());
            }
        }
        CardCommand::Schema { output, rules } => {
            let vocab = vocab_from(&rules)?;
            let text = serde_json::to_string_pretty(&schema::json_schema(vocab.registry()))?;
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
    let rules = rules_from(Some(&opts.rules)).map_err(|_| "rule files have errors")?;
    let read = coverage::fields_read(&rules.modules, &rules.vocab);
    let scenarios = rbb_compare::discover(&opts.pbs, &opts.scenarios)?;
    let mut score: HashMap<String, f64> = HashMap::new();
    let mut of = |name: &str| -> f64 {
        if let Some(s) = score.get(name) {
            return *s;
        }
        let path = opts.pbs.join("bbsa").join(format!("{name}.bbsa"));
        let s = coverage::load(&rules.vocab, &path)
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
    let rules = rbb_engine::load_rules(rules).map_err(|d| {
        for e in &d {
            eprintln!("{e}");
        }
        "rule files have errors"
    })?;
    let read = coverage::fields_read(&rules.modules, &rules.vocab);
    println!(
        "{} modules read {} card fields\n",
        rules.modules.len(),
        read.len()
    );
    println!(
        "{:24} {:14} {:>6} {:>8} {:>9} {:>9}",
        "card", "system", "read", "ignored", "unmapped", "coverage"
    );
    let mut covs = Vec::new();
    for f in files {
        let (name, card, unmapped) = coverage::load(&rules.vocab, f)?;
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
                println!("{}: settings with no card field", cov.name);
                for k in &cov.unmapped {
                    println!("  {k}");
                }
            }
        }
    }
    Ok(())
}

fn print_load_report(load: &bridge_card::LoadReport) {
    if let Some(schema) = &load.wrapper {
        eprintln!("info: Bridge-Classroom export ({schema}): read its card_data");
    }
    for key in &load.ignored {
        eprintln!("info: {key} ignored (not a card setting: a raw import record)");
    }
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

/// The time now in ISO 8601 (UTC, milliseconds), as JavaScript's
/// `Date.toISOString` writes Bridge-Classroom's `exportedAt`.
fn now_iso8601() -> String {
    let since = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = since.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        since.subsec_millis()
    )
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
