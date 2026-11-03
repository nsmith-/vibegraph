---
type: Session Report
title: "F2 report: the 1e-4 α floor is on by default and moves only the rows it raises"
description: "main merged (artifact format stays 15). floored_alphas returns None when no weight lies below the floor, so a survey that keeps every channel above it installs its weights bit for bit; the floor 1e-4 is the default in both multichannel integrands, the CLI (--alpha-floor, `off` to disable) and the probes, and is recorded in the integrate artifact. Against the split-only tree, the banked validate pair moves 6 cells (gg_to_gg, gg_to_gg_cg, uux_to_uux; integrals and samples; all pass → pass, at most 3.4e-5 per integral seed). F1's 41 ulp moves vanish. The MLM σ gate moves the 3 pp_to_ttx_0j1j_mlm cells (pass → pass); the four other MLM rows are bit-identical."
status: draft
tags: [multichannel, alpha-survey, phase-space, heavy-tail, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-31}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-30/31, one long job at a time"
  commit: "change fea0dfc (tree 00bea3af); base 567c02c (tree a957dc24), the merge of 4cfe188 with origin/main 3fbb177, floor off. Both validate runs on a clean tree (git diff sha256 e3b0c442…). Change binaries: validate_sigma sha256 d6531386…, validate_unweighting 248fa1f8…, validate_hadronic acbf3940…; the base binaries were rebuilt over in the same target/ and are named by commit and tree only."
  command: "see Method; logs, report trees and scripts under /home/user/scratch/ast-F2/, prefixed ast-F2-"
sources:
  - {id: brief, resource: "F2.md", title: "F2 brief"}
  - {id: f1, resource: "F1-report.md", title: "F1 report"}
---
# F2 report

## What changed

1. **Merge.** `origin/main` (3fbb177) merged as `567c02c`. Main had not
   bumped the integrate artifact's format, so it stays at 15 and now carries
   both the pair order and the α floor.
2. **No renormalisation when nothing is raised.** `floored_alphas` returns
   `Option<Vec<F>>`: `None` when no weight is below the floor, and callers keep
   the vector they have. The test
   `phasespace::channel::tests::a_floor_renormalises_only_when_it_raises_a_weight`
   pins this. A second test,
   `hadronic::tests::an_alpha_floor_moves_nothing_unless_it_raises_a_weight`,
   checks it end to end: on a real survey (`e+ e- > ta+ ta- h`), a floor below
   the lowest α leaves the installed weights and the whole trajectory
   bit-identical to no floor. A floor above the lowest α installs exactly the
   `floored_alphas` set.

   Mutation check: with the early return removed, both tests fail. The second
   fails by one ulp in each weight (`…535` vs `…534`), which is F1's ulp move.
3. **Default on.** `DEFAULT_ALPHA_FLOOR = 1e-4` (`phasespace::channel`) is now
   the starting value in `FixedBeamIntegrand` and `ProtonIntegrand`.
   `MultiplicitySum::set_alpha_floor` / `alpha_floor` apply one floor to every
   part. Other entry points:
   - CLI: `vibegraph integrate --alpha-floor <FLOOR>`, default `1e-4`; `off`
     or `0` turns it off (`parse_alpha_floor`).
   - Probes: `VG_2TO6_ALPHA_FLOOR` and `VG_TAIL_ALPHA_FLOOR` accept `off`. When
     unset they take the default. A tail-probe trace is still installed as
     recorded.
   - `validate_sigma`'s `with_integrand` passes the default.

   **Recorded in the artifact:** `IntegrateArtifact::alpha_floor:
   Option<f64>`, because the map choices and the grid rule are recorded there
   too. It is provenance only, since replay reads the installed
   `ChannelGrid::alpha`. A flat-RAMBO fixed-energy run records `None` because
   it surveys nothing.

   Docs updated: `set_alpha_floor`, `use_multichannel`, `adapt_alphas`,
   `MultiplicitySum::adapt_alphas`, the artifact's `FORMAT_VERSION` note, and
   `docs/src/cli/reference.md` (regenerated with `scripts/gen-cli-docs.sh`).

**Deviation:** when the floor raises a weight, the floored set now replaces
the trajectory's last entry instead of being appended. That last entry is the
post-step weights, which no survey drew with. This keeps the trajectory's
length independent of the floor, and the earlier entries are still the
weights each survey drew with (pinned in the hadronic test above).

