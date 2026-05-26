# UFO Sample Models

Place UFO model directories here for use as test inputs.

## Integration Tests

The `vibegraph::ufo` module contains integration tests that load real UFO models from
`research/refs/mg5amcnlo/models/` (SM, loop_sm, MSSM_SLHA2, taudecay_UFO). These tests
run automatically with `cargo test`; they skip gracefully if the submodule is not populated
(`git submodule update --init --depth=1`).

## Getting Models

Standard models are available from:
- https://feynrules.irmp.ucl.ac.be/wiki/ModelDatabaseMainPage
- https://github.com/mg5amcnlo/mg5amcnlo (ships SM, QED, etc. under models/)

## Suggested Models to Add

| Model | Process | Notes |
|---|---|---|
| `QED` | e+e- → μ+μ- | Simplest possible; analytic cross section known |
| `SM` | e+e- → μ+μ- (with Z) | Tests Z propagator |
| `Scalar` | φφ → φφ | Minimal scalar toy; single vertex |
