---
type: Caveat
title: MadGraph defects found by this project
description: "Register of MadGraph defects met while validating: where each sits, whether it changes a weight on a card we support, how vibegraph handles it, and its upstream status."
status: draft
tags: [madgraph, defects, reference, upstream-report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n07-weak, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L43-L67", title: "Note 07, summary of weaknesses"}
  - {id: n07-num, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L115-L161", title: "Note 07, numerical and model-loading bug tables (rows found here)"}
  - {id: n07-ps, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L162-L227", title: "Note 07, phase-space and I/O bug tables (rows found here)"}
  - {id: n07-aqcdup, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L367-L415", title: "Note 07, AQCDUP truncated pi and rambo.py"}
  - {id: n07-p1d, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L439-L538", title: "Note 07, ALOHA P1D flipped-fermion veto, with upstream draft"}
  - {id: n22-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/22-dynamical-scales-plan.md#L339-L357", title: "Note 22 close-out headline"}
  - {id: n27-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L44-L211", title: "Note 27 B1, the h to tau tau pole bin and get_channel_cut"}
  - {id: n29-dm, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L3324-L3521", title: "Note 29 chain D measurements D.M1, D.M4, D.M7"}
  - {id: n29-mll, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L3686-L3770", title: "Note 29 chain D, m(mumu) secondary axis"}
  - {id: n35-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L334-L431", title: "Note 35 E1 (GC_303 literal rounding)"}
  - {id: n36-b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L413-L457", title: "Note 36 B5, the coupling oracle"}
  - {id: n36-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L567-L650", title: "Note 36 B3 (genps.f uninitialised t)"}
  - {id: n36-b8, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L709-L749", title: "Note 36 7.1 B8 (injected aS)"}
  - {id: n38-s3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L802-L881", title: "Note 38 S3, $ as the pointwise integrand"}
  - {id: n38-z1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L1332-L1429", title: "Note 38 8.1 Z1 (patched $ t t~ reference)"}
  - {id: n41-15, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L191-L205", title: "Note 41 1.5, MadGraph defects met on the way"}
  - {id: n41-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1414", title: "Note 41 M3, D2 and R1"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2916-L3489", title: "Note 41 Z close-out"}
  - {id: rw-results, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/reweight-vs-madgraph-results.md#L32-L49", title: "Reweighting against MadGraph, what is measured"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/unwgt.f#L752-L761", title: "MadGraph unwgt.f, SCALUP and the truncated pi"}
  - {id: mg-aloha, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/aloha/create_aloha.py#L527-L530", title: "MadGraph create_aloha.py, the 1D numerator"}
  - {id: mg-aloha-flip, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/aloha/create_aloha.py#L262-L263", title: "MadGraph create_aloha.py, the momentum flip"}
  - {id: mg-export, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/export_v4.py#L7076", title: "MadGraph export_v4.py, aS injection"}
  - {id: mg-genps, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/genps.f#L1817", title: "MadGraph genps.f, get_channel_cut"}
  - {id: mg-rambo, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/rambo.py#L218", title: "MadGraph rambo.py overflow check"}
  - {id: mg-reweight, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/reweight.f#L1138", title: "MadGraph reweight.f, fixed_fac_scale precedence"}
  - {id: mg-banner, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/banner.py#L1706", title: "MadGraph banner.py setWeightName"}
  - {id: mg-286feb8, resource: "https://github.com/mg5amcnlo/mg5amcnlo/commit/286feb8e606a4e55951f6ea10ea0e3d145213b13", title: "mg5amcnlo commit 286feb8e, change sde_strategy2 to avoid negative weights"}
---
# MadGraph defects found by this project

This register covers defects in MadGraph5_aMC@NLO that this project met
first-hand while validating against it. MadGraph's own release-note bug history,
and the code-quality survey that reads it, are in
[the code-quality review](../references/codebases/madgraph5-code-quality-review.md).
How a defect is treated is set by
[the defect policy](madgraph-defect-policy.md): a defect that changes a weight on
a card we support is either reproduced bug-for-bug with a comment naming it, or
refused. It is never silently "fixed".

Line numbers are at the pinned tree `b7687064` (3.7.1) unless a row says
otherwise.

**Upstream status.** No defect below has been filed upstream. Report drafts for
the AQCDUP truncation, the ALOHA `P1D` veto and the grouped first call sit in note
07's appendix; finalising and filing them is the user's step
([madgraph-defect-reports-unfiled](../backlog/validation/madgraph-defect-reports-unfiled.md)).
The six MLM-study defects have no draft yet
([mlm-madgraph-defects-undrafted](../backlog/validation/mlm-madgraph-defects-undrafted.md)).

## Register

| defect | where | changes a weight we compare? | handling |
|---|---|---|---|
| `AQCDUP`/`AQEDUP` built with π truncated to 8 digits | `unwgt.f:760-761` | the record field only, +1.7e-8 | truncation reproduced before comparing |
| `$` never vetoes a flipped-slot fermion propagator | `create_aloha.py:262-263, 527-530` | yes: σ of `$ t t~` 37% high | reference made with patched ALOHA |
| grouped first `setclscales` call on the unpermuted point | `super_auto_dsig_group_v4.inc:842` | yes, process-dependent | registered deviation; see [its own concept](madgraph-permuted-first-call.md) |
| `aS` injected for a model with none, inconsistent with `G` | `export_v4.py:7076` | `AQCDUP` on the six toy rows | field measured, not enforced |
| UFO literal printed at seven digits in Fortran | Fortran model writer | `GC_303` by 1.2e-8 | Python `model_reader` is the arbiter |
| `t` read uninitialised in `get_channel_cut` | `genps.f`, branch at `:1938` | unreachable here | `tmin_for_channel ≠ -1` refused |
| 3.5.x `get_channel_cut` dimensionally wrong | 3.5.x `genps.f` | yes, at narrow poles | references come from 3.7.1 |
| 3.5.7 applied a PDF set's `αs(M_Z)` at `lpp = 0` | 3.5.7 param card writing | partonic σ by `0.920ⁿ` | not reproduced; cuts not comparable |
| six MLM-path defects | note 41 §1.5 (table below) | at most one, refused | see table |
| reweight module: one hypothesis per card survives | `reweight_interface.py` | the reweight oracle | one hypothesis per work area |
| `rambo.py` overflow warning never fires | `rambo.py:218` | no | none needed |

### `AQCDUP` with a truncated π

`unwgt.f` fills the event line's couplings as

```fortran
      aaqcd = g*g/4d0/3.1415926d0
      aaqed = gal(1)*gal(1)/4d0/3.1415926d0
```

while `g = √(4π·αs)` was built from full double-precision π
(`Source/MODEL/couplings.f`). The field is therefore `αs·π/3.1415926 =
αs·(1 + 1.7e-8)`, and `AQEDUP` carries the same factor[^mg-unwgt][^n07-aqcdup].
The bias is baked in before printing, so it is systematic and one-directional
and would survive a wider field. At the seven printed digits it is about a sixth
of the last digit, enough to move the rounding of roughly one event in twenty.

It was found by replaying banked events against our running `αs`: modelling the
truncation took the fraction of events reproducing MadGraph's printed digits from
about 95% to 100% on constant-scale runs. It is invisible at any tolerance looser
than about 1e-8, which is why the `AQCDUP` oracle gates on reproducing the printed
digits rather than on a chosen tolerance. Handling: the replay reproduces the
truncation before comparing; never read the field as `αs` itself. Note 07 cites
the lines as `:694-695`, which is the 3.5.x numbering.

### `$` and the flipped fermion slot (`FFV2P1D_1`)

`$ A` makes the amplitude multiply each marked propagator by a step function that
vanishes inside the Breit–Wigner window (ALOHA's `P1D` routines). The numerator is
written once for every spin[^mg-aloha]:

```python
numerator = "theta_functionr( (P(-1,id)**2 -(Mass(id)-BWCUTOFF*Width(id))**2 ) *( P(-1,id)**2 - (Mass(id)+BWCUTOFF*Width(id))**2),1,0)"
```

For a fermion propagator built from a vertex's second spinor slot, `need_P_sign`
is set and `re.sub(r'\b(P|PSlash)\(', r'-\1(', expr)` prefixes every `P(` with
`-`[^mg-aloha-flip]. `-P(-1,id)**2` parses as `-(p²)`, the argument becomes
`(p² + (M − cΓ)²)(p² + (M + cΓ)²) ≥ 0`, and the line is never zeroed. Writing the
square as a product fixes it; with the fix only `FFV2P1D_1` changes, and every
other `P1D` routine (vector, scalar, even-slot fermion) is byte-identical. By
reading only, not tested: a UFO custom propagator written with `P(-1,id)**2` for a
fermion would flip the same way[^n07-p1d].

Effect: on `u u~ > w+ b w- b~ $ t t~` at fixed √s = 500 GeV MadEvent zeroes the
`t~` window and not the `t` one, and reads 0.06811 ± 0.00008 pb against
vibegraph's 0.04982 ± 0.00004 (37% high). Rows whose marked line is a vector
boson (`$ z`, `$ w+`) agree within seed errors, as the scope predicts[^n38-s3].

Handling: vibegraph zeroes every marked line. The reference for this row is
generated by ALOHA patched with
`validation/madgraph/patches/aloha-p1d-flipped-fermion.patch`, applied by
`gen_onshell_veto.sh` to a copy of the pinned tree (row `uu_tt`); the unpatched
reading is kept as `uu_tt_mg371`. With the patch, MadGraph's standalone `SMATRIX`
equals vibegraph's zeroed amplitude to 1e-12 at six points, but MadEvent still
reads about 2% below this side and drifts with budget[^n38-z1]. That residual is
not this defect and is open:
[uux-wbwb-onshell-veto-tt-2pct-off](../backlog/validation/uux-wbwb-onshell-veto-tt-2pct-off.md).

### Grouped first call on the unpermuted point (H1)

Grouped MadEvent sets μR, the density scales and `q2bck`, and applies the MLM
`xqcut` rejection, on the unpermuted `PP` while the matrix element reads the
permuted `P1`. Full account, sizes and reproducer:
[madgraph-permuted-first-call](madgraph-permuted-first-call.md).

### `aS` injected into a model that declares none

A UFO with no `aS` external parameter gets one from the Fortran exporter:
`export_v4.py` logs `CRITICAL: aS not define as external parameter adding it!`
and appends `aS = 0.138` and `G = 4.1643` as internal parameters[^mg-export].
The two disagree: `G = 2√(π·aS)` gives 1.317 for 0.138, and 4.1643 is
`aS = 1.380`. `setrun.f` runs `αs(M_Z)` from `G`, so every event of such a model
carries an `AQCDUP` from a strong coupling of 1.38 in a model with no strong
interaction (about 0.43 at 250–500 GeV). The `CRITICAL` line is in each affected
row's `build.log`[^n36-b8].

Handling: the six toy rows (`ll_to_qqx_toy_*`, `qqx_to_o8o8_toy_dcolor`,
`p3r3_to_p3r3_toy_*`) carry `AQCDUP` measured and reported, not enforced; the
cause is asserted (`alpha_s_source()` is absent exactly on
`UNDECLARED_ALPHA_S_RUNS`). `validate_alphas` replays MadGraph's own events from
`G`'s 1.3799843265950287 to reproduce its field, never ours. `SCALUP` gates on all
six.

### Seven-digit UFO literals in the Fortran model

The Fortran writer prints a long UFO literal at seven significant digits:
`gHza`'s `0.4583333333333333` (11/24) becomes `4.583333D-01`, so the generated
`MATRIX1` runs on a `GC_303 = 2i·gHza/vevhat` that differs from the model's own
Python evaluation by 1.2e-8[^n35-e1]. MadGraph's Python `model_reader` on the same
card agrees with this crate to 1e-14 on every coupling of every banked row. Only
`ee_to_zh_smeft` and `wpwm_to_wpwmz_cw` carry it, and nothing else deviates
Fortran-against-Python[^n36-b5].

Handling: `coupling_oracle.rs` compares against Python as the arbiter and reports
Fortran, with the known writer deviation listed by name; a Fortran/Python
disagreement not on that list fails. `ee_to_zh_smeft`'s amplitude cell stays
`info` rather than matching a rounded reference on purpose. See
[the coupling oracle](coupling-oracle.md).

### `get_channel_cut` reads an uninitialised `t`

Under `sde_strat = 1` with `tmin_for_channel ≠ -1`, the branch
`if (t.lt.tmin_for_channel)` reads `t`, whose only assignment sits inside
`if (sde_strat.eq.2)`[^mg-genps][^n36-b3]. It is unreachable in this suite: every
banked run has `tmin_for_channel = -1`. Handling: this crate refuses
`tmin_for_channel ≠ -1` (`IgnoredPhysics`); a future row exercising it would have
no defined reference behaviour.

### The 3.5.x channel weight at narrow resonances

In 3.5.7 `get_channel_cut`, the `sde_strategy = 2` multichannel weight, computes
`tmp = (t-Mass)*(t+Mass)` with `t` already `p²`, so the weight never peaks on a
pole. On `e+ e- > mu+ mu- ta+ ta-` it handed 99.8% of the Higgs pole to
continuum channels, and the unwindowed σ was 2.3% low while its quoted 0.2%
error, and three fresh seeds, said otherwise. 3.7.1 has
`tmp = (t-Mass**2)` with a plain Breit–Wigner weight[^n27-b1]. The fix is
upstream commit `286feb8e`[^mg-286feb8], first released in 3.6.2 and never
backported to the 3.5.x LTS line, so no 3.5.x run is a valid narrow-resonance
reference at `sde_strategy = 2`. This is why references come from the pinned
3.7.1 ([madgraph-oracle-pinning](madgraph-oracle-pinning.md)); the six 3.5.7
runs kept in the bundle are the evidence for it. MadEvent's channel weighting is
described in
[madevent-single-diagram-enhancement](../phase-space/madevent-single-diagram-enhancement.md).

### 3.5.7's `αs(M_Z)` at fixed-energy beams

3.5.7 wrote the `nn23lo1` set's `αs(M_Z) = 0.130` into the parameter card of
`lpp = 0` runs, whose beams carry no PDF; 3.7.1 leaves the model's 0.118. Partonic
QCD references moved by `0.920ⁿ` in the power of `αs` between the two banks
while our σ tracked exactly, because every gate resolves `αs` from the run's own
parameter card. A partonic σ from a 3.5.7 bank is not comparable to a 3.7.1 one
([refdata-sigma-comparability](refdata-sigma-comparability.md)).

### The MLM path (note 41 §1.5)

Read off the pinned source while porting `ickkw = 1`[^n41-15]:

| where | defect | effect here |
|---|---|---|
| `reweight.f:1138` | `.not.fixed_fac_scale1.or.fixed_fac_scale2` precedence | changes a weight with exactly one fixed μF under matching; that card is refused, the message naming the line |
| `setcuts.f:939-942` | duplicate `iforest(2)` test | grids only |
| `cuts.f:565` | `ktdurham` `.and.`/`.or.` precedence | CKKW-L, out of scope |
| `addmothers.f:115` | compares `igraphs(1)` to a stale loop index | unreachable on every MLM row: `vec_igraph` is never 0 on a written event |
| `banner.py:1706` | `setWeightName` raises when `ickkw ≠ 0` (`"…".str(…)`) | Python systematics only |
| `rewgt` | reads the final-state `ipdgcl` left by the previous event | empty on every MLM row: no `IPROC` mixes jet and non-jet final-state flavours (`mlm_census.json`) |

### The reweight module

In 3.7.1, `launch` blocks after the first in one card each rewrite
`events_out.lhe` from the unmodified input, so only the last hypothesis survives
and a multi-launch card through `ReweightInterface.import_command_file` writes a
mislabelled weight; and a compiled `rwgt_dir` cannot be reused
(`setup_f2py_interface` reads an undefined `opts`)[^rw-results]. Handling:
`gen_reweight_oracle.py` runs each hypothesis in a fresh work area, and the cost
comparison estimates H hypotheses as setup + H × loop.

### `rambo.py`'s overflow check

`if(wt > 174 and iwarn[4] > 5)` should read `< 5`, as the massless case at line
164 does, so the massive-particle overflow warning never fires[^mg-rambo]. No
consumer here; vibegraph has its own RAMBO.

## Measured inconsistencies inside MadEvent

These are not located in code, but each bounds how a MadEvent number may be read:

- **Quoted errors are not spreads.** MadEvent seeds have scattered at χ²/dof up
  to 93 (`e+ e- > e+ e-`) against their own quotes, and seeds run in one shared
  directory correlate (χ²/dof 0.3–0.8)[^n41-z]. The seed policy and independent
  directories answer this
  ([madevent-reference-seed-policy](madevent-reference-seed-policy.md)).
- **A `dummy_cuts`-windowed run understates its own seed spread**, by 2.1× and
  1.9× in two `pt(γ)` windows of `ee_to_mumua`[^n29-dm].
- **The unweighted sample contradicts the run's own windowed σ.** On
  `ee_to_mumua`, MadEvent's sample puts 8.73% of σ in `pt(γ) ∈ [10, 20)` against
  9.40% from its own windowed runs (about 20σ), in both 3.5.7 and 3.7.1, and two
  complete `dummy_cuts` partitions of one run's phase space (in `pt(γ)` and in
  `m(μμ)`) disagree with each other by 16.7σ[^n29-mll]. The open part is
  [ee-mumua-radiative-return-sigma-high](../backlog/validation/ee-mumua-radiative-return-sigma-high.md).
  It is not the 3.7.1 `get_channel_cut` change: that run is `sde_strategy = 1`
  with `tmin_for_channel = -1`, where `get_channel_cut` returns 1 in both
  versions.
- **Multi-group unweighting is scheduling-sensitive.** A re-run of `pp_to_jj`'s
  card yields a different, equally valid sample, and the mixed MLM rows are not
  bit-reproducible across hosts
  ([refdata-banking-procedure](refdata-banking-procedure.md)).

## Misreading hazards that are not defects

- **`SCALUP` is the factorisation scale.** MadGraph writes
  `sqrt(max(q2fact(1), q2fact(2)))` (`unwgt.f:752`), by definition, and under
  matching with `pdfwgt` that is `q2bck`. It parts from μR wherever the
  clustering reads the two off different vertices; μR reaches the record only
  through `AQCDUP`. Note 22 listed this as a defect; it is not one
  (`lhef/build.rs` `scalup()`, pinned by
  `scalup_is_the_factorisation_scale_not_the_renormalisation_one`).
- **The banked `.lhe` is not `rw_events.f`'s format.** MadGraph's Python
  post-processing rewrites it through `lhe_parser.py`, in one of two numeric
  dialects. See [madgraph-reference-runs](madgraph-reference-runs.md).

[^n07-aqcdup]: Note 07, `unwgt.f` and `rambo.py` direct findings.
[^mg-unwgt]: MadGraph `unwgt.f` at `b7687064`, lines 752–761.
[^mg-aloha]: `create_aloha.py:527-530` at `b7687064`.
[^mg-aloha-flip]: `create_aloha.py:262-263` at `b7687064`.
[^n07-p1d]: Note 07 appendix, the `FFV2P1D_1` finding and upstream draft.
[^n38-s3]: Note 38 §4 S3, the σ rows of `$`.
[^n38-z1]: Note 38 §8.1, the patched reference: 0.04888 ± 0.00020 pb over five seeds against this side's 0.04982 ± 0.00004.
[^mg-export]: `export_v4.py:7076` at `b7687064`.
[^n36-b8]: Note 36 §7.1, B8.
[^n35-e1]: Note 35 §3 E1, the `ee_to_zh_smeft` attribution.
[^n36-b5]: Note 36 B5, the coupling oracle's findings.
[^mg-genps]: `genps.f:1817` (`get_channel_cut`) at `b7687064`.
[^n36-b3]: Note 36 B3.
[^n27-b1]: Note 27 B1 outcome and its upstream provenance.
[^mg-286feb8]: Upstream commit `286feb8e`, 2025-01-27.
[^n41-15]: Note 41 §1.5 and the M0/M1 census records.
[^rw-results]: `validation/madgraph/gen_reweight_oracle.py` docstring and the reweight-cost results note.
[^mg-rambo]: `rambo.py:218` at `b7687064`.
[^n41-z]: Note 41 Z, the independent-directory reference; note 38 §8.4 for Bhabha.
[^n29-dm]: Note 29, chain D runs D.M4 and D.M7.
[^n29-mll]: Note 29, chain D `m(μμ)` axis.
