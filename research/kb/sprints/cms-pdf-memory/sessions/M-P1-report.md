---
type: Session Report
title: "M-P1 report: CMS's 880 members in 0.69 GB, bit-identical"
description: "The 3.59 GB peak was 83% removable derived tables (coefficients 66%, a second copy of the grid 17%); a member is now its knots with the x cubic built on use, 404M interpolated values bit-identical to the old layout on 984 members, generate's LHE byte-identical, peak RSS 0.685 GB at -j 1 and 0.689 GB at -j 4, no slower end to end; a counting-allocator test pins bytes per knot."
status: draft
tags: [pdf, memory, cms, report, sprint-f]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/M`, branch `sprintf-pdf-memory`, from `cb1498a`.
Commit: the layout, the guard test and this report, one commit on the branch.
Implements the [M-P1 brief](M-P1.md) under the user's
[D1](../decisions/D1-lossless-knots-first.md). Scratch files are under
`/tmp/claude-0/-home-user-vibegraph/7497b6d2-0784-5f2c-a2c9-2ed0beaa757e/scratchpad/M-P1/`
(`$S` below): the cards, the harness sources (`m_p1_attrib.rs`,
`m_p1_bits.rs`, built as `vibegraph-lib` examples and not committed), the
baseline binaries and every log quoted here.

**Host.** Linux container, 4 cores, 15 GB. The sibling session in
`/home/user/wt/G` was building throughout: 1-minute load 1.6–6.0 during every
timing (each line below carries it as `load1`).

## 1. Attribution (before any code change)

`generate` built at `cb1498a` (`cargo build --profile release-debug -p vibegraph
--bin vibegraph`), kept as `$S/bin/vibegraph-baseline`. The artifact: MadGraph's
`pp_to_ll` cards with `lhaid` 325300 (`$S/cards`), `vibegraph integrate … -j 4`,
σ = 2120.21 ± 1.84 pb. The item's command (`$S/M-P1-gen.sh`): `generate grid.bin.zst
--seed 7 --nevents 10000 -j N --systematics "<the arguments of run_01_cms/command.txt>"`,
peak RSS from Python's `getrusage(RUSAGE_CHILDREN)` (`$S/M-P1-rusage.py`).

```
base-j1 exit=0 wall=32.96s user=24.82s sys=8.06s maxrss=3.588GB load1=4.35
base-j4 exit=0 wall=15.66s user=23.69s sys=3.86s maxrss=3.608GB load1=5.08
```

Heap attribution with a counting global allocator over the same 880 members
(`$S/bin/attrib-baseline members`, then `resolve` under `RAYON_NUM_THREADS=1|4`;
log `$S/M-P1-attrib-baseline.log`):

```
distinct members 880
total .dat bytes 1140.8 MB, members kept 880
retained heap 3576.2 MB (4.064 MB/member)
largest transient over a member load beyond retained: 3.41 MB
knots (x·f values) 78206183 = 596.7 MB as f64
  coeffs                   2372.3 MB   66.3%
  LogBicubic xf (copy)      596.7 MB   16.7%
  SubGrid xf                596.7 MB   16.7%
  SubGrid x+q2                2.6 MB    0.1%
  logxs+logq2s                2.6 MB    0.1%
  qcells                      2.2 MB    0.1%
  everything else             3.2 MB    0.1%
