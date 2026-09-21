//! The workbench window: toolbar, summary, scenario list, divergence and
//! board lists, and the board detail.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant, SystemTime};

use egui::{Color32, RichText, Sense};
use egui_extras::{Column, TableBuilder};
use rbb_compare::{short, BoardResult, Engines, Options, Report, Stats};

use crate::detail::Detail;

pub const GOOD: Color32 = Color32::from_rgb(60, 170, 90);
pub const BAD: Color32 = Color32::from_rgb(210, 80, 70);

type Outcome = Result<(Report, Arc<Engines>), String>;

struct Running {
    started: Instant,
    done: Arc<AtomicUsize>,
    total: Arc<AtomicUsize>,
    rx: mpsc::Receiver<Outcome>,
}

struct Loaded {
    report: Report,
    engines: Arc<Engines>,
    took: Duration,
}

/// Per-board outcome of a run, to show what changed on the next one.
#[derive(Clone, Copy, PartialEq)]
struct Mark {
    identical: bool,
    same_contract: bool,
    agree: usize,
}

#[derive(Default)]
struct Delta {
    now_identical: usize,
    no_longer_identical: usize,
    now_same_contract: usize,
    lost_same_contract: usize,
    calls: i64,
    /// Boards that got worse, then boards that got better (indices).
    worse: Vec<usize>,
    better: Vec<usize>,
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Divergences,
    Boards,
    Changes,
}

#[derive(Clone, Copy, PartialEq)]
enum SortBy {
    Calls,
    Auction,
    Contract,
    Name,
}

/// One row of the divergence table.
struct DivRow {
    auction: String,
    reference: String,
    ours: String,
    boards: Vec<usize>,
}

pub struct App {
    opts: Options,
    editor: String,
    running: Option<Running>,
    loaded: Option<Loaded>,
    error: Option<String>,
    previous: Option<HashMap<(String, String), Mark>>,
    delta: Option<Delta>,
    auto: bool,
    rules_mtime: Option<SystemTime>,
    last_poll: Instant,
    pending: bool,
    limit_text: String,
    /// Scenarios to load, space-separated; empty for all.
    scenarios_text: String,
    scenario: Option<String>,
    sort: SortBy,
    tab: Tab,
    divs: Vec<DivRow>,
    div: Option<usize>,
    /// Text filter on the divergence table.
    div_filter: String,
    board: Option<usize>,
    detail: Option<Detail>,
}

impl App {
    pub fn new(opts: Options, editor: String) -> App {
        let limit_text = opts.limit.map(|n| n.to_string()).unwrap_or_default();
        let scenarios_text = opts.scenarios.join(" ");
        let mut app = App {
            rules_mtime: rules_mtime(&opts.rules),
            opts,
            editor,
            running: None,
            loaded: None,
            error: None,
            previous: None,
            delta: None,
            auto: true,
            last_poll: Instant::now(),
            pending: true,
            limit_text,
            scenarios_text,
            scenario: None,
            sort: SortBy::Calls,
            tab: Tab::Divergences,
            divs: Vec::new(),
            div: None,
            div_filter: String::new(),
            board: None,
            detail: None,
        };
        app.pending = true;
        app
    }

    fn start(&mut self, ctx: &egui::Context) {
        if self.running.is_some() {
            self.pending = true;
            return;
        }
        self.pending = false;
        self.opts.limit = self.limit_text.trim().parse().ok();
        self.opts.scenarios = self
            .scenarios_text
            .split_whitespace()
            .map(str::to_string)
            .collect();
        // A scenario selected in the sidebar may no longer be loaded.
        if let Some(s) = &self.scenario {
            if !self.opts.scenarios.is_empty() && !self.opts.scenarios.contains(s) {
                self.scenario = None;
            }
        }
        let opts = self.opts.clone();
        let done = Arc::new(AtomicUsize::new(0));
        let total = Arc::new(AtomicUsize::new(0));
        let (tx, rx) = mpsc::channel();
        let (d, t, c) = (done.clone(), total.clone(), ctx.clone());
        std::thread::spawn(move || {
            let outcome = Engines::new(&opts.pbs, &opts.rules).and_then(|engines| {
                let engines = Arc::new(engines);
                rbb_compare::run_with(&opts, &engines, &|n, of| {
                    d.store(n, Ordering::Relaxed);
                    t.store(of, Ordering::Relaxed);
                    c.request_repaint();
                })
                .map(|r| (r, engines))
            });
            let _ = tx.send(outcome);
            c.request_repaint();
        });
        self.running = Some(Running {
            started: Instant::now(),
            done,
            total,
            rx,
        });
    }

