# E2 Design — Generate the Property-Path Reader from the SHACL Shapes

- **Id:** E2 (moonshot, `docs/jira/v26.10.22/SQLRDF-16.md`; standing: BLOCKED-BY v26.10.22 integration commit)
- **Status:** Design (read-only lane output; authoritative version)
- **Supersedes:** the inline lane draft's §2 drafting artifacts — the annotation table below is the single authoritative statement of the vocabulary.
- **Thesis:** shapes are the single grammar description. `ontologies/sql-ast.ttl` describes the AST vocabulary as SHACL; `src/sql_ast.rs` re-describes it as hand-written extraction code. E2 compiles the second copy from the first.
- **Pipeline to mirror:** `ggen/sql-dialect/` — `extract_statements.py` → `ontology/statements.ttl` → `queries/statements.rq` → `templates/kinds.tera` → `sql_statement_kinds.rs`, wired by `ggen/sql-dialect/ggen.toml`.

## 1. Current extract_* surface (inventory)

All anchors in `src/sql_ast.rs`:

| Function | Line | Role |
|---|---|---|
| `extract_expr` | :600 | legacy entry; permissive `ScopePos::SelectList` |
| `extract_expr_scoped` | :611 | core: discriminator dispatch + scope integrity (REFUSAL-01/03) |
| `extract_table_ref` | :932 | `sql:tableName/tableAlias/schemaName` → `TableRef` |
| `extract_from` | :952 | `sql:from` list vs legacy `sql:fromTable`; both = REFUSAL-01 |
| `extract_select_statement` | :983 | clause walk: joins, select, where, groupBy, having, orderBy+direction, limit/offset |
| `extract_merge_statement` | :1156 | targetTable/usingSource/mergeOn/whenMatched/setAssignments |
| `unroll_rdf_list` | :563 | bounded (64) cycle-checked rdf:List walk |
| `single_object_checked` | :549 | cardinality gate (>1 = REFUSAL-01) |
| `read_operator` | :916 | token enum or legacy axiom-IRI form |
| `finish_alias` | :892 | `sql:alias` attribute wrapper |

Hand-written trait impl: `DatasetReader` (:1365). Fallback layer: `compile_expression_form` (:1483) / `compile_command` (:1906).

**Property reads (predicate inventory):** `sql:scope, columnName, literalValue, value, aliasName, alias, function, functionName, arguments, argument, operator, left(+leftOperand), right(+rightOperand), select(+selectList), from(+fromTable), where(+whereClause), groupBy, having(+havingClause), orderBy, direction, limit, offset, joinClause, joinType, joinTable, joinCondition, tableName, tableAlias, schemaName, statementIndex, targetTable, usingSource, mergeOn, whenMatched, setAssignments, targetColumn, valueExpression`, plus `rdf:first/rdf:rest`.

**Cardinality discipline:** `single_object_checked` (clause predicates single-cardinality, duplicate = REFUSAL-01, never document-order-wins); ambiguity gate (:660-681, max one expression discriminator per node); alias-window "never both forms" refusals (:957, :1017, :1037, :1064, :836, :842).

**Scope threading:** `ScopePos` (:172) — SelectList/OrderBy/Having/Predicate/GroupBy/AggregateArgument — threaded through every `extract_expr_scoped` call; `FunctionCall` argument → `AggregateArgument` (:788, :817); alias wrapper inherits pos. Enforcement: `allows_aggregate`/`is_view_surface` (:181-192), `check_aggregate_position` (:881), REFUSAL-03 column-scope gates (:698-716).

**NOT derivable from shapes alone:** (a) scope positions — SHACL cannot say "this clause's children extract in position X"; (b) expression recursion — the depth-32 descent (:617, :764, :788, :817, :866-867) is an algorithm, not a shape; (c) literal typing — the xsd:*→`SqlLiteral` map (:731-742) and the malformed-integer-is-REFUSAL-01 law (:1085-1099, :1110-1138); also identifier validation (:48, :287) and optional-with-fallback semantics (`compile_expression_form`, :1483).

