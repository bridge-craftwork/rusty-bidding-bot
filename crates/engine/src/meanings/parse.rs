//! A heuristic reading of an opponent's announced meaning ("5+ spades,
//! 11-21 HCP", "Game-forcing, 22+ HCP or 9+ playing tricks", "transfer to
//! hearts"): what it says about the hand, as conditions in the rule
//! language (`hcp=15..17`, `S>=5`, `balanced`) and two flags, artificial
//! and forcing. It is not a grammar: it picks out the pieces Brill's seat
//! API names as the ones that carry (rusty-seat-service docs/seat-api.txt
//! section 5): suit lengths, HCP ranges, balanced, and the words
//! *artificial*, *forcing*, *game forcing* and *transfer*. What it cannot
//! place it leaves out, so a text it does not understand says nothing.
//!
//! Soundness over coverage: a clause that asks ("asks for a 4-card
//! major"), offers a choice ("22+ HCP or 9+ playing tricks") or denies
//! ("no 4-card major", read as a maximum) is read for what it certainly
//! says and no more.

use crate::knowledge::Range;
use crate::position::Forcing;
use bridge_types::{Call, Strain};

/// What an announced meaning says. Suits by index C D H S.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Parsed {
    pub hcp: Option<Range>,
    /// Total points (`points`).
    pub points: Option<Range>,
    pub len: [Option<Range>; 4],
    pub balanced: Option<bool>,
    pub artificial: bool,
    pub forcing: Forcing,
}

const SUIT_LETTERS: [&str; 4] = ["C", "D", "H", "S"];
const MAX_HCP: i32 = 37;

impl Parsed {
    /// Nothing read.
    pub fn is_empty(&self) -> bool {
        *self == Parsed::default()
    }

    /// What the text says about the hand, as rule-language conditions
    /// (`hcp=15..17`, `S>=5`, `C<=1`, `balanced`, `!balanced`).
    pub fn conditions(&self) -> Vec<String> {
        let mut out = Vec::new();
        let range = |name: &str, r: Range, max: i32, out: &mut Vec<String>| {
            if r.lo == r.hi {
                out.push(format!("{name}={}", r.lo));
            } else if r.lo <= 0 && r.hi < max {
                out.push(format!("{name}<={}", r.hi));
            } else if r.hi >= max && r.lo > 0 {
                out.push(format!("{name}>={}", r.lo));
            } else if r.lo > 0 || r.hi < max {
                out.push(format!("{name}={}..{}", r.lo, r.hi));
            }
        };
        if let Some(r) = self.hcp {
            range("hcp", r, MAX_HCP, &mut out);
        }
        if let Some(r) = self.points {
            range("points", r, MAX_HCP + 13, &mut out);
        }
        for s in (0..4).rev() {
            if let Some(r) = self.len[s] {
                range(SUIT_LETTERS[s], r, 13, &mut out);
            }
        }
        match self.balanced {
            Some(true) => out.push("balanced".into()),
            Some(false) => out.push("!balanced".into()),
            None => {}
        }
        out
    }

    /// The conditions and flags in one line, for a trace.
    pub fn summary(&self) -> String {
        let mut parts = self.conditions();
        if self.artificial {
            parts.push("artificial".into());
        }
        match self.forcing {
            Forcing::Round => parts.push("forcing".into()),
            Forcing::Game => parts.push("game forcing".into()),
            Forcing::None => {}
        }
        parts.join(", ")
    }

