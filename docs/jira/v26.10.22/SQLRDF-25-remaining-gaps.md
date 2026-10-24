# SQLRDF-25 — Remaining deferred gaps ledger

- **Id**: SQLRDF-25-RG
- **Title**: SQLRDF-25 deferred-gap ledger — star modifiers, PIVOT/UNPIVOT, composite literals, JSON operators
- **Status**: Open
- **Conformance target**: [`docs/reference/sql-rdf-rec.md`](../../reference/sql-rdf-rec.md) (cite by section per row); one-unit wasm law (engine untouched, admission law unchanged).

## Relationship to the SQLRDF-25 ticket

[`SQLRDF-25.md`](SQLRDF-25.md) is the parent order. Its "Already-implemented
boundary" (EXPLAIN/PRAGMA/SET/INSTALL/LOAD, MERGE INTO, statement sequences)
and its landed gaps (window functions, QUALIFY, CTEs — REFUSAL-01 arms for
window-outside-scope, frame, QUALIFY shape, CTE placement already appended in
[`src/sql_ast/refusals.rs`](../../../src/sql_ast/refusals.rs)) are **not
re-asked here**. This ledger carries the three parent rows still open and
splits them into five actionable rows. Nothing in this file duplicates a row
the parent marks as landed or refuted.

## Rows

### Row 1 — Star modifiers: `EXCLUDE` / `REPLACE` projection vocabulary

- **Surface**: `SELECT * EXCLUDE (col, …)`, `SELECT * REPLACE (expr AS col, …)`.
- **Current behavior (refusing)**: there is no star term at all. The
  projection list unrolls an rdf:List of expression nodes and refuses an empty
  list as `REFUSAL-01` (`refusal_arm::EMPTY_PROJECTION`,
  [src/sql_ast.rs:922-929](../../../src/sql_ast.rs)) — "projection is never
  zero-column … never SELECT *". A graph that wants star-plus-modifier
  therefore cannot be expressed, and a hand-authored star node has no
  extraction path (an expression node with no recognized term dies as an
  untyped terminal, per the comment at [src/sql_ast.rs:620](../../../src/sql_ast.rs)).
- **Proposed behavior**: append a REFUSAL-01 arm for a star/exclude/replace
  node appearing outside a projection list (scope-integrity, mirrors the
  window `pos.allows_window()` gate at [src/sql_ast.rs:533](../../../src/sql_ast.rs));
  add projection vocabulary `sql:star`, `sql:exclude`, `sql:replace` on the
  select item node with extract + render arms in `extract_select_statement`
  ([src/sql_ast.rs:807](../../../src/sql_ast.rs)) and the projection
  serializer; empty-projection refusal stays.
- **Court**: `tests/star_modifier_court.rs` (new) — real frozen-duckdb
  readback of the EXCLUDE/REPLACE column set + anti-vacuity twin that removes
  the EXCLUDE node and requires the court to flip.

### Row 2 — `COLUMNS('regex')` projection

- **Surface**: `SELECT COLUMNS('pattern')` (regex or lambda form).
- **Current behavior (refusing)**: same projection path as Row 1: the select
  list unroll only recognizes expression nodes; there is no `COLUMNS` term and
  no function-IRI for it, so a COLUMNS node has no recognized terminal and
  cannot be authored at all (or refuses `REFUSAL-01`/`UnsupportedTerm` at
  [src/sql_ast.rs:2287](../../../src/sql_ast.rs) if routed through the
  command/format arms).
- **COLUMNS is listed separately from EXCLUDE/REPLACE** because the grammar
  slots differ: EXCLUDE/REPLACE modify a star item inside the projection list;
  COLUMNS *is* the projection item. Landing EXCLUDE/REPLACE without COLUMNS
  leaves the parent ticket's gap 3 half-closed.
- **Proposed behavior**: a `sql:columns` projection item carrying a pattern
  string (or a lambda node, refused typed `REFUSAL-01` with a new append-only
  arm until the lambda form is specified); extract + render arm emitting
  `COLUMNS('…')` verbatim (pattern is identifier-quoted, never concatenated
  SQL — same escaping discipline as [src/sql_ast.rs:2208](../../../src/sql_ast.rs)).
- **Court**: `tests/columns_projection_court.rs` (new) — column-set readback
  on a real table; mutation twin flips the regex.

### Row 3 — PIVOT / UNPIVOT statement classes

