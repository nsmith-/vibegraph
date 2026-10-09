---
type: Validation Gate
title: Amplitude oracle against MadGraph AMP, JAMP and AMP2
description: "amplitude_oracle compares per point, helicity, diagram and flow with one fitted unit constant per process, asserts MadGraph's AMP2 configuration grouping, and reads banked event momenta."
status: draft
tags: [amplitudes, madgraph, oracle, colour-flow, configurations]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-p1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L704-L755", title: "Note 24 §P1 — the per-diagram gate and the c_i·AMP(i) correction"}
  - {id: n25-amps, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L127-L139", title: "Note 25 §3.2 — the amplitudes category"}
  - {id: n25-events, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L326-L354", title: "Note 25 §5.3 — amplitudes on MadGraph's own events"}
  - {id: n25-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L585-L621", title: "Note 25 §10 — what the layering landed"}
  - {id: n28-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L1916-L1955", title: "Note 28 B3 — the W-current row's 35 vs 21 configurations"}
  - {id: n28-s5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2286-L2304", title: "Note 28 S5 — ud_to_epemud_qcd0 registered as an amplitude row"}
  - {id: n28-s5-landed, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2379-L2394", title: "Note 28 S5 — KNOWN_LINEAR_DISAGREEMENT as a two-way list"}
  - {id: n28-s6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2685-L2710", title: "Note 28 S6 — measured after the crossing sign rule; the config-index oracle bug"}
  - {id: n29-f1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L353-L437", title: "Note 29 §F.1 — the fitted constants G and k"}
  - {id: n29-f3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L492-L560", title: "Note 29 §F.3 — hostile cases and the dump schema"}
  - {id: n29-f4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L561-L605", title: "Note 29 §F.4 — table schema and harvesting"}
  - {id: n29-f5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L606-L663", title: "Note 29 §F.5 — vacuity modes and what cannot be decided"}
  - {id: n29-f8, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L738-L773", title: "Note 29 §F.8 — what the dumps pin"}
  - {id: n29-f10, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L830-L976", title: "Note 29 §F.10 — k/G is one bit per configuration"}
  - {id: n29-f12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L1037-L1095", title: "Note 29 §F.12 — the sign of G tracks MadGraph's colour coefficients"}
  - {id: n29-f13, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L1096-L1152", title: "Note 29 §F.13 — rows without per-diagram coefficients"}
  - {id: n29-f14, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L1153-L1207", title: "Note 29 §F.14 — k-phase and sign-pattern hardening drafts"}
  - {id: n29-f15, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L1208-L1224", title: "Note 29 §F.15 — what the dumps cannot decide"}
  - {id: n35-v2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L1113-L1190", title: "Note 35 V2 — config_groups is IdentifyConfigTag; partitions as sets"}
  - {id: n35-wpwmz, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L1435-L1461", title: "Note 35 §10.5 — wpwm_to_wpwmz_cw's partition is not a shift"}
  - {id: n39-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/39-vector-vertex-signs.md#L73-L123", title: "Note 39 §3 — the standalone JAMP oracle and zero widths"}
  - {id: n39-gates, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/39-vector-vertex-signs.md#L172-L208", title: "Note 39 §6 — gates and mutation table"}
  - {id: code-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/amplitude_oracle.rs", title: "vibegraph-lib/tests/amplitude_oracle.rs"}
  - {id: code-standalone, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/standalone_jamps.rs", title: "vibegraph-lib/tests/standalone_jamps.rs"}
  - {id: mg-group-subprocs, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/group_subprocs.py", title: "MadGraph group_subprocs.py (IdentifyConfigTag)"}
---

# Amplitude oracle against MadGraph AMP, JAMP and AMP2

`vibegraph-lib/tests/amplitude_oracle.rs` is the gate behind the `amplitudes`
column of the validation report. It is hermetic: it reads only the committed
tables under `validation/madgraph/amplitudes/<key>.json` (one trial per table, 48
tables at `787070e`) and runs in about a second. Run it with
`pixi run validate-amplitudes`, or with `--skip-deps` to avoid regenerating the
tables. Per-row enforcement follows the row's `categories.amplitudes.mode` in
`validation/manifest.toml`; the manifest also lists the gate as the
`amplitudes-mg` standalone.

## What a table holds

Each table carries two labelled point sets and MadGraph's values at each[^n25-events]:

- **`event` points**: the first ~24 events of the row's banked MadGraph run,
  projected exactly on shell. LHE momenta are printed to ~11 significant digits,
  so a read-back point is off shell by ~1e-10, and two independently compiled
  programs then legitimately disagree through gauge-dependent parts. The
  committed file holds the projected momenta, so the Rust side never re-derives
  them. These points sit where the cross section lives.
- **`grid` points**: a fixed RAMBO grid from `gen_amplitude.py`, which covers the
  off-peak corners an event sample under-visits.
- At every point, the colour- and helicity-summed `|M|²`.
- At a few points of each set (`detail`, 6 per file on the original rows), per
  helicity row: `AMP(1:NGRAPHS)` as `detail.amps[hel][graph][re,im]` and
  `JAMP(1:NCOLOR)` as `detail.jamps[hel][flow][re,im]`[^n29-f4].
- MadGraph's colour coefficients `c_j` of `JAMP = Σ c_j AMP(j)` where they could
  be parsed (`jamp_coefficients`), the `AMP2` accumulator groups
  (`amp2_groups`), and the param card MadGraph evaluated with, so both sides use
  the same rounded inputs.

`bbx_to_ccx_emmm_qcd0` and `uux_to_ccx_emmm_qcd0` (615/579 diagrams) bank flows
only, so no per-diagram statement can be made about them. `pp_to_ll_qcd0` is the
one row whose event points come from a hadronic run, boosted into the partonic
centre of mass.

## What is asserted, per process

- The helicity set is MadGraph's `NHEL` table, and the diagram count is
  `NGRAPHS` where per-diagram amplitudes are banked.
- **One complex constant `G`**, fitted by least squares over every
  (point, helicity, diagram) entry, satisfies `A_i^vg = G · c_j · AMP_j^mg` for
  the diagrams *and* `J_f^vg = G · JAMP_f^mg` for the flows, at
  `LINEAR_REL_TOL = 1e-12` of the largest reference entry. The comparable object
  is `c_j · AMP_j`, not `AMP_j`: MadGraph puts the annihilation/exchange relative
  sign into the colour coefficient (`e+ e- > e+ e-` has `c = (−1,−1,+1,+1)`),
  this crate puts it into the diagram root, and only the product is
  observable[^n24-p1]. One constant for diagrams and flows together is what ties
  this crate's colour coefficients to MadGraph's.
- `|G| = 1` (a uniform rescaling is the one error the fit absorbs with zero
  residual, and it would rescale every `JAMP2` weight) and `Re G = 0`. The second
  is derived, not fitted: the UFO coupling carries the vertex `i` and the
  propagator its `−i`, so a tree with `V` vertices and `V − 1` propagators
  carries `i^V (−i)^{V−1} = i` exactly. MadGraph's `AMP()` leaves that `i` out.
  See [the global phase](../amplitudes/global-phase-i-counting.md). All rows
  measure `G = ±i`; only the sign is free, and it is printed, not asserted.
- Helicity combinations MadGraph omits (its amplitude vanishes) must vanish on
  this side too, for flows and for configuration amplitudes.
- `eval_jamp2` reproduces `Σ_hel |JAMP_f^mg|²` flow by flow, against MadGraph's
  JAMPs rather than this crate's, so it does not inherit the fit.
- **The integration configurations are MadGraph's.** `config_groups`
  (`helas/eval/compile.rs`) implements MadGraph's `IdentifyConfigTag`
  (`madgraph/iolibs/group_subprocs.py`, the rule behind `get_amp2_lines`'
  `config_map` branch): an external leg is its leg number, an internal line the
  propagating particle's `(colour, mass, width)`, the vertex interaction looked
  up and discarded, with MadGraph's Z/H-as-photon substitution on spacelike
  lines[^n35-v2]. The derived partition is compared with the `AMP2` groups of the
  generated `matrix1.f` **as sets of MadGraph graph indices**, mapped through
  `MG_DIAGRAM_ORDER`; a comparison of group sizes would compare two orderings
  rather than two partitions. A configuration's admitted colour flows must equal
  each member diagram's, which is what the merged `ICOLAMP` mask rests on.
- Each configuration amplitude (`run_config_amps`) equals MadGraph's bare
  `AMP()` up to a per-configuration constant `k`, with residual and `|k| − 1`
  both under `1e-12`. `|k| = 1` is what makes `AMP2` agree.
- `eval_amp2` reproduces MadGraph's `AMP2`, built from the banked amplitudes as
  the **coherent** `|Σ AMP|²` over each merged group, at `AMP2_REL_TOL = 1e-12`.
  This crate's `eval_amp2` is coherent within a configuration and the integrator
  runs on these same merged configurations (see
  [the channel set](../phase-space/channel-set.md) and
  [per-diagram AMP2](../amplitudes/per-diagram-amp2.md)).
- The helicity-pruned evaluator (production `eval_m2`) is bit-for-bit the
  unpruned one at every point; how far pruning moves `AMP2` is measured and
  printed, since dropping a combination whose coherent amplitude cancels does not
  make each diagram vanish.

`|M|²` is judged at `GRID_REL_TOL = EVENT_REL_TOL = 1e-12`, except that an
event point may also pass within `ULP_BUDGET = 10` times its own one-ulp
sensitivity. Only two points need that: `u u~ > c c~ e+ e- mu+ mu-` events whose
four-lepton mass sits on the Higgs pole, at 2.00 and 3.51 times their
sensitivity; that process is `NCOLOR = 1`, and its per-flow comparison holds at
1.5e-14 there.

## Diagram pairing and the three exemption lists

`MG_DIAGRAM_ORDER` banks the MadGraph graph index of each vibegraph diagram where
the two enumeration orders differ (`ee_to_tatah` is `[3,4,1,2,0]`). It is banked,
not searched, so a reordering on either side fails instead of being re-matched;
the pairing is over-determined by every helicity at every point under one
constant, so banking it is not fitting it[^n24-p1]. MadGraph lists a vertex's
Lorentz structures in the reverse of this crate's order, which is why SMEFT rows
pair by reversing each topology's block. For `g g > g g` and `gg_to_gg_cg` the
list is not a permutation: MadGraph writes each four-gluon contact as three
colour-ordered `AMP()`s.

Each list is **two-way**: an entry fails when its reason goes away.

| list | means | at `787070e` |
|---|---|---|
| `KNOWN_CONFIG_MERGE` | MadGraph merges several diagrams into one accumulator; each entry says why | 14 rows, e.g. `ee_to_ee` (γ/Z t-channel), `ud_to_epemud_qcd0` (35 diagrams, 21 accumulators), the 2 → 1 rows, the SMEFT rows whose SM and operator structures share a topology |
| `KNOWN_CONFIG_PAIRING_UNAVAILABLE` | no per-diagram table, so no pairing to compare partitions through; both partitions are printed and `\|M\|²`, JAMPs and JAMP2 still run | `wpwm_to_wpwmz_cw` (222 graphs; the partitions share only the multiset of group sizes, `{3×8, 7×8, 12, 17×4}`, and are not a shift of each other[^n35-wpwmz]) |
| `KNOWN_LINEAR_DISAGREEMENT` | the comparison runs and records instead of failing, so an `info` cell carries real numbers | **empty** — every row is gated at the linear level |

A manifest `amplitudes` cell may be `info` independently of these lists. At
`787070e` three are: `ee_to_wpwm_cw` (linear level exact; one grid point's
`|M|²` at 2.078e-12, 336× its own ulp sensitivity, with a helicity sum cancelling
by 875), `ee_to_zh_smeft` (MadGraph's Fortran rounds `GC_303`'s UFO literal to
seven digits; see [the coupling oracle](coupling-oracle.md)), and
`wpwm_to_wpwmz_cw` (`|M|²` 2.79e1 after the vector-vertex sign fix; the residual
belongs to O_W's five-vector structures, tracked as
[wpwmz-cw-ow-five-vector-residual](../backlog/validation/wpwmz-cw-ow-five-vector-residual.md),
and its partition as
[wpwmz-cw-config-partition-uncompared](../backlog/validation/wpwmz-cw-config-partition-uncompared.md)).

## The fitted constants carry bits, not phases

Measured over all 113 configurations of the banked set: `k/G ∈ {+1, −1}`
exactly, suite-wide worst residual 1.19e-13 (`ee_to_mumu_tata_qcd0`). Where
`jamp_coefficients` are banked, that bit *is* MadGraph's `c_j`; where they are
not (`gg_to_gg`, `gg_to_ttx`, `ud_to_epemud_qcd0`, `uux_to_uux` all bank
`jamp_coefficients: null`), the oracle runs no per-diagram fit, and `k/G` is the
only per-diagram sign oracle there is[^n29-f10][^n29-f13]. So the fitted content
is `1 + Σ N_config` bits, most of them already banked in the reference.

The sign of `G` tracks MadGraph's own colour-coefficient sign, not any invariant
of this crate: `e+ e- > mu+ mu-` and `u u~ > mu+ mu-` have identical topology
and fermion lines yet read `G = −i` and `+i`, with MadGraph's `c = (−1,−1)`
against `(+1,+1)`[^n29-f12]. A `G` sign is therefore never evidence about this
crate.

The gate asserts only `|k| = 1`. Asserting `|Im(k/G)| < LINEAR_REL_TOL` and
banking the per-process `run_config_amps` sign patterns (non-uniform on
`ee_to_tatah`, `ee_to_mumua`, `ee_to_mumu_tata_qcd0`) are open as
[config-amp-phase-and-sign-unpinned](../backlog/validation/config-amp-phase-and-sign-unpinned.md).
Asserting that `run_config_amps` *equals* the per-diagram amplitudes would fail
on those three rows, and asserting uniformity would be false[^n29-f14].

## The standalone JAMP gate

Processes no MadEvent row carries are compared by
`vibegraph-lib/tests/standalone_jamps.rs` against `output standalone` builds
(`gen_standalone_jamps.py`, tables in `validation/madgraph/standalone/`): every
NHEL helicity, every flow, at a few RAMBO points, one fitted `G` with
`|G| = 1`, `REL_TOL = 1e-12`[^n39-oracle]. Standalone output keeps the width of
a spacelike propagator; MadEvent zeroes it (`zerowidth_tchannel`) and this crate
follows MadEvent, so rows with a massive t-channel line (`ee_to_wpwmz`,
`wpwm_to_wpwm`, `ttx_to_gg_chg`, `wpwm_to_epem`) are generated with every width
zero on both sides (`zero_widths = True`). Through such a line the difference is
0.1–0.5 % per helicity, not a sign. Rows at `787070e`: `gg_to_ggg`,
`uux_to_ggg`, `ug_to_ug`, `ee_to_wpwmz`, `wpwm_to_wpwm`, `ttx_to_gg_chg`,
`tata_to_ttxh`, `tata_to_ttxhh`, `bbx_to_hh`, and `wpwm_to_epem` as a two-way
known disagreement
([wpwm-to-epem-neutrino-exchange-sign](../backlog/validation/wpwm-to-epem-neutrino-exchange-sign.md)).
It is blind to the phase of `G`, to compensating errors in diagrams reaching the
same flows with the same weights, and to kinematics outside its points[^n39-gates].
`g g > t t~ g` has no amplitude gate at all
([gg-ttxg-no-amplitude-gate](../backlog/validation/gg-ttxg-no-amplitude-gate.md)).

## Known blind spots

- A phase common to every diagram and flow is absorbed into `G`. It is genuine
  convention freedom (MadGraph chooses the overall sign of `JAMP(1)` per colour
  structure); `Re G = 0` narrows it to a sign.
- The split of a diagram's amplitude into the convention sign `φ_d` and the
  sign-free evaluation `H_d` is invisible: the gate passes iff `ρ_d/φ_d` is
  constant per process for any replacement `ρ`[^n29-f8]. Moving a sign between the
  two can never be validated by these tables.
- For `g g > g g`, trace-reversal partners carry identical JAMPs
  (`J₁ = J₆, J₂ = J₄, J₃ = J₅`), so swapping such a pair is invisible here, to
  `JAMP2` and to `|M|²`; `color_flow_tags_oracle` sees it. See
  [the colour oracles](colour-oracles.md).
- The CF matrix is not re-derived here; `color_cf_oracle` pins it.
- Majorana lines, loop level and Lorentz structures no banked model writes have
  no reference here[^n29-f5].
- `G` and `k` are not banked in the report row (`AmplitudesRow`), only the
  deviations; harvesting them needs `--nocapture` (G's sign) or a scratch copy of
  the harness (k).

Which sign channel each row exercises, and which channels have no varying
instance, is [convention-channel coverage](convention-channel-coverage.md).
Localising a failure of this gate is
[bit-exact amplitude debugging](bit-exact-amplitude-debugging.md); the sign
inventory itself is [convention signs](../amplitudes/convention-sign-inventory.md).

[^n24-p1]: Note 24 §P1, "New gate: `amp_diagram_oracle`".
[^n25-events]: Note 25 §5.3, "`amplitudes` on MG's own events".
[^n29-f4]: Note 29 §F.4, the table schema.
[^n29-f5]: Note 29 §F.5, what the investigation cannot decide.
[^n29-f8]: Note 29 §F.8, the `A_d = φ_d · H_d` framework.
[^n29-f10]: Note 29 §F.10, the harvested `k/G` dataset.
[^n29-f12]: Note 29 §F.12, witness pair W2.
[^n29-f13]: Note 29 §F.13 item 4.
[^n29-f14]: Note 29 §F.14 (a) and (b).
[^n35-v2]: Note 35 V2, channel grouping.
[^n35-wpwmz]: Note 35 §10.5.
[^n39-oracle]: Note 39 §3.
[^n39-gates]: Note 39 §6.
