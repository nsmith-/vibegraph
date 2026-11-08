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
