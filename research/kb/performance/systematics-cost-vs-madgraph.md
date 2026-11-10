---
type: Measurement
title: "Systematics weights against MadGraph's systematics module"
description: "At v0.2.0 on 20 000 pp_to_llj events, vibegraph's in-process systematics weights add 0.70 s for a 9-point scale request and 48 s for CMS's 926-weight request; MadGraph's module takes 8.7 s and 452 s (12× and 9.3×)."
status: draft
tags: [performance, madgraph-comparison, systematics, pdf, benchmarks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
measured:
  - {commit: 3cc7f9c2, tag: v0.2.0, host: "Intel Xeon (Ice Lake), 4-vCPU KVM guest, AlmaLinux 10.2", command: "taskset -c 2 <release asset> generate <artifact> --seed 7 --nevents 20000 -j 1 [--systematics \"<request>\"], 5 interleaved reps; python -m cProfile systematics.py in.lhe out.lhe <request>, 3 reps", data: "CERN host: /home/ncsmith/bench-v0.2.0/results/phase4/"}
---

# Systematics weights against MadGraph's systematics module

vibegraph writes MadGraph's systematics-module weights inside `generate`, from a request
recorded at `integrate` (`use_syst = T`). MadGraph computes them afterwards, with
`systematics.py` reading and rewriting the event file. This page times both on the same
events. The weights themselves are gated per event against MadGraph's output elsewhere.

**Headline (v0.2.0).** On 20 000 `pp_to_llj` events, CMS's request (926 weights over 880
PDF members) adds 48 s to vibegraph's `generate` against 452 s for MadGraph's module,
**9.3×**, at 725 MiB against 3.0 GiB peak memory. A 9-point scale request adds 0.70 s
against 8.7 s, **12×**.

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
