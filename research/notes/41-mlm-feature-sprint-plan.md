# 41 — `mlm` feature sprint plan: MadEvent parity for MLM-matched LO generation

**Status: IN PROGRESS (2026-09-28).** The §5 decisions are settled (user,
2026-09-28: every recommendation accepted). M0 and M1 dispatched.

This is the feature sprint that follows `process-grammar` (note 38). The goal:
a run card with `ickkw = 1` and `xqcut > 0`, over a proc card whose
`add process` lines differ in jet multiplicity, produces the same cross
sections and the same event record that MadEvent does. A Pythia 8 kT-MLM run
(`JetMatching:merge = on`) should then treat our file the same way it treats
MadEvent's.

**Division of labour.** MLM is split between two programs:
- **The matrix-element generator** does:
  - the `xqcut` cut;
  - the clustering scales;
  - the α_s and PDF reweighting;
  - the event record.
- **The shower** does jet matching and the veto. That is where the
  Sudakov suppression comes from.

This sprint builds the first half only. Nothing here showers.

The standing rules are unchanged:
- new physics lands informational and is enforced only once agreement is shown;
- every convention claim is pinned by a test that would fail if it were false;
- per-event fields are compared before cross sections;
- samplers are gated over seed sweeps.

## 1. MadGraph semantics (pinned tree `b7687064`, MadGraph 3.7.1)

Paths are relative to `research/refs/mg5amcnlo`. `SubProcesses/` means
`Template/LO/SubProcesses/`. This section is the specification, read from the
source. Where MadGraph is inconsistent with itself, the item says so.

### 1.1 Call flow per point (scalar path, `vector_size = 1`, the default)

1. **`setclscales(p, keepq2bck = .false.)`** is reached from
   `update_scale_coupling_vec` (`reweight.f:1856,1907`).
   - It clusters, applies the `xqcut` cut and sets μR/μF.
   - A failure sets the weight to 0.
   - `banner.py:4966` forces `dynamical_scale_choice = -1` whenever MadGraph
     auto-enables matching.
2. **`DSIG<n>`** (`auto_dsig_v4.inc:124-182`):
   - evaluates the PDFs at `sqrt(q2fact)`;
   - computes `rwgt = REWGT(p)`, then `SMATRIX`;
   - multiplies `DSIGUU *= rwgt`, then calls `UNWGT`.
3. **`rewgt`** calls `setclscales(p, keepq2bck = .true.)` a **second time**
   (`reweight.f:1465`), then multiplies in the α_s and PDF ratios
   (`reweight.f:1557-1795`).
4. **Colour and mothers use the clustered graph.** When `ickkw > 0`,
   `select_color` draws from `igraphs(1)` instead of the integration channel
   (`super_auto_dsig_group_v4.inc:1120-1142`), and `addmothers` takes its
   configuration from the same graph (`addmothers.f:109-116`).

### 1.2 What changes in `setclscales` under matching (`reweight.f:555-1284`)

vibegraph's port (`coupling/cluster/setclscales.rs`) already covers the
clustering walk, `iqjets`, `jfirst`/`jlast`/`jcentral`, the `xqcut` rejection
(`reweight.f:1063-1089` → `ScaleRefusal::JetCut`) and the `pdfwgt` μF branch.
What it does not model:

- **The early return is on `ickkw`.** With `ickkw = 0`, `xqcut` is a pure cut
  and the routine returns after it (`:1103-1106`). This is already ported.
- **The second call** (`keepq2bck = .true.`) differs from the first in three ways:
  - `q2fact` is non-zero on entry, so `pt2ijcl(jcentral_j) = q2fact(j)`
    (`:1114-1119`) overwrites the central clustering scale with the first
    call's matrix-element PDF scale;
  - `scalefact` is applied again (`:1138-1147`);
  - `q2bck` keeps the first call's value.
