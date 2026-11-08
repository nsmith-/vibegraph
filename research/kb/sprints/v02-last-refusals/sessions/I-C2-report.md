---
type: Session Report
title: "I-C2 report: refuse every uncut massless soft or collinear limit"
description: "Final-collinear, initial-collinear and soft limits that no compiled cut regulates are refused; all 62 seeded rows accepted under the equivalent-photon exemption; a pole probe shows pp_to_epemjj_qcd0 diverges at tree level (D1: refuse it)."
status: draft
tags: [cuts, ir-safety, guards, collinear, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-08}]
sources:
  - {id: c1, resource: "https://github.com/nsmith-/vibegraph/commit/83ec27d6", title: "83ec27d6 on sprinti-cuts"}
  - {id: c2, resource: "https://github.com/nsmith-/vibegraph/commit/71b5302e", title: "71b5302e on sprinti-cuts"}
---
Manager's spot checks: the commit identity of 83ec27d6, and the sweep log (62
`ACCEPTED` rows, `SWEEP DONE`). The gates, the ladder, the probe and the
mutations are the session's claims. The report was recovered from the
session's transcript after a container restart. The user then chose option
(a) ([D1](../decisions/D1-ewk-lljj-refused.md)); the flip and the row's
retirement follow in addendum 1, and the xqcut fix after review in addendum 2.
The manager read both addenda's gate logs (EXIT 0 on b2f3365d and 2b26c109).


All three gates are green on branch `sprinti-cuts` at `71b5302e`: the 62 banked seeded rows are all accepted, `cargo test --workspace` passes, and `bash validation/validate.sh` exits 0. Separately, a direct matrix-element probe shows that the banked electroweak card `pp_to_epemjj_qcd0` diverges at tree level. The guard currently accepts it (option b); refusing it (option a) is a one-line flip and is the user's decision.

Both commits are pushed, authored by Nicholas Smith, each with exactly one trailer `Assisted-by: claude-code:claude-opus-5-5` and no Co-authored-by:
- `83ec27d6` feat(cuts): refuse every uncut massless soft or collinear limit. Touches `/home/user/wt/sprinti-c2/vibegraph-lib/src/cuts.rs`.
- `71b5302e` test(proton): regulate the lepton pair in the degenerate-point census test. Touches `/home/user/wt/sprinti-c2/vibegraph-lib/src/proton.rs`.

No PR is opened. Nothing under `research/kb/` was touched, and the working tree is clean.

### Corrections to the brief
1. **Class 1's condition is wrong.** The brief says "`ptj = 0` and `drjj = 0`", but either cut missing is enough for a divergence.
   - With `ptj > 0` and `drjj = 0`, the collinear limit diverges: `s_ab = 2 pT_a pT_b (cosh Δy − cos Δφ)` goes to 0 at any fixed pT.
   - With `drjj > 0` and `ptj = 0`, the soft gluon diverges at any angle.
   - xqcut regulates only through the `mmjj = xqcut` rewrite (`auto_ptj_mjj = T`). An xqcut card with `auto_ptj_mjj = F`, `mmjj = 0` and `drjj = 0` is refused. vibegraph does not implement MadGraph's kT clustering cut (`reweight.f:1063-1077`), so nothing regulates that card here.
2. **"Partons above maxjetflavor" needs no rule of its own.** The rules read the per-pair compiled cuts, so a massless b at `maxjetflavor = 4` falls under `drbb`, `drbj` and `ptb` automatically.
3. **What the analysis added:** the soft limit of a massless vector boson, which only its own pT or energy minimum regulates, and the `γ* → f f̄` final-state splitting. Both are in the item's `closes_when` ("collinear or soft pole").

### The rules (cuts.rs: `MasslessLimit`, `diagram_massless_limits`, `limit_regulated`, `refuse_unregulated_limits`)
The rules read three-point vertices with one propagator and two external legs. Each limit is judged at fixed ŝ.
- **FinalCollinear** (legs a < b): two massless outgoing legs at a vertex whose third line is a massless propagator, with n_final ≥ 3. A 2→2 pair's invariant is ŝ itself, so that case belongs to the ŝ floor.
  - It is regulated only by that pair's compiled dR or pair-mass minimum.
  - A photon with a jet- or b-class parton keeps `CutError::UncutPhotonSplitting`.
  - Everything else gives `UncutCollinearSplitting{a, b, pdg_a, pdg_b, regulator}`, for example `"drjj > 0 or mmjj > 0"`.
