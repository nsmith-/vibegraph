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
