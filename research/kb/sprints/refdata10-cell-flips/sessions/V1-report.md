---
type: Session Report
title: "V1 report: both refdata-10 cells agree and gate"
description: "pp_to_ttx_0j1j_mlm reads @0/@1/total pulls +0.40/+0.38/+0.46 over ten seeds against its refdata-10 seed record; gg_to_gg_cg reads +0.89 over five seeds (+0.15 on the gate seed). Both cells flipped to gate, notes rewritten."
status: draft
tags: [sigma, mlm, ttx, smeftsim, gate-promotion, refdata, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: flip, resource: "https://github.com/nsmith-/vibegraph/commit/1158cfc", title: "1158cfc: gate the t t~ MLM and gg_to_gg_cg integrals cells on refdata-10"}
---
Both cells agree with their refdata-10 records and now gate. That is in
commit `1158cfc` on `refdata10-cell-flips`. No tolerance, floor or limit
moved. PR #42 was still open at edit time:
`gh api repos/nsmith-/vibegraph/pulls/42 --jq '.state, .merged'` printed
`open` / `false`. So the `validate_sigma.rs` edit sits on this branch's base,
where the fixed-beam gate spends one seed, `SEED`. See "Interaction with #42"
below.

Host: 4-core Xeon @ 2.80 GHz, shared with one sibling MC job (load average
5–8 throughout). Logs are under
`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/`
with the prefix `cellflips-V1-`.

## The references, recomputed by hand

I recomputed each record's reported values at the budget (`nevents` 10000,
seeds 20261101–05) outside the test code, then compared them with what the
gates print. Command: `python3` over
`validation/madgraph/output/seeded_sigma/<row>/seed_record.json`. The rule
is the unweighted mean ± max(quoted, spread/√n).

| row / part | n | mean (pb) | quoted | spread/√n | err | χ²/dof |
|---|--:|--:|--:|--:|--:|--:|
| ttx `@0` | 5 | 513.1058 | 0.2216 | 0.1214 | 0.2216 | 0.33 |
| ttx `@1` | 5 | 578.3374 | 1.1275 | 0.9433 | 1.1275 | 0.89 |
| ttx total | 5 | 1091.44 | 1.1491 | 0.9953 | 1.1491 | 0.92 |
| `gg_to_gg_cg` | 5 | 270658.0 | 80.17 | 88.06 | 88.06 | 1.15 |

The gates print the same numbers (`seed_record::describe` lines in the logs
below). Both gates already read the record under the seed policy. For ttx
the path is `seeded_reference` → `seed_record::part_reference`, and for
`gg_to_gg_cg` it is `BankedSigma::madevent_reference`. Neither still reads
the old single run, so the reader needed no fix. Both records set
`hard_survey = 2` with `as_set: true` and `validated_ew: false`, so they read
the reported estimator.

## `pp_to_ttx_0j1j_mlm`

Before launching I estimated the cost. The `pp_to_ll_0j2j_mlm` figure was
12 min a seed. A ttx seed measured 108–114 s here, so the gate's full ten
seeds fit easily.

Measurement, still in `info` mode, detached:
`cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_hadronic -- --ignored --nocapture --exact sigma_ttx_0j1j_mlm_vs_madevent`
(log `cellflips-V1-ttx.log`, 1104.93 s, `test result: ok`).

Per seed, in pb:

| seed | `@0` | `@1` | total | wall |
|---|--:|--:|--:|--:|
| 20260928 | 512.9270 ± 0.3890 | 578.9974 ± 0.9293 | 1091.9244 ± 1.0075 | 112 s |
| 20260929 | 513.6326 ± 0.3980 | 579.1308 ± 0.9783 | 1092.7634 ± 1.0562 | 114 s |
| 20260930 | 512.4278 ± 0.3924 | 579.5333 ± 0.9334 | 1091.9611 ± 1.0125 | 111 s |
| 20260931 | 514.1821 ± 0.3926 | 579.3940 ± 0.9453 | 1093.5761 ± 1.0236 | 108 s |
| 20260932 | 512.6557 ± 0.3901 | 577.9332 ± 0.9226 | 1090.5889 ± 1.0017 | 108 s |
| 20260933 | 513.0279 ± 0.3897 | 577.0816 ± 0.9123 | 1090.1095 ± 0.9921 | 112 s |
| 20260934 | 512.9004 ± 0.3892 | 578.0930 ± 0.9230 | 1090.9934 ± 1.0017 | 110 s |
| 20260935 | 513.0604 ± 0.3941 | 579.8981 ± 0.9382 | 1092.9586 ± 1.0176 | 110 s |
| 20260936 | 513.9751 ± 0.3936 | 580.1293 ± 0.9157 | 1094.1044 ± 0.9967 | 110 s |
| 20260937 | 513.4030 ± 0.3925 | 577.6052 ± 0.9316 | 1091.0083 ± 1.0109 | 112 s |