## 2. Annotation vocabulary — the seven annotations (authoritative)

One `sql:ScopePosition` class-family (individuals mirroring `ScopePos`: SelectList, OrderBy, HavingPos, Predicate, GroupByPos, AggregateArgument) plus seven properties on `sh:PropertyShape` blocks, appended to `ontologies/sql-ast.ttl` (reuse the `sql:` prefix; no second namespace):

| # | Annotation | Values | Derives (replaces hand-written) |
|---|---|---|---|
| 1 | `sql:scopePosition` | one of the 6 individuals | the clause→ScopePos binding hard-coded in `extract_select_statement` (:1003, :1031, :1043, :1054, :1070, :1082) |
| 2 | `sql:childScopePosition` | `AggregateArgument` (only on aggregate `sql:argument`/`sql:arguments` blocks) | the arg-position switch (:788, :817) |
| 3 | `sql:readerHint` | `column` \| `literal` \| `aliasWrap` \| `aggregateIri` \| `aggregateName` \| `binary` \| `tableRef` | the discriminator dispatch (:660-681) |
| 4 | `sh:defaultValueType` (existing SHACL) | `xsd:integer` \| `xsd:double` \| `xsd:boolean` | the xsd:*→SqlLiteral map (:731-742) and typing fallback when datatype is absent |
| 5 | `sql:cardinality` | `single-checked` \| `list-64` \| `optional` | `single_object_checked` vs `unroll_rdf_list(…,64)` vs `get_datatype_literal().is_some()` choice |
| 6 | `sql:recursesInto` | target shape | the hard-coded recursion (:764, :788, :817, :866-867) |
| 7 | `sql:posPropagates` / `sql:posResets` | bool / individual | operand-inherit vs reset at recursion edges |

**Declaration sweep (court prerequisite):** every predicate the reader reads must be declared in an `sh:property` block. Today's repo fails this: `sql:schemaName` (read :944) and `sql:direction` (read :1085) are undeclared, as is the entire `compile_command`/`compile_expression_form` vocabulary (:1483-1762: tableDefinition, columns, columnType, ifExists, dropTarget, alterAction, truncateTarget, analyzeTarget, comment*, copy*, export/importPath, prepare/execute/deallocateTarget, callTarget, pragmaName, configKey/Value, resetTarget, transactionAction, load/installTarget, extensionName, attach/detachTarget, databasePath, attachAlias, useTarget, repositoryTarget/Name/Prefix, publicKey). The declaration sweep is E2's first deliverable.

**Worked example (section 7 MERGE shape):**
```turtle
sql:MergeIntoStatementShape sh:property
  [ sh:path sql:usingSource ; sql:cardinality "single-checked" ; sql:recursesInto sql:SelectStatementShape ] ,
  [ sh:path sql:mergeOn ; sql:scopePosition sql:Predicate ; sql:recursesInto sql:Expression ] ,
  [ sh:path sql:whenMatched ; sql:cardinality "single-checked" ] ;  # → setAssignments list-64 of
  #   { sql:targetColumn (single-checked, identifier) ; sql:valueExpression (sql:scopePosition sql:Predicate, sql:recursesInto) }
```

## 3. ggen pipeline shape

New project `ggen/sql-reader/` mirroring the statement-kinds rule:

```
ggen/sql-reader/
  ggen.toml                     # ontology = shapes + annotations graph
  ontology/reader-annotations.ttl
  queries/reader_rules.rq       # ?shape ?path ?hint ?card ?pos ?childPos ?recurses ORDER BY ?ord
  queries/coverage.rq           # every shape-declared (shape, path) pair — court input
  templates/reader.tera         # match arms keyed on (readerHint, scopePosition)
  templates/coverage_table.rs.tera  # READER_COVERAGE: &[(&str, &str)]
  outputs (mode = "Overwrite"):
    src/sql_reader_gen.rs       # generated reader module
    src/sql_reader_coverage.rs  # generated coverage table (court input)
```

