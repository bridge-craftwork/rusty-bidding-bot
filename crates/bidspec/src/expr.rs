//! Parsing of calls and expressions from a token slice.
//!
//! Precedence, loosest first: `,` (and), `|` (or), `!` / `maybe`,
//! comparisons, `+` / `-`.

use crate::ast::*;
use crate::lexer::{Tok, Token};

/// Error: byte offset in the line, and message.
pub type PError = (usize, String);

pub struct Cursor<'a> {
    toks: &'a [Token],
    pos: usize,
    /// The line text the token offsets refer to.
    text: &'a str,
}

impl<'a> Cursor<'a> {
    pub fn new(toks: &'a [Token], text: &'a str) -> Self {
        Cursor { toks, pos: 0, text }
    }

    pub fn peek(&self) -> Option<&'a Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }

    fn peek_at(&self, n: usize) -> Option<&'a Tok> {
        self.toks.get(self.pos + n).map(|t| &t.tok)
    }

    pub fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }

    fn bump(&mut self) -> Option<&'a Token> {
        let t = self.toks.get(self.pos);
        self.pos += 1;
        t
    }

    /// Offset of the current token (or end of the last one), for errors.
    pub fn offset(&self) -> usize {
        match self.toks.get(self.pos) {
            Some(t) => t.start,
            None => self.toks.last().map_or(0, |t| t.end),
        }
    }

    pub fn err<T>(&self, msg: impl Into<String>) -> Result<T, PError> {
        Err((self.offset(), msg.into()))
    }

    fn eat(&mut self, tok: &Tok) -> bool {
        if self.peek() == Some(tok) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, tok: &Tok, what: &str) -> Result<(), PError> {
        if self.eat(tok) {
            Ok(())
        } else {
            self.err(format!("expected {what}"))
        }
    }

    fn word(&mut self, what: &str) -> Result<String, PError> {
        match self.peek() {
            Some(Tok::Word(w)) => {
                let w = w.clone();
                self.pos += 1;
                Ok(w)
            }
            _ => self.err(format!("expected {what}")),
        }
    }

    /// Is the next token directly after the current one, with no space?
    fn adjacent(&self) -> bool {
        match (self.toks.get(self.pos.wrapping_sub(1)), self.toks.get(self.pos)) {
            (Some(a), Some(b)) if self.pos > 0 => a.end == b.start,
            _ => false,
        }
    }

    pub fn expect_end(&self, context: &str) -> Result<(), PError> {
        if self.at_end() {
            Ok(())
        } else {
            let t = &self.toks[self.pos];
            self.err(format!("unexpected {:?} {context}", &self.text[t.start..t.end]))
        }
    }
}

const SUIT_VARS: &[&str] = &["M", "m", "x", "y"];

fn strain(letters: &str) -> Option<StrainSpec> {
    Some(match letters {
        "C" => StrainSpec::Lit(Strain::C),
        "D" => StrainSpec::Lit(Strain::D),
        "H" => StrainSpec::Lit(Strain::H),
        "S" => StrainSpec::Lit(Strain::S),
        "N" | "NT" => StrainSpec::Lit(Strain::N),
        v if SUIT_VARS.contains(&v) => StrainSpec::Var(v.to_string()),
        _ => return None,
    })
}

/// Which kinds of call are allowed where.
#[derive(Clone, Copy, PartialEq)]
pub enum CallPos {
    /// A rule's call: templates and relative calls allowed.
    Rule,
    /// An `after` pattern: `*` allowed.
    Pattern,
    /// A call used as a value in an expression.
    Value,
}