    fn narrow(slot: &mut Option<Range>, r: Range, conflict: &mut bool) {
        let merged = match *slot {
            Some(old) => old.intersect(r),
            None => r,
        };
        if merged.is_empty() {
            *conflict = true;
        } else {
            *slot = Some(merged);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(i32),
    Plus,
    Dash,
    Slash,
    Le,
    Lt,
    Ge,
    Gt,
    Word(String),
}

fn number_word(w: &str) -> Option<i32> {
    Some(match w {
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        _ => return None,
    })
}

/// A suit named by a word (`spades`, `spade`, a symbol written out), by
/// index C D H S.
fn suit_word(w: &str) -> Option<usize> {
    Some(match w {
        "clubs" | "club" | "♣" => 0,
        "diamonds" | "diamond" | "♦" => 1,
        "hearts" | "heart" | "♥" => 2,
        "spades" | "spade" | "♠" => 3,
        _ => return None,
    })
}

fn suit_letter(w: &str) -> Option<usize> {
    Some(match w {
        "c" => 0,
        "d" => 1,
        "h" => 2,
        "s" => 3,
        _ => return None,
    })
}

/// Lower case, the symbols and BBO's `!S` markup as suit words, dashes as
/// `-`, then split into clauses.
fn clauses(text: &str) -> Vec<String> {
    let mut s = String::with_capacity(text.len() + 16);
    let mut chars = text.chars().peekable();
    // A suit symbol right after a digit is a bid (`2♠` is 2S); anywhere
    // else (`5+♠`, `5+ ♠`) it is the suit.
    // (`4-5♥` is a length: a range ends there.)
    let suit = |s: &mut String, letter: &str, name: &str| {
        let head = s.trim_end_matches(|c: char| c.is_ascii_digit());
        let bid =
            head.len() < s.len() && s.len() - head.len() == 1 && !head.ends_with(['-', '/', '+']);
        if bid {
            s.push_str(letter);
            s.push(' ');
        } else {
            s.push(' ');
            s.push_str(name);
            s.push(' ');
        }
    };
    while let Some(c) = chars.next() {
        match c {
            '♠' | '♤' => suit(&mut s, "s", "spades"),
            '♥' | '♡' => suit(&mut s, "h", "hearts"),
            '♦' | '♢' => suit(&mut s, "d", "diamonds"),
            '♣' | '♧' => suit(&mut s, "c", "clubs"),
            '!' => match chars.peek().map(|c| c.to_ascii_lowercase()) {
                Some(l @ ('s' | 'h' | 'd' | 'c'))
                    if !chars.clone().nth(1).is_some_and(|n| n.is_alphabetic()) =>
                {
                    chars.next();
                    let (letter, name) = match l {
                        's' => ("s", "spades"),
                        'h' => ("h", "hearts"),
                        'd' => ("d", "diamonds"),
                        _ => ("c", "clubs"),
                    };
                    suit(&mut s, letter, name);
                }
                _ => s.push(' '),
            },
            '–' | '—' | '‒' | '−' => s.push('-'),
            '≥' => s.push_str(" >= "),
            '≤' => s.push_str(" <= "),
            '&' => s.push_str(" and "),
            c => s.extend(c.to_lowercase()),
        }
    }
    // "singleton or void" is one thing, not a choice.
    let s = s
        .replace("singleton or void", "short")
        .replace("void or singleton", "short")
        .replace("semi-balanced", "semibalanced")
        .replace("semi balanced", "semibalanced")
        .replace("semi-forcing", "semiforcing")
        .replace("non-forcing", "nonforcing")
        .replace("not forcing", "nonforcing")
        .replace("game-forcing", "game forcing")
        .replace("game-force", "game forcing")
        .replace("forcing to game", "game forcing");
    s.split([';', ',', '.', ':', '(', ')', '[', ']', '\n'])
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(str::to_string)
        .collect()
}

fn tokens(clause: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let chars: Vec<char> = clause.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_digit() {
            let mut n = 0i32;
            while i < chars.len() && chars[i].is_ascii_digit() {
                n = n
                    .saturating_mul(10)
                    .saturating_add(chars[i] as i32 - '0' as i32);
                i += 1;
            }
            out.push(Tok::Num(n));
            continue;
        }
        if c.is_alphabetic() {
            let mut w = String::new();
            while i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '\'') {
                w.push(chars[i]);
                i += 1;
            }
            match number_word(&w) {
                Some(n) => out.push(Tok::Num(n)),
                None => out.push(Tok::Word(w)),
            }
            continue;
        }
        let next = chars.get(i + 1).copied();
        match (c, next) {
            ('+', _) => out.push(Tok::Plus),
            ('-', _) => out.push(Tok::Dash),
            ('/', _) => out.push(Tok::Slash),
            ('<', Some('=')) => {
                out.push(Tok::Le);
                i += 1;
            }
            ('>', Some('=')) => {
                out.push(Tok::Ge);
                i += 1;
            }
            ('<', _) => out.push(Tok::Lt),
            ('>', _) => out.push(Tok::Gt),
            _ => {}
        }
        i += 1;
    }
    out
}

