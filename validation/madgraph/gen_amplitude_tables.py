#!/usr/bin/env python3
"""Bank one committed amplitude table per validated process: MadGraph's own
|M|^2, per-diagram AMP() and per-flow JAMP() at phase-space points the Rust gate
reads back verbatim (`vibegraph-lib/tests/amplitude_oracle.rs`).

Two labelled point sets per process, because they cover different things:

  event  MadGraph's own banked unweighted events, so the comparison sits where
         the cross section actually lives — on the resonance, against the run
         card's cuts, at the invariant masses the sample populates.
  grid   the RAMBO grid of `gen_amplitude.py`, which visits the off-peak corners
         an event sample under-populates by construction.

Every point carries the helicity- and colour-summed |M|^2. A few points of each
set additionally carry the finest linear object MadGraph exposes: AMP(i) per
diagram and JAMP(i) per colour flow, for every helicity combination in the
process's own NHEL table whose amplitude does not vanish identically. The
vanishing combinations are not stored; the gate requires ours to vanish there,
which is what keeps the omission an assertion rather than a gap.

## The on-shell projection

An LHE file prints momenta to about 11 significant digits, so a point read back
from one is off shell by ~1e-10 relative. Two independently compiled programs
legitimately disagree there by gauge-dependent parts, and the disagreement is
far larger than the floating-point noise the gate is trying to measure. Both
sides must therefore consume the *same* exactly-on-shell point, and the committed
file holds it: the projection runs here, once, and the Rust gate never re-derives
it. Per event, in this order:

  1. Legs are matched to the process's own leg order by PDG id. If the leg the
     process lists first is the one travelling in -z, every momentum is rotated
     by pi about the y axis ((px, py, pz) -> (-px, py, -pz), exact in floating
     point), which puts it on +z without disturbing anything else.
  2. The final state is boosted into the rest frame of its own printed total Q,
     which for a hadronic event is the partonic centre of mass. sqrt(s-hat) is
     taken from the same object as sqrt(Q.Q).
  3. The residual three-momentum left by the printed digits is removed by
     subtracting its mean from every final-state momentum.
  4. The final-state momenta are scaled by the single xi solving
     sum_i sqrt(m_i^2 + xi^2 |p_i|^2) = sqrt(s-hat) (Newton, the massive-RAMBO
     step), and each energy is recomputed as sqrt(m_i^2 + |p_i|^2) from the card
     mass. Scaling preserves the vanishing momentum sum of step 3.
  5. The beams are rebuilt along +/-z at that sqrt(s-hat) from the incoming card
     masses, exactly as the fixed grid builds them.

The result is on shell to the last bit by construction and conserves momentum to
rounding, at the kinematics of a real event rather than of a sampler.

Usage:
  pixi run -e madgraph generate-amplitude-tables
  python validation/madgraph/gen_amplitude_tables.py ee_to_mumu   # only this row

Prerequisites:
  pixi run -e madgraph build-amplitude     # builds mg_<name> and mg_amp_probe_<name>
  pixi run -e madgraph generate-amplitude  # writes the fixed-grid <name>_amplitude.csv
"""

import glob
import importlib
import json
import math
import os
import re
import sys
from dataclasses import dataclass

import numpy as np
import pylhe

HERE = os.path.dirname(os.path.abspath(__file__))
OUTPUT_DIR = os.path.join(HERE, "output")
TABLE_DIR = os.path.join(HERE, "amplitudes")
# The compiled mg_*.so matrix elements are build products in the work area.
sys.path.insert(0, os.path.join(OUTPUT_DIR, "f2py"))
sys.path.insert(0, HERE)

import gen_amplitude  # noqa: E402  (needs the work area on sys.path first)

# Banked events per process. Enough that a convention error has to survive two
# dozen independent kinematic configurations, few enough to commit.
N_EVENT_POINTS = 24

# Points of each set that carry the per-helicity tables. The per-diagram fit the
# gate performs is one constant over every (point, helicity, diagram) entry, so
# what a further point adds is another chance for a momentum-dependent phase to
# show up — three of each set is already a heavily over-determined fit.
N_DETAIL_POINTS = 3

