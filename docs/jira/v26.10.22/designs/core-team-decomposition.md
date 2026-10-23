# Core-team decomposition (operator brief, reconciled 2026-10-22)

Operator brief for splitting the largest Rust files. The mechanical module
split of eight trees is on disk. Public paths are unchanged. Wire layouts,
receipt formulas, and ABI op dispatch were not redesigned.

Line counts below are `wc -l` on the working tree after the switchover.
The brief's own counts were the pre-split parent files.

Rust shape used throughout: the parent stays `src/foo.rs` (or
`crates/graphlaw-eyeron/src/foo.rs`) and the children live in `src/foo/`.
That is a legal module pair. Parents were not replaced by `foo/mod.rs`.
`pub use` keeps the paths callers and doctests already use.

Staging folders `*_split/` are deleted, including their `SPLIT-NOTES.md`.

## Invariance still binding

| Surface | What must stay | What the split did |
|---|---|---|
| `receipt_chain` frame | `#[repr]` and the packed hash bytes | Moved `frame.rs` as a body. Epoch and the validator stayed in the parent. |
| Registry surface | Canonical JSON and Turtle key order, Blake3 of the emitted document | Moved `definitions`, `emit_json`, `emit_ttl`. `include_bytes!` paths were adjusted to the new file location. No key-order edit. |
| Reasoner / SQL | Branch order, step counters, recursion limits | Bodies moved. `SolveContext` was not substituted into `solve_backward_goal`. |
| ABI and dialect | Op names, refusal codes, JSON keys | Handlers moved. `lease.rs` is a draft comment. Sniff still returns the same refusal values. |

`tests/registry_hash_invariance.rs` does not exist. The registry courts on
disk are `tests/registry_artifacts.rs` and `tests/registry_dispatch.rs`.
They were not re-run as part of filing this note. The switchover gate that
did run:

```text
CARGO_TARGET_DIR=/tmp/gl-target cargo check -p graphlaw --tests --offline
CARGO_TARGET_DIR=/tmp/gl-target cargo check -p graphlaw-eyeron --offline
```

Both exited 0. `cargo check --workspace --tests --benches`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --workspace` were not run for this split.

## Primary tier

| Parent now | Lines | Children | Lines | Brief asked | Status |
|---|---:|---|---:|---|---|
| `crates/graphlaw-eyeron/src/reasoner.rs` | 4248 | `limits.rs` | 92 | limits module | Landed |
| | | `solve_context.rs` | 124 | `SolveContext` replaces the long parameter lists and drops `too_many_arguments` | Draft only. Module is `#[allow(dead_code)]`. `reasoner.rs:3` still allows `clippy::too_many_arguments`. |
| | | `builtins/mod.rs` | 946 | `BuiltinPredicate` trait and dispatcher | Functions moved. No trait. |
| | | `builtins/{list,log,math,string}.rs` | 903, 876, 845, 546 | those four builtin files | Landed |
| | | — | | `unify.rs`, `tabled.rs` | Not cut. Unification and tabling stayed in the parent. |
| `src/sql_ast.rs` | 2338 | `ast.rs`, `refusals.rs`, `serializer.rs` | 520, 603, 232 | `src/sql/{ast,refusals,serializer,builder}.rs` | Three children landed beside the parent. `src/sql/` and `builder.rs` were not created. See [sql-ast-decomposition.md](sql-ast-decomposition.md). |
| `src/receipt_chain.rs` | 1033 | `frame.rs`, `record.rs`, `store.rs` | 373, 848, 108 | `src/receipt/{frame,record,store}.rs` with `receipt/mod.rs` | Landed under `receipt_chain/`. Parent kept epoch and validator. |
| `crates/graphlaw-eyeron/src/parser.rs` | 342 | `cursor.rs`, `grammar.rs` | 438, 1500 | cursor, grammar, entry points in `parser/mod.rs` | Landed. Entry points stayed in `parser.rs`. `grammar.rs` is still the large unit. |
| `src/registry.rs` | 130 | `definitions.rs`, `emit_json.rs`, `emit_ttl.rs` | 884, 756, 333 | same three files, parent as façade | Landed. This is the brief's façade shape. |

Surveyed and left as one file:

| File | Lines then (brief) | Lines now | Note |
|---|---:|---:|---|
| `src/duckdb_eval.rs` | 1462 | 1681 | No split. The brief named an unused-import cleanup around the old line 932. Not done as its own edit. |
| `crates/graphlaw-eyeron/src/w3c_rdf.rs` | 1293 | 1293 | Survey only. No split requested. |

