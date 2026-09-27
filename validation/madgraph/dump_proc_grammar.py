#!/usr/bin/env python3
"""Run a corpus of proc cards through MadGraph's own parser and bank the result.

Every card is fed, line by line, to a ``MasterCmd`` exactly as ``mg5_aMC`` feeds
a proc card (``exec_cmd(line, precmd=True)``, so comments, ``;`` and ``\\``
continuations take MadGraph's path), with diagram generation stubbed out: the
parser, the command semantics (``generate`` resetting, ``add process``
appending, process numbering, ``define``) and every error MadGraph raises while
reading a line are MadGraph's own, but no amplitude is built. What is banked
per card is either MadGraph's error or the ``ProcessDefinition`` of every
process it holds afterwards, field by field and decay chains recursively.

Read back by the hermetic test ``vibegraph-lib/tests/proc_grammar_oracle.rs``,
which parses the same cards, resolves them against the same SM, and compares.

Two questions the parser alone cannot answer are measured with diagram
generation switched back on (lepton-collider processes, where it is fast) and
recorded under ``metadata.measurements``:

  duplicates         which ``add process`` combinations MadGraph refuses as a
                     ``Duplicate process``, and which it generates twice;
  decay_assignment   how a decay chain's decays are assigned to several
                     identical legs of the core, read off the combined matrix
                     elements ``HelasDecayChainProcess`` builds.

Usage:
  pixi run dump-proc-grammar
  python validation/madgraph/dump_proc_grammar.py

Reads the pinned ``research/refs/mg5amcnlo`` submodule (pure Python; no
Fortran, no ``pixi -e madgraph`` environment); ``MG5AMCNLO_PATH`` overrides it.
"""

import contextlib
import io
import json
import logging
import os
import sys

_here = os.path.dirname(os.path.abspath(__file__))
_out = os.path.join(_here, "proc_grammar.json")
_root = os.environ.get("MG5AMCNLO_PATH") or os.path.join(
    _here, "..", "..", "research", "refs", "mg5amcnlo"
)
sys.path.insert(0, os.path.abspath(_root))
logging.disable(logging.WARNING)

with contextlib.redirect_stdout(io.StringIO()):
    import madgraph.core.diagram_generation as diagram_generation  # noqa: E402
    import madgraph.core.helas_objects as helas_objects  # noqa: E402
    import madgraph.interface.madgraph_interface as madgraph_interface  # noqa: E402
    import madgraph.interface.master_interface as master_interface  # noqa: E402

REAL_MULTIPROCESS = madgraph_interface.diagram_generation.MultiProcess


class _StubAmplitude(diagram_generation.Amplitude):
    """An amplitude that only knows its process: enough for ``do_add``'s own
    bookkeeping (the initial-state count it compares), never equal to another,
    so no duplicate check fires on a stub."""

    def __init__(self, procdef):
        super().__init__()
        self._procdef = procdef

    def get_ninitial(self):
        return self._procdef.get_ninitial()

    def get_number_of_diagrams(self):
        return 0

    def __eq__(self, other):
        return self is other

    def __ne__(self, other):
        return self is not other


class _StubMultiProcess(dict):
    def __init__(self, procdef, **_):
        super().__init__(amplitudes=[_StubAmplitude(procdef)])

    def get(self, key):
        return self[key]


def _new_cmd():
    with contextlib.redirect_stdout(io.StringIO()):
        cmd = master_interface.MasterCmd()
    # A card MadGraph finds ambiguous (`z{T} z`) asks before continuing; a batch
    # run has nobody to answer, and the answer a script gets is the default.
    cmd.ask = lambda *args, **kwargs: "no"
    return cmd


def _run(cmd, card):
    """Feed a card as ``import_command_file`` does.

    A single process line with a ``[...]`` is read by ``extract_process``
    alone: ``MasterCmd`` would hand it to the NLO interface, and the LO
    interface's ``do_add`` would go on to loop generation, neither of which is
    the grammar this oracle banks. What ``extract_process`` makes of ``[...]``
    is the datum. (A decay-chain line with ``[...]`` still goes through the LO
    ``do_add``, which refuses it before reading it.)
    """
    lo = madgraph_interface.MadGraphCmd
    with contextlib.redirect_stdout(io.StringIO()):
        for line in card.split("\n"):
            line = line.strip()
            if not line:
                continue
            verb, _, rest = line.partition(" ")
            if "[" in line and verb in ("generate", "add"):
                if not cmd._curr_model:
                    cmd.exec_cmd("import model sm", precmd=True, printcmd=False)
                cmd.history.append(line)
                text = rest if verb == "generate" else rest.partition(" ")[2]
                if "," in text:
                    lo.do_add(cmd, "process " + text)
                    continue
                if verb == "generate":
                    cmd.clean_process()
                cmd._curr_proc_defs.append(
                    cmd.extract_process(text, proc_number=len(cmd._curr_proc_defs) + 1)
                )
            else:
                cmd.exec_cmd(line, precmd=True, printcmd=False)


