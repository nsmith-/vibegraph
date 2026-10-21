---
type: Session Report
title: "F2 report: run card first in generate, one CLI test module, and what unequal beams need"
description: "generate refuses a run card before reading the artifact; the CLI tests share tests/common; unequal beams are exact on the lpp = 0 path but the proton path is built for equal beams only, so part 3 stopped at the finding."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
Worktree `/home/user/wt/B`, branch `sprintd-cut-refusal`. Parts 1 and 2 are
done and committed; part 3 stopped at its finding, on the coordinator's
amended brief (correct √s for unequal beams, no refusal; stop and report if
the chain does not already handle an asymmetric collider). It does not on the
proton path, so `collider-energy-sum-of-beam-energies` stays open.

## 1. `generate-reads-run-card-after-artifact` (closed)

`generate::run` (`vibegraph-cli/src/generate.rs:757`) now reads the artifact
straight after `load_run_card`, so a refusable card is refused before the
artifact is opened, as `integrate` refuses it before enumeration. Every
refusal and its message is unchanged; the model still loads before the run
card in both commands, as before.

Test `cli_generate_refuses_the_run_card_before_reading_the_artifact`
(`cli_hard_errors.rs`): an artifact file that does not parse, with a clean
card (control: the error names the artifact, `cannot read …not_an_artifact`)
and with `30 = ptj1min` (the cut refusal, and no mention of the artifact).

Mutation: `generate.rs` reverted to the old order (stash), the new test only:

```
thread 'cli_generate_refuses_the_run_card_before_reading_the_artifact' (1048) panicked at vibegraph-cli/tests/cli_hard_errors.rs:643:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.27s
```

(`:643` is `assert!(stderr.contains("'ptj1min' (= 30"))`: the artifact error came
first.) Restored and touched; the binary passes 14/14.

## 2. `cli-test-helpers-copied-per-file` (closed)

`vibegraph-cli/tests/common/mod.rs` holds `vibegraph(cwd, home)` (the binary
with `VIBEGRAPH_HOME` set and the network, PDF-dir and UFO-dir overrides
removed), `vibegraph_behind_dead_proxy` (that plus `ALL_PROXY` at
`127.0.0.1:1` and `NO_PROXY`/`no_proxy` removed, which `cli_first_run` needs),
`stderr_of`, and `fixed_beams(ebeam1, ebeam2)` (the four `lpp`/`ebeam` lines).
It carries `#![allow(dead_code)]`, as `vibegraph-lib/tests/common` does,
because each test binary uses part of it.

