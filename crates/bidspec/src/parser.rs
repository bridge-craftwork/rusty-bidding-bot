//! Parsing of a whole `.bid` file: indentation, module header, contexts and
//! rules. Expressions and calls are parsed in `expr`.

use crate::ast::*;
use crate::expr::{self, CallPos, Cursor, PError};
use crate::lexer::{lex, Tok, Token};
use crate::Diagnostic;

const CLAUSES: &[&str] =
    &["shows", "when", "sets", "denies", "prefer", "priority", "replaces", "as", "alert", "announce"];
const HEADERS: &[&str] = &["card", "needs", "param"];

struct Line<'a> {
    no: usize,
    indent: usize,
    /// Text after the indentation, up to the end of the last token (so
    /// without a trailing comment).
    code: &'a str,
    toks: Vec<Token>,
}

impl Line<'_> {
    fn first_word(&self) -> Option<&str> {
        match self.toks.first().map(|t| &t.tok) {
            Some(Tok::Word(w)) => Some(w),
            _ => None,
        }
    }
}

struct Node {
    line: usize,
    children: Vec<Node>,
}

struct Parser<'a> {
    file: &'a str,
    lines: &'a [Line<'a>],
    diags: Vec<Diagnostic>,
}

pub fn parse(source: &str, file: &str) -> Result<Module, Vec<Diagnostic>> {
    let mut lines = Vec::new();
    let mut diags = Vec::new();
    for (i, raw) in source.lines().enumerate() {
        let trimmed = raw.trim_start_matches(' ');
        let indent = raw.len() - trimmed.len();
        match lex(trimmed) {
            Ok(toks) if toks.is_empty() => {}
            Ok(toks) => {
                let end = toks.last().unwrap().end;
                lines.push(Line { no: i + 1, indent, code: &trimmed[..end], toks });
            }
            Err((off, msg)) => diags.push(Diagnostic {
                file: file.to_string(),
                line: i + 1,
                col: indent + trimmed[..off].chars().count() + 1,
                message: msg,
            }),
        }
    }
    let mut p = Parser { file, lines: &lines, diags };
    let mut idx = 0;
    let tree = build(p.lines, &mut idx, None);
    let module = p.module(&tree);
    if p.diags.is_empty() {
        Ok(module.expect("no diagnostics means a module was built"))
    } else {
        Err(p.diags)
    }
}

/// Group lines into a tree by indentation.
fn build(lines: &[Line], idx: &mut usize, parent_indent: Option<usize>) -> Vec<Node> {
    let mut nodes = Vec::new();
    while *idx < lines.len() && parent_indent.is_none_or(|p| lines[*idx].indent > p) {
        let line = *idx;
        *idx += 1;
        let children = build(lines, idx, Some(lines[line].indent));
        nodes.push(Node { line, children });
    }
    nodes
}

impl<'a> Parser<'a> {
    fn error(&mut self, line: usize, offset: Option<usize>, message: impl Into<String>) {
        let lines = self.lines;
        let l = &lines[line];
        let col = offset.map_or(0, |off| l.indent + l.code[..off.min(l.code.len())].chars().count() + 1);
        self.diags.push(Diagnostic {
            file: self.file.to_string(),
            line: l.no,
            col,
            message: message.into(),
        });
    }

    fn perror(&mut self, line: usize, (off, msg): PError) {
        self.error(line, Some(off), msg);
    }

