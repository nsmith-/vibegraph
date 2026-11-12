---
type: Sprint Record
title: "Sprint J: the v0.2.1 fixes"
description: "Five fix PRs before v0.2.1: refusing artifacts from another enumeration, the pairing-weighted cut_decays default, two SMEFT and sde_strategy panics, the leg-order-dependent relative Fermi sign, and four SMEFTsim dipole defects; each pinned by a MadGraph oracle that failed before it. The release waits on the CERN host's CMS campaign."
status: draft
tags: [release, v0.2.1, cuts, decay-chains, identical-particles, artifacts, panics, fermion-sign, smeft, sprint-j, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-12}
sources:
  - {id: plan, resource: "sprint.md", title: "Sprint J plan"}
  - {id: log, resource: "log.md", title: "Sprint J log"}
  - {id: jw1, resource: "sessions/J-W1-report.md", title: "J-W1 report"}
  - {id: ja1, resource: "sessions/J-A1-report.md", title: "J-A1 report"}
  - {id: je1, resource: "sessions/J-E1-report.md", title: "J-E1 report"}
  - {id: je2, resource: "sessions/J-E2-report.md", title: "J-E2 report"}
  - {id: jf1, resource: "sessions/J-F1-report.md", title: "J-F1 report"}
  - {id: jd1, resource: "sessions/J-D1-report.md", title: "J-D1 report"}
  - {id: notes, resource: "../../../../docs/release-notes/v0.2.1.md", title: "v0.2.1 release notes"}
---
Sprint J ran six PRs from the [plan](sprint.md). The user approved #130–#134,
and each was squash-merged after a merge commit brought `main` in and CI
(`test`, `banked`) passed. Under the user's rule, CI was the gate: the sessions
ran fmt, clippy, targeted tests, failing-first checks and the MadGraph rows that
served as oracles, but no local full-workspace run after the first round. The
close-out is #129 (this PR).

## What landed

| PR | Squash | Closed | What it established |
|---|---|---|---|
| #130 | b76aa859 | `diagram-artifacts-replay-stale-enumeration` | Diagram and integrate artifacts record `ENUMERATION_VERSION` and refuse another by name. A known-answer test pins the set digests of seven cards to the version, so a change to enumeration output fails CI until the version is bumped. The mutation is recorded. ([J-A1](sessions/J-A1-report.md)) |
| #131 | a2a37b1a | `cut-decays-identical-quarks-cut-a-stricter-observable` | `cut_decays = F` with identical particles across a decay applies Σ_p w_p Θ_p \|M\|², the only behaviour (option A). The weighted integrand has no beam-collinear pole, and the limit guards run per exemption. Reweighting such a card is refused, and so is such a card with a `$` veto. ([J-W1](sessions/J-W1-report.md)) |
| #132 | 88e58cbc | `smeft-four-quark-compile-panics`, `sde-strategy-2-weights-only-first-fixed-energy-subprocess` | Each colour chain is taken from its occurrence's flow group, and an empty Lorentz term list is refused. Non-adjacent pairings are re-rooted through `build_rerooted`. At fixed energy, `sde_strategy = 2` builds one `ChannelSet` per subprocess. The four-quark rows agree with MadGraph per flow. ([J-E1](sessions/J-E1-report.md), [J-E2](sessions/J-E2-report.md)) |
| #133 | 95119d91 | `fermion-sign-wrong-for-crossed-pairings` | `Diagram::fermion_pairing_sign` orders each pair without reading leg numbers. One cause covered Bhabha with relisted legs (`e+ e- > e- e+` read 8.3% high) and the W class of `u u~ > t t~ b b~ QED<=2`. Three new standalone rows go from O(0.2–0.5) to 1e-15. `ENUMERATION_VERSION` is now 2. ([J-F1](sessions/J-F1-report.md)) |
| #134 | 5c704bb7 | `smeft-dipole-at-amplitude-root-disagrees` | Four SMEFTsim dipole defects: the reversed two-gamma chain order, the conjugation of a re-rooted chain, the Hermitian-partner coupling (limited to three-leg FFV gamma currents), and the colour-structure sign on two-fermion, two-vector contacts (ttH included). Any other vertex that would take that sign is refused. 14 new rows, including `ttx_to_wpwm_ctw` and `gg_to_ttxh_ctg`. ([J-D1](sessions/J-D1-report.md)) |

