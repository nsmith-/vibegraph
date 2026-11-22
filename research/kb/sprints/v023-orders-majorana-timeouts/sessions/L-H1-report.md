---
type: Session Report
title: "L-H1 report: reweighting in each event's own helicity"
description: "change helicity True (MadGraph's default, now also what a card without the line means) writes |M_hyp(λ)|²/|M_card(λ)|² in the event's SPINUP combination; three per-helicity MadGraph rows agree event by event to the printed digits, the default-off path is byte-identical to main, and the plan refuses what the ratio cannot serve."
status: draft
tags: [reweight, helicity, spinup, madgraph-parity, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
Branch `sprintl-helicity-reweight` off main 4b132cc5 (brief at c512cec4).
Closes `reweight-helicity-summed-only`; its file is deleted. Commits: 614ca877
(the feature, the e+e- row), 74bae80a (the hadronic and SMEFT rows, the
classification page, the item's deletion), and the commit adding this report.

## What changed

**The ratio is a run option, read from the card.** `HelicityRatio`
(`reweight/engine.rs`) is `Summed` or `Event`, carried in
`ReweightOptions::helicity`. The card parser reads `change helicity`
(`ReweightCard::stated_helicity`, `ReweightCard::helicity`); the CLI adds
`generate --reweight-helicity summed|event`, which applies where the card is
silent and is refused (usage error naming both) where the card's line says
otherwise. `change helicity` after the first `launch` is a new refusal (below).

**The helicity reaches the reweighter.** Both selections already carried the
drawn helicities (`EventSelection::helicity`, `ProtonSelection::helicity`), the
proton one in physical leg order. `ProtonSelection` gains `cm_helicity`, the
same combination in the representative's leg order (the order of
`ProtonEvent::cm`, the momenta the reweighter evaluates), and both selections
gain `helicity_m2`, the drawn combination's `|M_c|²` off the diagonal the draw
used (`hadronic::drawn_helicity`). `Reweighter::ratios` takes the combination
(`Option<&[i32]>`, read only under `Event`) and returns a `CardPoint`
(summed `|M|²`, and the combination's under `Event`).

**One combination on both paths.** Exact path: `eval_hel_m2` per slot, entry
`combo`. Polynomial path: `node_gram(.., Some(combo))` restricts the
`R_jk = Re Σ CF conj(J_j) J_k` contraction to that combination's JAMP chunk;
`None` is the old loop over the whole vector, unchanged arithmetic. What PR 1's
graded evaluator gives is used as is: `n_jamps`/`cf_weights` (the `jamp_cf`
contraction) make the per-combination chunk `combo * n_jamps ..`, so a
squared-order split needs no special case; `is_signed` drives a refusal. It
offers no single-combination evaluation, so each event still evaluates every
combination and keeps one (cost, not correctness).

**The run's audit sees the hand-off.** Under `Event`, `ReweightAudit` holds
each event's card-point `|M|²` in its helicity to the integrand's
`helicity_m2` (same 1e-9 tolerance as the summed check), counts reweighter
refusals, and counts records with two identical outgoing particles under
different mothers; any of them refuses the file. A new log line reports the
three counts.

**What an omitted `change helicity` line means now: the per-event-helicity
ratio.** MadGraph starts with `self.helicity_reweighting = True`
(`madgraph/interface/reweight_interface.py` L112), and the backlog item's own
framing (a card without the line "asks MadGraph for the per-helicity ratio")
made the old behaviour a recorded divergence. Matching it is the only reading
under which the same card means the same weights in both generators; refusing
cards without the line would break every existing card for no physics reason.
**User-visible change:** a reweight card that `generate` accepted before and
has no `change helicity` line now produces different `<wgt>` values (same
events, same σ in expectation, larger spread). Adding `change helicity False`,
or `--reweight-helicity summed`, restores the old output byte for byte
(bit-for-bit below). A card with `change helicity True`, refused before, is now
accepted. A card with `change helicity` after its first `launch`, accepted
before, is now refused.

## MadGraph semantics, cited at the pin (3.7.1)

| question | MadGraph | here |
|---|---|---|
| default and option | `helicity_reweighting = True` L112; `change helicity` L402-L403 | `ReweightCard::helicity`, default `Event` |
| when the option applies | each `launch` reweights the whole file as read (`do_launch` L497-L545) | a `change helicity` after the first `launch` is refused |
| SPINUP order | `get_helicity(orig_order)` (`lhe_parser.py` L2426-L2465): legs placed by PDG code into the matrix element's (initial, final) order, the first unused slot of a repeated code, intermediates skipped, the charge-conjugate order tried when a code is missing | the reweighter is handed the combination in the order of the momenta it evaluates (fixed beams: process order; proton beams: `cm_helicity`, the representative order). The oracle test rebuilds that order from the banked file the way `locate` rebuilds the momenta |
| `9` (unknown) | `9 not in hel_order`, else `nhel = -1`: the whole event falls back to the summed ME (L1597-L1601) | refused by name (`HelicityRefusal::Unrecorded`): a silent fallback would mix the two ratios in one file. vibegraph never writes 9 on an external leg |
| `0` | an ordinary label (longitudinal vector) | an ordinary label |
| lookup | `hel_dict[tuple(hel_order)]`, built from `me.get_helicity_matrix()` (L1843-L1850); a combination outside it is a `KeyError` | `HelicityRefusal::Unknown` |
| crossing / initial state | none: SPINUP is MadEvent's `NHEL` row (`unwgt.f` L609 writes `jpart(7,..) = nhel(i)`), read back as the same label | the label is the evaluator's own `helicities()` entry, the one the record wrote; the three MadGraph rows (MadGraph reading vibegraph's SPINUP) are the test that the labels agree |
| frame | z-boost to the partonic CM (L1632-L1639); the exchanged ordering is evaluated with its first parton along −z, no rotation | the event's partonic CM, the exchanged event rotated by π about x; a rotation keeps every helicity (pinned by `pp_llj_hel`, both orderings banked) |
| one combination | `SMATRIXHEL` sets `USERHEL` (`matrix_standalone_v4.inc` L1-L31); only `IHEL = USERHEL` is summed (L134-L149) | `eval_hel_m2` entry, or the Gram contraction on that chunk |
| normalisation | `ANS/IDEN`, times `HELAVGFACTOR` when `USERHEL ≠ -1` (L150-L153), on numerator and denominator alike | none: both cancel in the ratio, as does colour (summed in each combination on both sides) |
| `|M_card(λ)|² = 0` | `raise Exception("Invalid matrix element for original computation (weight=0)")` (`calculate_weight` L1155-L1169) | `HelicityRefusal::ZeroAtCard`; the file is refused |
| hypothesis non-zero where the card is zero | not reachable per event, so the reweighted σ silently misses that combination's share | refused at plan time (`ReweightError::HelicityRevived`), see below |
| identical particles from different mothers | `get_all_momenta` (`lhe_parser.py` L2710) returns several orders; MadGraph then sets `helicity_reweighting = False` for the rest of the file and averages the orders (L1586-L1592) | refused per event by the audit (cannot be reproduced per event: the switch is stateful) |
| polarized legs | `POLARIZATIONS`/`IS_BORN_HEL_SELECTED` filter (L137-L143), plus MadGraph's own polarization frame | refused (`HelicityPolarized`): the frame semantics are not established here |
| decay chains | the same per-helicity path, unless the ambiguity above | allowed; the ambiguous records are refused |

## Refusals (all named; no panic path added)

Plan time (`ReweightPlan::new`, under `Event`): `HelicityPolarized`,
`HelicitySigned` (generate's own `SignedMatrixElement` refusal already stops a
signed process first; this keeps the library path closed too),
`HelicityUnchecked` (2 → 1 and 1 → n, where `contributing_helicities` has no
probe set), `HelicityRevived` (a hypothesis whose contributing combinations,
on helicity pruning's own probe points and criterion, include one the card's
do not). Per event: `Unrecorded`, `Unknown`, `ZeroAtCard`, and the audit's
different-mothers count; each refuses the file and removes it.

`HelicityRevived` goes beyond MadGraph, which writes the biased weights. It is
pinned on SMEFTsim `u d~ > t b~ NP<=1` at a card with every Wilson coefficient
and the b mass at zero: `cHtbRe` (a right-handed `W t b` current) reaches the
b~'s other helicity; `ctWRe` does not and is accepted; the summed ratio
accepts both.

## Oracles

### 1. Event by event against MadGraph

`gen_reweight_oracle.py` gains a `helicity` flag per row and three rows,
`ee_tth_ymt_hel`, `pp_llj_hel`, `ee_ttx_smeft_hel` (the existing rows' process,
cards and hypotheses, with no `change helicity` line in either generator's
card, so both take their default); summed rows now write
`change helicity False` into vibegraph's card too. Banked events of `_hel`
rows carry `spin` (SPINUP per particle). The old rows are byte-identical in
the JSON (checked: every old row compares equal after loading).

Regeneration (pinned MadGraph 3.7.1 from a real copy of the submodule,
`VG_MG5_ROOT`, the madgraph env's python):

```
$ VG_MG5_ROOT=…-mg/mg5amcnlo VIBEGRAPH_BIN=…-mg/vibegraph-branch flock heavy.lock \
    python validation/madgraph/gen_reweight_oracle.py --work …-mg/oracle ee_tth_ymt_hel
== ee_tth_ymt_hel: generate e+ e- > t t~ h
    INFO reweight: 4 hypotheses, each ratio in each event's own helicity
    INFO reweight: 1 subprocess(es), 2 evaluations per event: 4 as a polynomial in ymt (2 amplitude monomials)
    INFO reweight: card-point |M|^2 checked against the integrand on 1000 events (part 0 direct 1000); largest relative deviation 4.4e-16; beyond 1e-9: 0; flavour mismatches: 0; non-finite weights: 0
    INFO reweight: 1000 events weighted in their own helicity; card-point |M|^2 in it checked against the integrand's, largest relative deviation 0.0e0, beyond 1e-9: 0; refused: 0; identical outgoing particles under different mothers: 0
       ymt0: 1000 events, largest |w_vg/w_mg - 1| 9.28e-08, 0 beyond 2e-7
     ymt100: 1000 events, largest |w_vg/w_mg - 1| 9.79e-08, 0 beyond 2e-7
     ymt250: 1000 events, largest |w_vg/w_mg - 1| 3.67e-08, 0 beyond 2e-7
     ymt350: 1000 events, largest |w_vg/w_mg - 1| 9.09e-08, 0 beyond 2e-7
real 3m8.029s
$ … pp_llj_hel
== pp_llj_hel: generate p p > l+ l- j QCD=2 QED=2
    INFO reweight: 3 hypotheses, each ratio in each event's own helicity
    INFO reweight: 24 subprocess(es), 4 evaluations per event: 3 evaluated directly
    INFO reweight: card-point |M|^2 checked against the integrand on 2000 events (part 0 direct 989, part 0 exchanged 1011); largest relative deviation 0.0e0; beyond 1e-9: 0; flavour mismatches: 0; non-finite weights: 0
    INFO reweight: 2000 events weighted in their own helicity; card-point |M|^2 in it checked against the integrand's, largest relative deviation 0.0e0, beyond 1e-9: 0; refused: 0; identical outgoing particles under different mothers: 0
         wz: 2000 events, largest |w_vg/w_mg - 1| 5.14e-08, 0 beyond 2e-7
        aew: 2000 events, largest |w_vg/w_mg - 1| 9.38e-08, 0 beyond 2e-7
         mz: 2000 events, largest |w_vg/w_mg - 1| 9.55e-08, 0 beyond 2e-7
real 0m16.674s
$ … ee_ttx_smeft_hel
== ee_ttx_smeft_hel: generate e+ e- > t t~ NP<=1
    INFO reweight: 11 hypotheses, each ratio in each event's own helicity
    INFO reweight: 1 subprocess(es), 4 evaluations per event: 11 as a polynomial in ctWRe,cHt,cHWB (4 amplitude monomials)
    INFO reweight: … 1000 events weighted in their own helicity; … beyond 1e-9: 0; refused: 0; …
       p000 … pm111: 1000 events each, largest |w_vg/w_mg - 1| 4.16e-08 … 5.85e-08, 0 beyond 2e-7
real 0m58.468s
```

These full-file lines are the `generate` path end to end (selection, the
hand-off, both proton beam orderings, the audit) against MadGraph reading
vibegraph's own SPINUP. The banked comparison, now enforced on `_hel` rows:

```
$ cargo test -p vibegraph-lib --test reweight_mg_oracle -- --nocapture
ee_tth_ymt_hel [exact|joint]  ymt0..ymt350: 200 events, largest |w/w_MG - 1| 1.42e-8 … 4.60e-8, 0 beyond 2e-7, largest |w/w_summed - 1| 3.35e-1 … 1.65e1
ee_ttx_smeft_hel [exact|joint] p000..pm111: 200 events, largest |w/w_MG - 1| 1.50e-8 … 2.68e-8, 0 beyond 2e-7, largest |w/w_summed - 1| 9.24e-3 … 1.03e0
pp_llj_hel [exact|default]   wz, aew, mz:  200 events, largest |w/w_MG - 1| 1.26e-8 … 1.39e-8, 0 beyond 2e-7, largest |w/w_summed - 1| 1.63e-3, 1.56e-1, 6.77e-2
test result: ok. 1 passed; 0 failed
```

(per-hypothesis lines in `/home/user/wt/logs/lh1-oracle-test2.log`). Tolerance:
`PRINTED = 2e-7`, the summed rows' scale (eight printed digits); worst
measured 4.60e-8 banked (4.3× headroom), 9.79e-8 over the full files (2×).
The banked `pp_llj_hel` events cover both beam orderings of 21 flavour pairs
(e.g. `u~ u` 15, `u u~` 6, `g d` 22, `d g` 23). The "different from summed"
column is the row's power: every row moves some weight by ≥ 1.6e-3, ≥ 8000×
the tolerance, and the test requires > 1e3 × PRINTED.

**Blind spots.** Both generators read the same SPINUP labels, so a convention
shared by vibegraph's writer and both readers (a label flipped on write and
read alike) is invisible; the MadGraph check is that its `NHEL` table indexes
the same physics by the same labels. A combination no banked event carries is
unchecked; a leg whose label is 0 cannot see a sign error (the tth row is blind
to mutation 3 below, the SMEFT row catches it). It is blind, as the summed rows
are, to the strong coupling and to `generate`'s subprocess indexing; the
full-file lines and the audit cover those.

### 2. Unbiasedness (5 seeds, then 20)

`e+ e- > t t~ h`, one grid, each seed generating the same 20000 events twice
(summed card, then no `change helicity` line; the driver checks the event
blocks are identical), σ = mean weight, the per-seed difference with its paired
error. 5 seeds:

```
$ python3 …-mg/unbiased.py …-mg/vibegraph-branch … 20000 101,202,303,404,505
     ymt0: sigma_summed 7.690685e-05  sigma_event 7.697429e-05  mean diff +6.744e-08 (rel +8.77e-04), seed spread of diff 3.02e-06, error of mean diff 1.35e-06, pull +0.05, chi2/dof of per-seed diffs against 0: 2.58/5
   ymt100: sigma_summed 7.362001e-04  sigma_event 7.360393e-04  mean diff -1.608e-07 (rel -2.18e-04), seed spread of diff 6.72e-07, error of mean diff 3.01e-07, pull -0.54, chi2/dof of per-seed diffs against 0: 1.37/5
   ymt250: sigma_summed 4.091981e-03  sigma_event 4.092450e-03  mean diff +4.691e-07 (rel +1.15e-04), seed spread of diff 7.34e-07, error of mean diff 3.28e-07, pull +1.43, chi2/dof of per-seed diffs against 0: 0.58/5
   ymt350: sigma_summed 7.907061e-03  sigma_event 7.908598e-03  mean diff +1.537e-06 (rel +1.94e-04), seed spread of diff 3.17e-06, error of mean diff 1.42e-06, pull +1.09, chi2/dof of per-seed diffs against 0: 0.69/5
```

Every mean difference is inside its error, but two of the four χ² read
far below 5 (0.58, 0.69), the warning sign of an overstated per-seed error or
correlated seeds. The seeds' events are distinct (no outgoing-momentum block of
seed 101 appears in seed 202), so the sweep was extended to 20 seeds on the
same grid, re-running the first five bit for bit:

```
$ python3 …-mg/unbiased.py …-mg/vibegraph-branch … 20000 101,202,…,2020
     ymt0: sigma_summed 7.772290e-05  sigma_event 7.760805e-05  mean diff -1.148e-07 (rel -1.48e-03), seed spread of diff 2.99e-06, error of mean diff 6.68e-07, pull -0.17, chi2/dof of per-seed diffs against 0: 26.76/20
   ymt100: sigma_summed 7.367021e-04  sigma_event 7.365779e-04  mean diff -1.242e-07 (rel -1.69e-04), seed spread of diff 9.14e-07, error of mean diff 2.04e-07, pull -0.61, chi2/dof of per-seed diffs against 0: 13.82/20
   ymt250: sigma_summed 4.091202e-03  sigma_event 4.091453e-03  mean diff +2.509e-07 (rel +6.13e-05), seed spread of diff 1.85e-06, error of mean diff 4.15e-07, pull +0.61, chi2/dof of per-seed diffs against 0: 12.16/20
   ymt350: sigma_summed 7.904888e-03  sigma_event 7.905649e-03  mean diff +7.605e-07 (rel +9.62e-05), seed spread of diff 6.35e-06, error of mean diff 1.42e-06, pull +0.54, chi2/dof of per-seed diffs against 0: 13.01/20
```

χ²/dof 0.61–1.34 with pulls below 0.7: the two estimators agree on every
hypothesis to 1e-4 relative (1.5e-3 at `ymt = 0`, where the per-event one is
noisiest), and the 5-seed χ² was a fluctuation. What this cannot see: a bias
common to both ratios (the denominators are shared), or one confined to a
helicity combination these hypotheses do not revive (`HelicityRevived` is the
guard there).

The per-event estimator's own error is 1.3–8.4× the summed one's per seed
(ymt0 worst: the Yukawa-only combinations carry weights `|M_hyp(λ)|²/|M_card(λ)|²`
far from one), which is the spread the item predicted.

### 3. Mutations

Each against `reweight_mg_oracle`, applied and restored by a driver
(`…-mg/mutations.py`; restore checked by sha256):

```
== mutation: summed ratio in event mode   (`if false && self.plan.helicity == HelicityRatio::Event`)
   panicked at vibegraph-lib/tests/reweight_mg_oracle.rs:471:21:
   assertion `left == right` failed: ee_tth_ymt_hel [exact] ymt0: per-event-helicity weights beyond 2e-7 of MadGraph's (largest 4.23e3)
   test result: FAILED. 0 passed; 1 failed
== mutation: wrong helicity: the beams' helicities in reversed leg order   (`swapped.swap(0, 1)` in `combination`)
   assertion `left == right` failed: ee_tth_ymt_hel [exact] ymt0: per-event-helicity weights beyond 2e-7 of MadGraph's (largest 1.20e5)
   test result: FAILED. 0 passed; 1 failed
== mutation: dropped sign convention: the last leg's label negated
   (ee_tth_ymt_hel passes: its last leg is the Higgs, label 0)
   assertion `left == right` failed: ee_ttx_smeft_hel [exact] p000: per-event-helicity weights beyond 2e-7 of MadGraph's (largest 2.75e-2)
   test result: FAILED. 0 passed; 1 failed
== restored engine.rs: sha256 98afdb1c9078d014 (original 98afdb1c9078d014): identical
```

And the generate-level hand-off, against the banked proton gate (the physical
order handed over in place of the representative order,
`Some(&selection.cm_helicity)` → `Some(&selection.helicity)`):

```
$ cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_reweight_proton -- --nocapture --test-threads 1
 INFO reweight: 5000 events weighted in their own helicity; card-point |M|^2 in it checked against the integrand's, largest relative deviation 6.4e2, beyond 1e-9: 720; refused: 704; …
ERROR 704 of 5000 events cannot be weighted in their own helicity, so the weights in /tmp/…/events.lhe are wrong and removed it; first: part 0 Exchanged: the event's helicities [-1, 1, 1, -1, -1] are not a combination the subprocess's amplitude carries
 INFO reweight: 3000 events weighted in their own helicity; … largest relative deviation 1.8e2, beyond 1e-9: 1075; refused: 169; …
test result: FAILED. 1 passed; 2 failed
restored: 3368f8c6ce27e57c original: 3368f8c6ce27e57c
```

### 4. Bit for bit against main 4b132cc

main's binary built from `git archive c512cec4` (main plus the brief only), this
branch's from 614ca877's source (unchanged since in `src/`); one grid per row,
`--seed 20261005 --nevents 500`, the reweight rows with
`change helicity False` (summed; main accepts the line as benign):

```
$ bash …-mg/bitforbit.sh …-mg/vibegraph-main …-mg/vibegraph-branch …
== ee_tth_ymt     (polynomial path, 2 hypotheses)
   main:   500 28cdee25409ee90bed117f8b8c5ba3af8d83428d8b1fbcdb57ab6c249c713a4e  file ef5c6c2dccfa89ec
   branch: 500 28cdee25409ee90bed117f8b8c5ba3af8d83428d8b1fbcdb57ab6c249c713a4e  file ef5c6c2dccfa89ec
== pp_llj         (exact path, 3 hypotheses, 145 systematics weights per event)
   main:   500 98f4065e2efe7a5d6dc215ceafd259ca8c56764b9b41201e4f8071e18680ba49  file 820906e6be6008e7
   branch: 500 98f4065e2efe7a5d6dc215ceafd259ca8c56764b9b41201e4f8071e18680ba49  file 820906e6be6008e7
== pp_llj_plain   (no reweight card)
   main:   500 8b3ee6adcc1fa3b1400468793826cd79d45774ea55f576e5dd9ddfd80ad4630b  file b79e9ea0bda3c582
   branch: 500 8b3ee6adcc1fa3b1400468793826cd79d45774ea55f576e5dd9ddfd80ad4630b  file b79e9ea0bda3c582
```

(event-block sha256, then the whole file's: byte-identical files; `diff` empty.)

### Unit tests added

- `engine::tests::per_event_helicity_ratios_reproduce_direct_evaluation`:
  every combination of `e+ e- > t t~ h` (polynomial and exact),
  `g g > t t~ h` (two flows, at an event α_s) and `e+ e- > mu+ mu-`
  (`MZ`, `aEWM1`) against unpruned amplitudes bound afresh, 1e-10; the
  card-weighted mean of the combinations' ratios equals the summed ratio (the
  identity behind unbiasedness), and the combination's card `|M|²` equals the
  direct one. Blind to a convention the unpruned evaluator shares.
- `per_event_helicity_refusals_are_named`: `Unrecorded` (none, and a 9),
  `Unknown` (bad label, short vector), `ZeroAtCard` (a combination J_z forbids
  with every momentum along the beam axis), the summed path reading none.
- `per_event_helicity_refuses_what_it_cannot_serve`: polarized
  (`e+ e- > z{0} h`), signed (`u u~ > u u~ QCD^2==2`), unchecked
  (`e+ e- > z`), revived (above); each accepted when summed.
- `card::tests::change_helicity_selects_the_ratio`, the classification rows,
  the `change helicity` after `launch` refusals.
- `generate::tests::a_refused_helicity_removes_its_file`: refused event,
  different mothers, card-point `|M|²` in the helicity off the integrand's.
- `vibegraph-cli/tests/cli_reweight_helicity.rs` (hermetic): card default,
  `True`, `False`, both flags, an agreeing flag; same events throughout, the two
  ratios' weights differing on most events and each spelling giving its set;
  the audit line on every per-event run; a contradicted flag refused.
- `cli_reweight_proton` (banked) now runs in the per-event mode (its card has
  no line) and asserts the new audit line: 5000 and 3000 events, deviation
  0.0e0, 0 refused, 0 ambiguous.

## Gates

All with `CARGO_TARGET_DIR=/home/user/wt/target-helicity-reweight` and the
lean variables, each cargo command under `flock /home/user/wt/heavy.lock`.

```
$ cargo fmt --all --check
fmt-rc=0
$ cargo clippy --workspace --all-targets -- -D warnings
rc=0                      (no warning, no error; log /home/user/wt/logs/lh1-clippy.log)
$ python3 scripts/kb.py lint      (after deleting the item)
1351 concepts, 465 backlog items; 0 errors, 5 warnings   (the 5 warnings are pre-existing `state: blocked` items)
```

**Hermetic workspace suite** (`…-mg/hermetic.sh`: `research/refs/mg5amcnlo`,
`validation/madgraph/output` and `validation/pdf` moved aside, an empty
submodule directory left as CI has it, all three restored by an `EXIT` trap):

```
$ cargo test --workspace
… 70 test binaries, 2288 passed, 0 failed, 26 ignored
test result: ok. 1455 passed; 0 failed; 15 ignored … (vibegraph-lib unit tests)
reweight_mg_oracle:    test result: ok. 1 passed; 0 failed
cli_reweight_helicity: test result: ok. 1 passed; 0 failed
input_card_refusals:   test result: ok. 24 passed; 0 failed
MARKER rc=0               (no NOT RESTORED line; the hidden directory is empty)
```

**Extended-validation targets** (`pixi run validate-reweight`'s two commands,
run directly):

```
$ cargo test -p vibegraph-lib --test reweight_mg_oracle -- --nocapture
test result: ok. 1 passed; 0 failed      (lines in oracle 1)
$ cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_reweight_proton -- --nocapture --test-threads 1
test hadronic_reweighting_is_taken_in_the_drawn_subprocess ... Audit { events: 5000, terms: [(0, "direct", 2486), (0, "exchanged", 2514)], largest: 0.0, m2_mismatches: 0, flavour_mismatches: 0, non_finite: 0 }
 INFO reweight: 5000 events weighted in their own helicity; card-point |M|^2 in it checked against the integrand's, largest relative deviation 0.0e0, beyond 1e-9: 0; refused: 0; identical outgoing particles under different mothers: 0
test matched_mixed_multiplicity_reweighting_is_taken_in_the_drawn_subprocess ... Audit { events: 3000, terms: [(0, "direct", 901), (0, "exchanged", 953), (1, "direct", 381), (1, "exchanged", 398), (2, "direct", 212), (2, "exchanged", 155)], largest: 6.6e-16, … }
 INFO reweight: 3000 events weighted in their own helicity; … largest relative deviation 0.0e0, beyond 1e-9: 0; refused: 0; identical outgoing particles under different mothers: 0
test result: ok. 3 passed; 0 failed; … finished in 44.14s
```

The per-combination `|M|²` comparison reads 0.0e0: the reweighter's
combination and the integrand's are the same arithmetic on the same
couplings, unlike the summed value, whose sum runs over differently pruned
combination sets. The mutation above shows the check is not vacuous.

## vg-validator requests

None: every oracle here ran locally in minutes.

## Brief corrections

- The brief says the reweight card parser "refuses it, or accepts a card that
  omits the line": both held, by construct (`change helicity True` refused, no
  line accepted with the divergence recorded); the decision above replaces both.
- The brief's cargo target (`…/sprintl-helicity-reweight/target`) was
  superseded by the assignment's `/home/user/wt/target-helicity-reweight`;
  that one was used.
- "MadGraph 3.5.7" is the packaged env's MadGraph; every reference here is the
  pinned 3.7.1 (`mg5_pinned.sh`, `VG_MG5_ROOT`), as the oracle requires.
- Deleting the item breaks the link to it in
  `research/kb/events/reweight-card-classification.md` (kb-lint is a CI gate),
  and that page's table described `change helicity True` as refused; the page
  is updated to the code (two rows changed, two added, the ruling rewritten).
  This is a kb edit beyond the report, made because the deletion the brief asks
  for requires it.

## Found

- **Summed reweighting differs from MadGraph on records with identical
  outgoing particles under different mothers.** MadGraph evaluates such an
  event in every leg order the mothers allow and averages
  (`identical_particle_in_prod_and_decay = average`, `reweight_interface.py`
  L1586-L1592, L1646-L1684); `generate` evaluates the record's own
  order. Only the per-event mode refuses these records now. Evidence: the code
  paths; no banked row has such a record. Close: a MadGraph row on a decay
  chain with a repeated final-state particle (e.g. `p p > t t~, t > b w+, w+ > j j`
  with a jet equal to a b-side parton), then match the average or refuse.
- **The summed MadGraph rows are still informational.** Every one has read 0
  beyond 2e-7 since they were banked (worst 9.98e-8 full file, 4.6e-8
  banked); the per-helicity rows are enforced at the same tolerance. Close:
  enforce `PRINTED` on every row in `reweight_mg_oracle`.
- **Each per-event-helicity event evaluates every helicity combination to use
  one.** `eval_hel_m2` and `eval_hel_jamps` fill all of them; on the exact path
  that is one full evaluation per distinct hypothesis point. Close: a
  single-combination evaluation of a pruned evaluator (the arena already
  separates combinations), timed with `bench_reweight.py` on the `_hel` rows.
- **Per-event-helicity reweighting of decays and 2 → 1 is refused**
  (`HelicityUnchecked`): `contributing_helicities` has no probe set when
  `n_in ≠ 2` or `n_ext ≤ 3`. Close: a probe set for 1 → n at rest (and the
  degenerate 2 → 1 kinematics), then the revival check runs there too.
- **Polarized processes are refused in per-event mode.** MadGraph's per-helicity
  path on a polarized leg goes through `IS_BORN_HEL_SELECTED` and its
  polarization frame; what SPINUP holds there and which frame the label means
  is not established here. Close: a MadGraph row on `e+ e- > z{0} h` with
  `change helicity True`.
- **A `change helicity` line after the first `launch` is refused, where
  MadGraph gives the launches before and after it different ratios.** Close,
  if wanted: a ratio per launch (`Launch` would carry it).
- **simshop shows the helicity-aware choice as disabled** (the item says so);
  it can now be enabled, and the default it should show is MadGraph's, the
  per-event ratio.
- **`--help` order of the shared parallel flags moved** when the generate
  arguments gained a flag (`-j`/`--tui`, `--parallel-diagrams`/`--output-ldjson`
  swapped in `docs/src/cli/reference.md`); regenerated, harmless, noted so a
  reviewer does not chase it.
