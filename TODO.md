# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate.

**Current position**: the **performance sprint** ✅ **closed** 2026-08-05,
eleven sessions over three tracks — I (integration engine), P (PDF hot path),
E (evaluator) — all merged, `main` @ `ca53336`. Measured on one host in one
sitting against the note-30 baseline: `pixi run --skip-deps validate`
**691 s → 391 s (−43.4%)** on the identical command with the census
cell-for-cell unchanged (**98 measured / 96 ✅ / 2 ⚠️ / 4 ⏳**); per-row
single-thread **integrals 842.6 s → 389.8 s (−53.7%)**; integrand throughput
against MadGraph on note 30 §5.3's CPU-time denominators **geomean
6.84× → 8.76×** over the same 26 rows; the per-point MATRIX1 comparison
**1.25× → 0.98×**, the evaluator itself −21.6%, with **8 of 14** processes now
faster per point than MadGraph (was 3); and `integrate` scales **4.70×** on
`dy13_default` and **5.36×** on `pp_to_llj` from `-j 1` to `-j 16` at a
byte-identical artifact, which retires note 30 §1's "MadGraph is a 16-way
parallel job farm and our integrator is one thread" caveat. Every bit-for-bit claim was checked at
the layer's own resolution — the 100 banked category row files, digests
compared — and no tolerance was relaxed anywhere in the sprint. Full record:
note 31 §6; the sessions are one line each in the closed-sprint history below.
Standing caveats: a partonic σ quoted from `refdata-2` is **not comparable** to
one from `refdata-3`/`refdata-4`/`refdata-5` (MadGraph 3.5.7 applied the PDF
set's `αs(M_Z) = 0.130` to `lpp = 0` runs; 3.7.1 keeps the model's `0.118` —
note 27 §B5); and the four re-carded runs' σ are **not comparable** across the
`refdata-4`→`refdata-5` boundary (MG-internal `nn23lo1` vs LHAPDF NNPDF2.3-QED
are different densities — `p p > b b~` moves −9.8%).

**Scope decision (user, 2026-08-02)**: the release goal is restricted to
**arbitrary fixed-order Standard Model processes** over unpolarized
proton–proton or fixed-energy partonic beams, without decay-chain syntax.
Every extension beyond that — BSM UFO support, other beam configurations,
polarization, decay chains — is explicitly descoped to the feature backlog
(see "Descoped from v1" below), and every descoped surface a card can still
reach must be a **hard error**, never a silent acceptance; the fixes closing
the remaining silent acceptances are validation-sprint items.

**Addendum sprint ✅ closed (2026-08-05, note 32)**, eight of nine sessions
merged (S9 killed clean, below), `main` @ `225657a`. Wave 1: S1 landed E3b
(cut-before-draw on the fixed-beam path, a bit-identical dead-work skip for
every accepted point) and I5 (`combine_seeds` unweighted in
`validate_hadronic.rs`, matching what I1 already did to VEGAS's own iteration
combination — every hadronic row moved ≤0.02%); S2 replaced the never-converging
extremum `w_max` with MadGraph's own `unwgt.f` truncation-ladder rule (not a
percentile — the session corrected its own brief on the mechanism), lifting the
five gating rows' acceptance from 22.2/20.6/23.3/10.6/4.21% to
54.1/52.1/52.9/38.9/9.98% at matched budgets, `p p > l+ l- j` **4.36×** cheaper
per effective unweighted event; S3 parallelised the α-survey (deterministic
chunking, bit-identical at `-j {1,4,16}`, asserted) and made `--target-rel`
convergence the CLI default (`--fixed-budget` for a reproducible fixed spend),
correcting note 32 §1.1's serial-floor decomposition en route (the survey was
41–52% of the `-j 16` wall, not the ~27% first quoted) — `-j 16` on
`dy13_default`/`pp_to_llj` at `--fixed-budget --neval 120000 --niter 12` now
reads **8.7×/9.5×** (min of 5 rounds, this session's re-measurement, host load
6–19 from background indexing), byte-identical artifacts asserted; S4 closed
all three `mg_perf_compare` findings (manifest-driven `gen_amplitude.py`
registry, byte-identical migration; host-labelled committed `mg_timings.json`;
the bench widened 14→19 rows) and found the omitted five rows were QCD-dense,
inverting the "we're already ahead" reading — the 19-row MATRIX1 geomean sits
worse than the 14-row one, by design, not regression; S5 (scoped as E4
allocator traffic) landed the merge-table hoist out of `ScaleChoice::cluster_scales`
(**−16.9% to −22.3%** ns/point on the three clustered-scale rows,
`validate_unweighting` **−16.8%**, the partonic σ gate **−11.1%**,
byte-identical); S9 (restore the pre-`3dab3a1` scalar packed-complex codegen)
was **killed clean** — the packed idiom is x86-specific, an in-house `MulAdd`
trait forcing it on this ARM (M3 Max) host cost **8–9%**, the opposite of a
win, so the pre-registered kill criterion fired and nothing merged (worktree
clean, branch carries no commit beyond the note-32 planning doc; the design
stays at note 32 §2 S9 for an x86 host to pick up). Wave 2: S6 sized five
hadronic σ budgets to reference precision under a ladder+sweep license
(`pp_to_jj`/`pp_to_bb_fixed`/`pp_to_bb`/`pp_to_bb_qcd2`/`pp_to_ll_scalefact2`
300k→75k; llj stays at 150k on both counts — its ladders are flat but the 75k
rung's seed scatter is not, and the re-carded `pp_to_llj` ladder still climbs
monotonically 0.04%→0.21% end to end, so no further cut is licensed there);
S7 turned the 2→6 rows on as `info` at the long tier — the "~1 ms/eval" premise
was stale by more than an order of magnitude (the gate's own harness reads
64/71 µs), but the real blocker is `MIN_CHANNEL_NEVAL` × channel count
(512 × 579/615 diagrams = 296 448/314 880 evaluations per iteration whatever
budget is asked) and a heavy multichannel tail whose single-seed pulls do not
shrink with budget (+4.8%/−4.5%/+3.5% at both ends of a 300k–1.2M ladder while
the five-seed mean holds inside 1.1% of a 0.30%-precision reference) — census
98→**100** measured, no existing cell moved; S8 (this session) re-recorded
every stale σ/percentile/cost figure the wave-1/2 sessions left behind and
took the addendum's own close-out measurements. Full record: note 32 §5.
Planning-time findings recorded in note 32 §1: the `-j 16` 4.7–5.4× ceiling was
a ~70–75% serial floor at 16 threads (S3 closed most of it); and
`gen_amplitude.py` bypassed the manifest while `mg_timings.json` reached no
artifact and carried no host identity (S4 closed both).

**Next action — the user's**: the first release tag is **`v0.1`**, decided
2026-08-02 and re-affirmed 2026-08-03 to follow the **performance sprint**.
That sprint and the addendum above are both closed now, so nothing is left
blocking the tag from either direction — it was never more than a cleanup, not
a correctness sprint. A 0.x line because no global backwards-compatibility
promise is made yet; a future "quality sprint" tightening the `pub` API surface
(backlog below) precedes any 1.0. Tagging runs `release.yml` and
`acceptance.yml` for the first time.

