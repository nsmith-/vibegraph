---
type: Validation Gate
title: kT clustering dump oracle
description: "Instrumented MadGraph 3.7.1 dumps every candidate, merge, frame change and scale per banked event; validate_kt_cluster must match all of it. What the dump and the gate cannot see."
status: draft
tags: [kt-clustering, scales, madgraph, oracle, per-event]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n28-k110, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L1146-L1267", title: "Note 28 K1.10 — what an instrumented run must record"}
  - {id: n28-k111, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L1268-L1298", title: "Note 28 K1.11 — findings for the engine"}
  - {id: n28-k31, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2021-L2064", title: "Note 28 K3.1 — every dumped event reproduces"}
  - {id: n28-k33, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2096-L2137", title: "Note 28 K3.3 — isbw is stale across events"}
  - {id: n28-k34, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2138-L2170", title: "Note 28 K3.4 — the dump cannot name a process directory"}
  - {id: n28-k36, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2199-L2219", title: "Note 28 K3.6 — branch coverage"}
  - {id: n28-k37, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L2220-L2244", title: "Note 28 K3.7 — confirmed against the bank"}
  - {id: n28-z3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L4159-L4226", title: "Note 28 Z.3 — green having compared nothing"}
  - {id: code-kt, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/validate_kt_cluster.rs", title: "vibegraph-lib/tests/validate_kt_cluster.rs"}
  - {id: code-manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/validation/madgraph/kt_cluster_dump_manifest.json", title: "validation/madgraph/kt_cluster_dump_manifest.json"}
  - {id: mg-cluster, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/cluster.f", title: "MadGraph Template/LO/SubProcesses/cluster.f"}
  - {id: mg-reweight, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/reweight.f", title: "MadGraph Template/LO/SubProcesses/reweight.f"}
---

# kT clustering dump oracle

The scale MadGraph writes per event (`SCALUP`, the `<rscale>`) is a coarse
oracle for its kT clustering: it cannot see a wrong tie-break on an event where
both candidates measure the same, nor a wrong PDG on a line the beam walk never
asks about. And `clusinfo` is gated on `ickkw ≠ 0` (`unwgt.f:838`), so no banked
LHE at `ickkw = 0` carries a `<clustering>` tag. An instrumented MadGraph build is
the only route to the merge sequence[^n28-k111]. The algorithm being checked is
[MadGraph kT clustering](../scales-pdf/kt-clustering-algorithm.md) and
[setclscales](../scales-pdf/setclscales.md); this crate's engine is
[the kT clustering engine](../scales-pdf/kt-clustering-engine.md).

## The dump

`validation/madgraph/gen_kt_cluster_dumps.sh` replays the banked runs through the
pinned MadGraph 3.7.1 (`b7687064`) with `Template/LO` instrumented. Records are
written per *written* event, in write order: `setclscales` runs on every phase-space
point, so the instrumentation buffers per `ivec` and lets `write_leshouche` flush
it when the event is emitted, the way `use_syst` already carries
`s_scale(ivec)` from `reweight.f` to `unwgt.f`[^n28-k110]. Reals are printed
`%24.17E`; the LHE's ten digits cannot replay an `(E − p_z)(E + p_z)`
cancellation. The pipe-separated records cover:

- **per process directory** (at `initcluster.f:48`): run constants (`ktscheme`,
  `ickkw`, `xqcut`, `scalefact`, the fixed-scale switches, `bwcutoff`,
  `dynamical_scale_choice`, …), `nqcd` per config, the `id_cl` merge map, the
  PDG and resonance tables, and every config's forest from `configs.inc`;
- **per event**: `iproc`, `iconfig`, `ivec`, the momenta as `cluster()` received
  them, the Breit–Wigner list, each `cluster()` call (`reweight.f:666`, `:998`,
  `:1028`);
- **per clustering attempt and pass**: every candidate pair with admissibility,
  the arm of the measure (`IS_DJB`, `IS_PYJB`, `FS_DJ_DURHAM`, `FS_DJ_HAD`,
  `FS_DJ_MLESS_MASSIVE`, `FS_SUMDOT_BW`, `FS_PYDJ`), raw and inflated values and
  the graphs `findmt` leaves alive; the winner; every merge with its leg sets,
  kind, `pt2ijcl`, `zcl`, `mt2ij` and PDGs; every frame change; the core and the
  `igraphs` collapse around `cluster.f:811-817`;
- **`setclscales`**: the beam walk's lines and jet flags, `jfirst`/`jlast`/
  `jcentral` raw and final, the override flags, the vertex scales at each stage,
  the μF and μR branch ids (`MUF` among `NEXT3, GEOM, GEOM_COLLAPSED, JC0_*,
  PDFWGT`; `MUR` by its `reweight.f` line), and the output scales and coupling.

**Precondition, checked before the dump is trusted**: re-running the banked cards
through the instrumented build reproduces `unweighted_events.lhe` byte-for-byte
modulo run metadata, so the `k`-th record is the `k`-th banked event. The
dump's `OUT` record must reproduce each event's own `SCALUP`, which proves the
instrumentation reads the live path.

`validation/madgraph/kt_cluster_dump_manifest.json` pins the dumps: eight runs at
`787070e`, 10 000 events each, all matched — the no-closed-form rows `pp_to_llj`,
`ee_to_mumua`, `ee_to_mumu_tata_qcd0`, `bbx_to_ccx_emmm_qcd0`,
`uux_to_ccx_emmm_qcd0`, and the controls `uux_to_uux` (the tie-break row),
`pp_to_bb_qcd2` (both the `gg` beam-leg route and the `qq̄` `mt2last` route) and
`ee_to_ttx` (the coloured-final-state discriminator) — with a SHA-256 per file and
a per-run `coverage` table of branch counts.

## The gate

`vibegraph-lib/tests/validate_kt_cluster.rs` is an **oracle-layer** gate: its
tests are `#[ignore]` with that reason, and
`pixi run -e madgraph validate-kt-cluster` builds the dumps (75 MB, outside the
reference bundle) and runs them. Absent dumps are an assertion failure naming the
task, not a skip. (In the banked layer the gate was once green on every fetching
checkout having compared nothing, because the dumps are not in the
bundle[^n28-z3].)

Given per event the integration channel, subprocess, momenta and the directory's
forests, the engine derives the whole merge graph (leg sets, line PDGs, resonance
map, coupling-order filter), the Breit–Wigner tagging, the clustering, the jet
memo's re-cluster decision, the beam walk and both scales. The comparison is
ordered — **merge sequence first, scales second** — and reports the first
divergence by merge index. Every attempt is compared, not only the accepted one.
Agreement is `AGREEMENT = 1e-12` relative on every scale and measure; observed
worst `0.0` on this platform (same expressions, same inputs, same order),
reported rather than required because a system libm may differ in the last
place[^n28-k31].

**Non-vacuity is asserted**: the engine must reproduce each run's `coverage`
counts branch for branch (`candidate_measure`, `boost`, `memo`, `mur_branch`,
`muf_branch`, `beam_crossing_inflation`, `cluster_calls_per_event`,
`igraphs1_is_iconfig`, `mt2last_override`, `jcentral_override_beam*`), and fails
if it takes a branch the reference never took.
`derived_channel_forests_match_the_generated_ones` checks the bijection between
this crate's derived channel forests and the generated ones, with the QCD order
that partitions them.

## Diagnosed exceptions

- **`isbw` is stale across events.** `cluster.f`'s on-shell flags live in a
  common block that `checkbw` clears only for the current integration channel's
  own timelike lines, so a leg set flagged under one channel stays flagged when
  the next event runs under another. It fires on 81 events of
  `bbx_to_ccx_emmm_qcd0` and 163 of `uux_to_ccx_emmm_qcd0` (both 2 → 6 with three
  Z bosons and a Higgs) and nowhere else. The engine takes a
  `carried_on_shell` argument; an event is first compared with it empty, and only
  on disagreement re-run with the reference's own extra flags. All 244 then
  reproduce completely, so the divergence is entirely that one input[^n28-k33].
  Production owns its own state, so the pure-function reading is correct there;
  a pure replay of a banked 2 → 6 run cannot reach those events.
- **The dump cannot name a process directory.** The per-directory tables carry
  no directory name, and the extraction de-duplicates them by text, so a run
  spanning several directories (`pp_to_bb_qcd2`, `pp_to_llj`) merges them, and
  `NQCD` collides outright. Forests are separable (an `IFOR` row's length names
  the directory); `nqcd` is re-derived from the forest; an event's directory is
  decided by a model test on its forest's vertices, with the event's own candidate
  list consulted once per undecided flavour assignment[^n28-k34]. A re-bank should
  carry the directory key
  ([kt-dump-tables-lack-directory-key](../backlog/hygiene/kt-dump-tables-lack-directory-key.md)).

## What the scale is not a function of

MadGraph's scale is not a pure function of (momenta, process): `filmap`'s
`nqcd(this_config)` filter, `checkbw`'s use of `this_config`, and the `njetstore`
memo with its restricted re-cluster (`reweight.f:985-1030`) all depend on the
channel or history. All three are live on the bank: the coupling-order filter on
`pp_to_bb_qcd2`, `igraphs(1) ≠ iconfig` on 7 to 7877 events per run, 1857
restricted re-clusters on `pp_to_llj`[^n28-k37]. See
[cluster-scale channel dependence](../scales-pdf/cluster-scale-channel-dependence.md).
Also confirmed against the bank: the `uux_to_uux` tie-break (32 inflated
candidates over 16 events), the dead mass-propagation guard implemented as
`A .or. B`, `mt2last` set only after a final-state last merge, and `ipartupdate`'s
in-event mutation of `ipdgcl`.

## Coverage the bank cannot judge

From the manifest's `coverage` union over the eight runs, these implemented
branches are reached by no dumped event[^n28-k36]:

- `ktscheme = 2` (`IS_PYJB`, `FS_PYDJ`);
- `dj`'s second massless–massive arm and its zero-three-momentum guard (only
  `FS_DJ_MLESS_MASSIVE_1` fires, on `pp_to_llj`);
- μR branches other than `L1153` and `L1169`; μF branches other than
  `GEOM_COLLAPSED` and `JC0_BOTH`;
- the `2 → 1` short-circuit, the `xqcut`/`xmtc` refusals, the μF floor refusal,
  and a fixed scale on one beam only.

`scalefact ≠ 1` is not in any dump but is pinned at the scale level by the
`pp_to_ll_scalefact2` row's replay ([the scale replay gate](scale-replay-gate.md)).
`ickkw = 1` is not in these dumps; the matched runs have their own instrumented
replay, [the MLM dump oracle](mlm-dump-oracle.md). `pp_to_jj`'s nine beam-crossing
tie-break events are enforced by signature only, with no dump
([pp-to-jj-tie-break-no-cluster-dump](../backlog/hygiene/pp-to-jj-tie-break-no-cluster-dump.md)).

*Blind to*: cross-event state the engine deliberately does not carry (the
`ipdgcl` common block, the per-directory jet memo) beyond the two diagnosed
exceptions; branches no banked run reaches.

[^n28-k110]: Note 28 K1.10, with the instrumentation points per record.
[^n28-k111]: Note 28 K1.11 findings 2 and 7.
[^n28-k31]: Note 28 K3.1.
[^n28-k33]: Note 28 K3.3.
[^n28-k34]: Note 28 K3.4.
[^n28-k36]: Note 28 K3.6, re-checked against the manifest's `coverage` tables at `787070e`.
[^n28-k37]: Note 28 K3.7, and the manifest's `pp_to_llj` counts.
[^n28-z3]: Note 28 Z.3.
