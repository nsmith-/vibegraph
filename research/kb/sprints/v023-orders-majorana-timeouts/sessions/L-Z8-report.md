---
type: Session Report
title: "L-Z8 report: hygiene pass on multicore generate's blocked accept/reject"
description: "Findings, mutation evidence and the -j 1 / -j 4 LHE identity oracle against main for the review of the blocked accept/reject pass (#180)."
status: draft
tags: [hygiene, unweighting, generate, parallel, review]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-generate` off main 8b3af5ab. The change under review is
#180 (`5b1f9f71`, [L-S1](L-S1.md), [report](L-S1-report.md)):
`vibegraph-lib/src/unweight.rs`, the new `unweight/blocks.rs`,
`generate/source.rs`, `generate.rs`, the CLI tests `cli_generate.rs` and
`cli_generate_proton.rs`, the harnesses `validate_samples.rs`,
`validate_unweighting.rs` and `validate_sigma.rs`, and `pixi.toml`.

## Findings

Written and committed before any code edit. Ranked; **fix** marks the ones
this session fixes.

| # | file:line | class | what is wrong | evidence | proposed fix | disposition |
|---|---|---|---|---|---|---|
| F1 | `generate/source.rs:99,119,597,616`; `hadronic.rs:3607`; `proton.rs:4229`; `validate_samples.rs:800,806`; `validate_sigma.rs:2821,2829` | hand-kept list that should be derived | The uniforms an accepted point's labels take are counted three times by hand: `FIXED_BEAM_EVENT_DRAWS = 4` / `PROTON_EVENT_DRAWS = 5` (and a bare `4` in two harnesses), the `select_event(.., u: [f64; 4])` / `[f64; 5]` signatures, and the literal-index rebuild `[u[0], u[1], u[2], u[3]]` in four `prepare` closures. A count one short panics on a worker (`u[3]` out of bounds); one too long burns a draw silently. | `grep -n 'EVENT_DRAWS\|\[u\[0\], u\[1\]' vibegraph-lib/src/generate/source.rs vibegraph-lib/tests/validate_s*.rs` shows the constants and four rebuilds; `select_event` signatures quoted above | Make the count a const parameter of `BlockedPass<T, const D: usize>` and hand `prepare` a `[f64; D]`, so `select_event`'s own array type fixes `D`; the two constants and the index rebuilds go. | **fix** |
| F2 | `unweight/blocks.rs:141`; `unweight.rs:767`; `validate_samples.rs:842`; `validate_unweighting.rs:507` | dead or duplicated code | The sample cross section `W·Σw/T` is written four times: `Unweighter::sigma_from_events`, `BlockedPass::sigma_from_events` (re-implementing the first over other statistics, its own doc says so), and inline in two harnesses, whose copies divide by `used.trials` with no zero guard (`0/0 = NaN` on a seed that accepts nothing). | `grep -rn 'event_weight_sum /\|event_weight_sum, self' vibegraph-lib/src/unweight* vibegraph-lib/tests/validate_{samples,unweighting}.rs` lists the four: `blocks.rs:144`, `unweight.rs:768`, `validate_samples.rs:842`, `validate_unweighting.rs:507` | One `Unweighter` method over a given `UnweightStats`, with the shipped pass's association `(W·Σw)/T` so the generated σ line keeps its bits; the other three call it. | **fix** |
| F3 | `validate_samples.rs:2798` (`every_demoted_row_is_a_gated_row`) | vacuous or tautological test | The guard says a demotion list entry that "silently demotes nothing" fails it, but it checks only that the key names a row. A `KS_INFO_ROWS` row whose `mode` is `"gate"` demotes nothing (its KS columns run at `row.mode`), and a `SCALE_INFO_ROWS` row whose `mode` is `"info"` demotes nothing either; both pass the guard. | Mutation M8 below: `ee_to_wpwm` back to `mode: "gate"` passes the guard | Assert each `KS_INFO_ROWS` row is `info` and each `SCALE_INFO_ROWS` row is `gate`. | **fix** |
| F4 | `unweight/blocks.rs:542-543` (`a_pass_that_cannot_accept_gives_up`) | vacuous or tautological test | The second `next` after giving up is asserted `None`, which holds even with the `exhausted` latch deleted: the counter is already past the budget, so the next block also returns `None`, but it consumes a block and moves the statistics. The test cannot see that. | Mutation M7 below | Also assert the statistics are unchanged by the second call. | **fix** |
| F5 | `validate_samples.rs:138-139` | comment that breaks AGENTS.md | "against `3.6e-6` and `0` for the two rows that used to disagree": history framing, naming neither row; the readings are an old bank's (`research/kb/history/notes/25-validation-layering-plan.md:667`). The change rewrote this paragraph and kept the clause. | quoted lines | Drop the clause. | **fix** |
| F6 | `unweight.rs:775-779` | comment that breaks AGENTS.md (doc does not match body) | `Unweighter::trial`'s doc says "three uniforms' worth of the caller's stream are consumed per trial"; the body it now delegates to consumes `2 + channel_grid_ndim(j)`, as `trial_with`'s own doc says. | `sed -n 771,800p vibegraph-lib/src/unweight.rs` against `trial_with`'s doc | Say `trial` is `trial_with` on the pass's own scratch, statistics and per-channel trailing streams, and point at its draw count. | **fix** |
| F7 | `unweight/blocks.rs:129` | dead or duplicated code (public item with no user) | `BlockedPass::unweighter()` has no caller anywhere. | `grep -rn '\.unweighter()' vibegraph-lib vibegraph-cli` returns nothing | Delete. | **fix** |
| F8 | `unweight/blocks.rs:45` | public item that should be private | `pub const BLOCK_TRIALS` is read only inside `blocks.rs`. | `grep -rn BLOCK_TRIALS --include=*.rs . \| grep -v unweight/blocks.rs` returns nothing | Make it private. | **fix** |
| F9 | `unweight/blocks.rs:183-186,199-203` | (behaviour/doc mismatch, not a listed class) | `stats()` is documented as "every trial up to the one that produced the last point handed out", but the two give-up branches leave it differently: a too-late acceptance leaves the last event's statistics, an exhausted block leaves every consumed block's, past the last point. Neither is the serial pass's count of the `max_trials` failing trials. | quoted lines | Give the give-up a single defined statistic; it moves what a short-sample report prints, so it is not a hygiene edit. | **Found** |
| F10 | `unweight.rs:780-877` | dead or duplicated code | Production generates through `BlockedPass` only; the serial `Unweighter::trial` / `next_event` and the `stats` / `scale_draw` state they keep now serve only tests (`unweight.rs` tests, `hadronic.rs:5933`, `validate_lhef.rs:531`, `validate_samples.rs:2712`, `decay_chain_ladder.rs:208`). Two accept/reject passes are kept in step. | `grep -rn '\.trial(\|\.next_event(integ' --include=*.rs vibegraph-lib vibegraph-cli` | Move the tests onto `BlockedPass` (or a one-thread wrapper of it) and delete the serial pass; crosses six files and the census probe that compares the two passes on purpose. | **Found** |

Suspected in the brief and not reproduced:

- *`trial` and `trial_with` overlapping beyond the intended delegation*: `trial`
  is a twelve-line wrapper that moves the pass's scratch, statistics and
  trailing streams into `trial_with` and back; no logic is duplicated. Only its
  doc is stale (F6).
- *The block size 1024 as an unexplained literal*: `BLOCK_TRIALS` carries a
  seven-line doc giving the two bounds it sits between (`blocks.rs:37-45`). Its
  visibility is the finding (F8).

## Identity oracle

Chosen before the first edit. Main 8b3af5ab is built in a detached worktree
(`/home/user/wt/lz8-main`, own target) and this branch in its own, both
`cargo build --profile release-debug -p vibegraph --bin vibegraph`. Each card
is integrated once with main's binary; both binaries then generate from that
same grid at `-j 1` and `-j 4`, seed `20260728`:

- fixed beams: `e+ e- > mu+ mu-` at 45.6 GeV a beam, `--neval 20000 --niter 4`,
  1000 events;
- proton: `p p > l+ l- j QCD=2 QED=2` on the banked `pp_to_llj_fixed` run card
  and the NNPDF set, `--fixed-budget --neval 100000 --niter 6`, 3000 events.

Identity holds when all four LHE files of a card are `cmp`-equal to main's
`-j 1` file and the `sampling:`, `overweight:` and `from the sample` lines
`generate` prints are equal, timestamps stripped. The driver is
`oracle.sh integrate|compare` in the session scratchpad; its output is below.
