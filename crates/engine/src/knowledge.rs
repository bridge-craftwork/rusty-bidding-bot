//! What is known about a hand: ranges for HCP and suit lengths, whether it
//! is balanced, and the resolved constraints it has shown.
//!
//! Narrowing is interval reasoning. It is sound (never excludes a hand that
//! satisfies the constraints) but not complete: a disjunction narrows to the
//! hull of its branches, and terms other than HCP, suit lengths, balance and
//! the point counts are kept in `constraints` without narrowing.
//!
//! The point counts are tied to the HCP and the suit lengths both ways
//! (`Bounds::bridge`): length points are bounded by the known lengths and
//! the deck (a suit known 6-8 gives 2-4), shortness likewise, so a call
//! showing 12+ total points with a known suit raises the HCP floor, and a
//! known HCP floor raises the points.

use bidspec::ast::{CmpOp, Expr};
use serde::Serialize;

use crate::facts::Valuation;

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

/// The most length points a hand can hold: thirteen cards in one suit.
const MAX_LENGTH_POINTS: i32 = 9;

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
    /// Support points with each suit as trump, C D H S, in whole points:
    /// HCP plus shortness capped by the trump length. A raise shows these
    /// and nothing else, so without them partner's floor for a slam
    /// decision was unknowable.
    pub tp: [Range; 4],
    /// Declarer points, whole: HCP plus one for each card beyond four in
    /// every suit (`hcp + length_points`, the count most rules write).
    pub dp: Range,
    /// How this seat's side counts `points` and `suit_points`.
    #[serde(skip)]
    pub valuation: Valuation,
    /// Everything this seat has shown or denied, resolved so that it refers
    /// only to the seat's own hand (see `eval::resolve`), printed.
    pub shown: Vec<String>,
    #[serde(skip)]
    pub constraints: Vec<Expr>,
}

impl Default for SeatKnowledge {
    fn default() -> Self {
        SeatKnowledge::with_valuation(Valuation::default())
    }
}

impl SeatKnowledge {
    /// Nothing known, before the point counts are tied to the HCP.
    fn unknown(valuation: Valuation) -> SeatKnowledge {
        SeatKnowledge {
            hcp: Range::new(0, 37),
            len: [Range::new(0, 13); 4],
            balanced: Tri::Unknown,
            pts: [Range::new(0, 4 * 37 + 4 * 4 + 4 * 9); 2],
            tp: [Range::new(0, 37 + 13); 4],
            dp: Range::new(0, 37 + MAX_LENGTH_POINTS),
            valuation,
            shown: Vec::new(),
            constraints: Vec::new(),
        }
    }
}

/// `5+` for a range open at the top, `0-3`, `4`.
fn open_range(r: Range, max: i32) -> String {
    if r.hi >= max && r.lo > 0 {
        format!("{}+", r.lo)
    } else {
        r.to_string()
    }
}

impl SeatKnowledge {
    /// What is known, in a line: the HCP range and suit lengths that are
    /// narrower than nothing known, and balance (`11-15 HCP, 5+ S,
    /// 0-3 H`). Empty when nothing is known.
    pub fn summary(&self) -> String {
        let fresh = SeatKnowledge::default();
        let mut parts = Vec::new();
        if self.hcp != fresh.hcp {
            parts.push(format!("{} HCP", open_range(self.hcp, fresh.hcp.hi)));
        }
        // Spades first, as a hand is written.
        for (i, name) in [(3, "S"), (2, "H"), (1, "D"), (0, "C")] {
            if self.len[i] != fresh.len[i] {
                parts.push(format!("{} {name}", open_range(self.len[i], 13)));
            }
        }
        match self.balanced {
            Tri::True => parts.push("balanced".into()),
            Tri::False => parts.push("unbalanced".into()),
            Tri::Unknown => {}
        }
        parts.join(", ")
    }

    pub fn is_contradiction(&self) -> bool {
        self.bounds().is_contradiction()
    }

    /// Nothing known yet, for a side that counts points by `valuation`.
    pub fn with_valuation(valuation: Valuation) -> SeatKnowledge {
        let mut k = SeatKnowledge::unknown(valuation);
        let mut b = k.bounds();
        b.normalize();
        k.set_bounds(b);
        k
    }

