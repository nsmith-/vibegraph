---
type: Validation Gate
title: Cross-section gates against MadGraph
description: "validate_sigma and validate_hadronic compare sigma through the production integrand under MadGraph's own run card: pull plus rel_tol, multi-seed statistics and the blind spots."
status: draft
tags: [validation, cross-section, madgraph, vegas, hadronic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: n18-regime, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L352-L378", title: "Note 18 §3 (hadronic validation regime)"}
  - {id: n18-h7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5 decision records (H7 hadronic-sigma, H8 cli-integrate)"}
  - {id: n18-outcome, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L935-L1039", title: "Note 18 Outcome (load-bearing findings)"}
  - {id: n19-v3b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/19-validation-pass-plan.md#L116-L138", title: "Note 19 V3b (σ gate through the run card)"}
  - {id: n24-p3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1649-L1772", title: "Note 24 P3 (the σ gate at lpp = 1)"}
  - {id: n25-integrals, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L140-L150", title: "Note 25 §3.3 (integrals)"}
  - {id: n25-generic, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L355-L376", title: "Note 25 §5.4 (integrals genericization)"}
  - {id: n25-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L561-L579", title: "Note 25 §9 (decisions)"}
  - {id: vsigma, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_sigma.rs#L1-L270", title: "validate_sigma.rs module docs, PULL_LIMIT, PULL_REPORTED_NOT_ASSERTED, Plan"}
  - {id: vhad, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_hadronic.rs#L1-L430", title: "validate_hadronic.rs module docs, budgets, combine_seeds"}
  - {id: runcard, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/runcard.rs#L10-L20", title: "runcard.rs accepted beam configurations"}
---

Two banked-layer test files compare our integrated cross section with
MadGraph's banked one, row by row, and write the `integrals` cells of the
[validation report](validation-report.md).

| file | rows | integrand |
|---|---|---|
| `vibegraph-lib/tests/validate_sigma.rs` | fixed-beam rows (`lpp1 = lpp2 = 0`): SM, SMEFTsim and toy-model rows, the seeded grammar rows, and the long-tier 2→6 rows | `FixedBeamIntegrand`, the engine behind `vibegraph integrate` |
| `vibegraph-lib/tests/validate_hadronic.rs` | proton rows (`lpp1 = lpp2 = 1`): Drell-Yan on two cards, the llj family, `pp_to_bb*`, `pp_to_jj`, the re-carded rows | `ProtonIntegrand`: flavour groups over `(τ, y)` with a per-diagram multichannel inner map, one VEGAS grid per `(group, diagram)` channel |

The run card accepts exactly these two beam configurations and rejects any
other.[^runcard] MLM-matched rows have their own gate:
[MLM σ gate](mlm-sigma-gate.md).

## Design rules

**MadGraph's own run card on both sides.** Each gate reads
`output/<row>/Cards/run_card.dat` (and, for fixed beams, the run's own
`param_card.dat`), so beams, cuts and scales cannot drift between the two sides
by construction.[^n19-v3b][^n18-regime] An active run-card cut we do not
implement is a hard error (`CutError::UnimplementedCutActive`), so gaps surface
loudly rather than integrating a different region.[^n19-v3b]

**Only the generic path.** σ goes through the production integrand and nothing
special-cased; there is no per-process integrand. A row the generic path cannot
match is recorded as informational with its measurement and filed, not fixed by
a bespoke path.[^n25-integrals][^n25-generic][^n25-decisions] The integrate
artifact carries a per-channel subsampler summary (map kinds, poles), which the
row file reprints, so what the sampler did is part of the record.[^n25-generic]

**Production statistics.** Integration is channel by channel
(`FixedBeamIntegrand::adapt_grids`): each channel on its own grid with a share
`αⱼ` of the budget, summed in quadrature, exactly what `vibegraph integrate`
does.[^vsigma] Each point's clustering configuration, which sets the per-event
scale, is drawn `∝ AMP2_c` per flavour group and beam ordering rather than
taken from the channel the sampler used; see
[the configuration draw](../scales-pdf/clustering-configuration-draw.md). Every
gated integration asserts `scale_draw_fallbacks() == 0`, so a non-finite `AMP2`
that would silently fall back to the sampling channel fails
loudly.[^vhad]

**The coupling is part of what is compared.** Fixed-beam rows run α_s at the
run card's per-event renormalisation scale
(`FixedBeamIntegrand::use_running_coupling`), as MadGraph does, not at the
param card's α_s; this is what lets QCD rows be asserted at all.[^vsigma]

## The statistic

A row's `Plan` (`plan_for` in `validate_sigma.rs`) says how it is
exercised:[^vsigma]

| plan | meaning |
|---|---|
| `Gate { neval, niter, rel_tol }` | assert `|pull| ≤ PULL_LIMIT` and `|σ_vg/σ_MG − 1| ≤ rel_tol` |
| `Info { reason }` | integrate and report against the bank, assert nothing; the rung a demotion lands on, with the disagreement recorded, never a widened `rel_tol` |
| `Long { rel_tol: Option, reason }` | measured by an oracle-layer task (`pixi run validate-sigma-2to6`); `Some` asserts like `Gate`, `None` reports |
| `Skip(reason)` | not integrated, with a printed reason |

`pull = (σ_vg − σ_MG)/√(err_vg² + err_MG²)` and `PULL_LIMIT = 3.5`. The limit is
a false-positive rate against a trial count: over five seeds per gated row the
worst pull per row ran 0.51 to 2.65, which is what a correct bound on the
maximum of five near-normal draws looks like. Reading "only" twice inside it is
the instrument working.[^vsigma]

`rel_tol` is set per row from the larger of the reference's own Monte Carlo
error with headroom and the measured five-seed spread, never fitted around the
achieved central value; see [gate thresholds](gate-thresholds.md). It is
always enforced, including on rows whose pull is only reported.

The hadronic gates measure each row over several seeds and combine them with
`combine_seeds`: the **unweighted** mean, `err = √(Σσᵢ²)/n`, and the seeds'
χ²/dof about the mean. Weighting by `1/σ²` would double-count a seed whose
variance came out low by chance, the same bias Lepage's theorem forbids for
VEGAS's own iterations. Each asserts a relative bound (`*_MAX_REL`, 0.005 on
every row) and a scatter bound (`*_MAX_CHI2_PER_DOF = 4.0`). On Drell-Yan the
relative bound is asserted together with a 3σ pull, `pull < 3 && rel < 0.005`:
with MadGraph's error near 0.05% the pull binds, at about 0.24%, and the
relative bound catches what the pull cannot, a seed whose quoted error a weight
tail inflated, which shrinks the pull without moving σ any closer.[^vhad] The scatter, not
the error, is what shows a missed region.[^vhad] Seed and budget discipline is
[seed sweeps and budget ladders](seed-sweeps-and-budget-ladders.md).

### When a pull is reported, not asserted

`PULL_REPORTED_NOT_ASSERTED` names rows whose pull is reported and whose
`rel_tol` stays enforced. A pull bounds a disagreement only while the residual
is Monte Carlo and shrinks with budget; for a systematic of measured size it
grows without bound as `err_vg` falls, so asserting it would fail a row that had
not moved. A row leaves the list only when its residual shrinks with budget and
its five-seed scatter sits near χ²/dof 1, never because its budget was cut. The
gate asserts that every listed row is still gated, so an exemption cannot hide
a row that asserts nothing.[^vsigma] Which rows are on it, and why, is
[sigma-row gating exceptions](sigma-row-gating-exceptions.md).

## References

Fixed-beam references are `validation/madgraph/sigma_reference.json`, extracted
from each run's `results.dat`; the grammar rows read seeded MadEvent references
(`grammar_sigma_reference.json`) under the
[MadEvent seed policy](madevent-reference-seed-policy.md). The llj, jj and
re-carded hadronic references are read straight from the banked run's own
`SubProcesses/results.dat` rather than copied into JSON: the Drell-Yan
generator regenerates `hadronic_sigma_reference.json` wholesale and would
silently drop an entry it does not know, and reading the run keeps the number
tied to the run it came from.[^n24-p3] The PDF set is `NNPDF23_lo_as_0130_qed`,
`lhaid 247000`, on both sides ([pinned PDF set](../scales-pdf/pinned-pdf-set.md)),
so no interpolation systematic enters.[^n18-outcome] The CLI resolves PDF sets
through `vibegraph-cli/src/assets.rs`
([asset resolution](../tooling/asset-resolution.md)). Which refdata cut a
reference came from matters: see
[refdata σ comparability](refdata-sigma-comparability.md).

Heavy suites need an optimised profile: the llj sweep is minutes under
`--profile release-debug` and hours unoptimised; `pixi run -e madgraph
validate-hadronic` runs the hadronic file.[^vhad][^n24-p3]

## What it covers and what it cannot see

The per-point [amplitude oracle](amplitude-oracle.md) is blind to everything
outside the matrix element; this gate is the coarse instrument that sees it:
beam configuration and flux, initial spin and colour averages, identical-particle
and phase-space symmetry factors, the cut filter, the phase-space measure, the
PDF luminosity and α_s.[^vsigma][^n19-v3b] On a proton row with a coloured
initial state it also checks that flavour groups sum to MadGraph's own
subprocesses, both beam orderings are counted once, and the `(τ, y)` map's
`τ_min` hint clips nothing.[^n24-p3]

It cannot see:[^n24-p3][^vsigma]

- **Anything σ integrates over**: a per-diagram phase, a colour-flow
  relabelling, a helicity-by-helicity error. Those are pinned at the amplitude
  level.
- **A map whose weight and density are wrong by one common factor**: the map is
  only used here, never compared.
- **Any distribution**: one number agreeing is one number agreeing. Shapes are
  the [samples gate](samples-gate.md); pointwise integrand factors are
  [integrand and sampler oracles](integrand-and-sampler-oracles.md).
- **A mis-sampled region of small measure**, reported *confidently*. Iterations
  that miss a region return a small integral and a small variance, so a single
  seed's pull reads a σ 25× low quoted to 5% as a mild few-σ miss. Only the
  seed sweep and the budget ladder defend against it.
- **A residual below the row's resolution**, roughly its reference's own error.

The `samples`-side counterpart of this list is the samples gate's blind-spot
table; the hadronic integrand itself is described in
[the proton integrand](../hadronic/proton-integrand.md).

## Worked lessons

- **Known-wrong informational comparison from day one.** The first hadronic σ
  was wired as a printed delta before its conventions were reconciled, so the
  moment it snapped to agreement was the end-to-end signal; it was enforced at
  close. (`AGENTS.md` carries the rule.)[^n18-regime]
- **A thin cut band wants a change of variables, not more iterations.** On the
  direct `xᵢ = x_min^(1−uᵢ)` map an `m_ll` window is a thin diagonal band in
  the unit square, and VEGAS left the `[60,120]` card 6% (5.8σ) low. Sampling
  `(τ, y)` turns the window into a one-dimensional bound on `τ`; both cards then
  came out near 0.1% with about 10× smaller error.[^n18-outcome] See
  [hadronic (τ, y) sampling](../phase-space/hadronic-tau-y-sampling.md).
- **Cuts are lab-frame, `|M|²` is partonic-CM.** The final state is boosted by
  the parton-system rapidity before `Cuts::pass`. For a back-to-back LO lepton
  pair `Δφ = π`, so `drll` never fires; check that a default cut exercises real
  logic before assuming it does.[^n18-outcome]
- **Read the reference toolchain's control flow, not the cited line.**
  MadGraph's `dr` threshold is squared once in a first-call block (`cuts.f`),
  and its LHAPDF link failed because the conda env's exported `LDFLAGS` defeated
  `make_opts`'s `ifeq($(origin LDFLAGS),undefined)` guard (fixed by appending
  `-lc++` in `gen_hadronic_sigma.sh`).[^n18-h7][^n18-outcome]

[^n18-regime]: Note 18 §3.
[^n18-h7]: Note 18 §5, records H7 and H8.
[^n18-outcome]: Note 18 Outcome.
[^n19-v3b]: Note 19 V3b.
[^n24-p3]: Note 24 P3: the gate, what it proves and cannot see, plan corrections.
[^n25-integrals]: Note 25 §3.3.
[^n25-generic]: Note 25 §5.4.
[^n25-decisions]: Note 25 §9, decision 4.
[^vsigma]: `vibegraph-lib/tests/validate_sigma.rs`, module docs, `PULL_LIMIT`, `PULL_REPORTED_NOT_ASSERTED`, `Plan`.
[^vhad]: `vibegraph-lib/tests/validate_hadronic.rs`, module docs, `LLJ_NEVAL`…`DY_MAX_CHI2_PER_DOF`, `combine_seeds`.
[^runcard]: `vibegraph-lib/src/runcard.rs` module docs.
