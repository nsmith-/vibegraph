# 25 — Validation layering and the per-process report (`validation-3` sprint plan)

Planned 2026-07-31, following the `user-distribution`/`proton-events` close-out.

## 1. The reframing

Two structural problems, named by the user, that the backlog items are symptoms of:

1. **Undelineated dependencies.** The suite's components need different external
   inputs (the `mg5amcnlo` submodule, the 1.0 GB MG `output/` work area, fetched
   PDF sets, f2py `.so` builds), but nothing states which test needs what. Three
   different gating mechanisms coexist — `required-features` in `Cargo.toml`,
   internal `#[cfg(feature = "extended-validation")]`, and runtime
   soft-skip-with-`eprintln!` — and the third is why green can mean "data
   absent" (TODO: *silent soft-skip tests*) and why `validate-pdf-grid` covered
   nothing for four sessions.
2. **Unstated per-process coverage.** Different processes see different amounts
   of validation, often for good reasons (per-helicity amplitude checks are done
   on single concrete subprocesses precisely so we do not have to reproduce
   MadGraph's process grouping), but the reasons live in scattered script
   headers and module docs, not in one place a reader can audit. Related
   principle, now explicit: **MG's process grouping is optimization-inspired,
   not physics-motivated — we owe agreement with it only where it affects our
   ability to validate.** Validating concrete subprocesses sidesteps it.

The answer is (a) a three-layer dependency contract every test declares itself
into, and (b) a per-process × per-category report table that makes coverage —
and deliberate non-coverage — visible.

## 2. The three layers

Names (decided 2026-07-31, scheme A of the proposals in §2.4):
**`hermetic` / `banked` / `oracle`**.

### 2.1 `hermetic` — every clone, every commit

- Runs on a clean clone: **no submodule, no network, no pixi env**. `cargo test`
  with no feature flags is exactly this layer, and it must be *complete* on a
  clean clone — zero silent skips. A test that cannot run hermetically does not
  belong in the default suite at all (this is the structural fix for the
  soft-skip item: the category "runs by default but quietly needs data" ceases
  to exist).
- Reference data: small committed files. Anything a test needs that is ≤ ~100 KB
  and stable gets generated once by the oracle toolchain, projected down, and
  **tracked in git** (as `sigma_reference.json`, `amp_reference.json`,
  `alphas/reference.csv`, `rambo_fixture.json` already are).
- Budget: **≤ 3 minutes** wall for the whole hook (`fmt` + build delta + test).
  Currently 1m05 after `[profile.dev] opt-level = 2`; the new committed-reference
  gates (§5.3) must fit inside the remaining headroom.
- The one test that inspects the submodule when present
  (`interned_blob_matches_submodule_exactly`) moves out of the default suite
  into the banked layer; the committed interned blob is the hermetic truth.

### 2.2 `banked` — frozen references, CI after the hermetic gate

- May assume: the `mg5amcnlo` submodule checked out, the **fetched** large
  reference data (PDF sets, and the new *banked-reference bundle*, §5.2), and
  the pixi envs. May **not** assume MadGraph has been *run* — everything MG
  produces that this layer consumes is fetched or committed, never regenerated
  here. That is what makes it runnable in CI on a fresh runner.
- Compiled under the optimized-with-debug-info profile (§2.5). Budget:
  **≤ 15 minutes**.
- Deliverable: the **validation report table** (§3), one driver, all categories.
- Tests whose *reference generation* is expensive but whose reference *output*
  is small live in `hermetic` if the vibegraph side is quick, here if the
  vibegraph side needs minutes.

### 2.3 `oracle` — reference regeneration + heavy compute

- May assume the full oracle toolchain: the `madgraph` pixi env actually running
  `mg5_aMC`, the LHAPDF C++ build, gfortran/f2py. This is where all reference
  generation lives (§5.1), preserving the existing caching discipline: never
  regenerate what the local work area already holds (`--skip-deps` semantics).
- Also holds the vibegraph-side runs too heavy for 15 minutes: 2→6 σ rows,
  budget-convergence sweeps (the note-24 lesson: seed agreement is necessary,
  budget convergence is the second axis), and — once `kt-clustering` lands —
  the dynamical-scale re-gates.
- Cadence: rarely in CI (release tags), routinely on the dev machine when
  amplitudes/color/coupling change (the `extended-validation` skill's map keeps
  deciding *which* gates after *which* change).

### 2.4 Naming — decided

Scheme A (by dependency) was chosen over cadence-based (`commit`/`merge`/
`release`) and plain (`quick`/`standard`/`deep`) alternatives: it names the
contract each layer is allowed to assume, which is the disease being cured.
"Banked" already is the project's word for frozen MG references; "oracle" is
the project's word for the reference-producing toolchain.

Spellings: `cargo test` stays the hermetic layer (no rename — its name is its
contract); `pixi run validate` runs the banked layer and emits the report;
`pixi run generate-references` + `pixi run validate-deep` are the oracle layer.

### 2.5 The `profiling` profile rename — decided

`[profile.profiling]` is `release` + `debug = 1` — release-with-debug-info,
used both for samply *and* as the compilation profile of every heavy gate. The
name suggests it exists only for the profiler. Rename to **`release-debug`**;
`scripts/profile.sh`, all pixi tasks, and note 15's rerun kit update in the
same change. One-line migration, no semantic change.

## 3. Categories and the report table

Four per-process categories, condensing today's eleven `validate-*` tasks; the
banked-layer driver runs all of them and renders one table.

### 3.1 `diagrams`

Same set of Feynman diagrams as MG, **to the extent we choose to emulate the
grouping**. Single-channel rows: exact per-subprocess diagram count (and, where
banked, per-diagram matching). Multi-channel rows: the per-flavor
concrete-subprocess union (this is exactly the deferred **V7** design, note 19
§3/§V7, landing as this category's multi-channel half) — *or* an explicit
`covered-by: [single-channel rows]` skip when we judge the union redundant.
**Metric: `k/n` diagrams matched.**

### 3.2 `amplitudes`

Per-phase-space-point × per-helicity × per-color-flow complex amplitude
agreement — the finest linear level, per the Physics Validation doctrine.
**Change of evaluation points**: instead of a self-chosen RAMBO/fixed grid, the
points are **MadGraph's own banked events** for that process, so the check sits
exactly where the cross section actually lives (peaks, cuts, the low-m_ll
region), not where our sampler happens to look. Design details in §5.3,
including the on-shell projection the note-24 mirror lesson forces, and the
allowance: if a flow convention differs from MG's, the gate may factorize into
(per-helicity summed over flows) × (per-flow summed over helicities) rather
than the full outer product. **Metric: max relative deviation.**

### 3.3 `integrals`

σ against banked MG, **always through the generic vibegraph path — no
special-cased integrands**. Rule-based subsampler composition (s-channel BW,
t-channel, massless-log maps) is the mechanism, and the integration artifact
gains a **subsampler summary** (per channel: map kinds, pole masses/widths)
that the report reprints, so what the sampler did is part of the record.
Consequence: the bespoke `DrellYanIntegrand` stops being the gated path for
`p p > e+ e-` (§5.4). **Metric: pull `(σ_vg − σ_MG)/σ_err` + seed-sweep
stability (spread, χ²/dof over ≥5 seeds).**

### 3.4 `samples` — the sprint's new scope

Distribution-level comparison of our unweighted events against MG's banked
10k-event samples: continuous observables (m_ll and other pair invariants,
cosθ*, pT, y — per process class) via **two-sample KS**, and the discrete
frequencies nothing else covers as realised samples — `SPINUP` helicity,
`ICOLUP` colour flow, and (llj) flavour-group population — via **χ²
homogeneity**. This is the deferred E3 detector, the MG-plot comparison row,
and the `low-mll-reconciliation` decider (the DY/2→4 row bins dσ/dm_ll down to
threshold) in one mechanism. **Metric: min KS p-value across observables (χ² p
for the discrete columns).**

### 3.5 Standalone gates (outside the table)

Process-independent, run once per banked-layer invocation, reported as a short
list under the table: `alphas` (committed grid bit-for-bit + AQCDUP replay over
banked LHEs), `pdf-grid` (committed oracle JSONs, §7), `helas` kernels,
`rambo` fixture, run-card defaults transcription, LHEF byte round-trip over the
banked runs, per-event scale replay (`SCALUP`/`<rscale>`/`<pdfrwt>`), and the
**Pythia consumption** gate (§5.6, which is per-emitted-sample but is a
format/consumability property, not a physics distribution).

### 3.6 The table

Rows grouped **single-channel** then **multi-channel**, sorted by increasing
final-state multiplicity. Cells carry the metric and a mark: ✅ gated-and-green,
⚠️ informational (metric shown, known discrepancy linked), ⏳ oracle-layer-only
(deliberately not run in this layer), ⛔ blocked (named blocker), `—`
deliberately-not-covered with a `covered-by` pointer. Mock of the intended
render (values illustrative):

| process | 2→N | diagrams | amplitudes | integrals | samples |
|---|---|---|---|---|---|
| `e+e- > mu+ mu-` | 2 | ✅ 1/1 | ✅ 3.1e-15 | ✅ pull −0.4, χ²/dof 1.1 | ✅ KS p 0.41 |
| `u u~ > u u~` | 2 | ✅ 2/2 | ✅ 5.6e-14 | ⚠️ pull −1.9 (−0.30% bias, multi-rung spine) | ✅ KS p 0.22 |
| … | | | | | |
| `u u~ > c c~ e+ e- mu+ mu-` | 6 | ✅ 76/76 | ✅ 6.3e-13 | ⏳ long | ⏳ long |
| **multi-channel** | | | | | |
| `p p > e+ e-` (dy13) | 2 | ✅ 4/4 flavors | — covered-by `uux_to_mumu` | ✅ pull +1.5 | ✅ KS p 0.35 |
| `p p > l+ l- j` (fixed) | 3 | ✅ 24/24 flavors | — covered-by 4 parton rows | ✅ pull +0.01, 5 seeds | ✅ KS p … / flavour χ² p … |
| `p p > l+ l- j` (dyn) | 3 | ✅ | — | ⛔ `kt-clustering` | ⛔ `kt-clustering` |

The rendered report (markdown + a machine JSON) lands in
`target/validation-report/`; CI uploads it as an artifact.

## 4. Inventory audit (what exists today)

### 4.1 `validation/madgraph/scripts` — 25 `.mg5` scripts

Rationale headers ("Why this exact process") are **present and accurate on all
18 single-channel scripts** — spot-checked claims (NCOLOR values, mass
thresholds, crossing relations) against the pipeline table and notes; no stale
claims found. **Missing or thin on the 7 oldest**: `ee_to_mumu`, `pp_to_ll`,
`pp_to_ll_qcd0`, `pp_to_llj`, `pp_to_llj_qcd2_qed2`, `pp_to_bb`,
`pp_to_bb_qcd2` predate the convention and carry two-line headers; they get
full rationale headers in L1, including what each is *not* used for.

Division of labour among the llj triple, to be documented rather than
consolidated (all three earn their keep): `pp_to_llj` (default orders) +
`pp_to_llj_qcd2_qed2` (explicit orders) pin order-constraint semantics in
`diagrams` and serve as the asserted-refused rows in `validate_scales`;
`pp_to_llj_fixed` (same generate line, fixed-scale run card) is the
integrals/samples/events subject. Same pattern for `pp_to_bb`(+`_qcd2`):
diagrams-only today, dynamical-scale cards, so integrals/samples are ⛔ until
`kt-clustering`.

**Coverage gap the table exposes**: no multi-channel row exercises a
QCD-initiated group end-to-end (DY and llj are EW-core). A `pp_to_bb_fixed`
script (fixed-scale run card, same pattern as llj_fixed) would fill the
multi-channel `integrals`+`samples` column for a pure-QCD group at the cost of
one banked MG run. **Decided: added, as an L4 rider** — it also gives
`has_mirror()` (note 24) its first gated consumer. If it does not pass its
gate on the first try, the row lands as ⚠️ with the measurement, a debug/fix
item is filed in the backlog, and the session does not chase it (§8).

**`validation/madgraph/README.md` is badly stale**: documents 7 of 25 scripts,
a pre-pixi workflow, and a "Future Extensions" list that is entirely done or
tracked in TODO. Rewritten in L1 as a thin index that defers to the manifest
(§5.1).

**Loose build products**: f2py `.so` modules, `__pycache__`, `mg_*_debug.npz`
sit gitignored *beside* the committed sources in `validation/madgraph/`. They
move under `output/` (the acknowledged work area) so the committed surface is
legible at a glance.

### 4.2 Registration-in-N-places

`build.sh` is already dynamic over `scripts/*.mg5`, but a new process still
touches `build_amplitude.sh`, `gen_amplitude.py`, reference JSONs, and two or
three Rust plan tables (note 24 P0/P1 lived this). §5.1's manifest makes the
process list single-source.

### 4.3 Gating mechanisms

10 lib test binaries carry `required-features = ["extended-validation"]`;
`validate_helas`, `validate_alphas`, `validate_scales`, `validate_pdf_grid`,
`validate_hadronic`, and the CLI extended tests use internal `#[cfg]` or
runtime skips; the known silent soft-skips (TODO list, note 24 §U1) are the
runtime-skip residue. Target state: **`required-features` is the only
mechanism** for layer membership; runtime skips survive only *inside* the
banked layer for optional sub-inputs, and then only through a skip-accounting
helper with a per-layer expected-skip manifest asserted in CI.

### 4.4 Reference data census

| Data | Size | Today | Target |
|---|---|---|---|
| `sigma/amp/jamp/hadronic_sigma_reference.json`, `runcard_defaults.json`, `dy13_*.dat`, `rambo_fixture.json`, `alphas/reference.csv` | 4–320 KB | committed | committed (unchanged) |
| per-process amplitude CSVs (18 × 10–27 KB) | ~280 KB | gitignored in `output/` | superseded by committed MG-event amplitude tables (§5.3) |
| `diagrams.json` (counts + per-flavor unions) | ~KB | gitignored, regenerated | **committed** (unlocks hermetic `diagrams`) |
| banked `.lhe.gz`, 25 runs, 10k events each + banners + `leshouche.inc` | 90 MB | gitignored local work area | **banked-reference bundle**, fetched (§5.2) |
| PDF sets | 101 MB | fetched (`fetch.sh`) | unchanged, via shared fetch |
| `pdf/oracle*.json` | small | gitignored **and absent** — the dead gate | **committed** (LHAPDF-oracle values are stable; regeneration stays oracle-layer) |
| MG process dirs (Fortran, logs) | ~0.9 GB | gitignored work area | unchanged, oracle-layer cache |
| `helas/reference.{csv,npz}` | 27 KB | gitignored | **committed** → `validate_helas` becomes hermetic |

## 5. Refactor design

### 5.1 The manifest: `validation/manifest.toml`

One committed file, the single source of truth per process: process string,
script path, class (single/multi-channel), n_final, per-category tier
assignment (`hermetic` / `banked` / `long` / `blocked(reason)` /
`covered-by(rows)`), banked artifacts, and the rationale line (which also
replaces the README's process list). Consumed by:

- the reference generators (which processes to build, which extractions to run),
- the Rust report driver (row set, expected cells — an *asserted* table shape,
  so "a row silently vanished" is a failure, the manifest-level analogue of the
  skip-accounting rule),
- the README index (rendered from it, not hand-maintained).

### 5.2 Reference generation: one entry point + the bundle

`pixi run generate-references` (oracle layer) replaces the ad-hoc constellation
(`build.sh`, `build_amplitude.sh`, `gen_amplitude.py`, `gen_amp_reference.py`,
`gen_jamp_reference.py`, `extract_diagrams.py`, `extract_sigma.py`,
`gen_hadronic_sigma.sh`, `gen_dy_oracle.sh`, `alphas/gen_reference.sh`,
`helas/gen_reference.py`, `pdf` oracle build) with one driver over the
manifest, staged as: *ensure-deps → run-MG (cached, `--skip-deps` semantics
preserved) → extract per category → write committed refs → assemble bundle*.
The per-stage scripts survive as internals; what is consolidated is the entry
point, the process list, and the caching policy.

**Shared acquisition functions** (used by both this driver and the banked
layer's setup): submodule init, PDF fetch, bundle fetch — one shell library
(`validation/fetch_common.sh`), mirroring the CLI's `assets.rs` seam
(pinned URL + SHA-256 + cache dir + explicit consent in CI via env var).

**The banked-reference bundle**: `vibegraph-refdata-<n>.tar.zst` — the 25
banked `.lhe.gz` (90 MB), banners, `leshouche.inc` per subprocess, run logs
the replay gates read — content-addressed by SHA-256 pinned in the manifest,
published as a GitHub release asset of a `refdata-<n>` tag (same consent-gated
fetch pattern as the PDF sets). Regenerating references bumps the bundle; the
manifest pin is the compatibility contract. This is what makes the banked layer
honest in CI without committing 90 MB.

**Compact-representation investigation (session L1b, decided)**: the bundle
above is the fallback; the preferred endpoint is an event representation small
enough to commit **in-repo**, which would collapse the banked layer's largest
fetch. Path to evaluate: `pylhe` → awkward-array → filter to only the fields
the gates actually read (momenta, PDG ids, `SPINUP`, `ICOLUP`, `SCALUP`,
weights, per-event `<rscale>`/`<pdfrwt>`) → parquet with aggressive
compression. The non-momenta fields are small-cardinality integers and the
momenta are ~11 significant digits; a 10–20× reduction (90 MB → 5–10 MB) is
plausible and would be committed per process. One consumer **cannot** use it:
the LHEF byte-for-byte round-trip gate reads raw `.lhe.gz` bytes by
construction. L1b therefore also decides that gate's fate — either it moves to
the oracle layer (raw files live in the work area) or a small representative
subset of raw runs (2–3 files, a few MB) stays in the bundle/repo for it.
Verdict criteria: total committed size, and bit-faithful momenta round-trip
through the parquet (the amplitudes category's on-shell projection, §5.3,
already breaks dependence on the printed digits). If the investigation says
no, the release-asset bundle stands with no further work.

### 5.3 `amplitudes` on MG's own events

New oracle-layer generator: read each process's banked `.lhe.gz` (via `pylhe`,
per the no-hand-written-parsers rule on the Python side), take the first N
events (N ≈ 24; plus the existing fixed grid, see below), **project each event
exactly on shell** (recompute E from |p⃗| and the card mass, restore momentum
conservation deterministically), then evaluate the f2py matrix element with
per-helicity, per-diagram `AMP()`, and per-flow `JAMP()` dumps *at the
projected momenta*, writing one committed JSON per process (~15–30 KB — the
current CSVs say this fits).

- **Why project**: note 24 §P2d — LHE momenta are printed to ~11 significant
  digits, so read-back points are off shell by ~1e-10 and two independently
  compiled programs legitimately diverge there by gauge-dependent parts.
  Both sides must consume *identical, exactly on-shell* points; the committed
  file holds the projected momenta so the Rust gate never re-derives them.
- **Keep the old fixed grid too**: MG events cluster where σ lives — which is
  the point — but they under-visit off-peak corners the current grid covers.
  Both point sets are cheap; the committed file carries both, labeled.
- **Factorization allowance**: where a colour-flow convention differs
  (the NCOLOR=6 JAMP-basis caveat), the gate may check per-helicity
  (flow-summed) × per-flow (helicity-summed) instead of the outer product,
  recorded per process in the manifest, never silently.
- **Layer**: committed refs + fast eval ⇒ this whole category becomes
  **hermetic** for all 18 single-channel rows. `validate_helas_mg`,
  `amp_diagram_oracle`, `color_jamp_oracle` fold into one gate over the new
  file; `color_cf_oracle`/`color_flow_tags_oracle` (which read MG Fortran
  sources, not events) stay banked-layer standalone.

### 5.4 `integrals` genericization

- **`p p > e+ e-` routes through `ProtonIntegrand`** as the gated path, and the
  bespoke `DrellYanIntegrand` is **deleted** (decided). If the general path
  cannot hit the banked 0.14%/0.07% agreement, that is a finding: the row goes
  in as ⚠️ informational with the measured pull, a debug/fix item is filed in
  the backlog, and the session moves on — it does **not** fix the sampler
  in-sprint (§8 discipline). Side effect: DY becomes generatable (the note-24
  "generate for Drell–Yan" follow-up), which the `samples` category needs
  anyway.
- The `dy_integrand_oracle` (pointwise) is re-targeted at `ProtonIntegrand` at
  DY kinematics or retired with the bespoke path.
- **Artifact subsampler summary**: `IntegrateArtifact` gains a per-channel
  record (map kinds, poles) surfaced in the report. Schema bump fv4→fv5.
- The four llj partonic σ̂ rows (`uux/ddx_to_epemg`, `gu/gux_to_epemu`) are
  banked in `sigma_reference.json` but hit `plan_for`'s catch-all Skip —
  **promote to Gate** (2→3 at 250+250 GeV is well inside budget).
- Budget (decided): the llj sweep runs **3 seeds × 300k** in the banked layer;
  the full 5-seed sweep plus the budget ladder live in the oracle layer. The
  VEGAS first-iteration fix **stays in the backlog** — no sampler changes this
  sprint.

### 5.5 `samples` machinery

Rust-side histogram module (no plotting — counts and test statistics only):

- Continuous: two-sample KS between our unweighted sample (20k events, frozen
  banked grids, fixed generation seed swept ×3) and MG's banked 10k. Our
  Buffer-strategy overweights (weight > 1) are handled by the weighted-ECDF
  form of the KS statistic rather than pretending weights are 1.
- Observables per class: all-pairs invariant masses, cosθ* (Collins–Soper for
  DY-like), per-particle pT/y where cuts sculpt them. The DY and 2→4 rows bin
  m_ll down to threshold — the `low-mll-reconciliation` decider; its verdict
  (MG under-covers vs we over-weight) is an explicit deliverable, and the
  `Plan::Info` row flips to Gate or files a MadGraph-defect note accordingly.
- Discrete: χ² homogeneity on `SPINUP`, `ICOLUP` flow labels (via the
  `leshouche.inc` dictionary), and llj flavour-group frequencies.
- KS/χ² thresholds: fixed p-floor (e.g. 1e-3) with the seed sweep guarding
  against a lucky draw; exact choices measured in-session, never loosened
  after the fact (standing rule).

### 5.6 Pythia consumption

New pixi env (`pythia` feature, conda-forge `pythia8`) + a small driver:
`Pythia::init` on each emitted `.lhe` from §5.5's generation step (llj and,
post-5.4, DY), require every event read and the hard process reconstructed —
colour lines exercised as *input* for the first time. Metric: n/n events
consumed. Banked layer (fast); listed standalone (§3.5).

### 5.7 The report driver

Each category gate writes per-row JSON (`target/validation-report/<row>.json`,
schema in the manifest's terms); a collator (Rust dev binary) renders the
markdown table + standalone list, asserts the manifest's expected cell set, and
exits nonzero on any ❌ or on a missing/unexpected cell. `pixi run validate` =
setup (shared fetch) → category gates → collator. CI banked job uploads the
report artifact; `ci.yml`'s existing job *is* the hermetic layer already.

## 6. Per-process tier × category assignment

`H` hermetic, `B` banked, `L` long, `⛔ kt` blocked on `kt-clustering`,
`cov→` covered-by.

**Single-channel** (all: diagrams H, amplitudes H — committed refs):

| process | 2→N | integrals | samples | notes |
|---|---|---|---|---|
| `ee_to_mumu` | 2 | B Gate | B | Z-pole beams |
| `ee_to_ee` | 2 | B Gate | B | t-channel + cuts |
| `ee_to_ttx` | 2 | B Gate | B | |
| `ee_to_wpwm` | 2 | B Gate | B | |
| `ee_to_zh` | 2 | B Gate | B | |
| `uux_to_mumu` | 2 | B Gate | B | |
| `uux_to_uux` | 2 | B Gate (⚠️ −0.30% bias stays visible) | B | |
| `gg_to_ttx` | 2 | B Gate | B | |
| `gg_to_gg` | 2 | B Gate | B | amplitudes may factorize (NCOLOR=6 basis) |
| `ee_to_mumua` | 3 | B Gate | B | |
| `ee_to_tatah` | 3 | B Gate | B | |
| `uux_to_epemg` | 3 | **B Gate (promoted)** | B | |
| `ddx_to_epemg` | 3 | **B Gate (promoted)** | B | |
| `gu_to_epemu` | 3 | **B Gate (promoted)** | B | |
| `gux_to_epemux` | 3 | **B Gate (promoted)** | B | |
| `ee_to_mumu_tata_qcd0` | 4 | B Info → decider in samples | B (m_ll decider) | |
| `uux_to_ccx_emmm_qcd0` | 6 | L | L | ~1 ms/eval, 24-dim |
| `bbx_to_ccx_emmm_qcd0` | 6 | L | L | |

**Multi-channel** (diagrams H incl. per-flavor union = V7; amplitudes `cov→`
concrete-subprocess rows, stated in the manifest):

| process | 2→N | diagrams | amplitudes | integrals | samples |
|---|---|---|---|---|---|
| `pp_to_ll` / dy13 cards | 2 | H (+V7) | cov→ `uux_to_mumu` | B Gate (general path, 2 cards) | B (needs §5.4 DY generate) |
| `pp_to_ll_qcd0` | 2 | H (+V7) | H (own banked CSV row) | cov→ dy13 | cov→ dy13 |
| `pp_to_bb`, `pp_to_bb_qcd2` | 2 | H (+V7) | cov→ (none yet — see `pp_to_bb_fixed` rider) | ⛔ kt | ⛔ kt |
| `pp_to_bb_fixed` (new, L4 rider) | 2 | H | cov→ new bb̄ parton rows if added | B Gate (⚠️+backlog on first-try failure) | B |
| `pp_to_llj_fixed` | 3 | H (+V7) | cov→ 4 parton rows | B Gate (seed budget per §5.4) | B (+flavour χ²) |
| `pp_to_llj`, `pp_to_llj_qcd2_qed2` | 3 | H (+V7, order semantics) | — | ⛔ kt (asserted-refused rows stay) | ⛔ kt |

## 7. TODO backlog mapping

**Satisfied by this plan's structure** (item → where):

- Downstream Pythia validation → §5.6 (banked, standalone).
- Event-sample vs MG statistics (incl. SPINUP/ICOLUP/flavour frequencies) →
  §5.5, the `samples` category.
- MG-plot distribution comparison → same mechanism (banked `.lhe` directly; no
  MG plot toolchain needed).
- `low-mll-reconciliation` → §5.5 m_ll decider; explicit verdict deliverable.
- V7 per-flavor diagram matching → §3.1 multi-channel `diagrams` half.
- Silent soft-skip tests → §2.1/§4.3 (structural: hermetic completeness +
  skip-accounting with asserted counts).
- `validate-pdf-grid` dead gate → §4.4 (oracle JSONs committed; regeneration
  oracle-layer).
- `run_card_dy.dat` verbatim copy → L6 hygiene: read from submodule via the
  banked layer's declared dependency, loud on absence (the layering makes
  "loud skip" well-defined).
- `clippy::approx_constant` → L6 rider (targeted `#[allow]` with the
  MG-source-tracking rationale).
- Flavour-group probe hardening (energy ladder, on-m_Z point, separation
  assertion) → L6 rider in the hermetic layer; the sound s-expression criterion stays
  feature-backlog.
- Minor pinned discrepancies (`ee_to_wpwm` mask; dy fixture `fixed_ren_scale`)
  → L6 riders.
- "Generate for Drell–Yan" (note 24, un-filed) → falls out of §5.4.
- Seed-sweep + budget-convergence lesson → §3.3 metric + long-layer budget
  ladder.

**Made visible but not resolved** (stay backlog, now with a table cell):

- `uux_to_uux` −0.30% bias → ⚠️ integrals cell; resolution = multi-rung spine
  (feature).
- 2→6 σ rows → ⏳ long-layer cells (cost problem, performance backlog).
- Dynamical-scale rows → ⛔ `kt-clustering` cells (feature).
- VEGAS first-iteration bias + `w_max` scan budget → feature/perf backlog
  (decided: neither is touched this sprint; the llj banked budget is 3 seeds).

**Untouched by this sprint**: `IdentityAmp` (rides `non-sm-ufo`), weekly
`acceptance.yml` schedule (blocked on first release — user), interned-SM
`--restrict` override (CLI feature).

**Closed by user decision (2026-07-31)**: licensing is resolved on `main` at
`a1a4ae7` (dual `MIT OR Apache-2.0`); the U2 question of whether banked test
artifacts need their own source-form notice is dropped — they are *outputs* of
the MadGraph program, not redistributed MadGraph source, so no notice is owed.
The refdata bundle (§5.2) inherits this position.

## 8. Sessions

Sized for `validation-dev` agents, one session each, dependency-ordered;
L2/L3 and L4/L5 pairs can run in parallel worktrees (pre-created off main,
COW-cloned data, per the worktree-fragility rule).

**Sprint discipline — expose, don't fix.** This sprint's job is to expose and
consolidate; a follow-up session tackles the backlog it generates. Every
session prompt carries this verbatim rule: *if a gate this sprint newly
exposes fails — a distribution disagrees, the general DY path misses the
banked σ, `pp_to_bb_fixed` misses on the first try — record the measurement as
a ⚠️ informational cell, file a debug/fix item in the TODO backlog, and move
on. Do not diagnose, do not tune, do not touch physics or sampler code to make
a new cell green.* The only failures a session must fix in-session are
regressions of *pre-existing* gates caused by its own refactoring (those are
its exit gate), and the standing rule stands: never a loosened tolerance.

- **L0 — Layering skeleton + manifest.** `manifest.toml`; tier naming applied;
  `required-features` as the sole gating mechanism (move the 5 internal-cfg
  tests); skip-accounting helper + expected-skip assertion; profile rename;
  pixi entry points (`validate`, `validate-deep`, `generate-references`
  stubs); CI banked job skeleton. Gate: clean-clone `cargo test` complete with
  zero skips; the moved tests still run under the banked task.
- **L1 — Reference consolidation + bundle.** §5.2 driver + `fetch_common.sh`;
  bundle assembly + pinned fetch; commit `diagrams.json`, `helas` refs, `pdf`
  oracle JSONs; relocate loose build products; README rewrite from manifest;
  rationale headers for the 7 old scripts. Gate: fresh-clone banked layer runs
  from fetch alone (no MG execution); oracle layer reproduces the bundle
  byte-identically from the local work area.
- **L1b — Compact banked representation (investigation).** §5.2: the
  `pylhe` → awkward → field-filter → parquet pipeline, measured against the
  in-repo size and momenta-fidelity criteria; decides the byte-round-trip
  gate's home. Deliverable is a written verdict + (if yes) the committed
  parquet refs and the gates' reader switched over; if no, the bundle stands.
  Can run in parallel with L2/L3 — only L4's reader choice depends on it.
- **L2 — Amplitudes on MG events.** §5.3 generator + committed refs + unified
  hermetic gate; fold the three point-level oracles; record factorization
  choices. Gate: all 18 rows ≤ the current tolerances on the fixed grid and a
  measured (not assumed) tolerance on the event points; `extended-validation`
  skill gates green.
- **L3 — Integrals genericization.** §5.4: DY through the general path (bespoke
  integrand deleted), llj parton rows promoted, artifact subsampler summary
  (fv5), seed/pull metrics standardized into row JSON. Gate: dy13 rows
  *measured* through the general path — expected to re-gate at the banked
  agreement, but a miss is a ⚠️ + backlog entry per the discipline, with the
  deletion kept; `validate-generate-proton` unmoved.
- **L4 — Samples.** §5.5 machinery + all banked-tier cells + the low-mll
  verdict. Rider: `pp_to_bb_fixed` banked run + row (decided). Gate: KS/χ²
  *measured and reported* on every non-blocked row, 3-seed swept — per the
  discipline above, a failing new cell is a ⚠️ + backlog entry, not a blocker
  for the session; the session fails only if the machinery cannot produce the
  measurements.
- **L5 — Pythia consumption.** §5.6 env + driver + standalone row. Gate: n/n
  on llj and DY samples.
- **L6 — Hygiene riders.** The §7 rider list. Gate: workspace clippy clean;
  soft-skip inventory empty; discrepancy items closed or explicitly re-filed.
- **L7 — Report driver + close-out.** §5.7 collator (needs L2–L5 row JSONs);
  CI wiring; TODO/README close-out. Gate: `pixi run validate` end-to-end
  ≤ 15 min on the dev machine, report complete per manifest.

## 9. Decisions (user, 2026-07-31) — all resolved

1. **Layer names**: scheme A — `hermetic` / `banked` / `oracle` (§2).
2. **Profile rename**: `release-debug` (§2.5).
3. **Bundle hosting**: release-asset bundle as fallback, plus session **L1b**
   investigating a compact in-repo representation
   (pylhe → awkward → field filter → aggressively compressed parquet), which
   may make hosting the event data in-repo viable (§5.2).
4. **Bespoke `DrellYanIntegrand`**: delete. If the general path does not pass
   the gate, the debug/fix goes to the backlog — not fixed in-sprint (§5.4).
5. **llj banked-tier budget**: 3 seeds; the VEGAS first-iteration fix stays in
   the backlog (§5.4).
6. **`pp_to_bb_fixed`**: added (L4 rider). If it does not pass its gate on the
   first try, ⚠️ + backlog (§4.1).

Plus the general directive baked into §8: sessions expose and consolidate;
newly-exposed failures are recorded, filed, and left for a follow-up
backlog-tackling session.

## 10. Close-out (2026-07-31)

Sprint closed on branch `validation-3`, all eight sessions landed.
The user decides the merge to `main`; nothing here is pushed or tagged.

### What each session landed

- **L0** — `validation/manifest.toml` as the per-process source of truth;
  `required-features` became the only mechanism deciding layer membership (five
  internally-`#[cfg]`'d gates moved); `[profile.profiling]` renamed
  `release-debug`; `validate` / `validate-deep` / `generate-references` entry
  points; CI's `banked` job. `cargo test` on a bare clone became complete.
- **L1** — one staged `generate-references` driver over every generator;
  `fetch_common.sh` as the only place that may download; the diagram counts, the
  HELAS grid and the two LHAPDF oracles committed; `vibegraph-refdata-1`
  assembled, pinned and fetchable; the `validation/madgraph` README rewritten as
  an index over the manifest.
- **L1b** — the compact in-repo event representation measured and **rejected**:
  the projection is exact but bottoms out at 27.5 MB against a 5–10 MB target
  (note 26). Its incidental finding — the bundle double-compresses — is what L7
  acted on.
- **L2** — the `amplitudes` category on MadGraph's own banked events, projected
  exactly on shell, plus the fixed grid: one committed table per process, one
  hermetic `amplitude_oracle` binary over all 19 rows in ~1.1 s, the three
  point-level oracles folded into it.
- **L3** — every hadronic σ through the general `ProtonIntegrand`; the bespoke
  `DrellYanIntegrand` deleted; `p p > e+ e-` re-gated on both dy13 cards through
  the general path; the artifact's per-channel subsampler summary (fv5); the
  first row files written for the collator.
- **L4** — the `samples` category: weighted-ECDF KS on the kinematic
  observables and χ² homogeneity on `SPINUP`, `ICOLUP` and the flavour
  assignment, three generation seeds per row at a 1e-4 p-floor. Two cells came
  out informational and the `low-mll-reconciliation` premise was falsified (see
  the register below). Rider: the `pp_to_bb_fixed` banked run.
- **L5** — Pythia 8.312 reads both emitted samples back, 2000/2000 each, with a
  colour-mutation negative control Pythia rejects.
- **L6** — the hygiene riders: workspace clippy clean, the coloured 2→3
  amplitudes inside the library sweeps, the flavour-group probe ladder widened,
  and the banked layer's 15-entry tolerated-skip table deleted as dead — a
  missing banked input now fails naming itself.
- **L7** — this section, the collator, and the `refdata-2` re-cut.

### The report

`pixi run validate` = fetch → clear the previous run's cells → every banked gate
→ the collator, which renders `target/validation-report/report.{md,json}` and
exits nonzero on a cell the manifest declares and nothing measured, a cell
nothing declared and something measured, a gate cell that failed, or a
measurement that disagrees with its declaration about mode or factorization.
Three rules decide what a cell says, all three stated in the binary's own docs:

1. **Nothing is inferred.** A hermetic cell could have been marked from the
   manifest tier plus "the hermetic suite passed"; that prints a green cell no
   measurement stands behind and keeps printing it after the gate stops covering
   the row. So `amplitude_oracle` and `validate_madgraph_diagrams` write row
   files like every other gate, and a declared-and-unmeasured cell fails.
2. **The worst measurement is the cell.** `pp_to_ll` arrives as two run cards;
   the cell reports the worse pull and lists both, so a cell cannot be made green
   by adding an easier variant.
3. **A standalone gate with its own driver is read from its file.** Pythia's
   verdict is rendered when present and reads "not run in this invocation" when
   absent, which is not a failure — and when its file predates this run's cells,
   the row says so rather than passing an old number off as today's.

Final table (`pixi run validate`, 5m32 on the dev machine, exit 0; 26 rows ×
4 categories = 104 cells, 72 measured: 68 ✅, 4 ⚠️, 4 ⏳ long, 18 ⛔ blocked,
10 covered-by/uncovered):

| process | 2→N | diagrams | amplitudes | integrals | samples |
|---|---|---|---|---|---|
| `ee_to_mumu` `e+ e- > mu+ mu-` | 2 | ✅ 2/2 | ✅ max rel 1.11e-14 | ✅ pull -0.55, chi2/dof 0.60 | ✅ KS p 0.033, chi2 p 0.049 |
| `ee_to_ee` `e+ e- > e+ e-` | 2 | ✅ 4/4 | ✅ max rel 2.66e-14 | ✅ pull -0.73, chi2/dof 1.55 | ✅ KS p 0.004, chi2 p 0.121 |
| `ee_to_ttx` `e+ e- > t t~` | 2 | ✅ 2/2 | ✅ max rel 4.97e-15 | ✅ pull +0.89, chi2/dof 1.69 | ✅ KS p 0.082, chi2 p 0.157 |
| `ee_to_wpwm` `e+ e- > w+ w-` | 2 | ✅ 3/3 | ✅ max rel 4.24e-14 | ✅ pull +1.81, chi2/dof 0.96 | ✅ KS p 0.052, chi2 p 0.202 |
| `ee_to_zh` `e+ e- > z h` | 2 | ✅ 1/1 | ✅ max rel 9.55e-14 | ✅ pull -1.14, chi2/dof 0.50 | ✅ KS p 0.031, chi2 p 0.117 |
| `uux_to_mumu` `u u~ > mu+ mu-` | 2 | ✅ 2/2 | ✅ max rel 1.66e-14 | ✅ pull +0.34, chi2/dof 0.59 | ✅ KS p 0.127, chi2 p 0.073 |
| `uux_to_uux` `u u~ > u u~` | 2 | ✅ 2/2 | ✅ max rel 5.61e-14 | ✅ pull -1.94, chi2/dof 1.75 | ⚠️ KS p 0.007, chi2 p <1e-300 [1] |
| `gg_to_ttx` `g g > t t~` | 2 | ✅ 3/3 | ✅ max rel 1.89e-15 | ✅ pull +0.35, chi2/dof 0.80 | ✅ KS p 0.220, chi2 p 0.169 |
| `gg_to_gg` `g g > g g` | 2 | ⚠️ 4/6 [2] | ✅ max rel 8.25e-14 | ✅ pull -0.53, chi2/dof 1.16 | ✅ KS p 0.014, chi2 p 0.003 |
| `ee_to_mumua` `e+ e- > mu+ mu- a` | 3 | ✅ 8/8 | ✅ max rel 3.94e-14 | ✅ pull +0.31, chi2/dof 0.97 | ✅ KS p 0.002, chi2 p 0.064 |
| `ee_to_tatah` `e+ e- > ta+ ta- h` | 3 | ✅ 5/5 | ✅ max rel 4.04e-14 | ✅ pull +0.44, chi2/dof 1.10 | ✅ KS p 0.002, chi2 p 0.499 |
| `uux_to_epemg` `u u~ > e+ e- g QCD=2 QED=2` | 3 | ✅ 4/4 | ✅ max rel 5.84e-14 | ⛔ `kt-clustering` [3] | ⛔ `kt-clustering` [4] |
| `ddx_to_epemg` `d d~ > e+ e- g QCD=2 QED=2` | 3 | ✅ 4/4 | ✅ max rel 3.98e-14 | ⛔ `kt-clustering` [3] | ⛔ `kt-clustering` [4] |
| `gu_to_epemu` `g u > e+ e- u QCD=2 QED=2` | 3 | ✅ 4/4 | ✅ max rel 3.75e-14 | ⛔ `kt-clustering` [3] | ⛔ `kt-clustering` [4] |
| `gux_to_epemux` `g u~ > e+ e- u~ QCD=2 QED=2` | 3 | ✅ 4/4 | ✅ max rel 3.91e-14 | ⛔ `kt-clustering` [3] | ⛔ `kt-clustering` [4] |
| `ee_to_mumu_tata_qcd0` `e+ e- > mu+ mu- ta+ ta- QCD=0` | 4 | ✅ 25/25 | ✅ max rel 4.20e-12 | ⚠️ pull +7.65, chi2/dof 0.99 [5] | ⚠️ KS p 3.6e-6, chi2 p <1e-300 [6] |
| `uux_to_ccx_emmm_qcd0` `u u~ > c c~ e+ e- mu+ mu- QCD=0` | 6 | ✅ 579/579 | ✅ max rel 2.05e-13 | ⏳ oracle layer [7] | ⏳ oracle layer |
| `bbx_to_ccx_emmm_qcd0` `b b~ > c c~ e+ e- mu+ mu- QCD=0` | 6 | ✅ 615/615 | ✅ max rel 5.89e-14 | ⏳ oracle layer | ⏳ oracle layer |
| **multi-channel** | | | | | |
| `pp_to_ll` `p p > l+ l-` | 2 | ✅ 2/2 | — covered by `uux_to_mumu` | ✅ pull -1.35, chi2/dof 1.19 (worst of 2: mmll_60_120 pull -1.35; default pull +0.25) | uncovered [8] |
| `pp_to_ll_qcd0` `p p > l+ l- QCD=0` | 2 | ✅ 2/2 | ✅ max rel 1.83e-14 | — covered by `pp_to_ll` | uncovered [9] |
| `pp_to_bb` `p p > b b~` | 2 | ✅ 4/4 | uncovered [10] | ⛔ `kt-clustering` [11] | ⛔ `kt-clustering` |
| `pp_to_bb_qcd2` `p p > b b~ QCD=2` | 2 | ✅ 6/6 | uncovered [12] | ⛔ `kt-clustering` | ⛔ `kt-clustering` |
| `pp_to_bb_fixed` `p p > b b~ QCD=2` | 2 | ✅ 6/6 | uncovered [13] | ⛔ `hadronic-shat-floor` [14] | ⛔ `hadronic-shat-floor` [15] |
| `pp_to_llj_fixed` `p p > l+ l- j QCD=2 QED=2` | 3 | ✅ 8/8 | — covered by `uux_to_epemg`, `ddx_to_epemg`, `gu_to_epemu`, `gux_to_epemux` | ✅ pull +0.11, chi2/dof 1.48 | ✅ KS p 0.044, chi2 p 0.144 |
| `pp_to_llj` `p p > l+ l- j` | 3 | ✅ 8/8 | uncovered [16] | ⛔ `kt-clustering` [17] | ⛔ `kt-clustering` |
| `pp_to_llj_qcd2_qed2` `p p > l+ l- j QCD=2 QED=2` | 3 | ✅ 8/8 | — covered by `uux_to_epemg`, `ddx_to_epemg`, `gu_to_epemu`, `gux_to_epemux` | ⛔ `kt-clustering` | ⛔ `kt-clustering` |

### Findings register

What the sprint exposed and deliberately did not fix, each with the gate that
measures it. All of these are in `TODO.md` with their evidence.

1. **The `h → τ⁺τ⁻` pole** (`higgs-pole-in-m-tautau`, replaces
   `low-mll-reconciliation`, whose premise L4 falsified). `ee_to_mumu_tata_qcd0`
   sits +2.2% above banked MadGraph, and binning dσ/dm_ll against MadGraph's own
   events down to threshold puts 159% of the offset in **one 200 MeV bin at the
   Higgs pole** — 7.137e-5 pb against 2.260e-5 pb, factor 3.16 at 22σ around a
   6.4 MeV resonance — with every bin below 20 GeV agreeing. The ratio is within
   errors of π, which a Breit–Wigner normalisation is the first place to look
   for. Decisive next step: a MadGraph run of this process with an m(τ⁺τ⁻) window
   around 125 GeV. Both its `integrals` and `samples` cells are ⚠️.
2. **`uux_to_uux` realised colour flows.** Every kinematic observable and the
   helicity frequencies agree; the `ICOLUP` frequencies do not (99.96% against
   90.4%, χ² 1015 on one degree of freedom, stable across seeds). Our 90/10 is
   exactly the banked JAMP² ratio, so the two sides are applying different
   colour-selection rules — the candidate being MadEvent's per-channel `ICOLAMP`
   conditioning. Neither is wrong as an LO assignment; they differ at order 1/N²
   and the shower is handed the difference.
3. **`hadronic-shat-floor`.** The general hadronic path cannot integrate a final
   state with no leptons: every ŝ lower bound the cut layer derives is a lepton
   bound, so `p p > b b~` leaves `shat_min = 0` and the first parton-density call
   asks for x = NaN. The `pp_to_bb_fixed` run is banked and its `diagrams` cell
   gates; its `integrals` and `samples` cells are ⛔ on this. The missing bound is
   the one the lepton branch already makes.
4. **The mirror term's visibility below the electroweak scale.** The control that
   makes the mirrored-beam identity check meaningful — dropping the mirror must
   move |M|² by more than 1e-3 — holds above 220 GeV and fails at √ŝ = 25 GeV,
   where the weakest mirror term is worth 8.4e-4. The identity itself holds to
   5.4e-13 everywhere measured. Wanted: the bound as a function of ŝ, not a wider
   ladder and a smaller number.
5. **`g g > g g` diagram counting, decided.** MadGraph writes the four-gluon
   contact term as one graph per colour structure and we write one diagram whose
   vertex carries all three, so 3 + 3 against 3 + 1. **Decision taken here, as
   the plan asked: report our count in our own convention and mark the cell ⚠️.**
   Re-splitting the enumeration to match a counting convention would change the
   thing being validated to make a number match, and the same process is pinned
   at 8.25e-14 per flow — far below what any difference in diagram content could
   survive.

### Bookkeeping the sweep turned up

- The `diagrams` cells declared tier `hermetic` while the only gate measuring
  them is registered `banked` (the 2→6 enumeration cost). The tiers now say
  `banked`, and the manifest says that a tier is where a gate is *registered*,
  not the weakest layer it could run in. The gate reads committed references
  only and takes 1.5 s, so moving the binary instead is a live option for a
  later session.
- `validate_sigma` writes an `ee_to_mumu_tata_qcd0` row note that L4's binning
  falsified ("localised at low m_ll"). The collator prefers the manifest's
  curated note over a measurement's own, so the report does not repeat it, but
  the string is still written into the row file.
- The `init-sm-submodule` pixi task runs `git submodule update --init`
  unconditionally and fails outside a git checkout, where `vg_ensure_submodule`
  guards on the file first. It bites only a `git archive` export, not CI.
- `Process`'s `Display` drops coupling-order constraints, so the report's
  measurement detail shows `p p > b b~` where the row is `p p > b b~ QCD=2`. The
  enumeration honours the constraint (4 diagrams against 6); only the printed
  form loses it.

### `refdata-2`

One re-cut, three reasons, because each one costs a new archive and a new pin.

- The `ee_to_mumu_tata_qcd0` subprocess's `matrix1_orig.f` carried a
  hand-added `COMMON/DBG_AMP/` block. Regenerating the process with
  `mg5_aMC` reproduces the file **except for the order MadGraph emits its `FK_*`
  declarations in**, which moves run to run — so the block was excised instead,
  and the result is byte-identical to the fresh generation but for that
  permutation. The `Events/` tree was not touched.
- `pp_to_bb_fixed` joined the bundle (`bundled = false` gone), so a fetching
  checkout has the one purely QCD-initiated multi-channel row.
- Event files travel decompressed, taking L1b's finding: **65 066 838 bytes
  against 90 597 923**, 28% smaller while carrying one more run. The unpack step
  gzips them back, so no gate changed. The byte-for-byte round-trip gate keeps
  its meaning — it compares Les Houches *text*, gzip is lossless, and the archive
  now holds exactly the bytes it asserts on rather than a container around them.
  A side effect worth having: a work area unpacked from a bundle now re-assembles
  to that same bundle, which a gzipped-member archive could not promise.

Verified: two assemblies byte-identical
(`4495d6df…f40e736c`); all 26 runs' decompressed event text unchanged sha256 for
sha256 through pack and unpack; a clean `git archive` export fed only by the
local bundle runs the whole banked layer green and renders a report identical to
the dev machine's, Pythia row apart.

### What the follow-up backlog session should take first

In order:

1. **The Higgs-pole bin.** It is the only ⚠️ that is a candidate *defect* rather
   than a convention difference, the decisive measurement is one banked MadGraph
   run with an m(τ⁺τ⁻) window, and the suspected cause (a Breit–Wigner
   normalisation) would move every resonant channel.
2. **`hadronic-shat-floor`.** Smallest, most contained, and it turns two ⛔ cells
   into measured ones on a row whose reference is already banked.
3. **`uux_to_uux` colour selection.** Read MadEvent's `ICOLAMP` conditioning and
   decide which rule we mean to implement; it is the only place the sprint found
   where what we hand a shower differs from what MadGraph hands one.
4. **The Drell-Yan `samples` gap.** One oracle-layer run banking events for the
   dy13 cards fills the last two `uncovered` cells that a run could fill.

Everything else in the register is either a feature (kT clustering, the
multi-rung spine) or a bookkeeping item above.
