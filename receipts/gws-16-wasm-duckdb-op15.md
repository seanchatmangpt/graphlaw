# GWS-16 — WASM ABI op-15 `duckdb` (native-side cursor-streamed analytical eval)

Standing: PARTIAL_ALIVE (implementation on HEAD c250490 + this lane's uncommitted diff; two foreign-lane commit races wiped the first build mid-session — rebuilt and re-greened on the current tree)
Context: solo · Date: 2026-10-21 · Lane: GWS-16 · Repo: /Users/sac/graphlaw (current branch main)
Design contract: `/Users/sac/graphlaw/docs/wasm-duckdb-op15-design.md`

## Scope per coordinator override

Op-15 is a NATIVE-side ABI op: frozen-duckdb's C++ core does not compile to
wasm32 (GWS-1 finding), so the op dispatches in `src/abi.rs` behind the
`duckdb` cargo feature, the wasm op table does NOT gain op 15, and the manifest
carries a `notes` field documenting the native-only op instead.

## Diff

- `/Users/sac/graphlaw/src/duckdb_op.rs` — NEW: exec/stream/close cursor state
  machine over frozen-duckdb; compile-time pragmas `memory_limit=32MiB`,
  `threads=1`, `preserve_insertion_order=false` applied to every connection
  (GAP-2, no wire override path); views accept only `{name,glob,hive}`
  (CSV via `read_csv_auto` — the GWS-4 injection pattern — parquet via
  `register_parquet_view`; `sql`/`pragma`/`query` keys in views are
  typed-refused); opaque BLAKE3 cursor ids; unknown/expired cursor → typed
  `DuckDb` refusal; `close` idempotent-ok.
- `/Users/sac/graphlaw/src/abi.rs` — dispatch arm `"duckdb"` (cfg duckdb);
  `MAX_OUTSTANDING_ALLOC_BYTES` = 16 MiB target-uniform (GAP-1); widened
  `str_field`/`term_json`/`check_limit`/`limit` to `pub(crate)`;
  `fail_with_details` constructor (cfg duckdb).
- `/Users/sac/graphlaw/src/lib.rs` — `pub mod duckdb_op` (cfg duckdb).
- `/Users/sac/graphlaw/src/dialect.rs` — `RefusalKind::DuckDb` variant.
- `/Users/sac/graphlaw/src/registry.rs` — `DuckDb` REFUSAL_CODES row
  (kind DuckDb, fields [message]) + `NATIVE_ONLY_OPS` cfg table registering
  the feature-gated op without perturbing the hashed registry document.
- `/Users/sac/graphlaw/wasm/src/abi_meta.rs` — `ERROR_CODES += "DuckDb"`
  (registry order: between NotAdmitted and UnverifiedLeaseRefused);
  `MAX_OUTSTANDING_BYTES` = 16 MiB target-uniform (GAP-1).
- `/Users/sac/graphlaw/src/abi_manifest.rs` — `notes` key documenting the
  native-only op-15 and the GAP-1/2/4 bounds.
- `/Users/sac/graphlaw/contracts/abi-manifest.json` — regenerated;
  `manifest_blake3` = `blake3:2321b6c70ad38e438cbdd16019c9225e64f4bc7c643d1517cdfc7ac4ab8f9f43`
  (pre-session `blake3:a30323d0…`; delta carries the CalVer 26.10.13→26.10.21
  bump absorbed by commit 213b554, the notes key, and the 16 MiB limit).
- `/Users/sac/graphlaw/registry/capability-registry.json/.ttl` — regenerated;
  `registry_blake3` = `blake3:0debaedbc40b72b1fb0c6003b59dac24a1d7a7c48b9344dd8befd2ede69ad044`
  (was `aa636fd1…`); surface_blake3 unchanged `c143fcfe…` — additive, no
  op-list change. Target-invariant: native and wasm builds produce the same
  digest (required by the op_examples native≡wasm byte court).
- `/Users/sac/graphlaw/registry/ARTIFACTS.blake3` — registry digest re-pinned
  to 0debaedb…; surface row unchanged.
- `/Users/sac/graphlaw/docs/refusals.md` — GAP-4 crosswalk rows: `DuckDb` code
  row + RefusalKind list entry; chunk limits ride `ResourceLimit`
  (`duckdb_chunk_rows`/`duckdb_chunk_bytes`/`duckdb_views`).
- `/Users/sac/graphlaw/Cargo.toml` — `[[test]] wasm_duckdb_op` with
  `required-features = ["duckdb"]`.
- `/Users/sac/graphlaw/tests/wasm_duckdb_op.rs` — NEW court, 5 tests.
- `/Users/sac/graphlaw/tests/registry_dispatch.rs` — dispatch-arms parser is
  now cfg-aware: an arm under `#[cfg(feature = "duckdb")]` counts only when
  the feature is active, and the registry side of the drift compare is
  `op_names() ∪ NATIVE_ONLY_OPS`. Court force preserved (still source-vs-
  registry drift detection; the 14-op capabilities table is untouched).
- `/Users/sac/graphlaw/tests/wasm_memory_wall.rs` — GAP-1 re-baseline
  (design-mandated "wasm-size court re-baseline"): outstanding cap constant
  268,435,456 → 16,777,216; at-cap probes moved to private instances (a 16 MiB
  alloc now consumes the whole outstanding budget, so sharing the module with
  parallel courts would starve them).

## Courts

`tests/wasm_duckdb_op.rs` 5/5 (stable across 3 consecutive runs):
(a) exec/stream/close happy path over an injected CSV (GWS-4 pattern) —
rows_emitted=2, page N-Triples carries the packed focus-node subject/predicate
IRIs, drain nulls the cursor, draining an exhausted cursor is empty-ok, close
idempotent (incl. never-existed cursor);
(b) `duckdb_chunk_rows` typed ResourceLimit refusal (10_001 → max 10_000) and
`duckdb_chunk_bytes` refusal with cursor survival + smaller-page retry;
(c) 20-case raw_call fuzz via `catch_unwind` — zero traps, every response a
typed envelope;
(d) native `builtin_eval_sql` (real front door via FixedQueryReader) ↔ ABI
op-15 exec: query_hash, receipt IRI (`{subject}/receipt_{hash16}`),
rows_emitted, and sql_statement byte-identical.
Unit tests in `src/duckdb_op.rs` 2/2; manifest court 10/10; registry_artifacts
19/19 (ARTIFACTS.blake3 pin); registry_dispatch 14/14 (both feature sets);
wasm_memory_wall 7/7 on the real rebuilt binary; op_examples
native≡wasm 3/3 (fresh artifact); graphlaw-wasm 6/6 (abi_meta mirror incl.
DuckDb; pin equality); wasm32-wasip1 build EXIT=0 (6,656,216 B at
target/wasm-abi; coordinator to re-pin priv/graphlaw.wasm.blake3 +
ARTIFACTS wasm row at the next artifact release commit per the header rule).

## Full gates (final tree)

- `cargo test --features duckdb` (full suite): zero FAILED / zero errors.
- `cargo test` (default, full suite): zero FAILED / zero errors.
- `cargo test -p graphlaw-wasm`: 6/6.
- Earlier run-level failures classified and resolved: chicago_fibo 4 fails and
  docs_release_claims 1 fail were foreign-lane concurrent-edit contention
  (both green on isolated rerun, 61/61 and 2/2); abi_manifest EOF race fixed
  by another lane's atomic write_manifest; the 256→16 MiB boundary and the
  cfg-gated dispatch arm were genuine court re-baselines landed in this lane.

## Standing

PARTIAL_ALIVE: the op is real, pragma-enforced, typed-refusal-only, and
byte-identical to the native builtin; the wasm-module leg of the design's
"in-guest" claim is impossible until frozen-duckdb compiles to wasm32 (GWS-1
BLOCKED), so the receipt/query_hash identity is proven native-ABI ↔ native
builtin. Cursor state is process-local and crash-only (documented contract).
Deferred to coordinator: wasm artifact digest re-pin in
registry/ARTIFACTS.blake3 (wasm row still pins 26.10.13-era bytes) and
priv/graphlaw.wasm.blake3.

## Falsifiers

- `cargo test --features duckdb --test wasm_duckdb_op` exits non-zero, or
- abi_manifest_court / registry_artifacts find committed bytes != regenerated
  (catches any un-regenerated manifest/registry drift), or
- `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` fails,
  or `cargo test --test wasm_memory_wall` fails (GAP-1 boundary holds).