def _dump(pd):
    legs = []
    for leg in pd["legs"]:
        legs.append(
            {
                "ids": list(leg["ids"]),
                "state": bool(leg["state"]),
                "polarization": list(leg["polarization"]),
                "tagged": bool(leg["is_tagged"]) if "is_tagged" in leg else False,
            }
        )
    required = pd["required_s_channels"]
    return {
        "id": pd["id"],
        "legs": legs,
        "required_s_channels": [list(x) for x in required],
        "forbidden_particles": list(pd["forbidden_particles"]),
        "forbidden_s_channels": list(pd["forbidden_s_channels"]),
        "forbidden_onsh_s_channels": list(pd["forbidden_onsh_s_channels"]),
        "orders": dict(pd["orders"]),
        "squared_orders": dict(pd["squared_orders"]),
        "sqorders_types": dict(pd["sqorders_types"]),
        "constrained_orders": {k: list(v) for k, v in pd["constrained_orders"].items()},
        "overall_orders": dict(pd["overall_orders"]),
        "perturbation_couplings": list(pd["perturbation_couplings"]),
        "NLO_mode": pd["NLO_mode"],
        "has_born": bool(pd["has_born"]),
        "decay_chains": [_dump(d) for d in pd["decay_chains"]],
    }


def parse_case(card):
    madgraph_interface.diagram_generation.MultiProcess = _StubMultiProcess
    try:
        cmd = _new_cmd()
        _run(cmd, card)
        return {"processes": [_dump(pd) for pd in cmd._curr_proc_defs]}
    except Exception as e:  # MadGraph's refusal is the datum
        return {"error": f"{type(e).__name__}: {str(e).strip().splitlines()[0]}"}
    finally:
        madgraph_interface.diagram_generation.MultiProcess = REAL_MULTIPROCESS


def _generate(card):
    cmd = _new_cmd()
    try:
        _run(cmd, card)
    except Exception as e:
        return None, f"{type(e).__name__}: {str(e).strip().splitlines()[0]}"
    return cmd, None


def measure_duplicates():
    out = []
    for card in DUPLICATE_CARDS:
        cmd, err = _generate(card)
        if err:
            outcome = err
        else:
            outcome = "; ".join(
                amp.nice_string_processes().replace("Process: ", "")
                for amp in cmd._curr_amps
            )
        out.append({"card": card, "outcome": outcome})
    return out


def measure_decay_assignment():
    out = []
    for card in DECAY_CARDS:
        cmd, err = _generate(card)
        if err:
            out.append({"card": card, "outcome": err})
            continue
        combined = []
        for amp in cmd._curr_amps:
            with contextlib.redirect_stdout(io.StringIO()):
                mes = helas_objects.HelasDecayChainProcess(amp).combine_decay_chain_processes()
            for me in mes:
                for p in me.get("processes"):
                    combined.append(
                        " | ".join(
                            s.strip()
                            for s in p.nice_string().replace("Process: ", "").splitlines()
                        )
                    )
        out.append({"card": card, "outcome": combined})
    return out


# ── The corpus ────────────────────────────────────────────────────────────────
#
# (name, card). A card without an `import model` gets MadGraph's default, the
# SM, which is what the Rust side resolves against too.

