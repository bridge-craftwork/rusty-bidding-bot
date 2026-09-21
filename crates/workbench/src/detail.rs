//! Board detail: the deal, both auctions, how the engine read BBA's calls,
//! and the engine's reasoning at the first difference.

use bridge_types::{Call, Deal, Direction, Hand, Suit};
use egui::{Color32, RichText};
use rbb_compare::{short, BoardResult, Engines};
use rbb_engine::{Decision, Interpretation, SeatKnowledge, Tri};

use crate::app::{BAD, GOOD};

const SEATS: [Direction; 4] = [
    Direction::West,
    Direction::North,
    Direction::East,
    Direction::South,
];

pub struct Detail {
    deal: Option<Deal>,
    /// The engine's reading of BBA's auction.
    reading: Option<Interpretation>,
    /// The engine's decision at the first difference.
    decision: Option<Decision>,
    error: Option<String>,
}

fn caller(dealer: Direction, i: usize) -> Direction {
    (0..i).fold(dealer, |d, _| d.next())
}

impl Detail {
    pub fn compute(b: &BoardResult, engines: &Engines) -> Detail {
        let deal = Deal::from_pbn(&b.deal);
        let engine = match engines.get(&b.ns_card, &b.ew_card) {
            Ok(e) => e,
            Err(e) => {
                return Detail {
                    deal,
                    reading: None,
                    decision: None,
                    error: Some(e),
                }
            }
        };
        let reading = Some(engine.interpret(b.dealer, b.vul, &b.reference));
        let decision = match (b.first_divergence, &deal) {
            (Some(d), Some(deal)) => Some(engine.bid(
                deal.hand(caller(b.dealer, d)),
                b.dealer,
                b.vul,
                &b.reference[..d],
            )),
            _ => None,
        };
        Detail {
            deal,
            reading,
            decision,
            error: None,
        }
    }