- **InitialCollinear** (beam, leg): a massless outgoing leg at a vertex with a massless incoming leg and a massless spacelike propagator.
  - It is regulated by the leg's pT minimum or by a rapidity maximum. At bounded y a fermion reaches t → 0 only by going soft, which is integrable.
  - Error: `UncutInitialCollinear`.
- **Soft** (leg): a massless spin-1 outgoing leg emitted off an external line through a propagator of that line's mass.
  - It is skipped when n_final = 2 and the other final leg is massless, because that limit is ŝ → 0.
  - It is regulated by the leg's pT or E minimum, or by a pair-mass minimum against any partner above that partner's mass².
  - Error: `UncutSoftEmission`.
- **Reusable predicate for I-C1:** `limit_regulated(limit, infos, single, pairs, legs)` answers "is this massless limit regulated by the compiled cuts?". It is module-private, because its arguments are private types. Limits are stored per subprocess in `ForcedResonances::massless_limits`, which replaces `photon_splittings`.
- **The option switch:** `initial_collinear_is_guarded(exchange, model)` returns `spin != 3`. It is the one predicate that exempts the equivalent-photon (or equivalent-gluon) limit through a massless vector exchange. Flipping it to (a) means returning `true`, rewriting `the_equivalent_photon_limit_is_not_refused` as a refusal, and handling the `pp_to_epemjj_qcd0` manifest row.
- **Behaviour change on non-banked cards:** a 2→2 photon–parton card such as `u g > u a` at `draj = 0` is no longer refused, because its ΔR is at least π.

### Tests and mutation evidence
New tests in cuts.rs, plus the rewritten photon test. The photon test keeps `u d > a u d` at `draj = 0` refused with UncutPhotonSplitting{0,1,pdg 2}.

| Test | What it checks |
|---|---|
| `an_uncut_final_state_parton_splitting_is_refused` | `u u~ > d d~ g` at drjj = 0 (ptj = 20) refused (legs 0,1, PDG 1/−1, "drjj > 0 or mmjj > 0"); `g g > g g g` refused; drjj = 0.1, mmjj = 5 and xqcut = 30 each accepted |
| `an_uncut_soft_gluon_is_refused` | `u u~ > d d~ g` and `u u~ > z g` at ptj = 0 refused as Soft (PDG 21, "ptj > 0 or ej"); `ej = 1` accepted |
| `an_uncut_initial_state_splitting_is_refused` | `g u > z u` at ptj = 0, etaj = −1 refused (beam 0 PDG 21, leg 1 PDG 2); etaj = 5 alone, or ptj = 20 alone, accepted |
| `an_uncut_lepton_photon_splitting_is_refused` | `e+ e- > mu+ mu- a` at dral = 0 refused (0,2, PDG −13/22, "dral > 0"); pta = 0 refused as soft; defaults accepted |
| `an_uncut_splitting_above_maxjetflavor_is_refused` | sm-no_b_mass `g g > b b~ g` at maxjetflavor 4 refused ("drbb > 0 or mmbb > 0"); maxjetflavor 5 accepted; massive b accepted |
| `the_equivalent_photon_limit_is_not_refused` | pins the switch: `u d > e+ e- u d QCD=0` on the EWK card's cuts compiles; with mmll = 0 the γ* → e+e- pair is refused |

Each class was disabled in turn and the new tests re-run; mutation script `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2/mutate.sh`, log `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-mutations.log`:

| Mutation | Tests that fail | Tests that pass |
|---|---|---|
| FinalCollinear only when one leg is a photon | final_state_parton, above_maxjetflavor, equivalent_photon | 4 |
| InitialCollinear off | initial_state | 6 |
| Soft off | lepton_photon, soft_gluon | 5 |
| Switch flipped to (a) | equivalent_photon | 6 |

### Every banked card still accepted
- **Sweep:** all 62 `output/seeded_sigma` rows, with proc, run and param cards from each row's first banner, through `vibegraph integrate` (release-debug at 83ec27d6) at `--fixed-budget --niter 1`.
- **Result:** all 62 ACCEPTED, `pp_to_epemjj_qcd0` included under (b).
  - 49 rows passed on the first pass at neval 1000.
  - 13 needed a second pass at neval 20000. Six of these failed the post-integration effective-sample-size check, which runs after the cuts compile. Seven SMEFT rows lacked `restrict_vg_*.dat`; I supplied them from `validation/madgraph/cards/smeft` in a scratch UFO dir.
