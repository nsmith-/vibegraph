#!/usr/bin/env python3
"""Run a corpus of proc cards through MadGraph's own diagram generation and bank
the s-channel content of every diagram.

Every card is fed, line by line, to a ``MasterCmd`` exactly as ``mg5_aMC``
feeds a proc card, with diagram generation switched on (pure Python: no
Fortran, no process directory). What is banked per card is either the error
MadGraph raised or, for every amplitude it holds afterwards:

  initial, final   the subprocess's PDG codes, each side sorted (a mirror
                   process is one amplitude, as it is one subprocess here);
  diagrams         the number of diagrams;
  s_channels       per diagram, the sorted list of its s-channel propagators'
                   ids as ``Vertex.get_s_channel_id`` orients them (the
                   particle as it flows towards the final state), the whole
                   list sorted: a multiset of multisets, independent of
                   diagram order and numbering. For one initial particle
                   the decaying particle's own line is not among them.

plus the members of ``p`` and ``j`` after the card, which is where MadGraph's
switch to the five-flavour scheme on importing a model with a massless b shows
(``add_default_multiparticles``).

Read back by the hermetic test ``vibegraph-lib/tests/schannel_census.rs``,
which enumerates the same cards and compares.

Usage:
  pixi run dump-schannel-census
  python validation/madgraph/dump_schannel_census.py

Reads the pinned ``research/refs/mg5amcnlo`` submodule; ``MG5AMCNLO_PATH``
overrides it. SMEFTsim is imported from a scratch copy, since MadGraph writes
its restricted-model cache next to the UFO it imports.
"""

import contextlib
import io
import json
import logging
import os
import shutil
import sys
import tempfile

_here = os.path.dirname(os.path.abspath(__file__))
_out = os.path.join(_here, "schannel_census.json")
_root = os.environ.get("MG5AMCNLO_PATH") or os.path.join(
    _here, "..", "..", "research", "refs", "mg5amcnlo"
)
_smeftsim = os.path.join(_here, "..", "ufo", "SMEFTsim_topU3l_MwScheme_UFO")
sys.path.insert(0, os.path.abspath(_root))
logging.disable(logging.WARNING)

with contextlib.redirect_stdout(io.StringIO()):
    import madgraph.interface.master_interface as master_interface  # noqa: E402


def _new_cmd():
    with contextlib.redirect_stdout(io.StringIO()):
        cmd = master_interface.MasterCmd()
    cmd.ask = lambda *args, **kwargs: "no"
    return cmd


def _error(e):
    return f"{type(e).__name__}: {str(e).strip().splitlines()[0]}"


def _s_channels(diagram, model, ninitial):
    # The last vertex is the n->0 one; a decay's diagrams carry one more,
    # artificial, vertex for the decaying particle's own line, which the
    # required-s-channel filter skips too (`diagram_generation.py:720`).
    last = -2 if ninitial == 1 else -1
    ids = []
    for vertex in diagram.get("vertices")[:last]:
        sid = vertex.get_s_channel_id(model, ninitial)
        if sid:
            ids.append(sid)
    return sorted(ids)


def census(card, substitutions):
    for key, value in substitutions.items():
        card = card.replace(key, value)
    cmd = _new_cmd()
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            for line in card.split("\n"):
                line = line.strip()
                if line:
                    cmd.exec_cmd(line, precmd=True, printcmd=False)
    except Exception as e:  # MadGraph's refusal is the datum
        return {"error": _error(e)}
    model = cmd._curr_model
    subprocesses = []
    for amp in cmd._curr_amps:
        process = amp.get("process")
        legs = process.get("legs")
        ninitial = len([leg for leg in legs if not leg.get("state")])
        subprocesses.append(
            {
                "process": process.base_string().strip(),
                "initial": sorted(leg.get("id") for leg in legs if not leg.get("state")),
                "final": sorted(leg.get("id") for leg in legs if leg.get("state")),
                "orders": dict(process.get("orders")),
                "diagrams": len(amp.get("diagrams")),
                "s_channels": sorted(
                    _s_channels(d, model, ninitial) for d in amp.get("diagrams")
                ),
            }
        )
    subprocesses.sort(key=lambda s: (s["initial"], s["final"]))
    return {
        "multiparticles": {k: list(cmd._multiparticles[k]) for k in ("p", "j")},
        "subprocesses": subprocesses,
    }


# ── The corpus ────────────────────────────────────────────────────────────────
#
# (name, card, what the Rust side enumerates it with). `@SMEFTSIM@` is replaced
# by the path of a scratch copy of the SMEFTsim UFO; the Rust side reads the
# committed one.

