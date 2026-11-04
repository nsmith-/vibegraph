---
type: Session Report
title: "H-C1 report: xpt* at or below zero, and the per-PDG pT minimum"
description: "xptj/xptb/xpta/xptl ≤ 0 read as off; pt_min_pdg applied as setcuts.f applies it, with MadGraph's refusals; two MadGraph defects found (the light-PDG refusal never fires; the setcuts.f:681 smin loop index removes phase space on some cards), local MadGraph σ against vibegraph, mutation evidence, the M3 banking request and Found."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-04}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Branch `sprinth-cuts` in `/home/user/wt-cuts`, from `sprinth-plan` at `9ba3d96`.
MadGraph is the pinned tree in `research/refs/mg5amcnlo` (`b7687064`), run
locally with `python3 research/refs/mg5amcnlo/bin/mg5_aMC`, `pdlabel =
nn23lo1`. Every MadGraph run directory, driver and log named below is under
the session scratchpad `…/scratchpad/H-C1/` (`mg/`, `vg/`); none is committed.

## What changed

- `vibegraph-lib/src/cuts.rs`
  - `NONPOSITIVE_IS_OFF`: `xptj`, `xptb`, `xpta`, `xptl` at a value ≤ 0 pass
    `detect_unimplemented`; a positive value is still `UnimplementedCutActive`.
  - `pt_min_pdg` leaves `UNIMPLEMENTED_CUTS`. `PdgPtMin::from_card` parses it
    as `banner.py:1220-1251` parses a dict entry (`{k: v, …}`, quoted keys,
    trailing comma, later keys win, the one-entry `k : v` / `k, v` / `k v`
    forms, `__type__`/`default` dropped as `banner.py:4712` drops them).
    `single_leg_cuts` gives every outgoing leg of a listed `|PDG|` a
    `SingleLegCut` with `pT ≥ threshold` and every other threshold inactive,
    replacing any class cut, exactly as `setcuts.f:318-334` overwrites all six
    of `etmin/etmax/emin/emax/etamin/etamax`. The block ignores `do_cuts`
    except through `cut_decays` (`setcuts.f:211`, `:322`): a Z or a neutrino
    is cut; a decay product under `cut_decays = F` is not (new
    `LegInfo::uncut_decay`).
  - New `CutError::PdgCutRefused`. Refused: a negative code
    (`banner.py:4717`), more than 25 codes (`:4714`), code 0 (names no particle,
    and listed first it would read as the empty list at `setcuts.f:318`), a
    malformed entry, the light codes 1–5, 11, 13, 15, 21, 22 (see defect D1),
    and the cards defect D2 reaches (`PdgPtMin::refuse_smin_overcount`).
  - The ŝ floor (`shat_min_hint`) takes the per-PDG threshold through the
    existing `Σ pT_min` bound; `spacelike_floor` sees it through `single`.
- `vibegraph-lib/src/runcard/classes.rs`: `xpt*` and `pt_min_pdg` are
  `Consumed` with reasons naming their new readers (`R_XPT`, `R_PT_MIN_PDG`).
- `research/kb/backlog/feature/{xpt-nonpositive-refused,pt-min-pdg-refused}.md`
  deleted.

## xpt*: every MadGraph reader

```
$ grep -rn -i -w "xptj\|xptb\|xpta\|xptl" Template/LO --include=*.f --include=*.inc
cuts.f:981  IF(xptj.gt.0d0)   cuts.f:994 IF(xpta.gt.0d0)   cuts.f:1007 IF(xptb.gt.0d0)   cuts.f:1020 IF(xptl.gt.0d0)
setcuts.f:540  max(ej,ptj,xptj,0d0,…)   :584 max(eb,ptb,xptb,0d0)   :591 max(xptb,0d0)
setcuts.f:613  max(ea,pta,xpta,0d0)     :622 max(xpta,0d0)          :645 max(el,ptl,xptl,0d0,…)   :659 max(xptl,0d0,…)
setcuts.f:752/760/763/772  if (xpt?.gt.0d0) fail_reason = …     setcuts.f:835/853/865/877  if(ncheck.eq.0.and. xpt? .gt. 0d0) xpt? = 0
initcluster.f:15-17, reweight.f:593-595, :1393-1395, setscales.f:29-31, Source/cuts.inc:78-80: declarations of /to_specxpt/ only
$ grep -rn "xpt[jbal]" madgraph/         →  banner.py:4378-4381 add_param only
```

