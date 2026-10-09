---
type: Validation Gate
title: Replaying MadGraph's per-event scales and alpha_s
description: "validate_scales replays each banked run's SCALUP, <rscale> and <pdfrwt> from the momenta and reproduces AQCDUP through the run's alpha_s source; every banked run is classified in its inventories."
status: draft
tags: [validation, scales, alpha-s, kt-clustering, lhef]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n24-p0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L446-L468", title: "Note 24 P0 (gate wiring: banking an amplitude banks a run)"}
  - {id: n24-p1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L683-L703", title: "Note 24 P1 (the q̄g gap closed)"}
  - {id: n24-p2b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1175-L1194", title: "Note 24 P2b (a grid-sourced bank is classified in two lists)"}
  - {id: n28-k38, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2245-L2270", title: "Note 28 K3.8 (the llj partonic rows have no dump)"}
  - {id: n28-k43, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2479-L2538", title: "Note 28 K4.3–K4.4 (every run through the clustering; decisions)"}
  - {id: n29-e5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1807-L1888", title: "Note 29 E.5 and acceptance tests (grid α_s runs join the AQCDUP oracle)"}
  - {id: n29-g6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5852-L5926", title: "Note 29 G.6–G.8 (classification guard; α_s source pinned by run logs)"}
  - {id: n29-g11, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5989-L5997", title: "Note 29 G.11 (what the σ cells cannot see)"}
  - {id: n40-oracles, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/40-per-group-dynamic-scales.md#L66-L151", title: "Note 40 §3 and §6 (per-event oracles for per-group scales)"}
  - {id: vscales, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_scales.rs#L1-L330", title: "validate_scales.rs module docs and inventories"}
  - {id: vscales-replay, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_scales.rs#L780-L860", title: "validate_scales.rs replay()"}
  - {id: valphas, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_alphas.rs#L100-L150", title: "validate_alphas.rs GRID_ALPHA_S_RUNS, GRID_ALPHA_S_TOL"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/unwgt.f#L751-L756", title: "MadGraph unwgt.f: SCALUP from q2fact"}
---

Every banked MadGraph run carries, per event, the scales MadGraph chose. The
scale-replay gate (`vibegraph-lib/tests/validate_scales.rs`, banked layer,
standalone row `scales-replay`) recomputes them from each event's printed
momenta through this crate's scale prescription and compares them with every
printed field. It is a per-event oracle, finer than any σ: a clustering that is
wrong event by event but right on average passes every cross-section cell and
fails here.[^n29-g11] The engine it exercises is
[the kT clustering engine](../scales-pdf/kt-clustering-engine.md); what the
fields mean is [record scales](../scales-pdf/record-scales.md).

## The oracle fields

| field | what MadGraph writes | precision |
|---|---|---|
| `SCALUP` | `sqrt(max(q2fact(1), q2fact(2)))`, the larger **factorisation** scale (`unwgt.f:751-752` at the pin)[^mg-unwgt] | 7 significant digits |
| `<rscale>` in `<mgrwt>` | `μR` itself (`reweight.f`'s `s_scale`) | 8 digits |
| `<pdfrwt beam="i">` | `μF` per beam | 8 digits |
| `AQCDUP` | `αs(μR)` | 7 digits; locates `μR` to about 1e-6 relative, finer than `SCALUP` |

`<mgrwt>` appears only with `use_syst`; runs without it are pinned by
`SCALUP` and `AQCDUP`. `SCALUP` doubles as `μR` only where the clustering reads
both from one vertex; the two massive-diquark toy rows
(`SCALUP_IS_NOT_MU_R`) are the general `q2fact(1) ≠ q2fact(2)` case, where the
`SCALUP`-vs-`μR` check is dropped and
`scalup_is_the_factorisation_scale_on_the_diquark_rows` measures both halves
instead.[^vscales]

**The budget is not a tolerance.** Each event's bound is the field's own
last-digit rounding plus how far the computed scale moves when each printed
momentum component is walked to the ends of its rounding interval (momenta are
printed to eleven digits). The incoming momenta are printed inputs too and are
walked as well. This matters: a forward leg's transverse mass is
`(E − p_z)(E + p_z)`, which can lose two orders of magnitude of precision before
any scale is formed. A reported worst case near 1.0 means the bound is tight,
not that the gate is marginal.[^vscales][^n28-k43]

## Configuration choice in the replay

MadGraph's cluster scale depends on the event **and** on the integration
configuration (coupling-order filter, resonance tagging), and an LHE record does
not carry the configuration. The replay adopts the first configuration whose
`μF` lands inside `SCALUP`'s budget and reads **every** other field, and the
independent `AQCDUP` oracle, off that same configuration, so a wrong clustering
cannot be repaired field by field.[^vscales-replay] How many events needed a
configuration other than the first is reported per run: on runs where it is
zero, the cluster scale is a function of the event alone. When every run was
first replayed (27 runs), it was non-zero on six: the `g q`-initiated llj rows
(7204 and 7231 events), `pp_to_llj_dyn`/`pp_to_llj` (5768 and 5572),
`ee_to_mumua` (370), `ee_to_mumu_tata_qcd0` (262) and `pp_to_bb_qcd2` (141,
a 2→2), and zero on the other twenty-one, `pp_to_jj` among them.[^n28-k43]

This is the oracle's search, not production's rule. Production draws each
point's configuration `∝ AMP2_c` per flavour group and beam ordering
([configuration draw](../scales-pdf/clustering-configuration-draw.md)). Two
further per-event oracles pin that rule:[^n40-oracles]

- `validate_hadronic::madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2`:
  on `pp_to_llj_dyn`, MadEvent's 7197 two-scale `q g → ℓℓq` events land at the
  higher scale 1429 times against 1488.3 expected (pull −1.83); the old
  unrotated-mirror reading expects 1748.8 (pull −9.31) and is asserted
  rejected. Blind to the `q q̄ → ℓℓg` groups (one scale per configuration).
- `cli_decay_chain_events::our_own_events_replay_in_their_own_flavour_group`:
  2000 generated events per card, each replayed in every configuration of its
  own group; one must return `SCALUP` to 1e-6 and `αs(μR)` must be `AQCDUP`.
  Blind to which configuration inside the group was drawn.

The draw's shares are pinned on one card only; `p p > t t~` and `p p > j j`
offer too few two-scale events. A 2→2 clusters every group to the same scale up
to rounding (`pp_to_jj` moves by 2e-9 relative), so those rows are blind to the
per-group and mirror errors.[^n40-oracles] A marginal `SCALUP` KS can pass
while two initial-state classes carry each other's scales; splitting the column
by class is what exposed it, but that split was a one-off diagnostic and is not
a committed column, so the `samples` gate's `SCALUP` KS is still the
marginal.[^n40-oracles]

## The inventories

Every banked run must be in exactly one coverage class, or the gate panics
naming the lists; a run directory the gate does not know is a failure, not a
skip.[^vscales] Classes (`Coverage` in `validate_scales.rs`):

| list | meaning |
|---|---|
| `CLUSTERED_RUNS` | `dynamical_scale_choice = -1` with a dynamic scale: every event clustered and replayed |
| `FIXED_SCALE_RUNS` (`pp_to_bb_fixed`, `pp_to_llj_fixed`, `ud_to_epemud_qcd0`) | all scales fixed; replayed with **no** forests supplied, so a fixed scale cannot hide behind a clustering result[^n24-p0] |
| `CLOSED_FORM_RUNS` (`gg_to_gg_cg`, choice 3) | one of `setscales.f`'s closed forms: momenta only, no channel ambiguity |
| `DECLINED_RUNS` | not replayed against the record, for a reason the gate *measures* (`declined_runs_decline_for_the_declared_reason`), so a lifted blocker fails and asks for promotion |

Declined reasons: `InstrumentedDump` (the two 2→6 rows: MadGraph's on-shell
flags carry across events, and 579/615 channels make a channel search a gate
almost anything passes; [the kT dump oracle](kt-cluster-dump-oracle.md)
reproduces all 20 000 events instead); `MatchedDump` (the five MLM rows, whose
`SCALUP` comes from the first of two `setclscales` calls;
[MLM dump oracle](mlm-dump-oracle.md)); `RefusedRunCard` (`wpwm_to_wpwmz_cw`,
`nhel = 1`).[^vscales]

Cross-cutting lists:

- `GRID_ALPHA_S_RUNS` (in both `validate_scales.rs` and `validate_alphas.rs`):
  the `pdlabel = lhapdf` runs, whose α_s MadGraph reads from the PDF grid
  (`alfas_functions_lhapdf.f`). `validate_scales.rs` lists the eight it replays;
  `validate_alphas.rs` lists thirteen, those eight plus the five MLM rows. It classifies which `AlphaSSource` arm a run
  takes. `validate_alphas::banked_run_logs_pin_the_alpha_s_source_rule` asserts
  set equality against the runs whose own MadGraph run log reports a grid α_s,
  and `resolve()` asserts the arm both ways, so a run that changes source fails
  instead of being reclassified quietly.[^n29-g6][^valphas] See
  [α_s sources](../scales-pdf/alpha-s-sources.md).
- `SCALEFACT_RUNS = [("pp_to_ll_scalefact2", 2.0)]`: the only run with
  `scalefact ≠ 1`, and the only oracle for where MadGraph applies it (once per
  scale on 3.7.1; `μF(beam 2) = μF(beam 1)` bit for bit in all 10 000
  events).[^vscales]
- `TIE_BREAK_MISSES = [("pp_to_jj", 9)]`: events admitted only with the exact
  `√(1 + 10⁻⁶)` crossing-inflation signature, count asserted for equality. Which
  of two degenerate candidates `cluster.f` chose is decided below the printed
  digits; settling it needs a `p p → j j` clustering dump
  ([backlog](../backlog/hygiene/pp-to-jj-tie-break-no-cluster-dump.md)).[^vscales]

**Banking a process banks a run.** `launch` builds `matrix1_optim.f` and writes
10 000 events, so registering an amplitude process adds a run that every
inventory must classify. A `pdlabel = lhapdf` run belongs in
`GRID_ALPHA_S_RUNS`, and a run whose `SCALUP` is its `μR` in
`validate_alphas`' `SCALUP_IS_THE_RENORMALISATION_SCALE` (the two
mixed-multiplicity MLM rows, where `SCALUP` and `μR` part on some events, are
deliberately left out of it).
All are asserted, so none can be forgotten silently.[^n24-p0][^n24-p1][^n24-p2b]
A re-bank that changes a run's α_s source leaves the gate red until the
classification moves with it; that is the guard working.[^n29-g6]

## The `AQCDUP` chain

`banked_events_reproduce_aqcdup_from_the_computed_scale` closes cluster scale →
`μR` → `αs(μR)` in one per-event comparison: it feeds the *computed* `μR`, not a
printed field, through `AlphaSSource::from_run_card(card, param_card_as, grid)`,
at a budget of half the printed digit plus how far the momenta move it. Each
factor is independently gated (`validate_alphas` reproduces `AQCDUP` from the
*printed* `SCALUP` through the same grid, at `GRID_ALPHA_S_TOL = 1e-14` for the
grid reading itself), so a failure here localises to the composition.[^n29-e5][^valphas]
Its negative control,
`the_grid_runs_need_the_grids_alpha_s_and_not_the_parameter_cards`, shows that
the parameter card's `αs(M_Z)` misses the printed field by about 2× its budget
on the fixed-scale grid runs, which is what makes the grid arm's agreement
informative; it asserts its own pinning set is non-empty.[^n29-g6]

Two other assertions ride on the replay: `banked_hadronic_runs_clear_the_factorisation_floor`
(the smallest replayed `μF` over every banked hadronic event clears the 2 GeV
floor; [factorisation-scale floor](../scales-pdf/factorisation-scale-floor.md))
and `every_banked_run_uses_the_clustering_default`.[^vscales][^n29-g6] Which
committed cards compile to which constants is `scales_run_cards.rs`, which reads
no events and runs hermetically.[^vscales]

## What it cannot see

- **Rows with no clustering dump** (the four llj partonic rows, among others):
  their only reference is the banked `SCALUP`/`<rscale>`/`<pdfrwt>`, which is
  blind to a wrong tie-break or a wrong line PDG that does not move the final
  number. What tests the path that produced their scale is the dumps on other
  processes.[^n28-k38][^n28-k43]
- **The geometric-mean form of the coloured two-body scale**: in a 2→2 with
  equal-mass legs `(djb₃·djb₄)^¼` equals either leg's `√djb`, and every banked
  coloured 2→2 has equal-mass legs.[^vscales]
- **Whether a fixed scale is right for the right reason**: a constant cannot
  tell `μR` from `μF` and no momentum moves it. The fixed rows' value is the
  controlled comparison with their dynamical twins.[^vscales]
- **The integration configuration** that MadGraph actually used, which the
  record does not carry (above).

The module doc of `validate_scales.rs` is partly out of date (it still says no
banked run pins `scalefact`); see
[validate_scales module doc stale](../backlog/hygiene/validate-scales-module-doc-stale.md).

[^n24-p0]: Note 24 P0, "Gate wiring".
[^n24-p1]: Note 24 P1, "The q̄ g gap, closed".
[^n24-p2b]: Note 24 P2b, item 3.
[^n28-k38]: Note 28 K3.8.
[^n28-k43]: Note 28 K4.3–K4.4.
[^n29-e5]: Note 29 E.5 and its acceptance table.
[^n29-g6]: Note 29 G.6–G.8.
[^n29-g11]: Note 29 G.11.
[^n40-oracles]: Note 40 §3 and §6.
[^vscales]: `vibegraph-lib/tests/validate_scales.rs`, module docs and inventories.
[^vscales-replay]: `vibegraph-lib/tests/validate_scales.rs`, `replay`.
[^valphas]: `vibegraph-lib/tests/validate_alphas.rs`.
[^mg-unwgt]: MadGraph `unwgt.f` at `b7687064`:
    ```fortran
          if(q2fact(1).gt.0.and.q2fact(2).gt.0)then
             sscale = sqrt(max(q2fact(1),q2fact(2)))
    ```
    The `validate_scales.rs` doc comment cites this as `unwgt.f:686`, a line number that does not match the pinned file.
