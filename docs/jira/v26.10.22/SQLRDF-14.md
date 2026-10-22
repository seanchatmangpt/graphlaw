# SQLRDF-14 — §3.1 letter conformance: re-parent the 35 PEG leaf classes under the six family classes

- **Id:** SQLRDF-14
- **Title:** §3.1 letter conformance — re-parent the 35 PEG leaf classes under the six family classes
- **Status:** Open

> **This is a follow-up ticket, not part of the v26.10.22 acceptance set.** The
> v26.10.22 acceptance is SQLRDF-01..SQLRDF-12 and their courts. This ticket
> registers a known divergence and a recommended fix for a later window.

## Conformance target

The sql-rdf Recommendation (`docs/reference/sql-rdf-rec.md`), §3.1 — the
statement-class hierarchy: the six family classes
(`sql:DataQueryStatement`, `sql:DataManipulationStatement`,
`sql:DataDefinitionStatement`, `sql:TransactionStatement`,
`sql:SessionStatement`, `sql:ExtensionStatement`) sit between `sql:Statement`
and the PEG leaf classes, so every leaf has a family ancestor.

## Current behavior

The 35 PEG leaf classes are generated **flat under `sql:Statement`** in
`ggen/sql-dialect/statement_classes.ttl` (generator-marked,
`statement_classes.ttl:1`):

```turtle
sql:ExternalResourceStatement a owl:Class ;
    rdfs:subClassOf sql:Statement .
```

— all 35 leaves carry `rdfs:subClassOf sql:Statement` (verified: 35 matches of
`rdfs:subClassOf sql:Statement` in the file), with no family class in between.
Meanwhile the six family classes are declared in the canonical ontology
(`ontologies/sql-ast.ttl:30-40`), also as direct `rdfs:subClassOf sql:Statement`.

Functionally harmless today: `compile_turtle` merges `sql-ast.ttl` +
`statement_classes.ttl` for compilation, and both `statement_kind()` (string
match at `src/sql_ast.rs:1528` etc.) and the SHACL target asserted types only —
nothing walks the subClassOf chain at admission time.

**Blocking design decision — `sql:TransactionStatement` name collision.**
`sql:TransactionStatement` is BOTH a PEG leaf in the pack
(`ggen/sql-dialect/statement_classes.ttl:56`) and a family class in the
ontology (`ontologies/sql-ast.ttl:36`). Re-parenting as-is would require the
leaf to be a subclass of itself. The pack leaf needs a rename (e.g.
`sql:BeginTransactionStatement`) or an explicit `subClassOf` chain in the pack
that resolves the collision; this decision gates the whole re-parenting.

## Required behavior

1. **Resolve the `sql:TransactionStatement` collision** — rename the PEG leaf
   in the pack, or declare the leaf explicitly as the sole member/child of the
   family class in the pack.
2. **Add family mapping to the ontology source of truth**: map each of the 35
   leaves to its family in `ggen/sql-dialect/ontology/statements.ttl`
   (e.g. `sql:BeginTransactionStatement rdfs:subClassOf
   sql:TransactionStatement`).
3. **Update `ggen/sql-dialect/queries/classes.rq`** to select the family
   ancestor and emit `rdfs:subClassOf <family>` instead of
   `rdfs:subClassOf sql:Statement`.
4. **Edit the template** `ggen/sql-dialect/templates/statement_classes.ttl.tera`
   to render the family parent.
5. Regenerate:
   ```bash
   ggen sync run ggen/sql-dialect/ggen.toml
   ```
   and diff all three outputs (`statement_classes.ttl`,
   `sql_statement_kinds.rs`, `sql_extensions.rs`) — the diff must show the
   family re-parenting and nothing else.
6. Courts: a merged-graph query court in `tests/sql_rdf_dialect.rs` asserting
   every leaf has a family ancestor.

## Acceptance

```bash
ggen sync run ggen/sql-dialect/ggen.toml
git diff ggen/sql-dialect/statement_classes.ttl ggen/sql-dialect/sql_statement_kinds.rs ggen/sql-dialect/sql_extensions.rs   # re-parenting only
cargo test --test sql_rdf_dialect
```

Falsifier: a merged-graph query asserting each of the 35 leaves has a family
ancestor returns 35/35. On the current tree it returns 0/35 (all leaves point
directly at `sql:Statement`); after the fix it must return 35/35.