CORPUS = [
    # Legs and spacing.
    ("plain", "import model sm\ngenerate e+ e- > mu+ mu-"),
    ("hadronic", "generate p p > e+ e-"),
    ("spacing_extra_blanks", "generate    e+   e-  >  mu+   mu-"),
    ("spacing_no_blank_before_gt", "generate p p> e+ e-"),
    ("spacing_no_blank_after_gt", "generate p p >e+ e-"),
    ("spacing_no_blanks_around_gt", "generate p p>e+ e-"),
    ("spacing_required_unspaced", "generate e+ e- > z>mu+ mu-"),
    ("pdg_codes", "generate 11 -11 > 13 -13"),
    ("pdg_codes_two_digit", "generate 21 21 > 6 -6"),
    ("pdg_code_single_digit", "generate 2 -2 > 11 -11"),
    ("pdg_code_unknown", "generate e+ e- > 99 -99"),
    ("repeat_count_particle", "generate e+ e- > 2e+ 2e-"),
    ("repeat_count_label", "generate e+ e- > 2j"),
    ("repeat_count_three", "generate p p > 3j"),
    ("case_insensitive_names", "generate E+ E- > MU+ MU-"),
    ("case_mixed_names", "generate e+ e- > Z H"),
    ("unknown_particle", "generate e+ e- > mu+ foo"),
    ("no_final_state", "generate e+ e- >"),
    ("no_separator", "generate e+ e-"),
    ("three_separators", "generate e+ e- > z > a > mu+ mu-"),
    ("decay_process", "generate t > w+ b"),
    # Process numbers.
    ("tag", "generate e+ e- > mu+ mu- @3"),
    ("tag_then_orders", "generate p p > j j @1 QED=0"),
    ("tag_unspaced", "generate e+ e- > mu+ mu-@2"),
    ("tag_spaced", "generate e+ e- > mu+ mu- @ 2"),
    ("tag_after_order_unspaced", "generate e+ e- > mu+ mu- QED=2@2"),
    ("tag_orders_loop", "generate p p > e+ e- QED=2 [real=QCD] @4"),
    # Coupling orders.
    ("order_le", "generate e+ e- > mu+ mu- QED<=2"),
    ("order_spaced", "generate e+ e- > mu+ mu- QED = 2 QCD =0"),
    ("order_exact", "generate e+ e- > mu+ mu- QED==2"),
    ("order_gt", "generate e+ e- > mu+ mu- QED>1"),
    ("order_gt_spaced", "generate e+ e- > mu+ mu- QCD > 2"),
    ("order_ge", "generate e+ e- > mu+ mu- QED>=2"),
    ("order_ne", "generate e+ e- > mu+ mu- QED!=2"),
    ("order_lt", "generate e+ e- > mu+ mu- QED<2"),
    ("order_strict", "generate e+ e- > mu+ mu- QED===2"),
    ("order_repeated_leftmost_wins", "generate p p > t t~ QCD=2 QCD=1"),
    ("order_two_negative", "generate e+ e- > mu+ mu- QED=-1 QCD=-1"),
    ("order_before_slash", "generate p p > e+ e- j QCD<=1 / t"),
    ("order_unknown", "generate e+ e- > mu+ mu- FOO=2"),
    ("order_alias_ew", "generate e+ e- > mu+ mu- EW=2"),
    ("order_alias_aew", "generate e+ e- > mu+ mu- aEW=1"),
    ("order_alias_as", "generate p p > t t~ aS=1"),
    ("order_weighted", "generate e+ e- > mu+ mu- WEIGHTED<=4"),
    ("squared_le", "generate e+ e- > mu+ mu- QED^2<=4"),
    ("squared_eq_reads_le", "generate e+ e- > mu+ mu- QED^2=4"),
    ("squared_exact_with_amp", "generate e+ e- > mu+ mu- QED^2==4 QCD=0"),
    ("squared_gt", "generate e+ e- > mu+ mu- QED^2>2"),
    ("required_digit_is_not_an_order", "generate p p > z > 13 -13"),
    # Loop specifications.
    ("loop_bare", "generate p p > e+ e- [QCD]"),
    ("loop_real", "generate p p > e+ e- [real=QCD]"),
    ("loop_real_unspaced", "generate p p > e+ e-[real=QCD]"),
    ("loop_tree", "generate p p > e+ e- [tree=QCD]"),
    ("loop_virt", "generate p p > e+ e- [virt=QCD]"),
    ("loop_bad_mode", "generate p p > e+ e- [foo=QCD]"),
    ("loop_real_constrained", "generate p p > e+ e- [real=QCD] QED==2"),
    # Restrictions.
    ("forbidden", "generate p p > e+ e- / z h"),
    ("forbidden_unspaced", "generate e+ e- > mu+ mu-/z"),
    ("forbidden_label", "generate p p > e+ e- j / p"),
    ("forbidden_pdg", "generate p p > e+ e- / 23"),
    ("forbidden_s", "generate p p > e+ e- $$ z"),
    ("forbidden_s_unspaced", "generate e+ e- > mu+ mu-$$z"),
    ("forbidden_s_split", "generate e+ e- > mu+ mu- $ $ z"),
    ("forbidden_onsh", "generate p p > e+ e- $ z"),
    ("forbidden_onsh_unspaced", "generate e+ e- > mu+ mu- $z"),
    ("onsh_then_s", "generate p p > e+ e- $ z $$ w+"),
    ("s_then_slash", "generate p p > e+ e- $$ w+ / h"),
    ("slash_then_onsh", "generate p p > e+ e- / h $ z"),
    ("onsh_then_slash", "generate e+ e- > mu+ mu- $ z / h"),
    ("slash_s_onsh", "generate p p > e+ e- / h $$ w+ $ z"),
    ("s_then_onsh", "generate p p > e+ e- $$ w+ $ z"),
    ("forbidden_s_or", "generate p p > e+ e- $$ z | a"),
    # Required s-channels.
    ("required", "generate e+ e- > z > mu+ mu-"),
    ("required_hadronic", "generate p p > z > e+ e-"),
    ("required_and", "generate e+ e- > z h > mu+ mu- b b~"),
    ("required_or_inline", "generate e+ e- > z | a > mu+ mu-"),
    ("required_or_label", "define v = z | a\ngenerate e+ e- > v > mu+ mu-"),
    ("required_or_label_and", "define v = z | a\ngenerate e+ e- > v h > mu+ mu- h"),
    ("required_plain_label", "generate e+ e- > l+ > e+ e-"),
    ("required_repeated", "generate e+ e- > z z > mu+ mu- mu+ mu-"),
    ("required_pdg", "generate e+ e- > 23 > mu+ mu-"),
    ("required_orientation", "generate p p > w- > e+ ve"),
    ("or_label_as_leg", "define v = z | a\ngenerate e+ e- > v mu+ mu-"),
    ("or_label_forbidden", "define v = z | a\ngenerate e+ e- > mu+ mu- / v"),
    # Multiparticle labels.
    ("define", "define q = u d u~ d~\ngenerate q q > e+ e-"),
    ("define_without_equals", "define q u d\ngenerate q q > e+ e-"),
    ("define_except", "define lep = e+ e- / e-\ngenerate e+ e- > lep e-"),
    ("define_pdg_members", "define x = 11 13\ngenerate e+ e- > x x"),
    ("define_redefine_p", "define p = p b b~\ngenerate p p > e+ e-"),
    ("define_case", "define MyP = u d\ngenerate myp myp > e+ e-"),
    ("define_j", "define j = g u d\ngenerate p p > j j"),
    ("define_after_use", "generate q q > e+ e-\ndefine q = u d"),
    ("define_empty", "define x\ngenerate e+ e- > mu+ mu-"),
    ("define_particle_name", "define e+ = u d\ngenerate e+ e- > mu+ mu-"),
    # Polarizations and tags.
    ("pol_vector", "generate e+ e- > w+{0} w-{T}"),
    ("pol_left_right", "generate e+ e- > z{L} z{R}"),
    ("pol_fermion", "generate e+ e- > mu+{L} mu-{R}"),
    ("pol_fermion_zero", "generate e+ e- > mu+{0} mu-"),
    ("pol_signed", "generate e+ e- > w+{+1} w-{-1}"),
    ("pol_auxiliary", "generate e+ e- > z{A} a"),
    ("pol_massless_zero", "generate e+ e- > a{0} a"),
    ("pol_no_blank_after", "generate e+ e- > z{T}z a"),
    ("pol_invalid", "generate e+ e- > w+{X} w-"),
    ("pol_hadronic", "generate p p > z{0} j"),
    ("tag_initial_tree", "generate !a! e- > e- a"),
    # Decay chains.
    ("chain", "generate p p > t t~, t > w+ b"),
    (
        "chain_nested",
        "generate p p > t t~, (t > w+ b, w+ > e+ ve), (t~ > w- b~, w- > e- ve~)",
    ),
    (
        "chain_tthh",
        "generate p p > t t~ h h, (t > w+ b, w+ > j j), (t~ > w- b~, w- > l- vl~)",
    ),
    ("chain_zz_one_decay", "generate e+ e- > z z, z > e+ e-"),
    ("chain_zz_two_decays", "generate e+ e- > z z, z > e+ e-, z > mu+ mu-"),
    ("chain_overall_orders", "generate p p > t t~, t > w+ b @2 QED=2"),
    ("chain_tag_on_core", "generate p p > t t~ @2, t > w+ b"),
    ("chain_loop", "generate p p > t t~, t > w+ b [QCD]"),
    ("chain_unbalanced", "generate p p > t t~, (t > w+ b"),
    ("chain_core_exact_order", "generate p p > t t~ QED==0, t > w+ b"),
    ("chain_decay_exact_order", "generate p p > t t~, t > w+ b QED==1"),
    ("chain_parenthesis_without_comma", "generate p p > t t~, (t > w+ b), t~ > w- b~"),
    ("chain_decay_not_in_core", "generate e+ e- > z z, h > b b~"),
    # Commands.
    ("add_process", "generate e+ e- > mu+ mu-\nadd process e+ e- > ta+ ta-"),
    (
        "numbering",
        "generate e+ e- > mu+ mu-\nadd process e+ e- > ta+ ta- @5\nadd process e+ e- > e+ e-",
    ),
    (
        "generate_resets",
        "generate e+ e- > mu+ mu-\ngenerate e+ e- > ta+ ta-\nadd process e+ e- > e+ e-",
    ),
    ("import_resets", "generate e+ e- > mu+ mu-\nimport model sm\nadd process e+ e- > a a"),
    ("add_without_generate", "add process e+ e- > mu+ mu-"),
    ("add_nothing", "add process"),
    ("add_no_separator", "add process e+ e-"),
    ("add_misspelt", "add proces e+ e- > mu+ mu-"),
    (
        "add_no_warning_duplicate",
        "generate e+ e- > mu+ mu-\nadd process e+ e- > mu+ mu- --no_warning=duplicate",
    ),
    ("add_identical", "generate e+ e- > mu+ mu-\nadd process e+ e- > mu+ mu-"),
    ("mixed_initial_states", "generate e+ e- > mu+ mu-\nadd process t > w+ b"),
    ("comment", "generate e+ e- > mu+ mu- # a comment"),
    ("semicolon", "define x = u d ; generate x x > e+ e-"),
    ("continuation", "generate e+ e- > \\\nmu+ mu-"),
]

