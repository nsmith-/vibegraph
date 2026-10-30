#!/usr/bin/env python3
"""Per-channel MadEvent results from an extracted LO gridpack, and how they sum.

    python3 -I mg_channels.py <gridpack dir> [--csv channels.csv] [--run pilotrun]

<gridpack dir> holds `process/madevent/SubProcesses`. For each P directory the
channels are the `symfact.dat` entries with a positive multiplicity, which is how
MadGraph's own `sum_html.collect_result` picks them; a channel directory is `G<name>`.

Each channel has one `<run>_results.dat` per MadEvent run. In a CMS gridpack,
`pilotrun_results.dat` is the gridpack survey (the cross section the generation
log reports) and `results.dat` is the gridpack's own test run, a different and
smaller integration. The first line's columns, as `sum_html.OneResult.read_results`
names them and `dsample.f` writes them:

    axsec    |σ|, the weighted mean of the last iterations (pb)
    xerru    its uncorrelated error (pb); inflated by √(χ²/dof) when χ²/dof > 1
    xerrc    a correlated error (pb), summed linearly
    nevents  points evaluated over all iterations
    nw       points with nonzero weight
    maxit    iterations run
    nunwgt   unweighted events written
    luminosity  nunwgt / axsec (pb⁻¹)
    wgt      the event weight used to unweight
    xsec     σ with sign, combined like axsec (pb)
    maxwgt, th_maxwgt, th_nunwgt   unweighting targets (when present)

and each later line is one combined iteration: `it, |σ|, error, eff, wmax, σ`,
with `eff = error·√nw/|σ|` (the per-point relative spread) and `wmax` the largest
weight scaled by the nonzero fraction. The channel's χ²/dof over those iterations
is printed only in `<run>_log.txt`, under "Results Last N iters".

A P directory's σ is the sum of its channels' xsec, and its error the quadrature
sum of xerru plus the linear sum of xerrc. The total is the sum over P directories,
which `SubProcesses/<run>_results.dat` records.
"""

import argparse
import csv
import glob
import math
import os
import re
import sys

FIRST = ("axsec", "xerru", "xerrc", "nevents", "nw", "maxit", "nunwgt", "luminosity", "wgt", "xsec",
         "maxwgt", "th_maxwgt", "th_nunwgt")
LAST_ITERS = re.compile(r"Results Last\s+(\d+) iters:.*?Chi\*\*2 per DoF\.\s*=\s*([0-9.eE+-]+|NaN)", re.S)
ACCUMULATED = re.compile(r"Accumulated results:.*?Chi\*\*2 per DoF\.\s*=\s*([0-9.eE+-]+|NaN)", re.S)
ITERATION = re.compile(r"^\s*Iteration\s+(\d+)\s+Mean:\s*(\S+)\s+Abs mean:\s*(\S+)\s+Fluctuation:\s*(\S+)", re.M)


def read_results(path):
    """-> (first-line fields, [(it, |σ|, err, eff, wmax, σ)]) or None if absent."""
    if not os.path.exists(path):
        return None
    lines = [l for l in open(path).read().split("<", 1)[0].splitlines() if l.strip()]
    if not lines:
        return None
    head = dict(zip(FIRST, (float(x) for x in lines[0].split())))
    iters = [tuple(float(x) for x in l.split()) for l in lines[1:] if len(l.split()) == 6]
    return head, iters


def read_log(path):
    if not os.path.exists(path):
        return {}
    text = open(path, errors="replace").read()
    out = {}
    m = LAST_ITERS.findall(text)
    if m:
        out["last_n"], out["chi2_last"] = int(m[-1][0]), float(m[-1][1])
    m = ACCUMULATED.findall(text)
    if m:
        out["chi2_all"] = float(m[-1])
    its = ITERATION.findall(text)
    if its:
        out["n_iter_log"] = len(its)
        out["sigma_iter1"] = float(its[0][2])
    return out


