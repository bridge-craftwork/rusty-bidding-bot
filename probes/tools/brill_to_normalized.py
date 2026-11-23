#!/usr/bin/env python3
"""A Brill daily's field PBN as the normalized results JSON that Bridge
Classroom's Game Analysis reads (bridge-classroom-fetch
docs/normalized-schema.md, schema 1.4).

    probes/tools/brill_to_normalized.py FIELD.pbn [--us Rusty] [-o out.json]

One envelope per file (one event, one session). Every table is a pair
number: North-South is the Brill robot North and the entrant South, so the
entrants are compared as North-South pairs, as in a one-winner movement.
East-West at table n is pair 100+n, so each table's opponents are a pair of
their own rather than one pair meeting everyone.
The double-dummy table and par come from `rbb dd` (cached). Auctions and the
card play of every table are carried, so the play is in play order (the PBN
lists each trick by seat).
"""

import argparse
import json
import os
import sys
from collections import Counter, defaultdict
from datetime import datetime, timezone

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import brill_report as br  # noqa: E402

SEATS = "NESW"
RANKS = "23456789TJQKA"


def hand_json(hand):
    """'AK2.Q.T9.8' (S.H.D.C) -> {"S": ["A","K","2"], ...} with '10'."""
    out = {}
    for suit, cards in zip("SHDC", hand.split(".")):
        out[suit] = ["10" if c == "T" else c for c in sorted(cards, key=RANKS.index, reverse=True)]
    return out


def vul_name(v):
    u = (v or "").upper().replace("-", "")
    return {"ALL": "Both", "BOTH": "Both", "NS": "NS", "EW": "EW"}.get(u, "None")


def contract_str(c):
    if not c:
        return "PASS"
    level, strain, dbl, _ = c
    return f"{level}{'NT' if strain == 'N' else strain}{['', 'X', 'XX'][dbl]}"


def card_str(card):
    return card[0] + ("10" if card[1] == "T" else card[1])


def play_order(lines, leader, strain):
    """PBN [Play] lines (one trick a line, columns in seat order from the
    first leader) as cards in the order played."""
    cols = [SEATS[(SEATS.index(leader) + i) % 4] for i in range(4)]
    order, lead = [], leader
    for line in lines:
        toks = line.split()
        if len(toks) < 4:
            break
        by_seat = {cols[i]: toks[i].upper() for i in range(4)}
        seq = [SEATS[(SEATS.index(lead) + i) % 4] for i in range(4)]
        trick = [(s, by_seat[s]) for s in seq if by_seat[s] not in ("-", "*", "--")]
        if not trick:
            break
        order.extend(card_str(c) for _, c in trick)
        if len(trick) < 4:
            break
        led = trick[0][1][0]

        def key(sc):
            suit, rank = sc[1][0], sc[1][1]
            return (2 if suit == strain else 1 if suit == led else 0, RANKS.index(rank))
        lead = max(trick, key=key)[0]
    return order or None


def par_list(row):
    out = []
    for part in (row.get("par") or "").split(";"):
        toks = part.split()
        if len(toks) < 2:
            continue
        side, body = toks[0], toks[1]
        level, strain = int(body[0]), body[1]
        rest = body[2:]
        dbl = "XX" if rest.startswith("XX") else "X" if rest.startswith("X") else ""
        seats = side if side in ("NS", "EW") else "NS"
        decl = max(seats, key=lambda s: br.dd_of(row, s, strain))
        out.append({"score": row["par_ns"],
                    "contract": f"{level}{'NT' if strain == 'N' else strain}{dbl}",
                    "declarer": decl})
    return out


