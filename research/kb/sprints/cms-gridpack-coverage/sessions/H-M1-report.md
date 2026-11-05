---
type: Session Report
title: "H-M1: massive incoming partons, MadGraph's prescription ported"
description: "Proton subprocesses with a massive b or c on a beam integrate with MadGraph's kinematics: x and ŝ from the massless convolution, mom2cx beams on their mass shells, the λ flux, the incoming threshold, a part of their own, and write_leshouche's anchored record. Local MadGraph σ and LHE oracles recorded; the survey-card σ is an M3 request."
status: draft
tags: [proton, kinematics, masses, madgraph-parity, cms, sprint-h, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Session H-M1 of [sprint H](../sprint.md), backlog item
`massive-incoming-partons-refused`, branch `sprinth-massive-incoming`.

## What MadGraph does (pinned source)

Read at the submodule pin, `Template/LO/SubProcesses` and `aloha/template_files`:

- **x and ŝ.** `gen_mom` (`genps.f:331-348`; `cm_rap` at `:345`) draws `τ, η` with `GENCMS`
  (`genps.f:1621`, `TAUMIN = 0`) and sets `ŝ = x₁x₂·stot`. Nothing in the
  convolution knows the parton masses: `ŝ` and the PDF `x` are the massless ones.
- **Beams.** `genps.f:411` calls `mom2cx(√ŝ, m(1), m(2), 1, 0, p1, p2)` with
  `m(i) = pmass(i)`. `mom2cx` (`aloha_functions.f:1255`) gives
  `E₁ = (ŝ + m₁² − m₂²)/(2√ŝ)`, `|p| = √((md2/√ŝ)² − 2(m₁²+m₂²) + ŝ)/2`
  along `+z`, clamped at `1e-99` under the root, so below threshold it returns
  `|p| ≈ 1.6e-50`, not a NaN.
- **Below threshold.** `one_tree` (`genps.f:815`) rejects
  `√ŝ < max(Σm_out, m₁+m₂)` with `jac = −5`, before any weight. The point
  carries zero weight. The 2 → 1 trap at `genps.f:772` returns before this
  check, which matters only for `M < m₁+m₂`.
- **Flux.** `genps.f:427`: `flux = 1/(2√LAMBDA(ŝ, m₁², m₂²))`, with `LAMBDA` in
  its expanded form (`genps.f:1498`).
- **The matrix element** is called at these `p` (massive incoming spinors);
  the mirror (`super_auto_dsig_group_v4.inc:815-819`) rotates `py, pz` of every
  leg, beams included.
- **Cuts and scales** read `p` boosted by `cm_rap = ½ln(x₁E₁/(x₂E₂))`, the
  massless rapidity. Scale choice 4 is `sumdot(p1, p2) = ŝ`, unchanged by the
  masses.
- **The event record** (`unwgt.f:633-725`) uses another boost. If beam 1's
  parton is massless, the boost is the one taking `ebi = (p⁰₁/x₁, …)` at the
  proton mass to `ebeam(1)`. Otherwise it uses beam 2. If either beam then ends
  above its beam energy, it falls back to `cm_rap`. The source comment says
  "wrong boost if both parton are massive!". For two equal masses,
  `p⁰₂ = √ŝ/2`, so the anchored boost *is* `cm_rap` up to the proton-mass term.
  For `IMIRROR = 2` the mass column is swapped (`unwgt.f:728`).
- **Subprocess classes** (`madgraph/iolibs/group_subprocs.py:454`, `find_process_classes`) key on
  every leg's mass, so `b b~` and `u u~` never share a `P` directory.

## What was implemented

- `derive_flavor_groups` no longer refuses a massive incoming leg.
  `ProtonError::MassiveInitialState` is gone. It is replaced by
  `UnequalInitialMasses`, which refuses a decomposition whose members differ in
  incoming masses.
- `split_by_outgoing_masses` became `split_by_external_masses`: the key is the
  incoming masses in beam order plus the outgoing multiset, so a massive `b`/`c`
  initial state is a part of its own, as in MadGraph. `MultiplicitySum`'s
  one-part-per-phase-space assertion uses the same key.
- `partonic_beams(√ŝ, [m₁, m₂])` puts the beams on their shells (`mom2cx`). The
  massless case is written out as before, bit for bit.
- `ProtonIntegrand` carries `initial_masses`, with these effects:
  - `build_frames` puts the partonic-CM beams on their shells, and the lab
    frame the cuts read is these beams boosted by `cm_rap`.
  - `inverse_flux` gives `1/(2√λ)` and returns `None` (zero weight) at
    `√ŝ ≤ m₁+m₂`.
  - `τ_min` is raised to `(m₁+m₂)²/s` where that exceeds the cut hint.
  - Massless parts take the old arithmetic unchanged: `1/(2·(√ŝ·√ŝ))` equals
    `1/((2√ŝ)√ŝ)` exactly, because doubling is exact.
- The flavour-group probe draws its beams through `partonic_beams` and lifts
  its energy floor above `m₁+m₂`.
- `ProtonIntegrand::record_momenta(event, ordering)` builds the LHE record frame
  of `write_leshouche`:
  - The beams sit on their shells for the drawn ordering. The exchanged ordering
    swaps the shells: beam 1 carries the member's second parton.
  - The anchor is beam 1 when its parton is massless, beam 2 otherwise. The
    boost takes the anchor's `E*/x` to its beam energy.
  - The boost falls back to `cm_rap` when either beam ends above its beam
    energy.
  - Massless parts return `event.lab` as before.
  - `generate/source.rs` calls it in place of `event.lab`. This is the one-line
    touch in the generate area.
- Not touched: the phase-space channel maps. They already build their beams
  with the diagram's own leg masses (`DiagramChannel::beams_at` →
  `beam_momenta`), as `one_tree` does with `p(1), p(2)`.