- **Log:** `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2/ic2-sweep.log`, with the first pass in `ic2-sweep.pass1.log`.
- **Positive control, same binary:** the ee_to_mumua card with `dral = 0` gives `ERROR failed to compile cuts: final-state legs 0 (PDG -13) and 2 (PDG 22) split off one massless propagator ... regulate it with dral > 0`.
- **Rows not in seeded_sigma** (grammar, decay-chain, onshell-veto and MLM JSON rows) were checked by reading their processes and cards, not run. None has a massless emission without a cut.
- **The diagrams-only row `z_to_uuxddx_decay`** would be refused (g* → d d̄ at zero cuts), which is correct physics. Nothing compiles its cuts.

### The pp_to_epemjj_qcd0 question
**Ladder.** `u d > e+ e- u d QCD=0 QED=4 $ h` (sm-no_b_mass) on the EWK run card with only ptj varied. Seeds 101–105, `--fixed-budget --neval 1000000 --niter 20`, about 80 s a seed. Unweighted mean, error = max(quoted/n, sd/√n). Driver `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2/ladder.sh`, log `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2/ic2-ladder.log`.

| ptj (GeV) | σ (pb) | Seed sd | χ²/dof |
|---|---|---|---|
| 10 | 0.118634 ± 0.000331 | 0.32% | 1.00 |
| 1 | 0.207237 ± 0.001371 | 1.48% | 2.20 |
| 0.1 | 0.207207 ± 0.000191 | 0.21% | 1.25 |
| 0.01 | 0.207338 ± 0.000168 | 0.18% | 0.79 |
| 0.001 | 0.208323 ± 0.000794 | 0.85% | 5.50 (seeds +0.6%, +2.0%) |
| 0 (the card) | 0.207333 ± 0.000139 | 0.05% | 0.13 |

- σ rises 75% from 10 to 1 GeV. Below 0.1 GeV it moves 0.06% ± 0.12% per decade, flat within resolution.
- ptj = 0.001 is heavy-tailed, and the ptj = 0 rung's χ²/dof of 0.13 is suspiciously low.

**Pole probe (the decisive oracle).** A scratch test, not committed: `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2/ic2_epa_probe.rs`. Copy it to `vibegraph-lib/tests/` and run `cargo test -p vibegraph-lib --test ic2_epa_probe -- --nocapture`.
- It evaluates |M|² at √ŝ = 1 TeV, x = 0.5, a fixed 3-body rest system, averaged over 16 azimuths, as the outgoing u approaches its beam.
- `u d > e+ e- u d QCD=0`: |M|²·|t| reads 2.0235e-6, 1.9378e-6, 1.93698e-6, 1.936969e-6 and 1.936969e-6 at |t| = 1.25e3, 1.25e1, 1.25e-1, 1.25e-3 and 1.25e-5 GeV². That is a constant, non-zero 1/t residue.
- Control `... / a`: 1.1e-8, 2.9e-11, 2.8e-13, 2.8e-15, 2.8e-17, so no pole.
- The measure goes as d|t| at fixed x, so σ goes as ∫dt/t, which diverges: the equivalent-photon log off a massless quark. **The card diverges at tree level.**

**Why the ladder plateaus.** With no pT cut, every peripheral rung draws its transfer against a 1 GeV² pole floor: `DEFAULT_POLE_SCALE = 1e3` × `POLE_FRACTION_OF_FIDUCIAL_SCALE = 1e-3` (diagram_channel.rs:1843–1871).
- Below about 1 GeV² the sampling density is flat while the integrand goes as 1/t.
- The estimator's expectation is infinite, and that region is reached only through rare huge weights. That matches the ptj = 0.001 tail and the too-small χ²/dof at ptj = 0.
- MadEvent's `10/stot` floor in `set_peaks` plays the same role, so the banked 0.36% agreement is two samplers' effective cutoffs agreeing.
- The flat ladder is therefore not evidence that the card is finite.
- I did not run a MadGraph ladder.

