---
type: Measurement
title: "Systematics weights against MadGraph's systematics module"
description: "On 20 000 pp_to_llj events, CMS's 926-weight request costs vibegraph 38.6 s against 455 s for MadGraph's systematics module on x86 at v0.2.1 (11.8×), and 14.6 s against 256 s on the M3 Max (17.5×), at about a quarter of the memory."
status: draft
tags: [performance, madgraph-comparison, systematics, pdf, benchmarks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
measured:
  - {commit: e4a90128, tag: v0.2.1, host: "Intel Xeon (Ice Lake), 4-vCPU KVM guest, AlmaLinux 10.2", command: "taskset -c 2 <v0.2.1 release asset> generate <artifact> --seed 7 --nevents 20000 -j 1 [--systematics \"<request>\"], 5 interleaved reps; systematics.py under cProfile, 3 reps; twin artifacts regenerated with v0.2.1", data: "CERN host: /home/ncsmith/bench-v0.2.0/results-v0.2.1/phase4/ and TABLES_EXTRA.md"}
  - {commit: 3cc7f9c2, tag: v0.2.0, host: "Apple M3 Max, interactive desktop (load 3.0–7.0)", command: "<v0.2.0 aarch64-apple-darwin asset> generate … -j 1 (no affinity), 5 reps; systematics.py under cProfile, 3 reps; artifacts integrated at -j 12 (σ = 527.4 ± 1.2 pb)", data: "M3 host: /Users/ncsmith/bench-v020/results/phase4/"}
  - {commit: 3cc7f9c2, tag: v0.2.0, host: "Intel Xeon (Ice Lake), 4-vCPU KVM guest, AlmaLinux 10.2", command: "taskset -c 2 <release asset> generate <artifact> --seed 7 --nevents 20000 -j 1 [--systematics \"<request>\"], 5 interleaved reps; python -m cProfile systematics.py in.lhe out.lhe <request>, 3 reps", data: "CERN host: /home/ncsmith/bench-v0.2.0/results/phase4/"}
---

# Systematics weights against MadGraph's systematics module

vibegraph writes MadGraph's systematics-module weights inside `generate`, from a request
recorded at `integrate` (`use_syst = T`). MadGraph computes them afterwards, with
`systematics.py` reading and rewriting the event file. This page times both on the same
events. The weights themselves are gated per event against MadGraph's output elsewhere.

**Headline.** On 20 000 `pp_to_llj` events, CMS's request (926 weights over 880 PDF
members) adds 38.6 s to vibegraph's `generate` on the x86 host at v0.2.1, against 455 s for
MadGraph's module: **11.8×**. On the M3 Max at v0.2.0 it is 14.6 s against 256 s,
**17.5×**. Peak memory is 806 MiB against 3.1 GiB (801 MiB against 3.2 GiB on the M3). A
9-point scale request adds about 0.5 s against 8.8 s on x86 (18×) and 0.21 s against 4.5 s
on the M3 (about 22×, noisy at that size).

## Current: x86 at v0.2.1 and the M3 Max at v0.2.0

