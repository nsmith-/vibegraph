# 14 — egglog: Language Notes (Datalog + Equality Saturation)

**Status:** Reference summary (2026-07-11). Backs the planned egg/egglog rewrite stage
mentioned in `TODO.md` (performance-sprint P5 note: "the static-arity form egg requires"
and "further cuts need *algebraic* rewrites"). Source: `research/refs/papers/egglog.md`
(arXiv:2304.04332, Zhang, Wang, Willsey, Tatlock, Panchekha — PLDI 2023). Crate added to
`vibegraph-lib/Cargo.toml` as `egglog = "2.0.0"`.

---

## 0. One-paragraph summary

egglog unifies two fixpoint reasoning frameworks that were previously separate: **Datalog**
(bottom-up relational deduction over a fixed database) and **equality saturation / EqSat**
(term rewriting that keeps every version of a term, deduplicated via an e-graph, instead of
committing to one rewrite order). The unification works because both are naturally
"add-only" fixpoint computations over a database of facts. egglog's core move is to replace
Datalog's *relations* (sets of tuples) with *functions* (partial maps with a `:merge` policy
for conflicting outputs) and to give the user *uninterpreted sorts* whose values can be
`union`-ed — i.e. e-class ids, implemented with a union-find. A `datatype` declaration is
sugar for a sort plus one constructor-function per variant, so building an AST and looking
it up in a hash-consed table becomes ordinary Datalog fact insertion. This is directly
relevant to vibegraph's planned rewrite/CSE stage on the flattened HELAS expression IR: the
node tree (`Add`, `Mul`, ...) becomes an egglog `datatype`, and the current hand-rolled
hash-cons + forward-scan CSE pass becomes a small set of `rewrite` rules run to saturation.

---

## 1. Datalog background (§2.1, needed to read the rest)

A Datalog program is a set of rules, each a conjunctive query:

```
Q(x) :- R1(x1), R2(x2), ..., Rn(xn).
```

- `Q(x)` is the **head**, `Ri(xi)` are the **body** atoms.
- Every variable in the head must appear in the body.
- Running a rule = find all substitutions σ that satisfy every body atom, apply σ to the
  head, add the resulting fact.
- The set of all rules defines the **immediate consequence operator (ICO)** `T_p`; the
  program is run by iterating `T_p` from the empty database until it stops changing
  (a fixpoint). Termination and fixpoint-uniqueness are guaranteed for pure Datalog.
- Classic extension: **lattice-valued relations** — a relation becomes a function to a
  lattice `L = (C, ⊑, ⊔)`, and the rule's head value is the **join (⊔)** of every value the
  body can produce. This is the ancestor of egglog's `:merge`.

## 2. Equality saturation background (§2.2)

- An **e-graph** is a set of **e-classes**; each e-class is a set of equivalent **e-nodes**;
  an e-node is a function symbol with **e-class** children (not e-node children — this
  indirection is what makes the representation compact/exponential).
- Two terms are equivalent iff represented by the same e-class. The relation is congruent:
  if `f(a1,...,an)` and `f(b1,...,bn)` are represented and `ai ≡ bi` for all `i`, the e-graph
  can conclude `f(a...) ≡ f(b...)` too (congruence closure).
- EqSat fires *all* rewrite rules per iteration and only ever **adds** nodes/unions classes —
  it never deletes the original term. This sidesteps the phase-ordering problem of classic
  term rewriting (e.g. rewriting `(a×2)/2 → (a≪1)/2` locally is fine but forecloses
  cancelling the `2/2` later; EqSat keeps both `a×2` and `a≪1` in the same e-class).
- Prior extensions egglog subsumes: **e-class analyses** (attach a semilattice value to each
  e-class, propagated child→parent only, single analysis per e-graph, written in the host
  language — this is egg's mechanism); **multi-patterns** (match several patterns
  simultaneously, e.g. TenSat's matmul-sharing rule); **relational e-matching** (Zhang et al.
  2022 — reduce e-matching to a relational query using a worst-case-optimal join; egglog's
  query engine descends directly from this, without the "dual representation" copying cost
  that a bolt-on e-graph-to-database translation pays).

## 3. The egglog language (§3) — by example

### 3.1 Datalog mode

```lisp
(relation edge (i64 i64))
(relation path (i64 i64))

(rule ((edge x y)) ((path x y)))
(rule ((path x y) (edge y z)) ((path x z)))

(edge 1 2) (edge 2 3) (edge 3 4)
(run)
(check (path 1 4))     ; succeeds
```

