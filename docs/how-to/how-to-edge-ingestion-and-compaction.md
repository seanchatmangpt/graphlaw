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
let engine = GraphLawDuckDB::new_in_memory()?;
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

Validate that the AST is safe, non-injected, and has mandatory bounded conditions:

```rust
use graphlaw::sql_ast::{validate_ast_shacl, RdfAstGraph};
use graphlaw::duckdb_eval::ParamValue;

// Parse the Turtle AST
let ast_graph = RdfAstGraph::from_turtle(ast_turtle_str)?;

// SHACL Pre-Flight Conformance Gate
validate_ast_shacl(&ast_graph)?;

// Vectorized execution over local buffers
let params = vec![ParamValue::Float(30.0)];
let (results, receipt) = engine.eval_ast_query(&ast_graph, "http://example.org/edge_compaction_query", &params)?;

println!("Compacted 10,000 raw events in {} ms", receipt.duration_ms);
println!("Receipt BLAKE3 Hash: {}", receipt.query_hash);
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

1. **Missing WHERE Bounding**: Queries omitting filtering boundaries are stopped at the SHACL gate (`AstError::ShapeViolation`) before any SQL touches the DuckDB execution engine.
2. **Safety Invariants**: Out-of-spec physical measurements (e.g., thermal limits or cumulative excursion durations) trigger automatic quarantine rules in PurRDF, locking actuators locally without requiring an upstream network coordinator.
