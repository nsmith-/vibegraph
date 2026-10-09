---
type: Design Decision
title: Per-event α_s by rescaling the folded constant pools
description: "ScaleAwareAmplitude moves pools to a per-event α_s by G-power tags, verified against full re-evaluation; couplings that are not monomials in G fall back to re-evaluating the model."
status: draft
tags: [alpha-s, couplings, evaluator, scales, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n22-2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L157-L201", title: "Note 22 §2 (where the running coupling multiplies in)"}
  - {id: n22-out, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L283-L338", title: "Note 22 session outcomes (D1–D4)"}
  - {id: n35-v3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1524-L1692", title: "Note 35 §10.9 addendum (V3), the scale-change fallback row"}
  - {id: rescale-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/rescale.rs#L1-L88", title: "helas/eval/rescale.rs module documentation and RescaleFallback"}
  - {id: vs-fallback, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_sigma.rs#L200-L218", title: "validate_sigma.rs SCALE_FALLBACK_ROWS"}
---

# Per-event `α_s` by rescaling the folded constant pools

## Decision

A dynamical renormalisation scale changes exactly one thing about a bound amplitude: its
two numeric constant pools (complex couplings, and real masses, widths and folded
coefficients). The compiled skeleton, the helicity expansion and the colour-factor matrix
do not depend on `α_s`. So `ScaleAwareAmplitude` (`vibegraph-lib/src/helas/eval/rescale.rs`)
owns a private copy of the pools and rewrites them per event, leaving the shared
`AmplitudeEvaluator` untouched.[^rescale-rs]

It does **not** scale diagrams by `(αs(μ)/αs(μ₀))^{n_d/2}` before summing them. That seam
does not exist: `lower_flows` builds one JAMP per colour flow as `Σ_d coeff_d · amp_d`, and
the result is hash-consed, constant-folded, helicity-expanded and CSE'd into one arena, so
per-diagram identity is gone before anything is summed. Restoring it would mean splitting
the root into per-(flow, order) JAMPs, which perturbs the compiled core, the JAMP dumps and
the zero-amplitude pruning the MadGraph amplitude gate rides on.[^n22-2] How the program is
compiled is [amplitudes/evaluator-architecture](../amplitudes/evaluator-architecture.md),
and how constants are folded is
[performance/constant-collection-and-fused-sums](../performance/constant-collection-and-fused-sums.md).

## Two paths, one oracle

- **Reference path.** Re-evaluate the model (`EvaluatedModel::set_alpha_s`, then
  `Folded::pools`). Exact for any model and any parameter graph, and slow, because the
  parameter graph is keyed by name.
- **Scaling path.** Every tree-level coupling of a renormalisable model is a monomial
  `k·Gⁿ` with `k` independent of `G` (`GC_10 = −G`, `GC_12 = i·G²`, …). The exponent is
  read symbolically off the UFO expressions and propagated through the folded constant
  subgraph (`Mul` adds exponents; `Add` requires them equal). A scale change is then
  `consts[i] ← base[i]·rⁿⁱ` with `r = G(αs)/G(αs_ref)`: a few multiplies, no allocation,
  no re-evaluation.

A sum of unequal exponents, anything reached through a function of `G`, or any other
`aS`-driven parameter is **detected, not assumed**: one untaggable entry sends the whole
amplitude down the reference path (`RescaleFallback`: a complex or real entry that is not
a monomial, `G` not scaling as `αs^½`, no non-zero reference `aS` on the card, or an
exponent span wider than 16). The fallback is never a guess.[^rescale-rs]

The scaling path is validated **against** the reference path, entry by entry, at 100 random
`αs` on the 14 amplitude-gate processes it was built against (`validate-scale-couplings`). The two agree to a few
ulp (worst 5 ulp measured); bit equality is unreachable because they are different
floating-point routes to the same value. At the card's own `αs`, `r = 1` exactly and the
pools return bit for bit to the bound ones, which is what leaves the MadGraph amplitude
gate untouched.[^n22-out]

**Cost.** Only a few pool entries per process carry a power of `G`, so a scale change is
nanoseconds against a matrix element of a microsecond or more; an amplitude with no strong
coupling has nothing to move. The pools are per-thread by ownership (`fork`), never shared
mutably, so a parallel integrator cannot have one thread read another's coupling.
Per-diagram coupling orders are not needed for this: a diagram's power of `G` is the
product of its vertices' constants.[^n22-out]

## Rows that take the fallback

Two banked rows re-evaluate the whole model on every scale change, listed in
`SCALE_FALLBACK_ROWS` (`validate_sigma.rs`):[^vs-fallback][^n35-v3]

| row | why | consequence |
|---|---|---|
| `gg_to_ttx_smlimit_qcd2` | SMEFTsim's effective `g g h` vertex, kept by an explicit `QCD<=2`, has a coupling that is not a monomial in `G` | the cost lands almost entirely on the α survey: 15.6 s at 4 000 × 2 against 17.3 s at the gate budget of 40 000 × 6 |
| `gg_to_gg_cg` | `O_G`'s higher-derivative four-gluon vertex carries `cG` alongside the strong coupling | same mechanism, a different operator |

The list is asserted **in both directions**: a row that stopped falling back leaves a stale
entry and fails, and a row that started falling back is loud rather than a silent
hundredfold on the banked layer. The model behind these rows is
[model/smeftsim-topu3l](../model/smeftsim-topu3l.md).

## Where it is used

The fixed-beam and proton integrands both move their amplitudes per point through it, after
the scale prescription gives `μR` ([scales-pdf/setclscales](setclscales.md)) and the `αs`
source gives `αs(μR)` ([scales-pdf/alpha-s-sources](alpha-s-sources.md)). An integrand whose
matrix elements carry no strong coupling does not rebind per point at all.

[^n22-2]: Note 22 §2, the design decision: rescale the constant pool, not the diagrams.
[^n22-out]: Note 22 session outcomes, D3: `ScaleAwareAmplitude` gated scaling against reference.
[^n35-v3]: Note 35 §10.9, the scale-change fallback row (`gg_to_ttx_smlimit_qcd2`) and its cost.
[^rescale-rs]: `vibegraph-lib/src/helas/eval/rescale.rs`, module documentation and `RescaleFallback`.
[^vs-fallback]: `vibegraph-lib/tests/validate_sigma.rs`, `SCALE_FALLBACK_ROWS`.
