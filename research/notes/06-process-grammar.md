# MadGraph5 Process-Specification Language: PEG Grammar

## Overview

This note formally captures the process-specification language used by MadGraph5_aMC@NLO
(`generate p p > e+ e- j`) as a PEG grammar, so that vibegraph can parse the same syntax
to steer its own diagram generator (the `feyngraph` crate).

---

## 1. Parser Location in mg5amcnlo

All source paths are relative to `research/refs/mg5amcnlo/`.

| Method | File | Lines | Role |
|--------|------|-------|------|
| `do_generate` | `madgraph/interface/madgraph_interface.py` | 4811–4820 | Entry point; delegates to `do_add` |
| `do_add` | same | 3232–3399 | Checks for decay chains; calls `extract_process` or `extract_decay_chain_process` |
| `check_process_format` | same | 1150–1238 | Validates parentheses, `>` count, placement of `/` and `$` |
| `extract_process` | same | 4822–5353 | **Core parser** — strips modifiers one by one with regex, then tokenises |
| `extract_decay_chain_process` | same | 5661–5749 | Recursive descent over comma-separated decay branches |
| `do_define` | same | 3527–3563 | Parses multiparticle alias definitions |
| `split_arg` | `madgraph/interface/extended_cmd.py` | 687–706 | Splits whitespace-separated tokens (respects quotes) |

Default multiparticle aliases are in `input/multiparticles_default.txt`:
```
p  = g u c d s u~ c~ d~ s~
j  = g u c d s u~ c~ d~ s~
l+ = e+ mu+
l- = e- mu-
vl = ve vm vt
vl~= ve~ vm~ vt~
```

---

## 2. How MadGraph Parses a Process String

`extract_process` (line 4822) strips modifiers from the string in this fixed order,
using successive `re.match` / `re.search` calls. Each strip mutates `line` in-place.

```
Step 1  @N              — proc_number_pattern  (line 4844)
Step 2  [ mode = ORDS ] — perturbation_couplings_pattern  (line 4852)
Step 3  NAME OP VALUE   — order_pattern  (line 4883, repeated until none left)
Step 4  / particles     — forbidden_particles  (line 5006)
Step 5  $$ particles    — forbidden_s_channels  (line 5022)
Step 6  $ particles     — forbidden_onsh_s_channels  (line 5029)
Step 7  A > B > C       — required_schannels  (line 5036)
Step 8  split_arg(line) → token loop, ">"-delimited  (line 5043–5242)
```

After all stripping, the remaining string is `"initial_state > final_state"` (or
`"initial_state > req_schan_state > final_state"` before step 7 strips it). The token
loop converts each whitespace-separated token to a `MultiLeg` object. Encountering `>`
flips `state` from initial (`False`) to final (`True`).

---

## 3. Key Regex Patterns (verbatim from source)

```python
# Step 1 – process tag  (line 4844)
proc_number_pattern = re.compile(r"^(.+)@\s*(\d+)\s*(.*)$")

# Step 2 – loop/NLO spec  (line 4852)
perturbation_couplings_pattern = re.compile(
    r"^(?P<proc>.+>.+)\s*\[\s*((?P<option>\w+)\s*\=)?\s*"
    r"(?P<pertOrders>(\w+\s*)*)\s*\]\s*(?P<rest>.*)$")

# Step 3 – coupling order constraint  (line 4883)
order_pattern = re.compile(
    r"^(?P<before>.+>.+)\s+(?P<name>(\w|(\^2))+)\s*(?P<type>"
    r"(=|(<=)|(==)|(===)|(!=)|(>=)|<|>))\s*(?P<value>-?\d+)\s*?(?P<after>.*)")

# Step 4 – forbidden particles  (line 5012/5014)
re.match(r"^(.+)\s*/\s*(.+\s*)(\$.*)$", line)   # / before $
re.match(r"^(.+)\s*/\s*(.+\s*)$", line)          # / without $

# Step 5 – forbidden s-channels  (line 5022)
re.match(r"^(.+)\s*\$\s*\$\s*(.+)\s*$", line)

# Step 6 – forbidden on-shell s-channels  (line 5029)
re.match(r"^(.+)\s*\$\s*(.+)\s*$", line)

# Step 7 – required s-channels  (line 5036)
re.match("^(.+?)>(.+?)>(.+)$", line)
```