    fn module(&mut self, tree: &[Node]) -> Option<Module> {
        let lines = self.lines;
        let Some((first, rest)) = tree.split_first() else {
            self.diags.push(Diagnostic {
                file: self.file.to_string(),
                line: 1,
                col: 0,
                message: "empty file: expected `module <name> \"<title>\"`".into(),
            });
            return None;
        };
        if lines[first.line].first_word() != Some("module") {
            self.error(first.line, Some(0), "a .bid file must start with `module <name> \"<title>\"`");
            return None;
        }
        let (name, title) = self.module_line(first.line);
        let mut module = Module {
            name,
            title,
            file: self.file.to_string(),
            card: Vec::new(),
            needs: Vec::new(),
            params: Vec::new(),
            contexts: Vec::new(),
        };
        for node in &first.children {
            self.header(node, &mut module);
        }
        for node in rest {
            match lines[node.line].first_word() {
                Some("after" | "when") => {
                    if let Some(ctx) = self.context(node) {
                        module.contexts.push(ctx);
                    }
                }
                Some("module") => self.error(node.line, Some(0), "only one module per file"),
                Some(w) if HEADERS.contains(&w) => {
                    self.error(node.line, Some(0), format!("`{w}` must be indented under `module`"))
                }
                _ => self.error(
                    node.line,
                    Some(0),
                    "expected a context (`after <auction>` or `when <condition>`) at the left margin",
                ),
            }
        }
        Some(module)
    }

