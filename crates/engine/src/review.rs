//! Reviewing an auction call by call: what is known about every seat after
//! each call, each side's state, the flags a call raises (forcing, game
//! forcing, invitational, alerted, artificial) and, when the hands are
//! known, how each hand values itself at that point.
//!
//! Computed on demand by replaying the auction (`Engine::review`), so the
//! comparison's per-board results stay as small as they were: the workbench's
//! knowledge view and `rbb explain-auction` call it for one board at a time.

use bidspec::ast::Alert;
use bridge_types::{Call, Direction, Hand, ScoringMethod, Strain, Vulnerability};
use serde::Serialize;

use crate::engine::{Engine, Step};
use crate::eval::{strain_of_suit, suit_of_strain};
use crate::facts::{Facts, Valuation};
use crate::knowledge::{Range, SeatKnowledge, Tri};
use crate::position::{side, Forcing, SideState};

/// Seats by `Direction::to_index`.
const SEATS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

/// One call of a reviewed auction and everything known after it.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewRow {
    /// 0-based, from the dealer's first call.
    pub index: usize,
    /// How the call was read (caller, call, rule, explanation, alert).
    pub step: Step,
    /// What is known about every seat after the call, by
    /// `Direction::to_index`. Not only the caller's can move: the deck's 40
    /// HCP caps the others.
    pub seats: [SeatKnowledge; 4],
    /// What this call narrowed, per seat.
    pub narrowed: [Narrowed; 4],
    /// Both sides' state after the call, by `side`.
    pub sides: [SideState; 2],
    pub flags: Flags,
    /// Each hand's own view of itself after the call, where the hand is
    /// known.
    pub own: [Option<OwnView>; 4],
}

/// Which parts of a seat's knowledge one call changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Narrowed {
    pub hcp: bool,
    /// C D H S.
    pub len: [bool; 4],
    pub balanced: bool,
    pub points: bool,
    /// Support points with each suit as trump, C D H S.
    pub tp: [bool; 4],
    /// New lines in `shown` (constraints the ranges may not capture).
    pub shown: usize,
}

impl Narrowed {
    pub fn any(&self) -> bool {
        self.hcp
            || self.len.iter().any(|&b| b)
            || self.balanced
            || self.points
            || self.tp.iter().any(|&b| b)
            || self.shown > 0
    }

    fn between(a: &SeatKnowledge, b: &SeatKnowledge) -> Narrowed {
        Narrowed {
            hcp: a.hcp != b.hcp,
            len: std::array::from_fn(|i| a.len[i] != b.len[i]),
            balanced: a.balanced != b.balanced,
            points: a.pts != b.pts || a.dp != b.dp,
            tp: std::array::from_fn(|i| a.tp[i] != b.tp[i]),
            shown: b.shown.len().saturating_sub(a.shown.len()),
        }
    }
}

/// What a call does beyond what it shows, for the calling side.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Flags {
    /// The call is forcing for one round (it set `forcing=round`).
    pub forcing: bool,
    /// The call established a game force (by its rule's `sets` or by a
    /// `force game after` line).
    pub game_force: bool,
    /// The side was already in a game force set by an earlier call.
    pub in_game_force: bool,
    /// The question the call asks partner, as `kind(args)`, when it set one.
    pub ask: Option<String>,
    /// The call invites (its question is an invitation: `invite`,
    /// `nt_invite`, `fit_invite`, `limit_raise`, ...).
    pub invitational: bool,
    /// The call is alerted (`Some("")` with no text).
    pub alert: Option<String>,
    /// The call is announced, with the announcement.
    pub announce: Option<String>,
    /// The rule marks the call artificial: not a place to play.
    pub artificial: bool,
    /// The trump suit this call agreed, if it set one.
    pub agrees: Option<Strain>,
}

impl Flags {
    /// Short tags: `F1`, `GF`, `inv`, `ask keycards(S)`, `alert`, `art`.
    pub fn tags(&self) -> Vec<String> {
        let mut v = Vec::new();
        if self.game_force {
            v.push("GF".to_string());
        } else if self.forcing {
            v.push("F1".to_string());
        }
        if self.invitational {
            v.push("inv".to_string());
        }
        if let Some(a) = &self.ask {
            v.push(format!("ask {a}"));
        }
        if let Some(t) = self.agrees {
            v.push(format!("agrees {}", strain_symbol(t)));
        }
        if self.alert.is_some() {
            v.push("alert".to_string());
        }
        if self.announce.is_some() {
            v.push("announced".to_string());
        }
        if self.artificial {
            v.push("art".to_string());
        }
        v
    }
}

