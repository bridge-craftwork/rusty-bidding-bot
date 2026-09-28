#!/usr/bin/env python3
"""Local workbench tickets: list them, and record a verdict.

    probes/tools/ticket.py list [--status open|resolved|wontfix|duplicate]
    probes/tools/ticket.py status DIR STATUS [--resolution TEXT] [--refs a,b]

Tickets live in `tickets/YYYY/MM/DD/NN.<status>.<slug>/` (the workbench
numbers each day's tickets from 01). The folder's status mirrors the
`status:` in ticket.md's frontmatter, so `status` updates both at once:
the frontmatter fields, then the folder name. context.json is evidence
and is never touched.
"""
import argparse
import pathlib
import re
import sys

STATUSES = ("open", "resolved", "wontfix", "duplicate")
ROOT = pathlib.Path(__file__).resolve().parents[2] / "tickets"


def frontmatter(md):
    """The frontmatter lines and the rest of ticket.md."""
    if not md.startswith("---\n"):
        raise SystemExit("ticket.md has no frontmatter")
    end = md.index("\n---\n", 4)
    return md[4:end].split("\n"), md[end:]


def set_field(lines, key, value):
    for i, line in enumerate(lines):
        if line.startswith(f"{key}:"):
            lines[i] = f"{key}: {value}".rstrip()
            return
    lines.append(f"{key}: {value}".rstrip())


def get_field(lines, key):
    for line in lines:
        if line.startswith(f"{key}:"):
            return line.split(":", 1)[1].strip()
    return ""


def quote(text):
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def tickets():
    return sorted(p.parent for p in ROOT.glob("*/*/*/*/ticket.md"))


def cmd_list(a):
    for d in tickets():
        lines, _ = frontmatter((d / "ticket.md").read_text())
        status = get_field(lines, "status")
        if a.status and status != a.status:
            continue
        print(f"{status:9} {d.relative_to(ROOT.parent)}")


def cmd_status(a):
    d = pathlib.Path(a.dir).resolve()
    md_path = d / "ticket.md"
    lines, rest = frontmatter(md_path.read_text())
    set_field(lines, "status", a.status)
    if a.resolution is not None:
        set_field(lines, "resolution", quote(a.resolution))
    if a.refs is not None:
        refs = [r.strip() for r in a.refs.split(",") if r.strip()]
        set_field(lines, "refs", "[" + ", ".join(refs) + "]")
    md_path.write_text("---\n" + "\n".join(lines) + rest)
    m = re.match(r"(\d+)\.[a-z]+\.(.*)", d.name)
    if not m:
        sys.exit(f"{d.name}: not an NN.<status>.<slug> folder; frontmatter updated only")
    new = d.with_name(f"{m.group(1)}.{a.status}.{m.group(2)}")
    if new != d:
        d.rename(new)
    print(new.relative_to(ROOT.parent))


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    ls = sub.add_parser("list")
    ls.add_argument("--status", choices=STATUSES)
    st = sub.add_parser("status")
    st.add_argument("dir")
    st.add_argument("status", choices=STATUSES)
    st.add_argument("--resolution")
    st.add_argument("--refs", help="commits, comma-separated")
    a = ap.parse_args()
    {"list": cmd_list, "status": cmd_status}[a.cmd](a)


if __name__ == "__main__":
    main()
