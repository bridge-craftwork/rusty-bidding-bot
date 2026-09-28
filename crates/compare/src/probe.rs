//! Probes: make a small reference set on purpose, to isolate one decision.
//!
//! Deals come from fixed holdings (optionally varied, e.g. the same hand
//! with 0-4 tens), from a dealer3 script, or at random. bba-cli bids them
//! with the auction forced up to a prefix and chosen cards (a named card, a
//! file, or a bare system, with toggles edited), and our engine replays the
//! result. Everything is written to an output directory so a probe can be
//! inspected or re-run.

use std::path::{Path, PathBuf};
use std::process::Command;

use bridge_card::{bbsa, Card, Value, Vocabulary};
use bridge_types::{
    Board, Call, Card as PlayingCard, Deal, Direction, Hand, Rank, ScoringMethod, Suit,
    Vulnerability,
};
use rbb_engine::Engine;
use serde::Serialize;

use crate::board::{self, BoardResult};

/// Where bba-cli is normally installed.
pub const DEFAULT_BBA_CLI: &str = "/Applications/Bridge Utilities/bba-cli";

/// Where the deals come from.
#[derive(Debug, Clone)]
pub enum Deals {
    /// These holdings for these seats; the other seats are dealt at random.
    /// The first holding is the one `variation` changes.
    Fixed {
        hands: Vec<(Direction, Hand)>,
        variation: Variation,
        layouts: usize,
    },
    /// A dealer3 script, producing `count` deals.
    Script {
        script: PathBuf,
        count: usize,
        dealer_bin: PathBuf,
    },
    /// `count` random deals.
    Random { count: usize },
    /// Complete deals, each with a label (built by `grid`).
    Explicit(Vec<(String, Deal)>),
}

/// How the first fixed holding is varied, to find what BBA's decision
/// turns on without depending on hands in the corpus.
#[derive(Debug, Clone)]
pub enum Variation {
    /// The holding as given.
    None,
    /// The same holding with 0-4 tens (same HCP and shape).
    Tens,
    /// Every single-card exchange with the two opponents' hands (partner's
    /// hand never changes): 13 x 26 variants of one fixed deal.
    Survey,
    /// From the holding to this one a card at a time, and each of those
    /// exchanges alone.
    MorphTo(Hand),
    /// These holdings in place of the first, each with a label; the other
    /// fixed hands stay, the rest of the cards go to the open seats.
    List(Vec<(String, Hand)>),
}

#[derive(Debug, Clone)]
pub struct ProbeOptions {
    pub deals: Deals,
    pub dealer: Direction,
    pub vul: Vulnerability,
    pub scoring: ScoringMethod,
    /// Calls forced before the decision under study.
    pub prefix: Vec<Call>,
    /// A card name in `pbs/bbsa`, a path to a `.bbsa`, or `bare:<system>`
    /// (2/1, sayc, polish, precision, acol).
    pub ns_card: String,
    pub ew_card: String,
    /// `.bbsa` key edits, e.g. ("Texas", 0), for each side.
    pub ns_set: Vec<(String, i64)>,
    pub ew_set: Vec<(String, i64)>,
    pub pbs: PathBuf,
    pub rules: PathBuf,
    pub bba_cli: PathBuf,
    pub out_dir: PathBuf,
    pub seed: u64,
    /// Card changes (`path=value`) for our engine's side of the comparison,
    /// e.g. `general.style=bba`; bba-cli never sees them.
    pub our_changes: Vec<String>,
    /// Bid every deal at each of these vulnerabilities (empty: `vul` only).
    pub vuls: Vec<Vulnerability>,
}