fn word<'a>(t: Option<&'a Tok>) -> Option<&'a str> {
    match t {
        Some(Tok::Word(w)) => Some(w.as_str()),
        _ => None,
    }
}

/// What a quantity is of.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Unit {
    Hcp,
    Points,
    Suit(usize),
    Majors,
    Minors,
    /// Something the text names that we do not read (keycards, tricks,
    /// support points, "a suit").
    Other,
    /// Nothing named: a bare range.
    None,
}

/// A quantity at `toks[i..]`: its range, whether it was written with a
/// `+` or a range (so a suit letter after it is a length, not a bid), and
/// where it ends.
fn quantity(toks: &[Tok], i: usize) -> Option<(Range, bool, usize)> {
    let w = |k: usize| word(toks.get(k));
    let n = |k: usize| match toks.get(k) {
        Some(Tok::Num(v)) => Some(*v),
        _ => None,
    };
    // Prefixed bounds: max 9, at most 9, up to 9, <= 9, less than 10.
    let prefixed = match (toks.get(i), w(i), w(i + 1)) {
        (Some(Tok::Le), _, _) => Some((1, false, false)),
        (Some(Tok::Lt), _, _) => Some((1, false, true)),
        (Some(Tok::Ge), _, _) => Some((1, true, false)),
        (Some(Tok::Gt), _, _) => Some((1, true, true)),
        (_, Some("max" | "maximum"), _) => Some((1, false, false)),
        (_, Some("min" | "minimum"), _) => Some((1, true, false)),
        (_, Some("at"), Some("most")) => Some((2, false, false)),
        (_, Some("at"), Some("least")) => Some((2, true, false)),
        (_, Some("up"), Some("to")) => Some((2, false, false)),
        (_, Some("less" | "fewer"), Some("than")) => Some((2, false, true)),
        (_, Some("more"), Some("than")) => Some((2, true, true)),
        _ => None,
    };
    if let Some((skip, floor, strict)) = prefixed {
        let v = n(i + skip)?;
        let r = match (floor, strict) {
            (true, false) => Range::new(v, i32::MAX),
            (true, true) => Range::new(v + 1, i32::MAX),
            (false, false) => Range::new(0, v),
            (false, true) => Range::new(0, v - 1),
        };
        return Some((r, true, i + skip + 1));
    }
    let a = n(i)?;
    match (toks.get(i + 1), n(i + 2)) {
        (Some(Tok::Plus), _) => Some((Range::new(a, i32::MAX), true, i + 2)),
        (Some(Tok::Dash | Tok::Slash), Some(b)) if b >= a => Some((Range::new(a, b), true, i + 3)),
        (Some(Tok::Word(o)), Some(b)) if (o == "to" || o == "or") && b >= a => {
            Some((Range::new(a, b), true, i + 3))
        }
        (Some(Tok::Word(o)), _) if o == "or" => match (w(i + 2), w(i + 3)) {
            (Some("more" | "longer" | "better"), _) => Some((Range::new(a, i32::MAX), true, i + 3)),
            (Some("less" | "fewer"), _) => Some((Range::new(0, a), true, i + 3)),
            _ => Some((Range::point(a), false, i + 1)),
        },
        _ => Some((Range::point(a), false, i + 1)),
    }
}

