# eyeron switchover fix-forward receipt (lane N25)

Date: 2026-10-22 · Lane: N25 · Repo: /Users/sac/graphlaw (canonical checkout, no commit)

## Census (as observed at lane start)

`CARGO_TARGET_DIR=target-lane-n25 cargo check -p graphlaw-eyeron` — **exit 0, 0 errors**.
The dispatch reported 31 lib errors (L45's cross-file visibility class: private `impl Parser`
methods split across cursor.rs/grammar.rs). At lane start the wired split
(`crates/graphlaw-eyeron/src/parser/` — `cursor.rs`, `grammar.rs`, wired by `parser.rs` via
`mod cursor; mod grammar;` with `pub(crate) use cursor::MAX_TERM_NESTING_DEPTH` and
`use cursor::{number_literal, Parser}` import glue) already compiled clean. The freeze had
already been fixed forward by the owning switchover lane before N25's first check; no
compile-break was present to fix.

Remaining split state (intentionally untouched — semantic-content law):

- `crates/graphlaw-eyeron/src/parser_split/` and `crates/graphlaw-eyeron/src/reasoner_split/`
  are **unwired staging dirs**. `parser_split/SPLIT-NOTES.md` itself states the staging copy
  "references `parser_split/` yet; switchover is a later phase", and pre-commits the switchover
  plan (pub(crate) promotions of resolve/bind_one/unify_triple/FactIndex for reasoner;
  cursor/grammar sibling visibility for parser).
- `crates/graphlaw-eyeron/src/reasoner.rs` remains monolithic; it already carries the
  pub(crate) promotions needed by its split plan (`pub(crate) resolve_pattern`,
  `pub(crate) struct FactIndex`, `pub(crate) fn index_of_facts`). `unify_triple`/`bind_one`
  stay private while the split is unwired — nothing references them across files, and
  promoting them with zero callers would be dead visibility, not glue.
- The parser switchover items from SPLIT-NOTES are present in the wired state: sibling
  visibility between cursor.rs and grammar.rs via `pub(crate)` items and the `parser.rs`
  facade re-exports.

## Fix classes applied by N25

None required — zero compile errors, zero warnings in graphlaw-eyeron lib target. No edits
made to any file; file ownership respected (no touching src/sql_ast.rs, duckdb_eval.rs,
duckdb_op.rs).

## Gates (real output, lane target dir `target-lane-n25`)

| Gate | Exit | Result |
|---|---|---|
| `cargo check -p graphlaw-eyeron` | 0 | 0 errors |
| `cargo test -p graphlaw-eyeron` | 0 | 21 passed; 0 failed (lib tests); 2 empty harnesses |
| `cargo test --test sql_rdf_dialect` | 0 | re-measured 2026-10-23: 34 passed; 0 failed (at receipt: 33 passed / 0 failed) |
| `cargo test --test chicago_validation` | 0 | re-measured 2026-10-23: 11 passed; 0 failed (at receipt: 9 passed / 0 failed) |

## Standing

ALIVE for the compile-health surface. The reasoner split (reasoner_split/) remains
PARTIAL_ALIVE staging — its switchover is a later phase per its own SPLIT-NOTES; when it
lands it needs the pub(crate) promotions named above promoted at the split seam, not
re-derived.
