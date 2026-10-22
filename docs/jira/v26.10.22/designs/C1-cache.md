# C1 — H_query content-addressed eval cache (design record)

Condensed from the C1 spec lane (full design in wave transcript). Maps to
SQLRDF-16 row C1; depends on R1 (canon-once) + E1 (alias-variant
canonicalization would make the key unstable across vocabulary).

## Key insight
The cache key IS the existing receipt: `H_query = BLAKE3(Canon(G_AST) ||
P_params)` (`src/duckdb_eval.rs:119-133`). Cache hit and receipt are one
object; the C21 receipt-store convention is reusable verbatim.

## Store
- Layer 1: process-wide `QueryCache { map: Mutex<HashMap<[u8;32] -> CacheEntry>>, epoch: AtomicU64 }`.
- Layer 2 (durable, optional): mirror `src/receipt_store.rs` convention —
  canonical JSON, content-addressed filename, linear parent->child chain,
  atomic put; subject `query-cache-v1`; do NOT reuse the law `Receipt` struct
  (its step_name allowlist is closed law vocabulary) — mirror the convention.
- `CacheEntry { h_query, receipt, grounding, epoch, stored_at_unix_ms }`.

## Correctness invariants
- C1-a: only SELECT-class results cacheable. `cacheable: bool` on
  `CompiledQuery`, set true only on the SelectStatement arm of compile_turtle;
  raw-SQL constructions (`duckdb_op.rs:214-218`) default false — composes
  with C2.
- C1-b: parameters already inside the key (`hash_param_buffer` tag-byte
  encoding); flipped literal = different key for free.
- C1-c staleness boundary (the honest core): H_query does NOT cover catalog
  state. Epoch bumps on: stored DML execution, register_parquet_view,
  ATTACH/DETACH, any raw_connection() call. Entry served only when
  `entry.epoch == current`. External writers to mounted files are OUTSIDE the
  boundary — disclosed, not assumed away. A hit proves a valid execution of
  that digest under a past-but-unchanged generation; it is not freshness.

## Seam
`eval_ast_query`: consult after hash computation when `compiled.cacheable`;
hit returns stored receipt with `cached: true` (separate `consult_ms` field,
do not overwrite stored `execution_ms`); miss -> eval -> store (success only).
`eval_admitted_ast` must skip its hash overwrite when `receipt.cached`.
Prerequisite fix (R1 realization): eval currently RE-canonicalizes
(`duckdb_eval.rs:176`) — must read `CompiledQuery::canon_graph_bytes` instead.

## Falsifier courts
F1 two connections, same graph+params -> one store entry, byte-equal receipt,
second is a hit. F2 flipped literal -> two entries. F3 DML graph -> never
stored, epoch bumps. F4 raw-SQL construction (cacheable:false) never stored.
F5 ATTACH/mount after cache -> epoch bump -> miss + re-execute.