/// One board of a probe, from the decision after the prefix on.
#[derive(Debug, Clone, Serialize)]
pub struct ProbeRow {
    /// What this variant changed ("base", "♥T for ♦Q", "step 2: ..."),
    /// for surveys and morphs.
    pub label: Option<String>,
    /// The seat whose decision is probed (first to call after the prefix).
    pub seat: Direction,
    pub hand: String,
    pub hcp: i32,
    pub tens: i32,
    /// Notrump points in quarters, as our engine counts them.
    pub points_q: i32,
    /// Suit points in quarters.
    pub suit_points_q: i32,
    pub reference: Option<Call>,
    pub ours: Option<Call>,
    pub board: BoardResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeReport {
    pub rows: Vec<ProbeRow>,
    pub out_dir: PathBuf,
}

impl ProbeReport {
    /// Boards where both sides made the probed decision, and agreed.
    pub fn agreement(&self) -> (usize, usize) {
        let decided: Vec<&ProbeRow> = self.rows.iter().filter(|r| r.reference.is_some()).collect();
        (
            decided.iter().filter(|r| r.reference == r.ours).count(),
            decided.len(),
        )
    }
}

const SYSTEMS: [(&str, &str); 5] = [
    ("2/1", "two_over_one"),
    ("sayc", "sayc"),
    ("polish", "polish_club"),
    ("precision", "precision"),
    ("acol", "acol"),
];

/// `.bbsa` text for a card spec, with edits applied. A `bare:` card is
/// written through `vocab` (the rules' own vocabulary).
pub fn card_text(
    spec: &str,
    pbs: &Path,
    edits: &[(String, i64)],
    vocab: &Vocabulary,
) -> Result<String, String> {
    let text = if let Some(system) = spec.strip_prefix("bare:") {
        let category = SYSTEMS
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(system))
            .map(|(_, v)| *v)
            .ok_or_else(|| {
                let names: Vec<&str> = SYSTEMS.iter().map(|(k, _)| *k).collect();
                format!("bare:{system}: expected one of {}", names.join(", "))
            })?;
        // Every toggle off; only the system type set.
        let mut card = Card::new(vocab);
        card.set("general.system_category", Value::Text(category.into()))
            .map_err(|e| e.to_string())?;
        bbsa::export(&card).0
    } else {
        let path = if spec.ends_with(".bbsa") {
            PathBuf::from(spec)
        } else {
            pbs.join("bbsa").join(format!("{spec}.bbsa"))
        };
        std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?
    };
    let mut lines: Vec<String> = text
        .lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect();
    for (key, value) in edits {
        let line = lines
            .iter_mut()
            .find(|l| l.rsplit_once('=').is_some_and(|(k, _)| k.trim() == key))
            .ok_or_else(|| {
                format!("no .bbsa key {key:?} (keys are as written in the file, e.g. \"Texas\")")
            })?;
        *line = format!("{key} = {value}");
    }
    Ok(lines.join("\r\n") + "\r\n")
}

/// Small deterministic generator for filling in the other hands.
pub(crate) struct Rng(pub(crate) u64);
impl Rng {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// The same holding with 0, 1, ... more tens: each step replaces the lowest
/// spot card (2-9) of the next suit, spades first, by that suit's ten. HCP
/// and shape stay the same.
pub fn ten_variants(hand: &Hand) -> Vec<Hand> {
    let mut out = vec![hand.clone()];
    let mut cur: Vec<PlayingCard> = hand.cards().to_vec();
    for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
        if cur.iter().any(|c| c.suit == suit && c.rank == Rank::Ten) {
            continue;
        }
        let lowest = cur
            .iter()
            .enumerate()
            .filter(|(_, c)| c.suit == suit && (c.rank as u8) < 10)
            .min_by_key(|(_, c)| c.rank as u8)
            .map(|(i, _)| i);
        if let Some(i) = lowest {
            cur[i] = PlayingCard::new(suit, Rank::Ten);
            out.push(Hand::from_cards(cur.clone()));
        }
    }
    out
}

pub(crate) fn fill(fixed: &[(Direction, Hand)], rng: &mut Rng) -> Result<Deal, String> {
    let mut used = [false; 52];
    let mut deal = Deal::new();
    for (seat, hand) in fixed {
        if hand.len() != 13 {
            return Err(format!("{seat:?}: a hand needs 13 cards"));
        }
        for c in hand.cards() {
            let i = c.to_index() as usize;
            if used[i] {
                return Err(format!("{} is in two hands", card_name(c)));
            }
            used[i] = true;
        }
        deal.set_hand(*seat, hand.clone());
    }
    let mut rest: Vec<PlayingCard> = (0..52u8)
        .filter(|i| !used[*i as usize])
        .filter_map(PlayingCard::from_index)
        .collect();
    for i in (1..rest.len()).rev() {
        let j = (rng.next() % (i as u64 + 1)) as usize;
        rest.swap(i, j);
    }
    let mut it = rest.into_iter();
    for seat in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        if !fixed.iter().any(|(s, _)| *s == seat) {
            deal.set_hand(seat, Hand::from_cards(it.by_ref().take(13).collect()));
        }
    }
    Ok(deal)
}