`max(…, x, 0d0)` with `x ≤ 0` is `max(…, 0d0)`, and every other read is
behind `> 0`, so a value ≤ 0 is inert everywhere in the LO template and the
MLM code (`reweight.f`, `initcluster.f`, `setscales.f` read none of the four).

## Oracles and mutation evidence

```
$ cargo test -p vibegraph-lib --lib cuts::
test result: ok. 75 passed; 0 failed; 0 ignored; 0 measured; 1141 filtered out
```

New tests: `a_nonpositive_xpt_compiles_to_the_default_cuts` (−1, 0, −0.5, −100
for each name pass detection and compile to the default's `Cuts` on jet, b,
photon and lepton final states), `a_positive_xpt_is_refused` (1e-9, 10, 1e5),
`pt_min_pdg_cuts_a_listed_heavy_leg` (hand-built `g u > z u`: 99.9 fails, 100
and 100.1 pass, an unlisted Z passes at 20, no rapidity cut on the Z at y = 6,
the jet keeps `etaj`), `pt_min_pdg_reads_the_absolute_code` (W+ and W−;
unlisted processes compile to the default filter), `pt_min_pdg_reaches_a_neutrino`,
`pt_min_pdg_follows_cut_decays`, `pt_min_pdg_enters_the_shat_floor_as_a_bound`
(floor `(100+20)² = 14400`, below ŝ of an accepted threshold point; MadEvent's
own `τ` floor on the same card is also 14400, see below),
`pt_min_pdg_spellings_read_as_banner_py_reads_them` (including MadGraph's
written `{'23': 100.0}`), `pt_min_pdg_refusals`,
`pt_min_pdg_refuses_the_cards_madevent_overcounts_smin_on`.

Mutations, each applied alone to the finished `cuts.rs` and the cuts tests
rerun (script `mutations.sh`, logs `H-C1_mutations.log`, `H-C1_mutations2.log`;
the file was restored and `cmp`-checked after each set):

| id | mutation | caught by |
|---|---|---|
| M1 | the `NONPOSITIVE_IS_OFF` skip disabled (`if false && …`): the pre-change behaviour | `a_nonpositive_xpt_compiles_to_the_default_cuts` FAILED (73 passed, 1 failed) |
| M2 | `*x <= 0.0` → `*x <= 1e-6` | `a_positive_xpt_is_refused` FAILED |
| M3 | the `uncut_decay` exemption dropped | `pt_min_pdg_follows_cut_decays` FAILED |
| M4 | the light-code refusal disabled | `pt_min_pdg_refusals` FAILED |
| M5 | `pdg.unsigned_abs()` → `pdg as u32` (signed lookup) | `pt_min_pdg_reads_the_absolute_code` FAILED |
| M6 | the per-PDG `SingleLegCut` not pushed | 5 tests FAILED (heavy leg, shat floor, cut_decays, absolute code, neutrino) |
| M7 | per-PDG `eta_max` −1 → 2.5 | 3 tests FAILED (heavy leg, neutrino, shat floor) |
| M8 | the 25-code cap off by one | `pt_min_pdg_refusals` FAILED |

M1 is the "fails before the change" evidence for xpt*: it reinstates the old
rule (refuse any value off the default) and the test fails. `<= 0.0` →
`< 0.0` is an equivalent mutant (0 is the default and is accepted before the
rule is read) and is not listed. The overcount refusal was added after the
mutation runs; its test is new and was not mutated.

## Per-event check (written events)

`lhe_pt_check.py FILE 23:THRESHOLD` reads every status-1 particle of |PDG| 23:

```
vibegraph generate H-C1_vg_zj_s1/grid.bin.zst --seed 7 --nevents 20000
  23:100    events=20000 listed_particles=20000 violations=0   min_pt={23: 100.00059544380336}
  23:100.5  events=20000 listed_particles=20000 violations=259 (non-vacuity)
MadGraph H-C1_zj_s101 unweighted_events.lhe.gz
  23:100    events=10000 listed_particles=10000 violations=0   min_pt={23: 100.00492382995556}
  23:100.5  events=10000 listed_particles=10000 violations=125
```

## σ: p p > z j at pt_min_pdg = {23: 100}, against MadGraph run here

