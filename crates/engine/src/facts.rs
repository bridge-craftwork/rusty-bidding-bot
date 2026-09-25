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

/// How points are counted, in quarter points. Two measures: total points
/// (`points`), HCP plus `ten` per ten plus `nt_length` per card beyond four
/// in a suit; and `suit_points`, HCP plus `length` per card beyond four.
/// Tens at ½ are what BBA does when probed hand by hand (see
/// docs/LANGUAGE.md, "Points"). Total points count a full point of length
/// (Rick, 2026-09-23: total points include length points; +9,718 IMPs
/// against par over the corpus, where BBA's own count leaves length out).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Valuation {
    pub ten: i32,
    pub length: i32,
    pub nt_length: i32,
}

impl Default for Valuation {
    fn default() -> Self {
        Valuation {
            ten: 2,
            length: 2,
            nt_length: 4,
        }
    }
}

impl Valuation {
    /// The valuation a card asks for: `general.style = bba` counts as BBA
    /// does, tens and no length at notrump (docs/LANGUAGE.md, "Points").
    pub fn for_card(card: &bridge_card::Card) -> Valuation {
        let bba = matches!(
            card.effective("general.style"),
            Some(bridge_card::Value::Text(s)) if s == "bba"
        );
        Valuation {
            nt_length: if bba { 0 } else { 4 },
            ..Valuation::default()
        }
    }
}

