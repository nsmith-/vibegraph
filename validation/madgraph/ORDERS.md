# MadGraph Coupling Orders Reference

Quick reference for the coupling order constraints used in validation scripts.

## Standard Model Couplings

In MadGraph and vibegraph, coupling orders refer to the perturbative expansion in fundamental couplings:

| Coupling | Symbol | Appearance | Example |
|----------|--------|-----------|---------|
| Strong (QCD) | α_s | Gluon exchange, quark–gluon vertices | `p p > j j` |
| Electroweak (QED) | α | Photon exchange, W/Z vertices | `e+ e- > e+ e-` |

## Coupling Order Semantics

### Default Behavior (no explicit order)

When no coupling order is specified, MadGraph includes **all tree-level diagrams** at leading order:

```mg5
generate e+ e- > mu+ mu-
```
Generates: QED=2 photon/Z exchange (tree-level QED)

```mg5
generate p p > l+ l-
```
Generates: 
- QCD=2 (initial-state quark coupling)
- QED=2 (dilepton final state)
- All topologies at tree level

### Explicit Upper Bounds

Use coupling name + operator + power to constrain diagrams:

#### `QCD=0` — Suppress strong coupling

```mg5
generate p p > l+ l- QCD=0
```
- Rejects all diagrams with gluons
- Keeps only electroweak (photon/Z) exchange
- Result: Pure QED dilepton production

#### `QCD=2` — Exactly 2 QCD couplings

```mg5
generate p p > b b~ QCD=2
```
- Selects diagrams with exactly 2 gluon exchanges
- Rejects higher-order contributions
- Result: LO strong production of bb̄ pair

#### `QED=2` — Exactly 2 QED couplings

Used in combination with QCD constraints to select specific coupling combinations:

```mg5
generate p p > l+ l- j QCD=2 QED=2
```
- Initial-state QCD: exactly 2 powers (qq̄g coupling)
- Final-state QED: exactly 2 powers (l⁺l⁻ production)
- Rejects higher-order or pure-QED variants

### Operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `=` | Exactly equal to | `QCD=2` → 2 gluons only |
| `==` | Strictly equal (same as `=`) | `QCD==2` |
| `<=` | Less than or equal | `QCD<=2` → up to 2 gluons |
| `>=` | Greater than or equal | `QCD>=2` → 2+ gluons |
| `<`, `>` | Strict inequalities | `QCD<2` → 0 or 1 gluon |
| `!=` | Not equal | `QCD!=2` → anything but 2 |

For squared orders (|M|²), `=` is interpreted as `<=` (standard convention).

## Tree-Level Process Examples

### e⁺e⁻ → μ⁺μ⁻

| Process String | Coupling Orders | Diagrams | Notes |
|---|---|---|---|
| `e+ e- > mu+ mu-` | QED=2 | 1 (photon) | Pure QED at tree |

**Details:** 
- One photon-exchange diagram
- At electron-muon threshold, Z boson also contributes but is part of QED=2 vertex structure
- Total: **1 diagram** in feyngraph (photon mediated at LO)

### p p → l⁺l⁻

| Process String | Coupling Orders | Diagrams | Notes |
|---|---|---|---|
| `p p > l+ l-` | QCD=2, QED=2 | ~4-6 | Default LO |
| `p p > l+ l- QCD=0` | QED=2 only | ~2-3 | Pure electroweak |

**Details:**
- Initial state: proton pair (up, down, or anti-flavors)
- QCD=2: one gluon from each initial quark
- QED=2: dilepton pair via γ or Z exchange
- Total with QCD=2 QED=2: typically **4–6 diagrams** depending on quark flavors

### p p → l⁺l⁻ j

| Process String | Coupling Orders | Diagrams | Notes |
|---|---|---|---|
| `p p > l+ l- j` | QCD=3, QED=2 | ~8-12 | Default LO |
| `p p > l+ l- j QCD=2 QED=2` | QCD=2, QED=2 | ~4-6 | Restricted |

**Details:**
- Initial state: quark pair
- Final state: dilepton + gluon (jet)
- Default: includes all tree topologies
- Constrained (QCD=2 QED=2): suppresses extra gluon radiation diagrams
- Typical counts: **8–12 diagrams** for default, **4–6** with constraint

### p p → bb̄

| Process String | Coupling Orders | Diagrams | Notes |
|---|---|---|---|
| `p p > b b~` | QCD=2 | 3-4 | Default LO |
| `p p > b b~ QCD=2` | QCD=2 only | 3-4 | Explicit constraint |

**Details:**
- Strong-coupling dominated (asymptotic freedom at LHC)
- At LO: primarily gluon exchange + quark-channel contributions
- Typical diagrams:
  - gg → bb̄ (gluon fusion)
  - qq̄ → bb̄ (quark annihilation)
- Total: **3–4 diagrams** (topologically distinct)

## Validation Script Decisions

The chosen coupling constraints serve pedagogical purposes:

1. **Default processes** (no explicit order):
   - Demonstrate MadGraph's automatic diagram enumeration
   - Serve as "integration test" for parsing and generation

2. **QCD=0 variant** (pure electroweak):
   - Tests order constraint filtering
   - Validates that selector correctly suppresses gluon topologies

3. **Explicit QCD=2 QED=2**:
   - Tests simultaneous constraints
   - Validates that coupling orders compose correctly

4. **Heavy-quark processes** (b, t, etc.):
   - Extend beyond light-lepton processes
   - Test quark flavor handling in diagram enumeration

## Future Extensions

### Adding decay chains

Not yet in validation scripts; requires extended process syntax:

```mg5
generate p p > w+ w- > l+ l- nu nu
```

This introduces **process hierarchies** (generate top process, then expand decays).
MadGraph output structure differs (decay chains in SubProcesses subtree).

### NLO processes

Virtual + real contributions require:
```mg5
generate p p > l+ l- [QCD]
```

This enables QCD corrections (one-loop, real-emission topologies).
Our validation focuses on **LO (tree-level)** only.

## References

- MadGraph5 manual: https://launchpad.net/madgraph5
- `research/notes/06-process-grammar.md` — vibegraph process grammar
- `research/notes/02-reference-implementations.md` — MG5 design patterns
