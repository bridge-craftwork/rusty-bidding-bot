//! The knowledge view: a board zoomed in, one row per call. The call in
//! its caller's column, what it means, its flags (forcing, game forcing,
//! invitational, alerted, artificial), then what is known about every seat
//! after it (the parts the call changed highlighted, everything on hover),
//! each hand's own view of itself (HCP, points, and with a fit the count by
//! role: declarer or support points), and each side's state.
//!
//! Computed on demand from the engine (`Engine::review`) when a board is
//! selected: the comparison's results do not carry it.

use bridge_types::Direction;
use egui::text::LayoutJob;
use egui::{Color32, RichText, TextFormat};
use rbb_compare::{short, BoardResult};
use rbb_engine::{knowledge_full, knowledge_parts, side_text, ReviewRow};

use crate::app::BAD;
use crate::detail::{
    auction_grid, compass, dd_grid, short_path, strain_and_declarer, Detail, ALERT, BBA,
};

/// What changed on a row.
const CHANGED: Color32 = Color32::from_rgb(235, 150, 30);
const FORCING: Color32 = Color32::from_rgb(220, 120, 40);
const GAME_FORCE: Color32 = Color32::from_rgb(210, 60, 60);
const INVITE: Color32 = Color32::from_rgb(70, 140, 220);

/// Columns in bridge order.
const ORDER: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

/// Which auction and which column groups the view shows (kept across
/// boards).
#[derive(Debug, Clone)]
pub struct KnowledgeView {
    pub open: bool,
    /// BBA's auction (as the engine reads it) instead of ours.
    pub bba: bool,
    pub meaning: bool,
    pub flags: bool,
    pub known: bool,
    /// In the known columns, only what each call changed.
    pub changes_only: bool,
    pub own: bool,
    pub sides: bool,
}

impl Default for KnowledgeView {
    fn default() -> Self {
        KnowledgeView {
            open: false,
            bba: false,
            meaning: true,
            flags: true,
            known: true,
            changes_only: false,
            own: true,
            sides: true,
        }
    }
}

/// `text` cut to `n` characters, with an ellipsis.
fn cut(text: &str, n: usize) -> String {
    if text.chars().count() <= n {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(n - 1).collect::<String>())
    }
}

/// The row's knowledge of seat `i` as coloured parts.
fn known_job(ui: &egui::Ui, r: &ReviewRow, i: usize, changes_only: bool) -> LayoutJob {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let normal = ui.visuals().text_color();
    let weak = ui.visuals().weak_text_color();
    let mut job = LayoutJob::default();
    let parts: Vec<_> = knowledge_parts(&r.seats[i], &r.narrowed[i])
        .into_iter()
        .filter(|p| !changes_only || p.changed)
        .collect();
    for (n, p) in parts.iter().enumerate() {
        if n > 0 {
            job.append(" · ", 0.0, TextFormat::simple(font.clone(), weak));
        }
        let color = if p.changed { CHANGED } else { normal };
        job.append(&p.text, 0.0, TextFormat::simple(font.clone(), color));
    }
    job
}

/// The call's flags as coloured tags; the alert text and warnings on hover.
fn flags_cell(ui: &mut egui::Ui, r: &ReviewRow) {
    let f = &r.flags;
    let resp = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if f.game_force {
                ui.label(RichText::new("GF").strong().color(GAME_FORCE));
            } else if f.forcing {
                ui.label(RichText::new("F1").strong().color(FORCING));
            } else if f.in_game_force {
                ui.label(RichText::new("(GF)").weak());
            }
            if f.invitational {
                ui.label(RichText::new("inv").strong().color(INVITE));
            }
            if let Some(a) = &f.ask {
                ui.label(RichText::new(format!("ask {a}")).weak());
            }
            if let Some(t) = f.agrees {
                ui.label(format!("agrees {}", rbb_engine::strain_symbol(t)));
            }
            if f.alert.is_some() {
                ui.label(RichText::new("alert").color(ALERT));
            }
            if f.announce.is_some() {
                ui.label(RichText::new("ann").color(ALERT));
            }
            if f.artificial {
                ui.label(RichText::new("art").italics());
            }
            if !r.step.warnings.is_empty() {
                ui.label(RichText::new("⚠").color(BAD));
            }
        })
        .response;
    let mut hover = Vec::new();
    if let Some(a) = f.alert.as_ref().filter(|a| !a.is_empty()) {
        hover.push(format!("alert: {a}"));
    }
    if let Some(a) = &f.announce {
        hover.push(format!("announced: {a}"));
    }
    hover.extend(r.step.warnings.iter().map(|w| format!("warning: {w}")));
    if !hover.is_empty() {
        resp.on_hover_text(hover.join("\n"));
    }
}

/// What the user asked for in the view.
#[derive(Debug, Default)]
pub struct Action {
    /// A rule location whose link was clicked.
    pub open: Option<(String, usize)>,
    /// "Report…" was clicked.
    pub report: bool,
}

