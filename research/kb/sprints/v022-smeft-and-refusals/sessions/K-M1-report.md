---
type: Session Report
title: "K-M1 report: the MLM reweighting panic on g2HDM charged Higgs"
description: "The panic is MadEvent-unreachable: 97 flavour groups whose matrix element vanishes at the card joined crossings of different topology on 0 = 0, and rewgt read a member's gluon onto the representative's s-channel quark. Vanishing groups are now flagged from the couplings and not clustered; any remaining sampled-point refusal is a named error instead of a panic."
status: draft
tags: [mlm, panic, clustering, rewgt, flavour-groups, madgraph-parity, report, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-18}
---
Worktree `/home/user/wt/sprintk-mlm-panic`, branch `sprintk-mlm-panic`, based
on `main` at `0340a89`. Repro bundle `mlm-panic-repro` (M300), PDF set
NNPDF31_nnlo_as_0118_mc_hessian_pdfas (325300) fetched from
lhapdfsets.web.cern.ch. MadGraph is the local 3.5.7 (`madgraph` pixi env of
the main checkout), outputs under `/home/user/wt/sprintk-mlm-panic-mg/`.

| commit | what |
|---|---|
| `41b63bd` | fix(proton): never cluster a flavour group whose matrix element vanishes; sampled-point refusals become a named fault (tests included) |
| (this commit) | kb: K-M1 report; the closed item's file deleted |

## Which reading held: MadEvent never reaches the merge

**Reproduction.** The repro command on the base (dev build, with a temporary
dump at the panic site) panicked about three minutes in, in the α-survey, on
the 7-leg part (`p p > h+ j`):

```
KM1DUMP gi=0 member=16 flavours=[2, -1, -11, 12, 5, -5, 21] x=[0.47901160277197474, 0.03236108237201791]
merges: [4,8]->12, [12,16]->28, [28,32]->60, [60,64]->124, core [1,2]->124
pt2: [6458.719413966847, 25449.283299099116, 90077.07814681061, 659836.7943480223, 2867126.724517016]
pdg (walk): 1:21  2:2  60:37  64:1  124:2
thread '<unnamed>' panicked at vibegraph-lib/src/proton.rs:3538:13:
matched reweighting on a sampled point: the clustering's merge 4 joins lines [60, 64] ...
```

Mask 60 is the whole h+ (legs 3–6: `e+ ve b b~`) and 64 is leg 7. Group 0's
representative is `g u > h+ d`, whose history has the s-channel `u → h+ d`
(merge 4, lines `[60, 64] → 124`, codes `37, 1 → 2`). Member 16 is
`u d~ > h+ g`. `rewgt` writes the member's jet code onto leg 7. That gives
`u → h+ g`, colours `3 → 1, 8`, which no arm of `ipartupdate` names.

**The host's suspicion is falsified.** It was a decay-product b joined to a
radiated parton. In fact the merge joins the *complete* charged Higgs to the
jet, and the panicking member has no b among its beams or jets. The
five-flavour `p` plays no part.

**Why the two share a group.** With `gur3x1 = ρ_tu = 0`, every `ρ^d = 0` and
a Cabibbo-only CKM (`V_td = V_ts = 0`, `V_tb = 1`), h± couples only to `c b̄`
and `t b̄`. Every subprocess whose h± attaches to a `u` line, or to `c` with
`d`/`s`, has `|M|² ≡ 0` at this card. `derive_flavor_groups` groups by `|M|²`
agreement at probe points, and `0 = 0` there. The colour pairing (one `T^a`
flow each) also agrees, so `g u > h+ d` and `u d~ > h+ g` join one group
although their diagrams differ.

**MadGraph evidence.** `generate p p > h+ j QCD=99, (h+ > t b~, (t > w+ b,
W+ > l+ vl))` with the bundled model and `define p = p b b~`, then `output`:

- MadGraph puts the two in different P-dirs. `P1_gq_hpq_hp_tbx_t_wpb_wp_lvl`
  holds `g u > … d` (iproc 1). `P1_qq_hpg_hp_tbx_t_wpb_wp_lvl` holds
  `u d~ > … g` (iproc 1). Both are generated, with zero couplings, as here.
- `configs.inc`: in P1_gq_hpq, config 1 has `IFOREST(-4) = (7,-3)` with
  `SPROP = 2` (u). That is the s-channel line {3..7}. P1_qq_hpg's two configs
  are t-channel only, `IFOREST(-4) = (1,7)` and `(1,-3)`. So `findmt` never
  admits the (60, 64) pair there.
