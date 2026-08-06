#!/usr/bin/env python3
# ---------------------------------------------------------------------------
# rewrite-ai-trailers.py -- convert AI `Co-Authored-By:` trailers to
# `Assisted-by: <harness>:<model>` across the whole of this repository's
# history.
#
# WHY: listing a model as a co-author asserts that the model is a contributor.
# It is not: the human who committed the code is responsible and liable for it.
# The scientific-python development guide
# (https://learn.scientific-python.org/development/guides/ai/) therefore
# forbids `Co-Authored-By:` for models and prescribes the Linux-kernel-style
# `Assisted-by:` trailer instead.
#
# ======================= READ THIS BEFORE RUNNING =========================
#
#   * THIS REWRITES EVERY COMMIT HASH IN THE REPOSITORY.  Every commit from
#     the first rewritten one onward gets a new SHA, because a commit's hash
#     covers its message.  After running you MUST force-push
#     (`git push --force-with-lease --all && git push --force-with-lease --tags`)
#     and every existing clone must either be re-cloned or hard-reset onto the
#     new history.  Coordinate with anyone who has a clone, and do this BEFORE
#     the repository is made public, not after.
#
#   * THIS DROPS GPG SIGNATURES.  A rewritten commit's signature no longer
#     matches its content, so both `git filter-repo` and `git filter-branch`
#     discard the `gpgsig` header.  The script counts signed commits during the
#     dry run and reports them so you know exactly what will be lost.  (In this
#     repository, as of writing: exactly one -- the GitHub web-flow merge
#     commit for PR #2, signed by GitHub's key, not by the author.)
#
#   * Old commit objects are NOT deleted by this script.  It writes a
#     `refs/backup/...` ref for every branch and tag first, and prints the
#     exact command to restore.  The old objects only disappear once you run
#     `git gc --prune=now` after deleting those backup refs.
#
# USAGE
#   scripts/rewrite-ai-trailers.py                # DRY RUN (default; touches nothing)
#   scripts/rewrite-ai-trailers.py --apply        # actually rewrite history
#   scripts/rewrite-ai-trailers.py --apply --engine filter-branch
#
#   --repo PATH        operate on PATH instead of the cwd's repository
#   --engine {auto,filter-repo,filter-branch}
#                      `auto` (default) prefers `git filter-repo` and falls
#                      back to `git filter-branch` when it is not installed.
#                      Both engines are checked by the same post-conditions
#                      below, so whichever runs, a bad result is caught before
#                      you push -- and the backup refs are still there.
#                      NOTE: the end-to-end test of this script was run on the
#                      filter-branch engine; filter-repo could not be exercised
#                      in that sandbox (it drives `git fast-import` over a pipe,
#                      which deadlocked there for reasons unrelated to this
#                      script). If filter-repo misbehaves for you, the
#                      verification will say so -- restore, then re-run with
#                      `--engine filter-branch`.
#
# ON `git filter-repo`
#   filter-repo normally (a) refuses to run outside a fresh clone unless given
#   `--force`, and (b) deletes the `origin` remote and rewrites
#   `refs/remotes/origin/*` into `refs/heads/*` on success.  Neither is wanted
#   here: this is a message-only rewrite of an existing working checkout, and
#   silently inventing local branches for every remote branch would be a
#   surprise.  So this script invokes filter-repo with BOTH `--force` and
#   `--partial`.  `--partial` keeps `origin`, keeps remote-tracking refs where
#   they are, skips the fresh-clone check, and skips the automatic gc (which is
#   what preserves the backup refs' objects).  It also passes:
#     --preserve-commit-hashes   so commit hashes quoted inside commit messages
#                                are left alone -- rewriting them would change
#                                more than the trailer lines.
#     --prune-empty never        so the commit count cannot change.
#     --prune-degenerate never   so merge topology cannot change.
#     --refs <explicit list>     every ref EXCEPT refs/backup/*, so the backup
#                                refs are not themselves rewritten.
#
# SCOPE OF THE EDIT
#   Only whole lines whose `Co-authored-by:` identity appears verbatim in the
#   MAPPING table below are replaced, in place, one line for one line.  Subject,
#   body, blank lines, indentation, the trailing newline, non-AI trailers and
#   any human `Co-authored-by:` are left byte-identical.  No tree is ever
#   touched; author, committer and both dates are preserved.  Running the
#   script a second time is a no-op: `Assisted-by:` is not `Co-authored-by:`,
#   so the census finds nothing to do and the script exits without invoking any
#   rewrite engine at all -- the hashes do not even change.
# ---------------------------------------------------------------------------

