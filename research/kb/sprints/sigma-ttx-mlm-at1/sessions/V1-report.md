---
type: Session Report
title: "V1 report: reference side and per-event share of the ttx @1 excess"
description: "The +1.28 % on @1 is not the reference (20 fresh directories agree with the bank) and not H1 (+0.03 ± 0.11 pb); it is +1.6 % on g g > t t~ g and +1.0 % on g q > t t~ q, appears only under matching, and mostly disappears with pdfwgt = F. Not yet attributed to a line of code."
status: draft
tags: [mlm, sigma, ttx, h1, madevent-reference, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources: []
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
---
This report is from the validation-dev agent. It is a diagnosis only. No
source change was committed, and the probes were reverted.

Every MadEvent number here comes from the pinned MadGraph 3.7.1, run through
`validation/madgraph/mg5_pinned.sh` under `pixi run -e madgraph`. Each run
checks it: `SubProcesses/MGVersion.txt` reads `3.7.1`, the LHE `<init>` reads
`PDFSUP = 247000 247000`, and the banner reads `vector_size = 1`. No 3.5.7
result was ever produced. The patched runs use a copy of the pinned tree with
`first-call-unpermuted-momenta.patch` applied (`VG_MG5_ROOT`), and every
generated `auto_dsig.f` was checked to read `CALL UPDATE_SCALE_COUPLING(P1, WGT)`.
Each seed runs in a fresh `cp -r` of a directory that was generated but never
run, which is the pattern `repro/permuted_first_call/repro.sh` uses. MadEvent
ran on 2 cores and vibegraph on 2 threads (`-j 2`). Scripts, logs and LHE
files are under the scratchpad, at
`/tmp/claude-0/-home-user-vibegraph/e8f0bc03-01f3-5b41-b6ee-7a04856445c0/scratchpad/ttx/`
(`ttx-results.txt`, `ttx-*.log`, `ttx-lhe/`).

**Budget.** A full banked-budget run of the mixed card (`nevents = 10000`)
takes 25–37 s on 2 cores here, against 19–26 s on the M3 bank host. That is
cheap enough to keep the banked budget and run 20 seeds per configuration,
plus a 100 000-event rung.

## Answer

| part | banked MadEvent (10 seeds, 9 shared dir) | fresh-dir MadEvent (20) | fresh + patched (19) | this side (banked, 10 seeds) |
|---|---|---|---|---|
| `@0` | 512.898 ± 0.181 | 512.997 ± 0.129 (χ²/dof 0.78) | 512.997 ± 0.133 (0.77) | 513.222 ± 0.177 |
| `@1` | 575.836 ± 0.777 | 575.290 ± 0.520 (0.86) | 575.409 ± 0.547 (0.97) | 583.192 ± 0.297 |
| total | 1088.640 ± 0.797 | 1088.295 ± 0.536 (0.88) | 1088.400 ± 0.563 (1.00) | 1096.413 ± 0.381 |

Each MadEvent column is the unweighted mean with error max(rms of quoted errors/√n,
spread/√n), seeds 20261201–20, and the χ²/dof is over seeds. Against the
fresh directories, this side's `@1` is +7.90 pb, which is +1.37 % and 13σ.

1. **The reference is not the cause.** The 20 independent directories agree
   with the banked shared-directory reference: `@1` differs by −0.55 ± 0.94 pb.
   The bank can stay as it is. No banking request follows from this session.
2. **H1 is negligible.** Patched and unpatched runs share each seed, so the
   points of each run's first iteration are the same, and the per-seed
   difference measures H1 directly. On `@1` it is +0.033 ± 0.111 pb, with a
   seed sd of 0.48. On `@0` it is +0.011 ± 0.012. By directory:
   `P1_gg_ttxg` +0.09 ± 0.10, `P1_gq_ttxq` −0.06 ± 0.04, `P1_qq_ttxg` 0.000.
   The only non-identity permutation on this row is t ↔ t̄ (`symperms.inc`:
   legs 3 ↔ 4 on configs 3, 7, 8, 9, 11, 14 of `gg_ttxg` and config 3 of `gq`,
   `qq` and `P0_gg`). The clustering is symmetric under it.
3. **The excess is on this side's matched `g g` and `g q` groups.**

| `@1` group | MadEvent fresh (20) | vibegraph alone (5 seeds, 200k × 8 neyman) | Δ | rel | pull |
|---|---|---|---|---|---|
| `gg > ttxg` | 409.297 ± 0.448 | 415.993 ± 0.247 | +6.70 | +1.64 % | 13.1 |
| `gq > ttxq` (+ mirrors, q̄) | 147.138 ± 0.282 | 148.541 ± 0.072 | +1.40 | +0.95 % | 4.8 |
| `qq̄ > ttxg` | 18.855 ± 0.045 | 18.909 ± 0.011 | +0.05 | +0.29 % | 1.2 |
| sum / `p p > t t~ j` alone | 575.290 ± 0.520 | 583.560 ± 0.433 | +8.27 | +1.44 % | |

   The `p p > t t~ j`-alone integration, 583.56, reproduces the composite's
   `@1` of 583.19, so integrating each group alone is faithful. The group sum,
   583.44, closes on it. MadEvent's per-directory values come from summing
   `G*/results.dat` per `P*` directory.

## Localisation (all on `g g > t t~ g` alone, both sides on the same card, 5 seeds each)

| card variant (row card except) | MadEvent | vibegraph | rel | pull |
|---|---|---|---|---|
| fixed μ = 173, `ickkw = 0`, `xqcut = 0`, `ptj = 30` | 347.544 ± 0.45 | 348.357 ± 0.229 | +0.23 % | 1.6 |
| dynamical −1, `ickkw = 0`, `xqcut = 0`, `ptj = 30` | 296.188 ± 0.383 | 297.414 ± 0.167 | +0.41 % | 2.9 |
| dynamical −1, `ickkw = 0`, `xqcut = 30` (pure cut) | 296.264 ± 0.406 | 297.272 ± 0.160 | +0.34 % | 2.3 |
| **row card** (`ickkw = 1`, `xqcut = 30`) | 410.316 ± 0.53 (10k ev), 409.887 ± 0.40 (100k ev, 3 seeds) | 415.993 ± 0.247 | +1.38 % / +1.49 % | 9.7 / 13 |
| row card, `xqcut = 60` | 170.268 ± 0.23 | 172.572 ± 0.098 | +1.35 % | 9.2 |
| row card, **`pdfwgt = F`** | 359.078 ± 0.511 | 360.356 ± 0.205 | +0.36 % | 2.3 |

- The excess needs matching. Unmatched, with the same cuts, the same
  dynamical clustering scale and the `xqcut` cut, the sides agree to within
  0.2–0.4 %. That is the size of the generic offset already filed for 2 → 4.
- With matching on, the gap is about +1.4 %. Turning off only `rewgt`'s
  density reweighting (`pdfwgt = F`, which also returns the matrix element's
  densities to the central scale) brings it back to +0.36 %. So about
  1.0–1.1 % of the 1.4 % lives in the `pdfwgt` branch: the first call's
  per-beam density scale and `rewgt`'s density ratios.
- Raising MadEvent's budget does not move it: 10 000 events give 410.32 and
  100 000 give 409.89. It also does not move with `xqcut`: +1.35 % at 60.
- **Phase space.** Compare unweighted shapes: 100k vibegraph events against
  300k MadEvent events, both `gg` matched. The excess concentrates at the
  larger beam momentum fraction:

  | x_max bin | ratio, `pdfwgt = T` | ratio, `pdfwgt = F` |
  |---|---|---|
  | 0.3–0.5 | 1.076 ± 0.015 | 1.016 ± 0.022 |
  | 0.5–1 | 1.10 ± 0.06 | 1.00 ± 0.08 |

  The ratios are dσ_vg/dσ_MG, against a total of 1.013 and 1.0036
  respectively. The high-m_tt̄ tail (> 1 TeV, +10–30 %) is there with either
  setting. That is the generic tail offset, not this.
- MadEvent was then run on `max(x1, x2) > 0.3` alone, through a user cut in
  `dummy_fct.f` (3 seeds): 31.23 ± 0.15 pb. Its inclusive run's share of that
  region is 30.65 ± 0.21, and vibegraph's is 33.05 ± 0.37. The dedicated run
  recovers +0.6 pb, which is coverage of MadEvent's own. vibegraph still sits
  1.8 pb (+5.8 %) above it.

**Ruled out, and how.**
- **The reference directory.** The fresh directories agree with the bank
  (table above).
- **H1.** It was measured directly, paired seed by seed.
- **MadEvent budget.** Going from 10k to 100k events does not move it.
- **`rewgt`'s density-denominator kill** (`pdfj2 < 1e-10`, "Too low scale for
  heavy quark"). An instrumented `reweight.f` printed every kill: there were
  0 over a full matched `gg` run. A no-kill copy reads 410.52 ± 0.54,
  unchanged.
- **The t-channel orientation of the channel forests.**
  `coupling/cluster/configs.rs` derives forests that differ from MadGraph
  3.7.1's `configs.inc` on 3 of 15 `gg_ttxg` configurations and 2 of 5
  `gq_ttxq`. MadGraph's `reorder_tchannels` flips them (`TSTRATEGY = 1`,
  `export_v4.py:5569`). Generated with `--t_strategy=2`, MadGraph's forests
  equal vibegraph's exactly (a probe compared canonical forests). Even so, the
  MadEvent σ does not move: `P1_gg_ttxg` reads 409.31 ± 0.62 over 10 seeds and
  `@1` 575.58 ± 0.73. The orientation is real but σ-inert. The clustering
  treats the two rootings alike.
