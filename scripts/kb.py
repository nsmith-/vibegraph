"""Tooling for the research knowledge bundle (OKF v0.2) and its backlog.

    kb.py backlog [--item SLUG | --area A --state S --priority P] [--claims] [-o FILE] [--link-base URL]
    kb.py lint
    kb.py index [--check]

Every concept is a markdown file with YAML frontmatter carrying a non-empty
`type`; `index.md` and `log.md` are reserved. Backlog items are the
`type: Backlog Item` concepts under `backlog/<area>/`; the backlog view is
rendered from them on demand and never committed, so parallel branches never
conflict over it. `--claims` reads the open pull requests' `Backlog: <slug>`
lines through the GitHub API (needs `GITHUB_TOKEN`).
"""

import argparse
import json
import os
import re
import sys
import tomllib
import urllib.request
from pathlib import Path

import yaml

REPO = Path(__file__).resolve().parent.parent
ROOT = REPO / "research" / "notes"
MANIFEST = REPO / "validation" / "manifest.toml"
GITHUB_REPO = os.environ.get("GITHUB_REPOSITORY", "nsmith-/vibegraph")
RESERVED = {"index.md", "log.md"}
# Directories whose listing is the generated backlog view, not an index.md:
# an index there would change with every item and conflict across branches.
NO_INDEX = {"backlog"}

AREAS = ["validation", "feature", "performance", "hygiene"]
STATES = ["open", "blocked", "needs-user"]
PRIORITIES = ["high", "medium", "low"]
DESCRIPTION_MAX = 200
ITEM_BODY_WARN = 40

FRONTMATTER = re.compile(r"\A---\n(.*?\n)---\n?(.*)\Z", re.S)
LINK = re.compile(r"\]\(([^)#\s]+\.md)(?:#[^)]*)?\)")


class Concept:
    def __init__(self, path, meta, body):
        self.path = path
        self.meta = meta
        self.body = body

    @property
    def rel(self):
        return self.path.relative_to(ROOT).as_posix()

    @property
    def slug(self):
        return self.path.stem

    def __getitem__(self, key):
        return self.meta.get(key)


def parse(path):
    """Return (meta, body, error); meta is None when the file has no valid frontmatter."""
    text = path.read_text(encoding="utf-8")
    m = FRONTMATTER.match(text)
    if not m:
        return None, text, "no YAML frontmatter"
    try:
        meta = yaml.safe_load(m.group(1))
    except yaml.YAMLError as e:
        return None, m.group(2), f"frontmatter does not parse: {e}"
    if not isinstance(meta, dict):
        return None, m.group(2), "frontmatter is not a mapping"
    return meta, m.group(2), None


def concept_files(root=ROOT):
    return sorted(p for p in root.rglob("*.md") if p.name not in RESERVED)


def load(root=ROOT):
    concepts, errors = [], []
    for p in concept_files(root):
        meta, body, err = parse(p)
        if err:
            errors.append((p, err))
        else:
            concepts.append(Concept(p, meta, body))
    return concepts, errors


def items(concepts):
    return [c for c in concepts if c["type"] == "Backlog Item"]


# --- lint -------------------------------------------------------------------


def one_line(value):
    return isinstance(value, str) and "\n" not in value.strip()


