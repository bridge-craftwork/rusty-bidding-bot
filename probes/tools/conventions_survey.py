#!/usr/bin/env python3
"""Data behind docs/conventions-survey.md (GitHub issue #4).

Reads, never writes, the sibling repos:
  Practice-Bidding-Scenarios  bbsa/*.bbsa, btn/*.btn, bba/*.pbn, the release layout
  Bridge-Classroom            src/utils/conventionCatalog.js
and this repo's crates/bridge-card/data/{fields,bbsa-map}.toml and conventions/*.bid.

  conventions_survey.py [--pbs DIR] [--classroom DIR] [--json OUT] [--compare compare.json]

Prints markdown tables on stdout (the survey doc quotes them) and, with
--json, everything it gathered. `--compare` takes an `rbb compare --json`
run and adds each scenario's agreement with BBA.
"""
import argparse
import collections
import glob
import json
import os
import re
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
# Sibling checkouts: the directory that holds this repo.
GITHUB = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))


# ── sources ────────────────────────────────────────────────────────────────

def read_bbsa(path):
    keys = {}
    for line in open(path, encoding="utf-8", errors="replace"):
        line = line.strip()
        if " = " not in line:
            continue
        k, v = line.rsplit(" = ", 1)
        if k == "Not defined":
            continue
        try:
            keys[k] = int(v)
        except ValueError:
            keys[k] = v
    return keys


def read_btn(path):
    head, chat, in_chat = {}, [], False
    for line in open(path, encoding="utf-8", errors="replace"):
        s = line.rstrip("\n")
        m = re.match(r"^# ([a-z-]+): ?(.*)$", s)
        if m and not in_chat and m.group(1) not in head:
            head[m.group(1)] = m.group(2).strip()
        if s.startswith("/*@chat"):
            in_chat = True
            continue
        if s.startswith("@chat*/"):
            in_chat = False
            continue
        if in_chat:
            chat.append(s)
    head["chat"] = "\n".join(chat).strip()
    return head


def read_pbn(path):
    txt = open(path, encoding="utf-8", errors="replace").read()
    # The first `% CCn - ` line names the card file. BBA's own exports go on
    # to list the card's settings with the same prefix; those are not names.
    cc = {}
    for side, val in re.findall(r"^% (CC[12]) - (.*)$", txt, re.M):
        if side not in cc and val.strip().lower().endswith(".bbsa"):
            cc[side] = re.split(r"[\\/]", val.strip())[-1][:-5]
    notes = collections.Counter()
    for m in re.finditer(r'^\[Note "\d+:(.*)"\]', txt, re.M):
        notes[norm_note(m.group(1))] += 1
    return {
        "boards": len(re.findall(r"^\[Board ", txt, re.M)),
        "cc1": cc.get("CC1", ""),
        "cc2": cc.get("CC2", ""),
        "notes": notes,
    }


def norm_note(n):
    """BBA's alert text with the suit- and count-specific parts removed."""
    n = re.sub(r"![SHDC]", "x", n)
    n = n.split(",")[0].strip()
    n = re.sub(r"\d+[-+]?\d*\s*(HCP|points|cards?)", "N \\1", n)
    return n


def layout_sections(pbs):
    path = os.path.join(pbs, "btn", "-button-layout-release.txt")
    sec, major, out = None, None, {}
    for line in open(path, encoding="utf-8"):
        s = line.strip()
        if s.startswith("[Major]"):
            major = s[7:].strip()
        elif s.startswith("[Section]"):
            sec = s[9:].strip()
        elif s and not s.startswith(("#", "[", "---")):
            for tok in re.split(r"[,()]", s):
                name = tok.strip().split(":")[0].strip()
                if name and name != "---" and name not in out:
                    out[name] = (major, sec)
    return out


