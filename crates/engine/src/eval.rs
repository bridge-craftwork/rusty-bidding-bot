//! Evaluating `.bid` expressions.
//!
//! The actor's own hand is exact when choosing a call (`Ctx::hand` is set)
//! and a set of ranges otherwise. Other seats are always ranges, so a
//! comparison about them is `Tri::True` only when it is known.

use std::collections::HashMap;

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

const SYMBOLS: &[&str] = &[
    "game",
    "round",
    "none",
    "suit",
    "notrump",
    "poor",
    "fair",
    "good",
    "excellent",
    "signoff",
    "invite",
    "slam_invite",
    "slam",
    "A",
    "K",
    "Q",
    "J",
    "T",
];
/// Strength bands, weakest first (see `Ctx::strength_as_hcp`).
pub const BANDS: [&str; 5] = ["signoff", "invite", "game", "slam_invite", "slam"];
/// Combined HCP for game and for small slam.
const GAME: i32 = 25;
const SLAM: i32 = 33;

fn flip(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Ge => CmpOp::Le,
        o => o,
    }
}

const SELF_ATTRS: &[&str] = &[
    "hcp",
    "points",
    "suit_points",
    "balanced",
    "semibalanced",
    "shortest",
    "longest",
    "controls",
    "losers",
];
const SELF_FUNCS: &[&str] = &["tp", "keycards", "has", "quality", "stop"];

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
}

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
                    ("suit", Val::Strain(s)) => *s != Strain::NoTrump,
                    ("suit", Val::Suit(_)) => true,
                    ("notrump", Val::Strain(s)) => *s == Strain::NoTrump,
                    ("none", Val::Nothing) => true,
                    ("suit" | "notrump" | "none", _) => false,
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
                    let v = suit_of_strain(*actual).map_or(Val::Strain(*actual), Val::Suit);
                    b.insert(path[0].name.clone(), v);
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
        match (l, r) {
            (Val::Strain(a), Val::Strain(b)) => eq_only(a == b),
            (Val::Call(a), Val::Call(b)) => eq_only(a == b),
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
                    _ => self.they_attr(seg)?,
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
        if SELF_ATTRS.contains(&n) {
            return Ok(match self.hand {
                Some(f) => exact_attr(f, n, self.valuation),
                None => self.knowledge_attr(self.self_knowledge(), self.actor, seg, b)?,
            });
        }
        Ok(match n {
            "opening" => Val::Bool(Tri::from_bool(self.pos.is_opening())),
            "passed_hand" => Val::Bool(Tri::from_bool(self.pos.passed_hand(self.actor))),
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
            w if SYMBOLS.contains(&w) => Val::Sym(w.to_string()),
            _ => return Err(format!("unknown term `{n}`")),
        })
    }

    fn self_func(&self, n: &str, args: &[Expr], b: &mut Bindings) -> R<Val> {
        if !SELF_FUNCS.contains(&n) {
            return Err(format!("unknown function `{n}`"));
        }
        let Some(f) = self.hand else {
            // Not tracked in knowledge yet.
            return Ok(match n {
                "has" | "stop" => Val::Bool(Tri::Unknown),
                _ => Val::Num(Range::new(0, 40)),
            });
        };
        Ok(match n {
            "tp" => Val::Num(Range::point(f.total_points(self.suit_arg(args, 0, b)?))),
            "keycards" => Val::Num(Range::point(f.keycards(self.suit_arg(args, 0, b)?))),
            "quality" => Val::Num(Range::point(
                self.suit_arg(args, 0, b)?.map_or(0, |s| f.quality(s)),
            )),
            "stop" => Val::Bool(Tri::from_bool(
                self.suit_arg(args, 0, b)?.is_some_and(|s| f.stop(s)),
            )),
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
        Ok(match n {
            "hcp" => Val::Num(k.hcp),
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
            "last" => self
                .pos
                .last_call_of(seat)
                .cloned()
                .map_or(Val::Nothing, Val::Call),
            "opened" => Val::Bool(Tri::from_bool(self.pos.opener() == Some(seat))),
            "has" | "stop" | "semibalanced" => Val::Bool(Tri::Unknown),
            "keycards" | "tp" | "controls" | "losers" | "quality" => Val::Num(Range::new(0, 40)),
            _ => return Err(format!("unknown attribute `{n}`")),
        })
    }

    fn seat_attr(&self, d: Direction, seg: &Segment, b: &mut Bindings) -> R<Val> {
        self.knowledge_attr(self.pos.knowledge(d), d, seg, b)
    }

    fn we_attr(&self, seg: &Segment, b: &mut Bindings) -> R<Val> {
        let side = self.pos.side_state(self.actor);
        Ok(match seg.name.as_str() {
            "trump" => side.trump.map_or(Val::Nothing, Val::Strain),
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
            "keycards" => {
                let mine = self.num(&self.self_func(
                    "keycards",
                    seg.args.as_deref().unwrap_or(&[]),
                    b,
                )?)?;
                // Partner's answer ("1 or 4"), limited by the deck: five
                // keycards in all, so "1 or 4" facing my 2 can only be 1.
                let suit = self.suit_arg(seg.args.as_deref().unwrap_or(&[]), 0, b)?;
                let theirs = self.partner_keycards(suit);
                let fits: Vec<i32> = (0..=5)
                    .filter(|v| theirs.contains(v) && mine.lo + v <= 5)
                    .collect();
                match (fits.first(), fits.last()) {
                    (Some(lo), Some(hi)) => Val::Num(Range::new(mine.lo + lo, mine.hi + hi)),
                    _ => Val::Num(Range::new(mine.lo, 5)),
                }
            }
            n => return Err(format!("unknown attribute `we.{n}`")),
        })
    }

    fn they_attr(&self, seg: &Segment) -> R<Val> {
        let opp = self.actor.next();
        Ok(match seg.name.as_str() {
            "bid" => Val::Bool(Tri::from_bool(self.pos.side_acted(opp))),
            "vul" => Val::Bool(Tri::from_bool(self.pos.is_vulnerable(opp))),
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
                    && SELF_ATTRS.contains(&path[0].name.as_str()) =>
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
                            Ok(Val::Strain(st)) => suit_of_strain(st).map(|s| path_expr(SUITS[s])),
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
                if SELF_ATTRS.contains(&n) || suit_index(n).is_some() {
                    return Some(e.clone());
                }
                match self.eval(e, b).ok()? {
                    Val::Suit(s) => Some(path_expr(SUITS[s])),
                    Val::Num(r) => r.as_point().map(|v| Expr::Int { value: v as i64 }),
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
                _ => None,
            },
        }
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
                "we" => path
                    .get(1)
                    .is_some_and(|s| matches!(s.name.as_str(), "hcp" | "keycards" | "points")),
                "partner" | "lho" | "rho" | "shown" | "they" => false,
                _ if path.len() > 1 => false,
                n => {
                    SELF_ATTRS.contains(&n)
                        || SELF_FUNCS.contains(&n)
                        || matches!(n, "slam_try" | "grand_try" | "strength" | "suit_strength")
                        || suit_index(n).is_some()
                        || matches!(b.get(n), Some(Val::Suit(_)))
                }
            }
        }
    }
}

