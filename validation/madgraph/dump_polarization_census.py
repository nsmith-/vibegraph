#!/usr/bin/env python3
"""Run a corpus of polarized proc cards through MadGraph's own generation and
bank what the polarization does to every matrix element.

Every card is fed, line by line, to a ``MasterCmd`` exactly as ``mg5_aMC``
feeds a proc card, with diagram generation switched on (pure Python: no
Fortran, no process directory). MadGraph's interactive question on an
ambiguous polarization (``check_polarization``, ``p p > z{T} z``) is answered
with its default, "no", which is what a batch run answers too. What is banked
per card is either the error MadGraph raised or, for every amplitude it holds
afterwards, the ``HelasMatrixElement`` built from it:

  legs         every external leg as ``[pdg, state, polarization]``, in
               MadGraph's leg order, after generation dropped the helicity 0
               of a massless boson (``diagram_generation.py:1751``);
  helicities   ``get_helicity_matrix()``: the NHEL rows the generated
               ``matrix.f`` sums over, in the same leg order;
  iden         ``get_denominator_factor()``: the IDEN the generated code
               divides by (initial spin and colour average times the
               identical-particle factor);
  identical    ``identical_particle_factor()``, keyed by pdg and polarization.

Read back by the hermetic test ``vibegraph-lib/tests/polarization_census.rs``,
which enumerates and compiles the same cards and compares.

Usage:
  pixi run dump-polarization-census
  python validation/madgraph/dump_polarization_census.py

Reads the pinned ``research/refs/mg5amcnlo`` submodule; ``MG5AMCNLO_PATH``
overrides it.
"""

import contextlib
import io
import json
import logging
import os
import sys

_here = os.path.dirname(os.path.abspath(__file__))
_out = os.path.join(_here, "polarization_census.json")
_root = os.environ.get("MG5AMCNLO_PATH") or os.path.join(
    _here, "..", "..", "research", "refs", "mg5amcnlo"
)
sys.path.insert(0, os.path.abspath(_root))
logging.disable(logging.WARNING)

with contextlib.redirect_stdout(io.StringIO()):
    import madgraph.interface.master_interface as master_interface  # noqa: E402
    import madgraph.core.helas_objects as helas_objects  # noqa: E402


def _new_cmd():
    with contextlib.redirect_stdout(io.StringIO()):
        cmd = master_interface.MasterCmd()
    cmd.ask = lambda *args, **kwargs: "no"
    return cmd


def _error(e):
    return f"{type(e).__name__}: {str(e).strip().splitlines()[0]}"


def census(card):
    cmd = _new_cmd()
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            for line in card.split("\n"):
                line = line.strip()
                if line:
                    cmd.exec_cmd(line, precmd=True, printcmd=False)
    except Exception as e:  # MadGraph's refusal is the datum
        return {"error": _error(e)}
    subprocesses = []
    for amp in cmd._curr_amps:
        with contextlib.redirect_stdout(io.StringIO()):
            me = helas_objects.HelasMatrixElement(amp, gen_color=False)
        process = me.get("processes")[0]
        legs = [
            [leg.get("id"), bool(leg.get("state")), list(leg.get("polarization"))]
            for leg in process.get("legs")
        ]
        subprocesses.append(
            {
                "process": process.nice_string().replace("Process: ", "").strip(),
                "legs": legs,
                "helicities": [list(h) for h in me.get_helicity_matrix()],
                "iden": me.get_denominator_factor(),
                "identical": process.identical_particle_factor(),
            }
        )
    subprocesses.sort(key=lambda s: s["process"])
    return {"subprocesses": subprocesses}


# ── The corpus ────────────────────────────────────────────────────────────────

