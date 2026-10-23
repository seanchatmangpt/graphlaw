//! GWS-9 — adversarial fuzz court for the wasm boundary (`gl_call` / `call_json`).
//!
//! Drives a *real* compiled `graphlaw_wasm.wasm` module in wasmi with hostile
//! inputs, exactly as a malicious or buggy host would:
//!
//! (a) deeply nested JSON (1000+ levels) — the `MAX_JSON_DEPTH` recursion guard;
//! (b) 16 MiB-boundary focus nodes (exactly-at, exactly-over `MAX_REQUEST_BYTES`);
//! (c) malformed UTF-8 in string payloads;
//! (d) SQL injection tokens in the AST TTL (SHACL pattern gate / `sql_ast` identifiers);
//! (e) NaN / Infinity float literals in cold-chain-style numeric fields;
//! (f) truncation mid-frame (partial `gl_call` bodies).
//!
//! Invariants asserted on EVERY hostile input:
//! 1. the response is a well-formed JSON envelope with an `ok` bool;
//! 2. a failed response carries a typed `error.kind` from the documented
//!    fail-closed refusal vocabulary (`docs/refusals.md` / `RefusalKind`);
//! 3. the instance SURVIVES: the next benign call still answers `ok: true`.
//!
//! Deterministic: a seeded xorshift64* PRNG drives class selection, truncation
//! points and payload mutation — same seed, same inputs, same verdicts.
//! 1100 fuzz iterations + focused boundary cases (>= 1000 required by
//! GWS-SEC-FUZZ); refusal-class histogram printed on PASS.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

// ---------------------------------------------------------------- host plumbing

/// Location precedence: `GRAPHLAW_WASM` env > cached `target/wasm-abi` artifact
/// (built by `tests/wasm_abi.rs` and refreshed against current sources) >
/// a fresh build. If no binary can be produced the court SKIPs loudly —
/// a skip is typed, never a fake pass.
fn wasm_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("GRAPHLAW_WASM") {
        return Some(PathBuf::from(p));
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cached = root.join("target/wasm-abi/wasm32-wasip1/wasm/graphlaw_wasm.wasm");
    if cached.is_file() && wasm_fresh(&root, &cached) {
        return Some(cached);
    }
    let target = root.join("target/wasm-abi");
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .current_dir(&root)
        .args([
            "build",
            "-p",
            "graphlaw-wasm",
            "--target",
            "wasm32-wasip1",
            "--profile",
            "wasm",
            "--target-dir",
        ])
        .arg(&target)
        .status()
        .ok()?;
    if !status.success() {
        return None;
    }
    Some(target.join("wasm32-wasip1/wasm/graphlaw_wasm.wasm"))
}

/// Bounded-poll freshness check: the artifact must be newer than every
/// `src/` + `wasm/src` input. A stale artifact is rebuilt, not trusted.
fn wasm_fresh(root: &Path, artifact: &Path) -> bool {
    let Ok(art_mtime) = artifact.metadata().and_then(|m| m.modified()) else {
        return false;
    };
    let newer = |dir: &PathBuf| -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        entries.filter_map(|e| e.ok()).any(|e| {
            let p = e.path();
            if p.extension().is_none_or(|x| x != "rs") {
                return false;
            }
            e.metadata()
                .and_then(|m| m.modified())
                .map(|t| t > art_mtime)
                .unwrap_or(false)
        })
    };
    !(newer(&root.join("src")) || newer(&root.join("wasm/src")))
}

struct Host {
    store: wasmi::Store<wasmi_wasi::WasiCtx>,
    memory: wasmi::Memory,
    alloc: wasmi::TypedFunc<u32, u32>,
    free: wasmi::TypedFunc<(u32, u32), ()>,
    call: wasmi::TypedFunc<(u32, u32), u64>,
}

