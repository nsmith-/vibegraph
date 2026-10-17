---
type: Session Report
title: "V1 report: today's numbers pass the amended rule; the comparison is gated"
description: "Both sides through Pythia on this host: B-vs-A and MadEvent's null pass both jet-rate legs and all five pulls; mlm_match.py implements D1 as amended, writes a row the collator renders, and the mlm-pythia standalone entry is flipped to gate."
status: draft
tags: [mlm, pythia, matching, gate-promotion, look-elsewhere, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: df3bbf9, resource: "https://github.com/nsmith-/vibegraph/commit/df3bbf9", title: "validation: gate the matched MLM Pythia comparison"}
---
Worktree `/home/user/wt/mlm-pythia-tolerance`, branch `mlm-pythia-tolerance`,
base `64c1496`. Host: 4-core linux-64 container shared with sibling σ gates;
Pythia 8.312 from conda-forge `linux-64/pythia8-8.312-py311h5756c45_105`.
Logs and scratch scripts are in the session scratchpad, all prefixed `mlmpy-V1-`.

## 1. Inputs and the run on the base commit

- **vibegraph samples.** `SEEDS=… pixi run -e pythia --skip-deps
  generate-mlm-pythia-samples` in two backgrounded halves (seeds 20260928–37,
  then 38–47, `RAYON_NUM_THREADS=2`). Wall time 21:59:51 → 23:39:40, including
  a 6 min release-debug build, then 23:39:51 → 01:26:46. That is about 9.5 to
  10.5 min a seed on this loaded host, against the brief's 7 min. Binary sha256
  `2aab2d3c…57b0`.
- **Pythia time.** I timed one run per side first: 106 s wall for two runs in
  parallel, driver build included. 410 runs at `--jobs 2` came to about 100 s a
  run, so **≈ 5.6 h in total, above the ~5 h the brief asked me to flag**. The
  runs went in four backgrounded seed chunks that fill the driver's per-run
  cache: seeds 01–03 in 102 min, 04–06 in 98 min, 07–08 in 61 min, 09–10 in
  60 min.
- **The recorded run on `64c1496`**, `pixi run -e pythia --skip-deps
  validate-mlm-pythia`, read all 410 runs back from the cache (exit 0, 60 s):

| | MadEvent | vibegraph | pull |
|---|---|---|---|
| acceptance `@0` | 0.8185 ± 0.0003 | 0.8181 ± 0.0004 | −0.76 |
| acceptance `@1` | 0.3607 ± 0.0011 | 0.3578 ± 0.0012 | −1.79 |
| acceptance `@2` | 0.3477 ± 0.0018 | 0.3423 ± 0.0021 | −1.99 |
| acceptance, all | 0.6446 ± 0.0006 | 0.6433 ± 0.0008 | −1.38 |
| merged σ (pb) | 685.70 ± 0.68 | 685.29 ± 0.80 | −0.39 |
| jet-rate χ² d01 / d12 / d23 | | | 29.9/26 (p 0.274), 31.5/23 (0.111), 29.0/20 (0.087); bootstrap 30.7, 30.9, 28.0 |

- **Reproduction against the concept's table.** It agrees within statistics,
  not bit for bit. There are two separate causes:
  - *Side B is a different sample set.* #23 (`9e141de`, MLM clustering in the
    partonic CM) changed what vibegraph writes in `<scales>` after the
    2026-10-01 record, so today's twenty files are not the files that record
    measured.
  - *Side A has the same files and the same Pythia seeds but a different
    host.* MadEvent's column moved from 0.8191 / 0.3603 / 0.3459 to
    0.8185 / 0.3607 / 0.3477. The scratch script `mlmpy_V1_shower_noise.py`
    computes the shower-only spread of the difference between two independent
    10-seed shower sets of the same events, √(2 Σ wᵢ² sᵢ² / 10) / Σ wᵢ. In
    those units the shifts are −1.29, +0.52 and +1.66 (overall −0.14). That is
    shower statistics.
  - *The runs are deterministic on one host.* I re-ran `run_01` and
    `vg-20260928` at seed 20261205 into a fresh `--work`, and both `.tsv` files
    are byte-identical to the cache (`cmp`). So the cache is sound, and the
    difference is between Pythia builds: the lock carries different osx-arm64
    and linux-64 builds. My hypothesis, not checked, is that the 2026-10-01
    record ran on the osx-arm64 bank host.

