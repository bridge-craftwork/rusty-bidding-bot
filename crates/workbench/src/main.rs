//! rbb workbench: run the comparison with BBA, browse where the engine
//! departs from it, and see why, with a re-run whenever a rule file is saved.

mod app;
mod detail;
mod knowledge;
mod ticket;

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
    #[arg(long, default_value = "cards/bbsa")]
    cards: PathBuf,
    /// At most this many boards per scenario.
    #[arg(short, long)]
    limit: Option<usize>,
    /// Scenarios to load: names, or patterns with `*` and `?` (e.g.
    /// 'Basic_*'); all when none are given. Quote a pattern so the shell
    /// does not try to expand it into filenames.
    scenarios: Vec<String>,
    /// Command to open a rule in an editor; {file} and {line} are replaced.
    #[arg(long, default_value = "code -g {file}:{line}")]
    editor: String,
    /// A card change for both sides, `path=value` (repeatable), e.g.
    /// `--set general.style=bba` to compare BBA's treatments.
    #[arg(long)]
    set: Vec<String>,
    /// GitHub repository for "Report…" issues, `owner/repo`; by default
    /// the one the origin remote names.
    #[arg(long)]
    ticket_repo: Option<String>,
    /// Print the `gh` commands a GitHub ticket would run instead of running
    /// them (also: RBB_TICKET_DRY_RUN=1). The local copy is still written.
    #[arg(long)]
    ticket_dry_run: bool,
    /// Select this board when the comparison has run (its number, e.g.
    /// `595`; `SCENARIO:595` when several scenarios are loaded) and open
    /// its knowledge view.
    #[arg(long)]
    board: Option<String>,
}

fn main() -> eframe::Result {
    let args = Args::parse();
    let tickets = app::TicketOptions {
        repo: args.ticket_repo,
        dry_run: args.ticket_dry_run || ticket::dry_run_from_env(),
    };
    let opts = rbb_compare::Options {
        pbs: args.pbs,
        scenarios: args.scenarios,
        limit: args.limit,
        rules: args.rules,
        par: false,
        dd_cache: PathBuf::from(".rbb-cache/dd.jsonl"),
        card_changes: args.set,
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
        Box::new(move |_cc| {
            let mut app = app::App::new(opts, args.cards, args.editor, tickets);
            app.show_board(args.board);
            Ok(Box::new(app))
        }),
    )
}
