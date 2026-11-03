---
type: Session Report
title: "T-V1 report: artifact version 15 with a channel-map digest; the CMS cards' σ is regulated by the 2 GeV factorisation floor"
description: "FORMAT_VERSION is 15. The artifact banks a SHA-256 over every channel's map_key, which generate checks on both paths. M1 fails the walk/density test at its first diagram. The CMS VBF cards' σ is finite. MadGraph's reweight.f, mirrored in vibegraph, zero-weights any point whose dynamical μF falls below 2 GeV, and that cuts the softest jet at pT = 2 GeV whatever ptj is. With a fixed scale the region opens and σ triples."
status: draft
tags: [artifact, versioning, spine, vbf, factorisation-scale, sprint-e, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
The session brief is [T-V1](T-V1.md). Code commit: `4fdc92b9` on
`sprinte-spine-fallback`. This report is committed after it.

## 1. The artifact version and the channel-map digest

**Scope change.** The brief said not to add a channel-map digest. The
coordinator replaced that on 2026-10-28, by the user's decision: add it in the
same version bump. Both parts are in `4fdc92b9`.

- **`FORMAT_VERSION = OLDEST_READABLE_VERSION = 15`** (`artifact.rs`). The doc
  paragraph for 15 records both changes: the no-cut spine maps, and the new
  field.
- **`IntegrateArtifact::channel_map_digest`** is the last field.
  `artifact::channel_map_digest(keys)` hashes, through
  `ufo::identity::digest_bytes`, every channel's `DiagramChannel::map_key`
  in channel order, each key terminated by `\n`. A key is printable ASCII
  with no newline, and the function `debug_assert`s that. So the encoding is
  injective on the ordered list.
- **The keys are captured where each channel is built.** That is
  `ProtonIntegrand::new_with_maps`, after `without_transfer_bound`, and both
  `FixedBeamIntegrand::use_multichannel*` methods. They are exposed as
  `channel_map_keys()`, and `MultiplicitySum` concatenates them in channel
  order. A flat-RAMBO run banks the digest of the empty list.
- **Integrate** stores the digest from the integrand it trained: the hadronic
  and the fixed-energy path.
- **Generate** runs `check_channel_maps` on both paths. On the hadronic path
  it runs after `check_channel_keys`. On the fixed-energy path it runs after
  the channels are installed. A mismatch is refused with:
  `the artifact's channel-map digest is <a> but this build's channels give
  <b>: the phase-space maps changed since this grid was trained, so
  re-integrate with `vibegraph integrate``.
- **Blind spot**, documented in the code and in both concepts. `map_key`
  encodes a map's parameters: topology, poles, floors, transfer bounds, beam
  masses and √s. It does not encode the code that draws from them. A change
  to how a map with an unchanged key samples or weighs a point passes the
  check, and still needs a hand bump under rule 3.
- **Concepts.** [The integrate artifact](../../../pipeline/integrate-artifact.md)
  gets the field row, check 6 with its blind spot, and the version moved to
  15. [Artifact format versioning](../../../pipeline/artifact-format-versioning.md)
  gets a rule-3 paragraph (map changes are automatic from 15) and the floor
  paragraph moved to 15.
- **Collision with #83.** The systematics PR also bumps to 15. Whichever
  merges second takes 16. Nothing was done about it here.

### Tests

| Test | What it pins |
|---|---|
| `artifact::tests::a_version_14_artifact_is_refused` | Version 14's real shape: this build's payload with the trailing digest removed (the test asserts the removed bytes are the digest), stamped 14. Refused by `UnsupportedVersion { found: 14, oldest: 15, expected: 15 }` with the re-integrate remedy. |
| `artifact::tests::a_version_13_artifact_is_refused` | Kept, now expects `oldest: 15`. |
| `artifact::tests::the_channel_map_digest_reads_order_and_boundaries` | Order, key boundaries (`["ab","c"]` ≠ `["a","bc"]`), a trailing empty key, and the known answer `sha256("a\nb\n")`. |
| `diagram_channel::tests::the_channel_map_digest_separates_a_spine_from_the_timelike_tree` | `u u~ > u u~` with the spine admitted vs `max_rungs = 0`. Both sets have the same channel count and the same `ndim` per channel, which is all the key check reads. The digests differ. One build twice gives one digest. Reversing the order changes it. |
| `cli_generate::a_channel_map_digest_that_moved_is_refused` (fixed-energy) | Round trip: `run()`'s artifact generates. The same artifact with only the digest changed is refused, naming both digests and "re-integrate", and writes no file. |
| `cli_generate_proton::a_hadronic_channel_map_digest_that_moved_is_refused` | The same on the hadronic path (`p p > e+ e-`, banked run card). The unaltered artifact generates 10 events first. |

### Mutations

Each mutation discards one `check_channel_maps` result in
`vibegraph-lib/src/generate.rs`, which is restored afterwards (`cmp` against
the backup gives identical). A first attempt, `let _ = …?;`, kept the `?`
and so mutated nothing. Both tests then passed, and the attempt is recorded
in `T-V1-digest-mut.log`. The corrected mutations:

```
== MUT-fixed: fixed-energy check discarded
    let _ = check_channel_maps(artifact, integ.channel_map_keys());
test a_channel_map_digest_that_moved_is_refused ... FAILED
thread 'a_channel_map_digest_that_moved_is_refused' panicked at vibegraph-cli/tests/cli_generate.rs:725:5:
generate accepted a channel-map digest that moved; it must refuse
== MUT-proton: hadronic check discarded
    let _ = check_channel_maps(artifact, &integ.channel_map_keys());
test a_hadronic_channel_map_digest_that_moved_is_refused ... FAILED
thread 'a_hadronic_channel_map_digest_that_moved_is_refused' panicked at vibegraph-cli/tests/cli_generate_proton.rs:696:5:
a moved channel-map digest was accepted
== restored
identical
```

## 2. Mutation M1 against the walk/density test

M1 restores `&& (n_out == 2 || fiducial_scale > F::zero())` in the spine
condition (`from_diagram_with`). Command:
`cargo test -p vibegraph-lib --test diagram_channel -- without_a_fiducial_scale a_spine_without_a_fiducial_scale`.

```
test a_spine_without_a_fiducial_scale_keeps_its_walk_and_its_density_in_step ... FAILED
test without_a_fiducial_scale_every_peripheral_diagram_gets_its_spine_on_a_pole_floor ... FAILED
thread 'a_spine_without_a_fiducial_scale_keeps_its_walk_and_its_density_in_step' panicked at vibegraph-lib/tests/diagram_channel.rs:2925:9:
u u~ > e+ e- g QCD=2 QED=2 diagram 0: no spine to check without a fiducial scale
thread 'without_a_fiducial_scale_every_peripheral_diagram_gets_its_spine_on_a_pole_floor' panicked at vibegraph-lib/tests/diagram_channel.rs:2809:9:
assertion `left == right` failed: u u~ > e+ e- g QCD=2 QED=2: a 1-line peripheral diagram built 0 rungs with no fiducial scale
test result: FAILED. 0 passed; 2 failed
```

The walk/density test now fails at its first diagram. The source was
restored. The diff against the pre-mutation backup shows only the digest test
added later.

## 3. Is the CMS cards' σ well defined?

**Yes. The regulator is not a kinematic cut.** Both MadGraph and vibegraph
zero-weight a phase-space point whose dynamical factorisation scale falls
below 2 GeV. On these cards that removes every event with a jet softer than
about 2 GeV, whatever `ptj` is. T-F1's hypothesis (nothing regulates σ) is
falsified for the cards as written. It holds for the same cards with a fixed
factorisation scale.

### MadGraph: where the regulator is

Read at the pinned submodule commit `b7687064`:

- `Template/LO/SubProcesses/reweight.f:1205-1218`, in `setclscales`:
  ```
  c     Check that factorization scale is >= 2 GeV
        if(lpp(1).ne.0.and.(q2fact(1).lt.4d0.and..not.fixed_fac_scale1).or.
       $   lpp(2).ne.0.and.(q2fact(2).lt.4d0.and..not.fixed_fac_scale2))then
           ...
           setclscales=.false.
  ```
- `reweight.f:1907-1908`, in `update_scale_coupling_vec`:
  `if(.not.setclscales(all_p(1,i) , .false., i))then all_wgt(i) = 0d0`. This
  is called from `auto_dsig_v4.inc:127` and
  `super_auto_dsig_group_v4.inc:312,842`, so the point's weight is zero.
- The cards set `fixed_fac_scale = False` and `dynamical_scale_choice = -1`
  (the clustering scale), so the floor is live. For a forward quark, the
  clustering scale on its beam is about the quark's pT.

Nothing else regulates the region:

- `setcuts.f` sets `etmin`, `s_min` and `smin` only from the card (`ptj`,
  `mmjj`, `mmll` …, lines 260-330 and 386-460). With `ptj = 0` these are 0.
- `cuts.f` applies the card's cuts and nothing hard-coded.
- `genps.f:1031-1033` draws a t-channel invariant flat in
  `[-tmax/stot, -tmin/stot]` with no floor.
- `tmin_for_channel` and `sde_strategy` (`genps.f:1817-1960`) only reshape the
  multichannel ratio `AMP2(channel)/XTOT`
  (`matrix_madevent_group_v4.inc:219-228`). That ratio sums to 1 over
  channels, so it does not change σ. At the card's defaults (`-1`, `1`),
  `get_channel_cut` returns 1 at its first line.

vibegraph applies the same floor: `MUF_FLOOR = 4.0` GeV²
(`coupling/cluster/setclscales.rs:43`), under the same condition at line 508.
`hadronic.rs::vetoes_point` turns `ScaleRefusal::FactorisationFloor` into a
zero-weight point. So **no implicit cut has to be added to vibegraph** to
compare with MadGraph. The comparison depends on vibegraph's clustering scale
matching MadGraph's per event near 2 GeV (see Found).

### vibegraph: the regulator scan

- **Setup.** Process `u c > e+ e- u c QCD=0 QED=4` (24 diagrams, 10 channels,
  all peripheral), `loop_sm-ckm_no_b_mass` at its restriction defaults. The
  DY MLL-50 run card with only `ptj` changed: `etaj = -1`,
  `ptl = drjj = drjl = 0`, `mmjj = 4`, `mmll = 50`, dynamical scale `-1`,
  NNPDF3.1 (lhaid 325300).
- **Budget.** `--fixed-budget --neval 1000000 --niter 15`, about 5 min per
  run on 2 threads.
- **Command.** `vibegraph --no-tui -v -y integrate proc_uc.dat --run-card
  run_card_ptj<P>.dat --ufo-dir T-V1-ufo --seed <S> -j 2 --fixed-budget
  --neval 1000000 --niter 15`. The binary is the pre-bump release-debug
  build. The phase-space code is the same as in `4fdc92b9`.

| ptj (GeV) | ln(1/ptj) | per-seed σ (pb), seeds 11/22/33/44/55 | mean ± sd/√5 (pb) | seed χ²/dof | mean quoted err |
|---|---|---|---|---|---|
| 3 | −1.10 | 0.10596 / 0.10537 / 0.10226 / 0.12282 / 0.10438 | 0.1082 ± 0.0037 | 4.34 | 0.0059 |
| 1 | 0.00 | 0.13948 / 0.15107 / 0.13587 / 0.14329 / 0.12719 | 0.1394 ± 0.0040 | 2.28 | 0.0071 |
| 0.3 | 1.20 | 0.14910 / 0.15455 / 0.13635 / 0.14135 / 0.12769 | 0.1418 ± 0.0047 | 6.27 | 0.0066 |
| 0.1 | 2.30 | 0.13691 / 0.12920 / 0.13839 / 0.14021 / 0.14368 | 0.1377 ± 0.0024 | 5.82 | 0.0035 |

Rung-to-rung slopes, in pb per unit ln(1/ptj):

| Step | Slope |
|---|---|
| 3 → 1 GeV | 0.028 ± 0.005 |
| 1 → 0.3 GeV | 0.002 ± 0.005 |
| 0.3 → 0.1 GeV | −0.004 ± 0.005 |

σ does not keep growing in the log. It rises while `ptj` is above 2 GeV and
is flat below. Pooling the 15 seeds with `ptj ≤ 1`, which all measure the same
quantity: mean 0.1396 pb, sd 0.0081, sd/√15 0.0021, χ²/dof 4.29.

**Budget caveat.** The brief asked for a budget at which the seeds agree. This
one does not reach it. Seed χ²/dof is 2–6 at every point, the quoted errors
understate the seed spread by about 2×, and the tails are heavy (seed 44 at
ptj 3 reads 0.1228 ± 0.020). The integrator warns that 6 of 10 channels
accept fewer than one point in 4. So the absolute σ here is not a converged
number. The regulator conclusion does not rest on it. It rests on the
per-event evidence below.

### Per-event evidence: the softest jet

`vibegraph generate --seed 7 --nevents 2000` was run on the seed-11 grids.
The spectrum is histogrammed by `T-V1-ptspec.py`.

dσ/d ln pT of the softest outgoing quark (pb):

| pT bin (GeV) | ptj = 1, dynamical μF | ptj = 0.1, dynamical μF | ptj = 1, fixed μF = 91.188 |
|---|---|---|---|
| 0.1–1 | — | 0 (0 events) | — |
| 1–1.78 | 0 (0 events) | 0 (0 events) | 0.371 (947 events) |
| 1.78–3.16 | 0.0766 | 0.0754 | 0.197 |
| 3.16–5.62 | 0.0757 | 0.0741 | 0.102 |
| 5.62–10 | 0.0478 | 0.0465 | 0.064 |
| 10–17.8 | 0.0264 | 0.0260 | 0.035 |

Minimum softest-jet pT over 2000 events:

| Run | Minimum softest-jet pT |
|---|---|
| ptj = 1, dynamical | 2.0006 GeV |
| ptj = 0.1, dynamical | 2.0045 GeV |
| ptj = 1, fixed scale | 1.0002 GeV (the card cut) |

- With the dynamical scale, the edge sits at 2 GeV whatever `ptj` is. The
  ptj = 0.1 and ptj = 1 spectra agree bin by bin.
- **Falsification control.** The fixed scale switches off exactly the floor:
  `fixed_ren_scale = fixed_fac_scale = True`, scale 91.188, else the ptj = 1
  card. The spectrum then reaches the card cut and rises toward it, and σ
  triples: 0.4510 ± 0.0283 and 0.4621 ± 0.0249 pb (seeds 11, 22), against
  0.139.
- So the collinear region T-F1 named is real, and is log-or-worse divergent.
  The rise is steeper than a plateau, which fits two quark lines each emitting
  a quasi-real photon. Only the 2 GeV μF veto removes it, and it removes about
  70% of the fixed-scale σ at ptj = 1.

### Per-diagram-class split

Not done. It is not cheap or clean. The artifact's per-channel σⱼ is the
single-diagram-enhancement partition of |M|², not a diagram class's
contribution. A physical photon vs W/Z t-channel split needs the photon
removed from the t-channel only, and `/ a` also removes the γ* → ℓℓ
s-channel.

### What this means for the ladder

- **The cards' σ is well defined**, as MadGraph defines it: the 2 GeV veto on
  the clustering-scale μF, applied identically on both sides. MadGraph's
  13.12 / 7.574 / 117.6 pb are therefore not obviously sampling artefacts.
- **The ladder can run once #82 lands.** But σ depends on how the event's
  clustering scale is computed near 2 GeV, which is a technical cut, not a
  physical one. A ladder mismatch should first be read as a clustering-scale
  mismatch near the floor, before a phase-space one.
- **The budget needs a convergence study first.** At 1e6 × 15 the seed
  spread is about 6% on one subprocess.

## Gates

On `4fdc92b9`, logs under the scratchpad (`T-V1-gate.log`,
`T-V1-test-ws.log`):

```
$ cargo fmt --all --check                                                    FMT_EXIT=0
$ cargo clippy -j 2 --workspace --all-targets -- -D warnings                 CLIPPY_EXIT=0
$ cargo clippy -j 2 --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
                                                                             CLIPPY_EXT_EXIT=0
$ python3 scripts/kb.py lint
999 concepts, 333 backlog items; 0 errors, 4 warnings                        KB_EXIT=0
$ cargo test -j 2 --workspace -- --test-threads 2                            TEST_EXIT=0
45 suites "test result: ok", 0 FAILED; 1710 passed, 0 failed
$ cargo test -p vibegraph --features extended-validation --test cli_generate --test cli_generate_proton -- channel_map_digest
test a_channel_map_digest_that_moved_is_refused ... ok
test a_hadronic_channel_map_digest_that_moved_is_refused ... ok
```

The four lint warnings are on backlog items this session did not touch.
`pixi` is not installed, so the banked `validate` layer was not run. The
change does not touch amplitudes or the coverage lists. It adds a field and
a refusal, and no map, draw or weight changes.

## Corrections to the brief

- **"Do not add a channel-map digest"** was superseded by the coordinator's
  scope change. The digest is in.
- **T-F1's hypothesis is falsified for the cards as written.** σ is
  regulated, by MadGraph's (and vibegraph's) μF ≥ 2 GeV veto under the
  dynamical scale, not by a cut. It is unregulated only with a fixed
  factorisation scale.
