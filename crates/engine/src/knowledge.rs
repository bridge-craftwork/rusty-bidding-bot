//! What is known about a hand: ranges for HCP and suit lengths, whether it
//! is balanced, and the resolved constraints it has shown.
//!
//! Narrowing is interval reasoning. It is sound (never excludes a hand that
//! satisfies the constraints) but not complete: a disjunction narrows to the
//! hull of its branches, and terms other than HCP, suit lengths and balance
//! are kept in `constraints` without narrowing.

use bidspec::ast::{CmpOp, Expr};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Range {
    pub lo: i32,
    pub hi: i32,
}

impl Range {
    pub const fn new(lo: i32, hi: i32) -> Range {
        Range { lo, hi }
    }
    pub const fn point(v: i32) -> Range {
        Range { lo: v, hi: v }
    }
    pub fn is_empty(&self) -> bool {
        self.lo > self.hi
    }
    pub fn intersect(&self, o: Range) -> Range {
        Range::new(self.lo.max(o.lo), self.hi.min(o.hi))
    }
    pub fn hull(&self, o: Range) -> Range {
        if self.is_empty() {
            return o;
        }
        if o.is_empty() {
            return *self;
        }
        Range::new(self.lo.min(o.lo), self.hi.max(o.hi))
    }
    pub fn as_point(&self) -> Option<i32> {
        (self.lo == self.hi).then_some(self.lo)
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.lo == self.hi {
            write!(f, "{}", self.lo)
        } else {
            write!(f, "{}-{}", self.lo, self.hi)
        }
    }
}

/// Three-valued truth: known true, known false, or not known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Tri {
    True,
    False,
    Unknown,
}

impl Tri {
    pub fn from_bool(b: bool) -> Tri {
        if b {
            Tri::True
        } else {
            Tri::False
        }
    }
    pub fn and(self, o: Tri) -> Tri {
        match (self, o) {
            (Tri::False, _) | (_, Tri::False) => Tri::False,
            (Tri::True, Tri::True) => Tri::True,
            _ => Tri::Unknown,
        }
    }
    pub fn or(self, o: Tri) -> Tri {
        match (self, o) {
            (Tri::True, _) | (_, Tri::True) => Tri::True,
            (Tri::False, Tri::False) => Tri::False,
            _ => Tri::Unknown,
        }
    }
    pub fn negate(self) -> Tri {
        match self {
            Tri::True => Tri::False,
            Tri::False => Tri::True,
            Tri::Unknown => Tri::Unknown,
        }
    }
    pub fn is_true(self) -> bool {
        self == Tri::True
    }
}

/// Suit letters in index order C, D, H, S.
pub const SUITS: [&str; 4] = ["C", "D", "H", "S"];

/// The most a hand's points can exceed its HCP, in quarters, with the
/// default valuation: four tens at ½ (notrump); nine cards beyond four at ½
/// (suit).
const MAX_BONUS: [i32; 2] = [8, 18];

