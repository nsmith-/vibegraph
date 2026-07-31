# `user-distribution` + `proton-events` — cards → `.lhe` for `p p > l+ l- j` (sprint plan)

**Goal (cross-track acceptance A):** a user with no dev toolchain drives
`p p > l+ l- j` (fixed renormalization/factorization scale) from cards to an
unweighted `.lhe` file entirely through the CLI:

```
vibegraph integrate --proc-card proc_card.dat --run-card run_card.dat --out llj.artifact
vibegraph generate  --artifact llj.artifact --nevents 10000 --out llj.lhe
```

on a machine that started from a downloaded release binary, with the PDF set and
UFO model resolved by name through `~/.vibegraph`.

Two tracks, one sprint:

- **Track P (`proton-events`)** — physics: hadronic multichannel integration and
  event generation at `lpp = 1`, gated against a fixed-scale MG rebank of
  `p p > l+ l- j`. Fixed scale is a deliberate scope cut: dynamical
  `dynamical_scale_choice = -1` beyond 2→2 requires kT clustering
  (`kt-clustering`, feature backlog), which this sprint must not pull in.
- **Track U (`user-distribution`)** — packaging: release binaries, default-PDF
  interning, `~/.vibegraph` model/PDF cache, first-run UX. Acceptance test is
  driving Track P's process end to end from a clean environment.

Process content matches the banked proc card exactly: `l+ = e+ mu+`,
`l- = e- mu-`, `p = j = g u c d s u~ c~ d~ s~` (4-flavor, no b), `QCD=2 QED=2` —
subprocess groups `qq̄ → ℓℓg` and `qg/gq̄ → ℓℓq` (MG's `P1_qq_llg` /
`P1_gq_llq`).

## Dependency check at open (2026-07-30)

Verified now; the opening session re-verifies anything it builds on.

**What exists (Track P substrate):**

- `FixedBeamIntegrand` implements `ChannelIntegrand` (`unweight.rs:55`) —
  per-diagram multichannel over `channel_count × channel_grid_ndim`, per-channel
  VEGAS grids banked in `IntegrateArtifact`, `event_in_channel` +
  `select_event` feeding `generate`'s accept/reject. The whole `generate` stack
  (channel draw `∝ w_maxⱼ`, helicity `∝ |M_hel|²`, flow `∝ JAMP2`, LHEF writer)
  is beam-agnostic *except* for the pieces P4 lists.
- `DrellYanIntegrand` (`hadronic.rs`) — the `(τ, y, cosθ)` PDF convolution, but
  bespoke: 2→2 only, two hard-coded flavor classes (up/down), **no**
  `ChannelIntegrand`, hence `generate`'s by-name `lpp = 0` refusal
  (`vibegraph-cli/src/generate.rs:357`).
- Cuts: `ptj`/`etaj`/`drjl`/`drll`/`ptl`/`etal` all compile in `cuts.rs`
  (jet-class handling incl. `maxjetflavor`); the unimplemented-cut detector
  hard-errors on active cuts we don't implement — the llj card will exercise it.
- Flow → `ICOLUP` dictionary was checked against `leshouche.inc` for 24/24
  banked subprocesses, which include `P1_qq_llg`/`P1_gq_llq`; the banked llj
  `.lhe.gz` is already in the byte-for-byte round-trip corpus. Colored
  *initial* states appear in banked events (`pp_to_bb`), so `ICOLUP` on
  incoming legs is not virgin territory — but our *writer* has never produced
  them.
- Banked MG runs `pp_to_llj` and `pp_to_llj_qcd2_qed2` exist but both use
  `dynamical_scale_choice = -1` (and MG-internal `pdlabel = nn23lo1`), so they
  cannot gate a fixed-scale σ. They stay banked for `kt-clustering`; their
  `validate_scales`/`validate_alphas` asserted-refusals must keep passing.

**What does not exist:**