Same rows, requests and method as the v0.2.0 run described below; on x86 both sides ran the
same day at v0.2.1, with the twin artifacts regenerated (σ = 527.4 ± 1.2 pb; a 2000-event
check found the twins' events identical). The M3 ran v0.2.0, whose macOS build the musl
allocator fix does not touch.

vibegraph, added `generate` time over the `use_syst = F` twin (median of 5, single thread):

| request | weights/event | x86 v0.2.1 plain / v3 | M3 v0.2.0 | peak RSS, x86 / M3 |
|---|--:|--:|--:|--:|
| default (`use_syst = T`, no flag) | 147 | 4.76 / 4.70 s | 1.85 s | 132 / 125 MiB |
| `scale9` | 8 | 0.48 / 0.48 s | 0.21 s | 51 / 35 MiB |
| `cms` | 926 | 38.6 / 37.6 s | 14.6 s | 806 / 801 MiB |

MadGraph `systematics.py` on the same events (median of 3; setup is `Systematics.__init__`,
the loop `Systematics.run`):

| request | x86 v0.2.1: wall / setup / loop | M3: wall / setup / loop | peak RSS, x86 / M3 |
|---|--:|--:|--:|
| `scale9` | 8.76 / 0.03 / 8.53 s | 4.5 / 0.03 / 4.4 s | 77 / 34–37 MiB |
| `cms` | 454.7 / 12.6 / 442.1 s | 255.8 / 18.6 / 237 s | 3093 / 3221–3281 MiB |

Ratios against our added time: `cms` 11.8× on x86 (11.5× on the loop alone; 12.1× at v3)
and 17.5× on the M3 (16.2×); `scale9` 18× and about 22×. On x86, v0.2.1's added time is
20–30% below v0.2.0's on the same host, from the musl allocator fix. The weights were
checked on x86 at v0.2.0 (MadGraph on our events reproduces all 926 CMS weights exactly, on
20 events); the M3 run checked only the weight counts.

The v0.2.0 x86 measurement below is the earlier record: `cms` 9.3×, `scale9` 12×.

## Setup

One host, one sitting (`3cc7f9c2`, 2026-11-10; [benchmark hosts](benchmark-hosts.md)).

- **Row.** `pp_to_llj` with its banked run card (`dynamical_scale_choice = -1`) and `lhaid`
  changed from 247000 to 325300: MadGraph's module needs the run's own PDF first in the
  list, and CMS's list starts at 325300.
- **Artifacts.** Two, `use_syst = T` and `F`, otherwise identical
  (`--fixed-budget --neval 120000 --niter 6 --seed 20260719`). They produce identical events
  (checked on 2000: momenta, flavours, `XWGTUP`).
- **Requests.**
  - `scale9`: `--mur=1,2,0.5 --muf=1,2,0.5 --pdf=325300@0 --dyn=-1 --alps=1`, 8 weights.
    vibegraph reads `--pdf=None` as a set name and refuses it, so the central member is
    pinned instead.
  - `cms`: the request from CMS's `runcmsgrid_LO.sh`, `--start_id=1001 --pdf=<43 sets>
    --mur=1,2,0.5 --muf=1,2,0.5 --together=muf,mur,dyn --dyn=-1,1,2,3,4 --alps=0.5,1,2`,
    926 weights. PDF sets from `validation/pdf/fetch_cms.sh`.
- **vibegraph.** Both musl release assets, `-j 1` pinned to one core (so the assets'
  thread contention does not apply), 5 interleaved repetitions.
- **MadGraph.** `systematics.py` under cProfile on the `use_syst = T` output with
  `<initrwgt>`/`<rwgt>` stripped and `<mgrwt>` kept, 3 repetitions. Setup is
  `Systematics.__init__`, the loop is `Systematics.run`.

An artifact integrated with `use_syst = T` writes MadGraph's default request (147 weights
per event) even without `--systematics`; the "none" baseline is the `use_syst = F` twin.

## Results

vibegraph, `generate` wall time (median of 5):

| request | weights/event | plain | v3 | added over none, plain / v3 | per event | peak RSS |
|---|--:|--:|--:|--:|--:|--:|
| none (`use_syst = F`) | 0 | 15.97 s | 14.97 s | — | — | 28 MiB |
| default (`use_syst = T`, no flag) | 147 | 22.23 s | 21.23 s | 6.26 / 6.26 s | 313 µs | 112 MiB |
| `scale9` | 8 | 16.66 s | 15.72 s | 0.70 / 0.75 s | 35–38 µs | 35 MiB |
| `cms` | 926 | 64.39 s | 62.96 s | 48.4 / 48.0 s | 2.40–2.42 ms | 725 MiB |

MadGraph `systematics.py` on the same 20 000 events:

| request | wall, 3 reps | setup | loop | per event | peak RSS |
|---|--:|--:|--:|--:|--:|
| `scale9` | 8.71–8.75 s | 0.03 s | 8.47–8.53 s | 424 µs | 78 MiB |
| `cms` | 447.4–460.4 s | 12.2–12.9 s | 434–448 s | 22.0 ms | 3 092 MiB |

Ratios against vibegraph's added time (plain): `scale9` 12.4×; `cms` 9.3× in total and
9.0× on the loop alone.

## Reading it

- **The work is not identical.** vibegraph's added time includes writing the larger event
  file; MadGraph's loop includes reading and rewriting it.
- **Correctness was checked on 20 events only here**: MadGraph's module on vibegraph's events
  reproduces all 926 CMS weights exactly. The per-event gate against MadGraph's output is
  the systematics sprint's.
- **A trap.** MadGraph's module on an input that already carries weights appends a second
  weight group, and its ids can collide with the existing group's (the `scale9` ids 1–8
  with the default group's).
