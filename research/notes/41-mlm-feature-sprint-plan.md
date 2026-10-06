---
type: Sprint Plan
title: "`mlm` feature sprint plan: MadEvent parity for MLM-matched LO generation"
description: "Plan, session records and close-out of the mlm sprint: xqcut, ickkw = 1 clustering scales, α_s/PDF reweighting, mixed multiplicity and the shower-ready event record."
note: "41"
created: 2026-09-28
status: stable
tags: [mlm-matching, merging, kt-clustering, lhef, sprint]
generated: {by: claude-code, at: 2026-09-28}
---
# 41 — `mlm` feature sprint plan: MadEvent parity for MLM-matched LO generation

**Status: Z1 DONE, WAITING ON `refdata-9` (2026-10-01).** Every session of
§4 has landed and Z1 closed the sprint in its container ("Z1 close-out
record", at the end of §4). B1 banks `refdata-9` on the bank host and Z2
flips the cells that agree (§4 "Z: close-out"). The §5 decisions were settled
on 2026-09-28 (user: every recommendation accepted).

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

#### M0 Landed (2026-09-28)

**What ran.** `validation/madgraph/gen_mlm_references.sh` (pixi task
`generate-mlm-references`, the `mlm` stage of `generate-references`) makes
every run of the five rows on the pinned 3.7.1 tree, ten seeds per row
(20260928–20260937), each the run card's full 10000 unweighted events, two
cores. The first seed is the samples-grade run under
`output/<row>/Events/run_01` (the bundle picks it up), always the first run of
a freshly generated directory so the replay can reproduce it; seeds 2–10 run
in `work/mlm/<row>`. Proc scripts are `scripts/<row>.mg5` (carrying
`# built-by:`, which `build.sh` now honours by skipping them), run cards are
`<row>_run_card.dat`, checked against each script's launch block before any
run; every banner is checked for `vector_size = 1`.

Card choices beyond the brief:
- `systematics_program = none` with `use_syst = T` kept (except `_alps2`):
  events still carry `<mgrwt>`, which the extractor replays entry by entry,
  and the post-run reweighting pass (no effect on σ or on the parton-level
  events) is skipped.
- `pp_to_ttx_0j1j_mlm` uses **xqcut = 30** and no `mmll`: 30 is MadGraph's own
  default for a mixed-multiplicity jet card (`banner.py:4958`) and the
  conventional top-pair value, keeps the one-jet sample from dominating, and
  pins the `ptj = mmjj = xqcut` rewrite at a second value.
- The two mixed cards spell `pdlabel` / `fixed_fac_scale` as MadGraph's own
  mixed-multiplicity template does; the llj cards spell `pdlabel1/2` and
  `fixed_fac_scale1/2` as theirs does.

**Per-seed σ** (`mlm_sigma_reference.json`; inverse-variance mean ±
max(quoted, spread/√n), χ²/dof of the seeds about it; wall time per
`generate_events` call):

| row | σ (pb) | quoted | spread/√10 | χ²/dof | wall (s) |
|---|---|---|---|---|---|
| `pp_to_llj_mlm` | 268.06 | 0.281 | 0.220 | 0.68 | 28–40 |
| `pp_to_llj_xqcut_only` | 212.55 | 0.229 | 0.150 | 0.41 | 55–153 |
| `pp_to_llj_mlm_alps2` | 240.71 | 0.252 | 0.145 | 0.32 | 25–35 |
| `pp_to_ll_0j2j_mlm` | 1064.33 | 0.792 | 0.622 | 0.62 | 101–125 |
| — `@0` | 664.81 | 0.519 | 0.317 | 0.37 | |
| — `@1` | 269.04 | 0.529 | 0.333 | 0.39 | |
| — `@2` | 130.49 | 0.260 | 0.199 | 0.62 | |
| `pp_to_ttx_0j1j_mlm` | 1088.54 | 0.805 | 0.701 | 0.70 | 19–33 |
| — `@0` | 512.90 | 0.169 | 0.181 | 1.03 | |
| — `@1` | 575.72 | 0.784 | 0.725 | 0.77 | |

Readings:
- Every row's seeds scatter *less* than they quote (χ²/dof 0.3–0.8; only
  the top-pair `@0` reads 1.03). Seeds
  2–10 of a row share one directory and each inherits its predecessors' grids,
  which may correlate them; the gate rule's max(quoted, spread/√n) takes the
  quote on every row, so the reference error is the conservative one. A gate
  tighter than 0.1% on these would need independent directories per seed.
- rewgt raises the llj cross section by 26.1% (268.06 against the pure-cut
  212.55), and alpsfact = 2 takes back 27.6 pb of that 55.5 (240.71): the
  alpha_s factor is most of the reweighting and moves by 10% of σ when its
  argument doubles.
- `@1` of the mixed card (269.04 ± 0.53) equals the single-multiplicity
  `pp_to_llj_mlm` (268.06 ± 0.28) at 1.6σ on the same cuts: MadEvent reweights
  every multiplicity alike at `ickkw = 1` (§1.3, `hmult` unread), measured.
- The `<init>` block of the mixed card has one line per `@N` with **LPRUP =
  N** (0, 1, 2), and each event's IDPRUP is its `@N`. The `P<n>` directory
  prefix equals `@N` on these cards too, so they cannot tell the two readings
  of §1.4 apart; a card with `@N` values not in order (`@5`, `@3`) would.
- Wall times: the llj rows run in half a minute to two and a half (the spread
  is load from the sibling session's builds), the mixed Drell-Yan row in under
  two minutes, the top-pair row in half a minute. The instrumented replay of `pp_to_llj_mlm` took 9 min for
  madevent plus 2 min of extraction; `pp_to_ll_0j2j_mlm` 56 min plus 6 (167641
  record sets flushed for 10000 kept events), `pp_to_ttx_0j1j_mlm` 7 min plus 1.

**The extended replay** (`gen_kt_cluster_dumps.sh` / `.py`,
`wrappers/ktdump*`). Each written event's record set now spans both
`setclscales` calls of its point: the first call opens the set (its `SCL`
record has `keepq2bck = F`), rewgt's second call appends (`SCL` with
`keepq2bck = T`), and rewgt's own records follow. New record types —
`Q2OVR`, `Q2BCK`, `PTCL`, `RWLEG`, `RWBEG`, `RWVX`, `RWPDF`, `RWKILL`, `RWEND`,
`CFG`, `CNT`, and outside the record sets `CONST2` and `MEMOX` — are documented
field by field in the module docstring of `gen_kt_cluster_dumps.py`, which is
the schema M1/M2 write against. rewgt runs after the flavour-combination draw
(`IPSEL ∝ |PD(IPSEL)|`, `auto_dsig_v4.inc:141-151`) and reads that
combination's codes, so its factor is per flavour combination: `RWBEG` carries
the drawn `IPSEL` with `IPROC` and `igraphs(1)`, the first `SCL` record
`IMIRROR` and the channel, and `RWLEG` each leg's `idup` for that combination
beside the `ipdgcl` rewgt found (before) and used (after). The dumps are `output/ktdump/dumps/<row>.jsonl.gz`,
pinned in `mlm_dump_manifest.json` (a separate manifest, so
`validate_kt_cluster` does not iterate the matched runs). Precondition kept:
every replay's event file is byte-identical to the banked run's (checked for
all rows run; the banner differs only in the output path and, on
`_xqcut_only`, in the run card's whitespace). The extractor's matched gate,
per event: SCALUP = sqrt(max q2bck) of the second call's exit; rewgt leaves
q2fact = q2bck; `<asrwt>` = the ISR/FSR vertices' sqrt(q2now) in order;
each `<pdfrwt>` = the matrix-element entry then one entry per RATIO step with
x *before* z; `<scales pt_clust_N>` = `PTCL OUT` legs 3..n. On
`pp_to_llj_mlm`, 10000/10000 events pass all of them (`<pdfrwt>` 4938 in
beam order, 5062 with the beams flipped by `write_leshouche`).

Two operational changes the replay needed: shards are streamed through gzip
(`VG_KTDUMP_GZIP=1`: the Fortran writes to a named pipe a detached gzip
drains; a matched 2 → 3 flushes ~60000 record sets per 10000 kept events,
gigabytes raw) and dropped after extraction (`VG_KT_DROP_RAW=1`); and
`madevent_seeds.sh` / `gen_kt_cluster_dumps.sh` link libstdc++ on Linux
(`mes_ldflags`) — the conda activation's LDFLAGS suppresses make_opts'
`STDLIB`, and every `pdlabel = lhapdf` run failed to link here.

For M1: `CONST` and `CONST2` come in more than one variant per directory
(deduplicated on text across every process that wrote one, including ones
that ran with the card's defaults unread, e.g. `ickkw = 0` and fixed scales on
`pp_to_llj_mlm`); take the variant whose `ickkw` and fixed-scale flags match
the run card. `asrwgtflavor` is **5** on these cards (the hidden default;
`isparton` reads max(asrwgtflavor, maxjetflavor) = 5), and `pdfwgt` is **F** on
`_xqcut_only` (T on the ickkw = 1 rows).

**Census 1 — the jet memo's restricted re-cluster fires (finding).**
- On `pp_to_llj_mlm` the restricted re-cluster branch
  (`RESTRICTED_RECLUSTER`) ran in 2246 of the 20000 `setclscales` calls behind
  the written events: 1123 events (11.2%) were scaled from a clustering
  restricted to their integration channel, in both calls. Over every point of
  the run: 252162 first-call and 28178 second-call restricted re-clusters and
  20 stores. `_alps2`: 2502 calls; `_xqcut_only`: 1407 of 10000 (one call per
  point); the banked ickkw = 0 `pp_to_llj` already had 1857 of 10000
  (`kt_cluster_dump_manifest.json`).
- Where it fires is a property of the channel, not of the job, on every llj
  row. The memo is per operating-system process and channel, set by the job's
  first point, so it *could* depend on which job made an event; measured per
  job directory (`njetstore_on_entry_by_channel`, keyed
  `<subprocess dir>/<channel dir>:iconfig`), every job of `P1_gq_llq`
  channels 1 and 2 (G1, G2a0, G2b0) stored **0** jets and every other job
  stored 1, with no directory holding two values. Every re-clustered written
  event comes from those two channels, and every event from them is
  re-clustered. So on these rows the branch is deterministic per event given
  its channel: the stored count is the jet count of the channel-restricted
  clustering, which is 0 on those two `g q > e+ e- q` channels for whichever
  point came first. The port starting the memo empty on every event (§1.2)
  reproduces the first event of a job, not these; M1 needs the channel-restricted
  jet count as the memo. Each event's first `SCL` record carries
  `njetstore(iconfig)` on entry, and each dumped event now names its job
  directory, so a per-event gate can be handed the state and check the rule.
- `pp_to_ll_0j2j_mlm`: 661 of 10000 written events re-clustered (1322 of
  20000 calls; 2071428 first-call and 157100 second-call restricted
  re-clusters at any point, 97 stores). 270 are `P1_gq_llq` events in the two
  channels whose memo holds 0, as on the llj rows; the other 391 are two-jet
  events (`P2_gq_llgq` 287, `P2_qq_llqq` 74, `P2_gg_llqq` 30) in channels whose
  memo holds 1 — or 2, for 5 of them. The `@0` channels store 0 jets, which a
  zero-jet event always matches. Again no job directory holds two values.
- `pp_to_ttx_0j1j_mlm`: the branch never fires (50 stores, no re-cluster);
  every `P0` job stored 0 jets and every `P1` job 1.
- `stop 4` never fired on any row.

**Census 2 — IPROC jet-ness (null).** No IPROC of any row mixes jet and
non-jet flavours on a final-state leg (`mlm_census.json`: 0 of 6 IPROCs on the
llj rows, 36 on `pp_to_ll_0j2j_mlm`, 6 on `pp_to_ttx_0j1j_mlm`), and per event no final-state leg's rewgt PDG differs from
its `idup` (`final_leg_ipdgcl_differs_from_idup`: False on every leg of every
matched event). The stale-`ipdgcl` defect of §1.5 cannot reach these rows.

**What the dump settles about §1 (and where §1 was incomplete).**
- §1.2: the overwrite `pt2ijcl(jcentral) = q2fact` has a guard the text omits,
  `jcentral(2) ≠ jcentral(1)`. On `pp_to_llj_mlm` both beams' jcentral is the
  core vertex (3) on every event, so only one overwrite applies; it moved the
  core scale on 7641 of 10000 events.
- §1.2/§1.3, not in the text: the **second call never recomputes μR** (`scale`
  is non-zero on entry; `MUR` branch `NOT_ENTERED` on all 10000), so rewgt's
  asref = alpha_s(μR of the first call).
- §1.1: in the grouped (default) mode the first call also runs in every
  IPROC's `IMODE = 4` PDF-selection pass before the final `DSIGPROC`, and the
  `IMODE = 5` grid-initialisation pass calls rewgt (so the second call) with no
  first call before it. Both touch the memo; neither reaches a written event's
  record set, which the final `DSIGPROC`'s first call opens.
- §1.3: a rising PDF scale at `n > jlast(j)` takes no ratio *and* leaves the
  mother's `pt2pdf` unset (action `NONE`); not reached on `pp_to_llj_mlm`
  (actions: FIRST 20000, RATIO 8877), `pp_to_ll_0j2j_mlm` (RATIO 4089,
  NOT_RISING 125) or `pp_to_ttx_0j1j_mlm` (RATIO 5227, NOT_RISING 16).
  Vertex classes on `pp_to_llj_mlm`: CORE 10000, ISR 8877, NONE 11123, no FSR,
  FAKE_ID or kill, mean rewgt 1.357 (range 1.000–2.092); the only FSR
  vertices are `pp_to_ll_0j2j_mlm`'s 79. `mt2last` replaced the last two
  scales in 1946 of `pp_to_ttx_0j1j_mlm`'s 20000 calls (4 on the Drell-Yan
  mixed card).
- §1.5 `addmothers.f:115`: `vec_igraph` is never 0 on a written event, so the
  stale-index fallback is unreachable on these rows; `vec_igraph = igraphs(1)`
  of the second call on every event, and differs from the integration channel
  on 525 of 10000 (the configuration colour and mothers come from, §1.1 item 4).
- §1.4: SCALUP = sqrt(max q2bck) verified per event (above); the `<scales>`
  fallback for non-jet legs is the collider √s (13000.00000) as stated.

**What remains.** Nothing of the M0 scope: all five rows ran (ten seeds
each), all five samples-grade runs replayed byte-identically, and their dumps
pass the matched gate on every event. For the close-out: `output/<row>` (five
directories) are the refdata-9 additions; the dumps (`output/ktdump/dumps`,
about 60 MB for the five) stay outside the bundle like the kT dumps, so a gate
reading them is oracle-layer. A tighter σ gate than the quoted ~0.1% per row
would need seeds in independent directories (see the χ²/dof reading above).

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

#### M1 Landed (dump gates), 2026-09-28

**Harness.** `vibegraph-lib/tests/validate_mlm_dumps.rs` (pixi task
`validate-mlm-dumps`, oracle layer, `#[ignore]` since the dumps are outside
the bundle) reads `mlm_dump_manifest.json`. It builds each event's channel
forests from its own process directory's `configs.inc`, `config_nqcd.inc` and
`config_subproc_map.inc` (`confsub`). The dump's tables carry no directory
name; the masses and widths come from the dump's `IFOR` rows. Each event is
read two ways:
- **Engine replay:** each `setclscales` call goes through the routine from the
  state MadEvent entered it with (its own momenta, memo, scales).
- **Production path:** `ScaleChoice::cluster_history` on one momentum set,
  with the memo rule and the card as this crate resolves them.

The fields are compared in order, and each one reports its first divergent
event. `CallRecords` already parses the `rewgt` records, so M2's factor-by-factor
gate can extend the same loop. The run cards resolve against the dumped
`CONST2` (xqcut, alpsfact, asrwgtflavor, maxjetflavor, pdfwgt, the rewritten
ptj/mmjj/drjj/drjl, scalefact). That check found one thing the port had wrong:
`setrun.f:82` clears `pdfwgt` at `ickkw = 0`. Every reader tests `ickkw > 0`
anyway, so nothing downstream changed.

**Agreement** (n/10000 per field; scales at 1e-12, worst 0 or ≤ 8e-15; AQCDUP
worst 3e-16):
- `pp_to_llj_mlm`, `pp_to_llj_mlm_alps2`: 10000/10000 on every field of both
  readings. The fields are the memo steps and branch, the stored count against
  `njetstore` on entry, the vertex scales after the rewrites, the μR/μF
  branches, both calls' μR and q2fact, `q2central` against `Q2BCK CENTRAL`,
  `Q2OVR`, `q2bck`, `CFG` (`vec_igraph`), SCALUP, AQCDUP, and no written event
  rejected by `xqcut`.
- `pp_to_llj_xqcut_only`: 10000/10000 on every field it has (one call).
- Controls, so that the matched fields can tell the readings apart:
  - the record scale differs from the density scale on 8877 events (8749
    `_alps2`);
  - `CFG` differs from the integration channel on 525 (531).
  - SCALUP alone never sees the split on the llj rows: 0 events. It is the
    larger of two scales, and matching lowers only the smaller. The `t t~` row
    (194) and the mixed row (351) are what exercise it.
- `pp_to_ttx_0j1j_mlm`, `pp_to_ll_0j2j_mlm`: the engine replay agrees on
  10000/10000. The production path agrees on every event whose matrix-element
  momenta are the sampled point or its mirror (6265/6265 and 9859/9859). The
  rest are the finding below.

**Finding: the first call clusters the unpermuted point.**
- `DSIGPROC` computes `P1 = SWITCHMOM(PP, PERMS(MAPCONFIG(ICONFIG)))`, then
  mirrors it, and the matrix element reads `P1`.
- `update_scale_coupling(pp, wgt)` — the first call, which sets μR, the density
  scales and `q2bck` — is handed **`PP`** (`super_auto_dsig_group_v4.inc:842`).
  `rewgt`'s second call is handed `P1`.
- Where the symmetry permutation is not the identity, the first call therefore
  clusters an event whose momenta are exchanged between legs of equal mass but
  different flavour, against the flavour table of `P1`:
  - `t ↔ t~` on 3735 of the `t t~` row's events;
  - quark legs on 141 of the mixed row's.
- On 2 and 68 of those events the scales differ from clustering `P1`:
  - μR up to 9% on `t t~`;
  - SCALUP up to 80% on `P2_qq_llqq`.
- The replay from `PP` reproduces MadEvent's first call on every one of them
  (3735/3735, 141/141).
- A sampled term here is `P1`, so this crate's scale is the physically labelled
  one. Under §1.5's policy this is a defect that changes a weight, so it is to
  be reproduced (the scale taken on the sampling channel's own unpermuted
  point) or refused, not left silently fixed.
- By the same code path it reaches `ickkw = 0` runs too, where the only call
  is the first. Whether any banked `ickkw = 0` row carries such a
  configuration is not measured here: the banked scale gates compare against
  replays of `PP` and could not see it.
- It is recorded here, not changed: which of the two it becomes is the
  manager's call. The harness reports those events under `info, permuted P1:`
  and does not gate on them.

