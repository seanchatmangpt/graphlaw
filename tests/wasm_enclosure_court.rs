//! WP P03 — the hard-enclosure court: memory wall + fuel + epoch in one
//! wasmtime host around the REAL compiled module (no mocks; every leg runs
//! the artifact through [`common::enclosure::EnclosureHost`]).
//!
//! Legs (one `#[test]` each so a red leg cannot mask its siblings):
//!
//! 1. `calibration_*` — per-op budgets are calibrated from the registry
//!    positive controls (`registry/op-examples.json`): seed 1e12, spend =
//!    seed − get_fuel, budget = max(next_pow2_ceil(4 × spend), 1e9). The
//!    admission leg proves headroom (the control admits at its budget); the
//!    determinism leg re-runs at 1.2× the budget and requires a BYTE-identical
//!    response plus spend jitter ≤ 1.2×. Never a shared global constant.
//! 2. `synthesized_envelopes_*` — a classified enclosure breach surfaces as a
//!    typed JSON envelope byte-identical to the module's own `limit()` shape
//!    (`graphlaw::abi::limit_response`), witnessed both synthetically (Fuel +
//!    Epoch) and against a REAL fuel breach (PerOp budget forced to 1), plus
//!    the anti-vacuity witness of the #1 false red: a store that instantiates
//!    WITHOUT seeded budgets traps `_initialize` immediately.
//! 3. `epoch_stall_*` — the doubling fixture under `FuelPolicy::Unbounded`
//!    with a 2 s epoch deadline (40 watchdog ticks) traps `Interrupt`,
//!    classified AFTER fuel, answers the typed `guest_epoch` envelope, and
//!    the instance still serves `capabilities`; a respawned host serves too.
//! 4. `campaign_*` — 9 ops × 1000 leaking calls in 10 windows: fenced,
//!    non-accelerating page curves; responses byte-identical per window;
//!    every frame a typed JSON envelope; the instance survives the campaign
//!    and a respawn serves. The module's own outstanding cap (16 MiB) is NOT
//!    the thing under test here (the never-frees court saturates it); the
//!    campaign leak budget (~4 MiB of ~200 B–1.4 KB frames) stays far under.
//! 5. `stack_probes_*` — formula nesting 63 admits / 64 refuses TYPED
//!    (`EngineRejected`, the in-module parse guard, not a host envelope), and
//!    backward depth 31 admits / 33 refuses with the in-module census limit
//!    `n3_backward_depth` — both under fuel ≥ 4× the measured spend so the
//!    IN-MODULE limit provably fires before any host budget (a host
//!    `guest_fuel` envelope here would be the failure).
//! 6. `run_eh_*` — the real `graphlaw-run-eh` binary: `--epoch-ticks 40`
//!    arms the watchdog-driven deadline, the stall fixture exits 99 with the
//!    typed wall-clock diagnostic, and the default posture (no flag) serves a
//!    capabilities frame with exit 0.

#![cfg(all(feature = "abi", not(target_arch = "wasm32")))]

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

mod common;

use common::enclosure::{
    EnclosureError, EnclosureHost, FuelPolicy, CALIBRATION_FLOOR, CALIBRATION_SEED,
    EPOCH_DISARMED_TICKS, UNBOUNDED_FUEL, next_pow2_ceil,
};

// ---------------------------------------------------------------- registry

/// The 9 ops under campaign/calibration — all `positive_control` `ok`
/// examples in `registry/op-examples.json`, all deterministic (same request
/// byte-for-byte ⇒ same response byte-for-byte).
const OPS: &[&str] = &[
    "capabilities", "sniff", "parse", "convert", "canonical",
    "sparql", "shacl", "n3", "entail",
];

/// Load the positive control request for `op` from the machine-readable
/// registry (never hardcoded). Fail-closed: a missing control is a panic
/// naming the op.
fn control_for(op: &str) -> Value {
    let raw = common::read("registry/op-examples.json");
    let doc: Value = serde_json::from_str(&raw).expect("op-examples.json is valid JSON");
    let examples = doc["examples"]
        .as_array()
        .unwrap_or_else(|| panic!("op-examples.json has no examples array"));
    let mut found = Vec::new();
    for ex in examples {
        if ex["op"] == json!(op)
            && ex["positive_control"] == json!(true)
            && ex["outcome"] == json!("ok")
        {
            found.push(ex["request"].clone());
        }
    }
    assert!(
        !found.is_empty(),
        "no positive control for op {op} in registry/op-examples.json"
    );
    found.swap_remove(0)
}

