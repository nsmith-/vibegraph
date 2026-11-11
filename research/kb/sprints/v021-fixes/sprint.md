---
type: Sprint
title: "Sprint J: v0.2.1 fixes"
description: "Before v0.2.1: the pairing-weighted cut_decays default, refusing diagram and integrate artifacts from an older enumeration, the SMEFT four-quark compile panic, the fixed-energy sde_strategy = 2 subprocess weights; then the release preparation."
status: draft
active: true
tags: [release, v0.2.1, cuts, decay-chains, identical-particles, artifacts, panics, sprint-j]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
sources:
  - {id: scope, resource: "../../pipeline/release-scope.md", title: "Release scope and the hard-error rule"}
  - {id: i, resource: "../v02-last-refusals/closeout.md", title: "Sprint I record and its Lessons"}
  - {id: d3, resource: "../v02-last-refusals/decisions/D3-identical-decays-behind-madgraph-compat.md", title: "Sprint I D3: the pairing-weighted default"}
---
## Goal and exit criteria

v0.2.0 is tagged on 3cc7f9c2. Main has since gained the musl allocator fix
(#128) and the `vibegraph enumerate` series (#114–#127). This sprint closes the
remaining panics and silent wrong answers before v0.2.1
([the hard-error rule](../../pipeline/release-scope.md)). It is done when:

- every claimed item's `closes_when` holds, with the oracle named below
  measured (not inferred) on the PR's head;
- every banked σ line is unchanged: bit for bit where the change cannot reach
  it, within error where it can;
- the CERN host's full CMS campaign on the release-candidate head shows no
  unexplained ok → not-ok and no panics (`full-cms-campaign-after-v02-cut`).
  `validate-deep` is skipped (the user's decision).

Every PR is based on `main`; none is stacked on another. The user tags the
release.

Out of scope: the multi-jet MLM timeouts; Majorana fermions, one-body phase
space, the coloured-scalar seagull, coupling-order inference, `import model`
options; param cards with entries for absent particles; `running.py`; #125.

## PRs, items and oracles

### PR 1: the pairing-weighted `cut_decays` default (`sprintj-weighted-cut`)

| Item | Pri | Oracle |
|---|---|---|
| `cut-decays-identical-quarks-cut-a-stricter-observable` | high | I-C1's probe (`ic1_window_probe.rs`) on the weighted integrand: w_p Θ_p \|M\|² ε² → 0 in its three configurations, and the unweighted control still constant. σ on `g g > z u u~, z > u u~` (MadGraph 7.211 pb) and on the full `g g > z j j, z > j j` card against local MadGraph, ≥ 5 seeds a side, unweighted mean, spread and χ²/dof read, with the difference attributed to the pairing, the interference and the exemption. Cards with no identical particles across decays stay bit for bit (every banked σ line, and a test). The guard runs per pairing with that pairing's exemptions, with a refusal test on a multi-body decay with massless products that fails before the change. |

**Decided (the user, 2026-11-11): option A.** The weighted cut is the only
behaviour; no `--madgraph-compat` flag is built in this sprint. MadGraph's
single pairing stays as site (d) of `madgraph-compat-sites-unconditional`, and
that item's text says so. The MadGraph σ on the two cards is recorded as an
attributed difference, not a match, and `release-scope.md` and the
registered-deviation text say the same.

### PR 2: refuse diagram artifacts from an older enumeration (`sprintj-stale-artifacts`)

| Item | Pri | Oracle |
|---|---|---|
| `diagram-artifacts-replay-stale-enumeration` (filed by this PR) | high | A test per artifact (diagram and integrate) that a file recording another enumeration version is refused by name with the remedy, failing before the change. A known-answer test that pins the enumeration output of a fixed card set to the version constant, and that fails when a deliberate change to conversion (the Fermi sign, say) is made without bumping it (mutation, recorded). |

**The design, and why.** Record an `ENUMERATION_VERSION` in both artifacts and
refuse a mismatch. A known-answer test makes it sound: it pins
`diagram_set_digest` of a small fixed card set (covering fermion flow, decay
chains, identical particles, a four-fermion contact vertex) to the version, so
a change to enumeration or conversion output fails CI until the version is
bumped. The alternatives:
- **Re-validation on read** re-enumerates the card, which at high multiplicity
  takes minutes (the CMS census's enumeration timeouts), and that defeats the
  point of the artifact.
- **A build identity** refuses an artifact on every rebuild, including the
  many builds that do not touch enumeration.

The blind spot: a change that moves only cards outside the known-answer set.
The concept says so.

### PR 3: two panics or silent answers (`sprintj-panics`)

| Item | Pri | Oracle |
|---|---|---|
| `smeft-four-quark-compile-panics` | medium | The repro card compiles and integrates, never panics. The cause is named: what produced an `Op::Add` with no operands, and whether it stood for a vanished amplitude (benign) or lost contributions (a dropped diagram). Per-diagram amplitudes against MadGraph standalone, like the other SMEFTsim rows. If a root-cause fix is not tractable, refuse it by name with a test. |
| `sde-strategy-2-weights-only-first-fixed-energy-subprocess` | medium | Reachability first: a card that reaches the path at fixed energy under `sde_strategy = 2` with two subprocesses of different diagram content, or a proof from the code that none does. If reachable, a fix or refusal with a test that fails before the change. If not, a test pinning the refusal that makes it unreachable, and the item re-scoped or closed with the evidence. |

### PR 4: close-out and release preparation (`sprintj-plan`, this PR)

The plan and the record; the workspace version 0.2.0 → 0.2.1;
`docs/release-notes/v0.2.1.md` (the musl allocator fix, the enumerate series,
this sprint's fixes, the known issues). Then the CERN host's full CMS campaign
on the release-candidate head, requested through the vibegraph manager. Its bar
is no unexplained ok → not-ok and no panics.

## Sessions

| Session | Agent | PR | Depends on | Work |
|---|---|---|---|---|
| [J-W1](sessions/J-W1.md) | validation-dev (Opus) | 1 | – | The pairing-weighted cut, the per-pairing guard, the probe and the σ rows. |
| [J-A1](sessions/J-A1.md) | feature-dev (Opus) | 2 | – | The enumeration version, its known-answer test, the refusals. |
| [J-E1](sessions/J-E1.md) | validation-dev (Opus) | 3 | – | The SMEFT compile panic. |
| [J-E2](sessions/J-E2.md) | validation-dev (Sonnet) | 3 | – | The `sde_strategy = 2` reachability and fix or re-scope. |
| J-Z | manager | 4 | PRs 1–3 merged | The version bump, the release notes, the campaign request, the record. |

J-A1, J-E1 and J-E2 run first, in parallel worktrees. J-E2 works on its own branch,
merged into `sprintj-panics` with a merge commit. Heavy runs (seed sweeps,
`validate.sh`, local MadGraph) go one at a time on this 4-core container, and
the manager serialises them.

## What is known before the sprint (manager, 2026-11-11)

- **No `--madgraph-compat` flag exists** (no CLI argument, no reference in the
  code). The user chose option A (above).
- **Diagram artifacts** (`diagrams/artifact.rs`, format version 2) carry a
  schema version, the model digest and `diagram_set_digest`, but nothing that
  identifies the enumeration or conversion that produced the sets. The integrate
  artifact (format version 23) embeds the same sets. Its amplitude probe
  recompiles from the banked diagrams, so it agrees with itself
  ([integrate artifact](../../pipeline/integrate-artifact.md), check 7's blind
  spot).
- **References.** Local MadGraph runs here (the pinned submodule, gfortran per
  [the MadGraph toolchain](../../tooling/madgraph-toolchain.md); no LHAPDF, so
  `nn23lo1`, the same set as `lhaid 247000`). The CMS campaign runs on the CERN
  host only.