    fn finish(&mut self, outcome: Outcome, took: Duration) {
        match outcome {
            Err(e) => self.error = Some(e),
            Ok((report, engines)) => {
                self.error = None;
                let marks: HashMap<(String, String), Mark> = report
                    .boards
                    .iter()
                    .map(|b| ((b.scenario.clone(), b.board.clone()), mark(b)))
                    .collect();
                self.delta = self.previous.as_ref().map(|prev| {
                    let mut d = Delta::default();
                    for (i, b) in report.boards.iter().enumerate() {
                        let Some(p) = prev.get(&(b.scenario.clone(), b.board.clone())) else {
                            continue;
                        };
                        let m = mark(b);
                        d.calls += m.agree as i64 - p.agree as i64;
                        match (p.identical, m.identical) {
                            (false, true) => d.now_identical += 1,
                            (true, false) => d.no_longer_identical += 1,
                            _ => {}
                        }
                        match (p.same_contract, m.same_contract) {
                            (false, true) => d.now_same_contract += 1,
                            (true, false) => d.lost_same_contract += 1,
                            _ => {}
                        }
                        if m.agree < p.agree || (p.identical && !m.identical) {
                            d.worse.push(i);
                        } else if m.agree > p.agree || (!p.identical && m.identical) {
                            d.better.push(i);
                        }
                    }
                    d
                });
                self.previous = Some(marks);
                // Keep the selected board if it is still there.
                let selected = self.board.and_then(|i| {
                    self.loaded.as_ref().map(|l| {
                        (
                            l.report.boards[i].scenario.clone(),
                            l.report.boards[i].board.clone(),
                        )
                    })
                });
                self.loaded = Some(Loaded {
                    report,
                    engines,
                    took,
                });
                self.board = selected.and_then(|(s, b)| {
                    self.loaded
                        .as_ref()?
                        .report
                        .boards
                        .iter()
                        .position(|x| x.scenario == s && x.board == b)
                });
                self.detail = None;
                self.rebuild_divs();
                if let Some(i) = self.board {
                    self.select_board(i);
                }
            }
        }
    }

    fn rebuild_divs(&mut self) {
        // Keep the selected divergence selected across re-runs.
        let keep = self
            .div
            .and_then(|i| self.divs.get(i))
            .map(|d| (d.auction.clone(), d.reference.clone(), d.ours.clone()));
        self.divs.clear();
        self.div = None;
        let Some(l) = &self.loaded else { return };
        let mut index: HashMap<(String, String, String), usize> = HashMap::new();
        for (i, b) in l.report.boards.iter().enumerate() {
            if self.scenario.as_ref().is_some_and(|s| s != &b.scenario) {
                continue;
            }
            let Some(d) = b.first_divergence else {
                continue;
            };
            let key = (
                b.reference[..d]
                    .iter()
                    .map(short)
                    .collect::<Vec<_>>()
                    .join(" "),
                short(&b.reference[d]),
                short(&b.replay[d]),
            );
            let n = *index.entry(key.clone()).or_insert_with(|| {
                self.divs.push(DivRow {
                    auction: key.0.clone(),
                    reference: key.1.clone(),
                    ours: key.2.clone(),
                    boards: vec![],
                });
                self.divs.len() - 1
            });
            self.divs[n].boards.push(i);
        }
        self.divs.sort_by(|a, b| {
            b.boards
                .len()
                .cmp(&a.boards.len())
                .then(a.auction.cmp(&b.auction))
        });
        if let Some((a, r, o)) = keep {
            self.div = self
                .divs
                .iter()
                .position(|d| d.auction == a && d.reference == r && d.ours == o);
        }
    }

    /// Boards shown in the Boards tab: the selected divergence's, else the
    /// selected scenario's, else all.
    fn board_list(&self) -> Vec<usize> {
        let Some(l) = &self.loaded else { return vec![] };
        if let Some(d) = self.div.and_then(|d| self.divs.get(d)) {
            return d.boards.clone();
        }
        l.report
            .boards
            .iter()
            .enumerate()
            .filter(|(_, b)| self.scenario.as_ref().is_none_or(|s| s == &b.scenario))
            .map(|(i, _)| i)
            .collect()
    }

