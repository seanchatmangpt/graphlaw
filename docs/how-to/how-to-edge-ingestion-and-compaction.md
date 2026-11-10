# How to: Edge Ingestion & Zero-Trust Semantic Compaction

This guide demonstrates how to configure `graphlaw` with its embedded `frozen-duckdb` analytical substrate to ingest dense local telemetry event streams, execute declarative SQL-in-RDF queries, and compact massive event streams into cryptographic state proofs. Reduction is workload-dependent; no fixed ratio is asserted (see `docs/explanation/edge-sovereignty-and-zero-trust-compaction.md`). As of 2026-11-09 the code below uses the vocabulary of `ontologies/sql-ast.ttl` and the signatures in `src/duckdb_eval.rs`; the snippets are illustrative (`rust,ignore`-class), not compiled doctests.

---

## Architecture Overview

```
[ Dense Raw Events (Sensors / Devices / Browser Sandbox) ]
                          │
                          ▼ Microsecond Appends
              [ In-Memory Columnar Buffer ]
              (frozen-duckdb Arrow/Parquet Views)
                          │
                          ▼ Window / Step Boundary Trigger
            [ SHACL Pre-Flight AST Conformance Gate ]
                          │
                          ▼ Vectorized Pushdown Execution
           [ Parameterized SQL & Columnar Reductions ]
                          │
                          ▼ Fact Lifting
             [ PurRDF Deductive Transition Rules ]
                          │
         ┌────────────────┴────────────────┐
         ▼                                 ▼
[ PASS: State Transition ]         [ FAIL-CLOSED Lockout ]
- Sealed State Transition          - Device Hardware Lockout
- BLAKE3 Execution Receipt         - Tamper-evident Audit Refusal
- Minimal RDF Proof Synced         - Zero Unverified Triples Transferred
```

---

## Step 1: Initialize the Embedded Engine

GraphLaw unconditionally includes `frozen-duckdb`. No optional feature flags are needed:

```rust
use graphlaw::duckdb_eval::GraphLawDuckDB;

// Initialize an in-memory or embedded disk connection
let engine = GraphLawDuckDB::open_in_memory()?;
let conn = engine.raw_connection();
```

---

## Step 2: Buffer Dense Event Telemetry

Create local tables or Parquet views for sensor telemetry:

```rust
conn.execute_batch(r#"
    CREATE TABLE telemetry_ring_buffer (
        event_id VARCHAR PRIMARY KEY,
        device_id VARCHAR,
        vibration_hz DOUBLE,
        temp_celsius DOUBLE,
        recorded_at TIMESTAMP
    );
"#)?;

// Ingest high-frequency telemetry events locally without network roundtrips
conn.execute_batch(r#"
    INSERT INTO telemetry_ring_buffer
    SELECT 'EVT-' || i, 'PUMP-TURBINE-01', 142.5 + i * 0.001, 38.2,
           TIMESTAMP '2026-10-21 14:00:00'
    FROM range(10000) t(i);
"#)?;
```

(A frozen-duckdb `Appender` also works for row-at-a-time ingest; the bulk
`INSERT ... SELECT` form above keeps the example free of per-row API detail.)

---

## Step 3: Define Declarative Aggregation AST in Turtle

Define the analytical reduction query as a formal RDF AST adhering to `ontologies/sql-ast.ttl`:

```turtle
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/> .

ex:edge_compaction_query a sql:SelectStatement ;
    sql:statementIndex 0 ;
    sql:fromTable [ sql:tableName "telemetry_ring_buffer" ] ;
    sql:selectList (
        [ sql:columnName "device_id" ]
        [ sql:aliasName "total_samples" ; sql:expression [
            sql:function <http://graphlaw.org/functions#COUNT> ;
            sql:argument [ sql:columnName "event_id" ] ] ]
        [ sql:aliasName "mean_vibration" ; sql:expression [
            sql:function <http://graphlaw.org/functions#AVG> ;
            sql:argument [ sql:columnName "vibration_hz" ] ] ]
        [ sql:aliasName "peak_temp" ; sql:expression [
            sql:function <http://graphlaw.org/functions#MAX> ;
            sql:argument [ sql:columnName "temp_celsius" ] ] ]
    ) ;
    sql:whereClause [
        sql:operator sql:GreaterThanOrEq ;
        sql:leftOperand [ sql:columnName "temp_celsius" ] ;
        sql:rightOperand [ sql:literalValue "30.0"^^xsd:double ]
    ] ;
    sql:groupBy ( [ sql:columnName "device_id" ] ) .
```

