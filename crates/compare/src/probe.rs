//! Probes: make a small reference set on purpose, to isolate one decision.
//!
//! Deals come from fixed holdings (optionally varied, e.g. the same hand
//! with 0-4 tens), from a dealer3 script, or at random. bba-cli bids them
//! with the auction forced up to a prefix and chosen cards (a named card, a
//! file, or a bare system, with toggles edited), and our engine replays the
//! result. Everything is written to an output directory so a probe can be
//! inspected or re-run.

use std::path::{Path, PathBuf};
use std::process::Command;

use bridge_card::{bbsa, Card, Value};
use bridge_types::{
    Board, Call, Card as PlayingCard, Deal, Direction, Hand, Rank, ScoringMethod, Suit,
    Vulnerability,
};
use rbb_engine::Engine;
use serde::Serialize;

use crate::board::{self, BoardResult};

/// Where bba-cli is normally installed.
pub const DEFAULT_BBA_CLI: &str = "/Applications/Bridge Utilities/bba-cli";

/// Where the deals come from.
#[derive(Debug, Clone)]
pub enum Deals {
    /// These holdings for these seats; the other seats are dealt at random.
    Fixed {
        hands: Vec<(Direction, Hand)>,
        vary_tens: bool,
        layouts: usize,
    },
    /// A dealer3 script, producing `count` deals.
    Script {
        script: PathBuf,
        count: usize,
        dealer_bin: PathBuf,
    },
    /// `count` random deals.
    Random { count: usize },
}

#[derive(Debug, Clone)]
pub struct ProbeOptions {
    pub deals: Deals,
    pub dealer: Direction,
    pub vul: Vulnerability,
    pub scoring: ScoringMethod,
    /// Calls forced before the decision under study.
    pub prefix: Vec<Call>,
    /// A card name in `pbs/bbsa`, a path to a `.bbsa`, or `bare:<system>`
    /// (2/1, sayc, polish, precision, acol).
    pub ns_card: String,
    pub ew_card: String,
    /// `.bbsa` key edits, e.g. ("Texas", 0), for each side.
    pub ns_set: Vec<(String, i64)>,
    pub ew_set: Vec<(String, i64)>,
    pub pbs: PathBuf,
    pub rules: PathBuf,
    pub bba_cli: PathBuf,
    pub out_dir: PathBuf,
    pub seed: u64,
}

/// One board of a probe, from the decision after the prefix on.
#[derive(Debug, Clone, Serialize)]
pub struct ProbeRow {
    /// The seat whose decision is probed (first to call after the prefix).
    pub seat: Direction,
    pub hand: String,
    pub hcp: i32,
    pub tens: i32,
    /// Notrump points in quarters, as our engine counts them.
    pub points_q: i32,
    /// Suit points in quarters.
    pub suit_points_q: i32,
    pub reference: Option<Call>,
    pub ours: Option<Call>,
    pub board: BoardResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeReport {
    pub rows: Vec<ProbeRow>,
    pub out_dir: PathBuf,
}

impl ProbeReport {
    /// Boards where both sides made the probed decision, and agreed.
    pub fn agreement(&self) -> (usize, usize) {
        let decided: Vec<&ProbeRow> = self.rows.iter().filter(|r| r.reference.is_some()).collect();
        (
            decided.iter().filter(|r| r.reference == r.ours).count(),
            decided.len(),
        )
    }
}

const SYSTEMS: [(&str, &str); 5] = [
    ("2/1", "two_over_one"),
    ("sayc", "sayc"),
    ("polish", "polish_club"),
    ("precision", "precision"),
    ("acol", "acol"),
];

/// `.bbsa` text for a card spec, with edits applied.
pub fn card_text(spec: &str, pbs: &Path, edits: &[(String, i64)]) -> Result<String, String> {
    let text = if let Some(system) = spec.strip_prefix("bare:") {
        let category = SYSTEMS
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(system))
            .map(|(_, v)| *v)
            .ok_or_else(|| {
                let names: Vec<&str> = SYSTEMS.iter().map(|(k, _)| *k).collect();
                format!("bare:{system}: expected one of {}", names.join(", "))
            })?;
        // Every toggle off; only the system type set.
        let mut card = Card::new();
        card.set("general.system_category", Value::Text(category.into()))
            .map_err(|e| e.to_string())?;
        bbsa::export(&card).0
    } else {
        let path = if spec.ends_with(".bbsa") {
            PathBuf::from(spec)
        } else {
            pbs.join("bbsa").join(format!("{spec}.bbsa"))
        };
        std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?
    };
    let mut lines: Vec<String> = text
        .lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect();
    for (key, value) in edits {
        let line = lines
            .iter_mut()
            .find(|l| l.rsplit_once('=').is_some_and(|(k, _)| k.trim() == key))
            .ok_or_else(|| {
                format!("no .bbsa key {key:?} (keys are as written in the file, e.g. \"Texas\")")
            })?;
        *line = format!("{key} = {value}");
    }
    Ok(lines.join("\r\n") + "\r\n")
}