Valid NLO modes (line 3035):
```python
_valid_nlo_modes = ['all','real','virt','sqrvirt','tree','noborn','LOonly','only']
```

Valid amplitude order operators (steps 3): `=`, `<=`, `==`, `===`, `!=`, `>=`, `<`, `>`  
For squared orders (`NAME^2`): `=` is silently interpreted as `<=`.

---

## 4. Particle Token Syntax

After all modifier stripping, `split_arg` (whitespace-split with quote awareness) produces
a list of tokens. Each token is one of:

| Token form | Example | Meaning |
|------------|---------|---------|
| `NAME` | `e-`, `mu+`, `t~`, `g`, `W+` | Single particle by model name |
| `N` (integer) | `11`, `-11` | PDG code |
| `ALIAS` | `p`, `j`, `l+` | Multiparticle alias (defined via `define`) |
| `dNAME` | `2e+` | Duplicate: `d` copies of particle NAME (d is a single digit) |
| `NAME{POL}` | `z{T}`, `e-{L}`, `a{+1,-1}` | Particle with helicity polarization |
| `!NAME!` | `!a!` | Tagged particle (photon tagging / UPC) |
| `>` | `>` | Separator: switches state from initial to final |

**Particle name characters** (inferred from model lookup logic, lines 5201–5223):
- Letters: `[a-zA-Z]` (case may fold to lower depending on model setting)
- Digits may appear as a leading duplication count only
- Special suffix chars: `+`, `-`, `~` (antiparticle marker)

**Polarization codes** (lines 5115–5188):

| Code | Meaning | Spin restriction |
|------|---------|-----------------|
| `T` | Transverse (+1 and −1) | spin-1 only |
| `L` | Left / helicity −1 | any (warns for spin-1) |
| `R` | Right / helicity +1 | any |
| `0` | Longitudinal | spin ≥ 2 |
| `A` | Auxiliary (99) | spin-1 only |
| `G` | Metric (4) | spin-1 only |
| `H` | Theta (5) | spin-1 only |
| `Q` | qq = long − Theta (6) | spin-1 only |
| `W` | Ward full propagator (7) | spin-1 only |
| `S` | Scalar = aux + width (9) | spin-1 only |
| `+d` | helicity +d (d=digit) | any |
| `-d` | helicity −d | any |
| `d` | helicity d | any |

Multiple codes can be comma-separated inside `{}`: `z{+1,-1}` = `z{T}`.

---

## 5. PEG Grammar

Grammar is written in pest.rs notation. Silent rules are prefixed `_`.
Whitespace around operators is optional unless explicitly noted.

