#!/usr/bin/env python3
"""Generate MadGraph amplitude reference CSVs on a fixed kinematic grid.

For each process `validation/manifest.toml` registers an `mg_amplitude` table
for, evaluates MadGraph's compiled MATRIX1 Fortran subroutine (built by
build_amplitude.sh against wrappers/generic.f) over RAMBO-sampled phase-space
points at one or more collision energies, and writes
  output/PROCESSNAME_amplitude.csv

Also times each process's MATRIX1 over a dedicated `profile_npoints` batch and
writes the table to output/mg_timings.json, alongside this machine's identity
({schema, host, processes: {name: {process_str, n_evals, total_ms,
ns_per_eval}}}) — see `host_info.py`.

CSV schema (momenta-based; the only schema the Rust test reads):
  # process: PROCESS_STRING     <- parsed by vibegraph's process grammar
  # n_ext: N
  m2_summed,E0,px0,py0,pz0,...  <- one row per phase-space point

m2_summed = sum_hel sum_color |M|^2 (NOT divided by IDEN / averaging factor),
matching vibegraph's AmplitudeEvaluator.eval_m2() up to the process's overall
color factor (applied on the Rust side).

External masses are read from each process's own Cards/param_card.dat, so the
generated momenta are on-shell for exactly the masses both MadGraph and
vibegraph evaluate with (bit-for-bit comparison).

The manifest is the single source of truth: a process's generation parameters
(the concrete process card, external PDGs, RAMBO grid, seed, timing batch
size) live once, in its `[[process]] mg_amplitude` table, not in a second
Python-side copy. One compiled module can serve more than one manifest row's
`amplitudes` gate (`pp_to_ll_qcd0` generates the table `uux_to_mumu` also
reads) — such rows carry no `mg_amplitude` table of their own.

Usage:
  python validation/madgraph/gen_amplitude.py
  python validation/madgraph/gen_amplitude.py ee_to_mumu gg_to_gg   # only these
      rows; the timing table is merged into the work area's rather than
      replaced, so a partial run does not drop the rows it did not measure.
  python validation/madgraph/gen_amplitude.py --dump-processes   # print the
      resolved registry as JSON and exit; no MadGraph module is imported.
      This is the migration oracle: run before and after any change to how
      the registry is built and diff the output.
  pixi run -e madgraph generate-amplitude

Prerequisites:
  pixi run -e madgraph build-amplitude
"""

import importlib
import json
import math
import os
import re
import sys
import time
import tomllib
from dataclasses import dataclass, field

import numpy as np

from host_info import host_block

HERE = os.path.dirname(os.path.abspath(__file__))
OUTPUT_DIR = os.path.join(HERE, "output")
MANIFEST_PATH = os.path.join(HERE, "..", "manifest.toml")

# The compiled mg_*.so matrix elements are build products, so they live in the
# work area rather than beside the sources that import them.
sys.path.insert(0, os.path.join(OUTPUT_DIR, "f2py"))


@dataclass
class Process:
    """One registered validation process.

    `process_str` doubles as the vibegraph proc-card line (particle names in
    UFO casing, e.g. 'W+', 'Z', 'H') and must describe the same process the
    .mg5 script generated.  `pdgs_in`/`pdgs_out` follow the process-string leg
    order; they select each leg's mass from the process's param_card.dat.
    """

    name: str  # output dir, module suffix (mg_NAME), and CSV prefix
    process_str: str
    pdgs_in: tuple[int, ...]
    pdgs_out: tuple[int, ...]
    sqrt_s_list: tuple[float, ...]
    npoints: int  # phase-space points per sqrt_s value
    seed: int
    # Batch size for the MATRIX1 timing measurement (independent of the
    # validation rows): large enough that the per-call wrapper overhead
    # amortizes, scaled down for the expensive high-multiplicity processes.
    profile_npoints: int = 10_000
    # The UFO model directory the row's .mg5 script imports, and the restrict
    # card it imports it with. Absent for a row generated against MadGraph's
    # default `sm`, which is what the manifest saying nothing means.
    model: str | None = None
    restrict: str | None = None