**Manager-verified on the merged trees.** Before each push the manager ran fmt,
both clippy runs, kb lint and these targeted tests:
- the known-answer test and the pairing tests (5 passed after #133's merge);
- `standalone_jamps`, 61 passed after #133 and 75 after #134, with
  `uux_to_ttxbbx_4f_nodipole` moved out of `KNOWN_DISAGREEMENT` and enforced;
- `helas:: diagrams:: ufo` lib tests (506) and `amplitude_oracle` (51) after
  #134.

#134 changed neither `Diagram::sign` nor the enumerated vertices, so the
known-answer test needed no re-pin. Its rewrites run inside `build_rerooted`,
before `compact_for_caller`. Taking #134's side of `root_lorentz.rs` wholesale
would have restored #132's panic.

## The identical-particle rows after #133 (manager)

#133 reverses the relative sign between pairings on decay chains with identical
particles. `chain.rs` relabels through the same `fermion_pairing_sign`. So
J-W1's σ attribution was re-measured on main 5c704bb7. The setup was identical
to J-W1's: run cards, seeds 1–5, `--target-rel 0.003 -j 3`, the 20000/40000-event
samples, the per-event scripts, and the same five MadGraph seeds. The full
tables are in [J-W1 §5](sessions/J-W1-report.md).

| card | vibegraph | MadGraph | vg/MG − 1 | interference | window union | per-event closure |
|---|---|---|---|---|---|---|
| ggzuu F | 7.249 ± 0.009 | 7.206 ± 0.013 | +0.60 % ± 0.22 % | −0.46 % | +0.84 % | +0.20 % |
| ggzuu T | 5.293 ± 0.007 | 5.288 ± 0.009 | +0.10 % ± 0.22 % | −0.51 % | +0.64 % | −0.03 % |
| ggzjj F | 150.16 ± 0.17 | 150.09 ± 0.23 | +0.05 % ± 0.21 % | −0.14 % | +0.19 % | −0.01 % |

The interference is now destructive, as Fermi statistics requires. vibegraph's
χ²/dof is 0.30–1.35.

**`e+ e- > z z, z > e+ e-` (`zz_ee`, informational).** Ten seeds read
+0.25% ± 0.08% against MadEvent (+0.26% before #133). A per-event decomposition
of 20000 events shows why the sign fix left it in place: the interference there
is negligible (⟨Σ_q|M_q|²/|M|²⟩ = 0.99996), and the window union gives +0.11%.
That leaves **+0.14% ± 0.08% unattributed**, filed as
[zz-ee-sigma-excess-not-attributed](../../backlog/validation/zz-ee-sigma-excess-not-attributed.md).
#133's notes had attributed this row's excess to the interference. The
manifest, `cli_decay_chain_events.rs` and the concepts now say otherwise.

## Release preparation

- Workspace version 0.2.0 → 0.2.1 (`Cargo.toml`, `Cargo.lock`, `pixi.toml`).
  `scripts/release-check.sh v0.2.1` passes.
- [Release notes](../../../../docs/release-notes/v0.2.1.md): the musl allocator
  fix (#128), `vibegraph enumerate` and diagram artifacts (#114–#121),
  configuration shares (#124, #126), the three physics fixes, the two panics,
  and the known issues with the post-#133 numbers.
- The CERN host's full CMS campaign on the release-candidate head is requested
  on #129 (`full-cms-campaign-after-v02-cut`). Its bar is no unexplained
  ok → not-ok and no panics. Expected movers: four-lepton, ZZ, `b b~ b b~` and
  electroweak `t t~ b b~` cards (#133); SMEFTsim top-dipole cards (#134); and
  `cut_decays = F` decay chains with identical particles across decays (#131).
  The user tags the release.

## Found

Filed by this record:
- [zz-ee-sigma-excess-not-attributed](../../backlog/validation/zz-ee-sigma-excess-not-attributed.md) (above).
- [fixed-beam-pairing-union-sigma-unmeasured](../../backlog/validation/fixed-beam-pairing-union-sigma-unmeasured.md) (J-W1 Found 3).
- [fixed-beam-reweight-audit-names-wrong-cause](../../backlog/validation/fixed-beam-reweight-audit-names-wrong-cause.md) (J-W1 Found 6).

Recorded, not filed (each report gives the closing step):
- **J-W1 Found 4.** Flavour-group members share the representative's pairing
  amplitudes, and no probe compares them.
- **J-W1 Found 5.** The weighted rows have no budget rung, which matters only
  if they are ever gated.
- **J-E2.** The fixed-energy scale is drawn from subprocess 0's
  configurations, where MadEvent draws it per subprocess. No σ difference was
  seen.
- **J-A1 Found 2.** The known-answer test has never run off Linux.
- **J-A1 Found 3.** `integrate`'s JSON result and `inspect` do not report the
  enumeration version.
- **J-D1 Found 1.** The rooting sweep never roots a reversed multi-gamma chain
  both ways.
- **J-D1 Found 3.** The item `conjugate-vector-current-fallback-silent` names a
  function that is now `UFOModel::conjugate_couplings`. Its premise stands.
- **J-D1 Found 4.** Hermitian-partner coverage outside the chain normal form is
  unmeasured.
- **J-D1 Found 6.** `boson_momentum_sign` and `fermion_structure_constant_sign`
  may be one rule.
- **J-E1.** Two re-checks remain: the full `u u~ > t t~ b b~ NP<=1` card against
  MadGraph with the dipoles on (J-E1 recorded 208.8 fb), and an
  `e+ e- > e- e+ NP<=1` row.
- **J-F1.** No banked row lists a fermion pair against its labels.

## Lessons

- **A note's attribution is a claim until an oracle splits it.** #133's
  description and the `zz_ee` note both expected the sign fix to move that row
  through its interference. The row did not move. The per-event decomposition
  showed the interference there is 4e-5 of σ, so the earlier attribution had
  never been measured on that card.
- **Merge order carries physics.** #131's attribution was measured before #133.
  #132's tripwire row could only pass after #133. #134's root rewrites had to
  move into #132's new function. Each PR description listed its merge
  interactions, and the manager resolved them at the merge commit, with the
  oracle re-run there.
