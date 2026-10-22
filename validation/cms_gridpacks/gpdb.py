#!/usr/bin/env python3
"""Build and query the CMS gridpack survey database (schema.sql).

    pixi run -e gridpacks gpdb <command> ...

Commands, in pipeline order:
  inventory  list every gridpack tarball under the CVMFS roots (or import a
             `<size> <path>` listing) into `inventory`
  sample     select a named, reproducible sample from the inventory, or import
             one from a TSV
  extract    pull the card-like members of each sampled tarball into the
             extract cache (one `tar` pass per tarball; skipped when the cache
             already matches the tarball's size and mtime)
  parse      (re)build every gridpack table from the extract cache
  scrape     extract + parse
  run        drive `vibegraph integrate` over a sample (see vgrun.py)
  status     row counts, and the outcome table of the latest runs

Tarball contents are untrusted: they are extracted into per-gridpack cache
directories, symlinks in them are never followed, and UFO model files are read
with `ast`, never imported.
"""

import argparse
import concurrent.futures
import datetime
import glob
import hashlib
import json
import os
import re
import socket
import sqlite3
import subprocess
import sys
import zlib

import parse as P

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
DEFAULT_DB = os.path.join(HERE, "gridpacks.sqlite")
DEFAULT_CACHE = os.path.join(HERE, "cache")
MG_DEFAULTS = os.path.join(REPO, "validation", "madgraph", "runcard_defaults.json")
INVENTORY_ROOTS = [
    "/cvmfs/cms-griddata.cern.ch/phys_generator/gridpacks_tarball",
    "/cvmfs/cms.cern.ch/phys_generator/gridpacks",
]
PDFSETS_INDEX = "/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current/pdfsets.index"
TARBALL_EXT = (".tar.xz", ".tgz", ".tar.gz", ".tar")

# Members kept from each tarball. GNU tar's unanchored wildcards match at any
# directory depth, which covers both the LO (process/madevent/...) and NLO
# (process/...) layouts.
EXTRACT_PATTERNS = [
    "InputCards/*", "process/madevent/Cards/*.dat", "process/Cards/*.dat", "mgbasedir/VERSION",
    "gridpack_generation*.log", "bin/internal/ufomodel/*", "mgbasedir/models/*/restrict_*.dat",
    "runcmsgrid.sh", "SubProcesses/results.dat",
]
# madspingrid holds MadSpin's own process directories, each with a copy of the model.
EXTRACT_EXCLUDES = ["*.pyc", "*.pyo", "*__pycache__*", "*.pkl", "mgbasedir/Template/*", "*/SubProcesses/P*",
                    "*/madspingrid/*"]


# --- database ----------------------------------------------------------------

SCHEMA_VERSION = 1


def connect(path):
    """Open the database, creating it from schema.sql when empty. Foreign keys are
    per connection in SQLite, so every session has to come through here."""
    db = sqlite3.connect(path)
    db.execute("PRAGMA foreign_keys = ON")
    db.execute("PRAGMA journal_mode = WAL")
    have = db.execute("PRAGMA user_version").fetchone()[0]
    if have == 0 and db.execute("SELECT count(*) FROM sqlite_master").fetchone()[0] == 0:
        with open(os.path.join(HERE, "schema.sql")) as fh:
            db.executescript(fh.read())
    elif have != SCHEMA_VERSION:
        sys.exit("%s has schema version %d; this gpdb needs %d. Create a fresh database and copy the tables "
                 "across (see schema.sql)." % (path, have, SCHEMA_VERSION))
    assert db.execute("PRAGMA foreign_keys").fetchone()[0] == 1
    return db


