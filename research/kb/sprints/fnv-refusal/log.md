# fnv-refusal log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (a04fc80, 4272a03): the manager re-ran `cli_fermion_flow` (4 passed) from the committed tree and read the `layout.rs` change (a compile-time match on both operands, replacing an unchecked fall-through; not on the evaluation path).
* **Correction to the brief, from F1:** only the FNV case panicked before the fix; both Majorana processes returned a σ with no error (`lt~ lt > nt nt`: 9341 ± 9068 pb, reading out of a neighbouring array). The silent wrong answer, not the panic, was the worse failure.
* **HY1 checked** (1727b8d, ce3cbf0): the manager re-ran `cli_fermion_flow` (6 passed) and fmt from the committed tree.
* **Close-out** (2026-10-20). The manager added a dated note to `majorana-fermions-unsupported` (the possibly over-strict propagator case).
