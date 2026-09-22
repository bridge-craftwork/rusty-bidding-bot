//! rbb workbench: run the comparison with BBA, browse where the engine
//! departs from it, and see why, with a re-run whenever a rule file is saved.

mod app;
mod detail;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "rbb-workbench",
    about = "Compare the engine with BBA and inspect divergences"
)]
struct Args {
    /// Practice-Bidding-Scenarios checkout.
    #[arg(long, default_value = "../Practice-Bidding-Scenarios")]
    pbs: PathBuf,
    /// Directory of .bid modules (watched for changes).
    #[arg(long, default_value = "conventions")]
    rules: PathBuf,
    /// Where `.test` files look up card names.
    #[arg(long, default_value = "crates/bridge-card/tests/fixtures/bbsa")]
    cards: PathBuf,
    /// At most this many boards per scenario.
    #[arg(short, long)]
    limit: Option<usize>,
    /// Scenarios to load; all when none are given.
    scenarios: Vec<String>,
    /// Command to open a rule in an editor; {file} and {line} are replaced.
    #[arg(long, default_value = "code -g {file}:{line}")]
    editor: String,
}

fn main() -> eframe::Result {
    let args = Args::parse();
    let opts = rbb_compare::Options {
        pbs: args.pbs,
        scenarios: args.scenarios,
        limit: args.limit,
        rules: args.rules,
        par: false,
        dd_cache: PathBuf::from(".rbb-cache/dd.jsonl"),
    };
    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("rbb workbench")
            .with_inner_size([1500.0, 950.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rbb workbench",
        native,
        Box::new(move |_cc| Ok(Box::new(app::App::new(opts, args.cards, args.editor)))),
    )
}
