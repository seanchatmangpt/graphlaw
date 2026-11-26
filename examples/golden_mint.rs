//! Mints the native side of graphlaw_ex's differential court: runs the SAME
//! ABI requests the Elixir side sends to the wasm, but through the native
//! `graphlaw::abi::call_json`, and writes the raw responses as goldens.
//! Usage: cargo run --example golden_mint -- <golden_dir>
use serde_json::json;

fn main() {
    let dir = std::env::args().nth(1).expect("golden dir");
    let data = std::fs::read_to_string(format!("{dir}/data.ttl")).expect("data.ttl");
    let query = std::fs::read_to_string(format!("{dir}/query.rq")).expect("query.rq");
    let canon = graphlaw::abi::call_json(&json!({"op": "canonical", "data": {"text": data}}));
    let sparql = graphlaw::abi::call_json(
        &json!({"op": "sparql", "data": {"text": data}, "query": query.trim()}),
    );
    std::fs::write(
        format!("{dir}/native_canonical.json"),
        serde_json::to_string_pretty(&canon).unwrap() + "\n",
    )
    .unwrap();
    std::fs::write(
        format!("{dir}/native_sparql.json"),
        serde_json::to_string_pretty(&sparql).unwrap() + "\n",
    )
    .unwrap();
    println!("canonical ok={} sparql ok={}", canon["ok"], sparql["ok"]);
}