- **Two factorisation scales per event:**
  - the *central* one, `q2bck = sf²·sqrt(pt2(jlast)·pt2(jcentral))`;
  - the *matrix-element PDF* one, `sf²·min(pt2(jfirst), central)` when
    `pdfwgt` (`:1195-1203`).

  The PDFs in `DSIG` read the second; SCALUP reads the first.
- **`ptclus`** (`:1228-1269`), per final-state leg:
  - the largest clustering kT among the jet vertices the leg takes part in;
  - otherwise the **collider** `sqrt(stot)`, not `√ŝ`.

  It feeds the `<scales>` tag (§1.4).
- **The per-channel jet memo** (`njetstore`, `:662-679`, `:985-1030`):
  - the first event of each channel fixes the channel's jet count;
  - a later event that disagrees is re-clustered restricted to the channel;
  - `stop 4` if that still fails.

  The port starts the memo empty on every event (`coupling/scales.rs`: "the jet
  memo starts empty on every event"). That reproduces MadEvent's first event
  per channel, not its later ones. Under matching the memo decides which events
  are re-clustered, so how often that branch fires is a census question (§4 M0).

### 1.3 `rewgt` under `ickkw = 1` (`reweight.f:1333-1824`)

`rewgt = 1` immediately if `ickkw ≤ 0` and not `use_syst` (`:1421`). Otherwise
it computes the product of the two factors below.

**α_s factor** (`:1557-1616`). Over clusterings `n < nexternal − 2` (the 2 → 2
core vertex is never reweighted):
- A vertex qualifies if either:
  - **ISR:** the mother is on a beam line and `goodjet(mother)` holds, where
    `goodjet(mother) = isparton(mother) ∧ goodjet(d1) ∧ goodjet(d2)`; or
  - **FSR:** the mother is final-state, `ispartonvx` holds, and at least one
    daughter is `goodjet`.

  In both cases the mother PDG must not be `fake_id`.
- On a qualifying vertex, with `q2now = pt2ijcl(n)`:
  - `q2now ≤ 4 GeV²` kills the event;
  - otherwise `rewgt *= α_s(alpsfact·sqrt(q2now)) / α_s(μR)` (`:1597-1601`).
- `isparton` means |id| ≤ max(`asrwgtflavor`, `maxjetflavor`), or a gluon.

**PDF-ratio factor** (`ickkw > 0 ∧ pdfwgt`, where `pdfwgt` defaults to on;
`:1617-1748`). Walk each beam line up its initial-state clusterings:
- `x ← x·z_n` **before** the evaluation.
- `q2now = min(pt2ijcl(n), q2bck(j))`, and exactly `q2bck(j)` at `jlast`.
- On the first initial-state vertex, no ratio: the matrix-element PDF was
  already evaluated there.
- On each later vertex with `n ≤ jlast` and a rising scale, multiply by
  `f_fl(x, q_now) / f_fl(x, q_prev)`, where `fl` is the flavour of the line
  *entering* the vertex.
- A denominator below 1e-10 kills the event.

Afterwards `q2fact = q2bck` (`:1789-1791`).

**Things that are not specifications of this weight:**
- `s_xpdf` stores x *before* z (`:1719-1723`).
- `systematics.py:1022-1030` telescopes differently.

**`hmult` and `highestmult` do nothing under `ickkw = 1`.** MadEvent treats
every multiplicity identically at parton level; only the shower's `nJetMax`
knows which multiplicity is highest. `ickkw = 2` is unreachable from the card
(`banner.py:4284`, `allowed = [0, 1]`).

### 1.4 Cuts, setup and the event record

- **Setup cuts** (`setcuts.f:156-189`, when `xqcut > 0`):
  - if `auto_ptj_mjj` (default T) and `ptj ≥ 0` and `ktscheme = 1`: **`ptj = xqcut`**;
  - otherwise `ptj > xqcut` → 0;
  - likewise **`mmjj = xqcut`** (otherwise `mmjj > xqcut` → 0);
  - `drjj = drjl = 0`.

  Particles with `do_cuts = .false.` are exempt: decay products under
  `cut_decays = F`, m > 20 GeV, neutrinos (`setcuts.f:201-215`). These change σ.
- **Phase-space hints** (`setxqcuts`, `myamp.f:340-551`): jet energy floors,
  s-channel minima, and a τ minimum built from the floors. These are grid
  hints, but the τ minimum acts as a hard cut; §4 M1 audits whether it is
  implied by the cuts above. There is a duplicate `iforest(2)` test at
  `setcuts.f:939-942`, which only affects the grids.
- **Run-card rules** (`banner.py:4543-4577`):
  - `xqcut > 0` with `ickkw = 0` is an error log and a 5 s sleep, then it runs
    as a pure cut;
  - `maxjetflavor = 6` with matching is an error;
  - `use_syst` forces `alpsfact = 1`;
  - `create_default_for_process` auto-enables `ickkw = 1`, `xqcut = 30` on
    mixed-multiplicity jet cards (`:4924-4966`).
- **Record** (`unwgt.f:558-867`, `addmothers.f`):
  - SCALUP is `sqrt(max(q2bck))` on the scalar path. The vector path restores
    the lowered matrix-element PDF scale instead (`auto_dsig_v4.inc:401`), so
    references must pin `vector_size = 1`.
  - AQCDUP is taken at μR.
  - `<scales pt_clust_N="…">` holds `ptclus`, keyed by the LHE line index
    (`addmothers.f:411-429`), written after the particles.
  - Status-2 resonances appear only if the clustering found them on their
    Breit–Wigner (`addmothers.f:253-268`).
  - IDPRUP is the `P<n>` group number; `<init>` has one line per group.
  - The header carries `<MGRunCard>` (`banner.py:69-86`).
  - `<clustering>` and `<mgrwt>` are for CKKW-L and systematics; they are out
    of scope.
- **Shower side**, for reference only (`madevent_interface.py:4396-4471`):
  - MadGraph drives Pythia with `JetMatching:setMad = off` and
    `qCut = 1.5·xqcut`;
  - `nJetMax = max_n_matched_jets` (`export_v4.py:5010-5024`);
  - `nQmatch = maxjetflavor`;
  - `Beams:setProductionScalesFromLHEF = on`, which is what reads
    `<scales>`. Partons whose scale is ≈ √s are excluded from matching
    (`Template/NLO/MCatNLO/Scripts/JetMatching.h:1473-1487`, an in-repo copy
    that may differ from the Pythia release).

### 1.5 MadGraph defects met on the way

| Where | Defect | Changes a weight? |
|---|---|---|
| `reweight.f:1138` | `.not.fixed_fac_scale1.or.fixed_fac_scale2` precedence | yes, only with one fixed μF |
| `setcuts.f:939-942` | duplicate `iforest(2)` test | no (grids only) |
| `cuts.f:565` | `ktdurham` `.and.`/`.or.` precedence | out of scope (CKKW-L) |
| `addmothers.f:115` | compares `igraphs(1)` to a stale loop index | to be measured (record only) |
| `banner.py:1706` | `setWeightName` raises when `ickkw ≠ 0` | no (Python, systematics) |
| `rewgt` | reads the final-state `ipdgcl` left by the previous event | only if jet-ness differs between flavour combinations of one `IPROC` |

**The policy for defects.** A defect that changes a weight on a card we
support is either reproduced bug-for-bug with a comment naming it, or
refused. It is never silently "fixed". The report goes in note 07.

## 2. Today's surface

vibegraph has most of the pieces, switched off in three places.

| Piece | State |
|---|---|
| kT clustering + `setclscales` | Ported and gated event by event against an instrumented MadEvent build (`validate_kt_cluster`, `gen_kt_cluster_dumps.sh`). `ScaleSettings` carries `ickkw`, `xqcut`, `pdfwgt`; `cluster_scales` hard-codes `ickkw: 0, xqcut: 0.0` (`coupling/scales.rs:416`). |
| Per-group, per-ordering scales | Landed (note 40). MLM reweighting depends on it, because every term must be reweighted in its own clustering. |
| Refusal 1: run card | `ScaleChoice::from_run_card` → `ScaleError::UnsupportedMatching` (`coupling/scales.rs:258-261`). `highestmult`, `alpsfact`, `asrwgtflavor`, `clusinfo`, `auto_ptj_mjj` are `IgnoredBenign(B_MLM)` (`runcard/classes.rs`), and `ktdurham`/`ptlund`/`dparameter` are unimplemented-cut errors (`cuts.rs:392`). |
| Refusal 2: proc card | `Unsupported::MixedMultiplicity` in `check_supported` (`diagrams/check.rs:281`). |
| Refusal 3: integrand | `ProtonIntegrand` reads `n_out` from `groups[0]` (`proton.rs:1449,1502`). `ChannelIntegrand` has a single `channel_grid_ndim()` (`unweight.rs:219`), with 21 call sites. |
| α_s and PDFs at arbitrary scales | `coupling::alphas` (MadGraph's RGE) and `pdf::xfx_q2` exist. |
| `@N` → `LPRUP`, one `<init>` line per process | Landed (note 38 E1). |
| LHEF header | A provenance comment only. No `<MGRunCard>`, no `<scales>`. |
| Pythia gate | Process-level consumption only (`validation/pythia/consume.py`). No `JetMatching`. |
| Status-2 records for free on-window resonances | Written for decay-chain cards only (TODO, E1 follow-ups). Under matching, MadGraph writes them from the clustering's Breit–Wigner test. |

## 3. Design

### 3.1 Two factorisation scales, one record scale

`EventScales` gains the split that §1.2 describes:
- `mu_f_pdf`, which the density rows read;
- `mu_f_record` (`q2bck`), which `SCALUP` reads.

Both are equal whenever `ickkw = 0`. Keeping them equal there makes every
existing artifact and LHE file byte-identical, and a test pins that (the note 40
§4 pattern).

### 3.2 The reweighting is a per-term factor beside the scale

`rewgt` is a pure function of:
- the clustering history `setclscales` already returns (`pt2`, `jfirst`, `jlast`,
  `jcentral`, `iqjets`, the merges with their `z` and PDG);
- the run-card constants;
- the `α_s` evaluator;
- the PDF.

It goes in `coupling/cluster/rewgt.rs` and returns a `Rewgt` struct with every
factor listed: the qualifying vertices, their α_s ratios, and the PDF-ratio
chain per beam with its (x, flavour, q_prev, q_now). The integrand only uses
the product. The factor list exists so the oracle can compare at the finest
linear level (AGENTS.md).

The factor applies per term: per flavour group and per beam ordering, on the
note-40 per-term path, which matching therefore always takes.

### 3.3 Mixed multiplicity: a composite integrand, not a wider one

MadEvent integrates each `P<n>` directory separately and unweights all channels
together. The equivalent here is a `MultiplicitySum` that owns one
`ProtonIntegrand` per final-state multiplicity and exposes their channels as one
list, offset by multiplicity.

`ChannelIntegrand::channel_grid_ndim` takes the channel index. The default
implementation returns the old constant, so single-multiplicity integrands are
untouched. The VEGAS grids are already per channel, so each grid gets its own
dimension.

The rejected alternative is padding every channel to the largest dimension.
It needs no trait change, but it gives the grids axes nothing depends on and
changes the uniform stream for every channel.

Consequences:
- The artifact schema is bumped.
- The budget is split across multiplicities through the existing per-channel
  allocation.
- The unweighter needs no change: channels are drawn ∝ `w_max_j` whatever
  multiplicity they belong to.

### 3.4 Colour from the clustered graph

Under `ickkw > 0`, the colour-flow draw and the resonance records take the
configuration the *clustering* chose (`igraphs(1)`), not the integration channel
(§1.1 item 4). `select_config_and_flow` gains that configuration as an input.
With `ickkw = 0` it passes the channel, which is today's behaviour, and a test
pins it.

### 3.5 The run card

- `ickkw` admits {0, 1}, matching MadGraph's `allowed`.
- `xqcut > 0` with `ickkw = 0` is accepted as a pure cut, with a warning
  (MadGraph's behaviour).
- `alpsfact`, `asrwgtflavor`, `auto_ptj_mjj` move to `Consumed`.
- `pdfwgt` is already consumed.
- `highestmult` stays `IgnoredBenign`, since MadEvent never reads it at
  `ickkw = 1`.
- `clusinfo` stays benign until `<clustering>` is written.
- `ktdurham`/`ptlund`/`dparameter` stay refused (CKKW-L).
- `Cuts::compile` applies the §1.4 rewrites (`ptj`/`mmjj` = `xqcut`,
  `drjj = drjl = 0`) with `do_cuts`'s exemptions. These are the resolved values
  recorded in the artifact.

## 4. Sessions

Order: M0 → M1 → M2 → M3 → M4 → M5. M6 can run after M3.

### M0: references and the oracle (validation-dev; oracle tier)

- **Cards.** All at 13 TeV, NNPDF as the banked rows, `vector_size = 1`, ≥ 5
  seeds each (≥ 10 for any gate tighter than 0.3%), stored per seed:
  - **`pp_to_llj_mlm`**: `p p > e+ e- j`, `ickkw = 1`, `xqcut = 20`, `ptj`
    auto. A single multiplicity, so M1 and M2 run without M3.
  - **`pp_to_ll_0j2j_mlm`**: `p p > e+ e- @0` + `j @1` + `j j @2`, the same
    card. The canonical MLM sample.
  - **`pp_to_llj_xqcut_only`**: `p p > e+ e- j` with `ickkw = 0`,
    `xqcut = 20`. The pure-cut branch.
  - **`pp_to_llj_mlm_alps2`**: as `pp_to_llj_mlm` with `alpsfact = 2` and
    `use_syst = F`. MadGraph forces `alpsfact = 1` under `use_syst`
    (`banner.py:4551-4555`, `setrun.f:151-159`), so the card must switch
    systematics off or it silently measures `alpsfact = 1`. This is the
    convention pin in M2, and vibegraph must apply the same override: a card
    with `use_syst = T` and `alpsfact ≠ 1` is read as `alpsfact = 1`.
  - **`pp_to_ttx_0j1j_mlm`**: `p p > t t~ @0` + `j @1`. The massive-core
    branch (`mt2last`, the massless–massive `dj` case). Scope decision §5 (d).
- **Extend the instrumented replay** (`gen_kt_cluster_dumps.sh`) so each banked
  event also records:
  - both `setclscales` calls' scales;
  - `q2bck`;
  - every `rewgt` vertex decision with its α_s ratio;
  - the PDF-ratio chain;
  - the final `rewgt`;
  - `ptclus`;
  - whether the `njetstore` re-cluster branch fired.

  The existing precondition holds: the replay's events must equal the bank's.
- **Censuses:**
  - how often the jet-memo branch fires per row (§1.2);
  - whether any `IPROC` mixes jet and non-jet final-state flavours (the stale
    `ipdgcl` defect in §1.5).

  A non-zero count is a finding for M1 to resolve, not a footnote.
- Register each row with `bundled = false`, `status = "planned"`. Wire a
  `generate-references` stage. The runs are banked as `refdata-9` at close
  (§4 Z).

### M1: `xqcut` and the `ickkw = 1` scales (feature-dev; after M0)

- Implement §3.5, lifting `UnsupportedMatching` for `ickkw ∈ {0, 1}`.
- Pass `ickkw`, `xqcut` and `pdfwgt` through `cluster_scales`.
- Implement the second-call semantics (§1.2) and the scale split (§3.1).
- Implement colour from the clustered graph (§3.4).
- Audit the τ minimum (§1.4): if it is not implied by the rewritten cuts, it
  is a cut, and it goes into `Cuts`.
- **Gates**, each against M0's dump:
  - event by event: the `xqcut` decision, both scale calls, `SCALUP`/`AQCDUP`
    on `pp_to_llj_mlm` and `pp_to_llj_xqcut_only`;
  - σ of `pp_to_llj_xqcut_only`, with seeds;
  - `pp_to_llj_mlm`'s σ stays **informational and known-wrong** (no
    reweighting yet), so the moment M2 goes live shows up end to end.
- Every existing artifact and LHE file must stay byte-identical at `ickkw = 0`.

#### M1 Landed (implementation), 2026-09-28

The implementation is in. The per-event gates against M0's dumps come later,
when the dumps exist. What changed, checked against the pinned source:

- **Run card** (`runcard/matching.rs`, applied in `RunCard::from_values`, so the
  artifact records the resolved card):
  - `ickkw ∉ {0, 1}` is refused (`banner.py:4284`, `allowed = [0, 1]`).
  - `ickkw = 1` with `maxjetflavor = 6` is refused (`banner.py:4556`).
  - `use_syst = T` forces `alpsfact = 1`. This is `setrun.f:151-159`, which
    applies it **whether or not matching is on**; `banner.py` applies it only
    under `ickkw > 0`.
  - With `xqcut > 0` the `setcuts.f:156-189` rewrites of `ptj` and `mmjj` apply,
    and `drjj = drjl = 0` whatever their sign (`banner.py:4562-4571` zeroes
    any nonzero value first). `xqcut > 0` with `ickkw = 0` logs a warning and
    runs as a pure cut. `do_cuts`'s exemptions were already in `Cuts::classify`.
  - `ickkw`, `alpsfact`, `asrwgtflavor`, `auto_ptj_mjj` and `use_syst` are
    `Consumed`. `xqcut` has left the unimplemented-cut list.
- **Scale prescription** (`coupling/scales.rs`):
  - `UnsupportedMatching` now covers only `ickkw ∉ {0, 1}`.
  - Decision (c): `ickkw = 1` with exactly one fixed μF is refused, and the
    message names the `reweight.f:1138` precedence.
  - Matching or `xqcut` at fixed beams or decays is refused
    (`FixedBeamMatching`). The fixed-beam integrand has no path to zero-weight
    a point the clustering rejects, and there is no reference for it.
  - With `xqcut > 0` or `ickkw = 1`, every event is clustered, even on a card
    that fixes every scale (`reweight.f:643`). A `JetCut` refusal zero-weights
    the term, as the factorisation floor already did.
  - `cluster_history` makes both `setclscales` calls, sharing one jet memo.
    The second call enters with the first call's `(μR, q2fact)`. In
    `setclscales.rs` that entry state now triggers the `:1114-1119` overwrite
    of the central vertices under `ickkw > 0`. `ClusterScales::q2central` is
    the value `:1141-1144` stores in `q2bck`.
- **Scale split** (`EventScales`):
  - `mu_f` (what the density rows read) is the first call's `q2fact`.
  - `mu_f_record` (what SCALUP reads) is `q2bck` when `pdfwgt` is set. Without
    `pdfwgt` it is the second call's `q2fact`, because `rewgt` restores `q2bck`
    only when `pdfwgt` is set (`:1789-1791`). §3.1 said "`q2bck`"
    unconditionally, which is right only for `pdfwgt = T`.
  - `clustered_config` is `igraphs(1) − 1`.
  - `EventScales::unmatched` makes the record scale equal the density scale and
    the configuration `None`. Every scale without matching goes through it.
- **Colour** (§3.4): `select_config_and_flow` takes `clustered: Option<usize>`.
  The proton selection passes the drawn term's `clustered_config`, which is
  `None` at `ickkw = 0`. A test pins the `None` path to the `AMP2` draw over a
  sweep of variates.
- **τ-minimum audit** (`myamp.f:337-560`, `setxqcuts` at `setcuts.f:892-955`):
  - The limit is `(Σ xe)²/s`. A jet's floor is `max(ptj, √(xqcut² − m²))`, and
    an s-channel pair meeting in a given channel takes a further floor of
    `xqcut` on its energy.
  - With the resolved `ptj = xqcut` (the default: `auto_ptj_mjj = T` and
    `ktscheme = 1`, the only `ktscheme` accepted), the limit is implied by the
    cuts. A leg's energy is at least its pT, and any pair holding a cut jet
    already has at least `xqcut` of energy. So it cuts nothing, and
    `cuts::madevents_xqcut_tau_floor_is_implied_by_the_rewritten_cuts` pins that
    on 200k sampled points.
  - With a resolved `ptj < xqcut` (`auto_ptj_mjj = F`, or `ptj < 0`), the limit
    is a cut that changes σ, and it differs between integration channels.
    Rather than build a channel-dependent cut, that card is refused
    (`XqcutAboveJetThreshold`). It is a scoped refusal, not an implementation.
- **Byte identity at `ickkw = 0`**: pinned by
  `without_matching_the_record_scale_is_the_density_scale` and
  `scalup_reads_the_record_scale`, and measured binary against binary against
  `85e1459`. The six runs were `p p > e+ e- j` on `pp_to_llj_fixed`'s and
  `pp_to_llj`'s banked cards, `p p > j j`, `p p > e+ e-`, and the fixed-beam
  `g u > e+ e- u` and `e+ e- > mu+ mu-`, each with `integrate` (20k × 4, seed
  7) and `generate` (500 events). Every `grid.bin.zst` is byte-identical, and
  every LHE file is identical except for the header line that names the
  artifact path.
- **Known-wrong comparison**: a `pp_to_llj_mlm` σ computed now lacks `rewgt`'s
  α_s and PDF factors and is expected to be wrong. It stays informational until
  M2.
- **Left for the dump-gate follow-up**: event-by-event comparison of the
  `xqcut` decision, both calls' scales, `q2bck`, SCALUP and AQCDUP on
  `pp_to_llj_mlm` and `pp_to_llj_xqcut_only`. Also the seeded σ of
  `pp_to_llj_xqcut_only`, and the jet-memo census (§1.2), which decides
  whether the per-event memo reset holds under matching.

### M2: `rewgt` (feature-dev; after M1)

- Implement `coupling/cluster/rewgt.rs` (§3.2) and wire it into the per-term
  path.
- **Gates:**
  - per event, factor by factor against the dump: vertex qualification, each
    α_s ratio, each PDF ratio, and then the product. The product alone cannot
    tell a missing vertex from a compensating PDF error;
  - σ of `pp_to_llj_mlm` against MadEvent, with seeds;
  - `pp_to_llj_mlm_alps2`, both as a σ and per event. It pins where `alpsfact`
    enters. A test that sets `alpsfact` but compares only `alpsfact = 1`
    events would not see it.
- **Negative control:** removing the α_s factor must fail the σ gate by far
  more than its tolerance.

### M3: mixed multiplicity (feature-dev, or performance-dev for the budget; after M2)

- Lift `MixedMultiplicity`, subject to §5 (a).
- Implement `MultiplicitySum` and the per-channel `ndim` (§3.3), and bump the
  artifact schema.
- Set IDPRUP from `@N` (or the `P<n>` numbering; M0 records which the reference
  uses).
- **Gates** on `pp_to_ll_0j2j_mlm`:
  - σ per `@N` and in total, with seeds;
  - the `samples` fractions per `@N`.
- A budget ladder on the 2-jet channels, read against the measured seed
  spread, because this is the heaviest row the suite has integrated.

### M4: the event record for the shower (feature-dev; after M3)

- Write `<scales pt_clust_N>` with the `ptclus` rule, including the collider
  √s fallback.
- Write status-2 resonances from the clustering's Breit–Wigner test under
  `ickkw > 0`.
- Write `<MGRunCard>` in the header, holding the resolved run card, so
  `JetMatching:setMad = on` works as well as MadGraph's own `setMad = off`
  driving.
- **Gates:** the `samples` category against M0's MadEvent events, per event
  field (scales, resonances, colour), and the existing Pythia consumption gate
  on the new sample.

### M5: matched end-to-end (validation-dev; after M4)

- Run MadEvent's and vibegraph's `pp_to_ll_0j2j_mlm` files through **one**
  Pythia configuration: MadGraph's own settings from §1.4, a fixed Pythia seed
  set, and the pinned Pythia version.
- **Compare:**
  - the matching acceptance per `@N`;
  - the merged σ;
  - differential jet rates `log10(d_01)`, `d_12`, `d_23`.
- This is statistical on both sides, so it gates on seed sweeps.
- It starts informational. It is the only check that sees the shower's reading
  of `<scales>`, so it stays in the report even while informational.

### M6: xqcut-aware phase space (performance-dev; after M3)

- Put the jet energy floors and s-channel minima (§1.4) into the multichannel
  maps and `Cuts::spacelike_floor()`.
- Measure unweighting efficiency and CPU-to-target against the note-30 baseline
  on `pp_to_ll_0j2j_mlm`.
- σ must not move (seed-sweep gate).

### Z: close-out

As note 38 §7:
- in this container, run the whole banked layer and register every cell;
- on the bank host, bank `refdata-9`;
- from the published bundle, flip each cell that agrees to `gate`;
- the TODO and note 41 close-out record.

## 5. Decisions (settled 2026-09-28, user: every recommendation accepted)

Each item records the question and the decision; "Recommendation" below is
what was adopted.

- **(a) Mixed multiplicity without matching.** MadGraph accepts
  `ickkw = 0, xqcut = 0` over mixed multiplicities and double-counts.
  - Recommendation: accept it for parity, with a warning like MadGraph's.
  - The alternative is refusing it unless `ickkw = 1`.
- **(b) CKKW-L** (`ktdurham`, `ptlund`, `<clustering>`). It is a different
  merging scheme that shares the clustering.
  - Recommendation: keep it refused, with its own backlog entry.
- **(c) Bug-for-bug on `reweight.f:1138`.** It only bites with exactly one
  fixed factorisation scale.
  - Recommendation: refuse that card combination under `ickkw = 1` rather than
    reproduce the defect.
- **(d) The `t t~ + j` row.** It is the only coverage of the massive-core
  branches, and it roughly doubles M0's MadEvent time.
  - Recommendation: include it.
- **(e) Whether M5 must pass before the sprint closes.**
  - Recommendation: close on M4. M5 stays in the report as an informational row
    with its own validation-backlog entry.

## 6. Risks

- **History dependence.** MadEvent's per-channel jet memo makes an event's
  scale depend on which event of its channel came first. If M0's census finds
  the re-cluster branch firing, per-event parity needs either the memo's state
  from the dump or a documented, measured σ-level tolerance.
- **Weight spread.** The α_s and PDF factors widen the weight distribution. The
  unweighting efficiency will drop, and truncation (`MaxRule`) needs re-reading
  on the matched rows.
- **Cost.** `p p > e+ e- j j` at `xqcut = 20` is heavier than anything the
  banked layer runs today. M3's ladder decides whether its σ cell is `banked` or
  `long`.
- **Pythia version.** `JetMatching`'s treatment of production scales has moved
  between releases, and the in-repo copy is from the NLO template. M5 pins the
  Pythia version in the pixi `pythia` environment and records it in the row.
- **Scalar vs vector SCALUP.** A reference produced with `vector_size > 1`
  records a different SCALUP (§1.4). M0's generator asserts `vector_size = 1`.
