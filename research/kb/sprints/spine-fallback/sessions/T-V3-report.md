---
type: Session Report
title: "T-V3 report: a planned no-pT-cut EWK e+ e- j j σ row, its gate and the M3 request; the MLM re-gates pass"
description: "Row pp_to_epemjj_qcd0 (p p > e+ e- j j QCD=0 QED=4 $ h, sm-no_b_mass, ptj = ptl = 0, mmjj = 4, mmll = 50) is planned: script, run card, manifest row, a `card` kind and a dry run in bank_seeded_sigma.sh, and a long-tier σ gate that prints AWAITING THE BANK. vibegraph's side at the gate budget (40000 × 12) reads 3.5804 ± 0.0159 pb over the gate's five seeds and 3.5760 ± 0.0085 over ten; between 12 and 24 iterations the mean moves +0.04% ± 0.4%. The two MLM rows T-F2's maps moved pass their σ re-gates on the merged tree."
status: draft
tags: [validation, spine, vbf, seeded-sigma, mlm, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
The session brief is [T-V3](T-V3.md). Worktree `/home/user/wt/T`, branch
`sprinte-spine-fallback`. Commits: `ac3e517b` (the row, the driver support,
the gate), `80a4c718` (the merge of `origin/sprinte-spine-fallback`, which
brought #82 from main), and the commit adding this report and the row's
measured note. Nothing is pushed.

Every cargo command ran with `CARGO_INCREMENTAL=0`, debuginfo off
(`CARGO_PROFILE_{DEV,TEST}_DEBUG=0`, `--config 'profile.release-debug.debug=0'`)
and `-j 2`, except the gate run itself, which uses the gate's own command
line. The container restarted at about 17:46 UTC and killed the runs in
flight; they were relaunched from resumable drivers on the merged tree (see
Incidents). The host was shared with sibling session S until about 19:30.

## 1. The row

### Process and model

```
import model sm-no_b_mass
generate p p > e+ e- j j QCD=0 QED=4 $ h
```

- **From the CMS card** (`DYto2L-2Jets-EWK_Bin-MLL-50/proc_card.dat`): the
  process line, `QCD=0 QED=4`, and its diagram exclusion. That card uses
  `$ h`; the two VBF cards use `$$ h` (see Corrections).
- **Dropped:** the three lepton flavours (`ell = e mu ta`, here `e` only),
  `loop_sm-ckm_no_b_mass` and its shipped param card. `sm-no_b_mass` gives a
  five-flavour `p` and `j` (MB = ymb = 0), a diagonal CKM and MadGraph's
  default card for that restriction, which vibegraph's built-in model carries
  (`sm-no_b_mass`, digest `75bfe0af…`). `sm` alone would make `j` four-flavour,
  unlike the CMS card.
- **The exclusion is vacuous here, by census.** vibegraph's enumeration of
  the process gives the same 87 subprocesses and 2424 diagrams with `$ h`,
  `$$ h`, `/ h` or no exclusion. A `/ a` control removes 1740 of them, so the
  `/` syntax is applied. At `--neval 1000 --niter 1 --seed 1` the `$ h`,
  `/ h` and unexcluded runs read the same σ, `3.673609 ± 0.550686 pb`
  (`T-V3-probe-*.log`, `T-V3-census-*.txt`). With massless electrons and
  quarks and ymb = 0, no tree diagram of this final state has a Higgs line.
  The census is vibegraph's own; MadGraph's is not taken here (MadGraph does
  not run on this host).
- **Channels:** 68 (merged over subprocess × diagram), against the CMS card's
  142.

### Run card

`validation/madgraph/pp_to_epemjj_qcd0_run_card.dat`, derived from the
committed `pp_to_ll_0j2j_mlm_run_card.dat` (MadGraph 3.7.1's full card,
13 TeV, `lhapdf` 247000, `dynamical_scale_choice = -1`, `fixed_ren_scale =
fixed_fac_scale = False`, `mmll = 50`, `drjj = drjl = 0`, `vector_size = 1`,
`systematics_program = none`). Every field that differs from it:

| field | base card (`pp_to_ll_0j2j_mlm`) | this row |
|---|---|---|
| `ickkw` | 1 | 0 |
| `xqcut` | 20 | 0 |
| `auto_ptj_mjj` | True | False |
| `asrwgtflavor` | 4 | 5 |
| `ptj` | 20.0 | 0.0 |
| `ptl` | 10.0 | 0.0 |
| `etaj` | 5.0 | -1.0 |
| `etal` | 2.5 | -1.0 |
| `drll` | 0.4 | 0.0 |
| `mmjj` | 0.0 | 4.0 |
| `maxjetflavor` | 4 | 5 |

`ickkw`, `xqcut`, `auto_ptj_mjj` and `asrwgtflavor` turn matching off (the
last two are inert without it; set as the CMS card has them). `maxjetflavor
= 5` makes `b` a jet, so `mmjj` reaches `b` pairs as in the CMS card.
`nevents = 10000` (unchanged) is the bank's budget.

### Banking inputs

- `validation/madgraph/scripts/pp_to_epemjj_qcd0.mg5`: the process plus a
  launch block of 23 `set` lines that `mes_check_card` checks against the
  run card. It carries `# built-by: …bank_seeded_sigma.sh`, so `build.sh`
  skips it and `build-diagrams` never launches it.
- `bank_seeded_sigma.sh`:
  - a `card` kind: a script with a committed `<row>_run_card.dat` and no
    banked run of its own, generated without its launch block on that card
    and the model's default param card;
  - an `@ewk-lljj` group, included in `@all`;
  - `DRY_RUN=1`. #82 brought its own dry run; the merge keeps #82's param-card
    checks and its dy13 param-card diff, and adds the budget line (both
    rungs' nevents and seeds) and the launch-block line.
- Manifest row `pp_to_epemjj_qcd0`, `status = "planned"`, with the rationale,
  `integrals` at tier `long`, mode `gate`, and this side's measurement in its
  note.

**Driver checks.** `bash -n validation/madgraph/bank_seeded_sigma.sh` passes
on the merged tree. The dry run on the merged tree
(`SEEDED_WORK=… SEEDED_OUT=… DRY_RUN=1 ROWS="pp_to_epemjj_qcd0
dy13_mmll_60_120_mz95" bash validation/madgraph/bank_seeded_sigma.sh`,
exit 0; the script's comment header, which the dry run also prints, is cut
here):

```
>>> dry run: 2 row(s), seeds [20261101 20261102 20261103 20261104 20261105] at the budget and [20261111 20261112] at 4x
=== pp_to_epemjj_qcd0
    kind       card
    run card   /home/user/wt/T/validation/madgraph/pp_to_epemjj_qcd0_run_card.dat
    param card none, the template default
    tree       pinned
    generator  scripts/pp_to_epemjj_qcd0.mg5 without its launch block; run card pp_to_epemjj_qcd0_run_card.dat; the model's default param card
    generation script:
        import model sm-no_b_mass
        generate p p > e+ e- j j QCD=0 QED=4 $ h
        output <scratch>/T-V3-dry2/work/pp_to_epemjj_qcd0/template -nojpeg
    budget     nevents 10000 at [20261101 20261102 20261103 20261104 20261105], 40000 at [20261111 20261112]
    launch-block check  23 settings agree with the run card
=== dy13_mmll_60_120_mz95
    kind       dy13
    run card   /home/user/wt/T/validation/madgraph/dy13_mmll_run_card.dat
    param card /home/user/wt/T/validation/madgraph/dy13_mz95_param_card.dat
    tree       pinned
    generator  generate p p > e+ e- (gen_hadronic_sigma.sh); run card dy13_mmll_run_card.dat; param card dy13_mz95_param_card.dat
    generation script:
        generate p p > e+ e-
        output <scratch>/T-V3-dry2/work/dy13_mmll_60_120_mz95/template -nojpeg
    budget     nevents 200000 at [20261101 20261102 20261103 20261104 20261105], 800000 at [20261111 20261112]
    param card entries that differ from dy13_param_card.dat:
           23 9.500000e+01 # MZ 
```

The pre-merge dry run (`T-V3-dryrun.log`) also ran the `mlm` and `script`
controls `pp_to_ll_0j2j_mlm` and `ud_to_epemud_qcd0`; both resolved to their
old kinds and cards. **Control:** with the row's run card copied and `mmjj`
set to 0, `mes_check_card` refuses it (`run card disagrees with the script
launch block: mmjj: script 4, card 0.0`, rc 1); on the committed card it
passes (rc 0).

### The gate

`vibegraph-cli/tests/validate_ewk_lljj_sigma.rs`, test
`ewk_epemjj_without_pt_cuts_matches_madevents_seeds`: long tier, `#[ignore]`,
pixi task `validate-ewk-lljj-sigma`, registered with `required-features =
["extended-validation"]`. This is where the `extended-validation` skill and
the manifest's layer notes put a gate too heavy for the banked budget: an
ignored test plus a task that passes `--ignored`.

- It reads the process and model from the row's `.mg5` script (every
  non-comment line before `output`), so the two sides cannot name different
  processes, and runs `vibegraph integrate` on the committed run card, five
  seeds (20261031–35) at `--fixed-budget --neval 40000 --niter 12`.
- Both sides are read under the seed policy (`common/seeds.rs`). It holds
  when `|pull| < 3`, `|rel| < 0.02`, and this side's seed χ²/dof lies inside
  (0.023, 4.617), the 0.1% and 99.9% quantiles on four degrees of freedom
  (the band the five-seed llj MLM gates use).
- With the row planned and no record, it integrates this side, prints
  `AWAITING THE BANK` with vibegraph's σ, and writes no cell. With a record it
  gates and writes the `integrals` cell. If the row is no longer planned and
  the record is missing, it fails through `vibegraph::validation::require`.
- `statrs` joins vibegraph-cli's dev-dependencies (the seed-record reader
  needs it), as #82 also added.

**Why this budget.** The 512-point channel floor dominates: 68 channels make
an iteration about 47000 points at `--neval 20000` and about 67000 at
`--neval 40000`. The convergence measurement (§2) puts 12 iterations at
40000 on the plateau to 0.4%. One seed costs about 0.8M points.

**What the gate cannot see** is in its module doc: a region every seed
misses; the split of σ over subprocesses (the record's `by_channel` values
would see it); the per-event scale; and the two param cards, which are not
compared.

**Gate run and runtime** (on the merged tree, the host otherwise idle):

```
$ cargo test -p vibegraph --profile release-debug --config 'profile.release-debug.debug=0' \
    --features extended-validation --test validate_ewk_lljj_sigma -- --ignored --nocapture
[pp_to_epemjj_qcd0] import model sm-no_b_mass; generate p p > e+ e- j j QCD=0 QED=4 $ h on …/validation/madgraph/pp_to_epemjj_qcd0_run_card.dat, --fixed-budget --neval 40000 --niter 12, seeds [20261031, 20261032, 20261033, 20261034, 20261035]
  seed 20261031: σ = 3.6111418e0 ± 3.11e-2 pb (χ²/dof 3.07), 404 s
  seed 20261032: σ = 3.5834316e0 ± 3.63e-2 pb (χ²/dof 2.48), 416 s
  seed 20261033: σ = 3.5822541e0 ± 2.43e-2 pb (χ²/dof 4.55), 401 s
  seed 20261034: σ = 3.5210295e0 ± 2.14e-2 pb (χ²/dof 2.20), 406 s
  seed 20261035: σ = 3.6042065e0 ± 2.39e-2 pb (χ²/dof 2.53), 409 s
[pp_to_epemjj_qcd0] AWAITING THE BANK (the manifest marks row pp_to_epemjj_qcd0 planned and this checkout has no seed record output/seeded_sigma/pp_to_epemjj_qcd0, so nothing is compared and no cell is written) | vibegraph σ = 3.5804127e0 ± 1.59e-2 pb (5 seeds, quoted 1.25e-2, spread 1.59e-2, χ²/dof = 2.42), 2035 s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2034.72s
```

- **Runtime: 2035 s (34 minutes)** on four idle cores, about 405 s a seed;
  the test binary was prebuilt.
- Each seed equals the campaign's iteration-12 value for that seed to the
  printed digits, so the campaign below is the gate's own measurement.
- The per-seed `χ²/dof` printed is the integrator's across-iteration value,
  not the seed-policy statistic.

## 2. vibegraph's own side

Command (`T-V3-run.sh`): `vibegraph --no-tui -v -y integrate proc_card.dat
--run-card pp_to_epemjj_qcd0_run_card.dat --seed S -j 2 --fixed-budget
--neval 40000 --niter N`, with `LHAPDF_DATA_PATH=validation/pdf`.

- Seed 20261031 ran on the pre-merge binary. The merged binary
  reproduces its iteration 3 to every printed digit (`9.337362e-9 ±
  1.723692e-10`, run `chk-s20261031`), and the gate run reproduces its
  iteration 12 (3.6111 pb).
- The rungs nest: a seed's 12-iteration result is the iteration-12 line of
  its 24-iteration log.

**At the gate budget (12 iterations), pb:**

| seeds | values | mean ± err (quoted, spread) | sd a seed | sd / mean quoted | χ²/dof |
|---|---|---|---|---|---|
| gate's, 20261031–35 | 3.6111, 3.5834, 3.5823, 3.5210, 3.6042 | **3.5804 ± 0.0159** (0.0125, 0.0159) | 0.0355 (0.99%) | 1.30 | 2.42 |
| extra, 20261041–45 | 3.5696, 3.5424, 3.5749, 3.5820, 3.5890 | 3.5716 ± 0.0115 (0.0115, 0.0080) | 0.0179 (0.50%) | 0.70 | 0.65 |
| all ten | | 3.5760 ± 0.0085 (0.0085, 0.0085) | 0.0269 (0.75%) | 1.02 | 1.36 |

**Budget convergence**, the gate's five seeds run on to 24 iterations
(`T-V3-analyse.py`):

| iterations | mean (pb) | err | sd a seed | χ²/dof |
|---|---|---|---|---|
| 3 | 3.6547 | 0.0348 | 1.26% | 0.37 |
| 6 | 3.6090 | 0.0187 | 0.82% | 0.50 |
| 12 (gate) | 3.5804 | 0.0159 | 0.99% | 2.42 |
| 24 | 3.5818 | 0.0112 | 0.63% | 0.98 |

- Paired, 3 → 12: −0.074 ± 0.035 pb (−2.0%), all five fall.
- Paired, 12 → 24: +0.0014 ± 0.0141 pb (+0.04%), three of five rise.
- Per seed, 12 → 24: 3.6111 → 3.5761, 3.5834 → 3.5891, 3.5823 → 3.6145,
  3.5210 → 3.5522, 3.6042 → 3.5771.

**Reading it.**
- At the gate's budget the mean has stopped moving to within 0.4% (one σ of
  the paired difference), against a 2% move between 3 and 12 iterations.
  This is five seeds. They detect a climb of the size T-F2 measured on the
  CMS card (+2.4%); they cannot rule out one under about 1%.
