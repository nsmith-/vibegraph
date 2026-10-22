"""Drive the vibegraph CLI over a scraped sample and record every attempt.

    gpdb run <sample> --campaign NAME [--adapt gpmodel,strip,runpatch] [--bake] [--gridpack-pdf]

Attempt 1 runs the gridpack's own proc card with the run card MadGraph used.
`--adapt` lets later attempts react to the previous refusal, cumulatively:
  gpmodel   import the gridpack's bundled UFO model (bin/internal/ufomodel, whose
            restrict_default.dat is the restriction MadGraph applied) under a
            fresh name, in place of the named model and restriction
  strip     drop the proc-card commands the error names (and everything from
            `launch` on)
  runpatch  reset the run-card field a refusal names to the MadGraph default
            the refusal quotes
Two changes apply up front instead: `--bake` writes the gridpack's param card
into a copy of its model's external parameters (the CLI has no param-card
option), and `--gridpack-pdf` passes the run card's lhaid as `--pdf-set`.
Every change made is listed in the attempt's `patches`, so `patches = '[]'`
marks an as-shipped result.
"""

import gzip
import hashlib
import json
import os
import re
import shutil
import signal
import socket
import subprocess
import time

import gpdb
import parse as P

DEFAULT_BINARY = os.path.join(gpdb.REPO, "target", "release", "vibegraph")
CVMFS_PDF = "/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current"
KEEP = {"import", "define", "generate", "add"}
RUNPATCH = [re.compile(r"run card sets '(\w+)' to \S+ \(MadGraph default '?([^')]+)'?\)"),
            re.compile(r"activates cut '(\w+)' \(= \S+, MG default ([^)]+)\)")]
MODEL_ERROR = re.compile(r"failed to load model|restrict variant|restriction|UFO|model director|no such model|"
                         r"could not find model", re.I)


def add_parser(sub):
    p = sub.add_parser("run", help="drive `vibegraph integrate` over a sample")
    p.add_argument("sample")
    p.add_argument("--campaign", required=True, help="name for this batch and its adaptation policy")
    p.add_argument("--binary", default=DEFAULT_BINARY)
    p.add_argument("--only", nargs="*", help="label prefixes")
    p.add_argument("--adapt", default="", help="comma list of gpmodel,strip,runpatch")
    p.add_argument("--bake", action="store_true", help="bake the gridpack param card into its model")
    p.add_argument("--gridpack-pdf", action="store_true", help="run with the PDF set the run card's lhaid names")
    p.add_argument("--pdf-dir", default=CVMFS_PDF if os.path.isdir(CVMFS_PDF) else
                   os.path.join(gpdb.REPO, "validation", "pdf"))
    p.add_argument("--neval", type=int, default=10000)
    p.add_argument("--niter", type=int, default=3)
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--threads", type=int, default=4)
    p.add_argument("--timeout", type=float, default=900)
    p.add_argument("--max-attempts", type=int, default=6)
    p.add_argument("--rerun", action="store_true", help="replace this build's earlier runs in this campaign")
    p.add_argument("--allow-network", action="store_true", help="let the CLI download missing assets")
    p.add_argument("--work", help="per-attempt card directories (default: work/ beside the database)")
    p.add_argument("--extra", nargs="*", default=[], help="further `integrate` arguments, appended verbatim")


def sha_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def register_build(db, binary):
    out = subprocess.run([binary, "--version"], capture_output=True, text=True, check=True).stdout
    version = out.splitlines()[0].split()[-1]
    m = re.search(r"-g([0-9a-f]{7,})", version) or re.fullmatch(r"([0-9a-f]{7,})(-dirty)?", version)
    bsha = sha_file(binary)
    row = db.execute("SELECT build_id FROM vg_build WHERE version = ? AND binary_sha256 = ?", (version, bsha)).fetchone()
    if row:
        return row[0], version
    cur = db.execute("INSERT INTO vg_build (version, git_sha, dirty, binary_sha256, binary_path, registered_at) "
                     "VALUES (?, ?, ?, ?, ?, ?)",
                     (version, m.group(1) if m else None, int(version.endswith("-dirty")), bsha,
                      os.path.abspath(binary), gpdb.now()))
    db.commit()
    return cur.lastrowid, version


