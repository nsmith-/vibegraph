---
type: Sprint Record
title: "Sprint L: v0.2.3, squared orders, Majorana, multijet timeouts and simshop's reports"
description: "Fifteen feature and validation PRs, eight hygiene passes and the kb close-outs before v0.2.3. Every claimed item closed on a measured oracle. The late validator lane found and closed two apparent σ defects as a pre-fix binary and the complex-mass scheme."
closed: 2026-11-25
status: draft
tags: [release, v0.2.3, squared-orders, majorana, chirality, mlm, performance, simshop, cms-gridpacks, sprint-l, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: plan, resource: "sprint.md", title: "Sprint L plan"}
  - {id: log, resource: "log.md", title: "Sprint L log"}
  - {id: pipeline, resource: "pipeline-closeout.md", title: "vg-sprint-L-pipeline close-out notes"}
  - {id: v03, resource: "../../decisions/v03-branch-after-v023.md", title: "The v0.3 split after v0.2.3"}
---
The sprint ran as two sprint sessions, `vg-sprint-L-physics` and
`vg-sprint-L-pipeline`. They dispatched one dev agent per PR and verified each
one. `vg-validator` (the CERN host) ran the campaign-scale gates on branch
builds, and from 2026-11-24 on dev builds through batch. The planner filed the
backlog, merged on green, and kept `vg-simshop` and `vg-batch` apart from
sprint traffic. Each session's brief and report is in `sessions/`.

## What landed

| PR | Squash | Session | What it established |
|---|---|---|---|
| #168 | 4b132cc5 | L-Q1 | Squared-order constraints (`^2==`, `^2<=`) by coupling-order split. The order classes partition \|M\|². |
| #169 | a5bf8fca | L-M1 | Majorana fermions and fermion-number violation, per diagram and per helicity against MadGraph `AMP()`. It also fixed the charged-scalar chirality bug: a bare chiral projector between non-self-conjugate particles was exchanged the wrong way round. |
| #172 | 7ca8d8bd | L-U1 | `$` run-card placeholders are refused, configuration-conditional UFO parameters load, and `time_of_flight` is handled. |
| #178 | c18042ed | (L-X1) | Goldstones flagged `GoldstoneBoson` are dropped, and list/dict propagators are read. This closes bstar_tW. |
| #171 | d5e7c71f | L-R1 | The model-parameter report and machine-readable artifact provenance (simshop). |
| #179 | e99f8ba2 | L-W1 | `DECAY Auto` bound to the tree-level two-body width (simshop). |
| #183 | abc3b31e | L-T1 | Streamed per-phase timing for `integrate` and `generate`. |
| #173 | f08aa3ce | L-T2 | Per-family verdicts for the v022-1009 never-retried timeouts. |
| #182 | dfdf85d4 | L-H1 | Reweighting in each event's own helicity (simshop). This changes the reweight default. |
| #186 | b4e284a3 | L-Y1 | Group index and run-global position on ldjson progress records. |
| #185 | ae88d176 | L-T3 | The conserved-number flavour prune, parallel enumeration walks, parallel set compilation, and one compile per set. |
| #180 | 5b1f9f71 | L-S1 | Multicore `generate`: blocked accept/reject on seed-placed streams. The output is independent of `-j`. |
| #170 | 2ac778e2 | L-V1 | The VBS stable-boson σ excess is two MadGraph effects: 2.9.18's sampling deficit, and MadEvent's default-survey bias. |
| #174 | 2d9f18ea | L-G1 | The g2HDM charged-Higgs excess is the chirality bug that #169 fixed. |
| #188 | 5a1c33e8 | L-G2 | `generate` resolves each group member's resonance code and colour from its own diagrams. |
| #184 | 8b3af5ab | L-P1, L-P2 | The multichannel transfer bound at the leg's own cut. This closes the TprimeBToTZ region miss. |
| #177 | 970be89d | L-X1, L-X2 | Campaign leftovers: TpTp is a card defect that MadGraph shares, bstar_tW, the stitch assert becomes a refusal, and param-card entries the model lacks bind as ignored, per the user's rulings. |
| #193–#200 | 049fdc3e, 0c6f398b, ab4288a1, cd35c4e6, 0f1c1e8e, 839033b9, 71ae08e4, 1d50eca1 | L-Z1 to L-Z8 | Eight hygiene passes, one per PR cluster. |
| #201 | 92877716 | L-U2 | The regression L-Z5 found: a `Parameter`-free `if` on an unevaluated condition loads again. |
| #191 | 486668d8 | | The committed `vibegraph-cli/target` build tree is untracked, and `target/` is ignored in every crate. |
| #206 | 3f4dd815 | | The dev-build CI workflow: static musl binaries of unreleased commits, for batch. |
| #210 | c16f8ec2 | | CI on `release/v*`, the backport flow, and no publish from a dispatch. |
| #211 | 54cecfac | | Release prep: version 0.2.3, `ENUMERATION_VERSION` 4, and the release notes the user approved. |
| #214 | c58fc146 | L-D1, L-D2 | Complex couplings on charged two-fermion contacts and on projector-written self-conjugate dipoles bind as MadGraph's. Card 6 (tZq EFT CPV) goes from 1.655 to 1.000 ± 0.007 of MadGraph at 4× budget, and card 7 (ttZ) from 1.215 to 1.015. |

