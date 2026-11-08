---
type: Session Report
title: "I-R1 report: the default run card follows create_default_for_process; an untabulated beam parton is refused"
description: "With no run card, vibegraph builds MadGraph's written default card (MLM on mixed jet multiplicities, lepton beams, sde_strategy, beams refused by name); σ on p p > e+ e- + j matches MadGraph to +0.006%; untabulated partons are a typed refusal."
status: draft
tags: [run-card, defaults, mlm, beams, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-08}]
sources:
  - {id: c1, resource: "https://github.com/nsmith-/vibegraph/commit/8ff0d638", title: "8ff0d638 on sprinti-runcard"}
  - {id: c2, resource: "https://github.com/nsmith-/vibegraph/commit/59be16b4", title: "59be16b4 on sprinti-runcard"}
---
Manager's spot checks:
- re-averaging the per-seed MadGraph `results.dat` and vibegraph σ lines;
- `asrwgtflavor = 4` in MadGraph seed 11's launched banner;
- the tail of the `validate.sh` log (EXIT=0, no FAILED lines).

The vibegraph manager's review of #111 checked the port against `banner.py`.

One correction. `lhaid 247000` is NNPDF23_lo_as_0130_qed, the set MadGraph's
built-in `nn23lo1` also is, so the σ comparison below is like for like: the
same PDF set, read by two interpolators. The PDF caveat below reads them as
different sets.


With no run card, vibegraph now builds the default card MadGraph's `output` writes for the process. On `p p > e+ e-` + `add process p p > e+ e- j` it now matches MLM (matched σ against local MadGraph: pull +0.06). On `e+ e- > mu+ mu-` it gets lepton beams. An incoming particle with no parton density on a proton beam is now a typed refusal instead of a panic. All gates are green.

## Commits (branch `sprinti-runcard`, pushed)
- `8ff0d638` feat(runcard): absent-card defaults follow create_default_for_process; refuse untabulated beam partons
- `59be16b4` test(runcard): borrow the single-subprocess slice in the matching test (clippy fix)

Both are authored by Nicholas Smith and end with the single trailer `Assisted-by: claude-code:claude-opus-5-5`.

Files:
- `/home/user/wt/sprinti-r1/vibegraph-lib/src/runcard/process_defaults.rs` (new): `ProcessFacts`, the rules, and a module doc listing every rule and what is done with it.
- `/home/user/wt/sprinti-r1/vibegraph-lib/src/runcard.rs`: `scattering_default(&ProcessFacts) -> Result<RunCard, RunCardError>`.
- `/home/user/wt/sprinti-r1/vibegraph-lib/src/config.rs`:
  - `process_facts` enumerates each line on its own, plus the core of each decay-chain line.
  - New refusals `LoadRunCardError::FixedScaleModel` and `Colour`.
  - Tests.
- `/home/user/wt/sprinti-r1/vibegraph-lib/src/proton.rs`: `ProtonError::UntabulatedParton`, raised by `refuse_untabulated_partons` at the top of `derive_flavor_groups` (before anything is compiled), with a test.
- Fixtures in `/home/user/wt/sprinti-r1/vibegraph-lib/tests/data/mg_default_run_cards/`: `pp_ee_add_pp_ee_j.dat`, `ee_mumu.dat`, `smeft_gg_tt_add_gg_tt_g.dat`. Each is the `run_card.dat` MadGraph 3.7.1 (pinned b7687064) `output` wrote, with comment lines removed.

## Every rule of `create_default_for_process` for two incoming particles (banner.py:4767-5066)

Reproduced, and checked against MadGraph's written cards:
- **`maxjetflavor` from the beams.** Unchanged.
- **Lepton beams:** `lpp = 0`, `ebeam = 500`, `use_syst = F`.
- **MLM matching** (:4929-4967). Fires when some maximal-multiplicity matrix element, minus one occurrence of each id of the first minimal one, is all `±1..5, 21`. It is decided on `proc[0]` of each matrix element, which for a decay chain is its undecayed core. It sets:
  - `ickkw = 1`, `xqcut = 30`, `drjj = drjl = 0`, `--alps=0.5,1,2` appended, `dynamical_scale_choice = -1`.
  - With the `mlm` block displayed, `asrwgtflavor = maxjetflavor` and `pdgs_for_merging_cut` = the model's coloured PDG ids (ghosts excluded) are written too.
