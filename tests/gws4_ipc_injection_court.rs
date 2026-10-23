#![cfg(feature = "abi")]
//! GWS-4 design court: in-WASM duckdb cannot read the host filesystem, so the
//! cold-chain case study's `register_parquet_view`/hive-partition path
//! (`src/duckdb_eval.rs::register_parquet_view`) cannot work as-written inside
//! the sandbox. The alternative under court: **data injection across the WASM
//! IPC boundary** — the corpus crosses as a linear-memory buffer
//! (`gl_alloc` → host `memory.write` → `gl_call(ptr, len)` → response), sized
//! against `MAX_REQUEST_BYTES` (16 MiB, `src/abi.rs`).
//!
//! The duckdb op now DISPATCHES IN-MODULE (src/duckdb_op.rs, op-15): an
//! injected `{"op":"duckdb",...}` request is answered with a duckdb-op-shaped
//! TYPED response (mode validation / SQLRDF-17 raw-SQL refusal / resource-limit
//! refusal), never `unknown op \`duckdb\``. The fail-closed property is
//! unchanged: the envelope under court gets a typed refusal envelope, no trap,
//! and the instance survives the next benign call.
//!
//! Proven here:
//! (a) a CSV corpus injected as a linear-memory buffer is loadable by the real
//!     frozen-duckdb eval path (`eval_ast_query`) — native, on the same bytes
//!     that cross the wasm boundary; the corpus→duckdb_eval injection leg runs
//!     through the HOST dispatch path (`graphlaw::abi::call_json`, op-15 exec
//!     with a `.csv` view mount). The same bytes cross the *compiled* wasm
//!     module's boundary and get a TYPED REFUSAL for the host-only op.
//!     Equivalence is whole-corpus `eval_ast_query` against the
//!     facility-partitioned chunks on the same bytes. A hand-rolled aggregator
//!     is not the oracle. The compiled module is hosted by wasmtime 48 with
//!     W3C exceptions (wasmi 2 rejects the exception tag section).
//! (b) the 16 MiB ABI bound vs corpus scale: bytes/row measured from the real
//!     corpus, chunking design = facility-partition chunks (mirrors the case
//!     study's `PARTITION_BY (facility_id)`), each chunk's wire envelope under
//!     the bound, aggregate-then-merge = whole-corpus aggregate (real duckdb).
//! (c) fail-closed: a single injection over `MAX_REQUEST_BYTES` is refused
//!     typed (`ResourceLimit` / `request_bytes`) natively AND through the
//!     compiled wasm module, and gl_alloc refuses over-cap allocations.
//!
//! Real collaborators only: the compiled wasm module (wasmtime 48, W3C
//! exceptions — wasmi 2 rejects the DuckDB-linked artifact's exception tag
//! section), real frozen-duckdb, a real generated 100-row corpus. No mocks.
//
// coordinator dev-dep: wasmtime 48 (runtime), wasmtime-wasi 48

use std::sync::{Mutex, OnceLock};

mod common;

use serde_json::{Value, json};

use graphlaw::abi::{MAX_REQUEST_BYTES, call_json};
use graphlaw::duckdb_eval::{GraphLawDuckDB, RdfTriple};
use graphlaw::purrdf::TermValue;
use graphlaw::sql_ast::compile_turtle;

// ---------------------------------------------------------------- corpus ---

/// Row shape mirrors the case study's `events` table
/// (facility_id, event_id, event_type, temp_celsius, duration_seconds).
struct ColdChainRow {
    facility: &'static str,
    event_id: String,
    temp: f64,
    duration: i64,
}

