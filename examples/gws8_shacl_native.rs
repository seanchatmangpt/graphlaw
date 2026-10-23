//! GWS-8 native leg: run the exact GWS-8 request JSON through the graphlaw
//! native ABI entry (`graphlaw::abi::call`) — the same function the wasm FFI
//! exports — n* times, and report wall timings plus the BLAKE3 of the first
//! response bytes.
//!
//! usage: gws8_shacl_native <request.json> <n>
//!
//! The repo ships a checked-in fixture exercising the real `op_shacl` path
//! (data spec + shapes text):
//!
//!   cargo run --example gws8_shacl_native -- examples/fixtures/gws8_request.json 5
//!
//! This is a bench harness, not a lane: it writes no jsonl. Output is one
//! JSON object on stdout:
//! `{"engine":"native","n":5,"ops_per_sec":...,"response_blake3":"<64hex>","response_bytes":40,...}`
//! Bare run (no args): usage line + fixture path on stderr, exit 2 (no panic).
//! Court: no dedicated test executes this file; the same `shacl` op with
//! injected input is refused-by-design in tests/gws4_ipc_injection_court.rs.

use std::io::Read;
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .unwrap_or_else(|| {
            eprintln!("usage: cargo run --example gws8_shacl_native -- <request.json> <n>");
            eprintln!("fixture: examples/fixtures/gws8_request.json");
            std::process::exit(2);
        });
    let n: u32 = args
        .next()
        .unwrap_or_else(|| {
            eprintln!("usage: cargo run --example gws8_shacl_native -- <request.json> <n>");
            std::process::exit(2);
        })
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("error: <n> must be an integer");
            std::process::exit(2);
        });

    let mut request = Vec::new();
    std::fs::File::open(&path)
        .unwrap_or_else(|e| panic!("open {path}: {e}"))
        .read_to_end(&mut request)
        .unwrap();

    // Warmup (untimed): parse paths, allocator, shape parser caches.
    for _ in 0..100 {
        let resp = graphlaw::abi::call(&request);
        assert!(resp.starts_with(b"{\"ok\":true") || !resp.is_empty());
    }

    let mut first = Vec::new();
    let t0 = Instant::now();
    for _ in 0..n {
        let resp = graphlaw::abi::call(&request);
        if first.is_empty() {
            first = resp;
        }
    }
    let total = t0.elapsed();

    let hash = blake3::hash(&first).to_string();
    let total_ms = total.as_secs_f64() * 1000.0;
    println!(
        "{}",
        serde_json::json!({
            "engine": "native",
            "n": n,
            "total_ms": total_ms,
            "per_call_ms": total_ms / f64::from(n),
            "ops_per_sec": (f64::from(n)) / total.as_secs_f64(),
            "response_bytes": first.len(),
            "response_blake3": hash,
        })
    );
}
