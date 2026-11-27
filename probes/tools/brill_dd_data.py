#!/usr/bin/env python3
"""Rusty's Brill daily scores, actual and double dummy, for the dashboard.

    probes/tools/brill_dd_data.py [--brill DIR] [--out web/brill/brill-dd.json]
                                  [--deployments probes/brill-deployments.toml] [-v]

Deployments recorded by rusty-seat-service on the droplet are merged into
the deployments file first by probes/tools/brill_deployments.py.

Reads every .rbb-cache/brill/<day>/mail.txt (the daily results mails that
probes/tools/brill_fetch.py saves) and writes web/brill/brill-dd.json, the
data of web/brill.html. Run it after brill_fetch.py; it is idempotent (the
output depends only on the mails, the history and the deployments file).

Per day and event (Daily IMP, Daily IMP - strongest hand, Daily MP, Daily
MP - strongest hand) the mail gives the result

    Daily IMP: 3rd of 28 with +10.9 IMP

and, under the event's own heading,

    played double dummy it would have been -2.6 IMP (cardplay +13.5); ...

that is, Rusty's contracts all played double dummy (the bidding's score)
and the cardplay's contribution. IMP scores are totals over the session
(8 boards), MP scores percentages. Mails without the double-dummy lines
(2026-10-26, 2026-11-21) take them from brill_report.py's history
(.rbb-cache/brill/history.jsonl: `bidding` and `play`, scored by us from
the field PBN the same way; `dd_source` says which). Every mail figure is
also checked against that history and the differences are printed.

Lines that look like a result or a double-dummy line but do not parse are
listed in the output's `unparsed` and on stderr. Standard library only.
"""

import argparse
import glob
import json
import os
import re
import subprocess
import sys
import tomllib

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

EVENTS = [
    ("imp", "Daily IMP", "IMP"),
    ("imp-strongest", "Daily IMP - strongest hand", "IMP"),
    ("mp", "Daily MP", "MP"),
    ("mp-strongest", "Daily MP - strongest hand", "MP"),
]
BY_NAME = {name: (eid, scoring) for eid, name, scoring in EVENTS}

NUM = r"[+-]?\d+(?:\.\d+)?"
RESULT = re.compile(
    rf"^(?P<name>Daily [^:]+): (?P<rank>\d+)(?:st|nd|rd|th) of (?P<field>\d+) with (?P<score>{NUM}) ?(?P<unit>IMP|%)$")
DD = re.compile(
    rf"^played double dummy it would have been (?P<dd>{NUM}) ?(?P<unit>IMP|%) \(cardplay (?P<cp>{NUM})\)"
    rf"(?:; declaring (?P<decl>level|{NUM} tricks?) over (?P<decl_n>\d+) board\(s\), "
    rf"defending (?P<def>level|{NUM} tricks?) over (?P<def_n>\d+))?$")
HEADING = re.compile(r"^(Daily [^:]+):$")