pub fn suit_index(name: &str) -> Option<usize> {
    SUITS.iter().position(|s| *s == name)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SeatKnowledge {
    pub hcp: Range,
    /// Suit lengths, C D H S.
    pub len: [Range; 4],
    pub balanced: Tri,
    /// Total points in quarter points, [notrump, suit] (see
    /// `facts::Valuation`).
    pub pts: [Range; 2],
    /// Everything this seat has shown or denied, resolved so that it refers
    /// only to the seat's own hand (see `eval::resolve`), printed.
    pub shown: Vec<String>,
    #[serde(skip)]
    pub constraints: Vec<Expr>,
}

impl Default for SeatKnowledge {
    fn default() -> Self {
        SeatKnowledge {
            hcp: Range::new(0, 37),
            len: [Range::new(0, 13); 4],
            balanced: Tri::Unknown,
            pts: [
                Range::new(0, 4 * 37 + MAX_BONUS[0]),
                Range::new(0, 4 * 37 + MAX_BONUS[1]),
            ],
            shown: Vec::new(),
            constraints: Vec::new(),
        }
    }
}

impl SeatKnowledge {
    pub fn is_contradiction(&self) -> bool {
        self.bounds().is_contradiction()
    }

    fn bounds(&self) -> Bounds {
        Bounds {
            hcp: self.hcp,
            len: self.len,
            balanced: self.balanced,
            pts: self.pts,
        }
    }

    fn set_bounds(&mut self, b: Bounds) {
        self.hcp = b.hcp;
        self.len = b.len;
        self.balanced = b.balanced;
        self.pts = b.pts;
    }

    /// Total points as whole points (fractions dropped), for comparisons:
    /// the shown range when a call showed that kind of points, else the
    /// other kind if shown (a suit invitation, then a notrump decision: both
    /// measure the same strength), else the HCP range.
    pub fn whole_points(&self, kind: usize) -> Range {
        let k = if self.points_shown(kind) {
            kind
        } else if self.points_shown(1 - kind) {
            1 - kind
        } else {
            return self.hcp;
        };
        Range::new(self.pts[k].lo.div_euclid(4), self.pts[k].hi.div_euclid(4))
    }

    /// Has a call said something about this kind of points beyond what the
    /// HCP range implies?
    pub fn points_shown(&self, kind: usize) -> bool {
        let p = self.pts[kind];
        p.lo > 4 * self.hcp.lo || p.hi < 4 * self.hcp.hi + MAX_BONUS[kind]
    }

    /// Record a constraint (already resolved to this seat's own hand) and
    /// narrow the ranges by it. Returns false if it contradicts what was
    /// known, in which case the ranges are left unchanged.
    pub fn add(&mut self, e: Expr) -> bool {
        let narrowed = narrow(self.bounds(), &e, true);
        let ok = !narrowed.is_contradiction();
        if ok {
            self.shown.push(e.to_string());
            self.constraints.push(e);
            // Earlier disjunctions may settle now ("4 hearts or 4 spades",
            // then "not 4 hearts"): narrow by everything once more.
            let mut b = narrowed;
            for c in &self.constraints {
                let again = narrow(b, c, true);
                if !again.is_contradiction() {
                    b = again;
                }
            }
            self.set_bounds(b);
        }
        ok
    }
}

/// The ranges `narrow` works on: the part of `SeatKnowledge` that is cheap
/// to copy (no printed or stored constraints).
#[derive(Clone, Copy)]
struct Bounds {
    hcp: Range,
    len: [Range; 4],
    balanced: Tri,
    pts: [Range; 2],
}

impl Bounds {
    fn is_contradiction(&self) -> bool {
        self.hcp.is_empty()
            || self.pts.iter().any(Range::is_empty)
            || self.len.iter().any(Range::is_empty)
    }

    /// Deck and shape consequences: lengths sum to 13; balanced means every
    /// suit has 2 to 5 cards.
    fn normalize(&mut self) {
        if self.balanced == Tri::True {
            for r in &mut self.len {
                *r = r.intersect(Range::new(2, 5));
            }
        }
        for _ in 0..2 {
            for i in 0..4 {
                let others_lo: i32 = (0..4).filter(|&j| j != i).map(|j| self.len[j].lo).sum();
                let others_hi: i32 = (0..4).filter(|&j| j != i).map(|j| self.len[j].hi).sum();
                self.len[i] = self.len[i].intersect(Range::new(13 - others_hi, 13 - others_lo));
            }
        }
        // Points are at least HCP; HCP are at most the whole points.
        for (p, bonus) in self.pts.iter_mut().zip(MAX_BONUS) {
            p.lo = p.lo.max(4 * self.hcp.lo);
            p.hi = p.hi.min(4 * self.hcp.hi + bonus);
            self.hcp.hi = self.hcp.hi.min(p.hi.div_euclid(4));
        }
        if self.balanced == Tri::Unknown && self.len.iter().any(|r| r.hi <= 1 || r.lo >= 6) {
            self.balanced = Tri::False;
        }
    }

    fn contradiction() -> Bounds {
        Bounds {
            hcp: Range::new(1, 0),
            ..SeatKnowledge::default().bounds()
        }
    }

    fn hull(&self, o: &Bounds) -> Bounds {
        if self.is_contradiction() {
            return *o;
        }
        if o.is_contradiction() {
            return *self;
        }
        let mut k = *self;
        k.hcp = self.hcp.hull(o.hcp);
        for i in 0..4 {
            k.len[i] = self.len[i].hull(o.len[i]);
        }
        for i in 0..2 {
            k.pts[i] = self.pts[i].hull(o.pts[i]);
        }
        k.balanced = if self.balanced == o.balanced {
            self.balanced
        } else {
            Tri::Unknown
        };
        k
    }

    fn range_mut(&mut self, attr: Attr) -> &mut Range {
        match attr {
            Attr::Hcp => &mut self.hcp,
            Attr::Len(s) => &mut self.len[s],
            Attr::Pts(i) => &mut self.pts[i],
        }
    }
}

#[derive(Clone, Copy)]
enum Attr {
    Hcp,
    Len(usize),
    /// Total points (0 notrump, 1 suit): kept in quarters, compared by
    /// whole points.
    Pts(usize),
}

fn attr_of(e: &Expr) -> Option<Attr> {
    let Expr::Path { path } = e else { return None };
    if path.len() != 1 || path[0].args.is_some() {
        return None;
    }
    match path[0].name.as_str() {
        "hcp" => Some(Attr::Hcp),
        "points" => Some(Attr::Pts(0)),
        "suit_points" => Some(Attr::Pts(1)),
        n => suit_index(n).map(Attr::Len),
    }
}

fn int_of(e: &Expr) -> Option<i32> {
    match e {
        Expr::Int { value } => Some(*value as i32),
        _ => None,
    }
}

fn flip(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Ge => CmpOp::Le,
        o => o,
    }
}