Users: `cli_hard_errors`, `cli_first_run`, `cli_fixed_energy`,
`cli_generate` (two lines: `mod common;` and `write_run_card`'s body) and
`cli_ufo_model` (found by grep; it wrote the same card). The cards' text is
unchanged: literals that read `250.0` are passed as the string `"250.0"`.
The deliberately broken card in `cli_a_run_card_that_does_not_parse_is_named`
stays literal. The other test files call
`Command::new(env!("CARGO_BIN_EXE_vibegraph"))` directly with their own
arguments and environments; they re-implement none of the helpers, so they
were left alone.

Test counts per binary, `cargo test -p vibegraph --tests -- --list`, before
and after (identical; `diff` empty):

```
unittests src/main.rs 106 · cli_decay 3 · cli_first_run 5 · cli_fixed_energy 5 · cli_generate 10
cli_hard_errors 14 · cli_onshell_veto 2 · cli_pdf_cache 4 · cli_reference_docs 0 · cli_ufo_model 2
```

Run of the five touched binaries after the change:

```
cli_first_run    test result: ok. 5 passed; 0 failed
cli_fixed_energy test result: ok. 5 passed; 0 failed
cli_generate     test result: ok. 10 passed; 0 failed
cli_hard_errors  test result: ok. 14 passed; 0 failed
cli_ufo_model    test result: ok. 2 passed; 0 failed
```

Mutation, to show every file reads its cards from the shared helper:
`fixed_beams` writing `1 = lpp1`, `--no-fail-fast`:

```
cli_first_run    test result: FAILED. 4 passed; 1 failed
cli_generate     test result: FAILED. 0 passed; 10 failed
cli_hard_errors  test result: FAILED. 5 passed; 9 failed
cli_ufo_model    test result: FAILED. 0 passed; 2 failed
```

and `ebeam2` written ten times larger: `cli_fixed_energy` FAILED, 3 passed, 2
failed (`fixed_energy_2to2_reproduces_madgraphs_sigma`,
`fixed_energy_nbody_finite_sigma`). Restored from a copy and touched.

## 3. `collider-energy-sum-of-beam-energies` (stopped at the finding; open)

### What the chain does today

**Fixed-energy beams (`lpp = 0`): already exact.**

- √ŝ: `integrate`'s fixed path takes `initial.sqrt_s()`
  (`vibegraph-cli/src/integrate.rs:1070`), from `FixedBeams::from_run_card`
  (`vibegraph-lib/src/hadronic.rs:734`), which is
  `beams::partonic_s([ebeam1, ebeam2], masses)`
  (`vibegraph-lib/src/phasespace/beams.rs:72`):
  `ŝ = m_a² + m_b² + 2(E_a E_b + |p_a||p_b|)`, which is `4 E_a E_b` for
  massless beams. The test
  `unequal_massless_energies_do_not_collide_at_the_energy_sum` (`beams.rs`)
  already pins `ŝ = 4·140·360` for 140 + 360 GeV.
- Cuts: `FixedBeamIntegrand::passes_cuts` (`hadronic.rs:2917`) boosts the
  configuration into the laboratory by `lab_beta`
  (`beams::lab_beta_z`, `beams.rs:82`) before the cuts read it.
- Written events: in the partonic CM frame (`FixedBeams::momenta`,
  `hadronic.rs:766`); `EBMUP` is `[ebeam1, ebeam2]` (`generate.rs:971`).
- So on this path `ebeam1 + ebeam2` survives only in prose:
  `vibegraph-lib/src/runcard.rs:188` (`BeamMode::FixedEnergy` doc, "√ŝ = ebeam1 + ebeam2")
  and the module doc of `vibegraph-cli/tests/cli_fixed_energy.rs:7`. Both are
  wrong for unequal beams and were left as found (part 3 stopped).

**Proton beams (`lpp = 1`): built for equal beams only.**

- √s: `let sqrt_s_had = rc.ebeam1 + rc.ebeam2;` at `integrate.rs:931`
  (`integrate_hadronic`) and `generate.rs:1712` (`generate_proton_sample`).
  It feeds `ProtonIntegrand::new_with_maps` and `multiplicity_maps`, is
  recorded in the artifact's `sqrt_s_had`, and becomes
  `s_had = sqrt_s_had²` (`vibegraph-lib/src/proton.rs:1586`).
- x₁, x₂: drawn as `τ = x₁x₂`, `y = ½ ln(x₁/x₂)`
  (`proton.rs:1880`–`:1999`), `ŝ = τ·s_had`. The PDFs are read at those
  `x`; that part is frame-independent.
- Lab frame: `ProtonIntegrand::build_frames` (`proton.rs:2004`–`:2019`) puts
  the partons at `x_i · √s/2` along `±z` and boosts the outgoing legs by
  `β = (x₁ − x₂)/(x₁ + x₂)`. Both assume `E₁ = E₂ = √s/2`. The lab momenta
  are what the cuts read and what `generate` writes
  (`generate.rs:1593`, `event.lab`).
- So for unequal proton beams √s is wrong ((E₁+E₂)² instead of ≈4E₁E₂), the
  incoming partons' lab energies are wrong, the boost is wrong, and with it
  every lab-frame cut and every written momentum.
- One proton-side reader is already exact: the clustering record's `ptclus`
  takes `collider_energy` (`vibegraph-lib/src/coupling/scales.rs:831`),
  MadGraph's `stot` with 0.938 GeV beams.

**`lpp` combinations.** `RunCard::parse` admits only `(1,1)` and `(0,0)`
(`vibegraph-lib/src/runcard.rs:431`), so no mixed path exists.

### What MadGraph does (b7687064)

- `stot`, `Template/LO/SubProcesses/genps.f:656-676`: beam masses `m1, m2`
  are 0.938 for `|lpp| = 1, 2` (`:662-663`), the leg masses otherwise;
  `stot = m1² + m2² + 2(E₁E₂ − p₁ᶻp₂ᶻ)` with `p₂ᶻ < 0` (`:676`); printed as
  "Set CM energy to" (`:678`).
- Proton beams: `ŝ = x₁x₂·stot` (`genps.f:348`), and the CM rapidity
  `cm_rap = ½ ln(x₁E₁/(x₂E₂))` (`:345`), which `rap()` adds before a
  rapidity cut reads it.
- Fixed beams: `xbk = (1, 1)` (`genps.f:249-250`), `ŝ = stot` (`:391`), and
  `cm_rap = ½ ln((p⁰+p³)/(p⁰−p³))` with `p⁰ = E₁+E₂`,
  `p³ = |p₁| − |p₂|` (`:386-389`).
- Written events: `write_leshouche` boosts to the laboratory
  (`Template/LO/SubProcesses/unwgt.f:633-717`): `β` from `get_betaz`
  (`unwgt.f:145-170`) taking an incoming parton's CM momentum, divided by its
  `x`, to the beam's lab momentum (`:648-656` beam 1, `:676-684` beam 2), and
  every leg boosted by it (`:703-706`), with the older
  `η = √(x₁E₁/(x₂E₂))` boost as a fallback when the boosted beam exceeds
  `ebeam` (`:709-717`). `EBMUP = ebeam(i)` (`Template/LO/Source/setrun.f:194-195`).

### Branch taken

The proton chain does not handle an asymmetric collider (frames, not only √s),
so on the amended brief part 3 stops here: no √s change, no refusal, and the
item stays open. Nothing in `vegas.rs`, `budget.rs` or the maps was touched.

### What asymmetric proton beams would need

1. **√s** at `integrate.rs:931` and `generate.rs:1712`: a beam-energy-aware
   value. Two candidates, and the choice matters for banked cells:
   - `2√(E₁E₂)` (massless beams). For equal beams it equals `E₁ + E₂` exactly
     in binary64 (`sqrt(fl(E·E)) = E`), so no banked cell can move.
   - MadGraph's `stot` with 0.938 GeV beams (`scales.rs:831`
     `collider_energy` already computes it). For equal beams it is not
     bit-identical: `√stot = 12999.999999999998` at 6500 + 6500,
     `499.99999999999994` at 250 + 250 and `100.00000000000001` at 50 + 50
     (python, same formula). That would move every bit-for-bit seeded
     proton row by an ulp of √s, so it needs either the massless form or a
     rebaseline.
