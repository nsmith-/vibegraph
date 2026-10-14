---
type: Session Brief
title: "F-G1: amplitude, model and diagram test fixes"
description: "Fix R-G1's 14 triaged findings in the amplitude/colour/model test targets, close config-amp-phase-and-sign-unpinned, and move the smeftsim and toy_models targets into the hermetic layer."
status: draft
agent: validation-dev (Opus)
depends_on: [triage]
closes: [config-amp-phase-and-sign-unpinned, smeftsim-vendored-checksum-not-hermetic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [fix protocol](fix-protocol.md). The findings are listed in
[triage.md](../triage.md), "Fix here, by session"; read each in its review
report.

**Findings:** R-G1.1, .2, .3, .4, .6, .7, .9, .10, .11, .12, .14 (docs and non-empty guards only), .23, .25, .26 (`sessions/R-G1-report.md`).

**Claimed items closed here:** config-amp-phase-and-sign-unpinned, smeftsim-vendored-checksum-not-hermetic.

**Notes:**
- **smeftsim-vendored-checksum-not-hermetic** is widened by the user's decision. Move the whole `smeftsim` and `toy_models` targets into the hermetic layer: drop their `required-features`, and update `validation/manifest.toml`'s layer for each test they own. Confirm every input is tracked (`git ls-files`) and that both run under a plain `cargo test`.
- **config-amp-phase-and-sign-unpinned:** its sign-pattern table is extracted from the banked set, which is present. Both asserts must fail when deliberately broken (the item says so).
- **R-G1.1:** a missing `output/` must now fail through `vibegraph::validation::require`. Show that run.
- **Banked gates:** `amplitude_oracle`, `coupling_oracle`, `color_cf_oracle`, `color_flow_tags_oracle` and `standalone_jamps`.
