//! Splits one line of a `.bid` file into tokens. Comments (`#` to end of
//! line, outside strings) are dropped.

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Int(i64),
    Word(String),
    /// A level followed directly by letters: `1N`, `2D`, `3x`, `1NT`.
    Call(u8, String),
    Str(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Pipe,
    Bang,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Dot,
    DotDot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    /// Byte offsets into the line text.
    pub start: usize,
    pub end: usize,
}

/// Error: byte offset and message.
pub type LexError = (usize, String);

pub fn lex(text: &str) -> Result<Vec<Token>, LexError> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let start = i;
        let tok = match c {
            b' ' => {
                i += 1;
                continue;
            }
            b'\t' => return Err((i, "tabs are not allowed; indent with spaces".into())),
            b'#' => break,
            b'"' => {
                let mut s = String::new();
                i += 1;
                loop {
                    match text[i..].chars().next() {
                        None => return Err((start, "unterminated string".into())),
                        Some('"') => {
                            i += 1;
                            break;
                        }
                        Some('\\') if text[i + 1..].starts_with('"') => {
                            s.push('"');
                            i += 2;
                        }
                        Some(ch) => {
                            s.push(ch);
                            i += ch.len_utf8();
                        }
                    }
                }
                Tok::Str(s)
            }
            b'0'..=b'9' => {
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let digits = &text[start..i];
                if i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                    let letters_start = i;
                    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
                        i += 1;
                    }
                    let level: u8 = digits
                        .parse()
                        .map_err(|_| (start, format!("bad level in {:?}", &text[start..i])))?;
                    Tok::Call(level, text[letters_start..i].to_string())
                } else {
                    Tok::Int(
                        digits
                            .parse()
                            .map_err(|_| (start, "number too large".to_string()))?,
                    )
                }
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                Tok::Word(text[start..i].to_string())
            }
            _ => {
                let two = text.get(i..i + 2).unwrap_or("");
                let (tok, len) = match two {
                    "!=" => (Tok::Ne, 2),
                    "<=" => (Tok::Le, 2),
                    ">=" => (Tok::Ge, 2),
                    "==" => (Tok::Eq, 2),
                    ".." => (Tok::DotDot, 2),
                    _ => match c {
                        b'(' => (Tok::LParen, 1),
                        b')' => (Tok::RParen, 1),
                        b'{' => (Tok::LBrace, 1),
                        b'}' => (Tok::RBrace, 1),
                        b',' => (Tok::Comma, 1),
                        b'|' => (Tok::Pipe, 1),
                        b'!' => (Tok::Bang, 1),
                        b'=' => (Tok::Eq, 1),
                        b'<' => (Tok::Lt, 1),
                        b'>' => (Tok::Gt, 1),
                        b'+' => (Tok::Plus, 1),
                        b'-' => (Tok::Minus, 1),
                        b'*' => (Tok::Star, 1),
                        b'.' => (Tok::Dot, 1),
                        _ => {
                            let ch = text[i..].chars().next().unwrap();
                            return Err((i, format!("unexpected character {ch:?}")));
                        }
                    },
                };
                i += len;
                tok
            }
        };
        out.push(Token { tok, start, end: i });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(s: &str) -> Vec<Tok> {
        lex(s).unwrap().into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn calls_numbers_and_ranges() {
        assert_eq!(
            toks("1N (P) hcp=15..17 # comment"),
            vec![
                Tok::Call(1, "N".into()),
                Tok::LParen,
                Tok::Word("P".into()),
                Tok::RParen,
                Tok::Word("hcp".into()),
                Tok::Eq,
                Tok::Int(15),
                Tok::DotDot,
                Tok::Int(17),
            ]
        );
        assert_eq!(
            toks("6{t}"),
            vec![Tok::Int(6), Tok::LBrace, Tok::Word("t".into()), Tok::RBrace]
        );
    }

    #[test]
    fn strings_keep_hash_and_unicode() {
        assert_eq!(toks(r#""♠ #1 \"x\"""#), vec![Tok::Str("♠ #1 \"x\"".into())]);
        assert!(lex("\"open").is_err());
    }
}