- **The `MLM` limitation** (a used coupling of a gluon vertex among coloured particles with no QCD order): `ickkw` goes back to 0 and `dynamical_scale_choice` from -1 to 3, while `xqcut` and `--alps` stay.
- **`sde_strategy`** is 2 only when all of these hold:
  - every line has an explicit QCD=0;
  - the colour basis is at a single power of Nc;
  - the final state is not all leptons/photons from partonic beams;
  - no line uses `$`.

Reproduced, then refused by the rule that already refuses the same card written by hand:
- `e p` (`lpp 0/1`, 1 TeV and 6.5 TeV) → `UnsupportedLpp`.
- Neutrino beams (`lpp = 0`, `ebeam = 1000`, `polbeam ∓100`) → the polbeam refusal, or `UnsupportedLpp` beside a proton.
- EVA with a W/Z on both beams (`lpp -3/3`, 15 TeV, `nhel = 1`, `pdlabel eva`) → `UnsupportedLpp`.
- EVA with a W/Z on one beam (`pdlabel1 = eva` at lpp 1/1) → `AsymmetricBeamPdf`.

Never taken:
- **Photon UPC.** `beam_id == [[22],[22]]` compares a set with a list and is always false, so MadGraph keeps proton beams too.
- **Interference (`^2`).** The proc card already refuses squared orders.

Not reproduced:
- **`fix_scale`** (a massless coloured scalar). MadGraph fixes both scales on the default card and refuses running scales at launch. With no run card, vibegraph refuses such a model by name (`FixedScaleModel`). I did not model the hidden-line read-back of `fixed_fac_scale` versus `fixed_fac_scale1/2`.

Display only: the frame, cut-class and `eva_pdf` blocks. They only decide which hidden lines are written.

The card built is the one MadEvent reads back at launch, not the one MadGraph holds in memory. A hidden parameter is written only when it is user-set or its block is displayed.

**Brief correction:** `sys_alpsfact` is set by the matching rule but is hidden and never written. It therefore reads back as the template's `None`, and stays `None` here. That is inert, because the default card's `systematics_arguments` line is the one read.

**On `ptj`, as the brief asked:** MadGraph's default card keeps `ptj = 20` with `auto_ptj_mjj = T`. `matching::resolve` raises it to `xqcut = 30`, as `setcuts.f` does, so `XqcutAboveJetThreshold` does not fire. The event record keeps 20, as MadGraph's banner does.

## Oracles

**1. Every field of the default card against MadGraph's written card.** The test compares every parameter both as resolved and as recorded in the event file. The PDF fields are excluded because MadGraph here has no LHAPDF and writes nn23lo1/230000.
- `absent_card_on_mixed_jet_multiplicities_is_madgraphs_matched_card`: no differences. It also pins by value `ickkw 1`, `xqcut 30`, `drjl 0`, `drjj 0`, `ptj 30`, `dyn -1`, `(maxjetflavor, asrwgtflavor) = (4, 4)`, the `--alps` list and `sys_alpsfact None`, and that the systematics request has `alps = [0.5, 1, 2]`.
- `absent_card_on_lepton_beams_is_madgraphs_lepton_card`: no differences. Pins `lpp (0, 0)`, `ebeam (500, 500)`, `use_syst F`.
- `absent_card_under_the_mlm_limitation_is_madgraphs_card` (SMEFTsim massless; `g g > t t~ NP<=1` + `g g > t t~ g NP<=1`): no differences, with `ickkw 0`, `xqcut 30`, `dyn 3`.
- `absent_card_on_unsupported_beams_is_refused_by_name`: `e- p > e- j` and `ve p > e- j` give lpp1=0, lpp2=1; `w+ w- > h` gives -3/3; `ve ve~ > e+ e-` gives polbeam1; `w+ d > u g` gives eva.
- `absent_card_sde_strategy_follows_madgraph`: each value is the one MadGraph wrote (local `output`):

