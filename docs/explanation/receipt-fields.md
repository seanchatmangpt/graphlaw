# Explanation: receipt fields and what they mean

GraphLaw receipts carry the fields a verifier needs to re-derive an admission without
trusting the operator. The mapping below is **informal only** — GraphLaw does not claim
conformance to any external receipt schema.

| portfolio R field | GraphLaw field |
|---|---|
| identity | `parent`, `child` (state ids), `subject_blake3` (external subject digest), `query_hash` (DuckDB AST execution) |
| authority | `authority`, `revision` (upstream engine: PurRDF, purrdf::n3, frozen-duckdb), `lease_id`, attestation `key_id` |
| consequence | `step`, `added`, `rows_emitted` |
| replay | `plan_blake3`, query hash, and `graphlaw-verify` replay from the start state |
| standing | not stored — derived by the verifier (`ADMITTED`/`REFUSED`) from the receipts |

## Receipt chains (graphlaw-compatible)

`graphlaw::receipt_chain` is a graphlaw-compatible receipt-chaining surface:
`ReceiptRecord` (`RECEIPT_RECORD_VERSION = 1`), frozen wire strings
(`graphlaw-chain/base`, `graphlaw-chain/v2-fold`), byte-exact BLAKE3 frame hashing —
golden-tested against the committed TCPS fixture (`tests/receipt_chain_golden_test.rs`,
5 tests) — a 4-stage validator, a JSONL `ReceiptStore`, and an epoch module. ggen
migrates onto this to retire its vendored graphlaw-core.

## The two-witness receipt model (DuckDB executions)

A DuckDB execution is witnessed by **two independent, deliberately non-interchangeable
digests**.

### Witness 1 — `H_query` on `ExecutionReceipt.query_hash`

`H_query = BLAKE3(Canon(G_AST) || P_params)` — the sql-rdf Recommendation §1.4 first
formula. Computed by `admitted_statement_hash`
(`src/duckdb_eval.rs#admitted_statement_hash`) from:

- `Canon(G_AST)` — the RDFC-1.0 canonical N-Quads form of the admitted AST graph,
  produced by `canonical_ast_graph_bytes` (`src/duckdb_eval.rs#canonical_ast_graph_bytes`) via
  `purrdf::parse_dataset` + `purrdf::try_canonicalize`; parse or canonicalization
  failure is the typed `EvaluatorError::ReceiptError`, never a panic.
- `P_params` — the typed parameter buffer, hashed in bind order by
  `hash_param_buffer` (`src/duckdb_eval.rs#hash_param_buffer`): one type tag byte per parameter
  (`0x01` String, `0x02` Integer LE, `0x03` Float bit-pattern LE, `0x04` Boolean,
  `0x00` Null) followed by little-endian value bytes.

`eval_ast_query_bounded` (`src/duckdb_eval.rs#eval_ast_query_bounded`) computes it
from the compile-time anchor `compiled.receipt_state` (`h_ast = BLAKE3(Canon(G_AST))`,
folded with the parameter buffer by `fold_params_anchored`, bit-identical to
`admitted_statement_hash`) or from `compiled.canon_graph_bytes`. A query carrying
neither is a typed `EvaluatorError::NoCanonicalGraph` refusal — there is no
raw-SQL-text hash fallback. The field and its formula are documented on
`ExecutionReceipt.query_hash` (`src/duckdb_eval.rs#ExecutionReceipt`).

Court: `tests/chicago_validation.rs::merge_receipt_is_blake3_of_canonical_graph_and_params_not_sql_text`
(replay equality, literal mutation flips, and an explicit assertion that `query_hash`
is **not** a hash of the SQL text), plus the replay-determinism court
`admitted_statement_receipt_is_canon_params_and_replay_deterministic`
(`src/duckdb_eval.rs#admitted_statement_receipt_is_canon_params_and_replay_deterministic`).

### Witness 2 — the state-readback digest (DML courts)

