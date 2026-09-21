//! The compiled form of a `.bid` module. It serializes to JSON (the IR that
//! the engine, the WASM build and other tools read).

use serde::{Deserialize, Serialize};

/// One `.bid` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Source file, as given to the parser.
    pub file: String,
    /// Activation: every condition must hold on the card.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub card: Vec<CardCond>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Param>,
    pub contexts: Vec<Context>,
}

/// `card <path> [= value]`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardCond {
    pub path: String,
    /// Required value; absent means "is on".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Literal>,
    pub line: usize,
}

/// `param <name> = <card path> [default <value>]`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Literal>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Literal {
    Bool(bool),
    Int(i64),
    Text(String),
}

/// An `after` and/or `when` context with the rules and contexts under it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<Vec<PatternCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<Expr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<Context>,
    pub line: usize,
}

/// One call in an `after` pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternCall {
    /// True for the opponents' calls (written in parentheses).
    pub theirs: bool,
    pub call: CallSpec,
}

/// A call as written in a rule or pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CallSpec {
    Pass,
    Double,
    Redouble,
    /// `(*)`: any call (patterns only).
    Any,
    Bid {
        level: u8,
        strain: StrainSpec,
    },
    /// `cheapest(x)`, `jump(x)`, `raise`, `new_suit`.
    Relative {
        func: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arg: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strain {
    C,
    D,
    H,
    S,
    N,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrainSpec {
    Lit(Strain),
    /// Suit variable: `M` (a major), `m` (a minor), `x` / `y` (any suit).
    Var(String),
    /// `{name}`: a state value or bound variable, e.g. `6{trump}`.
    Interp(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub call: CallSpec,
    pub explanation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alert: Option<Alert>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shows: Option<Expr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<Expr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denies: Option<Expr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sets: Vec<Assign>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefer: Option<Expr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Alert {
    Alert {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    Announce {
        text: String,
    },
}

/// `sets name=value`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assign {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArithOp {
    Add,
    Sub,
}

/// One segment of a dotted path, with arguments if it is a call:
/// `we.keycards(t).max` is `we`, `keycards(t)`, `max`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<Expr>>,
}

/// Conditions and values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Expr {
    And {
        all: Vec<Expr>,
    },
    Or {
        any: Vec<Expr>,
    },
    Not {
        expr: Box<Expr>,
    },
    /// `maybe <cond>`: not ruled out, rather than known.
    Maybe {
        expr: Box<Expr>,
    },
    Cmp {
        cmp: CmpOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `x = lo..hi`
    InRange {
        expr: Box<Expr>,
        lo: Box<Expr>,
        hi: Box<Expr>,
    },
    /// `x in 1|4`
    InSet {
        expr: Box<Expr>,
        values: Vec<i64>,
    },
    /// `we.trump is suit`
    Is {
        expr: Box<Expr>,
        what: String,
    },
    /// `asked [kind]`: partner's pending question to me.
    Asked {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<Box<Expr>>,
    },
    /// `answered [kind]`: partner answered my question.
    Answered {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<Box<Expr>>,
    },
    /// `shape 5-4-x-x`, `shape 4333`
    Shape {
        pattern: String,
    },
    Arith {
        arith: ArithOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Neg {
        expr: Box<Expr>,
    },
    Int {
        value: i64,
    },
    /// A call used as a value: `partner.last = 5{t}`.
    Call {
        call: CallSpec,
    },
    /// A name or dotted path: `hcp`, `partner.hcp.min`, `tp(M)`.
    Path {
        path: Vec<Segment>,
    },
}
