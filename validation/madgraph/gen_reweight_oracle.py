#!/usr/bin/env python3
"""Bank MadGraph's own reweighting of vibegraph's events, event by event.

For each row this script

  1. writes the process with MadGraph (`output standalone`, nothing compiled),
     for the param card and proc card MadGraph itself would put in a banner;
  2. integrates the process with vibegraph and generates an event file with
     `--reweight-card`, so each event carries vibegraph's `<rwgt>` weights;
  3. runs MadGraph's reweight module (`ReweightInterface`, standalone) on that
     same file, one hypothesis per call, with MadGraph's banner put in front and
     vibegraph's own reweighting markup taken out;
  4. prints, per hypothesis, how vibegraph's weights compare with MadGraph's on
     every event of the file — the full-pipeline comparison, informational —
     and banks the first `bank` events (flavours, momenta, couplings, `XWGTUP`)
     with MadGraph's weights and the param card it computed them with.

`vibegraph-lib/tests/reweight_mg_oracle.rs` reweights the banked events with the
library's own plan and compares weight by weight. The events are inputs there,
so the reference does not move when vibegraph's sampling does.

What MadGraph's reweight reads, and so what the comparison holds fixed:

  * the subprocess from the event's PDG codes and their order (`get_tag_and_order`),
    never `IDPRUP`; the beam-exchanged ordering is its own matrix element;
  * the momenta as written, boosted to the partonic centre of mass along z;
  * `alpha_s` from the event's `AQCDUP`, and `SCALUP^2` as the scale argument
    (which no tree-level SM or SMEFTsim coupling reads);
  * the event's own helicities by default (`helicity True`): the ratio of the one
    helicity configuration written. vibegraph's ratio is helicity-summed, so the
    card below sets `change helicity False`; both are unbiased for the
    reweighted cross section, but they differ event by event.

Each hypothesis runs in a fresh work area: MadGraph 3.7.1 cannot reuse a
compiled `rwgt_dir` (`setup_f2py_interface` reads an undefined `opts`), and
`launch` blocks after the first in one card each rewrite `events_out.lhe` from the
unmodified input, so only the last would survive.

Usage:
  python validation/madgraph/gen_reweight_oracle.py            # every row
  python validation/madgraph/gen_reweight_oracle.py pp_llj     # one row
  python validation/madgraph/gen_reweight_oracle.py --check pp_llj   # compare only, bank nothing

Prerequisites: the pinned MadGraph (`mg5_pinned.sh`), gfortran and numpy's f2py
(`pixi run -e madgraph`), the fetched PDF set for hadronic rows, and a built
`vibegraph` (`VIBEGRAPH_BIN`, default `target/release-debug/vibegraph`).
"""

import argparse
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
MG_ROOT = os.environ.get("VG_MG5_ROOT", os.path.join(ROOT, "research", "refs", "mg5amcnlo"))
REFERENCE = os.path.join(HERE, "reweight_mg_reference.json")
DEFAULT_WORK = os.path.join(HERE, "output", "reweight_oracle")


def ymt_scan():
    return [(f"ymt{int(v)}", [f"set yukawa 6 {v}"]) for v in (0.0, 100.0, 250.0, 350.0)]


def ymt_ymtau_grid():
    out = []
    for i, ymt in enumerate((0.0, 100.0, 250.0)):
        for j, ymtau in enumerate((0.0, 1.777, 5.0)):
            out.append((f"g{i}{j}", [f"set yukawa 6 {ymt}", f"set yukawa 15 {ymtau}"]))
    return out


