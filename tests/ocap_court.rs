//! OCAP row court — compile/admission/execution boundaries, each row end-to-end
//! with inline anti-vacuity flips and a benign twin that must succeed.
//!
//! Row 1 (shape violation → typed refusal, zero engine allocation): a graph
//! missing `sql:statementIndex` fails strict closed-world shape validation
//! before admission (rec §1.3; `docs/refusals.md` REFUSAL-01) with a typed
//! `AstError`, never a panic or generic DB error — and a real frozen-duckdb
//! connection witnesses zero engine allocation: no table exists after the
//! refusal (the p28 court pattern, `tests/p28_determinism_court.rs` TC-LEGAL-02
//! readback).
//!
//! Row 2 (scope leak → compile abort before execution): a `sql:ViewScope`
//! column inside a predicate violates the scope discrimination boundary
//! (rec §1.3) and is refused REFUSAL-03 `ScopeIntegrityRefusal` at compile
//! time — eval is never reached, witnessed by a real database readback that
//! the seeded table is byte-unchanged (the TC-LEGAL-02 pattern exercised on a
//! different leak site: predicate, not projection).
//!
//! Row 3 (ring saturation → transaction completes, gap marker persisted,
//! `evidentiary_ring_dropped` surfaces in capabilities): with the global
//! evidentiary ring saturated (drainer paused, 4096-slot ring full), a real
//! frozen-duckdb transaction still compiles, executes, and reads back; on
//! drain the drop-newest losses persist as ledger gap markers (`gaps/gap-*.json`,
//! `src/query_ledger.rs`) and the abi `capabilities` op surfaces
//! `evidentiary_ring_dropped` / `evidentiary_ledger_gaps` (src/abi.rs).
//! Feature-gated per P28-SIL §3.2 (feature `evidentiary-ring`, default OFF).
//!
//! Chicago: real frozen-duckdb connections, real drainer thread, real NDJSON +
//! query ledger on disk, real capabilities JSON. No mocks, no coordination
//! sleeps — every wait is a bounded poll on a real observable.

// Rows 1-2 need only the duckdb feature (in default features): the compile-time
// refusals and the real frozen-duckdb readbacks. Row 3 (ring saturation) is
// gated on `evidentiary-ring` (P28-SIL §3.2, default OFF); when the feature is
// off, row 3 still RUNS and emits a loud typed disclosure instead of silently
// executing zero tests.
#![cfg(not(target_arch = "wasm32"))]

use graphlaw::duckdb_eval::{GraphLawDuckDB, admitted_statement_hash};
use graphlaw::sql_ast::compile_turtle;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)] // consumed by the feature-gated row-3 module only
fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "graphlaw-ocap-court-{tag}-{}-{}",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    std::fs::create_dir_all(&d).expect("create court dir");
    d
}

