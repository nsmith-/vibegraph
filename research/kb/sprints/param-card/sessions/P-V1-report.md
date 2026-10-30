---
type: Session Report
title: "P-V1 report: banking inputs, MadGraph's coupling fusion, and the silent-card fixes"
description: "MadGraph's 14 fused coupling groups hold at MZ = 95 and ymt = 250 (measured with MadGraph's own restriction); the two moved-parameter rows, their cards and driver hooks, and a σ gate that reports them as awaiting the bank; the SLHA reader refuses non-numeric widths, short DECAY lines, non-integer codes, conflicting duplicates and reopened blocks; a card moving a parameter the restriction fixes to one is refused; the M3 banking request."
status: draft
tags: [validation, param-card, slha, madgraph, coupling-fusion, sigma, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
Worktree `/home/user/wt/P`, branch `sprinte-param-card`, from `e1281f9e`. Every
cargo command ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`; the release-debug gates also with `--config
'profile.release-debug.debug=0'`. Pixi is not installed, so each pixi task's
command ran directly. The container restarted twice during the gates; the
banked targets were then run one at a time from a resumable driver.

Commits: `c02147b3` (rows, gate, fusion check, parser and binding refusals),
`b9890cf8` (the CKM test's moved card), then this report. Nothing is pushed.

## 1. MadGraph's fused couplings

`validation/madgraph/check_coupling_fusion.py` runs the pinned MadGraph's own
restriction (`import model sm`, no generation), lists the coupling groups it
fuses, and evaluates the model's couplings at each point, checking three
things: no fused pair splits, no coupling removed as zero
becomes non-zero, and no parameter fixed to zero or one moves. A control point
that must fail (`lamWS = 0.2265`) guards against a check that cannot fail.

**Measured in this container** on HEAD `b9890cf8` (`python3 -I
validation/madgraph/check_coupling_fusion.py`, exit 0; MadGraph 3.7.1 from the
submodule). Its output is identical, line for line, to the run taken before
the WIP commit. The groups (member = factor × first):

```
fused groups (member = factor * first):
  GC_100(+1), GC_104(+1), GC_108(+1), GC_40(+1), GC_41(+1), GC_45(+1), GC_49(+1)
      GC_100 = (mdl_ee*mdl_complexi*mdl_conjg__CKM1x1)/(mdl_sw*mdl_sqrt__2)
      GC_104 = (mdl_ee*mdl_complexi*mdl_conjg__CKM2x2)/(mdl_sw*mdl_sqrt__2)
      GC_108 = (mdl_ee*mdl_complexi*mdl_conjg__CKM3x3)/(mdl_sw*mdl_sqrt__2)
      GC_40 = (mdl_ee*mdl_complexi)/(mdl_sw*mdl_sqrt__2)
      GC_41 = (mdl_CKM1x1*mdl_ee*mdl_complexi)/(mdl_sw*mdl_sqrt__2)
      GC_45 = (mdl_CKM2x2*mdl_ee*mdl_complexi)/(mdl_sw*mdl_sqrt__2)
      GC_49 = (mdl_CKM3x3*mdl_ee*mdl_complexi)/(mdl_sw*mdl_sqrt__2)
  GC_21(+1), GC_27(-1)
      GC_21 = -mdl_I2x33
      GC_27 = mdl_I3x33
  GC_15(+1), GC_30(-1)
      GC_15 = mdl_I1x33
      GC_30 = -mdl_I4x33
  GC_38(+1), GC_39(-1)
      GC_38 = -(mdl_ee*mdl_complexi)/(2.*mdl_sw)
      GC_39 = (mdl_ee*mdl_complexi)/(2.*mdl_sw)
  GC_3(+1), GC_4(-1)
      GC_3 = -(mdl_ee*mdl_complexi)
      GC_4 = mdl_ee*mdl_complexi
  GC_50(+1), GC_51(-1)
      GC_50 = -(mdl_cw*mdl_ee*mdl_complexi)/(2.*mdl_sw)
      GC_51 = (mdl_cw*mdl_ee*mdl_complexi)/(2.*mdl_sw)
  GC_54(+1), GC_56(-1)
      GC_54 = -mdl_ee__exp__2/(2.*mdl_sw)
      GC_56 = mdl_ee__exp__2/(2.*mdl_sw)
  GC_66(+1), GC_67(-1)
      GC_66 = -(mdl_ee__exp__2*mdl_vev)/(2.*mdl_cw)
      GC_67 = (mdl_ee__exp__2*mdl_vev)/(2.*mdl_cw)
  GC_70(+1), GC_73(-1)
      GC_70 = -(mdl_ee__exp__2*mdl_vev)/(4.*mdl_sw__exp__2)
      GC_73 = (mdl_ee__exp__2*mdl_vev)/(4.*mdl_sw__exp__2)
  GC_74(+1), GC_75(-1)
      GC_74 = -(mdl_ee__exp__2*mdl_vev)/(2.*mdl_sw)
      GC_75 = (mdl_ee__exp__2*mdl_vev)/(2.*mdl_sw)
  GC_77(+1), GC_78(-1)
      GC_77 = (mdl_ee__exp__2*mdl_vev)/(4.*mdl_cw) - (mdl_cw*mdl_ee__exp__2*mdl_vev)/(4.*mdl_sw__exp__2)
      GC_78 = -(mdl_ee__exp__2*mdl_vev)/(4.*mdl_cw) + (mdl_cw*mdl_ee__exp__2*mdl_vev)/(4.*mdl_sw__exp__2)
  GC_76(+1), GC_79(-1)
      GC_76 = -(mdl_ee__exp__2*mdl_vev)/(4.*mdl_cw) - (mdl_cw*mdl_ee__exp__2*mdl_vev)/(4.*mdl_sw__exp__2)
      GC_79 = (mdl_ee__exp__2*mdl_vev)/(4.*mdl_cw) + (mdl_cw*mdl_ee__exp__2*mdl_vev)/(4.*mdl_sw__exp__2)
  GC_7(+1), GC_9(-1)
      GC_7 = -mdl_ee__exp__2/(2.*mdl_cw)
      GC_9 = mdl_ee__exp__2/(2.*mdl_cw)
  GC_96(+1), GC_97(-1)
      GC_96 = -mdl_ytau
      GC_97 = mdl_ytau
```

The verdicts:

```
== restriction point: holds
   MZ 91.188, MW 80.41900245, ymt 173
   fused pairs split: none
== dy13_mmll_60_120_mz95: MZ = 95: holds
   MZ 95, MW 85.03597559, ymt 173
   fused pairs split: none
== ee_to_tth_ymt250: ymt = 250: holds
   MZ 91.188, MW 80.41900245, ymt 250
   fused pairs split: none
== generic: every non-zero external moved: holds
   MZ 90.4, MW 79.48925254, ymt 166.9
   fused pairs split: none
== control: lamWS = 0.2265 (vibegraph refuses this card): control splits, as it must
   MZ 91.188, MW 80.41900245, ymt 173
   fused pairs split: GC_108 = +1 * GC_100 (rel 2.565e-02); GC_40 = +1 * GC_100 (rel 2.565e-02); GC_49 = +1 * GC_100 (rel 2.565e-02)
PASS
```

The groups whose expressions depend on MZ (through `sw`, `cw`, `vev`) are the
`GC_100` group, `GC_38/39`, `GC_50/51`, `GC_54/56`, `GC_66/67`, `GC_70/73`,
`GC_74/75`, `GC_76/79`, `GC_77/78` and `GC_7/9`. No group depends on `ymt`;
`GC_96/97` is `ytau`. Each member is the first times ±1, or differs from it only
by diagonal CKM entries the restriction fixes to one (internal parameters of the
zero-locked `lamWS`, which vibegraph refuses to move). So no card that moves
external masses or Yukawas can split a group. Both rows are safe, and no
alternative row is needed.

The check cannot see a fusion that `restrict_default.dat` does not make, or a
split under a restriction other than the SM default. It reads MadGraph's
restriction code, not its generated Fortran, so a fusion applied differently at
output time would not show; `coupling_oracle` covers the generated side at the
restriction point.

## 2. The banking inputs

- `validation/madgraph/dy13_mz95_param_card.dat`: `dy13_param_card.dat` with
  MZ = 95. `bank_seeded_sigma.sh` gained a param-card hook for the `dy13` kind,
  and its dry run prints the entries that differ from the default card.
- `validation/madgraph/scripts/ee_to_tth_ymt250.mg5`, with `set yukawa 6 250` in
  its launch block. The driver now checks a launch block's three-argument `set`
  lines against the banked param card, and its run-card sets against the run card.
- Manifest rows `pp_to_ll_mz95` and `ee_to_tth_ymt250`, marked planned.

`bash -n` passes on `bank_seeded_sigma.sh` and `madevent_seeds.sh`. The dry run
on HEAD (`SEEDED_WORK=<scratch> DRY_RUN=1 ROWS="dy13_mmll_60_120_mz95
ee_to_tth_ymt250" bash validation/madgraph/bank_seeded_sigma.sh`, exit 0):

```
>>> dry run: 2 row(s), seeds [20261101 20261102 20261103 20261104 20261105] at the budget and [20261111 20261112] at 4x
=== dy13_mmll_60_120_mz95
    kind       dy13
    run card   /home/user/wt/P/validation/madgraph/dy13_mmll_run_card.dat
    param card /home/user/wt/P/validation/madgraph/dy13_mz95_param_card.dat
    tree       pinned
    generator  generate p p > e+ e- (gen_hadronic_sigma.sh); run card dy13_mmll_run_card.dat; param card dy13_mz95_param_card.dat
    generation script:
        generate p p > e+ e-
        output /tmp/claude-0/-home-user-vibegraph/80d5fa95-18c8-5bff-ac38-2a2fde4863d4/scratchpad/P-V1-dry-work/dy13_mmll_60_120_mz95/template -nojpeg
    budget     nevents 200000
    param card entries that differ from dy13_param_card.dat:
           23 9.500000e+01 # MZ 
=== ee_to_tth_ymt250
    !!! ee_to_tth_ymt250: no banked cards under output/ee_to_tth_ymt250 (pixi run fetch-refdata)
    WOULD FAIL: the row setup above refused it
```

`ee_to_tth_ymt250` has no process directory yet; the M3 builds it (step 3 below).

## 3. The σ gate

`vibegraph-cli/tests/validate_param_card_sigma.rs` (feature
`extended-validation`): three tests.

- **The two row tests.** Each integrates the row's process at its card over five
  seeds, combines them by unweighted mean, and gates against the row's seed
  record. The gate takes |pull| < 3 and |rel| < 0.005, plus χ²/dof < 4 for both
  sides. It writes the `integrals` cell. While the manifest marks a row planned
  and the seed record is absent, the test prints `AWAITING THE BANK` with the
  reason, and writes no cell.
- **The informational ratio** σ(ymt = 250)/σ(default) for `e+ e- > t t~ h`,
  against MadGraph's reweight module (P-F1's measurement), reported and not
  gated.