### Gates (run on 71b5302e under option b)
- **fmt and lint:** `cargo fmt --all --check` passes. `cargo clippy -p vibegraph-lib --lib --tests -- -D warnings` is clean, and validate.sh's workspace clippy step (extended-validation features, `-D warnings`) passed.
- **`cargo test --workspace --no-fail-fast`** (lean env): EXIT 0, 55 targets, 1992 passed, 0 failed, 26 ignored. Log `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-wstest.log`.
  - The first run at 83ec27d6 failed exactly one test, `proton::tests::a_point_with_a_massless_leg_at_rest_is_zeroed_and_counted`, with `Cut(UncutCollinearSplitting { a: 0, b: 1, pdg_a: -11, pdg_b: 11, regulator: "drll > 0 or mmll > 0" })`.
  - The refusal was correct: its p p > l+ l- j card had drll = mmll = 0. I set `mmll = 10` in 71b5302e.
  - The degenerate-momentum check runs before the cuts (proton.rs:2725), so the test's `census.degenerate == 2` assertion still tells the guard apart from the cuts.
- **`bash validation/validate.sh`:** EXIT 0. 85 test results, 2446 passed, 0 failed, 111 ignored. Collator: "77 rows x 4 categories: the measured cells are the declared cells", with no ❌ cell in `target/validation-report/report.md`. Log `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-validate.log`.
- **σ lines unchanged:** the gates passed, but this is not a bit-for-bit comparison against a main-branch report. The argument is from the code: the change adds compile-time refusals only, and `Cuts`' fields and `pass` are untouched.

### Found
1. **pp_to_epemjj_qcd0 diverges at tree level** (probe above). Its banked σ, and the 0.36% agreement with MadGraph, are set by each sampler's pole floor (vibegraph's 1 GeV², MadEvent's `10/stot`). What closes it: the user's (a)/(b) decision. Under (a), flip `initial_collinear_is_guarded`, invert the pinning test, and re-scope or retire the row, for example by giving it an etaj or ptj cut and re-banking.
2. **xqcut with `auto_ptj_mjj = F` has no regulator here.** vibegraph does not apply MadGraph's kT clustering cut (`reweight.f:1063-1077`). Such a card with `mmjj = 0` integrates in MadGraph but is refused by vibegraph: a parity gap, not a silent σ. What closes it: implement the kT clustering cut, or name it in the refusal.
3. **Decay-mode cards (all cuts removed) with any massless radiation are now refused**, for example `z > u u~ d d~` and `z > e+ e- a`. This is correct physics, but the CLI's absent-run-card path for decays will now refuse these processes outright. Worth a release-notes line.
4. **Misplaced doc comment (pre-existing, not mine):** in `vibegraph-lib/src/proton.rs` the doc block "A clustered term's scales are read in the partonic CM…" sits on `a_point_with_a_massless_leg_at_rest_is_zeroed_and_counted` (about line 7071) instead of on `clustered_scales_are_read_in_the_partonic_centre_of_mass`.
5. **A banked check of the guard on the production path is missing.** The 62-row sweep is a scratch driver (`/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2/sweep.sh` with `extract_cards.py`). Most σ gates call `Cuts::compile` without diagrams, so they never exercise the guard. A banked test that compiles every manifest row's cuts through `ForcedResonances::of` would keep "every banked card accepted" true automatically.
## I-C2 addendum: option (a) is in, and both gates pass

`pp_to_epemjj_qcd0` is now refused, its banked row is retired, and the new colourless-exemption test is in. The re-sweep reads 61 accepted and one refused, as expected. `cargo test --workspace` and `validate.sh` both exit 0 at the branch head `b2f3365d` (`sprinti-cuts`, pushed; `origin/sprinti-cuts` is at b2f3365d and the tree is clean).

### Commits
All are authored by Nicholas Smith and carry the single trailer `Assisted-by: claude-code:claude-opus-5-5`.
- `249dd1a0` feat(cuts): refuse the equivalent-photon limit; retire pp_to_epemjj_qcd0
- `b2f3365d` test(cuts): an exempt massless colourless core leg is refused by its limits

These sit on `83ec27d6` and `71b5302e` from the first report.

