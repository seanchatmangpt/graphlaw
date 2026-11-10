# How to export tables and attach databases under policy

Goal-oriented guide for two controlled egress surfaces: `COPY ... TO`
(egress witness) and `ATTACH` (side database admission). Reference text:
[DIALECTS](../DIALECTS.md) · [refusals](../refusals.md) ·
[run-the-wasm-module](run-the-wasm-module.md).

## COPY TO: expression form

Compile a `sql:CopyStatement` from the graph, not from a path literal
floating in prose. The compiler arm
(the `"CopyStatement"` arm in `src/sql_ast.rs`, expression form) reads:

```turtle
@prefix sql: <http://graphlaw.org/sql#> .
<urn:egress:c1> a sql:CopyStatement ;
    sql:statementIndex 0 ;
    sql:copyOperation [
        sql:tableName "telemetry" ;
        sql:filePath "/tmp/egress/out.csv" ;
        sql:fileFormat "CSV" ] .
```

`sql:fileFormat` is closed: `CSV` | `PARQUET` | `JSON`. Anything else is a
typed `UnsupportedTerm` compile refusal (`AstError::UnsupportedTerm` in the same arm).

## COPY TO: the digest rides in the receipt

A successful `COPY ... TO` streams the exported file through BLAKE3 and
carries the hex digest plus byte count on the `ExecutionReceipt` — never on
stdout (`src/duckdb_eval.rs#ExecutionReceipt`; the fields `artifact_digest_hex`, `artifact_bytes`, `artifact_warning`):

| field | meaning |
|---|---|
| `artifact_digest_hex` | hex BLAKE3 of the exported file, 8 KiB streaming |
| `artifact_bytes` | byte count of the same file |
| `artifact_warning` | typed reason when the file could not be read/hashed |

Two laws (SQLRDF-22 G3, documented on `src/duckdb_eval.rs#ExecutionReceipt`):

- `query_hash` does NOT fold the artifact fields — one replay identity per
  admitted statement, independent of where (or whether) bytes were
  exported.
- A COPY whose file cannot be read back keeps the receipt and records
  `artifact_warning`; digest/bytes stay `None` and are skipped on the
  wire. Never a lost receipt, never a panic.

```sh
cargo test --lib g3_copy
```

Witnessed 2026-10-23 (historical; re-run the command above before citing for a later tree), real file + real frozen-duckdb, independent
`std::fs` read + BLAKE3 compare:

```text
test duckdb_eval::tests::g3_copy_to_receipt_carries_artifact_digest ... ok
test duckdb_eval::tests::g3_copy_to_bad_directory_records_typed_warning ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out;
finished in 0.43s
```

### Scalar-fallback caveat

If a `sql:CopyStatement` carries no `sql:copyOperation` graph, the legacy
scalar template renders the destination itself — the path is hardcoded to
`/tmp/graphlaw-rdf-<table>.csv` (the scalar `"CopyStatement"` render arm in `src/sql_ast.rs`). That is a
lawful but unaddressed egress target. Prefer the expression form so the
path, format, and digest are all under your control and on the receipt.
The same asymmetry exists for ATTACH: the scalar `AttachStatement` arm
hardcodes `ATTACH ':memory:' AS <alias>` (the scalar `"AttachStatement"` render arm in `src/sql_ast.rs`) — it
cannot reach a file-backed database at all.

## ATTACH: allowlist policy

Compile the expression form (the `"AttachStatement"` arm in `src/sql_ast.rs`):

```turtle
<urn:attach:a1> a sql:AttachStatement ;
    sql:statementIndex 0 ;
    sql:attachTarget [
        sql:databasePath "/srv/data/warehouse.duckdb" ;
        sql:attachAlias "warehouse" ] .
```

Before the engine sees the statement, `enforce_attach_policy` runs
(`src/duckdb_op.rs#enforce_attach_policy`, pure seam
`enforce_attach_policy_with`; called for the single-statement path and for each
document member in `exec_document_on`):

- env `GRAPHLAW_ATTACH_ROOTS` is colon-separated absolute roots; it is the
  only attach surface.
- Unset or empty env is fail-closed: every ATTACH is refused.
- The database file must exist and canonicalize (best-effort
  `std::fs::canonicalize`; a failed canonicalize is a refusal) under at
  least one root.
- `':memory:'` never canonicalizes, so it is refused too — file-backed
  attach inside a root is the admitted form.
- Violations are typed `DuckDb` refusals
  (`attach path outside GRAPHLAW_ATTACH_ROOTS allowlist: <path>`), never
  engine errors. Symlink escape is closed by the canonicalize step.

