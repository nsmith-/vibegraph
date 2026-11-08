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
* I-R2 checked (7ca93e8d on `sprinti-r2`; report recorded). The manager reran `validate_pdf_grid` (20 passed) and `cli_worker_panic` (2 passed) on it. MadGraph integrates the zero-floor card (4.227e9 pb behind an implicit √ŝ ≥ 2 GeV), so that item's premise was wrong; a madgraph-defects row goes in at close-out.
* I-R1 checked (8ff0d638, 59be16b4). The manager re-averaged its MadGraph and vibegraph σ files and read its validate.sh log tail (EXIT=0). I-R2 merged into `sprinti-runcard` with main (b2fb8de2). Post-merge: fmt, workspace clippy, lib tests (1290 passed) and `cli_worker_panic` (2 passed). PR 1 opened as #111 (head ac241ef7), with its five claims moved off #110 and the items' files deleted. `mxx_only_part_antipart` is described as consumed in `run-card/madgraph-cut-conventions.md` and `field-classification.md`.
* The container restarted. I-C2 had finished: the guards are on 83ec27d6 and 71b5302e, its report was recovered from its transcript, and its gates were green under option (b). Its pole probe shows `pp_to_epemjj_qcd0` diverges at tree level (|M|²·|t| constant over 8 decades of t); the ladder's plateau is the samplers' pole floor.
* **Decision D1** (the user, 2026-11-08): option (a), so the guard refuses the equivalent-photon limit and the row is retired in PR 2 ([D1](decisions/D1-ewk-lljj-refused.md)). I-C2 resumed to flip the guard, retire the row and re-gate.
* Disk: 8.5 GB free after the restart. The `sprinti-r1` target was removed, and the `sprinti-r2` worktree too (merged in #111).
* I-C1 stopped before writing code: the window rule is IR-divergent through an exempt identical quark's beam-collinear limit in the other pairings (report recorded; the manager checked the probe log). **Decision D2** (the user, 2026-11-08): keep today's stricter, IR-safe rule for v0.2 ([D2](decisions/D2-cut-decays-keep-the-stricter-cut.md)).
  - `cut-decays-identical-quarks-cut-a-stricter-observable` is rewritten toward the per-pairing route and released from the sprint.
  - The "kinematic IR-safe fix" claim is corrected in `identical-particles-across-decays.md`, `madgraph-defects.md` and `madgraph-compat-sites-unconditional`.
  - `release-scope.md` now lists the `cut_decays` deviation.
* I-C2's re-sweep under D1: 61 accepted, `pp_to_epemjj_qcd0` refused with `UncutInitialCollinear` (subprocess `u u > e+ e- u u`).
* The vibegraph manager reviewed #111 (head ac241ef7) with no blocking findings, and will merge on green CI. Its update-branch brought in main (#112). Points for the close-out:
  - `lhaid 247000` and `nn23lo1` are the same set (NNPDF23_lo_as_0130_qed), so I-R1's σ comparison is like for like.
  - `exit(101)` skips destructors, so a buffered writer is not flushed on a panic; say so in the hook's doc.
  - The hook could emit ldjson's final error object.
* I-R1 report recorded.
* I-C2 addendum checked (249dd1a0, b2f3365d; both gate logs end EXIT 0). PR 2 opened as #113 (head 4ef935b8) after merging main. Its kb changes: the two closed items deleted, references to the retired row updated, and `ewk-lljj-row-needs-a-regulated-rebank` filed.
* Release notes drafted (`docs/release-notes/v0.2.0.md`; `scripts/release-check.sh` accepts v0.2.0-rc1 and v0.2.0). I-Z1's Found on the `release.yml` comment is moot: the comment already names `ring`.
* The user asked to run `validate-deep` and the CERN rerun in parallel, once the manager has merged #111 and #113. A synthetic RC branch (`sprinti-rc`) and its run were started, then withdrawn on the user's follow-up (wait for the merges); nothing ran past the build.
* The vibegraph manager reviewed #113 with one blocking finding: the guard ignores the per-event xqcut veto (`setclscales.rs:347-351`, MadEvent's `reweight.f:1063-1077`), so it refuses matched cards with `auto_ptj_mjj = F` that are regulated. I-C2's claim that the kT clustering cut is not implemented was wrong. I-C2 resumed to fix it. The vibegraph manager is running a refusal-only CMS census on #113's head on the CERN host; re-review waits for it.
* #111 merged as 2803c0e9, all checks green. I-C2 told to merge main into `sprinti-cuts` and re-gate on the merged head.
* **Decision D3** (the user, relayed by the vibegraph manager, 2026-11-08; documentation only in v0.2). Identical particles across decays move behind `--madgraph-compat`, which defaults to off for every site.
  - By default, every pairing is kept coherently and `cut_decays = F` becomes a pairing-weighted cut.
  - The banked MadGraph gates run with the flag on.
  - The D2 wording on H-C3 Found 2 is narrowed: it is unsound only for exemptions on the full |M|², and it is the right guard under the weighted cut.
  - Updated: the compat-sites item (its `closes_when` is now default off; site (d)), the `cut_decays` item (now the weighted-cut item), `release-scope.md`, `identical-particles-across-decays.md`, the `madgraph-defects` row and the release notes.