- **`|M|²` at fixed momenta.** This is PR #29's evidence: every `@1` class
  agrees per flow and per helicity, with MadEvent's width patterns.

**What the evidence chain says, and its blind spot.** `validate_mlm_dumps`
agrees, factor by factor, on every written MadEvent event. The integrals
nevertheless differ by 1.4 %, and the difference sits in the `pdfwgt` branch
at large x. Two explanations remain:

- **(a)** vibegraph's weight differs from MadEvent's in a region where
  MadEvent writes almost no events.
- **(b)** the dump gate's production path is not the integrand's path.

(b) is concrete. The dump test clusters each event with MadGraph's own
forests (`configs.inc`) in MadEvent's own integration channel (`iconfig`),
for the flavour combination MadEvent drew (`validate_mlm_dumps.rs:1194`,
`Directory::load`). The integrand does none of that:

- it clusters with forests derived from its own diagrams;
- it uses a configuration it draws ∝ `AMP2` (`proton.rs`, `per_group_sum` and
  `scale_channel`);
- it averages `rewgt` over the group's members by luminosity
  (`member_rewgts`).

None of those three steps is compared against MadEvent per event. The
falsifier for "the integrand's weight equals MadEvent's per point" is a
per-point comparison on *vibegraph's* events in the `x_max > 0.3` region: the
integrand's group term against an instrumented MadEvent `DSIG1`, with
`SUBDIAG(1) = 0` so that `SMATRIX1` returns the plain `|M|²`. That comparison
was not reached in this session. It needs a probe-only, uncommitted fixed-point
entry into `ProtonIntegrand` (`per_group_sum` at a given `x1`, `x2`, momenta
and configuration) and a small MadEvent driver (`setrun`, `setpara`, `setcuts`,
then `PASSCUTS`, `UPDATE_SCALE_COUPLING` and `DSIG1` per point). Neither lives
in `helas/`.