```pest
// ─── Whitespace ────────────────────────────────────────────────────────────────
WHITESPACE = _{ " " | "\t" }

// ─── Basic lexical atoms ───────────────────────────────────────────────────────

// A particle name: letters, digits, and the special suffix chars +/-/~
// Leading digit signals duplication, not part of name itself.
particle_name_char = _{ ASCII_ALPHA | ASCII_DIGIT | "+" | "-" | "~" | "_" }
particle_name      =  { ASCII_ALPHA ~ particle_name_char* }

// PDG code (signed integer)
pdg_code = { "-"? ~ ASCII_DIGIT+ }

// Multiparticle alias — same lexical form as particle_name; resolved at runtime.
// (Grammar cannot distinguish alias from particle; both match particle_name.)

// Duplication prefix: a single decimal digit before the particle name.
// E.g. "2e+" means two copies of e+.
duplication_count = { ASCII_NONZERO_DIGIT }

// ─── Polarization ──────────────────────────────────────────────────────────────
pol_letter = { ^"T" | ^"L" | ^"R" | ^"A" | ^"G" | ^"H" | ^"Q" | ^"W" | ^"S" }
pol_numeric = { ("+" | "-")? ~ ASCII_DIGIT }
pol_code = { pol_letter | pol_numeric }
polarization = { "{" ~ pol_code ~ ("," ~ pol_code)* ~ "}" }

// ─── Single particle token ─────────────────────────────────────────────────────
// A "bare" particle or alias name, possibly with duplication prefix and/or
// polarization suffix.
particle_token = {
    tagged_particle
  | duplication_count ~ particle_name ~ polarization?   // e.g. 2e+, 2p, 2z{T}
  | particle_name ~ polarization?                       // e.g. e+, mu-, p, z{L}
  | pdg_code                                            // e.g. 11, -11
}

// Tagged particle: !NAME! or dNAME! (initial-state UPC photon tagging)
tagged_particle = {
    "!" ~ particle_name ~ "!"
  | duplication_count ~ "!" ~ particle_name ~ "!"
}

// ─── Particle lists ────────────────────────────────────────────────────────────
particle_list = { particle_token ~ (WHITESPACE+ ~ particle_token)* }

// ─── Process body (states and required s-channel) ─────────────────────────────
// CORE FORM: "A B > C D" (initial > final)
// REQUIRED S-CHANNEL: "A B > X > C D" (double-arrow form)

initial_state = { particle_list }
final_state   = { particle_list }
s_channel_required = { particle_list }

// NOTE: MadGraph requires the two ">" to be separate whitespace-delimited tokens
// in practice, but parses them with regex `^(.+?)>(.+?)>(.+)$` without requiring
// surrounding whitespace.
process_body = {
    initial_state ~ WHITESPACE* ~ ">" ~ WHITESPACE*
    ~ s_channel_required ~ WHITESPACE* ~ ">" ~ WHITESPACE*
    ~ final_state
  | initial_state ~ WHITESPACE* ~ ">" ~ WHITESPACE*
    ~ final_state
}

// ─── Restrictions (placed after the final state) ───────────────────────────────
// Parsed in this order by MadGraph (steps 4–6 above).

// Exclusion: "/ g t" — remove particles from propagators entirely
forbidden_particles = { "/" ~ WHITESPACE* ~ particle_list }

// No s-channel (hard): "$$ Z" — particle cannot appear as any s-channel
forbidden_s_channels = { "$$" ~ WHITESPACE* ~ particle_list }

// No on-shell s-channel: "$ Z" — particle cannot be on-shell s-channel
// NOTE: MadGraph parses $$ before $, so single $ is unambiguous after $$.
forbidden_onsh_s_channels = { "$" ~ WHITESPACE* ~ particle_list }

restriction = {
    forbidden_particles
  | forbidden_s_channels
  | forbidden_onsh_s_channels
}

// ─── Coupling order constraints ────────────────────────────────────────────────
// Placed after the final-state list, before or after restrictions.
// NAME may include "^2" for squared-amplitude constraints.

order_name = { ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* ~ ("^2")? }

// Amplitude-level operators: =, <=, ==, ===, !=, >=, <, >
// Squared-order operators:   =, <=, ==, ===, !=, >=
order_op = { "===" | "==" | "<=" | ">=" | "!=" | "<" | ">" | "=" }

order_constraint = { order_name ~ WHITESPACE* ~ order_op ~ WHITESPACE* ~ ("-"? ~ ASCII_DIGIT+) }

// ─── Loop / NLO specification ─────────────────────────────────────────────────
nlo_mode   = { ^"sqrvirt" | ^"noborn" | ^"LOonly" | ^"virt" | ^"real" | ^"tree" | ^"all" | ^"only" }
pert_order = { ASCII_ALPHA ~ ASCII_ALPHANUMERIC* }
loop_spec  = {
    "[" ~ WHITESPACE* ~ (nlo_mode ~ WHITESPACE* ~ "=")? ~ WHITESPACE*
    ~ (pert_order ~ (WHITESPACE+ ~ pert_order)*)?
    ~ WHITESPACE* ~ "]"
}

// ─── Process tag ──────────────────────────────────────────────────────────────
process_tag = { "@" ~ WHITESPACE* ~ ASCII_DIGIT+ }

// ─── Core (LO, tree-level) simple process ─────────────────────────────────────
// This is the minimal form needed for LO diagram generation.
// 🟢 CORE elements are marked; 🔵 EXTENDED elements are for NLO/advanced use.

simple_process = {
    process_body                        // 🟢 required
    ~ (WHITESPACE+ ~ restriction)*      // 🟢 optional restrictions
    ~ (WHITESPACE+ ~ order_constraint)* // 🟢 coupling order bounds (LO: QCD=2 etc.)
    ~ (WHITESPACE+ ~ loop_spec)?        // 🔵 NLO perturbation orders
    ~ (WHITESPACE+ ~ process_tag)?      // 🟢 diagram-set tag (for multi-process runs)
}

// ─── Decay chain process ──────────────────────────────────────────────────────
// Comma-separated; parentheses for nested hierarchies.
// 🔵 EXTENDED — not needed for LO 2→N generation.

decay_branch = {
    "(" ~ WHITESPACE* ~ decay_chain_process ~ WHITESPACE* ~ ")"
  | simple_process
}

decay_chain_process = { simple_process ~ (WHITESPACE* ~ "," ~ WHITESPACE* ~ decay_branch)* }

// ─── Top-level process specification ─────────────────────────────────────────
process_spec = { decay_chain_process }

// ─── Multiparticle definition ─────────────────────────────────────────────────
// 🟢 CORE — aliases like "p", "j" are ubiquitous in process specs.
//
// Supports subtraction: "define q = p / g"
// Supports OR lists (used for required s-channel multiparticles): "define V = Z a"
define_rhs    = { particle_list }
define_except = { "/" ~ WHITESPACE* ~ particle_list }
multiparticle_def = {
    particle_name ~ WHITESPACE* ~ "="
    ~ WHITESPACE* ~ define_rhs
    ~ (WHITESPACE* ~ define_except)?
}

// ─── Top-level commands ───────────────────────────────────────────────────────
generate_cmd    = { ^"generate"    ~ WHITESPACE+ ~ process_spec }
add_process_cmd = { ^"add"         ~ WHITESPACE+ ~ ^"process" ~ WHITESPACE+ ~ process_spec }
define_cmd      = { ^"define"      ~ WHITESPACE+ ~ multiparticle_def }
```

