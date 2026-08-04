#!/usr/bin/env python3
"""Wall-clock timing of MadGraph's own stages, per process, on this machine.

    pixi run -e madgraph python validation/madgraph/time_stages.py \
        --out <scratch-dir> ee_to_mumu gg_to_gg ...

Each named process is regenerated from scratch into ``--out`` and every line
MadGraph prints is stamped with the seconds elapsed since that process started,
so the boundaries between its stages are read off the transcript rather than
guessed.  The stages, and the marker each one is delimited by:

    startup    process start -> "Checking for minimal orders" / "Trying process"
               (the Python interpreter and MadGraph's own import)
    generate   that -> "N processes with M diagrams generated in X s"
    output     "initialize a new directory" -> "Output to directory ... done."
    compile    "compile directory" -> "Running Survey"
    integrate  "Running Survey" -> "finish refine"      (survey and refine both)
    events     "Combining Events" -> "End Parton"

``generate`` is also reported as MadGraph's own self-timing, which is the number
printed on that line; the two differ by whatever the transcript charges to the
line itself and are a cross-check on the parse.

**The output directory is a scratch area, never the reference bank.**  Nothing
here reads or writes ``validation/madgraph/output``: these runs exist to be
timed, their physics is thrown away, and a regenerated process directory must
not be mistaken for a banked one.  ``--out`` is required for that reason.

The result is ``<out>/timings.json``, rewritten after every process so an
interrupted pass still leaves a readable record of what it got through.  It
carries the same machine identity the validation report's ``host.json`` does,
plus the Fortran compiler and flags MadGraph built with — a duration without
those is not a measurement of anything.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
SCRIPTS = HERE / "scripts"

# One (stage, opening marker, closing marker) per span read off the transcript.
# The markers are substrings of lines MadGraph prints; a stage whose markers do
# not both appear is recorded as null rather than as zero.
SPANS = [
    ("generate", r"Checking for minimal orders|Trying process", r"processes with .* diagrams generated in"),
    ("output", r"initialize a new directory", r"Output to directory .* done\."),
    ("compile", r"compile directory", r"Running Survey"),
    ("integrate", r"Running Survey", r"finish refine"),
    ("events", r"Combining Events", r"End Parton"),
]

SELF_TIMED = re.compile(r"processes with .* diagrams generated in\s+([0-9.]+)\s*s")
ALOHA_TIMED = re.compile(r"aloha creates .* in\s+([0-9.]+)\s*s")


def strip_ansi(line):
    return re.sub(r"\x1b\[[0-9;]*m", "", line)


def run_timed(argv, log_path, env=None):
    """Run a command, stamping each output line with seconds since it started.

    Returns ``(exit_status, elapsed_seconds, [(t, line), ...])``.
    """
    t0 = time.monotonic()
    stamped = []
    with open(log_path, "w") as log:
        proc = subprocess.Popen(
            argv,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
            env=env,
        )
        for raw in proc.stdout:
            t = time.monotonic() - t0
            line = strip_ansi(raw.rstrip("\n"))
            stamped.append((t, line))
            log.write(f"{t:10.3f}  {line}\n")
            log.flush()
        status = proc.wait()
    return status, time.monotonic() - t0, stamped


def spans_of(stamped):
    """The wall time of each stage, and MadGraph's own self-timings."""
    out = {}
    for name, opener, closer in SPANS:
        start = next((t for t, l in stamped if re.search(opener, l)), None)
        end = None
        if start is not None:
            end = next((t for t, l in stamped if t >= start and re.search(closer, l)), None)
        out[name] = None if (start is None or end is None) else round(end - start, 3)
    # Everything before the first stage marker is the Python interpreter and
    # MadGraph's own import, which is real cost but belongs to no physics stage.
    out["interpreter_startup"] = next(
        (round(t, 3) for t, l in stamped if re.search(SPANS[0][1], l)), None
    )
    self_gen = next((float(m.group(1)) for _, l in stamped if (m := SELF_TIMED.search(l))), None)
    self_aloha = next((float(m.group(1)) for _, l in stamped if (m := ALOHA_TIMED.search(l))), None)
    out["generate_self_reported"] = self_gen
    out["aloha_self_reported"] = self_aloha
    return out


def sysctl(key):
    try:
        v = subprocess.run(["sysctl", "-n", key], capture_output=True, text=True)
    except FileNotFoundError:
        return None
    v = v.stdout.strip()
    return v or None