# Above this diagram count the per-diagram table is left out and only the
# per-flow JAMPs are banked: the 2 -> 6 processes have 579 and 615 diagrams over
# 256 helicity combinations, which is megabytes per point.
MAX_DIAGRAMS_FOR_AMP_TABLE = 64


@dataclass
class Row:
    """One committed table: the process, and where its two point sets come from."""

    key: str
    # The fixed-grid registry entry (`gen_amplitude.PROCESSES`) supplying the
    # grid points, the process string and the leg PDG ids.
    grid_key: str
    # Banked events to project into the table. Zero for a 2 -> 1 process: its
    # phase space is a single point, MadEvent has no volume to integrate and
    # writes no event file, and the fixed grid already holds that one point.
    n_event_points: int = N_EVENT_POINTS


def rows():
    """Every validated process, in key order.

    `u u~ > mu+ mu-` is generated twice by MadGraph — once as its own script and
    once as the concrete subprocess of the `p p > l+ l- QCD=0` group — and both
    are validated rows, so both get a table. They share the fixed grid (the same
    process at the same energies) and differ in their event sets: partonic beams
    for one, boosted proton-beam events for the other.
    """
    out = [
        Row(p.name, p.name, 0 if len(p.pdgs_out) == 1 else N_EVENT_POINTS)
        for p in gen_amplitude.PROCESSES
    ]
    out.append(Row("uux_to_mumu", "pp_to_ll_qcd0"))
    return sorted(out, key=lambda r: r.key)


# ─────────────────────────── MadGraph source reading ──────────────────────────


def matrix_file(key):
    """The process's single SubProcesses/P*/matrix1_orig.f."""
    sub = os.path.join(OUTPUT_DIR, key, "SubProcesses")
    dirs = [d for d in sorted(os.listdir(sub)) if d.startswith("P")]
    assert len(dirs) == 1, f"{key}: expected one P* subprocess dir, got {dirs}"
    return os.path.join(sub, dirs[0], "matrix1_orig.f")


def join_continuations(src):
    """Fold Fortran fixed-form continuation lines into their statement."""
    return re.sub(r"\n     [^ ]", "", src)


def read_dimensions(key):
    """(NGRAPHS, NCOLOR, [flow structure labels]) from matrix1_orig.f.

    MadGraph prints one `C  <coefficient> <structure>` comment after each `DATA`
    block of the colour-factor matrix; the k-th names colour flow k. Two
    spellings of that block exist: 3.5.x writes one column of a square array of
    reals (`DATA (CF(I,  k),I=  1,  N)`), 3.7.x one row of the upper triangle of
    an integer array over a common denominator (`DATA (CF(I),I=  a,  b)`). The
    labels sit in the same place in both, and only the labels are read here.
    """
    with open(matrix_file(key)) as f:
        src = f.read()
    ngraphs = int(re.search(r"NGRAPHS=(\d+)", src).group(1))
    ncolor = int(re.search(r"NCOLOR=(\d+)", src).group(1))
    structures = re.findall(
        r"DATA \(CF\(I(?:, *\d+)?\),.*?\n(?:.*?\n)*?C +\d+ (\S.*?)\n", src
    )
    assert len(structures) >= ncolor, f"{key}: {len(structures)} structures for NCOLOR={ncolor}"
    return ngraphs, ncolor, structures[:ncolor]


def read_jamp_coefficients(key, ngraphs):
    """The colour coefficients c_i of `JAMP(1,1) = sum_i c_i AMP(i)`.

    These are the weights that turn the per-diagram amplitudes into the coherent
    single-flow amplitude, and they are where MadGraph puts the relative sign
    between an annihilation and an exchange diagram — vibegraph puts that sign in
    the diagram root instead, so the comparable object is the product c_i*AMP(i)
    rather than AMP(i) alone. Verified numerically against the probe's own
    JAMP(1) at every banked point, so a mis-parse cannot pass.
    """
    with open(matrix_file(key)) as f:
        src = join_continuations(f.read())
    stmt = re.search(r"JAMP\(1,1\)\s*=(.*)", src).group(1).replace(" ", "")
    coeffs = [0j] * ngraphs
    seen = set()
    for sign, coef, idx in re.findall(r"([+-]?)(?:\(([^)]*)\)\*)?AMP\((\d+)\)", stmt):
        i = int(idx) - 1
        assert i not in seen, f"{key}: AMP({idx}) appears twice in JAMP(1,1)"
        seen.add(i)
        if coef:
            parts = [float(p.replace("D", "E")) for p in coef.split(",")]
            value = complex(parts[0], parts[1] if len(parts) > 1 else 0.0)
        else:
            value = 1.0 + 0j
        coeffs[i] = -value if sign == "-" else value
    assert len(seen) == ngraphs, f"{key}: JAMP(1,1) covers {len(seen)}/{ngraphs} graphs"
    return coeffs


