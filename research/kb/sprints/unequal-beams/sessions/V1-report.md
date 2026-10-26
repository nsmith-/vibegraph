---
type: Session Report
title: "V1 report: unequal beams, what moves and how it is checked against MadGraph"
description: "No banked cell can move: every validation card has equal beams. Two guards added (a proton beam below 0.938 GeV refused at parse; generate refuses an artifact at a stale √s), six mutations caught. Banking request for a 6500 × 4000 Drell-Yan σ row. MadGraph's massive √stot differs by 1.0e-9 at 6500 × 4000 and by nothing at equal beams, not the 2e-8 the brief quoted."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-21}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
---
Worktree `/home/user/wt/S`, branch `sprintd-unequal-beams`. Scope items 1–5
are done. The manager's mid-session request to merge `origin/main` and fix
`edge_probe.rs` was **not done**: the session's permission layer refused
`git fetch`, `git merge` and `git show origin/main:…` (see "Merge of main"
below). Nothing in `vegas.rs`, `budget.rs`, the phase-space maps or F1's
kinematics was touched; no backlog item was edited.

## 1. What can move

F1 changed two things: proton √s (`E₁ + E₂` → `2√(E₁E₂)`) and the proton lab
frame (`build_frames`). The fixed-energy path (`FixedBeams`) is untouched.
A gate can move only if it reaches `ProtonBeams` / `ProtonIntegrand` or the
CLI's proton `integrate`/`generate`, **and** its card has `E₁ ≠ E₂`.

Machine check over every committed validation input (all `.mg5`, `.dat`,
`.sh`, `.json`, `.py`, `.toml` under `validation/`), comparing the set of
`ebeam1` values with the set of `ebeam2` values per file:

```
$ for f in $(git ls-files validation | grep -E '\.(mg5|dat|sh|json|py|toml)$'); do …; done
DIFF validation/madgraph/scripts/pp_to_jj.mg5: [6500 ] [1 6500 ]
scan-done
```

The single hit is a false positive: the comment text `ebeam1 = ebeam2 = 6500`
(line 18) contributes the `1`; its `set` lines are 6500 / 6500. Scripts that
set no beam energy inherit MadGraph's 6500 / 6500 default. So **every banked
card has equal beams**, and F1's bit-identity argument covers all of them
(`2·√(fl(E·E)) = 2E` to the bit, pinned at every banked beam energy by
`equal_beams_collide_at_the_energy_sum_to_the_bit`; the lab frame to the bit by
`equal_beams_build_the_energy_sum_lab_frame_to_the_bit`).

| gate (file) | layer | reaches | beams |
|---|---|---|---|
| `proton.rs`, `multiplicity.rs` unit tests (seeded proton integrations) | hermetic | `ProtonIntegrand`, `build_frames` | 6500 × 6500 literals; F1's unequal test at 6500 × 4000 is new |
| `runcard.rs` unit tests | hermetic | the new parse guard | default 6500 × 6500; new test at 6500 × 4000, 0.938 × 6500 |
| `cli_hard_errors` (new test), `cli_pdf_cache`, `cli_first_run` | hermetic | CLI proton path (refusals before integration) | dy13 cards 6500 × 6500; new test 6500 × 4000 / 6500 × 6500 |
| `validate_hadronic` (DY σ `dy13_default`, `dy13_mmll_60_120`; llj, bb, jj, scalefact2, MLM, draw rows) | banked | `ProtonIntegrand` with the **constant** `BEAMS = 6500 × 6500`, not the card's | equal |
| `decay_chain_ladder`, `cli_integrate`, `cli_decay_chain_events` | banked | `ProtonBeams::from_run_card` | decay_chain_pp13*, dy13 cards: 6500 × 6500 |
| `cli_generate_proton`, `validate_samples_proton`, `cli_reweight_proton`, `cli_pdf_cache_fetched`, `cli_onshell_veto` (pp row) | banked | CLI `integrate`/`generate` (`from_run_card`) | pp_to_llj*, dy13, MLM cards: 6500 × 6500 |
| `validate_scales` / matched record fields (`collider_energy`, `scales.rs`) | banked | MadGraph's massive `√stot`, a separate function F1 did not touch | equal |
| `validate_sigma`, `validate_samples`, `validate_lhef`, `validate_vegas`, `rambo_flat_mc`, onshell/grammar/decay fixed-beam rows | banked | `FixedBeams` only | unaffected by F1 |

