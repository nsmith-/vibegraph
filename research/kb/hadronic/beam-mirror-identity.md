---
type: Physics Convention
title: The beam-exchange mirror identity and its visibility bound
description: "|M_ba(p1,p2,q)|² = |M_ab(p1,p2,Rq)|² with R reflecting only outgoing legs; per-leg record fields swap under exchange; the 0.076 ŝ/(ŝ+m_Z²) bound that tests the reflected term."
status: draft
tags: [hadronic, mirror, beam-ordering, convention, proton]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L943-L1010", title: "Note 24 P2 (the mirror term is mandatory)"}
  - {id: n24-mirror, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1252-L1295", title: "Note 24 P2c (the mirror term, and what its test can and cannot catch)"}
  - {id: n24-record, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1885-L1906", title: "Note 24 (the exchanged beam ordering, and MadGraph's own answer)"}
  - {id: n27-b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L745-L750", title: "Note 27 B5 (mirror-term bound as a function of ŝ)"}
  - {id: n27-b5-out, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L902-L911", title: "Note 27 B5 outcome (the measured bound)"}
  - {id: n25-register, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L678-L719", title: "Note 25 findings register (item 4, the mirror term's visibility)"}
  - {id: proton-mirror, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/proton.rs#L380-L402", title: "FlavorGroup::mirror_into"}
  - {id: proton-floor, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/proton.rs#L3430-L3665", title: "Mirror test, mirror_visibility_floor and probe_mirror_visibility_ladder"}
---

# The beam-exchange mirror identity

## The identity

Diagram enumeration emits **one ordering per unordered initial state**: `g u` is
generated, `u g` is not ([process/subprocess-enumeration](../process/subprocess-enumeration.md)).
Both orderings are physical, because the two beams' densities are read at different
momentum fractions, so the missing one is restored through

```text
|M_{b a}(p₁, p₂, q)|² = |M_{a b}(p₁, p₂, R q)|²,   R: (E, pₓ, p_y, p_z) ↦ (E, pₓ, −p_y, −p_z)
```

`R` is the rotation by π about the x axis. It maps one partonic-CM beam onto the other
(`R p₁ = p₂`), so the rotated configuration with the beam slots swapped back is the
mirrored subprocess. **The beams are left where they are and only the outgoing legs are
reflected** (`FlavorGroup::mirror_into`).[^proton-mirror] Reflecting the whole point
would put beam 0 on `−z`, which breaks the partonic-CM contract the pruned evaluator
asserts; reflecting the outgoing legs alone is the same rotation composed with the beam
swap.[^n24-mirror]

So a [flavour group](flavour-groups.md) contributes

```text
xf_a(x₁)·xf_b(x₂)·|M(q)|²  +  xf_b(x₁)·xf_a(x₂)·|M(R q)|²
```

under **one** cut indicator on the unreflected `q`: `R` is an argument to the matrix
element, not a second event, and the final state is the same. A member whose beams carry
the same parton has one ordering and contributes no mirror term, since counting it twice
would double that subprocess.[^n24-p2] The full integrand is
[hadronic/proton-integrand](proton-integrand.md).

The mirror term is not a symmetry assumption. Drell–Yan's map is symmetric under the
exchange, so summing both luminosity orderings against one `|M|²` happened to work there;
a process with a jet is not symmetric, and dropping the mirror silently halves the `g q`
contribution.

## What the test catches and cannot catch

`the_mirrored_beam_ordering_needs_the_reflected_matrix_element` (`proton.rs`) enumerates
each group's mirrored subprocess from its own process string (`u~ u > e+ e- g`,
`u g > e+ e- u`, …), compiles it, and compares, on `p p > l+ l- j`:[^n24-mirror]

| what | measured | asserted |
|---|---|---|
| reflected representative against the explicitly mirrored subprocess | 5.4e-13 worst; 4.4e-15 on every point but one | `< 1e-11` |
| an `xz` reflection alone (`p_y ↦ −p_y`) | 7.7e-16 | `< 1e-12` |
| dropping the mirror (the representative at the unreflected point) | see the bound below | tenth percentile above the floor |

- **It catches** a dropped mirror term, and a reflection that does not reverse `p_z` (the
  load-bearing part of `R`): such a map reproduces the *direct* value and fails the first
  row.
- **It cannot catch** the sign of `p_y` in `R`. `|M|²` is invariant under the extra `xz`
  reflection, so `R` is pinned only up to it. That is asserted as a measurement: if the
  second row ever moved, the test's stated blind spot would be wrong and it fails. Both
  maps are correct implementations of the mirror.
- **Why the bound is 1e-11 and not 1e-14.** The 5.4e-13 is one point of 36, a RAMBO draw
  `8e-10` off the light cone and `2e-12` off momentum conservation, where two
  independently compiled programs route gauge-dependent parts differently. It is a
  property of the test; the integrand evaluates one program at both arguments. An
  earlier probe run at a deliberately off-shell point showed a 73% "disagreement" that
  was entirely this effect. **Any mirror-type check must use an on-shell, momentum-
  conserving point, or it measures gauge dependence.**

## The visibility bound is a function of ŝ

The control that makes the identity check meaningful is that dropping the mirror moves
`|M|²` visibly. A flat threshold is the wrong shape. The mirror term is the
beam-direction asymmetry of `p p > l+ l- j`, which grows like `ŝ` below the electroweak
scale and saturates above it, the shape of a `γ*/Z` core whose forward–backward asymmetry
is set by `ŝ/m_Z²`. The gate's floor (`mirror_visibility_floor`) is that shape, fitted to
the measured plateau and halved:[^n27-b5][^n27-b5-out][^proton-floor]

```rust
fn mirror_visibility_floor(sqrt_s: f64) -> f64 {
    const M_Z2: f64 = 91.188 * 91.188;
    let s = sqrt_s * sqrt_s;
    0.076 * s / (s + M_Z2)
}
```

The test asserts, at `√ŝ ∈ {25, 65, 150, 400, 1200}` GeV, that the **tenth percentile**
of the visibility over 32 draws exceeds the floor. The floor sits 1.58 to 4.86 times
under every point of `probe_mirror_visibility_ladder` (`#[ignore]`), which measures 25 GeV
to 4 TeV over three independent streams and two sample sizes (32 and 512), none of them
the gate's own draw.

**It is a percentile, not a minimum,** because the two orderings agree exactly wherever a
configuration is symmetric. The minimum visibility over the same draws falls by a decade
going from 32 to 512 points at every energy: it measures the sample size, not the physics.
A flat `1e-3` threshold on the weakest term holds above 220 GeV and fails at
`√ŝ = 25` GeV, where the weakest mirror term is worth 8.4e-4.[^n25-register]

## Per-leg record fields under exchange

The identity extends from `|M|²` to everything a record is filled from. `R` maps each
beam onto the other, so whatever the representative says about its leg 0 (colour lines,
helicity, mass) describes the event's **second** beam. Under `BeamOrdering::Exchanged`,
`FlavorGroup::event_legs` and `event_leg_colors` swap the two incoming entries of every
per-leg field and leave the outgoing legs untouched. `R` is a proper rotation, so a
helicity carries across unchanged. A beam exchange permutes the legs of a colour flow,
not the flows, so the flow index the draw chose is unaffected.[^n24-record]

`an_exchanged_ordering_relabels_the_beams_of_every_per_leg_field` checks this against
each mirrored subprocess compiled from its own proc card: per-helicity `|M_c|²` and
per-flow `JAMP2` agree to 1e-11 of the summed `|M|²` over all six `ℓℓj` groups, matched by
helicity *tuple* under the leg permutation, not by index. **Its blind spot:** `ℓℓj` has
one colour flow per subprocess, so the flow index cannot be permuted there and only that
flow's tags are compared.

MadGraph's banked events agree independently. `P1_gq_llq`'s `leshouche.inc` gives
`g u > e⁺e⁻ u` as `ICOLUP = (501,502), (502,0), 0, 0, (501,0)`; a banked event with the
quark on beam 1 carries `(502,0), (501,502), 0, 0, (501,0)`: the two incoming rows
exchanged, the outgoing row untouched, the same integers.

## The mirrored term's scale

Under a configuration-dependent scale the mirrored term is clustered as its own physical
event: the configuration is drawn from `AMP2` at `Rq`, and the clustering reads the lab
momenta rotated by π about x with the beams exchanged (`mirror_lab_into`), so the
representative's first parton is on the first beam. This is MadEvent's `IMIRROR = 2`
view, which clusters the unflipped momenta in its matrix element's flavour order. The
resulting per-beam `μF` is swapped back to physical beam order. The scale consequences
are [scales-pdf/clustering-configuration-draw](../scales-pdf/clustering-configuration-draw.md);
how events are drawn and written is [events/generate](../events/generate.md).

[^n24-p2]: Note 24 P2, the mirror term is mandatory and needs a pinning test.
[^n24-mirror]: Note 24 P2c, the identity as implemented and what its test can and cannot catch.
[^n24-record]: Note 24, the exchanged beam ordering against MadGraph's own events.
[^n27-b5]: Note 27 B5, the item asking for the bound as a function of ŝ.
[^n27-b5-out]: Note 27 B5 outcome, the fitted bound and why it is a percentile.
[^n25-register]: Note 25 findings register, item 4: the flat control failing below the electroweak scale.
[^proton-mirror]: `FlavorGroup::mirror_into`, `vibegraph-lib/src/proton.rs`.
[^proton-floor]: The mirror test, `mirror_visibility_floor` and the ladder, `vibegraph-lib/src/proton.rs` tests.