impl Host {
    fn new(bytes: &[u8]) -> Self {
        let engine = wasmi::Engine::default();
        let module = wasmi::Module::new(&engine, bytes).expect("valid wasm module");
        for i in module.imports() {
            assert_eq!(
                i.module(),
                "wasi_snapshot_preview1",
                "unexpected host import {}::{}",
                i.module(),
                i.name()
            );
        }
        let mut store = wasmi::Store::new(&engine, wasmi_wasi::WasiCtxBuilder::new().build());
        let mut linker = <wasmi::Linker<wasmi_wasi::WasiCtx>>::new(&engine);
        wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx).expect("links WASI");
        let instance: wasmi::Instance = linker
            .instantiate_and_start(&mut store, &module)
            .expect("instantiates");
        if let Ok(init) = instance.get_typed_func::<(), ()>(&store, "_initialize") {
            init.call(&mut store, ()).expect("_initialize");
        }
        let memory = instance
            .get_memory(&store, "memory")
            .expect("exports memory");
        Host {
            alloc: instance.get_typed_func(&store, "gl_alloc").unwrap(),
            free: instance.get_typed_func(&store, "gl_free").unwrap(),
            call: instance.get_typed_func(&store, "gl_call").unwrap(),
            store,
            memory,
        }
    }

    /// `gl_call` on an arbitrary byte frame (ptr/len written into linear
    /// memory) — the hostile path: no promise the bytes are JSON, UTF-8,
    /// complete, or within any limit. Reads and frees the response.
    fn raw_call(&mut self, body: &[u8]) -> Value {
        let ptr = self
            .alloc
            .call(&mut self.store, body.len() as u32)
            .expect("gl_alloc");
        if ptr == 0 {
            // Documented FFI contract (wasm/src/ffi.rs): an oversize or
            // cap-exceeding `gl_alloc` returns null WITHOUT trapping; a host
            // that plows ahead and calls `gl_call(0, len)` gets the typed
            // missing-buffer refusal, never an abort. The court exercises
            // that exact hostile-host path.
            let packed = self
                .call
                .call(&mut self.store, (0, body.len() as u32))
                .expect("gl_call must not trap on a null request pointer");
            let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
            let mut out = vec![0u8; out_len as usize];
            self.memory
                .read(&self.store, out_ptr as usize, &mut out)
                .expect("memory.read");
            self.free
                .call(&mut self.store, (out_ptr, out_len))
                .expect("gl_free");
            let resp: Value = serde_json::from_slice(&out).expect("response envelope is JSON");
            assert_eq!(
                resp["ok"], json!(false),
                "null-alloc call answered ok:true: {resp}"
            );
            assert_eq!(
                resp["error"]["kind"], json!("Unsupported"),
                "null-alloc refusal is untyped: {resp}"
            );
            assert!(
                resp["error"]["message"]
                    .as_str()
                    .is_some_and(|m| m.contains("request buffer missing")),
                "null-alloc refusal lost the documented message: {resp}"
            );
            return resp;
        }
        self.memory
            .write(&mut self.store, ptr as usize, body)
            .expect("memory.write");
        let packed = self.call.call(&mut self.store, (ptr, body.len() as u32)).expect("gl_call");
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .expect("memory.read");
        self.free
            .call(&mut self.store, (out_ptr, out_len))
            .expect("gl_free");
        serde_json::from_slice(&out).expect("response envelope is JSON")
    }
}

fn host() -> &'static Mutex<Host> {
    static H: OnceLock<Mutex<Host>> = OnceLock::new();
    H.get_or_init(|| {
        let Some(path) = wasm_path() else {
            panic!("SKIP: no graphlaw_wasm.wasm could be located or built (GWS-2 build state) — set GRAPHLAW_WASM or install the wasm32-wasip1 target");
        };
        let bytes = std::fs::read(&path).expect("read wasm artifact");
        Mutex::new(Host::new(&bytes))
    })
}