/// Small deterministic generator for filling in the other hands.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// The same holding with 0, 1, ... more tens: each step replaces the lowest
/// spot card (2-9) of the next suit, spades first, by that suit's ten. HCP
/// and shape stay the same.
pub fn ten_variants(hand: &Hand) -> Vec<Hand> {
    let mut out = vec![hand.clone()];
    let mut cur: Vec<PlayingCard> = hand.cards().to_vec();
    for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
        if cur.iter().any(|c| c.suit == suit && c.rank == Rank::Ten) {
            continue;
        }
        let lowest = cur
            .iter()
            .enumerate()
            .filter(|(_, c)| c.suit == suit && (c.rank as u8) < 10)
            .min_by_key(|(_, c)| c.rank as u8)
            .map(|(i, _)| i);
        if let Some(i) = lowest {
            cur[i] = PlayingCard::new(suit, Rank::Ten);
            out.push(Hand::from_cards(cur.clone()));
        }
    }
    out
}

fn fill(fixed: &[(Direction, Hand)], rng: &mut Rng) -> Result<Deal, String> {
    let mut used = [false; 52];
    let mut deal = Deal::new();
    for (seat, hand) in fixed {
        if hand.len() != 13 {
            return Err(format!("{seat:?}: a hand needs 13 cards"));
        }
        for c in hand.cards() {
            let i = c.to_index() as usize;
            if used[i] {
                return Err(format!("{} is in two hands", c.to_index()));
            }
            used[i] = true;
        }
        deal.set_hand(*seat, hand.clone());
    }
    let mut rest: Vec<PlayingCard> = (0..52u8)
        .filter(|i| !used[*i as usize])
        .filter_map(PlayingCard::from_index)
        .collect();
    for i in (1..rest.len()).rev() {
        let j = (rng.next() % (i as u64 + 1)) as usize;
        rest.swap(i, j);
    }
    let mut it = rest.into_iter();
    for seat in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        if !fixed.iter().any(|(s, _)| *s == seat) {
            deal.set_hand(seat, Hand::from_cards(it.by_ref().take(13).collect()));
        }
    }
    Ok(deal)
}