**Classification:** statistical, with pinned seeds. Where no α is below 1e-4
the change preserves order and is bit-for-bit. Where α values are raised,
the sampling density changes, so those rows gate on σ/pull at their existing
seeds.

## Gate

```text
$ /home/user/scratch/ast-F2/ast-F2-gates.sh     (log ast-F2-gates.log, tree = fea0dfc's content, 2026-10-31T00:45Z)
pixi run cargo fmt --all --check                                          --> exit 0
pixi run cargo clippy --workspace --all-targets -- -D warnings            --> exit 0
pixi run cargo clippy … --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings --> exit 0
pixi run cargo test --workspace --no-fail-fast                            --> exit 0  (46 result lines, 1743 passed, 0 failed, 20 ignored)
pixi run --skip-deps kb-lint                                              --> exit 0  (1013 concepts, 332 backlog items; 0 errors, 4 warnings, all pre-existing backlog `blocked_by`)
```

## Step 4: the banked validate pair

```text
ast-F2-validate-splitonly.log  567c02c a957dc24  VALIDATE EXIT 0  "76 rows × 4 categories = 304 cells: 218 measured … (213 ✅, 5 ⚠️, 13 ⏳, 73 —)"
ast-F2-validate-floor.log      fea0dfc 00bea3af  VALIDATE EXIT 0  "76 rows × 4 categories = 304 cells: 218 measured … (213 ✅, 5 ⚠️, 13 ⏳, 73 —)"
$ python3 -I ast-F1-scripts/cells.py ast-F2-report-splitonly ast-F2-report-floor
integrals/gg_to_gg:    pass -> pass | pull +1.7642 -> +1.7578 | sigma -2.86e-06 | chi2/dof 1.65 -> 1.66 | seeds moved 5/5, max |dσ/σ| 1.2e-05
integrals/gg_to_gg_cg: pass -> pass | pull +0.6784 -> +0.6790 | sigma +4.44e-06 | chi2/dof 0.97 -> 1.00 | seeds moved 5/5, max |dσ/σ| 3.4e-05
integrals/uux_to_uux:  pass -> pass | pull +0.4782 -> +0.4841 | sigma +3.07e-06 | chi2/dof 1.45 -> 1.44 | seeds moved 5/5, max |dσ/σ| 1.8e-05
samples/gg_to_gg:      pass -> pass | min_ks_p 0.1111 -> 0.1042, min_chi2_p 0.1401 -> 0.1294 | seeds moved 3/3, max 3.9e-04
samples/gg_to_gg_cg:   pass -> pass | min_ks_p 0.01609 -> 0.01570, min_chi2_p 0.03804 -> 0.03884 | seeds moved 3/3, max 3.2e-04
samples/uux_to_uux:    pass -> pass | min_ks_p 0.2986 -> 0.3041, min_chi2_p 0.3655 -> 0.3712 | seeds moved 3/3, max 1.8e-04
cells identical 221, moved 6
```

- **The 41 ulp moves vanish.** Exactly 6 cells move, and they are F1's three
  real movers with the same numbers as F1's forced-on arm, to the digits
  printed. Every cell whose survey leaves all α above 1e-4 is bit-identical
  to the split-only tree.
- The three moved integrals rows already run 5 seeds each and the samples
  rows 3. Every seed moves by ≤3.4e-5 (integrals). Pulls and χ²/dof stay
  where they were, well inside the gates. No enforced cell needs more seeds.
- **Determinism control:** the split-only run on refdata-11 and on refdata-12
  (same tree) agree on all 225 shared cells (`cells identical 225, moved
  0`). So a moved cell in the pair is the floor, not run-to-run noise.

## Step 5: long-tier rows the default touches

Because of step 2, a row whose survey leaves every α above 1e-4 is
bit-identical. A moved cell at pinned seeds therefore means the floor raised
something in that row.