def first_line(argv):
    try:
        r = subprocess.run(argv, capture_output=True, text=True)
    except FileNotFoundError:
        return None
    return r.stdout.strip().splitlines()[0] if r.stdout.strip() else None


def fortran_flags(proc_dir):
    """The compiler and flags the generated directory builds its Fortran with."""
    opts = proc_dir / "Source" / "make_opts"
    if not opts.is_file():
        return None
    keep = ("FC=", "FFLAGS", "OPTFLAG", "MACFLAG", "STDLIB", "LDFLAGS", "CXX=")
    return [
        l.strip()
        for l in opts.read_text().splitlines()
        if l.strip().startswith(keep)
    ]


def as_int(v):
    try:
        return int(v)
    except (TypeError, ValueError):
        return None


def host_block():
    return {
        "captured": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "cpu": {
            "arch": os.uname().machine,
            "model": sysctl("machdep.cpu.brand_string"),
            "logical_cpus": as_int(sysctl("hw.logicalcpu")),
            "physical_cpus": as_int(sysctl("hw.physicalcpu")),
            "performance_logical_cpus": as_int(sysctl("hw.perflevel0.logicalcpu")),
            "efficiency_logical_cpus": as_int(sysctl("hw.perflevel1.logicalcpu")),
            "frequency_hz": as_int(sysctl("hw.cpufrequency_max")),
            "frequency_note": "null where the OS exposes no clock (Apple Silicon: "
            "`hw.cpufrequency`/`hw.cpufrequency_max` are empty)",
            "memory_bytes": as_int(sysctl("hw.memsize")),
        },
        "scheduling": {
            "affinity": "none — MadGraph is run as it is in production, with no core "
            "pinning; madevent forks its own subprocess jobs and the OS places them",
            "cpu_count_seen_by_python": os.cpu_count(),
        },
        "os": {
            "family": sys.platform,
            "kernel": f"{os.uname().sysname} {os.uname().release}",
            "release": first_line(["sw_vers", "-productVersion"]),
        },
        "toolchain": {
            "python": sys.version.split()[0],
            "gfortran": first_line(["gfortran", "--version"]),
            "gcc": first_line(["gcc", "--version"]),
            "madgraph": (ROOT / "research/refs/mg5amcnlo/VERSION").read_text().strip()
            if (ROOT / "research/refs/mg5amcnlo/VERSION").is_file()
            else None,
        },
    }


def driver_for(process, out_dir, launch):
    """The .mg5 script to run, with its output redirected into the scratch area.

    ``launch = False`` truncates the script before its ``launch`` block, which is
    how the generate/output stages are reached without paying for the run.
    """
    src = (SCRIPTS / f"{process}.mg5").read_text()
    lines = []
    for line in src.splitlines():
        if re.match(r"^\s*output\s+\S+", line):
            line = re.sub(r"^(\s*output\s+)\S+", rf"\g<1>{out_dir / process}", line)
        if not launch and re.match(r"^\s*launch\b", line):
            break
        lines.append(line)
    return "\n".join(lines) + "\n"


def time_script_process(process, out_dir, logs):
    """A process whose whole life is one .mg5 script: generate, output and launch."""
    driver = Path(tempfile.mkstemp(prefix=f"vg_time_{process}_", suffix=".mg5")[1])
    driver.write_text(driver_for(process, out_dir, launch=True))
    env = dict(os.environ, LDFLAGS=os.environ.get("LDFLAGS", "") + " -lc++")
    status, elapsed, stamped = run_timed(
        ["bash", str(HERE / "mg5_pinned.sh"), str(driver)],
        logs / f"{process}.timed.log",
        env=env,
    )
    driver.unlink(missing_ok=True)
    record = {"process": process, "driver": "mg5 script", "status": status,
              "total_s": round(elapsed, 3)}
    record["stages"] = spans_of(stamped)
    record["fortran"] = fortran_flags(out_dir / process)
    return record


# The Drell-Yan rows have no .mg5 script: the reference layer builds them from a
# bare generate/output and a committed run card, keyed by run name.
DY13_CARDS = {
    "dy13_default": "dy13_default_run_card.dat",
    "dy13_mmll_60_120": "dy13_mmll_run_card.dat",
}


