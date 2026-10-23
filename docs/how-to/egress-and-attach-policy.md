# How to export tables and attach databases under policy

Goal-oriented guide for two controlled egress surfaces: `COPY ... TO`
(egress witness) and `ATTACH` (side database admission). Reference text:
[DIALECTS](../DIALECTS.md) · [refusals](../refusals.md) ·
[run-the-wasm-module](run-the-wasm-module.md).

## COPY TO: expression form

Compile a `sql:CopyStatement` from the graph, not from a path literal
floating in prose. The compiler arm
(`src/sql_ast.rs:1791`, expression form) reads:

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
typed `UnsupportedTerm` compile refusal (`src/sql_ast.rs:1798-1800`).

## COPY TO: the digest rides in the receipt

A successful `COPY ... TO` streams the exported file through BLAKE3 and
carries the hex digest plus byte count on the `ExecutionReceipt` — never on
stdout (`src/duckdb_eval.rs:307-316`, computed at `:1466-1488`):

| field | meaning |
|---|---|
| `artifact_digest_hex` | hex BLAKE3 of the exported file, 8 KiB streaming |
| `artifact_bytes` | byte count of the same file |
| `artifact_warning` | typed reason when the file could not be read/hashed |

Two laws (SQLRDF-22 G3, `src/duckdb_eval.rs:276-284`):

- `query_hash` does NOT fold the artifact fields — one replay identity per
  admitted statement, independent of where (or whether) bytes were
  exported.
- A COPY whose file cannot be read back keeps the receipt and records
  `artifact_warning`; digest/bytes stay `None` and are skipped on the
  wire. Never a lost receipt, never a panic.

```sh
cargo test --lib g3_copy
```

Witnessed 2026-10-23, real file + real frozen-duckdb, independent
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
`/tmp/graphlaw-rdf-<table>.csv` (`src/sql_ast.rs:2255-2258`). That is a
lawful but unaddressed egress target. Prefer the expression form so the
path, format, and digest are all under your control and on the receipt.
The same asymmetry exists for ATTACH: the scalar `AttachStatement` arm
hardcodes `ATTACH ':memory:' AS <alias>` (`src/sql_ast.rs:2317`) — it
cannot reach a file-backed database at all.

## ATTACH: allowlist policy

Compile the expression form (`src/sql_ast.rs:1940`):

```turtle
<urn:attach:a1> a sql:AttachStatement ;
    sql:statementIndex 0 ;
    sql:attachTarget [
        sql:databasePath "/srv/data/warehouse.duckdb" ;
        sql:attachAlias "warehouse" ] .
```

Before the engine sees the statement, `enforce_attach_policy` runs
(`src/duckdb_op.rs:228`, pure seam `:235`; call sites `:400` single
statement and `:491` per document member):

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

`enforce_attach_policy` is a HOST-side seam. It runs in `src/duckdb_op.rs`,
in the same process that links frozen-duckdb natively — it never runs
inside the wasm module. Two consequences:

1. **Preopening a directory bypasses the allowlist.** If you preopen a
   directory into the guest's WASI context (`preopen_dir` in your own
   host embedding), the guest can open any file under it directly —
   including a DuckDB database file — with no `GRAPHLAW_ATTACH_ROOTS`
   check, no canonicalize, no refusal. The allowlist constrains the
   ATTACH statement path, not WASI filesystem rights.
2. **The runner sets no metering or memory ceiling.** `graphlaw-run-eh`
   does no fuel/epoch metering, enforces no linear-memory limit, and
   passes the full host env via `inherit_env()`
   (`src/bin/graphlaw_run_eh.rs:236`). The import allowlist and 64 MiB
   request wall are the only containment — see SECURITY.md "EH host
   sandbox" for the full not-provided list. Do not treat the sandbox
   claims in [WASM](../WASM.md) as a substitute for the attach policy;
   they are weaker and different.

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

Full policy module count (13/0) is receipted in
`docs/jira/v26.10.22/RECEIPTS.md` (row W15).

## Driving it through graphlaw-run-eh

The wire recipe is the standard packet-4 frame
(`src/bin/graphlaw_run_eh.rs:341`): 4-byte big-endian length prefix, then
the JSON request — `{"op":"duckdb","mode":"exec","document":true,
"query_ast":"<iri>","ast":{"turtle":"<Turtle AST document>"},
"subject_prefix":"...","vocab_prefix":"..."}` — reply is
`{"ok":true,"result":<BatchReceipt>}` with one `ExecutionReceipt` per
member in `members` (`docs/DIALECTS.md` "Document sequences"). A COPY
member's `artifact_digest_hex`/`artifact_bytes` appear in its member
receipt; verify the digest against an independent BLAKE3 of the file.

Status: the in-runner execution step is PENDING, not witnessed. The
DuckDB-linked artifact traps on first `gl_call`
(`__cxa_thread_atexit` dtor runner, wasi-libc atexit walker) — SQLRDF-24,
receipts W30/W56 in `docs/jira/v26.10.22/RECEIPTS.md`. Until that lands,
host-side courts above are the witnessed rung; a loud PENDING is not a
pass.

## Status claims, re-read at use time

- SQLRDF-22 (COPY egress policy, ATTACH validation, egress digests): row
  in `docs/jira/v26.10.22/INDEX.md`; G1/G3/G4/G6 LANDED with courts, G5
  filed forward as SQLRDF-23.
- Verify with: `cargo test --lib g3_copy` and `cargo test --lib attach`.

## See Also

[DIALECTS](../DIALECTS.md) · [refusals](../refusals.md) ·
[run-the-wasm-module](run-the-wasm-module.md)
