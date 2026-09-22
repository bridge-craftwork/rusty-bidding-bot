//! Exact facts about one hand, computed once. Suits are indexed C=0, D=1,
//! H=2, S=3 (the order of `bridge_types::Suit`).

use bridge_types::Hand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub hcp: i32,
    pub len: [i32; 4],
    /// Per suit, bit `r` set when the hand holds rank value `r` (2..=14).
    pub ranks: [u16; 4],
    pub balanced: bool,
    /// Suit lengths sorted longest first.
    pub dist: [i32; 4],
    /// Number of tens held.
    pub tens: i32,
    /// Cards beyond four in each suit, summed (length points).
    pub excess: i32,
}

/// How total points are counted, in quarter points. There are two measures:
/// for notrump, HCP plus `ten` per ten; for a suit contract, HCP plus
/// `length` per card beyond four in a suit. The defaults (½ each) are what
/// BBA does when probed hand by hand (see docs/LANGUAGE.md, "Points").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Valuation {
    pub ten: i32,
    pub length: i32,
}

impl Default for Valuation {
    fn default() -> Self {
        Valuation { ten: 2, length: 2 }
    }
}

pub const ACE: u8 = 14;
pub const KING: u8 = 13;
pub const QUEEN: u8 = 12;
pub const JACK: u8 = 11;

impl Facts {
    pub fn new(hand: &Hand) -> Facts {
        let mut len = [0; 4];
        let mut ranks = [0u16; 4];
        let mut hcp = 0;
        for card in hand.cards() {
            let s = card.suit as usize;
            len[s] += 1;
            ranks[s] |= 1 << (card.rank as u8);
            hcp += card.hcp() as i32;
        }
        let mut dist = len;
        dist.sort_by(|a, b| b.cmp(a));
        let balanced = matches!(dist, [4, 3, 3, 3] | [4, 4, 3, 2] | [5, 3, 3, 2]);
        let tens = (0..4).filter(|&s| ranks[s] & (1 << 10) != 0).count() as i32;
        let excess = len.iter().map(|l| (l - 4).max(0)).sum();
        Facts {
            tens,
            excess,
            hcp,
            len,
            ranks,
            balanced,
            dist,
        }
    }

    /// Notrump points in quarters: HCP plus tens.
    pub fn points_q(&self, v: Valuation) -> i32 {
        4 * self.hcp + v.ten * self.tens
    }

    /// Suit points in quarters: HCP plus length beyond four.
    pub fn suit_points_q(&self, v: Valuation) -> i32 {
        4 * self.hcp + v.length * self.excess
    }

    /// Points of either kind: 0 notrump, 1 suit.
    pub fn points_of(&self, kind: usize, v: Valuation) -> i32 {
        if kind == 0 {
            self.points_q(v)
        } else {
            self.suit_points_q(v)
        }
    }

    pub fn has(&self, suit: usize, rank: u8) -> bool {
        self.ranks[suit] & (1 << rank) != 0
    }

    /// Aces plus the trump king.
    pub fn keycards(&self, trump: Option<usize>) -> i32 {
        let aces = (0..4).filter(|&s| self.has(s, ACE)).count() as i32;
        aces + trump.map_or(0, |t| self.has(t, KING) as i32)
    }

    /// Ace = 2, king = 1.
    pub fn controls(&self) -> i32 {
        (0..4)
            .map(|s| 2 * self.has(s, ACE) as i32 + self.has(s, KING) as i32)
            .sum()
    }

    /// Losing-trick count: per suit, the top min(length, 3) cards less the
    /// ace, the king (with 2+ cards) and the queen (with 3+ cards) held.
    pub fn losers(&self) -> i32 {
        (0..4)
            .map(|s| {
                let n = self.len[s].min(3);
                let winners = self.has(s, ACE) as i32
                    + (n >= 2 && self.has(s, KING)) as i32
                    + (n >= 3 && self.has(s, QUEEN)) as i32;
                n - winners
            })
            .sum()
    }

    /// Top honours (A, K, Q) held in a suit: 0 poor, 1 fair, 2 good,
    /// 3 excellent.
    pub fn quality(&self, suit: usize) -> i32 {
        [ACE, KING, QUEEN]
            .iter()
            .filter(|&&r| self.has(suit, r))
            .count() as i32
    }

    /// A, Kx, Qxx or Jxxx.
    pub fn stop(&self, suit: usize) -> bool {
        let l = self.len[suit];
        self.has(suit, ACE)
            || (l >= 2 && self.has(suit, KING))
            || (l >= 3 && self.has(suit, QUEEN))
            || (l >= 4 && self.has(suit, JACK))
    }

    /// HCP plus shortness in the side suits (void 5, singleton 3,
    /// doubleton 1) when `trump` is a suit; plain HCP for notrump.
    pub fn total_points(&self, trump: Option<usize>) -> i32 {
        let Some(t) = trump else { return self.hcp };
        self.hcp
            + (0..4)
                .filter(|&s| s != t)
                .map(|s| match self.len[s] {
                    0 => 5,
                    1 => 3,
                    2 => 1,
                    _ => 0,
                })
                .sum::<i32>()
    }

    /// Match a shape pattern against the sorted distribution: `4333`,
    /// `5-4-x-x` (x = any).
    pub fn shape_matches(&self, pattern: &str) -> Option<bool> {
        let parts: Vec<&str> = if pattern.contains('-') {
            pattern.split('-').collect()
        } else {
            pattern.split("").filter(|s| !s.is_empty()).collect()
        };
        if parts.len() != 4 {
            return None;
        }
        let mut ok = true;
        for (i, p) in parts.iter().enumerate() {
            if *p == "x" {
                continue;
            }
            ok &= p.parse::<i32>().ok()? == self.dist[i];
        }
        Some(ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_of_a_hand() {
        // S: AKQ5 H: KQ7 D: A95 C: K87
        let f = Facts::new(&Hand::from_pbn("AKQ5.KQ7.A95.K87").unwrap());
        assert_eq!(f.hcp, 21);
        assert_eq!(f.len, [3, 3, 3, 4]);
        assert!(f.balanced);
        assert_eq!(f.keycards(Some(3)), 3);
        assert_eq!(f.quality(3), 3);
        assert!(f.stop(1));
        assert_eq!(f.shape_matches("4333"), Some(true));
        assert_eq!(f.shape_matches("4-x-x-3"), Some(true));
        // Losers: spades 0, hearts KQx 1, diamonds Axx 2, clubs Kxx 2.
        assert_eq!(f.losers(), 5);
    }
}
