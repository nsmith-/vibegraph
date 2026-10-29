---
type: Session Report
title: "S-V1 report: the CMS-arguments gate, the MLM gates in CI, and FG"
description: "LHAPDF's yaml-cpp re-emission of SetDesc explains the CMS header hash; the header is byte-identical from the bundled .info files and from generate; every one of 37.04M CMS weights is MadGraph's string; MadGraph's default-argument MLM runs gated in the banked layer; the dump test's unmatched branch rewritten for the M3; FG narrowed (not the forests, not the channel search) but not settled; 3.76 GB peak RSS for the 880 members."
status: draft
tags: [systematics, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics`, from `db8e9666`.
Implements the [S-V1 brief](S-V1.md). Commits: `ea389944` (the header fix,
the optional `.info` keys, the CMS and MLM default-argument gates, the dump
test's unmatched branch), `895cb57c` (the replay on MadGraph's forests and the
dump test's record diagnostics), and the commit adding this report. Logs are
in the session's scratchpad (`S-V1-*`).

## 1. The CMS-arguments gate

### The header hash

The banked block is `ba7e21bc…`; the design predicted `2cbb8781…`. The cause
is not the folded HERAPDF20 `SetDesc` or the quoted CT18 ones (the brief's
guess): both come back from LHAPDF as the YAML text, as PyYAML read them. It
is how LHAPDF 6.5 stores metadata. `Info::load` (`src/Info.cc`, identical in
6.5.4 and 6.5.6, fetched and compared) parses the block with its bundled
yaml-cpp and stores every value as `YAML::Emitter << node` writes it. yaml-cpp
writes a string plain only where its `IsValidPlainScalar` allows; a string
holding `": "` is written double-quoted. Two `SetDesc`s of the list hold one:
`MSHT20nnlo_as118` (`Web address: www…`) and `PDF4LHC21_40_pdfas` (`41 and
42: alphas…`). MadGraph's `description` is that string, quotes included.

Fix: `SetInfo::lhapdf_description()` ports yaml-cpp's plain-scalar rule and
double-quoted writer (`pdf/grid.rs`); the header reads it, and `ErrorType` is
stored the same way. Pinned by
`pdf::grid::tests::a_set_description_is_what_lhapdf_returns` (the six real
`SetDesc` lines, verbatim, and thirteen yaml-cpp edge shapes).

The control confirms the cause exactly: writing the YAML text instead
reproduces the design's predicted hash.

```
cms header pp_to_ll             weights 926 sha256 ba7e21bc8c3cdaff identical=true
  control: the YAML text as the description: sha256 2cbb8781aabdb04c, 2 line(s) differ (MSHT20nnlo_as118, PDF4LHC21_40_pdfas)
… the same on pp_to_jj, pp_to_ll_0j2j_mlm, pp_to_ttx_0j1j_mlm
test the_cms_header_is_madgraphs_byte_for_byte ... ok
```

### A second defect the gate found

`parse_info` required `Particle` and `ErrorType`. Eleven of the 43 `.info`
files have no `Particle` (all CT18 and MSHT20 sets) and two no `ErrorType`
(the MSHT20 αs-range sets); LHAPDF requires neither. Before this session the
CMS list could not be resolved at all (`MissingKey … CT18ANNLO.info …
"Particle"`). Both are optional now (`ErrorType` absent reads LHAPDF's
`unknown`), pinned by `an_info_without_particle_or_error_type_opens`.

### The gates

`resolve` is split into `open_request` (the `.info` files only) and
`OpenedRequest::load` (the members), so the header can be gated in CI from the
bundled `.info` files.

| gate | where | result |
|---|---|---|
| `the_cms_header_is_madgraphs_byte_for_byte` | banked, default | 4/4 identical, 926 weights, `.info` sha256s checked against `pdfsets.json` |
| `cms_arguments_reproduce_madgraphs_weights` | banked, ignored (needs the sets on `SYSTEMATICS_CMS_PDFS`) | below |

The arguments are read from each run's banked `command.txt`. The 43 sets were
fetched from `lhapdfsets.web.cern.ch` (1.45 GB of archives; only the `.info`
and the 880 members the list reads were kept, 1.2 GB): 43/43 archive and
`.info` sha256s equal the oracle host's `pdfsets.json`.

```
resolved 43 sets, 880 members in 16.0 s
NNPDF31_nnlo_as_0118_mc_hessian_pdfas member 101: AlphaS_MZ 0.116 (set's 0.118)
NNPDF31_nnlo_as_0118_mc_hessian_pdfas member 102: AlphaS_MZ 0.12 (set's 0.118)
  overrides the set's AlphaS_MZ: PDF4LHC21_40_pdfas 41, 42; NNPDF31_nlo_hessian_pdfas 101, 102;
  NNPDF31_nnlo_as_0118_mc_hessian_pdfas 101, 102; NNPDF40_nnlo_pdfas 101, 102; NNPDF40_nnlo_hessian_pdfas 51, 52
```

| run | weights | exact | 1 unit | beyond | header (loaded members) | MLM-terms control | set-coupling control |
|---|---|---|---|---|---|---|---|
| `pp_to_ll` | 9 260 000 | 9 260 000 | 0 | 0 | identical | blind (4.98e-8), as expected | blind (3.55e-8), as expected: no αs power |
| `pp_to_jj` | 9 260 000 | 9 260 000 | 0 | 0 | identical | blind (4.98e-8), as expected | detected (4.60e-2) |
| `pp_to_ll_0j2j_mlm` | 9 260 000 | 9 260 000 | 0 | 0 | identical | detected (5.26e7) | detected (4.58e-2) |
| `pp_to_ttx_0j1j_mlm` | 9 260 000 | 9 260 000 | 0 | 0 | identical | detected (8.71e7) | detected (8.18e-2) |

`--alps` on the MLM runs: moved on 3527 and 5253 events, left at `XWGTUP`
on the other 6473 and 4747; on the unmatched runs left on all 10 000. The
set-coupling control evaluates every member whose header overrides its set's
`AlphaS_MZ` with the set's coupling: the member-level pin on real data.

### End to end through `generate`

`generate` on `pp_to_ll` at lhaid 325300 with `--systematics` and the CMS
arguments writes the banked block (`ba7e21bc…`) and 926 weights per event.
MadGraph 3.7.1's own module (LHAPDF 6.5.4, built in S-F2) on that file with
its weights stripped and the same arguments: `Will compute 926 weights per
event`, `<initrwgt>` identical, 9 260 000 weights, 9 259 925 equal, 75 one
unit, 0 beyond.

## 2. MadGraph's default-argument MLM runs, a banked gate

`madgraphs_default_arguments_on_the_banked_mlm_runs` (runs by default) over
the four `run_01_syst_default` files, arguments asserted from `command.txt`:

```
pp_to_llj_mlm         weights 1470000 exact 1470000 one-unit 0 beyond 0; --alps 8877 moved / 1123 still; control DETECTED (2.610e0)
pp_to_ll_0j2j_mlm     weights 1470000 exact 1470000 …; --alps 3527 / 6473; control DETECTED (2.721e0)
pp_to_ttx_0j1j_mlm    weights 1470000 exact 1470000 …; --alps 5253 / 4747; control DETECTED (1.438e1)
pp_to_llj_xqcut_only  weights 1470000 exact 1470000 …; --alps 0 / 10000; control blind (4.682e-8)
test madgraphs_default_arguments_on_the_banked_mlm_runs ... ok
```

It asserts every weight within one unit, the `--alps` weights moved on every
event with something to vary and still on every other, the count pinned to
the oracle host's (8877/3527/5253/0), and the dropped-terms control detected
exactly where such events exist. `madgraphs_module_on_other_files` shares the
check (`module_gate`).

## 3. The dump test's unmatched branch

`validate_mlm_dumps.rs` now reads, on an unmatched event, the codes and `x`
from the event's `SYST` record (`ipdgcl(j)`, `xbk(ib(j))` unrounded,
`reweight.f:1450-1457`) and the labels from `unwgt.f`'s `flip`
(`p(3,1) < 0` on the momenta the clustering read). Compiles; **not run**
(no dumps here). The M3's run of the old branch is the known-wrong evidence
(labels 4978/10000, `x` 6797 and 6540/10000).

The M3 re-run (refdata-12 now carries every `.inc` file the test reads, so
only the dumps are needed):

```
git fetch origin && git checkout <this branch's tip>
pixi run --skip-deps validate-mlm-dumps 2>&1 | tee validate-mlm-dumps.S-V1.log
```

Pass: every row's gated fields full, `pp_to_llj_xqcut_only` included. Please
post the `mgrwt differs` blocks and the `info: the written event rebuilt is
the clustered point` counts (for FG, below).

## 4. FG

What was measured here:
- **Not the forests.** `validate_mlm_records` now replays every banked record
  on this crate's forests and on MadGraph's `configs.inc`. The two tables are
  equal on all four rows (`ttx_0j1j`: beam-2 `q` 9022/9947, `<totfact>`
  7523/9947 on both).
- **Not the channel search.** With the header filter removed (an uncommitted
  probe), on `ttx_0j1j` 2697 events have no configuration of their subprocess
  whose record is MadEvent's, and no event is reproduced only by a
  configuration the header filter rejects.
- **Not the mirror reading.** Rebuilding a flipped event as a swap of the two
  incoming legs instead of a rotation changes no count (ttx: 7287 events
  reproduced both ways; 1090 flipped).

So the replay's ttx misses come from its inputs (the momenta rebuilt from
the file) or from a rule visible only on events the dump test calls permuted.
Their size (2697) is close to the permuted set (3744), while the M3 dump run
misses only about 110–220 of the permuted events with MadEvent's own inputs.
The dump test's production path is exact on all 6256 non-permuted `ttx`
events, so no production-rule defect is shown. A plausible reading is
MadEvent's registered first-call deviation (H1: `q2bck` and the first call
from the unpermuted `PP`). That is a hypothesis.

**Not settled.** The M3 run above settles it:
- if `info: the written event rebuilt is the clustered point` fails on the
  permuted events, the file does not carry the point MadEvent clustered, and
  the replay cannot gate them;
- the `mgrwt differs` blocks show, per permuted miss, MadEvent's and this
  crate's first-call scales and `q2bck`. Equal there with the record still
  different would mean a production defect; different there means H1.

## 5. Memory and wall time

`generate`, `pp_to_ll` at lhaid 325300, 10 000 events, release-debug
(`S-V1-scale.sh`, Python `getrusage` of the child). A sibling's `integrate`
used about three of the four cores throughout, so the wall times include
contention.

| request | `-j` | wall | user | peak RSS |
|---|---|---|---|---|
| the card's own (`errorset` of 325300, 147 weights) | 1 | 6.1 s | 5.3 s | 0.44 GB |
| CMS, 880 members, 926 weights | 1 | 54.6 s | 34.6 s | **3.76 GB** |
| the card's own | 4 | 4.9 s | 5.9 s | 0.45 GB |
| CMS | 4 | 27.0 s | 34.1 s | **3.77 GB** |

The `-j 1` and `-j 4` CMS files are byte-identical. 3.76 GB measured against
§4.4's 3.1–3.5 GB estimate, and CMS's 2 GB slot (O2). The gate's own process
peaked at 4.83 GB.

## Also

- **FH.** `group_resonances` and its call are byte-identical on `main`
  (`f5fb3e65`) and here, and the branch does not change what reaches it. So
  `main` refuses the card too, **by inspection; not run** (building `main` in
  a scratch worktree is a full release build on a disk with about 8 GB free).
- **The banked targets** for LHEF, PDF, scale and MLM, on the pinned
  refdata-12: below.

## Gates

`S-V1-banked.sh`: one cargo invocation per target, resumable, release-debug
where pixi uses it, `CARGO_BUILD_JOBS=2`, on `895cb57c`:

```
fmt                        cargo fmt --all --check                                   rc=0
clippy                     cargo clippy --workspace --all-targets -- -D warnings     rc=0
test-workspace             cargo test --workspace --no-fail-fast                     rc=0
                           49 suites: 1756 passed, 0 failed, 19 ignored
clippy-ext                 … --features vibegraph/extended-validation,vibegraph-lib/extended-validation   rc=0
validate_alphas            FAILED. 1 passed; 1 failed   (the new row, below)
validate_scales            FAILED. 4 passed; 6 failed   (the new row, below)
validate_scale_couplings   ok. 1 passed
validate_kt_cluster        ok. 0 passed; 2 ignored (the oracle part needs the dumps: not run)
validate_pdf_grid          ok. 20 passed
validate_mlm_dumps         ok. 0 passed; 3 ignored (needs the dumps: not run)
validate_mlm_records       ok. 1 passed (ttx beam-2 q 9022/9947, <totfact> 7523/9947 on both forests, as S-F2)
validate-mlm-samples       ok. 1 passed: Z +0.44, jet at sqrt(s) +0.01, <asrwt> 0.8877 vs 0.8877, chains 0.4438 vs 0.4439
validate_lhef              ok. 3 passed
validate_unweighting       ok. 1 passed
validate_hadronic          ok. 20 passed; 26 ignored
validate_systematics       ok. 4 passed; 3 ignored (S-F1's gates unchanged; the CMS header; the default-argument MLM gate)
validate_systematics.cms   ok. 1 passed (the 37.04M CMS weights, ignored target run with SYSTEMATICS_CMS_PDFS), 116 s
cli_generate_proton        ok. 8 passed; 2 ignored
cli_systematics            ok. 4 passed
cli_reweight_proton        ok. 2 passed
validate_samples_proton    ok. 12 passed; 6 ignored
cli_generate               ok. 17 passed
validate-mlm-sigma         ok. 5 passed (3673 s): llj_mlm +1.00, xqcut_only +0.85, alps2 +0.88,
                           ttx_0j1j total +0.75 (@0 +0.28, @1 +0.71), ll_0j2j total +1.54 (@0 +0.50,
                           @1 +0.37, @2 +2.08); ttx total 1092.332, every value as S-F2 measured
validate_sigma             ok. 8 passed; 33 ignored (906 s)
python3 scripts/kb.py lint 1008 concepts; 0 errors, 4 warnings (backlog, pre-existing)
```

**The refdata-12 pin breaks two banked targets, independently of this
session's code.** `validate_alphas` and `validate_scales` fail on a banked run
neither inventory declares: `ee_to_tth_ymt250`, the param-card sprint's row,
which the cut carries (`cd038fd1`). `validate_alphas`: the agreeing set is the
declared set plus exactly that row, so nothing else moved. `validate_scales`:
all six failures are the inventory assertion, `banked run ee_to_tth_ymt250 is
in none of this gate's inventories`, which stops those gates before they
measure anything. With the row added locally beside `ee_to_ttx` in both
inventories (uncommitted, reverted), `validate_alphas` passes 2/2 and
`validate_scales` 10/10. Which inventory the row belongs in is that row's
owner's call, and `sprinte-param-card` does not declare it either: Found FT.

Not run: the validation report's collate step, `validate-pythia` and
`validate-mlm-pythia` (no Pythia here), and the dump-reading targets (no
dumps).

## Mutations

`S-V1-mut.sh`, exact replacement and restore (`git diff --stat` on the file
empty after):

| mutation | fails |
|---|---|
| M1 `lhapdf_description` returns the YAML text | `pdf::grid::tests::a_set_description_is_what_lhapdf_returns` (`15 passed; 1 failed`), `validate_systematics::the_cms_header_is_madgraphs_byte_for_byte` |
| M2 `Particle` required again | `pdf::grid::tests::an_info_without_particle_or_error_type_opens` (`15 passed; 1 failed`) |

## Corrections to the brief

- The header difference is the `": "` quoting of two `SetDesc`s, not the
  HERAPDF20 fold or the CT18 quotes, which LHAPDF and PyYAML read alike.
- "About 4 GB" is the sets installed whole. The list reads 880 members:
  1.45 GB of archives to download, 1.2 GB kept.
- The design's §4.3 list of member-level `AlphaS_MZ` overrides misses
  `PDF4LHC21_40_pdfas` members 41 and 42.

## Found

- **FM. 880 members take 3.76 GB resident in `generate`**, against CMS's 2 GB
  slot (measured above; O2). The precomputed bicubic coefficients are about
  4 MB per member. Closes with a decision: per-member storage nearer LHAPDF's
  (raw knots, coefficients on demand or as `f32`), or the requirement
  relaxed. A re-measurement on the same command settles either.
- **FN. FG is open** (§4). Closes with the M3 run above.
- **FO. The CMS weights gate has no fetch task.** It needs the 880 members
  (scratch `S-V1-fetch.sh`: stream each archive, check its sha256 against
  `pdfsets.json`, keep the `.info` and the members read). A pixi task doing
  that would let the long tier run it.
- **FP. Knowledge bundle**: design §3.4 and §7 attribute the predicted hash to
  YAML parsing (it is LHAPDF's re-emission), §4.3 misses PDF4LHC21 41/42, and
  §4.4's memory estimate is now measured. For the close-out session.
- **FQ. Other `.info` entries are not re-emitted.** Only `SetDesc` and
  `ErrorType` go through the yaml-cpp port; LHAPDF re-emits every entry. No
  other entry reaches MadGraph's output today; a null or numeric `SetDesc`
  also reads as `scalar_text`, not yaml-cpp's `~` or original spelling.
- **FR. A non-test build warns `resolve_mom` is never used**
  (`helas/eval/analysis.rs:208`, also on `main`); CI's
  `clippy --all-targets` cannot see it because the tests use it.
- **FS. `generate` refuses `pp_to_ttx_0j1j_mlm`** on `main` too, by
  inspection (FH above). Closes with a matched `t t~ j` sample generated and
  a test.
- **FT. `ee_to_tth_ymt250` is undeclared in `validate_alphas` and
  `validate_scales`** on this branch and on `sprinte-param-card`, so both
  banked targets fail on refdata-12 (Gates above). Closes with the row
  declared (locally, as clustered beside `ee_to_ttx`, both pass) on whichever
  branch merges first.