- No hadronic `ChannelIntegrand` (the sprint's core deliverable).
- No amplitude-level bank for any llj subprocess (no
  `pp_to_llj*_amplitude.csv`).
- No CI workflows at all (`.github/workflows` absent) — Track U starts from
  zero.
- `NNPDF23_lo_as_0130_qed` is not committed; `validation/pdf/fetch.sh` fetches
  on demand.

**Known hazards inherited from the backlog (each has a session owner below):**

1. **Spacelike channel maps at 2→3.** Note 21's t-channel spine was built and
   validated for a *single spacelike line in 2→2*; llj's `qg → ℓℓq` diagrams
   have a single spacelike line in a 2→3 topology (multi-rung ≥2-line ladders
   stay deferred). Whether `diagram_channel.rs` produces a usable map there is
   **the sprint's principal physics risk** — P0 answers it before P2 commits to
   a design. Mitigation if the map is poor: multichannel is unbiased under a
   bad map (α-adaptation down-weights it), so the fallback is slower
   convergence + seed sweeps, not wrong answers — but see the `uux_to_uux`
   collinear-tail lesson (validation backlog): under-coverage can look stable.
   Seed sweeps are part of the gate, not an afterthought
   (resonance-sampling close-out).
2. **Symmetry-factor wrong home.** `FixedBeamIntegrand::new` derives
   `final_state_symmetry_factor` from `amps[0]` and applies it to every
   subprocess (`identical-particle-permutation`, feature backlog). llj final
   states (`ℓ⁺ℓ⁻g`, `ℓ⁺ℓ⁻q`) have no identical particles, so the factor is 1
   everywhere — P2 must *assert* that per subprocess rather than inherit the
   latent bug, or do the map-owned refactor if it falls out cheaply. Do not
   silently extend the `amps[0]` pattern to a multi-subprocess hadronic sum.
3. **Low-mll region.** The banked llj card has `mmll = 0`; the standing
   `low-mll-reconciliation` discrepancy (DY, +2.2% below `m_ll ≈ 20`) lives in
   exactly this corner. The rebank card should set `mmll = 50` so the new GATE
   row is not hostage to a known unresolved discrepancy; a second info-only row
   at `mmll = 0` is welcome but not owed.
4. **μF ≥ 2 GeV veto** — irrelevant here (fixed μF = 91.188), stays backlog.
5. **Pruned-eval frame contract** — `eval_m2` requires partonic-CM
   beams-along-±z momenta. The DY design already honors it (evaluate in
   partonic CM, boost by y for lab-frame cuts); P2 must preserve exactly that
   split.

---

## Track P — `proton-events`

Sessions are serial (each consumes the previous one's artifacts). Branch
`proton-events`, ff-merged at close like prior sprints.

### P0 — open: fixed-scale MG rebank + spacelike-map probe

**Rebank.** New banked run `pp_to_llj_fixed` (name final at session time; keep
it distinct from the two dynamical banks):

- proc card: byte-identical to `pp_to_llj_qcd2_qed2`'s
  (`generate p p > l+ l- j QCD=2 QED=2`, same multiparticle defs).
- run card deltas from the banked one: `fixed_ren_scale = True`,
  `fixed_fac_scale1/2 = True`, `scale = dsqrt_q2fact1/2 = 91.188`;
  `pdlabel = lhapdf`, `lhaid = 247000` (`NNPDF23_lo_as_0130_qed`, the set the
  DY gate already shares — the banked llj run's MG-internal `nn23lo1` is not
  loadable by our LHAPDF6 parser); `mmll = 50` (hazard 3); everything else
  (13 TeV, `ptj = 20`, `ptl = 10`, `etaj = 5`, `etal = 2.5`,
  `drll = drjl = 0.4`) unchanged.
- Bank: σ + run logs, `unweighted_events.lhe.gz`, `leshouche.inc` per
  subprocess, cards — the same shape as existing banked runs, via the
  `extended-validation` skill's regeneration path (respect `--skip-deps`
  semantics; MG runs happen in the **main checkout**, not a worktree, and the
  banked data is committed before any parallel session starts).
- Amplitude bank: MG standalone per-diagram, per-helicity complex amplitude
  dumps for one representative subprocess per class —
  `u u~ > e+ e- g`, `g u > e+ e- u`, and a down-type partner — the
  finest-linear-level oracles P1 gates against ("every oracle has a blind
  spot": |M|² alone cannot see phase/flow conventions on the new
  mixed-color initial states).

**Probe.** Enumerate the llj diagram sets in vibegraph, feed them to the
channel-tree derivation, and *look at the maps*: does `qg → ℓℓq` get a
spacelike spine channel, and does its map integrate a toy integrand sanely at
fixed ŝ? Output is a short written verdict in this note (extend-the-spine /
works-as-is / fall-back-to-flat), because P2's design hangs on it.

**Wire the new bank into the existing asserted lists** (`validate_alphas`,
`validate_scales`): the fixed-scale run should *pass* the constant-scale
branches the dynamical runs are refused on — `SCALUP`/`<rscale>`/`<pdfrwt>`
per-event replay becomes a GATE for this run, the first llj scales row.

### P1 — amplitude gate for the llj subprocesses

`validate_helas_mg` rows for the P0-banked subprocesses: per-diagram ×
per-helicity complex values, CF matrix vs MG's `DATA CF`, JAMP-weighted |M|²,
NHEL pinning — the full convention-channel treatment (note 19's V6 guards
name each channel; the new rows must state which pinned convention each one
exercises). The `qg` initial state is the first *mixed* quark+gluon incoming
color arrangement in the gate; expect fermion-flow/flow-basis subtleties there
before anywhere else (the color-flow sprint's slot-swap history). Tolerances:
whatever the existing 14 processes hold (≤1e-12 vs MG unless bit-exact);
never loosened.

Run the `extended-validation` skill's amplitude gates after this session (it
touches amplitude/color coverage).

### P2 — hadronic multichannel integrand (the heavy design session)

The core deliverable: a hadronic `ChannelIntegrand`, so everything downstream
(per-channel VEGAS banking, frozen-grid scans, `generate`'s accept/reject) is
inherited rather than rebuilt.

**Shape (to be confirmed against the P0 probe):** a `HadronicIntegrand`
wrapping the `FixedBeamIntegrand` machinery with a PDF outer layer:

- Outer map: `(τ, y)` exactly as DY — `ln τ` uniform with `τ_min` from
  `Cuts::shat_min` (with `mmll = 50` and `ptj = 20` both feeding the hint;
  verify the hint is not so loose the grid has to find the ridge alone),
  `y` flat in its τ-dependent range. These two coordinates prepend the inner
  channel map's coordinates; `channel_grid_ndim = 2 + inner_ndim`.
- Inner map: the per-diagram multichannel at **per-event ŝ = τs**, in the
  partonic CM with beams along ±z (hazard 5). The channel trees are
  ŝ-independent structures; their maps take ŝ as input — verify this is
  already true (it should be: `lpp = 0` builds them once for fixed ŝ) and lift
  any baked-in constant.
- Channel space: `(subprocess-group, diagram channel)` pairs, α-adapted
  jointly. Subprocess groups mirror MG's (`qq̄ → ℓℓg`, `qg/gq̄ → ℓℓq`), each
  with a flavor-class decomposition generalizing `dy_flavor_classes` beyond
  the DY up/down pair: group concrete flavors whose |M|² coincide (same
  coupling class, same masses), sum PDF luminosities within a group — incl.
  both beam orderings and the `q ↔ q̄` reflections. This generalization is the
  session's second-largest work item after the map itself; it must be derived
  from the diagram sets, not hand-listed.
- Luminosity: built from `x·f` products as DY does (the `1/x₁x₂` cancellation
  against the `dτ` Jacobian is already worked out in `hadronic.rs`'s module
  doc — keep that arithmetic).
- Cuts in the lab frame after the y-boost, as DY does.
- Symmetry factor: assert `final_state_symmetry_factor == 1` for every llj
  subprocess (hazard 2).
- μF/μR: fixed-scale only this sprint. `EventScaleSource::from_run_card`
  already refuses `dynamical_scale_choice = -1` without a closed form —
  confirm the refusal fires for a dynamical llj card and stays asserted.

**DY stays untouched.** `DrellYanIntegrand` is the bit-reproducibility anchor
(σ(pp→e⁺e⁻) reproduces bit-for-bit from banked artifacts). The new path must
*also* run DY as an **informational** cross-check — same σ within statistics —
per the "keep a known-wrong informational comparison running" convention;
whether the DY special case is later retired is a close-out decision, not a
mid-sprint one.

**In-session validation** (before P3's gate): fixed-ŝ slices of the new
integrand vs `FixedBeamIntegrand` on the same subprocess (PDF layer off ↔
factor of luminosity), and the DY informational row above.

### P3 — σ gate (`integrate` at `lpp = 1`, general path)

- `integrate_proton` routes non-DY processes through the new integrand
  (DY keeps its bespoke branch, hazard/anchor above). Artifact: per-channel
  grids keyed by `(subprocess-group, channel)` — this likely grows the
  `IntegrateArtifact` schema; if so bump `fv3 → fv4` with the fv3 reader kept
  for existing banked artifacts (model-identity fields carry over unchanged).
- **Gate:** σ(pp → ℓℓj, fixed scale) vs the P0 rebank. Acceptance shape as the
  DY rows: agreement at the sub-percent level MG's own error permits, and a
  **five-seed sweep** with pull statistics (`seed-sweep-over-fixed-seed-pull`);
  a single-seed pass is not a pass. DY GATE rows must stay bit-identical.
- The 2→3 cost sits well under the 2→6 `Plan::Skip` wall (~1 ms/eval); if the
  gate needs > a few minutes, that is a finding, not a tolerance discussion.

### P4 — event generation (`generate` at `lpp = 1`)

Extends `generate` past its by-name refusal, only for the path P2/P3 built:

- Channel draw `∝ w_maxⱼ` over the `(subprocess-group, channel)` space
  (note 23's E4 correction applies unchanged); within the drawn group, concrete
  flavor assignment `∝` that flavor's PDF-luminosity share at the event's
  `(x₁, x₂)` — a new per-event step with no `lpp = 0` counterpart. Its rule
  ("∝ luminosity share") needs a pinning test of its own (E1/E2 pattern: pin
  the rule, defer the realised-frequency comparison to the validation pass).
- Kinematics: incoming `PUP = x_i · E_beam` along ±z (lab frame), final state
  boosted by y — the LHE is lab-frame throughout; `XWGTUP`/`IDWTUP` machinery
  unchanged (Buffer default / StochasticRounding swappable).
- `ICOLUP` for colored incoming legs via the existing dictionary — new as
  *writer* output; check against the P0 `leshouche.inc` per subprocess, and
  keep MG's incoming-leg color/anticolor orientation convention (incoming
  quark carries color, incoming antiquark anticolor, gluon both).
- `SCALUP = 91.188` fixed; `AQCDUP = αs(91.188)` (MG's π-truncation defect,
  note 07, applies to how MG *writes* it — match banked formatting, as the
  round-trip already forced for other runs); `SPINUP ∝ |M_hel|²` unchanged.
- **Gates:** (a) the P0-banked fixed-scale `.lhe.gz` joins the byte-for-byte
  re-serialisation corpus; (b) `generate`'s own output self-reads via
  `lhef::parse` with per-event momentum/flavor/color-line consistency checks;
  (c) card/model mismatch refusals extend to the hadronic artifact fields
  (PDF set name in particular — a `generate` against an artifact banked with a
  different PDF must refuse); (d) the dynamical-scale llj cards stay refused
  by name with a message pointing at `kt-clustering`.
- Deliberately deferred (unchanged from note 23): Pythia consumption of the
  emitted `.lhe`, distribution-level event-sample-vs-MG statistics — both are
  the next validation pass's content and now have a second process waiting.

### P close-out

TODO pipeline table row updates (step 5/6 lose their `lpp = 1` deferral),
README user-path rewrite (Track U owns the text), memory + this note's
close-out section, backlog deltas: `identical-particle-permutation` either
absorbed (if P2 did the refactor) or annotated; `low-mll-reconciliation`
gains the llj `mmll = 0` info row if one was banked.

---

## Track U — `user-distribution`

Sessions U1–U3 are independent of Track P and of each other (parallelizable);
U4 integrates them and its final acceptance depends on P4. Branch
`user-distribution`. Light sessions — Sonnet-eligible (see dispatch).

### U1 — release binaries (CI)

`.github/workflows/release.yml` from scratch (no CI exists at all — decide in
session whether a minimal `cargo test` PR workflow rides along; it is in
scope if cheap, it is not the deliverable):

- Matrix: macOS arm64, macOS x86_64, Linux x86_64 (musl or manylinux-style
  glibc floor — session decision, note the choice); `cargo build --release`
  on tag push, artifacts attached to the GitHub release.
- `vibegraph --version` reports the git tag (build-script `git describe`,
  with a clean fallback for non-git builds from a source tarball).
- Acceptance: a tagged test release produces three binaries; each runs
  `--version` and a partonic `integrate` smoke on its platform (macOS x86_64
  via Rosetta or CI runner).

### U2 — default PDF set out of the box

- **License check first**: confirm `NNPDF23_lo_as_0130_qed`'s LHAPDF
  distribution terms permit redistribution (NNPDF sets are CC-BY-4.0; verify
  for this set specifically and record the finding + attribution text here).
- Decision embed-vs-fetch on measured size: the set is not in-repo today
  (fetched on demand); if member 0 is small enough to embed without bloating
  the binary, embed; otherwise first-use fetch from the LHAPDF CDN with
  SHA-256 pinning. Either way `integrate` on a hadronic card must work from a
  bare binary + network (or bare binary alone if embedded).
- The pin lives in code (set name → URL + checksum), not in a config the user
  can silently drift.

### U3 — `~/.vibegraph` cache

Library-level resolution utilities (CLI-agnostic, so tests drive them
directly):

- Layout: `~/.vibegraph/{ufo,pdf}/<name>/`; each entry checksum-pinned
  (UFO: the existing model digest; PDF: SHA-256 of the fetched archive).
- Resolution order, explicit and tested: `--flag` → env
  (`VIBEGRAPH_PDF_DIR` exists; add the UFO counterpart) → `~/.vibegraph` →
  repo-local `validation/pdf` (dev fallback last).
- Sources: LHAPDF `pdfsets` index for PDFs; FeynRules model database for UFO
  models. Network fetch only ever happens after U4's prompt — U3 itself is
  the resolution+storage layer with fetching behind a trait/callback so U4
  owns the interaction policy.

### U4 — first-run UX + acceptance

- Uncached model name / missing PDF set → "fetch it now? [URL, size,
  checksum]" prompt; `--no-network` (and non-TTY default) turns the prompt
  into a clean refusal naming the flag that would allow it. CI-safe by
  construction.
- **Acceptance A (the sprint's exit criterion):** a CI job (or documented
  clean-VM script) that downloads the U1 binary, runs
  `integrate` + `generate` on the fixed-scale llj cards with PDF/model
  resolution through U2/U3, and validates the emitted `.lhe` by re-parsing.
  Until P4 lands, the job runs against `p p > e+ e-` (DY) as scaffolding —
  flipping it to llj is the last commit of the sprint.
- README "quick start" rewrite: download → cards → `.lhe`, no toolchain.

---

## Validation regime (planned with the feature, per convention)

| Gate | Session | Level | Enforced? |
|---|---|---|---|
| llj per-diagram × per-helicity amplitudes vs MG standalone | P1 | finest-linear | GATE |
| CF / JAMP / NHEL for `qq̄→ℓℓg`, `qg→ℓℓq` | P1 | convention channels | GATE |
| Fixed-scale llj scales replay (`SCALUP`/`<rscale>`/`<pdfrwt>`) | P0 | per-event | GATE |
| New-path DY σ vs `DrellYanIntegrand` | P2 | σ, informational → enforced at close | INFO→GATE |
| σ(pp→ℓℓj) vs fixed-scale rebank, 5-seed sweep | P3 | σ | GATE |
| DY σ bit-reproducibility from banked artifacts | P3 (regression) | bit | GATE |
| Fixed-scale llj `.lhe.gz` byte round-trip | P4 | bytes | GATE |
| `generate` llj self-read + flavor/color consistency | P4 | per-event | GATE |
| Flavor-assignment rule pinning (`∝ luminosity share`) | P4 | rule | GATE |
| Dynamical-scale llj cards refused | P0/P4 (regression) | refusal | GATE |
| llj `mmll = 0` σ row | P3 (optional) | σ | INFO |
| Release binaries build + smoke per platform | U1 | build | GATE (CI) |
| Acceptance A: clean-env cards → `.lhe` | U4 | end-to-end | GATE (CI) |

Blind spots, stated up front: the P4 self-read gate cannot see a
self-consistently wrong format (note 23's E4 caveat — Pythia consumption is
the deferred detector); the σ gate cannot see distribution-shape errors that
integrate away (the deferred event-sample comparison is that detector); the
flavor-assignment pinning test checks the rule, not MG's realised frequencies.

## Execution notes (agent dispatch)

- **Steering:** Opus as sprint manager, one session per subagent dispatch.
  `feature-dev` for every session; Opus for P0/P2/P3/P4 and U2, Sonnet
  eligible for P1 (mechanical gate extension over banked data), U1, U3, U4.
  Never `general-purpose` (it ignores model overrides).
- **Worktrees:** pre-create off `main` per session and COW-clone
  `validation/` data in (`cp -c` on APFS); hard cd-verify at session start.
  Isolation has leaked twice before (eval-perf-2), especially on resume.
- **MG banking (P0) runs in the main checkout** — the MG toolchain and banked
  outputs live there; commit the bank to `main` (or the sprint branch) before
  P1+ or any U session runs in parallel, so worktrees fork from a complete
  bank.
- **Cross-track parallelism:** U1–U3 may run alongside P0–P2. U4's final flip
  waits on P4. Both tracks merge to `main` independently when their gates are
  green; Acceptance A is the sprint-close criterion, not a merge blocker for
  Track P.
- After any session touching amplitudes/color/coupling/diagram enumeration
  (P1, P2), invoke the `extended-validation` skill for the gate map.
- Session outcomes get an "### Px/Ux outcome" section appended to this note,
  as in notes 21–23; plan corrections are recorded, not silently absorbed.

---

## P0 outcome (2026-07-30) ✅

Branch `proton-events`. Rebank, amplitude bank, gate wiring and the spacelike-map
probe all landed; two plan corrections are recorded below rather than absorbed.

### What was banked

**`pp_to_llj_fixed`** — `validation/madgraph/scripts/pp_to_llj_fixed.mg5`, built
through the usual `build-diagrams` path in the main checkout.

The proc card is *not* byte-identical to `pp_to_llj_qcd2_qed2`'s and cannot be:
MadGraph writes the output directory name into `proc_card_mg5.dat`, so the two
differ on exactly that one line (`output pp_to_llj_fixed -nojpeg`) and agree
everywhere else — same `generate p p > l+ l- j QCD=2 QED=2`, same five
multiparticle `define`s, same `set` preamble.

Run-card deltas from the banked dynamical run, all confirmed in the written card:

| key | dynamical bank | `pp_to_llj_fixed` |
|---|---|---|
| `fixed_ren_scale` | False | **True** |
| `fixed_fac_scale1` / `fixed_fac_scale2` | False | **True** |
| `scale`, `dsqrt_q2fact1`, `dsqrt_q2fact2` | 91.188 (unused) | 91.188 (**used**) |
| `pdlabel1`/`pdlabel2` | `nn23lo1` | `pdlabel = lhapdf` |
| `lhaid` | 230000 | **247000** |
| `mmll` | 0.0 | **50.0** |

Everything else is untouched: 13 TeV, `ptj = 20`, `ptl = 10`, `etaj = 5`,
`etal = 2.5`, `drll = drjl = 0.4`, `maxjetflavor = 4`, `use_syst = True`,
`dynamical_scale_choice = -1` (inert once the three scales are fixed),
`scalefact = 1`.

Banked result: **σ = 422.84 ± 1.80 pb**, 10 000 unweighted events, subprocess
groups `P1_qq_llg` and `P1_gq_llq` each with their `leshouche.inc`. `use_syst`
is on, so every event carries `<rscale>` and per-beam `<pdfrwt>` alongside
`SCALUP` — all three print `91.188`.

**Amplitude bank** — three concrete subprocesses, one per class plus a flavour
control, each a single-subprocess `lpp = 0` run at `√ŝ = 500` so `launch` builds
both `matrix1_optim.f` (the |M|² oracle) and `matrix1_orig.f` (the per-diagram
`AMP` / per-flow `JAMP` oracle):

| run | process | NGRAPHS | NCOLOR | varies vs the baseline |
|---|---|---|---|---|
| `uux_to_epemg` | `u u~ > e+ e- g` | 4 | 1 | — (baseline) |
| `gu_to_epemu` | `g u > e+ e- u` | 4 | 1 | colour arrangement |
| `ddx_to_epemg` | `d d~ > e+ e- g` | 4 | 1 | initial flavour |

Registered in `build_amplitude.sh` (both `GENERIC_PROCESSES` and
`AMP_PROBE_PROCESSES`, so `mg_amp_probe_<name>` exists for
`compare_amps.py` / a future JAMP reference) and in `gen_amplitude.py`
(75 points each at `√ŝ = 50 / 200 / 500`).

They are **already informational rows in `validate_helas_mg`** and already agree:
`uux_to_epemg` 1.20e-14, `ddx_to_epemg` 1.43e-14, `gu_to_epemu` 3.18e-14
max relative difference. That is |M|²-level only — blind to per-diagram phase and
to the flow conventions P1 is there to pin — but it means P1 starts from a live
end-to-end comparison rather than from nothing.

### Gate wiring

- `validate_scales` gained a `Coverage::Fixed` arm and `FIXED_SCALE_RUNS`.
  `pp_to_llj_fixed` is replayed with **no** topology supplied (a fixed scale must
  not be allowed to hide behind a clustering result) and passes:
  **50 000 scale comparisons over 10 000 events, worst 0.000 of the printing
  budget** — the first llj scales row, and a GATE.
- The two dynamical llj runs stay asserted-refused with
  `ClusteringNotDegenerate`, unchanged. The refusal message already names the kT
  clustering of `cluster.f`.
- Registering an amplitude process also creates a *banked run*: `launch` is what
  builds `matrix1_optim.f`, and it writes 10 000 events. The three new partonic
  runs therefore entered the scales/alphas inventories and were classified —
  `Refused` (same 2→3 clustering as llj), so `CLUSTERING_REQUIRED_RUNS` went
  6 → 9, and added to `validate_alphas`'
  `SCALUP_IS_THE_RENORMALISATION_SCALE`. **Future sessions adding an amplitude
  process must expect to classify a run too.**
- `sigma_reference.json` regenerated (`extract-sigma` banks every `lpp = 0` run):
  the three new partonic σ̂ entries are committed. `validate_sigma` skips them via
  its catch-all `Plan::Skip("no evaluation plan for this directory")`; the 11
  asserted rows are unchanged. Adding partonic σ̂ rows for the llj subprocesses is
  a cheap follow-up for P1/P2, not owed here.

### Plan correction 1 — `pdlabel = lhapdf` puts αs outside our coupling layer

`lhaid = 247000` was the right call for the PDF, but it has a consequence note 24
did not anticipate. With `pdlabel = lhapdf` MadGraph links
`alfas_functions_lhapdf.f`, whose `ALPHAS(Q)` forwards to LHAPDF's
`alphasPDF(Q)`, and `RunningAlphaS::from_run_card` **refuses** such a card
(`AlphaSError::LhapdfRunning`) rather than substituting its own beta-function
solve. The run log is explicit:

```
 Old value of alpha_s from param_card:   0.13000280000000003
 New value of alpha_s from PDF lhapdf :  0.13000271085472234
 alpha_s for scale    91.188000000000002       is   0.13000271085472237
```

The refusal is load-bearing, not bookkeeping: the printed `AQCDUP` is
`1.300027e-01`, while the parameter card's own αs would print `1.300028e-01` —
**2.0× the field's printing budget, and digit-distinguishable on all 10 000
events**. This is now pinned by
`the_grid_alpha_s_runs_are_refused_for_a_measurable_reason`, and both `AQCDUP`
oracles step over the run through an explicit `GRID_ALPHA_S_RUNS` list rather
than by omission. `banked_run_logs_pin_the_alpha_s_source_rule` still checks the
parameter-card half of the source rule for it and asserts that the grid override
is a real change.

**What P3 needs.** σ(pp→ℓℓj) carries one power of αs, so the gate cannot be run
without αs at μR, and the parsed metadata is *not* enough to supply it. Checked
against the set itself:

- `pdf/grid.rs` parses `AlphaSInfo` — `mz`, `order`, `kind`, `qs`, `vals`,
  `lambda4`, `lambda5` — and nothing outside its own parse test reads any of it.
  There is no interpolator.
- The set's `AlphaS_MZ: 0.130003` is six digits, and the 51-knot table brackets
  `91.188` between `Q = 91.1876` (`αs = 0.1300028`) and `Q = 109.8541`
  (`αs = 0.1262725`) at seven digits each. MadGraph's
  `αs(91.188) = 0.13000271085472237` is therefore *not* a tabulated number: it
  is `AlphaS_Type: ipol`'s cubic interpolation in `log Q²` over those knots.
  Reproducing it exactly means writing that consumer.
- Reproducing it *well enough* does not. The bracketing knot at `Q = 91.1876` is
  within `9e-8` relative of MadGraph's value — five orders below any σ tolerance
  — so P3 can pin the single value against the banked run log's
  `alpha_s for scale 91.188... is 0.13000271085472237` line (which
  `validate_alphas` already knows how to read) and defer the interpolator.
- What P3 must **not** do is inherit the current default: `hadronic.rs:1079`
  takes αs from the parameter card (`evaluated.alpha_s()`). For Drell-Yan that
  is harmless — pp→ℓℓ carries no αs at LO — but for llj it is the wrong source,
  giving `0.1300028` where MadGraph used `0.13000271085`. Negligible in σ today,
  and wrong in kind: it would grow with any move off `μ = M_Z`.

### Plan correction 2 — hazard 1 was understated

Note 24 called the 2→3 spacelike map "the sprint's principal physics risk" and
offered the mitigation that "multichannel is unbiased under a bad map". The probe
found the risk is **not** a bad map but a *biased* one, which that mitigation does
not cover. Detail below.

### Probe verdict: **extend the spine — and floor its spacelike draw**

Not "works as is", and not "fall back to flat". Evidence, all now standing as
tests in `vibegraph-lib/tests/diagram_channel.rs`:

**1. The topology.** `u u~ > e+ e- g` — **4 of 4** diagrams carry exactly one
spacelike line; the cuts are `({g} | {ℓ⁺ℓ⁻})` twice and `({ℓ⁺ℓ⁻} | {g})` twice
(the gluon comes off beam 0's line or beam 1's; either way the recorded transfer
is the same invariant). `g u > e+ e- u` — **2 of 4**; the other two route the
internal quark through the full ŝ, which is timelike and bounds no subsystem.
In every case the spacelike line separates the lepton pair from the jet, and the
Z/γ* pole sits on the lepton-pair side. So llj is exactly one spacelike rung in a
three-body final state, with a *composite* subsystem on one side — one step
beyond the 2→2 the spine was built for.

**2. Nothing is built for it today.** `DiagramChannel::from_diagram` guards on
`spacelike.len() == 1 && n_out == 2`; every llj diagram falls through to the
all-timelike tree.

**3. The machinery is dimension-generic.** `sample_spine` / `spine_jacobian`
recurse into composite emitted/recoil nodes and the dimension count works out
(`3·n_out − 4`). Built by hand from each diagram's own cut through the public
`from_topology_tchannel`, a three-body spine produces on-shell, conserving points
and integrates `V_3` to flat RAMBO's value for all six cuts — **provided its
spacelike pole is regulated**.

**4. Unregulated, it is biased — by a factor of three.** With the model's
massless quark exchange the three-body spine overstates `V_3` by
**3.09×–3.48×**. The mechanism: for a massless spacelike line the transfer's
upper edge is `t_max = m² = 0` analytically, but it is computed as a cancelling
difference of two large quantities built from *different* expressions. Over
20 000 recoil invariants at `s = 2.5e5` it lands below zero 6 131 times, above
6 218 times, and exactly zero only 7 651 times, with `|t_max| ≤ 4e-8`. When it
lands just below zero, `t_pole_shapes` switches the propagator draw on with
`N = ln(|t_min|/|t_max|) ≈ 30` e-folds reaching `|t| ~ 1e-11` — while
`density()` recomputes `t` from the momenta with a cancellation error of the same
size. Since `sample()` takes its weight from `density()`, the sampling density
and the weighting density then describe different maps, and the estimator is
biased rather than merely noisy. **This is what invalidates the "multichannel is
unbiased under a bad map" mitigation: the map is not bad, it is wrong.**

The existing 2→2 spine is *not* affected, for two different reasons depending on
the process. With massive final legs (`e+ e- > W+ W-`) the edge sits strictly
below the pole and the propagator draw is well posed. With massless ones
(`u u~ > u u~`) both invariants are the fixed constant `0`, so `center` and
`span` reduce to identical floating-point products and cancel to exactly zero for
every `s` tested — the flat fallback then fires deterministically. The
degeneracy needs a *drawn* invariant on one side, which first appears at
`n_out ≥ 3`. The 2→2 case is safe by arithmetic accident, not by design.

**5. Regulated, it is worth having.** With the pole at `√ŝ/100 = 5 GeV`, on a toy
integrand carrying the two structures an llj matrix element has (the Z
Breit–Wigner on `s_ℓℓ`, a `1/(m₀²−t)²` peripheral factor), over five seeds:

| map | worst seed pull | per-point variance | verdict |
|---|---|---|---|
| regulated spine | 0.4 – 1.3 | 1.1e-16 – 1.0e-14 | self-consistent |
| all-timelike (what `from_diagram` builds today) | up to 4.9 | 5× – 2800× larger | lands up to **3.7×** away |
| flat RAMBO | up to **91.8** | up to 4e5× larger | confidently wrong |

The comparison is deliberately a seed sweep and not a single run: this is the
`uux_to_uux` collinear-tail failure mode, and both alternatives display it —
flat RAMBO returns e.g. `1.29e-8` where the spine returns `5.82e-9`, with a
reported error that does not admit the difference. A single-seed number from
either would have been believed.

### What P2 must take from this

1. **Lifting the `n_out == 2` guard is necessary but not sufficient.** Shipping
   it alone makes llj *worse* than the status quo: a biased map replaces an
   under-covering one. The spacelike draw needs a floor, the way `log_scale`
   already floors a zero-width timelike pole.
2. **The floor is process data, not model data.** The natural scale is the jet
   transverse-momentum cut (`ptj = 20` ⇒ `|t| ≳ 400 GeV²`, eleven orders above
   the 4e-8 noise). That means the channel derivation needs a cut scale as an
   input — a new coupling between `Cuts` and `diagram_channel` that P2's design
   has to accommodate, and which note 24 did not plan for.
3. **The `density == 1/weight` checks are vacuous.** `sample()` *defines* the
   weight as `1.0 / self.density(&momenta)`, so reciprocity holds by
   construction and cannot detect a sampling/weighting mismatch — the exact
   defect above. Both sites (`assert_valid` in the integration test and the
   in-crate `density_is_reciprocal_weight`) now carry a comment stating what they
   can and cannot see, so they read as a declared blind spot rather than as
   coverage; the assertions themselves are unchanged, since they still pin that
   the density is finite and non-zero on every generated point. **Closing the
   blind spot is a P2 item**: it needs `sample` to accumulate its own path
   weight as it walks, and compare that against `density` — until then, any new
   map must be checked by an integrated quantity (`V_n` against flat RAMBO, or a
   seed sweep), which is what actually caught this.
4. Half the `g q` diagrams have no spacelike line at all (the internal quark
   carries the full ŝ). The channel set for that group is therefore mixed:
   peripheral spines plus all-timelike trees, which the `MultiChannel` combiner
   already handles heterogeneously.

### Gate results

`cargo test --workspace` 539 passed / 0 failed. Extended validation, all green:

| gate | result |
|---|---|
| `validate-helas-mg` | 17 passed (14 enforced unchanged; 3 new llj rows informational) |
| `validate-scales` | 7 passed — `pp_to_llj_fixed` 50 000 comparisons, worst 0.000 of budget |
| `validate-alphas` | 4 passed — source rule pinned against 24 run logs |
| `validate-color-jamp` | 3 passed |
| `validate-diagrams` | 16 passed |
| `validate-sigma` | 11 rows asserted, unchanged (3 new dirs auto-skip) |
| `validate-lhef` | 3 passed |
| `validate-unweighting` | 1 passed |
| `validate-scale-couplings` | 1 passed |

No tolerance was changed anywhere.

### Files and commands

- Bank: `validation/madgraph/scripts/pp_to_llj_fixed.mg5`,
  `uux_to_epemg.mg5`, `gu_to_epemu.mg5`, `ddx_to_epemg.mg5`;
  registrations in `validation/madgraph/build_amplitude.sh` and
  `gen_amplitude.py`; `validation/madgraph/sigma_reference.json`.
  Regenerate with `pixi run -e madgraph build-diagrams` then
  `generate-amplitude` and `extract-sigma` (`validation/madgraph/output/` is
  git-ignored, as for every other run).
- Gates: `vibegraph-lib/tests/validate_scales.rs`,
  `vibegraph-lib/tests/validate_alphas.rs`.
- Probe: `vibegraph-lib/tests/diagram_channel.rs` (five new tests, in the
  default `cargo test` gate).

One MG-side detail worth keeping: `build.sh` must run with `LDFLAGS` carrying
`-lc++` for an LHAPDF run to link on macOS, the same workaround
`gen_hadronic_sigma.sh` documents. `pp_to_llj_fixed` is the first `build.sh`
process that needs it.

---

## P1 outcome (2026-07-30) ✅

Branch `proton-events`. All four llj amplitude rows are enforced, the `q̄ g`
coverage gap is closed, and a new per-diagram oracle carries the enforcement at
the finest linear level MadGraph exposes for a single-flow process. No tolerance
was changed anywhere.

### Rows enforced, and what each pins

`validate_helas_mg`'s `EXPECT_MATCH` went 14 → 18 (all at `REL_TOL = 1e-12`):

| row | max rel vs MG | convention channel it exercises |
|---|---|---|
| `uux_to_epemg` (`u u~ > e+ e- g`) | 1.20e-14 | reversed-FFV-bilinear parity, on all 4 diagrams, now on a **coloured** spine (previously only `e+ e- > mu+ mu-`'s leptonic one) |
| `ddx_to_epemg` (`d d~ > e+ e- g`) | 1.43e-14 | same channel, initial flavour varied — a disagreement localises against the row above |
| `gu_to_epemu` (`g u > e+ e- u`) | 3.18e-14 | crossed-line **spine sign**, on all 4 diagrams, in the first mixed adjoint+fundamental initial state |
| `gux_to_epemux` (`g u~ > e+ e- u~`) | 1.14e-14 | the same, with the fermion line entering as an antiparticle: MadGraph's colour structure flips `T(1,5,2)` → `T(1,2,5)` and its JAMP coefficients `+1` → `−1` |

Channel counts (`(vvv, spine, build, reversed)` over the canonical rooting, the
quantity `mg_guard_processes_exercise_every_convention_channel` reads):
`u u~ > e+ e- g` and `d d~ > e+ e- g` are `(0, 0, 0, 4)`; `g u > e+ e- u` and
`g u~ > e+ e- u~` are both `(0, 4, 0, 4)`. So the `q q̄` and `g q` classes differ
in exactly one channel, the spine sign — and it fires **uniformly**, which
matters below.

### The `q̄ g` gap, closed

`validation/madgraph/scripts/gux_to_epemux.mg5`, banked exactly as P0 costed it:
one `.mg5` script plus registrations in `build_amplitude.sh` (both lists) and
`gen_amplitude.py`. NGRAPHS=4, NCOLOR=1, σ̂ = 0.11816 ± 0.00026 pb at √ŝ = 500
against `gu_to_epemu`'s 0.11812 ± 0.00022.

P0's warning held: registering an amplitude process banks a run.
`CLUSTERING_REQUIRED_RUNS` went 9 → 10, `SCALUP_IS_THE_RENORMALISATION_SCALE`
gained the run, and `sigma_reference.json` gained its σ̂ (auto-skipped by
`validate_sigma`, as its three siblings are). The `-lc++` `LDFLAGS` workaround was
not needed — it is an LHAPDF-link requirement, and this is an `lpp = 0` run.

MadGraph's own colour data for the four:

| run | structure | JAMP coefficients | CF |
|---|---|---|---|
| `uux_to_epemg`, `ddx_to_epemg` | `T(5,2,1)` | `(+1, +1, +1, +1)` | 4 |
| `gu_to_epemu` | `T(1,5,2)` | `(+1, +1, +1, +1)` | 4 |
| `gux_to_epemux` | `T(1,2,5)` | `(−1, −1, −1, −1)` | 4 |

### New gate: `amp_diagram_oracle`

`validate_helas_mg` compares one real number per point. `jamp_reference.json` does
not apply at NCOLOR=1 (a single flow's JAMP is the coherent sum, which adds only a
global phase over |M|²), so the finest available oracle is `AMP(1:NGRAPHS)` per
helicity. `gen_amp_reference.py` banks it — plus the colour coefficients that
build `JAMP(1)` out of it — into `amp_reference.json`;
`vibegraph-lib/tests/amp_diagram_oracle.rs` consumes it. Pixi:
`generate-amp-reference`, `validate-amp-diagram`.

**Plan correction: the comparable object is `c_i · AMP(i)`, not `AMP(i)`.** The
first cut of the oracle compared diagram roots against raw `AMP(i)` and
immediately failed on `e+ e- > e+ e-` with a contaminated fit constant. The cause
is a convention split note 24 did not anticipate: **MadGraph puts the
annihilation/exchange relative sign in the colour coefficient** (`e+ e- > e+ e-`
carries `c = (−1, −1, +1, +1)`) **where vibegraph puts it in the diagram root**
(`fermi_sign`, colour coefficients uniformly +1). Neither is observable alone;
the product is. Comparing against `c_i · AMP(i)` makes the relation uniform across
all seven banked processes with a single constant each. The parse of the
`JAMP(1,1) = Σ c_i AMP(i)` statement is verified numerically against the probe's
own `JAMP(1)` at every banked point, so a mis-parse cannot pass.

What is asserted, per process: the helicity set equals MadGraph's `NHEL` table and
the diagram count equals `NGRAPHS`; **one** complex constant `G` fitted over every
(point, helicity, diagram) entry satisfies `A_i^vg = G · c_j · AMP_j^mg`
element-wise; `|G| = 1` **and `Re G = 0`** — the constant is `±i` and nothing
else, which pins the single factor of `i` vibegraph's roots carry and MadGraph's
`AMP()` does not; and the coherent amplitude follows the *same* `G`, which is what
pins vibegraph's own colour coefficients against MadGraph's.

Three already-enforced processes ride along as controls, one per rooting-sign
channel: `ee_to_ee` (crossed-line spine, and the only banked process with
non-uniform colour coefficients), `ee_to_wpwm` (Yang-Mills VVV), `ee_to_tatah`
(scalar bilinear). Results, worst 8.09e-15 over all seven:

```
  [ee_to_ee]      NGRAPHS=4: per-diagram 8.09e-15, coherent 7.83e-15 (G = -1i)
  [ee_to_wpwm]    NGRAPHS=3: per-diagram 1.28e-15, coherent 7.62e-16 (G = +1i)
  [ee_to_tatah]   NGRAPHS=5: per-diagram 5.81e-15, coherent 5.81e-15 (G = -1i)
  [uux_to_epemg]  NGRAPHS=4: per-diagram 1.37e-16, coherent 2.16e-16 (G = +1i)
  [ddx_to_epemg]  NGRAPHS=4: per-diagram 1.34e-15, coherent 1.20e-15 (G = +1i)
  [gu_to_epemu]   NGRAPHS=4: per-diagram 1.42e-15, coherent 1.30e-15 (G = -1i)
  [gux_to_epemux] NGRAPHS=4: per-diagram 3.92e-16, coherent 1.38e-15 (G = +1i)
```

**Second finding: the two enumeration orders are not always the same.** They agree
for six of the seven; `e+ e- > ta+ ta- H` does not, and its pairing `[3,4,1,2,0]`
is banked in `MG_DIAGRAM_ORDER` rather than searched for at run time, so a
reordering on either side fails the gate instead of being silently re-matched.
The pairing is over-determined by what it has to reproduce (5 points × 16
helicities × 5 diagrams under one constant), so banking it is not fitting it.

### Everything passed first try — so, what would have been caught?

Four mutation experiments, each reverted, run against the whole amplitude/colour
suite. This is the answer to "a gate that cannot see the convention is not
confirmation of it".

**1. Transpose the `T` chain's two fundamental ends in the flow derivation.**
Caught by `color_flow_tags_oracle` (22/30), `validate_helas_mg` (10/18, all four
llj rows among them), `amp_diagram_oracle` (all four llj rows),
`color_jamp_oracle` (2/3) and `validate_lhef`. `color_cf_oracle` is blind — a
uniform transpose is exactly its documented blind spot.

**2. Remove the historical MadGraph fermion slot swap in `slot_indices`** — i.e.
reintroduce the note-16 §6 bug class. Caught by `validate_helas_mg` (10/18,
including all four llj rows), `amp_diagram_oracle` (4/7, the four llj rows),
`color_flow_tags_oracle` (22/30), `color_jamp_oracle` (2/3). `color_cf_oracle`
blind again. So the new rows are live detectors of the colour-flow slot
convention, not passengers.

**3. Drop the spine sign from `fermi_sign` — a measured blind spot of all four new
rows.** Caught by `validate_helas_mg` on 7 rows (`ee_to_ee`, `ee_to_wpwm`,
`uux_to_uux`, `ee_to_mumua`, `ee_to_mumu_tata_qcd0`, `uux_to_ccx_emmm_qcd0`,
`bbx_to_ccx_emmm_qcd0`) and by `amp_diagram_oracle` on `ee_to_ee` and
`ee_to_wpwm` — and by **none of the four llj rows**. The reason is the channel
count above: the spine sign fires on *all four* diagrams of `g q → ℓℓq`, so
dropping it is a global sign, and a global sign is unobservable in |M|² and
absorbed into `G` by construction. The channel is pinned non-uniformly elsewhere
(`e+ e- > e+ e-` fires it on 2 of 4) and cross-checked by
`spine_sign_from_flow_matches_heuristic`. **The llj rows do not pin the spine
sign; do not claim they do.**

**4. A slot rule wrong only for the arrangement `g u~` uniquely introduces**
(adjoint index incoming *and* fundamental end incoming). Caught by exactly one
trial in each of three gates, and it is `gux_to_epemux` every time; 29 other
flow-tag trials, all of `color_cf_oracle`, all of `color_jamp_oracle` and all of
`validate_lhef` pass. Before this session that defect had no detector at all.

**But experiment 4 also exposes a limit worth stating.** It surfaces as a hard
`inconsistent color flow` compile error, not as a numerical disagreement with
MadGraph — because for a single-flow structure with one adjoint index there are
exactly two colour lines and the leg reps determine them completely. So
`color_flow_tags_oracle` has essentially **no free convention to check** on any of
the four llj rows: its connectivity comparison is forced, and only its
report-only label comparison (`labels=identical to MG`) carries information.
The colour arrangement of `g q̄ → ℓℓ q̄` is pinned by construction, not by
comparison. What the row genuinely adds is its per-diagram spinor content in that
arrangement, checked against MadGraph at 3.9e-16, and its |M|² at 1.14e-14.

Stated the other way: `T(1,2,5)` versus `T(1,5,2)` with the same `CF = 4` and the
same forced connectivity is a relabelling with no observable consequence at
NCOLOR=1 — which is why MadGraph is free to flip the JAMP sign, and why
`amp_diagram_oracle`'s `G` legitimately comes out `+i` for `g u~` where it is
`−i` for `g u`.

### Gate results

`cargo test --workspace`: 539 passed / 0 failed (unchanged from P0). Extended
validation, all green:

| gate | result |
|---|---|
| `validate-helas-mg` | **18 passed** (14 unchanged + 4 llj rows now enforced) |
| `validate-amp-diagram` | **7 passed** (new), worst 8.09e-15 |
| `validate-color-cf` | 30 passed (new pixi task for an existing gate) |
| `validate-color-flow-tags` | 30 passed (new pixi task for an existing gate) |
| `validate-color-jamp` | 3 passed |
| `validate-diagrams` | 16 passed |
| `validate-scales` | 7 passed |
| `validate-alphas` | 4 passed |
| `validate-sigma` | 11 rows asserted, unchanged |
| `validate-lhef` | 3 passed |
| `validate-unweighting` | 1 passed |
| `validate-scale-couplings` | 1 passed |

`color_cf_oracle` and `color_flow_tags_oracle` had no pixi entry, so the
`extended-validation` skill's colour instruction could not actually be followed
for two of the three colour gates; both now have one.

### What P2 must take from this

1. **Every llj subprocess class now has an enforced amplitude row**, so the
   hadronic integrand can be built against a matrix element that is pinned
   per-diagram, not just per-|M|². The four banked partonic runs (`√ŝ = 50 / 200
   / 500`, 75 points each) are also ready-made fixed-ŝ slices for P2's
   "PDF layer off ↔ factor of luminosity" in-session check.
2. **`g q` and `g q̄` are separate rows and separate flavour-class members.** They
   have the same σ̂ to within MC error and the same CF, but different colour
   structures and different `G`. When P2 derives flavour classes from the diagram
   sets, `q ↔ q̄` reflections may be grouped by |M|² equality — that grouping is
   sound here, and the gate above is what makes it checkable rather than assumed.
3. **Do not lean on `color_flow_tags_oracle` for llj colour correctness.** As
   measured above it is forced by the leg reps for these single-flow,
   one-adjoint processes. If P4 emits llj events, the ICOLUP it writes is
   determined; nothing about it is being validated by comparison.
4. A cheap follow-up neither P0 nor P1 owed: partonic σ̂ rows for the four llj
   subprocesses (`sigma_reference.json` already holds the numbers,
   `validate_sigma` skips them for want of an evaluation plan). That would give
   P2 a σ-level check at fixed ŝ before the PDF layer goes on.

### Files and commands

- New: `validation/madgraph/scripts/gux_to_epemux.mg5`,
  `validation/madgraph/gen_amp_reference.py`,
  `validation/madgraph/amp_reference.json`,
  `vibegraph-lib/tests/amp_diagram_oracle.rs`.
- Modified: `validation/madgraph/build_amplitude.sh`,
  `validation/madgraph/gen_amplitude.py`,
  `validation/madgraph/sigma_reference.json`, `pixi.toml`,
  `vibegraph-lib/Cargo.toml`, `vibegraph-lib/tests/validate_helas_mg.rs`,
  `vibegraph-lib/tests/validate_scales.rs`,
  `vibegraph-lib/tests/validate_alphas.rs`.
- Regenerate: `pixi run -e madgraph build-diagrams`, then `generate-amplitude`,
  `generate-amp-reference`, `extract-sigma`.

---

## P2 outcome (partial, 2026-07-30) ⚠️ INCOMPLETE — one commit landed, integrand not built

Branch `proton-events`, commit `1d8e9bf`. The session was cut off twice by
infrastructure stalls; the phase-space groundwork is committed and green, the
hadronic integrand itself is **not started**. This section records what is proven,
what the design settled on, and the order a successor should resume in.

### Landed: `1d8e9bf` — per-energy channels, spacelike floor, real weight check

Three changes to `phasespace`, all with every existing caller unchanged and the
default `cargo test` gate green (`cargo build` clean, 489 lib + 10 diagram-channel
+ CLI suites passing; no tolerance loosened).

1. **`ScaledChannel` / `ScaledMultiChannel`** (`phasespace/channel.rs`). `ndim` is
   *not* restated on `ScaledChannel` — it is a **subtrait of `PhaseSpaceMap`**, so
   exactly one `ndim` is in scope and no call site needs disambiguation. (The first
   cut declared `ndim` on both traits and produced ten E0034 ambiguities; the
   subtrait shape is the fix, not disambiguation syntax.) `DiagramChannel` stores
   `beam_masses` instead of pre-built `beams` and rebuilds them at the draw's
   energy. `kleiss_pittau_step` and `select_channel` are factored out as free
   functions so the reallocation rule is written once for both combiners.
2. **`from_diagram_regulated(d, model, sqrt_s, floor)`** floors the peripheral
   rung at `max(m², floor)`. A spine is built for `n_out > 2` **only** when a
   positive floor is supplied, so every existing `lpp = 0` caller (floor `0`)
   keeps bit-identical channels and the banked σ artifacts are untouched.
3. **`sample` accumulates its own path weight** as it walks, from the invariants
   it drew, instead of returning `1/self.density(momenta)`.

### Plan correction 3 — P0's bias was an artefact of the weight definition

This is the session's main finding and it **supersedes P0's "extend the spine —
and floor its spacelike draw" verdict in its stated form**.

With `sample` weighting by its own walk, the *unregulated* three-body spine
reproduces flat RAMBO's `V_3` to **1.003**. P0's measured 3.09×–3.48× overstatement
was produced by `sample` taking its weight from `density()`, which recomputes `t`
from the momenta with a cancellation error the drawn `t` does not have. P0's test
`the_unregulated_three_body_spine_is_biased_at_the_collinear_edge` therefore no
longer holds and has been replaced.

**The floor is still required, one level up.** `MultiChannel::sample_channel` and
`MultiChannel::sample` *discard* the channel's own weight and use
`αⱼ / Σₖ αₖ gₖ`, every `gₖ` from `Channel::density`. So the combiner — the only
form the hadronic integrand will use — is exposed to exactly the mismatch the walk
weight avoids. Measured over the six llj cuts at `√ŝ = 500`:

| spacelike pole | worst walk-vs-density gap | non-positive self-densities |
|---|---|---|
| unregulated (`m = 0`) | **3.99e4** | **145** of 800 000 |
| floored at 5 GeV | 1.2e-8 | 0 |

A non-positive density at a point the channel itself generated is a zero in the
combiner's denominator, not a mis-normalisation: an unfloored spine trips
`MultiChannel`'s `debug_assert!(g > 0)` outright. So the correct statement is
*"the unfloored spine breaks the density contract a combiner rests on"*, not
*"the unfloored spine is biased"*. Two tests in
`vibegraph-lib/tests/diagram_channel.rs` assert this:
`an_unregulated_three_body_spine_breaks_the_density_a_combiner_weights_by` and
`an_unregulated_spine_breaks_the_positive_density_contract`.

Both P0-annotated "vacuous reciprocity" sites lost their blind-spot comments,
because the blind spot is gone: the check now compares two independent
computations. New bound `WALK_DENSITY_TOL = 1e-7`, above the measured 7.1e-9
(worst over every diagram channel) and 1.2e-8 (floored llj spine), and twelve
orders below the 4e4 it exists to catch.

### Not started

Everything else in the P2 brief: `Cuts`-sourced floor, flavour groups, the
integrand, αs from the grid, the DY informational row, the fixed-ŝ slices.

### Design decisions already made — a successor should not re-derive these

Measured during the session (enumeration run against `sm-default`), all of it
input to the integrand's shape:

- **`p p > l+ l- j QCD=2 QED=2` enumerates 24 non-empty subprocesses**, and the
  enumerator emits **one ordering per unordered initial state**: `g u`, `g u~`,
  `u u~` are present; `u g`, `u~ g`, `u~ u` are **not**. `l+ l-` expands to `e`
  and `mu` (opposite-flavour pairs have no diagrams), so each initial state
  appears twice. Every subprocess has 4 diagrams and
  `final_state_symmetry_factor == 1`.
- **The mirror term is mandatory and is not a symmetry assumption.** DY gets away
  with summing both luminosity orderings against one `|M|²` because its map is
  symmetric; llj does not. The exact identity is `A(b,a; k) = A(a,b; Rk)` with `R`
  the rotation by π about x (`py, pz → −py, −pz`), since `Rp₁ = p₂`. So each group
  contributes `xf_a(x₁)xf_b(x₂)·|M(k)|² + xf_b(x₁)xf_a(x₂)·|M(Rk)|²`, both under
  the *same* cut indicator (the final state is the same). **This is a convention
  claim and needs a pinning test**: compare against an explicitly generated
  `u~ u > e+ e- g` amplitude, which P1's enforced rows make meaningful.
- **Group by measured `|M|²` equality, not by a hand-listed flavour table.**
  Probe every compiled subprocess at a few shared fixed phase-space points and
  group on relative agreement; assert within-group agreement at fresh points *and*
  cross-group disagreement, so the partition is neither assumed nor vacuous. This
  puts `u`/`c` together, `d`/`s` together, `e`/`mu` together (same initial state →
  the luminosity sum handles the multiplicity of 2 uniformly), and separates the
  up/down coupling classes. Whether `g q` and `g q̄` coincide pointwise is
  **unmeasured** — P1 only showed equal σ̂ within MC error — so let the probe decide
  the group count (expect 6, possibly 8).
- **Channel coverage across groups is already adequate; no mirrored channels
  needed.** `u u~ > e+ e- g` has 4/4 diagrams spacelike with partitions
  `emitted = {jet}` twice and `emitted = {ℓℓ}` twice; `g u > e+ e- u` has 2/4, both
  `emitted = {jet}`. The `g q` group's *mirror* peak is at small `(p_b1 − p_jet)²`,
  which by momentum conservation **equals** `(p_b0 − p_ℓℓ)²` — covered by the `q q̄`
  groups' `emitted = {ℓℓ}` spines. Since all channels pool into one `g = Σ αₖ gₖ`,
  coverage is fine. Channels are heavily duplicated across groups (up/down produce
  identical maps); harmless, α just splits, but worth a dedup follow-up.
- **The floor value.** `ptj = 20 ⇒ |t| ≳ 400 GeV²`, eleven orders above the 4e-8
  cancellation noise. Derive it as the largest single-leg pT threshold the cuts
  impose on a final-state leg (momentum balance forces the recoiling system to
  carry at least that). It is a **density regulator**: any positive value leaves
  the estimator unbiased, so its size is an efficiency-and-well-posedness choice,
  not a correctness one. A run with no active pT cut gets floor 0 and falls back
  to the all-timelike tree — the honest failure mode.
- **`τ_min` is loose by ≈2.2× and that is acceptable.** `Cuts::shat_min_hint`
  fires only its `mmll` branch for llj (the `2·ptl` branch needs exactly two final
  legs), giving `ŝ_min = 2500`. The true bound is
  `(√(mmll² + ptj²) + ptj)² ≈ 5454`. In a `ln τ` map that is 0.79 out of
  `ln(1/τ_min) ≈ 18.9`, i.e. ~4% of draws below threshold. Tightening it is
  optional; if done, keep DY's number **exactly** unchanged (DY is the
  bit-reproducibility anchor and `DrellYanIntegrand` is the only other consumer).
- **αs from the grid is a ~10-line interpolator, and log-linear is enough here.**
  `NNPDF23_lo_as_0130_qed` brackets `Q = 91.188` between knots 91.1876
  (`αs = 0.1300028`) and 109.8541 (`αs = 0.1262725`). Linear in `ln Q²` gives
  `0.13000271217` against MadGraph's `0.13000271085472237` — **1.0e-8 relative**,
  five orders below any σ tolerance. Mid-interval the cubic `ipol` and the linear
  interpolant differ by ~1.7e-4 relative, which is precisely why this suffices for
  a *fixed* scale sitting on a knot and would not for a dynamical one. Put
  `GridAlphaS` in `pdf/`, select it over `RunningAlphaS` when the card says
  `pdlabel = lhapdf` (mirroring MadGraph's own linking decision), and pin the single
  value against the banked run log.
- **Module placement:** put the new integrand in a new `vibegraph-lib/src/proton.rs`
  rather than growing `hadronic.rs` (already 2573 lines), making `BoundSubprocess`,
  `EventScaleSource` and friends `pub(crate)`. That keeps DY genuinely untouched.
- **`validation/pdf/NNPDF23_lo_as_0130_qed/` is not fetched in this checkout.**
  It was populated during the session by copying
  `.pixi/envs/madgraph/share/LHAPDF/NNPDF23_lo_as_0130_qed` (gitignored, 101 MB);
  `bash validation/pdf/fetch.sh` does the same from the network.

### Resume order

1. `Cuts::spacelike_floor()` (GeV²) + its unit tests. Cheap, self-contained, commit.
2. `GridAlphaS` in `pdf/` + `AlphaSSource` selection + the pinning test against the
   banked `alpha_s for scale 91.188... is 0.13000271085472237` run-log line. Commit.
3. Flavour-group derivation in `proton.rs` (probe-and-group, with the
   within-group/cross-group tests and the `symmetry_factor == 1` assert), plus the
   `A(b,a;k) = A(a,b;Rk)` pinning test against an explicitly generated `u~ u`
   subprocess. Commit — this is the largest single piece and stands alone.
4. `ProtonIntegrand`: `(τ, y)` outer map exactly as DY (keep `hadronic.rs`'s
   `1/x₁x₂`-vs-`dτ` arithmetic), inner `ScaledMultiChannel` at `√ŝ = √(τs)`,
   `channel_grid_ndim = 2 + 5 = 7`, `impl ChannelIntegrand`. Per-group prefactor
   `avg_g · S_g` inside the sum (the `g q` average is `1/96`, `q q̄` is `1/36`).
   Commit.
5. Joint α-adaptation over the `(group, diagram)` channel space, driven by the
   integrand (the outer map means it cannot live in the combiner);
   `kleiss_pittau_step` is already factored out for this.
6. In-session validation: fixed-ŝ slices vs `FixedBeamIntegrand` on P1's four
   banked partonic runs, and the DY informational row. Then the
   `extended-validation` skill.

### What P3 must know regardless

- The `(group, channel)` space is ~24 channels for llj, each wanting its own VEGAS
  grid; `MIN_CHANNEL_NEVAL = 512` puts a floor of ~12 000 evals/iter on the run.
- Every point evaluates one `|M|²` per group **twice** (direct + mirror), so ~12
  amplitude evaluations per point for llj. If P3's gate runs long, that is the
  reason, and channel dedup is the first lever.

---

## P2b outcome (2026-07-30) ✅ — the first two resume items

Branch `proton-events`, two commits. This session took items 1 and 2 of P2's resume
order: the cut-derived spacelike floor, and `αs` from the PDF grid. Items 3–6
(flavour groups, `ProtonIntegrand`, α-adaptation, the fixed-ŝ validation) are
untouched and their design in the P2 section stands.

### 1. `Cuts::spacelike_floor()` — the floor's scale is process data

`Cuts` already compiles the run card's single-leg `pT` thresholds into a per-leg
check list, so it is the object that can state the scale, and reading it off the
*compiled* cuts means class membership decides which legs contribute: a `pdg = 5`
leg carries `ptj = 20` at `maxjetflavor = 5` and nothing at `4`, where MadGraph
makes it a b with `ptb = 0`.

```
spacelike_floor() = (max over compiled single-leg cuts of pt_min)²
```

`ptj = 20` over `ptl = 10` gives **400 GeV²** for `p p > l+ l- j`, and `0` for a card
with no active `pT` cut — which leaves a peripheral spine unbuilt past two outgoing
legs rather than building an ill-posed one.

**The derivation, as implemented.** A peripheral rung off a massless beam that puts
a massless system at transverse momentum `pT` transfers
`|t| = 2E_beam(m² + pT²)/(E + p_z) ≥ pT²`. That is a *bound* wherever transverse
balance ties the two sides of the rung together — for a three-body final state it
always does, since the system opposite the jet carries the jet's own `pT` — and it
degrades to a *scale* past three outgoing legs, where a partition can balance
internally. The bound itself is pinned by
`a_transverse_momentum_threshold_bounds_the_transfer_it_implies`, which computes `t`
from momenta over 40 rapidities × 3 collision energies rather than asserting the
algebra. The degradation costs nothing: the floor enters `draw_t` and `t_measure`
alike, so any non-negative value leaves the estimator unbiased and only the
efficiency (and the well-posedness of the draw) depends on the size.

**`lpp = 0` is bit-identical, measured not assumed.**
`a_zero_spacelike_floor_leaves_every_channel_bit_identical` draws 200 points from
each of **17** channels over five processes (`e+ e- > mu+ mu-`, `u u~ > u u~`,
`u u~ > d d~ g`, both llj classes) through `from_diagram` and through
`from_diagram_regulated(…, 0.0)` and compares **bits** — momenta, walk weight and
density. All 17 identical. **9 of the 17 move** when floored at 400, which is what
stops the identity from being vacuous.

**Correction worth carrying forward:** the floor is applied as
`t_mass2 = max(m², floor)` for *any* spine, including the `2 → 2` ones that are
built without a floor. So handing a positive floor to a hadronic `2 → 2` process
changes its channels (unbiased, but not the banked map). `ProtonIntegrand` should
pass `cuts.spacelike_floor()` because it wants the three-body spine, not as a
blanket default.

### 2. `GridAlphaS` + `AlphaSSource` — the source, pinned in two places

`vibegraph-lib/src/pdf/alphas.rs` reads a set's `AlphaS_Qs` / `AlphaS_Vals` knots
with a linear interpolant in `log Q²`: endpoint-exact (a knot returns its tabulated
value bit-for-bit), refusing a scale outside the table and refusing any
`AlphaS_Type` other than `ipol` — an `analytic` set derives `αs` from `Λ_QCD` and
reading its table would report the wrong source. Since `log Q² = 2 log Q` the factor
of two cancels out of the interpolation parameter, so this is also linear in `log Q`;
the `log Q²` spelling is LHAPDF's.

`coupling::alphas::AlphaSSource` makes the choice, from the same field MadGraph
makes it from — it calls `RunningAlphaS::from_run_card` and routes only its
`LhapdfRunning` refusal to the grid, so there is one statement of the source rule
rather than two. Two new refusals: `GridUnavailable` (the label wants a set and none
was supplied — *not* a fall back to the β-function solve, which would run the set's
densities against a coupling the set was not fitted with) and `Grid(..)`.

Wired through `EventScaleSource`, which now holds `Option<AlphaSSource>` and takes
the set's `AlphaS_*` metadata in `from_run_card`; `running_alpha_s()` →
`alpha_s()` / `alpha_s_source()` on the two integrands, since the thing is no longer
necessarily a running one. **DY and the fixed-beam path pass `None` and are
unmoved**: DY carries no `αs` at LO so no source is built at all, and an `lpp = 0`
card never reaches the branch (`setrun.f` overwrites `pdlabel` with `none`, which
`no_pdf_ignores_the_cards_pdf_label` already pins). No production caller supplies a
grid yet — `ProtonIntegrand` is the first, and **it will need the `PdfSet`, not just
the `PdfMember`**: the tabulation lives in `set.info.alpha_s` and `DrellYanIntegrand`
holds only a member.

**The pin, and what it can see.** Two levels, both in `validate_alphas`:

| oracle | measurement |
|---|---|
| the run log's `alpha_s for scale 91.188… is 0.13000271085472237` (17 digits) | reproduced to **9.86e-9** relative |
| the log's `New value of alpha_s from PDF lhapdf` line at `M_Z` | same **9.86e-9** |
| `pp_to_llj_fixed`'s 10 000 `<event>` lines | **10 000 / 10 000** reproduce the printed `AQCDUP` digits exactly, worst **0.281** of the printing budget |

The third is the one with teeth, and it is new: P0 recorded these events as *outside*
the `AQCDUP` oracle because the β-function solve refuses the card. With the grid as
the source they are inside it, so `pp_to_llj_fixed` joins
`SCALUP_IS_THE_RENORMALISATION_SCALE` (23 runs, 230 000 events). The parameter card
is the source it replaces, and the test asserts at each scale that the two are
*further apart than half a printed digit* — `0.1300027` against `0.1300028`, twice
the budget — so a silent revert to the card's value fails the gate on all 10 000
events rather than passing unnoticed.

`GRID_ALPHA_S_TOL = 1e-7`, ten times the measured residual. It is deliberately not
tighter: LHAPDF interpolates these knots with a cubic and this reads them with a
straight line, so the residual is a property of *where the scale sits in its knot
interval*, not of the arithmetic. Tightening the bound would pin the knot spacing.

**What none of this can detect.**

- **Scale dependence.** This run fixes `μR = 91.188`, which *is* `M_Z`, so evaluating
  at `μR` and evaluating at the reference scale return the same number and nothing
  here separates them. A second banked run at another fixed scale would.
- **The interpolation shape.** `91.188` sits `2.4e-5` of the way into
  `[91.1876, 109.8541]`, where linear and cubic readings of the same knots agree to
  `1e-8`; mid-interval they differ by `~1.7e-4`. A dynamical scale needs the real
  `ipol` consumer *and* an LHAPDF-generated reference to gate it — neither exists
  yet, and `GRID_ALPHA_S_TOL` would correctly fail if a dynamical scale were wired
  in against the linear reading.
- **How `αs` enters σ.** There is no llj cross section yet. The power of `αs` in the
  llj integrand is unvalidated until item 4 lands and its σ meets the P0 rebank.

### Gate results

`cargo build` clean, `cargo test --workspace` **556 passed / 0 failed** — 539 at P1's
close, 546 after the floor commit, 556 with the `αs` tests. Every pre-existing test
is unchanged and no tolerance was loosened anywhere.

| gate | result |
|---|---|
| `validate-alphas` | **5 passed** (was 4): new grid-source row; `AQCDUP` now 230 000 events across **23** runs (was 220 000 / 22), worst 0.999 of budget |
| `validate-scales` | 7 passed, unchanged |
| `validate-sigma` | 11 rows asserted, unchanged |
| `validate_hadronic` (DY anchor) | 3 passed — σ default 934.416 ± 0.870 vs MG 933.110 ± 0.447 (rel 0.0014), mmll window 644.855 ± 0.570 vs 644.420 ± 0.315 (rel 0.0007), pointwise oracle worst 1.15e-14 — **the same numbers as before** |
| `validate-lhef` | 3 passed |
| `validate-unweighting` | 1 passed |
| `validate-scale-couplings` | 1 passed |
| `validate-helas-mg` | 18 passed, unchanged |
| `validate-pdf-grid` | **not runnable in this checkout** — `validation/pdf/oracle*.json` are gitignored and absent; the failure is a missing input file, unrelated to any code here. Regenerating needs the LHAPDF C++ oracle build and a second set fetched from the network. |

### What the next session (flavour groups) must know

1. **`Cuts` is now an input to the channel derivation.** The coupling note 24 did not
   plan for exists and is one method call; pass `cuts.spacelike_floor()` to
   `from_diagram_regulated` for the llj channels.
2. **`AlphaSSource` is ready but unwired to any integrand.** `ProtonIntegrand` builds
   its `EventScaleSource` with `Some(&set.info.alpha_s)`, which means it must hold or
   be handed the `PdfSet` — a small signature difference from `DrellYanIntegrand`.
3. **A grid-sourced bank is now classified in two lists.** P0's warning that
   registering an amplitude process banks a run gains a corollary: a
   `pdlabel = lhapdf` run belongs in `GRID_ALPHA_S_RUNS` *and*, once its `AQCDUP`
   reproduces, in `SCALUP_IS_THE_RENORMALISATION_SCALE`. Both are asserted, so
   neither can be forgotten silently.
4. Nothing in this session touched the `MultiChannel` combiner, the enumeration, or
   any amplitude, so P2's measured design facts (24 subprocesses, one ordering per
   unordered initial state, the mirror identity `A(b,a;k) = A(a,b;Rk)`, the group-by-
   measured-|M|² rule) all still stand as written.

---

## P2c outcome (2026-07-30) ✅ — the flavour-group derivation

Branch `proton-events`, one commit `0670068`. Resume item 3 of P2's order: the
`(subprocess-group, flavour-class)` decomposition, in a new
`vibegraph-lib/src/proton.rs`. Items 4–6 (`ProtonIntegrand`, α-adaptation, the
fixed-ŝ validation) are untouched and their design stands.

### The grouping rule, as implemented

`derive_flavor_groups(sets, model, evaluated, card)` takes the enumerated
`DiagramSet`s of one proc card and returns `FlavorGroups`. The partition is
**measured, not listed**: each non-empty subprocess is compiled, then evaluated at
12 shared probe points — flat RAMBO over the outgoing legs, massless beams along
±z, at `3×`, `5×` and `13×` a scale set by the outgoing pole masses (`100 GeV`
floor, so `300 / 500 / 1300` for `llj`). Two subprocesses share a group when their
whole `|M|²` trace agrees to `GROUP_REL_TOL = 1e-10`.

`p p > l+ l- j QCD=2 QED=2` partitions into **6 groups of 4**, exactly as P2
predicted, with **zero** within-group disagreement (bit-for-bit) and a **0.74**
worst-case separation between the closest two groups:

| group | members | initial states | `spin_color_average` |
|---|---|---|---|
| `g u > e+ e- u` | 4 | `g u`, `g c` × `e`, `mu` | 1/96 |
| `g d > e+ e- d` | 4 | `g d`, `g s` × `e`, `mu` | 1/96 |
| `g u~ > e+ e- u~` | 4 | `g u~`, `g c~` × `e`, `mu` | 1/96 |
| `g d~ > e+ e- d~` | 4 | `g d~`, `g s~` × `e`, `mu` | 1/96 |
| `u u~ > e+ e- g` | 4 | `u u~`, `c c~` × `e`, `mu` | 1/36 |
| `d d~ > e+ e- g` | 4 | `d d~`, `s s~` × `e`, `mu` | 1/36 |

The `e`/`mu` multiplicity needs no special case: the two lepton flavours are
*distinct members with the same initial state*, so the luminosity sum over members
supplies the factor of two by itself.

**`|M|²` equality alone is not enough, and the module says so.** It is a sum, so it
is blind to a global phase and would not move if two members' colour bases differed
by a relabelling. A group is therefore refused unless its members also share

- the outgoing pole masses (one phase-space map serves the group),
- an equal compiled `Cuts` (one cut indicator does) — `Cuts` gained `PartialEq`,
  which is equality of the *filter*: every field is a leg index and a threshold,
  the PDG codes having been consumed by `compile`,
- an equal colour basis (`n_flows` + `cf_matrix`), so an event's flow can be read
  off the representative,

and unless two distinct groups separate by more than `GROUP_SEPARATION_MIN = 1e-6`
— a partition produced by two traces landing either side of the tolerance by
rounding is a failed measurement, not a decomposition.

Both extra requirements are pinned against a case where they differ rather than
left as guards that might be vacuous: `a_group_sharing_one_cut_filter_is_a_real_requirement`
shows a `pdg = 5` leg compiles to a *different* filter from a light jet at
`maxjetflavor = 4` and the *same* one at `5`; the colour check is exercised in
`p p > t t~ QED=0`, where the two groups' `(n_flows, cf_matrix)` differ (both have
`n_flows = 2`, so the CF matrix is what separates them — `n_flows` alone would not
have).

### The mirror term, and what its test can and cannot catch

The identity, as implemented in `FlavorGroup::mirror_into`:

```text
|M_{b a}(p₁, p₂, q)|² = |M_{a b}(p₁, p₂, R q)|²,   R: (E, pₓ, p_y, p_z) ↦ (E, pₓ, −p_y, −p_z)
```

**The beams are left where they are and only the outgoing legs are reflected.**
That is the practically important refinement on P2's `A(b,a;k) = A(a,b;Rk)`:
reflecting *everything* would send beam 0 to `−z`, and the pruned evaluator's
partonic-CM contract wants the beams on the axis. Since `R p₁ = p₂`, reflecting the
outgoing legs alone is the same rotation with the beam slots swapped back.
`FlavorGroup::luminosity` returns `[direct, mirror]`, and a member whose beams
carry the *same* parton contributes to `direct` only — one ordering, not two.

`the_mirrored_beam_ordering_needs_the_reflected_matrix_element` enumerates the
mirrored subprocess of every group explicitly (`u~ u > e+ e- g`, `u g > e+ e- u`,
…), compiles it, and compares, over 36 points at three energies:

| what | measured |
|---|---|
| reflected representative vs the explicitly mirrored subprocess | **5.4e-13** worst |
| *dropping* the mirror (evaluating at the unreflected point) | wrong by **≥ 7.4e-2** at **every** point, up to 200× |
| a reflection in the `xz` plane alone (`p_y ↦ −p_y`) | **7.7e-16** — no change at all |

- **What it catches.** A dropped mirror term, at any point — this is the failure
  that would silently halve the `g q` contribution, and the margin never closes.
  A reflection that does not reverse `p_z` (the load-bearing part of `R`), since
  such a map reproduces the *direct* value to 1e-16 and so fails the first row by
  the whole 7.4e-2.
- **What it cannot catch.** The sign of `p_y`. `|M|²` is invariant under the extra
  `xz` reflection, so `R` is pinned only up to it — measured, and asserted as a
  measurement, not assumed. Nothing depends on the difference: both maps are
  correct implementations of the mirror.
- **The 5.4e-13, and why the bound is 1e-11.** It is one point of 36 — the RAMBO
  draw that is `8e-10` off the light cone and `2e-12` off momentum conservation,
  where two independently compiled programs route the gauge-dependent parts
  differently. Every other point agrees to `4.4e-15`. This is a property of the
  *test* — the integrand evaluates one program at both arguments — and the earlier
  cut of the probe, run at a deliberately off-shell point, produced a 73%
  "disagreement" that was entirely this effect. **Any future mirror-type check
  must use an on-shell, conserving point or it will measure gauge dependence.**

### `q ↔ q̄` grouping: checked, not asserted

P1 left this open with the note that `g q` and `g q̄` have the same σ̂ within MC
error. The probe **separates** them: `a_quark_and_its_antiquark_against_a_gluon_do_not_share_a_matrix_element`
measures a pointwise `|M|²` difference of up to **0.93**, on both the up and down
classes, and asserts they land in different groups.

That is the session's clearest illustration of the "every oracle has a blind spot"
rule: the banked partonic σ̂ at `√ŝ = 500` are `0.11812 ± 0.00022 pb` for
`g u > e+ e- u` and `0.11816 ± 0.00026 pb` for `g u~ > e+ e- u~` — indistinguishable
— so **a grouping criterion built on σ̂ would have merged them**, summing the
antiquark's luminosity against the quark's matrix element and the quark's colour
structure. The pointwise criterion is what makes the grouping sound, and P1's
per-diagram rows are what make each side individually trustworthy.

### Symmetry factor: asserted, and the `amps[0]` pattern deliberately not extended

`derive_flavor_groups` **refuses** any subprocess whose
`final_state_symmetry_factor` is not 1, with an error stating why: a matrix element
summed over subprocesses has no single owner for a final-state factor, so each term
would have to carry its own. All 24 `llj` subprocesses pass; the refusal is pinned
non-vacuously by `u u~ > g g` (factor 0.5), which is rejected.

`FixedBeamIntegrand::new`'s `amps[0]` shortcut was **not** extended to the hadronic
sum. The latent `identical-particle-permutation` item in the feature backlog is
untouched and unaggravated: `ProtonIntegrand` will multiply by 1 per group and the
refusal above is what keeps that honest.

### Gate results

`cargo build` clean, `cargo test --workspace` **564 passed / 0 failed** (556 at
P2b's close + 8 new; the pre-existing test inventory is byte-identical, checked by
listing tests before and after: 569 → 576 including ignored). No tolerance was
loosened anywhere; the only tolerances added are new ones with the measurements
above.

| gate | result |
|---|---|
| `validate_hadronic` (DY anchor) | 3 passed — σ default **934.416 ± 0.870** vs MG 933.110 (rel 0.0014), mmll window **644.855 ± 0.570** vs 644.420 (rel 0.0007), pointwise **1.15e-14** — identical to P2b |
| `validate-helas-mg` | 18 passed |
| `validate-amp-diagram` | 7 passed |
| `validate-color-cf` | 30 passed |
| `validate-color-flow-tags` | 30 passed |
| `validate-color-jamp` | 3 passed |
| `validate-diagrams` | 16 passed |
| `validate-alphas` | 5 passed |
| `validate-scales` | 7 passed |
| `validate-sigma` | 11 rows asserted |
| `validate-lhef` | 3 passed |
| `validate-unweighting` | 1 passed |
| `validate-scale-couplings` | 1 passed |
| `validate-pdf-grid` | still not runnable here (`validation/pdf/oracle*.json` gitignored and absent, as P2b recorded) |

### What the `ProtonIntegrand` session must know

1. **The API it consumes.** Per group: `evaluator()`, `diagrams()` (the channel
   derivation's input), `external_legs()`, `cuts()`, `final_masses()`,
   `spin_color_average()`, `members()`, `has_mirror()`, `mirror_into(k, &mut buf)`
   and `luminosity(pdf, x1, x2, mu_f) -> [direct, mirror]` /
   `member_luminosity(i, …)` for P4's per-event flavour draw. The per-point shape is

   ```text
   Σ_groups avg_g · ( L_direct · |M_g(q)|²  +  L_mirror · |M_g(R q)|² ) · Θ_cuts(q)
   ```

   with **one** cut indicator on the unreflected final state, and the symmetry
   factor identically 1.
2. **`cuts()` comes with the group**, compiled from the run card handed to
   `derive_flavor_groups`, so `cuts().spacelike_floor()` is already available where
   the channels are built. P2b's warning still applies: pass the floor because the
   three-body spine wants it, not as a blanket default.
3. **`derive_flavor_groups` refuses a heterogeneous outgoing mass list** across
   subprocesses (`UnequalFinalMasses`), because one phase-space map has to serve the
   whole sum. `llj` and `t t~` are both uniform; a process where they are not needs
   a per-group map and a wider design than this returns.
4. **Cost.** Enumeration of `llj` is 37 ms and compiling all 24 subprocesses is
   7 ms, so deriving the decomposition is not a startup concern — but note it
   compiles **all 24** and keeps only the 6 representatives. Amplitude evaluations
   per integrand point are `2 × 6 = 12` (direct + mirror per group), which is P2's
   estimate confirmed.
5. **`has_mirror()` is worth branching on** for a future `g g`-initiated process:
   its mirror luminosity is identically zero and the second `|M|²` evaluation is
   pure waste.
6. **Plan correction recorded rather than absorbed:** P2 wrote the mirror as
   `A(b,a;k) = A(a,b;Rk)` with `R` applied to the whole point. Applied literally
   that puts beam 0 on `−z`; the implementation reflects the outgoing legs only,
   which is the same rotation composed with the beam-slot swap and keeps the
   partonic-CM contract the pruned evaluator asserts.

---

## P2d outcome (2026-07-30) ✅ — the integrand

Branch `proton-events`, three commits (`f9fe0fc`, `8840e63`, `613af41`). This is the
last slice of P2: resume items 4, 5 and 6 — `ProtonIntegrand`, the joint
α-adaptation, and the in-session validation. P2's design stands; the corrections
below are recorded rather than absorbed.

### `ProtonIntegrand`, as built

In `vibegraph-lib/src/proton.rs`, alongside the decomposition it consumes. It
implements `ChannelIntegrand`, so `UnweightPass::scan`, the frozen-grid machinery and
`generate`'s accept/reject need no hadronic special case.

```text
σ = ∫ dτ dy dΦ_n  Σ_g avg_g · [ L_g^direct |M_g(q)|² + L_g^mirror |M_g(Rq)|² ] · Θ_cuts(q) / (2ŝ)
```

- **Outer map**: Drell–Yan's exactly — `τ = τ_min^(1−u₀)`, `y` flat over `|y| ≤ ½ln(1/τ)`,
  Jacobian `ln(1/τ_min)·2y_max` with the `1/x₁x₂` already cancelled against `dτ`.
  `τ_min = Cuts::shat_min()/s`.
- **Inner map**: one `ScaledMultiChannel` over **the pooled channels of every group** —
  `p p > l+ l- j` gives 24 = 6 × 4 — sampled at the event's own `√ŝ = √(τs)`.
  `channel_grid_ndim = 2 + 5 = 7`; the undivided mixture form (`value`) adds a channel-
  selection coordinate, `vegas_ndim = 8`.
- **Frames**: `|M|²` in the partonic CM with the beams on ±z (the pruned evaluator's
  contract, and the frame the channels generate in); the cut filter and the scale
  prescription in the lab frame, the outgoing legs boosted along z by `y`. Split
  exactly as DY's.
- **Floor**: `cuts().spacelike_floor()` = 400 GeV² for the llj card. **16 of the 24
  channels carry a peripheral spine at that floor and 0 of 24 do at floor zero** —
  measured in `every_diagram_of_every_group_becomes_a_floored_channel`, which is what
  makes "passed because the three-body spine wants it" a fact rather than a caption.
- **Scales**: `use_run_card_scales(model, evaluated, card, Option<&AlphaSInfo>)`, taking
  the set's `αs` tabulation (P2b's `AlphaSSource`). A non-constant prescription is
  resolved once at setup on a cut-passing draw, so a refusal surfaces there.
- Symmetry factor identically 1, guaranteed by `derive_flavor_groups`' refusal.

### Joint α-adaptation

`adapt_alphas(seed, n_survey, n_iter, damping)` runs `MultiChannel::adapt_alphas`'
survey→refine loop **from outside the combiner**, because the integrand owns the
`(τ, y)` coordinates and so owns the energy each draw is made at. `kleiss_pittau_step`
is shared; the surveyed `f` is the whole hadronic shape (Jacobian, flux, cut,
luminosity-weighted group sum), so weight flows to the channels carrying variance *of
the hadronic integral*.

Measured on llj: the weights spread by **81.9×** over 6 surveys of 4000 points, and the
mixture estimator's standard error at 30 000 draws falls to **0.77×** the uniform
mixture's, with the integral unmoved (0.7 combined standard errors).

**Plan correction — the adaptation must be measured on the mixture, not on
`adapt_grids`.** The first cut of that test compared uniform against adapted through
`adapt_grids` and the adapted run came out *noisier* (1.8e-10 against 1.6e-10). The
comparison is not budget-matched: `channel_neval(αⱼ, neval)` allocates `αⱼ·neval` under
a `MIN_CHANNEL_NEVAL = 512` floor, so a uniform 24-channel run at `neval = 4000` gives
every channel the floor while an adapted one gives the leading channels more and the
rest the same floor — different sample counts and different conditional densities per
grid. The estimator the weights actually minimise the variance of is the mixture one,
and on it the improvement is unambiguous.

### In-session validation — what it proves, and what it cannot see

| check | measurement |
|---|---|
| pointwise, against an independently assembled point | worst **8.12e-14** over 400 draws (85 inside the cuts, 315 cut-rejected) |
| fixed-`ŝ` slice vs `FixedBeamIntegrand`, `√ŝ = 200` | **0.31%**, pull 0.75 |
| fixed-`ŝ` slice vs `FixedBeamIntegrand`, `√ŝ = 500` | **0.63%**, pull 1.32 |
| Drell–Yan through the general path vs `DrellYanIntegrand` (synthetic PDF) | **0.28%**, pull −0.72 |
| Drell–Yan through the general path vs MadGraph (real PDF, real card) | **933.706 ± 1.843** vs **933.110 ± 0.447**, 0.3 combined σ, rel **0.0006** |

**The pointwise oracle** re-derives the `(τ, y)` map, both frames, the flux and the `2π`
measure, forms each member's `x·f` product directly from the parton distribution, and
takes the mirrored ordering from an **explicitly enumerated `b a > …` subprocess
evaluated at the unreflected point**. It therefore shares neither
`FlavorGroup::luminosity` nor `FlavorGroup::mirror_into` with the integrand: a dropped
mirror, a mirror at the wrong argument, a swapped beam ordering or a lost spin/colour
average all move it, and the mirror is measured to carry up to the whole of a group's
term at these points. *It cannot see* the phase-space weight (taken from its own copy
of the same channel construction) or an error the cut filter and the parton
distribution both share.

**The fixed-ŝ slice** freezes `(τ, y)` at `y = 0`, where the lab frame coincides with
the partonic CM and one cut filter therefore applies to both sides, and compares the
integrand's inner integral against `Σ_g (L^direct + L^mirror)_g · σ̂_g(ŝ)` with `σ̂_g`
from `FixedBeamIntegrand` through its **own** all-timelike per-diagram map. So the
flux, the `2π` measure, the spin/colour average and the symmetry factor are compared
across two independent phase-space maps, at two energies so the `√ŝ` dependence is a
shape and not one normalisation. *It cannot see* the rapidity boost (switched off) or
the mirror's *argument*: at `y = 0` the two orderings carry equal luminosity and
`∫|M(Rq)|²Θ(q)dΦ = ∫|M(q)|²Θ(q)dΦ` because `R` preserves the measure and every
observable the filter cuts on, so a mirror evaluated at the wrong point would still
integrate correctly here. The pointwise oracle is what pins the argument.

**The Drell–Yan row** is the "keep a known-wrong informational comparison running"
convention discharged: it is the one process the general path can already do end to end
against MadGraph, and it is where the two treatments of the mirrored ordering meet —
DY sums both luminosities against a *single* `|M(q)|²`, which is only right once the
angles are integrated, while the general path evaluates `|M(Rq)|²` pointwise. *It cannot
see* anything specific to a coloured initial state or a three-body final state: no
gluon-initiated group, no peripheral channel, no strong coupling, no jet cut.

**Tolerances.** The slice bounds (`rel < 3%`, `|pull| < 4`) sit above a measured
four-seed sweep at `√ŝ = 500` — rel `{1.31%, 0.16%, 0.05%, 0.39%}`, pulls
`{2.73, −0.33, 0.11, 0.82}` — and far below anything a lost factor produces; the
smallest normalisation error the test exists to catch is a factor of two. Raising the
partonic budget five-fold brings those same seeds to **0.04%–0.46%**, so the residual is
where the two Monte Carlos have converged to, not a disagreement between them. The
informational DY row asserts only `rel < 2%`. No existing tolerance was touched.

**Measured while building the oracle, worth keeping:** flat RAMBO is *not* a usable
partonic reference for llj at `√ŝ = 500`. At 40 000 points per group it undershoots the
multichannel result by **8%–18%** on every seed — it misses the `Z` pole inside the
`mmll = 50` window. The partonic side of the slice therefore uses
`use_multichannel` + VEGAS, and its own convergence was checked by raising the budget.

### Plan corrections

1. **`ln(1/τ_min) = 11.12`, not 18.9, and the waste is 7.5%, not 4%.** The hint
   (`ŝ_min = 2500`) against the true threshold (`(√(mmll²+ptj²)+ptj)² = 5454.1`) is loose
   by **2.18×** — P2's ratio was right — but that is `0.78` out of `11.12`, so **7.5% of
   `τ` draws land below threshold**, measured over 4000 draws, every one of them
   returning exactly `0.0`. Confirmed acceptable and left alone; tightening it would
   have to leave DY's number bit-identical.
2. **One cut filter across *groups* is a new requirement, and it is now enforced.**
   `derive_flavor_groups` checks the filter only *within* a group; the per-point shape
   needs a single `Θ`, so `ProtonIntegrand::new` refuses `GroupCutsDiffer`. A process
   whose groups are cut differently needs one indicator per group and a wider design.
3. **`has_mirror()` needs no branch in the integrand.** `FlavorGroup::luminosity`
   already returns a zero mirror for a same-parton initial state without touching the
   parton distribution, so `reflected != 0.0` *is* the skip P2c asked for; a `gg` group
   costs one matrix element per point, not two.
4. **The bound-amplitude ↔ group pairing needed a guard P2c's API did not specify.**
   `ProtonIntegrand::new` checks `std::ptr::eq(amp.evaluator(), group.evaluator())` per
   position. Crossing the pairing weights one group's matrix element with another's
   luminosity — a smooth shift of σ with no other symptom — and is pinned by a test that
   swaps two amplitudes.
5. **The α-adaptation measurement**, as above.

### Gate results

`cargo build` clean. `cargo test --workspace` **519 lib passed / 0 failed** (511 at
P2c's close + 8 new) plus the CLI suites; `validate_hadronic` **4 passed** (3 + the new
informational row). No tolerance was loosened anywhere.

| gate | result |
|---|---|
| `validate_hadronic` (DY anchor) | 4 passed — σ default **934.416 ± 0.870** vs MG 933.110, mmll window **644.855 ± 0.570** vs 644.420, pointwise **1.15e-14** — **identical to P2b and P2c**; new informational general-path row 933.706 ± 1.843 |
| `validate-helas-mg` | 18 passed |
| `validate-amp-diagram` | 7 passed |
| `validate-color-cf` | 30 passed |
| `validate-color-flow-tags` | 30 passed |
| `validate-color-jamp` | 3 passed |
| `validate-diagrams` | 16 passed |
| `validate-alphas` | 5 passed |
| `validate-scales` | 7 passed |
| `validate-sigma` | 11 rows asserted |
| `validate-lhef` | 3 passed |
| `validate-unweighting` | 1 passed |
| `validate-scale-couplings` | 1 passed |
| `validate-pdf-grid` | still not runnable here (`validation/pdf/oracle*.json` gitignored and absent, as P2b and P2c recorded) |

### What P3 must know

1. **The API.** `ProtonIntegrand::{new, use_run_card_scales, adapt_alphas,
   set_channel_alphas, channel_alphas, channel_ids, channel_count, channel_grid_ndim,
   value_in_channel, value, vegas_ndim, adapt_grids, integrate, tau_min,
   spacelike_floor, alpha_s_source}`, plus `impl ChannelIntegrand`. `adapt_grids`
   returns `Vec<ChannelIntegration>` and a combined `VegasResult`, the same shapes the
   fixed-beam path banks.
2. **Artifact keying.** A channel is a `ChannelId { group, diagram }`, not a diagram
   index — 24 of them for llj. The per-channel grid is over **7** coordinates where the
   fixed-beam path's is over `3n−4`, so the schema has to carry the dimension and the
   ids, and an `fv3` artifact cannot be read as a hadronic one by shape alone.
3. **Replay exactness.** Re-installing banked weights must go through
   `set_channel_alphas`; re-running `adapt_alphas` reproduces them only by accident, and
   `αⱼ` enters every channel's weight.
4. **Budget floor.** `MIN_CHANNEL_NEVAL = 512` × 24 channels = **12 288 evaluations per
   iteration** minimum, whatever `neval` says. Each point costs up to 12 amplitude
   evaluations (6 direct + 6 mirror) and 24 channel densities.
5. **Sweep the seeds.** Both the slice and the α tests show single-seed pulls up to
   2.7 on quantities that agree to a few per mille; a single-seed σ gate is not a pass.
6. **`ProtonIntegrand` needs the `PdfSet`**, not just the member — `set.info.alpha_s` is
   what `use_run_card_scales` wants (P2b's warning, now a signature).
7. **The dynamical llj card is refused at setup**, by the probe, with the clustering
   message; pinned in `a_dynamical_scale_is_refused_where_the_fixed_one_resolves`. The
   same test pins that the fixed card resolves to constants *and* that the coupling
   installed is the grid's (0.1300028) and not the parameter card's.
8. **Deduplication is still open.** Channels are heavily duplicated across groups (the
   up- and down-type groups build identical maps), so 24 channels carry fewer than 24
   distinct densities. Harmless — α just splits — but it is the first lever if P3's gate
   runs long.

### What P4 must know

The event side is **not** built. `ProtonIntegrand` has no counterpart to
`FixedBeamIntegrand::{event_in_channel, select_event}`: reconstructing an accepted
point's momenta, its `(x₁, x₂)`, its group, its concrete member (`member_luminosity` is
the draw P2c left ready), its helicity and its colour flow are all P4's. The lab-frame
momenta the record needs are already built per point inside `shape`; exposing them is
the natural first step.

## P3 outcome (2026-07-30) ✅ — the σ gate at `lpp = 1`

Branch `proton-events`, two commits: `d35fc98` (routing + artifact schema) and
`02e009a` (the gate). P2/P3's design stands; the corrections below are recorded
rather than absorbed.

### The routing, and a latent bug it uncovered

`integrate_proton` now dispatches. The general path builds the flavour-group
decomposition from **the proc card's own enumeration**, binds one amplitude per
group, adapts the channel weights on the hadronic mixture, and banks one grid per
`(group, diagram)` channel.

**The bug the dispatch replaced.** The old `lpp = 1` branch called
`generate_dy_subprocesses(model)`, which parses `generate p p > e+ e-` itself and
**ignores the caller's proc card entirely** — the `_parsed` argument was unused. So
`vibegraph integrate` on the llj cards would have integrated Drell–Yan and printed
`p p > l+ l- j` over it, with no error anywhere. Nothing detected this before P3
because nothing but Drell–Yan had ever been run at `lpp = 1`.

Drell–Yan is dispatched by an **exact match with no modifiers**, not by "the general
path failed": one process spec, no coupling constraint, no required or forbidden
s-channel, no forbidden propagator, and the printed process `p p > e+ e-`. The
modifier check is not decoration — `Display for ProcessSpec` drops modifiers, so
`p p > e+ e- QED=2 QCD=0 / a` and `p p > z > e+ e-` both *print* as the Drell–Yan
string over a different diagram set, and the bespoke integrand would have silently
ignored the difference. Pinned in
`only_an_unmodified_drell_yan_card_takes_the_bespoke_path`, in both directions.

### Artifact schema: `fv3 → fv4`, with the `fv3` reader kept

P2d's warning holds and is now the schema's stated reason: **a grid's coordinate
count does not identify its channel space.** `ChannelGrid` gains

```rust
pub enum ChannelKey {
    Whole,                                        // one grid over the whole map
    Diagram { diagram: usize },                   // fixed-energy per-diagram multichannel
    GroupDiagram { group: usize, diagram: usize },// hadronic, pooled across groups
}
```

and `FORMAT_VERSION` goes to `4`. The *dimension* needed no new field —
`VegasGrid::ndim()` already carries it — so what the bump adds is the identity, which
was genuinely underivable.

`read_from_path` dispatches on the version prefix: `4` decodes directly, `3` decodes
through `artifact::v3` and upgrades, anything else is refused by name with the
readable range in the message. The upgrade is not a guess: only two writers could
produce an `fv3` file — the Drell–Yan integrand's single grid over the whole map, and
the fixed-energy multichannel's grids in diagram order — so a lone channel becomes
`Whole` and several become `Diagram { j }`, and **no `fv3` channel can upgrade to a
hadronic key**. Model-identity fields carry over unchanged.

The version-refusal test had to be rewritten rather than left alone: it wrote
`FORMAT_VERSION - 1`, which is now a *supported* version, so it would have quietly
become a test of the upgrade path. It now writes version 2's own shape (no `model`
field, so every field after `process` sits one slot early — the exact payload a
silent positional misread would consume) and asserts the refusal, with the fv3
upgrade covered by its own test.

