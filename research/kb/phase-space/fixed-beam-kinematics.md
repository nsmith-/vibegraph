---
type: Derivation
title: "Fixed beams with masses: ŝ, CM energies, flux and frames"
description: "Beams on their own mass shells at run-card energies; ŝ, E* and |p*| via Källén; Møller flux 2λ^½; records in the CM frame, rapidity cuts boosted to the lab."
status: draft
tags: [phase-space, kinematics, beams, flux, frames]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n36-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L192-L352", title: "Note 36 B1 (massive fixed beams)"}
  - {id: guide-fixed, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/docs/src/guide/07-phase-space.md#L382", title: "Guide chapter 7, 'Fixed beams'"}
  - {id: mg-genps, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/genps.f#L676", title: "MadGraph genps.f (stot :676, cm_rap :388, flux :427)"}
  - {id: mg-rap, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/kin_functions.f#L95-L132", title: "MadGraph kin_functions.f rap()"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml#L1014", title: "validation/manifest.toml, qqx_to_o8o8_toy_dcolor integrals cell"}
---

# Fixed beams with masses

A run with `lpp = 0` collides the incoming particles themselves: no τ sampling,
no luminosity. The initial state is fixed by four numbers, the run card's
`ebeam1`/`ebeam2` and the two incoming legs' pole masses, and everything the
initial state contributes is derived from them once, in
`vibegraph-lib/src/phasespace/beams.rs`, and read by both the integrand and the
per-diagram channel maps. A map whose beams disagreed with the integrand's would
sample a different process than the one evaluated, and `|M|²` cannot see that.[^n36-b1]

The user-facing derivation, in KaTeX, is the guide's "Fixed beams" section
(`docs/src/guide/07-phase-space.md:382`).[^guide-fixed] This concept carries the
formulas, the code sites and the MadGraph correspondence; keep the two in step.

## The kinematics

In the laboratory both beams are on shell along `±z`
(`beams::lab_beam_momenta`):

```
p_a = (E_a, 0, 0, +√(E_a² − m_a²)),   p_b = (E_b, 0, 0, −√(E_b² − m_b²))
```

**Invariant** (`beams::partonic_s`):

```
ŝ = (p_a + p_b)² = m_a² + m_b² + 2 (E_a E_b + |p_a| |p_b|)
```

This is MadGraph's `stot = m1² + m2² + 2*(pi1(0)*pi2(0) − pi1(3)*pi2(3))`
(`genps.f:676`, with `pi2(3)` negative).[^mg-genps] It is `(E_a + E_b)²` only for
two massless beams **of equal energy**; massless beams at unequal energies
collide at `4 E_a E_b`. The banked `p3 r3` run (60 and 70 GeV beams at
250 + 250 GeV) has `√ŝ = 499.99275`, not 500.

**Centre-of-mass beams** (`beams::beam_momenta`), sharing one momentum
magnitude and splitting the energy by mass:

```
E_a* = (ŝ + m_a² − m_b²) / (2√ŝ),   E_b* = (ŝ − m_a² + m_b²) / (2√ŝ)
|p*| = λ^{1/2}(ŝ, m_a², m_b²) / (2√ŝ)
λ(x, y, z) = x² + y² + z² − 2xy − 2yz − 2zx
```

`beams::kallen` evaluates λ as `(a − b − c)² − 4bc`, not the expanded form:
the expansion loses most digits when λ is many orders below its terms (a soft
emission), and the sampler's walk and the density evaluate it at inputs one ulp
apart, so an ill-conditioned λ makes the two describe different maps. The
grouped form still cancels near the two-body threshold, where λ and the weight
it feeds both go to zero. `beams::beam_momenta_m2` takes invariants instead of
masses so an interior rung of a t-channel chain can carry a spacelike line
(`m² < 0`).

**Flux.** The `F` in `σ = (1/F) ∫ dΦ_n |M|²` is the Møller invariant

```
F = 4 √((p_a·p_b)² − m_a² m_b²) = 2 λ^{1/2}(ŝ, m_a², m_b²) = 4 |p*| √ŝ
```

which reduces to `2ŝ` for massless beams and is MadGraph's
`flux = 1/(2*SQRT(LAMBDA(s, m(1)**2, m(2)**2)))` (`genps.f:427`).
`FixedBeams::inverse_flux` (`hadronic.rs`) returns `1/F`. Relative to the
massless `2ŝ`, the error at `E ≫ m` is `−(m_a² + m_b²)/ŝ` to leading order, a
few per cent for beams of tens of GeV at hundreds. The weight chain the flux
enters is [pipeline/overview](../pipeline/overview.md).

## Frames

The laboratory and the partonic centre of mass differ by a boost along `z` of
rapidity

```
y_cm = ½ ln((p⁰ + p³)/(p⁰ − p³)),   p⁰ = E_a + E_b,   p³ = |p_a| − |p_b|
```

zero for equal masses at equal energies, `≈ 5.386e-3` on `p3 r3`
(`beams::lab_beta_z` returns the velocity; its test checks `artanh β` against
this closed form).

| What | Frame | MadGraph | vibegraph |
|---|---|---|---|
| Momentum generation, `|M|²`, channel maps | partonic CM | `genps.f` | `FixedBeams::momenta` |
| Event record | partonic CM (banked `p3 r3` events have `Σ pz = 0` at unequal beam energies) | — | record writes the CM beams with their masses |
| Rapidity cuts | laboratory | `rap()` adds `cm_rap` (`kin_functions.f:132`, `cm_rap` set at `genps.f:388`)[^mg-rap] | `passes_cuts` boosts by `lab_beta` first, skipped when it is exactly 0 |
| `<init>` `EBMUP` | — | `ebeam(i)` | run card's `ebeam1/2` |

MadGraph's `rap()` is in `Source/kin_functions.f`, not `cuts.f`.

## Code

- `FixedBeams` (`hadronic.rs:715`): `from_run_card(rc, legs)` takes the run
  card's energies and the first two legs' masses; `sqrt_s`, `momenta`,
  `inverse_flux`, `lab_beta`. `FixedBeams::massless(√s)` is the light-cone
  special case.
- `InitialState::Beams` wraps it beside the decay initial state (one particle
  at rest, flux `1/(2M)`; see [process/decay-processes](../process/decay-processes.md)).
- Channel maps read `beam_masses` off the diagram's legs and build the beams
  through the same `beams::beam_momenta`, so integrand and maps cannot
  disagree ([phase-space/diagram-channels](diagram-channels.md)).

## How it is pinned

- `beams.rs` unit tests reproduce MadGraph's banked `p3 r3` record:
  `√ŝ = 499.99275`, `E* = 248.69635 / 251.29639`, `|p*| = 241.35011`; and the
  unequal-energy massless case.
- The three massive-incoming toy rows (`qqx_to_o8o8_toy_dcolor`,
  `p3r3_to_p3r3_toy_epsilon`, `p3r3_to_p3r3_toy_sextet`) gate their `integrals`
  cells at `rel_tol 0.005`, set from a measured five-seed spread with a
  converging budget ladder (`validation/manifest.toml`).[^manifest] They are
  the only rows that measure the massive flux and frame at all.
- The `samples` gate's incoming-leg column compares the integrand's beams to
  the banked record at the record's printed precision (5.0e-9 GeV), on every
  fixed-beam row. It sees momenta, not the flux or the evaluation frame;
  the `integrals` cells see those.
- For massless beams every formula reduces exactly to the light-cone case
  (`λ^{1/2}(ŝ,0,0) = ŝ`, `E* = √ŝ/2`, `y_cm = 0` at equal energies), which is
  why massless rows were byte-identical across the change, and why only rows
  with massive incoming legs can detect an error here.

Not every lab-boost branch is reached by a banked cell: the cut boost is pinned
by `the_cut_filter_reads_the_laboratory_rapidity` (`hadronic.rs`), a hand-built point either side of an `|y| ≤ 1` cut, not by a reference
row with a rapidity cut on massive beams.

[^guide-fixed]: The guide's "Fixed beams" section, the user-facing form of this derivation.
[^mg-genps]: `genps.f` at `b7687064`: `stot` (:676), `cm_rap` (:388), `flux` (:427).
[^mg-rap]: `kin_functions.f` `rap()` at `b7687064`.
[^manifest]: The gated massive-beam `integrals` cells and their tolerance rationale.
[^n36-b1]: Note 36 B1: the derivation and its verification.
