---
type: Sprint
title: "Sprint I: v0.2 last refusals and fixes"
description: "Close every remaining card surface that panics or answers silently wrong before v0.2.0: run-card defaults and refusals, the cut_decays window rule, uncut collinear limits; then the release preparation."
status: draft
active: true
tags: [release, v0.2, run-card, defaults, cuts, ir-safety, decay-chains, phase-space, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: scope, resource: "../../pipeline/release-scope.md", title: "Release scope and the hard-error rule"}
  - {id: h, resource: "../cms-gridpack-coverage/closeout.md", title: "Sprint H record and its Lessons"}
  - {id: d5, resource: "../cms-gridpack-coverage/decisions/D5-cut-decays-exemption-and-b-cut-cards.md", title: "D5: the cut_decays deviation and the window rule as the later fix"}
---
## Goal and exit criteria

This is the last sprint before v0.2.0. It closes every remaining item where a
card reaches something vibegraph does not honour and gets a panic or a silent
wrong answer ([the hard-error rule](../../pipeline/release-scope.md)). It is
done when:

- every claimed item's `closes_when` holds, each with the MadGraph oracle named
  below, measured (not inferred) on the PR's head;
- every banked σ line is unchanged (bit for bit where the change cannot reach
  it, within error where it can);
- on the release-candidate head, `validate-deep` has run and its rows are
  recorded, and the CERN host's CMS campaign reads at least 66 ok and no panics
  against `main-202a540-200-pc`.

The user asked for as few PRs as possible: three.

