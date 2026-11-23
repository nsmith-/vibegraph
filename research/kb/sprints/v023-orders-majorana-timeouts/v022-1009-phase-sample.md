---
type: Measurement
title: "v022-1009 phase-timed sample: 21 never-retried cards on the PR 3 instrument"
description: "vg-validator's timed run of 21 of the 178 never-retried cards with streamed phase records: the last phase each entered, the sizes, wall, CPU and peak RSS."
tags: [performance, mlm, multijet, vbs, timing, measurement]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Relayed by `vg-validator` to the PR 3 manager on 2026-11-23. The raw files
stay on the validator host (`survey-pr3/out/<id>.{ldjson.gz,log.gz,status,time.json,cmd}`,
the merged `survey-pr3/out.report.json`, generator `survey-pr3/pr3_report.py`).
This page holds the values as relayed; nothing here was re-measured.

**Setup.** Build `bin-pr3/vibegraph-4aab24fb` (branch `sprintl-timeout-triage`
at 4aab24fb, release). Each card staged as its v022-1009 seed-1 attempt:
`integrate --neval 120000 --seed 1 -j 4 --parallel-diagrams --target-rel 0.01`,
cap 1200 s (`timeout -s KILL`), `MemoryMax 4500M`. Phase times are the
records' `elapsed_s` (s from process start). Card 65 was re-run alone at the
end; the first run overlapped another job by about 40 s, so the re-run is the
row to use (the overlapped one is in `out/overlap-65/`).

**Record integrity.** In every killed card each phase that started has a
start record and each phase that finished has an end record; no record is
missing or out of order.

## Per card

| family | id | exit | wall / user / sys (s) | peak RSS | phase ends (s) | last phase, last progress |
|---|---|---|---|---|---|---|
| W/Z/DY+4j | 58 | killed (cap) | 1200 wall; CPU not captured (wrapper only) | not captured | — | `enumerate`: done 3 941 639 of 15 944 049, line 9/10 |
| W/Z/DY+4j | 65 (re-run) | killed (cap) | 1200 / 1225 user | 3.34 GB | enumerate 40 → probe 1117 | `compile` from 1118: done 30 of 2425 |
| W/Z/DY+4j | 15 | killed (cap) | 1200 | — | — | `enumerate`: done 1 791 543 of 2 898 918, 101 268 diagrams, line 4/4 |
| W/Z/DY+4j | 1009 | killed (cap) | 1200 | — | — | `enumerate`: done 3 792 370 of 15 944 049, 120 445 diagrams, 23 962 subprocesses, line 5/5 |
| VBS | 133 | killed (cap) | 1200 | — | compile 80 | `alpha_survey` from 81: done 4 of 6 |
| VBS | 14 | killed (cap) | 1200 | — | probe 5 → 172, compile 350, survey 352 → 1196 | `vegas` from 1196 |
| VBS | 744 | killed (cap) | 1200 | — | vegas from 94 | `vegas`: done 4 of 6, 1.76e6 ns/eval |
| VBS | 710 | killed (cap) | 1200 | — | compile 13, survey 13 → 663 | `vegas` to the cap |
| VBF resonances | 290 | exit 1 at 14 s | 14 | — | all phases to `compile` recorded | refusal (process) in `compile` (cuts): "final-state leg 2 (PDG 2) leaves incoming leg 0 (PDG 2) through a massless spacelike propagator … collinear limit with the beam uncut" |
| VBF resonances | 636 | killed (cap) | 1200 | — | vegas from 385 | `vegas`: done 11 of 14, 2.5e5 ns/eval |
| VBF resonances | 655 | killed (cap) | 1200 | — | compile 239, survey 239 → 1152 | `vegas` from 1152 |
| GJ-4Jets | 32 | killed (cap) | 1200 | — | vegas from 344 | `vegas`: done 1, 5.8e5 ns/eval |
| GJ-4Jets | 25 | killed (cap) | 1200 | — | vegas from 477 | `vegas`: done 1, 9.8e5 ns/eval |
| TT-3Jets | 42 | killed (cap) | 1200 | — | vegas from 191 | `vegas`: done 1, 1.4e5 ns/eval |
| TT-3Jets | 46 | killed (cap) | 1200 | — | vegas from 414 | `vegas`: done 1, 4.6e5 ns/eval |
| aQGC | 196 | killed (cap) | 1200 | — | vegas from 213 | `vegas`: done 5 of 6, 1.04e6 ns/eval |
| aQGC | 195 | killed (cap) | 1200 | — | vegas from 589 | `vegas`: done 1, 2.19e6 ns/eval |
| other: QCD multijet MLM | 37 | exit −9 at 208 s (memory cap, not time) | 208 / 325 user | **4.60 GB** | enumerate 109 (152 984 diagrams, 18 156 subprocesses) | `amplitude_probe`; reached compile of `g g > g g g g g` in 54 s |
| other: older GJets | 992 | killed (cap) | 1200 | — | vegas from 347 | `vegas`: done 1, 5.8e5 ns/eval |
| other: TT FCNC | 590 | killed (cap) | 1200 | — | vegas from 379 | `vegas`: done 2, 8.8e5 ns/eval |
| other: VBS-like madspin/QCD | 722 | **exit 0** | 1079 / 4146 / 2 | 0.23 GB | enumerate 1, probe 3, pdf 4, compile 7, channels 7, survey 139, vegas 1076, configurations 1076, bank 1079 | σ = 0.074121 ± 0.00019 pb |

Card 37's phases before the kill, under the memory cap only; 58's CPU figure
(user 0.001 s, RSS 7 MB) is the wrapper's, so only its wall time and progress
count. "—" means the value was not relayed.
