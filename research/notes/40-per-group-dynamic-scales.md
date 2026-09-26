# 40 — Dynamic scales per flavour group and per beam ordering (2026-09-26)

**Status: LANDED** on `pg-z1s` (close-out of the `process-grammar` sprint, note 38
§4 E1's open finding (1)).

## 1. The defect

`ProtonIntegrand` sums every flavour group at every phase-space point. Under the kT
clustering (`dynamical_scale_choice = -1`) a point's scale depends on the
integration configuration it is clustered in, and the configurations belong to a
group. The integrand drew one configuration per point — `∝ AMP2_c` inside the group
of the *sampling channel* — clustered in it, and evaluated **every** group's term at
that one scale. The event was then labelled with the group its luminosity-weighted
`|M|²` drew, so its `SCALUP`/`AQCDUP` could be another group's.

MadEvent integrates each subprocess group (each `P` directory, each `IPROC` inside
it) separately: `DSIGPROC` calls `update_scale_coupling` and `setclscales` clusters
with `ipdgcl(·, igraphs(1), iproc)`, the flavours of the subprocess whose matrix
element the point evaluates (`super_auto_dsig_group_v4.inc`, `reweight.f:555`). A
point's scale is never another group's.

Working through the fix exposed a second, older error of the same kind. A group's
mirrored term (the member's partons on the other beams, `|M(Rq)|²`) was evaluated at
the direct term's scale: configuration drawn from `AMP2` at `q`, clustered at the
unrotated lab momenta. MadEvent's `IMIRROR = 2` flips the momenta it hands the
matrix element and clusters the *unflipped* `pp` with the subprocess's own flavour
order — i.e. it clusters the mirrored physical event in the orientation its matrix
element reads it in, and draws the configuration from that matrix element's `AMP2`.
The scale *values* the configurations offer coincide between the two orientations on
`p p → ℓℓj` (the clustering is rotation-invariant there), but the `AMP2` shares that
choose among them do not.

## 2. The fix

`ProtonIntegrand::shape` has two paths:

- **shared** (constant scales, or a closed form that reads no configuration): the
  old code unchanged — one scale, one pair of density rows, every group at it.
  Fixed-scale artifacts and LHE files are byte-identical to `14029ce` (§4).
- **per term** (the clustering): every group draws its own configuration from the
  one trailing uniform, `∝ AMP2_c` of its own matrix element at `q`, clusters at the
  lab momenta, reads its densities at its own `μF` and binds `αs(μR)`; a group with
  a mirrored ordering draws a second configuration from `AMP2` at `Rq` and clusters
  at the lab momenta rotated by π about x with the beams exchanged
  (`mirror_lab_into`), its per-beam `μF` swapped back to physical beam order.
  The draws share the uniform; each term's distribution is the rule's, and σ is
  linear in each term. The sampling channel survives only as the fallback of its
  own group's direct draw.

`ProtonEvent::group_scales` carries `[direct, mirrored]` per group; `select_event`
reads each term at its own scales and densities, and `ProtonSelection::scales` is
the drawn term's, which `generate` writes as `SCALUP`/`AQCDUP`.

Cost (§5): one extra `eval_amp2` per group per point (two for a mirrored group).

## 3. The per-event oracles

1. **Our own events replay in their own group**
   (`cli_decay_chain_events::our_own_events_replay_in_their_own_flavour_group`,
   banked). 2000 generated events per card, each replayed through the prescription
   in every configuration of the group its flavours name (an exchanged event read
   rotated, as MadEvent reads it); one has to return `SCALUP` to 1e-6 and `αs` of its
   `μR` has to be `AQCDUP`.

   | card | before (`14029ce`) | after |
   |---|---|---|
   | `p p > t t~` decayed (dyn) | 1379 / 2000 (all 621 misses are the other group's scale) | 2000 / 2000 |
   | `p p > l+ l- j` (dyn) | 1921 / 2000 (79, all the other group's) | 2000 / 2000 |

   Worst `SCALUP` relative 6.2e-9, `AQCDUP` |Δ| 5.3e-10 (printed at nine digits).
   *Blind to*: which configuration inside the group was drawn (any is accepted), and
   the mirror orientation on these two cards (both orientations offer the same scale
   values).
2. **MadEvent's configuration draw, event by event**
   (`validate_hadronic::madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2`,
   banked). On `pp_to_llj_dyn` every `q g → ℓℓq` event has configurations at two
   scales; the chance of the higher one under the rule is the `AMP2` share of the
   configurations that give it. Over MadEvent's 7197 such events: 1429 at the higher
   scale against 1488.3 expected (pull −1.83). The same exchanged events read
   unrotated — the old mirror convention — expect 1748.8 (pull −9.31), per group
   −5.2 to −6.7; the test asserts that control is rejected. *Blind to*: the
   `q q̄ → ℓℓg` groups (one scale for every configuration) and the integrand's own
   use of the rule, which (3) sees.
3. **`SCALUP` per initial-state class** (diagnostic, not committed): the `samples`
   gate's `SCALUP` KS column is a marginal over classes. Split by `g q` against
   `q q̄` on the gate's own seeds, against MadEvent's 10000 banked events:

   | build | all (min p) | `g q` (p) | `q q̄` (p) |
   |---|---|---|---|
   | `14029ce` | 7.3e-3 | 5e-9 – 4e-13 | 5e-12 – 1e-13 |
   | per group, mirror at `q` | 1.3e-4 (**fails the 1e-4 floor on one seed**) | 1e-4 – 1e-5 | 0.27 – 0.98 |
   | per group and ordering | 0.24 – 0.88 | 0.53 – 0.66 | 0.27 – 0.87 |

   The old marginal passed because two wrong classes compensated: `q q̄` events
   carried the higher `g q` scales and `g q` events the lower `q q̄` ones. Fixing the
   group alone uncovered the mirror error in the `g q` tail; fixing both agrees per
   class.

## 4. Byte identity of fixed-scale output

Old (`14029ce`) and new binaries, `integrate` at `--target-rel 1e-2` and `generate`
2000 events: `pp_to_llj_fixed`, `pp_to_bb_fixed`, `dy13_default`, the decayed
`p p > t t~` at fixed μ (D3's card), the two-`@N` Drell–Yan card, `ee_to_mumu` and
the fixed-beam dynamic `gu_to_epemu`: every `grid.bin.zst` byte-identical, every LHE
identical but for the header line naming the artifact's path.

## 5. σ rows

Twenty seeds per row at each row's enforced budget, run on the final code (both
defects fixed) and on `14029ce` with the same seeds (`probe_dynamic_rows_seed_sweep`,
`VG_SWEEP_SEEDS=20`). "rel" and "pull" are against MadGraph's banked σ; the paired
shift is new − old seed by seed.

| row | before: rel, pull, χ²/dof | after: rel, pull, χ²/dof | paired shift |
|---|---|---|---|
| `pp_to_llj_dyn` | +0.182%, +0.55, 1.06 | +0.005%, +0.01, 1.25 | −0.176% ± 0.008% |
| `pp_to_llj` | +0.087%, +0.26, 0.80 | +0.033%, +0.10, 0.57 | −0.054% ± 0.029% |
| `pp_to_bb_qcd2` | +0.007%, +0.10, 0.65 | +0.009%, +0.13, 0.65 | +0.002% ± 0.001% |
| `pp_to_jj` | +0.154%, +0.70, 1.34 | unchanged | 0 |
| `pp_to_bb` | −0.005%, −0.07, 0.94 | unchanged | 0 |
| `pp_to_ll_scalefact2` | −0.052%, −0.26, 0.46 | unchanged | 0 |

On the gates' own seeds, `pp_to_llj_dyn` reads −0.026% (pull −0.08, five seeds)
and `pp_to_llj` −0.047% (pull −0.13, three seeds). Every row stays inside its
tolerance, and no `rel_tol` or mode changed.

`pp_to_llj_dyn` moves from +0.18% to on top of MadGraph. The row's earlier +0.12%
to +0.18% residual, which its note attributed to the reference's own 0.33% error,
was mostly these two defects. Rows whose groups all cluster to the same scale do not
move: every 2 → 2 row, and `pp_to_ll_scalefact2`.

The full banked layer (`validation/validate.sh`) on the merged tree is exit 0, with
183 ✅ / 7 ⚠️ / 4 ⏳, the same cells as before the fix.

## 6. What is left

- The configuration draw's shares are pinned by (2) on one card; `p p > t t~` and
  `p p > j j` offer too few two-scale events of their own to repeat it.
- A 2 → 2 clusters every group to the same scale up to rounding (`pp_to_jj` moves by
  2e-9 relative), which is why that row is blind to both errors.