def lint(_args):
    concepts, errors = load()
    problems = [f"{p.relative_to(REPO)}: {e}" for p, e in errors]
    warnings = []
    slugs = {}
    for c in concepts:
        where = c.path.relative_to(REPO)
        t = c["type"]
        if not isinstance(t, str) or not t.strip():
            problems.append(f"{where}: `type` missing or empty")
        if c["description"] is not None and not one_line(c["description"]):
            problems.append(f"{where}: `description` must be one line")
        if c["type"] != "Backlog Item":
            continue
        if c.slug in slugs:
            problems.append(f"{where}: slug `{c.slug}` also used by {slugs[c.slug]}")
        slugs[c.slug] = where
        for key, allowed in (("area", AREAS), ("state", STATES), ("priority", PRIORITIES)):
            if c[key] not in allowed:
                problems.append(f"{where}: `{key}` is {c[key]!r}, expected one of {allowed}")
        if c["area"] in AREAS and c.path.parent.name != c["area"]:
            problems.append(f"{where}: `area: {c['area']}` but filed under {c.path.parent.name}/")
        for key in ("title", "description", "closes_when", "opened"):
            if not c[key]:
                problems.append(f"{where}: `{key}` is required on a backlog item")
        d = c["description"]
        if isinstance(d, str) and len(d) > DESCRIPTION_MAX:
            problems.append(f"{where}: `description` is {len(d)} characters (max {DESCRIPTION_MAX})")
        if not isinstance(c["blocked_by"] or [], list):
            problems.append(f"{where}: `blocked_by` must be a list")
        if c["state"] == "blocked" and not c["blocked_by"]:
            warnings.append(f"{where}: `state: blocked` without `blocked_by`; say in the body what it waits on")
        n = len(c.body.strip().splitlines())
        if n > ITEM_BODY_WARN:
            warnings.append(f"{where}: body is {n} lines; move the detail into a linked concept")
    known = {c.slug for c in items(concepts)}
    for c in items(concepts):
        for dep in c["blocked_by"] or []:
            if dep not in known:
                problems.append(f"{c.path.relative_to(REPO)}: `blocked_by` names `{dep}`, which is no backlog item")
    stale = index(argparse.Namespace(check=True), quiet=True)
    problems += [f"{p}: generated index is stale (run `pixi run kb-index`)" for p in stale]
    for w in warnings:
        print(f"warning: {w}", file=sys.stderr)
    for p in problems:
        print(f"error: {p}", file=sys.stderr)
    print(f"{len(concepts)} concepts, {len(items(concepts))} backlog items; "
          f"{len(problems)} errors, {len(warnings)} warnings", file=sys.stderr)
    return 1 if problems else 0


# --- index ------------------------------------------------------------------


def render_index(directory, concepts, is_root):
    here = [c for c in concepts if c.path.parent == directory]
    subdirs = sorted({c.path.relative_to(directory).parts[0] for c in concepts
                      if c.path.parent != directory and directory in c.path.parents})
    out = []
    if is_root:
        out += ["---", 'okf_version: "0.2"', "---", ""]
    name = "Research knowledge bundle" if is_root else directory.name
    out += [f"# {name}", "", "<!-- Generated by `pixi run kb-index` from the concepts' frontmatter; do not edit. -->", ""]
    by_type = {}
    for c in here:
        by_type.setdefault(c["type"], []).append(c)
    for t in sorted(by_type):
        out += [f"## {t}", ""]
        for c in by_type[t]:
            line = f"* [{c['title'] or c.slug}]({c.path.name})"
            if c["description"]:
                line += f" - {c['description']}"
            out.append(line)
        out.append("")
    if subdirs:
        out += ["## Directories", ""]
        for s in subdirs:
            n = sum(1 for c in concepts if (directory / s) in c.path.parents)
            if s in NO_INDEX:
                # No count: it would change with every item and conflict across branches.
                out.append(f"* [{s}]({s}/) - one file per item; rendered by `pixi run backlog`")
            else:
                out.append(f"* [{s}]({s}/index.md) - {n} concepts")
        out.append("")
    return "\n".join(out)


def index(args, quiet=False):
    concepts, _ = load()
    dirs = {ROOT}
    for c in concepts:
        rel = c.path.parent.relative_to(ROOT)
        if rel.parts and rel.parts[0] in NO_INDEX:
            continue
        dirs.add(c.path.parent)
        dirs.update(p for p in c.path.parent.parents if ROOT in p.parents)
    stale = []
    for d in sorted(dirs):
        text = render_index(d, concepts, d == ROOT)
        target = d / "index.md"
        current = target.read_text(encoding="utf-8") if target.exists() else None
        if current == text:
            continue
        stale.append(target.relative_to(REPO))
        if not args.check:
            target.write_text(text, encoding="utf-8")
    if args.check:
        if not quiet:
            for p in stale:
                print(f"stale: {p}", file=sys.stderr)
        return stale if quiet else (1 if stale else 0)
    if not quiet:
        print(f"wrote {len(stale)} index files", file=sys.stderr)
    return 0


# --- backlog view -----------------------------------------------------------