### 1. The guard (249dd1a0)
- **Exemption removed:** `initial_collinear_is_guarded` is gone, since nothing else used it. Every initial-collinear limit through a massless spacelike propagator is now guarded. The `cuts.rs` module doc says so and names the equivalent-photon case.
- **Test rewritten as a refusal:** `the_equivalent_photon_limit_is_refused`.
  - `u d > e+ e- u d QCD=0` on the EWK cuts (ptj = ptl = 0, etaj = etal = −1, drjj = drll = drjl = 0, mmjj = 4, mmll = 50, maxjetflavor = 5) gives `UncutInitialCollinear{beam 0, beam_pdg 2, leg 2, pdg 2}` with regulator "ptj > 0 or a rapidity maximum etaj".
  - With etaj = 5 the card compiles.
  - With mmll = 0 and etaj = 5 it is refused as `UncutCollinearSplitting{pdg_a −11, pdg_b 11}`, which is the γ* → e+e- check you asked me to keep.
- **Mutation:** I restored the exemption by adding `&& model.model().particle(prop).spin != 3` to the InitialCollinear arm. `the_equivalent_photon_limit_is_refused` then FAILS: it panics at its `unwrap_err` because the card compiles. The other 6 new tests pass. Log: `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-mutation-a.log`.

### 2. Row retired (249dd1a0)
Removed:
- the `validation/manifest.toml` row;
- `vibegraph-cli/tests/validate_ewk_lljj_sigma.rs` and its `[[test]]` in `vibegraph-cli/Cargo.toml`;
- the `validate-ewk-lljj-sigma` task and its `validate-deep` line in `pixi.toml`;
- the `@ewk-lljj` group, its place in `@all`, and the row name in the "card" kind's comment, all in `validation/madgraph/bank_seeded_sigma.sh`;
- `validation/madgraph/scripts/pp_to_epemjj_qcd0.mg5`;
- `validation/madgraph/pp_to_epemjj_qcd0_run_card.dat`.

The only reference left outside `research/kb/` is the manifest header's refdata "Cut 13" history note, which describes what the bundle holds.

### 3. Re-sweep (release-debug binary from 249dd1a0, all 62 seeded rows, neval 20000, niter 1)
- 62 rows: 61 ACCEPTED, 1 refused.
- Refused: `pp_to_epemjj_qcd0`, at subprocess `u u > e+ e- u u`, verbatim:
  > ERROR failed to decompose into flavour groups: cut compilation failed: final-state leg 2 (PDG 2) leaves incoming leg 0 (PDG 2) through a massless spacelike propagator in a kept diagram, and the run card leaves its collinear limit with the beam uncut: the propagator diverges as the leg becomes collinear with the beam, σ diverges logarithmically, and any cross section reported is set by how far the sampler reaches into it; regulate it with ptj > 0 or a rapidity maximum etaj
- Log: `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2/ic2-sweep.log`. The option-(b) run is kept beside it as `ic2-sweep.optionb.log`.

### 4. Colourless-exemption test (b2f3365d)
- **Test:** `an_exemption_of_a_massless_colourless_core_leg_is_refused`. It runs `e+ e- > mu+ mu- a` with a synthetic forced Z line, under `F = cut_decays`.
- **Which guard refuses it:** `refuse_unregulated_limits`. The old guard `refuse_ir_unsafe_exemption` passes both cases, and the test asserts that it does, because `massless_coloured_core` is 0 here.

| Exempt legs | What the exemption removes | Refused as |
|---|---|---|
| photon (`0b100`) | dral, pta | `UncutCollinearSplitting{0, 2, −13, 22}` (μ → μγ) |
| muon pair (`0b011`) | drll, dral | `UncutCollinearSplitting{0, 1, −13, 13}` (γ* → μ+μ-, reported first) |

- In both cases the regulator text reads "cut_decays = T (cut_decays = F leaves this decay product uncut)", and the same lines compile at `T = cut_decays`.
- **Mutation:** with the `refuse_unregulated_limits` call removed from `compile_with`, this test FAILS at its `unwrap_err`. Log: `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-mutation-colourless.log`.

**Is the old guard still needed?** Yes; it is not redundant, but I could not build a counterexample for it.
- It refuses an exempt massless coloured leg that is a core leg in any diagram, whether or not a divergence follows. `refuse_unregulated_limits` refuses only when that leg takes part in a reachable massless limit.
- So the old guard is the stricter of the two, and it runs first, so it is the one that reports on coloured legs.
- For colourless legs, the new guard is the only one that fires.
- I tried to construct a coloured core leg with no massless limit, for example `e+ e- > z u u~ / a, z > u u~`. vibegraph refuses those decay chains upstream as ambiguous ("identical particles between the decay and the rest of the process").
- So I have no measured case where only the old guard fires. On the hadron-collider cards each such coloured core leg also had an initial-collinear or final-collinear limit. I did not delete it.