Parent-plus-children for the reasoner tree is 8580 lines. The parent alone
is still 4248, so a one-line edit in unification still rebuilds that unit.
The string, list, log, and math builtins no longer sit in that unit.
`builtins/mod.rs` at 946 lines is the next compile-unit target inside the
reasoner, and it was not split further.

`parser/grammar.rs` at 1500 lines is the same situation for the parser.

## Secondary tier

| Parent now | Lines | Children | Lines | Brief asked | Status |
|---|---:|---|---:|---|---|
| `src/abi.rs` | 341 | `ops/meta.rs` | 71 | capabilities, sniff, canonical | Landed. DuckDB dispatch stayed in the parent. |
| | | `ops/parse_convert.rs` | 33 | parse, convert | Landed |
| | | `ops/query_engines.rs` | 233 | sparql, shacl, shex, n3, entail, datalog | Landed |
| | | `ops/governance.rs` | 511 | hooks, law, policy | Landed |
| | | `lease.rs` | 74 | `LeaseContext` admitted once, before any op | Draft comment only. `LeaseContext::admit` is `unimplemented!` inside a `//!` block. No op calls it. |
| `src/dialect.rs` | 583 | `refusal.rs`, `sniff.rs` | 265, 319 | refusal taxonomy and sniff | Landed. Sniff was not rewritten to take `&[u8]`. Refusal codes and JSON keys were not renamed. |
| `src/query_ledger.rs` | 29 | `batch.rs`, `storage.rs` | 459, 564 | pure batch fold vs POSIX writer | Landed as files. `LedgerBatch::fold_entry` was not added. See [cold-path-ledger.md](cold-path-ledger.md). |
| `src/law.rs` | 1021 | — | — | state wrapper vs step execution (brief counted 1015) | Not split |
| `src/evidence_ring.rs` | 914 | — | — | ring state vs drainer (brief counted 775) | Not split |
| `tests/chicago_cross_authority.rs` | 1342 | — | — | shared `tests/common/chicago_fixtures.rs` | Not extracted. `tests/common/mod.rs` exists and does not contain that fixture module. |
| `tests/chicago_fibo.rs` | 1232 | — | — | same fixture module | Not extracted |

`src/abi/ops/mod.rs` is 6 lines. There is no `handlers.rs`. The refined
partition (meta / parse_convert / query_engines / governance) is the one
that landed, which is the brief's second-pass cut rather than one
handlers file.

## Phases the brief named, against the tree

1. Reasoner limits and builtins — file split landed. `SolveContext` did not replace call sites. `unify.rs` and `tabled.rs` were not cut.
2. SQL AST / refusals / serializer — landed beside `src/sql_ast.rs`. `builder.rs` and the `src/sql/` rename were not done. The prep-lane reader/dataset/compile cut in [sql-ast-decomposition.md](sql-ast-decomposition.md) was not done.
3. Parser cursor and grammar — landed. `grammar.rs` remains 1500 lines.
4. Receipt frame/record/store and registry definitions/emit_json/emit_ttl — landed under the existing parent names.

Secondary phases A–D from the same brief:

- A, Chicago fixture extraction — not started.
- B, query-ledger batch vs storage — files landed; the pure `fold_entry` API was not added.
- C, dialect sniff and refusal — files landed; the `&[u8]` sniff rewrite was not done.
- D, ABI ops and lease — ops landed; lease admission was not turned on.

## Remainder, in an order that keeps one compile unit local

1. Cut `unify.rs` and `tabled.rs` out of `reasoner.rs` the same way builtins were cut: move items, `pub(crate)` what the parent still calls, no signature change.
2. Split `builtins/mod.rs` only if a later compile profile shows it still dominates incremental builds. A trait object dispatcher is a behavior change and stays out of a move-only cut.
3. Leave `SolveContext` unused until a dedicated change rewrites `solve_backward_goal` and the courts that pin step order.
4. Leave `lease.rs` as a module comment until an op-dispatch change admits a lease before the match. The `unimplemented!` in that comment is not compiled.
5. Extract `tests/common/chicago_fixtures.rs` only with `mod common;` from both Chicago courts, then `cargo test --test chicago_cross_authority --test chicago_fibo --test chicago_validation`.
6. Re-run `bash scripts/partition_court.sh` after any of the above. The last receipt (N21) still names deleted `*_split` paths. See [code-partition-census.md](code-partition-census.md).
