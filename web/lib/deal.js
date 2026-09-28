// Deals, seats, boards and PBN: the plain data the page moves around.
// No DOM and no WASM here, so Node runs it too (web/scripts/test.mjs).

export const SEATS = ['N', 'E', 'S', 'W']
export const SEAT_NAMES = { N: 'North', E: 'East', S: 'South', W: 'West' }
export const SUITS = ['S', 'H', 'D', 'C']
export const SUIT_SYMBOLS = { S: '♠', H: '♥', D: '♦', C: '♣', N: 'NT' }
const RANKS = 'AKQJT98765432'

/** The seat `k` places clockwise after `seat`. */
export function seatAfter(seat, k = 1) {
  return SEATS[(SEATS.indexOf(seat) + k + 400) % 4]
}

export function partnership(seat) {
  return seat === 'N' || seat === 'S' ? 'ns' : 'ew'
}

/** Dealer and vulnerability of a board number, as duplicate boards have them. */
export function boardDealer(board) {
  return SEATS[(board - 1) % 4]
}
export function boardVul(board) {
  const cycle = ['None', 'NS', 'EW', 'All', 'NS', 'EW', 'All', 'None',
    'EW', 'All', 'None', 'NS', 'All', 'None', 'NS', 'EW']
  return cycle[(board - 1) % 16]
}

/** Normalise a vulnerability to None / NS / EW / All, or null. */
export function normVul(v) {
  const s = String(v ?? '').trim().toLowerCase()
  if (s === '' || s === 'none' || s === '-' || s === 'love') return 'None'
  if (s === 'ns' || s === 'n-s') return 'NS'
  if (s === 'ew' || s === 'e-w') return 'EW'
  if (s === 'all' || s === 'both' || s === 'b') return 'All'
  return null
}

export function normSeat(s) {
  const c = String(s ?? '').trim().charAt(0).toUpperCase()
  return SEATS.includes(c) ? c : null
}

/**
 * A PBN deal string (`N:<N> <E> <S> <W>`, any first seat) as {N, E, S, W},
 * or null. Hands are not checked here: the engine's `validate` does that
 * and says what is wrong.
 */
export function parseDeal(text) {
  const m = /^\s*([NESWnesw])\s*:\s*(.+?)\s*$/.exec(String(text ?? ''))
  if (!m) return null
  const hands = m[2].split(/\s+/)
  if (hands.length !== 4) return null
  const first = m[1].toUpperCase()
  const out = {}
  hands.forEach((h, i) => { out[seatAfter(first, i)] = h })
  return out
}

export function dealToPbn(deal, first = 'N') {
  return `${first}:` + [0, 1, 2, 3].map((i) => deal[seatAfter(first, i)]).join(' ')
}

/** The deal turned `k` seats clockwise: North's hand goes to East for k=1. */
export function rotate({ deal, dealer, vul }, k) {
  k = ((k % 4) + 4) % 4
  if (!k) return { deal: { ...deal }, dealer, vul }
  const out = {}
  for (const s of SEATS) out[seatAfter(s, k)] = deal[s]
  const swap = k % 2 === 1
  return {
    deal: out,
    dealer: seatAfter(dealer, k),
    vul: swap ? ({ NS: 'EW', EW: 'NS' }[vul] ?? vul) : vul,
  }
}

/** A uniformly random deal. `random` returns a float in [0, 1). */
export function randomDeal(random = Math.random) {
  const cards = []
  for (const s of SUITS) for (const r of RANKS) cards.push(s + r)
  for (let i = cards.length - 1; i > 0; i--) {
    const j = Math.floor(random() * (i + 1))
    ;[cards[i], cards[j]] = [cards[j], cards[i]]
  }
  const deal = {}
  SEATS.forEach((seat, n) => {
    const mine = cards.slice(n * 13, n * 13 + 13)
    deal[seat] = SUITS.map((s) =>
      mine.filter((c) => c[0] === s).map((c) => c[1])
        .sort((a, b) => RANKS.indexOf(a) - RANKS.indexOf(b)).join('')).join('.')
  })
  return deal
}

