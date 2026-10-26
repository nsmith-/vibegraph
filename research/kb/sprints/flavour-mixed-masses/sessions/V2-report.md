---
type: Session Report
title: "V2 report: the mixed-mass Drell-Yan row gated on the M3's measurement"
description: "P1_qq_ll classified as the massless part, the known-wrong massless-tau control printed and asserted beside the three readings, the integrals cell flipped to gate. The record exists only on the M3, so the row stays planned and the measurement is not reproduced here."
verified: [{by: claude-code/claude-sonnet-5-5, at: 2026-10-23}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-23}
---
Worktree `/home/user/wt/L`, branch `sprintd-flavour-mixed-masses`, from
`68595ac`. Nothing in `vegas.rs`, `budget.rs` or the phase-space maps was
touched. The M3's readings (brief) are quoted, not reproduced: this checkout
has no seed record.

## 1. What changed

| file | lines | change |
|---|---|---|
| `vibegraph-lib/tests/validate_hadronic.rs` | 641-676 | `lepton_part_of` accepts `ll` (and `epem`/`emep`) as the massless part, `taptam`/`tamtap` as the tau part, panics otherwise; two hermetic tests |
| | 743-751 | structure assertion: the two parts carry the same group count, member counts, diagram counts and incoming legs |
| | 876-955 | the three cells now gate (`status` pass/fail, `failures` asserted at the end); the control `ta_part_on_massless_channels` is printed beside them, not written as a cell, and asserted outside the bounds |
| `validation/manifest.toml` | 531 | `integrals` `mode = "gate"`, note rewritten as measured; `status = "planned"` kept |

**Classification.** The seed record (`SeedRun`, `ChannelValues`) holds channel
keys `P<N>_<initial>_<final>/G<tag>` and file names only: no process line, no
PDG list per directory. So the label is the identification, documented at
`lepton_part_of`: MadGraph groups on every leg's mass, so a directory's final
token names its mass; `ll` is the label of the massless group (its first
process was written with the label), `taptam` the tau group. The brief's
panic for unknown names is kept and pinned by
`lepton_part_refuses_an_unknown_directory_name`. I did not copy the M3's
commit; `epem`/`emep` stay accepted since F1's expectation was that spelling.

**Control.** The e part's σ stands in for the ta part on massless channels. The
substitution is asserted structurally (item above), not integrated again. It
cannot assert the matrix elements equal: the evaluators differ in which
helicity configurations survive pruning (the tau is massive), and that is the
effect under test. The control reads the e part's five-seed value against the
ta directory and requires |pull| >= 3 and |rel| >= 0.005 (both bounds
violated, not either). On the M3's numbers: 934.520 +- 0.616 against
923.374 +- 0.262, pull +16.7, rel +0.0121 (11.1 pb). **The brief and manifest
said "-1.15%"**: that is the ta part relative to the e part; the control is
the e part relative to the ta directory, +1.21%. The manifest note now says
that.

**Mode mismatch constraint.** The collator rejects a measured cell whose mode
differs from the manifest cell's (`validation-report/src/main.rs:380`), so an
`info` control cannot be written as a cell of a `gate` row; it is printed and
asserted only.

## 2. Commands and output

All with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`;
log `fmL-V2-gate.log`:

```
### fmt             cargo fmt --all --check            exit 0
### clippy          cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 14.00s
### clippy ext      ... --features vibegraph/extended-validation
    Finished `dev` profile [optimized] target(s) in 6.04s
### cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic -- sigma_mixed lepton_part --nocapture
[pp_to_ll_etau] NOT YET BANKED (the manifest marks this row planned and this checkout has no seed record for dy13_etau_default; the split into two parts is checked, no cell is written)
test sigma_mixed_outgoing_masses_vs_mg ... ok
test lepton_part_follows_the_directory_names_madgraph_wrote ... ok
test lepton_part_refuses_an_unknown_directory_name - should panic ... ok
test result: ok. 3 passed; 0 failed
### cargo test -p vibegraph-validation-report   45 / 20 / 12 passed, 0 failed
### python3 scripts/kb.py index   wrote 1 index files
### python3 scripts/kb.py lint    916 concepts, 296 backlog items; 0 errors, 4 warnings
```

(the four warnings are existing `blocked` items without `blocked_by`.)
`cargo test --workspace`: see the last section.

**Measurement body run against a synthetic record.** To run the new gating and
control code end to end, I wrote a made-up seed record (five seeds, two
directories `P1_qq_ll`, `P1_qq_taptam` with the M3's quoted means, invented
errors), ran the test and deleted it. It is not evidence about MadGraph. It
shows the code runs and the three cells and the control are computed: vibegraph
side reproduces the M3 line exactly (e 934.520 +- 0.616, ta 923.762 +- 0.610,
total 1858.282 +- 0.867, all three "gate: within the gate's tolerance"), and
the control line reads `pull = +17.00 | rel = +0.0121 | control: outside the
gate's tolerance`. Also confirmed there: the directory sums equal
`results.dat` for the synthetic seeds (by construction, so no information).
The vibegraph numbers matching the M3's to three decimals is a check on
determinism of this side with the five pinned seeds.

## 3. What the M3 must do when it re-cuts the bundle

1. Commit `validation/madgraph/output/seeded_sigma/dy13_etau_default/` with
   the bundle.
2. In `validation/manifest.toml`, row `pp_to_ll_etau`: delete
   `status = "planned"` (line 527). If the bundle is not re-cut in the same
   PR, add `bundled = false` instead.
3. Run `cargo test -p vibegraph-lib --features extended-validation --test
   validate_hadronic -- sigma_mixed --nocapture`; the `NOT YET BANKED` line
   must be replaced by the measurement and the control line, with the three
   gate lines `within the gate's tolerance`. Once the row is not planned an
   absent record fails through `validation::require`.
4. Check the manifest note's numbers against that output (they are quoted
   from the M3's comment, not re-measured here). The e-part MadEvent error
   there is 0.255 and the ta 0.262 as quoted; the control pull +16.7 follows
   from 0.262.
5. Run the collator: the three cells `__e_part`, `__ta_part`, `__total` are
   now `gate` cells and must report pass.
6. If `bank_seeded_sigma.sh`'s record carries the directories' process lines
   in the future, `lepton_part_of` could classify on them (see Found).

## Found

1. **`lepton_part_of` classifies on the directory label.** The seed record has
   no per-directory process line or PDG list, so a label rename in a future
   MadGraph (or a card spelling the leptons differently) panics rather than
   misclassifies. Close: have `bank_seeded_sigma.sh` keep each P directory's
   `processes.dat`/`matrix` process line in the record, and classify on its
   PDG codes and masses.
2. **The control is structural plus a borrowed value.** It relies on the e part
   standing in for a massless tau; a direct integration (a card with the tau's
   mass set to zero) would pin the claim numerically. Close: a banked-free
   test integrating `p p > ta+ ta-` with `MTA = 0` against the e part.
3. **A control cell cannot live in a gate row** (collator mode-mismatch rule).
   If known-wrong controls should appear in the report, the cell vocabulary
   needs a control kind.

## Workspace test

```
$ cargo test --workspace > fmL-V2-ws.log ; tail -1
EXIT 0        (grep -c "FAILED\|panicked" -> 0)
```

`pixi run validate` was not run: pixi is absent and this checkout has no
reference bundle.
