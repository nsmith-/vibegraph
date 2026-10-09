---
type: Design
title: 1 → n decay processes and partial widths
description: "A decay is a one-initial process at rest with flux 1/2M, integrated to a partial width; MadEvent's decay-run card defaults and event layout."
status: draft
tags: [decays, partial-width, process-grammar, run-card, madevent]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n38-decays, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L129-L156", title: "Note 38 §1.3, decays and decay chains in MadGraph"}
  - {id: n38-d1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L497-L598", title: "Note 38 §4 D1, 1→n decay processes"}
  - {id: mg-banner-decay, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4784", title: "MadGraph banner.py, a decay's default run card"}
  - {id: mg-setcuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setcuts.f#L137", title: "MadEvent setcuts.f, nincoming = 1"}
measured:
  - {commit: fa07c04, pr: 12, landed_in: 1539abc, command: "cli_decay, ten seeds at --target-rel 2e-3, against MadEvent 3.7.1 ten seeds of 10k events"}
---

`generate t > w+ b` is an ordinary process with one initial particle. MadEvent integrates
it to a **partial width** rather than a cross section, and vibegraph does the same.
[^n38-decays]

## What a decay is here

- **Enumeration.** `diagrams::enumerate_decay` runs one decay's enumeration, the automatic
  lowest-`WEIGHTED` search included, as a unit, and refuses a process without exactly one
  initial particle (`DiagramError::NotADecay`). It returns a `Vec<DiagramSet>`, because a
  decay written with labels (`w+ > j j`) has several concrete assignments. With one initial
  particle every propagator is an s-channel line oriented away from the decaying particle,
  which is what `>` and `$$` filter on ([s-channel restrictions](s-channel-restrictions.md)).
  The same function enumerates each decay of a [decay chain](decay-chains.md).
- **Kinematics.** `hadronic::InitialState` is `Beams(FixedBeams)` or `Decay(DecayAtRest)`.
  A decay supplies √s = M, the incoming momentum `(M, 0, 0, 0)`, flux **1/(2M)**, no
  boost, and `Observable::PartialWidth` in GeV. The fixed-beam integrand and its
  per-diagram channels serve it unchanged; every line is timelike, so every channel is the
  all-timelike tree.
- **Amplitudes.** No evaluator change was needed: the evaluator takes an incoming massive
  fermion, vector or scalar as it is, and helicity pruning is skipped when `n_in != 2`
  (`helas/eval/compile.rs`).
- **Cards.** A 1 → n line passes the [card check](supported-card-check.md); a card cannot
  mix processes with different numbers of initial particles (a MadGraph error too). A
  polarization on the decaying particle (`t{L} > w+ b`) is refused, since at rest its
  helicity is a spin projection on an axis nothing pins; polarized products
  (`t > w+{0} b`) are supported ([polarization](polarization.md)).

## MadEvent's decay run, transcribed

Read from the pinned checkout (3.7.1) [^n38-d1]:

- `banner.py:4784` writes a decay's default run card with `remove_all_cut()`, and `:5045`
  forces `sde_strategy = 1` [^mg-banner-decay].
- `setcuts.f:137` (`nincoming.eq.1`) sets `lpp = 0`, `ebeam = M/2`, `scale = M` and
  `fixed_ren_scale` unless the card fixes it, and both `fixed_fac_scale` true
  [^mg-setcuts].
- **Cuts are applied**, by `cuts.f` in the rest frame, except the ŝ window, which is
  guarded by `nincoming.eq.2` (`cuts.f:310`).
- Measured on `t > w+ b` with `dsqrt_q2fact = 50/60` and `scale = 70`: `SCALUP` is 60, the
  larger fixed factorisation scale; `AQCDUP` is αs(M_t) = 0.1076279 unless
  `fixed_ren_scale`, then αs(70) = 0.1229055; `dynamical_scale_choice = 3` changes nothing.

`RunCard::decay_default` and `RunCard::for_decay` transcribe these rules
(`runcard_decay_defaults.json` pins the 107 cut resets), and `use_running_coupling`
applies `for_decay` itself on a decay. A 2 → n card carrying `remove_all_cut`'s reset
values of `ktdurham`, `ptlund`, `dparameter` or `deltaeta` is accepted, since `cuts.f`'s
`> 0` guards leave them off.

Two decay-card refusals are stricter than MadEvent:

- `sde_strategy = 2` on a decay card, because channel forests are 2 → n only
  ([backlog](../backlog/feature/decay-card-sde-strategy-2-refused.md));
- a proton-beam-only physics field off its default, refused at parse although a decay
  ignores it ([backlog](../backlog/feature/decay-card-proton-only-field-refused.md)).

MLM matching on a decay is refused too
([backlog](../backlog/feature/mlm-at-fixed-beams-or-decays-refused.md)).

## The event file

MadEvent's layout for `t > b e+ ve`, matched field by field [^n38-d1]:

- `<init>`: `6 0 1.730000e+02 0.000000e+00 0 0 247000 247000 -4 1`, with `XSECUP` the
  width in GeV.
- Per event: the top with status −1 at rest (MadEvent prints `pz` as 1e-14), the products'
  mothers `1 0` (`unwgt.f:741`), `XWGTUP` the width, `SCALUP` 91.188, `AQCDUP` 0.1076279.
- `PDFSUP` is MadEvent's `get_pdf_id(pdlabel)` (247000 on the banked fixed-energy runs),
  as for every fixed-energy run.
- MadEvent also writes a status-2 `W` when it is inside its Breit–Wigner window; this
  generator writes resonance records for decay-chain cards only
  ([backlog](../backlog/feature/plain-process-onwindow-resonance-records.md)).

`check-events` reads an empty beam 2 as a decay.

## Evidence

- **Two-body widths** (`sm_decay_widths.json`, from the model's `decays.py` through
  MadGraph's `model_reader` on `restrict_default`): 19 open channels of t, W+, Z and H
  agree to ≤ 5e-13 in Γ and 1e-12 in `Σ|M|²` at four orientations.
- **Many-body |M|²** (`decay_amplitudes.json`, MadGraph standalone `SMATRIX`, 24 points
  each): `t > b e+ ve` 5.7e-14, `h > e+ e- mu+ mu-` 1.7e-14, `z > e+ e- mu+ mu-`
  (8 diagrams) 1.3e-12, `t > b e+ ve a` 3.3e-14.
- **Widths**: ten seeds here at `--target-rel 2e-3` (`cli_decay`), ten MadEvent seeds of
  10k events, each mean ± max(quoted, spread/√n):

| Row | exact | MadEvent | here | pull |
|---|---|---|---|---|
| `t > w+ b` | 1.4914721 | 1.491506 ± 2.3e-5 | 1.491582 ± 1.6e-4 | +0.71 vs exact |
| `z > e+ e-` | 0.08396539 | 0.08396686 ± 1.3e-6 | 0.08397159 ± 8.7e-6 | +0.71 vs exact |
| `h > e+ e- mu+ mu-` | 2.4193128e-7 | 2.416009e-7 ± 1.5e-10 | 2.421078e-7 ± 1.4e-10 | +1.25 vs exact |
| `t > b e+ ve` | — | 0.1632493 ± 4.4e-5 | 0.1632357 ± 4.0e-5 | −0.23 vs MG |
| `h > 4l`, ptl 10, etal 2.5, drll 0.4, mmll 12 | — | 8.121266e-8 ± 1.0e-10 | 8.125823e-8 ± 6.1e-11 | +0.38 vs MG |
| `t > b e+ ve`, ptb 20, etab 2.5, ptl 15, etal 2.5, drbl 0.4 | — | 0.1418069 ± 5.8e-5 | 0.1417557 ± 4.4e-5 | −0.71 vs MG |

  χ²/dof over seeds here: 0.54–1.85. The `h > e+ e- mu+ mu-` exact width is a quadrature
  over the two off-shell Z lines (`decay_semianalytic.py`, 3e-9).
- **The `h > e+ e- mu+ mu-` row is a seed-sweep lesson.** There MadEvent is the side that
  misses: its 10k-event seeds sit 0.14% low (−2.2σ of their mean, 24 runs over three
  setups all ≈ 2.416), scatter 1.33× their quoted error, and converge up with budget
  (three 100k-event seeds −0.05%). Here, thirty seeds at 120k×10 read +1.25σ, five at
  480k×20 −0.3σ. A five-seed first cut read +3.5σ: its seeds were the high tail of this
  side's distribution and the reference was the low-biased one. Compare against the exact
  width where one exists, and read the seed spread, as
  [seed sweeps and budget ladders](../validation/seed-sweeps-and-budget-ladders.md) asks.
  Flat RAMBO is useless as a control here (seeds scatter 10% against a 0.6% quote).
- **Sample** (`t > b e+ ve`, 10k unit-weight events against MadEvent's 10k): m(e+ ve)
  χ²/dof 1.15 over 16 bins, m(b e+) 0.84 over 17, every bin within 4σ; `AQCDUP` equal to
  MadEvent's within 5e-7.

**VEGAS on a constant integrand.** A single-channel two-body width is a constant, yet grid
refinement follows the first iteration's sampling noise and the result reads ±0.03%
instead of exact; such a run should skip refinement
([backlog](../backlog/performance/vegas-refinement-noises-constant-integrand.md)).

[^n38-decays]: Note 38 §1.3: 1 → n processes as `ninitial = 1`, integrated to a partial width; no mixed initial counts.
[^n38-d1]: Note 38 §4, 1→n decay processes: enumeration, integrand, decay run card, event file, gates and findings.
[^mg-banner-decay]: `madgraph/various/banner.py` L4784 (`remove_all_cut()` for `ninitial == 1`) and L5045 (`sde_strategy`).
[^mg-setcuts]: `Template/LO/SubProcesses/setcuts.f` L137.