- **T-F1's Found** says the `u c` probe is finite "only because the default
  `|η_j| < 5`". Here `etaj = -1` and σ is still finite, for the reason above.

## Found

- **The CMS σ hinges on the clustering-scale floor.**
  - *What is wrong:* about 70% of the fixed-scale σ at ptj = 1 sits below the
    veto (0.45 → 0.14 pb), and the edge is at softest-jet pT = 2 GeV. A small
    per-event difference between vibegraph's clustering μF and MadGraph's near
    2 GeV moves σ by percents.
  - *To close:* pin vibegraph's per-event μF against MadGraph's own record,
    e.g. the clustering scales of a banked `p p > l+ l- j j` run with the
    dynamical scale. Then compare the fraction of points vetoed. Do this
    before reading any ladder residual as phase space.
- **The 1e6 × 15 budget does not converge `u c > e+ e- u c` with no cuts.**
  - *What is wrong:* seed χ²/dof is 2–6 (4.29 pooled over 15 seeds), quoted
    errors are about 2× below the spread, and 6 of 10 channels accept fewer
    than one point in 4.
  - *To close:* a budget-ladder study at 20+ seeds on two rungs (AGENTS.md),
    before the CMS ladder, with the per-channel acceptance logged.
- **The channel-map digest is blind to sampling-code changes.**
  - *What is wrong:* `map_key` encodes map parameters only.
  - *To close:* fold a hand-maintained map-algorithm version constant into the
    digest input, so a sampling-code change still needs one edit, in the
    phase-space code. Or keep relying on rule 3 and say so in the
    phase-space module docs.
- **`integrate-artifact.md` is stale beyond this change.**
  - *What is wrong:* its contents table lists `pdf_set`/`pdf_member`, where
    the field is `inputs`. Check 4 says no PDF checksum is recorded, which
    version 14 added. The `sampler` row says `None` means the writer predates
    the field.
  - *To close:* a concept pass against `artifact.rs` at version 15.
- **No cheap per-diagram-class σ split.**
  - *What is wrong:* the channel diagnostics give the SDE partition, not a
    diagram class's contribution.
  - *To close:* if the photon vs W/Z question matters for the ladder, add a
    diagram filter on t-channel propagator species at enumeration. A model
    restriction is too coarse.
