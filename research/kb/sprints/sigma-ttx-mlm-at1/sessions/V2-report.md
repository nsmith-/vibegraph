---
type: Session Report
title: "V2 report: per-point integrand against MadEvent DSIG1 in the pdfwgt branch"
description: "The integrand clusters each matched term in the lab frame, and MadEvent clusters in the partonic CM frame. The beam tie-break in cluster.f reads the sign of p_z, so about 16 % of g g > t t~ g points cluster their gluon onto the other beam. Clustering in the CM makes 26 000 points agree with an instrumented DSIG1 on every factor (worst 3e-7), and lowers gg by 1.05 % and gq by 0.09 %. A residual of about +0.6 % on gg and +0.9 % on gq remains. It is not in the per-point integrand."
status: draft
tags: [mlm, sigma, ttx, clustering, frame, per-point-oracle, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources: []
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
---
This report is from the validation-dev agent. It is a diagnosis only. The
only file committed is this report. The probes (a `#[doc(hidden)]` fixed-point
entry in `proton.rs`, an environment toggle and a test driver) were reverted,
and a copy is kept in the scratchpad.

Every MadEvent number comes from the pinned 3.7.1 trees that V1 generated
(`MGVersion.txt` reads `3.7.1`). They were compiled under the banked row card
(`pp_to_ttx_0j1j_mlm_run_card.dat`, lhaid 247000, `vector_size = 1`) by a
500-event `generate_events`, run through `mg5_pinned.sh` under
`pixi run -e madgraph`. All of the scratch is under
`/tmp/claude-0/-home-user-vibegraph/e8f0bc03-01f3-5b41-b6ee-7a04856445c0/scratchpad/`:

- `ttx2-drv/`: the drivers, the point extractor, the comparators and the probe diff.
- `ttx2-me-gg/`, `ttx2-me-mix/`: the compiled trees, with `ptdriver` built in `P1_gg_ttxg` and in `P1_gq_ttxq`.
- `ttx2-pts/`: the points, both sides' dumps and the `*.cmp*.json` tables.

## Verdict

**The defect is on this side. The integrand clusters in the wrong frame.**
`ProtonIntegrand::per_group_sum` (`proton.rs`) clusters each group's direct and
mirrored terms through `term_at(sc, &lab, …)`, so it uses **lab-frame**
momenta. MadEvent clusters the momenta that `DSIGPROC` hands to
`update_scale_coupling`, which are `PP`/`P1` in the **partonic CM frame**. On
those momenta, `rap()` adds `cm_rap` itself.

Most of the clustering is invariant under the longitudinal boost (p_T, m_T,
kT), but one rule is not. When cluster.f chooses which beam an initial-state
emission merges with, it reads the sign of p_z:

```fortran
c     prefer clustering when outgoing in direction of incoming
                     if(sign(1d0,pcl(3,idi)).ne.sign(1d0,pcl(3,idj)))
     $                    pt2ij(idij)=pt2ij(idij)*(1d0+1d-6)
```

`djb` is the same for both beams, so this 1e-6 factor decides between them.
At large x_max the partonic system is strongly boosted. The gluon's p_z often
changes sign between the CM and the lab, and the integrand then clusters the
emission onto the other beam. That swaps which beam gets the ISR scale and
which gets the hard scale. It also changes μR (the hard scale is recomputed
after a different recoil), α_s(μR), |M|²(α_s), the α_s and PDF factors of
`rewgt`, and the beam densities.

This is what V1's explanation (b) predicted: the production path and the
dump-gate path are different. `validate_mlm_dumps` replays MadEvent's dumped
`PP`/`P1`, which are CM momenta, so it cannot see the integrand's lab frame.
The `ScaleEvent` doc in `coupling/scales.rs` ("in the lab frame — the
clustering's beam measure is a transverse mass about the beam axis") states
the frame-invariance assumption. Nothing tested it, and it is false for this
tie-break.

## Evidence chain

### 1. Instrumented DSIG1

The driver is `ttx2-drv/ptdriver.f` for gg and `ptdriver_gq.f` for gq, linked
against the compiled P directory's objects and `matrix1_optim.o`. It calls
`setrun`, `setpara`, `setcuts` and `rmarin`. Then, per point, per
configuration `c` of `mapconfig`, per subprocess and mirror (gq), and per
`IPSEL` (gq), it calls:

1. `PASSCUTS(PP)`, then `UPDATE_SCALE_COUPLING(PP)`.
2. `SMATRIX` twice, with `MULTI_CHANNEL` false (plain |M|²) and true (the SDE share α_c).
3. `DSIG(PP, WGT, 5)` for |M|² · α_c · `rewgt`.

From the `TO_SYST` common it reads the α_s factor of `rewgt`, which is
Π α_s(alpsfact · s_qalps)/α_s(μR). The PDF factor is `rewgt` divided by that.
A one-line scratch patch to `auto_dsig1/2.f` exposes `PD(1:4)` and forces
`IPSEL`, so that each flavour member's `rewgt` is read.

Each configuration is evaluated once before the measured pass, so that
`njetstore` is filled as in a running job. The integrand at a point is then
W_MG = Σ_c α_c · PD · |M|² · `rewgt`, which is what MadEvent's multichannel
sum integrates. The symmetric t ↔ t̄ configurations are evaluated at the
point itself, which is the H1-patched semantics (V1 measured H1 as null).

**Mirror orientation.** For `IMIRROR = 2` at a physical point (x1, x2,
finals f), the driver evaluates the sampled point PP = (beams, R(f)) with
`XBK = (x2, x1)` and `cm_rap → −cm_rap`. `DSIGPROC`'s P1 = R(PP) then has the
subprocess's first parton on the −z beam and the finals f. `IB(1) = 2` was
confirmed on every mirrored row.

### 2. Fixed-point entry on this side

The probe was a `#[doc(hidden)]` `ProtonIntegrand::probe_point(x1, x2, out_cm, v)`.
It runs `build_frames`, the cuts and `per_group_sum` exactly as the integrand
does, and returns each group's AMP2, scales, member `rewgt`, the `rewgt` α_s
factor, |M|² at the term's coupling, and the term with and without `rewgt`. The
configuration draw was integrated exactly: v was placed at each
configuration's CDF midpoint, so W_vg = Σ_c P_c · term_c.

The test (`ttx2-drv/zz_ttx_point_probe.rs`) builds the integrand the way
`vibegraph integrate` does. The `VG_PROBE_CM` toggle made `per_group_sum` pass
`cm` instead of `lab` to `term_at`, for both orderings.

### 3. Points and results

`ttx2-pts/*.cmp.txt`, with the tables in `*.cmp.json`.

| sample | points | frame | points differing (\|W_vg/W_MG − 1\| > 1e-6) | worst \|rel\| over μF1, μF2, μR, α_s, \|M\|², rewgt α_s part, rewgt PDF part, luminosity |
|---|---|---|---|---|
| vg `gg` events, x_max > 0.3 | 400 | lab | 107 | μF 12.6, PDF part 0.63, luminosity 1.8 |
| vg `gg` events, x_max < 0.1 (control) | 200 | lab | 15 | μF 5.6 |
| vg `gg` events, unbiased | 20 000 | lab | 3 330 (16.6 %) | |
| MadEvent `gg` events (V1's mixts2 LHE), unbiased | 20 000 | lab | 3 135 | |
| the four samples above | 40 600 | **CM** | **0** | ≤ 2e-10 (\|M\|²), ≤ 1e-14 the rest |
| MadEvent `gq` events, x_max > 0.3 / < 0.1 / unbiased | 400 / 200 / 3 000 | lab | 39 / 12 / 238 | μF 10.9 |
| the same, all 2 subprocesses × 2 orderings × 4 members | 3 600 | **CM** | **0** | ≤ 3e-7 (\|M\|²), ≤ 2e-14 the rest |

Every per-term check agrees in the CM frame on all 26 000 points: the cuts,
both μF, μR, α_s, |M|², each member's luminosity, `rewgt` and its α_s and PDF
parts. The configuration does not matter on either side. MadEvent's scales
and `rewgt` are the same for all 15 `gg` configurations at every point, and so
are this side's. The integrand differs from MadEvent only through the frame.

**Mechanism count** (20 000 vg `gg` points): 3 197 points differ and flip the
gluon's p_z sign between CM and lab; 98 flip and agree; 133 differ without a
gluon flip; 16 572 neither flip nor differ. The flip explains the dominant
part. The rest is presumably where a top's p_z or the order of merges decides
the tie, and the CM switch closes those points too.

### 4. Per-point factor table (`gg`, vg events, lab clustering against MadEvent and against CM clustering)

`f1 f2` is x1 f1 · x2 f2 at the term's μF. W is per unit conversion, in f · f (not x f) units.

| pt | x1, x2 | gluon p_z CM / lab | side | μF1 | μF2 | μR | α_s(μR) | ⟨\|M\|²⟩ | rewgt α_s part | rewgt PDF part | x1f1·x2f2 | W |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 0.0050, 0.3135 | +58.3 / −30.7 | MG | 35.207 | 209.411 | 209.411 | 0.115098 | 9.364447e-03 | 1.330326 | 1.328002 | 1.035694 | 10.83102 |
| | | | vg lab | 206.099 | 35.207 | 206.099 | 0.115333 | 9.421974e-03 | 1.327613 | 0.692964 | 1.946955 | 10.66790 |
| | | | vg CM | 35.207 | 209.411 | 209.411 | 0.115098 | 9.364447e-03 | 1.330326 | 1.328002 | 1.035694 | 10.83102 |
| 16 | 0.3564, 0.0196 | −5.8 / +179.5 | MG | 459.281 | 95.375 | 459.281 | 0.104575 | 7.732335e-04 | 1.234434 | 0.991204 | 0.2115308 | 0.02865024 |
| | | | vg lab | 95.375 | 483.555 | 483.555 | 0.103952 | 7.594806e-04 | 1.241840 | 1.030093 | 0.3246063 | 0.04514701 |
| | | | vg CM | 459.281 | 95.375 | 459.281 | 0.104575 | 7.732335e-04 | 1.234434 | 0.991204 | 0.2115308 | 0.02865024 |
| 20 | 0.0187, 0.3023 | +55.6 / −2.4 | MG | 32.077 | 399.920 | 399.920 | 0.106289 | 1.425795e-02 | 1.466080 | 1.441372 | 0.4051227 | 2.162842 |
| | | | vg lab | 387.999 | 32.077 | 387.999 | 0.106671 | 1.441234e-02 | 1.460826 | 0.560570 | 0.7949295 | 1.662412 |
| | | | vg CM | 32.077 | 399.920 | 399.920 | 0.106289 | 1.425795e-02 | 1.466080 | 1.441372 | 0.4051227 | 2.162842 |
| 40 | 0.5594, 0.0049 | −143.1 / +146.9 | MG | 196.630 | 97.930 | 196.630 | 0.116039 | 3.622373e-03 | 1.107872 | 1.059678 | 0.05082859 | 0.07874737 |
| | | | vg lab | 97.930 | 176.148 | 176.148 | 0.117766 | 3.786477e-03 | 1.091631 | 0.979812 | 0.06926100 | 0.1021913 |
| | | | vg CM | 196.630 | 97.930 | 196.630 | 0.116039 | 3.622373e-03 | 1.107872 | 1.059678 | 0.05082859 | 0.07874737 |
| 0 (hi, same sign) | 0.0215, 0.3642 | +221.6 / +14.5 | MG = vg lab = vg CM | 97.110 | 428.100 | 428.100 | 0.105439 | 9.382284e-04 | 1.220864 | 1.008575 | 0.1821555 | 0.02683409 |
| 400 (lo control) | 0.0494, 0.0363 | −103.4 / −86.5 | MG = vg lab = vg CM | 183.103 | 56.505 | 183.103 | 0.117147 | 6.496946e-03 | 1.200906 | 1.097018 | 5.365282 | 25.61567 |

The first factor that differs is μF per beam: the ISR scale lands on the other
beam. All the rest follow from it. With matching, the PDF part of `rewgt`
moves most, by 0.69 against 1.33 at point 1, which is why V1's `pdfwgt = F`
removed most of the gap. The α_s factor moves by at most 4–7 %.

### 5. Fix arithmetic (σ)

These runs used the same CLI, the same seeds and the same budget as V1
(200 000 × 8, `--allocate neyman`, `-j 2`). The only change was the toggle
(`VG_PROBE_CM=1`). Without the toggle, the probe build reproduces V1's seed
20260928 exactly (416.684494). The paired shifts are per seed.

| group | lab (V1, 5 seeds) | CM (5 seeds) | paired shift | MadEvent fresh (V1, 20) | residual |
|---|---|---|---|---|---|
| `gg > ttxg` | 415.993 | 411.645 ± 0.25 | **−4.347 ± 0.067 pb (−1.045 %)** | 409.297 ± 0.448 | +2.35 pb (+0.57 %), pull 4.6 |
| `gq > ttxq` | 148.541 | 148.414 ± 0.073 | **−0.126 ± 0.019 pb (−0.085 %)** | 147.138 ± 0.282 | +1.28 pb (+0.87 %), pull 4.4 |

In the CM frame, seeds 20260928 to 20260932, gg reads 412.228241,
411.985377, 410.788886, 411.791208 and 411.433744 pb, and gq reads
148.471915, 148.291106, 148.205141, 148.613728 and 148.489364 pb.

On `@1` the expected shift is about −4.5 pb (qq̄ is untested but tiny). That
is about 57 % of V1's +7.90 pb against the fresh directories. `@1` would read
about 578.7 against 575.29 ± 0.52, which is +0.6 %.

**Per-point estimators, a cross-check that is not fully consistent.** On
20 000 points:

- E_vg[W_MG/W_vg] = 0.99316 ± 0.00051, which implies vg is +0.69 % high.
- E_MG[W_vg/W_MG] = 1.01132 ± 0.00060, which implies +1.13 %.

The paired integration, at +1.05 %, sits between them. If both event samples
followed their own integrands, the product of the two estimators would be 1.
It is 1.0044 ± 0.0008, so at least one sample does not follow its integrand
at the 0.4 % level, correlated with the region where the frames disagree.
V1 measured MadEvent's under-coverage at x_max > 0.3 (+2 % from a dedicated
run). That is the likelier cause, but this session did not separate it from
the vg sample. The paired integration is the number to use.

### 6. The `gq` beam-scale reading (V1's Found 4): not a defect, and nothing to record

`DSIG1` tests `Q2FACT(IB(1))` but reads `Q2FACT(1)`. Under `IMIRROR = 2`,
`update_scale_coupling` clusters `PP`, which is the subprocess's own
orientation: its first parton is on +z. So `Q2FACT` is indexed by the
subprocess's parton, while `XBK` is swapped into physical-beam order. Reading
`Q2FACT(1)` for parton 1 is therefore right. The mismatched index only changes
which scale the zero test reads, and both scales are nonzero on a clustered
event.

This side swaps the mirrored term's μF back into physical-beam order
(`per_group_sum`). On 3 000 + 600 `gq` points it equals MadEvent per member,
per ordering and per beam to ≤ 1.6e-14. MadEvent never reads a density at the
other beam's scale, and the effect on σ is zero.

## What would falsify the verdict

- A CM-clustered integrand that still disagrees with `DSIG1` on some point.
  None did, over 26 000 points: lab and CM, the gg and gq samples, the
  vg-drawn and MadEvent-drawn samples, high and low x.
- A fixed-seed σ shift from the frame change that does not match the per-point
  share. The paired −1.05 % is between the two per-point estimators. Those
  estimators disagree by 0.44 %, and the difference is attributed to
  sample-coverage, not to the integrand.
- MadEvent clustering in the lab. It does not: `DSIGPROC` passes `PP`/`P1`,
  which are generated in the partonic CM, and the same driver reproduces DSIG1
  bit for bit on the CM inputs.

## Brief corrections

- **"`SUBDIAG(1) = 0` so `SMATRIX1` returns the plain |M|²":** wrong in 3.7.1.
  With `MULTI_CHANNEL` true, `SMATRIX1` multiplies by `AMP2(CHANNEL)/XTOT`
  for any `CHANNEL`, and `AMP2(0)` is out of bounds. The plain |M|² needs
  `MULTI_CHANNEL = .false.` (`COMMON/TO_MATRIX`). The driver does that, and
  takes α_c from a second call.
- **Calling `DSIG1` from a driver needs `RMARIN` first.** `DSIG1` draws
  helicity, colour and `IPSEL` from `RANMAR`. Without initialization, the
  draws write out of bounds and corrupt `passcuts`'s saved cut arrays (seen as
  `ptjmax4 = 0` and every later point failing).
- **The order "`PASSCUTS`, `UPDATE_SCALE_COUPLING`, `DSIG1`" is right**, with
  one condition for a mirrored term at a physical point: it must start from
  the sampled `PP` = (beams, R(finals)), not from the physical momenta. A
  rotation leaves |M|² unchanged, so a naive driver measures the direct term
  twice.
- **`vibegraph generate` fails on `gq.dat`** (Found 2). The gq points
  therefore come from MadEvent's own `@1` events (V1's mixts2 LHE), not from
  vg events. For a per-point comparison that is equivalent.