/// 100-row synthetic cold-chain corpus, 4 facility partitions (25 rows each).
/// Deterministic (seeded arithmetic, no RNG) so the court is replayable.
fn corpus() -> Vec<ColdChainRow> {
    let facilities = ["FAC-US-01", "FAC-EU-02", "FAC-AP-03", "FAC-LATAM-04"];
    let mut rows = Vec::with_capacity(100);
    for i in 0..100 {
        rows.push(ColdChainRow {
            facility: facilities[i % 4],
            event_id: format!("EVT-{:04}", 101 + i),
            // excursions above -20C on every 9th row (compliance signal)
            temp: -30.0 + ((i * 7) % 170) as f64 / 10.0,
            duration: 60 + ((i * 13) % 900) as i64,
        });
    }
    rows
}

fn csv_bytes(rows: &[ColdChainRow]) -> Vec<u8> {
    let mut s = String::from("facility_id,event_id,temp_celsius,duration_seconds\n");
    for r in rows {
        s.push_str(&format!(
            "{},{},{},{}\n",
            r.facility, r.event_id, r.temp, r.duration
        ));
    }
    s.into_bytes()
}

/// The wire envelope a host would send for one chunk: the corpus bytes ride
/// inside the ABI's single JSON request document. This envelope measures the
/// wire size of a corpus injection against `MAX_REQUEST_BYTES`; in-wasm the
/// duckdb op dispatches in-module and mode-validates it (typed `DuckDb`
/// refusal for an exec envelope without `query_ast`).
fn wire_envelope(csv: &[u8]) -> Vec<u8> {
    let body = String::from_utf8(csv.to_vec()).expect("corpus is utf8");
    json!({
        "op": "duckdb",
        "mode": "exec",
        "corpus": body,
        "dialect": "csv",
    })
    .to_string()
    .into_bytes()
}

fn source_rows(csv: &[u8]) -> usize {
    let text = std::str::from_utf8(csv).expect("utf8");
    text.lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .count()
}

// ------------------------------------------------------ real duckdb path ---

/// Load a CSV buffer into a fresh in-memory duckdb and run the aggregate
/// through the REAL eval path (`eval_ast_query`), returning the SQL-level
/// aggregate recovered from the receipt's projected RDF triples.
fn duckdb_whole(corpus_csv: &[u8]) -> (usize, Vec<(String, i64)>) {
    let db = GraphLawDuckDB::open_in_memory().expect("duckdb opens");
    load_csv(&db, corpus_csv);
    let compiled = agg_compiled();
    let receipt = db
        .eval_ast_query("urn:gws4:q_whole", &compiled, "urn:gws4:r", "urn:gws4:v")
        .expect("eval runs");
    (
        receipt.rows_emitted,
        triples_to_counts(&receipt.result_triples),
    )
}

/// Per-chunk real duckdb eval on a facility partition, then merge.
fn duckdb_chunked(chunks: &[Vec<u8>]) -> (usize, Vec<(String, i64)>) {
    let mut total_rows = 0;
    let mut merged: Vec<(String, i64)> = Vec::new();
    for chunk in chunks {
        let db = GraphLawDuckDB::open_in_memory().expect("duckdb opens");
        load_csv(&db, chunk);
        let compiled = agg_compiled();
        let receipt = db
            .eval_ast_query("urn:gws4:q_chunk", &compiled, "urn:gws4:r", "v")
            .map_err(|e| format!("{e}"))
            .unwrap_or_else(|e| panic!("chunk eval failed: {e}"));
        total_rows += receipt.rows_emitted;
        merged.extend(triples_to_counts(&receipt.result_triples));
    }
    // merge = fold per-chunk counts per facility
    let mut by_facility: Vec<(String, i64)> = Vec::new();
    for (fac, n) in merged {
        if let Some(e) = by_facility.iter_mut().find(|(f, _)| *f == fac) {
            e.1 += n;
        } else {
            by_facility.push((fac, n));
        }
    }
    by_facility.sort();
    (total_rows, by_facility)
}