### Deviation, recorded

The anchored record boost keeps the anchor and its beam massless, as
`ProtonBeams` does throughout. MadGraph puts them at the proton mass 0.938. The
two boosts differ at `O(m_p²x²/E*²)`. Measured on MadGraph's own records below,
the gap is at most 2.4e-5 in the anchor's `E/(xE_b)` (`g b`). In
`|Y − ½ln(x₁/x₂)|` it is at most 2.8e-4 (`b b~`, `mmll = 20`) and 1.2e-3
(`c c~`, `mmll = 10`), against 4e-7 and 1.6e-6 in vibegraph's records. I
**infer** that this is the proton-mass term, which grows as `E*/x` falls: the
lower-`ŝ` sample shows the larger gap. Reading `unwgt.f`, MadGraph's
massless-parton record takes the same anchored boost, so this crate's light-cone
record already differs from it by the same term. The deviation predates this
session and stays consistent.

## Oracles, each with its command and output


Scratch paths below are under `$SP = /tmp/claude-0/-home-user-vibegraph/0bdd8e0a-85fe-5422-8ff6-63eab43fffee/scratchpad`.
MadGraph is the pinned submodule, `python3 research/refs/mg5amcnlo/bin/mg5_aMC`,
run locally with gfortran and `pdlabel = nn23lo1` (LHAPDF is not installed).
vibegraph reads the same run card with `pdlabel = lhapdf`, `lhaid = 247000`
(NNPDF23_lo_as_0130_qed from `validation/pdf`).

### 1. Amplitudes at massive incoming momenta: per-helicity |M|² against local MadGraph standalone

`output standalone` for `b b~ > mu+ mu-` (sm, MB = 4.7), `c c~ > mu+ mu-`
(sm-c_mass, MC = 1.55) and `g b > z b` (sm). A Fortran driver called
`SMATRIX`/`SMATRIXHEL` at 30 partonic-CM points: beams from `mom2cx`, the
outgoing pair at three angles, at `√ŝ` = 12/30/91.188/500 (bb), 4/12/91.188 (cc)
and 100/200/1000 (gb). `IDEN` and `HELAVGFACTOR` are taken out. The script is
`$SP/H-M1-mg/H-M1-sa_points.py`. Its output is committed as
`vibegraph-lib/tests/data/massive_incoming_standalone.json` and read by the
hermetic test `proton::tests::massive_incoming_helicity_amplitudes_match_madgraph_standalone`.