**What CI's banked job on this branch must show:** every banked cell
unchanged, the bit-for-bit (seeded) proton rows bit-for-bit and each
`REL_TOL`/pull cell at its previous reading. A moved number on any proton row
would contradict the bit-identity argument and is a bug in F1, not noise. The
report's proton σ cells should read the same digits as on `main`.

This worktree has no PDF sets, no `validation/madgraph/output` and no
`research/refs/mg5amcnlo` content, so the banked layer was not run here.

## 2. Guard: a proton beam below the proton mass

MadGraph (b7687064), `madgraph/various/banner.py:4676-4690`,
`RunCardLO.check_validity`: for `lpp ∈ {1, 2}` and no ion mass, an `ebeam`
below 0.938 is `InvalidRunCard("Energy for beam %i lower than proton mass")`,
except exactly 0, which it rewrites to 0.938 ("At-rest proton mode"). Then
`genps.f:670-671` clamps `ebeam` up to the beam mass.

vibegraph's proton kinematics are massless, so the at-rest proton is not
representable; `RunCard::from_values` refuses any proton-beam (`lpp = (1,1)`)
energy that is not finite and `≥ 0.938` with
`RunCardError::ProtonBeamBelowMass { name, value }`:

```
run card sets ebeam1 = 0 on a proton beam: a proton beam carries at least the
proton mass, 0.938 GeV. MadGraph refuses an energy below it, and runs 0 as a
proton at rest (banner.py, RunCardLO.check_validity), a massive beam that this
build's massless proton kinematics cannot represent
```

`mass_ion1/2` is already refused when non-default (`classes.rs`), so the
proton-mass line is the only one that applies. `for_decay` sets `lpp = 0` and
is unaffected. Test `runcard::tests::a_proton_beam_below_the_proton_mass_is_refused_naming_it`:
0, −6500, 0.9, NaN and inf are refused naming the beam and the value; 0.938 ×
6500, 6500 × 4000, and fixed-energy beams at 0.5 GeV still parse.

## 3. Guard: an artifact at a stale √s

`generate` (`vibegraph-cli/src/generate.rs`) now calls
`refuse_stale_collider_energy(&artifact, ProtonBeams::from_run_card(&rc))` on
the proton path, right after the card/PDF mismatch check and before the PDF is
loaded or anything is enumerated. It compares `artifact.sqrt_s_had` with
`2√(E₁E₂)` by `to_bits` and refuses, naming both:

```
this artifact records a collider energy sqrt(s) = 10500 GeV, and the run
card's proton beams of 6500 and 4000 GeV collide at sqrt(s) =
2*sqrt(ebeam1*ebeam2) = 10198.03902718557 GeV; its grids were trained at a
different collision energy. Re-integrate with the current build
```

**Why equal-beam artifacts banked before F1 still load:** they recorded
`E + E`, and `2·√(fl(E·E))` is the same double (F1's sweep test pins it for
every banked energy and 100 000 others in [1e-2, 1e5] GeV). An exact
comparison therefore accepts them; a tolerance would have added nothing. The
card comparison already matched `ebeam1/2`, so this check sees only the
formula change. Fixed-energy artifacts are not checked (F1 did not change
that path).

Tests:
- `generate::tests::a_proton_artifact_at_a_stale_collider_energy_is_refused`
  (unit): stale `10500` at 6500 × 4000 refused with both numbers; `2√(E₁E₂)`
  accepted; equal beams recording `6500 + 6500` accepted.
- `cli_generate_refuses_a_proton_artifact_at_a_stale_collider_energy`
  (`cli_hard_errors`, hermetic, through `main`): integrates `e+ e- > mu+ mu-`
  at fixed beams, rewrites the artifact to a proton card (default PDF set,
  member 0) and runs `generate --no-network`. Stale `10500` → refused with the
  message above. Controls: the same artifact recording `2√(E₁E₂)`, and an
  equal-beam one recording `13000`, pass the check and stop later on the PDF
  set (stderr names `NNPDF23_lo_as_0130_qed`), which proves the refusal is the
  one that fired and the check is wired before the PDF load.

## Mutation evidence

