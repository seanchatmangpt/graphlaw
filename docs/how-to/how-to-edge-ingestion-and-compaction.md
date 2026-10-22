# How to: Edge Ingestion & Zero-Trust Semantic Compaction

This guide demonstrates how to configure `graphlaw` with its embedded `frozen-duckdb` analytical substrate to ingest dense local telemetry event streams, execute declarative SQL-in-RDF queries, and compact massive event streams into cryptographic state proofs with 1,000:1 to 10,000:1 reduction.

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
let mut appender = conn.appender("telemetry_ring_buffer")?;
for i in 0..10_000 {
    appender.append_row([
        format!("EVT-{i}"),
        "PUMP-TURBINE-01".to_string(),
        142.5 + (i as f64 * 0.001),
        38.2,
        "2026-10-21 14:00:00".to_string(),
    ])?;
}
appender.flush()?;
```

---

## Step 3: Define Declarative Aggregation AST in Turtle

Define the analytical reduction query as a formal RDF AST adhering to `ontologies/sql-ast.ttl`:

```turtle
@prefix sql: <http://graphlaw.org/sql#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

:edge_compaction_query a sql:SelectQuery ;
    sql:selectClause [
        sql:columns (
            [ sql:column "device_id" ]
            [ sql:aggregate "COUNT" ; sql:argument "event_id" ; sql:alias "total_samples" ]
            [ sql:aggregate "AVG"   ; sql:argument "vibration_hz" ; sql:alias "mean_vibration" ]
            [ sql:aggregate "MAX"   ; sql:argument "temp_celsius" ; sql:alias "peak_temp" ]
        )
    ] ;
    sql:fromClause [ sql:tableName "telemetry_ring_buffer" ] ;
    sql:whereClause [
        sql:condition [
            sql:leftColumn "temp_celsius" ;
            sql:operator ">=" ;
            sql:rightParam "$1"
        ]
    ] ;
    sql:groupByClause [ sql:columns ( "device_id" ) ] .
```

---

## Step 4: Validate AST through SHACL Pre-Flight Gate & Execute

The SHACL gate runs inside `compile_turtle`: the Turtle source is validated
against `ontologies/sql-ast.ttl` + `ggen/sql-dialect/statement_classes.ttl`
before any SQL reaches the engine, and parameter literals come from the AST
itself (`sql:rightParam`), not a separate parameter vector:

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
println!("Receipt BLAKE3 Hash: {:?}", receipt.query_hash);
```

---

## Step 5: Mint Signed Proof & Transition State

Bridge the returned tabular metrics into deductive state triples and assert verifiable transitions:

```rust
// Materialize only the verified summary triples (10,000 raw events -> 4 triples)
// Upstream sync payload is under 1 KB instead of multi-megabyte raw logs.
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
    receipt.query_hash
);
```

---

## Disconnected Fail-Closed Operations

When operating in denied, degraded, intermittent, or limited (DDIL) connectivity environments, GraphLaw guarantees **fail-closed security**:

1. **Typed Pre-Flight Refusals**: Malformed or non-conforming ASTs are stopped at the SHACL gate (`AstError::ShaclViolation` carrying a stable `[REFUSAL-nn]` code, see `docs/refusals.md`) before any SQL touches the DuckDB execution engine. `sql:whereClause` is optional, not mandatory.
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
Cargo feature (default off; P28-SIL PRD). Honest caveat: prepared statements
are cached per connection in an LRU pinned to 256
(`set_prepared_statement_cache_capacity(256)` at construction); plans are
flushed on any stored-data mutation so no stale plan survives a schema or
data change.
