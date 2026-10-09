//! `rbb dd`: double-dummy tables and par for the deals of PBN files, one
//! JSON line per distinct (deal, vulnerability). Written for
//! `probes/tools/brill_report.py`, which reads a Brill field PBN (the same
//! deal at many tables) and needs each deal solved once.
//!
//! Games are the blocks between blank lines; `[Deal "#"]` and
//! `[Vulnerable "#"]` repeat the previous game's value (PBN export format).
//! Tables are cached in the same file as `compare`'s (`--dd-cache`).

use std::collections::BTreeSet;
use std::path::Path;

use bridge_types::{Deal, Direction, Vulnerability, DECLARERS, STRAINS};
use rayon::prelude::*;
use rbb_compare::par;

/// The distinct (deal, vulnerability) pairs of a PBN text, in PBN order
/// (`N:...` with North first), in the order first seen.
pub fn deals_of(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let (mut deal, mut vul) = (None::<String>, None::<String>);
    let (mut prev_deal, mut prev_vul) = (None::<String>, None::<String>);
    let mut flush = |deal: &mut Option<String>,
                     vul: &mut Option<String>,
                     prev_deal: &mut Option<String>,
                     prev_vul: &mut Option<String>| {
        if let Some(d) = deal.take() {
            let v = vul.take().unwrap_or_else(|| "None".into());
            if seen.insert((d.clone(), v.clone())) {
                out.push((d.clone(), v.clone()));
            }
            *prev_deal = Some(d);
            *prev_vul = Some(v);
        }
        *vul = None;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            flush(&mut deal, &mut vul, &mut prev_deal, &mut prev_vul);
            continue;
        }
        let Some((tag, value)) = tag_line(line) else {
            continue;
        };
        match tag {
            "Deal" => {
                deal = if value == "#" {
                    prev_deal.clone()
                } else {
                    Deal::from_pbn(value).map(|d| d.to_pbn(Direction::North))
                }
            }
            "Vulnerable" => {
                vul = if value == "#" {
                    prev_vul.clone()
                } else {
                    Some(value.to_string())
                }
            }
            _ => {}
        }
    }
    flush(&mut deal, &mut vul, &mut prev_deal, &mut prev_vul);
    out
}

fn tag_line(line: &str) -> Option<(&str, &str)> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?;
    let (tag, rest) = inner.split_once(char::is_whitespace)?;
    let value = rest.trim().strip_prefix('"')?.strip_suffix('"')?;
    Some((tag, value))
}

/// One JSON object: the deal, its vulnerability, the tricks by declarer
/// (N, E, S, W) and strain (C, D, H, S, NT), and par (North-South's score,
/// the par contracts, and the rank of the highest par contract, 1C = 1).
pub fn row(deal: &str, vul: &str, cache: &par::DdCache) -> Result<serde_json::Value, String> {
    let d = Deal::from_pbn(deal).ok_or_else(|| format!("bad deal {deal:?}"))?;
    if !d.is_valid() {
        return Err(format!("not a full, valid deal: {deal:?}"));
    }
    let v = Vulnerability::from_pbn(vul).ok_or_else(|| format!("bad vulnerability {vul:?}"))?;
    let table = cache.table(&d);
    let mut tricks = serde_json::Map::new();
    for decl in DECLARERS {
        let t: Vec<u8> = STRAINS.iter().map(|s| table.tricks(decl, *s)).collect();
        tricks.insert(decl.to_char().to_string(), t.into());
    }
    let (par_ns, par_text) = par::par_ns(&table, v);
    Ok(serde_json::json!({
        "deal": deal,
        "vul": v.to_pbn(),
        "tricks": tricks,
        "par_ns": par_ns,
        "par": par_text,
        "par_rank": par::par_rank(&table, v),
    }))
}

/// Solve every distinct deal of `inputs` and print one JSON line each.
pub fn run(inputs: &[impl AsRef<Path>], cache: &Path) -> Result<(), String> {
    let mut pairs = Vec::new();
    for p in inputs {
        let p = p.as_ref();
        let text = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
        for pair in deals_of(&text) {
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
    }
    let cache = par::DdCache::open(cache.to_path_buf());
    let rows: Vec<Result<serde_json::Value, String>> =
        pairs.par_iter().map(|(d, v)| row(d, v, &cache)).collect();
    for r in rows {
        println!("{}", r?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEAL: &str = "N:AKJ642.742.Q5.J7 T.A953.T942.K953 98.KJ6.AK87.AQ62 Q753.QT8.J63.T84";

    #[test]
    fn games_repeat_with_a_hash_and_dedupe() {
        let text = format!(
            "[Board \"1\"]\n[Vulnerable \"None\"]\n[Deal \"{DEAL}\"]\n[South \"A\"]\n\n\
             [Board \"1\"]\n[Vulnerable \"#\"]\n[Deal \"#\"]\n[South \"B\"]\n\n\
             [Board \"2\"]\n[Vulnerable \"NS\"]\n[Deal \"{DEAL}\"]\n"
        );
        let got = deals_of(&text);
        assert_eq!(
            got,
            vec![
                (DEAL.to_string(), "None".to_string()),
                (DEAL.to_string(), "NS".to_string())
            ]
        );
    }

    #[test]
    fn a_deal_not_led_by_north_is_rotated() {
        let text =
            "[Deal \"S:98.KJ6.AK87.AQ62 Q753.QT8.J63.T84 AKJ642.742.Q5.J7 T.A953.T942.K953\"]\n";
        assert_eq!(deals_of(text)[0].0, DEAL);
    }

    #[test]
    fn a_row_carries_the_table_and_par() {
        let dir = std::env::temp_dir().join(format!("rbb-dd-test-{}", std::process::id()));
        let cache = par::DdCache::open(dir.join("dd.jsonl"));
        let r = row(DEAL, "None", &cache).unwrap();
        // The corpus's own table for this deal (1N.pbn board 1): N and S
        // make 11 in spades, 10 in notrump.
        assert_eq!(r["tricks"]["N"][3], 11);
        assert_eq!(r["tricks"]["S"][4], 10);
        assert_eq!(r["par_ns"], 450);
        assert_eq!(r["par_rank"], 19); // 4S
        let _ = std::fs::remove_dir_all(dir);
    }
}
