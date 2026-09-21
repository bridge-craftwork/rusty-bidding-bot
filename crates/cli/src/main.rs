use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use bridge_card::{bbsa, schema, Card};
use clap::{Parser, Subcommand};

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
        Command::Call {
            hand,
            auction,
            dealer,
            vul,
            card,
            ew_card,
            rules,
            json,
        } => call(
            &hand,
            &auction,
            dealer,
            &vul,
            &card,
            ew_card.as_deref(),
            &rules,
            json,
        ),
    }
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
    card: &Path,
    ew_card: Option<&Path>,
    rules: &Path,
    json: bool,
) -> Result<()> {
    use bridge_types::{Call, Direction, Hand, Vulnerability};
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
    let d = engine.bid(&hand, dealer, vul, &calls);
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