    /// Words of a header line after its keyword, split on whitespace.
    fn header_words(&self, line: usize) -> Vec<&'a str> {
        let lines = self.lines;
        lines[line].code.split_whitespace().skip(1).collect()
    }

    fn module_line(&mut self, line: usize) -> (String, Option<String>) {
        let lines = self.lines;
        let code = lines[line].code;
        let rest = code["module".len()..].trim_start();
        let name_end = rest.find(' ').unwrap_or(rest.len());
        let name = rest[..name_end].to_string();
        if !valid_id(&name) {
            self.error(line, Some(0), "expected `module <name>`; names use a-z, 0-9 and `-`");
        }
        let title = lines[line].toks.iter().find_map(|t| match &t.tok {
            Tok::Str(s) => Some(s.clone()),
            _ => None,
        });
        (name, title)
    }

    fn header(&mut self, node: &Node, module: &mut Module) {
        let lines = self.lines;
        let line = node.line;
        if !node.children.is_empty() {
            self.error(node.children[0].line, Some(0), "unexpected indentation under a header line");
        }
        let words = self.header_words(line);
        let no = lines[line].no;
        match lines[line].first_word() {
            Some("needs") => {
                if words.is_empty() {
                    self.error(line, None, "expected `needs <module>`");
                }
                for w in words {
                    if valid_id(w) {
                        module.needs.push(w.to_string());
                    } else {
                        self.error(line, None, format!("{w:?} is not a module name"));
                    }
                }
            }
            Some("card") => match words.as_slice() {
                [path] => module.card.push(CardCond { path: path.to_string(), value: None, line: no }),
                [path, "=", value] => module.card.push(CardCond {
                    path: path.to_string(),
                    value: Some(literal(value)),
                    line: no,
                }),
                _ => self.error(line, None, "expected `card <path>` or `card <path> = <value>`"),
            },
            Some("param") => match words.as_slice() {
                [name, "=", path] => module.params.push(Param {
                    name: name.to_string(),
                    path: path.to_string(),
                    default: None,
                    line: no,
                }),
                [name, "=", path, "default", value] => module.params.push(Param {
                    name: name.to_string(),
                    path: path.to_string(),
                    default: Some(literal(value)),
                    line: no,
                }),
                _ => self.error(line, None, "expected `param <name> = <card path> [default <value>]`"),
            },
            _ => self.error(line, Some(0), "expected `card`, `needs` or `param` under `module`"),
        }
    }

    fn context(&mut self, node: &Node) -> Option<Context> {
        let lines = self.lines;
        let line = node.line;
        let l = &lines[line];
        let toks = &l.toks[1..];
        let mut ctx = Context { after: None, when: None, rules: Vec::new(), contexts: Vec::new(), line: l.no };
        let parsed = if l.first_word() == Some("after") {
            let split = toks
                .iter()
                .position(|t| t.tok == Tok::Word("when".into()))
                .unwrap_or(toks.len());
            let pattern = self.pattern(&toks[..split], l.code);
            let when = if split < toks.len() { Some(self.condition(&toks[split + 1..], l.code)) } else { None };
            pattern.and_then(|p| {
                ctx.after = Some(p);
                when.transpose().map(|w| ctx.when = w)
            })
        } else {
            self.condition(toks, l.code).map(|w| ctx.when = Some(w))
        };
        if let Err(e) = parsed {
            self.perror(line, e);
            return None;
        }
        for child in &node.children {
            match lines[child.line].first_word() {
                Some("after" | "when") => {
                    if let Some(c) = self.context(child) {
                        ctx.contexts.push(c);
                    }
                }
                Some(w) if CLAUSES.contains(&w) => self.error(
                    child.line,
                    Some(0),
                    format!("`{w}` must be indented under a rule (a call and its explanation)"),
                ),
                _ => {
                    if let Some(rule) = self.rule(child) {
                        ctx.rules.push(rule);
                    }
                }
            }
        }
        Some(ctx)
    }

    fn pattern(&self, toks: &[Token], text: &str) -> Result<Vec<PatternCall>, PError> {
        let mut c = Cursor::new(toks, text);
        let mut calls = Vec::new();
        while !c.at_end() {
            let theirs = c.peek() == Some(&Tok::LParen);
            let start = c.offset();
            let call = if theirs {
                expr::paren_call(&mut c)?
            } else {
                expr::call(&mut c, CallPos::Pattern)?
            };
            if calls.last().is_some_and(|p: &PatternCall| p.theirs == theirs) {
                return Err((
                    start,
                    "calls must alternate between our side and (theirs), in parentheses".into(),
                ));
            }
            calls.push(PatternCall { theirs, call });
        }
        match calls.last() {
            None => c.err("expected an auction after `after`"),
            Some(last) if !last.theirs => c.err(
                "a pattern ends with the call just before my turn, which is RHO's: add `(P)` or `(*)`",
            ),
            _ => Ok(calls),
        }
    }

    fn condition(&self, toks: &[Token], text: &str) -> Result<Expr, PError> {
        let mut c = Cursor::new(toks, text);
        if c.at_end() {
            return c.err("expected a condition");
        }
        let e = expr::expr(&mut c)?;
        c.expect_end("in condition")?;
        Ok(e)
    }

    fn rule(&mut self, node: &Node) -> Option<Rule> {
        let lines = self.lines;
        let line = node.line;
        let (head, clauses) = split_clauses(&lines[line].toks);
        let l = &lines[line];
        let mut c = Cursor::new(head, l.code);
        let call = match expr::call(&mut c, CallPos::Rule) {
            Ok(call) => call,
            Err(e) => {
                self.perror(line, e);
                return None;
            }
        };
        let Some(explanation) = expr::string(&mut c) else {
            let off = c.offset();
            self.error(line, Some(off), "a rule needs an explanation string after the call");
            return None;
        };
        if let Err(e) = c.expect_end("after the explanation; expected a clause such as `shows`") {
            self.perror(line, e);
        }
        let mut rule = Rule {
            call,
            explanation,
            alert: None,
            shows: None,
            when: None,
            denies: None,
            sets: Vec::new(),
            prefer: None,
            priority: None,
            replaces: None,
            id: None,
            line: l.no,
        };
        let mut all = vec![(line, clauses)];
        for child in &node.children {
            if !child.children.is_empty() {
                self.error(child.children[0].line, Some(0), "unexpected indentation");
            }
            let (head, clauses) = split_clauses(&lines[child.line].toks);
            if !head.is_empty() {
                self.error(
                    child.line,
                    Some(0),
                    "expected a clause (shows, when, sets, denies, prefer, priority, replaces, as); \
                     a rule cannot contain another rule",
                );
                continue;
            }
            all.push((child.line, clauses));
        }
        for (line, clauses) in all {
            for (kw, body) in clauses {
                if let Err(e) = self.clause(&mut rule, line, &kw, body) {
                    self.perror(line, e);
                }
            }
        }
        Some(rule)
    }

    fn clause(&self, rule: &mut Rule, line: usize, kw: &Token, body: &[Token]) -> Result<(), PError> {
        let lines = self.lines;
        let text = lines[line].code;
        let Tok::Word(name) = &kw.tok else { unreachable!() };
        let dup = |what: &str| Err((kw.start, format!("`{what}` given twice for this rule")));
        let raw = || -> Result<String, PError> {
            match (body.first(), body.last()) {
                (Some(a), Some(b)) if !text[a.start..b.end].contains(' ') => {
                    Ok(text[a.start..b.end].to_string())
                }
                _ => Err((kw.end, format!("expected one name after `{name}`"))),
            }
        };
        let mut c = Cursor::new(body, text);
        match name.as_str() {
            "shows" | "when" | "denies" => {
                let e = self.condition(body, text).map_err(|e| {
                    if body.is_empty() { (kw.end, format!("expected a condition after `{name}`")) } else { e }
                })?;
                let slot = match name.as_str() {
                    "shows" => &mut rule.shows,
                    "when" => &mut rule.when,
                    _ => &mut rule.denies,
                };
                *slot = Some(match slot.take() {
                    None => e,
                    Some(Expr::And { mut all }) => {
                        all.push(e);
                        Expr::And { all }
                    }
                    Some(prev) => Expr::And { all: vec![prev, e] },
                });
            }
            "prefer" => {
                if rule.prefer.is_some() {
                    return dup("prefer");
                }
                rule.prefer = Some(self.condition(body, text)?);
            }
            "priority" => {
                if rule.priority.is_some() {
                    return dup("priority");
                }
                let neg = matches!(body.first().map(|t| &t.tok), Some(Tok::Minus));
                match body.get(neg as usize).map(|t| &t.tok) {
                    Some(Tok::Int(n)) if body.len() == 1 + neg as usize => {
                        rule.priority = Some(if neg { -n } else { *n })
                    }
                    _ => return Err((kw.end, "expected a whole number after `priority`".into())),
                }
            }
            "sets" => loop {
                let name = match c.peek() {
                    Some(Tok::Word(w)) => w.clone(),
                    _ => return c.err("expected `name=value`"),
                };
                let assign = expr::assignment(&mut c).map(|value| Assign { name, value })?;
                rule.sets.push(assign);
                if c.at_end() {
                    break;
                }
                expr::comma(&mut c)?;
            },
            "replaces" => {
                if rule.replaces.is_some() {
                    return dup("replaces");
                }
                rule.replaces = Some(raw()?);
            }
            "as" => {
                if rule.id.is_some() {
                    return dup("as");
                }
                rule.id = Some(raw()?);
            }
            "alert" | "announce" => {
                if rule.alert.is_some() {
                    return dup("alert/announce");
                }
                let text = match body {
                    [] => None,
                    [Token { tok: Tok::Str(s), .. }] => Some(s.clone()),
                    _ => return Err((kw.end, format!("expected a quoted string after `{name}`"))),
                };
                rule.alert = Some(if name == "alert" {
                    Alert::Alert { text }
                } else {
                    let Some(text) = text else {
                        return Err((kw.end, "`announce` needs the text to announce".into()));
                    };
                    Alert::Announce { text }
                });
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

/// Split a rule line into the head (call and explanation) and its clauses,
/// each clause keyword with the tokens up to the next keyword.
fn split_clauses(toks: &[Token]) -> (&[Token], Vec<(Token, &[Token])>) {
    let mut depth = 0i32;
    let mut starts = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        match &t.tok {
            Tok::LParen => depth += 1,
            Tok::RParen => depth -= 1,
            Tok::Word(w) if depth == 0 && CLAUSES.contains(&w.as_str()) => starts.push(i),
            _ => {}
        }
    }
    let head_end = starts.first().copied().unwrap_or(toks.len());
    let mut clauses = Vec::new();
    for (n, &s) in starts.iter().enumerate() {
        let end = starts.get(n + 1).copied().unwrap_or(toks.len());
        clauses.push((toks[s].clone(), &toks[s + 1..end]));
    }
    (&toks[..head_end], clauses)
}

fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn literal(s: &str) -> Literal {
    match s {
        "true" => Literal::Bool(true),
        "false" => Literal::Bool(false),
        _ => match s.parse() {
            Ok(n) => Literal::Int(n),
            Err(_) => Literal::Text(s.trim_matches('"').to_string()),
        },
    }
}