def channels(sub, run):
    rows = []
    for pdir in sorted(glob.glob(os.path.join(sub, "P*"))):
        if not os.path.isdir(pdir):
            continue
        listed = {}
        for line in open(os.path.join(pdir, "symfact.dat")):
            name, mfactor = line.split()
            if float(mfactor) > 0:
                listed["G" + name] = int(float(mfactor))
        present = {os.path.basename(d) for d in glob.glob(os.path.join(pdir, "G*")) if os.path.isdir(d)}
        for g in sorted(listed.keys() | present, key=lambda s: [float(x) for x in s[1:].split(".")]):
            row = {"P": os.path.basename(pdir), "G": g, "mfactor": listed.get(g, ""), "listed": g in listed,
                   "present": g in present}
            for tag, fname in (("survey", run + "_results.dat"), ("test", "results.dat")):
                r = read_results(os.path.join(pdir, g, fname))
                if r is None:
                    continue
                head, iters = r
                for k, v in head.items():
                    row["%s_%s" % (tag, k)] = v
                row["%s_iters" % tag] = ";".join("%d:%.5g±%.3g" % (it[0], it[1], it[2]) for it in iters)
            for k, v in read_log(os.path.join(pdir, g, run + "_log.txt")).items():
                row["survey_" + k] = v
            rows.append(row)
    return rows


def total(sub, run):
    r = read_results(os.path.join(sub, run + "_results.dat"))
    return r[0] if r else None


def summarize(rows, tot, out):
    p = lambda *a: print(*a, file=out)
    for tag in ("survey", "test"):
        p("## %s (%s)" % (tag, "<run>_results.dat" if tag == "survey" else "results.dat"))
        grand, grand_e2 = 0.0, 0.0
        for P in sorted({r["P"] for r in rows}):
            rs = [r for r in rows if r["P"] == P and r["listed"] and ("%s_xsec" % tag) in r]
            missing = [r["G"] for r in rows if r["P"] == P and r["listed"] and ("%s_xsec" % tag) not in r]
            zero = [r["G"] for r in rs if r["%s_xsec" % tag] == 0]
            s = sum(r["%s_xsec" % tag] for r in rs)
            e2 = sum(r["%s_xerru" % tag] ** 2 for r in rs)
            ec = sum(r["%s_xerrc" % tag] for r in rs)
            e = math.sqrt(e2 + ec ** 2)
            grand += s
            grand_e2 += e ** 2
            p("%-18s %3d channels  σ = %.5g ± %.3g pb  (xerrc sum %.3g)  missing %s  zero %s" % (
                P, len(rs), s, e, ec, missing or "none", zero or "none"))
        p("%-18s sum = %.5g ± %.3g pb" % ("all P", grand, math.sqrt(grand_e2)))
        if tag == "survey" and tot:
            p("%-18s %.5g ± %.3g pb  (SubProcesses/<run>_results.dat)" % ("recorded total", tot["xsec"], tot["xerru"]))
        p("")
    sv = [r for r in rows if "survey_xsec" in r]
    for key, label in (("survey_xsec", "σ"), ("survey_xerru", "error")):
        p("## top 10 channels by survey %s" % label)
        for r in sorted(sv, key=lambda r: -r[key])[:10]:
            p("  %-16s %-7s σ = %.4g ± %.3g  χ²/dof(last %s) = %s  iterations %s  σ(it 1) = %s" % (
                r["P"], r["G"], r["survey_xsec"], r["survey_xerru"], r.get("survey_last_n", "?"),
                r.get("survey_chi2_last", "?"), int(r["survey_maxit"]), r.get("survey_sigma_iter1", "?")))
        p("")
    p("## channels whose last-iterations χ²/dof > 1 (MadEvent then inflates xerru by √(χ²/dof))")
    hot = sorted((r for r in sv if r.get("survey_chi2_last", 0) > 1), key=lambda r: -r["survey_chi2_last"])
    for r in hot:
        p("  %-16s %-7s χ²/dof = %.3g  σ = %.4g ± %.3g  (%.2g%% of P total)  iters %s" % (
            r["P"], r["G"], r["survey_chi2_last"], r["survey_xsec"], r["survey_xerru"],
            100 * r["survey_xsec"] / max(1e-300, sum(x["survey_xsec"] for x in sv if x["P"] == r["P"])),
            r["survey_iters"]))
    if not hot:
        p("  none")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("gridpack")
    ap.add_argument("--run", default="pilotrun")
    ap.add_argument("--csv")
    args = ap.parse_args(argv)
    sub = os.path.join(args.gridpack, "process", "madevent", "SubProcesses")
    rows = channels(sub, args.run)
    if args.csv:
        keys = []
        for r in rows:
            keys += [k for k in r if k not in keys]
        with open(args.csv, "w", newline="") as fh:
            w = csv.DictWriter(fh, keys, lineterminator="\n")
            w.writeheader()
            w.writerows(rows)
    summarize(rows, total(sub, args.run), sys.stdout)


if __name__ == "__main__":
    main()