pub const ACE: u8 = 14;
pub const KING: u8 = 13;
pub const QUEEN: u8 = 12;
pub const JACK: u8 = 11;
pub const TEN: u8 = 10;

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

    /// Total points in quarters: HCP plus tens plus length.
    pub fn points_q(&self, v: Valuation) -> i32 {
        4 * self.hcp + v.ten * self.tens + v.nt_length * self.excess
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

    /// BBA's notrump counts for responding to a 15-17 1NT at matchpoints,
    /// fitted per shape to random 7-11 HCP hands BBA bid
    /// (conventions/notrump/one-nt.notes.md, "BBA's count"): weights for
    /// A, K, Q, J and tens, a charge for a doubleton spade or heart without
    /// the ace or king, and a threshold. `bba_nt_points` is scaled so BBA
    /// invites (2NT rather than pass) from 8, `bba_nt_game_points` so it
    /// bids game (3NT rather than 2NT) from 10. Shapes not fitted count HCP
    /// + 1/2 a ten.
    pub fn bba_nt_points(&self) -> i32 {
        const INVITE: [([i32; 4], [i32; 8]); 12] = [
            ([2, 2, 4, 5], [425, 300, 205, 95, 3, 18, 74, 738]),
            ([2, 2, 5, 4], [421, 300, 207, 94, -1, 13, 53, 767]),
            ([2, 3, 3, 5], [387, 300, 177, 70, 27, 1, 0, 694]),
            ([2, 3, 4, 4], [393, 300, 189, 88, 8, 95, 0, 703]),
            ([2, 3, 5, 3], [433, 300, 194, 95, 0, 52, 0, 767]),
            ([3, 2, 3, 5], [441, 300, 186, 89, 17, 0, 54, 739]),
            ([3, 2, 4, 4], [415, 300, 184, 88, 33, 0, 8, 747]),
            ([3, 2, 5, 3], [401, 300, 190, 87, 7, 0, 11, 740]),
            ([3, 3, 2, 5], [398, 300, 190, 83, 38, 0, 0, 726]),
            ([3, 3, 3, 4], [402, 300, 203, 91, 76, 0, 0, 825]),
            ([3, 3, 4, 3], [400, 300, 200, 88, 74, 0, 0, 822]),
            ([3, 3, 5, 2], [399, 300, 200, 92, 1, 0, 0, 746]),
        ];
        self.bba_count(&INVITE, 8)
    }

    pub fn bba_nt_game_points(&self) -> i32 {
        const GAME: [([i32; 4], [i32; 8]); 12] = [
            ([2, 2, 4, 5], [370, 300, 197, 102, 20, 16, 74, 912]),
            ([2, 2, 5, 4], [378, 300, 194, 104, 26, 20, 43, 921]),
            ([2, 3, 3, 5], [410, 300, 182, 84, 34, 1, 0, 934]),
            ([2, 3, 4, 4], [375, 300, 199, 108, 23, 75, 0, 926]),
            ([2, 3, 5, 3], [389, 300, 201, 102, 25, 50, 0, 928]),
            ([3, 2, 3, 5], [373, 300, 203, 108, 27, 0, 82, 928]),
            ([3, 2, 4, 4], [389, 300, 183, 91, 38, 0, 40, 931]),
            ([3, 2, 5, 3], [410, 300, 195, 86, 35, 0, 2, 942]),
            ([3, 3, 2, 5], [407, 300, 204, 100, 51, 0, 0, 980]),
            ([3, 3, 3, 4], [396, 300, 206, 103, 47, 0, 0, 985]),
            ([3, 3, 4, 3], [397, 300, 209, 104, 46, 0, 0, 989]),
            ([3, 3, 5, 2], [409, 300, 207, 102, 44, 0, 0, 976]),
        ];
        self.bba_count(&GAME, 10)
    }

    /// The same two counts at IMPs (fitted to the same hands bid at IMPs;
    /// the invitation line hardly moves, game comes a point or more earlier
    /// with a five-card minor or 4-4 minors).
    pub fn bba_nt_imp_points(&self) -> i32 {
        const INVITE: [([i32; 4], [i32; 8]); 12] = [
            ([2, 2, 4, 5], [431, 300, 204, 94, 3, 7, 80, 735]),
            ([2, 2, 5, 4], [423, 300, 206, 95, 1, 19, 52, 770]),
            ([2, 3, 3, 5], [404, 300, 185, 75, 28, 7, 0, 711]),
            ([2, 3, 4, 4], [394, 300, 189, 88, 6, 99, 0, 699]),
            ([2, 3, 5, 3], [434, 300, 195, 96, -3, 54, 0, 762]),
            ([3, 2, 3, 5], [444, 300, 185, 90, 18, 0, 60, 739]),
            ([3, 2, 4, 4], [418, 300, 184, 91, 31, 0, 5, 755]),
            ([3, 2, 5, 3], [399, 300, 191, 89, -2, 0, 7, 733]),
            ([3, 3, 2, 5], [408, 300, 194, 89, 37, 0, 0, 747]),
            ([3, 3, 3, 4], [405, 300, 201, 92, 78, 0, 0, 822]),
            ([3, 3, 4, 3], [401, 300, 195, 92, 80, 0, 0, 816]),
            ([3, 3, 5, 2], [395, 300, 196, 98, -7, 0, 0, 739]),
        ];
        self.bba_count(&INVITE, 8)
    }

    pub fn bba_nt_imp_game_points(&self) -> i32 {
        const GAME: [([i32; 4], [i32; 8]); 12] = [
            ([2, 2, 4, 5], [354, 300, 187, 76, 2, 22, 151, 745]),
            ([2, 2, 5, 4], [354, 300, 180, 83, 2, 33, 117, 747]),
            ([2, 3, 3, 5], [423, 300, 176, 88, 27, 63, 0, 814]),
            ([2, 3, 4, 4], [349, 300, 192, 98, 10, 149, 0, 778]),
            ([2, 3, 5, 3], [391, 300, 200, 100, 10, 125, 0, 819]),
            ([3, 2, 3, 5], [365, 300, 194, 89, 11, 0, 165, 767]),
            ([3, 2, 4, 4], [378, 300, 174, 86, 27, 0, 91, 789]),
            ([3, 2, 5, 3], [414, 300, 179, 90, 28, 0, 64, 810]),
            ([3, 3, 2, 5], [408, 300, 200, 93, 28, 0, 0, 862]),
            ([3, 3, 3, 4], [399, 300, 206, 108, 66, 0, 0, 980]),
            ([3, 3, 4, 3], [397, 300, 208, 110, 63, 0, 0, 983]),
            ([3, 3, 5, 2], [411, 300, 202, 96, 29, 0, 0, 869]),
        ];
        self.bba_count(&GAME, 10)
    }

    /// BBA's count after Stayman and no fit (e.g. 1NT-2C-2D): 3NT rather than 2NT, matchpoints; scaled so the higher call starts at 10.
    pub fn bba_stay_nt_points(&self) -> i32 {
        self.bba_linear(
            &[
                394, 300, 195, 87, 31, -170, -18, -1, 4, 0, -3, 87, -3, -13, 11, 942,
            ],
            10,
        )
    }

    /// BBA's count the same at IMPs; scaled so the higher call starts at 10.
    pub fn bba_stay_nt_imp_points(&self) -> i32 {
        self.bba_linear(
            &[
                393, 300, 195, 87, 35, -220, -31, -213, -82, 0, 0, 71, -82, -169, -213, 699,
            ],
            10,
        )
    }

    /// BBA's count with the heart fit found: an invitation rather than pass, matchpoints; scaled so the higher call starts at 8.
    pub fn bba_stay_raise_points(&self) -> i32 {
        self.bba_linear(
            &[
                395, 300, 194, 81, -1, -2, 0, -149, -19, 0, 14, -1, 9, 76, 75, 723,
            ],
            8,
        )
    }

    /// BBA's count the same at IMPs; scaled so the higher call starts at 8.
    pub fn bba_stay_raise_imp_points(&self) -> i32 {
        self.bba_linear(
            &[
                413, 300, 201, 93, 1, -6, 0, -126, -15, 0, 12, 2, 9, 68, 55, 734,
            ],
            8,
        )
    }

    /// BBA's count with the heart fit found: game rather than an invitation, matchpoints; scaled so the higher call starts at 10.
    pub fn bba_stay_game_points(&self) -> i32 {
        self.bba_linear(
            &[
                396, 300, 192, 67, 3, 13, 0, -167, -57, 0, 20, -4, -17, 25, 179, 862,
            ],
            10,
        )
    }

    /// BBA's count the same at IMPs; scaled so the higher call starts at 10.
    pub fn bba_stay_game_imp_points(&self) -> i32 {
        self.bba_linear(
            &[
                420, 300, 193, 71, 9, 28, 0, -339, -137, 0, 14, 4, -110, -144, -5, 639,
            ],
            10,
        )
    }

    /// BBA's count with the spade fit found: an invitation rather than pass, matchpoints; scaled so the higher call starts at 8.
    pub fn bba_stay_sraise_points(&self) -> i32 {
        self.bba_linear(
            &[
                404, 300, 196, 87, 1, 0, -34, -134, 3, 0, 1, 19, 4, 45, 83, 738,
            ],
            8,
        )
    }

    /// BBA's count the same at IMPs; scaled so the higher call starts at 8.
    pub fn bba_stay_sraise_imp_points(&self) -> i32 {
        self.bba_linear(
            &[
                407, 300, 199, 101, 4, 0, -44, -86, -11, 0, 5, 17, -1, 9, 83, 715,
            ],
            8,
        )
    }

    /// BBA's count with the spade fit found: game rather than an invitation, matchpoints; scaled so the higher call starts at 10.
    pub fn bba_stay_sgame_points(&self) -> i32 {
        self.bba_linear(
            &[
                412, 300, 194, 67, 2, 0, -21, -116, -36, 0, 9, 14, -17, -3, 129, 874,
            ],
            10,
        )
    }

    /// BBA's count the same at IMPs; scaled so the higher call starts at 10.
    pub fn bba_stay_sgame_imp_points(&self) -> i32 {
        self.bba_linear(
            &[
                432, 300, 200, 75, 4, 0, -14, -273, -112, 0, 4, 27, -104, -177, -42, 660,
            ],
            10,
        )
    }

    /// A fitted BBA count by the name rules use (`bba_nt_points`, ...).
    pub fn bba_named(&self, name: &str) -> Option<i32> {
        Some(match name {
            "bba_nt_points" => self.bba_nt_points(),
            "bba_nt_game_points" => self.bba_nt_game_points(),
            "bba_nt_imp_points" => self.bba_nt_imp_points(),
            "bba_nt_imp_game_points" => self.bba_nt_imp_game_points(),
            "bba_stay_nt_points" => self.bba_stay_nt_points(),
            "bba_stay_nt_imp_points" => self.bba_stay_nt_imp_points(),
            "bba_stay_raise_points" => self.bba_stay_raise_points(),
            "bba_stay_raise_imp_points" => self.bba_stay_raise_imp_points(),
            "bba_stay_game_points" => self.bba_stay_game_points(),
            "bba_stay_game_imp_points" => self.bba_stay_game_imp_points(),
            "bba_stay_sraise_points" => self.bba_stay_sraise_points(),
            "bba_stay_sraise_imp_points" => self.bba_stay_sraise_imp_points(),
            "bba_stay_sgame_points" => self.bba_stay_sgame_points(),
            "bba_stay_sgame_imp_points" => self.bba_stay_sgame_imp_points(),
            _ => return None,
        })
    }

    /// A fitted linear count after Stayman (conventions/notrump/stayman.notes.md,
    /// "BBA's counts after Stayman"): weights, in hundredths, for aces,
    /// kings, queens, jacks, tens, a doubleton spade / heart without A or K,
    /// the shapes 4-3-3-3, 4-4-3-2, 5-3-3-2, 5-4-2-2, a five-card minor,
    /// each doubleton, singleton and void; then the threshold. In whole
    /// points, scaled so the threshold falls at `at`.
    fn bba_linear(&self, w: &[i32; 16], at: i32) -> i32 {
        let count = |r: u8| (0..4).filter(|&s| self.has(s, r)).count() as i32;
        let bare = |s: usize| (self.len[s] <= 2 && !self.has(s, ACE) && !self.has(s, KING)) as i32;
        let shape = |d: [i32; 4]| (self.dist == d) as i32;
        let f = [
            count(ACE),
            count(KING),
            count(QUEEN),
            count(JACK),
            self.tens,
            bare(3),
            bare(2),
            shape([4, 3, 3, 3]),
            shape([4, 4, 3, 2]),
            shape([5, 3, 3, 2]),
            shape([5, 4, 2, 2]),
            (self.len[0].max(self.len[1]) >= 5) as i32,
            self.len.iter().filter(|&&l| l == 2).count() as i32,
            self.len.iter().filter(|&&l| l == 1).count() as i32,
            self.len.iter().filter(|&&l| l == 0).count() as i32,
        ];
        let sum: i32 = f.iter().zip(w).map(|(x, w)| x * w).sum();
        (sum - w[15] + 100 * at).div_euclid(100)
    }

    /// A fitted count (see `bba_nt_points`), in whole points, where `at` is
    /// the fitted threshold.
    fn bba_count(&self, table: &[([i32; 4], [i32; 8])], at: i32) -> i32 {
        let shape = [self.len[3], self.len[2], self.len[1], self.len[0]];
        let count = |r: u8| (0..4).filter(|&s| self.has(s, r)).count() as i32;
        let bare = |s: usize| self.len[s] <= 2 && !self.has(s, ACE) && !self.has(s, KING);
        let hundredths = match table.iter().find(|(sh, _)| *sh == shape) {
            Some((_, w)) => {
                w[0] * count(ACE)
                    + w[1] * count(KING)
                    + w[2] * count(QUEEN)
                    + w[3] * count(JACK)
                    + w[4] * self.tens
                    - w[5] * bare(3) as i32
                    - w[6] * bare(2) as i32
                    - w[7]
                    + 100 * at
            }
            None => 100 * self.hcp + 50 * self.tens,
        };
        hundredths.div_euclid(100)
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

    /// The length of the second-longest suit.
    pub fn second_longest(&self) -> i32 {
        self.dist[1]
    }

    /// Length points: one for each card beyond four in every suit.
    pub fn length_points(&self) -> i32 {
        (0..4).map(|s| (self.len[s] - 4).max(0)).sum()
    }

    /// How many of the top five honours (A K Q J T) are held.
    pub fn top5(&self, suit: usize) -> i32 {
        [ACE, KING, QUEEN, JACK, TEN]
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

    /// Support points: HCP plus shortness in the side suits when `trump` is
    /// a suit — void 5, singleton 3, doubleton 1 — and plain HCP for
    /// notrump.
    ///
    /// **The shortness is capped by the number of trumps held** (Rick,
    /// 2026-09-23): a hand can only ruff as often as it has trumps, so
    /// three trumps are worth at most three points of shortness however
    /// many short suits there are. Without the cap a 3-1-4-5 hand with a
    /// void counted the whole 5.
    pub fn total_points(&self, trump: Option<usize>) -> i32 {
        let Some(t) = trump else { return self.hcp };
        let shortness: i32 = (0..4)
            .filter(|&s| s != t)
            .map(|s| match self.len[s] {
                0 => 5,
                1 => 3,
                2 => 1,
                _ => 0,
            })
            .sum();
        self.hcp + shortness.min(self.len[t])
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

    #[test]
    fn shortness_is_capped_by_the_trumps_held() {
        // Four trumps, a void and a doubleton: 5 + 1 = 6, capped at 4.
        let f = Facts::new(&Hand::from_pbn("KQ54.AQ876.65.").unwrap());
        assert_eq!(f.len[3], 4, "four spades");
        assert_eq!(f.total_points(Some(3)), f.hcp + 4);
        // The same shortness with only three trumps is worth three.
        let f = Facts::new(&Hand::from_pbn("KQ5.AQ8765.65.").unwrap());
        assert_eq!(f.total_points(Some(3)), f.hcp + 3);
        // Nothing short: support points are just the high cards.
        let f = Facts::new(&Hand::from_pbn("KQ54.A87.653.J84").unwrap());
        assert_eq!(f.total_points(Some(3)), f.hcp);
        // Notrump ignores shape entirely.
        assert_eq!(f.total_points(None), f.hcp);
    }
}