Driver `/tmp/claude-0/-home-user-vibegraph/5fc875eb-a86f-52ce-bbc1-f353ce6c8fb0/scratchpad/beamsS-V1-scripts/mutate.py`,
log `…/scratchpad/beamsS-V1-mutate.log`. Each mutation applied, the named
tests run, the file restored and touched (`restored True` after each):

| mutation | runcard unit | generate unit | CLI test |
|---|---|---|---|
| R1 proton-beam check removed | **FAILED** (`runcard.rs:1105`) | – | – |
| R2 threshold `> 0` instead of `≥ 0.938` | **FAILED** (the 0.9 case) | – | – |
| R3 `is_finite` dropped | **FAILED** (the inf case) | – | – |
| G1 guard not called from `generate` | – | ok | **FAILED** (`cli_hard_errors.rs:696`) |
| G2 guard always passes | – | **FAILED** (`generate.rs:2211`) | **FAILED** |
| G3 guard compares against `E₁ + E₂` | – | **FAILED** | **FAILED** |

G1 is the reason for the CLI test: the unit test cannot see the call site.

## 4. Banking request: one unequal-beam Drell-Yan σ row

**Card** `validation/madgraph/dy_beams_6500_4000_run_card.dat`: a byte copy of
`dy13_default_run_card.dat` with exactly two changes, `4000.0 = ebeam2` and
the header line naming the beams. Same process (`p p > e+ e-`), same PDF
(`lhapdf`, 247000), same fixed scales (91.188), same cuts (`ptl 10`,
`etal 2.5`, `drll 0.4`), same `nevents 200000`, `use_syst False`. The two rows
then differ only in the beams.