```
$ cargo test --workspace --lib -- massive_incoming_helicity --nocapture
bbmumu: worst relative deviation 1.25e-14
ccmumu: worst relative deviation 1.44e-14
gbzb: worst relative deviation 4.98e-15
test result: ok. 1 passed; 0 failed
```

The deviation is the worst over every helicity, normalised to the largest
helicity at the point, and over the helicity sum. Blind spot: per-helicity
|M|² cannot see a phase per helicity or a colour-flow permutation. The complex
per-diagram and per-flow values at massive incoming spinors are already gated
by the banked hermetic rows `bbx_to_h_identity` (per-diagram, MB = 4.18),
`tata_to_ttx_tensor4f` (per-diagram, massive τ beams) and
`bbx_to_ccx_emmm_qcd0` (per-helicity coherent, MB = 4.7) in `amplitude_oracle`.
No new M3 amplitude bank is needed.

During setup the driver first compared raw `SMATRIXHEL` and was off by a uniform
4. That is MadGraph's `HELAVGFACTOR` in `SMATRIXHEL`, not a physics difference:
the helicity sum agreed at 2e-15.

### 2. Kinematics (hermetic, fails before the change)

`proton::tests::massive_partons_sit_on_their_mass_shells_with_madgraphs_flux`
(`b b~ > mu+ mu-`, 6500 × 4000 GeV beams, three `(x₁, x₂)`) checks five things:
- each partonic beam has `m² = MB²` to 1e-10;
- `p₁ + p₂ = (√ŝ, 0, 0, 0)` to 1e-12 at `ŝ = x₁x₂s`;
- `1/F` equals MadGraph's `1/(2√LAMBDA)` in its expanded form to 1e-12, and is
  not `1/(2ŝ)`;
- `None` at and below `√ŝ = 2MB`;
- the lab sum has mass `√ŝ` at rapidity `cm_rap`.

`a_massive_parton_record_is_anchored_where_madevent_anchors_it` (`g b > b a`)
checks the record frame:
- Direct ordering: the massless beam 1 sits at `x₁E₁` to 1e-12, and the b is
  on its shell and not at `x₂E₂`.
- Exchanged ordering: the b is on beam 1, the gluon anchors beam 2 at `x₂E₂`.
- Momentum is conserved in both orderings.

`multiplicity::tests::massive_incoming_partons_split_into_a_decomposition_of_their_own`
checks a five-flavour `p p > mu+ mu-`: two parts, with incoming masses
`[0,0]` and `[4.7,4.7]`, the second holding only `b`/`b~` members.

### 3. Refusals

What stays refused:
- a decomposition whose members differ in incoming masses (`UnequalInitialMasses`),
  pinned by `unequal_incoming_masses_are_refused_in_one_decomposition`;
- everything that was refused before for other reasons.

The old `MassiveInitialState` had no Rust test. The string survives only in
`validation/cms_gridpacks/test_parse.py`, which classifies old logs and is left
alone.

### 4. Mutation evidence

`$SP/H-M1-mutate.py` applies each mutation, runs the five new tests and
restores the source. `git diff --stat` was unchanged afterwards. Log:
`$SP/H-M1-mutations.log`.

| Mutation | Tests that fail |
|---|---|
| M0: restore a refusal of massive incoming legs (the state before the change) | kinematics, record, split, unequal-mass refusal |
| M1: massless flux `1/(2ŝ)` for massive partons | kinematics |
| M2: light-cone partonic beams | kinematics, record |
| M3: no incoming threshold | kinematics |
| M4: record always anchored on beam 2 | record |
| M5: exchanged record keeps the direct shells | record |
| M6: split key ignores incoming masses | split |

