//! Test cases written next to the rules they test (`<module>.test`).
//!
//! ```text
//! # Settings apply to the lines below them and can change part way.
//! card    21GF-DEFAULT        # a card name (looked up in the cards
//!                             # directory) or a path to .bbsa / card JSON
//! dealer  S
//! vul     None
//! scoring MP
//!
//! # seat hand              | auction           | expect | why
//! N 82.QJ973.K94.J83       | 1NT P             | 2D     | five hearts, weak: transfer
//! S AK5.KJ7.Q942.K83       | 1NT P 2D P        | !P     | the transfer is forcing
//! N 8.KQ9732.K94.Q83       | 1NT P             | 4D/2D  | either route is fine
//! ```
//!
//! The seat must be the one to call after the auction. `expect` is a call,
//! `!call` (anything but), or alternatives separated by `/`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bridge_card::{bbsa, Card};
use bridge_types::{Call, Direction, Hand, ScoringMethod, Vulnerability};
use serde::Serialize;

use crate::Engine;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expect {
    AnyOf(Vec<Call>),
    Not(Call),
}

impl Expect {
    pub fn accepts(&self, call: &Call) -> bool {
        match self {
            Expect::AnyOf(calls) => calls.contains(call),
            Expect::Not(c) => c != call,
        }
    }
}

impl std::fmt::Display for Expect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expect::AnyOf(calls) => {
                let v: Vec<String> = calls.iter().map(|c| c.to_pbn()).collect();
                write!(f, "{}", v.join("/"))
            }
            Expect::Not(c) => write!(f, "anything but {}", c.to_pbn()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Case {
    pub file: String,
    pub line: usize,
    pub card: String,
    pub dealer: Direction,
    pub vul: Vulnerability,
    pub scoring: ScoringMethod,
    pub seat: Direction,
    pub hand: String,
    pub auction: Vec<Call>,
    pub expect: Expect,
    pub why: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub case: Case,
    pub got: Call,
    pub explanation: String,
    pub passed: bool,
    /// Candidates, one per line, for a failure report.
    pub trace: String,
}

impl Outcome {
    /// One line: `file:line: seat hand | auction | expected X, got Y (meaning)`.
    pub fn summary(&self) -> String {
        let c = &self.case;
        let auction: Vec<String> = c.auction.iter().map(|c| c.to_pbn()).collect();
        format!(
            "{}:{}: {} {} | {} | expected {}, got {} ({})",
            c.file,
            c.line,
            c.seat.to_char(),
            c.hand,
            auction.join(" "),
            c.expect,
            self.got.to_pbn(),
            self.explanation
        )
    }

    /// The summary, the case's `why`, and for a failure the candidates.
    pub fn report(&self) -> String {
        let mut s = self.summary();
        if !self.case.why.is_empty() {
            s += &format!("\n    why: {}", self.case.why);
        }
        if !self.passed {
            s += "\n";
            s += &self.trace;
        }
        s
    }
}

fn parse_calls(s: &str) -> Result<Vec<Call>, String> {
    s.split_whitespace()
        .map(|c| Call::from_pbn(c).ok_or_else(|| format!("bad call {c:?}")))
        .collect()
}

fn parse_expect(s: &str) -> Result<Expect, String> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('!') {
        return Ok(Expect::Not(
            parse_calls(rest)?
                .into_iter()
                .next()
                .ok_or("expected a call after !")?,
        ));
    }
    let calls = s
        .split('/')
        .map(|c| Call::from_pbn(c.trim()).ok_or_else(|| format!("bad call {c:?}")))
        .collect::<Result<Vec<_>, _>>()?;
    if calls.is_empty() {
        return Err("expected a call".into());
    }
    Ok(Expect::AnyOf(calls))
}

/// Parse a `.test` file.
pub fn parse(text: &str, file: &str) -> Result<Vec<Case>, Vec<String>> {
    let mut cases = Vec::new();
    let mut errors = Vec::new();
    let mut card = "21GF-DEFAULT".to_string();
    let mut dealer = Direction::South;
    let mut vul = Vulnerability::None;
    let mut scoring = ScoringMethod::Matchpoints;
    for (n, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut err = |m: String| errors.push(format!("{file}:{}: {m}", n + 1));
        if !line.contains('|') {
            let (key, value) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
            let value = value.trim();
            match key {
                "card" => card = value.to_string(),
                "dealer" => match value.chars().next().and_then(|c| Direction::from_char(c.to_ascii_uppercase())) {
                    Some(d) => dealer = d,
                    None => err(format!("dealer {value:?}: expected N, E, S or W")),
                },
                "vul" => match Vulnerability::from_pbn(value) {
                    Some(v) => vul = v,
                    None => err(format!("vul {value:?}: expected None, NS, EW or All")),
                },
                "scoring" => match ScoringMethod::from_pbn(value) {
                    Some(s) => scoring = s,
                    None => err(format!("scoring {value:?}: expected MP or IMP")),
                },
                _ => err(format!("expected a setting (card, dealer, vul, scoring) or a case `seat hand | auction | expect | why`, found {key:?}")),
            }
            continue;
        }
        let cols: Vec<&str> = line.split('|').map(str::trim).collect();
        if cols.len() < 3 {
            err("a case needs `seat hand | auction | expect` and an optional `| why`".into());
            continue;
        }
        let (seat_hand, auction, expect) = (cols[0], cols[1], cols[2]);
        let why = cols.get(3).copied().unwrap_or("").to_string();
        let Some((seat, hand)) = seat_hand.split_once(char::is_whitespace) else {
            err("expected `seat hand`, e.g. `N 82.QJ973.K94.J83`".into());
            continue;
        };
        let Some(seat) = seat
            .chars()
            .next()
            .and_then(|c| Direction::from_char(c.to_ascii_uppercase()))
        else {
            err(format!("seat {seat:?}: expected N, E, S or W"));
            continue;
        };
        let hand = hand.trim();
        if Hand::from_pbn(hand).is_none_or(|h| h.len() != 13) {
            err(format!("hand {hand:?}: expected 13 cards as S.H.D.C"));
            continue;
        }
        let auction = match parse_calls(auction) {
            Ok(a) => a,
            Err(e) => {
                err(e);
                continue;
            }
        };
        let next = (0..auction.len()).fold(dealer, |d, _| d.next());
        if next != seat {
            err(format!(
                "after this auction with dealer {} it is {}'s turn, not {}'s",
                dealer.to_char(),
                next.to_char(),
                seat.to_char()
            ));
            continue;
        }
        let expect = match parse_expect(expect) {
            Ok(e) => e,
            Err(e) => {
                err(e);
                continue;
            }
        };
        cases.push(Case {
            file: file.to_string(),
            line: n + 1,
            card: card.clone(),
            dealer,
            vul,
            scoring,
            seat,
            hand: hand.to_string(),
            auction,
            expect,
            why,
        });
    }
    if errors.is_empty() {
        Ok(cases)
    } else {
        Err(errors)
    }
}