def load_processes() -> list["Process"]:
    """The `mg_amplitude`-bearing rows of `validation/manifest.toml`, as `Process` records.

    Iterated in the manifest's own row order (single-channel rows in their
    written order, then the one multi-channel row that owns a generation,
    `pp_to_ll_qcd0`) — nothing downstream depends on registry order, since
    every process writes its own independent CSV and the timing table is
    keyed by name.
    """
    with open(MANIFEST_PATH, "rb") as f:
        manifest = tomllib.load(f)
    procs = []
    for entry in manifest["process"]:
        mg = entry.get("mg_amplitude")
        if mg is None:
            continue
        procs.append(
            Process(
                name=entry["key"],
                process_str=mg["process"],
                pdgs_in=tuple(mg["pdgs_in"]),
                pdgs_out=tuple(mg["pdgs_out"]),
                sqrt_s_list=tuple(mg["sqrt_s_list"]),
                npoints=mg["npoints"],
                seed=mg["seed"],
                profile_npoints=mg.get("profile_npoints", 10_000),
                model=entry.get("model"),
                restrict=entry.get("restrict"),
            )
        )
    return procs


PROCESSES = load_processes()


def matrix_file(name: str) -> str:
    """The process's single `SubProcesses/P*/matrix1_orig.f`."""
    sub = os.path.join(OUTPUT_DIR, name, "SubProcesses")
    dirs = [d for d in sorted(os.listdir(sub)) if d.startswith("P")]
    assert len(dirs) == 1, f"{name}: expected one P* subprocess dir, got {dirs}"
    return os.path.join(sub, dirs[0], "matrix1_orig.f")


def nhel_table(name: str, n_ext: int) -> list[list[int]]:
    """MadGraph's own NHEL table for a process, in its own row order.

    Read from the generated `DATA (NHEL(I,N),I=1,n_ext) / ... /` block rather
    than enumerated as a product, so the helicity set is MadGraph's.
    """
    with open(matrix_file(name)) as f:
        src = re.sub(r"\n     [^ ]", "", f.read())
    combos = []
    for row in re.findall(r"DATA \(NHEL\(I,\s*\d+\),I=1,\s*\d+\) /([^/]*)/", src):
        vals = [int(v) for v in row.replace(" ", "").split(",")]
        assert len(vals) == n_ext, f"{name}: NHEL row {vals} is not {n_ext} long"
        combos.append(vals)
    assert combos, f"{name}: no NHEL rows found in {matrix_file(name)}"
    return combos


class ProbeM2:
    """|M|^2 through the per-diagram probe rather than the shared wrapper.

    `wrappers/generic.f` calls the helicity-summed `MATRIX1` subroutine that
    MadEvent's helicity recycling writes. Where recycling did not run, the matrix
    element keeps `matrix1_orig.f`'s per-helicity `MATRIX1` function instead — a
    different argument list behind the same name. The probe module calls that
    form, and summing its value over MadGraph's own NHEL table is the same
    helicity- and colour-summed number the wrapper would have returned.
    """

    def __init__(self, name: str, n_ext: int):
        self.module = importlib.import_module(f"mg_amp_probe_{name}")
        self.helicities = [
            np.array(h, dtype=np.int32) for h in nhel_table(name, n_ext)
        ]

    def mg_eval_m2(self, p, card):
        return sum(
            float(self.module.mg_eval_m2_hel(p, hel, card)) for hel in self.helicities
        )

    def mg_eval_m2_batch(self, p_batch, card):
        for k in range(p_batch.shape[2]):
            self.mg_eval_m2(np.asfortranarray(p_batch[:, :, k]), card)


def summed_m2_module(name: str, n_ext: int):
    """The helicity- and colour-summed |M|^2 of a process, however it is reachable.

    `build_amplitude.sh` compiles `mg_<name>` against `wrappers/generic.f` only
    where MadEvent's helicity recycling rewrote the matrix element, and says so
    when it declines. Two kinds of process reach here without one: a 2 -> 1
    process, whose `launch` has no phase space to integrate and stops before
    recycling, and an all-scalar process, which has a single helicity
    combination for recycling to work on and keeps `matrix1_orig.f` under the
    optimised name. Both are served by [`ProbeM2`], which reaches the same
    number through the per-diagram probe every row builds anyway. Selecting on
    the module rather than on the Fortran is deliberate: a work area restored
    from the reference bundle carries `matrix1_orig.f` and not the optimised
    file, so the source is not there to read on every checkout and the compiled
    module is.
    """
    try:
        return importlib.import_module(f"mg_{name}")
    except ModuleNotFoundError:
        return ProbeM2(name, n_ext)


def param_card_path(proc: Process) -> str:
    return os.path.join(OUTPUT_DIR, proc.name, "Cards", "param_card.dat")


