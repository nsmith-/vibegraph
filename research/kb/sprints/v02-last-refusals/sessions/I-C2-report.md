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
retirement follow in an addendum.


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