DML courts (`tests/chicago_validation.rs#state_receipt`) hash a **different
input**: the executed SQL text, a `0xff` separator, and the ordered table readback
(`SELECT id, COALESCE(name, '') FROM t ORDER BY id, name` over the real frozen-duckdb
connection). `dml_receipts_bind_readback_and_distinguish_statements`
(`tests/chicago_validation.rs#dml_receipts_bind_readback_and_distinguish_statements`) uses it to prove insert/update/delete/merge
distinguish each other and replay identically. SQLRDF-09 keeps the two digests
distinct (`tests/chicago_validation.rs#merge_receipt_is_blake3_of_canonical_graph_and_params_not_sql_text`).

### When each is the right witness

- **`H_query`**: you want to bind the receipt to *what was admitted* — the graph the
  law admitted plus the typed parameters. It is content-addressed, syntax-independent,
  and verifiable without the database. Use it for identity, replay, and cross-runtime
  checks.
- **State readback**: you want to bind the receipt to *what the database became* —
  the actual row state after a mutating statement. `H_query` says nothing about
  whether the engine applied the statement; the ordered readback does. Use it for
  DML effect courts (Chicago-style real-state assertion).
- They compose: `H_query` proves the statement was admitted; the readback proves the
  statement took effect. Neither substitutes for the other, and the courts pin them
  as unequal digests.

### Determinism: cosmetic RDF variation does not move `H_query`

Because `Canon(G_AST)` is RDFC-1.0 canonical form, syntax-only variations of the same
dataset produce identical canonical bytes and therefore identical `H_query`:

- statement order and prefix renames (`tests/canonical_abi.rs#canonical_identity_invariant_under_reordering_and_prefix_renaming`)
- blank-node label choices, via isomorphism canonicalization
  (`tests/canonical_abi.rs#canonical_handles_blank_node_isomorphism`)
- whitespace/quoting style (Turtle parse then canonicalize discards it)

Identity *does* move when a statement's content changes
(`tests/canonical_abi.rs#canonical_identity_moves_when_a_statement_changes`) and when the typed parameter buffer changes
(`src/duckdb_eval.rs#admitted_statement_receipt_is_canon_params_and_replay_deterministic`).

Probe receipt (2026-10-22, tree @ ed3be81; historical — re-run before citing for a later tree):

```
$ CARGO_TARGET_DIR=target-docs cargo test --test canonical_abi
running 5 tests
test canonical_identity_invariant_under_reordering_and_prefix_renaming ... ok
test canonical_refuses_invalid_rdf_with_typed_error ... ok
test canonical_identity_moves_when_a_statement_changes ... ok
test canonical_returns_rdfc_id_and_canonical_nquads ... ok
test canonical_handles_blank_node_isomorphism ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

$ CARGO_TARGET_DIR=target-docs cargo test --test chicago_validation -- \
    merge_receipt_is_blake3_of_canonical_graph_and_params_not_sql_text \
    dml_receipts_bind_readback_and_distinguish_statements
running 2 tests
test merge_receipt_is_blake3_of_canonical_graph_and_params_not_sql_text ... ok
test dml_receipts_bind_readback_and_distinguish_statements ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.17s
```

## Closed deviation: hand-built-AST fallback (SQLRDF-17)

An earlier revision hashed hand-built `CompiledQuery`s (no compile-time canon anchor)
by re-canonicalizing `ast_graph_turtle` at execution time, with a legacy raw-SQL-text
hash arm behind it. Both arms are deleted: the `(None, None)` arm of the receipt match
in `eval_ast_query_bounded` returns `EvaluatorError::NoCanonicalGraph`, so every
admitted execution carries the canonical-graph identity. `canonical_ast_graph_bytes`
remains the public canonicalizer used at compile time and by `eval_admitted_ast`, which
canonicalizes caller-supplied Turtle.

## History

The two dated session receipts are historical records, preserved under `docs/archive/`:

- receipt-2026-10-09-receipt-chain-and-refuse-hooks
- receipt-2026-10-10-publish-dod