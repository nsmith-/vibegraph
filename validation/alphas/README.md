# Running `alpha_s` reference

`reference.csv` is the output of MadGraph's own `ALPHAS` routine, evaluated over a
grid of `(asmz, nloop, Q)` and committed so the bit-for-bit comparison in
`vibegraph-lib/tests/validate_alphas.rs` runs in the default test suite.

## Regenerating

```bash
pixi run -e madgraph generate-alphas-reference
```

`gen_reference.sh` compiles `$CONDA_PREFIX/MG5_aMC/Template/LO/Source/alfas_functions.f`
— the unmodified template source, not a copy — against `driver.f`, which sets the
`/a_block/` common block and calls `ALPHAS`. Nothing about the algorithm is
restated here; if MadGraph is bumped, this task regenerates against whatever the
new template contains and the Rust comparison reports the difference.

## Why the flags matter

`-ffp-contract=off` stops the compiler fusing `a*b + c` into an FMA. Rust emits no
such contraction, so without it the two sides differ in the low bits of every
Newton iterate for reasons unrelated to the algorithm, and a bit-for-bit gate
becomes impossible to hold.

## Grid

`driver.f` covers `nloop = 1, 2, 3`, four `alpha_s(M_Z)` values (the two parameter
card settings the banked runs use, the value MadGraph's `G -> asmz` round trip
actually produces for a card holding `0.130`, and the `nn23lo` table entry), and
66 scales: both sides of each flavour threshold at `1e-7` relative separation,
both sides of `M_Z`, the scales the banked runs report, and a logarithmic sweep
from 1 GeV to 14 TeV. `reference_grid_straddles_every_branch` asserts that
coverage from the Rust side, so trimming the grid fails the test rather than
quietly shrinking the net.