def read_amp2_groups(key):
    """The AMP indices each `AMP2(k)` accumulator sums, in MadGraph's config order.

    `AMP2` is the per-integration-configuration squared amplitude MadEvent's
    single-diagram enhancement weights a channel by, and — through
    `SELECT_COLOR`'s `ICOLAMP(iflow, iconfig, iproc)` mask — the distribution an
    event's colour flow is drawn conditional on. Which diagrams get one is the
    whole content of `get_amp2_lines` (`madgraph/iolibs/export_v4.py`): a diagram
    with a vertex wider than the narrowest diagram's widest vertex — a four-point
    contact — gets no accumulator, no configuration and no `ICOLAMP` column.

    Two statement forms appear, and they are not the same sum:
      `AMP2(k)=AMP2(k)+AMP(i)*DCONJG(AMP(i))+...`  one diagram's amplitudes,
                                                   squared separately and added;
      `AMP2(k)=AMP2(k)+(AMP(i)+AMP(j))*DCONJG(...)` several diagrams of one
                                                   configuration, added first.
    Anything else is a third form this parser has not seen and must not guess at,
    so it asserts rather than falling through.

    Returns 0-based AMP indices, one list per configuration.
    """
    with open(matrix_file(key)) as f:
        src = join_continuations(f.read())
    groups = []
    for stmt in re.findall(r"AMP2\(\d+\)=AMP2\(\d+\)\+(.*)", src):
        rhs = stmt.strip().replace(" ", "")
        if rhs.startswith("("):
            coherent = rhs[1 : rhs.index(")*")]
            indices = [int(i) for i in re.findall(r"AMP\((\d+)\)", coherent)]
            rebuilt = "(" + "+".join(f"AMP({i})" for i in indices) + ")"
            rebuilt = f"{rebuilt}*DCONJG{rebuilt}"
        else:
            indices = [
                int(i) for i in re.findall(r"AMP\((\d+)\)\*DCONJG\(AMP\(\1\)\)", rhs)
            ]
            rebuilt = "+".join(f"AMP({i})*DCONJG(AMP({i}))" for i in indices)
        assert rebuilt.upper() == rhs.upper(), (
            f"{key}: unrecognised AMP2 statement '{rhs}' (parsed back as '{rebuilt}')"
        )
        groups.append([i - 1 for i in indices])
    assert groups, f"{key}: no AMP2 accumulator lines in {matrix_file(key)}"
    return groups


def helicity_combos(key, n_ext):
    """MadGraph's own NHEL table, in its own row order.

    Read from the generated `DATA (NHEL(I,N),I=1,n_ext) / ... /` block rather
    than enumerated as a product, so the banked helicity set is MadGraph's and a
    disagreement with vibegraph's enumeration is a finding rather than something
    the reference quietly conforms to.
    """
    return gen_amplitude.nhel_table(key, n_ext)


def param_card_path(key):
    return os.path.join(OUTPUT_DIR, key, "Cards", "param_card.dat")


# ───────────────────────────────── point sets ────────────────────────────────


def read_grid_points(grid_key):
    """(process_str, n_ext, [(momenta, m2)]) from the fixed-grid amplitude CSV."""
    path = os.path.join(OUTPUT_DIR, f"{grid_key}_amplitude.csv")
    with open(path) as f:
        lines = [line for line in f if line.strip()]
    process_str = next(
        line.split(":", 1)[1].strip() for line in lines if line.startswith("# process:")
    )
    n_ext = next(int(line.split(":")[1]) for line in lines if line.startswith("# n_ext:"))
    data = [line for line in lines if not line.startswith("#")][1:]  # skip column header
    points = []
    for row in data:
        vals = [float(x) for x in row.split(",")]
        momenta = [vals[1 + 4 * leg : 5 + 4 * leg] for leg in range(n_ext)]
        points.append((momenta, vals[0]))
    return process_str, n_ext, points