## Gates

None. The only file committed is this report. The source was restored with
`git checkout vibegraph-lib/src/proton.rs`, and the untracked probe test was
deleted. Before the commit, `git diff --stat` was empty. `pixi run kb-index`
and `pixi run kb-lint` were run on the bundle.

## Recommendation (for the fix session)

1. In `per_group_sum`, cluster both orderings in the partonic CM frame. For
   the direct term that is `cm`. For the mirrored term, it is
   `mirror_lab_into` applied to `cm`, so that the mirror is still
   R(physical CM), which is what MadEvent's `PP` is.
2. Check every other place that feeds lab momenta to the clustering: the
   records' `ptclus` and `SCALUP` come from the same history, and
   `proton.rs`'s test helper calls `term_at(sc, &lab, …)`.
3. Correct the `ScaleEvent` doc.
4. Expected σ after the fix: `gg` −1.05 %, `gq` −0.09 %, `@1` ≈ −4.5 pb. Two
   things should not move: `@0`, whose 2 → 2 clustering has no
   initial-state emission, and `validate_mlm_dumps`, which already feeds CM
   momenta.
5. Every other matched or clustered-scale row should be re-measured, because
   any row whose clustering can merge an emission with either beam is
   exposed. At low x the effect is small (15/200 points here, net −0.08 %),
   which is why lighter rows may not have shown it.