A `(rule (query...) (actions...))` is egglog's basic unit — **query first, actions
second**, the reverse of textbook Datalog's `head :- body`. The query is a list of
*patterns* that must all match (binding pattern variables); the actions run once per match
and typically assert new facts. `relation` declares a Datalog-style predicate: sugar for a
function returning the built-in unit type (see §3.2).

### 3.2 `function` + `:merge` — relations generalize to partial maps

egglog's actual primitive is not the relation but the **function**: every table is backed by
a **map**, not a set, enforcing a functional dependency from inputs to output. A `relation`
desugars to `(function ... unit)`. Because functions can be redefined for the same input
(directly via `set`, or indirectly when a `union` makes two previously-distinct inputs
equal), every function needs a **`:merge` expression** telling egglog how to reconcile the
old and new output values:

```lisp
(function edge (i64 i64) i64)
(function path (i64 i64) i64 :merge (min old new))

(rule ((= (edge x y) len)) ((set (path x y) len)))
(rule ((= (path x y) xy) (= (edge y z) yz))
      ((set (path x z) (+ xy yz))))

(set (edge 1 2) 10) (set (edge 2 3) 10) (set (edge 1 3) 30)
(run)
(check (path 1 3))     ; prints 20 — shortest path wins via :merge = min
```

`old`/`new` are bound inside the `:merge` expression body. This is the shortest-path lattice
(`⊑` = `≥`, join = `min`) expressed directly as a merge policy — no separate lattice
machinery needed.

### 3.3 `sort` + `union` — user-defined equality (this is where e-graphs enter)

Base types (`i64`, `String`, ...) cannot be unioned — they denote themselves. A `(sort Name)`
declaration introduces a fresh **uninterpreted sort**: a set of opaque integer ids plus a
union-find over them. `union` merges two ids of the same sort into one equivalence class;
egglog keeps the *database* canonicalized against this union-find (all stored ids are
canonical representatives), which is exactly what makes querying "modulo equality" free.

```lisp
(sort Node)
(function mk (i64) Node)
(relation edge (Node Node))
(relation path (Node Node))
;; ... same path rules as above ...
(edge (mk 1) (mk 2)) (edge (mk 2) (mk 3)) (edge (mk 5) (mk 6))
(union (mk 3) (mk 5))          ; vertex contraction
(run)
(check (path (mk 1) (mk 6)))   ; true only because of the union
```

Every function has a **`:default`** expression used to make it total: calling `(f x)` when
`x` has no entry evaluates `:default`, stores the result, and returns it. For functions
returning a user-defined sort, the implicit default is "make a fresh id" (union-find
`make-set`); for base types, the implicit default is to crash (must be set explicitly or
supplied via `:default`).

### 3.4 `datatype` + `rewrite` — this *is* equality saturation

```lisp
(datatype Math
  (Num i64) (Var String) (Add Math Math) (Mul Math Math))

(define expr1 (Mul (Num 2) (Add (Var "x") (Num 3))))
(define expr2 (Add (Num 6) (Mul (Num 2) (Var "x"))))

(rewrite (Add a b) (Add b a))
(rewrite (Mul a (Add b c)) (Add (Mul a b) (Mul a c)))
(rewrite (Add (Num a) (Num b)) (Num (+ a b)))
(rewrite (Mul (Num a) (Num b)) (Num (* a b)))

(run)
(check (= expr1 expr2))
```

- `(datatype T (C1 ...) (C2 ...) ...)` desugars to `(sort T)` plus one `function` declaration
  per constructor, each returning `T`. Constructors' implicit `:merge` is `union` — i.e. if
  the same constructor call is ever mapped to two different ids (which happens once
  congruence closure forces it), those ids get unioned. Combined with database
  canonicalization, this *is* congruence closure: the built-in equivalence relation becomes a
  congruence w.r.t. every constructor for free.
- `(define x e)` desugars to a nullary function + `set`: `(function x () T) (set (x) e)`.
  Evaluating `e` inserts every subterm into the database via the constructors' `:default`
  (fresh-id) behavior — this is how nested terms populate tables without an explicit
  Datalog-style bottom-up derivation.
- `(rewrite p1 p2)` desugars to `(rule ((= __v p1)) ((union __v p2)))` — match `p1`, bind it,
  union with `p2`. **Rewriting is non-destructive** (only adds facts/unions, never deletes)
  and **matching is modulo equality** (queries run against the canonicalized database) — the
  two properties that define EqSat, both falling out of ordinary egglog semantics with no
  special-casing.
