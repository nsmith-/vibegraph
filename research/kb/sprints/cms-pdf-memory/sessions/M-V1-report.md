---
type: Session Report
title: "M-V1 report: the table oracle in the tree, the 880 members fetched, the item's clauses met"
description: "An ignored lib test carries the precomputed-table interpolator and finds 0 of 414M readings different over 984 members, catches a one-ulp mutation (1.68M) and a wrong-band seam read (1.88M), and fails on an empty tree; a fetch script rebuilds the 880-member tree byte-identical to the reference copy; peak RSS 0.685 GiB at -j 1 and 0.688–0.701 GiB at -j 4 by wait4 and VmHWM; the CMS-arguments gate exact on 4 × 9.26M weights."
status: draft
tags: [pdf, memory, cms, validation, report, sprint-f]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/M`, branch `sprintf-pdf-memory`, from `82ba52d`.
Implements the [M-V1 brief](M-V1.md). Scratch files are under
`/tmp/claude-0/-home-user-vibegraph/7497b6d2-0784-5f2c-a2c9-2ed0beaa757e/scratchpad/M-V1/`
(`$S` below): every log quoted here, the fetched tree (`$S/fetch-test/tree`),
the RSS wrapper and the mutant patches.

**Host.** Linux container, 4 cores. The sibling session in `/home/user/wt/G`
was active throughout; 1-minute load was 3.6–10.3 during the runs below, and
each timing line carries it.

## 1. The bit-level oracle (`pdf::table_oracle`)

`vibegraph-lib/src/pdf/table_oracle.rs`, compiled only under
`#[cfg(all(test, feature = "extended-validation"))]`, the skill's convention for
a library unit test that needs inputs a bare clone lacks. It is a lib test, not
an integration test, because the comparison runs both interpolators through the
production `Continuation` and positivity clamp, which are crate-private.

- **The oracle** is the table interpolator of `86b8e5c` (identical to `cb1498a`
  in `src/pdf/`): `[a, b, c, d]` per `(x interval, Q² knot, flavor)` from
  `ddx_logx`, the same `QCell` stencil, Horner chain, Hermite and bilinear
  fallback, behind `Bicubic2D`.
- **Readings compared**, `to_bits`, no tolerance: per point the all-flavor row
  (`try_xfx_all`, 16 slots) and `try_xfx_q2` for 16 PDG codes (0, 22 and the
  absent 7 included), against the same composition (guards, interpolation or
  continuation, clamp) over the table interpolator. A `Result` that differs
  (one side refuses) counts as a difference.
- **Points per member**: every knot of every band; 40 x values at every band's
  lower and upper Q² edge (every seam) and one ulp either side; every x knot and
  one ulp either side; 3000 log-uniform points; past the grid (below `x_min`,
  below `Q²_min`, above `Q²_max`, the corners, one ulp past each edge).
- **Members**: every `<set>_NNNN.dat` under `$SYSTEMATICS_CMS_PDFS` (the 880),
  all 101 of NNPDF23 when `validation/pdf` holds it, and three in-memory members
  with two-knot (bilinear) bands, which no CMS set has.
- **Non-vacuity**: it asserts at least one member under the tree, every member
  compared, every point category non-empty, more than half the readings nonzero,
  and prints the counts.

### It passes on the 880 members

```
$ SYSTEMATICS_CMS_PDFS=/home/user/cms-pdfs $S/bin/libtest-new --ignored --nocapture pdf::table_oracle
880 members under /home/user/cms-pdfs, 101 from validation/pdf
  ABMP16als118_5_nnlo                      members   30 values    6258240 nonzero    4494517 differ 0
  ...                                      (all 47 rows differ 0: $S/M-V1-oracle-new.log)
  HERAPDF20_NNLO_EIG                       members   29 values   29717344 nonzero   20008292 differ 0
  NNPDF23_lo_as_0130_qed                   members  101 values   30839744 nonzero   24748403 differ 0
  synthetic_mixed                          members    1 values     188928 nonzero     147357 differ 0
  synthetic_two_knot                       members    1 values     141664 nonzero     110575 differ 0
members 984, points by category knot 7615689, seam/edge±ulp 1378087, random 2952000, continuation 986952
compared 413847296 values (288399049 nonzero), differ 0
test result: ok. 1 passed; ... finished in 34.73s           (load1 8.5 -> 10.3)
```

The same result over the tree this session fetched (section 2), from a clean
rebuild after the mutants (`$S/M-V1-oracle-fetched.log`, 10.6 s) and again from
the pixi task's command (`$S/M-V1-cms-gates.log`, 13.9 s): 413847296 values,
differ 0.

### It catches a one-ulp mutation of the production path

`(vh - vl) / xc.dlogx` → `(vh - vl) * (1.0 / xc.dlogx)` in `Band::x_cubic`
(`$S/M-V1-mutant-ulp.patch`, reverted):