fn negate(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Eq => CmpOp::Ne,
        CmpOp::Ne => CmpOp::Eq,
        CmpOp::Lt => CmpOp::Ge,
        CmpOp::Le => CmpOp::Gt,
        CmpOp::Gt => CmpOp::Le,
        CmpOp::Ge => CmpOp::Lt,
    }
}

/// `apply_const` for an attribute; points compare by whole points, so a
/// bound n covers quarters 4n..=4n+3.
fn apply_attr(attr: Attr, r: Range, op: CmpOp, n: i32) -> Range {
    match attr {
        Attr::Pts(_) => match op {
            CmpOp::Eq => r.intersect(Range::new(4 * n, 4 * n + 3)),
            CmpOp::Ne => r,
            CmpOp::Lt => apply_const(r, CmpOp::Lt, 4 * n),
            CmpOp::Le => apply_const(r, CmpOp::Le, 4 * n + 3),
            CmpOp::Gt => apply_const(r, CmpOp::Gt, 4 * n + 3),
            CmpOp::Ge => apply_const(r, CmpOp::Ge, 4 * n),
        },
        _ => apply_const(r, op, n),
    }
}

fn apply_const(r: Range, op: CmpOp, n: i32) -> Range {
    match op {
        CmpOp::Eq => r.intersect(Range::point(n)),
        CmpOp::Ne => {
            let mut r = r;
            if r.lo == n {
                r.lo += 1;
            }
            if r.hi == n {
                r.hi -= 1;
            }
            r
        }
        CmpOp::Lt => r.intersect(Range::new(i32::MIN, n - 1)),
        CmpOp::Le => r.intersect(Range::new(i32::MIN, n)),
        CmpOp::Gt => r.intersect(Range::new(n + 1, i32::MAX)),
        CmpOp::Ge => r.intersect(Range::new(n, i32::MAX)),
    }
}

