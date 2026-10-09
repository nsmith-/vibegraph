---
type: Backlog Item
title: Validation and test doc comments contradict the tests they describe
description: "Test and gate docs still say validate_sigma skips without output/, decay-chain events are unwritten, gg→gg has per-diagram channels, the mirror shares one scale, and link deleted tests."
area: hygiene
state: open
priority: low
closes_when: "Every site listed in the body describes what its test or gate now does, and cargo doc on the test targets reports no broken intra-doc link."
blocked_by: []
opened: 2026-10-09
tags: [stale-comment, tests, validation, hygiene-sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: sigma, resource: "../../../../vibegraph-lib/tests/validate_sigma.rs", title: "tests/validate_sigma.rs"}
---
Each site was checked against the code on 2026-10-09 (paths under `vibegraph-lib/` unless stated):

- `tests/validate_sigma.rs:108-109`: runs "only when the gitignored MadGraph `output/` tree is present … otherwise every process is skipped". The gate calls `vibegraph::validation::require` on a missing banked run, and the banked layer takes no runtime skip.
- `tests/validate_sigma.rs:205-209` (`SCALE_FALLBACK_ROWS`): "17 s at its gate budget against 16 s at a twentieth of it". The Plan comment (~:578-581) gives 17.3 s at 40000×6 and 15.6 s at 4000×2, a thirtieth.
- `tests/validate_pdf_grid.rs:404-408`: multigrid coverage on "MSHT20lo_as130 … a three-band set". The oracle is NNPDF31_lo_as_0130 with two subgrids, as the module doc (~:8-13) says.
- `tests/decay_chain_ladder.rs:6-7`: "no file is written: a decay-chain event file waits on its status-2 resonance records". `generate` writes them now (`lhef/resonance.rs`; `vibegraph-cli/src/generate.rs` ~:897).
- `tests/diagram_channel.rs:1224`: intra-doc link to `a_massless_spacelike_pole_puts_the_transfer_edge_on_rounding_noise`, which no longer exists.
- `vibegraph-cli/tests/cli_decay_chain_events.rs:1090-1094`: the mirrored ordering "takes the same scale as the direct one by construction on both sides". `per_group_sum` draws the mirror's configuration at its own argument (`proton.rs` ~:2191-2207).
- `src/hadronic.rs:3626-3639` (`a_sampled_channel_names_the_integration_channel_of_its_own_diagram`): "four diagrams, four sampling channels, three integration channels … sampler channel 2 is clustered in integration channel 2, not 3". The test asserts three channels, one per MadGraph configuration.
- `src/coupling/cluster/configs.rs:562-571` (`the_channel_to_config_map_is_not_the_identity`): still describes a per-diagram sampler.
- `src/proton.rs:3619-3622` (the `mirror_visibility_floor` ladder probe): "at sample sizes and stream seeds the gate does not use". Its `(32, 0x0FF5_E7ED)` row is the gate's own draw (~:3516-3517), and the neighbouring doc (~:3443) says "from 36 draws to 512" where the ladder uses 32.

Found by drafters D3, D10 and D11 and verifiers V4, V7, V10 and V11.
