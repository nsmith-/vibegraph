---
type: Design
title: Why the designed reference rows exist
description: "The rows built to isolate one feature: ud_to_epemud_qcd0 as the multi-rung spine reference, pp_to_jj as the QCD capstone, and the llj family that splits order spelling, scale and PDF."
status: draft
tags: [validation, manifest, reference-rows, madgraph, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n24-p0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L392-L445", title: "Note 24 P0 (what was banked for llj)"}
  - {id: n28-d, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L254-L279", title: "Note 28 §6 (decisions D1–D4)"}
  - {id: n28-s26, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L1765-L1803", title: "Note 28 S2.6 (the spine reference process and its card)"}
  - {id: n28-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L1916-L1955", title: "Note 28 S4 B3 (the spine reference σ and what it found)"}
  - {id: n28-k44, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2507-L2538", title: "Note 28 K4.4 (decisions on llj partonic, 2→6, pp_to_jj)"}
  - {id: n28-s6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2685-L2745", title: "Note 28 S6 (crossing sign rule: measured, row promoted)"}
  - {id: n28-c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L3625-L3775", title: "Note 28 C.3–C.6 (pp_to_jj capstone)"}
  - {id: n28-c24, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L3918-L3984", title: "Note 28 C2.4 (pp_to_jj σ cell gated)"}
  - {id: n28-z1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L4100-L4135", title: "Note 28 Z.1 (the duplicate llj run pruned)"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml", title: "validation/manifest.toml rationale fields and cell notes"}
  - {id: diagram-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/diagrams/diagram.rs#L570", title: "Diagram::fermion_line_sign"}
---

Most rows in the [process manifest](process-manifest.md) are there because a
process is common. A handful were designed: their process and run card were
chosen so that one feature of the chain is the only thing that can make them
disagree with MadGraph. Each row's one-paragraph reason is its `rationale`
field in `validation/manifest.toml`, and its current measurement is in its cell
notes; read both there. This concept records the cross-row design (which rows
pair with which, and what each pair isolates) and what those rows have
caught.[^manifest]

| row | isolates | why this card |
|---|---|---|
| `ud_to_epemud_qcd0` | multi-rung t-channel phase space | one flavour assignment with a full ladder; no α_s, no PDF |
| `pp_to_jj` | the whole QCD chain at MadGraph's defaults | default run card; unequal symmetry factors in one flavour group |
| `pp_to_llj_fixed` | the hadronic chain with a coloured initial state, at fixed scale | everything replayable exactly |
| `pp_to_llj_dyn` | the scale prescription alone | `pp_to_llj_fixed`'s card with the three `fixed_*_scale` switches off |
| `pp_to_llj` | coupling-order spelling, and the low-`m_ll` region | default orders, `mmll = 0` |
| `uux_to_epemg`, `ddx_to_epemg`, `gu_to_epemu`, `gux_to_epemux` | the llj amplitudes per class | single-subprocess `lpp = 0` runs at `√ŝ = 500` |

## `ud_to_epemud_qcd0`: the multi-rung spine reference

`u d > e+ e- u d QCD=0` at fixed partonic beams (250 + 250 GeV) is the
reference for the [t-channel spine](../phase-space/t-channel-spine.md). Its 35
diagrams split 12 / 14 / 9 over one, two and three spacelike lines in a single
subprocess, and its rungs are asymmetric (the blobs are a jet, a lepton and a
lepton pair; poles mix massless lines with `m_Z`), so an ordering test on it is
not vacuous.[^n28-s26]

The proposed process was `p p > e+ e- j j` at `QCD=0`. Enumerated, that is 112
non-empty subprocesses and 3024 diagrams, which `derive_flavor_groups` turns
into 60 groups pooling 1608 sampling channels: about 67× the per-point cost of
the 24-channel llj row, and MadGraph would have had to integrate and bank all
112 subprocesses. The narrowed card keeps the whole ladder spectrum at a
channel count comparable to llj. The flavour union and the `(τ, y)` convolution
are already gated by the Drell-Yan and llj rows, so fixing the initial state
loses no coverage.[^n28-d][^n28-s26] The middle option, if the proton path is
ever wanted on a ladder, is `p p > e+ e- u u~ QCD=0`: 4 subprocesses, 131
diagrams and channels, rung counts `{0: 64, 1: 44, 2: 14, 3: 9}`, but half its
diagrams have no spacelike line.[^n28-s26]

Card choices, each load-bearing:[^n28-s26]

- `QCD=0` and `lpp = 0` with all three scales fixed at `m_Z`: with no α_s in
  the matrix element and no PDF on either beam, no scale choice can reach σ, so
  the row is independent of the scale work by construction.
- `ptj = 20` regulates the t-channel singularity that would otherwise dominate
  σ̂, and is the scale the spine's fiducial bound uses.
- `mmll = 50`, for the same reason as `pp_to_llj_fixed` below.

**What it caught.** It is the first banked process with a `W` between two quark
lines and two mixed (initial↔final) fermion lines. σ first came out 7.7× high,
and `|M|²` disagreed point by point by factors of 2 to 63 while every countable
property (35 diagrams, `NCOLOR = 2`, `CF`, 8 helicities, spin/colour average
1/36) agreed: a missed cancellation between diagrams, not a normalisation. The
cause was a missing crossing sign on mixed fermion lines.[^n28-b3] The rule now
lives in `Diagram::fermion_line_sign` (`vibegraph-lib/src/diagrams/diagram.rs`)[^diagram-rs];
see [fermion-line sign](../amplitudes/fermion-line-sign.md). With it, every
diagram matches MadGraph's bare `AMP()` at `6.6e-15` under one global phase
(`G = −i`), and `amplitude_oracle`'s `KNOWN_LINEAR_DISAGREEMENT` is
empty.[^n28-s6][^manifest] Two oracle details surfaced on the way: MadGraph
groups this process's 35 diagrams into 21 `AMP2` accumulators
(`N_MAX_CG = 21`), and its grouping is `[0,2,4,6],[1,3,5,7],…`, so a
per-configuration comparison that flattened the grouping to get `AMP()` indices
was scrambled. Rows in `KNOWN_CONFIG_MERGE` now pair each configuration with
the diagram behind it.[^n28-b3][^n28-s6] The full method is
[bit-exact amplitude debugging](bit-exact-amplitude-debugging.md).

Its σ gates at `rel_tol 0.01`, set from the five-seed spread, not from the
reference error. The spread was recorded as worst `|rel| 2.6e-3`; the
September 2026 headroom census reran the same seeds after the note-34 draw
commits and measured `3.954e-3`, still well inside 0.01 (see
[the seed-headroom census](seed-headroom-census-2026-09.md)).
Either way: at four times the budget our
error is a third of MadGraph's, so the pull is floored by the reference and the
budget ladder cannot shrink the residual. What the ladder does show is that no
defect migrates between seeds.[^n28-s6][^manifest]

## `pp_to_jj`: the capstone

`p p > j j` on MadGraph's shipped run card, departing only in `pdlabel` and
`lhaid` (MadGraph's internal `nn23lo1` is not an LHAPDF6 grid we can
read).[^n28-d][^manifest] It needs the whole chain at once: a per-event kT
cluster scale, a sum over MadGraph's own 65 concrete assignments, and a flavour
group mixing subprocesses with unequal identical-particle factors (`g g > g g`
and `q q~ > g g` carry 1/2, `q g > q g` and `g g > q q~` carry 1).[^manifest]

**What it caught.** The enumeration listed outgoing permutations MadGraph does
not, so σ came out +36% high, and the `samples` cell failed every column except
`SPINUP` (the helicity fractions are least sensitive to reweighting one
subprocess against another). The scalar σ alone would have read the
double-counted subprocesses as normalisation, which is why
`jj_subprocesses_are_madgraphs_own`
(`vibegraph-lib/tests/validate_hadronic.rs`) now asserts the 65 assignments
equal to the run's `leshouche.inc` entry for entry, each with its outgoing
legs in MadGraph's order. 52 of the 65 have two different outgoing flavours and
neither side lists the swap. The near-degenerate relabelled channels also cost
variance: the enumerated arm scattered at χ²/dof 3.41 where the repaired one
sits near 1, and the repair removed exactly six channels (25 → 19
grids).[^n28-c][^n28-c24][^manifest] Its `ICOLUP` χ² later caught flavour-group
members borrowing the representative's colour-flow table; see
[the samples gate](samples-gate.md).

**Its tolerance is the reference's error, not a partition band.** The expected
large channel-partition ambiguity turned out to be the smallest of any
clustered row (gap `+1.03e-3` against its own `9.6e-4` Monte Carlo), for a
structural reason: a 2→2 final state gives the clustering no merge to choose, so
`μR` and both `μF` are functions of the momenta alone. So `JJ_MAX_REL = 0.005`
(2.3× MadGraph's own 0.22%), five seeds, and the pull asserted: MadGraph's error
dominates the combination, so no budget here can drive the pull up.[^n28-c][^n28-c24]
It is blind below about 0.2%. Its five-seed ladder over an eightfold budget
(75k to 600k) showed no resolved asymptote, so it is converged at the scale the
comparison is made at, not demonstrably asymptotic.[^n28-c24][^manifest] Before
the vector-vertex sign fix ([vector-vertex signs](../amplitudes/vector-vertex-signs.md))
the gate read `+0.33%`, the tightest measured cell in the banked layer (clearing
`rel_tol` by 1.5×), which is why it runs five seeds rather than three. The
manifest's current reading, after that fix, is `+0.18%` (pull `+0.80`) over the
same five seeds at `75 000 × 10`. The doc comments on `JJ_SEEDS`, `JJ_NEVAL` and
`JJ_MAX_REL` in `validate_hadronic.rs` still quote the pre-fix `+0.33%` and
ladder.[^manifest]

Two cells stay open on purpose. `diagrams` is `uncovered`: we count 15
topologies to MadGraph's 17, the whole deficit `g g > g g` (4 against 6),
because the four-gluon contact term is written once here and once per colour
structure there ([diagram enumeration](../process/diagram-enumeration.md)).
`amplitudes` is `uncovered`: `gg_to_gg` and `uux_to_uux` cover two
subprocesses, and the others would need parton rows.[^manifest] Nine of its
10 000 events replay their scale only to a `√(1 + 10⁻⁶)` tie-break inflation;
see [scale replay](scale-replay-gate.md) and
[the missing jj cluster dump](../backlog/hygiene/pp-to-jj-tie-break-no-cluster-dump.md).[^n28-k44]

## The llj family

Three proton runs share one generate line family and differ in exactly one
dimension each:[^manifest][^n24-p0]

| row | orders | scales | PDF | `mmll` |
|---|---|---|---|---|
| `pp_to_llj_fixed` | `QCD=2 QED=2` | all three fixed at 91.188 | `lhapdf`, 247000 | 50 |
| `pp_to_llj_dyn` | `QCD=2 QED=2` | `fixed_*_scale` off (MadGraph's clustering, `dynamical_scale_choice = -1`) | `lhapdf`, 247000 | 50 |
| `pp_to_llj` | default | dynamical | `lhapdf`, 247000 (re-carded from `nn23lo1`; see [refdata σ comparability](refdata-sigma-comparability.md)) | 0 |

- **`pp_to_llj_fixed`** proves the hadronic chain for a coloured initial state,
  a three-body final state, a jet cut and a strong coupling, the four things the
  Drell-Yan rows cannot see. Every per-event scale field is replayable exactly.
  `mmll = 50` keeps it off the low-`m_ll` photon-pole region. Its proc card
  differed from that of the since-pruned `pp_to_llj_qcd2_qed2` (below) only in
  the `output` line MadGraph writes into it.[^n24-p0]
- **`pp_to_llj_dyn`** differs from it in the three fixed-scale booleans and
  nothing else (its `SCALUP` takes 9966 distinct values over 10 000 events,
  against 1). A σ that agrees on the fixed row and disagrees here isolates the
  scale prescription, which is how the configuration-draw fix was
  attributed.[^manifest] See
  [the configuration draw](../scales-pdf/clustering-configuration-draw.md).
- **`pp_to_llj`** pairs with the explicit-order rows to pin coupling-order
  semantics (the two spellings must select the same diagram content), and is
  the only banked σ drawn from the photon-pole side of `m_ll`, regulated by
  `ptl = 10` alone. That region makes it the hardest budget of the re-carded
  rows.[^manifest]

The four partonic llj rows bank the amplitudes per class: `uux_to_epemg` as
baseline, `gu_to_epemu` varying the colour arrangement, `ddx_to_epemg` the
initial flavour, and `gux_to_epemux` the antiquark. Each is a
single-subprocess `lpp = 0` run, so `launch` builds both `matrix1_optim.f` and
the per-diagram `matrix1_orig.f`.[^n24-p0] They have no clustering dump, so
their scale reference is blind to a wrong tie-break or line PDG that does not
move the final number.[^n28-k44]

A fourth spelling, `pp_to_llj_qcd2_qed2`, was pruned: its event payloads were
byte-identical to `pp_to_llj`'s (the explicit orders coincide with the
defaults for this process) and its kT dump matched field for field, so the
census counted one measurement twice. Coverage was unchanged, because
`pp_to_llj_fixed`/`_dyn` carry the explicit spelling.[^n28-z1]

## Where the gates' calibrations stand

The llj gates' budget comments in `validate_hadronic.rs` still quote ladder
readings from before later sampler changes, including a "monotone climb" on
`pp_to_llj` that a 40-seed ensemble showed to be a five-seed misread
([seed sweeps](seed-sweeps-and-budget-ladders.md)). Re-recording them is
[llj gate comments quote pre-floor ladders](../backlog/hygiene/llj-gate-comments-quote-pre-floor-ladders.md).
For the gates themselves see [the σ gate](sigma-gate.md).

[^n24-p0]: Note 24 P0, "What was banked".
[^n28-d]: Note 28 §6, decisions D1, D2 and D4.
[^n28-s26]: Note 28 S2.6.
[^n28-b3]: Note 28 S4 B3.
[^n28-k44]: Note 28 K4.4.
[^n28-s6]: Note 28 S6, "Measured" and "The row, promoted".
[^n28-c]: Note 28 C.3–C.6.
[^n28-c24]: Note 28 C2.4.
[^n28-z1]: Note 28 Z.1.
[^manifest]: `validation/manifest.toml`, each row's `rationale` and cell notes.
[^diagram-rs]: `vibegraph-lib/src/diagrams/diagram.rs`, `fermion_line_sign`.