The gate's cells, as printed by the gate-mode run after the flip (same
command, `cellflips-V1-validate.log` lines 11203–11209, `TTX_EXIT 0`):

| part | vibegraph (10 seeds) | χ²/dof (band 0.128–3.097) | MadEvent (5 seeds) | pull | rel | status |
|---|--:|--:|--:|--:|--:|---|
| `@0` | 513.2192 ± 0.1794 | 2.09 | 513.1058 ± 0.2216 | +0.40 | +0.02% | gate: pass |
| `@1` | 578.7796 ± 0.3273 | 1.26 | 578.3374 ± 1.1275 | +0.38 | +0.08% | gate: pass |
| total | 1091.999 ± 0.421 | 1.76 | 1091.440 ± 1.149 | +0.46 | +0.05% | gate: pass |

I also read the first five seeds alone, recomputed from the per-seed lines
with the same rule. `@0` reads 513.1650 ± 0.3250 (χ²/dof 3.43, pull +0.15).
`@1` reads 578.9977 ± 0.4213 (0.46, +0.55). The total reads
1092.1628 ± 0.4966 (1.20, +0.58). So agreement does not depend on reading
ten seeds.

`@1` on this side, 578.7796 ± 0.3273, matches the 578.780 ± 0.327 that
`sigma-ttx-mlm-at1` V4 recorded on the same seeds after the CM-frame
clustering fix, to every digit that record printed. The +8.9σ excess at
refdata-9 is gone against the `hard_survey = 2` reference, as that sprint
predicted.

The flip changes `"info"` to `"gate"` in `sigma_ttx_0j1j_mlm_vs_madevent` and
rewrites its doc comment. The gate's assertion already existed and was used
unchanged: |pull| < 3 and χ²/dof inside the band, per `@N` and on the total.

## `gg_to_gg_cg`

The measurement used an uncommitted throwaway probe, reverted before the
edit. It called `integrate(dir, process, 40_000, 6, seed)` (the plan budget)
on `HEADROOM_SEEDS` and then seeds 55–99, compared against
`madevent_reference`, and printed `combine` after each seed.
`cargo test ... --test validate_sigma -- --ignored --nocapture --exact probe_tmp_cellflips_gg_cg`
with `VG_TMP_SEEDS=10` (log `cellflips-V1-ggcg.log`, 719.98 s).

| seed | σ (pb) | pull | rel | chi2/dof (iterations) |
|---|--:|--:|--:|--:|
| 20260719 | 2.707089e5 ± 330.6 | +0.15 | +1.88e-4 | 0.82 |
| 11 | 2.704886e5 ± 295.7 | −0.55 | −6.26e-4 | 1.10 |
| 22 | 2.706761e5 ± 308.8 | +0.06 | +6.71e-5 | 1.09 |
| 33 | 2.712777e5 ± 331.6 | +1.81 | +2.29e-3 | 4.13 |
| 44 | 2.709013e5 ± 370.2 | +0.64 | +8.99e-4 | 1.74 |
| 55 | 2.707542e5 ± 337.2 | +0.28 | +3.55e-4 | 0.81 |
| 66 | 2.710335e5 ± 325.8 | +1.11 | +1.39e-3 | 0.80 |
| 77 | 2.707919e5 ± 277.7 | +0.46 | +4.95e-4 | 1.82 |
| 88 | 2.706213e5 ± 287.7 | −0.12 | −1.36e-4 | 1.11 |
| 99 | 2.706962e5 ± 324.1 | +0.11 | +1.41e-4 | 2.29 |

| read | mean (pb) | err (quoted / spread) | pull | rel | seed χ²/dof |
|---|--:|--:|--:|--:|--:|
| 5 seeds (`HEADROOM_SEEDS`) | 2.708105e5 | 146.8 (146.8 / 133.9) | +0.89 | +5.6e-4 | 0.88 |
| 10 seeds | 2.707950e5 | 101.2 (101.2 / 71.4) | +1.02 | +5.1e-4 | 0.50 |

