---
type: Session Report
title: "H-C3 report: an IR-unsafe cut_decays exemption is refused; the every-pairing rule registered"
description: "Cuts::compile_with refuses an exemption from the cuts that leaves uncut a massless coloured leg which is a core parton in some kept diagram (CutError::IrUnsafeExemption). Today's every-pairing rule never produces one, and the guard is bit for bit on two decay-chain cards and one plain card (σ, grid and events SHA-256). Its unit test refuses MadEvent's configuration-1 mask on g g > z u u~, z > u u~, passes today's, and fails with the guard removed. The kb registers vibegraph's every-pairing rule as a deviation of its own cut: stricter than cut_decays = F asks, −26% / −6.8%, σ informational, with the kinematic window rule named as the intended fix. Compat site (g) is folded into site (d)."
status: draft
tags: [cuts, decay-chains, identical-particles, madgraph-parity, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-06}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-06}
sources:
  - {id: hc2, resource: "H-C2-report.md", title: "H-C2 report: the exemption is IR-unsafe on an every-pairing |M|²"}
  - {id: hk1, resource: "H-K1-report.md", title: "H-K1 report, §5b: the measured −26% / −6.8%"}
---
Session H-C3 of sprint H. Worktree `/home/user/wt-cd`, branch
`sprinth-cut-decays`, from `57e1ad92` (H-C2's report on `sprinth-group-cuts`):

```
$ cd /home/user/wt-cd && git rev-parse --show-toplevel && git branch --show-current && git log --oneline -1
/home/user/wt-cd
sprinth-cut-decays
57e1ad92 kb: H-C2 report, cut_decays exemption stopped: IR-unsafe on an every-pairing |M|²
```

`$S` is the scratch directory
`/tmp/claude-0/-home-user-vibegraph/0bdd8e0a-85fe-5422-8ff6-63eab43fffee/scratchpad/H-C3`.

## 1. The guard

**Where the mask is built.** `Cuts::compile_with` (`cuts.rs`) computes
`uncut_products`: `0` at `cut_decays = T`, `ForcedResonances::decay_products()`
at `F`. That mask then sets `do_cuts = false` and `uncut_decay = true` on each
exempt leg, and every reader H-C2 listed reads those two flags. So the mask has
one producer, and the guard sits right after it:

```rust
refuse_ir_unsafe_exemption(uncut_products, forced, legs)?;
```

**What it checks.** `ForcedResonances::of` now also records
`massless_coloured_core`: an OR over every diagram, including a diagram with
no forced line, of the final-state legs that are massless (`model.mass == 0`)
and coloured (`|color| ≠ 1`) and that sit under none of that diagram's forced
lines. `refuse_ir_unsafe_exemption` errors when `uncut & massless_coloured_core`
is non-zero. The error names the lowest such final-state position and its PDG:

```
CutError::IrUnsafeExemption { leg, pdg }
"cut_decays = F would leave final-state leg {leg} (PDG {pdg}) uncut, but it is a massless
 coloured core parton in a kept diagram: the exemption is IR-unsafe, its collinear region
 diverges, and the channels' spacelike floor would cap it silently"
```

By construction `decay_products()` is the intersection over patterns of each
pattern's forced-line union. So today's mask is never core in a diagram that
forces lines. It could only trip on a set that mixes forced and unforced
diagrams, which `decay_products` ignores. No existing card does that: see the
`cargo test` and `validate.sh` results below.

**Tests** (`cuts.rs`, `an_exemption_of_a_massless_coloured_core_leg_is_refused`,
built from the real stitched diagrams under the default SM):

- `g g > z u u~, z > u u~`: 4 patterns, `decay_products() = 0`,
  `massless_coloured_core = 0b1111`. The hand-built masks `0b0011` (MadEvent's
  configuration-1 Z legs), `0b1100`, `0b0001` and `0b1000` are each refused with
  `IrUnsafeExemption`, naming the lowest set bit and `|pdg| = 2`. Today's mask
  passes, and `compile_with` at `F = cut_decays`, `ptj 20`, `etaj 5`,
  `drjj 0.4` compiles with `spacelike_floor() = 400`.
- `g g > z u u~, z > d d~`: 1 pattern, a two-leg exemption, and
  `massless_coloured_core` = the two core quarks. `compile_with` passes on the
  non-empty exemption, and adding the core legs to the mask is refused.

```
$ CARGO_TARGET_DIR=/home/user/wt-cd-target CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
    cargo test -p vibegraph-lib --lib cuts::tests::an_exemption
test cuts::tests::an_exemption_of_a_massless_coloured_core_leg_is_refused ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1276 filtered out; finished in 0.01s
```

**Mutation.** With `if unsafe_legs == 0 {` changed to
`if unsafe_legs == 0 || true {` (the guard removed), the bad mask gets through
and the test fails at its first `unwrap_err` (`$S/mutation.log`):

```
   2: core::result::unwrap_failed
   3: vibegraph::cuts::tests::an_exemption_of_a_massless_coloured_core_leg_is_refused
failures:
    cuts::tests::an_exemption_of_a_massless_coloured_core_leg_is_refused
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1276 filtered out; finished in 0.07s
```

The mutation was reverted (`grep -c "|| true" vibegraph-lib/src/cuts.rs` → `0`),
and the test passes again.

**Blind spot.** "Coloured" is read from the UFO `color` attribute and
"massless" from the evaluated mass. A massless colourless leg (a photon, a
massless lepton) that is a core particle in some pairing has its own collinear
singularity against a charged leg, and the guard does not see it. No rule
today exempts such a leg either (Found).

## 2. Bit for bit

Release binaries: `vibegraph-before` built with `cuts.rs` at `57e1ad92`, and
`vibegraph-after` at the commit (`$S/H-C3-build.log`: `git diff --stat` empty
after the restore, `cmp` → `DIFFERENT`, `BUILD-EXIT 0`). Each runs
`integrate <proc> --run-card <rc> --seed 3 --fixed-budget --neval 40000 --niter 6`,
then `generate grid.bin.zst --seed 7 --nevents 2000`. The SHA-256 is of the full
grid artifact, and of the LHE from the first `<event>` on (`$S/H-C3-bfb.sh`,
`$S/H-C3-bfb.out`). The cards:

- `ggzjj`: `g g > z j j, z > j j` at `cut_decays = F` (H-K1's card). Its
  twelve distinct-flavour subprocesses carry a non-empty exemption that the
  guard inspects.
- `zjee`: `p p > z j, z > e+ e-` at `cut_decays = F`, with the two leptons
  exempt.
- `llj`: the plain `p p > e+ e- j`.

```
before ggzjj: σ = 139.9 ± 1.1 pb grid-sha c3389cc5d3c59dfbd7716568384d1edbd06e6191a2419516a8025c909573641d events=2000 bytes=2185349 events-sha 0e2d43c1e624e7443669b7439fe2498fe7e84d591780e953c6a678e7a257c6a9
before zjee: σ = 400.6 ± 1.0 pb grid-sha e38ffc7ab3c93246882196aa9a650b894698f4bdab1002c724062dc5f5b879ae events=2000 bytes=12958912 events-sha 2682d5a1d0c9f1cc0d345ca0ae104129a89d34bc310479b570994b35d5b33ec6
before llj: σ = 212.23 ± 0.69 pb grid-sha 86793e56d63ebf3b75287a9faa439e5e35a13dee33515a605605b9db07c641d8 events=2000 bytes=12672672 events-sha 1b554ce0cda81568ef2d79f1e3eed1320c324946e8ee719ec978d4e3db6715b5
after ggzjj: σ = 139.9 ± 1.1 pb grid-sha c3389cc5d3c59dfbd7716568384d1edbd06e6191a2419516a8025c909573641d events=2000 bytes=2185348 events-sha 0e2d43c1e624e7443669b7439fe2498fe7e84d591780e953c6a678e7a257c6a9
after zjee: σ = 400.6 ± 1.0 pb grid-sha e38ffc7ab3c93246882196aa9a650b894698f4bdab1002c724062dc5f5b879ae events=2000 bytes=12958911 events-sha 2682d5a1d0c9f1cc0d345ca0ae104129a89d34bc310479b570994b35d5b33ec6
after llj: σ = 212.23 ± 0.69 pb grid-sha 86793e56d63ebf3b75287a9faa439e5e35a13dee33515a605605b9db07c641d8 events=2000 bytes=12672671 events-sha 1b554ce0cda81568ef2d79f1e3eed1320c324946e8ee719ec978d4e3db6715b5
```

None of the events hashes is `e3b0c442…`, the hash of empty input: every file
holds 2000 events. Every integrate run exited 0 (`int-exit 0` six times). The
one-byte difference is the artifact path in the header. Nothing else in the
headers differs:

```
$ for c in ggzjj zjee llj; do diff <(sed '/<event>/,$d' bfb_before_$c.lhe) <(sed '/<event>/,$d' bfb_after_$c.lhe); done
5c5
< artifact bfb_before_ggzjj/grid.bin.zst
---
> artifact bfb_after_ggzjj/grid.bin.zst
(the same one-line diff for zjee and llj)
```

`llj` reproduces H-K1's hashes for the same card and flags
(`86793e56d63ebf3b…`, `1b554ce0cda81568…`).

## 3. The kb

All three edits follow the user's framing, relayed by the manager.
`cut_decays = F` is the default in both programs (`banner.py:4306`
`add_param("cut_decays", False, cut='d')`; `runcard.rs:1126`
`("cut_decays", Def::B(false))`). MadGraph's configuration-1 exemption
implements the card on its single-pairing |M|². vibegraph's every-pairing rule
cuts a stricter observable, all four quarks. The dropped interference is the
existing ~0.2% row and is not the cause.

- `validation/madgraph-defects.md`: a new row after the identical-particles
  row, **approximation (vibegraph's cut)**. It states both rules, the measured
  −26% / −6.8% (H-K1 §5b), and the handling: a registered deviation of
  vibegraph's cut (user, 2026-11-06), σ informational. It names the intended
  fix, the kinematic window rule (exempt the pairing whose pair mass is inside
  the forced window, closest to the pole if several are). It says why exempting
  configuration 1's legs needs site (d) (H-C2's divergence), and it names
  `CutError::IrUnsafeExemption`.
- `process/identical-particles-across-decays.md`: a `cut_decays = F`
  paragraph in "What each side computes" (the two rules, the numbers, the
  deviation, the intended fix, and the guard), a bullet in "How it is gated",
  the compat bullet extended, a sentence in the description, and two sources
  (H-K1, H-C2).
- `backlog/feature/madgraph-compat-sites-unconditional.md` (delegated
  explicitly): site (d) now carries the single pairing, the
  `identical_decay_chain_factor`, the per-pairing window and the
  configuration-1 exemption together. It states the constraint (the exemption
  is IR-unsafe without the single-pairing |M|², H-C2) and adds one line that
  the kinematic window rule is the default-path fix, tracked separately. Site
  (g) never existed on this branch. It exists only on `origin/sprinth-kb` (its
  line 66), so a merge with that branch will conflict on the item. Resolve it
  by deleting (g).

**Manifest.** No banked σ row has an identical particle in both a decay and
the core at `cut_decays = F`:

```
$ grep -n "z > j j\|> j j, \|z j j\|w > j j\|identical particles" validation/manifest.toml
1708:note = "e+ e- > z z, z > e+ e- is printed, not gated: with identical particles across its decays ...
1740:rationale = "Decay-chain stitching against MadGraph's combined matrix elements ...
```

Line 1708 is `decay-chain-sigma`'s `zz_ee`, already informational. Every one of
its four electrons is a decay product in both pairings, so at `cut_decays = F`
both programs exempt all four and this deviation does not reach it. Line 1740
is the structural census, which has no cuts. No note or gate changed.

## 4. Gates

```
$ cargo fmt --all --check                                         → exit 0
$ cargo clippy --workspace --all-targets -- -D warnings           → CLIPPY-EXIT 0   ($S/H-C3-clippy.log)
$ cargo test --workspace                                          → TEST-EXIT 0     ($S/H-C3-test.log)
  55 test-result lines: passed 1969, failed 0, ignored 25
$ python3 scripts/kb.py index                                     → exit 0
$ python3 scripts/kb.py lint
1142 concepts, 353 backlog items; 0 errors, 5 warnings
```

The 5 warnings are the same count as before the change (checked on a stash).
The compat item's "body is N lines" warning was already there before this
change.

`validation/validate.sh` ran detached (`nohup setsid`), with
`CARGO_TARGET_DIR=/home/user/wt-cd-valtarget`, `CARGO_BUILD_JOBS=2`, and an
untracked `.cargo/config.toml` holding `[profile.release-debug]` `debug = 0`.
That file was deleted afterwards. The log is `$S/H-C3-validate.log` and a copy
of the report is `$S/H-C3-validation-report.md`:

```
85 test-result lines: passed 2423, failed 0, ignored 110
zz_ee  e+ e- > z z, z > e+ e-  MG 4.719580e-4 ± 3.33e-7 (5 seeds, χ²/dof 1.29) | ours 4.731611e-4 ± 1.64e-7 (χ²/dof 1.49) | ours/MG − 1 = +2.549e-3 …
test decay_chain_cross_sections_match_madevent_over_seeds ... ok
test a_hadronic_decay_chain_sample_matches_madevents_records ... ok
77 rows × 4 categories = 308 cells: 218 measured in the layers this run drove (213 ✅, 5 ⚠️, 14 ⏳, 76 — / uncovered).
The measured cells are exactly the cells the manifest declares, every gate cell passed, …
VALIDATE-EXIT 0
```

The five ⚠️ cells are informational cells measured and deliberately not
enforced, all outside this change: `gg_to_gg` diagrams 4/6,
`ee_to_wpwm_cw` amplitudes, `gg_to_gg_cg` diagrams 21/27, `ee_to_zh_smeft`
amplitudes, and `ee_to_mumua` samples. `zz_ee` reads +2.55e-3, the
registered pairing interference (+2.6e-3 ± 0.8e-3 banked), and stays
informational.

## Commits

- `398a9061` feat(cuts): refuse an IR-unsafe cut_decays exemption; register the stricter cut
- this report

## Found

- **The guard sees only coloured legs.** A massless colourless leg (photon, or
  a lepton under a massless-lepton restriction) that is a core particle in
  some pairing and a decay product in another has the same failure mode: an
  uncut γ or ℓ collinear to a charged leg. Nothing exempts one today, because
  the every-pairing rule never does. The kinematic window rule, though, would
  exempt by pairing. To close: when the window rule lands, extend
  `massless_coloured_core` to every massless leg with a gauge charge, or
  restate the guard over the exempting pairing's |M|² support.
- **The kinematic window rule needs a per-point exemption, not a per-subprocess
  mask.** `uncut_products` is compiled once per subprocess. The window rule
  chooses the exempt pairing per point, so it cannot go through this one mask.
  The guard then has to run per pairing: each pairing's exempt legs must be
  decay products of that pairing's diagrams. Those diagrams are a subset of
  the kept ones, so the guard as written (OR over every diagram) would refuse
  the rule's masks. To close: in the window-rule item, rebuild the guard over
  the diagrams of the exempting pairing, not of the whole set.
- **Release build warns `resolve_mom` is never used**
  (`vibegraph-lib/src/helas/eval/analysis.rs:208`). It shows in
  `cargo build --release -p vibegraph`, from a file this session did not touch
  (`$S/H-C3-build.log`). Clippy on all targets is clean, which suggests (not
  checked) a caller only under `cfg(test)` or a feature. To close: gate it the same way, or
  remove it.
- **Site (g) lives only on `origin/sprinth-kb`.** This branch's version of the
  compat item never had it, and the merge will conflict there (§3).