- Crucially, egglog uses the *same* rule mechanism for uninterpreted rewrites (`Add a b →
  Add b a`) and for rules that compute with interpreted values (`Add (Num a) (Num b) → Num
  (+ a b)`, calling the built-in i64 `+`). Tools like egg require splitting these into
  separate rewrite rules vs. e-class-analysis code in the host language; egglog needs only
  rules.
- `extract` (mentioned in §3.4, detailed via cost discussion in Appendix A.4) returns the
  smallest/cheapest term represented by a given e-class — the payoff step after saturation.

### 3.5 Beyond both: the "demand" pattern (Appendix A.1)

Because egglog can create a fresh id as a **placeholder** for `(f x)` before `f`'s value at
`x` is known (via `:default`), and subsequent rules can later resolve that id via `union`, it
naturally expresses top-down/on-demand computations that plain bottom-up Datalog needs a
manual "demand relation" to simulate (Soufflé-style demand transformation, shown side by side
in the paper). This is a genuine third mode beyond "Datalog" and "EqSat": using ids as logic
variables representing *not-yet-known* information, à la Prolog/miniKanren — but strictly
monotonic, no backtracking.

## 4. Summary of ways to define relations/tables

| Form | Backing | Equality-aware? | Use when |
|---|---|---|---|
| `(relation R (T1 ... Tn))` | map to unit (sugar for `function`) | only if `Ti` are sorts | pure fact membership, Datalog-style |
| `(function f (T1...) Tout :merge E)` | partial map, functional dependency enforced | if any `Ti`/`Tout` is a `sort` | analyses with a resolvable conflict policy (lattice join, `min`/`max`, arbitrary expr over `old`/`new`) |
| `(sort S)` | union-find over opaque ids | yes, by definition | any type you want to unify (e-class ids) |
| `(datatype T (C1 T1...) ...)` | `sort T` + one constructor-`function` per variant, `:merge` = `union` | yes | AST / term representation for EqSat; congruence closure is automatic |
| `(define x e)` | nullary `function` + `set` | inherits from `e`'s type | naming a term / seeding the database with a specific expression |

`:default` controls what happens on a **miss** (get-or-make-set for sort outputs; crash for
base-type outputs unless supplied); `:merge` controls what happens on a **conflict** (two
different asserted/derived values for the same input tuple — from an explicit `set` or from a
`union` that aliases previously-distinct inputs). Both are per-function, arbitrary egglog
expressions over bound variables (`old`, `new` for merge) — not restricted to lattice joins,
which is strictly more expressive than prior Datalog-with-lattices systems (Flix, Ascent).

## 5. Formal core syntax (§4.1, Fig. 5)

Core egglog (the subset the paper gives a fixpoint semantics for — full surface syntax
desugars into this):

```
Program   P ::= R1, ..., Rn
Rule      R ::= A :- A1, ..., Am
Atom      A ::= f(p1,...,pk) ↦ o | f(p1,...,pk)
Pattern   p ::= f(p1,...,pk) | o
Term      t ::= f(t1,...,tk) | v
Base pat  o ::= v | x
Constant  v ::= c | n          (c: interpreted constant, n: uninterpreted/id)
```

One head atom per rule, no `union` primitive (desugars away) — `:merge` in core is either
`union` (id outputs) or a lattice join (interpreted-constant outputs); full egglog allows
`:merge` to be any expression, and multiple actions per rule.

### Semantics sketch (§4.2–4.3)

Evaluation alternates two operators to a fixpoint:

1. **Inflationary immediate consequence** `T_P↑`: fire all rules against the current
   canonicalized instance, union the newly derived facts into the database (inflationary —
   old facts are never dropped, unlike monotone-only classic Datalog, because egglog rules
   can be non-monotonic, e.g. rules that inspect a value that later increases under `:merge`).