CORPUS = [
    # Required s-channels.
    ("ee_z_mumu", "generate e+ e- > z > mu+ mu-"),
    ("ee_a_mumu", "generate e+ e- > a > mu+ mu-"),
    ("ee_mumu", "generate e+ e- > mu+ mu-"),
    ("ee_or_inline_mumu", "generate e+ e- > z | a > mu+ mu-"),
    ("ee_or_label_mumu", "define v = z | a\ngenerate e+ e- > v > mu+ mu-"),
    ("udx_wp_epve", "generate u d~ > w+ > e+ ve"),
    ("udx_wm_epve", "generate u d~ > w- > e+ ve"),
    ("ee_zh_mumubbx", "generate e+ e- > z h > mu+ mu- b b~"),
    ("ee_za_mumua", "generate e+ e- > z a > mu+ mu- a"),
    ("ee_label_and", "define zh = z h\ngenerate e+ e- > zh > mu+ mu- b b~"),
    ("ee_ttx_wpbwmbx", "generate e+ e- > t t~ > w+ b w- b~"),
    ("ee_t_wpbwmbx", "generate e+ e- > t > w+ b w- b~"),
    ("ee_wpbwmbx", "generate e+ e- > w+ b w- b~"),
    ("uux_ttx_wpbwmbx", "generate u u~ > t t~ > w+ b w- b~"),
    ("pp_wp_epve", "generate p p > w+ > e+ ve"),
    ("pp_wm_epve", "generate p p > w- > e+ ve"),
    ("pp_z_epemj", "generate p p > z > e+ e- j"),
    ("pp_ttx_wpbwmbx", "generate p p > t t~ > w+ b w- b~"),
    # Forbidden s-channels.
    ("uux_wpbwmbx", "generate u u~ > w+ b w- b~"),
    ("uux_wpbwmbx_no_ttx", "generate u u~ > w+ b w- b~ $$ t t~"),
    ("uux_wpbwmbx_no_t", "generate u u~ > w+ b w- b~ $$ t"),
    ("uux_wpbwmbx_no_tx", "generate u u~ > w+ b w- b~ $$ t~"),
    ("ee_wpbwmbx_no_ttx", "generate e+ e- > w+ b w- b~ $$ t t~"),
    ("pp_wpbwmbx_no_ttx", "generate p p > w+ b w- b~ $$ t t~"),
    ("udx_epve_no_wp", "generate u d~ > e+ ve $$ w+"),
    ("udx_epve_no_wm", "generate u d~ > e+ ve $$ w-"),
    ("ee_mumu_no_z", "generate e+ e- > mu+ mu- $$ z"),
    ("ee_ee_no_a", "generate e+ e- > e+ e- $$ a"),
    ("both_restrictions", "generate e+ e- > t t~ > w+ b w- b~ $$ z"),
    # The automatic WEIGHTED search with a filter that empties the lowest order.
    ("uux_a_ddx", "generate u u~ > a > d d~"),
    ("uux_ddx_no_g", "generate u u~ > d d~ $$ g"),
    ("uux_ddx", "generate u u~ > d d~"),
    ("uux_z_epemg", "generate u u~ > z > e+ e- g"),
    # WEIGHTED constraints.
    ("weighted_le", "generate u u~ > d d~ WEIGHTED<=4"),
    ("weighted_eq", "generate u u~ > d d~ WEIGHTED=4"),
    ("weighted_too_low", "generate e+ e- > mu+ mu- WEIGHTED<=3"),
    ("weighted_pp", "generate p p > e+ e- j WEIGHTED<=5"),
    ("weighted_with_qcd", "generate p p > j j WEIGHTED<=4 QCD<=1"),
    # One initial particle: MadGraph's own reading, banked for decays.
    ("decay_t_bepve", "generate t > b e+ ve"),
    ("decay_t_bepve_no_wp", "generate t > b e+ ve $$ w+"),
    ("decay_t_bepve_no_t", "generate t > b e+ ve $$ t"),
    ("decay_t_wp_bepve", "generate t > w+ > b e+ ve"),
    ("decay_t_wm_bepve", "generate t > w- > b e+ ve"),
    ("decay_h_zz", "generate h > e+ e- mu+ mu-"),
    ("decay_h_zz_no_z", "generate h > e+ e- mu+ mu- $$ z"),
    ("decay_h_zz_no_h", "generate h > e+ e- mu+ mu- $$ h"),
    # The five-flavour switch on import.
    ("four_flavour_pp_ll", "import model sm\ngenerate p p > e+ e-"),
    ("five_flavour_pp_ll", "import model sm-no_b_mass\ngenerate p p > e+ e-"),
    ("five_flavour_pp_jj", "import model sm-no_b_mass\ngenerate p p > j j"),
    ("five_flavour_no_masses", "import model sm-no_masses\ngenerate p p > e+ e-"),
    ("five_flavour_zeromass_ckm", "import model sm-zeromass_ckm\ngenerate p p > e+ e-"),
    (
        "five_flavour_define_before",
        "define p = g u u~\nimport model sm-no_b_mass\ngenerate p p > e+ e-",
    ),
    (
        "five_flavour_define_after",
        "import model sm-no_b_mass\ndefine p = g u u~\ngenerate p p > e+ e-",
    ),
    (
        "five_flavour_define_through_p",
        "import model sm-no_b_mass\ndefine q = p / g\ngenerate q q > e+ e-",
    ),
    (
        "four_flavour_define_b",
        "define p = p b b~\nimport model sm\ngenerate p p > e+ e-",
    ),
    ("five_flavour_forbidden_j", "import model sm-no_b_mass\ngenerate u u~ > e+ e- g / j"),
    ("five_flavour_smeftsim", "import model @SMEFTSIM@-SMlimit_massless\ngenerate p p > e+ e-"),
]


def main():
    scratch = tempfile.mkdtemp(prefix="schannel_census_")
    try:
        smeftsim = os.path.join(scratch, "SMEFTsim_topU3l_MwScheme_UFO")
        shutil.copytree(_smeftsim, smeftsim)
        substitutions = {"@SMEFTSIM@": smeftsim}
        cases = []
        for name, card in CORPUS:
            result = census(card, substitutions)
            print(
                f"{name}: "
                + (
                    result["error"]
                    if "error" in result
                    else f"{sum(s['diagrams'] for s in result['subprocesses'])} diagrams in "
                    f"{len(result['subprocesses'])} subprocesses"
                ),
                file=sys.stderr,
            )
            cases.append({"name": name, "card": card, **result})
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    doc = {
        "metadata": {
            "generator": "validation/madgraph/dump_schannel_census.py",
            "madgraph": "research/refs/mg5amcnlo",
            "smeftsim": "validation/ufo/SMEFTsim_topU3l_MwScheme_UFO (as @SMEFTSIM@)",
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