## Gates

None run. No source change was committed. The two probe tests (a per-group
split appended to `validate_hadronic.rs`, a forest comparison appended to
`validate_kt_cluster.rs`) were reverted with `git checkout`. Before the
commit, `git diff --stat` showed only the manager's own uncommitted edit to
`V1.md`.

## Brief corrections

- Step 3 cannot be done "from the dump" on this host. The MLM dumps live in
  the gitignored work area and are not in `refdata-9`, so `validate_mlm_dumps`
  cannot run here. H1's per-event share was measured instead by pairing seeds
  inside MadEvent. That covers rejected points, because they are integrated
  over, but it cannot name individual points.
- "Localise from the per-group σ both sides report": this side reports no
  per-group σ. The composite's channels are not groups, because every channel
  integrates every group, and the probe summed by `ChannelId::group` only
  recovers the per-multiplicity total. Per-group σ needs one integration per
  group card, as done here.

## Found

1. **Matched `gg` / `gq` excess sits in the `pdfwgt` branch.** The integrand
   reads +1.4 % over MadEvent on matched `g g > t t~ g` and +1.0 % on
   `g q > t t~ q`. `pdfwgt = F` removes about three quarters of it. The excess
   concentrates at x_max > 0.3, where it is +5.8 % against MadEvent's
   dedicated run. To close it, compare per point on vibegraph's events: the
   integrand's group term against an instrumented MadEvent `DSIG1`, as
   described above.
2. **The dump gate's production path is not the integrand's.**
   `validate_mlm_dumps` checks `cluster_history` and `rewgt` with MadGraph's
   forests, MadEvent's `iconfig` and MadEvent's flavour draw. The integrand
   uses derived forests, an `AMP2` draw and a luminosity-weighted member
   average. A gate that replays MadEvent's events through the integrand's own
   path would close the blind spot: the same forests and the configuration
   draw, compared against the dumped weight.
3. **Derived forests differ from MadGraph 3.7.1's `configs.inc`.** MadGraph
   flips the t-channel chain per configuration (`reorder_tchannels`,
   `t_strategy = 0` default; `TSTRATEGY = 1` on configs 2, 3 of `gq_ttxq` and
   three of `gg_ttxg`). The rootings disagree on those configurations. No
   σ effect was measured on this row. Any forest-level gate against 3.7.1
   output (`DERIVED_FOREST_RUNS` holds no row with two massive or mixed
   t-channel propagators) would see it. Either reproduce the reordering, or
   compare up to rooting.
4. **`gq` mirror density scale in the generated `DSIG1`.** For beam 1 it tests
   `Q2FACT(IB(1))` and then reads `Q2FACT(1)`
   (`P1_gq_ttxq/auto_dsig1.f`, the `QSCALE` block). Under `pdfwgt` the two
   beams' scales differ, so the mirrored ordering may read the other beam's
   density scale. This has not been checked against this side. It could carry
   part of `gq`'s +1.0 %, and per the binding rule it is not to be "fixed" here
   if it is MadEvent's.

The work left over is a hand-off, as the manager allowed. No banking request
follows: the reference stands.
