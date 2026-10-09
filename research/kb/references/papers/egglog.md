---
type: Paper
title: "egglog: unifying Datalog and equality saturation"
description: "arXiv:2304.04332 (Zhang et al., PLDI 2023): Datalog and EqSat background, the language, its fixpoint semantics and implementation, and where the egglog 2.0.0 crate departs from the paper."
resource: "https://arxiv.org/abs/2304.04332"
status: draft
tags: [egglog, e-graphs, equality-saturation, datalog, paper]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n14, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/14-egglog-notes.md#L11-L322", title: "Note 14, egglog language notes (paper summary)"}
  - {id: cargo, resource: "vibegraph-lib/Cargo.toml#L50", title: "egglog = \"2.0.0\""}
  - {id: egraph-rs, resource: "vibegraph-lib/src/helas/eval/egraph.rs#L1-L33", title: "egraph.rs module docs: the identity round trip"}
---

"Better Together: Unifying Datalog and Equality Saturation" (Zhang, Wang,
Flatt, Cao, Zucker, Rosenthal, Tatlock, Willsey; PLDI 2023). egglog joins two
fixpoint frameworks that were separate: Datalog (bottom-up deduction over a
database of facts) and equality saturation (term rewriting that keeps every
equivalent form in an e-graph instead of committing to one rewrite order). Both
are add-only fixpoint computations over a database, which is what makes the
union work[^n14].

## Background the paper assumes

**Datalog.** A rule is a conjunctive query `Q(x) :- R1(x1), …, Rn(xn)`; every
head variable appears in the body. Running the rules to a fixpoint from the
empty database (the immediate consequence operator) terminates with a unique
result. Lattice-valued relations generalise a relation to a function into a
lattice whose head value is the join of what the body produces; this is the
ancestor of egglog's `:merge`.

**E-graphs and EqSat.** An e-graph is a set of e-classes, each a set of
equivalent e-nodes whose children are e-classes, not e-nodes; that indirection
represents exponentially many terms compactly. Equivalence is congruent: if
`aᵢ ≡ bᵢ` for all `i` then `f(a…) ≡ f(b…)`. EqSat fires every rewrite each
round and only adds nodes or merges classes, never deletes, which removes the
phase-ordering problem (`(a×2)/2` keeps both `a×2` and `a≪1`, so the `2/2`
can still cancel). egglog subsumes e-class analyses (egg's per-class semilattice
values), multi-patterns, and relational e-matching (e-matching as a relational
join).

## The language

- `(relation R (T…))` is a Datalog predicate; it is sugar for a function to
  the unit type. `(rule (query…) (actions…))` puts the query first, the reverse
  of `head :- body`.
- `(function f (T…) Tout :merge E)` is the real primitive: a partial map with a
  functional dependency. `:merge` (over `old` and `new`) resolves a conflict
  when two values are asserted for one input, whether by `set` or because a
  `union` made two inputs equal. With `:merge (min old new)` the shortest-path
  program is a few lines.
- `(sort S)` declares an uninterpreted sort: opaque ids under a union-find.
  `union` merges ids, and the database is kept canonical with respect to the
  union-find, so queries run modulo equality.
- `:default` makes a function total: a miss on a sort-valued function makes a
  fresh id; a miss on a base-typed one is an error unless a default is given.
- `(datatype T (C1 …) …)` is a sort plus one constructor function per variant,
  each with `:merge` = `union`. With canonicalisation this *is* congruence
  closure. `(define x e)` is a nullary function plus `set`, and inserting `e`
  inserts every subterm.
- `(rewrite p1 p2)` desugars to `(rule ((= v p1)) ((union v p2)))`. Rewriting
  is non-destructive and matching is modulo equality, the two defining
  properties of EqSat, with no special case. The same mechanism serves
  uninterpreted rewrites (`Add a b → Add b a`) and computing ones
  (`Add (Num a) (Num b) → Num (+ a b)`), which egg splits between rules and
  host-language analyses.
- `extract` returns the cheapest term of an e-class, the payoff after
  saturation (the paper's Appendix A.4 discusses costs).
- Ids created by `:default` can stand for not-yet-known values that later rules
  `union` away: a top-down "demand" mode, monotonic, with no backtracking.

| Form | Backing | Equality-aware | Use |
|---|---|---|---|
| `relation` | map to unit | if its argument types are sorts | fact membership |
| `function … :merge` | partial map | if an argument or the output is a sort | analyses with a conflict policy |
| `sort` | union-find over ids | yes | anything to be unified |
| `datatype` | sort plus constructors, `:merge union` | yes | terms for EqSat |
| `define` | nullary function plus `set` | as its type | naming a seeded term |

## Semantics and implementation

The core language (§4.1) has one head atom per rule and no `union`; `:merge` in
the core is `union` for id outputs or a lattice join. Evaluation alternates two
operators to a fixpoint: an inflationary immediate-consequence step that fires
all rules against the canonical database, and **rebuilding**, which
re-canonicalises entries keyed by ids a `union` made non-canonical and resolves
the resulting conflicts with `:merge`, repeatedly. For `:merge union`,
rebuilding is congruence closure. Semi-naïve evaluation joins only against the
facts added in the previous round, so e-matching is incremental for free.

The database is map-backed, which makes get-or-default term construction and
conflict detection cheap. The query engine is relational e-matching via
Generic Join (worst-case-optimal), faster than bolting an e-graph onto a
database because no data is copied between the two. The original `egg-smol`
implementation was about 4 200 lines of Rust, designed language-first.

Case studies (§6): a Steensgaard-style points-to analysis, 4.96× faster than
the Soufflé baseline because egglog's union-find replaces a hand-built one; and
Herbie, whose unsound post-validated rewrites (`x/x → 1` with `x` possibly 0)
become sound side-condition rules computed during saturation.

## The crate is not the paper

The `egglog` crate vibegraph depends on (`2.0.0`) is a later implementation
with syntax the paper does not cover: `:cost` and `:unextractable` annotations,
`ruleset` / `run-schedule` / `saturate` scheduling, `subsume`, and
`constructor` declarations. Write rules against the crate's own documentation
and examples, not the paper's syntax. Choosing a preferred form among
equivalent ones is an extraction-cost question (`:cost`), not a `:merge` one:
`:merge` resolves conflicts during saturation and does not pick a winner
afterwards[^n14].

## Relevance to vibegraph

`helas/eval/egraph.rs` declares the lowered binary-arity expression tree as an
egglog datatype (one constructor per op, with leaf payloads as base-sort
fields), inserts a whole tree in one `let`, and extracts it back unchanged: no
rewrite rules are registered, so it is an identity round trip[^egraph-rs]. No
production path consumes it. Greedy extraction over the slot-traffic cost model
could not realise the sharing payoff on these amplitudes, so the rewrite stage
waits on a global (ILP-style) extractor with a compute-aware cost model
([egglog rewrite stage](../../performance/egglog-rewrite-stage.md),
[DAG extraction](../../performance/egraph-dag-extraction.md),
[egraph-sharing-rewrites-need-global-extractor](../../backlog/performance/egraph-sharing-rewrites-need-global-extractor.md)).
The hash-consing CSE in `lower.rs` does the congruence-closure part today.

[^n14]: Note 14, egglog language notes; authors confirmed on arXiv.
[^egraph-rs]: `vibegraph-lib/src/helas/eval/egraph.rs`, module docs.