fn load_csv(db: &GraphLawDuckDB, csv: &[u8]) {
    let text = std::str::from_utf8(csv).expect("corpus is utf8");
    let mut stmts = String::from(
        "CREATE TABLE cold_chain (facility_id VARCHAR, event_id VARCHAR, \
         temp_celsius DOUBLE, duration_seconds BIGINT);\n",
    );
    for (i, line) in text.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let c: Vec<&str> = line.split(',').collect();
        let esc = |s: &str| s.replace('\'', "''");
        stmts.push_str(&format!(
            "INSERT INTO cold_chain VALUES ('{}', '{}', {}, {});\n",
            esc(c[0]),
            esc(c[1]),
            c[2],
            c[3]
        ));
    }
    db.raw_connection()
        .execute_batch(&stmts)
        .expect("load runs");
}

/// SELECT facility_id, COUNT(event_id) AS n FROM cold_chain GROUP BY
/// facility_id — admitted as an RDF AST graph and compiled with
/// `compile_turtle`. SQLRDF-17: hand-built ASTs carry no canon anchor and are
/// a typed `NoCanonicalGraph` refusal at eval, so the court admits the query
/// the same way every consumer does.
fn agg_compiled() -> graphlaw::sql_ast::CompiledQuery {
    compile_turtle(AGG_TURTLE).expect("aggregate AST graph compiles")
}

/// The admission record: the aggregate query as a Turtle AST graph document.
/// Shared verbatim between the direct-eval legs and the host op-15 injection
/// leg, so the same AST drives both paths.
const AGG_TURTLE: &str = "@prefix sql: <http://graphlaw.org/sql#> .\n\
         @prefix fn: <http://graphlaw.org/functions#> .\n\
         @prefix ex: <urn:gws4:q#> .\n\
         ex:FacilityCount a sql:SelectStatement ;\n\
             sql:statementIndex 1 ;\n\
             sql:fromTable [ sql:tableName \"cold_chain\" ] ;\n\
             sql:selectList (\n\
                 [ sql:columnName \"facility_id\" ]\n\
                 [ sql:aliasName \"n\" ;\n\
                   sql:expression [ sql:function fn:COUNT ;\n\
                                    sql:argument [ sql:columnName \"event_id\" ] ] ]\n\
             ) ;\n\
             sql:groupBy ( [ sql:columnName \"facility_id\" ] ).";

/// Recover (facility, n) from the eval path's RDF projection. Each row projects
/// to subject `<prefix>/row_k`, with `.../v/facility_id` and `.../v/n` literals.
fn triples_to_counts(triples: &[RdfTriple]) -> Vec<(String, i64)> {
    let mut out: Vec<(String, i64)> = Vec::new();
    for t in triples {
        if !t.predicate.ends_with("/n") {
            continue;
        }
        let fac = triples
            .iter()
            .find(|f| f.subject == t.subject && f.predicate.ends_with("/facility_id"))
            .map(|f| match &f.object {
                TermValue::Literal { lexical_form, .. } => lexical_form.clone(),
                _ => panic!("facility_id literal"),
            })
            .expect("facility column present");
        let n = match &t.object {
            TermValue::Literal { lexical_form, .. } => {
                lexical_form.parse::<i64>().expect("count literal")
            }
            _ => panic!("count literal"),
        };
        out.push((fac, n));
    }
    out.sort();
    out
}

// ------------------------------------------------------- wasm IPC boundary ---

use wasmtime::{Config, Engine, Instance, Linker, Module, Store, TypedFunc};
use wasmtime_wasi::WasiCtxBuilder as P1WasiCtxBuilder;
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};

struct Host {
    store: Store<WasiP1Ctx>,
    memory: wasmtime::Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), i64>,
}