## ATTACH policy vs the wasm sandbox: read this before preopening

`enforce_attach_policy` lives in `src/duckdb_op.rs`, which is part of the
`graphlaw` crate that the single `graphlaw-wasm` module links (the module
dispatches `duckdb` itself, `docs/WASM.md`). So the policy runs inside the
guest too, with these consequences (as of 2026-11-09):

1. **The sandbox environment is empty.** `graphlaw-run-eh` builds a sterile WASI
   context with no `inherit_env` / `env` call, so `GRAPHLAW_ATTACH_ROOTS` is unset
   in the guest and every in-guest ATTACH is refused fail-closed
   (`tests/wasm_env_isolation_court.rs`; SECURITY.md "EH host sandbox").
2. **The allowlist is not a WASI filesystem right.** The runner preopens no
   directory, so a guest cannot reach host files at all. If YOUR host embedding
   preopens a directory (`preopen_dir`), the guest can open files under it
   directly — including a DuckDB database file — and the allowlist constrains only
   the ATTACH statement path, not WASI rights. Setting the env in your own
   embedding is how you would admit roots in-guest; that is an embedder decision
   the shipped runner does not make.
3. **Resource containment is real but narrow.** `graphlaw-run-eh` runs each
   request under a fuel budget (`FUEL_PER_REQUEST`, exhaustion exits 98) and an
   epoch watchdog (exit 99 on a stalled guest, `--epoch-ticks` re-arms), bounds
   frame prefixes by `MAX_REQUEST_BYTES` (exit 92), and caps the request frame at
   `WALL_BYTES` (64 MiB). It sets **no linear-memory ceiling**. See SECURITY.md
   "EH host sandbox" for the full provided / not-provided list; do not treat the
   [WASM](../WASM.md) sandbox claims as a substitute for the attach policy.

## Worked examples (host-witnessed)

The G6 courts run a real create/insert/select lifecycle against a real
file-backed DuckDB inside a pid-keyed `/tmp` root, plus the two refusal
arms:

```sh
cargo test --lib attach
```

Witnessed 2026-10-23:

```text
test duckdb_op::tests::attach_inside_root_executes_and_reads_back ... ok
test duckdb_op::tests::attach_outside_root_refused_connection_unpoisoned ... ok
test duckdb_op::tests::attach_env_unset_refused ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 89 filtered out;
finished in 0.88s
```

The policy seam is also pinned over the public wire by
`tests/attach_bypass_court.rs` (comment-camouflaged ATTACH, env unset, admitted form
executes). Re-run both before citing counts for a later tree.

## Driving it through graphlaw-run-eh

The wire recipe is the standard packet-4 frame
(`read_frame` in `src/bin/graphlaw_run_eh.rs`): 4-byte big-endian length prefix, then
the JSON request — `{"op":"duckdb","mode":"exec","document":true,
"query_ast":"<iri>","ast":{"turtle":"<Turtle AST document>"},
"subject_prefix":"...","vocab_prefix":"..."}` — reply is
`{"ok":true,"result":<BatchReceipt>}` with one `ExecutionReceipt` per
member in `members` (`docs/DIALECTS.md` "Document sequences"). A COPY
member's `artifact_digest_hex`/`artifact_bytes` appear in its member
receipt; verify the digest against an independent BLAKE3 of the file.

Status: the former first-`gl_call` trap of the DuckDB-linked artifact (SQLRDF-24,
`__cxa_thread_atexit` / wasi-libc atexit walker) is closed by the uniform new-EH
rebuild (`docs/explanation/eh-closure-story.md`). In-runner DuckDB execution,
including COPY egress, is exercised by the env-gated courts
`tests/wasm_duckdb_lifecycle_eh.rs` and `tests/wasm_fibo_egress_eh.rs` (they
loud-skip without `GRAPHLAW_WASM`; a loud SKIP is not a pass).

## Status claims, re-read at use time

- SQLRDF-22 (COPY egress policy, ATTACH validation, egress digests): G1/G3/G4/G6
  landed with courts (above), G5 filed forward as SQLRDF-23 (document
  sequences, `document-sequences.md`). Standing is not stored here — re-read it
  from the register (`docs/jira/v26.10.22/INDEX.md`) at use time.
- Verify with: `cargo test --lib g3_copy` and `cargo test --lib attach`.

## See Also

[DIALECTS](../DIALECTS.md) · [refusals](../refusals.md) ·
[run-the-wasm-module](run-the-wasm-module.md)