## 2. Today's numbers under the amended rule

These come from `pixi run -e pythia python mlmpy_V1_measure.py` (scratch). It
reads the same cache through the base commit's own functions, with α = 0.5 %
per leg and splits A = files 1–11 against 12–21, B = seeds 28–37 against 38–47.
The implemented driver later printed the same values to every digit.

| | B vs A | MadEvent null (A vs A) | vibegraph null (info) |
|---|---|---|---|
| pulls `@0` / `@1` / `@2` / all / σ | −0.76 / −1.79 / −1.99 / −1.38 / −0.39 | −1.33 / +0.45 / +0.55 / −1.19 / −1.48 | −0.79 / **−2.96** / +0.25 / −1.39 / −1.51 |
| χ² d01 / d12 / d23 | 29.9/26, 31.5/23, 29.0/20 | 36.1/25, 32.0/22, 29.1/19 | 11.1/25, 15.8/22, 20.7/19 |
| p | 0.274, 0.111, 0.087 | 0.069, 0.077, 0.064 | 0.993, 0.824, 0.356 |
| χ² leg: min p × 3 | 0.262, pass | 0.193, pass | 1.07, pass |
| N_bins | 72 (27 + 24 + 21) | 69 | 69 |
| z* (2 Φ̄(z*) = 0.005 / N_bins) | 3.978 | 3.968 | 3.968 |
| max \|pull\| (histogram, bin, log10 range) | −2.91 (d12, bin 1, [0.0, 0.1]) | −2.93 (d01, bin 11, [1.0, 1.1]) | −2.59 (d23, bin 2, [0.1, 0.2]) |
| per-bin leg: p_min × N_bins | 0.258, pass | 0.234, pass | 0.656, pass |

Neither B-vs-A nor the null fails, so per step 2 I went on to implement. The
MadEvent null's d12 no longer reads p 0.034 on this host (it reads 0.077, χ²
32.0/22). The null's worst χ² is now d23 at p 0.064, still × 3 = 0.19.

**Proposed edit to D1's "Today's numbers" paragraph** (the manager writes D1):

> **Today's numbers** (V1, 2026-10-17, linux-64 Pythia 8.312, today's twenty
> vibegraph samples): N_bins = 72 (27 + 24 + 21), z* = 3.98. B vs A: max
> |pull| 2.91 in d12, bin [0.0, 0.1] (p_min × 72 = 0.26), χ² leg min p × 3 =
> 0.26 (d23 p 0.087); pulls −0.76 / −1.79 / −1.99 / −1.38 / −0.39. MadEvent
> null: 69 bins, z* 3.97, max |pull| 2.93 in d01 [1.0, 1.1] (p_min × 69 =
> 0.23), χ² min p × 3 = 0.19 (d23 p 0.064; d12 reads 0.077 on this host, not
> 0.034). vibegraph null (information): max |pull| 2.59, χ² min p × 3 = 1.07,
> `@1` pull −2.96. Both legs pass on B vs A and on the null.

## 3. Implementation (`df3bbf9`)

- **`validation/pythia/mlm_match.py`**
  - `PULL_LIMIT = 3`, `JET_RATE_ALPHA = 0.005`. `gated_pulls` gives the five
    pulls; `verdict` fails a pull that is missing or has |pull| ≥ 3.
  - `bin_pulls` gives B − A per normalised bin over the χ²'s delta-method
    errors. `z_star` uses `statistics.NormalDist`.
  - `jet_rate_legs` runs both legs. The χ² leg fails when min p × 3 < α; the
    per-bin leg fails when erfc(|z|max/√2) × N_bins < α.
  - `null_split` puts the first ⌈n/2⌉ files against the rest, under the
    comparison's own statistics, without bootstrap.
  - `verdict` covers the null: a failing null makes the jet-rate verdict
    `info` / "null failed", and a missing null (fewer than two A files) makes
    it `info` / "no null". Nothing is rescaled either way, and a failing null
    never excuses a pull.
  - `compare_sides` factors out the comparison so the nulls reuse it.
  - The row gains `mode: "gate"`, `status`, `nulls` (full) and `gate`, the
    compact record the collator reads. The process exits 1 on fail.
  - The localisation variables are untouched and not in the verdict.