/// Narrow `k` by `e` (or by its negation when `positive` is false).
fn narrow(k: Bounds, e: &Expr, positive: bool) -> Bounds {
    let mut out = k;
    match e {
        Expr::And { all } if positive => {
            for sub in all {
                out = narrow(out, sub, true);
            }
            // A second pass lets later conjuncts tighten earlier relations.
            for sub in all {
                out = narrow(out, sub, true);
            }
        }
        Expr::Or { any } if !positive => {
            for sub in any {
                out = narrow(out, sub, false);
            }
        }
        Expr::And { all } => {
            // not (a and b) = not a or not b
            if all.is_empty() {
                return Bounds::contradiction();
            }
            return hull_of(k, all.iter().map(|sub| (sub, false)));
        }
        Expr::Or { any } => {
            if any.is_empty() {
                return Bounds::contradiction();
            }
            return hull_of(k, any.iter().map(|sub| (sub, true)));
        }
        Expr::Not { expr } => return narrow(k, expr, !positive),
        Expr::Cmp { cmp, lhs, rhs } => {
            let op = if positive { *cmp } else { negate(*cmp) };
            match (attr_of(lhs), attr_of(rhs), int_of(lhs), int_of(rhs)) {
                (Some(a), _, _, Some(n)) => {
                    let r = out.range_mut(a);
                    *r = apply_attr(a, *r, op, n);
                }
                (_, Some(a), Some(n), _) => {
                    let r = out.range_mut(a);
                    *r = apply_attr(a, *r, flip(op), n);
                }
                // Relations between two attributes (S>=H); not for points,
                // which are in different units.
                (Some(a), Some(b), _, _)
                    if !matches!(a, Attr::Pts(_)) && !matches!(b, Attr::Pts(_)) =>
                {
                    let (ra, rb) = (*out.range_mut(a), *out.range_mut(b));
                    let (na, nb) = match op {
                        CmpOp::Ge => (
                            ra.intersect(Range::new(rb.lo, i32::MAX)),
                            rb.intersect(Range::new(i32::MIN, ra.hi)),
                        ),
                        CmpOp::Gt => (
                            ra.intersect(Range::new(rb.lo + 1, i32::MAX)),
                            rb.intersect(Range::new(i32::MIN, ra.hi - 1)),
                        ),
                        CmpOp::Le => (
                            ra.intersect(Range::new(i32::MIN, rb.hi)),
                            rb.intersect(Range::new(ra.lo, i32::MAX)),
                        ),
                        CmpOp::Lt => (
                            ra.intersect(Range::new(i32::MIN, rb.hi - 1)),
                            rb.intersect(Range::new(ra.lo + 1, i32::MAX)),
                        ),
                        CmpOp::Eq => (ra.intersect(rb), rb.intersect(ra)),
                        CmpOp::Ne => (ra, rb),
                    };
                    *out.range_mut(a) = na;
                    *out.range_mut(b) = nb;
                }
                _ => {}
            }
        }
        Expr::InRange { expr, lo, hi } => {
            if let (Some(a), Some(lo), Some(hi)) = (attr_of(expr), int_of(lo), int_of(hi)) {
                let (lo, hi) = if matches!(a, Attr::Pts(_)) {
                    (4 * lo, 4 * hi + 3)
                } else {
                    (lo, hi)
                };
                if positive {
                    let r = out.range_mut(a);
                    *r = r.intersect(Range::new(lo, hi));
                } else {
                    let below = apply_const(*out.range_mut(a), CmpOp::Lt, lo);
                    let above = apply_const(*out.range_mut(a), CmpOp::Gt, hi);
                    *out.range_mut(a) = below.hull(above);
                }
            }
        }
        Expr::InSet { expr, values } => {
            if let (Some(a), true) = (attr_of(expr), positive) {
                let r = out.range_mut(a);
                let hull = values
                    .iter()
                    .map(|v| r.intersect(Range::point(*v as i32)))
                    .fold(Range::new(1, 0), |acc, x| acc.hull(x));
                *r = hull;
            }
        }
        Expr::Path { path } if path.len() == 1 && path[0].name == "balanced" => {
            let want = Tri::from_bool(positive);
            if out.balanced == want.negate() {
                return Bounds::contradiction();
            }
            out.balanced = want;
        }
        _ => {}
    }
    out.normalize();
    out
}

fn hull_of<'a>(k: Bounds, branches: impl Iterator<Item = (&'a Expr, bool)>) -> Bounds {
    let mut acc = Bounds::contradiction();
    for (e, pos) in branches {
        acc = acc.hull(&narrow(k, e, pos));
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expr(s: &str) -> Expr {
        let src = format!("module t \"t\"\n\nwhen opening\n  P \"x\" shows {s}\n");
        bidspec::parse(&src, "t").unwrap().contexts[0].rules[0]
            .shows
            .clone()
            .unwrap()
    }

    #[test]
    fn narrows_ranges_and_balance() {
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("hcp=15..17, balanced")));
        assert_eq!(k.hcp, Range::new(15, 17));
        assert_eq!(k.len[3], Range::new(2, 5));
        assert!(k.add(expr("H>=4, S<=3")));
        assert_eq!(k.len[2], Range::new(4, 5));
        assert_eq!(k.len[3], Range::new(2, 3));
    }

    #[test]
    fn negation_and_disjunction() {
        let mut k = SeatKnowledge::default();
        k.add(expr("hcp=12..21"));
        // Denies "15-17 and balanced": nothing narrows until balance is known.
        k.add(Expr::Not {
            expr: Box::new(expr("hcp=15..17, balanced")),
        });
        assert_eq!(k.hcp, Range::new(12, 21));
        k.add(expr("balanced"));
        // Now the denial collapses: hcp is 12-14 or 18-21, whose hull is 12-21.
        assert_eq!(k.hcp, Range::new(12, 21));
        // Relations between suits.
        let mut k = SeatKnowledge::default();
        k.add(expr("S>=5, S>=H, H>=5"));
        assert_eq!(k.len[3].lo, 5);
        assert_eq!(k.len[2], Range::new(5, 8));
    }

    #[test]
    fn contradictions_are_rejected() {
        let mut k = SeatKnowledge::default();
        k.add(expr("hcp<=7"));
        assert!(!k.add(expr("hcp>=10")));
        assert_eq!(k.hcp, Range::new(0, 7));
    }
}