### 5. Gates (on b2f3365d)
- **`cargo fmt --all --check`:** clean.
- **`cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings`:** clean on 249dd1a0, run directly, and again inside validate.sh on b2f3365d.
- **`cargo test --workspace --no-fail-fast`** (lean env): EXIT 0, 55 targets, 1993 passed, 0 failed, 26 ignored. Log: `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-wstest.log`.
- **`bash validation/validate.sh`:**
  - EXIT 0, 84 test results, 2443 passed, 0 failed, 110 ignored. That is one target fewer than before, the retired `validate_ewk_lljj_sigma`.
  - Collator: "76 rows x 4 categories: the measured cells are the declared cells".
  - No ❌ cell in `target/validation-report/report.md` (the one ❌ is the legend), and no `pp_to_epemjj` mention in the report.
  - Log: `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-validate.log`. The option-(b) runs are kept as `*.optionb.log`.

### Found (adds to the first report's list)
6. **The row needs a regulated re-bank.** `pp_to_epemjj_qcd0` should come back on a regulated card (an etaj or ptj cut) to keep coverage of the peripheral, no-fiducial-scale regime. Its refdata seed record stays in the bundle unused. This is the follow-up you said you would file.
7. **The old guard is broader than the physics needs.** `refuse_ir_unsafe_exemption` refuses an exempt coloured core leg whether or not any limit makes it divergent. It is a candidate for restating over `limit_regulated` once a case is found where only it fires, or once it is proven to have none.

## I-C2 addendum 2: the xqcut false refusal is fixed, and all gates are green on `2b26c109`

The guard now counts MLM's per-event xqcut veto as a regulator for coloured jet–jet splittings, so the refused matched cards compile. The zero-τ semantic conflict from merging #111 is fixed. On the final head `2b26c109` the 62-row sweep, `cargo test --workspace` and `validate.sh` all pass. Nothing is uncommitted, and I pushed nothing after `2b26c109`.