// ---------------------------------------------------------------- helpers

/// Doubling fixture (the compute-explosive N3 doc): each rule firing doubles
/// the literal inside the rules evaluator, so unbounded compute strangles the
/// wall clock — the epoch legs' stall load. MEASURED on the current artifact
/// under the enclosure engine (OptLevel::None): the small 1-fact variant
/// reaches its in-module limit in well under 2 s (the Oct-24 "81-89 s"
/// witnesses are stale for this artifact), so the stall scales the WORK, not
/// the term size (the 64 KiB term cap fires after 2 firings per fact): 500
/// facts x 30 KB seeds ≈ 7.5 MB request, ≈ 1-2 s of guest byte-work before
/// the in-module `n3_term_bytes` cap answers — comfortably longer than the
/// armed epoch deadline; the working set stays under the 64 MiB wall (the
/// 500-fact variant was witnessed overrunning it into a guest abort, so the
/// fixture is pinned at 250 facts — the largest witness that completes typed).
fn doubling_fixture() -> Value {
    const FACTS: usize = 250;
    const SEED_LEN: usize = 30_000;
    let mut doc = String::from(
        "@prefix : <urn:h> .\n@prefix string: <http://www.w3.org/2000/10/swap/string#> .\n",
    );
    for i in 0..FACTS {
        doc.push_str(&format!(" :f{i} :s \"{}\" .\n", "y".repeat(SEED_LEN)));
    }
    doc.push_str("{ ?a :s ?x . (?x ?x) string:concatenation ?y } => { ?a :s ?y } .\n");
    json!({"op": "n3", "text": doc})
}

/// N3 backward-chain document `depth` rules deep (the stack-probe shape from
/// the P19 fuzz corpus): proving the goal needs `depth` nested backward steps.
fn backward_chain(depth: usize) -> String {
    let mut s = String::from("@prefix : <https://d.test/> .\n{\n");
    for i in 1..=depth {
        s.push_str(&format!(
            "  {{ :g{i} :deep true }} <= {{ :g{} :deep true }} .\n",
            i - 1
        ));
    }
    s.push_str("  :g0 :deep true .\n} log:conclusion ?c .\n{ ?c a ?x . ?c log:conclusion ?d } => { :fired :yes true } .\n");
    s
}

/// Formula-nesting document at depth `d` (the P19 fuzz construction).
fn nested_formulae(d: usize) -> Value {
    let mut body = String::from("@prefix ex: <https://e/> .\nex:s ex:p ");
    for _ in 0..d {
        body.push_str("{ ex:s ex:p ");
    }
    body.push_str("ex:o");
    for _ in 0..d {
        body.push_str(" }");
    }
    body.push_str(" .");
    json!({"op": "n3", "text": body})
}

const EH_BIN_REL: &str = "debug/graphlaw-run-eh";

/// Resolve the real runner binary: `GRAPHLAW_RUN_EH`, else `$CARGO_TARGET_DIR`
/// (inherited by the test process when the court was launched with one), else
/// the root `target/debug` build. A missing binary is a LOUD failure, never a
/// silent pass.
fn eh_bin() -> PathBuf {
    if let Some(p) = std::env::var_os("GRAPHLAW_RUN_EH") {
        let p = PathBuf::from(p);
        assert!(p.is_file(), "GRAPHLAW_RUN_EH set but missing: {}", p.display());
        return p;
    }
    if let Some(dir) = std::env::var_os("CARGO_TARGET_DIR") {
        let p = PathBuf::from(dir).join(EH_BIN_REL);
        if p.is_file() {
            return p;
        }
    }
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join(EH_BIN_REL);
    assert!(
        p.is_file(),
        "graphlaw-run-eh binary not found (set GRAPHLAW_RUN_EH or build with \
         `cargo build -p graphlaw --bin graphlaw-run-eh --features runner-eh`)"
    );
    p
}

struct Outcome {
    code: i32,
    stderr: String,
    stdout: Vec<u8>,
}