def default_brill_dir():
    """The main checkout's cache: a worktree has none of its own."""
    here = os.path.join(ROOT, ".rbb-cache", "brill")
    if os.path.isdir(here):
        return here
    try:
        common = subprocess.run(["git", "-C", ROOT, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                                capture_output=True, text=True, check=True).stdout.strip()
        main = os.path.join(os.path.dirname(common), ".rbb-cache", "brill")
        if os.path.isdir(main):
            return main
    except (OSError, subprocess.CalledProcessError):
        pass
    return here


def tricks(text):
    if text is None:
        return None
    if text == "level":
        return 0
    return int(text.split()[0])


def num(text):
    v = float(text)
    return int(v) if v == int(v) and "." not in text else v


def parse_mail(day, text, unparsed):
    """{event id: row} for one mail."""
    rows = {}
    current = None
    for raw in text.splitlines():
        line = raw.strip()
        if not line:
            continue
        m = RESULT.match(line)
        if m:
            name = m["name"]
            if name not in BY_NAME:
                unparsed.append({"date": day, "line": line, "reason": "unknown event"})
                continue
            eid, scoring = BY_NAME[name]
            if (scoring == "IMP") != (m["unit"] == "IMP"):
                unparsed.append({"date": day, "line": line, "reason": "unit does not match the event"})
                continue
            rows.setdefault(eid, {}).update(
                date=day, event=eid, scoring=scoring, actual=num(m["score"]),
                rank=int(m["rank"]), field=int(m["field"]))
            continue
        if line.startswith("Daily ") and (" of " in line and " with " in line):
            unparsed.append({"date": day, "line": line, "reason": "result line"})
            continue
        m = HEADING.match(line)
        if m:
            current = BY_NAME.get(m[1], (None,))[0]
            if current is None:
                unparsed.append({"date": day, "line": line, "reason": "unknown event heading"})
            continue
        if line.startswith("played double dummy"):
            m = DD.match(line)
            if not m or current is None:
                unparsed.append({"date": day, "line": line,
                                 "reason": "double-dummy line" if not m else "no event heading above it"})
                continue
            row = rows.setdefault(current, {})
            row.update(dd=num(m["dd"]), cardplay=num(m["cp"]), dd_source="mail")
            if m["decl"] is not None:
                row["declaring"] = {"tricks": tricks(m["decl"]), "boards": int(m["decl_n"])}
                row["defending"] = {"tricks": tricks(m["def"]), "boards": int(m["def_n"])}
            continue
        if line.startswith("https://") and current is None:
            # The tournament link under each result line.
            last = [r for r in rows.values() if "actual" in r and "url" not in r]
            if last:
                last[-1]["url"] = line
    return rows


def load_history(path):
    """{(day, event id): last record} from brill_report.py's history."""
    out = {}
    if not os.path.exists(path):
        return out
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        day = r["date"].replace(".", "-")
        name = re.sub(r" \d{4}-\d\d-\d\d$", "", r["event"])
        if name in BY_NAME:
            out[(day, BY_NAME[name][0])] = r
    return out


def boards_in_pbn(dirpath, eid, day):
    slug = {"imp": "daily-imp", "imp-strongest": "daily-imp-strongest-hand",
            "mp": "daily-mp", "mp-strongest": "daily-mp-strongest-hand"}[eid]
    p = os.path.join(dirpath, f"{slug}-{day}-field.pbn")
    if not os.path.exists(p):
        return None
    n = len(re.findall(r'^\[South "Rusty"\]', open(p, encoding="utf-8", errors="replace").read(), re.M))
    return n or None


VERSION_KEYS = ("rbb", "seat", "card", "ben")


def carry_versions(deployments):
    """Give every deployment `in_force`: the rbb, seat, card and BEN versions
    in force from its first day on (its own fields, the earlier entries' for
    the rest), so the dashboard names both Rusty's and BEN's version for
    every period. Entries of one day share the state at the end of that
    day: a BEN change and a bidding change on the same day both count."""
    now = {}
    for day in sorted({d["date"] for d in deployments}):
        same = [d for d in deployments if d["date"] == day]
        for d in same:
            now.update({k: d[k] for k in VERSION_KEYS if d.get(k)})
        for d in same:
            d["in_force"] = dict(now)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--brill", default=None, help="the days' folder (default: .rbb-cache/brill of the main checkout)")
    ap.add_argument("--out", default=os.path.join(ROOT, "web", "brill", "brill-dd.json"))
    ap.add_argument("--deployments", default=os.path.join(ROOT, "probes", "brill-deployments.toml"))
    ap.add_argument("-v", "--verbose", action="store_true", help="list every figure that differs from the history")
    args = ap.parse_args()

    brill = args.brill or default_brill_dir()
    history = load_history(os.path.join(brill, "history.jsonl"))
    unparsed, rows, diffs = [], [], []
    days = sorted(d for d in os.listdir(brill) if re.fullmatch(r"\d{4}-\d\d-\d\d", d)) if os.path.isdir(brill) else []
    if not days:
        sys.exit(f"brill_dd_data: no days under {brill}")
    for day in days:
        mail = os.path.join(brill, day, "mail.txt")
        if not os.path.exists(mail):
            unparsed.append({"date": day, "line": None, "reason": "no mail.txt"})
            continue
        got = parse_mail(day, open(mail, encoding="utf-8").read(), unparsed)
        for eid, _, scoring in EVENTS:
            row = got.get(eid)
            if not row or "actual" not in row:
                unparsed.append({"date": day, "line": None, "reason": f"no result for {eid}"})
                continue
            h = history.get((day, eid))
            if "dd" not in row and h and h.get("bidding") is not None:
                row.update(dd=round(h["bidding"], 1), cardplay=round(h["play"], 1), dd_source="field")
            elif "dd" not in row:
                row.update(dd=None, cardplay=None, dd_source=None)
                unparsed.append({"date": day, "line": None, "reason": f"no double-dummy score for {eid}"})
            if h:
                row["boards"] = h.get("boards")
                for key, hk in (("actual", "score"), ("dd", "bidding"), ("cardplay", "play")):
                    if row.get(key) is not None and h.get(hk) is not None and row.get("dd_source") == "mail":
                        d = abs(row[key] - h[hk])
                        if d > 0.15:
                            diffs.append((day, eid, key, row[key], round(h[hk], 2)))
            if not row.get("boards"):
                row["boards"] = boards_in_pbn(os.path.join(brill, day), eid, day)
            order = ["date", "event", "scoring", "actual", "dd", "cardplay", "rank", "field", "boards",
                     "dd_source", "declaring", "defending", "url"]
            rows.append({k: row[k] for k in order if k in row})

    deployments = []
    if os.path.exists(args.deployments):
        with open(args.deployments, "rb") as f:
            deployments = sorted(tomllib.load(f).get("deployment", []),
                                 key=lambda d: (d["date"], d.get("started_at", ""), d["kind"]))
    carry_versions(deployments)

    out = {
        "about": "Rusty in the Brill daily events: each session's actual score and the score of its contracts "
                 "played double dummy (the bidding's), from the daily results mails. IMP: total over the session; "
                 "MP: percent. Generated by probes/tools/brill_dd_data.py.",
        "through": days[-1],
        "events": [{"id": e, "name": n, "scoring": s} for e, n, s in EVENTS],
        "deployments": deployments,
        "rows": rows,
        "unparsed": unparsed,
    }
    os.makedirs(os.path.dirname(args.out), exist_ok=True)
    text = json.dumps(out, indent=1, ensure_ascii=False) + "\n"
    old = open(args.out, encoding="utf-8").read() if os.path.exists(args.out) else None
    if text != old:
        with open(args.out, "w", encoding="utf-8") as f:
            f.write(text)
    print(f"brill_dd_data: {len(rows)} sessions over {len(days)} days ({days[0]} to {days[-1]}), "
          f"{len(deployments)} deployments -> {os.path.relpath(args.out)}"
          f"{'' if text != old else ' (unchanged)'}", file=sys.stderr)
    for u in unparsed:
        print(f"  unparsed {u['date']}: {u['reason']}" + (f": {u['line']}" if u["line"] else ""), file=sys.stderr)
    fallback = [r for r in rows if r.get("dd_source") == "field"]
    if fallback:
        print(f"  double dummy from the field PBN (no mail line): "
              f"{', '.join(sorted({r['date'] for r in fallback}))}", file=sys.stderr)
    if diffs:
        # brill_report.py scores the field PBN itself; small differences
        # (a field that grew after the mail, rounding) are expected.
        worst = max(diffs, key=lambda d: abs(d[3] - d[4]))
        print(f"  {len(diffs)} mail figures differ from history.jsonl by more than 0.15 "
              f"(largest {worst[0]} {worst[1]} {worst[2]}: mail {worst[3]}, history {worst[4]}; -v lists them)",
              file=sys.stderr)
        if args.verbose:
            for d in diffs:
                print(f"    {d[0]} {d[1]} {d[2]}: mail {d[3]}, history {d[4]}", file=sys.stderr)


if __name__ == "__main__":
    main()
