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
