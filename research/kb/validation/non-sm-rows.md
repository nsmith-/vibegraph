---
type: Validation Gate
title: SMEFTsim and toy rows in the MadGraph oracle
description: "How non-SM rows are banked and gated: per-class restrict cards, order bounds in process strings, interactions.json, the primitive coverage table and the ee > tt~ NP<=1 capstone."
status: draft
tags: [smeftsim, toy-ufo, non-sm, oracle, coverage]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n35-v1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L676-L746", title: "Note 35 V1, SMEFTsim into the oracle pipeline"}
  - {id: n35-l2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L747-L787", title: "Note 35 L2, the SM-limit gate"}
  - {id: n35-c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L788-L841", title: "Note 35 C, the capstone"}
  - {id: n35-cov, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L842-L902", title: "Note 35 §5, coverage table and V1's corrections"}
  - {id: n35-v2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1113-L1190", title: "Note 35 V2, banked-layer hygiene"}
  - {id: n35-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1191-L1218", title: "Note 35 §7, decisions"}
  - {id: n35-101, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1281-L1358", title: "Note 35 §10.1, what the sprint leaves gated"}
  - {id: n35-107, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1494-L1515", title: "Note 35 §10.7, diagram-count cells"}
  - {id: n35-109, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1524-L1692", title: "Note 35 §10.9 (V3), sigma and samples for the non-SM rows"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml", title: "validation/manifest.toml (cell notes of the non-SM rows)"}
---
# SMEFTsim and toy rows in the MadGraph oracle

Twenty-two reference rows are generated against a UFO model other than
MadGraph's built-in `sm`: sixteen under the vendored
[SMEFTsim topU3l MwScheme UFO](../model/smeftsim-topu3l.md) and six under the two
authored [toy models](toy-ufo-models.md) (`vibegraph_toy_UFO`,
`vibegraph_toy_color_UFO`). They exist to put every Lorentz and colour primitive
a non-SM model can reach under a MadGraph-gated row, since no SM process reaches
`Epsilon`, a γ-chain with a momentum slash, `Gamma5` inside a chain, a cyclic
tensor four-fermion structure or a baryonic colour atom.

## How a row is banked

- **The model and card are manifest fields.** A row carries `model` (the
  repository-relative UFO directory) and `restrict`; the `.mg5` script imports
  `<model>-<restrict>`. `build.sh` copies the committed model into
  `output/models/` (the bundle leaves that directory out and restages it) and
  rewrites the repo-relative path to the work area's[^n35-v1].
- **One Wilson-coefficient class per row.** The cards
  `validation/madgraph/cards/smeft/restrict_vg_<class>.dat` are copies of
  `restrict_massless.dat` with one class of coefficients non-zero at the shipped
  values, so MadGraph and vibegraph prune vertices identically and a row compiles
  only the structures its class needs. Restriction semantics are
  [model/restriction-semantics](../model/restriction-semantics.md); the vendored
  copy is never edited.
- **Process strings carry their order bound.** Under SMEFTsim's `NP` hierarchy
  (99) the `WEIGHTED` default drops every NP diagram, so the generate line, the
  manifest's `mg_amplitude.process` and the banked table's `process` all carry
  `NP<=1`; two assertions hold table and manifest equal and their order bounds
  equal to the script's generate line. Until that was in place, every SMEFTsim
  amplitude comparison was measuring an SM subset against an `NP<=1`
  table[^n35-l2].
- **Interaction counts are a two-sided check.** Each script runs `display
  interactions`; `extract_interactions.py` reads MadGraph's count (after its
  per-coupling-order splitting and the restriction) from the `build.log` into the
  committed `validation/madgraph/interactions.json`, and `tests/smeftsim.rs`
  asserts this crate's split counts against it: 62 under `SMlimit_massless`, 913
  under `massless`, 154 under `vg_c4l`, 82 under `vg_c4q`. `tests/smeftsim.rs`
  also holds `GATED_ROWS` equal to the manifest's gated SMEFTsim rows.
- **The defaults are MadGraph's generated card.** A restrict card's non-zero
  values become the model's defaults and its zeros lock, which is what MadGraph
  writes into the generated `param_card.dat`.
  `restricted_defaults_are_madgraphs_generated_param_card` compares 421 external
  parameters over 13 gated rows against MadGraph's own cards at ≤ 1e-12. Before
  that, a card-less evaluation of a restricted model was silently its SM limit
  (the Wilson coefficients stayed at `parameters.py`'s zeros)[^n35-c]. MadGraph
  also fixes parameters a restrict card sets to exactly `1`; no card here does
  ([restrict-card-unit-values-not-fixed](../backlog/feature/restrict-card-unit-values-not-fixed.md)).

## Coverage: every primitive has a gated row

| row | process | card | primitives |
|---|---|---|---|
| `ee_to_mumu_smlimit` | `e+ e- > mu+ mu-` | `SMlimit_massless` | loader, MW scheme, splitting, zero-coupling pruning |
| `gg_to_ttx_smlimit` (+`_qcd2`) | `g g > t t~` (`QCD<=2`) | `SMlimit_massless` | the same with colour; `_qcd2` adds the SMHLOOP `g g h` diagram |
| `ee_to_ttx_smlimit` | `e+ e- > t t~` | `SMlimit_massless` | massive fermions under the loader |
| `bbx_to_h_identity` | `b b~ > h NP<=1` | `cbH` | bare `Identity` FFS |
| `gg_to_h_cpeven` / `_cpodd` | `g g > h NP<=1` | `cHG` / + `cHGtil` | `P(1,2)*P(2,1)` VVS; `Epsilon` VVS, its sign via interference |
| `ee_to_wpwm_cw` | `e+ e- > W+ W- NP<=1` | `cW` + `cWtil` | VVV with three momenta; `Epsilon` VVV |
| `gg_to_gg_cg` | `g g > g g NP<=1` | `cG` + `cGtil` | higher-derivative VVVV, `Epsilon` VVVV, `f·f·f` colour |
| `ee_to_ttx_dipole` | `e+ e- > t t~ NP<=1` | `ctW` + `ctB`, Re and Im (the `ctZ`/`ctA` directions) | momentum-slashed γ-chains; `Gamma5` in a chain |
| `ee_to_zh_smeft` | `e+ e- > Z h NP<=1` | `cHW` + `cHB` + `cHWB` + `cHDD` | derivative VVS, input-scheme shifts |
| `ee_to_mumu_4f` | `e+ e- > mu+ mu- NP<=1` | `cll1` + `cle` + `cee` | scalar and vector four-fermion, both pairings in one vertex |
| `uux_to_ttx_4f` | `u u~ > t t~ NP<=1` | `cQj11`, `cQj18`, `ctu1`, `ctu8` | four-quark `T·T` / `Identity·Identity` colour |
| `tata_to_ttx_tensor4f` | `ta+ ta- > t t~ NP<=1` | `cleQt3` with `MTA`, `ymtau` restored | cyclic tensor⊗tensor |
| `ee_to_ttx_smeft` | `e+ e- > t t~ NP<=1` | `massless` (all) | every class at once; the σ capstone |
| `wpwm_to_wpwmz_cw` | `W+ W- > W+ W- Z NP<=1` | `cW` + `cWtil` (the same card) | a five-vector vertex |

Card facts that shape the table[^n35-cov]: every cyclic tensor⊗tensor structure
reaches its vertex only through a lepton-Yukawa coupling, which
`restrict_massless` zeroes, so the tensor row needs τ beams with `MTA` and `ymtau`
restored; `restrict_massless`'s `SMEFTcpv` block is zero, so the CP-odd
coefficients are values chosen on the per-class cards, and **the capstone reaches
no `Epsilon` and no tensor four-fermion structure**; `Gamma5` inside a chain is
reached only through `ctWIm`/`ctBIm`. The toy rows carry what SMEFTsim cannot:
the `d` colour basis, literal `Sigma`, the Yukawa-only fermion line, baryonic
`Epsilon` and sextets ([toy-ufo-models](toy-ufo-models.md)).

## Cell status

Read the live state from `validation/manifest.toml`; at `6ccc6e4`:
- **gate**: every `diagrams` cell but `gg_to_gg_cg`'s; every `amplitudes` cell
  but three; `integrals` and `samples` on every row that has a banked σ and
  sample, except `gg_to_gg_cg`'s σ. Integrals budgets are sized from the
  reference's own error; each `rel_tol` is the measured five-seed spread with
  headroom and each budget ladder is flat ([sigma-gate](sigma-gate.md)).
- **`ee_to_wpwm_cw` amplitudes, info**: the linear level agrees exactly, and 47 of
  48 |M|² points agree below 1e-12, but one point sits at 2.08e-12 where the
  helicity sum cancels by a factor 875; enforcing it would mean loosening the
  budget for one point.
- **`ee_to_zh_smeft` amplitudes, info**: the `GC_303` seven-digit Fortran literal
  ([madgraph-defects](madgraph-defects.md)); this crate agrees with MadGraph's
  Python to 6.5e-15.
- **`wpwm_to_wpwmz_cw` amplitudes, info**: |M|² 2.79e1 after the vector-vertex
  sign fix of note 39, owned by O_W's five-vector and momentum-bearing contact
  structures ([wpwmz-cw-ow-five-vector-residual](../backlog/validation/wpwmz-cw-ow-five-vector-residual.md));
  its `integrals` and `samples` are uncovered because MadGraph chose `nhel = 1`
  for it, which this crate refuses: `nhel = 1` is in scope but not built
  ([nhel1-run-cards-refused](../backlog/feature/nhel1-run-cards-refused.md)).
- **`gg_to_gg_cg`**: `diagrams` info by convention (21 diagrams against
  MadGraph's 27 `NGRAPHS`, one `AMP()` per diagram and colour-ordered contact
  structure, as SM `gg_to_gg` sits at 4/6); `integrals` info, taken at MadGraph's
  own `dynamical_scale_choice = 3`, a converged −0.22% offset with no attribution
  ([gg-to-gg-cg-sigma-offset](../backlog/validation/gg-to-gg-cg-sigma-offset.md)).
- **2 → 1 rows** (`bbx_to_h_identity`, `gg_to_h_cp*`): `integrals` and `samples`
  uncovered; MadEvent has no volume to integrate and banked no σ or events.
- **Massive incoming legs** (`qqx_to_o8o8_toy_dcolor`, both `p3r3` rows) gate: the
  fixed-beam integrand puts each beam on its own mass shell with the Møller flux,
  and the samples gate's incoming-leg column compares the beams against the
  banked record at its printed precision.
- **`AQCDUP` on the six toy rows** is measured, not enforced: MadGraph injects an
  inconsistent `aS` into models that declare none.
- **`gg_to_ttx_smlimit_qcd2` and `gg_to_gg_cg`** are the suite's two
  scale-change fallbacks: SMEFTsim's effective `g g h` coupling and `O_G`'s
  four-gluon vertex are not monomials in `G`, so the model is re-evaluated per
  scale change; `SCALE_FALLBACK_ROWS` (`validate_sigma.rs`) asserts membership
  both ways.

## The capstone

`ee_to_ttx_smeft`, `e+ e- > t t~ NP<=1` at √s = 500 GeV under `restrict_massless`,
is the one banked σ whose every class is on. At 160000 × 8 it reads
2.222986 ± 6.565e-4 pb against MadGraph's 2.2223 ± 5.257e-4, pull +0.82; seven
seeds span pulls −0.80 to +1.22, the budget ladder is flat, and `rel_tol = 0.002`
is set from the seed spread. The gate is not blind to the SMEFT content: the same
process under `SMlimit_massless` gives 0.5496 pb, a factor 4.04 below. The CLI
path (`--ufo-dir`, the `-<restrict>` suffix, the artifact's model label and
digest) is pinned by the hermetic `cli_ufo_model.rs`[^n35-c].

## What these cells cannot see

- The `samples` KS and χ² columns compare normalised distributions of outgoing
  legs: blind to σ, and to the beams except through the incoming-leg column. Three
  rows once carried a 6–7% σ error and cleared the KS floor comfortably.
- A rounding both sides share is invisible to the coupling and amplitude gates.
- The σ gate cannot see amplitude residuals of 1e-12 or a 6e-11 derived-parameter
  spread, far below any budget's Monte-Carlo error.
- `NP^2==1` and other squared-order constraints are refused, not compared. They
  are in scope but not built
  ([squared-order-constraints-refused](../backlog/feature/squared-order-constraints-refused.md)).

[^n35-v1]: Note 35 V1.
[^n35-l2]: Note 35 L2.
[^n35-cov]: Note 35 §5 and V1's corrections (a)–(h).
[^n35-c]: Note 35 C, including F1's loader observation, corrected.
