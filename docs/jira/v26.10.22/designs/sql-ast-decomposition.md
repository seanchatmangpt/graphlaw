# sql_ast.rs decomposition design (post-integration refactor prep)

Condensed from the decomposition design lane. Current: 2,410 lines
monolithic. All existing `graphlaw::sql_ast::{...}` imports must keep
resolving — the refactor is a directory module with a facade `pub use` layer,
path-transparent, zero consumer changes.

## Layout
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