def fetch_claims():
    """Map slug -> [PR numbers] from open pull requests' `Backlog: <slug>` lines."""
    token = os.environ.get("GITHUB_TOKEN")
    if not token:
        print("warning: --claims without GITHUB_TOKEN; claims not shown", file=sys.stderr)
        return {}
    claims, page = {}, 1
    while True:
        req = urllib.request.Request(
            f"https://api.github.com/repos/{GITHUB_REPO}/pulls?state=open&per_page=100&page={page}",
            headers={"Authorization": f"Bearer {token}", "Accept": "application/vnd.github+json"})
        with urllib.request.urlopen(req) as r:
            pulls = json.load(r)
        for pr in pulls:
            for slug in re.findall(r"^Backlog:\s*(\S+)\s*$", pr.get("body") or "", re.M):
                claims.setdefault(slug, []).append(pr["number"])
        if len(pulls) < 100:
            return claims
        page += 1


def census():
    m = tomllib.loads(MANIFEST.read_text(encoding="utf-8"))
    counts, info = {}, []
    for p in m["process"]:
        for cat, cell in p.get("categories", {}).items():
            for x in cell if isinstance(cell, list) else [cell]:
                key = (x.get("tier"), x.get("mode"))
                counts[key] = counts.get(key, 0) + 1
                if x.get("mode") == "info":
                    info.append(f"`{p['key']}` {cat} ({x.get('tier')})")
    tiers = ["hermetic", "banked", "long"]
    lines = [
        f"Declared in `validation/manifest.toml`: {len(m['process'])} process rows. "
        "Which cells are measured green comes from a `validation-report` run, not from here.",
        "",
        "| Tier | gate | info |",
        "|---|---|---|",
    ]
    for t in tiers:
        lines.append(f"| {t} | {counts.get((t, 'gate'), 0)} | {counts.get((t, 'info'), 0)} |")
    other = {t: n for (t, mode), n in counts.items() if t not in tiers}
    lines += ["", ", ".join(f"{n} {t}" for t, n in sorted(other.items())) + " cells carry no mode.", ""]
    if info:
        lines += ["Informational cells: " + "; ".join(info) + ".", ""]
    return lines


def link(c, base):
    return f"{base}{c.rel}" if base is not None else c.path.relative_to(REPO).as_posix()


def item_line(c, base, claims):
    s = f"- [{c['title']}]({link(c, base)}) — {c['description']}"
    tags = [c["state"]] if c["state"] != "open" else []
    if c["blocked_by"]:
        tags.append("after " + ", ".join(f"`{d}`" for d in c["blocked_by"]))
    if c.slug in claims:
        tags.append("claimed by " + ", ".join(
            f"[#{n}](https://github.com/{GITHUB_REPO}/pull/{n})" for n in claims[c.slug]))
    if tags:
        s += " *(" + "; ".join(tags) + ")*"
    return s


def show_item(concepts, slug):
    by_slug = {c.slug: c for c in items(concepts)}
    if slug not in by_slug:
        print(f"no backlog item `{slug}`", file=sys.stderr)
        return 1
    c = by_slug[slug]
    print(c.path.read_text(encoding="utf-8").rstrip())
    seen, chain = {slug}, list(c["blocked_by"] or [])
    if chain:
        print("\n## Blocked by\n")
    while chain:
        d = chain.pop(0)
        if d in seen:
            continue
        seen.add(d)
        dep = by_slug.get(d)
        print(f"- `{d}`: " + (f"{dep['title']} — {dep['description']}" if dep else "(no such item)"))
        chain += list(dep["blocked_by"] or []) if dep else []
    by_path = {c.path.resolve(): c for c in concepts}
    linked = []
    for target in LINK.findall(c.body):
        p = (c.path.parent / target).resolve()
        if p not in [x[0] for x in linked]:
            linked.append((p, by_path.get(p)))
    if linked:
        print("\n## Linked concepts\n")
        for p, lc in linked:
            rel = os.path.relpath(p, REPO)
            if lc:
                print(f"- `{rel}`: {lc['title']} — {lc['description']}")
            else:
                print(f"- `{rel}`" + ("" if p.exists() else " (missing)"))
    return 0


