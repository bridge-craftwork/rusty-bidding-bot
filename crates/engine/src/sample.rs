//! A fixed sample of random hands, used to measure how much a call says
//! (descriptiveness). Deterministic, so rankings are reproducible.

use bridge_types::{Card, Hand};

use crate::facts::Facts;

/// Number of deals sampled (four hands each).
const DEALS: usize = 5_000;

/// xorshift64*: small, fast, and good enough for shuffling.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

pub fn pool() -> Vec<Facts> {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut deck: Vec<Card> = (0..52).filter_map(Card::from_index).collect();
    let mut out = Vec::with_capacity(DEALS * 4);
    for _ in 0..DEALS {
        for i in (1..52).rev() {
            let j = rng.below(i + 1);
            deck.swap(i, j);
        }
        for h in 0..4 {
            out.push(Facts::new(&Hand::from_cards(
                deck[h * 13..h * 13 + 13].to_vec(),
            )));
        }
    }
    out
}