Records and kb-only PRs:
- #189 and #190: the pipeline close-out and the v0.3 decisions;
- #202 and #204: per-member XWGTUP, L-X3;
- #205: the PR 11 gates;
- #203: the g2HDM close-out;
- #208: L-I1;
- #187, #181, #192, #207 and #209: Found items and planner filings;
- #213 and #215: stage B's card 6/7 findings, the imaginary-coefficient census and the dim6top convergence item.

## Validation on the merged code

- **g2HDM charged Higgs.**
  - V1, 5 seeds: 894 reads vg/MG 0.9991 ± 0.0088 (0.9903 without seed 4) and 889 reads 0.9995 ± 0.0062.
  - There is no claim of 1%-level agreement on 894. Its six-body rows are heavy-tailed: n_eff/N is 0.07–0.29% on every seed (`g2hdm-894-six-body-rows-heavy-tailed`).
  - V2 on batch, one seed per card on main 3f4dd815: 890–893 and 895 read 0.986–1.016.
  - These cards take 3.3–4.2 h each at campaign budget, and 895 peaks at 4.6 GB, so they need batch.
- **PR 11 cards.**
  - 157 (bstar_tW): 1.0069 ± 0.0075 over 20 seeds.
  - Bprime 156, 877 and 878: 0.9925, 1.0023 and 0.9886 over 20 seeds.
  - DarkPhoton 79, 418 and 419: 0.991–0.998.
  - DarkPhoton 78: bimodal at 4× (`darkphoton-mzd40-seeds-confidently-low`).
- **PR 5.**
  - Set A: the five cards flagged at > 3σ (127, 615–618) are main's missed region, which PR 5 removes.
  - Set B: 49 of 59 cards are within 2σ. 665 and 653 read low on both builds at fixed budget (`vbf-heavy-resonance-low-at-campaign-budget`).
  - The 126, 127 and 611–619 rows are before/after evidence for PR 5 only. Both sides ran with `complex_mass_scheme` stripped, so they are not MG agreement.
- **PR 1, request A.** All 26 squared-order cards now parse; in v022-1009, 23 were refused and 3 timed out.
  - VLQ 124, 125 and 610 agree with MG's signed σ on main.
  - VLQ 612 and 614 agree with MG in the real-mass scheme at 8×: 0.986 ± 0.019 and 0.982 ± 0.044. Against their complex-mass gridpack references they are not comparable (below).
  - The 14 ssWW cards enumerate but exceed the campaign cap (`ssww-enumerate-oom-in-teardown`).
  - The aajj cards stop on photon isolation (`photon-isolation-ptgmin-unimplemented`).
- **PR 2's 31 campaign targets** (TypeIII, LQ S3, suu, VLQ to ττ) and the 7 HNL cards are all decay chains. All are refused at the decay-chain Majorana/FNV guard (`majorana-in-decay-chain-refused`). PR 2's validation covers undecayed processes only.
- **Census 2b.** PR 2 leaves σ unchanged on the 2HDM cards 122 and 585.

## Two apparent defects, closed on measurements

- **The VLQ interference sign.**
  - PR 1's request A ran on bin-pr1 9f196713, which does not contain #169. On 612 and 614 it gave the opposite sign to MadGraph.
  - L-I1 showed main matching MG 3.7.1 standalone per diagram to 1e-15, and the pre-#169 build off by the chirality exchange alone.
  - The same run found that gpdb had compared negative-weight runs against the absolute integral. nsmith-/vibegraph-valdb#3 fixed it; the live DB is now at schema 5, and 15 references changed.
