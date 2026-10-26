---
type: Physics Convention
title: MadGraph cut conventions (cuts.f) and what vibegraph implements
description: "Cut families and class membership, rapidity not pseudorapidity, ΔR and mass thresholds as signed squares, lab-frame evaluation, and parse-and-detect refusal of unimplemented cuts."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}, {by: claude-code/claude-opus-5-5, at: 2026-10-22}]
tags: [run-card, cuts, conventions, madgraph-parity, kinematics]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n18-inventory, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L163-L196", title: "Note 18 §1.5, the cut inventory and what must actually cut"}
  - {id: n18-design, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L312-L341", title: "Note 18 §2.6, cuts.rs as a compiled filter"}
  - {id: n18-h6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5, H6 cuts.f convention pins and H7 lab-frame cuts"}
  - {id: n18-outcome, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L935-L1039", title: "Note 18 outcome, load-bearing findings"}
  - {id: mg-cuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cuts.f#L225", title: "MadGraph LO cuts.f (FIRSTTIME squaring of dr)"}
  - {id: mg-kin, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/kin_functions.f#L95", title: "MadGraph kin_functions.f rap, R2, DELTA_PHI"}
  - {id: mg-setcuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setcuts.f#L200-L235", title: "MadGraph setcuts.f class membership"}
  - {id: mg-ht, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cuts.f#L517-L913", title: "MadGraph LO cuts.f, the jet and b transverse-momentum lists and the HT cuts"}
  - {id: mg-ht-defaults, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4401-L4411", title: "MadGraph banner.py, the HT parameters and their defaults"}
---

# MadGraph cut conventions and what vibegraph implements

To agree with a MadGraph cross section, vibegraph must apply the cuts MadGraph
applies **by default** — `ptl = 10`, `etal = 2.5`, `drll = 0.4`, `ptj = 20`,
`etaj = 5`, `pta = 10`, `etaa = 2.5`, `drjj = draa = draj = drjl = dral = 0.4`
are active out of the box — and must notice every cut it does not
implement[^n18-inventory]. `Cuts::compile` / `compile_with`
(`vibegraph-lib/src/cuts.rs`) classifies each final-state leg into MadGraph's
letter classes and bakes the active thresholds into a flat check list;
`Cuts::pass` is the phase-space indicator. The enforced reference is the
generated Fortran (`SubProcesses/cuts.f`, `setcuts.f`, `Source/kin_functions.f`
of the LO template), not the run-card comments or the Python layer[^n18-design].

## Class membership (`setcuts.f:200-235`)

| class | rule |
|---|---|
| jet `j` | `|pdg| ≤ min(maxjetflavor, 6)` or `pdg = 21` |
| b | `maxjetflavor < |pdg| ≤ 5` |
| lepton `l` | `|pdg| ∈ {11, 13, 15}` |
| photon `a` | `pdg = 22` |
| neutrino `n` | `|pdg| ∈ {12, 14, 16}` |
| heavy | mass above 10 GeV |

`maxjetflavor` (default 4) decides b-versus-jet membership. Single-leg cuts are
skipped (`do_cuts = false`) for neutrinos, for legs heavier than 20 GeV, and,
under `cut_decays = F`, for decay products of a forced line[^mg-setcuts].

## Conventions

- **Rapidity, not pseudorapidity.** The single-leg `eta{j,b,a,l}` cut and the
  ΔR separation use `rap = ½·ln((E+pz)/(E−pz))` (`kin_functions.f:95`, applied
  as `abs(rap)` at `cuts.f:426`). Equal to pseudorapidity for massless legs, but
  rapidity is the enforced definition[^mg-kin].
- **ΔR² = Δφ² + Δy²** (`kin_functions.f:42`, `R2`), with
  `Δφ = acos(clamp(·, ±0.99999999))` (`DELTA_PHI`, `:164`, clamp at `:181-182`), the azimuthal
  opening angle in `[0, π]`, so wrap-around is intrinsic. The clamp invents up to
  `acos(0.99999999) ≈ 1.5e-4` of separation for a collinear pair, and every
  separation bound derived from a ΔR threshold is taken at the correspondingly
  relaxed radius (`DELTA_PHI_CLAMP_SLACK`).
- **The `dr` threshold is squared once.** `setcuts.f:346` stores the raw value,
  and `cuts.f`'s FIRSTTIME block (`:207-280`) squares it (`r2min = r2min·|r2min|`,
  `cuts.f:225`, "Since r2 returns distance squared") before the
  `r2(...) < r2min` test (`:443`). The effective bound is the ordinary
  `ΔR ≥ dr`; vibegraph stores the signed square `dr·|dr|`[^mg-cuts]. Read the
  surrounding control flow, not just the cited line: the cited line alone
  suggests a different bound.
- **Mass and `ptll` thresholds are signed squares** too: `mm{…}` as `mm·|mm|`
  against `(p_i+p_j)²` (`setcuts.f:399`, `cuts.f:485`), `ptll` as `ptll·|ptll|`
  against `(Σpx)² + (Σpy)²` (`setcuts.f:479`, `cuts.f:462`).
- **`mmll` and `ptll` apply to same-flavour opposite-charge lepton pairs only**
  (`setcuts.f:396, 473`).
- **An energy minimum is a strict `≤` reject** (`cuts.f:413`).
- **The ŝ window** (`dsqrt_shat`, `dsqrt_shatmax`) compares `(p₁+p₂)²` against
  the squared thresholds, and only with two incoming legs (`cuts.f:310`,
  `nincoming.eq.2`), so a decay never reads it[^n18-h6].

## HT (`htjmin`, `htjmax`, `ihtmin`, `ihtmax`)

`cuts.f` builds two lists of transverse momenta, one per jet-class leg
(`is_a_j`) and one per b-class leg (`is_a_b`) (`cuts.f:521-534`), then
cuts on two scalar sums[^mg-ht]:

- **`htj`** is the sum over the jet-class legs (`cuts.f:876-880`): gluons and
  quarks with `|pdg| ≤ min(maxjetflavor, 6)`. At the default
  `maxjetflavor = 4` a b quark is not in it; at 5 it is; at 3 the charm quark
  leaves it too. It fails when `htj < htjmin` or when `htjmax ≥ 0` and
  `htj > htjmax` (`cuts.f:892`), so the thresholds themselves pass.
- **`iht`** ("inclusive") is `htj` plus the b-class legs
  (`maxjetflavor < |pdg| ≤ 5`) (`cuts.f:898-906`), with the same edges against
  `ihtmin` / `ihtmax` (`cuts.f:908`). It is not over every visible leg:
  leptons, photons and neutrinos are in neither sum. At `maxjetflavor ≥ 5`
  `iht = htj`.
- **No preselection.** The lists are filled from the class alone, with no pT,
  rapidity or `do_cuts` test (`cuts.f:524-534`): decay products under
  `cut_decays = F` and jet-class legs above 20 GeV (a top at
  `maxjetflavor = 6`) count. The run card says so in words: "ht cuts always
  apply" on decay products (`run_card.dat:102`).
- **Frame.** Transverse momenta are invariant under the beam-axis boost, so the
  lab and partonic frames give the same sums.
- **No jets.** With no jet-class leg, `htjmin > 0` rejects the point
  (`cuts.f:817`) and the `htj` maximum is never read (the `njets > 0` branch,
  `:842`); `iht` is still checked (`:908`). A zero `htj` reproduces both.
- **Defaults** (`banner.py:4402-4405`): `htjmin = 0`, `htjmax = -1`,
  `ihtmin = 0`, `ihtmax = -1`, and `remove_all_cut` resets them to the same
  values. A minimum at or below zero and a negative maximum never reject[^mg-ht-defaults].

The partial sums `ht2min`..`ht4max` (`htjmin4`/`htjmax4`, `cuts.f:878-890`)
are over the 2, 3 or 4 hardest jets and stay parse-and-detect. MadEvent also
folds `htjmin²` into its `smin` (`setcuts.f:571`); vibegraph's `shat_min`
takes `max(htjmin, ihtmin)²`, from `√ŝ ≥ Σ p_T` over the summed legs.

## Cuts are evaluated in the laboratory frame

The matrix element is evaluated in the partonic centre of mass (beams along
±z, the pruned evaluator's contract), but MadGraph's cuts are lab-frame
observables: `cuts.f` shifts every rapidity by the parton system's rapidity
(`cm_rap`). So the final-state momenta are boosted along z by
`y = ½ln(x₁/x₂)` before `Cuts::pass`, and `|M|²` stays in the CM; where the two
frames coincide the boost is skipped. A boost moves η and ΔR but not a
back-to-back LO pair's `Δφ = π`, so `drll = 0.4` never fires on
`p p > e+ e-` — worth knowing before assuming a default cut exercises real logic
on a given process[^n18-h6][^n18-outcome].

## What is implemented, and what is parse-and-detect

Implemented: the ŝ window; single-leg pT, E, rapidity min and max for classes
j, b, a, l; pairwise ΔR and invariant mass (min and max) over the class-pair
tags; `ptll`; `mmnl`; `htjmin`, `htjmax`, `ihtmin`, `ihtmax`; the decay-chain Breit–Wigner windows (`cut_bw` in
`myamp.f`: a forced line rejects the point unless `|√p² − M| < bwcutoff·Γ`,
a positive Γ floored at `M·small_width_treatment`) and `cut_decays`. Under `xqcut > 0` the
jet thresholds are rewritten first ([run-card/matching-parameters](matching-parameters.md)).

Everything else tagged `cut=` in `banner.py` is **parse-and-detect**:
`UNIMPLEMENTED_CUTS` lists `misset(max)`, `ptheavy`, `ptonium`/`etaonium`,
`xpt{j,b,a,l}`, the leading-object `ptj1..4`/`ptl1..4` min/max and `cutuse`, the
partial sums `ht2min`..`ht4max`, `ptgmin` (Frixione isolation), `xetamin`/`deltaeta` (VBF),
`ktdurham`/`dparameter`/`ptlund` (CKKW-L merging) and the per-PDG dictionary
cuts. `Cuts::compile` returns `CutError::UnimplementedCutActive` when any of
them differs from its MadGraph default — or from the reset value MadGraph's
`remove_all_cut` gives a decay card, which is equally inactive. An active but
unimplemented cut is never silently ignored; the error marks exactly when a real
process needs one implemented[^n18-design][^n18-outcome]. The isolation
parameters (`r0gamma`, `xn`, `epsgamma`, `isoem`) and `mxx_only_part_antipart`
qualify only refused cuts and are classified benign
([run-card/field-classification](field-classification.md)).

Small inventory facts: `etajmin`/`etaamin` carry `cut='a'` in `banner.py`
(harmless; `setcuts.f` keys `etaXmin` by class)[^n18-h6].

## What the compiled cuts also provide

The cuts feed the phase-space maps, as bounds that must hold at every accepted
point:

- `shat_min()`: a lower bound on ŝ (`dsqrt_shat²`, `mmll²` when an
  `l⁺l⁻` pair is present, `(2·ptl)²` with exactly two final leptons,
  `max(htjmin, ihtmin)²`), used by the
  hadronic `(τ, y)` map ([phase-space/hadronic-tau-y-sampling](../phase-space/hadronic-tau-y-sampling.md));
- `spacelike_floor()` and `timelike_floor(slots)`: regulator scales and
  invariant-mass floors implied by the cuts
  ([phase-space/cut-implied-timelike-floors](../phase-space/cut-implied-timelike-floors.md),
  [phase-space/spacelike-floor](../phase-space/spacelike-floor.md)).

A cut band that is thin in the integration variables defeats VEGAS: the
direct `x₁, x₂` map made the `m_ll ∈ [60, 120]` window a thin diagonal band and
came out 6% low before the `(τ, y)` map made it a one-dimensional bound[^n18-outcome].

## Validation

Per-cut boundary unit tests (both sides of each threshold, the rapidity sign,
the ΔR φ-wrap) and the `UnimplementedCutActive` detection test pin the
semantics; the end-to-end check is the σ gate against MadGraph runs sharing the
same card ([validation/sigma-gate](../validation/sigma-gate.md)). A MadGraph
reference that needs a cut outside this set uses MadGraph's dummy-cuts hook,
whose own conventions are [tooling/madgraph-custom-cuts](../tooling/madgraph-custom-cuts.md).
The parser and defaults are [run-card/run-card-parser-and-defaults](run-card-parser-and-defaults.md).

[^n18-inventory]: Note 18 §1.5, the default-active cuts and the inventory.
[^n18-design]: Note 18 §2.6, implemented families and parse-and-detect.
[^n18-h6]: Note 18 §5 H6 (cuts.f pins) and H7 (lab-frame cuts).
[^n18-outcome]: Note 18 outcome: the `(τ, y)` remap, lab-frame cuts, the dr-squaring lesson.
[^mg-cuts]: MadGraph `cuts.f:225` (the squaring, inside the FIRSTTIME block `:207-280`) and `:443` (the test), at the pinned commit.
[^mg-kin]: MadGraph `kin_functions.f:95` (`rap`), `:42` (`R2`), `:164` (`DELTA_PHI`, clamp `:181-182`).
[^mg-setcuts]: MadGraph `setcuts.f:200-235`, `do_cuts` (`:201-215`) and class membership (`:217-235`).
[^mg-ht]: MadGraph `cuts.f:521-534` (the lists), `:817` (no jets), `:874-913` (the HT cuts); `setcuts.f:498-507` (the parameters' commons), at the pinned commit.
[^mg-ht-defaults]: MadGraph `banner.py:4402-4405` (defaults), `:3858-3873` (`remove_all_cut`); `run_card.dat:102, 218-223` (the template's wording).
