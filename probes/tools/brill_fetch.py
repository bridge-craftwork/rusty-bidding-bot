#!/usr/bin/env python3
"""Fetch the Brill dailies that probes/tools/brill-drive-saver.gs saved to
Google Drive, into .rbb-cache/brill/<day>/, and optionally report on them.

    probes/tools/brill_fetch.py [--report] [--config PATH]

The Apps Script (see its header for the one-time install) saves each
"daily event results" mail's field PBNs and text to Drive and serves them
to this script through its web app, given the private token. Config:
~/.config/rusty-bidding-bot/brill-drive.json

    {"url": "https://script.google.com/macros/s/.../exec", "token": "..."}

Files already on disk are not fetched again. With --report, every day that
gained files runs probes/tools/brill_report.py on its PBNs (report.md and
report.json beside them) and appends to .rbb-cache/brill/history.jsonl.
Standard library only.
"""

import argparse
import json
import os
import subprocess
import sys
import urllib.parse
import urllib.request

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OUT = os.path.join(ROOT, ".rbb-cache", "brill")
CONFIG = os.path.expanduser("~/.config/rusty-bidding-bot/brill-drive.json")


def get(cfg, **params):
    q = urllib.parse.urlencode({"token": cfg["token"], **params})
    with urllib.request.urlopen(f"{cfg['url']}?{q}", timeout=120) as r:
        body = r.read().decode("utf-8")
    if body == "forbidden":
        sys.exit("brill_fetch: the web app refused the token (check the config)")
    return body


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--config", default=CONFIG)
    ap.add_argument("--report", action="store_true", help="run brill_report.py on each day that gained files")
    a = ap.parse_args()
    try:
        cfg = json.load(open(a.config))
    except OSError:
        sys.exit(f"brill_fetch: no config at {a.config} (see probes/tools/brill-drive-saver.gs, steps 4-6)")

    days = json.loads(get(cfg, list=1))["days"]
    new_days = []
    for day in sorted(days):
        folder = os.path.join(OUT, day)
        gained = False
        for f in days[day]:
            path = os.path.join(folder, f["name"])
            if os.path.exists(path):
                continue
            os.makedirs(folder, exist_ok=True)
            with open(path, "w", encoding="utf-8") as out:
                out.write(get(cfg, id=f["id"]))
            print(f"{day}/{f['name']}")
            gained = True
        if gained:
            new_days.append(day)
    if not new_days:
        print("brill_fetch: nothing new")
    if a.report:
        for day in new_days:
            folder = os.path.join(OUT, day)
            pbns = sorted(os.path.join(folder, n) for n in os.listdir(folder) if n.endswith(".pbn"))
            if not pbns:
                continue
            subprocess.run(
                [sys.executable, os.path.join(ROOT, "probes", "tools", "brill_report.py"), *pbns,
                 "--out", os.path.join(folder, "report.md"),
                 "--json", os.path.join(folder, "report.json"),
                 "--history", os.path.join(OUT, "history.jsonl")],
                cwd=ROOT, check=False)


if __name__ == "__main__":
    main()