- A replay driver (`/home/user/wt/sprintk-mlm-panic-mg/drv/km1drv_main.f`)
  runs in each P-dir. It calls `setrun`, `setpara`, `setcuts`, then per
  config `initcluster`, `cut_bw` and `setclscales(p, .false.)` twice: the first
  call within the config, the second over all graphs. It is fed the captured
  point (`drv/point.in`). Output, second calls:

```
P1_gq_hpq  iproc 1 config 1:  setclscales T
  merge 1   4   8 ->  12  pdg -11 12 24  pt2   6458.7194139668472
  merge 2  12  16 ->  28  pdg  24  5  6  pt2  25449.283299099116
  merge 3  28  32 ->  60  pdg   6 -5 37  pt2  90077.078146810614
  merge 4  60  64 -> 124  pdg  37  1  2  pt2 659836.79434802232
  merge 5   1   2 -> 124  pdg  21  2  2
P1_qq_hpg  iproc 1 config 1 (and 2):  setclscales T
  merges 1-3 as above
  merge 4   1  64 ->  65  pdg   2 21  2  pt2 659836.13451188791
  merge 5  65   2 ->  60  pdg   2  1 37
```

The representative's history is MadEvent's for `g u > h+ d`, merge for merge
and digit for digit in `pt2ijcl`. On the same point MadEvent clusters
`u d~ > h+ g` with the gluon onto beam 1. Neither `setclscales` stops. The
core's `pt2ijcl` (8315 in the driver against 2.87e6, which is `μR²`, in the dump) is not compared.
The dump holds the history `rewgt` reads after setclscales has rewritten the core. The driver
prints `setclscales(p, .false.)`'s state, which has not been through that rewrite in the same way;
that difference was not traced further, since the core does not enter the failing merge.

The local 3.5.7's `checkbw` reads `OnBW` from the phase-space generator. The
pinned template's (which vibegraph ports) calls `cut_bw(p)` itself. Without
calling `cut_bw` in the driver, the W, t and h+ merges came out unflagged,
with kT measures (758.9, 130820, 33928) in place of masses. The order was
unchanged.

## The fix

1. **Vanishing groups are flagged from the couplings** (`vanishes_at_card`,
   `FlavorGroup::vanishes`). A subprocess vanishes when every diagram passes
   through a vertex whose every coupling (`bound_coupling` per term)
   evaluates to exactly `0` at the bound card. A group is flagged when every
   member is. Nothing sampled is read: no `|M|²` at a sampled point, and not
   the probe traces. A debug assertion checks that a flagged group's probe
   trace is zero.
2. **A flagged group is neither clustered nor reweighted.** `per_group_sum`
   treats it like a group whose cuts reject the point. MadEvent clusters each
   P-dir on its own diagrams and multiplies by `|M|² = 0`, so σ, events and
   every record of a written event are the same either way.