### THE GATE — σ(pp → ℓ⁺ℓ⁻ j), fixed scale

`validate_hadronic::sigma_llj_fixed_scale_vs_mg`. Reference: the banked
`pp_to_llj_fixed` run, **σ = 422.840 ± 1.805 pb**, read out of that run's own
`SubProcesses/results.dat` rather than copied into a committed JSON — the number
cannot then drift from the run it came from. The run card is read from the same
run's `Cards/run_card.dat`, so MadGraph and this crate consume the identical file.

Five seeds, at `neval = 300 000 × 10` iterations after a `8 000 × 5` α-survey:

| seed | σ (pb) | rel | pull |
|---|---|---|---|
| 20260730 | 423.059 ± 0.439 | +0.0005 | +0.12 |
| 20260731 | 423.558 ± 0.424 | +0.0017 | +0.39 |
| 20260732 | 422.526 ± 0.425 | −0.0007 | −0.17 |
| 20260733 | 423.081 ± 0.411 | +0.0006 | +0.13 |
| 20260734 | 422.051 ± 0.417 | −0.0019 | −0.43 |
| **inverse-variance mean** | **422.850 ± 0.189** | **+0.00002** | **+0.01** |

χ²/dof of the five about their mean: **1.90**. Wall clock **117 s** for the sweep,
**118 s** for the whole `validate_hadronic` suite.