def strip_named(text, err):
    errl = err.lower()
    out, dropped, after_launch = [], [], False
    for line in text.splitlines():
        tok = line.split()
        cmd = tok[0].lower() if tok and not tok[0].startswith("#") else None
        if after_launch:
            dropped.append(line)
            continue
        if cmd and cmd not in KEEP and re.search(r"\b%s\b" % re.escape(cmd), errl):
            dropped.append(line)
            if cmd == "launch":
                after_launch = True
            continue
        out.append(line)
    return "\n".join(out) + "\n", [l.split()[0] for l in dropped if l.split()]


def patch_run(text, name, value):
    out = []
    for line in text.splitlines(True):
        body = line.split("!")[0]
        if "=" in body and body.rsplit("=", 1)[1].split("#")[0].strip().lower() == name.lower():
            line = " %s = %s" % (value, line[len(body.rsplit("=", 1)[0]) + 1:].lstrip())
        out.append(line)
    return "".join(out)


def stage_model(src, dest_root, name, restriction=None):
    """Copy a bundled UFO to <dest_root>/<name>/, adding restrict_<restriction>.dat as
    a copy of its restrict_default.dat when the gridpack named a restriction."""
    target = os.path.join(dest_root, name)
    shutil.rmtree(target, ignore_errors=True)
    shutil.copytree(src, target, symlinks=True, ignore=shutil.ignore_patterns("*.pkl", "*.pyc", "__pycache__"))
    for dirpath, _, files in os.walk(target):
        for f in files:
            if os.path.islink(os.path.join(dirpath, f)):
                os.remove(os.path.join(dirpath, f))
    if restriction:
        want = os.path.join(target, "restrict_%s.dat" % restriction)
        default = os.path.join(target, "restrict_default.dat")
        if not os.path.exists(want) and os.path.exists(default):
            shutil.copy(default, want)
    return target


def bake(model_dir, param_text):
    """Write param-card values into the model's external Parameter(value=...) entries."""
    vals, block = {}, None
    for raw in param_text.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        tok = line.split()
        if tok[0].upper() == "BLOCK":
            block = tok[1].upper() if len(tok) > 1 else None
        elif tok[0].upper() == "DECAY":
            block = None
            if len(tok) >= 3:
                vals[("DECAY", (int(tok[1]),))] = tok[2]
        elif block and len(tok) >= 2:
            try:
                vals[(block, tuple(int(t) for t in tok[:-1]))] = tok[-1]
            except ValueError:
                pass
    path = os.path.join(model_dir, "parameters.py")
    text = open(path).read()
    out, n = [], 0
    for chunk in re.split(r"(?=\n\s*\w+\s*=\s*Parameter\()", text):
        if re.search(r"nature\s*=\s*['\"]external['\"]", chunk):
            b = re.search(r"lhablock\s*=\s*['\"](\w+)['\"]", chunk)
            c = re.search(r"lhacode\s*=\s*\[([^\]]*)\]", chunk)
            if b and c:
                key = (b.group(1).upper(), tuple(int(x) for x in c.group(1).split(",") if x.strip()))
                if key in vals:
                    new = re.sub(r"value\s*=\s*[-+0-9.eEdD]+", "value = %s" % vals[key], chunk, count=1)
                    n += new != chunk
                    chunk = new
        out.append(chunk)
    with open(path, "w") as fh:
        fh.write("".join(out))
    return n


def model_sha(model_dir):
    files = gpdb.walk_files(model_dir)
    return P.model_identity({r: sha_file(p) for r, p in files.items() if r.endswith(".py")})


