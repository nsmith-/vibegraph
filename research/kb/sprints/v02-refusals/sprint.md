---
type: Sprint
title: "v0.2 refusals: panics and silent wrong answers"
description: "Every model or card surface that panics, defaults silently or reports σ = 0 as a success is refused with a named error or fixed; part one of sprint G, one PR."
status: draft
active: true
tags: [refusal, panic, ufo, param-card, restrict-card, reweight, sigma, sprint-g, v0.2]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
## Goal and exit criteria

Sprint G makes every card or model surface that panics today, or silently
gives a wrong or empty answer, a refusal with a named error, or fixes it. It is
a v0.2 must-have under [the hard-error rule](../../pipeline/release-scope.md).
This PR is its first part. Done when each claimed item's `closes_when` holds,
every clause pinned by a test that fails before the change or by a recorded
measurement, CI is green and the gates below pass.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `spin-2-panics-instead-of-refusing` (option (a), refuse)
- `lorentz-and-spin-code-paths-panic-instead-of-refusing`
- `ufo-required-fields-default-silently`
- `restriction-zero-coupling-threshold-differs`
- `input-cards-lack-field-classification`
- `zero-cross-section-reported-as-ok`

The diagnoses (card 048's one-point σ, VEGAS under-coverage, the heap aborts)
are the second part, `v02-diagnoses` (sprint folder on its own PR), on their own
PR; the misleading CLI output lines are the optional third,
`v02-cli-output`.

**Why this split.** Everything here is a deterministic refusal or a rule port
with a hermetic test that fails before the change. It can converge on CI and
the banked gates without waiting on another host. The diagnoses depend on the
CERN host (CVMFS gridpacks) and the M3 (macOS soaks), and on long statistical
runs. Bundling them would hold these refusals hostage to relays. The CLI lines
change the artifact format and the summary output that sprint F's work also
reads, so they go last and alone.

## Decisions (user, 2026-11-03)

- **G1. Spin 2 is refused** (option (a)); support is out of scope for v0.2.
- **G2. σ = 0 and too few nonzero points are both refused.** `integrate` exits
  non-zero and writes no artifact when every point has zero weight, naming how
  many points passed the cuts and whether |M|² vanished on those that did. A
  run with fewer nonzero points than a threshold is also refused: its σ means
  nothing either. The threshold is set from measurement, far below what any
  banked row reaches at its budget. `generate` refuses an artifact recorded at
  σ = 0.
- **G3. Port MadGraph's restriction rule** (|g| < 1e-13 is zero; strict-zero
  rerun when any coupling lies in [1e-13, 1e-10)). It is a MadGraph
  compatibility choice, so it is added as a site of
  [madgraph-compat-sites-unconditional](../../backlog/feature/madgraph-compat-sites-unconditional.md),
  whose off mode would use a cleaner rule.
- **G4. All three input cards get a classification table**, the param card
  first, then restrict and reweight.

- **G6. A param card's decay tables are carried** (user, 2026-11-04), not
  refused: the branching ratios go into the `<slha>` header as MadGraph writes
  them, so MadWidth-written gridpack cards bind and the shower sees their BRs.
- **G7. A negative mass flips its width's sign, as in MadGraph** (user,
  2026-11-04): MadGraph's `write_inc_file` rule is ported into the model
  evaluation and pinned against MadGraph on MSSM_SLHA2 (`Mneu3 < 0` in its
  default restriction); the param-card refusal of negative masses is lifted.
  **Amended (user, 2026-11-04):** no supported model has a negative mass
  (MSSM's `Mneu3` is a Majorana neutralino, and Majorana fermions are
  refused), and the sites downstream of the width (phase-space Breit–Wigner
  map, on-shell checks, cuts, LHEF resonances) are not compared with
  MadGraph. So the ported rule stays, pinned, and a negative mass on any
  non-Majorana particle is refused at binding.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| G-F1 | feature-dev (Opus) | – | hermetic, lints, amplitude gates | spin-2, lorentz-and-spin |
| G-F2 | feature-dev (Opus) | – | hermetic, lints, restriction/model gates | ufo-required-fields, restriction-threshold |
| G-F3 | feature-dev (Opus) | G-F2 | hermetic, lints, param-card and reweight gates | input-cards-lack-field-classification |
| G-F4 | feature-dev (Opus) | CERN re-run of the H2ToH1H3 pair | hermetic, lints, CLI tests, banked σ gates unchanged | zero-cross-section-reported-as-ok |
| G-F5 | feature-dev (Opus) | G-F3 | hermetic, lints, LHEF and amplitude gates; MSSM per-helicity check | – (decisions G6, G7) |
| G-HY1 | hygiene-dev (Opus) | G-F1–F5 | hermetic, lints | – |

G-F1 and G-F2 touch disjoint files (`helas/eval`, `ufo/lorentz.rs` and
`ufo/particles.rs`'s helicity list against the `ufo` loaders' field parsing and
restriction) and run in parallel. G-F3 follows G-F2 because both edit the
`ufo` card and restriction code. G-F4 edits `integrate`'s end of run and
`generate`'s artifact read; it waits for the CERN host's `--bake` re-run of
the two zero-σ gridpacks, which says whether their zero is the default
parameters or a vibegraph bug.

Two dev agents run at a time in this container (4 cores, ~30 GB free), with
the heavy gate suites serialised by the manager.

## Coordination with sprint F

Sprint F (CMS production readiness) works on `generate`, PDF loading and
systematics. The only overlap here is G-F4's refusal in `generate` of a σ = 0
artifact, a few lines at the artifact read. The manager tells the vibegraph
manager before that lands.