### The five-seed sweep was necessary and *not sufficient* — the budget scan is the finding

This is the sharpest thing P3 measured, and it is a correction to how the sweep was
briefed.

The first budget tried was `neval = 60 000`, and it **passed a single-seed check, a
five-seed check, and the seed-scatter check** while being **1.0% low**:

| `neval`/iter | 5-seed mean (pb) | rel vs MG | χ²/dof | wall |
|---|---|---|---|---|
| 60 000 | 418.476 ± 0.438 | **−1.03%** | 1.55 | 27 s |
| 150 000 | 421.658 ± 0.270 | −0.28% | 0.47 | 61 s |
| 300 000 | 422.850 ± 0.189 | +0.002% | 1.90 | 117 s |
| 600 000 | 423.524 ± 0.133 | +0.16% | 0.37 | 229 s |

At 60 000 the five seeds were **mutually consistent** (pulls −2.36, −2.11, −2.67,
−2.33, −1.08; χ²/dof 1.55 about their own mean) and **collectively wrong**. A sweep
detects a seed that missed a region; it cannot detect a bias every seed shares,
because the seeds do not disagree about it. Seed spread and budget convergence are
two axes, and only the second one moved this number.

**Where the bias comes from.** `VegasGrid::adapt` puts *every* iteration into
`combine_iterations`' `1/σ²` weighted mean, including the first ones on an unadapted
grid. An early iteration that undersamples the peak returns both a low integral and
a low variance, so it is weighted *up*. The steps halve as the budget doubles
(−3.18, −1.19, −0.67 pb), the signature of an `O(1/N)` bias, so the limit is around
`424` pb — inside MadGraph's own `±1.805` (0.43%) at every budget from 150 000 up.