The standalone amplitude test passes under every mutation, as it should: it
builds its own momenta.

### 5. σ against local MadGraph, 5 seeds each, unweighted mean

Common settings: 13 TeV, `nn23lo1`/247000, fixed `μR = μF = 91.188`,
`ptl = 0`, `etal = −1`, `drll = 0`. Cards:

- `mmll = 20` for `b b~ > mu+ mu-` plus `add process b~ b > mu+ mu-` (sm).
- `mmll = 10` for `c c~ > mu+ mu-` plus `add process c~ c > mu+ mu-` (sm-c_mass).
- `g b > z b` with `define p = g b`, `generate p p > z b` (sm), no cuts on the b
  and `maxjetflavor = 4`.

MadGraph seeds were 11–15 (31–35 for `p p > z b`), with 10000 events and the
σ read from `Integrated weight` / `<init>`. vibegraph seeds were 1–5 under the
default convergence target (0.1%). The MadGraph cards are
`$SP/H-M1-mg/out_*.mg5` and `launch_*.mg5`; the vibegraph cards are
`$SP/H-M1-vg/*.proc` and `*.run_card`. `$SP/H-M1-stats.py` produces the
summary below; its output is `$SP/H-M1-stats.out`.

Per-seed σ [pb] (MadGraph value ± error, then vibegraph value ± error):

```
bbsym  MG 81.637±0.0364 81.449±0.0375 81.610±0.0367 81.630±0.0368 81.618±0.0408
       VG 81.6562±0.0576 81.7006±0.0609 81.7078±0.0712 81.6303±0.0649 81.6994±0.0580
bb0sym MG 77.618±0.0326 77.419±0.0375 77.482±0.0368 77.508±0.0374 77.550±0.0374     (MB = 0, sm-no_b_mass)
       VG 77.5795±0.0551 77.6071±0.0583 77.6280±0.0684 77.5289±0.0677 77.6146±0.0531
ccsym  MG 2031.224±0.875 2032.364±0.618 2033.268±0.968 2030.754±0.971 2031.868±1.128
       VG 2031.272±1.529 2031.675±1.538 2032.145±1.627 2030.031±1.752 2032.065±1.954
pbzb   MG 994.813±0.714 994.868±0.684 995.271±0.742 995.420±0.859 994.508±0.603
       VG 995.209±0.758 995.326±0.812 996.025±0.967 996.515±0.977 995.009±0.875
```

```
bbsym (MB=4.7)
  mg: mean 81.5888  seed spread (sd) 0.07885 (9.66e-04)  se(scatter) 0.0353  se(quoted) 0.0168  chi2/dof 4.45
  vg: mean 81.6788  seed spread (sd) 0.03391 (4.15e-04)  se(scatter) 0.0152  se(quoted) 0.028  chi2/dof 0.28
  vg-mg: +0.09004 (+1.10e-03); pull +2.35 (scatter se), +2.75 (quoted se)
ccsym (MC=1.55)
  mg: mean 2031.9  seed spread (sd) 0.9821 (4.83e-04)  se(scatter) 0.439  se(quoted) 0.415  chi2/dof 1.14
  vg: mean 2031.44  seed spread (sd) 0.8598 (4.23e-04)  se(scatter) 0.385  se(quoted) 0.755  chi2/dof 0.24
  vg-mg: -0.458 (-2.25e-04); pull -0.78 (scatter se), -0.53 (quoted se)
bb0sym (MB=0 control, sm-no_b_mass)
  mg: mean 77.5154  seed spread (sd) 0.07448 (9.61e-04)  se(scatter) 0.0333  se(quoted) 0.0163  chi2/dof 4.56
  vg: mean 77.5916  seed spread (sd) 0.0393 (5.06e-04)  se(scatter) 0.0176  se(quoted) 0.0272  chi2/dof 0.36
  vg-mg: +0.07624 (+9.84e-04); pull +2.02 (scatter se), +2.40 (quoted se)
g b + b g > z b (MB=4.7)
  mg: mean 994.976  seed spread (sd) 0.3679 (3.70e-04)  se(scatter) 0.165  se(quoted) 0.324  chi2/dof 0.28
  vg: mean 995.617  seed spread (sd) 0.6312 (6.34e-04)  se(scatter) 0.282  se(quoted) 0.394  chi2/dof 0.48
  vg-mg: +0.6409 (+6.44e-04); pull +1.96 (scatter se), +1.25 (quoted se)
mass effect sigma(MB=4.7)/sigma(MB=0) mg: 1.05255 +- 0.00143
mass effect sigma(MB=4.7)/sigma(MB=0) vg: 1.05268 +- 0.00069
```