def git_head():
    try:
        return subprocess.run(["git", "-C", REPO, "describe", "--tags", "--always", "--dirty"],
                              capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def new_scrape(db, argv):
    cur = db.execute("INSERT INTO scrape (started_at, command, scraper_git, host) VALUES (?, ?, ?, ?)",
                     (now(), " ".join(argv), git_head(), socket.gethostname()))
    db.commit()
    return cur.lastrowid


def db_dir(args):
    """Paths stored in the database (cache dirs, traces) are relative to the database's directory."""
    return os.path.dirname(os.path.abspath(args.db))


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")


def put_blob(db, data):
    """Store bytes once by sha256; returns the hash."""
    sha = hashlib.sha256(data).hexdigest()
    if db.execute("SELECT 1 FROM blob WHERE sha256 = ?", (sha,)).fetchone() is None:
        packed = zlib.compress(data, 9) if len(data) > 4096 else data
        db.execute("INSERT INTO blob (sha256, size_bytes, encoding, content) VALUES (?, ?, ?, ?) "
                   "ON CONFLICT (sha256) DO NOTHING",
                   (sha, len(data), "zlib" if packed is not data else "raw", packed))
    return sha


def get_blob(db, sha):
    row = db.execute("SELECT encoding, content FROM blob WHERE sha256 = ?", (sha,)).fetchone()
    if row is None:
        return None
    return zlib.decompress(row[1]) if row[0] == "zlib" else row[1]


# --- inventory -----------------------------------------------------------------

def cmd_inventory(db, args, scrape_id):
    rows = []
    if args.from_file:
        for line in open(args.from_file):
            tok = line.split(None, 1)
            if len(tok) == 2 and tok[1].strip().endswith(TARBALL_EXT):
                path = tok[1].strip()
                rows.append((path, root_of(path), int(tok[0]), None))
    else:
        for root in args.roots or INVENTORY_ROOTS:
            expr = []
            for ext in TARBALL_EXT:
                expr += ["-o", "-name", "*" + ext]
            cmd = ["find", root, "-type", "f", "("] + expr[1:] + [")", "-printf", r"%s\t%T@\t%p\n"]
            print("scanning", root, flush=True)
            out = subprocess.run(cmd, capture_output=True, text=True).stdout
            for line in out.splitlines():
                size, mtime, path = line.split("\t", 2)
                rows.append((path, root, int(size), int(float(mtime))))
    db.executemany("""INSERT INTO inventory (path, root, size_bytes, mtime, last_seen_scrape)
                      VALUES (?, ?, ?, ?, ?)
                      ON CONFLICT (path) DO UPDATE SET size_bytes = excluded.size_bytes,
                        mtime = coalesce(excluded.mtime, inventory.mtime),
                        last_seen_scrape = excluded.last_seen_scrape""",
                   [r + (scrape_id,) for r in rows])
    db.commit()
    print("%d tarballs recorded" % len(rows))


def root_of(path):
    for r in INVENTORY_ROOTS:
        if path.startswith(r + "/"):
            return r
    return os.path.dirname(path)


# --- sample ------------------------------------------------------------------------

def label_of(path):
    base = os.path.basename(path)
    for ext in TARBALL_EXT:
        if base.endswith(ext):
            base = base[: -len(ext)]
    return re.sub(r"_(slc\d|el\d).*$", "", base)


def unique_labels(paths):
    """{path: label}, with a short hash of the path appended wherever two tarballs
    (a v1 and a v2 folder, say) reduce to the same label; the runner names each
    gridpack's work directory by its label."""
    groups = {}
    for p in paths:
        groups.setdefault(label_of(p), []).append(p)
    return {p: (lab if len(ps) == 1 else "%s-%s" % (lab, hashlib.sha256(p.encode()).hexdigest()[:6]))
            for lab, ps in groups.items() for p in ps}


def cmd_sample(db, args, scrape_id):
    if args.from_tsv:
        import_sample_tsv(db, args, scrape_id)
        return
    import sample as S
    params = S.default_params()
    for kv in args.param or []:
        k, v = kv.split("=", 1)
        params[k] = json.loads(v)
    paths = [r[0] for r in db.execute("SELECT path FROM inventory ORDER BY path")]
    if not paths:
        sys.exit("the inventory is empty: run `gpdb inventory` first")
    rows = S.stratified(paths, params)
    labels = unique_labels([r["path"] for r in rows])
    db.execute("DELETE FROM sample WHERE sample_name = ?", (args.name,))
    db.executemany("INSERT INTO sample (sample_name, path, stratum, family, order_guess, rank, label) "
                   "VALUES (?, ?, ?, ?, ?, ?, ?)",
                   [(args.name, r["path"], r["stratum"], r["family"], r["order_guess"], r["rank"],
                     labels[r["path"]]) for r in rows])
    upsert_sample_def(db, args.name, S.RULE, json.dumps(params, sort_keys=True), scrape_id)
    db.commit()
    print("%s: %d gridpacks" % (args.name, len(rows)))


def upsert_sample_def(db, name, rule, params, scrape_id):
    # Not INSERT OR REPLACE: that deletes the row, and the delete cascades to the sample's members.
    db.execute("INSERT INTO sample_def (sample_name, rule, params, created_scrape) VALUES (?, ?, ?, ?) "
               "ON CONFLICT (sample_name) DO UPDATE SET rule = excluded.rule, params = excluded.params, "
               "created_scrape = excluded.created_scrape", (name, rule, params, scrape_id))


def import_sample_tsv(db, args, scrape_id):
    """TSV with a header naming at least `path`; optional label, stratum, family, order_guess."""
    with open(args.from_tsv) as fh:
        head = fh.readline().rstrip("\n").split("\t")
        rows = [dict(zip(head, line.rstrip("\n").split("\t"))) for line in fh if line.strip()]
    labels = unique_labels([r["path"] for r in rows])
    db.execute("DELETE FROM sample WHERE sample_name = ?", (args.name,))
    for i, r in enumerate(rows):
        db.execute("INSERT INTO sample (sample_name, path, stratum, family, order_guess, rank, label) "
                   "VALUES (?, ?, ?, ?, ?, ?, ?)",
                   (args.name, r["path"], r.get("stratum", "imported"), r.get("family", ""), r.get("order_guess"),
                    i, r.get("label") or labels[r["path"]]))
    upsert_sample_def(db, args.name, "imported", json.dumps({"from_tsv": os.path.basename(args.from_tsv)}),
                      scrape_id)
    db.commit()
    print("%s: imported %d gridpacks" % (args.name, len(rows)))


def sampled(db, name, only=None):
    rows = db.execute("SELECT path, label FROM sample WHERE sample_name = ? ORDER BY stratum, family, rank",
                      (name,)).fetchall()
    if not rows:
        sys.exit("no sample named %r" % name)
    if only:
        rows = [r for r in rows if any(r[1].startswith(o) for o in only)]
    return rows


# --- extract -----------------------------------------------------------------------

def cache_dir(cache, path):
    return os.path.join(cache, hashlib.sha256(path.encode()).hexdigest()[:20])


def extract_one(cache, path, force=False):
    """Extract one tarball's kept members into <cache>/<key>/x; returns (path, status)."""
    d = cache_dir(cache, path)
    src = os.path.join(d, "source.json")
    try:
        st = os.stat(path)
    except OSError as e:
        return path, "missing: %s" % e
    want = {"path": path, "size": st.st_size, "mtime": int(st.st_mtime),
            "patterns": EXTRACT_PATTERNS, "excludes": EXTRACT_EXCLUDES}
    if not force and os.path.exists(src):
        have = json.load(open(src))
        if all(have.get(k) == want[k] for k in want) and have.get("status") == "ok":
            return path, "cached"
    x = os.path.join(d, "x")
    if os.path.exists(x):
        subprocess.run(["rm", "-rf", x], check=True)
    os.makedirs(x)
    cmd = ["tar", "-xf", path, "-C", x, "--no-same-owner", "--no-same-permissions",
           "--wildcards", "--no-anchored"]
    cmd += ["--exclude=" + e for e in EXTRACT_EXCLUDES] + EXTRACT_PATTERNS
    t0 = datetime.datetime.now()
    p = subprocess.run(cmd, capture_output=True, text=True)
    n = sum(len(f) for _, _, f in os.walk(x))
    # tar exits 2 when some patterns match nothing, which every gridpack does for
    # one layout or the other; what matters is that the kept files arrived.
    status = "ok" if n > 0 and p.returncode in (0, 2) else "failed: rc=%d %s" % (p.returncode, p.stderr[-500:])
    want.update(status=status, files=n, seconds=(datetime.datetime.now() - t0).total_seconds(),
                tar_stderr=p.stderr[-2000:])
    with open(src, "w") as fh:
        json.dump(want, fh, indent=1)
    return path, status


def cmd_extract(db, args, scrape_id):
    rows = sampled(db, args.sample, args.only)
    os.makedirs(args.cache, exist_ok=True)
    with concurrent.futures.ThreadPoolExecutor(args.jobs) as ex:
        futs = [ex.submit(extract_one, args.cache, path, args.force) for path, _ in rows]
        for i, f in enumerate(concurrent.futures.as_completed(futs), 1):
            path, status = f.result()
            print("[%d/%d] %s %s" % (i, len(rows), status, label_of(path)), flush=True)


# --- parse ---------------------------------------------------------------------------

def walk_files(root):
    """{relpath: abspath} of regular files under root, never following symlinks."""
    out = {}
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if not os.path.islink(os.path.join(dirpath, d))]
        for f in filenames:
            p = os.path.join(dirpath, f)
            if not os.path.islink(p) and os.path.isfile(p):
                out[os.path.relpath(p, root)] = p
    return out


