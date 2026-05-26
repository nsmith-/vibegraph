# MadGraph Diagram Validation

Validation of vibegraph's diagram enumeration against MadGraph5_aMC@NLO reference output.

## Directory Structure

```
validation/madgraph/
├── scripts/           # MadGraph batch scripts (.mg5 files)
├── build.sh          # Script to run all mg5_aMC processes
├── extract_diagrams.py # Python script to extract diagram counts from output
├── output/           # Generated MadGraph process directories (created by build.sh)
└── README.md         # This file
```

## Processes Covered

### Default Order (leading-order diagrams)

- **ee_to_mumu.mg5**: `e+ e- > mu+ mu-`
  - Pure QED at tree level
  - Reference: 1 photon-exchange diagram

- **pp_to_ll.mg5**: `p p > l+ l-`
  - Dilepton production at hadron collider
  - Includes both QCD initial-state and QED dilepton

- **pp_to_llj.mg5**: `p p > l+ l- j`
  - Dilepton + jet production
  - With one additional gluon radiation

- **pp_to_bb.mg5**: `p p > b b~`
  - Heavy quark pair production
  - QCD-dominant process

### Explicit Order Constraints

- **pp_to_ll_qcd0.mg5**: `p p > l+ l- QCD=0`
  - Pure electroweak: no strong coupling
  - Tests order constraint filtering

- **pp_to_llj_qcd2_qed2.mg5**: `p p > l+ l- j QCD=2 QED=2`
  - Fixed coupling orders
  - Tests order constraint enforcement

- **pp_to_bb_qcd2.mg5**: `p p > b b~ QCD=2`
  - Fixed QCD order
  - Tests order selectivity

## Usage

### 1. Generate MadGraph Reference Output

Requires pixi with the `madgraph` environment configured:

```bash
pixi run -e madgraph build-diagrams
```

This:
- Runs each `.mg5` script via `mg5_aMC`
- Creates output directories under `validation/madgraph/output/`
- Each directory contains `SubProcesses/P*/` with diagram files

Expected output structure:
```
output/
├── ee_to_mumu_lo/
│   ├── proc_card.dat
│   ├── SubProcesses/
│   │   ├── P0_ee_mumu/
│   │   │   ├── *.ps        (diagram PostScript files)
│   │   │   └── ...
│   └── ...
├── pp_to_ll_lo/
│   └── ...
└── ...
```

### 2. Extract Diagram Metadata

```bash
pixi run -e madgraph extract-diagrams
```

This:
- Reads each `proc_card.dat` and counts `.ps` files in `SubProcesses/`
- Writes `validation/madgraph/output/diagrams.json` with counts per process
- Example output:
  ```json
  {
    "ee_to_mumu_lo": {
      "process": "e+ e- > mu+ mu-",
      "total_diagrams": 1,
      "diagrams_by_subprocess": {
        "P0_ee_mumu": 1
      }
    },
    "pp_to_ll_lo": {
      "process": "p p > l+ l-",
      "total_diagrams": 4,
      "diagrams_by_subprocess": {
        "P0_qq_ll": 2,
        "P1_qq_ll": 2
      }
    }
  }
  ```

### 3. Run Validation Tests

With `diagrams.json` in place, run the Rust test suite:

```bash
cargo test -p vibegraph-lib --test validate_madgraph_diagrams
```

Each test:
- Parses a process string with vibegraph
- Generates diagrams using the SM UFO model
- Compares count against MadGraph reference
- Reports match/mismatch

Example output:
```
test validate_ee_to_mumu ... ok
test validate_pp_to_ll ... ok
test validate_pp_to_llj ... ok
test validate_pp_to_bb ... ok
test validate_pp_to_ll_qcd0 ... ok
test validate_pp_to_llj_explicit_orders ... ok
test validate_pp_to_bb_qcd2 ... ok
```

## Order Constraint Semantics

### Default Behavior

When no coupling order is specified, MadGraph generates all tree-level contributions:

- **`e+ e- > mu+ mu-`**: QED=2 (photon)
- **`p p > l+ l-`**: QCD=2 (from initial quarks) + QED=2 (dilepton)
- **`p p > b b~`**: QCD=2 (strong production)

### Explicit Constraints

- **`QCD=0`**: Suppress strong coupling; keep only electroweak diagrams
- **`QCD=2 QED=2`**: Exactly 2 of each coupling (rejects higher-order combinations)

The PEG parser in `vibegraph-lib/src/diagrams/parse.rs` handles these constraints;
the selector in `vibegraph-lib/src/diagrams/selector.rs` filters diagrams accordingly.

## Troubleshooting

### MadGraph runs produce no diagrams

- Verify MadGraph is installed: `pixi run -e madgraph mg5_aMC --version`
- Check that `SubProcesses/P*/` directories exist in output
- MadGraph may refuse to overwrite existing directories; delete old output and retry

### `diagrams.json` not found

Run `pixi run -e madgraph extract-diagrams` (which depends on `build-diagrams`).

### Diagram count mismatches

- Check that vibegraph's order constraint parsing matches MadGraph's interpretation
- See `research/notes/06-process-grammar.md` for coupling order semantics
- Verify UFO model is correctly loaded (see test for path)

## Future Extensions

- [ ] Add decay processes (e.g., `p p > w+ w- > l+ l- nu nu`)
- [ ] Test NLO processes (virtual + real)
- [ ] Cross-check against LHE event file decay chains
- [ ] Validate color-flow assignments in event output
- [ ] Compare running times / optimization opportunities