Over ten seeds the worst |rel| is 2.29e-3 (seed 33) and the worst |pull| is
1.81. The flip makes `Plan::Gate { 40_000, 6, rel_tol: 0.005 }`, the
SMEFTsim rows' convention (`ee_to_ttx_dipole`, `ee_to_zh_smeft` and others),
which sits 2.2× outside the worst seed. `PULL_LIMIT` stays 3.5.

The gate as it ran in `pixi run --skip-deps validate` (log line 10198):
`[gg_to_gg_cg] GATE vg = 2.707089e5 +- 3.306e2 pb | MG = 2.706580e5 +- 8.806e1 pb | pull = +0.15 | rel = +1.88e-4 | chi2/dof = 0.82 (40000x6)`.
The collated report cell reads `✅ pull +0.15, chi2/dof 0.82`.

The earlier −0.16% to −0.22% "converged offset" was read against the single
banked run, iseed 21 (271240 ± 230). Against the seed record it is gone:
+5.6e-4 at 1.7σ of the five-seed read. This is consistent with
`sigma-gg-cg`'s attribution (quadrature 270650.9, ten-times-budget MadEvent
270652.5 ± 23).

## Interaction with #42

On this base the fixed-beam gate integrates `SEED` alone, so the
`gg_to_gg_cg` gate asserts pull +0.15 and rel +1.9e-4. Once #42 merges, the
gate reads the five `HEADROOM_SEEDS` under the seed policy: pull +0.89, rel
+5.6e-4 in the probe above, which also passes. The manifest note records
both readings. The arm I edited is textually separate from #42's hunks in
that file. The nearest of those hunks is the `SEED`/`HEADROOM_SEEDS` block
at about line 215, while my `#[allow(dead_code)]` sits on `Plan::Info` at
about line 237. Git may still need a hand-merge there.

## The one change beyond the arm

`gg_to_gg_cg` was the last row on `Plan::Info`. With the flip,
`cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings`
failed with `error: variant 'Info' is never constructed`
(`cellflips-V1-validate.log` lines 323–335). I kept the variant, because
the enforcement ladder's demotion rung is `Info`, and gave it a per-site
`#[allow(dead_code)]` with one doc line saying why. This is the only edit in
`validate_sigma.rs` outside the arm. The manager's "touch nothing else"
instruction did not anticipate it, so I am flagging it here.

## Manifest

- `pp_to_ttx_0j1j_mlm.integrals`: `mode = "gate"`. The note is rewritten on
  refdata-10. It covers what is compared, the three readings, the gate rule,
  the attributed history in one sentence, and what the cell cannot see. The
  blind spots are distributions, and on `@1` an offset under about 3.5 pb
  (0.6%), which is 3σ at the reference's 1.13 pb error, the dominant term.
  The note also keeps the permuted-P1 deviation.
- `gg_to_gg_cg.integrals`: `mode = "gate"`. The note is rewritten on the
  seed record. It gives both readings (gate seed, five-seed and ten-seed),
  the tolerance and its headroom, and the history. The blind spot is anything
  σ integrates over, plus an offset under about 0.44%: on the one gate seed,
  3.5 × √(331² + 88²) pb, which binds before `rel_tol`.
- `gg_to_gg_cg.samples`: one stale clause, "the −0.22% its integrals cell
  records", now names the normalisation generically.
- The MLM section's header comment now says `validate-mlm-sigma` gates
  `pp_to_ttx_0j1j_mlm` too.

## Gates

