# MERGE Shape Coverage Receipt (Lane L123)

Date: 2026-10-22. Lane: L123, /Users/sac/graphlaw (one canonical checkout).
Finding closed: L33's REC-ONLY — the rec §7 MERGE vocabulary had zero SHACL
coverage in `ontologies/sql-ast.ttl`.

## Files touched

- `ontologies/sql-ast.ttl` — hand-authored (no generator header; verified),
  edited in place. MERGE additions only.
- `ASSETS.blake3` — regenerated (`UPDATE_ASSET_MANIFEST=1`); sql-ast.ttl line
  now `8b302ea3...  19562  Turtle  ontologies/sql-ast.ttl` (was `172e10f8... 15824`).
- `/tmp/merge_shapes_gate.py` — L71-pattern pyshacl /tmp gate (not a repo file).

## Shapes added (closed-world, minCount per compiler requirements)

Targets rec §7 vocabulary, cross-checked against `src/sql_ast.rs`
`extract_merge_statement` (the compiler is ground truth; all four top-level
clauses are compiler-required via MissingProperty, hence minCount 1, maxCount 1):

- `sql:MergeIntoStatement` owl:Class, subClassOf sql:Statement.
- `sql:MergeUpdateAction` owl:Class, subClassOf sql:ASTNode.
- `sql:MergeStatementShape` (targetClass sql:MergeIntoStatement):
  - sql:targetTable 1..1, sh:node sql:TableRefShape
  - sql:usingSource 1..1, sh:node sql:SelectStatementShape (sub-select re-enters the §4 select shapes)
  - sql:mergeOn 1..1, sh:node sql:BinaryExpressionShape
  - sql:whenMatched 1..1, sh:node sql:MergeUpdateActionShape
- `sql:MergeUpdateActionShape` (targetClass sql:MergeUpdateAction):
  - sql:setAssignments 1..1, sh:node sql:OrderedListShape (rdf:List collection shape per §4 pattern)
- `sql:targetTable` NOT re-declared: it already exists in the ggen dialect pack
  (ggen/sql-dialect/statement_classes.ttl, owl:DatatypeProperty) — alias-legality
  law: do not re-declare what already exists. usingSource/mergeOn/whenMatched/
  setAssignments/targetColumn/valueExpression were undeclared anywhere; left as
  path-only references (property IRIs used by shapes, no ontology-level
  re-declaration needed for shape targeting).

## Courts (real, executed)

Test files (tests/*.rs) are contended by other lanes in this wave — every
acceptance-set test file is dirty in `git status`, so no repo test file was
edited. Courts proven via the /tmp pyshacl gate (`python3 /tmp/merge_shapes_gate.py`,
exit 0):

1. rec §7 example document — CONFORMS (PASS).
2. Same document minus sql:mergeOn — REFUSES; violation carries the
   MergeStatementShape sql:mergeOn minCount=1 component. (PASS)
3. Same document with two sql:mergeOn values — REFUSES; maxCount=1. (PASS)

MERGE-SHAPES-GATE: PASS, exit 0. A loud skip is not a pass — this gate exits 1
on any flipped expectation.

## Compiler-side court proposal (coordinator to land with test ownership)

Proposed `#[test]` for `tests/sql_rdf_conformance.rs` (§4 shapes rung pattern):

```rust
#[test]
fn section7_merge_shapes_admit_and_refuse() {
    // 1. The rec §7 MERGE example (docs/reference/sql-rdf-rec.md) validated
    //    against ontologies/sql-ast.ttl with the crate shacl engine must
    //    conform (sql:MergeStatementShape, sql:MergeUpdateActionShape).
    // 2. The same graph minus sql:mergeOn must produce a Violation on the
    //    sql:mergeOn minCount component (typed refusal analog of
    //    MissingProperty("http://graphlaw.org/sql#mergeOn")).
    // 3. FLIP: the missing-mergeOn graph restored must conform again.
}
```

## Gates (actual output)

`CARGO_TARGET_DIR=target-lane123 cargo test --test sql_rdf_dialect --test corpus_conformance --test w3c_conformance`

- sql_rdf_dialect: 12 passed; 0 failed (1 ignored)
- corpus_conformance: 25 passed; 0 failed
- w3c_conformance: 8 passed; 0 failed
- corpus_conformance asset_manifest_pins_every_byte initially FAILED (stale
  manifest, expected consequence of the shapes edit) — regenerated via
  `UPDATE_ASSET_MANIFEST=1 cargo test --test corpus_conformance
  asset_manifest_pins_every_byte`, then full triple run all green.
- Alias gate (mandated): sql_rdf_dialect green — alias documents still admitted.

## Standing

ALIVE on the shapes surface (real pyshacl validation, real cargo courts).
Not committed — coordinator owns git.