/// Does `e` refer to the actor's own hand (so it must not be folded to a
/// constant from knowledge)?
fn mentions_self(e: &Expr, b: &Bindings, params: &HashMap<String, Val>) -> bool {
    match e {
        Expr::Path { path } => {
            let n = path[0].name.as_str();
            if path.len() == 1 && path[0].args.is_some() {
                return SELF_FUNCS.contains(&n);
            }
            path.len() == 1
                && !params.contains_key(n)
                && !b.contains_key(n)
                && SELF_ATTRS.contains(&n)
        }
        Expr::Cmp { lhs, rhs, .. } => {
            // A suit or suit variable in a comparison is a length.
            let suitish = |x: &Expr| match x {
                Expr::Path { path } if path.len() == 1 && path[0].args.is_none() => {
                    suit_index(&path[0].name).is_some()
                        || matches!(b.get(&path[0].name), Some(Val::Suit(_)))
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
        // Whole points: 9¾ counts as 9.
        "points" => Val::Num(Range::point(f.points_q(v).div_euclid(4))),
        "suit_points" => Val::Num(Range::point(f.suit_points_q(v).div_euclid(4))),
        "balanced" => Val::Bool(Tri::from_bool(f.balanced)),
        "semibalanced" => Val::Bool(Tri::from_bool(f.dist[3] >= 2 && f.dist[0] <= 6)),
        "shortest" => Val::Num(Range::point(f.dist[3])),
        "longest" => Val::Num(Range::point(f.dist[0])),
        "controls" => Val::Num(Range::point(f.controls())),
        "losers" => Val::Num(Range::point(f.losers())),
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
pub fn konst(b: bool) -> Expr {
    if b {
        Expr::And { all: vec![] }
    } else {
        Expr::Or { any: vec![] }
    }
}