def execute(cmd, out_path, timeout):
    """Run cmd with output to out_path; -> (exit code or None, timed_out, wall_s, max_rss_kb)."""
    t0 = time.monotonic()
    with open(out_path, "w") as fh:
        p = subprocess.Popen(cmd, stdout=fh, stderr=subprocess.STDOUT, start_new_session=True)
        while True:
            pid, status, ru = os.wait4(p.pid, os.WNOHANG)
            if pid:
                p.returncode = os.waitstatus_to_exitcode(status)
                return p.returncode, False, time.monotonic() - t0, ru.ru_maxrss
            if time.monotonic() - t0 > timeout:
                os.killpg(p.pid, signal.SIGKILL)
                _, status, ru = os.wait4(p.pid, 0)
                return None, True, time.monotonic() - t0, ru.ru_maxrss
            time.sleep(0.2)


def gridpack_inputs(base, cache_rel):
    root = os.path.join(base, cache_rel, "x")
    files = gpdb.walk_files(root)
    proc = gpdb.pick(files, "proc_input") or gpdb.pick(files, "proc_mg5")
    run = gpdb.pick(files, "run", prefer=("process/madevent/", "process/")) or gpdb.pick(files, "run_input")
    param = gpdb.pick(files, "param", prefer=("process/madevent/", "process/"))
    ufo = gpdb.ufo_root(files)
    return ({"proc": files.get(proc), "run": files.get(run), "param": files.get(param),
             "ufomodel": os.path.join(root, ufo) if ufo else None})


def register_campaign(db, args):
    """A campaign name is bound to the policy it was first run with."""
    policy = json.dumps({"adapt": sorted(a for a in args.adapt.split(",") if a), "bake": args.bake,
                         "gridpack_pdf": args.gridpack_pdf, "pdf_dir": args.pdf_dir, "neval": args.neval,
                         "niter": args.niter, "seed": args.seed, "threads": args.threads, "timeout": args.timeout,
                         "max_attempts": args.max_attempts, "allow_network": args.allow_network,
                         "extra": args.extra}, sort_keys=True)
    row = db.execute("SELECT policy FROM vg_campaign WHERE campaign = ?", (args.campaign,)).fetchone()
    if row is None:
        db.execute("INSERT INTO vg_campaign VALUES (?, ?, ?)", (args.campaign, policy, gpdb.now()))
        db.commit()
    elif json.loads(row[0]) != json.loads(policy):
        raise SystemExit("campaign %r was run with policy %s; this invocation asks for %s. Use a new campaign name."
                         % (args.campaign, row[0], policy))


def cmd_run(db, args, scrape_id):
    build_id, version = register_build(db, args.binary)
    register_campaign(db, args)
    args.work = args.work or os.path.join(gpdb.db_dir(args), "work")
    adapt = {a for a in args.adapt.split(",") if a}
    unknown = adapt - {"gpmodel", "strip", "runpatch"}
    if unknown:
        raise SystemExit("unknown --adapt values: %s" % ", ".join(sorted(unknown)))
    os.makedirs(os.path.join(gpdb.db_dir(args), "runs"), exist_ok=True)
    rows = gpdb.sampled(db, args.sample, args.only)
    print("build %s (id %d), campaign %s, %d gridpacks" % (version, build_id, args.campaign, len(rows)), flush=True)
    for i, (path, label) in enumerate(rows, 1):
        g = db.execute("SELECT gridpack_id, cache_dir, parse_status, pdf_name, lhaid FROM gridpack WHERE path = ?",
                       (path,)).fetchone()
        if g is None or g[2] != "ok":
            print("[%d/%d] %s: not scraped, skipped" % (i, len(rows), label), flush=True)
            continue
        gid = g[0]
        done = db.execute("SELECT count(*) FROM vg_run WHERE gridpack_id = ? AND build_id = ? AND campaign = ?",
                          (gid, build_id, args.campaign)).fetchone()[0]
        if done and not args.rerun:
            continue
        last = run_gridpack(db, args, build_id, gid, label, g[1], g[3], g[4], adapt, replace=bool(done))
        print("[%d/%d] %s: %s" % (i, len(rows), label, last), flush=True)