pub(crate) fn card_name(c: &PlayingCard) -> String {
    format!("{}{}", c.suit.symbol(), c.rank.to_char())
}

/// Give `seat` the card `add` in place of `drop`: the seat holding `add`
/// takes `drop`.
fn exchange(deal: &Deal, seat: Direction, drop: PlayingCard, add: PlayingCard) -> Deal {
    let mut d = deal.clone();
    let holder = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ]
    .into_iter()
    .find(|s| deal.hand(*s).cards().contains(&add))
    .expect("every card is in some hand");
    let swap = |h: &Hand, out: PlayingCard, inn: PlayingCard| {
        Hand::from_cards(
            h.cards()
                .iter()
                .map(|c| if *c == out { inn } else { *c })
                .collect(),
        )
    };
    d.set_hand(seat, swap(deal.hand(seat), drop, add));
    d.set_hand(holder, swap(deal.hand(holder), add, drop));
    d
}

/// Every single-card exchange between `seat` and its two opponents.
pub(crate) fn survey(base: &Deal, seat: Direction) -> Vec<(String, Deal)> {
    let mut out = vec![("base".to_string(), base.clone())];
    let opponents = [seat.next(), seat.next().next().next()];
    let mut mine: Vec<PlayingCard> = base.hand(seat).cards().to_vec();
    mine.sort_by_key(|c| std::cmp::Reverse(c.to_index()));
    for drop in &mine {
        let mut theirs: Vec<PlayingCard> = opponents
            .iter()
            .flat_map(|s| base.hand(*s).cards().to_vec())
            .collect();
        theirs.sort_by_key(|c| std::cmp::Reverse(c.to_index()));
        for add in theirs {
            out.push((
                format!("{} for {}", card_name(&add), card_name(drop)),
                exchange(base, seat, *drop, add),
            ));
        }
    }
    out
}

/// From `seat`'s hand to `target` a card at a time (same-suit exchanges
/// first), then each exchange alone. The `fixed` seats (partner, when his
/// hand is given) may not give up cards.
pub(crate) fn morph(
    base: &Deal,
    seat: Direction,
    target: &Hand,
    fixed: &[Direction],
) -> Result<Vec<(String, Deal)>, String> {
    let from = base.hand(seat).cards().to_vec();
    let to = target.cards().to_vec();
    let mut outs: Vec<PlayingCard> = from.iter().filter(|c| !to.contains(c)).copied().collect();
    let mut ins: Vec<PlayingCard> = to.iter().filter(|c| !from.contains(c)).copied().collect();
    for c in &ins {
        let holder = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
        .into_iter()
        .find(|s| base.hand(*s).cards().contains(c))
        .expect("every card is in some hand");
        if holder != seat && fixed.contains(&holder) {
            return Err(format!(
                "{} is in {holder:?}'s fixed hand: it cannot move to {seat:?}",
                card_name(c)
            ));
        }
    }
    // Pair the exchanges: the same suit first, highest cards first.
    outs.sort_by_key(|c| std::cmp::Reverse(c.to_index()));
    ins.sort_by_key(|c| std::cmp::Reverse(c.to_index()));
    let mut pairs = Vec::new();
    let mut rest_in = Vec::new();
    for i in ins {
        if let Some(k) = outs.iter().position(|o| o.suit == i.suit) {
            pairs.push((outs.remove(k), i));
        } else {
            rest_in.push(i);
        }
    }
    pairs.extend(outs.into_iter().zip(rest_in));
    let mut out = vec![("from".to_string(), base.clone())];
    let mut cur = base.clone();
    for (n, (o, i)) in pairs.iter().enumerate() {
        cur = exchange(&cur, seat, *o, *i);
        out.push((
            format!("step {}: {} for {}", n + 1, card_name(i), card_name(o)),
            cur.clone(),
        ));
    }
    if pairs.len() > 1 {
        for (o, i) in &pairs {
            out.push((
                format!("only {} for {}", card_name(i), card_name(o)),
                exchange(base, seat, *o, *i),
            ));
        }
    }
    Ok(out)
}

