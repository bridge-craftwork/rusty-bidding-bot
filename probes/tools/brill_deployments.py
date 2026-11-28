#!/usr/bin/env python3
"""Merge rusty-seat-service's recorded deployments into the Brill dashboard's
deployments file.

    probes/tools/brill_deployments.py [--from-json FILE] [--host bridge-droplet]
                                      [--deployments probes/brill-deployments.toml]
                                      [--dry-run]

rusty-seat-service adds a row to its `deployments` table whenever a
start-up's versions differ from the last row's (its commit, image tag,
rusty-bidding-bot, convention-card, rule pack, card, BEN), and serves them
at `/deployments?key=<dashboard secret>`. This asks for them over ssh
(`ssh bridge-droplet curl -s localhost:8006/deployments?key=...`, the key
read from the droplet's .env on the droplet, so it never leaves it), or
reads them from FILE (`{"deployments": [...]}`, the same JSON), and appends
an entry to probes/brill-deployments.toml for each change not there yet:

- kind "bidding" when the service's commit, rusty-bidding-bot, the
  convention-card crate, the rule pack or the card changed; kind "play"
  when BEN did; both (two entries, same day) when both did;
- `date`: the first Brill daily played on it. A daily's date is Danish and
  its tables open at 12:00 Danish time, so a start before noon in
  Copenhagen counts from that day, a later one from the next;
- `confidence = "recorded"`, `started_at` (UTC), `recorded_id` (the row's
  id: a rerun adds nothing), and `rbb`/`seat`/`card` or `ben`, the
  versions it put in force.

The first recorded row is compared with the versions the hand-kept
entries put in force (their last `rbb`, `card` and `ben`), so the first
deploy of the recording service adds an entry only if it changes those;
every later row is compared with the row before it. Once recording has
begun, add no hand-kept entries after it. Only appends, keeping the file's comments.
Until the seat service with `/deployments` is deployed the droplet answers
404 and this says so and changes nothing. Then run brill_dd_data.py.
Standard library only.
"""

import argparse
import datetime
import json
import os
import subprocess
import sys
import tomllib
from zoneinfo import ZoneInfo

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
DENMARK = ZoneInfo("Europe/Copenhagen")
# The remote side: the dashboard key from the droplet's .env, used there.
REMOTE = ("key=$(sed -n 's/^RUSTY_SEAT_SERVICE_DASHBOARD_SECRET=//p' /opt/bridge-craftwork/.env); "
          "curl -s -w '\\n%{http_code}' \"http://localhost:8006/deployments?key=${key}\"")

# What makes a row a bidding change, and a play change.
BIDDING = ("git_sha", "rbb_commit", "convention_card_commit", "pack_id", "card")
PLAY = ("ben_image", "ben_version")


def fetch(host):
    """The droplet's rows, or None (with the reason on stderr) when it has
    no history to give."""
    try:
        out = subprocess.run(["ssh", "-o", "ConnectTimeout=15", host, REMOTE],
                             capture_output=True, text=True, timeout=60)
    except (OSError, subprocess.TimeoutExpired) as e:
        print(f"brill_deployments: ssh {host}: {e}", file=sys.stderr)
        return None
    if out.returncode != 0:
        print(f"brill_deployments: ssh {host} failed: {out.stderr.strip()}", file=sys.stderr)
        return None
    body, _, code = out.stdout.rstrip("\n").rpartition("\n")
    if code != "200":
        why = "the deployed rusty-seat-service predates /deployments" if code == "404" else f"HTTP {code}"
        print(f"brill_deployments: no deployment history on {host} ({why}); nothing changed", file=sys.stderr)
        return None
    try:
        return json.loads(body)["deployments"]
    except (ValueError, KeyError) as e:
        print(f"brill_deployments: unreadable answer from {host}: {e}", file=sys.stderr)
        return None


def first_daily(started_at):
    """The first Brill daily (Danish date) a deployment at `started_at`
    (UTC, RFC 3339) played all of."""
    t = datetime.datetime.fromisoformat(started_at.replace("Z", "+00:00")).astimezone(DENMARK)
    day = t.date() if t.hour < 12 else t.date() + datetime.timedelta(days=1)
    return day.isoformat()


def seat_name(row):
    d = row.get("git_describe") or "unknown"
    if d == "unknown":
        return (row.get("git_sha") or "unknown")[:7]
    return d


def in_force(entries):
    """The rbb, card and BEN versions the file's entries leave in force."""
    now = {}
    for e in sorted(entries, key=lambda e: (e["date"], e.get("started_at", ""), e["kind"])):
        now.update({k: e[k] for k in ("rbb", "seat", "card", "ben") if e.get(k)})
    return now