def role_of(rel):
    base = os.path.basename(rel)
    if rel.startswith("InputCards/"):
        for key, role in (("proc_card", "proc_input"), ("run_card", "run_input"), ("customizecards", "customize"),
                          ("madspin", "madspin"), ("reweight", "reweight"), ("extramodels", "extramodels")):
            if key in base:
                return role
        return "input_other"
    if "/bin/internal/ufomodel/" in "/" + rel:
        return "ufo"
    if re.match(r"mgbasedir/models/[^/]+/restrict_.*\.dat$", rel):
        return "mg_restrict"
    if "/Cards/" in "/" + rel:
        return {"run_card.dat": "run", "param_card.dat": "param", "param_card_default.dat": "param_default",
                "proc_card_mg5.dat": "proc_mg5"}.get(base, "cards_other")
    if base.startswith("gridpack_generation") and base.endswith(".log"):
        return "gen_log"
    if base == "results.dat":
        return "results_dat"
    if rel == "mgbasedir/VERSION":
        return "mg_version"
    if base == "runcmsgrid.sh":
        return "runcmsgrid"
    return "other"


def pick(files, role, prefer=()):
    cands = sorted(r for r in files if role_of(r) == role)
    for p in prefer:
        for c in cands:
            if p in c:
                return c
    return cands[0] if cands else None


