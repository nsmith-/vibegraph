---
type: Backlog Item
title: Run-card Opaque defaults differ from MadGraph's banner.py
description: "Three Opaque run-card fields store an empty default where banner.py has a value, so a card writing MadGraph's own default reads as an override."
area: hygiene
state: open
priority: medium
closes_when: The Opaque default payloads match banner.py under parse_value normalisation, defaults_match_banner_py_dump compares Opaque payloads, and the known-mismatch set pinned by opaque_defaults_known_to_differ_from_banner_py is empty.
blocked_by: []
opened: 2026-08-02
tags: [runcard, madgraph-parity, oracle-blind-spot]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L525-L531", title: "TODO.md entry T040"}
---
`defaults_match_banner_py_dump` (`vibegraph-lib/src/runcard.rs` ~:1076) checks
the run-card defaults table against `banner.py`, but its `Def::O` arm compares
nothing (~:1123, with `Def::O => ParamValue::Opaque(String::new())` at ~:120).
Three fields store an empty default where MadGraph has a real one:

- `mxx_only_part_antipart` (`{'default': False}`)
- `pdgs_for_merging_cut` (`[21, 1, 2, 3, 4, 5, 6]`)
- `systematics_arguments`

`mxx_only_part_antipart` matters today: 35 banked cards write MadGraph's own
default for it, and our code reads that as an override. `me_frame` was the
fourth. It now stores `Def::L("1, 2")` (~:653) and is `Consumed` through
`RunCard::frame_id` (checked 2026-10-09).

Nothing goes wrong yet, because the three are classified `IgnoredBenign` for
reasons that do not depend on the default.
`opaque_defaults_known_to_differ_from_banner_py`
(`vibegraph-lib/src/runcard/classes.rs` ~:591) pins exactly these three names,
so a MadGraph bump that changes the set fails loudly. The fix is to normalise
Python reprs into the table's payloads. It should land before any enforcement
covers these fields. Detail: [note 29 §C2.5](../../29-v01-validation-sprint-plan.md);
`TODO.md` mis-cites this as note 28.
