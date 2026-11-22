---
type: Session Report
title: "L-M1 report: Majorana fermions and fermion-number violation against MadGraph per diagram"
description: "Fermion lines without a fermion-number arrow are oriented by one end's charge and read with each vertex's own orientation; per-diagram, per-helicity AMP() oracles on the toy Majorana and diquark models and on MSSM neutralinos, the refusals that stay, and what the oracles cannot see."
status: draft
tags: [majorana, fermion-flow, fermion-number-violation, madgraph-oracle, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
DRAFT — filled in as the session measures.

## MadGraph's conventions, from source

Pinned checkout `research/refs/mg5amcnlo` at b7687064 (3.7.1 development line;
the standalone tables below were generated with it through `mg5_pinned.sh`).

1. **External Majorana wavefunctions start as particles.** An external
   wavefunction's `state` and `is_part` come from its leg; a self-conjugate
   fermion is `is_part` (positive PDG) — `HelasWavefunction.set`, `particle`
   branch (`madgraph/core/helas_objects.py:915-922`). Its HELAS call takes
   `state_id = -(-1)**get_with_flow('is_part')` (`:1508-1509`), so an initial
   Majorana is `IXXXXX(nsf=+1)` and a final one `OXXXXX(nsf=+1)` until a clash
   flips it.
2. **A clash flips the flow back to the outermost Majorana.** When two fermion
   mothers of an amplitude or wavefunction have the same `get_with_flow('state')`
   (`HelasWavefunctionList.check_and_fix_fermion_flow`, `:2301-2402`), the line is
   followed toward its external leg (`check_majorana_and_flip_flow`, `:1207-1404`):
   wavefunctions before the outermost Majorana swap `state` and `is_part` (the
   Majorana itself becomes `IXXXXX(nsf=-1)`/`OXXXXX(nsf=-1)`, the conjugate
   spinor), those after it flip `fermionflow`. With no Majorana on the line (a
   Dirac fermion-number-violating vertex) it retries with `force_flip_flow = True`
   (`:2393`) and flips `fermionflow` of the first clashing mother down to its
   external leg.
3. **Conjugate routines where the flow runs against the vertex.**
   `get_conjugate_index` (`:2144-2176` for a wavefunction, `:3152-3173` for an
   amplitude) tags fermion pair `k` of the vertex with `C<k>` when either fermion of
   the pair has `fermionflow < 0`, or (`majorana_conjugates`, `:2468-2500`) the pair
   is two *different* Majoranas in the wrong in/out order. A pair of a Majorana
   and a Dirac fermion of positive flow, and a pair of two identical Majoranas,
   are never conjugated.
4. **ALOHA's conjugate routine is `C Γᵀ C⁻¹`.** `apply_conjugation`
   (`aloha/create_aloha.py:196-230`) multiplies the kernel by
   `C(new, old+1) · Γ · C(new+1, old)`, documented there as `C Γᵀ C⁻¹`, and
   swaps the pair's in/out roles. It refuses a pair beyond the first of a
   structure whose fermion lines are not `(1,2)(3,4)…`: "Unable to deal with
   4(or more) point interactions in presence of majorana particle/flow
   violation" (`:206-214`).
5. **The fermion factor is the parity of the external numbers along the lines.**
   `calculate_fermionfactor` (`:2907-3026`) lists, per pair of fermion mothers,
   the external number reached by each (`get_fermion_order`, `:1419-1466`), the
   pair in the order its mothers are sorted for the routine call (ket first), and
   takes `sign_flips_to_order` (`:3028-3043`), the parity of the bubble sort.
6. **Octet Majoranas** (gluinos) get one more `−1` on the coupling of an
   `FVI`-type wavefunction with two octet mothers
   (`set_octet_majorana_coupling_sign`, `:1106-1121`).