from __future__ import annotations

import argparse
import os
import re
import shlex
import shutil
import subprocess
import tempfile
import sys
from collections import Counter, deque

# ===========================================================================
# ============================ EDITABLE MAPPING =============================
# ===========================================================================
#
# This is the ONLY thing you should need to edit.
#
# KEY   -- the identity part of the trailer, exactly as it appears in history:
#          everything after `Co-authored-by:` with surrounding whitespace
#          stripped.  Matching of the `Co-authored-by:` label itself is
#          case-insensitive (this history contains both `Co-Authored-By:` and
#          `Co-authored-by:`); the KEY below is matched case-SENSITIVELY and in
#          full, so a key can never match a trailer it was not meant for.
# VALUE -- the complete replacement line, written in place of the whole
#          original line.
#
# Any `Co-authored-by:` identity NOT listed here is left completely untouched
# and is reported under "UNMAPPED" in the census -- that is the escape hatch
# for genuine human co-authors.
#
# Form is `Assisted-by: <harness>:<model>`, per
# https://learn.scientific-python.org/development/guides/ai/ .  Where the
# harness is known but the model was never recorded, the `:<model>` half is
# omitted rather than invented.

MAPPING: dict[str, str] = {
    # --- Claude Code, model recorded in the trailer ------------------------
    "Claude Opus 5 <noreply@anthropic.com>":       "Assisted-by: claude-code:claude-opus-5",
    "Claude Opus 4.8 <noreply@anthropic.com>":     "Assisted-by: claude-code:claude-opus-4-8",
    # The "(1M context)" variant is the same model with a larger context
    # window configured in the harness, so it maps to the same model id.
    "Claude Opus 4.8 (1M context) <noreply@anthropic.com>":
                                                   "Assisted-by: claude-code:claude-opus-4-8",
    "Claude Sonnet 5 <noreply@anthropic.com>":     "Assisted-by: claude-code:claude-sonnet-5",
    "Claude Sonnet 4.6 <noreply@anthropic.com>":   "Assisted-by: claude-code:claude-sonnet-4-6",
    "Claude Fable 5 <noreply@anthropic.com>":      "Assisted-by: claude-code:claude-fable-5",
    "Claude Haiku 4.5 <noreply@anthropic.com>":    "Assisted-by: claude-code:claude-haiku-4-5",

    # --- Claude Code, model NOT recorded -----------------------------------
    # REVIEW ME.  These commits predate the harness putting a model name in
    # the trailer, so the version is genuinely unknown.  `claude-code:claude`
    # records exactly what history recorded -- the family, not the version --
    # rather than guessing a version that may be wrong.  If you would rather
    # state nothing at all about the model, use "Assisted-by: claude-code".
    "Claude <noreply@anthropic.com>":              "Assisted-by: claude-code:claude",

    # --- GitHub Copilot ----------------------------------------------------
    # Harness is unambiguous from the account; Copilot never recorded which
    # model served the request, so no `:<model>` half.
    "Copilot <223556219+Copilot@users.noreply.github.com>":
                                                   "Assisted-by: github-copilot",

    # --- Qwen ---------------------------------------------------------------
    # REVIEW ME -- THE HARNESS HALF IS A GUESS.  The trailer records the model
    # slug (`qwen/qwen3-coder-next`) but not the harness; the only evidence is
    # the `noreply@github.com` address, which points at a GitHub-hosted agent
    # (Copilot's bring-your-own-model picker uses exactly this `vendor/model`
    # slug form).  That is suggestive, not conclusive -- the same slug form is
    # used by OpenRouter-backed tools.  If you do not want to assert a harness
    # you are not sure of, use "Assisted-by: qwen3-coder-next" (model only) or
    # "Assisted-by: unknown-harness:qwen3-coder-next".
    "qwen/qwen3-coder-next <noreply@github.com>":  "Assisted-by: github-copilot:qwen3-coder-next",
}

