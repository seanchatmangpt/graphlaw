# GWS-16 — WASM ABI op-15 `duckdb` (native-side cursor-streamed analytical eval)

Standing: PARTIAL_ALIVE (implementation landed on current HEAD; two foreign-lane
commit races wiped the first build — rebuilt and re-greened on HEAD 213b554)
Context: solo · Date: 2026-10-21 · Lane: GWS-16 · Repo: /Users/sac/graphlaw (current branch, main)
Design contract: `/Users/sac/graphlaw/docs/wasm-duckdb-op15-design.md`

## Scope per coordinator override

Op-15 is a NATIVE-side ABI op: frozen-duckdb's C++ core does not compile to
wasm32 (GWS-1 finding), so the op dispatches in `src/abi.rs` behind the
`duckdb` cargo feature, the wasm op table does NOT gain op 15, and the manifest
carries a `notes` field documenting the native-only op instead.

## Diff (files, absolute paths)

- `/Users/sac/graphlaw/src/duckdb_op.rs` — NEW: exec/stream/close cursor state
  machine over frozen-duckdb; compile-time pragmas `memory_limit=32MiB`,
  `threads=1`, `preserve_insertion_order=false` applied to every connection
  (GAP-2, no wire override path); views accept only `{name,glob,hive}`
  (CSV via `read_csv_auto`, parquet via `register_parquet_view`; SQL/pragma
  keys in views are typed-refused); opaque BLAKE3 cursor ids; unknown/expired
  cursor → typed `DuckDb` refusal; `close` idempotent-ok.
- `/Users/sac/graphlaw/src/abi.rs` — dispatch arm `"duckdb"` (cfg duckdb);
  `MAX_OUTSTANDING_ALLOC_BYTES` split-config: 16 MiB wasm32 / 256 MiB native
  (GAP-1); `str_field`/`term_json`/`check_limit`/`limit` widened to
  `pub(crate)`; `fail_with_details` constructor.
- `/Users/sac/graphlaw/src/lib.rs` — `pub mod duckdb_op` (cfg duckdb).
- `/Users/sac/graphlaw/src/dialect.rs` — `RefusalKind::DuckDb` variant.
- `/Users/sac/graphlaw/src/registry.rs` — `DuckDb` REFUSAL_CODES row
  (kind DuckDb, fields [message]).
- `/Users/sac/graphlaw/wasm/src/abi_meta.rs` — `ERROR_CODES += "DuckDb"`
  (registry order, between NotAdmitted and UnverifiedLeaseRefused);
  `MAX_OUTSTANDING_BYTES` split-config 16 MiB wasm32 / 256 MiB native.
- `/Users/sac/graphlaw/src/abi_manifest.rs` — `notes` key documenting the
  native-only op-15, GAP-1/2/4 bounds.
- `/Users/sac/graphlaw/contracts/abi-manifest.json` — regenerated;
  `manifest_blake3` = `blake3:60284879adb38fbbf50396e83216b384882ae914ad8692516003314bed53a826`
  (was `blake3:a30323d0…` pre-session; the a303→6028 delta also carries the
  CalVer bump 26.10.13 → 26.10.21 absorbed by commit 213b554's regeneration).
- `/Users/sac/graphlaw/registry/capability-registry.json/.ttl` — regenerated
  (DuckDb code row); `registry_blake3` = `blake3:b6fb56566c9779bc4ccaab7c18bd746eb306a82c8b1275ed01123cdf5e51cd7d`
  (was `aa636fd1…`); surface_blake3 unchanged (`c143fcfe…`) — additive,
  no op-list change.
- `/Users/sac/graphlaw/registry/ARTIFACTS.blake3` — registry digest re-pinned
  (surface row unchanged by content; digest identical).
- `/Users/sac/graphlaw/docs/refusals.md` — GAP-4 crosswalk rows: `DuckDb` code
  row + RefusalKind list entry; chunk limits ride `ResourceLimit`
  (`duckdb_chunk_rows`/`duckdb_chunk_bytes`/`duckdb_views`).
- `/Users/sac/graphlaw/Cargo.toml` — `[[test]] wasm_duckdb_op` with
  `required-features = ["duckdb"]`.
- `/Users/sac/graphlaw/tests/wasm_duckdb_op.rs` — NEW court, 5 tests.

## Courts (all on HEAD 213b554 + this diff)

`tests/wasm_duckdb_op.rs` 5/5 (stable across 3 consecutive runs, 1.05-1.08s):
(a) exec/stream/close happy path over an injected CSV (GWS-4 pattern) —
rows_emitted=2, page N-Triples carries the packed focus-node subject/predicate
IRIs, drain-nulls cursor, drain of exhausted cursor is empty-ok, close
idempotent (incl. never-existed cursor);
(b) `duckdb_chunk_rows` typed ResourceLimit refusal (10_001 → max 10_000), and
`duckdb_chunk_bytes` refusal with cursor survival + smaller-page retry;
(c) 20-case raw_call fuzz via `catch_unwind` — zero traps, every response a
typed envelope;
(d) native `builtin_eval_sql` (real front door, FixedQueryReader) ↔ ABI op-15
exec: query_hash, receipt IRI (`{subject}/receipt_{hash16}`), rows_emitted,
and sql_statement byte-identical.
Unit tests in `src/duckdb_op.rs` 2/2; manifest court 10/10;
registry_artifacts 19/19 (incl. ARTIFACTS.blake3 pin); graphlaw-wasm 6/6
(abi_meta mirror incl. DuckDb; split-config pin equality);
wasm32-wasip1 build EXIT=0 with the 16 MiB split-config bound.

## Full gates

`cargo test --features duckdb` (full): green — 166 "test result: ok" lines,
0 FAILED, 0 errors (run pre-wipe; re-verified targeted courts post-rebuild as
above; two earlier "failures" classified foreign-lane contention:
chicago_fibo 4 fails under concurrent builds → 61/61 on rerun; manifest
court EOF-parse race → fixed by another lane's atomic write_manifest).
Default `cargo test` (full): green, same profile; docs_release_claims flake
mid-run was a concurrent quickstart.md edit by another lane → 2/2 on rerun.

## Standing

PARTIAL_ALIVE: the op is real, pragma-enforced, typed-refusal-only, and
byte-identical to the native builtin; the wasm-module leg of the design's
"in-guest" claim is impossible until frozen-duckdb compiles to wasm32 (GWS-1
BLOCKED), so the receipt/query_hash identity is proven native-ABI ↔ native
builtin. Cursor state is process-local and crash-only (documented contract).
wasm-size court re-baseline deferred to the coordinator's artifact re-pin
commit (wasm32 binary rebuilt locally: 6,656,026 B at target/wasm-abi).

## Falsifiers

- `cargo test --features duckdb --test wasm_duckdb_op` exits non-zero, or
- abi_manifest_court finds committed bytes != regenerated (would catch any
  un-regenerated manifest drift), or
- `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile wasm` fails.