This is a property of the integrator, not of the hadronic integrand: the same
combination runs the fixed-beam path. It is visible here first because the llj
integrand has 24 pooled channels each carrying a 7-dimensional grid, so a given
`neval` buys each channel far fewer points than a partonic 2 → 2 run does — and
`MIN_CHANNEL_NEVAL = 512` is not a fix, it is the floor that makes the small
channels' grids the noisiest part of the sum. **Worth filing**: discard-first-`k`
iterations (or an unweighted final pass over the trained grids) would remove it, and
would let this gate run at a quarter of the budget.

**The bound, and why it is where it is.** `LLJ_MAX_REL = 0.005`, above MadGraph's own
0.43% error (no agreement tighter than the reference's precision is meaningful) and
below the 1.0% an under-converged budget produces (which is what it is there to
catch). The whole measured budget family — 0.28%, 0.002%, 0.16% — sits inside it, so
the bound brackets a family and not one number. The scatter bound `χ²/dof < 4` sits
above all four measured values (1.55, 0.47, 1.90, 0.37). **No existing tolerance was
touched.** The 300 000 budget was chosen for cost; the near-perfect 0.002% at exactly
that budget is where the rising estimator happens to cross MadGraph's number and
should not be read as more than the 0.2% the family supports.

### What the gate proves, and what it cannot see

**Proves**, on real parton distributions, a real run card and MadGraph's own number
for the same cards: the whole hadronic chain for a process with a **coloured initial
state, a three-body final state, a jet cut and a strong coupling** — the four things
every Drell–Yan row is blind to. Specifically: that the six flavour groups and their
24 members sum to MadGraph's own 24 subprocesses (`auto_dsig.f` lists exactly the
same 16 + 8), that both beam orderings are counted once each, that the spacelike
floor's reshaped spine leaves the estimator unbiased, that `αs` off the PDF grid at
`μR = m_Z` is the coupling MadGraph used, and that the `(τ, y)` map's `τ_min` hint
does not clip real phase space.

