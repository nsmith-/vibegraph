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