    fn select_board(&mut self, i: usize) {
        self.board = Some(i);
        self.detail = self
            .loaded
            .as_ref()
            .map(|l| Detail::compute(&l.report.boards[i], &l.engines));
    }

    pub fn open_in_editor(&self, file: &str, line: usize) {
        let parts: Vec<String> = self
            .editor
            .split_whitespace()
            .map(|p| {
                p.replace("{file}", file)
                    .replace("{line}", &line.to_string())
            })
            .collect();
        if let Some((cmd, args)) = parts.split_first() {
            let _ = std::process::Command::new(cmd).args(args).spawn();
        }
    }
}

fn mark(b: &BoardResult) -> Mark {
    Mark {
        identical: b.first_divergence.is_none(),
        same_contract: b.contracts_match(),
        agree: b
            .reference
            .iter()
            .zip(&b.replay)
            .filter(|(r, e)| r == e)
            .count(),
    }
}

/// Newest modification time of any `.bid` file under `dir`.
fn rules_mtime(dir: &Path) -> Option<SystemTime> {
    fn walk(dir: &Path, best: &mut Option<SystemTime>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p: PathBuf = e.path();
            if p.is_dir() {
                walk(&p, best);
            } else if p.extension().is_some_and(|x| x == "bid") {
                if let Ok(t) = e.metadata().and_then(|m| m.modified()) {
                    if best.is_none_or(|b| t > b) {
                        *best = Some(t);
                    }
                }
            }
        }
    }
    let mut best = None;
    walk(dir, &mut best);
    best
}

fn pct(x: f64) -> String {
    format!("{:.1}%", 100.0 * x)
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(r) = &self.running {
            if let Ok(outcome) = r.rx.try_recv() {
                let took = r.started.elapsed();
                self.running = None;
                self.finish(outcome, took);
            }
        }
        if self.last_poll.elapsed() > Duration::from_millis(700) {
            self.last_poll = Instant::now();
            let m = rules_mtime(&self.opts.rules);
            if m != self.rules_mtime {
                self.rules_mtime = m;
                if self.auto {
                    self.pending = true;
                }
            }
        }
        if self.pending && self.running.is_none() {
            self.start(ctx);
        }
        ctx.request_repaint_after(Duration::from_millis(800));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        egui::Panel::left("scenarios")
            .resizable(true)
            .default_size(360.0)
            .show(ui, |ui| self.scenarios(ui));
        egui::Panel::bottom("detail")
            .resizable(true)
            .default_size(470.0)
            .show(ui, |ui| {
                egui::ScrollArea::both().id_salt("detail-scroll").show(ui, |ui| {
                let action = match (&self.detail, self.board, &self.loaded) {
                    (Some(d), Some(i), Some(l)) => d.ui(ui, &l.report.boards[i]),
                    _ => {
                        ui.label("Select a board to see both auctions and the engine's reasoning.");
                        None
                    }
                };
                if let Some((file, line)) = action {
                    self.open_in_editor(&file, line);
                }
            });
            });
        egui::CentralPanel::default().show(ui, |ui| self.lists(ui));
    }
}

