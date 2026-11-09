---
type: Session Report
title: "I-R2 report: zero τ floor refusal, mxx_only_part_antipart, worker-panic exit"
description: "ZeroTauFloor refuses a card with no ŝ floor before any point (MadGraph integrates it behind an implicit √ŝ ≥ 2 GeV); mxx_only_part_antipart's keys join pdg_cut with light codes refused; a worker panic exits 101 at once."
status: draft
tags: [refusal, cuts, run-card, panic, rayon, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-08}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/7ca93e8d91519a9b8363bf45d5a568219389b64e", title: "7ca93e8d on sprinti-r2"}
---
Manager's spot checks:
- the commit's author and trailer;
- a rerun on 7ca93e8d of `validate_pdf_grid` (release-debug, extended-validation: 20 passed) and `cli_worker_panic` (2 passed);
- the session's MadGraph launch (the √ŝ figures in its log were not re-read).

The mutation results and the gate counts are the session's claims.

## 1. Zero τ floor (`zero-shat-floor-panics-with-nan-x`)

`ProtonIntegrand::build` returns `ProtonError::ZeroTauFloor` when `τ_min ≤ 0`, after `τ_min` is computed and before the τ mixture is built. It names the outgoing legs with no positive pT minimum (`Cuts::has_pt_floor`). On `p p > b b~` under `sm-no_b_mass` at `maxjetflavor 4`, `ptb 0`, `etab -1`:

```
ERROR failed to build the hadronic integrand: subprocess g g > b b~ has no lower bound on ŝ: the outgoing leg 3 (PDG 5), leg 4 (PDG -5) are massless and uncut (no pT minimum, and no ŝ, invariant-mass or HT cut bounds the final state), so τ reaches 0 and the cross section diverges; give them a pT cut
```

- **Test:** `proton::tests::a_card_with_no_shat_floor_is_refused_naming_its_uncut_legs`, with the card at `ptb 20` as a control.
- **Mutation:** with the check disabled, the test fails, and the CLI falls back to "every one of the 8323 points … zero weight … the cuts reject every point".

**Brief corrections:**
- On d87ad3a the card no longer panics with `x = NaN`. Sprint H's degenerate-point guard zeroes every point, and the run ends in a misleading cut-rejection refusal.
- MadGraph does not refuse the card. A local launch (`/home/user/ir2-mg`) prints `Cross-section : 4.227e+09 +- 2.402e+07 pb`. Its logs show `Define smin to 0.0`, `Warning: No cutoff for shat integral found` (myamp.f:527-530), the 1/s transform at 4/stot (the `dynscale_cut` branch, myamp.f:546-551) and `Too low fact scales` (reweight.f:1210). The events have √ŝ ≥ 2.002 GeV (median 6.4 GeV), so the number is set by an implicit regulator, not by the cuts.

## 2. `mxx_only_part_antipart`

The manager approved the refusal reading. `PdgPtMin::from_card` builds MadGraph's `pdg_cut` as `banner.py:4708-4750` does:
- Every code `mxx_only_part_antipart` lists joins at `pT ≥ 0`.
- Both fields share the refusals: light codes (`PDG_CUT_LIGHT`), negative codes and 0, more than 25 codes, values that are not booleans, and `banner.py:4749`'s missing-`default` exception.
- The `setcuts.f:681` overcount counts the union of both fields.
- A per-PDG entry at `pT ≤ 0` compiles to no single-leg cut, as `setcuts.f:318-334` replaces the class cuts.
- The `classes.rs` row becomes `Consumed`.

**Why no σ moves:** every code that carries a class cut (j/b/l/a) is light and refused. Top at `maxjetflavor 6` is heavy, with `do_cuts` false. Every banked card writes `{'default': False}` and `pt_min_pdg {}`.

**Tests** (`cuts.rs`, four): the default lists nothing; light codes are refused through both entry points; `{23: True}` compiles equal to the default card; the shared refusals. **Mutation** (an empty mxx input): 1 passed, 3 failed.

## 3. Worker panic

**Cause:** `adapt_blocks_iteration` is one `par_iter`, a recursive `join` tree. In rayon-core 1.13.0 (`join/mod.rs:174-186`), a panicking side waits for its sibling to complete before it unwinds, and rayon has no cancellation. So the rest of the iteration runs after the message.

**Fix:** `exit_on_panic()` is installed first in `main`. It chains the previous hook and then calls `process::exit(101)`. The TUI hook wraps it.

**Test hook:** a debug-only fault point, `VIBEGRAPH_FAULT_WORKER_PANIC=<ms>` (`vegas.rs`), compiled out of release builds. `cli_worker_panic.rs` checks text and ldjson modes: exit 101, the message present, and an exit within 10 s. **Mutation** (the hook not installed): "the run outlived its worker's panic by 32.2 s", 0 passed and 2 failed.

## Gates (7ca93e8d)

- **fmt and clippy:** `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean.
- **Workspace tests:** `cargo test --workspace --no-fail-fast` gives 1993 passed, 0 failed, 25 ignored.
- **`validate.sh`:** exit 101, with 2432 passed and 13 failed. All 13 failures are in `validate_pdf_grid`, where NNPDF31_lo_as_0130 was absent (a manager provisioning error). The collator reported 213 ✅, 5 ⚠️ (the standing informational cells) and 14 ⏳, with the measured cells equal to the declared cells and no ❌.
- **Rerun:** `validate_pdf_grid` alone, after the set was fetched, passed 20 of 20.

## Found

1. `run-card/madgraph-cut-conventions.md:145` and `run-card/field-classification.md` (~:161) still call `mxx_only_part_antipart` benign.
2. The degenerate-point guard can present a structural integrand error as "the cuts reject every point". Look for other such paths.
3. In ldjson mode a panic now exits promptly but still emits no final object (`ldjson-stream-gaps`). The `exit_on_panic` hook is the place to fix that.
4. Worktree seeding omitted NNPDF31_lo_as_0130.
5. A madgraph-defects row: MadGraph integrates a card with no ŝ floor behind an implicit √ŝ ≳ 2 GeV regulator.