pub fn call(c: &mut Cursor, pos: CallPos) -> Result<CallSpec, PError> {
    let start = c.offset();
    let spec = match c.peek() {
        Some(Tok::Word(w)) => match w.as_str() {
            "P" | "Pass" => {
                c.bump();
                CallSpec::Pass
            }
            "X" => {
                c.bump();
                CallSpec::Double
            }
            "XX" => {
                c.bump();
                CallSpec::Redouble
            }
            "cheapest" | "jump" if pos == CallPos::Rule => {
                let func = w.clone();
                c.bump();
                c.expect(&Tok::LParen, "`(` after the function name")?;
                let arg = c.word("a suit or variable")?;
                c.expect(&Tok::RParen, "`)`")?;
                CallSpec::Relative { func, arg: Some(arg) }
            }
            "raise" | "new_suit" if pos == CallPos::Rule => {
                let func = w.clone();
                c.bump();
                CallSpec::Relative { func, arg: None }
            }
            _ => return c.err(format!("expected a call, found {w:?}")),
        },
        Some(Tok::Star) if pos == CallPos::Pattern => {
            c.bump();
            CallSpec::Any
        }
        Some(Tok::Call(level, letters)) => {
            let (level, letters) = (*level, letters.clone());
            c.bump();
            let Some(strain) = strain(&letters) else {
                return Err((
                    start,
                    format!("{level}{letters}: strain must be C D H S N or a variable M m x y"),
                ));
            };
            CallSpec::Bid { level, strain }
        }
        Some(Tok::Int(level)) if pos != CallPos::Pattern && c.peek_at(1) == Some(&Tok::LBrace) => {
            let level = *level;
            c.bump();
            c.bump();
            let name = c.word("a name inside `{}`")?;
            c.expect(&Tok::RBrace, "`}`")?;
            CallSpec::Bid { level: level.clamp(0, 99) as u8, strain: StrainSpec::Interp(name) }
        }
        _ => return c.err("expected a call"),
    };
    if let CallSpec::Bid { level, .. } = spec {
        if !(1..=7).contains(&level) {
            return Err((start, format!("level {level} is not 1 to 7")));
        }
    }
    Ok(spec)
}

/// `(call)`: an opponent's call in a pattern.
pub fn paren_call(c: &mut Cursor) -> Result<CallSpec, PError> {
    c.expect(&Tok::LParen, "`(`")?;
    let call = call(c, CallPos::Pattern)?;
    c.expect(&Tok::RParen, "`)` after the opponent's call")?;
    Ok(call)
}

/// `name=value` in a `sets` clause; returns the value.
pub fn assignment(c: &mut Cursor) -> Result<Expr, PError> {
    c.word("a state name")?;
    c.expect(&Tok::Eq, "`=`")?;
    or(c)
}

pub fn comma(c: &mut Cursor) -> Result<(), PError> {
    c.expect(&Tok::Comma, "`,` between assignments")
}

/// A quoted string, if next.
pub fn string(c: &mut Cursor) -> Option<String> {
    match c.peek() {
        Some(Tok::Str(s)) => {
            let s = s.clone();
            c.bump();
            Some(s)
        }
        _ => None,
    }
}

/// Full expression: `a, b | c, !d`.
pub fn expr(c: &mut Cursor) -> Result<Expr, PError> {
    let mut all = vec![or(c)?];
    while c.eat(&Tok::Comma) {
        all.push(or(c)?);
    }
    Ok(if all.len() == 1 { all.pop().unwrap() } else { Expr::And { all } })
}

fn or(c: &mut Cursor) -> Result<Expr, PError> {
    let mut any = vec![unary(c)?];
    while c.eat(&Tok::Pipe) {
        any.push(unary(c)?);
    }
    Ok(if any.len() == 1 { any.pop().unwrap() } else { Expr::Or { any } })
}

fn unary(c: &mut Cursor) -> Result<Expr, PError> {
    if c.eat(&Tok::Bang) {
        return Ok(Expr::Not { expr: Box::new(unary(c)?) });
    }
    if c.peek() == Some(&Tok::Word("maybe".into())) {
        c.bump();
        return Ok(Expr::Maybe { expr: Box::new(unary(c)?) });
    }
    comparison(c)
}