2. **Beam energies into `ProtonIntegrand`** in place of a lone
   `sqrt_s_had`: the five constructors (`proton.rs:1394`–`:1487`) and their
   callers in `integrate.rs` and `generate.rs`.
3. **`build_frames`** (`proton.rs:2004`): partons at `x_i E_i`, boost
   `β = (x₁E₁ − x₂E₂)/(x₁E₁ + x₂E₂)`, which is MadGraph's
   `cm_rap = ½ ln(x₁E₁/(x₂E₂))` (`genps.f:345`) as a velocity. The `(τ, y)`
   draw needs no change: `|y| ≤ ½ ln(1/τ)` is the `x_i ≤ 1` bound whatever the
   energies.
4. The proton-path tests that build lab momenta from `SQRT_S_HAD / 2`
   (`proton.rs:4404`, `:4524`, `:5748`) stay valid for equal beams; a new
   unit test at unequal energies would pin (1) and (3) against hand values.
5. **Docs**: `runcard.rs:188`, `proton.rs:1372` ("`√s = E₁ + E₂`"),
   `cli_fixed_energy.rs:7`, and the no-op ceiling's doc
   (`vibegraph-lib/src/cuts.rs:1043-1044`), which states the sum as the
   generator's convention.
6. **A reference**: no banked MadGraph row has unequal beams (all 46 rows of
   `validation/madgraph/sigma_reference.json`, and every `ebeam` in the
   generator scripts, are equal), so σ and the written frame at unequal
   energies would need a new banked run to gate against.