Reading:

- **b at 4.7.** vibegraph is +1.10e-3 above MadGraph. The massless-b control
  (`sm-no_b_mass`, both sides, same seeds) shows the same offset, +0.98e-3, so
  the offset is in the massless `b b~` chain. The likely cause is the built-in
  `nn23lo1` against the LHAPDF grid, but that is **inferred, not measured**.
  The mass treatment is visible in the ratio: σ(4.7)/σ(0) = 1.05268 ± 0.00069
  in vibegraph against 1.05255 ± 0.00143 in MadGraph. That is a +5.3% effect,
  matched to 1.2e-4. The known-wrong comparison is the zeroed-mass treatment
  the user rejected: vibegraph's massless-b σ against MadGraph's massive one,
  −4.9%, about 100× the combined error (scatter-based standard errors).
- **c at 1.55.** −2.25e-4, pull −0.8.
- **Unequal masses, both orderings (`g b`, `b g`).** +6.4e-4, pull +2.0 on the
  scatter error and +1.25 on the quoted one.
- **MadGraph's own spread.** MadGraph's five `b b~` seeds have χ²/dof 4.5: its
  quoted errors are below its own seed scatter, and seed 12 is low in both the
  massive and the massless run. vibegraph's χ²/dof are 0.24–0.48, so its quoted
  errors are conservative. Five seeds give a spread; they do not calibrate a
  ±1e-3 statement. None of these rows is a gate.

The MadGraph side of every row is a symmetric card, with both orderings
generated. On the explicit card alone (`generate b b~ > mu+ mu-`) MadGraph gives
40.79 pb, half of 81.59, with b on beam 1 in all 10000 events. vibegraph's
proc cards here are the explicit ones (`$SP/H-M1-vg/*.proc`), and it adds the
mirrored ordering for every `a ≠ b` initial state, so it reports the symmetric
σ either way. That predates this session (Found 1).

### 6. The written LHE: incoming E, pz and mass, and the beam/x relation

One seed each, 10000 events. MadGraph ran with `use_syst = True` so `<mgrwt>`
carries `x₁, x₂`. vibegraph writes the same block. The commands were
`$SP/H-M1-lhebreak.py FILE 6500` (per beam ordering) and
`$SP/H-M1-lhecheck.py FILE 6500`.

