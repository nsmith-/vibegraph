"""Write mlm_sigma_reference.json from the per-seed MadEvent runs of
gen_mlm_references.sh.

Usage: python3 write_mlm_sigma_reference.py ROWS_FILE OUT_JSON VERSION_FILE HERE NB_CORE

ROWS_FILE has one line per finished run, `row|iseed|sigma|err|wall|kind|rundir`,
where sigma and err are the run's results.dat value, wall the seconds of its
generate_events call, and kind one of

    samples  the samples-grade run, output/<row>/Events/run_01: the first run of
             a freshly generated directory, banked
    fresh    a run that is the only run of its own freshly generated directory;
             its event file is banked under output/<row>/Events/run_s<iseed>
    sigma    a further seed in the row's one shared directory, work/mlm/<row>,
             not banked

A row with fresh runs is a row whose reference is read from independent
directories: its `runs` are the samples run and the fresh runs, each from a
directory of its own, and the shared-directory seeds go to
`shared_directory_runs`, which no gate reads. The seeds of one directory
inherit its grids and are not independent draws (their seed chi2/dof about
their mean reads 0.3-0.8 against MadEvent's quoted errors), so they are kept
as a record and nothing more. Every other row keeps the layout it always had:
`runs` is the samples run plus the shared seeds.

Rows not in ROWS_FILE keep their committed entries.
"""

import gzip
import json
import os
import re
import sys

rows_path, out_path, version_path, here, nb_core = sys.argv[1:6]


def init_processes(rundir):
    # The <init> block's per-process lines: XSECUP XERRUP XMAXUP LPRUP. On a
    # card with `@N` process tags LPRUP is N, so this is sigma per multiplicity.
    with gzip.open(os.path.join(rundir, "unweighted_events.lhe.gz"), "rt") as f:
        text = f.read(200000)
    block = text[text.index("<init>") + 6 : text.index("</init>")].strip().splitlines()
    nprup = int(block[0].split()[-1])
    out = {}
    for line in block[1 : 1 + nprup]:
        xsec, xerr, _xmax, lprup = line.split()[:4]
        out[lprup] = {"sigma_pb": float(xsec), "err_pb": float(xerr)}
    return out


old = json.load(open(out_path))["rows"] if os.path.exists(out_path) else {}
rows = {}
for line in open(rows_path):
    name, seed, sigma, err, wall, kind, rundir = line.strip().split("|")
    if kind not in ("samples", "fresh", "sigma"):
        sys.exit("unknown run kind %r for %s seed %s" % (kind, name, seed))
    if name not in rows:
        script = open(os.path.join(here, "scripts", name + ".mg5")).read()
        proc = [ln.strip() for ln in script.splitlines()
                if re.match(r"\s*(generate|add process)\b", ln)]
        card = open(os.path.join(here, name + "_run_card.dat")).read()
        nev = re.search(r"^\s*(\d+)\s*=\s*nevents\b", card, re.M).group(1)
        rows[name] = {"process": proc, "script": "scripts/%s.mg5" % name,
                      "run_card": "%s_run_card.dat" % name, "nevents": int(nev),
                      "runs": [], "fresh": [], "shared": []}
    run = {"iseed": int(seed), "sigma_pb": float(sigma), "err_pb": float(err),
           "wall_s": int(wall), "samples": kind == "samples",
           "by_lprup": init_processes(rundir)}
    if kind == "samples":
        run["events"] = "output/%s/Events/run_01/unweighted_events.lhe.gz" % name
        rows[name]["runs"].append(run)
    elif kind == "fresh":
        run["events"] = "output/%s/Events/run_s%s/unweighted_events.lhe.gz" % (name, seed)
        rows[name]["fresh"].append(run)
    else:
        rows[name]["shared"].append(run)

for name, row in rows.items():
    fresh, shared = row.pop("fresh"), row.pop("shared")
    if fresh:
        row["independent_directories"] = True
        row["runs"].extend(fresh)
        row["shared_directory_runs"] = shared
    else:
        row["independent_directories"] = False
        row["runs"].extend(shared)

version = re.search(r"version\s*=\s*(\S+)", open(version_path).read()).group(1)
out = {
    "_comment": "MadEvent cross sections (pb) for MLM-matched LO generation (ickkw = 1 / "
                "xqcut), one run per iseed, each the run card's full unweighted event count. "
                "The run marked samples is output/<row>/Events/run_01, the first run of a "
                "freshly generated directory, which the reference bundle carries and the "
                "instrumented replay reproduces. A row with independent_directories = true "
                "reads its reference from runs that each come from a directory of their own: "
                "the samples run and one freshly generated directory per further seed, whose "
                "event files are banked as output/<row>/Events/run_s<iseed> (the field events "
                "names each banked file). Its shared_directory_runs are further seeds run one "
                "after another in one directory (work/mlm/<row>), which inherit each other's "
                "grids; they are kept as a record and are not the reference. On a row with "
                "independent_directories = false every seed after the samples run is such a "
                "shared-directory seed, in runs. Gates read runs under the seed policy of "
                "validation/manifest.toml: the inverse-variance mean with an error of "
                "max(quoted, spread/sqrt(n)). wall_s is the whole generate_events call on %s "
                "cores, generation of the directory excluded, survey, refine and compilation "
                "included. by_lprup is the run's own <init> block per process (LPRUP = the @N "
                "tag where the card has them). Generated by "
                "validation/madgraph/gen_mlm_references.sh." % nb_core,
    "mg_version": version,
    "rows": dict(old, **rows),
}
with open(out_path, "w") as f:
    json.dump(out, f, indent=2)
    f.write("\n")
print("wrote", out_path)