impl App {
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let running = self.running.is_some();
            if ui
                .add_enabled(!running, egui::Button::new("▶ Run"))
                .clicked()
            {
                self.pending = true;
            }
            ui.checkbox(&mut self.auto, "re-run when a .bid file is saved");
            ui.label("scenarios:");
            let r = ui
                .add(
                    egui::TextEdit::singleline(&mut self.scenarios_text)
                        .desired_width(160.0)
                        .hint_text("all"),
                )
                .on_hover_text(
                    "Scenario names separated by spaces; empty loads all 342. Press Enter to run.",
                );
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.pending = true;
            }
            ui.label("boards per scenario:");
            ui.add(
                egui::TextEdit::singleline(&mut self.limit_text)
                    .desired_width(50.0)
                    .hint_text("all"),
            );
            ui.checkbox(&mut self.opts.par, "par (slow first time)");
            ui.separator();
            ui.label(
                RichText::new(format!(
                    "rules: {}   PBS: {}",
                    self.opts.rules.display(),
                    self.opts.pbs.display()
                ))
                .weak(),
            );
        });
        if let Some(r) = &self.running {
            let (d, t) = (
                r.done.load(Ordering::Relaxed),
                r.total.load(Ordering::Relaxed),
            );
            let frac = if t == 0 { 0.0 } else { d as f32 / t as f32 };
            ui.add(egui::ProgressBar::new(frac).text(format!("comparing… {d}/{t} boards")));
        }
        if let Some(e) = &self.error {
            ui.label(RichText::new(format!("⚠ {e}")).color(BAD).monospace());
        }
        if let Some(l) = &self.loaded {
            let t = &l.report.summary.total;
            let calls = t.calls_all();
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(format!("calls agreeing with BBA: {}", pct(calls.rate())))
                        .strong(),
                );
                ui.label(format!(
                    "NS {}  EW {}",
                    pct(t.calls[0].rate()),
                    pct(t.calls[1].rate())
                ));
                ui.separator();
                ui.label(format!("identical auctions {}", pct(t.auction_rate())));
                ui.label(format!("same contract {}", pct(t.contract_rate())));
                if t.par.scored > 0 {
                    ui.separator();
                    ui.label(format!(
                        "vs par: ours closer {}, BBA closer {}, net {:+} IMPs",
                        t.par.ours_closer, t.par.reference_closer, t.par.imps_vs_reference
                    ));
                }
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "{} boards in {:.1}s",
                        t.boards,
                        l.took.as_secs_f64()
                    ))
                    .weak(),
                );
            });
            if let Some(d) = &self.delta {
                ui.horizontal_wrapped(|ui| {
                    ui.label("since last run:");
                    let signed = |n: i64| {
                        let c = if n > 0 {
                            GOOD
                        } else if n < 0 {
                            BAD
                        } else {
                            Color32::GRAY
                        };
                        RichText::new(format!("{n:+}")).color(c).strong()
                    };
                    ui.label(signed(d.calls));
                    ui.label("calls agree,");
                    ui.label(signed(d.now_identical as i64));
                    ui.label("/");
                    ui.label(signed(-(d.no_longer_identical as i64)));
                    ui.label("identical auctions,");
                    ui.label(signed(d.now_same_contract as i64));
                    ui.label("/");
                    ui.label(signed(-(d.lost_same_contract as i64)));
                    ui.label("same contract");
                });
            }
        }
    }

    fn scenarios(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scenarios");
        ui.horizontal(|ui| {
            ui.label("sort:");
            ui.selectable_value(&mut self.sort, SortBy::Calls, "calls");
            ui.selectable_value(&mut self.sort, SortBy::Auction, "auction");
            ui.selectable_value(&mut self.sort, SortBy::Contract, "contract");
            ui.selectable_value(&mut self.sort, SortBy::Name, "name");
        });
        let Some(l) = &self.loaded else { return };
        let mut rows: Vec<&Stats> = l.report.summary.scenarios.iter().collect();
        let key = |s: &Stats| match self.sort {
            SortBy::Calls => s.calls_all().rate(),
            SortBy::Auction => s.auction_rate(),
            SortBy::Contract => s.contract_rate(),
            SortBy::Name => 0.0,
        };
        if self.sort == SortBy::Name {
            rows.sort_by(|a, b| a.name.cmp(&b.name));
        } else {
            rows.sort_by(|a, b| {
                key(a)
                    .partial_cmp(&key(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        let all = self.scenario.is_none();
        let mut clicked = None;
        let mut show_all = false;
        if ui
            .selectable_label(all, format!("All scenarios ({})", rows.len()))
            .clicked()
            && !all
        {
            show_all = true;
        }
        TableBuilder::new(ui)
            .id_salt("scenario-table")
            .striped(true)
            .sense(Sense::click())
            .column(Column::remainder().at_least(140.0).clip(true))
            .columns(Column::auto(), 3)
            .header(20.0, |mut h| {
                h.col(|ui| {
                    ui.strong("scenario");
                });
                h.col(|ui| {
                    ui.strong("calls");
                });
                h.col(|ui| {
                    ui.strong("auction");
                });
                h.col(|ui| {
                    ui.strong("contract");
                });
            })
            .body(|body| {
                body.rows(18.0, rows.len(), |mut row| {
                    let s = rows[row.index()];
                    row.set_selected(self.scenario.as_deref() == Some(&s.name));
                    row.col(|ui| {
                        ui.label(&s.name);
                    });
                    row.col(|ui| {
                        ui.label(pct(s.calls_all().rate()));
                    });
                    row.col(|ui| {
                        ui.label(pct(s.auction_rate()));
                    });
                    row.col(|ui| {
                        ui.label(pct(s.contract_rate()));
                    });
                    if row.response().clicked() {
                        clicked = Some(s.name.clone());
                    }
                });
            });
        if show_all {
            self.scenario = None;
            self.rebuild_divs();
        }
        if let Some(name) = clicked {
            self.scenario = Some(name);
            self.rebuild_divs();
            self.tab = Tab::Divergences;
        }
    }

    fn lists(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, Tab::Divergences, "Divergences");
            ui.selectable_value(&mut self.tab, Tab::Boards, "Boards");
            let changes = self
                .delta
                .as_ref()
                .map_or(0, |d| d.worse.len() + d.better.len());
            ui.selectable_value(
                &mut self.tab,
                Tab::Changes,
                format!("Changes since last run ({changes})"),
            );
            ui.separator();
            ui.label(
                RichText::new(match &self.scenario {
                    Some(s) => format!("scenario: {s}"),
                    None => "all scenarios".into(),
                })
                .weak(),
            );
        });
        ui.separator();
        if self.loaded.is_none() {
            ui.label(if self.running.is_some() {
                "Running the comparison…"
            } else {
                "No results yet."
            });
            return;
        }
        match self.tab {
            Tab::Divergences => self.divergence_table(ui),
            Tab::Boards => {
                let list = self.board_list();
                if let Some(d) = self.div.and_then(|d| self.divs.get(d)) {
                    ui.label(format!(
                        "{} boards where, after `{}`, BBA bids {} and we bid {}",
                        list.len(),
                        d.auction,
                        d.reference,
                        d.ours
                    ));
                }
                self.board_table(ui, &list, "boards");
            }
            Tab::Changes => {
                let (worse, better) = self
                    .delta
                    .as_ref()
                    .map(|d| (d.worse.clone(), d.better.clone()))
                    .unwrap_or_default();
                ui.label(RichText::new(format!("worse: {} boards", worse.len())).color(BAD));
                self.board_table(ui, &worse, "worse");
                ui.label(RichText::new(format!("better: {} boards", better.len())).color(GOOD));
                self.board_table(ui, &better, "better");
            }
        }
    }

    fn divergence_table(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("filter:");
            ui.add(
                egui::TextEdit::singleline(&mut self.div_filter)
                    .desired_width(260.0)
                    .hint_text("e.g. 1NT P 2D P 2S"),
            );
            if !self.div_filter.is_empty() && ui.small_button("✕").clicked() {
                self.div_filter.clear();
            }
            ui.label(
                RichText::new(
                    "matches the auction followed by BBA's call, or either call alone; pick \"All scenarios\" to search everywhere",
                )
                .weak(),
            );
        });
        // Match "<auction so far> <BBA call>" so a whole auction can be pasted,
        // and also each call on its own.
        let needle = self.div_filter.trim().to_uppercase();
        let shown: Vec<usize> = (0..self.divs.len())
            .filter(|&i| {
                let d = &self.divs[i];
                needle.is_empty()
                    || format!("{} {}", d.auction, d.reference)
                        .trim()
                        .to_uppercase()
                        .contains(&needle)
                    || d.reference.to_uppercase() == needle
                    || d.ours.to_uppercase() == needle
            })
            .collect();
        let total: usize = shown.iter().map(|&i| self.divs[i].boards.len()).sum();
        ui.label(
            RichText::new(format!("{} divergence points, {total} boards", shown.len())).weak(),
        );
        let mut clicked = None;
        TableBuilder::new(ui)
            .id_salt("divergence-table")
            .striped(true)
            .sense(Sense::click())
            .column(Column::auto().at_least(50.0))
            .column(Column::initial(260.0).clip(true))
            .columns(Column::auto().at_least(40.0), 2)
            .column(Column::remainder().clip(true))
            .header(20.0, |mut h| {
                for t in ["boards", "auction so far", "BBA", "ours", "scenarios"] {
                    h.col(|ui| {
                        ui.strong(t);
                    });
                }
            })
            .body(|body| {
                body.rows(18.0, shown.len(), |mut row| {
                    let i = shown[row.index()];
                    let d = &self.divs[i];
                    row.set_selected(self.div == Some(i));
                    row.col(|ui| {
                        ui.label(d.boards.len().to_string());
                    });
                    row.col(|ui| {
                        ui.monospace(if d.auction.is_empty() {
                            "(opening)"
                        } else {
                            &d.auction
                        });
                    });
                    row.col(|ui| {
                        ui.monospace(&d.reference);
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(&d.ours).monospace().color(BAD));
                    });
                    row.col(|ui| {
                        if let Some(l) = &self.loaded {
                            let mut names: Vec<&str> = d
                                .boards
                                .iter()
                                .map(|&b| l.report.boards[b].scenario.as_str())
                                .collect();
                            names.dedup();
                            let n = names.len();
                            names.truncate(3);
                            let more = if n > 3 {
                                format!(" +{}", n - 3)
                            } else {
                                String::new()
                            };
                            ui.label(RichText::new(format!("{}{more}", names.join(", "))).weak());
                        }
                    });
                    if row.response().clicked() {
                        clicked = Some(i);
                    }
                });
            });
        if let Some(i) = clicked {
            self.div = Some(i);
            self.tab = Tab::Boards;
            if let Some(&first) = self.divs[i].boards.first() {
                self.select_board(first);
            }
        }
    }

    fn board_table(&mut self, ui: &mut egui::Ui, list: &[usize], salt: &str) {
        let Some(l) = &self.loaded else { return };
        let mut clicked = None;
        TableBuilder::new(ui)
            .id_salt(salt)
            .striped(true)
            .sense(Sense::click())
            .max_scroll_height(if salt == "boards" {
                f32::INFINITY
            } else {
                220.0
            })
            .column(Column::initial(170.0).clip(true))
            .column(Column::auto().at_least(40.0))
            .column(Column::initial(250.0).clip(true))
            .column(Column::initial(250.0).clip(true))
            .columns(Column::auto().at_least(70.0), 2)
            .column(Column::remainder())
            .header(20.0, |mut h| {
                for t in [
                    "scenario",
                    "board",
                    "BBA",
                    "ours",
                    "BBA contract",
                    "our contract",
                    "vs par",
                ] {
                    h.col(|ui| {
                        ui.strong(t);
                    });
                }
            })
            .body(|body| {
                body.rows(18.0, list.len(), |mut row| {
                    let i = list[row.index()];
                    let b = &l.report.boards[i];
                    row.set_selected(self.board == Some(i));
                    let calls = |c: &[bridge_types::Call]| {
                        c.iter().map(short).collect::<Vec<_>>().join(" ")
                    };
                    row.col(|ui| {
                        ui.label(&b.scenario);
                    });
                    row.col(|ui| {
                        ui.label(&b.board);
                    });
                    row.col(|ui| {
                        ui.monospace(calls(&b.reference));
                    });
                    row.col(|ui| {
                        let color = if b.first_divergence.is_none() {
                            GOOD
                        } else {
                            BAD
                        };
                        ui.label(RichText::new(calls(&b.ours)).monospace().color(color));
                    });
                    row.col(|ui| {
                        ui.monospace(b.reference_contract.as_deref().unwrap_or("passed out"));
                    });
                    row.col(|ui| {
                        let c = b.our_contract.as_deref().unwrap_or("passed out");
                        let color = if b.contracts_match() { GOOD } else { BAD };
                        ui.label(RichText::new(c).monospace().color(color));
                    });
                    row.col(|ui| {
                        if let Some(p) = &b.par {
                            let ours = (p.ours_ns - p.par_ns).abs();
                            let theirs = (p.reference_ns - p.par_ns).abs();
                            let (text, color) = match ours.cmp(&theirs) {
                                std::cmp::Ordering::Less => ("ours closer", GOOD),
                                std::cmp::Ordering::Greater => ("BBA closer", BAD),
                                std::cmp::Ordering::Equal => ("equal", Color32::GRAY),
                            };
                            ui.label(RichText::new(text).color(color));
                        }
                    });
                    if row.response().clicked() {
                        clicked = Some(i);
                    }
                });
            });
        if let Some(i) = clicked {
            self.select_board(i);
        }
    }
}
