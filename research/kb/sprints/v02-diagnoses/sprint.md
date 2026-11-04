---
type: Sprint
title: "v0.2 diagnoses: one-point σ, VEGAS under-coverage, heap aborts"
description: "Diagnose card 048's one-point σ, make VEGAS detect (or warn on) an under-covered cut slice, and attribute the two macOS heap-corruption aborts; part two of sprint G, one PR."
status: draft
active: true
tags: [diagnosis, vegas, heap-corruption, decay-chains, sprint-g, v0.2]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
## Goal and exit criteria

The second part of sprint G (the first is
`v02-refusals` (sprint folder on its own PR), which also says why the sprint is
split). Each item here is a silent wrong answer whose cause is not known yet.
Done when each claimed item's `closes_when` holds, every clause backed by a
test or a recorded measurement (statistical claims under AGENTS.md's seed
policy), and CI is green.

## Scope

- `vector-triplet-chain-one-nonzero-point`
- `vegas-under-covers-heavily-cut-slice`
- `malloc-abort-under-proton-sample-load`
- `validate-hadronic-sigabrt-unexplained`

## Decisions (user, 2026-11-03)

- **G5. VEGAS under-coverage.** Detecting it is the fix. If reliable detection
  does not fit the sprint, the known-limitation fallback is a **runtime
  warning**: `integrate` warns when a cheap indicator says the result is at
  risk. The proposal goes to the user before it lands.
- Card 048's run is also caught by v02-refusals' too-few-nonzero-points
  refusal (decision G2); the diagnosis here still says why one point passed
  and why μF read 0.

- **G8. A whole-state narrow s-channel resonance gets a Breit–Wigner τ map**
  (user, 2026-11-04), fixed here (G-V4), not refused.
- **G9. MadEvent's small-width convention is a registered deviation** (user,
  2026-11-04), and a site of the MadGraph-compatibility flag.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| G-V1 | validation-dev (Opus) | CERN host relay (card 048 per-event record) | hermetic, lints; a test pinning any defect found | vector-triplet-chain-one-nonzero-point |
| G-V2 | validation-dev (Opus) | – | the slice against the non-adaptive estimate, ≥5 seeds per arm; banked σ gates unchanged | vegas-under-covers-heavily-cut-slice |
| G-V3 | validation-dev (Opus) | M3 soak relay | sanitizer and audit runs recorded | malloc-abort-under-proton-sample-load, validate-hadronic-sigabrt-unexplained |
| G-V4 | feature-dev (Opus) | G-V1 | the σ gates; ≥5 seeds against MadEvent on `p p > h z` | – (decisions G8, G9; G-V1 Found 1) |
| G-HY2 | hygiene-dev (Sonnet) | G-V1–V4 | hermetic, lints | – |

**Hosts.** Card 048's gridpack lives on CVMFS, reachable only from the CERN
host; the macOS soak (`MallocNanoZone=0` against the default) runs on the
user's M3. Both requests are posted as exact step lists on this PR and relayed
by the vibegraph manager. G-V3's Linux side: an `unsafe` audit of the
dependency tree, Miri on the pure-Rust suspect paths (`ScratchSpace`, the kT
clustering) where it can run, and clang AddressSanitizer covering `zstd-sys`.