- **The 612/614 deficit after the sign was right.**
  - The 4× and 8× rungs excluded budget. L-I1 found the t-channel zero-width sets matching on all 12 subprocesses.
  - MG 3.7.1 with the complex-mass scheme on against off gives on/off 1.228 (612) and 1.74 (614), which is the whole gap.
  - The gridpacks set the scheme, and the campaign's `strip:set` patch removed it silently.
  - The 4× "dip" was 5-seed scatter: at 20 seeds, 4× and 8× agree to 0.4%.

## Release

v0.2.3 is not tagged yet. The release candidate is main 54cecfac (#211). Before the tag:
- `vg-validator`'s targeted check of v022-1009's failures, on the dev build of 54cecfac. Stage A (`integrate --check` on all 1000 cards) is running. Stage B (1-seed integrations of newly running cards) waits on the user's approval of its batch cost.
- The history rewrite dropping `vibegraph-cli/target/`, from d5e7c71f on. No v0.2.x tag contains that tree, so no tag moves.

After the tag: cut `release/v0.2`, run the non-publishing dispatch from it, and activate the ruleset (`release-v02-branch-dry-run-and-ruleset`, needs-user).

## Carried forward

The high-priority items are:
- `majorana-in-decay-chain-refused` (38 CMS cards);
- `uniform-unit-weight-cms-samples` (v0.3);
- `committed-build-output-in-history`.

The medium items include:
- `complex-mass-scheme-refused` (now 14 cards, with the on/off measurement);
- `half-percent-residual-after-hard-survey-134-967`;
- `photon-isolation-ptgmin-unimplemented`;
- `vbf-heavy-resonance-low-at-campaign-budget`;
- `darkphoton-mzd40-seeds-confidently-low`;
- `standalone-harness-cannot-compare-squared-order-split`;
- `ssww-enumerate-oom-in-teardown`;
- `gpdb-compares-sigma-after-physics-changing-patches`.

The four MLM sampling-cost items ship on the v0.3 line only. In it, the
one-group-per-point evaluation lands before lane batching.

## Lessons

- **An artifact that embeds its model cannot see a reader fix.** #178 fixed
  the `GoldstoneBoson` spelling without a version bump. Until #211 bumped
  `ENUMERATION_VERSION` to 4, HEAD read v0.2.2's Bprime artifact silently at
  17.70 pb against 2.527 fresh, and main-built artifacts from between the two
  carried the same exposure. This is sprint K's #161 lesson again, one layer
  down (`artifact-version-blind-to-ufo-reader-changes`).

- **Name the build of every campaign row.** PR 1's request A ran on a
  branch build that lacked a fix already on main. A wrong-sign interference
  then read as a new bug for a day. Checking `merge-base --is-ancestor` against
  the fixes that touch the model's vertex classes would have settled it at once.
- **Self-consistency is not an oracle.** "The order classes partition |M|², so
  only the interference is wrong" checked vibegraph against itself. Against
  MadGraph, |M2|² was wrong too. Go to the per-diagram comparison first.
- **A patch that removes a physics command makes the row incomparable.** The
  silent `strip:set` of `complex_mass_scheme` produced 14 cards of ratios that
  read as physics. It also confounded the earlier "3–40% low" attribution on the
  pair-production cards.
- **Five seeds cannot calibrate a rung.** Card 614's 5-seed 4× spread was 1.8×
  too small, and it manufactured a dip. Card 878's 5-seed −3.5σ was the same
  effect.
- **A lane that logs "done" without its output is a silent failure.** All 10
  MadGraph np1/np2 runs failed on a path while the lane logged "done". The step
  now stops when there is no cross section.
- **A 2,889-file build tree passed review.** PR 8's agent staged
  `vibegraph-cli/target/` (about 917 MB), and the PR 8 hygiene pass did not flag
  it. `ci-does-not-refuse-tracked-ignored-files` covers the CI side.
- **A fixed-energy scan is not a σ oracle.** L-D1's first RAMBO scan at fixed
  √ŝ put the CP-odd defect at 0.99–1.09 of \|M\|². That scan never reached the
  collinear b̄ region or the high-pT tail, where σ lives, and the defect was the
  whole 1.66×. A one-subprocess σ pair, with and without the suspect
  coefficients, settled it in minutes.
- **A rule signed off on one model can break another.** The planner-signed
  self-conjugate rule closed the dim6top rows and broke five enforced SMEFTsim
  rows. The bit-identity requirement on every other table is what caught it.
- **Unique session IDs.** L-X1 was used twice, and #204 renamed the second to
  L-X3. Draw IDs from the sessions directory, not from memory.
