---
type: Backlog Item
title: VIBEGRAPH_NO_NETWORK means different things to the CLI and the fetch scripts
description: "The CLI refuses downloads when VIBEGRAPH_NO_NETWORK is set to any value; validation/fetch_common.sh refuses only on the literal 1, so =true stops the binary but not the fetch scripts."
area: hygiene
state: open
priority: low
closes_when: "The CLI and fetch_common.sh read VIBEGRAPH_NO_NETWORK by one rule, and the rule is stated where both document it."
blocked_by: []
opened: 2026-10-09
tags: [network-policy, tooling, consistency]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: cli, resource: "../../../../vibegraph-cli/src/network.rs", title: "vibegraph-cli/src/network.rs ~:99, var_os(NO_NETWORK_VAR).is_some()"}
  - {id: sh, resource: "../../../../validation/fetch_common.sh", title: "validation/fetch_common.sh vg_consent (~:74)"}
---
`NetworkPolicy::from_env` (`vibegraph-cli/src/network.rs` ~:99) refuses
whenever `VIBEGRAPH_NO_NETWORK` is present, whatever its value.
`vg_consent` (`validation/fetch_common.sh` ~:74) refuses only when it equals
`1`, and its header documents `=1`. Under `VIBEGRAPH_NO_NETWORK=true` (or `=0`,
which a user might expect to mean "allow") the binary refuses and the fetch
scripts download. Pick one rule. "Set at all" is the stricter, fail-closed
choice. Found by Phase 2 drafter D15.