3. **No sampled point can panic** on this path. `member_rewgt` returns the
   `RewgtError`. The two `per-event scale on a sampled point` sites (`scales_at`,
   `term_at`) stop panicking. Each keeps the first refusal as a
   `SampledPointFault` (`ProtonIntegrand::fault`, `MultiplicitySum::fault`), and
   the point (or the group's factors) gets weight zero. `integrate` refuses
   after the α-survey and after the integration. `generate` refuses after the
   scan and after emission. The message names the member by PDG codes, the
   merge and its lines, for example `matched reweighting of PDG 2 -3 > 24 21 on a
   sampled point: the clustering's merge 1 joins lines [4, 8] with a colour
   structure reweight.f's ipartupdate does not name, where MadEvent stops the
   run; the run is refused`.
4. An info line counts the vanishing groups per decomposition. A debug line
   lists their members.

### Why skip rather than cluster each member on its own diagrams

MadEvent clusters each subprocess on its own diagrams. Inside a non-vanishing
group that is what vibegraph already does. Equal non-zero `|M|²` with a shared
colour pairing means the members are relabellings of one diagram set
(generation copies, charge conjugates), and the representative's history with
the member's jet codes is the member's own. The two options differ only on
vanishing groups. There, MadEvent's own-diagram clustering feeds a factor that
multiplies `|M|² = 0`. Per-member clustering would cost one clustering per
member and point, where today's cost is one per group, and would change no
recorded number. The one thing it would reproduce that skipping does not is a
MadEvent *stop* on a zero-weight subprocess's own topology. This card has
none: `P1_qq_hpg` clusters the point cleanly.

### What the flag reads, and what it does not

- A coupling that is zero because a **param-card value** is zero (`ρ_tu = 0`
  here, or `λ = 0` in the test) is flagged. The flag reads the bound card, not
  the restriction. A coupling removed **by the restriction** never produces a
  diagram, so there is nothing to flag. Both are exact zeros of the coupling
  at the card the run evaluates, and both are flagged alike.
- A coupling whose expression cancels to an exact `0.0` at the card is
  flagged; that is correct. A tiny non-zero coupling is not.
- A matrix element that vanishes only through **interference between diagrams**
  is not flagged. Such a group is still clustered. If it reaches an unnamed
  vertex, it now ends in the named refusal rather than a panic.

### Census: 97 groups, 872 subprocesses, the same at every mass

The `--check` info lines on M300 come per decomposition (`p p > h±` /
`h± j` / `h± j j`, split by outgoing masses). Totalled: 97 vanishing flavour
groups, 872 subprocesses. The same 97 / 872 at M450, M900, M1000, M1250,
M1500 and M1750, since the zero is in the couplings, not the masses. Before
the fix, the 97 groups whose probe trace was exactly `0` were these same
groups, counted with a temporary dump. Every zero-trace group is therefore
structurally zero, and no non-zero group is flagged. Which subprocesses:
those whose every h± vertex is `u`–`d_j` (`ρ_tu = 0`, `ρ^d = 0`) or
`c`–`d`/`c`–`s` (`V_td = V_ts = 0` under the Cabibbo-only CKM). For example
`u d~ > h+ g`, `g u > h+ d`, `g c > h+ s`, `u b~ > h+ g`, `g g > h+ d u~`,
`u u > h+ u d`, and their charge conjugates. Merging `e`/`μ` copies leaves 436
distinct processes, 218 with h+ and their 218 charge conjugates. Of the h+
ones, 5 are `2 → 1` (`u d~`, `u s~`, `u b~`, `c d~`, `c s~ > h+`), 15 have
one jet and 198 have two. The full list (`-vv`, one debug line per group) is
in `/home/user/wt/km1-census-m300.log`.

## Oracles

Every command below ran on this host. Paths under `/home/user/wt/` are
session scratch, not committed.

**1. Reproduction on the base.** Dev build of `b1673e9` (= `0340a89` plus the
brief), temporary dump at the panic site:

```
$ LHAPDF_DATA_PATH=/home/user/inputs/pdf vibegraph integrate proc_card.dat \
    --run-card run_card.dat --param-card param_card.dat --ufo-dir models \
    --neval 120000 --target-rel 0.01 --seed 1 -j 4 --no-tui
...
thread '<unnamed>' panicked at vibegraph-lib/src/proton.rs:3538:13:
matched reweighting on a sampled point: the clustering's merge 4 joins lines
[60, 64] with a colour structure reweight.f's ipartupdate does not name, where
MadEvent stops the run
```

The dump also showed every one of the four panicking worker points in group 0,
member 16.

**2. MadEvent clustering of the captured point.** See the section above. Driver
`/home/user/wt/sprintk-mlm-panic-mg/drv/km1drv_main.f`, built per P-dir by
`drv/build.sh`, run as `ulimit -s unlimited; ./km1drv < drv/point.in`.
Outputs are in `drv/P1_gq_hpq_hp_tbx_t_wpb_wp_lvl.out` and
`drv/P1_qq_hpg_hp_tbx_t_wpb_wp_lvl.out` (`mlevel = 22` in the scratch copy's
`message.inc`). The result: the representative's merge sequence equals
MadEvent's for `g u > h+ d` to the printed digit, and MadEvent clusters
`u d~ > h+ g` differently and without a stop.

**3. Tests (failing first).** In `vibegraph-lib/src/proton.rs`:

- `a_group_joined_on_a_vanishing_matrix_element_is_not_clustered`. SM `ckm`
  restriction, `Block Wolfenstein 1 0.0`, matched `p p > w+ j` (ickkw 1,
  xqcut 20). It asserts that `g u > w+ s` and `u s~ > w+ g` share a group, that
  it alone is flagged, that over 200 sampled events it is never scaled or
  reweighted, that other groups are, and that no fault is recorded. Clustered
  on purpose, the same group reaches the vertex: the fault records
  `PDG 2 -3 > 24 21` with `UnnamedColourStructure`, and the factors are zero.
  Run with the skip disabled, the first half fails:
  ```
  assertion `left == right` failed
    left: Some(Rewgt { member: "PDG 2 -3 > 24 21", error: UnnamedColourStructure { n: 1, daughters: [4, 8] } })
   right: None
  ```
  On `0340a89` that refusal is the panic.
- `a_subprocess_with_one_live_diagram_does_not_vanish`. `u u~ > w+ w-` at
  `λ = 0` has some diagrams with an all-zero vertex and some without, and is
  not flagged. `u s~ > w+ g` is flagged. At the restriction's own `λ` it is not.

Gates on `41b63bd`, `-j 2` (`CARGO_BUILD_JOBS=2`, lean variables):

```
$ cargo fmt --all --check                                   rc=0
$ cargo clippy --workspace --all-targets -- -D warnings     rc=0
$ cargo test --workspace --no-fail-fast                     rc=0
  68 test binaries: 2193 passed, 0 failed, 26 ignored
  vibegraph-lib unit tests: 1391 passed; 0 failed; 15 ignored
```

The banked layer (`validate.sh`) was not run locally, as the brief asks. CI's
`banked` job is the gate.

**4. Bit for bit on banked matched cards.** `0340a89` built from
`git archive` in a separate target dir (deleted afterwards) against this
branch. Same cards (`validation/madgraph/output/<row>/Cards/run_card.dat`,
proc card from the row's script), `--fixed-budget --neval 20000 --niter 4
--seed 7 -j 2`, `LHAPDF_DATA_PATH=validation/pdf`:

| row | main grid sha256 | branch grid sha256 | σ main / branch |
|---|---|---|---|
| `pp_to_llj_mlm` | `d4a0be7c3f4cd1c806453b1ca75e01af5cb3b751c946273f4725cb96821d9555` | same | 268.2 ± 1.8 pb / same |
| `pp_to_ttx_0j1j_mlm` | `8d613b5a09de264171e49e76a481528de9023f4a1153dbb355a137bb2659af05` | same | 1.0934 ± 0.0046 nb / same |

Both runs exit 0, so the new fault refusal never fired on either card.

**5. The seven masses.** M450–M1750 cards are the bundled M300 card with `MASS 35`,
`36` and `37` set to the mass. Widths are left at M300's values, since the
bundle has no other cards. Widths enter neither the couplings nor the flag.
`integrate --check` passes at every mass. Each reports 97 vanishing groups and
872 subprocesses, in 13 decompositions, the same split as M300.
(M1750's first check ran in the wrong directory, because its card directory
had not been created. Rerun with the release binary: 97 / 872, `check ok`.)

Then, per mass, `integrate --neval 10000 --fixed-budget --niter 1 --seed 1
-j 2`, killed once the main integration announced itself. The survey's fault
check runs before that line, so a fault would have refused first:

```
M450  survey_completed=1 panics=0 surveys=13 refusal=   (21:52:59 → 22:03:23)
M900  survey_completed=1 panics=0 surveys=13 refusal=   (22:03:38 → 22:12:34)
M1000 survey_completed=1 panics=0 surveys=13 refusal=   (22:12:34 → 22:21:45)
M1250 survey_completed=1 panics=0 surveys=13 refusal=   (22:21:45 → 22:30:40)
M1500 survey_completed=1 panics=0 surveys=13 refusal=   (22:30:40 → 22:39:21)
M1750 survey_completed=1 panics=0 surveys=13 refusal=   (22:42:47 → 22:51:23)
```

What each exercised: all 13 decompositions' α-surveys, 6 × 10 000 points
each. Every point is clustered and `rewgt`-ed on every non-vanishing group
that passes cuts. The 97 vanishing groups are skipped by construction. These
runs do not show that the base would have panicked at this reduced budget;
the base was only run at M300 with the full survey.

**6. M300, seed 1, release build, `-j 2`: timing only, no σ.** The repro
command (`--neval 120000 --target-rel 0.01`) ran from 20:22:19:

- The α-survey (13 parts × 6 × 40 000) took 30 min with no fault. The base
  panicked inside this same survey.
- VEGAS has 2176 channels, of which 2142 sit at the 512-point floor.
- Iteration 1 had 1 172 281 points at 809.9 µs/eval and finished at 21:07:57.
- Iteration 2 had 2 680 158 points at 920.4 µs/eval and finished at 21:49:04.
- The run was killed by PID at the 90-min cap (SIGTERM twice, rc 143).

With `--min-iters 6`, one seed is about 30 + 16 + 5 × 41 ≈ 4.2 h at `-j 2`.
Five seeds would be about 21 h here, so no sweep was run.

**σ oracle, open.** M300 σ against MadGraph's 10.04 ± 0.039 pb (the gridpack's
own run), at five seeds with unweighted mean, spread and χ²/dof, using this
branch's binary on the CERN host's batch-proxy. Both use lhaid 325300 and
`ebeam = 6800`. The gridpack's MadGraph version is not in the bundle. This
oracle checks the card's matched pipeline end to end. It does not check the
fix, because the flagged groups contributed exactly `0` before the fix too.

## What each test cannot detect

- `a_group_joined_on_a_vanishing_matrix_element_is_not_clustered` uses SM
  `p p > w+ j` at `λ = 0`, a stand-in for the g2HDM card, which needs a model
  outside the repository. It pins the mechanism (same vertex class:
  colourless boson + gluon into an s-channel quark), not the g2HDM run.
- `a_subprocess_with_one_live_diagram_does_not_vanish` pins the per-diagram
  rule on one mixed process. It cannot see a matrix element that is zero by
  interference, which the flag deliberately ignores.
- The MadGraph replay checks one point and two P-dirs (`p p > h+ j`). It is
  blind to the 8-leg (`h± j j`) topologies, and to the BW-flag carry-over
  between events that `cluster.f` keeps in a common block (the driver
  recomputes per call).
- The bit-for-bit runs are two cards at a small fixed budget. They show no
  change where no group vanishes; they do not exercise the fault path.
- σ against MadGraph checks the matched pipeline on this card, **not the fix**.
  Flagged groups contributed exactly `0` before the fix as well.
- "Fails before" for the new test is shown on the branch with the skip
  disabled, not on `0340a89`, where the test does not compile (no
  `vanishes()`). There it records `UnnamedColourStructure { n: 1, daughters:
  [4, 8] }` for `PDG 2 -3 > 24 21`, the exact analogue of the g2HDM refusal.

## Corrections to the brief

- The suspected mechanism (decay b + radiated parton under the five-flavour
  `p`) is not it; see above.
- The local MadGraph 3.5.7 is not the pinned template. Its `checkbw` does not
  call `cut_bw`, and its `cluster`/`setclscales` take no `ivec`. A replay against
  3.5.7 has to call `cut_bw` itself to reproduce the pinned template's on-shell
  tagging, which is what vibegraph ports.
- The gridpack's MadGraph version is not recorded in the bundle. Its run card
  lacks `custom_fcts` (3.5.7 warns that it takes the default), so it predates
  3.5. lhaid 325300 and `ebeam = 6800` match our runs.

## Found

- **Vanishing subprocesses still cost a run its budget.** The 97 groups are
  still compiled, probed, channel-mapped, surveyed and sampled. At M300 three
  whole decompositions are vanishing (1, 2 and 167 channels, survey mean
  exactly `0`), and each takes 1/13 of the α-survey. So 23 % of a 30-min
  survey goes to them, and at least 170 of the 2176 channels sit at the
  512-point floor every iteration (≥ 87 040 of 1 172 281 points, ≥ 7.4 %).
  The rest of the vanishing groups share decompositions with live ones, and
  their own channels were not counted here, so the true saving is larger.
  Dropping vanishing groups, and decompositions left empty, before the maps
  are built would close it. The grids of these cards would change.
- **Grouping on `0 = 0`.** `derive_flavor_groups` lets identically zero
  subprocesses of different topology share a group. They are harmless now,
  since a flagged group is never clustered, but any other per-group use of the
  representative's diagrams (configurations, resonance tables, event labels)
  carries the same assumption. Dropping them, as in the item above, would
  remove the class.
- **The fault refuses late.** A `SampledPointFault` voids the run, but
  integrate notices only after the survey or after the whole integration, and
  generate only after the scan or after emission. Raising the run's
  `StopSignal` from the integrand would end it at the next iteration
  boundary. In generate, the event file is already renamed into place when
  the refusal comes.
- **The fixed-beam path still panics on a sampled-point scale error**
  (`hadronic.rs:2746`, `per-event scale on a sampled point`). It is not
  reachable from the proton path fixed here.
- **Weights to another param card cannot recover a vanishing subprocess**
  (the same as MadGraph). A run at `ρ_tu = 0` has no events from `u`-line
  subprocesses, so reweighting it to `ρ_tu ≠ 0` misses them. This is worth a
  warning when the reweight card moves a parameter that un-vanishes a group.
- **The local MadGraph 3.5.7 is not the pinned template.** Its
  `checkbw`/`cluster`/`setclscales` differ (no `cut_bw` call, no `ivec`), so a
  replay against the local install needs the driver to call `cut_bw`. A
  replay driver against the pinned template, beside
  `gen_kt_cluster_dumps.sh`, would make single-point MadEvent clustering
  checks routine.