impl Host {
    /// None = the artifact cannot be hosted under the available runtimes and
    /// `GRAPHLAW_WASM` was not pinned: the courts SKIP LOUD with a typed
    /// reason (a silent pass is not allowed). When `GRAPHLAW_WASM` IS pinned,
    /// any failure is a hard FAIL — the pinned artifact is the unit.
    fn new() -> Result<Self, String> {
        // Single shared build path (tests/common/mod.rs): honors CARGO_TARGET_DIR,
        // cargo-flock serialized, typed BLOCKED-LINK panic on link failure.
        let bytes = std::fs::read(common::wasm_path()).map_err(|e| format!("artifact read: {e}"))?;
        // wasmi 2.0.0 rejects the DuckDB-linked artifact's W3C exception tag
        // section; wasmtime 48 with `wasm_exceptions(true)` hosts it.
        let mut cfg = Config::new();
        cfg.wasm_exceptions(true);
        let engine = Engine::new(&cfg).map_err(|e| format!("engine: {e}"))?;
        let module = Module::new(&engine, &bytes[..]).map_err(|e| format!("module: {e}"))?;
        for i in module.imports() {
            if i.module() != "wasi_snapshot_preview1" {
                return Err(format!("unexpected host import {}::{}", i.module(), i.name()));
            }
        }
        let mut store = Store::new(
            &engine,
            P1WasiCtxBuilder::new().inherit_stdio().inherit_env().build_p1(),
        );
        let mut linker = <Linker<WasiP1Ctx>>::new(&engine);
        add_to_linker_sync(&mut linker, |ctx| ctx).map_err(|e| format!("wasi: {e}"))?;
        let instance: Instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("instantiate: {e}"))?;
        if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
            init.call(&mut store, ()).map_err(|e| format!("_initialize: {e}"))?;
        }
        let missing =
            |name: &'static str, e: wasmtime::Error| format!("missing or mistyped export {name}: {e}");
        Ok(Host {
            memory: instance
                .get_memory(&mut store, "memory")
                .ok_or("module does not export memory")?,
            alloc: instance
                .get_typed_func::<u32, u32>(&mut store, "gl_alloc")
                .map_err(|e| missing("gl_alloc", e))?,
            free: instance
                .get_typed_func::<(u32, u32), ()>(&mut store, "gl_free")
                .map_err(|e| missing("gl_free", e))?,
            call: instance
                .get_typed_func::<(u32, u32), i64>(&mut store, "gl_call")
                .map_err(|e| missing("gl_call", e))?,
            store,
        })
    }

    /// The injection primitive under court: write `body` into linear memory,
    /// cross the boundary with gl_call, read + free the response.
    fn inject(&mut self, body: &[u8]) -> Value {
        let ptr = self.alloc.call(&mut self.store, body.len() as u32).unwrap();
        assert_ne!(ptr, 0, "gl_alloc refused a chunk-sized buffer");
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .unwrap();
        let packed = self
            .call
            .call(&mut self.store, (ptr, body.len() as u32))
            .unwrap() as u64;
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .unwrap();
        // gl_call CONSUMES (frees) the request buffer (wasm/src/lib.rs:13-14);
        // freeing it again here is a double free. Only the response buffer is
        // still host-owned post-call.
        self.free.call(&mut self.store, (out_ptr, out_len)).unwrap();
        serde_json::from_slice(&out).expect("response is JSON")
    }

    /// gl_call with an over-cap declared length on a live buffer.
    fn inject_declared_over(&mut self, live_len: u32, declared: u32) -> Value {
        let ptr = self.alloc.call(&mut self.store, live_len).unwrap();
        let packed = self.call.call(&mut self.store, (ptr, declared)).unwrap() as u64;
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .unwrap();
        // Same ownership contract: gl_call consumed the request buffer.
        self.free.call(&mut self.store, (out_ptr, out_len)).unwrap();
        serde_json::from_slice(&out).expect("response is JSON")
    }
}

/// Shared host, or `None` when the artifact cannot be hosted under the
/// available runtimes (typed loud-SKIP path — never a faked pass).
fn host() -> &'static Mutex<Option<Host>> {
    static H: OnceLock<Mutex<Option<Host>>> = OnceLock::new();
    H.get_or_init(|| {
        Mutex::new(match Host::new() {
            Ok(h) => Some(h),
            Err(e) => {
                if std::env::var_os("GRAPHLAW_WASM").is_some() {
                    panic!("GRAPHLAW_WASM is pinned; the artifact must host: {e}");
                }
                eprintln!(
                    "[SKIP] gws4 wasm IPC legs: typed-loud SKIP — artifact cannot be hosted \
                     under available runtimes: {e}"
                );
                None
            }
        })
    })
}