/// How a hand values itself after a call: HCP, total points, and, when a
/// trump suit is agreed or the hand can see an eight-card fit with what
/// partner has shown, the count by role that the slam conditions use
/// (conventions/slam/slam-entry.bid, Rick 2026-10-01).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OwnView {
    pub hcp: i32,
    /// Total points as the `points` term counts them (HCP, tens, length).
    pub points: i32,
    /// One for each card beyond four in every suit.
    pub length_points: i32,
    pub fit: Option<FitView>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FitView {
    pub trump: Strain,
    /// Agreed in the auction (`we.trump`), or only seen by this hand.
    pub agreed: bool,
    /// Trumps held, and the most partner has shown at least.
    pub mine: i32,
    pub partner_shown: i32,
    pub role: Role,
    /// `hcp + length_points` for the long hand, `tp(trump)` otherwise.
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// More trumps than partner has shown: declarer points (HCP + length).
    Declarer,
    /// Fewer: support points (HCP + shortness, capped by the trumps).
    Support,
    /// As many: both hands count support points.
    Equal,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Declarer => "decl",
            Role::Support => "sup",
            Role::Equal => "sup=",
        }
    }
}

impl OwnView {
    pub fn new(f: &Facts, v: Valuation, trump: Option<Strain>, partner: &SeatKnowledge) -> OwnView {
        let agreed = trump.and_then(suit_of_strain);
        // No suit agreed: the suit where my cards and partner's shown
        // length make eight, the most cards first, then the higher suit.
        let seen = || {
            (0..4)
                .filter(|&s| f.len[s] + partner.len[s].lo >= 8)
                .max_by_key(|&s| (f.len[s] + partner.len[s].lo, s))
        };
        let fit = agreed
            .map(|s| (s, true))
            .or_else(|| seen().map(|s| (s, false)));
        OwnView {
            hcp: f.hcp,
            points: f.points_q(v).div_euclid(4),
            length_points: f.length_points(),
            fit: fit.map(|(s, agreed)| {
                let (mine, theirs) = (f.len[s], partner.len[s].lo);
                let role = match mine.cmp(&theirs) {
                    std::cmp::Ordering::Greater => Role::Declarer,
                    std::cmp::Ordering::Less => Role::Support,
                    std::cmp::Ordering::Equal => Role::Equal,
                };
                let count = match role {
                    Role::Declarer => f.hcp + f.length_points(),
                    _ => f.total_points(Some(s)),
                };
                FitView {
                    trump: strain_of_suit(s),
                    agreed,
                    mine,
                    partner_shown: theirs,
                    role,
                    count,
                }
            }),
        }
    }

    /// `12 hcp 13 pts` and, with a fit, `· ♥ sup 14` (`♥?` when only this
    /// hand sees the fit).
    pub fn text(&self) -> String {
        let mut s = format!("{} hcp {} pts", self.hcp, self.points);
        if let Some(f) = &self.fit {
            s += &format!(
                " · {}{} {} {}",
                strain_symbol(f.trump),
                if f.agreed { "" } else { "?" },
                f.role.name(),
                f.count
            );
        }
        s
    }
}

pub fn strain_symbol(s: Strain) -> &'static str {
    match s {
        Strain::Clubs => "♣",
        Strain::Diamonds => "♦",
        Strain::Hearts => "♥",
        Strain::Spades => "♠",
        Strain::NoTrump => "NT",
    }
}

/// Asks that invite: partner accepts or declines.
fn is_invitation(kind: &str) -> bool {
    kind.contains("invite") || matches!(kind, "limit_raise" | "quant" | "slam_try")
}

/// One piece of a seat's compact knowledge line, and whether the row's
/// call changed it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Part {
    pub text: String,
    pub changed: bool,
}

/// `5+` for a range open at the top.
fn open(r: Range, max: i32) -> String {
    if r.hi >= max && r.lo > 0 {
        format!("{}+", r.lo)
    } else {
        r.to_string()
    }
}

