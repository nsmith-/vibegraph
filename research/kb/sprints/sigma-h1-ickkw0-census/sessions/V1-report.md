---
type: Session Report
title: "V1 report: H1 census of the banked grouped rows"
description: "Every banked MadEvent run checked for a clustered first call, symmetry permutations (machine-read) and a same-seed patched/unpatched screen. No banked row besides the two already registered carries a measurable H1, and no row needs re-banking."
status: draft
tags: [h1, madevent-reference, scales, grouping, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
Brief steps 1 and 2, phase B (permutations) and phase C0 (zero-variance screen)
are done. Phase C1, the statistical runs, was not started, at the manager's
instruction; the screen leaves nothing for it to measure on this sprint's rows.
Every MadGraph step ran on the pinned 3.7.1 tree through
`validation/madgraph/mg5_pinned.sh` under `pixi run -e madgraph --frozen`.

## Verdict

| row(s) | first call clusters | σ can move | non-identity permutations | C0 screen | verdict |
|---|---|---|---|---|---|
| `pp_to_llj`, `pp_to_llj_dyn` | yes | yes | none | `pp_to_llj_dyn` identical (mirror control) | unaffected |
| `pp_to_llj_mlm`, `_mlm_alps2`, `_xqcut_only` | yes | yes | none | not needed | unaffected |
| `uux_to_epemg`, `ddx_to_epemg`, `gu_to_epemu`, `gux_to_epemux` | yes | yes (α_s) | none | not needed | unaffected |
| `pp_to_ll`, `_qcd0`, `_scalefact2`, `uux_to_uux`, `uux_to_ztg` | yes | yes | none | not needed | unaffected |
| `pp_to_jj` | yes | yes | `P1_gg_gg`, `P1_gg_qq` config 3, `P1_qq_qq` config 2: `(1,2,4,3)` | identical | unaffected |
| `pp_to_bb`, `pp_to_bb_qcd2` | yes | yes | `P1_gg_bbx` config 3: `(1,2,4,3)` | identical | unaffected |
| `gg_to_gg`, `gg_to_ttx`, `gg_to_ttx_smlimit`, `_qcd2` | yes | yes (α_s) | config 3 (4 for `_qcd2`): `(1,2,4,3)` | identical | unaffected |
| `pp_ttx_lep_dyn` (decay-chain events) | yes | yes | `gg` config 3: the whole t ↔ t̄ chain swap | identical | unaffected |
| `uu_full`, `uu_tt` (onshell veto) | yes | yes (α_s) | configs 4, 6, 8 (b ↔ b̄) and 9 (W⁺ ↔ W⁻ and b ↔ b̄) | **differ**, at 3e-5 of σ | H1 present, negligible: see below |
| `pp_to_ll_0j2j_mlm` | yes | yes | `P2_qq_llqq` 10/24, `P2_gg_llqq` 8/16, `P2_qq_llgg` 6/16 configs | not re-measured | registered (D2) |
| `pp_to_ttx_0j1j_mlm` | yes | yes | t ↔ t̄ only (`P0_gg_ttx`, `P1_*`) | not re-measured | sibling #23: +0.03 ± 0.11 pb on `@1` |
| every `ee_*` run, the toy rows, `uux_to_mumu`, `uux_to_ttx_4f`, `tata_to_ttx_tensor4f`, `*_ccx_emmm_qcd0`, `wpwm_to_wpwmz_cw` | yes | **no** (fixed beams, no α_s coupling) | not read | — | unaffected |
| `gg_to_gg_cg` | no (`dynamical_scale_choice = 3`) | — | — | — | unaffected |
| `pp_to_bb_fixed`, `pp_to_llj_fixed`, `ud_to_epemud_qcd0`, `dy13_*`, onshell `pp_*`, decay-chain `pp_ttx_lep` | no (all scales fixed) | — | — | — | unaffected |

**Banking request: none.** No row needs re-banking, and no new registration is
needed beyond `pp_to_ll_0j2j_mlm`'s existing one.

## 1. When the first call clusters (pinned tree 3.7.1)

`update_scale_coupling_vec` (`Template/LO/SubProcesses/reweight.f:1856`) calls
`setclscales` on every point. `setclscales` returns before clustering when
`ickkw ≤ 0`, `xqcut ≤ 0`, and `q2fact` and `scale` are both already positive.
So the first call clusters exactly when:
- `ickkw > 0`, or
- `xqcut > 0`, or
- `dynamical_scale_choice = -1` and not all three of `fixed_ren_scale`,
  `fixed_fac_scale1` and `fixed_fac_scale2` are set.

The other dynamical choices (1 to 4, `setscales.f`) are final-state sums or
√ŝ, which a final-state permutation leaves unchanged.

A clustered first call moves σ in three ways:
- through the PDF scales, which needs `lpp ≠ 0`;
- through μR, which needs an α_s-dependent coupling;
- through a rejection: xqcut or xmtc, or the 2 GeV factorisation floor
  (`reweight.f:1206`), which also needs `lpp ≠ 0`.

The α_s dependence was read from the couplings each `matrix*_orig.f` uses,
looked up in the model's `couplings.py`. In the SM these are `GC_10`, `GC_11`
and `GC_12`, and in SMEFTsim `GC_6` and `GC_7`. The toy models have none.

**The brief's criterion was incomplete in two directions.**
- `xqcut > 0` clusters even at `ickkw = 0`, as in `pp_to_llj_xqcut_only`.
- `dynamical_scale_choice = -1` with all three scales fixed does not cluster.

## 2. Card census

`validation/madgraph/census_permuted_first_call.py runs` reads every banner
under `validation/madgraph/output` (89 runs in 72 directories). Results:
- All are `group_subprocesses Auto`, so every 2 → N output is grouped.
- All are MadGraph 3.7.1, except the six deliberate 3.5.7 runs
  (`ee_to_mumu_tata_qcd0`'s window, anti-window and controls, and `var_sde1`).
  These are e+e- at QCD = 0, which H1 cannot reach.
- Eleven banners print the "UNKNOWN DEVELOPMENT VERSION" warning with
  `VERSION 3.7.1`. These are the cut-5 and cut-9 rows, generated from a copy of
  the pin without `.git`, as the manifest's cut-9 comment says.
- `vector_size = 1` and `nb_warp = 1` are explicit in 25 banners. The rest
  omit them, which means the default of 1 (`banner.py:4465`).

## 3. The vectorised path

`DSIG_VEC` calls `update_scale_coupling_vec(all_p, …)`
(`super_auto_dsig_group_v4.inc:312`) on the unpermuted, unmirrored `ALL_P`. It
does so before `DSIGPROC_VEC` builds `ALL_P1` (`SWITCHMOM` `:959`, mirror
`:983-984`), so it has the same shape as the scalar first call.

`dsample.f:205` takes the scalar `dsig` when `VECSIZE_USED ≤ 1`, and
`VECSIZE_MEMMAX = nb_warp × vector_size` (`banner.py:4765`) is 1 for every
banked run. **Every banked run took the scalar path**, where the one-line patch
is the whole fix. The vectorised path stays unmeasured, and no banked row
reaches it.

## 4. Phase B: the permutations, machine-read

Each row's own MG5 script was run up to `output` with the pinned tree. The
off-bundle rows used their generator's process line. `uu_tt` was also
generated on the ALOHA-p1d-patched copy, as its reference was, and its
`symperms.inc` matches the pinned output's byte for byte.

`census_permuted_first_call.py perms` reads `symperms.inc`, `configs.inc`'s
`MAPCONFIG` and `symfact_orig.dat`.

- **Positive control first.** The R1 grouped reproducer reads configs 2, 5, 6
  and 8 as `(1,2,3,5,4)`, mapped onto 1, 3, 4 and 7, which is R1's own
  reading.
- **The regenerated directories are the banked ones.** For all 23 bundle
  rows, every P directory name, `leshouche.inc` and `matrix*_orig.f` (version-banner comment lines aside) matches
  the bundle's.
- `pp_to_ll_0j2j_mlm`'s `P2_qq_llqq` maps 10 of its 24 configs, which is the
  H1 concept's figure.

The non-identity entries are those in the verdict table. Every three-body
final state reads identity. γ/Z → l⁺l⁻ is a single vertex, so no two
distinct configurations differ by a final-leg swap.

## 5. Phase C0: same-seed screen

**Design.** For each row, one freshly generated directory was copied twice:
- the run card was the banked card (bundle rows) or the committed card with
  the generator's overrides (off-bundle rows);
- `nevents = 2000`, `iseed = 20261015`, `use_syst = F`;
- `run_mode = 2` on 2 cores, with `LDFLAGS += -lstdc++` for the LHAPDF link;
- one copy is unpatched;
- in the other, every `SubProcesses/P*/auto_dsig.f` has
  `CALL UPDATE_SCALE_COUPLING(PP, WGT)` → `(P1, WGT)`. This is `repro.sh`'s
  edit, the generated form of `patches/first-call-unpermuted-momenta.patch`.

**Comparison.** It covers every channel's `results.dat` with the timing lines
dropped, and every line of the 2000-event unweighted sample. The sample
carries SCALUP and AQCDUP, which at `ickkw = 0` are the first call's.
Identical output means the integrand was identical at every sampled point to
printed precision, so H1 is zero on that row. This needs no statistics.

**Control.** The R1 grouped process with `run_card_ickkw0.dat` differs as R1
does:

| control | σ unpatched | σ patched | G4 | G7 |
|---|---|---|---|---|
| R1 grouped, `ickkw = 0` | 104.82 pb | 107.02 pb (+2.1 %) | 24.99 → 26.18 (+4.7 %) | 20.50 → 23.08 (+12.6 %) |

The screen therefore sees H1 where it exists.

**Results** (from `scratchpad/h1/h1-c0-summary.txt`):

```
pp_to_jj: channels 12, differing results.dat lines 0; events 2000, differing event lines 0; sigma unp [0.68059E+09 0.25144E+07] pat [0.68059E+09 0.25144E+07]
gg_to_gg: channels 2, differing results.dat lines 0; events 2000, differing event lines 0; sigma unp [0.14281E+06 0.25943E+03] pat [0.14281E+06 0.25943E+03]
pp_to_bb: channels 3, differing results.dat lines 0; events 2000, differing event lines 0; sigma unp [0.37577E+09 0.62686E+06] pat [0.37577E+09 0.62686E+06]
pp_to_bb_qcd2: channels 5, differing results.dat lines 0; events 2000, differing event lines 0; sigma unp [0.37575E+09 0.62690E+06] pat [0.37575E+09 0.62690E+06]
gg_to_ttx: channels 2, differing results.dat lines 0; events 2000, differing event lines 0; sigma unp [0.13564E+02 0.36842E-01] pat [0.13564E+02 0.36842E-01]
gg_to_ttx_smlimit: channels 2, differing results.dat lines 0; events 2000, differing event lines 0; sigma unp [0.13588E+02 0.36943E-01] pat [0.13588E+02 0.36943E-01]
gg_to_ttx_smlimit_qcd2: channels 3, differing results.dat lines 0; events 2000, differing event lines 0; sigma unp [0.13489E+02 0.35873E-01] pat [0.13489E+02 0.35873E-01]
pp_ttx_lep_dyn: channels 3, differing results.dat lines 0; events 2000, differing event lines 0; sigma unp [0.58553E+01 0.29834E-01] pat [0.58553E+01 0.29834E-01]
pp_to_llj_dyn: channels 8, differing results.dat lines 0; events 2000, differing event lines 2290; sigma unp [0.41402E+03 0.24851E+01] pat [0.41402E+03 0.24851E+01]
uu_full: channels 12, differing results.dat lines 16; events 2000, differing event lines 2161; sigma unp [0.13678E+02 0.10622E-01] pat [0.13677E+02 0.10622E-01]
uu_tt: channels 12, differing results.dat lines 16; events 2000, differing event lines 10004; sigma unp [0.48565E-01 0.20864E-03] pat [0.48566E-01 0.20865E-03]
r1_grouped: channels 4, differing results.dat lines 16; events 2000, differing event lines 15998; sigma unp [0.10482E+03 0.80061E+00] pat [0.10702E+03 0.88647E+00]
```

- **2 → 2 rows and the decay chain: identical.** This confirms the reading
  that equal-mass swapped legs carry equal p_T and transverse mass, so every
  kT measure is unchanged. It holds on the massive final states
  (`gg_to_ttx`, `pp_to_bb`) and on `pp_ttx_lep_dyn`, whose central t t̄ scale
  is the symmetric `mt2last`.
- **`pp_to_llj_dyn`: identity permutations, mirrored processes. The mirror
  control is identical in everything H1 can reach.** That covers every
  channel's `results.dat`, and every event line's weight, SCALUP, AQEDUP and
  AQCDUP. The 2290 differing event lines differ only in p_z and E: every
  other column and the event headers are identical. The worst relative
  difference over all 65455 numbers is 3.4e-5, and it is typically 1e-9.
  Handing the first call the mirrored `P1` changes the last digits of the
  written longitudinal momenta. This is cosmetic, and the cause was not
  isolated.
- **`uu_full` and `uu_tt`: H1 is present and negligible.** Only the channels
  holding permuted configs move. `G1` holds configs 1 and 9; `G3`, `G5` and
  `G7` hold 3/4, 5/6 and 7/8. The dominant channels are identical in both
  copies.
  - `uu_full`: `G1` reads 0.020590 → 0.020180 pb. All channel shifts sum to
    −4.1e-4 pb on 13.678 pb, **−3.0e-5 of σ**. The permuted channels hold
    0.15 % of σ.
  - `uu_tt`: `G1` reads 1.2217e-4 → 1.2285e-4 pb. The sum is +7.1e-7 pb on
    0.048565 pb, **+1.5e-5 of σ**. The permuted channels hold 0.28 % of σ.
  - This bound needs no statistics: H1 acts only on points of the permuted
    channels, so even a 100 % shift of those channels moves `uu_tt` by at
    most 0.28 %. **H1 cannot be the ~2 % by which MadEvent reads below this
    side on `uu_tt`.** That result goes to sibling PR #28.

## Files

- `validation/madgraph/census_permuted_first_call.py` (committed): the `runs`
  and `perms` census.
- Scratch, uncommitted, all under
  `/tmp/claude-0/-home-user-vibegraph/e8f0bc03-01f3-5b41-b6ee-7a04856445c0/scratchpad/h1/`:
  - `h1-gen.sh`: phase-B generation.
  - `h1-c0.sh` and `h1-cmp.sh`: the C0 pair and its comparison.
  - `h1-perms-all.tsv`: the full permutation table.
  - `h1-c0-summary.txt`, plus `c0/*.log` and `c0/*.channels`.
  - `h1-c0-channels-permuted.txt`: the per-channel tables of the three
    differing pairs.
- The process directories were deleted after reading.

## Found

- **`validate_scales` cannot see H1, by construction.** `replay()`
  (`vibegraph-lib/tests/validate_scales.rs:798`) searches configurations until
  one reproduces `SCALUP`. MadEvent's first-call scale on `PP` under config c
  equals this crate's clustering of `P1` under c's permuted twin, and that
  twin is in the searched set. The replay is also blind to first-call
  rejections. Its "what this cannot see" list should say so. This census found
  no row where the blindness hides a shift, but the next census should not
  read a full replay pass as H1 evidence.
- **Correction to the backlog item and the sprint page.** The exposure
  criterion should read `ickkw > 0`, or `xqcut > 0`, or
  (`dynamical_scale_choice = -1` and some scale not fixed), and it should add
  the condition that σ can move: PDFs or an α_s coupling. The item's
  `closes_when` is met by this report: every banked row is checked, and the
  affected rows are `pp_to_ll_0j2j_mlm` (registered),
  `pp_to_ttx_0j1j_mlm` (#23: negligible) and `uu_full`/`uu_tt` (negligible,
  bounded above).
- **The first-call argument perturbs written longitudinal momenta at
  1e-9.** Seen on `pp_to_llj_dyn`'s mirrored events when the first call is
  handed `P1`. It is cosmetic and was not isolated. It matters only to whoever
  next compares patched and unpatched event files at bit level.