def read_masses(card_path):
    """PDG id -> mass from the param card's BLOCK MASS (absent ids are 0)."""
    return gen_amplitude.read_masses(card_path)


def banked_lhe(key):
    """The process's banked unweighted event file, relative path and full path."""
    pattern = os.path.join(OUTPUT_DIR, key, "Events", "*", "unweighted_events.lhe.gz")
    matches = sorted(glob.glob(pattern))
    assert matches, f"{key}: no banked unweighted_events.lhe.gz under Events/"
    return os.path.relpath(matches[0], OUTPUT_DIR), matches[0]


def select_legs(event, pdgs_in, pdgs_out):
    """The event's momenta in the process's leg order, or None if it is another
    subprocess of the same group.

    Status +/-2 lines (MadGraph's intermediate resonances) are not external legs
    and are dropped. Identical particles are assigned to slots first-come, which
    is deterministic and — since either assignment is a valid phase-space point
    both sides are evaluated at — loses nothing.
    """
    incoming = [p for p in event.particles if int(p.status) == -1]
    outgoing = [p for p in event.particles if int(p.status) == 1]
    if len(incoming) != len(pdgs_in) or len(outgoing) != len(pdgs_out):
        return None

    def assign(particles, pdgs):
        remaining = list(particles)
        picked = []
        for pdg in pdgs:
            match = next((p for p in remaining if int(p.id) == pdg), None)
            if match is None:
                return None
            remaining.remove(match)
            picked.append(match)
        return picked

    legs_in = assign(incoming, pdgs_in)
    legs_out = assign(outgoing, pdgs_out)
    if legs_in is None or legs_out is None:
        return None

    p_in = [[p.e, p.px, p.py, p.pz] for p in legs_in]
    p_out = [[p.e, p.px, p.py, p.pz] for p in legs_out]
    if p_in[0][3] < 0.0:
        # The leg the process lists first travels in -z: rotate by pi about y,
        # which is a sign flip on px and pz and therefore exact.
        p_in = [[e, -px, py, -pz] for e, px, py, pz in p_in]
        p_out = [[e, -px, py, -pz] for e, px, py, pz in p_out]
    return p_in, p_out


def project_on_shell(p_out_printed, m_in, m_out):
    """The on-shell projection documented in this module's header.

    Takes the printed final-state momenta and the card masses, returns the full
    momentum list (incoming then outgoing) together with the residuals worth
    reporting: the largest |p^2 - m^2|/s-hat and the momentum-conservation
    imbalance, both of which should sit at rounding rather than at the printed
    precision.
    """
    q = np.array(p_out_printed, dtype=np.float64).sum(axis=0)
    s_hat = q[0] ** 2 - q[1:] @ q[1:]
    sqrt_s = math.sqrt(s_hat)

    # Into the rest frame of the printed final-state total. The 1/(1+gamma) form
    # stays well conditioned as beta -> 0 (a lepton-collider event is already
    # there).
    gamma = q[0] / sqrt_s
    beta = q[1:] / q[0]
    a = gamma * gamma / (1.0 + gamma)
    spatial = np.zeros((len(p_out_printed), 3))
    for i, p in enumerate(p_out_printed):
        vec = np.array(p[1:], dtype=np.float64)
        spatial[i] = vec + beta * (a * (beta @ vec) - gamma * p[0])

    # Remove what the printed digits left over, then restore sqrt(s-hat) with the
    # single massive-RAMBO scale factor. Scaling preserves the vanishing sum.
    spatial -= spatial.mean(axis=0)
    p2 = (spatial**2).sum(axis=1)
    m2 = np.array(m_out) ** 2
    xi = 1.0
    for _ in range(100):
        e = np.sqrt(m2 + xi**2 * p2)
        f = e.sum() - sqrt_s
        if abs(f) <= 1e-16 * sqrt_s:
            break
        xi -= f / (xi * (p2 / e).sum())
    spatial *= xi

    out = []
    for i in range(len(p_out_printed)):
        energy = math.sqrt(m2[i] + spatial[i] @ spatial[i])
        out.append([energy, *spatial[i]])

    beam1, beam2 = gen_amplitude.beam_momenta(sqrt_s, m_in[0], m_in[1])
    momenta = [beam1, beam2, *out]

    masses = [*m_in, *m_out]
    onshell = max(
        abs(p[0] ** 2 - p[1] ** 2 - p[2] ** 2 - p[3] ** 2 - m**2) / s_hat
        for p, m in zip(momenta, masses)
    )
    imbalance = max(
        abs(momenta[0][c] + momenta[1][c] - sum(p[c] for p in out)) / sqrt_s
        for c in range(4)
    )
    return momenta, onshell, imbalance


