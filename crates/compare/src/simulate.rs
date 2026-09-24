//! Simulate a decision double dummy: for each hand of a grid spec, deal
//! partner from a pool of hands that made partner's call (e.g. the hands BBA
//! opens 1NT), the opponents at random, and solve. How often does the
//! partnership make 8 or 9 tricks in notrump? A statistical yardstick for
//! the calls BBA makes, not a model of how it makes them.

use bridge_types::{Card, Deal, Direction, Hand, Strain};
use rayon::prelude::*;
use serde::Serialize;

use crate::grid::Spec;

#[derive(Debug, Clone, Serialize)]
pub struct SimRow {
    pub label: String,
    pub hand: String,
    pub samples: usize,
    /// Average notrump tricks, partner (the opener) declaring.
    pub mean_tricks: f64,
    pub make8: f64,
    pub make9: f64,
}

/// Small deterministic generator, seeded per hand.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

fn hand_of(s: &str) -> Result<Hand, String> {
    Hand::from_pbn(s.trim())
        .filter(|h| h.len() == 13)
        .ok_or_else(|| format!("{s:?} is not a 13-card hand"))
}

/// The hands of a spec (a `hands` list only), with labels.
fn spec_hands(spec: &Spec) -> Result<Vec<(String, Hand)>, String> {
    spec.hands
        .iter()
        .map(|line| {
            let parts: Vec<&str> = line.split('|').map(str::trim).collect();
            let (label, h) = match parts.as_slice() {
                [h] => (h.to_string(), *h),
                [l, h, ..] => (l.to_string(), *h),
                _ => return Err(format!("{line:?}")),
            };
            Ok((label, hand_of(h)?))
        })
        .collect()
}

pub fn run(spec: &Spec, pool: &[Hand], samples: usize, seed: u64) -> Result<Vec<SimRow>, String> {
    if spec.hands.is_empty() {
        return Err("simulate takes a spec with a `hands` list".into());
    }
    let dealer = spec
        .dealer
        .chars()
        .next()
        .and_then(|c| Direction::from_char(c.to_ascii_uppercase()))
        .ok_or("dealer must be N, E, S or W")?;
    let seat = (0..spec.prefix.split_whitespace().count()).fold(dealer, |d, _| d.next());
    let partner = seat.partner();
    let hands = spec_hands(spec)?;
    Ok(hands
        .par_iter()
        .enumerate()
        .map(|(i, (label, me))| {
            let mut rng = Rng((seed ^ (i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)) | 1);
            let mine = me.cards();
            let (mut tricks, mut m8, mut m9, mut n) = (0u64, 0usize, 0usize, 0usize);
            let mut tries = 0;
            while n < samples && tries < samples * 200 {
                tries += 1;
                let p = &pool[(rng.next() % pool.len() as u64) as usize];
                if p.cards().iter().any(|c| mine.contains(c)) {
                    continue;
                }
                let mut rest: Vec<Card> = (0..52u8)
                    .filter_map(Card::from_index)
                    .filter(|c| !mine.contains(c) && !p.cards().contains(c))
                    .collect();
                for k in (1..rest.len()).rev() {
                    let j = (rng.next() % (k as u64 + 1)) as usize;
                    rest.swap(k, j);
                }
                let mut deal = Deal::new();
                deal.set_hand(seat, me.clone());
                deal.set_hand(partner, p.clone());
                deal.set_hand(seat.next(), Hand::from_cards(rest[..13].to_vec()));
                deal.set_hand(partner.next(), Hand::from_cards(rest[13..].to_vec()));
                let t = bridge_solver::par::solve_dd_table(&deal).tricks(partner, Strain::NoTrump);
                tricks += t as u64;
                m8 += (t >= 8) as usize;
                m9 += (t >= 9) as usize;
                n += 1;
            }
            let f = |x: usize| if n == 0 { 0.0 } else { x as f64 / n as f64 };
            SimRow {
                label: label.clone(),
                hand: me.to_pbn(),
                samples: n,
                mean_tricks: if n == 0 {
                    0.0
                } else {
                    tricks as f64 / n as f64
                },
                make8: f(m8),
                make9: f(m9),
            }
        })
        .collect())
}

/// Read a pool file: one hand (S.H.D.C) per line; blank lines and # comments
/// skipped.
pub fn read_pool(path: &std::path::Path) -> Result<Vec<Hand>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    text.lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .map(hand_of)
        .collect()
}
