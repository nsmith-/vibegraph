---
type: Caveat
title: MadGraph defects found by this project
description: "Register of MadGraph defects met while validating: where each sits, whether it changes a weight on a card we support, how vibegraph handles it, and its upstream status."
status: draft
tags: [madgraph, defects, reference, upstream-report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n07-tables, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L43-L227", title: "Note 07, weaknesses and bug tables (rows marked found here)"}
  - {id: n07-aqcdup, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L367-L415", title: "Note 07, AQCDUP truncated pi and rambo.py"}
  - {id: n07-p1d, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L439-L538", title: "Note 07, ALOHA P1D flipped-fermion veto, with upstream draft"}
  - {id: n22-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/22-dynamical-scales-plan.md#L339-L357", title: "Note 22 close-out headline"}
  - {id: n27-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L44-L211", title: "Note 27 B1, the h to tau tau pole bin and get_channel_cut"}
  - {id: n29-d, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L3324-L3770", title: "Note 29 chain D measurements"}
  - {id: n35-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L334-L431", title: "Note 35 E1 (GC_303 literal rounding)"}
  - {id: n36-b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L413-L457", title: "Note 36 B5, the coupling oracle"}
  - {id: n36-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L567-L650", title: "Note 36 B3 (genps.f uninitialised t)"}
  - {id: n36-b8, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L709-L749", title: "Note 36 7.1 B8 (injected aS)"}
  - {id: n38-s3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L802-L881", title: "Note 38 S3, $ as the pointwise integrand"}
  - {id: n38-z1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L1332-L1429", title: "Note 38 8.1 Z1 (patched $ t t~ reference)"}
  - {id: n41-15, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L191-L205", title: "Note 41 1.5, MadGraph defects met on the way"}
  - {id: n41-mlm, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1414", title: "Note 41 M3, D2 and R1"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2916-L3489", title: "Note 41 Z close-out"}
  - {id: rw-results, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/reweight-vs-madgraph-results.md#L32-L49", title: "Reweighting against MadGraph, what is measured"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/unwgt.f#L752-L761", title: "MadGraph unwgt.f, SCALUP and the truncated pi"}
  - {id: mg-aloha, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/aloha/create_aloha.py#L527-L530", title: "MadGraph create_aloha.py, the 1D numerator (flip at L262-L263)"}
  - {id: mg-export, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/export_v4.py#L7076", title: "MadGraph export_v4.py, aS injection"}
  - {id: mg-genps, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/genps.f#L1817", title: "MadGraph genps.f, get_channel_cut"}
  - {id: mg-286feb8, resource: "https://github.com/mg5amcnlo/mg5amcnlo/commit/286feb8e606a4e55951f6ea10ea0e3d145213b13", title: "mg5amcnlo commit 286feb8e, change sde_strategy2 to avoid negative weights"}
  - {id: mg-rambo, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/rambo.py#L218", title: "MadGraph rambo.py overflow check"}
---
# MadGraph defects found by this project

Defects in MadGraph5_aMC@NLO that this project met first-hand while validating.
MadGraph's own release-note bug history is in
[the code-quality review](../references/codebases/madgraph5-code-quality-review.md).
Treatment follows [the defect policy](madgraph-defect-policy.md): a defect that
changes a weight on a card we support is reproduced bug-for-bug with a comment
naming it, or refused, never silently "fixed". Lines are at the pinned tree
`b7687064` (3.7.1) unless stated.

**Upstream status: nothing filed.** Report drafts for the AQCDUP truncation,
the ALOHA `P1D` veto and the grouped first call are in note 07's appendix, drafted
and unfiled; filing is the user's step
([madgraph-defect-reports-unfiled](../backlog/validation/madgraph-defect-reports-unfiled.md)).
The six MLM-path defects have no draft
([mlm-madgraph-defects-undrafted](../backlog/validation/mlm-madgraph-defects-undrafted.md)).

| defect | where | changes a compared weight? | handling |
|---|---|---|---|
| `AQCDUP`/`AQEDUP` with π truncated to 8 digits | `unwgt.f:760-761` | the record field, +1.7e-8 | truncation reproduced before comparing |
| `$` never vetoes a flipped-slot fermion propagator | `create_aloha.py:262-263, 527-530` | σ of `$ t t~` 37% high | reference from patched ALOHA |
| grouped first `setclscales` on the unpermuted point (H1) | `super_auto_dsig_group_v4.inc:842` | process-dependent | registered deviation, [own concept](madgraph-permuted-first-call.md) |
| `aS` injected into a model with none, inconsistent with `G` | `export_v4.py:7076` | `AQCDUP` on six toy rows | measured, not enforced |
| UFO literal printed at seven digits in Fortran | Fortran model writer | `GC_303` by 1.2e-8 | Python `model_reader` is the arbiter |
| `t` read uninitialised in `get_channel_cut` | `genps.f`, branch at `:1938` | unreachable here | `tmin_for_channel ≠ -1` refused |
| 3.5.x `get_channel_cut` dimensionally wrong | 3.5.x `genps.f` | yes, at narrow poles | references come from 3.7.1 |
| 3.5.7 applied a PDF set's `αs(M_Z)` at `lpp = 0` | 3.5.7 parameter card | partonic σ by `0.920ⁿ` | cuts across it not comparable |
| six MLM-path defects | note 41 §1.5 | at most one, refused | table below |
| reweight module keeps one hypothesis per card | `reweight_interface.py` | the reweight oracle | one hypothesis per work area |
| `rambo.py` overflow warning never fires | `rambo.py:218` | no | none needed |

**`AQCDUP` with a truncated π.** `unwgt.f` writes
`aaqcd = g*g/4d0/3.1415926d0` (and `aaqed` alike) while `g = √(4π·αs)` used full
π, so the field is `αs·(1 + 1.7e-8)`[^mg-unwgt]. The bias is applied before
printing, one-directional, and about a sixth of the last printed digit: it moves
the rounding of roughly one event in twenty. It was found by replaying banked
events; modelling it took exact digit agreement from about 95% to 100% of
events. The `AQCDUP` replay gates on reproducing the printed digits, which is what
exposed it; never read the field as `αs` itself[^n07-aqcdup]. Note 07 quotes
`:694-695`, the 3.5.x numbering.

**`$` and the flipped fermion slot (`FFV2P1D_1`).** The `$` veto multiplies each
marked propagator by a step function vanishing on its Breit–Wigner window,
written once for every spin as
`theta_functionr( (P(-1,id)**2 -(Mass(id)-BWCUTOFF*Width(id))**2 ) *( P(-1,id)**2 - … ),1,0)`.
For a fermion built from a vertex's second spinor slot, the momentum flip
`re.sub(r'\b(P|PSlash)\(', r'-\1(', expr)` turns `P(-1,id)**2` into `-(p²)`, the
argument is never negative, and the line is never zeroed[^mg-aloha]. Writing the
square as a product fixes it; only `FFV2P1D_1` changes. By reading, untested: a
UFO custom propagator written with `**2` for a fermion would flip the same way.
On `u u~ > w+ b w- b~ $ t t~` at 500 GeV MadEvent zeroes the `t~` window and not
the `t` one and reads 0.06811 pb against vibegraph's 0.04982 (37% high); rows
vetoing a vector boson agree within seed errors[^n38-s3]. The reference is made
with `validation/madgraph/patches/aloha-p1d-flipped-fermion.patch`, applied by
`gen_onshell_veto.sh` to a copy of the pinned tree (row `uu_tt`; the unpatched
reading stays as `uu_tt_mg371`)[^n07-p1d]. With the patch, MadGraph's standalone
`SMATRIX` equals vibegraph's zeroed amplitude to 1e-12, yet MadEvent still sits
about 2% low and drifts with budget[^n38-z1]; that residual is not this defect
([uux-wbwb-onshell-veto-tt-2pct-off](../backlog/validation/uux-wbwb-onshell-veto-tt-2pct-off.md)).

**`aS` injected into a model that declares none.** The Fortran exporter logs
`CRITICAL: aS not define as external parameter adding it!` and appends
`aS = 0.138` and `G = 4.1643`; `G = 2√(π·aS)` gives 1.317 for 0.138, and 4.1643 is
`aS = 1.380`[^mg-export]. `setrun.f` runs `αs(M_Z)` from `G`, so such a model's
events carry an `AQCDUP` from a strong coupling of 1.38 (about 0.43 at
250–500 GeV) in a model with no strong interaction. On the six toy rows
(`ll_to_qqx_toy_*`, `qqx_to_o8o8_toy_dcolor`, `p3r3_to_p3r3_toy_*`) the field is
measured and reported, with the cause asserted (`alpha_s_source()` absent exactly
on `UNDECLARED_ALPHA_S_RUNS`); `validate_alphas` replays MadGraph's events from
`G`'s 1.3799843265950287, never ours, and `SCALUP` gates[^n36-b8].

**Seven-digit UFO literals.** `gHza`'s `0.4583333333333333` (11/24) is written
`4.583333D-01`, so `MATRIX1` runs on a `GC_303 = 2i·gHza/vevhat` 1.2e-8 off the
model's own Python value; MadGraph's Python `model_reader` agrees with this crate
to 1e-14 on every coupling of every row[^n35-e1]. Only `ee_to_zh_smeft` and
`wpwm_to_wpwmz_cw` carry it. `coupling_oracle.rs` takes Python as the arbiter and
lists the Fortran deviation by name; an unlisted Fortran/Python disagreement
fails ([coupling-oracle](coupling-oracle.md))[^n36-b5]. `ee_to_zh_smeft`'s
amplitude cell stays `info` rather than matching a rounded reference.

**`get_channel_cut` reads an uninitialised `t`** under `sde_strat = 1` with
`tmin_for_channel ≠ -1`: its only assignment is inside `if (sde_strat.eq.2)`
[^mg-genps]. Every banked run has `tmin_for_channel = -1`; this crate refuses any
other value (`IgnoredPhysics`), since a row using it would have no defined
reference behaviour[^n36-b3].

**The 3.5.x channel weight at narrow resonances.** 3.5.7's `sde_strategy = 2`
weight computes `tmp = (t-Mass)*(t+Mass)` with `t` already `p²`, so it never
peaks on a pole: it handed 99.8% of a Higgs pole to continuum channels, and the
unwindowed σ came out 2.3% low with a quoted 0.2% error that three fresh seeds
confirmed. 3.7.1 has `tmp = (t-Mass**2)` with a plain Breit–Wigner
weight[^n27-b1]. The fix, upstream `286feb8e`[^mg-286feb8], shipped in 3.6.2 and
was never backported to 3.5.x, which is why references come from 3.7.1
([madgraph-oracle-pinning](madgraph-oracle-pinning.md); MadEvent's channel weight
is [phase-space/madevent-single-diagram-enhancement](../phase-space/madevent-single-diagram-enhancement.md)).

**3.5.7's `αs(M_Z)` at fixed-energy beams.** 3.5.7 wrote the `nn23lo1` set's
`αs(M_Z) = 0.130` into the parameter card of `lpp = 0` runs, which carry no PDF;
3.7.1 keeps the model's 0.118. Partonic QCD references moved by `0.920ⁿ` while
our σ tracked exactly, since every gate reads `αs` from the run's own card
([refdata-sigma-comparability](refdata-sigma-comparability.md)).

**The MLM path** (read off the source while porting `ickkw = 1`)[^n41-15]:

| where | defect | effect here |
|---|---|---|
| `reweight.f:1138` | `.not.fixed_fac_scale1.or.fixed_fac_scale2` precedence | changes a weight with exactly one fixed μF under matching; refused, the message naming the line |
| `setcuts.f:939-942` | duplicate `iforest(2)` test | grids only |
| `cuts.f:565` | `ktdurham` `.and.`/`.or.` precedence | CKKW-L, out of scope |
| `addmothers.f:115` | compares `igraphs(1)` to a stale loop index | unreachable: `vec_igraph` is never 0 on a written MLM event |
| `banner.py:1706` | `setWeightName` raises when `ickkw ≠ 0` (`"…".str(…)`) | Python systematics only |
| `rewgt` | reads the final-state `ipdgcl` left by the previous event | empty: no `IPROC` of any MLM row mixes jet and non-jet final-state flavours (`mlm_census.json`) |

**The reweight module.** `launch` blocks after the first in one card each
rewrite `events_out.lhe` from the unmodified input, so a multi-launch card keeps
only the last hypothesis and writes a mislabelled weight; and a compiled
`rwgt_dir` cannot be reused (`setup_f2py_interface` reads an undefined `opts`).
`gen_reweight_oracle.py` runs each hypothesis in a fresh work area, and the cost
comparison estimates H hypotheses as setup + H × loop[^rw-results].

**`rambo.py`.** `if(wt > 174 and iwarn[4] > 5)` should read `< 5`, as the
massless case does, so the massive overflow warning never fires[^mg-rambo]. Nothing
here uses it.

## Also bounding how MadGraph numbers are read

- **Measured inconsistencies inside MadEvent**, not located in code: quoted errors
  that are not spreads, windowed runs that understate their spread, an unweighted
  sample contradicting its own run's windowed σ, and multi-group unweighting that
  does not reproduce. They are listed in
  [madgraph-reference-runs](madgraph-reference-runs.md); the `ee_to_mumua` part is
  not the 3.7.1 `get_channel_cut` change, since that run is `sde_strategy = 1` with
  `tmin_for_channel = -1`, where `get_channel_cut` returns 1 in both
  versions[^n29-d].
- **`SCALUP` is not a defect.** MadGraph writes `sqrt(max(q2fact(1),
  q2fact(2)))` (`unwgt.f:752`), the factorisation scale by definition, and under
  matching with `pdfwgt` that is `q2bck`; μR reaches the record only through
  `AQCDUP`. Reading `SCALUP` as μR is a misreading hazard, pinned in this crate by
  `scalup_is_the_factorisation_scale_not_the_renormalisation_one`
  (`lhef/build.rs`) ([scales-pdf/record-scales](../scales-pdf/record-scales.md))[^n22-close].

[^mg-unwgt]: `unwgt.f:752-761` at `b7687064`.
[^n07-aqcdup]: Note 07, "`SubProcesses/unwgt.f` — Direct Bug Found".
[^mg-aloha]: `create_aloha.py:262-263` (flip) and `:527-530` (numerator) at `b7687064`.
[^n38-s3]: Note 38 §4 S3, the σ rows of `$`.
[^n07-p1d]: Note 07 appendix, the `FFV2P1D_1` finding, scope measurement and upstream draft.
[^n38-z1]: Note 38 §8.1: the patched reference reads 0.04888 ± 0.00020 pb (five seeds, χ²/dof 5.9) against 0.04982 ± 0.00004.
[^mg-export]: `export_v4.py:7076` at `b7687064`.
[^n36-b8]: Note 36 §7.1, B8; note 07's model-loading table.
[^n35-e1]: Note 35 §3 E1, the `ee_to_zh_smeft` attribution.
[^n36-b5]: Note 36 B5.
[^mg-genps]: `genps.f:1817` (`get_channel_cut`) at `b7687064`.
[^n36-b3]: Note 36 B3.
[^n27-b1]: Note 27 B1 outcome and its upstream provenance.
[^mg-286feb8]: Upstream commit `286feb8e`, 2025-01-27.
[^n41-15]: Note 41 §1.5 with the M0 census and M1 records that measured each effect.
[^rw-results]: `validation/madgraph/gen_reweight_oracle.py` docstring and the reweight-cost results note.
[^mg-rambo]: `rambo.py:218` at `b7687064`.
[^n29-d]: Note 29 chain D; its Run 0 records `get_channel_cut = 1` for this run in both versions.
[^n22-close]: Note 22 close-out listed `SCALUP` as a defect; note 35 §10.1 item 9 and `lhef/build.rs` record what it is.