Measured on HEAD:

```
[ee_to_tth_ymt250] INFO σ(default) = 2.0170961e-3 ± 6.05e-7 pb (χ²/dof 1.52), σ(ymt=250) = 4.0961865e-3 ± 1.01e-6 pb (χ²/dof 1.02), 5 seeds each
[ee_to_tth_ymt250] INFO vibegraph σ(250)/σ(default) = 2.03073 ± 0.00079 | MadGraph reweight ⟨w250/XWGTUP⟩ = 2.03726 ± 0.00550 over 200 events | pull = -1.18
[ee_to_tth_ymt250] INFO known-wrong: σ(250) against σ(default), rel = +1.0307
[pp_to_ll_mz95] AWAITING THE BANK (... no seed record output/seeded_sigma/dy13_mmll_60_120_mz95 ...)
[ee_to_tth_ymt250] AWAITING THE BANK (... no seed record output/seeded_sigma/ee_to_tth_ymt250 ...)
```

The ratio oracle is weak. MadGraph's side is 200 events, so its uncertainty
dominates and the check cannot resolve anything under about 0.5 %. The
known-wrong row (+103 %) shows the gate sees the moved card at all.

## 4. The silent-card fixes

**SLHA parser** (`vibegraph-lib/src/ufo/slha.rs`). Each of these is now a
`SlhaError::Parse` naming its line:

- a `DECAY` width that is neither a number nor `Auto`;
- a `DECAY` line with no width;
- a block entry whose code tokens are not integers;
- a `(block, code)` repeated at a different value. MadGraph raises "is already
  define"; a repeat at the same value is read, as MadGraph reads it.
- a block opened twice. MadGraph's `ParamCard.append` (`self[obj.name] = obj`)
  silently keeps only the second copy, so a merge would evaluate a different
  point than MadGraph, and refusal is the only reading both sides share.

Each new state's `QNUMBERS` block is kept under its PDG code (`qnumbers
9000005`), as MadGraph names it, so two states' blocks are two blocks.

**Parameters fixed to one** (`vibegraph-lib/src/ufo/{parameters.rs,bind.rs}`).
`apply_restrict` records the externals a restriction sets to exactly 1 in
`params.ones`, as MadGraph's `detect_special_parameters` does. `bind_card`
refuses a card that moves one (`CardError::MovesFixedOne`), and `bind_recorded`
refuses one in a run's record. No repository restriction fixes an external to
one: the interned SM's `ones` is empty for every restriction (pinned by
`the_interned_sm_fixes_nothing_to_one`), since the SM default's ones are internal
CKM entries. So the refusal is tested on the toy UFO's `restrict_all.dat`, which
fixes the width `WO8` to 1.

**Test helpers that relied on a merged reopened block** now write one header per
block: the reweight engine's `fresh` card and `sm_ckm_no_b_mass`'s
`moved_card`. `reweight_mg_oracle` leaves the banked `qnumbers` entries out of its
rebuilt card (see Found); they carry no model parameter, and the oracle binds 143
SMEFT card entries.

**Mutation evidence.** Each mutation was applied to the source, the
`ufo::slha ufo::bind ufo::parameters` unit tests were run, and the file was
restored and byte-compared against the original. All 9 mutations were caught:

| mutation | caught by |
|---|---|
| width_not_number | `lines_the_card_cannot_honour_are_parse_errors_at_their_line` |
| decay_short | the same |
| non_integer_key | the same |
| conflicting_duplicate | the same |
| reopened_block | the same |
| qnumbers_naming | `each_new_state_has_its_own_qnumbers_block`, `an_entry_the_model_lacks_is_refused` |
| ones_refusal | `moving_a_parameter_the_restriction_fixes_to_one_is_refused` |
| ones_detection | the same, and `only_a_restricted_value_of_exactly_one_is_fixed` |
| ones_record | `moving_a_parameter_the_restriction_fixes_to_one_is_refused` |

## Banking request (for the M3)

**M3 banking request (sprint E, P-V1): two moved-parameter σ rows**

Branch `sprinte-param-card` (PR #82) at the P-V1 commit. From the repository root,
with the reference bundle and PDFs in place (`pixi run fetch-refdata`, `pixi run fetch-pdf`).

1. Coupling-fusion check (MadGraph's own restriction, no generation; seconds):

       pixi run -e madgraph python -I validation/madgraph/check_coupling_fusion.py

   Expect: 14 fused groups listed; `holds` for the restriction point,
   `dy13_mmll_60_120_mz95: MZ = 95`, `ee_to_tth_ymt250: ymt = 250` and the generic point;
   `control splits, as it must` for the lamWS control; last line `PASS`, exit 0.
   **Fail → stop**: do not bank; send the output back.

2. Dry run (prints what would run; no MadGraph, no cargo):

       DRY_RUN=1 ROWS="dy13_mmll_60_120_mz95 ee_to_tth_ymt250" pixi run -e madgraph bank-seeded-sigma

   Expect for `dy13_mmll_60_120_mz95`: kind `dy13`, run card `dy13_mmll_run_card.dat`,
   param card `dy13_mz95_param_card.dat`, budget `nevents 200000`, and the one differing
   line `23 9.500000e+01 # MZ`. `ee_to_tth_ymt250` prints `WOULD FAIL ... no banked cards
   under output/ee_to_tth_ymt250` until step 3.

3. Build the t t~ h process directory (only the missing one is built; ~minutes):

       pixi run -e madgraph build-diagrams
       grep -E '^ +6 2\.500000e\+02 # ymt' validation/madgraph/output/ee_to_tth_ymt250/Cards/param_card.dat
       grep -iE '^ +6 1\.730000e\+02 # MT' validation/madgraph/output/ee_to_tth_ymt250/Cards/param_card.dat

   Expect: `Processing: ee_to_tth_ymt250.mg5`, `✓ Completed`, every other script skipped;
   both greps print one line (ymt moved, MT not).
   **Fail** if the build fails or either grep prints nothing.

4. Dry run again (same command as 2). Expect `ee_to_tth_ymt250`: kind `script`, run card and
   param card under `output/ee_to_tth_ymt250/Cards`, no `WOULD FAIL` (the launch block's
   run-card sets and its `set yukawa 6 250` both checked against those cards).

5. Bank both rows (~20 min for the Drell-Yan row at 5 × 200k + 2 × 800k events, a few minutes for t t~ h):

       ROWS="dy13_mmll_60_120_mz95 ee_to_tth_ymt250" pixi run -e madgraph bank-seeded-sigma

   Expect: `>>> [row] done` for both, then the screen block with SCREEN, SURVEY and SCATTER
   lines per row. **Pass**: both rows done; no SCREEN or SCATTER line reads FLAGGED; no SURVEY
   line reads INERT. A FLAGGED line is not a failure of the bank but needs the driver's
   `SEED_PLAN=confirm20` / `ladder20` follow-up before the record is used; report it.
   Expect σ near 747 pb (Drell-Yan; vibegraph reads 746.87 ± 0.73 pb on one seed) and near
   4.10e-03 pb (t t~ h; vibegraph 4.09e-03).

6. The gate, on the M3:

       cargo test -p vibegraph --profile release-debug --features extended-validation \
         --test validate_param_card_sigma -- --nocapture --test-threads 1

   Expect two `GATE` lines (`pp_to_ll_mz95`, `ee_to_tth_ymt250`) with |pull| < 3,
   |rel| < 0.005 and χ²/dof < 4, and `test result: ok. 3 passed`.

7. Send back: `validation/madgraph/output/seeded_sigma/dy13_mmll_60_120_mz95/`,
   `validation/madgraph/output/seeded_sigma/ee_to_tth_ymt250/`,
   `validation/madgraph/output/ee_to_tth_ymt250/` (for the bundle),
   `validation/madgraph/work/seeded_sigma/screen.txt`, both rows'
   `work/seeded_sigma/<row>.driver.log`, and the stdout of steps 1, 3, 5 and 6.

## Gates

Commands as run, with the tail of each output. The table is the final tree
(`b9890cf8` plus nothing).

```
$ cargo fmt --all --check
exit=0