| process | `sde_strategy` |
|---|---|
| `e+ e- > mu+ mu- QCD=0` | 2 |
| `e+ e- > u u~ QCD=0` | 2 |
| `e+ e- > u u~ u u~ QCD=0` | 2 |
| `u u~ > z QCD=0` | 2 |
| `p p > e+ e- QCD=0` | 1 |
| `e+ e- > u u~ $ z QCD=0` | 1 |
| `e+ e- > u u~ QED=2` | 1 |
| no orders | 1 |

The MadGraph cards came from `python3 mg5/bin/mg5_aMC gen{,2,3}.mg5`, run from a copy of the submodule in `scratchpad/ir1/`.

**2. σ with no run card** (on 8ff0d638). Driver: `scratchpad/ir1/sweep/run.sh`.
- **MadGraph:** `launch` (`bin/generate_events -f`) on the card `output` wrote. Only `iseed` was changed, plus `use_syst = F`, which does not touch σ. PDF nn23lo1 (built in; there is no LHAPDF). Each seed's launched banner holds `ickkw = 1` and `xqcut = 30.0`. `results.dat` σ ± err in pb:

| seed | σ (pb) |
|---|---|
| 11 | 1017.3 ± 2.40 |
| 12 | 1021.2 ± 2.29 |
| 13 | 1018.4 ± 2.27 |
| 14 | 1023.2 ± 2.10 |
| 15 | 1020.1 ± 2.45 |

- **vibegraph:** `vibegraph integrate proc.dat --seed N --fixed-budget --neval 400000 --niter 15`, with no `--run-card`, PDF NNPDF23_lo_as_0130_qed (LHAPDF 247000) from `validation/pdf`. A 20-event `generate` from seed 1 records in `<MGRunCard>`: `1 = ickkw`, `30.0 = xqcut`, `0.0 = drjl`, `0.0 = drjj`, `4 = asrwgtflavor`, `20.0 = ptj`, `-1 = dynamical_scale_choice`, and the `--alps` list. σ in pb:

| seed | σ (pb) |
|---|---|
| 1 | 1020.36 ± 0.81 |
| 2 | 1020.97 ± 0.71 |
| 3 | 1019.61 ± 0.70 |
| 4 | 1019.86 ± 0.70 |
| 5 | 1019.70 ± 0.71 |

- **Statistics:**

| side | mean (pb) | sd | sem | χ²/dof |
|---|---|---|---|---|
| MadGraph | 1020.04 | 2.32 | 1.04 | 1.09 |
| vibegraph | 1020.10 | 0.57 | 0.25 | 0.63 |

  vibegraph − MadGraph = +0.06 pb (+0.006%), pull +0.06.
- **PDF caveat:** sprint H measured nn23lo1 at +0.32% over LHAPDF 247000 on Drell-Yan. That would be about 3 pb here, about 3σ of the combined sem, and none is seen. On this matched card the offset is not reproduced, or is compensated; I did not resolve which.
- **Known-wrong row:** the pre-change default, reproduced with an empty `--run-card`, gives 1095.81 / 1095.80 / 1095.68 pb (seeds 1-3), +7.4% over MadGraph. So the feature going live is visible end to end.

**3. Untabulated parton.** `proton::tests::an_untabulated_beam_parton_is_refused` checks `e+ e- > mu+ mu-` and `ta+ ta- > mu+ mu-` on an explicit lpp=1 card, and `w+ d > u g`. Each gives `UntabulatedParton` naming e+, ta+ or W+, raised before compilation.