**Cannot see**, and this is the whole of the σ level's blindness:

- Anything σ integrates over. A per-diagram phase, a colour-flow relabelling, a
  helicity-by-helicity error — all leave `Σ|M|²` and hence σ exactly alone. Those are
  pinned at the amplitude level by P1's `validate_helas_mg` and `amp_diagram_oracle`
  rows for `uux_to_epemg`, `gu_to_epemu`, `ddx_to_epemg`.
- A map whose weight and density are both wrong by one common factor: the phase-space
  map is compared against nothing here, only used.
- Any *distribution*. Agreement of one number is agreement of one number; the shapes
  are the next validation pass's content.
- Whether `mmll = 50` hides anything. The standing `low-mll-reconciliation` entry says
  the Drell–Yan comparison carries a few-percent discrepancy below the `Z` window, and
  this run was banked at `mmll = 50` precisely so as not to inherit it. **The optional
  `mmll = 0` informational row was not built** — it needs its own MadGraph rebank,
  which is a P0-shaped session, not a line in this one.

### Plan corrections

1. **The five-seed sweep is a floor, not a gate.** As above: seeds agreeing with each
   other is not seeds agreeing with the truth. Any future σ gate on a
   many-channel/high-dimension integrand needs a **budget scan** alongside the seed
   sweep, and the budget it settles on should be recorded with the scan that chose it.
   `LLJ_NEVAL`'s doc comment carries the scan for exactly that reason.