max bands 4; two-knot bands: none
resolve threads=1 members=880 9.8s: live heap 3.8 MB -> 3584.3 MB (+3580.5), peak heap 3586.5 MB, RSS 7.4 -> 3598.1 MB, HWM 3598.1 MB
resolve threads=4 members=880 4.6s: live heap 3.8 MB -> 3584.3 MB (+3580.6), peak heap 3586.5 MB, RSS 7.5 -> 3647.3 MB, HWM 3647.3 MB
```

Every member costs 47.9–48.0 bytes per knot (six `f64`), every set alike. The
field sizes are computed from each member's shapes; their sum is the measured
retained heap to 3.2 MB.

- **Coefficients 66.3%, the interpolator's copy of the grid 16.7%**: 83.0% of
  the peak, all of it removed by the knot-only layout. The stop condition (under
  about 60%) does not apply.
- **The grid itself is held twice**: `PdfMember` kept its `Vec<SubGrid>`, and
  `LogBicubic` cloned `xf` for the bilinear fallback. The brief counted five
  `f64` per knot; it is six.
- **Parse transients** are not a factor: at most 3.4 MB above the retained heap
  during one member's load, freed before the next.
- **Allocator**: RSS exceeds live heap by 14 MB at `-j 1` and 63 MB at `-j 4`
  (glibc arenas and fragmentation), 0.4% and 1.7% of the peak.
- **Other copies**: none; `everything else` is 3.2 MB over 880 members.

## 2. The layout

`vibegraph-lib/src/pdf/interp.rs`, `mod.rs`, `extrap.rs` (tests only).

- `LogBicubic` owns the member's `Vec<SubGrid>`; `PdfMember::subgrids()` reads
  them from it, so the grid is held once. Per band it keeps a `Band`: the
  log-knot axes, the bounds, the flavour maps and the Q² stencil constants
  (`QCell`, kept: 56 bytes per Q² interval, 2.2 MB over the list).
- `coeffs` and the `xf` copy are gone. `Band::x_cubic` builds `[a, b, c, d]` at
  the point from the four `x·f` values around the x interval, with LHAPDF's
  `_ddx` and `_computePolynomialCoefficients` expressions in their order, and
  runs the same Horner chain. The one-sided slopes at the first and last x
  knot are selected per point through an `XCell` (clamped neighbour, the
  interval's own spacing), mirroring `QCell`; each selected branch is the old
  `ddx_logx` expression term for term.
- The bilinear fallback reads the same `xf`.
- Parse transients were left as they are (3.4 MB, freed per member, section 1).

**Classification: order-preserving.** Every operation is the old one on the
same operands; only when it happens moved. Gated bit for bit.

## 3. Gates

### Bit-identical interpolation

Scratch harness against the baseline build (`$S/m_p1_bits.rs`): `dump` on the
`cb1498a` build wrote every value to disk (3.1 GB, `$S/bits`); `check` on the
new build recomputes and compares `to_bits`, value for value. Per member: the
all-flavour row (`xfx_all`, 16 slots) and `xfx_q2` for 16 PDG codes (including
0, 22 and the absent 7) at:

- every knot of every band;
- each band's lower and upper Q² edge (so every seam), and one ulp either side,
  at 40 random x; every x knot, one ulp either side;
- 3000 log-uniform random points over the grid;
- the continuation: below `x_min`, below `Q²_min`, above `Q²_max`, the
  corners, and one ulp past `x_min`, `Q²_min`, `Q²_max`.

Members: all 101 of NNPDF23_lo_as_0130_qed, all 880 of the CMS list (43 sets:
ABMP16, CT18 ×8, HERAPDF2.0, MSHT20 ×3, NNPDF3.0/3.1/4.0, PDF4LHC21), and three
in-memory members. Multi-Q²-subgrid sets: HERAPDF20_NNLO_EIG (4 bands), MSHT20
(3), NNPDF30 and NNPDF40 pch (3), NNPDF31 and NNPDF40 (2). **No real set in
the list has a two-knot band**, nor NNPDF23 (`two-knot bands: none` above), so
the in-memory members cover the bilinear path: `synthetic_mixed` (a Hermite
band, a two-knot band, a Hermite band), `synthetic_two_knot`, and
`synthetic_three_knot` (three Q² knots, both one-sided Q² slopes).

```
$ SYSTEMATICS_CMS_PDFS=/home/user/cms-pdfs $S/bin/bits-new1 check $S/bits
mode check: members 984, points by category (knot, seam/edge±ulp, random, continuation) [7615689, 1070171, 2952000, 986952]
  ABMP16als118_5_nnlo                           members   30 values    6163200 differ 0
  ...                                           (every set: differ 0; $S/M-P1-bits-check-new1.log)
  HERAPDF20_NNLO_EIG                            members   29 values   28978656 differ 0
  NNPDF23_lo_as_0130_qed                        members  101 values   30516544 differ 0
  synthetic_mixed                               members    1 values     185376 differ 0
  synthetic_two_knot                            members    1 values     140480 differ 0
