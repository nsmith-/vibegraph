---
type: Design
title: The matched event record for the shower
description: "What an ickkw = 1 event carries for Pythia's MLM matching: <scales pt_clust_N>, status-2 lines of the clustered configuration, <MGRunCard>, matched SCALUP/AQCDUP."
status: draft
tags: [events, mlm, lhef, pythia, matching]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n41-flow, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L49-L68", title: "Note 41 §1.1, call flow per point"}
  - {id: n41-setcl, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L69-L104", title: "Note 41 §1.2, setclscales under matching (ptclus)"}
  - {id: n41-record, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L146-L190", title: "Note 41 §1.4, cuts, setup and the event record"}
  - {id: n41-colour, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L276-L283", title: "Note 41 §3.4, colour from the clustered graph"}
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L303-L530", title: "Note 41 M0, references and the dump oracle"}
  - {id: n41-m4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1415-L1616", title: "Note 41 M4, the event record for the shower"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2916-L3489", title: "Note 41 Z, close-out (bundle and cell flips)"}
  - {id: mg-addmothers, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/madgraph/iolibs/template_files/addmothers.f#L253-L268", title: "MadGraph 3.7.1 addmothers.f, the status-2 rule"}
---

# The matched event record for the shower

A run with `ickkw = 1` (MLM matching, proton beams only) writes the extra
record fields a shower's MLM matching reads. At `ickkw = 0` none of them is
written and the file is byte-identical to a plain run's. The matching itself —
the `xqcut` clustering cut, the two `setclscales` calls, `rewgt` — is
[scales-pdf/mlm-matching](../scales-pdf/mlm-matching.md),
[scales-pdf/mlm-scales](../scales-pdf/mlm-scales.md) and
[scales-pdf/mlm-rewgt](../scales-pdf/mlm-rewgt.md); the run-card side is
[run-card/matching-parameters](../run-card/matching-parameters.md). This
concept records only what lands in the event file.

## What a matched record carries

| field | rule | where |
|---|---|---|
| `SCALUP` | `√max(q2bck)` when `pdfwgt` is set, otherwise the second call's `q2fact`; the record scale, not the lowered scale the densities read | `EventScales::mu_f_record`, `build::scalup` |
| `AQCDUP` | `αs` at the **first** call's μR (the second call never recomputes μR) | |
| `<scales pt_clust_N="v" …>` | one entry per outgoing line, after the particles; `N` the line's 1-based position in the event (so shifted by status-2 lines), `v` its `ptclus` printed as `f16.5` and trimmed | `build::pt_clust_scales` |
| status-2 lines | timelike lines of the **clustered** configuration whose leg set the integration channel put on its Breit–Wigner | `SubprocessResonances::clustered_on_shell`; CLI `matched_event_record` |
| colour flow | drawn in the clustered configuration's `ICOLAMP` row | `select_config_and_flow(…, clustered)` |
| `<MGRunCard>` | the run card as MadGraph records it (below), in the header | `lhef::write::mg_run_card` |
| `IDPRUP` / `<init>` | the event's `@N`; one `<init>` line per `@N` with `LPRUP = N` | |

Not written: `<clustering>` (CKKW-L) and `<mgrwt>` (systematics), which are out
of scope[^n41-record]. The per-field conventions shared with unmatched files
are [events/lhef-record-conventions](lhef-record-conventions.md); how a mixed
`@0 + @1 + @2` card's weights and `<init>` lines are normalised is
[events/multi-process-normalisation](multi-process-normalisation.md).

### `ptclus` and the collider-energy fallback

Per final-state leg, `ptclus` is the largest clustering `√pt2ijcl` among the
jet vertices the leg takes part in while still a good jet; otherwise the
**collider** `√stot`, not `√ŝ` (`reweight.f:1225-1269`)[^n41-setcl]. vibegraph
reads it off the second call's finished clustering
(`coupling/cluster/setclscales.rs`, `ptclus`). Details the plan text lacked
and the port follows: the `goodjet` demotion on a non-jet `ipart` leg is applied
before the vertex test of the same entry; the terminal vertex's daughters are
its first beam line and the leftover line; `etot` is `genps.f`'s formula with
the proton mass 0.938, which cancels analytically and lands on
12999.999999999998, printed `13000.00000`[^n41-m4].

Pythia's `Beams:setProductionScalesFromLHEF = on` reads these as each parton's
starting scale, and its MLM matching leaves out a light parton whose scale is
the collider energy ([events/pythia-interop](pythia-interop.md)). That makes
`<scales>` load-bearing: deleting it from vibegraph's files moves the `@1`
matching acceptance by +25σ ([validation/mlm-pythia-matched-comparison](../validation/mlm-pythia-matched-comparison.md)).

### Status-2 lines under matching

`addmothers` takes its configuration from the clustering's graph
(`igraphs(1)`, `addmothers.f:109-116`) and, under `ickkw > 0`, writes a timelike
line as status 2 when `isbw(idij(i))` is set[^mg-addmothers]:

```fortran
if(ickkw.eq.0.and.OnBW(i))then
   jpart(6,i)=2                 ! resonance whose mass is preserved
else if (ickkw.gt.0) then
   if(isbw(idij(i))) then
      jpart(6,i)=2
   else
      jpart(6,i)=3              ! documentation only, not written
```

`isbw` is filled by `checkbw` over the **integration channel's** propagators
(`cluster.f:386-432`, re-run on each `cluster` call), keyed by leg set; the
clustered configuration contributes only the line list and the codes. So the
rule is: a timelike line of the clustered configuration is written when its leg
set is one the channel put on its Breit–Wigner (`MatchedRecord::on_shell`) — not
"if the clustering found it on its Breit–Wigner". No line is written when the
drawn flow is below leading colour in that configuration (`is_LC`,
`addmothers.f:129-131, 195`). The layout (mothers `1 2`, momentum the daughters'
sum, mass their virtuality, colour what they leave open, `SPINUP 9`) is the
decay-chain one, [events/resonance-records](resonance-records.md). Under
matching the resonance tables are built for every card, not only decay-chain
ones[^n41-m4].

Two caveats bound this reading. `isbw` entries are cleared only for the
channel's own leg sets, so a clustered configuration with a timelike leg set
the channel lacks would read a stale flag; and on every banked MLM row the
clustered configuration and the integration channel give the Z the same code,
so no gate yet distinguishes MadEvent's two possible readings:
[validation/resonance-code-readings-unseparated](../backlog/validation/resonance-code-readings-unseparated.md).

### Colour from the clustered graph

Under `ickkw > 0` MadEvent's `select_color` masks with `igraphs(1)` instead of
the integration channel (`super_auto_dsig_group_v4.inc:1120-1142`), and
`addmothers` writes from the same graph[^n41-flow][^n41-colour]. The proton
selection passes the drawn term's `EventScales::clustered_config`
(`igraphs(1) − 1`) to `select_config_and_flow`, which then skips the `AMP2`
draw; at `ickkw = 0` it is `None` and the configuration is drawn as usual
([events/colour-and-helicity-selection](colour-and-helicity-selection.md)).
On `pp_to_llj_mlm` the clustered graph differs from the integration channel on
525 of 10000 events[^n41-m0].

## `<MGRunCard>`: MadGraph's record of the card, not the resolved card

MadGraph writes the card after `banner.py`'s own edits and before the
Fortran's (`setrun.f`, `setcuts.f`), which rerun on any card read back. So the
record keeps the card's `ptj` and `mmjj` (their rewrite to `xqcut` is
`setcuts.f`'s), zeroes `drjj`/`drjl` under `xqcut`, zeroes `mmjj` above `xqcut`
only without `auto_ptj_mjj`, and forces `alpsfact = 1` only under matching with
`use_syst`. `RunCard::banner_values` keeps those few values beside the resolved
ones (`#[serde(skip)]`, so no artifact changes)[^n41-m4]. On the mixed row both
sides record `mmjj = 0.0` while the resolved card holds 20.

MadGraph wraps the card in `<![CDATA[ … ]]>`; vibegraph writes it as element
text with only `<`, `>`, `&` escaped, because Pythia 8.312 drops CDATA content
and would otherwise read a 4-byte card under `JetMatching:setMad = on`
([validation/pythia-setmad-drops-cdata-run-card](../backlog/validation/pythia-setmad-drops-cdata-run-card.md)).
`mg_run_card_matches_madevents_banner` (`validate_mlm_dumps.rs`) compares the
card vibegraph writes with MadEvent's banner field by field: every shared field
agrees but `iseed` (MadEvent records its run seed; vibegraph's generator seed is
a flag naming another stream). MadEvent alone writes eight empty list
parameters; vibegraph alone writes the hidden ones MadEvent leaves out[^n41-m4].

## Evidence and its limits

- **Per event against MadEvent's dumps** (`validate_mlm_dumps`, oracle layer):
  `ptclus`, the file's `<scales>` string, and the status-2 lines (code, legs,
  mothers, mass, colour) agree on every gated event of the four matched rows
  (10000/10000 on the llj rows; on the mixed and `t t̄` rows every non-permuted
  event, 9840 and 6256 against the `refdata-9` regeneration, with the
  permuted-`P1` events read `info`)[^n41-m4][^n41-z]. Every record field also
  agrees on the permuted events: the record reads the second call, which
  clusters `P1` on both sides
  ([validation/madgraph-permuted-first-call](../validation/madgraph-permuted-first-call.md)).
  These samples cells are `gate`, flipped from regenerated dumps since the
  dumps are outside the bundle, and render ⏳ because the dump test writes no
  collator row ([validation/mlm-dump-oracle](../validation/mlm-dump-oracle.md),
  [validation/mlm-dump-gates-no-collator-row](../backlog/validation/mlm-dump-gates-no-collator-row.md)).
- **The written sample** (`cli_generate_proton`):
  `a_matched_sample_carries_the_shower_record` (structure of `<MGRunCard>`,
  `<scales>` keys, leptons at `13000.00000`, the Z record) and
  `matched_sample_record_fractions_against_madevent` (the Z on 0.9543 of events
  against 0.9507, a jet at the collider energy on 0.1183 against 0.1123).
- **What only the shower sees**: whether Pythia's matching reads `<scales>` as
  intended. That is the matched comparison through Pythia
  ([validation/mlm-pythia-matched-comparison](../validation/mlm-pythia-matched-comparison.md)),
  which stays informational.
- No `t t̄` event lists a resonance (the tops are external), and no Drell–Yan
  event lists a photon.

[^n41-flow]: Note 41 §1.1, item 4: colour and mothers use the clustered graph.
[^n41-setcl]: Note 41 §1.2: `ptclus` and the collider-energy fallback.
[^n41-record]: Note 41 §1.4, the record list (`<clustering>`, `<mgrwt>` out of scope).
[^n41-colour]: Note 41 §3.4.
[^n41-m0]: Note 41 M0: `vec_igraph` against the integration channel; `LPRUP = N`.
[^n41-m4]: Note 41 M4 Landed: the record fields, the plan corrections, the CDATA finding, the per-event comparison.
[^n41-z]: Note 41 Z2 landed: dump counts against the regenerated dumps and the samples-cell flips.
[^mg-addmothers]: MadGraph 3.7.1 `addmothers.f:253-268`.