# Whole lines to DELETE outright (e.g. `🤖 Generated with [Claude Code](...)`
# banner lines that some harnesses append).  Compared after stripping
# surrounding whitespace.
#
# EMPTY ON PURPOSE: a scan of all 683 commits in this repository found no
# `🤖`, no "Generated with" banner, and no `Signed-off-by:` line.  Kept as a
# hook in case such lines appear later.  Note that deleting a line also tends
# to orphan the blank line above it, so a deletion is not the strictly
# one-line-per-commit edit that a substitution is.
LINE_DELETIONS: set[str] = set()

# ===========================================================================
# ========================= END OF EDITABLE MAPPING =========================
# ===========================================================================


BACKUP_PREFIX = "refs/backup/"

# The label is matched case-insensitively; groups keep every byte of the line
# so the replacement can preserve any trailing CR / whitespace verbatim.
_TRAILER_RE = re.compile(rb"^(co-authored-by:)([ \t]*)(.*?)([ \t\r]*)$", re.IGNORECASE)

_MAPPING_B = {k.encode(): v.encode() for k, v in MAPPING.items()}
_DELETIONS_B = {s.encode() for s in LINE_DELETIONS}


# --------------------------------------------------------------------------
# The rewrite itself.  This single function is used by the dry run, by
# filter-repo's --message-callback, by filter-branch's --msg-filter and by the
# post-rewrite verification, so all four can never disagree.
# --------------------------------------------------------------------------
def rewrite_message(message: bytes) -> bytes:
    """Return `message` with mapped AI trailers replaced. Byte-exact elsewhere."""
    lines = message.split(b"\n")
    out = []
    changed = False
    for line in lines:
        if _DELETIONS_B and line.strip() in _DELETIONS_B:
            changed = True
            continue
        m = _TRAILER_RE.match(line)
        if m is not None:
            identity = m.group(3)
            replacement = _MAPPING_B.get(identity)
            if replacement is not None:
                out.append(replacement + m.group(4))
                changed = True
                continue
        out.append(line)
    if not changed:
        return message
    return b"\n".join(out)


def classify_message(message: bytes) -> tuple[Counter, Counter]:
    """Return (mapped identity counts, unmapped `Co-authored-by` identity counts)."""
    mapped: Counter = Counter()
    unmapped: Counter = Counter()
    for line in message.split(b"\n"):
        m = _TRAILER_RE.match(line)
        if m is None:
            continue
        identity = m.group(3)
        if identity in _MAPPING_B:
            mapped[identity] += 1
        else:
            unmapped[identity] += 1
    return mapped, unmapped


# --------------------------------------------------------------------------
# git plumbing helpers
# --------------------------------------------------------------------------
def git(repo: str, *args: str, check: bool = True) -> str:
    r = subprocess.run(["git", "-C", repo, *args],
                       stdin=subprocess.DEVNULL, capture_output=True, text=True)
    if check and r.returncode != 0:
        die(f"git {' '.join(args)} failed:\n{r.stderr.strip()}")
    return r.stdout