/// What is known about a seat, compactly, in the parts that say something:
/// `15-17 hcp`, `♠2-4`, `bal`, `pts 16-19`, `♥sup 10-12`. A suit length or
/// a support count appears only when narrowed from nothing known. Each part
/// says whether `n` (the row's call) changed it.
pub fn knowledge_parts(k: &SeatKnowledge, n: &Narrowed) -> Vec<Part> {
    let fresh = SeatKnowledge::default();
    let mut parts = Vec::new();
    let mut push = |text: String, changed: bool| parts.push(Part { text, changed });
    if k.hcp != fresh.hcp {
        push(format!("{} hcp", open(k.hcp, fresh.hcp.hi)), n.hcp);
    }
    for s in [3, 2, 1, 0] {
        if k.len[s] != fresh.len[s] {
            push(
                format!("{}{}", strain_symbol(strain_of_suit(s)), open(k.len[s], 13)),
                n.len[s],
            );
        }
    }
    match k.balanced {
        Tri::True => push("bal".into(), n.balanced),
        Tri::False => push("unbal".into(), n.balanced),
        Tri::Unknown => {}
    }
    // Declarer points (HCP + length), which most rules show; then total
    // points when a call showed them (a strength band) and they say more
    // than the declarer points' floor.
    let decl = k.declarer_points_shown();
    if decl {
        push(format!("decl {}", open(k.dp, 37)), n.points);
    }
    for kind in [1, 0] {
        let p = k.whole_points(kind);
        if k.points_shown(kind) && (!decl || p.lo > k.dp.lo) {
            push(
                format!("{}pts {}", if kind == 0 { "nt " } else { "" }, open(p, 37)),
                n.points,
            );
            break;
        }
    }
    for s in [3, 2, 1, 0] {
        let t = k.tp[s];
        if t.lo > k.hcp.lo {
            push(
                format!("{}sup {}", strain_symbol(strain_of_suit(s)), open(t, 37)),
                n.tp[s],
            );
        }
    }
    parts
}

/// Everything known about a seat, for a tooltip: every range, then what it
/// has shown and denied.
pub fn knowledge_full(k: &SeatKnowledge) -> String {
    let mut s = format!(
        "{} HCP  ♠{} ♥{} ♦{} ♣{}  balanced {:?}\ndeclarer points {}  points (suit) {}  (notrump) {}\nsupport points ♠{} ♥{} ♦{} ♣{}",
        k.hcp,
        k.len[3],
        k.len[2],
        k.len[1],
        k.len[0],
        k.balanced,
        k.dp,
        k.whole_points(1),
        k.whole_points(0),
        k.tp[3],
        k.tp[2],
        k.tp[1],
        k.tp[0],
    );
    if !k.shown.is_empty() {
        s += "\n\nshown / denied:\n";
        s += &k.shown.join("\n");
    }
    s
}

/// A side's state in a few words: `trump ♥ · GF (N) · asked invite(H) by S`.
pub fn side_text(st: &SideState) -> String {
    let mut parts = vec![];
    if let Some(t) = st.trump {
        parts.push(format!("trump {}", strain_symbol(t)));
    }
    match (st.forcing, st.forcing_by) {
        (Forcing::None, _) => {}
        (f, by) => parts.push(format!(
            "{}{}",
            if f == Forcing::Game { "GF" } else { "F1" },
            by.map(|d| format!(" ({})", d.to_char()))
                .unwrap_or_default()
        )),
    }
    if let Some(a) = &st.ask {
        parts.push(format!("asked {} by {}", ask_text(a), a.by.to_char()));
    }
    if let Some(a) = &st.answered {
        parts.push(format!("answered {}", ask_text(a)));
    }
    parts.join(" · ")
}

fn ask_text(a: &crate::position::Ask) -> String {
    if a.args.is_empty() {
        a.kind.clone()
    } else {
        let args: Vec<&str> = a.args.iter().map(|s| strain_symbol(*s)).collect();
        format!("{}({})", a.kind, args.join(","))
    }
}

