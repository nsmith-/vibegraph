---
type: Physics Convention
title: "MadGraph's renormalisation and factorisation scale choices"
description: "Fixed scales, dynamical_scale_choice 1–5 closed forms and -1 clustering, per-beam μF, what -1 reduces to on 2 → 2 runs, and which choices vibegraph honours on which beams."
status: draft
tags: [scales, madgraph, run-card, kt-clustering, convention]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n22-12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L60-L89", title: "Note 22 §1.2 (the scale itself; per-beam μF; scalefact placement)"}
  - {id: n22-13, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L90-L126", title: "Note 22 §1.3 (what -1 collapses to on the banked events)"}
  - {id: n22-14, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L127-L156", title: "Note 22 §1.4 (the per-event oracle: SCALUP, AQCDUP, mgrwt)"}
  - {id: n22-4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L217-L255", title: "Note 22 §4 (risks: the fixed DY branches, loud refusals)"}
  - {id: n22-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L283-L338", title: "Note 22 close-out (scale compilation; per-beam μF wired)"}
  - {id: n28-k18, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L963-L1096", title: "Note 28 §K1.8 (reconciliation of the collapse table with the general path)"}
  - {id: n28-k42, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2452-L2478", title: "Note 28 §K4.2 (closed forms replaced; scalefact once)"}
  - {id: n29-a3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4541-L4606", title: "Note 29 C2 amendment A.3 (choices 1–5 refused only where read)"}
  - {id: n36-b6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L458-L524", title: "Note 36 §4 B6 (choices 1–5 honoured at fixed beams; gg_to_gg_cg)"}
  - {id: mg-setscales, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setscales.f#L44-L100", title: "MadGraph 3.7.1 setscales.f (set_ren_scale)"}
  - {id: mg-cuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cuts.f#L1233-L1241", title: "MadGraph 3.7.1 cuts.f (where the scales are set)"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/unwgt.f#L752", title: "MadGraph 3.7.1 unwgt.f (SCALUP)"}
---
# MadGraph's renormalisation and factorisation scale choices

A MadGraph LO run has one renormalisation scale `μR` (`scale`) and one
factorisation scale **per beam**, `q2fact(1)` and `q2fact(2)` (stored squared).
The run card picks how each is set; vibegraph compiles the same card into a
`ScaleChoice` (`coupling/scales.rs`) whose output `EventScales` carries `μR`
and a two-element `μF`. `μR` sets `αs` ([alpha-s-sources](alpha-s-sources.md));
each `μF` sets one beam's PDFs. What the event record writes is
[record-scales](record-scales.md).

## Fixed scales

`fixed_ren_scale` with `scale`, and `fixed_fac_scale1`/`fixed_fac_scale2` with
`dsqrt_q2fact1`/`2`, fix each scale independently. The older single
`fixed_fac_scale` fills in for whichever per-beam flag a card leaves alone
(`banner.py`'s `post_set_fixed_fac_scale`); a card setting all three
inconsistently is the one combination where vibegraph's disjunction and
MadGraph's tracking differ, and MadGraph warns about it. `cuts.f` calls
`set_ren_scale` only when `μR` is not fixed and `set_fac_scale` only when a beam
is not fixed
([`cuts.f:1233-1241`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cuts.f#L1233-L1241)),
so a fixed scale never sees `scalefact`. A fully fixed card resolves once at
setup and reads no kinematics.

The fixed branches are load-bearing: the hadronic Drell–Yan σ reference cards
(`validation/madgraph/dy13_default_run_card.dat`, `dy13_mmll_run_card.dat`)
fix both scales at `91.188`, so those σ numbers must not move under any scale
change. `scales_run_cards.rs` asserts both cards still compile to those
constants. The fixed and dynamic branches are pinned by disjoint evidence.[^n22-4]

## Dynamical choices

`dynamical_scale_choice` applies to every scale not fixed. `set_ren_scale` in
[`setscales.f:44-100`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setscales.f#L44-L100),
sums over final-state legs `i = 3 … nexternal`:

| choice | `μR` before `scalefact` |
|---|---|
| `-1` (default) | `0` here; set by kT clustering in `setclscales` |
| `0` | user-edited Fortran (`user_dynamical_scale`) |
| `1` | `Σ E_T` |
| `2` | `Σ m_T`, `m_T = √((E+p_z)(E−p_z))` |
| `3` | `Σ m_T / 2` |
| `4` | `√ŝ` |
| `5` | `√(p₁²)`, the decaying particle's mass |

then `rscale = scalefact·rscale`; any other value stops. For 1–5,
`set_fac_scale` squares that already-scaled value into each non-fixed
`q2fact`, so `μF` carries exactly one factor too. vibegraph refuses `0` and
unknown values (`ScaleError::UnsupportedChoice`).

**`-1` is a clustering.** The event is clustered with MadGraph's kT algorithm
down to a core ([kt-clustering-algorithm](kt-clustering-algorithm.md)), and
`setclscales` reads `μR` and each beam's `μF` off the vertices a colour line
passes through ([setclscales](setclscales.md)). Under `-1` it is `reweight.f`
that applies `scalefact`, one power to `μR` and one to each `μF` on the pinned
3.7.1 in every branch (MadGraph 3.5.7 applied it twice to beam 2 in the
colourless-beam branch). `pp_to_ll_scalefact2` (`SCALEFACT_RUNS` in
`tests/validate_scales.rs`) is the banked oracle for that placement, beside
`scalefact_reaches_every_scale_exactly_once`.[^n28-k42] The clustered scale
depends on the integration channel
([cluster-scale-channel-dependence](cluster-scale-channel-dependence.md)), and a
beam whose dynamical `μF` falls below 2 GeV zero-weights the point
([factorisation-scale-floor](factorisation-scale-floor.md)).

## What `-1` reduces to on a 2 → 2

For a 2 → 2 the outgoing transverse momenta are equal and opposite, so
equal-mass legs have `djb₃ = djb₄` exactly, and `nexternal = 4` means the
clustering's boost never fires. The general path then collapses to a few closed
forms, measured on the banked events and derived from the code (`djb` is
`m² + p_T²` with a PDF on either beam and `E²` with none):[^n22-13][^n28-k18]

| runs | beams | `-1` gives | why |
|---|---|---|---|
| `gg_to_gg`, `gg_to_ttx`, `uux_to_uux` | `lpp = 0` | `√ŝ/2 = 250` | a beam–leg merge wins by a factor 4 over the final-state pair; every vertex is `ŝ/4` |
| `uux_to_uux`, the crossed events | `lpp = 0` | `250.000125` | both admissible beam–leg candidates crossed, so the `1 + 1e-6` inflation survives; `SCALUP` prints `250.0001` |
| `ee_to_ee`, `ee_to_wpwm` | `lpp = 0` | `250` | colourless beams: no colour line, `μR` falls through to `√pt2ijcl(nexternal−2)`; the inflation never reaches the scale |
| `ee_to_mumu`, `ee_to_ttx`, `ee_to_zh`, `ee_to_tatah`, `uux_to_mumu` | `lpp = 0` | `√ŝ = 500` | s-channel only: one admissible merge, `djb(p₃+p₄) = ŝ` |
| `pp_to_bb`, `pp_to_bb_qcd2` | `lpp = 1` | `√(m_T(b)·m_T(b̄))` | `g g`: beam–leg merge; `q q̄`: the `mt2last` override; equal by `djb₃ = djb₄` |
| `pp_to_ll`, `pp_to_ll_qcd0` | `lpp = 1` | `m(ℓℓ)`, same `μF` on both beams | s-channel only; `p_T(pair) = 0` |

Two corrections a reader is likely to need:

- **"`lpp = 0` ⇒ constant scale" is false.** `bbx_to_ccx_emmm_qcd0` has
  `lpp = 0` and fixed `√ŝ = 500` yet 8720 distinct `SCALUP` values in 10 000
  events (the boost moves the frame `djb = E²` is taken in); `ee_to_ttx` sits at
  500, not 250, because the central line tracks the *beam* colour
  (`qcdline(j) = isqcd(beam j)`, `reweight.f:741`), not the final state.
- **The geometric-mean form is unpinned.** On every banked coloured 2 → 2 the
  legs have equal mass, so `(djb₃·djb₄)^¼` and either leg's `√djb` are the same
  number; the code evaluates `(pt2ijcl(2)⁴)^{0.125}`.

The table is MadGraph's behaviour and a consistency check on the general path.
vibegraph has no closed-form implementation of `-1`; every `-1` scale goes
through the kT engine ([kt-clustering-engine](kt-clustering-engine.md)). With
more than two final-state legs (`pp_to_llj*`, `ee_to_mumua`,
`ee_to_mumu_tata_qcd0`, the two `2 → 6` runs) there is no closed form.

## Which choices vibegraph honours

| choice | fixed beams (`FixedBeamIntegrand`) | hadron beams (`ProtonIntegrand`) |
|---|---|---|
| fully fixed | honoured | honoured |
| `-1` | honoured (kT engine) | honoured (kT engine) |
| `1`–`5` | honoured (`ClosedForms::Honour`, `hadronic.rs`) | refused: `ScaleError::UnhonouredScaleChoice` (`ClosedForms::Refuse`, `proton.rs`) |
| `-1` with exactly one `μF` fixed | refused (`MixedFixedFactorisationScales`) | refused |

The closed forms are refused at hadron beams because no banked hadronic run
selects one: a scale taken under it would feed the parton densities with nothing
on the other side, giving a plausible, smooth, wrong σ. Fixed beams honour them
because `gg_to_gg_cg`'s banked run selects choice 3 and replays 10 000 events
(20 000 scale comparisons) at worst 0.999 of the printing budget in
`validate_scales`, with its `samples` cell gated; its σ cell is informational at
a converged `−0.22 %` offset that neither the scale formula nor the process
explains ([gg-to-gg-cg-sigma-offset](../backlog/validation/gg-to-gg-cg-sigma-offset.md)).[^n36-b6]
**The hadron-beam refusal should be revisited when a banked hadronic run selects
a closed form.** The refusal is on the *value*, applied in
`ScaleChoice::from_run_card_for` only where the choice is read: a fully fixed
card accepts any choice, because both scale entry points short-circuit before
reading it. The field itself stays `Consumed`
([field-classification](../run-card/field-classification.md));
`an_unhonoured_scale_choice_is_refused` (`coupling/scales.rs`) pins the gate and
its fully-fixed exemption. A decay's scales are fixed before a prescription is
compiled (`RunCard::for_decay`).[^n29-a3]

## The per-event oracle

Every banked run's `unweighted_events.lhe.gz` carries `SCALUP` and `AQCDUP` on
each `<event>` line, a per-event oracle finer than σ. `SCALUP` is the
**factorisation** scale, `sqrt(max(q2fact(1), q2fact(2)))`
([`unwgt.f:752`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/unwgt.f#L752));
it doubles as `μR` only where the clustering reads both off the same vertex.
`AQCDUP` recovers `μR` to about `1e-6` relative independently of any scale field.
Runs with `use_syst` also write `<mgrwt>`: `<rscale>` is `μR`, `<pdfrwt>` each
beam's `μF`, both `E15.8`. `AQEDUP` is constant (`1/132.507`): α_EW does not run
in MadGraph's LO path. The gate built on these is
[scale-replay-gate](../validation/scale-replay-gate.md).[^n22-14]

[^n22-4]: Note 22 §4, "DY must not move".
[^n22-13]: Note 22 §1.3, measured over the banked runs.
[^n28-k18]: Note 28 §K1.8, each row derived from the 3.7.1 code path.
[^n28-k42]: Note 28 §K4.2; the 3.5.7 double application is note 22 §1.2's reading of the packaged template.
[^n36-b6]: Note 36 §4 B6, landed items (1) and (4).
[^n29-a3]: Note 29 C2 amendment A.3, as amended by B6.
[^n22-14]: Note 22 §1.4.