    /// Draw the detail. Returns a rule location when its link is clicked.
    pub fn ui(&self, ui: &mut egui::Ui, b: &BoardResult) -> Option<(String, usize)> {
        let mut open = None;
        ui.horizontal_wrapped(|ui| {
            ui.heading(format!("{} — board {}", b.scenario, b.board));
            ui.label(format!(
                "dealer {}   vul {}   NS card {}   EW card {}",
                b.dealer.to_char(),
                b.vul.to_pbn(),
                b.ns_card,
                b.ew_card
            ));
        });
        if let Some(e) = &self.error {
            ui.label(RichText::new(e).color(BAD));
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "BBA: {}",
                b.reference_contract.as_deref().unwrap_or("passed out")
            ));
            let color = if b.contracts_match() { GOOD } else { BAD };
            ui.label(
                RichText::new(format!(
                    "ours: {}",
                    b.our_contract.as_deref().unwrap_or("passed out")
                ))
                .color(color),
            );
            if let Some(p) = &b.par {
                ui.label(format!(
                    "par {:+} ({})   BBA {:+}   ours {:+}  (NS scores, double dummy)",
                    p.par_ns, p.par_contract, p.reference_ns, p.ours_ns
                ));
            }
        });
        ui.separator();

        ui.horizontal_top(|ui| {
            if let Some(deal) = &self.deal {
                compass(ui, deal);
            }
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.strong("BBA");
                auction_grid(
                    ui,
                    "bba-auction",
                    b.dealer,
                    &b.reference,
                    b.first_divergence,
                    Color32::from_rgb(90, 140, 220),
                );
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.strong("ours");
                auction_grid(
                    ui,
                    "our-auction",
                    b.dealer,
                    &b.ours,
                    b.first_divergence,
                    BAD,
                );
            });
        });
        ui.separator();

        if let (Some(d), Some(dec)) = (b.first_divergence, &self.decision) {
            let seat = caller(b.dealer, d);
            let hand = self
                .deal
                .as_ref()
                .map(|x| hand_line(x.hand(seat)))
                .unwrap_or_default();
            ui.label(
                RichText::new(format!(
                    "First difference at call {}: {} holds {}. BBA bid {}, the engine chose {} — {}",
                    d + 1,
                    seat.to_char(),
                    hand,
                    short(&b.reference[d]),
                    short(&dec.call),
                    dec.explanation
                ))
                .strong(),
            );
            if let Some(a) = b.reference_alerts.get(d).cloned().flatten() {
                ui.label(format!("BBA's alert for {}: {a}", short(&b.reference[d])));
            }
            ui.label(RichText::new("Candidates, best-ranked first:").weak());
            egui::Grid::new("candidates")
                .striped(true)
                .num_columns(6)
                .show(ui, |ui| {
                    for h in ["call", "prio", "descr", "outcome", "meaning", "rule"] {
                        ui.strong(h);
                    }
                    ui.end_row();
                    for c in &dec.candidates {
                        let color = match c.outcome.as_str() {
                            "chosen" => GOOD,
                            "outranked" => Color32::from_rgb(200, 160, 60),
                            _ => Color32::GRAY,
                        };
                        let is_bba = c.call == b.reference[d];
                        let call = RichText::new(short(&c.call)).monospace().strong();
                        ui.label(if is_bba { call.underline() } else { call });
                        ui.label(c.priority.to_string());
                        ui.label(format!("{:.3}", c.descriptiveness));
                        ui.label(RichText::new(&c.outcome).color(color));
                        ui.label(&c.explanation);
                        if ui
                            .link(format!("{}:{}", short_path(&c.rule.file), c.rule.line))
                            .clicked()
                        {
                            open = Some((c.rule.file.clone(), c.rule.line));
                        }
                        ui.end_row();
                    }
                });
            if !dec.candidates.iter().any(|c| c.call == b.reference[d]) {
                ui.label(
                    RichText::new(format!(
                        "No rule offers BBA's {} here.",
                        short(&b.reference[d])
                    ))
                    .color(BAD),
                );
            }
            for w in &dec.warnings {
                ui.label(RichText::new(format!("warning: {w}")).color(BAD));
            }

            ui.add_space(6.0);
            ui.label(RichText::new("What each seat had shown at that point:").weak());
            let pos = &dec.auction.position;
            egui::Grid::new("knowledge")
                .striped(true)
                .num_columns(2)
                .show(ui, |ui| {
                    for s in SEATS {
                        ui.monospace(s.to_char().to_string());
                        knowledge(ui, pos.knowledge(s));
                        ui.end_row();
                    }
                });
            for (side, name) in [(0usize, "NS"), (1, "EW")] {
                let st = &pos.sides[side];
                let mut parts = vec![];
                if let Some(t) = st.trump {
                    parts.push(format!("trump {t}"));
                }
                parts.push(format!("forcing {:?}", st.forcing).to_lowercase());
                if let Some(a) = &st.ask {
                    parts.push(format!("asked {} by {}", a.kind, a.by.to_char()));
                }
                if let Some(a) = &st.answered {
                    parts.push(format!("answered {} (asked by {})", a.kind, a.by.to_char()));
                }
                ui.label(format!("{name}: {}", parts.join(", ")));
            }
            ui.separator();
        }

        if let Some(reading) = &self.reading {
            egui::CollapsingHeader::new("How the engine read BBA's auction")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("reading")
                        .striped(true)
                        .num_columns(6)
                        .show(ui, |ui| {
                            for h in [
                                "#",
                                "seat",
                                "BBA",
                                "BBA alert",
                                "engine's reading",
                                "engine would bid",
                            ] {
                                ui.strong(h);
                            }
                            ui.end_row();
                            for (i, step) in reading.steps.iter().enumerate() {
                                ui.label((i + 1).to_string());
                                ui.monospace(step.caller.to_char().to_string());
                                ui.monospace(short(&step.call));
                                ui.label(
                                    b.reference_alerts
                                        .get(i)
                                        .cloned()
                                        .flatten()
                                        .unwrap_or_default(),
                                );
                                ui.horizontal(|ui| {
                                    match &step.explanation {
                                        Some(e) => {
                                            ui.label(e);
                                        }
                                        None => {
                                            ui.label(RichText::new("no rule").color(Color32::GRAY));
                                        }
                                    }
                                    if let Some(r) = &step.rule {
                                        if ui
                                            .link(format!("{}:{}", short_path(&r.file), r.line))
                                            .clicked()
                                        {
                                            open = Some((r.file.clone(), r.line));
                                        }
                                    }
                                    knowledge(ui, &step.knowledge);
                                });
                                let ours = &b.replay[i];
                                let color = if ours == &step.call { GOOD } else { BAD };
                                ui.label(RichText::new(short(ours)).monospace().color(color));
                                ui.end_row();
                            }
                        });
                });
        }
        open
    }
}

fn short_path(p: &str) -> &str {
    p.find("conventions/")
        .map_or(p, |i| &p[i + "conventions/".len()..])
}

