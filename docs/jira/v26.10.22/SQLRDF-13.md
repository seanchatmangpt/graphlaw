# SQLRDF-13 — Validation-scoped RDFS closure for §4 sh:class constraints

- **Id:** SQLRDF-13
- **Title:** Validation-scoped RDFS closure for §4 `sh:class` constraints
- **Status:** Open

> **This is a follow-up ticket, not part of the v26.10.22 acceptance set.** The
> v26.10.22 acceptance is SQLRDF-01..SQLRDF-12 and their courts. This ticket
> registers a known divergence and a recommended fix for a later window.

## Conformance target

The sql-rdf Recommendation (`docs/reference/sql-rdf-rec.md`), §4 — the SQL AST
ontology and its SHACL shapes (`ontologies/sql-ast.ttl`). The shapes are
normative for what a `sql` document must look like, and the admission law
(`src/law.rs`) is normative for when a document is refused with a typed
diagnostic.

## Current behavior

The shapes at `ontologies/sql-ast.ttl:315-330` withhold two normative
constraints. In the `sql:SelectStatementShape` the `sql:where` property shape
carries only `sh:maxCount 1`; the intended `sh:class sql:Expression` is
commented out with the note:

```
# Alias window: sh:class sql:Expression is withheld — data documents do not
# carry the ontology's rdfs:subClassOf axioms, so the validator's asserted
# type closure would reject normative sql:BinaryExpression values.
```

The same withholding applies to `sh:class sql:ASTNode` on the
`sql:BinaryExpressionShape` operands (same block, `ontologies/sql-ast.ttl:315-330`).

Root cause: `purrdf-shapes` reasons over the **asserted** data graph only. A
data document states `rdf:type sql:BinaryExpression`; the ontology states
`sql:BinaryExpression rdfs:subClassOf sql:Expression .`, but that axiom lives
in a different graph, so the validator's type closure never contains
`sql:Expression` and the constraint would reject every normative document.

Two call sites currently consume the asserted-only validation:

- `src/sql_ast.rs:1128-1160` — `validate_ast_shacl` parses the AST dataset
  with `purrdf::parse_dataset`, parses the shapes with
  `crate::shacl::engine::parse_shapes`, and validates the dataset alone.
- `src/law.rs:849-852` — the law-side path parses shapes over
  `self.dataset` and validates it, refusing with `LawError::NotAdmitted` on
  non-conformance.

Because `sh:class` on `sql:where` is withheld, a document whose `sql:where`
points at a literal (e.g. `"x"^^xsd:string` as the object of `sql:where`)
passes the cardinality check and fails later — a deep `extract`/compile error
instead of a SHACL diagnostic.

## Required behavior

Restore the two withheld constraints — `sh:class sql:Expression` on
`sql:where`, `sh:class sql:ASTNode` on the `BinaryExpressionShape` operands —
without widening the divergence between what the shapes say and what the
validator can enforce.

**Recommended fix (Option 2 from the analysis): validation-scoped closure.**
At each validation call site, build a throwaway dataset that unions the data
document with the ontology's `rdfs:subClassOf` axioms, run
`purrdf::entail::materialize` with the `Rdfs` entailment regime (or a
subClassOf-only subset), validate the materialized closure, and discard the
result. Critically, this closure is scoped strictly **away from `LawState`**:
`LawState`'s identity is the canonical BLAKE3 of its admitted graph, so the
materialized closure must never be folded into `LawState` — it exists only as
a transient input to `validate_dataset` and is dropped after the report.

Concretely:

1. `ontologies/sql-ast.ttl` — un-withhold the two `sh:class` constraints
   (remove the alias-window comments).
2. `src/sql_ast.rs` `validate_ast_shacl` — before `validate_dataset`, merge
   the shapes/ontology axioms into a temp dataset, materialize RDFS, validate
   the closure, discard.
3. `src/law.rs` law-side path — same throwaway-closure validation; the
   admitted graph stored in `LawState` remains exactly the data document
   (canon BLAKE3 unchanged).
4. Courts: extend the existing SHACL courts to cover the restored constraints.

## Acceptance

A real court in `tests/sql_rdf_dialect.rs`:

```bash
cargo test --test sql_rdf_dialect
```

Falsifier: a `sql` document whose `sql:where` points at a literal must be
refused at admission with a SHACL diagnostic (`AstError::ShaclViolation` /
`LawError::NotAdmitted` with the sh:message), not a deep `extract`/compile
error. Today it is not — that court must fail on the current tree and pass
after the fix.