def map_groups():
    """.bbsa key -> the `# ── Group ──` heading it sits under in bbsa-map.toml."""
    groups, cur = {}, "General"
    for line in open(os.path.join(REPO, "crates/bridge-card/data/bbsa-map.toml"), encoding="utf-8"):
        m = re.match(r"^# ── (.+?) ─", line)
        if m:
            cur = m.group(1)
        m = re.match(r'^"([^"]+)"\s*=', line)
        if m and not line.startswith('"notrump.') and cur:
            groups.setdefault(m.group(1), cur)
        if line.startswith("[implied]"):
            cur = None
    return groups


def read_map():
    data = tomllib.load(open(os.path.join(REPO, "crates/bridge-card/data/bbsa-map.toml"), "rb"))
    mapping = {}
    for k, v in data.items():
        if k in ("implied", "derived"):
            continue
        if isinstance(v, str):
            mapping[k] = [v]
        elif "set" in v:
            mapping[k] = list(v["set"].keys())
        elif "index" in v:
            mapping[k] = [v["index"]]
    derived = []
    for d in data.get("derived", []):
        srcs = list(d.get("when", {}).keys())
        dsts = list(d.get("set", {}).keys()) + list(d.get("default", {}).keys())
        derived.append((srcs, dsts))
    return mapping, derived, data.get("implied", {})


def read_fields():
    data = tomllib.load(open(os.path.join(REPO, "crates/bridge-card/data/fields.toml"), "rb"))
    fields, aliases = {}, {}
    for sec, entries in data.items():
        for k, v in entries.items():
            path = f"{sec}.{k}"
            fields[path] = v
            for a in v.get("aliases", []):
                aliases[a if "." in a and a.split(".")[0] in data else f"{sec}.{a}"] = path
    return fields, aliases


def read_modules():
    """module name -> {'file', 'card': [paths], 'param': [paths], 'rules': n}"""
    mods = {}
    for f in sorted(glob.glob(os.path.join(REPO, "conventions", "*", "*.bid"))):
        name, cards, params, rules = None, [], [], 0
        for line in open(f, encoding="utf-8"):
            m = re.match(r"^module\s+(\S+)", line)
            if m:
                name = m.group(1)
            m = re.match(r"^\s+card\s+([a-z_0-9.]+)", line)
            if m:
                cards.append(m.group(1))
            m = re.match(r"^\s+param\s+\w+\s*=\s*([a-z_0-9.]+)", line)
            if m:
                params.append(m.group(1))
            if re.match(r"^\s+(P|X|XX|[1-7](C|D|H|S|N|NT|x|m|M|y|z|j|o))\b", line):
                rules += 1
        mods[name or os.path.basename(f)] = {
            "file": os.path.relpath(f, REPO), "card": cards, "param": params, "rules": rules,
            "lines": sum(1 for _ in open(f, encoding="utf-8")),
        }
    return mods


def read_catalog(classroom, aliases):
    js = open(os.path.join(classroom, "src/utils/conventionCatalog.js"), encoding="utf-8").read()
    body = js[js.index("export const CONVENTION_CATALOG"):]
    out = []
    for block in re.findall(r"\{\s*id:(.*?)\n  \}", body, re.S):
        def get(k):
            m = re.search(k + r":\s*'([^']*)'", "id:" + block)
            return m.group(1) if m else None
        path = get("cardPath")
        out.append({
            "id": get("id"), "name": get("name"), "section": get("section"),
            "cardPath": path, "field": aliases.get(path, path), "skillPath": get("skillPath"),
        })
    return out


# ── analysis ───────────────────────────────────────────────────────────────

IMP_STEPS = [20, 50, 90, 130, 170, 220, 270, 320, 370, 430, 500, 600, 750, 900,
             1100, 1300, 1500, 1750, 2000, 2250, 2500, 3000, 3500, 4000]


def imps(diff):
    d = abs(diff)
    return sum(1 for s in IMP_STEPS if d >= s)