### 5.1 Core vs Extended

| Feature | Grammar rule | Needed for LO tree? |
|---------|-------------|-------------------|
| `A > B` process body | `process_body` | 🟢 Yes |
| `A > X > B` required s-channel | `process_body` (alt) | 🟢 Yes |
| `/` forbidden particles | `forbidden_particles` | 🟢 Yes |
| `$` forbidden on-shell s-chan | `forbidden_onsh_s_channels` | 🟢 Yes |
| `$$` forbidden s-chan | `forbidden_s_channels` | 🟢 Yes |
| `QCD=2`, `QED<=4` orders | `order_constraint` | 🟢 Yes |
| `@N` process tag | `process_tag` | 🟢 Yes (multi-process runs) |
| `define alias = particles` | `multiparticle_def` | 🟢 Yes |
| Duplication `2e+` | `particle_token` | 🟢 Yes |
| `NAME^2 <= N` squared orders | `order_constraint` | 🟡 Advanced LO |
| `{L}`,`{T}` polarization | `polarization` | 🟡 Advanced LO |
| `[ QCD ]` loop spec | `loop_spec` | 🔵 NLO only |
| `[all=QCD]` NLO modes | `loop_spec` | 🔵 NLO only |
| `p p > t t~, (t > b w+)` decay chains | `decay_chain_process` | 🔵 Extended |
| `!a!` tagged particles | `tagged_particle` | 🔵 UPC only |

---

## 6. Data Flow: Parsed String → Diagram Generation

```
do_generate(line)
  └─ do_add("process " + line)
       ├─ [if ',' in line] extract_decay_chain_process(line)
       │    ├─ extract_process(core_part)     → ProcessDefinition (core)
       │    └─ extract_process(decay_part)*   → ProcessDefinition (each decay)
       │         appended to core.decay_chains
       └─ [otherwise] extract_process(line)   → ProcessDefinition
            │
            └─ diagram_generation.MultiProcess(myprocdef)
                 │  (madgraph/core/diagram_generation.py)
                 ├─ .get('amplitudes')  → AmplitudeList
                 │    each Amplitude contains DiagramList
                 └─ appended to self._curr_amps
```

