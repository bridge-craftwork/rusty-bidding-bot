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
    }
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
                eprintln!("{} keys have no card field (kept in bba_passthrough):", report.passthrough.len());
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