DUPLICATE_CARDS = [
    "import model sm\ngenerate p p > e+ e-\nadd process u u~ > e+ e-",
    "import model sm\ngenerate e+ e- > mu+ mu-\nadd process e+ e- > mu+ mu-",
    "import model sm\ngenerate e+ e- > mu+ mu- @1\nadd process e+ e- > mu+ mu- @2",
    "import model sm\ngenerate e+ e- > mu+ mu- @1\nadd process e+ e- > mu+ mu- @1",
    "import model sm\ngenerate e+ e- > mu+ mu- @1\n"
    "add process e+ e- > mu+ mu- @1 --no_warning=duplicate",
    "import model sm\ngenerate p p > e+ e- @1\nadd process u u~ > e+ e- @1",
    "import model sm\ngenerate e+ e- > mu+ mu-\nadd process e- e+ > mu+ mu-",
    "import model sm\ngenerate e+ e- > mu+ mu- EW=0",
]

DECAY_CARDS = [
    "import model sm\ngenerate e+ e- > z z, z > e+ e-",
    "import model sm\ngenerate e+ e- > z z, z > e+ e-, z > mu+ mu-",
    "import model sm\ngenerate e+ e- > z z, z > e+ e-, z > mu+ mu-, z > ta+ ta-",
    "import model sm\ngenerate e+ e- > z z, z > l+ l-",
    "import model sm\ngenerate e+ e- > w+ w-, w+ > e+ ve",
    "import model sm\ngenerate e+ e- > z z, h > b b~",
]


def main():
    cases = []
    for name, card in CORPUS:
        cases.append({"name": name, "card": card, **parse_case(card)})
    doc = {
        "metadata": {
            "generator": "validation/madgraph/dump_proc_grammar.py",
            "madgraph": "research/refs/mg5amcnlo",
            "model": "sm",
            "measurements": {
                "duplicates": measure_duplicates(),
                "decay_assignment": measure_decay_assignment(),
            },
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