### Python objects created

**`base_objects.MultiLeg`** (one per particle token):
```python
MultiLeg({
    'ids':          [11, -11, ...],  # list of PDG codes (expanded multiparticle aliases)
    'state':        True,            # False=initial, True=final
    'polarization': [-1],            # [] if unspecified
})
```

**`base_objects.ProcessDefinition`** (one per simple process):
```python
ProcessDefinition({
    'legs':                  MultiLegList([...]),
    'model':                 Model(...),
    'id':                    0,           # from @N
    'orders':                {'QCD': 2},  # amplitude order upper bounds
    'squared_orders':        {'QCD': 4},  # |M|^2 order bounds
    'sqorders_types':        {'QCD': '<='}, 
    'constrained_orders':    {},          # equality constraints on orders
    'forbidden_particles':   [21],        # PDG codes
    'forbidden_onsh_s_channels': [23],   # PDG codes
    'forbidden_s_channels':  [],         # PDG codes
    'required_s_channels':   [[23,22]],  # list of PDG lists (OR semantics per inner list)
    'overall_orders':        {},
    'perturbation_couplings':['QCD'],    # for NLO
    'has_born':              True,
    'NLO_mode':              'tree',     # or 'virt', 'real', etc.
    'split_orders':          [],
    'decay_chains':          [...],      # nested ProcessDefinition objects
})
```

**`diagram_generation.MultiProcess`** (the diagram generator):
- Iterates over all particle combinations (expanding multiparticle aliases)
- For each concrete particle assignment creates an `Amplitude`
- Each `Amplitude` holds a `DiagramList`

---

## 7. Worked Example: `generate e+ e- > mu+ mu-`

### Step-by-step through `extract_process`

Input string: `"e+ e- > mu+ mu-"`

**Step 1 — proc tag `@N`:** regex `^(.+)@\s*(\d+)\s*(.*)$` → no match → `proc_number = 0`

**Step 2 — loop spec `[...]`:** regex needs `.+>.+` and `\[...\]` → no match → `LoopOption = 'tree'`

**Step 3 — coupling orders:** regex needs `NAME OP VALUE` after `>` → no match → `orders = {}`, `squared_orders = {}`

**Step 4 — forbidden particles `/`:** `"/"` not in string → skipped

**Step 5 — `$$`:** not in string → skipped

**Step 6 — `$`:** not in string → skipped

**Step 7 — required s-channel `> X >`:** only one `>` in string → no match

**Step 8 — token loop** (after `split_arg("e+ e- > mu+ mu-")` → `["e+", "e-", ">", "mu+", "mu-"]`):

| Token | state | Action |
|-------|-------|--------|
| `e+`  | False | look up in model → PDG −11 → `MultiLeg({'ids': [-11], 'state': False})` |
| `e-`  | False | PDG 11 → `MultiLeg({'ids': [11], 'state': False})` |
| `>`   | →True | flip `state = True` |
| `mu+` | True  | PDG −13 → `MultiLeg({'ids': [-13], 'state': True})` |
| `mu-` | True  | PDG 13 → `MultiLeg({'ids': [13], 'state': True})` |

**Object created:**
```python
ProcessDefinition({
    'legs': [
        MultiLeg({'ids': [-11], 'state': False}),   # e+  (initial)
        MultiLeg({'ids': [11],  'state': False}),   # e-  (initial)
        MultiLeg({'ids': [-13], 'state': True}),    # mu+ (final)
        MultiLeg({'ids': [13],  'state': True}),    # mu- (final)
    ],
    'model':    <SM model>,
    'id':       0,
    'orders':   {},              # no coupling order constraints specified
    'NLO_mode': 'tree',
    ...
})
```

**Diagram generation:**
```python
myproc = MultiProcess(myprocdef)
# Expands: only one particle assignment (no multiparticle aliases)
# Generates Amplitude for e+ e- > mu+ mu-
# At LO in QED: one diagram (t-channel photon)
#   e+(p1) e-(p2) → γ* → mu+(p3) mu-(p4)
# Plus any other diagrams allowed by model couplings
```