Out of scope: one-body phase space, Majorana fermions, the coloured-scalar
seagull, `import model` options, coupling-order inference, the sampler seams
(#78), the multi-jet timeouts.

## PRs, items and oracles

### PR 1: run-card defaults and refusals (`sprinti-runcard`)

| Item | Pri | Oracle |
|---|---|---|
| `absent-run-card-mixed-multiplicity-unmatched` | high | The `run_card.dat` local MadGraph's `output` writes for `p p > e+ e-` + `add process p p > e+ e- j` (diffed field by field against `RunCard::scattering_default`), and a σ row with no run card against local MadGraph `launch`, ≥ 5 seeds a side. |
| `absent-run-card-lepton-beams-panic` | high | The card MadGraph writes for `e+ e- > mu+ mu-` (and for `e- p > e- j`, `ve p > e- j`, `w+ w- > h`: each must become MadGraph's beams and then the existing named refusal, never a panic); a refusal test for an untabulated initial parton on an explicit `lpp = 1` card. |
| `zero-shat-floor-panics-with-nan-x` | medium | MadGraph's own check, quoted with its line (local MadGraph `launch` of `p p > b b~` under `sm-no_b_mass` at `ptb = 0` shows what it does); a typed refusal test on that card. |
| `mxx-only-part-antipart-silently-ignored` | medium | Implement as MadGraph does if cheap: its keys enter `pdg_cut` with the per-PDG defaults (`banner.py:4708-4750`), and `setcuts.f:318-334` overwrites those legs' class cuts. Oracle: σ on a finite card (e.g. `p p > e+ e-` with an `mmll` window and `{'default': False, 11: True}`) against local MadGraph at ≥ 5 seeds. If that is not cheap, refuse it off default as `IgnoredPhysics`. |
| `rayon-worker-panic-leaves-process-alive` | medium | A CLI test with a test-only panic hook in a parallel integration worker: exit within seconds, non-zero, the message on stderr, in text and ldjson modes; the 40-minute case explained from the code. |

### PR 2: cuts (`sprinti-cuts`)

| Item | Pri | Oracle |
|---|---|---|
| `uncut-collinear-limits-beyond-photon-quark-unrefused` | high | One refusal test per class (`q → q g`/`g → g g` at `ptj = 0`, `drjj = 0`, no `xqcut`; `ℓ → ℓ γ` at `dral = 0`; a parton above `maxjetflavor`), each failing before the change; every banked card still accepted and every banked σ line unchanged. |
| `cut-decays-identical-quarks-cut-a-stricter-observable` | high | User decision: bring the window rule into v0.2. σ on `g g > z u u~, z > u u~` (MadGraph 7.211 ± 0.028 pb, one seed) and `g g > z j j, z > j j` (MadGraph 150.61, 5 seeds; vibegraph −6.8%) against local MadGraph at ≥ 5 seeds a side; bit for bit on cards with no identical particles across decays; a test that a soft quark is never exempted. |
| `ir-unsafe-exemption-guard-sees-coloured-legs-only` | medium | Restated over the exempting pairing (H-C3 Found 1 and 2), with a test that exempts a massless colourless core leg and is refused. |

### PR 3: close-out and release preparation (`sprinti-plan`, this PR)

Session [I-Z1](sessions/I-Z1.md) (the user's instruction, 2026-11-08):
`build.rs` describes `v*` tags only; tags with a hyphen publish as
prereleases; the release notes come from a committed file, and publishing fails
without one; the workspace version 0.1.0 → 0.2.0. Then the sprint record; the release notes
draft with their known issues; `validate-deep` on the release-candidate head,
recorded; the CERN host's CMS campaign rerun (bar: ≥ 66 ok, no panics, against
`main-202a540-200-pc`).

## Sessions

| Session | Agent | PR | Depends on | Work |
|---|---|---|---|---|
| [I-R1](sessions/I-R1.md) | feature-dev | 1 | – | The absent-card defaults (both `create_default_for_process` items) and the untabulated-parton refusal. |
| [I-R2](sessions/I-R2.md) | feature-dev | 1 | – | The zero τ floor refusal, `mxx_only_part_antipart`, the worker-panic exit. |
| [I-C2](sessions/I-C2.md) | validation-dev | 2 | – | The uncut-collinear guards. |
| [I-S1](sessions/I-S1.md) | – | – | withdrawn | The spacelike floor left the sprint: the host's reference shows no σ bias at convergence, only seed spread (log, 2026-11-08). |
| [I-C1](sessions/I-C1.md) | validation-dev | 2 | I-C2 | The `cut_decays` window rule and the restated exemption guard. |
| [I-Z1](sessions/I-Z1.md) | feature-dev (Sonnet) | 3 | – | Release tooling: `v*` tags, prereleases, committed notes, the version bump. |
| I-Z | manager | 3 | PRs 1–2 merged | Release notes content, `validate-deep`, the CMS rerun request, the record. |

I-R1 and I-R2 run in parallel (light builds). I-C2 and I-S1's derivation run
next; I-C1 follows I-C2 because both rewrite the guards in `cuts.rs`. Heavy
runs (seed sweeps, `validate.sh`, local MadGraph) go one at a time on this
4-core container; the manager serialises them.

## What is known before the sprint (manager, 2026-11-08)

- **`create_default_for_process` (`banner.py:4767-5060`) changes more than the
  items name.** The matching rule (`:4924-4966`) fires when some
  maximal-multiplicity process exceeds a minimal one by jets only (`j` legs
  `1..5, 21`), and then sets `ickkw = 1`, `xqcut = 30`, `drjj = 0`, `drjl = 0`,
  `sys_alpsfact = "0.5 1 2"`, appends `--alps=0.5,1,2` to
  `systematics_arguments`, and `dynamical_scale_choice = -1`. A model with
  `MLM` in its limitations turns it back to `ickkw = 0` (`:5045`). The beam
  rules set `lpp = 0`, `ebeam = 500`, `use_syst = F` for lepton beams; `e p`
  gives `(0, 1)` with `1k`/`6500`; neutrino beams set `polbeam ±100`; `W`/`Z`
  beams give EVA (`lpp = ±3`, `nhel = 1`). `sde_strategy` is set per process
  too (`:4990-5060`). Each changed field is reproduced, or the card it writes is
  refused by the existing rule that would refuse it if written by hand.
- **`mxx_only_part_antipart`.** `banner.py:4708-4711` adds its keys to
  `pdg_cut`; H-C1 found the light-code guard at `banner.py:4722` never fires,
  so `{22: True}` does reach `setcuts.f:318-334` for photons.
- **References.** Local MadGraph runs here (the pinned submodule's
  `bin/mg5_aMC`; gfortran installed; no LHAPDF, so `nn23lo1`, a +0.32% offset
  against LHAPDF 247000 on Drell-Yan, sprint H's H-D2). Only references for the
  spacelike floor (since withdrawn) and the CMS campaign come from the CERN host.
- **`long-tier-gates-never-run-in-ci` is not claimed.** Its `closes_when` asks
  for a CI workflow; this sprint runs `validate-deep` by hand on the
  release-candidate head and records it.
