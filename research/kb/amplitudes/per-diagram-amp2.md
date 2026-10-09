---
type: Design
title: Per-diagram AMP2 and configurations
description: "Op::Configs roots and eval_amp2; MadGraph's get_amp2_lines configuration rule, KNOWN_CONFIG_MERGE, and the per-diagram phase oracle; weights applied at read-out."
status: draft
tags: [amp2, configurations, colour-selection, amplitude-oracle, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n27-b6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L912-L1038", title: "Note 27 §B6 (the per-diagram AMP2 accumulator)"}
  - {id: n27-findings, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L1223-L1243", title: "Note 27 §7 findings register (pruning moves AMP2; config merge)"}
  - {id: zen4-s7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/topdown-zen4-results.md#L299-L355", title: "Top-down Zen 4 results §7 (configuration amplitudes read bare, weighted at read-out)"}
  - {id: code-compile, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/compile.rs", title: "config_carrying_diagrams, config_groups, config_tag, select_config_and_flow"}
  - {id: code-run, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/run.rs", title: "eval_amp2, run_config_amps"}
  - {id: code-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/amplitude_oracle.rs", title: "amplitude_oracle: configurations, AMP2 and the per-configuration phase"}
  - {id: mg-amp2, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L1390", title: "MadGraph export_v4.py, get_amp2_lines (L1390) and get_icolamp_lines (L1295)"}
  - {id: mg-tag, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/group_subprocs.py#L56", title: "MadGraph group_subprocs.py, IdentifyConfigTag"}
---

# Per-diagram AMP2 and configurations

MadGraph's generated `matrix1.f` accumulates, besides `|M|²` and `JAMP2`, a
per-**configuration** array `AMP2(c) = Σ_h |Σ_{a ∈ c} AMP(a)|²`. MadEvent uses it
to pick each event's integration configuration, and through that the colour
flow the event is written with. Vibegraph reproduces the array, the
configuration set behind it, and the draw.

`AMP2` enters no integrand and no cross section. It decides which colour flow
(`ICOLUP`) an event carries and, where intermediate resonance records are
written, which ones. Vibegraph writes those status-2 records on decay-chain
cards and under matching; a plain process does not yet
([plain-process-onwindow-resonance-records](../backlog/feature/plain-process-onwindow-resonance-records.md)).

## Which diagrams get a configuration

`config_carrying_diagrams` (`helas/eval/compile.rs`) is MadGraph's
`get_amp2_lines` rule[^mg-amp2]: over the diagrams that have vertices, take the smallest
of their largest vertex arities, and drop every diagram whose largest vertex
exceeds it. In practice this keeps diagrams built from three-point vertices
and drops four-point contact diagrams. `g g > g g`'s four-gluon diagram gets no
`AMP2` and no configuration, so nothing can be drawn to it and its colour
structures never mask a flow. A contact diagram still contributes to `|M|²`
and to every JAMP it reaches; it gets no channel because it has no propagator
to enhance[^code-compile].

## How diagrams group into configurations

`config_groups` partitions those diagrams the way MadGraph's channel mapping
does, by `IdentifyConfigTag` (`madgraph/iolibs/group_subprocs.py`), the tag
`find_mapping_diagrams` keys `diagram_maps` by[^mg-tag]:

- an external leg is identified by its leg number, an internal line by the
  propagating particle's `(colour, mass, width)`; the interaction at each vertex
  is discarded;
- on an initial-state (spacelike) line a `Z` or `H` takes the photon's mass and
  width, so a t-channel γ and a t-channel Z are **one** configuration while the
  s-channel pair stays two;
- two diagrams that differ only in which interaction joins the same particles
  are one configuration. No SM process shows this, but a toy scalar carrying an
  `Identity` and a `Gamma5` bilinear on the same vertices does.

The tag is built from the baked propagator momenta (a tree is fixed by its
external-leg bipartitions), sorted over internal lines. Groups are numbered in
the order their first diagram appears, as `get_amp2_lines` numbers them.

The evaluator's configurations **are** these groups: `AmplitudeEvaluator::compile`
iterates `config_groups` and gives each group every `(diagram, colour chain)`
amplitude its diagrams own. A diagram that no colour chain references has no
amplitude, and a group left with none is dropped[^code-compile].
`config_amp_counts()` gives the number of amplitudes per configuration;
`config_diagram(c)` is the first member, the representative MadGraph's
`configs.inc` writes. A merged accumulator is one integration channel on both
sides; see [phase-space/multichannel](../phase-space/multichannel.md).

## The evaluator side

`Op::Configs` is a variadic root:
`(Configs <amplitude root> A_0 A_1 … A_{k−1})`. Child 0 is the amplitude root
proper (a single JAMP scalar, or the `Flows` node). The rest are the
colour-stripped amplitudes of the configuration-carrying diagrams, in
configuration order. They are the same lowered subtrees the JAMPs are built
from, referenced again, so the DAG stays one program, CSE is untouched and no
arithmetic is added. Like `Flows`, `Configs` computes nothing: keeping the wires
under the root keeps their slots live to the end of the pass, and the values
are read from the arena afterwards. The helicity expansion carries the bundle
through[^n27-b6].

The `|·|²` and the helicity fold are not arena nodes. The arena has no
real-valued square or fold op, and adding one would touch the op ↔ s-expression
bijection, the layout, the egraph schema and the op-coverage census for no
gain. `BoundAmplitude::eval_amp2` forms
`AMP2(c) = Σ_h |Σ_{a ∈ c} w_a · A_a|²` in Rust, as `eval_jamp2` does for flows.
Within a configuration the amplitudes are summed **coherently** and then
squared, which is `get_amp2_lines`' `(Σ AMP)·dconjg(Σ AMP)`[^code-run].

**Weights at read-out.** Each `A_a` is colour-stripped: no symmetry factor, no
Fermi sign, no colour coefficient, exactly as MadGraph's `AMP(a)` is. The
constant folding (`pair_config_weights` in `fold.rs`) splits each bundle
amplitude into a `(weight, value)` pair. The bundle pins the bare value;
the weight is a constant-pool leaf, or a unit coefficient when there is none,
carried in `Program::amp_weights` beside `amp_locs`. `eval_amp2` and
`run_config_amps` multiply the weights back in from the bound pools. This
leaves the JAMP sum as the product's only reader, so the
`sym·fermi·coupling·coeff` factor folds into that sum's `AddScaled` weight;
see [performance/constant-collection-and-fused-sums](../performance/constant-collection-and-fused-sums.md).
It removes about 21% of the 2→6 row's instructions but does not shrink the
scalar arena: every configuration amplitude stays pinned to the end of the
pass either way. Shrinking that arena would need `AMP2` accumulated inside the
pass, with an order that retires a level's amplitudes before the next level
fills[^zen4-s7].

## The draw

`AmplitudeEvaluator::select_config_and_flow(amp2, jamp2, [u0, u1], clustered)`
is MadEvent's `SELECT_COLOR`[^code-compile]:

1. Draw the configuration `c ∝ AMP2(c)` with `u0`. In a matched run,
   `clustered` (the configuration the clustering chose, MadEvent's
   `igraphs(1)`) replaces the draw and `u0` goes unread.
2. Draw the flow `∝ JAMP2(i)` with `u1`, restricted to the flows configuration
   `c` reaches: its `ICOLAMP` column (`config_flows(c)`, the union over the
   configuration's diagrams).
3. If no configuration carries weight, draw over every flow, the fallback
   `SELECT_COLOR` takes when its masked cumulant ends at zero.

A single-flow process reduces to a no-op. The selected configuration is also
the event's `ICONFIG`, the diagram whose s-channel propagators become
intermediate records where they are written, so colour flow and resonance structure come from one
configuration. The per-event wiring is
[events/colour-and-helicity-selection](../events/colour-and-helicity-selection.md).

The configuration is drawn, not taken from the sampler's channel. Under
single-diagram enhancement configuration `j`'s integrand carries
`AMP2_j / Σ_i AMP2_i`, so the configurations of written events follow that
share whatever the sampler did. For a process whose propagators are all
massless the per-diagram channel maps degenerate onto one another, and the
sampled channel carries no information about which diagram produced a point.

**Caveat: this is MadEvent's channel weight only under its default
integration strategy.** `SMATRIX` rescales every `AMP2(j)` by
`GET_CHANNEL_CUT` (the product of the configuration's inverse squared
propagator denominators), and under `sde_strategy = 2` it replaces the amplitude
by that product outright. That moves the written colour flow of a process whose
`ICOLAMP` rows separate the flows configuration by configuration; it never
reaches `|M|²` or σ.

## Helicity pruning moves AMP2

`|M|²` is bit-for-bit under helicity pruning, because the dropped combinations
are ~1e-30 of the coherent sum. The incoherent per-configuration sum has no such
protection: a combination is dropped when the coherent amplitude cancels
(typically by `J_z` conservation about the beam axis), but its individual
diagram amplitudes do not vanish. When the oracle was introduced the gap was
39.5% on `gg_to_ttx`, 3.2% on `gg_to_gg` and exactly 0 on every other
row[^n27-findings].

Production draws from the **pruned** evaluator. That is the analogue of
MadEvent's own `GOODHEL`-filtered accumulation (`|T| > ANS·LIMHEL/NCOMB`,
`LIMHEL = 1e-8`, against our 1e-24), and the measured `ICOLUP` frequencies
agree with MadGraph on it. The amplitude gate measures the gap on every run
(`amp2_pruned` in the report row) rather than assuming it; it is reported, not
asserted. Pruning itself is
[helicity-sum-and-pruning](../amplitudes/helicity-sum-and-pruning.md).

## The oracle

`tests/amplitude_oracle.rs` checks, on every banked amplitude table[^code-oracle]:

- **The partition.** `config_groups`' partition equals the `amp2_groups` banked
  from MadGraph's own generated `matrix1.f` (the `AMP()` indices each `AMP2()`
  accumulator sums), in MadGraph's order. `g g > g g` gives `[[3],[4],[5]]`
  (0-based `AMP()` indices, so `AMP(4..6)`): `AMP(1..3)`, the contact diagram's
  three colour structures, carry none.
- **`KNOWN_CONFIG_MERGE`** lists the rows whose derived partition merges
  diagrams, each with the reason. It has 14 entries. The SM ones are `ee_to_ee`
  (t-channel γ and Z share one; colourless, so the label cannot reach an
  event) and `ud_to_epemud_qcd0` (21 accumulators over 35 diagrams). The toy
  `ll_to_qqx_toy_yukawa` (the `Identity`/`Gamma5` scalar) is the row the rule
  was read for. The three 2→1 rows (`bbx_to_h_identity`, `gg_to_h_cpeven`,
  `gg_to_h_cpodd`) have no internal line, so MadGraph writes one accumulator
  over every graph. The remaining SMEFTsim and toy rows put the SM structure
  and the new ones on the same boson line, which is one topology to the channel
  mapping (`ee_to_ttx_dipole`, `ee_to_wpwm_cw`, `ee_to_zh_smeft`, `gg_to_gg_cg`,
  `wpwm_to_wpwmz_cw`, `ee_to_mumu_4f`, `ee_to_ttx_smeft`,
  `ll_to_qqx_toy_dipole`). The list is checked both ways: a listed row that
  stops merging fails, and an unlisted row that merges fails. It records why a
  row merges; it is not an exemption from the comparison.
- **The mask.** Each configuration's union of reached flows equals every member
  diagram's own, which is what makes the union equal MadGraph's
  one-representative `ICOLAMP` column.
- **The amplitudes.** Each configuration amplitude equals MadGraph's `AMP()` up
  to a **per-configuration** unit constant `k`, fitted over every
  `(point, helicity)` entry. It is per configuration, not global, on purpose:
  MadGraph puts the annihilation/exchange relative sign in the colour
  coefficient and vibegraph puts it in the diagram root, so a global fit would
  show a spurious residual. `|k| = 1` is the part with teeth; it is what
  `AMP2` rests on, and it fails for a stray symmetry factor or coupling.
- **`AMP2` itself.** `eval_amp2` reproduces `Σ_h |AMP^mg|²` per configuration
  (`AMP2_REL_TOL = 1e-12`, normalised to the largest `AMP2` at the point), and a
  helicity combination MadGraph omits must give zero configuration amplitudes
  here too.

Known gaps:

- **The phase of `k` is free, and the per-process sign pattern is not banked.**
  Only `|k| = 1` is asserted. Measured over every banked configuration,
  `k/G ∈ {±1}` exactly (`G` the process-wide `±i`), and
  `run_config_amps()[i]` differs from the single-diagram compile by ±1 in a
  per-process pattern that is non-uniform on three rows (`ee_to_tatah`,
  `ee_to_mumua`, `ee_to_mumu_tata_qcd0`). Neither is pinned. This is inert today:
  `eval_amp2` is sign-blind and `run_config_amps` has no production consumer.
  Open as [config-amp-phase-and-sign-unpinned](../backlog/validation/config-amp-phase-and-sign-unpinned.md).
- **`wpwm_to_wpwmz_cw`** has no banked diagram pairing, so its 21-group
  partition matches MadGraph's only in group sizes
  (`KNOWN_CONFIG_PAIRING_UNAVAILABLE`); see
  [wpwmz-cw-config-partition-uncompared](../backlog/validation/wpwmz-cw-config-partition-uncompared.md).
- **The diagram pairing** between our enumeration and MadGraph's is banked in
  `MG_DIAGRAM_ORDER`, not searched for, so a reordering on either side fails
  loudly instead of being re-matched.

[^n27-b6]: Note 27 §B6: the `Op::Configs` design, the oracle, and the selection rule. The colour draw is configuration ∝ `AMP2`, then flow ∝ `JAMP2` inside that configuration's `ICOLAMP` row.
[^n27-findings]: Note 27 §B6 and §7, findings 4 and 5. Its "MadGraph's merge is not derivable from the diagram list" is superseded: `config_groups` derives it with `IdentifyConfigTag`, and the evaluator's configurations follow it.
[^zen4-s7]: Top-down Zen 4 results §7; measured on an Emerald Rapids VM, the instruction counts per row in its table.
[^code-compile]: `vibegraph-lib/src/helas/eval/compile.rs`: `config_carrying_diagrams`, `config_groups`, `config_tag`, the configuration loop in `compile`, and `select_config_and_flow`. The doc comments on `config_groups` ("not the partition `AmplitudeEvaluator` integrates over") and `select_color_flow` ("these configurations stay one per diagram") contradict the compile loop, which uses `config_groups`; the code holds.
[^code-run]: `vibegraph-lib/src/helas/eval/run.rs`, `eval_amp2`.
[^code-oracle]: `vibegraph-lib/tests/amplitude_oracle.rs`: `KNOWN_CONFIG_MERGE`, `AMP2_REL_TOL`, `MG_DIAGRAM_ORDER`, the per-configuration fit, and `amp2_pruned`.
[^mg-amp2]: `get_amp2_lines` and `get_icolamp_lines` in `madgraph/iolibs/export_v4.py`.
[^mg-tag]: `IdentifyConfigTag` in `madgraph/iolibs/group_subprocs.py`.
