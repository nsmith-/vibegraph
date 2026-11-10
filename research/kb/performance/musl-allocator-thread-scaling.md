---
type: Measurement
title: musl allocator and thread scaling
description: "Static musl builds of integrate ran slower at -j 4 than at -j 1 because musl's allocator serialises on a lock; mimalloc as the musl global allocator restores glibc-level scaling. Also measures glibc builds (dynamic, 2.17 floor, static) as the alternative."
status: draft
tags: [performance, musl, glibc, allocator, mimalloc, jemalloc, threads, rayon, release]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: cern, resource: "https://github.com/nsmith-/vibegraph/releases/tag/v0.2.0", title: "v0.2.0 release assets, as first benchmarked on the 4-vCPU Ice Lake host"}
  - {id: release-yml, resource: ".github/workflows/release.yml", title: "release.yml build matrix (the musl legs)"}
  - {id: cli-main, resource: "vibegraph-cli/src/main.rs", title: "the musl-only #[global_allocator]"}
measured:
  - {commit: c3867684, host: "4-vCPU KVM Intel Xeon @ 2.10 GHz, Ubuntu 24.04 (glibc 2.39, musl 1.2.4), Rust 1.97.0, quiet (timed runs one at a time)", command: "vibegraph integrate <p p > j j card> --fixed-budget --neval 48000 --niter 10 --seed 12345 -j {1,2,4} under /usr/bin/time -v, 3 rounds per variant, allocator variants built from a source copy differing only in the global allocator"}
  - {commit: e1fc2cdb, host: "same host", command: "same command, the release.yml musl legs built from this commit (cargo build --release --locked --target x86_64-unknown-linux-musl --package vibegraph, with and without RUSTFLAGS=-C target-cpu=x86-64-v3), 5 rounds against the glibc build of c3867684"}
---

# musl allocator and thread scaling

The Linux release binaries are static `x86_64-unknown-linux-musl` builds,
baseline and `x86-64-v3`.[^release-yml] With musl's own allocator, `vibegraph
integrate` slowed down as threads were added: `-j 4` took several times the
`-j 1` wall, with system time of the order of user time. The cause is lock
contention in musl's allocator. `vibegraph-cli` therefore declares mimalloc as
the `#[global_allocator]` on `target_env = "musl"` only.[^cli-main]

## Measurements