/// The unit after a quantity, from `toks[j..]`, and where it ends.
fn unit(toks: &[Tok], mut j: usize, written_as_length: bool) -> (Unit, usize) {
    let w = |k: usize| word(toks.get(k));
    // "4-card", "5 cards", "6+ card suit": the length's own word.
    let mut card = false;
    if toks.get(j) == Some(&Tok::Dash) && matches!(w(j + 1), Some("card" | "cards")) {
        j += 2;
        card = true;
    } else if matches!(w(j), Some("card" | "cards")) {
        j += 1;
        card = true;
    }
    while matches!(w(j), Some("in" | "of" | "the" | "a")) {
        j += 1;
    }
    match w(j) {
        Some("hcp" | "hp" | "hcps") => (Unit::Hcp, j + 1),
        Some("high") if matches!(w(j + 1), Some("card" | "cards")) => (Unit::Hcp, j + 3),
        Some("total" | "tp") => (Unit::Points, j + 1),
        Some("support" | "dummy") => (Unit::Other, j + 1),
        // "15-17p": the European shorthand counts high cards.
        Some("p") if !card => (Unit::Hcp, j + 1),
        Some("points" | "pts" | "pt" | "point") if !card => (Unit::Points, j + 1),
        Some("majors" | "major") if w(j) == Some("majors") => (Unit::Majors, j + 1),
        Some("minors") => (Unit::Minors, j + 1),
        Some(x) => {
            if let Some(s) = suit_word(x) {
                return (Unit::Suit(s), j + 1);
            }
            if let Some(s) = suit_letter(x) {
                // "5+S" is a length; "4C" alone is a bid.
                if written_as_length || card {
                    return (Unit::Suit(s), j + 1);
                }
                return (Unit::Other, j + 1);
            }
            if card
                || matches!(
                    x,
                    "keycards"
                        | "keycard"
                        | "aces"
                        | "ace"
                        | "kings"
                        | "tricks"
                        | "playing"
                        | "losers"
                        | "controls"
                        | "suit"
                        | "suits"
                        | "major"
                        | "minor"
                )
            {
                (Unit::Other, j + 1)
            } else {
                (Unit::None, j)
            }
        }
        None => (if card { Unit::Other } else { Unit::None }, j),
    }
}

/// Words in a clause that make it a question, not a description.
fn asks(words: &[&str]) -> bool {
    words.iter().any(|w| {
        matches!(
            *w,
            "ask"
                | "asks"
                | "asking"
                | "inquiry"
                | "enquiry"
                | "inquires"
                | "enquires"
                | "relay"
                | "puppet"
                | "looking"
                | "seeks"
                | "seeking"
        )
    })
}

fn negates(words: &[&str]) -> bool {
    words.iter().any(|w| {
        matches!(
            *w,
            "no" | "not" | "denies" | "deny" | "denying" | "without" | "never" | "lacks"
        )
    })
}

/// The suit after "transfer to" (`hearts`, `2h`, `2♥` written out): by
/// index C D H S.
fn transfer_target(toks: &[Tok]) -> Option<usize> {
    let at = toks.iter().enumerate().position(|(k, t)| {
        matches!(t, Tok::Word(w) if w == "transfer" || w == "transfers")
            && word(toks.get(k + 1)) == Some("to")
    })?;
    let mut j = at + 2;
    while matches!(word(toks.get(j)), Some("the" | "a")) {
        j += 1;
    }
    if let Some(Tok::Num(l)) = toks.get(j) {
        if (1..=7).contains(l) {
            j += 1;
        }
    }
    let w = word(toks.get(j))?;
    suit_word(w).or_else(|| suit_letter(w))
}

/// The suit a transfer bid shows when the text does not say: the next
/// suit up (2♦ for hearts, 2♥ for spades, 2♣ for diamonds). A spade or
/// notrump transfer is a minor transfer, whose target depends on the
/// method.
fn transfer_from_call(call: Option<&Call>) -> Option<usize> {
    match call {
        Some(Call::Bid { strain, .. }) => match strain {
            Strain::Clubs => Some(1),
            Strain::Diamonds => Some(2),
            Strain::Hearts => Some(3),
            _ => None,
        },
        _ => None,
    }
}