impl Engine {
    /// Replay `calls` and record, after each, what is known about every
    /// seat, both sides' state, the call's flags and (for the seats whose
    /// hand is given, by `Direction::to_index`) the hand's own view. The
    /// calls are read exactly as `interpret` reads them.
    pub fn review(
        &self,
        dealer: Direction,
        vul: Vulnerability,
        scoring: ScoringMethod,
        calls: &[Call],
        hands: &[Option<Hand>; 4],
    ) -> Vec<ReviewRow> {
        let facts: Vec<Option<Facts>> = hands.iter().map(|h| h.as_ref().map(Facts::new)).collect();
        let mut pos = self.start(dealer, vul, scoring);
        let mut rows = Vec::with_capacity(calls.len());
        for (index, call) in calls.iter().enumerate() {
            let before = pos.clone();
            let step = self.advance(&mut pos, call);
            let caller = step.caller;
            let (was, now) = (&before.sides[side(caller)], &pos.sides[side(caller)]);
            let ask = now
                .ask
                .as_ref()
                .filter(|a| a.by == caller && was.ask.as_ref() != Some(*a));
            let (alert, announce) = match &step.alert {
                Some(Alert::Alert { text }) => (Some(text.clone().unwrap_or_default()), None),
                Some(Alert::Announce { text }) => (None, Some(text.clone())),
                None => (None, None),
            };
            let game_force = now.forcing == Forcing::Game && was.forcing != Forcing::Game;
            let flags = Flags {
                forcing: now.forcing == Forcing::Round && now.forcing_by == Some(caller),
                game_force,
                in_game_force: was.forcing == Forcing::Game,
                ask: ask.map(ask_text),
                invitational: ask.is_some_and(|a| is_invitation(&a.kind)),
                alert,
                announce,
                artificial: step.artificial,
                agrees: now.trump.filter(|_| now.trump != was.trump),
            };
            let own = std::array::from_fn(|i| {
                let f = facts[i].as_ref()?;
                let seat = SEATS[i];
                Some(OwnView::new(
                    f,
                    self.valuation(seat),
                    pos.sides[side(seat)].trump,
                    pos.knowledge(seat.partner()),
                ))
            });
            rows.push(ReviewRow {
                index,
                narrowed: std::array::from_fn(|i| {
                    Narrowed::between(&before.knowledge[i], &pos.knowledge[i])
                }),
                seats: pos.knowledge.clone(),
                sides: pos.sides.clone(),
                step,
                flags,
                own,
            });
        }
        rows
    }
}

/// The review as text, one block per call: the call, its flags and
/// meaning, then each seat whose knowledge it changed (`*` marks the parts
/// it changed) with that hand's own view, and the calling side's state
/// when it moved. For a terminal or a ticket.
pub fn review_text(rows: &[ReviewRow]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for r in rows {
        let s = &r.step;
        let tags = r.flags.tags();
        let _ = write!(
            out,
            "{:>2}. {} {:<4} {}",
            r.index + 1,
            s.caller.to_char(),
            s.call.to_string(),
            s.explanation.as_deref().unwrap_or("(no rule)")
        );
        if !tags.is_empty() {
            let _ = write!(out, "  [{}]", tags.join(", "));
        }
        if let Some(rule) = &s.rule {
            let _ = write!(out, "  {}:{}", short_path(&rule.file), rule.line);
        }
        out.push('\n');
        if let Some(a) = r.flags.alert.as_ref().filter(|a| !a.is_empty()) {
            let _ = writeln!(out, "      alert: {a}");
        }
        if let Some(a) = &r.flags.announce {
            let _ = writeln!(out, "      announced: {a}");
        }
        for (i, seat) in SEATS.iter().enumerate() {
            // The caller always, the others when this call moved them.
            if *seat != s.caller && !r.narrowed[i].any() {
                continue;
            }
            let parts = knowledge_parts(&r.seats[i], &r.narrowed[i]);
            let text: Vec<String> = parts
                .iter()
                .map(|p| format!("{}{}", p.text, if p.changed { "*" } else { "" }))
                .collect();
            let known = if text.is_empty() {
                "nothing known".to_string()
            } else {
                text.join(" · ")
            };
            let own = r.own[i]
                .as_ref()
                .map(|o| format!("   [own: {}]", o.text()))
                .unwrap_or_default();
            let _ = writeln!(out, "      {}: {known}{own}", seat.to_char());
        }
        let st = &r.sides[side(s.caller)];
        let prev = rows
            .get(r.index.wrapping_sub(1))
            .map(|p| &p.sides[side(s.caller)]);
        if prev != Some(st) && *st != SideState::default() {
            let name = if side(s.caller) == 0 { "NS" } else { "EW" };
            let _ = writeln!(out, "      {name}: {}", side_text(st));
        }
    }
    out
}

/// A rule file's path from `conventions/` on.
fn short_path(p: &str) -> &str {
    p.find("conventions/")
        .map_or(p, |i| &p[i + "conventions/".len()..])
}
