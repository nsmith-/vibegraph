---
type: Design
title: MLM run-card parameters and the xqcut cut rewrites
description: "ickkw, xqcut, maxjetflavor, pdfwgt, alpsfact and use_syst rules; the ptj = mmjj = xqcut and drjj = drjl = 0 rewrites applied when the card is resolved; and the matching combinations that are refused."
status: draft
tags: [run-card, mlm, matching, xqcut, cuts]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n41-record, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L146-L190", title: "Note 41 §1.4, setup cuts and run-card rules"}
  - {id: n41-card, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L284-L298", title: "Note 41 §3.5, the run card"}
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L303-L530", title: "Note 41 M0, the reference cards"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 M1, xqcut and the ickkw = 1 scales"}
  - {id: mg-setcuts-xqcut, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setcuts.f#L156-L189", title: "MadGraph setcuts.f, the xqcut rewrites"}
  - {id: mg-banner-mlm, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4543-L4577", title: "MadGraph banner.py RunCardLO.check_validity, matching rules"}
---

# MLM run-card parameters and the `xqcut` cut rewrites

MadGraph edits a matched card in three places before anything reads it:
`banner.py`'s `RunCardLO.check_validity` refuses some combinations and rewrites
others (`banner.py:4543-4577`), `setrun.f` forces `alpsfact` under systematics
(`setrun.f:151-159`), and `setcuts.f` rewrites the jet cuts once `xqcut` is set
(`setcuts.f:156-189`). vibegraph applies all of them once, in
`runcard::matching::resolve` (`vibegraph-lib/src/runcard/matching.rs`), called
from `RunCard::from_values`; every consumer — the cut filter, the scale
prescription, the artifact that banks the card — reads the resolved card. Each
rewrite is idempotent, so a card resolved twice (a decay's derived card) reads
the same values[^n41-m1]. What matching then does with these values is
[scales-pdf/mlm-matching](../scales-pdf/mlm-matching.md).

## The rules

| parameter / combination | vibegraph | MadGraph source |
|---|---|---|
| `ickkw ∉ {0, 1}` | refused (`UnsupportedIckkw`) | `banner.py:4284`, `allowed = [0, 1]` |
| `ickkw = 1` with `maxjetflavor = 6` | refused (`MatchedTopJets`) | `banner.py:4556` |
| `use_syst = T` with `alpsfact ≠ 1` | `alpsfact` set to 1, with a warning, **whether or not matching is on** | `setrun.f:151-159` (`banner.py` does it only under `ickkw > 0`, which the Fortran supersedes) |
| `xqcut > 0` with `ickkw = 0` | accepted as a pure cut, with a warning | `banner.py`: an error log and a 5 s sleep, then it runs |
| `ickkw = 1` or `xqcut > 0` at fixed beams or on a decay | refused (`ScaleError::FixedBeamMatching`) | none: no fixed-beam reference, and the fixed-beam integrand cannot zero-weight a point the clustering rejects ([feature/mlm-at-fixed-beams-or-decays-refused](../backlog/feature/mlm-at-fixed-beams-or-decays-refused.md)) |
| `ickkw = 1` with exactly one μF fixed | refused (`MatchingWithOneFixedFactorisationScale`) | `reweight.f:1138` reads `.not.fixed_fac_scale1.or.fixed_fac_scale2` without the parentheses the surrounding branches imply, applying `scalefact` and `q2bck` to one beam only; refused rather than reproduced |
| `pdfwgt` | consumed only at `ickkw > 0` | `setrun.f:82` clears it at `ickkw = 0` |
| `ktscheme ≠ 1`, `chcluster` | refused ([run-card/field-classification](field-classification.md)) | |
| `ktdurham`, `ptlund`, `dparameter` (CKKW-L) | refused as unimplemented cuts | |
| `highestmult`, `clusinfo`, `pdgs_for_merging_cut` | benign | `hmult` is read only in `hmult .or. ickkw == 1`; `clusinfo` adds the `<clustering>` block; the last qualifies only refused cuts |

`ickkw`, `xqcut`, `alpsfact`, `asrwgtflavor`, `auto_ptj_mjj`, `use_syst`,
`pdfwgt` and `maxjetflavor` are `Consumed`[^n41-card][^n41-m1].
`asrwgtflavor` reads as 5 on MadGraph's matched cards (the hidden default;
`isparton` reads `max(asrwgtflavor, maxjetflavor)`)[^n41-m0].

## The `xqcut` cut rewrites

When `xqcut > 0` (`setcuts.f:156-189`)[^mg-setcuts-xqcut]:

- if `auto_ptj_mjj` (default `T`) and `ptj ≥ 0` and `ktscheme = 1`:
  **`ptj = xqcut`**; otherwise a `ptj > xqcut` is set to 0;
- if `auto_ptj_mjj` and `mmjj ≥ 0`: **`mmjj = xqcut`**; otherwise an
  `mmjj > xqcut` is set to 0;
- **`drjj = drjl = 0`**, whatever their sign (`banner.py:4562-4571` zeroes any
  nonzero value before `setcuts.f`'s `> 0` test sees it).

Legs with `do_cuts = .false.` are exempt as for every cut: decay products under
`cut_decays = F`, masses above 20 GeV, neutrinos (`setcuts.f:201-215`)
([run-card/madgraph-cut-conventions](madgraph-cut-conventions.md)). These
rewrites change σ, and they are the resolved values the artifact records.
Tests: `xqcut_sets_the_jet_thresholds_under_auto_ptj_mjj`,
`without_auto_ptj_mjj_thresholds_above_xqcut_are_zeroed`,
`no_xqcut_leaves_the_jet_cuts_alone` (`runcard/matching.rs`).

## The τ floor, and why `ptj < xqcut` is refused

MadEvent's `setxqcuts` (`myamp.f:337-560`, called at `setcuts.f:892-955`) builds
phase-space hints under `xqcut`: jet energy floors `max(ptj, √(xqcut² − m²))`,
an extra floor of `xqcut` on the energy of an outgoing pair meeting in an
s-channel of the given channel, and a lower limit on τ, `(Σ xe)²/s`. The τ
limit acts as a hard cut. With the resolved `ptj = xqcut` (the default, the
only `ktscheme` accepted being 1) it is implied by the rewritten cuts — a leg's
energy is at least its pT, and a pair holding a cut jet has at least `xqcut` —
so it cuts nothing; `cuts::madevents_xqcut_tau_floor_is_implied_by_the_rewritten_cuts`
pins that on 200k sampled points. With a resolved `ptj < xqcut`
(`auto_ptj_mjj = F`, or `ptj < 0`) it becomes a σ-changing cut that differs
between integration channels, and rather than build a channel-dependent cut the
card is refused (`XqcutAboveJetThreshold`)[^n41-m1]. MadGraph accepts that card,
so this is a parity gap:
[feature/mlm-ptj-below-xqcut-refused](../backlog/feature/mlm-ptj-below-xqcut-refused.md).
How the `xqcut` floors enter the phase-space maps is
[phase-space/cut-implied-timelike-floors](../phase-space/cut-implied-timelike-floors.md).

## Clustering under `xqcut`

With `ickkw = 1` or `xqcut > 0`, every event is clustered, even on a card that
fixes every scale (`reweight.f:643`), and a point with a jet vertex below
`xqcut` is zero-weighted (`ScaleRefusal::JetCut`) term by term
([scales-pdf/mlm-scales](../scales-pdf/mlm-scales.md)).

## What MadGraph does that vibegraph does not

- `create_default_for_process` auto-enables `ickkw = 1`, `xqcut = 30` on
  mixed-multiplicity jet cards when MadGraph *writes* a process's default card
  (`banner.py:4924-4966`), and forces `dynamical_scale_choice = -1` then
  (`banner.py:4966`). vibegraph reads the card it is given; a MadGraph-written
  card already carries these values[^n41-record].
- MadGraph's `<MGRunCard>` record of a matched card is the card after
  `banner.py`'s edits only, not the resolved card; vibegraph writes that record
  from `RunCard::banner_values` ([events/mlm-matched-event-record](../events/mlm-matched-event-record.md)).

## Reference cards

The matched references run at 13 TeV with `vector_size = 1` (the vector path
restores a different `SCALUP`): `pp_to_llj_mlm` (`xqcut = 20`, `ptj` auto),
`pp_to_ll_0j2j_mlm` (`@0 + @1 + @2`), `pp_to_llj_xqcut_only` (`ickkw = 0`, the
pure-cut branch), `pp_to_llj_mlm_alps2` (`alpsfact = 2` with `use_syst = F` —
otherwise MadGraph silently measures `alpsfact = 1`), and `pp_to_ttx_0j1j_mlm`
(`xqcut = 30`, MadGraph's own default for a mixed-multiplicity jet card, pinning
the rewrite at a second value)[^n41-m0]. Their gates are
[validation/mlm-sigma-gate](../validation/mlm-sigma-gate.md) and
[validation/mlm-dump-oracle](../validation/mlm-dump-oracle.md).

[^n41-record]: Note 41 §1.4: setup cuts, phase-space hints, run-card rules.
[^n41-card]: Note 41 §3.5, the run-card design.
[^n41-m0]: Note 41 M0: the five reference cards and the card choices beyond the brief.
[^n41-m1]: Note 41 M1 landed: the run-card rules as implemented, the τ-minimum audit, the `setrun.f:82` finding.
[^mg-setcuts-xqcut]: MadGraph `setcuts.f:156-189`.
