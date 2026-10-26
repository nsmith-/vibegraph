---
type: Design Decision
title: "Licensing: the interned SM model and banked MadGraph outputs"
description: "vibegraph is MIT OR Apache-2.0; binaries embed MG5's models/sm, so --version prints THIRD-PARTY-NOTICES pinned to the source commit; banked MadGraph outputs owe no MG source notice."
status: stable
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}, {by: "human:nsmith-", at: 2026-10-09}]
tags: [licensing, release, madgraph, distribution]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-u1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2098-L2352", title: "Note 24 §U1 outcome, third-party notice follow-up"}
  - {id: n25-backlog, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L453-L500", title: "Note 25 §7: licensing closed by user decision 2026-07-31"}
  - {id: notices, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/THIRD-PARTY-NOTICES", title: "THIRD-PARTY-NOTICES"}
  - {id: mg-license, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/LICENSE", title: "MadGraph5_aMC@NLO LICENSE at the pinned commit"}
---
# Licensing: the interned SM model and banked MadGraph outputs

## What vibegraph is licensed under

The crates are dual-licensed `MIT OR Apache-2.0` (`license` in
`vibegraph-lib/Cargo.toml`, `vibegraph-cli/Cargo.toml` and `validation-report/Cargo.toml`; texts in
`LICENSE-MIT` and `LICENSE-APACHE` at the root), settled by the user on
2026-07-31.[^n25-backlog]

## What the binary redistributes, and the notice it owes

Every binary embeds material derived from MadGraph5_aMC@NLO's `models/sm`,
committed at `vibegraph-lib/src/ufo/sm_assets/` and compiled in with
`include_str!`/`include_bytes!` (`vibegraph-lib/src/ufo/sm.rs`):

- the nine `restrict_*.dat` cards, byte for byte;
- `restrict_ckm_no_b_mass.dat`, byte for byte from the `models/sm` that CMS
  genproductions bundles in its gridpacks (gridpack `041_WWjj_SS_ewk`; sha256
  `ad6e3eaf03443bc14148d392fe98deb712de29166061a7408eff521a47591b05`, pinned by
  `vendored_card_is_the_cms_genproductions_bytes`). MadGraph does not ship it;
  it is `restrict_ckm.dat` with `MB`, `ymb` and the tau width changed, so its
  text is MadGraph's param-card writer's output with values edited by CMS.
  It needs no licence of its own beyond MadGraph's (user decision,
  2026-10-20);
- `sm_parsed.bin.zst`, a serialised parse of the rest of `models/sm`
  (`particles.py`, `parameters.py`, `vertices.py`, `lorentz.py`,
  `couplings.py` and the other `.py` sources), produced by the `gen_sm_blob`
  dev binary. It is a derivative of the UFO model definition.

It also embeds MadGraph's `input/particles_name_default.txt`, the default
particle names by PDG code that `import model` applies, byte for byte at
`vibegraph-lib/src/ufo/mg_input/` (`include_str!` in
`vibegraph-lib/src/ufo/names.rs`, whose test pins its SHA-256). The file's
header puts it under the same MadGraph5_aMC@NLO licence, so
`THIRD-PARTY-NOTICES` lists it in a section of its own, at the same commit.

`models/sm` has no licence of its own; it is covered by the mg5amcnlo root
`LICENSE`, an NCSA-style licence (Copyright (c) 2009, 2013, the MadTeam).
Clause 2 conditions redistribution in binary form on reproducing the
copyright notice, the conditions and the disclaimer in "the documentation or
other materials provided with the distribution".[^mg-license] It does not
require runtime display.[^n24-u1]

`THIRD-PARTY-NOTICES` at the repository root states what is redistributed,
names the source (`https://github.com/mg5amcnlo/mg5amcnlo`, `models/sm`, at
commit `b7687064b9a013317ca164aa1395bc9c0e39ae1e`, the commit the assets were
last regenerated from), and reproduces the licence in full. **Keep that commit
reference in step with the assets**: rerunning `gen_sm_blob` from a newer
submodule checkout means updating the pointer in the file.

How the notice travels with the binary: release assets are bare executables,
not archives, so the binary is the only vehicle that reliably accompanies
every copy. `vibegraph --version` prints the version line, then
`THIRD-PARTY-NOTICES`, `LICENSE-MIT` and `LICENSE-APACHE`, all compiled in
(`LONG_VERSION` in `vibegraph-cli/src/main.rs`; `-V` prints the version line
only). `long_version_carries_every_required_notice` checks the text; the
release smoke test and `scripts/acceptance.sh` grep the built binary's
`--version` for the MadTeam copyright line. The release also attaches the
three files beside the binaries. Details in
[release binaries](release-binaries.md).

## What is not owed: banked outputs

**Decision (user, 2026-07-31):** banked reference artifacts (MadGraph process
directories, event files, run cards and logs, the `refdata-N` bundle) are
*outputs* of the MadGraph program, not redistributed MadGraph source, so they
owe no MadGraph source-form notice.[^n25-backlog] The
[refdata bundle](../validation/refdata-bundle.md) inherits this position. The
same reading covers MadGraph-written files committed under `validation/`,
such as `validation/madgraph/dy13_param_card.dat` ("PARAM_CARD AUTOMATICALLY
GENERATED BY MG5").

No MadGraph source is committed as a test fixture either: the run-card parser
fixture, `vibegraph-lib/tests/data/run_card_parser_fixture.dat`, states in its
header that it is hand-written in MadGraph's run-card syntax, "not a copy of
any MadGraph file", so clause 1 (source-form redistribution) is not engaged.

## Not covered here

- The PDF set the CLI fetches is not redistributed at all; why, and the
  unchecked premise behind it, is [pdf-set-distribution](pdf-set-distribution.md).
- `assets/badger.png` was generated by the user with Google's Nano Banana
  image generator (user, PR #18 review, 2026-10-09). It is not compiled into
  the binary or packaged by `release.yml` (it exists for the README), so it
  carries no binary-redistribution obligation.

[^n24-u1]: Note 24 §U1 outcome, "Follow-up: third-party notice for the interned MG5 SM model".
[^n25-backlog]: Note 25 §7, "Closed by user decision (2026-07-31)".
[^mg-license]: MadGraph5_aMC@NLO `LICENSE`, clause 2, at `b7687064`.