- **MLM σ gate (run before and after):** both runs report `test result: ok.
  5 passed; 0 failed`. The before run took 1766 s (`ast-F2-mlm-splitonly.log`,
  567c02c) and the after run 2367 s (`ast-F2-mlm-floor.log`, fea0dfc).
  `cells.py` over the two report trees: `cells identical 228, moved 9`. Six of
  the moved cells are the banked cells above. The other three are MLM cells:

  ```text
  integrals/pp_to_ttx_0j1j_mlm__at0:   pass -> pass | pull +0.2835 -> +0.3099 | σ +1.32e-05 | χ²/dof 0.86 -> 0.85 | seeds moved 10/10, max 1.2e-04
  integrals/pp_to_ttx_0j1j_mlm__at1:   pass -> pass | pull +0.7058 -> +0.7302 | σ +4.88e-05 | χ²/dof 0.60 -> 0.47 | seeds moved 10/10, max 4.3e-04
  integrals/pp_to_ttx_0j1j_mlm__total: pass -> pass | pull +0.7531 -> +0.7827 | σ +3.21e-05 | χ²/dof 0.77 -> 0.64 | seeds moved 10/10, max 2.1e-04
  ```

  `pp_to_llj_mlm`, `pp_to_llj_mlm_alps2`, `pp_to_llj_xqcut_only` and the four
  `pp_to_ll_0j2j_mlm` cells are bit-identical. For example, `pp_to_llj_mlm`
  seed 20260957 gives `2.684576e2 ± 3.623e-1` in both runs.
- **2 → 6 rows (`validate-sigma-2to6`, `ladder-2to6`):** touched heavily. In
  F1's traces, 511–566 of 615 channels are below 1e-4 per seed on bbx, and
  485–511 of 579 on uux. Not rerun here: the bbx ladder is out of scope, and
  F1's split + floor arm is the measurement of the new default.
- **`ladder-bb`, `ladder-recarded`, `validate-mlm-samples`,
  `validate-generate-proton`, `validate-reweight`'s proton CLI test:** these
  go through `ProtonIntegrand` or the CLI and so now run under the default
  floor. Whether any of their surveys leave an α below 1e-4 is not measured.
  This is listed under Found.

## Concepts the change makes wrong (for the manager)

- `research/kb/phase-space/multichannel.md`, § α-adaptation: "After the
  survey `α` is frozen". The survey's α is now raised to 1e-4 where below and
  renormalised, then frozen. The `1e-12`-of-the-sum floor inside
  `kleiss_pittau_step` is a different floor and is unchanged.
- `research/kb/phase-space/channel-budget-allocation.md` and
  `channel-set.md`: worth checking for statements that installed α is the
  survey's output as surveyed. A grep turned up no explicit mention of the
  floor.

## Method notes

- **Reference data:** the worktree had refdata-11, and main's #82
  (`validate_param_card_sigma`) needs refdata-12's seed records. The first
  split-only run exited 101 on that test
  (`ast-F2-validate-splitonly-refdata11.log`).
  - Comparing the two bundles' `tar -tv` listings showed refdata-12 =
    refdata-11 + 487 new members, with none removed or changed. I installed
    those 487 members into `validation/madgraph/output` (`.lhe` → `gzip -n`,
    as `vg_ensure_refdata` does) and set the stamp to refdata-12's sha
    (`04fd869b…`).
  - Both reported runs are on refdata-12.
  - An earlier MLM-before attempt died of ENOSPC during my trial unpack
    (`ast-F2-mlm-splitonly-enospc.log`). That log is not a result.
- **Container restart:** a restart around 00:40Z interrupted the first gate
  run during `cargo test`, after fmt and both clippys had passed
  (`ast-F2-gates-interrupted.log`). It was rerun in full on the same diff
  (sha `a3d86090…`) before the commit.
- **Scripts:** `ast-F2-validate.sh <tag> <dir>`, `ast-F2-mlm.sh <tag>`,
  `ast-F2-pair-base.sh`, `ast-F2-pair-floor.sh`, `ast-F2-gates.sh`,
  `ast-F1-scripts/cells.py`. All are under `/home/user/scratch/ast-F2/`. The
  first-half logs were copied there read-only from the retired session
  scratchpad.

## Found

1. **Long-tier rows run under the default floor and are not measured.**
   `ladder-bb`, `ladder-recarded`, `validate-mlm-samples`,
   `validate-generate-proton` and the proton CLI reweight test now run with
   α floor 1e-4. Whether any of their surveys has α below 1e-4, and so
   whether their banked budgets and ladders still hold, is unmeasured. To
   close: one run of each before and after (or `--alpha-floor off` against
   the default), diffing cells. With step 2, bit-identity is the oracle for
   "not touched".
2. **The brief's base needed refdata-12.** The brief's base (4cfe188 merged
   with main) cannot pass `validate` on the refdata-11 bundle a worktree
   COW-copied before #82 carries. Worktree setup for later sessions should
   copy or fetch refdata-12.
