---
type: Design Decision
title: "Several @N processes in one event file: per-part normalisation and <init>"
description: "Each final-state multiplicity's weights are scaled to its own integrated σ (mean-weight convention, overweights kept); one <init> line per @N with XSECUP and XERRUP split by weight share; the header keeps the pre-normalisation estimate."
status: draft
tags: [events, lhef, normalisation, multi-process, mlm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n38-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L1032-L1143", title: "Note 38 E1, one <init> entry per @N"}
  - {id: n41-m5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1617-L1822", title: "Note 41 M5, samples against their integrations"}
  - {id: n41-fa, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1823-L2005", title: "Note 41 F-A, the weight tail and MadEvent's normalisation"}
  - {id: n41-p12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2299-L2557", title: "Note 41 P12, each part normalised to its integration"}
  - {id: n41-fb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2006-L2298", title: "Note 41 F-B, the sample against its integration after channel merging"}
  - {id: mg-unwgt-store, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/Template/LO/SubProcesses/unwgt.f#L270-L446", title: "MadGraph 3.7.1 unwgt.f store_events"}
---

# Several `@N` processes in one event file

A card may sum several processes (`generate … @1`, `add process … @2`) and,
at proton beams, several final-state multiplicities
([phase-space/mixed-multiplicity-integrand](../phase-space/mixed-multiplicity-integrand.md);
the card grammar is [process/proc-card-grammar](../process/proc-card-grammar.md)).
Two separate things then have to be decided: how the event weights are
normalised, and what the `<init>` block declares per process. A **part** is one
integrated final-state multiplicity of a sum, or the whole run when there is
one; a **process** is one `@N`.

## The rule (`Buffer`, `IDWTUP = −4`)

`Buffer::emit` (`vibegraph-lib/src/lhef/emit.rs`)[^n41-p12]:

1. **Each part is normalised to its own integration.** All events of part `k`
   are scaled by one factor so that `Σ_{i∈k} XWGTUP_i / N = σ_k`, where `N` is
   the file's total event count and `σ_k` the part's integrated cross section
   (`generate.rs`, `part_sigmas`, from the artifact's per-channel σ over the
   part's channel offsets). This is the mean-weight convention `IDWTUP = −4`
   promises, the one MadGraph's `event_norm = average` and Pythia's `main164`
   read. A single-multiplicity run passes no parts and the whole file is
   normalised to `artifact.sigma_pb`.
2. **Overweights keep their weight**, rescaled with their part; nothing is
   truncated. The only bias is the ratio estimator's, `O(1/n_k)`.
3. **`<init>` has one line per `@N`** (`LPRUP = N`). Process `p` declares

   ```text
   XSECUP_p = Σ_k σ_k f_pk
   XERRUP_p = √( Σ_k (f_pk Δσ_k)² + σ_k² f_pk (1 − f_pk) / n_k )
   ```

   with `f_pk` the process's share of part `k`'s generator weight and `n_k` the
   part's event count. Where an `@N` is exactly one part, this is that part's
   integrated σ and error. `XMAXUP_p` is the largest weight written for it.
4. **The header keeps the sample's own estimate**:
   `sample estimate before normalisation σ̂ +- err`, with `σ̂ = W·Σw/T` from the
   accept/reject pass and error `σ̂·√(Σw²)/Σw` (high by `1/√(1−ε)`, 1–2% at
   typical efficiencies), plus one line per part on a sum.
   `sample_estimate_in` reads it back.

The `XSECUP` split within a part is the binomial rule E1 introduced for two
`@N` lines of one multiplicity: on `dy_two_procs` (`p p > e+ e- / z @1`,
`add process p p > mu+ mu- / a @2`) the file's `@1` share was 0.02686 against
MadEvent's 0.02672 over 50k events, and both strategies' `<init>` split equals
their own event split to 1e-6[^n38-e1].

`StochasticRounding` writes unit weights, cannot carry a per-part scale, and is
refused on a sum over multiplicities; on several `@N` of one multiplicity it
draws the sample twice to know the split before writing `<init>`
([events/lhef-weight-strategy](lhef-weight-strategy.md)).

## Why: the file's σ must not be the sample's

Before this rule the buffered file declared the sample's own σ̂ with the
integration's error. On the mixed `pp_to_ll_0j2j_mlm` row that understated the
file's error about 7×: at 10000 events and ~3% efficiency the binomial error of
σ̂ alone is ~1%, and the five samples sat −0.45% to +2.20% from their
integrations — consistent with that error, not a uniform offset[^n41-m5][^n41-fa].
Measured over ten 10000-event samples, the declared σ's χ²/dof over files went
from 32.8 to 1.02 (`@0`/`@1`/`@2` 0.66/0.90/1.89), and through Pythia the merged
σ's file-to-file χ²/dof from 4.67 to 0.77[^n41-p12]. Per-part rather than
whole-file normalisation was chosen because the parts are where the binomial
scatter lives (MadEvent's `@N` shares scatter at 0.03–0.10 of their binomial
variance, ours at 3–7× before the change), and a part is large enough
(≥ 1200 events) that the ratio-estimator bias is negligible, unlike a
per-channel normalisation over a handful of events[^n41-fa].

## MadEvent's behaviour, the comparison point

MadEvent normalises per **channel**, more finely, and truncates[^n41-fa][^mg-unwgt-store]:

- `store_events` takes `target_wgt` from the `trunc_max` ladder, keeps
  overweights at their own weight (`max(|w|, target_wgt)`), and rescales the
  channel's events so they sum to its integrated |σ|
  (`xscale = xsecabs/xsum`, `unwgt.f:404` at 3.7.1);
- `combine_events` scales each channel's file again to its `axsec`
  (`lhe_parser.py:1154,1159`), picks one `max_wgt` for the requested event
  count at `trunc_error = 1e-2`, and writes every kept event at **one** weight
  under `event_norm = average`. Overweights are truncated to that weight and
  the loss is logged as `trunc_cross`.

So MadEvent's file σ is the integration's by construction, at the price of a
≤ ~1% truncation bias in shapes. vibegraph takes the same normalisation at the
part level and keeps the overweights unbiased; the shape noise those
overweights carry (largest `w/w_max` 119 on one mixed-row sample) is not removed
by any normalisation and traces to the VEGAS grid tail
([phase-space/vegas-grid-weight-tail](../phase-space/vegas-grid-weight-tail.md)).
Capping overweights at unit weight was measured offline (merged-σ χ²/dof 0.57)
and not adopted, being biased.

## What checks it

- `each_part_is_normalised_to_its_integration`,
  `processes_that_split_parts_declare_their_shares`,
  `an_undeclared_part_is_refused` (`lhef/emit.rs`, hermetic).
- `cli_generate_proton` asserts `mean XWGTUP = XSECUP` and `XSECUP ± XERRUP`
  equal to the integration's (both to 1e-6) — structural checks of the writer —
  and, as the part with teeth, the header's pre-normalisation σ̂ against the
  integration as a pull over both errors, below `SAMPLE_PULL_MAX = 3.5`. The
  mixed-multiplicity case asserts the same per `@N`. Since the file's own σ
  equals the integration by construction, only the header pull measures the
  accept/reject pass[^n41-p12].
- The sample-against-integration scatter on merged channels is the
  unweighting's truncation: on `llj_fixed` five seeds read −1.34 to +0.83%
  (pulls −1.89 to +1.13, `Σpull²/n` 1.17)[^n41-fb][^n41-p12].

The matched comparison through Pythia, which reads `XWGTUP` and so sees this
normalisation, is [validation/mlm-pythia-matched-comparison](../validation/mlm-pythia-matched-comparison.md);
the matched record's other fields are [events/mlm-matched-event-record](mlm-matched-event-record.md).

[^n38-e1]: Note 38 E1, `@N` → `LPRUP`, the binomial `XERRUP` split, `dy_two_procs`.
[^n41-m5]: Note 41 M5, samples against their integrations (−0.45% to +2.20%).
[^n41-fa]: Note 41 F-A: what drives the file-to-file scatter, MadEvent's normalisation, proposal 1.
[^n41-p12]: Note 41 P12, policy 1, its gate and measurements.
[^n41-fb]: Note 41 F-B, the sample against its integration after merging.
[^mg-unwgt-store]: MadGraph 3.7.1 `unwgt.f` `store_events` (truncation ladder `:358-367`, rescale to the channel integral `:404`, overweights kept at their weight `:430-436`).