| command | result |
|---|---|
| `pixi run --skip-deps validate` (`cellflips-V1-validate.log`, 10:58–13:10 UTC) | `cargo test --workspace --profile release-debug` with extended-validation: 60 `test result: ok`, 0 `FAILED`. Collator: "every gate cell passed, and every measurement agrees with the manifest about what it is". Exit 101 came only from the clippy step (`!!! clippy failed`), the dead-code error above, fixed before commit |
| `cargo fmt --all --check` | `FMT_OK` |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings` (after the allow) | `CLIPPY_EXT_EXIT 0` |
| `cargo clippy --workspace --all-targets -- -D warnings` | `CLIPPY_EXIT 0` |
| `sigma_ttx_0j1j_mlm_vs_madevent` in gate mode | `TTX_EXIT 0`, three `gate: pass` |
| `cargo run --profile release-debug -p vibegraph-validation-report` after the final note edit | "64 rows x 4 categories: the measured cells are the declared cells". 256 cells: 204 measured (199 ✅, 5 ⚠️, 12 ⏳, 40 uncovered). The ttx row reads `✅ pull +0.46, chi2/dof 1.76 (worst of 3 ...)` |

Enforced cells are unmoved: the collator found every gate cell passing. The
full banked layer ran on this host. I did not run a separate unoptimised
`cargo build` / `cargo test`, because the release-debug workspace test with
extended validation is a superset of it and disk was at 9.9 GB free.

## Brief corrections

- Scope 1's "if it still reads the old single run, that is the fix" did not
  apply. Both gates already read the seed records.
- Scope 2 locates the cell at "`Plan::Info` ~:665". It was at :665, and it
  was the only remaining `Info` row, which the brief did not anticipate. That
  is what produced the dead-code fallout.
- The backlog item for `gg_to_gg_cg` quotes a "pull +0.15" reading. That is
  exactly the single-`SEED` reading. The five-seed reading is +0.89.

## Found

1. **`validate_sigma.rs` module doc is stale on `gg_to_gg_cg`.** Lines 72–76
   say two rows "are not integrated at all", naming `gg_to_gg_cg`'s
   `dynamical_scale_choice = 3`. The row has been integrated, and is now
   gated. The paragraph predates this session. I left it alone under the
   "touch nothing else in that file" instruction. Closing it means rewriting
   the paragraph to name only `wpwm_to_wpwmz_cw`.
2. **The `mlm-sigma-gate` concept still describes `pp_to_ttx_0j1j_mlm` as
   `info`.** The places are the gate list ("`sigma_ttx_0j1j_mlm_vs_madevent`
   (info)"), the refdata-9 table rows marked "(info)", and the "Standing
   residuals" bullet "t t̄ @1 is +1.28%, undiagnosed". The references
   paragraph also says the other rows' references are shared-directory
   seeds. Closing it means updating the concept with this report's table.
   That is close-out work, since I may not edit `research/kb/` outside this
   report. `sigma-gate.md`'s `Plan` table is still accurate, but no row sits
   on `Info` now.
3. **ttx `@0`'s per-seed quoted error understates its seed scatter by about
   1.45×, persistently.** χ²/dof is 2.09 over ten seeds and 3.43 over the
   first five here, and was 1.98 and 2.01 at refdata-9. The gate passes
   because err = max(quoted, spread/√n) takes the spread, and 2.09 sits
   inside the 3.097 band upper edge with only 1.5× headroom. A sampler
   change that widens `@0`'s scatter a little would fail the band, not the
   pull. Evidence is the per-seed table above. Closing it means a ≥20-seed
   read of `@0` at this budget to measure the true spread, and looking at
   whether `@0`'s single channel's iteration errors are underestimated, for
   example by the Neyman split starving it in some seeds.
4. **The ttx reference error dominates the `@1` cell.** It is 1.13 pb from
   five seeds, against this side's 0.33. At 3σ the cell cannot see an `@1`
   offset under about 3.5 pb (0.6%). The record has two 4×-budget seeds
   (579.30 ± 1.30 and 580.46 ± 1.18) that are not read. Closing it needs
   more fresh-directory MadEvent seeds at the budget, a re-bank on the M3
   host. If wanted, the request to post is
   `validation/madgraph/bank_seeded_sigma.sh` for `pp_to_ttx_0j1j_mlm`, 15
   more fresh-directory seeds at `nevents = 10000`, `hard_survey = 2`, with
   expected output `seeded_sigma/pp_to_ttx_0j1j_mlm/s<iseed>/` and an updated
   `seed_record.json`. It is not needed for this cell to gate.
5. **The ttx manifest note's per-event part rests on refdata-9 dumps taken
   before the CM-frame clustering fix (`6740954`).** The note now points to
   the samples cell for those numbers rather than restating them. Whether
   `validate_mlm_dumps` was re-run on this row after the fix is not
   something this session could check: the oracle layer and the dumps are
   not in the bundle. The samples cell's note still carries the 2026-10-03
   measurement. Closing it means re-running `pixi run validate-mlm-dumps` on
   the bank host and re-dating that note.
