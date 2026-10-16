---
type: Session Report
title: "F2 report: wpwm_to_wpwmz_cw gates, with only its rounded-coupling graphs' modulus exempted"
description: "The six O_W rows that carry GC_303/GC_299 now bank AMP() and gate per MadGraph graph at 1e-12. Only the graphs carrying the seven-digit couplings get an exemption, by name: their modulus is compared at the registered deviation (−1.197023e-8, −2.769559e-8), and their phase and every other graph gate at 1e-12. The MadEvent amplitudes cell of wpwm_to_wpwmz_cw flips to gate at |M|² 9.6e-15. Its configuration partition, compared through a pairing measured on the standalone table, is MadGraph's. The approved exemption needed a per-graph own-scale check: the table-normalised 1e-12 gate cannot see eight of the twelve GC_303 graphs left out of the list."
status: draft
tags: [amplitude, smeft, five-vector, vvs, config-groups, registered-deviation, standalone, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
sources:
  - {id: gate, resource: "https://github.com/nsmith-/vibegraph/commit/fe3043f", title: "validation: gate wpwm_to_wpwmz_cw per graph, exempting only the rounded-coupling graphs' modulus"}
---
Items: `wpwmz-cw-ow-five-vector-residual`, `wpwmz-cw-config-partition-uncompared`.
Built on `5d4a346`. Everything is in one commit, `fe3043f`. Nothing under
`vibegraph-lib/src` changed.

## Corrections to the brief

- **"Exempt only the modulus" needs a second check to be exact.** The brief's
  falsifier says dropping one graph from the `GC_303` list fails. Under the
  1e-12 per-graph gate alone it does not for 8 of the 12 graphs, because that
  gate is normalised to the largest graph of the table. The worst graph then
  moves only to 1.0e-14–1.2e-13, and the worst flow to ≤3.3e-13 (output below).
  I added a per-graph check normalised to each graph's own size,
  `GRAPH_RATIO_TOL` = 1e-10. Measured rounding is ≤1.6e-13 over the 1174 graphs
  of the six rows, and the smallest deviation is 1.2e-8. Every other check still
  gates at 1e-12.
- **The MadEvent row needs the exemption too.** The restated item wants that cell
  gated. Its table banks no `AMP()`, so it cannot be corrected per graph from
  MadGraph's side. `amplitude_oracle` therefore names the same graphs, maps them
  through the measured pairing to our diagrams, and divides this side's share of
  them by `1 + deviation` before it compares `|M|²`, the flows and `JAMP2`. A row
  that also banks `AMP()`, or has several flows, is refused, not compared
  uncorrected.
- **The brief says "`GC_303` (and `GC_299`) graphs"; there are more rows than
  `wpwm_to_wpwmz_cw`.** All six of B′'s `SEVEN_DIGIT_COUPLING_ROWS` now have
  per-graph tables and named lists. The list is gone.
- **`smeftsim.rs`'s `GATED_ROWS` had to take the row.** It is a two-way list of
  the manifest's gated SMEFTsim rows. Its op census needed no allowlist change.
  The brief did not name it.

## What changed

- `validation/madgraph/gen_standalone_jamps.py`: the six rows set `bank_amps`.
  Two new fields keep the tables small: `amp_points`, which banks `AMP()` on the
  first N points only, and `amp_helicity_stride`. JAMPs and |M|² still go out at
  every point and helicity.
  - Regenerated through `mg5_pinned.sh`; all six logs read `VERSION 3.7.1`.
  - Momenta, |M|², JAMPs and param card are byte-identical to the committed
    tables on every row (scripted comparison, all `True`).
  - Sizes: `wpwm_to_wpwmz_cw` 2.5 MB (1 point), `wpwm_to_wpwma_cw` 1.7 MB
    (1 point), `ee_to_wpwmwpwm_cw` 2.1 MB (1 point, every third NHEL entry;
    6.1 MB with all of them). The stride-3 subset still has a non-zero entry on
    all 688 graphs, which the test asserts.
- `vibegraph-lib/tests/common/mg_graphs.rs` (new), shared by both suites:
  - `WPWM_TO_WPWMZ_CW_GRAPHS`, the pairing;
  - `ROUNDED_COUPLING_GRAPHS`, per row, each coupling with its measured
    deviation and its graphs by `AMP()` index.
- `standalone_jamps.rs`:
  - per-graph pairings for the six rows;
  - the named-graph modulus factors, applied to the graph entries and to the
    flow;
  - `GRAPH_RATIO_TOL`;
  - `SEVEN_DIGIT_COUPLING_ROWS` and `row_tolerance` removed;
  - three tests: `rounded_coupling_rows_have_tables`,
    `wpwm_to_wpwmz_cw_sees_a_swapped_pairing` and
    `wpwm_to_wpwmz_cw_sees_a_wrong_rounded_list`.
- `amplitude_oracle.rs`: the `wpwm_to_wpwmz_cw` entry in `MG_DIAGRAM_ORDER`.
  `KNOWN_CONFIG_PAIRING_UNAVAILABLE` is empty and keeps its two-way mechanism.
  `Rounded` does the correction. It also cross-checks each registered deviation
  against the row's coupling table, where one exists: Python over Fortran
  within 1e-13.
- `smeftsim.rs`: `GATED_ROWS` gains the row.
- `validation/manifest.toml`: the `wpwm_to_wpwmz_cw` amplitudes cell goes from
  `info` to `gate`, with the `GC_303` cause in its note.

## The pairing, derived from MadGraph's values

The rule is F1's:
1. Each diagram goes to the graphs whose `c_k·AMP_k` has overlap > 1 − 1e-10 with
   it over every banked entry.
2. Ties go to the unique graph whose ratio has unit modulus.

Scratch scripts: `F2-pairing.py`, and a temporary dump probe that was deleted
before the commit.

| row | graphs | entries | ties resolved by modulus | bijective |
|---|---|---|---|---|
| `wpwm_to_az_cw` | 14 | 162 | 0 | yes |
| `az_to_wpwm_cw` | 14 | 162 | 0 | yes |
| `aa_to_wpwm_cw` | 14 | 108 | 0 | yes |
| `wpwm_to_wpwma_cw` | 222 | 162 | 180 | yes |
| `wpwm_to_wpwmz_cw` | 222 | 243 | 152 | yes |
| `ee_to_wpwmwpwm_cw` | 688 | 108 | 496 | yes |

On `wpwm_to_wpwmz_cw`, the one-point derivation is identical to F1's four-point
order (`identical to F1: True`).

The standalone and MadEvent numberings agree. I extracted the amplitude-closing
`CALL` lines of the standalone `matrix.f` and of the MadEvent
`P1_wpwm_wpwmz/matrix1_orig.f`: 222 lines each, and `diff` printed `IDENTICAL`.

## The exempted graphs per row, with measured deviations

Each list is the same set by two independent routes:
- the graphs whose per-graph modulus reads the deviation;
- the graphs whose wavefunction chain in MadGraph's `matrix.f` passes through a
  HELAS call with the coupling (scratch `F2-carriers.py`, run on the standalone
  `matrix.f` and on the MadEvent `matrix1_orig.f`).

The two routes agree on every row. Values are `|this side| / |MadGraph| − 1`,
printed by `cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture`.
`AMP()` indices are 1-based.

| row | coupling | graphs (`AMP`) | measured |
|---|---|---|---|
| `wpwm_to_az_cw` | `GC_303` | 4 | −1.197024002e-8 |
| `az_to_wpwm_cw` | `GC_303` | 4 | −1.197023269e-8 |
| `aa_to_wpwm_cw` | `GC_299` | 4 | −2.769558349e-8 |
| `wpwm_to_wpwmz_cw` | `GC_303` | 3–5, 9–11, 59–61, 65–67 | −1.197023791e-8 … −1.197023380e-8 |
| `wpwm_to_wpwma_cw` | `GC_303` | 6–8, 12–14, 62–64, 68–70 | −1.197023114e-8 … −1.197022892e-8 |
| `wpwm_to_wpwma_cw` | `GC_299` | 3–5, 9–11, 59–61, 65–67 | −2.769558705e-8 … −2.769558505e-8 |
| `ee_to_wpwmwpwm_cw` | `GC_303` | 92–94, 98–103, 107–109, 204–206, 210–215, 219–221 (24) | −1.197023802e-8 … −1.197023169e-8 |
| `ee_to_wpwmwpwm_cw` | `GC_299` | 89–91, 95–97, 201–203, 207–209 (12) | −2.769558960e-8 … −2.769558816e-8 |

- Registered values: `GC_303` −1.197023e-8 and `GC_299` −2.769559e-8.
- `wpwm_to_wpwmz_cw`'s coupling table gives `|python|/|fortran| − 1` =
  −1.1970231e-8 for `GC_303`. `amplitude_oracle` checks that within 1e-13.
- Every other graph reads |ratio| − 1 ≤ 1.2e-13 (`F2-pairing.py`, before any
  correction).

## Gates

Every cargo call ran with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

- `cargo fmt --all --check`: `FMT EXIT 0`.
- `cargo clippy --workspace --all-targets -- -D warnings`: `CLIPPY EXIT 0`.
- `cargo test --workspace --no-fail-fast`, first run: 38 suites, 1433 passed,
  **1 failed**, 16 ignored. The failure was
  `smeftsim::gated_rows_are_the_manifest_s_gated_smeftsim_rows`: `listed` lacked
  `wpwm_to_wpwmz_cw` against `declared`. Fixed by adding the row to `GATED_ROWS`.
  Then `cargo test -p vibegraph-lib --test smeftsim`: `test result: ok. 13 passed`,
  including `gated_row_diagram_counts_match_madgraph` and `gated_rows_op_census`
  on the new row. No other file changed between the two runs. The workspace
  suite was not rerun in full; the validate run below re-ran the full suite.
- `standalone_jamps`: `test result: ok. 49 passed; 0 failed`. The six rows,
  each at 1e-12:

  | row | worst flow | worst \|M\|² | worst graph | own scale | \|G\| − 1 |
  |---|---|---|---|---|---|
  | `wpwm_to_az_cw` | 6.627e-15 | 3.789e-15 | 3.535e-15 | 9.659e-15 | 1e-16 |
  | `az_to_wpwm_cw` | 2.690e-14 | 6.712e-15 | 1.922e-15 | 2.442e-15 | 4e-16 |
  | `aa_to_wpwm_cw` | 1.099e-15 | 6.169e-16 | 3.036e-16 | 6.439e-15 | 4e-15 |
  | `wpwm_to_wpwmz_cw` | 1.931e-14 | 1.489e-15 | 1.460e-15 | 6.217e-15 | 7e-16 |
  | `wpwm_to_wpwma_cw` | 2.173e-14 | 1.931e-15 | 1.881e-15 | 5.551e-15 | 2e-15 |
  | `ee_to_wpwmwpwm_cw` | 2.477e-13 | 2.792e-14 | 2.487e-15 | 1.579e-13 | 1e-15 |

  Under B′'s row-wide 1e-7 the same rows read worst flow 8.5e-11 to 6.4e-10.
- `amplitude_oracle`: `test result: ok. 49 passed`. The row's line:

  ```
  [wpwm_to_wpwmz_cw] GC_303 on MadGraph AMP[3, 4, 5, 9, 10, 11, 59, 60, 61, 65, 66, 67] = our diagrams [96, 97, 98, 102, 103, 104, 142, 143, 144, 148, 149, 150], compared at |ours| / |MadGraph| - 1 = -1.197023e-8
  [wpwm_to_wpwmz_cw] 'W+ W- > W+ W- Z NP<=1' NGRAPHS=222 NCOLOR=1: |M|² max_rel grid 9.61e-15 / event 6.19e-15, per-diagram 0.00e0 (not banked), per-flow 3.31e-14, JAMP2 5.28e-15 (G = -1i, |G|-1 = 1.6e-15, 0 helicity combinations pruned); 21 configurations (MadGraph merges some into one accumulator): ...
  ```

  - The partition comparison raised nothing, so the 21 groups equal MadGraph's
    `AMP2` accumulators as sets of graph indices.
  - Our diagrams 96–98 and 142–144 are F1's vector-source `V(a,Z,H)` diagrams;
    102–104 and 148–150 are its Higgs-producing ones.
- `pixi run --skip-deps validate`: `VALIDATE EXIT 0`, 63 suites, 1819 passed,
  0 failed, 84 ignored. Cell line: `64 rows × 4 categories = 256 cells: 203
  measured in the layers this run drove (197 ✅, 6 ⚠️, 13 ⏳, 40 — / uncovered)`.
  `The measured cells are exactly the cells the manifest declares, every gate
  cell passed`. B′ reported 196 ✅ and 7 ⚠️.

## Cells that moved

Every per-cell JSON was diffed against the C2 after-run report (B′'s baseline;
B′ moved only two informational notes), with durations removed:

```
diagrams 56 differ []
amplitudes 48 differ ['ee_to_wpwm_cw.json', 'wpwm_to_wpwmz_cw.json']
integrals 52 differ []
samples 53 differ []
```

- `wpwm_to_wpwmz_cw` amplitudes: `mode info → gate`, `status info → pass`,
  `max_rel_grid 27.94 → 9.61e-15`, `max_rel_event 10.49 → 6.19e-15`,
  `per_flow 0.748 → 3.31e-14`, `jamp2 25.5 → 5.28e-15`.
- `ee_to_wpwm_cw`: only `note` differs, still `info/info`. This is B′'s
  configuration-sign-pattern note, already in B′'s report.

No σ or sample cell moved.

## Falsifiers, with their failing output

Committed in `standalone_jamps`, so they run on every commit:

- **Swapped pairing.** `wpwm_to_wpwmz_cw_sees_a_swapped_pairing` swaps the
  diagrams paired with AMP(15) and AMP(22). These are the photon and Z exchanges
  of one topology, exactly proportional (overlap 1), with a modulus ratio of 3.6.
  ```
  swapped AMP(15)/AMP(22): worst graph 2.301e-3, worst flow 1.931e-14
  ```
  The flow cannot see it; the per-graph gate fails at 2.3e-3.
- **Wrong rounded list.** `wpwm_to_wpwmz_cw_sees_a_wrong_rounded_list` leaves
  each of the twelve `GC_303` graphs out in turn, then names AMP(1), the
  five-vector contact, which does not carry `GC_303`:
  ```
  AMP(3) left out: worst graph 1.213e-13, on its own scale 1.197e-8, worst flow 3.264e-13
  AMP(4) left out: worst graph 1.039e-13, on its own scale 1.197e-8, worst flow 2.790e-13
  AMP(5) left out: worst graph 6.131e-12, on its own scale 1.197e-8, worst flow 1.647e-11
  AMP(9) left out: worst graph 3.708e-14, on its own scale 1.197e-8, worst flow 1.181e-13
  AMP(10) left out: worst graph 3.013e-14, on its own scale 1.197e-8, worst flow 8.962e-14
  AMP(11) left out: worst graph 2.113e-12, on its own scale 1.197e-8, worst flow 5.680e-12
  AMP(59) left out: worst graph 5.878e-14, on its own scale 1.197e-8, worst flow 1.840e-13
  AMP(60) left out: worst graph 5.037e-14, on its own scale 1.197e-8, worst flow 1.576e-13
  AMP(61) left out: worst graph 3.295e-12, on its own scale 1.197e-8, worst flow 1.088e-11
  AMP(65) left out: worst graph 1.191e-14, on its own scale 1.197e-8, worst flow 1.227e-13
  AMP(66) left out: worst graph 1.020e-14, on its own scale 1.197e-8, worst flow 1.052e-13
  AMP(67) left out: worst graph 3.027e-12, on its own scale 1.197e-8, worst flow 8.946e-12
  AMP(1) named: worst graph 3.250e-10, on its own scale 1.197e-8, worst flow 1.002e-9
  ```
  Every mutation fails the own-scale check (1.197e-8 against 1e-10). Eight of
  the left-out graphs (AMP 3, 4, 9, 10, 59, 60, 65, 66) pass the 1e-12 graph and
  flow gates; that is why `GRAPH_RATIO_TOL` exists.

Run by temporary edits to `mg_graphs.rs`, restored afterwards (`cmp`: `RESTORED`).
Command: `cargo test -p vibegraph-lib --test amplitude_oracle -- wpwm_to_wpwmz_cw`.

- **Swapped AMP(15)/AMP(22) in the shared pairing:** the partition fails.
  ```
  [wpwm_to_wpwmz_cw] the integration configurations are not MadGraph's: ours group [..., [14, 22, 23, 24, 25, 26, 27], [15, 16, 17, 18, 19, 20, 21], ...
  test result: FAILED. 0 passed; 1 failed
  ```
- **AMP(5) left out:**
  ```
  [wpwm_to_wpwmz_cw] per-flow JAMPs disagree with MadGraph beyond the per-diagram constant (max element-wise deviation 1.951e-11) ...
  test result: FAILED
  ```
- **No registration for the row:**
  ```
  per-flow JAMPs disagree ... (max element-wise deviation 2.534e-11) ...
  test result: FAILED
  ```
- **AMP(4) left out: passes**, at `|M|² grid 2.88e-13 / event 4.74e-13,
  per-flow 3.32e-13`. That is the MadEvent row's blind spot, and it is stated
  in the module header. The list is pinned per graph by the standalone row.

## Blind spots

- **A uniform error of the registered size on exactly the listed graphs.** A
  modulus error of −1.197e-8 on the twelve `GC_303` graphs on this side is
  absorbed by the exemption. Their phase is still gated.
- **The registered deviation is measured, not derived.** For `GC_303`,
  MadGraph's own coupling table (Python against Fortran) confirms it to 3e-15.
  No table carries `GC_299`, so its value rests on the 25 per-graph measurements
  alone (spread 4e-15).
- **The MadEvent row cannot check its own list.** It reads the standalone row's
  list through a pairing whose numbering identity rests on a textual diff of the
  closing calls, done here and not in a test. A graph left out of the list but
  small against the amplitude passes it (AMP(4) above).
- **Points and helicities.** `AMP()` is banked at one point per row, and at a
  third of the NHEL entries for `ee_to_wpwmwpwm_cw`. The flows still run at
  every banked point and helicity. A graph wrong only in a region or helicity
  the subset misses would be seen only through the flows.
- **Ties.** 152 of `wpwm_to_wpwmz_cw`'s 222 diagrams have more than one
  overlap-1 graph, and all are resolved by modulus. Two graphs proportional *and*
  of equal modulus would be ambiguous. None exist here; the derivation asserts
  that.
- **Global phase and relabelling.** As before, `G` absorbs a sign shared by every
  diagram.

## Found

- **Stale notes on the same manifest row.** The `diagrams`, `integrals` and
  `samples` notes of `wpwm_to_wpwmz_cw` still say the amplitudes "disagree
  grossly" or quote `|M|^2 2.79e1`. Evidence: `validation/manifest.toml`, the row's
  other three cells. To close: rewrite those notes. They were out of this
  session's edit scope.
- **`ee_to_zh_smeft` can gate the same way.** Its amplitudes cell is `info` for
  exactly this `GC_303` cause (|M|² 6.00e-11). Its table banks `AMP()`, so
  `Rounded` refuses it today. To close: correct the per-diagram and
  per-configuration levels with the named factors as well, register its
  `GC_303` diagrams, and flip the cell.
- **`GC_299` has no coupling-oracle row.** The coupling oracle has no row with
  it, so its deviation has no Python/Fortran cross-check. F1 found this too. To
  close: bank a coupling table for a photon–photon–Higgs process (for example
  `aa_to_wpwm_cw` as a MadEvent row), and add it to `KNOWN_FORTRAN_DEVIATIONS`.
- **The standalone/MadEvent numbering identity is not a test.** To close:
  `gen_standalone_jamps.py` banks each graph's closing call and coupling (F1's
  Found, too), and `amplitude_oracle` compares them with the MadEvent row's
  `matrix1_orig.f` lines.
- **The kb's `madgraph-defect-policy` table names the outcome as decided; no kb
  concept records the mechanism.** That covers `ROUNDED_COUPLING_GRAPHS`, the
  own-scale tolerance and the MadEvent-side correction. To close: a paragraph in
  `validation/amplitude-oracle` and `validation/gate-thresholds`.