/// Tokens that end an operand (so `asked` alone is not followed by a kind).
fn ends_operand(t: Option<&Tok>) -> bool {
    matches!(t, None | Some(Tok::Comma | Tok::Pipe | Tok::RParen))
}

fn comparison(c: &mut Cursor) -> Result<Expr, PError> {
    match c.peek() {
        Some(Tok::Word(w)) if w == "shape" => {
            c.bump();
            let start = c.offset();
            let mut end = start;
            while !ends_operand(c.peek()) {
                end = c.bump().unwrap().end;
            }
            let pattern: String = c.text[start..end].chars().filter(|ch| *ch != ' ').collect();
            if pattern.is_empty() {
                return c.err("expected a shape such as 4333 or 5-4-x-x");
            }
            return Ok(Expr::Shape { pattern });
        }
        Some(Tok::Word(w)) if w == "asked" || w == "answered" => {
            let asked = w == "asked";
            c.bump();
            let kind = if ends_operand(c.peek()) { None } else { Some(Box::new(arith(c)?)) };
            return Ok(if asked { Expr::Asked { kind } } else { Expr::Answered { kind } });
        }
        _ => {}
    }
    let lhs = arith(c)?;
    let cmp = match c.peek() {
        Some(Tok::Eq) => CmpOp::Eq,
        Some(Tok::Ne) => CmpOp::Ne,
        Some(Tok::Lt) => CmpOp::Lt,
        Some(Tok::Le) => CmpOp::Le,
        Some(Tok::Gt) => CmpOp::Gt,
        Some(Tok::Ge) => CmpOp::Ge,
        Some(Tok::Word(w)) if w == "in" => {
            c.bump();
            let mut values = Vec::new();
            loop {
                let neg = c.eat(&Tok::Minus);
                match c.peek() {
                    Some(Tok::Int(n)) => {
                        values.push(if neg { -n } else { *n });
                        c.bump();
                    }
                    _ => return c.err("expected a number in the `in` set"),
                }
                if !c.eat(&Tok::Pipe) {
                    break;
                }
            }
            return Ok(Expr::InSet { expr: Box::new(lhs), values });
        }
        Some(Tok::Word(w)) if w == "is" => {
            c.bump();
            let what = c.word("a word after `is` (suit, notrump, none)")?;
            return Ok(Expr::Is { expr: Box::new(lhs), what });
        }
        _ => return Ok(lhs),
    };
    c.bump();
    let rhs = arith(c)?;
    if cmp == CmpOp::Eq && c.eat(&Tok::DotDot) {
        let hi = arith(c)?;
        return Ok(Expr::InRange { expr: Box::new(lhs), lo: Box::new(rhs), hi: Box::new(hi) });
    }
    Ok(Expr::Cmp { cmp, lhs: Box::new(lhs), rhs: Box::new(rhs) })
}

