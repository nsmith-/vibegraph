---
type: Session Report
title: "V1 interim report: H1 census, first-call criterion and run plan"
description: "Which banked grouped MadEvent runs have a clustering first call, which can move with it, which path they took (all scalar), and the plan for the permutation machine-check and the diagnostic MadEvent runs."
status: draft
tags: [h1, madevent-reference, scales, grouping, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
Interim report, written at the manager's phasing point: brief steps 1 and 2
(read-only), plus the plan for step 3. No MadGraph or MadEvent process has been
started. The census below is complete for the card-level questions and still
**pending on its last column**, the machine-checked `symperms.inc` read: the
generation step it needs (MadGraph `output`, no launch) was refused by the
session's permission layer as interfering with the loaded host, so it moves to
the compute slot as phase B.

## 1. When the first call clusters (read, pinned tree 3.7.1)

`update_scale_coupling_vec` (`Template/LO/SubProcesses/reweight.f:1856`) calls
`setclscales` on every point, but `setclscales` returns before clustering
(`reweight.f`, the `ickkw.le.0.and.(xqcut.le.0d0.or.init_mode).and.q2fact>0.and.scale>0`
test) whenever both scales are already set. So the first call clusters, and H1
can exist, exactly when

- `ickkw > 0`, or
- `xqcut > 0` (also at `ickkw = 0`: the xqcut test then still rejects), or
- `dynamical_scale_choice = -1` and not all of `fixed_ren_scale`,
  `fixed_fac_scale1`, `fixed_fac_scale2`.

`dynamical_scale_choice = 1..4` are sums over final-state legs or √ŝ
(`setscales.f`), invariant under a final-state permutation.

A clustered first call moves σ only through what it sets:
- the PDF scales, which need `lpp ≠ 0`;
- μR, which needs an α_s-dependent coupling in the matrix element;
- a rejection: the xqcut / xmtc tests, or the 2 GeV factorisation floor
  (`reweight.f:1206`), which needs `lpp ≠ 0`.

So a fixed-beam run with no α_s-dependent coupling clusters but cannot move.
The `IMIRROR = 2` flip alone negates p_y and p_z of every leg, a rotation by π
about x, under which the kT measures are invariant; only `PERMS` matters.

**The brief's criterion is incomplete in two directions.**
- `xqcut > 0` at `ickkw = 0` also clusters: `pp_to_llj_xqcut_only`.
- `dynamical_scale_choice = -1` with all three fixed flags set does not
  cluster: `pp_to_bb_fixed`, `pp_to_llj_fixed`, `ud_to_epemud_qcd0`, and the
  `dy13_*` cards behind `hadronic_sigma_reference.json` and the `pp_*` rows of
  `onshell_veto_reference.json`.

## 2. Card census (machine-read from every banked banner)

Script: `scratchpad/h1/census_permuted_first_call.py runs` (uncommitted until
phase B tests its `perms` half). It reads `output/*/Events/*/*banner.txt` and
takes the MadGraph version from `<MGVersion>`, the grouping from the proc card
plus the 1 → N rule (`madgraph_interface.py:9259`), and the run-card switches.
The α_s dependence comes from the couplings each `matrix*_orig.f` uses,
looked up in the model's `couplings.py`. In the SM, `GC_10`, `GC_11` and
`GC_12` depend on `G`. In SMEFTsim, `GC_6` and `GC_7` do. The toy models'
couplings do not.

All 72 run directories are `group_subprocesses Auto`, so every 2 → N output is
grouped. All banners are MadGraph 3.7.1, except the six deliberate 3.5.7
runs (`validation/madgraph-oracle-pinning`). Those are `ee_to_mumu_tata_qcd0`'s window, anti-window and three
controls, and `var_sde1`. They are e+e- runs with QCD = 0, so H1 cannot
reach them.

Eleven banners print MadGraph's "UNKNOWN DEVELOPMENT VERSION" warning with
`VERSION 3.7.1` (the `-dev` rows below). They are the cut-5 re-carded rows and
the cut-9 MLM rows, generated from a copy of the pin without `.git`, as the
manifest's cut-9 comment says. This is not a version finding.

| run | MG | final | beams | first call clusters | can σ move | perms (phase B) | note |
|---|---|---|---|---|---|---|---|
| `pp_to_llj`, `pp_to_llj_dyn` | 3.7.1(-dev) | 3 | pp | yes (dyn −1) | yes: PDFs, α_s | pending | gated, rel_tol 0.005 |
| `pp_to_llj_mlm`, `_mlm_alps2` | 3.7.1-dev | 3 | pp | yes (ickkw 1) | yes | pending | long tier, \|pull\| < 3 |
| `pp_to_llj_xqcut_only` | 3.7.1-dev | 3 | pp | yes (xqcut 20, ickkw 0) | yes, incl. rejection | pending | long tier, \|pull\| < 3 |
| `pp_to_ll_0j2j_mlm` | 3.7.1-dev | 2–4 | pp | yes | yes | non-identity (D2) | measured, not re-measured |
| `pp_to_ttx_0j1j_mlm` | 3.7.1-dev | 2–3 | pp | yes | yes | non-identity (t↔t~) | claimed by `sigma-ttx-mlm-at1` |
| `uux_to_epemg`, `ddx_to_epemg` | 3.7.1 | 3 | fixed | yes | yes: α_s¹ | pending | rel_tol 0.01 |
| `gu_to_epemu`, `gux_to_epemux` | 3.7.1 | 3 | fixed | yes | yes: α_s¹ | pending | rel_tol 0.005 |
| `pp_to_jj`, `pp_to_bb`, `pp_to_bb_qcd2` | 3.7.1(-dev) | 2 | pp | yes | yes | pending (likely non-identity) | 2 → 2: see below |
| `pp_to_ll`, `_qcd0`, `_scalefact2` | 3.7.1(-dev) | 2 | pp | yes | yes: PDFs | pending | 2 → 2 |
| `gg_to_gg`, `gg_to_ttx`, `gg_to_ttx_smlimit`, `_qcd2`, `uux_to_uux`, `uux_to_ztg` | 3.7.1 | 2 | fixed | yes | yes: α_s | pending | 2 → 2 |
| `uux_to_mumu`, `uux_to_ttx_4f`, `tata_to_ttx_tensor4f`, the toy rows, every `ee_*` run | 3.7.1 | 2–4 | fixed | yes | **no**: no PDFs, no α_s coupling | n/a | unaffected |
| `bbx_/uux_to_ccx_emmm_qcd0`, `wpwm_to_wpwmz_cw` | 3.7.1 | 3–6 | fixed | yes | no (EW only) | n/a | unaffected |
| `gg_to_gg_cg` | 3.7.1 | 2 | fixed | no (dyn 3) | — | n/a | unaffected |
| `pp_to_bb_fixed`, `pp_to_llj_fixed`, `ud_to_epemud_qcd0`, `dy13_*` | 3.7.1 | 2–4 | — | no (all fixed) | — | n/a | unaffected |
| off-bundle `onshell_veto` `uu_full` / `uu_tt` (`u u~ > w+ b w- b~`) | 3.7.1 | 4 | fixed | yes | yes: α_s | pending, **suspect** | see Found |
| off-bundle `decay_chain_events` `pp_ttx_lep_dyn` | 3.7.1 | 6 | pp | yes | yes | pending (likely t↔t~ chain swap) | σ not gated, SCALUP/AQCDUP info |
| off-bundle grammar, decay-chain e+e-, widths, `pta_window` | 3.7.1 / both | — | fixed or 1 → N | — | no α_s or no clustering | n/a | unaffected |

**2 → 2 rows.** For a 2 → 2 final state, the legs a symmetry permutation can
exchange have equal mass (`IdentifySGConfigTag`, `diagram_symmetry.py:316`,
keys external legs on spin, colour, \|charge\| if colourless, mass and width)
and equal and opposite p_T. So every kT measure (`djb`, and `mT²` of the
central pair) is the same before and after the swap.

The predicted H1 is zero except for `cluster.f`'s 1e-6 crossing inflation.
That is a code reading, not a measurement. `validate_scales` says the same for
`pp_to_jj` ("a 2 → 2 final state gives the clustering no merge to choose").
Phase C tests it at zero variance on `pp_to_jj`.

**Existing per-event gates cannot see H1 on these rows.**
- `validate_scales` replays each event by trying configurations until one
  reproduces `SCALUP` (`replay()`, `validate_scales.rs:798`). MadEvent's
  first-call scale on `PP` under config c is this crate's clustering of
  `P1 = π(PP)` under the permuted twin of c, and that twin is in the searched
  set. So the replay passes with or without H1.
- The kept-event replays are also blind to first-call rejections, which is
  where D2 found most of H1's size.

## 3. The vectorised path

`update_scale_coupling_vec(all_p, …)` (`super_auto_dsig_group_v4.inc:312`) has
the same shape as the scalar path. `DSIG_VEC` calls it on the unpermuted,
unmirrored `ALL_P` before `DSIGPROC_VEC` builds `ALL_P1` (`SWITCHMOM` at
`:959`, the mirror at `:983-984`).

`dsample.f:205` takes the scalar `dsig` when `VECSIZE_USED ≤ 1`. In the scalar
path, `DSIGPROC`'s first call (`:842`) is guarded by `VECSIZE_MEMMAX ≤ 1`.
`VECSIZE_MEMMAX = nb_warp × vector_size` (`banner.py:4765`). `vector_size` is
a hidden parameter defaulting to 1 (`banner.py:4465`).

Machine-read: 25 banners carry `vector_size = 1` and `nb_warp = 1` explicitly.
The rest omit both, which means the default of 1. **Every banked run took the
scalar path, so the one-line patch is the complete fix for all of them.** The
vectorised path stays unmeasured, and no banked row needs it.

## 4. Plan for step 3 (not started)

**Phase B: machine-check the permutations.** This is generation only, with no
MadEvent and no compilation. The pinned MG (`mg5_pinned.sh` under
`pixi run -e madgraph --frozen`) runs `output` for each pending row: the row's
own script up to `output`, with the SMEFT model staged as `build.sh` does.
That makes 23 bundle rows, plus the R1 grouped reproducer as a positive
control, plus `uu_full` and `pp_ttx_lep_dyn`.

From each P directory, read `symperms.inc`, `configs.inc`'s `mapconfig` and
`symfact.dat`, and list the non-identity permutations and the configs they sit
on. The control has to reproduce R1's configs 2, 5, 6 and 8 as `(1,2,3,5,4)`
before any row's result is read.

Cost: one core, about 15–30 min in total; `pp_to_ll_0j2j_mlm` and the decay
chain take about 2–3 min each, the others 10–60 s. Disk is about 1–2 GB, and
the directories are deleted after the read.

**Phase C: MadEvent diagnostics.** These run only on rows where phase B finds
non-identity permutations and the σ can move. They use fresh copies of a
freshly generated directory, `NB_CORE = 2`, and `repro.sh`'s one-line
`P1` edit.

C0, a zero-variance screen:
- Run unpatched and patched at the **same seed**, from one generated
  directory, and compare every channel's `results.dat`.
- If they are identical, H1 is exactly zero on that row, with no statistics
  needed.
- `nevents = 2000` (the screen needs an identical integrand, not precision).
- About 2 × 3–5 min per 2 → 2 or 2 → 3 row.
- `pp_to_jj` runs first: it tests the 2 → 2 immunity argument on a group with
  mixed-flavour swaps.

C1, statistical, only for rows that fail C0:
- Five seeds × {grouped, patched}, one fresh directory each, as in R1.
- The error is the spread over seeds; the shift is read against the row's
  gate tolerance and pull.
- llj family (rel_tol 0.5 %, MadEvent's 0.33 % at 10k events): 10000 events,
  10 runs × 4–6 min, about 50 min per card.
  - The three MLM llj cards share the process. Their perms result carries
    over, but their σ shift does not (xqcut rejections), so each permuted MLM
    card needs its own C1 (about 50 min each).
- Fixed-beam e+e-g family (rel_tol 0.5–1 %): 10000 events, 10 × 2–3 min,
  about 25 min per row.

Expected total:
- If phase B finds only 2 → 2 rows and the off-bundle suspects permuted: about
  30 min (B) plus 10 min (`pp_to_jj` C0).
- Worst case (every 3-body row permuted and failing C0): about 5 h. The order
  would then be the cheapest first: `gu_to_epemu`, `gux_to_epemux`,
  `uux_to_epemg`, `ddx_to_epemg`, then `pp_to_llj_dyn`, `pp_to_llj`, then the
  MLM cards.
- `uu_full` is left to `sigma-uux-wbwb-veto` (see Found) unless the manager
  says otherwise.

My prediction, which is not evidence: the 2 → 3 rows have identity
permutations. Their single subprocesses, and llj's groups, have no two
distinct configurations related by a final-leg swap, since γ/Z → l⁺l⁻ is one
vertex. The refdata-9 dumps of `pp_to_ll_0j2j_mlm` put all 77 of its
permuted events with other first-call scales in `@2`; whether any of its 160
permuted events are `@1` (`p p > e+ e- j`) is not on the record I read. If
phase B confirms it, the only measured row will be the
`pp_to_jj` control, and no banking request will follow.

## Banking request

None yet. It depends on phases B and C.

## Found

- **`uu_full` / `uu_tt` is an H1 suspect.** The onshell-veto `u u~ > w+ b w- b~`
  rows have a clustered first call (fixed beams, `dynamical_scale_choice = -1`,
  α_s-dependent). Their final state has two pairs the symmetry tag treats as
  equal: `b`/`b~` (same colour key and mass) and `w+`/`w-` (colourless, equal
  \|charge\| and mass). The manifest records "MadEvent still reads about 2 %
  below this side and moves down with budget where this side does not, which
  is unexplained". H1 biases grouped MadEvent low, as R1 and D2 show. Phase B
  would show whether its configurations are permuted; a C0 pair would show
  whether H1 moves it. This belongs to `sigma-uux-wbwb-veto`, so relay it
  there.
- **`validate_scales`' channel search makes it blind to H1 by construction**
  (§2). Its "what this cannot see" list does not say so. A doc line there, or
  in the H1 concept's "Open", would stop the next census from reading
  10000/10000 as evidence.

