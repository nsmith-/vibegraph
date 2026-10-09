---
type: Validation Gate
title: "Release acceptance: released binary, cards in, events out"
description: "scripts/acceptance.sh on a published release: refusal without consent, consented pinned PDF fetch, llj integrate, offline generate from cache, check-events; run by acceptance.yml."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [acceptance, release, ci, pdf, lhef]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-u4-acc, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2816-L2902", title: "Note 24 §U4 outcome: Acceptance A, what it covers and cannot see"}
  - {id: n24-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L3058-L3113", title: "Note 24 close-out: Acceptance A, one process"}
  - {id: script, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/scripts/acceptance.sh", title: "scripts/acceptance.sh"}
  - {id: workflow, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/.github/workflows/acceptance.yml", title: ".github/workflows/acceptance.yml"}
measured:
  command: "bash scripts/acceptance.sh --binary target/release/vibegraph (live CERN download)"
---
# Release acceptance: released binary, cards in, events out

The question this gate answers is the one no other check can: does the
artifact a user downloads work on a machine that never built it? It is not a
physics gate.

**State: it has never passed in CI.** See
[acceptance-yml-never-passed](../backlog/hygiene/acceptance-yml-never-passed.md)
for the open state and the next step, and
[acceptance-yml-fails-on-refdata-releases](../backlog/hygiene/acceptance-yml-fails-on-refdata-releases.md)
for why every `refdata-*` prerelease currently produces a red run. The script
itself is exercised locally through `--binary`.

## The script

`scripts/acceptance.sh` depends on nothing in the repository beyond being
stored there: it writes its own cards and, without `--binary`, downloads its
own binary. `curl` and a POSIX shell are the whole dependency list
(`sha256sum` or `shasum` is used when present to verify the asset), so
the same script is the CI job and the documented clean-VM reproduction.

Setup:

1. Obtain the binary. From a release (`--repo`, optional `--tag`, default
   `releases/latest/download`), verify it against the release's `SHA256SUMS`
   when both the file and a hashing tool exist (saying so plainly when they do
   not), and require `--version` to print the MadTeam copyright line, since a
   binary without the notice should not have shipped
   ([licensing](licensing.md)). With `--binary <path>`, the path is made
   absolute before the script changes directory; a relative path once broke
   step 1 and was misreported as "the refusal does not name the flag".
2. Write `proc_card.dat` (`import model sm`, `generate p p > l+ l- j QCD=2 QED=2`)
   and a run card: 13 TeV, `lhapdf`/`247000`, all three scales fixed at 91.188
   (a 2 → 3 dynamical scale would need kT clustering), `mmll 50`, `ptj 20`,
   `ptl 10`, `etaj 5`, `etal 2.5`, `drll 0.4`, `drjl 0.4`.
3. `VIBEGRAPH_HOME` points at a scratch cache root; `VIBEGRAPH_PDF_DIR`,
   `VIBEGRAPH_UFO_DIR` and `VIBEGRAPH_NO_NETWORK` are unset; everything runs
   in a work directory with no `validation/pdf` beside it, so the dev fallback
   cannot satisfy a resolution that should reach the cache.

The four steps, budget `NEVAL=20000`, `NITER=4`, `NEVENTS=2000`:

| step | command | what it proves |
|---|---|---|
| 1 | `integrate`, no consent flag | an unattended run refuses, naming `--yes` and the URL, and leaves no cache entry |
| 2 | `--yes integrate` | the 27 MB set downloads, matches the compiled-in SHA-256, is published into the cache; prints a σ |
| 3 | `--no-network generate` | the event file is written; generation needs the same PDF set and nothing may fetch it, so this is also the cache-hit check |
| 4 | `check-events --min-events 2000` | the `.lhe` reads back through the binary's own parser |

`p p > l+ l- j` is the subject because it is hadronic, so PDF resolution,
consent, the pinned download and the cache all sit on the path under test,
and `generate` supports it, so one set of cards reaches an event file. The
budget keeps wall time dominated by the download. Measured once end to end
against a locally built release binary with a live download: **26.9 s wall,
3.4 s CPU**, σ = 414.4 ± 3.0 pb.[^n24-closeout] That σ is compared to
nothing; it sits below the gated value because a tenth of the gate's budget
carries the low-budget bias measured at the time (416.9 pb at 30 000, 423.7 at
300 000). The σ gate itself is [validation/sigma-gate](../validation/sigma-gate.md).

`check-events` checks momentum balance (initial minus final, intermediates
excluded), mass shells against `E²`, `XWGTUP ≤ XMAXUP`, unit weights when
`IDWTUP = +3`, `IDPRUP` against a declared `<init>` process, mother indices in
range, and `--min-events`, at default tolerance `1e-6`. It shares its
assumptions with the writer, so a self-consistently wrong format passes, and
it is blind to physics: a wrong matrix element, cut or sampler produces events
that satisfy every identity. It catches damage, not error.[^n24-u4-acc]
`a_generated_sample_passes_check_events_and_a_damaged_one_does_not`
(`cli_first_run.rs`) keeps it non-vacuous by displacing one leg's `pz` by a GeV
and requiring rejection.

## The workflow

`.github/workflows/acceptance.yml` runs the script on one `ubuntu-latest`
runner, on `release: published` and on `workflow_dispatch` (input `tag`;
empty means the latest release). `release.yml` dispatches it after publishing,
because its own token's release event does not start workflows
([release binaries](release-binaries.md)).

Not on pushes or PRs, deliberately: its input is a published release asset,
which a branch does not have, and it downloads 27 MB every run on purpose
(caching the set would remove the path under test). `ci.yml` covers the same
consent, resolution and event-file logic offline on every change. One
platform only: `release.yml` already smoke-tests each platform natively, and
the data path is not platform-specific.

## What it cannot see

- **Physics.** Every number it prints is unchecked; σ gates live in the
  validation suite.
- **macOS** at the data-path level.
- **Upstream repackaging on a cache hit.** On the miss path it downloads and
  verifies every run, so on a timer it would be the second detector, beside an
  ignored test, for CERN re-tarring the pinned archive
  ([asset resolution](asset-resolution.md)). The weekly `schedule` trigger is
  not on yet: [acceptance-yml-weekly-schedule](../backlog/hygiene/acceptance-yml-weekly-schedule.md).
  The workflow's header comment still gives "until a first release exists" as
  the reason; the backlog item records that `v0.1.0` exists and the real
  blocker is that no run has been green.
- **Anything until it runs green once.** Its GitHub-side behaviour is
  unproven.

An earlier two-leg transcript in note 24 (7.5 s, labelled "verified for
real") cannot have come from the committed script; only the run quoted above is
reproducible.[^n24-closeout]

[^n24-u4-acc]: Note 24 §U4, "Acceptance A: what it covers today, and what it cannot see"; the `check-events` design.
[^n24-closeout]: Note 24 close-out, "Acceptance A: one process, cards to events".