- The early iterations read **high** here, where the CMS DY card's R1 read
  low (T-F2's paired R1 → R4 climb, +0.32 ± 0.12 pb). The cards and the
  budgets differ, so this is not a contradiction. Here the early grids
  over-weight a region rather than miss one.
- The gate's five seeds scatter by 1.3× their quoted errors (χ²/dof 2.42,
  driven by seed 20261034 at 3.521). The extra five sit at 0.7×. Ten seeds
  read 1.02×. Five-seed scatter statistics are themselves unreliable to a
  factor of about 2, so no conclusion is drawn from the 2.42 beyond it being
  inside the band.
- **Expected value for the M3:** vibegraph 3.580 ± 0.016 pb on the gate's
  seeds (3.576 ± 0.009 on ten). This is not a tuning target.

Pilot, for cost: seed 1 at `--neval 20000 --niter 12`, 1062 s on 2 threads
under load, σ = 3.5739 ± 0.0482 pb.

## 3. The M3 request

**M3 banking request (sprint E, T-V3): the no-pT-cut electroweak e+ e- j j σ row `pp_to_epemjj_qcd0`**

Branch `sprinte-spine-fallback` (PR #81) at the T-V3 report commit. From the
repository root, with the reference bundle and PDFs in place (`pixi run
fetch-refdata`, `pixi run fetch-pdf`). Nothing here needs `build-diagrams`:
the script carries `# built-by:`, so `build.sh` skips it, and the bank
generates its own never-run template.

1. Dry run (no MadGraph, no cargo; seconds):

       DRY_RUN=1 ROWS=pp_to_epemjj_qcd0 pixi run -e madgraph bank-seeded-sigma

   Expect: `kind card`, run card `validation/madgraph/pp_to_epemjj_qcd0_run_card.dat`,
   `param card none, the template default`, the generation script ending
   `import model sm-no_b_mass` / `generate p p > e+ e- j j QCD=0 QED=4 $ h` /
   `output <work>/pp_to_epemjj_qcd0/template -nojpeg`, `budget nevents 10000 at
   [20261101 ... 20261105], 40000 at [20261111 20261112]`, and `launch-block
   check 23 settings agree with the run card`. **Fail → stop** on `WOULD FAIL`.

2. Smoke: one seed at the budget, into a scratch area, to size MadEvent's cost
   (CMS's survey on the sibling three-flavour card took 3–4 CPU hours; this
   row has one lepton flavour, so expect roughly half that for the survey,
   plus refine to 10000 events). NB_CORE is the cores one MadEvent run uses;
   set it to what the M3 can give a single run:

       mkdir -p /tmp/vg-smoke
       time SEEDS=20261101 SEEDS_4X="" NB_CORE=8 \
         SEEDED_OUT=/tmp/vg-smoke/out SEEDED_WORK=/tmp/vg-smoke/work \
         ROWS=pp_to_epemjj_qcd0 pixi run -e madgraph bank-seeded-sigma
       cat /tmp/vg-smoke/out/pp_to_epemjj_qcd0/s20261101/vg_seed_result.txt
       grep -E 'Cross-section|Nb of events' /tmp/vg-smoke/work/pp_to_epemjj_qcd0/logs/s20261101.1.log | tail -3

   Expect: `>>> [pp_to_epemjj_qcd0] done`; the record log prints one seed line
   and `fewer than two seeds at nevents 10000: no seed-policy reading`; σ near
   3.58 pb. The wall time of this one seed sizes the bank: about
   5 × T(1x) + 2 × T(4x), with T(4x) between 1.5 and 4 × T(1x) (refine for
   40000 events). Send the `time` line, `vg_seed_result.txt` and the driver
   log back **before** step 3 if the projection exceeds a day.
   **Fail** if the run fails twice or σ is not finite and positive.

3. Bank the row. The smoke seed is reused: copied in, it is read back under
   the same card instead of re-run.

       mkdir -p validation/madgraph/output/seeded_sigma/pp_to_epemjj_qcd0
       cp -R /tmp/vg-smoke/out/pp_to_epemjj_qcd0/s20261101 \
         validation/madgraph/output/seeded_sigma/pp_to_epemjj_qcd0/
       NB_CORE=8 ROWS=pp_to_epemjj_qcd0 pixi run -e madgraph bank-seeded-sigma

   Expect: `>>> [pp_to_epemjj_qcd0 s20261101] kept under this card; read back`
   in the row's driver log, then `>>> [pp_to_epemjj_qcd0] done` and the screen
   block. **Pass**: the SCREEN and SCATTER lines do not read FLAGGED and the
   SURVEY line does not read INERT (`mmjj = 4` is below 0.01 × 13000, so the
   survey should honour `hard_survey = 2`). A FLAGGED line is not a failed
   bank, but the record must not be used until the driver's `SEED_PLAN=confirm20`
   (SCATTER) or `ladder20` (SCREEN) follow-up has run on it; report it.

4. The gate, on the M3 (five vibegraph seeds; about 34 minutes on four
   idle cores here, less on the M3):

       pixi run validate-ewk-lljj-sigma

   or, without pixi tasks,

       cargo test -p vibegraph --profile release-debug --features extended-validation \
         --test validate_ewk_lljj_sigma -- --ignored --nocapture

   Expect five `seed ...: σ = ...` lines, then one `[pp_to_epemjj_qcd0] GATE`
   line and `test result: ok. 1 passed`. The pass condition is |pull| < 3,
   |rel| < 0.02 and this side's seed χ²/dof inside (0.1, 4.0).
   vibegraph's side was measured here at 3.5804 ± 0.0159 pb on the gate's five seeds (3.5760 ± 0.0085 over ten; one seed scatters by about 1%).
   A failure is a finding, not a reason to re-bank: send it back as is.

5. Send back: `validation/madgraph/output/seeded_sigma/pp_to_epemjj_qcd0/`
   (whole, for the bundle), `validation/madgraph/work/seeded_sigma/screen.txt`,
   `validation/madgraph/work/seeded_sigma/pp_to_epemjj_qcd0.driver.log`, the
   smoke's `time` line, and the stdout of steps 1, 3 and 4.

## 4. The MLM re-gates

On the merged tree (`80a4c718`), detached (`T-V3-resume.sh`), each target
command as `pixi.toml`'s task with `-j 2` and debuginfo off.

```
sigma rc=0 wall=5551s rev=80a4c718  cargo test -j 2 -p vibegraph-lib --profile release-debug --config profile.release-debug.debug=0 --features extended-validation --test validate_hadronic -- --ignored --nocapture sigma_ll_0j2j_mlm_vs_madevent sigma_ttx_0j1j_mlm_vs_madevent
samples rc=0 wall=224s rev=80a4c718  cargo test -j 2 -p vibegraph --profile release-debug --config profile.release-debug.debug=0 --features extended-validation --test cli_generate_proton -- --ignored --nocapture matched_sample_record_fractions_against_madevent
```

The three llj tests of `validate-mlm-sigma` ran before the restart, on
`ac3e517b` (pre-merge). The restart killed that run while the two
mixed-multiplicity tests were in flight, so on the merged tree only those two
were re-run. T-F2's census says the llj rows' maps did not move.

| row (part) | recorded pull (manifest) | this session | vibegraph σ (pb) | MadEvent σ (pb) | rel | this side's χ²/dof |
|---|---|---|---|---|---|---|
| `pp_to_ttx_0j1j_mlm` @0 | +0.31 | +0.68 | 513.2801 ± 0.1281 | 513.1058 ± 0.2216 | +0.03% | 0.91 |
| `pp_to_ttx_0j1j_mlm` @1 | +0.70 | +0.74 | 579.2045 ± 0.2950 | 578.3374 ± 1.128 | +0.15% | 1.39 |
| `pp_to_ttx_0j1j_mlm` total | **+0.75** | **+0.88** | 1092.485 ± 0.2823 | 1091.440 ± 1.149 | +0.10% | 0.98 |
| `pp_to_ll_0j2j_mlm` @0 | +0.50 | +0.49 | 665.4251 ± 0.2298 | 665.0684 ± 0.6926 | +0.05% | 0.98 |
| `pp_to_ll_0j2j_mlm` @1 | +0.37 | +0.42 | 268.9310 ± 0.3233 | 268.6159 ± 0.6744 | +0.12% | 0.72 |
| `pp_to_ll_0j2j_mlm` @2 | +2.08 | +2.01 | 132.4986 ± 0.3825 | 131.5321 ± 0.2914 | +0.73% | 0.35 |
| `pp_to_ll_0j2j_mlm` total | **+1.54** | **+1.42** | 1066.855 ± 0.5511 | 1065.220 ± 1.010 | +0.15% | 0.55 |
| `pp_to_llj_mlm` (pre-merge) | **+1.00** | **+1.00** | 268.7417 ± 0.1155 | 268.1360 ± 0.5932 | +0.23% | 0.81 |
| `pp_to_llj_xqcut_only` (pre-merge) | +0.85 | +0.85 | 212.7019 ± 0.1528 | 212.3040 ± 0.4441 | +0.19% | 1.52 |
| `pp_to_llj_mlm_alps2` (pre-merge) | +0.88 | +0.88 | 240.9774 ± 0.1439 | 240.4980 ± 0.5242 | +0.20% | 0.26 |

Every gate line reads `gate: pass`; the two re-run tests end `test result:
ok. 2 passed`.
- The two moved rows shift by at most 0.4 in pull. The 0j2j @2 offset, the
  registered permuted-P1 and 2 → 4 deviations (about 0.7% of @2), stays where
  it was (+0.73%).
- The three llj rows reproduce their recorded pulls exactly, as T-F2's census
  predicts for unmoved maps.

**`validate-mlm-samples`** (`pp_to_llj_mlm`, informational):

```
events listing the Z: vibegraph 0.9527 (2858/3000), MadEvent 0.9507 (9507/10000), pull +0.44
events with the jet at the collider energy: vibegraph 0.1123 (337/3000), MadEvent 0.1123 (1123/10000), pull +0.01
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 14.24s
```

The manifest's last record (2026-10-03, bank host) reads 0.9547 (+0.89) and
0.1070 (−0.81). Both move toward MadEvent; 3000 events give each fraction a
binomial error of about 0.004 and 0.006, so neither move is resolved.

## Gates (on the merged tree, `80a4c718`)

`T-V3-gates.sh`, `T-V3-gates-summary.log`:

```
fmt rc=0 wall=6s  cargo fmt --all --check
clippy rc=0 wall=75s  cargo clippy -j 2 --workspace --all-targets -- -D warnings
clippy-ev rc=0 wall=77s  cargo clippy -j 2 --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
clippy-ev-lib rc=0 wall=72s  cargo clippy -j 2 -p vibegraph-lib --all-targets --features extended-validation -- -D warnings
test rc=0 wall=874s  cargo test -j 2 --workspace --no-fail-fast -- --test-threads 2
                     49 suites, 1745 passed, 0 failed, 24 ignored
kblint rc=0 wall=11s  python3 scripts/kb.py lint → 1016 concepts, 332 backlog items; 0 errors, 4 warnings
```

The four kb warnings are the pre-existing `state: blocked` backlog items.
After that run the gate's χ²/dof band and its budget comments changed, along
with the manifest note, this report and the regenerated sessions index. On
the final tree:

```
cargo fmt --all --check                                          → exit 0
cargo clippy -j 2 -p vibegraph --all-targets --features extended-validation -- -D warnings
                                                                 → Finished, no warning
cargo test -j 2 -p vibegraph-validation-report                   → 45 + 20 + 13 passed, 0 failed
python3 scripts/kb.py index                                      → wrote 1 index files
python3 scripts/kb.py lint                                       → 1017 concepts, 332 backlog items; 0 errors, 4 warnings
```

The gate run above (§1) is on the final tree's gate code. The workspace
`cargo test` and the two workspace clippy runs were not repeated after these
edits, which touch only the gate file's constants and comments, the manifest
note text and markdown.

## Corrections to the brief

- **"The CMS card uses `$$ h` or `/ t t~ h`."** The DY MLL-50 card uses
  `$ h`; the two VBF cards use `$$ h`. No CMS card uses `/ t t~ h`. The row
  keeps `$ h`, and the census shows every spelling is the same diagram set
  for this final state.
- **"`bank_seeded_sigma.sh` support … checked with a dry run."** This branch
  had no dry run before the merge; one was added here, and #82's arrived in
  the merge. The merged driver is the union.
- **"The run takes about an hour."** The MLM σ re-gate alone took 5551 s on
  the shared host for the two mixed-multiplicity tests (ten seeds each, about
  400 s a seed).

## Incidents

- **Container restart, about 17:46 UTC.** It killed the MLM σ run (three llj
  tests done, the two mixed-multiplicity tests in flight) and the campaign
  driver (seed 20261031 recorded, 20261032 in flight). The worktree survived.
  Relaunched on the merged tree from `T-V3-resume.sh`: the binary rebuilt and
  checked against the recorded seed (bit-equal iteration 3), recorded
  campaign tags skipped, and only the two unfinished MLM tests re-run.
- **A sleep-based watcher.** At 16:34 I started a backgrounded
  `until [ -s … ]; do sleep 5; done`, against the brief, and stopped it with
  TaskStop within a minute. It ran nothing else.
- **`cd /home/user/wt/T && cargo …` in two harness-backgrounded commands.**
  Each ran in its own shell, which the harness confirmed left the session's
  directory unchanged, and wrote nothing outside `/home/user/wt/T/target` and
  the scratchpad. It is still the form the brief forbids; the later commands
  use `--manifest-path`.
- **Shared host.** Sibling session S ran a workspace build in
  `/home/user/wt/S` during the first two hours. Timings before about 19:30
  are on a loaded host; the gate runtime above is on an idle one.

## Found

- **MadGraph's diagram census of this row is not taken.**
  - *Evidence:* the `$ h` claim rests on vibegraph's census alone (2424
    diagrams across all four spellings). The bank keeps results, not
    `matrix*.f`.
  - *To close:* run `extract-diagrams` on the bank's template
    (`work/seeded_sigma/pp_to_epemjj_qcd0/template`) and compare per
    subprocess, which also opens the row's `diagrams` cell.
