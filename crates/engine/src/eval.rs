//! Evaluating `.bid` expressions.
//!
//! The actor's own hand is exact when choosing a call (`Ctx::hand` is set)
//! and a set of ranges otherwise. Other seats are always ranges, so a
//! comparison about them is `Tri::True` only when it is known.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use bidspec::ast::{ArithOp, CallSpec, CmpOp, Expr, Segment, StrainSpec};
use bridge_types::{Call, Direction, Strain};

use crate::facts::{Facts, Valuation, ACE, JACK, KING, QUEEN};
use crate::knowledge::{suit_index, Range, SeatKnowledge, Tri, SUITS};
use crate::position::{Forcing, Position};

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Num(Range),
    Bool(Tri),
    Suit(usize),
    Strain(Strain),
    Call(Call),
    Sym(String),
    /// `kind(args)` as written in `sets ask=keycards(trump)`.
    Ask(String, Vec<Strain>),
    /// No value: no trump agreed, no call made yet.
    Nothing,
}

pub type Bindings = HashMap<String, Val>;

pub fn strain_of_suit(s: usize) -> Strain {
    [
        Strain::Clubs,
        Strain::Diamonds,
        Strain::Hearts,
        Strain::Spades,
    ][s]
}

pub fn suit_of_strain(s: Strain) -> Option<usize> {
    match s {
        Strain::Clubs => Some(0),
        Strain::Diamonds => Some(1),
        Strain::Hearts => Some(2),
        Strain::Spades => Some(3),
        Strain::NoTrump => None,
    }
}

pub fn strain_from_spec(s: bidspec::ast::Strain) -> Strain {
    use bidspec::ast::Strain as S;
    match s {
        S::C => Strain::Clubs,
        S::D => Strain::Diamonds,
        S::H => Strain::Hearts,
        S::S => Strain::Spades,
        S::N => Strain::NoTrump,
    }
}

fn quality_level(word: &str) -> Option<i32> {
    Some(match word {
        "poor" => 0,
        "fair" => 1,
        "good" => 2,
        "excellent" => 3,
        _ => return None,
    })
}

fn rank_value(word: &str) -> Option<u8> {
    Some(match word {
        "A" => ACE,
        "K" => KING,
        "Q" => QUEEN,
        "J" => JACK,
        "T" => 10,
        _ => return None,
    })
}

/// A name the evaluator accepts and what it means. The lists below are
/// both what `check_terms` allows and the reference `rbb bid terms`
/// prints (docs/CONTRACT.md), so a new term is documented where it is
/// added.
#[derive(Debug, Clone, Copy)]
pub struct Term {
    pub name: &'static str,
    /// How it takes arguments, as written: `(x)`, `(rank, x)`; empty for none.
    pub args: &'static str,
    pub meaning: &'static str,
}

const fn t(name: &'static str, meaning: &'static str) -> Term {
    Term {
        name,
        args: "",
        meaning,
    }
}

const fn f(name: &'static str, args: &'static str, meaning: &'static str) -> Term {
    Term {
        name,
        args,
        meaning,
    }
}

fn named(list: &[Term], n: &str) -> bool {
    list.iter().any(|t| t.name == n)
}

/// Words that stand for themselves (`we.forcing = game`, `quality(x) >= good`).
const SYMBOLS: &[Term] = &[
    t(
        "game",
        "a forcing level (`we.forcing = game`), and a strength band",
    ),
    t(
        "round",
        "a forcing level: partner may not pass at the next turn",
    ),
    t(
        "none",
        "no forcing level; with `is`, no trump agreed (`we.trump is none`)",
    ),
    t(
        "suit",
        "with `is`: the agreed trump is a suit (`we.trump is suit`)",
    ),
    t("notrump", "with `is`: the agreed strain is notrump"),
    t("poor", "suit quality 0: none of A, K, Q"),
    t("fair", "suit quality 1: one of A, K, Q"),
    t("good", "suit quality 2: two of A, K, Q"),
    t("excellent", "suit quality 3: A, K and Q"),
    t(
        "signoff",
        "strength band: too weak to invite (`strength=signoff`)",
    ),
    t(
        "invite",
        "strength band: invitational opposite partner's range",
    ),
    t("slam_invite", "strength band: enough to invite slam"),
    t(
        "slam",
        "strength band: enough for slam opposite partner's minimum",
    ),
    t("A", "the ace, in `has(A, x)`"),
    t("K", "the king, in `has(K, x)`"),
    t("Q", "the queen, in `has(Q, x)`"),
    t("J", "the jack, in `has(J, x)`"),
    t("T", "the ten, in `has(T, x)`"),
];
/// Strength bands, weakest first (see `Ctx::strength_as_hcp`).
pub const BANDS: [&str; 5] = ["signoff", "invite", "game", "slam_invite", "slam"];
/// Combined HCP for game and for small slam.
const GAME: i32 = 25;
const SLAM: i32 = 33;
/// `captain`: partner's shown range is at most this wide (high minus low,
/// so 3 is a range of four points: 12-15).
const CAPTAIN_WIDTH: i32 = 3;

fn flip(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Ge => CmpOp::Le,
        o => o,
    }
}

/// My own hand, exact while I choose a call (bare, or `me.`); what I
/// have shown otherwise.
const SELF_ATTRS: &[Term] = &[
    t("hcp", "high-card points (A 4, K 3, Q 2, J 1)"),
    t("tens", "tens held"),
    t(
        "points",
        "total points: HCP + 1/2 a ten + 1 a card beyond four (whole part)",
    ),
    t(
        "suit_points",
        "suit points: HCP + 1/2 a card beyond four (whole part)",
    ),
    t("balanced", "4-3-3-3, 4-4-3-2 or 5-3-3-2"),
    t(
        "semibalanced",
        "no singleton or void, no suit longer than six",
    ),
    t("shortest", "length of the shortest suit"),
    t("longest", "length of the longest suit"),
    t("second_longest", "length of the second-longest suit"),
    t(
        "length_points",
        "one for each card beyond four in every suit",
    ),
    t(
        "bba_nt_points",
        "BBA's count opposite 1NT, scaled so it invites from 8 (matchpoints)",
    ),
    t(
        "bba_nt_game_points",
        "BBA's count opposite 1NT, scaled so it bids game from 10 (matchpoints)",
    ),
    t("bba_nt_imp_points", "`bba_nt_points` at IMPs"),
    t("bba_nt_imp_game_points", "`bba_nt_game_points` at IMPs"),
    t(
        "bba_stay_nt_points",
        "BBA's count after Stayman, no fit: 3NT rather than 2NT from 10 (matchpoints)",
    ),
    t("bba_stay_nt_imp_points", "`bba_stay_nt_points` at IMPs"),
    t(
        "bba_stay_raise_points",
        "BBA's count after Stayman, heart fit: invite rather than pass from 8 (matchpoints)",
    ),
    t(
        "bba_stay_raise_imp_points",
        "`bba_stay_raise_points` at IMPs",
    ),
    t(
        "bba_stay_game_points",
        "BBA's count after Stayman, heart fit: game rather than invite from 10 (matchpoints)",
    ),
    t("bba_stay_game_imp_points", "`bba_stay_game_points` at IMPs"),
    t(
        "bba_stay_sraise_points",
        "BBA's count after Stayman, spade fit: invite rather than pass from 8 (matchpoints)",
    ),
    t(
        "bba_stay_sraise_imp_points",
        "`bba_stay_sraise_points` at IMPs",
    ),
    t(
        "bba_stay_sgame_points",
        "BBA's count after Stayman, spade fit: game rather than invite from 10 (matchpoints)",
    ),
    t(
        "bba_stay_sgame_imp_points",
        "`bba_stay_sgame_points` at IMPs",
    ),
    t("controls", "controls: ace 2, king 1"),
    t("losers", "losing-trick count"),
    t(
        "quick_tricks",
        "quick tricks, whole part: A-K 2, A-Q 1½, A 1, K-Q 1, K-x ½ (Culbertson)",
    ),
    t(
        "bare_suits",
        "how many side suits are `bare(x)`: 2+ cards without the ace or king",
    ),
];
/// Functions of my own hand (bare, or `me.`).
const SELF_FUNCS: &[Term] = &[
    f(
        "tp",
        "(x)",
        "total points with x as trump: support points, shortness capped by trumps held",
    ),
    f(
        "keycards",
        "(x)",
        "aces plus the king of x (`keycards(N)`: the four aces)",
    ),
    f("has", "(rank, x)", "holds that card (A K Q J T) in x"),
    f(
        "quality",
        "(x)",
        "top honours (A, K, Q) in x: poor 0, fair 1, good 2, excellent 3",
    ),
    f("top5", "(x)", "how many of A K Q J T are held in x"),
    f("stop", "(x)", "a stopper in x: A, Kx, Qxx or Jxxx"),
    f(
        "bare",
        "(x)",
        "x is a side suit (not the agreed trump) of 2+ cards without the ace or king",
    ),
    f(
        "safe_level",
        "(x)",
        "the level our trumps make safe, the Law of Total Tricks: `we.fit(x).min - 6`",
    ),
];

/// Printed name of a strain for explanations.
pub fn strain_symbol(s: Strain) -> &'static str {
    match s {
        Strain::Clubs => "♣",
        Strain::Diamonds => "♦",
        Strain::Hearts => "♥",
        Strain::Spades => "♠",
        Strain::NoTrump => "NT",
    }
}

pub struct Ctx<'a> {
    pub pos: &'a Position,
    pub actor: Direction,
    /// The actor's exact hand, when choosing a call.
    pub hand: Option<&'a Facts>,
    /// The rule's module parameters.
    pub params: &'a HashMap<String, Val>,
    /// How total points are counted.
    pub valuation: Valuation,
    /// Another seat's knowledge capped by my own HCP (`seat_attr`), shared
    /// by the evaluations of one decision: building it clones the seat's
    /// whole knowledge, once per attribute read without the cache.
    pub private: Option<&'a PrivateCache>,
}

/// Capped knowledge by (seat, HCP bound), for one position (`Ctx::private`).
pub type PrivateCache = RefCell<HashMap<(usize, i32), Rc<SeatKnowledge>>>;

type R<T> = Result<T, String>;

impl<'a> Ctx<'a> {
    fn partner(&self) -> Direction {
        self.actor.partner()
    }

    /// Evaluate a condition.
    pub fn cond(&self, e: &Expr, b: &mut Bindings) -> R<Tri> {
        match self.eval(e, b)? {
            Val::Bool(t) => Ok(t),
            other => Err(format!("`{e}` is not a condition (it is {other:?})")),
        }
    }

    pub fn eval(&self, e: &Expr, b: &mut Bindings) -> R<Val> {
        Ok(match e {
            Expr::And { all } => {
                let mut t = Tri::True;
                for sub in all {
                    t = t.and(self.cond(sub, b)?);
                    if t == Tri::False {
                        break;
                    }
                }
                Val::Bool(t)
            }
            Expr::Or { any } => {
                let mut t = Tri::False;
                for sub in any {
                    t = t.or(self.cond(sub, b)?);
                    if t == Tri::True {
                        break;
                    }
                }
                Val::Bool(t)
            }
            Expr::Not { expr } => Val::Bool(self.cond(expr, b)?.negate()),
            Expr::Maybe { expr } => Val::Bool(Tri::from_bool(self.cond(expr, b)? != Tri::False)),
            Expr::Cmp { cmp, lhs, rhs } => {
                if let Some(e) = self.strength_as_points(*cmp, lhs, rhs)? {
                    return self.eval(&e, b);
                }
                let (l, r) = (self.eval(lhs, b)?, self.eval(rhs, b)?);
                Val::Bool(self.compare(*cmp, &l, &r)?)
            }
            Expr::InRange { expr, lo, hi } => {
                let n = self.num(&self.eval(expr, b)?)?;
                let lo = self.num(&self.eval(lo, b)?)?;
                let hi = self.num(&self.eval(hi, b)?)?;
                Val::Bool(if n.lo >= lo.hi && n.hi <= hi.lo {
                    Tri::True
                } else if n.hi < lo.lo || n.lo > hi.hi {
                    Tri::False
                } else {
                    Tri::Unknown
                })
            }
            Expr::InSet { expr, values } => {
                let n = self.num(&self.eval(expr, b)?)?;
                let hits = values
                    .iter()
                    .filter(|v| n.lo <= **v as i32 && **v as i32 <= n.hi)
                    .count();
                Val::Bool(match (n.as_point(), hits) {
                    (_, 0) => Tri::False,
                    (Some(_), _) => Tri::True,
                    _ => Tri::Unknown,
                })
            }
            Expr::Is { expr, what, not } => {
                let v = self.eval(expr, b)?;
                let t = match (what.as_str(), &v) {
                    // A named value: a card option (`style is relay`) or a
                    // symbol (`we.forcing is none`). Before the names below,
                    // which an option may share.
                    (name, Val::Sym(s)) => s == name,
                    ("suit", Val::Strain(s)) => *s != Strain::NoTrump,
                    ("suit", Val::Suit(_)) => true,
                    ("notrump", Val::Strain(s)) => *s == Strain::NoTrump,
                    ("none", Val::Nothing) => true,
                    ("suit" | "notrump" | "none", _) => false,
                    // An unset card option is none of its values.
                    (_, Val::Nothing) => false,
                    // Any other name is a suit: do both name the same one?
                    (name, _) => {
                        let other = self.eval(&path_expr(name), b)?;
                        match (strain_value(&v), strain_value(&other)) {
                            (Some(a), Some(c)) => a == c,
                            (None, None) => true,
                            _ => false,
                        }
                    }
                };
                Val::Bool(Tri::from_bool(t != *not))
            }
            Expr::Asked { kind } => {
                let ask = self
                    .pos
                    .side_state(self.actor)
                    .ask
                    .as_ref()
                    .filter(|a| a.by == self.partner());
                Val::Bool(Tri::from_bool(self.match_ask(ask, kind.as_deref(), b)?))
            }
            Expr::Answered { kind } => {
                let ans = self
                    .pos
                    .side_state(self.actor)
                    .answered
                    .as_ref()
                    .filter(|a| a.by == self.actor);
                Val::Bool(Tri::from_bool(self.match_ask(ans, kind.as_deref(), b)?))
            }
            Expr::Shape { pattern } => match self.hand {
                Some(f) => Val::Bool(Tri::from_bool(
                    f.shape_matches(pattern)
                        .ok_or_else(|| format!("bad shape `{pattern}`"))?,
                )),
                None => Val::Bool(Tri::Unknown),
            },
            Expr::Arith { arith, lhs, rhs } => {
                let l = self.num(&self.eval(lhs, b)?)?;
                let r = self.num(&self.eval(rhs, b)?)?;
                Val::Num(match arith {
                    ArithOp::Add => Range::new(l.lo + r.lo, l.hi + r.hi),
                    ArithOp::Sub => Range::new(l.lo - r.hi, l.hi - r.lo),
                })
            }
            Expr::Neg { expr } => {
                let n = self.num(&self.eval(expr, b)?)?;
                Val::Num(Range::new(-n.hi, -n.lo))
            }
            Expr::Int { value } => Val::Num(Range::point(*value as i32)),
            Expr::Call { call } => match self.concrete_call(call, b)? {
                Some(c) => Val::Call(c),
                None => Val::Nothing,
            },
            Expr::Path { path } => self.path(path, b)?,
        })
    }