/// Loud-skip guard for the wasm legs: returns early (with a typed SKIP line
/// already emitted) when no host is available.
macro_rules! skip_if_no_host {
    ($guard:expr) => {
        if $guard.as_ref().is_none() {
            return;
        }
    };
}

// ------------------------------------------------------------------ court ---

#[test]
fn gws4_a_corpus_injected_as_linear_memory_buffer_loads_into_duckdb_eval() {
    let rows = corpus();
    let csv = csv_bytes(&rows);

    // (a) the same bytes, injected as one buffer, load and evaluate through the
    // REAL frozen-duckdb eval path with the AST compiler driving it.
    let (rows_out, counts) = duckdb_whole(&csv);
    assert_eq!(
        rows_out, 4,
        "GROUP BY emits one result row per facility: {counts:?}"
    );
    assert_eq!(counts.len(), 4, "four facility groups: {counts:?}");
    for (fac, n) in &counts {
        assert_eq!(*n, 25, "{fac}");
    }

    // (a') the identical bytes cross the compiled wasm module's linear-memory
    // boundary inside the ABI wire envelope and get a duckdb-op-shaped TYPED
    // response: the op dispatches in-module (src/duckdb_op.rs), so the
    // injected envelope is mode-validated — `mode: "exec"` without the
    // mandatory `query_ast` field is a typed `DuckDb` refusal, no crash, no
    // partial execution.
    let envelope = wire_envelope(&csv);
    assert!(
        envelope.len() < MAX_REQUEST_BYTES,
        "envelope under the bound"
    );
    let r = {
        let mut guard = host().lock().unwrap_or_else(|e| e.into_inner());
        skip_if_no_host!(guard);
        guard.as_mut().unwrap().inject(&envelope)
    };
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "DuckDb", "{r}");
    let message = r["error"]["message"].as_str().unwrap();
    assert!(message.contains("missing string field `query_ast`"), "{r}");

    // (a''-wasm) fail-closed survival: after the refused injection the SAME
    // instance answers a benign parse request — no trap, no poisoned state.
    let benign = json!({"op": "capabilities"});
    let r = {
        let mut guard = host().lock().unwrap_or_else(|e| e.into_inner());
        skip_if_no_host!(guard);
        guard.as_mut().unwrap().inject(benign.to_string().as_bytes())
    };
    assert_eq!(r["ok"], true, "{r}");

    // (a'') the corpus→duckdb_eval injection leg, on the HOST dispatch path:
    // the corpus is mounted as a `.csv` view through op-15 exec
    // (`graphlaw::abi::call_json`), the AST SQL is compiled by compile_turtle
    // and evaluated by the real eval path, and a cursor + receipt come back.
    let dir = std::env::temp_dir().join(format!("gws4_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let csv_path = dir.join("cold_chain.csv");
    std::fs::write(&csv_path, &csv).expect("write corpus csv");
    let exec_req = json!({
        "op": "duckdb",
        "mode": "exec",
        "views": [{"name": "cold_chain", "glob": csv_path.to_str().unwrap()}],
        "query_ast": "urn:gws4:q_host",
        "ast": {"turtle": AGG_TURTLE},
        "subject_prefix": "urn:gws4:r",
        "vocab_prefix": "urn:gws4:v",
    });
    let r = call_json(&exec_req);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["rows_emitted"], 4, "one group row per facility: {r}");
    let cursor = r["cursor"].as_str().unwrap_or_else(|| {
        panic!("ok envelope missing cursor field: {r}")
    });
    assert!(!cursor.is_empty(), "cursor empty: {r}");
    assert!(
        r["receipt"]
            .as_str()
            .unwrap()
            .starts_with("urn:gws4:r/receipt_"),
        "{r}"
    );
    // Stream the cursor back and confirm the RDF projection carries the
    // 25-rows-per-facility counts.
    let r = call_json(
        &json!({"op": "duckdb", "mode": "stream", "cursor": r["cursor"], "max_rows": 10}),
    );
    assert_eq!(r["ok"], true, "{r}");
    let nt = r["nquads"].as_str().unwrap();
    assert_eq!(nt.matches("<urn:gws4:v/n>").count(), 4, "{nt}");
    assert!(nt.contains("\"25\""), "{nt}");
    let _ = std::fs::remove_dir_all(&dir);

    // (b') a buffer the module DOES accept, at corpus scale, crosses and
    // processes fully inside wasm: the case-study RDF projection of the same
    // 100 events as an ntriples `parse` request (the transport under court).
    let nt = corpus_to_ntriples(&rows);
    let law_req = json!({"op": "parse", "text": nt, "dialect": "ntriples"});
    let r = {
        let mut guard = host().lock().unwrap_or_else(|e| e.into_inner());
        skip_if_no_host!(guard);
        guard
            .as_mut()
            .unwrap()
            .inject(law_req.to_string().as_bytes())
    };
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["quads"], 300, "100 events x 3 triples, evaluated in-wasm");
}