def read_masses(card_path: str) -> dict[int, float]:
    """PDG id -> mass from the param_card's BLOCK MASS (absent ids are 0)."""
    masses: dict[int, float] = {}
    in_mass_block = False
    with open(card_path) as f:
        for line in f:
            stripped = line.split("#", 1)[0].strip()
            if not stripped:
                continue
            if stripped.lower().startswith("block"):
                in_mass_block = stripped.lower().split()[1] == "mass"
                continue
            if in_mass_block:
                parts = stripped.split()
                if len(parts) >= 2:
                    masses[int(parts[0])] = float(parts[1])
    return masses


def rambo(n: int, sqrt_s: float, masses: list[float], rng: np.random.Generator) -> np.ndarray:
    """n on-shell 4-momenta with the given masses, summing to (sqrt_s, 0, 0, 0).

    Standard Kleiss-Stirling-van der Bij construction (isotropic massless q_i,
    one boost+scale onto the CM frame), followed by the massive-RAMBO momentum
    rescaling: solve xi in sum_i sqrt(m_i^2 + xi^2 |p_i|^2) = sqrt_s (Newton),
    then k_i = (sqrt(m_i^2 + xi^2 |p_i|^2), xi * p_i).  Returns (n, 4) rows of
    [E, px, py, pz].
    """
    q = np.zeros((n, 4))
    for i in range(n):
        cth = 2.0 * rng.random() - 1.0
        phi = 2.0 * math.pi * rng.random()
        sth = math.sqrt(max(0.0, 1.0 - cth * cth))
        e = -math.log(max(1e-300, rng.random() * rng.random()))
        q[i] = [e, e * sth * math.cos(phi), e * sth * math.sin(phi), e * cth]

    big_q = q.sum(axis=0)
    mass = math.sqrt(big_q[0] ** 2 - big_q[1:] @ big_q[1:])
    b = -big_q[1:] / mass
    gamma = big_q[0] / mass
    a = 1.0 / (1.0 + gamma)
    x = sqrt_s / mass

    p = np.zeros((n, 4))
    for i in range(n):
        bq = b @ q[i, 1:]
        p[i, 0] = x * (gamma * q[i, 0] + bq)
        p[i, 1:] = x * (q[i, 1:] + b * q[i, 0] + a * bq * b)

    if all(m == 0.0 for m in masses):
        return p

    # Massive rescaling: Newton for xi (f is monotonic; xi=1 iff all masses 0).
    p2 = (p[:, 1:] ** 2).sum(axis=1)
    m2 = np.array(masses) ** 2
    xi = math.sqrt(max(1e-12, 1.0 - (sum(masses) / sqrt_s) ** 2))
    for _ in range(100):
        e = np.sqrt(m2 + xi**2 * p2)
        f = e.sum() - sqrt_s
        if abs(f) < 1e-13 * sqrt_s:
            break
        xi -= f / (xi * (p2 / e).sum())
    k = np.zeros((n, 4))
    k[:, 0] = np.sqrt(m2 + xi**2 * p2)
    k[:, 1:] = xi * p[:, 1:]
    return k


def final_state(
    n: int, sqrt_s: float, masses: list[float], rng: np.random.Generator
) -> np.ndarray:
    """The outgoing momenta of one phase-space point in the CM frame.

    A one-particle final state has no phase space to sample: momentum
    conservation fixes the single momentum to (sqrt_s, 0, 0, 0) and sqrt_s to
    that particle's mass, so the "grid" of a 2 -> 1 row is the one point the
    process has and RAMBO is neither used nor meaningful there.
    """
    if n == 1:
        assert abs(sqrt_s - masses[0]) < 1e-9 * max(1.0, sqrt_s), (
            f"a 2 -> 1 process exists only at sqrt_s = {masses[0]}, not {sqrt_s}"
        )
        return np.array([[sqrt_s, 0.0, 0.0, 0.0]])
    return rambo(n, sqrt_s, masses, rng)


def beam_momenta(sqrt_s: float, m1: float, m2: float) -> tuple[list[float], list[float]]:
    """CM-frame beam 4-momenta along ±z for incoming masses (m1, m2)."""
    s = sqrt_s**2
    e1 = (s + m1**2 - m2**2) / (2.0 * sqrt_s)
    e2 = (s + m2**2 - m1**2) / (2.0 * sqrt_s)
    pz = math.sqrt(max(0.0, e1**2 - m1**2))
    return [e1, 0.0, 0.0, pz], [e2, 0.0, 0.0, -pz]