/// The window's title for board `b`.
pub fn title(b: &BoardResult) -> String {
    format!("Knowledge — {} board {}", b.scenario, b.board)
}

/// The header: the hands in the compass layout (dealer and vulnerability in
/// the corner), the auction shown, and the double-dummy table with par.
fn header(ui: &mut egui::Ui, kv: &KnowledgeView, b: &BoardResult, d: &Detail) {
    let rows = &d.review[usize::from(kv.bba)];
    ui.horizontal_top(|ui| {
        if let Some(deal) = &d.deal {
            let corner = format!("dealer {}\nvul {}", b.dealer.to_char(), b.vul.to_pbn());
            ui.vertical(|ui| compass(ui, deal, Some(&corner)));
            ui.add_space(24.0);
        }
        ui.vertical(|ui| {
            let (calls, name, color) = if kv.bba {
                (&b.reference, "BBA", BBA)
            } else {
                (&b.ours, "ours", BAD)
            };
            ui.strong(format!("{name}: {}", contract(kv, b)));
            // The engine's alerts and announcements, and BBA's own alerts
            // on its auction.
            let alerts: Vec<Option<String>> = (0..calls.len())
                .map(|i| {
                    let bba = kv.bba.then(|| b.reference_alerts.get(i).cloned().flatten());
                    let f = rows.get(i).map(|r| &r.flags);
                    bba.flatten()
                        .or_else(|| f.and_then(|f| f.alert.clone()))
                        .or_else(|| f.and_then(|f| f.announce.clone()))
                })
                .collect();
            let problems: &[rbb_compare::Problem] = if kv.bba { &[] } else { &b.problems };
            auction_grid(
                ui,
                "knowledge-auction",
                b.dealer,
                calls,
                b.first_divergence,
                color,
                problems,
                &alerts,
            );
        });
        ui.add_space(24.0);
        ui.vertical(|ui| match &b.dd {
            Some(dd) => {
                dd_grid(
                    ui,
                    dd,
                    b.reference_contract
                        .as_deref()
                        .and_then(strain_and_declarer),
                    b.our_contract.as_deref().and_then(strain_and_declarer),
                    b.contracts_match(),
                );
                if let Some(p) = &b.par {
                    ui.label(format!("par {:+} ({})", p.par_ns, p.par_contract));
                    ui.label(
                        RichText::new(format!(
                            "BBA {:+}   ours {:+}  (NS)",
                            p.reference_ns, p.ours_ns
                        ))
                        .weak(),
                    );
                }
            }
            None => {
                ui.label(RichText::new("double dummy: not solved yet").weak());
            }
        });
    });
}

/// The contract the shown auction reaches.
fn contract<'a>(kv: &KnowledgeView, b: &'a BoardResult) -> &'a str {
    let c = if kv.bba {
        &b.reference_contract
    } else {
        &b.our_contract
    };
    c.as_deref().unwrap_or("passed out")
}