Cards: `import model sm; generate p p > z j`; MadGraph launched with
`nevents 10000`, `iseed`, `pdlabel nn23lo1`, `use_syst False`,
`fixed_ren_scale/fixed_fac_scale True`, `scale = dsqrt_q2fact1 = dsqrt_q2fact2 =
91.188`, `pt_min_pdg {23: 100}`, everything else the generated default
(`ptj 20`, `etaj 5`, `maxjetflavor 4`). vibegraph read MadGraph's own
`Cards/run_card.dat` from `H-C1_zj_s101` with only `pdlabel = lhapdf`,
`lhaid = 247000` changed (`vg/zj_mg_run_card.dat`), so the per-PDG line it
parsed is MadGraph's spelling `{'23': 100.0}`. MadGraph values are
`SubProcesses/results.dat`; vibegraph's are `integrate --seed N` at the
default 0.1% target. Statistics are unweighted means over seeds
(`seedstats.py`: sample sd, sem, χ²/dof of the seeds about their mean on their
quoted errors).

| run | seeds | mean (pb) | sd | sem | χ²/dof |
|---|---|---|---|---|---|
| vibegraph | 20 (1–20) | 604.136 | 0.501 | 0.112 | 0.85 |
| MadGraph default survey | 10 (101–110) | 603.005 | 0.807 | 0.255 | 0.52 |
| MadGraph `hard_survey = 2` | 5 (401–405) | 603.55 | 0.792 | 0.354 | 0.50 |
| control, `pt_min_pdg = {}`: vibegraph | 10 | 11905.4 | 11.1 | 3.51 | 1.06 |
| control, `pt_min_pdg = {}`: MadGraph default | 5 (301–305) | 11891.6 | 12.4 | 5.56 | 0.43 |

vibegraph − MadGraph: +1.13 ± 0.28 pb (+0.19%, 4.1σ on the sems) against the
default survey, +0.59 ± 0.37 pb (+0.10%, 1.6σ) against `hard_survey = 2`, and
+13.8 ± 6.6 pb (+0.12%, 2.1σ) on the uncut control. The offset is the same
size with and without the cut, and moves by half when MadGraph's survey is
hardened, which is the [default survey bias](../../../validation/madevent-default-survey-bias.md)
pattern rather than the cut; every difference is inside the 0.5% `rel_tol` of
the enforced hadronic σ rows. The ratio σ(cut)/σ(uncut) agrees to +0.07%.
This is not a banked row (see the M3 request). MadGraph's seeds scatter at
about half their quoted errors (χ²/dof ≈ 0.5); the comparison reads the
seed sems, not the quoted errors.

## MadGraph defects found

**D1: the light-PDG refusal never fires** (`banner.py:4722`). The keys of
`pt_min_pdg` are strings (`banner.py:1238`, `new_value[x] = y`), and the
refusal tests `pdg in pdg_to_cut for pdg in [1,2,3,4,5,21,22,11,13,15]`
with integers. Probe (`probe_banner.py`, `RunCardLO()`, `rc['pt_min_pdg'] =
…`, `update_system_parameter_for_include()`):

```
{23: 100} OK [23] [100.0]
{21: 50} OK [21] [50.0]
{22: 50} OK [22] [50.0]
{1023: 100} OK [1023] [100.0]
{-23: 100} RAISED MadGraph5Error Some PDG specific cuts are defined using negative pdg code
```

and a launched run on `{22: 100, 24: 0, 25: 0}` wrote `PDG_CUT(…) = 22` into
`Source/run_card.inc` and ran. On a light code MadEvent then replaces the
class's pT, energy and rapidity cuts of those legs (`setcuts.f:326-331`):
`{21: 50}` drops `etaj` from every gluon. That changes weights; outcome here:
**refused** (policy outcome 2, what MadGraph intends), with the defect named
in the error and at `PDG_CUT_LIGHT`.

**D2: the per-PDG `smin` term reads the list index as the leg index**
(`setcuts.f:678-690`):

```
      do i = 1,pdg_cut(0)
         do j=nincoming, nexternal
            if (abs(idup(i,1,iproc)).eq.pdg_cut(i))then
               k = j
               if(do_cuts(i)) nb_pdg = nb_pdg + 1
```

The test does not depend on `j`: it fails throughout or passes on all
`nexternal − nincoming + 1` iterations, and `smin += nb_pdg·(pmass(nexternal)²
+ ptmin²)`. `smin` is a generation bound on hadronic beams after all — not
through `GENCMS` (`genps.f:1659` has `TAUMIN = 0d0 !SMIN/S`), but through
`myamp.f:569-570`, `if (xo.lt.smin/stot) xo = smin/stot`, the lower edge of
the `1/s` τ transform. Which code is `i`-th is the iteration order of a
Python set of strings, so it moves with the Python hash seed:

```
$ for h in 1..7; PYTHONHASHSEED=$h python3 -I probe_set.py <a3g Cards/run_card.dat> '{22: 100, 24: 0, 25: 0}'
1 [24, 22, 25]  2 [25, 24, 22]  3 [22, 25, 24]  4 [24, 25, 22]  5 [22, 24, 25]  6 [24, 25, 22]  7 [24, 25, 22]
```

Measured on `generate u u~ > a g g g` with `pt_min_pdg = {22: 100, 24: 0,
25: 0}` (fixed scales 91.188, nn23lo1, defaults otherwise), each seed run with
the pinned `setcuts.f` ("defect") and with `patch setcuts_index.patch`
(`do j=nincoming+1`, `idup(j,…)`, `do_cuts(j)`; "fixed"), same seeds 201–205:

| hash order | variant | `Define smin to` | τ floor `xo·stot` (`Transforming s_hat 1/s`) | σ mean ± sem (pb) | event files |
|---|---|---|---|---|---|
| (22, 24, 25), seed 5 | defect | 3700 | 25600 | 6.2930 ± 0.0098 | identical to fixed, seeds 201 and 205 md5-compared |
| (22, 24, 25), seed 5 | fixed | 13700 | 25600 | 6.2930 ± 0.0098 | |
| (24, 25, 22), seed 4 | defect | **53700** | **53700** | **6.2507 ± 0.0061** | none below ŝ 53700 (min 53714.6, 40000 events) |
| (24, 25, 22), seed 4 | fixed | 13700 | 25600 | 6.2930 ± 0.0098 | |

The defect at the photon-third order is −0.67% ± 0.18% (3.7σ) on σ, lower on
every one of the five same-seed pairs. Its size checks against the unaffected
sample: 247 of 50000 events (0.49% ± 0.03%) of the fixed runs have ŝ < 53700
(smallest 43943 GeV², above the kinematic bound 200² = 40000 from the photon's
pT ≥ 100 balanced by the gluons). With one listed code, as on the CMS
DarkPhoton-monojet card `{1023: 100}` and the `{23: 100}` proxy, `i = 1`
compares the first incoming parton and the term is zero (measured: proxy
`smin` 8315.25 = m_Z², τ floor 14400 from `etot`), so the defect only lowers
the hint there and changes no weight.

Outcome here: **refused**. Reproducing is impossible — MadEvent's own result
depends on its Python hash seed — and the cards reaching it are narrow: at
least three listed codes and a listed outgoing leg with `do_cuts` (at most
20 GeV, not a neutrino, not an uncut decay product). In the SM every such leg
is a light code, already refused; a BSM light particle (a dark photon below
20 GeV) is not. `refuse_smin_overcount` refuses that combination order-blind.

Both defects need a catalogue entry in
[MadGraph defects](../../../validation/madgraph-defects.md) and an upstream draft
(Found F1, F2).

## Gates

```
$ cargo fmt --all --check                                         → exit 0
$ cargo clippy --workspace --all-targets -- -D warnings           → Finished … clippy exit 0
$ cargo test --workspace                                          → 50 suites: 1814 passed, 0 failed, 25 ignored; test exit 0
$ python3 scripts/kb.py lint                                      → 1054 concepts, 340 backlog items; 0 errors, 4 warnings (pre-existing `state: blocked` items)
$ bash validation/validate.sh                                     → validate exit 0
    (clippy with the extended-validation features clean; 80 suites: 2263 passed, 0 failed, 109 ignored;
     "77 rows × 4 categories = 308 cells: 218 measured in the layers this run drove (213 ✅, 5 ⚠️, 14 ⏳,
     76 — / uncovered)", "the measured cells are the declared cells")
```

`cargo test --workspace` and the two clippy runs were at the tree of commit
`cbfc8df` (the implementation); this report is the only file the second commit
adds. `validate.sh` ran with an untracked `.cargo/config.toml` setting
`[profile.release-debug] debug = 0`, deleted afterwards. No banked reference
row reads `pt_min_pdg` or a non-default `xpt*`, so the banked layer checks only
that nothing else moved.

## M3 banking request: `pp_to_zj_ptminpdg`

Only needed for a committed manifest σ row; the comparison above is the
evidence for this PR.

1. Process script `validation/madgraph/scripts/pp_to_zj_ptminpdg.mg5`:
   ```
   import model sm
   generate p p > z j
   output pp_to_zj_ptminpdg
   ```
