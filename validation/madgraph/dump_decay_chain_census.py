#!/usr/bin/env python3
"""Run a corpus of decay-chain proc cards through MadGraph's own generation and
bank the matrix elements it combines them into.

Every card is fed, line by line, to a ``MasterCmd`` exactly as ``mg5_aMC``
feeds a proc card, with diagram generation switched on, and the amplitudes it
holds are turned into matrix elements the way ``output`` does
(``HelasMultiProcess``, whose ``generate_matrix_elements`` hands a
``DecayChainAmplitude`` to ``HelasDecayChainProcess`` and
``combine_decay_chain_processes``). Pure Python: no Fortran, no process
directory. What is banked per card is either the error MadGraph raised or, for
every combined matrix element:

  diagrams                 its number of diagrams: core diagrams times decay
                           diagrams, each decay on the legs it was assigned
                           (no permutation of identical particles between
                           decays);
  identical_particle_factor  the denominator MadGraph divides by, which for
                           identical decay chains is ``identical_decay_chain_
                           factor`` rather than the final state's own factor;
  processes                every (process, decays) combined into it, each with
                           the core's PDG codes, the concrete decays (each with
                           its own), and the final state with every decay
                           substituted in place (``get_legs_with_decays``);

plus the warnings MadGraph logged (a decay without a matching core particle is
discarded with one).

Read back by the hermetic test ``vibegraph-lib/tests/decay_chain_census.rs``.

Usage:
  pixi run dump-decay-chain-census
  python validation/madgraph/dump_decay_chain_census.py

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
_out = os.path.join(_here, "decay_chain_census.json")
_root = os.environ.get("MG5AMCNLO_PATH") or os.path.join(
    _here, "..", "..", "research", "refs", "mg5amcnlo"
)
sys.path.insert(0, os.path.abspath(_root))

with contextlib.redirect_stdout(io.StringIO()):
    import madgraph.core.helas_objects as helas_objects  # noqa: E402
    import madgraph.interface.master_interface as master_interface  # noqa: E402


class _Warnings(logging.Handler):
    def __init__(self):
        super().__init__(level=logging.WARNING)
        self.messages = []

    def emit(self, record):
        text = record.getMessage().replace("$RED", "").strip()
        self.messages.append(text.splitlines()[0] if text else text)


def _new_cmd():
    with contextlib.redirect_stdout(io.StringIO()):
        cmd = master_interface.MasterCmd()
    cmd.ask = lambda *args, **kwargs: "no"
    return cmd


def _error(e):
    return f"{type(e).__name__}: {str(e).strip().splitlines()[0]}"


def _decay(process):
    legs = process.get("legs")
    return {
        "initial": [leg.get("id") for leg in legs if not leg.get("state")],
        "final": [leg.get("id") for leg in legs if leg.get("state")],
        "decays": [_decay(d) for d in process.get("decay_chains")],
    }


def census(card):
    cmd = _new_cmd()
    handler = _Warnings()
    logging.getLogger("madgraph").addHandler(handler)
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            for line in card.split("\n"):
                line = line.strip()
                if line:
                    cmd.exec_cmd(line, precmd=True, printcmd=False)
            multi = helas_objects.HelasMultiProcess(cmd._curr_amps)
    except Exception as e:  # MadGraph's refusal is the datum
        return {"error": _error(e), "warnings": handler.messages}
    finally:
        logging.getLogger("madgraph").removeHandler(handler)
    elements = []
    for me in multi.get("matrix_elements"):
        processes = []
        for process in me.get("processes"):
            entry = _decay(process)
            entry["with_decays"] = {
                "initial": [
                    leg.get("id") for leg in process.get_legs_with_decays() if not leg.get("state")
                ],
                "final": [
                    leg.get("id") for leg in process.get_legs_with_decays() if leg.get("state")
                ],
            }
            entry["process"] = process.nice_string().replace("Process: ", "").strip()
            processes.append(entry)
        elements.append(
            {
                "diagrams": len(me.get("diagrams")),
                "identical_particle_factor": me.get("identical_particle_factor"),
                "processes": processes,
            }
        )
    return {"warnings": handler.messages, "matrix_elements": elements}


# ── The corpus ────────────────────────────────────────────────────────────────

CORPUS = [
    # Identical particles from identical decays: every combination with repetition.
    ("ee_zz_ee", "generate e+ e- > z z, z > e+ e-"),
    # As many decays as decaying legs: in order.
    ("ee_zz_ee_mumu", "generate e+ e- > z z, z > e+ e-, z > mu+ mu-"),
    ("ee_zz_mumu_ee", "generate e+ e- > z z, z > mu+ mu-, z > e+ e-"),
    # A label in a decay: one decay spec, two concrete decays.
    ("ee_zz_ll", "generate e+ e- > z z, z > l+ l-"),
    # More decays than decaying legs.
    ("ee_zz_three", "generate e+ e- > z z, z > e+ e-, z > mu+ mu-, z > ta+ ta-"),
    ("ee_zh", "generate e+ e- > z h, z > e+ e-"),
    ("ee_ttx", "generate e+ e- > t t~, t > w+ b, t~ > w- b~"),
    # The out-of-order case: one concrete decay each, written in another order.
    ("ee_ttx_reversed", "generate e+ e- > t t~, t~ > w- b~, t > w+ b"),
    ("ee_ttx_t_only", "generate e+ e- > t t~, t > w+ b"),
    ("ee_ttx_nested", "generate e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~"),
    (
        "ee_ttx_nested_both",
        "generate e+ e- > t t~, (t > w+ b, w+ > e+ ve), (t~ > w- b~, w- > e- ve~)",
    ),
    ("ee_ttx_three_body", "generate e+ e- > t t~, t > b e+ ve, t~ > b~ e- ve~"),
    ("ee_ww_jj", "generate e+ e- > w+ w-, w+ > j j, w- > j j"),
    # Colour through a coloured decay, with a gluon in the core.
    ("uux_ttxg", "generate u u~ > t t~ g, t > w+ b"),
    ("gg_ttx", "generate g g > t t~, t > w+ b, t~ > w- b~"),
    # Hadronic labels.
    ("pp_zj", "generate p p > z j, z > l+ l-"),
    ("pp_ttx", "generate p p > t t~, t > w+ b, t~ > w- b~"),
    # A decay whose particle the core does not have: MadGraph drops it with a warning.
    ("ee_ttx_unused_w", "generate e+ e- > t t~, t > w+ b, w+ > e+ ve"),
    # A decay with more than one initial particle.
    ("not_a_decay", "generate e+ e- > z z, e+ e- > mu+ mu-"),
]


def main():
    cases = []
    for name, card in CORPUS:
        result = census(card)
        print(
            f"{name}: "
            + (
                result["error"]
                if "error" in result
                else f"{len(result['matrix_elements'])} matrix elements, "
                f"{sum(m['diagrams'] for m in result['matrix_elements'])} diagrams"
            ),
            file=sys.stderr,
        )
        cases.append({"name": name, "card": card, **result})
    doc = {
        "metadata": {
            "generator": "validation/madgraph/dump_decay_chain_census.py",
            "madgraph": "research/refs/mg5amcnlo",
        },
        "cases": cases,
    }
    with open(_out, "w") as f:
        json.dump(doc, f, indent=1, sort_keys=False)
        f.write("\n")
    n_err = sum(1 for c in cases if "error" in c)
    print(f"wrote {_out}: {len(cases)} cards, {n_err} refused by MadGraph")


if __name__ == "__main__":
    main()