```
  CT18NNLO                                 members   59 values   20167616 ... differ 82134
  NNPDF23_lo_as_0130_qed                   members  101 values   30839744 ... differ 172018
  synthetic_two_knot                       ...                              differ 0
compared 413847296 values (288401370 nonzero), differ 1677543
DIFF ABMP16als118_5_nnlo member 0: pdg -4 at x=1e0 Q²=2.0000036179599995e1: Ok(-2.524354896707238e-29) against Ok(2.524354896707238e-29)
test result: FAILED.
```

Every set fails; the two-knot member is unaffected, as it should be, since the
bilinear path builds no cubic (`$S/M-V1-oracle-mutant-ulp.log`).

### It cannot pass vacuously

```
$ SYSTEMATICS_CMS_PDFS=$S/empty-tree … pdf::table_oracle
panicked at vibegraph-lib/src/pdf/table_oracle.rs:754:5:
no member on disk under …/M-V1/empty-tree
test result: FAILED.
$ (SYSTEMATICS_CMS_PDFS unset) … → panicked: SYSTEMATICS_CMS_PDFS: NotPresent
```

### Wiring

- `pixi run validate-cms-pdfs` (long tier, `[tasks]`): the oracle, then
  `cms_arguments_reproduce_madgraphs_weights` and
  `the_pdfas_member_reads_its_own_coupling_and_clamp` with `LHAPDF_DATA_PATH` on
  the same tree. `SYSTEMATICS_CMS_PDFS` defaults to `validation/pdf/cms`.
  Depends on `fetch-pdf` and `fetch-cms-pdfs`. Listed in `validate-deep`.
- The skill's change → gate map names it for interpolator and systematics
  changes, and its long-tier block lists it.
- **Pixi is not installed here, so neither task was run through pixi.** The
  gate task's `cmd` string was read out of `pixi.toml` with `tomllib` and run
  under `bash` from the worktree root (section 3).

## 2. Fetching the 880 members