def read(p):
    with open(p, "rb") as fh:
        return fh.read()


def text(p):
    return read(p).decode("utf-8", errors="replace")


def pdf_names():
    names = {}
    if os.path.exists(PDFSETS_INDEX):
        for line in open(PDFSETS_INDEX):
            tok = line.split()
            if len(tok) >= 2 and tok[0].isdigit():
                names[int(tok[0])] = tok[1]
    return names


CHILD_TABLES = ["gridpack_file", "gridpack_model", "process_line", "multiparticle", "proc_command",
                "run_card_entry", "param_card_diff", "customize_line", "mg_xsec", "mg_generation",
                "mg_diagram_gen", "mg_step"]


def parse_one(db, path, cache, base, scrape_id, mg_defaults, pdfs):
    d = cache_dir(cache, path)
    src = os.path.join(d, "source.json")
    if not os.path.exists(src):
        return "not extracted"
    meta = json.load(open(src))
    row = db.execute("SELECT gridpack_id FROM gridpack WHERE path = ?", (path,)).fetchone()
    label = label_of(path)
    if row is None:
        gid = db.execute("INSERT INTO gridpack (path, label) VALUES (?, ?)", (path, label)).lastrowid
    else:
        gid = row[0]
        for t in CHILD_TABLES:
            db.execute("DELETE FROM %s WHERE gridpack_id = ?" % t, (gid,))
    db.execute("""UPDATE gridpack SET label = ?, size_bytes = ?, mtime = ?, cache_dir = ?, extract_status = ?,
                  scrape_id = ?, parser_version = ?, parse_status = NULL, parse_error = NULL, mg_version = NULL,
                  order_ = NULL, matching = NULL, lpp1 = NULL, lpp2 = NULL, sqrt_s_gev = NULL, lhaid = NULL,
                  pdf_name = NULL, generated_os = NULL WHERE gridpack_id = ?""",
               (label, meta["size"], meta["mtime"], os.path.relpath(d, base), meta["status"], scrape_id,
                P.PARSER_VERSION, gid))
    if meta["status"] != "ok":
        db.execute("UPDATE gridpack SET parse_status = 'skipped' WHERE gridpack_id = ?", (gid,))
        return "extract " + meta["status"]
    # The reset above already opened a transaction, so this savepoint nests in it:
    # a failed fill leaves no partial rows, and nothing commits until cmd_parse does.
    db.execute("SAVEPOINT fill")
    try:
        fill_gridpack(db, gid, walk_files(os.path.join(d, "x")), scrape_id, mg_defaults, pdfs)
    except Exception as e:  # recorded per gridpack so one odd tarball cannot stop a scrape
        db.execute("ROLLBACK TO fill")
        db.execute("RELEASE fill")
        db.execute("UPDATE gridpack SET parse_status = 'failed', parse_error = ? WHERE gridpack_id = ?",
                   ("%s: %s" % (type(e).__name__, e), gid))
        return "parse failed: %s" % e
    db.execute("RELEASE fill")
    db.execute("UPDATE gridpack SET parse_status = 'ok' WHERE gridpack_id = ?", (gid,))
    return "ok"