fn corpus_to_ntriples(rows: &[ColdChainRow]) -> String {
    let mut s = String::new();
    for r in rows {
        s.push_str(&format!(
            "<urn:cc:{eid}> <urn:cc:facility> <urn:cc:{fac}> .\n\
             <urn:cc:{eid}> <urn:cc:temp> \"{temp}\" .\n\
             <urn:cc:{eid}> <urn:cc:duration> \"{dur}\" .\n",
            eid = r.event_id,
            fac = r.facility,
            temp = r.temp,
            dur = r.duration
        ));
    }
    s
}

#[test]
fn gws4_b_chunk_streaming_design_meets_16mib_bound_and_merge_equals_whole() {
    let rows = corpus();
    let csv = csv_bytes(&rows);
    let bytes_per_row = (csv.len() as f64) / 100.0;

    // Scale projection: rows at which the single-injection envelope crosses
    // MAX_REQUEST_BYTES. Production corpus scale is typed as the follow-up;
    // here the design must show the bound is reachable and the chunking
    // design answers it.
    let max_rows_single = (MAX_REQUEST_BYTES as f64 / (bytes_per_row * 1.4)) as u64; // 1.4 = JSON envelope overhead factor
    assert!(
        max_rows_single < 10_000_000,
        "bound is reachable at production scale, not absurdly far: {max_rows_single}"
    );

    // Facility-partition chunks, mirroring the case study's PARTITION_BY.
    let mut chunks: Vec<Vec<u8>> = vec![Vec::new(); 4];
    let mut header = "";
    for (i, line) in std::str::from_utf8(&csv).unwrap().lines().enumerate() {
        if i == 0 {
            header = line;
            continue;
        }
        let fac_idx = (i - 1) % 4;
        chunks[fac_idx].extend_from_slice(line.as_bytes());
        chunks[fac_idx].push(b'\n');
    }

    // Each chunk's wire envelope stays far under the bound.
    for chunk in &chunks {
        let envelope = wire_envelope(chunk);
        assert!(
            envelope.len() < MAX_REQUEST_BYTES,
            "chunk envelope {} under {}",
            envelope.len(),
            MAX_REQUEST_BYTES
        );
    }

    // Per-chunk: the real eval's group count equals the source rows in the chunk.
    let mut merged: Vec<(String, i64)> = Vec::new();
    for chunk in &chunks {
        let mut full = header.as_bytes().to_vec();
        full.push(b'\n');
        full.extend_from_slice(chunk);
        let rows = source_rows(&full);
        let db = GraphLawDuckDB::open_in_memory().expect("duckdb opens");
        load_csv(&db, &full);
        let compiled = agg_compiled();
        let receipt = db
            .eval_ast_query("urn:gws4:q_chunk", &compiled, "urn:gws4:r", "urn:gws4:v")
            .expect("chunk eval");
        let per_chunk = triples_to_counts(&receipt.result_triples);
        assert_eq!(per_chunk.len(), 1, "one facility per chunk");
        assert_eq!(receipt.rows_emitted, 1, "one group row per facility chunk");
        assert_eq!(
            per_chunk[0].1 as u64, rows as u64,
            "group count equals source rows"
        );
        assert_eq!(rows, 25, "chunk carries 25 source rows");
        merged.extend(per_chunk);
    }
    merged.sort();
    assert_eq!(merged.len(), 4);
    for (fac, n) in &merged {
        assert_eq!(*n, 25, "{fac}");
    }

    // Aggregate-then-merge == whole-corpus real duckdb aggregate.
    let (whole_rows, whole_counts) = duckdb_whole(&csv);
    let (chunked_rows, chunked_counts) = duckdb_chunked(&chunks_with_header(header, &chunks));
    assert_eq!(whole_rows, chunked_rows);
    assert_eq!(whole_counts, chunked_counts, "merge equals whole");
}