## Mutation evidence (each test fails on the change undone)
- **Refusal call removed, old panic restored:** the test panics with "initial-state parton -11 is not a tabulated parton density" (FAILED).
- **`apply` reduced to the old `maxjetflavor`-only rule:** five config tests FAILED (mixed, lepton, MLM-limitation, unsupported beams, sde). The existing `maxjetflavor` tests still pass.
- **`non_qcd_gluon_emission` forced false:** the MLM-limitation test fails with the diff `ickkw 1 vs 0`, `dynamical_scale_choice -1 vs 3`.

## Gates (59be16b4)
- `cargo fmt --all --check`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: Finished, no warnings.
- `cargo test --workspace`: WSEXIT=0. 52 binaries, 1994 passed, 0 failed, 25 ignored.
- `bash validation/validate.sh`: EXIT=0. Verification line: "77 rows × 4 categories = 308 cells: 218 measured in the layers this run drove (213 ✅, 5 ⚠️, 14 ⏳, 76 — / uncovered). The measured cells are exactly the cells the manifest declares, every gate cell passed, and every measurement agrees with the manifest about what it is." The 5 ⚠️ are the deliberately unenforced cells (gg_to_gg, ee_to_wpwm_cw, gg_to_gg_cg, ee_to_zh_smeft, ee_to_mumua).
- **Banked lines:** the banked gates build their cards explicitly (`RunCard::parse` / `RunCard::default()`, and the latter is unchanged). Every CLI test that calls `integrate` mentions a run card, but I did not check every invocation one by one. So as far as I can tell no banked card runs without a run card, and every gate passed. I did not diff σ lines bit for bit against main.

## Notes for the manager
- The default card now enumerates each process line on its own instead of the whole card at once. The cross-line duplicate-subprocess refusal therefore surfaces at integration rather than while building the card. A decay-chain line costs one extra core-only enumeration.
- "First minimal matrix element" follows vibegraph's generation order, which can differ from MadGraph's. It only matters when the lowest multiplicity mixes subprocesses that differ in more than jets.
- `single_color` is computed with `colorize_process`, and only when every line has an explicit QCD=0.

## Found
1. **An explicit card with no `sde_strategy` line gets 2 in MadGraph on single-colour processes.** At launch, `common_run_interface.py:6574-6580` (`check_card_consistency`) sets `SDE_strategy` from `single_color` whenever the card has no `sde_strategy` line (not user-set). vibegraph keeps the template's 1. This changes configuration draws (cluster scales, colour flows) for hand-written cards such as `p p > e+ e-`. Fix: apply the same rule when parsing a card that lacks the line, and test it against a MadGraph launch of such a card.
2. **MadGraph refuses some explicit cards at launch that vibegraph runs.** On a process with the `MLM` limitation, `check_card_consistency` refuses `dynamical_scale_choice = -1` or `ickkw = 1` (:6549-6556). On a `fix_scale` model it refuses running scales (:6563-6570). Fix: carry those limitations to the parsed card and refuse it by name.
3. **The `fix_scale` default is refused rather than reproduced.** Fix: model `fixed_ren_scale`/`fixed_fac_scale` with their hidden-line read-back, then compare against a MadGraph `output` of a model with a massless coloured scalar.
4. **Decays of mixed multiplicity.** `create_default_for_process`'s matching rule also runs when `ninitial == 1` (e.g. `z > j j` + `add process z > j j j`), but `decay_default` ignores it. Fix: decide whether matching a decay is honoured or refused, and test against MadGraph's written card.
5. **The +0.32% nn23lo1-over-LHAPDF offset from sprint H is not seen on this matched card** (pull +0.06 at about 0.1% resolution). Fix: a 20-seed comparison with MadGraph on `pdlabel lhapdf, lhaid 247000` would separate PDF provenance from a compensating difference.

Scratch evidence is in `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ir1/`: the MadGraph outputs, `sweep/ir1-mg.txt`, `sweep/ir1-vg.txt`, the `kw_s*` and `vg_s*` artifacts, and `../ir1-wstest.log` and `../ir1-validate.log`.