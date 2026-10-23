//! sql_rdf_egress — Divan benchmarks for egress serialization: RDF triple
//! materialization + N-Triples rendering (`ExecutionReceipt.result_triples`,
//! the P2 target) vs raw row-value-bytes BLAKE3 digest (the columnar-digest
//! proxy; there is no `as_columnar_slice` on the frozen-duckdb surface).
//!
//! Chicago: a real `GraphLawDuckDB::open_in_memory` connection, a real 10k-row
//! table seeded via raw SQL (`CREATE TABLE t AS SELECT i AS id, i*1.5 AS val
//! FROM range(10000) t(i)`), and real BLAKE3 digests over real row bytes.
//! No mocks, no sleeps. ENGINE_LOCK is taken once in setup and held for the
//! process lifetime (engine-heavy eval serialized against other suites).
//!
//! The N-Triples renderer mirrors `src/duckdb_op.rs::ntriples_line` (private),
//! reading `purrdf::TermValue` directly — same output shape, no mocks.

use std::hint::black_box;
use std::sync::{Mutex, OnceLock};

use graphlaw::duckdb_eval::{ExecutionReceipt, GraphLawDuckDB};
use graphlaw::purrdf::TermValue;
use graphlaw::sql_ast::compile_turtle;

const ROWS: i64 = 10_000;

/// Estimated N-Triples egress bytes per row for the seeded shape:
/// subject `urn:r:/row_NNNNNN` (~16 B) + two predicates (`urn:p:/id`,
/// `urn:p:/val`, ~10 B each) + literal objects (`"NNNN"^^<...#integer>` ~35 B,
/// `"N.NN E0"^^<...#double>` ~55 B) + separators — ~90 B per triple.
const NT_BYTES_PER_ROW: u64 = 180;

/// Raw columnar bytes per row for the seeded shape: `id` BIGINT (8 B) +
/// `val` DOUBLE (8 B).
const RAW_BYTES_PER_ROW: u64 = 16;

/// Engine lock: taken once in setup, held for process lifetime.
static ENGINE_LOCK: OnceLock<()> = OnceLock::new();
static SETUP: OnceLock<Mutex<Setup>> = OnceLock::new();

struct Setup {
    db: GraphLawDuckDB,
    compiled: graphlaw::sql_ast::CompiledQuery,
}

fn setup() -> std::sync::MutexGuard<'static, Setup> {
    SETUP.get_or_init(|| {
        Mutex::new(build_setup())
    }).lock().unwrap_or_else(|e| e.into_inner())
}

fn build_setup() -> Setup {
        ENGINE_LOCK.get_or_init(|| {
            // Hold the engine mutex for the process lifetime: this bench
            // binary is the only engine user in its process.
            let _guard = engine_lock();
        });
        let db = GraphLawDuckDB::open_in_memory().expect("in-memory frozen-duckdb");
        db.raw_connection()
            .execute(
                &format!("CREATE TABLE t AS SELECT i AS id, i*1.5 AS val FROM range({ROWS}) t(i)"),
                [],
            )
            .expect("seed 10k-row table");
        let compiled = compile_turtle(SELECT_TURTLE).expect("compile SELECT turtle");
        Setup { db, compiled }
}

fn engine_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

const SELECT_TURTLE: &str = r#"
@prefix sql: <http://graphlaw.org/sql#> .
@prefix ex:  <http://example.org/queries#> .

ex:Q a sql:SelectStatement ;
    sql:statementIndex 1 ;
    sql:fromTable [ sql:tableName "t" ] ;
    sql:selectList ( [ sql:columnName "id" ] [ sql:columnName "val" ] ) .
"#;

/// Leg 1 — the P2 target: full egress materialization through the eval path
/// (`result_triples`: TermValue literals + row/predicate IRIs per cell).
#[divan::bench(bytes_count = ROWS as u64 * NT_BYTES_PER_ROW)]
fn egress_result_triples() -> ExecutionReceipt {
    let s = setup();
    black_box(
        s.db
            .eval_ast_query("http://example.org/egress", &s.compiled, "urn:r:", "urn:p:")
            .expect("10k-row eval"),
    )
}

/// Leg 2 — N-Triples string formatting over an already-materialized
/// `result_triples` vector (10k rows = 20k triples), mirroring
/// `duckdb_op::ntriples_line`.
#[divan::bench(bytes_count = ROWS as u64 * NT_BYTES_PER_ROW)]
fn ntriples_format() -> usize {
    let s = setup();
    let receipt = s
        .db
        .eval_ast_query("http://example.org/egress", &s.compiled, "urn:r:", "urn:p:")
        .expect("10k-row eval");
    let triples = &receipt.result_triples;
    assert_eq!(triples.len(), (ROWS * 2) as usize, "2 triples per row");
    let mut total = 0usize;
    for t in triples {
        let obj = match &t.object {
            TermValue::Iri(i) => format!("<{i}>"),
            TermValue::Blank { label, .. } => format!("_:{label}"),
            TermValue::Literal {
                lexical_form,
                datatype,
                ..
            } => {
                let escaped = escape_nt(lexical_form);
                match datatype.strip_prefix("http://www.w3.org/2001/XMLSchema#") {
                    Some("string") => format!("\"{escaped}\""),
                    Some(dt) => format!("\"{escaped}\"^^<http://www.w3.org/2001/XMLSchema#{dt}>"),
                    None => format!("\"{escaped}\"^^<{datatype}>"),
                }
            }
            TermValue::Triple { .. } => continue,
        };
        total += black_box(format!(
            "<{}> <{}> {} .\n",
            t.subject, t.predicate, obj
        ))
        .len();
    }
    total
}

/// Leg 3 — columnar-digest proxy: BLAKE3 over raw row value bytes read via
/// frozen-duckdb row getters (`ValueRef::Int` / `ValueRef::Double`), the same
/// getter seam `duckdb_eval`'s row loop uses. No string rendering anywhere.
#[divan::bench(bytes_count = ROWS as u64 * RAW_BYTES_PER_ROW)]
fn columnar_digest_proxy() -> [u8; 32] {
    let s = setup();
    let mut stmt = s
        .db
        .raw_connection()
        .prepare("SELECT id, val FROM t")
        .expect("prepare raw scan");
    let mut rows = stmt.query([]).expect("query raw scan");
    let mut hasher = blake3::Hasher::new();
    while let Some(row) = rows.next().expect("row") {
        for col in 0..2 {
            match row.get_ref(col).expect("value ref") {
                frozen_duckdb::duckdb::types::ValueRef::Int(i) => {
                    hasher.update(&i.to_ne_bytes());
                }
                frozen_duckdb::duckdb::types::ValueRef::BigInt(i) => {
                    hasher.update(&i.to_ne_bytes());
                }
                frozen_duckdb::duckdb::types::ValueRef::Double(d) => {
                    hasher.update(&d.to_ne_bytes());
                }
                other => panic!("unexpected ValueRef variant in seeded table: {other:?}"),
            }
        }
    }
    *black_box(hasher.finalize().as_bytes())
}

/// N-Triples string escaping (backslash and quote), per `duckdb_op::escape_nt`.
fn escape_nt(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn main() {
    divan::main();
}