- **`validation/pythia/test_mlm_match.py`** holds 16 stdlib-`unittest` tests on
  synthetic histograms (three of 24 bins, so 72 bins in all), run by the new
  pixi task `test-mlm-pythia`:
  - z* at 72 bins is 3.96–3.99, and erfc(z*/√2) × 72 = α holds to 1e-12.
  - Each pull fails alone at ±3.0 and 3.4, and passes at 2.99. A missing pull
    fails.
  - **A two-bin 3σ excess passes both legs.** This pins the blind spot.
  - **A two-bin 4.1σ excess (beyond z*) passes the χ² leg (min p × 3 ≈ 0.06)
    and the per-bin leg catches it.** The test checks the histogram and bin it
    reports.
  - A deficit counts like an excess.
  - A spread 1.6σ distortion is caught by the χ² leg only.
  - **A failing null** makes the jet rates `info`, "null failed", and the
    comparison passes on the pulls. A null failing on the χ² leg alone still
    counts as failed, a failing null does not excuse a pull, and with no null
    the jet rates are `info`.
  - The row record carries every D1 field.
- **The collator** (`validation-report/src/render.rs`, `main.rs`, `manifest.rs`):
  - `standalone_verdict` sends a row with `"row": "mlm-pythia"` to
    `mlm_pythia_verdict`. That renders the mark (✅ pass with the jet rates
    gating, ⚠️ pass with the jet rates `info`, ❌ fail), the five pulls, the
    three B-vs-A p-values with min × 3, the max |pull| with its histogram and
    range, z* and N_bins, the same for the MadEvent null, and the vibegraph
    null as information.
  - `[[standalone]]` gains an optional `mode`. `check_standalone_modes` reports
    a problem when the row file present records a different mode. Negative
    control: with the row's `mode` edited to `info`, the collator reports
    `standalone 'mlm-pythia' is declared mode 'gate' but …/mlm_pythia.json
    records mode 'info'`.
  - There are two renderer unit tests.
- **`validation/manifest.toml`:** the `mlm-pythia` standalone entry gets
  `row = "mlm_pythia.json"`, `mode = "gate"` and the test file in `targets`.
  Its rationale now states the rule and its note the recorded run (section 4).
- **`pixi.toml`:** the `validate-mlm-pythia` comment now describes the gate,
  and the `test-mlm-pythia` task is new.

## 4. The recorded run under the rule

On `df3bbf9`'s tree before the commit, `pixi run -e pythia --skip-deps
validate-mlm-pythia` printed:

```
  B vs A: chi2 leg min p x 3 = 0.262 [pass]; per-bin leg max |pull| 2.91 (log10_d12 bin 1) against z* 3.98 at 72 bins [pass]
  null madevent vs madevent: chi2 leg min p x 3 = 0.193 [pass]; per-bin leg max |pull| 2.93 (log10_d01 bin 11) against z* 3.97 at 69 bins [pass]
  null vibegraph vs vibegraph (information): chi2 leg min p x 3 = 1.07 [pass]; per-bin leg max |pull| 2.59 (log10_d23 bin 2) against z* 3.97 at 69 bins [pass]
  null madevent vs madevent pulls: 0 -1.33, 1 +0.45, 2 +0.55, all -1.19, merged_sigma -1.48
  null vibegraph vs vibegraph pulls: 0 -0.79, 1 -2.96, 2 +0.25, all -1.39, merged_sigma -1.51
verdict: PASS (pulls |pull| < 3: all pass; jet rates gate: pass)
```

It exited 0 in 71 s from the cache, and the acceptance and χ² lines are
identical to section 1. Then `cargo run -q -p vibegraph-validation-report`
rendered the row:

```
| `mlm-pythia` | oracle | ✅ pass — pulls (|pull| < 3): @0 -0.76, @1 -1.79, @2 -1.99, all -1.38, merged σ -0.39; jet rates gate (pass), at α = 0.005 per leg: χ² p d01 / d12 / d23 0.274 / 0.111 / 0.087 (min × 3 = 0.262, pass); max |pull| 2.91 in log10_d12 [0.0, 0.1] against z* 3.98 at 72 bins (pass); MadEvent null pass: χ² p d01 / d12 / d23 0.069 / 0.077 / 0.064 (min × 3 = 0.193, pass); max |pull| 2.93 in log10_d01 [1.0, 1.1] against z* 3.97 at 69 bins (pass); vibegraph null (information) pass: χ² p d01 / d12 / d23 0.993 / 0.824 / 0.356 (min × 3 = 1.067, pass); max |pull| 2.59 in log10_d23 [0.1, 0.2] against z* 3.97 at 69 bins (pass); Pythia 8.3120e+00 |
```

The collator exited 1 only because no other gate ran in this worktree: all 406
of its problems are "declared … and no gate wrote it", and none concerns a
standalone gate.

## Gates

| command | result |
|---|---|
| `pixi run -e pythia test-mlm-pythia` | `Ran 16 tests … OK` |
| `cargo test -p vibegraph-validation-report` | 38 + 6 passed, 0 failed |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy -p vibegraph-validation-report --all-targets -- -D warnings` | exit 0 (the only crate whose Rust changed) |
| `cargo build --workspace && cargo test --workspace` (lean env vars, 07:02 → 07:25) | exit 0; 39 result lines, 1480 passed, 0 failed, 16 ignored |
| `pixi run -e pythia --skip-deps validate-mlm-pythia` | PASS (above) |