def git_stdin(repo: str, args: list[str], payload: bytes) -> bytes:
    """Run git with `payload` on stdin, delivered via a temporary FILE.

    Deliberately not `subprocess.run(input=...)`: that hands the child a pipe,
    and a child which both reads a large stdin and writes a large stdout can
    deadlock against the parent depending on how the parent multiplexes. A
    regular file has no such failure mode, and git treats it identically.
    """
    with tempfile.TemporaryDirectory() as td:
        inp = os.path.join(td, "stdin")
        with open(inp, "wb") as fh:
            fh.write(payload)
        with open(inp, "rb") as fh:
            r = subprocess.run(["git", "-C", repo, *args], stdin=fh,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if r.returncode != 0:
        die(f"git {' '.join(args)} failed:\n{r.stderr.decode(errors='replace')}")
    return r.stdout


def die(msg: str) -> None:
    print(f"\nERROR: {msg}", file=sys.stderr)
    sys.exit(1)


def all_refs(repo: str, include_backup: bool = False) -> list[str]:
    """Every direct ref -- heads, tags and remote-tracking -- except backups.

    Symbolic refs (typically `refs/remotes/origin/HEAD`) are excluded: they
    resolve through their target, so rewriting or backing them up as direct
    refs would only pin them to a stale commit.
    """
    out = []
    for line in git(repo, "for-each-ref",
                    "--format=%(refname)%09%(symref)").splitlines():
        name, _, symref = line.partition("\t")
        if symref:
            continue
        if not include_backup and name.startswith(BACKUP_PREFIX):
            continue
        out.append(name)
    return sorted(out)


def read_commits(repo: str, shas: list[str]) -> dict[str, tuple[bytes, bytes]]:
    """sha -> (raw header block, raw message).  One `git cat-file --batch` call.

    Reads the object bytes exactly as stored, with no formatting applied by
    git-log, so a comparison of these bytes is a comparison of the real thing.
    """
    if not shas:
        return {}
    data = git_stdin(repo, ["cat-file", "--batch"], "\n".join(shas).encode())
    out: dict[str, tuple[bytes, bytes]] = {}
    pos = 0
    for sha in shas:
        nl = data.index(b"\n", pos)
        header = data[pos:nl].split()
        if len(header) != 3 or header[1] != b"commit":
            die(f"unexpected cat-file header for {sha}: {data[pos:nl]!r}")
        size = int(header[2])
        body = data[nl + 1: nl + 1 + size]
        pos = nl + 1 + size + 1  # object bytes + the trailing newline
        split = body.find(b"\n\n")
        if split < 0:
            die(f"commit {sha} has no header/message separator")
        out[sha] = (body[:split], body[split + 2:])
    return out


def read_graph(repo: str, refs: list[str]) -> tuple[list[str], dict[str, list[str]]]:
    """Return (all commit shas, sha -> parent shas) reachable from `refs`."""
    txt = git(repo, "rev-list", "--parents", *refs)
    order: list[str] = []
    parents: dict[str, list[str]] = {}
    for line in txt.splitlines():
        bits = line.split()
        order.append(bits[0])
        parents[bits[0]] = bits[1:]
    return order, parents


def header_field(header: bytes, name: bytes) -> bytes | None:
    for line in header.split(b"\n"):
        if line.startswith(name + b" "):
            return line[len(name) + 1:]
    return None


# --------------------------------------------------------------------------
# census
# --------------------------------------------------------------------------
class Census:
    def __init__(self) -> None:
        self.mapped: Counter = Counter()
        self.unmapped: Counter = Counter()
        self.affected_commits = 0
        self.total_commits = 0
        self.existing_assisted_by = 0
        self.signed_commits: list[str] = []
        self.per_commit_hits: dict[str, int] = {}


def take_census(repo: str, shas: list[str],
                objs: dict[str, tuple[bytes, bytes]]) -> Census:
    c = Census()
    c.total_commits = len(shas)
    for sha in shas:
        header, msg = objs[sha]
        if header_field(header, b"gpgsig") is not None or b"\ngpgsig" in header:
            c.signed_commits.append(sha)
        mapped, unmapped = classify_message(msg)
        c.mapped.update(mapped)
        c.unmapped.update(unmapped)
        n = sum(mapped.values())
        if n:
            c.affected_commits += 1
            c.per_commit_hits[sha] = n
        for line in msg.split(b"\n"):
            if line.lower().startswith(b"assisted-by:"):
                c.existing_assisted_by += 1
    return c


def print_census(c: Census) -> None:
    print("=" * 74)
    print("TRAILER CENSUS")
    print("=" * 74)
    print(f"  commits scanned                     : {c.total_commits}")
    print(f"  commits carrying >=1 mapped trailer : {c.affected_commits}")
    print(f"  commits carrying none               : {c.total_commits - c.affected_commits}")
    print(f"  pre-existing `Assisted-by:` lines   : {c.existing_assisted_by}")
    print(f"  GPG-signed commits (signature lost) : {len(c.signed_commits)}"
          + (f"  -> {', '.join(s[:12] for s in c.signed_commits)}" if c.signed_commits else ""))
    print()
    print("  MAPPED  (count  original trailer identity  ->  replacement line)")
    if not c.mapped:
        print("    (none)")
    for identity, n in c.mapped.most_common():
        print(f"    {n:5d}  Co-authored-by: {identity.decode()}")
        print(f"           -> {_MAPPING_B[identity].decode()}")
    print(f"    {'-' * 5}")
    print(f"    {sum(c.mapped.values()):5d}  total trailer lines to rewrite")
    print()
    print("  UNMAPPED `Co-authored-by:` identities (LEFT UNCHANGED -- human co-authors)")
    if not c.unmapped:
        print("    (none -- every Co-authored-by trailer in this history is an AI one)")
    for identity, n in c.unmapped.most_common():
        print(f"    {n:5d}  {identity.decode()}")
    multi = sorted((n, s) for s, n in c.per_commit_hits.items() if n > 1)
    print()
    print(f"  commits with more than one mapped trailer: {len(multi)}")
    for n, sha in reversed(multi[-5:]):
        print(f"    {sha[:12]}  {n} trailers")
    print("=" * 74)


# --------------------------------------------------------------------------
# safety
# --------------------------------------------------------------------------
def preflight(repo: str, applying: bool) -> None:
    top = git(repo, "rev-parse", "--show-toplevel").strip()
    print(f"repository : {top}")
    if git(repo, "rev-parse", "--is-bare-repository").strip() == "true":
        die("this is a bare repository; run in a normal checkout")
    head = git(repo, "symbolic-ref", "--quiet", "HEAD", check=False).strip()
    branch = git(repo, "rev-parse", "--abbrev-ref", "HEAD").strip()
    print(f"HEAD       : {branch}")
    if not applying:
        return
    if not head:
        die("HEAD is detached. Check out a branch before rewriting, so the "
            "rewrite has a branch to move.")
    dirty = git(repo, "status", "--porcelain").strip()
    if dirty:
        die("working tree is not clean. Commit or stash first -- a rewrite "
            "moves every branch and would strand your changes.\n"
            + "\n".join("  " + l for l in dirty.splitlines()[:20]))
    if git(repo, "rev-parse", "--verify", "--quiet", "REBASE_HEAD",
           check=False).strip():
        die("a rebase appears to be in progress")


def make_backup(repo: str, refs: list[str]) -> str:
    existing = git(repo, "for-each-ref", "--format=%(refname)",
                   BACKUP_PREFIX).split()
    if existing:
        die(f"{len(existing)} refs already exist under {BACKUP_PREFIX} from a "
            f"previous run. Inspect them, then delete with:\n"
            f"  git for-each-ref --format='delete %(refname)' {BACKUP_PREFIX} | "
            f"git update-ref --stdin")
    cmds = []
    for ref in refs:
        sha = git(repo, "rev-parse", ref).strip()
        # refs/heads/main -> refs/backup/heads/main
        cmds.append(f"create {BACKUP_PREFIX}{ref[len('refs/'):]} {sha}")
    git_stdin(repo, ["update-ref", "--stdin"], ("\n".join(cmds) + "\n").encode())
    restore = (
        "git for-each-ref --format='update refs/%(refname:lstrip=2) %(objectname)' "
        f"{BACKUP_PREFIX} | git update-ref --stdin"
    )
    print()
    print("-" * 74)
    print(f"BACKUP: created {len(cmds)} refs under {BACKUP_PREFIX}")
    for c in cmds:
        print("  " + c.split(" ", 1)[1])
    print()
    print("  TO RESTORE THE ORIGINAL HISTORY, run exactly:")
    print(f"    {restore}")
    print()
    print("  Then, once you are satisfied, drop the backups with:")
    print(f"    git for-each-ref --format='delete %(refname)' {BACKUP_PREFIX} "
          f"| git update-ref --stdin")
    print("-" * 74)
    return restore


# --------------------------------------------------------------------------
# engines
# --------------------------------------------------------------------------
def have_filter_repo() -> str | None:
    return shutil.which("git-filter-repo") or shutil.which("filter-repo")


def run_filter_repo(repo: str, exe: str, refs: list[str]) -> None:
    me = os.path.abspath(__file__)
    # Import this file once per process and delegate, so the message logic in
    # the callback is literally the function above, not a copy of it.
    callback = (
        "_m = globals().get('_vg_rw')\n"
        "if _m is None:\n"
        "    import importlib.util as _ilu\n"
        f"    _sp = _ilu.spec_from_file_location('_vg_rw_mod', {me!r})\n"
        "    _m = _ilu.module_from_spec(_sp)\n"
        "    _sp.loader.exec_module(_m)\n"
        "    globals()['_vg_rw'] = _m\n"
        "return _m.rewrite_message(message)\n"
    )
    cmd = [exe,
           "--force",              # existing checkout, not a fresh clone
           "--partial",            # keep `origin`, keep refs/remotes/*, no gc
           "--preserve-commit-hashes",   # do not touch hashes quoted in messages
           "--prune-empty", "never",
           "--prune-degenerate", "never",
           "--refs", *refs,        # every ref except refs/backup/*
           "--message-callback", callback]
    print("\n$ " + " ".join(
        (c if "\n" not in c else "<callback>") for c in cmd))
    r = subprocess.run(cmd, cwd=repo)
    if r.returncode != 0:
        die("git filter-repo failed")


def run_filter_branch(repo: str, refs: list[str]) -> None:
    me = os.path.abspath(__file__)
    env = dict(os.environ, FILTER_BRANCH_SQUELCH_WARNING="1")
    heads = [r for r in refs if not r.startswith("refs/tags/")]
    worker = f"{shlex.quote(sys.executable)} {shlex.quote(me)} --msg-filter-worker"
    cmd = ["git", "filter-branch", "--force",
           "--msg-filter", worker,
           "--tag-name-filter", "cat",
           "--", *heads]
    print("\n$ " + " ".join(cmd))
    r = subprocess.run(cmd, cwd=repo, env=env)
    if r.returncode != 0:
        die("git filter-branch failed")
    # filter-branch stashes the pre-rewrite refs itself; we already have our
    # own backup namespace, so drop its copy to avoid two competing ones.
    orig = git(repo, "for-each-ref", "--format=delete %(refname)",
               "refs/original/").strip()
    if orig:
        git_stdin(repo, ["update-ref", "--stdin"], (orig + "\n").encode())


# --------------------------------------------------------------------------
# verification
# --------------------------------------------------------------------------
def verify(repo: str, refs: list[str], census: Census) -> bool:
    print()
    print("=" * 74)
    print("POST-REWRITE VERIFICATION")
    print("=" * 74)
    ok = True

    def check(label: str, passed: bool, detail: str = "") -> None:
        nonlocal ok
        ok = ok and passed
        print(f"  [{'PASS' if passed else 'FAIL'}] {label}"
              + (f"  {detail}" if detail else ""))

    backup_refs = git(repo, "for-each-ref", "--format=%(refname)",
                      BACKUP_PREFIX).split()
    old_shas, old_parents = read_graph(repo, backup_refs)
    new_shas, new_parents = read_graph(repo, refs)

    check("commit count unchanged", len(old_shas) == len(new_shas),
          f"{len(old_shas)} -> {len(new_shas)}")
    if len(old_shas) != len(new_shas):
        return False

    old_objs = read_commits(repo, old_shas)
    new_objs = read_commits(repo, new_shas)

    # ---- parallel walk: pair old and new histories by graph position -------
    # Both graphs are isomorphic (a message-only rewrite cannot change
    # topology), so pairing tip-to-tip and then parent-to-parent positionally
    # is exact -- no reliance on rev-list ordering happening to agree.
    pairs: dict[str, str] = {}
    queue: deque[tuple[str, str]] = deque()
    for ref in refs:
        old_tip = git(repo, "rev-parse", f"{BACKUP_PREFIX}{ref[len('refs/'):]}").strip()
        new_tip = git(repo, "rev-parse", ref).strip()
        queue.append((old_tip, new_tip))
    topology_ok = True
    while queue:
        o, n = queue.popleft()
        if o in pairs:
            if pairs[o] != n:
                topology_ok = False
            continue
        pairs[o] = n
        po, pn = old_parents.get(o, []), new_parents.get(n, [])
        if len(po) != len(pn):
            topology_ok = False
            continue
        for a, b in zip(po, pn):
            queue.append((a, b))
    check("history topology preserved (parent arity, parallel walk)", topology_ok)
    check("parallel walk reached every commit",
          len(pairs) == len(old_shas), f"{len(pairs)}/{len(old_shas)} paired")
    if len(pairs) != len(old_shas) or not topology_ok:
        return False

    # ---- the strong check: identical trees, everywhere --------------------
    tree_mismatch = []
    ident_mismatch = []
    msg_mismatch = []
    changed_line_total = 0
    for o, n in pairs.items():
        oh, om = old_objs[o]
        nh, nm = new_objs[n]
        if header_field(oh, b"tree") != header_field(nh, b"tree"):
            tree_mismatch.append((o, n))
        for field in (b"author", b"committer"):
            if header_field(oh, field) != header_field(nh, field):
                ident_mismatch.append((o, n, field.decode()))
        expected = rewrite_message(om)
        if expected != nm:
            msg_mismatch.append((o, n))
        ol, nl = om.split(b"\n"), nm.split(b"\n")
        if len(ol) == len(nl):
            changed_line_total += sum(1 for a, b in zip(ol, nl) if a != b)

    check("every commit's tree hash identical to its pre-rewrite counterpart",
          not tree_mismatch,
          f"{len(pairs)} commits compared"
          if not tree_mismatch else f"{len(tree_mismatch)} mismatched")
    check("author + committer identities and both dates preserved",
          not ident_mismatch,
          f"{len(pairs) * 2} header fields compared"
          if not ident_mismatch else f"{len(ident_mismatch)} mismatched")
    check("every new message is exactly rewrite_message(old message)",
          not msg_mismatch,
          "nothing outside the mapped trailer lines changed"
          if not msg_mismatch else f"{len(msg_mismatch)} mismatched")
    check("changed-line count equals mapped-trailer count",
          changed_line_total == sum(census.mapped.values()),
          f"{changed_line_total} lines changed, "
          f"{sum(census.mapped.values())} trailers mapped")

    # ---- trailer accounting ------------------------------------------------
    after = take_census(repo, new_shas, new_objs)
    check("zero mapped AI Co-Authored-By lines remain",
          sum(after.mapped.values()) == 0,
          f"{sum(after.mapped.values())} remaining")
    expected_ab = census.existing_assisted_by + sum(census.mapped.values())
    check("Assisted-by count equals pre-rewrite AI-trailer count",
          after.existing_assisted_by == expected_ab,
          f"{after.existing_assisted_by} == {census.existing_assisted_by} pre-existing "
          f"+ {sum(census.mapped.values())} converted")
    check("unmapped (human) Co-authored-by trailers untouched",
          after.unmapped == census.unmapped,
          f"{sum(census.unmapped.values())} such lines")

    # ---- a diff anyone can rerun by hand ----------------------------------
    main_new = refs[0]
    for r in refs:
        if r == "refs/heads/main":
            main_new = r
    main_old = f"{BACKUP_PREFIX}{main_new[len('refs/'):]}"
    diff = git(repo, "diff", "--stat", main_old, main_new)
    check(f"`git diff {main_old} {main_new}` is empty",
          diff.strip() == "", repr(diff.strip()[:120]))

    print("=" * 74)
    print("RESULT: " + ("ALL CHECKS PASSED" if ok else "CHECKS FAILED"))
    print("=" * 74)
    return ok


# --------------------------------------------------------------------------
def show_samples(objs: dict[str, tuple[bytes, bytes]], census: Census,
                 shas: list[str], limit: int = 3) -> None:
    """Print full before/after messages: the multi-trailer commit, a couple of
    single-trailer ones with different identities, and one with no trailer."""
    picked: list[str] = []
    multi = sorted(census.per_commit_hits.items(), key=lambda kv: -kv[1])
    if multi:
        picked.append(multi[0][0])
    seen: set[bytes] = set()
    for sha in shas:
        if len(picked) >= limit:
            break
        if sha in picked:
            continue
        mapped, _ = classify_message(objs[sha][1])
        if len(mapped) == 1 and sum(mapped.values()) == 1:
            ident = next(iter(mapped))
            if ident not in seen:
                seen.add(ident)
                picked.append(sha)
    untouched = next((s for s in shas if s not in census.per_commit_hits), None)
    if untouched:
        picked.append(untouched)

    for sha in picked:
        msg = objs[sha][1]
        new = rewrite_message(msg)
        n = census.per_commit_hits.get(sha, 0)
        print()
        print("#" * 74)
        print(f"# SAMPLE {sha}   ({n} mapped trailer(s))")
        print("#" * 74)
        if new == msg:
            print("--- message (UNCHANGED, shown to prove no-trailer commits are "
                  "left alone) ---")
            print(msg.decode(errors="replace").rstrip("\n"))
            continue
        ol, nl = msg.split(b"\n"), new.split(b"\n")
        print("--- unified view: '-' = before, '+' = after, ' ' = byte-identical ---")
        for a, b in zip(ol, nl):
            if a == b:
                print("  " + a.decode(errors="replace"))
            else:
                print("- " + a.decode(errors="replace"))
                print("+ " + b.decode(errors="replace"))


# --------------------------------------------------------------------------
def main() -> int:
    ap = argparse.ArgumentParser(
        description="Rewrite AI Co-Authored-By trailers to Assisted-by across "
                    "all of git history. Dry run unless --apply is given.")
    ap.add_argument("--apply", action="store_true",
                    help="actually rewrite history (default is a dry run)")
    ap.add_argument("--repo", default=".", help="repository path (default: cwd)")
    ap.add_argument("--engine", choices=("auto", "filter-repo", "filter-branch"),
                    default="auto")
    ap.add_argument("--samples", type=int, default=3,
                    help="how many before/after samples to print in the dry run")
    ap.add_argument("--msg-filter-worker", action="store_true",
                    help=argparse.SUPPRESS)  # internal: filter-branch --msg-filter
    args = ap.parse_args()

    if args.msg_filter_worker:
        sys.stdout.buffer.write(rewrite_message(sys.stdin.buffer.read()))
        return 0

    repo = args.repo
    preflight(repo, applying=args.apply)

    refs = all_refs(repo)
    if not refs:
        die("no refs found")
    shas, _ = read_graph(repo, refs)
    objs = read_commits(repo, shas)
    census = take_census(repo, shas, objs)

    print(f"refs       : {len(refs)} "
          f"({sum(1 for r in refs if r.startswith('refs/heads/'))} heads, "
          f"{sum(1 for r in refs if r.startswith('refs/tags/'))} tags, "
          f"{sum(1 for r in refs if r.startswith('refs/remotes/'))} remote-tracking)")
    for r in refs:
        print(f"             {r}")
    print()
    print_census(census)

    if sum(census.mapped.values()) == 0:
        print()
        print("Nothing to do: no mapped AI trailers remain in history.")
        print("(This is what an idempotent second run looks like -- no engine is "
              "invoked, so not a single commit hash changes.)")
        return 0

    if not args.apply:
        show_samples(objs, census, shas, limit=args.samples)
        print()
        print("=" * 74)
        print(f"DRY RUN. {census.affected_commits} of {census.total_commits} "
              f"commits would change, {sum(census.mapped.values())} trailer lines "
              f"in total.")
        print("Nothing has been modified. Re-run with --apply to rewrite.")
        print("=" * 74)
        return 0

    engine = args.engine
    fr = have_filter_repo()
    if engine == "auto":
        engine = "filter-repo" if fr else "filter-branch"
        print(f"\nengine     : {engine} "
              + (f"({fr})" if fr else "(git filter-repo not installed)"))
    elif engine == "filter-repo" and not fr:
        die("--engine filter-repo requested but git-filter-repo is not installed")

    make_backup(repo, refs)

    if engine == "filter-repo":
        run_filter_repo(repo, fr, refs)
    else:
        run_filter_branch(repo, refs)

    if not verify(repo, refs, census):
        print("\nOne or more post-conditions FAILED. Restore with the command "
              "printed above.", file=sys.stderr)
        return 1

    print()
    print("Every commit hash has changed. To publish:")
    print("  git push --force-with-lease --all")
    print("  git push --force-with-lease --tags")
    print("Every existing clone must be re-cloned or hard-reset onto the new "
          "history.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