def read_event_points(key, proc, masses, n_points):
    """Projected momenta from the process's own banked events.

    Events of another subprocess of the same group are skipped: a grouped
    MadGraph run writes several flavour assignments into one file, and only the
    one this table's matrix element was generated for is a point for it.
    """
    relpath, path = banked_lhe(key)
    m_in = [masses.get(abs(pdg), 0.0) for pdg in proc.pdgs_in]
    m_out = [masses.get(abs(pdg), 0.0) for pdg in proc.pdgs_out]

    points, seen, worst_onshell, worst_imbalance = [], 0, 0.0, 0.0
    for event in pylhe.LHEFile.fromfile(path).events:
        seen += 1
        legs = select_legs(event, list(proc.pdgs_in), list(proc.pdgs_out))
        if legs is None:
            continue
        momenta, onshell, imbalance = project_on_shell(legs[1], m_in, m_out)
        worst_onshell = max(worst_onshell, onshell)
        worst_imbalance = max(worst_imbalance, imbalance)
        points.append(momenta)
        if len(points) == n_points:
            break
    assert len(points) == n_points, (
        f"{key}: only {len(points)} of the first {seen} banked events match "
        f"{proc.process_str}"
    )
    return relpath, points, worst_onshell, worst_imbalance


# ─────────────────────────────── the tables ──────────────────────────────────


def detail_indices(n_points, n_detail):
    """Which points of a set carry the per-helicity tables: the first `n_detail`."""
    return set(range(min(n_points, n_detail)))


def evaluate(key, row, proc):
    """Everything one process's committed table holds."""
    process_str, n_ext, grid_points = read_grid_points(row.grid_key)
    assert process_str == proc.process_str, (
        f"{key}: grid CSV says '{process_str}', registry says '{proc.process_str}'"
    )
    ngraphs, ncolor, structures = read_dimensions(key)
    combos = helicity_combos(key, n_ext)
    coefficients = read_jamp_coefficients(key, ngraphs) if ncolor == 1 else None
    card = param_card_path(key)
    masses = read_masses(card)

    # The helicity-summed |M|^2 is what the event points need; the grid points
    # carry MadGraph's own value from the CSV. A row with no event set (a 2 -> 1
    # process, whose `launch` writes no events) therefore never needs it.
    m2_module = (
        gen_amplitude.summed_m2_module(key, n_ext) if row.n_event_points else None
    )
    probe = importlib.import_module(f"mg_amp_probe_{key}")

    if row.n_event_points:
        relpath, event_points, worst_onshell, worst_imbalance = read_event_points(
            key, proc, masses, row.n_event_points
        )
    else:
        relpath, event_points, worst_onshell, worst_imbalance = None, [], 0.0, 0.0

    amp2_groups = read_amp2_groups(key)

    banked = []
    with_amps = ngraphs <= MAX_DIAGRAMS_FOR_AMP_TABLE
    for label, momenta_list, m2_list in (
        ("event", event_points, [None] * len(event_points)),
        ("grid", [m for m, _ in grid_points], [v for _, v in grid_points]),
    ):
        detail_at = detail_indices(len(momenta_list), N_DETAIL_POINTS)
        for i, momenta in enumerate(momenta_list):
            p = np.asfortranarray(np.array(momenta, dtype=np.float64).T)
            m2 = m2_list[i]
            if m2 is None:
                m2 = float(m2_module.mg_eval_m2(p, card))
            entry = {"set": label, "momenta": momenta, "m2": m2}
            if i in detail_at:
                entry["detail"] = dump_detail(
                    key, probe, p, card, combos, coefficients, with_amps
                )
            banked.append(entry)

    table = {
        "key": key,
        "process": process_str,
        "n_ext": n_ext,
        "n_graphs": ngraphs,
        "n_flows": ncolor,
        "flow_structures": structures,
        "jamp_coefficients": (
            [[c.real, c.imag] for c in coefficients] if coefficients else None
        ),
        "amp2_groups": amp2_groups,
        "helicities": combos,
        "param_card": open(card).read().splitlines(),
        "grid_source": f"{row.grid_key}_amplitude.csv",
        "event_source": relpath,
        "amp_table": with_amps,
        "amp_table_note": (
            None
            if with_amps
            else f"{ngraphs} diagrams over {len(combos)} helicity combinations: "
            "only the per-flow JAMPs are banked"
        ),
        "points": banked,
    }
    stats = {
        "onshell": worst_onshell,
        "imbalance": worst_imbalance,
        "n_detail": sum(1 for e in banked if "detail" in e),
    }
    return table, stats