`validation/pdf/fetch_cms.sh`, on `validation/fetch_common.sh`: consent asked
once for the whole list (`vg_consent`; `VIBEGRAPH_NO_NETWORK` refuses, no
terminal refuses), downloads through `vg_download`, SHA-256 through
`vg_verify_sha`. It reads `pdfsets.json` from `run_01_cms`
(`CMS_PDFSETS_JSON`, default `pp_to_ll`'s); checks each archive against
`archive_sha256` before unpacking and deletes a mismatching one; extracts only
`<set>.info` and the first `members_in_list` members (resolving `./` spellings
from the listing); checks the unpacked `.info` against `info_sha256`; fetches
`pdfsets.index`; marks a finished set `.done` and skips it next time. Pixi:
`fetch-cms-pdfs` (depends on `fetch-refdata`); `/validation/pdf/cms/` is
ignored.

| run | what | result |
|---|---|---|
| no consent, no terminal | 4-set subset | `⊘ not downloading 4 of the 4 CMS PDF sets … no terminal to ask at`, exit 1, nothing written |
| `VIBEGRAPH_NO_NETWORK` set | same | `⊘ $VIBEGRAPH_NO_NETWORK is set`, exit 1 |
| fresh directory | CT18NNLO_as_0116 (1 of 1), CT18ANNLO (1 of 59), HERAPDF20_NNLO_EIG (29, four bands), ABMP16als118_5_nnlo (30) | 12.9 s, exit 0; `diff -r -x .done` against `/home/user/cms-pdfs/<set>`: identical, all four; `pdfsets.index` identical |
| full list, killed by `timeout 25` | resumption | 11 more sets done, then exit 124 mid-list |
| full list, rerun | resumes | skips the 15 done sets, fetches 28, 2m39s, exit 0 |
| whole tree | 43 sets, 880 `.dat` | `diff -r -x .done $S/fetch-test/tree /home/user/cms-pdfs`: no output, exit 0 |
| rerun, `VIBEGRAPH_NO_NETWORK=1` | nothing pending | `⊘ all 43 sets … already in …`, exit 0 (no consent needed) |
| one archive SHA altered in the list | negative control | `!!! checksum mismatch … expected 0015764b… got 9915764b…`, archive deleted, nothing unpacked, exit 1 |

One attempt of the negative control met `curl: (35) Recv failure: Connection
reset by peer` on `pdfsets.index` and stopped with `!!! pdfsets.index was not
acquired`; the retry ran. The script does not retry on its own; rerunning
resumes.

## 3. Independent closing measurements

### Peak RSS

`$S/M-V1-peakrss.py`, my own wrapper: `os.wait4` on exactly the child's pid
(`ru_maxrss`, not M-P1's aggregate `RUSAGE_CHILDREN`), and `/proc/<pid>/status`
`VmHWM` polled every 20 ms as a second reading. Binary: `cargo build -p vibegraph
--profile release-debug --config 'profile.release-debug.debug=0' --bin
vibegraph` at this branch (production code is `428c4d8`'s; the only `src`
change is the `cfg(test)` module line), sha256 `54bef1a4…`. Command:
`$S/M-V1-gen.sh`, `generate grid.bin.zst --seed 7 --nevents 10000 -j N
--systematics "<args>"`, the arguments extracted at run time from
`run_01_cms/command.txt` minus `--result` and `--lhapdf_config`;
`LHAPDF_DATA_PATH` is the tree fetched in section 2. The artifact is M-P1's
(`grid.bin.zst`, sha256 `9d127565…`, `pp_to_ll` cards at lhaid 325300),
copied. Interleaved `-j 1`/`-j 4`, five rounds (`$S/M-V1-rss.log`):

| `-j` | wait4 maxrss, GiB, 5 runs | VmHWM, GiB | wall (s) | user (s) | load1 |
|---|---|---|---|---|---|
| 1 | 0.685 0.685 0.685 0.685 0.685 (717828–718204 kB) | 0.685 ×5 | 23.09 23.50 23.31 25.10 24.10 | 21.98–23.06 | 4.8–6.9 |
| 4 | 0.698 0.699 0.701 0.693 0.688 (721732–734656 kB) | 0.689–0.701 | 13.01 13.06 9.84 13.93 9.08 | 21.93–22.86 | 4.7–7.1 |

Largest: 734656 kB = 0.701 GiB = 0.752 GB. The first output holds 10 000
events and 9 260 000 `<wgt>` lines (926 per event); the five runs at each `-j` are byte-identical, `-j 1` equals
`-j 4`, and both equal M-P1's baseline (`cb1498a`) outputs except for the
`artifact` path line in the header.

### The CMS-arguments gate, re-run

The `validate-cms-pdfs` command string out of `pixi.toml`, under bash,
`SYSTEMATICS_CMS_PDFS=$S/fetch-test/tree` (`$S/M-V1-cms-gates.log`, 127 s
including builds):

```
resolved 43 sets, 880 members in 2.8 s
pp_to_ll            weights 9260000 exact 9260000 one-unit 0 beyond 0 (100.00000% exact)
pp_to_jj            weights 9260000 exact 9260000 one-unit 0 beyond 0 (100.00000% exact)
pp_to_ll_0j2j_mlm   weights 9260000 exact 9260000 one-unit 0 beyond 0 (100.00000% exact)
pp_to_ttx_0j1j_mlm  weights 9260000 exact 9260000 one-unit 0 beyond 0 (100.00000% exact)
cms <run>: header from the loaded members identical=true          (all four)
control the set's coupling for those members: blind (pp_to_ll), DETECTED 4.599e-2, 4.582e-2, 8.179e-2
control MLM terms dropped: blind, blind, DETECTED 5.262e7, 8.708e7
test cms_arguments_reproduce_madgraphs_weights ... ok
test the_pdfas_member_reads_its_own_coupling_and_clamp ... ok
test result: ok. 2 passed; 0 failed
```

The controls are blind exactly where the test's documentation says (no αs power
on `pp_to_ll`, no emission on the unmatched runs).

### Control: a member read through the wrong band

Chosen over the bytes-per-member variant because it exercises the new oracle on
the property it exists for. Two mutations of `LogBicubic::select_band`, each
built, run over the fetched tree, reverted:

1. `b.q2_max < q2` → `b.q2_max <= q2` (`$S/M-V1-mutant-seam.patch`): detected,
   10196650 differences on every set. Coarse: it also refuses the grid's top
   Q² edge (`falls in no subgrid`), so it does not isolate seams.
2. `partition_point(|b| b.q2_max <= q2).min(len - 1)` (`$S/M-V1-mutant-seam2.patch`):
   only a point exactly on an interior seam moves to the upper band. Detected,
   1878622 differences:

```
  HERAPDF20_NNLO_EIG          differ 900254     MSHT20nnlo_as118          differ 911820
  MSHT20nnlo_as_largerange    differ 14028      MSHT20nnlo_as_smallrange  differ 14028
  NNPDF30_lo_as_0118          differ 6074       NNPDF30_lo_as_0130        differ 6898
  NNPDF30_nlo_nf_5_pdfas      differ 7184       NNPDF30_nnlo_as_0118_hessian differ 12856
  synthetic_mixed             differ 5480       (every other set differ 0)
DIFF HERAPDF20_NNLO_EIG member 0: row slot 2 at x=9.9e-7 Q²=2.0448999999999997e0: -2.700149e-1 against 0e0
```

The blind sets are informative. NNPDF31 and NNPDF40 have two bands and NNPDF40
pch three, yet a seam read through the upper band changes no bit:
their duplicated seam rows are identical, and the Hermite reproduces a knot row
exactly at `t = 0` and `t = 1`. On those sets the lower-band rule is not
observable. HERAPDF (the charm threshold, where charm is zero below), MSHT and
NNPDF30 carry seam rows that differ, and there the rule is pinned.

### The item's `closes_when`, clause by clause

> generate with the CMS runcmsgrid.sh systematics request (880 members) on
> pp_to_ll at lhaid 325300 peaks below 2 GB resident, measured, with the
> CMS-arguments weights gate still exact.

| clause | evidence | verdict |
|---|---|---|
| `generate` with CMS's request (880 members) on `pp_to_ll` at lhaid 325300 | the command above: arguments from `run_01_cms/command.txt`, 43 sets / 880 members resolved, 926 weights per event, `pp_to_ll` artifact at 325300 | met |
| peaks below 2 GB resident | 0.685 GiB at `-j 1`, at most 0.701 GiB (0.752 GB) at `-j 4`, 10 runs, two mechanisms agreeing to 0.4 MB | met, 1.25 GB margin; also under D3's 1.5 GB |
| measured | recorded measurement, wrapper, binary hash, input hashes and logs above | met |
| the CMS-arguments weights gate still exact | 4 × 9260000 weights exact, 0 one-unit, headers identical, controls as documented | met |

D4's parts: the CI guard (`pdf_member_bytes`) passes in the default suite
below; the fetch script exists and was run. Closing the backlog item is left to
the manager.

## 4. Gates

| gate | command | result |
|---|---|---|
| table oracle | section 1 | 413847296 values, differ 0 |
| CMS gates | section 3 | 2 passed |
| fmt | `cargo fmt --all --check` | clean |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | clean (one `release-debug`-only `resolve_mom` warning is outside this profile) |
| clippy, gate features | `… --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings` | clean (one `release-debug`-only `resolve_mom` warning is outside this profile)_EXT |
| hermetic | `cargo test --workspace --no-fail-fast` | 51 targets: 1805 passed, 0 failed, 25 ignored, 332 s (load1 4.7–7.2); `pdf_member_bytes` passes |

Cargo ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`, and `release-debug` with `debug = 0` (by
`--config`, or an untracked `.cargo/config.toml` removed afterwards). The banked
layer (`validate.sh`) was not run: nothing in amplitude, eval or coverage code
changed.

## Corrections to the brief

- **The oracle is a lib unit test, not an integration test beside the other
  `#[ignore]` CMS test.** The other is registered by `required-features` in
  `validate_systematics`; this one needs `Continuation` and the clamp, which
  are `pub(crate)`, so it is gated by `cfg(feature = "extended-validation")` as
  the skill prescribes for lib tests.
- **"The two CMS gates"**: the task runs the oracle and the CMS-arguments gate,
  and also `the_pdfas_member_reads_its_own_coupling_and_clamp`, the third
  ignored test that reads a CMS set (325300) and is otherwise run by hand.
- **`86b8e5c`** and `cb1498a` have the same `src/pdf/`; the oracle is taken
  from either.
- **§4.4's per-member, per-reading and heap figures are M-P1's**, which the
  updated section cites through the sprint; the RSS and gate figures are this
  session's.

## Found

- **The PDF grid oracle's multigrid set cannot see the seam rule.** Evidence:
  under the seam2 mutant, `NNPDF31_lo_as_0130` member 0, the member
  `validate_pdf_grid` probes for its seams, has `differ 0`, as do every NNPDF31
  and NNPDF40 set; HERAPDF20, MSHT20 and NNPDF30 detect it. So the lower-band
  rule at a seam (LHAPDF's first-in-range band) is pinned against LHAPDF only by
  the hermetic `subgrid_walk_selects_first_in_range_band` on synthetic bands,
  and against the table form by this oracle. What would close it: an LHAPDF
  probe set whose seam rows differ (HERAPDF20_NNLO_EIG or an MSHT20 member) in
  `validation/pdf/oracle_multigrid.json`.
- **`fetch_cms.sh` has no retry.** A `Connection reset by peer` on one download
  stops the run; rerunning resumes. What would close it: `curl --retry 3` in
  `vg_download`, which every fetch shares.
- **`pdfsets.index` is fetched unpinned.** It is the LHAPDF server's current
  index, with no hash in `pdfsets.json`; the gates read only the 43 sets'
  ids from it. What would close it: record its SHA-256 beside `pdfsets.json`
  when the bundle is next banked, or have the gates build the id map from the
  `.info` `SetIndex` fields.
- **`resolve_mom` dead under `release-debug`** (M-P1's Found) still prints its
  warning in every `release-debug` build.