fn make_deals(opts: &ProbeOptions) -> Result<Vec<(Option<String>, Deal)>, String> {
    let mut rng = Rng(opts.seed | 1);
    match &opts.deals {
        Deals::Fixed {
            hands,
            variation,
            layouts,
        } => {
            // Vary the first holding; keep any others as given.
            let (first, others) = hands.split_first().ok_or("no --hand given")?;
            // Opponents trade cards with the varied hand; a fixed partner
            // never does.
            let partner = first.0.partner();
            let fixed_seats: Vec<Direction> = hands
                .iter()
                .map(|(s, _)| *s)
                .filter(|s| *s == partner)
                .collect();
            match variation {
                Variation::Survey | Variation::MorphTo(_) => {
                    // One deal, varied card by card.
                    let base = fill(hands, &mut rng)?;
                    let v = match variation {
                        Variation::Survey => survey(&base, first.0),
                        Variation::MorphTo(t) => morph(&base, first.0, t, &fixed_seats)?,
                        _ => unreachable!(),
                    };
                    Ok(v.into_iter().map(|(l, d)| (Some(l), d)).collect())
                }
                Variation::List(list) => {
                    let mut out = Vec::new();
                    for (label, hand) in list {
                        let mut fixed = vec![(first.0, hand.clone())];
                        fixed.extend(others.iter().cloned());
                        let deal = fill(&fixed, &mut rng)
                            .map_err(|e| format!("variant {label:?} ({}): {e}", hand.to_pbn()))?;
                        out.push((Some(label.clone()), deal));
                    }
                    Ok(out)
                }
                Variation::None | Variation::Tens => {
                    let variants = if matches!(variation, Variation::Tens) {
                        ten_variants(&first.1)
                    } else {
                        vec![first.1.clone()]
                    };
                    let mut out = Vec::new();
                    for v in variants {
                        for _ in 0..(*layouts).max(1) {
                            let mut fixed = vec![(first.0, v.clone())];
                            fixed.extend(others.iter().cloned());
                            out.push((None, fill(&fixed, &mut rng)?));
                        }
                    }
                    Ok(out)
                }
            }
        }
        Deals::Explicit(list) => Ok(list
            .iter()
            .map(|(l, d)| (Some(l.clone()), d.clone()))
            .collect()),
        Deals::Random { count } => (0..*count)
            .map(|_| fill(&[], &mut rng).map(|d| (None, d)))
            .collect(),
        Deals::Script {
            script,
            count,
            dealer_bin,
        } => {
            let output = Command::new(dealer_bin)
                .arg(script)
                .args([
                    "-p",
                    &count.to_string(),
                    "-s",
                    &opts.seed.to_string(),
                    "-f",
                    "printpbn",
                ])
                .output()
                .map_err(|e| format!("{}: {e}", dealer_bin.display()))?;
            if !output.status.success() {
                return Err(format!(
                    "dealer3 failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            let text = String::from_utf8_lossy(&output.stdout);
            let boards = bridge_encodings::pbn::read_pbn(&text).map_err(|e| e.to_string())?;
            Ok(boards.into_iter().map(|b| (None, b.deal)).collect())
        }
    }
}

fn scoring_arg(s: ScoringMethod) -> &'static str {
    match s {
        ScoringMethod::Matchpoints | ScoringMethod::BAM => "MP",
        _ => "IMP",
    }
}

/// Make the reference with bba-cli and compare our engine with it.
pub fn run(opts: &ProbeOptions) -> Result<ProbeReport, String> {
    std::fs::create_dir_all(&opts.out_dir)
        .map_err(|e| format!("{}: {e}", opts.out_dir.display()))?;
    let rules = rbb_engine::load_rules(&opts.rules).map_err(|d| {
        d.iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let ns_text = card_text(&opts.ns_card, &opts.pbs, &opts.ns_set, &rules.vocab)?;
    let ew_text = card_text(&opts.ew_card, &opts.pbs, &opts.ew_set, &rules.vocab)?;
    let (ns_path, ew_path) = (opts.out_dir.join("ns.bbsa"), opts.out_dir.join("ew.bbsa"));
    let write =
        |p: &Path, t: &str| std::fs::write(p, t).map_err(|e| format!("{}: {e}", p.display()));
    write(&ns_path, &ns_text)?;
    write(&ew_path, &ew_text)?;

    let deals = make_deals(opts)?;
    let vuls = if opts.vuls.is_empty() {
        vec![opts.vul]
    } else {
        opts.vuls.clone()
    };
    // Every deal at every vulnerability, the vulnerabilities innermost.
    let mut labels: Vec<Option<String>> = Vec::new();
    let mut boards: Vec<Board> = Vec::new();
    for (label, d) in deals {
        for v in &vuls {
            labels.push(label.clone());
            boards.push(
                Board::new()
                    .with_number(boards.len() as u32 + 1)
                    .with_dealer(opts.dealer)
                    .with_vulnerability(*v)
                    .with_deal(d.clone()),
            );
        }
    }
    let input = opts.out_dir.join("deals.pbn");
    let output = opts.out_dir.join("bba.pbn");
    write(&input, &bridge_encodings::pbn::write_pbn(&boards))?;

    let mut cmd = Command::new(&opts.bba_cli);
    cmd.arg("-i").arg(&input).arg("-o").arg(&output);
    cmd.arg("--ns-conventions")
        .arg(&ns_path)
        .arg("--ew-conventions")
        .arg(&ew_path);
    cmd.args(["--event", "probe", "--scoring", scoring_arg(opts.scoring)]);
    // Every call's meaning in [Note]s, not only the alerts: grid cells
    // carry the meaning of BBA's call.
    cmd.arg("--all-meanings");
    if !opts.prefix.is_empty() {
        let prefix: Vec<String> = opts.prefix.iter().map(Call::to_pbn).collect();
        cmd.arg("--auction-prefix").arg(prefix.join(" "));
    }
    let out = cmd
        .output()
        .map_err(|e| format!("{}: {e}", opts.bba_cli.display()))?;
    if !out.status.success() {
        return Err(format!(
            "bba-cli failed: {}{}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        ));
    }

    let (mut ns_card, _) =
        bbsa::import(&rules.vocab, &ns_text, Some("ns")).map_err(|e| e.to_string())?;
    let (mut ew_card, _) =
        bbsa::import(&rules.vocab, &ew_text, Some("ew")).map_err(|e| e.to_string())?;
    for change in &opts.our_changes {
        let (path, value) = change
            .split_once('=')
            .ok_or_else(|| format!("{change:?}: expected path=value"))?;
        let value = match value {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            v => v
                .parse::<i64>()
                .map(Value::Int)
                .unwrap_or_else(|_| Value::Text(v.to_string())),
        };
        for card in [&mut ns_card, &mut ew_card] {
            card.set(path, value.clone()).map_err(|e| e.to_string())?;
        }
    }
    let engine = Engine::new(&ns_card, &ew_card, &rules);
    let valuation = rbb_engine::Valuation::for_card(&ns_card);

    let bba_boards = bridge_encodings::pbn::read_pbn_file(&output).map_err(|e| e.to_string())?;
    let k = opts.prefix.len();
    let mut rows = Vec::new();
    for mut b in bba_boards {
        b.extra_tags.retain(|(n, _)| n != "Scoring");
        b.extra_tags
            .push(("Scoring".into(), scoring_arg(opts.scoring).into()));
        let Some(mut r) = board::compare(&engine, "probe", &b) else {
            continue;
        };
        r.ns_card = opts.ns_card.clone();
        r.ew_card = opts.ew_card.clone();
        let seat = (0..k).fold(r.dealer, |d, _| d.next());
        let hand = b.deal.hand(seat);
        let facts = rbb_engine::Facts::new(hand);
        let label = b
            .number
            .and_then(|n| labels.get(n as usize - 1).cloned().flatten());
        rows.push(ProbeRow {
            label,
            seat,
            hand: hand.to_pbn(),
            hcp: facts.hcp,
            tens: facts.tens,
            points_q: facts.points_q(valuation),
            suit_points_q: facts.suit_points_q(valuation),
            reference: r.reference.get(k).cloned(),
            ours: r.replay.get(k).cloned(),
            board: r,
        });
    }
    let report = ProbeReport {
        rows,
        out_dir: opts.out_dir.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        let _ = std::fs::write(opts.out_dir.join("report.json"), json);
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_variants_keep_hcp_and_shape() {
        let h = Hand::from_pbn("AK52.KQ73.A95.J8").unwrap();
        let v = ten_variants(&h);
        assert_eq!(v.len(), 5); // 0..=4 tens
        for (i, x) in v.iter().enumerate() {
            assert_eq!(x.hcp(), h.hcp());
            assert_eq!(x.distribution(), h.distribution());
            assert_eq!(rbb_engine::Facts::new(x).tens, i as i32);
        }
        assert_eq!(v[4].to_pbn(), "AKT5.KQT7.AT9.JT");
    }

    fn board2() -> Deal {
        Deal::from_pbn("N:A5.K43.T764.QT65 T8763.Q95.AQ52.2 K92.AT.KJ983.AJ8 QJ4.J8762..K9743")
            .unwrap()
    }

    #[test]
    fn survey_changes_one_card_and_never_partners_hand() {
        let base = board2();
        let v = survey(&base, Direction::North);
        assert_eq!(v.len(), 1 + 13 * 26);
        for (label, d) in &v[1..] {
            let before = base.hand(Direction::North).cards();
            let after = d.hand(Direction::North).cards();
            let changed = after.iter().filter(|c| !before.contains(c)).count();
            assert_eq!(changed, 1, "{label}");
            assert_eq!(
                d.hand(Direction::South),
                base.hand(Direction::South),
                "{label}"
            );
        }
    }

    #[test]
    fn morph_walks_to_the_target_a_card_at_a_time() {
        let base = board2();
        let target = Hand::from_pbn("A5.K43.7654.Q965").unwrap();
        let v = morph(&base, Direction::North, &target, &[Direction::South]).unwrap();
        // from, 2 steps, and the 2 exchanges alone
        assert_eq!(
            v.len(),
            5,
            "{:?}",
            v.iter().map(|(l, _)| l).collect::<Vec<_>>()
        );
        let last = &v[2].1;
        let mut a = last.hand(Direction::North).cards().to_vec();
        let mut b = target.cards().to_vec();
        a.sort_by_key(|c| c.to_index());
        b.sort_by_key(|c| c.to_index());
        assert_eq!(a, b);
        // A card in partner's fixed hand cannot move.
        let t = Hand::from_pbn("A5.K43.T764.QJT5").unwrap();
        assert!(morph(&base, Direction::North, &t, &[Direction::South]).is_err());
    }

    #[test]
    fn a_variant_list_keeps_partner_and_rejects_clashes() {
        let south = (
            Direction::South,
            Hand::from_pbn("K92.A7.KJ532.A43").unwrap(),
        );
        let opts = |list: Vec<(String, Hand)>| ProbeOptions {
            deals: Deals::Fixed {
                hands: vec![
                    (
                        Direction::North,
                        Hand::from_pbn("A5.K86.Q764.9865").unwrap(),
                    ),
                    south.clone(),
                ],
                variation: Variation::List(list),
                layouts: 1,
            },
            dealer: Direction::South,
            vul: Vulnerability::None,
            scoring: ScoringMethod::Matchpoints,
            prefix: vec![],
            ns_card: String::new(),
            ew_card: String::new(),
            ns_set: vec![],
            ew_set: vec![],
            pbs: PathBuf::new(),
            rules: PathBuf::new(),
            bba_cli: PathBuf::new(),
            out_dir: PathBuf::new(),
            seed: 1,
            our_changes: vec![],
            vuls: vec![],
        };
        let v = Hand::from_pbn("A5.K86.9764.QT85").unwrap();
        let deals = make_deals(&opts(vec![("C QT".into(), v.clone())])).unwrap();
        assert_eq!(deals.len(), 1);
        assert_eq!(deals[0].0.as_deref(), Some("C QT"));
        assert_eq!(deals[0].1.hand(Direction::North), &v);
        assert_eq!(deals[0].1.hand(Direction::South), &south.1);
        // The diamond king is South's: the variant is refused, by name.
        let bad = Hand::from_pbn("A5.Q86.KT64.9875").unwrap();
        let err = make_deals(&opts(vec![("D KT".into(), bad)])).unwrap_err();
        assert!(err.contains("D KT") && err.contains("♦K"), "{err}");
    }

    #[test]
    fn bare_card_turns_everything_off() {
        let rules = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conventions");
        let vocab = Vocabulary::load(&rules).unwrap();
        let text = card_text("bare:2/1", Path::new("."), &[("Texas".into(), 1)], &vocab).unwrap();
        let entries = bbsa::parse(&text).unwrap();
        assert!(entries.iter().any(|(k, v)| k == "Texas" && *v == 1));
        assert!(entries.iter().any(|(k, v)| k == "System type" && *v == 0));
        let on: Vec<_> = entries
            .iter()
            .filter(|(k, v)| *v != 0 && k != "Texas")
            .collect();
        assert!(on.is_empty(), "{on:?}");
    }
}