    fn bounds(&self) -> Bounds {
        Bounds {
            hcp: self.hcp,
            len: self.len,
            balanced: self.balanced,
            pts: self.pts,
            tp: self.tp,
            dp: self.dp,
            v: self.valuation,
        }
    }

    fn set_bounds(&mut self, b: Bounds) {
        self.hcp = b.hcp;
        self.len = b.len;
        self.balanced = b.balanced;
        self.pts = b.pts;
        self.tp = b.tp;
        self.dp = b.dp;
    }

    /// Length points (one a card beyond four in every suit), as far as the
    /// lengths, the deck and the declarer points allow.
    pub fn length_points(&self) -> Range {
        let lp = derived(&self.len).lp;
        lp.intersect(Range::new(
            self.dp.lo - self.hcp.hi,
            self.dp.hi - self.hcp.lo,
        ))
    }

    /// The ranges the HCP and lengths alone allow for this kind of points
    /// (quarters), with nothing shown about points themselves.
    fn implied_points(&self, kind: usize) -> Range {
        let (ten, w) = weights(self.valuation, kind);
        let lp = derived(&self.len).lp;
        Range::new(
            4 * self.hcp.lo + w * lp.lo,
            4 * self.hcp.hi + 4 * ten + w * lp.hi,
        )
    }

    /// Have the calls said something about declarer points beyond what the
    /// HCP and lengths imply?
    pub fn declarer_points_shown(&self) -> bool {
        let lp = derived(&self.len).lp;
        self.dp.lo > self.hcp.lo + lp.lo || self.dp.hi < self.hcp.hi + lp.hi
    }

    /// Total points as whole points (fractions dropped), for comparisons:
    /// the shown range when a call showed that kind of points, else the
    /// other kind if shown (a suit invitation, then a notrump decision: both
    /// measure the same strength), else the HCP range with its floor raised
    /// to what the HCP floor and the known lengths make.
    pub fn whole_points(&self, kind: usize) -> Range {
        let k = if self.points_shown(kind) {
            kind
        } else if self.points_shown(1 - kind) {
            1 - kind
        } else {
            // Nothing shown about points: the HCP, with the floor the
            // HCP and the known lengths give the points.
            let floor = self.pts[kind].lo.div_euclid(4).max(self.hcp.lo);
            return Range::new(floor, self.hcp.hi.max(floor));
        };
        Range::new(self.pts[k].lo.div_euclid(4), self.pts[k].hi.div_euclid(4))
    }

    /// Has a call said something about this kind of points beyond what the
    /// HCP range implies?
    pub fn points_shown(&self, kind: usize) -> bool {
        let p = self.pts[kind];
        let implied = self.implied_points(kind);
        p.lo > implied.lo || p.hi < implied.hi
    }

    /// Record a constraint (already resolved to this seat's own hand) and
    /// narrow the ranges by it. Returns false if it contradicts what was
    /// known, in which case the ranges are left unchanged.
    pub fn add(&mut self, e: Expr) -> bool {
        match self.added(&e) {
            Some(b) => {
                self.shown.push(e.to_string());
                self.constraints.push(e);
                self.set_bounds(b);
                true
            }
            None => false,
        }
    }

    /// The ranges `add(e)` leaves, or `None` if `e` contradicts them.
    fn added(&self, e: &Expr) -> Option<Bounds> {
        let narrowed = narrow(self.bounds(), e, true);
        if narrowed.is_contradiction() {
            return None;
        }
        // Earlier disjunctions may settle now ("4 hearts or 4 spades",
        // then "not 4 hearts"): narrow by everything once more, `e` last
        // (it is the last constraint once recorded).
        let mut b = narrowed;
        for c in self.constraints.iter().chain(std::iter::once(e)) {
            let again = narrow(b, c, true);
            if !again.is_contradiction() {
                b = again;
            }
        }
        Some(b)
    }