fn suit_cards(hand: &Hand, suit: Suit) -> String {
    let mut cards = hand.cards_in_suit(suit);
    cards.sort_by_key(|c| std::cmp::Reverse(c.rank));
    let s: String = cards.iter().map(|c| c.rank.to_char()).collect();
    if s.is_empty() {
        "—".into()
    } else {
        s
    }
}

fn hand_line(hand: &Hand) -> String {
    [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
        .iter()
        .map(|s| format!("{}{}", s.symbol(), suit_cards(hand, *s)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn hand_block(ui: &mut egui::Ui, seat: Direction, hand: &Hand) {
    ui.vertical(|ui| {
        ui.label(RichText::new(format!("{}  {} HCP", seat.to_char(), hand.hcp())).strong());
        for s in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
            let color = if s.is_red() {
                Color32::from_rgb(200, 60, 60)
            } else {
                ui.visuals().text_color()
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(s.symbol().to_string()).color(color));
                ui.monospace(suit_cards(hand, s));
            });
        }
    });
}

fn compass(ui: &mut egui::Ui, deal: &Deal) {
    egui::Grid::new("compass")
        .spacing([18.0, 6.0])
        .show(ui, |ui| {
            ui.label("");
            hand_block(ui, Direction::North, deal.hand(Direction::North));
            ui.label("");
            ui.end_row();
            hand_block(ui, Direction::West, deal.hand(Direction::West));
            ui.label("");
            hand_block(ui, Direction::East, deal.hand(Direction::East));
            ui.end_row();
            ui.label("");
            hand_block(ui, Direction::South, deal.hand(Direction::South));
            ui.label("");
            ui.end_row();
        });
}

fn auction_grid(
    ui: &mut egui::Ui,
    id: &str,
    dealer: Direction,
    calls: &[Call],
    mark: Option<usize>,
    color: Color32,
) {
    egui::Grid::new(id).spacing([14.0, 2.0]).show(ui, |ui| {
        for s in SEATS {
            ui.strong(s.to_char().to_string());
        }
        ui.end_row();
        let offset = SEATS.iter().position(|s| *s == dealer).unwrap_or(0);
        for _ in 0..offset {
            ui.label("");
        }
        for (i, c) in calls.iter().enumerate() {
            let text = RichText::new(short(c)).monospace();
            if Some(i) == mark {
                ui.label(text.strong().color(Color32::WHITE).background_color(color));
            } else {
                ui.label(text);
            }
            if (offset + i + 1) % 4 == 0 {
                ui.end_row();
            }
        }
    });
}

/// One line of ranges; the constraints behind them on hover.
fn knowledge(ui: &mut egui::Ui, k: &SeatKnowledge) {
    let mut s = format!(
        "{} HCP  ♠{} ♥{} ♦{} ♣{}",
        k.hcp, k.len[3], k.len[2], k.len[1], k.len[0]
    );
    if k.balanced == Tri::True {
        s += "  balanced";
    } else if k.balanced == Tri::False {
        s += "  unbalanced";
    }
    let r = ui.label(RichText::new(s).monospace().weak());
    if !k.shown.is_empty() {
        r.on_hover_text(k.shown.join("\n"));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    /// Compute and draw the detail of every fixture board headlessly: no
    /// panics, and a trace wherever the engine departs from BBA.
    #[test]
    fn detail_renders_for_every_fixture_board() {
        let here = Path::new(env!("CARGO_MANIFEST_DIR"));
        let opts = rbb_compare::Options {
            pbs: here.join("../compare/tests/fixtures/pbs"),
            scenarios: vec![],
            limit: None,
            rules: here.join("../../conventions"),
            par: false,
            dd_cache: std::env::temp_dir().join("rbb-workbench-test-dd.jsonl"),
        };
        let engines = Engines::new(&opts.pbs, &opts.rules).unwrap();
        let report = rbb_compare::run_with(&opts, &engines, &|_, _| {}).unwrap();
        assert!(!report.boards.is_empty());
        let ctx = egui::Context::default();
        for b in &report.boards {
            let d = Detail::compute(b, &engines);
            assert!(d.error.is_none(), "{:?}", d.error);
            assert!(d.deal.is_some());
            assert_eq!(d.decision.is_some(), b.first_divergence.is_some());
            if let (Some(i), Some(dec)) = (b.first_divergence, &d.decision) {
                // The trace reproduces the engine's replayed call.
                assert_eq!(dec.call, b.replay[i]);
            }
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                d.ui(ui, b);
            });
            // No renderer here: drop the font-texture updates.
            out.textures_delta.clear();
        }
    }
}
