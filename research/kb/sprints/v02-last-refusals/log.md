# v02-last-refusals log

## 2026-11-08
* **Creation**: Sprint folder created by the sprint-I manager session, started by the vibegraph manager session on the user's instruction.
* Reference data in the container: refdata-13, NNPDF23_lo_as_0130_qed (`validation/pdf/fetch.sh`), and the `mg5amcnlo` submodule at b7687064. gfortran installed, so local MadGraph runs here (`nn23lo1`).
* User decisions carried in by the brief: the mixed-multiplicity default matches MadGraph; the `cut_decays` window rule comes into v0.2; the CERN host establishes the spacelike-floor reference first.
* Plan posted (sprint.md, sessions I-R1, I-R2, I-C2, I-C1, I-S1). Three PRs: run-card (`sprinti-runcard`), cuts (`sprinti-cuts`), close-out (`sprinti-plan`).
* Corrections to the items found while planning: `create_default_for_process`'s matching rule also sets `drjj = 0`, `sys_alpsfact` and `dynamical_scale_choice = -1`, and its beam rules cover `e p`, neutrino and EVA beams; `setcuts.f:808` in the pinned source is the grouping trap, not the τ-floor check; the CERN host's spacelike reference was requested by the vibegraph manager.
* `long-tier-gates-never-run-in-ci` is not claimed: its `closes_when` needs a CI workflow, and this sprint runs `validate-deep` by hand.
* The vibegraph manager relayed, on the user's instruction:
  - The CERN host started the spacelike reference: `import model loop_sm-ckm_no_b_mass`, `generate p p > a j j QCD=0 QED=3 $ t t~`, with the GJ-2Jets-EWK_Bin-PTG-40 gridpack's param card and run card (lhaid 315200, draj 0.4, ptj 0.1 or 1.0; pta 40, mmjj 4, drjj 0, etaj/etaa −1, dynamical_scale_choice −1 unchanged). vibegraph runs at neval 20000, niter 3, seeds 1–5 on d87ad3ae, stock and with `VG_DIAG_FLOOR=min` (the minimum over legs with a single-leg cut, not a per-rung bound). MadGraph runs two iseeds per card (about ±0.3%). The item's earlier floor=min numbers can't be re-checked and are superseded.
  - PR 3 gains three release fixes: `build.rs` describes `v*` tags only; tags with a hyphen publish as prereleases; the notes come from a committed file, and the publish job fails without it. Each is to be proven by a manual `release.yml` run, without pushing a tag. This answers the release-notes question.
  - The vibegraph manager reviews and merges PRs 1 and 2; the user reviews PR 3.
* I-R1 and I-R2 dispatched (feature-dev, Opus) on `sprinti-runcard` and `sprinti-r2`. I-Z1 dispatched (feature-dev, Sonnet) on `sprinti-release`, with the version bump folded in.
* `send_message` to the vibegraph manager fails (`elevated_session_untrusted_device`). Messages go through the user.
* The vibegraph manager relayed the CERN host's spacelike reference, on `p p > a j j QCD=0 QED=3 $ t t~` (loop_sm-ckm_no_b_mass, GJ-2Jets-EWK_Bin-PTG-40, lhaid 315200, pta 40, draj 0.4). MadGraph 3.7.1 gives 54.27 ± 0.11 pb at ptj 0.1 and 53.48 ± 0.12 pb at ptj 1. vibegraph's ratio to it over 5 seeds:

  | Build | ptj 0.1 | ptj 1 |
  |---|---|---|
  | stock, neval 2e5 | 1.000 ± 0.006 (χ²/dof 2.05) | 0.992 ± 0.006 |
  | smallest-threshold floor, neval 2e5 | 0.995 ± 0.003 | 0.995 ± 0.002 |
  | stock, neval 2e4 | 0.995 ± 0.024 (χ²/dof 3.9) | 0.973 ± 0.007 (χ²/dof 0.45) |

  The largest-threshold floor removes no visible σ at convergence. It costs variance: stock's per-seed errors are 2–3× larger and heavy-tailed. Sprint H's +6% was a low-budget sampling deficit. The manager recommends dropping the item from sprint I and asked to pause it until the user decides. I-S1 had not been dispatched; it is on hold.
* The user agreed, relayed by the vibegraph manager: the spacelike floor is out of sprint I and v0.2. The item becomes `performance/spacelike-floor-largest-threshold-inflates-seed-spread` (medium), gated on seed spread at fixed neval, and moves on the vibegraph manager's kb branch. Its `Backlog:` line is dropped from #110, and I-S1 is withdrawn. PR 2 is the `cut_decays` window rule plus the collinear guards.
* I-R2's validate.sh exited 101: 13 failures, all in `validate_pdf_grid`, because NNPDF31_lo_as_0130 (`fetch-pdf-multigrid`, a dependency of `pixi run validate`) was not fetched at provisioning. A manager error: the provisioning list in the brief names `validation/pdf/fetch.sh` without the set argument. The set was fetched and copied into every worktree; I-R2 reruns that target.
