# SQLRDF-17 — ABI duckdb exec accepts raw SQL text (C2 hole P1)

- **Id:** SQLRDF-17
- **Title:** ABI duckdb exec accepts raw SQL text (C2 hole P1) — spin-out of
  moonshot row C2 from [SQLRDF-16](SQLRDF-16.md)
- **Status:** Open (follow-up; spin-out of SQLRDF-16 row C2)
- **Standing:** **BLOCKED-BY: v26.10.22 integration commit** — no fix lands
  before the coordinator integrates the v26.10.22 sql-rdf wave (same gate as
  SQLRDF-16). File-of-record for the C2 falsifier
  ("Raw-SQL admissions > 0 — any admission path accepts raw SQL text without a
  compiled AST graph / typed refusal").
- **Source:** C2 inventory's product hole P1 (THESIS C2 lane report).

## Current behavior (verified on disk, 2026-10-22)

- [`src/duckdb_op.rs:198-219`](../../../src/duckdb_op.rs) — the ABI `duckdb`
  op takes an `ast` object, reads `str_field(ast, "text")` (line 203), and
  feeds that string straight into `CompiledQuery { sql, ...,
  ast_graph_turtle: None, canon_graph_bytes: None }` (lines 211-219). Raw SQL
  text executes with no `compile_turtle` pass, no AST graph, no receipt over
  `BLAKE3(Canon(G_AST) || P_params)`.
- [`src/duckdb_eval.rs:178-189`](../../../src/duckdb_eval.rs) — the legacy
  `(None, None)` hash arm (lines 183-188) hashes `compiled.sql` as bytes plus
  params. This arm is the fingerprint of the hole: every admission through it
  is a raw-SQL admission that never saw the canonical AST graph.
- [`src/dialect.rs:740-758`](../../../src/dialect.rs) — sniff misroute: a bare
  SQL document whose first keyword is `SELECT`/`INSERT`/... matches the SPARQL
  keyword set and routes to `Dialect::Sparql`, producing a Sparql skeleton
  instead of a `BareSql`-style typed refusal. There is no `BareSql` reason
  class in the dialect layer today.
- [`src/duckdb_eval.rs:155`](../../../src/duckdb_eval.rs) —
  `raw_connection()` is `pub`, exposing the interior frozen-duckdb connection
  to any caller; call sites: `src/duckdb_op.rs:147`, `src/duckdb_eval.rs:461`,
  `src/duckdb_eval.rs:563` (all in-crate today, so the gate is containment
  work, not an escape in progress).

## Required behavior

1. **No raw-SQL admissions.** The ABI duckdb op accepts a Turtle AST document
   (compiled via `compile_turtle` → `canon_graph_bytes`), never bare SQL
   text. Alternative minimal form: refuse the legacy `(None, None)` hash arm
   as a typed refusal (`LegacySqlHashRefused`, stable code in
   [`docs/refusals.md`](../../refusals.md)).
2. **BareSql reason class.** The dialect sniffer gains a `BareSql` reason
   class: a document whose skeleton is SQL (leading SQL keyword, no RDF
   envelope) refuses with that class instead of misrouting to the Sparql
   skeleton.
3. **Trusted-interior gating.** `raw_connection()` becomes `pub(crate)` or
   gated behind a `TrustedInterior` marker so the interior connection is not
   part of the public admission surface.
4. **All three sub-holes close together or none do** — the census falsifier
   (below) counts admissions, so closing only the op while the sniff misroute
   or the `pub` accessor remains still yields raw-SQL admissions > 0.

## Falsifier / Acceptance

The C2 falsifier is the census test (design in the THESIS C2 lane report):

- **Static**: grep/allowlist census — zero call sites construct
  `CompiledQuery` with `ast_graph_turtle: None, canon_graph_bytes: None`
  outside a named allowlist (which shrinks to empty at closure).
- **Dynamic**: a typed-refusal probe on every ABI op surface — feeding raw
  SQL text to each op must return the typed refusal (`BareSql` /
  `LegacySqlHashRefused`), never an executed statement.
- **No-legacy-arm assertion**: a compile-time or court assertion that the
  `(None, None)` hash arm at `src/duckdb_eval.rs:183-188` is unreachable
  (removed or refused before hash computation).

- **Acceptance run:** `cargo test --test chicago_validation` plus the census
  court landed with this ticket; anti-vacuity via
  `bash scripts/mutation_court.sh` must flip on an injected raw-SQL admission
  path. A gate that cannot exit 1 on the injected path is FAIL.
- Real receipt with embedded command output lands in
  [RECEIPTS.md](RECEIPTS.md) at closure; a receipt with no embedded output is
  not a receipt.

## Already-implemented boundary

The graph-backed path exists and is receipted: `compile_turtle` →
`canon_graph_bytes` → `admitted_statement_hash` over
`BLAKE3(Canon(G_AST) || P_params)` (`src/duckdb_eval.rs:170-182`). This
ticket does not re-ask for it; it asks that the ABI op use it and that the
raw-SQL bypass routes be typed refusals.

## Finding F2 (2026-10-22, docs lane): op-15 exec mints receipts for non-executed statements

Beyond admitting raw SQL (P1 above), the legacy `mode:"exec"` path mints a
receipt IRI, query hash and cursor for **DDL text whose effect is never
observable** — the receipt asserts an execution that the engine cannot
subsequently witness. Mechanism:

- `src/duckdb_op.rs:74-82` — `open_connection()` opens a **fresh in-memory
  duckdb per `exec` call**. Any catalog mutation (CREATE TABLE, CTAS, INSERT)
  inside one exec dies with that connection; nothing persists to a readback.
- `src/duckdb_op.rs:231-261` — despite that, `exec` unconditionally mints
  `receipt_iri` + `query_hash` + `cursor` from the returned
  `ExecutionReceipt` and returns `ok:true`.
- `src/duckdb_eval.rs:352-354` / `:543-552` — `prepare_cached` accepts DDL,
  and the receipt (`rows_emitted = row_index`) is formed even for statements
  that produce no query result.

**Probe receipt (real output, this tree, 2026-10-22):** a temporary probe
(test file deleted after capture) ran the op-15 wire path
(`graphlaw::duckdb_op::dispatch`, `mode:"exec"`) twice plus readback:

```
EXEC_DDL_OUT = Ok(Object {"cursor": String("9f204e3fb0e8dff6884d09fdf99ab4ee4fb599873c7125f2e405e59d6e592926"), "more": Bool(false), "query_hash": String("590996ead1802c490e077107dcf5c38ead2ca289a840f6ef33b95a6d21af7ea4"), "receipt": String("https://example.org/f2/s/receipt_590996ead1802c49"), "rows_emitted": Number(0), "sql": String("CREATE TABLE f2_probe(x INTEGER)")})  # historical: probe capture 2026-10-22, test deleted after capture (cursor/query_hash digests not registry artifacts)
EXEC_DDL_AS_SELECT_OUT = Ok(Object {"cursor": String("b390b9a179973da6c63918c786d3c89d7db026107794425ad2f2aac153f4c5a0"), "more": Bool(true), "query_hash": String("4520423323c67dccd991ffb4fccfb964af3b887099df9e3b29b47619bef86036"), "receipt": String("https://example.org/f2/s/receipt_4520423323c67dcc"), "rows_emitted": Number(1), "sql": String("CREATE TABLE f2_probe2 AS SELECT 42 AS v")})  # historical: probe capture 2026-10-22, test deleted after capture (cursor/query_hash digests not registry artifacts)
READBACK_OUT = Err(String("Fail { refusal: Refusal { kind: DuckDb, ..., message: \"duckdb: DuckDB execution failure: Catalog Error: Table with name f2_probe does not exist!\\nDid you mean \\\"pg_attribute\\\"?\\n\\nLINE 1: SELECT * FROM f2_probe\\n                      ^^^^^^^^\" }, ... }"))
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

The CTAS variant mints a receipt with `rows_emitted: 1` and a streamable
cursor (`more: true`) for DDL; a follow-up readback exec gets **Catalog
Error: Table with name f2_probe does not exist!** — the receipted statement
left nothing behind to read back. (Two compounding causes: per-call
in-memory connections, and receipts minted for non-result statements.)

**Required behavior (extends Required behavior 1):** `mode:"exec"` must
either (a) refuse non-query / catalog-mutating statements as a typed refusal
before `eval_ast_query`, or (b) hold one durable connection per ABI session
so the receipted effect is actually replayable — never mint a receipt for a
statement whose consequence is unreachable. The readback Catalog Error is
the falsifier: a receipted statement with no subsequent witness is a
receipt-minting defect under the repo's R=receipt(A) law.