## Found

1. **The integrand clusters in the lab frame, and MadEvent clusters in the
   partonic CM** (this report's verdict). cluster.f's tie-break
   `pt2ij·(1+1e-6)` reads p_z's sign. Lab clustering flips the beam
   assignment on 16.6 % of `gg > ttxg` points and is worth +1.05 % of σ on
   `gg` and +0.09 % on `gq`. To close it, cluster in the CM in
   `per_group_sum` (both orderings) and re-measure the matched and
   clustered-scale rows. The durable oracle is item 3.
2. **`vibegraph generate` refuses a `p p > t t~ j` (quark-only `j`) artifact.**
   It stops with "the resonance 2 of g u > t t~ u has no flavour in the
   subprocess with outgoing [6, -6, 1] that shares its matrix element". The
   integration succeeded (148.64 ± 0.26 pb), and the run is in
   `ttx2-vg/ttx2-gq-gen.log`. The process has no resonance in the
   physical sense. The resonance record of a mixed-flavour group (u and d
   members sharing a matrix element) looks up the wrong member. To close it,
   reproduce with `ttx2-vg/gq.dat` and the row card, then fix the
   member-resonance mapping.
3. **No gate exercises the integrand's own frame construction.**
   `validate_mlm_dumps` replays MadEvent's CM `PP`/`P1` directly, so a frame
   error in the production path cannot reach it. This is V1's Found 2, now
   shown to hide a 1 % effect. To close it, add a gate that rebuilds each
   dumped event from (x1, x2, CM finals) through the integrand's own path
   (`build_frames`, `per_group_sum`, `term_at`) and compares the term's
   scales and `rewgt` against the dump. Or bank a per-point `DSIG1` table
   from the driver in `ttx2-drv/`. That is new banked reference data, so the
   user produces it on the M3 host.
4. **A residual excess remains after the fix: gg +0.57 % (pull 4.6) and gq
   +0.87 % (pull 4.4).** At those points the per-point integrand is exact,
   so the residual is at integration level. It could be this side's
   phase-space measure (flux, Jacobian), the generic 2 → 3/4 offset already
   filed (V1 measured +0.23–0.41 % on unmatched gg), or MadEvent's coverage.
   The estimator product, 1.0044 ± 0.0008, says that at least one event
   sample does not follow its integrand. To close it, take the
   `sigma-2to4-offset` result, run an unmatched gq comparison, and repeat
   V1's dedicated x_max > 0.3 MadEvent run after the fix.
5. **Unexplained 133 / 20 000 points** differ in the lab without a flip of
   the gluon's p_z. They agree in the CM, so the fix covers them, but the
   frame-dependent step they hit is not identified. Candidates are a top's
   p_z, or cluster.f's boost when `nleft > 4`, which is not reached at five
   legs. Worth one look in the fix session.

No banking request follows from this session.
