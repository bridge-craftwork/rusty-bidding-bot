//! Grids: probe one decision over a list of hands (or a survey or morph of
//! one hand), at several vulnerabilities and scorings, from a small spec
//! file, and lay BBA's calls out as a table: one row per hand, one column
//! per condition.
//!
//! The deciding hand is the first to call after the prefix. Partner's hand
//! is the first of the spec's `partner` hands that shares no card with the
//! variant (BBA's decision does not depend on partner's cards: the same
//! responder bids the same over five different 1NT openers); the opponents
//! get the rest.

use std::path::{Path, PathBuf};

use bridge_types::{Call, Deal, Direction, Hand, ScoringMethod, Vulnerability};
use serde::{Deserialize, Serialize};

use crate::probe::{self, Deals, ProbeOptions, Rng};

/// A grid spec, read from TOML.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    /// Card for both sides (a name in PBS bbsa/, a .bbsa path, or bare:2/1).
    pub card: String,
    /// East-West's card, when it differs.
    pub ew_card: Option<String>,
    pub dealer: String,
    /// Calls forced before the decision, e.g. "1NT Pass".
    #[serde(default)]
    pub prefix: String,
    /// Vulnerabilities (None, NS, EW, All); default None.
    #[serde(default)]
    pub vuls: Vec<String>,
    /// Scorings (MP, IMP); default MP.
    #[serde(default)]
    pub scoring: Vec<String>,
    /// Partner hands, tried in order: the first that shares no card with
    /// the variant is used.
    #[serde(default)]
    pub partner: Vec<String>,
    /// When no listed partner fits, deal one: balanced, HCP in this range
    /// (e.g. [15, 17] for a 1NT opener), from the cards left.
    pub partner_auto: Option<[u32; 2]>,
    /// Card changes for our engine only, e.g. "general.style=bba".
    #[serde(default)]
    pub our: Vec<String>,
    /// `.bbsa` key edits for both sides' cards, "Key=value".
    #[serde(default)]
    pub set: Vec<String>,
    /// "label | S.H.D.C", or just "S.H.D.C"; "label | hand | partner"
    /// gives that hand its own partner.
    #[serde(default)]
    pub hands: Vec<String>,
    /// Every single-card exchange between this hand and its opponents.
    pub survey: Option<String>,
    /// From the first hand to the second a card at a time, and each
    /// exchange alone.
    pub morph: Option<[String; 2]>,
    #[serde(default = "one")]
    pub seed: u64,
}

fn one() -> u64 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Mode {
    List,
    Survey,
    Morph,
}

/// One condition: a vulnerability and a scoring.
#[derive(Debug, Clone, Serialize)]
pub struct Column {
    pub vul: Vulnerability,
    pub scoring: ScoringMethod,
}