def dump_detail(key, probe, p, card, combos, coefficients, with_amps):
    """AMP() and JAMP() for every helicity combination that does not vanish."""
    indices, amps, jamps = [], [], []
    for h, hel in enumerate(combos):
        amp, jamp = probe.mg_eval_amp(p, np.array(hel, dtype=np.int32), card)
        amp, jamp = np.asarray(amp), np.asarray(jamp)
        if not amp.any() and not jamp.any():
            continue
        if coefficients is not None:
            rebuilt = sum(c * a for c, a in zip(coefficients, amp))
            assert abs(rebuilt - jamp[0]) <= 1e-10 * max(abs(jamp[0]), 1e-30), (
                f"{key}: the parsed JAMP coefficients do not rebuild "
                f"JAMP(1)={jamp[0]} from AMP() at helicity {hel}"
            )
        indices.append(h)
        jamps.append([[float(z.real), float(z.imag)] for z in jamp])
        if with_amps:
            amps.append([[float(z.real), float(z.imag)] for z in amp])
    detail = {"helicities": indices, "jamps": jamps}
    if with_amps:
        detail["amps"] = amps
    return detail


# ───────────────────────────────── output ────────────────────────────────────


def write_table(path, table):
    """One line per point, so a regenerated table diffs point by point."""
    head = {k: v for k, v in table.items() if k != "points"}
    with open(path, "w") as f:
        f.write("{\n")
        for k, v in head.items():
            f.write(f" {json.dumps(k)}: {json.dumps(v)},\n")
        f.write(' "points": [\n')
        for i, point in enumerate(table["points"]):
            comma = "," if i + 1 < len(table["points"]) else ""
            f.write(f"  {json.dumps(point)}{comma}\n")
        f.write(" ]\n}\n")


def main():
    os.makedirs(TABLE_DIR, exist_ok=True)
    registry = {p.name: p for p in gen_amplitude.PROCESSES}
    selected = [a for a in sys.argv[1:] if not a.startswith("-")]
    all_rows = rows()
    unknown = sorted(set(selected) - {r.key for r in all_rows})
    assert not unknown, f"no such table row: {unknown}"
    wanted = [r for r in all_rows if not selected or r.key in selected]
    total = 0
    for row in wanted:
        proc = registry[row.grid_key]
        table, stats = evaluate(row.key, row, proc)
        path = os.path.join(TABLE_DIR, f"{row.key}.json")
        write_table(path, table)
        size = os.path.getsize(path)
        total += size
        print(
            f"[{row.key}] {table['process']}: {len(table['points'])} points "
            f"({row.n_event_points} event + "
            f"{len(table['points']) - row.n_event_points} grid), "
            f"{stats['n_detail']} with per-helicity tables, NGRAPHS={table['n_graphs']} "
            f"NCOLOR={table['n_flows']}, projection: off-shell <= {stats['onshell']:.1e}, "
            f"imbalance <= {stats['imbalance']:.1e}, {size / 1024:.0f} KiB"
        )
    print(f"wrote {len(wanted)} tables to {TABLE_DIR} ({total / 1024:.0f} KiB total)")


if __name__ == "__main__":
    main()
