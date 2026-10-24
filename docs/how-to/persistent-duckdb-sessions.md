# How to run persistent DuckDB sessions over the op-15 wire

Grounded in `src/duckdb_op.rs` as landed; the conformance text lives in
`docs/DIALECTS.md` ("DuckDB op session mode"). Wire transport is the
packet-4 stdio protocol (4-byte big-endian length-prefixed JSON frames)
spoken by `graphlaw-run-eh`.

## Why sessions

Without a `"session"` field, every `exec` opens a **fresh in-memory
connection** and throws it away (`src/duckdb_op.rs:326-339`): a table
created in one call is gone by the next. A **named session** registers one
persistent in-memory connection in a process-local registry, so later
`exec` calls on the same name reuse it — the CREATE→INSERT→SELECT
lifecycle works across frames.

Session state is process-local and crash-only (`sessions()`,
`src/duckdb_op.rs:83-85`): it dies with the host process, like cursor
state. Keep the whole lifecycle inside one host process.

## The lifecycle in four frames

1. **Open** — `{"op":"duckdb","mode":"open","session":"<name>"}` →
   `{"ok":true,"session":"<name>"}`. The connection gets the same GAP-2
   pragmas as every graphlaw connection (memory_limit 32 MiB, threads=1);
   callers cannot override them from the wire.
2. **Exec** — `{"op":"duckdb","mode":"exec","session":"<name>", ...}`
   routes to the named session connection instead of a fresh one. Other
   fields are the standard exec shape (`query_ast`, `subject_prefix`,
   `vocab_prefix`, `ast.turtle`).
3. **Stream** — `{"op":"duckdb","mode":"stream","session":"<name>",
   "cursor":"<id>","max_rows":N}` pages a SELECT's result set; repeat while
   `"more":true`.
4. **Close** — `{"op":"duckdb","mode":"close","session":"<name>"}` drops
   the session (and optionally a `"cursor"`). Unknown handles are
   idempotent-ok so a crash-recovery path cannot wedge on close.

## Limits and typed refusals

- **16 live sessions max** (`MAX_SESSIONS`, `src/duckdb_op.rs:79`); the
  17th open is a `duckdb_sessions` ResourceLimit refusal
  (`src/duckdb_op.rs:622`). Session state is process-local and
  crash-only — it dies with the host instance, never persists to disk;
  a restarted host starts with zero sessions.
- **Duplicate open** is refused: "session `<name>` already exists (close
  it before reopening)" (`src/duckdb_op.rs:618-621`).
- **Unknown or expired session** on `exec` with `"session"`:
  "unknown or expired session `<name>`" (`src/duckdb_op.rs:334`).
- Closing a session frees its connection; reopening the same name
  succeeds after close.

Witnessed over the real wire (graphlaw-run-eh + packet-4 frames; exit 0):

```text
open1: {'ok': True, 'session': 'dup'}
open2: {'error': {'details': {'code': 'DuckDb',
  'message': 'session `dup` already exists (close it before reopening)'},
  'kind': 'DuckDb', ...}, 'ok': False}
exec-unknown: {'error': {'details': {'code': 'DuckDb',
  'message': 'unknown or expired session `nope`'}, 'kind': 'DuckDb', ...},
  'ok': False}
close-unknown: {'ok': True}
```

## Multi-statement documents against a session

`"document":true` on an exec request compiles a whole Turtle AST document
(`sql:StatementSequence`) via `compile_turtle_document` and evaluates it
member-by-member on the target connection — a named session or a fresh
connection, same routing (`src/duckdb_op.rs:383-384, 474`). Success
answers a **BatchReceipt**: `query_iri`, 64-hex `doc_hash`, `members`
(one `ExecutionReceipt` each, spine order), per-member `rows_emitted`,
`failed_at: null`.

Laws on this path:

- **No wire `params`** — a non-empty wire param list is the SQLRDF-17
  refusal; each member's literals live in the AST document itself
  (`src/duckdb_op.rs:480-487`).
- **Partial-commit semantics** — best-effort sequential autocommit
  (`src/duckdb_eval.rs:401-405`): a mid-sequence member failure answers
  the typed `BatchMemberFailed` envelope (`src/duckdb_op.rs:507-524`)
  whose `partial_receipt` holds exactly `members[0..failed_at]` — their
  effects **stay committed** (not a hidden rollback); `members[i..]` never
  executed (`BatchReceipt`, `src/duckdb_eval.rs:400-424`).

Witnessed success over the real wire (`document:true` exec against the
named session `howto-doc`, two-member CREATE→INSERT; exit 0):