---

## Step 4: Validate AST through SHACL Pre-Flight Gate & Execute

The SHACL gate runs inside `compile_turtle`: the Turtle source is validated
against `ontologies/sql-ast.ttl` + `ggen/sql-dialect/statement_classes.ttl`
before any SQL reaches the engine, and parameter literals come from the AST
itself (`sql:literalValue`), not a separate parameter vector:

```rust
use graphlaw::sql_ast::compile_turtle;

// SHACL Pre-Flight Conformance Gate + compile to parameterized SQL
let compiled = compile_turtle(ast_turtle_str)?;

// Vectorized execution over local buffers
let receipt = engine.eval_ast_query(
    "http://example.org/edge_compaction_query",
    &compiled,
    "http://fleet.internal/subject/",
    "http://fleet.internal/predicate/",
)?;

println!("Compacted 10,000 raw events into {} triples", receipt.result_triples.len());
println!("Receipt BLAKE3 Hash: {}", hex::encode(receipt.query_hash));
```

---

## Step 5: Mint Signed Proof & Transition State

Bridge the returned tabular metrics into deductive state triples and assert verifiable transitions:

```rust
// Materialize only the verified summary triples (10,000 raw events -> one
// grouped row). The payload size is workload-dependent; this one is a few hundred bytes.
let sync_payload = format!(
    r#"
    @prefix dev: <http://fleet.internal/terms#> .
    @prefix gl:  <http://graphlaw.org/terms#> .

    dev:PUMP-TURBINE-01 a dev:MonitoredAsset ;
        dev:sampleCount 10000 ;
        dev:meanVibration 147.5 ;
        dev:peakTemp 38.2 ;
        gl:auditReceipt "{}" .
    "#,
    hex::encode(receipt.query_hash)
);
```

---

## Disconnected Fail-Closed Operations

When operating in denied, degraded, intermittent, or limited (DDIL) connectivity environments, GraphLaw guarantees **fail-closed security**:

1. **Typed Pre-Flight Refusals**: Malformed or non-conforming ASTs are stopped at the SHACL gate (a typed `AstError` carrying a stable `[REFUSAL-nn]` code, see `docs/refusals.md`) before any SQL touches the DuckDB execution engine. `sql:whereClause` is optional on a SELECT; it is mandatory on UPDATE/DELETE (REFUSAL-05).
2. **Safety Invariants**: Out-of-spec physical measurements (e.g., thermal limits or cumulative excursion durations) trigger automatic quarantine rules in PurRDF, locking actuators locally without requiring an upstream network coordinator.

---

## Execution Receipts and the Evidentiary Path

Every admitted statement seals its execution receipt as
`H_query = BLAKE3(Canon(G_AST) || P_params)` at execution time
(`src/duckdb_eval.rs`, `ExecutionReceipt::query_hash`); `Canon(G_AST)` is
computed once at compile (`purrdf::try_canonicalize`) and pre-absorbed into
`receipt_state`, so the per-receipt fold covers only the typed parameter
buffer. Mutation invalidation is a separate witness, not folded into
`H_query`: a mutating statement (DML/ATTACH/DETACH/DDL) bumps the C1 plan
epoch and flushes frozen-duckdb's prepared-statement cache after successful
execution only, so a failed statement never invalidates. The SPSC evidentiary
ring (`src/evidence_ring.rs`) and batch-chained query ledger
(`src/query_ledger.rs`) are Phase-2 surfaces behind the `evidentiary-ring`
Cargo feature (default off). Honest caveat: prepared statements
are cached per connection in an LRU pinned to 256
(`PREPARED_STATEMENT_CACHE_CAPACITY`, applied at construction via
`set_prepared_statement_cache_capacity`); plans are
flushed on any stored-data mutation so no stale plan survives a schema or
data change.
