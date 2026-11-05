---
type: Session Report
title: "H-J1 report: with no run card, maxjetflavor follows the incoming partons as in the card MadGraph writes"
description: "The absent-run-card default for a scattering process is now RunCard::scattering_default over the incoming PDG ids of the generated subprocesses (those with a diagram): maxjetflavor = max(4, heaviest incoming quark below 7) when an incoming parton is a quark up to b, a gluon or a photon, as banner.py's create_default_for_process writes it. asrwgtflavor is left at 5: MadGraph sets it in memory only, never writes it (a hidden parameter not in user_set), and reads back 5 at launch; the item's closes_when asked for both. MadGraph's own output cards give 5 for p p > j j and b b~ > z under sm-no_b_mass, 4 for p p > j j under sm and for p p > w+ under sm-no_b_mass. p p > j j under sm-no_b_mass with no run card: vibegraph 698.834 ± 0.263 µb against MadEvent (hard_survey = 2) 699.026 ± 0.298 µb, −2.7e-4, pull −0.48, five seeds each, chi2/dof 0.89 and 0.66; before the change the same card stopped at the CutIndicatorDiffers grouping refusal. Outgoing b quarks in both samples have pT ≥ 20 GeV and |eta| ≤ 5, at 1.69% and 1.66% of outgoing partons."
status: draft
tags: [run-card, cuts, five-flavour, madgraph-parity, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-05}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: mg-default, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4767", title: "MadGraph banner.py RunCardLO.create_default_for_process"}
  - {id: mg-write, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L3010", title: "MadGraph banner.py RunCard.write, to_write = user_set"}
  - {id: mg-export, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L264", title: "MadGraph export_v4.py, create_default_for_process over the matrix elements' processes"}
  - {id: survey-bias, resource: "../../../validation/madevent-default-survey-bias.md", title: "MadEvent's default survey settings bias σ low"}
---
Session H-J1 of sprint H, item `absent-run-card-maxjetflavor-ignores-beam-flavours`.
Worktree `/home/user/wt-mjf`, branch `sprinth-maxjetflavor` off `origin/main`
`1c4e628c`, verified before any action:

```
$ cd /home/user/wt-mjf && git rev-parse --show-toplevel && git branch --show-current && git log --oneline -1
/home/user/wt-mjf
sprinth-maxjetflavor
1c4e628c Sprint G (3/3, optional): CLI summary lines that mislead (#90)
```

Scratch, none committed: `/home/user/H-J1-scratch/` (`mg/` MadGraph scripts,
logs and kept cards under `mg/keep/`; `vg/` vibegraph cards and logs;
`H-J1-*.log`, `H-J1-mutate.py`, `H-J1-stats.py`, `H-J1-bpt.py`).

## 1. What MadGraph's default card does, from source and from its own output

`RunCardLO.create_default_for_process` (`banner.py:4767`) is called by
`export_v4.py:264-279` with `processes = [me.get('processes') ...]`: the
processes of the generated matrix elements, so a flavour combination with no
diagram does not enter. For two incoming particles it collects `beam_id`, the
ids of every non-final leg, and (`banner.py:4806-4810`):

```python
if any(i in beam_id for i in [1,-1,2,-2,3,-3,4,-4,5,-5,21,22]):
    maxjetflavor = max([4]+[abs(i) for i in beam_id if  -7< i < 7])
    self['maxjetflavor'] = maxjetflavor
    self['asrwgtflavor'] = maxjetflavor
```

**`asrwgtflavor` does not reach the card MadEvent runs.** It is a hidden
parameter (`add_param("asrwgtflavor", 5, hidden=True, ...)`, `banner.py:4290`),
the assignment above does not add it to `user_set` (`__setitem__`'s
`change_userdefine=False`, `banner.py:1086`), and `RunCard.write` writes the
hidden parameters in `user_set` only (`to_write = set(self.user_set)`,
`banner.py:3015`). `launch` reads `Cards/run_card.dat` back
(`madevent_interface.py:5725`), so the card MadEvent runs holds the template 5
whatever the beams. Measured on MadGraph's own output (script
`mg/out_check.mg5`: `output madevent` of each card, no launch), read back with
MadGraph's own `RunCard`:

```
$ grep maxjetflavor H-J1_*/Cards/run_card.dat     # (one line per output)
H-J1_ppw_5f    (sm-no_b_mass, p p > w+)   4 = maxjetflavor
H-J1_ppjj_5f   (sm-no_b_mass, p p > j j)  5 = maxjetflavor
H-J1_ppjj_4f   (sm,          p p > j j)  4 = maxjetflavor
H-J1_bbz_5f    (sm-no_b_mass, b b~ > z)   5 = maxjetflavor
$ grep -c asrwgtflavor H-J1_ppjj_4f/Cards/run_card.dat
0
$ python3 -I -c "...banner.RunCard(d+'/Cards/run_card.dat')..."
H-J1_ppjj_4f maxjetflavor 4 asrwgtflavor 5
H-J1_ppjj_5f maxjetflavor 5 asrwgtflavor 5
H-J1_ppw_5f maxjetflavor 4 asrwgtflavor 5
```

`p p > w+` under `sm-no_b_mass` is the case that separates "the generated
subprocesses" from "the `p` label": `p` holds the b, but no b-initiated
subprocess makes a W+ under the diagonal CKM, and MadGraph writes 4.

**The brief's and the item's `asrwgtflavor` clause is wrong.** Following it
would have set `asrwgtflavor = 4` for every four-flavour process with no run
card, where MadEvent runs 5; that is a new difference for the MLM α_s
reweighting (`is_parton`, `coupling/cluster/rewgt.rs:302`), not a fix. vibegraph
keeps the template 5, which is what MadEvent runs in both schemes. A mutation
(M4 below) pins it.

### Everything else `create_default_for_process` sets from the process

| Rule (`banner.py`) | vibegraph's absent card |
|---|---|
| `ninitial == 1`: `remove_all_cut`, `use_syst = F`, `SDE_strategy = 1` (`:4785-4788`, `:5045`) | followed (`RunCard::decay_default`, unchanged) |
| hadronic beams: `maxjetflavor` (`:4806-4809`) | **followed now** |
| hadronic beams: `asrwgtflavor` (`:4810`) | in-memory only in MadGraph, see above; template 5 kept |
| lepton beams: `lpp = 0`, `ebeam = 500`, `use_syst = F` (`:4835-4840`); e p: one `lpp = 0` beam at 1 TeV (`:4815-4825`); neutrino beams `polbeam` (`:4893-4920`); EVA beams (`:4847-4890`) | **not followed**: Found 1 |
| mixed multiplicity, all extra legs jets: `ickkw = 1`, `xqcut = 30`, `drjj = drjl = 0`, `sys_alpsfact`, `--alps`, `dynamical_scale_choice = -1` (`:4924-4966`) | **not followed** (documented in `run-card/matching-parameters.md`): Found 2 |
| `pdgs_for_merging_cut = colored_pdgs` (`:4781`) | parameter is `IgnoredBenign` in `runcard/classes.rs:399` |
| `^2` interference: `use_syst = F`, `dynamical_scale_choice = 3`, `sde_strategy = 2` (`:4970-4988`) | not applicable to a card vibegraph accepts (not checked further) |
| single-colour-flow `sde_strategy = 2` (`:4994-5014`); `$` forces 1 (`:5055-5059`) | template 1 kept; see Found 3 |
| loop-induced `nhel = 1` | not applicable (tree level) |
| polarized massive legs: display `frame` block only | no value changes |

MadGraph's output for the rules not followed (`mg/out_check2.mg5`):

```
== H-J1_ppww      (sm, p p > w+ w-)                           1 = lpp1, 1 = sde_strategy, 4 = maxjetflavor
== H-J1_ppee01j   (sm, p p > e+ e- ; add process p p > e+ e- j) 1 = ickkw, 30.0 = xqcut, 0.0 = drjl, '--alps=0.5,1,2' in systematics_arguments
== H-J1_eemumu    (sm, e+ e- > mu+ mu-)                       0 = lpp1, 500.0 = ebeam1, False = use_syst
```

## 2. Implementation

- `RunCard::scattering_default(incoming)` (`vibegraph-lib/src/runcard.rs`): the
  LO defaults with `maxjetflavor` set by the rule above; `asrwgtflavor`
  untouched; the `systematics_arguments` line marked written as in
  `RunCard::default`.
- `GlobalConfig::load_run_card(parsed, model, pool)`
  (`vibegraph-lib/src/config.rs`): a `--run-card` file is parsed and returned
  as is (an explicit value always wins); without one, a decay gets
  `decay_default`, and a scattering process is enumerated
  (`generate_from_proc_card_in`) and gets `scattering_default` over the PDG ids
  of the incoming legs of every `DiagramSet` with at least one diagram.
  `load_decay_run_card` is folded into it. New error type `LoadRunCardError`.
- `vibegraph-cli/src/integrate.rs::load_run_card` passes the model and the
  enumeration pool; the `--run-card` help text, `docs/src/cli/reference.md`
  (regenerated with `scripts/gen-cli-docs.sh`) and `docs/src/cli/overview.md`
  say what the absent default is.

The beam content comes from the enumeration rather than from the `p`
definition, so the proc card's own `define p`, the model's default `p`, and
subprocesses with no diagram are all handled as MadGraph handles them. The cost
is one extra enumeration on the no-run-card path only (`p p > j j j`: 0.08 s
serial per the `EnumerationPool` measurement; `p p > e+ e- j j j` 3.4–8.9 s);
`generate` reads the card from the artifact, so it never re-derives it.

## 3. Tests and mutation evidence

New tests in `config::tests`:

- `absent_card_takes_the_jet_flavours_from_the_beams`: `p p > j j` →
  `(maxjetflavor, asrwgtflavor) = (5, 5)` under `sm-no_b_mass`, `(4, 5)` under `sm`.
- `absent_card_counts_only_generated_subprocesses`: `p p > w+` under
  `sm-no_b_mass` → `(4, 5)`; `b b~ > z` → `(5, 5)` (both measured on MadGraph above).
- `absent_card_without_incoming_partons_keeps_the_template_flavours`:
  `e+ e- > b b~`, the decay `z > b b~`, and `t t~ > z` (no parton up to b,
  gluon or photon, so MadGraph's guard keeps 4) → `(4, 5)`.
- `an_explicit_card_keeps_its_jet_flavours`: under `sm-no_b_mass`, a card with
  `4 = maxjetflavor` and `4 = asrwgtflavor` → `(4, 4)`, an empty card →
  `(4, 5)`; under `sm`, `6 = maxjetflavor` → `(6, 5)`.
- `load_run_card_defaults_when_absent` adapted to the new signature.

```
$ cargo test -p vibegraph-lib --lib config::
test config::tests::absent_card_counts_only_generated_subprocesses ... ok
test config::tests::an_explicit_card_keeps_its_jet_flavours ... ok
test config::tests::absent_card_without_incoming_partons_keeps_the_template_flavours ... ok
test config::tests::load_run_card_defaults_when_absent ... ok
test config::tests::absent_card_takes_the_jet_flavours_from_the_beams ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1242 filtered out
```

Mutations (`H-J1-mutate.py`: apply, `cargo test -q -p vibegraph-lib --lib
config::`, restore; `H-J1-mutations.log`):

```
## M0 none (baseline): exit 0
   test result: ok. 9 passed; 0 failed
## M1 absent card is the template card (the code before the change): exit 101
   ---- config::tests::absent_card_counts_only_generated_subprocesses stdout ----
   ---- config::tests::absent_card_takes_the_jet_flavours_from_the_beams stdout ----
## M2 beam ids include subprocesses with no diagram (multiparticle expansion): exit 101
   ---- config::tests::absent_card_counts_only_generated_subprocesses stdout ----
## M3 the beam rule overrides an explicit card: exit 101
   ---- config::tests::an_explicit_card_keeps_its_jet_flavours stdout ----
## M4 asrwgtflavor follows maxjetflavor (banner.py's in-memory value): exit 101
   ---- config::tests::absent_card_counts_only_generated_subprocesses stdout ----
   ---- config::tests::absent_card_takes_the_jet_flavours_from_the_beams stdout ----
## M5 no hadronic-parton guard: exit 101
   ---- config::tests::absent_card_without_incoming_partons_keeps_the_template_flavours stdout ----
```

M1 is the code before the change (the absent card was `RunCard::default()` for
every scattering process), so the 5F test fails before it. The explicit-card
test cannot fail on the old code (an explicit card was already honoured); M3
is the mutation it is there to catch. What these tests cannot see: they pin
the values against my reading of MadGraph's output above, not against a
committed MadGraph dump (Found 4).

## 4. σ and per-event oracle against local MadGraph

Card: `import model sm-no_b_mass` / `generate p p > j j`, no run card on
either side. MadGraph: `output madevent H-J1_ppjj_5f`, then one fresh copy of
that clean output per seed, `launch` with only `iseed`, `nevents 20000`,
`hard_survey 2`, `pdlabel lhapdf`, `lhaid 247000`, `use_syst False` set
(`mg/H-J1-mg-sigma.sh`); the card MadGraph wrote at `output` has
`5 = maxjetflavor`, and every launched card kept it:

```
s11: 1 iseed-set,  5 = maxjetflavor,  2 = hard_survey      (same for s12..s15)
```

vibegraph: `vibegraph integrate ppjj_5f.dat --out ... --seed N`, no
`--run-card`, default 0.1 % target, seeds 1-5 (`vg/H-J1-vg-sigma.sh`, release
build of this branch). σ from MadGraph's `SubProcesses/results.dat` and
vibegraph's summary line:

```
$ python3 H-J1-stats.py
mg       n=5 mean 6.990260e+08 pb  sd 6.656e+05 (9.52e-04)  se(scatter) 2.977e+05  se(quoted) 3.714e+05  chi2/dof 0.66
         seeds: 6.98460e+08±8.9e+05, 7.00100e+08±8.0e+05, 6.98880e+08±8.1e+05, 6.98520e+08±8.2e+05, 6.99170e+08±8.4e+05
fix      n=5 mean 6.988340e+08 pb  sd 5.879e+05 (8.41e-04)  se(scatter) 2.629e+05  se(quoted) 2.864e+05  chi2/dof 0.89
         seeds: 6.98950e+08±6.8e+05, 6.99020e+08±6.4e+05, 6.98540e+08±6.4e+05, 6.99620e+08±6.2e+05, 6.98040e+08±6.2e+05
fix-mg: -1.9200e+05 pb (-2.747e-04); pull -0.48 (scatter), -0.41 (quoted)
```

**The known-wrong side.** The absent default before the change is the
template card, which an empty `--run-card` reproduces exactly. With it the
same card does not integrate:

```
$ vibegraph integrate ppjj_5f.dat --run-card empty_run_card.dat --seed 1 ...   # exit 1
ERROR failed to decompose into flavour groups: subprocesses g g > u u~ and g g > b b~ share a
matrix element but compile to different cut filters, so no single cut indicator covers the sum
```

**Per event.** 5000 events from the seed-1 artifact (`vibegraph generate
--seed 7 --nevents 5000`) and MadGraph seed 11's 20000 unweighted events,
outgoing partons (`H-J1-bpt.py`):

```
vibegraph  b: 169 outgoing, min pT 20.051 GeV, max |eta| 4.921, pT<20: 0
           light: 9831 outgoing, min pT 20.001 GeV, max |eta| 4.996, pT<20: 0
MadGraph   b: 663 outgoing, min pT 20.006 GeV, max |eta| 4.963, pT<20: 0
           light: 39337 outgoing, min pT 20.000 GeV, max |eta| 4.999, pT<20: 0
```

The b quarks take the jet cuts on both sides (`ptj = 20`, `etaj = 5`; with
`maxjetflavor = 4` they would have `ptb = 0`, `etab = -1`), at 1.69 % ± 0.13 %
and 1.66 % ± 0.06 % of outgoing partons. vibegraph's event file records
`5 = maxjetflavor` and `5 = asrwgtflavor` in its `<MGRunCard>`.

### Effect on `CutIndicatorDiffers`

The change takes every five-flavour card run *without* a run card out of
`CutIndicatorDiffers` for the b/light split (the template card's
`maxjetflavor = 4` was what put `g g > u u~` and `g g > b b~` under different
filters; `p p > j j` above is the measured case). A card that sets
`maxjetflavor = 4` explicitly under a massless b still reaches it, so item 8
(`five-flavour-ppjjj-refused-cut-indicator-differs`) keeps its scope for
explicit cards; its text that the refusal is reached "with no run card until
the default is fixed" no longer holds.

## 5. Gates

```
$ cargo fmt --all --check                                        # exit 0
$ cargo clippy --workspace --all-targets -- -D warnings          # Finished ... EXIT 0
$ cargo test --workspace                                         # EXIT 0
  51 test binaries, "test result: ok" in all; passed 1926, failed 0, ignored 25
  test committed_cli_reference_matches_the_binary ... ok
```

`bash validation/validate.sh`, detached (`nohup setsid`), `CARGO_TARGET_DIR=/home/user/H-J1-scratch/validate-target`,
an untracked `.cargo/config.toml` with `[profile.release-debug] debug = 0`
(deleted afterwards); log `H-J1-validate.log`:

```
validate exit=0
84 test binaries: passed 2380, failed 0, ignored 110
77 rows × 4 categories = 308 cells: 218 measured in the layers this run drove (213 ✅, 5 ⚠️, 14 ⏳, 76 — / uncovered).
The measured cells are exactly the cells the manifest declares, every gate cell
passed, and every measurement agrees with the manifest about what it is.
```

The five ⚠️ are the manifest's deliberately unenforced cells (`gg_to_gg`
diagrams 4/6, `ee_to_wpwm_cw` and `ee_to_zh_smeft` amplitudes,
`gg_to_gg_cg` diagrams 21/27, `ee_to_mumua` samples KS); every gate cell
passed with the change in place (whether any of them reads the absent-card default was not audited).

## Found

1. **No run card on lepton beams gives proton beams and a panic.**
   MadGraph writes `lpp = 0`, `ebeam = 500`, `use_syst = F` for `e+ e- >
   mu+ mu-` (`H-J1_eemumu` above; `banner.py:4835-4840`; also the e p,
   neutrino and EVA rules). vibegraph's absent card keeps `lpp = 1`, and
   `vibegraph integrate` on `generate e+ e- > mu+ mu-` with no run card
   panics: `thread 'main' panicked at vibegraph-lib/src/proton.rs:945:25:
   initial-state parton -11 is not a tabulated parton density` (exit 101;
   same with an explicit empty card, so the panic predates this change;
   `vg/H-J1-ee.log`). Closing it: apply the lepton-beam rules in
   `scattering_default` from the same incoming ids, and turn the panic into a
   refusal for an explicit `lpp = 1` card with leptons in the initial state.
2. **No run card on a mixed-multiplicity jet card runs unmatched where
   MadGraph matches.** `p p > e+ e-` + `add process p p > e+ e- j` gets
   `ickkw = 1`, `xqcut = 30`, `drjl = 0`, `--alps=0.5,1,2` from MadGraph's
   output (`H-J1_ppee01j`, `banner.py:4924-4966`); vibegraph's absent card
   keeps `ickkw = 0`. `run-card/matching-parameters.md` records it as "vibegraph
   reads the card it is given", but with no card given it is a silent
   difference in σ and events. Closing it: the same rule in
   `scattering_default` from the generated subprocesses' legs, with a σ row
   against MadGraph on `p p > e+ e-` 0+1j.
3. **`SDE_strategy` and the single-colour-flow rule are unverified.**
   `banner.py:4994-5014` moves `sde_strategy` to 2 for single-colour-flow
   processes outside two exceptions, and vibegraph consumes `SDE_strategy`
   (`R_SDE_STRATEGY`: configuration weights and the clustering scale source).
   The one card tried, `p p > w+ w-`, writes 1 (as vibegraph's template), so no
   difference was observed; a card that does get 2 was not found. Closing it:
   a census of MadGraph's written `sde_strategy` over the survey's cards.
4. **The default-card values are pinned by tests against hand-read MadGraph
   output, not a committed dump.** `validation/madgraph/dump_runcard_defaults.py`
   dumps the template and the decay card; it could also dump
   `create_default_for_process`'s written card for a few process
   definitions (5F/4F `p p > j j`, `p p > w+`, `e+ e- > mu+ mu-`, a mixed
   multiplicity), so a MadGraph bump that changes the rule fails a test.
5. **Item 8's link breaks with this item's deletion.** `python3 scripts/kb.py
   lint`: `error: research/kb/backlog/feature/five-flavour-ppjjj-refused-cut-indicator-differs.md:
   link to ../validation/absent-run-card-maxjetflavor-ignores-beam-flavours.md
   does not resolve`. Item 8 is not mine to edit; it needs its `sources` entry
   and body link retargeted (to this report) and its "with no run card" sentence
   updated per §4.
6. **A dead-code warning in a release build.** `cargo build --release -p
   vibegraph` warns `method resolve_mom is never used`
   (`vibegraph-lib/src/helas/eval/analysis.rs:208`); not from this change, and
   not seen by `cargo clippy --workspace --all-targets` (it is used under test).