**Script**: `gen_hadronic_sigma.sh` gains a third call,
`read -r SU EU < <(run_one beams_6500_4000 dy_beams_6500_4000_run_card.dat)`,
and the JSON a third key `beams_6500_4000` (process dir
`output/dy13_beams_6500_4000`; the `dy13_` prefix is that script's
directory naming, not a claim about √s). `bank_seeded_sigma.sh`'s `dy13`
family gains the row name in its `@hadronic` list (line 171), the family map
(line 191) and the card map (lines 408-411), so the reference is a seeded
record (5 seeds at the row's budget + 2 at 4×, `hard_survey = 2`) under the
seed policy, not a single run. **Generator**: the pinned MadGraph 3.7.1
submodule (b7687064) via `mg5_pinned.sh`, macOS host as for refdata 10. The
row bundles its `Events/run_beams_6500_4000` sample from the same invocation.
MadGraph's `banner.py:1675-1685` equal-beam check is inside its Rivet block
only, so the plain run accepts 6500 × 4000.

**Manifest row** (full text):

```toml
[[process]]
key = "pp_to_ll_beams_6500_4000"
process = "p p > l+ l-"
script = "validation/madgraph/gen_hadronic_sigma.sh"
class = "multi-channel"
n_final = 2
status = "planned"
rationale = "Drell-Yan between proton beams of 6500 and 4000 GeV: the only banked run whose beams differ, so the only place the collision energy 2 sqrt(E1 E2), the parton energies x_i E_i and the lab boost cm_rap = 1/2 ln(x1 E1 / (x2 E2)) reach a comparison with MadGraph. Same process, PDF, scales and cuts as pp_to_ll's dy13_default card, byte for byte apart from ebeam2, so a disagreement is about the beams. At equal beams every one of those quantities is bit-identical to the energy-sum form, which is why no other row can see them."
artifacts = ["output/dy13_beams_6500_4000", "output/seeded_sigma/dy13_beams_6500_4000", "hadronic_sigma_reference.json", "dy_beams_6500_4000_run_card.dat", "dy13_proc_card.dat", "dy13_param_card.dat"]
[process.categories]
diagrams = { tier = "covered-by", rows = ["pp_to_ll"] }
amplitudes = { tier = "covered-by", rows = ["uux_to_mumu"] }
integrals = { tier = "banked", mode = "gate", note = "five seeds a side under the seed policy, gated as pp_to_ll's dy13 cells are: |pull| < 3 and |rel| < 0.005 together, seeds' chi2/dof < 4. <measured pull, rel, chi2/dof and MadGraph seed record go here>. Known-wrong control, measured once and recorded here: the build with the energy-sum frames (sqrt(s) = E1 + E2, no cm_rap) must fail this gate. Blind to the direction of the boost: p p is symmetric and the etal cut is |eta|, so swapping the beams leaves sigma unchanged; the samples cell carries that." }
samples = { tier = "banked", mode = "gate", note = "the same invocation's events. y(l-) and the per-beam incoming-energy KS are the columns that see the boost direction and x_i E_i per beam, which sigma cannot. <measured minimum KS / chi2 p-values go here>" }
```

Before its measurement the cells are `status = "planned"`; flip both to
measured `gate` cells in the PR that lands the bundle. If the manager prefers
it as a third variant of `pp_to_ll` rather than its own row, the cell texts
move unchanged.

**Which tests read it**:
- σ: `validate_hadronic.rs`, a third `check_dy_run("beams_6500_4000",
  "dy_beams_6500_4000_run_card.dat")` test. **This needs a code change
  first**: `adapted_integrand` builds the integrand with the file's constant
  `BEAMS` (6500 × 6500), not the card's beams; it must take
  `ProtonBeams::from_run_card(rc)` (bit-identical on the existing equal-beam
  cards). Without it the new test would integrate at 13 TeV against a
  10.2 TeV reference and fail loudly, not silently. The same file's
  `dilepton_lab` and `SQRT_S_HAD` uses (`:654`, `:1696`, `:4422`) are
  equal-beam oracles and stay.
- samples: `validate_samples_proton.rs`, a third `DY_ROWS` entry (run
  `dy13_beams_6500_4000`, events `run_beams_6500_4000`, variant
  `beams_6500_4000`). That path goes through the CLI, which already reads the
  card's beams; no code change.

**Is MadGraph's massive √stot below the σ gate's resolution?** Yes, by far,
and the brief's figure is wrong (see section 5): MadGraph's √stot differs from
`2√(E₁E₂)` by **−1.0e-9** relative at 6500 × 4000. Its massive lab boost moves
lepton rapidities by ~2e-8 absolute (cut edges at `|η| = 2.5`). The gate
resolves `|rel| < 5e-3` with MC errors ~5e-4; both effects are ≥ 10⁴ below it.

## 5. Measurement note: massless versus MadGraph's massive beams (F1 Found 2)

Computed from `genps.f:656-676` (`stot = m₁² + m₂² + 2(E₁E₂ − p₁ᶻp₂ᶻ)`,
`m = 0.938`) and `unwgt.f:145-170, 636-665` (`get_betaz` from a massive beam):

```
6500 x 6500: sqrt_s massless 13000.0             MG 12999.999999999998  rel -1.1e-16
6500 x 4000: sqrt_s massless 10198.03902718557   MG 10198.039016815881  rel -1.0168e-09
```

Analytically `stot − 4E₁E₂ = −m²(√r − 1/√r)² + O(m⁴/E²)`, `r = E₁/E₂`:
**exactly zero at equal beams** (`p₁p₂ = E² − m²`, so `stot = 4E²`), and
`−2.0e-9` in `s` (−1.0e-9 in √s) at 6500 × 4000. F1's report and the brief
said ≈2e-8 "at equal beams too"; that is wrong for √s (it is the size of the
boost residual, below). At equal beams the only √s difference is rounding,
which F1's mutation M4 saw as one ulp.

Lab boost (cm rapidity, MadGraph's `get_betaz` from a 0.938 GeV beam versus
vibegraph's `½ ln(x₁E₁/(x₂E₂))`):

```
E1    E2    x1     x2     y_MG                 y_vg                 diff
6500  4000  0.1    0.1    0.2427539124578446   0.2427539078908504   4.6e-09
6500  4000  0.3    0.05   1.1386336872520997   1.138633642504878    4.5e-08
6500  6500  0.3    0.05   0.8958797588135532   0.8958797346140275   2.4e-08
6500  6500  0.01   0.002  0.8047189751522303   0.8047189562170501   1.9e-08
```

So the written lab momenta differ from MadGraph's by `O(1e-8)` in rapidity
at equal beams as well. That is the expected residual for any future
per-event LHEF comparison tighter than ~1e-7; every current tolerance is far
above it. No kinematics were changed.

## Gate

Run on the working tree before commit, with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

```
$ cargo fmt --all --check
fmt exit=0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 20.17s
clippy exit=0
$ cargo test --workspace
test exit=0
40 `test result` lines; summed: passed 1555, failed 0, ignored 16   (F1: 1552 + the 3 new)
  test runcard::tests::a_proton_beam_below_the_proton_mass_is_refused_naming_it ... ok
  test generate::tests::a_proton_artifact_at_a_stale_collider_energy_is_refused ... ok
  test cli_generate_refuses_a_proton_artifact_at_a_stale_collider_energy ... ok
$ python3 scripts/kb.py index && python3 scripts/kb.py lint
836 concepts, 290 backlog items; 0 errors, 4 warnings
```

The four lint warnings predate this branch (`state: blocked` without
`blocked_by` on items it does not touch). This gate ran on the branch **without**
main merged, so it never compiled `edge_probe.rs`; CI's merged build will still
fail until that call site is fixed.

No banked layer here (no PDF sets, no reference bundle, no MadGraph
submodule content); section 1 states what CI must show.

## Merge of main (manager's mid-session request): not done

The manager asked for `git fetch origin main && git merge --no-ff origin/main`
and a fix at `vibegraph-lib/src/proton/edge_probe.rs:142`. The session's
permission layer denied `git stash`, `git fetch origin main`, `git merge
--no-ff origin/main` (the local `origin/main` ref, 3994351, already has #49)
and then `git show origin/main:…`. I did not work around it. What I could see
before the denials (`git grep` on `origin/main`): the only constructor call in
`edge_probe.rs` is

```
origin/main:vibegraph-lib/src/proton/edge_probe.rs:142:    let mut integ = ProtonIntegrand::new(&inp.groups, amps, &inp.evaluated, pdf, SQRT_S_HAD, MU_F)
```

and main touches none of the files this session changed (no conflict expected
in `runcard.rs`, `generate.rs`, `cli_hard_errors.rs`). The fix is
`SQRT_S_HAD` → `ProtonBeams::equal(SQRT_S_HAD)` (plus the import), which is
bit-identical: halving and doubling are exact and `2√(fl(E·E)) = 2E` is
pinned by F1's sweep. It needs a session (or the user) with permission to
merge; then `cargo clippy --workspace --all-targets -- -D warnings` and
`cargo test -p vibegraph-lib --lib proton`.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`, one `Assisted-by` line, not pushed:

- (this commit) feat(beams): refuse a proton beam below the proton mass and an artifact at a stale sqrt(s)

No merge commit and no edge_probe fix commit (see above).

## Found

1. **The CI compile break on #67 is not fixed.** `edge_probe.rs:142` on main
   passes an `f64` where F1's `ProtonIntegrand::new` takes `ProtonBeams`.
   Close: merge main and pass `ProtonBeams::equal(SQRT_S_HAD)` (section
   "Merge of main").
2. **`validate_hadronic` integrates every proton row at a constant 6500 ×
   6500, whatever the card says.** `adapted_integrand` and the other
   `ProtonIntegrand` sites use `const BEAMS`, not
   `ProtonBeams::from_run_card(rc)`. Harmless today (every card is 6500 ×
   6500) but it would make the unequal-beam row integrate at the wrong
   energy. Close: read the beams from the card at each site, bit-identical on
   the current cards. That is a prerequisite of the banking request.
3. **The doc of `collider_energy` (`coupling/scales.rs:826-830`) says "the mass
   cancels analytically, `stot = 4 E₁ E₂`".** It cancels only at equal beams;
   at unequal beams `stot = 4E₁E₂ − m²(√r − 1/√r)² + O(m⁴/E²)`. The code is
   MadGraph's formula and is right; only the claim is too broad. Close: state
   the equal-beam condition.
4. **F1's Found 2 and the V1 brief overstate the √s residual.** MadGraph's
   √stot equals `2√(E₁E₂)` exactly at equal beams and differs by 1.0e-9 at
   6500 × 4000, not ≈2e-8. The ~2e-8 is the lab-boost residual (rapidity), and
   that one exists at equal beams too. Close: record the two numbers
   separately in the beams/kinematics concept.
5. **Fixed-energy beams (`lpp = 0`) accept a non-positive or non-finite
   `ebeam`.** The new guard is proton-only by the brief's scope; a
   fixed-energy card at `ebeam1 = 0` parses (the test asserts 0.5 GeV parses;
   0 was not tried). Close: decide the fixed-beam line (MadGraph's
   `genps.f:670-671` clamps to the leg mass) and refuse below it at parse.