    /// A copy whose ranges are exactly what `add(e)` would leave on a
    /// clone (unchanged if `e` contradicts them), without copying what
    /// the seat has shown: its `shown` and `constraints` are empty. For a
    /// short-lived view that reads only the ranges (another seat capped by
    /// my own HCP while choosing, `eval::Ctx::seat_attr`), where cloning
    /// every constraint was most of the cost.
    pub fn narrowed_view(&self, e: &Expr) -> SeatKnowledge {
        let mut k = SeatKnowledge {
            shown: Vec::new(),
            constraints: Vec::new(),
            ..SeatKnowledge::unknown(self.valuation)
        };
        k.set_bounds(self.added(e).unwrap_or_else(|| self.bounds()));
        k
    }

    /// Narrow by a fact the deck implies (another seat's shown length),
    /// without recording it among what this seat has shown: the ranges
    /// take it, and earlier disjunctions narrow again, but `constraints`
    /// (the calls' own meaning, which keys the sample of consistent hands)
    /// stay as they were.
    pub fn narrow_by_deck(&mut self, e: &Expr) {
        let narrowed = narrow(self.bounds(), e, true);
        if narrowed.is_contradiction() {
            return;
        }
        let mut b = narrowed;
        for c in &self.constraints {
            let again = narrow(b, c, true);
            if !again.is_contradiction() {
                b = again;
            }
        }
        self.set_bounds(b);
    }

    /// What the constraints alone say, without the deck's narrowing from
    /// other seats: the same ranges whenever the constraints are the same.
    pub fn from_constraints(&self) -> SeatKnowledge {
        let mut k = SeatKnowledge::with_valuation(self.valuation);
        for c in &self.constraints {
            k.add(c.clone());
        }
        k
    }
}

/// The ranges `narrow` works on: the part of `SeatKnowledge` that is cheap
/// to copy (no printed or stored constraints).
#[derive(Clone, Copy, PartialEq)]
struct Bounds {
    hcp: Range,
    len: [Range; 4],
    balanced: Tri,
    pts: [Range; 2],
    /// Support points per trump suit, C D H S (whole points).
    tp: [Range; 4],
    /// Declarer points: HCP plus length points (whole points).
    dp: Range,
    v: Valuation,
}

/// Per kind of points (0 `points`, 1 `suit_points`), in quarters: what a
/// ten adds and what a card beyond four adds (`facts::Facts::points_q`).
fn weights(v: Valuation, kind: usize) -> (i32, i32) {
    if kind == 0 {
        (v.ten, v.nt_length)
    } else {
        (0, v.length)
    }
}

/// The most `f` reaches over the hands whose suit lengths lie in `len` and
/// add up to 13. `f` is convex in the lengths (length points, shortness),
/// so its maximum over that polytope is at a vertex: three lengths at an
/// end of their ranges, the fourth what the deck leaves. `None` when no
/// length assignment fits.
fn vertex_max(len: &[Range; 4], f: impl Fn(&[i32; 4]) -> i32) -> Option<i32> {
    let mut best: Option<i32> = None;
    for free in 0..4 {
        let others: Vec<usize> = (0..4).filter(|&j| j != free).collect();
        for mask in 0..8 {
            let mut l = [0; 4];
            for (bit, &j) in others.iter().enumerate() {
                l[j] = if mask & (1 << bit) != 0 {
                    len[j].hi
                } else {
                    len[j].lo
                };
            }
            l[free] = 13 - others.iter().map(|&j| l[j]).sum::<i32>();
            if len[free].lo <= l[free] && l[free] <= len[free].hi {
                let v = f(&l);
                best = Some(best.map_or(v, |b| b.max(v)));
            }
        }
    }
    best
}

/// Length points the known lengths allow: at least the sum of each suit's
/// sure excess over four, at most what the most lopsided hand that fits
/// the ranges and the deck holds.
fn length_points(len: &[Range; 4]) -> Range {
    if len.iter().any(Range::is_empty) {
        return Range::new(0, MAX_LENGTH_POINTS);
    }
    let lo = len.iter().map(|r| (r.lo - 4).max(0)).sum();
    let hi =
        vertex_max(len, |l| l.iter().map(|x| (x - 4).max(0)).sum()).unwrap_or(MAX_LENGTH_POINTS);
    Range::new(lo, hi.min(MAX_LENGTH_POINTS))
}

/// Shortness points for one side suit (`Facts::total_points`).
fn shortness(l: i32) -> i32 {
    match l {
        0 => 5,
        1 => 3,
        2 => 1,
        _ => 0,
    }
}