def convert(path, us, rbb, cache):
    with open(path, encoding="utf-8", errors="replace") as f:
        games = br.parse_pbn(f.read())
    tables, bad = br.tables_of(games, us)
    dd = br.dd_tables(tables, rbb, cache)
    scoring = br.event_scoring(tables, "MP")
    entrants = []
    for t in tables:
        if t["south"] not in entrants:
            entrants.append(t["south"])
    number = {name: i + 1 for i, name in enumerate(entrants)}
    by_board = defaultdict(list)
    for t in tables:
        by_board[t["board"]].append(t)
    boards, warnings = [], list(bad)
    user_pair = None
    for bno in sorted(by_board, key=lambda b: int(b) if b.isdigit() else 0):
        bt = by_board[bno]
        deal = Counter(t["deal"] for t in bt).most_common(1)[0][0]
        bt = [t for t in bt if t["deal"] == deal]
        row = dd.get(deal)
        t0 = bt[0]
        results, mine = [], None
        for t in bt:
            c = t["contract"]
            tricks = t["tricks"]
            score = br.ns_score(c, tricks, t["vul"]) if c and tricks is not None else (0 if not c else None)
            n = number[t["south"]]
            strain = c[1] if c else None
            leader = SEATS[(SEATS.index(c[3]) + 1) % 4] if c else None
            play = play_order(t["play"], leader, None if strain == "N" else strain) if c and t["play"] else None
            res = {
                "contract": contract_str(c),
                "declarer": c[3] if c else None,
                "tricks": tricks if c else None,
                "score": score,
                "matchpoints": None,
                "percentage": None,
                "imps": None,
                "ns_pair": {"number": n, "section": "A", "strat": None, "strat_ranks": [],
                            "players": [{"name": t["north"] or "Robot (Brill)", "acbl_id": None, "external_ids": {}},
                                        {"name": t["south"], "acbl_id": None, "external_ids": {}}]},
                "ew_pair": {"number": 100 + n, "section": "A", "strat": None, "strat_ranks": [],
                            "players": [{"name": t["west"] or "Robot (Brill)", "acbl_id": None, "external_ids": {}},
                                        {"name": t["east"] or "Robot (Brill)", "acbl_id": None, "external_ids": {}}]},
                "auction": [("PASS" if x == "Pass" else x) for x, _ in t["calls"]] or None,
                "play": play,
            }
            if t["ours"]:
                mine = len(results)
                user_pair = res["ns_pair"]
            results.append(res)
        # Matchpoints for the North-South field (2 a beat, 1 a tie).
        scored = [r for r in results if r["score"] is not None]
        top = 2 * (len(scored) - 1)
        for r in scored:
            if scoring == "MP" and top > 0:
                mp = sum(2 if r["score"] > o["score"] else 1 if r["score"] == o["score"] else 0
                         for o in scored if o is not r)
                r["matchpoints"] = mp / 2
                r["percentage"] = round(100 * mp / top, 2)
            elif scoring == "IMP":
                r["imps"] = br.cross_imps(r["score"], [o["score"] for o in scored if o is not r])
        board = {
            "number": int(bno) if bno.isdigit() else bno,
            "section": "A",
            "dealer": t0["dealer"],
            "vulnerability": vul_name(t0["vul"]),
            "deal": {s: hand_json(t0["hands"][s]) for s in SEATS},
            "double_dummy": ({s: dict(zip(["C", "D", "H", "S", "NT"], row["tricks"][s])) for s in SEATS}
                             if row else None),
            "par": par_list(row) if row else [],
            "results": results,
            "user_result_index": mine,
        }
        boards.append(board)
    date = (t0["date"] or "").replace(".", "-")
    session = {"session_number": 1, "time": None, "table_count": len(entrants), "boards": boards,
               "pairs": {str(number[n]): [{"name": "Robot (Brill)", "acbl_id": None, "external_ids": {}},
                                          {"name": n, "acbl_id": None, "external_ids": {}}]
                         for n in entrants},
               "partial": False, "warnings": warnings}
    if user_pair:
        session["user_pair"] = {"section": "A", "direction": "NS", "pair_number": user_pair["number"],
                                "players": user_pair["players"]}
    name = t0["event"]
    env = {
        "schema_version": "1.4",
        "source": "brill",
        "provider": {"id": "rusty-bidding-bot/brill_to_normalized", "version": "1.0.0", "kind": "manual"},
        "coverage": {"cardplay": "all-tables", "auction": "all-tables", "results": "all-tables",
                     "sections": "not-applicable", "player_names": "usernames", "sections_labelled": False},
        "fetched_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "tournaments": [{"sanction": None, "schedule_url": None, "name": "Brill Challenge",
                         "events": [{"event_id": name, "event_type": "open_pairs", "name": name,
                                     "date": date,
                                     "scoring": "matchpoints" if scoring == "MP" else "imps",
                                     "sessions": [session]}]}],
    }
    if user_pair:
        env["capture"] = {"context": f"{user_pair['players'][1]['name']} (South, with Brill)",
                          "players": [p["name"] for p in user_pair["players"]],
                          "pair": f"A-NS{user_pair['number']}"}
    return env


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("pbn")
    ap.add_argument("--us", default="Rusty")
    ap.add_argument("-o", "--out")
    ap.add_argument("--rbb", default=br.RBB)
    ap.add_argument("--dd-cache", default=os.path.join(br.ROOT, ".rbb-cache", "dd.jsonl"))
    a = ap.parse_args()
    env = convert(a.pbn, a.us, a.rbb, a.dd_cache)
    text = json.dumps(env, ensure_ascii=False)
    if a.out:
        with open(a.out, "w") as f:
            f.write(text)
        print(f"wrote {a.out}", file=sys.stderr)
    else:
        print(text)


if __name__ == "__main__":
    main()