def time_dy13(name, out_dir, logs):
    """`p p > e+ e-`, which the reference layer drives in two commands.

    The process directory comes from a bare generate/output script and the run
    from `bin/generate_events` over the committed run card, so the two are timed
    separately and their stages joined into one record.
    """
    proc_dir = out_dir / name
    driver = Path(tempfile.mkstemp(prefix="vg_time_dy13_", suffix=".mg5")[1])
    driver.write_text(f"generate p p > e+ e-\noutput {proc_dir} -nojpeg\n")
    env = dict(os.environ, LDFLAGS=os.environ.get("LDFLAGS", "") + " -lc++")
    env["LHAPDF_DATA_PATH"] = os.pathsep.join(
        p for p in [str(ROOT / "validation/pdf"), os.environ.get("LHAPDF_DATA_PATH", "")] if p
    )
    s1, e1, gen_stamped = run_timed(
        ["bash", str(HERE / "mg5_pinned.sh"), str(driver)],
        logs / f"{name}.generate.timed.log",
        env=env,
    )
    driver.unlink(missing_ok=True)
    if s1 != 0:
        return {"process": name, "driver": "generate_events", "status": s1,
                "total_s": round(e1, 3), "stages": spans_of(gen_stamped), "fortran": None}

    shutil.copy(HERE / DY13_CARDS[name], proc_dir / "Cards/run_card.dat")
    cfg = proc_dir / "Cards/me5_configuration.txt"
    if cfg.is_file():
        kept = [
            l for l in cfg.read_text().splitlines()
            if not re.match(r"\s*#?\s*(automatic_html_opening|notification_center)\s*=", l)
        ]
        cfg.write_text("\n".join(kept + ["automatic_html_opening = False",
                                         "notification_center = False"]) + "\n")
    s2, e2, run_stamped = run_timed(
        [str(proc_dir / "bin/generate_events"), "-f", f"run_{name.removeprefix('dy13_')}"],
        logs / f"{name}.run.timed.log",
        env=env,
    )
    stages = spans_of(gen_stamped)
    stages.update({k: v for k, v in spans_of(run_stamped).items()
                   if k in ("compile", "integrate", "events")})
    return {"process": name, "driver": "generate_events",
            "status": s1 or s2, "total_s": round(e1 + e2, 3),
            "stages": stages, "fortran": fortran_flags(proc_dir)}


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", required=True,
                    help="scratch directory the timed runs are generated into; "
                         "must not be the reference bank")
    ap.add_argument("processes", nargs="+")
    args = ap.parse_args()

    out_dir = Path(args.out).resolve()
    bank = (HERE / "output").resolve()
    if out_dir == bank or bank in out_dir.parents:
        sys.exit(f"refusing to generate into the reference bank: {out_dir}")
    logs = out_dir / "logs"
    logs.mkdir(parents=True, exist_ok=True)

    timings = out_dir / "timings.json"
    # A pass may be taken in several sittings — the checkpoint after the
    # representative rows is one — so an existing record is extended rather than
    # replaced, and `pass_wall_s` accumulates across the sittings it took.
    doc = json.loads(timings.read_text()) if timings.is_file() else {
        "schema": 1,
        "_comment": "Wall-clock seconds per MadGraph stage, per process, measured by "
                    "validation/madgraph/time_stages.py on the machine in `host`. A "
                    "measurement about that machine, not a reference: absolute times do "
                    "not carry across hosts, nothing gates on these numbers, and they "
                    "are not part of the refdata bundle. The runs behind them were "
                    "generated into a scratch directory and thrown away.",
        "host": host_block(), "runs": [], "pass_wall_s": 0.0
    }
    already = round(doc.get("pass_wall_s", 0.0), 3)
    pass_start = time.monotonic()
    for process in args.processes:
        print(f">>> timing {process} ...", flush=True)
        if process in DY13_CARDS:
            record = time_dy13(process, out_dir, logs)
        else:
            record = time_script_process(process, out_dir, logs)
        doc["runs"] = [r for r in doc["runs"] if r["process"] != process] + [record]
        doc["pass_wall_s"] = round(already + time.monotonic() - pass_start, 3)
        timings.write_text(json.dumps(doc, indent=2) + "\n")
        st = record["stages"]
        print(f"    {process}: total {record['total_s']:.1f} s  "
              f"generate {st['generate']}  output {st['output']}  compile {st['compile']}  "
              f"integrate {st['integrate']}  events {st['events']}  (exit {record['status']})",
              flush=True)
    print(f"wrote {timings}")


if __name__ == "__main__":
    main()
