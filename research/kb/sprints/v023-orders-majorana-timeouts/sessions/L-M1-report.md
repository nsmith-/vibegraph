---
type: Session Report
title: "L-M1 report: Majorana fermions and fermion-number violation against MadGraph per diagram"
description: "Fermion lines without a fermion-number arrow are oriented by one end's charge and read with each vertex's own orientation; per-diagram, per-helicity AMP() oracles on the toy Majorana and diquark models and on MSSM neutralinos, the refusals that stay, and what the oracles cannot see."
status: draft
tags: [majorana, fermion-flow, fermion-number-violation, madgraph-oracle, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
Branch `sprintl-majorana` off main b2434a0 (brief 2fe3109). WIP checkpoint
fa8f071 (pushed at the manager's request), then the session commit named at the
end. MadGraph tables generated with the pinned checkout (3.7.1 development line,
b7687064) through `mg5_pinned.sh`; no 3.5.7 run is behind any banked number.

## MadGraph's conventions, from source

Pinned checkout `research/refs/mg5amcnlo` at b7687064 (3.7.1 development line;
the standalone tables below were generated with it through `mg5_pinned.sh`).

1. **External Majorana wavefunctions start as particles.** An external
   wavefunction's `state` and `is_part` come from its leg; a self-conjugate
   fermion is `is_part` (positive PDG) — `HelasWavefunction.set`, `particle`
   branch (`madgraph/core/helas_objects.py:915-922`). Its HELAS call takes
   `state_id = -(-1)**get_with_flow('is_part')` (`:1508-1509`), so an initial
   Majorana is `IXXXXX(nsf=+1)` and a final one `OXXXXX(nsf=+1)` until a clash
   flips it.
2. **A clash flips the flow back to the outermost Majorana.** When two fermion
   mothers of an amplitude or wavefunction have the same `get_with_flow('state')`
   (`HelasWavefunctionList.check_and_fix_fermion_flow`, `:2301-2402`), the line is
   followed toward its external leg (`check_majorana_and_flip_flow`, `:1207-1404`):
   wavefunctions before the outermost Majorana swap `state` and `is_part` (the
   Majorana itself becomes `IXXXXX(nsf=-1)`/`OXXXXX(nsf=-1)`, the conjugate
   spinor), those after it flip `fermionflow`. With no Majorana on the line (a
   Dirac fermion-number-violating vertex) it retries with `force_flip_flow = True`
   (`:2393`) and flips `fermionflow` of the first clashing mother down to its
   external leg.
3. **Conjugate routines where the flow runs against the vertex.**
   `get_conjugate_index` (`:2144-2176` for a wavefunction, `:3152-3173` for an
   amplitude) tags fermion pair `k` of the vertex with `C<k>` when either fermion of
   the pair has `fermionflow < 0`, or (`majorana_conjugates`, `:2468-2500`) the pair
   is two *different* Majoranas in the wrong in/out order. A pair of a Majorana
   and a Dirac fermion of positive flow, and a pair of two identical Majoranas,
   are never conjugated.
4. **ALOHA's conjugate routine is `C Γᵀ C⁻¹`.** `apply_conjugation`
   (`aloha/create_aloha.py:196-230`) multiplies the kernel by
   `C(new, old+1) · Γ · C(new+1, old)`, documented there as `C Γᵀ C⁻¹`, and
   swaps the pair's in/out roles. For a structure of more than two fermions it
   refuses only when the structure's fermion lines are not `(1,2)(3,4)…`:
   "Unable to deal with 4(or more) point interactions in presence of majorana
   particle/flow violation" (`:206-214`). **Correction to the backlog item and
   the brief:** MadGraph does not refuse every Majorana four-fermion vertex,
   only those with a non-consecutive pairing. This branch still refuses all of
   them (none is validated).
5. **The fermion factor is the parity of the external numbers along the lines.**
   `calculate_fermionfactor` (`:2907-3026`) lists, per pair of fermion mothers,
   the external number reached by each (`get_fermion_order`, `:1419-1466`), the
   pair in the order its mothers are sorted for the routine call (ket first), and
   takes `sign_flips_to_order` (`:3028-3043`), the parity of the bubble sort.
6. **Octet Majoranas** (gluinos) get one more `−1` on the coupling of an
   `FVI`-type wavefunction with two octet mothers
   (`set_octet_majorana_coupling_sign`, `:1106-1121`).

## How this branch reads a line with no fermion-number arrow

Denner, Eck, Hahn and Küblbeck (Nucl. Phys. B387 (1992) 467): choose a flow on
each line, read each vertex as `Γ` along its arrow and `CΓᵀC⁻¹` against it, give
each external the spinor its flow names (`v̄` for an incoming particle read
against its fermion number), each propagator `S(p)` along the flow, and take the
parity of the external spinors in chain order. The amplitude does not depend on
the flow. MadGraph's rules 1–5 above are that prescription with one particular
flow per diagram. The evaluator already reads every Dirac line this way, with
the flow fixed by fermion number except on a crossed line, which it reads
conjugated; what it lacked was a flow on a line whose ends share a charge.

1. **Orientation** (`Diagram::orient_fermion_lines`, `diagrams/diagram.rs`). A
   line whose two end legs carry the same all-incoming charge gets one end
   flipped (`Leg::charge`): the Majorana end when exactly one end is Majorana,
   else the end of higher index. A line whose ends already differ keeps them, so
   no fermion-number-conserving diagram changes. Every downstream reader of
   `Leg::charge` (external spinor type, momentum sign, bra/ket typing, the
   pairing sign) then sees a line with one particle end and one antiparticle end.
2. **Per-vertex orientation** (`against_arrow`, `helas/eval/diagram_eval.rs`).
   The rooting decides `Γ` against `CΓᵀC⁻¹` (the chirality conjugation of a
   gamma-chained projector) from `LegAdjoint::crossed`, a per-line bit. It is now
   recomputed per vertex from where the pair's input spinor is bound: against the
   arrow exactly when the spinors sit at the slots their adjoints name. On every
   line that keeps fermion number this equals the crossed bit; a `debug_assert!`
   in `Baker::bake_node` holds that on every Dirac diagram the suite compiles
   (2255 hermetic tests, debug assertions on). Only Majorana and FNV lines can
   differ from vertex to vertex.
3. **Sign.** The pairing order `fermion_pairing_sign` uses differs from
   MadGraph's ket-first fermion factor by `−1` per incoming leg read as an
   antiparticle, a constant per process while lines keep fermion number. An
   oriented line can read an incoming leg either way from one diagram to the next,
   so each incoming leg read against its PDG charge (a Majorana's PDG charge being
   a particle's) takes one more `−1`.
4. **Axial current.** A `Gamma` chained to a `Gamma5` (`γ^μγ⁵`) is split into
   `γ^μP_R − γ^μP_L` before rooting (`split_chained_gamma5`): a `Gamma5` node is
   never chirality-conjugated, and against the arrow `C(γ^μγ⁵)ᵀC⁻¹ = +γ^μγ⁵`
   while the vector part flips. No Dirac structure in a validated model is a
   single gamma beside a `Gamma5`; the toy axial `nt nt vt` vertex is the only
   one, and without the split its s-channel diagram is off by exactly `−1`.
5. **Hermitian-conjugate partner** (`conjugate_standalone_projectors`). Diagram
   enumeration matches a vertex to its legs read all-incoming; MadGraph reads them
   all-outgoing. Where a vertex's particles are not their own conjugate set, the
   two pick the two hermitian-conjugate halves of the interaction, and MadGraph
   evaluates `γ⁰Γ†γ⁰` of the matched structure with the matched coupling (its
   `ee_to_n1n2_mssm` HELAS call `FFS2_3(…, GC_408)` reads V_399's coupling with
   the opposite chirality of V_399's `FFS1`). `γ⁰Γ†γ⁰ = Γ` for every gamma or sigma
   chain and the identity; for a bare projector it is the other chirality, and
   (by `P_χ = (1 ± γ⁵)/2`) for a bare `Gamma5` its negative. **This is also a
   Dirac fix**: main gets the bottom exchange of MSSM `t t~ > H+ H-` wrong
   (overlap 0.85 with MadGraph's `AMP(5)`), because the `t~ b H+` vertex carries
   different couplings on its two chiralities. No Standard-Model vertex has a
   bare single-chirality projector on a non-self-conjugate vertex, so SM rows do
   not move (see the bit-for-bit section).
6. **Refusals that stay or are new** (`check_fermion_flow`, `ConvertError`):
   - a Majorana or FNV pair on a vertex of more than two fermions
     (`MajoranaFermion`, `FermionNumberViolation`, now naming the vertex);
   - a vertex of two identical Majoranas whose structure is not its own charge
     conjugate (`MajoranaPairNotSelfConjugate`): read from either end it gives
     opposite values, and MadGraph reads `Γ` along whatever flow its generation
     picks. `LorentzStructure::charge_conjugation_parity` decides it; the toy
     model's `nt nt vt` vertex was a bare `γ^μ` (a Majorana has no vector
     current) and is now the axial current;
   - a decay-chain diagram with a Majorana or FNV line (stitching orients core
     and decays separately; not validated);
   - the UFO `C` operator (unchanged; none of the census-family models fetched
     writes one: RPVMSSM_UFO, Sdd_diquark_UFO, LQ_UFO, SM_HeavyN_AllMassive_LO,
     SeesawTypeIII_M-100_UFO, MSSM_SLHA2).

## The oracle: per diagram × per helicity against MadGraph `AMP()`

Sixteen standalone tables, each banking `AMP()` with JAMP(1)'s coefficient at 3
RAMBO points × every helicity, enforced by `vibegraph-lib/tests/standalone_jamps.rs`
(single `G` fitted over every graph entry; every graph `|ours − G·c·AMP| / max`
below `1e-12`). Generated with

```
export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH
export VG_MODELS_DIR=<copy of validation/ufo models>
python validation/madgraph/gen_standalone_jamps.py <key> --work <dir>
```

(the toy rows have no restrict card: `Row.restrict = None`, recorded as
`"restrict_card": null`). The diagnosis behind the pairings, every diagram
against every graph with its complex ratio and overlap:

```
VG_PROBE_KEY=<key>[,<key>…] cargo test -p vibegraph-lib --test standalone_jamps \
    probe_graph_ratios -- --ignored --nocapture
```

Output on the session's final code (every diagram pairs one graph at overlap
1.000000000000 with a ratio of modulus 1; one phase per process):

| table | process | model | what it holds | diagrams | ratio |
|---|---|---|---|---|---|
| `ll_to_ntnt_toy_majorana` | `lt~ lt > nt nt` | toy | external Majoranas; axial `nt nt vt` on a final line of two Majorana ends; t/u lines Dirac–Majorana | 3 | −i |
| `ll_to_scscx_toy_majorana` | `lt~ lt > sc sc~` | toy | Majorana propagator only | 1 | −i |
| `scscx_to_ntnt_toy_majorana` | `sc sc~ > nt nt` | toy | two Majorana ends through a Dirac propagator | 2 | +i |
| `ntl_to_ntl_toy_majorana` | `nt lt > nt lt` | toy | initial Majorana, mixed lines | 3 | +i |
| `ntnt_to_llx_toy_majorana` | `nt nt > lt lt~` | toy | initial–initial Majorana line | 3 | +i |
| `ll_to_ntntvt_toy_majorana` | `lt~ lt > nt nt vt` | toy | Majorana propagators between axial and scalar vertices | 12 | +i |
| `ab_to_ab_toy_fnv` | `at bt > at bt` | toy FNV | two particles at the diquark vertex, beside vector exchange | 2 | +i |
| `ab_to_dqvt_toy_fnv` | `at bt > dq vt` | toy FNV | Dirac propagator between vector and FNV vertex | 2 | −i |
| `dqxvt_to_abx_toy_fnv` | `dq~ vt > at~ bt~` | toy FNV | two final antiparticles from the FNV vertex | 2 | −i |
| `ee_to_n1n2_mssm` | `e+ e- > n1 n2` | MSSM_SLHA2 | chiral `n2 n1 Z` on a two-Majorana final line; chiral selectron vertices | 5 | −i |
| `uux_to_n1n1_mssm` | `u u~ > n1 n1` | MSSM_SLHA2 | axial `n1 n1 Z`, t/u squarks | 5 | −i |
| `udx_to_x1pn1_mssm` | `u d~ > x1+ n1` | MSSM_SLHA2 | Dirac chargino end + Majorana end, chiral `x1 n1 W` | 3 | −i |
| `en1_to_en1_mssm` | `e- n1 > e- n1` | MSSM_SLHA2 | initial neutralino; selectron s/u channels | 5 | +i |
| `ttx_to_hphm_mssm` | `t t~ > H+ H-` | MSSM_SLHA2 | Dirac control: `t~ b H+` with different chiral couplings | 5 | −i |
| `ee_to_n1n3_mssm` | `e+ e- > n1 n3` | MSSM_SLHA2 | external neutralino of negative mass (`Mneu3 = −363.8`) | 5 | −i |
| `x1px1m_to_wpwm_mssm` | `x1+ x1- > w+ w-` | MSSM_SLHA2 | all four neutralinos (n3 negative) as t-channel propagators on chiral `x n W` vertices | 8 | −i |

(The MSSM model used is the pinned submodule's `models/MSSM_SLHA2` with
`restrict_default.dat`; it spells the Goldstone flag `goldstoneboson`, which main
reads. Every CMS-store model fetched below spells it `goldstoneboson` too.)

The two negative-mass rows were probed (`VG_PROBE_KEY=ee_to_n1n3_mssm,x1px1m_to_wpwm_mssm`):
every diagram at overlap 1.000000000000 and ratio −i, except that the matcher put
`x1px1m` diagram 2 (the `h02` exchange) on `AMP(2)` at ratio +0.0193i, because the
`h01` and `h02` s-channel graphs are proportional at every point; the banked
pairing holds diagram 2 against `AMP(3)`, and the gate passes (below).

### The gate's own numbers

`cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture`, the first
fourteen rows (log `/home/user/wt/logs/lm1-mutations.log`, "baseline"):

```
ab_to_dqvt_toy_fnv: 36 flow entries, G=(7.8e-35,-1.0), worst flow 1.839e-16, worst |M|² 3.036e-16; 72 graph entries, worst graph 1.891e-16
dqxvt_to_abx_toy_fnv: 36 flow entries, G=(-1.3e-17,-1.0), worst flow 3.223e-16, worst |M|² 4.514e-16; 72 graph entries, worst graph 6.590e-16
ll_to_ntnt_toy_majorana: 48 flow entries, G=(5.3e-18,-1.0), worst flow 1.208e-15, worst |M|² 1.413e-15; 144 graph entries, worst graph 9.971e-16
ab_to_ab_toy_fnv: 48 flow entries, G=(1.1e-20,1.0), worst flow 2.825e-16, worst |M|² 4.664e-16; 96 graph entries, worst graph 5.606e-16
ll_to_scscx_toy_majorana: 12 flow entries, G=(0,-1.0), worst flow 1.965e-16, worst |M|² 2.781e-16; 12 graph entries, worst graph 1.965e-16
ll_to_ntntvt_toy_majorana: 144 flow entries, G=(3.7e-18,1.0), worst flow 1.224e-15, worst |M|² 7.080e-16; 1728 graph entries, worst graph 6.060e-16
ntl_to_ntl_toy_majorana: 48 flow entries, G=(-1.7e-17,1.0), worst flow 3.601e-16, worst |M|² 5.497e-16; 144 graph entries, worst graph 4.191e-16
ntnt_to_llx_toy_majorana: 48 flow entries, G=(-3.7e-18,1.0), worst flow 1.313e-15, worst |M|² 1.595e-15; 144 graph entries, worst graph 1.483e-15
scscx_to_ntnt_toy_majorana: 12 flow entries, G=(3.4e-18,1.0), worst flow 9.335e-16, worst |M|² 1.442e-15; 24 graph entries, worst graph 7.980e-16
ee_to_n1n2_mssm: 48 flow entries, G=(2.2e-17,-1.0), worst flow 1.951e-15, worst |M|² 3.031e-15; 240 graph entries, worst graph 1.885e-15
en1_to_en1_mssm: 48 flow entries, G=(-1.4e-17,1.0), worst flow 1.260e-15, worst |M|² 2.530e-15; 240 graph entries, worst graph 1.247e-15
udx_to_x1pn1_mssm: 48 flow entries, G=(2.0e-16,-1.0), worst flow 3.328e-14, worst |M|² 1.222e-14; 144 graph entries, worst graph 1.702e-14
ttx_to_hphm_mssm: 12 flow entries, G=(9.9e-17,-1.0), worst flow 2.144e-15, worst |M|² 5.663e-16; 60 graph entries, worst graph 3.090e-15
uux_to_n1n1_mssm: 48 flow entries, G=(-8.7e-18,-1.0), worst flow 4.980e-16, worst |M|² 8.273e-16; 240 graph entries, worst graph 4.878e-16
test result: ok. 14 passed; 0 failed
```

All sixteen rows, with every other target, on the committed code
(`/home/user/wt/sprintl-majorana-mg/gate.sh`, logs `/home/user/wt/logs/lm1-gate2-*.log`):

```
cargo fmt --all --check                                  rc=0
cargo test --workspace --no-fail-fast                    rc=0  2257 passed, 0 failed, 27 ignored (70 targets)
cargo clippy --workspace --all-targets -- -D warnings    rc=0
```

`pixi run validate` was not run (CI's banked job runs it). The diagram gate was
run once on fa8f071 plus the then working tree:
`cargo test -p vibegraph-lib --features extended-validation --test validate_madgraph_diagrams`
→ `63 passed; 0 failed`.

### Each ingredient is seen: one-at-a-time mutations

`/home/user/wt/sprintl-majorana-mg/mutations.sh` undoes one ingredient in the
working tree, runs the fourteen rows, restores (log
`/home/user/wt/logs/lm1-mutations.log`). Every mutation fails rows at O(1):

| mutation | rows that fail (worst graph) |
|---|---|
| M1: `crossed` per line, not per vertex (`against_arrow` off) | `ntl_to_ntl` (1.57), `ll_to_ntntvt` (0.40), `ntnt_to_llx` (1.33), `en1_to_en1` (0.080) |
| M2: no `−1` per incoming leg read against its PDG charge | `ab_to_ab` (0.92), `ntl_to_ntl` (0.91), `en1_to_en1` (0.99) |
| M3: no hermitian-partner chirality exchange | `en1_to_en1`, `udx_to_x1pn1`, `ee_to_n1n2`, `uux_to_n1n1` (1.00 each), `ttx_to_hphm` (0.41) |
| M4: no `γ^μγ⁵` split | `ll_to_ntnt` (1.43), `ll_to_ntntvt` (1.18), `ntl_to_ntl` (1.57), `ntnt_to_llx` (1.33) |

The orientation rule itself (which end of a same-charge line is flipped) has no
mutation here: by Denner's invariance either choice gives the same amplitude,
and no test flips it (see Found).

## What each test cannot see

- **The per-diagram gate** (one complex `G` per table): a phase common to every
  diagram of a process; any helicity or phase-space region outside 3 RAMBO points;
  two compensating errors inside one MadGraph graph (every row here pairs one
  diagram to one graph, so none). Every coupling in these models is `i × real`, so
  the gate cannot tell a coupling from its complex conjugate: the hermitian-partner
  exchange (rule 5) keeps the matched coupling, as MadGraph does, and a model with
  complex mixing (CP violation) would read the conjugate on both sides — agreement
  with MadGraph there would not be agreement with the field theory.
- **Widths**: every MSSM and toy row is compared at zero widths (spacelike
  propagators); the width negation for a negative Majorana mass is not reached.
- **The `debug_assert!` that the per-vertex bit equals the crossed bit** on Dirac
  lines: it is evaluated only on diagrams the suites compile, and not in release.
- **The C-parity refusal** (`charge_conjugation_parity`): reads chains of at most one
  `Gamma`/`Sigma`; anything else returns `None` and is refused, so a C-even
  two-gamma structure on an identical Majorana pair is refused rather than
  validated. It ignores colour, so the gluino's `g go go` vertex (`γ^μ`, C-odd,
  times the antisymmetric `f^{abc}`, symmetric overall) is refused.
- **The CLI tests** (`cli_fermion_flow.rs`) show only that the toy processes
  integrate to a finite σ, not the σ.
- **The σ comparison**: one process, one energy; a sampler blind spot common to
  five seeds would not show (the seeds agree with MadEvent at 0.05%).

## Nothing else moves: Dirac bit for bit

`/home/user/wt/sprintl-majorana-mg/bitforbit.sh`: main b2434a0 built in a separate
worktree and target (both deleted afterwards) against the branch (fa8f071 plus
the working tree of this commit's code), `vibegraph integrate --fixed-budget
--neval 20000 --niter 5`, iseed 7:

```
ee_ee   (e+ e- > e+ e-, 500 GeV, config_share_ee_ee_run_card.dat)
  main   grid_sha256=12c4c3ce9571deb1c67befc2cb28ce127f91cfb87df586978bc56bd6722b5e6a  σ = 156.14 ± 0.29 pb
  branch grid_sha256=12c4c3ce9571deb1c67befc2cb28ce127f91cfb87df586978bc56bd6722b5e6a  σ = 156.14 ± 0.29 pb
ee_uuxg (e+ e- > u u~ g, config_share_ee_uuxg_run_card.dat)
  main   grid_sha256=a7c249da3f9f7bd78cc5516aeee29ce3b4b899d38eedb866f1230de3fedfabdd  σ = 347.3 ± 1.2 fb
  branch grid_sha256=a7c249da3f9f7bd78cc5516aeee29ce3b4b899d38eedb866f1230de3fedfabdd  σ = 347.3 ± 1.2 fb
dy      (p p > e+ e-, 13 TeV, dy13_default_run_card.dat, NNPDF31 lhaid 247000)
  main   grid_sha256=37185d8cc8b3e0a9ebaa4220e4624dde679d9920cfa886fb59d16b569f3240b3  σ = 934.2 ± 5.0 pb
  branch grid_sha256=37185d8cc8b3e0a9ebaa4220e4624dde679d9920cfa886fb59d16b569f3240b3  σ = 934.2 ± 5.0 pb
```

The run was made before the last two table rows and a doc comment in
`ufo/bind.rs` were added; neither touches a non-test code path. The hermetic suite's
existing Dirac rows (`standalone_jamps`, `amplitude_oracle`, the `validate_madgraph_diagrams`
63 rows) pass unchanged. **Not bit for bit, by design:** any vertex whose particles
are not their own conjugate set and which carries a bare single-chirality
projector or bare `Gamma5` (rule 5) — main is wrong there (`ttx_to_hphm_mssm`).

## σ on one row

`lt~ lt > sc sc~` (toy Majorana model, the internal Majorana propagator), √s =
500 GeV, all parameters at the model's defaults. MadEvent (pinned 3.7.1 dev,
`/home/user/wt/sprintl-majorana-mg/sigma_me.sh`, three seeds, `generate_events`):

```
seed 11 Cross-section : 0.514  +- 0.000645  pb
seed 12 Cross-section : 0.5146 +- 0.000743  pb
seed 13 Cross-section : 0.5134 +- 0.0005826 pb
```

vibegraph (`sigma.sh`, release binary of the branch, default convergence, iseed 1–5):

```
σ = 513.55 ± 0.16 fb
σ = 513.75 ± 0.16 fb
σ = 513.70 ± 0.16 fb
σ = 513.88 ± 0.16 fb
σ = 513.83 ± 0.16 fb
```

Unweighted mean 513.74 fb, seed sd 0.13 fb (standard error 0.057 fb), χ²/dof
against the mean 0.64 (4 dof). MadEvent's three seeds: mean 514.0 fb, sd 0.6 fb
(standard error 0.35 fb). Difference −0.26 ± 0.36 fb (−0.05%, 0.7σ). The earlier
unvalidated 0.5158 ± 0.0023 pb sits within its own error of both.

## Campaign: census families

The census's per-card list lives on the CERN host, so the representative
processes below stand in for its families, run with the branch's release binary
(`/home/user/wt/sprintl-majorana-mg/campaign.sh`, `integrate --fixed-budget
--neval 2000 --niter 2`, p p at 13 TeV unless noted). Models are the CMS model
store's (`https://cms-project-generators.web.cern.ch/cms-project-generators/`) or
the pinned MSSM_SLHA2; every one spells the Goldstone flag `goldstoneboson`.

| family | model | process | result |
|---|---|---|---|
| neutralinos / higgsinos | MSSM_SLHA2 | `p p > n1 n2`, `p p > n2 n2`, `p p > x1+ n1` | integrate |
| gluinos | MSSM_SLHA2 | `p p > go go` | refused: `V_57` joins two `go` with `FFV1`, not its own charge conjugate (colour ignored, see Found) |
| RPV | RPVMSSM_UFO | `d s > d s`, `p p > n1 n1` | integrate |
| diquark | Sdd_diquark_UFO | `d d > d d`, `d d > omega d` | integrate |
| leptoquark | LQ_UFO | `p p > r23 r23~` | integrates; `u e- > u e-` needs a lepton beam (card error, mine) |
| HNL | SM_HeavyN_AllMassive_LO | `p p > e+ N1` | integrates; `u d~ > e+ e+ d~ u` enumerated no diagram at default orders (not investigated) |
| type-III seesaw | SeesawTypeIII_M-100_UFO | `p p > tr0 tr-` | not run: the directory's `-` was read as a restriction (`SeesawTypeIII_M`), a card error of mine |

None of these is compared with MadGraph; they say only that the refusal is gone
and the amplitude compiles and integrates.

## Commands a follow-up validation-dev session starts from

- Tables: `validation/madgraph/standalone/{ll_to_ntnt,ll_to_scscx,scscx_to_ntnt,ntl_to_ntl,ntnt_to_llx,ll_to_ntntvt}_toy_majorana.json`,
  `{ab_to_ab,ab_to_dqvt,dqxvt_to_abx}_toy_fnv.json`,
  `{ee_to_n1n2,uux_to_n1n1,udx_to_x1pn1,en1_to_en1,ttx_to_hphm,ee_to_n1n3,x1px1m_to_wpwm}_mssm.json`;
  rows in `gen_standalone_jamps.py`. MadGraph work areas (not committed):
  `/home/user/wt/sprintl-majorana-mg/standalone/<key>/`.
- Regenerate one: `PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH
  VG_MODELS_DIR=<copies of validation/ufo models> python
  validation/madgraph/gen_standalone_jamps.py <key> --work <dir>`.
- Per-diagram dump: `VG_PROBE_KEY=<keys> cargo test -p vibegraph-lib --test
  standalone_jamps probe_graph_ratios -- --ignored --nocapture` (dumps of this
  session: `/home/user/wt/logs/lm1-probe2.log`, `lm1-probe3.log` (A/B of rule 5),
  `lm1-probe4.log`).
- Mutations: `/home/user/wt/sprintl-majorana-mg/mutations.sh`.

## Long-run requests (vg-validator)

None needed for what is banked here. If the manager wants the σ reference banked
as a seeded MadEvent record: branch `sprintl-majorana` at the session commit, model
`validation/ufo/vibegraph_toy_majorana_UFO` (no restrict card), `generate lt~ lt >
sc sc~`, run card lpp1 = lpp2 = 0, ebeam 250/250, five seeds through
`bank-seeded-sigma`; expected 0.5140 pb ± 0.1%.

## Found

1. **Charged-scalar Dirac vertices were wrong on main.** Any vertex whose particles
   are not their own conjugate set with a bare `ProjM`/`ProjP` of different
   couplings (2HDM/MSSM `t~ b H+`) read the wrong chirality: MSSM `t t~ > H+ H-`
   misses MadGraph's bottom exchange (overlap 0.85). Fixed here (rule 5), banked as
   `ttx_to_hphm_mssm`. A σ survey of charged-Higgs cards on main would have been
   wrong; worth naming in the release notes.
2. **Gluinos stay refused.** The identical-Majorana C-parity check ignores colour;
   `g go go` is C-odd in Lorentz and odd in `f^{abc}`, so symmetric overall.
   Closing needs the check to read the colour structure's exchange parity and a
   per-diagram table (`g g > go go`), with MadGraph's octet-Majorana coupling sign
   (`set_octet_majorana_coupling_sign`, `helas_objects.py:1106`) pinned.
3. **Four-fermion Majorana/FNV refused wholesale**; MadGraph handles consecutive
   pairings. Needs a Majorana contact fixture and a table.
4. **Decay chains with Majorana or FNV lines are refused** at stitching (e.g.
   `p p > n2 n1, n2 > n1 e+ e-`). Needs orientation across the stitched propagator
   and a decay-chain table.
5. **No test flips the orientation choice.** A test that orients each same-charge
   line from the other end and asserts identical amplitudes would pin Denner's
   invariance in this evaluator directly, independent of MadGraph.
6. **Complex couplings.** Rule 5 keeps the matched coupling, as MadGraph does;
   with complex mixing both sides read `c` where the field theory reads `c*`. A
   CP-violating MSSM card against an independent reference (or a hermiticity
   check on the UFO) would decide whether to conjugate.
7. **KB pages now stale** (links repointed to this report, text not updated):
   `amplitudes/fermion-flow-and-crossing.md`, `amplitudes/fermion-line-sign.md`
   (Caveats), `model/ufo-aloha-type-matrix.md` row 2,
   `model/ufo-authoring-for-madgraph.md`, `validation/toy-ufo-models.md` (the
   Majorana and FNV toys are now gated, and `nt nt vt` is axial),
   `validation/cms-gridpack-refusals-202a540.md` row 14,
   `backlog/validation/negative-mass-rules-undecided.md` (its blocker is gone; a
   negative Majorana mass is now read by amplitudes and agrees at zero width).
8. **Heavy-lock starvation.** A sibling's `generate_events` held
   `/home/user/wt/heavy.lock` for over 70 minutes (PID 11224), and `flock` is not
   FIFO: this session's first MadGraph row waited ~45 minutes. A queue with a
   per-hold time limit would keep the 20-minute rule enforceable.
9. **Campaign cards to rerun** properly: `u e- > u e-` with lepton beams,
   SeesawTypeIII with a model directory name without `-`, an HNL
   lepton-number-violating `p p > e+ e+ j j` at the census's orders.