def divergences_by_note(boards):
    """For each BBA alert text (normalised): the boards whose BBA auction
    carries it, and the boards where our auction first departs from BBA's at
    a call BBA alerted with it, with the IMPs (par as the yardstick, as
    `rbb compare` counts them) on those boards."""
    out = collections.defaultdict(lambda: {"boards": 0, "first_div": 0, "imps": 0})
    for b in boards:
        alerts = b.get("reference_alerts") or []
        for n in {norm_note(a) for a in alerts if a}:
            out[n]["boards"] += 1
        i = b.get("first_divergence")
        if i is None or i >= len(alerts) or not alerts[i]:
            continue
        n = norm_note(alerts[i])
        out[n]["first_div"] += 1
        p = b.get("par") or {}
        if p.get("par_ns") is not None and p.get("reference_ns") is not None and p.get("ours_ns") is not None:
            out[n]["imps"] += imps(p["reference_ns"] - p["par_ns"]) - imps(p["ours_ns"] - p["par_ns"])
    return dict(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pbs", default=os.path.join(GITHUB, "Practice-Bidding-Scenarios"))
    ap.add_argument("--classroom", default=os.path.join(GITHUB, "Bridge-Classroom"))
    ap.add_argument("--json")
    ap.add_argument("--compare", help="an `rbb compare --json` output")
    a = ap.parse_args()

    cards = {os.path.basename(p)[:-5]: read_bbsa(p) for p in sorted(glob.glob(os.path.join(a.pbs, "bbsa", "*.bbsa")))}
    mapping, derived, implied = read_map()
    fields, aliases = read_fields()
    mods = read_modules()
    catalog = read_catalog(a.classroom, aliases)
    sections = layout_sections(a.pbs)

    read_by = collections.defaultdict(set)  # field -> modules reading it
    for name, m in mods.items():
        for p in m["card"] + m["param"]:
            read_by[aliases.get(p, p)].add(name)
    # A field that only feeds a derived field counts as read through it.
    via = collections.defaultdict(set)
    for srcs, dsts in derived:
        for s in srcs:
            for d in dsts:
                if read_by.get(d):
                    via[s] |= read_by[d]

    scen = {}
    for p in sorted(glob.glob(os.path.join(a.pbs, "btn", "*.btn"))):
        name = os.path.basename(p)[:-4]
        if name.startswith("-"):
            continue
        h = read_btn(p)
        pbn = os.path.join(a.pbs, "bba", name + ".pbn")
        info = read_pbn(pbn) if os.path.exists(pbn) else {"boards": 0, "cc1": "", "cc2": "", "notes": collections.Counter()}
        scen[name] = {
            "bba_works": h.get("bba-works", "").lower() == "true",
            "gib_works": h.get("gib-works", "").lower() == "true",
            "ns": info["cc1"] or h.get("convention-card-ns", ""),
            "ew": info["cc2"] or h.get("convention-card-ew", ""),
            "btn_ns": h.get("convention-card-ns", ""), "btn_ew": h.get("convention-card-ew", ""),
            "boards": info["boards"], "notes": info["notes"],
            "filter": h.get("auction-filter", ""), "chat": h["chat"],
            "section": (sections.get(name) or (None, None))[1],
            "major": (sections.get(name) or (None, None))[0],
            "button": h.get("button-text", ""),
        }

    div_by_note = {}
    if a.compare:
        cmp = json.load(open(a.compare))
        div_by_note = divergences_by_note(cmp.get("boards", []))
        for row in cmp.get("summary", cmp).get("scenarios", []):
            if row.get("name") in scen:
                ns, ew = row["calls"]
                scen[row["name"]]["compare"] = {
                    "ns_calls": ns["agree"] / ns["total"] if ns["total"] else None,
                    "calls": (ns["agree"] + ew["agree"]) / max(1, ns["total"] + ew["total"]),
                    "contracts": row["contracts_match"] / max(1, row["boards"]),
                    "imps": row["par"]["imps_vs_reference"],
                    "no_rule": row["boards_with_no_rule"],
                }

    # cards used by scenarios, NS side weighted by boards
    card_use = collections.Counter()
    card_boards = collections.Counter()
    for s in scen.values():
        for side in ("ns", "ew"):
            card_use[(s[side], side)] += 1
            card_boards[(s[side], side)] += s["boards"]

    all_keys = []
    for c in cards.values():
        for k in c:
            if k not in all_keys:
                all_keys.append(k)

    def key_status(k):
        if k not in mapping:
            return "passthrough", [], set()
        fs = mapping[k]
        rd = set()
        for f in fs:
            rd |= read_by.get(f, set()) | via.get(f, set())
        return ("read" if rd else "mapped"), fs, rd

    key_rows = []
    for k in all_keys:
        on = [c for c, kv in cards.items() if kv.get(k) not in (0, None)]
        ns_scen = [n for n, s in scen.items() if cards.get(s["ns"], {}).get(k) not in (0, None)]
        ew_scen = [n for n, s in scen.items() if cards.get(s["ew"], {}).get(k) not in (0, None)]
        st, fs, rd = key_status(k)
        key_rows.append({
            "key": k, "cards_on": on, "n_cards_on": len(on),
            "ns_scen": len(ns_scen), "ew_scen": len(ew_scen),
            "ns_boards": sum(scen[n]["boards"] for n in ns_scen),
            "status": st, "fields": fs, "modules": sorted(rd),
            "values": sorted({str(kv.get(k)) for kv in cards.values()}),
        })

    notes_total = collections.Counter()
    notes_scen = collections.defaultdict(set)
    for n, s in scen.items():
        for note, c in s["notes"].items():
            notes_total[note] += c
            notes_scen[note].add(n)

    out = {
        "cards": list(cards), "keys": key_rows, "modules": mods,
        "catalog": [dict(c, modules=sorted(read_by.get(c["field"] or "", set()) | via.get(c["field"] or "", set())),
                         in_fields=(c["field"] in fields)) for c in catalog],
        "scenarios": {n: dict(s, notes=dict(s["notes"].most_common())) for n, s in scen.items()},
        "card_use": {f"{c}|{side}": [card_use[(c, side)], card_boards[(c, side)]] for (c, side) in card_use},
        "notes": [(n, c, len(notes_scen[n])) for n, c in notes_total.most_common()],
        "fields": len(fields), "fields_read": sorted(f for f in fields if read_by.get(f) or via.get(f)),
        "mapped_keys": len(mapping),
    }
    if a.json:
        json.dump(out, open(a.json, "w"), indent=1, default=list)

    # ── markdown ──
    w = sys.stdout.write
    w(f"cards: {len(cards)}; keys: {len(all_keys)}; mapped: {sum(1 for r in key_rows if r['status'] != 'passthrough')}; "
      f"read by rules: {sum(1 for r in key_rows if r['status'] == 'read')}\n")
    w(f"fields: {len(fields)}; read by rules: {len(out['fields_read'])}; modules: {len(mods)}; "
      f"module lines: {sum(m['lines'] for m in mods.values())}\n")
    w(f"scenarios: {len(scen)}; bba-works false: {sum(1 for s in scen.values() if not s['bba_works'])}; "
      f"boards: {sum(s['boards'] for s in scen.values())}\n\n")

    w("## card use (scenarios, boards) by side\n")
    for (c, side), n in sorted(card_use.items(), key=lambda x: (-x[1], x[0])):
        w(f"| {c or '(none)'} | {side} | {n} | {card_boards[(c, side)]} |\n")

    groups = map_groups()
    label = {"read": "rules", "mapped": "field only", "passthrough": "**passthrough**"}
    w("\n## keys\n")
    order = list(dict.fromkeys(list(groups.values()) + ["Not mapped"]))
    for g in order:
        rows = [r for r in key_rows if groups.get(r["key"], "Not mapped") == g]
        if not rows:
            continue
        w(f"\n#### {g}\n\n| .bbsa key | cards on (of {len(cards)}) | scenarios, NS / EW | engine | card field | modules reading it |\n"
          "|---|---:|---:|---|---|---|\n")
        for r in rows:
            w(f"| {r['key']} | {r['n_cards_on']} | {r['ns_scen']} / {r['ew_scen']} | {label[r['status']]} | "
              f"{', '.join('`' + f + '`' for f in r['fields'])} | {', '.join(r['modules'])} |\n")

    w("\n## keys that differ between cards\n")
    for r in key_rows:
        if 0 < r["n_cards_on"] < len(cards):
            w(f"- {r['key']}: on in {', '.join(r['cards_on'])}\n")

    def pc(x):
        return "" if x is None else f"{100 * x:.0f}%"

    w("\n## scenarios\n| scenario | section | NS card | EW card | BBA works | boards | NS calls = BBA | contract = BBA | vs BBA (IMPs) |\n"
      "|---|---|---|---|---|---:|---:|---:|---:|\n")
    for n, s in sorted(scen.items(), key=lambda x: (str(x[1]["section"]), x[0])):
        c = s.get("compare", {})
        w(f"| {n} | {s['section'] or '(not in layout)'} | {s['ns']} | {s['ew']} | {'yes' if s['bba_works'] else '**no**'} | "
          f"{s['boards']} | {pc(c.get('ns_calls'))} | {pc(c.get('contracts'))} | {c.get('imps', '')} |\n")

    w("\n## families (layout sections)\n| section | scenarios | BBA works: no | boards | NS cards | NS calls = BBA | contract = BBA | vs BBA (IMPs) |\n"
      "|---|---:|---:|---:|---|---:|---:|---:|\n")
    fam = collections.defaultdict(list)
    for n, s in scen.items():
        fam[s["section"] or "(not in layout)"].append(s)
    for sec, ss in sorted(fam.items(), key=lambda x: -sum(s["boards"] for s in x[1])):
        cs = [s["compare"] for s in ss if "compare" in s]
        bd = sum(s["boards"] for s in ss)
        wsum = lambda k: sum(c[k] * 1 for c in cs if c[k] is not None) / max(1, sum(1 for c in cs if c[k] is not None))
        nsc = collections.Counter(s["ns"] for s in ss)
        w(f"| {sec} | {len(ss)} | {sum(1 for s in ss if not s['bba_works'])} | {bd} | "
          f"{', '.join(f'{k} ({v})' if len(nsc) > 1 else k for k, v in nsc.most_common())} | "
          f"{pc(wsum('ns_calls')) if cs else ''} | {pc(wsum('contracts')) if cs else ''} | {sum(c['imps'] for c in cs) if cs else ''} |\n")

    w("\n## bba-works: false\n")
    for n, s in sorted(scen.items()):
        if not s["bba_works"]:
            w(f"### {n}\n- cards: {s['ns']} / {s['ew']}; boards {s['boards']}; section {s['section']}\n"
              f"- filter: `{s['filter']}`\n- top notes: {dict(s['notes'].most_common(6))}\n```\n{s['chat'][:900]}\n```\n")

    w("\n## catalog\n| id | name | field | in fields | modules |\n")
    for c in out["catalog"]:
        w(f"| {c['id']} | {c['name']} | {c['field']} | {c['in_fields']} | {', '.join(c['modules'])} |\n")

    w("\n## BBA notes (top 250)\n| BBA's note | times | scenarios | boards | we first differ at it | vs BBA there (IMPs) |\n"
      "|---|---:|---:|---:|---:|---:|\n")
    for n, c, k in out["notes"][:250]:
        dv = div_by_note.get(n, {})
        w(f"| {n} | {c} | {k} | {dv.get('boards', '')} | {dv.get('first_div', '')} | {dv.get('imps', '')} |\n")
    out["div_by_note"] = div_by_note
    if a.json:
        json.dump(out, open(a.json, "w"), indent=1, default=list)

    w("\n## modules\n")
    for n, m in mods.items():
        w(f"- {n} ({m['file']}, {m['lines']} lines): card {m['card']} param {m['param']}\n")


if __name__ == "__main__":
    main()