def fill_gridpack(db, gid, files, scrape_id, mg_defaults, pdfs):
    shas = {}
    for rel, p in sorted(files.items()):
        role = role_of(rel)
        shas[rel] = put_blob(db, read(p))
        db.execute("INSERT INTO gridpack_file (gridpack_id, path_in_tarball, role, sha256) VALUES (?, ?, ?, ?)",
                   (gid, rel, role, shas[rel]))

    upd = {}
    v = pick(files, "mg_version")
    if v:
        m = re.search(r"version\s*=\s*(\S+)", text(files[v]))
        upd["mg_version"] = m.group(1) if m else None

    proc_rel = pick(files, "proc_input") or pick(files, "proc_mg5")
    proc = P.parse_proc_card(text(files[proc_rel])) if proc_rel else None
    if proc:
        for label, members in proc["defines"].items():
            db.execute("INSERT OR REPLACE INTO multiparticle VALUES (?, ?, ?)", (gid, label, members))
        for seq, (kind, body, pp) in enumerate(proc["processes"]):
            db.execute("INSERT INTO process_line VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                       (gid, seq, kind, body, pp["tag"], pp["initial_state"], pp["final_state"], pp["n_final"],
                        json.dumps(pp["coupling_orders"], sort_keys=True), " ".join(pp["excluded"]),
                        int(pp["has_decay_chain"]), pp["nlo_bracket"]))
        for seq, (cmd, line) in enumerate(proc["commands"]):
            db.execute("INSERT INTO proc_command VALUES (?, ?, ?, ?)", (gid, seq, cmd, line))
        upd["order_"] = "NLO" if any(pp["nlo_bracket"] is not None for _, _, pp in proc["processes"]) else "LO"

    run_rel = pick(files, "run", prefer=("process/madevent/", "process/")) or pick(files, "run_input")
    rc = {}
    if run_rel:
        for key, value, hidden in P.parse_run_card(text(files[run_rel])):
            dtext, is_def = P.default_for(mg_defaults, key, value)
            db.execute("INSERT INTO run_card_entry VALUES (?, ?, ?, ?, ?, ?)",
                       (gid, key, value, int(hidden), dtext, is_def))
            rc[key] = value
    num = lambda k: P._f(rc.get(k, "").replace("d", "e")) if rc.get(k) is not None else None
    if rc:
        # A card without the key runs with MadGraph's default (ickkw = 0, no matching).
        ickkw = num("ickkw") if "ickkw" in rc else float(mg_defaults.get("ickkw", 0))
        upd["matching"] = {0: "none", 1: "MLM", 3: "FxFx"}.get(int(ickkw), "ickkw=%d" % ickkw) if ickkw is not None \
            else None
        upd["lpp1"] = int(num("lpp1")) if num("lpp1") is not None else None
        upd["lpp2"] = int(num("lpp2")) if num("lpp2") is not None else None
        if num("ebeam1") is not None and num("ebeam2") is not None:
            upd["sqrt_s_gev"] = num("ebeam1") + num("ebeam2")
        lhaid = num("lhaid")
        if lhaid is not None:
            upd["lhaid"] = int(lhaid)
            upd["pdf_name"] = pdfs.get(int(lhaid))

    param_rel = pick(files, "param", prefer=("process/madevent/", "process/"))
    pdef_rel = pick(files, "param_default", prefer=("process/madevent/", "process/"))
    if param_rel and pdef_rel:
        for block, ids, val, dval in P.param_diff(text(files[param_rel]), text(files[pdef_rel])):
            db.execute("INSERT OR REPLACE INTO param_card_diff VALUES (?, ?, ?, ?, ?)", (gid, block, ids, val, dval))

    cust = pick(files, "customize")
    if cust:
        sets = [l.strip() for l in text(files[cust]).splitlines() if l.strip().lower().startswith("set ")]
        for seq, l in enumerate(sets):
            db.execute("INSERT INTO customize_line VALUES (?, ?, ?)", (gid, seq, l))

    fill_model(db, gid, files, shas, proc, scrape_id)
    fill_mg_stats(db, gid, files, upd)

    if upd:
        cols = sorted(upd)
        db.execute("UPDATE gridpack SET %s WHERE gridpack_id = ?" % ", ".join("%s = ?" % c for c in cols),
                   [upd[c] for c in cols] + [gid])