fn chunks_with_header(header: &str, chunks: &[Vec<u8>]) -> Vec<Vec<u8>> {
    chunks
        .iter()
        .map(|c| {
            let mut full = header.as_bytes().to_vec();
            full.push(b'\n');
            full.extend_from_slice(c);
            full
        })
        .collect()
}

#[test]
fn gws4_c_fail_closed_when_corpus_exceeds_bounds() {
    // Native: a single request over MAX_REQUEST_BYTES is refused typed.
    let over = vec![b'x'; MAX_REQUEST_BYTES + 1];
    let r = call_json(&json!({"op": "capabilities", "pad": String::from_utf8(over).unwrap()}));
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["kind"], "ResourceLimit", "{r}");
    assert_eq!(r["error"]["details"]["code"], "ResourceLimit", "{r}");
    assert_eq!(r["error"]["details"]["limit"], "request_bytes", "{r}");
    assert_eq!(r["error"]["details"]["max"], MAX_REQUEST_BYTES, "{r}");

    // Wasm, real module: gl_alloc refuses an over-cap allocation outright.
    {
        let mut guard = host().lock().unwrap_or_else(|e| e.into_inner());
        skip_if_no_host!(guard);
        let h = guard.as_mut().unwrap();
        assert_eq!(
            h.alloc
                .call(&mut h.store, (MAX_REQUEST_BYTES + 1) as u32)
                .unwrap(),
            0
        );
        // A live buffer with an over-cap DECLARED length is refused typed,
        // without the host ever materializing the corpus.
        let r = {
            let live = h.alloc.call(&mut h.store, 8).unwrap();
            assert_ne!(live, 0);
            h.inject_declared_over(8, (MAX_REQUEST_BYTES + 1) as u32)
        };
        assert_eq!(r["ok"], false, "{r}");
        assert!(
            r["error"]["message"]
                .as_str()
                .unwrap()
                .contains("resource limit `request_bytes`"),
            "{r}"
        );
    }

    // A 16 MiB-sized corpus envelope is still inside the bound (positive
    // control): a just-under-cap buffer allocates and calls cleanly.
    {
        let mut guard = host().lock().unwrap_or_else(|e| e.into_inner());
        skip_if_no_host!(guard);
        let h = guard.as_mut().unwrap();
        let ok_len = MAX_REQUEST_BYTES - 4096;
        let ptr = h.alloc.call(&mut h.store, ok_len as u32).unwrap();
        assert_ne!(ptr, 0);
        h.free.call(&mut h.store, (ptr, ok_len as u32)).unwrap();
    }
}