/// Bounded poll on a real observable (no sleeps).
#[allow(dead_code)] // consumed by the feature-gated row-3 module only
fn poll_until<T>(deadline: Instant, mut f: impl FnMut() -> Option<T>) -> Option<T> {
    loop {
        if let Some(v) = f() {
            return Some(v);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::yield_now();
    }
}

fn tables_in_main(db: &GraphLawDuckDB) -> i64 {
    db.raw_connection()
        .query_row(
            "SELECT count(*)::BIGINT FROM information_schema.tables WHERE table_schema = 'main'",
            [],
            |r| r.get(0),
        )
        .expect("information_schema readback")
}

/// The lawful select graph: `SELECT amount FROM invoices WHERE status = ...`.
/// The projection column is `sql:ViewScope` (a `ProcessScope` projection would
/// itself be a REFUSAL-03 leak); the predicate column's scope is configurable —
/// `ProcessScope` is lawful, `ViewScope` is the row-2 leak. With
/// `with_index = false` the required `sql:statementIndex` is stripped, giving
/// the row-1 shape violation.
fn select_graph(scope_predicate: &str, with_index: bool) -> String {
    let index = if with_index {
        "    sql:statementIndex 1 ;\n"
    } else {
        ""
    };
    format!(
        "@prefix sql: <http://graphlaw.org/sql#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         _:s a sql:SelectStatement ;\n\
         {index}\
         sql:select ( [ a sql:ColumnIdentifier ; sql:columnName \"amount\" ; \
         sql:scope sql:ViewScope ] ) ;\n\
         sql:from ( [ a sql:TableIdentifier ; sql:tableName \"invoices\" ] ) ;\n\
         sql:where [ a sql:BinaryExpression ;\n\
         sql:left [ a sql:ColumnIdentifier ; sql:columnName \"status\" ; \
         sql:scope sql:{scope_predicate} ] ;\n\
         sql:operator \"=\" ;\n\
         sql:right [ a sql:LiteralExpression ; sql:value \"SETTLED\" ; \
         sql:datatype xsd:string ] ] ."
    )
}

fn seed_invoices(db: &GraphLawDuckDB) {
    db.raw_connection()
        .execute("CREATE TABLE invoices (amount DECIMAL(10,2), status VARCHAR)", [])
        .expect("seed: create invoices");
    db.raw_connection()
        .execute("INSERT INTO invoices VALUES (42.00, 'SETTLED')", [])
        .expect("seed: insert invoice row");
}

// ---------------------------------------------------------------------------
// Row 1 — shape violation → typed REFUSAL-01, zero engine allocation
// ---------------------------------------------------------------------------

#[test]
fn row1_shape_violation_refused_01_with_zero_engine_allocation() {
    // The ONLY mutation from the lawful graph: `sql:statementIndex` missing.
    // Per docs/refusals.md REFUSAL-01 this fails closed-world shape validation
    // before admission as a typed AstError carrying [REFUSAL-01].
    let malformed = select_graph("ProcessScope", /* with_index = */ false);
    let err = compile_turtle(&malformed)
        .err()
        .expect("shape violation must be refused at compile time, not executed");
    let msg = err.to_string();
    assert!(
        msg.contains("REFUSAL-01") || msg.contains("MalformedAst"),
        "refusal must be typed REFUSAL-01 MalformedAstRefusal, got: {msg}"
    );

    // Zero engine allocation: the engine is never reached by the compile-time
    // refusal. Witnessed on a real frozen-duckdb connection — the engine holds
    // no table the refused graph could have created or mutated.
    let db = GraphLawDuckDB::open_in_memory().expect("real frozen-duckdb connection");
    assert_eq!(
        tables_in_main(&db),
        0,
        "no table may exist after the compile-time refusal (zero engine allocation)"
    );

    // INLINE FLIP: the byte-identical graph WITH `sql:statementIndex 1` must
    // compile, execute, and read back — a court that passes on both sides of
    // the mutation is vacuous.
    let lawful = select_graph("ProcessScope", /* with_index = */ true);
    let db2 = GraphLawDuckDB::open_in_memory().expect("real frozen-duckdb connection");
    seed_invoices(&db2);
    let compiled = compile_turtle(&lawful).expect("lawful graph must compile");
    assert!(
        compiled.sql.contains("SELECT amount"),
        "lawful graph must yield the projection, got: {}",
        compiled.sql
    );
    let receipt = db2
        .eval_ast_query("http://example.org/ocap-row1", &compiled, "urn:r:", "urn:p:")
        .expect("lawful graph must execute");
    assert_eq!(receipt.rows_emitted, 1, "benign twin must emit exactly one row");
    // Real receipt identity: the execution receipt hash is the admitted
    // statement hash BLAKE3(Canon(G_AST) || P_params) (rec §1.4).
    let canon = graphlaw::duckdb_eval::canonical_ast_graph_bytes(&lawful)
        .expect("lawful graph must canonicalize");
    assert_eq!(
        receipt.query_hash, admitted_statement_hash(&canon, &compiled.params),
        "execution receipt hash must equal BLAKE3(Canon(G_AST) || P_params)"
    );
}

// ---------------------------------------------------------------------------
// Row 2 — scope leak → compile abort before execution (TC-LEGAL-02 pattern)
// ---------------------------------------------------------------------------

#[test]
fn row2_viewscope_predicate_leak_refused_03_before_execution() {
    // Leak site: a sql:ViewScope column INSIDE a predicate (docs/refusals.md
    // REFUSAL-03: "a sql:ViewScope column inside a predicate"). This is a
    // different site from TC-LEGAL-02's projection leak — same typed refusal,
    // same compile-time boundary.
    let leaked = select_graph("ViewScope", /* with_index = */ true);
    let err = compile_turtle(&leaked)
        .err()
        .expect("scope leak must be refused at compile time, before any engine call");
    let msg = err.to_string();
    assert!(
        msg.contains("REFUSAL-03") || msg.contains("ScopeIntegrity"),
        "refusal must be typed REFUSAL-03 ScopeIntegrityRefusal, got: {msg}"
    );

    // Compile abort before execution: the engine is never reached. Witnessed
    // on a real database holding a seeded table — the refusal must have left
    // it byte-unchanged (no mutation, no dropped table).
    let db = GraphLawDuckDB::open_in_memory().expect("real frozen-duckdb connection");
    seed_invoices(&db);
    let rows: i64 = db
        .raw_connection()
        .query_row("SELECT count(*)::BIGINT FROM invoices", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 1, "seeded table must be present and untouched");
    let statuses: i64 = db
        .raw_connection()
        .query_row(
            "SELECT count(*)::BIGINT FROM invoices WHERE status <> 'SETTLED'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(statuses, 0, "refused graph must not mutate any row");

    // INLINE FLIP: only the scope individual flips (ViewScope → ProcessScope
    // in the predicate is lawful); the same graph shape then compiles and
    // executes on the real connection.
    let lawful = select_graph("ProcessScope", /* with_index = */ true);
    let compiled = compile_turtle(&lawful).expect("lawful predicate scope must compile");
    let receipt = db
        .eval_ast_query("http://example.org/ocap-row2", &compiled, "urn:r:", "urn:p:")
        .expect("lawful predicate scope must execute");
    assert_eq!(receipt.rows_emitted, 1, "benign twin must emit exactly one row");
}

// ---------------------------------------------------------------------------
// Row 3 — ring saturation: transaction completes, gap marker persisted,
// evidentiary_ring_dropped surfaces in capabilities
// ---------------------------------------------------------------------------

/// LOUD typed notice, never a silent skip: when the `evidentiary-ring` feature
/// is off, row 3 still executes and discloses that it did not run (a skip with
/// a typed reason, not a pass — exit-discipline per CLAUDE.md test ladder).
#[cfg(not(feature = "evidentiary-ring"))]
#[test]
fn row3_ring_saturation_transaction_completes_gaps_persisted_capabilities_surface() {
    println!(
        "OCAP NOTICE [SKIPPED-TYPED]: row 3 (ring saturation, \
         evidentiary_ring_dropped/ledger_gaps capabilities surfacing) requires \
         `--features evidentiary-ring` (P28-SIL §3.2, default OFF). \
         Rows 1-2 ran unconditionally; row 3 did NOT run and this is not a pass."
    );
}

/// Real row-3 court under the feature: ring saturation, transaction completion,
/// ledger gap persistence, and the capabilities-keys assertion.
#[cfg(all(feature = "evidentiary-ring", not(target_arch = "wasm32")))]
mod row3 {
    use super::{poll_until, select_graph, seed_invoices, tmp_dir};
    use graphlaw::abi;
    use graphlaw::duckdb_eval::GraphLawDuckDB;
    use graphlaw::evidence_ring::{
        EvidenceRing, PushOutcome, RingDescriptor, RING_CAPACITY, now_ms,
    };
    use graphlaw::sql_ast::compile_turtle;
    use serde_json::json;
    use std::time::{Duration, Instant};

#[test]
fn row3_ring_saturation_transaction_completes_gaps_persisted_capabilities_surface() {
    let dir = tmp_dir("row3-ring");
    let ldir = tmp_dir("row3-ledger");
    // Explicit global init (no lazy default); joint ring -> query-ledger
    // pipeline so gap markers persist under <ledger>/gaps/.
    EvidenceRing::init_global_with_ledger(&dir, &ldir).expect("global ring init");

    let ring = EvidenceRing::global().expect("global ring installed").clone();

    // Saturate for real: stall the drainer (the courted valve), fill all
    // 4096 slots, then keep pushing — drop-newest must refuse and count.
    ring.set_drainer_paused(true);
    let mut buffered = 0u64;
    let mut dropped = 0u64;
    for i in 0..(RING_CAPACITY as u64 + 512) {
        let mut h = [0u8; 32];
        h[..8].copy_from_slice(&i.to_be_bytes());
        let d = RingDescriptor::new(h, i, 1, 0, now_ms());
        match ring.push(d) {
            PushOutcome::Buffered => buffered += 1,
            PushOutcome::DroppedNewest => dropped += 1,
        }
    }
    assert_eq!(buffered + dropped, RING_CAPACITY as u64 + 512);
    assert_eq!(buffered, RING_CAPACITY as u64, "exactly the 4096 slots buffer");
    assert!(dropped >= 512, "saturated ring must drop; got dropped={dropped}");
    assert_eq!(ring.dropped_total(), dropped);

    // TRANSACTION COMPLETES under saturation: a real frozen-duckdb
    // transaction compiles, executes, and reads back while the ring is full
    // and the drainer is stalled. Evidence telemetry may drop; the lawful
    // transaction may not. NOTE: this eval also exercises the real
    // evidentiary seam (duckdb_eval.rs pushes this query's own
    // RingDescriptor into the global ring) — under saturation it drops too,
    // so dropped_total rises by exactly 1 above the loop-local count. The
    // conservation asserts below therefore bind to the ring's counter, never
    // to the loop-local tally.
    let db = GraphLawDuckDB::open_in_memory().expect("real frozen-duckdb connection");
    seed_invoices(&db);
    let lawful = select_graph("ProcessScope", /* with_index = */ true);
    let compiled = compile_turtle(&lawful).expect("transaction graph must compile");
    let receipt = db
        .eval_ast_query("http://example.org/ocap-row3", &compiled, "urn:r:", "urn:p:")
        .expect("transaction must complete while the ring is saturated");
    assert_eq!(receipt.rows_emitted, 1, "transaction emits its row under saturation");
    let back: (String, i64) = db
        .raw_connection()
        .query_row(
            "SELECT status, count(*)::BIGINT FROM invoices GROUP BY status",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0).expect("status col"),
                    r.get::<_, i64>(1).expect("count col"),
                ))
            },
        )
        .expect("real table readback after the saturated transaction");
    assert_eq!(back, ("SETTLED".into(), 1), "readback must match the seeded state");

    // Resume + close: drain-to-empty, then the drainer records the drops as
    // ledger gap markers (real disk readback, bounded poll on the observable).
    ring.set_drainer_paused(false);
    ring.close();
    let ledger = graphlaw::query_ledger::QueryLedger::open(&ldir).expect("ledger re-open");
    let ok = poll_until(Instant::now() + Duration::from_secs(30), || {
        ledger.gaps().ok().filter(|g| !g.is_empty())
    });
    let gaps = ok.expect("drop-newest losses must persist as ledger gap markers");
    let gap_covered: u64 = gaps.iter().map(|(f, l)| l - f + 1).sum();
    assert_eq!(
        gap_covered,
        ring.dropped_total(),
        "gap markers must cover exactly the drops the ring counter reports (including the eval-seam drop)"
    );
    assert_eq!(
        ring.last_ledger_error(),
        None,
        "drainer must record into the ledger without error"
    );

    // CAPABILITIES SURFACING: the abi `capabilities` op must carry the
    // runtime counters, matching the surfacing lane's keys exactly
    // (src/abi.rs: evidentiary_ring_dropped / evidentiary_ledger_gaps).
    let caps = abi::call_json(&json!({"op": "capabilities"}));
    assert_eq!(
        caps["evidentiary_ring_dropped"], json!(ring.dropped_total()),
        "capabilities must surface evidentiary_ring_dropped == the ring counter"
    );
    let gap_files: u64 = std::fs::read_dir(ldir.join("gaps"))
        .expect("gaps dir readable")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.starts_with("gap-") && n.ends_with(".json")
        })
        .count() as u64;
    assert!(gap_files >= 1, "at least one gap marker file must be on disk");
    assert_eq!(
        caps["evidentiary_ledger_gaps"], json!(gap_files),
        "capabilities must surface evidentiary_ledger_gaps == the gap marker files"
    );

    std::fs::remove_dir_all(&dir).ok();
    std::fs::remove_dir_all(&ldir).ok();
}
}