def ufo_root(files):
    """The bundled model MadGraph ran with: process/madevent/bin/internal/ufomodel (LO) or
    process/bin/internal/ufomodel (NLO), ahead of any copy elsewhere in the tarball."""
    roots = sorted({r[: r.index("/ufomodel/") + len("/ufomodel")] for r in files if role_of(r) == "ufo"},
                   key=lambda r: (not r.startswith("process/madevent/"), not r.startswith("process/"), len(r), r))
    return roots[0] if roots else None


def fill_model(db, gid, files, shas, proc, scrape_id):
    prefix = ufo_root(files)
    if prefix is None:
        return
    py = {os.path.relpath(rel, prefix): rel for rel in files
          if rel.startswith(prefix + "/") and rel.endswith(".py")}
    model_sha = P.model_identity({r: shas[full] for r, full in py.items()})
    imp = proc["import"] if proc else None
    if db.execute("SELECT 1 FROM ufo_model WHERE model_sha256 = ?", (model_sha,)).fetchone() is not None:
        if imp:
            db.execute("UPDATE ufo_model SET canonical_name = coalesce(canonical_name, ?) WHERE model_sha256 = ?",
                       (imp[0], model_sha))
    else:
        st = P.ufo_stats({r: text(files[full]) for r, full in py.items()})
        cols = ["model_sha256", "canonical_name", "first_seen_scrape"] + sorted(st)
        vals = [model_sha, imp[0] if imp else None, scrape_id] + [st[k] for k in sorted(st)]
        db.execute("INSERT INTO ufo_model (%s) VALUES (%s)" % (", ".join(cols), ", ".join("?" * len(cols))), vals)
        for r, full in py.items():
            db.execute("INSERT INTO ufo_model_file VALUES (?, ?, ?)", (model_sha, r, shas[full]))
    restrict = os.path.join(prefix, "restrict_default.dat")
    mg_restrict = None
    if imp:
        base = imp[0].rsplit("/", 1)[-1]
        cand = "mgbasedir/models/%s/restrict_%s.dat" % (base, imp[1] or "default")
        mg_restrict = shas.get(cand)
    db.execute("INSERT INTO gridpack_model VALUES (?, ?, ?, ?, ?, ?, ?)",
               (gid, model_sha, imp[0] if imp else None, imp[1] if imp else None, imp[2] if imp else None,
                shas.get(restrict), mg_restrict))


