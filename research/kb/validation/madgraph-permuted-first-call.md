---
type: Caveat
title: Grouped MadEvent sets scales on the unpermuted point
description: "Grouped DSIGPROC clusters and rejects the unpermuted PP while the matrix element reads P1; vibegraph clusters P1. A registered deviation (H1), with its measured size and reproducer."
status: draft
tags: [madgraph, madevent, scales, grouping, mlm, h1]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n07-h1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L539-L727", title: "Note 07 appendix, grouped MadEvent scales and rejects the unpermuted point"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L625-L757", title: "Note 41 M1 dump gates, the finding"}
  - {id: n41-m2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L877-L1013", title: "Note 41 M2 dump gates, permuted P1"}
  - {id: n41-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1215-L1414", title: "Note 41 D2 diagnosis, decisions and R1"}
  - {id: n41-m4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1415-L1616", title: "Note 41 M4 (record fields on permuted events)"}
  - {id: n41-z2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L3403-L3489", title: "Note 41 Z2, dumps against refdata-9"}
  - {id: mg-dsig, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc#L805-L842", title: "MadGraph super_auto_dsig_group_v4.inc, DSIGPROC"}
---
# Grouped MadEvent sets scales on the unpermuted point

## Mechanism

In grouped output (`group_subprocesses True`, MadGraph's default), `DSIGPROC`
in `super_auto_dsig_group_v4.inc` (pinned tree `b7687064`, 3.7.1)[^mg-dsig]:

1. builds `P1 = SWITCHMOM(PP, PERMS(MAPCONFIG(ICONFIG)))` (`:805`), the channel's
   symmetry permutation of the sampled point;
2. mirrors `P1` when `IMIRROR = 2` (`:814-826`);
3. calls `update_scale_coupling(pp, wgt)` (`:842`), handing the **unpermuted**
   `PP` to the first `setclscales` call:

```fortran
      IF (VECSIZE_MEMMAX.LE.1.and.imode.ne.5) THEN ! no-vector (NB not VECSIZE_USED!)
            call update_scale_coupling(pp, wgt)
      endif
```

The matrix element and `REWGT`'s second `setclscales` call read `P1`. The first
call sets μR, sets `q2fact` (which `DSIG` evaluates the densities at before
`REWGT` runs), sets `q2bck`, and can reject the point (the `xqcut` test on every
clustering vertex with a jet daughter, `reweight.f:1066-1085`, or the 2 GeV
factorisation floor). `REWGT` returns 0 when the second call rejects, so a point
is kept only if both accept. Where the permutation is not the identity, the first
call clusters an event whose momenta do not match the flavour table and diagram
of `P1`[^n07-h1].

Non-grouped output is unaffected: `P1` is `PP` boosted, with no permutation
(`auto_dsig_v4.inc:125-127`).

## Where it occurs

From the generated `symperms.inc` and `config_subproc_map.inc`:
- When an identical-flavour and a mixed-flavour subprocess share a group:
  `u u > z u u` and `u d > z u d` both land in `P1_qq_zqq`, whose configs 2, 5, 6
  and 8 are integrated as the permutation `(1,2,3,5,4)` of configs 1, 3, 4 and 7;
  two of `u d`'s four diagrams sit on configs 2 and 8, where the swap exchanges
  the physical `u` and `d`.
- In `p p > e+ e- j j`, `P1_qq_llqq` (the `P2_qq_llqq` of the mixed card) maps 10
  of its 24 configs, and the mixed-flavour `q q'` subprocesses have half their
  diagrams on them.
- `t ↔ t~` in `p p > t t~ j`: thousands of permuted events, scales almost never
  different.

It is **not limited to flavour swaps**: with matching off, a channel whose only
permuted config swaps `u u`'s identical quarks rises 5.7% (18σ) once fixed,
while another channel of the same kind does not move. Which clustering path makes
the difference is not isolated; the channel-restricted re-cluster of the jet memo
(`reweight.f:662-679`) is the suspect, unconfirmed.

## Size

**MadGraph-only reproducer** (`validation/madgraph/repro/permuted_first_call/`):
`define q = u d; generate u q > z u q` at 13 TeV, grouped against
`group_subprocesses False`, and grouped with the one-line fix
`validation/madgraph/patches/first-call-unpermuted-momenta.patch`
(`update_scale_coupling(p1, wgt)`). Five seeds per variant, each in a freshly
generated directory, error the spread over seeds[^n07-h1]:

| card | grouped (pb) | non-grouped (pb) | grouped + fix (pb) | grouped vs non-grouped | fix vs non-grouped |
|---|---|---|---|---|---|
| `ickkw = 1`, `xqcut = 40` | 54.48 ± 0.14 | 56.14 ± 0.10 | 56.33 ± 0.13 | −3.0%, −9.8σ | +1.2σ |
| default (`ickkw = 0`, `dynamical_scale_choice = -1`) | 104.28 ± 0.34 | 108.60 ± 0.45 | 108.62 ± 0.35 | −4.0%, −7.6σ | 0.0σ |

So it reaches unmatched runs at MadGraph's default dynamical scale. Every grouped
MadEvent run whose output has non-identity symmetry permutations and uses
`dynamical_scale_choice = -1` is exposed, matched or not.

**On `pp_to_ll_0j2j_mlm`.** Almost all of the effect is in points only one side
keeps, which the per-event dumps cannot see:
- on MadEvent's kept events the weight factor `rewgt · αs^n(μR) · f₁f₂(μF)`
  formed on each side's own scales moves `@2` by −0.01% ± 0.10%;
- patching MadEvent itself to hand the first call `P1` raises `P2_qq_llqq` by
  0.33 ± 0.11 pb (+1.8%, 18.063 → 18.394 pb), and `P2_gg_llqq` does not move;
- so H1 is about +0.25% of that row's `@2`, through first-call
  rejections[^n41-d2]. Its place in the `@2` budget is in
  [mlm-at2-excess-decomposition](mlm-at2-excess-decomposition.md).

A weight-factor ratio computed from `RWBEG`'s `q2fact` fields is wrong: those are
the second call's output, which equals the first call's on only part of the
events, while `DSIG` reads the densities at the first call's `q2fact`. Read the
first call's `SCLOUT` record instead (the dump docstring says so).

On the dumps regenerated for `refdata-9`[^n41-z2]:
- `pp_to_ttx_0j1j_mlm`: 3744 permuted events, every one agreeing on every field;
  none with other first-call scales;
- `pp_to_ll_0j2j_mlm`: 160 permuted events, 83 agreeing, 77 with other
  first-call scales, all `@2` (58 `P2_qq_llqq`, 19 `P2_gg_llqq`); the harness's
  informational weight-factor ratio between the two sides spans 0.67–1.60, mean
  0.996.

The event record is unaffected: `<scales>`, the resonances and colour come from
the second call, which clusters `P1` on both sides, and every record field agrees
on the permuted events too[^n41-m4].

## Decision and handling

H1 is documented, not reproduced and not refused (user decision). vibegraph
clusters both calls on the momenta the matrix element reads, which is the fixed
behaviour. Reproducing MadEvent would need its per-channel symmetry permutation
(`SYMCONF`, `PERMS(MAPCONFIG)`), which the integrand does not carry, since its
channels are its own diagrams; refusing would refuse the canonical matched card.
Under [the defect policy](madgraph-defect-policy.md) it is therefore a
registered deviation, listed with the other defects in
[madgraph-defects](madgraph-defects.md):
- the manifest notes of the affected rows name it;
- the per-event gates report permuted events as `info, permuted P1` and do not
  gate on them ([mlm-dump-oracle](mlm-dump-oracle.md));
- note 07 carries the upstream report draft; filing is the user's step
  ([madgraph-defect-reports-unfiled](../backlog/validation/madgraph-defect-reports-unfiled.md)).

A σ comparison against a grouped MadEvent reference with such permutations is
biased by a process-dependent amount (+1.8% on `P2_qq_llqq`, 3–4% in the
reproducer). Both scale paths involved are described in
[scales-pdf/mlm-scales](../scales-pdf/mlm-scales.md) and
[scales-pdf/clustering-configuration-draw](../scales-pdf/clustering-configuration-draw.md).

## Open

- **Banked `ickkw = 0` grouped rows are unchecked.** The banked scale gates
  compare against replays of `PP` and could not see it:
  [h1-ickkw0-grouped-rows-unchecked](../backlog/validation/h1-ickkw0-grouped-rows-unchecked.md).
- **The vectorised path** (`vector_size > 1`): `DSIG_VEC` calls
  `update_scale_coupling_vec(all_p, …)` (`:312`) before `DSIGPROC_VEC` builds and
  mirrors `ALL_P1`, which is local and differently dimensioned, so the fix there
  moves the call after the mirror loop with a copy. Read, not measured.
- **H1's share of `pp_to_ttx_0j1j_mlm`'s `@1` excess** is unmeasured
  ([ttx-mlm-at1-sigma-high](../backlog/validation/ttx-mlm-at1-sigma-high.md)).
- The clustering path behind the identical-quark channels.

Rejected candidates for the reproducer, which show no effect: `u u~ > e+ e- u u~`
alone (patched and unpatched bit-identical; its permuted channels carry 0.17% of
σ), `g g > e+ e- u u~` and `t t~ j` (their swaps leave the clustering pairs
symmetric), and the same subprocesses written with `add process`, which land in
separate directories with no permutation.

[^mg-dsig]: `super_auto_dsig_group_v4.inc:805,842` at `b7687064`.
[^n07-h1]: Note 07 appendix, "`super_auto_dsig_group_v4.inc` — Direct Bug Found".
[^n41-d2]: Note 41, D2 diagnosis ("H1 is real, and lives in `P2_qq_llqq`") and D2 decisions.
[^n41-z2]: Note 41, Z2 "Dumps".
[^n41-m4]: Note 41 M4, per-event record gates on permuted events.