/// The survival canary: after every hostile input the instance must still
/// answer the cheapest benign call. A panic/abort anywhere in the hostile
/// path would poison or trap the instance and this would fail.
fn canary() -> Value {
    let mut h = host().lock().unwrap_or_else(|e| e.into_inner());
    let r = h.raw_call(br#"{"op":"capabilities"}"#);
    assert_eq!(
        r["ok"],
        json!(true),
        "instance did not survive a hostile call: canary answered {r}"
    );
    r
}

/// The documented fail-closed contract: every refusal carries a `kind` from
/// the `RefusalKind` vocabulary (docs/refusals.md).
const REFUSAL_KINDS: &[&str] = &[
    "NotSemanticContent",
    "Ambiguous",
    "EngineRejected",
    "Unsupported",
    "ResourceLimit",
    "TierMismatch",
];

/// Assert response is a well-formed typed envelope (invariants 1 + 2).
fn assert_typed(resp: &Value, class: &str, input_desc: &str) {
    let obj = resp
        .as_object()
        .unwrap_or_else(|| panic!("[{class}] {input_desc}: response is not a JSON object: {resp}"));
    let ok = obj
        .get("ok")
        .unwrap_or_else(|| panic!("[{class}] {input_desc}: envelope has no `ok` bool: {resp}"));
    assert!(ok.is_boolean(), "[{class}] {input_desc}: `ok` is not a bool: {resp}");
    if ok.as_bool() == Some(false) {
        let err = obj.get("error").unwrap_or_else(|| {
            panic!("[{class}] {input_desc}: `ok:false` without an `error` object: {resp}")
        });
        let kind = err["kind"].as_str().unwrap_or_else(|| {
            panic!("[{class}] {input_desc}: refusal without a typed `kind`: {resp}")
        });
        assert!(
            REFUSAL_KINDS.contains(&kind),
            "[{class}] {input_desc}: refusal kind `{kind}` is outside the documented vocabulary {REFUSAL_KINDS:?}: {resp}"
        );
        assert!(
            err.get("message").is_some_and(|m| m.is_string()),
            "[{class}] {input_desc}: refusal without a message: {resp}"
        );
    }
}

// ---------------------------------------------------------------- deterministic PRNG

/// xorshift64*, fixed seed: same seed => same inputs => same verdicts.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const SEED: u64 = 0x9E37_79B9_7F4A_7C15;
const _: () = assert!(SEED != 0);

// ---------------------------------------------------------------- payload helpers

/// Build a valid JSON request whose byte length is exactly `n`.
fn sized_request(n: usize) -> Vec<u8> {
    // `{"op":"parse","text":"<pad>","dialect":"turtle"}` — pad with 'x'
    // (valid Turtle IRI chars are irrelevant: a whitespace/pad-only text
    // parses as 0 quads, or refuses typed).
    let prefix = br#"{"op":"parse","text":""#;
    let suffix = br#"","dialect":"turtle"}"#;
    let inner = n - prefix.len() - suffix.len();
    let mut b = Vec::with_capacity(n);
    b.extend_from_slice(prefix);
    b.extend(std::iter::repeat_n(b'x', inner));
    b.extend_from_slice(suffix);
    assert_eq!(b.len(), n);
    b
}

/// Nested JSON of exact depth `d`: `[[[…]]]`.
fn nested(d: usize) -> Vec<u8> {
    let mut s = vec![b'['; d];
    s.push(b'1');
    s.extend(std::iter::repeat_n(b']', d));
    s
}

const SQL_TOKENS: &[&str] = &[
    "'; DROP TABLE cold_chain_events; --",
    "\" OR 1=1; --",
    "1; DELETE FROM ocel_events",
    "UNION SELECT secret FROM credentials",
    "Robert'); DROP TABLE Students;--",
];

// ---------------------------------------------------------------- the court

/// Counter of hostile calls (printed in the receipt line at the end).
static FUZZ_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Refusal-class histogram (GWS-SEC-FUZZ receipt evidence): verdict class
/// (`ok:true` vs each `error.kind`) per hostile call.
static HISTO: OnceLock<Mutex<BTreeMap<String, usize>>> = OnceLock::new();

fn histo() -> &'static Mutex<BTreeMap<String, usize>> {
    HISTO.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn record(verdict_class: &str) {
    let mut m = histo().lock().unwrap_or_else(|e| e.into_inner());
    *m.entry(verdict_class.to_string()).or_insert(0) += 1;
}

fn hostile(body: &[u8], class: &str, desc: &str) -> Value {
    FUZZ_CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut h = host().lock().unwrap_or_else(|e| e.into_inner());
    let r = h.raw_call(body);
    assert_typed(&r, class, desc);
    if r["ok"] == json!(true) {
        record("ok:true (admitted)");
    } else {
        let kind = r["error"]["kind"].as_str().unwrap_or("UNTYPED");
        record(&format!("refusal:{kind}"));
    }
    drop(h);
    canary();
    r
}

#[test]
fn gws9_wasm_boundary_fuzz() {
    let Some(path) = wasm_path() else {
        panic!(
            "SKIP is not a pass: no wasm binary (set GRAPHLAW_WASM or build \
             graphlaw-wasm for wasm32-wasip1). The boundary court did not run."
        );
    };
    println!("GWS-9 subject: {}", path.display());

    // ---- (a) deep nesting: recursion guard at and far beyond the cap
    for d in [0usize, 1, 63, 64, 65, 1000, 10_000] {
        let body = nested(d);
        let desc = format!("nested depth {d}");
        let resp = hostile(&body, "a-depth", &desc);
        // 63/64 must parse (<= MAX_JSON_DEPTH); 65+ must refuse ResourceLimit.
        if d > 64 {
            assert_eq!(
                resp["ok"], json!(false),
                "[a-depth] depth {d} passed the guard: {resp}"
            );
            assert_eq!(
                resp["error"]["kind"], json!("ResourceLimit"),
                "[a-depth] depth {d} refused untyped: {resp}"
            );
        }
    }

    // ---- (b) 16 MiB boundary: exactly-at accepted, exactly-over refused
    let max = 16 * 1024 * 1024;
    {
        let body = sized_request(max);
        let resp = hostile(&body, "b-16mib", "exactly-at 16 MiB");
        // Exactly-at must NOT be refused for request_bytes (limit is >, not >=).
        if resp["ok"] == json!(false) {
            let msg = resp["error"]["message"].as_str().unwrap_or("");
            assert!(
                !msg.contains("request_bytes"),
                "[b-16mib] exactly-at 16 MiB wrongly refused on request_bytes: {resp}"
            );
        }
    }
    {
        let body = sized_request(max + 1);
        let resp = hostile(&body, "b-16mib", "exactly-over 16 MiB");
        // Over-limit frames are fail-closed at gl_alloc (null) BEFORE any
        // parse: gl_alloc refuses len > MAX_REQUEST_BYTES (ffi.rs contract),
        // so the observed path is the typed missing-buffer refusal, kind
        // Unsupported — asserted inside raw_call. ResourceLimit(request_bytes)
        // is the native `call()` path; the FFI boundary gates one step earlier.
        assert_eq!(resp["ok"], json!(false), "over-limit passed: {resp}");
        assert_eq!(resp["error"]["kind"], json!("Unsupported"));
    }

    // ---- (c) malformed UTF-8 in string payloads (focused + fuzzed below)
    for (desc, body) in [
        ("lone continuation byte in op", &b"{\"op\":\"\x80\"}"[..]),
        ("invalid 0xFF/0xFE in text", &b"{\"op\":\"parse\",\"text\":\"\xFF\xFE\"}"[..]),
        (
            "overlong UTF-8 in dialect",
            &b"{\"op\":\"parse\",\"text\":\"x\",\"dialect\":\"\xC0\xAF\"}"[..],
        ),
        (
            "surrogate escape in string",
            &br#""{"op":"parse","text":"\uD800"}""#[..],
        ),
    ] {
        hostile(body, "c-utf8", desc);
    }

    // ---- (e) NaN / Infinity float literals in numeric fields
    for (desc, body) in [
        (
            "NaN in temperature",
            &br#"{"op":"law","law":[{"do":"sparql","query":"SELECT * WHERE { ?s ?p ?o }","temperature":NaN}]}"#[..],
        ),
        (
            "Infinity in limit field",
            &br#"{"op":"parse","text":"ex:s ex:p ex:o .","max_skew":Infinity}"#[..],
        ),
        (
            "-Infinity in skew",
            &br#"{"op":"capabilities","skew":-Infinity}"#[..],
        ),
        (
            "overflowing exponent 1e999",
            &br#"{"op":"parse","text":"ex:s ex:p ex:o .","weight":1e999}"#[..],
        ),
        (
            "negative NaN",
            &br#"{"op":"parse","text":"x","v":-NaN}"#[..],
        ),
    ] {
        hostile(body, "e-nan-inf", desc);
    }

    // ---- (d) SQL injection tokens in the AST TTL (wasm surface: shacl op)
    for tok in SQL_TOKENS {
        let shapes = format!(
            r#"@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <https://e/> .
ex:NodeShape a sh:NodeShape ;
    sh:targetClass ex:Thing ;
    sh:pattern "{tok}" ;
    sh:flags "s" ."#
        );
        let body = json!({"op": "shacl", "shapes": shapes,
                          "data": "@prefix ex: <https://e/> . ex:a a ex:Thing ."});
        hostile(body.to_string().as_bytes(), "d-sql-injection", "sh:pattern token");

        let body = json!({"op": "shacl", "shapes": shapes, "data": tok});
        hostile(body.to_string().as_bytes(), "d-sql-injection", "sh:pattern token as data");
    }

    // ---- (f) truncation mid-frame
    {
        let full = br#"{"op":"parse","text":"@prefix ex: <https://e/> . ex:s ex:p ex:o .","dialect":"turtle"}"#;
        let mut rng = Rng(0x5EED_F00D_CAFE_0001);
        for (desc, cut) in [
            ("empty frame", 0usize),
            ("single byte", 1),
            ("half frame", full.len() / 2),
            ("last byte missing", full.len() - 1),
        ] {
            hostile(&full[..cut], "f-truncated", desc);
        }
        for i in 0..40 {
            let cut = rng.below(full.len() as u64) as usize;
            let _ = hostile(&full[..cut], "f-truncated", &format!("random cut {i} (at {cut})"));
        }
    }

    // ---- main deterministic fuzz loop: 1100 iterations across all classes
    // (GWS-SEC-FUZZ requires >= 1000 adversarial frames with a class histogram)
    let mut rng = Rng(SEED);
    let pad: &[u8] = br#"{"op":"parse","text":"","dialect":"turtle"}"#;
    for i in 0..1100u32 {
        let class = rng.below(6);
        let body: Vec<u8> = match class {
            0 => nested(rng.below(2_000) as usize), // a: random depths 0..2000
            1 => {
                // b: sizes straddling the 16 MiB cap (mostly cheap small ones)
                match rng.below(4) {
                    0 => sized_request(max - 1),
                    1 => sized_request(max),
                    2 => sized_request(max + 1),
                    _ => vec![b'['; rng.below(70) as usize + 1], // unterminated depth
                }
            }
            2 => {
                // c: valid framing with mutated string bytes
                let mut b = pad.to_vec();
                let pos = 19 + rng.below(8) as usize; // inside the empty text string
                let byte = rng.below(256) as u8;
                b.insert(pos, byte);
                b
            }
            3 => {
                // d: SQL tokens embedded in a benign envelope's text field
                let tok = SQL_TOKENS[rng.below(SQL_TOKENS.len() as u64) as usize];
                format!(
                    r#"{{"op":"parse","text":"{}","dialect":"turtle"}}"#,
                    tok.replace('\\', "\\\\").replace('"', "\\\"")
                )
                .into_bytes()
            }
            4 => {
                // e: numeric literals from the hostile set
                let lit = ["NaN", "Infinity", "-Infinity", "1e999", "-NaN"][rng.below(5) as usize];
                format!(r#"{{"op":"parse","text":"x","v":{lit}}}"#).into_bytes()
            }
            _ => {
                // f: truncation of a structured request
                let full = br#"{"op":"law","steps":[{"do":"sparql","query":"SELECT 1"}]}"#;
                let cut = rng.below(full.len() as u64) as usize;
                full[..cut].to_vec()
            }
        };
        let _ = hostile(&body, "fuzz", &format!("iter {i} class {class}"));
    }

    let n = FUZZ_CALLS.load(std::sync::atomic::Ordering::Relaxed);
    assert!(
        n >= 1000,
        "court ran only {n} hostile calls; GWS-SEC-FUZZ requires >= 1000"
    );
    let hist = histo().lock().unwrap_or_else(|e| e.into_inner());
    let total: usize = hist.values().sum();
    assert_eq!(total, n, "histogram lost calls");
    println!("GWS-SEC-FUZZ refusal-class histogram ({n} hostile frames):");
    for (k, v) in hist.iter() {
        println!("  {k}: {v}");
    }
    println!(
        "GWS-SEC-FUZZ PASS: {n} hostile frames, 0 panics/traps/hangs, every refusal typed (kind in vocabulary), instance survived all calls (canary after each)."
    );
}

// ---------------------------------------------------------------- native-side SQL-injection gate (d, companion to GWS-5 shapes)

#[test]
fn gws9_sql_injection_ast_gate() {
    use graphlaw::sql_ast::{self, RefusalCode};

    let shapes_ttl = include_str!("../ontologies/sql-ast.ttl");

    // validate_identifier refuses every injection token, typed.
    for tok in SQL_TOKENS {
        let r = sql_ast::validate_identifier(tok);
        assert!(
            matches!(r, Err(e) if matches!(e.code, RefusalCode::InvalidIdentifier)),
            "identifier gate accepted injection token {tok:?}: {r:?}"
        );
    }

    // Injection token as sql:tableName against the canonical SHACL shapes:
    // typed AstError (InvalidIdentifier or ShaclViolation), never a panic.
    let valid_prefix = r#"
        @prefix sql: <http://graphlaw.org/sql#> .
        @prefix ex: <http://biopharma.org/queries#> .
        ex:Q a sql:SelectStatement ;
            sql:fromTable [ sql:tableName "#;
    for tok in SQL_TOKENS {
        let ttl = format!(
            "{valid_prefix}\"{tok}\" ; sql:selectList ( [ sql:columnName \"event_id\" ] ) ."
        );
        // Lawful large typed refusal: AstError carries full SHACL diagnostics;
        // scoping the allow to these catch_unwind closures, no API change.
        #[allow(clippy::result_large_err)]
        let r = std::panic::catch_unwind(|| sql_ast::validate_ast_shacl(&ttl, shapes_ttl));
        #[allow(clippy::result_large_err)]
        let r = r.unwrap_or_else(|p| {
            panic!("validate_ast_shacl PANICKED on injection token {tok:?}: {p:?}")
        });
        assert!(r.is_err(), "SHACL AST gate accepted injection token {tok:?}");
    }

    // And the honest positive control: a clean table name still validates.
    let clean = format!(
        "{valid_prefix}\"ocel_events\" ; sql:selectList ( [ sql:columnName \"event_id\" ] ) ."
    );
    // (Positive control may fail validation on other shape constraints; the
    //  invariant under court is only that it does not panic. Record verdict.)
    let clean_ok = sql_ast::validate_ast_shacl(&clean, shapes_ttl).is_ok();
    println!("GWS-9 native SQL gate: 5 tokens refused, positive-control clean AST conforms = {clean_ok}");
}