/// What support points add to the HCP with `t` as trump: shortness in the
/// side suits, capped by the trumps held.
fn support_bonus(len: &[Range; 4], t: usize) -> Range {
    if len.iter().any(Range::is_empty) {
        return Range::new(0, 13);
    }
    let side = |l: &[i32; 4]| {
        (0..4)
            .filter(|&j| j != t)
            .map(|j| shortness(l[j]))
            .sum::<i32>()
    };
    let lo: i32 = (0..4)
        .filter(|&j| j != t)
        .map(|j| shortness(len[j].hi))
        .sum();
    let hi = vertex_max(len, side).unwrap_or(15);
    Range::new(lo.min(len[t].lo), hi.min(len[t].hi))
}

/// What the lengths alone bound: the length points and, per trump suit,
/// what shortness adds to the HCP.
#[derive(Clone, Copy)]
struct Derived {
    lp: Range,
    bonus: [Range; 4],
}

/// `Derived` for these lengths, from a small per-thread cache: narrowing
/// asks for the same few length ranges over and over, and each answer
/// takes a walk over the vertices.
fn derived(len: &[Range; 4]) -> Derived {
    let compute = || Derived {
        lp: length_points(len),
        bonus: std::array::from_fn(|t| support_bonus(len, t)),
    };
    if len.iter().any(|r| r.lo < 0 || r.hi > 13 || r.is_empty()) {
        return compute();
    }
    // Eight numbers 0..=13, four bits each.
    let key = len
        .iter()
        .fold(0u32, |k, r| (k << 8) | ((r.lo as u32) << 4) | r.hi as u32);
    const SLOTS: usize = 1 << 12;
    thread_local! {
        static CACHE: std::cell::RefCell<Vec<Option<(u32, Derived)>>> =
            std::cell::RefCell::new(vec![None; SLOTS]);
    }
    let slot = (key.wrapping_mul(0x9e37_79b1) >> 20) as usize % SLOTS;
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        match c[slot] {
            Some((k, d)) if k == key => d,
            _ => {
                let d = compute();
                c[slot] = Some((key, d));
                d
            }
        }
    })
}

/// `-(-a).div_euclid(b)`: division rounding up.
fn ceil_div(a: i32, b: i32) -> i32 {
    -(-a).div_euclid(b)
}

impl Bounds {
    fn is_contradiction(&self) -> bool {
        self.hcp.is_empty()
            || self.pts.iter().any(Range::is_empty)
            || self.tp.iter().any(Range::is_empty)
            || self.dp.is_empty()
            || self.len.iter().any(Range::is_empty)
    }

    /// Deck and shape consequences: lengths sum to 13; balanced means every
    /// suit has 2 to 5 cards. Then the point counts against the HCP and
    /// the lengths (`bridge`).
    fn normalize(&mut self) {
        for _ in 0..3 {
            let before = *self;
            self.shape();
            if self.is_contradiction() {
                return;
            }
            self.bridge();
            if *self == before {
                break;
            }
        }
        if self.balanced == Tri::Unknown && self.len.iter().any(|r| r.hi <= 1 || r.lo >= 6) {
            self.balanced = Tri::False;
        }
    }

    fn shape(&mut self) {
        if self.balanced == Tri::True {
            for r in &mut self.len {
                *r = r.intersect(Range::new(2, 5));
            }
        }
        for _ in 0..2 {
            for i in 0..4 {
                // Clamped to the deck first: an open range ends at i32::MAX,
                // and three of those overflow the sum.
                let others = (0..4).filter(|&j| j != i);
                let others_lo: i32 = others.clone().map(|j| self.len[j].lo.clamp(0, 14)).sum();
                let others_hi: i32 = others.map(|j| self.len[j].hi.clamp(-1, 13)).sum();
                self.len[i] = self.len[i].intersect(Range::new(13 - others_hi, 13 - others_lo));
            }
        }
    }