/// Draw the view for board `b`.
pub fn ui(ui: &mut egui::Ui, kv: &mut KnowledgeView, b: &BoardResult, d: &Detail) -> Action {
    let mut act = Action::default();
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Report…")
            .on_hover_text(
                "File a ticket on this board: your note, everything the board detail \
                 shows, and this view's review of the auction shown (as `rbb \
                 explain-auction` prints it).",
            )
            .clicked()
        {
            act.report = true;
        }
        ui.separator();
        ui.strong(format!("{} — board {}", b.scenario, b.board));
        ui.label(format!("NS {}   EW {}", b.ns_card, b.ew_card));
    });
    if let Some(e) = &d.error {
        ui.label(RichText::new(e).color(BAD));
    }
    header(ui, kv, b, d);
    ui.separator();
    let open = &mut act.open;
    ui.horizontal_wrapped(|ui| {
        ui.label("auction:");
        ui.selectable_value(&mut kv.bba, false, "ours");
        ui.selectable_value(&mut kv.bba, true, "BBA's (as the engine reads it)");
        ui.separator();
        ui.label("show:");
        ui.checkbox(&mut kv.meaning, "meaning");
        ui.checkbox(&mut kv.flags, "flags");
        ui.checkbox(&mut kv.known, "what is known");
        if kv.known {
            ui.checkbox(&mut kv.changes_only, "only what each call changed");
        }
        ui.checkbox(&mut kv.own, "own view").on_hover_text(
            "Each hand's view of itself: HCP and total points; with a trump suit \
                 agreed (♥) or an eight-card fit this hand can see with what partner \
                 has shown (♥?), the count the slam conditions use: declarer points \
                 (HCP + length) for the hand with more trumps than partner has shown \
                 (decl), support points (HCP + shortness, capped by the trumps) for \
                 the other (sup), and support points for both with equal length (sup=).",
        );
        ui.checkbox(&mut kv.sides, "side state");
    });
    let rows = &d.review[usize::from(kv.bba)];
    let calls = if kv.bba { &b.reference } else { &b.ours };
    if rows.len() != calls.len() {
        ui.label(RichText::new("the review does not match the auction").color(BAD));
    }
    ui.label(
        RichText::new(
            "Highlighted: what the row's call changed. Hover a seat's knowledge for every \
             range and what it has shown and denied; hover a call for its meaning.",
        )
        .weak(),
    );
    ui.separator();

    let mut cols = 1 + 4;
    if kv.meaning {
        cols += 2;
    }
    if kv.flags {
        cols += 1;
    }
    if kv.known {
        cols += 4;
    }
    if kv.own {
        cols += 4;
    }
    if kv.sides {
        cols += 2;
    }
    egui::ScrollArea::both()
        .id_salt("knowledge-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("knowledge-grid")
                .striped(true)
                .num_columns(cols)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("#");
                    for s in ORDER {
                        ui.strong(s.to_char().to_string());
                    }
                    if kv.meaning {
                        ui.strong("meaning");
                        ui.strong("rule");
                    }
                    if kv.flags {
                        ui.strong("flags");
                    }
                    if kv.known {
                        for s in ORDER {
                            ui.strong(format!("{} known", s.to_char()));
                        }
                    }
                    if kv.own {
                        for s in ORDER {
                            ui.strong(format!("{} own", s.to_char()));
                        }
                    }
                    if kv.sides {
                        ui.strong("NS");
                        ui.strong("EW");
                    }
                    ui.end_row();

                    for (n, r) in rows.iter().enumerate() {
                        let prev = n.checked_sub(1).map(|p| &rows[p]);
                        let step = &r.step;
                        let diverged = !kv.bba && b.first_divergence == Some(r.index);
                        let num = RichText::new((r.index + 1).to_string());
                        if diverged {
                            ui.label(num.strong().color(BAD))
                                .on_hover_text("the first call where ours departs from BBA's");
                        } else {
                            ui.label(num);
                        }
                        let mut hover =
                            step.explanation.clone().unwrap_or_else(|| "no rule".into());
                        if let Some(a) = r.flags.alert.as_ref().filter(|a| !a.is_empty()) {
                            hover += &format!("\nalert: {a}");
                        }
                        if let Some(a) = &r.flags.announce {
                            hover += &format!("\nannounced: {a}");
                        }
                        for s in ORDER {
                            if s == step.caller {
                                let t = RichText::new(short(&step.call)).monospace().strong();
                                let t = if r.flags.alert.is_some() || r.flags.announce.is_some() {
                                    t.background_color(ALERT.gamma_multiply(0.35))
                                } else {
                                    t
                                };
                                ui.label(t).on_hover_text(&hover);
                            } else {
                                ui.label("");
                            }
                        }
                        if kv.meaning {
                            match &step.explanation {
                                Some(e) => ui.label(cut(e, 48)).on_hover_text(&hover),
                                None => ui.label(RichText::new("no rule").color(Color32::GRAY)),
                            };
                            match &step.rule {
                                Some(rule) => {
                                    if ui
                                        .link(format!("{}:{}", short_path(&rule.file), rule.line))
                                        .on_hover_text(&rule.module)
                                        .clicked()
                                    {
                                        *open = Some((rule.file.clone(), rule.line));
                                    }
                                }
                                None => {
                                    ui.label("");
                                }
                            }
                        }
                        if kv.flags {
                            flags_cell(ui, r);
                        }
                        if kv.known {
                            for s in ORDER {
                                let i = s.to_index();
                                let job = known_job(ui, r, i, kv.changes_only);
                                ui.label(job).on_hover_text(knowledge_full(&r.seats[i]));
                            }
                        }
                        if kv.own {
                            for s in ORDER {
                                let i = s.to_index();
                                match &r.own[i] {
                                    Some(o) => {
                                        let moved = prev
                                            .and_then(|p| p.own[i].as_ref())
                                            .is_some_and(|p| p.fit != o.fit);
                                        let t = RichText::new(o.text());
                                        let resp =
                                            ui.label(if moved { t.color(CHANGED) } else { t });
                                        if let Some(f) = &o.fit {
                                            resp.on_hover_text(format!(
                                                "{} trumps here, partner has shown {}+; {} {}: {}",
                                                f.mine,
                                                f.partner_shown,
                                                if f.agreed {
                                                    "agreed"
                                                } else {
                                                    "fit seen by this hand"
                                                },
                                                rbb_engine::strain_symbol(f.trump),
                                                match f.role {
                                                    rbb_engine::Role::Declarer =>
                                                        "declarer points (HCP + length)",
                                                    rbb_engine::Role::Support =>
                                                        "support points (HCP + shortness)",
                                                    rbb_engine::Role::Equal =>
                                                        "equal length: support points for both",
                                                }
                                            ));
                                        }
                                    }
                                    None => {
                                        ui.label("");
                                    }
                                }
                            }
                        }
                        if kv.sides {
                            for side in 0..2 {
                                let st = &r.sides[side];
                                let moved = prev
                                    .map_or(*st != Default::default(), |p| p.sides[side] != *st);
                                let t = RichText::new(side_text(st));
                                ui.label(if moved { t.color(CHANGED) } else { t });
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    act
}
