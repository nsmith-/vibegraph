---
type: Physics Convention
title: What SCALUP and AQCDUP hold, and the scale every record carries
description: "SCALUP is the larger per-beam μF, not μR; AQCDUP is αs(μR), where MadGraph's carries a truncated π; no-aS models record 0; fixed-beam records carry the card default's clustered scale."
status: draft
tags: [lhef, scales, scalup, aqcdup, alpha-s]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n07-aqcdup, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/07-mg5-code-quality.md#L367-L403", title: "Note 07 (unwgt.f: truncated π in AQCDUP)"}
  - {id: n35-v2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L1113-L1190", title: "Note 35 V2 (banked-layer hygiene, the p3r3 AQCDUP reading)"}
  - {id: n35-z1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L1359-L1397", title: "Note 35 §10.2 (colour-toy runs promoted in validate_scales)"}
  - {id: n36-b8, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L709-L749", title: "Note 36 §7.1 (B8, the fixed-beam scale record, user decision)"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L547-L624", title: "Note 41 M1 (the scale split)"}
  - {id: build-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/lhef/build.rs#L21-L41", title: "lhef/build.rs scalup()"}
  - {id: hadronic-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/hadronic.rs#L2773-L2794", title: "FixedBeamIntegrand::record_scales"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/unwgt.f#L694-L695", title: "MadGraph unwgt.f (aaqcd = g*g/4d0/3.1415926d0)"}
  - {id: mg-export, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/export_v4.py#L7076-L7079", title: "MadGraph export_v4.py (aS injected for a model without one)"}
measured:
  - {commit: 2803e17, command: "pixi run --skip-deps validate"}
---

# What `SCALUP` and `AQCDUP` hold

## `SCALUP` is the factorisation scale

`SCALUP` is the larger of the two per-beam factorisation scales, as `q2fact` holds them
when the event is written. MadGraph's `unwgt.f:750-756` writes
`sqrt(max(q2fact(1), q2fact(2)))`, and `lhef::build::scalup` (`vibegraph-lib/src/lhef/build.rs`)
is[^build-rs]

```rust
pub fn scalup(scales: &EventScales) -> f64 {
    scales.mu_f_record[0].max(scales.mu_f_record[1])
}
```

It is **not** the renormalisation scale. Reading it as `μR` is a misreading hazard, not a
MadGraph defect: the Les Houches accord defines `SCALUP` as the scale the densities were
evaluated at. The two coincide whenever the prescription reads both off the same vertex,
which is every closed-form clustering, and that coincidence is why the field is so often
read as `μR`. Where the clustering splits them the reading is wrong. Unit test:
`scalup_is_the_factorisation_scale_not_the_renormalisation_one`.

Measured cases where they part:

- **The two `2 → 6` rows** (`bbx_to_ccx_emmm_qcd0`, `uux_to_ccx_emmm_qcd0`): evaluating
  `αs` at the printed `SCALUP` misses `AQCDUP` by up to 9%, five orders outside the
  printing budget (`scalup_is_not_the_renormalisation_scale`, `validate_scales`).
- **The diquark rows** (`p3r3_to_p3r3_toy_epsilon`, `_sextet`): two distinct massive colour
  triplets, so `partonline` ends at the first initial-state merge and `jcentral` differs
  per side. Each beam keeps its own `μF` (248.696354 and 251.296392 GeV), and `μR` is the
  four-factor geometric mean, 249.992993 GeV. `SCALUP = max(μF)` reproduces at 0.158 of
  budget and sits 2.6e4 budgets from `μR`; inverting `AQCDUP` returns `μR` to 8.9e-8
  relative. `scalup_is_the_factorisation_scale_on_the_diquark_rows` asserts all three
  halves, so a run that stopped parting the scales fails.[^n35-z1]

`μR` reaches the record through `AQCDUP`, which is the finer oracle for it: seven printed
digits of `αs` locate the scale to about 1e-6 relative. How the scales themselves are
computed is [scales-pdf/setclscales](setclscales.md).

**Under MLM matching** the record's `μF` is not the scale the densities were read at.
`EventScales` carries `mu_f` (the densities') and `mu_f_record` (what `SCALUP` reads).
Without matching they are one value (`EventScales::unmatched`, `coupling/scales.rs:79-86`).
Under `ickkw = 1`, `mu_f_record` is the first call's `q2bck` when `pdfwgt` is set and the
second call's `q2fact` otherwise, because `rewgt` restores `q2bck` only under `pdfwgt`
(`reweight.f:1789-1791`).[^n41-m1] The full account is [scales-pdf/mlm-scales](mlm-scales.md).

## `AQCDUP` is `αs(μR)`; MadGraph's carries a truncated π

`unwgt.f:694-695` fills the coupling fields as[^mg-unwgt]

```fortran
      aaqcd = g*g/4d0/3.1415926d0
      aaqed = gal(1)*gal(1)/4d0/3.1415926d0
```

while `g = √(4π·αs)` was built from full double-precision π. So MadGraph's `AQCDUP` is
`αs·π/3.1415926 = αs·(1 + 1.7e-8)`, and `AQEDUP` carries the same truncation. The bias is
baked in before printing, systematic and one-directional. At the field's precision it is
about a sixth of the last printed digit, enough to move the rounding of roughly one event
in twenty.[^n07-aqcdup]

`rw_events.f:182` writes the event line as `(i2,i5,e16.7e3,3e15.7)`, so `SCALUP`, `AQEDUP`
and `AQCDUP` carry seven significant digits (digits 9–10 are `0` on every banked event).

- vibegraph writes the **untruncated** `αs(μR)` (`EventHeader::alpha_qcd`); the defect is
  not a convention of the field. `aqcdup_does_not_reproduce_the_truncated_pi` pins the
  choice by asserting the size of the difference.
- The replay gates model the truncation (`TRUNCATED_PI` in `validate_scales.rs`) and
  compare MadGraph's printed digits exactly. Modelling it took exact digit reproduction
  from about 95% to 100% of events; it is invisible at any tolerance looser than ~1e-8,
  which is why gating on printed digits rather than a chosen tolerance exposed it.

The defect is registered in [validation/madgraph-defects](../validation/madgraph-defects.md);
the replay gate is [validation/scale-replay-gate](../validation/scale-replay-gate.md).

## Models that declare no `aS`

A UFO with no `aS` among its external parameters leaves `SMINPUTS` out of the generated
parameter card. MadGraph's `export_v4.py:7076-7079` then injects `aS = 0.138` beside
`G = 4.1643` with a `CRITICAL` log line, and `setrun.f` runs the coupling from `G`'s
`1.3799843265950287`; the two halves disagree by a factor of ten.[^mg-export][^n36-b8]
That value is declared once (`common::UNDECLARED_ALPHA_S_MZ`, for the six
`UNDECLARED_ALPHA_S_RUNS`: `ll_to_qqx_toy_dipole`, `_tensor`, `_yukawa`,
`p3r3_to_p3r3_toy_epsilon`, `_sextet`, `qqx_to_o8o8_toy_dcolor`), and
`validate_alphas::banked_run_logs_pin_the_alpha_s_source_rule` checks it against the
printed log line.[^n35-v2] The injection itself is a MadGraph defect
([validation/madgraph-defects](../validation/madgraph-defects.md)).

vibegraph does not write a strong coupling into a model with no strong interaction: with
no `αs` source the record's `AQCDUP` is `0`. On those six rows `AQCDUP` is measured and
reported with the reason, not gated, and the cause is checked
(`alpha_s_source().is_some()` must equal the row not being in `UNDECLARED_ALPHA_S_RUNS`).
Their `SCALUP` gates.[^n36-b8]

## Fixed-beam records follow the card's default (decision)

**Decision (user, 2026-09-07): the record is faithful to the card.** Under MadGraph's
shipped defaults (`dynamical_scale_choice = -1`, both `fixed_*_scale = False`) MadEvent runs
`setclscales` for every event, even when the matrix element carries no strong coupling, and
writes the clustered scale and a running coupling. A fixed-beam vibegraph record does the
same: it carries what the banked card produces. A per-row `ignore` list for the field was
proposed and rejected as unfaithful to the reference. Anyone who wants no clustering on
such a run re-cards it.[^n36-b8]

How it is built (`FixedBeamIntegrand`, `hadronic.rs`):

- `use_running_coupling` compiles the prescription and the coupling from the card alone.
- `alpha_s_dependent` decides only whether the **integrand** reads them. With no `αs` in the
  matrix element there is no clustering or configuration draw on the integration path, so
  σ is untouched (bit-identical on all 45 `integrals` cells, measured at `2803e17`).
- `record_scales` still draws and clusters, for the record. A point the prescription
  rejects has no scale to report, and the event source stops rather than inventing one. A
  caller that installed no prescription at all falls back to the card's
  `max(dsqrt_q2fact1, dsqrt_q2fact2)` and `AQCDUP = 0`.[^hadronic-rs]

`SCALUP` and `AQCDUP` are enforced at printed precision on the fixed-beam rows, for
example `ee_to_mumu` at 91.2 / 0.1179977, `ee_to_ee` at 250, `ee_to_ttx` at 500, `p3r3`
at 251.2964 (7.9e-6 inside a 5e-5 budget), with the six no-`aS` rows' `AQCDUP` as the one
exception above.

## On proton beams

A proton event carries the scales of the term that was drawn: under the per-term scale
path each flavour group and each beam ordering has its own `μR` and `μF`, and `generate`
writes the drawn term's ([hadronic/proton-integrand](../hadronic/proton-integrand.md)).
The `AQCDUP` there is evaluated by the run's `αs` source, the PDF set's own tabulation for
`pdlabel = lhapdf` ([scales-pdf/alpha-s-sources](alpha-s-sources.md)). Field layouts are
[events/lhef-record-conventions](../events/lhef-record-conventions.md).

[^n07-aqcdup]: Note 07, the truncated π in `unwgt.f`'s `AQCDUP`, and the event-line format.
[^n35-v2]: Note 35 V2, the inventories and the `p3r3` `AQCDUP` finding.
[^n35-z1]: Note 35 §10.2, the `p3r3` rows replayed: `SCALUP` is `μF`, `μR` recovered from `AQCDUP`.
[^n36-b8]: Note 36 §7.1, B8: the fixed-beam scale record (user decision, 2026-09-07).
[^n41-m1]: Note 41 M1, the density and record factorisation scales.
[^build-rs]: `lhef::build::scalup` and its documentation.
[^hadronic-rs]: `FixedBeamIntegrand::record_scales`, `vibegraph-lib/src/hadronic.rs`.
[^mg-unwgt]: MadGraph `unwgt.f:694-695`.
[^mg-export]: MadGraph `export_v4.py:7076-7079`, the `aS` injection.