    /// Each point count is the HCP plus a bonus the lengths bound, so each
    /// bounds the other: points >= HCP + the least bonus, HCP >= points -
    /// the most bonus, and the same at the top. Declarer points are HCP +
    /// length points (`dp - hcp` also bounds the length points, and so the
    /// lengths); `points` and `suit_points` add tens and length by the
    /// side's valuation; support points add shortness capped by the trumps.
    /// A raise showing 10-12 support points therefore says 12 is the
    /// ceiling on the high cards, and a 2♥ overcall of 12+ total points
    /// with five or six hearts says 10+ HCP.
    fn bridge(&mut self) {
        let mut d = derived(&self.len);
        let lp = d.lp;
        // Declarer points.
        self.dp = self
            .dp
            .intersect(Range::new(self.hcp.lo + lp.lo, self.hcp.hi + lp.hi));
        self.hcp = self
            .hcp
            .intersect(Range::new(self.dp.lo - lp.hi, self.dp.hi - lp.lo));
        let len = self.len;
        self.narrow_length_points(Range::new(
            self.dp.lo - self.hcp.hi,
            self.dp.hi - self.hcp.lo,
        ));
        if self.len != len {
            d = derived(&self.len);
        }
        let lp = d.lp;
        // Total points, quarters: 4 HCP + ten a ten + w a card beyond four.
        for kind in 0..2 {
            let (ten, w) = weights(self.v, kind);
            let bonus = Range::new(w * lp.lo, 4 * ten + w * lp.hi);
            let p = &mut self.pts[kind];
            *p = p.intersect(Range::new(
                4 * self.hcp.lo + bonus.lo,
                4 * self.hcp.hi + bonus.hi,
            ));
            self.hcp = self.hcp.intersect(Range::new(
                ceil_div(p.lo - bonus.hi, 4),
                (p.hi - bonus.lo).div_euclid(4),
            ));
            // Against declarer points: points - 4 dp = ten a ten + (w - 4)
            // a card beyond four.
            let d = w - 4;
            let (dl, dh) = if d >= 0 {
                (d * lp.lo, 4 * ten + d * lp.hi)
            } else {
                (d * lp.hi, 4 * ten + d * lp.lo)
            };
            *p = p.intersect(Range::new(4 * self.dp.lo + dl, 4 * self.dp.hi + dh));
            self.dp = self.dp.intersect(Range::new(
                ceil_div(p.lo - dh, 4),
                (p.hi - dl).div_euclid(4),
            ));
        }
        // Support points, whole: HCP + shortness capped by the trumps.
        for t in 0..4 {
            let b = d.bonus[t];
            let r = &mut self.tp[t];
            *r = r.intersect(Range::new(self.hcp.lo + b.lo, self.hcp.hi + b.hi));
            self.hcp = self.hcp.intersect(Range::new(r.lo - b.hi, r.hi - b.lo));
        }
    }

    /// Lengths from a bound on the length points: with at most `m`, no
    /// suit is longer than four plus `m` less what the other suits surely
    /// add; with at least `m`, a suit must make up what the others cannot.
    fn narrow_length_points(&mut self, r: Range) {
        let excess = |x: i32| (x - 4).max(0);
        for i in 0..4 {
            let others_lo: i32 = (0..4)
                .filter(|&j| j != i)
                .map(|j| excess(self.len[j].lo))
                .sum();
            let others_hi: i32 = (0..4)
                .filter(|&j| j != i)
                .map(|j| excess(self.len[j].hi))
                .sum();
            // Below four only when the bound is already contradicted (the
            // declarer-point intersection has emptied the range then).
            self.len[i].hi = self.len[i].hi.min(4 + r.hi - others_lo);
            let need = r.lo - others_hi;
            if need > 0 {
                self.len[i].lo = self.len[i].lo.max(4 + need);
            }
        }
    }

