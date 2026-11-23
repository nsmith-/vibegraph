---
type: Session Report
title: "L-T2 report: per-family timeout verdicts from the v022-1009 phase sample"
description: "Enumeration walks ~97% flavour-violating subprocesses on W/Z+4j; the probe is a serial duplicate of the compile; target runs pay six channel-floor-sized VEGAS iterations at 0.6–9 ms CPU per point; the α survey spends 240k points per part; card 37's memory is one helicity-expanded g g > 5g arena. The triage item closes."
status: draft
tags: [performance, mlm, multijet, vbs, timing, triage, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-timeout-triage`, from 24513cc9. Analysis and kb only: no
code changed, and the instrument (`vibegraph-cli/src/phase.rs` and its
wiring) is untouched. The values read are vg-validator's, relayed in
[v022-1009-phase-sample.md](../v022-1009-phase-sample.md): the first relay
(per card) and the second (sizes, iteration lines, enumeration line starts),
which this session added to that page as relayed. Local measurements are
dev profile (opt-level 2, no LTO), `-j 2`, and are used for ratios and
counts only, never compared in absolute time with the validator's release
binary. The `lt2-scripts/`, `lt2-mem/` and `lt1-smoke/` paths below are in the
session scratchpad and are not banked; every number taken from them is quoted here.

**Closes** `multijet-mlm-cards-time-out-in-the-gridpack-campaign`: its file is
deleted in this commit, and its account is below.

## (a) Coverage

Checked against [v022-1009-never-retried.tsv](../v022-1009-never-retried.tsv)
with `awk -F'\t' '$1!~/^#/{print $1}' … | sort | uniq -c` and a lookup of
each sampled id:

| family | cards in the list | sampled | ids |
|---|---|---|---|
| W/Z/DY+4j | 56 | 4 | 58, 65, 15, 1009 |
| VBS | 47 | 4 | 133, 14, 744, 710 |
| VBF resonances | 22 | 3 | 290, 636, 655 |
| GJ-4Jets | 19 | 2 | 32, 25 |
| TT-3Jets | 10 | 2 | 42, 46 |
| aQGC | 7 | 2 | 196, 195 |
| other (17) | QCD multijet MLM 4, older GJets 6, TT FCNC 4, VBS-like madspin/QCD 3 | 1 per group | 37, 992, 590, 722 |

Every sampled id is in the list under the family the sample page gives it.
Every family has at least 2 cards and each group of 'other' has one, so (a)
holds.

## (b) Verdict per family

"Last phase" follows the item's rule: a card killed at the cap counts by its
last started phase. "Time" names the phase that took most of the 1200 s
where it differs. Where a card's cost is bounded rather than measured, the
bound and its assumptions are in the sections below.

| family | last phase (rule) | time | cause within the phase | how uniform; not covered |
|---|---|---|---|---|
| W/Z/DY+4j | `enumerate` ×3 (58, 15, 1009), `compile` ×1 (65) | `enumerate` ×3; 65: `amplitude_probe` 1077 of 1200 s | the alias walk hands ~97% flavour-violating subprocesses to feyngraph, one at a time; the probe is a serial duplicate of the compile | uniform: every 4j-line card died in its last line. Not sampled: DYJets HT, DYto2L-4Jets, WBtoLNuB-4Jets, Zto2Nu-4Jets, WtoLNu (inclusive) |
| VBS | `vegas` ×3 (14, 744, 710), `alpha_survey` ×1 (133) | `alpha_survey` in 133, 14 (844 s), 710 (650 s); `vegas` in 744 | per-point cost (2.7–7 ms wall at 4 threads on 1-part cards) times the survey's 240k points per part and VEGAS's six floor-sized iterations | the four agree on "sampling"; the split between survey and VEGAS follows the per-point cost. Not sampled: ZZTo4Q, OSWW, ZNuNuZJJjj, polarised ssWW semilep |
| VBF resonances | `vegas` ×2 (636, 655), refusal ×1 (290) | 655: `alpha_survey` 913 s; 636: `vegas` | 655: survey over 6 parts (1.44M points); 636: iterations inconsistent (χ²/dof 26 → 8), far from the target after 11 iterations | two different causes in two cards. Not sampled: Radion→ZZ (7 cards), Z′→WW |
| GJ-4Jets | `vegas` ×2 | `vegas` | six iterations of ≥ 1068 × 512 points at 2.3–3.9 ms CPU per point | uniform (both cards one warm-up done). Both shapes (GJ-, GJets-) covered |
| TT-3Jets | `vegas` ×2 | `vegas` | as GJ: 1068-channel floor, 0.57–1.8 ms CPU per point | uniform. Not sampled: dileptonic and semileptonic decayed variants |
| aQGC | `vegas` ×2 | `vegas` | 195: 8.8 ms CPU per point on 170 channels; 196: borderline, needs ≈ 1290 s | not uniform in margin: 196 is cost-borderline, 195 is 2.6× the cap. 5 of 7 not sampled |
| other | 37 `amplitude_probe` (memory); 992, 590 `vegas`; 722 finished | — | 37: the helicity-expanded arena of `g g > g g g g g`; 992 and 590 as GJ/TT | one card per group; 722 is borderline cost |

So the timeouts are four causes, not one per card:
1. **Enumeration** on the 4-jet lines (W/Z/DY+4j).
2. **The duplicate, serial compile** in the amplitude probe and the compile
   phase (W/Z/DY+4j, VBS 14, and every card with many sets).
3. **Sampling cost**: per-point cost times a point count the channel floor
   and `--min-iters 6` fix, not `--neval` (GJ, TT, aQGC, VBS, VBF, other).
   The α survey is the first place it is paid.
4. **Memory** of one helicity-expanded evaluator (card 37).

Card 636 adds a fifth, convergence rather than cost. Card 290 is a correct
refusal.

### Enumerate (58, 15, 1009)

**What "done N of M" counts.** `generate_sets_inner`
(`vibegraph-lib/src/diagrams/mod.rs:1184`) walks the Cartesian product of the
line's legs' alias expansions, initial state outermost: `M = Πᵢ |aliasᵢ|`.
With `p = j` = 11 partons (sm-no_b_mass: g, u c d s b and their antiparticles)
and 3 × 3 lepton/neutrino choices, a `V + n j` line has `M = 11ⁿ⁺² × 9`:
1 449 459 at 3 j and 15 944 049 at 4 j, which are the totals relayed for 58
and 1009 (15's last line, 2 898 918 = 11⁵ × 9 × 2, is a b-tagged shape).
`done` counts every candidate, including duplicates (same unordered content)
and charge violators, which cost almost nothing. A candidate that survives
both is handed to feyngraph (`DiagramGenerator::assign_topologies` over every
cached tree topology on the line's leg count), **whether or not the model
gives it a diagram**, and is pushed as a set.

**The walk is exponential in the line's legs twice over.** The candidate
count grows ×11 per jet, and each survivor's cost grows with the tree-topology
count: 220 topologies on 6 legs, 2485 on 7 (from the smoke logs'
`tree topologies on N legs` lines).

**Where the time goes, measured on L-T1's W(→ℓν)+0–4j smoke log** (the
existing `lt1-smoke/w4j.{ldjson,log}`, dev, `-j 2`; no new run):

```
$ python3 -I lt2-scripts/enum_lines.py lt1-smoke/w4j.ldjson
line 5: t 0.4->3.5 s (3.1 s), done 131769/131769, subprocesses 3294, diagrams +600 (cum 684), 0.94 ms/subprocess
line 7: t 6.8->161.2 s (154.4 s), done 1449459/1449459, subprocesses 13734, diagrams +5376 (cum 6660), 11.24 ms/subprocess
line 8: t 161.7->300.0 s (138.2 s), done 905132/1449459, subprocesses 11791, diagrams +5256 (cum 11916), 11.72 ms/subprocess
```

On line 7 (ℓ⁺ν + 3j), 13 734 subprocesses reached feyngraph and 289 have
diagrams (`INFO … N diagrams for …` lines). The debug `enumerated in` times
of those 289 sum to 3.995 s, so **150 of the line's 154 s went to the
13 445 subprocesses with no diagram**, about 11.2 ms each against 13.8 ms for
a non-empty one.

**Why they are empty, checked exactly.** With a diagonal CKM every vertex of
sm-no_b_mass conserves six additive numbers: three lepton flavours and three
quark-generation numbers. A script recomputes the walk (dedup on unordered
content, charge filter, as the code does) and applies those six numbers:

```
$ python3 -I lt2-scripts/flavour_prune.py lt1-smoke/w4j.log
W+ +2j: survivors 3294, flavour-conserving 180 (5.5%)
W+ +3j: survivors 13734, flavour-conserving 288 (2.1%)
W- +3j: survivors 13734, flavour-conserving 288 (2.1%)
non-empty logged by shape (lepton sign, n_final): {('+', 2): 6, ('+', 3): 19, ('+', 4): 182, ('+', 5): 288, ('-', 2): 6, ('-', 3): 19, ('-', 4): 182, ('-', 5): 278}
non-empty violating the numbers: 0 []
$ python3 -I lt2-scripts/fp_lib.py   # lt2--flavour4.log
W+ +4j: candidates 15944049, survivors 45720, flavour-conserving 1284 (2.81%)
Znunu +3j: candidates 1449459, survivors 19224, flavour-conserving 498 (2.59%)
Znunu +4j: candidates 15944049, survivors 60318, flavour-conserving 1878 (3.11%)
```

The survivor count (13 734) matches the run's own `subprocesses` exactly, so
the script reproduces the walk. On the completed 3j line the 288
flavour-conserving subprocesses are exactly the 288 non-empty ones, and no
non-empty subprocess of the whole run violates the numbers. (The 2j shape
logs 182 non-empty against 180 conserving. Each of those lines is followed by
a one-candidate trial walk that logs again, so the two extra lines are
repeats, and none violates.)

**Release-binary scale.** At 1145 s into 1009's 4j line the run had 23 962
subprocesses, about 48 ms each with feyngraph on 4 threads. The script gives
60 318 survivors for that line, so finishing it costs about 2900 s if the
per-survivor rate holds, or about 4500 s by the validator's linear
extrapolation in candidates. Both are extrapolations. W+4j (58) has two such
lines. With the prune, 1878 survivors at the same rate is about 90 s, and the
rate per survivor is about the same empty or not.

**What parallelises.** The outer loop over candidates is serial.
`--parallel-diagrams` (`EnumerationPool::Ambient`) only lets feyngraph's
internal `par_iter`s use the pool *inside* one subprocess's topology
assignment. The subprocesses are independent and would parallelise across
the pool, but the prune comes first: it removes ~97% of the work exactly,
where parallelism divides it by at most the core count. MadGraph never visits
these subprocesses, because its leg-combining recursion only builds what the
vertex table allows
([diagram-enumeration-cost-unmeasured](../../../backlog/feature/diagram-enumeration-cost-unmeasured.md)).
This session answers that item's first question: enumeration is on the
critical path.

### The amplitude probe (65, 14, 37)

`amplitude_probe::measure` (`vibegraph-lib/src/amplitude_probe.rs:255`) maps
`compile_class` + 4 evaluations over every set with `sets.iter()`, serially.
`derive_flavor_groups` (`proton.rs:1107-1133`) compiles every set again with
the same `compile_class`, also in a serial loop, and keeps all of them.
Evidence that the cost is the duplicate:
- **14**: probe 5 → 172 s (167 s), compile 172 → 350 s (178 s).
- **L-T1 ssWW**: 12.54 s against 12.10 s.
- **65**: probe 1077 s at 1225 user-s for 1200 wall-s, so the probe ran on
  one of four cores.

Reusing the compile's evaluators removes the probe's compile and keeps only
4 evaluations per set. On 65 that is most of the 1077 s, but the compile
phase would then take about as long itself. 65 was 30 sets into a
2425-set part at the cap. The progress total is per part, so its per-set
rate cannot be read off. Both loops are serial, so parallelising the compile
over sets is the second half of the fix. On **memory**, the probe holds one
evaluator at a time and the compile phase holds all of them. Removing the
probe therefore does not lower the peak: see card 37.

This is the existing
[amplitude-probe-compiles-each-set-twice](../../../backlog/performance/amplitude-probe-compiles-each-set-twice.md)
(priority low, "seconds per run"). The evidence here makes it a
timeout cause: see Found.

### VEGAS at one iteration (GJ, TT, aQGC 195, 992, 590)

**Point count.** Under `--target-rel` a run stops only at
`iteration ≥ min_iters.max(warmup + 2)` (`budget.rs:1210`), and
`--min-iters` defaults to **6** (`integrate.rs:500`): two warm-ups and four
kept iterations at least, whatever the error. The first iteration's points
are `Σⱼ max(shareⱼ, 512)`, so the channel floor `n_ch × MIN_CHANNEL_NEVAL`
sets them, not `--neval 120000`. Later iterations are re-floored on accepted
points up to `MAX_FLOOR_ACCEPTANCE_SCALE` = 4×. Measured: warm-up 1 is
1.08–1.16× the floor on the 1068-channel cards (609 993 points against
546 816) and 2.18× on 195.

**ns/eval rises after warm-up 1**: 744 from 0.35 to 1.44–1.76 ms, 196 from
0.74 to 1.04 ms, 636 from 0.16 to 0.29 ms, L-T1 ssWW from 82 to 212 µs. A
point the cuts reject is cheap, and the trained grid sends more points
through the cuts, so warm-up 1's rate understates the cost of every later
iteration.

**Arithmetic.** T1 is warm-up 1's points times its ns/eval. R is the time
left after it, during which warm-up 2 did not finish. Lower bound on the
wall the run needed: `max(t_vegas + 6·T1, 1200 + 4·0.6·R)`. The second form
takes the kept iterations at 0.6× warm-up 2's points, the smallest ratio
observed on 196, 744 and 636 (iterations 3 and later against warm-up 2's
points: 0.83–0.91, 0.67–0.72, 0.53–0.63).

| id | channels / groups | t_vegas (s) | T1 (s) | CPU per point | needed ≥ | × cap |
|---|---|---|---|---|---|---|
| 32 | 1068 / 237 | 344 | 351.7 | 2.31 ms | 2454 s | 2.0 |
| 25 | 1068 / 237 | 477 | 580.0 | 3.93 ms | 3957 s | 3.3 |
| 42 | 1068 / 40 | 191 | 89.5 | 0.57 ms | 3407 s | 2.8 |
| 46 | 1068 / 40 | 414 | 278.0 | 1.83 ms | 2419 s | 2.0 |
| 195 | 170 / 165 | 589 | 416.3 | 8.77 ms | 3087 s | 2.6 |
| 992 | 1060 / 193 | 347 | 355.5 | 2.32 ms | 2480 s | 2.1 |
| 590 | 372 / 128 | 379 | 210.4 + 459.1 (two warm-ups) | 3.2–3.5 ms | 2150 s | 1.8 |
| 196 | 108 / 72 | 213 | five iterations end at 1097.8 s | 3.0–4.2 ms | ≈ 1290 s (iteration 6 ≈ 190 s) | 1.07 |
| 744 | 101 / 212 | 94 | four end at 1061.6 s | 1.4–7.0 ms | ≈ 1630 s | 1.36 |

CPU per point is ns/eval × 4 threads. It assumes the 4 threads were busy, and
722's 4146 user-s in 1079 wall-s says they are in VEGAS. The 196 and 744
rows were already below the 1% target by their quoted error: 0.39% and 0.43%.
`--min-iters 6` alone kept them running. 196's χ²/dof of 5.8 would widen its
stop error to about 0.9%.

**Verdict.** The cost is the per-point cost times a point budget that
`--neval` and `--target-rel` do not set: the channel floor (546 816 points on
a 1068-channel MLM card) and six iterations, two of them warm-ups. The
per-point cost is 40–237 flavour groups evaluated at every point; on 237
groups that is 9.7 µs CPU per group, and per group it ranges 10–53 µs. That is
[mlm-every-group-matrix-element-evaluated-per-point](../../../backlog/performance/mlm-every-group-matrix-element-evaluated-per-point.md)
and
[mlm-matrix-element-evaluated-twice-per-group](../../../backlog/performance/mlm-matrix-element-evaluated-twice-per-group.md);
195 is not MLM-matched and pays the same pooling over its 165 groups. The
warm-up-schedule hypothesis from L-T1's Found is the same effect: warm-up 2 is
the first iteration at the corrected floor and the higher accepted fraction.

### alpha_survey (133, 14, 710, 655)

`MultiplicitySum::adapt_alphas` runs `ProtonIntegrand::adapt_alphas` **per
part**, each `ADAPT_ITERS = 6` iterations of `survey_points(neval)` =
`clamp(120000, 10k, 40k)` = 40 000 points. A card therefore surveys
`parts × 240 000` points, whatever its channel count or how σ divides across
the parts. Survey wall per point:
- **Single-part cards**: it equals VEGAS warm-up 1's: 195 2.29 against 2.19
  ms, 196 0.82 against 0.74, 744 0.36 against 0.35. The survey point costs
  what an integrand point costs. Nothing in the survey is special.
- **133**: four iterations in the 1119 s it had, so 5.6–7.0 ms wall per
  point.
- **710**: 649 s, 2.7 ms.
- **14**: 3 parts, 720k points, 844 s, 1.17 ms.
- **655**: 6 parts, 1.44M points, 912 s, 0.63 ms.
- **636**: 15 parts, 3.6M points, 338 s.

So "why so many points" is the per-part survey, multiplied on the
mass-split cards: 636 has 15 parts and 655 has 6. It overlaps
[mass-split-parts-thin-the-per-group-budget](../../../backlog/performance/mass-split-parts-thin-the-per-group-budget.md),
which is about grids, not the survey.

**Per-point cost on the VBS cards.** 133 and 710 have the same structure: 528
subprocesses, 496 flavour groups, 298 channels. Only the helicities differ:
8136 against 904 (710 is polarised LL). The cost per point falls only about
2.3×, from about 6.3 to 2.7 ms. On a linear model `a + b·helicities`, about
2.2 ms wall (9 ms CPU) per point does not scale with helicities. It is a
fixed cost: the 496 groups' kinematics, cuts and PDF weights, or the
298-channel density sum. These records cannot tell which; a profile can (see
Found). 496 groups from 528 subprocesses means the |M|² grouping barely
merges anything on these cards.

14 would need at least about 3200 s if a VEGAS point costs what its survey
point does. That is bounded, not measured: no VEGAS iteration finished.

### Card 290: the refusal is correct, and the v0.2.2 timeout is not explained by code

The refusal (`CutError::UncutInitialCollinear`, `cuts.rs:845`) fires only
when the compiled cuts give the leg neither `pt_min > 0`, nor
`eta_max ≥ 0`, nor a pT-rank floor (`cuts.rs:558-560`). So card 290's run
card leaves the outgoing u quark's beam-collinear limit open. The limit is a
massless spacelike exchange between incoming and outgoing u (in VBF W
production, a t-channel photon): `∫ dt/t` reaches `t = 0`.

MadGraph applies no default regulator for it. `setcuts.f:260-300` sets each
leg's `etmin` from the run card (`ptj`, `ptl`, …) and nothing else, and
`auto_ptj_mjj` (`banner.py:4304`, `setcuts.f:157`) applies only with MLM
(`xqcut > 0`). MadEvent would integrate the logarithm to wherever its
t-channel mapping reaches. The refusal is therefore right, and the card
leaves the timeout list as a named refusal: sprint L's
"uncut massless limits" class.

**But nothing in the code changed between v0.2.2 and 4aab24fb:**

```
$ git log --format='%h %s' v0.2.2..4aab24fb -- vibegraph-lib vibegraph-cli Cargo.toml Cargo.lock
f809054d feat(cli): stream per-phase start and end records
$ git diff --stat v0.2.2 4aab24fb -- vibegraph-lib vibegraph-cli Cargo.lock Cargo.toml rust-toolchain.toml
 vibegraph-cli/src/generate.rs            |   9 +
 vibegraph-cli/src/integrate.rs           |  98 +++++++++-
 vibegraph-cli/src/main.rs                |   2 +
 vibegraph-cli/src/output.rs              |  12 +-
 vibegraph-cli/src/phase.rs               | 213 +++++++++++++++++++++
 vibegraph-cli/tests/cli_output_ldjson.rs |   2 +-
 vibegraph-cli/tests/cli_phase_timing.rs  | 307 +++++++++++++++++++++++++++++++
```

`vibegraph-lib` is identical, and the CLI diff only adds phase records.
v0.2.2 given the same input would therefore have refused in 14 s too. The
v022-1009 "timeout on 5 seeds" for 290 must come from a different input or
harness record: the staged card, the command, or the outcome
classification. Open: see the validator request.

### Card 722: borderline cost, no speed-up

The same `git log` shows no code change that could speed it up. 722 finished
in 1079 s, 0.90 of the cap, on 4146 user-s: 3.84 cores busy, 1069 s of it in
`alpha_survey` + `vegas`. On a shared batch host, a 10% margin is within
contention, so its v0.2.2 timeouts are explained by the cap, like the
retries in (c). That is inferred: v022-1009's host load per slot is not in
the records.

### Card 37: where the memory goes

Local, approved: `integrate --check` on `g g > g g g g g` (sm-no_b_mass, the
MLM run card from L-T1), dev, `-j 2`, `timeout -s KILL 300`, RSS sampled
every second by the driver (`lt2-scripts/mem_run.sh`, `lt2-mem/rss.txt`):

```
enumerate: 0.168 s (diagrams 2485, subprocesses 1)
compile{process=g g > g g g g g}: compiled program arena_nodes=138323 arena_slots=42595 amplitudes=7245 configurations=945 helicities=128
compile{process=g g > g g g g g}: compiled in 66.682 s
amplitude_probe end: wall_s 141.2
compile end: wall_s 142.5 (colour_flows 720, compiled_diagrams 2485, helicities 112)
channels end: 945; result check ok
status: gg5g rc=0 wall=284.121124294 peak_rss_kB=5003788
RSS: 70 MB through the 66.7 s compile, then +~150 MB/s from its end, 4.46 GB at +46 s,
     peak 4.93 GB at 136 s in the probe, 0.92 GB after the probe ended, 5.00 GB at 275 s in the compile phase
```

The compile phase took 142.5 s against the probe's 141.2 s: the duplicate
again, on one set.

The compiled program is small: 70 MB at the end of the 66.7 s compile. The
growth starts when the compiled evaluator is first bound. `compile_class` calls
`prune_zero_helicities` (`helas/eval/compile.rs:559`), whose first
`BoundAmplitude::bind` builds `folded_hel`, `self.folded.expand_helicities(&self.helicities)`
(`compile.rs:235-238`), over all **128** helicities, before any is pruned.
That is 138 323 arena nodes × 128 helicities, about 17.7M node-helicity
entries, at about 250 B each. Peak 4.93 GB in dev against the campaign's
4.60 GB release kill at `MemoryMax 4500M`: the one `g g > 5g` set alone
exceeds the limit. Tree-level pruning would keep about 112 of the 128
helicities (the all-equal and one-flipped helicities vanish), so pruning
before expanding would barely help. The compile phase's record confirms 112
were kept. The memory is the helicity expansion
itself. The compile phase would build the same arena again, and keep it with
every other set's, so removing the duplicate probe compile does not fix 37.

The compile phase peaked at 5.00 GB, re-expanding this set's arena
while it held the set. On a card with 18 156 subprocesses (37), it holds
every set's evaluator.

### Card 636: convergence, not cost

11 iterations in 802 s. The quoted error reached 1.06% at iteration 11, but
the iteration χ²/dof went 26.5 → 16.1 → … → 8.0, and the estimate jumped
+2σ between iterations 4 and 5. With the stop reading `max(quoted, emp)` per
channel, the run was about √8 × 1.06% ≈ 3% from stopping. It would need on
the order of (3/1)² ≈ 9× its kept iterations: about 70 more at 67 s, so
about 5600 s (bounded, inferred from the stop rule). Its σ, 5.65e-24 GeV⁻²
= 2.2e-15 pb, is implausibly small for a 1.8 TeV VBF Radion → WW. That is
not a timeout cause, but it is a possible silent wrong answer (see Found).

## (c) The cap-limited retries

Of 155 seed runs in v022-1009 that hit the espresso cap on cards with at
least one ok seed, 117 were ok at microcentury and 36 of the remaining 38 at
workday. Those are explained by the cap: they are cost, not hangs, and need
no item of their own. They are the same per-point-cost × floor-iteration
picture as the never-retried 178, at smaller multiples of the cap. 722 (0.90
of the cap, on one seed) and 196 (≈ 1.07×) sit at that boundary inside the
sample. The 2 of 155 not ok at workday are not identified in what was
relayed.

## (d) The causes as items

Drafted in Found:
1. enumeration of flavour-violating subprocesses (new, high);
2. the serial compile loops (new, high);
3. the floor-and-min-iters point budget of target runs (new, high);
4. the per-part α survey (new, medium);
5. the unprofiled per-point fixed cost on wide-group VBS cards (new, medium);
6. the helicity-expansion memory (new, high);
7. card 636's σ and convergence (new, validation, medium).

Existing items cited instead of duplicated:
- `amplitude-probe-compiles-each-set-twice`: the probe duplicate. Raise to
  high with the evidence above.
- `mlm-every-group-matrix-element-evaluated-per-point` and
  `mlm-matrix-element-evaluated-twice-per-group`: per-point cost on MLM
  cards. The first also describes the non-MLM pooling of 195 and the VBS
  cards.
- `diagram-enumeration-cost-unmeasured`: answered in part.
- `mass-split-parts-thin-the-per-group-budget`: related to the per-part
  survey.

Checked and not the cause here:
- `mlm-vanishing-flavour-groups-still-sampled`: no relayed size shows
  vanishing groups.
- `zero-weight-integrand-runs-to-point-cap`: every VEGAS card has passing
  points.
- `generate-parallel-diagrams-is-a-no-op`: `generate` is not in the campaign
  command.

## Oracles, and what each cannot detect

- **Coverage check**: id-by-family lookup in the tsv. It cannot detect a
  family label that is wrong in the tsv itself: families come from
  gridpack-name prefixes.
- **The walk reproduction**: matches the run's subprocess count exactly
  (13 734) and its non-empty count exactly (288) on the completed 3j line.
  It cannot see a model whose vertices conserve fewer numbers (a CKM model):
  the six numbers are this model's, hand-listed. The drafted item requires
  them to be derived from the vertex table. The 4j counts are the script's,
  not a run's.
- **Release-scale enumerate cost**: 48 ms per survivor is the mean over the
  first 23 962 survivors of 1009's 4j line. Survivors later in the walk may
  cost differently, so the 2900 s and 90 s figures are extrapolations.
- **VEGAS lower bounds**: assume the 4 threads busy and kept iterations
  ≥ 0.6 × warm-up 2's points. They cannot detect a run that would have hit
  `--max-points` first, or one whose later iterations shrink faster than
  observed on 196, 744 and 636.
- **Survey per-point cost**: wall ÷ (parts × 240 000). It cannot apportion
  between parts, nor split the integrand from the density sum.
- **Memory run**: one set, dev profile, `-j 2`, RSS sampled at 1 s, which can
  miss a sub-second peak. It identifies the helicity expansion by timing (RSS
  flat through the compile, growing from its end), not by allocation
  profiling.
- **290 and 722**: the `git log`/`diff` shows no library change. It cannot
  see a difference in the inputs v022-1009 used.

## Validator request (draft; the manager sends it)

No re-run, reading existing files only:
1. **Card 290**: diff v022-1009's staged card, run card and command (all five
   seeds) against `survey-pr3/out/290.cmd` and its staged cards, and give the
   v022-1009 outcome record (exit status, last log lines). Expected: an input
   or harness difference, since the library is identical.
2. **Card 636**: the gridpack's own MadGraph σ (banner or `results.dat`), to
   compare with 2.2e-15 pb.
3. **Card 722**: v022-1009's per-seed wall and the batch slot's load, if
   recorded.

## Brief corrections

- **"Card 290 now refuses … where v0.2.2 timed out"** implies a change
  between them. There is none in the library (see above), so the outcome
  difference is in the inputs or the record, not the code.
- **"Card 722 … did something between v0.2.2 and 4aab24fb speed it up?"**:
  no code change; borderline cost.
- **"The warm-up schedule L-T1 Found on ssWW, or the point budget"** are one
  mechanism, not alternatives. Also, `--neval 120000` does not set the point
  budget on these cards: the channel floor and `--min-iters 6` (default) do.
- **"Amplitude probe … would reusing the compile's evaluators remove it?"**:
  it removes the probe's cost, not the compile's, and not the memory: both
  loops are serial, and the memory is the helicity expansion, which the
  compile phase repeats.
- **"Is the probe's cost and memory the duplicate compile?"** (37): the cost
  yes, the memory no.
- The relayed iteration σ values are in GeV⁻², not pb (the integrand's
  natural unit; `Observable::per_natural_unit` converts only the result).
  Corrected on the sample page.
- The `alpha_survey` phase has no per-iteration rate line, so the brief's
  "cost per point" is wall ÷ points here.
- `amplitude-probe-compiles-each-set-twice` already exists. L-T1's Found
  re-raised it as new.

## Kb edits

- [v022-1009-phase-sample.md](../v022-1009-phase-sample.md) gains the second
  relay, as relayed. The only conversion is GeV⁻² to pb, marked.
- The closed item is deleted. Two pages linked to it and now link to this
  report:
  - `research/kb/validation/cms-gridpack-refusals-202a540.md`;
  - `research/kb/backlog/validation/idm-two-to-four-reads-one-percent-high.md`.
  The second is a backlog item. The link repair is the only change there,
  and the manager may prefer to make it differently.

## Found

Ready-to-file items. The manager files them; none is created here.

**1. `enumeration-hands-flavour-violating-subprocesses-to-feyngraph`**
- **title**: "Enumeration builds topologies for every charge-conserving
  subprocess, ~97% of which the model gives no diagram"
- **area**: performance. **priority**: high.
- **closes_when**: "Before topology assignment, an assignment whose total of
  any additive quantum number conserved by every vertex of the model is
  nonzero is skipped. The numbers are derived from the vertex table, not
  listed by hand. The diagram sets and enumeration census are identical on
  the banked layer and on a CKM model card. The enumerate wall time on a
  W(→ℓν)+0–4j MLM card and on Z(→νν)+4j is measured before and after."
- **body**: `generate_sets_inner` calls `assign_topologies` on every
  surviving (deduplicated, charge-conserving) assignment.
  - On L-T1's W+0–4j smoke run (dev, `-j 2`), the 3j line handed 13 734
    subprocesses to feyngraph, and 289 had diagrams. 150 of the line's 154 s
    went to the empty ones, at about 11 ms each against 14 ms for a
    non-empty one.
  - A reproduction of the walk gives exactly the run's 13 734, and its 288
    lepton-flavour- and generation-conserving subprocesses are exactly the
    non-empty ones.
  - 4j lines: W 45 720 survivors, 1 284 conserving; Z→νν 60 318 survivors,
    1 878 conserving.
  - Campaign cards 58, 15 and 1009 (release, `-j 4 --parallel-diagrams`)
    died in their last line at 3.9M, 1.8M and 3.8M of 15.9M, 2.9M and 15.9M
    candidates. 1009 had 23 962 subprocesses at about 48 ms each, so about
    2900 s for the line, against about 90 s for its 1 878 conserving ones.
  - Then, separately: the candidate loop is serial, and `--parallel-diagrams`
    only parallelises inside one subprocess's assignment, so a parallel loop
    over subprocesses is the next lever.
  - Cite `diagram-enumeration-cost-unmeasured` (feature), whose first
    question this answers.

**2. `set-compilation-loops-are-serial`**
- **title**: "The amplitude probe and the compile phase compile every
  diagram set one at a time"
- **area**: performance. **priority**: high.
- **closes_when**: "Set compilation (`derive_flavor_groups`, and
  `amplitude_probe::measure` while it compiles) runs across the rayon pool,
  results in set order. σ, grids and probe values are bit for bit on the
  banked layer. The wall time of both phases is measured on a Z(→jj)+0–4j
  card and a VBS ssWW card."
- **body**: Card 65 (Zto2Q-4Jets) spent 1077 s in the probe at 1225 user-s
  for 1200 wall-s, one core of four. The probe and compile loops
  (`amplitude_probe.rs:262`, `proton.rs:1107`) are serial `iter()`s. Card
  14: probe 167 s, compile 178 s. Pair it with
  `amplitude-probe-compiles-each-set-twice`, which should rise to high on
  this evidence: 65's probe is 90% of its cap.

**3. `target-runs-pay-six-channel-floor-iterations`**
- **title**: "A --target-rel run costs at least six iterations of
  n_channels × 512 points, whatever --neval"
- **area**: performance. **priority**: high.
- **closes_when**: "integrate reports, after warm-up 1, the minimum wall
  time its stop rule implies (channels × floor × min-iters × measured
  ns/eval) as a warning. Then either the warm-up and floor schedule for wide
  splits is changed, with σ within the multi-seed criterion on the MLM σ
  rows and a 1068-channel GJ or TT card, and cost × variance recorded over
  ≥10 seeds, or the schedule is kept and the measurement recording why is in
  a performance concept."
- **body**: `--min-iters` defaults to 6 (two warm-ups) and the first
  iteration spends `Σ max(shareⱼ, 512)`.
  - On 1068-channel MLM cards warm-up 1 is about 610k points against
    `--neval 120000`. Later iterations are re-floored up to 4× on accepted
    points, and ns/eval rises 1.2–4× after warm-up 1 as more points pass the
    cuts (744: 0.35 → 1.76 ms).
  - Lower bounds on the wall needed: 32 2454 s, 25 3957 s, 42 3407 s, 46
    2419 s, 992 2480 s, 195 3087 s, 590 2150 s (1.8–3.3× the 1200 s cap).
  - 196 and 744 were below the 1% target by quoted error (0.39%, 0.43%) at
    iterations 5 and 4 and kept going only for `--min-iters`.
  - Per-point cost (cite the two MLM matrix-element items) is the other
    factor.
  - Related: `niter-3-campaigns-have-no-chi2-check`.

**4. `alpha-survey-spends-240k-points-per-part`**
- **title**: "The α survey spends 6 × 40 000 points on every multiplicity or
  mass part, whatever its share"
- **area**: performance. **priority**: medium.
- **closes_when**: "The survey's points are budgeted across parts (by
  channel count or the previous survey's spread) with a per-part floor. The
  survey wall time on 655 or 636 is measured, and σ is unchanged within the
  multi-seed criterion on the MLM rows and one mass-split VBF card."
- **body**: `MultiplicitySum::adapt_alphas` runs a full
  `ADAPT_ITERS × survey_points(neval)` survey per part.
  - 655 (6 parts): 1.44M points, 912 s.
  - 636 (15 parts): 3.6M points, 338 s.
  - 14 (3 parts): 720k points, 844 s.
  - On single-part cards a survey point costs what a VEGAS warm-up point
    costs (195 2.29 against 2.19 ms, 744 0.36 against 0.35), so the survey's
    cost is points × integrand cost.
  - Cite `mass-split-parts-thin-the-per-group-budget` and
    `adapt-survey-bounds-untested-at-hundreds-of-channels`.

**5. `wide-group-proton-point-cost-unprofiled`**
- **title**: "On 496-group VBS cards a point costs ~9 ms CPU, mostly not in
  helicities"
- **area**: performance. **priority**: medium.
- **closes_when**: "A profile of the integrand point on a VBS_WPMZTo4Q-shaped
  card attributes its cost (matrix elements, kinematics, cuts, PDF weights,
  channel densities), and the dominant part is filed or fixed."
- **body**:
  - 133 and 710 share 528 subprocesses, 496 flavour groups and 298 channels.
    Their helicities are 8136 against 904, yet their survey costs per point
    are only about 6.3 against 2.7 ms wall at 4 threads. About 9 ms CPU per
    point does not scale with helicities.
  - 528 subprocesses group into 496 flavour groups, so the |M|² grouping
    barely merges anything there.
  - Cite `mlm-every-group-matrix-element-evaluated-per-point`, whose
    one-term draw applies to this unmatched pooling too.

**6. `helicity-expansion-exceeds-memory-on-seven-gluon-sets`**
- **title**: "Binding the g g > 5g evaluator expands all 128 helicities into
  a 4.5–4.9 GB arena"
- **area**: performance. **priority**: high.
- **closes_when**: "Peak RSS of `integrate --check` on `g g > g g g g g` is
  below 2 GB (measured), with |M|² bit for bit or within 1e-12 on the
  amplitude oracle and σ unchanged on the banked layer; the compile phase's
  memory on a `p p > 5j` card (all sets held) is measured."
- **body**:
  - Card 37 (QCD-5Jets) was killed at 4.60 GB in the probe, at
    `MemoryMax 4500M`.
  - Local, dev: the compile of `g g > 5g` (2485 diagrams, 138 323 arena
    nodes, 128 helicities) runs at 70 MB for 66.7 s. RSS then grows about
    150 MB/s, from the first bind in `prune_zero_helicities`
    (`folded.expand_helicities` over all helicities), to a 4.93 GB peak.
  - Pruning keeps about 112 of the 128, so the fix is not pruning first. It
    is to evaluate without materialising nodes × helicities at once (by
    helicity chunk, or an unexpanded arena for wide sets).
  - The compile phase holds every set's evaluator, so card 37 needs this
    whether or not the probe duplicate goes.

**7. `vbf-radion-1800-sigma-implausible-and-unconverged`** (validation)
- **title**: "VBFRadionTo2W M1800 integrates to 2.2e-15 pb with iteration
  χ²/dof 8–26"
- **area**: validation. **priority**: medium.
- **closes_when**: "Card 636's σ is compared with its gridpack's MadGraph σ,
  and either the difference is diagnosed per event or per diagram, or the
  value is confirmed and the iteration inconsistency is attributed to a
  channel."
- **body**:
  - Card 636 (15 parts, 1544 flavour groups, 236 channels) ran 11 VEGAS
    iterations in 802 s to 5.65e-24 GeV⁻² (2.2e-15 pb).
  - The estimate jumped +2σ between iterations 4 and 5, with χ²/dof 26.5
    falling to 8.0.
  - At the stop rule's widened error the run was about 3% from a 1% target.
  - A 1.8 TeV VBF radion would ordinarily be ~fb, not ~1e-15 pb, so the value
    may be a silent wrong answer, or a parameter card with a vanishing
    coupling (cf. `param-card-zero-couplings-keep-dead-diagrams`).

**Not items (for the manager):**
- The validator's iteration σ were labelled pb but are GeV⁻².
- Card 290's v022-1009 record is inconsistent with an unchanged library (see
  the validator request).
- The phase instrument has no per-iteration rate on `alpha_survey`. If wanted,
  it belongs with the instrument on PR 3a, not here.

## Manager addenda

- Card 636: vg-validator found the gridpack's MadGraph 2.9.18 σ is 2.222e-15 ± 4.2e-18 pb, so vg/MG ≈ 0.99 and the value is not a silent wrong answer. Draft item 7 was filed as `vbf-radion-1800-iterations-inconsistent` (validation, low): convergence and timeout only.
- PR 3b (planner ruling): items 1 and 2 plus `amplitude-probe-compiles-each-set-twice`. Items 3–6 go to the next sprint.