def changes(rows, existing):
    """The entries to add for `rows` (oldest first), given the file's
    entries."""
    done = {e.get("recorded_id") for e in existing if e.get("recorded_id") is not None}
    # The first row continues the hand-kept history; later rows, the row
    # before them (the recorded history is complete from its first row).
    # Only the hand-kept entries dated before the first row: a later one
    # (written by hand for a change the rows also record) is not yet in force.
    hand = [e for e in existing if e.get("confidence") != "recorded"]
    first = first_daily(rows[0]["started_at"]) if rows else ""
    before = in_force([e for e in hand if e["date"] < first])
    # A change already written by hand (same kind, day and version) is not
    # added again.
    stated = {(e["date"], e["kind"], e.get("ben") if e["kind"] == "play" else e.get("rbb")) for e in hand}
    out, prev = [], None
    for row in rows:
        if prev is None:
            # Against the hand-kept history: only what it records.
            bidding = (before.get("rbb") != row["rbb_tag"]
                       or (before.get("card") and before["card"] != row["card"]))
            play = before.get("ben") != row["ben_version"]
            changed = [k for k, a in (("rbb_tag", before.get("rbb")), ("card", before.get("card")),
                                      ("ben_version", before.get("ben"))) if a != row[k]]
            old = {"rbb_tag": before.get("rbb"), "card": before.get("card"), "ben_version": before.get("ben")}
        else:
            changed = [k for k in BIDDING + PLAY + ("rbb_tag", "convention_card_tag", "git_describe")
                       if prev.get(k) != row.get(k)]
            bidding = any(k in changed for k in BIDDING)
            play = any(k in changed for k in PLAY)
            old = prev
        prev = row
        if row["id"] in done or not (bidding or play):
            continue
        common = {
            "date": first_daily(row["started_at"]),
            "confidence": "recorded",
            "started_at": row["started_at"],
            "recorded_id": row["id"],
        }
        diff = "; ".join(f"{k} {old.get(k) or '?'} -> {row.get(k)}" for k in changed
                         if k not in ("git_sha", "rbb_commit", "convention_card_commit") + PLAY)
        evidence = (f"rusty-seat-service deployments row {row['id']}, started {row['started_at']}: "
                    f"seat {seat_name(row)} (image :{row.get('image_tag')}), rbb {row['rbb_tag']} "
                    f"({row['rbb_commit'][:7]}), convention-card {row['convention_card_tag']}, "
                    f"pack {row['pack_id']}, card {row['card']}, BEN {row['ben_version']}")
        if bidding and (common["date"], "bidding", row["rbb_tag"]) in stated:
            bidding = False
        if play and (common["date"], "play", row["ben_version"]) in stated:
            play = False
        if bidding:
            out.append({**common, "kind": "bidding",
                        "label": f"seat {seat_name(row)} / rbb {row['rbb_tag']}",
                        "what": f"Recorded: {diff or 'a new seat-service build'}",
                        "evidence": evidence,
                        "rbb": row["rbb_tag"], "seat": seat_name(row), "card": row["card"]})
        if play:
            out.append({**common, "kind": "play", "label": f"BEN {row['ben_version']}",
                        "what": f"Recorded: BEN {old.get('ben_version') or '?'} -> {row['ben_version']}",
                        "evidence": evidence, "ben": row["ben_version"]})
    return out


def toml_entry(e):
    order = ["date", "kind", "label", "what", "evidence", "confidence", "started_at", "recorded_id",
             "rbb", "seat", "card", "ben"]
    lines = ["", "[[deployment]]"]  # a blank line before each, as the file has
    for k in order:
        if k in e:
            v = e[k]
            lines.append(f"{k} = {v}" if isinstance(v, int) else f"{k} = {json.dumps(v, ensure_ascii=False)}")
    return "\n".join(lines) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--from-json", help="read the rows from this file instead of the droplet")
    ap.add_argument("--host", default="bridge-droplet")
    ap.add_argument("--deployments", default=os.path.join(ROOT, "probes", "brill-deployments.toml"))
    ap.add_argument("--dry-run", action="store_true", help="print the entries instead of appending them")
    args = ap.parse_args()

    if args.from_json:
        with open(args.from_json, encoding="utf-8") as f:
            rows = json.load(f)["deployments"]
    else:
        rows = fetch(args.host)
        if rows is None:
            return
    rows = sorted(rows, key=lambda r: r["id"])
    with open(args.deployments, "rb") as f:
        existing = tomllib.load(f).get("deployment", [])
    new = changes(rows, existing)
    text = "".join(toml_entry(e) for e in new)
    if args.dry_run or not new:
        sys.stdout.write(text)
        print(f"brill_deployments: {len(rows)} recorded rows, {len(new)} new entries"
              f"{' (dry run)' if args.dry_run and new else ''}", file=sys.stderr)
        return
    with open(args.deployments, "a", encoding="utf-8") as f:
        f.write(text)
    tomllib.loads(open(args.deployments, encoding="utf-8").read())  # still valid
    print(f"brill_deployments: {len(rows)} recorded rows, {len(new)} new entries -> "
          f"{os.path.relpath(args.deployments)}; now run probes/tools/brill_dd_data.py", file=sys.stderr)


if __name__ == "__main__":
    main()