2. Run card `validation/madgraph/pp_to_zj_ptminpdg_run_card.dat`: the
   generated default of that output with `nevents 10000`, `pdlabel lhapdf`,
   `lhaid 247000`, `use_syst False`, `fixed_ren_scale True`,
   `fixed_fac_scale True`, `scale 91.188`, `dsqrt_q2fact1 91.188`,
   `dsqrt_q2fact2 91.188`, `pt_min_pdg {23: 100}` (the card actually run here
   is `mg/H-C1_zj_s101/Cards/run_card.dat`, `nn23lo1`, and
   `vg/zj_mg_run_card.dat` is it on lhaid 247000).
3. Bank as a `card` row through `bank_seeded_sigma.sh`
   (`ROWS=pp_to_zj_ptminpdg pixi run -e madgraph bank-seeded-sigma`): five
   fresh-directory seeds at `hard_survey = 2` and the two 4× seeds the
   driver adds, per AGENTS.md's seed policy; if `SCREEN` flags, 20 + 20 for
   the ladder.
4. Bank `output/seeded_sigma/pp_to_zj_ptminpdg/` (each seed's
   `SubProcesses/results.dat`, `P*/symfact.dat` and `G*/results.dat`, banner,
   `<init>`, `run_config.inc`, `vg_seed_result.txt`, `seed_record.json`) and
   one seed's `Events/run_*/unweighted_events.lhe.gz` for the per-event pT
   gate.
5. Record `Source/run_card.inc`'s `PDG_CUT` and `PTMIN4PDG` lines and one
   `G*/log.txt` `Define smin to` / `Transforming s_hat` line per P directory.

vibegraph side, ready: 20 seeds, 604.136 ± 0.112 pb (table above), from
`vibegraph integrate zj_proc_card.dat --run-card zj_mg_run_card.dat --seed N`.

## Corrections to the brief

- "banner.py … refuses … the light PDGs 1–5, 11, 13, 15, 21, 22": it means
  to and does not (D1). This crate refuses them.
- "setcuts.f:310–330": the `etmin` block is `setcuts.f:318-334`; the `smin`
  hint is `678-690`, not `675-690`; the per-pair `s_min` for `mxx_min_pdg` is
  `438-463`.
- "smin is a generation bound": on hadronic beams only through
  `myamp.f:569-570`; `GENCMS` does not read it (`genps.f:1659`). It is bound
  there, and D2 reaches a weight through it.

## Found

- **F1 — catalogue D1.** `banner.py:4722` compares string keys with integers,
  so MadEvent accepts and applies a per-PDG cut on light quarks, b, charged
  leptons, gluons and photons, replacing their class cuts. Evidence: the probe
  above and the launched `{22: …}` runs. Closes with a row in
  `research/kb/validation/madgraph-defects.md` and an upstream draft
  (fix: `int(pdg) in …`).
- **F2 — catalogue D2.** `setcuts.f:681-683` uses `i` for `j`; on cards with ≥3
  per-PDG codes and a light listed outgoing leg it raises MadEvent's τ floor
  past the cuts, by hash-seed chance (−0.67% ± 0.18% on `u u~ > a g g g`).
  Closes with a catalogue row, an upstream draft with the patch
  (`setcuts_index.patch`, three lines) and the reproducer above.
- **F3 — `mxx_only_part_antipart` is not benign.** `classes.rs` files it
  `IgnoredBenign(B_MXX)` ("qualifies the mxx_min_pdg cut alone"), but
  `banner.py:4711` puts its keys into `pdg_cut`, and `setcuts.f:318-334` then
  overwrites the class cuts of those legs with the per-PDG defaults:
  `{'default': False, 22: True}` would remove `pta`/`etaa` from photons in
  MadEvent while this crate keeps them. Closes by refusing a non-default key
  set (or folding its keys into `PdgPtMin`), with a test.
- **F4 — `pt_min_pdg` order of `pdg_cut` is per-invocation random in MadGraph**
  (string-key set order). Harmless for the per-leg cut, which loops over every
  code, but any future per-PDG reproduction that depends on list order (the
  `mxx_min_pdg` pairs at `setcuts.f:438-463` are order-independent too) needs
  the same caution. Informational; closes when catalogued with F2.
- **F5 — the proxy offset.** vibegraph sits +0.12% … +0.19% above MadGraph's
  default survey on `p p > z j` with and without the cut, +0.10% against
  `hard_survey = 2`. Inside tolerance, but if the M3 row is banked at
  `hard_survey = 2` and the 1× vs 4× screen moves it, a budget ladder on this
  row says whether the residual is MadEvent's survey.
