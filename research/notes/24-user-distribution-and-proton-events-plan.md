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