2. **Rebuilding** `R^∞`: whenever a `union` (directly, or as a `:merge` side effect) makes two
   ids equivalent, every function entry keyed by a now-non-canonical tuple must be
   re-canonicalized; this can itself create new functional-dependency conflicts, resolved by
   invoking `:merge` again, iterated to a fixpoint. For `:merge = union` this rebuilding
   procedure *is* congruence closure (Downey et al. 1980, same lineage as egg's rebuilding).

**Semi-naïve evaluation** (§4.3): standard Datalog trick, carried over — each round only
joins against the *delta* of facts added last round (`ΔDB_i`), avoiding rediscovering the
same derivations repeatedly. egglog gets this "for free" from its Datalog-style query engine;
incremental e-matching is otherwise rare (some SMT solvers only).

## 6. Implementation notes (§5)

- **Functional (map-backed) database**, not relational (set-backed) — required for the
  get-or-default term-construction pattern (`:default`) to be efficient, and for detecting
  the first kind of functional-dependency conflict (explicit `set` collision) cheaply.
- **Query engine** = relational e-matching (Zhang et al. 2022) via **Generic Join**
  (worst-case-optimal multi-way join, Ngo et al.), further sped up over that prior work
  because egglog *is* the database already — no copying into/out of a separate e-graph
  structure to e-match, and semi-naïve evaluation comes for free.
- ~4,200 LOC Rust in the original `egg-smol` implementation (predecessor name of the
  `egglog` crate). Designed language-first (text format + library), unlike egg which is
  Rust-library-only.

## 7. Case studies (§6, brief)

- **Points-to analysis** (Steensgaard-style, unification-based): egglog's native union-find
  sort is a direct fit for what cclyzer++ had to hand-roll a bespoke union-find for on top of
  Soufflé. 4.96× faster than the Soufflé baseline.
- **Herbie** (floating-point rewriting): egglog lets Herbie replace *unsound* rewrites
  (guarded only by post-hoc validation, e.g. `x/x → 1` applied even when `x` might be 0) with
  Datalog-style side-condition rules computed *during* saturation, soundly and faster.

## 8. Relevance to vibegraph

`TODO.md` (performance-sprint P5, `research/notes/13-typed-repr-conventions-design.md`)
already frames the current binary-arity flattened node IR + hash-cons CSE as groundwork for
an "egg" stage; this paper is the design reference for that stage, and confirms the shape:

- The lowered expression tree (`Add`, `Mul`, kernel-fusion nodes, ...) maps directly onto a
  `datatype` declaration — one constructor-function per node variant, sort = e-class id.
  Congruence closure (deduplicating structurally-identical subexpressions across diagrams,
  which the current hash-cons CSE pass does by hand) is then automatic.
- The physics-specific algebraic identities the note flags as needing an "algebraic" rewrite
  stage beyond current CSE (constant folding, coupling-constant regrouping, chiral-pair
  fusion patterns already hand-coded in `kernel_fusion`/`ff...` fused vertices) become
  `rewrite` rules run to saturation, instead of hand-written peephole passes over the node
  tree — the same trade a peephole optimizer makes vs. a proper term-rewriting system.
  Fused-kernel dispatch (currently a hard-coded pattern match, see the chiral-pair FFV
  fusion committed in `283d164`) is a natural `rewrite` target: match the general node
  pattern, replace with the fused-kernel node, extract by cost so the fused form always wins.
- Preferring the fused/kernel form over the general form should be encoded as an extraction
  **cost** policy, not via `:merge`: the crate's `:cost` annotation (on constructors/functions,
  not covered in the 2023 paper — see caveat below) is the mechanism `egglog` v2.0.0 actually
  provides for cost-based extraction. `:merge` resolves functional-dependency conflicts during
  saturation; it is the wrong tool for picking a winner among several equivalent e-class members
  after the fact.
- **Caveat:** this note summarizes the *paper* (arXiv:2304.04332, describing the `egg-smol`
  prototype). The `egglog` crate actually vendored (`Cargo.toml`, v2.0.0) is a substantially
  later, actively-developed implementation and has surface syntax not covered here (e.g.
  `:cost`/`:unextractable` annotations, `ruleset`/`run-schedule`/`saturate` scheduling,
  `subsume`, ADT `constructor`/`declare` forms per `egglog-ast` — confirmed present in the
  vendored crate source under `~/.cargo/registry/src/.../egglog-ast-2.0.0`). Before designing
  the rewrite-stage schedule, check the crate's own docs/examples rather than assuming the
  2023 paper syntax is current.

## References

- Paper: Zhang, Wang, Willsey, Tatlock, Panchekha, "Better Together: Unifying Datalog and
  Equality Saturation," PLDI 2023. arXiv:2304.04332. Local copy:
  `research/refs/papers/egglog.md` (fetch via `research/refs/fetch-papers.sh egglog`).
- Prior EqSat: egg (Willsey et al., PLDI 2021) — e-class analyses, the tool egglog subsumes.
- Prior relational e-matching: Zhang et al., "Relational E-Matching," 2022 — the Generic Join
  technique egglog's query engine builds on.
- Crate: `egglog` v2.0.0 on crates.io, added to `vibegraph-lib/Cargo.toml`.