/// Every `.test` file under `dir`, sorted.
pub fn find(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        if dir.is_file() {
            out.push(dir.to_path_buf());
            return;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "test") {
                    out.push(p);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

fn load_card(spec: &str, test_dir: &Path, cards_dir: &Path) -> Result<Card, String> {
    let candidates = [
        test_dir.join(spec),
        PathBuf::from(spec),
        cards_dir.join(format!("{spec}.bbsa")),
    ];
    let path = candidates.iter().find(|p| p.is_file()).ok_or_else(|| {
        format!(
            "card {spec:?} not found (looked in {} and next to the test)",
            cards_dir.display()
        )
    })?;
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if path.extension().is_some_and(|e| e == "bbsa") {
        bbsa::import(&text, Some(spec))
            .map(|(c, _)| c)
            .map_err(|e| e.to_string())
    } else {
        Card::from_json(&text)
            .map(|(c, _)| c)
            .map_err(|e| e.to_string())
    }
}

/// Run the cases in `files` against the rules in `rules`. Both sides play
/// each case's card.
pub fn run(files: &[PathBuf], rules: &Path, cards_dir: &Path) -> Result<Vec<Outcome>, Vec<String>> {
    let modules = crate::load_modules(rules)
        .map_err(|d| d.iter().map(|d| d.to_string()).collect::<Vec<_>>())?;
    let mut engines: HashMap<(String, PathBuf), Engine> = HashMap::new();
    let mut outcomes = Vec::new();
    let mut errors = Vec::new();
    for file in files {
        let text = match std::fs::read_to_string(file) {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("{}: {e}", file.display()));
                continue;
            }
        };
        let cases = match parse(&text, &file.display().to_string()) {
            Ok(c) => c,
            Err(e) => {
                errors.extend(e);
                continue;
            }
        };
        let dir = file.parent().unwrap_or(Path::new(".")).to_path_buf();
        for case in cases {
            let key = (case.card.clone(), dir.clone());
            if !engines.contains_key(&key) {
                match load_card(&case.card, &dir, cards_dir) {
                    Ok(card) => {
                        engines.insert(key.clone(), Engine::new(&card, &card, &modules));
                    }
                    Err(e) => {
                        errors.push(format!("{}:{}: {e}", case.file, case.line));
                        continue;
                    }
                }
            }
            let engine = &engines[&key];
            let hand = Hand::from_pbn(&case.hand).expect("checked when parsed");
            let d = engine.bid(&hand, case.dealer, case.vul, case.scoring, &case.auction);
            let trace = d
                .candidates
                .iter()
                .map(|c| {
                    format!(
                        "    {:5} {:5.3} {}",
                        c.call.to_pbn(),
                        c.descriptiveness,
                        c.outcome
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            outcomes.push(Outcome {
                passed: case.expect.accepts(&d.call),
                got: d.call,
                explanation: d.explanation,
                trace,
                case,
            });
        }
    }
    if errors.is_empty() {
        Ok(outcomes)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_settings_and_cases() {
        let text = "card X\ndealer S\n# c\nN 82.QJ973.K94.J83 | 1NT P | 2D | weak transfer\nS AK5.KJ7.Q942.K83 | 1NT P 2D P | !P\n";
        let cases = parse(text, "t.test").unwrap();
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].seat, Direction::North);
        assert!(cases[1]
            .expect
            .accepts(&Call::bid(2, bridge_types::Strain::Hearts)));
        assert!(!cases[1].expect.accepts(&Call::Pass));
    }

    #[test]
    fn wrong_seat_is_an_error() {
        let e = parse("dealer S\nS 82.QJ973.K94.J83 | 1NT P | 2D\n", "t.test").unwrap_err();
        assert!(e[0].contains("it is N's turn"), "{e:?}");
    }
}