```
=== bbsym MadGraph
(-5, 5) 5027 masses [4.7, 4.7] beam1 |E/xE-1| median 1.78e-02 max 2.37e+03; beam2 median 1.62e-02 max 3.50e+03; frac(beam1<1e-6) 0.000 frac(beam2<1e-6) 0.000
(5, -5) 4973 masses [4.7, 4.7] beam1 |E/xE-1| median 1.68e-02 max 1.34e+03; beam2 median 1.56e-02 max 2.50e+03; frac(beam1<1e-6) 0.000 frac(beam2<1e-6) 0.000
10000 events; worst |m - m_pole| 4.42e-05 GeV; beam-2 parton with pz>0: 1009
worst |Y - 1/2 ln(x1/x2)| 2.77e-04; E=x*Ebeam (1e-6) at beam1 0, beam2 0, neither 10000
=== bbsym vibegraph
(-5, 5) 4943 masses [4.7, 4.7] beam1 |E/xE-1| median 1.55e-02 max 3.71e+03; beam2 median 1.66e-02 max 3.36e+03; frac(beam1<1e-6) 0.000 frac(beam2<1e-6) 0.000
(5, -5) 5057 masses [4.7, 4.7] beam1 |E/xE-1| median 1.68e-02 max 5.96e+03; beam2 median 1.63e-02 max 2.33e+03; frac(beam1<1e-6) 0.000 frac(beam2<1e-6) 0.000
10000 events; worst |m - m_pole| 5.53e-05 GeV; beam-2 parton with pz>0: 1050
worst |Y - 1/2 ln(x1/x2)| 4.26e-07; E=x*Ebeam (1e-6) at beam1 2, beam2 2, neither 9998
=== pbzb MadGraph
(5, 21) 5034 masses [4.7, 0.0] beam1 |E/xE-1| median 1.63e-03 max 1.52e+01; beam2 median 2.05e-06 max 2.11e-05; frac(beam1<1e-6) 0.000 frac(beam2<1e-6) 0.288
(21, 5) 4966 masses [0.0, 4.7] beam1 |E/xE-1| median 3.55e-08 max 1.32e-05; beam2 median 1.68e-03 max 4.26e+00; frac(beam1<1e-6) 0.933 frac(beam2<1e-6) 0.000
=== pbzb vibegraph
(5, 21) 4964 masses [4.7, 0.0] beam1 |E/xE-1| median 1.61e-03 max 9.27e+00; beam2 median 6.67e-09 max 4.89e-08; frac(beam1<1e-6) 0.000 frac(beam2<1e-6) 1.000
(21, 5) 5036 masses [0.0, 4.7] beam1 |E/xE-1| median 6.63e-09 max 4.74e-08; beam2 median 1.66e-03 max 1.35e+01; frac(beam1<1e-6) 1.000 frac(beam2<1e-6) 0.001
=== ccsym MadGraph
10000 events; worst |m - m_pole| 1.47e-04 GeV; beam-2 parton with pz>0: 2008
worst |Y - 1/2 ln(x1/x2)| 1.21e-03; E=x*Ebeam (1e-6) at beam1 0, beam2 0, neither 10000
=== ccsym vibegraph
10000 events; worst |m - m_pole| 1.05e-04 GeV; beam-2 parton with pz>0: 1975
worst |Y - 1/2 ln(x1/x2)| 1.62e-06; E=x*Ebeam (1e-6) at beam1 0, beam2 0, neither 10000
```

What this establishes, beyond what the code reading alone shows:

- The incoming mass column is the pole mass, and the momenta are on it at the
  printing precision.
- With two equal masses neither parton is at `xE_b`. The system rapidity is
  `½ln(x₁/x₂)`: MadGraph's anchored boost collapses to `cm_rap`. The
  beam-2 parton has `pz > 0` in 10% (b) and 20% (c) of events on both sides.
- With one massive parton the massless one is anchored at `xE_b`, on whichever
  beam it sits, in both orderings. In vibegraph this holds to the 8-digit
  printing of `x`. In MadGraph it holds to 2.4e-5, the proton-mass term of the
  recorded deviation.
- MadGraph writes the `b` on beam 1 with mass 4.7 under `IMIRROR = 2`, as this
  record does.

### 7. Massless parts are bit for bit

On the massless path nothing in the arithmetic changed: the light-cone beams,
the lab beams `xᵢEᵢ`, `1/(2·ŝ)` and `τ_min`. The decomposition key also keeps
massless parts in the same order. Measured: the base commit (9ba3d96, built in
a throwaway worktree) and this branch ran
`p p > mu+ mu- j` (4-flavour, `mmll = 60`) with
`integrate --seed 3 --fixed-budget --neval 40000 --niter 4`, then
`generate --seed 5 --nevents 2000`.

```
base.log:σ = 404.424986 ± 1.333815 pb
new.log:σ = 404.424986 ± 1.333815 pb
per-iteration lines (timings stripped), md5: 780ece82f0d4cb28651635790a10337b both
event block of the LHE (from the first <event>), sha256: 18c9cca3…d9416e both
```