    fn match_ask(
        &self,
        ask: Option<&crate::position::Ask>,
        kind: Option<&Expr>,
        b: &mut Bindings,
    ) -> R<bool> {
        let Some(ask) = ask else { return Ok(false) };
        let Some(kind) = kind else { return Ok(true) };
        let Expr::Path { path } = kind else {
            return Err(format!("`{kind}` is not a question kind"));
        };
        let [seg] = path.as_slice() else {
            return Err(format!("`{kind}` is not a question kind"));
        };
        if seg.name != ask.kind {
            return Ok(false);
        }
        for (i, arg) in seg.args.iter().flatten().enumerate() {
            let Some(actual) = ask.args.get(i) else {
                return Ok(false);
            };
            match arg {
                Expr::Path { path }
                    if path.len() == 1
                        && !b.contains_key(&path[0].name)
                        && path[0].args.is_none() =>
                {
                    // `transfer(M)` binds only a major, `transfer(m)` a minor.
                    let name = &path[0].name;
                    let suit = suit_of_strain(*actual);
                    if matches!(name.as_str(), "M" | "m")
                        && !suit.is_some_and(|s| crate::engine::var_allows(name, s))
                    {
                        return Ok(false);
                    }
                    let v = suit.map_or(Val::Strain(*actual), Val::Suit);
                    b.insert(name.clone(), v);
                }
                other => {
                    if strain_value(&self.eval(other, b)?) != Some(*actual) {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    /// `strength <op> <band>` as a condition on the actor's total points,
    /// given partner's range p in whole points (points when partner has
    /// shown points, else HCP): game needs 25 combined, slam 33.
    ///
    /// | band        | my points                      |
    /// |-------------|--------------------------------|
    /// | signoff     | at most 24 - p.max             |
    /// | invite      | 25 - p.max ..= 24 - p.min      |
    /// | game        | 25 - p.min ..= 32 - p.max      |
    /// | slam_invite | 33 - p.max ..= 32 - p.min      |
    /// | slam        | at least 33 - p.min            |
    ///
    /// Points compare by their whole part, so invite 8-9 means 8 to 9¾.
    /// A band can be empty (no invitation once partner's range is exact).
    fn strength_as_points(&self, cmp: CmpOp, lhs: &Expr, rhs: &Expr) -> R<Option<Expr>> {
        let name = |e: &Expr| match e {
            Expr::Path { path } if path.len() == 1 && path[0].args.is_none() => {
                Some(path[0].name.clone())
            }
            _ => None,
        };
        let kind = |s: &str| match s {
            "strength" => Some(0),
            "suit_strength" => Some(1),
            _ => None,
        };
        let (band, cmp, kind) = match (name(lhs), name(rhs)) {
            (Some(s), Some(band)) if kind(&s).is_some() => (band, cmp, kind(&s).unwrap()),
            (Some(band), Some(s)) if kind(&s).is_some() => (band, flip(cmp), kind(&s).unwrap()),
            _ => return Ok(None),
        };
        let i = BANDS
            .iter()
            .position(|b| *b == band)
            .ok_or_else(|| format!("`{band}` is not a strength band ({})", BANDS.join(", ")))?
            as i32;
        let (lo, hi) = self.band(i, kind);
        let measure = if kind == 0 { "points" } else { "suit_points" };
        let points = || Box::new(path_expr(measure));
        let int = |v: i32| Box::new(Expr::Int { value: v as i64 });
        let at_least = |v: i32| Expr::Cmp {
            cmp: CmpOp::Ge,
            lhs: points(),
            rhs: int(v),
        };
        let at_most = |v: i32| Expr::Cmp {
            cmp: CmpOp::Le,
            lhs: points(),
            rhs: int(v),
        };
        let within = Expr::InRange {
            expr: points(),
            lo: int(lo),
            hi: int(hi),
        };
        Ok(Some(match cmp {
            CmpOp::Eq => within,
            CmpOp::Ne => Expr::Not {
                expr: Box::new(within),
            },
            CmpOp::Ge => at_least(lo),
            CmpOp::Gt => at_least(hi + 1),
            CmpOp::Le => at_most(hi),
            CmpOp::Lt => at_most(lo - 1),
        }))
    }

    /// The whole-point range of strength band `i` (see
    /// `strength_as_points`).
    pub fn band(&self, i: i32, kind: usize) -> (i32, i32) {
        let p = self.pos.knowledge(self.partner()).whole_points(kind);
        match i {
            0 => (0, GAME - 1 - p.hi),
            1 => (GAME - p.hi, GAME - 1 - p.lo),
            2 => (GAME - p.lo, SLAM - 1 - p.hi),
            3 => (SLAM - p.hi, SLAM - 1 - p.lo),
            _ => (SLAM - p.lo, 40),
        }
    }

    /// The keycard counts partner has shown for `trump` (an answer such as
    /// `keycards(H) in 1|4`); all counts when partner has shown none.
    fn partner_keycards(&self, trump: Option<usize>) -> Vec<i32> {
        let mut possible: Vec<i32> = (0..=5).collect();
        let is_keycards = |e: &Expr| match e {
            Expr::Path { path } if path.len() == 1 && path[0].name == "keycards" => {
                match path[0].args.as_deref() {
                    Some([Expr::Path { path: a }]) => suit_index(&a[0].name) == trump,
                    _ => false,
                }
            }
            _ => false,
        };
        fn walk(e: &Expr, f: &dyn Fn(&Expr) -> bool, possible: &mut Vec<i32>) {
            match e {
                Expr::And { all } => all.iter().for_each(|x| walk(x, f, possible)),
                Expr::InSet { expr, values } if f(expr) => {
                    possible.retain(|v| values.contains(&(*v as i64)));
                }
                Expr::Cmp {
                    cmp: CmpOp::Eq,
                    lhs,
                    rhs,
                } if f(lhs) => {
                    if let Expr::Int { value } = **rhs {
                        possible.retain(|v| *v as i64 == value);
                    }
                }
                _ => {}
            }
        }
        for c in &self.pos.knowledge(self.partner()).constraints {
            walk(c, &is_keycards, &mut possible);
        }
        possible
    }

    /// A value as a number range. Suits count as the actor's length there.
    pub fn num(&self, v: &Val) -> R<Range> {
        match v {
            Val::Num(r) => Ok(*r),
            Val::Suit(s) => Ok(self.self_len(*s)),
            Val::Strain(st) => match suit_of_strain(*st) {
                Some(s) => Ok(self.self_len(s)),
                None => Err("notrump has no length".into()),
            },
            Val::Sym(w) => quality_level(w)
                .map(Range::point)
                .ok_or_else(|| format!("`{w}` is not a number")),
            // A condition in arithmetic counts 1 when it holds, 0 when it
            // does not (`we.fit(x).min - unfavourable`).
            Val::Bool(t) => Ok(match t {
                Tri::True => Range::point(1),
                Tri::False => Range::point(0),
                Tri::Unknown => Range::new(0, 1),
            }),
            other => Err(format!("expected a number, found {other:?}")),
        }
    }

    fn self_knowledge(&self) -> &SeatKnowledge {
        self.pos.knowledge(self.actor)
    }

    fn self_len(&self, s: usize) -> Range {
        match self.hand {
            Some(f) => Range::point(f.len[s]),
            None => self.self_knowledge().len[s],
        }
    }

    fn compare(&self, op: CmpOp, l: &Val, r: &Val) -> R<Tri> {
        let eq_only = |t: bool| -> R<Tri> {
            match op {
                CmpOp::Eq => Ok(Tri::from_bool(t)),
                CmpOp::Ne => Ok(Tri::from_bool(!t)),
                _ => Err(format!("cannot order {l:?} and {r:?}")),
            }
        };
        // A card field holding a bid ("through 2♥") compares as that bid.
        let as_bid = |v: &Val, other: &Val| match (v, other) {
            (Val::Sym(s), Val::Call(_)) => bid_of_text(s).map(Val::Call),
            _ => None,
        };
        let (l, r) = (
            &as_bid(l, r).unwrap_or_else(|| l.clone()),
            &as_bid(r, l).unwrap_or_else(|| r.clone()),
        );
        match (l, r) {
            (Val::Strain(a), Val::Strain(b)) => eq_only(a == b),
            // Bids are ordered by rank (`rho.last <= 2H`); other calls
            // only compare equal or not.
            (Val::Call(a), Val::Call(b)) => match (op, bid_rank(a), bid_rank(b)) {
                (CmpOp::Eq | CmpOp::Ne, _, _) => eq_only(a == b),
                (_, Some(x), Some(y)) => Ok(Tri::from_bool(match op {
                    CmpOp::Ge => x >= y,
                    CmpOp::Gt => x > y,
                    CmpOp::Le => x <= y,
                    _ => x < y,
                })),
                _ => eq_only(a == b),
            },
            (Val::Nothing, Val::Nothing) => eq_only(true),
            (Val::Nothing, _) | (_, Val::Nothing) => eq_only(false),
            (Val::Call(_), _) | (_, Val::Call(_)) => eq_only(false),
            (Val::Sym(a), Val::Sym(b)) => eq_only(a == b),
            _ => {
                let (a, b) = (self.num(l)?, self.num(r)?);
                Ok(match op {
                    CmpOp::Ge => cmp3(a.lo >= b.hi, a.hi < b.lo),
                    CmpOp::Gt => cmp3(a.lo > b.hi, a.hi <= b.lo),
                    CmpOp::Le => cmp3(a.hi <= b.lo, a.lo > b.hi),
                    CmpOp::Lt => cmp3(a.hi < b.lo, a.lo >= b.hi),
                    CmpOp::Eq => cmp3(a.as_point().is_some() && a == b, a.hi < b.lo || a.lo > b.hi),
                    CmpOp::Ne => cmp3(a.hi < b.lo || a.lo > b.hi, a.as_point().is_some() && a == b),
                })
            }
        }
    }

    fn path(&self, path: &[Segment], b: &mut Bindings) -> R<Val> {
        // The partnership sums, for a caller that did not expand them
        // (rules are expanded when they are flattened: `macros`).
        if let Some(e) = crate::macros::sugar(path) {
            return self.eval(&e?, b);
        }
        let first = &path[0];
        let (mut val, rest) = match first.name.as_str() {
            "partner" | "lho" | "rho" | "shown" | "me" | "we" | "they" if path.len() > 1 => {
                let seat = match first.name.as_str() {
                    "partner" => Some(self.partner()),
                    "lho" => Some(self.actor.next()),
                    "rho" => Some(self.actor.prev()),
                    _ => None,
                };
                let seg = &path[1];
                let v = match (first.name.as_str(), seat) {
                    (_, Some(d)) => self.seat_attr(d, seg, b)?,
                    ("shown", _) => {
                        self.knowledge_attr(self.self_knowledge(), self.actor, seg, b)?
                    }
                    ("me", _) => self.name(seg, b)?,
                    ("we", _) => self.we_attr(seg, b)?,
                    _ => self.they_attr(seg, b)?,
                };
                (v, &path[2..])
            }
            _ => (self.name(first, b)?, &path[1..]),
        };
        for seg in rest {
            val = match (seg.name.as_str(), &val) {
                ("min", Val::Num(r)) => Val::Num(Range::point(r.lo)),
                ("max", Val::Num(r)) => Val::Num(Range::point(r.hi)),
                (n, v) => return Err(format!("`.{n}` does not apply to {v:?}")),
            };
        }
        Ok(val)
    }

    fn suit_arg(&self, args: &[Expr], i: usize, b: &mut Bindings) -> R<Option<usize>> {
        let e = args.get(i).ok_or("missing argument")?;
        match self.eval(e, b)? {
            Val::Suit(s) => Ok(Some(s)),
            Val::Strain(s) => Ok(suit_of_strain(s)),
            Val::Nothing => Ok(None),
            v => Err(format!("`{e}` is not a suit ({v:?})")),
        }
    }

    /// A bare name: bindings, parameters, suits, the actor's hand, state.
    fn name(&self, seg: &Segment, b: &mut Bindings) -> R<Val> {
        let n = seg.name.as_str();
        if let Some(args) = &seg.args {
            return self.self_func(n, args, b);
        }
        if let Some(v) = b.get(n) {
            return Ok(v.clone());
        }
        if let Some(v) = self.params.get(n) {
            return Ok(v.clone());
        }
        if let Some(s) = suit_index(n) {
            return Ok(Val::Suit(s));
        }
        if n == "N" || n == "NT" {
            return Ok(Val::Strain(Strain::NoTrump));
        }
        if n == "strength" || n == "suit_strength" {
            return Err(format!(
                "`strength` is compared with a band: strength=invite, strength>=game ({})",
                BANDS.join(", ")
            ));
        }
        if n == "bare_suits" {
            let trump = self
                .pos
                .side_state(self.actor)
                .trump
                .and_then(suit_of_strain);
            return Ok(Val::Num(match self.hand {
                Some(f) => {
                    Range::point((0..4).filter(|&s| Some(s) != trump && f.bare(s)).count() as i32)
                }
                None => Range::new(0, 4),
            }));
        }
        if named(SELF_ATTRS, n) {
            return Ok(match self.hand {
                Some(f) => exact_attr(f, n, self.valuation),
                None => self.knowledge_attr(self.self_knowledge(), self.actor, seg, b)?,
            });
        }
        Ok(match n {
            "opening" => Val::Bool(Tri::from_bool(self.pos.is_opening())),
            // `me.last`: my own last call, as `partner.last` is theirs.
            "last" => self
                .pos
                .last_call_of(self.actor)
                .cloned()
                .map_or(Val::Nothing, Val::Call),
            "passed_hand" => Val::Bool(Tri::from_bool(self.pos.passed_hand(self.actor))),
            "bids" => Val::Num(Range::point(self.pos.bids_of(self.actor))),
            // `me.has_bid`: I have made a bid, not only passes or doubles.
            "has_bid" => Val::Bool(Tri::from_bool(self.pos.has_bid(self.actor))),
            "seat" => {
                let mut d = self.pos.dealer;
                let mut seat = 1;
                while d != self.actor {
                    d = d.next();
                    seat += 1;
                }
                Val::Num(Range::point(seat))
            }
            "vul" => Val::Bool(Tri::from_bool(self.pos.is_vulnerable(self.actor))),
            // Our side's last bid is game or higher (and it is our contract).
            "game_reached" => Val::Bool(Tri::from_bool(
                self.pos.side_acted(self.actor) && !self.pos.below_game(self.actor),
            )),
            "imps" => Val::Bool(Tri::from_bool(self.pos.is_imps())),
            "matchpoints" => Val::Bool(Tri::from_bool(!self.pos.is_imps())),
            "favourable" | "unfavourable" => {
                let (us, them) = (
                    self.pos.is_vulnerable(self.actor),
                    self.pos.is_vulnerable(self.actor.next()),
                );
                Val::Bool(Tri::from_bool(if n == "favourable" {
                    !us && them
                } else {
                    us && !them
                }))
            }
            // I place the contract: partner has put his hand in a range of
            // four points or less and mine is wider, or partner has
            // answered my question (Rick, 2026-09-30).
            "captain" => {
                let side = self.pos.side_state(self.actor);
                let width = |d: Direction| {
                    let r = self.pos.knowledge(d).whole_points(0);
                    r.hi - r.lo
                };
                let (p, me) = (width(self.partner()), width(self.actor));
                Val::Bool(Tri::from_bool(
                    side.answered.as_ref().is_some_and(|a| a.by == self.actor)
                        || (p <= CAPTAIN_WIDTH && p < me),
                ))
            }
            "trump" => self.we_attr(seg, b)?,
            // Judgment hooks. Placeholders until real evaluators are written.
            "slam_try" => {
                let we = self.num(&self.we_attr(&plain("hcp"), b)?)?;
                Val::Bool(cmp3(we.lo >= 31, we.hi < 31))
            }
            "grand_try" => {
                let we = self.num(&self.we_attr(&plain("hcp"), b)?)?;
                Val::Bool(cmp3(we.lo >= 35, we.hi < 35))
            }
            w if named(SYMBOLS, w) => Val::Sym(w.to_string()),
            _ => return Err(format!("unknown term `{n}`")),
        })
    }

    fn self_func(&self, n: &str, args: &[Expr], b: &mut Bindings) -> R<Val> {
        match n {
            // My own side of the control-bid dialogue (`me.denied(x)`).
            "denied" | "cued" => {
                return self.knowledge_attr(
                    self.self_knowledge(),
                    self.actor,
                    &Segment {
                        name: n.to_string(),
                        args: Some(args.to_vec()),
                    },
                    b,
                );
            }
            // Is the cheapest bid in x still below game in our agreed suit?
            "under_game" => {
                let s = self
                    .suit_arg(args, 0, b)?
                    .ok_or("under_game(x): x is a suit")?;
                // Game in the agreed suit: 4H 22, 4S 23, 5C 25, 5D 26; else 3NT.
                let game = match self.pos.side_state(self.actor).trump {
                    Some(Strain::Hearts) => 22,
                    Some(Strain::Spades) => 23,
                    Some(Strain::Clubs) => 25,
                    Some(Strain::Diamonds) => 26,
                    _ => 19,
                };
                return Ok(Val::Bool(Tri::from_bool(
                    self.pos.cheapest_rank(s).is_some_and(|r| r < game),
                )));
            }
            // The cheapest bid in x, as level * 5 + strain (C0 .. S3): for
            // `prefer 0 - cheapest_rank(x)`, the cheapest of several calls.
            "cheapest_rank" => {
                let s = self
                    .suit_arg(args, 0, b)?
                    .ok_or("cheapest_rank(x): x is a suit")?;
                return Ok(Val::Num(Range::point(
                    self.pos.cheapest_rank(s).unwrap_or(99),
                )));
            }
            _ => {}
        }
        if !named(SELF_FUNCS, n) {
            return Err(format!("unknown function `{n}`"));
        }
        if n == "safe_level" {
            let e = crate::macros::sugar(&[Segment {
                name: n.into(),
                args: Some(args.to_vec()),
            }])
            .expect("safe_level is sugar")?;
            return self.eval(&e, b);
        }
        let Some(f) = self.hand else {
            // Not tracked in knowledge yet.
            return Ok(match n {
                "has" | "stop" | "bare" => Val::Bool(Tri::Unknown),
                _ => Val::Num(Range::new(0, 40)),
            });
        };
        Ok(match n {
            "tp" => Val::Num(Range::point(f.total_points(self.suit_arg(args, 0, b)?))),
            "keycards" => Val::Num(Range::point(f.keycards(self.suit_arg(args, 0, b)?))),
            "quality" => Val::Num(Range::point(
                self.suit_arg(args, 0, b)?.map_or(0, |s| f.quality(s)),
            )),
            "top5" => Val::Num(Range::point(
                self.suit_arg(args, 0, b)?.map_or(0, |s| f.top5(s)),
            )),
            "stop" => Val::Bool(Tri::from_bool(
                self.suit_arg(args, 0, b)?.is_some_and(|s| f.stop(s)),
            )),
            "bare" => {
                let trump = self
                    .pos
                    .side_state(self.actor)
                    .trump
                    .and_then(suit_of_strain);
                Val::Bool(Tri::from_bool(
                    self.suit_arg(args, 0, b)?
                        .is_some_and(|s| Some(s) != trump && f.bare(s)),
                ))
            }
            "has" => {
                let rank = match args.first() {
                    Some(Expr::Path { path }) => rank_value(&path[0].name),
                    _ => None,
                }
                .ok_or("has(rank, suit): rank is A K Q J or T")?;
                Val::Bool(Tri::from_bool(
                    self.suit_arg(args, 1, b)?.is_some_and(|s| f.has(s, rank)),
                ))
            }
            _ => unreachable!(),
        })
    }

    fn knowledge_attr(
        &self,
        k: &SeatKnowledge,
        seat: Direction,
        seg: &Segment,
        b: &mut Bindings,
    ) -> R<Val> {
        let n = seg.name.as_str();
        if let Some(s) = suit_index(n) {
            return Ok(Val::Num(k.len[s]));
        }
        if let Some(Val::Suit(s)) = b.get(n) {
            return Ok(Val::Num(k.len[*s]));
        }
        // `partner.trump`: the length in our agreed suit.
        if n == "trump" {
            return match self
                .pos
                .side_state(self.actor)
                .trump
                .and_then(suit_of_strain)
            {
                Some(s) => Ok(Val::Num(k.len[s])),
                None => Err("no trump suit agreed".into()),
            };
        }
        Ok(match n {
            "hcp" => Val::Num(k.hcp),
            // Nobody tracks another hand's tens.
            "tens" => Val::Num(Range::new(0, 4)),
            "points" => Val::Num(k.whole_points(0)),
            "suit_points" => Val::Num(k.whole_points(1)),
            "balanced" => Val::Bool(k.balanced),
            "shortest" => Val::Num(Range::new(
                k.len.iter().map(|r| r.lo).min().unwrap_or(0),
                k.len.iter().map(|r| r.hi).min().unwrap_or(13),
            )),
            "longest" => Val::Num(Range::new(
                k.len.iter().map(|r| r.lo).max().unwrap_or(0),
                k.len.iter().map(|r| r.hi).max().unwrap_or(13),
            )),
            "second_longest" => {
                let mut los: Vec<i32> = k.len.iter().map(|r| r.lo).collect();
                let mut his: Vec<i32> = k.len.iter().map(|r| r.hi).collect();
                los.sort_unstable_by(|a, b| b.cmp(a));
                his.sort_unstable_by(|a, b| b.cmp(a));
                Val::Num(Range::new(los[1], his[1]))
            }
            "length_points" => Val::Num(Range::new(
                k.len.iter().map(|r| (r.lo - 4).max(0)).sum(),
                k.len.iter().map(|r| (r.hi - 4).max(0)).sum(),
            )),
            "last" => self
                .pos
                .last_call_of(seat)
                .cloned()
                .map_or(Val::Nothing, Val::Call),
            "opened" => Val::Bool(Tri::from_bool(self.pos.opener() == Some(seat))),
            "bids" => Val::Num(Range::point(self.pos.bids_of(seat))),
            "has_bid" => Val::Bool(Tri::from_bool(self.pos.has_bid(seat))),
            // `partner.jumped`: their last bid skipped a level in its strain.
            "jumped" => Val::Bool(Tri::from_bool(self.pos.jumped(seat))),
            // `partner.bypassed(x)`: their last bid skipped a bid in x that
            // was available (Position::bypassed). Public, like `last`.
            // The control-bid dialogue: `denied(x)`, a suit this seat skipped;
            // `cued(x)`, a suit it has shown a control in.
            "denied" | "cued" => {
                let st = &self.pos.sides[crate::position::side(seat)];
                let mask = if n == "denied" { st.denied } else { st.cued }[seat.to_index()];
                match self.suit_arg(seg.args.as_deref().unwrap_or(&[]), 0, b)? {
                    Some(s) => Val::Bool(Tri::from_bool(mask & (1 << s) != 0)),
                    None => Val::Bool(Tri::False),
                }
            }
            // An optional second argument, a call, is where the ladder
            // starts: `partner.bypassed(C, 3{trump})`.
            "bypassed" => {
                let args = seg.args.as_deref().unwrap_or(&[]);
                let floor = match args.get(1) {
                    Some(e) => match self.eval(e, b)? {
                        Val::Call(c) => Some(c),
                        v => return Err(format!("bypassed: `{e}` is not a call ({v:?})")),
                    },
                    None => None,
                };
                match self.suit_arg(args, 0, b)? {
                    Some(s) => Val::Bool(Tri::from_bool(self.pos.bypassed_above(
                        seat,
                        s,
                        floor.as_ref(),
                    ))),
                    None => Val::Bool(Tri::False),
                }
            }
            // Public history: a natural bid in the strain, ever.
            "named" => Val::Bool(Tri::from_bool(
                self.pos.named(seat, self.strain_arg(seg, b)?),
            )),
            "has" | "stop" | "semibalanced" | "bare" => Val::Bool(Tri::Unknown),
            // Support points are known when a raise showed them.
            "tp" => match self.suit_arg(seg.args.as_deref().unwrap_or(&[]), 0, b)? {
                Some(t) => Val::Num(k.tp[t]),
                None => Val::Num(k.hcp),
            },
            "keycards" | "controls" | "losers" | "quality" | "top5" | "quick_tricks" => {
                Val::Num(Range::new(0, 40))
            }
            "bare_suits" => Val::Num(Range::new(0, 4)),
            // BBA's fitted counts (Facts::bba_named) are about the actor's
            // own hand; for anyone else they are unknown.
            n if n.starts_with("bba_") => Val::Num(Range::new(0, 40)),
            _ => return Err(format!("unknown attribute `{n}`")),
        })
    }

    fn seat_attr(&self, d: Direction, seg: &Segment, b: &mut Bindings) -> R<Val> {
        // Choosing a call, I also know my own HCP: another seat holds at
        // most 40 minus mine and the other two seats' minimums. Narrowing a
        // copy lets that seat's earlier disjunctions collapse too (partner's
        // takeout double is the shape one when he cannot hold 17).
        if let (Some(f), true) = (self.hand, d != self.actor) {
            let k = self.pos.knowledge(d);
            let others: i32 = Direction::ALL
                .iter()
                .filter(|&&t| t != d && t != self.actor)
                .map(|&t| self.pos.knowledge(t).hcp.lo)
                .sum();
            let bound = 40 - f.hcp - others;
            if bound < k.hcp.hi {
                let capped = || {
                    let mut private = k.clone();
                    private.add(hcp_at_most(bound));
                    Rc::new(private)
                };
                let private = match self.private {
                    Some(cache) => {
                        let key = (d.to_index(), bound);
                        let hit = cache.borrow().get(&key).cloned();
                        hit.unwrap_or_else(|| {
                            let p = capped();
                            cache.borrow_mut().insert(key, p.clone());
                            p
                        })
                    }
                    None => capped(),
                };
                return self.knowledge_attr(&private, d, seg, b);
            }
        }
        self.knowledge_attr(self.pos.knowledge(d), d, seg, b)
    }

    /// A strain argument: a suit, a suit variable, `N` or `trump`.
    fn strain_arg(&self, seg: &Segment, b: &mut Bindings) -> R<Strain> {
        let e = match seg.args.as_deref() {
            Some([e]) => e,
            _ => return Err(format!("`{}(x)` takes one strain", seg.name)),
        };
        if matches!(e, Expr::Path { path } if path.len() == 1 && path[0].name == "N") {
            return Ok(Strain::NoTrump);
        }
        match self.eval(e, b)? {
            Val::Suit(s) => Ok(strain_of_suit(s)),
            Val::Strain(st) => Ok(st),
            v => Err(format!("`{e}` is not a strain ({v:?})")),
        }
    }

    fn we_attr(&self, seg: &Segment, b: &mut Bindings) -> R<Val> {
        let side = self.pos.side_state(self.actor);
        Ok(match seg.name.as_str() {
            "trump" => side.trump.map_or(Val::Nothing, Val::Strain),
            "named" => {
                let st = self.strain_arg(seg, b)?;
                Val::Bool(Tri::from_bool(
                    self.pos.named(self.actor, st) || self.pos.named(self.partner(), st),
                ))
            }
            "forcing" => Val::Sym(
                match side.forcing {
                    Forcing::None => "none",
                    Forcing::Round => "round",
                    Forcing::Game => "game",
                }
                .into(),
            ),
            "gf" => Val::Bool(Tri::from_bool(side.forcing == Forcing::Game)),
            "hcp" => {
                let mine = self.num(&self.name(&plain("hcp"), b)?)?;
                let theirs = self.pos.knowledge(self.partner()).hcp;
                Val::Num(Range::new(mine.lo + theirs.lo, mine.hi + theirs.hi))
            }
            // The partnership's support points with `t` as trump: what the
            // slam decision is made on, since a fit is worth more than the
            // high cards say.
            "tp" => {
                let args = seg.args.as_deref().unwrap_or(&[]);
                let mine = self.num(&self.self_func("tp", args, b)?)?;
                let theirs = match self.suit_arg(args, 0, b)? {
                    Some(t) => self.pos.knowledge(self.partner()).tp[t],
                    None => self.pos.knowledge(self.partner()).hcp,
                };
                Val::Num(Range::new(mine.lo + theirs.lo, mine.hi + theirs.hi))
            }
            "keycards" => {
                let mine = self.num(&self.self_func(
                    "keycards",
                    seg.args.as_deref().unwrap_or(&[]),
                    b,
                )?)?;
                // Partner's answer ("1 or 4"), limited by the deck: five
                // keycards in all, so "1 or 4" facing my 2 can only be 1.
                // With no trump suit (`keycards(N)`) they are the four aces.
                let suit = self.suit_arg(seg.args.as_deref().unwrap_or(&[]), 0, b)?;
                let deck = if suit.is_some() { 5 } else { 4 };
                let theirs = self.partner_keycards(suit);
                let fits: Vec<i32> = (0..=deck)
                    .filter(|v| theirs.contains(v) && mine.lo + v <= deck)
                    .collect();
                match (fits.first(), fits.last()) {
                    (Some(lo), Some(hi)) => Val::Num(Range::new(mine.lo + lo, mine.hi + hi)),
                    _ => Val::Num(Range::new(mine.lo, deck)),
                }
            }
            n => return Err(format!("unknown attribute `we.{n}`")),
        })
    }

    fn they_attr(&self, seg: &Segment, b: &mut Bindings) -> R<Val> {
        let opp = self.actor.next();
        // Both opponents' ranges added: `they.hcp`, `they.fit(x)`.
        let sum = |a: Val, c: Val| -> R<Val> {
            let (a, c) = (self.num(&a)?, self.num(&c)?);
            Ok(Val::Num(Range::new(a.lo + c.lo, a.hi + c.hi)))
        };
        // The opponents' last bid, level and strain.
        let last_bid = || {
            (0..self.pos.calls.len())
                .rev()
                .filter(|&i| {
                    crate::position::side(self.pos.caller(i)) != crate::position::side(self.actor)
                })
                .find_map(|i| match self.pos.calls[i] {
                    Call::Bid { level, strain } => Some((level, strain)),
                    _ => None,
                })
        };
        Ok(match seg.name.as_str() {
            "bid" => Val::Bool(Tri::from_bool(self.pos.side_acted(opp))),
            "vul" => Val::Bool(Tri::from_bool(self.pos.is_vulnerable(opp))),
            "hcp" => sum(
                self.seat_attr(opp, seg, b)?,
                self.seat_attr(opp.partner(), seg, b)?,
            )?,
            "fit" => {
                let x = match seg.args.as_deref() {
                    Some([x]) => x,
                    _ => return Err("`they.fit(x)` takes one suit".into()),
                };
                let suit = match self.eval(x, b)? {
                    Val::Suit(s) => s,
                    Val::Strain(st) => {
                        suit_of_strain(st).ok_or("they.fit: notrump has no length")?
                    }
                    v => return Err(format!("they.fit: `{x}` is not a suit ({v:?})")),
                };
                let len = Segment {
                    name: SUITS[suit].into(),
                    args: None,
                };
                sum(
                    self.seat_attr(opp, &len, b)?,
                    self.seat_attr(opp.partner(), &len, b)?,
                )?
            }
            "level" => last_bid().map_or(Val::Nothing, |(l, _)| Val::Num(Range::point(l as i32))),
            "strain" => last_bid().map_or(Val::Nothing, |(_, s)| match suit_of_strain(s) {
                Some(x) => Val::Suit(x),
                None => Val::Strain(s),
            }),
            // One of their last calls is not a pass (a seat yet to call
            // counts as still bidding).
            "still_bidding" => Val::Bool(Tri::from_bool(
                self.pos.last_call_of(opp) != Some(&Call::Pass)
                    || self.pos.last_call_of(opp.partner()) != Some(&Call::Pass),
            )),
            "game_reached" => Val::Bool(Tri::from_bool(
                self.pos.side_acted(opp) && !self.pos.below_game(opp),
            )),
            n => return Err(format!("unknown attribute `they.{n}`")),
        })
    }

    /// A rule's call with variables filled in. `None` when it names a strain
    /// that has no value (no trump agreed).
    pub fn concrete_call(&self, spec: &CallSpec, b: &mut Bindings) -> R<Option<Call>> {
        Ok(Some(match spec {
            CallSpec::Pass => Call::Pass,
            CallSpec::Double => Call::Double,
            CallSpec::Redouble => Call::Redouble,
            CallSpec::Bid { level, strain } => {
                let strain = match strain {
                    StrainSpec::Lit(s) => strain_from_spec(*s),
                    StrainSpec::Var(v) | StrainSpec::Interp(v) => {
                        match self.eval(&path_expr(v), b)? {
                            Val::Suit(s) => strain_of_suit(s),
                            Val::Strain(s) => s,
                            Val::Nothing => return Ok(None),
                            other => return Err(format!("`{v}` is not a strain ({other:?})")),
                        }
                    }
                };
                Call::Bid {
                    level: *level,
                    strain,
                }
            }
            CallSpec::Any | CallSpec::Relative { .. } => return Ok(None),
        }))
    }

    /// Replace `{name}` in an explanation with its value.
    pub fn interpolate(&self, text: &str, b: &mut Bindings) -> String {
        let mut out = String::new();
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            out.push_str(&rest[..open]);
            let Some(close) = rest[open..].find('}') else {
                break;
            };
            let name = &rest[open + 1..open + close];
            if let Some(i) = BANDS.iter().position(|x| *x == name) {
                let (lo, hi) = self.band(i as i32, 0);
                out.push_str(&match (lo.max(0), hi) {
                    (lo, hi) if lo > hi => "—".to_string(),
                    (lo, 40) => format!("{lo}+"),
                    (lo, hi) if lo == hi => lo.to_string(),
                    (lo, hi) => format!("{lo}-{hi}"),
                });
                rest = &rest[open + close + 1..];
                continue;
            }
            let shown = match self.eval(&path_expr(name), b) {
                Ok(Val::Suit(s)) => strain_symbol(strain_of_suit(s)).to_string(),
                Ok(Val::Strain(s)) => strain_symbol(s).to_string(),
                Ok(Val::Num(r)) => r.to_string(),
                _ => format!("{{{name}}}"),
            };
            out.push_str(&shown);
            rest = &rest[open + close + 1..];
        }
        out.push_str(rest);
        out
    }

    /// `e` with every condition that does not depend on the hand and is
    /// known from the auction replaced by its value, for evaluating it on
    /// many hands (`descriptiveness_of`). Exact: a hand only narrows what
    /// the auction says about the other seats, so what is known without it
    /// stays so. Not under `maybe`, which a narrower range can turn false.
    pub fn fold_public(&self, e: &Expr, b: &mut Bindings) -> Expr {
        let known = |e: &Expr, b: &mut Bindings| {
            if hand_dependent(e, b) || has_maybe(e) {
                return None;
            }
            match self.eval(e, b) {
                Ok(Val::Bool(Tri::True)) => Some(konst(true)),
                Ok(Val::Bool(Tri::False)) => Some(konst(false)),
                _ => None,
            }
        };
        match e {
            Expr::And { .. }
            | Expr::Or { .. }
            | Expr::Not { .. }
            | Expr::Cmp { .. }
            | Expr::Is { .. }
            | Expr::InRange { .. }
            | Expr::InSet { .. }
            | Expr::Path { .. } => match known(e, b) {
                Some(k) => k,
                None => self.fold_parts(e, b),
            },
            _ => self.fold_parts(e, b),
        }
    }

    /// `fold_public` below the top of `e`.
    fn fold_parts(&self, e: &Expr, b: &mut Bindings) -> Expr {
        let rec = |x: &Expr, b: &mut Bindings| Box::new(self.fold_public(x, b));
        match e {
            Expr::And { all } => Expr::And {
                all: all.iter().map(|x| *rec(x, b)).collect(),
            },
            Expr::Or { any } => Expr::Or {
                any: any.iter().map(|x| *rec(x, b)).collect(),
            },
            Expr::Not { expr } => Expr::Not { expr: rec(expr, b) },
            Expr::Cmp { cmp, lhs, rhs } => Expr::Cmp {
                cmp: *cmp,
                lhs: rec(lhs, b),
                rhs: rec(rhs, b),
            },
            Expr::Arith { arith, lhs, rhs } => Expr::Arith {
                arith: *arith,
                lhs: rec(lhs, b),
                rhs: rec(rhs, b),
            },
            _ => e.clone(),
        }
    }

    /// Rewrite a `shows` (or denial) as a constraint on the actor's own hand:
    /// variables become suit letters, parameters and other seats' values
    /// become numbers, and state conditions become true or false. Terms that
    /// cannot be resolved to a number are dropped (treated as true), which
    /// loses information but never excludes a possible hand.
    pub fn resolve(&self, e: &Expr, b: &mut Bindings) -> Expr {
        self.resolve_with(e, b, &mut false)
    }

    /// `resolve`, returning `None` if any term had to be dropped. Needed
    /// wherever the result is negated: dropping a term there would claim more
    /// than is known.
    pub fn resolve_exact(&self, e: &Expr, b: &mut Bindings) -> Option<Expr> {
        let mut lossy = false;
        let r = self.resolve_with(e, b, &mut lossy);
        (!lossy).then_some(r)
    }

    fn resolve_with(&self, e: &Expr, b: &mut Bindings, lossy: &mut bool) -> Expr {
        let mut dropped = || {
            *lossy = true;
            konst(true)
        };
        match e {
            Expr::And { all } => Expr::And {
                all: all.iter().map(|x| self.resolve_with(x, b, lossy)).collect(),
            },
            Expr::Or { any } => Expr::Or {
                any: any.iter().map(|x| self.resolve_with(x, b, lossy)).collect(),
            },
            Expr::Not { expr } => {
                // A term dropped inside a negation would turn "not (unknown)"
                // into "not true" = false. Drop the whole negation instead.
                let mut inner_lossy = false;
                let inner = self.resolve_with(expr, b, &mut inner_lossy);
                if inner_lossy {
                    *lossy = true;
                    konst(true)
                } else {
                    Expr::Not {
                        expr: Box::new(inner),
                    }
                }
            }
            Expr::Shape { .. } => e.clone(),
            Expr::Cmp { cmp, lhs, rhs } => {
                if let Ok(Some(e)) = self.strength_as_points(*cmp, lhs, rhs) {
                    return self.resolve_with(&e, b, lossy);
                }
                // A comparison that does not involve the actor's own hand is
                // a fact about the auction: fold it to a constant if known.
                if !mentions_self(e, b, self.params) {
                    match self.cond(e, b) {
                        Ok(Tri::True) => return konst(true),
                        Ok(Tri::False) => return konst(false),
                        _ => {}
                    }
                }
                match (self.term(lhs, b), self.term(rhs, b)) {
                    (Some(l), Some(r)) => Expr::Cmp {
                        cmp: *cmp,
                        lhs: Box::new(l),
                        rhs: Box::new(r),
                    },
                    _ => dropped(),
                }
            }
            Expr::InRange { expr, lo, hi } => {
                match (self.term(expr, b), self.term(lo, b), self.term(hi, b)) {
                    (Some(x), Some(l), Some(h)) => Expr::InRange {
                        expr: Box::new(x),
                        lo: Box::new(l),
                        hi: Box::new(h),
                    },
                    _ => dropped(),
                }
            }
            Expr::InSet { expr, values } => match self.term(expr, b) {
                Some(x) => Expr::InSet {
                    expr: Box::new(x),
                    values: values.clone(),
                },
                None => dropped(),
            },
            Expr::Path { path }
                if path.len() == 1
                    && path[0].args.is_none()
                    && named(SELF_ATTRS, &path[0].name) =>
            {
                e.clone()
            }
            _ => match self.cond(e, b) {
                Ok(Tri::False) => konst(false),
                Ok(Tri::True) => konst(true),
                _ => dropped(),
            },
        }
    }

    /// A term of a comparison, resolved (see `resolve`).
    fn term(&self, e: &Expr, b: &mut Bindings) -> Option<Expr> {
        match e {
            Expr::Int { .. } => Some(e.clone()),
            Expr::Path { path } if path.len() == 1 => {
                let seg = &path[0];
                let n = seg.name.as_str();
                if let Some(args) = &seg.args {
                    let args = args
                        .iter()
                        .map(|a| match self.eval(a, b) {
                            Ok(Val::Suit(s)) => Some(path_expr(SUITS[s])),
                            // Notrump (`keycards(N)`: aces only) stays as written.
                            Ok(Val::Strain(st)) => Some(
                                suit_of_strain(st)
                                    .map_or_else(|| a.clone(), |s| path_expr(SUITS[s])),
                            ),
                            _ => Some(a.clone()),
                        })
                        .collect::<Option<Vec<_>>>()?;
                    return Some(Expr::Path {
                        path: vec![Segment {
                            name: n.into(),
                            args: Some(args),
                        }],
                    });
                }
                if named(SELF_ATTRS, n) || suit_index(n).is_some() {
                    return Some(e.clone());
                }
                match self.eval(e, b).ok()? {
                    Val::Suit(s) => Some(path_expr(SUITS[s])),
                    // `trump`: the agreed suit, a length like a suit letter.
                    Val::Strain(st) => suit_of_strain(st).map(|s| path_expr(SUITS[s])),
                    Val::Num(r) => r.as_point().map(|v| Expr::Int { value: v as i64 }),
                    Val::Bool(Tri::True) => Some(Expr::Int { value: 1 }),
                    Val::Bool(Tri::False) => Some(Expr::Int { value: 0 }),
                    _ => None,
                }
            }
            Expr::Arith { arith, lhs, rhs } => {
                let (l, r) = (self.term(lhs, b)?, self.term(rhs, b)?);
                match (&l, &r) {
                    (Expr::Int { value: a }, Expr::Int { value: c }) => Some(Expr::Int {
                        value: if *arith == ArithOp::Add { a + c } else { a - c },
                    }),
                    _ => Some(Expr::Arith {
                        arith: *arith,
                        lhs: Box::new(l),
                        rhs: Box::new(r),
                    }),
                }
            }
            Expr::Neg { expr } => match self.term(expr, b)? {
                Expr::Int { value } => Some(Expr::Int { value: -value }),
                other => Some(Expr::Neg {
                    expr: Box::new(other),
                }),
            },
            _ => match self.eval(e, b).ok()? {
                Val::Num(r) => r.as_point().map(|v| Expr::Int { value: v as i64 }),
                Val::Bool(Tri::True) => Some(Expr::Int { value: 1 }),
                Val::Bool(Tri::False) => Some(Expr::Int { value: 0 }),
                _ => None,
            },
        }
    }
}

/// Can `when` be ruled out from public knowledge alone? Its public parts
/// must be known true: a conjunction is impossible when any part is, a
/// disjunction when every branch is, and a condition on the caller's own
/// hand is never impossible (the hand is not known). So `style is bba, H>=4`
/// is ruled out when the style is not bba, whatever the hand.
pub fn publicly_impossible(ctx: &Ctx, e: &Expr, b: &Bindings) -> bool {
    match e {
        Expr::And { all } => all.iter().any(|x| publicly_impossible(ctx, x, b)),
        Expr::Or { any } => any.iter().all(|x| publicly_impossible(ctx, x, b)),
        _ if hand_dependent(e, b) => false,
        _ => ctx.cond(e, &mut b.clone()) != Ok(Tri::True),
    }
}

/// Does the truth of `e` depend on the actor's own hand? (Everything else
/// in a condition is public: what each seat has shown, and the state.) A
/// `when` that does not depend on the hand, and is not known true, cannot
/// be what the caller relied on.
pub fn hand_dependent(e: &Expr, b: &Bindings) -> bool {
    match e {
        Expr::And { all } => all.iter().any(|x| hand_dependent(x, b)),
        Expr::Or { any } => any.iter().any(|x| hand_dependent(x, b)),
        Expr::Not { expr } | Expr::Maybe { expr } | Expr::Neg { expr } => hand_dependent(expr, b),
        Expr::Cmp { lhs, rhs, .. } => hand_dependent(lhs, b) || hand_dependent(rhs, b),
        Expr::InRange { expr, lo, hi } => {
            hand_dependent(expr, b) || hand_dependent(lo, b) || hand_dependent(hi, b)
        }
        Expr::InSet { expr, .. } => hand_dependent(expr, b),
        Expr::Arith { lhs, rhs, .. } => hand_dependent(lhs, b) || hand_dependent(rhs, b),
        Expr::Shape { .. } => true,
        Expr::Is { .. } | Expr::Asked { .. } | Expr::Answered { .. } => false,
        Expr::Int { .. } | Expr::Call { .. } => false,
        Expr::Path { path } => {
            let first = path[0].name.as_str();
            match first {
                "me" => true,
                // Every `we.` term that adds my own hand to partner's
                // range. One missing from this list is not an error: the
                // condition is judged with no hand, comes out false, and
                // the candidate vanishes silently.
                "we" => path.get(1).is_some_and(|s| {
                    matches!(
                        s.name.as_str(),
                        "hcp" | "keycards" | "points" | "tp" | "fit"
                    )
                }),
                "partner" | "lho" | "rho" | "shown" | "they" => false,
                _ if path.len() > 1 => false,
                n => {
                    named(SELF_ATTRS, n)
                        || named(SELF_FUNCS, n)
                        || matches!(n, "slam_try" | "grand_try" | "strength" | "suit_strength")
                        || suit_index(n).is_some()
                        || matches!(b.get(n), Some(Val::Suit(_)))
                        // Bare `trump` in a comparison or a sum is my
                        // length in the agreed suit (`trump > partner.trump.min`).
                        || n == "trump"
                }
            }
        }
    }
}

/// Does `e` count a condition in arithmetic (`we.fit(x).min +
/// doubler_four(x)`)?
pub fn counts_conditions(e: &Expr) -> bool {
    let condition = |x: &Expr| {
        matches!(
            x,
            Expr::And { .. }
                | Expr::Or { .. }
                | Expr::Not { .. }
                | Expr::Maybe { .. }
                | Expr::Cmp { .. }
                | Expr::Is { .. }
                | Expr::InRange { .. }
                | Expr::InSet { .. }
        ) || matches!(x, Expr::Path { path } if path.iter().any(|s| matches!(
            s.name.as_str(),
            "vul" | "favourable" | "unfavourable" | "imps" | "matchpoints" | "balanced"
                | "opening" | "game_reached" | "captain" | "still_bidding" | "bid"
        )))
    };
    match e {
        Expr::Arith { lhs, rhs, .. } => {
            condition(lhs) || condition(rhs) || counts_conditions(lhs) || counts_conditions(rhs)
        }
        Expr::And { all } => all.iter().any(counts_conditions),
        Expr::Or { any } => any.iter().any(counts_conditions),
        Expr::Not { expr } | Expr::Maybe { expr } | Expr::Neg { expr } => counts_conditions(expr),
        Expr::Cmp { lhs, rhs, .. } => counts_conditions(lhs) || counts_conditions(rhs),
        Expr::InRange { expr, lo, hi } => {
            counts_conditions(expr) || counts_conditions(lo) || counts_conditions(hi)
        }
        _ => false,
    }
}

fn has_maybe(e: &Expr) -> bool {
    match e {
        Expr::Maybe { .. } => true,
        Expr::And { all } => all.iter().any(has_maybe),
        Expr::Or { any } => any.iter().any(has_maybe),
        Expr::Not { expr } | Expr::Neg { expr } => has_maybe(expr),
        Expr::Cmp { lhs, rhs, .. } | Expr::Arith { lhs, rhs, .. } => {
            has_maybe(lhs) || has_maybe(rhs)
        }
        Expr::InRange { expr, lo, hi } => has_maybe(expr) || has_maybe(lo) || has_maybe(hi),
        Expr::InSet { expr, .. } | Expr::Is { expr, .. } => has_maybe(expr),
        Expr::Path { path } => path.iter().any(|s| s.args.iter().flatten().any(has_maybe)),
        _ => false,
    }
}

/// The terms in `e` that read the board's conditions (vulnerability,
/// scoring, seat), which the calls so far do not fix: `vul`,
/// `unfavourable`, `they.vul`, ...
pub fn board_terms(e: &Expr, out: &mut Vec<Expr>) {
    match e {
        Expr::And { all } => all.iter().for_each(|x| board_terms(x, out)),
        Expr::Or { any } => any.iter().for_each(|x| board_terms(x, out)),
        Expr::Not { expr } | Expr::Maybe { expr } | Expr::Neg { expr } => board_terms(expr, out),
        Expr::Cmp { lhs, rhs, .. } | Expr::Arith { lhs, rhs, .. } => {
            board_terms(lhs, out);
            board_terms(rhs, out);
        }
        Expr::InRange { expr, lo, hi } => {
            board_terms(expr, out);
            board_terms(lo, out);
            board_terms(hi, out);
        }
        Expr::InSet { expr, .. } | Expr::Is { expr, .. } => board_terms(expr, out),
        Expr::Path { path } => {
            let board = path.iter().any(|s| {
                matches!(
                    s.name.as_str(),
                    "vul" | "favourable" | "unfavourable" | "imps" | "matchpoints" | "seat"
                )
            });
            if board && !out.contains(e) {
                out.push(e.clone());
            }
            for s in path {
                s.args.iter().flatten().for_each(|x| board_terms(x, out));
            }
        }
        Expr::Asked { .. }
        | Expr::Answered { .. }
        | Expr::Shape { .. }
        | Expr::Int { .. }
        | Expr::Call { .. } => {}
    }
}

/// Does `e` refer to the actor's own hand (so it must not be folded to a
/// constant from knowledge)?
fn mentions_self(e: &Expr, b: &Bindings, params: &HashMap<String, Val>) -> bool {
    match e {
        Expr::Path { path } => {
            let n = path[0].name.as_str();
            if path.len() == 1 && path[0].args.is_some() {
                return named(SELF_FUNCS, n);
            }
            path.len() == 1 && !params.contains_key(n) && !b.contains_key(n) && named(SELF_ATTRS, n)
        }
        Expr::Cmp { lhs, rhs, .. } => {
            // A suit or suit variable in a comparison is a length.
            let suitish = |x: &Expr| match x {
                Expr::Path { path } if path.len() == 1 && path[0].args.is_none() => {
                    suit_index(&path[0].name).is_some()
                        || matches!(b.get(&path[0].name), Some(Val::Suit(_)))
                        || (path[0].name == "trump" && !b.contains_key("trump"))
                }
                _ => false,
            };
            suitish(lhs)
                || suitish(rhs)
                || mentions_self(lhs, b, params)
                || mentions_self(rhs, b, params)
        }
        Expr::Arith { lhs, rhs, .. } => {
            mentions_self(lhs, b, params) || mentions_self(rhs, b, params)
        }
        Expr::Neg { expr } | Expr::Not { expr } => mentions_self(expr, b, params),
        Expr::Shape { .. } => true,
        _ => false,
    }
}

/// A suit or strain as a strain, for `is` comparisons.
fn strain_value(v: &Val) -> Option<Strain> {
    match v {
        Val::Suit(s) => Some(strain_of_suit(*s)),
        Val::Strain(s) => Some(*s),
        _ => None,
    }
}

/// A card's text as a bid ("2♥", "2H", "3NT"), for comparing a call with
/// a card field such as `doubles.support.through`.
fn bid_of_text(s: &str) -> Option<Call> {
    let t: String = s
        .trim()
        .chars()
        .map(|c| match c {
            '♠' => 'S',
            '♥' => 'H',
            '♦' => 'D',
            '♣' => 'C',
            c => c,
        })
        .collect();
    Call::from_pbn(&t).filter(|c| matches!(c, Call::Bid { .. }))
}

/// A bid's place in the bidding order (1♣ lowest, 7NT highest).
fn bid_rank(c: &Call) -> Option<i32> {
    match c {
        Call::Bid { level, strain } => {
            let s = match strain {
                Strain::Clubs => 0,
                Strain::Diamonds => 1,
                Strain::Hearts => 2,
                Strain::Spades => 3,
                Strain::NoTrump => 4,
            };
            Some(*level as i32 * 5 + s)
        }
        _ => None,
    }
}

fn cmp3(known_true: bool, known_false: bool) -> Tri {
    if known_true {
        Tri::True
    } else if known_false {
        Tri::False
    } else {
        Tri::Unknown
    }
}

fn exact_attr(f: &Facts, n: &str, v: Valuation) -> Val {
    match n {
        "hcp" => Val::Num(Range::point(f.hcp)),
        "tens" => Val::Num(Range::point(f.tens)),
        // Whole points: 9¾ counts as 9.
        "points" => Val::Num(Range::point(f.points_q(v).div_euclid(4))),
        "suit_points" => Val::Num(Range::point(f.suit_points_q(v).div_euclid(4))),
        "balanced" => Val::Bool(Tri::from_bool(f.balanced)),
        "semibalanced" => Val::Bool(Tri::from_bool(f.dist[3] >= 2 && f.dist[0] <= 6)),
        "shortest" => Val::Num(Range::point(f.dist[3])),
        "longest" => Val::Num(Range::point(f.dist[0])),
        "second_longest" => Val::Num(Range::point(f.second_longest())),
        "length_points" => Val::Num(Range::point(f.length_points())),
        n if n.starts_with("bba_") => Val::Num(Range::point(f.bba_named(n).unwrap_or(0))),
        "controls" => Val::Num(Range::point(f.controls())),
        "losers" => Val::Num(Range::point(f.losers())),
        "quick_tricks" => Val::Num(Range::point(f.quick_trick_halves().div_euclid(2))),
        _ => unreachable!(),
    }
}

fn plain(name: &str) -> Segment {
    Segment {
        name: name.into(),
        args: None,
    }
}

pub fn path_expr(name: &str) -> Expr {
    Expr::Path {
        path: vec![plain(name)],
    }
}

/// Constant true (`And []`) or false (`Or []`).
/// `hcp <= n`, as a constraint for the knowledge store.
pub fn hcp_at_most(n: i32) -> Expr {
    Expr::Cmp {
        cmp: CmpOp::Le,
        lhs: Box::new(path_expr("hcp")),
        rhs: Box::new(Expr::Int { value: n as i64 }),
    }
}

pub fn konst(b: bool) -> Expr {
    if b {
        Expr::And { all: vec![] }
    } else {
        Expr::Or { any: vec![] }
    }
}

// ── Checking names before anything runs ─────────────────────────────
//
// The evaluator reports an unknown name only when a rule is tried, and a
// failed `when` just rules the candidate out, so a misspelt term (or a
// term an older engine does not have) silently switched rules off.
// `check_terms` walks every condition at load time instead. The lists
// below are what `name`, `self_func`, `knowledge_attr`, `we_attr` and
// `they_attr` accept; `terms_match_the_evaluator` keeps them in step.

/// Bare state names (`name`).
/// Bare state names (`name`).
const STATE_TERMS: &[Term] = &[
    t("opening", "no one has bid yet"),
    t(
        "last",
        "my own last call (`me.last`); compare with a call: `me.last=1N`",
    ),
    t("passed_hand", "I have called, and only passed"),
    t(
        "bids",
        "how many bids I have made, not counting passes and doubles (`me.bids`)",
    ),
    t(
        "has_bid",
        "I have made a bid, not only passes or doubles (`me.has_bid`)",
    ),
    t("seat", "my seat from the dealer, 1 to 4"),
    t("vul", "my side is vulnerable"),
    t(
        "game_reached",
        "our side's last bid is game or higher and it is our contract",
    ),
    t("imps", "IMPs or other total-point scoring"),
    t("matchpoints", "matchpoints or board-a-match"),
    t("favourable", "we are not vulnerable and they are"),
    t("unfavourable", "we are vulnerable and they are not"),
    t(
        "captain",
        "I place the contract: partner's shown points span 4 or fewer (12-15) and mine more, or partner has answered my question",
    ),
    t("trump", "`we.trump`"),
    t(
        "slam_try",
        "judgment hook: slam is worth trying (placeholder: combined HCP >= 31)",
    ),
    t(
        "grand_try",
        "judgment hook: grand slam is worth trying (placeholder: combined HCP >= 35)",
    ),
];
/// Functions of my own hand or position (`self_func`), besides SELF_FUNCS.
/// Functions of my own position (`self_func`), besides SELF_FUNCS.
const POSITION_FUNCS: &[Term] = &[
    f(
        "denied",
        "(x)",
        "in the control-bid dialogue I skipped x (`me.denied(x)`)",
    ),
    f(
        "cued",
        "(x)",
        "in the control-bid dialogue I have shown a control in x",
    ),
    f(
        "under_game",
        "(x)",
        "the cheapest bid in x is below game in our agreed suit (3NT if none)",
    ),
    f(
        "cheapest_rank",
        "(x)",
        "the cheapest bid in x as level x 5 + C0 D1 H2 S3 (`prefer 0 - cheapest_rank(x)`)",
    ),
];
/// What `partner.`, `lho.`, `rho.` and `shown.` take (`knowledge_attr`),
/// besides suits, variables and `bba_*`.
/// What `partner.`, `lho.`, `rho.` and `shown.` take (`knowledge_attr`),
/// besides suits, variables and `bba_*`. Values are ranges: a comparison
/// holds when it is known (`maybe` for "not ruled out").
const SEAT_ATTRS: &[Term] = &[
    t("hcp", "HCP shown"),
    t("tens", "tens (never tracked: 0..4)"),
    t(
        "points",
        "total points shown (HCP when no call showed points)",
    ),
    t("suit_points", "suit points shown"),
    t("balanced", "shown balanced"),
    t("shortest", "length of the shortest suit, as far as shown"),
    t("longest", "length of the longest suit, as far as shown"),
    t(
        "second_longest",
        "length of the second-longest suit, as far as shown",
    ),
    t("length_points", "cards beyond four, as far as shown"),
    t(
        "last",
        "that seat's last call (`partner.last=3N`, `=P`, `=X`, `=XX`)",
    ),
    t("opened", "that seat made the opening bid"),
    t("bids", "how many bids that seat has made"),
    t(
        "has_bid",
        "that seat has made a bid, not only passes or doubles",
    ),
    t(
        "jumped",
        "that seat's last bid was at least a level above the cheapest in its strain",
    ),
    f(
        "denied",
        "(x)",
        "in the control-bid dialogue that seat skipped x",
    ),
    f(
        "cued",
        "(x)",
        "in the control-bid dialogue that seat has shown a control in x",
    ),
    f(
        "bypassed",
        "(x[, call])",
        "that seat's last bid went past an available bid in x (above `call` when given)",
    ),
    f(
        "named",
        "(x)",
        "that seat has made a natural bid in x (a suit or N) at any point; calls a rule marks artificial do not count",
    ),
    f("has", "(rank, x)", "not tracked: unknown"),
    f("stop", "(x)", "not tracked: unknown"),
    t("semibalanced", "not tracked: unknown"),
    f(
        "tp",
        "(x)",
        "support points with x as trump, when a raise showed them; else HCP",
    ),
    f("keycards", "(x)", "not tracked: 0..40 (see `we.keycards`)"),
    t("controls", "not tracked: 0..40"),
    t("losers", "not tracked: 0..40"),
    t("quick_tricks", "not tracked: 0..40"),
    t("bare_suits", "not tracked: 0..4"),
    f("bare", "(x)", "not tracked: unknown"),
    t("trump", "length in our agreed suit (`partner.trump.min`)"),
    f("quality", "(x)", "not tracked: 0..40"),
    f("top5", "(x)", "not tracked: 0..40"),
];
/// `we.`: our partnership, my hand and partner's range together.
const WE_ATTRS: &[Term] = &[
    t(
        "trump",
        "the agreed strain (`is suit`, `is notrump`, `is none`), usable as a suit",
    ),
    t("forcing", "`none`, `round` or `game`"),
    f(
        "named",
        "(x)",
        "either of us has made a natural bid in x (a suit or N) at any point; artificial calls do not count",
    ),
    t("gf", "we are in a game force (`we.forcing = game`)"),
    t(
        "hcp",
        "my HCP plus partner's range: `hcp + partner.hcp` (`.min`, `.max` are partner's ends)",
    ),
    t(
        "points",
        "my points plus partner's range: `points + partner.points`",
    ),
    f(
        "fit",
        "(x)",
        "my length in x plus partner's range: `x + partner.x`; `.min` is the known fit",
    ),
    f(
        "tp",
        "(x)",
        "my support points with x as trump plus partner's: `tp(x) + partner.tp(x)`",
    ),
    f(
        "keycards",
        "(x)",
        "my keycards plus partner's answer, within the deck's five",
    ),
];
/// `they.`: the opponents.
const THEY_ATTRS: &[Term] = &[
    t("bid", "the opponents have bid or doubled"),
    t("vul", "the opponents are vulnerable"),
    t("hcp", "LHO's and RHO's HCP ranges added"),
    f("fit", "(x)", "LHO's and RHO's lengths in x added"),
    t(
        "level",
        "the level of the opponents' last bid (none before they bid)",
    ),
    t(
        "strain",
        "the strain of the opponents' last bid, usable as a suit",
    ),
    t(
        "still_bidding",
        "one of the opponents' last calls is not a pass (or one has not called yet)",
    ),
    t(
        "game_reached",
        "the opponents' last bid is game or higher and it is their contract",
    ),
];
/// Other bare names (`name`).
const BARE_NAMES: &[Term] = &[
    t("N", "notrump, as a strain (`NT` too)"),
    t("NT", "notrump, as a strain"),
    t(
        "strength",
        "my total points as a band: `strength=invite`, `strength>=game`",
    ),
    t("suit_strength", "the same with suit points"),
];
/// Names accepted by pattern rather than by list: documented here, checked
/// by `bare_name_ok` and `seat_attr_ok`.
const PATTERN_NAMES: &[Term] = &[
    t("S H D C", "a suit: its length in a comparison (`S>=5`), else the suit (`stop(S)`)"),
    t("M, m, x, y, z, t, ...", "suit variables, bound by a pattern (`after 1M (P)`) or a question; `M` a major, `m` a minor"),
    t("<param>", "a module `param`, the card value it names"),
    t("partner.bba_*", "any `bba_` count of another seat: unknown (0..40)"),
    t(".min / .max", "the ends of a range: `partner.hcp.min`"),
];

/// Markdown reference of every name a condition may use, grouped as
/// `check_terms` reads them: what `rbb bid terms` prints, and what the
/// generated section of docs/CONTRACT.md holds.
pub fn terms_reference() -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let mut table = |title: &str, intro: &str, prefix: &str, list: &[Term]| {
        let _ = writeln!(out, "### {title}\n");
        if !intro.is_empty() {
            let _ = writeln!(out, "{intro}\n");
        }
        let _ = writeln!(out, "| Term | Meaning |\n|---|---|");
        for term in list {
            let _ = writeln!(
                out,
                "| `{prefix}{}{}` | {} |",
                term.name, term.args, term.meaning
            );
        }
        out.push('\n');
    };
    table(
        "My own hand",
        "Exact while I choose a call; what I have shown when my call is being \
         read. Bare or with `me.` (`me.hcp`). They may not appear in a \
         context's `when`.",
        "",
        SELF_ATTRS,
    );
    table(
        "Functions of my own hand",
        "Bare or with `me.`. `x` is a suit, a suit variable or `trump`.",
        "",
        SELF_FUNCS,
    );
    table(
        "Auction state",
        "Bare or with `me.`. Public, so fine in a context's `when`, except the \
         judgment hooks `slam_try` and `grand_try`.",
        "",
        STATE_TERMS,
    );
    table(
        "My position in the auction",
        "Bare or with `me.`.",
        "",
        POSITION_FUNCS,
    );
    table(
        "Other seats",
        "With `partner.`, `lho.`, `rho.`, or `shown.` (what I have shown), \
         besides a suit or suit variable for its length (`partner.S`, \
         `partner.M`). Values are ranges: a comparison holds when it is \
         known; `maybe` asks whether it is still possible.",
        "partner.",
        SEAT_ATTRS,
    );
    table("Our partnership", "", "we.", WE_ATTRS);
    table("The opponents", "", "they.", THEY_ATTRS);
    table("Other names", "", "", BARE_NAMES);
    table("Words", "Values compared with `=` or `is`.", "", SYMBOLS);
    table(
        "Names accepted by form",
        "Checked by shape rather than listed.",
        "",
        PATTERN_NAMES,
    );
    table(
        "State set by `sets`",
        "Not conditions: what a rule's `sets` clause may assign \
         (`sets forcing=game, trump=x`). Another name, or a value of another \
         form, refuses the load.",
        "sets ",
        crate::engine::SET_KEYS,
    );
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}

/// A suit variable (`x`, `M`, `t`, ...), bound by a pattern or a question.
pub(crate) fn is_variable(n: &str) -> bool {
    n == "M" || (n.len() == 1 && n.chars().all(|c| c.is_ascii_lowercase()))
}

fn bare_name_ok(n: &str, params: &[&str]) -> bool {
    is_variable(n)
        || params.contains(&n)
        || suit_index(n).is_some()
        || named(BARE_NAMES, n)
        || named(SELF_ATTRS, n)
        || named(STATE_TERMS, n)
        || named(SYMBOLS, n)
}

fn seat_attr_ok(n: &str) -> bool {
    suit_index(n).is_some() || is_variable(n) || n.starts_with("bba_") || named(SEAT_ATTRS, n)
}

fn check_path(path: &[Segment], params: &[&str], out: &mut Vec<String>) {
    let first = &path[0];
    let prefixed = path.len() > 1
        && matches!(
            first.name.as_str(),
            "partner" | "lho" | "rho" | "shown" | "me" | "we" | "they"
        );
    let (head, rest) = if prefixed {
        let seg = &path[1];
        let n = seg.name.as_str();
        let ok = match first.name.as_str() {
            "we" => named(WE_ATTRS, n),
            "they" => named(THEY_ATTRS, n),
            "me" => match &seg.args {
                Some(_) => named(SELF_FUNCS, n) || named(POSITION_FUNCS, n),
                None => bare_name_ok(n, params),
            },
            _ => seat_attr_ok(n),
        };
        if !ok {
            out.push(format!("unknown term `{}.{n}`", first.name));
        }
        (seg, &path[2..])
    } else {
        let n = first.name.as_str();
        let ok = match &first.args {
            Some(_) => named(SELF_FUNCS, n) || named(POSITION_FUNCS, n),
            None => bare_name_ok(n, params),
        };
        if !ok {
            let what = if first.args.is_some() {
                "function"
            } else {
                "term"
            };
            out.push(format!("unknown {what} `{n}`"));
        }
        (first, &path[1..])
    };
    for a in head.args.iter().flatten() {
        check_expr(a, params, out);
    }
    for seg in rest {
        if !matches!(seg.name.as_str(), "min" | "max") {
            out.push(format!("`.{}` is not `.min` or `.max`", seg.name));
        }
    }
}

fn check_expr(e: &Expr, params: &[&str], out: &mut Vec<String>) {
    match e {
        Expr::And { all } => all.iter().for_each(|x| check_expr(x, params, out)),
        Expr::Or { any } => any.iter().for_each(|x| check_expr(x, params, out)),
        Expr::Not { expr } | Expr::Maybe { expr } | Expr::Neg { expr } => {
            check_expr(expr, params, out)
        }
        Expr::InSet { expr, .. } | Expr::Is { expr, .. } => check_expr(expr, params, out),
        Expr::Cmp { lhs, rhs, .. } | Expr::Arith { lhs, rhs, .. } => {
            check_expr(lhs, params, out);
            check_expr(rhs, params, out);
        }
        Expr::InRange { expr, lo, hi } => {
            check_expr(expr, params, out);
            check_expr(lo, params, out);
            check_expr(hi, params, out);
        }
        Expr::Path { path } => check_path(path, params, out),
        // Question kinds are free names (`asked keycards(t)`), shapes and
        // calls are checked by the parser.
        Expr::Asked { .. }
        | Expr::Answered { .. }
        | Expr::Shape { .. }
        | Expr::Int { .. }
        | Expr::Call { .. } => {}
    }
}

/// Terms in `e` that depend on the chooser's own hand: my HCP, points,
/// shape, a suit or suit variable compared as a length, `me.hcp`, and
/// the `we.` sums that add my hand. A context's `when` is checked before
/// the hand is known, so such a term makes it never true.
fn hand_terms(e: &Expr, out: &mut Vec<String>) {
    let length = |x: &Expr, out: &mut Vec<String>| {
        if let Expr::Path { path } = x {
            if path.len() == 1 && path[0].args.is_none() {
                let n = path[0].name.as_str();
                if suit_index(n).is_some() || is_variable(n) {
                    out.push(format!("`{n}` (my length)"));
                }
            }
        }
    };
    match e {
        Expr::And { all } => all.iter().for_each(|x| hand_terms(x, out)),
        Expr::Or { any } => any.iter().for_each(|x| hand_terms(x, out)),
        Expr::Not { expr } | Expr::Maybe { expr } | Expr::Neg { expr } => hand_terms(expr, out),
        // A suit compared, or in arithmetic, is my length there.
        Expr::Cmp { lhs, rhs, .. } | Expr::Arith { lhs, rhs, .. } => {
            length(lhs, out);
            length(rhs, out);
            hand_terms(lhs, out);
            hand_terms(rhs, out);
        }
        Expr::InRange { expr, lo, hi } => {
            length(expr, out);
            hand_terms(expr, out);
            hand_terms(lo, out);
            hand_terms(hi, out);
        }
        Expr::InSet { expr, .. } => {
            length(expr, out);
            hand_terms(expr, out);
        }
        Expr::Shape { pattern } => out.push(format!("`shape {pattern}`")),
        Expr::Path { path } => {
            let n = path[0].name.as_str();
            let self_term = |m: &str, args: bool| {
                if args {
                    named(SELF_FUNCS, m)
                } else {
                    named(SELF_ATTRS, m)
                        || matches!(m, "slam_try" | "grand_try" | "strength" | "suit_strength")
                }
            };
            match (n, path.get(1)) {
                ("me", Some(seg)) if self_term(&seg.name, seg.args.is_some()) => {
                    out.push(format!("`me.{}`", seg.name))
                }
                ("we", Some(seg))
                    if matches!(
                        seg.name.as_str(),
                        "hcp" | "keycards" | "points" | "tp" | "fit"
                    ) =>
                {
                    out.push(format!("`we.{}`", seg.name))
                }
                (_, None) if self_term(n, path[0].args.is_some()) => out.push(format!("`{n}`")),
                _ => {}
            }
        }
        Expr::Is { .. }
        | Expr::Asked { .. }
        | Expr::Answered { .. }
        | Expr::Int { .. }
        | Expr::Call { .. } => {}
    }
}

/// Every name in every condition (`when`, `shows`, `denies`, `prefer`)
/// is one the engine knows, and so is every `sets` state and value
/// (`check_sets`). Conditions are checked as the engine runs them, with
/// every definition inlined (`macros`); the definitions themselves are
/// checked too. Errors name the file and line.
pub fn check_terms(modules: &[bidspec::Module]) -> Vec<bidspec::Diagnostic> {
    use crate::macros::{qualified, Defines};
    let defines = Defines::new(modules);
    // A definition's card parameters, as the rules that use it see them.
    let defined: Vec<String> = modules
        .iter()
        .filter(|m| !m.defines.is_empty())
        .flat_map(|m| m.params.iter().map(|p| qualified(&m.name, &p.name)))
        .collect();
    fn walk(
        m: &bidspec::Module,
        c: &bidspec::ast::Context,
        params: &[&str],
        defines: &Defines,
        out: &mut Vec<bidspec::Diagnostic>,
    ) {
        let diag = |line: usize, message: String| bidspec::Diagnostic {
            file: m.file.clone(),
            line,
            col: 0,
            message,
        };
        let expand = |line: usize, e: &Expr, out: &mut Vec<bidspec::Diagnostic>| {
            let mut errors = Vec::new();
            let x = defines.expand(e, &mut errors);
            out.extend(errors.into_iter().map(|msg| diag(line, msg)));
            x
        };
        if let Some(w) = &c.when {
            let w = expand(c.line, w, out);
            let mut terms = Vec::new();
            hand_terms(&w, &mut terms);
            terms.dedup();
            if !terms.is_empty() {
                out.push(diag(
                    c.line,
                    format!(
                        "{} in a context's `when` depends on the hand: contexts are \
                         checked before the hand is known, so it is never true; move it \
                         to the rules' `when` or `shows`",
                        terms.join(", ")
                    ),
                ));
            }
            let mut msgs = Vec::new();
            check_expr(&w, params, &mut msgs);
            out.extend(msgs.into_iter().map(|msg| diag(c.line, msg)));
        }
        for r in &c.rules {
            for e in [&r.shows, &r.when, &r.denies, &r.prefer]
                .into_iter()
                .flatten()
            {
                let e = expand(r.line, e, out);
                let mut msgs = Vec::new();
                check_expr(&e, params, &mut msgs);
                out.extend(msgs.into_iter().map(|msg| diag(r.line, msg)));
            }
        }
        for inner in &c.contexts {
            walk(m, inner, params, defines, out);
        }
    }
    let mut out = Vec::new();
    let mut seen: HashMap<&str, &str> = HashMap::new();
    for m in modules {
        let mut params: Vec<&str> = m.params.iter().map(|p| p.name.as_str()).collect();
        params.extend(defined.iter().map(String::as_str));
        for d in &m.defines {
            let diag = |message: String| bidspec::Diagnostic {
                file: m.file.clone(),
                line: d.line,
                col: 0,
                message,
            };
            let n = d.name.as_str();
            if bare_name_ok(n, &[])
                || named(SELF_FUNCS, n)
                || named(POSITION_FUNCS, n)
                || named(BARE_NAMES, n)
            {
                out.push(diag(format!("`define {n}`: `{n}` is already a term")));
            }
            if let Some(first) = seen.insert(n, &m.file) {
                out.push(diag(format!("`{n}` is defined twice (first in {first})")));
            }
            let mut errors = Vec::new();
            let body = defines.expand(&d.body, &mut errors);
            let mut msgs = Vec::new();
            check_expr(&body, &params, &mut msgs);
            out.extend(
                errors
                    .into_iter()
                    .chain(msgs)
                    .map(|msg| diag(format!("`{n}`: {msg}"))),
            );
        }
        for c in &m.contexts {
            walk(m, c, &params, &defines, &mut out);
        }
        // A force is judged when a call is made, by anyone reading it: its
        // condition is public. `call` is the call being made.
        let mut fparams = params.clone();
        fparams.push("call");
        for f in &m.forces {
            let Some(w) = &f.when else { continue };
            let diag = |message: String| bidspec::Diagnostic {
                file: m.file.clone(),
                line: f.line,
                col: 0,
                message,
            };
            let mut errors = Vec::new();
            let w = defines.expand(w, &mut errors);
            let mut msgs = Vec::new();
            check_expr(&w, &fparams, &mut msgs);
            let mut terms = Vec::new();
            hand_terms(&w, &mut terms);
            terms.dedup();
            if !terms.is_empty() {
                msgs.push(format!(
                    "{} in a `force` depends on the caller's hand: a force is public, \
                     judged the same by everyone who reads the call",
                    terms.join(", ")
                ));
            }
            out.extend(errors.into_iter().chain(msgs).map(diag));
        }
    }
    out.extend(crate::engine::check_sets(modules));
    out
}

#[cfg(test)]
mod term_tests {
    use super::*;
    use bridge_types::{ScoringMethod, Vulnerability};

    fn when_of(expr: &str) -> Expr {
        let src = format!("module t \"t\"\nwhen {expr}\n  P  \"x\"\n");
        let m =
            bidspec::compile(&src, "t.bid", &bridge_card::Registry::parse("").unwrap()).unwrap();
        m.contexts[0].when.clone().unwrap()
    }

    fn errors(expr: &str) -> Vec<String> {
        let mut out = Vec::new();
        check_expr(&when_of(expr), &["style"], &mut out);
        out
    }

    #[test]
    fn bare_trump_in_a_comparison_is_my_own_length() {
        // Read for another seat, `trump > partner.trump.min` is about the
        // caller's hand: not public, so it cannot rule the call out.
        let b = Bindings::new();
        assert!(hand_dependent(&when_of("trump > partner.trump.min"), &b));
        assert!(hand_dependent(&when_of("S > partner.S.min"), &b));
        assert!(!hand_dependent(&when_of("partner.trump.min >= 3"), &b));
        assert!(!hand_dependent(&when_of("we.trump is suit"), &b));
    }

    #[test]
    fn card_text_reads_as_a_bid_and_bids_are_ordered() {
        let two_h = Call::Bid {
            level: 2,
            strain: Strain::Hearts,
        };
        assert_eq!(bid_of_text("2♥"), Some(two_h.clone()));
        assert_eq!(bid_of_text(" 2H "), Some(two_h.clone()));
        assert_eq!(bid_of_text("X"), None);
        assert_eq!(bid_of_text("e.g. 2♦"), None);
        let one_s = Call::Bid {
            level: 1,
            strain: Strain::Spades,
        };
        let two_s = Call::Bid {
            level: 2,
            strain: Strain::Spades,
        };
        assert!(bid_rank(&one_s) < bid_rank(&two_h));
        assert!(bid_rank(&two_h) < bid_rank(&two_s));
        assert_eq!(bid_rank(&Call::Double), None);
    }

    #[test]
    fn unknown_names_are_reported() {
        assert!(errors("partner.jumped, me.last=P, hcp>=12, tp(S)>=10").is_empty());
        assert!(errors("style is bba, we.forcing = game, they.bid").is_empty());
        assert!(errors("partner.hcp.min>=15, x>=4, M is not x").is_empty());
        assert_eq!(errors("me.opened"), ["unknown term `me.opened`"]);
        assert_eq!(errors("opened"), ["unknown term `opened`"]);
        assert_eq!(errors("partner.hpc>=10"), ["unknown term `partner.hpc`"]);
        assert_eq!(errors("we.bid"), ["unknown term `we.bid`"]);
        assert_eq!(errors("stopper(S)"), ["unknown function `stopper`"]);
        assert_eq!(errors("hcp.low>=3"), ["`.low` is not `.min` or `.max`"]);
    }

    /// The 40-HCP deck: publicly, a seat's maximum is 40 minus the
    /// others' minimums, and a takeout-double-like disjunction collapses
    /// to its shape branch once the power branch is out of reach;
    /// privately, the actor's own HCP counts too.
    #[test]
    fn the_deck_collapses_a_power_branch() {
        use crate::facts::Facts;
        use bridge_types::Hand;
        let double = when_of("(S>=3, D>=3, C>=3) | hcp>=17");
        let mut pos = Position::new(
            Direction::East,
            Vulnerability::None,
            ScoringMethod::Matchpoints,
        );
        let south = Direction::South.to_index();
        assert!(pos.knowledge[south].add(double));
        assert_eq!(pos.knowledge[south].len[3].lo, 0); // spades, C D H S
                                                       // Public: East 14+, North 12+, West 0+: South holds at most 14.
        pos.knowledge[Direction::East.to_index()].add(when_of("hcp>=14"));
        pos.knowledge[Direction::North.to_index()].add(when_of("hcp>=12"));
        pos.apply_deck_hcp();
        assert_eq!(pos.knowledge[south].hcp.hi, 14);
        assert_eq!(
            pos.knowledge[south].len[3].lo, 3,
            "{:?}",
            pos.knowledge[south]
        );

        // Private: nothing public about North, but North holds 10 and East
        // 14+, so South holds at most 16 from North's seat.
        let mut pos = Position::new(
            Direction::East,
            Vulnerability::None,
            ScoringMethod::Matchpoints,
        );
        pos.knowledge[south].add(when_of("(S>=3, D>=3, C>=3) | hcp>=17"));
        pos.knowledge[Direction::East.to_index()].add(when_of("hcp>=14"));
        let hand = Facts::new(&Hand::from_pbn("T3.J.AJ8763.KJ87").unwrap());
        let params = HashMap::new();
        let ctx = Ctx {
            pos: &pos,
            actor: Direction::North,
            hand: Some(&hand),
            params: &params,
            valuation: Valuation::default(),
            private: None,
        };
        let mut b = Bindings::new();
        assert_eq!(ctx.cond(&when_of("partner.S>=3"), &mut b), Ok(Tri::True));
        assert_eq!(ctx.cond(&when_of("partner.hcp<=16"), &mut b), Ok(Tri::True));
        // Publicly South is still unknown.
        assert_eq!(pos.knowledge[south].len[3].lo, 0);
    }

    /// A context's `when` is judged before the hand is known: terms about
    /// my own hand there are reported, public ones are not.
    #[test]
    fn hand_terms_in_a_context_are_reported() {
        let check = |when: &str| {
            let src = format!("module t \"t\"\nafter 1x (P)\n  when {when}\n    P  \"x\"\n");
            let m = bidspec::compile(&src, "t.bid", &bridge_card::Registry::parse("").unwrap())
                .unwrap();
            check_terms(&[m])
                .into_iter()
                .map(|d| d.message)
                .collect::<Vec<_>>()
        };
        for ok in [
            "x is not C, they.bid",
            "me.last=P, !lho.opened",
            "partner.hcp.min >= 12, partner.x>=3",
            "we.forcing = game",
        ] {
            assert!(check(ok).is_empty(), "{ok}: {:?}", check(ok));
        }
        for (bad, term) in [
            ("hcp + partner.hcp.min <= 24", "`hcp`"),
            ("x>=3", "`x` (my length)"),
            ("stop(S)", "`stop`"),
            ("shape 4333", "`shape 4333`"),
            ("we.hcp >= 25", "`hcp`"),
            ("we.fit(S).min >= 8", "`S` (my length)"),
            ("me.points >= 12", "`me.points`"),
        ] {
            let msgs = check(bad);
            assert!(
                msgs.len() == 1 && msgs[0].starts_with(term),
                "{bad}: {msgs:?}"
            );
        }
    }

    /// Definitions are checked where they are made and where they are
    /// used; their card parameters are their own module's.
    #[test]
    fn definitions_are_checked() {
        let module = |src: &str| bidspec::parse(src, "t.bid").unwrap();
        let msgs = |mods: &[bidspec::Module]| {
            check_terms(mods)
                .into_iter()
                .map(|d| d.message)
                .collect::<Vec<_>>()
        };
        let d = module(
            "module d \"d\"\n  param style = x.y\n\
             define held(x) = has(A,x), style is bba\n\
             define hcp = balanced\n\
             define loop = loop\n\
             define oops = hpc>=3\n",
        );
        let u = module(
            "module u \"u\"\nwhen opening\n  P \"x\"  when held(S), held\n  P \"y\"  when held(S, H)\n",
        );
        let m = msgs(&[d.clone(), u]);
        for want in [
            "`define hcp`: `hcp` is already a term",
            "`loop` is defined in terms of itself",
            "unknown term `hpc`",
            "`held` takes 1 argument (x)",
        ] {
            assert!(m.iter().any(|x| x.contains(want)), "{want}: {m:?}");
        }
        assert!(!m.iter().any(|x| x.contains("style")), "{m:?}");
        let twice = module("module e \"e\"\ndefine held(x) = x>=3\n");
        assert!(msgs(&[d, twice])
            .iter()
            .any(|x| x.contains("`held` is defined twice")));
    }

    /// The judgment layer's Phase 0 terms (docs/JUDGMENT-LAYER.md).
    #[test]
    fn phase_0_terms() {
        use crate::facts::Facts;
        use bridge_types::Hand;
        let bid = |s: &str| Call::from_pbn(s).unwrap();
        // South deals, NS vulnerable: 1NT (2H) 3S (P), North to call.
        let mut pos = Position::new(
            Direction::South,
            Vulnerability::NorthSouth,
            ScoringMethod::from_pbn("IMP").unwrap(),
        );
        for c in ["1N", "2H", "3S", "P"] {
            pos.calls.push(bid(c));
        }
        let s = Direction::South.to_index();
        let (w, e) = (Direction::West.to_index(), Direction::East.to_index());
        pos.knowledge[s].add(when_of("hcp=15..17"));
        pos.knowledge[s].add(when_of("S>=2"));
        pos.knowledge[w].add(when_of("hcp=8..11"));
        pos.knowledge[w].add(when_of("H>=6"));
        pos.knowledge[e].add(when_of("H>=2"));
        let hand = Facts::new(&Hand::from_pbn("AKJ74.K2.Q2.QT93").unwrap());
        let params = HashMap::new();
        let ctx = |actor, hand| Ctx {
            pos: &pos,
            actor,
            hand,
            params: &params,
            valuation: Valuation::default(),
            private: None,
        };
        let north = ctx(Direction::North, Some(&hand));
        let holds = |c: &Ctx, e: &str| {
            let e = crate::macros::Defines::default().expand(&when_of(e), &mut Vec::new());
            c.cond(&e, &mut Bindings::new())
        };
        for e in [
            "unfavourable, !favourable",
            "they.level = 2, they.strain is H, they.bid",
            "they.fit(H).min = 8, they.hcp.min = 8",
            "they.still_bidding",
            "!they.game_reached",
            "we.fit(S).min = 7, safe_level(S) = 1, we.hcp.min = 30",
            "we.points.max >= 30",
            "captain",
            "quick_tricks = 2, bare(C), bare(D), !bare(H), !bare(S), bare_suits = 2",
            // A condition counts 1 or 0.
            "we.fit(S).min + unfavourable = 8, 5 - favourable = 5",
        ] {
            assert_eq!(holds(&north, e), Ok(Tri::True), "{e}");
        }
        // South limited his hand: North places the contract, not South.
        let south = ctx(Direction::South, None);
        assert_eq!(holds(&south, "captain"), Ok(Tri::False));
        assert_eq!(holds(&south, "favourable"), Ok(Tri::False));
        // A condition not yet known counts 0..1.
        assert_eq!(holds(&north, "(partner.H>=3) + 1 >= 2"), Ok(Tri::Unknown));
        // With a trump agreed, the trump suit is not bare.
        let mut agreed = pos.clone();
        agreed.sides[0].trump = Some(Strain::Clubs);
        let north = Ctx {
            pos: &agreed,
            ..ctx(Direction::North, Some(&hand))
        };
        assert_eq!(holds(&north, "!bare(C), bare_suits = 1"), Ok(Tri::True));
    }

    /// Every name the checker accepts, the evaluator accepts too.
    #[test]
    fn terms_match_the_evaluator() {
        let pos = Position::new(
            Direction::South,
            Vulnerability::None,
            ScoringMethod::Matchpoints,
        );
        let params = HashMap::new();
        let ctx = Ctx {
            pos: &pos,
            actor: Direction::South,
            hand: None,
            params: &params,
            valuation: Valuation::default(),
            private: None,
        };
        let arg = |n: &str| match n {
            "has" => "(A, S)",
            "bypassed" | "denied" | "cued" | "tp" | "keycards" | "stop" | "quality" | "top5"
            | "under_game" | "cheapest_rank" | "bare" | "safe_level" | "fit" | "named" => "(S)",
            _ => "",
        };
        let mut exprs: Vec<String> = Vec::new();
        // The reference's `args` column says which names take arguments.
        for n in STATE_TERMS.iter().chain(SELF_ATTRS) {
            assert!(n.args.is_empty(), "{}", n.name);
            exprs.push(format!("me.{}", n.name));
            exprs.push(n.name.to_string());
        }
        for n in SELF_FUNCS.iter().chain(POSITION_FUNCS) {
            assert!(!n.args.is_empty(), "{}", n.name);
            exprs.push(format!("{}{}", n.name, arg(n.name)));
        }
        for n in SEAT_ATTRS {
            assert_eq!(n.args.is_empty(), arg(n.name).is_empty(), "{}", n.name);
            exprs.push(format!("partner.{}{}", n.name, arg(n.name)));
        }
        for n in WE_ATTRS {
            let a = if n.args.is_empty() { "" } else { "(S)" };
            exprs.push(format!("we.{}{a}", n.name));
        }
        for n in THEY_ATTRS {
            exprs.push(format!("they.{}{}", n.name, arg(n.name)));
        }
        // `strength` is only compared with a band (tested with the bands).
        exprs.extend(["N", "NT"].map(String::from));
        for x in exprs {
            let e = when_of(&format!("{x} = 1 | !{x} = 1"));
            let Expr::Or { any } = &e else { panic!("{x}") };
            let mut b = Bindings::new();
            if let Err(err) = ctx.eval(&any[0], &mut b) {
                assert!(!err.contains("unknown"), "{x}: {err}");
            }
            let mut msgs = Vec::new();
            check_expr(&e, &[], &mut msgs);
            assert!(msgs.is_empty(), "{x}: {msgs:?}");
        }
    }
}
