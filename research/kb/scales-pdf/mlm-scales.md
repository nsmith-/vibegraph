---
type: Algorithm
title: "setclscales under matching: two calls, two factorisation scales"
description: "MadEvent's first and second setclscales calls, q2bck, the central-scale overwrite and its jcentral guard, ptclus, and vibegraph's density versus record factorisation scale."
status: draft
tags: [mlm, scales, setclscales, q2bck, ptclus]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n41-flow, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L49-L68", title: "Note 41 §1.1 (call flow per point)"}
  - {id: n41-12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L69-L104", title: "Note 41 §1.2 (setclscales under matching)"}
  - {id: n41-31, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L225-L234", title: "Note 41 §3.1 (two factorisation scales, one record scale)"}
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L303-L530", title: "Note 41 M0 (references, the extended replay, censuses)"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 M1 (implementation and dump gates)"}
  - {id: n41-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1260-L1345", title: "Note 41 D2 diagnosis (H1 sized)"}
  - {id: n41-m4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1415-L1616", title: "Note 41 M4 (ptclus and the record)"}
  - {id: n41-z2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L3403-L3489", title: "Note 41 Z2 (re-verified from refdata-9)"}
  - {id: mg-reweight, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L1103-L1269", title: "MadGraph reweight.f (setclscales under matching, ptclus)"}
  - {id: scales-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/coupling/scales.rs#L58-L87", title: "EventScales"}
---

# `setclscales` under matching: two calls, two factorisation scales

Under `ickkw = 1` MadEvent calls `setclscales` **twice** per point: once from
`update_scale_coupling` before the matrix element, and once inside `rewgt`
(`reweight.f:1465`) with `keepq2bck = .true.` ([scales-pdf/mlm-matching](mlm-matching.md)
has the call flow).[^n41-flow] The unmatched walk is [scales-pdf/setclscales](setclscales.md); this
concept is what matching changes. Line numbers are `reweight.f` at the pinned 3.7.1 tree.[^mg-reweight]

## What matching changes in the walk

- **No early return.** With `ickkw = 0`, `xqcut` is a pure cut and the routine returns
  after it (`:1103-1106`). With `xqcut > 0` or `ickkw = 1`, every event is clustered, even
  on a card that fixes every scale (`:643`). The `xqcut` rejection (`:1063-1089`) is
  `ScaleRefusal::JetCut` and zero-weights the term.
- **Two factorisation scales per event.**[^n41-12]
  - the **central** one, `q2bck = sf²·√(pt2(jlast)·pt2(jcentral))` (`:1141-1144`);
  - the **matrix-element PDF** one, `sf²·min(pt2(jfirst), central)` when `pdfwgt`
    (`:1195-1203`).

  `DSIG` reads the densities at the second, lowered scale, before `REWGT` runs
  (`auto_dsig1.f`, `QSCALE = DSQRT(Q2FACT(…))`).

## The second call

It enters with the first call's `(μR, q2fact)` still set, and differs from the first in
four ways:[^n41-12][^n41-m0]

1. **The central-scale overwrite** (`:1114-1119`). Because `q2fact` is non-zero on entry,
   `pt2ijcl(jcentral(1)) = q2fact(1)`, and `pt2ijcl(jcentral(2)) = q2fact(2)` **only if
   `jcentral(2) ≠ jcentral(1)`**. On `pp_to_llj_mlm` both beams' `jcentral` is the core
   vertex on every event, so one overwrite applies; it moves the core scale on 7641 of
   10000 events.
2. **`μR` is never recomputed.** `scale` is non-zero on entry, so the `MUR` branch is not
   entered (on all 10000 events). `rewgt`'s reference coupling `asref` is
   `αs(μR of the first call)`, and so is `AQCDUP`.
3. **`scalefact` is applied again** (`:1138-1147`).
4. **`q2bck` keeps the first call's value.**

After `rewgt`, `q2fact = q2bck` (`:1789-1791`), but only when `pdfwgt` is set. What
`rewgt` multiplies in from the second call's clustering is
[scales-pdf/mlm-rewgt](mlm-rewgt.md).

## What vibegraph carries

`ScaleChoice::cluster_history` makes both calls, sharing one jet memo; the second enters
with the first call's `(μR, q2fact)`, which triggers the `:1114-1119` overwrite in
`setclscales.rs` under `ickkw > 0`. `ClusterScales::q2central` is the value `:1141-1144`
stores in `q2bck`. The result is `EventScales` (`coupling/scales.rs`):[^scales-rs][^n41-m1]

| field | value under `ickkw = 1` | read by |
|---|---|---|
| `mu_r` | the first call's `μR` | the coupling, `AQCDUP`, `rewgt`'s `asref` |
| `mu_f` | the first call's `q2fact` (the lowered matrix-element PDF scale) | the density rows |
| `mu_f_record` | `q2bck` when `pdfwgt`; the second call's `q2fact` otherwise | `SCALUP` |
| `clustered_config` | `igraphs(1) − 1` | the colour-flow draw and the mothers |

Without matching, `EventScales::unmatched` sets `mu_f_record = mu_f` and
`clustered_config = None`, so an unmatched artifact or LHE file does not depend on the
split; a test pins that.[^n41-31] `setrun.f:82` clears `pdfwgt` at `ickkw = 0`; every
reader tests `ickkw > 0` anyway.

**`SCALUP` is `√max(q2bck)`** on MadEvent's scalar path. The vectorised path
(`vector_size > 1`) restores the lowered PDF scale instead (`auto_dsig_v4.inc:401`), so
every reference pins `vector_size = 1`, and vibegraph follows the scalar path. On the
single-multiplicity `ℓℓj` rows `SCALUP` never sees the split: it is the larger of two
scales and matching lowers only the smaller. The `t t̄` row (194 events) and the mixed
`0j2j` row (351) are what exercise it. Record semantics in general:
[scales-pdf/record-scales](record-scales.md).

## The jet memo across both calls

The memo (`njetstore`, `:662-679`, `:985-1030`) is consulted on every call that clusters,
so on both calls under matching. The restricted re-cluster branch fires often: on
`pp_to_llj_mlm`, 1123 events (11.2%) were scaled from a clustering restricted to their
integration channel. Where it fires is a property of the channel: in every job of
`P1_gq_llq` channels 1 and 2 the stored count is 0, and every other job stores 1.
The port stores this event's channel-restricted count, and
`the_jet_memo_is_the_channel_s_restricted_jet_count` measured, on all five MLM rows, that
every one of the 135 channels gives a single count over every event of its directory.
So the port's memo equals MadEvent's on every event, not only a channel's first.[^n41-m1]
See [scales-pdf/cluster-scale-channel-dependence](cluster-scale-channel-dependence.md).

## `ptclus`

`ptclus` (`:1225-1269`, `setclscales.rs::ptclus`) is computed from the second call, per
final-state leg:[^n41-m4]

- the largest `√pt2ijcl(n)` among jet vertices (`isjetvx`) at which the leg's daughter is
  still `goodjet`, reading `pt2ijcl` after every rewrite. A daughter's `goodjet` is
  demoted on a non-jet `ipart` leg before the vertex test of the same entry, and the
  terminal vertex's daughters are its first beam line and the leftover line;
- otherwise the **collider** energy `etot = √stot` (`genps.f:653-676`, with the proton
  mass 0.938, which cancels analytically), not `√ŝ`. At 13 TeV it evaluates to
  12999.999999999998 and prints as `13000.00000`.

It feeds `<scales pt_clust_N>`, which Pythia reads through
`Beams:setProductionScalesFromLHEF`; the record side is
[events/mlm-matched-event-record](../events/mlm-matched-event-record.md).

## The permuted first call

Grouped MadEvent hands the first call the **unpermuted** point `PP`
(`super_auto_dsig_group_v4.inc:842`), while the matrix element and the second call read
the permuted `P1 = SWITCHMOM(PP, PERMS(MAPCONFIG(ICONFIG)))`. Where the symmetry
permutation is not the identity, the first call clusters momenta exchanged between legs of
equal mass but different flavour against `P1`'s flavour table. vibegraph clusters both
calls on `P1`, the physically labelled point.[^n41-m1] **This is a documented deviation by
user decision (2026-09-29): neither reproduced nor refused.**

Its size is not in the kept events. On the regenerated dumps, 160 of `pp_to_ll_0j2j_mlm`'s
events are permuted and 77 of those have different first-call scales (all `@2`); their
weight factor, MadEvent's over vibegraph's, ranges 0.67–1.60 with mean 0.996.[^n41-z2] The
size is in the points only one side keeps, through first-call rejections: a patched
MadEvent measures `P2_qq_llqq` rising by +1.8%, about +0.25% of `@2`.[^n41-d2] Full
account: [validation/madgraph-permuted-first-call](../validation/madgraph-permuted-first-call.md).

**Reading the dump correctly.** `RWBEG`'s `q2fact` fields are the **second** call's
output, not the scales the densities were read at; those are the first call's `SCLOUT`
q2fact. The two agree on only 7284 of 10000 events. A weight-factor comparison that read
`RWBEG` as the density scale gets the wrong H1 size; `gen_kt_cluster_dumps.py`'s
docstring states the correct reading.[^n41-d2][^n41-m4]

## Validation

Per event against the instrumented MadEvent dump (`validate_mlm_dumps`, scales at 1e-12,
worst at or below 8e-15; `AQCDUP` worst 3e-16): every field of both calls agrees on
10000/10000 events of `pp_to_llj_mlm`, `_alps2` and `_xqcut_only`. On `pp_to_ttx_0j1j_mlm`
and `pp_to_ll_0j2j_mlm` the production path agrees on every non-permuted event, and the
permuted ones are reported `info, permuted P1`. Controls show the fields can tell the
readings apart: the record scale differs from the density scale on 8877 `ℓℓj` events,
and the clustered configuration differs from the integration channel on 525.[^n41-m1]
See [validation/mlm-dump-oracle](../validation/mlm-dump-oracle.md).

[^n41-flow]: Note 41 §1.1, the call flow.
[^n41-12]: Note 41 §1.2, `setclscales` under matching (its omissions corrected by M0's dump).
[^n41-31]: Note 41 §3.1, the scale split and byte identity at `ickkw = 0`.
[^n41-m0]: Note 41 M0, what the dump settles: the `jcentral` guard, `μR` not recomputed, the memo census.
[^n41-m1]: Note 41 M1, implementation, dump gates, the jet-memo proof and the permuted-`P1` finding.
[^n41-d2]: Note 41 D2 diagnosis, H1 sized, and the `RWBEG` misreading.
[^n41-m4]: Note 41 M4, `ptclus` and the corrected dump docstring.
[^n41-z2]: Note 41 Z2, counts on the dumps regenerated for `refdata-9`.
[^mg-reweight]: MadGraph `reweight.f`, `setclscales` under matching and `ptclus`.
[^scales-rs]: `EventScales`, `vibegraph-lib/src/coupling/scales.rs`.