- **Surface**: `PIVOT … ON … USING … IN …` and `UNPIVOT … INTO …`.
- **Current behavior (refusing)**: statement-class dispatch
  (`compile_statement_arm`, [src/sql_ast.rs:1531](../../../src/sql_ast.rs))
  has no PIVOT/UNPIVOT arm; a graph typing the node as anything unrecognized
  falls through to the typed `AstError::UnsupportedTerm` carrying the
  statement-class IRI (the REFUSAL-02 carrier per the comment at
  [src/sql_ast.rs:1525-1537](../../../src/sql_ast.rs)) — typed, but the class
  simply does not exist.
- **Proposed behavior**: two new statement classes with SHACL shapes in
  `ontologies/sql-ast.ttl` / `ontologies/shapes/` (`sql:PivotStatement`,
  `sql:UnpivotStatement`) + extract/render arms; a PIVOT node carrying a
  malformed ON/USING/IN member refuses `REFUSAL-01` with a new append-only arm.
  PIVOT column-set readback is the conformance observable.
- **Court**: `tests/pivot_court.rs` (new) — PIVOT column-set readback named as
  the conformance observable in the parent ticket; anti-vacuity twin removes
  the IN list.

### Row 4 — Composite literals (struct / map / list) and accessors

- **Surface**: `{'k': v}`, `['a','b']`, `struct_pack(...)`, and the
  slicing/transform accessor surface: element access on any composite
  (`[i]`, `.[i:j]`, `.*`).
- **Current behavior (refusing)**: no expression term; a composite-literal
  node has no recognized terminal and dies untyped (per the
  [src/sql_ast.rs:620](../../../src/sql_ast.rs) comment), or refuses typed
  `UnsupportedTerm` if routed through a function-IRI path. No render arm, no
  SHACL shape, no operator slot for `[i]` / slicing.
- **Proposed behavior**: expression terms `sql:structLiteral` (key/value rdf:List),
  `sql:listLiteral` (rdf:List of expr nodes), and an accessor operator slot
  (`sql:indexAccess` / `sql:sliceAccess`); a nested-composite depth cap
  refuses `REFUSAL-01` (append-only arm, mirroring the depth-capped list
  unroll discipline at [src/sql_ast.rs:919](../../../src/sql_ast.rs) `unroll_rdf_list(&select_head, 64)`).
- **Court**: `tests/composite_literal_court.rs` (new) — real readback of
  `list_extract` / struct field access; mutation twin corrupts a member index.

### Row 5 — JSON operators `->` / `->>`

- **Surface**: `col->'$.k'` / `col->>'$.k'` operator tokens. `->` doubles as
  list/index access (shared with Row 4's accessor slot); `->>` extracts as text.
- **Current behavior (refusing)**: `Operator::from_token` / `Operator::from_iri`
  ([src/sql_ast.rs:694-703](../../../src/sql_ast.rs)) have no `->`/`->>`
  arms; a graph carrying them refuses typed (unknown operator token/IRI,
  REFUSAL-01-class) before any engine call. The engine already executes them.
- **Operator-token overlap with Row 4**: `->` doubles as list/index access in
  DuckDB; when Row 4's accessor slot lands first, Row 5 is the operator-token
  path through the same render site — land Row 4 before Row 5 or specify the
  overlap explicitly.
- **Proposed behavior**: two operator tokens in the operator vocabulary
  (`sql:jsonExtract`, `sql:jsonExtractText`), extract + render arms emitting
  the exact token, scope rules: legal only where ordinary operators are.
- **Court**: `tests/json_operator_court.rs` (new) — real `->`/`->>` value
  readback; mutation twin swaps the tokens and requires the court to flip.

## Constraints (inherited from the parent ticket)

- Engine untouched — the wasm artifact already executes every surface above.
- Admission law unchanged — `src/law.rs` out of scope.
- Refusal codes append-only; never renumber.
- No new `duckdb` feature gate or `#[cfg(target_arch)]` split.

## Acceptance

- `cargo test --test sql_rdf_dialect` — new shapes/classes compile and admit, exit 0.
- `cargo test --test chicago_validation` — real-readback courts per row, exit 0.
- `bash scripts/shape_mutation_court.sh` — a shape flip for a new surface must flip the corresponding court (anti-vacuity, non-vacuous gate).
- `WASI_SDK_PATH="/opt/wasi-sdk" cargo test -p graphlaw-wasm` — one-unit law holds.
- Real embedded output in the receipt; a receipt with no embedded output is not a receipt.
