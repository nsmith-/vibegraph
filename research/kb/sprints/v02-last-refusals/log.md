# v02-last-refusals log

## 2026-11-08
* **Creation**: Sprint folder created by the sprint-I manager session, started by the vibegraph manager session on the user's instruction.
* Reference data in the container: refdata-13, NNPDF23_lo_as_0130_qed (`validation/pdf/fetch.sh`), and the `mg5amcnlo` submodule at b7687064. gfortran installed, so local MadGraph runs here (`nn23lo1`).
* User decisions carried in by the brief: the mixed-multiplicity default matches MadGraph; the `cut_decays` window rule comes into v0.2; the CERN host establishes the spacelike-floor reference first.
* Plan posted (sprint.md, sessions I-R1, I-R2, I-C2, I-C1, I-S1). Three PRs: run-card (`sprinti-runcard`), cuts (`sprinti-cuts`), close-out (`sprinti-plan`).
* Corrections to the items found while planning: `create_default_for_process`'s matching rule also sets `drjj = 0`, `sys_alpsfact` and `dynamical_scale_choice = -1`, and its beam rules cover `e p`, neutrino and EVA beams; `setcuts.f:808` in the pinned source is the grouping trap, not the τ-floor check; the CERN host's spacelike reference was requested by the vibegraph manager.
* `long-tier-gates-never-run-in-ci` is not claimed: its `closes_when` needs a CI workflow, and this sprint runs `validate-deep` by hand.
