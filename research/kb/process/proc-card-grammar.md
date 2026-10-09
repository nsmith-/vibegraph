---
type: Physics Convention
title: Proc-card grammar and command semantics (MadGraph parity)
description: "How MadGraph reads a process line and proc card: modifier strip order and regexes, leg classification, coupling-order operators, @N, generate/add process, duplicates, the p/j rewrite."
status: draft
tags: [process-grammar, proc-card, madgraph-parity, parser, coupling-orders]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n06-grammar, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/06-process-grammar.md#L12-L150", title: "Note 06 §1–§4, MadGraph's process parser, regexes and token syntax"}
  - {id: n06-flow, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/06-process-grammar.md#L334-L454", title: "Note 06 §6–§7, data flow and the e+ e- > mu+ mu- walk-through"}
  - {id: n06-edges, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/06-process-grammar.md#L587-L620", title: "Note 06 §9, grammar edge cases"}
  - {id: n38-line, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L47-L76", title: "Note 38 §1.1, the process line"}
  - {id: n38-commands, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L157-L169", title: "Note 38 §1.4, proc-card commands"}
  - {id: n38-g1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L310-L361", title: "Note 38 §4 G1, grammar, AST and the parser oracle"}
  - {id: n38-s2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L433-L496", title: "Note 38 §4 S2, WEIGHTED bounds and the p/j rewrite"}
  - {id: n38-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1032-L1143", title: "Note 38 §4 E1, @N and add process by content"}
  - {id: mg-extract-process, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L4822", title: "MadGraph madgraph_interface.py extract_process"}
  - {id: mg-orders, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L4883-L5000", title: "MadGraph madgraph_interface.py coupling-order parsing"}
  - {id: mg-do-add, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L3270-L3380", title: "MadGraph madgraph_interface.py do_add"}
  - {id: mg-generate, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L4791-L4820", title: "MadGraph madgraph_interface.py clean_process and do_generate"}
  - {id: mg-multiparticles, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L5998-L6060", title: "MadGraph madgraph_interface.py add_default_multiparticles"}
measured:
  - {commit: 1f5f924, pr: 12, landed_in: 1539abc, command: "cargo test -p vibegraph-lib --test proc_grammar_oracle"}
---

vibegraph reads MadGraph proc cards as MadGraph does. The parser
(`vibegraph-lib/src/diagrams/parse.rs`) mirrors MadGraph's command interface:
`extract_process` for a process line, `extract_decay_chain_process` for one with decay
chains, `do_define` for multiparticle labels, and `precmd`'s line handling (`#` comments,
`;` separators, `\` continuations). Where MadGraph states its grammar as regular
expressions, **the same expressions are applied, in the same order** (`regex` crate),
including the ones whose edges look accidental. There is no PEG process grammar. The
result is a `ProcCardAst` that records every construct a card names, supported or not;
deciding what is supported is [the card check](supported-card-check.md), and resolving
names against a model is `diagrams::resolve`. The user-facing description is
`docs/src/guide/03-diagrams.md`, "The process grammar".

**Oracle.** MadGraph's own parser. `validation/madgraph/dump_proc_grammar.py` runs 132
cards through `MasterCmd` with generation stubbed and banks each `ProcessDefinition` as
JSON (`proc_grammar.json`); the hermetic `proc_grammar_oracle` test agrees field by field
on all 92 cards MadGraph reads and refuses all 40 it refuses. [^n38-g1] Line numbers
below are at the MadGraph pin `b7687064` (3.7.1).

## How MadGraph reads a card

```text
do_generate(line)                    clean_process(): every earlier process is discarded
  └─ do_add("process " + line)
       ├─ ',' in line → extract_decay_chain_process → core ProcessDefinition + decay_chains
       └─ otherwise   → extract_process            → ProcessDefinition
            └─ diagram_generation.MultiProcess → amplitudes
```

[^mg-generate] [^mg-do-add] `add process` appends without the reset. A card cannot mix
processes with different numbers of initial particles (`do_add`, L3317). [^n06-flow]

## The process line: modifiers come off in a fixed order

`extract_process` [^mg-extract-process] strips modifiers with successive anchored
`re.match` calls, each mutating the line, in this order:

| Step | Construct | Pattern |
|---|---|---|
| 1 | `@N` process number | `^(.+)@\s*(\d+)\s*(.*)$`: text may follow, so `p p > j j @1 QED=0` is valid |
| 2 | `[...]` loop / perturbation spec | `^(?P<proc>.+>.+)\s*\[\s*((?P<option>\w+)\s*=)?\s*(?P<pertOrders>(\w+\s*)*)\s*\]\s*(?P<rest>.*)$` |
| 3 | coupling orders, repeated | `^(?P<before>.+>.+)\s+(?P<name>(\w\|(\^2))+)\s*(?P<type>(=\|(<=)\|(==)\|(===)\|(!=)\|(>=)\|<\|>))\s*(?P<value>-?\d+)\s*?(?P<after>.*)` |
| 4 | `/` forbidden particles | `^(.+)\s*/\s*(.+\s*)(\$.*)$`, else `^(.+)\s*/\s*(.+\s*)$` |
| 5 | `$$` forbidden s-channels | `^(.+)\s*\$\s*\$\s*(.+)\s*$` |
| 6 | `$` forbidden on-shell s-channels | `^(.+)\s*\$\s*(.+)\s*$` |
| 7 | `> … >` required s-channels | `^(.+?)>(.+?)>(.+)$` |

Then the remainder is split on whitespace into legs, `>` switching from initial to final
state. Consequences, each pinned by a parser test:

- **`$$` is tried before `$`**, so `$$ Z` is never read as `$` twice. [^n06-edges]
- **The modifiers are not order-free.** Only some orders of `/`, `$$` and `$` parse:
  `p p > e+ e- $$ w+ / h` reads, `p p > e+ e- / h $$ w+` does not
  (`restriction_order_follows_madgraphs_expressions`).
- **Whitespace matters.** MadGraph's spacing fix-up regex was meant to cover
  `[ ] / , $ > |` but, as Python parses the character class, only ever matches a `]`
  between non-blanks. So `p p>e+ e-` is a MadGraph error and an error here
  (`unspaced_separators_are_not_legs`). `check_process_format`'s separator test is `>\D`,
  so `QCD^2>2` is not read as a separator.
- **Greedy `(.+)`** means the *last* occurrence of `@N` wins; coupling orders may repeat.
- **Case.** MadGraph lowercases the line when the model is not case-sensitive. Here names
  match the model exactly and then case-insensitively, which agrees for every model whose
  particle names do not differ by case alone.

Valid NLO modes inside `[...]` are `all`, `real`, `virt`, `sqrvirt`, `tree`, `noborn`,
`LOonly`, `only`; any `[...]` is refused by the check (LO only), and `[...]` together with
a decay chain is a MadGraph error and a parse error here.

## Legs

Each leg token is classified in MadGraph's order: [^n38-line]

1. a **multiparticle label** (`p`, `j`, `l+`, or one the card `define`s);
2. an **integer PDG code** (`11`, `-11`);
3. a **model particle name** (`e-`, `t~`, `W+`);
4. only when none matches, a **leading digit as a repeat count** (`2e+` = `e+ e+`).

`11 -11 > 21 21` is four PDG codes, not repeats (`pdg_codes_are_codes_not_repeat_counts`).
The parser has no model, so it does steps 1, 2 and 4 and leaves step 3 to resolution; a
token that is both a model particle and a repeat count followed by a name is refused as
`AmbiguousRepeat` rather than guessed. A repeat count of 0 is an error.

A leg may also carry a **polarization** (`w+{0}`, `z{T}`, `e-{L}`; see
[polarization](polarization.md)) and a **photon tag** (`!a!`, an NLO feature, refused).

**Restriction lists** (`extract_particle_ids`, L5591) are or-lists of and-lists. Separate
names are and-ed; a plain multiparticle expands into the and-list; a `|` or-multiparticle
(`define v = z | a`) gives alternatives. Or-multiparticles are allowed **only in required
s-channels** (`ParseError::OrMultiparticle` elsewhere), and `> A A >` is an error. `/ X`
forbids by |PDG|, so a charged `X` is forbidden in both orientations.

## Coupling-order operators

The order regex matches eight operators, but MadGraph accepts only four for amplitude
orders. [^mg-orders]

| Written | MadGraph | Here |
|---|---|---|
| `QED=n` | at most n (warns unless n = 0) | upper bound |
| `QED<=n` | at most n | upper bound |
| `QED==n` | diagrams not at exactly n dropped, **plus** squared order `QED^2==2n` | each diagram exactly n |
| `QED>n` | no amplitude cap, diagrams at order ≤ n dropped, **plus** squared order `QED^2>2n` | each diagram above n |
| `<`, `>=`, `!=`, `===` | error | `ParseError::OrderOperator` |
| `NAME^2…`, `aEW=n`, `aS=n` | squared-order constraint (`=` read as `<=`; `aEW`/`aS` become `QED^2`/`QCD^2` at 2n) | refused: not yet supported |
| `WEIGHTED<=n`, `WEIGHTED=n` | per-diagram weighted bound | supported, the same bound the automatic search uses |
| `WEIGHTED==n`, `WEIGHTED>n` | adds a squared-order constraint | refused (`Unsupported::WeightedOrder`) |

- `==` here is MadGraph's reading: `filter_constrained_orders` keeps only diagrams at
  exactly n, so every surviving interference term sits at 2n.
- `>` here is equivalent to MadGraph's reading too. MadGraph's
  `apply_squared_order_constraints` first runs `filter_constrained_orders`, which drops
  every diagram whose order is not above n (`diagram_generation.py:864`,
  `base_objects.py:2906-2908`); the squared constraint `QED^2>2n` it also records is
  then satisfied by every surviving pair. Enumeration applies the same per-diagram
  filter (`diagrams/selector.rs`). No MadGraph comparison of a `>` card is banked.
- An order name the model does not define is refused at resolution. `EW=n` on the SM is
  accepted by MadGraph and constrains nothing there: MadGraph validates the aliased name
  `QED` but records the written `EW`, which no SM coupling carries. It is refused here.
- Squared-order constraints are in scope but not yet supported
  ([backlog](../backlog/feature/squared-order-constraints-refused.md)). How orders select
  diagrams, and the model's own caps, are in [coupling orders](../model/coupling-orders.md).

## Process numbers and commands

- **`@N`.** When absent, MadGraph numbers a line by the count of `generate`/`add process`
  lines in the history (L3309); here `SupportedProcess::id` is the `@N` or the line's
  position since the last `generate`. Each `@N` is one LHEF process (`LPRUP`, one `<init>`
  line each); see [generate](../events/generate.md). [^n38-e1]
- **`generate`** discards every earlier process (`clean_process`), then appends;
  **`add process`** appends. [^n38-commands]
- **Duplicates.** MadGraph raises `Duplicate process … found` (L3378) only for identical
  amplitudes, process number included: `generate p p > e+ e-` plus
  `add process u u~ > e+ e-` generates `u u~ > e+ e-` twice (also under one `@1`), and
  `--no_warning=duplicate` drops the second copy. Here any concrete subprocess reached from
  two lines is refused (`DiagramError::DuplicateSubprocess`), flag or not, because both
  would be added to σ. Lines that split a process by disjoint polarizations are accepted.
  `p p > w+ j` plus `add process p p > j w-` is not a duplicate: final states are grouped
  by content.
- **Mixed multiplicities.** Lines with different final-state multiplicities are accepted
  and summed, as MadGraph sums its `P<n>` directories; matching them is the run card's
  business. Different numbers of initial particles are refused, as in MadGraph.
- **`set`** lines that change physics (`complex_mass_scheme`, `group_subprocesses`,
  `ignore_six_quark_processes`, …) are refused off their defaults; benign ones are
  classified with a reason. Lines fed to `launch`'s questions (run- and param-card edits)
  are refused (`LaunchDialogue`): the cards are their own files.

## Default multiparticles and the five-flavour rewrite

MadGraph's defaults (`input/multiparticles_default.txt`):

```text
p  = g u c d s u~ c~ d~ s~        j = same
l+ = e+ mu+        l- = e- mu-
vl = ve vm vt      vl~ = ve~ vm~ vt~
```

On `import model`, `p` and `j` follow `add_default_multiparticles` [^mg-multiparticles]:
`b b~` is added when the model's b is massless and removed when it is massive, and the
photon is removed. The rewrite is replayed from the card's history, so a label defined
through `p` after the import sees it. Only the SM's `no_b_mass`, `no_masses` and
`zeromass_ckm` variants switch to five flavours; the SMEFTsim restrictions keep
`MB = 4.18`. Disabling the rewrite fails 21 census cards. [^n38-s2]

## Worked example: `generate e+ e- > mu+ mu-`

No modifier matches, so `orders = {}` and the automatic `WEIGHTED` search runs. The legs
are `e+` (−11) and `e-` (11) initial, `mu+` (−13) and `mu-` (13) final. In the SM there
are **two** diagrams, the s-channel photon and the s-channel Z; there is no t-channel
diagram for this process. [^n06-flow]

## Related

- [The card check](supported-card-check.md): what the AST may contain and still run.
- [s-channel restrictions](s-channel-restrictions.md): what `>`, `$$` and `$` do.
- [Decay chains](decay-chains.md): the `,` and `(…)` syntax and its decay assignment.
- [MadGraph5_aMC@NLO](../references/codebases/madgraph5-amcnlo.md): the pinned upstream.

[^n06-flow]: Note 06 §6–§7: MadGraph's data flow; the walk-through's diagram count is corrected here to two (γ and Z).
[^n06-edges]: Note 06 §9: `$$` before `$`, greedy matches, `=` semantics, case and whitespace.
[^n38-line]: Note 38 §1.1: modifier order from the back, leg precedence, restriction lists.
[^n38-commands]: Note 38 §1.4: `generate`, `add process`, duplicates, `set`.
[^n38-g1]: Note 38 §4, grammar, AST and the one check: the oracle, duplicates and decay assignment measured, and the corrections (only four operators, `]`-only spacing fix-up, `EW`, `/` by |PDG|, `set` after `launch`).
[^n38-s2]: Note 38 §4, `>` and `$$` as diagram filters: `WEIGHTED<=n` lifted, the `p`/`j` rewrite and its census.
[^n38-e1]: Note 38 §4, event records and `add process` completion: `@N` → `LPRUP`, `add process` grouped by content.
[^mg-extract-process]: `madgraph/interface/madgraph_interface.py` `extract_process`, L4822 (`proc_number_pattern` L4844, `order_pattern` L4883).
[^mg-orders]: `madgraph/interface/madgraph_interface.py` L4883–L5000: aliases, `^2` handling, `==`/`>` adding squared orders.
[^mg-do-add]: `madgraph/interface/madgraph_interface.py` `do_add`: decay-chain branch L3282, mixed initial states L3317, duplicates L3375–L3379.
[^mg-generate]: `madgraph/interface/madgraph_interface.py` `clean_process` L4791 and `do_generate` L4811.
[^mg-multiparticles]: `madgraph/interface/madgraph_interface.py` `add_default_multiparticles`, L5998; the `p`/`j` loop at L6042.