CORPUS = [
    # Vector polarizations in the final state.
    ("ee_wp0_wmT", "generate e+ e- > w+{0} w-{T}"),
    ("ee_wp0_wm", "generate e+ e- > w+{0} w-"),
    ("ee_wpT_wm", "generate e+ e- > w+{T} w-"),
    ("ee_wp0_wm0", "generate e+ e- > w+{0} w-{0}"),
    ("ee_wp_signed", "generate e+ e- > w+{+1} w-{-1}"),
    ("ee_z0_h", "generate e+ e- > z{0} h"),
    ("ee_zT_h", "generate e+ e- > z{T} h"),
    ("uux_zT_g", "generate u u~ > z{T} g"),
    # Identical particles told apart by their polarization.
    ("ee_zL_zR", "generate e+ e- > z{L} z{R}"),
    ("ee_z0_z0", "generate e+ e- > z{0} z{0}"),
    ("ee_zT_zT", "generate e+ e- > z{T} z{T}"),
    ("ee_z0_zT", "generate e+ e- > z{0} z{T}"),
    ("ee_zT_z", "generate e+ e- > z{T} z"),
    ("ee_zL_zT", "generate e+ e- > z{L} z{T}"),
    ("ee_zRL_zLR", "generate e+ e- > z{RL} z{LR}"),
    ("ee_a0_a", "generate e+ e- > a{0} a"),
    # Fermions, in the final and the initial state.
    ("ee_mupL_mumR", "generate e+ e- > mu+{L} mu-{R}"),
    ("ee_tL_tx", "generate e+ e- > t{L} t~"),
    ("ee_emL_mumu", "generate e+ e-{L} > mu+ mu-"),
    ("ee_epR_emL_mumu", "generate e+{R} e-{L} > mu+ mu-"),
    ("uLux_z0g", "generate u{L} u~ > z{0} g"),
    # Initial-state vectors: the average is over the listed states.
    ("gT_g_ttx", "generate g{T} g > t t~"),
    ("gp_g_ttx", "generate g{+1} g > t t~"),
    ("gp_gm_ttx", "generate g{+1} g{-1} > t t~"),
    # Helicity 0 on a massless boson: kept by the parser, dropped at generation.
    ("g0_g_ttx", "generate g{0} g > t t~"),
    ("ee_a0_z", "generate e+ e- > a{0} z"),
    ("ee_a0T_z", "generate e+ e- > a{0T} z"),
    ("ee_a0T_comma_z", "generate e+ e- > a{0,T} z"),
    ("uux_v0_g", "define v = z a\ngenerate u u~ > v{0} g"),
    ("pp_z0_j", "generate p p > z{0} j"),
    # Codes MadGraph reads that are not a helicity state of the particle.
    ("ee_z2_h", "generate e+ e- > z{2} h"),
    ("ee_z00_h", "generate e+ e- > z{00} h"),
    ("ee_z_hR", "generate e+ e- > z h{R}"),
    # 1 -> n decays: a polarized decay product, and the decaying particle.
    ("decay_t_wp0_b", "generate t > w+{0} b"),
    ("decay_t_wpT_b", "generate t > w+{T} b"),
    ("decay_tL_wp_b", "generate t{L} > w+ b"),
    # Two process lines over the same final state.
    ("add_disjoint", "generate e+ e- > z{0} h\nadd process e+ e- > z{T} h"),
    ("add_overlap", "generate e+ e- > z{0} h\nadd process e+ e- > z h"),
]


def main():
    cases = []
    for name, card in CORPUS:
        entry = {"name": name, "card": card}
        entry.update(census(card))
        cases.append(entry)
        status = entry.get("error") or f"{len(entry['subprocesses'])} subprocesses"
        print(f"{name:22s} {status}")
    with open(os.path.join(_root, "VERSION")) as f:
        version = f.read().strip()
    out = {
        "metadata": {
            "generator": "validation/madgraph/dump_polarization_census.py",
            "madgraph": version,
        },
        "cases": cases,
    }
    with open(_out, "w") as f:
        json.dump(out, f, indent=1)
        f.write("\n")
    print(f"wrote {_out}")


if __name__ == "__main__":
    main()