def backlog(args):
    concepts, errors = load()
    for p, e in errors:
        print(f"warning: {p.relative_to(REPO)}: {e}", file=sys.stderr)
    if args.item:
        return show_item(concepts, args.item)
    claims = fetch_claims() if args.claims else {}
    known = {c.slug for c in items(concepts)}
    for slug, prs in claims.items():
        if slug not in known:
            print(f"warning: PR {', '.join(f'#{n}' for n in prs)} claims `{slug}`, which is no backlog item",
                  file=sys.stderr)
    base = args.link_base
    selected = [c for c in items(concepts)
                if (not args.area or c["area"] == args.area)
                and (not args.state or c["state"] == args.state)
                and (not args.priority or c["priority"] == args.priority)]
    filtered = bool(args.area or args.state or args.priority)
    out = ["# Backlog", "",
           "<!-- Generated by `pixi run backlog` from the backlog items; do not edit. -->", ""]
    if not filtered:
        out += ["Rendered from the one-file backlog items under `research/notes/backlog/`. "
                "Each item says what is wrong and what closes it; the PR that closes "
                "an item deletes its file. An open pull request claims an item with a "
                "`Backlog: <slug>` line in its description.", ""]
        sprints = [c for c in concepts if c["type"] == "Sprint" and c["active"]]
        out += ["## Current position", ""]
        out += [f"- [{c['title']}]({link(c, base)}) — {c['description']}" for c in sprints] or \
               ["No sprint is active."]
        out.append("")
        decisions = [c for c in concepts if c["type"] == "Design Decision" and c["status"] != "deprecated"]
        verified = [c for c in decisions
                    if any(str(v.get("by", "")).startswith("human:") for v in c["verified"] or [])]
        if decisions:
            out += ["## Standing decisions", ""]
            out += [f"- [{c['title']}]({link(c, base)}) — {c['description']}" for c in verified]
            if len(decisions) > len(verified):
                out.append(f"- {len(decisions) - len(verified)} more design decisions are drafted "
                           "but not yet reviewed by a person.")
            out.append("")
        user = [c for c in selected if c["state"] == "needs-user"]
        if user:
            out += ["## Open, and the user's call", ""] + [item_line(c, base, claims) for c in user] + [""]
        out += ["## Census", ""] + census()
        facts = [c for c in concepts if c["type"] in ("Measurement", "Caveat")]
        if facts:
            out += ["## Standing measurement facts and caveats", ""]
            out += [f"- [{c['title']}]({link(c, base)}) — {c['description']}" for c in facts] + [""]
    for area in AREAS:
        group = [c for c in selected if c["area"] == area]
        if not group:
            continue
        out += [f"## {area.capitalize()} backlog ({len(group)})", ""]
        for prio in PRIORITIES:
            g = sorted((c for c in group if c["priority"] == prio), key=lambda c: c["title"].lower())
            if g:
                out += [f"### {prio.capitalize()} priority", ""] + [item_line(c, base, claims) for c in g] + [""]
    if not filtered:
        records = [c for c in concepts if c["type"] == "Sprint Record"]
        records.sort(key=lambda c: str(c["closed"] or ""), reverse=True)
        out += ["## Closed sprints", ""]
        for c in records:
            when = f" ({c['closed']})" if c["closed"] else ""
            out.append(f"- [{c['title']}]({link(c, base)}){when} — {c['description']}")
        out.append("")
    text = "\n".join(out)
    if args.output:
        Path(args.output).write_text(text, encoding="utf-8")
    else:
        sys.stdout.write(text)
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("backlog", help="render the backlog view, or one item with --item")
    b.add_argument("--item", help="print one item, its blocked_by chain and its linked concepts")
    b.add_argument("--area", choices=AREAS)
    b.add_argument("--state", choices=STATES)
    b.add_argument("--priority", choices=PRIORITIES)
    b.add_argument("--claims", action="store_true", help="annotate claims from open PRs (needs GITHUB_TOKEN)")
    b.add_argument("--link-base", help="URL prefix for links, e.g. a GitHub blob URL of research/notes/")
    b.add_argument("-o", "--output", help="write to a file instead of stdout")
    b.set_defaults(func=backlog)
    sub.add_parser("lint", help="check the bundle's conformance and the backlog items").set_defaults(func=lint)
    i = sub.add_parser("index", help="regenerate the index.md files")
    i.add_argument("--check", action="store_true", help="only report stale index files")
    i.set_defaults(func=index)
    args = ap.parse_args()
    sys.exit(args.func(args))


if __name__ == "__main__":
    main()