### Commits on `sprinti-cuts`
Each is authored by Nicholas Smith with the single trailer `Assisted-by: claude-code:claude-opus-5-5`; all are pushed and none is force-pushed.
- `aee3812e` fix(cuts): count MLM's xqcut clustering veto as a jet-pair regulator. Touches `vibegraph-lib/src/cuts.rs` and `vibegraph-lib/src/hadronic.rs`.
- `e4fddba9` Merge main into sprinti-cuts: `--no-ff` of `origin/main` at `2803c0e9` (#111). No textual conflict.
- `2b26c109` test(proton): reach the zero tau floor past the beam-collinear guard. Touches `vibegraph-lib/src/proton.rs`.

### Correction to the brief
**The suggested test card can't reach the guard.** The card was `auto_ptj_mjj = F`, `xqcut = 20`, `ptj = 0`, `mmjj = 0`, `drjj = 0`.
- `runcard/matching.rs` refuses any resolved `ptj` other than `xqcut`. A `ptj` below xqcut is refused, and one above it is zeroed and then refused (`XqcutAboveJetThreshold`).
- So with `ptj = 0` the run card is refused before the cut compilation ever runs.
- Every xqcut card that does compile therefore has `ptj = xqcut`. The tests use `ptj = 20` and assert that `ptj = 0` is refused by the run card.

**My first report's xqcut claim was wrong.** It said the kT clustering cut is not implemented; `setclscales.rs:347-351` implements it.

### Derivation
1. **When the veto applies.**
   - MadEvent runs `setclscales` on every event from `update_scale_coupling_vec` (`reweight.f:1907`), and under `ickkw > 0` also from `rewgt`.
   - Its xqcut block (`reweight.f:1063-1077`) runs whenever `xqcut > 0`, whatever `ickkw` or the scale choice is. It fails the event if any merge has a daughter that is a single external final-state leg with `iqjets > 0` and `√pt2ijcl < xqcut`.
   - vibegraph does the same in `setclscales.rs:347-351`. `compile_scale_source` builds the clustering prescription whenever `xqcut > 0`, even when every scale is fixed (`hadronic.rs` test "a card fixing every scale still has a per-event prescription"). `scales.rs:362` refuses `xqcut > 0` on fixed beams and decays.
   - New predicate: `kt_veto_applies(rc) = xqcut > 0 && lpp1 != 0 && lpp2 != 0`.
2. **Which limits the veto bounds.** I measured this per event and found the veto depends on the configuration.
   - The integrand clusters each event in a configuration it draws from `AMP2` (`proton.rs` `per_group_sum` → `scale_channel`), or from the propagator products at `sde_strategy = 2`.
   - **Configurations whose diagram splits the jet pair at one vertex:** the pair's kT measure goes to zero, so it is merged first, at a jet vertex (`is_jet_vertex`: QCD mother and daughters, a jet daughter and a jet mother). That tags the jets, and the event is vetoed. The first merge is never one whose `pt2` a transverse-mass override replaces.
   - **Other configurations:** the pair is clustered elsewhere and the event is kept. Their share of the `AMP2` draw vanishes as the pair's invariant does, as fast as |M|² grows, so they leave the limit integrable.
   - Probe trace for `u u~ > e+ e- d d~` with the pair ΔR = 0.05 apart: in configurations 0–3 the merges are (ee) → (Z, d̄) → … and nothing is vetoed. In configurations 4–7 the first merge is (d, d̄) at pt2 = 2.25 GeV², with iqjets(d) = iqjets(d̄) = 1, and the event is vetoed.
3. **What the veto does not cover.**
   - A splitting off a colourless line (`γ* → q q̄`), because it is not a jet vertex.
   - A photon: `q → q γ` fails the `isqcd` test of `isjetvx`.
   - Leptons, b-class or other non-jet legs, and decay products exempted by `cut_decays = F`.
   - Beam-collinear and soft limits need no veto clause, because every compiling xqcut card has `ptj = xqcut`.
4. **The rule.** `FinalCollinear` now carries `coloured`, which says whether the propagator carries colour. A pair split off a coloured line in one diagram and a colourless line in another gives two limits, each checked separately. `kt_veto_regulates` counts `FinalCollinear{coloured: true}` as regulated when both legs are jet-class (`Letter::Jet`) and neither is an exempt decay product. For a coloured jet pair the regulator text now reads "drjj > 0, mmjj > 0 or xqcut > 0".

### Tests and mutations
| Test | What it shows | Mutation and result |
|---|---|---|
| `hadronic::tests::the_xqcut_veto_fails_every_event_near_a_coloured_jet_splitting` | On `auto_ptj_mjj = F`, `xqcut = ptj = 20`, `mmjj = drjj = 0`, an event with the pair ΔR = 0.05 apart at pT = 30 is vetoed in every configuration whose diagram has the pair at a vertex (`u u~ > e+ e- d d~`, `u g > e+ e- u g`, `u u~ > g g g`). At ΔR = 2 no configuration vetoes. For `u u~ > a u u~` (photon and quark collinear) no configuration vetoes. | Veto block disabled (`if false && settings.xqcut > 0.0`): FAILS ("u u~ > e+ e- d d~, configuration 4: Ok(Scales(...))") |
| `cuts::tests::the_xqcut_veto_regulates_a_coloured_jet_splitting` | The matched card compiles all three processes. The same cuts without xqcut are refused with "drjj > 0, mmjj > 0 or xqcut > 0". It also asserts that `ptj = 0` is refused by the run card. | `kt_veto_regulates` disabled: FAILS; the other 8 cuts tests pass |
| `cuts::tests::the_xqcut_veto_leaves_photon_and_colourless_splittings_to_their_cuts` | On the matched card, `u d > a u d` at `draj = 0` is still `UncutPhotonSplitting`. `u u~ > e+ e- d d~ QED=4` has `FinalCollinear{2,3,coloured:false}` and is refused with "drjj > 0 or mmjj > 0". | Veto widened to "any leg a jet, any propagator": FAILS |

Logs:
- cuts mutations: `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-mutation-xqcut.log`
- veto-block mutation output: `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-mutation-xqcut.log`

**Zero-τ semantic conflict with #111** (`2b26c109`, `proton::tests::a_card_with_no_shat_floor_is_refused_naming_its_uncut_legs`):
- **Both refusals pinned:** the `ptb = 0`, `etab = -1` card is now asserted to be refused by the guard as `ProtonError::Cut(CutError::UncutInitialCollinear { pdg: ±5 })`. `ZeroTauFloor` is reached with `ptb = 0`, `etab = 2.5`, and the original assertions are kept: `ZeroTauFloor`, the text names "leg 3 (PDG 5)" and "leg 4 (PDG -5)", and `ptb = 20` is accepted.
- **Why the guard accepts the etab = 2.5 card:**
  - the rapidity maximum regulates the beam-collinear limits (b off a gluon or b beam through a massless spacelike line);
  - two outgoing legs give no final-state limit;
  - a quark has no soft limit;
  - ŝ has no lower bound.
- **Verified:** the test passes, so the card compiles and the integrand refuses it with `ZeroTauFloor`.
- **The card route stays:** `ZeroTauFloor` is still reachable through a card, so no direct backstop test was needed.

### Gates
| Gate | Head | Result | Log |
|---|---|---|---|
| fmt and workspace clippy (`-D warnings`, extended-validation features) | `aee3812e` | clean; clippy also passes again inside `validate.sh` on `2b26c109` | — |
| 62-row sweep | `e4fddba9` | 61 ACCEPTED; `pp_to_epemjj_qcd0` refused with the same `UncutInitialCollinear` text as before. `2b26c109` changes only a test, so this holds for the final head. | `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2/ic2-sweep.log` |
| `cargo test --workspace --no-fail-fast` | `2b26c109` | EXIT 0; 56 targets, 2011 passed, 0 failed, 26 ignored | `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-wstest.log` |
| `bash validation/validate.sh` | `2b26c109` | EXIT 0; 85 test results, 2459 passed, 0 failed, 110 ignored; collator "76 rows x 4 categories: the measured cells are the declared cells"; the only ❌ in `report.md` is the legend | `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic2-validate.log` |

The full run on `e4fddba9` was stopped once the conflict appeared and rerun on `2b26c109`.

I did not run an end-to-end CLI integration of a matched `auto_ptj_mjj = F` card. The acceptance is pinned at the cut-compilation and per-event level.

### Corrected xqcut paragraph for the PR text (ready to paste)
> **MLM's xqcut.** `xqcut` regulates jets in two ways, and the guard reads both. Its rewrites reach the compiled cuts: the run card leaves every `xqcut` card with `ptj = xqcut` (it refuses any other resolved jet threshold), which regulates every beam-collinear and soft jet limit, and under `auto_ptj_mjj = T` it also sets `mmjj = xqcut`. Its per-event clustering veto, which vibegraph implements as MadEvent does (`setclscales.rs`, `reweight.f:1063-1077`) on every event with `xqcut > 0` on hadron beams, whatever `ickkw`, fails an event whose clustering merges a jet below `xqcut`. The integrand clusters in a configuration drawn from `AMP2`. Configurations whose diagram splits a coloured jet pair at one vertex merge the pair first and veto it near its collinear limit. The others carry an `AMP2` share that vanishes there. So on an `xqcut` card, a final-state splitting of two jet-class legs off a coloured line (`q → q g`, `g → g g`, `g → q q̄`) counts as regulated without `drjj` or `mmjj`, which matters under `auto_ptj_mjj = F`. The veto does not reach a splitting off a colourless line (`γ* → q q̄`), a photon (`q → q γ` is not a jet vertex), a lepton or a b-class leg, so those still need their own ΔR or mass cut. A per-event test pins the veto in each splitting configuration of `g* → q q̄`, `q → q g` and `g → g g`, and its absence for `q → q γ`.

### Found
8. **The veto's coverage is pinned on three topologies, not proven in general.** The `iqjets` bookkeeping in `final_state_vertex` can clear a quark's jet tag at a later colourless final-state vertex. I have not proven that, for a `g* → q q̄` pair whose quark line later meets such a vertex (for example `p p > j j j j` with an s-channel Z/γ* to quarks), the veto still fires in every splitting configuration. What closes it: extend the per-event test over the processes the CMS census exercises, or derive the `iqjets` invariant.
9. **Coloured and colourless splittings of the same pair are judged separately.** A coloured jet pair that some diagram also splits off a colourless line (`γ* → q q̄` at QED ≥ 4) still needs `mmjj` or `drjj` on an xqcut card. Configurations with the γ* mother may not veto, so I kept this refusal conservative. A per-event probe of those configurations would decide whether it is a false refusal.
