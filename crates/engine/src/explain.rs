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
//!
//! People and robots want different texts (issue #98), so the text is
//! written for an `Audience`: the step keeps what the call showed
//! (`Announced`) and renders it for whoever asks (`Step::announcement_for`).

use serde::Serialize;

use crate::{Forcing, Range, SeatKnowledge, Tri};

/// Who reads the text. `Human` is the classroom, the auction explorer, the
/// CLI and the WASM default; `Brill` is the seat service, whose opponents
/// are robots that pull lengths, HCP ranges and a few keywords out of free
/// text (rusty-seat-service docs/seat-api.txt §5: length, point range and
/// the words *artificial*, *forcing*, *game forcing* and *transfer*).
///
/// `Brill` words the text the way a heuristic parser reads it:
/// - lengths in digits: "five or more hearts" is "5+ hearts", "at most
///   one diamond" is "0-1 diamonds", "six hearts" is "6 hearts";
/// - a transfer names its target ("transfer to spades" after Texas);
/// - no negated "forcing" for a substring match to misread: "not forcing"
///   and "non-forcing" are "passable", "forcing to game" is "game forcing".
///
/// The thresholds are the same for both (the `Policy` keeps them apart for
/// when they are not). Lower floors were tried on Stayman, Jacoby
/// transfer, Splinters and Jacoby 2NT deals and only misled: "5+ HCP"
/// beside "13+ support points" (Jacoby 2NT) reads as the strength shown,
/// and a 2-card floor puts "2+ diamonds" on keycard asks and signoffs,
/// which a parser can take for the suit the call shows.
///
/// `Human` is the text as it always was.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Audience {
    #[default]
    Human,
    Brill,
}

impl Audience {
    pub const ALL: [Audience; 2] = [Audience::Human, Audience::Brill];

    pub fn as_str(self) -> &'static str {
        match self {
            Audience::Human => "human",
            Audience::Brill => "brill",
        }
    }

    fn policy(self) -> Policy {
        match self {
            Audience::Human => Policy {
                hcp_floor: HCP_FLOOR,
                length_floor: LENGTH_FLOOR,
                length_ceiling: LENGTH_CEILING,
                robot: false,
            },
            Audience::Brill => Policy {
                hcp_floor: HCP_FLOOR,
                length_floor: LENGTH_FLOOR,
                length_ceiling: LENGTH_CEILING,
                robot: true,
            },
        }
    }
}

impl std::fmt::Display for Audience {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Audience {
    type Err = String;
    fn from_str(s: &str) -> Result<Audience, String> {
        match s.to_ascii_lowercase().as_str() {
            "human" => Ok(Audience::Human),
            "brill" => Ok(Audience::Brill),
            _ => Err(format!("audience must be human or brill, not {s:?}")),
        }
    }
}

/// What an audience is told, as numbers and switches.
struct Policy {
    hcp_floor: i32,
    length_floor: i32,
    length_ceiling: i32,
    /// Lengths in digits, the transfer's target, no negated "forcing".
    robot: bool,
}

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

/// The part of a seat's knowledge an explanation reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sketch {
    pub hcp: Range,
    /// C D H S.
    pub len: [Range; 4],
    pub balanced: Tri,
    /// Support points, C D H S.
    pub tp: [Range; 4],
}

impl From<&SeatKnowledge> for Sketch {
    fn from(k: &SeatKnowledge) -> Sketch {
        Sketch {
            hcp: k.hcp,
            len: k.len,
            balanced: k.balanced,
            tp: k.tp,
        }
    }
}

/// What one call showed, kept with its step (`Step::announced`), so the
/// knowledge is worked out once and the text rendered for any audience.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announced {
    /// The rule's description of the call (may be empty).
    pub description: String,
    pub before: Sketch,
    pub after: Sketch,
    pub others: [Sketch; 3],
    pub artificial: bool,
    pub forcing: Forcing,
    /// The agreed suit, C D H S index: support points are stated in it.
    pub trump: Option<usize>,
    /// The suit the call asks partner to bid when it is a transfer (C D H
    /// S index); only `Audience::Brill` states it.
    pub transfer: Option<usize>,
}