def run_gridpack(db, args, build_id, gid, label, cache_rel, pdf_name, lhaid, adapt, replace=False):
    base = gpdb.db_dir(args)
    inp = gridpack_inputs(base, cache_rel)
    if not inp["proc"] or not inp["run"]:
        return "no proc or run card"
    proc_text = gpdb.text(inp["proc"])
    run_text = gpdb.text(inp["run"])
    imp = next((P.parse_import(l) for l in proc_text.splitlines() if P.parse_import(l)), None)
    work = os.path.join(args.work, args.campaign, label)
    shutil.rmtree(work, ignore_errors=True)
    models = os.path.join(work, "models")
    os.makedirs(models)
    patches, ufo_dir, staged = [], None, None
    gp_name = "gp_" + hashlib.sha256(label.encode()).hexdigest()[:8]

    def use_gridpack_model(text):
        new = []
        for l in text.splitlines():
            pi = P.parse_import(l)
            new.append("import model %s%s" % (gp_name, (" " + pi[2]) if pi and pi[2] else "") if pi else l)
        return "\n".join(new) + "\n"

    if inp["ufomodel"] and (args.bake or (imp and imp[0] != "sm")):
        if args.bake:
            staged = stage_model(inp["ufomodel"], models, gp_name)
            proc_text = use_gridpack_model(proc_text)
            patches.append("model:%s" % gp_name)
            n = bake(staged, gpdb.text(inp["param"])) if inp["param"] else 0
            patches.append("bake:%d" % n)
        else:
            staged = stage_model(inp["ufomodel"], models, imp[0].rsplit("/", 1)[-1], imp[1])
        ufo_dir = models
    pdf_set = None
    if args.gridpack_pdf:
        if pdf_name:
            pdf_set = pdf_name
            patches.append("pdf:%s" % pdf_name)
        else:
            patches.append("pdf:unresolved lhaid %s" % lhaid)

    summary = None
    for attempt in range(1, args.max_attempts + 1):
        adir = os.path.join(work, "attempt%d" % attempt)
        os.makedirs(adir)
        pc, rc = os.path.join(adir, "proc_card.dat"), os.path.join(adir, "run_card.dat")
        open(pc, "w").write(proc_text)
        open(rc, "w").write(run_text)
        out_dir, trace = os.path.join(adir, "out"), os.path.join(adir, "trace.log")
        cmd = [args.binary, "integrate", pc, "--run-card", rc, "--pdf-dir", args.pdf_dir,
               "--neval", str(args.neval), "--niter", str(args.niter), "--fixed-budget", "--seed", str(args.seed),
               "-j", str(args.threads), "--parallel-diagrams", "--no-tui", "--out", out_dir, "--log-file", trace]
        cmd += ["--yes"] if args.allow_network else ["--no-network"]
        if ufo_dir:
            cmd += ["--ufo-dir", ufo_dir]
        if pdf_set:
            cmd += ["--pdf-set", pdf_set]
        cmd += args.extra
        started = gpdb.now()
        code, timed_out, wall, rss = execute(cmd, os.path.join(adir, "output.txt"), args.timeout)
        if replace and attempt == 1:
            # --rerun: the old rows go in the same transaction that records the new
            # attempt, so an interrupted rerun leaves the earlier history intact.
            old = db.execute("SELECT trace_path FROM vg_run WHERE gridpack_id = ? AND build_id = ? AND campaign = ?",
                             (gid, build_id, args.campaign)).fetchall()
            db.execute("DELETE FROM vg_run WHERE gridpack_id = ? AND build_id = ? AND campaign = ?",
                       (gid, build_id, args.campaign))
        output = gpdb.text(os.path.join(adir, "output.txt"))
        trace_text = gpdb.text(trace) if os.path.exists(trace) else ""
        r = P.parse_vg_output(output, trace_text, code, timed_out)
        cur = db.execute(
            """INSERT INTO vg_run (gridpack_id, build_id, campaign, attempt, patches, argv, proc_sha256, run_sha256,
               ufo_model_sha256, pdf_set, neval, niter, seed, threads, fixed_budget, started_at, host, wall_s,
               max_rss_kb, exit_code, timed_out, outcome, sigma_pb, err_pb, chi2_dof, n_evals,
               n_channels, n_subprocesses, n_diagrams, vg_model_hash, error_text, output_sha256)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (gid, build_id, args.campaign, attempt, json.dumps(patches), json.dumps(cmd),
             gpdb.put_blob(db, proc_text.encode()), gpdb.put_blob(db, run_text.encode()),
             model_sha(staged) if staged else None, pdf_set, args.neval, args.niter, args.seed, args.threads, 1,
             started, socket.gethostname(), wall, rss, code, int(timed_out), r["outcome"],
             r["sigma_pb"], r["err_pb"], r["chi2_dof"], r["n_evals"], r["n_channels"], r["n_subprocesses"],
             r["n_diagrams"], r["vg_model_hash"], r["error_text"], gpdb.put_blob(db, output.encode())))
        run_id = cur.lastrowid
        gpdb.put_reasons(db, run_id, r["reasons"])
        if trace_text:
            dest = os.path.join(base, "runs", "%d.trace.log.gz" % run_id)
            with open(trace, "rb") as fi, gzip.open(dest, "wb") as fo:
                shutil.copyfileobj(fi, fo)
            db.execute("UPDATE vg_run SET trace_path = ? WHERE run_id = ?", (os.path.relpath(dest, base), run_id))
        for stage, secs in r["stages"].items():
            db.execute("INSERT INTO vg_run_stage VALUES (?, ?, ?)", (run_id, stage, secs))
        for m in r["multiplicities"]:
            db.execute("INSERT INTO vg_run_multiplicity VALUES (?, ?, ?, ?, ?, ?)",
                       (run_id, m["tag"], m["n_out"], m["sigma_pb"], m["err_pb"], m["n_channels"]))
        db.commit()
        if replace and attempt == 1:
            for (tp,) in old:
                if tp and os.path.exists(os.path.join(base, tp)):
                    os.remove(os.path.join(base, tp))
        shutil.rmtree(out_dir, ignore_errors=True)
        if os.path.exists(trace):
            os.remove(trace)
        summary = "attempt %d %s%s%s" % (attempt, r["outcome"],
                                         ": " + r["reasons"][0][0][:90] if r["reasons"] else "",
                                         " σ=%.4g±%.2g pb" % (r["sigma_pb"], r["err_pb"]) if r["sigma_pb"] else "")
        if r["outcome"] in ("ok", "timeout"):
            break
        err = r["error_text"] or ""
        rp = next((m for m in (rx.search(err) for rx in RUNPATCH) if m), None) if "runpatch" in adapt else None
        if rp:
            run_text = patch_run(run_text, rp.group(1), rp.group(2).strip())
            patches.append("runpatch:%s=%s" % (rp.group(1), rp.group(2).strip()))
            continue
        if ("gpmodel" in adapt and inp["ufomodel"] and not any(p.startswith("model:") for p in patches)
                and MODEL_ERROR.search(err)):
            staged = stage_model(inp["ufomodel"], models, gp_name)
            ufo_dir = models
            proc_text = use_gridpack_model(proc_text)
            patches.append("model:%s" % gp_name)
            continue
        if "strip" in adapt and not any(p.startswith("strip:") for p in patches):
            stripped, dropped = strip_named(proc_text, err)
            if dropped:
                proc_text = stripped
                patches.append("strip:%s" % ",".join(dropped))
                continue
        break
    return summary