fn make_deals(opts: &ProbeOptions) -> Result<Vec<Deal>, String> {
    let mut rng = Rng(opts.seed | 1);
    match &opts.deals {
        Deals::Fixed {
            hands,
            vary_tens,
            layouts,
        } => {
            // Vary the first holding; keep any others as given.
            let (first, others) = hands.split_first().ok_or("no --hand given")?;
            let variants = if *vary_tens {
                ten_variants(&first.1)
            } else {
                vec![first.1.clone()]
            };
            let mut out = Vec::new();
            for v in variants {
                for _ in 0..(*layouts).max(1) {
                    let mut fixed = vec![(first.0, v.clone())];
                    fixed.extend(others.iter().cloned());
                    out.push(fill(&fixed, &mut rng)?);
                }
            }
            Ok(out)
        }
        Deals::Random { count } => (0..*count).map(|_| fill(&[], &mut rng)).collect(),
        Deals::Script {
            script,
            count,
            dealer_bin,
        } => {
            let output = Command::new(dealer_bin)
                .arg(script)
                .args([
                    "-p",
                    &count.to_string(),
                    "-s",
                    &opts.seed.to_string(),
                    "-f",
                    "printpbn",
                ])
                .output()
                .map_err(|e| format!("{}: {e}", dealer_bin.display()))?;
            if !output.status.success() {
                return Err(format!(
                    "dealer3 failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            let text = String::from_utf8_lossy(&output.stdout);
            let boards = bridge_encodings::pbn::read_pbn(&text).map_err(|e| e.to_string())?;
            Ok(boards.into_iter().map(|b| b.deal).collect())
        }
    }
}

fn scoring_arg(s: ScoringMethod) -> &'static str {
    match s {
        ScoringMethod::Matchpoints | ScoringMethod::BAM => "MP",
        _ => "IMP",
    }
}

/// Make the reference with bba-cli and compare our engine with it.
pub fn run(opts: &ProbeOptions) -> Result<ProbeReport, String> {
    std::fs::create_dir_all(&opts.out_dir)
        .map_err(|e| format!("{}: {e}", opts.out_dir.display()))?;
    let ns_text = card_text(&opts.ns_card, &opts.pbs, &opts.ns_set)?;
    let ew_text = card_text(&opts.ew_card, &opts.pbs, &opts.ew_set)?;
    let (ns_path, ew_path) = (opts.out_dir.join("ns.bbsa"), opts.out_dir.join("ew.bbsa"));
    let write =
        |p: &Path, t: &str| std::fs::write(p, t).map_err(|e| format!("{}: {e}", p.display()));
    write(&ns_path, &ns_text)?;
    write(&ew_path, &ew_text)?;

    let deals = make_deals(opts)?;
    let boards: Vec<Board> = deals
        .into_iter()
        .enumerate()
        .map(|(i, d)| {
            Board::new()
                .with_number(i as u32 + 1)
                .with_dealer(opts.dealer)
                .with_vulnerability(opts.vul)
                .with_deal(d)
        })
        .collect();
    let input = opts.out_dir.join("deals.pbn");
    let output = opts.out_dir.join("bba.pbn");
    write(&input, &bridge_encodings::pbn::write_pbn(&boards))?;

    let mut cmd = Command::new(&opts.bba_cli);
    cmd.arg("-i").arg(&input).arg("-o").arg(&output);
    cmd.arg("--ns-conventions")
        .arg(&ns_path)
        .arg("--ew-conventions")
        .arg(&ew_path);
    cmd.args(["--event", "probe", "--scoring", scoring_arg(opts.scoring)]);
    if !opts.prefix.is_empty() {
        let prefix: Vec<String> = opts.prefix.iter().map(Call::to_pbn).collect();
        cmd.arg("--auction-prefix").arg(prefix.join(" "));
    }
    let out = cmd
        .output()
        .map_err(|e| format!("{}: {e}", opts.bba_cli.display()))?;
    if !out.status.success() {
        return Err(format!(
            "bba-cli failed: {}{}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        ));
    }

    let (ns_card, _) = bbsa::import(&ns_text, Some("ns")).map_err(|e| e.to_string())?;
    let (ew_card, _) = bbsa::import(&ew_text, Some("ew")).map_err(|e| e.to_string())?;
    let modules = rbb_engine::load_modules(&opts.rules).map_err(|d| {
        d.iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let engine = Engine::new(&ns_card, &ew_card, &modules);
    let valuation = rbb_engine::Valuation::default();

    let bba_boards = bridge_encodings::pbn::read_pbn_file(&output).map_err(|e| e.to_string())?;
    let k = opts.prefix.len();
    let mut rows = Vec::new();
    for mut b in bba_boards {
        b.extra_tags.retain(|(n, _)| n != "Scoring");
        b.extra_tags
            .push(("Scoring".into(), scoring_arg(opts.scoring).into()));
        let Some(mut r) = board::compare(&engine, "probe", &b) else {
            continue;
        };
        r.ns_card = opts.ns_card.clone();
        r.ew_card = opts.ew_card.clone();
        let seat = (0..k).fold(r.dealer, |d, _| d.next());
        let hand = b.deal.hand(seat);
        let facts = rbb_engine::Facts::new(hand);
        rows.push(ProbeRow {
            seat,
            hand: hand.to_pbn(),
            hcp: facts.hcp,
            tens: facts.tens,
            points_q: facts.points_q(valuation),
            suit_points_q: facts.suit_points_q(valuation),
            reference: r.reference.get(k).cloned(),
            ours: r.replay.get(k).cloned(),
            board: r,
        });
    }
    let report = ProbeReport {
        rows,
        out_dir: opts.out_dir.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        let _ = std::fs::write(opts.out_dir.join("report.json"), json);
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_variants_keep_hcp_and_shape() {
        let h = Hand::from_pbn("AK52.KQ73.A95.J8").unwrap();
        let v = ten_variants(&h);
        assert_eq!(v.len(), 5); // 0..=4 tens
        for (i, x) in v.iter().enumerate() {
            assert_eq!(x.hcp(), h.hcp());
            assert_eq!(x.distribution(), h.distribution());
            assert_eq!(rbb_engine::Facts::new(x).tens, i as i32);
        }
        assert_eq!(v[4].to_pbn(), "AKT5.KQT7.AT9.JT");
    }

    #[test]
    fn bare_card_turns_everything_off() {
        let text = card_text("bare:2/1", Path::new("."), &[("Texas".into(), 1)]).unwrap();
        let entries = bbsa::parse(&text).unwrap();
        assert!(entries.iter().any(|(k, v)| k == "Texas" && *v == 1));
        assert!(entries.iter().any(|(k, v)| k == "System type" && *v == 0));
        let on: Vec<_> = entries
            .iter()
            .filter(|(k, v)| *v != 0 && k != "Texas")
            .collect();
        assert!(on.is_empty(), "{on:?}");
    }
}