def smeft_basis():
    """A basis grid in (ctWRe, cHt, cHWB): every point with at most two unit
    coefficients, the doubled axes, and one point with all three on."""
    points = [(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (1, 0, 1), (0, 1, 1),
              (2, 0, 0), (0, 2, 0), (0, 0, 2), (-1, 1, 1)]
    out = []
    for a, b, c in points:
        name = f"p{a}{b}{c}".replace("-", "m")
        # SMEFT block codes: 17 ctWRe, 31 cHt, 9 cHWB.
        out.append((name, [f"set SMEFT 17 {float(a)}", f"set SMEFT 31 {float(b)}",
                           f"set SMEFT 9 {0.5 * c}"]))
    return out


SMEFT = "SMEFTsim_topU3l_MwScheme_UFO"

# Hypotheses are written as LHA addresses, which both generators read the same way.
ROWS = {
    # Hadronic, both beam orderings, flavour groups of several members; every
    # hypothesis on the exact path. `aEWM1` and `MZ` reach the couplings through
    # chains of internal parameters (`aEWM1 -> aEW -> ee`, `MZ -> MW -> sw`).
    "pp_llj": dict(
        model="sm",
        process="generate p p > l+ l- j QCD=2 QED=2",
        run_card="validation/madgraph/output/pp_to_llj_fixed/Cards/run_card.dat",
        budget=("40000", "4"),
        nevents=2000,
        bank=200,
        couplings=None,
        hypotheses=[
            ("wz", ["set decay 23 2.6"]),
            ("aew", ["set sminputs 1 130.0"]),
            ("mz", ["set mass 23 91.0"]),
        ],
    ),
    # The polynomial path in one coupling: `a + b*ymt`, two amplitude classes.
    "ee_tth_ymt": dict(
        model="sm",
        process="generate e+ e- > t t~ h",
        run_card="validation/madgraph/onshell_ee_run_card.dat",
        budget=("20000", "4"),
        nevents=1000,
        bank=200,
        couplings="ymt",
        hypotheses=ymt_scan(),
    ),
    # The joint path in two couplings: `1, ymt, ymtau, ymt*ymtau`, a 3x3 grid.
    "tata_tth_grid": dict(
        model="sm",
        process="generate ta+ ta- > t t~ h",
        run_card="validation/madgraph/onshell_ee_run_card.dat",
        budget=("20000", "4"),
        nevents=1000,
        bank=200,
        couplings="ymt,ymtau",
        hypotheses=ymt_ymtau_grid(),
    ),
    # SMEFT with one insertion per diagram: a top dipole (a new Lorentz structure),
    # `cHt` (an NP coupling on the Standard Model's own Z t t~ structure) and `cHWB`
    # (the {mW, mZ, GF} input-scheme shift, which moves the gauge couplings of both
    # fermion lines), jointly: K = 1 + 3.
    "ee_ttx_smeft": dict(
        model=SMEFT,
        restrict="massless",
        process="generate e+ e- > t t~ NP<=1",
        run_card="validation/madgraph/output/ee_to_ttx_smeft/Cards/run_card.dat",
        budget=("20000", "4"),
        nevents=1000,
        bank=200,
        couplings="ctWRe,cHt,cHWB",
        hypotheses=smeft_basis(),
    ),
}


def run(cmd, **kw):
    r = subprocess.run(cmd, capture_output=True, text=True, stdin=subprocess.DEVNULL, **kw)
    if r.returncode != 0:
        sys.stderr.write(r.stdout[-4000:] + r.stderr[-4000:])
        raise SystemExit(f"failed: {' '.join(cmd[:3])} ...")
    return r


def vibegraph_model(row):
    """The `import model` argument vibegraph reads, and its `--ufo-dir` flags."""
    if row["model"] == "sm":
        return "sm", []
    return f"{row['model']}-{row['restrict']}", ["--ufo-dir", os.path.join(ROOT, "validation", "ufo")]


def madgraph_model(row, work):
    """The `import model` argument MadGraph reads. A vendored UFO is staged as a copy,
    because MadGraph writes its restricted-model cache into the model directory."""
    if row["model"] == "sm":
        return "sm"
    staged = os.path.join(work, "models", row["model"])
    if not os.path.isdir(staged):
        shutil.copytree(os.path.join(ROOT, "validation", "ufo", row["model"]), staged)
    return f"{staged}-{row['restrict']}"


def madgraph_cards(row, work):
    """MadGraph's own param card and proc card for the row's process."""
    out = os.path.join(work, "mgproc")
    if not os.path.isfile(os.path.join(out, "Cards", "param_card.dat")):
        script = os.path.join(work, "output.mg5")
        with open(script, "w") as f:
            f.write(f"import model {madgraph_model(row, work)}\n{row['process']}\n"
                    f"output standalone {out} -f\n")
        run(["bash", os.path.join(HERE, "mg5_pinned.sh"), script])
    return os.path.join(out, "Cards", "param_card.dat"), os.path.join(out, "Cards", "proc_card_mg5.dat")


def vibegraph_events(row, work, binary, seed):
    model, ufo = vibegraph_model(row)
    proc = os.path.join(work, "proc_card.dat")
    with open(proc, "w") as f:
        f.write(f"import model {model}\n{row['process']}\n")
    card = os.path.join(work, "reweight_card.dat")
    with open(card, "w") as f:
        for hid, lines in row["hypotheses"]:
            f.write(f"launch --rwgt_name={hid}\n" + "".join(f" {l}\n" for l in lines))
    run_card = os.path.join(ROOT, row["run_card"])
    pdf = ["--pdf-dir", os.path.join(ROOT, "validation", "pdf")]
    grid = os.path.join(work, "integrate")
    neval, niter = row["budget"]
    run([binary, "integrate", proc, *ufo, "--run-card", run_card, "--out", grid, *pdf,
         "--fixed-budget", "--neval", neval, "--niter", niter, "--seed", seed])
    lhe = os.path.join(work, "events.lhe")
    extra = ["--reweight-couplings", row["couplings"]] if row["couplings"] else []
    log = run([binary, "generate", os.path.join(grid, "grid.bin.zst"), proc, *ufo, "--run-card", run_card,
               *pdf, "--seed", seed, "--nevents", str(row["nevents"]), "--reweight-card", card,
               *extra, "-o", lhe, "--force"]).stderr
    for line in log.splitlines():
        if "reweight:" in line:
            print("   ", line.strip())
    return lhe


def madgraph_input(lhe, proc_card, run_card, param_card, path):
    text = open(lhe).read()
    text = re.sub(r"<initrwgt>.*?</initrwgt>\n", "", text, flags=re.S)
    text = re.sub(r"<rwgt>.*?</rwgt>\n", "", text, flags=re.S)
    banner = ("<MG5ProcCard>\n" + open(proc_card).read() + "\n</MG5ProcCard>\n"
              "<MGRunCard>\n" + open(run_card).read() + "\n</MGRunCard>\n"
              "<slha>\n" + open(param_card).read() + "\n</slha>\n")
    with open(path, "w") as f:
        f.write(text.replace("<header>\n", "<header>\n" + banner, 1))


def weights_of(path):
    return [
        {k: float(v) for k, v in re.findall(r"<wgt id='([^']+)'>\s*(\S+)\s*</wgt>", ev)}
        for ev in open(path).read().split("<event")[1:]
    ]


def madgraph_weights(row, work, lhe, proc_card, param_card):
    run_card = os.path.join(ROOT, row["run_card"])
    out = {}
    for hid, lines in row["hypotheses"]:
        here = os.path.join(work, "mg_" + hid)
        os.makedirs(here, exist_ok=True)
        inp = os.path.join(here, "events.lhe")
        madgraph_input(lhe, proc_card, run_card, param_card, inp)
        card = os.path.join(here, "reweight_card.dat")
        with open(card, "w") as f:
            f.write("change helicity False\n" f"launch --rwgt_name={hid}\n" + "".join(f" {l}\n" for l in lines))
        code = (
            "import sys, os\n"
            f"sys.path.insert(0, {MG_ROOT!r})\n"
            f"os.chdir({here!r})\n"
            "import madgraph.interface.reweight_interface as rwi\n"
            f"cmd = rwi.ReweightInterface({inp!r}, allow_madspin=True)\n"
            f"cmd.import_command_file({card!r})\n"
            "cmd.do_quit('')\n"
        )
        run([sys.executable, "-c", code])
        out[hid] = [w[hid] for w in weights_of(os.path.join(here, "events_out.lhe"))]
    return out


def events_of(lhe):
    """Each event's info line and particle lines, as vibegraph wrote them."""
    events = []
    for body in re.findall(r"<event>\n(.*?)</event>", open(lhe).read(), flags=re.S):
        lines = [l for l in body.splitlines() if l.strip() and not l.lstrip().startswith("<")]
        info = lines[0].split()
        nup = int(info[0])
        particles = [l.split() for l in lines[1 : 1 + nup]]
        vg = dict(re.findall(r"<wgt id='([^']+)'>\s*(\S+)\s*</wgt>", body))
        events.append(dict(
            xwgtup=float(info[2]),
            scalup=float(info[3]),
            aqcdup=float(info[5]),
            pdg=[int(p[0]) for p in particles],
            status=[int(p[1]) for p in particles],
            # px py pz E, as written.
            momenta=[[float(x) for x in p[6:10]] for p in particles],
            vibegraph={k: float(v) for k, v in vg.items()},
        ))
    return events


def param_entries(param_card):
    """`(block, code, value)` for every numeric entry of an SLHA card."""
    entries, block = [], None
    for raw in open(param_card):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        words = line.split()
        if words[0].lower() == "block":
            block = words[1].lower()
        elif words[0].lower() == "decay":
            entries.append(["decay", [int(words[1])], float(words[2])])
            block = None
        elif block is not None:
            entries.append([block, [int(w) for w in words[:-1]], float(words[-1])])
    return entries


def relative_difference(vg, mg):
    """|w_vg / w_mg - 1| for one event: two zeros agree, a zero on one side only
    or a non-finite weight is an infinite difference."""
    if not (math.isfinite(vg) and math.isfinite(mg)):
        return math.inf
    if mg == 0.0:
        return 0.0 if vg == 0.0 else math.inf
    return abs(vg / mg - 1.0)


def compare(events, mg, hypotheses):
    """Per hypothesis, over every event: events compared, the largest relative
    difference, and how many sit beyond twice the files' eight-digit printing."""
    out = {}
    for hid, _ in hypotheses:
        rel = [relative_difference(e["vibegraph"][hid], m) for e, m in zip(events, mg[hid])]
        out[hid] = dict(events=len(rel), largest=max(rel, default=0.0), beyond_2e7=sum(r > 2e-7 for r in rel))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("rows", nargs="*", default=sorted(ROWS))
    ap.add_argument("--work", default=DEFAULT_WORK)
    ap.add_argument("--seed", default="20261005")
    ap.add_argument("--check", action="store_true", help="compare, bank nothing")
    args = ap.parse_args()
    # MadGraph runs from inside each hypothesis's work area, so its paths must
    # not depend on the directory this was started from.
    args.work = os.path.abspath(args.work)
    binary = os.environ.get("VIBEGRAPH_BIN", os.path.join(ROOT, "target", "release-debug", "vibegraph"))
    mg_version = open(os.path.join(MG_ROOT, "VERSION")).read().split("\n")[0].split("=")[1].strip()

    reference = json.load(open(REFERENCE)) if os.path.isfile(REFERENCE) else {}
    reference.setdefault("rows", {})
    for key in args.rows:
        row = ROWS[key]
        work = os.path.join(args.work, key)
        os.makedirs(work, exist_ok=True)
        print(f"== {key}: {row['process']}")
        param_card, proc_card = madgraph_cards(row, work)
        lhe = vibegraph_events(row, work, binary, args.seed)
        mg = madgraph_weights(row, work, lhe, proc_card, param_card)
        events = events_of(lhe)
        assert all(len(v) == len(events) for v in mg.values()), "MadGraph dropped events"
        summary = compare(events, mg, row["hypotheses"])
        for hid, s in summary.items():
            print(f"   {hid:>8}: {s['events']} events, largest |w_vg/w_mg - 1| {s['largest']:.2e}, "
                  f"{s['beyond_2e7']} beyond 2e-7")
        if args.check:
            continue
        banked = events[: row["bank"]]
        for k, e in enumerate(banked):
            # vibegraph's own weights are summarised in `full_file`, not banked.
            del e["vibegraph"]
            e["madgraph"] = {hid: mg[hid][k] for hid, _ in row["hypotheses"]}
        reference["rows"][key] = dict(
            model=row["model"],
            restrict=row.get("restrict"),
            process=row["process"],
            run_card=row["run_card"],
            couplings=row["couplings"].split(",") if row["couplings"] else None,
            hypotheses=[dict(id=h, lines=l) for h, l in row["hypotheses"]],
            param_card=param_entries(param_card),
            full_file=summary,
            events=banked,
        )
    if args.check:
        return
    reference["madgraph_version"] = mg_version
    reference["generator"] = "validation/madgraph/gen_reweight_oracle.py"
    with open(REFERENCE, "w") as f:
        json.dump(reference, f, indent=None, separators=(",", ":"))
        f.write("\n")
    print(f"wrote {os.path.relpath(REFERENCE, ROOT)}")


if __name__ == "__main__":
    main()