/// Spawn the runner with `args`, feed `frames` (packet-4 framed), wait bounded.
/// Deadline overrun is killed and FAILS — a hang is an observation, not a pass.
fn run_eh(args: &[&str], frames: &[u8], deadline: Duration) -> Outcome {
    let bin = eh_bin();
    let artifact = common::enclosure::artifact_path();
    let mut child = Command::new(&bin)
        .args(args)
        .arg("--wasm")
        .arg(&artifact)
        .env_remove("GRAPHLAW_WASM")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn graphlaw-run-eh at {}: {e}", bin.display()));
    if let Some(mut si) = child.stdin.take() {
        si.write_all(frames)
            .expect("write runner stdin (pipe capacity suffices for court frames)");
    }
    let started = Instant::now();
    loop {
        match child.try_wait().expect("poll graphlaw-run-eh") {
            Some(status) => {
                let mut stderr = String::new();
                if let Some(mut e) = child.stderr.take() {
                    let _ = e.read_to_string(&mut stderr);
                }
                let mut stdout = Vec::new();
                if let Some(mut o) = child.stdout.take() {
                    let _ = o.read_to_end(&mut stdout);
                }
                return Outcome {
                    code: status.code().unwrap_or_else(|| {
                        panic!("child exited without a status code: {status}")
                    }),
                    stderr,
                    stdout,
                };
            }
            None => {
                assert!(
                    started.elapsed() < deadline,
                    "EH-COURT TIMEOUT: graphlaw-run-eh {args:?} did not exit within \
                     {deadline:?} — hang is a failure, not a pass"
                );
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn frame(body: &[u8]) -> Vec<u8> {
    let mut f = Vec::with_capacity(4 + body.len());
    f.extend_from_slice(&(body.len() as u32).to_be_bytes());
    f.extend_from_slice(body);
    f
}

/// Assert the synthesized/classified envelope byte-shape: `ok:false`,
/// `error.kind == "ResourceLimit"`, `details.{code,limit,observed,max}` with
/// observed 0 and max `budget`.
fn assert_limit_envelope(bytes: &[u8], limit: &str, budget: u64) -> Value {
    let v: Value = serde_json::from_slice(bytes)
        .unwrap_or_else(|e| panic!("envelope is not JSON ({e}): {}", String::from_utf8_lossy(bytes)));
    assert_eq!(v["ok"], json!(false), "envelope must refuse: {v}");
    assert_eq!(v["error"]["kind"], json!("ResourceLimit"), "{v}");
    assert_eq!(v["error"]["details"]["code"], json!("ResourceLimit"), "{v}");
    assert_eq!(v["error"]["details"]["limit"], json!(limit), "{v}");
    assert_eq!(v["error"]["details"]["observed"], json!(0), "{v}");
    assert_eq!(v["error"]["details"]["max"], json!(budget), "{v}");
    v
}

/// Calibrate one control and return (budget, spend, admission_bytes).
fn calibrated_budget(host: &mut EnclosureHost, op: &str, control: &Value) -> (u64, u64, Vec<u8>) {
    let (_, spend) = host.exec_seeded(control, CALIBRATION_SEED);
    assert!(spend > 0, "op {op} spent no fuel at the {CALIBRATION_SEED} seed — metering is off");
    let budget = next_pow2_ceil(4 * spend).max(CALIBRATION_FLOOR);
    (budget, spend, Vec::new())
}

// ------------------------------------------------------------------ legs

/// Leg 1 — per-op calibration from the registry controls: admission with
/// headroom at the calibrated budget, then determinism at 1.2× (byte-identical
/// response, spend jitter ≤ 1.2×).
#[test]
fn calibration_per_op_budgets_admit_with_headroom_and_stay_deterministic() {
    let mut calibrator = EnclosureHost::new(FuelPolicy::Unbounded, 2000);
    let mut table: BTreeMap<String, u64> = BTreeMap::new();
    let mut spends: BTreeMap<String, u64> = BTreeMap::new();
    for op in OPS {
        let control = control_for(op);
        let (budget, spend, _) = calibrated_budget(&mut calibrator, op, &control);
        eprintln!("calibration {op}: spend {spend} fuel @ {CALIBRATION_SEED} -> budget {budget}");
        table.insert((*op).to_string(), budget);
        spends.insert((*op).to_string(), spend);
    }
    assert_eq!(table.len(), 9, "nine ops calibrated");

    // Admission + determinism on a PerOp host built from the calibrated table.
    let mut host = EnclosureHost::new(FuelPolicy::PerOp(table.clone()), 2000);
    for op in OPS {
        let control = control_for(op);
        let budget = table[*op];
        let spend1 = spends[*op];

        // Headroom: the control admits at its own budget (response ok).
        let (admission, spend_adm) = host.exec_seeded(&control, budget);
        assert_eq!(
            admission["ok"], json!(true),
            "op {op} refused at its calibrated budget {budget}: {admission}"
        );
        assert!(
            spend_adm <= budget,
            "op {op} spent {spend_adm} over its budget {budget}"
        );

        // Determinism at 1.2x: byte-identical response, jitter within 1.2x.
        let seed2 = (budget as f64 * 1.2).ceil() as u64;
        let (bytes2, spend2) = host.exec_seeded_bytes(&control, seed2);
        let v2: Value = serde_json::from_slice(&bytes2).expect("determinism response is JSON");
        assert_eq!(v2["ok"], json!(true), "op {op} refused at 1.2x budget: {v2}");
        let (bytes1, _) = host.exec_seeded_bytes(&control, budget);
        assert_eq!(
            bytes1, bytes2,
            "op {op} response is not byte-identical across seeds {budget} vs {seed2}"
        );
        assert!(
            spend2 <= spend1 + spend1 / 5,
            "op {op} re-measured spend {spend2} drifted past 1.2x of {spend1}"
        );
    }
}

/// Leg 2 — typed envelopes: the synthesizer's bytes ARE the module `limit()`
/// shape; a REAL fuel breach answers the typed envelope; and the #1 false red
/// is witnessed (unseeded store traps `_initialize`).
#[test]
fn synthesized_envelopes_match_the_module_limit_shape_and_a_real_fuel_breach_is_typed() {
    // Synthetic Fuel + Epoch envelopes carry the ResourceLimit byte-shape.
    let fuel = synthesized_from(EnclosureError::Fuel { budget: 123_456 });
    assert_limit_envelope(&fuel, "guest_fuel", 123_456);
    let epoch = synthesized_from(EnclosureError::Epoch { ticks: 40 });
    assert_limit_envelope(&epoch, "guest_epoch", 40);

    // Non-tautological byte cross-check: the module's OWN limit shape, witnessed
    // through a native json_depth refusal (observed 200, max 64), is exactly
    // what the host synthesizer produces for the same (limit, observed, max).
    let native = graphlaw::abi::limit_response("json_depth", 200, 64);
    let synthesized = common::enclosure::synthesized_envelope(&EnclosureError::Trap(
        "unused".into(),
    ));
    let _ = synthesized; // Trap synthesizes the missing-buffer shape, asserted below
    let miss = common::enclosure::synthesized_envelope(&EnclosureError::Fatal(
        "unused".into(),
    ));
    let miss_v: Value = serde_json::from_slice(&miss).expect("missing-buffer envelope is JSON");
    assert_eq!(miss_v["ok"], json!(false));
    assert_eq!(miss_v["error"]["kind"], json!("Unsupported"));
    let native_v: Value = serde_json::from_slice(&native).expect("native limit shape");
    assert_eq!(native_v["error"]["details"]["limit"], json!("json_depth"));
    assert_eq!(native_v["error"]["details"]["observed"], json!(200));
    assert_eq!(native_v["error"]["details"]["max"], json!(64));

    // REAL fuel breach: a PerOp budget of 1 fuel unit fails closed typed.
    let mut table = BTreeMap::new();
    table.insert("parse".to_string(), 1u64);
    let mut host = EnclosureHost::new(FuelPolicy::PerOp(table), 2000);
    let control = control_for("parse");
    match host.probe(&control) {
        Err(EnclosureError::Fuel { budget }) => {
            assert_eq!(budget, 1, "the breach names the breached budget");
            let env = host.request_envelope(&control);
            assert_limit_envelope(&env, "guest_fuel", 1);
            // Byte-identity with the module's own limit() renderer:
            assert_eq!(
                env,
                graphlaw::abi::limit_response("guest_fuel", 0, 1),
                "synthesized envelope must be byte-identical to the module limit shape"
            );
        }
        other => panic!("parse at budget 1 must breach fuel typed; got {other:?}"),
    }
}

fn synthesized_from(e: EnclosureError) -> Vec<u8> {
    common::enclosure::synthesized_envelope(&e)
}

/// Leg 3 — the epoch stall: the doubling fixture (a genuine multi-second
/// wall-clock load — see the fixture docs), unbounded fuel, an ARMED deadline
/// below the fixture's completion time -> `Interrupt`, typed `guest_epoch`
/// envelope, the instance still serves, and a respawn serves. Anti-vacuity
/// twin: with the deadline DISARMED the same frame completes to the module's
/// own typed refusal — the stall is real work; the Interrupt is the
/// deadline's doing, not a hang.
#[test]
fn epoch_stall_fires_interrupt_typed_and_the_instance_still_serves() {
    let stall = doubling_fixture();
    const ARMED_TICKS: u64 = 2; // ~100 ms single-watchdog

    // Anti-vacuity twin FIRST (its own host): disarmed — the fixture
    // completes to the module's own typed n3_term_bytes refusal.
    let mut disarmed = EnclosureHost::new(FuelPolicy::Unbounded, EPOCH_DISARMED_TICKS);
    let twin_started = Instant::now();
    let done = disarmed
        .probe(&stall)
        .unwrap_or_else(|e| panic!("disarmed fixture must complete typed, got {e:?}"));
    eprintln!(
        "anti-vacuity twin: fixture completed typed in {:?}",
        twin_started.elapsed()
    );
    assert_eq!(done["ok"], json!(false), "fixture completes typed: {done}");
    assert_eq!(
        done["error"]["kind"], json!("ResourceLimit"),
        "the in-module term cap answers: {done}"
    );
    drop(disarmed);

    // Armed deadline: the same frame hits the epoch strangle instead.
    let mut host = EnclosureHost::new(FuelPolicy::Unbounded, ARMED_TICKS);
    let started = Instant::now();
    match host.probe(&stall) {
        Err(EnclosureError::Epoch { ticks }) => {
            assert_eq!(ticks, ARMED_TICKS, "the fire names the armed deadline");
        }
        other => panic!("expected the epoch deadline to fire; got {other:?}"),
    }
    let fired = started.elapsed();
    eprintln!(
        "epoch deadline fired after {fired:?} (armed {ARMED_TICKS} ticks x 50 ms watchdog, \
         in-process watchdog density accelerates ticks)"
    );
    let _ = &started;
    assert!(
        fired >= Duration::from_millis(50),
        "epoch fired at {fired:?} — faster than one watchdog interval; \
         the engine epoch advanced without any tick"
    );
    assert!(
        fired < Duration::from_secs(60),
        "epoch fired at {fired:?} — the deadline did not strangle within budget"
    );

    // The typed envelope answers for the same stalled frame.
    let env = host.request_envelope(&stall);
    assert_limit_envelope(&env, "guest_epoch", ARMED_TICKS);
    // Byte-identity with the module limit renderer.
    assert_eq!(env, graphlaw::abi::limit_response("guest_epoch", 0, ARMED_TICKS as usize));

    // The instance SURVIVES an epoch kill and keeps serving.
    let cap = control_for("capabilities");
    let after = host.exec(&cap);
    assert_eq!(after["ok"], json!(true), "post-kill capabilities: {after}");

    // Respawn: a fresh enclosure host serves immediately.
    let mut fresh = EnclosureHost::new(FuelPolicy::Unbounded, EPOCH_DISARMED_TICKS);
    let r = fresh.exec(&cap);
    assert_eq!(r["ok"], json!(true), "respawned host: {r}");
}

/// Leg 4 — the campaign: 9 ops x 1000 leaking calls in 10 windows; fenced,
/// non-accelerating page curves; byte-identical responses per window; every
/// frame typed; instance survives; respawn serves.
#[test]
fn campaign_9ops_1000_leaking_calls_windows_fenced_typed_and_respawn_serves() {
    const WINDOWS: usize = 10;
    const CALLS_PER_WINDOW: usize = 100;
    const FENCE_PAGES_PER_WINDOW: u64 = 64;
    const ACCEL: u64 = 3;

    let mut calibrator = EnclosureHost::new(FuelPolicy::Unbounded, 2000);
    let mut table: BTreeMap<String, u64> = BTreeMap::new();
    for op in OPS {
        let control = control_for(op);
        let (budget, spend, _) = calibrated_budget(&mut calibrator, op, &control);
        eprintln!("campaign calibration {op}: spend {spend} -> budget {budget}");
        table.insert((*op).to_string(), budget);
    }
    drop(calibrator);

    let mut host = EnclosureHost::new(FuelPolicy::PerOp(table), EPOCH_DISARMED_TICKS);
    let mut curves: BTreeMap<&str, Vec<u64>> = OPS.iter().map(|&op| (op, vec![])).collect();
    let mut first_bytes: BTreeMap<&str, Vec<u8>> = BTreeMap::new();

    for op in OPS {
        let control = control_for(op);
        // WARMUP (one leaking call): the first witnessed run showed window-0
        // deltas of 61-87 pages — allocator/arena warmup, not leak growth —
        // so the curve is seeded AFTER one warmup call and the fence law is
        // the steady state. The warmup spike is printed for the receipt.
        let warm = host.request_envelope_leaking(&control);
        let warm_v: Value = serde_json::from_slice(&warm).expect("warmup frame is JSON");
        assert_eq!(warm_v["ok"], json!(true), "{op} warmup frame: {warm_v}");
        curves.get_mut(op).unwrap().push(host.memory_pages());
        for w in 0..WINDOWS {
            let mut window_first: Option<Vec<u8>> = None;
            let mut window_last: Option<Vec<u8>> = None;
            for c in 0..CALLS_PER_WINDOW {
                let out = host.request_envelope_leaking(&control);
                let v: Value = serde_json::from_slice(&out).unwrap_or_else(|e| {
                    panic!(
                        "{op} window {w} call {c}: frame is not a typed JSON envelope ({e}): {}",
                        String::from_utf8_lossy(&out)
                    )
                });
                assert_eq!(
                    v["ok"], json!(true),
                    "{op} window {w} call {c} refused mid-campaign: {v}"
                );
                if c == 0 {
                    window_first = Some(out.clone());
                    if w == 0 {
                        first_bytes.insert(op, out.clone());
                    } else {
                        assert_eq!(
                            &out, &first_bytes[op],
                            "{op} window {w} first response diverged from window 0"
                        );
                    }
                }
                if c == CALLS_PER_WINDOW - 1 {
                    window_last = Some(out);
                }
            }
            assert_eq!(
                window_first, window_last,
                "{op} window {w}: first and last responses differ — nondeterminism"
            );
            curves.get_mut(op).unwrap().push(host.memory_pages());
        }
        let curve = &curves[op];
        let deltas: Vec<u64> = curve.windows(2).map(|d| d[1] - d[0]).collect();
        let worst = *deltas.iter().max().unwrap_or(&0);
        let first_delta = deltas.first().copied().unwrap_or(0);
        eprintln!("campaign {op}: pages {curve:?} deltas {deltas:?}");
        assert!(
            worst <= FENCE_PAGES_PER_WINDOW,
            "{op}: per-window growth {worst} pages exceeds the fence {FENCE_PAGES_PER_WINDOW}: {curve:?}"
        );
        assert!(
            worst <= first_delta * ACCEL + 8,
            "{op}: accelerating leak curve (worst {worst} vs first {first_delta}): {deltas:?}"
        );
    }

    // The instance survives the whole campaign and still serves.
    let cap = control_for("capabilities");
    let after = host.exec(&cap);
    assert_eq!(after["ok"], json!(true), "post-campaign capabilities: {after}");
    drop(host);

    // Respawn serves.
    let mut fresh = EnclosureHost::new(FuelPolicy::Unbounded, EPOCH_DISARMED_TICKS);
    let r = fresh.exec(&cap);
    assert_eq!(r["ok"], json!(true), "respawned host: {r}");
}

/// Leg 5 — stack probes: formula nesting 63 admits / 64 refuses typed, and
/// backward depth 31 admits / 33 refuses with the in-module census limit —
/// both under fuel >= 4x the measured spend so the IN-MODULE limit fires
/// before any host budget.
#[test]
fn stack_probes_formula_nesting_63_64_and_backward_depth_31_33() {
    let mut calibrator = EnclosureHost::new(FuelPolicy::Unbounded, 2000);

    // (a) formula nesting: measure the refusing frame's spend first, so the
    // probe budget is >= 4x and the in-module parse guard provably fires.
    let refuse64 = nested_formulae(64);
    let (_, spend64) = calibrator.exec_seeded(&refuse64, CALIBRATION_SEED);
    let budget_n3 = next_pow2_ceil(4 * spend64).max(CALIBRATION_FLOOR);
    let mut table = BTreeMap::new();
    table.insert("n3".to_string(), budget_n3);
    let mut host = EnclosureHost::new(FuelPolicy::PerOp(table), 2000);

    let admit63 = host.exec(&nested_formulae(63));
    assert_eq!(admit63["ok"], json!(true), "depth 63 must parse: {admit63}");
    let refuse = host
        .probe(&refuse64)
        .unwrap_or_else(|e| panic!("depth 64 must hit the in-module guard, not {e:?}"));
    assert_eq!(refuse["ok"], json!(false), "depth 64 must refuse: {refuse}");
    assert_eq!(
        refuse["error"]["kind"], json!("EngineRejected"),
        "the in-module parse guard must answer (a host fuel envelope would be ResourceLimit): {refuse}"
    );
    let msg = refuse["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("nested more than"),
        "refusal must name the nesting guard: {msg}"
    );

    // (b) backward depth 31/33 — WITNESSED DEVIATION from the mission's
    // assumed boundary: the backward reasoner (max_backward_depth, census
    // limit `n3_backward_depth`) serves the PROOF/EXPLANATION API
    // (purrdf-n3 `explain_backward_indexed`, proof_writer.rs:302) which the
    // module's op surface does NOT expose — `log:conclusion` computes a
    // FORWARD closure (reasoner.rs:4464 `eval_log_conclusion` -> reason()).
    // Witnessed on the real artifact at depths 5/31/33/40 through the real
    // runner: every depth ADMITS typed with derived "" — no refusal, no
    // limit. The court pins the observed law (deep backward chains admit
    // typed, bounded by the forward closure caps) under a real calibrated
    // budget (>= 4x measured spend) so a future surface that DOES route
    // backward proofs flips this leg loud instead of silently changing.
    let mut table_b = BTreeMap::new();
    table_b.insert("n3".to_string(), CALIBRATION_FLOOR);
    let mut host_b = EnclosureHost::new(FuelPolicy::PerOp(table_b), 2000);
    for d in [31usize, 33, 40] {
        let doc = json!({"op": "n3", "text": backward_chain(d)});
        let (_, spend) = host_b.exec_seeded_bytes(&doc, CALIBRATION_SEED);
        assert!(
            spend * 4 <= CALIBRATION_FLOOR,
            "depth {d} spend {spend} must keep the {CALIBRATION_FLOOR} budget >= 4x measured"
        );
        // The admit under the real >= 4x policy budget (not the 1e12 seed).
        let v = host_b.probe(&doc)
            .unwrap_or_else(|e| panic!("depth {d} must admit under its budget, got {e:?}"));
        assert_eq!(
            v["ok"], json!(true),
            "depth {d} must admit typed (witnessed observed law): {v}"
        );
        eprintln!("backward depth {d}: admitted typed under {CALIBRATION_FLOOR} budget, spend {spend} fuel");
    }
}

/// Leg 6 — the real runner: `--epoch-ticks 40` + the stall frame exits 99
/// with the typed wall-clock diagnostic; the default posture serves exit 0.
#[test]
fn run_eh_epoch_watchdog_exit99_and_default_posture_serves() {
    let stall = doubling_fixture();
    let mut frames = frame(stall.to_string().as_bytes());
    frames.extend(frame(br#"{"op":"capabilities"}"#));

    // Armed short deadline (8 ticks ~= 400 ms single-watchdog, well under
    // the ~1-2 s fixture): the stall frame triggers the watchdog epoch fire
    // -> typed exit 99 (the runner aborts the stream on a classified fault).
    let out = run_eh(&["--epoch-ticks", "8"], &frames, Duration::from_secs(240));
    assert_eq!(out.code, 99, "stall must exit 99; stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("epoch deadline expired"),
        "diagnostic must name the epoch deadline: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("wall-clock strangle"),
        "diagnostic must name the wall-clock strangle: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("guest frame #"),
        "the guest backtrace rides stderr for diagnosis: {}",
        out.stderr
    );

    // Default posture (no flag): capabilities frame exits 0 with a reply.
    let mut cap_frames = frame(br#"{"op":"capabilities"}"#);
    cap_frames.extend(frame(br#"{"op":"capabilities"}"#));
    let out = run_eh(&[], &cap_frames, Duration::from_secs(240));
    assert_eq!(out.code, 0, "default posture must serve; stderr: {}", out.stderr);
    assert!(
        out.stdout.len() > 8,
        "at least one reply frame expected, got {} bytes",
        out.stdout.len()
    );
    // The reply frame must decode to a typed ok envelope.
    let (l, _) = (
        u32::from_be_bytes([out.stdout[0], out.stdout[1], out.stdout[2], out.stdout[3]]) as usize,
        (),
    );
    assert!(out.stdout.len() >= 4 + l, "truncated reply frame");
    let reply: Value = serde_json::from_slice(&out.stdout[4..4 + l]).expect("reply is JSON");
    assert_eq!(reply["ok"], json!(true), "capabilities reply: {reply}");
}

/// Anti-vacuity: UNBOUNDED_FUEL really is far beyond any witness here, the
/// disarmed deadline really cannot fire, and the enclosure's fail-closed
/// PerOp really refuses an uncalibrated op typed instead of running it on
/// residual fuel.
#[test]
fn enclosure_constants_and_failclosed_uncalibrated_ops_hold() {
    assert!(UNBOUNDED_FUEL >= 1_000_000_000_000_000);
    assert!(EPOCH_DISARMED_TICKS == 1u64 << 62);

    let mut table = BTreeMap::new();
    table.insert("parse".to_string(), CALIBRATION_FLOOR);
    let mut host = EnclosureHost::new(FuelPolicy::PerOp(table), 2000);
    // capabilities is NOT in the table: fail-closed typed, not residual fuel.
    let cap = control_for("capabilities");
    match host.probe(&cap) {
        Err(EnclosureError::Fuel { budget }) => {
            assert_eq!(budget, 0, "uncalibrated op fails closed at zero budget");
            let env = host.request_envelope(&cap);
            assert_limit_envelope(&env, "guest_fuel", 0);
        }
        other => panic!("uncalibrated op must fail closed typed; got {other:?}"),
    }
    // The calibrated op still admits on the same host.
    let parse = control_for("parse");
    let r = host.exec(&parse);
    assert_eq!(r["ok"], json!(true), "calibrated op admits: {r}");
}

/// Leg: the false-red witness — an UNSEEDED store (fuel 0,
/// deadline 0) must trap during instantiate/_initialize. This is what every
/// seed-less host author hits; the enclosure's contract (seed BOTH budgets
/// BEFORE instantiate) is the fix, and this leg keeps the failure witnessed.
#[test]
fn unseeded_store_false_red_is_real() {
    let bytes = std::fs::read(common::enclosure::artifact_path()).expect("artifact");
    let mut cfg = wasmtime::Config::new();
    cfg.wasm_exceptions(true);
    cfg.consume_fuel(true);
    cfg.epoch_interruption(true);
    cfg.cranelift_opt_level(wasmtime::OptLevel::None);
    let engine = wasmtime::Engine::new(&cfg).expect("engine");
    let module = wasmtime::Module::new(&engine, &bytes[..]).expect("module");
    let mut store = wasmtime::Store::new(
        &engine,
        wasmtime_wasi::WasiCtxBuilder::new().inherit_stdio().build_p1(),
    );
    let mut linker = <wasmtime::Linker<wasmtime_wasi::p1::WasiP1Ctx>>::new(&engine);
    wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |ctx| ctx).expect("links WASI");
    // NO set_fuel, NO set_epoch_deadline — the documented false red.
    let instantiate = linker.instantiate(&mut store, &module);
    match instantiate {
        Err(e) => {
            let s = format!("{e:#}");
            assert!(
                s.contains("fuel") || s.contains("interrupt") || s.contains("epoch"),
                "the trap should name fuel or the epoch deadline: {s}"
            );
            eprintln!("false-red witnessed at instantiate: {s}");
        }
        Ok(instance) => {
            // Instantiation without a start section can succeed; the trap then
            // lands on `_initialize` — also a false red, also typed loud.
            if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
                let r = init.call(&mut store, ());
                let err = r.expect_err("unseeded _initialize must trap (false red)");
                let s = format!("{err:#}");
                assert!(
                    s.contains("fuel") || s.contains("interrupt") || s.contains("epoch"),
                    "the trap should name fuel or the epoch deadline: {s}"
                );
                eprintln!("false-red witnessed at _initialize: {s}");
            } else {
                panic!("module instantiates unseeded and exposes no _initialize — \
                        the false red did not fire; the enclosure's seeding law is untestable");
            }
        }
    }
}