def gen_process(proc: Process) -> tuple[int, list[list[float]], dict]:
    """Evaluate a process's MadGraph amplitude over its phase-space sample.

    Leg order and momenta conventions: leg 1 along +z, leg 2 along -z (both
    incoming), then the outgoing legs in process order.  Returns (n_ext, rows,
    timing) where each row is [m2, E0,px0,py0,pz0, E1,...] over all n_ext legs
    and timing is the profile_batch record.
    """
    n_ext = len(proc.pdgs_in) + len(proc.pdgs_out)
    module = summed_m2_module(proc.name, n_ext)
    card = param_card_path(proc)
    masses = read_masses(card)
    m_in = [masses.get(abs(pdg), 0.0) for pdg in proc.pdgs_in]
    m_out = [masses.get(abs(pdg), 0.0) for pdg in proc.pdgs_out]
    rng = np.random.default_rng(proc.seed)

    rows: list[list[float]] = []
    for sqrt_s in proc.sqrt_s_list:
        assert len(m_out) == 1 or sqrt_s > sum(m_out) + 1.0, (
            f"{proc.name}: sqrt_s={sqrt_s} too close to threshold {sum(m_out)}"
        )
        assert len(m_out) > 1 or proc.npoints == 1, (
            f"{proc.name}: a 2 -> 1 process has one phase-space point, "
            f"not {proc.npoints}"
        )
        beam1, beam2 = beam_momenta(sqrt_s, m_in[0], m_in[1])
        for _ in range(proc.npoints):
            out = final_state(len(m_out), sqrt_s, m_out, rng)
            p = np.zeros((4, n_ext), dtype=np.float64, order="F")
            p[:, 0] = beam1
            p[:, 1] = beam2
            for j in range(len(m_out)):
                p[:, 2 + j] = out[j]
            m2 = float(module.mg_eval_m2(p, card))
            row = [m2]
            for leg in range(n_ext):
                row.extend(float(x) for x in p[:, leg])
            rows.append(row)

    timing = profile_batch(module, proc, card, m_in, m_out)
    return n_ext, rows, timing


def profile_batch(
    module, proc: Process, card: str, m_in: list[float], m_out: list[float]
) -> dict:
    """Rough MATRIX1 timing via the batch entry point (not a rigorous benchmark).

    Times a dedicated RAMBO batch of `proc.profile_npoints` points at the highest
    registered collision energy, so the Fortran-call overhead amortizes and the
    ns/eval is comparable to vibegraph's own per-point evaluation timing.
    """
    n = proc.profile_npoints
    n_ext = len(m_in) + len(m_out)
    sqrt_s = proc.sqrt_s_list[-1]
    rng = np.random.default_rng(proc.seed + 1_000_000)  # independent of the CSV stream
    beam1, beam2 = beam_momenta(sqrt_s, m_in[0], m_in[1])
    p_batch = np.zeros((4, n_ext, n), dtype=np.float64, order="F")
    for k in range(n):
        p_batch[:, 0, k] = beam1
        p_batch[:, 1, k] = beam2
        out = final_state(len(m_out), sqrt_s, m_out, rng)
        for j in range(len(m_out)):
            p_batch[:, 2 + j, k] = out[j]
    module.mg_eval_m2_batch(p_batch, card)  # warm-up
    t0 = time.perf_counter()
    module.mg_eval_m2_batch(p_batch, card)
    t1 = time.perf_counter()
    total_ms = (t1 - t0) * 1e3
    ns_per_eval = (t1 - t0) / n * 1e9
    print(f"  MATRIX1 timing [{proc.name}]: {n} evals in {total_ms:.2f} ms  ({ns_per_eval:.0f} ns/eval)")
    return {"process_str": proc.process_str, "n_evals": n, "total_ms": total_ms, "ns_per_eval": ns_per_eval}


def write_csv(path: str, proc: Process, n_ext: int, rows: list[list[float]]):
    comps = ",".join(f"{c}{leg}" for leg in range(n_ext) for c in ("E", "px", "py", "pz"))
    with open(path, "w") as f:
        f.write(f"# process: {proc.process_str}\n")
        f.write(f"# n_ext: {n_ext}\n")
        f.write(f"m2_summed,{comps}\n")
        for row in rows:
            f.write(",".join(repr(v) for v in row) + "\n")
    print(f"  Wrote {len(rows)} rows ({n_ext} legs) to {path}")


