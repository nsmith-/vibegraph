---
type: Overview
title: The LO generator pipeline
description: "The chain from a UFO model and two cards to unweighted LHE events: each stage, the module it lives in, how σ and the event weight are defined, and the e+e-→μ+μ- starter test."
status: draft
tags: [pipeline, overview, cross-section, architecture]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n00-steps, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/00-overview.md#L12-L56", title: "Note 00: goal, pipeline steps and toy process"}
  - {id: n01-lo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L128-L135", title: "Note 01: MadGraph5_aMC@NLO LO pipeline"}
  - {id: hadronic-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/hadronic.rs#L1563-L1582", title: "FixedBeamIntegrand master formula (hadronic.rs)"}
  - {id: unweight-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/unweight.rs#L1-L40", title: "Unweighting module doc (unweight.rs)"}
  - {id: diagrams-mod, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/diagrams/mod.rs#L1-L26", title: "diagrams module doc: parse, check, enumerate"}
  - {id: ee-tests, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_vegas.rs#L81-L145", title: "validate_vegas: sigma_qed_limit and sigma_z_pole"}
---
vibegraph is a leading-order (tree-level) Monte Carlo event generator written
from scratch in Rust, following the pipeline MadGraph5_aMC@NLO uses at LO and
validated against a pinned MadGraph at every stage.[^n00-steps][^n01-lo] What
the release covers is set by [the release-scope decision](../decisions/release-scope-lo-mlm.md);
how unsupported inputs are refused is in [release scope](release-scope.md).

## Stages

A run is `vibegraph integrate <proc_card> --run-card …` followed by
`vibegraph generate`, and optionally `vibegraph check-events` on the file it
wrote. Those three subcommands are the whole CLI (`vibegraph-cli/src/main.rs`).

| Stage | What happens | Where |
|---|---|---|
| 1. Model | Parse the UFO's Python files (particles, parameters, couplings, vertices, Lorentz and colour structures); apply the restrict card; digest the restricted model. The SM ships interned as a compressed blob. | `vibegraph-lib/src/ufo/` |
| 2. Process | Parse the whole proc card, refuse unsupported features in one check, then enumerate the tree diagrams of every concrete subprocess. | `vibegraph-lib/src/diagrams/` |
| 3. Amplitude | Compile each subprocess into a helicity program (HELAS-style currents and vertices, colour flows), bind it to the model's numerical parameters, evaluate the colour- and helicity-summed \|M\|². | `vibegraph-lib/src/helas/` |
| 4. Phase space | Map the unit hypercube to momenta: flat RAMBO, or a multichannel of per-diagram channels shaped by propagator poles; one VEGAS grid per channel. | `phasespace/`, `vegas.rs`, `budget.rs` |
| 5. Cross section | Build the integrand (flux, averaging, cuts, scales and α_s, PDFs at proton beams), integrate, and write the trained grids to an artifact. | `hadronic.rs`, `proton.rs`, `pdf/`, `coupling/`, `cuts.rs`, `runcard/`, `artifact.rs` |
| 6. Events | Replay the frozen grids, accept/reject channel by channel, select helicity and colour flow per event, write LHEF. | `unweight.rs`, `select.rs`, `lhef/`, `vibegraph-cli/src/generate.rs` |

Diagram enumeration still runs on the `feyngraph` crate (pinned by git rev in
`vibegraph-lib/Cargo.toml`), but its graphs are converted at once into
vibegraph's own `Diagram` (`diagrams/diagram.rs`). That type carries the
diagram's complete Fermi sign and symmetry factor as graph properties, and it is
what every later stage reads.[^diagrams-mod] Enumeration itself is described in
[diagram enumeration](../process/diagram-enumeration.md). Mixed final-state
multiplicities (MLM) are a sum of per-multiplicity integrands
(`multiplicity.rs`); reweighting is in `reweight/`.

## Cross section and event weight

The fixed-beam master formula, as `FixedBeamIntegrand` implements it:[^hadronic-rs]

```text
σ̂ = 1/F · ⟨spin·colour avg⟩ · ∫ dΦ_n Σ_sub S_sub |M_sub|²
  = 1/F · avg · (2π)^{4−3n} · ⟨w_map · Σ_sub S_sub |M_sub|²⟩_uniform
```

- `F = 2 λ^{1/2}(ŝ, m_a², m_b²)` is the Møller flux, `2ŝ` for massless beams,
  and `2M` for a `1 → n` decay.
- `avg = 1/Π_a (n_spin · n_colour)` over the incoming legs, derived per process
  (`initial_spin_color_average`).
- `w_map` is the map's weight. RAMBO's `R_n` times `(2π)^{4−3n}` gives the full
  `dΦ_n`, and the multichannel shares that normalisation.
- `S_sub = 1/Π_s n_s!` is each subprocess's own identical-particle factor.
- The cut indicator multiplies everything. Results are converted to pb with
  `GEV2_TO_PB = 3.893793721e8`.

At proton beams ([the hadronic integrand](../hadronic/proton-integrand.md)) the
point also carries the parton luminosity and the `(τ, y)` Jacobian, and sums
both beam orderings.

**The event weight is the integrand weight.** It is this whole product,
`|M|²` × map Jacobian × flux × averaging × PDFs, at the sampled point. It is
not a bare `|M|²/max|M|²`. The integral is `σ = Σ_j σ_j` over channels, each
trained on its own grid. `generate` draws channel `j` with probability
`∝ w_max_j` and accepts a point with `min(1, w_j(x)/w_max_j)`. A point above
its channel's maximum is kept with weight above 1 and counted, which keeps the
estimator unbiased.[^unweight-rs] The algorithm and its overweight accounting
are in [unweighting](../events/unweighting.md).

## The starter process

`e+ e- > mu+ mu-` is the first integration test. In the SM it goes through
both the photon and the Z. The analytic `σ = 4πα²/(3s)` the tests use is the
**photon-only** (QED) result in the massless limit, so the tests use it only
far below the Z pole:[^ee-tests]

- `validate_vegas::sigma_qed_limit` and `rambo_flat_mc::flat_mc_two_body_normalization`
  run at √s = 10 GeV with α = 1/132.507. There the Z interference is about
  0.5%, so both allow 3%. The flat-RAMBO test pins the map's `R_n` and
  `(2π)^{4−3n}` normalisation.
- `validate_vegas::sigma_z_pole` checks the full γ+Z cross section at
  √s = M_Z against MadGraph's 2025 pb, with `ptl > 10 GeV` and `etal < 2.5`
  applied as a cos θ window. The tolerance is 1e-3.

Both test files are in the banked layer (`required-features = ["extended-validation"]`).

[^n00-steps]: Note 00 §Goal–§Toy Process (the original six-step outline; its unweighting line predates the integrand-weight definition above).
[^n01-lo]: Note 01, MadGraph5_aMC@NLO, "LO pipeline (our scope)".
[^hadronic-rs]: `vibegraph-lib/src/hadronic.rs`, `FixedBeamIntegrand` doc and `prefactor()`.
[^unweight-rs]: `vibegraph-lib/src/unweight.rs` module doc.
[^diagrams-mod]: `vibegraph-lib/src/diagrams/mod.rs` and `diagram.rs` (`Diagram::sign`, `symmetry_factor`).
[^ee-tests]: `vibegraph-lib/tests/validate_vegas.rs` and `tests/rambo_flat_mc.rs`.
