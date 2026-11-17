---
type: Session Report
title: "K-D1 report: σ 4× MadGraph's on a decay chain with repeated decays"
description: "Stitching kept every Feynman diagram twice when two identical decays sit on a vertex's interchangeable boson slots (h01 > a01 a01, a01 > b b~, a01 > b b~): |M|² exactly 4× at every point. Fixed by a slot-symmetric duplicate key, with failing-first tests, a census assertion of the decay-chain symmetry factor against MadGraph's, MadGraph standalone per-helicity amplitudes, and σ at two masses within 2σ of MadGraph."
status: draft
tags: [decay-chains, identical-particles, symmetry-factor, sigma, silent-wrong-answer, cms-gridpacks, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-17}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Worktree `/home/user/wt/sprintk-repeated-decay`, branch `sprintk-repeated-decay`,
based on `main` at `703f57f`, with `origin/main` (`c528465`, which holds #141 and
#149) merged in before the bit-for-bit step. MadGraph is the pinned 3.7.1
submodule, run with the `madgraph` pixi environment's Python. MadGraph outputs
are in `/home/user/wt/sprintk-repeated-decay-mg/`. The host has 4 cores, and this
session used 2 of them.

| commit | what |
|---|---|
| `3d9f7e8` | fix(chain): stitch a graph once when identical decays meet at interchangeable slots, with its tests and three census cards |
| `5ee3c28` | merge `origin/main` |
| (final) | this report; deletes the two closed backlog items |

## Model and cards

The genproductions card `SUSY_WH_WToAll_HToAATo4B_M-MASS` (from a blobless clone
of `cms-sw/genproductions`) imports `NMSSM --modelname` and lists `NMSSM.tar.gz`
in its extramodels. Its only customisation is `mass 25 125.0`, `mass 36 MASS`. The
CMS model store serves that tarball:
`https://cms-project-generators.web.cern.ch/cms-project-generators/NMSSM.tar.gz`,
sha256 `284ec1947ee7c42e485573e31d7a40f109b15050cc7d290f13ddac5021a005d7`. So the
repro needed no bundle from the host. The process is the card's:
`p p > w± h01, (h01 > a01 a01, a01 > b b~, a01 > b b~)`.

## The cause

Neither reading in the item holds:

- The `1/(2!·2!)` is applied. `outgoing_symmetry_factor` divides by the stitched
  final state's `Π n!`.
- Each `a01 > b b~` spec is assigned to one a01. `assignments()` gives a single
  assignment.

The defect is in `diagrams/chain.rs`, `stitch_set`. To close the stitched set
under identical-particle permutations, it relabels the final state with every
`block_permutation` and removes duplicate graphs by `Diagram::canonical`. That
form keeps the slot order within a vertex as part of a diagram's identity. Moving
both `b` and both `b̄` between the two `a01` decays rebuilds the same graph with
the two `a01` lines exchanged between the `V_719(a01,a01,h01)` vertex's two `a01`
slots. Its canonical form differs, so the graph was kept a second time. Every
Feynman diagram appeared twice, the amplitude was `2(A+B)`, and |M|² was exactly
4× at every phase-space point. No stitching, renumbering or census case had two
decays meeting at an interchangeable pair of slots, so nothing caught it. It
affects every chain of that shape: `h > z z` and `h > a a`-type vertices with two
identical decays to identical products.

### Evidence, per diagram × per helicity

A scratch probe (`#[ignore]`d, uncommitted) dumped the stitched census and each
diagram's per-helicity amplitudes, compiled alone, for
`h01 > a01 a01, a01 > b b~, a01 > b b~` under the NMSSM UFO:

```
SET h01 > b b~ b b~  (4 diagrams)
 diagram 0  v0 V_719(a01,a01,h01) rays [P0[1, 2], P1[3, 4], L0]   sign 1
 diagram 1  v0 V_719(a01,a01,h01) rays [P0[1, 4], P1[2, 3], L0]   sign -1
 diagram 2  v0 V_719(a01,a01,h01) rays [P0[2, 3], P1[1, 4], L0]   sign -1
 diagram 3  v0 V_719(a01,a01,h01) rays [P0[3, 4], P1[1, 2], L0]   sign 1
```

- Diagram 3 is diagram 0, and diagram 2 is diagram 1, each with the vertex's two
  `a01` lines swapped.
- At a fixed point, over all 16 helicities, `max(|A0−A3| + |A1−A2|) = 0.0`
  exactly.
- Colour-summed with `CF = [[9,3],[3,9]]` and divided by `2!·2!`:
  - all 4 diagrams give `1.999589218460841e-12`;
  - the 2 distinct diagrams give `4.998973046152103e-13`;
  - the ratio is `4.000000000000000`.
- The evaluator's own `eval_m2` at that point was 3.999999999971× the 4-diagram
  value, the identical-particle factor being applied downstream. So the 4
  diagrams are exactly what it summed.

### MadGraph standalone

`output standalone` of the same decay chain
(`/home/user/wt/sprintk-repeated-decay-mg/sa_haa`) has `NGRAPHS=1` and
`DATA IDEN/ 2/`: one pairing, divided by `identical_decay_chain_factor`. With
`matrix.f` patched to print `AMP(1)`, a driver evaluated it at the probe's point
under MadGraph's own `param_card.dat`, which both sides read:

```
helicities paired: 16 of 16
ratio vg diagram0 / MG AMP(1): re in [-1.2e-11, 1.0e-11], im in [0.99999999998, 1.00000000002]
max |vg_d0 - i*MG|/|MG| : 2.5e-11
MG SMATRIX sum = 5.104574724053188e-13;  9*sum|AMP|^2/IDEN(2) = 5.104574724053189e-13
```

The ratio is a uniform `i`, a benign global phase, at the 10 digits the probe
prints. After the fix the same point gives `eval_m2 = 1.999588220274463e-12`,
exactly a quarter of the `7.998352881097851e-12` before it. The stitched set
holds 2 diagrams, and the WH card gives 2 per subprocess (`u d~ > W+ b b~ b b~`,
…; 8 diagrams in 4 subprocesses, against 16 before).

## The fix

`stitch_set` now finds duplicates by `slot_symmetric_key`.

- At every vertex, the lines on slots of the same particle are put in a fixed
  order before `canonical()` runs, by the lowest-numbered external leg beyond each
  line. Same particle means the same `ParticleId` in the UFO interaction's slot
  list, which is read incoming, so the flow direction matches too.
- A tree's lines from one vertex reach disjoint sets of legs, so the order is
  fixed and the same for every arrangement.
- Slots of different particles are never reordered.
- Identical-fermion slots are left alone too. Exchanging two identical fermions
  moves them between the vertex's spinor pairings, so the result is a different
  diagram.
- The ambiguity check ("two lines either of which is the decay's resonance") uses
  the same key. The diagram kept is the first one reached, so a card without such
  duplicates keeps exactly its diagram list (see the bit-for-bit check below).
- Merged diagrams are collected only for tests (`generate_chain_with_merges`).

## Tests

Each new test was run on the fixed code and on the code with the duplicate key
reverted to `canonical()`.

`cargo test -p vibegraph-lib --lib helas::eval::stitching`, with the key
reverted:

```
test helas::eval::stitching::identical_decays_on_interchangeable_slots_are_stitched_once ... FAILED
  left: 4
 right: 2
test helas::eval::stitching::merged_stitched_diagrams_have_equal_amplitudes ... FAILED
  (the h > z z card: 0 merges up to slot order, at least 1 expected)
test helas::eval::stitching::stitched_chains_are_the_filtered_final_state ... FAILED
  e+ e- > z h, (h > z z, z > e+ e-, z > e+ e-) | e+ e- > Z e+ e- e+ e-: 4 stitched, 2 filtered;
  2 stitched not among the filtered, 0 filtered not stitched
test helas::eval::stitching::decays_on_slots_of_different_particles_are_all_kept ... ok
```

With the fix, all 6 pass. The merge test prints:

```
e+ e- > z z, z > e+ e-: 4 merges, 0 only up to identical-boson slot order, worst relative amplitude difference 0.0e0
e+ e- > z h, (h > z z, z > e+ e-, z > e+ e-): 2 merges, 2 only up to identical-boson slot order, worst relative amplitude difference 5.0e-17
e+ e- > z h, z > b b~, h > b b~: 0 merges, 0 only up to identical-boson slot order, worst relative amplitude difference 0.0e0
```

- **`identical_decays_on_interchangeable_slots_are_stitched_once`** checks that
  the h → zz card has 2× the core's diagrams.
- **`stitched_chains_are_the_filtered_final_state`** gained two cases:
  `e+ e- > z h, (h > z z, z > e+ e-, z > e+ e-)` and
  `e+ e- > z h, z > b b~, h > b b~`.
- **`merged_stitched_diagrams_have_equal_amplitudes`** checks that every diagram
  the key drops has the kept one's per-helicity, per-flow amplitudes at a fixed
  point (relative 1e-10). A merge of two different diagrams fails here instead of
  silently losing one.
  - Blind spot: a dropped diagram whose amplitude happens to equal the kept one's
    at that point.
- **`decays_on_slots_of_different_particles_are_all_kept`** checks that the
  identical `b b̄` decays on the `Z` and `h` slots of `Z Z h` keep all four
  arrangements. It passes on both codes. A spin-keyed rule would pass it too: the
  props' particles and the decay vertices differ, so the canonical forms differ
  anyway. It is a regression guard; the amplitude test is what would catch a
  wrong merge.

`cargo test -p vibegraph-lib --test decay_chain_census` now asserts the
symmetry factor per subprocess. This crate keeps every pairing and divides by
`Π n!`; MadGraph keeps the pairing on its blocks and divides by
`identical_particle_factor`. The test requires
`stitched · factor_MG = on_blocks · Π n!`. It also matches each decay block's
forced line to the particle MadGraph decays there (by `|pdg|`), since the
unordered block positions could not tell `Z → b b̄` from `h → b b̄`. Three cards
were banked: `ee_zh_h_zz_ee`, `ee_zh_h_zz_ee_mumu` and `ee_zh_bb_bb`. MadGraph
gives 1 diagram each, with factors 2, 1 and 1.

```
# key reverted
1 of 33 cards disagree with MadGraph:
  ee_zh_h_zz_ee ([-11, 11], [23, -11, 11, -11, 11]): MadGraph 1 diagrams, here 2 on its blocks (4 in all)
# fixed
ee_zh_h_zz_ee: ... [23, -11, 11, -11, 11]: 1 (+1 permuted), factor MadGraph 2 / here 4
33 cards: 30 stitched and matched (845 subprocesses, 107 of them reached by two decay assignments), 3 refused as MadGraph refuses or drops
test result: ok. 1 passed
```

The assertion holds on every banked subprocess (845). A graph stitched twice
doubles `stitched` with `on_blocks` unchanged when the duplicate falls outside
the blocks, or fails the diagram count when it falls on them, as here. It is
blind to a factor error shared by MadGraph's `identical_particle_factor` and the
final state's `Π n!`.

- The census dump regenerated the three new cases.
- In the same run, the existing case `cms_zto2q_zjj_jj` failed inside MadGraph
  (`name sm already consider as a python library cann't be reassigned`), once the
  default `sm` was already imported in that process.
- Its banked value (89 matrix elements) is kept unchanged, and only the new cases
  were added to the JSON.

## σ against MadGraph

Setup:

- Cards: the WH card at M-15 and M-40, both `w+` and `w-` lines, with the CMS run
  card.
- `lhaid` is 315200 (NNPDF31_lo_as_0130) on both sides instead of CMS's
  `$DEFAULT_PDF_SETS`, and `use_syst` is off. The reference must be local, and
  that PDF is the one fetched here.
- Param card: MadGraph's generated card with MH01 = 125 and MxA01 = M, read by
  both sides. The widths are the model defaults (WA01 = 2.495587e-4).
- MadGraph: `generate_events`, 10k events, `nb_core = 2`, seeds 101–105.
- vibegraph: `integrate` at the merged head, `-j 2`, default target, seeds 1–5.
- Drivers: `kd1-mg.sh`, `kd1-vg.sh`, `kd1-sigma.py` in
  `/home/user/wt/sprintk-repeated-decay-mg/`.

**Stated expectation.** MadGraph integrates `|A|²/2` inside pairing A's windows.
This crate integrates `|A+B|²/4` inside the union of both pairings' windows. The
difference is the pairing interference plus the union's extra region. A
multichannel Monte Carlo over the decay (200k points, Breit–Wigner sampling on
both pairings, per-helicity A and B from the evaluator) measured it. The CMS run
card cuts nothing on the b quarks, so the decay-level ratio is the σ ratio.

| Setting | Interference `I/(\|A\|²+\|B\|²)` | Windowed vg/MG − 1 (symmetric difference estimator) |
|---|---|---|
| M-15 | 4.5e-11 ± 3.1e-11 | 1.1e-11 ± 5.4e-12 |
| M-40 | 2.6e-9 ± 1.5e-9 | 1.8e-9 ± 1.4e-9 |
| Control, M-15 with Γ_a = 1.5 GeV | −1.08e-3 ± 1.7e-4 | +3.5e-3 ± 3.9e-4 |

So vg/MG = 1 to better than 1e-8 at both masses. The interference is suppressed
by roughly (Γ/M)², because a resonant A is nearly imaginary and an off-shell B
nearly real. The wide-width control shows the estimator sees the effect when it
is there. This is far below a 5-seed sweep's resolution.

```
$ python kd1-sigma.py
M-15:
  vg: n=5 mean=2.632700e-02 pb  seed sd=2.70e-05 (0.103%)  se=1.21e-05  mean quoted err=2.52e-05  chi2/dof=1.10
  MG: n=5 mean=2.631320e-02 pb  seed sd=1.73e-05 (0.066%)  se=7.75e-06  mean quoted err=2.30e-05  chi2/dof=0.55
  vg/MG = 1.00052 ± 0.00055; expectation 1 (to < 1e-8); pull +0.96
M-40:
  vg: n=5 mean=2.070480e-01 pb  seed sd=1.90e-04 (0.092%)  se=8.50e-05  mean quoted err=1.96e-04  chi2/dof=1.02
  MG: n=5 mean=2.068200e-01 pb  seed sd=1.81e-04 (0.087%)  se=8.08e-05  mean quoted err=1.84e-04  chi2/dof=0.96
  vg/MG = 1.00110 ± 0.00057; expectation 1 (to < 1e-8); pull +1.94
```

| | vg seeds 1–5 (fb) | MG seeds 101–105 (fb) |
|---|---|---|
| M-15 | 26.310, 26.292, 26.355, 26.352, 26.326 | 26.295, 26.315, 26.296, 26.328, 26.332 |
| M-40 | 206.76, 207.14, 207.14, 207.24, 206.96 | 206.68, 206.87, 206.62, 206.85, 207.08 |

- Both pulls are below 3, and the vg χ²/dof values are near 1.
- Both masses sit on the high side. Their 1/σ²-weighted mean is
  +0.08% ± 0.04% (2.0σ).
- Five seeds cannot calibrate a 0.1% offset (AGENTS.md). Before reading anything
  into it, a 20-seed comparison would be needed, along with a budget ladder for
  MadEvent, which has converged upward with budget on other decay-chain rows.
- The 0.08% is consistent with zero at that resolution. It is not the 4× defect
  and not the interference, which is below 1e-8.

**Before the fix, same card, same seed.** The main binary (`54ded74`) at M-15,
seed 1, gave `enumerated 16 diagrams in 4 subprocesses` and `σ = 105.240 ± 0.095
fb`. The branch at seed 1 gives `8 diagrams` and `σ = 26.310 ± 0.024 fb`. The
ratio is 4.0000: the same sampling with a uniformly 4× integrand. The host's
3.99–4.02 is the same factor seen through independent MC on both sides.

Only the WH card was run, at two masses. The item names the VBFH and TTH
`HToAATo4B` cards too. They share the `(h01 > a01 a01, a01 > b b~, a01 > b b~)`
decay, which is where the defect sat, but they were not run.

## Bit-for-bit on cards without repeated decays

Main `54ded74` was built in its own target dir, which was deleted afterwards.
`c528465` differs from it only in `research/kb`: `git diff --stat 54ded749
c5284657 -- '*.rs' '*.toml' '*.json'` is empty. Each card was run with
`integrate --seed 11 --neval 40000 --fixed-budget --niter 4 -j 2` on the banked
decay-chain run cards (`kd1-bitdiff.sh`):

```
main:
ee_zz_emu rc=0 σ = 945.2 ± 1.2 ab grid=8ca37cb41dbd95c0
ee_zz_ee rc=0 σ = 472.7 ± 1.0 ab grid=6e9aa5e6f7704f24
ee_ttx_nested rc=0 σ = 56.83 ± 0.13 fb grid=3aecce363eecde07
pp_ttx rc=0 σ = 6.396 ± 0.024 pb grid=143659e6780e43d2
branch: identical (diff reports IDENTICAL)
```

`ee_zz_ee` has identical decays at non-interchangeable slots. Its duplicates were
already found by `canonical`, so its diagram list and grid are unchanged.

## Gates

- On `3d9f7e8`: `cargo fmt --all --check` clean; `cargo clippy --workspace
  --all-targets -- -D warnings` rc 0; `cargo build` rc 0; `cargo test
  --workspace` rc 0 (61 suites, 2117 passed, 0 failed, 27 ignored).
- On the merge `5ee3c28`: clippy rc 0, `cargo build` rc 0, `cargo test --workspace` rc 0 (62 suites, 2120 passed, 0 failed, 26 ignored).
- `validate.sh` was not run locally; CI's `banked` job is that gate.

## What each oracle cannot see

- **Per-helicity standalone comparison:** blind to a global phase (here `i`) and
  to anything outside the one point and one param card.
- **The 4.000 arithmetic:** blind to a convention both copies of a diagram share.
- **Census assertion:** blind to a factor error MadGraph's
  `identical_particle_factor` and the final state's `Π n!` would share. It matches
  blocks by `|pdg|`, so it cannot tell a resonance from its antiparticle at one
  block.
- **σ sweep:** five seeds detect a missed region, not a 0.1% offset.
- **Interference prediction:** decay-level only, which equals the σ ratio here
  because the run card has no b cuts.

## Brief corrections

- Neither of the item's two readings is the cause; the cause is the duplicate
  removal in stitching.
- The model was reachable from the CMS model store.
- The comparison used an LO PDF (315200) on both sides, not CMS's NNLO default.
  The defect is combinatorial and PDF-independent.

## Found

- Every decay chain that puts two identical decays with identical products on a
  vertex's interchangeable boson slots was 4× too high: `h > z z`, `h > a a`,
  `h01 > a01 a01` with two identical decays. Chains with more such decays could
  have been off by more. The fix covers the class, and the census now banks one
  case of it.
- `dump_decay_chain_census.py` cannot regenerate `cms_zto2q_zjj_jj` in one full
  run. MadGraph raises `name sm already consider as a python library cann't be
  reassigned` once the default `sm` was imported earlier in the process. The
  banked value predates this and was kept. Closing it means running that card in
  its own process, or resetting MadGraph's model-module cache between cards.
- `e+ e- > z h, z > b b~, h > b b~` matches the filtered undecayed set only up to
  slot order (0 equal as containers, 4 up to slot order). Every amplitude
  matches, but stitched and enumerated diagrams can bind a vertex's `Z*`/`Z` slots
  in different orders.
- Both WH masses read vg/MG ≈ +0.05% and +0.11% (pulls 0.96 and 1.94). Settling
  whether this is MadEvent's budget convergence needs 20+ seeds per side and a
  MadEvent budget ladder.
- The VBFH and TTH `HToAATo4B` cards, and the remaining masses of all four cards,
  were not run here.
