# SQLRDF-25 — Dialect expansion: window functions, QUALIFY, star-modifiers, CTEs, PIVOT/UNPIVOT, composite literals, JSON operators

- **Id**: SQLRDF-25
- **Title**: Dialect expansion — seven verified DuckDB surface gaps with no AST terms or compile targets
- **Status**: Open
- **Conformance target**: The sql-rdf Recommendation at [`docs/reference/sql-rdf-rec.md`](../../reference/sql-rdf-rec.md) (cite by section per gap below); the one-unit wasm law (engine untouched, admission law unchanged).

## Already-implemented boundary (verified on disk — do not re-file)

These claims were REFUTED as gaps; each is already implemented and must not
be re-asked for by this or any ticket:

- **EXPLAIN / PRAGMA / SET / INSTALL / LOAD** — compile_command arms exist at
  `src/sql_ast.rs:1854-1932` with renderers at `src/sql_ast.rs:2250-2312`;
  `LOAD icu` has been witnessed executing in-wasm.
- **MERGE INTO** — fully implemented: §7 shapes, `extract_merge_statement`,
  the merge epoch court, and the §7 upsert court.
- **Statement sequences** — SQLRDF-23 landed (compile + eval + wire;
  `sql_ast` 32/0, `duckdb_eval` 18/0, `duckdb_op` 16/0).

## Confirmed gaps

Verified against the tree: **no AST terms and no compile targets exist in
`src/sql_ast.rs` for any of the following**. The frozen-duckdb engine
supports all of them natively — the wasm artifact already executes each
surface; only the admission/compile surface is missing.

1. **Window functions** — `OVER (PARTITION BY … ORDER BY … frame)` has no
   expression model (`src/sql_ast.rs` expression enum has no window arm).
2. **QUALIFY clause** — no statement shape or compile arm.
3. **Star modifiers** — `SELECT * EXCLUDE (…)` / `REPLACE (…)` and
   `COLUMNS('regex')` have no projection vocabulary.
4. **CTEs** — `WITH` (standard and `RECURSIVE`) has no term or compile
   target on `SelectStatement`.
5. **PIVOT / UNPIVOT** — no statement class or shapes.
6. **Composite literals** — struct/map/list literals and their
   slicing/transform accessors have no expression terms.
7. **JSON operators** — `->` / `->>` have no operator tokens in the
   operator vocabulary or expression compiler.

## Required behavior

Per gap, land all four hops:

1. **Vocabulary + ggen template** — expression/AST shapes in the dialect
   pack (`ggen/sql-dialect/`, template `statement_classes.ttl.tera` →
   `statement_classes.ttl`).
2. **Compile targets** in [`src/sql_ast.rs`](../../../src/sql_ast.rs)
   (extract + render arms; refusal arms append-only via
   [`docs/refusals.md`](../../refusals.md) if a surface must refuse).
3. **SHACL admission** in [`ontologies/sql-ast.ttl`](../../../ontologies/sql-ast.ttl)
   / `ontologies/shapes/`.
4. **Courts** with real frozen-duckdb readbacks and **anti-vacuity twins**:
   window-function result values, CTE recursion depth, PIVOT column-set
   readback — each court must flip under its mutation twin
   (`scripts/mutation_court.sh` / `shape_mutation_court.sh` class).

## Constraints

- **Engine untouched** — the wasm artifact already executes every surface
  above; no `wasm/`, frozen-duckdb, or engine-side edits.
- **Admission law unchanged** — `src/law.rs` is out of scope.
- Refusal codes are append-only; never renumber.
- No new `duckdb` feature gate or `#[cfg(target_arch)]` split — one unit.

## Acceptance

- `cargo test --test sql_rdf_dialect` — new shapes/classes compile and admit,
  exit 0.
- `cargo test --test chicago_validation` — real-readback courts per gap
  (window fn results, recursive CTE depth, PIVOT column sets), exit 0.
- `bash scripts/shape_mutation_court.sh` — a shape flip for a new surface
  must flip the corresponding court (anti-vacuity, non-vacuous gate).
- `WASI_SDK_PATH="/opt/wasi-sdk" cargo test -p graphlaw-wasm` — one-unit
  law holds: the new surfaces compile and evaluate in-module.
- Real embedded output in the receipt; a receipt with no embedded output is
  not a receipt.
