---
type: Sprint
title: "Sprint H: CMS gridpack coverage"
description: "Lift the cheapest refusals of the 200-gridpack CMS survey (#71, campaign main-23d3a76-200-pc): colour-basis splits, near-degenerate groups, xpt* ≤ 0, pt_min_pdg, decay-chain duplicate subprocesses and massive incoming partons, each validated against MadGraph."
status: draft
active: false
tags: [cms, gridpacks, coverage, flavour-groups, cuts, decay-chains, proton, sprint-h]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: campaign, resource: "https://github.com/nsmith-/vibegraph/pull/71#issuecomment-6605096410", title: "Campaign main-23d3a76-200-pc on #71"}
---
## Goal and exit criteria

The campaign `main-23d3a76-200-pc` refuses 139 of 199 gridpacks. This sprint
turns the cheapest refusals the CERN host ranked into correct runs. It is done
when every claimed item's `closes_when` holds, each lift carries the MadGraph
oracle named below, and a CERN-host rerun of the affected gridpacks has been
compared against `main-23d3a76-200-pc`.

Out of scope: NLO `[QCD]`, Majorana and fermion-number violation, squared
orders, per-group cut indicators.

## Items, PRs and oracles

| # | Item | Gridpacks | PR (branch) | Oracle |
|---|---|---|---|---|
| 1 | `crossing-pair-colour-basis-differs-refused`, plus the "colour flows cannot be paired up" refusal | 15 + 4 | `colour-split-groups` (written on the CERN host) | A test that fails before the change; mutation evidence; per-subprocess event colour labels on one HT-binned γ/QCD card; the banked colour gates unchanged. |
| 2 | `ckm-flavour-groups-near-degenerate-refused` | 5 | `sprinth-near-degenerate` | Needs the user's decision first. A test on the γ+jets pair that shows the 8e-7 split vanishes with the CKM phase or the W width at zero (CP violation), and one on the 1.5e-10 VBS_WPMZTo4Q pair. |
| 3 | `xpt-nonpositive-refused` (new) | 2 | `sprinth-cuts` | A refusal test: xpt* ≤ 0 compiles to the default cuts; a positive value is still refused. |
| 4 | `pt-min-pdg-refused` (new) | 2 | `sprinth-cuts` | Per-event pT of the listed PDG in written events; a local MadGraph σ on an SM proxy (`p p > z j` at `{23: 100}`), banked on the M3 only if it becomes a manifest row. |
| 5 | `decay-chain-duplicate-subprocess-vs-madgraph` | 11 | `sprinth-decay-duplicate` | First MadGraph's own behaviour (`combine_decay_chain_processes`, measured by a local `output` and `launch`); then the diagram census against MadGraph's matrix elements and a σ row on a small `Zto2Q`-like card. |
| 6 | `massive-incoming-partons-refused` | 14 | `sprinth-massive-incoming` | Per-helicity amplitudes at massive incoming momenta against MadGraph standalone, and local MadGraph σ on a b-mass and a c-mass case from the survey. |
| 7 | `absent-run-card-maxjetflavor-ignores-beam-flavours` (high; filed by the vibegraph manager) | – | `sprinth-maxjetflavor`, after the colour-split PR | A test pins 5 for `p p > j j` under sm-no_b_mass and 4 under sm with no run card; an explicit value still wins; checked against `banner.py` `create_default_for_process`. |
| 8 | `five-flavour-ppjjj-refused-cut-indicator-differs` (high; filed by the vibegraph manager) | – | with item 2's grouping work, after the colour-split PR | `p p > j j j` under sm-no_b_mass at maxjetflavor 4 integrates; σ against local MadGraph by the multi-seed criterion. |
| 9 | `explicit-initial-state-ordering-counted-twice` (high; found by H-M1) | – | `sprinth-mirror` | Local MadGraph σ on explicit single-ordering and both-ordering cards, massless and massive, ≥5 seeds per side. |

## Sessions

| Session | Agent | Work |
|---|---|---|
| H-G0 | validation-dev | Why the charge-conjugate colour bases differ, and why the Singlet-model flows cannot be paired. Read-only; feeds the review of item 1. |
| H-G1 | manager, then hygiene-dev | Review of `colour-split-groups` when it is pushed. |
| H-C1 | feature-dev | Items 3 and 4. |
| H-N1 | validation-dev | Item 2, after the decision. |
| H-D1 | validation-dev | Item 5: MadGraph's behaviour from source plus local MadGraph `output` and `launch`; then the implementation, per the decision. |
| H-M1 | feature-dev | Item 6: MadGraph's prescription (`mom2cx` at `genps.f:411`, flux `1/(2√λ(ŝ, m1², m2²))` at `genps.f:427`), ported. |
| H-X | CERN host | Close-out rerun of the affected gridpacks. |

## What is known before the sprint (manager, 2026-11-04)

- `xpt*`: every MadGraph read is guarded by `> 0` or `max(·, 0)`
  (`cuts.f:1007`; `setcuts.f:540–660`, `752–763`), for all four of
  `xptj`, `xptb`, `xpta`, `xptl`.
- Decay chains: `combine_decay_chain_processes` (`helas_objects.py:5427`)
  merges two combinations only when their `IdentifyMETag`s are equal.
  Combinations with different diagram sets, as in `Zto2Q`, become separate
  matrix elements, which suggests MadGraph sums them incoherently. **Inferred
  from source**; a local `output` measures it.
- In the code, the decay-chain refusal is `chain.rs:210`, which reuses the
  "produced by two process lines" message with the same line twice.
- `setcuts.f:678–687` indexes `idup` with the `pdg_cut` loop index; H-C1
  checks whether that `smin` hint is a MadGraph defect that reaches a weight.