impl Default for Announced {
    fn default() -> Announced {
        let k = Sketch::from(&SeatKnowledge::default());
        Announced {
            description: String::new(),
            before: k,
            after: k,
            others: [k; 3],
            artificial: false,
            forcing: Forcing::None,
            trump: None,
            transfer: None,
        }
    }
}

impl Announced {
    pub fn new(s: &Shown) -> Announced {
        Announced {
            description: s.description.to_string(),
            before: s.before.into(),
            after: s.after.into(),
            others: s.others.map(Sketch::from),
            artificial: s.artificial,
            forcing: s.forcing,
            trump: s.trump,
            transfer: None,
        }
    }

    /// The text for `audience`.
    pub fn render(&self, audience: Audience) -> String {
        render(self, &audience.policy())
    }
}

/// The explanation for one call, for people (`Audience::Human`).
pub fn explanation(s: &Shown) -> String {
    explanation_for(s, Audience::Human)
}

/// The explanation for one call, for `audience`.
pub fn explanation_for(s: &Shown, audience: Audience) -> String {
    Announced::new(s).render(audience)
}

fn render(s: &Announced, p: &Policy) -> String {
    let mut description = words(s.description.trim());
    if p.robot {
        description = robot_words(&description);
    }
    let said = description.to_lowercase();

    let mut extras = Vec::new();
    let hcp_cap = (40 - s.others.iter().map(|k| k.hcp.lo).sum::<i32>()).min(37);
    if let Some(r) = narrowed(s.after.hcp, s.before.hcp, hcp_cap, p.hcp_floor, hcp_cap) {
        extras.push(format!("{r} HCP"));
    }
    // Support points in the agreed suit, where they show more than the HCP
    // do (the other suits' are the same count again).
    if let Some(i) = s.trump {
        let tp = s.after.tp[i];
        if tp.lo > s.after.hcp.lo {
            if let Some(r) = narrowed(tp, s.before.tp[i], i32::MAX, p.hcp_floor, i32::MAX) {
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
        (_, Tri::Unknown) if flat(&s.after) && !flat(&s.before) => Some("balanced"),
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
                    p.length_floor,
                    p.length_ceiling,
                );
                if let Some(r) = r {
                    extras.push(format!("{r} {name}"));
                }
            }
        }
    }
    if p.robot {
        if let Some((_, name)) = s.transfer.and_then(|t| SUITS.iter().find(|(i, _)| *i == t)) {
            extras.push(format!("transfer to {name}"));
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

/// A description as a robot's parser reads it (`Audience::Brill`): lengths
/// in digits ("five or more hearts" is "5+ hearts", "at most one diamonds"
/// is "0-1 diamonds", "Six hearts" is "6 hearts", "three or more" is
/// "3+"), and no negated "forcing" for a substring match to misread ("not
/// forcing" and "non-forcing" are "passable", "forcing to game" is "game
/// forcing").
fn robot_words(text: &str) -> String {
    const NUMBERS: [&str; 13] = [
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
        "twelve", "thirteen",
    ];
    const SUIT_WORDS: [(&str, &str); 8] = [
        ("spades", "spades"),
        ("spade", "spades"),
        ("hearts", "hearts"),
        ("heart", "hearts"),
        ("diamonds", "diamonds"),
        ("diamond", "diamonds"),
        ("clubs", "clubs"),
        ("club", "clubs"),
    ];
    // A token is a word and the punctuation after it ("hearts," "18:").
    fn bare(t: &str) -> &str {
        t.trim_end_matches([',', ';', ':', '.', ')'])
    }
    fn tail(t: &str) -> &str {
        &t[bare(t).len()..]
    }
    let number = |t: &str| {
        (bare(t) == t)
            .then(|| NUMBERS.iter().position(|n| n.eq_ignore_ascii_case(t)))
            .flatten()
            .map(|i| i + 1)
    };
    let suit = |t: &str| {
        SUIT_WORDS
            .iter()
            .find(|(w, _)| w.eq_ignore_ascii_case(bare(t)))
            .map(|(_, s)| *s)
    };
    let tokens: Vec<&str> = text.split(' ').collect();
    let mut out: Vec<String> = Vec::new();
    let mut k = 0;
    while k < tokens.len() {
        let at = |j: usize| tokens.get(k + j).copied().unwrap_or("");
        // "at most N <suit>"
        if at(0).eq_ignore_ascii_case("at") && at(1) == "most" {
            if let (Some(n), Some(s)) = (number(at(2)), suit(at(3))) {
                out.push(format!("0-{n} {s}{}", tail(at(3))));
                k += 4;
                continue;
            }
        }
        if let Some(n) = number(at(0)) {
            // "N or more <suit>", "N or more"
            if at(1) == "or" && bare(at(2)) == "more" {
                match suit(at(3)).filter(|_| tail(at(2)).is_empty()) {
                    Some(s) => {
                        out.push(format!("{n}+ {s}{}", tail(at(3))));
                        k += 4;
                    }
                    None => {
                        out.push(format!("{n}+{}", tail(at(2))));
                        k += 3;
                    }
                }
                continue;
            }
            // "N <suit>"
            if let Some(s) = suit(at(1)) {
                out.push(format!("{n} {s}{}", tail(at(1))));
                k += 2;
                continue;
            }
        }
        out.push(at(0).to_string());
        k += 1;
    }
    let mut text = out.join(" ");
    for (from, to) in [
        ("non-forcing", "passable"),
        ("Non-forcing", "Passable"),
        ("not forcing", "passable"),
        ("Not forcing", "Passable"),
        ("forcing to game", "game forcing"),
        ("Forcing to game", "Game forcing"),
    ] {
        text = text.replace(from, to);
    }
    text
}

/// At most 5 and at least 2 in every suit.
fn flat(k: &Sketch) -> bool {
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
    fn robot_wording() {
        assert_eq!(
            robot_words("8+ total points, five or more hearts"),
            "8+ total points, 5+ hearts"
        );
        assert_eq!(
            robot_words("Splinter: 4+ spades, game values, at most one diamonds"),
            "Splinter: 4+ spades, game values, 0-1 diamonds"
        );
        assert_eq!(
            robot_words("Four hearts, five spades, game forcing"),
            "4 hearts, 5 spades, game forcing"
        );
        assert_eq!(
            robot_words("Forcing 1NT: three or more, up to 18"),
            "Forcing 1NT: 3+, up to 18"
        );
        assert_eq!(
            robot_words("4+ clubs, 5+ hearts, not forcing"),
            "4+ clubs, 5+ hearts, passable"
        );
        assert_eq!(
            robot_words("Cue bid: 13+, no stopper, forcing to game"),
            "Cue bid: 13+, no stopper, game forcing"
        );
        // Numbers that are not lengths stay words.
        assert_eq!(
            robot_words("5+ spades: forcing one round"),
            "5+ spades: forcing one round"
        );
    }

    #[test]
    fn audiences_differ_only_where_brill_needs_it() {
        // A two-card floor is left out for both: beside a keycard ask it
        // would read as the suit shown.
        let fresh = SeatKnowledge::default();
        let after = know((10, 21), [(0, 8), (2, 8), (0, 8), (0, 8)]);
        let s = shown("Six or more ♥", &fresh, &after, [&fresh, &fresh, &fresh]);
        assert_eq!(explanation(&s), "Six or more hearts; 10-21 HCP");
        assert_eq!(explanation_for(&s, Audience::Brill), "6+ hearts; 10-21 HCP");
        // A transfer names its suit for Brill only.
        let mut a = Announced::new(&s);
        a.transfer = Some(3);
        assert_eq!(a.render(Audience::Human), "Six or more hearts; 10-21 HCP");
        assert_eq!(
            a.render(Audience::Brill),
            "6+ hearts; 10-21 HCP, transfer to spades"
        );
        assert_eq!("brill".parse::<Audience>(), Ok(Audience::Brill));
        assert_eq!("Human".parse::<Audience>(), Ok(Audience::Human));
        assert!("robot".parse::<Audience>().is_err());
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
