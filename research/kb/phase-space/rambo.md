---
type: Algorithm
title: Massive RAMBO
description: "The Kleiss–Stirling–Ellis massive RAMBO map and weight, its F-generic signature, and the uniforms-replay oracle that pins it."
status: draft
tags: [rambo, phase-space, flat-sampling, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n18-rambo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L246-L257", title: "Note 18 §2.3, massive RAMBO over F: Real"}
  - {id: n18-regime, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L352-L378", title: "Note 18 §3, validation regime (RAMBO rows)"}
  - {id: n18-h3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5, decision record H3 (RAMBO and its oracle)"}
  - {id: n32-2to6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L478-L597", title: "Note 32 §5.1, flat RAMBO on the 2→6 rows"}
  - {id: rambo-rs, resource: "vibegraph-lib/src/phasespace/rambo.rs", title: "rambo, rambo_massless, rambo_massive, RamboPoint"}
  - {id: oracle-rs, resource: "vibegraph-lib/tests/rambo_oracle.rs", title: "Uniforms-replay oracle against validation/rambo/rambo_fixture.json"}
  - {id: flatmc-rs, resource: "vibegraph-lib/tests/rambo_flat_mc.rs", title: "flat_mc_two_body_normalization, flat_mc_partonic_sigma"}
  - {id: kse, resource: "https://doi.org/10.1016/0010-4655(86)90119-0", title: "R. Kleiss, W. J. Stirling, S. D. Ellis, A new Monte Carlo treatment of multiparticle phase space at high energies, CPC 40 (1986) 359"}
  - {id: mg-rambo, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/rambo.py#L218", title: "MadGraph various/rambo.py, massive overflow warning"}
---

## The map

`phasespace::rambo` (`rambo.rs:50`) is the flat phase-space map of Kleiss,
Stirling and Ellis[^kse], decoupled from the generator:

```rust
pub fn rambo<F: Real>(sqrt_s: F, masses: &[F], u: &[F]) -> RamboPoint<F>
// RamboPoint { momenta, weight, xi }
```

It consumes exactly `4n` uniforms and returns `n` on-shell momenta summing to
`(√ŝ, 0, 0, 0)`:[^rambo-rs]

1. `n` isotropic massless vectors `qᵢ`, four uniforms each: `cos θ`, `φ`, and
   `q⁰ = −ln(r₃r₄)` (so `q⁰ ∼ Γ(2)`);
2. one boost and scale onto total momentum `(√ŝ, 0, 0, 0)`, giving massless
   momenta of constant weight;
3. if any mass is non-zero, a Newton solve for the rescale `ξ` in
   `Σᵢ √(mᵢ² + ξ²|p⃗ᵢ|²) = √ŝ`, with an `F`-relative tolerance `1e-13·√ŝ`,
   mapping `pᵢ → (√(mᵢ² + ξ²|p⃗ᵢ|²), ξ p⃗ᵢ)`.

The weight is the massless volume times the massive rescale Jacobian:

```
R_n(massless) = (π/2)^{n−1} ŝ^{n−2} / ((n−1)! (n−2)!)
J_massive     = (Σ|p⃗ᵢ|/√ŝ)^{2n−3} · (Σ|p⃗ᵢ|²/Eᵢ)⁻¹ · √ŝ · Πᵢ(|p⃗ᵢ|/Eᵢ)
```

so a flat average of `weight · f` estimates `∫ dR_n f`. The `(2π)^{4−3n}`
factors of the full `dΦ_n` measure live in the cross-section prefactor, not in
the weight[^n18-h3]. All-zero masses take a fast path: no Newton solve,
`xi = 1`, weight the massless volume.

`rambo_massless(sqrt_s, n, &mut rng)` and `rambo_massive` are thin wrappers
that draw the `4n` uniforms from a generator in the original call order and
call the same arithmetic (`massless_momenta`, and `rambo`), so the massless
output is bit-for-bit what it was before the map was split from its generator;
existing benches and sanity tests kept their goldens[^n18-h3]. Both return
momenta only and discard the weight, so `rambo_massive`'s points are on-shell,
momentum-conserving test kinematics, not an unbiased flat sample. Behind the phase-space seam the same map is
`RamboChannel` ([channel contract](channel-contract.md)).

## Where it is used, and where it must not be

Flat RAMBO is the volume reference every channel map is checked against
(`V_n` reproduced within MC error) and the raw sampler of a fixed-beam
integrand before a multichannel map is installed. It is not a production
sampler for peaked integrands: on a six-leg final state the physical poles sit
on a set of vanishing flat measure, and flat RAMBO misses the 2→6 rows' cross
sections by eleven and fifteen orders of magnitude despite a 46% cut-survival
rate[^n32-2to6]. Hadronic runs draw `(τ, y)` first and run the partonic map at
the drawn energy ([τ, y sampling](hadronic-tau-y-sampling.md)).

## The oracle that pins it

The map is checked by **replaying uniforms**, not by sharing a generator
stream, which is also why no MadGraph-compatible RNG is needed
([RNG substreams](rng-substreams-and-parallel-determinism.md)):[^n18-regime]

| oracle | checks | tolerance, observed | blind to |
|---|---|---|---|
| `rambo_oracle.rs::replay_matches_python`: `(u[4n], momenta, ξ, weight)` dumped by the pure-stdlib `validation/rambo/dump_rambo_fixture.py`, 8 cases | the deterministic map | momenta and `ξ` rel ≤ 1e-13, weight ≤ 1e-12; observed 1.3e-15 and 3.4e-16 | which uniform feeds which draw |
| bits→uniform conversion goldens and one seeded end-to-end momenta golden | stream addressing and draw order | exact | — |
| conservation and on-shell fuzz over random `(n, masses, √ŝ)`, threshold-adjacent included | numerics | — | any exactly-conserving wrong map |
| `flat_mc_two_body_normalization`: σ(e⁺e⁻ → μ⁺μ⁻) at √s = 10 GeV against the QED-only `4πα²/(3s)` | the `R_n` volume and the `(2π)` measure factor | asserts rel < 0.03 (MC noise plus the Z interference the formula omits); observed 929.4 ± 0.5 pb against 928.9 pb, rel 6e-4 at N = 2e5 | per-point errors that integrate away |

`flat_mc_partonic_sigma` (`#[ignore]`) integrates
`u u~ > c c~ e+ e- mu+ mu- QCD=0` at √ŝ = 500 GeV, uncut, against MadGraph's
banked 6.556e-7 pb. Four seeds at N = 2e4 read 1.13–5.30e-6 pb, a factor of
several apart and all above the bank: flat sampling on a collinear-peaked
integrand has a heavy tail and its naive `σ/√N` understates the error, and the
banked run likely carries default lepton cuts this estimate omits. Its
assertion is a same-order band, an order-of-magnitude end-to-end check only;
the two-body analytic comparison is the normalisation gate[^n18-h3].

## MadGraph's copy

MadGraph's `madgraph/various/rambo.py` is a direct Python transcription. Its
massive overflow warning tests `iwarn[4] > 5` where `< 5` is meant
(line 218), so the warning never fires:[^mg-rambo]

```python
if(wt > 174  and iwarn[4] > 5):
    print(" RAMBO WARNS: WEIGHT = EXP(%s) MAY OVERFLOW" % wt)
```

[^kse]: Kleiss, Stirling, Ellis 1986.
[^rambo-rs]: `vibegraph-lib/src/phasespace/rambo.rs`, module doc.
[^n18-h3]: Note 18 §5, H3.
[^n18-regime]: Note 18 §3, the RAMBO rows of the oracle table.
[^n32-2to6]: Note 32 §5.1, the 2→6 rows.
[^mg-rambo]: MadGraph `various/rambo.py` at the pinned commit.