2. **The `lpp = 1` branch ignored its proc card.** Recorded above; it was a real
   latent bug, not a design choice, and the dispatch is what closes it.
3. **`validate_hadronic` needed a pixi task and an optimised profile.** It had neither
   — its module doc named a bare `cargo test`, which for the llj sweep means *hours*
   unoptimised against 2 minutes under `--profile profiling`.
   `pixi run -e madgraph validate-hadronic` now exists, depending on `fetch-pdf`,
   `generate-hadronic-sigma` and `build-diagrams`.
4. **The banked llj σ is read from the run, not banked into JSON.**
   `hadronic_sigma_reference.json` is regenerated wholesale by
   `gen_hadronic_sigma.sh`, which does not know about llj; an llj entry added to it
   would be silently dropped on the next Drell–Yan rebank. `results.dat` is the same
   pair of fields that script itself parses, so nothing is lost by reading it directly
   and the number cannot desynchronise from its run.
5. **P2d's operational facts all held.** `set_channel_alphas` was not needed (this
   session surveys, it does not replay), and every other item —`PdfSet` not
   `PdfMember`, the 12 288-evaluation floor, `αs` from the grid, the 7.5% of `τ` draws
   returning zero — behaved exactly as recorded. The dynamical llj card's refusal
   naming `kt-clustering` stays asserted in
   `a_dynamical_scale_is_refused_where_the_fixed_one_resolves`.

### Gate results

`cargo build` clean. `cargo test --workspace` **520 lib passed / 0 failed** (519 at
P2d's close + the `fv3` upgrade test; the version-refusal test was rewritten, not
added) plus the CLI suites — the binary's own **9** (8 + the dispatch test),
`cli_fixed_energy` 2, `cli_generate` 3.

| gate | result |
|---|---|
| `validate_hadronic` | **5 passed** — llj **422.850 ± 0.189** vs MG **422.840 ± 1.805** (0.01 combined σ, rel 2e-5) |
| ↳ Drell–Yan anchor | σ default **934.416 ± 0.870**, mmll window **644.855 ± 0.570**, pointwise **1.15e-14** — **identical to P2b, P2c and P2d** |
| ↳ DY general-path info row | 933.706 ± 1.843, rel 0.0006 — unmoved |
| `cli_integrate` (extended) | 3 passed — CLI σ **934.415866 ± 0.869944** and **644.855362 ± 0.569603**, bit-for-bit as banked |
| `validate-helas-mg` | 18 passed |
| `validate-amp-diagram` | 7 passed |
| `validate-color-cf` | 30 passed |
| `validate-color-flow-tags` | 30 passed |
| `validate-color-jamp` | 3 passed |
| `validate-diagrams` | 16 passed |
| `validate-alphas` | 5 passed |
| `validate-scales` | 7 passed |
| `validate-sigma` | 11 rows asserted, unchanged |
| `validate-lhef` | 3 passed |
| `validate-unweighting` | 1 passed |
| `validate-scale-couplings` | 1 passed |
| `validate-pdf-grid` | still not runnable here (`validation/pdf/oracle*.json` gitignored and absent, as P2b–P2d recorded) |

### What P4 must know

1. **The artifact is ready and `generate` is not.** An `fv4` hadronic artifact carries
   `ChannelKey::GroupDiagram { group, diagram }` per grid, in the integrand's channel
   order — which is `channel_ids()`' order, group-major. `generate` must rebuild the
   decomposition from the same proc card and check that the derived `channel_ids()`
   match the banked keys **pairwise**, not just in count: two orderings with the same
   multiset of keys would load the wrong grid onto each channel with no other symptom.
2. **Replay must go through `set_channel_alphas`**, still. P3 did not exercise it (it
   surveys); P4 is the first consumer and the first place a re-survey would silently
   produce different weights.
3. **The event side is still unbuilt**, exactly as P2d left it —
   `event_in_channel` / `select_event` have no `ProtonIntegrand` counterpart, and the
   lab-frame momenta `shape` already builds per point are the natural first thing to
   expose. The concrete-flavour draw (`FlavorGroup::member_luminosity`) is ready and
   unused.
4. **`generate` refuses `lpp != 0` by name** (`generate.rs`, `rc.beam_mode() !=
   BeamMode::FixedEnergy`); that refusal is what P4 replaces.
5. **The convergence bias reaches P4 too.** `generate`'s accept/reject runs over the
   *frozen* banked grids, so it inherits whatever those grids were trained to — but
   `w_max` scanning and the unweighting efficiency will both be set by the same
   under-sampled small channels. Budget the banked integration accordingly, and do not
   assume a `neval` that gave a good σ gives a good `w_max`.
6. **`ICOLUP` for coloured incoming legs is genuinely new writer output** and the
   `leshouche.inc` of `P1_qq_llg` and `P1_gq_llq` in the banked run is the oracle,
   per the P4 plan. Nothing in P3 touched colour.

## P4 outcome (2026-07-31) ✅ — `generate` at `lpp = 1`

Branch `proton-events`. `vibegraph generate` now covers proton beams for the path
P2/P3 built, and the sprint's exit criterion — cards to `.lhe` for `ℓℓj` — is
reached. P4's design stands; the corrections below are recorded rather than
absorbed.

### What was built

**Library.** `ProtonIntegrand` gains the event side `FixedBeamIntegrand` already
had. `event_in_channel(channel, u)` returns the accepted point in both frames
together with its `(x₁, x₂)` and the scales the matrix element ran at;
`select_event(event, [u₀…u₃])` draws the labels a record needs. Two supporting
pieces on the record layer: `ColorFlowTags::permuted` and
`SubprocessRecord::relabelled`, which is how one compiled amplitude serves 24
concrete subprocesses under two beam orderings.