I did not run `pixi run --skip-deps validate`. No amplitude, eval or coverage
list was touched, and the one manifest change is a standalone entry, which
`the_committed_manifest_parses` and the collator render cover.

## Brief corrections

- "Flip the manifest cell to `gate` (long tier)": the comparison is a
  `[[standalone]]` entry, and the schema had no tier or mode for one. Its
  layer, `oracle`, is the standalone equivalent of the long tier. I added an
  optional `mode` with a collator check rather than invent a tier.
- "About 7 min a seed" was 9.5 to 10.5 min here, and the Pythia stage is about
  5.6 h at two jobs on this host, not under 5 h.
- D1's "today the null's worst is d12 p 0.034" is a different host's shower
  realisation. It reads p 0.077 here, which matters for how D1 quotes it.

## Found

1. **vibegraph's `@1` files scatter beyond their errors on today's samples.**
   - *Evidence:* the per-file χ²/dof about the side mean, from the recorded
     row's `sides.b.acceptance`, is `@0` 1.54, `@1` 1.87 and `@2` 0.59 (19 dof;
     `@1` p ≈ 0.01), with merged σ at 0.67. The 2026-10-01 samples read 0.75 /
     0.88 / 0.92, and MadEvent reads 0.65 / 0.96 / 0.66 today. The vibegraph
     null's `@1` pull is −2.96, against +0.23 on the old samples.
   - *Effect on the gate:* the rule takes B's event-spread error as honest. If
     the excess is real, B's `@1` error is low by about √1.87, and the B-vs-A
     `@1` pull would read about −1.5 instead of −1.79. The verdict holds, but
     the rule cannot see this excess, and no per-file scatter check is in D1.
   - *To close:* attribute it, either to #23's `<scales>` change or to
     overweight files (two files carry `@1` errors of 0.0079 and 0.0069 against
     0.0050 typical), and decide whether a per-file χ²/dof band joins the rule.
     A 20-seed re-check on the vibegraph side needs no banking.
2. **The comparison is not reproducible across Pythia builds, and the record
   does not say which build it ran on.**
   - *Evidence:* the same MadEvent files and seeds moved by −1.3, +0.5 and
     +1.7 shower-only σ between the 2026-10-01 record and this host, while runs
     are byte-identical on one host.
   - *To close:* record Pythia's build (`pythia8-config`, the platform) in the
     row, and have the gate concept quote numbers with their host.
3. **The KB pages still describe the comparison as `info`.** The gate concept
   (`validation/mlm-pythia-matched-comparison.md`, "It is `info`", "The row is
   `info`, with no collator row", the results and nulls tables),
   `validation-report.md`'s standalone section, and the
   `mlm-pythia-comparison-no-tolerance` item all say so. The close-out should
   update them to today's record and close the item.
4. **The row's Pythia version reads `8.3120e+00`.** `mlm_match.cc` prints
   Pythia's float version. The collator shows it as written; formatting it
   as `8.312` is cosmetic.
5. **The pull rule and the per-bin pulls are correlated, and the per-bin
   correlation is unquantified.** The five pulls are not independent: `all` is
   a weighted mix of the three `@N`, and the merged σ tracks `all`. The ~1.3 %
   false-fail figure in D1 is an upper bound. The per-bin pulls share each
   histogram's normalisation, so the leg is conservative as D1 says, but by an
   unmeasured amount. A toy study of the null split's per-bin |pull|max
   distribution would calibrate it.
