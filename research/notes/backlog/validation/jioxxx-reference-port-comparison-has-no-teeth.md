---
type: Backlog Item
title: test_eval_jioxxx cannot see the longitudinal-term denominator
description: "jioxxx divides the massive-vector longitudinal term by complex m²−imΓ, the production kernel by real m² as ALOHA does; massless external fermions make q·J = 0, so the test cannot tell."
area: validation
state: open
priority: low
closes_when: "test_eval_jioxxx includes a case with q·J ≠ 0 (massive external fermions or a non-conserved current), and the reference port and kernel agree there or the difference is documented as HELAS vs ALOHA."
blocked_by: []
opened: 2026-10-09
tags: [helas, reference-port, oracle-blind-spot, propagator]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: vertex, resource: "../../../../vibegraph-lib/src/helas/vertex.rs", title: "helas/vertex.rs jioxxx (~:118-126)"}
  - {id: kernel, resource: "../../../../vibegraph-lib/src/helas/eval/kernel.rs", title: "helas/eval/kernel.rs propagate_vector_bare (~:90-104)"}
  - {id: test, resource: "../../../../vibegraph-lib/src/helas/eval/run.rs", title: "helas/eval/run.rs test_eval_jioxxx (~:2149, ~:2304-2325)"}
---
The HELAS reference port `jioxxx` (`vibegraph-lib/src/helas/vertex.rs`
~:118-126) forms the unitary-gauge longitudinal subtraction as
`(J·q)/(m² − imΓ)`. The production kernel `propagate_vector_bare`
(`helas/eval/kernel.rs` ~:90-104) divides by real `m²`, as ALOHA's
`OM3 = 1/M3**2` does. `test_eval_jioxxx` (`helas/eval/run.rs` ~:2149) compares
the two with massless external fermions, where `q·J = 0` and the term vanishes.
So the comparison is blind to exactly the place they differ.

Either add a case where `q·J ≠ 0`, or align the port with ALOHA and say so. The
production kernel is the one the MadGraph amplitude gates pin. This is
oracle-quality work, not a physics bug. Found by Phase 2 drafter D9.