Generated module shape: rows fold into a recursive `match` on `readerHint` — hint=`column` → identifier-validated ColumnRef arm; hint=`literal` → literal arm typed via `sh:defaultValueType` (replacing the map at :731-742); hint=`aliasWrap`/`aggregateIri`/`aggregateName`/`binary` → the five SqlExpr constructors with pos/childPos threaded from the row (*pos threading is data, not code*); card=`single-checked` → `single_object_checked`; card=`list-64` → `unroll_rdf_list(…,64)`.

Closure property: `compile_turtle` already `include_str!`s the shapes (`src/sql_ast.rs:1319-1323`). The generated module is built from, and the court reads, the same embedded shapes bytes: same-bytes shapes → SHACL validation → generated reader → coverage court. That closure *is* the single-grammar-description property.

## 4. Hand-written residue and the enforced split

Irreducible residue (stays hand-written):
1. **Expression recursion** — the descent algorithm, depth bound 32, cycle detection.
2. **The scope lattice** — `allows_aggregate`/`is_view_surface` (:181-192), REFUSAL-03 gates (:881-888, :698-716). The clause→pos *binding* becomes data; the lattice *law* stays code.
3. **MERGE composition** — `compile_merge` (:1235): param-buffer merge, sub-select embedding.
4. `compile_command`/`compile_expression_form` (:1483-2051) — scalar-template fallback; E2 scopes to the expression reader; second generated module later.
5. `validate_identifier`/`validate_qualified_identifier` (:48, :287) — law, not grammar; generated code calls it.

**Module boundary:** `sql_reader_gen.rs` may reference only the kernel (`RdfAstGraphReader`, `ScopePos`, `validate_identifier`, `AstError`/typed refusals, `SqlExpr`/`SqlLiteral`). It may not mention any `sql:*` property IRI outside its generated coverage table, and may not redefine cardinality or scope law. Law in kernel, grammar in table.

**Coverage court** (`scripts/sql_reader_coverage_court.sh`, modeled on `scripts/mutation_court.sh` — GUARDED set, `--check` exit 3, snapshot/restore). Note: no `sdk_coverage` court exists in this repo; `mutation_court.sh` is the operative local model. Three checks:
1. **Declaration** — every reader-read property IRI appears in some `sh:path` (fails against today's tree; the sweep fixes it).
2. **Generated-covers-shapes** — every `(shape, path)` in the TTL appears in the generated `READER_COVERAGE` table.
3. **No-residue** — property IRIs in `src/sql_ast.rs` minus the generated coverage table = residue = FAIL (exit 3). Grep-based set difference; no engine needed.

## 5. Falsifier

Delete the `sql:orderBy` property block from `ontologies/sql-ast.ttl`, re-run `ggen sync run`, re-run the coverage court. `sql:orderBy` vanishes from `coverage.rq` and the regenerated reader silently drops ORDER BY — silent miscompilation is the failure mode E2 exists to prevent, so the court must flip. Check 2 catches it directly; check 3 catches hand-written residue still reading the deleted IRI. Exit 3 under `--check`. Anti-vacuity (window law, `CLAUDE.md:24`): the court itself must be mutation-tested — delete a coverage-table row → court fails.

## Anchor summary

| Artifact | Path |
|---|---|
| Reader trait + extract_* | `src/sql_ast.rs` (trait :536, extract_expr_scoped :611, clause walk :983, MERGE :1156) |
| Shapes TTL | `ontologies/sql-ast.ttl` |
| Pipeline to mirror | `ggen/sql-dialect/` (`ggen.toml`, `queries/statements.rq`, `templates/kinds.tera`) |
| Court model | `scripts/mutation_court.sh` |
| Moonshot row | `docs/jira/v26.10.22/SQLRDF-16.md` |
| Anti-vacuity law | `CLAUDE.md:24` |