$ cargo clippy --workspace --all-targets -- -D warnings
exit=0

$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
exit=0

$ cargo test --workspace --no-fail-fast
49 binaries, 1735 passed, 0 failed, 19 ignored; exit=0

$ python3 scripts/kb.py lint
0 errors, 4 warnings (state: blocked backlog items this session did not touch)
```

The banked targets the extended-validation skill names for a model-binding,
reweighting and LHEF change, plus the new gate. They ran one at a time from a
resumable driver that appends each verdict as soon as the target finishes:

```
reweight_mg_oracle PASS exit=0 18s | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
coupling_oracle PASS exit=0 47s | test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.79s
amplitude_oracle PASS exit=0 14s | test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.86s
validate_lhef PASS exit=0 209s | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 84.01s
validate_param_card_sigma PASS exit=0 160s | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.93s
cli_integrate PASS exit=0 38s | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.79s
cli_reweight_proton PASS exit=0 106s | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 96.70s
cli_generate_proton PASS exit=0 34s | test result: ok. 5 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 21.45s
ALL TARGETS DONE
```

The pixi tasks these replace: `validate-reweight` (`reweight_mg_oracle`,
`cli_reweight_proton`), `validate-couplings`, `validate-amplitudes`,
`validate-lhef` and `validate-generate-proton`. `cli_integrate` and
`validate_param_card_sigma` have no task. The two ignored tests in
`cli_generate_proton` are the oracle-layer MLM test and a probe, both
`#[ignore]` by design. Worst values read from the logs:

- amplitude oracle: `uux_to_ttx_4f` |M|² max_rel 4.59e-15. `uux_to_ccx_emmm_qcd0`
  reaches 1.99e-11 on one event, which its own conditioning report attributes to
  the point.
- coupling oracle: worst gap against MadGraph's Python 5.41e-15 (`GC_286`, 24 ulp).
- `sm_interned_blob`: 2 passed (run before the test-helper fixes, which do not
  touch it).

**Not run here:** `pixi run --skip-deps validate` as one job. Two whole-layer
attempts died. The first was my error: a cargo profile variable
(`CARGO_PROFILE_RELEASE_DEBUG_DEBUG`, which cargo reads as a key of
`profile.release`) stopped every build before any gate ran. The relaunch was
killed by a container restart while still compiling. The rest of
the banked layer is unchanged by this session, apart from the stricter card
reader. That reader reaches every gate that reads a param card, and the targets
above are the ones that do.

**The first gate run's failures, and their causes.**

- **The engine and oracle helpers.** `cargo test --workspace` without
  `--no-fail-fast` failed in the lib target on the reweight engine's helper
  (12 tests), and `reweight_mg_oracle` failed on the merged `qnumbers` entries.
  Both are fixed above.
- **A hidden failure.** Cargo stops at the first failing test target, so the
  `sm_ckm_no_b_mass` failure only showed on the next run; it is fixed in
  `b9890cf8`.
