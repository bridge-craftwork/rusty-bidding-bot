//! What the table is told about a call (`Step::announcement`): the rule's
//! description, then what the call showed that the description does not
//! already say. People read it in a tooltip; robots read it too: Brill
//! (rusty-seat-service, docs/seat-api.txt §5) takes suit lengths, HCP
//! ranges and the words *artificial*, *forcing*, *game forcing* and
//! *transfer* out of this text, so those are what it adds, with suits
//! spelled out: "5+ spades; 10-21 HCP", "Stayman: asks for a 4-card major;
//! artificial, forcing".
//!
//! A bound is stated only when this call narrowed it, it is not implied by
//! something else, and it is worth reading: a heart maximum of 8 after five
//! spades is the hand's arithmetic, an HCP maximum of 25 after partner's 15-17
//! notrump is the deck's, and "1+ clubs" or "0-9 spades" says nothing a reader
//! would act on. A low HCP floor is worse than nothing, since a reader takes
//! "5+ HCP" for the strength shown; a raise that shows 13+ support points
//! says that instead.

use crate::{Forcing, Range, SeatKnowledge, Tri};

/// What one call showed, as the engine read it.
pub struct Shown<'a> {
    /// The rule's description of the call (may be empty).
    pub description: &'a str,
    /// The caller, before and after the call.
    pub before: &'a SeatKnowledge,
    pub after: &'a SeatKnowledge,
    /// The other three seats after the call, for what the deck implies.
    pub others: [&'a SeatKnowledge; 3],
    /// The rule marked the call artificial.
    pub artificial: bool,
    /// The force this call set (`None` when it set none).
    pub forcing: Forcing,
    /// The caller's side's agreed suit, by index into
    /// `SeatKnowledge::len` (C D H S): support points are stated in it only.
    pub trump: Option<usize>,
}

/// Suits as a hand is written, by index into `SeatKnowledge::len` (C D H S).
const SUITS: [(usize, &str); 4] = [(3, "spades"), (2, "hearts"), (1, "diamonds"), (0, "clubs")];

/// The least HCP floor worth stating.
const HCP_FLOOR: i32 = 6;
/// The least suit-length floor, and the greatest suit-length ceiling, worth
/// stating.
const LENGTH_FLOOR: i32 = 3;
const LENGTH_CEILING: i32 = 6;

/// The explanation for one call.
pub fn explanation(s: &Shown) -> String {
    let description = words(s.description.trim());
    let said = description.to_lowercase();

    let mut extras = Vec::new();
    let hcp_cap = (40 - s.others.iter().map(|k| k.hcp.lo).sum::<i32>()).min(37);
    if let Some(r) = narrowed(s.after.hcp, s.before.hcp, hcp_cap, HCP_FLOOR, hcp_cap) {
        extras.push(format!("{r} HCP"));
    }
    // Support points in the agreed suit, where they show more than the HCP
    // do (the other suits' are the same count again).
    if let Some(i) = s.trump {
        let tp = s.after.tp[i];
        if tp.lo > s.after.hcp.lo {
            if let Some(r) = narrowed(tp, s.before.tp[i], i32::MAX, HCP_FLOOR, i32::MAX) {
                extras.push(format!("{r} support points"));
            }
        }
    }
    let balanced = match (s.before.balanced, s.after.balanced) {
        (b, a) if a != b && a == Tri::True => Some("balanced"),
        // Two or more in every suit is not "unbalanced" to a reader, though
        // it is outside the strict 4-3-3-3 / 4-4-3-2 / 5-3-3-2: a 5-4-2-2
        // says no shape word at all, as a 4-5-2-2 says "balanced" below.
        (b, a) if a != b && a == Tri::False && s.after.len.iter().any(|r| r.lo < 2) => {
            Some("unbalanced")
        }
        // A notrump range shows no shape flag, but 2-5 in every suit is
        // balanced as far as anyone reading the text is concerned.
        (_, Tri::Unknown) if flat(s.after) && !flat(s.before) => Some("balanced"),
        _ => None,
    };
    match balanced {
        Some("balanced") => extras.push("balanced".into()),
        other => {
            if let Some(word) = other {
                extras.push(word.into());
            }
            // The longest suit shown: a maximum elsewhere that does not go
            // below it only says that suit is the longest ("0-6 hearts"
            // beside "5+ spades").
            let longest = s.after.len.iter().map(|r| r.lo).max().unwrap_or(0);
            for (i, name) in SUITS {
                let in_hand: i32 = (0..4).filter(|&j| j != i).map(|j| s.after.len[j].lo).sum();
                let in_deck: i32 = s.others.iter().map(|k| k.len[i].lo).sum();
                let mut cap = (13 - in_hand).min(13 - in_deck);
                if s.after.len[i].lo < longest {
                    cap = cap.min(longest);
                }
                let r = narrowed(
                    s.after.len[i],
                    s.before.len[i],
                    cap,
                    LENGTH_FLOOR,
                    LENGTH_CEILING,
                );
                if let Some(r) = r {
                    extras.push(format!("{r} {name}"));
                }
            }
        }
    }
    if s.artificial {
        extras.push("artificial".into());
    }
    match s.forcing {
        Forcing::Round => extras.push("forcing".into()),
        Forcing::Game => extras.push("game forcing".into()),
        Forcing::None => {}
    }
    extras.retain(|e| !said.contains(&e.to_lowercase()));

    match (description.is_empty(), extras.is_empty()) {
        (_, true) => description,
        (true, false) => extras.join(", "),
        (false, false) => format!("{description}; {}", extras.join(", ")),
    }
}

