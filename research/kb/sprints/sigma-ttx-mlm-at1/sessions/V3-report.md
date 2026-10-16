---
type: Session Report
title: "V3 report: the clustered rows on the CM fix, and the ttx @1 residual"
description: "The CM fix moves ttx @1 by −4.465 ± 0.083 pb (paired, 10 seeds), @0 by 0.000 and pp_to_ll_0j2j_mlm by nothing measurable. @1 is left +0.45 % (3.8σ) above MadEvent's equal-weight recombination, and +0.61 % above its reported σ. Recombination removes about a quarter of the residual, all of it on gg. The rest is gg +0.36 % (2.3σ) and gq +0.87 % (4.2σ). The gq part appears without matching and at a fixed scale (+0.53 %), and does not move with budget on either side, so it is neither clustering nor scales."
status: draft
tags: [mlm, sigma, ttx, drell-yan, clustering, iteration-combination, madevent-reference, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources: []
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
---
This report is from the validation-dev agent. It measures and diagnoses only.
The only file committed is this report.

**MadEvent provenance.** Every MadEvent number comes from the pinned 3.7.1,
run through `mg5_pinned.sh` under `pixi run -e madgraph`. No patch was applied
(`VG_MG5_ROOT` was unset). Each run's log line checks `MGVersion.txt` (3.7.1).
For the row runs it also checks the banner's `vector_size = 1`, the LHE's
`PDFSUP = 247000 247000`, and that every `auto_dsig.f` has
`UPDATE_SCALE_COUPLING(PP, WGT)`. Each seed ran in a fresh copy of a directory
that was generated but never run, on 2 cores. For the row, that directory is
V1's `ttx/ttx-pristine-fresh`.

**vibegraph builds.** The binaries are `vibegraph integrate` CLIs, dev profile
(`opt-level = 2`), 2 threads, stripped copies under `ttx3/bin/`:

- `vg-fixed` is built at `6740954`.
- `vg-prefix` is the same tree with `proton.rs`, `hadronic.rs` and
  `coupling/scales.rs` checked out from `1483624`. The files were restored
  afterwards.

The composite runs at the rows' budget: `--fixed-budget --allocate neyman
--neval 200000 --niter 8`, seeds 20260928–37. V1 found the CLI equal to
`mlm_mixed_sigma_row` to every printed digit. Here `vg-prefix` reproduces both
rows' committed 10-seed cell means to every printed digit: ttx @0 513.2199,
@1 583.2445; ll @0 665.2581, @1 268.5922, @2 131.2669, total 1065.117. It also
reproduces V1's per-seed ttx values for seeds 20260928 and 20260929. So the
build and the harness are what the cells measured.

**Scratch.** Everything is under
`/tmp/claude-0/-home-user-vibegraph/e8f0bc03-01f3-5b41-b6ee-7a04856445c0/scratchpad/ttx3/`:

- drivers: `ttx3-me-run.sh`, `ttx3-me-var.sh`, `ttx3-vg-run.sh`, `ttx3-queue*.sh`;
- per-seed results: `ttx3-me-results.txt`, `ttx3-vg-results.txt`;
- every MadEvent run's `results.dat` and `P*/G*/results.dat`: `me-keep/*.tgz`, 55 runs;
- analysis: `ttx3-iter.py` (reported against equal-weight, per P directory) and `ttx3-vg.py` (seed rule, paired shift).

## Answers

1. **`pp_to_ttx_0j1j_mlm` on the fix.** `@0` does not move: the paired shift is
   −0.0007 ± 0.016 pb. `@1` moves by **−4.465 ± 0.083 pb** (−0.77 %). Every
   seed moved between −4.15 and −5.05. V2 predicted about −4.5. After the fix,
   `@1` is still +0.53 % (3.6σ) above the bank and +0.61 % (5.7σ) above 20
   fresh MadEvent directories.
2. **MadEvent's iteration bias is real here, but it is about a quarter of the
   residual.** Per seed, the sum over channels of the equal-weight mean of the
   iterations in `G*/results.dat`, minus the reported σ, over 20 fresh seeds:
   - `@1`: +0.87 ± 0.44 pb (+0.15 %);
   - `P1_gg_ttxg`: +0.88 ± 0.37;
   - `P1_gq_ttxq`: −0.005 ± 0.23;
   - `P1_qq_ttxg`: +0.002 ± 0.04;
   - `@0`: +0.10 ± 0.11.

   Against the recombined MadEvent, `@1` reads +2.62 pb (+0.45 %, 3.8σ). The
   gg residual falls from +0.57 % to +0.36 % (2.3σ). The gq residual stays at
   +0.87 % (4.2σ). The recombination does not close the residual.
3. **The gq residual is not matching, not clustering and not the scale.** On
   `g q > t t~ q` alone, without matching (`ickkw = 0`, `xqcut = 0`,
   `ptj = 30`), vibegraph is +0.66 % (4.1σ) above MadEvent's equal-weight
   value at the dynamical clustering scale. It is +0.53 % (3.5σ) above at a
   fixed μ = 173. Neither side's budget moves it: MadEvent at 100k events
   against vibegraph at 800k × 8 still reads +0.59 % (9σ). The unmatched gg
   control agrees after recombination (+0.07 %, 0.6σ). Stopped here, as the
   brief says. Localising further needs per-point checks on vibegraph-drawn
   gq points (see Found).
4. **`pp_to_ll_0j2j_mlm` does not move.** The paired shifts, fixed minus
   pre-fix, are:
   - `@0`: +0.008 ± 0.018 pb;
   - `@1`: −0.007 ± 0.023;
   - `@2`: +0.32 ± 0.31;
   - total: +0.33 ± 0.29.

   The `@2` per-seed shifts scatter by ±1.6 pb, because the changed weights
   steer VEGAS's adaptation, so the mean shift is consistent with zero at
   0.25 %. On the fix, the row still passes its gate on every part: pulls 0.82,
   1.13, 1.08, 1.90, and χ²/dof 0.71, 0.97, 1.18, 0.84, inside 0.13–3.10.

## Table

Columns:

- **pre-fix, fixed:** this side, 10 seeds (20260928–37), seed rule mean ± max(rms quoted/√n, sd/√n).
- **MadEvent reported:** 20 fresh pinned directories (seeds 20261201–20) for ttx, the bank's 21 directories for ll.
- **MadEvent equal-weight:** the same ttx directories, each channel's iterations re-combined with equal weight.
- **pulls:** fixed against the bank / against the reported value / against the equal-weight value.

| row | part | pre-fix | fixed | shift (paired) | MadEvent bank | MadEvent reported (fresh) | MadEvent equal-weight | rel (fixed vs eq) | pull bank / rep / eq |
|---|---|---|---|---|---|---|---|---|---|
| ttx | `@0` | 513.2199 ± 0.1753 | 513.2192 ± 0.1794 | −0.0007 ± 0.0161 | 512.8748 ± 0.1805 | 512.9967 ± 0.1288 | 513.0979 ± 0.1469 | +0.02 % | 1.35 / 1.01 / 0.52 |
| ttx | `@1` | 583.2445 ± 0.3015 | 578.7796 ± 0.3273 | **−4.4649 ± 0.0833** | 575.7170 ± 0.7880 | 575.2904 ± 0.5200 | 576.1639 ± 0.6047 | +0.45 % | 3.59 / 5.68 / 3.80 |
| ttx | total | 1096.4644 ± 0.4021 | 1091.9988 ± 0.4214 | −4.4656 ± 0.0842 | 1088.580 ± 0.807 | 1088.2871 ± 0.5357 | 1089.2618 ± 0.6266 | +0.25 % | 3.76 / 5.45 / 3.62 |
| ll | `@0` | 665.2581 ± 0.2169 | 665.2662 ± 0.2169 | +0.0081 ± 0.0184 | 664.9237 ± 0.3572 | — | — | | 0.82 / — / — |
| ll | `@1` | 268.5922 ± 0.4514 | 268.5857 ± 0.4518 | −0.0065 ± 0.0225 | 267.9183 ± 0.3799 | — | — | | 1.13 / — / — |
| ll | `@2` | 131.2669 ± 0.6615 | 131.5916 ± 0.6662 | +0.3247 ± 0.3080 | 130.8444 ± 0.1946 | — | — | | 1.08 / — / — |
| ll | total | 1065.1172 ± 0.6847 | 1065.4435 ± 0.7407 | +0.3263 ± 0.2919 | 1063.686 ± 0.552 | — | — | | 1.90 / — / — |

The seed χ²/dof of the ttx fixed runs is @0 2.09, @1 1.26, total 1.76; the
pre-fix values were 1.98, 1.04, 1.57. MadEvent's 20 fresh seeds read @0 0.78,
@1 0.86, total 0.88. The equal-weight recombination was not run on ll, because
the bank's directories keep no `G*/results.dat`.

### Per group of ttx `@1`, and the localisation runs

The vibegraph column per card:

- `gg`, matched: V2's CM-toggle runs (5 seeds). `vg-fixed` reproduces V2's seed 20260928 exactly, 412.228241.
- `gq`, matched: V2's CM-toggle runs (5 seeds).
- `qq̄`, matched: V1's pre-fix runs, not re-run on the fix.
- unmatched rows: `vg-fixed`, 200 000 × 8, 5 seeds; the second gq row at 800 000 × 8, 3 seeds.

The MadEvent columns are fresh pinned directories. The unmatched cards are
V1's `ttx-dynnoxq` and `ttx-fixed` cards (`ickkw = 0`, `xqcut = 0`, `ptj = 30`).

| card | vibegraph | MadEvent reported | MadEvent equal-weight (paired shift) | MadEvent seeds, χ²/dof | rel vs eq | pull vs rep / eq |
|---|---|---|---|---|---|---|
| `gg > ttxg`, matched (row card) | 411.645 ± 0.25 | 409.2974 ± 0.4479 | 410.1739 ± 0.5900 (+0.876 ± 0.371) | 20, 0.63 | +0.36 % | 4.58 / 2.30 |
| `gq > ttxq`, matched | 148.414 ± 0.073 | 147.1379 ± 0.2819 | 147.1328 ± 0.2941 (−0.005 ± 0.232) | 20, 1.42 | +0.87 % | 4.38 / 4.23 |
| `qq̄ > ttxg`, matched | 18.909 ± 0.011 | 18.8550 ± 0.0447 | 18.8572 ± 0.0528 | 20, 0.51 | +0.28 % | 1.17 / — |
| `gg > ttxg`, unmatched, dynamical −1 | 297.0769 ± 0.1578 | 296.4231 ± 0.3403 | 296.8627 ± 0.3075 (+0.440 ± 0.250) | 10, 1.61 | +0.07 % | 1.74 / 0.62 |
| `p p > t t~ j` (j = quarks), unmatched, dynamical −1 | 118.4941 ± 0.0546 | 117.6831 ± 0.1872 | 117.7167 ± 0.1805 (+0.034 ± 0.051) | 10, **4.68** | +0.66 % | 4.16 / 4.12 |
| the same, MadEvent 100k events / vibegraph 800k × 8 | 118.6026 ± 0.0479 | 117.7985 ± 0.0843 | 117.9071 ± 0.0580 (+0.109 ± 0.038) | 5, **3.79** | +0.59 % | 8.29 / 9.25 |
| the same, unmatched, fixed μ = 173 | 137.9029 ± 0.0601 | 137.1245 ± 0.2112 | 137.1773 ± 0.1972 (+0.053 ± 0.050) | 10, **4.67** | +0.53 % | 3.54 / 3.52 |

## Reading

- **The fix is what V2 measured, on the full composite.** −4.465 pb on `@1`,
  against V2's per-group −4.347 (gg) and −0.126 (gq), which sum to −4.47. The
  qq̄ share is zero within this. `@0` is unaffected to 0.003 %. Its 2 → 2
  clustering has no initial-state emission to tie-break.
- **Iteration weighting.** On this row the bias sits in `P1_gg_ttxg` (+0.21 %)
  and on unmatched gg (+0.15 %). It is absent on gq and qq̄. That is the size
  `madevent-iteration-weighting-bias` measured on its 2 → 4 (0.1–0.2 %), the
  low end of the brief's 0.1–2 %. It is a MadEvent property, so nothing on this
  side changes. It does mean the gg residual (+0.36 %, 2.3σ) is now within the
  generic multi-leg offset already filed. The unmatched gg control agrees to
  +0.07 %.
- **gq is a separate, scale-independent discrepancy.** It is present without
  matching and at a fixed scale, at nearly the matched size (+0.53 to +0.66 %
  against +0.87 %; the 0.2 % difference is 0.8σ). So neither the CM fix nor
  anything in the clustering, `rewgt` or the dynamical scale carries it. Two
  features single gq out:
  - MadEvent's own seeds scatter well beyond their quoted errors on gq, χ²/dof
    4.7 at 10k events and 3.8 at 100k. Matched gq reads 1.4, and gg 0.6–1.6.
    This is the "confidently wrong" signature `AGENTS.md` warns about.
  - Neither side's budget closes it. MadEvent 10k → 100k moves +0.19 pb
    (equal-weight); vibegraph 200k → 800k moves +0.11 pb.

  V2 matched every factor per point on 3 600 of MadEvent's own gq points. The
  remaining candidates are therefore:
  - **(a)** a region MadEvent's gq channels under-sample, in a way more
    iterations do not repair;
  - **(b)** a weight or cut difference at points MadEvent rarely or never
    writes. V2 had no vibegraph-drawn gq points (Found 2 of V2), so nothing
    has tested this side's gq integrand on its own sample.
- **Falsifiers.** Any of these would contradict the reading:
  - a per-point comparison on vibegraph-drawn gq points, at fixed μ, that
    disagrees with MadEvent's `DSIG1` (that would make it (b));
  - a MadEvent run restricted to a region where vibegraph's gq σ density is
    high that recovers the gap (that would make it (a));
  - the gq gap moving between seeds as the budget grows (that would make it a
    sampling bug).

## Banking request

None. The comparisons above need no new banked reference data, and V1
already showed the bank agrees with fresh directories.

**Non-banking request.** The committed integrals cells for both rows are
`measured 2026-10-14` on the pre-fix tree. To refresh them on the fix, someone
must run `pixi run validate-mlm-sigma` (release-debug, the test harness). The
expected values are those of the `fixed` column above. The ttx cell stays
`info`: `@1` is 3.6σ from the bank. The ll cell stays `gate` / pass.

## Brief corrections

- **"`target/` (release-debug, built on the fix)".** There was no
  `release-debug` build. `target/` held only `debug` (dev profile, 14 GB), so
  `validate-mlm-sigma` would have needed a new release-debug build on a disk
  with 3–5 GB free. The measurement used dev-profile `vibegraph integrate`
  instead. `vg-prefix` reproduces the committed cell means to every printed
  digit, so the profile does not matter for these numbers. Deleting
  `target/debug/incremental` (3 GB, cache only) was needed to keep free disk
  above 3 GB. The manager later removed `target/debug` itself.
- **"V1's report and scratch hold the pre-fix per-group seeds".** They hold
  per-group CLI seeds and per-seed composite values for 20260928/29 only. The
  pre-fix composite was re-run on all 10 seeds, so the shifts above are
  paired on every seed.
- **"Scripts `iter_seed.py` and `combine.py` under `2to4-work/`".** They read
  that sprint's run layout (`runs/<label>/keep_*/G.tgz`) and sum over all P
  directories. `ttx3-iter.py` applies the same rule per P directory and per
  `@N`. It confirms that `results.dat` lists the last three iterations
  combined with weight mean²/σ² (`dsample.f:288-345`). The equal-weight mean
  is over those three.
- **The brief's residual figures (gg +0.57 %, gq +0.87 %)** are against
  MadEvent's reported σ. They reproduce exactly here: gg +0.574 %, gq +0.867 %.

## Gates

No source change, so no build or test gate applies. The working tree was
restored after the pre-fix build. `git diff --stat HEAD` was empty before the
report was added. `python3 scripts/kb.py index` and `python3 scripts/kb.py
lint` were run on the bundle before the commit.

## Found

1. **A gq-specific excess, independent of matching and scales.** vibegraph's
   `p p > t t~ j` (quark `j`) reads +0.53 % (fixed μ) to +0.66 % (dynamical)
   above pinned MadEvent without matching. That is 3.5–9σ, stable under 4–10×
   budget on both sides. MadEvent's gq seeds scatter with χ²/dof 3.8–4.7. It
   makes up about half of ttx `@1`'s post-fix residual. To close it:
   - compare per point, on vibegraph-drawn gq points at fixed μ = 173,
     against V2's `ptdriver_gq.f` (`ttx2-drv/`). This needs vibegraph to
     write gq events, see 3;
   - or compare σ in slices (x_max, the quark's p_T and η, m_tt̄) between
     vibegraph and dedicated MadEvent runs with a user cut, as V1 did for gg.
2. **The gg post-fix residual (+0.36 %, 2.3σ against equal-weight MadEvent)
   is within the generic multi-leg offset.** Unmatched gg agrees to +0.07 %
   after recombination. No action beyond the existing offset item, unless a
   20-seed ladder says otherwise.
3. **V2's Found 2 blocks item 1.** `vibegraph generate` refuses
   `p p > t t~ j` with quark-only `j` (the mixed-flavour group's resonance
   lookup). It is still open.
4. **The ttx and ll integrals cells are stale.** They were measured pre-fix.
   The ttx `@1` cell reads 583.24 against 578.78 on the fix. Refresh with
   `pixi run validate-mlm-sigma` on the fix (non-banking).
