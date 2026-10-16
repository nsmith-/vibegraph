# MadEvent: the first `setclscales` call clusters the unpermuted point

A MadGraph-only reproducer. It uses nothing from this repository except the
pinned MadGraph tree and the MG5 scripts and cards in this directory. The fix is
`../../patches/first-call-unpermuted-momenta.patch`.

## The defect (MadGraph 3.7.1, tree `b7687064`)

In grouped output, `DSIGPROC`
(`madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc`) builds
`P1 = SWITCHMOM(PP, PERMS(MAPCONFIG(ICONFIG)))` (`:805`) and mirrors it
(`:814-826`). The matrix element and `REWGT`'s second `setclscales` call read
`P1`. The first call, `update_scale_coupling(pp, wgt)` (`:842`), is handed the
unpermuted `PP`.

That first call:
- sets μR;
- sets the PDF scales, at which `DSIG` evaluates the densities before `REWGT`;
- sets `q2bck`;
- can reject the point: the `xqcut` test on every clustering vertex with a jet
  daughter (`Template/LO/SubProcesses/reweight.f:1066-1085`).

`REWGT` returns 0 when the second call rejects (`reweight.f:1465`). A point
therefore survives only if both calls accept it.

When a channel's symmetry permutation exchanges equal-mass legs of different
flavour, the first call clusters a relabelled event, and σ comes out low.
Non-grouped output never permutes: there, `P1` is `PP` boosted.

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

## Running it

Any MadGraph 3.7.x with a Fortran compiler will do:

```
MG5=/path/to/MG5_aMC_v3_7_1/bin/mg5_aMC ./repro.sh /tmp/permtest 11 12 13 14 15
```

In this repository, with the pinned tree:

```
pixi run -e madgraph env MG5="bash $PWD/validation/madgraph/mg5_pinned.sh" \
  bash validation/madgraph/repro/permuted_first_call/repro.sh <workdir> 11 12 13 14 15
```

- Every seed and variant runs in a fresh copy of a freshly generated process
  directory.
- One line per run is printed and appended to `<workdir>/results.txt`.
- The summary gives each variant's mean and its sample standard deviation over
  seeds. It then gives each variant's pull against `nongrouped`, using the
  measured spreads rather than MadEvent's quoted errors.
- `RUN_CARD=run_card_ickkw0.dat` repeats the test with MLM off: the default
  card, `dynamical_scale_choice = -1`, `use_syst = F`.

## Expected output

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
  [RUN_CARD=$PWD/validation/madgraph/repro/permuted_first_call/run_card_ickkw0.dat] \
  bash validation/madgraph/repro/permuted_first_call/repro.sh <workdir> 11 12 13 14 15
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
