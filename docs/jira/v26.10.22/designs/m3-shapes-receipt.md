# M3 Shapes No-Drift Receipt

Date: 2026-10-22. Lane M3 (owns `ontologies/`, `ggen/sql-dialect/`).

Positive no-drift verification: `CARGO_TARGET_DIR=target-lane-m3 cargo test --test sql_rdf_dialect`
— exit 0, **25 passed, 0 failed, 0 ignored**, finished 0.87s.

The shape-coupled courts all pass against the current
`/Users/sac/graphlaw/ontologies/sql-ast.ttl` and the `ggen/sql-dialect/` statement-kind
pack: `shape_violation_refuses_before_duckdb` (SHACL closed-world minCount/maxCount
admission), `section4_string_operator_tokens_compile` (§4 operator token enum),
`disallowed_operator_token_is_refusal_01`, `d1_empty_select_list_is_refusal_01_not_select_star`,
`d4_direction_iri_object_is_refusal_01_not_silent_asc`,
`d5_operator_dual_carry_string_and_iri_is_refusal_01`, `missing_statement_index_is_refusal_01_before_duckdb`,
plus the §7 MERGE, DuckDB execution/readback, and query-hash determinism courts.

No live shape or statement-kind drift is named by any court. Alias properties
(`sql:selectList`, `sql:fromTable`, `sql:whereClause`) remain admitted alongside the
normative clause names. No edits to the source graph or generator were required; no
generated projection was touched.

Standing: ALIVE as of this run, subject to the exact command and exit above.

## Post-switchover no-drift re-run (V-L15)

Date: 2026-10-22. Lane V-L15 (same ownership: `ontologies/sql-ast.ttl`, `ontologies/shapes/ast_governance.ttl`, `ggen/sql-dialect/`).

Shape-coupled courts after the switchover, all exit 0:

- `CARGO_TARGET_DIR=target-lane-vl15 cargo test --test sql_rdf_dialect` — **33 passed, 0 failed**, 0.54s.
- `CARGO_TARGET_DIR=target-lane-vl15 cargo test --test corpus_conformance` — **12 passed, 0 failed, 1 ignored**, 8.20s.
- `CARGO_TARGET_DIR=target-lane-vl15 cargo test --test w3c_conformance` — **8 passed, 0 failed**, 0.11s.

No shape/name drift surfaced by any court; no fixes to shape references were required and no
source graph or generated projection was edited. The 1 ignored in corpus_conformance is the
pre-existing ignored case, not a skip-masquerading-as-pass of a shape court. Standing: ALIVE
against the post-switchover shapes as of the exact commands and exits above.