**The jet memo (M0's finding), rule and proof.**
- `reweight.f:662-679` restricts the clustering to `iconfig` whenever
  `njetstore(iconfig) = -1`. `:985-998` then stores the count of final-state
  `iqjets > 0` and re-clusters unrestricted; every later point compares its
  unrestricted count with the stored one and re-clusters restricted on a
  mismatch (`stop 4` if that fails too).
- MadEvent consults the memo on every call that clusters. That is every call
  under `ickkw > 0` or `xqcut > 0`, or when a scale is dynamic; it is skipped
  only on the `:643` early return.
- The port's rule was already "store the count of *this* event's restricted
  clustering, then proceed", which is MadEvent's value exactly when the
  restricted count is a property of the channel alone. That is plausible,
  since a restricted clustering follows the channel's forest, but the jet
  tagging reads kinematics (`ipartupdate`'s hardness comparisons), so it
  needed measuring.
- `the_jet_memo_is_the_channel_s_restricted_jet_count` measures it. It
  clusters every event of a directory restricted to **every** channel of that
  directory (not only the event's own), on all five rows. Every one of the 135
  channels (8 + 8 + 8 + 29 + 82) gives a single count over every event, and
  every census channel agrees (8/8, 8/8, 8/8, 29/29, 53/53). Per event, the
  production path's stored count equals the dumped `njetstore` on entry, and
  its restricted-re-cluster branch equals MadEvent's, on every event of every
  row.
- So the rule is MadEvent's value on every event, not only a channel's first,
  and no code changed. The doc on `cluster_scales` now says so, citing the
  measurement. `ickkw = 0` behaviour is untouched.
- The first run of the proof found counts of both 0 and 1 on `P2_qq_llqq`
  channels 17–24. The cause was the harness, not the rule: it had filled
  `confsub` with every channel for every subprocess. With the directory's own
  `config_subproc_map.inc` it is a single value on every channel.

**σ of `pp_to_llj_xqcut_only`** (`sigma_llj_xqcut_only_vs_madevent`, pixi task
`validate-mlm-sigma`, long tier, info; 150000 × 10 per seed):

| seed | σ (pb) | rel |
|---|---|---|
| 20260941 | 212.907 ± 0.306 | +0.17% |
| 20260942 | 212.799 ± 0.325 | +0.12% |
| 20260943 | 212.577 ± 0.351 | +0.01% |
| 20260944 | 212.346 ± 0.348 | −0.10% |
| 20260945 | 213.256 ± 0.331 | +0.33% |
| **mean** | **212.777 ± 0.149** (χ²/dof 1.03) | **+0.11%**, pull +0.84 |

The reference is MadEvent at 212.549 ± 0.229 pb (ten seeds, max(quoted,
spread/√n) = the quote). Five seeds cannot calibrate a difference below the
reference's own 0.11%. The row is recorded `info`, to flip at close-out from
the published bundle.

**Other fixes the references needed.**
- The llj MLM cards spell the PDF per beam (`lhapdf = pdlabel1/2`), which the
  run card refused.
- `pdlabel1/2` are now resolved as `banner.py`'s `PDLabelBlock` does at proton
  beams: equal labels set `pdlabel`, and different ones are refused
  (`AsymmetricBeamPdf`), as MadGraph refuses them. A card that never names the
  per-beam labels resolves to exactly what it parsed to. Fixed beams leave
  them inert.

**Left.**
- `rewgt` per event (M2's follow-up, on the same harness).
- `pp_to_llj_mlm`'s seeded σ (M2).
- The permuted-`P1` decision above.

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

#### M2 Landed (implementation), 2026-09-28

The reweighting is in and live on the per-term path. The per-event gates
against M0's dumps, and the seeded σ gates, come later, when the dumps are
committed. What changed, checked line by line against `reweight.f:1333-1824`:

- **`coupling/cluster/rewgt.rs`**: `rewgt(history, colors, settings,
  flavours, x, αs, x·f) → Rewgt`, a pure function. `Rewgt` lists every vertex
  (its codes after `ipartupdate`, `ipart(1, mother)`, and a class: `Core`,
  `Isr`, `Fsr`, or why not — `IsrNotParton`, `FsrNotPartonVertex`,
  `FsrNoPartonDaughter`) with its `αs` ratio (`q2`, numerator, `αs(μR)`), per
  beam every chain step (`n`, the line entering, its flavour, `x` after `z`,
  `q_prev`, `q_now`, and `First` / `Ratio` / `NoRise` / `PastLast`), the
  product, and a kill (`AlphaSScale`, `PdfDenominator`). The product equals the
  listed factors bit for bit (`product_of_factors`). `RewgtHistory` is what it
  reads of a `ClusterHistory`: the second call's merges (with `zcl`), `pt2ijcl`,
  `jlast`, `iqjets` and line codes, `q2bck`, the first call's `μR`, and the
  momenta.
- **Wiring** (`proton.rs`, `hadronic.rs`): `EventScaleSource::point_history`
  returns the scales and, under matching, the `RewgtHistory` of the same two
  calls. `per_group_sum` computes each member's factor per ordering
  (`member_rewgts`) and weights each member's luminosity by it
  (`term_luminosity`); the event selection draws `(member, ordering)` with the
  same factors (`ProtonEvent::group_rewgt`). At `ickkw = 0` the old expressions
  run unchanged.

Where §1.3 was wrong or incomplete against the source:

- **The factor is per flavour combination, not per group.** `DSIG` draws one
  `IPSEL` `∝ PD(IPSEL)` (`auto_dsig_v4.inc:141-147`), `rewgt` reads that
  combination's codes, and the product multiplies the whole `PD(0)·|M|²`. So
  the per-term path is per flavour group, per beam ordering **and per member**:
  each member's luminosity carries its own factor, the draw's expectation.
- **Which codes `rewgt` reads.** The incoming legs and the final-state *jets*
  take the drawn combination's codes (`:1531-1537`); every other line keeps
  `ipdgcl` as the scale walk left it, and `rewgt` re-runs `ipartupdate` over
  every merge, the core included (`:1577`), which re-transmits jet flavours
  onto the spacelike lines. A non-jet quark transmits nothing, so a `b` line at
  `maxjetflavor = 4` keeps the forest's `|tprid|`, and its density is read for
  `+5` whichever way the line runs.
- **`goodjet` on the external legs** (`:1538-1549`): a beam is a parton line if
  `isparton`; a final-state leg if `iqjets > 0`, or if it is a parton that is
  not a jet. A jet-flavour leg the walk did not tag is not a parton line.
- **The density floor is on `f`, not `x·f`**: `pdg2pdf` returns `x·f / x`
  (`pdg2pdf_lhapdf6.f`, `pdg2pdf = pdg2pdf/x`).
- **`x ← x·z` only when `0 < z < 1`**; the core's `z` is `1`.
- **The chain includes the core.** The `αs` restriction to
  `n < nexternal − 2` does not apply to it; the ratio at `jlast` is usually
  taken at the core.
- **A step that does not rise copies the scale** (`pt2pdf(mother) =
  pt2pdf(daughter)`). A rising step past `jlast` sets nothing; it is
  unreachable from a consistent walk, since the chain's scale is `q2bck` from
  `jlast` on and nothing later exceeds it (pinned as `past_jlast_...`).
- **`fake_id`** never occurs here: the forests never split a higher vertex
  (`configs.rs`, pinned by `no_forest_line_carries_a_code_outside_the_model`).
- **`ipartupdate` failing** (`stop 3`) refuses the event with an error rather
  than leaving the provenance unset.

**Known deviation class (for M0's census).** `ipdgcl` is carried across
events. A final-state leg that is not a jet keeps whatever code the table last
held — the subprocess's first combination until an event labels it a jet —
and `setclscales` reads the codes an earlier `rewgt` left (the previous
event's for the first call; this event's incoming codes, set at `:1437`, for
the second). `RewgtHistory::pdg` is this event's walk over the group
representative's codes, which is MadEvent's first event of a run. The two
differ only where a group's members differ in jet-ness or in which vertices
transmit a flavour, which §1.5's census sizes.

**Tests** (hermetic, `rewgt.rs`): ISR only, FSR + ISR, `goodjet` propagating
from untagged daughters, the combination's codes replacing the walk's, a
non-jet final-state leg keeping the walk's code, a `b` leg with
`maxjetflavor = 4` at `asrwgtflavor = 5` and `4`, `alpsfact = 2`, `pdfwgt = F`,
both kills (and their boundaries), the `q2bck` cap and `jlast`, the listed
factors against the product, and a negative control: dropping the `αs` ratio
moves a σ-like integral by more than ten times a 1 % gate. In `proton.rs`,
`matched_terms_carry_each_member_s_own_reweighting` reclusters sampled points
and recomputes every member's factor bit for bit, and checks the beam order of
the momentum fractions changes it.

**Byte identity at `ickkw = 0`**, binary against binary against `0759ea6`
(throwaway worktree, since deleted), on M1's six cases (`p p > l+ l- j` on
`pp_to_llj_fixed`'s and `pp_to_llj`'s cards, `p p > j j`, `p p > e+ e-`,
fixed-beam `g u > e+ e- u` and `e+ e- > mu+ mu-`; `integrate` 20k × 4, seed
7, `generate` 500 events): every `grid.bin.zst` identical, every LHE file
identical but for the header line naming the artifact path. A shared
`CARGO_TARGET_DIR` does not separate two worktrees of one workspace: cargo
hashes path packages workspace-relative, so the second build silently reused
the first's binary until `cargo clean -p` forced it. Compare binary hashes
before trusting such a check.

**Informational σ** (no reference committed yet): `p p > e+ e- j` on
`pp_to_llj_dyn`'s card with `ickkw = 1`, `xqcut = 20` (written in the
session's scratchpad, since M0's card was not committed), `integrate
--fixed-budget --neval 200000 --niter 10 --seed 20260928`: **σ = 268.52 ±
0.35 pb**, χ²/dof 1.35, 106 s wall on 4 shared cores. One seed, so not
evidence. For orientation only: M0's uncommitted working-tree MadEvent runs of
the same row read 267.1–268.9 pb over five seeds.

**Left for the dump-gate follow-up**: per event, factor by factor against M0's
dump (vertex classes, each `αs` ratio, each chain entry, the product); the
seeded σ of `pp_to_llj_mlm` and `pp_to_llj_mlm_alps2` (σ and per event); the
negative control at the σ gate; and sizing the stale-`ipdgcl` class with
M0's census.

#### M2 Landed (dump gates), 2026-09-28

**Harness.** `validate_mlm_dumps.rs` (`pixi run validate-mlm-dumps`) now
recomputes each matched event's `rewgt` from the production path's history
(`ClusterHistory::rewgt_history`, the same one the integrand reads) for the
flavour combination MadEvent drew (`RWLEG`'s `idup`), at `RWBEG`'s momentum
fractions. It compares in order, first divergence reported:
- per vertex (`RWVX`): the class (`CORE`/`ISR`/`FSR`/`NONE`/`KILL_Q2`), the
  lines, the codes after `ipartupdate`, `ipart(1, mother)`, and on a
  reweighted vertex `kt²`, `αs(alpsfact·kt)` and the ratio;
- `asref` and `jlast`;
- per beam (`RWPDF`): each step's vertex, flavour and action, `x` after `z`,
  `q²_now`, `q²_prev`, both densities and the ratio;
- the kill (`RWKILL`), the product against `RWEND`, and the product against the
  listed factors;
- a convention pin for the proton wiring: the clustering's beam 1 takes the
  `x` of the physical beam its leg 1 arrives on (`ib(1)` against the sign of
  `P1`'s leg-1 `p_z`), 10000/10000 on every matched row. That is the order the
  mirrored term's `[x₂, x₁]` assumes.

On `pp_to_llj_xqcut_only` it checks `rewgt ≡ 1`. Every tolerance is 1e-12:
this crate's reading of the NNPDF grid and its `αs` tabulation agree with
MadEvent's LHAPDF to a few ulp (worst 1e-15), not merely to the 1e-6 the
harness allows `AQCDUP`.

**Agreement** (n/N per field, every field listed above):

| row | gated events | agreeing on every factor | worst |
|---|---|---|---|
| `pp_to_llj_mlm` | 10000 | 10000 | 3.2e-15 (`q²`), product 8.7e-16 |
| `pp_to_llj_mlm_alps2` | 10000 | 10000 | 4.6e-15, product 8.7e-16 |
| `pp_to_llj_xqcut_only` | 10000 | `rewgt = 1`: 10000 | — |
| `pp_to_ttx_0j1j_mlm` | 6265 | 6265 | product 7.4e-16 |
| `pp_to_ll_0j2j_mlm` | 9859 | 9859 | 7.9e-15, product 2.9e-15 |

The mixed row carries the suite's only FSR vertices, all agreeing. No row has
a kill or a `NONE` chain step.

**Permuted `P1` (info, not gated).** On the events whose first call clustered
the unpermuted `PP` (M1's finding), `asref` and `q2bck` follow the first
call's scales:
- `pp_to_ttx_0j1j_mlm`: 3733 of 3735 agree on every factor. The 2 whose
  first-call scales differ also differ in `asref`, the density scales and the
  product (worst 6.5e-3).
- `pp_to_ll_0j2j_mlm`: 73 of 141 agree. On the other 68, the vertices, `x` and
  `αs(alpsfact·kt)` still agree; `asref`, `q²` and the product do not, and on
  2 of them the chain's actions differ too (a step that rises on one side and
  not the other).

**For the `@2` diagnosis.** All 68 are `@2` events: 53 `P2_qq_llqq`, 15
`P2_gg_llqq`, out of the row's 1214 `@2` events. On each, the weight factor
`rewgt · αs(μR)^n · f₁(x₁, μF₁) f₂(x₂, μF₂)` was formed both ways. Each side
used its own scales and factor, with this crate's `αs` and densities (which
agree with MadEvent's). Measured:
- MadEvent's factor over this crate's is 0.978 on average, from 0.675 to 1.405;
- this crate's over MadEvent's is 1.035 on average.

MadEvent's events are unweighted, so the difference these events make to `@2`'s
σ is about 68/1214 × 3.5% = **+0.2%**, not the +1.5% excess.

The dump only holds points MadEvent kept, so it cannot see one kind of
difference. A point that MadEvent's first call rejects on `PP` (an `xqcut`
jet vertex, or the factorisation floor), but that clusters cleanly on `P1`,
carries weight only here. It is the remaining candidate on the permuted
configurations, and the harness cannot measure it.

**Seeded σ** (`validate-mlm-sigma`, long tier, info; 150000 × 10 per seed;
MadEvent's ten seeds under the seed policy):

| seed | `pp_to_llj_mlm` σ (pb) | rel |
|---|---|---|
| 20260951 | 269.506 ± 0.457 | +0.54% |
| 20260952 | 268.614 ± 0.497 | +0.21% |
| 20260953 | 267.731 ± 0.419 | −0.12% |
| 20260954 | 269.043 ± 0.427 | +0.37% |
| 20260955 | 269.517 ± 0.478 | +0.54% |
| 20260956 | 267.651 ± 0.448 | −0.15% |
| 20260957 | 267.557 ± 0.413 | −0.19% |
| 20260958 | 269.117 ± 0.412 | +0.39% |
| 20260959 | 268.129 ± 0.405 | +0.03% |
| 20260960 | 268.733 ± 0.591 | +0.25% |
| **mean** | **268.560 ± 0.145** (χ²/dof 2.96) | **+0.19%**, pull +1.58 |

MadEvent: 268.060 ± 0.282 pb. The first five seeds alone read 268.882 ±
0.204 (χ²/dof 2.90, pull +2.37). The next five moved the mean down by 0.3 pb.
That movement, and a χ²/dof near 3, say the per-seed errors are understated
by about √3. The heavy tail `rewgt`'s range (1.0–2.1) adds to the weights is
the likely cause. With the error inflated by √χ²/dof, the pull is about +1.3.
Ten seeds cannot calibrate a difference below the reference's 0.1%.

| seed | `pp_to_llj_mlm_alps2` σ (pb) | rel |
|---|---|---|
| 20260961 | 240.586 ± 0.425 | −0.05% |
| 20260962 | 241.419 ± 0.401 | +0.29% |
| 20260963 | 240.735 ± 0.360 | +0.01% |
| 20260964 | 241.143 ± 0.396 | +0.18% |
| 20260965 | 241.375 ± 0.378 | +0.28% |
| **mean** | **241.052 ± 0.176** (χ²/dof 0.90) | **+0.14%**, pull +1.11 |

MadEvent: 240.710 ± 0.252 pb. This is the `alpsfact` convention pin. The
dump already shows the numerator read at `2·kt` on every reweighted vertex.

All three llj rows sit high by 0.11–0.19% (the pure cut +0.11%, M1). The
shift is common to them and does not follow the reweighting. A shared
offset of that size lies inside every reference's error and is not resolved
here.

**Negative control** (asserted by `validate_mlm_dumps`, more than 1%
required). Over MadEvent's unweighted events, σ without the `αs` ratios is
σ·⟨1/A⟩, where `A` is an event's product of `αs` ratios. This crate's ratios
and MadEvent's give the same ⟨1/A⟩ to six digits:

| row | ⟨1/A⟩ | effect of dropping the `αs` factor |
|---|---|---|
| `pp_to_llj_mlm` | 0.8547 | −14.5% (−39 pb, about 140 of the reference's errors) |
| `pp_to_llj_mlm_alps2` | 0.9522 | −4.8% |
| `pp_to_ll_0j2j_mlm` | 0.9337 | −6.6% |
| `pp_to_ttx_0j1j_mlm` | 0.9046 | −9.5% |

**What the dumps contradict or settle in this note.**
- The "known deviation class" of the M2 implementation record (stale
  `ipdgcl`) is empty on these rows. M0's census found no IPROC mixing jet
  and non-jet flavours, and every factor agrees here.
- §1.3 and the implementation record agree with the source on every event.
  One point is now measured: the chain's action at `n > jlast` (`NONE` /
  `PastLast`) is never reached.
- The M2 implementation record's byte-identity caution extends across
  sessions. A second worktree building into the same `CARGO_TARGET_DIR` (a
  concurrent session's) overwrote this worktree's test binary. The first
  dump-gate run here silently executed a build without the `rewgt` fields.
  This pass therefore built in a private target (`debug = 0`, 0.7 GB).

**Left.** The permuted-`P1` decision (reproduce or refuse) is still open. It
now has its size on `@2`: +0.2% from the kept events, plus an unmeasured
share from points only one side keeps. The llj σ rows flip from the published
bundle at close-out.

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

#### M3 Landed (implementation), 2026-09-28

A card of several final-state multiplicities now runs at proton beams, as
§3.3 designed it. What changed:

- **Proc card** (`diagrams/check.rs`): `Unsupported::MixedMultiplicity` is
  gone, and so is its row in the backlog table. Decision (a): an unmatched sum
  (`ickkw = 0`) runs, with a warning that it double counts
  (`multiplicity::split_by_multiplicity`). MadGraph itself logs nothing
  specific on such a card; it only auto-enables matching in the default card
  (`banner.py:4924-4966`). Fixed-energy beams and decays refuse a mixed card
  (`refuse_mixed_multiplicity` in the CLI), since only the proton path has
  the composite.
- **`multiplicity.rs`**: `split_by_multiplicity` partitions the enumeration
  by outgoing-leg count, in increasing order, keeping the card's order within
  each part. `MultiplicitySum` owns one fully configured `ProtonIntegrand` per
  part and exposes their channels as one list with offsets. Each part keeps
  its own `αⱼ`, normalised over its own channels, so each part's terms sum to
  its own σ. `channel_keys` gives `ChannelKey::MultiplicityChannel
  { final_state, group, channel }`, except with a single part, which keeps
  `GroupChannel`. `part_results` gives σ per multiplicity.
- **Trait**: `ChannelIntegrand::channel_grid_ndim(channel)`. §3.3 asked for a
  default that returns the old constant, but the trait holds no constant a
  default could return. So the method is required, and every uniform
  implementer ignores the index. The concrete types keep their inherent
  zero-argument `channel_grid_ndim()`, so none of the concrete call sites
  changed. `integrate_channels` builds each grid at its channel's dimension;
  `Unweighter` scans and draws each channel at its own.
- **Budget across multiplicities**:
  - Each part is α-surveyed on its own mixture.
  - The per-iteration budget is split by `nₖ ∝ sₖ`, where `sₖ` is the
    standard deviation of part `k`'s mixture estimator. It is formed as
    `√(Σⱼ αⱼ Wⱼ − σₖ²)`. The survey now also returns its mean `σₖ`
    (`ProtonIntegrand::survey_mean`, pinned to the mean of `value` over the
    survey's own points). `Wⱼ` is unchanged bit for bit.
  - The allocation per channel is `αⱼ · share(k)`. The banked `alpha` is the
    term's own `αⱼ`, which is what a replay installs.
  - Under a Neyman allocation, every channel is re-split by its measured
    spread from the second iteration on (see the finding below).
- **Maps**: an artifact banks one set of map choices. For several parts they
  are settled once over the union of the parts' shapes (`union_shape`). The
  only shape-dependent choice is the split angle. A part with no
  soft-emission split is unaffected by the soft-emission map, so the union
  settles each part where it would settle alone.
- **Artifact** (format 10): adds `ChannelKey::MultiplicityChannel`.
  - The writer records the oldest version whose schema holds its keys
    (`IntegrateArtifact::version_for`). A single-multiplicity artifact
    therefore stays a version-9 file, byte for byte, and an older reader
    still reads it.
  - A version-9 file decodes directly.
  - `generate` refuses an artifact below version 10 on a card of several
    multiplicities, naming both versions. This follows the pattern of
    `refuse_stale_artifact_on_clustering_scale`.
  - An older build refuses a version-10 file by its version.
- **Record**:
  - Each event's `IDPRUP` is its member's `@N`. MadEvent writes the number of
    the `P<n>` directory, and `group_subprocs.py:444,675` sets that number to
    the group's first process's `id`, i.e. its `@N`. M0's run reads LPRUP
    0/1/2 on this card, so the two readings coincide here.
  - `<init>` keeps one line per `@N`, with `XSECUP` split by the sample's
    shares (note 38 E1).
  - The unweighter needed no change: it draws each channel `∝ w_maxⱼ`,
    whatever multiplicity the channel belongs to.

**Byte identity.** Single-multiplicity runs were compared binary against
binary with `562ccb5`, on M1's six cases (`integrate` 20k × 4, seed 7;
`generate` 500 events):
- binaries: base `efc94d3e…`, new `d1979529…`; `cargo clean -p` ran before
  each build;
- every `grid.bin.zst` is identical, with the same sha256 prefixes M2
  recorded (`a7946b89…` for `llj_fixed`);
- every LHE file is identical except the header line that names the artifact
  path.

**Tests** (hermetic):
- `multiplicity.rs`:
  - the split order;
  - offsets, `locate`, per-channel `ndim` (4 and 7), keys and samplers, and
    values bit for bit against the parts;
  - a single part is its part bit for bit (terms and grids);
  - the sum's σ per part matches each part integrated alone (pulls +0.17,
    +0.75), and an unweighted sample splits across multiplicities as σ does;
- `budget.rs`: a two-dimension toy integrates to `1 + 1/3`;
- `artifact.rs`: `version_for` and round trips at versions 9 and 10;
- `check.rs`: the card passes with ids 0/1/2;
- CLI:
  - `cli_hard_errors`: the fixed-energy refusal;
  - `cli_generate_proton::a_mixed_multiplicity_card_is_integrated_and_sampled_as_a_sum`:
    `@0 + @1` on the banked card end to end — the warning, version 10, keys,
    grid dimensions, per-part `αⱼ` sums, `IDPRUP` with its leg count, the
    sample split, and the stale-version refusal.

**Finding (budget), to file.** On `pp_to_ll_0j2j_mlm`, the 336 two-jet
channels are floor-bound: every one of them sits at the accepted-point floor
(512, raised up to the 2048-point cap). So the two-jet σ is set by the floors
and not by the multiplicity split. The same seed gives the same two-jet
result bit for bit under by-α and under Neyman.

Neyman reallocation keeps the iteration total at the pre-correction sum. The
floors eat most of that, which starves the four `@0` channels: `@0` came out
at ±6.6 pb under Neyman against ±0.70 pb by α, at the same seed and budget.
Neyman is the default of a `--target-rel` run, so on this card the default
integration is several times less efficient than `--fixed-budget` by α. This
is a property of `neyman_allocation`'s total, which the composite exposes. It
is not specific to the composite. It belongs with M6 or a performance pass.

**Cost.** The two-jet part dominates, at ~250 µs per point on 4 cores, and
0.3–0.9 ms under the 3× oversubscription the container had during the sweep.
Each point sums the 336-channel mixture density and reclusters every member
under matching. One `--fixed-budget --neval 200000 --niter 8` seed took
2235 s of wall time on the shared container; MadEvent took 101–125 s per seed
on the same row.

**σ, informational** (§4 M1 is changing the per-channel jet memo, which M0
found firing on 6.6 % of this row's events, so matched σ will move; the gate
stays informational until that lands):

`pp_to_ll_0j2j_mlm` on M0's card (byte-identical to the committed
`pp_to_ll_0j2j_mlm_run_card.dat`), `integrate --fixed-budget --neval 200000
--niter 8` by α, seeds 20260928–32, final binary. Per seed:

| seed | @0 (pb) | @1 (pb) | @2 (pb) | total (pb) | wall |
|---|---|---|---|---|---|
| 20260928 | 665.02 ± 0.70 | 268.13 ± 1.48 | 132.76 ± 1.01 | 1065.90 ± 1.92 | 2235 s |
| 20260929 | 665.05 ± 0.68 | 269.82 ± 2.12 | 133.22 ± 2.41 | 1068.09 ± 3.28 | 1367 s |
| 20260930 | 665.11 ± 0.69 | 273.84 ± 2.86 | 132.33 ± 0.88 | 1071.28 ± 3.07 | 1230 s |
| 20260931 | 666.03 ± 0.68 | 266.32 ± 1.49 | 131.35 ± 0.85 | 1063.70 ± 1.84 | 1545 s |
| 20260932 | 665.37 ± 0.69 | 268.32 ± 2.46 | 132.24 ± 0.92 | 1065.94 ± 2.72 | 1129 s |

Against M0's ten MadEvent seeds (seeds 2–10 share grids):

| | vibegraph, 5 seeds (sd, χ²/dof) | MadEvent, 10 seeds (sd) | difference |
|---|---|---|---|
| @0 | 665.32 ± 0.19 (0.42, 0.39) | 664.80 ± 0.32 (1.00) | +0.08 %, +1.4σ |
| @1 | 269.29 ± 1.27 (2.83, 1.83) | 269.11 ± 0.33 (1.05) | +0.07 %, +0.1σ |
| @2 | 132.38 ± 0.31 (0.70, 0.44) | 130.44 ± 0.20 (0.63) | **+1.49 %, +5.2σ** |
| total | 1066.98 ± 1.28 (2.86, 1.43) | 1064.37 ± 0.62 (1.97) | +0.25 %, +1.8σ |

`@0` and `@1` agree. `@2` is 1.5 % high, well outside both spreads.

The sum is not what moves `@2`: the part's channel terms are the part's own,
bit for bit (the `multiplicity.rs` tests). The composite does change the
budget, but each two-jet channel sits at its floor either way.

Two causes are still open:
- the jet memo, whose restricted re-cluster M0 counted on 6.6 % of this
  row's events;
- the convergence of 336 floor-bound, heavy-tailed channels.

**Budget ladder** on the two-jet channels. The rung is `p p > e+ e- j j @2`
alone, run by the `562ccb5` binary — the code path before this session — at
`--neval 200000 --niter 16`, seeds 20260928–30:

| | @2 (pb) |
|---|---|
| rung, per seed | 132.22 ± 0.45, 132.62 ± 1.04, 133.00 ± 0.50 |
| rung mean | 132.61 (sd 0.39) |
| same runs' running estimate after 8 iterations | 132.03, 131.81, 132.52 |
| the sum at 8 iterations (table above) | 132.38 (sd 0.70) |

What the ladder says:
- **The excess predates the composite.** It is the same without the sum and
  without this session's code.
- **It does not shrink with iterations.** A floor-bound heavy tail
  converging from below would show that, and it does not.
- **`--neval` cannot extend the ladder.** Every two-jet channel is at its
  floor, so a larger `--neval` leaves their points unchanged; iterations are
  the only knob.

Three seeds per rung cannot calibrate a rung-to-rung difference (AGENTS.md
asks for 20 or more). They can bound a drift of the 1.5 % size, and they show
none. The jet memo is the next suspect. It is M1's fix, and this row should be
re-measured once that lands.

**Left for later**:
- gating σ per `@N` and the `samples` fractions against M0's reference, once
  M1's jet-memo fix lands and `@2` is diagnosed;
- the Neyman-allocation finding;
- the per-point cost of wide mixtures, a performance item beside M6.

The unweighted sample (seed 20260928's artifact, 10 000 events, 532 s) split
by `IDPRUP` as 0.6346 / 0.2481 / 0.1173 for `@0` / `@1` / `@2`. MadEvent's
samples-grade run splits 0.6264 / 0.2522 / 0.1214. The differences are +1.2σ,
−0.7σ and −0.9σ in the two samples' combined binomial error, which sees
nothing at the 1.5 % level. The sample's σ is within 0.12 % of the
integration's; the efficiency is 3.5 %, and 4.5 % of σ sits above `w_max`
(largest `w/w_max` 62).

#### D2 diagnosis: the `@2` excess, 2026-09-28

The question was M3's `@2`: 132.38 ± 0.31 pb against MadEvent's 130.44 ± 0.20,
which is +1.49 % and 5.2σ. No production code changed. The throwaway probes
lived in the worktree and were reverted. They were rebuilt and re-run in a
private target: test binaries `33b47372…` (dump harness) and `021b3f74…`
(lib), with identical outputs.

**Answer.** The +1.94 pb is not one vibegraph defect. It splits into four
parts:

| part | pb | how it was measured |
|---|---|---|
| the reference sits low against independent MadEvent runs | +0.65 | 7 fresh directories vs M0's ten seeds |
| H1, MadEvent's first `setclscales` call on `PP` | +0.33 ± 0.11 | MadEvent patched to hand the first call `P1`, 4 runs vs 5 |
| M3's composite vs vibegraph run per directory | +0.38 ± 0.33 | not significant |
| a generic 2 → 4 offset, present without matching | ≈ +0.6 | fixed scale, no `xqcut`; also at fixed beams |

The first three are specific to this row. The fourth is not MLM.

**Localisation by subprocess directory** (MLM card, pb). The columns are:
- MadEvent reference: M0's ten seeds of the mixed card. Seeds 2–10 share one
  directory, and in the mixed card `@2` gets about 12 % of the events.
- MadEvent fresh: `p p > e+ e- j j` alone, 10000 events, one freshly
  generated directory per seed. Five seeds, plus two at 50000 events.
- MadEvent patched: the same fresh directories, with the first call handed
  `P1`. Four seeds.
- vibegraph: `gg`, `gq` and `llgg` are one run each at `--neval 2000000
  --niter 8`. `qq_llqq` is three seeds at 200000 × 8. Every run is one
  directory's subprocesses integrated alone, with M3's binary `d1979529…`.

| directory | MG ref | MG fresh | MG patched | vibegraph | vg / fresh | vg / patched |
|---|---|---|---|---|---|---|
| `P2_gg_llqq` | 11.353 ± 0.042 | 11.378 ± 0.040 | 11.322 ± 0.006 | 11.369 ± 0.007 | −0.1 % | +0.4 % |
| `P2_gq_llgq` | 91.418 ± 0.178 | 91.934 ± 0.126 | 91.817 ± 0.116 | 92.320 ± 0.069 | +0.4 % | +0.6 % |
| `P2_qq_llgg` | 9.670 ± 0.030 | 9.710 ± 0.022 | 9.659 ± 0.014 | 9.733 ± 0.007 | +0.2 % | +0.8 % |
| `P2_qq_llqq` | 17.999 ± 0.073 | 18.063 ± 0.062 | 18.394 ± 0.077 | 18.579 ± 0.063 | +2.9 % | +1.0 % |
| sum | 130.44 | 131.09 | 131.19 | 132.00 | +0.7 % | +0.6 % |

Errors are the spread over seeds divided by √n. Every MadEvent column scatters
more than it quotes. For example, `gq` has a spread of 0.33 pb across fresh
directories, against 0.22 quoted per run. The two 50000-event fresh runs read
131.00 and 130.54, so more events do not raise MadEvent's `@2`. Fresh
mixed-card directories read `@2` = 130.46 (M0's samples seed), 130.51, 131.64
and 130.61, a mean of 130.81 ± 0.28.

**Eliminated, and by what.**
- **Scales, `rewgt`, PDFs, α_s, per event.** On MadEvent's own events, the
  weight factor f₁f₂(μF) · α_s^n(μR) · `rewgt` was compared, each side at its own
  scales. The ratio is exactly 1 (at 1e-9) on all 9859 non-permuted events, and
  each of the three factors is 1 separately.
  - MadEvent's side was taken from the dump: the first call's `SCLOUT` q2fact
    and μR, `RWEND`, and the drawn combination's `RWLEG` codes.
  - Absolute densities agree with the dumped LHAPDF values on all 4089 `RWPDF`
    points, worst 5e-16.
- **Matrix element and symmetry factors (H3).** |M|² was compared with a
  MadGraph standalone build on MadEvent's own event momenta:
  - `u u~ > e+ e- g g` (400 points), `g u > e+ e- g u` (300), `u d > e+ e- u d`
    (104), `u u > e+ e- u u` (137), including 31 points with √ŝ > 600 GeV;
  - all agree at 6e-8, which is the precision of the LHE records;
  - the identical-particle 1/2 of `g g` and `u u` is where the member carries
    it.
- **The flavour sum.** Every one of the 1214 `@2` events maps to a vibegraph
  member. The localisation shows no missing or doubled group.
- **The clustering configuration draw.** Under matching, the weight depends
  on the configuration on every `gq`, `gg` and `qq_llqq` event. Through the
  jet memo's restricted re-cluster, it moves by up to a factor of 2.
  - vibegraph draws the configuration ∝ `AMP2`. Its expected factor over
    MadEvent's at MadEvent's own channel, E_vg/W_MG, is 0.994 ± 0.005 on `@2`
    (0.988 ± 0.006 on `gq`).
  - The statistic of `validate_hadronic`'s draw test agrees on the events it
    can read: MadEvent's channel landing in the highest-weight class gives
    pulls of −0.57 (`gq`), −1.02 (`qq_llqq`) and +2.20 (`gg`).
- **vibegraph's sampler (H2, our side).** Ten times the budget moves no
  directory:
  - `qq_llgg`: 9.732 ± 0.022 over five seeds, against 9.733 ± 0.007;
  - `gg`: 11.381 ± 0.008 (five seeds) against 11.369 ± 0.007;
  - `gq`: 92.13 ± 0.24 against 92.32 ± 0.07.

  The default card's split-angle map, and three others, agree at fixed beams
  (below).

**H1 is real, and lives in `P2_qq_llqq`.**
- On kept events it is nothing. On MadEvent's 68 permuted `@2` events whose
  first-call scales differ (53 `qq_llqq`, 15 `gg`), W_vg/W_MG averages 0.998
  (0.58–1.63). That moves `@2` by −0.01 % ± 0.10 %.
- Its size is in the points only one side keeps. The patched MadEvent
  measures that inside MadEvent itself: `qq_llqq` rises by 0.33 ± 0.11 pb
  (+1.8 %), and `gg` does not move (−0.04 ± 0.06).
- M2's figure (+0.2 % on kept events) read MadEvent's PDF scale from `RWBEG`'s
  q2fact. That is the *second* call's output on 10000/10000 events, and equals
  the first call's on only 7284. `DSIG` evaluates the densities before `REWGT`,
  at the first call's q2fact (`auto_dsig1.f`, `QSCALE = DSQRT(Q2FACT(…))`
  before `REWGT(PP,1)`). `gen_kt_cluster_dumps.py`'s docstring calls those
  fields "the matrix-element PDF scales", which is wrong for the same reason.

**The generic offset.** It survives every piece of MLM being switched off:
- **Fixed scales, `ickkw = 0`, `xqcut = 0`, plain `ptj = mmjj = 20`.**
  vibegraph reads 102.36, against MadEvent's 101.65 ± 0.14 (six runs), which is
  +0.7 %. By directory: `gg` +0.6 %, `llgg` +0.2 %, `gq` +0.7 %, `qq_llqq`
  +0.9 %.
- **Fixed beams, √s = 500 GeV, `u u~ > e+ e- g g`, the same cuts.**
  vibegraph reads 0.52894 ± 0.00024 (four seeds at 2M × 8), against MadEvent's
  0.52743 ± 0.00040 (six runs, four of them in fresh directories), which is
  +0.29 %, 3.2σ.
  - There are no PDFs or running scales here, and |M|² is identical point by
    point.
  - vibegraph's own split-angle maps agree with one another: isotropic
    0.52945, windowed 0.52934, soft-all 0.52900.

  This is the cheapest reproducer: about 30 s per MadEvent run and 75 s per
  vibegraph run.
- On `qq_llgg` the unweighted samples place vibegraph's surplus in the
  high-ŝ tail: m_ll > 150 is +14 %, and ŝ > 600 GeV at the peak is +9.5 %.
  MadEvent's own dedicated m_ll > 150 run (0.2341 ± 0.0004) sits 11 % above
  its inclusive run's tail and 0.7 % below vibegraph's 0.2357 ± 0.0002.
- It is not settled which side is right. Two things point at MadEvent's
  coverage of tails:
  - its dedicated slice recovers most of the tail;
  - its runs scatter beyond their quoted errors.

  A vibegraph acceptance difference in regions MadEvent never populates is
  the one class the per-event oracle cannot see.

**Recommendations.**
- **H1 is a policy decision.** It is +1.8 % on `P2_qq_llqq` and +0.25 % on
  `@2`, mostly through the first call rejecting points it clusters
  as `PP`.
  - Reproducing it needs MadEvent's per-channel symmetry permutation
    (`SYMCONF`, `PERMS(MAPCONFIG)`), which the integrand does not carry: its
    channels are its own diagrams.
  - Refusing would refuse this canonical card.
  - Documenting the deviation with this measured size is the cheaper choice.
    The user decides.
- **Regenerate `@2`'s reference from independent directories.** Use one
  freshly generated directory per seed, and preferably `@2` in its own run.
  Its error should be the spread over those directories, not the per-run quote.
- **Gate at a tolerance that includes MadEvent's measured inter-directory
  spread.** That is about 0.2–0.4 % per `@2` run.
- **File the generic 2 → 4 offset as its own validation item**, with the
  fixed-beam `u u~ > e+ e- g g` reproducer. It is outside MLM.

#### D2 decisions (user, 2026-09-29)

- **H1 is documented, not reproduced or refused.** vibegraph clusters the
  first `setclscales` call on the same (permuted) momenta as the matrix
  element and the second call. MadEvent's first call reads the unpermuted
  `PP` (`super_auto_dsig_group_v4.inc:805,842`). The measured size is +1.8 %
  of `P2_qq_llqq` and about +0.25 % of `pp_to_ll_0j2j_mlm`'s `@2`, mostly
  through first-call rejections. It is a registered deviation: the manifest
  notes of the affected rows name it, per-event gates keep reporting the
  permuted events as `info, permuted P1`, and note 07 carries the upstream
  report draft.
- **The `@2` reference is regenerated at close-out** from one freshly
  generated MadEvent directory per seed. Its error is the spread over those
  directories, and the σ gate's tolerance includes MadEvent's measured
  inter-directory spread (about 0.2–0.4 % per `@2` run).
- **The generic 2 → 4 offset is its own validation item** (TODO.md), with
  the fixed-beam `u u~ > e+ e- g g` reproducer. It does not block MLM.

#### R1: a MadGraph-only reproducer for H1, 2026-09-29

H1 is now demonstrated inside MadGraph alone, and the report draft is in note 07 ("`super_auto_dsig_group_v4.inc` —
Direct Bug Found"). The reproducer is `validation/madgraph/repro/permuted_first_call/`. The fix is
`validation/madgraph/patches/first-call-unpermuted-momenta.patch`.

**The test.** The process is `define q = u d; generate u q > z u q`, at 13 TeV.
- It compares the grouped output with `group_subprocesses False`, and with the grouped output carrying the
  one-line `PP → P1` fix.
- Each variant runs five seeds in freshly generated directories. The error is the spread over those seeds.
- Grouping puts `u u` and `u d` in one directory. Configs 2, 5, 6 and 8 are then integrated as the final-quark
  swap of others, and two of `u d`'s four diagrams lie on them.

**Results.**

| card | grouped (pb) | non-grouped (pb) | grouped + fix (pb) | grouped vs non-grouped | fix vs non-grouped |
|---|---|---|---|---|---|
| `ickkw = 1`, `xqcut = 40` | 54.48 | 56.14 | 56.33 | −3.0 %, −9.8σ | +1.2σ |
| default card (`ickkw = 0`, `dynamical_scale_choice = -1`) | 104.28 | 108.60 | 108.62 | −4.0 %, −7.6σ | 0.0σ |

**Findings.**
- H1 therefore reaches unmatched runs at MadGraph's default dynamical scale.
- It is not limited to flavour swaps:
  - With MLM off, the channel whose only permuted config swaps `u u`'s identical quarks rises by 5.7 % (18σ)
    once the fix is applied.
  - The other channel of that kind is identical on every seed.
  - The clustering path responsible is not isolated.
- Rejected candidates:
  - `u u~ > e+ e- u u~` alone: patched and unpatched are bit-identical; its permuted channels carry 0.17 % of
    σ.
  - `g g > e+ e- u u~` and `t t~ j`: their swaps leave the clustering pairs symmetric.
  - The same subprocesses written with `add process` land in separate directories and have no permutation.
- The vectorised path (`update_scale_coupling_vec`, `:312`) has the same shape. It has no one-line fix and is
  not measured.
- For this crate:
  - The D2 decision stands: vibegraph keeps the fixed behaviour.
  - Grouped MadEvent references that use `dynamical_scale_choice = -1` and contain non-identity permutations
    are biased whether or not they are matched.
  - No banked `ickkw = 0` row has been checked for such permutations.

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

#### M4 Landed, 2026-09-29

A matched run (`ickkw = 1`) now writes the event record a shower's MLM
matching reads: `<scales pt_clust_N>` after each event's particles, the
status-2 resonances the clustering found on their Breit–Wigner, and an
`<MGRunCard>` in the header. At `ickkw = 0` nothing changes, byte for byte.
Every rule was read from the pinned source first; where this section's plan
text was short of it, the list at the end says so.

**What changed.**
- **`ptclus`** (`coupling/cluster/setclscales.rs`, `ptclus`): `reweight.f:1225-1269`
  as a pure function of a finished call's `ClusterScales` — per merge, its two
  daughters (the terminal vertex's are its first beam line and the leftover
  line), each daughter's two `ipart` legs; a final-state leg takes the largest
  `√pt2ijcl(n)` among jet vertices (`isjetvx`) where the daughter is still
  `goodjet`, otherwise, if still unset, `etot = √stot`. The `goodjet` demotion
  (`iqjets = 0` on an `ipart` leg) is applied before the vertex test of the same
  entry, as in the source. `etot` is `genps.f:653-676`'s formula with the proton
  mass 0.938: it cancels analytically (`stot = 4E₁E₂`) and lands on
  12999.999999999998, which is what MadEvent dumps; the record prints
  `13000.00000`.
- **The record fields** (`coupling/scales.rs`, `MatchedRecord`,
  `ClusterHistory::matched_record`): the second call's `ptclus` and its
  `isbw` leg sets (`Clustering::tagged`, the integration channel's
  propagators `cut_bw` put on their Breit–Wigner). They travel with each
  term (`PointHistory::Scales::record`, `ProtonEvent::group_records`,
  `ProtonSelection::record`); `EventScales` stays `Copy` and unchanged.
- **The resonances** (`lhef/resonance.rs`, `clustered_on_shell`; CLI
  `matched_event_record`): a timelike line of the *clustered* configuration is
  written when its leg set is one of the `isbw` sets (`addmothers.f:257-264`),
  none when the drawn flow is below leading colour there (`is_LC`); the
  layout is the existing `event_with_intermediates` (mothers `1 2`, the
  daughters' sum and virtuality, open colour, `SPINUP 9`). Under matching the
  tables are built for every card, not only decay-chain ones.
- **`<scales>`** (`lhef/build.rs`, `pt_clust_scales`): one `pt_clust_N` per
  outgoing line, `N` its 1-based position (so shifted by the status-2 lines,
  `ito(i)` in `addmothers.f:411-429`), `f16.5` trimmed.
- **`<MGRunCard>`** (`lhef/write.rs`, `mg_run_card`; `RunCard::banner_values`;
  `LheWriter::begin_with_blocks`, `EmitPlan::header_blocks`): every resolved
  parameter as `value = name`, written only for matched runs. The values are
  MadGraph's record of the card, which is not the resolved card: `banner.py`
  writes the card after its own edits (`alpsfact = 1` under matching with
  `use_syst`, `drjj = drjl = 0` and, without `auto_ptj_mjj`, `mmjj` above
  `xqcut` zeroed under `xqcut`), before `setcuts.f`'s `ptj = mmjj = xqcut` and
  `setrun.f`'s `alpsfact` rule, which rerun on any card read back.
  `RunCard` keeps those few values beside the resolved ones (`#[serde(skip)]`,
  so an artifact is unchanged).

**The CDATA finding.** MadGraph wraps the card in `<![CDATA[ … ]]>`. Pythia
8.312 (the pixi `pythia` environment) drops a CDATA section's content: on
M0's own `pp_to_ll_0j2j_mlm` file `Info::header("MGRunCard")` is 4 bytes, and
`JetMatching:setMad = on` warns "Madgraph merging parameters not found" and
runs at its defaults (`qCut = 10`, `nQmatch = 5`). The same file with the two
CDATA markers deleted gives the full 14084-byte card and `qCut = 20`,
`nQmatch = 4`, `clFact = 1`. That is presumably why MadGraph drives Pythia with
`setMad = off`. This crate writes the card as escaped element text (only `<`,
`>`, `&`), which Pythia reads.

**Per event against MadEvent** (`validate_mlm_dumps`, `compare_record`; the
production path's second call on each dumped event, against the dump and the
banked `unweighted_events.lhe.gz` read in the dump's order):

| row | gated | `ptclus` = `PTCL SETCL` / `OUT` | `<scales>` in the file | status-2 (code, legs) | status-2 (mothers, mass, colour) | events with a Z |
|---|---|---|---|---|---|---|
| `pp_to_llj_mlm` | 10000 | 10000 / 10000 (worst 0) | 10000 | 10000 | 10000 | 9507 |
| `pp_to_llj_mlm_alps2` | 10000 | 10000 / 10000 | 10000 | 10000 | 10000 | 9495 |
| `pp_to_ll_0j2j_mlm` | 9859 | 9859 / 9859 | 9859 | 9859 | 9859 | 9477 |
| `pp_to_ttx_0j1j_mlm` | 6265 | 6265 / 6265 | 6265 | 6265 | 6265 | 0 |

- `<scales>` is compared as the file's own string: every outgoing line `N`
  (keys shifted past the Z), the leg it holds found by `p_x`, `|p_y|`, and
  `format!("{:.5}")` of this crate's value for that leg equal to the printed
  one.
- The status-2 expectation is built from MadEvent's own forest of the
  clustered configuration (`CFG`, already gated by M1) and this crate's `isbw`
  sets, and compared with the file's status-2 lines: codes, the legs each
  mothers, mothers `1 2`, mass = daughters' virtuality, colour = what they
  leave open. The writer's own layout (`event_with_intermediates`) is the
  decay-chain one already gated; `cli_generate_proton` checks the matched
  file's layout end to end.
- Permuted `P1` (D2 decision, reported `info, permuted P1`, not gated): 3735
  `t t~` and 141 Drell–Yan events, and every record field agrees on all of
  them too. The record reads the second call, which clusters `P1` on both
  sides, so the H1 deviation does not reach the record.
- The control, and its limit: on every event with a Z, the on-shell leg set
  read in the integration channel's forest carries the same code (23) as in
  the clustered configuration's. `cluster.f:811-817` hands `igraphs` to the
  integration channel whenever it is compatible, and `isbw` needs the channel
  to have a Z on its window, so the clustered configuration is the Z one.
  These rows cannot tell the clustered-configuration reading of the
  resonance code from the channel reading; a row where they differ would.
- No `t t~` event lists a resonance (the tops are external and nothing else
  is on a Breit–Wigner), and no Drell–Yan event lists a photon.

**`<MGRunCard>` field by field** (`mg_run_card_matches_madevents_banner`,
the card this crate writes for each row's committed card against the banner of
M0's file, names compared without case): 64/65 fields both carry agree on the
llj rows, 50/51 on `t t~`, 84/85 on the mixed row. The one difference is
`iseed` (MadEvent records its run seed; this crate the card's, since its
generator seed is a flag naming another random stream). `ickkw`, `xqcut`,
`maxjetflavor`, `alpsfact` agree on every row (`alpsfact` 2.0 on `_alps2`).
MadEvent alone writes eight empty or `{}` list parameters; this crate alone
writes 110–144 hidden ones MadEvent leaves out. `mmjj` reads `0.0` on both
sides of the mixed row: the resolved card's 20 is `setcuts.f`'s, which is why
the record is not the resolved card.

**The matched sample** (`cli_generate_proton`):
- `a_matched_sample_carries_the_shower_record` (banked): `pp_to_llj_mlm`'s
  card, 3000 events — `<MGRunCard>` with the four `setMad` fields and the
  card's `ptj`, one `<scales>` per event keyed exactly by the outgoing
  lines, leptons at `13000.00000`, jets there or at ≥ `xqcut`, the Z with
  mothers `1 2`, colour `0 0`, `SPINUP 9`, its leptons alone below it, mass
  within 15 widths.
- `matched_sample_record_fractions_against_madevent` (oracle,
  `pixi run validate-mlm-samples`): 3000 events off a 20000 x 4 integration (seed 20260731) against the banked 10000: the Z on 0.9543 against 0.9507 (pull +0.81), a jet at the collider energy on 0.1183 against 0.1123 (pull +0.91).
- The mixed row's smoke sample (`--neval 50000 --niter 4`, seed 20260929,
  3000 events): the Z on 0.9487 of events (MadEvent 0.9477 of 10000), a jet
  at the collider energy on 0.0630 (0.0668), `pt_clust` keys equal the
  outgoing lines on 3000/3000 (10000/10000), IDPRUP 1914 / 730 / 356.

**Byte identity at `ickkw = 0`**, binary against binary against `069a951`
(throwaway worktree with its own target, since deleted; binaries
`3df53a9e…` base, `4702425f…` final), M1's six cases (`integrate` 20k × 4,
seed 7; `generate` 500 events): every `grid.bin.zst` identical (`a7946b89…`
`llj_fixed`, `b831c9d4…` `llj`, `0c12fa8c…` `jj`, `351337ce…` `dy`,
`6bebef12…` `gu`, `d640093a…` `ee`, the prefixes M2 and M3 recorded), every LHE
file identical but for the header line naming the artifact path. The run
card's banner values are `#[serde(skip)]`, so no artifact changes, and every
record field is written only when the clustering's second call exists.

**Pythia** (pixi `pythia` environment, 8.312):
- The consumption gate's own passes (`consume.run_pass`) on the mixed smoke
  sample: process level 3000/3000 consumed, none refused, none mismatched,
  read to end of file, no message; shower level 3000/3000, with Pythia's own
  shower complaints (`weight above unity`, one `stuck in loop`). MadEvent's
  file cannot be put through the same driver: its header is not well-formed
  XML for `ElementTree`.
- Matching (a C++ driver, `JetMatchingMadgraph` with `jetAlgorithm = 2`,
  `coneRadius = 1.0`, `nJetMax = 2`, `scheme = 1`,
  `setProductionScalesFromLHEF = on`, 1500 accepted events each):
  | file | mode | `MGRunCard` read | qCut, nQmatch, clFact | LHE events for 1500 kept |
  |---|---|---|---|---|
  | vibegraph | MadGraph's (`setMad = off`, `qCut = 1.5 xqcut`) | 3079 B | 30, 4, 1 | 2287 |
  | vibegraph | `setMad = on` | 3079 B, xqcut 20, ickkw 1, maxjetflavor 4, alpsfact 1 | 20, 4, 1 | 2419 |
  | MadEvent | MadGraph's | 4 B (CDATA dropped) | 30, 4, 1 | 2348 |
  | MadEvent | `setMad = on` | 4 B: "Madgraph merging parameters not found", No xqcut/ickkw/maxjetflavor/alpsfact | 10, 5, 1 | 3464 |
  | MadEvent, CDATA markers removed | `setMad = on` | 14084 B, same four values | 20, 4, 1 | 2461 |

  Pythia reads our `<scales>`, resonances and card with no warning about the
  record; the parameters `setMad` reports are MadEvent's own, where Pythia can
  read MadEvent's card at all. `setMad = on` sets `qCut = xqcut`, not
  MadGraph's `1.5 xqcut`. The acceptances are a smoke reading, not a
  comparison (M5).

**Other fixes.**
- `gen_kt_cluster_dumps.py`: `RWBEG`'s `q2fact` fields are the second call's,
  not the densities' scales (D2); the docstring says so, with
  `auto_dsig_v4.inc:127-151`. The harness's informational weight-factor ratio
  on permuted events read them as the densities' scales; it now reads the
  first call's `SCLOUT`, as D2 did. On the mixed row's 68 events it now reads
  MadEvent over this crate 1.022 on average, range 0.613–1.731, the inverse
  of D2's 0.58–1.63 (M2's printout had 0.978, 0.675–1.405).

**Where this section's plan was short of the source.**
- §1.4 "status-2 resonances appear only if the clustering found them on their
  Breit–Wigner": the test is on the *integration channel's* propagators
  (`checkbw` over `this_config`'s forest, `cluster.f:386-432`, re-run by each
  `cluster` call), keyed by leg set and applied to the clustered
  configuration's timelike lines; the clustered configuration contributes only
  the line list and codes. `isbw` entries are cleared only for the channel's
  own leg sets, so a clustered configuration with a timelike leg set the
  channel lacks reads a stale flag; unreachable on these rows (the lepton pair
  is a line of every configuration).
- `is_LC` (`addmothers.f:129-131, 195`): no resonance is written when the
  drawn flow is below leading colour in the configuration.
- The `ptclus` rule reads `pt2ijcl` after every rewrite, demotes a daughter's
  `goodjet` on a non-jet `ipart` leg, and treats the terminal vertex's first
  beam line and leftover line as its daughters; `etot` is `genps.f`'s
  `√stot` with proton masses.
- §1.4's `<MGRunCard>` "holding the resolved run card": MadGraph's record is
  the card after `banner.py` only, and it is CDATA, which the pinned Pythia
  cannot read (above).

**What remains for M5.** Showering both files through one Pythia
configuration with seed sweeps: acceptance per `@N`, merged σ, the
differential jet rates. The `qCut` choice matters there (`setMad` gives
`xqcut`, MadGraph `1.5 xqcut`). The mixed row's σ reference is regenerated
at close-out (D2).

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

#### M5 Landed, 2026-09-29

MadEvent's and vibegraph's matched `pp_to_ll_0j2j_mlm` samples go through
one Pythia configuration, MadGraph's own. The two agree on the matching
acceptance of every `@N`, on the merged σ and on the jet-rate shapes, within
statistics of 50000 events a side. With `<scales>` removed, the vibegraph
file fails every one of those comparisons by far. No production code
changed.

**The driver** (`validation/pythia/mlm_match.py` + `mlm_match.cc`, pixi
task `validate-mlm-pythia` in the `pythia` environment; the vibegraph samples
come from `generate_mlm_samples.sh`, task `generate-mlm-pythia-samples`):
- The C++ driver reads a Pythia command file, as main164 does. It runs
  main164's hook, `JetMatchingMadgraph` (what `CombineMatchingInput` picks
  for an LHEF at `scheme = 1`), subclassed only to record each event's
  IDPRUP, the process-level veto, the MLM veto, whether `next()` returned
  the event, and `getDJR()`.
- The Python side takes two sets of files and one configuration. It builds
  the driver against the environment's Pythia, showers every file with the
  same seeds, and writes JSON: acceptance per `@N`, merged σ, the three
  jet-rate histograms and their comparison.
- **Statistics.** The Les Houches event is the unit. Each event's outcome is
  averaged over the seeds, the errors are the spread over events (delta method
  for the normalised shapes), and 200 resamplings re-derive the bin errors. The
  per-file values and their χ²/dof about the side's mean test whether events
  are independent.
- **Weights.** Every event counts with its XWGTUP, as Pythia weights it at
  IDWTUP = −4. MadEvent's weights are all equal. vibegraph keeps the events
  its unweighting found above `w_max` at their own weight: 3.3–6.9 % of σ on
  these samples, largest `w/w_max` 119. Counting events instead would move
  vibegraph's overall acceptance from 0.6431 to 0.6513: overweight events are
  vetoed more often than the rest.
- A run whose command file is unchanged, with the input's and the driver's
  sha256 in it, is read back rather than re-run.

**MadGraph's own Pythia settings** (pinned tree, 3.7.1). `do_pythia8` runs
Pythia's own `main164` unless `--old_interface` is given
(`madevent_interface.py:4600-4655`). The command file is
`setup_Pythia8RunAndCard` applied to `Template/LO/Cards/pythia8_card_default.dat`:

| setting | value | source |
|---|---|---|
| `Beams:frameType` | 4 | `banner.py:1925` (PY8Card, always written) |
| `Check:epTolErr` | 1e-2 | `banner.py:1931` |
| `JetMatching:etaJetMax` | 1000 | `banner.py:1936`, always written (Pythia's default is 2.5) |
| `JetMatching:setMad` | off | `madevent_interface.py:4398` |
| `JetMatching:qCut` | 1.5·xqcut = 30 | `:4408-4409`, when the card leaves −1 (the default card does) |
| `Beams:setProductionScalesFromLHEF` | on | `:4419` |
| `JetMatching:merge`, `scheme` | on, 1 | `:4456-4457` |
| `JetMatching:nQmatch` | maxjetflavor = 4 | `:4460` |
| `JetMatching:coneRadius` | 1.0 | `:4462` |
| `JetMatching:nJetMax` | max_n_matched_jets = 2 | `:4467-4471`, `export_v4.py:5010-5024` |
| `JetMatching:doShowerKt` | off | `pythia8_card_default.dat` |

Notes on the settings:
- Everything else is at Pythia's defaults: the Monash tune, MPI and
  hadronisation on, the internal PDF, `doVeto = on`. `doVeto` is switched off
  only for the old interface with `use_syst`, `:4452-4455`.
- The jet algorithm is not a setting. `JetMatchingMadgraph::initAfterBeams`
  forces the kT `SlowJet` whatever `jetAlgorithm` says.
- The matching counts a light parton only if its production scale is below
  1.999·√(E_A E_B) (`sortIncomingProcess`). That is the path by which `<scales>`
  enters, beside each parton's shower start.
- `nJetMax` is taken as the largest number of light partons in any event,
  which equals MadGraph's rule on this card.
- The hadron level runs, but it cannot change these observables: the veto and
  the DJRs are decided in `doVetoPartonLevelEarly`.

**Samples.**
- **MadEvent:** the samples-grade `run_01` (iseed 20260928), plus four
  directories freshly generated from M0's proc lines and the committed run card
  (seeds 20261101–04, `nb_core = 2`, `vector_size = 1` checked). Each run is
  10000 events, and σ_LHE runs from 1062.30 to 1065.36 pb.
- **vibegraph:** the sprint tip `fe5d039`, release-debug binary
  `0c8c9362…`. Seed 20260928 was integrated here (`--fixed-budget --allocate
  neyman --neval 200000 --niter 8`), and its `grid.bin.zst` is byte-identical
  to M6's `mix-ney-n-28` (binary `96b787fc…`).
- On that evidence, M6's artifacts for seeds 20260929–32 were reused. Their
  integrations read 1065.23, 1067.24, 1063.95 and 1066.21 pb, and 1064.89 pb
  for seed 28.
- Each sample is 10000 events from `generate`, with `--seed` equal to the
  integration seed. Their sample σ (mean XWGTUP) is 1088.34, 1060.45, 1065.50,
  1073.48 and 1086.51 pb. That is −0.45 % to +2.20 % against their
  integrations, not the uniform +2.0–2.4 % M6 recorded.
- **Pythia:** 8.312, with seeds 20261201–06 on every file.

**Results** (six seeds, 5 × 10000 events a side; A = MadEvent, B = vibegraph):

| | MadEvent | vibegraph | B − A | pull |
|---|---|---|---|---|
| acceptance `@0` | 0.8193 ± 0.0009 | 0.8188 ± 0.0009 | −0.06 % | −0.35 |
| acceptance `@1` | 0.3636 ± 0.0025 | 0.3592 ± 0.0026 | −1.2 % | −1.23 |
| acceptance `@2` | 0.3478 ± 0.0040 | 0.3408 ± 0.0068 | −2.0 % | −0.89 |
| acceptance, all | 0.6468 ± 0.0014 | 0.6422 ± 0.0021 | −0.71 % | −1.80 |
| merged σ (pb) | 688.20 ± 1.48 | 690.30 ± 2.28 | +0.30 % | +0.77 |
| merged σ / σ_LHE | 0.6468 ± 0.0014 | 0.6422 ± 0.0021 | −0.71 % | −1.81 |

- The merged σ follows main164: the sum of the accepted events' XWGTUP over
  the file's number of events, averaged over files.
- MadEvent's files scatter as their errors say: the merged-σ χ²/dof over files
  is 1.36 and the `@N` acceptances 1.13 / 0.25 / 0.11.
- vibegraph's do not: merged σ 4.67, and `@2` acceptance 3.06, the file with
  the `w/w_max = 119` event reading 0.289. With the file spread as the error
  (MadEvent sd 3.8 pb, vibegraph sd 9.2 pb, over √5), the merged σ is
  +2.1 ± 4.5 pb. The normalised value is −0.0046 ± 0.0037, or −1.25σ.

| jet rate | χ² / dof (delta) | χ² (bootstrap) | p | seeds 1–3 | seeds 4–6 |
|---|---|---|---|---|---|
| log10 d01 | 15.1 / 24 | 14.2 | 0.92 | 8.9 | 18.3 |
| log10 d12 | 33.6 / 21 | 34.6 | 0.040 | 26.9 | 30.1 |
| log10 d23 | 12.9 / 17 | 13.8 | 0.74 | 19.5 | 12.2 |

- **Binning.** 0.1-wide bins in log10(d/GeV) on [0, 3], plus under- and
  overflow; the underflow also holds events with fewer clustering steps. Bins
  are merged from the left until each holds 100 accepted events on both sides.
- The bootstrap errors are 0.87–1.17 times the delta-method ones in every bin.
- The d01 χ² of 8.9 on seeds 1–3 (p = 0.998) was a shower fluctuation, since
  seeds 4–6 read 18.3.
- d12 shows a structured 2–3 % difference: vibegraph is low at d12 = 1.6–3 GeV
  (bins −2.7 and −2.0σ) and high at 4–10 GeV (up to +1.9σ). It stands at
  p = 0.04 among three histograms and is not resolved here.
- Plots: scratchpad `mlm-m5-djr-*.png`.

**Null test** (MadEvent against MadEvent: `run_01` and seeds 01–02 against
03–04; Pythia seeds 1–3):
- acceptance pulls +1.46, +0.43, +0.42;
- all +2.06, which is the directories' `@N` composition;
- jet-rate χ² 16.1/21, 22.4/19, 26.0/16.

The error model reads at its scale.

**Negative controls.**
- **`<scales>` removed** (every `<scales …>` line deleted from the vibegraph
  files, so each parton's scale is SCALUP):
  - `@1` goes to 0.4914 (+25σ against MadEvent), `@2` to 0.4216 (+8.1σ) and
    `@0` to 0.8171;
  - the merged σ moves +6.9 %;
  - the jet-rate χ² goes to 937/24, 494/21 and 348/17, with a step in d01 at
    log10 qCut.

  The comparison sees `<scales>` at more than ten times its resolution.
- **qCut 45 on both sides:**
  - MadEvent's acceptances go to 0.907 / 0.254 / 0.210;
  - vibegraph follows them (pulls +0.90, +1.30, −0.64; χ² 24.0/24, 21.3/21,
    16.1/18).

  The observables move with the matching scale, and the agreement holds there
  too.

**Secondary: `setMad = on`** (no qCut, nQmatch or merge line; Pythia takes
them from `<MGRunCard>`):
- **The CDATA-stripped MadEvent files** (14084-byte card) against vibegraph's
  (3079 bytes). Both give qCut 20 (= xqcut, not MadGraph's 1.5·xqcut),
  nQmatch 4, clFact 1:
  - acceptance 0.6915 / 0.6927, 0.4137 / 0.4101, 0.4802 / 0.4761 (pulls
    +0.54, −0.88, −0.45);
  - χ² 22.1/24, 19.2/20, 11.5/17.

  Both headers drive Pythia identically.
- **MadEvent's file as written:**
  - Pythia reads a 4-byte card, leaves `merge` at its default (off), and
    matches nothing: 10000/10000 events accepted, and no MLM veto is reached.
  - M4 saw qCut 10 and nQmatch 5 there only because its driver set
    `merge = on` explicitly.
  - The driver now stops on such a run, naming the settings Pythia read back.

**Where this section and M4 were short.**
- §1.4's shower-side list:
  - MadGraph 3.7.1 runs Pythia's `main164`, not the MG5aMC_PY8_interface
    (that is `--old_interface`).
  - It always writes `JetMatching:etaJetMax = 1000` and
    `Check:epTolErr = 1e-2`.
  - `jetAlgorithm` is inert.
  - The exclusion cut is 1.999·√(E_A E_B) in 8.312's `JetMatching.h`.
- The qCut rule is `1.5·xqcut` when the card leaves −1. The brief's
  "max(1.5·xqcut, xqcut + 10)" is not in the 3.7.1 source.
- M4's matching smoke ran at Pythia's default `etaJetMax = 2.5`, not
  MadGraph's 1000. Its acceptances ("LHE events for 1500 kept") are not
  MadGraph's configuration.
- "Merged σ = σ_LHE × acceptance" holds for an equal-weight file only. At
  IDWTUP = −4 main164 sums XWGTUP, which vibegraph's overweights make
  different from counting.

**What remains.**
- The row stays `info`. A gate would want:
  - more than five vibegraph samples, or samples whose overweights are
    tamed, because the heavy weights make vibegraph's file scatter three to
    five times its per-event error;
  - a look at the d12 shape at the 2 % level.
- The MadEvent side is regenerable from the four fresh seeds, but only
  `run_01` is banked. The task's default side A is `run_01` alone, and a
  multi-file comparison passes the fresh files explicitly (`--a`).

#### F-A Landed: the weight tail, 2026-09-30

The question was M5's and M6's two findings on `pp_to_ll_0j2j_mlm`: overweight
events carrying 3–7 % of σ with files that scatter 2–5× their per-event
errors, and a `--target-rel 2e-3` run that never stops. They share a root, and
it is not the w_max estimate. **No production code changed.** Two policies are
proposed below with offline measurements; neither is implemented.

All probes ran on the sprint tip `ef660f3` with env-gated instrumentation that
was never committed to the result (scratchpad `mlm-fa/probe-instrumentation.patch`).
Without the switches the probe binary reproduces the base bit for bit: M5's
seed-20260928 sample (1088.339837 pb, 311639 trials), M6's target run through
its first seventeen iterations, and M6's `pp_to_llj_mlm` seed-20260951 artifact
(269.473227 ± 0.458074 pb).

**Localisation** (M5's seed-20260928 artifact and sample, every trial logged
with its channel). The excess above w_max, `Σ(r − 1)` over the trials, is
2.65 % of σ:

| part | share of σ | excess / own σ | excess / total σ | share of Σr² | scan draws per channel |
|---|---|---|---|---|---|
| `@0` (4 channels) | 0.623 | 0.9 % | 0.55 % | 0.28 | 36–42k |
| `@1` (24) | 0.253 | 4.6 % | 1.17 % | 0.52 | 0.8–3.9k |
| `@2` (336) | 0.124 | 7.4 % | 0.92 % | 0.20 | 0.9–1.8k |

- `@0` sits at the 1 % the truncation rule asks for. The floor-bound `@1` and
  `@2` channels do not.
- Two `@1` channels, (group 1, channel 1) and (group 5, channel 3), carry 45 %
  of the sample's Σr² between them, with 12–15 % of their own σ above w_max.
- The target run's stop is held by `@2`. At iteration 5 one two-jet channel,
  (group 1, channel 11), whose largest point carried 71 % of one iteration's
  integral, has χ²/dof 20 and holds 49 % of the scaled variance the stop reads.

**Root cause: the adapted VEGAS grids make the tail; the maps do not.** On
the same artifact, 30000 scan points per channel on nine channels, with the
banked grid and with a flat one over the same map:

| | banked grid | flat grid |
|---|---|---|
| `@0` channels: per-point relative variance | 0.8–1.1 | 158–178 |
| `@0`: Hill tail index (top 20 of ~25k) | 1.3–1.7 | 5.3–6.5 |
| `@1` channels: per-point relative variance | 3.9–14.7 | 23.0–23.6 |
| `@1`: Hill index | 0.9–1.9 | 5.8–9.3 |
| `@1`: peak / mean | 118–433 | 77–87 |

Adaptation buys the bulk (1.6–200× in per-point variance) and pays with a tail
of index ≤ 2, where the variance barely exists. The top points show where the
tail sits. On every channel dumped, the heaviest weights sit in single wide
bins of the first two coordinates:
- `@0`'s second coordinate has its edge bins 17× wider than average: a large
  lepton-pair rapidity, where `etal = 2.5` keeps a narrow slice of decay
  angles. The grid adapts on `Σ(f·w)²` per bin, so a bin that mostly fails the
  cuts is starved, and the points that pass in it carry the width.
- `@1` shows the same on its first two coordinates: single bins 10–40× and
  13–19× wider than average.

MadEvent's grid counters exactly this: it adapts on `Σ|w|` per bin
(`dsample.f:1890`) and rescales each bin by `non_zero / inon_zero`, the
inverse of the bin's own acceptance, capped at 10⁴ (`dsample.f:2106-2124`).
Adding that rescale to `adapt_blocks_iteration`'s histogram (probe only) moved
`pp_to_llj_mlm`'s median Hill index from 1.8 to 2.2 (worst channel 1.0–1.2 to
1.5–1.6; two seeds each, 150k × 10). It helps and does not cure; the rest
of the tail is not located.

**The w_max estimate is not the lever.** The truncation ladder cannot step
off the top of a scan whose largest weight alone exceeds 1 % of its sum, which
under a tail index near 2 is any scan below ~10⁴ points. The floor-bound
channels scan 0.8–3.9k, so their rule is the extremum. Raising every channel's
scan to at least 8000 points (seed 20260928):

| | share scan (M5) | ≥ 8000 per channel |
|---|---|---|
| scan points | 0.53M | 3.0M |
| Σ w_max | 3.30e4 | 7.61e4 |
| efficiency | 3.21 % | 1.40 % |
| excess above w_max | 2.65 % | 2.34 % |
| `@1` / `@2` excess of own σ | 4.6 / 7.4 % | 2.3 / 8.8 % |
| largest w/w_max | 36.5 | 65.8 |
| CPU | 677 s | 5572 s |

Eight times the CPU buys 0.3 % of σ and halves the efficiency. Over a tail of
index ≤ 2 the maxima do not settle (`ScanBudget`'s own finding on
`pp_to_llj`), so no scan floor is proposed.

**What drives the file-to-file scatter.** The sample σ a file declares is
`Σ w_max · Σ(event weights) / trials`, the sample's own estimate. At 10000
events and 3 % efficiency its binomial error alone is 0.97 %, and 1.11 % with
the overweights (seed 20260928's trials). The five samples' deviations from
their integrations (+2.20, −0.45, −0.16, +0.90, +1.90 %) are consistent with
that. `XERRUP` is still the integration's error (±0.16 %), so the file
understates its own σ's error about 7×. MadEvent's `@N` σ shares scatter at
0.03–0.10 of their binomial variance, because MadEvent pins every channel to
its integral; ours scatter at 3.2 (`@0`) and 7.1 (`@2`) times it.

**MadEvent's behaviour** (pinned tree):
- Per channel, points above `twgt·fudge·ran` are stored weighted
  (`unwgt.f:239-266`). `store_events` takes `target_wgt` from the `trunc_max`
  ladder (`unwgt.f:355-372`), keeps overweights at their own weight
  (`unwgt.f:397-446`), and rescales the channel's events so they sum to its
  integrated |σ| (`xscale = xsecabs/xsum`, `unwgt.f:409-411`).
- `combine_events` scales each channel's file again to its `axsec`
  (`lhe_parser.py:1154,1159`), picks `max_wgt` for `event_target = nevents` at
  `trunc_error = 1e-2` (`lhe_parser.py:533-545`,
  `madevent_interface.py:3883,3922`), and writes every kept event at one
  weight, `sum(across)` under `event_norm = average`
  (`lhe_parser.py:1224-1239,571`). Overweights are **truncated** to that
  weight; the loss is logged as `trunc_cross`.
- So the file's σ is the integration's by construction, and the shape carries
  a ≤ ~1 % truncation bias.
- Iterations: the last three, weighted by `xmean²/xsigma²`
  (`dsample.f:296-311`), χ² over those three (`dsample.f:316-320`), error ×√χ²
  when χ² > 1 (`dsample.f:332`). A refine stops on enough events and χ² < 10
  (`dsample.f:376`). Zooming is off (`dsample.f:873-875`).

**Proposal 1 (policy): normalise each `@N` to its integration.** Rescale the
weights of each multiplicity's events so they sum to that part's integrated σ,
and declare the integration's σ and error. The expectation is unchanged; the
bias is that of a ratio estimator, O(1/n) over ≥ 1200 events per part. This is
MadEvent's per-channel normalisation, taken at the multiplicity level so that
no part is normalised over a handful of events. Replayed offline on M5's five
showered files (same Pythia outcomes, weights rescaled; errors are event spread
⊕ integration error; pixi `pythia` env, `mlm-fa/policy_replay.py`):

| policy | merged σ (pb) | χ²/dof over files | `@0` / `@1` / `@2` acceptance χ²/dof |
|---|---|---|---|
| as written | 689.98 | 5.82 | 1.08 / 0.57 / 2.49 |
| file normalised to the integration | 685.00 | 3.10 | unchanged |
| each `@N` normalised | 686.56 | 0.87 | unchanged |
| + overweights capped at unit weight (MadEvent) | 689.38 | 0.57 | 0.72 / 1.10 / 1.86 |

MadEvent: 688.20 ± 1.48, χ²/dof 1.36. Per-`@N` normalisation removes the
merged-σ excess without bias. The `@2` acceptance excess is the overweights
themselves (the w/w_max = 119 event), which only truncation, a biased policy,
or a lighter tail removes. The `cli_generate_proton` sample-vs-integration
check would then read the logged sample σ, not `XSECUP`.

**Proposal 2 (policy): the stop's consistency factor for an unweighted mean.**
A target run combines iterations unweighted, but `stop_scale` forms χ² from
each iteration's own σᵢ. One spike iteration then puts the quiet iterations
tens of σ from the mean, and the channel's factor decays only as 1/(n − 1).
The unweighted mean's own check is the between-iteration scatter against its
quoted variance: `max(1, emp/quoted)`, with `emp = Σnᵢ²(Iᵢ − Ī)²/W² · n/(n−1)`
and `quoted = Σnᵢ²σᵢ²/W²`. Replayed on the seed-20260928 target run's
per-channel histories (`mlm-fa/stop_replay.py`; the replay reproduces the run's
quoted errors):

| iteration | quoted | current factor | pooled factor | holding the current stop |
|---|---|---|---|---|
| 4 | 3.10e-3 | 1.10e-2 | 4.30e-3 | (g1, c11) of `@2`, χ²/dof 48 |
| 6 | 2.16e-3 | 4.64e-3 | 2.66e-3 | the same, 25 |
| 8 | 1.63e-3 | 2.71e-3 | **1.86e-3** | the same, 13 |
| 12 | 1.62e-3 | 4.72e-3 | 1.76e-3 | (g4, c11) of `@2`, 36 |
| 15 | 1.37e-3 | 2.41e-3 | 1.50e-3 | the same, 11 |
| 16 | 1.37e-3 | 2.45e-2 | 1.47e-3 | (g23, c0) of `@2`, 5675 |
| 17 | 1.35e-3 | 1.78e-2 | 1.47e-3 | the same, 4012 |

- Iteration 16's jump in the run's printed χ²/dof (2.48 → 18.08, the mean
  over 364 channels) is that one channel. It carries 0.004 % of σ; one point
  in one iteration put its quiet iterations about 75σ from its mean.
- Every channel that holds the current stop is a floor-bound two-jet channel
  with a single-point spike. Under the pooled factor no channel exceeds 1.8
  after iteration 5.

Under the pooled factor the run stops at iteration 8, the earliest allowed
past the six-iteration minimum where the pooled error is ≤ 2e-3, with
1064.89 ± 1.73 pb: M6's fixed-budget artifact, since the Neyman draws agree.
The five-seed fixed sweep is 1065.50 ± 0.89 (sd 1.26, χ²/dof 0.42), so the
quoted errors are, if anything, conservative across seeds. They do not
support the 1.5–3× the current factor adds before a spike, let alone the
13–18× after one. The current rule never gets below 2.41e-3 in 17
iterations; M6 saw it still running at 32.

**Byte identity.** Nothing in production changed, so every integration and
sample is byte-identical to `ef660f3`.

**What remains.**
- The decisions on proposals 1 and 2.
- The grid tail itself: the acceptance rescale, adapting on `Σ|w|`, or a
  bound on bin width, each measured on the banked rows (every integration
  moves). This is the root of both findings.
- F-B's map merge gives the floor-bound channels 4–9× the points in both the
  integration and the scan. Re-measure the excess after it lands.

### F-B: channel merging (performance-dev; after M6)

#### F-B Landed, 2026-09-30

A hadronic run now builds one sampling channel per distinct phase-space map
instead of one per `(group, diagram)` pair. Pairs whose maps are the same
function share a channel. The channel carries their summed selection
weight, pays one coverage floor, and costs one density term in the mixture
sum. On `pp_to_ll_0j2j_mlm` the channels go from 364 to 43 (`@0` 4 → 1,
`@1` 24 → 6, `@2` 336 → 36).

**MadEvent's rule** (pinned tree; paths as in §1). A `P<n>` directory's
channels are its `configs.inc` configurations, one per `IdentifyConfigTag`
class across every subprocess of the directory
(`madgraph/iolibs/group_subprocs.py:315-385`). The tag is the topology with
each leg's number, spin, mass, width and colour, and each propagator's
colour, mass and width (`:56-103`). Colour is part of it, so a gluon and a
photon exchange are two configurations. A t-channel Z or H takes the photon's
mass and width, so it joins the photon's configuration.
`config_subproc_map.inc` holds `CONFSUB(IPROC, iconfig)`, the diagram of
subprocess `IPROC` on that configuration, or 0 where it has none
(`group_subprocs.py:387-404`, written by `export_v4.py:6670-6681`). A point
of channel `iconfig` is handled per subprocess like this:
- **Which subprocesses:** only those with `CONFSUB ≠ 0` enter the
  subprocess selection and the sum. A subprocess without that diagram
  contributes nothing in that channel (`super_auto_dsig_group_v4.inc:562,
  584, 629, 677`).
- **Its channel weight:** `AMP2(CONFSUB(IPROC, channel))` over the sum of
  its own mapped `AMP2` (`matrix_madevent_group_v4.inc:214-235`).
- **Its clustering:** the candidate graphs are the directory's, filtered to
  those the subprocess has (`id_cl` filled only where `confsub(iproc, ignum)
  ≠ 0`, `cluster.f:265-281`). `chcluster` restricts the clustering to
  `iconfig` itself (`cluster.f:466-470`), and the result is `igraphs(1)`.
- **Its jet memo:** `njetstore(iconfig)`, one entry per directory
  configuration, shared by every subprocess of the directory
  (`reweight.f:588-592, 663, 985-1030`).
- **Its colour:** `select_color(…, iconfig, IPROC, …)`
  (`matrix_madevent_group_v4.inc:241`), which takes `igraphs(1)` under
  `ickkw > 0` (`super_auto_dsig_group_v4.inc:1120-1142`).

A second sharing layer, the symmetric configurations (`SYMCONF`, `PERMS`,
`:799-805`), integrates permutation-related configurations once and permutes
the point. vibegraph has no counterpart; it is not part of this change.

**Why vibegraph needs no configuration rule.** vibegraph's channels do not
carry MadEvent's channel decomposition. The estimator is the one-sample
mixture `f/g` with `g = Σⱼ αⱼ gⱼ`. At every point each group draws its
clustering configuration `∝ AMP2` from its own matrix element
(`per_group_sum`, note 29 chain B). That draw is the conditional expectation
of MadEvent's channel sum, and it never reads the sampling channel. The same
configuration feeds the jet memo, which is the configuration's restricted jet
count (M1's proof), the colour and the record. So a merged channel hands
every group exactly what an unmerged one did, and so does MadEvent's shared
channel in expectation. The one place the sampling channel still reaches a
term is the draw's fallback, where a group's `AMP2` carries no probability
and so its term carries none. There the group keeps its own diagram of the
merged channel, or its first configuration if the channel has none of its
diagrams. Without a merge that is the old rule exactly (`proton.rs:2159`).

**The merge key is identity of the function.**
- `DiagramChannel::map_identity` (`phasespace/diagram_channel.rs:1053`)
  covers every field the draw and the density read: `√s`, `n_out`, beam
  masses, the tree or spine with every node's slot, mass, `μ`, mask, shape
  fingerprint, floor, resonance, forced window, angular map with its energy
  floors, each rung's pole, transfer cap and remainder floor, and the
  t-channel lines. Each float is taken by `integer_decode`, which is exact and
  tells −0 from +0.
- Each struct is destructured whole, so a new field that is not added to the
  key does not compile.
- Topology, masses, widths and cut floors are all in the key, as the brief
  asked. A key built from the *diagram* instead (MadGraph's tag) would be the
  wrong one here. It separates gluon and photon exchange, which have the same
  map, and it would put a t-channel Z with the photon, which does not have
  the same map.
- `pairs_with_equal_map_identities_draw_and_weigh_every_point_alike`
  (`proton.rs:4136`) measures the key. It runs every pair of channels of
  `p p > l+ l- j` (24 pairs of groups, 6 maps) and of the matched
  `p p > e+ e- j j` (336, 36):
  - two channels with equal identities draw bit-identical momenta and
    weights from 24 shared `(√ŝ, u)` draws, and give bit-identical densities
    at points drawn from every channel (44 and 2072 such pairs);
  - two channels with different identities differ in density at some of
    those points (232 and 54208 pairs);
  - the integrand's channels are the identity classes, in order of first
    appearance, with `αⱼ = pairsⱼ / N`.
- `a_merged_channel_is_the_sum_of_the_unmerged_terms_it_replaces`
  (`proton.rs:4235`) checks the value. On the matched llj card, a merged
  channel's term equals the sum of the unmerged terms it replaces
  (`new_unmerged_with_maps`) at the same uniforms. The worst relative
  difference is 8.3e-16 over 1214 nonzero terms: the two density sums group
  their terms differently, and nothing else differs.

**Implementation.**
- **Building the channels** (`proton.rs:1497-1591`):
  - `ProtonIntegrand::build` keys each pair's channel by its identity and
    appends a repeat to the first channel's `channel_members`.
  - The first-built pair names the channel (`channel_ids`).
  - When anything merged, the initial weights are `pairsⱼ / N`. That makes
    the starting mixture density the unmerged uniform one, and a
    Kleiss–Pittau step on it is the step on the pairs summed.
  - When nothing merged, no line of the path changes.
- **Keys** (`multiplicity.rs:220-252`): a channel of several pairs is
  `ChannelKey::MergedChannel { final_state, group, channel, pairs }`
  (`artifact.rs:172`), and every other channel keeps its old key.
- **Format 11** (`artifact.rs:97-106`): `version_for` records the oldest
  version that holds the keys (`:693`). An artifact with no merged channel is
  still version 9 or 10, byte for byte.
- **Refusing stale artifacts:** `IntegrateArtifact::refuse_unmerged_grids`
  (`artifact.rs:709`) refuses an artifact older than version 11 on a process
  whose channels merge, and names both versions and the pair count.
  `generate` calls it before its key check (`vibegraph-cli/src/generate.rs:1357`,
  a three-line edit in the sibling session's file). An older build refuses a
  version-11 file by its version.
- **Unchanged:** the grid dimension stays per channel, as M3 built it. The
  fixed-beam integrand is untouched.

**Byte identity: M1's six cases.** Base binary `ef660f3` against this
change, `integrate` 20k × 4 (seed 7) and `generate` 500 events
(`RAYON_NUM_THREADS=2`). The base grids reproduce M4's base sha256 prefixes.

| case | channels (pairs → maps) | grid | LHE |
|---|---|---|---|
| `llj_fixed` (`p p > l+ l- j`, fixed scale) | 24 → 6 | differs (merges) | differs |
| `llj` (`pp_to_llj` card) | 24 → 6 | differs (merges) | differs |
| `jj` | 15 → 3 | differs (merges) | differs |
| `dy` (`p p > e+ e-`) | 4 → 1 | differs (merges) | differs |
| fixed-beam `g u > e+ e- u` | 4 (fixed-beam path) | identical `6bebef12…` | identical but the artifact-path line |
| fixed-beam `e+ e- > mu+ mu-` | 2 (fixed-beam path) | identical `d640093a…` | identical but the artifact-path line |

Every hadronic case of M1's list has flavour groups that share maps, so none
can stay byte-identical. On each, σ is instead shown by a five-seed sweep at
`--fixed-budget --neval 100000 --niter 8`, seeds 20261001–05, base and new
interleaved. The spreads are over the seeds; the brackets give sd, χ²/dof
and the mean quoted error.

| case | base σ (pb) | new σ (pb) | new − base | rel²·CPU base → new (quoted) |
|---|---|---|---|---|
| `llj_fixed` | 424.29 ± 0.45 (0.99, 1.25, 0.96) | 424.59 ± 0.37 (0.83, 1.14, 0.78) | +0.5σ | 1.7× less |
| `llj` | 505.66 ± 0.46 (1.04, 0.35, 1.77) | 506.64 ± 0.43 (0.95, 0.54, 1.30) | +1.5σ | 2.1× less |
| `jj` (×10⁸) | 6.8014 ± 0.0011 (0.0025, 0.08, 0.0089) | 6.7938 ± 0.0047 (0.0105, 1.46, 0.0087) | −1.6σ | 1.08× less |
| `dy` | 934.54 ± 0.43 (0.95, 0.47, 1.37) | 934.94 ± 0.37 (0.82, 0.39, 1.32) | +0.7σ | 1.05× less |

Every merging row of the banked hadronic gate passes (below). The fixed-beam
path is unchanged.

**Measurements.** Base is `ef660f3` (`mlm-fb-vibegraph-base`, sha256
`2743820e…`). Its seed-20260928 run of the mixed row reproduces M6's
fixed-Neyman `mix-ney-n-28` bit for bit (σ, every iteration). M6's seeds
20260929–32 are therefore this binary's too, and are reused as the base
sweep. The container was shared with the sibling session throughout (load
7–12 on 4 cores). CPU seconds are recorded but carry that noise; point
counts and errors do not.

`pp_to_ll_0j2j_mlm`, `--fixed-budget --allocate neyman --neval 200000
--niter 8`:

| | base (5 seeds, 28–32) | new (10 seeds, 28–37) |
|---|---|---|
| channels `@0`/`@1`/`@2` | 4 / 24 / 336 | 1 / 6 / 36 |
| iteration 2, α split before → after the acceptance-raised floors | 363,660 → 806,744 | 211,717 → 254,738 |
| last-iteration points | 528,060 | ~227,500 |
| evaluations over 8 iterations | 4.52M | 1.85M |
| CPU per run | 3,117 s (M6, quieter host); 5,412 s (seed 28 here, loaded) | 881 s (loaded) |
| CPU per point | 0.69 ms (M6); 1.20 ms (seed 28 here) | 0.48 ms |
| `@0` | 665.23 ± 0.29 (sd 0.65, χ²/dof 0.87, quoted 0.70) | 665.26 ± 0.18 (0.56, 0.66, 0.69) |
| `@1` | 268.22 ± 0.61 (1.36, 0.75, 1.60) | 268.59 ± 0.41 (1.28, 0.90, 1.31) |
| `@2` | 132.06 ± 0.23 (0.51, 0.32, 0.93) | 131.26 ± 0.66 (2.07, 1.89, 1.46) |
| total | 1065.50 ± 0.56 (1.26, 0.41, 1.99) | 1065.11 ± 0.61 (1.93, 1.02, 2.12) |
| rel²·CPU, quoted error | 1.10e-2 s | 3.64e-3 s (3.0× less) |
| rel²·CPU, seed spread | 4.37e-3 s | 2.89e-3 s (1.5× less) |
| ε_unw (5000 events at `--seed 20260731`, seeds 28–32) | 2.06 % (1.69–2.46) | 3.40 % (1.86–5.44), 1.65× higher |

Readings:
- **σ per `@N` does not move** beyond the spread: `@0` +0.03, `@1` +0.37
  (+0.5σ), `@2` −0.80 ± 0.70 (−1.1σ), total −0.39 ± 0.83.
- **The floor spend falls about 10×.** The acceptance-raised floors added
  443k points to an iteration before, and add 43k now. At fixed `--neval` a
  run draws 2.4× fewer points.
- **A point is cheaper.** The mixture sums 43 densities instead of 364.
  Measured under different load, the CPU per point falls 1.4× against M6's
  quiet base and 2.6× against the loaded base run here. The per-group
  reclustering is unchanged and now dominates.
- **At the same `--neval`, `@2` is the weak part.** Before, the floors
  bought `@2` ~600k points an iteration against the 13k its budget share
  asked for. That over-sampled its tail, and its χ²/dof was 0.32. Now `@2`
  gets what the part split and Neyman give it, both computed from quoted
  spreads. Its heavy tail makes its quoted error an underestimate (χ²/dof
  1.89 over ten seeds), and its seed spread is 4× base's at 3.5× less CPU.
  Measured by the seed spread, the total gains 1.5× in variance × time at
  this `--neval`, not the 3× the quoted errors give.
- **At the same number of points, every part is better.** Three seeds
  (28–30) at `--neval 600000`:
  - 4.99M evaluations against base's 4.52M, in 1,432 s of CPU against
    base's 3,117 s;
  - the floors add 32k points an iteration, not 443k;
  - σ: `@0` 664.97 ± 0.16 (quoted 0.39), `@1` 268.38 ± 0.30 (0.58), `@2`
    132.08 ± 0.37 (sd 0.64, χ²/dof 0.77, quoted 0.73), total 1065.42 ± 0.74
    (quoted 1.01, χ²/dof 1.55);
  - the total's quoted error halves (1.99 → 1.01 pb) and `@2`'s falls
    (0.93 → 0.73). By quoted error, rel²·CPU is 1.28e-3 s, 8.6× below base.
    Three seeds cannot calibrate a spread; they read 1.28 pb on the total
    against base's 1.26.
- **So the merge is a budget knob, not a loss.** With the redundant floors
  gone, `--neval` sets how many points `@2` gets. A run at the base's point
  count gets base's `@2` precision and better `@0`/`@1` in half the CPU. A
  run at the base's `--neval` spends 2.4× fewer points and under-samples
  `@2`'s tail. Pricing that tail in the allocation (the part split's `sₖ`
  and Neyman) is the lever that would remove the need to over-ask. This
  change does not touch the allocation.

`pp_to_llj_mlm`, `--fixed-budget --neval 150000 --niter 10` by α, seeds
20260951–60, base and new interleaved:

| | base | new |
|---|---|---|
| channels | 24 | 6 |
| iteration points after the floors | 150.5k → 157.1k | 150.0k (no floor binds) |
| σ (pb) | 268.45 ± 0.24 (sd 0.76, χ²/dof 3.06, quoted 0.455) | 268.53 ± 0.10 (sd 0.33, χ²/dof 0.80, quoted 0.380) |
| CPU per run | 249 s | 233 s |
| rel²·CPU, quoted / seed spread | 7.4e-4 / 2.0e-3 s | 4.7e-4 / 3.5e-4 s (1.6× / 5.7× less) |
| ε_unw (5000 events at `--seed 20260731`, seeds 51–55) | 1.88 % (0.84–3.08) | 4.27 % (3.66–5.09), 2.3× higher |

- The merged grids converge consistently. The row's χ²/dof, which M2 and M6
  recorded at 2.4–3.0, is 0.80 over ten seeds, and the banked
  `validate-mlm-sigma` row reads 0.86 (below).
- The seed spread is 2.3× smaller at 6 % less CPU.

**The sample against its integration** (`cli_generate_proton`, `llj_fixed`,
the five `probe_sample_sigma_seed_headroom` seeds): −1.34, +0.27, +0.18,
+0.83 and −0.65 %, mean −0.14 %, sd 0.84 %. Base (M6) read +0.36, −0.43,
+0.01, −0.26 and −0.40 %, sd 0.33 %. There is no bias, but the scatter is
2.5× wider. The gate's own seed sits at −1.34 % against its 1.5 % bound
(1.1× headroom). On the matched rows the same reading did not widen. Sample
against integration over seeds 28–32 of the mixed row: base +2.0, −0.9, −0.5,
+0.8, +1.7 %; new +1.9, −0.5, +0.4, +1.4, −0.1 %. On `pp_to_llj_mlm`, seeds
51–55: base −0.1, −1.0, +0.4, −2.0, −2.5 %; new −1.0, −0.0, −0.6, +0.4,
−1.8 %. The scatter is the unweighting's truncation, which the sibling
session owns. It should be read there before this gate's seed moves.

**Gates** (final tree; the banked tasks' own cargo commands, run through
`pixi run` with the private build settings):
- `cargo fmt --all --check`: clean. `cargo clippy --workspace --all-targets
  -- -D warnings`, plain and with `vibegraph/extended-validation,
  vibegraph-lib/extended-validation`: clean.
- `cargo test --workspace`: 33 binaries, 1291 passed, 0 failed, 15 ignored.
- `validate-mlm-dumps`: 3 passed. Every per-event field agrees as M1 and M4
  recorded, and every jet-memo channel has a single restricted count
  (8/8/8/29/82).
- `validate-scales`: 10 passed. `validate-unweighting`: 1 passed.
  `validate_samples`: 6 passed. `validate_samples_proton`: 11 passed, min
  KS p 6.7e-2.
- `validate-generate-proton`: 5 passed. The sample reads −1.342 % against the
  1.5 % bound (above).
- `validate-hadronic`: 14 passed. Pulls: `llj_fixed` +0.05, `llj_dyn` +0.07,
  `pp_to_llj` +0.61, `jj` +0.72, `default` +0.54, `mmll_60_120` +0.42,
  `pp_to_ll_scalefact2` −0.47, `pp_to_bb` +0.27, `pp_to_bb_qcd2` +0.28,
  `bb_fixed` +1.22. Against M6's run the quoted errors fell on the llj and
  `jj` rows (`llj_fixed` 0.290 → 0.250, `llj_dyn` 0.288 → 0.247, `pp_to_llj`
  0.642 → 0.527 pb, `jj` 4.43e5 → 4.15e5 pb). The Drell–Yan rows scatter
  about their base values (`default` 0.567 → 0.557, `mmll_60_120` 0.384 →
  0.444 pb).
- `validate-mlm-sigma` (info):
  - `pp_to_llj_xqcut_only` 212.608 ± 0.126 (χ²/dof 0.82, pull +0.23);
  - `pp_to_llj_mlm` 268.433 ± 0.119 (ten seeds, χ²/dof 0.86, pull +1.22;
    M6: 268.560 ± 0.145, χ²/dof 2.96);
  - `pp_to_llj_mlm_alps2` 241.255 ± 0.150 (χ²/dof 1.04, pull +1.86; M6:
    241.052 ± 0.176).

**Classification.** Statistical. The value at a point is unchanged (to the
rounding of the density sum), but the channels, their grids and their
streams change wherever a map is shared. Where nothing is shared, the run is
byte-identical.

**Where the plan was wrong.**
- "Which clustering configuration a merged channel hands each group" (M6,
  the brief) presumes the channel chooses one. vibegraph's configuration is
  drawn per point per group from `AMP2`, independently of the sampling
  channel, so the merge needed no rule beyond the fallback.
- M1's byte-identity cases were expected to include non-merging hadronic
  processes. All four merge: even `p p > e+ e-` has four groups on one map.
  Only the fixed-beam cases stay byte-identical.
- "Makes the floors about 9× cheaper" holds for the floor spend (10×). It
  did not say that the floors were also what gave `@2` its points. At the
  same `--neval` the gain measured by the seed spread is 1.5×. At the same
  point count it is 8.6× by quoted error (three seeds).

**What remains.**
- Price `@2`'s heavy tail in the allocation: the part split's `sₖ` and the
  Neyman re-split both read quoted spreads.
- Read the unweighting truncation of merged channels (the sample's scatter
  above) in the unweighting work.
- MadEvent's symmetric-configuration sharing (`SYMCONF`) has no counterpart.

#### P12 Landed: both F-A policies, 2026-10-01

F-A's two proposals, adopted by the user on 2026-10-01, are implemented: the
buffered writer normalises each `@N` to its integration, and the target stop
reads a pooled consistency factor. Fixed-budget integrations are unchanged
byte for byte. Event files change in their weights, `<init>` and header only.

**Policy 1: each part normalised to its integration** (`lhef/emit.rs`,
`Buffer`).
- The events of each part are scaled by one factor so that
  `Σ_{i∈k} XWGTUP / N = σₖ`. A part is a final-state multiplicity of a sum, or
  the whole run when there is one. `IDWTUP = −4` and the mean-weight
  convention are unchanged: "the part's weights sum to σₖ" is read over the
  file's `N` events. That is what MadGraph's `event_norm = average` and
  main164 expect at `IDWTUP = −4`, and what M5's merged σ sums.
- **Overweights keep their weight**, rescaled with their part, and nothing is
  truncated. The one bias is the ratio estimator's, `O(1/nₖ)`. This is stated
  on `Buffer`.
- **`<init>`.** Process `p` declares `XSECUP_p = Σₖ σₖ f_{pk}`, where `f_{pk}`
  is its share of part `k`'s generator weight, and
  `XERRUP_p = √(Σₖ (f_{pk}Δσₖ)² + σₖ² f_{pk}(1 − f_{pk})/nₖ)`. Where an `@N`
  is one multiplicity, these are exactly the part's integrated σ and error.
  `XMAXUP` is the largest weight written.
- **Header.** `sample estimate before normalisation <σ̂> +- <err>` is the
  sample's own `W·Σw/T`, with the error `σ̂·√(Σw²)/Σw`
  (`sample_estimate_error`). The error drops the `1/T` term, so it is high by
  `1/√(1 − ε)`, 1–2 % at these efficiencies. A sum adds one line per part with
  its pre-normalisation estimate and the σ it was normalised to.
  `sample_estimate_in` reads the line back.
- **Plumbing.** `WeightedEvent::part` and `EmitPlan::parts`. `generate` fills
  them from the artifact's per-channel σ over `MultiplicitySum::offsets`
  (`generate.rs`, `part_sigmas`). A single-multiplicity run passes no parts,
  and the whole file is normalised to `artifact.sigma_pb`.
- **`StochasticRounding` is unchanged.** Unit weights cannot carry a per-part
  scale, and its `XSECUP` was already the integration's. Its process split is
  still the sample's.

**The replacement sample check** (`cli_generate_proton`). Once the file is
normalised, `XSECUP` equals the integration by construction. The gate now
asserts three things:
- `mean XWGTUP = XSECUP` and `XSECUP ± XERRUP` equals the integration's, both
  to 1e-6. This is a structural check of the writer.
- The header's pre-normalisation `σ̂` against the integration, as a pull over
  the two errors in quadrature, below `SAMPLE_PULL_MAX = 3.5`. This is the
  part with teeth. It measures the accept/reject pass, which the normalisation
  hides from the file's σ. Its scale is the sample's binomial error (0.70 % at
  20000 events), not a fixed fraction. The 1.5 % relative bound sat 1.1× above
  the gate's own seed at −1.34 %. That seed now reads −1.342 %, pull −1.89.
  The five-seed probe (`probe_sample_sigma_seed_headroom`) reads pulls −1.89, +0.38, +0.25, +1.13 and −0.89, from the same relative distances F-B recorded (−1.34, +0.27, +0.18, +0.83, −0.65 %). `Σpull²/n` is 1.17 and the bound's headroom is 1.9×.
- The mixed-multiplicity case asserts per `@N`: `XSECUP` and `XERRUP` equal
  that multiplicity's integrated σ and error, and its written weights sum to
  it over `N` (@0 668.345100 against 668.345063 pb, @1 212.138900 against
  212.138873). It also asserts the pre-normalisation pull (−0.64). It keeps
  its count-share check of `@0` against the integration.

The fixed-beam `cli_generate` gate reads `σ̂` from the header in its existing
`4/√N` band. Its strategy-total comparison is now exact by construction, and
the code says so: the strategies are compared on shape.

**Policy 2: pooled consistency factor** (`budget.rs`, `pooled_scale`).
- The stop widens each channel's quoted variance by `max(1, emp/quoted)`:
  - `quoted = Σnᵢ²σᵢ²/W²`;
  - `emp = Σnᵢ²(Iᵢ − Ī)²/W² · k/(k − 1)` over the `k` kept iterations;
  - the widened variance is therefore `max(quoted, emp)`.
- Zero-variance iterations need no filter: the pooled factor never divides by
  one iteration's variance.
- One iteration gets factor 1. Two get `(I₁ − I₂)²/(σ₁² + σ₂²)`. On equal
  iterations with equal errors the factor equals the old χ²/dof (pinned by
  test). A target stop needs at least two kept iterations (`min_iters ≥
  warm-up + 2`), so a single iteration only ever reaches the final report.
- The factor is the check of an unweighted mean, the only combination a
  target run accepts. Under inverse-variance runs it is still formed for the
  report and decides nothing.
- **What changes:** only the stop decision, `ConvergenceReport::scaled_rel`,
  the progress projection, and the log line, now labelled
  "consistency-scaled". A fixed budget draws the same points and banks the
  same bits (below). A target run that stops at the same iteration as before
  gives an identical σ and artifact.

**Replay.** The Rust `scaled_rel` was fed F-A's per-channel histories from
the seed-20260928 target run (364 channels, `FA-BLK` lines of
`mlm-fa/tgt28-17iter.log`) by a temporary test that was not committed. It
reproduces `stop_replay.py`'s pooled column to every printed digit:
4.298e-3, 3.020e-3, 2.655e-3, 2.142e-3, **1.863e-3** at iteration 8,
1.879e-3, … 1.470e-3 at 17. The first iteration at or below 2e-3 is 8. The
quoted error there is 1.626e-3: 1064.89 ± 1.73 pb.

**Hermetic tests** (`cargo test --workspace`: 33 binaries, 1297 passed, 0
failed, 15 ignored).
- `each_part_is_normalised_to_its_integration`: two parts, a source σ̂ 17 %
  off. Each `@N`'s written weights sum over `N` to its σₖ (1e-6), `XSECUP` and
  `XERRUP` are exactly σₖ and Δσₖ, every weight keeps its ratio to its part's
  unit-weight events (overweights 3.5 and 2.0), and the header's σ̂ reads back.
- `processes_that_split_parts_declare_their_shares`,
  `an_undeclared_part_is_refused`.
- `the_sample_estimate_error_matches_its_own_spread`: 400 accept/reject
  replicas with a sixth of the events at weight 4. The spread is the quoted
  error × `√(1 − ε)` within 8 %. `σ̂/√N` would be 15 % low.
- `a_single_spike_holds_the_chi2_stop_and_not_the_pooled_one`: a synthetic
  two-channel history, the small channel with one point at ten times its
  integral in iteration 3. The old χ² rule (kept in the test as the
  comparison) never reaches 2e-3 in 40 iterations, and its factor on the
  spike channel exceeds 100. The pooled rule stops within 10, with the spike
  channel's factor below 2.
- `the_stop_scale_is_the_scatter_of_the_mean_over_its_quoted_variance`
  (7/3, equal to χ²/dof), `one_and_two_iteration_histories`,
  `unequal_iterations_are_weighted_by_their_point_counts`, and the reworked
  `a_zero_variance_iteration_cannot_blow_up_the_stop_scale`.
  `a_wide_split_with_empty_iterations_still_converges` passes unchanged.

**Byte identity: M1's six cases.** `integrate` 20k × 4 at seed 7 and
`generate` 500 at seed 7, base `d2b4b7b` built in its own worktree and target
(`mlm-p12-vibegraph-base`, sha256 `4525033b36a31b52…`), against this change
(`mlm-p12-vibegraph-new`, `21a37592ebc72336…`). LHE columns from
`mlm-p12-lhecmp.py`:

| case | grid (both) | events differing outside XWGTUP | XWGTUP new/base | `<init>` | header |
|---|---|---|---|---|---|
| `llj_fixed` | identical `f74a18d2f2329e9a` | 0 / 500 | 0.908956 (uniform) | 1 process line | +σ̂ line, strategy text, path |
| `llj` | identical `77ff26344ca02fab` | 0 / 500 | 1.009912 | 1 | same |
| `jj` | identical `97cca7cd481c30fc` | 0 / 500 | 0.942306 | 1 | same |
| `dy` | identical `f932cf0f3d9b015f` | 0 / 500 | 1.001079 | 1 | same |
| fixed-beam `gu` | identical `6bebef12c2d263bf` | 0 / 500 | 1.017279–1.017280 | 1 | same |
| fixed-beam `ee` | identical `d640093ae7642d3f` | 0 / 500 | 1.011998–1.011999 | 1 | same |

- The beam line of `<init>` and the trailer are identical. Each file's base
  `XSECUP` is exactly the new header's σ̂: `llj_fixed` 4.683304e2 → the
  integration's 4.256919e2, which is −9 % on 500 events.
- On the matched row, `integrate` with the new binary at seed 20260928
  (`--fixed-budget --allocate neyman --neval 200000 --niter 8`) is byte-equal
  to F-B's `mix-new-28` artifact (`c28f74ac…`). Base and new `generate` of
  that seed differ only in the 10000 XWGTUPs (ratio 0.956–1.003 by part), the
  three process lines and the header.
- The banked `validate-hadronic` pulls equal F-B's to every digit:
  `llj_fixed` +0.05, `llj_dyn` +0.07, `pp_to_llj` +0.61, `jj` +0.72,
  `mmll_60_120` +0.42.

**Measurements.**

`pp_to_ll_0j2j_mlm`, ten 10000-event samples. They come from F-B's ten
fixed-budget artifacts (seeds 20260928–37, `--seed` equal to the integration
seed), and use `mlm-p12-samplestats.py`. The "before" rows are what the base
writer declared: σ̂ with the integration's error.

| | mean (pb) | sd | χ²/dof over files |
|---|---|---|---|
| after: declared `XSECUP` ± `XERRUP` | 1064.95 ± 0.64 | 1.93 | **1.02** |
| before: σ̂ ± integration error (as written) | 1067.65 | 10.98 | 32.8 |
| before: σ̂ ± its own error ⊕ integration's | 1065.48 ± 3.71 | 10.98 | 0.75 |
| after `@0` / `@1` / `@2` | 665.26 / 268.35 / 130.94 | 0.56 / 1.28 / 2.07 | 0.66 / 0.90 / 1.89 |
| before `@0` / `@1` / `@2` (σ̂ₖ, integration error) | 663.57 / 269.69 / 134.11 | 4.90 / 4.30 / 5.10 | 51.2 / 13.9 / 16.1 |

- The declared σ now scatters as its error says. Its χ²/dof is the
  integrations' own, with `@2`'s 1.89 as F-B recorded.
- The sample estimate's own error is honest: the ten pulls of σ̂ against
  their integrations have `Σp²/n` 0.71, worst −1.46.
- The base file stated a 1 % estimate with a 0.18 % error.

**Pythia** (M5's driver and configuration, Pythia 8.312, seeds
20261201–06). Side A is M5's: `run_01` and the four fresh MadEvent
directories, read back from M5's cache. Side B is samples 20260928–32.

| | MadEvent | vibegraph (M5) | vibegraph (now) | pull now |
|---|---|---|---|---|
| acceptance `@0` | 0.8193 ± 0.0009 | 0.8188 ± 0.0009 | 0.8195 ± 0.0009 | +0.14 |
| acceptance `@1` | 0.3636 ± 0.0025 | 0.3592 ± 0.0026 | 0.3566 ± 0.0025 | −2.01 |
| acceptance `@2` | 0.3478 ± 0.0040 | 0.3408 ± 0.0068 | 0.3342 ± 0.0052 | −2.08 |
| acceptance, all | 0.6468 ± 0.0014 | 0.6422 ± 0.0021 | 0.6429 ± 0.0019 | −1.69 |
| merged σ (pb) | 688.20 ± 1.48 | 690.30 ± 2.28 | 685.06 ± 1.98 | −1.27 |
| merged σ χ²/dof over files | 1.36 | 4.67 | **0.77** | |
| `@0` / `@1` / `@2` acceptance χ²/dof | 1.13 / 0.25 / 0.11 | 1.08 / 0.57 / 3.06 | 1.01 / 1.55 / 0.98 | |
| jet rates χ² d01 / d12 / d23 | | 15.1/24, 33.6/21, 12.9/17 | 30.1/24 (p 0.18), 24.9/21 (p 0.25), 18.8/17 (p 0.34) | |

- The merged σ and every acceptance now scatter file to file as their
  errors say: 4.67 → 0.77, and `@2` 3.06 → 0.98.
- The merged σ, 685.06 ± 1.98, is −0.46 % from MadEvent. F-A's offline
  replay of this policy gave 686.56 on M5's (pre-merge) files.
- The `@1` and `@2` acceptances read −1.9 % and −3.9 % (about 2σ each). A
  per-part scale cannot move them: an acceptance is a ratio inside one part.
  These are new samples from F-B's merged grids, so the comparison with M5's
  −1.2 % and −2.0 % is across sample sets. It is recorded here for M5's
  close-out gate, not resolved.

**Target runs** (`--target-rel 2e-3`, Neyman by default).

`pp_to_ll_0j2j_mlm`, `--neval 200000 --max-iters 24`:

| seed | pooled: stop, evaluations, σ (pb) | old χ² (base binary): stop, evaluations, σ (pb) |
|---|---|---|
| 20260928 | **8**, 1.85M, 1066.40 ± 1.90 (quoted 0.178 %, scaled 0.1995 %) | 11, 2.53M, 1065.83 ± 1.60 (0.150 %, 0.1896 %) |
| 20260929 | 10, 2.31M, 1064.67 ± 1.74 | 15, 3.44M, 1066.32 ± 1.54 |
| 20260930 | 10, 2.31M, 1065.94 ± 1.87 | 15, 3.45M, 1064.13 ± 1.54 |
| F-A's replay (pre-merge, 364 channels), 20260928 | 8, 1064.89 ± 1.73 (0.163 %, 0.186 %) | never in 17 (M6 still running at 32) |

- After F-B's merge the old rule does stop on this row, at 11–15 iterations.
  The pooled rule stops at 8–10 and spends 1.4× fewer points.
- On seed 20260928 the pooled stop lands where F-A's replay did. Its
  artifact is byte-equal to the fixed-budget seed-28 artifact: the Neyman
  draws agree, so a target run stopped at iteration 8 is the fixed run.
- All six σ agree with the ten-seed fixed sweep, 1065.11 ± 0.61 (sd 1.93).
  The pooled three read 1065.67 (sd 0.90), +0.5σ against their mean quoted
  error over √3.

`pp_to_llj_mlm`, seeds 20260951–55, default `--neval`:

| seed | pooled: stop, σ (pb) | old χ² (base): stop, σ (pb) |
|---|---|---|
| 51 | 7, 268.671 ± 0.484 | 8, 268.458 ± 0.455 |
| 52 | 6, 268.593 ± 0.520 | 6, 268.593 ± 0.520 |
| 53 | 6, 268.742 ± 0.499 | 6, 268.742 ± 0.499 |
| 54 | 7, 269.030 ± 0.513 | 8, 268.961 ± 0.481 |
| 55 | 9, 268.717 ± 0.437 | 9, 268.717 ± 0.437 |

- After F-B's merge this row's iterations are consistent (χ²/dof 0.80), and
  both rules stop within one iteration of each other. Where they stop at the
  same iteration, σ is the same to the bit.
- The pooled mean is 268.75 (sd 0.17), against the fixed ten-seed sweep's
  268.53 ± 0.10 (sd 0.33): +1.0σ of the target runs' mean quoted error over
  √5.
- M6's pre-merge runs needed 7–25 iterations.

**Gates** (tree `d4b8ae3`, which differs from the final commit only in comments, this record and `TODO.md`; the banked tasks' own cargo commands with the
private build settings, `RAYON_NUM_THREADS = 2`):
- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`, plain and with
  both `extended-validation` features: clean.
- `cargo test --workspace`: as above.
- `validate-unweighting`: 1 passed. `validate-generate-proton`: 5 passed.
- `validate_samples`: 6 passed. `validate_samples_proton`: 11 passed, min KS
  p 6.7e-2.
- `validate-lhef`: 3 passed. `validate-hadronic`: 14 passed (pulls above).
- `validate-mlm-sigma`: 3 passed, every row as F-B recorded:
  `pp_to_llj_xqcut_only` 212.608 ± 0.126 (pull +0.23), `pp_to_llj_mlm`
  268.433 ± 0.119 (ten seeds, χ²/dof 0.86, pull +1.22), `pp_to_llj_mlm_alps2`
  241.255 ± 0.150 (pull +1.86).
- `probe_sample_sigma_seed_headroom`: as above.
- The Pythia consumption gate (`validate-pythia`, `consume.py`) reads event
  structure, not weights (no weight or σ access in `consume.py`), so it was
  not rerun. `validate-mlm-pythia`, which does read XWGTUP, is the table
  above.

**Where the brief was short.**
- "Weights sum to the part's σ" is read under the file's mean-weight
  convention, `Σ XWGTUP/N`. A literal sum would be `IDWTUP = −3` /
  `event_norm = sum`, and would break main164's and M5's normalisation at
  `−4`.
- The brief gave "iteration 8 factor 1.86e-3" as a factor. 1.86e-3 is the
  pooled-scaled relative error at iteration 8; the channel factors are of
  order one.
- `cli_decay`'s top-decay `XSECUP` check uses stochastic rounding, whose
  `XSECUP` was already the integration's. It is unchanged.

**What remains.**
- The `@1` / `@2` Pythia acceptances, about 2σ low on these samples, go to
  M5's close-out gate.
- The VEGAS grid tail (backlog) is still the root of the overweights. The
  normalisation fixes the file's σ, not the shape noise the overweights carry.
- `StochasticRounding` has no per-part normalisation.

#### C Landed: the `@1` / `@2` acceptances at four times the statistics, 2026-10-01

The question was P12's: on its regenerated samples (seeds 28–32) the `@1` and
`@2` Pythia matching acceptances read about 2σ low against MadEvent (−2.01,
−2.08). **It does not persist. At four times the events per side the pulls
are −0.95 and −0.98, and half of P12's deficit was MadEvent's five files
reading high.** No production code changed.

**Samples.**
- **MadEvent, 21 files:** `run_01`, M5's four fresh directories (seeds
  20261101–04), and sixteen new fresh directories (seeds 20261105–20). Each
  directory is generated by the pinned 3.7.1 tree from M0's proc lines and
  the committed run card with `iseed` set (`nb_core = 2`, `vector_size = 1`
  checked in every file). Each holds 10000 events, and the sixteen new runs
  read σ_LHE from 1059.8 to 1068.2 pb. The script is M5's `mg-fresh.sh`, with
  the `madgraph` pixi environment taken from the `mlm-m0` worktree.
- **vibegraph, 20 files:** P12's ten (seeds 20260928–37, from F-B's
  artifacts) and ten new ones, seeds 20260938–47. The new ones were
  integrated with P12's binary (`21a37592…`) at P12's configuration
  (`--fixed-budget --allocate neyman --neval 200000 --niter 8`) and generated
  at 10000 events with `--seed` equal to the integration seed. Their
  integrations read 1062.2–1068.6 pb, mean 1065.04, against F-B's ten-seed
  1065.11 ± 0.61.
- **Pythia:** 8.312, M5's driver (sha256 `1723111b…`, the build P12 used)
  and configuration (MadGraph's own: `qCut = 30`). Ten seeds, 20261201–10,
  on every file. P12's and M5's cached runs for seeds 1–6 are reused: the
  cache key holds each input's sha256.
- **Statistics:** the variance of a `@1` event's seed-averaged outcome is
  0.187/S from the shower plus 0.045 from the Les Houches event (P12's runs).
  At ten seeds the shower part is 0.019, so events, not seeds, set the error.

**Results** (A = MadEvent, 21 × 10000; B = vibegraph, 20 × 10000; ten seeds):

| | MadEvent | vibegraph | B − A | pull | P12 (5 + 5, six seeds) |
|---|---|---|---|---|---|
| acceptance `@0` | 0.8191 ± 0.0003 | 0.8187 ± 0.0004 | −0.05 % | −0.80 | +0.14 |
| acceptance `@1` | 0.3603 ± 0.0011 | 0.3587 ± 0.0012 | −0.44 % | −0.95 | −2.01 |
| acceptance `@2` | 0.3459 ± 0.0018 | 0.3432 ± 0.0021 | −0.78 % | −0.98 | −2.08 |
| acceptance, all | 0.6447 ± 0.0006 | 0.6441 ± 0.0007 | −0.09 % | −0.66 | −1.69 |
| merged σ (pb) | 685.77 ± 0.68 | 685.98 ± 0.80 | +0.03 % | +0.19 | −1.27 |
| jet rates χ² d01 / d12 / d23 | | | | 24.5/26 (p 0.55), 14.4/23 (0.91), 20.7/20 (0.42) | 30.1/24, 24.9/21, 18.8/17 |

Bootstrap-error χ²: 25.2, 14.5, 21.3.

**File-to-file spreads** (χ²/dof of the per-file values about the side's mean):

| | `@0` | `@1` | `@2` | merged σ (sd) |
|---|---|---|---|---|
| MadEvent, 21 files | 0.63 | 1.00 | 0.80 | 1.04 (3.18 pb) |
| vibegraph, 20 files | 0.75 | 0.88 | 0.92 | 0.53 (2.48 pb) |

Both sides scatter as their errors say, so the per-event errors carry the
comparison. The per-file `@1` values run 0.3473–0.3691 on MadEvent's side and
0.3500–0.3678 on vibegraph's.

**Nulls** (cached runs re-keyed, same ten seeds):

| split | `@0` | `@1` | `@2` | all | merged σ | χ² d01 / d12 / d23 |
|---|---|---|---|---|---|---|
| MadEvent: files 1–11 against 12–21 | +0.09 | −0.47 | −0.07 | −0.96 | −1.26 | 17.1/25, **35.6/22 (p 0.034)**, 18.3/19 |
| vibegraph: seeds 28–37 against 38–47 | +0.67 | +0.23 | +1.36 | +0.69 | +0.66 | 34.3/25 (p 0.10), 13.6/22, 21.7/19 |
| MadEvent: M5's five against the sixteen new | +0.46 | **−1.45** | −0.20 | −1.34 | −1.48 | 12.5/24, 30.3/21 (p 0.086), 14.4/18 |

- **Where P12's deficit came from.** M5's five MadEvent files read `@1`
  0.3631 ± 0.0022 at ten seeds, 1.45σ above the sixteen new directories'
  0.3594 ± 0.0012. P12's first five vibegraph files read 0.3566 at six
  seeds; the twenty read 0.3587. Both reference sets sat on the far side of
  their means.
- **d12.** The MadEvent-against-MadEvent split reads d12 χ² 35.6/22 (p 0.034).
  That is M5's vibegraph-against-MadEvent tension (33.6/21, p 0.040) in a
  comparison with no vibegraph in it. M5's d12 shape is within the
  comparison's own scatter, and at twenty files a side it reads 14.4/23.

**Localisation, done anyway at the full statistics** (`localise.py`: per-event
outcomes binned in LHE variables, each `@N`'s difference split into a
fraction term `Σ(f_B − f_A)a_A` and an acceptance term `Σf_B(a_B − a_A)`).
No bin of any variable reaches 3σ.
- `@1` by initial state: `qg` 0.3519 / 0.3509 (−0.60), `qq̄` 0.3809 / 0.3779
  (−0.82).
- `@2` by initial state: `gg` −1.36, `qg` −1.03, `qq'` −1.57, `qq̄` +1.31.
  `qq'`'s weighted fraction is 0.1203 against 0.1106 (+1.85). Those are the
  four-quark subprocesses where D2's permuted-P1 deviation and the heaviest
  overweights sit.
- Binned in the lowest `pt_clust` of the event, the lowest and highest parton
  pT, the largest parton |η|, SCALUP, and `pt_clust / pT` (which separates
  partons clustered to the beam from the rest), the acceptance χ² over bins
  is 1.6/2 to 11.0/9 on the twelve tables. The fraction χ² is 0.1/1 to
  11.7/7.
- Near the matching scale (`pt_clust` in [20, 25) and [25, 30) GeV, where the
  acceptance climbs from 0.07 to 0.21), the two sides agree bin by bin
  within 1.5σ on `@1` and `@2` (worst −1.32 on `@1` at [25, 30), +1.46 on
  `@2` at [22, 25)).

**The hypotheses, measured though not needed:**

| sample set (B) | files | `@0` | `@1` | `@2` | merged σ (pb) |
|---|---|---|---|---|---|
| post-F-B, `--neval 200000` (above) | 20 | −0.80 | −0.95 | −0.98 | 685.98 ± 0.80 (+0.19) |
| post-F-B, `--neval 600000` (F-B's `mix600-new-28..30`) | 3 | −0.38 | −0.83 | −1.35 | 685.14 ± 1.99 (−0.30) |
| pre-F-B (M5's samples, base writer) | 5 | −0.03 | −0.67 | −0.27 | 690.70 ± 1.96 (+2.37; file χ²/dof 5.24, no per-`@N` normalisation) |

- Tripling `--neval` does not move `@2` up (0.3386 ± 0.0051 against
  0.3432 ± 0.0021), so F-B's under-sampled `@2` tail is not what P12 saw.
- The pre-F-B files agree with MadEvent on every acceptance. Their merged σ
  is high because the base writer declared each sample's own σ̂ (P12's
  policy 1 is the fix). Pre- and post-F-B acceptances agree.
- The overweights do not reach the acceptances at this resolution. The
  vibegraph files' acceptance χ²/dof is 0.75–0.92, against M5's 3.06 on
  `@2`.

**Verdict: statistical.** P12's −2σ readings came from five files a side,
with MadEvent's five reading 1.45σ high on `@1`. At four times the
statistics every acceptance agrees within 1σ, and so do the merged σ and the
three jet-rate shapes. Both sides' file spreads match their errors, and the
nulls read on the same scale. The three `@N` pulls (−0.80, −0.95, −0.98)
share a sign, but their combination is the overall acceptance at −0.66. At
the current resolution, vibegraph's `@1` and `@2` acceptances differ from
MadEvent's by −0.44 ± 0.45 % and −0.78 ± 0.80 %.

**Commands.** Scratchpad `mlm-c/`:
- `mg-fresh.sh` and `vg-samples.sh` make the samples;
- `py.sh NAME A B …` runs `validation/pythia/mlm_match.py` on the shared
  cache `mlm-c/work`, with the JSON in `res-NAME.json`;
- `clone.py` re-keys cached runs for the nulls;
- `localise.py` and `perfile.py` read the results.

**Where the brief was short.** "The regenerated vibegraph samples now read
`@1` and `@2` low" put the shift on vibegraph's side. Half of it was
MadEvent's: P12's reference, `run_01` plus M5's four directories, reads `@1`
0.3631 at ten seeds, against 0.3594 for sixteen further directories of the
same card.

**What remains.**
- The row stays `info`. The comparison now has the statistics a gate would
  need: 21 MadEvent and 20 vibegraph files, both side spreads at their
  errors, and a MadEvent-against-MadEvent null on record. What the close-out
  decides:
  - whether to bank the sixteen MadEvent directories as `--a` inputs (2 MB
    each); the task's default side A is still `run_01` alone;
  - the tolerance. At this size the comparison resolves ±0.3 % (`@1`) and
    ±0.5 % (`@2`) per side, 1σ.
- The VEGAS grid tail is unchanged. It no longer shows in the acceptances at
  this resolution.

### M6: xqcut-aware phase space (performance-dev; after M3)

- Put the jet energy floors and s-channel minima (§1.4) into the multichannel
  maps and `Cuts::spacelike_floor()`.
- Measure unweighting efficiency and CPU-to-target against the note-30 baseline
  on `pp_to_ll_0j2j_mlm`.
- σ must not move (seed-sweep gate).

#### M6 Landed, 2026-09-29

One production change, to the budget: a Neyman re-split is now handed what
the α split spends in the same iteration, floors included. The xqcut floors
needed no change. A tighter τ floor was built, measured and dropped.

Every number below comes from the `069a951` binary (`afb22629…`) and the
final one (`96b787fc…`), both release-debug builds in a private target. The
container has 4 cores and was shared with a sibling session. The load average
was 9–10 until about 05:00 and 4.0 after, so CPU times from different hours
are not comparable. Point counts and errors are, and they carry the comparison.

**Baseline**, `pp_to_ll_0j2j_mlm`, `--fixed-budget --neval 200000 --niter 8`,
seed 20260928, on the `069a951` binary:

| allocation | @0 (pb) | @1 (pb) | @2 (pb) | total (pb) | points | ε_unw |
|---|---|---|---|---|---|---|
| by α | 665.02 ± 0.70 | 268.13 ± 1.48 | 132.76 ± 1.01 | 1065.90 ± 1.92 | 4.50M | 2.01 % |
| Neyman | 661.18 ± 6.56 | 269.83 ± 2.77 | 132.76 ± 1.01 | 1063.77 ± 7.19 | 3.33M | 1.96 % |

- The by-α row reproduces M3's seed-20260928 row bit for bit. M3's five-seed
  table is therefore this binary's by-α sweep.
- ε_unw is the sampled efficiency of `generate --nevents 5000` on each run's
  own artifact. The scan's predicted value agrees within 2 %.
- Where the points go, by α: the survey splits the budget 81.0 / 12.4 / 6.7 %
  over @0 / @1 / @2, so @2 is asked for 13.4k points an iteration. Its 336
  channels are all floor-bound, and from the second iteration on the
  acceptance-raised floors draw 602k of the iteration's 807k points; 144 of
  the 336 channels sit at the 2048-point cap.
- `pp_to_llj_mlm`, `--fixed-budget --neval 150000 --niter 10`, seeds
  20260951–55: 268.72 ± 0.30 pb (sd 0.67, χ²/dof 2.42).
- `pp_to_llj_mlm`, the default `--target-rel` run (Neyman) at 2e-3, same
  seeds: 2.11M evaluations to the stop (1.58M–2.54M).

**xqcut floors: already in the maps.** MadEvent's xqcut floors are all
grid hints except one:
- `setxqcuts` (`setcuts.f:892-944`) sets `xqcuti = √(xqcut² − m²)` on jet
  legs and `xqcutij = xqcut` on jet pairs meeting at an s-channel vertex.
- `set_peaks` (`myamp.f:337-551`) reads them into a leg's energy floor `xe` and
  an s-channel mass floor `xm`. `xm` only presets a grid (`setgrid`,
  `dsample.f:938`, which keeps 10 % of the bins below it). The one hard limit
  is the τ minimum `etot²/s`, which M1 proved implied by the cuts.

After the `ptj = mmjj = xqcut` rewrite (`setcuts.f:156-189`, ported by M1),
the compiled cuts carry both floors, so the maps already start there:
- a massless jet's `energy_floor` is `xqcut`;
- a jet pair's `timelike_floor` is `xqcut²`;
- `spacelike_floor` is `xqcut²`.

These are provable lower bounds, so they remove only zero-weight regions and
cannot move σ. `cuts::madevents_xqcut_floors_reach_the_maps_through_the_rewritten_cuts`
pins all three. The kT veto itself (`reweight.f:1063-1089`) cannot be turned
into a floor:
- it is decided per flavour group in that group's clustering, and every point
  sums every group;
- no point loses all of its groups to it (0 of the 1.77M one- and two-jet
  draws of a three-iteration probe);
- a jet pair's `d_ij = m²·p_T,min/p_T,max` is at least `xqcut²` only when
  `m ≥ xqcut`, so no invariant floor tighter than `mmjj` follows from it.

**The τ floor, built and dropped.** The partition bound
`√ŝ ≥ max over partitions Σ max(Σ p_T,min, √timelike_floor)` is `(50 + 20·n_j)`
on these cards, against today's `max(mmll, Σ p_T,min)`. It is provably implied:
a probe counted 0 accepted points below it. Measured:
- **Waste it would remove:** 6–7.5 % of the draws in the survey and first
  iteration, then 0.8–1.1 % per iteration once VEGAS adapts.
- **Mixed row:** it moved the last iteration's points by −0.1 %.
- **`pp_to_llj_mlm`:** rel²·CPU went from 516 ± 95 to 460 ± 99 µs, not
  significant.
- **`pp_to_llj_fixed`** (banked, `xqcut = 0`, `mmll = 50`): the summed `w_max`
  rose from 7.80e3 to 1.33e4, and the predicted ε_unw fell from 5.45 % to
  3.18 %.
  - `cli_generate_proton`'s sample-against-integration bound failed at −1.80 %
    (bound 1.5 %).
  - Its five-seed headroom probe widened from {+0.36, −0.43, +0.01, −0.26,
    −0.40} % to {−1.80, −0.56, +1.57, +1.03, +0.23} %.
  - σ itself held: `validate_hadronic` pulls were −0.00 and −0.04 on the two
    rows it moved.

The floor worsens unweighting on the banked row it touches and buys nothing
measurable on the matched ones. Why a higher τ floor fattens the weight tail
was not diagnosed. It was reverted.

**The Neyman starvation.**
- **Cause:** `integrate_channels` handed `neyman_allocation` the uncorrected
  total `Σ max(shareⱼ, 512)`, while the floors it enforced were the
  acceptance-corrected ones, up to 2048 each. On this card the 336 raised
  two-jet floors (~570k) exceed that total (364k), so every channel was
  pinned at its floor and the four zero-jet channels drew ~2k points instead
  of ~162k.
- **Fix** (`budget.rs:748-762`): the re-split is handed `Σ max(shareⱼ, floorⱼ)`,
  the α split's own spend in that iteration. The acceptance correction becomes
  a coverage cost both rules pay alike, and Neyman moves only the points above
  the floors. This is a property of the allocation, not of the composite.
- **Test:** `budget::a_neyman_split_spends_what_the_alpha_split_spends_past_raised_floors`
  pins it on a 41-channel toy. It fails on the old total.
- **Unchanged:** by-α runs are unchanged bit for bit, and so are Neyman runs
  whose floors never rise above their shares. The fixed Neyman run spends the
  α split's points to within rounding (3,330,568 against 3,330,624 over
  iterations 3–8).

**Before and after, the mixed row**, `--fixed-budget --allocate neyman --neval
200000 --niter 8`:

| | total (pb) | sd | χ²/dof | quoted error per seed |
|---|---|---|---|---|
| by α (M3, 5 seeds; both binaries) | 1066.98 ± 1.28 | 2.86 | 1.43 | 1.84–3.28 |
| Neyman, `069a951` (3 seeds) | 1068.63 ± 4.27 | 5.07 | 0.43 | 6.57–8.32 |
| Neyman, fixed (5 seeds) | 1065.50 ± 0.89 | 1.26 | 0.42 | 1.73–2.23 |

Readings:
- **Fixed Neyman against by α,** at equal points and matched seeds: the
  quoted variance falls by 1.00–2.44×, mean 1.69×. At the same quiet load,
  seed 20260931 cost 3,191 s of CPU under Neyman against 3,407 s by α.
- **The by-α run on the final binary** reproduces M3's seed-20260931 row bit
  for bit (666.03 ± 0.68, 266.32 ± 1.49, 131.35 ± 0.85, 1063.70 ± 1.84).
- **Against the old Neyman:** 13×, from rel²·CPU 0.143 ± 0.033 s (3 seeds) to
  0.0110 ± 0.0025 s (5 seeds). The old Neyman drew 26 % fewer points, all of
  them cheap zero-jet ones, and cost the same CPU (2964–3035 s against
  2926–3242 s).
- **ε_unw at seed 20260928:** 1.87 % after, 1.96 % before (Neyman) and 2.01 %
  by α. One seed each, so the fix does not measurably change ε_unw.
- **CPU-to-target, projected** from rel²·CPU: 0.2 % costs about 2,700 s of
  CPU after, against about 36,000 s before.
- **CPU-to-target, measured:** a real `--target-rel 2e-3` run of the fixed
  binary (seed 20260928) never stopped. It was killed after 32 iterations,
  16.5M points and 13,289 s of CPU.
  - Its quoted error was 0.109 %, but the stop reads each channel's error
    widened by `√max(1, χ²/dof)`.
  - The row's χ²/dof rose from 1.9 at iteration 8 to 5.6 at iteration 32, and
    the quoted error fell slower than `1/√n` (0.163 % → 0.109 % over 4× the
    iterations).
  - So on this card the stopping rule's consistency factor, not the
    allocation, now bounds the time to a target.
  - The pre-fix target run was not attempted, since its projection is 13×
    worse.

**σ did not move.** The five fixed-Neyman seeds (20260928–32) against the
same card:

| | vibegraph, fixed Neyman (sd, χ²/dof) | M3 by α | MadEvent |
|---|---|---|---|
| @0 | 665.23 ± 0.32 (0.65, 0.87) | 665.32 ± 0.19 | 664.81 ± 0.52 |
| @1 | 268.22 ± 0.72 (1.36, 0.78) | 269.29 ± 1.27 | 269.04 ± 0.53 |
| @2 | 132.06 ± 0.42 (0.51, 0.32) | 132.38 ± 0.31 | 131.09 ± 0.11 (D2, 7 dirs) |
| total | 1065.50 ± 0.89 (1.26, 0.42) | 1066.98 ± 1.28 | 1064.37 ± 0.62 |

- Every difference from the by-α sweep is under 1σ of the combined errors.
- `@2` is +0.74 % over D2's independent-directory value. That is inside D2's
  measured H1 + generic-offset budget, and M3's by-α sweep (+0.98 %) matches
  it.
- `pp_to_llj_mlm` at the default target, 2e-3, seeds 20260951–55: σ
  268.51 ± 0.20 after against 268.58 ± 0.25 before (χ²/dof 1.00 and 1.59).
  Evaluations to the stop were 1.84M (0.88M–3.10M) against 2.11M
  (1.58M–2.54M): no difference the five seeds resolve. On 24 channels the
  floors barely rise past the shares.
- **At `xqcut = 0`:** a by-α (`--fixed-budget`) run draws the same points in
  the same order as before. The `pp_to_llj_fixed` artifact (300k × 8, seed
  20260731) and the `pp_to_llj_mlm` artifact (150k × 10, seed 20260951) are
  byte-identical to the base binary's. So is `pp_to_llj_fixed`'s 20000-event
  LHE file, except the header line naming the artifact path. A Neyman run (the
  `--target-rel` default) changes its draws wherever an acceptance-raised floor
  exceeds its share, at any `xqcut`. No banked gate integrates under Neyman,
  and `validate-hadronic` agrees on every row (below).

**Gates** (final tree, `a65f930` plus this record):
- `cargo fmt --check` and both `clippy -D warnings` passes (default, and
  `extended-validation`) are clean.
- `cargo test --workspace`: 33 binaries, 1282 passed, 0 failed, 15 ignored.
- The banked tasks' own cargo commands, run through `pixi run` with the
  private build settings (`--skip-deps` equivalents):
  - `validate_hadronic`: 14 passed. `llj_fixed` pull −0.07, `llj_dyn` −0.08,
    `pp_to_llj` −0.13, `jj` +0.80.
  - `validate_unweighting`: 1 passed.
  - `cli_generate_proton`: 4 passed. Sample +0.355 %, identical to base.
  - `validate_samples`: 6 passed.
  - `validate_sigma`: 7 passed.
  - `validate-mlm-sigma`: `pp_to_llj_xqcut_only` 212.777, `pp_to_llj_mlm`
    268.560 and `pp_to_llj_mlm_alps2` 241.052 pb. These are M1's and M2's
    values to the printed digit, since their fixed-budget draws are
    unchanged.

**Where §4 M6 was wrong.**
- The xqcut energy floors and s-channel minima were already in the maps and
  in `spacelike_floor`, since M1 ported the card rewrites. What was missing
  was MadEvent's τ bound, and on these cards that bound does not pay.
- The session's lever was the allocation, and the "note-30 baseline" does not
  exist for this row. The baseline is measured above.

**What remains.** The two-jet cost is the floors: 336 channels × up to 2048
points. The same card decomposes into only **36 distinct channel maps** among
the 336, and 6 among the one-jet part's 24. That was measured by comparing
every built channel's full map, across the 28 two-jet flavour groups.
- Merging channels with identical maps across flavour groups, as MadEvent's
  `config_subproc_map` does, would cut the two-jet floor cost about 9×.
- It needs a rule for which clustering configuration a merged channel hands
  each group, for the jet memo and for colour. It also needs a new
  channel-key schema.
- That is its own session. No change to `MIN_CHANNEL_NEVAL` was made: the
  floor is the coverage guarantee, and the grouping is what removes its
  redundancy.

The sample-against-integration σ of this row at 5000 events is +2.0 to +2.4 %
before and after, with overweights carrying 5 % of σ. That is a truncation
reading for the unweighting follow-up, not this change.

### Z: close-out

As note 38 §7, in three phases, in order:
1. **Z1**, in this container: run the whole banked layer, register every
   cell, regenerate `@2`'s reference, wire the generators (done; record below).
2. **B1**, on the bank host: generate every MLM run and bank `refdata-9`.
3. **Z2**, wherever `refdata-9` can be fetched: re-verify, write the mixed
   rows' σ gate, and flip each cell that agrees to `gate`.

The sprint closes when Z2 is green.

#### Z.1 What the bundle must carry

Note 38 §7.1's three categories, for every MLM artifact. Inputs (proc scripts
`scripts/<row>.mg5`, run cards `<row>_run_card.dat`, R1's reproducer and patch)
are committed and are not references.

| artifact | category | B1 | check |
|---|---|---|---|
| `mlm_census.json` (jet memo, IPROC jet-ness) | 1: committed structural | regenerated by the `mlm` stage (`dump_mlm_census.py`, from the dumps and process directories) | must match **exactly** |
| `mlm_dump_manifest.json` (pins the five dumps, their per-event matched-record census) | 1: committed structural | regenerated by the `mlm` stage's replay | every field **exactly**, except `sha256` (Python's `gzip` writes the time into the header) and the shard names inside `memo_branch_examples` (`raw.<pid>.gz`), which are per host; commit the regenerated file so it pins the dumps the host gated |
| `configs.json`, `sigma_reference.json`, `diagrams.json`, `interactions.json` | 1: committed structural | `refs` stage | **unchanged**: their extractors skip the MLM rows (proton beams, several `P<n>` directories, no hermetic `diagrams` cell) |
| `mlm_sigma_reference.json` | 2: committed MadEvent σ scalars | re-measured: 5 rows × ten seeds (the samples run plus nine in one shared directory), plus `pp_to_ll_0j2j_mlm`'s twenty fresh directories | every `sigma_pb`, `err_pb` and `by_lprup` value **within its seed error**; bit-equal is expected (note 38 §8.4 found 176/176 MadEvent seeds bit-equal between this container and the bank host). `wall_s` differs |
| `output/<row>` × 5: Cards, `Events/run_01` (event file, banner, seed record), `results.dat`, `leshouche.inc`, `matrix*_orig.f`, run logs, `build.log` | 3: runs a gate needs | banked (the `mlm` stage makes `run_01` as the first run of a fresh directory) | `validate_lhef` round-trips each event file byte for byte, `validate_scales` declares each (`MatchedDump`), `mg_run_card_matches_madevents_banner` reads the banners, `matched_sample_record_fractions_against_madevent` reads `pp_to_llj_mlm`'s events |
| `output/pp_to_ll_0j2j_mlm/Events/run_s20261101` … `run_s20261120` (event file, banner, seed record; ~2 MB each) | 3: runs a gate needs | banked, new: one freshly generated directory per seed, the directory dropped after the run | the σ reference's runs, and side A of the matched Pythia comparison (`mlm_match.py`'s default); `validate_lhef` sweeps them too |
| `output/ktdump/dumps/<row>.jsonl.gz` (58 MB) and the full process directories | regenerated, not banked (oracle layer) | regenerated by the `mlm` stage: the instrumented replay must reproduce `run_01` byte for byte | `validate_mlm_dumps` also reads each directory's `configs.inc`, `config_nqcd.inc` and `config_subproc_map.inc`, which the bundle does not carry, so the dumps alone would not make it banked |
| `work/mlm/<row>` (the shared-directory seeds) | regenerated, not banked | the `mlm` stage | only their scalars are committed |
| vibegraph's matched samples (`target/mlm-pythia-samples`) | not a MadGraph reference | `pixi run -e pythia generate-mlm-pythia-samples` (optional on B1) | — |

#### Z.2 Z1 (this container)

1. The whole banked layer once, end to end: `pixi run --skip-deps validate` at
   the sprint tip against `refdata-8` plus the MLM outputs on disk, collator
   included. Fix what fails; report every rendered cell.
2. `pp_to_ll_0j2j_mlm`'s reference from independent directories, the generator
   able to reproduce it with one fresh directory per seed, and the decision on
   banking the event files as the Pythia gate's side A.
3. Every MLM row and cell registered for B1, and the artifacts classified (Z.1).
4. The loose ends: M5's two release-build warnings; the `release-debug`
   environment-variable trap documented.
5. This step list, and the bookkeeping (`TODO.md`, the backlogs, the record).

#### Z.3 B1: banking `refdata-9` on the bank host

The bank host needs:
- a checkout at the Z1 commit (branch `mlm-z`);
- the submodule at its pin (`b7687064`, MadGraph 3.7.1), run through
  `validation/madgraph/mg5_pinned.sh`;
- the pixi `madgraph` environment with gfortran and LHAPDF, and both PDF sets
  fetched (`NNPDF23_lo_as_0130_qed`, `NNPDF31_lo_as_0130`);
- for the optional Pythia step, the pixi `pythia` environment (Pythia 8.312);
- about 4 GB free for the five MLM process directories, the replays' work
  areas and the dumps; the twenty fresh directories are made and dropped one at
  a time.

Steps:
1. **Start from the current bank.** `pixi run fetch-refdata` unpacks
   `refdata-8` into `validation/madgraph/output/`; nothing in it is regenerated.
2. **Generate.**
   `pixi run -e madgraph generate-references deps madgraph seeds mlm refs`.
   - `madgraph` and `seeds` find every `refdata-8` directory and finished seed
     and do nothing new (`build.sh` skips the MLM scripts by their
     `# built-by:` line).
   - `mlm` (`gen_mlm_references.sh`, `NB_CORE = 2`): five fresh `output/<row>`
     directories with `run_01` (seed 20260928), nine shared-directory seeds a
     row (20260929–37) in `work/mlm/<row>`, twenty fresh directories for
     `pp_to_ll_0j2j_mlm` (20261101–20) banked as `Events/run_s<seed>`, then the
     five instrumented replays (each must reproduce its `run_01` byte for byte,
     or the stage stops) and the census. In this container the runs took
     30–160 s each on two cores (a mixed-card directory about a minute more to
     generate) and the replays 8–62 min, so about four hours in all; the bank
     host is faster.
   - `refs` reruns every committed table.
   - Two things have never run on macOS: the twenty-directory loop (the
     generator's dry run was on Linux with a stubbed MadGraph) and the
     instrumented replay with gzip-through-a-named-pipe (`VG_KTDUMP_GZIP`).
     A failure in either is a generator finding, as note 38 §8.4's four were.
3. **Reproduction check before bundling** (`git diff --stat` on
   `validation/`, then each file):
   - exactly equal: `mlm_census.json`, `configs.json`, `sigma_reference.json`,
     `diagrams.json`, `interactions.json`, and every other `refdata-8`-era
     table (no generator outside the MLM ones changed);
   - `mlm_dump_manifest.json`: only `sha256` and the shard names in
     `memo_branch_examples` may differ;
   - `mlm_sigma_reference.json`: every value within its seed error, `wall_s`
     aside. A value outside its error, or any structural difference, is a
     finding: stop and report it, do not overwrite.
4. **Gate on the host.** `pixi run --skip-deps validate` against the work area.
   The MLM runs are on disk there for the first time; every gate that sweeps
   the work area (`color_cf_oracle`, `validate_alphas`, `validate_scales`,
   `validate_lhef`, …) already met the five `run_01` directories in Z1, where
   two of them failed and were fixed (Z1 record), and `validate_lhef` meets the
   twenty `run_s*` files for the first time. Then the oracle-layer MLM gates
   against the fresh dumps and runs, recorded with their output:
   - `pixi run -e madgraph validate-mlm-dumps` (3 tests; every per-event field
     as recorded in M1, M2 and M4; the jet-memo proof on 8/8/8/29/82 channels);
   - `pixi run -e madgraph validate-mlm-samples`;
   - `pixi run -e madgraph validate-mlm-sigma` (reads only the committed JSON;
     P12's pulls +0.23, +1.22, +1.86 are the last record);
   - optional: `pixi run -e pythia generate-mlm-pythia-samples` and
     `pixi run -e pythia validate-mlm-pythia` (side A now defaults to the 21
     banked files; C read every acceptance within 1σ).
5. **Assemble and publish.**
   - `bash validation/madgraph/assemble_bundle.sh` gives
     `vibegraph-refdata-9.tar.zst` once `[refdata]` names it (set
     `version = 9` and `archive` first; the script reports the hash it made).
   - It must be exactly `refdata-8`'s members, all byte-identical, plus the
     five MLM process directories' banked files and the twenty `run_s*` runs.
   - Publish it as the `refdata-9` release asset. Update `[refdata]` in
     `validation/manifest.toml`: `version`, `archive`, `url`, `sha256`,
     `size_bytes`, and a comment line for cut 9.
   - Drop `bundled = false` and `status = "planned"` on the five MLM rows.
   - Commit, with the host, MadGraph version and wall time in the message
     body.

#### Z.4 Z2: re-verify from the published bundle

On a machine that has never generated a run (this container with
`validation/madgraph/output` removed, or CI):
1. `pixi run fetch-refdata` verifies the `refdata-9` hash, and
   `pixi run validate` passes.
2. **Write the mixed rows' σ gate** (long tier, beside the llj rows in
   `validate_hadronic`, run by `validate-mlm-sigma`): σ per `@N` and in total,
   ten or more seeds at `--fixed-budget --allocate neyman --neval 200000
   --niter 8` through the composite, both sides read under the seed policy
   (mean ± max(quoted, spread/√n)). For `pp_to_ll_0j2j_mlm` the reference is
   the 21 independent directories, so its error is their spread: `@2` 0.203 pb
   against 0.177 quoted, which is MadEvent's measured inter-directory spread
   entering the tolerance.
3. **Flip each cell that agrees**, only where this run (or B1's, for the
   oracle-layer dumps) measured it:

| cell | from | to | tolerance |
|---|---|---|---|
| `pp_to_llj_xqcut_only` integrals | long / info | long / gate | \|pull of the means\| < 3 with both policy errors; this side's seed χ²/dof inside its 0.1–99.9 % band (note 38 §8.5's rule). P12 read +0.23 (tree `d4b8ae3`) |
| `pp_to_llj_mlm` integrals | long / info | long / gate | the same. P12 read +1.22 |
| `pp_to_llj_mlm_alps2` integrals | long / info | long / gate | the same. P12 read +1.86 |
| `pp_to_ll_0j2j_mlm` integrals | uncovered | long / gate | per `@N` and total, \|pull\| < 3 with both policy errors. The registered deviations (H1 +0.33 pb on `@2`, the generic 2 → 4 offset ≈ +0.6 pb) sum to 0.7 % of `@2`, inside 3σ only while this side's `@2` error stays near its measured 0.66 pb; a budget that brings it below ~0.3 pb needs them as an explicit allowance or the gate fails on known causes |
| `pp_to_ttx_0j1j_mlm` integrals | uncovered | long / info | first measurement; flip to gate only after a five-seed agreement is on record |
| samples of `pp_to_llj_mlm`, `_alps2`, `pp_to_ll_0j2j_mlm`, `pp_to_ttx_0j1j_mlm` (`validate_mlm_dumps`) | long / info | long / gate | every per-event field at 1e-12 (AQCDUP 1e-6), `<scales>` string-equal, status-2 lines equal; permuted-`P1` events stay `info`. Measured from B1's freshly regenerated dumps, since the dumps are not in the bundle (the user's call: note 38 §7.4 flips only from the bundle) |

   Nothing else flips: the `diagrams` and `amplitudes` cells stay
   `uncovered`, `pp_to_llj_xqcut_only`'s samples cell waits on its
   `validate_samples_proton` row, and `mlm-pythia` stays `info` (no collator
   row; its tolerance is an open decision in the validation backlog).
4. Note 41's Z2 record; the sprint is closed.

#### Z.5 Findings that leave with the sprint

To the validation backlog (`TODO.md`, "Open findings from `mlm`"): Z2's gate
and flips; the generic 2 → 4 offset; the llj rows' shared +0.03–0.23 %;
H1 as a registered deviation, with the unchecked `ickkw = 0` grouped rows;
the MadGraph defect reports to file (R1's, and §1.5's six); the CDATA /
`setMad` interplay; `StochasticRounding`'s missing per-part normalisation; the
Pythia comparison's tolerance; the resonance code's two readings;
`pp_to_llj_xqcut_only`'s samples cell; `g g > t t~ g`'s missing
amplitude-level gate. To the performance backlog: the VEGAS
grid tail; `@2`'s heavy tail in the allocation; the per-point cost of wide
mixtures; MadEvent's `SYMCONF` sharing; the τ floor that fattened the tail.

#### Z1 close-out record, 2026-10-01 (this container, on `mlm-z` from `3e51f8a`)

Commits: `37c4490` (the two gate fixes, the reference, the generator, the
registrations) and the close-out commit after it (this record, the step list,
`TODO.md`, the skill, the docs).

**The banked layer end to end.** The first run (tip `3e51f8a`, `refdata-8`
plus the five MLM directories on disk) failed in `color_cf_oracle` on
`pp_to_ttx_0j1j_mlm/P1_gg_ttxg`, "graph 11: vibegraph writes 1 colour
structures, MadGraph 3", and `cargo test`'s fail-fast left every later target
unrun (1267 passed before it, 1 failed). That oracle sweeps every banked
process directory, so it met `g g > t t~ g` for the first time here; CI on
`refdata-8` cannot, and B1's host run would have.

- **Diagnosis** (a throwaway dump of both sides' per-graph columns, not
  committed). The colour matrix agrees exactly, and so do all sixteen graphs'
  JAMP columns as sets: the one difference is the four-gluon contact graph
  (MadGraph's diagram 16, `AMP(16..18)`), whose three colour structures come in
  reverse order — ours `(s₀, s₁, s₂)` are MadGraph's `(s₂, s₁, s₀)`, every
  coefficient equal. The oracle sorts graphs by their normalised columns and
  pairs them by position; with the structures in another order and the unit
  taken from the first column, the contact graph sorted elsewhere on the two
  sides and the pairing slipped, which is what the message reported.
- **Why the order is a labelling.** MadGraph calls
  `VVVV1P0_1(W(1,1), W(1,2), W(1,5), …)`: the off-shell gluon sits in the
  vertex's first slot and the three external gluons follow. A slot order with
  the off-shell gluon last instead maps the colour structures
  `{l₁l₂}{l₅g*}`, `{l₁l₅}{l₂g*}`, `{l₁g*}{l₂l₅}` onto MadGraph's third, second
  and first, which is the reversal observed, and maps the Lorentz structures
  `VVVV1`, `VVVV3`, `VVVV4` the same way, so the vertex is unchanged. This
  crate's slot order for the contact was not read out; the reversal is what
  such a relabelling gives, and nothing in the columns distinguishes it from
  one. What must hold is that each colour structure multiplies its own Lorentz
  structure, which this oracle
  cannot see and the per-flow amplitude gates do: `amplitude_oracle`'s
  `gg_to_gg`, and `standalone_jamps`' `uux_to_ggg` and `gg_to_ggg`, whose
  contact vertex has an internal leg exactly as here. The only other four-gluon
  contacts in the banked runs are the three `g g > g g` subprocesses
  (`gg_to_gg`, `gg_to_gg_cg`, `pp_to_jj`'s `P1_gg_gg`), whose four legs are all
  external, and there the two orders agree.
- **Fix** (`color_cf_oracle.rs`): `normalise_group` returns a graph's columns
  in a form independent of the order its structures came in (each non-zero
  column's leading phase tried as the graph's single unit, the smallest sorted
  form kept), so the comparison is of the set of structures under one unit:
  a sign on one structure, a missing or doubled structure, or a coefficient
  moved to another flow still fails. A new trial,
  `jamp-normalisation/structure-controls`, pins that on the contact graph's own
  columns (reversed and rotated by −i: equal, unit ratio −i; one structure
  negated: different; one coefficient moved: different). After it the oracle
  passes 97/97, `P1_gg_ttxg` with the colour matrix exact and "JAMP 18 columns
  over 16 graphs max_rel=0.00e0 (2 sign-flipped)".
- **What is left open.** On `g g > t t~ g` itself no gate compares
  amplitudes: the pairing is pinned on the crossing-related `u u~ > g g g` and
  `g g > g g g`, not on this subprocess. A `standalone_jamps` table for
  `g g > t t~ g` would pin it directly (validation backlog).

The second run got past the colour oracle and stopped in `validate_alphas`
(clippy also failed, on the new code: a `type_complexity` lint, fixed by using
the file's own `NormalisedGraph` alias). Both of its tests assert that every
banked run on disk is declared in an inventory, and M0 had declared the MLM
rows in `validate_scales` (`MatchedDump`) but not here: "pp_to_ll_0j2j_mlm:
alpha_s source arm disagrees with GRID_ALPHA_S_RUNS". The five rows run at
`pdlabel = lhapdf`, so they now sit in `GRID_ALPHA_S_RUNS`, and the run logs
pin the source on all five (0.118 → 0.13000271085472234, reproduced from the
set's 51 knots to 0, the two sources 240054 printed half-digits apart). The
events oracle then measured where `AQCDUP` is reproduced from `SCALUP`:
10000/10000 on `pp_to_llj_mlm`, `_alps2` and `_xqcut_only` (worst 0.995–0.997
of the printing budget), which join `SCALUP_IS_THE_RENORMALISATION_SCALE`;
16/10000 outside on `pp_to_ll_0j2j_mlm` and 254/10000 on `pp_to_ttx_0j1j_mlm`
(up to 10⁵ budgets), which stay outside: under `ickkw = 1` `SCALUP` is
`√max(q2bck)` and `AQCDUP` is at the first call's `μR`, and the mixed rows are
where those part. Their `AQCDUP` is gated at `μR` by `validate_mlm_dumps`
(worst 3e-16). After both: "AQCDUP: 540000 events across 54 runs within their
printing budget, worst 0.999 of budget". As with `validate_scales`, an
unbundled row is skipped when absent (`present`), so `refdata-8` checkouts are
unaffected.

The targets after it, run on their own first (no fail-fast) so a third failure
would not cost another full pass, all passed: `validate_hadronic` 14 (275 s),
`validate_helas` 1, `validate_lhef` 3 (every banked file byte-identical,
the five MLM `run_01` among them: 10000 events each, 1–3 process entries),
`validate_madgraph_diagrams` 57, `validate_pdf_grid` 20, `validate_samples` 6,
`validate_scale_couplings` 1, `validate_scales` 10 (the MLM rows declined as
`MatchedDump`), `validate_sigma` 7, `validate_unweighting` 1, `validate_vegas`
3; `validate_kt_cluster` and `validate_mlm_dumps` are all `#[ignore]`.

The third run, on the final tree: `pixi run --skip-deps validate` (tree `37c4490` plus this record's
documentation edits), clippy on the extended-validation targets and the
collator included: exit 0. Clippy clean; 59 test targets, 1698 passed, 0
failed, 81 ignored (the heaviest, `validate_samples_proton`, 640 s; 37 minutes
in all on four cores). The
collator: 64 rows × 4 categories = 256 cells, **203 measured (196 ✅, 7 ⚠️),
11 ⏳, 42 covered-by or uncovered**, "the measured cells are exactly the cells
the manifest declares, every gate cell passed". The seven ⚠️ are the standing
ones (the `gg_to_gg` and `gg_to_gg_cg` diagram-count conventions,
`gg_to_gg_cg`'s σ, the `ee_to_wpwm_cw`, `ee_to_zh_smeft` and
`wpwm_to_wpwmz_cw` amplitude cells, `ee_to_mumua`'s samples). Against Z2's
59 rows the five new rows add 20 cells and no measured one: their 7 long-tier
cells render ⏳ (the 2 → 6 rows give the other 4) and 13 `uncovered`. The MLM
rows, cell by cell:

| row | diagrams | amplitudes | integrals | samples |
|---|---|---|---|---|
| `pp_to_llj_mlm` | uncovered | uncovered | ⏳ oracle layer (`validate-mlm-sigma`, info) | ⏳ oracle layer (`validate-mlm-dumps`, info) |
| `pp_to_llj_xqcut_only` | uncovered | uncovered | ⏳ oracle layer (info) | uncovered |
| `pp_to_llj_mlm_alps2` | uncovered | uncovered | ⏳ oracle layer (info) | ⏳ oracle layer (info) |
| `pp_to_ttx_0j1j_mlm` | uncovered | uncovered | uncovered | ⏳ oracle layer (info) |
| `pp_to_ll_0j2j_mlm` | uncovered | uncovered | uncovered | ⏳ oracle layer (info) |

The `mlm-pythia` standalone renders as owned by the oracle layer. None of the
long-tier MLM tasks was run here (no production code changed); their last
record is P12's (`validate-mlm-sigma`, `validate-mlm-dumps`) and C's
(`validate-mlm-pythia`).

**`@2`'s reference from independent directories** (`mlm_sigma_reference.json`,
`pp_to_ll_0j2j_mlm`, now `independent_directories: true`). The runs are the
samples-grade `run_01` (seed 20260928, the first run of its own freshly
generated directory) and the twenty fresh directories M5 (20261101–04) and C
(20261105–20) generated from M0's proc lines and the committed run card, one
directory each, `nb_core = 2`, `vector_size = 1` in every banner. Each run's
σ and error are its `results.dat` (as logged by the run), and `by_lprup` its
own `<init>` block. M0's nine shared-directory seeds moved to
`shared_directory_runs`, which nothing reads. Read under the seed policy
(inverse-variance mean, error max(quoted, spread/√n)), pb:

| | 21 independent runs (the reference) | the 20 fresh only | M0's 9 shared-directory seeds | M0's 10 (the old reference) |
|---|---|---|---|---|
| `@0` | 665.001 ± 0.348 (χ²/dof 0.60) | 665.015 ± 0.356 | 664.817 ± 0.547 | 664.805 ± 0.519 |
| `@1` | 267.874 ± 0.370 (0.77) | 267.851 ± 0.380 | 269.121 ± 0.559 | 269.037 ± 0.529 |
| `@2` | **130.931 ± 0.203** (1.24) | 130.954 ± 0.212 | 130.489 ± 0.274 | 130.487 ± 0.260 |
| total | 1063.662 ± 0.547 (0.62) | 1063.670 ± 0.560 | 1064.423 ± 0.836 | 1064.329 ± 0.792 |

- **`@2` is the one part whose directories scatter beyond their quotes**:
  sd 0.93 pb (0.71 % per run) against 0.84 pb quoted, χ²/dof 1.24, so its
  error is the spread, 0.203 pb against 0.177 quoted. That is the
  inter-directory spread D2 asked the tolerance to include, now measured on
  21 directories rather than 7, and it enters the reference error itself.
- **Cross-check against D2's 131.09 ± 0.11** (seven fresh `p p > e+ e- j j`
  directories): −0.16 ± 0.23 pb, −0.7σ. D2's four fresh mixed-card
  directories read 130.81 ± 0.28.
- **The reference moved**: `@2` +0.44 pb (+0.34 %), `@1` −1.16 pb (−0.43 %,
  −1.8σ in the two errors combined), the total −0.67 pb. The shared
  directory read `@1` high as well as `@2` low; its seeds' χ²/dof of 0.4–0.7
  was the sign that they were not independent draws.
- For orientation only (hand sweeps, not a gate): F-B's ten vibegraph seeds
  at `--fixed-budget --allocate neyman --neval 200000 --niter 8` read `@0`
  665.26 ± 0.18, `@1` 268.59 ± 0.41, `@2` 131.26 ± 0.66 and the total
  1065.11 ± 0.61, which against this reference is +0.04 %, +0.27 %, +0.25 %
  and +0.14 % (+0.7σ, +1.3σ, +0.5σ, +1.8σ). `@2`'s +0.25 ± 0.53 % is
  consistent both with no difference and with D2's budget of about +0.7 %
  (H1 +0.25 %, the generic 2 → 4 offset ≈ +0.45 %); ten seeds at this budget
  cannot tell the two apart.
- **Why 21 and not 20.** The brief named the twenty fresh event files; the
  samples-grade run is itself the first run of a freshly generated directory,
  independent of every other, and the bundle carries it either way. Leaving it
  out would make the reference's runs and the banked runs differ by one. The
  twenty alone read 130.954 ± 0.212 (column 2).

**How it was written, and how B1 reproduces it.** The JSON writer moved out of
`gen_mlm_references.sh` into `write_mlm_sigma_reference.py`, which knows three
kinds of run: `samples`, `fresh` (the only run of its own directory, banked as
`output/<row>/Events/run_s<seed>`) and `sigma` (a shared-directory seed). A row
with fresh runs is written with `independent_directories: true`, its `runs`
the samples and fresh runs and its shared seeds in `shared_directory_runs`;
every other row with `independent_directories: false` and its layout
unchanged; banked runs name their event file (`events`). Fed M0's own seed
records (`output/<row>/Events/run_01` and `mlm-m0`'s
`work/mlm/<row>/Events/run_s*`), it reproduces all 50 committed runs of the
five rows field for field; the twenty fresh runs come from the logged
`results.dat` values and the event files' `<init>` blocks. The generator
gained `FRESH_ROWS` (default `pp_to_ll_0j2j_mlm`) and `FRESH_SEEDS` (default
20261101–20): each such seed is generated into `work/mlm/<row>_fresh/<seed>`,
run, banner-checked, its event file, banner and seed record copied into
`output/<row>/Events/run_s<seed>`, and the directory dropped; a banked seed
whose record matches the card it would run with is read back. Dry run with a
stubbed MadGraph (scratchpad `mlm-z/dry`): a first pass generated the samples
directory, the shared directory with one seed and two fresh directories (four
runs, the fresh directories dropped and their files banked); a second pass ran
nothing and read all four back; an edited run card re-ran the
samples seed and the fresh seed it named. The replay (`gen_kt_cluster_dumps.sh`)
reads only `run_01`, so the extra runs beside it change nothing there, and the
`refs` extractors skip the MLM rows.

**Side A of the matched Pythia comparison: banked.** C's recommendation is
adopted. The twenty fresh event files (~2 MB each) go into `refdata-9` beside
`run_01`, so the comparison C measured — 21 MadEvent files, every `@N`
acceptance resolved to 0.3–0.5 % a side — runs from the bundle without
MadGraph, and the σ reference's runs are auditable from it. `mlm_match.py`'s
default side A is now those 21 files, read from the reference's `events`
fields (it stops naming the missing file and where it comes from when they are
absent, as on `refdata-8`), its default Pythia seeds ten (20261201–10), and
`generate_mlm_samples.sh` makes twenty vibegraph samples (20260928–47) by
default, C's configuration. The comparison stays `info`.

**Registration** (`validation/manifest.toml`). The five rows keep
`status = "planned"` and `bundled = false`; `pp_to_ll_0j2j_mlm`'s artifacts
name the twenty `run_s*` runs. Corrected notes: the `diagrams` cells of the two
mixed rows (mixed multiplicities integrate since M3; what a cell needs is
per-`@N` counts), both mixed rows' `integrals` cells (the target gate does not
exist yet, the new reference with its numbers, the cross-check, and on
`pp_to_ttx_0j1j_mlm` that this side's σ has never been measured), the MLM
section header (independent directories; B1's one command), and the
`mlm-pythia` standalone's inputs. No cell changed tier or mode: every one the
long-tier tasks measure is `info` and the rest `uncovered`. The manifest was
final before the third run, so its collator read it.

**Loose ends.**
- M5's two warnings in a release lib build without features: `cross_check_node`
  is called only from `validate_arenas`' debug / `extended-validation` block
  and from a test helper, and `MultivectorWf` is used only in that block.
  Both are now behind the same `cfg` (`any(test, debug_assertions,
  feature = "extended-validation")` for the function), and so are the two
  imports only it reads (`NodeAnalysis`, `NodeId`), which warned once it was
  gated. `cargo check -p vibegraph-lib --release`, and with `--lib --tests`,
  in a scratch target: no warnings.
- `CARGO_PROFILE_RELEASE_DEBUG_DEBUG` cannot work: cargo reads it as a key
  under `profile.release` and fails ("could not load config key
  `profile.release` … invalid type: Option value") whatever its value. The
  `extended-validation` skill now says so and gives the two ways that work
  (`--config 'profile.release-debug.debug=0'` on a cargo line; an untracked
  `.cargo/config.toml` for `validate.sh` and pixi tasks, which take no cargo
  flags). This run used the latter.

**Docs.** README's scope and the guide's pipeline and hadronic chapters still
said matching was refused or out of scope; they now say what this sprint built
(the matrix-element half, the veto left to the shower). `mdbook` is not
installed here, so the guide was not built.

**Bookkeeping.** `TODO.md`: current position "waiting on `refdata-9`", `mlm`
in the closed-sprint history in three lines, the feature entry collapsed to a
pointer with the refusals that remain, the pipeline table, an "Open findings
from `mlm`" section in the validation backlog and five items at the head of the
performance backlog (Z.5). The performance backlog's head held a fragment of an
older MLM entry with its first line lost; it is replaced by those items.
The `extended-validation` skill: the `mlm` stage and an MLM row in the gate map.

**Gates** (final tree, `CARGO_TARGET_DIR` private, debug info off):
- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean; with both
  `extended-validation` features: clean (inside the third run).
- `cargo test --workspace`: 33 binaries, 1297 passed, 0 failed, 15 ignored.
- `pixi run --skip-deps validate`: exit 0, as above.

**Corrections to the brief.**
- "The gate reads mean ± max(quoted, spread/√n)": no gate reads
  `pp_to_ll_0j2j_mlm`'s σ yet. The only seeded MLM σ gate
  (`mlm_sigma_row`) runs the three single-multiplicity llj rows. The JSON now
  carries what the mixed rows' gate needs, and Z2 writes it (Z.4).
- "Twenty … independent directories": twenty-one, with the samples-grade run
  (above).
- The MLM dumps cannot make `validate_mlm_dumps` banked by being bundled: it
  also reads each process directory's `configs.inc`, `config_nqcd.inc` and
  `config_subproc_map.inc`, which the bundle does not carry. They stay
  oracle-layer, regenerated by B1.
- The full banked layer exercises the MLM rows only through what is not
  `#[ignore]`: `validate_lhef` round-trips their five `run_01` files,
  `validate_scales` declares them, and `cli_generate_proton` runs the matched
  and mixed cards end to end. Every MLM comparison against MadEvent is a
  long-tier task.

#### B1 Landed: `refdata-9`, 2026-10-03

Bank host: Apple M3 Max (16 cores), macOS 15.7 (Darwin 24.6), MadGraph 3.7.1
at `b7687064` through `mg5_pinned.sh`, pixi `madgraph` environment. Worktree
`vibegraph-wt/b1` at `e9342ee`, the work area unpacked from `refdata-8`.

**Generate.** `pixi run -e madgraph generate-references deps madgraph seeds
mlm refs`: 415 min wall. The twenty-directory loop and the gzip-through-a-
named-pipe replay ran on macOS for the first time without a finding; every
instrumented replay reproduced its `run_01` byte for byte. Two generator
findings, both in `refs`, fixed in `30267de`:
- `refs` needs files the bundle trims from the process directories
  (`coloramps.inc` and the rest of a buildable `SubProcesses`), so it cannot
  run on a work area unpacked from the bundle alone. Here they were filled in,
  overwriting nothing (`cp -Rcn`), from a full `refdata-8` work area.
- The amplitude-table generator reads a probe module for every row
  `gen_amplitude_tables.py` emits, including `uux_to_mumu`, which borrows
  `pp_to_ll_qcd0`'s `mg_amplitude` and so was never built by
  `build_amplitude.sh` (`ModuleNotFoundError: mg_amp_probe_uux_to_mumu`).
  `build_amplitude.sh` now builds a probe for each key
  `gen_amplitude_tables.py --dump-keys` prints. Two bash-3.2 faults surfaced
  on the narrowed rerun and are fixed with it: the empty-generic-set guard and
  an unbound empty array.

**Reproduction check** (committed in `83f34fd`).
- `mlm_sigma_reference.json`: every run within its seed error of the committed
  value (|pull| < 0.5). `pp_to_llj_xqcut_only` and `pp_to_llj_mlm_alps2`
  10/10 bit-equal, `pp_to_llj_mlm` 9/10; `pp_to_ll_0j2j_mlm` and
  `pp_to_ttx_0j1j_mlm` are not bit-equal across hosts.
- `mlm_census.json`: not exactly equal, against Z.3's "exactly" — its event
  counts come from the two rows whose MadEvent runs are not bit-reproducible
  across hosts. Its findings hold: no mixed jet-ness, every channel's memo
  single-valued, no restricted re-cluster on t t̄, no stale final-state
  `ipdgcl`. `mlm_dump_manifest.json` likewise differs beyond `sha256` and the
  shard names on those rows; committed so it pins the dumps this host gated.
- `grammar_sigma_reference.json`, `decay_chain_sigma_reference.json`: the
  `seeds` stage reran them, against Z.3's "do nothing new", because their work
  areas are not banked; every value bit-identical, `wall_s` only. The same
  stage rebuilt `dy13_default` and `dy13_mmll_60_120` (event bodies
  byte-identical, banners and logs not); their `refdata-8` bytes were restored
  before assembling.
- Every other committed table regenerated byte-identical.

**Gates on the host.**
- `pixi run --skip-deps validate`: exit 0, 175 min; 59 test targets, 1776
  passed, 0 failed, 81 ignored; collator "the measured cells are the declared
  cells".
- `validate-mlm-dumps`: 3/3. `validate-mlm-samples`: 1/1 (the Z on 0.9547
  against 0.9507, pull +0.89; a jet at the collider energy 0.1070 against
  0.1123, −0.81). `validate-mlm-sigma`: 3/3, pulls +0.23 (`xqcut_only`, five
  seeds, χ²/dof 0.82), +0.86 (`mlm`, ten, 0.86), +1.86 (`alps2`, five, 1.04).
- The optional Pythia step was not run.

**Assembled and published.** `vibegraph-refdata-9.tar.zst`, 4245 files,
166043012 bytes, sha256 `4183504c…fc341f`, assembled in 8.5 min: `refdata-8`'s
3824 members, each byte-identical to the `refdata-8` archive's own, plus 421
MLM files (five `run_01` and twenty `run_s*`). Published as the `refdata-9`
release asset; the downloaded asset hashes to the pin. The five rows lost
`status = "planned"` and `bundled = false`, and the collator still reads the
declared cells.

The MLM directories' proc cards print MadGraph's "UNKNOWN DEVELOPMENT VERSION"
banner: the submodule was copied into the worktree without its `.git`. The copy
is the pinned commit, every source file and `VERSION` (3.7.1); the manifest's
cut-9 comment says so. A future bank host should check out the submodule rather
than copy it.

#### Z2 Landed: re-verified from `refdata-9`, 2026-10-03

On the bank host. The sprint is closed.

**From the published bundle.** A fresh worktree (`vibegraph-wt/z2`) whose
`validation/madgraph/output` came only from `VIBEGRAPH_FETCH_CONSENT=1 pixi run
fetch-refdata` (the release download, hash-verified, 4245 files). `pixi run
--skip-deps validate` at `590f87a`: exit 0, 13 min on a quiet host; 59 targets,
1698 passed, 0 failed, 83 ignored; "203 measured (196 ✅, 7 ⚠️, 13 ⏳, 40
uncovered)", "the measured cells are the declared cells". B1's 1776 is the
same suite: its work area also held the instrumented replays' copies of the
five MLM process directories under `output/ktdump/`, so `color_cf_oracle` and
`color_flow_tags_oracle` ran each of the 78 MLM subprocess trials twice; on the
bundle every one runs once. A first run of the same worktree at `26e8048`
aborted in `validate_hadronic` (SIGABRT, no panic message, 44 targets and 1574
tests passed before it) while a σ session loaded the host to a load average of
~115 on 16 cores; the target alone then passed 14/14 on the same data, and the
full run above is clean. That is note 36's unattributed malloc abort under
load, seen once more.

**The mixed rows' σ gate** (`590f87a`, validation-dev session):
`sigma_ll_0j2j_mlm_vs_madevent` (gate) and `sigma_ttx_0j1j_mlm_vs_madevent`
(info) in `validate_hadronic`, run by `validate-mlm-sigma`. Each builds the
composite as `vibegraph integrate` does (process lines from the row's script,
`split_by_multiplicity`, the on-shell vetoes, union-shape maps,
`MultiplicitySum`, the α survey, then `--fixed-budget --allocate neyman
--neval 200000 --niter 8`), ten seeds (20260928–37), and writes one collator
cell per `@N` plus the total. The first seed reproduces the CLI to every printed
digit on both rows. Both sides read mean ± max(quoted, spread/√n) (MadEvent's
the inverse-variance mean, this side's the unweighted one); a gating cell needs
|pull| < 3 and this side's seed χ²/dof inside its 0.1–99.9 % band.
`mlm_sigma_row` (the llj rows) now applies the same rule and asserts when it
gates. `pixi run -e madgraph --skip-deps validate-mlm-sigma`, exit 0, 9.5 min
(bit-identical on a second run):

| row | vibegraph (pb) | MadEvent (pb) | pull | rel | χ²/dof |
|---|---|---|---|---|---|
| `pp_to_llj_xqcut_only` | 212.608 ± 0.126 (5) | 212.549 ± 0.229 | +0.23 | +0.03 % | 0.82 |
| `pp_to_llj_mlm` | 268.433 ± 0.119 (10) | 268.169 ± 0.284 | +0.86 | +0.10 % | 0.86 |
| `pp_to_llj_mlm_alps2` | 241.255 ± 0.154 (5) | 240.710 ± 0.252 | +1.85 | +0.23 % | 1.04 |
| `pp_to_ll_0j2j_mlm` `@0` | 665.258 ± 0.217 | 665.001 ± 0.348 (21) | +0.63 | +0.04 % | 0.66 |
| `pp_to_ll_0j2j_mlm` `@1` | 268.594 ± 0.451 | 267.874 ± 0.370 | +1.23 | +0.27 % | 0.95 |
| `pp_to_ll_0j2j_mlm` `@2` | 131.275 ± 0.663 | 130.908 ± 0.195 | +0.53 | +0.28 % | 1.98 |
| `pp_to_ll_0j2j_mlm` total | 1065.126 ± 0.684 | 1063.646 ± 0.546 | +1.69 | +0.14 % | 1.06 |
| `pp_to_ttx_0j1j_mlm` `@0` (info) | 513.222 ± 0.177 | 512.898 ± 0.181 | +1.28 | +0.06 % | 2.01 |
| `pp_to_ttx_0j1j_mlm` `@1` (info) | 583.192 ± 0.297 | 575.836 ± 0.777 | **+8.85** | **+1.28 %** | 0.99 |
| `pp_to_ttx_0j1j_mlm` total (info) | 1096.413 ± 0.381 | 1088.640 ± 0.797 | +8.80 | +0.71 % | 1.42 |

`@2`'s error is its seed spread, 0.66 pb, as Z.4 expected, so the registered
deviations (H1, the generic 2 → 4 offset) need no allowance at this budget.
A mixed-row seed takes ~35 s on this host; the multi-thousand-second figures
earlier in this note were a loaded four-core container.

**t t̄ `@1` is +1.28 % high, undiagnosed.** All ten seeds (581.9–584.5 pb) sit
above MadEvent's whole range (572.5–579.7) while `@0` agrees. Not yet
separated: the reference's shared directory (nine of ten seeds; on `0j2j` the
shared directory moved `@1` by −0.43 %, the other direction) and H1's
first-call rejections, unmeasured on this row. The cell stays `info`, and the
five-seed agreement Z.4 asks before a gate cannot be had until it is
diagnosed. Filed in the validation backlog.

**Dumps** (`validate-mlm-dumps`, against B1's regenerated dumps): 3/3. The new
dumps move Z1's counts: `pp_to_ttx_0j1j_mlm` 6256 non-permuted events agree and
3744 permuted (3735 before), every one agreeing on every field, 0 with other
first-call scales (2 before); `pp_to_ll_0j2j_mlm` 9840 non-permuted agree, 160
permuted (141 before), 83 agreeing and 77 with other first-call scales, all
`@2` (58 `P2_qq_llqq`, 19 `P2_gg_llqq`), weight-factor ratio 0.67–1.60, mean
0.996.

**Flips** (each note carries the 2026-10-03 measurement): the three llj rows'
integrals long/info → long/gate; `pp_to_ll_0j2j_mlm` integrals uncovered →
long/gate; `pp_to_ttx_0j1j_mlm` integrals uncovered → long/info; the samples
cells of `pp_to_llj_mlm`, `_alps2`, `pp_to_ll_0j2j_mlm` and
`pp_to_ttx_0j1j_mlm` long/info → long/gate (permuted-`P1` events stay info,
which `validate_mlm_dumps` already enforced). Those four samples cells render
⏳: the dump test writes no collator row, and the schema has no per-event
samples kind to write. Nothing else flipped. After `validate-mlm-sigma`, the
collator reads 208 measured (200 ✅, 8 ⚠️, 8 ⏳, 40 uncovered), the declared
cells.

**Corrections to Z.4.** `pp_to_ll_0j2j_mlm`'s reference after B1 is `@2`
130.908 ± 0.195 and total 1063.646 ± 0.546 (Z.4 and the Z1 record quote
Z1's 130.931 ± 0.203 and 1063.662); the gate reads the JSON. The permuted-event
counts above supersede "141". Under "both policy errors" the llj rows now take
this side's error as max(quoted, spread/√n), which moves `alps2` from +1.86 to
+1.85.

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