```text
open: {'ok': True, 'session': 'howto-doc'}
batch: {"doc_hash": "df81804d5974869d3f435a656c49296c441fd0ed30797af8577878e6041fb19e",  # historical example-run digest
        "failed_at": null, "ok": true, "query_iri": "urn:doc-howto",
        "rows_emitted": [0, 1],
        "members": [
          {"query_iri": "urn:doc-howto#member-0", "rows_emitted": 0,
           "sql_statement": "CREATE TABLE doc_events (batch_id INTEGER)"},
          {"query_iri": "urn:doc-howto#member-1", "rows_emitted": 1,
           "result_triples": [
             {"subject": "urn:row//row_1", "predicate": "urn:p//Count",
              "object": {"Literal": {"lexical_form": "1",
                         "datatype": "http://www.w3.org/2001/XMLSchema#integer"}}}],
           "sql_statement": "INSERT INTO doc_events (batch_id) VALUES (42)"}]}
```

Court witness for the failing-member envelope: `src/duckdb_op.rs`
lib courts, `document_exec_failing_member_yields_typed_batch_member_failed`
(failing SELECT reading a missing table → `failed_at = 2`, members 0-1 in
the partial receipt with real committed effects).

## Worked example: CREATE → INSERT → SELECT → close

One host process, six frames. Driver:
`python3 /tmp/glaw_frames.py` (packet-4 framer; CREATE/INSERT documents
are the court-admitted FIBO fixtures `tests/fixtures/fibo/tc01_create.ttl`
and `tc01_insert.ttl`). Real output:

```text
--- 1. open session ---
{"ok": true, "session": "howto-demo"}
--- 2. exec CREATE (session=S) ---
{"cursor": "713c33ff90457f7431dd4831550e68c5f8acb67d536428643998f00c78719830",  # historical example-run digest
 "more": false, "ok": true,
 "query_hash": "357791cfab8c7c4090d0ba059a8bcfe136f2e9ddd68c5c48aeff0607bd718554",  # historical example-run digest
 "receipt": "http://example.org/data/receipt_357791cfab8c7c4090d0ba059a8bcfe136f2e",
 "rows_emitted": 0,
 "sql": "CREATE TABLE fibo_accounts (id INTEGER, balance DECIMAL, facility_id VARCHAR)"}
--- 3. exec INSERT (session=S) ---
{"cursor": "6b2d20b6561fcf73e96093f5cd43c254cd68e3b23d830ba7fdd9c7daa4c0b6e5",  # historical example-run digest
 "more": true, "ok": true,
 "query_hash": "3cad3a0725e6da2e9dc16e36890f1725be5acd66eb083dcb7b412c60dd2f0cd6",  # historical example-run digest
 "receipt": "http://example.org/data/receipt_3cad3a0725e6da2e9dc16e36890f1725be5a",
 "rows_emitted": 1,
 "sql": "INSERT INTO fibo_accounts (id, balance, facility_id) VALUES (1, 1250.50, 'FAC-CORP-9882')"}
--- 4. exec SELECT (session=S) ---
{"cursor": "294e628b394de4ab84ae0d68c621c4cb99a36f4f972128792b55d41e1b3587a2",  # historical example-run digest
 "more": true, "ok": true,
 "query_hash": "abe8c01f785d2be608d4d5f5713accbd5f23e737fc0dd4fda421695185145c15",  # historical example-run digest
 "receipt": "http://example.org/data/receipt_abe8c01f785d2be608d4d5f5713accbd5f23e",
 "rows_emitted": 1,
 "sql": "SELECT balance FROM fibo_accounts ORDER BY balance"}
--- 5. stream SELECT rows (session=S) ---
{"more": false,
 "nquads": "<http://example.org/data/row_1> <http://example.org/vocab/balance> \
\"1250.500\"^^<http://www.w3.org/2001/XMLSchema#decimal> .\n"}

--- 6. close session ---
{"ok": true}
[host exit: 0]
```

The SELECT in frame 4 reads the row INSERT put down in frame 3 — on the
session, not a fresh connection. Without `"session"` on frames 2-4, frame
4 would fail on a missing table.

Binary used: `graphlaw-run-eh` built from this tree (feature `runner-eh`,
`src/bin/graphlaw_run_eh.rs`) hosting `priv/graphlaw.wasm`. To replay:
build the host (`cargo build --features runner-eh --bin graphlaw-run-eh`),
then run a packet-4 framer that sends the six frames above to one host
process over stdin.

## See Also

- `docs/DIALECTS.md` — session + document-sequence conformance text
- `docs/wasm-integration.md` — packet-4 stdio protocol and EH host
- `tests/wasm_duckdb_lifecycle_eh.rs` — the black-box lifecycle court
- `src/duckdb_op.rs` — the op-15 surface (open/exec/stream/close)
