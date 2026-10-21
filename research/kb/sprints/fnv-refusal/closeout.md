---
type: Sprint Record
title: "Majorana and fermion-number-violating models are refused, not panicked on"
description: "Diagram generation refuses a Majorana fermion (external or propagator) and a fermion-number-violating vertex with a named error; before, the FNV case panicked and Majorana processes silently returned a σ."
closed: 2026-10-20
status: draft
tags: [majorana, fermion-flow, refusal, panic, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/61", title: "nsmith-/vibegraph#61"}
---
One PR ([nsmith-/vibegraph#61](https://github.com/nsmith-/vibegraph/pull/61))
closing `fermion-number-violation-panics-instead-of-refusing`. Sessions:
[F1](sessions/F1-report.md) (feature-dev, Opus) and
[HY1](sessions/HY1-report.md) (hygiene-dev, Sonnet).

## What was established

- **The worse failure was silent.** Before the fix only the FNV toy process
  panicked (at the `run.rs` site the survey hit). Both Majorana toy processes
  returned a σ with exit 0: `lt~ lt > nt nt` gave 9341 ± 9068 pb, read out of
  a neighbouring array. A survey card on a Majorana model that appeared to run
  may therefore have printed a wrong σ.
- **The check.** `Diagram::check_fermion_flow` runs on every diagram that
  survives the `$$` filter. It refuses a self-conjugate spin-½ particle on an
  external leg or a propagator (`ConvertError::MajoranaFermion`) and a vertex
  whose fermion pairing joins two particles or two antiparticles
  (`ConvertError::FermionNumberViolation`), reading all slots all-incoming by
  PDG sign. It refuses per process: the same models' Dirac-only processes
  integrate.
- **One continuity condition is what the evaluator needs.** A debug assertion
  checks every surviving fermion line has a particle end and an antiparticle
  end; the three panic sites need exactly that, and are now reachable only
  from hand-built diagrams. `helas/eval/layout.rs`'s bra/ket pairing matches
  both operands and panics with an invariant message instead of falling
  through (compile time, not the evaluation path).
- **Pinned by** six CLI tests on two toy UFOs (`vibegraph_toy_majorana_UFO`,
  `vibegraph_toy_fnv_UFO`, described in
  [toy UFO models](../../validation/toy-ufo-models.md)), with mutation
  evidence for the message kind and for the check's position after the `$$`
  veto. The survey's own cards (047, 049, 052–054, 061) were not rerun.

The manager re-ran the new CLI tests from the committed tree after F1 and HY1.

## Backlog

- Deleted: `fermion-number-violation-panics-instead-of-refusing`.
- Noted on `majorana-fermions-unsupported`: the propagator case that may be
  stricter than needed.
- Not filed separately: the CLI test-helper duplication (the same item filed
  on the cut-refusal PR, `cli-test-helpers-copied-per-file`, covers it); the
  byte-identical `object_library.py`/`function_library.py` across the four toy
  UFOs; a library-level test of `check_fermion_flow`; an unpaired fermion slot
  in a Lorentz structure (no model in reach writes one).
- Asked of the planner: rerun the survey's BSM cards on this branch on the
  CERN host.