def sanity_check_ee_mumu(rows: list[list[float]]):
    """Cross-check sum_hel |M|^2 = 4 e^4 (1 + cos^2 theta) off the Z pole.

    Uses the low-energy (sqrt_s ~ 10 GeV) rows, where gamma exchange dominates;
    5% tolerance absorbs the residual Z interference.
    """
    aewm1 = 132.507  # MadGraph default SMINPUTS aEWM1 (sanity-guard only)
    ee = math.sqrt(4.0 * math.pi / aewm1)
    checked = 0
    for row in rows:
        e_tot = row[1] + row[5]  # E of the two beams
        if abs(e_tot - 10.0) > 0.5:
            continue
        # cos theta of leg 3 (mu-) w.r.t. the +z beam axis
        e3, pz3 = row[13], row[16]
        cos_theta = pz3 / e3
        m2_qed = 4.0 * ee**4 * (1.0 + cos_theta**2)
        rel = abs(row[0] - m2_qed) / m2_qed
        assert rel < 0.05, (
            f"QED sanity check failed at cos={cos_theta:.2f}: "
            f"MG={row[0]:.4e}, QED={m2_qed:.4e}, rel={rel:.3f}"
        )
        checked += 1
    assert checked > 0, "no sqrt_s=10 rows found for the QED sanity check"
    print(f"  Sanity check passed: ee->mumu matches the QED formula on {checked} points.")


def dump_processes() -> None:
    """Print the resolved registry as JSON, sorted by name, and exit.

    Imports no MadGraph module and touches no compiled `.so` — this is the
    migration oracle for `validation/manifest.toml`'s `mg_amplitude` tables:
    run before and after a change to how the registry is built (the manifest
    move, a future field rename) and diff the two dumps. Sorted rather than
    manifest-order so the comparison is insensitive to a row's position in the
    manifest file.
    """
    records = [
        {
            "name": p.name,
            "process_str": p.process_str,
            "pdgs_in": list(p.pdgs_in),
            "pdgs_out": list(p.pdgs_out),
            "sqrt_s_list": list(p.sqrt_s_list),
            "npoints": p.npoints,
            "seed": p.seed,
            "profile_npoints": p.profile_npoints,
            **({"model": p.model} if p.model is not None else {}),
            **({"restrict": p.restrict} if p.restrict is not None else {}),
        }
        for p in PROCESSES
    ]
    records.sort(key=lambda r: r["name"])
    print(json.dumps(records, indent=2, sort_keys=True))


if __name__ == "__main__":
    if "--dump-processes" in sys.argv[1:]:
        dump_processes()
        sys.exit(0)

    os.makedirs(OUTPUT_DIR, exist_ok=True)

    selected = [a for a in sys.argv[1:] if not a.startswith("-")]
    known = {p.name for p in PROCESSES}
    unknown = sorted(set(selected) - known)
    assert not unknown, f"no such registered process: {unknown}"
    procs = [p for p in PROCESSES if not selected or p.name in selected]

    timings_path = os.path.join(OUTPUT_DIR, "mg_timings.json")
    timings: dict[str, dict] = {}
    if selected and os.path.exists(timings_path):
        with open(timings_path) as f:
            timings = json.load(f).get("processes", {})
    for proc in procs:
        print(f"Generating {proc.name} ({proc.process_str})...")
        n_ext, rows, timing = gen_process(proc)
        timings[proc.name] = timing
        if proc.name == "ee_to_mumu":
            sanity_check_ee_mumu(rows)
        write_csv(
            os.path.join(OUTPUT_DIR, f"{proc.name}_amplitude.csv"),
            proc,
            n_ext,
            rows,
        )

    with open(timings_path, "w") as f:
        json.dump(
            {
                "schema": 1,
                "_comment": "MATRIX1 ns/eval per process, measured by this script on the "
                "machine in `host`. A measurement about that machine, not a reference: "
                "absolute times do not carry across hosts and this file is not part of "
                "the refdata bundle. scripts/mg_perf_compare.sh reads it, falling back "
                "to the committed validation/madgraph/mg_timings.json when this work "
                "area lacks one.",
                "host": host_block(),
                "processes": timings,
            },
            f,
            indent=2,
        )
    print(f"Wrote MATRIX1 timing table to {timings_path}")

    print("Done.")