The resulting `Amplitude.get_diagrams()` contains one `Diagram` for the single tree-level
Feynman diagram with an off-shell photon in the t-channel.

---

## 8. Feyngraph Gap Analysis

The `feyngraph` submodule (`research/refs/feyngraph/`) is a git submodule pointing to
[`https://github.com/Jens-Braun/FeynGraph`](https://github.com/Jens-Braun/FeynGraph)
but **has not been initialized** in this worktree (directory is empty).

The following gap analysis is based on the grammar above and FeynGraph's GitHub
description, which states it is a "Rust library for generating Feynman diagrams from
Feynman rules."

### What vibegraph needs feyngraph to accept

For **LO tree-level generation** (the immediate vibegraph goal), the inputs to the
diagram generator are:

1. **A list of external legs** — each with:
   - PDG code (signed integer)
   - State (initial vs. final)
   
2. **A model** (vertex table mapping particle-triples / -quadruples to coupling orders)

3. **Coupling order bounds** — e.g. `{QCD: 2}` meaning "include only diagrams with
   at most 2 QCD vertices"

4. **Forbidden propagator particles** — PDG code set

5. **Forbidden s-channel propagators** — PDG code sets

6. **Required s-channel propagators** — PDG code sets

### Likely gaps (to verify once feyngraph is initialized)

| Feature | MadGraph uses | feyngraph status |
|---------|--------------|-----------------|
| Multiparticle alias expansion | yes (p, j, l+, …) | Unknown — likely done before calling feyngraph |
| Coupling order filtering | yes (QCD=2, QED=4) | Unknown |
| Forbidden propagators | yes (/ particles) | Unknown |
| Required s-channels | yes (> X >) | Unknown |
| Mirror-process deduplication | yes | Unknown |
| Decay chain recursion | yes | Unknown — vibegraph may handle this layer itself |

**Recommendation:** vibegraph should implement a **translation layer** that:
1. Parses the MadGraph process string using the grammar above
2. Expands multiparticle aliases
3. Constructs the external-legs list and filter sets
4. Calls feyngraph with the expanded, concrete inputs

This decouples the MadGraph syntax layer from the diagram-generation layer,
matching how MadGraph itself separates `extract_process` from `MultiProcess`.

---

## 9. Notes on Grammar Ambiguities and Edge Cases

### 9.1 Operator precedence in the un-stripped string
MadGraph does not use a formal PEG parser. It uses `re.match` (anchored at the start
of string) repeatedly, which means the *last* occurrence wins for `@N` and coupling
orders (since `(.+)` is greedy). The grammar above assumes the idiomatic usage where
each modifier appears at most once, except coupling order constraints which may appear
multiple times.

### 9.2 `$` vs `$$` ambiguity
`$$` is checked before `$`, so a string like `"p p > e+ e- $$ Z"` correctly triggers
the hard s-channel rule, not the on-shell rule. A PEG parser naturally handles this
by ordering the alternatives with `$$` before `$`.

### 9.3 Coupling order `=` semantics
- For amplitude orders (`NAME` without `^2`): `=` and `<=` both mean "at most N"
  (MadGraph warns and coerces `=` to `<=` unless value is 0).
- For squared orders (`NAME^2`): `=` is silently coerced to `<=`.
- `==` on amplitude orders creates both an amplitude constraint and a squared-order
  constraint `NAME^2 == 2*N`.

### 9.4 Case sensitivity
If the model is not case-sensitive, the entire process string is lowercased before
tokenisation (line 5002). Most SM models are case-insensitive; some BSM models are
case-sensitive. The grammar should be applied after this normalisation.

### 9.5 Whitespace
`split_arg` uses the pattern
```python
re.findall(r"(?:[^\s'\"]|(?:'|\")(?:\\.|[^\"'])*(?:\"|'))+", line)
```
which respects single- and double-quoted strings and backslash-escaped spaces.
In practice, process strings never contain quotes; all tokens are whitespace-delimited.
The regex modifiers (`/`, `$`, `[...]`, `@`) are stripped **before** `split_arg`, so they
do not need to be whitespace-delimited from the particle tokens.