**Standing measurement facts** (note 30 baseline, note 31 §6 close-out; all
one host, M3 Max). The layer's own run-to-run spread is **0.8% median / 3.4%
worst** on rows above 1 s, so a sub-1% claim is not measurable there. Two
readings of note 30 §3.2 are **corrected** by note 31 §6.2. Its `diagrams`
column is not per-row work: `sm_model()` is process-wide interned, so under
default test parallelism the 26 rows race its lazy initialisation and each
charges itself the contention — run one at a time the category is **1.29 s**,
not 14.2 s. Its `amplitudes` column, by contrast, **is** real work and
reproduces under either protocol; only the sentence "never builds an evaluator"
is wrong, since that gate runs enumeration and `AmplitudeEvaluator::compile` per
row — which is exactly why its two 2→6 rows cost ~1.1 s while every other row is
≤ 0.02 s. Chain B's per-point configuration draw was **≈1.0 µs/point
(21%)** on `gu_to_epemu`/`gux_to_epemux` at the baseline and is **≈870 ns
(≈18%)** after the evaluator sessions (note 31 §E3). PDF interpolation, which
note 30's profiles put at 14–19% of self time wherever there are protons, is
now **1.4–2.0%** (note 31 §2.4); the evaluator is correspondingly **62–79%**,
with `fill_arenas` alone 33–43%, and is where any further broad win has to come
from.

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 19 rows agree with MadGraph at ≤5.9e-13 on the fixed grid (`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) and at ≤6e-14 on MadGraph's own banked events — except the two `ee_to_mumu_tata_qcd0` events near the Higgs pole, where the point's own one-ulp conditioning exceeds the deviation. Beneath \|M\|²: per-diagram `c_i·AMP(i)` on every single-flow row with ≤64 diagrams, per-flow `JAMP()` on all 19, one fitted constant `G = ±i` serving both |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS (two-phase `adapt`/`sample_frozen` serde object, deterministic rayon chunking, one grid **per channel**) + 2-body LIPS + massive RAMBO generic over `F: Real` with splittable `ChaCha8` substreams + MadGraph-style multichannel (per-diagram propagator-pole channel trees, BW/t-channel/massless-log maps, variance-minimising weight, α-adaptation), rebuilt per event ŝ at proton beams with the t-channel draw floored by `Cuts::spacelike_floor()`. The multi-rung t-channel spine and the per-subprocess identical-particle factor are in production (`kt-spine` Track S, note 28) |
| 5 | Cross-section integration + running couplings | ✅ Done | Leptonic `sigma_z_pole`/`sigma_qed_limit`; hadronic σ(pp→e⁺e⁻) via pure-Rust LHAPDF6 parser + log-bicubic interp and compiled MG run-card cuts, vs MG 0.14%/0.07%; MG's `αs` RGE + per-event `μR`/per-beam `μF` (`coupling/`); `vibegraph integrate` persists per-channel VEGAS grids in `IntegrateArtifact` (fv5: model identity + a per-channel subsampler summary). `lpp = 1` over an **arbitrary** process via `ProtonIntegrand` — measured flavour groups (pointwise \|M\|² + masses + `Cuts` + colour basis), both beam orderings by outgoing-leg reflection, `αs` off the PDF grid. σ gates: 17 partonic GATE rows incl. the 3 QCD 2→2s, `pp_to_bb_fixed` and all 4 llj subprocesses at the kT-clustered per-event scale, σ(pp→e⁺e⁻) on both dy13 cards through the *general* path (**933.905 ± 0.567** vs MG 933.230 ± 0.480; **644.203 ± 0.384** vs 644.330 ± 0.283), and σ(pp→ℓ⁺ℓ⁻j) fixed-scale **424.428 ± 0.432 pb** over three seeds vs MG 423.840 ± 1.518 (pull +0.37). At a *dynamical* scale each point's cluster scale is taken in the integration configuration drawn from the point's own squared amplitudes (`∝ AMP2_c/Σ AMP2`, MadEvent's enhancement-weight conditional, note 29 chain B): `gu_to_epemu` **+0.029%** (pull +0.13) / `gux_to_epemux` **−0.165%** (pull −0.70) and σ(pp→ℓ⁺ℓ⁻j) **+0.25%** (pull +0.73), all GATE at `rel_tol` 0.005 set by the references' own errors. The four `refdata-5` re-carded rows gate on the same path at their own reference-precision-sized budgets (addendum S6, note 32): `pp_to_bb` +0.05% at 75k, `pp_to_bb_qcd2` +0.01% at 75k, `pp_to_llj` +0.18% at 150k (its ladder still climbs monotonically across 75k–600k, 0.04%→0.21%, which is why it did not cut to 75k with the other three), `pp_to_ll_scalefact2` +0.02% at 75k. The `p p > j j` capstone runs the same path on the canonical QCD process and is **GATE**: **6.813339e8 ± 5.496e5 pb** over three seeds vs MG 6.788500e8 ± 1.473e6, rel **+0.37%** at pull **+1.58**, at `rel_tol` 0.005 (75k, addendum S6) — the reference's own 0.22% with headroom, pull asserted, since its channel-partition ambiguity is only `1.0e-3` (its own Monte-Carlo error, because a 2 → 2 gives the clustering no merge to choose). It sums over MadGraph's own 65 concrete assignments, pinned entry for entry against the run's `leshouche.inc` (all figures in this row measured fresh 2026-08-05 by the addendum close-out session, note 32 S8; run `pixi run --skip-deps validate-sigma` / `validate-hadronic` to reproduce) |
| 6 | Unweighted event output (LHEF) | ✅ Done | Accept/reject over the frozen per-channel grids (channel `∝ w_maxⱼ`, overweights kept at weight `>1` and counted), per-event helicity (`∝ \|M_hel\|²`) selection, colour selection via MadEvent's `SELECT_COLOR` rule (configuration `∝ AMP2_d`, flow `∝ JAMP2` inside its `ICOLAMP` row) with **per-member colour-flow tables** — each flavour member's tags derived under the structurally-determined flow permutation, refuse-on-ambiguity — checked against MG's `leshouche.inc` (73/73 concrete subprocesses over 47 files; note 29 chain A), `SCALUP`/`AQCDUP` from `coupling::scales`, four-layer `lhef/` writer/reader that re-serialises all 37 banked MG runs byte-for-byte (744 759 events, both of MadGraph's serialisation dialects, source-text pass-through by construction). `vibegraph generate` refuses mismatched cards/models, swappable weight strategy (`Buffer` `IDWTUP=-4` / `StochasticRounding` `+3`). `lpp = 1` gated: `validate-generate-proton` takes the llj cards to a `.lhe` (flavour draw ∝ per-group luminosity × σ̂, sample σ within `SIGMA_MAX_REL = 0.015` of the banked run). `p p > e+ e-` reaches an event file too, on the same general path. Pythia 8.312 reads both emitted samples back end to end (2000/2000 each, colour-mutation negative control rejected). Event samples are compared against MadGraph's banked ones column by column (`samples` category: weighted-ECDF KS on the kinematics, chi-squared on `SPINUP`/`ICOLUP`/flavour) |

## Closed-sprint history

One line each; the note is the full record. Earlier sprints
(`helas-generalize`, `mg-validation-coverage`, `cleanup-refactor`,
`performance-sprint`) live in git history and notes 12/13.

- **`color-flow`** (feature, merged 2026-07-12) — multi-flow JAMPs + exact CF |M|²; note 16 (incl. the VVVV phase-bug root cause and the fermion-flow slot-swap debrief).
- **`validation-sprint`** (validation, closed 2026-07-13) — `gg_to_gg` NCOLOR=6 enforced, VVVV −i fixed; notes 12/16.
- **Eval performance program** (performance, closed 2026-07-17) — layout/folding/SoA + helicity expansion + helicity filtering; vs-MG gap 8.6×–110× → **1.2×–3.5×**; note 15. Ratios are single-host (M3 Max); rerun kit `scripts/mg_perf_compare.sh` + note 15 §2.4. Contract: pruned evaluators need partonic-CM beams-along-±z momenta.
- **`hadronic-xsec`** (feature, closed 2026-07-19) — PDF convolution + run-card cuts + (τ,y) VEGAS; σ(pp→e⁺e⁻) vs MG 0.14%/0.07%; `integrate` CLI + artifact; note 18.
- **`validation-2`** (validation, closed 2026-07-21) — V1–V6: NHEL pinning 14/14, proc-card `integrate`, σ-level gate, PDF seam, rooting-soundness (all rooting signs lifted to `fermi_sign`, 0/133), convention-channel guards; V7 deferred (below); note 19.
- **`eval-perf-2`** (performance, closed + merged 2026-07-21) — mul-split, one-shot DAG validation, ZEROAMP skipping, fewest-ext-leg rooting; `forward` **1.18×–2.19×** every process, ≤1e-12 vs MG; note 20.
- **`resonance-sampling`** (feature, closed + merged 2026-07-26) — MadGraph-style multichannel in production; 2 resonant σ rows SKIP→GATE. Transferable lesson: a fixed-seed pull cannot validate a sampler — VEGAS's 1/σ² combination makes a missed region *confidently wrong*, so seed sweeps are part of the gate; note 21.
- **`dynamical-scales`** (feature, closed + merged 2026-07-27) — MG's `αs` RGE + per-event `μR`/per-beam `μF` through the constant pools; 3 QCD σ rows GATE, DY unmoved; found MG's `AQCDUP` π-truncation and `SCALUP` ≠ μR defects (note 07) and the missing `gg→gg` symmetry factor; note 22.
- **`event-output-lhef`** (feature, closed + merged 2026-07-28) — JAMP2 flow selection + `leshouche.inc`-checked `ICOLUP` dictionary, accept/reject unweighting, byte-pinned LHEF writer/reader, `vibegraph generate`, model identity in the artifact (fv3); two plan corrections recorded (channel draw `∝ w_maxⱼ`; `IDWTUP=-4` not required for overweights); note 23.
- **`user-distribution` + `proton-events`** (feature, two tracks, closed + merged 2026-07-31) — Track P: llj amplitude rows 14→18 plus a per-diagram `AMP()` oracle, measured flavour groups, `ProtonIntegrand`, σ(pp→ℓ⁺ℓ⁻j) and `generate` gated at `lpp = 1` (artifact fv3→fv4). Track U: release/CI/acceptance workflows, `~/.vibegraph` cache, consent-gated pinned PDF fetch, `check-events`. Transferable lesson: **a seed sweep is necessary and not sufficient** — five mutually-consistent seeds were collectively 1.0% low, so budget convergence is a second axis. Also `[profile.dev] opt-level = 2` cut `cargo test` 3m16s → 1m05s with nothing weakened; note 24.
- **`validation-3`** (validation, closed 2026-07-31) — three declared dependency layers (`hermetic`/`banked`/`oracle`) with `validation/manifest.toml` as the single per-process source of truth; the `amplitudes` category made hermetic on MadGraph's own banked events; every hadronic σ moved onto the general `ProtonIntegrand`; the new `samples` category (KS + χ² against MadGraph's event samples); Pythia consumption; and one asserted report table over 26 rows × 4 categories. Transferable lesson: **a report is only evidence if every green cell is a recorded measurement** — inferring a cell from "the suite passed" is the same failure as a vacuous check. Findings register in note 25 §10.
- **`v3-backlog`** (validation follow-up, closed + merged 2026-08-01) — every register finding resolved rather than tolerated: the h→ττ pole was **MadGraph 3.5.7's `get_channel_cut` defect** (`(t-Mass)*(t+Mass)` on `t = p²`; upstream fix `286feb8e6`, first in 3.6.2) and both cells now GATE against 3.7.1; the colour draw reproduces MadEvent's `SELECT_COLOR` via per-diagram `AMP2_d` (both χ² targets hit); `Cuts::shat_min` derives `setcuts.f`'s general bounds (`pp_to_bb_fixed` σ GATE); DY events banked with a live `dσ/dm_ll` gate; references re-banked on **3.7.1** into `refdata-3` (finding: 3.5.7 ran every `lpp = 0` process at `αs(M_Z) = 0.130`, so refdata-2/3 partonic σ are not comparable); the LHE writer round-trips **both** MG serialisation dialects by construction (34/34 byte-for-byte, 14/34 still reproduced with source dropped); latent `IDWTUP = -3` σ-misread fixed en route. Census 72/68/4 → **75/74/1**; note 27.
- **note-29 validation sprint** (validation, closed 2026-08-03, branch `val4`) — seven design→implement→review chains + the §G re-bank. A: conjugate colour tags fixed by **per-member colour-flow tables** under a structurally-determined permutation (dijet `ICOLUP` χ² p 0 → 0.105–0.263, T5 0/80 000; the design's premise falsified twice by measurement en route). B: **MadEvent's per-point `AMP2_c` scale-channel draw** in production (pure function of `(channel, u)`, zero bits from existing streams) — partition gaps collapsed to MC noise, `gu`/`gux`/llj_dyn tolerances retired 0.02/0.015 → **0.005**, σ(pp→ℓ⁺ℓ⁻j) rel −0.68% → **−0.01%**; reviewer derived the missing `this_config` reconciliation from MG source. C1+C2: every descoped card surface a **hard error** (polarization, decay chains, `propagators.py`, 209-field audit with 23 refused, μF ≥ 2 GeV veto as zero-weight, `dynamical_scale_choice` 1–5 refused). D: `ee_to_mumua` drift adjudicated **D1 — the reference owns it** (MG's own partitions disagree at ≥15σ; our total matches its m(μμ) re-integration at 0.16σ; tolerances unchanged). E: ForcePositive with LHAPDF's own clamp semantics; `validate_kt_cluster` a declared oracle tier; +2 cells. F: U(1) charge-flow phase — pre-registered negative result. §G: **`refdata-5` pinned** (four runs re-carded onto lhaid 247000, member list identical name-for-name; publication pending), all 8 ⛔ cells enforced. Census 87/85✅/2⚠️ → **98/96✅/2⚠️**. Transferable lesson: **pre-register the may-move set** — chain B's escalation diff landing byte-exactly on its five predicted cells is what made the sprint's biggest σ change auditable at a glance; note 29 close-out.
- **`kt-spine`** (feature, two tracks, closed 2026-08-02) — Track K: MadGraph's general kT clustering reproduced merge for merge against an instrumented 3.7.1 (90 000 dumped events, zero observed deviation), the closed forms deleted so `dynamical_scale_choice = -1` takes one path, `GridAlphaS` made LHAPDF's own `AlphaS_Ipol` and the density grid continued past its edges — then the flips: 6 asserted-refused scale rows became per-event replays, the 4 llj partonic σ rows and their `samples` cells left `blocked`, σ(pp→ℓ⁺ℓ⁻j) re-gated at the dynamical scale, and the capstone **`p p > j j`** gated on MadGraph's shipped run-card defaults (**6.803009e8 ± 2.511e5 pb** vs MG 6.788500e8 ± 1.4726e6, rel +0.21%, pull +0.97). Track S: the identical-particle factor moved into the phase-space map per subprocess, and the multi-rung t-channel spine landed in production. Two bugs the sprint found rather than assumed: the **fixed-beam path was never regulated** (every prior "what is the spine worth" measurement was taken on flat transfer draws), and `p p > j j`'s σ was **36% high** because a repeated final-state label enumerated `g u > g u` and `g u > u g` as two subprocesses. Transferable lesson: **a per-event field is a finer oracle than a cross section, and it exists more often than it looks** — the clustering was pinned by an instrumented replay of MadGraph's own intermediates long before any σ moved, which is why every σ flip that followed had a diagnosis attached. Census 75/74/1 → **87/85/2** over 29 rows; note 28.
- **`perf-sprint-3`** (performance, three tracks, closed 2026-08-05) — eleven sessions against the note-30 baseline, all merged. Layer result, one host one sitting: `pixi run --skip-deps validate` **691 s → 391 s (−43.4%)** on the identical command with the census cell-for-cell unchanged; per-row single-thread **integrals 842.6 s → 389.8 s (−53.7%)**; integrand throughput vs MadGraph on note 30 §5.3's CPU-time denominators **geomean 6.84× → 8.76×** over the same 26 rows (`pp_to_jj` 2.3× → 4.6×, `pp_to_llj_fixed` 5.8× → 9.1×); per-point MATRIX1 **1.25× → 0.98×** with the evaluator itself −21.6% and 8 of 14 processes now beating MadGraph (was 3); `integrate` **4.70×**/**5.36×** from `-j 1` to `-j 16` on `dy13_default`/`pp_to_llj` at a byte-identical artifact. **Track I**: I1 made VEGAS's iteration combination unweighted (`5b3952d`/`59887a3`) — a 4000-seed offline study showed the plan's warm-up discard removes essentially none of the bias and its parenthetical was the real lever — collapsing the llj ladders (`pp_to_llj` span 2.09% → 0.46%) and re-pinning `LLJ_NEVAL` 300k → 150k; I2 gave the `w_max` scan its own budget (`24aad64`/`928df28`) and **falsified its own premise**, the maxima never converging (`Σⱼ w_maxⱼ ∝ n^0.508` over 2.4 decades — a Pareto weight tail of index ≈ 2), so the lever is the percentile rule not the budget; I3 made the hadronic integrand `Sync` and added `-j/--parallel` (`6497f7e`/`1b527cc`), bit-for-bit at any thread count by construction; I4 added convergence-targeted integration with hard-split Neyman allocation (`30d44d1`/`539f0da`), 64/64 calibration runs meeting target, CPU parity with MG on llj and 4.2–4.5× less CPU on dy13. **Track P**: P1 replaced the per-flavour PDF reads with an f64-only all-flavour kernel (`71a7ef3`/`a66d58a`) — `xfx_all` 112 ns vs 504 ns, PDF share 14.5% → 1.38% and 19.4% → 2.00%, **no tolerance relaxed**; P1b added an absolute screen to the continuation oracle (`91c5a79`/`0fd6344`) and recorded why Horner+FMA stays rejected in `cubic_hermite`. **Track E**: E1 studied execution order (`52327b2`/`7416c1d`) and E1b made op-blocked-within-ASAP-levels the production schedule (`486e237`/`94af883`) — −17.3% eval geomean, bit-for-bit across all 100 banked row files; E2 hoisted the arenas into local slices in `fill_arenas` (`54d666f`/`d2d7520`), header reloads 143 → 20, while measuring and **rejecting** threaded dispatch (+7.7%) and force-inlined sret kernels; E2b shared the four spinor products in the chiral currents (`ced17fb`/`027e710`); E3 found the plan's prefix design a **NO-GO by measurement** and landed an arena-reuse cache instead (`6a8e91c`/`50fb671`), order-preserving and bit-for-bit. Transferable lesson: **four of the eleven sessions refuted their own brief's mechanism and still delivered** — I1's discard, I2's budget, E2's dispatch, E3's prefix — because each was pre-committed to a measurement that could kill it; the close-out then did the same to its own brief, whose prescribed `RAYON_NUM_THREADS=1` per-row protocol serialises every concurrent row through one global worker and would have published an ~8× phantom regression. Note 31 §6.
- **`perf-3-addendum`** (performance/validation cleanup, eight of nine sessions, closed 2026-08-05) — S1 E3b (bit-identical cut-before-draw on the fixed-beam path) + I5 (`combine_seeds` unweighted, matching I1); S2 read `w_max` off MadGraph's own `unwgt.f` truncation-ladder rule instead of a never-converging scan extremum, unweighting efficiency on five rows 22.2/20.6/23.3/10.6/4.21% → 54.1/52.1/52.9/38.9/9.98%, `p p > l+ l- j` 4.36× cheaper per effective event; S3 parallelised the α-adaptation survey (bit-identical at `-j {1,4,16}`) and made `--target-rel` the CLI's default convergence mode, correcting note 32 §1.1's own serial-floor decomposition (41–52% of the `-j 16` wall, not ~27%); S4 closed all three `mg_perf_compare` findings (manifest-driven registry on both arms, host-labelled committed `mg_timings.json`, bench widened 14→19 rows) and found the newly-included rows were QCD-dense, inverting the sample's earlier bias (19-row MATRIX1 geomean 0.95×, 14-row continuity check 1.06×); S5 hoisted a per-event merge-table rebuild out of the clustered-scale path, −16.9% to −22.3% ns/point on the three clustered rows; S6 sized five hadronic σ budgets to reference precision under a ladder+sweep license (300k → 75k), `validate` CPU −288 s against only −18 s of wall on that cut alone, because `validate` runs its rows concurrently; S7 turned the 2→6 rows on as `info` at the long tier — the "~1 ms/eval" skip premise was stale by more than an order of magnitude, the real cost floor is `MIN_CHANNEL_NEVAL` × channel count, and the physics agrees under the multichannel (five-seed mean inside 1.1% of a 0.30%-precision bank) even though single-seed pulls do not shrink with budget — census 98 → **100**; S9 (restore the pre-lane-FMA scalar packed-complex codegen) was **killed clean**: the idiom is x86-specific and cost 8–9% on this ARM host, the opposite of a win, so nothing merged. S8 re-recorded every σ/percentile/cost figure the wave-1/2 sessions left stale and took the addendum's own close-out measurements: `validate` wall 443.3 s against note 31's 391 s reads as a regression only because the host was not quiet (`mds_stores`/`mediaanalysisd` held load average 6–25 through the run) — CPU is the reliable instrument here, per S6's own −288 s CPU / −18 s wall split on its cut alone; census both numbers explained (98 in the layers `validate` drives vs **100** once the oracle layer's `validate-sigma-2to6` has run); `-j 16` on `dy13_default`/`pp_to_llj` at `--fixed-budget --neval 120000 --niter 12` reads **8.68×**/**9.48×**, bit-identical artifacts. Transferable lesson: **every session corrected something in its own brief** — mechanisms (S2's truncation ladder, S7's `MIN_CHANNEL_NEVAL`), decompositions (S3's 41–52%), and inverted hypotheses (S4's QCD-dense bench, S9's x86-only idiom) — consistent with the pattern the base sprint already showed. Note 32 §5.

---

## 🔎 Validation backlog

### Standing findings to diagnose (from the note-29 sprint; never a loosened tolerance)

- **`ud_to_epemud_qcd0`'s event sample fails its `ICOLUP` χ² at ≈650 on 1 dof**
  (p ≈ 0, seed-stable, 60 000 events over 6 seeds) while kinematics and
  `SPINUP` clear their floors — measured the moment chain E wrote the
  comparison. This is the **fixed-beam** record path (`SubprocessRecord::new`),
  not chain A's relabelled-member mechanism (that fix is in and gated on the
  hadronic rows): a colour-flow convention gap in the mixed-line topology's
  flow→`ICOLUP` dictionary is the standing hypothesis. The row is `info` until
  diagnosed; the diagnosis session should start from the banked run's
  `leshouche.inc` against `color_flow_tags` for this process class.
- **`ee_to_mumua` residual ~1% ours-high excess in the radiative-return
  `pt(γ)` windows** — the one thing chain D's D1 verdict left unattributed,
  localised to MadGraph's `pt(γ)/η(γ)` coverage rather than either matrix
  element or the Z propagator. The named next probe: the 2D
  `[39.4, 77) × [86, 96)` `(pt(γ), m(μμ))` cell. Related watch item: the
  row's `samples` KS floor headroom is **3.6×** (re-measured 2026-08-05,
  `3.605e-4` against the `1e-4` floor — `ee_to_wpwm`'s `pt(w+)` cell is now
  the closest of any gating row, at `1.573e-4`/`1.6×`, unrelated to this
  row's diagnosis) — if either cell flaps, the D record (MG's sample
  contradicts MG's own integrals at ≥15σ) is the diagnosis context for
  `ee_to_mumua`, and the floor does not move.
- **`p p > j j`'s across-group scale spread is `4.999999e-7`, not zero**,
  while every within-group spread is exactly `0.0` (chain B-0's census).
  Nothing in production reads the group axis for scales, so it moves no cell —
  but it is far too large to be rounding on a 2→2 whose scale ought to be
  group-independent, and it is the size of the effect a future group-axis
  change would expose. Worth one look at where the group enters.

### Sharper oracles the sprint named but did not build

- **A `SCALUP` column in the `samples` category** — the sharpest missing
  oracle (chain B review): no samples cell compares `SCALUP`, though
  MadGraph's banked LHEs carry it and `validate_scales` already replays it
  for MG's own events. Two findings wait on it: rows that compile no scale
  prescription emit the run-card `SCALUP` and `AQCDUP = 0`
  (`vibegraph-cli/src/generate.rs:349`) while MadGraph's own `ee_to_mumua`
  events carry a clustered channel-dependent `SCALUP` — σ is right (nothing
  reads the scale) but the *records* differ, and no gate sees it; and a
  per-event scale distribution check is the one oracle that would catch an
  `AMP2_c`-share error that preserves σ.
- **Expose the drawn scale configuration and close chain B's two accepted
  gaps**: assert the `∝ AMP2_c/Σ AMP2` frequency law end-to-end (today it is
  factored into four independently-gated pieces — `select.rs`'s binomial
  test, the order pin, the colour-draw χ², and σ), and promote the
  zero-spread census to a banked assertion on the cheapest inert rows so a
  future change that makes a scale configuration-dependent on a declared-inert
  row fails a standing gate instead of a one-time manual diff. Blocked on
  nothing but runtime cost for the latter.
- **`scale_draw_fallbacks()` is counted and read by nothing** — a NaN `AMP2`
  falls back to the sampler's channel silently (`select_index` returns `None`
  on a non-finite total). One assertion that the counter is zero on the gated
  rows makes the silent path loud.
- **`RunningAlphaS::eval` returns NaN silently below ~0.5 GeV** (the two-loop
  `newton1` seed takes `ln` of a negative argument). The μF ≥ 2 GeV veto and
  the μR floor bound today's exposure, but the surface is a silent-NaN class:
  one guard (error or clamp, matching MG's own behaviour) closes it.
- **`k`/`G` measured exactly ±1 (and the per-process route-sign patterns)** —
  chain F's settled leads: one `|Im(k/G)|` assertion converts free phases to
  pinned bits (no reference data, no tolerance move); `run_config_amps()[i]`
  disagrees in sign with the single-diagram compile on 3 processes (exactly
  ±1, spread 0, production evaluators, no production consumer — eval_amp2 is
  sign-blind). Any future assertion must pin per-process sign patterns, not
  uniformity.

### Deferred coverage

- **V7 per-flavor diagram matching** — deferred from `validation-2`: Python
  extractor + Rust sorted-PDG matching + JSON regen, with a real-finding risk
  (whether vibegraph enumerates MG's exact concrete-subprocess union). Design
  preserved in note 19 §3 / §V7.
- **`diagrams.json` carries counts only, not the per-flavour union** — the
  committed reference is what the existing extractor produces, so the
  multi-channel `diagrams` cells assert a summed count and not the concrete
  subprocess list the manifest describes. Filling that in is the deferred V7
  design (above) reaching `extract_diagrams.py`; until then the manifest's
  "includes the per-flavour concrete-subprocess union" notes describe the
  intent, not the current assertion.
- **`IdentityAmp` process-level coverage** — the last `KNOWN_UNCOVERED` op; needs
  an `Identity` scalar bilinear the SM lacks, so it rides with `non-sm-ufo`
  (feature backlog).
- **Flavour-group probe coverage** — `derive_flavor_groups` partitions on sampled
  `|M|²`, which is complete but unsound whatever the probe set: two subprocesses
  differing only where the probe does not look are merged silently. The probe
  ladder is hardened (five rungs down to a fifth of the base energy and onto the
  `Z` mass, closest-pair separation measured at **0.74**, asserted > 0.1); the
  sound replacement is the s-expression criterion (feature backlog). Accepted
  for v1 on the MG-helicity-filtering precedent (see the feature-backlog entry).
  (`proton.rs`, note 24 §P2c.)
- **Pythia consumption gate — what it cannot see.** The gate reads both emitted
  samples n/n and its negative control proves it is not colour-blind, but four
  things stay outside it. (a) Only the `Buffer` strategy (`IDWTUP = -4`) is fed
  to Pythia; `StochasticRounding` (`+3`) writes a different `<init>` header and
  is unexercised. (b) The reconstruction check compares the *multiset of outgoing
  PDG codes* against the file, so a permuted or corrupted momentum would be
  consumed silently — comparing Pythia's `process` four-momenta against the
  record's would close that. (c) The negative control mutates `ICOLUP(1)` only,
  on one event, so an error confined to `ICOLUP(2)` or to the beam-side
  connectivity is not shown to be detectable. (d) Nothing checks Pythia's
  interpretation of `SCALUP`, `AQCDUP` or the `<init>` cross section — the file
  is proven *readable*, not proven to mean what we intended.

### Gate + tooling hygiene

- **`mg_perf_compare` bypasses the manifest on both arms and banks no artifact**
  — ✅ **done (addendum S4, note 32 §1.2)**. `gen_amplitude.py` carried its own
  hardcoded `PROCESSES` registry instead of reading `validation/manifest.toml`
  (whose header claimed the generators read it), and `mg_timings.json` existed
  only in the gitignored work area — not in the refdata bundle, not in the
  report, no host identity in the JSON — so a fetched checkout could not run
  `scripts/mg_perf_compare.sh` at all. `eval_strategies.rs` was a *third*
  hand-synced copy of the registry benching only **14 of the 19**
  MATRIX1-comparable processes — the five silently dropped
  (`uux_to_epemg`, `ddx_to_epemg`, `gu_to_epemu`, `gux_to_epemux`,
  `ud_to_epemud_qcd0`) are exactly the QCD llj-class rows, biasing the MATRIX1
  table toward the EW rows already ahead. All three fixed: `manifest.toml`
  gained a per-row `mg_amplitude` table and `gen_amplitude.py` reads it (dry-run
  parameter dump verified byte-identical across the migration, no reference CSV
  regenerated); `mg_timings.json` carries host identity and a host-labelled
  copy is committed beside `timings.json`, with `mg_perf_compare.sh` falling
  back to it and reporting one-sided rows instead of dropping them;
  `eval_strategies.rs` derives its row set from the manifest at runtime,
  covering all 19 rows. **The widened table reads worse, and that is the
  finding, not a regression**: the five previously-dropped rows are QCD-dense,
  inverting the sample's earlier bias toward EW rows the crate already beats —
  the 19-row MATRIX1 geomean, **0.95×**, is the new standing baseline, with the
  14-row figure recorded alongside it as the continuity check (**1.06×** on
  this run of the bench, relayed from the session report and not independently
  re-measured by S8 — bench noise on this machine has not been characterised
  against note 31's ~0.98× closely enough to call the two consistent). (Note 32
  S4.)

- **Weekly `schedule` trigger on `acceptance.yml`** — left off because it can only
  fail until a first release exists. Turn it on once one does: it is also the
  second detector for the "CERN repackages the PDF archive" risk, whose only
  other detector is an `#[ignore]`d test nobody runs on a timer. (Note 24 §U2.)
- **Small hygiene the note-29 sprint left named**: the `blocked` tier is now a
  documented manifest schema slot used by nothing (keep or retire — a schema
  decision); `probe_recarded_budget_ladder`, `probe_bb_budget_ladder` and
  `probe_2to6_budget_ladder` — ✅ **all three now have pixi tasks**
  (`ladder-recarded`/`ladder-bb`/`ladder-2to6`, addendum S6/S7, note 32) and
  are all named in `validate-deep`'s long-tier text. The `~26 min` figure this
  bullet quoted for `probe_recarded_budget_ladder` was itself stale — S6
  measured all eight hadronic budget ladders (llj fixed + dyn, `bb_fixed`, the
  four re-carded rows) at **357 s total on 16 cores**, over 4× less than
  claimed and the number to use going forward; `pp_to_llj`'s recarded
  `integrals` gate now runs at **150k** points/iteration, not 600k — its
  ladder still climbs monotonically over the whole eightfold range (0.04% to
  0.21%), so `150k` is the lowest rung whose scatter a three-seed gate can
  read without a single seed dominating, not a converged rung, and the
  standing follow-up is unchanged: `info`, never a wider tolerance, is the
  fallback if the climb ever needs addressing (addendum S6, note 32); the
  direct-vs-mirror ordering is a third partition axis chain B named with a
  falsifier but nothing measures; chain B's draw raises low-budget seed
  scatter (χ²/dof 6.38 at 75k, clean ≥150k) — a future budget reduction on
  `pp_to_llj_dyn` would bite;
  the `Opaque` run-card default payload fix (note 28 §C2.5).
- **`pp_to_jj`'s banked event sample is not reproducible across MG re-runs** —
  σ is identical to all printed digits and single-group runs regenerate
  bit-identically, but `pp_to_jj`'s five subprocess groups make the unweighting
  draw sensitive to job scheduling, so a re-run yields a different (equally
  valid) event sample. The banked sample is the reference; any "regenerate the
  bank byte-for-byte" claim must exempt multi-group runs, and C's `samples`
  gate compares distributions, not bytes. (Sb, note 28.)
- **`pp_to_jj`'s 9 tie-break events want a K2-style clustering dump** — K4
  enforces them by signature (the `√(1+1e-6)` beam-crossing inflation is the
  only difference, `<rscale>`'s printed digits pin it) and asserts the count,
  but only an instrumented dump of a `p p > j j` run would show the merge
  sequence directly. The sprint banked the run without a dump; a future
  oracle-layer pass can add one via `gen_kt_cluster_dumps.sh`. (Note 28 §K4.)
- **The K2 clustering-dump format should key its per-directory tables by
  process-directory name** — the writer already has the name (`SHARD` records)
  but the extraction drops it, which forced K3 to disambiguate merged tables
  by forest-row length plus a per-event candidate-list consult on 2 of 9 runs
  (`pp_to_bb_qcd2`, `pp_to_llj*`, with an outright `NQCD` collision). A
  re-extraction keyed by directory removes that whole exception class.
  (Note 28 §K3.4.)
- **`pixi run -e madgraph extract-diagrams` silently re-runs MadGraph** for any
  script whose output directory is missing (`depends-on = ["build-diagrams"]`
  → `build.sh` regenerates). Anyone holding a run directory aside must invoke
  only `--skip-deps` tasks, or the held-out run comes back as a fresh MG job.
---

## 🧩 Feature backlog

### Descoped from v1 (user, 2026-08-02)

Each of these is out of the release goal's restricted scope. The validation
sprint makes every one a hard error where a card can ask for it (slate item 4
above); the entries here are the eventual features.

- **Beam polarization** (`polbeam1`/`polbeam2`) — polarized matrix-element
  sums and the per-event `SPINUP` consequences.
- **Beam configurations beyond unpolarized `p p` and fixed-energy partonic** —
  antiproton beams (`lpp = -1`, Tevatron), mixed configurations, lepton-PDF /
  photon beams. `RunCard::parse` admits exactly (0,0) and (1,1) today.
- **Decay-chain process syntax** (`p p > t t~, t > w+ b`) and 1→n
  single-particle decay processes — the grammar and the phase space both
  assume a 2→n hard process.
- **Custom UFO propagators** (`propagators.py`, UFO 2.0) — parse the file and
  thread the propagator forms through the HELAS compiler.
- **Non-SM UFO models** — the `non-sm-ufo` checklist below; the README's
  scope section points at it as the natural next scope step.

### In-scope features

- **s-expression program identity for flavour grouping** — a dedicated future
  sprint, user-scoped. Today's `derive_flavor_groups` partitions subprocesses by
  sampled `|M|²` agreement: **complete but unsound** — two programs that differ
  only where the probe does not look are merged, and the merge is silent.
  **Accepted for v1** (user, 2026-08-02): probe-based judgment has MadGraph
  precedent — MG's own helicity filtering drops vanishing helicity
  configurations on the same sampled-probe basis — and the probe ladder is
  hardened (below); the sound criterion remains the right eventual replacement.
  Replace it with a sound-but-conservative criterion: two subprocesses share a
  group iff their compiled programs are *identical as s-expressions*. Three
  prerequisites, in order:
  1. **Universal constant ids.** Compare UFO-stable coupling/particle
     identities, never per-compilation pool slot indices — flavour-dependent
     couplings can share a slot, so slot-index equality would be **unsound**,
     the exact failure the new criterion exists to remove.
  2. **Canonicalization of the un-optimized s-expression.** Lowering carries a
     ±1-CSE-node nondeterminism (note 15 §4–5) and diagram order is unstable
     (cf. `MG_DIAGRAM_ORDER`), so the comparison must run on a canonical form,
     before optimization, with a deterministic diagram ordering.
  3. **Colour folded into the s-expr language**, so the colour basis is part of
     the compared term rather than a side condition checked separately.
  Being conservative, it can only *split* groups that are genuinely equal —
  costing compiled programs, never correctness. Keep the sampled criterion as an
  independent cross-check when it lands: they should agree, and a disagreement
  is a finding. (`proton.rs`, note 24 §P2c.)
- **Streaming `IDWTUP = -4`** by deterministic two-pass replay — the interface
  hook (`EventSource::restart`) is in place and contract-tested; not needed while
  100k-event runs buffer in ~42 MB. (Note 23 close-out.)
- **Massless-t-channel fiducial cut** (sprint plan: note 28 §S2/D3) — a
  massless beam pins `t_max = 0` (collinear edge) where the t-map falls back to
  flat; whether a fiducial cut is wanted instead is unresolved for a physical
  massless-initial-state t-channel. (Note 21 close-out.)
- **Re-examine the "no spine without a scale past two outgoing legs" policy** —
  the policy predates the peripheral-kinematics conditioning fixes (grouped
  Källén, `γ = E/√s`), which removed most of the unregulated-spine defect it
  guarded against: with the grouped form the massless transfer edge is the
  exact analytic zero whenever the emitted subsystem carries a fixed
  invariant, and only composite emitted sides still exhibit the defect. The
  conservative fallback is kept; whether it is still the right default is an
  open measurement. (Note 28 §S3 deviations.)
- **`typed-units`** — research `uom`/`dimensioned`/`units` crates for typed
  four-momenta and cross sections.
- **Self-contained `generate` artifact** (user, 2026-08-02; post-v0.1) — one
  file a clean worker machine can sample from. Today a proton-beam worker
  needs the binary + artifact + both cards + the PDF set (unweighting reads
  densities and grid-αs per trial point; the README documents the
  copy-to-working-dir workaround), and a non-SM run needs its UFO directory
  too. Three pieces, taken together as one feature:
  1. **Bundle the compiled program** (design in note 23; absorbed from the
     performance backlog, whose trigger — setup climbing to a noticeable
     share of a generation run — still applies: compilation is 0.05–0.29 s
     against ~13 s for a 20k-event `generate` today). Key
     `(model digest, process, compiler schema version)` is derivable from
     banked fields, no schema bump needed. Note 23's recorded obstacles: no
     serde in `helas::eval`; `folded_hel` is a lazy `OnceLock` and the
     expanded arena is the large part; `prune_zero_helicities`' kinematic
     contract must be rechecked on load.
  2. **Bundle the PDF data the run reads** — the member's grid file verbatim,
     or a subgrid slice pinned to the run's (x, Q²) support; which, is part of
     the design. Keeps the artifact's refuse-on-mismatch property: the banked
     set name/member already gate, the data would too.
  3. **Investigate compactifying the VEGAS grids** — long-term: per-channel
     grids dominate artifact size on multichannel processes; quantization,
     sparser binning, or shared axes are unexplored.
- **Quality sprint: tighten the `pub` API surface** (user, 2026-08-02) —
  before any backwards-compatibility promise (i.e. before 1.0): audit what
  `vibegraph-lib` exports, demote what only the CLI and the validation crates
  consume, and decide what the supported library surface actually is. Until
  then releases stay on the 0.x line (first tag `v0.1`).

### `non-sm-ufo` — collected boundaries a non-SM UFO model will hit

**Explicitly descoped from v1** (user, 2026-08-02): the release goal is the SM
UFO, and the README's scope section says so and points here. The UFO surface is
deliberately model-generic, but "generic" currently ends at the SM's feature
set. None of these block anything; collected so a future BSM-model task scopes
against a checklist instead of rediscovering each wall one hard error at a
time. A small dedicated test model (or a public BSM UFO) would be the natural
vehicle for several at once — and would also retire the standing gap that no
non-SM model has ever been loaded end to end, so "model-generic" is currently
exercised on SM evidence alone.

- **Color sextets and baryonic epsilons**: the color engine handles
  Singlet/Triplet/AntiTriplet/Octet only (`helas/repr/color.rs`); sextet tensors
  `K6`/`K6Bar`/`T6` (diquark models) and the baryon-number-violating
  `Epsilon`/`EpsilonBar` (e.g. RPV SUSY) are deliberate hard errors
  (`ufo/color.rs::SextetUnsupported`, `helas/color/tensor.rs`). Note the two
  distinct "6"s: NCOLOR=6 (flow-basis dimension) is fully supported; the sextet
  *representation* is not. MG's reference algebra lives in `color_algebra.py`;
  support means new `ColorTensor` atoms + trace-basis reduction rules + CF
  products, validated the color-flow way (CF oracle vs MG's DATA CF, then the
  JAMP-weighted |M|² gate).
- **Spin codes beyond {1, 2, 3}**: `helicity_states_for_spin` (`eval/compile.rs`)
  future-proofs the spin-2 helicity list (code 5), but nothing downstream builds
  tensor external wavefunctions or propagators; spin-3/2 (code 4) is an
  `UnsupportedSpin` error. Ghost codes stay irrelevant at LO.
- **Majorana fermions** (MSSM neutralinos, gluinos): fermion-flow handling
  assumes Dirac-continuous lines end to end — no flow-flip/charge-conjugation
  machinery. Classically subtle sign territory; the `color-flow` fermion-flow
  slot-swap bug shows how delicate the flow conventions are even pure-Dirac.
- **`IdentityAmp` process-level coverage**: needs an `Identity` scalar bilinear
  the SM lacks — a natural rider on whichever small test model lands first.
- **Loop-level UFOs** (`loop_sm`, NLO models): out of the LO charter (parser
  history in note 04).

---

## ⚡ Performance backlog

- **Per-flow α tuning — offline gain measurement first** (user, 2026-08-01;
  sequenced after B6, which provides the shares). Stratify the integrand by
  leading-colour share `s_i = |JAMP_i|²CF_ii / Σ_k |JAMP_k|²CF_kk` (positive,
  partition of unity, interference apportioned pro rata) and tune a separate
  channel-mixture α per stratum. **Stage 1 is a measurement, not a sampler**:
  the Kleiss–Pittau optimal α and its variance are computable offline from
  recorded `g_j(x)`, `f(x)` and `s_i(x)` on existing samples — report the
  achievable variance reduction against the ×(strata) evaluation overhead
  before building anything; a small number dies here like note 26's parquet.
  The blocker that stood here is gone: `uux_to_uux`/`gg_to_gg`'s channel maps are
  no longer bit-identical and their α no longer sits at uniform (note 28 §S4 B2),
  so per-flow α is no longer a no-op on those rows. Flows still overlap heavily,
  so the gain is the inter-stratum covariance term, expected modest. **Guardrail:
  split the tuning, never the coverage** — every stratum keeps every channel
  with an α floor, or the `sde_strategy`-class fragility (note 27 §B1) is
  rebuilt on our side.
- **Stratified-parallel integration axes** (user, 2026-08-01) — the iterative
  VEGAS+α loop needs an embarrassingly parallel axis for SIMD/multi-thread
  promotion. Catalogued, exact-first (no partition function, no fragility):
  (a) **channel-block stratification** — ✅ **done** (note 31 §I4). The hard split
  was already there (`adapt_grids` is per-channel deterministic with a 512 floor);
  what landed on top of it is `budget.rs`'s allocation and stopping rules and
  `vegas::adapt_blocks_iteration`, which runs every channel's iteration in one
  rayon region keyed `(channel, chunk)`. The multinomial survives only in the
  undivided comparison estimator, which is not the production σ path — the
  α-survey is the same deterministic chunking now too (addendum S3, note 32,
  below); (b) **helicity strata** — `Σ_hel |M_hel|²` is an exact orthogonal
  decomposition (no interference for unpolarized beams), so helicity classes
  (parity-folded, zero-classes dropped) can carry their own budgets/grids;
  first real consumer for `mg-single-helicity-bench`; (c) **flavour groups ×
  beam orderings** (hadronic) — already independent integrals, and **no longer
  blocked**: the `RefCell` scratch is gone (`SubprocessProto`/`BoundSubprocess`
  plus `ThreadLocal` scratch, note 31 §I3), so both integrands are `Sync`;
  (d) **frozen-pass bulk** — `sample_frozen` is already embarrassingly parallel;
  keep the sequential adapt phase short and put the budget in frozen passes;
  (e) **batch-size vs iteration-count** — ✅ **measured** (addendum S3, note 32):
  a free measurement taken alongside the survey parallelisation, adopted
  nowhere (it moves no gate). Partition-based axes (per-diagram AMP2 shares à la MadEvent
  G-directories, per-diagram-class = per *distinct* map) are second tier:
  real cluster-scale precedent, but they carry the routing fragility and need
  the same coverage guardrail as the per-flow item above.
- **`w_max` from a percentile, not from an extremum** — ✅ **done (addendum S2,
  note 32)**. The frozen scan had a budget of its own (`--scan-points`), and
  that turned out not to be the lever: the maxima never converge. On the llj
  grids `Σⱼ w_maxⱼ ∝ n^0.508` over 2.4 decades of scan budget (10³–2.56·10⁵
  draws/channel, no plateau) while the σ-share above the maxima falls only as
  `n^−0.455` — one statement, a Pareto weight tail of index ≈ 2, confirmed by a
  Hill estimator (α = 2.08–2.40 on the twelve σ-carrying channels, which also
  own the maxima). So no scan budget fixes llj's ~5e-3 σ-share; the rule had to
  change. `MaxRule` implements MadGraph's own `unwgt.f` truncation-ladder rule
  (not literally a percentile — the session corrected that framing): the
  maximum is the lowest scanned weight leaving under `excess_share` (1%) of
  that channel's scanned cross section above it, re-normalised. The estimator
  stays unbiased either way (overweights are kept at weight > 1), so what this
  bought is unweighting efficiency and sample lumpiness, not correctness — the
  five gating rows' acceptance rose from 22.2/20.6/23.3/10.6/4.21% to
  54.1/52.1/52.9/38.9/9.98% at matched budgets, and `p p > l+ l- j` at 300k×8
  needed 2 269 051 trials for 20 000 events before, 477 125 after (4.36× cheaper
  per effective event). `Unweighter::scan` now runs its per-channel scans on a
  rayon `par_iter`, bit-identical at `--max-truncation 0`. (`unweight`, note 31
  §I2, note 32 S2.)
- **Seeds are still combined by inverse variance** — ✅ **done (addendum S1,
  note 32)**. `combine_seeds` (`validate_hadronic.rs`) was the same defect one
  level up from the one the iteration combination shed: a `1/σ²` weighted mean
  over per-seed results, now unweighted (`err = √(Σᵢ σᵢ²)/n`) to match. Every
  hadronic row moved, second-order as expected (≤0.02%). (Note 31 §I1.)
- **Convergence mode is opt-in, and should be the CLI default** — ✅ **done
  (addendum S3, note 32)**. `integrate` now converges to `--target-rel` (0.1%
  default, below every banked reference's own MC error) unless `--fixed-budget`
  asks for a fixed `--neval × --niter` spend; every caller that wanted a fixed
  budget (CLI tests, `generate_samples.sh`, `scripts/acceptance.sh`) now says so
  explicitly. The artifact-byte re-pin this flip implied "resolved empty" — no
  pinned CLI/Pythia-sample artifact actually depended on the old default budget
  path once the explicit-budget callers were updated. (Note 31 §I4.)
- **Stale σ values in the gate sources, left by the budget re-pin** — ✅ **done
  (addendum S8, note 32, this session)**. Re-recorded from fresh runs:
  `validate_hadronic.rs`'s ten hadronic GATE rows, the `RECARDED_ROWS` ladder
  comments and per-row report note, `validate_samples.rs`'s KS p-floor headroom
  comment, and the `ee_to_mumua` `samples` note in `validation/manifest.toml`.
  (Note 31 §I1.)
- **`pp_to_jj`'s integration budget was never bias-set** — ✅ **done (addendum
  S6, note 32), inside the general budget-alignment rule** (size every σ gate's
  budget to the banked reference's own error, floored by seed scatter and
  ladder flatness). `pp_to_jj` was flat to 0.08% across the 75k–600k ladder and
  cut 300k → 75k along with `pp_to_bb_fixed`, `pp_to_bb`, `pp_to_bb_qcd2` and
  `pp_to_ll_scalefact2`; llj stayed at 150k on both the fixed-scale and
  re-carded rows — flat ladders but a dirty 75k rung on the former, a still-climbing
  ladder on the latter. Same host, one sitting: `validate` 360.6 s → 342.6 s
  wall, 1608.9 s → 1321.3 s CPU, census and every tolerance unchanged. (Note 31
  §I1, §6.3; note 32 S6.)
- **The serial tail of a parallel `integrate` run** — ✅ **done (addendum S3,
  note 32; the `-j 16` Amdahl decomposition is note 32 §1.1, corrected by the
  session to 41–52% of the wall rather than ~27%)**. `survey_variance` now runs
  its point loop in one rayon region over I3's deterministic chunking
  (bit-identical at `-j {1,4,16}`, asserted) — its own share of a `--niter 1`
  run fell from 1.04 s/0.24 s of an `-j 16` `p p > l+ l- j`/`p p > e+ e-` wall
  to negligible, and the fitted Amdahl ceilings moved from 6.7×/5.8× to
  18.2×/20.1×. Measured `-j 16` walls on `--fixed-budget --neval 120000
  --niter 12`, min of 5 rounds: `dy13_default` **8.7×**, `pp_to_llj` **9.5×**
  (S8 re-measurement, host load 6–19 from background indexing — see note 32
  §5). Chunk size stayed the second, smaller knob, measured **inert** and never
  tuned. (Note 31 §I3/§I4; note 32 S3.)
- **Per-stage timing capture** — ✅ **done 2026-08-04, note 30.** Both halves
  landed: (a) `validation/madgraph/time_stages.py` regenerates named processes
  into a scratch directory and reads MG's generate / output / compile /
  integrate / events boundaries off a per-line-timestamped transcript, writing
  a host-labelled `timings.json`; (b) every report row carries `duration_s`
  and each run writes one `target/validation-report/host.json` beside its rows,
  rendered as the report's `## Timing` section. Machine identity is recorded in
  full on both sides except the CPU clock, which Apple Silicon does not expose
  through `sysctl` and which is therefore `null` rather than a vendor figure.
  Timings stay out of the refdata bundle. **The regeneration cost is measured:
  1001.6 s = 16 min 42 s wall for all 31 process directories with their
  launches** — the `madgraph` stage of `generate_references.sh` on an M3 Max,
  warm caches, ~20.5 s of it a one-off LHAPDF set install. "Multi-hour" was off
  by an order of magnitude for *that* stage; the `refs` stage (f2py modules,
  amplitude tables, α_s and PDF oracles) stays unmeasured because timing it
  means writing into the reference bank. Left open by note 30 §8: the `refs`
  stage; whether MadEvent's `results.dat` point count includes the survey pass;
  and a per-phase `duration_s` inside a row, which is what would give our side
  a counterpart to MG's `output` + `compile` column.
- **Accept/reject allocator traffic, and the per-event scale that feeds it** —
  **partially done (addendum S5, note 32)**. The unweighting profile was the
  most allocation-bound of the four note-30 profiles: **18.4% allocator + libc
  mem, 5.1% `BTreeMap`**, and kT clustering at its highest share (9.2%) because
  accept/reject re-derives the per-event scale on every trial. S5 landed one
  piece: `ScaleChoice::cluster_scales` rebuilt three `BTree` containers per
  event; `MergeTablesByOrder` now builds one table set per coupling order at
  setup and hands it through `ClusterInput::tables`, cutting `probe_scale_cost`
  ns/point **−16.9%** (`gg_to_gg`), **−21.6%** (`gg_to_ttx`), **−22.3%**
  (`uux_to_uux`), `validate_unweighting` **−16.8%**, the partonic σ gate
  **−11.1%**, byte-identical throughout. **Not done**: `ScaleChoice::clustered`
  still heap-allocates its beam–leg candidate list per event
  (`coupling/scales.rs:376`, `Vec::with_capacity(input.set.n_external)`) and
  `setclscales.rs`'s clustering itself allocates several more `Vec`s per call
  (`attempts`, `traces`, `pt2`, `mt2`, `lines`) that S5 did not touch — the
  scratch-reuse continuation into `setclscales.rs` is the open remainder of E4.
  (`coupling/scales.rs`, `coupling/cluster/setclscales.rs`; `validate_sigma.rs`
  `probe_scale_cost`; note 30 §7.2, note 31 §E4, note 32 S5.)
- **Tighter spacelike floor** — `Cuts::spacelike_floor() = pT_min²` is provable
  but 10–100× looser than the true fiducial floor: S2's D3 measurement found the
  cut-surviving region above `|t| ≈ 4 000–40 000 GeV²` where the floor sits at
  400. A tighter derived bound scales the bounded-`t_max` variance win (measured
  1.67–1.83×) with it. (Note 28 §S2.5.)
- **2→6 σ rows** — ✅ **turned on as `info` at the long tier (addendum S7, note
  32)**. `uux_to_ccx_emmm_qcd0`/`bbx_to_ccx_emmm_qcd0` are `Plan::Long` now, not
  `Plan::Skip` — the skip's "~1 ms/eval" premise was stale by more than an order
  of magnitude (the gate's own harness reads 64/71 µs; even the flat-RAMBO map
  the skip blamed reads only 48/82 µs and is wrong for a different reason: six
  outgoing legs put the poles on a set of vanishing flat measure, so it misses
  these cross sections by eleven and fifteen orders of magnitude despite passing
  cuts on 46% of its draws). The real floor is `MIN_CHANNEL_NEVAL` (512) times
  579/615 per-diagram channels — 296 448/314 880 evaluations every iteration
  whatever budget is asked — and a heavy-tailed multichannel estimator: five
  seeds at 300k/600k/1.2M hold the mean inside 1.1% of a bank whose own error is
  0.30%, but single seeds swing +4.8%/−4.5%/+3.5% at both ends of that ladder
  and do not shrink with budget, which is why the rows are measured (`info`, not
  `gate`) rather than tolerance-bounded. Runs at 600k×8, one seed, on every
  `validate-sigma-2to6` invocation; `ladder-2to6`, `ladder-bb`, `ladder-recarded`
  are now named pixi tasks in `validate-deep`'s long-tier text. Census 98 → 100
  measured; the two `samples` cells stay ⏳ with their cost recorded rather than
  assumed (117/45 trials/event, inside the 400-trial budget, but ~40
  unparallelisable minutes for the pair). (`validate_sigma.rs`; note 32 S7.)
- **Cut before the configuration draw on the fixed-beam path** — ✅ **done
  (addendum S1, note 32)**. `FixedBeamIntegrand` drew the scale configuration
  before checking the cut, so a rejected point still paid `eval_amp2` +
  `set_alpha_s` for a channel selection it would discard — ~190 ns of dead work
  on 22% of `gu_to_epemu`/`gux_to_epemux` points. `scale_u` is always a slice of
  the point's own already-supplied `u`, never an independently advanced RNG
  stream, so cutting first changes no random draw: bit-identical for every
  accepted point. `ProtonIntegrand::shape` already cut first; the fixed-beam
  doc comments that repeated note 30 §6's "points the cuts reject return before
  the draw" (true on the hadronic path, false on the partonic one before this
  fix) are now accurate on both integrands. (Note 31 §E3, note 32 S1.)
- **`feyngraph-perf`** — `AssignWorkspace::assign()` (`workspace.rs:L122`) calls
  itertools `.counts()` (a fresh `HashMap`) per candidate vertex per topology per
  subprocess — ~340M allocations for pp→qq̃4l. Fix: pre-compute per-vertex counts
  in `AssignWorkspace::new()`. Submodule change, dedicated session. Vibegraph-side
  mitigations already applied: topology caching per `(n_ext, n_loops)` and the
  charge-conservation pre-filter (~86% of candidates eliminated).
- **`egraph-rewrite`** (blocked) — remaining rule families are *sharing* rewrites
  invisible to tree-cost extraction; path to yes needs a global/ILP extractor +
  compute-aware `WorkCost` + a ≥3-consumer demo process. Substrate on `main`:
  egglog round-trip skeleton (`egraph.rs`, parked) + the DAG-cost extractor.
  (Notes 14, 15 §4–5; known ±1-CSE-node lowering nondeterminism noted there.)
- **`mg-single-helicity-bench`** — still no consumer. A6 verdict: the fair
  comparison needs an MG single-config timing, which means editing the generated
  Fortran driver + `gen_amplitude.py` and regenerating reference data. E2
  outcome: accept/reject selects helicity off the `eval_hel_m2` diagonal (one
  helicity-summed evaluation per accepted event), so single-helicity evaluation
  never became the hot path. Re-sequence under whatever first needs a single
  fixed helicity in a loop. (Note 23 §E2.)
- **The lane-FMA commit's scalar toll, and `MulAdd` for `NumericArray`** —
  **workaround attempted and killed clean (addendum S9, note 32)**. `3dab3a1`
  shared one real-FMA complex path between the scalar and lane fields (lanes
  −22–35%) because `Complex<NumericArray>` lacks `num_traits::MulAdd`, and its
  own message recorded the price: scalar forward +3.5%, "shipped as-is since
  forward is the least-used path" — but lanes never entered production, so the
  toll lands on the production evaluator. S9's workaround was an in-house
  complex multiply-add trait (default = the shared real-FMA body, `f64`
  override deferring to `Complex<f64>`'s `num_traits::MulAdd`/packed idiom),
  kill-gated on the win still measuring ≥2% on the current tip — but the packed
  idiom turned out to be **x86-specific**: forcing it on this ARM host (M3 Max)
  cost **8–9%**, the opposite of a win, so the pre-registered kill criterion
  fired and nothing merged (worktree clean, branch carries no commit past the
  note-32 planning doc). The clean long-term fix is still an **upstream**
  `numeric_array` contribution implementing `num_traits::MulAdd` (orphan rule
  forbids it in-tree); the in-house-trait design stays at note 32 §2 S9 for
  whoever revisits this on an x86 host, where the original 3.5% may still be
  worth recovering.
- **Per-lane scales** — `eval_m2_lanes` can only batch points sharing one `αs`;
  a SIMD-batched dynamic-scale integrator would need the scaling fused into the
  constant loads. Nothing needs it today. (`helas/eval/rescale.rs`.)
- **`generate-stream` Part B** — lazy `generate_*` iterator (long-tail, from
  `cleanup-refactor`).
- **`Coeff(f64)` → `CoeffRat`** — optional cleanup now that `Op::CoeffRat` exists
  for color; the remaining `f64` leaves (Lorentz-structure and fermi-sign
  coefficients) could migrate. No consumer blocked. (Note 16 §5.)
