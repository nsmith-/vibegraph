---
type: Algorithm
title: "The μF ≥ 2 GeV factorisation-scale veto"
description: "Per beam carrying a PDF with a dynamical μF, a point with q2fact < 4 GeV² is zero-weighted as in reweight.f; PointScales routes it; setup refuses only if every probe draw is vetoed."
status: draft
tags: [scales, pdf, veto, setclscales, hadronic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n29-c24, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4025-L4121", title: "Note 29 Chain C2 §C2.4 (the μF ≥ 2 GeV veto: reference semantics, reachable runs)"}
  - {id: n29-a0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4346-L4392", title: "Note 29 C2 amendment A.0 (the veto existed; it panicked)"}
  - {id: n29-a1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4393-L4472", title: "Note 29 C2 amendment A.1 (routing through PointScales)"}
  - {id: n29-a2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4473-L4540", title: "Note 29 C2 amendment A.2 (tests; refuse at setup)"}
  - {id: n29-a5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4645-L4667", title: "Note 29 C2 amendment A.5 (eight reachable runs)"}
  - {id: n29-a6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4668-L4699", title: "Note 29 C2 amendment A.6 (blind spots)"}
  - {id: mg-veto, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L1205-L1220", title: "MadGraph 3.7.1 reweight.f (the 2 GeV check)"}
  - {id: mg-zero, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L1907-L1908", title: "MadGraph 3.7.1 reweight.f (a failed setclscales zeroes the weight)"}
---
# The μF ≥ 2 GeV factorisation-scale veto

MadGraph's `setclscales` fails, and the point's weight is set to zero, when a
beam that carries a parton density and takes a dynamical factorisation scale
ends below `μF = 2 GeV`. It is a veto on the point, not a scale and not an
error. vibegraph reproduces it inside the clustering and routes it as a zero
weight. The scale it tests comes from [setclscales](setclscales.md); the
choices that make `μF` dynamical are in
[madgraph-scale-choice](madgraph-scale-choice.md).

## Reference semantics

[`reweight.f:1205-1220`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L1205-L1220):

```fortran
c     Check that factorization scale is >= 2 GeV
      if(lpp(1).ne.0.and.(q2fact(1).lt.4d0.and..not.fixed_fac_scale1).or.
     $   lpp(2).ne.0.and.(q2fact(2).lt.4d0.and..not.fixed_fac_scale2))then
         ...   ! warn on the first ten
         setclscales=.false.
         clustered = .false.
         return
      endif
```

and at the call site
([`:1907-1908`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L1907-L1908))
`if(.not.setclscales(...)) all_wgt(i) = 0d0`. Three properties no summary
carries:

- it is **per beam**;
- it applies only to a beam that both carries a PDF (`lpp ≠ 0`) **and** has a
  dynamical factorisation scale;
- the comparison is **strict on the square**: `q2fact < 4`, so exactly 2 GeV
  survives.

The point keeps its place in the sample and contributes nothing. Older readings
cite `reweight.f:1185` (3.5.7 numbering); the code is the same.

## Implementation

The check is in the clustering itself, `coupling/cluster/setclscales.rs`, after
the scale synthesis:

```rust
for beam in 0..2 {
    if settings.beam_has_pdf[beam] && q2fact[beam] < MUF_FLOOR && !settings.fixed_fac[beam]
    {
        return Err(ScaleRefusal::FactorisationFloor);
    }
}
```

with `MUF_FLOOR = 4.0`, comparing the square as MadGraph stores it.
`ScaleChoice::cluster_scales` returns it unchanged as
`ScaleError::Clustering(FactorisationFloor)`, so the scale replay gate sees the
refusal *as* a refusal when it replays banked events.

Above that, `EventScaleSource::point_scales` (`hadronic.rs`) is the one place
that turns it into a weight:

```rust
pub enum PointScales {
    Scales(EventScales),
    Vetoed,
}
```

`FactorisationFloor` and the `xqcut` clustering cut `JetCut` map to
`PointScales::Vetoed`; every other `ScaleError` stays an error, because it means
the prescription does not apply to this process, which no amount of sampling
fixes. Giving the distinction a type makes every call site say what it does
with a veto; a new call site cannot silently inherit a panic.[^n29-a1]

| call site | on `Vetoed` |
|---|---|
| `ProtonIntegrand`'s per-point scales | the term's weight is `0.0`, before the coupling is moved and before the PDFs are queried below their grid |
| `ProtonIntegrand::event_in_channel` | inherits it through the same evaluation, so a sample and its integral veto the same points |
| both `probe_scale`s | keep drawing past the vetoed point |
| `FixedBeamIntegrand` | unreachable: a fixed-energy card gives `beam_has_pdf = [false, false]`, so the clustering never constructs the floor refusal, and a fixed-energy card with `ickkw ≠ 0` or `xqcut > 0` is refused at compile (`ScaleError::FixedBeamMatching`), so `JetCut` cannot arise either; the site carries an explicit `unreachable!` |

The fixed-beam argument is upstream of the routing: it depends on the card, not
on which integrand runs.

## Setup diagnostics

MadGraph warns on the first ten vetoed points and goes quiet. vibegraph does not
count vetoes per event: a partially vetoed run is legitimate physics with a
correct σ, and a per-event counter would be mutable state inside the VEGAS loop.
Only the degenerate case acts. The setup probe draws up to `SCALE_PROBE_DRAWS =
64` points; if at least one passes the cuts and **every** such draw is vetoed, the
run is refused with `HadronicError::FactorisationScaleBelowFloor`:

> every sampled point's factorisation scale fell below the 2 GeV floor a parton
> density is fitted down to, so the cross section this card asks for is zero by
> construction

If no draw passes the cuts, the probe says nothing about the scale. A vetoed
point is never written, so it carries no record scale
([record-scales](record-scales.md)). A run whose
64 probe draws find support while the bulk of the measure is vetoed integrates
normally: the σ is right, merely inefficient.[^n29-a2]

## Tests

| test | fails on | cannot detect |
|---|---|---|
| `a_sub_threshold_factorisation_scale_gives_zero_weight` (`proton.rs`): a card whose `scalefact` drives `μF` below 2 GeV everywhere returns exactly `0.0`; the same fixture at `scalefact = 1` returns nonzero | the veto turned back into an error | a veto firing too often on a normal card |
| `a_vetoed_point_is_dropped_from_generation_too` (`proton.rs`) | a veto wired into the integrand but bypassed on the generation path | whether kept events carry the right scales |
| `the_factorisation_floor_is_unreachable_on_fixed_beams` (`hadronic.rs`) | a `beam_has_pdf` guard dropped or inverted | anything about proton runs |
| `banked_hadronic_runs_clear_the_factorisation_floor` (`tests/validate_scales.rs`) | a banked run reaching the floor, or a reachable run missing from `FLOOR_REACHABLE_RUNS` | runs that are not banked |

The two halves of the first test must stay in one test: alone, "vetoed" cannot
be told from "cut away" or "PDF returned zero".

## Why no banked reference exercises it

A banked sample cannot contain a counter-example: MadGraph vetoed such points
before writing them. Eight banked runs can reach the floor (`lpp = (1,1)` with
at least one dynamical `μF`): `pp_to_bb`, `pp_to_bb_qcd2`, `pp_to_jj`,
`pp_to_ll`, `pp_to_ll_qcd0`, `pp_to_ll_scalefact2`, `pp_to_llj`,
`pp_to_llj_dyn` (`FLOOR_REACHABLE_RUNS`). The matched and pure-cut MLM rows are
left out of that test, since their floor check runs inside two `setclscales`
calls; `validate_mlm_dumps` checks that no written event of theirs is rejected.
The `dy13` cards fix both scales, so
`setclscales` returns before the check. The minimum banked `μF`
(`SCALUP = sqrt(max(q2fact(1), q2fact(2)))`, `unwgt.f:752`, cross-checked per
beam against `<pdfrwt>` where written):[^n29-c24][^n29-a5]

| run | min μF (GeV) | headroom |
|---|---|---|
| `pp_to_bb`, `pp_to_bb_qcd2` | 4.7003 | ×2.35 |
| `pp_to_jj` | 20.0003 | ×10.0 |
| `pp_to_ll_qcd0` | 20.0393 | ×10.0 |
| `pp_to_llj` | 21.5932 | ×10.8 |

The minima for the other three reproduce the same picture, and the global minimum
over every banked hadronic event is **4.70 GeV**. The floors are structural: the
light-jet runs sit on their cards' `ptj = 20`, and the `b b̄` runs on `m_b = 4.7`,
since the clustered core's transverse mass cannot fall below its heaviest leg.

## Blind spots

- **The threshold is a transcription.** `MUF_FLOOR = 4.0` is pinned by reading
  `reweight.f`; no banked run approaches it (nearest ×2.35), so every test above
  would pass with a wrong constant.
- **One route into the floor.** The tests reach sub-threshold support through
  `scalefact`. A floor reached through another branch of the `μF` synthesis
  (the backfill or beam-1-from-first branches) is covered only by the shared
  comparison in `setclscales.rs`.[^n29-a6]
- **The banked minima are this crate's replay.**
  `banked_hadronic_runs_clear_the_factorisation_floor` replays `μF` with our
  clustering rather than reading MadGraph's own `q2fact`, so a common-mode
  scale error moving both sides together would not show.

[^n29-a1]: Note 29 C2 amendment A.0–A.1. The veto predated the design; the defect was that it panicked mid-integration.
[^n29-a2]: Note 29 C2 amendment A.2.
[^n29-c24]: Note 29 §C2.4.
[^n29-a5]: Note 29 C2 amendment A.5 (eight runs, not five).
[^n29-a6]: Note 29 C2 amendment A.6.