/** A small seeded generator (mulberry32), so a random deal can be linked. */
export function seeded(seed) {
  let a = seed >>> 0
  return () => {
    a = (a + 0x6d2b79f5) >>> 0
    let t = a
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

export function hcp(hand) {
  const pts = { A: 4, K: 3, Q: 2, J: 1 }
  return [...String(hand)].reduce((n, c) => n + (pts[c] ?? 0), 0)
}

/** Split an auction string into calls, spelling them as the engine does. */
export function parseCalls(text) {
  if (Array.isArray(text)) return text.map(normCall)
  return String(text ?? '').split(/[\s,]+/).filter(Boolean).map(normCall)
}

export function normCall(c) {
  const s = String(c).trim().toUpperCase()
  if (s === 'P' || s === 'PASS' || s === 'AP') return 'Pass'
  if (s === 'D' || s === 'X' || s === 'DBL') return 'X'
  if (s === 'R' || s === 'XX' || s === 'RDBL') return 'XX'
  const m = /^([1-7])(C|D|H|S|N|NT)$/.exec(s)
  if (m) return m[1] + (m[2] === 'N' ? 'NT' : m[2])
  return String(c).trim()
}

/** Is the auction over (three passes after a bid, or four passes)? */
export function auctionComplete(calls) {
  const n = calls.length
  if (n >= 4 && calls.slice(0, 4).every((c) => c === 'Pass') && n === 4) return true
  if (n < 4) return false
  const bid = calls.some((c) => /^[1-7]/.test(c))
  return bid && calls.slice(-3).every((c) => c === 'Pass')
}


/**
 * Boards of a PBN file: [{board, dealer, vul, deal, scoring, auction, notes,
 * tags}]. `auction` is the file's own auction from the dealer (BBA's, for the
 * PBS corpus), each call `{call, note}` with the text of its `=n=` note.
 * Enough PBN for deal files and bba-cli output, not a full reader: games are
 * separated by blank lines, as PBN requires.
 */
export function parsePbn(text) {
  const boards = []
  for (const game of String(text).split(/\r?\n[ \t]*\r?\n/)) {
    const tags = {}
    const auction = []
    const notes = {}
    let section = null
    for (const raw of game.split(/\r?\n/)) {
      const line = raw.trim()
      if (!line || line.startsWith('%') || line.startsWith(';')) continue
      const tag = /^\[(\w+)\s+"(.*)"\]$/.exec(line)
      if (tag) {
        section = tag[1]
        if (tag[1] === 'Note') {
          const n = /^(\d+):(.*)$/.exec(tag[2])
          if (n) notes[n[1]] = n[2]
        } else {
          tags[tag[1]] = tag[2]
        }
        continue
      }
      if (section !== 'Auction') continue
      for (const tok of line.split(/\s+/)) {
        const ref = /^=(\d+)=$/.exec(tok)
        if (ref) {
          if (auction.length) auction[auction.length - 1].noteRef = ref[1]
          continue
        }
        if (!tok || tok === '-' || tok === '*' || tok.startsWith('$')) continue
        if (tok.toUpperCase() === 'AP') {
          auction.push({ call: 'Pass' }, { call: 'Pass' }, { call: 'Pass' })
          continue
        }
        auction.push({ call: normCall(tok.replace(/[!?]+$/, '')) })
      }
    }
    if (!tags.Deal) continue
    for (const c of auction) {
      if (c.noteRef) c.note = notes[c.noteRef] ?? null
      delete c.noteRef
    }
    const n = parseInt(tags.Board, 10)
    boards.push({
      board: Number.isFinite(n) ? n : null,
      dealer: normSeat(tags.Dealer) ?? (Number.isFinite(n) ? boardDealer(n) : 'N'),
      vul: normVul(tags.Vulnerable) ?? (Number.isFinite(n) ? boardVul(n) : 'None'),
      deal: parseDeal(tags.Deal),
      scoring: /imp/i.test(tags.Scoring ?? '') ? 'IMP' : (tags.Scoring ? 'MP' : null),
      auction,
      tags,
    })
  }
  return boards
}