/// Read an announced meaning. `call` is the call it explains, when known:
/// it places a transfer whose text does not name the suit.
pub fn parse(text: &str, call: Option<&Call>) -> Parsed {
    let mut p = Parsed::default();
    let mut conflict = [false; 6];
    let mut explicit_len = [false; 4];
    let mut transfer: Option<usize> = None;
    let mut natural = false;
    let clauses = clauses(text);
    // "2-3 hearts, or a minimum 3=4=3=3": a clause that starts with `or`
    // is the other half of a choice, and so is the one before it.
    let starts_or = |c: &str| c == "or" || c.starts_with("or ");
    let uncertain: Vec<bool> = (0..clauses.len())
        .map(|k| starts_or(&clauses[k]) || clauses.get(k + 1).is_some_and(|n| starts_or(n)))
        .collect();
    for (ci, clause) in clauses.iter().enumerate() {
        let toks = tokens(clause);
        let words: Vec<&str> = toks
            .iter()
            .filter_map(|t| match t {
                Tok::Word(w) => Some(w.as_str()),
                _ => None,
            })
            .collect();
        let has = |w: &str| words.contains(&w);
        // Completing or accepting a transfer is not one.
        let completes = words.iter().any(|w| {
            matches!(
                *w,
                "completes"
                    | "complete"
                    | "completing"
                    | "completion"
                    | "accepts"
                    | "accept"
                    | "accepting"
                    | "acceptance"
                    | "accepted"
                    | "completed"
            )
        });
        // Flags.
        if !completes
            && words.iter().any(|w| {
                matches!(
                    *w,
                    "artificial"
                        | "conventional"
                        | "transfer"
                        | "transfers"
                        | "relay"
                        | "puppet"
                        | "ask"
                        | "asks"
                        | "asking"
                        | "inquiry"
                        | "enquiry"
                        | "waiting"
                        | "cue"
                        | "cuebid"
                        | "stayman"
                        | "blackwood"
                        | "keycard"
                        | "rkcb"
                        | "gerber"
                )
            })
            && !has("natural")
            && !(has("not") && has("artificial"))
            && !has("nonartificial")
        {
            p.artificial = true;
        }
        if has("natural") {
            natural = true;
        }
        let game = words.windows(2).any(|w| w == ["game", "forcing"]) || has("gf");
        if game {
            p.forcing = Forcing::Game;
        } else if (has("forcing") || has("f1")) && p.forcing == Forcing::None {
            p.forcing = Forcing::Round;
        }
        if (has("transfer") || has("transfers")) && !completes {
            if let Some(t) = transfer_target(&toks) {
                transfer = Some(t);
            } else if transfer.is_none() {
                transfer = transfer_from_call(call);
            }
        }
        // Shape words.
        if has("balanced") && !negates(&words) {
            p.balanced = Some(true);
        }
        if has("unbalanced")
            || (has("balanced") && negates(&words))
            || words.iter().any(|w| {
                matches!(
                    *w,
                    "suited" | "suiter" | "onesuited" | "twosuited" | "threesuited"
                )
            })
        {
            p.balanced = Some(false);
        }
        let asking = asks(&words);
        let negated = negates(&words);
        // A choice inside the clause ("22+ HCP or 9+ playing tricks"):
        // neither half is certain. "4 or 5 hearts" is a range, read
        // below as one quantity, so `or` counts only between quantities.
        if asking || uncertain[ci] || or_between_quantities(&toks) {
            continue;
        }
        // Shortness: "short in spades", "singleton heart", "void in clubs".
        for (k, t) in toks.iter().enumerate() {
            if let Tok::Word(w) = t {
                let short = matches!(
                    w.as_str(),
                    "short" | "shortness" | "singleton" | "splinter" | "void"
                );
                if !short || negated {
                    continue;
                }
                let mut j = k + 1;
                while matches!(word(toks.get(j)), Some("in" | "a" | "the")) {
                    j += 1;
                }
                if let Some(s) = word(toks.get(j)).and_then(suit_word) {
                    let r = if w == "void" {
                        Range::point(0)
                    } else {
                        Range::new(0, 1)
                    };
                    Parsed::narrow(&mut p.len[s], r, &mut conflict[2 + s]);
                    explicit_len[s] = true;
                }
            }
        }
        // "both majors", "the minors" with no number: four or more in each.
        let pair_unnumbered =
            |name: &str| words.contains(&name) && !toks.iter().any(|t| matches!(t, Tok::Num(_)));
        if !negated {
            for (name, suits) in [("majors", [2, 3]), ("minors", [0, 1])] {
                if pair_unnumbered(name) {
                    for s in suits {
                        Parsed::narrow(&mut p.len[s], Range::new(4, 13), &mut conflict[2 + s]);
                        explicit_len[s] = true;
                    }
                }
            }
        }
        // Quantities.
        let mut i = 0;
        while i < toks.len() {
            let Some((r, as_length, end)) = quantity(&toks, i) else {
                i += 1;
                continue;
            };
            // "5-5 majors", "5-4 in the minors": the shorter of the two
            // in each.
            let shape = match (toks.get(i + 1), toks.get(i + 2)) {
                (Some(Tok::Dash), Some(Tok::Num(b))) => Some(r.lo.min(*b).max(0)),
                _ => None,
            };
            let (u, after) = unit(&toks, end, as_length);
            match u {
                Unit::Hcp if !negated => {
                    let r = Range::new(r.lo.max(0), r.hi.min(MAX_HCP));
                    Parsed::narrow(&mut p.hcp, r, &mut conflict[0]);
                }
                Unit::Points if !negated => {
                    let r = Range::new(r.lo.max(0), r.hi.min(MAX_HCP + 13));
                    Parsed::narrow(&mut p.points, r, &mut conflict[1]);
                }
                Unit::Suit(s) => {
                    // A plain count ("five spades", "a 5-card suit") is a
                    // floor, as bridge prose means it, unless "exactly".
                    let exactly = i > 0 && word(toks.get(i - 1)) == Some("exactly");
                    let r = if !as_length && r.lo == r.hi && !exactly && !negated {
                        Range::new(r.lo, 13)
                    } else {
                        r
                    };
                    let r = if negated {
                        // "no 4-card spade suit", "denies 4 hearts".
                        (r.lo >= 1).then(|| Range::new(0, r.lo - 1))
                    } else {
                        Some(Range::new(r.lo.max(0), r.hi.min(13)))
                    };
                    if let Some(r) = r {
                        Parsed::narrow(&mut p.len[s], r, &mut conflict[2 + s]);
                        explicit_len[s] = true;
                    }
                }
                Unit::Majors | Unit::Minors => {
                    let suits = if u == Unit::Majors { [2, 3] } else { [0, 1] };
                    let r = if negated {
                        (r.lo >= 1).then(|| Range::new(0, r.lo - 1))
                    } else {
                        let lo = shape.unwrap_or(r.lo).max(0);
                        Some(Range::new(lo, 13))
                    };
                    if let Some(r) = r {
                        for s in suits {
                            Parsed::narrow(&mut p.len[s], r, &mut conflict[2 + s]);
                            explicit_len[s] = true;
                        }
                    }
                }
                Unit::None if !negated => {
                    // A bare range: HCP if it reads as one ("15-17"), the
                    // clause names no other unit, and no length follows.
                    let bare = toks.get(i + 1) == Some(&Tok::Dash)
                        && r.lo >= 6
                        && r.hi <= MAX_HCP
                        && r.hi > r.lo
                        && r.hi - r.lo <= 12
                        && !words.iter().any(|w| {
                            suit_word(w).is_some()
                                || matches!(
                                    *w,
                                    "hcp"
                                        | "points"
                                        | "pts"
                                        | "card"
                                        | "cards"
                                        | "majors"
                                        | "minors"
                                )
                        });
                    if bare {
                        Parsed::narrow(&mut p.hcp, r, &mut conflict[0]);
                    }
                }
                _ => {}
            }
            i = after.max(end).max(i + 1);
        }
    }
    if let Some(t) = transfer {
        if !explicit_len[t] && !natural {
            Parsed::narrow(&mut p.len[t], Range::new(5, 13), &mut conflict[2 + t]);
        }
        if !natural {
            p.artificial = true;
        }
    }
    // A part read two ways that cannot both hold is not read at all.
    if conflict[0] {
        p.hcp = None;
    }
    if conflict[1] {
        p.points = None;
    }
    for s in 0..4 {
        if conflict[2 + s] {
            p.len[s] = None;
        }
    }
    p
}