impl Column {
    pub fn name(&self) -> String {
        let s = if matches!(self.scoring, ScoringMethod::Matchpoints) {
            "MP"
        } else {
            "IMP"
        };
        format!("{}/{}", self.vul.to_pbn(), s)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Cell {
    pub bba: Option<Call>,
    pub ours: Option<Call>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Row {
    pub label: String,
    pub hand: String,
    pub hcp: i32,
    pub tens: i32,
    /// Suit lengths, spades first.
    pub shape: [usize; 4],
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Grid {
    pub mode: Mode,
    pub seat: Direction,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
    pub out_dir: PathBuf,
}

/// Where the tools are.
#[derive(Debug, Clone)]
pub struct Env {
    pub pbs: PathBuf,
    pub rules: PathBuf,
    pub bba_cli: PathBuf,
    pub out_dir: PathBuf,
}

fn hand(s: &str, what: &str) -> Result<Hand, String> {
    Hand::from_pbn(s.trim())
        .filter(|h| h.len() == 13)
        .ok_or_else(|| format!("{what}: {s:?} is not a 13-card hand S.H.D.C"))
}

fn overlaps(a: &Hand, b: &Hand) -> bool {
    a.cards().iter().any(|c| b.cards().contains(c))
}

/// The deal for `variant` in `seat`: the first partner hand that fits, the
/// rest to the opponents.
fn deal_for(
    variant: &Hand,
    label: &str,
    seat: Direction,
    partners: &[Hand],
    auto: Option<[u32; 2]>,
    avoid: &[&Hand],
    rng: &mut Rng,
) -> Result<Deal, String> {
    let mut fixed = vec![(seat, variant.clone())];
    let clear = |p: &Hand| !overlaps(p, variant) && avoid.iter().all(|a| !overlaps(p, a));
    match (partners.iter().find(|p| clear(p)), auto) {
        (Some(p), _) => fixed.push((seat.partner(), p.clone())),
        (None, Some([lo, hi])) => fixed.push((seat.partner(), auto_partner(variant, avoid, lo, hi, rng)?)),
        (None, None) if partners.is_empty() => {}
        (None, None) => {
            return Err(format!(
                "{label} ({}): every partner hand shares a card with it; add one to `partner` or set `partner_auto`",
                variant.to_pbn()
            ))
        }
    }
    probe::fill(&fixed, rng)
}

/// A balanced hand with `lo..=hi` HCP from the cards not in `variant` or
/// `avoid`.
fn auto_partner(
    variant: &Hand,
    avoid: &[&Hand],
    lo: u32,
    hi: u32,
    rng: &mut Rng,
) -> Result<Hand, String> {
    use bridge_types::Card as PlayingCard;
    let used = |c: &PlayingCard| {
        variant.cards().contains(c) || avoid.iter().any(|a| a.cards().contains(c))
    };
    let rest: Vec<PlayingCard> = (0..52u8)
        .filter_map(PlayingCard::from_index)
        .filter(|c| !used(c))
        .collect();
    for _ in 0..200_000 {
        let mut pool = rest.clone();
        for i in (1..pool.len()).rev() {
            let j = (rng.next() % (i as u64 + 1)) as usize;
            pool.swap(i, j);
        }
        let h = Hand::from_cards(pool.into_iter().take(13).collect());
        let f = rbb_engine::Facts::new(&h);
        if (lo as i32..=hi as i32).contains(&f.hcp) && f.balanced {
            return Ok(h);
        }
    }
    Err(format!(
        "no balanced {lo}-{hi} partner found for {}",
        variant.to_pbn()
    ))
}

pub fn read_spec(path: &Path) -> Result<Spec, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn run(spec: &Spec, env: &Env) -> Result<Grid, String> {
    let dealer = spec
        .dealer
        .chars()
        .next()
        .and_then(|c| Direction::from_char(c.to_ascii_uppercase()))
        .ok_or("dealer must be N, E, S or W")?;
    let prefix: Vec<Call> = spec
        .prefix
        .split_whitespace()
        .map(|c| Call::from_pbn(c).ok_or_else(|| format!("bad call {c:?} in prefix")))
        .collect::<Result<_, _>>()?;
    let seat = (0..prefix.len()).fold(dealer, |d, _| d.next());
    let vuls: Vec<Vulnerability> = if spec.vuls.is_empty() {
        vec![Vulnerability::None]
    } else {
        spec.vuls
            .iter()
            .map(|v| Vulnerability::from_pbn(v).ok_or_else(|| format!("bad vulnerability {v:?}")))
            .collect::<Result<_, _>>()?
    };
    let scorings: Vec<ScoringMethod> = if spec.scoring.is_empty() {
        vec![ScoringMethod::Matchpoints]
    } else {
        spec.scoring
            .iter()
            .map(|s| ScoringMethod::from_pbn(s).ok_or_else(|| format!("bad scoring {s:?}")))
            .collect::<Result<_, _>>()?
    };
    let partners: Vec<Hand> = spec
        .partner
        .iter()
        .map(|p| hand(p, "partner"))
        .collect::<Result<_, _>>()?;
    let set: Vec<(String, i64)> = spec
        .set
        .iter()
        .map(|s| {
            let (k, v) = s.rsplit_once('=').ok_or("set takes \"Key=value\"")?;
            let v = v
                .trim()
                .parse()
                .map_err(|_| format!("{s:?}: value must be a number"))?;
            Ok((k.trim().to_string(), v))
        })
        .collect::<Result<_, String>>()?;

    let mut rng = Rng(spec.seed | 1);
    let (mode, deals) = match (&spec.survey, &spec.morph, spec.hands.is_empty()) {
        (Some(h), None, true) => {
            let base = deal_for(
                &hand(h, "survey")?,
                "survey",
                seat,
                &partners,
                spec.partner_auto,
                &[],
                &mut rng,
            )?;
            (Mode::Survey, probe::survey(&base, seat))
        }
        (None, Some([a, b]), true) => {
            // A partner clear of both ends of the morph.
            let target = hand(b, "morph target")?;
            let base = deal_for(
                &hand(a, "morph")?,
                "morph",
                seat,
                &partners,
                spec.partner_auto,
                &[&target],
                &mut rng,
            )?;
            let fixed: Vec<Direction> = if partners.is_empty() {
                vec![]
            } else {
                vec![seat.partner()]
            };
            (
                Mode::Morph,
                probe::morph(&base, seat, &hand(b, "morph target")?, &fixed)?,
            )
        }
        (None, None, false) => {
            let mut out = Vec::new();
            for line in &spec.hands {
                let parts: Vec<&str> = line.split('|').map(str::trim).collect();
                let (label, h, own) = match parts.as_slice() {
                    [h] => (h.to_string(), *h, None),
                    [l, h] => (l.to_string(), *h, None),
                    [l, h, p] => (l.to_string(), *h, Some(hand(p, "partner")?)),
                    _ => return Err(format!("{line:?}: expected label | hand [| partner]")),
                };
                let v = hand(h, &label)?;
                let ps = own.map_or_else(|| partners.clone(), |p| vec![p]);
                out.push((
                    label.clone(),
                    deal_for(&v, &label, seat, &ps, spec.partner_auto, &[], &mut rng)?,
                ));
            }
            (Mode::List, out)
        }
        _ => return Err("give exactly one of `hands`, `survey` or `morph`".into()),
    };

    let columns: Vec<Column> = scorings
        .iter()
        .flat_map(|s| {
            vuls.iter().map(|v| Column {
                vul: *v,
                scoring: *s,
            })
        })
        .collect();
    let mut rows: Vec<Row> = deals
        .iter()
        .map(|(label, d)| {
            let h = d.hand(seat);
            let f = rbb_engine::Facts::new(h);
            Row {
                label: label.clone(),
                hand: h.to_pbn(),
                hcp: f.hcp,
                tens: f.tens,
                shape: [
                    f.len[3] as usize,
                    f.len[2] as usize,
                    f.len[1] as usize,
                    f.len[0] as usize,
                ],
                cells: vec![],
            }
        })
        .collect();

    // One bba-cli run per scoring; every vulnerability in the same run.
    for (si, scoring) in scorings.iter().enumerate() {
        let opts = ProbeOptions {
            deals: Deals::Explicit(deals.clone()),
            dealer,
            vul: vuls[0],
            scoring: *scoring,
            prefix: prefix.clone(),
            ns_card: spec.card.clone(),
            ew_card: spec.ew_card.clone().unwrap_or_else(|| spec.card.clone()),
            ns_set: set.clone(),
            ew_set: set.clone(),
            pbs: env.pbs.clone(),
            rules: env.rules.clone(),
            bba_cli: env.bba_cli.clone(),
            out_dir: env.out_dir.join(format!("run{si}")),
            seed: spec.seed,
            our_changes: spec.our.clone(),
            vuls: vuls.clone(),
        };
        let report = probe::run(&opts)?;
        // Boards come back in order: deal by deal, vulnerabilities innermost.
        if report.rows.len() != deals.len() * vuls.len() {
            return Err(format!(
                "bba-cli returned {} boards for {} deals x {} vulnerabilities",
                report.rows.len(),
                deals.len(),
                vuls.len()
            ));
        }
        for (i, r) in report.rows.iter().enumerate() {
            rows[i / vuls.len()].cells.push(Cell {
                bba: r.reference.clone(),
                ours: r.ours.clone(),
            });
        }
    }
    // Cells were pushed scoring-major; columns are scoring-major too.
    let grid = Grid {
        mode,
        seat,
        columns,
        rows,
        out_dir: env.out_dir.clone(),
    };
    std::fs::create_dir_all(&env.out_dir).map_err(|e| e.to_string())?;
    if let Ok(json) = serde_json::to_string_pretty(&grid) {
        let _ = std::fs::write(env.out_dir.join("grid.json"), json);
    }
    let _ = std::fs::write(env.out_dir.join("grid.tsv"), grid.tsv());
    Ok(grid)
}

fn short(c: &Option<Call>) -> String {
    c.as_ref().map_or("-".into(), crate::short)
}

impl Grid {
    /// Tab-separated: label, hand, HCP, tens, then BBA and ours per column.
    pub fn tsv(&self) -> String {
        let mut out = String::from("label\thand\thcp\ttens");
        for c in &self.columns {
            out += &format!("\t{0} BBA\t{0} ours", c.name());
        }
        out.push('\n');
        for r in &self.rows {
            out += &format!("{}\t{}\t{}\t{}", r.label, r.hand, r.hcp, r.tens);
            for c in &r.cells {
                out += &format!("\t{}\t{}", short(&c.bba), short(&c.ours));
            }
            out.push('\n');
        }
        out
    }

    /// Rows whose BBA calls differ from the first row's in any column.
    pub fn changes_from_first(&self) -> Vec<&Row> {
        let Some(first) = self.rows.first() else {
            return vec![];
        };
        self.rows[1..]
            .iter()
            .filter(|r| {
                r.cells
                    .iter()
                    .zip(&first.cells)
                    .any(|(a, b)| a.bba != b.bba)
            })
            .collect()
    }

    /// A readable table. A cell shows BBA's call, and ours after "≠" when
    /// they differ. `rows` limits the rows shown (a survey shows its
    /// changes only).
    pub fn table(&self, rows: &[&Row]) -> String {
        let lw = rows
            .iter()
            .map(|r| r.label.chars().count())
            .max()
            .unwrap_or(5)
            .max(5);
        let cw = self
            .columns
            .iter()
            .map(|c| c.name().len())
            .max()
            .unwrap_or(6)
            .max(11);
        let mut out = format!("{:lw$}  {:18} {:>3} {:>2}", "", "hand", "HCP", "T");
        for c in &self.columns {
            out += &format!("  {:cw$}", c.name());
        }
        out.push('\n');
        for r in rows {
            out += &format!("{:lw$}  {:18} {:>3} {:>2}", r.label, r.hand, r.hcp, r.tens);
            for c in &r.cells {
                let cell = if c.bba == c.ours {
                    short(&c.bba)
                } else {
                    format!("{} ≠{}", short(&c.bba), short(&c.ours))
                };
                out += &format!("  {cell:cw$}");
            }
            out.push('\n');
        }
        out
    }

    /// For a survey: the exchanges that keep the shape, counted by what
    /// they do to HCP and tens, with BBA's calls in the first column.
    pub fn survey_summary(&self) -> String {
        use std::collections::BTreeMap;
        let Some(base) = self.rows.first() else {
            return String::new();
        };
        let mut groups: BTreeMap<(i32, i32), BTreeMap<String, usize>> = BTreeMap::new();
        let mut other: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for r in &self.rows[1..] {
            let call = short(&r.cells[0].bba);
            if r.shape == base.shape {
                *groups
                    .entry((r.hcp - base.hcp, r.tens - base.tens))
                    .or_default()
                    .entry(call)
                    .or_default() += 1;
            } else {
                let s = r.shape.map(|n| n.to_string()).join("-");
                *other.entry(s).or_default().entry(call).or_default() += 1;
            }
        }
        let fmt = |m: &BTreeMap<String, usize>| {
            m.iter()
                .map(|(c, n)| format!("{c} {n}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut out = format!(
            "same shape ({}), by change in HCP and tens [{}]:\n",
            base.shape.map(|n| n.to_string()).join("-"),
            self.columns[0].name()
        );
        for ((h, t), m) in &groups {
            out += &format!("  HCP {h:+} tens {t:+}: {}\n", fmt(m));
        }
        out += "shape changes, by new shape:\n";
        for (s, m) in &other {
            out += &format!("  {s}: {}\n", fmt(m));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spec_reads_and_picks_the_first_partner_that_fits() {
        let spec: Spec = toml::from_str(
            r#"
card = "Basic-Bridge"
dealer = "S"
prefix = "1NT Pass"
partner = ["K92.A7.KJ532.A43", "K92.A7.J532.AKJ4"]
hands = ["C QT | A5.K86.9764.QT85", "D KT | A5.Q86.KT64.9875"]
"#,
        )
        .unwrap();
        let partners: Vec<Hand> = spec.partner.iter().map(|p| hand(p, "p").unwrap()).collect();
        let mut rng = Rng(1);
        // D KT clashes with the first partner's diamond king: the second is used.
        let v = hand("A5.Q86.KT64.9875", "v").unwrap();
        let d = deal_for(&v, "D KT", Direction::North, &partners, None, &[], &mut rng).unwrap();
        assert_eq!(d.hand(Direction::South), &partners[1]);
        assert_eq!(d.hand(Direction::North), &v);
        // No partner fits: an error naming the variant.
        let err = deal_for(
            &v,
            "D KT",
            Direction::North,
            &partners[..1],
            None,
            &[],
            &mut rng,
        )
        .unwrap_err();
        assert!(err.contains("D KT"), "{err}");
        assert!(toml::from_str::<Spec>("card = \"x\"\ndealer = \"S\"\nbogus = 1").is_err());
    }
}