- **Clippy.** With `extended-validation`, clippy flagged one needless borrow in
  the new gate.

## Brief corrections

- The brief asks for "the reweight gates, detached" only. The manager later
  widened the banked gate to the eight targets above, run singly after two
  container restarts.
- P-F1's Found 3 (fusion) is closed by measurement, not by a row change: no
  fused group splits at either point.

## Found

- **The banked reweight reference loses the QNUMBERS PDG codes.**
  `gen_reweight_oracle.py` writes each new state's QNUMBERS entries under a bare
  `qnumbers` block. In `rows.ee_ttx_smeft.param_card` of
  `reweight_mg_reference.json`, 16 entries for four states sit under one name
  with clashing codes, and the oracle can no longer rebuild that part of the
  card. Closing it: write the name with the PDG code (`qnumbers <pdg>`) and
  regenerate with `pixi run -e madgraph generate-reweight-oracle`.
- **`pixi` is absent on the cloud host.** `pixi run --skip-deps validate`
  returns exit 127 here, so each gate ran as the cargo command its task names.
  Closing it: install pixi in the environment setup, or name `bash
  validation/validate.sh` as the fallback in the dev-agent brief.
- **Whole-layer `validate.sh` does not survive restarts.** It is one multi-hour
  job that writes its cells only as each test binary ends, so a restart loses
  the whole run. Closing it: a resumable driver over the layer's targets, like
  the one used here, that records each verdict as it lands.