    fn contradiction() -> Bounds {
        Bounds {
            hcp: Range::new(1, 0),
            ..SeatKnowledge::unknown(Valuation::default()).bounds()
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
        for i in 0..4 {
            k.tp[i] = self.tp[i].hull(o.tp[i]);
        }
        k.dp = self.dp.hull(o.dp);
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
            Attr::Tp(s) => &mut self.tp[s],
            Attr::Dp => &mut self.dp,
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
    /// Support points with one suit as trump, in whole points.
    Tp(usize),
    /// Declarer points, `hcp + length_points` written together.
    Dp,
}

/// A term of a comparison as `narrow` sees it: one attribute, the length
/// points alone (not stored: they bound the lengths), or a sum of suit
/// lengths (`S + H >= 9`).
#[derive(Clone)]
enum Term {
    One(Attr),
    LengthPoints,
    Lengths(Vec<usize>),
}

/// One summand of `sum_of`.
#[derive(Clone, Copy)]
enum Part {
    A(Attr),
    LengthPoints,
}

fn attr_of(e: &Expr) -> Option<Attr> {
    let Expr::Path { path } = e else { return None };
    if path.len() != 1 {
        return None;
    }
    // `tp(S)`: the suit is a literal by the time a `shows` is recorded, a
    // bound variable having been substituted by `eval::term`.
    if let Some(args) = &path[0].args {
        if path[0].name == "tp" && args.len() == 1 {
            if let Expr::Path { path: arg } = &args[0] {
                if arg.len() == 1 {
                    return suit_index(&arg[0].name).map(Attr::Tp);
                }
            }
        }
        return None;
    }
    match path[0].name.as_str() {
        "hcp" => Some(Attr::Hcp),
        "points" => Some(Attr::Pts(0)),
        "suit_points" => Some(Attr::Pts(1)),
        n => suit_index(n).map(Attr::Len),
    }
}

/// The attributes added up in `e`, and the constant: `hcp +
/// length_points - 1` is `([hcp, length points], -1)`. `None` when an
/// attribute is taken away or anything else appears.
fn sum_of(e: &Expr) -> Option<(Vec<Part>, i32)> {
    use bidspec::ast::ArithOp;
    match e {
        Expr::Arith { arith, lhs, rhs } => {
            let (mut l, lk) = sum_of(lhs)?;
            let (r, rk) = sum_of(rhs)?;
            if *arith == ArithOp::Add {
                l.extend(r);
                Some((l, lk + rk))
            } else if r.is_empty() {
                Some((l, lk - rk))
            } else {
                None
            }
        }
        Expr::Int { value } => Some((vec![], *value as i32)),
        Expr::Path { path }
            if path.len() == 1 && path[0].args.is_none() && path[0].name == "length_points" =>
        {
            Some((vec![Part::LengthPoints], 0))
        }
        _ => attr_of(e).map(|a| (vec![Part::A(a)], 0)),
    }
}

/// `e` as a term `narrow` can bound, and the constant added to it:
/// `hcp + length_points + 2` is declarer points plus 2, `S + 4` spades
/// plus 4, `S + H` a sum of lengths.
fn linear(e: &Expr) -> Option<(Term, i32)> {
    let (parts, k) = sum_of(e)?;
    let term = match parts.as_slice() {
        [Part::A(a)] => Term::One(*a),
        [Part::LengthPoints] => Term::LengthPoints,
        [Part::A(Attr::Hcp), Part::LengthPoints] | [Part::LengthPoints, Part::A(Attr::Hcp)] => {
            Term::One(Attr::Dp)
        }
        many if many.len() > 1 => Term::Lengths(
            many.iter()
                .map(|p| match p {
                    Part::A(Attr::Len(s)) => Some(*s),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?,
        ),
        _ => return None,
    };
    Some((term, k))
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
            // A term plus constants against a number: a bare attribute, a
            // resolved partnership sum (`S + 4 >= 8`: four or more),
            // declarer points (`hcp + length_points >= 12`).
            if let (Some((t, k)), Some(n)) = (linear(lhs), int_of(rhs)) {
                constrain(&mut out, &t, op, n - k);
                out.normalize();
                return out;
            }
            if let (Some(n), Some((t, k))) = (int_of(lhs), linear(rhs)) {
                constrain(&mut out, &t, flip(op), n - k);
                out.normalize();
                return out;
            }
            match (attr_of(lhs), attr_of(rhs)) {
                // Relations between two attributes (S>=H); not for points,
                // which are in different units.
                (Some(a), Some(b)) if !matches!(a, Attr::Pts(_)) && !matches!(b, Attr::Pts(_)) => {
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
            if let (Some((t, k)), Some(lo), Some(hi)) = (linear(expr), int_of(lo), int_of(hi)) {
                let (lo, hi) = (lo - k, hi - k);
                if positive {
                    constrain(&mut out, &t, CmpOp::Ge, lo);
                    constrain(&mut out, &t, CmpOp::Le, hi);
                } else {
                    // Below or above: the hull of the two, each narrowed
                    // through to the other ranges first.
                    let mut below = out;
                    constrain(&mut below, &t, CmpOp::Lt, lo);
                    below.normalize();
                    let mut above = out;
                    constrain(&mut above, &t, CmpOp::Gt, hi);
                    above.normalize();
                    return below.hull(&above);
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

/// Is `e` held exactly by the ranges once narrowed by it: a conjunction
/// of bounds on HCP, a suit length, a kind of points, support points or
/// declarer points, and balance? A hand inside the ranges then satisfies
/// it, so a filter over hands may skip it.
pub fn held_by_ranges(e: &Expr) -> bool {
    let exact = |t: Option<(Term, i32)>| matches!(t, Some((Term::One(_), _)));
    match e {
        Expr::And { all } => all.iter().all(held_by_ranges),
        Expr::Cmp { cmp, lhs, rhs } => {
            *cmp != CmpOp::Ne
                && ((exact(linear(lhs)) && int_of(rhs).is_some())
                    || (int_of(lhs).is_some() && exact(linear(rhs))))
        }
        Expr::InRange { expr, lo, hi } => {
            exact(linear(expr)) && int_of(lo).is_some() && int_of(hi).is_some()
        }
        Expr::Path { path } => path.len() == 1 && path[0].name == "balanced",
        Expr::Not { expr } => {
            matches!(&**expr, Expr::Path { path } if path.len() == 1 && path[0].name == "balanced")
        }
        _ => false,
    }
}

/// Bound `t op n` into `out` (before `normalize` carries it through).
fn constrain(out: &mut Bounds, t: &Term, op: CmpOp, n: i32) {
    match t {
        Term::One(a) => {
            let r = out.range_mut(*a);
            *r = apply_attr(*a, *r, op, n);
        }
        Term::LengthPoints => {
            if op != CmpOp::Ne {
                let r = apply_const(Range::new(0, MAX_LENGTH_POINTS), op, n);
                if r.is_empty() {
                    *out = Bounds::contradiction();
                } else {
                    out.narrow_length_points(r);
                }
            }
        }
        Term::Lengths(suits) => {
            // Each length is at least the sum's floor less what the
            // others can hold, at most its ceiling less what they must.
            let r = apply_const(Range::new(0, 13 * suits.len() as i32), op, n);
            if op == CmpOp::Ne {
                return;
            }
            for (i, &s) in suits.iter().enumerate() {
                let others = suits.iter().enumerate().filter(|&(j, _)| j != i);
                let lo: i32 = others.clone().map(|(_, &o)| out.len[o].lo).sum();
                let hi: i32 = others.map(|(_, &o)| out.len[o].hi).sum();
                out.len[s] = out.len[s].intersect(Range::new(r.lo - hi, r.hi - lo));
            }
        }
    }
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
    fn narrows_a_partnership_sum() {
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("S + 4 >= 8")));
        assert_eq!(k.len[3].lo, 4);
        assert!(k.add(expr("H + 3 + 1 - 1 = 8")));
        assert_eq!((k.len[2].lo, k.len[2].hi), (5, 5));
        assert!(k.add(expr("points + 15 <= 24")));
        assert_eq!(k.whole_points(0).hi, 9);
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
    fn a_raise_records_support_points_and_bounds_the_high_cards() {
        let mut k = SeatKnowledge::default();
        // A limit raise: support points and nothing else, which used to
        // leave partner's strength completely unknown.
        assert!(k.add(expr("tp(S)=10..12, S>=4")));
        assert_eq!(k.tp[3], Range::new(10, 12));
        assert_eq!(k.hcp.hi, 12, "support points are at least the high cards");
        // Only spades were agreed; hearts keep the bound the high cards
        // and the possible heart length imply.
        assert_eq!(k.tp[2], Range::new(0, k.hcp.hi + k.len[2].hi));
        // Support points never fall below the high cards.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("hcp>=13")));
        assert_eq!(k.tp[3].lo, 13);
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

    /// Total points raise the HCP floor by what length can add at most,
    /// given the known lengths (and the deck), and the HCP floor raises
    /// the points. Soundness on random hands: eval.rs,
    /// `knowledge_soundness`.
    #[test]
    fn points_and_hcp_bound_each_other() {
        // A 2♥ overcall, 12+ total points, with the shape pinned: six
        // hearts and nothing else beyond four make two length points.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("H=6, S<=3, D<=3, C<=4, hcp+length_points>=12")));
        assert_eq!(k.hcp.lo, 10);
        assert_eq!(k.dp.lo, 12);
        assert_eq!(k.whole_points(0).lo, 12, "declarer points feed `points`");
        // Five or six hearts, no other suit beyond four: one or two.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("H>=5, hcp+length_points>=12")));
        assert!(k.hcp.lo < 10, "a long suit could make the points");
        assert!(k.add(expr("H<=6, S<=4, D<=4, C<=4")));
        assert_eq!(k.hcp.lo, 10, "earlier constraints narrow again");
        // A known HCP floor and a six-card suit: 17+ declarer points.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("hcp>=15, H>=6")));
        assert_eq!(k.dp.lo, 17);
        assert_eq!(k.whole_points(1).lo, 16, "suit points: half a point a card");
        // Total points by BBA's count (no length at notrump): tens only.
        let bba = Valuation {
            nt_length: 0,
            ..Valuation::default()
        };
        let mut k = SeatKnowledge::with_valuation(bba);
        assert!(k.add(expr("points>=13")));
        assert_eq!(k.hcp.lo, 11, "four tens at a half");
        // The default count adds length, so a 10-HCP hand can hold more
        // than 12 points (the old bound assumed tens only).
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("hcp<=10")));
        assert!(k.pts[0].hi >= 4 * 10 + 8 + 4 * 9);
    }

    #[test]
    fn declarer_points_bound_the_lengths() {
        // 13+ with at most 10 HCP: three length points, only clubs left.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("hcp<=10, S<=4, H<=4, D<=4, hcp+length_points>=13")));
        assert_eq!(k.len[0].lo, 7);
        // No length points: no suit beyond four.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("length_points<=0")));
        assert!(k.len.iter().all(|r| r.hi == 4));
        // A sum of lengths.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("S+H>=9, S<=4")));
        assert_eq!(k.len[2].lo, 5);
        // An in-range of declarer points narrows both ends.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("hcp+length_points=9..11, H=4, S=3, D=3, C=3")));
        assert_eq!(k.hcp, Range::new(9, 11));
    }

    #[test]
    fn support_points_bound_the_high_cards_from_below() {
        // 19+ support points with five trumps and no side suit shorter
        // than two: eight side cards make two doubletons at most (2-2-4),
        // two points.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("tp(H)>=19, H=5, S>=2, D>=2, C>=2")));
        assert_eq!(k.hcp.lo, 17);
        // Shortness is capped by the trumps: three trumps add three.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr("tp(S)>=12, S=3")));
        assert_eq!(k.hcp.lo, 9);
    }

    #[test]
    fn a_total_point_branch_collapses() {
        // The takeout double: 12+ and the shape, or 17+ HCP. Limited later
        // to 16 in total points, the power branch is out.
        let mut k = SeatKnowledge::default();
        assert!(k.add(expr(
            "hcp+length_points>=12, ((S>=3, D>=3, C>=3) | hcp>=17)"
        )));
        assert_eq!(k.len[3].lo, 0);
        assert!(k.add(expr("hcp+length_points<=16")));
        assert_eq!((k.len[3].lo, k.len[1].lo, k.len[0].lo), (3, 3, 3));
        // A pass that denies the double caps the HCP at 16: either under
        // 12 in total points, or without the shape and under 17.
        let mut k = SeatKnowledge::default();
        assert!(k.add(Expr::Not {
            expr: Box::new(expr(
                "hcp+length_points>=12, ((S>=3, D>=3, C>=3) | hcp>=17)"
            )),
        }));
        assert_eq!(k.hcp.hi, 16);
    }

    #[test]
    fn contradictions_are_rejected() {
        let mut k = SeatKnowledge::default();
        k.add(expr("hcp<=7"));
        assert!(!k.add(expr("hcp>=10")));
        assert_eq!(k.hcp, Range::new(0, 7));
    }
}
