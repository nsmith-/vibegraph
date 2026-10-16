---
type: Backlog Item
title: "MadGraph output parsers exist in several copies"
description: "Fortran D exponents, the MGRunCard reader, the results.dat first line and symfact.dat multiplicities are each parsed in two to five places that must agree."
area: hygiene
state: open
priority: low
closes_when: "The Rust copies share one parser module (a tests/common file the seed-record writer includes by path), and each remaining shell copy is pinned to the same rule by a test."
blocked_by: []
opened: 2026-10-16
tags: [hygiene, madgraph, parsers, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
From [HY1](../../sprints/sigma-reference-scatter/sessions/HY1-report.md) (H29):

- Fortran `D` exponents: `seed-record` `fortran_float` and
  `vibegraph-lib/src/lhef/parse.rs:136`.
- `<MGRunCard>` reader: `seed-record` `banner_value` and
  `vibegraph-cli/tests/cli_generate_proton.rs:1143` `mg_run_card_fields`.
- `results.dat` first line: `seed-record` `value_and_error`,
  `validate_hadronic.rs:217`, `madevent_seeds.sh` (`mes_run_seed`,
  `mes_check_run`) and `gen_pta_windows.sh` `run_one`.
- `symfact.dat` positive multiplicity: `seed-record` `parse_symfact` and
  `bank_seeded_sigma.sh` `keep_run`; the two must agree on which channels are
  summed.