/// At most 5 and at least 2 in every suit.
fn flat(k: &SeatKnowledge) -> bool {
    k.len.iter().all(|r| r.lo >= 2 && r.hi <= 5)
}

/// A range as text when the call narrowed it to something worth stating:
/// `lo-hi` when the maximum is news, `lo+` when only the minimum is. A
/// maximum is news below `cap` (what is implied anyway) and up to `ceiling`;
/// a minimum from `floor`.
fn narrowed(after: Range, before: Range, cap: i32, floor: i32, ceiling: i32) -> Option<String> {
    let hi_new = after.hi < before.hi && after.hi < cap && after.hi <= ceiling;
    let lo_new = after.lo > before.lo && after.lo >= floor;
    if hi_new {
        Some(if after.lo == after.hi {
            after.lo.to_string()
        } else {
            format!("{}-{}", after.lo, after.hi)
        })
    } else if lo_new {
        Some(format!("{}+", after.lo))
    } else {
        None
    }
}

/// Suit symbols as text: a letter after a level (`4♠` is `4S`), the
/// suit's name before a word it describes (`♠ queen` is `spade queen`),
/// else the suit's plural (`3+ ♠` is `3+ spades`).
pub fn words(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (k, &c) in chars.iter().enumerate() {
        let suit = match c {
            '♠' => Some(("S", "spade")),
            '♥' => Some(("H", "heart")),
            '♦' => Some(("D", "diamond")),
            '♣' => Some(("C", "club")),
            _ => None,
        };
        let Some((letter, name)) = suit else {
            out.push(c);
            continue;
        };
        let after_level = k > 0 && chars[k - 1].is_ascii_digit();
        let before_word = chars.get(k + 1) == Some(&' ')
            && chars.get(k + 2).is_some_and(|n| n.is_ascii_lowercase());
        if after_level {
            out.push_str(letter);
        } else {
            out.push_str(name);
            if !before_word {
                out.push('s');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn know(hcp: (i32, i32), len: [(i32, i32); 4]) -> SeatKnowledge {
        SeatKnowledge {
            hcp: Range::new(hcp.0, hcp.1),
            len: len.map(|(lo, hi)| Range::new(lo, hi)),
            ..Default::default()
        }
    }

    const ANY: [(i32, i32); 4] = [(0, 13); 4];

    fn shown<'a>(
        description: &'a str,
        before: &'a SeatKnowledge,
        after: &'a SeatKnowledge,
        others: [&'a SeatKnowledge; 3],
    ) -> Shown<'a> {
        Shown {
            description,
            before,
            after,
            others,
            artificial: false,
            forcing: Forcing::None,
            trump: None,
        }
    }

    #[test]
    fn opening_one_spade() {
        let fresh = SeatKnowledge::default();
        // C D H S: five-plus spades leaves at most 8 elsewhere.
        let after = know((10, 21), [(0, 8), (0, 8), (0, 8), (5, 13)]);
        let other = know((0, 30), [(0, 13), (0, 13), (0, 13), (0, 8)]);
        let s = shown("5+ spades", &fresh, &after, [&other, &other, &other]);
        assert_eq!(explanation(&s), "5+ spades; 10-21 HCP");
    }

    #[test]
    fn longest_suit_is_not_news_elsewhere() {
        // The engine's 1♠: spades the longest suit, so 0-6 in the others.
        let fresh = SeatKnowledge::default();
        let after = know((10, 21), [(0, 6), (0, 6), (0, 6), (5, 13)]);
        let s = shown("5+ spades", &fresh, &after, [&fresh, &fresh, &fresh]);
        assert_eq!(explanation(&s), "5+ spades; 10-21 HCP");
    }

    #[test]
    fn support_points_in_the_agreed_suit_only() {
        let fresh = SeatKnowledge::default();
        let mut after = know((12, 21), [(0, 1), (0, 8), (0, 8), (5, 13)]);
        after.tp = [
            Range::new(12, 60),
            Range::new(12, 60),
            Range::new(12, 60),
            Range::new(13, 60),
        ];
        let mut s = shown("Shortness", &fresh, &after, [&fresh, &fresh, &fresh]);
        s.trump = Some(3);
        assert_eq!(
            explanation(&s),
            "Shortness; 12-21 HCP, 13+ support points, 5+ spades, 0-1 clubs"
        );
    }

    #[test]
    fn two_in_every_suit_is_not_unbalanced() {
        // 1♥ P 2NT P 3NT on a card that opens 5-3-3-2 1NT: 5-4-2-2 or 6-3-2-2.
        let before = know((10, 21), [(0, 8), (0, 8), (5, 13), (0, 8)]);
        let mut after = know((15, 17), [(2, 4), (2, 4), (5, 6), (2, 4)]);
        after.balanced = Tri::False;
        let fresh = SeatKnowledge::default();
        let s = shown(
            "15-17, no shortness",
            &before,
            &after,
            [&fresh, &fresh, &fresh],
        );
        assert_eq!(
            explanation(&s),
            "15-17, no shortness; 15-17 HCP, 5-6 hearts"
        );
    }

    #[test]
    fn stayman_flags() {
        let fresh = SeatKnowledge::default();
        let mut s = shown(
            "Stayman: asks for a 4-card major",
            &fresh,
            &fresh,
            [&fresh, &fresh, &fresh],
        );
        s.artificial = true;
        s.forcing = Forcing::Round;
        assert_eq!(
            explanation(&s),
            "Stayman: asks for a 4-card major; artificial, forcing"
        );
    }

    #[test]
    fn already_said_is_not_repeated() {
        let fresh = SeatKnowledge::default();
        let mut after = know((5, 37), [(0, 9), (0, 9), (0, 9), (4, 13)]);
        after.tp[3] = Range::new(13, 60);
        let mut s = shown(
            "Jacoby 2NT: 4+ ♠, game forcing",
            &fresh,
            &after,
            [&fresh, &fresh, &fresh],
        );
        s.artificial = true;
        s.forcing = Forcing::Game;
        s.trump = Some(3);
        assert_eq!(
            explanation(&s),
            "Jacoby 2NT: 4+ spades, game forcing; 13+ support points, artificial"
        );
    }

    #[test]
    fn notrump_range_is_balanced() {
        let fresh = SeatKnowledge::default();
        let after = know((15, 17), [(2, 5); 4]);
        let s = shown("15-17 HCP", &fresh, &after, [&fresh, &fresh, &fresh]);
        assert_eq!(explanation(&s), "15-17 HCP; balanced");
    }

    #[test]
    fn balanced_is_said_once() {
        // 1NT 2C 2H: the 1NT opener's hearts narrow, still 2-5 everywhere.
        let before = know((15, 17), [(2, 5); 4]);
        let after = know((15, 17), [(2, 5), (2, 5), (4, 5), (2, 5)]);
        let fresh = SeatKnowledge::default();
        let s = shown("4+ hearts", &before, &after, [&fresh, &fresh, &fresh]);
        assert_eq!(explanation(&s), "4+ hearts");
    }

    #[test]
    fn deck_caps_are_not_news() {
        // Responder to partner's 15-17: 25 HCP at most is the deck talking.
        let fresh = SeatKnowledge::default();
        let partner = know((15, 17), [(2, 5); 4]);
        let after = know((0, 25), [(0, 11); 4]);
        let s = shown("", &fresh, &after, [&partner, &fresh, &fresh]);
        assert_eq!(explanation(&s), "");
        // A pass that denies an overcall does say something.
        let after = know((0, 14), ANY);
        let s = shown("No overcall", &fresh, &after, [&partner, &fresh, &fresh]);
        assert_eq!(explanation(&s), "No overcall; 0-14 HCP");
    }

    #[test]
    fn rebid_shortness() {
        // 1H 1S 1NT: at most three spades is news, hearts 5-10 is not.
        let before = know((10, 21), [(0, 8), (0, 8), (5, 13), (0, 8)]);
        let after = know((12, 14), [(0, 8), (0, 8), (5, 10), (0, 3)]);
        let fresh = SeatKnowledge::default();
        let s = shown(
            "12-14 HCP, balanced",
            &before,
            &after,
            [&fresh, &fresh, &fresh],
        );
        assert_eq!(explanation(&s), "12-14 HCP, balanced; 0-3 spades");
    }

    #[test]
    fn weak_bounds_are_not_stated() {
        // Stayman: 1+ HCP and 0-9 in each suit are true and useless.
        let fresh = SeatKnowledge::default();
        let partner = know((15, 17), [(2, 5); 4]);
        let after = know((1, 25), [(0, 9); 4]);
        let s = shown("Stayman", &fresh, &after, [&fresh, &partner, &fresh]);
        assert_eq!(explanation(&s), "Stayman");
        // 1+ in a minor beside a game try.
        let before = know((10, 21), [(0, 8), (0, 8), (0, 8), (5, 13)]);
        let after = know((14, 18), [(1, 8), (1, 8), (3, 8), (5, 13)]);
        let s = shown("Game try", &before, &after, [&fresh, &fresh, &fresh]);
        assert_eq!(explanation(&s), "Game try; 14-18 HCP, 3+ hearts");
    }

    #[test]
    fn suit_symbols() {
        assert_eq!(
            words("3+ ♠, splinter in ♦"),
            "3+ spades, splinter in diamonds"
        );
        assert_eq!(
            words("Cue bid 4♣ shows ♣ control"),
            "Cue bid 4C shows club control"
        );
        assert_eq!(
            words("2 or 5 keycards, no ♠ queen"),
            "2 or 5 keycards, no spade queen"
        );
        assert_eq!(words("Keycard ask in ♥"), "Keycard ask in hearts");
    }
}
