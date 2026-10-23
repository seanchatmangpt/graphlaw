# sql_ast.rs decomposition design

Public path stays `graphlaw::sql_ast::…`. The module root stayed
`src/sql_ast.rs`. Children live in `src/sql_ast/`.

## On disk (2026-10-22)

Measured with `wc -l` after the switchover. `cargo check -p graphlaw --tests --offline` exit 0.

| File | Lines | What it holds |
|---|---:|---|
| `src/sql_ast.rs` | 2338 | Module root: `RdfAstGraphReader`, `compile_turtle`, dataset reader, tests |
| `src/sql_ast/ast.rs` | 520 | Expression and clause types, `CompiledQuery`, `MergeStatement` |
| `src/sql_ast/refusals.rs` | 603 | `RefusalCode`, `AstError`, `refusal_arm`, `RefusalContext`, `PackedIdent` |
| `src/sql_ast/serializer.rs` | 232 | `SelectStatement` and `serialize_expr` |

`CompiledQuery::admission_seal` is `pub(super)` so the parent court can read it. A struct literal outside `sql_ast` is still impossible. No field order, `repr`, or receipt formula change.

## Not executed

The prep-lane layout below (`mod.rs`, `error.rs`, `scope.rs`, `compiled.rs`, `reader.rs`, `dataset.rs`, `compile.rs`) was not cut. The same-day operator brief asked for `src/sql/{ast,refusals,serializer,builder}.rs` and a cleanup of an unused import in `src/duckdb_eval.rs`. `builder.rs` was not created. The parent was not renamed to a directory. `src/sql/` does not exist.

## Prep-lane layout (unexecuted)
src/sql_ast/{mod.rs (facade), error.rs (~60), ast.rs (~550), scope.rs (~150),
compiled.rs (~120), reader.rs (~760), dataset.rs (~700, private),
compile.rs (~350)}.

## Moves cleanly
error (leaf) → ast/scope (data) → compiled (blake3-only) → compile
(orchestration) → reader (trait touches only public types) → dataset (the
private `outgoing` map + RdfTerm + 10 helpers + trait impl form a closed
unit; move last, atomically).

## The three genuine seams
1. `compile_expression_form`-style concrete-type helpers if trait defaults
   call them — needs `pub(crate)`, surface it early by ordering reader.rs
   before dataset.rs.
2. `CANONICALIZATION_COUNT` — `#[cfg(test)]` static shared between
   compile_turtle and the canon test: keep `pub(crate)` under cfg(test) or
   move the test with compile.rs.
3. `SelectStatement::serialize_expr` shared by compile and compile_merge —
   both go to compile.rs together.

## Risk ranking
error/ast/scope/compiled = mechanical · compile = low-med (the cfg(test)
static) · reader = medium (760-line visibility audit) · dataset = highest
(largest contiguous block, atomic move, green build between every step).

## Sequencing
1 error → 2 scope+ast → 3 compiled → 4 reader (seam audit) → 5 dataset
(atomic) → 6 compile + counter → 7 facade pub use finalized + grep-verify
every consumer import resolves. Green build between every step.
