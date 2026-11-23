---
type: Measurement
title: "v022-1009 phase-timed sample: 21 never-retried cards on the PR 3 instrument"
description: "vg-validator's timed run of 21 of the 178 never-retried cards with streamed phase records: the last phase each entered, the sizes, wall, CPU and peak RSS; then channel, compile and VEGAS-iteration sizes and enumeration line starts."
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

## Second relay: sizes and iteration lines

Relayed by `vg-validator` on 2026-11-23 for L-T2, extracted from the same
`survey-pr3/out` files (nothing re-run). Host files:
`survey-pr3/pr3_extract_LT2.txt` and `survey-pr3/pr3_extract_alpha.short.txt`
(generator `survey-pr3/pr3_extract.py`). The `alpha_survey` phase logs one
"surveying N channels over 6 × 40000 points" line per part and no
per-iteration rate, so only its wall time is known.

Compile sizes: subprocesses / compiled_diagrams / helicities / colour_flows /
parts / flavour_groups. "Survey parts" lists each part's channel count.

| id | channels | compile | survey parts | survey wall (s) | VEGAS warm-up 1 (points at ns/eval) |
|---|---|---|---|---|---|
| 32 | 1068 | 740/24544/5488/1608/4/237 | 3, 15, 105, 945 | 184.7 | 609 993 at 576 493 |
| 25 | 1068 | as 32 | as 32 | 315.9 | 589 807 at 983 415 |
| 42 | 1068 | 264/4327/1420/742/4/40 | 3, 15, 105, 945 | 131.7 | 631 541 at 141 740 |
| 46 | 1068 | as 42 | as 42 | 355.4 | 607 379 at 457 664 |
| 195 | 170 | 255/14262/2970/207/1/165 | 170 | 548.6 | 189 904 at 2 192 124 |
| 196 | 108 | 72/12240/648/144/1/72 | 108 | 197.3 | 157 068 at 740 498 |
| 992 | 1060 | 740/20872/4560/1316/4/193 | 3, 15, 105, 937 | 189.3 | 613 187 at 579 734 |
| 590 | 372 | 214/5789/1700/630/3/128 | 4, 31, 337 | 372.6 | 265 917 at 791 039 |
| 636 | 236 | 7452/3702/14958/3702/15/1544 | 15 parts: 38, 54, 19, 19, 19, 18, 11, 11, 11, 4, 8, 8, 4, 8, 4 | 338.3 | 217 809 at 162 356 |
| 744 | 101 | 360/4144/4968/424/1/212 | 101 | 86.3 | 133 003 at 347 015 |
| 14 | 550 | 864/124200/1098/1080/3/540 | 220, 110, 220 | 843.6 | none (VEGAS started at 1196 s) |
| 133 | 298 | 528/36744/8136/992/1/496 | 298 | killed at survey 4/6 | — |
| 710 | 298 | 528/36744/904/992/1/496 | 298 | 649.3 | none |
| 655 | 178 | 9856/5656/19276/5336/6/2364 | 108, 30, 12, 4, 12, 12 | 912.1 | none |

Later VEGAS iterations, points at ns/eval. The iteration log prints the
running integral in GeV⁻², the integrand's natural unit (the relay labelled
it pb); the pb values below are converted with `GEV2_TO_PB`.

| id | warm-up 2 | kept iterations | last running estimate |
|---|---|---|---|
| 590 | 520 949 at 881 303 | — | — |
| 196 | 217 221 at 849 606 | 3: 197 614 at 1 001 602; 4: 187 597 at 1 045 886; 5: 182 754 at 1 038 523 | iter 5: 5.287757e-9 ± 2.05e-11 GeV⁻² (2.059 pb), χ²/dof 5.761 |
| 744 | 238 377 at 1 439 328 | 3: 171 882 at 1 719 612; 4: 160 852 at 1 757 902 | iter 4: 3.756866e-8 ± 1.62e-10 GeV⁻² (14.63 pb), χ²/dof 1.135 |
| 636 | 515 487 at 198 811 | 3–11: 324 257 → 270 958 points at 237 173–313 558 | iter 11: 5.652203e-24 ± 6.01e-26 GeV⁻² (2.20e-15 pb), χ²/dof 26.5 (iter 4) falling to 8.0 (iter 11); iter 5 jumps about +2σ above iters 3–4 |

636's running estimates, iterations 3 to 11 (GeV⁻²): 5.380022e-24 ± 2.20e-25,
5.325475e-24 ± 1.20e-25, 5.644755e-24 ± 1.57e-25, 5.635729e-24 ± 1.20e-25,
5.611063e-24 ± 9.70e-26, 5.644902e-24 ± 8.63e-26, 5.650498e-24 ± 7.54e-26,
5.644043e-24 ± 6.68e-26, 5.652203e-24 ± 6.01e-26. 196's, iterations 3 to 5:
5.356492e-9 ± 3.71e-11, 5.304870e-9 ± 2.52e-11, 5.287757e-9 ± 2.05e-11.
744's, iterations 3 and 4: 3.767163e-8 ± 2.36e-10, 3.756866e-8 ± 1.62e-10.

**Enumeration lines.** Start time of each process line, from the progress
records' timestamps relative to the card's first record, with the line's
candidate total:

| id | lines |
|---|---|
| 58 | 1 at 0.0 s (1089), 2 at 0.0 (1089), 3 at 0.0 (11 979), 4 at 0.2 (11 979), 5 at 0.3 (131 769), 6 at 2.3 (131 769), 7 at 4.3 (1 449 459), 8 at 45.4 (1 449 459), 9 at 89.4 (15 944 049); line 10 never started; line 9 at 3 941 639 at the cap |
| 15 | 1 at 0.0 (2178), 2 at 0.0 (23 958), 3 at 0.8 (263 538), 4 at 23.1 (2 898 918); line 4 at 1 791 543 at the cap |
| 1009 | 1 at 0.0 (1089), 2 at 0.0 (11 979), 3 at 0.2 (131 769), 4 at 2.6 (1 449 459), 5 at 54.9 (15 944 049); line 5 at 3 792 370 at the cap |

The validator's linear extrapolation of the last line to completion, about
4500 s, is an extrapolation, not a measurement.