/// `or` between two quantities with their units ("22+ HCP or 9+ playing
/// tricks", "5+ hearts or 6+ clubs"): a choice.
fn or_between_quantities(toks: &[Tok]) -> bool {
    toks.iter().enumerate().any(|(k, t)| {
        *t == Tok::Word("or".into())
            && k > 0
            && !matches!(toks[k - 1], Tok::Num(_))
            && toks[..k].iter().any(|t| matches!(t, Tok::Num(_)))
            && toks[k + 1..].iter().any(|t| matches!(t, Tok::Num(_)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conds(text: &str) -> Vec<String> {
        parse(text, None).conditions()
    }

    fn c(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn brill_examples() {
        // seat-api.txt section 3 and 5.
        assert_eq!(conds("5+ spades, 12-21 HCP"), c(&["hcp=12..21", "S>=5"]));
        let p = parse("Game-forcing, 22+ HCP or 9+ playing tricks", None);
        assert_eq!(p.forcing, Forcing::Game);
        assert!(p.hcp.is_none(), "a choice is not certain: {p:?}");
        // Support with no suit named says nothing we can place.
        assert_eq!(conds("3-card support, 6-9 HCP"), c(&["hcp=6..9"]));
    }

    #[test]
    fn hcp_forms() {
        assert_eq!(conds("15-17 HCP"), c(&["hcp=15..17"]));
        assert_eq!(conds("15-17p"), c(&["hcp=15..17"]));
        assert_eq!(conds("15–17 hcp"), c(&["hcp=15..17"]));
        assert_eq!(conds("15 to 17 HCP"), c(&["hcp=15..17"]));
        assert_eq!(conds("11+ HCP"), c(&["hcp>=11"]));
        assert_eq!(conds("11+hcp"), c(&["hcp>=11"]));
        assert_eq!(conds("max 9 HCP"), c(&["hcp<=9"]));
        assert_eq!(conds("at most 9 HCP"), c(&["hcp<=9"]));
        assert_eq!(conds("less than 10 HCP"), c(&["hcp<=9"]));
        assert_eq!(conds("at least 16 high card points"), c(&["hcp>=16"]));
        assert_eq!(conds("≥16 HCP"), c(&["hcp>=16"]));
        assert_eq!(conds("0-14 HCP"), c(&["hcp<=14"]));
        assert_eq!(conds("12-22 total points"), c(&["points=12..22"]));
        assert_eq!(conds("6-9 points"), c(&["points=6..9"]));
        // A bare range is the strength when nothing else is named.
        assert_eq!(conds("15-17"), c(&["hcp=15..17"]));
        assert_eq!(conds("Weak: 10-12"), c(&["hcp=10..12"]));
    }

    #[test]
    fn lengths() {
        assert_eq!(conds("5+ spades"), c(&["S>=5"]));
        assert_eq!(conds("5+♠"), c(&["S>=5"]));
        assert_eq!(conds("5+ ♥"), c(&["H>=5"]));
        assert_eq!(conds("5+!S"), c(&["S>=5"]));
        assert_eq!(conds("5+S"), c(&["S>=5"]));
        assert_eq!(conds("4 hearts"), c(&["H>=4"]));
        assert_eq!(conds("4+ hearts"), c(&["H>=4"]));
        assert_eq!(conds("4-5 hearts"), c(&["H=4..5"]));
        assert_eq!(conds("4 or 5 hearts"), c(&["H=4..5"]));
        assert_eq!(conds("5-card spade suit"), c(&["S>=5"]));
        assert_eq!(conds("6+ card club suit"), c(&["C>=6"]));
        assert_eq!(conds("at least 4 diamonds"), c(&["D>=4"]));
        assert_eq!(conds("five spades"), c(&["S>=5"]));
        assert_eq!(conds("exactly 4 hearts"), c(&["H=4"]));
        assert_eq!(conds("0-1 clubs"), c(&["C<=1"]));
        assert_eq!(conds("5+ in hearts"), c(&["H>=5"]));
        // A bid is not a length.
        assert_eq!(conds("Cue bid 4C shows club control"), Vec::<String>::new());
        assert!(conds("6+ card suit").is_empty());
    }

    #[test]
    fn two_suiters() {
        assert_eq!(conds("both majors"), c(&["S>=4", "H>=4"]));
        assert_eq!(conds("4-5♥"), c(&["H=4..5"]));
        assert_eq!(conds("5-5 in the majors"), c(&["S>=5", "H>=5"]));
        assert_eq!(conds("5-4 majors"), c(&["S>=4", "H>=4"]));
        assert_eq!(conds("both minors, 5-5 or longer"), c(&["D>=4", "C>=4"]));
        assert_eq!(conds("5+ hearts and 4+ clubs"), c(&["H>=5", "C>=4"]));
    }

    #[test]
    fn denials() {
        assert_eq!(conds("no 4-card major"), Vec::<String>::new());
        assert_eq!(conds("denies 4 spades"), c(&["S<=3"]));
        assert_eq!(conds("no 5-card major"), Vec::<String>::new());
        assert_eq!(conds("no 4-card majors"), c(&["S<=3", "H<=3"]));
        assert_eq!(conds("not balanced"), c(&["!balanced"]));
    }

    #[test]
    fn shortness() {
        assert_eq!(conds("singleton or void in spades"), c(&["S<=1"]));
        assert_eq!(conds("short in hearts"), c(&["H<=1"]));
        assert_eq!(conds("Splinter: shortness in ♦"), c(&["D<=1"]));
        assert_eq!(conds("void in clubs"), c(&["C=0"]));
    }

    #[test]
    fn shape_words() {
        assert_eq!(conds("balanced"), c(&["balanced"]));
        assert_eq!(conds("semi-balanced"), Vec::<String>::new());
        assert_eq!(conds("unbalanced"), c(&["!balanced"]));
        assert_eq!(conds("one-suited"), c(&["!balanced"]));
        assert_eq!(conds("two suited, 5-5"), c(&["!balanced"]));
    }

    #[test]
    fn flags() {
        let p = parse(
            "Stayman: asks for a 4-card major; artificial, forcing",
            None,
        );
        assert!(p.artificial);
        assert_eq!(p.forcing, Forcing::Round);
        assert!(p.conditions().is_empty(), "the ask shows nothing: {p:?}");
        assert_eq!(parse("Game forcing", None).forcing, Forcing::Game);
        assert_eq!(parse("GF", None).forcing, Forcing::Game);
        assert_eq!(parse("forcing to game", None).forcing, Forcing::Game);
        assert_eq!(parse("non-forcing", None).forcing, Forcing::None);
        assert_eq!(parse("not forcing", None).forcing, Forcing::None);
        assert_eq!(parse("semi-forcing", None).forcing, Forcing::None);
        assert!(!parse("natural, to play", None).artificial);
        assert!(parse("Relay", None).artificial);
        assert!(parse("cue bid, limit raise or better", None).artificial);
    }

    #[test]
    fn choices_between_clauses() {
        // Our 1NT-2D-2H: "2-3 hearts, or a minimum 3=4=3=3".
        let p = parse(
            "Completes transfer: 2-3 hearts, or a minimum 3=4=3=3; 2-4 hearts",
            Some(&Call::Bid {
                level: 2,
                strain: Strain::Hearts,
            }),
        );
        assert_eq!(p.conditions(), c(&["H=2..4"]));
        assert!(!p.artificial, "completing a transfer is not one");
    }

    #[test]
    fn transfers() {
        let p = parse("Transfer to hearts", None);
        assert!(p.artificial);
        assert_eq!(p.conditions(), c(&["H>=5"]));
        assert_eq!(parse("transfer to 2♠", None).conditions(), c(&["S>=5"]));
        assert_eq!(
            parse("Jacoby transfer to 2H", None).conditions(),
            c(&["H>=5"])
        );
        assert_eq!(
            parse("transfer to spades, 6+ spades", None).conditions(),
            c(&["S>=6"])
        );
        // The call places a transfer the text does not.
        let two_d = Call::Bid {
            level: 2,
            strain: Strain::Diamonds,
        };
        assert_eq!(parse("Transfer", Some(&two_d)).conditions(), c(&["H>=5"]));
        let two_s = Call::Bid {
            level: 2,
            strain: Strain::Spades,
        };
        assert!(parse("Transfer", Some(&two_s)).conditions().is_empty());
    }

    #[test]
    fn convention_headlines_say_nothing_alone() {
        // Brill's `means` is often only the name: that is the name table's
        // job (meanings::AnnouncedNames), not the parser's.
        assert!(parse("Cappelletti", None).is_empty());
        assert!(parse("Splinter", None).conditions().is_empty());
        assert!(parse("", None).is_empty());
        assert!(parse("Natural", None).is_empty());
    }

    #[test]
    fn ben_and_gib_styles() {
        // GIB on BBO: semicolons, symbols, a total-points line.
        assert_eq!(
            conds("5+ ♠; 11-21 HCP; 12-22 total points"),
            c(&["hcp=11..21", "points=12..22", "S>=5"])
        );
        // BBO markup and the long range form.
        assert_eq!(conds("6+!H 5 to 10 HCP"), c(&["hcp=5..10", "H>=6"]));
        // Cappelletti 2C as other robots write it.
        let p = parse("Cappelletti: one long suit, 6+ cards", None);
        assert!(p.conditions().is_empty(), "{p:?}");
        let p = parse("Michaels: 5+ hearts and 5+ spades", None);
        assert_eq!(p.conditions(), c(&["S>=5", "H>=5"]));
    }

    #[test]
    fn a_contradiction_drops_the_part() {
        assert_eq!(conds("15-17 HCP; 20+ HCP"), Vec::<String>::new());
    }
}
