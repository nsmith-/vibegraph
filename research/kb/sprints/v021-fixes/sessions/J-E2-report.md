---
type: Session Report
title: "J-E2 report: fixed-energy sde_strategy 2 panics on two decay-chain subprocesses; the forests are now per subprocess"
description: "A fixed-energy sde_strategy = 2 card with two decay-chain lines that reach one ordered final state through different forced resonances panicked in ChannelSet::channel_cuts, because the forests were derived over the union of every subprocess's diagrams and indexed with the first subprocess's configuration count. use_running_coupling now takes one diagram list per subprocess and derives one ChannelSet each. Cards whose subprocesses share diagrams, and single-subprocess cards, give bit-identical sigma."
status: draft
tags: [phase-space, multichannel, sde-strategy, events, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-11}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-11-11, own CARGO_TARGET_DIR (the shared one ran stale binaries)"
  commit: "base 9a1349c; failing test b543a6d; fix on top"
  command: "see each oracle"
---
## Verdict

The path is reachable and the item's account was wrong in one respect.

- **Reachable.** Two decay-chain lines that force different resonances onto the
  same ordered final state clear every refusal before the integrand
  (`InconsistentExternals`, the spin-average check, enumeration's
  duplicate-subprocess check, which lets lines with disjoint decay assignments
  through). Reproducer: `e+ e- > z z, z > b b~, z > b b~` with
  `add process e+ e- > h z, h > b b~, z > b b~` at `sde_strategy = 2`.
- **Cause (corrects the item).** The item says forests were derived for
  subprocess 0 only. `use_running_coupling` derived one `ChannelSet` over the
  union of every subprocess's diagrams, paired with subprocess 0's evaluator.
  On the reproducer the union has 8 configurations and each subprocess 4, so
  `channel_cuts` asserts (`graph.rs:222`, left 4 right 8) for subprocess 0
  itself, not only for `k > 0`. Where the union's configuration groups equal a
  subprocess's (subprocesses with identical diagrams, such as the polarized
  pairs) nothing fails.
- **Fix.** `use_running_coupling` takes one diagram list per subprocess;
  `compile_configuration_weights` derives one `ChannelSet` each;
  `config_weights` is `Option<Vec<ChannelSet>>` and `split_term` and the
  event draw index it by subprocess. `hadronic::subprocess_diagrams(sets)`
  builds the argument in `compile_subprocesses` order; every caller uses it.

## Cards

All fixed-energy, `lpp = 0`, `ebeam = 500`; `integrate --fixed-budget`.

| Card | Result |
|---|---|
| `e+ e- > mu+ mu-` twice | refused at enumeration (duplicate subprocess) |
| `z{0} z{T}` plus `z{T} z{0}` | refused at enumeration (duplicate subprocess) |
| `w+ w-` chain plus `z z` chain to `u d~ d u~` / `u u~ d d~` (both orders), `e+ ve e- ve~` pair | refused, `InconsistentExternals` |
| `z{0} h` plus `z{T} h` | reaches the path; runs before and after, sigma bits identical |
| `w+{0} w-` plus `w+{T} w-` | reaches the path; runs before and after, sigma bits identical |
| c1 `z z` chain plus `h z` chain | panicked before (4 vs 8); after sigma 4.892 +- 0.019 fb (20000 x 4, seed 7); 2000 events generated |
| c2 `z z` chain plus `a z` chain | panicked before (4 vs 12); after sigma 3.936 +- 0.014 fb; 2000 events generated |
| c3 `h z` chain plus `a z` chain | panicked before (4 vs 12); after sigma 1.930 +- 0.029 fb; 2000 events generated |
| c1 at `sde_strategy = 1` | no panic before or after (no forests read); sigma 4.892 +- 0.019 fb, equal to the strategy-2 sigma as it must be |
| `c c~ g, g > b b~` plus `c c~ z, z > b b~`, `dynamical_scale_choice = -1` | sde 1: 8.501 +- 0.063 fb before and after; sde 2: panicked before, 8.501 +- 0.063 fb after |
| `g b b~, g > b b~` plus `z b b~, z > b b~` | refused (ambiguous decay resonance among identical particles) |
| `e+ e- > z g g, z > b b~` | no diagram |

## Oracles

- **Failing test, then passing.** `fixed_energy_sde_strategy_two_draws_each_subprocess_from_its_own_forest`
  (`hadronic.rs`) on c1: committed alone as b543a6d and failing with
  `graph.rs:222` left 4 right 8, in an own target dir; passing after the fix.
  After the fix it also asserts one `ChannelSet` per subprocess, each with its
  evaluator's `n_configs` configurations. It cannot detect a wrong forest of
  the right size, nor a wrong subprocess index among equal-sized sets.
- **sigma bit for bit, fixed seed 7, 4000 x 3, before (binary built at
  b543a6d) and after.** The artifact `sigma_pb` and `sigma_err_pb` bit patterns,
  read with `IntegrateArtifact::read_from_path`:
  - `z{0} h` plus `z{T} h`: `3f8a3dc40c35c9ac` both.
  - `w+{0} w-` plus `w+{T} w-`: `40058496e1aed5eb` both (channels identical).
  - `e+ e- > mu+ mu-`: `3fbab2b9ce4abb23` both; the artifact bytes are
    identical.
  These pin sigma and the sampler; they are blind to a configuration share
  moving between configurations. The shares did move, as the fix intends:
  in the polarized pair the second subprocess's 6.29% was banked as
  `no configuration` and is now `config 0`.
- **Scale source (hadronic.rs `compile_scale_source`).** Sound, so unchanged.
  It is given the union of the diagrams with subprocess 0's evaluator, and the
  union's configurations lead with subprocess 0's own, so the configuration
  index drawn from subprocess 0's `AMP2` names the same forest; the sampling
  channels are the union's and are checked against it
  (`assert_channels_match_the_scale_source`). The probe card above has a live
  draw (`scale_draw_ndim = 1`, `alpha_s_dependent = true`, a temporary test)
  and gives the same sigma before and after at strategy 1. One scale is
  resolved per point from subprocess 0's configuration before the subprocess
  is chosen; that is the integrand's existing design, not a wrong forest.
  Not tested: that the union's leading configurations equal subprocess 0's
  forests beyond the matching counts and sigma.

## Gates

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean (rc 0) after `clippy --fix` removed needless borrows; the earlier
  needless-borrow lints are fixed.
- `cargo test --workspace` (own target dir): 2068 passed, 0 failed, rc 0, run twice (before and after the clippy fixes).
- `python3 scripts/kb.py lint`: the 5 warnings predate this branch. Three errors say the generated indexes of `sprints/`, `sprints/v021-fixes/` and its `sessions/` are stale: those directories' plan files are on the plan branch, so `kb-index` was not run here (it would conflict on merge); run it after the merge.
- `pixi run --skip-deps validate`: not run (manager approval needed first).

## Found

- The item `sde-strategy-2-weights-only-first-fixed-energy-subprocess` should
  read: the forests were derived over the union of the subprocesses' diagrams
  with subprocess 0's evaluator, which panics (not only mis-draws) whenever the
  union has more configurations than a subprocess; reproducer c1. Closed by
  this change.
- The fixed-energy scale is drawn from subprocess 0's configurations for
  every point, whichever subprocess the event later belongs to. MadEvent draws
  it per subprocess. No card showed a difference in sigma; it is worth an item
  if a scale-sensitive decay-chain row is ever banked.