## Gates

The commands ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`. Logs are in `$SP/H-M1-*.log`.

```
$ cargo fmt --all --check                                         -> clean (exit 0)
$ cargo clippy --workspace --all-targets -j 3 -- -D warnings      -> CLIPPY EXIT 0
$ cargo test --workspace -j 3                                     -> TEST EXIT 0
  50 "test result" lines, all ok, summed: passed 1809 failed 0 ignored 25
$ bash validation/validate.sh     (banked layer; untracked .cargo/config.toml with
                                   [profile.release-debug] debug = 0, deleted afterwards)
  extended-validation clippy: Finished, no warnings
  80 "test result" lines: passed 2258 failed 0 ignored 109
  [report] 77 rows x 4 categories: the measured cells are the declared cells
  77 rows × 4 categories = 308 cells: 218 measured in the layers this run drove
  (213 ✅, 5 ⚠️, 14 ⏳, 76 — / uncovered). "every gate cell passed"
  VALIDATE EXIT 0
```

The five ⚠️ are the manifest's informational cells (`gg_to_gg`, `gg_to_gg_cg`
diagrams; `ee_to_wpwm_cw`, `ee_to_zh_smeft` amplitudes; `ee_to_mumua`
samples). None is a massive-parton row, and every enforced cell keeps its
status. The first `validate.sh` run was stopped by the harness at its 2-hour
background limit during the tests (22 test targets had run, 0 failures; log
`$SP/H-M1-validate-run1-killed.log`). The figures above are from the full
rerun, detached from the harness (`$SP/H-M1-validate2.log`).

## Backlog item status

The item's `closes_when` needs a σ match on **one of the survey's cards**. The
local cards above are not survey cards, and this container has neither the
gridpack cards nor LHAPDF. The item file stays, and closes on the M3 run below.

## M3 request: σ on two survey cards, and two banked rows

Run on the M3, which has `mg5_aMC` at the pin and LHAPDF.

1. **Pick the cards.** In the gridpack database (#71, campaign
   `main-23d3a76-200-pc`), select the refusals whose message contains
   `has an incoming leg of mass`. Take one at 4.7 GeV (b) and one at 1.2–1.4
   GeV (c), preferring the smallest final-state multiplicity. Record each
   gridpack name and its card hashes.
2. **MadGraph.** For each card, with its `proc_card`, `run_card` and the
   `param_card`/`customizecards` as shipped:
   `mg5_aMC proc_card.dat` (which runs `output`), then five `launch` runs into
   the same directory, each with `set iseed N` for N = 11..15,
   `set use_syst True` (so `<mgrwt>` records `x₁, x₂`), `set nevents 10000`
   and the shipped PDF (`lhapdf`/`lhaid`). Bank, per run:
   - `Events/run_0N/unweighted_events.lhe.gz`;
   - `Events/run_0N/*_banner.txt`;
   - the σ and error from `<MGGenerationInfo>` and `<init>`.
   Bank once: the generated `Cards/` and `SubProcesses/P*/` names. Five fresh
   `output` directories, rather than five launches in one, are what
   `bank_seeded_sigma.sh` banks; either is acceptable for a five-seed spread.
3. **vibegraph** on the same cards, from this branch:
   `vibegraph integrate proc_card.dat --run-card run_card.dat --param-card param_card.dat --seed S --out run_S`
   for S = 1..5, then `vibegraph generate run_1/grid.bin.zst --seed 7 -o events.lhe`.
   Report:
   - the five σ, their unweighted mean, sd and χ²/dof;
   - for one seed of each side, the §6 checks per beam ordering: incoming
     `m` against the pole mass, `|Y − ½ln(x₁/x₂)|` with `Y` the rapidity of
     `p₁ + p₂`, and `E/(xE_b)` per beam, reading `x` from `<pdfrwt>`. The two
     scripts that did this here (`$SP/H-M1-lhecheck.py`, `$SP/H-M1-lhebreak.py`,
     about 40 lines each) are not committed.
4. **Banked rows (optional, if the manager wants a permanent gate).**
   - Rows: `bbx_to_mumu_mb` (`b b~ > mu+ mu-` plus `add process b~ b > mu+ mu-`,
     sm, `mmll = 20`) and `ccx_to_mumu_mc` (sm-c_mass, `mmll = 10`), both
     with §5's run card at `pdlabel = lhapdf`, `lhaid = 247000`.
   - Bank them through `bank-seeded-sigma` as `integrals` cells. Add the
     massless-b control `sm-no_b_mass` with the same card, so the gate can read
     the mass-effect ratio, which is insensitive to the shared massless offset
     of §5.
   - These are committed references, so they are an M3 job and not this
     session's.


## Found

1. **Explicit-ordering cards are integrated with both beam orderings.**
   - Evidence: §5. MadGraph `generate b b~ > mu+ mu-` gives 40.79 pb with b on
     beam 1 in all 10000 events, and `generate g b > z b` gives 497.2 pb with g
     on beam 1 in all 10000. The symmetric cards give 81.59 and 994.98.
   - `Subprocess::has_mirror` is `incoming[0] != incoming[1]`, so vibegraph adds
     the mirror for every unequal pair, whatever the card wrote. Measured:
     vibegraph's proc card for the §5 `b b~` row is the explicit
     `generate b b~ > mu+ mu-`, and it gives 81.68 pb. MadGraph gives 40.79 on
     that card and 81.59 on the symmetric one. The massless case
     (`generate u u~ > e+ e-`) runs through the same line of code; it was not
     run.
   - To close: carry from the proc card whether each ordering was generated
     (MadGraph's `mirror_process`), and give the mirror term luminosity only when
     it was. Add an explicit-ordering σ row against MadGraph.
2. **A ~1e-3 offset on `b b~` initial states that does not depend on the b mass.**
   - Evidence: §5. vibegraph is +1.10e-3 above MadGraph at MB = 4.7 and
     +0.98e-3 at MB = 0, with the same seeds.
   - Suspected cause: MadGraph's built-in `nn23lo1` against LHAPDF set 247000
     for the b density at `Q = 91.188`. **Inferred**: the ccsym row, at
     −2.3e-4, shows no such offset for c.
   - To close: compare `xf_b(x, 91.188)` from both tables on an x grid (the
     `validate-pdf-grid` oracle has the machinery), or rerun the row on the M3
     with LHAPDF on both sides.
3. **MadGraph's own `b b~` seed scatter exceeds its quoted error** (χ²/dof 4.5
   over seeds 11–15, in both the massive and massless runs). Any banked `b b~` row
   should take its tolerance from the measured spread, not from MadGraph's error.
4. **Untested with massive incoming partons**, because no oracle was run:
   - MLM matching (`ickkw = 1`): the clustering now reads massive partonic-CM
     beams, as MadGraph's `cluster.f` does;
   - the kT-clustered scale (`dynamical_scale_choice = -1` beyond 2 → 2);
   - reweighting.
   To close: a matched or kT-scale massive-parton card against MadGraph's
   per-event scales (`validate-scales`-style dumps).
5. **The record boost takes the beams massless** (the recorded deviation). If
   bit-level parity with MadGraph's record is ever wanted, put `m_p = 0.938` into
   the anchored boost, and into the massless-parton record, which has the same
   `O(m_p²x²/E*²)` gap today.

## Brief corrections

- The brief reads "check what MadGraph does when ŝ < (m1+m2)²". `mom2cx` does
  not fail there: it clamps `|p|` at `√1e-99/2`. The rejection happens in
  `one_tree` (`genps.f:815`). The flux would be a NaN if it were formed,
  but it never is.
- The brief lists "the phase-space maps" as a site. They needed no change: the
  diagram channels already generate against massive beams from the diagram's own
  legs.
- The brief proposes cards like `b b~ > h`. A card written as one ordering is
  where the Found 1 doubling appears, so the σ rows use symmetric cards.