**CLI.** `generate` dispatches on beam mode. The hadronic path rebuilds the
flavour decomposition from the proc card, installs the banked selection weights
through `set_channel_alphas` (P4 is that method's first consumer, as P3 warned),
scans the frozen grids and emits. `--pdf-set` / `--pdf-dir` are new flags, and
the PDF set is compared against the artifact's before anything is loaded.

### The flavour draw, and the test that pins it

Within a drawn group the concrete flavour and the beam ordering are one
categorical draw over `(member, ordering)` with weight
`L_i^o(x₁, x₂) · |M(q or Rq)|²`. The members of a group share their matrix element
*exactly* — that is what makes them a group — so at a fixed ordering the whole of
what separates them is their parton luminosity at the event's own `(x₁, x₂)`, and
the rule reduces to "∝ luminosity share". The ordering split is the one place the
matrix element re-enters, because `|M(q)|²` and `|M(Rq)|²` differ.

`a_members_share_of_the_draw_is_its_share_of_the_parton_luminosity` pins it by
**sweeping** the uniform rather than sampling it, so the measured shares are the
rule's own and carry no Monte Carlo error — the only residual is the sweep's grid,
and every margin is quoted in units of that resolution:

| | measured |
|---|---|
| deviation from the luminosity share | **0.18** resolutions |
| what a *uniform* draw would be off by | ≥ **6.3** |
| what an *exchanged-beam* assignment would be off by | up to **72.3** |

The oracle forms each member's `x·f` product straight from the parton
distribution, so it shares no code with `FlavorGroup::member_luminosity`. Two
things had to be arranged for the margins to exist at all, and both are asserted
rather than assumed: the point is chosen off central rapidity (`x₁ > 5x₂`), and
the probe distribution's `x` shape differs sharply per flavour — `probe_pdf` is
flat enough in `x` that exchanging `x₁` and `x₂` moves the shares by only 9e-4,
which would have left the beam orientation unpinned. The group's entry point is
found by bisection on `u₀` (the group index rises with it, being read off a
cumulative distribution), so even the smallest group is reached.

*What it cannot see*: realised frequencies over a generated sample against
MadGraph's own. That is a distribution-level question and stays with the deferred
validation pass.

### The exchanged beam ordering, and MadGraph's own answer

The claim is that the mirror identity extends from `|M|²` to the accumulators a
record is filled from: `R` maps each beam momentum onto the other's, so the
representative's leg 0 describes the event's *second* beam, and the two incoming
legs of every per-leg field trade places while the outgoing legs are untouched.
That is a convention claim about helicity labels and colour lines, which no cross
section can see.

`an_exchanged_ordering_relabels_the_beams_of_every_per_leg_field` measures it
against the mirrored subprocess compiled from **its own** proc card: per-helicity
`|M_c|²` and per-flow `JAMP2` agree to **1e-11** of the summed `|M|²` over all six
`ℓℓj` groups, matched by helicity *tuple* under the leg permutation rather than by
index. *What it cannot see*: `ℓℓj` has one colour flow per subprocess, so the flow
*index* is unpermutable here and only the tags of that one flow are compared.

MadGraph's banked events say the same thing independently, and this was checked by
eye before the test was written. `P1_gq_llq`'s `leshouche.inc` gives
`g u > e⁺e⁻ u` as `ICOLUP = (501,502), (502,0), 0, 0, (501,0)`; a banked event with
the quark on beam 1 carries `(502,0), (501,502), 0, 0, (501,0)` — the two incoming
rows exchanged and the outgoing row untouched, with the same integers.

### `w_max` against budget — the measurement P3 asked for

P3's warning was right and the effect is larger here than for any gated process. A
frozen scan estimates each channel's maximum on that channel's own share of the
integration budget, so `w_max` inherits the same under-sampled small channels that
biased σ. Measured on the banked llj cards, 20 000 events per row:

| `neval` | banked σ (pb) | overweight rate | **share of σ above `w_max`** | largest `w/w_max` | efficiency |
|---|---|---|---|---|---|
| 30 000 | 416.876 ± 1.632 | 1.48e-4 | **3.24e-2** | 23.5 | 9.62e-3 |
| 100 000 | 419.893 ± 0.831 | 7.87e-5 | **1.50e-2** | 9.4 | 1.03e-2 |
| 300 000 | 423.731 ± 0.473 | 4.75e-5 | **8.43e-3** | 15.0 | 1.17e-2 |
| 600 000 | 423.670 ± 0.325 | 2.00e-5 | **5.28e-3** | 11.1 | 8.18e-3 |

For scale: note 23's gated `e⁺e⁻ → μ⁺μ⁻` shows an overweight share of **3.04e-4**
and a largest ratio of **1.009**. The llj run is 20–100× worse and the share is
still falling at 600 000, so the scan is nowhere near converged at any budget this
gate can afford. The estimator stays unbiased — an overweight point is kept at a
weight above one — so what the tail costs is the sample's *spread*, and under
`IDWTUP = +3` it would cost lumpiness (an event of weight 25 becomes 25 copies).
The largest ratio moves non-monotonically (23.5, 9.4, 15.0, 11.1) exactly because
it is an extremum estimate: one channel finding a new maximum moves `Σⱼ w_maxⱼ` and
the efficiency with it, which is why the efficiency column is not monotone either.

**A `neval` that gives a good σ does not give a good `w_max`** — confirmed, and the
two do not even improve in step.

### The sample's own σ does not inherit the integrator's bias

A five-seed sweep of the emitted file's mean `XWGTUP` against the banked σ, 20 000
events each:

| `neval` | deviations | mean |
|---|---|---|
| 100 000 | −0.19, **+2.29, +1.67, +1.37, +1.11** % | **+1.25%** |
| 300 000 | −0.36, −0.14, −0.62, +0.47, +0.29 % | **−0.07%** |

Four of five on the same side at the low budget is not a fluctuation. The reason is
worth recording, because it is an independent confirmation of P3's diagnosis: the
accept/reject pass is a **single** pass over the frozen grids, so its estimator does
*not* go through `combine_iterations`' `1/σ²` weighting of iterations, and it
therefore converges to the true σ while the banked number is still about 1% low.
The sample and the integration disagree at 100 000 by roughly the integrator's own
bias, and agree at 300 000 where that bias has gone. **The gate runs at 300 000 for
this reason, not for the wall clock**, and `SIGMA_MAX_REL = 0.015` brackets the
converged spread while staying below what an unconverged budget produces.

### The four gates

**(a) The banked fixed-scale `.lhe.gz` is in the byte-for-byte corpus.** It needed
no work: `banked_files_round_trip_byte_for_byte` *discovers* every banked run, so
`pp_to_llj_fixed` joined when P0 banked it — 10 000 events / 59 493 legs,
byte-identical, inside a corpus that is now 25 runs and 248 747 events. What it
needed was a **guard**: a discovered corpus can silently stop covering something.
The test now requires at least one run with a hadron-collider `<init>` (proton beam
ids and an LHAPDF id in `PDFSUP`) and at least one with colour lines on an incoming
leg — the two layouts this crate's writer emits at proton beams and did not emit
before — and names which run supplies each.

**(b) `generate`'s own output self-reads.** New gate
`validate-generate-proton` (`cli_generate_proton`, pixi task added): one `integrate`
run on MadGraph's own banked run card, 20 000 events, read back through
`lhef::parse`. Per event: `NUP`, statuses, mothers, four-momentum balance
(≤ **7.02e-11** of the event's own incoming energy), on-shellness (≤ **5.11e-9** of
its own `ŝ`), the beam partons on their own side of the axis carrying at most their
beam's energy, `SCALUP` exactly `91.188`, and `AQCDUP` within **1.09e-7** of the PDF
grid's coupling. `<init>` carries `2212 2212`, the run card's beam energies, and the
LHAPDF id in `PDFSUP` — the same fields MadGraph's own file carries, compared
against it.

Two of the checks are real MadGraph oracles rather than self-comparisons, and they
are the ones the hadronic path introduced:

- **Flavour.** Every emitted `IDUP` row must be one of the 24 subprocesses
  MadGraph's `leshouche.inc` lists, or that subprocess with its two beams
  exchanged. The oracle is read from the generated Fortran, not from MadGraph's
  event sample, so a flavour its 10 000 events happen to miss is still admissible
  and one it could never produce is still refused. Observed: **48 distinct
  assignments** — all 24 subprocesses in both orderings.
- **Colour.** Every event's connectivity must be one MadGraph's own events exhibit
  for the same arrangement of gluon / quark / antiquark / leptons. Observed: all 6
  initial-state arrangements, matching MadGraph's set exactly.

σ(sample) **423.158 ± 0.209 pb** against the integration's **423.731 ± 0.473 pb**
(−0.135%), mean `XWGTUP` equal to the declared `XSECUP` to < 1e-6.

*What (b) cannot see* — note 23's E4 caveat applies unchanged and is the reason (a)
exists: the file is read back by **our own parser**, which shares its assumptions
with our writer, so a self-consistently wrong format is invisible here. It is also
blind, as every replay is, to anything the integrand gets wrong. And neither
MadGraph oracle above says anything about *how often* each flavour or colour flow
is drawn — only about which are admissible.

**(c) A different PDF set is refused.** `pdf_mismatches` compares the artifact's
`pdf_set` and `pdf_member` against the ones this run would read, and the check runs
*before* the set is loaded, so a mismatch is refused by name rather than by a load
failure. The gap it closes is specific: the run card pins the LHAPDF *id*, but the
set a run actually loads is named by a flag, so an artifact and a command line can
agree on every card and still disagree about which tabulation trained the grids.
A fixed-energy run is held to the artifact's `none`/`0` by the same check.

**(d) The dynamical-scale card stays refused — in two places, and the plan had it
in one.** See the corrections below.

### Plan corrections

1. **`generate` at `lpp = 1` covers the general path only; `p p > e⁺e⁻` still
   refuses, by name.** The bespoke Drell–Yan integrand banks a single grid over its
   whole `(τ, y) × cosθ` map, which is not a channel decomposition, so there is
   nothing for an accept/reject pass to draw a channel from. This is not a
   regression — Drell–Yan was never generatable — but it is the opposite of the
   shape one would guess, and it decides Track U: **the acceptance job must move to
   `ℓℓj`, because Drell–Yan is the one hadronic process `generate` cannot do.**
2. **Gate (d) cannot be what it was written as.** `generate` never reaches the
   clustering refusal: a dynamical card does not match the one that trained the
   grids, so the card-mismatch check fires first — and no artifact of a dynamical
   card can exist, because `integrate` is where that card is refused. The test
   therefore asserts **both** refusals, on a card built from the banked one by
   turning off its three `fixed_*_scale` switches and nothing else, so neither
   refusal can be coming from something else about the card. The first names
   `run card \`fixed_ren_scale\``; the second names the clustering.
3. **The sample's σ estimator is bias-free where the integrator's is not**, as
   above. Worth carrying into any future comparison of a generated sample against
   its own banked σ.
4. **P3's operational facts all held.** The banked keys matched the derived
   `channel_ids()` pairwise on the first run; `set_channel_alphas` behaved; the
   artifact is `fv4`. The pairwise check is implemented and reports the offending
   position, but nothing has yet made it fire in anger.
5. **`AQCDUP` and `AQEDUP` sit a few 1e-7 from MadGraph's printed fields**, in both
   cases for a stated reason and neither is a gate. MadGraph writes
   `1.30002700e-01`; undoing its `π` truncation gives `1.30002698e-01` against our
   `1.30002712e-01`, so the two interpolations of the *same* `αs` grid differ by
   1.1e-7 relative. `AQEDUP` differs by 1.7e-7 because ours is the model's `aEW` and
   MadGraph's is `1/aEWM1` from its param card.

### The `cargo test` wall clock — a close-out item

The `pre-commit` hook runs a full `cargo test`, and that is now **18m46s** in the
debug profile. It is long enough to be a real obstacle to committing at all. The
likely contributors are the heavy default-feature tests this sprint added, all
running unoptimised: the flavour-group probe compiles 24 subprocesses, and
`ProtonIntegrand`'s adaptation, fixed-`ŝ` slices and (new) label sweeps each drive
many thousands of hadronic integrand points. Worth either moving the heaviest of
them behind a feature or giving the hook a profile.

### Gate results

`cargo fmt --check` clean. `cargo test --workspace` **523 lib passed / 0 failed**
(520 at P3's close + 3 new) plus the CLI suites — the binary's own **9**,
`cli_fixed_energy` 2, `cli_generate` 3. No tolerance was loosened anywhere; the one
tolerance *tightened* is the new gate's `AQCDUP` bound, from a guessed `1e-6` to
MadGraph's own printing budget for the field.

| gate | result |
|---|---|
| **`validate-generate-proton`** (new) | **3 passed** — 20 000 events, **48** flavour assignments over **6** initial-state arrangements, all in MadGraph's tables; σ(sample) **423.158 ± 0.209 pb** vs integration **423.731 ± 0.473 pb** (−0.135%); balance 7.02e-11, on-shell 5.11e-9, `AQCDUP` 1.09e-7 |
| `validate-lhef` | 3 passed — **248 747 events / 1 314 204 particle lines across 25 banked runs** byte-identical, `pp_to_llj_fixed` among them (10 000 events / 59 493 legs) |
| `validate_hadronic` | **5 passed** — llj **422.850 ± 0.189** vs MG **422.840 ± 1.805** (χ²/dof 1.90), seed for seed identical to P3 |
| ↳ Drell–Yan anchor | σ default **934.416 ± 0.870**, mmll window **644.855 ± 0.570**, pointwise **1.15e-14** — **identical to P2b, P2c, P2d and P3** |
| ↳ DY general-path info row | 933.706 ± 1.843, rel 0.0006 — unmoved |
| `cli_integrate` | 3 passed — CLI σ **934.415866 ± 0.869944** and **644.855362 ± 0.569603**, bit-for-bit as banked (the `integrate` PDF-loading refactor moved nothing) |
| `validate-alphas` | 5 passed — `pp_to_llj_fixed` **10 000/10 000** printed `AQCDUP` digits from the PDF grid, worst **0.281** of budget |
| `validate-scales` | 7 passed — `pp_to_llj_fixed` 50 000 comparisons, worst **0.000** of budget |
| `validate-color-flow-tags` | 30 passed — `pp_to_llj_fixed/P1_gq_llq` labels **identical to MadGraph's** |
| `validate-color-jamp` | 3 passed |
| `validate-helas-mg` | 18 passed — `gg_to_gg` 8.25e-14, `gg_to_ttx` 1.89e-15, unchanged |
| `validate-unweighting` | 1 passed — `ee_to_mumua` eff 2.872e-2, max `w/w_max` 8.393, unchanged |
| `validate-sigma` | 11 GATE rows asserted, unchanged |
| `validate-pdf-grid` | still not runnable here (`validation/pdf/oracle*.json` gitignored and absent, as P2b–P3 recorded) |

### What the sprint close-out needs

1. **TODO pipeline table**: steps 5 and 6 lose their `lpp = 1` deferral for the
   general path. Drell–Yan `generate` stays deferred and should be named as such.
2. **New pixi task** `validate-generate-proton`, and a line in the validation
   regime table.
3. **New backlog item — the unweighting scan budget.** `w_max` is scanned on the
   integration's own per-channel `neval`, and on a 24-channel hadronic run that
   leaves 0.5%–3% of the cross section above the maxima. Options: scan on a budget
   of its own, raise the maxima by a safety factor, or adopt MadGraph's two-pass
   `unwgt.f` treatment. This is the first process where it matters.
4. **Reinforces P3's discard-first-`k` item**: the same under-sampled channels drive
   both the integrator's `O(1/N)` bias and the `w_max` undershoot.
5. **New follow-up — `generate` for Drell–Yan**, i.e. either give the bespoke
   integrand a channel decomposition or route `p p > e⁺e⁻` through the general path
   for event generation only.
6. **Deferred, unchanged**: Pythia consumption of the emitted `.lhe`, and
   distribution-level event-sample-vs-MadGraph statistics. Both now have two
   processes waiting.