fn arith(c: &mut Cursor) -> Result<Expr, PError> {
    let mut lhs = term(c)?;
    loop {
        let op = match c.peek() {
            Some(Tok::Plus) => ArithOp::Add,
            Some(Tok::Minus) => ArithOp::Sub,
            _ => return Ok(lhs),
        };
        c.bump();
        let rhs = term(c)?;
        lhs = Expr::Arith { arith: op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
    }
}

fn term(c: &mut Cursor) -> Result<Expr, PError> {
    match c.peek() {
        Some(Tok::Minus) => {
            c.bump();
            Ok(Expr::Neg { expr: Box::new(term(c)?) })
        }
        Some(Tok::LParen) => {
            c.bump();
            let e = expr(c)?;
            c.expect(&Tok::RParen, "`)`")?;
            Ok(e)
        }
        Some(Tok::Int(_)) if c.peek_at(1) == Some(&Tok::LBrace) => {
            Ok(Expr::Call { call: call(c, CallPos::Value)? })
        }
        Some(Tok::Int(n)) => {
            let n = *n;
            c.bump();
            Ok(Expr::Int { value: n })
        }
        Some(Tok::Call(..)) => Ok(Expr::Call { call: call(c, CallPos::Value)? }),
        Some(Tok::Word(_)) => path(c),
        Some(Tok::DotDot) => c.err("`..` only follows `=`, as in `hcp=15..17`"),
        _ => c.err("expected a value or condition"),
    }
}

fn path(c: &mut Cursor) -> Result<Expr, PError> {
    let mut segs = Vec::new();
    loop {
        let name = c.word("a name")?;
        let args = if c.peek() == Some(&Tok::LParen) && c.adjacent() {
            c.bump();
            let mut args = vec![or(c)?];
            while c.eat(&Tok::Comma) {
                args.push(or(c)?);
            }
            c.expect(&Tok::RParen, "`)` after the arguments")?;
            Some(args)
        } else {
            None
        };
        segs.push(Segment { name, args });
        if !c.eat(&Tok::Dot) {
            break;
        }
    }
    Ok(Expr::Path { path: segs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse(s: &str) -> Expr {
        let toks = lex(s).unwrap();
        let mut c = Cursor::new(&toks, s);
        let e = expr(&mut c).unwrap();
        c.expect_end("").unwrap();
        e
    }

    fn name(n: &str) -> Expr {
        Expr::Path { path: vec![Segment { name: n.into(), args: None }] }
    }

    #[test]
    fn comma_binds_looser_than_pipe() {
        let e = parse("hcp>=8, H=4 | S=4");
        let Expr::And { all } = e else { panic!("{e:?}") };
        assert_eq!(all.len(), 2);
        assert!(matches!(all[1], Expr::Or { .. }));
    }

    #[test]
    fn ranges_with_arithmetic() {
        let e = parse("hcp=33-partner.hcp.max..32-partner.hcp.min");
        let Expr::InRange { expr, lo, .. } = e else { panic!("{e:?}") };
        assert_eq!(*expr, name("hcp"));
        assert!(matches!(*lo, Expr::Arith { arith: ArithOp::Sub, .. }));
    }

    #[test]
    fn calls_paths_and_sets() {
        let e = parse("partner.last=5{t}, we.keycards(t).max<=3");
        let Expr::And { all } = e else { panic!() };
        let Expr::Cmp { rhs, .. } = &all[0] else { panic!() };
        assert!(matches!(**rhs, Expr::Call { .. }));
        let Expr::Cmp { lhs, .. } = &all[1] else { panic!() };
        let Expr::Path { path } = &**lhs else { panic!() };
        assert_eq!(path.len(), 3);
        assert!(path[1].args.is_some());

        assert!(matches!(parse("keycards(t) in 1|4"), Expr::InSet { .. }));
        assert!(matches!(parse("we.trump is suit"), Expr::Is { .. }));
        assert!(matches!(parse("!shape 4333"), Expr::Not { .. }));
        assert_eq!(parse("shape 5-4-x-x"), Expr::Shape { pattern: "5-4-x-x".into() });
        assert!(matches!(parse("asked keycards(t)"), Expr::Asked { kind: Some(_) }));
        assert!(matches!(parse("!asked"), Expr::Not { .. }));
    }

    #[test]
    fn rule_calls() {
        let toks = lex("7{t} 3x cheapest(x) 8C").unwrap();
        let mut c = Cursor::new(&toks, "");
        assert!(matches!(
            call(&mut c, CallPos::Rule).unwrap(),
            CallSpec::Bid { level: 7, strain: StrainSpec::Interp(_) }
        ));
        assert!(matches!(
            call(&mut c, CallPos::Rule).unwrap(),
            CallSpec::Bid { level: 3, strain: StrainSpec::Var(_) }
        ));
        assert!(matches!(call(&mut c, CallPos::Rule).unwrap(), CallSpec::Relative { .. }));
        assert!(call(&mut c, CallPos::Rule).is_err());
    }
}