Size: roughly 50–100 lines of library code in `proton.rs` plus signature
plumbing in the two commands, one unit test, and a new reference row (an MG
regeneration).

### Every reader of √s, and whether a banked cell could move

| reader | site | what it uses √s for |
|---|---|---|
| proton integrand | `proton.rs:1586`, `:1997`, `:2012` | `s_had` for `ŝ = τ s`, `τ_min`, the lab frame |
| phase-space maps | `integrate.rs:944`, `generate.rs:1746` → `multiplicity_maps` → `process_shape` / `maps.channel(…, sqrt_s_had, …)` | the channel invariants' upper bound and the map rule |
| configuration weights | `proton.rs:2365`, `:2729` (`channel_cuts(…, s_had, …)`) | the `s_tot·1e-10` regulator (`coupling/cluster/graph.rs:252`) |
| artifact | `integrate.rs:999`, `artifact.rs:363` | recorded `sqrt_s_had`, printed `√s` (`integrate.rs:685`) |
| clustering record | `scales.rs:831` | MadGraph's own `stot` (already exact) |
| no-op cut ceiling | `cuts.rs:1063` | `(ebeam1 + ebeam2)/2`, ≥ √(E₁E₂) ≥ any single pT, so sound at any energies |
| fixed-beam path | `hadronic.rs:734`, `beams.rs:72` | exact `ŝ` already |

Gates that read it: every `validate_sigma`/`validate_samples*` proton row,
`cli_integrate`, `cli_generate_proton`, `cli_reweight_proton`, the
decay-chain and on-shell-veto CLI gates, and every fixed-beam row through
`partonic_s`. All banked rows are equal-beam, so with the massless `2√(E₁E₂)`
no banked cell could move; with MadGraph's massive `stot` the bit-for-bit proton
rows would move by an ulp of √s.

## Gate

Run from the committed parts 1 and 2 plus this report, with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

```
$ cargo fmt --all --check
fmt exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 20.94s
clippy exit 0
$ cargo test --workspace
test exit 0
37 `test result: ok` lines; summed: passed 1549, failed 0, ignored 16
$ python3 scripts/kb.py lint      (after `python3 scripts/kb.py index` for this report)
829 concepts, 289 backlog items; 0 errors, 4 warnings
```

The four lint warnings are pre-existing (`state: blocked` without `blocked_by`
on items this branch does not touch). No MG net run: no amplitude, colour,
coupling, phase-space or sampling code changed.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`, one `Assisted-by` line each:

- `30d51b4` fix(cli): generate loads the run card before reading the artifact
  (deletes `generate-reads-run-card-after-artifact`)
- `66b5551` test(cli): one shared module for the binary, stderr and fixed-beam
  cards (deletes `cli-test-helpers-copied-per-file`)
- this report's commit (report and the regenerated sessions index).
  `collider-energy-sum-of-beam-energies` is not deleted.

## Found

1. **The event-record frame for unequal fixed beams is contradicted between
   MadGraph's source and the knowledge bundle.** `unwgt.f:633-717` at
   b7687064 boosts every written event into the laboratory, including
   `lpp = 0` (the beam-2 branch at `:670-697`, taken when `lpp2 = 0` and the boost at `:703-706`);
   `research/kb/phase-space/fixed-beam-kinematics.md` (Frames table) says
   MadGraph writes the CM frame, citing banked `p3 r3` events with `Σ pz = 0`
   (equal energies, masses 60/70, where the lab and the CM differ by
   `y = 5.386e-3`), and vibegraph writes the CM frame. The banked event files
   are not in this worktree, so it was not settled. Close: read a banked
   `p3 r3` event's beam records (250/250 in the lab, 248.70/251.30 in the CM)
   and either correct the concept or boost the fixed-beam record.
2. **Fixed-beam √ŝ docs state the energy sum.** `runcard.rs:188` and
   `cli_fixed_energy.rs:7` say `√ŝ = ebeam1 + ebeam2`; the code is exact
   (`beams.rs:72`). Close: state `ŝ = (p_a + p_b)²` there (could ride with
   the open item).
3. **`proton.rs:1372` documents `√s = E₁ + E₂`** as the integrand's contract;
   it goes with the open item's fix.
