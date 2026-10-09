---
type: Design
title: The toy UFO models as validation instruments
description: "vibegraph_toy_UFO and vibegraph_toy_color_UFO put one Lorentz or colour structure in one vertex, so conventions no SM or SMEFT process isolates are gated against MadGraph."
status: draft
tags: [validation, ufo, toy-model, colour, lorentz]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n35-t1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L905-L991", title: "Note 35 T1 (authoring the toy UFOs and banking their oracle)"}
  - {id: n35-d, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1191-L1218", title: "Note 35 §7 (decisions D1–D5)"}
  - {id: n35-gated, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1281-L1358", title: "Note 35 §10.1 (what the sprint leaves gated; pinned conventions)"}
  - {id: n35-census, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1416-L1434", title: "Note 35 §10.4 (the toy models' op census)"}
  - {id: fact-sign, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/facts/fermion-line-sign-ignores-vertex-content.md", title: "Phase B fact: the fermion-line reversal sign ignores vertex content"}
  - {id: toy-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/toy_models.rs#L1-L200", title: "vibegraph-lib/tests/toy_models.rs"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml", title: "validation/manifest.toml toy rows"}
  - {id: diagram-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/diagrams/diagram.rs#L570", title: "Diagram::fermion_line_sign"}
---

Two small UFO models live under `validation/ufo/` (MIT, written here):
`vibegraph_toy_UFO` and `vibegraph_toy_color_UFO`. Neither is physics anyone
would simulate. Each vertex holds one structure that no model in reach isolates,
with couplings chosen so the structure is visible in `|M|²`, and every row is
gated against MadGraph at every level. MadGraph imports any UFO directory, so
they use the same oracle machinery as the SM and
[SMEFTsim rows](non-sm-rows.md).[^n35-t1][^toy-rs]

## Why hand-written rather than adopted

The public candidates each bury one wanted structure in hundreds of vertices
(`RS` for spin-2, `sextet_diquarks` for `K6`, RPV models for baryonic `ε`), and
none was in the pinned MadGraph checkout (its `models/` holds `sm`, `loop_sm`,
`MSSM_SLHA2`, `hgg_plugin`, `taudecay_UFO`). A hand-written model isolates each
primitive in one vertex with couplings we choose (a user decision: generate, not
adopt).[^n35-t1][^n35-d] Size such a model from a measured probe of the loader,
not from reading it.[^n35-gated]

## Contents

| model | fields | structures |
|---|---|---|
| `vibegraph_toy_UFO` | Dirac lepton `lt`, Dirac quark `qt`, singlet scalar, singlet vector, octet scalar `o8` (5 fields, 7 vertices) | literal-`Sigma` dipole `FFVD = Sigma(3,-1,2,-2)*P(-1,3)*ProjM(-2,1)`; tensor contact `FFFFT = Sigma(-1,-2,2,1)*Sigma(-1,-2,4,3)` and its γγ expansion `FFFFG` as a separate coupling on the same vertex; `Identity`; `Gamma5`; `FFV1`; colour `d(1,2,3)` on `o8 o8 o8` |
| `vibegraph_toy_color_UFO` | all scalars: two distinct colour triplets `p3`, `r3` (distinct because `Epsilon(1,2,3)` is antisymmetric), a triplet and a sextet diquark | `Epsilon`/`EpsilonBar`; `K6`/`K6Bar` (sextet leg first, per `color_algebra.py`); every Lorentz structure the scalar `1` |

The colour model is all-scalar because two fermions reach a diquark only
through a fermion-number-violating vertex, i.e. charge conjugation, which is
out of scope.[^n35-t1] Restrict cards select one structure family at a time
(`restrict_{dipole,tensor,yukawa,dcolor,all}`, `restrict_{eps,k6,all}`).

The six rows, all gated in `diagrams`, `amplitudes`, `integrals` and
`samples`:[^manifest][^toy-rs]

| row | process | isolates |
|---|---|---|
| `ll_to_qqx_toy_dipole` | `lt~ lt > qt qt~ NP<=1` | literal `Sigma` in a dipole, interfering with a plain gauge coupling (pins `Sigma`'s sign) |
| `ll_to_qqx_toy_tensor` | `lt~ lt > qt qt~ NP<=1 NPGG<=1` | `Sigma⊗Sigma` and its γγ spelling in one process |
| `ll_to_qqx_toy_yukawa` | `lt~ lt > qt qt~ NP<=2 NPCP<=2` | bare `Identity` and `Gamma5` bilinears |
| `qqx_to_o8o8_toy_dcolor` | `qt qt~ > o8 o8 NP<=2` | the symmetric structure constant `d` in a colour basis |
| `p3r3_to_p3r3_toy_epsilon` | `p3 r3 > p3 r3 NP<=2` (`restrict_eps`) | baryonic `Epsilon`/`EpsilonBar` |
| `p3r3_to_p3r3_toy_sextet` | `p3 r3 > p3 r3 NP<=2` (`restrict_k6`) | sextet Clebsch–Gordan `K6`/`K6Bar` |

Order bounds in the process strings are load-bearing: each structure family has
its own coupling order, which makes MadGraph split a two-structure vertex into
two interactions (two diagrams, an `AMP()` each), and MadGraph's WEIGHTED
default would otherwise drop the higher-order half of the interference these
rows are built around.[^toy-rs]

## What they pinned

Conventions the toy rows fixed, each by a test that fails if it is false:[^n35-gated][^n35-t1]

- **ALOHA's `Sigma` is half the textbook `(i/2)[γ^μ, γ^ν]`** (`L_Sigma.sigma`
  carries ±½), measured as `AMP(FFFFG)/AMP(FFFFT) = 4 × ggam/gtens` to 4.7e-14;
  a kernel at textbook normalisation is 4× too large on the tensor row and 2× on
  the dipole row. The square is blind to `Sigma`'s global sign; the dipole row,
  linear and interfering with the gauge coupling, pins it.
- **A chiral projector beside a literal `Sigma` keeps its chirality**: `σ^{μν}`
  commutes with `γ⁵`; it is `γ^μ P_χ = P_χ̄ γ^μ` that conjugates.
- **`Sigma⊗Sigma` equals its γγ expansion** per diagram and per helicity in one
  process.
- **The all-incoming crossing exchanges `Epsilon ↔ EpsilonBar` and
  `K6 ↔ K6Bar`**; without the swap the colour matrix stops reducing to a
  scalar. See
  [colour crossing, ε and sextets](../amplitudes/colour-crossing-epsilon-and-sextets.md).
- **MadGraph's `AMP2` grouping is `IdentifyConfigTag`**: the Yukawa row's four
  scalar exchanges on one s-channel propagator get one `AMP2` accumulator from
  MadGraph, which first exposed a grouping gap here. See
  [per-diagram AMP2](../amplitudes/per-diagram-amp2.md).
- **A restrict card's non-zero values are the model's defaults.** A card-less
  run of a restricted model had silently been its SM limit; pinned by
  `restricted_defaults_are_madgraphs_generated_param_card` (421 external
  parameters over 13 rows against MadGraph's generated cards). See
  [restriction semantics](../model/restriction-semantics.md).
- **`SCALUP` is `μF`, not `μR`, where the clustering reads them off different
  vertices**: the two diquark rows are the first banked 2→2 with
  `q2fact(1) ≠ q2fact(2)` (two distinct massive triplet beams); see
  [scale replay](scale-replay-gate.md).

**The fermion-line sign: a lesson in two parts.** The `d`-colour row exposed a
sign no SM process isolates: every SM fermion line reaches a gauge vertex, so
`qt qt~ > o8 o8`, a line built only of Yukawa-type bilinears, was the first place
the per-propagator fermion-line sign could be wrong without every other row
noticing.[^n35-gated] But the rule first adopted from it, that the reversal sign
depends on the line's Dirac-matrix content, was wrong: on that row it cancelled
a second bug (the operator-free scalar vertices `SSS1`/`SSSS1` missing their
scalar-sink −1). Real Yukawa-line processes overturned it: after both fixes,
`ta+ ta- > t t~ h h` went from 24 wrong per-diagram signs of 96 to 0, and the
toy row still agrees per flow. The current rule, in
`Diagram::fermion_line_sign` (`vibegraph-lib/src/diagrams/diagram.rs`), gives
every uncrossed line one −1 per internal fermion propagator whatever its
vertices, and a crossed line a single −1; standalone JAMP rows
(`tata_to_ttxh`, `tata_to_ttxhh`, `bbx_to_hh`) gate it per
helicity.[^fact-sign][^diagram-rs] See
[fermion-line sign](../amplitudes/fermion-line-sign.md).

So a toy model is a strong instrument and a weak oracle on its own: it isolates
a structure no real model isolates, and for the same reason it can pin a rule
that only looks right because two errors cancel there. Pair it with a real
process carrying the same structure before believing a convention it fixes. This
is the general point of
[oracle blind spots](oracle-blind-spots-and-non-vacuity.md).

## Blind spots of the colour oracles

Both colour oracles normalise a column by its leading fourth root of unity, so
they are blind to a sign between a diagram's amplitude and its flow assembly by
construction: the `d` row once matched MadGraph's `AMP()` to 4e-16 and its CF
matrix exactly while `JAMP(1)` had the opposite sign. Only `|M|²` with a `d` in
an interference saw it.[^n35-t1] See [colour oracles](colour-oracles.md).

## The op census

`vibegraph-lib/tests/toy_models.rs` reads the gated toy rows back from the
manifest, checks each row's process string against the one its amplitude table
was generated for, asserts the diagram counts against `diagrams.json`, and runs
a two-way op census against an allowlist that must shrink as coverage
grows.[^n35-census][^toy-rs] The toy family is the only model family that reaches
`SigmaVout`, `SigmaOut` and the bare `Gamma5Amp` at process level, and it also
reaches `FierzOut`/`FierzOutRev`/`FierzPair` and `IdentityAmp`. Ops it cannot
reach (no `ProjP`, no `Epsilon` or `Metric` in a Lorentz structure, every
`Gamma` rooted at its vector leg in these 2→2 rows) are listed with the reason,
and some rest on hermetic kernel identities instead. See
[evaluator kernel coverage](evaluator-kernel-coverage.md).

## Authoring notes for MadGraph

Recorded while writing them (details in
[UFO authoring for MadGraph](../model/ufo-authoring-for-madgraph.md)):[^n35-t1]

- A vertex whose particle multiset is not self-conjugate needs its h.c. listed
  too: MadGraph looks vertices up by sorted PDG tuple with initial legs
  conjugated.
- A model with no `T(a,i,j)` vertex leaves `find_color_anti_color_rep` guessing
  the 3/3̄ labelling from the particle's colour sign: invisible without an `ε`
  (a uniform transpose), fatal with one.
- A model must declare `QCD`, or the unbounded WEIGHTED search fails.
- In the toy models MadGraph's `aS` is injected, not declared; see
  [sigma-row gating exceptions](sigma-row-gating-exceptions.md) for the
  `AQCDUP` consequence. The Lorentz structures themselves are typed per
  [the UFO/ALOHA type matrix](../model/ufo-aloha-type-matrix.md).

[^n35-t1]: Note 35 T1, brief and "Landed".
[^n35-d]: Note 35 §7, decision D3.
[^n35-gated]: Note 35 §10.1, the pinned conventions and the bugs the toy rows found.
[^n35-census]: Note 35 §10.4.
[^fact-sign]: The Phase B fact on the fermion-line sign (the reversal of note 35's rule 5).
[^toy-rs]: `vibegraph-lib/tests/toy_models.rs`.
[^manifest]: `validation/manifest.toml`, the six toy rows.
[^diagram-rs]: `vibegraph-lib/src/diagrams/diagram.rs`, `fermion_line_sign`.