compared 403993984 values (281838801 nonzero), differ 0
```

Why it cannot pass vacuously: the value count per member is asserted equal to
the dump's; 70% of the values are nonzero; and one ulp of reassociation in the
new code — `(vh - vl) * (1.0 / xc.dlogx)` for `(vh - vl) / xc.dlogx` — fails it:

```
DIFF synthetic_mixed 0 value 27532: old 3.235036856804145e1 new 3.235036856804146e1
  CT18NNLO                                      members   59 values   19863648 differ 81811
  NNPDF23_lo_as_0130_qed                        members  101 values   30516544 differ 171900
compared 403993984 values (281838801 nonzero), differ 1670286
```

(`$S/M-P1-bits-check-mutant.log`; the mutation was reverted.)

End to end, every `generate` output of the new build is byte-identical to the
baseline's on the same seed: 10/10 runs below (`cmp`).

### CMS gate and the default gates

`cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_systematics -- --ignored --nocapture cms_arguments_reproduce_madgraphs_weights`
with `SYSTEMATICS_CMS_PDFS=/home/user/cms-pdfs`:

```
pp_to_ll            weights 9260000 exact 9260000 one-unit 0 beyond 0 (100.00000% exact)
pp_to_jj            weights 9260000 exact 9260000 one-unit 0 beyond 0 (100.00000% exact)
pp_to_ll_0j2j_mlm   weights 9260000 exact 9260000 one-unit 0 beyond 0 (100.00000% exact)
pp_to_ttx_0j1j_mlm  weights 9260000 exact 9260000 one-unit 0 beyond 0 (100.00000% exact)
test cms_arguments_reproduce_madgraphs_weights ... ok
```

Headers identical on all four; the controls as in S-V1 (set coupling: blind on
`pp_to_ll`, detected 4.60e-2, 4.58e-2, 8.18e-2; MLM terms: detected 5.26e7,
8.71e7 on the MLM runs).

| gate | command | result |
|---|---|---|
| systematics (default) | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_systematics` | 4 passed, 3 ignored |
| PDF grid oracles | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_pdf_grid` | 20 passed |
| guard | `cargo test -p vibegraph-lib --test pdf_member_bytes` | passed (below) |
| fmt | `cargo fmt --all --check` | clean |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| clippy, gate features | `… --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings` | clean |
| hermetic | `cargo test --workspace --no-fail-fast` | 51 targets: 1805 passed, 0 failed, 25 ignored |
| member coupling (ignored) | `LHAPDF_DATA_PATH=/home/user/cms-pdfs cargo test … --test validate_systematics -- --ignored the_pdfas_member_reads_its_own_coupling_and_clamp` | passed |

The PDF grid oracles' multigrid set, NNPDF31_lo_as_0130, was not in
`validation/pdf`; it was linked in from `/home/user/cms-pdfs` (member 0, the
oracle's) for the run and removed after.

### Guard (D4)

`vibegraph-lib/tests/pdf_member_bytes.rs` installs a counting global allocator
(one test in the binary, so nothing else allocates while it measures) and
asserts at most 10 bytes per knot (one `f64` plus a quarter) for a synthetic
member shaped like an NNPDF one (100 x knots, three bands, 13 flavours) built
from scratch, and, where `validation/pdf` has it, members 0, 1 and 100 of
NNPDF23 loaded from their `.dat`.

On the new layout:

```
synthetic member: 57200 knots, 467124 bytes held, 8.17 bytes/knot
NNPDF23_lo_as_0130_qed member 0: 70000 knots, 567459 bytes held, 8.11 bytes/knot
test result: ok. 1 passed
```

On the baseline layout (`git stash` of `src/pdf`, same test file):

```
synthetic member: 57200 knots, 2736964 bytes held, 47.85 bytes/knot
a synthetic member holds 47.85 bytes per knot, over 10: it carries a derived table beside its knots
test result: FAILED. 0 passed; 1 failed
```

A single extra per-knot `f64` table (16 bytes per knot) fails it too.

### Memory (D3) and time (D2)

Five repeats, interleaved base/new at each `-j`, same artifact and seed
(`$S/M-P1-interleave.sh`, log `$S/M-P1-interleave-new1.log`):

| build | `-j` | wall (s), 5 runs | median wall | median user | peak RSS (GB), 5 runs |
|---|---|---|---|---|---|
| baseline | 1 | 26.89 25.82 25.60 25.16 26.75 | 25.82 | 23.79 | 3.588 3.588 3.587 3.587 3.587 |
| new | 1 | 23.88 23.84 23.96 22.90 24.02 | 23.88 | 23.12 | **0.685** ×5 |
| baseline | 4 | 8.57 8.76 8.28 11.03 9.81 | 8.76 | 22.92 | 3.591 3.592 3.607 3.602 3.592 |
| new | 4 | 8.60 8.55 8.58 7.94 7.96 | 8.55 | 22.26 | **0.689 0.694 0.689 0.688 0.689** |

load1 1.6–3.8 over these runs. Peak RSS falls 5.2×, to 0.685–0.694 GB, under
D3's 1.5 GB at both `-j`. Wall time is 0.92× at `-j 1` and 0.98× at `-j 4`;
system time falls (1.7–2.1 s to 0.5–1.5 s: fewer pages to fault in).

The new build's heap over the list (`$S/bin/attrib-new1`, log
`$S/M-P1-attrib-new.log`): 607.2 MB retained, 0.690 MB per member, against
596.7 MB of knots; `resolve` peaks at RSS 622 MB (`-j 1`) and 630 MB (`-j 4`).

**Per-member evaluation cost.** All 880 members × 2000 log-uniform points
(x ∈ [1e-4, 0.5], Q² ∈ [1e2, 1e4]), `xfx_all`, two interleaved rounds of four
repeats (`$S/M-P1-bench.log`, load1 5.6–5.8):

| build | ns per all-flavour reading |
|---|---|
| baseline | 173.7–177.2 |
| new | 315.4–328.4 |

1.83× per reading in isolation. It does not show end to end because PDF
readings are a small share of this command (below).

## Corrections to the brief

- **Six `f64` per knot, not five.** The grid was held twice (`PdfMember`'s
  subgrids and `LogBicubic`'s `xf`); the knots alone are one sixth of the heap.
- **The command is not dominated by PDF readings here.** Baseline user time is
  23–25 s at both `-j`; the 880-member readings over 10 000 events cost about
  `10⁴ × 880 × 2 × 175 ns ≈ 3 s` of it. The S-V1 figures quoted in the sprint
  (54.6 s wall, 34.6 s user) were taken under a sibling's integrate on three of
  four cores. Doubling the per-reading cost therefore did not move the wall time.
- **No two-knot band exists in any of the 43 sets or NNPDF23**; the bilinear
  path is gated on in-memory members only.

## Found

- **The per-reading cost doubled (1.83×) and is recoverable without losing
  exactness.** The x cubic is rebuilt per flavour with three divisions, four
  times per point. Evidence: the bench table above. What would close it: build
  the four Q²-knot rows for all flavours at once over the contiguous `(ix, iq,
  ·)` rows so the divisions vectorise, and skip the duplicated `iq_lo == iq` /
  `iq_hi == iq + 1` rows at band edges; gate with the same bit harness. Worth it
  only for a command whose time PDF readings dominate (the hadronic integrand
  reads one member; this command's readings are ~3 s of ~24 s).
- **The bit-identity harness is not in the tree.** It lives in `$S`
  (`m_p1_bits.rs`, 3.1 GB of baseline dumps). M-V1 can re-run it on this host;
  a future change to the interpolator has no committed bit-level oracle beyond
  the LHAPDF probe points of `validate_pdf_grid` and the eight printed digits of
  the weights gate. What would close it: an `#[ignore]` test carrying a
  precomputed-table oracle, run over the CMS sets in the long tier.
- **`resolve_mom` is dead code under `--profile release-debug`** (`warning: method
  resolve_mom is never used`, `vibegraph-lib/src/helas/eval/analysis.rs:208`) at
  `cb1498a`, before this change; clippy's dev profile does not see it.