- **The gate cannot see the split of σ over subprocesses.**
  - *Evidence:* it compares the total. The seed record keeps `by_channel`
    values, and vibegraph's artifact keeps per-channel σ, but the two channel
    sets are not mapped onto each other.
  - *To close:* compare per subprocess group (`P*` directory against
    vibegraph's flavour group), as the `dy13_etau` gate does per directory.
- **The early iterations read high on this row.**
  - *Evidence:* every one of five seeds falls between iteration 3 and
    iteration 12 (−2.0%), and the 6-iteration mean is still +0.8% above the
    plateau.
  - *To close:* find which channels carry the excess at iteration 3 (the
    per-channel σ in the artifact at `--niter 3` against `--niter 12`). It
    matters for any lower budget, such as the generate harness's R1 grids.
- **The two-seed bank screen will be weak on this row.**
  - *Evidence:* vibegraph's per-seed scatter here is about 1%, and MadEvent's
    on a 68-channel 2 → 4 process with refine is unmeasured. The SCREEN line
    reads two 4× seeds against five.
  - *To close:* if SCREEN or SCATTER flags, run `SEED_PLAN=confirm20` or
    `ladder20` on the M3 before the record is used, as the driver says.
- **A planned row's gate costs its full runtime while it waits.**
  - *Evidence:* the AWAITING path integrates five seeds (about 34 minutes)
    to print vibegraph's σ, so `cargo test -- --ignored` across the workspace
    spends that time on an unbanked row.
  - *To close:* keep it (the printed σ is the expected value the bank is
    read against), or gate the integration behind an environment variable
    while the row is planned.
