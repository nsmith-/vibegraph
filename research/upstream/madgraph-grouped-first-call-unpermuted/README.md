# MadEvent: the first `setclscales` call clusters the unpermuted point

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler and the MG5 scripts and cards in this directory, and nothing from
vibegraph.

## What it shows

In grouped output (`group_subprocesses True`, the default), MadEvent sets the
dynamical scales, and applies the MLM `xqcut` rejection, on the sampled point
before the channel's symmetry permutation. The matrix element is evaluated on
the permuted point. On `define q = u d; generate u q > z u q`, the one-line
fix below raises grouped σ:
- by 3.7 % (14.6σ) on 3.8.1 and 4.2 % on 3.7.1 at default cards;
- by 3.4 % (about 15σ on 3.8.1) with MLM.

On 3.7.1 the fixed grouped output agrees with the non-grouped output.

## Status

**Filed** as [mg5amcnlo/mg5amcnlo#453](https://github.com/mg5amcnlo/mg5amcnlo/issues/453) on 2026-10-19.

**Fixed upstream for 3.8.2** (closed 2026-10-25). [PR #462](https://github.com/mg5amcnlo/mg5amcnlo/pull/462)
(merge [`e76e9458`](https://github.com/mg5amcnlo/mg5amcnlo/commit/e76e94580c2a6c2cd312ec578181f477c1436d78)) is on the
3.8.2 development branch; no LTS change is needed.
- The scalar path takes the draft's one-line fix,
  `update_scale_coupling(p1, wgt)` ([`fdfe9483`](https://github.com/mg5amcnlo/mg5amcnlo/commit/fdfe9483dfbb3160ed2906add6f4ffbe6f88c618)).
- The vectorised path, which the draft flagged untested, now sets the scales
  per warp in `DSIGPROC_VEC` after the permutation and mirror
  ([`0612d849`](https://github.com/mg5amcnlo/mg5amcnlo/commit/0612d849e212628c5c232a7ab3446b86b752bd30)). The same commit
  keeps `q2bck` per event, which every event of a batch used to share.
- `coupl.inc` now sizes `all_G`/`all_mu_r` by `VECSIZE_MEMMAX`, fixing a crash
  when `vector_size` is raised after output
  ([`3ce090e6`](https://github.com/mg5amcnlo/mg5amcnlo/commit/3ce090e661a16ce1e3309f9c5f964df825ba8688)).
- A new acceptance test, `test_group_subprocess_permuted_scale`, compares
  grouped (scalar and vector) with non-grouped output. The PR reports grouped σ
  about 3 % low before the fix, against 3.7–3.9 % here.

**Present in 3.8.1** (ran). It is also present in 3.7.1.
- **Absent from the 3.5.17 LTS release** (ran).
- **Introduced in 3.6.0.** The unpermuted `update_scale_coupling(pp, wgt)` call
  first appears in `v3.6.0`, by reading the tags; this was not bisected. The report draft is
[`report-draft.md`](report-draft.md).

## Where

`madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc`, `DSIGPROC`:

| | 3.7.1 | 3.8.1 |
|---|---|---|
| `P1 = SWITCHMOM(PP, PERMS(...))` | `:805` | `:819` |
| mirror of `P1` | `:814-826` | `:829` |
| `call update_scale_coupling(pp, wgt)` | `:842` | `:856` |
| vectorised `update_scale_coupling_vec(all_p, ...)` | `:312` | `:317` |

The fix is one line: `call update_scale_coupling(p1, wgt)`. In this
repository it is `validation/madgraph/patches/first-call-unpermuted-momenta.patch`.

## Root cause

- *Verified* (the `patched` variant): handing the first call `P1` instead of
  `PP` removes the deficit.
  - On 3.7.1, grouped + fix agrees with non-grouped at both cards.
  - On 3.8.1 it does so once the unrelated refine change below is undone.
- *Inferred from source:*
  - `DSIGPROC` builds `P1 = SWITCHMOM(PP, PERMS(MAPCONFIG(ICONFIG)))` and
    mirrors it, and the matrix element and `REWGT`'s second `setclscales` call
    read `P1`. The first call, `update_scale_coupling(pp, wgt)`, is handed the
    unpermuted `PP`.
  - That first call sets μR, the PDF scales at which `DSIG` evaluates the
    densities, and `q2bck`. It can also reject the point: the `xqcut` test on
    every clustering vertex with a jet daughter
    (`Template/LO/SubProcesses/reweight.f:1066-1085`).
  - `REWGT` returns 0 when the second call rejects, so a point survives only
    if both calls accept it.
  - Non-grouped output never permutes: there, `P1` is `PP` boosted.
- *Not isolated:* the clustering path that makes some identical-quark
  permutations matter (channel `G4` at the default card) and not others
  (`G3`).
- *Inferred, not measured:* the vectorised path (`vector_size > 1`) has the
  same defect and needs a larger change.

## The test

The process is `define q = u d; generate u q > z u q`, which has two
subprocesses, `u u > z u u` and `u d > z u d`.
- In grouped output they share one `P1_qq_zqq`. Its `symperms.inc` maps configs
  2, 5, 6 and 8 onto others with the final-quark swap `(1,2,3,5,4)`.
- `u d > z u d` has two of its four diagrams on those configs (2 and 8), so on
  them the swap exchanges the physical `u` and `d`.
- With `set group_subprocesses False`, the same two subprocesses get their own
  directories and no permutation.

The two outputs must give the same σ. A third variant, the grouped output with
the one-line fix applied, is the control.

The run card is MadGraph 3.7.1's default LO card with these changes:

| setting | value | effect |
|---|---|---|
| `ickkw` | 1 | MLM matching |
| `xqcut` | 40 | `auto_ptj_mjj` then sets `ptj = mmjj = 40` and `drjj = 0` |
| `nevents` | 5000 | |
| `use_syst` | F | |

Beams are the default: `pp` at 13 TeV, the built-in `nn23lo1` PDF, and the
default dynamical scale. No LHAPDF is needed.

## Running it (versions tested: 3.7.1 `b7687064`, 3.8.1 `a5b17913`)

Any MadGraph 3.7.x or 3.8.x with a Fortran compiler will do:

```
MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/permtest 11 12 13 14 15
```

In vibegraph, with the pinned 3.7.1 tree (submodule commit `b7687064`):

```
pixi run -e madgraph env MG5="bash $PWD/validation/madgraph/mg5_pinned.sh" \
  bash research/upstream/madgraph-grouped-first-call-unpermuted/repro.sh <workdir> 11 12 13 14 15
```

- Every seed and variant runs in a fresh copy of a freshly generated process
  directory.
- One line per run is printed and appended to `<workdir>/results.txt`.
- The summary gives each variant's mean and its sample standard deviation over
  seeds. It then gives each variant's pull against `nongrouped`, using the
  measured spreads rather than MadEvent's quoted errors.
- `RUN_CARD=run_card_ickkw0.dat` repeats the test with MLM off: the default
  card, `dynamical_scale_choice = -1`, `use_syst = F`.

## Expected against actual

**Expected:** grouped and non-grouped give the same σ; the patched grouped
output is the control.

These are the MadGraph 3.7.1 runs at pin `b7687064`, seeds 11–15. The build
was the Linux conda toolchain from `pixi run -e madgraph`, sharing 4 cores with
other jobs.

```
grouped    seed 11     sigma 54.102 +- 0.222 pb  (47s)
nongrouped seed 11     sigma 56.019 +- 0.196 pb  (64s)
patched    seed 11     sigma 56.146 +- 0.206 pb  (94s)
grouped    seed 12     sigma 54.612 +- 0.179 pb  (256s)
nongrouped seed 12     sigma 56.362 +- 0.193 pb  (99s)
patched    seed 12     sigma 56.346 +- 0.18 pb  (84s)
grouped    seed 13     sigma 54.308 +- 0.164 pb  (150s)
nongrouped seed 13     sigma 55.83 +- 0.236 pb  (54s)
patched    seed 13     sigma 56.069 +- 0.201 pb  (62s)
grouped    seed 14     sigma 54.928 +- 0.219 pb  (82s)
nongrouped seed 14     sigma 56.278 +- 0.185 pb  (80s)
patched    seed 14     sigma 56.832 +- 0.186 pb  (103s)
grouped    seed 15     sigma 54.468 +- 0.166 pb  (138s)
nongrouped seed 15     sigma 56.228 +- 0.283 pb  (194s)
patched    seed 15     sigma 56.279 +- 0.158 pb  (144s)

variant     n   mean [pb]   spread [pb]  err(mean)  mean quoted err
grouped     5  54.4836  0.313      0.14    0.19
nongrouped  5  56.1434  0.216      0.0966    0.219
patched     5  56.3344  0.299      0.134    0.186
grouped - nongrouped: -1.66 pb (-2.96%), pull -9.8 sigma (measured spreads)
patched - nongrouped: +0.191 pb (+0.34%), pull +1.2 sigma (measured spreads)
```

- The grouped output is 3.0 % low, a pull of −9.8σ against the measured
  seed spreads.
- The patched grouped output agrees with the non-grouped one (+1.2σ).

Per channel, `<workdir>/channels.txt` gives the grouped and patched means over
seeds 13–15:

| channel | grouped (pb) | patched (pb) | shift | pull |
|---|---|---|---|---|
| `G1` | 13.342 | 14.369 | +7.7 % | +14.5σ |
| `G7` | 11.605 | 12.156 | +4.7 % | +5.7σ |
| `G3` | 16.306 | 16.297 | −0.06 % | 0.0σ |
| `G4` | 13.315 | 13.572 | +1.9 % | +2.1σ |

- `G1` and `G7` hold configs 2 and 8. There the swap exchanges the physical
  `u` and `d` of `u d > z u d`.
- `G3` and `G4` hold configs 5 and 6, whose swap exchanges only the identical
  quarks of `u u > z u u`.
- `G4`'s +2.1σ is not significant here. With MLM off, below, the same channel
  moves by 18σ.

**Wall time.** Each run takes about 50–250 s, most of it compilation and
survey. The 15 runs above took 28 minutes in total on a loaded 4-core machine.
Three seeds (9 runs) already separate the grouped and non-grouped means by
more than 5σ of the measured spread.

### With MLM off (`RUN_CARD=run_card_ickkw0.dat`)

The card is MadGraph's default LO card as generated, with `nevents = 5000` and
`use_syst = F`. That means `ickkw = 0`, no `xqcut`, and
`dynamical_scale_choice = -1`. The first call is then the only `setclscales`
call. It sets μR and μF from the clustering. At `ickkw = 0` its only possible
rejection is the 2 GeV floor on the factorisation scale.

```
grouped    seed 11     sigma 104.95 +- 0.357 pb  (87s)
nongrouped seed 11     sigma 109.66 +- 0.355 pb  (157s)
patched    seed 11     sigma 109.63 +- 0.374 pb  (73s)
grouped    seed 12     sigma 105.05 +- 0.382 pb  (101s)
nongrouped seed 12     sigma 106.92 +- 0.399 pb  (127s)
patched    seed 12     sigma 108.82 +- 0.428 pb  (76s)
grouped    seed 13     sigma 103.32 +- 0.313 pb  (118s)
nongrouped seed 13     sigma 108.79 +- 0.4 pb  (111s)
patched    seed 13     sigma 107.79 +- 0.332 pb  (110s)
grouped    seed 14     sigma 103.67 +- 0.388 pb  (74s)
nongrouped seed 14     sigma 108.85 +- 0.357 pb  (98s)
patched    seed 14     sigma 107.89 +- 0.408 pb  (77s)
grouped    seed 15     sigma 104.43 +- 0.331 pb  (159s)
nongrouped seed 15     sigma 108.77 +- 0.412 pb  (116s)
patched    seed 15     sigma 108.96 +- 0.352 pb  (95s)

variant     n   mean [pb]   spread [pb]  err(mean)  mean quoted err
grouped     5  104.284  0.768      0.343    0.354
nongrouped  5  108.598  1.01      0.451    0.385
patched     5  108.618  0.774      0.346    0.379
grouped - nongrouped: -4.314 pb (-3.97%), pull -7.6 sigma (measured spreads)
patched - nongrouped: +0.02 pb (+0.02%), pull +0.0 sigma (measured spreads)
```

- With default cards, grouped output is 4.0 % low (−7.6σ).
- The patched grouped output agrees with the non-grouped one.

Per channel, the grouped and patched means over seeds 11–15:

| channel | grouped (pb) | patched (pb) | shift | pull |
|---|---|---|---|---|
| `G1` | 28.046 | 27.957 | −0.3 % | −0.3σ |
| `G3` | 30.916 | 30.916 | identical on every seed | |
| `G4` | 24.760 | 26.173 | +5.7 % | +18σ |
| `G7` | 20.561 | 23.570 | +14.6 % | +36σ |

`G4`'s only permuted config (6) swaps the two identical quarks of `u u > z u u`
(`u d` has no diagram on it), so identical-particle permutations are affected
too. `G3`'s config 5 is the same kind of swap and is not affected. Which
clustering path distinguishes the two was not isolated.

**Wall time, both tests.** The MLM test took 28 minutes (15 runs) and the
MLM-off test took 27 minutes (15 runs), on a 4-core machine shared with other
builds.

## MadGraph 3.8.1 (latest release, tag `v3.8.1`, commit `a5b17913`)

The defect is unchanged in 3.8.1. The call is `super_auto_dsig_group_v4.inc:856`,
`P1` is built at `:819` and mirrored at `:829`, and the generated `auto_dsig.f`
carries the same `CALL UPDATE_SCALE_COUPLING(PP, WGT)`.

The runs below used the same conda toolchain, on 2026-10-16, with:

```
pixi run -e madgraph env VG_MG5_ROOT=<3.8.1 checkout> MG5="bash $PWD/validation/madgraph/mg5_pinned.sh" \
  [RUN_CARD=$PWD/research/upstream/madgraph-grouped-first-call-unpermuted/run_card_ickkw0.dat] \
  bash research/upstream/madgraph-grouped-first-call-unpermuted/repro.sh <workdir> 11 12 13 14 15
```

With MLM off (`run_card_ickkw0.dat`):

```
grouped    seed 11     sigma 106.64 +- 0.306 pb  (37s)
nongrouped seed 11     sigma 109.66 +- 0.355 pb  (48s)
patched    seed 11     sigma 110.59 +- 0.399 pb  (21s)
grouped    seed 12     sigma 107.33 +- 0.349 pb  (26s)
nongrouped seed 12     sigma 106.92 +- 0.399 pb  (42s)
patched    seed 12     sigma 111.09 +- 0.356 pb  (33s)
grouped    seed 13     sigma 106.61 +- 0.391 pb  (24s)
nongrouped seed 13     sigma 108.79 +- 0.4 pb  (38s)
patched    seed 13     sigma 110.8 +- 0.428 pb  (22s)
grouped    seed 14     sigma 106.13 +- 0.333 pb  (26s)
nongrouped seed 14     sigma 108.85 +- 0.357 pb  (32s)
patched    seed 14     sigma 110.16 +- 0.329 pb  (25s)
grouped    seed 15     sigma 106.49 +- 0.452 pb  (22s)
nongrouped seed 15     sigma 108.77 +- 0.412 pb  (38s)
patched    seed 15     sigma 110.13 +- 0.442 pb  (24s)
variant     n   mean [pb]   spread [pb]  err(mean)  mean quoted err
grouped     5  106.64  0.436      0.195    0.366
nongrouped  5  108.598  1.01      0.451    0.385
patched     5  110.554  0.414      0.185    0.391
grouped - nongrouped: -1.958 pb (-1.80%), pull -4.0 sigma (measured spreads)
patched - nongrouped: +1.956 pb (+1.80%), pull +4.0 sigma (measured spreads)
```

MLM card (`run_card.dat`):

```
grouped seed 11: attempt 1 failed, see /tmp/claude-0/-home-user-vibegraph/f027113e-ae0f-56cd-90fe-cca0ffca80be/scratchpad/runs/h1_381_mlm/grouped_11.log
grouped    seed 11     sigma 55.612 +- 0.193 pb  (35s)
nongrouped seed 11     sigma 56.056 +- 0.193 pb  (33s)
patched    seed 11     sigma 57.382 +- 0.187 pb  (42s)
grouped    seed 12     sigma 55.61 +- 0.203 pb  (29s)
nongrouped seed 12     sigma 56.394 +- 0.194 pb  (40s)
patched    seed 12     sigma 57.786 +- 0.195 pb  (39s)
grouped    seed 13     sigma 55.314 +- 0.212 pb  (28s)
nongrouped seed 13     sigma 55.929 +- 0.241 pb  (30s)
patched    seed 13     sigma 57.312 +- 0.24 pb  (27s)
grouped    seed 14     sigma 55.877 +- 0.213 pb  (46s)
nongrouped seed 14     sigma 56.247 +- 0.184 pb  (41s)
patched    seed 14     sigma 57.654 +- 0.219 pb  (47s)
grouped    seed 15     sigma 55.652 +- 0.182 pb  (59s)
nongrouped seed 15     sigma 56.222 +- 0.277 pb  (90s)
patched    seed 15     sigma 57.517 +- 0.211 pb  (37s)
variant     n   mean [pb]   spread [pb]  err(mean)  mean quoted err
grouped     5  55.613  0.2      0.0896    0.201
nongrouped  5  56.1696  0.18      0.0806    0.218
patched     5  57.5302  0.194      0.0867    0.21
grouped - nongrouped: -0.5566 pb (-0.99%), pull -4.6 sigma (measured spreads)
patched - nongrouped: +1.361 pb (+2.42%), pull +11.5 sigma (measured spreads)
```

Grouped against grouped + fix, at the same seeds:

| card | grouped (pb) | grouped + fix (pb) | shift |
|---|---|---|---|
| default | 106.64 ± 0.20 | 110.55 ± 0.19 | +3.7 %, 14.6σ |
| MLM | 55.61 ± 0.09 | 57.53 ± 0.09 | +3.4 %, 15.4σ |

**The non-grouped comparison is confounded in 3.8.1.** The fixed grouped
output now reads +1.8 % (default) and +2.4 % (MLM) above the non-grouped one.
The non-grouped σ at the default card is bit-identical per seed to 3.7.1.

The extra shift is a change to MadEvent's refine step. It is not this defect:
- Upstream commit `ebbdc974` ("move back to refine_grid=False", merged into
  3.8.0 with 3.5.16) sets `keep_grid_for_refine = False` in
  `madgraph/madevent/gen_ximprove.py`.
- It raises grouped σ by about 2 % in every channel, including `G3`, whose
  configs carry no permutation (default card, `patched` per channel: `G3`
  31.72 against 30.92 pb in 3.7.1).

It was located by bisecting the first-parent merges from `v3.7.1` to `v3.8.1`
at seed 11, `patched`. The value steps 109.63 → 110.78 pb at merge
`31c57a65e`. A later reshuffle of the random stream (`aa59ff474`) moves it
to 110.59.

With that one setting restored (`keep_grid_for_refine = True` and its
`base_directory` line in a copy of the 3.8.1 tree), 3.8.1 reproduces 3.7.1's
picture:

```
grouped    seed 11     sigma 105.09 +- 0.342 pb  (25s)
nongrouped seed 11     sigma 109.66 +- 0.355 pb  (36s)
patched    seed 11     sigma 109.65 +- 0.371 pb  (23s)
grouped    seed 12     sigma 104.72 +- 0.43 pb  (33s)
nongrouped seed 12     sigma 106.92 +- 0.399 pb  (40s)
patched    seed 12     sigma 109.15 +- 0.448 pb  (34s)
grouped    seed 13     sigma 103.95 +- 0.329 pb  (36s)
nongrouped seed 13     sigma 108.79 +- 0.4 pb  (39s)
patched    seed 13     sigma 107.99 +- 0.337 pb  (32s)
grouped    seed 14     sigma 103.51 +- 0.363 pb  (20s)
nongrouped seed 14     sigma 108.85 +- 0.357 pb  (22s)
patched    seed 14     sigma 108.06 +- 0.392 pb  (17s)
grouped    seed 15     sigma 104.62 +- 0.328 pb  (21s)
nongrouped seed 15     sigma 108.77 +- 0.412 pb  (35s)
patched    seed 15     sigma 108.41 +- 0.354 pb  (20s)
variant     n   mean [pb]   spread [pb]  err(mean)  mean quoted err
grouped     5  104.378  0.636      0.285    0.358
nongrouped  5  108.598  1.01      0.451    0.385
patched     5  108.652  0.723      0.323    0.38
grouped - nongrouped: -4.22 pb (-3.89%), pull -7.9 sigma (measured spreads)
patched - nongrouped: +0.054 pb (+0.05%), pull +0.1 sigma (measured spreads)
```

## MadGraph 3.5.17 (latest LTS, tag `v3.5.17`, commit `4492b3e3`)

**Not affected.** The code differs:
- 3.5.17's `DSIGPROC` has no `update_scale_coupling` call.
- The first `setclscales` call runs inside `PASSCUTS(P1)` (`cuts.f:1236`), on
  the permuted point. `PASSCUTS` caches its result through `CUTSDONE`.

The `patched` variant does not apply here, because the line it edits does not
exist. So these runs compare grouped against non-grouped. They used the same
command as above, with `VARIANTS="grouped nongrouped"`, on 2026-10-17.

With MLM off (`run_card_ickkw0.dat`):

```
grouped    seed 11     sigma 109.57 +- 0.479 pb  (32s)
nongrouped seed 11     sigma 108.67 +- 0.338 pb  (71s)
grouped    seed 12     sigma 110.88 +- 0.421 pb  (54s)
nongrouped seed 12     sigma 108.71 +- 0.412 pb  (25s)
grouped    seed 13     sigma 110.4 +- 0.471 pb  (22s)
nongrouped seed 13     sigma 109.01 +- 0.384 pb  (22s)
grouped    seed 14     sigma 110.35 +- 0.456 pb  (18s)
nongrouped seed 14     sigma 108.65 +- 0.37 pb  (38s)
grouped    seed 15     sigma 110.5 +- 0.532 pb  (14s)
nongrouped seed 15     sigma 108.56 +- 0.371 pb  (339s)
variant     n   mean [pb]   spread [pb]  err(mean)  mean quoted err
grouped     5  110.34  0.478      0.214    0.472
nongrouped  5  108.72  0.171      0.0766    0.375
grouped - nongrouped: +1.62 pb (+1.49%), pull +7.1 sigma (measured spreads)
```

MLM card (`run_card.dat`):

```
grouped    seed 11     sigma 57.41 +- 0.185 pb  (30s)
nongrouped seed 11     sigma 56.205 +- 0.192 pb  (131s)
grouped    seed 12     sigma 57.8 +- 0.205 pb  (45s)
nongrouped seed 12     sigma 56.514 +- 0.188 pb  (44s)
grouped    seed 13     sigma 57.522 +- 0.234 pb  (48s)
nongrouped seed 13     sigma 56.481 +- 0.216 pb  (77s)
grouped    seed 14     sigma 57.396 +- 0.21 pb  (27s)
nongrouped seed 14     sigma 56.642 +- 0.193 pb  (73s)
grouped    seed 15     sigma 57.559 +- 0.23 pb  (27s)
nongrouped seed 15     sigma 55.826 +- 0.189 pb  (69s)
variant     n   mean [pb]   spread [pb]  err(mean)  mean quoted err
grouped     5  57.5374  0.163      0.0728    0.213
nongrouped  5  56.3336  0.325      0.146    0.196
grouped - nongrouped: +1.204 pb (+2.14%), pull +7.4 sigma (measured spreads)
```

Grouped reads above non-grouped here, not below. The size matches 3.8.1's
refine-grid offset, and 3.5.17 carries the same `keep_grid_for_refine = False`
(`gen_ximprove.py:1183`). The 3.5.17 grouped means (110.34 and 57.54 pb) equal
3.8.1's grouped + fix (110.55 and 57.53 pb).

With `keep_grid_for_refine = True` restored in a copy of the 3.5.17 tree, at
the default card:

```
grouped    seed 11     sigma 109.28 +- 0.366 pb  (53s)
nongrouped seed 11     sigma 108.67 +- 0.338 pb  (53s)
grouped    seed 12     sigma 105.94 +- 0.348 pb  (51s)
nongrouped seed 12     sigma 108.71 +- 0.412 pb  (24s)
grouped    seed 13     sigma 108.55 +- 0.333 pb  (86s)
nongrouped seed 13     sigma 109.01 +- 0.384 pb  (22s)
grouped    seed 14     sigma 108.77 +- 0.437 pb  (32s)
nongrouped seed 14     sigma 108.65 +- 0.37 pb  (35s)
grouped    seed 15     sigma 107.6 +- 0.362 pb  (46s)
nongrouped seed 15     sigma 108.56 +- 0.371 pb  (320s)
variant     n   mean [pb]   spread [pb]  err(mean)  mean quoted err
grouped     5  108.028  1.32      0.589    0.369
nongrouped  5  108.72  0.171      0.0766    0.375
grouped - nongrouped: -0.692 pb (-0.64%), pull -1.2 sigma (measured spreads)
```

That is −0.6 % (−1.2σ; seed 12 alone reads 105.94). Under the same setting,
3.8.1's stock grouped output is −3.9 % (−7.9σ). There is no H1-sized deficit
on 3.5.17.

## Notes

- MadEvent's parallel `Source` build occasionally fails with
  `libgammaUPC.a: error adding symbols: archive has no index`. `repro.sh`
  retries a failed run once in a fresh copy. The build failed on 2 of the 30 runs
  above.
- The patched variant edits the generated `auto_dsig.f` rather than
  regenerating with a patched MadGraph. Generating with the patch applied to
  the template was checked to give a byte-identical
  `SubProcesses/P1_qq_zqq/auto_dsig.f` and no other difference in the
  subprocess directory.