`p p > j j` (vibegraph's default run card: 6500 GeV beams, NNPDF23 LO), fixed
budget 482,554 evaluations over 10 iterations, seed 12345. Wall is the median of
three rounds with the range in brackets. Context switches are voluntary plus
involuntary, per second of wall.

| build | -j | wall s | user s | sys s | ctx sw/s | speedup |
|---|--:|--:|--:|--:|--:|--:|
| musl, system allocator | 1 | 24.60 [23.67–24.79] | 24.3 | 0.1 | 4 | 1.00 |
| same | 2 | 75.08 [72.36–75.35] | 101.5 | 28.1 | 22k | 0.33 |
| same | 4 | 140.26 [139.21–145.85] | 206.6 | 206.2 | 73k | 0.18 |
| musl-v3, system allocator | 1 | 23.46 [23.03–24.00] | 23.1 | 0.1 | 4 | 1.00 |
| same | 4 | 150.00 [143.40–158.40] | 207.2 | 219.6 | 75k | 0.16 |
| musl + mimalloc | 1 | 16.49 [16.27–19.03] | 16.3 | 0.1 | 5 | 1.00 |
| same | 2 | 9.10 [8.80–9.54] | 17.5 | 0.1 | 14 | 1.81 |
| same | 4 | 5.07 [4.50–5.79] | 18.9 | 0.1 | 108 | 3.25 |
| musl-v3 + mimalloc | 1 | 16.62 [15.88–17.20] | 16.4 | 0.1 | 4 | 1.00 |
| same | 4 | 4.47 [4.42–4.51] | 16.6 | 0.1 | 94 | 3.72 |
| musl + jemalloc (tikv-jemallocator 0.6) | 1 | 16.96 [16.40–17.29] | 16.8 | 0.1 | 4 | 1.00 |
| same | 4 | 4.56 [4.43–4.93] | 17.0 | 0.1 | 98 | 3.72 |
| glibc, dynamic (native, needs glibc 2.39) | 1 | 19.19 [18.50–19.49] | 18.9 | 0.1 | 4 | 1.00 |
| same | 2 | 9.85 [9.51–10.26] | 19.0 | 0.1 | 13 | 1.95 |
| same | 4 | 5.06 [4.94–5.18] | 19.1 | 0.1 | 98 | 3.79 |
| glibc, static (`+crt-static`) | 1 | 18.24 [17.22–18.65] | 18.0 | 0.1 | 5 | 1.00 |
| same | 4 | 5.11 [4.93–5.35] | 18.9 | 0.1 | 83 | 3.57 |
| glibc 2.17 floor (`cargo zigbuild`, zig 0.17.1) | 1 | 19.20 [18.35–19.70] | 18.9 | 0.0 | 4 | 1.00 |
| same | 4 | 4.96 [4.95–5.32] | 18.6 | 0.1 | 83 | 3.87 |
| glibc + mimalloc | 1 | 16.34 [15.76–16.76] | 16.1 | 0.1 | 4 | 1.00 |
| same | 4 | 4.63 [4.60–4.74] | 17.4 | 0.1 | 93 | 3.53 |

The musl release legs as built at `e1fc2cdb`, five rounds each, interleaved
with the dynamic glibc build:

| build | -j | wall s (median, all five) | sys s | speedup | wall / glibc |
|---|--:|---|--:|--:|--:|
| musl + mimalloc | 1 | 17.66 (17.05, 17.64, 17.66, 17.94, 19.23) | 0.10 | 1.00 | 0.91 |
| same | 2 | 10.10 (9.62–10.57) | 0.06 | 1.75 | 0.92 |
| same | 4 | 5.11 (4.98, 5.08, 5.11, 5.51, 5.74) | 0.08 | 3.46 | 0.96 |
| musl-v3 + mimalloc | 1 | 17.01 (16.84, 16.97, 17.01, 17.11, 18.85) | 0.05 | 1.00 | 0.88 |
| same | 2 | 9.15 (8.73–10.01) | 0.07 | 1.86 | 0.84 |
| same | 4 | 4.68 (4.57, 4.67, 4.68, 5.19, 5.25) | 0.08 | 3.63 | 0.88 |
| glibc, dynamic | 1 | 19.34 (19.04–20.68) | 0.05 | 1.00 | 1 |
| same | 2 | 10.93 (10.60–11.01) | 0.05 | 1.77 | 1 |
| same | 4 | 5.34 (5.25–6.03) | 0.06 | 3.62 | 1 |

The two sweeps differ by up to 7% for the same allocator, which is this
host's drift between sessions. Compare rows within one table.

On the 4-vCPU Ice Lake host that first showed the problem, the v0.2.0
`-v3` asset read 26.6 / 43.4 / 63.9 s at `-j 1/2/4` against a glibc build's
21.7 / 5.8 s at `-j 1/4`.[^cern] It degraded less than this host does, but in
the same way.

## Attribution

The controlled A/B is the allocator alone: the musl variants above differ only
in the `#[global_allocator]`, and swapping it moves `-j 4` from 140 s to 5 s
with the same artifact. A syscall count under `strace -f -c` at `-j 4`
(`--neval 48000 --niter 2`) shows where the time goes:

| build | futex calls (errors) | futex s | mmap / munmap calls |
|---|--:|--:|--:|
| musl, system allocator | 4,239,061 (972,484) | 214.2 | 91,646 / 91,480 |
| musl + mimalloc | 152 (14) | 0.33 | 9 munmap |
| glibc | 135 (13) | 0.25 | 14 munmap |

musl's allocator takes one process-wide lock per call and returns freed spans
to the kernel at once. Every worker allocates in the per-point loop, so the
workers queue on that lock and churn `mmap`/`munmap`. The single-thread gap
(24.6 s against glibc's 19.2 s) is in user time with no system time, so it is
the cost of musl's allocator code itself, not of the lock. mimalloc removes
both. It also beats glibc's allocator at `-j 1` (16.3 s against 19.2 s), so
this binary's allocation pattern is a measurable part of a single-thread run
on any libc.

## Correctness

The allocator changes no arithmetic. The `grid.bin.zst` artifact is
byte-identical across allocators and thread counts within one libc and ISA
level:

| process | musl builds | musl-v3 builds | glibc builds |
|---|---|---|---|
| `p p > j j` (σ = 678.85 ± 0.97 µb) | `74139b1d…`: system allocator, mimalloc, jemalloc, at every `-j` | `2f364b1a…`: system allocator, mimalloc | `14782786…`: dynamic, static, 2.17 floor, mimalloc |
| `e+ e- > u u~ g`, 500 GeV, 4 channels (σ = 345.36 ± 0.52 fb) | `1c9e993e…`: system allocator, mimalloc at `e1fc2cdb`, `-j 1` and `-j 4` | `068a5635…`: system allocator, mimalloc at `e1fc2cdb` | `f5ac6614…`: dynamic at `c3867684` and `e1fc2cdb` |

The digests differ between musl and glibc and between baseline and `-v3`, with
σ equal to every printed digit. libm and the instruction set change the last
bits, which no allocator choice affects.

## The glibc alternative

- **Dynamic, native build.** It scales, but a binary built on Ubuntu 24.04 needs
  `GLIBC_2.39` (`pidfd_spawnp`, `pidfd_getpid` from std; `GLIBC_2.34` for
  `pthread_*` and `__libc_start_main`). It does not start on EL7, EL8 or EL9.
- **glibc 2.17 floor via `cargo zigbuild --target x86_64-unknown-linux-gnu.2.17`.**
  It builds on the ordinary runner with zig from PyPI. Its highest required
  symbol version is `GLIBC_2.17`. It needs `libc`, `libm`, `libpthread` and
  `libdl`, but not `libgcc_s`. It ran `p p > j j` under CentOS 7's
  `ld-2.17.so` and libraries, where the native build fails on `GLIBC_2.34`.
  The C dependencies (zstd, ring, mimalloc) are then compiled by `zig cc`
  rather than gcc. It is 13.9 MB against 16.2 MB native. It runs at glibc
  speed, about 15% slower single-threaded than musl + mimalloc.
- **Static glibc (`-C target-feature=+crt-static`).** It links, and no linker warning
  appears in the cargo output. The binary still carries glibc's
  NSS loader (`libnss_%s.so`, `/etc/nsswitch.conf`). A PDF download without a
  proxy reads `nsswitch.conf`, `resolv.conf`, `hosts` and `gai.conf` and asks
  nscd. It references `getaddrinfo` and `getpwuid_r`. On a host whose
  `nsswitch.conf` names a service outside glibc's built-in `files` and `dns`
  (`sss`, `resolve`, `myhostname`), static glibc `dlopen`s that host's NSS
  module, which must match the glibc it was linked against. That host case is
  inferred, not run.

So musl with mimalloc is static and runs on any kernel. It is as fast as glibc
at `-j 4` and faster at `-j 1`. At the time of the measurement it added no C
source the binary did not already compile, because egglog's default features
built mimalloc; `vibegraph-lib` has since turned those off, so mimalloc is now
compiled only for the musl CLI. It keeps
the asset names. The binary grows by 179 KB (16,390,832 to 16,569,416 bytes);
jemalloc adds 677 KB and a new C build.

## Scope: musl only

glibc and macOS keep their system allocators. glibc scales without help. On
macOS, two open heap-corruption aborts are diagnosed against libmalloc's own
checks, and changing the allocator there would change what those checks can
see. The 15% single-thread gain glibc + mimalloc shows here is the reason to
reconsider applying it on every target.