def fill_mg_stats(db, gid, files, upd):
    xsecs = []
    gen, diag, steps = {}, [], []
    logs = sorted(r for r in files if role_of(r) == "gen_log")
    if logs:
        lg = P.parse_gen_log(text(files[logs[0]]))
        gen, diag, steps, xsecs = lg["generation"], lg["diagram_gen"], lg["steps"], lg["xsecs"]
        upd["generated_os"] = lg["os"]
    res_rel = pick(files, "results_dat", prefer=("process/madevent/", "process/"))
    if res_rel:
        rd = P.parse_results_dat(text(files[res_rel]))
        if rd["xsec"]:
            xsecs.append(rd["xsec"])
        gen["results_cumulated_time_s"] = rd["cumulated_time_s"]
    if diag:
        gen["diagram_gen_s"] = sum(x["seconds"] for x in diag)
    ref = P.choose_reference(xsecs)
    for i, x in enumerate(xsecs):
        db.execute("INSERT OR REPLACE INTO mg_xsec VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                   (gid, x["source"], x["line"], x["step"], x["sigma_pb"], x["err_pb"], x["nevents"], int(i == ref)))
    if gen:
        cols = sorted(gen)
        db.execute("INSERT INTO mg_generation (gridpack_id, %s) VALUES (?, %s)" % (", ".join(cols), ", ".join(
            "?" * len(cols))), [gid] + [gen[c] for c in cols])
    for seq, x in enumerate(diag):
        db.execute("INSERT INTO mg_diagram_gen VALUES (?, ?, ?, ?, ?)",
                   (gid, seq, x["n_processes"], x["n_diagrams"], x["seconds"]))
    # A step that runs no jobs of its own re-prints the previous step's CPU total.
    for prev, s in zip(steps, steps[1:]):
        if s["cpu_s"] is not None and s["cpu_s"] == prev["cpu_s"]:
            s["cpu_s"] = None
    for seq, s in enumerate(steps):
        db.execute("INSERT INTO mg_step VALUES (?, ?, ?, ?, ?, ?, ?)",
                   (gid, seq, s["step"], s["first_line"], s["wall_s"], s["cpu_s"], s["n_jobs"]))


def mg_submodule_version():
    path = os.path.join(REPO, "research", "refs", "mg5amcnlo", "VERSION")
    if not os.path.exists(path):
        return "unknown"
    m = re.search(r"version\s*=\s*(\S+)", open(path).read())
    return m.group(1) if m else "unknown"


def cmd_parse(db, args, scrape_id):
    rows = sampled(db, args.sample, args.only)
    mg_defaults = P.load_mg_defaults(MG_DEFAULTS)
    pdfs = pdf_names()
    # run_card_entry.mg_default is the pinned submodule's default, not the gridpack's own MG version's.
    db.execute("INSERT OR REPLACE INTO meta VALUES ('run_card_defaults', ?)",
               ("%s (MG %s)" % (os.path.relpath(MG_DEFAULTS, REPO), mg_submodule_version()),))
    counts = {}
    for i, (path, label) in enumerate(rows, 1):
        status = parse_one(db, path, args.cache, db_dir(args), scrape_id, mg_defaults, pdfs)
        db.commit()
        counts[status.split(":")[0]] = counts.get(status.split(":")[0], 0) + 1
        if status != "ok" or args.verbose:
            print("[%d/%d] %s %s" % (i, len(rows), status, label), flush=True)
    print("parsed:", counts)


def cmd_scrape(db, args, scrape_id):
    cmd_extract(db, args, scrape_id)
    cmd_parse(db, args, scrape_id)


# --- status ------------------------------------------------------------------------

def cmd_status(db, args, scrape_id):
    for t in ("inventory", "sample", "gridpack", "ufo_model", "blob", "mg_xsec", "vg_build", "vg_run"):
        print("%-12s %8d" % (t, db.execute("SELECT count(*) FROM %s" % t).fetchone()[0]))
    bad = db.execute("PRAGMA foreign_key_check").fetchall()
    if bad:
        print("foreign-key violations:", bad[:10], "(%d)" % len(bad))
    print()
    q = """SELECT build_id, version, campaign, outcome, coalesce(refusal_class, ''), count(*)
           FROM comparison GROUP BY 1, 2, 3, 4, 5 ORDER BY 1, 3, 6 DESC"""
    for row in db.execute(q):
        print("%3d %-28s %-14s %-8s %-22s %4d" % row)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--db", default=DEFAULT_DB)
    ap.add_argument("--cache", default=DEFAULT_CACHE)
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("inventory")
    p.add_argument("--from-file", help="a `<size> <path>` listing instead of scanning CVMFS")
    p.add_argument("--roots", nargs="*")

    p = sub.add_parser("sample")
    p.add_argument("name")
    p.add_argument("--from-tsv")
    p.add_argument("--param", action="append", help="KEY=JSON override of sample.default_params()")

    for name in ("extract", "parse", "scrape"):
        p = sub.add_parser(name)
        p.add_argument("sample")
        p.add_argument("--only", nargs="*", help="label prefixes")
        p.add_argument("-j", "--jobs", type=int, default=4)
        p.add_argument("--force", action="store_true", help="re-extract even when the cache matches")
        p.add_argument("-v", "--verbose", action="store_true")

    import vgrun
    vgrun.add_parser(sub)

    sub.add_parser("status")
    args = ap.parse_args(argv)
    db = connect(args.db)
    scrape_id = new_scrape(db, sys.argv) if args.cmd != "status" else None
    {"inventory": cmd_inventory, "sample": cmd_sample, "extract": cmd_extract, "parse": cmd_parse,
     "scrape": cmd_scrape, "run": vgrun.cmd_run, "status": cmd_status}[args.cmd](db, args, scrape_id)


if __name__ == "__main__":
    main()
