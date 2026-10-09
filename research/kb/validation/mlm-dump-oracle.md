---
type: Validation Gate
title: MLM per-event dump oracle
description: "Instrumented MadEvent replay and validate_mlm_dumps: engine replay and production path against both setclscales calls, rewgt factor by factor and the matched record, at 1e-12."
status: draft
tags: [mlm, oracle, per-event, scales, rewgt, madevent]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L303-L530", title: "Note 41 M0, references and the extended replay"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 M1, implementation and dump gates"}
  - {id: n41-m2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L758-L1013", title: "Note 41 M2, rewgt and its dump gates"}
  - {id: n41-m4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1415-L1616", title: "Note 41 M4, the event record for the shower"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2916-L3489", title: "Note 41 Z, close-out, B1 and Z2"}
  - {id: mg-reweight, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/reweight.f#L1787-L1791", title: "MadGraph reweight.f, q2bck restored only under pdfwgt"}
---
# MLM per-event dump oracle

A matched MadEvent run is replayed through an instrumented MadEvent that dumps
every intermediate of both `setclscales` calls and of `rewgt` for each written
event. `vibegraph-lib/tests/validate_mlm_dumps.rs` then recomputes those
intermediates and compares them field by field. It is the finest oracle the MLM
port has: a σ ([mlm-sigma-gate](mlm-sigma-gate.md)) integrates over everything, and a per-event field shows which
quantity is wrong and on which event. The semantics it checks are described in
[scales-pdf/mlm-scales](../scales-pdf/mlm-scales.md) and
[scales-pdf/mlm-rewgt](../scales-pdf/mlm-rewgt.md); it extends the unmatched
[kT clustering dump oracle](kt-cluster-dump-oracle.md).

## The dumps

- **Generator**: `gen_kt_cluster_dumps.sh` / `.py` with the `wrappers/ktdump*`
  instrumentation, run by the `mlm` stage of `generate-references`. The record
  types are documented field by field in `gen_kt_cluster_dumps.py`'s module
  docstring, which is the schema. Each written event's record set spans both
  calls: the first call opens it (its `SCL` record has `keepq2bck = F`),
  `rewgt`'s second call appends (`keepq2bck = T`), and `rewgt`'s records follow
  (`Q2OVR`, `Q2BCK`, `PTCL`, `RWLEG`, `RWBEG`, `RWVX`, `RWPDF`, `RWKILL`, `RWEND`,
  `CFG`, `CNT`; outside the sets `CONST2` and `MEMOX`).
- **Precondition**: the replay's event file must be byte-identical to the banked
  `run_01`, or the stage stops.
- **Where**: `output/ktdump/dumps/<row>.jsonl.gz`, pinned by
  `mlm_dump_manifest.json` (separate from `kt_cluster_dump_manifest.json`, so the
  kT gate does not iterate the matched runs). `dump_mlm_census.py` writes
  `mlm_census.json` from the dumps and process directories.
- **Mechanics**: a matched 2 → 3 flushes about 60000 record sets per 10000 kept
  events, so shards stream through gzip via a named pipe (`VG_KTDUMP_GZIP=1`) and
  are dropped after extraction (`VG_KT_DROP_RAW=1`). On Linux,
  `madevent_seeds.sh` and the replay link libstdc++ explicitly, because the conda
  activation's `LDFLAGS` suppresses `make_opts`' `STDLIB`.
- **Not banked.** The harness also reads each process directory's `configs.inc`,
  `config_nqcd.inc` and `config_subproc_map.inc`, which the bundle does not
  carry, so bundling the dumps would not make the gate banked. The dumps are
  regenerated on each bank host; two rows' MadEvent runs are not bit-reproducible
  across hosts, so the committed manifest pins the dumps the bank host
  gated[^n41-z].

## The harness

`pixi run -e madgraph validate-mlm-dumps`: oracle layer, `#[ignore]`, run with
`--ignored --test-threads 1`. Each event is read two ways:
- **engine replay**: each `setclscales` call runs from the exact state MadEvent
  entered it with (its momenta, memo and scales);
- **production path**: `ScaleChoice::cluster_history` on one momentum set, with
  the memo rule and the card as this crate resolves them.

Channel forests come from the event's own process directory (masses and widths
from the dump's `IFOR` rows). The run cards resolve against the dumped `CONST2`.
Fields are compared in order and each reports its first divergent event[^n41-m1].

| group | fields |
|---|---|
| scales | the jet-memo steps, branch and stored count against `njetstore` on entry; vertex scales after the rewrites; both calls' μR and `q2fact`; `q2central` against `Q2BCK CENTRAL`; `Q2OVR`; `q2bck`; `CFG` (`vec_igraph`); `SCALUP`; `AQCDUP`; no written event rejected by `xqcut` |
| `rewgt` | per vertex: class (`CORE`/`ISR`/`FSR`/`NONE`/`KILL_Q2`), lines, codes after `ipartupdate`, `ipart(1, mother)`, and on a reweighted vertex `kt²`, `αs(alpsfact·kt)` and the ratio; `asref`, `jlast`; per beam each chain step's vertex, flavour, action, `x` after `z`, `q²` now and before, both densities and the ratio; the kill; the product against `RWEND` and against the listed factors |
| beam order | the clustering's beam 1 takes the `x` of the physical beam its leg 1 arrives on, the order the mirrored term assumes |
| record | `ptclus` against `PTCL SETCL`/`OUT`; `<scales>` as the file's own string; status-2 lines (codes, legs, mothers `1 2`, mass, colour) |

`rewgt` is recomputed for the flavour combination MadEvent drew (`RWLEG`'s
`idup`) at `RWBEG`'s momentum fractions. Every tolerance is 1e-12; this crate's
NNPDF grid reading and `αs` agree with MadEvent's LHAPDF to a few ulp (worst
1e-15). `AQCDUP` is allowed 1e-6 and measures 3e-16[^n41-m2]. Beside it,
`mg_run_card_matches_madevents_banner` compares the `<MGRunCard>` this crate
writes with the banner field by field (only `iseed` differs), and
`matched_sample_record_fractions_against_madevent` (`validate-mlm-samples`)
compares record fractions of a generated sample.

## Semantics the dumps settled

- **The second call never recomputes μR.** `scale` is non-zero on entry, so
  `rewgt`'s `asref` is `αs` of the first call's μR. The overwrite
  `pt2ijcl(jcentral) = q2fact` has a guard, `jcentral(2) ≠ jcentral(1)`[^n41-m0].
- **`SCALUP` is `√max(q2bck)` only when `pdfwgt` is set**; otherwise it is the
  second call's `q2fact`, because `rewgt` restores `q2bck` only under `pdfwgt`
  (`reweight.f:1787-1791`)[^mg-reweight]. `setrun.f` clears `pdfwgt` at
  `ickkw = 0`.
- **`RWBEG`'s `q2fact` fields are the second call's output, not the densities'
  scales.** `DSIG` evaluates the densities at the first call's `q2fact` before
  `REWGT` runs; read the first call's `SCLOUT`. The generator's docstring says so.
- **The jet memo is the channel-restricted jet count.**
  `the_jet_memo_is_the_channel_s_restricted_jet_count` clusters every event of a
  directory restricted to every channel: each of 135 channels (8, 8, 8, 29, 82
  over the five rows) gives one count over every event, and the production
  path's stored count equals the dumped `njetstore` on entry on every event.
- **The `rewgt` factor is per flavour combination**, not per group: `DSIG` draws
  one `IPSEL ∝ PD(IPSEL)` and `rewgt` reads that combination's codes.
- **Status-2 resonances** are tested with `checkbw` on the integration channel's
  propagators and applied to the clustered configuration's timelike lines
  (`cluster.f:386-432`); none is written below leading colour (`is_LC`).
- **`<MGRunCard>`** is `banner.py`'s edited card, CDATA-wrapped
  ([events/mlm-matched-event-record](../events/mlm-matched-event-record.md)).

## Agreement

On the dumps regenerated for `refdata-9` (B1, Z2)[^n41-z]:

| row | gated (non-permuted) | permuted `P1`, reported as info |
|---|---|---|
| `pp_to_llj_mlm` | 10000, every field | — |
| `pp_to_llj_mlm_alps2` | 10000, every field | — |
| `pp_to_llj_xqcut_only` | 10000 (one call; `rewgt ≡ 1`) | — |
| `pp_to_ttx_0j1j_mlm` | 6256 | 3744, all agreeing |
| `pp_to_ll_0j2j_mlm` | 9840 | 160: 83 agree, 77 with other first-call scales |

The permuted events are [H1](madgraph-permuted-first-call.md): MadEvent's first
call clustered the unpermuted point, and the engine replay from `PP` reproduces
it on every one. The samples cells of `pp_to_llj_mlm`, `_alps2`,
`pp_to_ll_0j2j_mlm` and `pp_to_ttx_0j1j_mlm` are `long`/`gate` on this test.

## Controls

The matched fields have to be able to tell the readings apart, and do (counts
from the dumps the gates first ran on):
- the record scale differs from the density scale on 8877 `pp_to_llj_mlm` events;
- `CFG` differs from the integration channel on 525;
- `SCALUP` alone never sees the scale split on the llj rows (it is the larger
  scale, and matching lowers only the smaller); the `t t̄` row (194 events) and
  the mixed row (351) exercise it.

**Negative control, asserted.** Over MadEvent's events, σ without the `αs` ratios
is σ·⟨1/A⟩, `A` an event's product of ratios; this crate's and MadEvent's agree to
six digits, and the test requires more than a 1% effect:

| row | ⟨1/A⟩ | dropping the `αs` factor |
|---|---|---|
| `pp_to_llj_mlm` | 0.8547 | −14.5% (about 140 reference errors) |
| `pp_to_llj_mlm_alps2` | 0.9522 | −4.8% |
| `pp_to_ll_0j2j_mlm` | 0.9337 | −6.6% |
| `pp_to_ttx_0j1j_mlm` | 0.9046 | −9.5% |

`alpsfact = 2` is pinned by its own row: the dump shows the numerator read at
`2·kt` on every reweighted vertex.

## Blind spots

- **Only kept points.** A point MadEvent's first call rejects but `P1` keeps
  carries weight on one side only; that is where H1's size lives, and the dump
  cannot see it. Likewise any acceptance difference in a region MadEvent never
  populates.
- **The resonance code's two readings** (clustered configuration against
  integration channel) coincide on every MLM row
  ([resonance-code-readings-unseparated](../backlog/validation/resonance-code-readings-unseparated.md)).
- **Conditions the rows never reach**: no kill, no `NONE` or past-`jlast` chain
  step, no `fake_id`, no mixed jet-ness in an `IPROC`, `stop 4` never fired.
- **No collator row**: the test imports nothing from the report module, so its
  four gated samples cells render ⏳
  ([mlm-dump-gates-no-collator-row](../backlog/validation/mlm-dump-gates-no-collator-row.md)).

[^n41-z]: Note 41, Z.1 classification, B1 reproduction check and Z2 dumps.
[^n41-m1]: Note 41 M1, "Landed (dump gates)".
[^n41-m2]: Note 41 M2, "Landed (dump gates)".
[^n41-m0]: Note 41 M0, "What the dump settles about §1".
[^mg-reweight]: `reweight.f:1787-1791` at `b7687064`.
